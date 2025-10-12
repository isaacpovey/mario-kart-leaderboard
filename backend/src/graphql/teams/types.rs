use async_graphql::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Team {
    pub id: Uuid,
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub team_num: i32,
    pub score: Option<i32>,
}

impl From<crate::models::Team> for Team {
    fn from(model: crate::models::Team) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            match_id: model.match_id,
            team_num: model.team_num,
            score: model.score,
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

    async fn score(&self) -> Option<i32> {
        self.score
    }
}
