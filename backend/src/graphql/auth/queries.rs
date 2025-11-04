use crate::auth::{create_jwt, verify_password};
use crate::config::Config;
use crate::graphql::context::GraphQLContext;
use crate::models;
use async_graphql::*;
use uuid::Uuid;

#[derive(Default)]
pub struct AuthQuery;

#[Object]
impl AuthQuery {
    async fn login(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The group ID")] group_id: ID,
        #[graphql(desc = "The group password")] password: String,
    ) -> Result<String> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let config = ctx.data::<Config>()?;

        let group_uuid = Uuid::parse_str(&group_id).map_err(|_| Error::new("Invalid group ID"))?;

        let group = models::Group::find_by_id(&gql_ctx.pool, group_uuid)
            .await?
            .ok_or_else(|| Error::new("Invalid credentials"))?;

        // Verify password - returns Err if invalid
        verify_password(&password, &group.password)
            .map_err(|_| Error::new("Invalid credentials"))?;

        let token = create_jwt(group_uuid, &config.jwt_secret)?;

        Ok(token)
    }
}
