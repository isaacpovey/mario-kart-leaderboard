mod common;

use common::{fixtures, setup};
use mario_kart_leaderboard_backend::services::track_selection::select_tracks;
use std::collections::HashSet;

#[tokio::test]
async fn test_select_tracks_returns_correct_number() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let tracks = select_tracks(&ctx.pool, tournament.id, 4)
        .await
        .expect("Failed to select tracks");

    assert_eq!(tracks.len(), 4, "Should return exactly 4 tracks");
}

#[tokio::test]
async fn test_select_tracks_returns_unique_tracks() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let tracks = select_tracks(&ctx.pool, tournament.id, 8)
        .await
        .expect("Failed to select tracks");

    let track_ids: HashSet<_> = tracks.iter().map(|t| t.id).collect();
    assert_eq!(
        track_ids.len(),
        tracks.len(),
        "All selected tracks should be unique"
    );
}

#[tokio::test]
async fn test_select_tracks_first_match_no_history() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let tracks = select_tracks(&ctx.pool, tournament.id, 4)
        .await
        .expect("Failed to select tracks");

    assert_eq!(
        tracks.len(),
        4,
        "Should select tracks even with no match history"
    );
}

#[tokio::test]
async fn test_select_tracks_avoids_recently_used() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let first_tracks = select_tracks(&ctx.pool, tournament.id, 4)
        .await
        .expect("Failed to select first tracks");

    let test_match = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 4)
        .await
        .expect("Failed to create test match");

    for (i, track) in first_tracks.iter().enumerate() {
        sqlx::query(
            "INSERT INTO rounds (match_id, round_number, track_id, completed)
             VALUES ($1, $2, $3, false)",
        )
        .bind(test_match.id)
        .bind(i as i32 + 1)
        .bind(track.id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to insert round");
    }

    let second_tracks = select_tracks(&ctx.pool, tournament.id, 4)
        .await
        .expect("Failed to select second tracks");

    let first_track_ids: HashSet<_> = first_tracks.iter().map(|t| t.id).collect();
    let second_track_ids: HashSet<_> = second_tracks.iter().map(|t| t.id).collect();

    let overlap = first_track_ids.intersection(&second_track_ids).count();
    assert!(
        overlap < 4,
        "Second selection should prefer different tracks when possible"
    );
}

#[tokio::test]
async fn test_select_tracks_different_tournaments_isolated() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test tournaments");
    let tournament1 = &tournaments[0];
    let tournament2 = &tournaments[1];

    let tracks_for_t1 = select_tracks(&ctx.pool, tournament1.id, 4)
        .await
        .expect("Failed to select tracks for tournament 1");

    let test_match = fixtures::create_test_match(&ctx.pool, group.id, tournament1.id, 4)
        .await
        .expect("Failed to create test match");

    for (i, track) in tracks_for_t1.iter().enumerate() {
        sqlx::query(
            "INSERT INTO rounds (match_id, round_number, track_id, completed)
             VALUES ($1, $2, $3, false)",
        )
        .bind(test_match.id)
        .bind(i as i32 + 1)
        .bind(track.id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to insert round");
    }

    let tracks_for_t2 = select_tracks(&ctx.pool, tournament2.id, 4)
        .await
        .expect("Failed to select tracks for tournament 2");

    assert_eq!(
        tracks_for_t2.len(),
        4,
        "Tournament 2 should get tracks regardless of tournament 1's history"
    );
}

#[tokio::test]
async fn test_select_tracks_request_single_track() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let tracks = select_tracks(&ctx.pool, tournament.id, 1)
        .await
        .expect("Failed to select tracks");

    assert_eq!(tracks.len(), 1, "Should return exactly 1 track");
}

#[tokio::test]
async fn test_select_tracks_all_tracks_have_names() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let tracks = select_tracks(&ctx.pool, tournament.id, 4)
        .await
        .expect("Failed to select tracks");

    for track in &tracks {
        assert!(!track.name.is_empty(), "Track should have a name");
    }
}
