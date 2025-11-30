mod common;

use async_graphql::{Request, Variables, value};
use common::{fixtures, setup};
use mario_kart_leaderboard_backend::{
    graphql::context::GraphQLContext,
    services::notification_manager::NotificationManager,
};

// ============================================================================
// Tests for `match_by_id` query
// ============================================================================

#[tokio::test]
async fn test_match_by_id_happy_path() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let query = r#"
        query MatchById($matchId: ID!) {
            matchById(matchId: $matchId) {
                id
                tournamentId
                numOfRounds
                completed
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string()
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
    let match_data = data.get("matchById").expect("matchById field not found");

    assert_eq!(
        match_data.get("id").and_then(|v| v.as_str()),
        Some(match_record.id.to_string().as_str())
    );
    assert_eq!(
        match_data.get("numOfRounds").and_then(|v| v.as_i64()),
        Some(2)
    );
    assert_eq!(
        match_data.get("completed").and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[tokio::test]
async fn test_match_by_id_with_resolved_fields() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 3)
        .await
        .expect("Failed to create test match");

    let teams = fixtures::create_test_teams(&ctx.pool, group.id, match_record.id, 4)
        .await
        .expect("Failed to create test teams");

    let players = fixtures::create_test_players(&ctx.pool, group.id, 4)
        .await
        .expect("Failed to create test players");

    // Add players to teams (one player per team)
    for (i, (team, player)) in teams.iter().zip(players.iter()).enumerate() {
        sqlx::query("INSERT INTO team_players (group_id, team_id, player_id, rank) VALUES ($1, $2, $3, $4)")
            .bind(group.id)
            .bind(team.id)
            .bind(player.id)
            .bind(i as i32 + 1)
            .execute(&ctx.pool)
            .await
            .expect("Failed to add player to team");
    }

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 3)
        .await
        .expect("Failed to create test rounds");

    let query = r#"
        query MatchById($matchId: ID!) {
            matchById(matchId: $matchId) {
                id
                numOfRounds
                rounds {
                    roundNumber
                    track {
                        id
                    }
                }
                teams {
                    id
                    teamNum
                    score
                }
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string()
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
    let match_data = data.get("matchById").expect("matchById field not found");

    assert_eq!(
        match_data.get("numOfRounds").and_then(|v| v.as_i64()),
        Some(3)
    );

    let rounds = match_data
        .get("rounds")
        .expect("rounds field not found")
        .as_array()
        .expect("rounds should be an array");
    assert_eq!(rounds.len(), 3);

    let teams = match_data
        .get("teams")
        .expect("teams field not found")
        .as_array()
        .expect("teams should be an array");
    assert_eq!(teams.len(), 4);
}

#[tokio::test]
async fn test_match_by_id_invalid_id() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let query = r#"
        query MatchById($matchId: ID!) {
            matchById(matchId: $matchId) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": "not-a-valid-uuid"
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected validation error");
    assert!(
        response.errors[0].message.contains("Invalid match ID"),
        "Expected Invalid match ID error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_match_by_id_not_found() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let query = r#"
        query MatchById($matchId: ID!) {
            matchById(matchId: $matchId) {
                id
            }
        }
    "#;

    // Use a valid UUID that doesn't exist
    let fake_uuid = "00000000-0000-0000-0000-000000000000";

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": fake_uuid
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected not found error");
    assert!(
        response.errors[0].message.contains("Match not found"),
        "Expected Match not found error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_match_by_id_unauthorized() {
    let ctx = setup::setup_test_db().await;

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

    let match_record = fixtures::create_test_match(&ctx.pool, group1.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let query = r#"
        query MatchById($matchId: ID!) {
            matchById(matchId: $matchId) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string()
        })))
        .data(ctx.config.clone());

    // Authenticate as group2, trying to access group1's match
    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group2.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected unauthorized error");
    assert!(
        response.errors[0].message.contains("Unauthorized"),
        "Expected Unauthorized error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_match_by_id_no_auth() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let query = r#"
        query MatchById($matchId: ID!) {
            matchById(matchId: $matchId) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string()
        })))
        .data(ctx.config.clone());

    // No authentication
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

// ============================================================================
// Tests for `create_match_with_rounds` mutation - Happy Paths
// ============================================================================

#[tokio::test]
async fn test_create_match_with_rounds_basic() {
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

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!, $playersPerRace: Int) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
                playersPerRace: $playersPerRace
            ) {
                id
                tournamentId
                numOfRounds
                completed
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string(),
            "playerIds": player_ids,
            "numRaces": 2,
            "playersPerRace": 4
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
    let match_data = data
        .get("createMatchWithRounds")
        .expect("createMatchWithRounds field not found");

    assert_eq!(
        match_data.get("numOfRounds").and_then(|v| v.as_i64()),
        Some(2)
    );
    assert_eq!(
        match_data.get("completed").and_then(|v| v.as_bool()),
        Some(false)
    );

    // Verify teams were created
    let match_id_str = match_data
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id not found");
    let match_id = uuid::Uuid::parse_str(match_id_str).expect("invalid uuid");

    let teams: Vec<(i32,)> =
        sqlx::query_as("SELECT team_num FROM teams WHERE match_id = $1 ORDER BY team_num")
            .bind(match_id)
            .fetch_all(&ctx.pool)
            .await
            .expect("Failed to fetch teams");

    assert_eq!(teams.len(), 4, "Should have 4 teams");

    // Verify rounds were created
    let rounds: Vec<(i32,)> =
        sqlx::query_as("SELECT round_number FROM rounds WHERE match_id = $1 ORDER BY round_number")
            .bind(match_id)
            .fetch_all(&ctx.pool)
            .await
            .expect("Failed to fetch rounds");

    assert_eq!(rounds.len(), 2, "Should have 2 rounds");
}

