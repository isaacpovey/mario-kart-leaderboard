mod common;

use async_graphql::{Request, Variables, value};
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::{
    graphql::context::GraphQLContext,
    services::notification_manager::{LobbyNotification, NotificationManager},
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

#[tokio::test]
async fn test_check_out_player_removes_from_lobby() {
    let ctx = setup::setup_test_db().await;
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");
    let players = fixtures::create_test_players(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test players");
    let (alice, bob) = (&players[0], &players[1]);

    mario_kart_leaderboard_backend::models::LobbyEntry::check_in(&ctx.pool, group.id, alice.id)
        .await
        .expect("check_in alice");
    mario_kart_leaderboard_backend::models::LobbyEntry::check_in(&ctx.pool, group.id, bob.id)
        .await
        .expect("check_in bob");

    let mutation = r#"
        mutation CheckOut($playerId: ID!) {
            checkOutPlayer(playerId: $playerId) { id name }
        }
    "#;

    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "playerId": alice.id.to_string()
        })))
        .data(ctx.config.clone());

    let gql_ctx =
        GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(response.errors.is_empty(), "Expected no errors: {:?}", response.errors);

    let data = response.data.into_json().expect("Failed to parse response");
    let lobby = data.get("checkOutPlayer").unwrap().as_array().unwrap();
    assert_eq!(lobby.len(), 1, "Expected exactly bob remaining after checking out alice");
    assert_eq!(
        lobby[0].get("id").and_then(|v| v.as_str()),
        Some(bob.id.to_string().as_str())
    );
}

#[tokio::test]
async fn test_check_out_missing_player_is_noop() {
    let ctx = setup::setup_test_db().await;
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");
    let players = fixtures::create_test_players(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test players");
    let alice = &players[0];

    let mutation = r#"
        mutation CheckOut($playerId: ID!) {
            checkOutPlayer(playerId: $playerId) { id }
        }
    "#;

    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "playerId": alice.id.to_string()
        })))
        .data(ctx.config.clone());

    let gql_ctx =
        GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(response.errors.is_empty(), "Expected no errors: {:?}", response.errors);
    let data = response.data.into_json().expect("Failed to parse response");
    let lobby = data.get("checkOutPlayer").unwrap().as_array().unwrap();
    assert_eq!(lobby.len(), 0, "Expected empty lobby");
}

#[tokio::test]
async fn test_check_out_unknown_or_cross_group_player_is_noop() {
    // The docstring on `check_out_player` promises idempotency. A client may
    // hold a stale player id (deleted, or belonging to a different group)
    // and need to clear it from local UI by issuing a check-out. The DELETE
    // is bounded by the authenticated `group_id`, so this is safe; the
    // mutation must not error.
    let ctx = setup::setup_test_db().await;

    let group_a = fixtures::create_test_group(&ctx.pool, "Group A", "password")
        .await
        .expect("create group A");
    let group_b = fixtures::create_test_group(&ctx.pool, "Group B", "password")
        .await
        .expect("create group B");
    // A player that exists, but in a different group.
    let outsider =
        mario_kart_leaderboard_backend::models::Player::create(&ctx.pool, group_b.id, "Zoe")
            .await
            .expect("create outsider");
    // A player id that doesn't exist at all.
    let phantom_id = uuid::Uuid::new_v4();

    let mutation = r#"
        mutation CheckOut($playerId: ID!) {
            checkOutPlayer(playerId: $playerId) { id }
        }
    "#;

    for target in [outsider.id, phantom_id] {
        let request = Request::new(mutation)
            .variables(Variables::from_value(value!({
                "playerId": target.to_string()
            })))
            .data(ctx.config.clone());
        let gql_ctx =
            GraphQLContext::new(ctx.pool.clone(), Some(group_a.id), NotificationManager::new());
        let response = ctx.schema.execute(request.data(gql_ctx)).await;
        assert!(
            response.errors.is_empty(),
            "Expected no errors for {}, got: {:?}",
            target,
            response.errors
        );
        let data = response.data.into_json().expect("parse response");
        let lobby = data.get("checkOutPlayer").unwrap().as_array().unwrap();
        assert_eq!(lobby.len(), 0, "Expected group A's lobby to remain empty");
    }
}

