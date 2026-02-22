use crate::db::DbPool;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PlayerRaceScore {
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub round_number: i32,
    pub player_id: Uuid,
    pub position: i32,
    pub all_time_elo_change: Option<i32>,
    pub all_time_elo_after: Option<i32>,
    pub tournament_elo_change: Option<i32>,
    pub tournament_elo_after: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PlayerTrackAggregation {
    pub track_name: String,
    pub average_position: f64,
    pub races_played: i64,
}

impl PlayerRaceScore {
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_match_id(
        pool: &DbPool,
        match_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT group_id, match_id, round_number, player_id, position,
                    all_time_elo_change, all_time_elo_after,
                    tournament_elo_change, tournament_elo_after, created_at
             FROM player_race_scores
             WHERE match_id = $1
             ORDER BY round_number ASC, position ASC",
        )
        .bind(match_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_match_and_round(
        pool: &DbPool,
        match_id: Uuid,
        round_number: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT group_id, match_id, round_number, player_id, position,
                    all_time_elo_change, all_time_elo_after,
                    tournament_elo_change, tournament_elo_after, created_at
             FROM player_race_scores
             WHERE match_id = $1 AND round_number = $2
             ORDER BY position ASC",
        )
        .bind(match_id)
        .bind(round_number)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = rounds.len()))]
    pub async fn find_by_rounds(
        pool: &DbPool,
        rounds: &[(Uuid, i32)],
    ) -> Result<Vec<Self>, sqlx::Error> {
        let match_ids: Vec<Uuid> = rounds.iter().map(|(match_id, _)| *match_id).collect();
        let round_numbers: Vec<i32> = rounds.iter().map(|(_, round_num)| *round_num).collect();

        sqlx::query_as::<_, Self>(
            "SELECT group_id, match_id, round_number, player_id, position,
                    all_time_elo_change, all_time_elo_after,
                    tournament_elo_change, tournament_elo_after, created_at
             FROM player_race_scores
             WHERE (match_id, round_number) IN (SELECT UNNEST($1::uuid[]), UNNEST($2::int[]))
             ORDER BY match_id, round_number, position ASC",
        )
        .bind(&match_ids)
        .bind(&round_numbers)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn find_elo_history_by_tournament_id(
        pool: &DbPool,
        tournament_id: Uuid,
    ) -> Result<Vec<(Uuid, String, i32, String)>, sqlx::Error> {
        sqlx::query_as::<_, (Uuid, String, i32, String)>(
            "SELECT
                prs.player_id,
                p.name as player_name,
                prs.tournament_elo_after,
                prs.created_at::text as timestamp
             FROM player_race_scores prs
             INNER JOIN matches m ON prs.match_id = m.id
             INNER JOIN players p ON prs.player_id = p.id
             WHERE m.tournament_id = $1
               AND prs.tournament_elo_after IS NOT NULL
             ORDER BY prs.created_at ASC",
        )
        .bind(tournament_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn find_track_stats_by_player(
        pool: &DbPool,
        player_id: Uuid,
    ) -> Result<Vec<PlayerTrackAggregation>, sqlx::Error> {
        sqlx::query_as::<_, PlayerTrackAggregation>(
            "SELECT
                t.name AS track_name,
                AVG(prs.position)::float8 AS average_position,
                COUNT(*)::bigint AS races_played
             FROM player_race_scores prs
             INNER JOIN rounds r ON prs.match_id = r.match_id AND prs.round_number = r.round_number
             INNER JOIN tracks t ON r.track_id = t.id
             WHERE prs.player_id = $1
             GROUP BY t.id, t.name
             ORDER BY average_position ASC",
        )
        .bind(player_id)
        .fetch_all(pool)
        .await
    }
}
