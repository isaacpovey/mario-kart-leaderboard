use uuid::Uuid;

const K_FACTOR: f64 = 32.0;

#[derive(Debug, Clone)]
pub struct PlayerResult {
    pub player_id: Uuid,
    pub position: i32,
    pub current_elo: i32,
}

#[derive(Debug, Clone)]
pub struct EloChange {
    pub player_id: Uuid,
    pub elo_change: i32,
    pub new_elo: i32,
}

pub fn calculate_elo_changes(results: &[PlayerResult]) -> Vec<EloChange> {
    let num_players = results.len();

    results
        .iter()
        .map(|player| {
            let expected_score = calculate_expected_score(player, results);
            let actual_score = position_to_score(player.position, num_players);
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

fn calculate_expected_score(player: &PlayerResult, all_results: &[PlayerResult]) -> f64 {
    all_results
        .iter()
        .filter(|other| other.player_id != player.player_id)
        .map(|other| {
            let rating_diff = (other.current_elo - player.current_elo) as f64;
            1.0 / (1.0 + 10_f64.powf(rating_diff / 400.0))
        })
        .sum::<f64>()
        / (all_results.len() - 1) as f64
}

fn position_to_score(position: i32, total_players: usize) -> f64 {
    (total_players as i32 - position) as f64 / (total_players - 1) as f64
}
