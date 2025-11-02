use crate::graphql::groups::GroupLoader;
use crate::graphql::matches::{MatchLoader, MatchesByTournamentLoader};
use crate::graphql::players::{PlayerLoader, PlayersByGroupLoader};
use crate::graphql::rounds::{PlayersByRoundLoader, RoundsByMatchLoader};
use crate::graphql::teams::{PlayersByTeamLoader, TeamsByMatchLoader};
use crate::graphql::tournaments::TournamentLoader;
use crate::graphql::tracks::TrackLoader;
use async_graphql::dataloader::DataLoader;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct GraphQLContext {
    pub pool: PgPool,
    pub group_id: Option<Uuid>,
    pub group_loader: Arc<DataLoader<GroupLoader>>,
    pub player_loader: Arc<DataLoader<PlayerLoader>>,
    pub players_by_group_loader: Arc<DataLoader<PlayersByGroupLoader>>,
    pub tournament_loader: Arc<DataLoader<TournamentLoader>>,
    pub match_loader: Arc<DataLoader<MatchLoader>>,
    pub matches_by_tournament_loader: Arc<DataLoader<MatchesByTournamentLoader>>,
    pub rounds_by_match_loader: Arc<DataLoader<RoundsByMatchLoader>>,
    pub players_by_round_loader: Arc<DataLoader<PlayersByRoundLoader>>,
    pub teams_by_match_loader: Arc<DataLoader<TeamsByMatchLoader>>,
    pub players_by_team_loader: Arc<DataLoader<PlayersByTeamLoader>>,
    pub track_loader: Arc<DataLoader<TrackLoader>>,
}

impl GraphQLContext {
    pub fn new(pool: PgPool, group_id: Option<Uuid>) -> Self {
        // Note: PgPool cloning is cheap (Arc-based), so these clones are acceptable
        Self {
            group_loader: Arc::new(DataLoader::new(
                GroupLoader::new(pool.clone()),
                tokio::spawn,
            )),
            player_loader: Arc::new(DataLoader::new(
                PlayerLoader::new(pool.clone()),
                tokio::spawn,
            )),
            players_by_group_loader: Arc::new(DataLoader::new(
                PlayersByGroupLoader::new(pool.clone()),
                tokio::spawn,
            )),
            tournament_loader: Arc::new(DataLoader::new(
                TournamentLoader::new(pool.clone()),
                tokio::spawn,
            )),
            match_loader: Arc::new(DataLoader::new(
                MatchLoader::new(pool.clone()),
                tokio::spawn,
            )),
            matches_by_tournament_loader: Arc::new(DataLoader::new(
                MatchesByTournamentLoader::new(pool.clone()),
                tokio::spawn,
            )),
            rounds_by_match_loader: Arc::new(DataLoader::new(
                RoundsByMatchLoader::new(pool.clone()),
                tokio::spawn,
            )),
            players_by_round_loader: Arc::new(DataLoader::new(
                PlayersByRoundLoader::new(pool.clone()),
                tokio::spawn,
            )),
            teams_by_match_loader: Arc::new(DataLoader::new(
                TeamsByMatchLoader::new(pool.clone()),
                tokio::spawn,
            )),
            players_by_team_loader: Arc::new(DataLoader::new(
                PlayersByTeamLoader::new(pool.clone()),
                tokio::spawn,
            )),
            track_loader: Arc::new(DataLoader::new(
                TrackLoader::new(pool.clone()),
                tokio::spawn,
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
