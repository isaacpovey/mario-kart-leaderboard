use crate::graphql::results::types::{PlayerMatchResult, PlayerRaceResult};
use crate::graphql::teams::types::Team;
use crate::graphql::tournaments::types::LeaderboardEntry;
use async_graphql::*;
use uuid::Uuid;

/// Race result update payload for subscriptions
///
/// Contains all data needed for the frontend to update its cache automatically
/// when race results are recorded. Structured to match existing query shapes
/// for seamless graphcache integration.
#[derive(Clone)]
pub struct RaceResultUpdate {
    pub match_id: Uuid,
    pub tournament_id: Uuid,
    pub round_number: i32,
    pub race_results: Vec<PlayerRaceResult>,
    pub player_aggregates: Vec<PlayerMatchResult>,
    pub leaderboard: Vec<LeaderboardEntry>,
    pub round_completed: bool,
    pub match_completed: bool,
    pub teams: Vec<Team>,
}

#[Object]
impl RaceResultUpdate {
    /// ID of the match these results belong to
    async fn match_id(&self) -> ID {
        ID(self.match_id.to_string())
    }

    /// ID of the tournament these results belong to
    async fn tournament_id(&self) -> ID {
        ID(self.tournament_id.to_string())
    }

    /// Round number (1-indexed)
    async fn round_number(&self) -> i32 {
        self.round_number
    }

    /// Race results for this specific round
    async fn race_results(&self) -> &[PlayerRaceResult] {
        &self.race_results
    }

    /// Updated player match aggregates (average position, total ELO changes)
    async fn player_aggregates(&self) -> &[PlayerMatchResult] {
        &self.player_aggregates
    }

    /// Updated tournament leaderboard
    async fn leaderboard(&self) -> &[LeaderboardEntry] {
        &self.leaderboard
    }

    /// Whether this round is now completed
    async fn round_completed(&self) -> bool {
        self.round_completed
    }

    /// Whether the entire match is now completed
    async fn match_completed(&self) -> bool {
        self.match_completed
    }

    /// Team information with updated scores (if match completed)
    async fn teams(&self) -> &[Team] {
        &self.teams
    }
}
