use crate::db::DbPool;
use chrono::NaiveDate;
use sqlx::{FromRow, Postgres, Transaction};
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Tournament {
    pub id: Uuid,
    pub group_id: Uuid,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub winner: Option<Uuid>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CompletedTournamentRow {
    pub id: Uuid,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub winner: Option<Uuid>,
    pub winner_name: Option<String>,
    pub winner_avatar_filename: Option<String>,
    pub participant_count: i64,
}

impl Tournament {
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, start_date, end_date, winner FROM tournaments WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = ids.len()))]
    pub async fn find_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, start_date, end_date, winner FROM tournaments WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_group_id(
        pool: &DbPool,
        group_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, start_date, end_date, winner
             FROM tournaments
             WHERE group_id = $1
             ORDER BY start_date DESC NULLS LAST",
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn count_completed_by_group_id(
        pool: &DbPool,
        group_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM tournaments WHERE group_id = $1 AND winner IS NOT NULL",
        )
        .bind(group_id)
        .fetch_one(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn find_completed_by_group_id_paginated(
        pool: &DbPool,
        group_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CompletedTournamentRow>, sqlx::Error> {
        sqlx::query_as::<_, CompletedTournamentRow>(
            "SELECT t.id, t.start_date, t.end_date, t.winner,
                    p.name AS winner_name, p.avatar_filename AS winner_avatar_filename,
                    (SELECT COUNT(*)::bigint FROM player_tournament_scores pts
                     WHERE pts.tournament_id = t.id) AS participant_count
             FROM tournaments t
             LEFT JOIN players p ON p.id = t.winner
             WHERE t.group_id = $1 AND t.winner IS NOT NULL
             ORDER BY t.start_date DESC NULLS LAST, t.end_date DESC NULLS LAST
             LIMIT $2 OFFSET $3",
        )
        .bind(group_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn create(
        pool: &DbPool,
        group_id: Uuid,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO tournaments (group_id, start_date, end_date)
             VALUES ($1, $2, $3)
             RETURNING id, group_id, start_date, end_date, winner",
        )
        .bind(group_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn get_active_tournament(
        pool: &DbPool,
        group_id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id
             FROM tournaments
             WHERE group_id = $1 AND winner IS NULL
             ORDER BY start_date DESC NULLS LAST
             LIMIT 1",
        )
        .bind(group_id)
        .fetch_optional(pool)
        .await
    }

    #[instrument(level = "debug", skip(tx))]
    pub async fn set_winner(
        tx: &mut Transaction<'_, Postgres>,
        tournament_id: Uuid,
        winner_id: Uuid,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "UPDATE tournaments
             SET winner = $1
             WHERE id = $2
             RETURNING id, group_id, start_date, end_date, winner",
        )
        .bind(winner_id)
        .bind(tournament_id)
        .fetch_one(&mut **tx)
        .await
    }
}
