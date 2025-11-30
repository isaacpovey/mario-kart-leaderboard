mod common;

use async_graphql::{Request, Variables, value};
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::{
    graphql::context::GraphQLContext,
    services::notification_manager::NotificationManager,
};

#[tokio::test]
async fn test_record_round_results_basic() {
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

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let teams = fixtures::create_test_teams(&ctx.pool, group.id, match_record.id, 2)
        .await
        .expect("Failed to create test teams");

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 2)
        .await
        .expect("Failed to create test rounds");

    fixtures::add_players_to_round(
        &ctx.pool,
        group.id,
        match_record.id,
        1,
        teams[0].id,
        &players.iter().map(|p| p.id).collect::<Vec<_>>(),
    )
    .await
    .expect("Failed to add players to round");

    let mutation = r#"
        mutation RecordResults($matchId: ID!, $roundNumber: Int!, $results: [PlayerResultInput!]!) {
            recordRoundResults(matchId: $matchId, roundNumber: $roundNumber, results: $results) {
                id
                completed
            }
        }
    "#;

    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string(),
            "roundNumber": 1,
            "results": [
                {"playerId": players[0].id.to_string(), "position": 1},
                {"playerId": players[1].id.to_string(), "position": 5},
                {"playerId": players[2].id.to_string(), "position": 10},
                {"playerId": players[3].id.to_string(), "position": 15},
            ]
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let data = response.data.into_json().expect("Failed to parse response");
    let match_data = data
        .get("recordRoundResults")
        .expect("recordRoundResults field not found");

    assert_eq!(
        match_data.get("completed").and_then(|v| v.as_bool()),
        Some(false)
    );

    let race_scores: Vec<(i32,)> = sqlx::query_as(
        "SELECT position FROM player_race_scores
         WHERE match_id = $1 AND round_number = 1
         ORDER BY position",
    )
    .bind(match_record.id)
    .fetch_all(&ctx.pool)
    .await
    .expect("Failed to fetch race scores");

    assert_eq!(race_scores.len(), 4);
    assert_eq!(race_scores[0].0, 1);
    assert_eq!(race_scores[1].0, 5);
    assert_eq!(race_scores[2].0, 10);
    assert_eq!(race_scores[3].0, 15);
}

#[tokio::test]
async fn test_record_round_results_updates_elo() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test players");

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 1)
        .await
        .expect("Failed to create test match");

    let teams = fixtures::create_test_teams(&ctx.pool, group.id, match_record.id, 1)
        .await
        .expect("Failed to create test teams");

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 1)
        .await
        .expect("Failed to create test rounds");

    fixtures::add_players_to_round(
        &ctx.pool,
        group.id,
        match_record.id,
        1,
        teams[0].id,
        &players.iter().map(|p| p.id).collect::<Vec<_>>(),
    )
    .await
    .expect("Failed to add players to round");

    let mutation = r#"
        mutation RecordResults($matchId: ID!, $roundNumber: Int!, $results: [PlayerResultInput!]!) {
            recordRoundResults(matchId: $matchId, roundNumber: $roundNumber, results: $results) {
                id
            }
        }
    "#;

    let winner_id = players[0].id;
    let loser_id = players[1].id;
    let winner_initial_elo = players[0].elo_rating;
    let loser_initial_elo = players[1].elo_rating;

    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string(),
            "roundNumber": 1,
            "results": [
                {"playerId": winner_id.to_string(), "position": 1},
                {"playerId": loser_id.to_string(), "position": 2},
            ]
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let updated_players = sqlx::query_as::<_, (uuid::Uuid, i32)>(
        "SELECT id, elo_rating FROM players WHERE id = ANY($1)",
    )
    .bind([winner_id, loser_id])
    .fetch_all(&ctx.pool)
    .await
    .expect("Failed to fetch updated players");

    let winner_new_elo = updated_players
        .iter()
        .find(|(id, _)| *id == winner_id)
        .map(|(_, elo)| *elo)
        .expect("Winner not found");

    let loser_new_elo = updated_players
        .iter()
        .find(|(id, _)| *id == loser_id)
        .map(|(_, elo)| *elo)
        .expect("Loser not found");

    assert!(
        winner_new_elo > winner_initial_elo,
        "Winner (1st place) should gain ELO: initial={}, updated={}",
        winner_initial_elo,
        winner_new_elo
    );

    // In a 24-player race, finishing 2nd means beating 22 CPU opponents (ELO 1000)
    // and losing to only 1 human opponent (ELO 1200). Since beating 22 lower-rated
    // opponents provides more ELO gain than losing to 1 equal-rated opponent,
    // 2nd place can actually gain ELO. This is mathematically correct behavior.
    assert!(
        loser_new_elo != loser_initial_elo,
        "2nd place ELO should change: initial={}, updated={}",
        loser_initial_elo,
        loser_new_elo
    );

    // Verify both players' ratings are reasonable
    assert!(
        winner_new_elo > loser_new_elo,
        "Winner should have higher ELO than 2nd place after race: winner={}, loser={}",
        winner_new_elo,
        loser_new_elo
    );
}