#[tokio::test]
async fn test_group_lobby_ordered_by_name() {
    let ctx = setup::setup_test_db().await;
    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    // Create players with names that sort unambiguously.
    let carol = mario_kart_leaderboard_backend::models::Player::create(&ctx.pool, group.id, "Carol")
        .await
        .expect("Failed to create Carol");
    let alice = mario_kart_leaderboard_backend::models::Player::create(&ctx.pool, group.id, "Alice")
        .await
        .expect("Failed to create Alice");
    let bob = mario_kart_leaderboard_backend::models::Player::create(&ctx.pool, group.id, "Bob")
        .await
        .expect("Failed to create Bob");

    // Check in out of order to confirm the ORDER BY p.name ASC takes effect.
    mario_kart_leaderboard_backend::models::LobbyEntry::check_in(&ctx.pool, group.id, carol.id)
        .await
        .expect("check_in carol");
    mario_kart_leaderboard_backend::models::LobbyEntry::check_in(&ctx.pool, group.id, bob.id)
        .await
        .expect("check_in bob");
    mario_kart_leaderboard_backend::models::LobbyEntry::check_in(&ctx.pool, group.id, alice.id)
        .await
        .expect("check_in alice");

    let query = r#"
        query {
            currentGroup {
                lobby { id name }
            }
        }
    "#;

    let request = Request::new(query).data(ctx.config.clone());
    let gql_ctx =
        GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(response.errors.is_empty(), "Expected no errors: {:?}", response.errors);

    let data = response.data.into_json().expect("Failed to parse response");
    let lobby = data
        .get("currentGroup").unwrap()
        .get("lobby").unwrap()
        .as_array().unwrap();

    assert_eq!(lobby.len(), 3);
    assert_eq!(lobby[0].get("id").and_then(|v| v.as_str()), Some(alice.id.to_string().as_str()));
    assert_eq!(lobby[1].get("id").and_then(|v| v.as_str()), Some(bob.id.to_string().as_str()));
    assert_eq!(lobby[2].get("id").and_then(|v| v.as_str()), Some(carol.id.to_string().as_str()));
}

#[tokio::test]
async fn test_lobby_persists_across_match_creation() {
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

    for p in &players {
        mario_kart_leaderboard_backend::models::LobbyEntry::check_in(&ctx.pool, group.id, p.id)
            .await
            .expect("check_in");
    }

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let mutation = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!) {
            createMatchWithRounds(
                tournamentId: $tournamentId,
                playerIds: $playerIds,
                numRaces: $numRaces
            ) { id }
        }
    "#;

    let request = Request::new(mutation)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string(),
            "playerIds": player_ids,
            "numRaces": 4
        })))
        .data(ctx.config.clone());

    let gql_ctx =
        GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(response.errors.is_empty(), "Expected no errors: {:?}", response.errors);

    // Lobby must still contain all 4 players
    let lobby = mario_kart_leaderboard_backend::models::LobbyEntry::find_by_group_id(&ctx.pool, group.id)
        .await
        .expect("find_by_group_id");
    assert_eq!(lobby.len(), 4, "Lobby should be unchanged after match creation");
}

#[tokio::test]
async fn test_lobby_updated_subscription_fires_on_check_in_and_filters_by_group() {
    use futures::StreamExt;
    use std::time::Duration;

    let ctx = setup::setup_test_db().await;

    let group_a = fixtures::create_test_group(&ctx.pool, "Group A", "password")
        .await
        .expect("create group A");
    let group_b = fixtures::create_test_group(&ctx.pool, "Group B", "password")
        .await
        .expect("create group B");

    let alice = mario_kart_leaderboard_backend::models::Player::create(&ctx.pool, group_a.id, "Alice")
        .await
        .expect("create Alice");
    let outsider = mario_kart_leaderboard_backend::models::Player::create(&ctx.pool, group_b.id, "Zoe")
        .await
        .expect("create outsider");

    let notification_manager = NotificationManager::new();

    // Subscribe as group A.
    let request = Request::new(r#"subscription { lobbyUpdated { id name } }"#)
        .data(ctx.config.clone())
        .data(GraphQLContext::new(
            ctx.pool.clone(),
            Some(group_a.id),
            notification_manager.clone(),
        ));
    let mut stream = ctx.schema.execute_stream(request);

    // The stream's broadcast receiver is only registered when the stream
    // begins running. Spawn the notifications after a small delay so the
    // receiver is live by the time we send.
    let pool = ctx.pool.clone();
    let nm = notification_manager.clone();
    let group_a_id = group_a.id;
    let group_b_id = group_b.id;
    let alice_id = alice.id;
    let outsider_id = outsider.id;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Notification for group B must NOT reach the group A subscriber.
        mario_kart_leaderboard_backend::models::LobbyEntry::check_in(&pool, group_b_id, outsider_id)
            .await
            .expect("check in outsider");
        nm.notify_lobby(LobbyNotification { group_id: group_b_id });

        // Notification for group A must be delivered.
        mario_kart_leaderboard_backend::models::LobbyEntry::check_in(&pool, group_a_id, alice_id)
            .await
            .expect("check in alice");
        nm.notify_lobby(LobbyNotification { group_id: group_a_id });
    });

    let response = tokio::time::timeout(Duration::from_secs(3), stream.next())
        .await
        .expect("subscription timed out")
        .expect("subscription stream ended early");

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);

    let data = response.data.into_json().expect("parse response");
    let lobby = data.get("lobbyUpdated").expect("lobbyUpdated").as_array().expect("array");

    // The yielded lobby is group A's only, and contains only Alice — proving
    // the group B notification was filtered out (otherwise the first event
    // delivered would have been for group B).
    assert_eq!(lobby.len(), 1);
    assert_eq!(
        lobby[0].get("id").and_then(|v| v.as_str()),
        Some(alice.id.to_string().as_str())
    );
    assert_eq!(lobby[0].get("name").and_then(|v| v.as_str()), Some("Alice"));
}
