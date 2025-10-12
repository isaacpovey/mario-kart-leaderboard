use chrono::{NaiveDate, Utc};
use mario_kart_leaderboard_backend::{
    auth::hash_password,
    models::{Group, Match, Player, Round, Team, Tournament},
};
use sqlx::PgPool;
use uuid::Uuid;

/// Create a test group with hashed password
pub async fn create_test_group(
    pool: &PgPool,
    name: &str,
    password: &str,
) -> Result<Group, sqlx::Error> {
    let password_hash = hash_password(password).expect("Failed to hash password");
    Group::create(pool, name, &password_hash).await
}

/// Create multiple test groups
pub async fn create_test_groups(pool: &PgPool, count: usize) -> Result<Vec<Group>, sqlx::Error> {
    let params: Vec<_> = (0..count)
        .map(|i| (format!("Test Group {}", i + 1), format!("password{}", i + 1)))
        .collect();

    futures::future::try_join_all(
        params.iter().map(|(name, password)| create_test_group(pool, name, password))
    ).await
}

/// Create a test player
pub async fn create_test_player(
    pool: &PgPool,
    group_id: Uuid,
    name: &str,
) -> Result<Player, sqlx::Error> {
    Player::create(pool, group_id, name).await
}

/// Create multiple test players for a group
pub async fn create_test_players(
    pool: &PgPool,
    group_id: Uuid,
    count: usize,
) -> Result<Vec<Player>, sqlx::Error> {
    let names: Vec<_> = (0..count)
        .map(|i| format!("Player {}", i + 1))
        .collect();

    futures::future::try_join_all(
        names.iter().map(|name| create_test_player(pool, group_id, name))
    ).await
}

/// Create a test tournament
pub async fn create_test_tournament(
    pool: &PgPool,
    group_id: Uuid,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
) -> Result<Tournament, sqlx::Error> {
    Tournament::create(pool, group_id, start_date, end_date).await
}

/// Create multiple test tournaments for a group
pub async fn create_test_tournaments(
    pool: &PgPool,
    group_id: Uuid,
    count: usize,
) -> Result<Vec<Tournament>, sqlx::Error> {
    futures::future::try_join_all(
        (0..count).map(|i| {
            let start_date = NaiveDate::from_ymd_opt(2024, 1, i as u32 + 1);
            let end_date = NaiveDate::from_ymd_opt(2024, 1, i as u32 + 7);
            create_test_tournament(pool, group_id, start_date, end_date)
        })
    ).await
}

/// Create a test match via direct DB insert
pub async fn create_test_match(
    pool: &PgPool,
    group_id: Uuid,
    tournament_id: Uuid,
    num_of_rounds: i32,
) -> Result<Match, sqlx::Error> {
    sqlx::query_as::<_, Match>(
        "INSERT INTO matches (group_id, tournament_id, time, rounds, completed)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, group_id, tournament_id, time, rounds, completed",
    )
    .bind(group_id)
    .bind(tournament_id)
    .bind(Utc::now())
    .bind(num_of_rounds)
    .bind(false)
    .fetch_one(pool)
    .await
}

/// Create a test team for a match
pub async fn create_test_team(
    pool: &PgPool,
    group_id: Uuid,
    match_id: Uuid,
    team_num: i32,
) -> Result<Team, sqlx::Error> {
    sqlx::query_as::<_, Team>(
        "INSERT INTO teams (group_id, match_id, team_num)
         VALUES ($1, $2, $3)
         RETURNING id, group_id, match_id, team_num, score",
    )
    .bind(group_id)
    .bind(match_id)
    .bind(team_num)
    .fetch_one(pool)
    .await
}

/// Create multiple test teams for a match
pub async fn create_test_teams(
    pool: &PgPool,
    group_id: Uuid,
    match_id: Uuid,
    count: i32,
) -> Result<Vec<Team>, sqlx::Error> {
    futures::future::try_join_all(
        (0..count).map(|i| create_test_team(pool, group_id, match_id, i + 1))
    ).await
}

/// Create a test round for a match
pub async fn create_test_round(
    pool: &PgPool,
    match_id: Uuid,
    round_number: i32,
    track_id: Option<Uuid>,
) -> Result<Round, sqlx::Error> {
    sqlx::query_as::<_, Round>(
        "INSERT INTO rounds (match_id, round_number, track_id, completed)
         VALUES ($1, $2, $3, $4)
         RETURNING match_id, round_number, track_id, completed",
    )
    .bind(match_id)
    .bind(round_number)
    .bind(track_id)
    .bind(false)
    .fetch_one(pool)
    .await
}

/// Create multiple test rounds for a match
pub async fn create_test_rounds(
    pool: &PgPool,
    match_id: Uuid,
    count: i32,
) -> Result<Vec<Round>, sqlx::Error> {
    let track_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM tracks LIMIT 1")
        .fetch_optional(pool)
        .await?;

    futures::future::try_join_all(
        (0..count).map(|i| create_test_round(pool, match_id, i + 1, track_id))
    ).await
}

/// Add players to a round
pub async fn add_players_to_round(
    pool: &PgPool,
    group_id: Uuid,
    match_id: Uuid,
    round_number: i32,
    team_id: Uuid,
    player_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    futures::future::try_join_all(
        player_ids.iter().enumerate().map(|(position, player_id)| {
            sqlx::query(
                "INSERT INTO round_players (group_id, match_id, round_number, player_id, team_id, player_position)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(group_id)
            .bind(match_id)
            .bind(round_number)
            .bind(player_id)
            .bind(team_id)
            .bind((position + 1) as i32)
            .execute(pool)
        })
    ).await?;
    Ok(())
}
