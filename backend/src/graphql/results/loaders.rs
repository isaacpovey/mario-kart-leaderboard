
use crate::db::DbPool;
use crate::models::{PlayerMatchScore, PlayerRaceScore};
use async_graphql::dataloader::*;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct PlayerRaceScoresByRoundLoader {
    pool: DbPool,
}

impl PlayerRaceScoresByRoundLoader {
    pub fn new(pool: DbPool) -> Self {
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
    pool: DbPool,
}

impl PlayerMatchScoresByMatchLoader {
    pub fn new(pool: DbPool) -> Self {
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

pub struct PlayerTeammateContributionLoader {
    pool: DbPool,
}

impl PlayerTeammateContributionLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<(Uuid, Uuid)> for PlayerTeammateContributionLoader {
    type Value = i32;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(
        &self,
        keys: &[(Uuid, Uuid)],
    ) -> Result<HashMap<(Uuid, Uuid), Self::Value>, Self::Error> {
        let match_ids: Vec<Uuid> = keys.iter().map(|(match_id, _)| *match_id).collect();
        let player_ids: Vec<Uuid> = keys.iter().map(|(_, player_id)| *player_id).collect();

        let rows = sqlx::query_as::<_, (Uuid, Uuid, i64)>(
            "SELECT match_id, beneficiary_player_id, COALESCE(SUM(contribution_amount), 0)::bigint
             FROM player_teammate_elo_contributions
             WHERE match_id = ANY($1) AND beneficiary_player_id = ANY($2)
             GROUP BY match_id, beneficiary_player_id",
        )
        .bind(&match_ids)
        .bind(&player_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(std::sync::Arc::new)?;

        Ok(rows
            .into_iter()
            .map(|(match_id, player_id, contribution)| ((match_id, player_id), contribution as i32))
            .collect())
    }
}
