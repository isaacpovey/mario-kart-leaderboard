
//! Match Service
//!
//! This module provides high-level match creation orchestration, coordinating
//! team allocation, track selection, race distribution, and database persistence.
//!
//! ## Match Creation Workflow
//!
//! 1. Validate inputs (player count, race configuration)
//! 2. Fetch players from database
//! 3. Allocate players to balanced teams (using team_allocation service)
//! 4. Select tracks avoiding recent usage (using track_selection service)
//! 5. Distribute players across races (using race_allocation service)
//! 6. Persist all data in a single transaction:
//!    - Create match record
//!    - Create teams
//!    - Associate players with teams
//!    - Create rounds with tracks
//!    - Create round player assignments

use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models;
use crate::services::{race_allocation, team_allocation, track_selection};
use crate::services::notification_manager::{NotificationManager, RaceResultNotification};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Type alias for round player record tuple: (group_id, match_id, round_number, player_id, team_id, player_position)
type RoundPlayerRecord = (Uuid, Uuid, i32, Uuid, Uuid, i32);

/// Validates match creation inputs.
///
/// Ensures that:
/// - At least one player is provided
/// - Number of races is positive
/// - Players per race is positive
/// - Players per race doesn't exceed total players
/// - Total slots (races × players_per_race) >= number of players
///
/// # Arguments
///
/// * `player_uuids` - Slice of player UUIDs
/// * `num_races` - Number of races to create
/// * `players_per_race` - Maximum players per race
///
/// # Returns
///
/// Result indicating success or failure with descriptive error messages
///
/// # Errors
///
/// Returns an error if any validation rule fails
pub fn validate_create_match_inputs(
    player_uuids: &[Uuid],
    num_races: i32,
    players_per_race: i32,
) -> Result<()> {
    let num_players = player_uuids.len() as i32;

    if player_uuids.is_empty() {
        return Err(AppError::InvalidInput(
            "At least one player is required".to_string(),
        ));
    }

    if num_races <= 0 {
        return Err(AppError::InvalidInput(
            "Number of races must be positive".to_string(),
        ));
    }

    if players_per_race <= 0 {
        return Err(AppError::InvalidInput(
            "Players per race must be positive".to_string(),
        ));
    }

    if players_per_race > num_players {
        return Err(AppError::InvalidInput(
            "Players per race cannot exceed total number of players".to_string(),
        ));
    }

    let total_slots = num_races * players_per_race;
    if total_slots < num_players {
        return Err(AppError::InvalidInput(format!(
            "Invalid configuration: {num_races} races with {players_per_race} players per race gives {total_slots} total slots, which is less than {num_players} players. Each player must be able to participate in at least one race."
        )));
    }

    Ok(())
}

/// Creates a complete match with teams, tracks, and race allocations.
///
/// This is the main orchestration function that coordinates all match creation steps:
/// 1. Validates inputs
/// 2. Fetches players from database
/// 3. Allocates teams using balanced ELO distribution
/// 4. Selects tracks avoiding recent tournament usage
/// 5. Allocates players to races fairly
/// 6. Persists everything in a single transaction
/// 7. Emits notification for real-time updates
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `group_id` - UUID of the group
/// * `tournament_id` - UUID of the tournament
/// * `player_ids` - Slice of player UUIDs participating
/// * `num_races` - Number of races in the match
/// * `players_per_race` - Maximum players per race
/// * `notification_manager` - NotificationManager for emitting match creation events
///
/// # Returns
///
/// Result containing the created match record
///
/// # Errors
///
/// Returns an error if:
/// - Input validation fails
/// - Database operations fail
/// - Team/race allocation algorithms fail
/// - Track selection fails
pub async fn create_match_with_rounds(
    pool: &DbPool,
    group_id: Uuid,
    tournament_id: Uuid,
    player_ids: &[Uuid],
    num_races: i32,
    players_per_race: i32,
    notification_manager: &NotificationManager,
) -> Result<models::Match> {
    validate_create_match_inputs(player_ids, num_races, players_per_race)?;

    let players = models::Player::find_by_ids(pool, player_ids).await?;

    if players.len() != player_ids.len() {
        return Err(AppError::NotFound(
            "One or more players not found".to_string(),
        ));
    }

    let teams = team_allocation::allocate_teams(&players, &players_per_race);
    let tracks = track_selection::select_tracks(pool, tournament_id, num_races).await?;
    let race_allocations =
        race_allocation::allocate_races(&players, &teams, num_races, players_per_race)?;

    let match_record = create_match_in_transaction(
        pool,
        group_id,
        tournament_id,
        num_races,
        &teams,
        &tracks,
        &race_allocations,
    )
    .await?;

    // Emit notification for match creation (round_number = 0 indicates match creation)
    tracing::info!(
        "Match created: match_id={}, tournament_id={}, emitting notification",
        match_record.id,
        tournament_id
    );

    let notification = RaceResultNotification {
        match_id: match_record.id,
        tournament_id,
        round_number: 0, // 0 indicates match creation, not a race result
        group_id,
    };

    notification_manager.notify(notification);

    Ok(match_record)
}

