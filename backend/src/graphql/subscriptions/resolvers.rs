use crate::graphql::context::GraphQLContext;
use crate::graphql::results::types::{PlayerMatchResult, PlayerRaceResult};
use crate::graphql::subscriptions::types::RaceResultUpdate;
use crate::graphql::teams::types::Team;
use crate::graphql::tournaments::types::LeaderboardEntry;
use crate::models;
use crate::services::notification_manager::RaceResultNotification;
use async_graphql::*;
use futures::stream::Stream;
use std::time::Duration;
use uuid::Uuid;

#[derive(Default)]
pub struct Subscription;

#[Subscription]
impl Subscription {
    /// Subscribe to race result updates for a specific tournament
    ///
    /// Receives real-time updates when race results are recorded for any match
    /// in the specified tournament. Updates include race scores, player aggregates,
    /// leaderboard changes, and completion status.
    ///
    /// # Authorization
    ///
    /// Only receives updates for matches belonging to the authenticated user's group
    async fn race_results_updated(
        &self,
        ctx: &Context<'_>,
        tournament_id: ID,
    ) -> Result<impl Stream<Item = Result<RaceResultUpdate>>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let tournament_uuid = Uuid::parse_str(&tournament_id.0)
            .map_err(|_| Error::new("Invalid tournament ID"))?;

        tracing::info!(
            "NOTIFY STEP 3: Subscription started for tournament_id={}, group_id={}",
            tournament_uuid,
            group_id
        );

        let notification_manager = gql_ctx.notification_manager.clone();
        let pool = gql_ctx.pool.clone();

