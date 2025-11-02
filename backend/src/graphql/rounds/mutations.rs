use crate::elo::{self, PlayerResult};
use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use crate::scoring;
use async_graphql::*;
use std::collections::HashMap;
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

        validate_results(&results)?;

        let player_uuids: Result<Vec<Uuid>> = results
            .iter()
            .map(|r| Uuid::parse_str(&r.player_id).map_err(|_| Error::new("Invalid player ID")))
            .collect();
        let player_uuids = player_uuids?;

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

        let round_players = get_round_players(&gql_ctx.pool, match_uuid, round_number).await?;
        validate_players_in_round(&player_uuids, &round_players)?;

        let players = models::Player::find_by_ids(&gql_ctx.pool, &player_uuids).await?;
        let player_elos = create_player_elo_map(&players);

        let player_results = create_player_results(&results, &player_elos)?;
        let elo_changes = elo::calculate_elo_changes(&player_results);

        let updated_match = record_results_in_transaction(
            &gql_ctx.pool,
            group_id,
            match_uuid,
            round_number,
            &results,
            &elo_changes,
            &match_record,
        )
        .await?;

        Ok(Match::from(updated_match))
    }
}

fn validate_results(results: &[PlayerResultInput]) -> Result<()> {
    if results.is_empty() {
        return Err(Error::new("At least one player result is required"));
    }

    let positions: Vec<i32> = results.iter().map(|r| r.position).collect();

    if positions.iter().any(|&p| !(1..=24).contains(&p)) {
        return Err(Error::new("Positions must be between 1 and 24"));
    }

    let unique_positions: std::collections::HashSet<i32> = positions.iter().copied().collect();
    if unique_positions.len() != positions.len() {
        return Err(Error::new("Duplicate positions are not allowed"));
    }

    Ok(())
}

async fn get_round_players(
    pool: &sqlx::PgPool,
    match_id: Uuid,
    round_number: i32,
) -> Result<Vec<Uuid>> {
    let player_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT player_id FROM round_players
         WHERE match_id = $1 AND round_number = $2",
    )
    .bind(match_id)
    .bind(round_number)
    .fetch_all(pool)
    .await?;

    Ok(player_ids)
}

fn validate_players_in_round(
    submitted_players: &[Uuid],
    round_players: &[Uuid],
) -> Result<()> {
    let submitted_set: std::collections::HashSet<Uuid> =
        submitted_players.iter().copied().collect();
    let round_set: std::collections::HashSet<Uuid> = round_players.iter().copied().collect();

    if submitted_set != round_set {
        return Err(Error::new(
            "Results must include all players in this round, no more and no less",
        ));
    }

    Ok(())
}

fn create_player_elo_map(players: &[models::Player]) -> HashMap<Uuid, i32> {
    players
        .iter()
        .map(|p| (p.id, p.elo_rating))
        .collect()
}

fn create_player_results(
    results: &[PlayerResultInput],
    player_elos: &HashMap<Uuid, i32>,
) -> Result<Vec<PlayerResult>> {
    results
        .iter()
        .map(|r| {
            let player_id = Uuid::parse_str(&r.player_id)
                .map_err(|_| Error::new("Invalid player ID"))?;
            let current_elo = player_elos
                .get(&player_id)
                .ok_or_else(|| Error::new("Player not found"))?;

            Ok(PlayerResult {
                player_id,
                position: r.position,
                current_elo: *current_elo,
            })
        })
        .collect()
}

