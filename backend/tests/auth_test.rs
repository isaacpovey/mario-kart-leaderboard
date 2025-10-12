mod common;

use async_graphql::{value, Request, Variables};
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::{auth::verify_jwt, graphql::context::GraphQLContext};

#[tokio::test]
async fn test_login_query_success() {
    let ctx = setup::setup_test_db().await;

    // Create a test group with known credentials
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "test_password")
        .await
        .expect("Failed to create test group");

    let query = r#"
        query Login($groupId: ID!, $password: String!) {
            login(groupId: $groupId, password: $password)
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "groupId": group.id.to_string(),
            "password": "test_password"
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), None);
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let data = response.data.into_json().expect("Failed to parse response");
    let token = data
        .get("login")
        .expect("login field not found")
        .as_str()
        .expect("Token should be a string");

    // Verify the JWT token
    let claims = verify_jwt(token, &ctx.config.jwt_secret).expect("Failed to verify JWT");
    let token_group_id = claims
        .group_id()
        .expect("Failed to extract group_id from claims");
    assert_eq!(token_group_id, group.id);
}

#[tokio::test]
async fn test_login_query_invalid_credentials() {
    let ctx = setup::setup_test_db().await;

    // Create a test group
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "correct_password")
        .await
        .expect("Failed to create test group");

    let query = r#"
        query Login($groupId: ID!, $password: String!) {
            login(groupId: $groupId, password: $password)
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "groupId": group.id.to_string(),
            "password": "wrong_password"
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), None);
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected authentication error");
    assert!(
        response.errors[0].message.contains("Invalid credentials"),
        "Expected Invalid credentials error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_create_group_mutation() {
    let ctx = setup::setup_test_db().await;

    let query = r#"
        mutation CreateGroup($name: String!, $password: String!) {
            createGroup(name: $name, password: $password)
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "name": "Test Group",
            "password": "test_password"
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), None);
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let data = response.data.into_json().expect("Failed to parse response");
    let token = data
        .get("createGroup")
        .expect("createGroup field not found")
        .as_str()
        .expect("Token should be a string");

    assert!(!token.is_empty(), "Token should not be empty");
}