#[tokio::test]
async fn test_record_round_results_with_multiple_players_and_varied_elos() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 5)
        .await
        .expect("Failed to create test players");

    sqlx::query("UPDATE players SET elo_rating = $1 WHERE id = $2")
        .bind(1400)
        .bind(players[0].id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to update player 0 ELO");

    sqlx::query("UPDATE players SET elo_rating = $1 WHERE id = $2")
        .bind(1200)
        .bind(players[1].id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to update player 1 ELO");

    sqlx::query("UPDATE players SET elo_rating = $1 WHERE id = $2")
        .bind(1200)
        .bind(players[2].id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to update player 2 ELO");

    sqlx::query("UPDATE players SET elo_rating = $1 WHERE id = $2")
        .bind(1100)
        .bind(players[3].id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to update player 3 ELO");

    sqlx::query("UPDATE players SET elo_rating = $1 WHERE id = $2")
        .bind(1000)
        .bind(players[4].id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to update player 4 ELO");

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 1)
        .await
        .expect("Failed to create test match");

    let teams = fixtures::create_test_teams(&ctx.pool, group.id, match_record.id, 1)
        .await
        .expect("Failed to create test teams");

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 1)
        .await
        .expect("Failed to create test rounds");

    fixtures::add_players_to_round(
        &ctx.pool,
        group.id,
        match_record.id,
        1,
        teams[0].id,
        &players.iter().map(|p| p.id).collect::<Vec<_>>(),
    )
    .await
    .expect("Failed to add players to round");

    let mutation = r#"
        mutation RecordResults($matchId: ID!, $roundNumber: Int!, $results: [PlayerResultInput!]!) {
            recordRoundResults(matchId: $matchId, roundNumber: $roundNumber, results: $results) {
                id
            }
        }
    "#;

    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string(),
            "roundNumber": 1,
            "results": [
                {"playerId": players[0].id.to_string(), "position": 1},
                {"playerId": players[1].id.to_string(), "position": 3},
                {"playerId": players[2].id.to_string(), "position": 10},
                {"playerId": players[3].id.to_string(), "position": 15},
                {"playerId": players[4].id.to_string(), "position": 20},
            ]
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let updated_players = sqlx::query_as::<_, (uuid::Uuid, i32)>(
        "SELECT id, elo_rating FROM players WHERE id = ANY($1)",
    )
    .bind(players.iter().map(|p| p.id).collect::<Vec<_>>())
    .fetch_all(&ctx.pool)
    .await
    .expect("Failed to fetch updated players");

    let get_new_elo = |player_id: uuid::Uuid| -> i32 {
        updated_players
            .iter()
            .find(|(id, _)| *id == player_id)
            .map(|(_, elo)| *elo)
            .expect("Player not found")
    };

    let p0_new_elo = get_new_elo(players[0].id);
    let p1_new_elo = get_new_elo(players[1].id);
    let p2_new_elo = get_new_elo(players[2].id);
    let p3_new_elo = get_new_elo(players[3].id);
    let p4_new_elo = get_new_elo(players[4].id);

    assert!(
        p0_new_elo > 1400,
        "1st place (started at 1400) should gain ELO: {}",
        p0_new_elo
    );

    assert!(
        p1_new_elo > 1200,
        "3rd place (started at 1200) should gain ELO: {}",
        p1_new_elo
    );

    // Note: The higher-rated player (1400 ELO) finishing 1st gains LESS ELO
    // than the mid-rated player (1200 ELO) finishing 3rd, because the 1400
    // player is EXPECTED to beat lower-rated opponents. This is correct ELO behavior.

    assert!(
        p4_new_elo < 1000,
        "20th place (started at 1000) should lose ELO: {}",
        p4_new_elo
    );

    assert!(
        p0_new_elo > p1_new_elo
            && p1_new_elo > p2_new_elo
            && p2_new_elo > p3_new_elo
            && p3_new_elo > p4_new_elo,
        "Final rankings should maintain order: {} > {} > {} > {} > {}",
        p0_new_elo,
        p1_new_elo,
        p2_new_elo,
        p3_new_elo,
        p4_new_elo
    );
}

#[tokio::test]
async fn test_record_round_results_completes_match_when_all_rounds_done() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test players");

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let teams = fixtures::create_test_teams(&ctx.pool, group.id, match_record.id, 1)
        .await
        .expect("Failed to create test teams");

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 2)
        .await
        .expect("Failed to create test rounds");

    for round_num in 1..=2 {
        fixtures::add_players_to_round(
            &ctx.pool,
            group.id,
            match_record.id,
            round_num,
            teams[0].id,
            &players.iter().map(|p| p.id).collect::<Vec<_>>(),
        )
        .await
        .expect("Failed to add players to round");
    }

    let mutation = r#"
        mutation RecordResults($matchId: ID!, $roundNumber: Int!, $results: [PlayerResultInput!]!) {
            recordRoundResults(matchId: $matchId, roundNumber: $roundNumber, results: $results) {
                id
                completed
            }
        }
    "#;

    let request1 = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string(),
            "roundNumber": 1,
            "results": [
                {"playerId": players[0].id.to_string(), "position": 1},
                {"playerId": players[1].id.to_string(), "position": 2},
            ]
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response1 = ctx.schema.execute(request1.data(gql_ctx)).await;

    assert!(response1.errors.is_empty());
    let data1 = response1
        .data
        .into_json()
        .expect("Failed to parse response");
    let match_data1 = data1.get("recordRoundResults").unwrap();
    assert_eq!(
        match_data1.get("completed").and_then(|v| v.as_bool()),
        Some(false)
    );

    let request2 = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string(),
            "roundNumber": 2,
            "results": [
                {"playerId": players[1].id.to_string(), "position": 1},
                {"playerId": players[0].id.to_string(), "position": 2},
            ]
        })))
        .data(ctx.config.clone());

    let gql_ctx2 = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response2 = ctx.schema.execute(request2.data(gql_ctx2)).await;

    assert!(response2.errors.is_empty());
    let data2 = response2
        .data
        .into_json()
        .expect("Failed to parse response");
    let match_data2 = data2.get("recordRoundResults").unwrap();
    assert_eq!(
        match_data2.get("completed").and_then(|v| v.as_bool()),
        Some(true)
    );

    let team_scores: Vec<(i32,)> =
        sqlx::query_as("SELECT score FROM teams WHERE match_id = $1")
            .bind(match_record.id)
            .fetch_all(&ctx.pool)
            .await
            .expect("Failed to fetch team scores");

    assert_eq!(team_scores.len(), 1);
}

