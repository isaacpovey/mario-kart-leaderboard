mod common;

use common::{fixtures, setup};
use mario_kart_leaderboard_backend::services::tournament_completion::complete_tournament;
use uuid::Uuid;

async fn setup_tournament_with_data(
    pool: &sqlx::PgPool,
    group_id: Uuid,
    tournament_id: Uuid,
    player_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    let test_match = fixtures::create_test_match(pool, group_id, tournament_id, 2).await?;

    let teams = fixtures::create_test_teams(pool, group_id, test_match.id, 2).await?;

    let track_id: Uuid = sqlx::query_scalar("SELECT id FROM tracks LIMIT 1")
        .fetch_one(pool)
        .await?;

    for round_num in 1..=2 {
        sqlx::query(
            "INSERT INTO rounds (match_id, round_number, track_id, completed)
             VALUES ($1, $2, $3, true)",
        )
        .bind(test_match.id)
        .bind(round_num)
        .bind(track_id)
        .execute(pool)
        .await?;
    }

    for (i, player_id) in player_ids.iter().enumerate() {
        let team = &teams[i % 2];
        let team_num = (i % 2 + 1) as i32;

        for round_num in 1..=2 {
            sqlx::query(
                "INSERT INTO round_players (group_id, match_id, round_number, player_id, team_id, player_position)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(group_id)
            .bind(test_match.id)
            .bind(round_num)
            .bind(player_id)
            .bind(team.id)
            .bind(team_num)
            .execute(pool)
            .await?;
        }

        let position = (i + 1) as i32;
        let elo_change = 50 - (i as i32 * 25);

        for round_num in 1..=2 {
            sqlx::query(
                "INSERT INTO player_race_scores
                 (group_id, match_id, round_number, player_id, position,
                  all_time_elo_change, all_time_elo_after, tournament_elo_change, tournament_elo_after, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())",
            )
            .bind(group_id)
            .bind(test_match.id)
            .bind(round_num)
            .bind(player_id)
            .bind(position)
            .bind(elo_change)
            .bind(1200 + elo_change)
            .bind(elo_change)
            .bind(1200 + elo_change)
            .execute(pool)
            .await?;
        }

        sqlx::query(
            "INSERT INTO player_match_scores
             (group_id, match_id, player_id, position, elo_change, tournament_elo_change,
              tournament_elo_from_races, tournament_elo_from_contributions)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(group_id)
        .bind(test_match.id)
        .bind(player_id)
        .bind(position)
        .bind(elo_change * 2)
        .bind(elo_change * 2)
        .bind(elo_change * 2)
        .bind(0)
        .execute(pool)
        .await?;

        sqlx::query(
            "INSERT INTO player_tournament_scores
             (tournament_id, player_id, group_id, elo_rating)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(tournament_id)
        .bind(player_id)
        .bind(group_id)
        .bind(1200 + elo_change * 2)
        .execute(pool)
        .await?;
    }

    for (i, player_id) in player_ids.iter().enumerate() {
        if i > 0 {
            let teammate_id = player_ids[(i + 2) % player_ids.len()];
            let contribution = 5 + (i as i32);
            let source_elo_change = 50 - (i as i32 * 25);
            sqlx::query(
                "INSERT INTO player_teammate_elo_contributions
                 (match_id, round_number, source_player_id, beneficiary_player_id, source_tournament_elo_change, contribution_amount)
                 VALUES ($1, 1, $2, $3, $4, $5)",
            )
            .bind(test_match.id)
            .bind(player_id)
            .bind(teammate_id)
            .bind(source_elo_change)
            .bind(contribution)
            .execute(pool)
            .await?;
        }
    }

    sqlx::query("UPDATE matches SET completed = true WHERE id = $1")
        .bind(test_match.id)
        .execute(pool)
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_complete_tournament_happy_path() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 4)
        .await
        .expect("Failed to create test players");
    let player_ids: Vec<Uuid> = players.iter().map(|p| p.id).collect();

    setup_tournament_with_data(&ctx.pool, group.id, tournament.id, &player_ids)
        .await
        .expect("Failed to setup tournament data");

    let result = complete_tournament(&ctx.pool, tournament.id, group.id).await;

    assert!(result.is_ok(), "Tournament completion should succeed");
    let completed_tournament = result.unwrap();
    assert!(
        completed_tournament.winner.is_some(),
        "Winner should be set"
    );
}

#[tokio::test]
async fn test_complete_tournament_sets_correct_winner() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 4)
        .await
        .expect("Failed to create test players");
    let player_ids: Vec<Uuid> = players.iter().map(|p| p.id).collect();

    setup_tournament_with_data(&ctx.pool, group.id, tournament.id, &player_ids)
        .await
        .expect("Failed to setup tournament data");

    let result = complete_tournament(&ctx.pool, tournament.id, group.id).await;
    let completed_tournament = result.expect("Tournament completion should succeed");

    let highest_elo_player: (Uuid,) = sqlx::query_as(
        "SELECT player_id FROM player_tournament_scores
         WHERE tournament_id = $1
         ORDER BY elo_rating DESC LIMIT 1",
    )
    .bind(tournament.id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Should find highest ELO player");

    assert_eq!(
        completed_tournament.winner,
        Some(highest_elo_player.0),
        "Winner should be player with highest tournament ELO"
    );
}

#[tokio::test]
async fn test_complete_tournament_unauthorized() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let other_group = fixtures::create_test_group(&ctx.pool, "Other Group", "password")
        .await
        .expect("Failed to create other group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 4)
        .await
        .expect("Failed to create test players");
    let player_ids: Vec<Uuid> = players.iter().map(|p| p.id).collect();

    setup_tournament_with_data(&ctx.pool, group.id, tournament.id, &player_ids)
        .await
        .expect("Failed to setup tournament data");

    let result = complete_tournament(&ctx.pool, tournament.id, other_group.id).await;

    assert!(result.is_err(), "Should fail for unauthorized group");
}

#[tokio::test]
async fn test_complete_tournament_already_completed() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 4)
        .await
        .expect("Failed to create test players");
    let player_ids: Vec<Uuid> = players.iter().map(|p| p.id).collect();

    setup_tournament_with_data(&ctx.pool, group.id, tournament.id, &player_ids)
        .await
        .expect("Failed to setup tournament data");

    let first_result = complete_tournament(&ctx.pool, tournament.id, group.id).await;
    assert!(first_result.is_ok(), "First completion should succeed");

    let second_result = complete_tournament(&ctx.pool, tournament.id, group.id).await;
    assert!(
        second_result.is_err(),
        "Second completion should fail (already completed)"
    );
}

#[tokio::test]
async fn test_complete_tournament_creates_stats() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 4)
        .await
        .expect("Failed to create test players");
    let player_ids: Vec<Uuid> = players.iter().map(|p| p.id).collect();

    setup_tournament_with_data(&ctx.pool, group.id, tournament.id, &player_ids)
        .await
        .expect("Failed to setup tournament data");

    let _ = complete_tournament(&ctx.pool, tournament.id, group.id)
        .await
        .expect("Tournament completion should succeed");

    let stat_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tournament_stats WHERE tournament_id = $1",
    )
    .bind(tournament.id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Should count stats");

    assert!(
        stat_count.0 > 0,
        "Tournament stats should be created after completion"
    );
}

#[tokio::test]
async fn test_complete_tournament_not_found() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let fake_tournament_id = Uuid::new_v4();

    let result = complete_tournament(&ctx.pool, fake_tournament_id, group.id).await;

    assert!(result.is_err(), "Should fail for non-existent tournament");
}
