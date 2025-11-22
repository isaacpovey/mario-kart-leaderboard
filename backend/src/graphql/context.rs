use crate::db::DbPool;
use crate::graphql::groups::GroupLoader;
use crate::graphql::matches::{MatchLoader, MatchesByTournamentLoader};
use crate::graphql::players::{PlayerActiveTournamentEloLoader, PlayerLoader, PlayersByGroupLoader};
use crate::graphql::results::{PlayerMatchScoresByMatchLoader, PlayerRaceScoresByRoundLoader, PlayerTeammateContributionLoader};
use crate::graphql::rounds::{PlayersByRoundLoader, RoundsByMatchLoader};
use crate::graphql::teams::{PlayersByTeamLoader, TeamsByMatchLoader};
use crate::graphql::tournaments::{ActiveTournamentLoader, PlayerTournamentEloLoader, TournamentLoader};
use crate::graphql::tracks::TrackLoader;
use async_graphql::dataloader::{DataLoader, HashMapCache};
use std::sync::Arc;
use uuid::Uuid;

pub struct GraphQLContext {
    pub pool: DbPool,
    pub group_id: Option<Uuid>,
    pub group_loader: Arc<DataLoader<GroupLoader, HashMapCache>>,
    pub player_loader: Arc<DataLoader<PlayerLoader, HashMapCache>>,
    pub players_by_group_loader: Arc<DataLoader<PlayersByGroupLoader, HashMapCache>>,
    pub player_active_tournament_elo_loader:
        Arc<DataLoader<PlayerActiveTournamentEloLoader, HashMapCache>>,
    pub tournament_loader: Arc<DataLoader<TournamentLoader, HashMapCache>>,
    pub active_tournament_loader: Arc<DataLoader<ActiveTournamentLoader, HashMapCache>>,
    pub player_tournament_elo_loader: Arc<DataLoader<PlayerTournamentEloLoader, HashMapCache>>,
    pub match_loader: Arc<DataLoader<MatchLoader, HashMapCache>>,
    pub matches_by_tournament_loader: Arc<DataLoader<MatchesByTournamentLoader, HashMapCache>>,
    pub rounds_by_match_loader: Arc<DataLoader<RoundsByMatchLoader, HashMapCache>>,
    pub players_by_round_loader: Arc<DataLoader<PlayersByRoundLoader, HashMapCache>>,
    pub teams_by_match_loader: Arc<DataLoader<TeamsByMatchLoader, HashMapCache>>,
    pub players_by_team_loader: Arc<DataLoader<PlayersByTeamLoader, HashMapCache>>,
    pub track_loader: Arc<DataLoader<TrackLoader, HashMapCache>>,
    pub player_race_scores_by_round_loader:
        Arc<DataLoader<PlayerRaceScoresByRoundLoader, HashMapCache>>,
    pub player_match_scores_by_match_loader:
        Arc<DataLoader<PlayerMatchScoresByMatchLoader, HashMapCache>>,
    pub player_teammate_contribution_loader:
        Arc<DataLoader<PlayerTeammateContributionLoader, HashMapCache>>,
}

impl GraphQLContext {
    pub fn new(pool: DbPool, group_id: Option<Uuid>) -> Self {
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
            player_active_tournament_elo_loader: Arc::new(DataLoader::with_cache(
                PlayerActiveTournamentEloLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            tournament_loader: Arc::new(DataLoader::with_cache(
                TournamentLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            active_tournament_loader: Arc::new(DataLoader::with_cache(
                ActiveTournamentLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            player_tournament_elo_loader: Arc::new(DataLoader::with_cache(
                PlayerTournamentEloLoader::new(pool.clone()),
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
            player_race_scores_by_round_loader: Arc::new(DataLoader::with_cache(
                PlayerRaceScoresByRoundLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            player_match_scores_by_match_loader: Arc::new(DataLoader::with_cache(
                PlayerMatchScoresByMatchLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            player_teammate_contribution_loader: Arc::new(DataLoader::with_cache(
                PlayerTeammateContributionLoader::new(pool.clone()),
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
