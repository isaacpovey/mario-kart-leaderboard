use mario_kart_leaderboard_backend::services::score_calculation::calculate_team_scores_from_positions;
use uuid::Uuid;

// ============================================================================
// Tests for `calculate_team_scores_from_positions`
// ============================================================================

#[test]
fn test_calculate_team_scores_single_team_single_race() {
    let team_id = Uuid::new_v4();
    let race_scores = vec![(team_id, 1)];

    let scores = calculate_team_scores_from_positions(&race_scores, 1);

    assert_eq!(scores.len(), 1);
    assert_eq!(scores.get(&team_id), Some(&15.0));
}

#[test]
fn test_calculate_team_scores_multiple_teams() {
    let team1 = Uuid::new_v4();
    let team2 = Uuid::new_v4();
    let race_scores = vec![(team1, 1), (team2, 2)];

    let scores = calculate_team_scores_from_positions(&race_scores, 1);

    assert_eq!(scores.len(), 2);
    assert_eq!(scores.get(&team1), Some(&15.0));
    assert_eq!(scores.get(&team2), Some(&12.0));
}

#[test]
fn test_calculate_team_scores_multiple_races() {
    let team_id = Uuid::new_v4();
    let race_scores = vec![(team_id, 1), (team_id, 2)];

    let scores = calculate_team_scores_from_positions(&race_scores, 2);

    let expected_avg = (15.0 + 12.0) / 2.0;
    assert_eq!(scores.get(&team_id), Some(&expected_avg));
}

#[test]
fn test_calculate_team_scores_empty_input() {
    let race_scores: Vec<(Uuid, i32)> = vec![];

    let scores = calculate_team_scores_from_positions(&race_scores, 1);

    assert!(scores.is_empty());
}

#[test]
fn test_calculate_team_scores_position_points_mapping() {
    let team_id = Uuid::new_v4();

    let test_cases = vec![
        (1, 15.0),
        (2, 12.0),
        (3, 10.0),
        (4, 9.0),
        (5, 8.0),
        (6, 7.0),
        (7, 6.0),
        (8, 5.0),
        (9, 4.0),
        (10, 3.0),
        (11, 2.0),
        (12, 1.0),
        (13, 0.0),
        (24, 0.0),
    ];

    for (position, expected_points) in test_cases {
        let race_scores = vec![(team_id, position)];
        let scores = calculate_team_scores_from_positions(&race_scores, 1);
        assert_eq!(
            scores.get(&team_id),
            Some(&expected_points),
            "Position {} should give {} points",
            position,
            expected_points
        );
    }
}

#[test]
fn test_calculate_team_scores_average_calculation() {
    let team_id = Uuid::new_v4();
    let race_scores = vec![
        (team_id, 1),
        (team_id, 3),
        (team_id, 5),
        (team_id, 7),
    ];

    let scores = calculate_team_scores_from_positions(&race_scores, 4);

    let expected_avg = (15.0 + 10.0 + 8.0 + 6.0) / 4.0;
    assert_eq!(scores.get(&team_id), Some(&expected_avg));
}

#[test]
fn test_calculate_team_scores_all_same_position() {
    let team1 = Uuid::new_v4();
    let team2 = Uuid::new_v4();
    let race_scores = vec![
        (team1, 1),
        (team1, 1),
        (team2, 1),
        (team2, 1),
    ];

    let scores = calculate_team_scores_from_positions(&race_scores, 2);

    assert_eq!(scores.get(&team1), Some(&15.0));
    assert_eq!(scores.get(&team2), Some(&15.0));
}

#[test]
fn test_calculate_team_scores_multiple_teams_multiple_races() {
    let team1 = Uuid::new_v4();
    let team2 = Uuid::new_v4();
    let team3 = Uuid::new_v4();
    let race_scores = vec![
        (team1, 1),
        (team2, 2),
        (team3, 3),
        (team1, 3),
        (team2, 1),
        (team3, 2),
    ];

    let scores = calculate_team_scores_from_positions(&race_scores, 2);

    let team1_expected = (15.0 + 10.0) / 2.0;
    let team2_expected = (12.0 + 15.0) / 2.0;
    let team3_expected = (10.0 + 12.0) / 2.0;

    assert_eq!(scores.get(&team1), Some(&team1_expected));
    assert_eq!(scores.get(&team2), Some(&team2_expected));
    assert_eq!(scores.get(&team3), Some(&team3_expected));
}

#[test]
fn test_calculate_team_scores_worst_positions() {
    let team_id = Uuid::new_v4();
    let race_scores = vec![
        (team_id, 24),
        (team_id, 23),
        (team_id, 22),
    ];

    let scores = calculate_team_scores_from_positions(&race_scores, 3);

    assert_eq!(scores.get(&team_id), Some(&0.0));
}

#[test]
fn test_calculate_team_scores_single_player_per_team() {
    let team1 = Uuid::new_v4();
    let team2 = Uuid::new_v4();
    let race_scores = vec![(team1, 1), (team2, 12)];

    let scores = calculate_team_scores_from_positions(&race_scores, 1);

    assert_eq!(scores.get(&team1), Some(&15.0));
    assert_eq!(scores.get(&team2), Some(&1.0));
}
