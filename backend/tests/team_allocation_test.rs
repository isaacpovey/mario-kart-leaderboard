use mario_kart_leaderboard_backend::models::Player;
use mario_kart_leaderboard_backend::services::team_allocation::{allocate_teams, calculate_team_sizes};
use std::collections::HashMap;
use uuid::Uuid;

fn create_test_player(name: &str, elo_rating: i32) -> Player {
    Player {
        id: Uuid::new_v4(),
        group_id: Uuid::new_v4(),
        name: name.to_string(),
        elo_rating,
        avatar_filename: None,
    }
}

// ============================================================================
// Tests for `calculate_team_sizes`
// ============================================================================

#[test]
fn test_calculate_team_sizes_even_distribution() {
    let sizes = calculate_team_sizes(12, 4);
    assert_eq!(sizes, vec![3, 3, 3, 3]);
}

#[test]
fn test_calculate_team_sizes_uneven_distribution() {
    let sizes = calculate_team_sizes(10, 3);
    assert_eq!(sizes, vec![4, 3, 3]);
}

#[test]
fn test_calculate_team_sizes_remainder_distributed() {
    let sizes = calculate_team_sizes(11, 4);
    assert_eq!(sizes, vec![3, 3, 3, 2]);
}

#[test]
fn test_calculate_team_sizes_single_team() {
    let sizes = calculate_team_sizes(5, 1);
    assert_eq!(sizes, vec![5]);
}

#[test]
fn test_calculate_team_sizes_more_teams_than_players() {
    let sizes = calculate_team_sizes(2, 4);
    assert_eq!(sizes, vec![1, 1, 0, 0]);
}

#[test]
fn test_calculate_team_sizes_single_player() {
    let sizes = calculate_team_sizes(1, 2);
    assert_eq!(sizes, vec![1, 0]);
}

#[test]
fn test_calculate_team_sizes_two_players_two_teams() {
    let sizes = calculate_team_sizes(2, 2);
    assert_eq!(sizes, vec![1, 1]);
}

// ============================================================================
// Tests for `allocate_teams`
// ============================================================================

#[test]
fn test_allocate_teams_basic_four_players() {
    let players = vec![
        create_test_player("Alice", 1400),
        create_test_player("Bob", 1300),
        create_test_player("Charlie", 1200),
        create_test_player("Dave", 1100),
    ];
    let elo_map: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();

    let teams = allocate_teams(&players, &2, &elo_map);

    assert_eq!(teams.len(), 2);
    assert_eq!(teams[0].players.len(), 2);
    assert_eq!(teams[1].players.len(), 2);
}

#[test]
fn test_allocate_teams_balanced_elo_distribution() {
    let players = vec![
        create_test_player("High1", 1500),
        create_test_player("High2", 1400),
        create_test_player("Low1", 900),
        create_test_player("Low2", 800),
    ];
    let elo_map: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();

    let teams = allocate_teams(&players, &2, &elo_map);

    let elo_diff = (teams[0].total_elo - teams[1].total_elo).abs();
    assert!(
        elo_diff <= 200,
        "Teams should be reasonably balanced, got diff: {}",
        elo_diff
    );
}

#[test]
fn test_allocate_teams_odd_number_of_players() {
    let players = vec![
        create_test_player("Player1", 1400),
        create_test_player("Player2", 1300),
        create_test_player("Player3", 1200),
    ];
    let elo_map: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();

    let teams = allocate_teams(&players, &2, &elo_map);

    assert_eq!(teams.len(), 2);
    let total_players: usize = teams.iter().map(|t| t.players.len()).sum();
    assert_eq!(total_players, 3);
}

#[test]
fn test_allocate_teams_large_elo_gaps() {
    let players = vec![
        create_test_player("Pro1", 2000),
        create_test_player("Pro2", 1900),
        create_test_player("Newbie1", 800),
        create_test_player("Newbie2", 700),
    ];
    let elo_map: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();

    let teams = allocate_teams(&players, &2, &elo_map);

    for team in &teams {
        let has_high_elo = team.players.iter().any(|p| p.elo_rating >= 1500);
        let has_low_elo = team.players.iter().any(|p| p.elo_rating < 1000);
        assert!(
            has_high_elo && has_low_elo,
            "Each team should have mixed skill levels"
        );
    }
}

#[test]
fn test_allocate_teams_same_elo_all_players() {
    let players = vec![
        create_test_player("Player1", 1200),
        create_test_player("Player2", 1200),
        create_test_player("Player3", 1200),
        create_test_player("Player4", 1200),
    ];
    let elo_map: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();

    let teams = allocate_teams(&players, &2, &elo_map);

    assert_eq!(teams.len(), 2);
    assert_eq!(teams[0].total_elo, teams[1].total_elo);
}

#[test]
fn test_allocate_teams_single_player() {
    let players = vec![create_test_player("Solo", 1200)];
    let elo_map: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();

    let teams = allocate_teams(&players, &2, &elo_map);

    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].players.len(), 1);
}

#[test]
fn test_allocate_teams_many_players() {
    let players: Vec<Player> = (0..12)
        .map(|i| create_test_player(&format!("Player{}", i), 1000 + i * 50))
        .collect();
    let elo_map: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();

    let teams = allocate_teams(&players, &4, &elo_map);

    assert_eq!(teams.len(), 4);
    let total_players: usize = teams.iter().map(|t| t.players.len()).sum();
    assert_eq!(total_players, 12);
}

#[test]
fn test_allocate_teams_uses_custom_elo_map() {
    let players = vec![
        create_test_player("Alice", 1000),
        create_test_player("Bob", 1500),
    ];
    let mut custom_elo_map = HashMap::new();
    custom_elo_map.insert(players[0].id, 2000);
    custom_elo_map.insert(players[1].id, 500);

    let teams = allocate_teams(&players, &2, &custom_elo_map);

    let alice_team = teams
        .iter()
        .find(|t| t.players.iter().any(|p| p.name == "Alice"))
        .expect("Alice should be in a team");
    assert_eq!(
        alice_team.total_elo, 2000,
        "Alice's team should use custom ELO (2000), not default (1000)"
    );
}

#[test]
fn test_allocate_teams_players_sorted_by_elo() {
    let players = vec![
        create_test_player("Low", 800),
        create_test_player("High", 1600),
        create_test_player("Mid", 1200),
    ];
    let elo_map: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();

    let teams = allocate_teams(&players, &3, &elo_map);

    assert_eq!(teams.len(), 3);
    let team_elos: Vec<i32> = teams.iter().map(|t| t.total_elo).collect();
    assert!(
        team_elos.contains(&1600),
        "Highest ELO player should be in own team"
    );
    assert!(
        team_elos.contains(&1200),
        "Mid ELO player should be in own team"
    );
    assert!(
        team_elos.contains(&800),
        "Low ELO player should be in own team"
    );
}

#[test]
fn test_allocate_teams_team_numbers_assigned() {
    let players = vec![
        create_test_player("Player1", 1200),
        create_test_player("Player2", 1200),
    ];
    let elo_map: HashMap<Uuid, i32> = players.iter().map(|p| (p.id, p.elo_rating)).collect();

    let teams = allocate_teams(&players, &2, &elo_map);

    assert_eq!(teams[0].team_num, 1);
    assert_eq!(teams[1].team_num, 2);
}

#[test]
#[should_panic(expected = "attempt to divide by zero")]
fn test_allocate_teams_empty_players_panics() {
    let players: Vec<Player> = vec![];
    let elo_map: HashMap<Uuid, i32> = HashMap::new();

    let _ = allocate_teams(&players, &2, &elo_map);
}
