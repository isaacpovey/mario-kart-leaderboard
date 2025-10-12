use crate::models::Match;
use async_graphql::dataloader::*;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct MatchLoader {
    pool: PgPool,
}

impl MatchLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for MatchLoader {
    type Value = Match;
    type Error = std::sync::Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let matches = Match::find_by_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        Ok(matches.into_iter().map(|m| (m.id, m)).collect())
    }
}

pub struct MatchesByTournamentLoader {
    pool: PgPool,
}

impl MatchesByTournamentLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<Uuid> for MatchesByTournamentLoader {
    type Value = Vec<Match>;
    type Error = std::sync::Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let matches = Match::find_by_tournament_ids(&self.pool, keys)
            .await
            .map_err(std::sync::Arc::new)?;

        // Group matches by tournament_id using functional fold
        let grouped = matches
            .into_iter()
            .fold(HashMap::<Uuid, Vec<Match>>::new(), |mut acc, m| {
                acc.entry(m.tournament_id).or_default().push(m);
                acc
            });

        Ok(grouped)
    }
}
