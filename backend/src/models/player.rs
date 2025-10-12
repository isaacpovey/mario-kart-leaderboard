use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Player {
    pub id: Uuid,
    pub group_id: Uuid,
    pub name: String,
    pub elo_rating: i32,
}

impl Player {
    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, name, elo_rating FROM players WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_ids(pool: &sqlx::PgPool, ids: &[Uuid]) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, name, elo_rating FROM players WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_group_id(
        pool: &sqlx::PgPool,
        group_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, name, elo_rating FROM players WHERE group_id = $1 ORDER BY elo_rating DESC"
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_group_ids(
        pool: &sqlx::PgPool,
        group_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, name, elo_rating FROM players WHERE group_id = ANY($1) ORDER BY elo_rating DESC"
        )
        .bind(group_ids)
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        group_id: Uuid,
        name: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO players (group_id, name) VALUES ($1, $2) RETURNING id, group_id, name, elo_rating"
        )
        .bind(group_id)
        .bind(name)
        .fetch_one(pool)
        .await
    }
}
