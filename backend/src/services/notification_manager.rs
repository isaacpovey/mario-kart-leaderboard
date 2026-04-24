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

/// Manager for handling GraphQL subscription notifications
///
/// Uses a broadcast channel for in-memory pub/sub and PostgreSQL LISTEN
/// for cross-instance communication
#[derive(Clone)]
pub struct NotificationManager {
    sender: broadcast::Sender<RaceResultNotification>,
    slot_assignment_sender: broadcast::Sender<SlotAssignmentNotification>,
}

impl NotificationManager {
    /// Creates a new notification manager with a broadcast channel
    ///
    /// Channel capacity is set to 100 to handle bursts of notifications
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        let (slot_assignment_sender, _) = broadcast::channel(100);
        Self { sender, slot_assignment_sender }
    }

    /// Subscribe to race result notifications
    ///
    /// Returns a receiver that will get all future notifications
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
