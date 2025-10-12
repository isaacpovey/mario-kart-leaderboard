use crate::models::Tournament;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
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
