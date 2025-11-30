use crate::graphql::context::GraphQLContext;
use crate::graphql::results::types::PlayerRaceResult;
use crate::graphql::tracks::types::Track;
use async_graphql::*;
use sqlx;
use uuid::Uuid;

#[derive(Clone)]
pub struct RoundPlayer {
    pub player: crate::models::Player,
    pub team_id: Uuid,
}

#[Object]
impl RoundPlayer {
    async fn id(&self) -> ID {
        self.player.id.to_string().into()
    }

    async fn name(&self) -> &str {
        &self.player.name
    }

    async fn avatar_filename(&self) -> Option<&str> {
        self.player.avatar_filename.as_deref()
    }

    async fn team_id(&self) -> ID {
        self.team_id.to_string().into()
    }
}

#[derive(Clone)]
pub struct Round {
    pub match_id: Uuid,
    pub round_number: i32,
    pub track_id: Option<Uuid>,
    pub completed: bool,
    pub cached_track: Option<Option<crate::models::Track>>,
    pub cached_result_player_ids: Option<Vec<Uuid>>,
    pub cached_results: Option<Vec<crate::models::PlayerRaceScore>>,
}

impl From<crate::models::Round> for Round {
    fn from(model: crate::models::Round) -> Self {
        Self {
            match_id: model.match_id,
            round_number: model.round_number,
            track_id: model.track_id,
            completed: model.completed,
            cached_track: None,
            cached_result_player_ids: None,
            cached_results: None,
        }
    }
}

impl Round {
    pub fn from_with_tracks_and_results(
        model: crate::models::Round,
        track: Option<crate::models::Track>,
        result_player_ids: Vec<Uuid>,
        results: Vec<crate::models::PlayerRaceScore>,
    ) -> Self {
        Self {
            match_id: model.match_id,
            round_number: model.round_number,
            track_id: model.track_id,
            completed: model.completed,
            cached_track: Some(track),
            cached_result_player_ids: Some(result_player_ids),
            cached_results: Some(results),
        }
    }
}

#[Object]
impl Round {
    async fn round_number(&self) -> i32 {
        self.round_number
    }

    async fn completed(&self) -> bool {
        self.completed
    }

    async fn track(&self, ctx: &Context<'_>) -> Result<Option<Track>> {
        if let Some(cached) = &self.cached_track {
            return Ok(cached.as_ref().map(|t| Track::from(t.clone())));
        }

        let Some(track_id) = self.track_id else {
            return Ok(None);
        };

        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let track = gql_ctx
            .track_loader
            .load_one(track_id)
            .await?
            .map(Track::from);

        Ok(track)
    }

    async fn players(&self, ctx: &Context<'_>) -> Result<Vec<RoundPlayer>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let round_player_records: Vec<(Uuid, Uuid)> = sqlx::query_as(
            "SELECT player_id, team_id FROM round_players
             WHERE match_id = $1 AND round_number = $2
             ORDER BY player_position",
        )
        .bind(self.match_id)
        .bind(self.round_number)
        .fetch_all(&gql_ctx.pool)
        .await?;

        let player_ids: Vec<Uuid> = round_player_records.iter().map(|(pid, _)| *pid).collect();

        let players_map = gql_ctx.player_loader.load_many(player_ids).await?;

        Ok(round_player_records
            .into_iter()
            .filter_map(|(player_id, team_id)| {
                players_map.get(&player_id).map(|player| RoundPlayer {
                    player: player.clone(),
                    team_id,
                })
            })
            .collect())
    }

    async fn results(&self, ctx: &Context<'_>) -> Result<Vec<PlayerRaceResult>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let race_scores = if let Some(cached) = &self.cached_results {
            cached.clone()
        } else {
            return Ok(gql_ctx
                .player_race_scores_by_round_loader
                .load_one((self.match_id, self.round_number))
                .await?
                .unwrap_or_default()
                .into_iter()
                .map(PlayerRaceResult::from)
                .collect());
        };

        let player_ids: Vec<uuid::Uuid> = race_scores.iter().map(|s| s.player_id).collect();

        let players_map = gql_ctx
            .player_loader
            .load_many(player_ids)
            .await?;

        Ok(race_scores
            .into_iter()
            .filter_map(|score| {
                players_map.get(&score.player_id).map(|player| {
                    PlayerRaceResult::from_with_player(score, player.clone())
                })
            })
            .collect())
    }
}
