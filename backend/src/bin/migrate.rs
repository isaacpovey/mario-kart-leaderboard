use clap::{Parser, Subcommand};
use mario_kart_leaderboard_backend::error::Result;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[derive(Parser)]
#[command(name = "migrate")]
#[command(about = "Database migration tool for Mario Kart Leaderboard")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all pending migrations
    Up,
    /// Revert the last migration
    Down,
    /// Show migration status
    Status,
    /// Create a new migration file
    Add { name: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("DATABASE_URL not set, using default");
        "postgresql://postgres:password@localhost/mario_kart".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    match cli.command {
        Commands::Up => {
            println!("Running migrations...");
            sqlx::migrate!("./migrations").run(&pool).await?;
            println!("Migrations completed successfully!");
        }
        Commands::Down => {
            println!("Reverting last migration...");
            sqlx::migrate!("./migrations").undo(&pool, 1).await?;
            println!("Migration reverted successfully!");
        }
        Commands::Status => {
            println!("Checking migration status...");
            let migrator = sqlx::migrate!("./migrations");
            println!("Available migrations: {}", migrator.migrations.len());
        }
        Commands::Add { name } => {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let filename = format!("migrations/{}_{}.sql", timestamp, name);
            std::fs::write(&filename, "-- Add your SQL here\n")?;
            println!("Created migration file: {}", filename);
        }
    }

    Ok(())
}
