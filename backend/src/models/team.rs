use sqlx::FromRow;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Team {
    pub id: Uuid,
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub team_num: i32,
    pub score: Option<i32>,
}

impl Team {
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_match_id(
        pool: &sqlx::PgPool,
        match_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, match_id, team_num, score
             FROM teams
             WHERE match_id = $1
             ORDER BY team_num ASC",
        )
        .bind(match_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = match_ids.len()))]
    pub async fn find_by_match_ids(
        pool: &sqlx::PgPool,
        match_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, match_id, team_num, score
             FROM teams
             WHERE match_id = ANY($1)
             ORDER BY match_id, team_num ASC",
        )
        .bind(match_ids)
        .fetch_all(pool)
        .await
    }
}
