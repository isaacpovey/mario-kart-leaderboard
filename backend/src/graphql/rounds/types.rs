use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use crate::graphql::results::types::PlayerRaceResult;
use crate::graphql::tracks::types::Track;
use async_graphql::*;
use uuid::Uuid;

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

    async fn track_id(&self) -> Option<ID> {
        self.track_id.map(|id| ID(id.to_string()))
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

    async fn players(&self, ctx: &Context<'_>) -> Result<Vec<Player>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let player_ids = if let Some(cached_ids) = &self.cached_result_player_ids {
            cached_ids.clone()
        } else {
            return Ok(gql_ctx
                .players_by_round_loader
                .load_one((self.match_id, self.round_number))
                .await?
                .unwrap_or_default()
                .into_iter()
                .map(Player::from)
                .collect());
        };

        let players_map = gql_ctx
            .player_loader
            .load_many(player_ids.clone())
            .await?;

        Ok(player_ids
            .into_iter()
            .filter_map(|id| players_map.get(&id).map(|p| Player::from(p.clone())))
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
