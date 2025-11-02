use crate::models::Team;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
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

        let grouped = teams
            .into_iter()
            .fold(HashMap::<Uuid, Vec<Team>>::new(), |mut acc, t| {
                acc.entry(t.match_id).or_default().push(t);
                acc
            });

        Ok(grouped)
    }
}
