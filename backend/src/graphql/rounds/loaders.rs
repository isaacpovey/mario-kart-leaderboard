
use crate::db::DbPool;
use crate::models::{Player, Round};
use async_graphql::dataloader::*;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct RoundsByMatchLoader {
    pool: DbPool,
}

impl RoundsByMatchLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for RoundsByMatchLoader {
    type Value = Vec<Round>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let rounds = Round::find_by_match_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let grouped = rounds
            .into_iter()
            .fold(HashMap::<Uuid, Vec<Round>>::new(), |mut acc, r| {
                acc.entry(r.match_id).or_default().push(r);
                acc
            });

        Ok(grouped)
    }
}

pub struct PlayersByRoundLoader {
    pool: DbPool,
}

impl PlayersByRoundLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<(Uuid, i32)> for PlayersByRoundLoader {
    type Value = Vec<Player>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(
        &self,
        keys: &[(Uuid, i32)],
    ) -> Result<HashMap<(Uuid, i32), Self::Value>, Self::Error> {
        let player_rounds = Player::find_by_round_keys(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let grouped = player_rounds.into_iter().fold(
            HashMap::<(Uuid, i32), Vec<Player>>::new(),
            |mut acc, (round_key, player)| {
                acc.entry(round_key).or_default().push(player);
                acc
            },
        );

        Ok(grouped)
    }
}
