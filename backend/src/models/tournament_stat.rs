use crate::db::DbPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, Transaction, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "tournament_stat_type", rename_all = "snake_case")]
pub enum TournamentStatType {
    BestTeammate,
    WorstTeammate,
    BestRace,
    WorstRace,
    BiggestSwing,
    MostHelped,
    MostHurt,
    BestMatch,
    WorstMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiggestSwingData {
    pub high_value: i32,
    pub low_value: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct TournamentStat {
    pub id: Uuid,
    pub tournament_id: Uuid,
    pub stat_type: TournamentStatType,
    pub player_id: Uuid,
    pub value: i32,
    pub extra_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl TournamentStat {
    pub async fn find_by_tournament_id(
        pool: &DbPool,
        tournament_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, tournament_id, stat_type, player_id, value, extra_data, created_at
             FROM tournament_stats
             WHERE tournament_id = $1",
        )
        .bind(tournament_id)
        .fetch_all(pool)
        .await
    }

    pub async fn insert_batch(
        tx: &mut Transaction<'_, Postgres>,
        stats: &[(Uuid, TournamentStatType, Uuid, i32, Option<serde_json::Value>)],
    ) -> Result<(), sqlx::Error> {
        if stats.is_empty() {
            return Ok(());
        }

        let (tournament_ids, stat_types, player_ids, values, extra_datas): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = stats.iter().fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            |(mut tids, mut stypes, mut pids, mut vals, mut extras),
             (tid, stype, pid, val, extra)| {
                tids.push(*tid);
                stypes.push(*stype);
                pids.push(*pid);
                vals.push(*val);
                extras.push(extra.clone());
                (tids, stypes, pids, vals, extras)
            },
        );

        sqlx::query(
            "INSERT INTO tournament_stats (tournament_id, stat_type, player_id, value, extra_data)
             SELECT * FROM UNNEST($1::uuid[], $2::tournament_stat_type[], $3::uuid[], $4::int[], $5::jsonb[])",
        )
        .bind(&tournament_ids)
        .bind(&stat_types)
        .bind(&player_ids)
        .bind(&values)
        .bind(&extra_datas)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
