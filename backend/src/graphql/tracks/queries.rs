use crate::graphql::context::GraphQLContext;
use crate::graphql::tracks::types::Track;
use crate::models;
use async_graphql::*;

#[derive(Default)]
pub struct TracksQuery;

#[Object]
impl TracksQuery {
    /// Get all available tracks
    async fn tracks(&self, ctx: &Context<'_>) -> Result<Vec<Track>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let tracks = models::Track::find_all(&gql_ctx.pool).await?;

        Ok(tracks.into_iter().map(Track::from).collect())
    }
}
