use chrono::{DateTime, Utc};
use sqlx::{FromRow, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PlayerTeammateEloContribution {
    pub match_id: Uuid,
    pub round_number: i32,
    pub source_player_id: Uuid,
    pub beneficiary_player_id: Uuid,
    pub source_tournament_elo_change: i32,
    pub contribution_amount: i32,
    pub created_at: DateTime<Utc>,
}

impl PlayerTeammateEloContribution {
    pub async fn insert_contributions_batch(
        tx: &mut Transaction<'_, Postgres>,
        contributions: &[(Uuid, i32, Uuid, Uuid, i32, i32)],
    ) -> Result<(), sqlx::Error> {
        if contributions.is_empty() {
            return Ok(());
        }

        let (match_ids, round_numbers, source_player_ids, beneficiary_player_ids, source_changes, contribution_amounts): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = contributions.iter().fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            |(mut mids, mut rnums, mut spids, mut bpids, mut schanges, mut camounts),
             (mid, rnum, spid, bpid, schange, camount)| {
                mids.push(*mid);
                rnums.push(*rnum);
                spids.push(*spid);
                bpids.push(*bpid);
                schanges.push(*schange);
                camounts.push(*camount);
                (mids, rnums, spids, bpids, schanges, camounts)
            },
        );

        sqlx::query(
            "INSERT INTO player_teammate_elo_contributions
             (match_id, round_number, source_player_id, beneficiary_player_id, source_tournament_elo_change, contribution_amount)
             SELECT * FROM UNNEST($1::uuid[], $2::int[], $3::uuid[], $4::uuid[], $5::int[], $6::int[])",
        )
        .bind(&match_ids)
        .bind(&round_numbers)
        .bind(&source_player_ids)
        .bind(&beneficiary_player_ids)
        .bind(&source_changes)
        .bind(&contribution_amounts)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_match_total_for_players(
        tx: &mut Transaction<'_, Postgres>,
        match_id: Uuid,
        player_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, i32>, sqlx::Error> {
        let results = sqlx::query_as::<_, (Uuid, i64)>(
            "SELECT beneficiary_player_id, SUM(contribution_amount)::bigint
             FROM player_teammate_elo_contributions
             WHERE match_id = $1 AND beneficiary_player_id = ANY($2)
             GROUP BY beneficiary_player_id",
        )
        .bind(match_id)
        .bind(player_ids)
        .fetch_all(&mut **tx)
        .await?;

        Ok(results
            .into_iter()
            .map(|(player_id, sum)| (player_id, sum as i32))
            .collect())
    }

    pub async fn get_match_total_for_player(
        pool: &sqlx::PgPool,
        match_id: Uuid,
        player_id: Uuid,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_as::<_, (i64,)>(
            "SELECT COALESCE(SUM(contribution_amount), 0)::bigint
             FROM player_teammate_elo_contributions
             WHERE match_id = $1 AND beneficiary_player_id = $2",
        )
        .bind(match_id)
        .bind(player_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 as i32)
    }
}
