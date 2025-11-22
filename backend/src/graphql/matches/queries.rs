use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use async_graphql::*;
use uuid::Uuid;

#[derive(Default)]
pub struct MatchesQuery;

#[Object]
impl MatchesQuery {
    async fn match_by_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The match ID")] match_id: ID,
    ) -> Result<Match> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let match_uuid = Uuid::parse_str(&match_id).map_err(|_| Error::new("Invalid match ID"))?;

        let match_record = models::Match::find_by_id(&gql_ctx.pool, match_uuid)
            .await?
            .ok_or_else(|| Error::new("Match not found"))?;

        if match_record.group_id != group_id {
            return Err(Error::new("Unauthorized"));
        }

        Ok(Match::from(match_record))
    }
}
