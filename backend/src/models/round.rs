use crate::db::DbPool;
use sqlx::FromRow;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

use super::{PlayerRaceScore, Track};

#[derive(Debug, Clone, FromRow)]
pub struct Round {
    pub match_id: Uuid,
    pub round_number: i32,
    pub track_id: Option<Uuid>,
    pub completed: bool,
}

impl Round {
    pub async fn find_one(
        pool: &DbPool,
        match_id: Uuid,
        round_number: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT match_id, round_number, track_id, completed
             FROM rounds
             WHERE match_id = $1 AND round_number = $2",
        )
        .bind(match_id)
        .bind(round_number)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_match_id(
        pool: &DbPool,
        match_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT match_id, round_number, track_id, completed
             FROM rounds
             WHERE match_id = $1
             ORDER BY round_number ASC",
        )
        .bind(match_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn get_by_match_with_tracks_and_results(
        pool: &DbPool,
        match_id: Uuid,
    ) -> Result<Vec<RoundWithTracksAndResults>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (
            Uuid, i32, Option<Uuid>, bool,
            Option<Uuid>, Option<String>,
            Option<Uuid>, Option<i32>, Option<i32>, Option<i32>,
            Option<Uuid>,
        )>(
            "SELECT
                r.match_id, r.round_number, r.track_id, r.completed,
                t.id as track_db_id, t.name as track_name,
                prs.player_id as result_player_id,
                prs.position as result_position,
                prs.all_time_elo_change,
                prs.tournament_elo_change,
                rp.player_id as round_player_id
             FROM rounds r
             LEFT JOIN tracks t ON t.id = r.track_id
             LEFT JOIN player_race_scores prs
               ON prs.match_id = r.match_id
               AND prs.round_number = r.round_number
             LEFT JOIN round_players rp
               ON rp.match_id = r.match_id
               AND rp.round_number = r.round_number
             WHERE r.match_id = $1
             ORDER BY r.round_number ASC, prs.position ASC, rp.player_id ASC",
        )
        .bind(match_id)
        .fetch_all(pool)
        .await?;

        let grouped = rows.into_iter().fold(
            HashMap::<i32, RoundWithTracksAndResults>::new(),
            |mut acc, (match_id, round_number, track_id, completed,
                       opt_track_id, opt_track_name,
                       opt_player_id, opt_position, opt_all_time_elo, opt_tournament_elo,
                       opt_round_player_id)| {

                let entry = acc.entry(round_number).or_insert_with(|| {
                    RoundWithTracksAndResults {
                        round: Round { match_id, round_number, track_id, completed },
                        track: opt_track_id.zip(opt_track_name).map(|(id, name)| Track { id, name }),
                        result_player_ids: Vec::new(),
                        results: Vec::new(),
                    }
                });

                if let Some(player_id) = opt_player_id {
                    if !entry.result_player_ids.contains(&player_id) {
                        entry.result_player_ids.push(player_id);
                    }
                    entry.results.push(PlayerRaceScore {
                        group_id: Uuid::nil(),
                        match_id,
                        round_number,
                        player_id,
                        position: opt_position.unwrap(),
                        all_time_elo_change: opt_all_time_elo,
                        all_time_elo_after: None,
                        tournament_elo_change: opt_tournament_elo,
                        tournament_elo_after: None,
                        created_at: chrono::Utc::now(),
                    });
                } else if let Some(round_player_id) = opt_round_player_id {
                    if !entry.result_player_ids.contains(&round_player_id) {
                        entry.result_player_ids.push(round_player_id);
                    }
                }

                acc
            },
        );

        let mut result: Vec<RoundWithTracksAndResults> = grouped.into_values().collect();
        result.sort_by_key(|r| r.round.round_number);

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct RoundWithTracksAndResults {
    pub round: Round,
    pub track: Option<Track>,
    pub result_player_ids: Vec<Uuid>,
    pub results: Vec<PlayerRaceScore>,
}
