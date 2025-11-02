mod common;

use async_graphql::{value, Request, Variables};
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::graphql::context::GraphQLContext;

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

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id));
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

    assert_eq!(match_data.get("completed").and_then(|v| v.as_bool()), Some(false));

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

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id));
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let updated_players = sqlx::query_as::<_, (uuid::Uuid, i32)>(
        "SELECT id, elo_rating FROM players WHERE id = ANY($1)",
    )
    .bind(&[winner_id, loser_id])
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
        "Winner should gain ELO: initial={}, updated={}",
        winner_initial_elo,
        winner_new_elo
    );
    assert!(
        loser_new_elo < loser_initial_elo,
        "Loser should lose ELO: initial={}, updated={}",
        loser_initial_elo,
        loser_new_elo
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

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id));
    let response1 = ctx.schema.execute(request1.data(gql_ctx)).await;

    assert!(response1.errors.is_empty());
    let data1 = response1.data.into_json().expect("Failed to parse response");
    let match_data1 = data1.get("recordRoundResults").unwrap();
    assert_eq!(match_data1.get("completed").and_then(|v| v.as_bool()), Some(false));

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

    let gql_ctx2 = GraphQLContext::new(ctx.pool.clone(), Some(group.id));
    let response2 = ctx.schema.execute(request2.data(gql_ctx2)).await;

    assert!(response2.errors.is_empty());
    let data2 = response2.data.into_json().expect("Failed to parse response");
    let match_data2 = data2.get("recordRoundResults").unwrap();
    assert_eq!(match_data2.get("completed").and_then(|v| v.as_bool()), Some(true));

    let team_scores: Vec<(f64,)> = sqlx::query_as(
        "SELECT score FROM team_match_scores WHERE match_id = $1",
    )
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

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id));
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

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id));
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

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id));
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

    let gql_ctx2 = GraphQLContext::new(ctx.pool.clone(), Some(group.id));
    let response2 = ctx.schema.execute(request2.data(gql_ctx2)).await;

    assert!(!response2.errors.is_empty());
    assert!(response2.errors[0].message.contains("already completed"));
}
