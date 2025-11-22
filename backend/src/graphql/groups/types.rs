use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use async_graphql::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
}

impl From<crate::models::Group> for Group {
    fn from(model: crate::models::Group) -> Self {
        Self {
            id: model.id,
            name: model.name,
        }
    }
}

#[Object]
impl Group {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn players(&self, ctx: &Context<'_>) -> Result<Vec<Player>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let players = gql_ctx
            .players_by_group_loader
            .load_one(self.id)
            .await?
            .unwrap_or_default();

        Ok(players.into_iter().map(Player::from).collect())
    }
}
