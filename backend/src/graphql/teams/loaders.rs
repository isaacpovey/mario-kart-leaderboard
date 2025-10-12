use crate::models::Team;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
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

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let teams = Team::find_by_match_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let match_ids: HashSet<Uuid> = teams.iter().map(|t| t.match_id).collect();
        let grouped: HashMap<Uuid, Vec<Team>> = match_ids
            .into_iter()
            .map(|match_id| {
                let match_teams: Vec<Team> = teams
                    .iter()
                    .filter(|t| t.match_id == match_id)
                    .cloned()
                    .collect();
                (match_id, match_teams)
            })
            .collect();

        Ok(grouped)
    }
}
