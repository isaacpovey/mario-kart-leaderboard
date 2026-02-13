
//! Track Selection Service
//!
//! Selects tracks using a "shuffle bag" pattern: all tracks are played in random
//! order before any track repeats. With N total tracks and R rounds per match,
//! one full cycle = ceil(N/R) matches.
//!
//! Uses cycle-position-based selection to avoid the sliding window bug where
//! tracks could repeat within a cycle at boundary crossings.

use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use uuid::Uuid;

pub async fn select_tracks(
    pool: &DbPool,
    tournament_id: Uuid,
    num_races: i32,
) -> Result<Vec<models::Track>> {
    let all_tracks = models::Track::find_all(pool).await?;
    let total_track_count = all_tracks.len();
    let num_races = num_races as usize;

    let total_rounds_played: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM rounds r
         JOIN matches m ON m.id = r.match_id
         WHERE m.tournament_id = $1 AND r.track_id IS NOT NULL",
    )
    .bind(tournament_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to count rounds: {e}")))?;

    let cycle_position = (total_rounds_played as usize) % total_track_count;

    let current_cycle_track_ids: HashSet<Uuid> = if cycle_position == 0 {
        HashSet::new()
    } else {
        sqlx::query_scalar(
            "SELECT r.track_id
             FROM rounds r
             JOIN matches m ON m.id = r.match_id
             WHERE m.tournament_id = $1 AND r.track_id IS NOT NULL
             ORDER BY m.time DESC, r.round_number DESC
             LIMIT $2",
        )
        .bind(tournament_id)
        .bind(cycle_position as i32)
        .fetch_all(pool)
        .await?
        .into_iter()
        .collect()
    };

    let (available_tracks, used_tracks): (Vec<models::Track>, Vec<models::Track>) = all_tracks
        .into_iter()
        .partition(|track| !current_cycle_track_ids.contains(&track.id));

    let mut rng = rand::rng();

    if available_tracks.len() >= num_races {
        let mut track_pool = available_tracks;
        track_pool.shuffle(&mut rng);
        Ok(track_pool.into_iter().take(num_races).collect())
    } else {
        // Cycle boundary: take all remaining, then draw from new cycle
        let mut selected = available_tracks;
        let remaining_needed = num_races - selected.len();

        let mut new_cycle_pool = used_tracks;
        new_cycle_pool.shuffle(&mut rng);
        selected.extend(new_cycle_pool.into_iter().take(remaining_needed));

        selected.shuffle(&mut rng);
        Ok(selected)
    }
}
