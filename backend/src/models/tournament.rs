use chrono::NaiveDate;
use sqlx::FromRow;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Tournament {
    pub id: Uuid,
    pub group_id: Uuid,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub winner: Option<Uuid>,
}

impl Tournament {
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, start_date, end_date, winner FROM tournaments WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = ids.len()))]
    pub async fn find_by_ids(pool: &sqlx::PgPool, ids: &[Uuid]) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, start_date, end_date, winner FROM tournaments WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_group_id(
        pool: &sqlx::PgPool,
        group_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, start_date, end_date, winner
             FROM tournaments
             WHERE group_id = $1
             ORDER BY start_date DESC NULLS LAST",
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn create(
        pool: &sqlx::PgPool,
        group_id: Uuid,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO tournaments (group_id, start_date, end_date)
             VALUES ($1, $2, $3)
             RETURNING id, group_id, start_date, end_date, winner",
        )
        .bind(group_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await
    }
}
