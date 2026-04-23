//! Notification Manager Service
//!
//! This module provides a pub/sub system for GraphQL subscriptions using PostgreSQL LISTEN/NOTIFY.
//! It enables real-time updates across multiple backend instances by leveraging PostgreSQL's
//! built-in notification system combined with in-memory broadcast channels for active subscribers.

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
        let subscriber_count = self.sender.receiver_count();
        tracing::info!(
            "NOTIFY STEP 2: Broadcasting to {} active subscribers - match_id={}, tournament_id={}",
            subscriber_count,
            notification.match_id,
            notification.tournament_id
        );
        match self.sender.send(notification) {
            Ok(_) => tracing::info!("NOTIFY STEP 2: Successfully broadcast notification"),
            Err(e) => tracing::error!("NOTIFY STEP 2: Failed to broadcast - no receivers: {:?}", e),
        }
    }

    pub fn notify_lobby(&self, notification: LobbyNotification) {
        let subscriber_count = self.lobby_sender.receiver_count();
        tracing::info!(
            "LOBBY NOTIFY: Broadcasting to {} active subscribers - group_id={}",
            subscriber_count,
            notification.group_id
        );
        match self.lobby_sender.send(notification) {
            Ok(_) => tracing::info!("LOBBY NOTIFY: Successfully broadcast"),
            Err(e) => tracing::error!("LOBBY NOTIFY: Failed to broadcast - no receivers: {:?}", e),
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
