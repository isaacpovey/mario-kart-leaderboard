//! Team Allocation Service
//!
//! This module provides algorithms for creating balanced teams based on player ELO ratings.
//! Teams are allocated using a greedy algorithm that ensures:
//! - Teams have approximately equal total ELO
//! - Team sizes are as balanced as possible
//! - Higher-rated players are distributed evenly across teams
//!
//! ## Algorithm
//!
//! 1. Sort players by ELO (highest first)
//! 2. Calculate team sizes (distributing remainder players evenly)
//! 3. Iteratively assign each player to the team with the lowest total ELO
//!    that still has capacity

use crate::models;
use tracing::instrument;

/// Represents a team with its players and total ELO rating
#[derive(Debug, Clone)]
pub struct Team {
    pub team_num: i32,
    pub players: Vec<models::Player>,
    pub total_elo: i32,
}

/// Calculates how many players should be on each team.
///
/// Distributes players as evenly as possible across teams.
/// If there's a remainder, the first teams get one extra player each.
///
/// # Arguments
///
/// * `num_players` - Total number of players to distribute
/// * `num_teams` - Number of teams to create
///
/// # Returns
///
/// Vector where each element is the size of the corresponding team
///
/// # Examples
///
/// ```
/// # use mario_kart_leaderboard_backend::services::team_allocation::calculate_team_sizes;
/// // 10 players across 3 teams: [4, 3, 3]
/// let sizes = calculate_team_sizes(10, 3);
/// assert_eq!(sizes, vec![4, 3, 3]);
///
/// // 12 players across 4 teams: [3, 3, 3, 3]
/// let sizes = calculate_team_sizes(12, 4);
/// assert_eq!(sizes, vec![3, 3, 3, 3]);
/// ```
#[instrument(level = "debug")]
pub fn calculate_team_sizes(num_players: usize, num_teams: usize) -> Vec<usize> {
    let base_size = num_players / num_teams;
    let remainder = num_players % num_teams;

    (0..num_teams)
        .map(|team_idx| {
            if team_idx < remainder {
                base_size + 1
            } else {
                base_size
            }
        })
        .collect()
}

/// Allocates players to teams using a balanced ELO distribution algorithm.
///
/// This function implements a greedy algorithm that creates balanced teams:
/// 1. Players are sorted by ELO rating (highest first)
/// 2. Each player is assigned to the team with the lowest total ELO that still has capacity
/// 3. This ensures teams are balanced in both size and skill level
///
/// # Arguments
///
/// * `players` - Slice of players to allocate to teams
/// * `players_per_race` - Maximum number of players per race (determines number of teams)
///
/// # Returns
///
/// Vector of teams with players distributed across them
///
/// # Examples
///
/// ```ignore
/// # use mario_kart_leaderboard_backend::services::team_allocation::allocate_teams;
/// # use mario_kart_leaderboard_backend::models::Player;
/// # use uuid::Uuid;
/// let group_id = Uuid::new_v4();
/// let players = vec![
///     Player { id: Uuid::new_v4(), group_id, name: "Alice".to_string(), elo_rating: 1400 },
///     Player { id: Uuid::new_v4(), group_id, name: "Bob".to_string(), elo_rating: 1200 },
///     Player { id: Uuid::new_v4(), group_id, name: "Charlie".to_string(), elo_rating: 1000 },
/// ];
///
/// let teams = allocate_teams(&players, &2);
/// assert_eq!(teams.len(), 2);
/// // Team 1: Alice (1400), Team 2: Bob (1200), Team 1: Charlie (1000)
/// // Results in balanced teams: Team 1 (2400), Team 2 (1200)
/// ```
#[instrument(level = "info", skip(players), fields(num_players = players.len(), players_per_race = *players_per_race))]
pub fn allocate_teams(players: &[models::Player], players_per_race: &i32) -> Vec<Team> {
    let mut sorted_players = players.to_vec();
    sorted_players.sort_by(|a, b| b.elo_rating.cmp(&a.elo_rating));

    let num_players = players.len();
    let num_teams = std::cmp::min(*players_per_race as usize, num_players);
    let team_sizes = calculate_team_sizes(num_players, num_teams);

    let initial_teams: Vec<Team> = (0..num_teams)
        .map(|team_idx| Team {
            team_num: (team_idx + 1) as i32,
            players: Vec::new(),
            total_elo: 0,
        })
        .collect();

    sorted_players
        .into_iter()
        .fold(initial_teams, |teams, player| {
            let team_idx = teams
                .iter()
                .enumerate()
                .filter(|(idx, team)| team.players.len() < team_sizes[*idx])
                .min_by_key(|(_, team)| team.total_elo)
                .map(|(idx, _)| idx)
                .unwrap_or(0);

            teams
                .into_iter()
                .enumerate()
                .map(|(idx, team)| {
                    if idx == team_idx {
                        Team {
                            team_num: team.team_num,
                            players: team
                                .players
                                .iter()
                                .cloned()
                                .chain(std::iter::once(player.clone()))
                                .collect(),
                            total_elo: team.total_elo + player.elo_rating,
                        }
                    } else {
                        team
                    }
                })
                .collect()
        })
}
