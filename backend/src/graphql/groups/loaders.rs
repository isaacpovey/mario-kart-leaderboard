use crate::models::Group;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct GroupLoader {
    pool: PgPool,
}

impl GroupLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for GroupLoader {
    type Value = Group;
    type Error = std::sync::Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let groups = Group::find_by_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        Ok(groups.into_iter().map(|group| (group.id, group)).collect())
    }
}
