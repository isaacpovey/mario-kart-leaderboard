//! Race Allocation Service
//!
//! This module provides algorithms for distributing players fairly across multiple races.
//! The algorithm ensures:
//! - Each player races an appropriate number of times based on team size
//! - Players from smaller teams race more frequently to balance participation
//! - Players avoid racing in consecutive rounds when possible
//! - Each race has exactly one player from each team
//!
//! ## Algorithm
//!
//! 1. Calculate base races per player (total slots / num players)
//! 2. Adjust per team: smaller teams race more to compensate
//! 3. For each race:
//!    - Select one player from each team
//!    - Prefer players with remaining race quota
//!    - Avoid players from previous race when possible
//!    - Decrement selected player's remaining races

use crate::error::{AppError, Result};
use crate::models;
use crate::services::team_allocation::Team;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Represents the allocation of players to a specific race
#[derive(Debug, Clone)]
pub struct RaceAllocation {
    pub race_number: i32,
    pub player_ids: Vec<Uuid>,
}

/// Allocates players to races using a fair distribution algorithm.
///
/// This function distributes players across multiple races ensuring:
/// - Each race has one player from each team
/// - Players from smaller teams race more frequently (to balance total races)
/// - Players avoid racing consecutively when possible
/// - Each player races approximately the expected number of times
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
    players: &[models::Player],
    teams: &[Team],
    num_races: i32,
    _players_per_race: i32,
) -> Result<Vec<RaceAllocation>> {
    let total_slots = num_races * teams.len() as i32;
    let num_players = players.len();

    let avg_team_size: f64 =
        teams.iter().map(|t| t.players.len()).sum::<usize>() as f64 / teams.len() as f64;
    let base_races_per_player = total_slots as f64 / num_players as f64;

    let mut team_state: Vec<(i32, Vec<Uuid>, HashMap<Uuid, i32>)> = teams
        .iter()
        .map(|team| {
            let team_size = team.players.len() as f64;
            let races_per_player =
                (base_races_per_player * (avg_team_size / team_size)).round() as i32;

            let player_races: HashMap<Uuid, i32> = team
                .players
                .iter()
                .map(|p| (p.id, races_per_player))
                .collect();

            let player_ids: Vec<Uuid> = team.players.iter().map(|p| p.id).collect();

            (team.team_num, player_ids, player_races)
        })
        .collect();

    let mut allocations = Vec::new();

    for race_num in 0..num_races {
        let previous_race_players: HashSet<Uuid> = if race_num > 0 {
            allocations
                .get((race_num - 1) as usize)
                .map(|alloc: &RaceAllocation| alloc.player_ids.iter().copied().collect())
                .unwrap_or_default()
        } else {
            HashSet::new()
        };

        let mut race_players = Vec::new();

        for (_team_num, team_player_ids, player_races) in team_state.iter_mut() {
            let available: Vec<&Uuid> = team_player_ids
                .iter()
                .filter(|pid| *player_races.get(pid).unwrap_or(&0) > 0)
                .collect();

            let selected_player = if available.is_empty() {
                team_player_ids
                    .first()
                    .ok_or_else(|| AppError::Internal("Team has no players".to_string()))?
            } else if available.len() == 1 {
                available[0]
            } else {
                *available
                    .iter()
                    .find(|pid| !previous_race_players.contains(**pid))
                    .unwrap_or(&available[0])
            };

            race_players.push(*selected_player);

            if let Some(count) = player_races.get_mut(selected_player) {
                *count = (*count - 1).max(0);
            }
        }

        allocations.push(RaceAllocation {
            race_number: race_num + 1,
            player_ids: race_players,
        });
    }

    Ok(allocations)
}
