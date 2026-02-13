use crate::graphql::context::GraphQLContext;
use crate::models::{PlayerMatchScore, PlayerRaceScore, Tournament};
use async_graphql::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct Player {
    pub id: Uuid,
    pub group_id: Uuid,
    pub name: String,
    pub elo_rating: i32,
    pub avatar_filename: Option<String>,
}

impl From<crate::models::Player> for Player {
    fn from(model: crate::models::Player) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            name: model.name,
            elo_rating: model.elo_rating,
            avatar_filename: model.avatar_filename,
        }
    }
}

#[derive(Clone, SimpleObject)]
pub struct PlayerTrackStats {
    pub track_name: String,
    pub average_position: f64,
    pub races_played: i64,
}

#[derive(Clone, SimpleObject)]
pub struct PlayerMatchHistoryEntry {
    pub match_id: ID,
    pub match_time: DateTime<Utc>,
    pub position: i32,
    pub elo_change: i32,
    pub tournament_elo_change: i32,
}

#[Object]
impl Player {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn avatar_filename(&self) -> Option<&str> {
        self.avatar_filename.as_deref()
    }

    async fn elo_rating(&self) -> i32 {
        self.elo_rating
    }

    async fn current_tournament_elo(&self, ctx: &Context<'_>) -> Result<Option<i32>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let tournament_elo = gql_ctx
            .player_active_tournament_elo_loader
            .load_one((self.id, self.group_id))
            .await?;

        Ok(tournament_elo)
    }

    async fn track_stats(&self, ctx: &Context<'_>) -> Result<Vec<PlayerTrackStats>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let stats = PlayerRaceScore::find_track_stats_by_player(&gql_ctx.pool, self.id).await?;

        Ok(stats
            .into_iter()
            .map(|s| PlayerTrackStats {
                track_name: s.track_name,
                average_position: s.average_position,
                races_played: s.races_played,
            })
            .collect())
    }

    async fn match_history(&self, ctx: &Context<'_>) -> Result<Vec<PlayerMatchHistoryEntry>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let active_tournament_id =
            Tournament::get_active_tournament(&gql_ctx.pool, self.group_id).await?;

        let Some(tournament_id) = active_tournament_id else {
            return Ok(vec![]);
        };

        let matches =
            PlayerMatchScore::find_by_player_and_tournament(&gql_ctx.pool, self.id, tournament_id)
                .await?;

        Ok(matches
            .into_iter()
            .filter(|m| m.completed)
            .map(|m| PlayerMatchHistoryEntry {
                match_id: ID(m.match_id.to_string()),
                match_time: m.match_time,
                position: m.position,
                elo_change: m.elo_change,
                tournament_elo_change: m.tournament_elo_change,
            })
            .collect())
    }
}
