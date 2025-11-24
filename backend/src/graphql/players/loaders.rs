
use crate::db::DbPool;
use crate::models::Player;
use async_graphql::dataloader::*;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct PlayerLoader {
    pool: DbPool,
}

impl PlayerLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for PlayerLoader {
    type Value = Player;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
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
    pool: DbPool,
}

impl PlayersByGroupLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for PlayersByGroupLoader {
    type Value = Vec<Player>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
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

pub struct PlayerActiveTournamentEloLoader {
    pool: DbPool,
}

impl PlayerActiveTournamentEloLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<(Uuid, Uuid)> for PlayerActiveTournamentEloLoader {
    type Value = i32;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(
        &self,
        keys: &[(Uuid, Uuid)],
    ) -> Result<HashMap<(Uuid, Uuid), Self::Value>, Self::Error> {
        let player_ids: Vec<Uuid> = keys.iter().map(|(player_id, _)| *player_id).collect();
        let group_ids: Vec<Uuid> = keys.iter().map(|(_, group_id)| *group_id).collect();

        let rows = sqlx::query_as::<_, (Uuid, Uuid, i32)>(
            "SELECT pts.player_id, t.group_id, pts.elo_rating
             FROM player_tournament_scores pts
             JOIN tournaments t ON t.id = pts.tournament_id
             WHERE pts.player_id = ANY($1)
               AND t.group_id = ANY($2)
               AND t.winner IS NULL",
        )
        .bind(&player_ids)
        .bind(&group_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(std::sync::Arc::new)?;

        Ok(rows
            .into_iter()
            .map(|(player_id, group_id, elo_rating)| ((player_id, group_id), elo_rating))
            .collect())
    }
}
