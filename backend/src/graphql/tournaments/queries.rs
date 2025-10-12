use crate::graphql::context::GraphQLContext;
use crate::graphql::tournaments::types::Tournament;
use crate::models;
use async_graphql::*;

#[derive(Default)]
pub struct TournamentsQuery;

#[Object]
impl TournamentsQuery {
    async fn tournaments(&self, ctx: &Context<'_>) -> Result<Vec<Tournament>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let tournaments = models::Tournament::find_by_group_id(&gql_ctx.pool, group_id).await?;

        Ok(tournaments.into_iter().map(Tournament::from).collect())
    }
}
