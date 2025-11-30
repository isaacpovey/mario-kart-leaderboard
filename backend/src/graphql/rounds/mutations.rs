use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use crate::services::result_recording;
use async_graphql::*;
use sqlx;
use uuid::Uuid;

#[derive(Default)]
pub struct RoundsMutation;

#[derive(InputObject)]
pub struct PlayerResultInput {
    pub player_id: ID,
    pub position: i32,
}

#[Object]
impl RoundsMutation {
    async fn record_round_results(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The match ID")] match_id: ID,
        #[graphql(desc = "The round number")] round_number: i32,
        #[graphql(desc = "Player results for this round")] results: Vec<PlayerResultInput>,
    ) -> Result<Match> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let match_uuid = Uuid::parse_str(&match_id).map_err(|_| Error::new("Invalid match ID"))?;

        let player_uuids_with_positions: Result<Vec<(Uuid, i32)>> = results
            .iter()
            .map(|r| {
                Uuid::parse_str(&r.player_id)
                    .map_err(|_| Error::new("Invalid player ID"))
                    .map(|uuid| (uuid, r.position))
            })
            .collect();
        let player_uuids_with_positions = player_uuids_with_positions?;

        let positions: Vec<i32> = player_uuids_with_positions
            .iter()
            .map(|(_, pos)| *pos)
            .collect();
        if positions.iter().any(|&p| !(1..=24).contains(&p)) {
            return Err(Error::new("Positions must be between 1 and 24"));
        }
        let unique_positions: std::collections::HashSet<i32> = positions.iter().copied().collect();
        if unique_positions.len() != positions.len() {
            return Err(Error::new("Duplicate positions are not allowed"));
        }
        if player_uuids_with_positions.is_empty() {
            return Err(Error::new("At least one player result is required"));
        }

        let match_record = models::Match::find_by_id(&gql_ctx.pool, match_uuid)
            .await?
            .ok_or_else(|| Error::new("Match not found"))?;

        if match_record.group_id != group_id {
            return Err(Error::new("Unauthorized"));
        }

        if match_record.completed {
            return Err(Error::new("Match is already completed"));
        }

        let round = models::Round::find_one(&gql_ctx.pool, match_uuid, round_number)
            .await?
            .ok_or_else(|| Error::new("Round not found"))?;

        if round.completed {
            return Err(Error::new("Round is already completed"));
        }

        let round_players =
            result_recording::get_round_players(&gql_ctx.pool, match_uuid, round_number).await?;
        let player_uuids: Vec<Uuid> = player_uuids_with_positions
            .iter()
            .map(|(uuid, _)| *uuid)
            .collect();
        result_recording::validate_players_in_round(&player_uuids, &round_players)?;

        let updated_match = result_recording::record_race_results(
            &gql_ctx.pool,
            group_id,
            match_uuid,
            round_number,
            &player_uuids_with_positions,
            &match_record,
            &gql_ctx.notification_manager,
        )
        .await?;

        Ok(Match::from(updated_match))
    }

    async fn swap_round_player(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The match ID")] match_id: ID,
        #[graphql(desc = "The round number")] round_number: i32,
        #[graphql(desc = "The player to replace")] current_player_id: ID,
        #[graphql(desc = "The replacement player")] new_player_id: ID,
    ) -> Result<Match> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let match_uuid =
            Uuid::parse_str(&match_id).map_err(|_| Error::new("Invalid match ID"))?;
        let current_player_uuid =
            Uuid::parse_str(&current_player_id).map_err(|_| Error::new("Invalid current player ID"))?;
        let new_player_uuid =
            Uuid::parse_str(&new_player_id).map_err(|_| Error::new("Invalid new player ID"))?;

        let match_record = models::Match::find_by_id(&gql_ctx.pool, match_uuid)
            .await?
            .ok_or_else(|| Error::new("Match not found"))?;

        if match_record.group_id != group_id {
            return Err(Error::new("Match not found"));
        }

        let round = models::Round::find_one(&gql_ctx.pool, match_uuid, round_number)
            .await?
            .ok_or_else(|| Error::new("Round not found"))?;

        if round.completed {
            return Err(Error::new("Cannot swap players in a completed round"));
        }

        let current_round_player: Option<(Uuid, i32)> = sqlx::query_as(
            "SELECT team_id, player_position FROM round_players
             WHERE match_id = $1 AND round_number = $2 AND player_id = $3",
        )
        .bind(match_uuid)
        .bind(round_number)
        .bind(current_player_uuid)
        .fetch_optional(&gql_ctx.pool)
        .await?;

        let (team_id, player_position) = current_round_player
            .ok_or_else(|| Error::new("Current player not found in this round"))?;

        let is_on_same_team: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM team_players WHERE team_id = $1 AND player_id = $2)",
        )
        .bind(team_id)
        .bind(new_player_uuid)
        .fetch_one(&gql_ctx.pool)
        .await?;

        if !is_on_same_team {
            return Err(Error::new("Replacement player must be on the same team"));
        }

        let already_in_round: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM round_players WHERE match_id = $1 AND round_number = $2 AND player_id = $3)",
        )
        .bind(match_uuid)
        .bind(round_number)
        .bind(new_player_uuid)
        .fetch_one(&gql_ctx.pool)
        .await?;

        if already_in_round {
            return Err(Error::new("Replacement player is already in this round"));
        }

        sqlx::query(
            "DELETE FROM round_players WHERE match_id = $1 AND round_number = $2 AND player_id = $3",
        )
        .bind(match_uuid)
        .bind(round_number)
        .bind(current_player_uuid)
        .execute(&gql_ctx.pool)
        .await?;

        sqlx::query(
            "INSERT INTO round_players (group_id, match_id, round_number, player_id, team_id, player_position)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(group_id)
        .bind(match_uuid)
        .bind(round_number)
        .bind(new_player_uuid)
        .bind(team_id)
        .bind(player_position)
        .execute(&gql_ctx.pool)
        .await?;

        let updated_match = models::Match::find_by_id(&gql_ctx.pool, match_uuid)
            .await?
            .ok_or_else(|| Error::new("Match not found after swap"))?;

        Ok(Match::from(updated_match))
    }
}
