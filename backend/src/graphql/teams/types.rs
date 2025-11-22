use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use async_graphql::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Team {
    pub id: Uuid,
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub team_num: i32,
    pub score: Option<i32>,
    pub cached_players: Option<Vec<crate::models::Player>>,
}

impl From<crate::models::Team> for Team {
    fn from(model: crate::models::Team) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            match_id: model.match_id,
            team_num: model.team_num,
            score: model.score,
            cached_players: None,
        }
    }
}

impl Team {
    pub fn from_with_players(model: crate::models::Team, players: Vec<crate::models::Player>) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            match_id: model.match_id,
            team_num: model.team_num,
            score: model.score,
            cached_players: Some(players),
        }
    }
}

#[Object]
impl Team {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn team_num(&self) -> i32 {
        self.team_num
    }

    async fn name(&self) -> String {
        format!("Team {}", self.team_num)
    }

    async fn score(&self) -> Option<i32> {
        self.score
    }

    async fn players(&self, ctx: &Context<'_>) -> Result<Vec<Player>> {
        if let Some(cached) = &self.cached_players {
            return Ok(cached.iter().map(|p| Player::from(p.clone())).collect());
        }

        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let players = gql_ctx
            .players_by_team_loader
            .load_one(self.id)
            .await?
            .unwrap_or_default()
            .into_iter()
            .map(Player::from)
            .collect();

        Ok(players)
    }
}
