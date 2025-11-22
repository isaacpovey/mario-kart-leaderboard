use crate::db::DbPool;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub password: String, // Hashed password
}

impl Group {
    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT id, name, password FROM groups WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT id, name, password FROM groups WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(pool)
            .await
    }

    pub async fn find_by_name(
        pool: &DbPool,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT id, name, password FROM groups WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(
        pool: &DbPool,
        name: &str,
        password_hash: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO groups (name, password) VALUES ($1, $2) RETURNING id, name, password",
        )
        .bind(name)
        .bind(password_hash)
        .fetch_one(pool)
        .await
    }
}
