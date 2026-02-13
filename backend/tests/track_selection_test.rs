mod common;

use common::{fixtures, setup};
use mario_kart_leaderboard_backend::models;
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
    assert_eq!(
        overlap, 0,
        "Second selection should have zero overlap with first within the same cycle"
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

async fn persist_tracks_as_rounds(
    pool: &sqlx::PgPool,
    group_id: uuid::Uuid,
    tournament_id: uuid::Uuid,
    tracks: &[models::Track],
) {
    let test_match = fixtures::create_test_match(pool, group_id, tournament_id, tracks.len() as i32)
        .await
        .expect("Failed to create test match");

    for (i, track) in tracks.iter().enumerate() {
        sqlx::query(
            "INSERT INTO rounds (match_id, round_number, track_id, completed)
             VALUES ($1, $2, $3, false)",
        )
        .bind(test_match.id)
        .bind(i as i32 + 1)
        .bind(track.id)
        .execute(pool)
        .await
        .expect("Failed to insert round");
    }
}

#[tokio::test]
async fn test_full_cycle_no_duplicates() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let all_tracks = models::Track::find_all(&ctx.pool)
        .await
        .expect("Failed to fetch all tracks");
    let total_track_count = all_tracks.len();
    let rounds_per_match = 6;
    let matches_per_cycle = total_track_count / rounds_per_match;

    let mut cycle_track_ids: Vec<uuid::Uuid> = Vec::new();

    // Play a full cycle (5 matches x 6 rounds = 30 tracks)
    for _ in 0..matches_per_cycle {
        let tracks = select_tracks(&ctx.pool, tournament.id, rounds_per_match as i32)
            .await
            .expect("Failed to select tracks");

        assert_eq!(tracks.len(), rounds_per_match);

        let match_ids: HashSet<uuid::Uuid> = tracks.iter().map(|t| t.id).collect();
        assert_eq!(match_ids.len(), rounds_per_match, "Tracks within a match should be unique");

        // No overlap with previously selected tracks in this cycle
        let overlap: Vec<_> = cycle_track_ids
            .iter()
            .filter(|id| match_ids.contains(id))
            .collect();
        assert!(
            overlap.is_empty(),
            "Tracks should not repeat within a cycle, but found {} duplicates",
            overlap.len()
        );

        cycle_track_ids.extend(match_ids);
        persist_tracks_as_rounds(&ctx.pool, group.id, tournament.id, &tracks).await;
    }

    assert_eq!(
        cycle_track_ids.len(),
        total_track_count,
        "A full cycle should cover all {} tracks",
        total_track_count
    );
    let unique_cycle: HashSet<_> = cycle_track_ids.iter().collect();
    assert_eq!(
        unique_cycle.len(),
        total_track_count,
        "All tracks in a cycle should be unique"
    );

    // Start of a new cycle (6th match) — should still produce valid tracks
    let new_cycle_tracks = select_tracks(&ctx.pool, tournament.id, rounds_per_match as i32)
        .await
        .expect("Failed to select tracks for new cycle");

    assert_eq!(new_cycle_tracks.len(), rounds_per_match);
    let new_ids: HashSet<_> = new_cycle_tracks.iter().map(|t| t.id).collect();
    assert_eq!(new_ids.len(), rounds_per_match, "New cycle tracks should be unique");
}
