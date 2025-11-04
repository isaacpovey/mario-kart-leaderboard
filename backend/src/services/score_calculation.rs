//! Score Calculation Service
//!
//! This module provides functions for calculating aggregate scores for players and teams.
//! The service handles:
//! - Player match aggregates (average position, ELO changes)
//! - Team score calculations from race positions
//! - Match completion status checking
//! - Database updates for team scores
//!
//! ## Score Calculation
//!
//! - Player scores: Average position across all races in a match
//! - Team scores: Average points across all rounds (using position-to-points conversion)
//! - ELO changes: Aggregated from individual race results

use crate::error::Result;
use crate::models;
use crate::services::elo;
use crate::services::scoring;
use std::collections::HashMap;
use uuid::Uuid;

/// Calculates aggregate statistics for all players in a match.
///
/// This function computes:
/// - Average position for each player across all races
/// - Total all-time ELO change from the current round
/// - Total tournament ELO change from the current round
///
/// # Arguments
///
/// * `tx` - Active database transaction
/// * `match_id` - UUID of the match
/// * `current_round_all_time_elo_changes` - All-time ELO changes from the current round
/// * `current_round_tournament_elo_changes` - Tournament ELO changes from the current round
///
/// # Returns
///
/// Result containing a vector of tuples: (player_id, avg_position, all_time_elo_change, tournament_elo_change)
///
/// # Errors
///
/// Returns an error if database queries fail
pub async fn calculate_player_match_aggregates(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    match_id: Uuid,
    current_round_all_time_elo_changes: &[elo::EloChange],
    current_round_tournament_elo_changes: &[elo::EloChange],
) -> Result<Vec<(Uuid, i32, i32, i32)>> {
    let all_race_scores: Vec<models::PlayerRaceScore> = sqlx::query_as(
        "SELECT group_id, match_id, round_number, player_id, position,
                all_time_elo_change, all_time_elo_after,
                tournament_elo_change, tournament_elo_after, created_at
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
                acc.entry(score.player_id).or_default().push(score.position);
                acc
            });

    let all_time_elo_change_map: HashMap<Uuid, i32> = current_round_all_time_elo_changes
        .iter()
        .map(|change| (change.player_id, change.elo_change))
        .collect();

    let tournament_elo_change_map: HashMap<Uuid, i32> = current_round_tournament_elo_changes
        .iter()
        .map(|change| (change.player_id, change.elo_change))
        .collect();

    let all_player_ids: Vec<Uuid> = player_positions.keys().copied().collect();
    let teammate_contributions =
        models::PlayerTeammateEloContribution::get_match_total_for_players(
            tx,
            match_id,
            &all_player_ids,
        )
        .await?;

    let aggregates = player_positions
        .into_iter()
        .map(|(player_id, positions)| {
            let avg_position =
                (positions.iter().sum::<i32>() as f64 / positions.len() as f64).round() as i32;
            let all_time_elo_change = all_time_elo_change_map.get(&player_id).copied().unwrap_or(0);
            let tournament_elo_change_from_races =
                tournament_elo_change_map.get(&player_id).copied().unwrap_or(0);
            let teammate_contribution = teammate_contributions.get(&player_id).copied().unwrap_or(0);
            let tournament_elo_change = tournament_elo_change_from_races + teammate_contribution;
            (player_id, avg_position, all_time_elo_change, tournament_elo_change)
        })
        .collect();

    Ok(aggregates)
}

/// Checks if all rounds in a match have been completed.
///
/// # Arguments
///
/// * `tx` - Active database transaction
/// * `match_id` - UUID of the match to check
///
/// # Returns
///
/// Result containing true if all rounds are completed, false otherwise
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn check_all_rounds_completed(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    match_id: Uuid,
) -> Result<bool> {
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

/// Calculates and stores team scores for a match.
///
/// This function:
/// 1. Fetches all race scores for the match
/// 2. Calculates average team scores using position-to-points conversion
/// 3. Stores scores in team_match_scores table
/// 4. Updates the teams table with the final score
///
/// # Arguments
///
/// * `tx` - Active database transaction
/// * `group_id` - UUID of the group
/// * `match_id` - UUID of the match
///
/// # Returns
///
/// Result indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// - Database queries fail
/// - Score calculation fails
pub async fn calculate_and_store_team_scores(
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

    let (num_rounds,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rounds WHERE match_id = $1")
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

/// Calculates team scores from race positions.
///
/// Converts positions to points using the scoring module, then calculates
/// the average score per team across all rounds.
///
/// # Arguments
///
/// * `race_scores` - Slice of tuples containing (team_id, position)
/// * `num_rounds` - Total number of rounds in the match
///
/// # Returns
///
/// HashMap mapping team IDs to their average scores
///
/// # Examples
///
/// ```
/// # use mario_kart_leaderboard_backend::services::score_calculation::calculate_team_scores_from_positions;
/// # use uuid::Uuid;
/// # use std::collections::HashMap;
/// let team_id = Uuid::new_v4();
/// let race_scores = vec![(team_id, 1), (team_id, 2)];
/// let scores = calculate_team_scores_from_positions(&race_scores, 2);
/// // Team finished 1st (15 pts) and 2nd (12 pts) = average of 13.5 points
/// ```
pub fn calculate_team_scores_from_positions(
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
