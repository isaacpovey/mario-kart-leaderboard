
use crate::db::DbPool;
use crate::models::Track;
use async_graphql::dataloader::*;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct TrackLoader {
    pool: DbPool,
}

impl TrackLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for TrackLoader {
    type Value = Track;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let tracks = Track::find_by_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let mapped = tracks.into_iter().map(|track| (track.id, track)).collect();

        Ok(mapped)
    }
}
