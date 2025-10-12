use crate::auth::{create_jwt, hash_password};
use crate::config::Config;
use crate::graphql::context::GraphQLContext;
use crate::models;
use crate::validation::{validate_name, validate_password};
use async_graphql::*;

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    async fn create_group(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The group name")] name: String,
        #[graphql(desc = "The group password")] password: String,
    ) -> Result<String> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let config = ctx.data::<Config>()?;

        // Validate inputs
        validate_name(&name, "Group name")?;
        validate_password(&password)?;

        let password_hash = hash_password(&password)?;

        let group = models::Group::create(&gql_ctx.pool, name.trim(), &password_hash).await?;

        let token = create_jwt(group.id, &config.jwt_secret)?;

        Ok(token)
    }
}
