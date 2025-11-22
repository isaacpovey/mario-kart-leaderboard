use async_graphql::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::graphql::results::types::PlayerMatchResult;

#[derive(Clone)]
pub struct Match {
    pub id: Uuid,
    pub group_id: Uuid,
    pub tournament_id: Uuid,
    pub time: DateTime<Utc>,
    pub num_of_rounds: i32,
    pub completed: bool,
}

impl From<crate::models::Match> for Match {
    fn from(model: crate::models::Match) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            tournament_id: model.tournament_id,
            time: model.time,
            num_of_rounds: model.num_of_rounds,
            completed: model.completed,
        }
    }
}

#[Object]
impl Match {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn tournament_id(&self) -> ID {
        ID(self.tournament_id.to_string())
    }

    async fn time(&self) -> String {
        self.time.to_rfc3339()
    }

    async fn num_of_rounds(&self) -> i32 {
        self.num_of_rounds
    }

    async fn rounds(&self, ctx: &Context<'_>) -> Result<Vec<crate::graphql::rounds::Round>> {
        let context = ctx.data_unchecked::<crate::graphql::GraphQLContext>();
        let rounds = context
            .rounds_by_match_loader
            .load_one(self.id)
            .await?
            .unwrap_or_default();
        Ok(rounds.into_iter().map(|r| r.into()).collect())
    }

    async fn teams(&self, ctx: &Context<'_>) -> Result<Vec<crate::graphql::teams::Team>> {
        let context = ctx.data_unchecked::<crate::graphql::GraphQLContext>();

        let teams_with_players = crate::models::Team::get_by_match_with_players(&context.pool, self.id).await?;

        Ok(teams_with_players
            .into_iter()
            .map(|(team, players)| crate::graphql::teams::Team::from_with_players(team, players))
            .collect())
    }

    async fn completed(&self) -> bool {
        self.completed
    }

    async fn player_results(&self, ctx: &Context<'_>) -> Result<Vec<PlayerMatchResult>> {
        let context = ctx.data_unchecked::<crate::graphql::GraphQLContext>();
        let results = context
            .player_match_scores_by_match_loader
            .load_one(self.id)
            .await?
            .unwrap_or_default();
        Ok(results.into_iter().map(|r| r.into()).collect())
    }
}