#[tokio::test]
async fn test_create_match_with_rounds_verifies_snake_draft() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    // Create 8 players with different ELO ratings
    let mut players = Vec::new();
    for i in 0..8 {
        let player =
            fixtures::create_test_player(&ctx.pool, group.id, &format!("Player {}", i + 1))
                .await
                .expect("Failed to create player");

        // Update ELO to create variance (highest to lowest)
        sqlx::query("UPDATE players SET elo_rating = $1 WHERE id = $2")
            .bind(1800 - (i * 100))
            .bind(player.id)
            .execute(&ctx.pool)
            .await
            .expect("Failed to update ELO");

        players.push(player);
    }

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!, $playersPerRace: Int) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
                playersPerRace: $playersPerRace
            ) {
                id
                numOfRounds
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string(),
            "playerIds": player_ids,
            "numRaces": 4,
            "playersPerRace": 4
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
    let match_data = data
        .get("createMatchWithRounds")
        .expect("createMatchWithRounds field not found");

    let match_id_str = match_data
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id not found");
    let match_id = uuid::Uuid::parse_str(match_id_str).expect("invalid uuid");

    // Verify team players were assigned
    let team_player_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM team_players tp
         JOIN teams t ON t.id = tp.team_id
         WHERE t.match_id = $1",
    )
    .bind(match_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("Failed to count team players");

    assert_eq!(
        team_player_count.0, 8,
        "All 8 players should be assigned to teams"
    );

    // Verify round_players were assigned correctly
    let round_player_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM round_players WHERE match_id = $1")
            .bind(match_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("Failed to count round players");

    // 4 rounds * 4 players per round = 16 entries
    assert_eq!(
        round_player_count.0, 16,
        "Should have 16 round player entries"
    );
}

#[tokio::test]
async fn test_create_match_with_rounds_verifies_tracks_assigned() {
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

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
            ) {
                id
                rounds {
                    roundNumber
                    track {
                        id
                    }
                }
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string(),
            "playerIds": player_ids,
            "numRaces": 3
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
    let match_data = data
        .get("createMatchWithRounds")
        .expect("createMatchWithRounds field not found");

    let rounds = match_data
        .get("rounds")
        .expect("rounds field not found")
        .as_array()
        .expect("rounds should be an array");

    assert_eq!(rounds.len(), 3, "Should have 3 rounds");

    // Verify all rounds have tracks
    for round in rounds {
        let track = round.get("track");
        assert!(
            track.is_some() && !track.unwrap().is_null(),
            "Each round should have a track"
        );
        let track_id = track.unwrap().get("id");
        assert!(
            track_id.is_some() && !track_id.unwrap().is_null(),
            "Each track should have an id"
        );
    }
}

// ============================================================================
// Tests for `create_match_with_rounds` mutation - Validation Errors
// ============================================================================

#[tokio::test]
async fn test_create_match_with_rounds_invalid_tournament_id() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let players = fixtures::create_test_players(&ctx.pool, group.id, 4)
        .await
        .expect("Failed to create test players");

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
            ) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": "not-a-valid-uuid",
            "playerIds": player_ids,
            "numRaces": 2
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected validation error");
    assert!(
        response.errors[0].message.contains("Invalid tournament ID"),
        "Expected Invalid tournament ID error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_create_match_with_rounds_tournament_not_found() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let players = fixtures::create_test_players(&ctx.pool, group.id, 4)
        .await
        .expect("Failed to create test players");

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
            ) {
                id
            }
        }
    "#;

    let fake_uuid = "00000000-0000-0000-0000-000000000000";

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": fake_uuid,
            "playerIds": player_ids,
            "numRaces": 2
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected not found error");
    assert!(
        response.errors[0].message.contains("Tournament not found"),
        "Expected Tournament not found error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_create_match_with_rounds_uneven_slot_distribution() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    let players = fixtures::create_test_players(&ctx.pool, group.id, 5)
        .await
        .expect("Failed to create test players");

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!, $playersPerRace: Int) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
                playersPerRace: $playersPerRace
            ) {
                id
            }
        }
    "#;

    // 2 races * 4 players per race = 8 slots for 5 players
    // The allocation algorithm handles uneven distributions correctly
    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string(),
            "playerIds": player_ids,
            "numRaces": 2,
            "playersPerRace": 4
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(
        response.errors.is_empty(),
        "Should create match successfully with uneven slot distribution: {:?}",
        response.errors
    );

    let data = response.data.into_json().expect("Failed to parse response");
    let match_data = data
        .get("createMatchWithRounds")
        .expect("Should have createMatchWithRounds in response");
    let match_id = match_data
        .get("id")
        .and_then(|id| id.as_str())
        .expect("Should return match ID");

    assert!(!match_id.is_empty(), "Match ID should not be empty");
}

