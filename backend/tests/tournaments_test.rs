mod common;

use async_graphql::{Request, Variables, value};
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::{
    graphql::context::GraphQLContext,
    services::notification_manager::NotificationManager,
};

#[tokio::test]
async fn test_tournaments_query() {
    let ctx = setup::setup_test_db().await;

    // Create test data
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let _tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test tournaments");

    let query = r#"
        query {
            tournaments {
                id
                startDate
                endDate
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
    let tournaments = data
        .get("tournaments")
        .expect("tournaments field not found")
        .as_array()
        .expect("tournaments should be an array");

    assert_eq!(tournaments.len(), 2, "Expected 2 tournaments");
}

#[tokio::test]
async fn test_create_tournament_mutation() {
    let ctx = setup::setup_test_db().await;

    // Create a test group first
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let query = r#"
        mutation CreateTournament($startDate: String, $endDate: String) {
            createTournament(startDate: $startDate, endDate: $endDate) {
                id
                startDate
                endDate
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "startDate": "2024-01-01",
            "endDate": "2024-01-07"
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
    let tournament = data
        .get("createTournament")
        .expect("createTournament field not found");

    assert_eq!(
        tournament.get("startDate").and_then(|v| v.as_str()),
        Some("2024-01-01")
    );
    assert_eq!(
        tournament.get("endDate").and_then(|v| v.as_str()),
        Some("2024-01-07")
    );
}

#[tokio::test]
async fn test_create_tournament_with_invalid_dates() {
    let ctx = setup::setup_test_db().await;

    // Create a test group first
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let query = r#"
        mutation CreateTournament($startDate: String, $endDate: String) {
            createTournament(startDate: $startDate, endDate: $endDate) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "startDate": "invalid-date"
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected date format error");
    assert!(
        response.errors[0]
            .message
            .contains("Invalid start date format"),
        "Expected date format error, got: {}",
        response.errors[0].message
    );
}