#[tokio::test]
async fn test_record_round_results_validation_invalid_position() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test players");

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 1)
        .await
        .expect("Failed to create test match");

    let teams = fixtures::create_test_teams(&ctx.pool, group.id, match_record.id, 1)
        .await
        .expect("Failed to create test teams");

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 1)
        .await
        .expect("Failed to create test rounds");

    fixtures::add_players_to_round(
        &ctx.pool,
        group.id,
        match_record.id,
        1,
        teams[0].id,
        &players.iter().map(|p| p.id).collect::<Vec<_>>(),
    )
    .await
    .expect("Failed to add players to round");

    let mutation = r#"
        mutation RecordResults($matchId: ID!, $roundNumber: Int!, $results: [PlayerResultInput!]!) {
            recordRoundResults(matchId: $matchId, roundNumber: $roundNumber, results: $results) {
                id
            }
        }
    "#;

    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string(),
            "roundNumber": 1,
            "results": [
                {"playerId": players[0].id.to_string(), "position": 25},
                {"playerId": players[1].id.to_string(), "position": 2},
            ]
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty());
    assert!(response.errors[0].message.contains("between 1 and 24"));
}

#[tokio::test]
async fn test_record_round_results_validation_duplicate_position() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test players");

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 1)
        .await
        .expect("Failed to create test match");

    let teams = fixtures::create_test_teams(&ctx.pool, group.id, match_record.id, 1)
        .await
        .expect("Failed to create test teams");

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 1)
        .await
        .expect("Failed to create test rounds");

    fixtures::add_players_to_round(
        &ctx.pool,
        group.id,
        match_record.id,
        1,
        teams[0].id,
        &players.iter().map(|p| p.id).collect::<Vec<_>>(),
    )
    .await
    .expect("Failed to add players to round");

    let mutation = r#"
        mutation RecordResults($matchId: ID!, $roundNumber: Int!, $results: [PlayerResultInput!]!) {
            recordRoundResults(matchId: $matchId, roundNumber: $roundNumber, results: $results) {
                id
            }
        }
    "#;

    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string(),
            "roundNumber": 1,
            "results": [
                {"playerId": players[0].id.to_string(), "position": 1},
                {"playerId": players[1].id.to_string(), "position": 1},
            ]
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty());
    assert!(response.errors[0].message.contains("Duplicate positions"));
}