#[tokio::test]
async fn test_create_match_with_rounds_players_per_race_exceeds_total() {
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

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!, $playersPerRace: Int) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
                playersPerRace: $playersPerRace
            ) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string(),
            "playerIds": player_ids,
            "numRaces": 1,
            "playersPerRace": 4  // More than the 2 players we have
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected validation error");
    assert!(
        response.errors[0]
            .message
            .contains("cannot exceed total number of players"),
        "Expected 'cannot exceed total number of players' error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_create_match_with_rounds_zero_races() {
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

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
            ) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string(),
            "playerIds": player_ids,
            "numRaces": 0
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected validation error");
    assert!(
        response.errors[0].message.contains("must be positive"),
        "Expected 'must be positive' error, got: {}",
        response.errors[0].message
    );
}

// ============================================================================
// Tests for `create_match_with_rounds` mutation - Auth Errors
// ============================================================================

#[tokio::test]
async fn test_create_match_with_rounds_unauthorized() {
    let ctx = setup::setup_test_db().await;

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

    let players = fixtures::create_test_players(&ctx.pool, group1.id, 4)
        .await
        .expect("Failed to create test players");

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
            ) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string(),
            "playerIds": player_ids,
            "numRaces": 2
        })))
        .data(ctx.config.clone());

    // Authenticate as group2, trying to create match in group1's tournament
    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group2.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected unauthorized error");
    assert!(
        response.errors[0].message.contains("Tournament not found"),
        "Expected 'does not belong to your group' error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_create_match_with_rounds_no_auth() {
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

    let player_ids: Vec<String> = players.iter().map(|p| p.id.to_string()).collect();

    let query = r#"
        mutation CreateMatch($tournamentId: ID!, $playerIds: [ID!]!, $numRaces: Int!) {
            createMatchWithRounds(
                tournamentId: $tournamentId
                playerIds: $playerIds
                numRaces: $numRaces
            ) {
                id
            }
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "tournamentId": tournament.id.to_string(),
            "playerIds": player_ids,
            "numRaces": 2
        })))
        .data(ctx.config.clone());

    // No authentication
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

