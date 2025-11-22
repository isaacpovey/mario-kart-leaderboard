
use crate::db::DbPool;
use crate::models::Player;
use async_graphql::dataloader::*;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct PlayersByTeamLoader {
    pool: DbPool,
}

impl PlayersByTeamLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for PlayersByTeamLoader {
    type Value = Vec<Player>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let player_teams = Player::find_by_team_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let grouped = player_teams.into_iter().fold(
            HashMap::<Uuid, Vec<Player>>::new(),
            |mut acc, (team_id, player)| {
                acc.entry(team_id).or_default().push(player);
                acc
            },
        );

        Ok(grouped)
    }
}
