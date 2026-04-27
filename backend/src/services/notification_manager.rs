//! Notification Manager Service
//!
//! Pub/sub for GraphQL subscriptions. Three `tokio::sync::broadcast` channels:
//!
//! - **Race results** (`sender`) — bridged across backend instances via
//!   PostgreSQL LISTEN/NOTIFY. Publishers call [`NotificationManager::publish`].
//! - **Lobby updates** (`lobby_sender`) — bridged the same way via
//!   [`NotificationManager::publish_lobby`].
//! - **Slot assignments** (`slot_assignment_sender`) — currently in-process
//!   only; ephemeral grid-slot assertions with no DB persistence. Will be
//!   moved to the same pg_notify bridge in a follow-up.
//!
//! For the bridged channels, each publisher executes `pg_notify` on the
//! database. Every backend instance has a single [`PgListener`] open via
//! [`NotificationManager::start_listener`] that subscribes to both bridged
//! channels; on receipt it dispatches by channel name and fans the event out
//! to local subscribers through the corresponding broadcast channel. The
//! publishing instance receives its own notification through the same path
//! (~ms round-trip), so there is no separate self-delivery code path.

use crate::db::DbPool;
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tokio::sync::broadcast;
use uuid::Uuid;

const RACE_RESULTS_CHANNEL: &str = "race_results_updates";
const LOBBY_CHANNEL: &str = "lobby_updates";

/// Notification payload for race result updates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaceResultNotification {
    pub match_id: Uuid,
    pub tournament_id: Uuid,
    pub round_number: i32,
    pub group_id: Uuid,
}

/// Notification payload for grid slot assignment events.
///
/// Ephemeral: in-process broadcast only, no DB persistence. Carries the
/// asserted state of a single slot in a round of a match. The
/// `source_client_id` lets subscribers echo-filter their own publishes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotAssignmentNotification {
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub round_number: i32,
    pub slot_number: i32,
    pub player_id: Option<Uuid>,
    pub source_client_id: String,
}

/// Notification payload for lobby updates (check-in / check-out)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LobbyNotification {
    pub group_id: Uuid,
}

/// Manager for handling GraphQL subscription notifications
///
/// Holds `tokio::sync::broadcast` channels that deliver events to every local
/// subscriber. Cross-instance delivery is handled by the `publish_*` methods
/// (which execute `pg_notify`) and [`Self::start_listener`] (which receives via
/// `LISTEN` and re-broadcasts locally).
#[derive(Clone)]
pub struct NotificationManager {
    sender: broadcast::Sender<RaceResultNotification>,
    slot_assignment_sender: broadcast::Sender<SlotAssignmentNotification>,
    lobby_sender: broadcast::Sender<LobbyNotification>,
}

