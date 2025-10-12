use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use async_graphql::*;
use chrono::Utc;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const DEFAULT_PLAYERS_PER_RACE: i32 = 4;
const CONSECUTIVE_RACE_PENALTY: i32 = 1000;

#[derive(Default)]
pub struct MatchesMutation;

#[Object]
impl MatchesMutation {
    async fn create_match_with_rounds(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The tournament ID")] tournament_id: ID,
        #[graphql(desc = "The player IDs participating in this match")] player_ids: Vec<ID>,
        #[graphql(desc = "The number of races (num_races * players_per_race must be divisible by number of players)")]
        num_races: i32,
        #[graphql(desc = "The number of players per race (default: 4)")]
        players_per_race: Option<i32>,
    ) -> Result<Match> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;
        
        let tournament_uuid =
            Uuid::parse_str(&tournament_id).map_err(|_| Error::new("Invalid tournament ID"))?;

        let player_uuids: Result<Vec<Uuid>> = player_ids
            .iter()
            .map(|id| Uuid::parse_str(id).map_err(|_| Error::new("Invalid player ID")))
            .collect();
        let player_uuids = player_uuids?;

        let players_per_race = players_per_race.unwrap_or(DEFAULT_PLAYERS_PER_RACE);
        
        validate_create_match_inputs(
            &player_uuids,
            num_races,
            players_per_race,
        )?;

        let tournament = models::Tournament::find_by_id(&gql_ctx.pool, tournament_uuid)
            .await?
            .ok_or_else(|| Error::new("Tournament not found"))?;

        if tournament.group_id != group_id {
            return Err(Error::new("Tournament does not belong to your group"));
        }

        let players = models::Player::find_by_ids(&gql_ctx.pool, &player_uuids).await?;

        if players.len() != player_uuids.len() || players.iter().any(|p| p.group_id != group_id) {
            return Err(Error::new("One or more players not found"));
        }


        let teams = allocate_teams(&players, &players_per_race);
        
        let tracks = select_tracks(&gql_ctx.pool, tournament_uuid, num_races).await?;
        
        let race_allocations = allocate_races(
            &players,
            num_races,
            players_per_race,
        )?;
        
        let match_result = create_match_in_transaction(
            &gql_ctx.pool,
            group_id,
            tournament_uuid,
            num_races,
            &teams,
            &tracks,
            &race_allocations,
        )
        .await?;

        Ok(Match::from(match_result))
    }
}

