use crate::models::Round;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
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

        let grouped = rounds
            .into_iter()
            .fold(HashMap::<Uuid, Vec<Round>>::new(), |mut acc, r| {
                acc.entry(r.match_id).or_default().push(r);
                acc
            });

        Ok(grouped)
    }
}
