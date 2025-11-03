use uuid::Uuid;

const K_FACTOR: f64 = 32.0;
const TOTAL_RACE_SIZE: usize = 24;
const CPU_ELO: i32 = 1000;

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

fn create_full_field(human_results: &[PlayerResult]) -> Vec<PlayerResult> {
    let mut full_field = Vec::with_capacity(TOTAL_RACE_SIZE);

    full_field.extend_from_slice(human_results);

    let human_positions: std::collections::HashSet<i32> =
        human_results.iter().map(|r| r.position).collect();

    for position in 1..=TOTAL_RACE_SIZE as i32 {
        if !human_positions.contains(&position) {
            full_field.push(PlayerResult {
                player_id: Uuid::nil(),
                position,
                current_elo: CPU_ELO,
            });
        }
    }

    full_field
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

fn position_to_score(position: i32) -> f64 {
    (TOTAL_RACE_SIZE as i32 - position) as f64 / (TOTAL_RACE_SIZE - 1) as f64
}
