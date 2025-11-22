use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use async_graphql::*;
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Clone)]
pub struct Tournament {
    pub id: Uuid,
    pub group_id: Uuid,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub winner: Option<Uuid>,
}

impl From<crate::models::Tournament> for Tournament {
    fn from(model: crate::models::Tournament) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            start_date: model.start_date,
            end_date: model.end_date,
            winner: model.winner,
        }
    }
}

#[Object]
impl Tournament {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn start_date(&self) -> Option<String> {
        self.start_date.map(|d| d.to_string())
    }

    async fn end_date(&self) -> Option<String> {
        self.end_date.map(|d| d.to_string())
    }

    async fn winner_id(&self) -> Option<ID> {
        self.winner.map(|id| ID(id.to_string()))
    }

    async fn matches(&self, ctx: &Context<'_>) -> Result<Vec<Match>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let matches = gql_ctx
            .matches_by_tournament_loader
            .load_one(self.id)
            .await?
            .unwrap_or_default();

        Ok(matches.into_iter().map(Match::from).collect())
    }

    async fn leaderboard(&self, ctx: &Context<'_>) -> Result<Vec<LeaderboardEntry>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let entries =
            models::PlayerTournamentScore::get_tournament_leaderboard(&gql_ctx.pool, self.id)
                .await?;

        Ok(entries
            .into_iter()
            .map(|(player_id, player_name, elo_rating, all_time_elo, avatar_filename)| LeaderboardEntry {
                player_id,
                player_name,
                elo_rating,
                all_time_elo,
                total_score: elo_rating,
                avatar_filename,
            })
            .collect())
    }
}

#[derive(Clone)]
pub struct LeaderboardEntry {
    pub player_id: Uuid,
    pub player_name: String,
    pub elo_rating: i32,
    pub all_time_elo: i32,
    pub total_score: i32,
    pub avatar_filename: Option<String>,
}

#[Object]
impl LeaderboardEntry {
    async fn player_id(&self) -> ID {
        ID(self.player_id.to_string())
    }

    async fn player_name(&self) -> &str {
        &self.player_name
    }

    async fn elo_rating(&self) -> i32 {
        self.elo_rating
    }

    async fn all_time_elo(&self) -> i32 {
        self.all_time_elo
    }

    async fn total_score(&self) -> i32 {
        self.total_score
    }

    async fn avatar_filename(&self) -> Option<&str> {
        self.avatar_filename.as_deref()
    }
}

#[derive(Clone)]
pub struct ActiveTournamentWithLeaderboard {
    pub tournament: Tournament,
    pub leaderboard: Vec<LeaderboardEntry>,
}

#[Object]
impl ActiveTournamentWithLeaderboard {
    async fn id(&self) -> ID {
        ID(self.tournament.id.to_string())
    }

    async fn start_date(&self) -> Option<String> {
        self.tournament.start_date.map(|d| d.to_string())
    }

    async fn end_date(&self) -> Option<String> {
        self.tournament.end_date.map(|d| d.to_string())
    }

    async fn winner_id(&self) -> Option<ID> {
        self.tournament.winner.map(|id| ID(id.to_string()))
    }

    async fn leaderboard(&self) -> &[LeaderboardEntry] {
        &self.leaderboard
    }

    async fn matches(&self, ctx: &Context<'_>) -> Result<Vec<Match>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let matches = gql_ctx
            .matches_by_tournament_loader
            .load_one(self.tournament.id)
            .await?
            .unwrap_or_default();

        Ok(matches.into_iter().map(Match::from).collect())
    }
}
