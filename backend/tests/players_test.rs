mod common;

use async_graphql::{Request, Variables, value};
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::{
    graphql::context::GraphQLContext,
    services::notification_manager::NotificationManager,
};

#[tokio::test]
async fn test_players_query() {
    let ctx = setup::setup_test_db().await;

    // Create test data
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let _players = fixtures::create_test_players(&ctx.pool, group.id, 3)
        .await
        .expect("Failed to create test players");

    let query = r#"
        query {
            players {
                id
                name
            }
        }
    "#;

    let request = Request::new(query).data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let data = response.data.into_json().expect("Failed to parse response");
    let players = data
        .get("players")
        .expect("players field not found")
        .as_array()
        .expect("players should be an array");

    assert_eq!(players.len(), 3, "Expected 3 players");
}

#[tokio::test]
async fn test_create_player_mutation() {
    let ctx = setup::setup_test_db().await;

    // Create a test group first
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let query = r#"
        mutation CreatePlayer($name: String!) {
            createPlayer(name: $name) {
                id
                name
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "name": "Test Player"
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
    let player = data
        .get("createPlayer")
        .expect("createPlayer field not found");

    assert_eq!(
        player.get("name").and_then(|v| v.as_str()),
        Some("Test Player")
    );
}

#[tokio::test]
async fn test_create_player_without_auth() {
    let ctx = setup::setup_test_db().await;

    let query = r#"
        mutation CreatePlayer($name: String!) {
            createPlayer(name: $name) {
                id
                name
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "name": "Test Player"
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), None, NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected authentication error");
    assert!(
        response.errors[0]
            .message
            .contains("Authentication required"),
        "Expected Authentication required error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_disabled_player_excluded_from_players_query() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let players = fixtures::create_test_players(&ctx.pool, group.id, 3)
        .await
        .expect("Failed to create test players");

    fixtures::disable_player(&ctx.pool, players[0].id)
        .await
        .expect("Failed to disable player");

    let query = r#"
        query {
            players {
                id
                name
            }
        }
    "#;

    let request = Request::new(query).data(ctx.config.clone());
    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let data = response.data.into_json().expect("Failed to parse response");
    let players_data = data
        .get("players")
        .expect("players field not found")
        .as_array()
        .expect("players should be an array");

    assert_eq!(players_data.len(), 2, "Expected 2 active players");
    assert!(
        !players_data
            .iter()
            .any(|p| p.get("id").and_then(|v| v.as_str()) == Some(players[0].id.to_string().as_str())),
        "Disabled player should not appear in players query"
    );
}

#[tokio::test]
async fn test_disabled_player_still_visible_by_id() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let players = fixtures::create_test_players(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test players");
    let player = &players[0];

    fixtures::disable_player(&ctx.pool, player.id)
        .await
        .expect("Failed to disable player");

    let query = r#"
        query PlayerById($playerId: ID!) {
            playerById(playerId: $playerId) {
                id
                name
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "playerId": player.id.to_string()
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
    let player_data = data.get("playerById").expect("playerById field not found");

    assert_eq!(
        player_data.get("id").and_then(|v| v.as_str()),
        Some(player.id.to_string().as_str())
    );
    assert_eq!(
        player_data.get("name").and_then(|v| v.as_str()),
        Some(player.name.as_str())
    );
}

#[tokio::test]
async fn test_past_tournament_placings_ranks_among_all_players() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let players = fixtures::create_test_players(&ctx.pool, group.id, 4)
        .await
        .expect("Failed to create test players");

    let tournament = fixtures::create_test_tournament(
        &ctx.pool,
        group.id,
        chrono::NaiveDate::from_ymd_opt(2024, 1, 1),
        chrono::NaiveDate::from_ymd_opt(2024, 1, 7),
    )
    .await
    .expect("Failed to create test tournament");

    let elos = [1400, 1300, 1200, 1100];
    for (player, elo) in players.iter().zip(elos) {
        sqlx::query(
            "INSERT INTO player_tournament_scores
             (tournament_id, player_id, group_id, elo_rating)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(tournament.id)
        .bind(player.id)
        .bind(group.id)
        .bind(elo)
        .execute(&ctx.pool)
        .await
        .expect("Failed to insert tournament score");
    }

    sqlx::query("UPDATE tournaments SET winner = $1 WHERE id = $2")
        .bind(players[0].id)
        .bind(tournament.id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to mark tournament completed");

    let query = r#"
        query PlayerById($playerId: ID!) {
            playerById(playerId: $playerId) {
                id
                pastTournamentPlacings {
                    tournamentId
                    placing
                    totalPlayers
                }
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "playerId": players[1].id.to_string()
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
    let player_data = data.get("playerById").expect("playerById field not found");
    let placings = player_data
        .get("pastTournamentPlacings")
        .expect("pastTournamentPlacings field not found")
        .as_array()
        .expect("pastTournamentPlacings should be an array");

    assert_eq!(placings.len(), 1, "Expected one past placing");
    assert_eq!(
        placings[0].get("placing").and_then(|v| v.as_i64()),
        Some(2),
        "Second-highest ELO should place 2nd"
    );
    assert_eq!(
        placings[0].get("totalPlayers").and_then(|v| v.as_i64()),
        Some(4),
        "Total players should include all tournament participants"
    );
    assert_eq!(
        placings[0].get("tournamentId").and_then(|v| v.as_str()),
        Some(tournament.id.to_string().as_str())
    );
}
