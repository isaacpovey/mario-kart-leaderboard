use crate::db::DbPool;
use crate::models::{LobbyEntry, Player};
use async_graphql::dataloader::*;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct LobbyByGroupLoader {
    pool: DbPool,
}

impl LobbyByGroupLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for LobbyByGroupLoader {
    type Value = Vec<Player>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let entries = LobbyEntry::find_by_group_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        if entries.is_empty() {
            return Ok(HashMap::new());
        }

        let player_ids: Vec<Uuid> = entries.iter().map(|e| e.player_id).collect();
        let players = Player::find_by_ids(&self.pool, &player_ids)
            .await
            .map_err(std::sync::Arc::new)?;

        let player_by_id: HashMap<Uuid, Player> =
            players.into_iter().map(|p| (p.id, p)).collect();

        // Group entries by group_id, preserving order (entries are already ORDER BY p.name ASC)
        let grouped = entries.into_iter().fold(
            HashMap::<Uuid, Vec<Player>>::new(),
            |mut acc, entry| {
                if let Some(player) = player_by_id.get(&entry.player_id).cloned() {
                    if !player.disabled {
                        acc.entry(entry.group_id).or_default().push(player);
                    }
                }
                acc
            },
        );

        Ok(grouped)
    }
}
