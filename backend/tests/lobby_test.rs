mod common;

use async_graphql::{Request, Variables, value};
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::{
    graphql::context::GraphQLContext,
    services::notification_manager::NotificationManager,
};

#[tokio::test]
async fn test_check_in_player_adds_to_lobby() {
    let ctx = setup::setup_test_db().await;
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");
    let players = fixtures::create_test_players(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test players");
    let alice = &players[0];

    let mutation = r#"
        mutation CheckIn($playerId: ID!) {
            checkInPlayer(playerId: $playerId) {
                id
                name
            }
        }
    "#;

    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "playerId": alice.id.to_string()
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
    let lobby = data
        .get("checkInPlayer")
        .expect("checkInPlayer field not found")
        .as_array()
        .expect("checkInPlayer should be an array");

    assert_eq!(lobby.len(), 1, "Expected exactly one player in lobby");
    assert_eq!(
        lobby[0].get("id").and_then(|v| v.as_str()),
        Some(alice.id.to_string().as_str())
    );
}

#[tokio::test]
async fn test_check_in_player_idempotent() {
    let ctx = setup::setup_test_db().await;
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");
    let players = fixtures::create_test_players(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test players");
    let alice = &players[0];

    let mutation = r#"
        mutation CheckIn($playerId: ID!) {
            checkInPlayer(playerId: $playerId) { id }
        }
    "#;
    let vars = Variables::from_value(value!({ "playerId": alice.id.to_string() }));

    // First check-in
    let gql_ctx_1 =
        GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let r1 = ctx.schema
        .execute(Request::new(mutation).variables(vars.clone()).data(ctx.config.clone()).data(gql_ctx_1))
        .await;
    assert!(r1.errors.is_empty(), "first check-in errors: {:?}", r1.errors);

    // Second check-in (same player) — should still succeed and still show exactly one entry
    let gql_ctx_2 =
        GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let r2 = ctx.schema
        .execute(Request::new(mutation).variables(vars).data(ctx.config.clone()).data(gql_ctx_2))
        .await;
    assert!(r2.errors.is_empty(), "second check-in errors: {:?}", r2.errors);

    let data = r2.data.into_json().expect("Failed to parse response");
    let lobby = data.get("checkInPlayer").unwrap().as_array().unwrap();
    assert_eq!(lobby.len(), 1, "Expected exactly one player after double check-in");
}

#[tokio::test]
async fn test_check_in_rejects_cross_group_player() {
    let ctx = setup::setup_test_db().await;

    let group_a = fixtures::create_test_group(&ctx.pool, "Group A", "password")
        .await
        .expect("Failed to create group A");
    let group_b = fixtures::create_test_group(&ctx.pool, "Group B", "password")
        .await
        .expect("Failed to create group B");

    // Player belongs to group B
    let players_b = fixtures::create_test_players(&ctx.pool, group_b.id, 1)
        .await
        .expect("Failed to create players in group B");
    let outsider = &players_b[0];

    let mutation = r#"
        mutation CheckIn($playerId: ID!) {
            checkInPlayer(playerId: $playerId) { id }
        }
    "#;

    // Authenticated as group A, try to check in group B's player
    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "playerId": outsider.id.to_string()
        })))
        .data(ctx.config.clone());

    let gql_ctx =
        GraphQLContext::new(ctx.pool.clone(), Some(group_a.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        !response.errors.is_empty(),
        "Expected an error for cross-group check-in"
    );
    assert_eq!(
        response.errors[0].message, "Player not found",
        "Expected 'Player not found' error"
    );

    // Verify no row was inserted in either group's lobby
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM lobby_entries")
        .fetch_one(&ctx.pool)
        .await
        .expect("count query failed");
    assert_eq!(count.0, 0);
}
