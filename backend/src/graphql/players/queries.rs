use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use crate::models;
use async_graphql::*;

#[derive(Default)]
pub struct PlayersQuery;

#[Object]
impl PlayersQuery {
    async fn players(&self, ctx: &Context<'_>) -> Result<Vec<Player>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let players = models::Player::find_by_group_id(&gql_ctx.pool, group_id).await?;

        Ok(players.into_iter().map(Player::from).collect())
    }
}
