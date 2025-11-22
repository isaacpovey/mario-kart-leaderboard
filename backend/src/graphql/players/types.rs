use crate::graphql::context::GraphQLContext;
use crate::graphql::groups::types::Group;
use async_graphql::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Player {
    pub id: Uuid,
    pub group_id: Uuid,
    pub name: String,
    pub elo_rating: i32,
    pub avatar_filename: Option<String>,
}

impl From<crate::models::Player> for Player {
    fn from(model: crate::models::Player) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            name: model.name,
            elo_rating: model.elo_rating,
            avatar_filename: model.avatar_filename,
        }
    }
}

#[Object]
impl Player {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn elo_rating(&self) -> i32 {
        self.elo_rating
    }

    async fn avatar_filename(&self) -> Option<&str> {
        self.avatar_filename.as_deref()
    }

    async fn current_tournament_elo(&self, ctx: &Context<'_>) -> Result<Option<i32>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let tournament_elo = gql_ctx
            .player_active_tournament_elo_loader
            .load_one((self.id, self.group_id))
            .await?;

        Ok(tournament_elo)
    }

    async fn group(&self, ctx: &Context<'_>) -> Result<Option<Group>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let group = gql_ctx
            .group_loader
            .load_one(self.group_id)
            .await?
            .map(Group::from);

        Ok(group)
    }
}
