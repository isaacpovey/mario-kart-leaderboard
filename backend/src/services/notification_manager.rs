//! Notification Manager Service
//!
//! Pub/sub for GraphQL subscriptions. Two `tokio::sync::broadcast` channels:
//!
//! - **Race results** (`sender`) — bridged across backend instances via
//!   PostgreSQL LISTEN/NOTIFY. Publishers call [`NotificationManager::publish`],
//!   which executes `pg_notify` on the database. Every backend instance has a
//!   [`PgListener`] open via [`NotificationManager::start_listener`]; on receipt
//!   it fans the event out to local subscribers through the broadcast channel.
//!   The publishing instance receives its own notification through the same
//!   path (~ms round-trip), so there is no separate self-delivery code path.
//! - **Lobby updates** (`lobby_sender`) — in-process only. Lobby state changes
//!   fast and small, and the current deployment runs a single backend instance;
//!   cross-instance propagation can be added later by extending `start_listener`
//!   to also subscribe to a `lobby_updates` Postgres channel.

use crate::db::DbPool;
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tokio::sync::broadcast;
use uuid::Uuid;

const RACE_RESULTS_CHANNEL: &str = "race_results_updates";

/// Notification payload for race result updates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaceResultNotification {
    pub match_id: Uuid,
    pub tournament_id: Uuid,
    pub round_number: i32,
    pub group_id: Uuid,
}

/// Notification payload for lobby updates (check-in / check-out)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LobbyNotification {
    pub group_id: Uuid,
}

/// Manager for handling GraphQL subscription notifications
///
/// Holds `tokio::sync::broadcast` channels that deliver events to every local
/// subscriber. For the race-results channel, cross-instance delivery is handled
/// by [`Self::publish`] (which executes `pg_notify`) and [`Self::start_listener`]
/// (which receives via `LISTEN` and re-broadcasts locally). The lobby channel
/// is currently in-process only.
#[derive(Clone)]
pub struct NotificationManager {
    sender: broadcast::Sender<RaceResultNotification>,
    lobby_sender: broadcast::Sender<LobbyNotification>,
}

impl NotificationManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        let (lobby_sender, _) = broadcast::channel(100);
        Self { sender, lobby_sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RaceResultNotification> {
        self.sender.subscribe()
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
    /// Spawns a background task that listens on [`RACE_RESULTS_CHANNEL`] and
    /// re-broadcasts received notifications to all local subscribers via
    /// [`Self::notify`].
    pub async fn start_listener(self, database_url: &str) -> Result<()> {
        let mut listener = PgListener::connect(database_url).await?;
        listener.listen(RACE_RESULTS_CHANNEL).await?;

        tracing::info!(
            "PostgreSQL LISTEN started for {} channel",
            RACE_RESULTS_CHANNEL
        );

        tokio::spawn(async move {
            loop {
                match listener.recv().await {
                    Ok(notification) => {
                        tracing::info!(
                            "NOTIFY STEP 2: PostgreSQL listener received notification payload: {}",
                            notification.payload()
                        );

                        match serde_json::from_str::<RaceResultNotification>(notification.payload())
                        {
                            Ok(data) => self.notify(data),
                            Err(e) => {
                                tracing::error!("Failed to parse notification payload: {}", e);
                            }
                        }
                    }
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
