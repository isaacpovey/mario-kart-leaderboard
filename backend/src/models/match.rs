use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Type, PartialEq, Eq)]
#[sqlx(type_name = "team_creation_mode", rename_all = "lowercase")]
pub enum TeamCreationMode {
    Balanced,
    Full,
}

#[derive(Debug, Clone, FromRow)]
pub struct Match {
    pub id: Uuid,
    pub group_id: Uuid,
    pub tournament_id: Uuid,
    pub time: DateTime<Utc>,
    pub rounds: i32,
    pub team_mode: TeamCreationMode,
    pub completed: bool,
}

impl Match {
    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, tournament_id, time, rounds, team_mode, completed
             FROM matches
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_ids(pool: &sqlx::PgPool, ids: &[Uuid]) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, tournament_id, time, rounds, team_mode, completed
             FROM matches
             WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_tournament_id(
        pool: &sqlx::PgPool,
        tournament_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, tournament_id, time, rounds, team_mode, completed
             FROM matches
             WHERE tournament_id = $1
             ORDER BY time DESC",
        )
        .bind(tournament_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_tournament_ids(
        pool: &sqlx::PgPool,
        tournament_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, tournament_id, time, rounds, team_mode, completed
             FROM matches
             WHERE tournament_id = ANY($1)
             ORDER BY time DESC",
        )
        .bind(tournament_ids)
        .fetch_all(pool)
        .await
    }
}
