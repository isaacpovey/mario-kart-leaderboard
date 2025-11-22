use crate::graphql::context::GraphQLContext;
use crate::graphql::tournaments::types::{ActiveTournamentWithLeaderboard, LeaderboardEntry, Tournament};
use crate::models;
use async_graphql::*;

#[derive(Default)]
pub struct TournamentsQuery;

#[Object]
impl TournamentsQuery {
    async fn tournaments(&self, ctx: &Context<'_>) -> Result<Vec<Tournament>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let tournaments = models::Tournament::find_by_group_id(&gql_ctx.pool, group_id).await?;

        Ok(tournaments.into_iter().map(Tournament::from).collect())
    }

    async fn active_tournament(&self, ctx: &Context<'_>) -> Result<Option<ActiveTournamentWithLeaderboard>> {
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
                    .map(|(player_id, player_name, elo_rating, all_time_elo, avatar_filename)| {
                        LeaderboardEntry {
                            player_id,
                            player_name,
                            elo_rating,
                            all_time_elo,
                            avatar_filename,
                        }
                    })
                    .collect();

                Ok(Some(ActiveTournamentWithLeaderboard {
                    tournament: Tournament::from(tournament),
                    leaderboard,
                }))
            }
            None => Ok(None),
        }
    }
}
