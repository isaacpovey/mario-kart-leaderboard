use mario_kart_leaderboard_backend::services::teammate_elo::calculate_teammate_contributions;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_contribution_is_10_percent() {
    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();
    let team = Uuid::new_v4();

    let results = vec![(player1, 1)];

    let mut player_to_team = HashMap::new();
    player_to_team.insert(player1, team);
    player_to_team.insert(player2, team);

    let mut team_to_players = HashMap::new();
    team_to_players.insert(team, vec![player1, player2]);

    let mut elo_changes = HashMap::new();
    elo_changes.insert(player1, 50);

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0].source_player_id, player1);
    assert_eq!(contributions[0].beneficiary_player_id, player2);
    assert_eq!(contributions[0].source_tournament_elo_change, 50);
    assert_eq!(contributions[0].contribution_amount, 10);
    assert_eq!(adjustments.get(&player2), Some(&10));
}

#[test]
fn test_contribution_negative_elo() {
    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();
    let team = Uuid::new_v4();

    let results = vec![(player1, 10)];

    let mut player_to_team = HashMap::new();
    player_to_team.insert(player1, team);
    player_to_team.insert(player2, team);

    let mut team_to_players = HashMap::new();
    team_to_players.insert(team, vec![player1, player2]);

    let mut elo_changes = HashMap::new();
    elo_changes.insert(player1, -50);

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0].source_tournament_elo_change, -50);
    assert_eq!(contributions[0].contribution_amount, -10);
    assert_eq!(adjustments.get(&player2), Some(&-10));
}

#[test]
fn test_no_self_contribution() {
    let player1 = Uuid::new_v4();
    let team = Uuid::new_v4();

    let results = vec![(player1, 1)];

    let mut player_to_team = HashMap::new();
    player_to_team.insert(player1, team);

    let mut team_to_players = HashMap::new();
    team_to_players.insert(team, vec![player1]);

    let mut elo_changes = HashMap::new();
    elo_changes.insert(player1, 50);

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 0);
    assert_eq!(adjustments.len(), 0);
}

#[test]
fn test_multiple_teammates() {
    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();
    let player3 = Uuid::new_v4();
    let team = Uuid::new_v4();

    let results = vec![(player1, 1)];

    let mut player_to_team = HashMap::new();
    player_to_team.insert(player1, team);
    player_to_team.insert(player2, team);
    player_to_team.insert(player3, team);

    let mut team_to_players = HashMap::new();
    team_to_players.insert(team, vec![player1, player2, player3]);

    let mut elo_changes = HashMap::new();
    elo_changes.insert(player1, 100);

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 2);
    assert_eq!(adjustments.get(&player2), Some(&20));
    assert_eq!(adjustments.get(&player3), Some(&20));

    let beneficiaries: Vec<Uuid> = contributions
        .iter()
        .map(|c| c.beneficiary_player_id)
        .collect();
    assert!(beneficiaries.contains(&player2));
    assert!(beneficiaries.contains(&player3));
    assert!(!beneficiaries.contains(&player1));
}

#[test]
fn test_contribution_rounding() {
    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();
    let team = Uuid::new_v4();

    let results = vec![(player1, 1)];

    let mut player_to_team = HashMap::new();
    player_to_team.insert(player1, team);
    player_to_team.insert(player2, team);

    let mut team_to_players = HashMap::new();
    team_to_players.insert(team, vec![player1, player2]);

    let mut elo_changes = HashMap::new();
    elo_changes.insert(player1, 47);

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0].contribution_amount, 9);
    assert_eq!(adjustments.get(&player2), Some(&9));
}

#[test]
fn test_empty_results() {
    let results: Vec<(Uuid, i32)> = vec![];
    let player_to_team = HashMap::new();
    let team_to_players = HashMap::new();
    let elo_changes = HashMap::new();

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 0);
    assert_eq!(adjustments.len(), 0);
}

