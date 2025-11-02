use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use crate::graphql::tracks::types::Track;
use async_graphql::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Round {
    pub match_id: Uuid,
    pub round_number: i32,
    pub track_id: Option<Uuid>,
    pub completed: bool,
}

impl From<crate::models::Round> for Round {
    fn from(model: crate::models::Round) -> Self {
        Self {
            match_id: model.match_id,
            round_number: model.round_number,
            track_id: model.track_id,
            completed: model.completed,
        }
    }
}

#[Object]
impl Round {
    async fn round_number(&self) -> i32 {
        self.round_number
    }

    async fn track_id(&self) -> Option<ID> {
        self.track_id.map(|id| ID(id.to_string()))
    }

    async fn completed(&self) -> bool {
        self.completed
    }

    async fn track(&self, ctx: &Context<'_>) -> Result<Option<Track>> {
        let Some(track_id) = self.track_id else {
            return Ok(None);
        };

        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let track = gql_ctx
            .track_loader
            .load_one(track_id)
            .await?
            .map(Track::from);

        Ok(track)
    }

    async fn players(&self, ctx: &Context<'_>) -> Result<Vec<Player>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let players = gql_ctx
            .players_by_round_loader
            .load_one((self.match_id, self.round_number))
            .await?
            .unwrap_or_default()
            .into_iter()
            .map(Player::from)
            .collect();

        Ok(players)
    }
}
