use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use crate::graphql::tournaments::types::LeaderboardEntry;
use async_graphql::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
}

impl From<crate::models::Group> for Group {
    fn from(model: crate::models::Group) -> Self {
        Self {
            id: model.id,
            name: model.name,
        }
    }
}

#[Object]
impl Group {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn players(&self, ctx: &Context<'_>) -> Result<Vec<Player>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let players = gql_ctx
            .players_by_group_loader
            .load_one(self.id)
            .await?
            .unwrap_or_default();

        Ok(players.into_iter().map(Player::from).collect())
    }

    async fn all_time_leaderboard(&self, ctx: &Context<'_>) -> Result<Vec<LeaderboardEntry>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let entries = sqlx::query_as::<_, (Uuid, String, i32, Option<String>)>(
            "SELECT id, name, elo_rating, avatar_filename
             FROM players
             WHERE group_id = $1
             ORDER BY elo_rating DESC",
        )
        .bind(self.id)
        .fetch_all(&gql_ctx.pool)
        .await?;

        Ok(entries
            .into_iter()
            .map(|(player_id, player_name, elo_rating, avatar_filename)| LeaderboardEntry {
                player_id,
                player_name,
                avatar_filename,
                elo_rating: elo_rating,
                all_time_elo: elo_rating,
                total_score: elo_rating,
            })
            .collect())
    }
}
