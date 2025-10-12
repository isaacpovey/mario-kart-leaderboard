mod common;

use async_graphql::Request;
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::graphql::context::GraphQLContext;

#[tokio::test]
async fn test_current_group_query() {
    let ctx = setup::setup_test_db().await;

    // Create a test group
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let query = r#"
        query {
            currentGroup {
                id
                name
            }
        }
    "#;

    let request = Request::new(query).data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id));
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let data = response.data.into_json().expect("Failed to parse response");
    let current_group = data
        .get("currentGroup")
        .expect("currentGroup field not found");

    assert_eq!(
        current_group.get("name").and_then(|v| v.as_str()),
        Some("Test Group")
    );
}

#[tokio::test]
async fn test_current_group_without_auth() {
    let ctx = setup::setup_test_db().await;

    let query = r#"
        query {
            currentGroup {
                id
                name
            }
        }
    "#;

    let request = Request::new(query).data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), None);
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
