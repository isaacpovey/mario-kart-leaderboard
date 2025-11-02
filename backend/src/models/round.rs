use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Round {
    pub match_id: Uuid,
    pub round_number: i32,
    pub track_id: Option<Uuid>,
    pub completed: bool,
}

impl Round {
    pub async fn find_one(
        pool: &sqlx::PgPool,
        match_id: Uuid,
        round_number: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT match_id, round_number, track_id, completed
             FROM rounds
             WHERE match_id = $1 AND round_number = $2",
        )
        .bind(match_id)
        .bind(round_number)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_match_id(
        pool: &sqlx::PgPool,
        match_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT match_id, round_number, track_id, completed
             FROM rounds
             WHERE match_id = $1
             ORDER BY round_number ASC",
        )
        .bind(match_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_match_ids(
        pool: &sqlx::PgPool,
        match_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT match_id, round_number, track_id, completed
             FROM rounds
             WHERE match_id = ANY($1)
             ORDER BY match_id, round_number ASC",
        )
        .bind(match_ids)
        .fetch_all(pool)
        .await
    }
}
