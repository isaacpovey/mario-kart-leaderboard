use crate::db::DbPool;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct LobbyEntry {
    pub group_id: Uuid,
    pub player_id: Uuid,
    pub checked_in_at: DateTime<Utc>,
}

impl LobbyEntry {
    /// Insert a player into the lobby. Idempotent: calling twice is a no-op.
    #[instrument(level = "debug", skip(pool))]
    pub async fn check_in(
        pool: &DbPool,
        group_id: Uuid,
        player_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO lobby_entries (group_id, player_id)
             VALUES ($1, $2)
             ON CONFLICT (group_id, player_id) DO NOTHING",
        )
        .bind(group_id)
        .bind(player_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Remove a player from the lobby. Idempotent: missing row is a no-op.
    #[instrument(level = "debug", skip(pool))]
    pub async fn check_out(
        pool: &DbPool,
        group_id: Uuid,
        player_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM lobby_entries WHERE group_id = $1 AND player_id = $2")
            .bind(group_id)
            .bind(player_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// List lobby entries for a group, oldest check-in first.
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_group_id(
        pool: &DbPool,
        group_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT group_id, player_id, checked_in_at FROM lobby_entries
             WHERE group_id = $1
             ORDER BY checked_in_at ASC",
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
    }

    /// Batched lookup for DataLoader.
    #[instrument(level = "debug", skip(pool), fields(batch_size = group_ids.len()))]
    pub async fn find_by_group_ids(
        pool: &DbPool,
        group_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT group_id, player_id, checked_in_at FROM lobby_entries
             WHERE group_id = ANY($1)
             ORDER BY checked_in_at ASC",
        )
        .bind(group_ids)
        .fetch_all(pool)
        .await
    }
}
