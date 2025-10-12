use chrono::NaiveDate;
use mario_kart_leaderboard_backend::{
    auth::hash_password,
    models::{Group, Player, Tournament},
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
    let mut groups = Vec::new();
    for i in 0..count {
        let group = create_test_group(
            pool,
            &format!("Test Group {}", i + 1),
            &format!("password{}", i + 1),
        )
        .await?;
        groups.push(group);
    }
    Ok(groups)
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
    let mut players = Vec::new();
    for i in 0..count {
        let player = create_test_player(pool, group_id, &format!("Player {}", i + 1)).await?;
        players.push(player);
    }
    Ok(players)
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
    let mut tournaments = Vec::new();
    for i in 0..count {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, i as u32 + 1);
        let end_date = NaiveDate::from_ymd_opt(2024, 1, i as u32 + 7);
        let tournament = create_test_tournament(pool, group_id, start_date, end_date).await?;
        tournaments.push(tournament);
    }
    Ok(tournaments)
}
