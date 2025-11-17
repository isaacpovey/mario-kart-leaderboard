//! Result Recording Service
//!
//! This module provides orchestration for recording race results, updating ELO ratings,
//! and calculating aggregate scores. The service handles:
//! - Result validation (positions, player participation)
//! - ELO calculation and updates
//! - Player race score recording
//! - Player match aggregate updates
//! - Round completion tracking
//! - Team score calculation when match completes
//!
//! ## Result Recording Workflow
//!
//! 1. Validate result inputs (positions, duplicates)
//! 2. Fetch players and round participants from database
//! 3. Validate that submitted players match round participants
//! 4. Calculate ELO changes using the ELO service
//! 5. Persist results in a transaction:
//!    - Insert player race scores
//!    - Update player ELO ratings
//!    - Update/insert player match aggregates
//!    - Mark round as completed
//!    - If all rounds complete: calculate and store team scores, mark match complete

use crate::error::{AppError, Result};
use crate::models;
use crate::services::elo::{self, PlayerResult};
use crate::services::score_calculation;
use crate::services::teammate_elo;
use std::collections::HashMap;
use uuid::Uuid;

/// Validates race result inputs.
///
/// Ensures that:
/// - At least one result is provided
/// - All positions are between 1 and 24
/// - No duplicate positions exist
///
/// # Arguments
///
/// * `results` - Slice of player result inputs with position data
///
/// # Returns
///
/// Result indicating success or failure with descriptive error messages
///
/// # Errors
///
/// Returns an error if any validation rule fails
pub fn validate_results<T>(results: &[T]) -> Result<()>
where
    T: AsRef<PlayerResultData>,
{
    if results.is_empty() {
        return Err(AppError::InvalidInput(
            "At least one player result is required".to_string(),
        ));
    }

    let positions: Vec<i32> = results.iter().map(|r| r.as_ref().position).collect();

    if positions.iter().any(|&p| !(1..=24).contains(&p)) {
        return Err(AppError::InvalidInput(
            "Positions must be between 1 and 24".to_string(),
        ));
    }

    let unique_positions: std::collections::HashSet<i32> = positions.iter().copied().collect();
    if unique_positions.len() != positions.len() {
        return Err(AppError::InvalidInput(
            "Duplicate positions are not allowed".to_string(),
        ));
    }

    Ok(())
}

/// Trait for accessing player result data (to support multiple input types)
pub trait AsRef<T> {
    fn as_ref(&self) -> &T;
}

/// Common data structure for player results
pub struct PlayerResultData {
    pub player_id: String,
    pub position: i32,
}

