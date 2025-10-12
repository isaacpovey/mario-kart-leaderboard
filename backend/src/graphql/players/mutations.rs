use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use crate::models;
use crate::validation::validate_name;
use async_graphql::*;

#[derive(Default)]
pub struct PlayersMutation;

#[Object]
impl PlayersMutation {
    async fn create_player(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The player name")] name: String,
    ) -> Result<Player> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        // Validate input
        validate_name(&name, "Player name")?;

        let player = models::Player::create(&gql_ctx.pool, group_id, name.trim()).await?;

        Ok(Player::from(player))
    }
}
