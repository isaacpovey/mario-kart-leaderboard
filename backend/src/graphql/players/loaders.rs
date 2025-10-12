use crate::models::Player;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct PlayerLoader {
    pool: PgPool,
}

impl PlayerLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for PlayerLoader {
    type Value = Player;
    type Error = std::sync::Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let players = Player::find_by_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        Ok(players
            .into_iter()
            .map(|player| (player.id, player))
            .collect())
    }
}

pub struct PlayersByGroupLoader {
    pool: PgPool,
}

impl PlayersByGroupLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for PlayersByGroupLoader {
    type Value = Vec<Player>;
    type Error = std::sync::Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let players = Player::find_by_group_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        // Group players by group_id using functional fold
        let grouped =
            players
                .into_iter()
                .fold(HashMap::<Uuid, Vec<Player>>::new(), |mut acc, player| {
                    acc.entry(player.group_id).or_default().push(player);
                    acc
                });

        Ok(grouped)
    }
}
