use crate::graphql::context::GraphQLContext;
use crate::graphql::groups::types::Group;
use crate::models;
use async_graphql::*;

#[derive(Default)]
pub struct GroupsQuery;

#[Object]
impl GroupsQuery {
    async fn current_group(&self, ctx: &Context<'_>) -> Result<Group> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let group = models::Group::find_by_id(&gql_ctx.pool, group_id)
            .await?
            .ok_or_else(|| Error::new("Group not found"))?;

        Ok(Group::from(group))
    }
}
