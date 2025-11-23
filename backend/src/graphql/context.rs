use crate::db::DbPool;
use crate::graphql::groups::GroupLoader;
use crate::graphql::matches::MatchesByTournamentLoader;
use crate::graphql::players::{PlayerActiveTournamentEloLoader, PlayerLoader, PlayersByGroupLoader};
use crate::graphql::results::{PlayerMatchScoresByMatchLoader, PlayerRaceScoresByRoundLoader, PlayerTeammateContributionLoader};
use crate::graphql::rounds::PlayersByRoundLoader;
use crate::graphql::teams::PlayersByTeamLoader;
use crate::graphql::tracks::TrackLoader;
use crate::services::notification_manager::NotificationManager;
use async_graphql::dataloader::{DataLoader, HashMapCache};
use std::sync::Arc;
use uuid::Uuid;

pub struct GraphQLContext {
    pub pool: DbPool,
    pub group_id: Option<Uuid>,
    pub notification_manager: NotificationManager,
    pub group_loader: Arc<DataLoader<GroupLoader, HashMapCache>>,
    pub player_loader: Arc<DataLoader<PlayerLoader, HashMapCache>>,
    pub players_by_group_loader: Arc<DataLoader<PlayersByGroupLoader, HashMapCache>>,
    pub player_active_tournament_elo_loader:
        Arc<DataLoader<PlayerActiveTournamentEloLoader, HashMapCache>>,
    pub matches_by_tournament_loader: Arc<DataLoader<MatchesByTournamentLoader, HashMapCache>>,
    pub players_by_round_loader: Arc<DataLoader<PlayersByRoundLoader, HashMapCache>>,
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
    pub fn new(
        pool: DbPool,
        group_id: Option<Uuid>,
        notification_manager: NotificationManager,
    ) -> Self {
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
            matches_by_tournament_loader: Arc::new(DataLoader::with_cache(
                MatchesByTournamentLoader::new(pool.clone()),
                tokio::spawn,
                HashMapCache::default(),
            )),
            players_by_round_loader: Arc::new(DataLoader::with_cache(
                PlayersByRoundLoader::new(pool.clone()),
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
            notification_manager,
        }
    }

    pub fn authenticated_group_id(&self) -> Result<Uuid, async_graphql::Error> {
        self.group_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))
    }
}
