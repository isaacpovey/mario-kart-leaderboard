//! Teammate ELO Service
//!
//! This module provides pure functions for calculating teammate ELO contributions.
//! When players on a team compete in races, their teammates receive 10% of the
//! tournament ELO changes as a bonus contribution.

use std::collections::HashMap;
use uuid::Uuid;

/// Represents a single teammate ELO contribution record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeammateContribution {
    pub source_player_id: Uuid,
    pub beneficiary_player_id: Uuid,
    pub source_tournament_elo_change: i32,
    pub contribution_amount: i32,
}

/// Calculates teammate ELO contributions for a race.
///
/// For each player who participates in a race, their teammates receive 20% of
/// that player's tournament ELO change. This function performs the pure calculation
/// logic without any database operations.
///
/// # Arguments
///
/// * `results` - Slice of (player_id, position) tuples for race participants
/// * `player_to_team` - Map of player IDs to their team IDs
/// * `team_to_players` - Map of team IDs to vectors of player IDs on that team
/// * `tournament_elo_changes` - Map of player IDs to their tournament ELO changes
///
/// # Returns
///
/// A tuple containing:
/// - Vec of TeammateContribution records to be stored
/// - HashMap of beneficiary player IDs to their total ELO adjustment
///
/// # Examples
///
/// ```
/// use mario_kart_leaderboard_backend::services::teammate_elo::calculate_teammate_contributions;
/// use std::collections::HashMap;
/// use uuid::Uuid;
///
/// let player1 = Uuid::new_v4();
/// let player2 = Uuid::new_v4();
/// let team = Uuid::new_v4();
///
/// let results = vec![(player1, 1)];
///
/// let mut player_to_team = HashMap::new();
/// player_to_team.insert(player1, team);
/// player_to_team.insert(player2, team);
///
/// let mut team_to_players = HashMap::new();
/// team_to_players.insert(team, vec![player1, player2]);
///
/// let mut elo_changes = HashMap::new();
/// elo_changes.insert(player1, 50);
///
/// let (contributions, adjustments) = calculate_teammate_contributions(
///     &results,
///     &player_to_team,
///     &team_to_players,
///     &elo_changes,
/// );
///
/// assert_eq!(contributions.len(), 1);
/// assert_eq!(contributions[0].source_player_id, player1);
/// assert_eq!(contributions[0].beneficiary_player_id, player2);
/// assert_eq!(contributions[0].contribution_amount, 10);
/// assert_eq!(adjustments.get(&player2), Some(&10));
/// ```
pub fn calculate_teammate_contributions(
    results: &[(Uuid, i32)],
    player_to_team: &HashMap<Uuid, Uuid>,
    team_to_players: &HashMap<Uuid, Vec<Uuid>>,
    tournament_elo_changes: &HashMap<Uuid, i32>,
) -> (Vec<TeammateContribution>, HashMap<Uuid, i32>) {
    let mut contributions = Vec::new();
    let mut tournament_elo_adjustments: HashMap<Uuid, i32> = HashMap::new();

    for (source_player_id, _position) in results {
        if let Some(team_id) = player_to_team.get(source_player_id) {
            if let Some(teammates) = team_to_players.get(team_id) {
                if let Some(&tournament_elo_change) = tournament_elo_changes.get(source_player_id) {
                    let contribution_amount =
                        (tournament_elo_change as f64 * 0.2).round() as i32;

                    for teammate_id in teammates {
                        if teammate_id != source_player_id {
                            contributions.push(TeammateContribution {
                                source_player_id: *source_player_id,
                                beneficiary_player_id: *teammate_id,
                                source_tournament_elo_change: tournament_elo_change,
                                contribution_amount,
                            });

                            *tournament_elo_adjustments.entry(*teammate_id).or_insert(0) +=
                                contribution_amount;
                        }
                    }
                }
            }
        }
    }

    (contributions, tournament_elo_adjustments)
}
