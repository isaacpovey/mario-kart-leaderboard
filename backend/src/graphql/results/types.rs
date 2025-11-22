use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use async_graphql::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct PlayerRaceResult {
    pub match_id: Uuid,
    pub round_number: i32,
    pub player_id: Uuid,
    pub position: i32,
    pub all_time_elo_change: Option<i32>,
    pub all_time_elo_after: Option<i32>,
    pub tournament_elo_change: Option<i32>,
    pub tournament_elo_after: Option<i32>,
}

impl From<crate::models::PlayerRaceScore> for PlayerRaceResult {
    fn from(model: crate::models::PlayerRaceScore) -> Self {
        Self {
            match_id: model.match_id,
            round_number: model.round_number,
            player_id: model.player_id,
            position: model.position,
            all_time_elo_change: model.all_time_elo_change,
            all_time_elo_after: model.all_time_elo_after,
            tournament_elo_change: model.tournament_elo_change,
            tournament_elo_after: model.tournament_elo_after,
        }
    }
}

#[Object]
impl PlayerRaceResult {
    async fn player(&self, ctx: &Context<'_>) -> Result<Player> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let player = gql_ctx
            .player_loader
            .load_one(self.player_id)
            .await?
            .ok_or_else(|| Error::new("Player not found"))?;

        Ok(Player::from(player))
    }

    async fn position(&self) -> i32 {
        self.position
    }

    async fn all_time_elo_change(&self) -> Option<i32> {
        self.all_time_elo_change
    }

    async fn all_time_elo_after(&self) -> Option<i32> {
        self.all_time_elo_after
    }

    async fn tournament_elo_change(&self) -> Option<i32> {
        self.tournament_elo_change
    }

    async fn tournament_elo_after(&self) -> Option<i32> {
        self.tournament_elo_after
    }
}

#[derive(Clone)]
pub struct PlayerMatchResult {
    pub match_id: Uuid,
    pub player_id: Uuid,
    pub position: i32,
    pub elo_change: i32,
    pub tournament_elo_change: i32,
    pub tournament_elo_from_races: i32,
    pub tournament_elo_from_contributions: i32,
}

impl From<crate::models::PlayerMatchScore> for PlayerMatchResult {
    fn from(model: crate::models::PlayerMatchScore) -> Self {
        Self {
            match_id: model.match_id,
            player_id: model.player_id,
            position: model.position,
            elo_change: model.elo_change,
            tournament_elo_change: model.tournament_elo_change,
            tournament_elo_from_races: model.tournament_elo_from_races,
            tournament_elo_from_contributions: model.tournament_elo_from_contributions,
        }
    }
}

#[Object]
impl PlayerMatchResult {
    async fn player(&self, ctx: &Context<'_>) -> Result<Player> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let player = gql_ctx
            .player_loader
            .load_one(self.player_id)
            .await?
            .ok_or_else(|| Error::new("Player not found"))?;

        Ok(Player::from(player))
    }

    async fn position(&self) -> i32 {
        self.position
    }

    async fn elo_change(&self) -> i32 {
        self.elo_change
    }

    async fn tournament_elo_change(&self) -> i32 {
        self.tournament_elo_change
    }

    async fn tournament_elo_from_races(&self) -> i32 {
        self.tournament_elo_from_races
    }

    async fn tournament_elo_from_contributions(&self) -> i32 {
        self.tournament_elo_from_contributions
    }

    async fn teammate_contribution(&self, ctx: &Context<'_>) -> Result<i32> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let contribution = gql_ctx
            .player_teammate_contribution_loader
            .load_one((self.match_id, self.player_id))
            .await?
            .unwrap_or(0);

        Ok(contribution)
    }
}
