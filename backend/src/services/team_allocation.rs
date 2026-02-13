
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
use rand::seq::SliceRandom;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

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
/// * `elo_ratings` - Map of player IDs to ELO ratings (typically tournament ELO)
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
/// # use std::collections::HashMap;
/// # use uuid::Uuid;
/// let group_id = Uuid::new_v4();
/// let players = vec![
///     Player { id: Uuid::new_v4(), group_id, name: "Alice".to_string(), elo_rating: 1400 },
///     Player { id: Uuid::new_v4(), group_id, name: "Bob".to_string(), elo_rating: 1200 },
///     Player { id: Uuid::new_v4(), group_id, name: "Charlie".to_string(), elo_rating: 1000 },
/// ];
/// let elo_ratings: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();
///
/// let teams = allocate_teams(&players, &2, &elo_ratings);
/// assert_eq!(teams.len(), 2);
/// // Team 1: Alice (1400), Team 2: Bob (1200), Team 1: Charlie (1000)
/// // Results in balanced teams: Team 1 (2400), Team 2 (1200)
/// ```
#[instrument(level = "info", skip(players, elo_ratings), fields(num_players = players.len(), players_per_race = *players_per_race))]
pub fn allocate_teams(
    players: &[models::Player],
    players_per_race: &i32,
    elo_ratings: &HashMap<Uuid, i32>,
) -> Vec<Team> {
    let get_elo = |player: &models::Player| -> i32 {
        elo_ratings
            .get(&player.id)
            .copied()
            .unwrap_or(player.elo_rating)
    };

    let mut sorted_players = players.to_vec();
    sorted_players.sort_by(|a, b| get_elo(b).cmp(&get_elo(a)));

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
            let player_elo = get_elo(&player);
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
                            total_elo: team.total_elo + player_elo,
                        }
                    } else {
                        team
                    }
                })
                .collect()
        })
}

/// Allocates players to teams randomly (no ELO balancing).
///
/// Players are shuffled randomly and then assigned sequentially to teams.
/// Team sizes are still balanced using `calculate_team_sizes`.
/// Total ELO is computed per team for display purposes.
#[instrument(level = "info", skip(players, elo_ratings), fields(num_players = players.len(), players_per_race = *players_per_race))]
pub fn allocate_teams_randomly(
    players: &[models::Player],
    players_per_race: &i32,
    elo_ratings: &HashMap<Uuid, i32>,
) -> Vec<Team> {
    let get_elo = |player: &models::Player| -> i32 {
        elo_ratings
            .get(&player.id)
            .copied()
            .unwrap_or(player.elo_rating)
    };

    let mut shuffled_players = players.to_vec();
    shuffled_players.shuffle(&mut rand::rng());

    let num_players = players.len();
    let num_teams = std::cmp::min(*players_per_race as usize, num_players);
    let team_sizes = calculate_team_sizes(num_players, num_teams);

    // Build prefix sums of team sizes so we can slice the shuffled players
    let offsets: Vec<usize> = team_sizes
        .iter()
        .scan(0usize, |acc, &size| {
            let start = *acc;
            *acc += size;
            Some(start)
        })
        .collect();

    offsets
        .iter()
        .zip(team_sizes.iter())
        .enumerate()
        .map(|(team_idx, (&offset, &size))| {
            let team_players: Vec<models::Player> =
                shuffled_players[offset..offset + size].to_vec();
            let total_elo: i32 = team_players.iter().map(|p| get_elo(p)).sum();

            Team {
                team_num: (team_idx + 1) as i32,
                players: team_players,
                total_elo,
            }
        })
        .collect()
}
