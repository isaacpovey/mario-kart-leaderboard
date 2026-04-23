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
