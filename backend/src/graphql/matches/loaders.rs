
use crate::db::DbPool;
use crate::models::Match;
use async_graphql::dataloader::*;
use std::collections::{HashMap, HashSet};
use tracing::instrument;
use uuid::Uuid;

pub struct MatchLoader {
    pool: DbPool,
}

impl MatchLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for MatchLoader {
    type Value = Match;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let matches = Match::find_by_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        Ok(matches.into_iter().map(|m| (m.id, m)).collect())
    }
}

pub struct MatchesByTournamentLoader {
    pool: DbPool,
}

impl MatchesByTournamentLoader {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for MatchesByTournamentLoader {
    type Value = Vec<Match>;
    type Error = std::sync::Arc<sqlx::Error>;

    #[instrument(level = "debug", skip(self), fields(batch_size = keys.len()))]
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let matches = Match::find_by_tournament_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        let tournament_ids: HashSet<Uuid> = matches.iter().map(|m| m.tournament_id).collect();
        let grouped: HashMap<Uuid, Vec<Match>> = tournament_ids
            .into_iter()
            .map(|tournament_id| {
                let tournament_matches: Vec<Match> = matches
                    .iter()
                    .filter(|m| m.tournament_id == tournament_id)
                    .cloned()
                    .collect();
                (tournament_id, tournament_matches)
            })
            .collect();

        Ok(grouped)
    }
}