        let stream = async_stream::stream! {
            let mut receiver = notification_manager.subscribe();

            tracing::info!("NOTIFY STEP 3: Waiting for notifications...");

            loop {
                tokio::select! {
                    notification = receiver.recv() => {
                        match notification {
                            Ok(notif) => {
                                tracing::info!(
                                    "NOTIFY STEP 3: Received notification - match_id={}, tournament_id={}, round={}, group_id={}",
                                    notif.match_id,
                                    notif.tournament_id,
                                    notif.round_number,
                                    notif.group_id
                                );

                                if notif.tournament_id == tournament_uuid && notif.group_id == group_id {
                                    tracing::info!("NOTIFY STEP 3: Notification passed filters");

                                    // Handle match creation notifications (round_number = 0) differently
                                    if notif.round_number == 0 {
                                        tracing::info!("NOTIFY STEP 3: Match creation notification (round=0), yielding minimal data");

                                        // Fetch leaderboard to provide in the update
                                        match fetch_tournament_leaderboard(&pool, notif.tournament_id).await {
                                            Ok(leaderboard) => {
                                                let update = RaceResultUpdate {
                                                    match_id: notif.match_id,
                                                    tournament_id: notif.tournament_id,
                                                    round_number: 0,
                                                    race_results: Vec::new(),
                                                    player_aggregates: Vec::new(),
                                                    leaderboard,
                                                    round_completed: false,
                                                    match_completed: false,
                                                    teams: Vec::new(),
                                                };
                                                tracing::info!("NOTIFY STEP 3: Yielding match creation update to stream");
                                                yield Ok(update)
                                            },
                                            Err(e) => {
                                                tracing::error!("Failed to fetch leaderboard for match creation: {:?}", e);
                                                yield Err(e);
                                            }
                                        }
                                        continue;
                                    }

                                    tracing::info!("NOTIFY STEP 3: Race result notification, fetching full data...");

                                    match fetch_race_result_data(&pool, &notif).await {
                                        Ok(update) => {
                                            tracing::info!("NOTIFY STEP 3: Yielding update to stream");
                                            yield Ok(update)
                                        },
                                        Err(e) => {
                                            tracing::error!("Failed to fetch race result data: {:?}", e);
                                            yield Err(e);
                                        }
                                    }
                                } else {
                                    tracing::info!(
                                        "NOTIFY STEP 3: Notification filtered out - expected tournament_id={} group_id={}, got tournament_id={} group_id={}",
                                        tournament_uuid,
                                        group_id,
                                        notif.tournament_id,
                                        notif.group_id
                                    );
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                                tracing::warn!("Subscription lagged, skipped {} messages", skipped);
                                continue;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                tracing::info!("Notification channel closed");
                                break;
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(30)) => {
                        tracing::debug!("NOTIFY STEP 3: Timeout tick (keeping connection alive)");
                        continue;
                    }
                }
            }
        };

        Ok(stream)
    }
}

async fn fetch_race_result_data(
    pool: &crate::db::DbPool,
    notification: &RaceResultNotification,
) -> Result<RaceResultUpdate> {
    tracing::info!("NOTIFY STEP 3: Fetching race results...");
    let race_results = fetch_race_results(pool, notification.match_id, notification.round_number).await?;
    tracing::info!("NOTIFY STEP 3: Race results fetched: {} results", race_results.len());

    tracing::info!("NOTIFY STEP 3: Fetching player aggregates...");
    let player_aggregates = fetch_player_aggregates(pool, notification.match_id).await?;
    tracing::info!("NOTIFY STEP 3: Player aggregates fetched: {} aggregates", player_aggregates.len());

    tracing::info!("NOTIFY STEP 3: Fetching tournament leaderboard...");
    let leaderboard = fetch_tournament_leaderboard(pool, notification.tournament_id).await?;
    tracing::info!("NOTIFY STEP 3: Tournament leaderboard fetched: {} entries", leaderboard.len());

    tracing::info!("NOTIFY STEP 3: Fetching completion status...");
    let (round_completed, match_completed) = fetch_completion_status(
        pool,
        notification.match_id,
        notification.round_number,
    )
    .await?;
    tracing::info!("NOTIFY STEP 3: Completion status fetched: round_completed={}, match_completed={}", round_completed, match_completed);

    let teams = if match_completed {
        tracing::info!("NOTIFY STEP 3: Match completed, fetching teams...");
        let t = fetch_teams(pool, notification.match_id).await?;
        tracing::info!("NOTIFY STEP 3: Teams fetched: {} teams", t.len());
        t
    } else {
        tracing::info!("NOTIFY STEP 3: Match not completed, skipping teams fetch");
        Vec::new()
    };

    tracing::info!("NOTIFY STEP 3: All data fetched successfully, creating update");

    Ok(RaceResultUpdate {
        match_id: notification.match_id,
        tournament_id: notification.tournament_id,
        round_number: notification.round_number,
        race_results,
        player_aggregates,
        leaderboard,
        round_completed,
        match_completed,
        teams,
    })
}

async fn fetch_race_results(
    pool: &crate::db::DbPool,
    match_id: Uuid,
    round_number: i32,
) -> Result<Vec<PlayerRaceResult>> {
    let scores = sqlx::query_as::<_, models::PlayerRaceScore>(
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
    .await?;

    Ok(scores.into_iter().map(PlayerRaceResult::from).collect())
}

async fn fetch_player_aggregates(
    pool: &crate::db::DbPool,
    match_id: Uuid,
) -> Result<Vec<PlayerMatchResult>> {
    let scores = sqlx::query_as::<_, models::PlayerMatchScore>(
        "SELECT group_id, match_id, player_id, position, elo_change,
                tournament_elo_change, tournament_elo_from_races,
                tournament_elo_from_contributions, created_at
         FROM player_match_scores
         WHERE match_id = $1
         ORDER BY position ASC",
    )
    .bind(match_id)
    .fetch_all(pool)
    .await?;

    Ok(scores.into_iter().map(PlayerMatchResult::from).collect())
}

async fn fetch_tournament_leaderboard(
    pool: &crate::db::DbPool,
    tournament_id: Uuid,
) -> Result<Vec<LeaderboardEntry>> {
    let entries = models::PlayerTournamentScore::get_tournament_leaderboard(pool, tournament_id).await?;

    Ok(entries
        .into_iter()
        .map(|(player_id, player_name, elo_rating, all_time_elo, avatar_filename)| {
            LeaderboardEntry {
                player_id,
                player_name,
                elo_rating,
                all_time_elo,
                avatar_filename,
            }
        })
        .collect())
}

async fn fetch_completion_status(
    pool: &crate::db::DbPool,
    match_id: Uuid,
    round_number: i32,
) -> Result<(bool, bool)> {
    let (round_completed, match_completed): (bool, bool) = sqlx::query_as(
        "SELECT r.completed, m.completed
         FROM rounds r
         JOIN matches m ON m.id = r.match_id
         WHERE r.match_id = $1 AND r.round_number = $2",
    )
    .bind(match_id)
    .bind(round_number)
    .fetch_one(pool)
    .await?;

    Ok((round_completed, match_completed))
}

async fn fetch_teams(pool: &crate::db::DbPool, match_id: Uuid) -> Result<Vec<Team>> {
    let teams = sqlx::query_as::<_, models::Team>(
        "SELECT id, match_id, name, score
         FROM teams
         WHERE match_id = $1
         ORDER BY score DESC NULLS LAST",
    )
    .bind(match_id)
    .fetch_all(pool)
    .await?;

    Ok(teams.into_iter().map(Team::from).collect())
}
