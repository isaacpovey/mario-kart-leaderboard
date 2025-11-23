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

/// Manager for handling GraphQL subscription notifications
///
/// Uses a broadcast channel for in-memory pub/sub and PostgreSQL LISTEN
/// for cross-instance communication
#[derive(Clone)]
pub struct NotificationManager {
    sender: broadcast::Sender<RaceResultNotification>,
}

impl NotificationManager {
    /// Creates a new notification manager with a broadcast channel
    ///
    /// Channel capacity is set to 100 to handle bursts of notifications
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    /// Subscribe to race result notifications
    ///
    /// Returns a receiver that will get all future notifications
    pub fn subscribe(&self) -> broadcast::Receiver<RaceResultNotification> {
        self.sender.subscribe()
    }

    /// Manually notify subscribers (used for local broadcasting)
    ///
    /// This is called when the current instance publishes a notification
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
                tracing::error!("NOTIFY STEP 2: Failed to broadcast - no receivers: {:?}", e);
            }
        }
    }

    /// Start listening to PostgreSQL NOTIFY events
    ///
    /// Spawns a background task that listens to the `race_results_updates` channel
    /// and broadcasts received notifications to all local subscribers
    pub async fn start_listener(self, database_url: &str) -> Result<()> {
        let mut listener = PgListener::connect(database_url).await?;
        listener.listen("race_results_updates").await?;

        tracing::info!("PostgreSQL LISTEN started for race_results_updates channel");

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
                            Ok(data) => {
                                tracing::info!(
                                    "NOTIFY STEP 2: Parsed notification - match_id={}, tournament_id={}, round={}, group_id={}",
                                    data.match_id,
                                    data.tournament_id,
                                    data.round_number,
                                    data.group_id
                                );

                                match self.sender.send(data) {
                                    Ok(_) => {
                                        tracing::info!("NOTIFY STEP 2: Broadcast notification to {} subscribers", self.sender.receiver_count());
                                    }
                                    Err(e) => {
                                        tracing::error!("NOTIFY STEP 2: Failed to broadcast notification: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse notification payload: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error receiving PostgreSQL notification: {}", e);
                    }
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
