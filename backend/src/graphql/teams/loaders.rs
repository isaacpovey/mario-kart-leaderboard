use crate::models::{Player, Team};
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct TeamsByMatchLoader {
    pool: PgPool,
}

impl TeamsByMatchLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for TeamsByMatchLoader {
    type Value = Vec<Team>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let teams = Team::find_by_match_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let grouped = teams
            .into_iter()
            .fold(HashMap::<Uuid, Vec<Team>>::new(), |mut acc, t| {
                acc.entry(t.match_id).or_default().push(t);
                acc
            });

        Ok(grouped)
    }
}

pub struct PlayersByTeamLoader {
    pool: PgPool,
}

impl PlayersByTeamLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for PlayersByTeamLoader {
    type Value = Vec<Player>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let player_teams = Player::find_by_team_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let grouped = player_teams.into_iter().fold(
            HashMap::<Uuid, Vec<Player>>::new(),
            |mut acc, (team_id, player)| {
                acc.entry(team_id).or_default().push(player);
                acc
            },
        );

        Ok(grouped)
    }
}