// ============================================================================
// Tests for `cancel_match` mutation
// ============================================================================

#[tokio::test]
async fn test_cancel_match_success() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let query = r#"
        mutation CancelMatch($matchId: ID!) {
            cancelMatch(matchId: $matchId)
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string()
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
    let result = data
        .get("cancelMatch")
        .expect("cancelMatch field not found")
        .as_bool()
        .expect("cancelMatch should be a boolean");

    assert!(result, "cancelMatch should return true");

    let match_exists: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM matches WHERE id = $1")
            .bind(match_record.id)
            .fetch_optional(&ctx.pool)
            .await
            .expect("Failed to check match existence");

    assert!(match_exists.is_none(), "Match should be deleted");
}

#[tokio::test]
async fn test_cancel_match_with_results_fails() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let _rounds = fixtures::create_test_rounds(&ctx.pool, match_record.id, 2)
        .await
        .expect("Failed to create test rounds");

    let players = fixtures::create_test_players(&ctx.pool, group.id, 2)
        .await
        .expect("Failed to create test players");

    sqlx::query(
        "INSERT INTO player_race_scores
         (group_id, match_id, round_number, player_id, position, all_time_elo_change, all_time_elo_after, tournament_elo_change, tournament_elo_after, created_at)
         VALUES ($1, $2, 1, $3, 1, 10, 1210, 10, 1210, NOW())",
    )
    .bind(group.id)
    .bind(match_record.id)
    .bind(players[0].id)
    .execute(&ctx.pool)
    .await
    .expect("Failed to insert race score");

    let query = r#"
        mutation CancelMatch($matchId: ID!) {
            cancelMatch(matchId: $matchId)
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string()
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected error for match with results");
    assert!(
        response.errors[0]
            .message
            .contains("race results have been recorded"),
        "Expected 'race results have been recorded' error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_cancel_match_unauthorized() {
    let ctx = setup::setup_test_db().await;

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

    let match_record = fixtures::create_test_match(&ctx.pool, group1.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let query = r#"
        mutation CancelMatch($matchId: ID!) {
            cancelMatch(matchId: $matchId)
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string()
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group2.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected unauthorized error");
    assert!(
        response.errors[0].message.contains("Match not found"),
        "Expected 'Match not found' error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_cancel_match_not_found() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let query = r#"
        mutation CancelMatch($matchId: ID!) {
            cancelMatch(matchId: $matchId)
        }
    "#;

    let fake_uuid = "00000000-0000-0000-0000-000000000000";

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": fake_uuid
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), Some(group.id), NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected not found error");
    assert!(
        response.errors[0].message.contains("Match not found"),
        "Expected 'Match not found' error, got: {}",
        response.errors[0].message
    );
}

#[tokio::test]
async fn test_cancel_match_no_auth() {
    let ctx = setup::setup_test_db().await;

    let group = fixtures::create_test_group(&ctx.pool, "Test Group", "password")
        .await
        .expect("Failed to create test group");

    let tournaments = fixtures::create_test_tournaments(&ctx.pool, group.id, 1)
        .await
        .expect("Failed to create test tournaments");

    let tournament = &tournaments[0];

    let match_record = fixtures::create_test_match(&ctx.pool, group.id, tournament.id, 2)
        .await
        .expect("Failed to create test match");

    let query = r#"
        mutation CancelMatch($matchId: ID!) {
            cancelMatch(matchId: $matchId)
        }
    "#;

    let request = Request::new(query)
        .variables(Variables::from_value(value!({
            "matchId": match_record.id.to_string()
        })))
        .data(ctx.config.clone());

    let gql_ctx = GraphQLContext::new(ctx.pool.clone(), None, NotificationManager::new());
    let response = ctx.schema.execute(request.data(gql_ctx)).await;

    assert!(!response.errors.is_empty(), "Expected authentication error");
    assert!(
        response.errors[0]
            .message
            .contains("Authentication required"),
        "Expected 'Authentication required' error, got: {}",
        response.errors[0].message
    );
}
