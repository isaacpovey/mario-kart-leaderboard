use crate::graphql::context::GraphQLContext;
use crate::graphql::tournaments::types::{
    build_player_elo_history, ActiveTournamentWithLeaderboard, LeaderboardEntry, Tournament,
    TournamentDetail, TournamentStat, TournamentSummary,
};
use crate::models;
use async_graphql::*;
use uuid::Uuid;

#[derive(Default)]
pub struct TournamentsQuery;

#[Object]
impl TournamentsQuery {
    async fn tournaments(&self, ctx: &Context<'_>) -> Result<Vec<TournamentSummary>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let tournaments = models::Tournament::find_by_group_id(&gql_ctx.pool, group_id).await?;

        Ok(tournaments
            .into_iter()
            .map(TournamentSummary::from)
            .collect())
    }

    async fn active_tournament(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<ActiveTournamentWithLeaderboard>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let tournament = sqlx::query_as::<_, models::Tournament>(
            "SELECT id, group_id, start_date, end_date, winner
             FROM tournaments
             WHERE group_id = $1 AND winner IS NULL
             ORDER BY start_date DESC NULLS LAST
             LIMIT 1",
        )
        .bind(group_id)
        .fetch_optional(&gql_ctx.pool)
        .await?;

        match tournament {
            Some(tournament) => {
                let entries = models::PlayerTournamentScore::get_tournament_leaderboard(
                    &gql_ctx.pool,
                    tournament.id,
                )
                .await?;

                let leaderboard = entries
                    .into_iter()
                    .map(
                        |(player_id, player_name, elo_rating, all_time_elo, avatar_filename)| {
                            LeaderboardEntry {
                                player_id,
                                player_name,
                                elo_rating,
                                all_time_elo,
                                avatar_filename,
                            }
                        },
                    )
                    .collect();

                Ok(Some(ActiveTournamentWithLeaderboard {
                    tournament: Tournament::from(tournament),
                    leaderboard,
                }))
            }
            None => Ok(None),
        }
    }

    async fn tournament_by_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The tournament ID")] id: ID,
    ) -> Result<Option<TournamentDetail>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let tournament_id =
            Uuid::parse_str(&id).map_err(|_| Error::new("Invalid tournament ID"))?;

        let tournament = models::Tournament::find_by_id(&gql_ctx.pool, tournament_id).await?;

        match tournament {
            Some(tournament) if tournament.group_id == group_id => {
                let leaderboard_entries =
                    models::PlayerTournamentScore::get_tournament_leaderboard(
                        &gql_ctx.pool,
                        tournament.id,
                    )
                    .await?;

                let all_players: Vec<(Uuid, String)> = leaderboard_entries
                    .iter()
                    .map(|(player_id, player_name, _, _, _)| (*player_id, player_name.clone()))
                    .collect();

                let leaderboard: Vec<LeaderboardEntry> = leaderboard_entries
                    .into_iter()
                    .map(
                        |(player_id, player_name, elo_rating, all_time_elo, avatar_filename)| {
                            LeaderboardEntry {
                                player_id,
                                player_name,
                                elo_rating,
                                all_time_elo,
                                avatar_filename,
                            }
                        },
                    )
                    .collect();

                let stats_models =
                    models::TournamentStat::find_by_tournament_id(&gql_ctx.pool, tournament.id)
                        .await?;
                let stats: Vec<TournamentStat> =
                    stats_models.into_iter().map(TournamentStat::from).collect();

                let elo_history_raw =
                    models::PlayerRaceScore::find_elo_history_by_tournament_id(
                        &gql_ctx.pool,
                        tournament.id,
                    )
                    .await?;
                let player_elo_history = build_player_elo_history(elo_history_raw, &all_players);

                Ok(Some(TournamentDetail {
                    id: tournament.id,
                    start_date: tournament.start_date,
                    end_date: tournament.end_date,
                    winner: tournament.winner,
                    leaderboard,
                    stats,
                    player_elo_history,
                }))
            }
            _ => Ok(None),
        }
    }
}