/// Internal function: Persists match data in a single database transaction.
///
/// Creates all necessary database records for a match:
/// - Match record
/// - Teams
/// - Team players (batch insert with UNNEST)
/// - Rounds with tracks
/// - Round players (batch insert with UNNEST)
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `group_id` - UUID of the group
/// * `tournament_id` - UUID of the tournament
/// * `num_races` - Number of races
/// * `teams` - Slice of allocated teams
/// * `tracks` - Slice of selected tracks
/// * `race_allocations` - Slice of race allocations
///
/// # Returns
///
/// Result containing the created match record
///
/// # Errors
///
/// Returns an error if any database operation fails (transaction will be rolled back)
async fn create_match_in_transaction(
    pool: &DbPool,
    group_id: Uuid,
    tournament_id: Uuid,
    num_races: i32,
    teams: &[team_allocation::Team],
    tracks: &[models::Track],
    race_allocations: &[race_allocation::RaceAllocation],
) -> Result<models::Match> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to start database transaction: {e}")))?;

    let match_record = sqlx::query_as::<_, models::Match>(
        "INSERT INTO matches (group_id, tournament_id, time, rounds, completed)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, group_id, tournament_id, time, rounds, completed",
    )
    .bind(group_id)
    .bind(tournament_id)
    .bind(Utc::now())
    .bind(num_races)
    .bind(false)
    .fetch_one(tx.as_mut())
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create match record: {e}")))?;

    let team_id_pairs: Vec<(i32, Uuid)> = {
        let mut pairs = Vec::with_capacity(teams.len());
        for team in teams {
            let (team_id,): (Uuid,) = sqlx::query_as(
                "INSERT INTO teams (group_id, match_id, team_num)
                 VALUES ($1, $2, $3)
                 RETURNING id",
            )
            .bind(group_id)
            .bind(match_record.id)
            .bind(team.team_num)
            .fetch_one(tx.as_mut())
            .await?;

            pairs.push((team.team_num, team_id));
        }
        pairs
    };
    let team_id_map: HashMap<i32, Uuid> = team_id_pairs.into_iter().collect();

    let team_player_records: Vec<(Uuid, Uuid, Uuid, i32)> = teams
        .iter()
        .filter_map(|team| {
            team_id_map.get(&team.team_num).map(|&team_id| {
                team.players
                    .iter()
                    .enumerate()
                    .map(|(rank, player)| (group_id, team_id, player.id, (rank + 1) as i32))
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .collect();

    if !team_player_records.is_empty() {
        let (group_ids, team_ids, player_ids, ranks): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
            team_player_records.into_iter().fold(
                (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |(mut gs, mut ts, mut ps, mut rs), (g, t, p, r)| {
                    gs.push(g);
                    ts.push(t);
                    ps.push(p);
                    rs.push(r);
                    (gs, ts, ps, rs)
                },
            );

        sqlx::query(
            "INSERT INTO team_players (group_id, team_id, player_id, rank)
             SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[], $4::int[])",
        )
        .bind(&group_ids)
        .bind(&team_ids)
        .bind(&player_ids)
        .bind(&ranks)
        .execute(tx.as_mut())
        .await?;
    }

    let all_player_ids: Vec<Uuid> = teams
        .iter()
        .flat_map(|team| team.players.iter().map(|p| p.id))
        .collect();

    if !all_player_ids.is_empty() {
        let group_ids_for_scores: Vec<Uuid> = vec![group_id; all_player_ids.len()];
        let match_ids_for_scores: Vec<Uuid> = vec![match_record.id; all_player_ids.len()];
        let zeros: Vec<i32> = vec![0; all_player_ids.len()];

        sqlx::query(
            "INSERT INTO player_match_scores (
                group_id, match_id, player_id, position,
                elo_change, tournament_elo_change,
                tournament_elo_from_races, tournament_elo_from_contributions
             )
             SELECT * FROM UNNEST(
                $1::uuid[], $2::uuid[], $3::uuid[], $4::int[],
                $5::int[], $6::int[], $7::int[], $8::int[]
             )",
        )
        .bind(&group_ids_for_scores)
        .bind(&match_ids_for_scores)
        .bind(&all_player_ids)
        .bind(&zeros)
        .bind(&zeros)
        .bind(&zeros)
        .bind(&zeros)
        .bind(&zeros)
        .execute(tx.as_mut())
        .await?;
    }

    let player_team_map: HashMap<Uuid, Uuid> = teams
        .iter()
        .flat_map(|team| {
            team_id_map
                .get(&team.team_num)
                .into_iter()
                .flat_map(move |&team_id| team.players.iter().map(move |p| (p.id, team_id)))
        })
        .collect();

    let team_to_team_num: HashMap<Uuid, i32> = teams
        .iter()
        .filter_map(|team| team_id_map.get(&team.team_num).map(|&team_id| (team_id, team.team_num)))
        .collect();

    for (idx, track) in tracks.iter().enumerate() {
        sqlx::query(
            "INSERT INTO rounds (match_id, round_number, track_id)
             VALUES ($1, $2, $3)",
        )
        .bind(match_record.id)
        .bind((idx + 1) as i32)
        .bind(track.id)
        .execute(tx.as_mut())
        .await?;
    }

    let round_player_records: Result<Vec<RoundPlayerRecord>> = race_allocations
        .iter()
        .flat_map(|allocation| {
            allocation
                .player_ids
                .iter()
                .map(|player_id| {
                    player_team_map
                        .get(player_id)
                        .ok_or_else(|| {
                            AppError::Internal("Player team mapping not found".to_string())
                        })
                        .and_then(|&team_id| {
                            team_to_team_num
                                .get(&team_id)
                                .ok_or_else(|| {
                                    AppError::Internal("Team number not found".to_string())
                                })
                                .map(|&team_num| {
                                    (
                                        group_id,
                                        match_record.id,
                                        allocation.race_number,
                                        *player_id,
                                        team_id,
                                        team_num,
                                    )
                                })
                        })
                })
        })
        .collect();
    let round_player_records = round_player_records?;

    if !round_player_records.is_empty() {
        let (group_ids, match_ids, round_numbers, player_ids, team_ids, positions): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = round_player_records.into_iter().fold(
            (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            |(mut gs, mut ms, mut rns, mut ps, mut ts, mut pos), (g, m, rn, p, t, po)| {
                gs.push(g);
                ms.push(m);
                rns.push(rn);
                ps.push(p);
                ts.push(t);
                pos.push(po);
                (gs, ms, rns, ps, ts, pos)
            },
        );

        sqlx::query(
            "INSERT INTO round_players (group_id, match_id, round_number, player_id, team_id, player_position)
             SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::int[], $4::uuid[], $5::uuid[], $6::int[])",
        )
        .bind(&group_ids)
        .bind(&match_ids)
        .bind(&round_numbers)
        .bind(&player_ids)
        .bind(&team_ids)
        .bind(&positions)
        .execute(tx.as_mut())
        .await?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to commit transaction: {e}")))?;

    Ok(match_record)
}
