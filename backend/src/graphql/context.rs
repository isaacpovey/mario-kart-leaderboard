use crate::graphql::groups::GroupLoader;
use crate::graphql::matches::{MatchLoader, MatchesByTournamentLoader};
use crate::graphql::players::{PlayerLoader, PlayersByGroupLoader};
use crate::graphql::rounds::{PlayersByRoundLoader, RoundsByMatchLoader};
use crate::graphql::teams::{PlayersByTeamLoader, TeamsByMatchLoader};
use crate::graphql::tournaments::TournamentLoader;
use crate::graphql::tracks::TrackLoader;
use async_graphql::dataloader::{DataLoader, HashMapCache};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct GraphQLContext {
    pub pool: PgPool,
    pub group_id: Option<Uuid>,
    pub group_loader: Arc<DataLoader<GroupLoader, HashMapCache>>,
    pub player_loader: Arc<DataLoader<PlayerLoader, HashMapCache>>,
    pub players_by_group_loader: Arc<DataLoader<PlayersByGroupLoader, HashMapCache>>,
    pub tournament_loader: Arc<DataLoader<TournamentLoader, HashMapCache>>,
    pub match_loader: Arc<DataLoader<MatchLoader, HashMapCache>>,
    pub matches_by_tournament_loader: Arc<DataLoader<MatchesByTournamentLoader, HashMapCache>>,
    pub rounds_by_match_loader: Arc<DataLoader<RoundsByMatchLoader, HashMapCache>>,
    pub players_by_round_loader: Arc<DataLoader<PlayersByRoundLoader, HashMapCache>>,
    pub teams_by_match_loader: Arc<DataLoader<TeamsByMatchLoader, HashMapCache>>,
    pub players_by_team_loader: Arc<DataLoader<PlayersByTeamLoader, HashMapCache>>,
    pub track_loader: Arc<DataLoader<TrackLoader, HashMapCache>>,
}

impl GraphQLContext {
    pub fn new(pool: PgPool, group_id: Option<Uuid>) -> Self {
        Self {
            group_loader: Arc::new(DataLoader::with_cache(
                GroupLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            player_loader: Arc::new(DataLoader::with_cache(
                PlayerLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            players_by_group_loader: Arc::new(DataLoader::with_cache(
                PlayersByGroupLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            tournament_loader: Arc::new(DataLoader::with_cache(
                TournamentLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            match_loader: Arc::new(DataLoader::with_cache(
                MatchLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            matches_by_tournament_loader: Arc::new(DataLoader::with_cache(
                MatchesByTournamentLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            rounds_by_match_loader: Arc::new(DataLoader::with_cache(
                RoundsByMatchLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            players_by_round_loader: Arc::new(DataLoader::with_cache(
                PlayersByRoundLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            teams_by_match_loader: Arc::new(DataLoader::with_cache(
                TeamsByMatchLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            players_by_team_loader: Arc::new(DataLoader::with_cache(
                PlayersByTeamLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            track_loader: Arc::new(DataLoader::with_cache(
                TrackLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            pool,
            group_id,
        }
    }

    pub fn authenticated_group_id(&self) -> Result<Uuid, async_graphql::Error> {
        self.group_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))
    }
}