fn validate_create_match_inputs(
    player_uuids: &[Uuid],
    num_races: i32,
    players_per_race: i32,
) -> Result<()> {
    let num_players = player_uuids.len() as i32;

    if player_uuids.is_empty() {
        return Err(Error::new("At least one player is required"));
    }

    if num_races <= 0 {
        return Err(Error::new("Number of races must be positive"));
    }

    if players_per_race <= 0 {
        return Err(Error::new("Players per race must be positive"));
    }

    if players_per_race > num_players {
        return Err(Error::new("Players per race cannot exceed total number of players"));
    }

    let total_slots = num_races * players_per_race;
    if total_slots % num_players != 0 {
        let races_per_player = total_slots as f64 / num_players as f64;
        return Err(Error::new(format!(
            "Invalid configuration: {num_races} races with {players_per_race} players per race gives {total_slots} total slots, which cannot be evenly divided among {num_players} players (would be {races_per_player:.2} races per player). Total slots must be divisible by number of players."
        )));
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct Team {
    team_num: i32,
    players: Vec<models::Player>,
    #[allow(dead_code)]
    total_elo: i32,
}

fn allocate_teams(players: &[models::Player], players_per_race: &i32) -> Vec<Team> {
    let mut sorted_players = players.to_vec();
    sorted_players.sort_by(|a, b| b.elo_rating.cmp(&a.elo_rating));

    let num_players = players.len();
    let num_teams = std::cmp::min(*players_per_race as usize, num_players);

    let player_team_assignments: Vec<(usize, models::Player)> = sorted_players
        .into_iter()
        .enumerate()
        .map(|(player_idx, player)| (snake_draft_index(player_idx, num_teams), player))
        .collect();

    (0..num_teams)
        .map(|team_idx| {
            let team_players: Vec<models::Player> = player_team_assignments
                .iter()
                .filter(|(assigned_team_idx, _)| *assigned_team_idx == team_idx)
                .map(|(_, player)| player.clone())
                .collect();

            let total_elo = team_players.iter().map(|p| p.elo_rating).sum();

            Team {
                team_num: (team_idx + 1) as i32,
                players: team_players,
                total_elo,
            }
        })
        .collect()
}

fn snake_draft_index(pick_number: usize, num_teams: usize) -> usize {
    let round = pick_number / num_teams;
    let position = pick_number % num_teams;

    if round % 2 == 0 {
        position
    } else {
        num_teams - 1 - position
    }
}

async fn select_tracks(
    pool: &sqlx::PgPool,
    tournament_id: Uuid,
    num_races: i32,
) -> Result<Vec<models::Track>> {
    let all_tracks = models::Track::find_all(pool).await?;
    let total_track_count = all_tracks.len();
    
    let recently_used_track_ids: HashSet<Uuid> = sqlx::query_scalar(
        "SELECT r.track_id
         FROM rounds r
         JOIN matches m ON m.id = r.match_id
         WHERE m.tournament_id = $1 AND r.track_id IS NOT NULL
         ORDER BY m.time DESC, r.round_number DESC
         LIMIT $2",
    )
    .bind(tournament_id)
    .bind(total_track_count as i32)
    .fetch_all(pool)
    .await?
    .into_iter()
    .collect();
    
    let (available_tracks, recently_used_tracks): (Vec<models::Track>, Vec<models::Track>) = all_tracks
        .into_iter()
        .partition(|track| !recently_used_track_ids.contains(&track.id));
    
    let track_pool = if available_tracks.len() >= num_races as usize {
        available_tracks
    } else {
        available_tracks
            .into_iter()
            .chain(recently_used_tracks.into_iter())
            .collect()
    };

    let mut track_pool = track_pool;
    track_pool.shuffle(&mut rand::rng());
    let selected: Vec<models::Track> = track_pool
        .into_iter()
        .take(num_races as usize)
        .collect();

    Ok(selected)
}

#[derive(Debug, Clone)]
struct RaceAllocation {
    race_number: i32,
    player_ids: Vec<Uuid>,
}

#[derive(Debug, Clone)]
struct AllocationState {
    races_remaining: HashMap<Uuid, i32>,
    allocations: Vec<RaceAllocation>,
}

fn allocate_races(
    players: &[models::Player],
    num_races: i32,
    players_per_race: i32,
) -> Result<Vec<RaceAllocation>> {
    let total_slots = num_races * players_per_race;
    let races_per_player = total_slots / players.len() as i32;
    
    let initial_state = AllocationState {
        races_remaining: players
            .iter()
            .map(|p| (p.id, races_per_player))
            .collect(),
        allocations: Vec::new(),
    };
    
    let final_state = (0..num_races).fold(initial_state, |state, race_num| {
        allocate_single_race(state, race_num, players, players_per_race)
    });

    Ok(final_state.allocations)
}

fn allocate_single_race(
    state: AllocationState,
    race_num: i32,
    all_players: &[models::Player],
    players_per_race: i32,
) -> AllocationState {
    let available_players: Vec<&models::Player> = all_players
        .iter()
        .filter(|p| *state.races_remaining.get(&p.id).unwrap_or(&0) > 0)
        .collect();
    
    let previous_race_players: HashSet<Uuid> = if race_num > 0 {
        state.allocations
            .get((race_num - 1) as usize)
            .map(|alloc| alloc.player_ids.iter().copied().collect())
            .unwrap_or_default()
    } else {
        HashSet::new()
    };
    
    let selected_players = select_best_players_for_race(
        &available_players,
        players_per_race as usize,
        &previous_race_players,
    );
    
    let new_allocations = state
        .allocations
        .iter()
        .cloned()
        .chain(std::iter::once(RaceAllocation {
            race_number: race_num + 1,
            player_ids: selected_players.iter().map(|p| p.id).collect(),
        }))
        .collect();

    let selected_player_ids: HashSet<Uuid> = selected_players.iter().map(|p| p.id).collect();
    let new_races_remaining = state.races_remaining
        .into_iter()
        .map(|(player_id, remaining_races)| {
            if selected_player_ids.contains(&player_id) {
                (player_id, remaining_races - 1)
            } else {
                (player_id, remaining_races)
            }
        })
        .collect();

    AllocationState {
        races_remaining: new_races_remaining,
        allocations: new_allocations,
    }
}

fn select_best_players_for_race(
    available_players: &[&models::Player],
    num_players: usize,
    previous_race_players: &HashSet<Uuid>,
) -> Vec<models::Player> {
    if available_players.len() <= num_players {
        return available_players.iter().map(|&p| p.clone()).collect();
    }
    
    let combinations = generate_combinations(available_players, num_players);
    
    let best_combo = combinations
        .into_iter()
        .min_by_key(|combo| {
            let players: Vec<models::Player> = combo.iter().map(|&p| p.clone()).collect();
            score_combination(&players, previous_race_players)
        })
        .unwrap_or_else(|| {
            available_players.iter().take(num_players).copied().collect()
        });
    
    best_combo.iter().map(|&p| p.clone()).collect()
}

fn score_combination(players: &[models::Player], previous_race_players: &HashSet<Uuid>) -> i32 {
    let elos: Vec<i32> = players.iter().map(|p| p.elo_rating).collect();
    let mean_elo = elos.iter().sum::<i32>() as f64 / elos.len() as f64;
    let variance = elos
        .iter()
        .map(|&elo| {
            let diff = elo as f64 - mean_elo;
            (diff * diff) as i32
        })
        .sum::<i32>();
    
    let consecutive_count = players
        .iter()
        .filter(|p| previous_race_players.contains(&p.id))
        .count() as i32;

    variance + (consecutive_count * CONSECUTIVE_RACE_PENALTY)
}

fn generate_combinations<T: Clone>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.is_empty() {
        return vec![];
    }

    let first = &items[0];
    let rest = &items[1..];
    
    let with_first: Vec<Vec<T>> = generate_combinations(rest, k - 1)
        .into_iter()
        .map(|combo| {
            std::iter::once(first.clone())
                .chain(combo)
                .collect()
        })
        .collect();
    
    let without_first = generate_combinations(rest, k);
    
    with_first
        .into_iter()
        .chain(without_first)
        .collect()
}

async fn create_match_in_transaction(
    pool: &sqlx::PgPool,
    group_id: Uuid,
    tournament_id: Uuid,
    num_races: i32,
    teams: &[Team],
    tracks: &[models::Track],
    race_allocations: &[RaceAllocation],
) -> Result<models::Match> {
    let mut tx = pool.begin().await?;

    // Insert match
    let match_record = sqlx::query_as::<_, models::Match>(
        "INSERT INTO matches (group_id, tournament_id, time, rounds, completed)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, group_id, tournament_id, time, rounds, completed",
    )
    .bind(group_id)
    .bind(tournament_id)
    .bind(Utc::now())
    .bind(num_races)
    .bind(false)
    .fetch_one(&mut *tx)
    .await?;


    let team_id_pairs: Vec<(i32, Uuid)> = {
        let mut pairs = Vec::with_capacity(teams.len());
        for team in teams {
            let (team_id,): (Uuid,) = sqlx::query_as(
                "INSERT INTO teams (group_id, match_id, team_num)
                 VALUES ($1, $2, $3)
                 RETURNING id",
            )
            .bind(group_id)
            .bind(match_record.id)
            .bind(team.team_num)
            .fetch_one(&mut *tx)
            .await?;

            pairs.push((team.team_num, team_id));
        }
        pairs
    };
    let team_id_map: HashMap<i32, Uuid> = team_id_pairs.into_iter().collect();
    
    for team in teams {
        let team_id = team_id_map
            .get(&team.team_num)
            .ok_or_else(|| Error::new("Team ID not found in map"))?;
        for (rank, player) in team.players.iter().enumerate() {
            sqlx::query(
                "INSERT INTO team_players (group_id, team_id, player_id, rank)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(group_id)
            .bind(team_id)
            .bind(player.id)
            .bind((rank + 1) as i32)
            .execute(&mut *tx)
            .await?;
        }
    }
    
    let player_team_map: HashMap<Uuid, Uuid> = teams
        .iter()
        .filter_map(|team| {
            team_id_map.get(&team.team_num).map(|&team_id| {
                team.players.iter().map(move |p| (p.id, team_id)).collect::<Vec<_>>()
            })
        })
        .flatten()
        .collect();
    
    for (idx, track) in tracks.iter().enumerate() {
        sqlx::query(
            "INSERT INTO rounds (match_id, round_number, track_id)
             VALUES ($1, $2, $3)",
        )
        .bind(match_record.id)
        .bind((idx + 1) as i32)
        .bind(track.id)
        .execute(&mut *tx)
        .await?;
    }
    
    for allocation in race_allocations {
        for (position, player_id) in allocation.player_ids.iter().enumerate() {
            let team_id = player_team_map
                .get(player_id)
                .ok_or_else(|| Error::new("Player team mapping not found"))?;
            sqlx::query(
                "INSERT INTO round_players (group_id, match_id, round_number, player_id, team_id, player_position)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(group_id)
            .bind(match_record.id)
            .bind(allocation.race_number)
            .bind(player_id)
            .bind(team_id)
            .bind((position + 1) as i32)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(match_record)
}