#[test]
fn test_solo_player_no_teammates() {
    let player1 = Uuid::new_v4();
    let team = Uuid::new_v4();

    let results = vec![(player1, 1)];

    let mut player_to_team = HashMap::new();
    player_to_team.insert(player1, team);

    let mut team_to_players = HashMap::new();
    team_to_players.insert(team, vec![player1]);

    let mut elo_changes = HashMap::new();
    elo_changes.insert(player1, 50);

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 0);
    assert_eq!(adjustments.len(), 0);
}

#[test]
fn test_multiple_source_players_same_team() {
    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();
    let player3 = Uuid::new_v4();
    let team = Uuid::new_v4();

    let results = vec![(player1, 1), (player2, 2)];

    let mut player_to_team = HashMap::new();
    player_to_team.insert(player1, team);
    player_to_team.insert(player2, team);
    player_to_team.insert(player3, team);

    let mut team_to_players = HashMap::new();
    team_to_players.insert(team, vec![player1, player2, player3]);

    let mut elo_changes = HashMap::new();
    elo_changes.insert(player1, 50);
    elo_changes.insert(player2, 30);

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 4);

    assert_eq!(adjustments.get(&player1), Some(&6));
    assert_eq!(adjustments.get(&player2), Some(&10));
    assert_eq!(adjustments.get(&player3), Some(&16));
}

#[test]
fn test_realistic_scenario_teammates_not_in_race() {
    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();
    let player3 = Uuid::new_v4();
    let team = Uuid::new_v4();

    let results = vec![(player1, 1)];

    let mut player_to_team = HashMap::new();
    player_to_team.insert(player1, team);
    player_to_team.insert(player2, team);
    player_to_team.insert(player3, team);

    let mut team_to_players = HashMap::new();
    team_to_players.insert(team, vec![player1, player2, player3]);

    let mut elo_changes = HashMap::new();
    elo_changes.insert(player1, 50);

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 2);

    assert_eq!(contributions[0].source_player_id, player1);
    assert!(
        contributions[0].beneficiary_player_id == player2
            || contributions[0].beneficiary_player_id == player3
    );
    assert_eq!(contributions[0].contribution_amount, 10);

    assert_eq!(contributions[1].source_player_id, player1);
    assert!(
        contributions[1].beneficiary_player_id == player2
            || contributions[1].beneficiary_player_id == player3
    );
    assert_eq!(contributions[1].contribution_amount, 10);

    assert_eq!(adjustments.get(&player1), None);
    assert_eq!(adjustments.get(&player2), Some(&10));
    assert_eq!(adjustments.get(&player3), Some(&10));
}

#[test]
fn test_independent_teams() {
    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();
    let player3 = Uuid::new_v4();
    let player4 = Uuid::new_v4();
    let team1 = Uuid::new_v4();
    let team2 = Uuid::new_v4();

    let results = vec![(player1, 1), (player3, 2)];

    let mut player_to_team = HashMap::new();
    player_to_team.insert(player1, team1);
    player_to_team.insert(player2, team1);
    player_to_team.insert(player3, team2);
    player_to_team.insert(player4, team2);

    let mut team_to_players = HashMap::new();
    team_to_players.insert(team1, vec![player1, player2]);
    team_to_players.insert(team2, vec![player3, player4]);

    let mut elo_changes = HashMap::new();
    elo_changes.insert(player1, 50);
    elo_changes.insert(player3, 30);

    let (contributions, adjustments) =
        calculate_teammate_contributions(&results, &player_to_team, &team_to_players, &elo_changes);

    assert_eq!(contributions.len(), 2);

    assert_eq!(adjustments.get(&player2), Some(&10));
    assert_eq!(adjustments.get(&player4), Some(&6));

    assert_eq!(adjustments.get(&player1), None);
    assert_eq!(adjustments.get(&player3), None);

    let beneficiaries: Vec<Uuid> = contributions
        .iter()
        .map(|c| c.beneficiary_player_id)
        .collect();
    assert!(beneficiaries.contains(&player2));
    assert!(beneficiaries.contains(&player4));
    assert!(!beneficiaries.contains(&player1));
    assert!(!beneficiaries.contains(&player3));
}
