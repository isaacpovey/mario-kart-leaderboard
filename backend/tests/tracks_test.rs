mod common;

use async_graphql::Request;
use common::setup;
use mario_kart_leaderboard_backend::graphql::context::GraphQLContext;

#[tokio::test]
async fn test_tracks_query() {
    let ctx = setup::setup_test_db().await;

    let query = r#"
        query {
            tracks {
                id
                name
            }
        }
    "#;

    let request = Request::new(query).data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), None);
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Expected no errors: {:?}",
        response.errors
    );

    let data = response.data.into_json().expect("Failed to parse response");
    let tracks = data
        .get("tracks")
        .expect("tracks field not found")
        .as_array()
        .expect("tracks should be an array");

    // Tracks are populated via migrations, so we should have some
    assert!(!tracks.is_empty(), "Expected tracks from migrations");
}
