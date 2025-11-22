
//! Race Allocation Service
//!
//! This module provides algorithms for distributing players fairly across multiple races.
//! The algorithm ensures:
//! - Each player races an appropriate number of times based on team size
//! - Each race has exactly one player from each team
//! - Players are matched by similar ELO ratings to create balanced races
//! - Position indices are distributed evenly to prevent exhaustion
//!
//! ## Algorithm
//!
//! 1. Sort players within each team by ELO (highest to lowest)
//! 2. Identify primary team (team with most players)
//! 3. Pre-allocate position indices for each team:
//!    - Primary team: simple rotation (0→1→2→0→1→2...)
//!    - Other teams with equal position uses: map to primary team's pattern proportionally
//!      (e.g., primary position 0→team position 0, primary positions 1,2→team position 1)
//!      This ensures high-ELO players race together and low-ELO players race together
//!    - Other teams: use weighted allocation considering ELO distance and usage balance
//! 4. For each race:
//!    - Use pre-determined position index from each team's schedule
//!    - Select the player at that position index
//!
//! This approach prevents position index exhaustion while maintaining good ELO balance,
//! especially in the final races of a match.

use crate::error::{AppError, Result};
use crate::models;
use crate::services::team_allocation::Team;
use uuid::Uuid;

/// Represents the allocation of players to a specific race
#[derive(Debug, Clone)]
pub struct RaceAllocation {
    pub race_number: i32,
    pub player_ids: Vec<Uuid>,
}

/// Player data with ELO for sorting and matching
#[derive(Debug, Clone)]
struct PlayerWithElo {
    id: Uuid,
    elo: i32,
}


