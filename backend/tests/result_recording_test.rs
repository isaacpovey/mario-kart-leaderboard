use mario_kart_leaderboard_backend::models::Player;
use mario_kart_leaderboard_backend::services::result_recording::{
    create_player_elo_map, create_player_results,
};
use std::collections::HashMap;
use uuid::Uuid;

fn create_test_player(id: Uuid, group_id: Uuid, name: &str, elo_rating: i32) -> Player {
    Player {
        id,
        group_id,
        name: name.to_string(),
        elo_rating,
        avatar_filename: None,
    }
}

#[test]
fn test_create_player_elo_map() {
    let group_id = Uuid::new_v4();
    let player1_id = Uuid::new_v4();
    let player2_id = Uuid::new_v4();

    let players = vec![
        create_test_player(player1_id, group_id, "Player 1", 1200),
        create_test_player(player2_id, group_id, "Player 2", 1500),
    ];

    let elo_map = create_player_elo_map(&players);

    assert_eq!(elo_map.len(), 2);
    assert_eq!(elo_map.get(&player1_id), Some(&1200));
    assert_eq!(elo_map.get(&player2_id), Some(&1500));
}

#[test]
fn test_create_player_elo_map_empty() {
    let players: Vec<Player> = vec![];
    let elo_map = create_player_elo_map(&players);
    assert_eq!(elo_map.len(), 0);
}

#[test]
fn test_create_player_elo_map_single_player() {
    let group_id = Uuid::new_v4();
    let player_id = Uuid::new_v4();

    let players = vec![create_test_player(player_id, group_id, "Player", 1300)];

    let elo_map = create_player_elo_map(&players);

    assert_eq!(elo_map.len(), 1);
    assert_eq!(elo_map.get(&player_id), Some(&1300));
}

#[test]
fn test_create_player_results_success() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();

    let results = vec![(uuid1, 1), (uuid2, 2)];

    let mut elos = HashMap::new();
    elos.insert(uuid1, 1200);
    elos.insert(uuid2, 1300);

    let player_results = create_player_results(&results, &elos).unwrap();

    assert_eq!(player_results.len(), 2);
    assert_eq!(player_results[0].player_id, uuid1);
    assert_eq!(player_results[0].position, 1);
    assert_eq!(player_results[0].current_elo, 1200);
    assert_eq!(player_results[1].player_id, uuid2);
    assert_eq!(player_results[1].position, 2);
    assert_eq!(player_results[1].current_elo, 1300);
}

#[test]
fn test_create_player_results_missing_player() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();

    let results = vec![(uuid1, 1), (uuid2, 2)];

    let mut elos = HashMap::new();
    elos.insert(uuid1, 1200);

    let result = create_player_results(&results, &elos);

    assert!(result.is_err());
}

#[test]
fn test_create_player_results_empty() {
    let results: Vec<(Uuid, i32)> = vec![];
    let elos = HashMap::new();

    let player_results = create_player_results(&results, &elos).unwrap();

    assert_eq!(player_results.len(), 0);
}

#[test]
fn test_create_player_results_all_missing() {
    let uuid1 = Uuid::new_v4();
    let results = vec![(uuid1, 1)];
    let elos = HashMap::new();

    let result = create_player_results(&results, &elos);

    assert!(result.is_err());
}

#[test]
fn test_create_player_results_varied_positions() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let uuid3 = Uuid::new_v4();

    let results = vec![(uuid1, 1), (uuid2, 12), (uuid3, 24)];

    let mut elos = HashMap::new();
    elos.insert(uuid1, 1500);
    elos.insert(uuid2, 1200);
    elos.insert(uuid3, 900);

    let player_results = create_player_results(&results, &elos).unwrap();

    assert_eq!(player_results.len(), 3);
    assert_eq!(player_results[0].position, 1);
    assert_eq!(player_results[0].current_elo, 1500);
    assert_eq!(player_results[1].position, 12);
    assert_eq!(player_results[1].current_elo, 1200);
    assert_eq!(player_results[2].position, 24);
    assert_eq!(player_results[2].current_elo, 900);
}
