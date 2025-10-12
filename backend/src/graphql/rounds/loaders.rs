use crate::models::Round;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct RoundsByMatchLoader {
    pool: PgPool,
}

impl RoundsByMatchLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for RoundsByMatchLoader {
    type Value = Vec<Round>;
    type Error = std::sync::Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let rounds = Round::find_by_match_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let match_ids: HashSet<Uuid> = rounds.iter().map(|r| r.match_id).collect();
        let grouped: HashMap<Uuid, Vec<Round>> = match_ids
            .into_iter()
            .map(|match_id| {
                let match_rounds: Vec<Round> = rounds
                    .iter()
                    .filter(|r| r.match_id == match_id)
                    .cloned()
                    .collect();
                (match_id, match_rounds)
            })
            .collect();

        Ok(grouped)
    }
}
