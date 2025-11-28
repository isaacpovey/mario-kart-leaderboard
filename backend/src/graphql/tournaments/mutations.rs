use crate::graphql::context::GraphQLContext;
use crate::graphql::tournaments::types::Tournament;
use crate::models;
use crate::services::tournament_completion;
use async_graphql::*;
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Default)]
pub struct TournamentsMutation;

#[Object]
impl TournamentsMutation {
    async fn create_tournament(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The tournament start date (YYYY-MM-DD)")] start_date: Option<String>,
        #[graphql(desc = "The tournament end date (YYYY-MM-DD)")] end_date: Option<String>,
    ) -> Result<Tournament> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let start = start_date
            .map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
            .transpose()
            .map_err(|_| Error::new("Invalid start date format. Use YYYY-MM-DD"))?;

        let end = end_date
            .map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
            .transpose()
            .map_err(|_| Error::new("Invalid end date format. Use YYYY-MM-DD"))?;

        let tournament = models::Tournament::create(&gql_ctx.pool, group_id, start, end).await?;

        Ok(Tournament::from(tournament))
    }

    async fn complete_tournament(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The tournament ID")] tournament_id: ID,
    ) -> Result<Tournament> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let tournament_uuid = Uuid::parse_str(&tournament_id)
            .map_err(|_| Error::new("Invalid tournament ID"))?;

        let tournament = tournament_completion::complete_tournament(
            &gql_ctx.pool,
            tournament_uuid,
            group_id,
        )
        .await?;

        Ok(Tournament::from(tournament))
    }
}