/// Allocates players to races using ELO-based matching with positional rotation.
///
/// This function distributes players across multiple races ensuring:
/// - Each race has one player from each team
/// - Players from smaller teams race more frequently (to balance total races)
/// - Players are matched by closest ELO rating across teams
/// - Primary team players are selected in rotating positions (1→2→3→1→2→3...)
///
/// # Arguments
///
/// * `players` - Slice of all players participating in the match
/// * `teams` - Slice of teams with their assigned players
/// * `num_races` - Total number of races in the match
/// * `_players_per_race` - Maximum players per race (currently unused, kept for compatibility)
///
/// # Returns
///
/// Result containing a vector of race allocations, one per race
///
/// # Errors
///
/// Returns an error if a team has no players
///
/// # Examples
///
/// ```ignore
/// # use mario_kart_leaderboard_backend::services::race_allocation::allocate_races;
/// # use mario_kart_leaderboard_backend::services::team_allocation::Team;
/// # use mario_kart_leaderboard_backend::models::Player;
/// # use uuid::Uuid;
/// let group_id = Uuid::new_v4();
/// let players = vec![
///     Player { id: Uuid::new_v4(), group_id, name: "Alice".to_string(), elo_rating: 1400 },
///     Player { id: Uuid::new_v4(), group_id, name: "Bob".to_string(), elo_rating: 1200 },
/// ];
///
/// let teams = vec![
///     Team { team_num: 1, players: vec![players[0].clone()], total_elo: 1400 },
///     Team { team_num: 2, players: vec![players[1].clone()], total_elo: 1200 },
/// ];
///
/// let allocations = allocate_races(&players, &teams, 4, 2).unwrap();
/// assert_eq!(allocations.len(), 4);
/// // Each race should have 2 players (one from each team)
/// ```
pub fn allocate_races(
    _players: &[models::Player],
    teams: &[Team],
    num_races: i32,
    _players_per_race: i32,
) -> Result<Vec<RaceAllocation>> {
    // Build team state with sorted players
    let team_state: Vec<(i32, Vec<PlayerWithElo>)> = teams
        .iter()
        .map(|team| {
            let mut sorted_players: Vec<PlayerWithElo> = team
                .players
                .iter()
                .map(|p| PlayerWithElo {
                    id: p.id,
                    elo: p.elo_rating,
                })
                .collect();

            sorted_players.sort_by(|a, b| b.elo.cmp(&a.elo));

            (team.team_num, sorted_players)
        })
        .collect();

    let primary_team_index = team_state
        .iter()
        .enumerate()
        .max_by_key(|(_, (_, players))| players.len())
        .map(|(idx, _)| idx)
        .ok_or_else(|| AppError::Internal("No teams available".to_string()))?;

    let primary_team_size = team_state[primary_team_index].1.len();

    // Pre-allocate position indices for each team
    let team_position_schedules: Vec<Vec<usize>> = team_state
        .iter()
        .enumerate()
        .map(|(team_idx, (_, team_players))| {
            let team_size = team_players.len();
            let races_for_team = num_races as usize;

            if team_idx == primary_team_index {
                // Primary team: simple rotation (0, 1, 2, 0, 1, 2, ...)
                (0..races_for_team).map(|i| i % team_size).collect()
            } else {
                // Non-primary teams: map positions to align with primary team pattern
                let uses_per_position = races_for_team / team_size;
                let extra_uses = races_for_team % team_size;

                // Check if all positions have equal target uses
                let all_equal = extra_uses == 0;

                if all_equal {
                    // Map primary positions to this team's positions while maintaining even distribution
                    let target_uses_per_position = uses_per_position;
                    let mut used_counts = vec![0; team_size];
                    let mut schedule = Vec::new();

                    for race_idx in 0..races_for_team {
                        let primary_pos = race_idx % primary_team_size;
                        // Ideal position based on primary team mapping
                        let ideal_pos = (primary_pos * team_size) / primary_team_size;

                        // Choose position: prefer ideal if available, otherwise choose least-used
                        let chosen_pos = if used_counts[ideal_pos] < target_uses_per_position {
                            ideal_pos
                        } else {
                            (0..team_size)
                                .filter(|&p| used_counts[p] < target_uses_per_position)
                                .min_by_key(|&p| used_counts[p])
                                .unwrap_or(0)
                        };

                        schedule.push(chosen_pos);
                        used_counts[chosen_pos] += 1;
                    }

                    schedule
                } else {
                    // If not equal, use weighted allocation
                    let target_uses: Vec<usize> = (0..team_size)
                        .map(|i| uses_per_position + if i < extra_uses { 1 } else { 0 })
                        .collect();

                    let mut used_counts = vec![0; team_size];
                    let mut schedule = Vec::new();

                    for race_idx in 0..races_for_team {
                        let primary_position_idx = race_idx % primary_team_size;
                        let primary_elo = team_state[primary_team_index].1[primary_position_idx].elo;

                        // Find best position considering both ELO and usage
                        let best_position = (0..team_size)
                            .filter(|&pos_idx| used_counts[pos_idx] < target_uses[pos_idx])
                            .min_by_key(|&pos_idx| {
                                let candidate_elo = team_players[pos_idx].elo;
                                let elo_diff = (candidate_elo - primary_elo).abs();

                                // Strongly prefer less-used positions
                                let usage_penalty = (used_counts[pos_idx] as i32) * 5000;

                                elo_diff + usage_penalty
                            })
                            .unwrap_or(0);

                        schedule.push(best_position);
                        used_counts[best_position] += 1;
                    }

                    schedule
                }
            }
        })
        .collect();

    // Build allocations using pre-determined position schedules
    let mut allocations = Vec::new();

    for race_num in 0..num_races {
        let race_idx = race_num as usize;
        let mut race_players = Vec::new();

        for (team_idx, (_, team_players)) in team_state.iter().enumerate() {
            let position_idx = team_position_schedules[team_idx][race_idx];
            let player = &team_players[position_idx];
            race_players.push(player.id);
        }

        allocations.push(RaceAllocation {
            race_number: race_num + 1,
            player_ids: race_players,
        });
    }

    Ok(allocations)
}
