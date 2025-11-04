use mario_kart_leaderboard_backend::services::elo::*;
use uuid::Uuid;

fn create_player(id: u128, position: i32, elo: i32) -> PlayerResult {
    PlayerResult {
        player_id: Uuid::from_u128(id),
        position,
        current_elo: elo,
    }
}

#[test]
fn test_elo_two_equal_players_head_to_head() {
    let results = vec![create_player(1, 1, 1200), create_player(2, 2, 1200)];

    let changes = calculate_elo_changes(&results);

    assert_eq!(changes.len(), 2);

    let winner = changes
        .iter()
        .find(|c| c.player_id == Uuid::from_u128(1))
        .unwrap();
    let loser = changes
        .iter()
        .find(|c| c.player_id == Uuid::from_u128(2))
        .unwrap();

    assert!(winner.elo_change > 0, "Winner should gain ELO");
    assert!(
        loser.elo_change > 0,
        "2nd place also gains ELO (beats 22 CPUs at lower ELO)"
    );
    assert!(
        winner.elo_change > loser.elo_change,
        "Winner should gain more than 2nd place"
    );
}

#[test]
fn test_elo_underdog_wins() {
    let results = vec![create_player(1, 1, 1000), create_player(2, 2, 1400)];

    let changes = calculate_elo_changes(&results);

    let underdog = changes
        .iter()
        .find(|c| c.player_id == Uuid::from_u128(1))
        .unwrap();
    let _favorite = changes
        .iter()
        .find(|c| c.player_id == Uuid::from_u128(2))
        .unwrap();

    assert!(underdog.elo_change > 0, "Underdog should gain ELO");

    let equal_match_changes =
        calculate_elo_changes(&[create_player(1, 1, 1200), create_player(2, 2, 1200)]);
    let equal_winner = equal_match_changes
        .iter()
        .find(|c| c.player_id == Uuid::from_u128(1))
        .unwrap();

    assert!(
        underdog.elo_change > equal_winner.elo_change,
        "Underdog beating higher rated player should gain more ELO than equal match"
    );
}

#[test]
fn test_elo_with_cpu_opponents() {
    let results = vec![create_player(1, 1, 1200), create_player(2, 2, 1200)];

    let changes = calculate_elo_changes(&results);

    assert_eq!(changes.len(), 2);

    let first = changes
        .iter()
        .find(|c| c.player_id == Uuid::from_u128(1))
        .unwrap();
    let second = changes
        .iter()
        .find(|c| c.player_id == Uuid::from_u128(2))
        .unwrap();

    assert!(
        first.elo_change > 0,
        "1st place should gain ELO (beats 23 opponents including 22 CPUs)"
    );
    assert!(
        second.elo_change > 0,
        "2nd place should gain ELO (beats 22 CPUs at lower ELO)"
    );
    assert!(
        first.new_elo > second.new_elo,
        "1st place should end with higher ELO"
    );
}

#[test]
fn test_elo_many_human_players_ranking() {
    let results = vec![
        create_player(1, 1, 1200),
        create_player(2, 3, 1200),
        create_player(3, 5, 1200),
        create_player(4, 10, 1200),
        create_player(5, 15, 1200),
        create_player(6, 20, 1200),
    ];

    let changes = calculate_elo_changes(&results);

    let elo_gains: Vec<(u128, i32)> = changes
        .iter()
        .map(|c| (c.player_id.as_u128(), c.elo_change))
        .collect();

    let get_change = |id: u128| elo_gains.iter().find(|(pid, _)| *pid == id).unwrap().1;

    let change_1st = get_change(1);
    let change_3rd = get_change(2);
    let change_5th = get_change(3);
    let change_10th = get_change(4);
    let change_20th = get_change(6);

    assert!(
        change_1st > change_3rd,
        "1st should gain more than 3rd: {} vs {}",
        change_1st,
        change_3rd
    );
    assert!(
        change_3rd > change_5th,
        "3rd should gain more than 5th: {} vs {}",
        change_3rd,
        change_5th
    );
    assert!(
        change_5th > change_10th,
        "5th should gain more than 10th: {} vs {}",
        change_5th,
        change_10th
    );

    assert!(
        change_20th < 0,
        "20th place should lose ELO (only beats 4 opponents): {}",
        change_20th
    );
}

#[test]
fn test_elo_edge_case_single_player() {
    let results = vec![create_player(1, 1, 1200)];

    let changes = calculate_elo_changes(&results);

    assert_eq!(changes.len(), 1);
    let change = &changes[0];

    assert!(
        change.elo_change.abs() < 50,
        "Single player should have minimal ELO change: {}",
        change.elo_change
    );
}

#[test]
fn test_elo_extreme_rating_differences() {
    let results = vec![create_player(1, 1, 500), create_player(2, 2, 2000)];

    let changes = calculate_elo_changes(&results);

    let underdog = changes
        .iter()
        .find(|c| c.player_id == Uuid::from_u128(1))
        .unwrap();
    let favorite = changes
        .iter()
        .find(|c| c.player_id == Uuid::from_u128(2))
        .unwrap();

    assert!(
        underdog.elo_change > 20,
        "500 ELO player beating 2000 ELO player should gain significant ELO: {}",
        underdog.elo_change
    );

    assert!(
        underdog.elo_change > favorite.elo_change,
        "Underdog should gain more"
    );
}

#[test]
fn test_position_to_score_boundary() {
    use mario_kart_leaderboard_backend::services::elo::position_to_score;

    let score_1st = position_to_score(1);
    let score_24th = position_to_score(24);

    assert_eq!(score_1st, 1.0, "1st place should have score of 1.0");
    assert_eq!(score_24th, 0.0, "24th place should have score of 0.0");
}

#[test]
fn test_full_field_creation() {
    use mario_kart_leaderboard_backend::services::elo::create_full_field;

    let results = vec![create_player(1, 1, 1200), create_player(2, 5, 1200)];

    let full_field = create_full_field(&results);

    assert_eq!(full_field.len(), 24, "Full field should have 24 players");

    let cpu_count = full_field
        .iter()
        .filter(|p| p.player_id == Uuid::nil())
        .count();
    assert_eq!(cpu_count, 22, "Should have 22 CPU players");

    let human_count = full_field
        .iter()
        .filter(|p| p.player_id != Uuid::nil())
        .count();
    assert_eq!(human_count, 2, "Should have 2 human players");

    let has_position_1 = full_field.iter().any(|p| p.position == 1);
    let has_position_5 = full_field.iter().any(|p| p.position == 5);
    assert!(
        has_position_1 && has_position_5,
        "Human positions should be in full field"
    );
}
