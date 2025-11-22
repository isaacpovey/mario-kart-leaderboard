use crate::db::DbPool;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PlayerMatchScore {
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub player_id: Uuid,
    pub position: i32,
    pub elo_change: i32,
    pub tournament_elo_change: i32,
    pub tournament_elo_from_races: i32,
    pub tournament_elo_from_contributions: i32,
    pub created_at: DateTime<Utc>,
}

impl PlayerMatchScore {
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_match_id(
        pool: &DbPool,
        match_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT group_id, match_id, player_id, position,
                    elo_change, tournament_elo_change,
                    tournament_elo_from_races, tournament_elo_from_contributions,
                    created_at
             FROM player_match_scores
             WHERE match_id = $1
             ORDER BY position ASC",
        )
        .bind(match_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = match_ids.len()))]
    pub async fn find_by_match_ids(
        pool: &DbPool,
        match_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT group_id, match_id, player_id, position,
                    elo_change, tournament_elo_change,
                    tournament_elo_from_races, tournament_elo_from_contributions,
                    created_at
             FROM player_match_scores
             WHERE match_id = ANY($1)
             ORDER BY match_id, position ASC",
        )
        .bind(match_ids)
        .fetch_all(pool)
        .await
    }
}
