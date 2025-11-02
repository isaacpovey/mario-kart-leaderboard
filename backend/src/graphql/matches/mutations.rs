use crate::graphql::context::GraphQLContext;
use crate::graphql::matches::types::Match;
use crate::models;
use async_graphql::*;
use chrono::Utc;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const DEFAULT_PLAYERS_PER_RACE: i32 = 4;

#[derive(Default)]
pub struct MatchesMutation;

#[Object]
impl MatchesMutation {
    async fn create_match_with_rounds(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The tournament ID")] tournament_id: ID,
        #[graphql(desc = "The player IDs participating in this match")] player_ids: Vec<ID>,
        #[graphql(desc = "The number of races")]
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
            &teams,
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
    if total_slots < num_players {
        return Err(Error::new(format!(
            "Invalid configuration: {num_races} races with {players_per_race} players per race gives {total_slots} total slots, which is less than {num_players} players. Each player must be able to participate in at least one race."
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

fn calculate_team_sizes(num_players: usize, num_teams: usize) -> Vec<usize> {
    let base_size = num_players / num_teams;
    let remainder = num_players % num_teams;

    (0..num_teams)
        .map(|team_idx| {
            if team_idx < remainder {
                base_size + 1
            } else {
                base_size
            }
        })
        .collect()
}

fn allocate_teams(players: &[models::Player], players_per_race: &i32) -> Vec<Team> {
    let mut sorted_players = players.to_vec();
    sorted_players.sort_by(|a, b| b.elo_rating.cmp(&a.elo_rating));

    let num_players = players.len();
    let num_teams = std::cmp::min(*players_per_race as usize, num_players);
    let team_sizes = calculate_team_sizes(num_players, num_teams);

    let initial_teams: Vec<Team> = (0..num_teams)
        .map(|team_idx| Team {
            team_num: (team_idx + 1) as i32,
            players: Vec::new(),
            total_elo: 0,
        })
        .collect();

    sorted_players.into_iter().fold(initial_teams, |teams, player| {
        let team_idx = teams
            .iter()
            .enumerate()
            .filter(|(idx, team)| team.players.len() < team_sizes[*idx])
            .min_by_key(|(_, team)| team.total_elo)
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        teams
            .into_iter()
            .enumerate()
            .map(|(idx, team)| {
                if idx == team_idx {
                    Team {
                        team_num: team.team_num,
                        players: team.players.iter().cloned().chain(std::iter::once(player.clone())).collect(),
                        total_elo: team.total_elo + player.elo_rating,
                    }
                } else {
                    team
                }
            })
            .collect()
    })
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

fn allocate_races(
    players: &[models::Player],
    teams: &[Team],
    num_races: i32,
    _players_per_race: i32,
) -> Result<Vec<RaceAllocation>> {
    let total_slots = num_races * teams.len() as i32;
    let num_players = players.len();

    let avg_team_size: f64 = teams.iter().map(|t| t.players.len()).sum::<usize>() as f64 / teams.len() as f64;
    let base_races_per_player = total_slots as f64 / num_players as f64;

    let mut team_state: Vec<(i32, Vec<Uuid>, HashMap<Uuid, i32>)> = teams
        .iter()
        .map(|team| {
            let team_size = team.players.len() as f64;
            let races_per_player = (base_races_per_player * (avg_team_size / team_size)).round() as i32;

            let player_races: HashMap<Uuid, i32> = team.players
                .iter()
                .map(|p| (p.id, races_per_player))
                .collect();

            let player_ids: Vec<Uuid> = team.players.iter().map(|p| p.id).collect();

            (team.team_num, player_ids, player_races)
        })
        .collect();

    let mut allocations = Vec::new();

    for race_num in 0..num_races {
        let previous_race_players: HashSet<Uuid> = if race_num > 0 {
            allocations
                .get((race_num - 1) as usize)
                .map(|alloc: &RaceAllocation| alloc.player_ids.iter().copied().collect())
                .unwrap_or_default()
        } else {
            HashSet::new()
        };

        let mut race_players = Vec::new();

        for (_team_num, team_player_ids, player_races) in team_state.iter_mut() {
            let available: Vec<&Uuid> = team_player_ids
                .iter()
                .filter(|pid| *player_races.get(pid).unwrap_or(&0) > 0)
                .collect();

            let selected_player = if available.is_empty() {
                team_player_ids.first().unwrap()
            } else if available.len() == 1 {
                available[0]
            } else {
                *available
                    .iter()
                    .find(|pid| !previous_race_players.contains(**pid))
                    .unwrap_or(&available[0])
            };

            race_players.push(*selected_player);

            if let Some(count) = player_races.get_mut(selected_player) {
                *count = (*count - 1).max(0);
            }
        }

        allocations.push(RaceAllocation {
            race_number: race_num + 1,
            player_ids: race_players,
        });
    }

    Ok(allocations)
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
