use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use crate::services::match_service;
use async_graphql::*;
use uuid::Uuid;

const DEFAULT_PLAYERS_PER_RACE: i32 = 4;

#[derive(Default)]
pub struct MatchesMutation;

#[Object]
impl MatchesMutation {
    async fn create_match_with_rounds(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The tournament ID")] tournament_id: ID,
        #[graphql(desc = "The player IDs participating in this match")] player_ids: Vec<ID>,
        #[graphql(desc = "The number of races")] num_races: i32,
        #[graphql(desc = "The number of players per race (default: 4)")] players_per_race: Option<
            i32,
        >,
    ) -> Result<Match> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let tournament_uuid =
            Uuid::parse_str(&tournament_id).map_err(|_| Error::new("Invalid tournament ID"))?;

        let player_uuids: Result<Vec<Uuid>> = player_ids
            .iter()
            .map(|id| Uuid::parse_str(id).map_err(|_| Error::new("Invalid player ID")))
            .collect();
        let player_uuids = player_uuids?;

        let players_per_race = players_per_race.unwrap_or(DEFAULT_PLAYERS_PER_RACE);

        let tournament = models::Tournament::find_by_id(&gql_ctx.pool, tournament_uuid)
            .await?
            .ok_or_else(|| Error::new("Tournament not found"))?;

        if tournament.group_id != group_id {
            return Err(Error::new("Tournament not found"));
        }

        let players = models::Player::find_by_ids(&gql_ctx.pool, &player_uuids).await?;

        if players.iter().any(|p| p.group_id != group_id) {
            return Err(Error::new("One or more players cound not be found"));
        }

        let match_result = match_service::create_match_with_rounds(
            &gql_ctx.pool,
            group_id,
            tournament_uuid,
            &player_uuids,
            num_races,
            players_per_race,
            &gql_ctx.notification_manager,
        )
        .await?;

        Ok(Match::from(match_result))
    }
}