/// Fetches player IDs assigned to a specific round.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `match_id` - UUID of the match
/// * `round_number` - Round number (1-indexed)
///
/// # Returns
///
/// Result containing a vector of player UUIDs
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_round_players(
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

/// Validates that submitted player results match round participants exactly.
///
/// # Arguments
///
/// * `submitted_players` - Slice of player UUIDs from submitted results
/// * `round_players` - Slice of player UUIDs assigned to the round
///
/// # Returns
///
/// Result indicating success or failure
///
/// # Errors
///
/// Returns an error if the sets don't match exactly
pub fn validate_players_in_round(submitted_players: &[Uuid], round_players: &[Uuid]) -> Result<()> {
    let submitted_set: std::collections::HashSet<Uuid> =
        submitted_players.iter().copied().collect();
    let round_set: std::collections::HashSet<Uuid> = round_players.iter().copied().collect();

    if submitted_set != round_set {
        return Err(AppError::InvalidInput(
            "Results must include all players in this round, no more and no less".to_string(),
        ));
    }

    Ok(())
}

/// Creates a mapping of player IDs to their current ELO ratings.
///
/// # Arguments
///
/// * `players` - Slice of player models
///
/// # Returns
///
/// HashMap mapping player UUIDs to ELO ratings
pub fn create_player_elo_map(players: &[models::Player]) -> HashMap<Uuid, i32> {
    players.iter().map(|p| (p.id, p.elo_rating)).collect()
}

/// Converts GraphQL input results to ELO PlayerResult structs.
///
/// # Arguments
///
/// * `results` - Slice of result inputs with player IDs and positions
/// * `player_elos` - HashMap of current ELO ratings by player ID
///
/// # Returns
///
/// Result containing a vector of PlayerResult structs for ELO calculation
///
/// # Errors
///
/// Returns an error if:
/// - Player ID is invalid UUID format
/// - Player not found in ELO map
pub fn create_player_results(
    results: &[(Uuid, i32)],
    player_elos: &HashMap<Uuid, i32>,
) -> Result<Vec<PlayerResult>> {
    results
        .iter()
        .map(|(player_id, position)| {
            let current_elo = player_elos
                .get(player_id)
                .ok_or_else(|| AppError::NotFound("Player not found".to_string()))?;

            Ok(PlayerResult {
                player_id: *player_id,
                position: *position,
                current_elo: *current_elo,
            })
        })
        .collect()
}

/// High-level orchestration function for recording race results with dual ELO tracking.
///
/// This is the main entry point that:
/// 1. Fetches players from database (for all-time ELO)
/// 2. Gets or creates tournament ELO records (lazy initialization at 1200)
/// 3. Calculates all-time ELO changes using current all-time ELO
/// 4. Calculates tournament ELO changes using current tournament ELO (independent)
/// 5. Calls record_results_in_transaction to persist everything
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `group_id` - UUID of the group
/// * `match_id` - UUID of the match
/// * `round_number` - Round number (1-indexed)
/// * `results` - Slice of tuples containing (player_id, position)
/// * `match_record` - Current match record
///
/// # Returns
///
/// Result containing the updated match record
///
/// # Errors
///
/// Returns an error if any operation fails
pub async fn record_race_results(
    pool: &sqlx::PgPool,
    group_id: Uuid,
    match_id: Uuid,
    round_number: i32,
    results: &[(Uuid, i32)],
    match_record: &models::Match,
) -> Result<models::Match> {
    let player_ids: Vec<Uuid> = results.iter().map(|(id, _)| *id).collect();

    let players = models::Player::find_by_ids(pool, &player_ids).await?;
    let all_time_player_elos = create_player_elo_map(&players);

    let mut tx = pool.begin().await.map_err(|e| {
        AppError::Internal(format!("Failed to start transaction for ELO fetch: {e}"))
    })?;

    let tournament_player_elos = models::PlayerTournamentScore::get_or_create_batch(
        &mut tx,
        &player_ids,
        match_record.tournament_id,
        group_id,
    )
    .await?;

    tx.commit().await.map_err(|e| {
        AppError::Internal(format!("Failed to commit tournament ELO fetch transaction: {e}"))
    })?;

    let all_time_player_results = create_player_results(results, &all_time_player_elos)?;
    let all_time_elo_changes = elo::calculate_elo_changes(&all_time_player_results);

    let tournament_player_results = create_player_results(results, &tournament_player_elos)?;
    let tournament_elo_changes = elo::calculate_elo_changes(&tournament_player_results);

    record_results_in_transaction(
        pool,
        group_id,
        match_id,
        round_number,
        results,
        &all_time_elo_changes,
        &tournament_elo_changes,
        match_record,
    )
    .await
}

/// Records race results and updates all related data in a single transaction.
///
/// This is the main orchestration function that:
/// 1. Inserts player race scores
/// 2. Updates player ELO ratings
/// 3. Updates player match aggregates (avg position, total ELO change)
/// 4. Marks round as completed
/// 5. If all rounds complete:
///    - Calculates and stores team scores
///    - Marks match as completed
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `group_id` - UUID of the group
/// * `match_id` - UUID of the match
/// * `round_number` - Round number (1-indexed)
/// * `results` - Slice of tuples containing (player_id, position)
/// * `elo_changes` - Slice of ELO changes calculated by ELO service
/// * `match_record` - Current match record
///
/// # Returns
///
/// Result containing the updated match record (with completed=true if match finished)
///
/// # Errors
///
/// Returns an error if any database operation fails (transaction will be rolled back)
pub async fn record_results_in_transaction(
    pool: &sqlx::PgPool,
    group_id: Uuid,
    match_id: Uuid,
    round_number: i32,
    results: &[(Uuid, i32)],
    all_time_elo_changes: &[elo::EloChange],
    tournament_elo_changes: &[elo::EloChange],
    match_record: &models::Match,
) -> Result<models::Match> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to start database transaction: {e}")))?;

    let all_time_elo_map: HashMap<Uuid, &elo::EloChange> =
        all_time_elo_changes.iter().map(|c| (c.player_id, c)).collect();
    let tournament_elo_map: HashMap<Uuid, &elo::EloChange> =
        tournament_elo_changes.iter().map(|c| (c.player_id, c)).collect();

    for (player_id, position) in results {
        let all_time_change = all_time_elo_map
            .get(player_id)
            .ok_or_else(|| AppError::Internal("Missing all-time ELO change".to_string()))?;
        let tournament_change = tournament_elo_map
            .get(player_id)
            .ok_or_else(|| AppError::Internal("Missing tournament ELO change".to_string()))?;

        sqlx::query(
            "INSERT INTO player_race_scores (
                group_id, match_id, round_number, player_id, position,
                all_time_elo_change, all_time_elo_after,
                tournament_elo_change, tournament_elo_after
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(group_id)
        .bind(match_id)
        .bind(round_number)
        .bind(player_id)
        .bind(position)
        .bind(all_time_change.elo_change)
        .bind(all_time_change.new_elo)
        .bind(tournament_change.elo_change)
        .bind(tournament_change.new_elo)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to insert player race score: {e}")))?;
    }

    let player_teams: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT tp.player_id, tp.team_id
         FROM team_players tp
         INNER JOIN teams t ON tp.team_id = t.id
         WHERE t.match_id = $1",
    )
    .bind(match_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to fetch player teams: {e}")))?;

    let player_to_team: HashMap<Uuid, Uuid> = player_teams.into_iter().collect();

    let team_to_players: HashMap<Uuid, Vec<Uuid>> =
        player_to_team
            .iter()
            .fold(HashMap::new(), |mut acc, (player_id, team_id)| {
                acc.entry(*team_id).or_default().push(*player_id);
                acc
            });

    let tournament_elo_change_map: HashMap<Uuid, i32> = tournament_elo_map
        .iter()
        .map(|(player_id, change)| (*player_id, change.elo_change))
        .collect();

    let (teammate_contributions, tournament_elo_adjustments) =
        teammate_elo::calculate_teammate_contributions(
            results,
            &player_to_team,
            &team_to_players,
            &tournament_elo_change_map,
        );

    let contributions: Vec<(Uuid, i32, Uuid, Uuid, i32, i32)> = teammate_contributions
        .iter()
        .map(|c| {
            (
                match_id,
                round_number,
                c.source_player_id,
                c.beneficiary_player_id,
                c.source_tournament_elo_change,
                c.contribution_amount,
            )
        })
        .collect();

    let already_updated_players: std::collections::HashSet<Uuid> =
        tournament_elo_adjustments.keys().copied().collect();

    if !contributions.is_empty() {
        models::PlayerTeammateEloContribution::insert_contributions_batch(&mut tx, &contributions)
            .await?;

        let beneficiary_ids: Vec<Uuid> = tournament_elo_adjustments.keys().copied().collect();

        let beneficiary_current_elos = models::PlayerTournamentScore::get_or_create_batch(
            &mut tx,
            &beneficiary_ids,
            match_record.tournament_id,
            group_id,
        )
        .await?;

        let tournament_elo_updates_with_contributions: Vec<(Uuid, Uuid, i32)> =
            tournament_elo_adjustments
                .into_iter()
                .map(|(player_id, adjustment)| {
                    let base_elo = tournament_elo_map
                        .get(&player_id)
                        .map(|change| change.new_elo)
                        .or_else(|| beneficiary_current_elos.get(&player_id).copied())
                        .unwrap_or(1200);

                    (player_id, match_record.tournament_id, base_elo + adjustment)
                })
                .collect();

        if !tournament_elo_updates_with_contributions.is_empty() {
            models::PlayerTournamentScore::update_elo_batch(
                &mut tx,
                &tournament_elo_updates_with_contributions,
            )
            .await?;
        }
    }

    for change in all_time_elo_changes {
        sqlx::query(
            "UPDATE players
             SET elo_rating = $1
             WHERE id = $2",
        )
        .bind(change.new_elo)
        .bind(change.player_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update player ELO rating: {e}")))?;
    }

    let tournament_elo_updates: Vec<(Uuid, Uuid, i32)> = tournament_elo_changes
        .iter()
        .filter(|c| !already_updated_players.contains(&c.player_id))
        .map(|c| (c.player_id, match_record.tournament_id, c.new_elo))
        .collect();

    if !tournament_elo_updates.is_empty() {
        models::PlayerTournamentScore::update_elo_batch(&mut tx, &tournament_elo_updates).await?;
    }

    let player_match_updates = score_calculation::calculate_player_match_aggregates(
        &mut tx,
        match_id,
        round_number,
        all_time_elo_changes,
        tournament_elo_changes,
    )
    .await?;

    for (
        player_id,
        avg_position,
        all_time_total_change,
        tournament_change_from_races,
        tournament_change_from_contributions,
    ) in player_match_updates
    {
        let tournament_total_change =
            tournament_change_from_races + tournament_change_from_contributions;

        sqlx::query(
            "UPDATE player_match_scores
             SET position = $3,
                 elo_change = elo_change + $4,
                 tournament_elo_from_races = tournament_elo_from_races + $5,
                 tournament_elo_from_contributions = tournament_elo_from_contributions + $6,
                 tournament_elo_change = tournament_elo_change + $7
             WHERE match_id = $1 AND player_id = $2",
        )
        .bind(match_id)
        .bind(player_id)
        .bind(avg_position)
        .bind(all_time_total_change)
        .bind(tournament_change_from_races)
        .bind(tournament_change_from_contributions)
        .bind(tournament_total_change)
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

    let all_rounds_completed =
        score_calculation::check_all_rounds_completed(&mut tx, match_id).await?;

    let updated_match = if all_rounds_completed {
        score_calculation::calculate_and_store_team_scores(&mut tx, group_id, match_id).await?;

        sqlx::query_as::<_, models::Match>(
            "UPDATE matches
             SET completed = true
             WHERE id = $1
             RETURNING id, group_id, tournament_id, time, rounds, completed",
        )
        .bind(match_id)
        .fetch_one(&mut *tx)
        .await?
    } else {
        match_record.clone()
    };

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to commit transaction: {e}")))?;

    Ok(updated_match)
}
