use crate::db::DbPool;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Match {
    pub id: Uuid,
    pub group_id: Uuid,
    pub tournament_id: Uuid,
    pub time: DateTime<Utc>,
    #[sqlx(rename = "rounds")]
    pub num_of_rounds: i32,
    pub completed: bool,
}

impl Match {
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, tournament_id, time, rounds, completed
             FROM matches
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = ids.len()))]
    pub async fn find_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, tournament_id, time, rounds, completed
             FROM matches
             WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_tournament_id(
        pool: &DbPool,
        tournament_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, tournament_id, time, rounds, completed
             FROM matches
             WHERE tournament_id = $1
             ORDER BY time DESC",
        )
        .bind(tournament_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = tournament_ids.len()))]
    pub async fn find_by_tournament_ids(
        pool: &DbPool,
        tournament_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, tournament_id, time, rounds, completed
             FROM matches
             WHERE tournament_id = ANY($1)
             ORDER BY time DESC",
        )
        .bind(tournament_ids)
        .fetch_all(pool)
        .await
    }
}
