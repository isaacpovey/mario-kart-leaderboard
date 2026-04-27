mod common;

use std::time::Duration;

use mario_kart_leaderboard_backend::services::notification_manager::{
    LobbyNotification, NotificationManager, RaceResultNotification, SlotAssignmentNotification,
};
use tokio::time::timeout;
use uuid::Uuid;

use common::setup::setup_test_db;

/// Verifies the LISTEN/NOTIFY bridge: a `publish` round-trips through
/// PostgreSQL and reaches local subscribers via the `start_listener` task.
///
/// This is the regression test for the bug where publishers only fanned out
/// in-process and silently failed to reach subscribers on other instances.
#[tokio::test]
async fn publish_round_trips_to_local_subscriber() {
    let ctx = setup_test_db().await;

    let manager = NotificationManager::new();
    manager
        .clone()
        .start_listener(&ctx.config.database_url)
        .await
        .expect("failed to start listener");

    let mut receiver = manager.subscribe();

    let expected = RaceResultNotification {
        match_id: Uuid::new_v4(),
        tournament_id: Uuid::new_v4(),
        round_number: 3,
        group_id: Uuid::new_v4(),
    };

    manager
        .publish(&ctx.pool, expected.clone())
        .await
        .expect("publish failed");

    let received = timeout(Duration::from_secs(5), receiver.recv())
        .await
        .expect("timed out waiting for notification")
        .expect("receiver closed");

    assert_eq!(received.match_id, expected.match_id);
    assert_eq!(received.tournament_id, expected.tournament_id);
    assert_eq!(received.round_number, expected.round_number);
    assert_eq!(received.group_id, expected.group_id);
}

/// A second `NotificationManager` listening on the same database receives
/// events published by the first — proves cross-instance delivery, which
/// is the whole point of bridging through `pg_notify`.
#[tokio::test]
async fn publish_reaches_a_separate_manager_listening_on_the_same_db() {
    let ctx = setup_test_db().await;

    let publisher = NotificationManager::new();
    let subscriber = NotificationManager::new();

    subscriber
        .clone()
        .start_listener(&ctx.config.database_url)
        .await
        .expect("failed to start subscriber listener");

    let mut receiver = subscriber.subscribe();

    let expected = RaceResultNotification {
        match_id: Uuid::new_v4(),
        tournament_id: Uuid::new_v4(),
        round_number: 1,
        group_id: Uuid::new_v4(),
    };

    publisher
        .publish(&ctx.pool, expected.clone())
        .await
        .expect("publish failed");

    let received = timeout(Duration::from_secs(5), receiver.recv())
        .await
        .expect("timed out waiting for notification")
        .expect("receiver closed");

    assert_eq!(received.match_id, expected.match_id);
}

/// Lobby counterpart of `publish_round_trips_to_local_subscriber`: confirms the
/// LISTEN/NOTIFY bridge is wired up for the lobby channel as well.
#[tokio::test]
async fn publish_lobby_round_trips_to_local_subscriber() {
    let ctx = setup_test_db().await;

    let manager = NotificationManager::new();
    manager
        .clone()
        .start_listener(&ctx.config.database_url)
        .await
        .expect("failed to start listener");

    let mut receiver = manager.subscribe_lobby();

    let expected = LobbyNotification {
        group_id: Uuid::new_v4(),
    };

    manager
        .publish_lobby(&ctx.pool, expected.clone())
        .await
        .expect("publish_lobby failed");

    let received = timeout(Duration::from_secs(5), receiver.recv())
        .await
        .expect("timed out waiting for lobby notification")
        .expect("receiver closed");

    assert_eq!(received.group_id, expected.group_id);
}

/// A second `NotificationManager` receives lobby events published by the first,
/// proving cross-instance delivery for the lobby channel.
#[tokio::test]
async fn publish_lobby_reaches_a_separate_manager_listening_on_the_same_db() {
    let ctx = setup_test_db().await;

    let publisher = NotificationManager::new();
    let subscriber = NotificationManager::new();

    subscriber
        .clone()
        .start_listener(&ctx.config.database_url)
        .await
        .expect("failed to start subscriber listener");

    let mut receiver = subscriber.subscribe_lobby();

    let expected = LobbyNotification {
        group_id: Uuid::new_v4(),
    };

    publisher
        .publish_lobby(&ctx.pool, expected.clone())
        .await
        .expect("publish_lobby failed");

    let received = timeout(Duration::from_secs(5), receiver.recv())
        .await
        .expect("timed out waiting for lobby notification")
        .expect("receiver closed");

    assert_eq!(received.group_id, expected.group_id);
}

/// Slot-assignment counterpart of `publish_round_trips_to_local_subscriber`:
/// confirms the LISTEN/NOTIFY bridge is wired up for the slot-assignment
/// channel as well.
#[tokio::test]
async fn publish_slot_assignment_round_trips_to_local_subscriber() {
    let ctx = setup_test_db().await;

    let manager = NotificationManager::new();
    manager
        .clone()
        .start_listener(&ctx.config.database_url)
        .await
        .expect("failed to start listener");

    let mut receiver = manager.subscribe_slot_assignments();

    let expected = SlotAssignmentNotification {
        group_id: Uuid::new_v4(),
        match_id: Uuid::new_v4(),
        round_number: 2,
        slot_number: 5,
        player_id: Some(Uuid::new_v4()),
        source_client_id: "client-abc".to_string(),
    };

    manager
        .publish_slot_assignment(&ctx.pool, expected.clone())
        .await
        .expect("publish_slot_assignment failed");

    let received = timeout(Duration::from_secs(5), receiver.recv())
        .await
        .expect("timed out waiting for slot assignment notification")
        .expect("receiver closed");

    assert_eq!(received.group_id, expected.group_id);
    assert_eq!(received.match_id, expected.match_id);
    assert_eq!(received.round_number, expected.round_number);
    assert_eq!(received.slot_number, expected.slot_number);
    assert_eq!(received.player_id, expected.player_id);
    assert_eq!(received.source_client_id, expected.source_client_id);
}

/// A second `NotificationManager` receives slot-assignment events published
/// by the first, proving cross-instance delivery for the slot-assignment
/// channel.
#[tokio::test]
async fn publish_slot_assignment_reaches_a_separate_manager_listening_on_the_same_db() {
    let ctx = setup_test_db().await;

    let publisher = NotificationManager::new();
    let subscriber = NotificationManager::new();

    subscriber
        .clone()
        .start_listener(&ctx.config.database_url)
        .await
        .expect("failed to start subscriber listener");

    let mut receiver = subscriber.subscribe_slot_assignments();

    let expected = SlotAssignmentNotification {
        group_id: Uuid::new_v4(),
        match_id: Uuid::new_v4(),
        round_number: 1,
        slot_number: 3,
        player_id: None,
        source_client_id: "client-xyz".to_string(),
    };

    publisher
        .publish_slot_assignment(&ctx.pool, expected.clone())
        .await
        .expect("publish_slot_assignment failed");

    let received = timeout(Duration::from_secs(5), receiver.recv())
        .await
        .expect("timed out waiting for slot assignment notification")
        .expect("receiver closed");

    assert_eq!(received.match_id, expected.match_id);
    assert_eq!(received.slot_number, expected.slot_number);
    assert_eq!(received.player_id, expected.player_id);
    assert_eq!(received.source_client_id, expected.source_client_id);
}
