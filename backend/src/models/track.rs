use crate::db::DbPool;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Track {
    pub id: Uuid,
    pub name: String,
}

impl Track {
    pub async fn find_all(pool: &DbPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT id, name FROM tracks ORDER BY name")
            .fetch_all(pool)
            .await
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT id, name FROM tracks WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT id, name FROM tracks WHERE id = ANY($1) ORDER BY name")
            .bind(ids)
            .fetch_all(pool)
            .await
    }
}
