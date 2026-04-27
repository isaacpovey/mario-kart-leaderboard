//! Notification Manager Service
//!
//! Pub/sub for GraphQL subscriptions. Runs three independent `tokio::sync::broadcast` channels:
//!
//! - **Race results** (`sender`) — bridged to PostgreSQL LISTEN/NOTIFY via `start_listener`,
//!   so race-result events propagate across backend instances.
//! - **Lobby updates** (`lobby_sender`) — in-process only. Lobby state changes fast and small,
//!   and the current deployment runs a single backend instance; cross-instance propagation
//!   can be added later by extending `start_listener` to also subscribe to a
//!   `lobby_updates` Postgres channel.
//! - **Slot assignments** (`slot_assignment_sender`) — in-process only, ephemeral grid-slot
//!   assertions with no DB persistence.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tokio::sync::broadcast;
use uuid::Uuid;

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

    pub fn notify(&self, notification: RaceResultNotification) {
        tracing::debug!(
            match_id = %notification.match_id,
            tournament_id = %notification.tournament_id,
            subscribers = self.sender.receiver_count(),
            "broadcasting race result notification"
        );
        if let Err(e) = self.sender.send(notification) {
            tracing::error!("failed to broadcast race result notification (no receivers): {:?}", e);
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

    pub async fn start_listener(self, database_url: &str) -> Result<()> {
        let mut listener = PgListener::connect(database_url).await?;
        listener.listen("race_results_updates").await?;

        tracing::info!("PostgreSQL LISTEN started for race_results_updates channel");

        tokio::spawn(async move {
            loop {
                match listener.recv().await {
                    Ok(notification) => {
                        match serde_json::from_str::<RaceResultNotification>(notification.payload()) {
                            Ok(data) => {
                                if let Err(e) = self.sender.send(data) {
                                    tracing::error!("Failed to broadcast race notification: {}", e);
                                }
                            }
                            Err(e) => tracing::error!("Failed to parse race notification payload: {}", e),
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
