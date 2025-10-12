use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use async_graphql::*;
use uuid::Uuid;

#[derive(Default)]
pub struct MatchesQuery;

#[Object]
impl MatchesQuery {
    async fn matches(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The tournament ID")] tournament_id: ID,
    ) -> Result<Vec<Match>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let tournament_uuid = Uuid::parse_str(&tournament_id)
            .map_err(|_| Error::new("Invalid tournament ID"))?;

        let tournament = models::Tournament::find_by_id(&gql_ctx.pool, tournament_uuid)
            .await?
            .ok_or_else(|| Error::new("Tournament not found"))?;

        if tournament.group_id != group_id {
            return Err(Error::new("Unauthorized"));
        }

        let matches = models::Match::find_by_tournament_id(&gql_ctx.pool, tournament_uuid).await?;

        Ok(matches.into_iter().map(Match::from).collect())
    }
}
