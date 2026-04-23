//! Notification Manager Service
//!
//! Pub/sub for GraphQL subscriptions. Runs two independent `tokio::sync::broadcast` channels:
//!
//! - **Race results** (`sender`) — bridged to PostgreSQL LISTEN/NOTIFY via `start_listener`,
//!   so race-result events propagate across backend instances.
//! - **Lobby updates** (`lobby_sender`) — in-process only. Lobby state changes fast and small,
//!   and the current deployment runs a single backend instance; cross-instance propagation
//!   can be added later by extending `start_listener` to also subscribe to a
//!   `lobby_updates` Postgres channel.

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

/// Notification payload for lobby updates (check-in / check-out)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LobbyNotification {
    pub group_id: Uuid,
}

/// Manager for handling GraphQL subscription notifications
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
