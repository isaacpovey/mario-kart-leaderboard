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

impl PlayerRaceScore {
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_match_id(
        pool: &sqlx::PgPool,
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
        pool: &sqlx::PgPool,
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
        pool: &sqlx::PgPool,
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
}