impl NotificationManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        let (slot_assignment_sender, _) = broadcast::channel(100);
        let (lobby_sender, _) = broadcast::channel(100);
        Self {
            sender,
            slot_assignment_sender,
            lobby_sender,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RaceResultNotification> {
        self.sender.subscribe()
    }

    /// Subscribe to slot assignment notifications.
    ///
    /// In-process only — these events are not bridged to PostgreSQL.
    pub fn subscribe_slot_assignments(&self) -> broadcast::Receiver<SlotAssignmentNotification> {
        self.slot_assignment_sender.subscribe()
    }

    /// Broadcast a slot assignment notification to all in-process subscribers.
    pub fn notify_slot_assignment(&self, notification: SlotAssignmentNotification) {
        tracing::debug!(
            group_id = %notification.group_id,
            match_id = %notification.match_id,
            round_number = notification.round_number,
            slot_number = notification.slot_number,
            subscribers = self.slot_assignment_sender.receiver_count(),
            "broadcasting slot assignment notification"
        );
        if let Err(e) = self.slot_assignment_sender.send(notification) {
            tracing::error!(
                "failed to broadcast slot assignment notification (no receivers): {:?}",
                e
            );
        }
    }

    pub fn subscribe_lobby(&self) -> broadcast::Receiver<LobbyNotification> {
        self.lobby_sender.subscribe()
    }

    /// Publish a race result notification to all backend instances.
    ///
    /// Executes `SELECT pg_notify(...)` on the supplied pool. Every backend
    /// instance with [`Self::start_listener`] running — including the caller —
    /// will receive the event via `LISTEN` and re-broadcast it to its local
    /// subscribers.
    ///
    /// Should be called *after* the originating transaction has committed:
    /// the notification is fire-and-observe, and re-firing it on rollback
    /// would mislead listeners.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails or the `pg_notify` query
    /// fails. Callers typically log and continue, since the underlying data
    /// has already been committed and a missed live update is recoverable
    /// (the next refetch / reconnect resyncs from the database).
    pub async fn publish(
        &self,
        pool: &DbPool,
        notification: RaceResultNotification,
    ) -> Result<()> {
        let payload = serde_json::to_string(&notification).map_err(|e| {
            AppError::Internal(format!("Failed to serialize notification payload: {e}"))
        })?;

        tracing::info!(
            "NOTIFY STEP 1.5: Publishing pg_notify on {} - match_id={}, tournament_id={}, round={}, group_id={}",
            RACE_RESULTS_CHANNEL,
            notification.match_id,
            notification.tournament_id,
            notification.round_number,
            notification.group_id,
        );

        sqlx::query("SELECT pg_notify($1, $2)")
            .bind(RACE_RESULTS_CHANNEL)
            .bind(&payload)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Publish a lobby notification to all backend instances.
    ///
    /// Mirror of [`Self::publish`] for the lobby channel; see its docs for
    /// semantics, ordering, and error handling.
    pub async fn publish_lobby(
        &self,
        pool: &DbPool,
        notification: LobbyNotification,
    ) -> Result<()> {
        let payload = serde_json::to_string(&notification).map_err(|e| {
            AppError::Internal(format!("Failed to serialize lobby notification payload: {e}"))
        })?;

        tracing::debug!(
            group_id = %notification.group_id,
            "publishing lobby pg_notify on {}",
            LOBBY_CHANNEL
        );

        sqlx::query("SELECT pg_notify($1, $2)")
            .bind(LOBBY_CHANNEL)
            .bind(&payload)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Broadcast a race result notification to local subscribers only.
    ///
    /// Used by the [`Self::start_listener`] task once an event has been
    /// received from PostgreSQL, and by tests that want to simulate event
    /// delivery without a real listener. Production code paths should call
    /// [`Self::publish`] instead so that other backend instances also receive
    /// the event.
    pub fn notify(&self, notification: RaceResultNotification) {
        let subscriber_count = self.sender.receiver_count();
        tracing::info!(
            "NOTIFY STEP 2: Broadcasting to {} active subscribers - match_id={}, tournament_id={}",
            subscriber_count,
            notification.match_id,
            notification.tournament_id
        );

        match self.sender.send(notification) {
            Ok(_) => {
                tracing::info!("NOTIFY STEP 2: Successfully broadcast notification");
            }
            Err(e) => {
                tracing::debug!(
                    "NOTIFY STEP 2: No local subscribers for notification: {:?}",
                    e
                );
            }
        }
    }

    /// Broadcast a lobby notification to local subscribers only.
    ///
    /// Counterpart to [`Self::notify`] for the lobby channel; called by the
    /// listener loop and by tests. Production code paths should call
    /// [`Self::publish_lobby`] instead.
    pub fn notify_lobby(&self, notification: LobbyNotification) {
        tracing::debug!(
            group_id = %notification.group_id,
            subscribers = self.lobby_sender.receiver_count(),
            "broadcasting lobby notification"
        );
        if let Err(e) = self.lobby_sender.send(notification) {
            tracing::error!("failed to broadcast lobby notification (no receivers): {:?}", e);
        }
    }

    /// Start listening to PostgreSQL NOTIFY events
    ///
    /// Spawns a background task that listens on both [`RACE_RESULTS_CHANNEL`]
    /// and [`LOBBY_CHANNEL`] and dispatches received notifications to the
    /// matching local broadcast channel via [`Self::notify`] /
    /// [`Self::notify_lobby`].
    pub async fn start_listener(self, database_url: &str) -> Result<()> {
        let mut listener = PgListener::connect(database_url).await?;
        listener
            .listen_all([RACE_RESULTS_CHANNEL, LOBBY_CHANNEL])
            .await?;

        tracing::info!(
            "PostgreSQL LISTEN started for channels: {}, {}",
            RACE_RESULTS_CHANNEL,
            LOBBY_CHANNEL
        );

        tokio::spawn(async move {
            loop {
                match listener.recv().await {
                    Ok(notification) => match notification.channel() {
                        RACE_RESULTS_CHANNEL => {
                            tracing::info!(
                                "NOTIFY STEP 2: PostgreSQL listener received notification payload: {}",
                                notification.payload()
                            );

                            match serde_json::from_str::<RaceResultNotification>(
                                notification.payload(),
                            ) {
                                Ok(data) => self.notify(data),
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to parse race result notification payload: {}",
                                        e
                                    );
                                }
                            }
                        }
                        LOBBY_CHANNEL => {
                            match serde_json::from_str::<LobbyNotification>(notification.payload())
                            {
                                Ok(data) => self.notify_lobby(data),
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to parse lobby notification payload: {}",
                                        e
                                    );
                                }
                            }
                        }
                        other => {
                            tracing::warn!(
                                "Received notification on unexpected channel: {}",
                                other
                            );
                        }
                    },
                    Err(e) => tracing::error!("Error receiving PostgreSQL notification: {}", e),
                }
            }
        });

        Ok(())
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}