#[tokio::test]
async fn test_record_round_results_validation_already_completed() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test players");

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 1)
        .await
        .expect("Failed to create test match");

    let teams = fixtures::create_test_teams(&ctx.pool, group.id, match_record.id, 1)
        .await
        .expect("Failed to create test teams");

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 1)
        .await
        .expect("Failed to create test rounds");

    fixtures::add_players_to_round(
        &ctx.pool,
        group.id,
        match_record.id,
        1,
        teams[0].id,
        &players.iter().map(|p| p.id).collect::<Vec<_>>(),
    )
    .await
    .expect("Failed to add players to round");

    let mutation = r#"
        mutation RecordResults($matchId: ID!, $roundNumber: Int!, $results: [PlayerResultInput!]!) {
            recordRoundResults(matchId: $matchId, roundNumber: $roundNumber, results: $results) {
                id
            }
        }
    "#;

    let request1 = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string(),
            "roundNumber": 1,
            "results": [
                {"playerId": players[0].id.to_string(), "position": 1},
                {"playerId": players[1].id.to_string(), "position": 2},
            ]
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response1 = ctx.schema.execute(request1.data(gql_ctx)).await;

    assert!(response1.errors.is_empty());

    let request2 = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string(),
            "roundNumber": 1,
            "results": [
                {"playerId": players[0].id.to_string(), "position": 2},
                {"playerId": players[1].id.to_string(), "position": 1},
            ]
        })))
        .data(ctx.config.clone());

    let gql_ctx2 = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response2 = ctx.schema.execute(request2.data(gql_ctx2)).await;

    assert!(!response2.errors.is_empty());
    assert!(response2.errors[0].message.contains("already completed"));
}

#[tokio::test]
async fn test_teammates_have_same_player_position() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");
    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 6)
        .await
        .expect("Failed to create test players");

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let teams = fixtures::create_test_teams(&ctx.pool, group.id, match_record.id, 2)
        .await
        .expect("Failed to create test teams");

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 2)
        .await
        .expect("Failed to create test rounds");

    fixtures::add_players_to_round(
        &ctx.pool,
        group.id,
        match_record.id,
        1,
        teams[0].id,
        &[players[0].id, players[1].id, players[2].id],
    )
    .await
    .expect("Failed to add players to round 1");

    fixtures::add_players_to_round(
        &ctx.pool,
        group.id,
        match_record.id,
        2,
        teams[1].id,
        &[players[3].id, players[4].id, players[5].id],
    )
    .await
    .expect("Failed to add players to round 2");

    let round_players: Vec<(uuid::Uuid, uuid::Uuid, i32)> = sqlx::query_as(
        "SELECT player_id, team_id, player_position
         FROM round_players
         WHERE match_id = $1
         ORDER BY team_id, player_id",
    )
    .bind(match_record.id)
    .fetch_all(&ctx.pool)
    .await
    .expect("Failed to fetch round players");

    let team0_positions: Vec<i32> = round_players
        .iter()
        .filter(|(_, team_id, _)| team_id == &teams[0].id)
        .map(|(_, _, pos)| *pos)
        .collect();

    let team1_positions: Vec<i32> = round_players
        .iter()
        .filter(|(_, team_id, _)| team_id == &teams[1].id)
        .map(|(_, _, pos)| *pos)
        .collect();

    assert_eq!(team0_positions.len(), 3);
    assert_eq!(team1_positions.len(), 3);

    assert!(
        team0_positions.iter().all(|&pos| pos == team0_positions[0]),
        "All players on team 0 should have the same player_position. Got: {:?}",
        team0_positions
    );

    assert!(
        team1_positions.iter().all(|&pos| pos == team1_positions[0]),
        "All players on team 1 should have the same player_position. Got: {:?}",
        team1_positions
    );

    assert_ne!(
        team0_positions[0], team1_positions[0],
        "Different teams should have different player_positions"
    );
}
