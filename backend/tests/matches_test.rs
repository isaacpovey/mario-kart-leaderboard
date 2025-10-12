mod common;

use async_graphql::{value, Request, Variables};
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::graphql::context::GraphQLContext;

#[tokio::test]
async fn test_matches_query() {
    let ctx = setup::setup_test_db().await;

    // Create test data
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    let query = r#"
        query Matches($tournamentId: ID!) {
            matches(tournamentId: $tournamentId) {
                id
                tournamentId
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string()
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
    let matches = data
        .get("matches")
        .expect("matches field not found")
        .as_array()
        .expect("matches should be an array");

    // Should be 0 matches as we haven't created any yet
    assert_eq!(matches.len(), 0);
}

#[tokio::test]
async fn test_matches_query_unauthorized() {
    let ctx = setup::setup_test_db().await;

    // Create test data for two different groups
    let group1 = fixtures::create_test_group(&ctx.pool, "Test Group 1", "password")
        .await
        .expect("Failed to create test group");

    let group2 = fixtures::create_test_group(&ctx.pool, "Test Group 2", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group1.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    let query = r#"
        query Matches($tournamentId: ID!) {
            matches(tournamentId: $tournamentId) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string()
        })))
        .data(ctx.config.clone());

    // Authenticate as group2, trying to access group1's tournament
    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group2.id));
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected unauthorized error");
    assert!(
        response.errors[0].message.contains("Unauthorized"),
        "Expected Unauthorized error, got: {}",
        response.errors[0].message
    );
}
