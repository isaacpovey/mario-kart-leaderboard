use crate::models::Tournament;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct TournamentLoader {
    pool: PgPool,
}

impl TournamentLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for TournamentLoader {
    type Value = Tournament;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let tournaments = Tournament::find_by_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        Ok(tournaments
            .into_iter()
            .map(|tournament| (tournament.id, tournament))
            .collect())
    }
}

pub struct ActiveTournamentLoader {
    pool: PgPool,
}

impl ActiveTournamentLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for ActiveTournamentLoader {
    type Value = Uuid;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let mut result = HashMap::new();

        for group_id in keys {
            if let Some(tournament_id) = Tournament::get_active_tournament(&self.pool, *group_id)
                .await
                .map_err(std::sync::Arc::new)?
            {
                result.insert(*group_id, tournament_id);
            }
        }

        Ok(result)
    }
}

pub struct PlayerTournamentEloLoader {
    pool: PgPool,
}

impl PlayerTournamentEloLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<(Uuid, Uuid)> for PlayerTournamentEloLoader {
    type Value = i32;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[(Uuid, Uuid)]) -> Result<HashMap<(Uuid, Uuid), Self::Value>, Self::Error> {
        let player_ids: Vec<Uuid> = keys.iter().map(|(player_id, _)| *player_id).collect();
        let tournament_ids: Vec<Uuid> = keys.iter().map(|(_, tournament_id)| *tournament_id).collect();

        let rows = sqlx::query_as::<_, (Uuid, Uuid, i32)>(
            "SELECT player_id, tournament_id, elo_rating
             FROM player_tournament_scores
             WHERE player_id = ANY($1) AND tournament_id = ANY($2)",
        )
        .bind(&player_ids)
        .bind(&tournament_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(std::sync::Arc::new)?;

        Ok(rows
            .into_iter()
            .map(|(player_id, tournament_id, elo_rating)| ((player_id, tournament_id), elo_rating))
            .collect())
    }
}
