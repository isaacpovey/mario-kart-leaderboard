use crate::models::Track;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct TrackLoader {
    pool: PgPool,
}

impl TrackLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for TrackLoader {
    type Value = Track;
    type Error = std::sync::Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let tracks = Track::find_by_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let mapped = tracks
            .into_iter()
            .map(|track| (track.id, track))
            .collect();

        Ok(mapped)
    }
}
