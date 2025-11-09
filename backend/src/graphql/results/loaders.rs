use crate::models::{PlayerMatchScore, PlayerRaceScore};
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct PlayerRaceScoresByRoundLoader {
    pool: PgPool,
}

impl PlayerRaceScoresByRoundLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<(Uuid, i32)> for PlayerRaceScoresByRoundLoader {
    type Value = Vec<PlayerRaceScore>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(
        &self,
        keys: &[(Uuid, i32)],
    ) -> Result<HashMap<(Uuid, i32), Self::Value>, Self::Error> {
        let scores = PlayerRaceScore::find_by_rounds(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let grouped = scores.into_iter().fold(
            HashMap::<(Uuid, i32), Vec<PlayerRaceScore>>::new(),
            |mut acc, score| {
                acc.entry((score.match_id, score.round_number))
                    .or_default()
                    .push(score);
                acc
            },
        );

        Ok(grouped)
    }
}

pub struct PlayerMatchScoresByMatchLoader {
    pool: PgPool,
}

impl PlayerMatchScoresByMatchLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for PlayerMatchScoresByMatchLoader {
    type Value = Vec<PlayerMatchScore>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let scores = PlayerMatchScore::find_by_match_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let grouped = scores
            .into_iter()
            .fold(HashMap::<Uuid, Vec<PlayerMatchScore>>::new(), |mut acc, score| {
                acc.entry(score.match_id).or_default().push(score);
                acc
            });

        Ok(grouped)
    }
}
