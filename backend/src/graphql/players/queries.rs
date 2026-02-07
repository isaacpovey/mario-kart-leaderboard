use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use crate::models;
use async_graphql::*;
use uuid::Uuid;

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

    async fn player_by_id(&self, ctx: &Context<'_>, player_id: ID) -> Result<Option<Player>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let player_uuid = Uuid::parse_str(&player_id)
            .map_err(|_| Error::new("Invalid player ID format"))?;

        let player = models::Player::find_by_id(&gql_ctx.pool, player_uuid).await?;

        match player {
            Some(p) if p.group_id == group_id => Ok(Some(Player::from(p))),
            Some(_) => Err(Error::new("Player not found")),
            None => Ok(None),
        }
    }
}
