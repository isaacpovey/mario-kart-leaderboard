use mario_kart_leaderboard_backend::{
    auth::{create_jwt, hash_password},
    error::{AppError, Result},
    models::{Group, Player},
    services::validation::{validate_name, validate_password},
};
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashSet, env};

const DEFAULT_GROUP_NAME: &str = "Dev Group";
const DEFAULT_GROUP_PASSWORD: &str = "devpassword";
const PLAYER_NAMES: &[&str] = &["Mario", "Luigi", "Peach", "Bowser"];

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").map_err(|_| {
        AppError::InvalidInput("DATABASE_URL not set — configure backend/.env".to_string())
    })?;
    let jwt_secret = env::var("JWT_SECRET").map_err(|_| {
        AppError::InvalidInput("JWT_SECRET not set — configure backend/.env".to_string())
    })?;

    let group_name =
        env::var("SEED_GROUP_NAME").unwrap_or_else(|_| DEFAULT_GROUP_NAME.to_string());
    let group_password =
        env::var("SEED_GROUP_PASSWORD").unwrap_or_else(|_| DEFAULT_GROUP_PASSWORD.to_string());

    validate_name(&group_name, "Group name")?;
    validate_password(&group_password)?;

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    println!("seed: connecting to database … ok");
    println!("seed: ⚠️  dev data only — do NOT run against production");

    let group = match Group::find_by_name(&pool, &group_name).await? {
        Some(existing) => {
            println!(
                "seed: group '{}' already exists — reusing id {}",
                group_name, existing.id
            );
            existing
        }
        None => {
            let hash = hash_password(&group_password)?;
            let created = Group::create(&pool, &group_name, &hash).await?;
            println!(
                "seed: created group '{}' with id {}",
                group_name, created.id
            );
            created
        }
    };

    let existing_players = Player::find_by_group_id(&pool, group.id).await?;
    let existing_names: HashSet<&str> =
        existing_players.iter().map(|p| p.name.as_str()).collect();

    for name in PLAYER_NAMES {
        if existing_names.contains(*name) {
            println!("seed: player '{}' already exists — skipping", name);
            continue;
        }
        Player::create(&pool, group.id, name).await?;
        println!("seed: created player '{}'", name);
    }

    let token = create_jwt(group.id, &jwt_secret)?;
    println!("\nGROUP_ID: {}", group.id);
    println!("JWT: {}", token);

    Ok(())
}
