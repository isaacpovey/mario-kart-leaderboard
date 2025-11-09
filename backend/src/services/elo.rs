//! ELO Rating System for Mario Kart Races
//!
//! This module implements an ELO rating system adapted for 24-player Mario Kart races.
//! Unlike traditional 1v1 ELO, this system accounts for:
//! - Multiple players racing simultaneously (up to 24 positions)
//! - CPU opponents filling empty positions (assumed ELO of 800)
//! - Normalized scoring based on final race position
//!
//! ## How It Works
//!
//! The ELO calculation follows these steps:
//!
//! 1. **Create Full Field**: Fill a 24-player race with human players and CPU opponents
//! 2. **Calculate Expected Score**: For each player, calculate their expected performance
//!    against all other opponents using the standard ELO formula
//! 3. **Calculate Actual Score**: Convert race position (1-24) to a normalized score (1.0-0.0)
//! 4. **Apply K-Factor**: Calculate rating change using K=100 and the difference between
//!    expected and actual scores
//!
//! ## Examples
//!
//! ### Basic 2-Player Race
//! ```rust
//! # use mario_kart_leaderboard_backend::services::elo::*;
//! # use uuid::Uuid;
//! let results = vec![
//!     PlayerResult {
//!         player_id: Uuid::new_v4(),
//!         position: 1,
//!         current_elo: 1200,
//!     },
//!     PlayerResult {
//!         player_id: Uuid::new_v4(),
//!         position: 2,
//!         current_elo: 1200,
//!     },
//! ];
//!
//! let changes = calculate_elo_changes(&results);
//! // 1st place gains ELO (beats 23 opponents: 1 human + 22 CPUs)
//! // 2nd place also gains ELO (beats 22 CPU opponents at 1000 ELO)
//! // 1st place gains more than 2nd place
//! ```
//!
//! ### Why 2nd Place Can Gain ELO
//!
//! In a 24-player race, finishing 2nd means:
//! - Lost to 1 opponent (1st place)
//! - Beat 22 opponents (positions 3-24, all CPUs at ELO 800)
//!
//! When both human players have ELO 1200:
//! - Beating 22 lower-rated opponents (800 ELO) provides significant ELO gain
//! - Losing to 1 equal-rated opponent (1200 ELO) provides moderate ELO loss
//! - Net result: 2nd place gains ELO (but less than 1st place)
//!
//! ## Constants

use std::cmp::max;
use tracing::instrument;
use uuid::Uuid;

/// K-factor determines the maximum rating change per race
const K_FACTOR: f64 = 100.0;

/// Total number of racers in a Mario Kart race (including CPUs)
const TOTAL_RACE_SIZE: usize = 24;

const MAX_CPU_ELO: i32 = 1400;
const MIN_CPU_ELO: i32 = 600;
const CPU_ELO_DECREASE: i32 = 100;

/// Represents a player's result in a single race
#[derive(Debug, Clone)]
pub struct PlayerResult {
    pub player_id: Uuid,
    pub position: i32,
    pub current_elo: i32,
}

/// Represents the ELO rating change for a player after a race
#[derive(Debug, Clone)]
pub struct EloChange {
    pub player_id: Uuid,
    pub elo_change: i32,
    pub new_elo: i32,
}

/// Calculates ELO rating changes for all players in a race.
///
/// Takes a list of human players and their finishing positions, fills the remaining
/// positions with CPU opponents, and calculates the appropriate ELO change for each
/// human player based on their performance against all opponents.
///
/// # Arguments
///
/// * `results` - Slice of PlayerResult structs representing human players' race results
///
/// # Returns
///
/// Vector of EloChange structs containing the rating changes for each player
///
/// # Example
///
/// ```rust
/// # use mario_kart_leaderboard_backend::services::elo::*;
/// # use uuid::Uuid;
/// let results = vec![
///     PlayerResult { player_id: Uuid::new_v4(), position: 1, current_elo: 1400 },
///     PlayerResult { player_id: Uuid::new_v4(), position: 10, current_elo: 1200 },
/// ];
/// let changes = calculate_elo_changes(&results);
/// // changes[0] will have a positive elo_change (1st place)
/// // changes[1] will have a smaller positive or negative change (10th place)
/// ```
#[instrument(level = "info", fields(player_count = results.len()))]
pub fn calculate_elo_changes(results: &[PlayerResult]) -> Vec<EloChange> {
    let full_field = create_full_field(results);

    results
        .iter()
        .map(|player| {
            let expected_score = calculate_expected_score(player, &full_field);
            let actual_score = position_to_score(player.position);
            let elo_change = (K_FACTOR * (actual_score - expected_score)).round() as i32;
            let new_elo = player.current_elo + elo_change;

            EloChange {
                player_id: player.player_id,
                elo_change,
                new_elo,
            }
        })
        .collect()
}

/// Internal function: Creates a full 24-player field by filling empty positions with CPU opponents.
/// Exposed for testing purposes.
#[instrument(level = "debug", fields(human_count = human_results.len()))]
pub fn create_full_field(human_results: &[PlayerResult]) -> Vec<PlayerResult> {
    let mut full_field = Vec::with_capacity(TOTAL_RACE_SIZE);

    full_field.extend_from_slice(human_results);

    let human_positions: std::collections::HashSet<i32> =
        human_results.iter().map(|r| r.position).collect();

    for position in 1..=TOTAL_RACE_SIZE as i32 {
        if !human_positions.contains(&position) {
            let cpu_elo = max(MIN_CPU_ELO, MAX_CPU_ELO - ((position - 1) * CPU_ELO_DECREASE));
            full_field.push(PlayerResult {
                player_id: Uuid::nil(),
                position,
                current_elo: cpu_elo,
            });
        }
    }

    full_field
}

#[instrument(level = "debug", skip(all_results), fields(opponent_count = all_results.len().saturating_sub(1), player_elo = player.current_elo))]
fn calculate_expected_score(player: &PlayerResult, all_results: &[PlayerResult]) -> f64 {
    let opponent_count = all_results.len().saturating_sub(1);
    if opponent_count == 0 {
        return 0.5;
    }

    all_results
        .iter()
        .filter(|other| other.player_id != player.player_id)
        .map(|other| {
            let rating_diff = (other.current_elo - player.current_elo) as f64;
            1.0 / (1.0 + 10_f64.powf(rating_diff / 400.0))
        })
        .sum::<f64>()
        / opponent_count as f64
}

/// Internal function: Converts race position (1-24) to normalized score (1.0-0.0).
/// Exposed for testing purposes.
pub fn position_to_score(position: i32) -> f64 {
    if TOTAL_RACE_SIZE <= 1 {
        return 0.5;
    }
    (TOTAL_RACE_SIZE as i32 - position) as f64 / (TOTAL_RACE_SIZE - 1) as f64
}
