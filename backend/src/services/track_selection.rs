//! Track Selection Service
//!
//! This module provides track selection algorithms that avoid recently used tracks
//! when possible. The algorithm ensures:
//! - Tracks are selected randomly for variety
//! - Recently used tracks are avoided when sufficient alternatives exist
//! - If insufficient fresh tracks exist, recently used tracks are included
//!
//! ## Algorithm
//!
//! 1. Fetch all available tracks
//! 2. Query recently used tracks from previous matches in the tournament
//! 3. Partition tracks into available (not recently used) and recently used
//! 4. If enough available tracks exist, use only those; otherwise mix both pools
//! 5. Shuffle and select the required number of tracks

use crate::error::Result;
use crate::models;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use uuid::Uuid;

/// Selects tracks for a match, avoiding recently used tracks when possible.
///
/// This function implements an intelligent track selection algorithm that:
/// - Queries all available tracks from the database
/// - Identifies recently used tracks in the tournament
/// - Prefers tracks that haven't been used recently
/// - Falls back to including recently used tracks if needed
/// - Randomly shuffles the pool and selects the required number
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `tournament_id` - UUID of the tournament
/// * `num_races` - Number of tracks to select
///
/// # Returns
///
/// Result containing a vector of selected tracks
///
/// # Errors
///
/// Returns an error if:
/// - Database queries fail
/// - No tracks are available
///
/// # Examples
///
/// ```ignore
/// let pool = create_pool(&config).await?;
/// let tournament_id = Uuid::new_v4();
/// let tracks = select_tracks(&pool, tournament_id, 4).await?;
/// assert_eq!(tracks.len(), 4);
/// ```
pub async fn select_tracks(
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

    let (available_tracks, recently_used_tracks): (Vec<models::Track>, Vec<models::Track>) =
        all_tracks
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
    let selected: Vec<models::Track> = track_pool.into_iter().take(num_races as usize).collect();

    Ok(selected)
}
