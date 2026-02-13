use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use crate::services::match_service;
use async_graphql::*;
use sqlx;
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
        #[graphql(desc = "Whether to assign teams randomly instead of by ELO balance (default: false)")]
        random_teams: Option<bool>,
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
        let random_teams = random_teams.unwrap_or(false);

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
            random_teams,
            &gql_ctx.notification_manager,
        )
        .await?;

        Ok(Match::from(match_result))
    }

    async fn cancel_match(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The match ID to cancel")] match_id: ID,
    ) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let match_uuid =
            Uuid::parse_str(&match_id).map_err(|_| Error::new("Invalid match ID"))?;

        let match_record = models::Match::find_by_id(&gql_ctx.pool, match_uuid)
            .await?
            .ok_or_else(|| Error::new("Match not found"))?;

        if match_record.group_id != group_id {
            return Err(Error::new("Match not found"));
        }

        let has_results: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM player_race_scores WHERE match_id = $1)",
        )
        .bind(match_uuid)
        .fetch_one(&gql_ctx.pool)
        .await?;

        if has_results {
            return Err(Error::new(
                "Cannot cancel match: race results have been recorded",
            ));
        }

        sqlx::query("DELETE FROM matches WHERE id = $1")
            .bind(match_uuid)
            .execute(&gql_ctx.pool)
            .await?;

        Ok(true)
    }
}