async fn record_results_in_transaction(
    pool: &sqlx::PgPool,
    group_id: Uuid,
    match_id: Uuid,
    round_number: i32,
    results: &[PlayerResultInput],
    elo_changes: &[elo::EloChange],
    match_record: &models::Match,
) -> Result<models::Match> {
    let mut tx = pool.begin().await?;

    for result in results {
        let player_id = Uuid::parse_str(&result.player_id)
            .map_err(|_| Error::new("Invalid player ID"))?;

        sqlx::query(
            "INSERT INTO player_race_scores (group_id, match_id, round_number, player_id, position)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(group_id)
        .bind(match_id)
        .bind(round_number)
        .bind(player_id)
        .bind(result.position)
        .execute(&mut *tx)
        .await?;
    }

    for change in elo_changes {
        sqlx::query(
            "UPDATE players
             SET elo_rating = $1
             WHERE id = $2",
        )
        .bind(change.new_elo)
        .bind(change.player_id)
        .execute(&mut *tx)
        .await?;
    }

    let player_match_updates =
        calculate_player_match_aggregates(&mut tx, match_id, elo_changes).await?;

    for (player_id, avg_position, total_elo_change) in player_match_updates {
        sqlx::query(
            "INSERT INTO player_match_scores (group_id, match_id, player_id, position, elo_change)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (match_id, player_id)
             DO UPDATE SET position = $4, elo_change = player_match_scores.elo_change + $5",
        )
        .bind(group_id)
        .bind(match_id)
        .bind(player_id)
        .bind(avg_position)
        .bind(total_elo_change)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "UPDATE rounds
         SET completed = true
         WHERE match_id = $1 AND round_number = $2",
    )
    .bind(match_id)
    .bind(round_number)
    .execute(&mut *tx)
    .await?;

    let all_rounds_completed = check_all_rounds_completed(&mut tx, match_id).await?;

    let updated_match = if all_rounds_completed {
        calculate_and_store_team_scores(&mut tx, group_id, match_id).await?;

        let completed_match = sqlx::query_as::<_, models::Match>(
            "UPDATE matches
             SET completed = true
             WHERE id = $1
             RETURNING id, group_id, tournament_id, time, rounds, completed",
        )
        .bind(match_id)
        .fetch_one(&mut *tx)
        .await?;

        completed_match
    } else {
        match_record.clone()
    };

    tx.commit().await?;

    Ok(updated_match)
}

async fn calculate_player_match_aggregates(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    match_id: Uuid,
    current_round_elo_changes: &[elo::EloChange],
) -> Result<Vec<(Uuid, i32, i32)>> {
    let all_race_scores: Vec<models::PlayerRaceScore> = sqlx::query_as(
        "SELECT group_id, match_id, round_number, player_id, position
         FROM player_race_scores
         WHERE match_id = $1
         ORDER BY round_number ASC, position ASC",
    )
    .bind(match_id)
    .fetch_all(&mut **tx)
    .await?;

    let player_positions: HashMap<Uuid, Vec<i32>> =
        all_race_scores
            .iter()
            .fold(HashMap::new(), |mut acc, score| {
                acc.entry(score.player_id)
                    .or_default()
                    .push(score.position);
                acc
            });

    let elo_change_map: HashMap<Uuid, i32> = current_round_elo_changes
        .iter()
        .map(|change| (change.player_id, change.elo_change))
        .collect();

    let aggregates = player_positions
        .into_iter()
        .map(|(player_id, positions)| {
            let avg_position = (positions.iter().sum::<i32>() as f64 / positions.len() as f64)
                .round() as i32;
            let elo_change = elo_change_map.get(&player_id).copied().unwrap_or(0);
            (player_id, avg_position, elo_change)
        })
        .collect();

    Ok(aggregates)
}

async fn check_all_rounds_completed(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, match_id: Uuid) -> Result<bool> {
    let (incomplete_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)
         FROM rounds
         WHERE match_id = $1 AND completed = false",
    )
    .bind(match_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(incomplete_count == 0)
}

async fn calculate_and_store_team_scores(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    group_id: Uuid,
    match_id: Uuid,
) -> Result<()> {
    let race_scores = sqlx::query_as::<_, (Uuid, i32)>(
        "SELECT rp.team_id, prs.position
         FROM player_race_scores prs
         JOIN round_players rp ON rp.match_id = prs.match_id
             AND rp.round_number = prs.round_number
             AND rp.player_id = prs.player_id
         WHERE prs.match_id = $1",
    )
    .bind(match_id)
    .fetch_all(&mut **tx)
    .await?;

    let (num_rounds,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM rounds WHERE match_id = $1",
    )
    .bind(match_id)
    .fetch_one(&mut **tx)
    .await?;

    let team_scores = calculate_team_scores_from_positions(&race_scores, num_rounds as i32);

    for (team_id, score) in team_scores {
        sqlx::query(
            "INSERT INTO team_match_scores (group_id, match_id, team_id, score)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (match_id, team_id)
             DO UPDATE SET score = $4",
        )
        .bind(group_id)
        .bind(match_id)
        .bind(team_id)
        .bind(score)
        .execute(&mut **tx)
        .await?;

        sqlx::query(
            "UPDATE teams
             SET score = $1
             WHERE id = $2",
        )
        .bind(score.round() as i32)
        .bind(team_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

fn calculate_team_scores_from_positions(
    race_scores: &[(Uuid, i32)],
    num_rounds: i32,
) -> HashMap<Uuid, f64> {
    let team_points: HashMap<Uuid, Vec<i32>> =
        race_scores
            .iter()
            .fold(HashMap::new(), |mut acc, &(team_id, position)| {
                let points = scoring::position_to_points(position);
                acc.entry(team_id).or_default().push(points);
                acc
            });

    team_points
        .into_iter()
        .map(|(team_id, points)| {
            let total: i32 = points.iter().sum();
            let average = total as f64 / num_rounds as f64;
            (team_id, average)
        })
        .collect()
}
