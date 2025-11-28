use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::{BiggestSwingData, Tournament, TournamentStat, TournamentStatType};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone)]
struct StatResult {
    player_id: Uuid,
    value: i32,
    extra_data: Option<serde_json::Value>,
}

pub async fn complete_tournament(
    pool: &DbPool,
    tournament_id: Uuid,
    group_id: Uuid,
) -> Result<Tournament> {
    let tournament = Tournament::find_by_id(pool, tournament_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Tournament not found".to_string()))?;

    if tournament.group_id != group_id {
        return Err(AppError::Unauthorized(
            "Tournament not in group".to_string(),
        ));
    }

    if tournament.winner.is_some() {
        return Err(AppError::InvalidInput(
            "Tournament already completed".to_string(),
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to start transaction: {e}")))?;

    let winner_id = calculate_winner(&mut tx, tournament_id).await?;
    let stats = calculate_all_stats(&mut tx, tournament_id).await?;

    let updated_tournament = Tournament::set_winner(&mut tx, tournament_id, winner_id).await?;

    let stat_records: Vec<_> = stats
        .into_iter()
        .map(|(stat_type, result)| {
            (
                tournament_id,
                stat_type,
                result.player_id,
                result.value,
                result.extra_data,
            )
        })
        .collect();

    TournamentStat::insert_batch(&mut tx, &stat_records).await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to commit transaction: {e}")))?;

    Ok(updated_tournament)
}

async fn calculate_winner(
    tx: &mut Transaction<'_, Postgres>,
    tournament_id: Uuid,
) -> Result<Uuid> {
    let result: Option<(Uuid,)> = sqlx::query_as(
        "SELECT player_id
         FROM player_tournament_scores
         WHERE tournament_id = $1
         ORDER BY elo_rating DESC
         LIMIT 1",
    )
    .bind(tournament_id)
    .fetch_optional(&mut **tx)
    .await?;

    result
        .map(|(id,)| id)
        .ok_or_else(|| AppError::InvalidInput("No players in tournament".to_string()))
}

async fn calculate_all_stats(
    tx: &mut Transaction<'_, Postgres>,
    tournament_id: Uuid,
) -> Result<Vec<(TournamentStatType, StatResult)>> {
    let race_stats = calculate_race_stats(tx, tournament_id).await?;
    let contribution_stats = calculate_contribution_stats(tx, tournament_id).await?;
    let match_stats = calculate_match_stats(tx, tournament_id).await?;

    Ok([race_stats, contribution_stats, match_stats].concat())
}

async fn calculate_race_stats(
    tx: &mut Transaction<'_, Postgres>,
    tournament_id: Uuid,
) -> Result<Vec<(TournamentStatType, StatResult)>> {
    #[derive(sqlx::FromRow)]
    struct RaceStatRow {
        stat_type: String,
        player_id: Uuid,
        value_primary: i32,
        value_secondary: Option<i32>,
        value_tertiary: Option<i32>,
    }

    let rows: Vec<RaceStatRow> = sqlx::query_as(
        r#"
        WITH tournament_matches AS (
            SELECT id AS match_id
            FROM matches
            WHERE tournament_id = $1
        ),
        race_scores AS (
            SELECT
                prs.player_id,
                prs.tournament_elo_change,
                prs.tournament_elo_after
            FROM player_race_scores prs
            INNER JOIN tournament_matches tm ON prs.match_id = tm.match_id
            WHERE prs.tournament_elo_change IS NOT NULL
        ),
        best_race AS (
            SELECT
                player_id,
                tournament_elo_change AS value_primary
            FROM race_scores
            ORDER BY tournament_elo_change DESC
            LIMIT 1
        ),
        worst_race AS (
            SELECT
                player_id,
                tournament_elo_change AS value_primary
            FROM race_scores
            ORDER BY tournament_elo_change ASC
            LIMIT 1
        ),
        player_swings AS (
            SELECT
                player_id,
                MAX(tournament_elo_after) - MIN(tournament_elo_after) AS swing_delta,
                MAX(tournament_elo_after) AS high_point,
                MIN(tournament_elo_after) AS low_point
            FROM race_scores
            GROUP BY player_id
        ),
        biggest_swing AS (
            SELECT
                player_id,
                swing_delta AS value_primary,
                high_point AS value_secondary,
                low_point AS value_tertiary
            FROM player_swings
            ORDER BY swing_delta DESC
            LIMIT 1
        )
        SELECT
            'best_race' AS stat_type, player_id, value_primary, NULL::int AS value_secondary, NULL::int AS value_tertiary
        FROM best_race
        UNION ALL
        SELECT
            'worst_race' AS stat_type, player_id, value_primary, NULL, NULL
        FROM worst_race
        UNION ALL
        SELECT
            'biggest_swing' AS stat_type, player_id, value_primary, value_secondary, value_tertiary
        FROM biggest_swing
        "#,
    )
    .bind(tournament_id)
    .fetch_all(&mut **tx)
    .await?;

    if rows.len() < 3 {
        return Err(AppError::InvalidInput(
            "Not enough race data to calculate stats".to_string(),
        ));
    }

    let stats = rows
        .into_iter()
        .filter_map(|row| {
            let stat_type = match row.stat_type.as_str() {
                "best_race" => TournamentStatType::BestRace,
                "worst_race" => TournamentStatType::WorstRace,
                "biggest_swing" => TournamentStatType::BiggestSwing,
                _ => return None,
            };

            let extra_data = if stat_type == TournamentStatType::BiggestSwing {
                row.value_secondary.zip(row.value_tertiary).map(|(high, low)| {
                    serde_json::to_value(BiggestSwingData {
                        high_value: high,
                        low_value: low,
                    })
                    .unwrap_or_default()
                })
            } else {
                None
            };

            Some((
                stat_type,
                StatResult {
                    player_id: row.player_id,
                    value: row.value_primary,
                    extra_data,
                },
            ))
        })
        .collect();

    Ok(stats)
}

async fn calculate_contribution_stats(
    tx: &mut Transaction<'_, Postgres>,
    tournament_id: Uuid,
) -> Result<Vec<(TournamentStatType, StatResult)>> {
    #[derive(sqlx::FromRow)]
    struct ContributionStatRow {
        stat_type: String,
        player_id: Uuid,
        value_primary: i64,
    }

    let rows: Vec<ContributionStatRow> = sqlx::query_as(
        r#"
        WITH tournament_matches AS (
            SELECT id AS match_id
            FROM matches
            WHERE tournament_id = $1
        ),
        contributions AS (
            SELECT
                ptec.source_player_id,
                ptec.beneficiary_player_id,
                ptec.contribution_amount
            FROM player_teammate_elo_contributions ptec
            INNER JOIN tournament_matches tm ON ptec.match_id = tm.match_id
        ),
        source_aggregates AS (
            SELECT
                source_player_id AS player_id,
                SUM(contribution_amount) AS total_contribution
            FROM contributions
            GROUP BY source_player_id
        ),
        best_teammate AS (
            SELECT player_id, total_contribution AS value_primary
            FROM source_aggregates
            ORDER BY total_contribution DESC
            LIMIT 1
        ),
        worst_teammate AS (
            SELECT player_id, total_contribution AS value_primary
            FROM source_aggregates
            ORDER BY total_contribution ASC
            LIMIT 1
        ),
        beneficiary_aggregates AS (
            SELECT
                beneficiary_player_id AS player_id,
                SUM(contribution_amount) AS total_received
            FROM contributions
            GROUP BY beneficiary_player_id
        ),
        most_helped AS (
            SELECT player_id, total_received AS value_primary
            FROM beneficiary_aggregates
            ORDER BY total_received DESC
            LIMIT 1
        ),
        most_hurt AS (
            SELECT player_id, total_received AS value_primary
            FROM beneficiary_aggregates
            ORDER BY total_received ASC
            LIMIT 1
        )
        SELECT
            'best_teammate' AS stat_type, player_id, value_primary
        FROM best_teammate
        UNION ALL
        SELECT
            'worst_teammate', player_id, value_primary
        FROM worst_teammate
        UNION ALL
        SELECT
            'most_helped', player_id, value_primary
        FROM most_helped
        UNION ALL
        SELECT
            'most_hurt', player_id, value_primary
        FROM most_hurt
        "#,
    )
    .bind(tournament_id)
    .fetch_all(&mut **tx)
    .await?;

    if rows.len() < 4 {
        return Err(AppError::InvalidInput(
            "Not enough teammate contribution data to calculate stats".to_string(),
        ));
    }

    let stats = rows
        .into_iter()
        .filter_map(|row| {
            let stat_type = match row.stat_type.as_str() {
                "best_teammate" => TournamentStatType::BestTeammate,
                "worst_teammate" => TournamentStatType::WorstTeammate,
                "most_helped" => TournamentStatType::MostHelped,
                "most_hurt" => TournamentStatType::MostHurt,
                _ => return None,
            };

            Some((
                stat_type,
                StatResult {
                    player_id: row.player_id,
                    value: row.value_primary as i32,
                    extra_data: None,
                },
            ))
        })
        .collect();

    Ok(stats)
}

async fn calculate_match_stats(
    tx: &mut Transaction<'_, Postgres>,
    tournament_id: Uuid,
) -> Result<Vec<(TournamentStatType, StatResult)>> {
    #[derive(sqlx::FromRow)]
    struct MatchStatRow {
        stat_type: String,
        player_id: Uuid,
        value_primary: i32,
    }

    let rows: Vec<MatchStatRow> = sqlx::query_as(
        r#"
        WITH tournament_match_scores AS (
            SELECT
                pms.player_id,
                pms.tournament_elo_change
            FROM player_match_scores pms
            INNER JOIN matches m ON pms.match_id = m.id
            WHERE m.tournament_id = $1
        ),
        best_match AS (
            SELECT player_id, tournament_elo_change AS value_primary
            FROM tournament_match_scores
            ORDER BY tournament_elo_change DESC
            LIMIT 1
        ),
        worst_match AS (
            SELECT player_id, tournament_elo_change AS value_primary
            FROM tournament_match_scores
            ORDER BY tournament_elo_change ASC
            LIMIT 1
        )
        SELECT
            'best_match' AS stat_type, player_id, value_primary
        FROM best_match
        UNION ALL
        SELECT
            'worst_match', player_id, value_primary
        FROM worst_match
        "#,
    )
    .bind(tournament_id)
    .fetch_all(&mut **tx)
    .await?;

    if rows.len() < 2 {
        return Err(AppError::InvalidInput(
            "Not enough match data to calculate stats".to_string(),
        ));
    }

    let stats = rows
        .into_iter()
        .filter_map(|row| {
            let stat_type = match row.stat_type.as_str() {
                "best_match" => TournamentStatType::BestMatch,
                "worst_match" => TournamentStatType::WorstMatch,
                _ => return None,
            };

            Some((
                stat_type,
                StatResult {
                    player_id: row.player_id,
                    value: row.value_primary,
                    extra_data: None,
                },
            ))
        })
        .collect();

    Ok(stats)
}
