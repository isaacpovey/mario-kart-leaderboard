use crate::db::DbPool;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

const STARTING_TOURNAMENT_ELO: i32 = 1200;

#[derive(Debug, Clone, FromRow)]
pub struct PlayerTournamentScore {
    pub player_id: Uuid,
    pub tournament_id: Uuid,
    pub group_id: Uuid,
    pub elo_rating: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PlayerTournamentScore {
    pub async fn get_or_create_batch(
        tx: &mut Transaction<'_, Postgres>,
        player_ids: &[Uuid],
        tournament_id: Uuid,
        group_id: Uuid,
    ) -> Result<HashMap<Uuid, i32>, sqlx::Error> {
        let existing_scores = sqlx::query_as::<_, (Uuid, i32)>(
            "SELECT player_id, elo_rating FROM player_tournament_scores
             WHERE player_id = ANY($1) AND tournament_id = $2",
        )
        .bind(player_ids)
        .bind(tournament_id)
        .fetch_all(&mut **tx)
        .await?;

        let existing_map: HashMap<Uuid, i32> = existing_scores.into_iter().collect();
        let missing_player_ids: Vec<Uuid> = player_ids
            .iter()
            .filter(|id| !existing_map.contains_key(id))
            .copied()
            .collect();

        if !missing_player_ids.is_empty() {
            let player_values: Vec<(Uuid, Uuid, Uuid, i32)> = missing_player_ids
                .iter()
                .map(|&player_id| {
                    (player_id, tournament_id, group_id, STARTING_TOURNAMENT_ELO)
                })
                .collect();

            let (player_ids_to_insert, tournament_ids_to_insert, group_ids_to_insert, elos): (
                Vec<_>,
                Vec<_>,
                Vec<_>,
                Vec<_>,
            ) = player_values.into_iter().fold(
                (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |(mut pids, mut tids, mut gids, mut elos), (pid, tid, gid, elo)| {
                    pids.push(pid);
                    tids.push(tid);
                    gids.push(gid);
                    elos.push(elo);
                    (pids, tids, gids, elos)
                },
            );

            sqlx::query(
                "INSERT INTO player_tournament_scores (player_id, tournament_id, group_id, elo_rating)
                 SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[], $4::int[])
                 ON CONFLICT (player_id, tournament_id) DO NOTHING",
            )
            .bind(&player_ids_to_insert)
            .bind(&tournament_ids_to_insert)
            .bind(&group_ids_to_insert)
            .bind(&elos)
            .execute(&mut **tx)
            .await?;
        }

        let final_scores = sqlx::query_as::<_, (Uuid, i32)>(
            "SELECT player_id, elo_rating FROM player_tournament_scores
             WHERE player_id = ANY($1) AND tournament_id = $2",
        )
        .bind(player_ids)
        .bind(tournament_id)
        .fetch_all(&mut **tx)
        .await?;

        Ok(final_scores.into_iter().collect())
    }

    pub async fn update_elo_batch(
        tx: &mut Transaction<'_, Postgres>,
        updates: &[(Uuid, Uuid, i32)],
    ) -> Result<(), sqlx::Error> {
        let (player_ids, tournament_ids, new_elos): (Vec<_>, Vec<_>, Vec<_>) =
            updates.iter().fold(
                (Vec::new(), Vec::new(), Vec::new()),
                |(mut pids, mut tids, mut elos), (pid, tid, elo)| {
                    pids.push(*pid);
                    tids.push(*tid);
                    elos.push(*elo);
                    (pids, tids, elos)
                },
            );

        sqlx::query(
            "UPDATE player_tournament_scores pts
             SET elo_rating = u.new_elo, updated_at = NOW()
             FROM UNNEST($1::uuid[], $2::uuid[], $3::int[]) AS u(player_id, tournament_id, new_elo)
             WHERE pts.player_id = u.player_id AND pts.tournament_id = u.tournament_id",
        )
        .bind(&player_ids)
        .bind(&tournament_ids)
        .bind(&new_elos)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_tournament_leaderboard(
        pool: &DbPool,
        tournament_id: Uuid,
    ) -> Result<Vec<(Uuid, String, i32, i32, Option<String>)>, sqlx::Error> {
        sqlx::query_as::<_, (Uuid, String, i32, i32, Option<String>)>(
            "SELECT p.id, p.name, pts.elo_rating, p.elo_rating, p.avatar_filename
             FROM player_tournament_scores pts
             JOIN players p ON p.id = pts.player_id
             WHERE pts.tournament_id = $1
             ORDER BY pts.elo_rating DESC",
        )
        .bind(tournament_id)
        .fetch_all(pool)
        .await
    }
}
