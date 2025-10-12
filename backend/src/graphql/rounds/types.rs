use async_graphql::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Round {
    pub match_id: Uuid,
    pub round_number: i32,
    pub track_id: Option<Uuid>,
    pub completed: bool,
}

impl From<crate::models::Round> for Round {
    fn from(model: crate::models::Round) -> Self {
        Self {
            match_id: model.match_id,
            round_number: model.round_number,
            track_id: model.track_id,
            completed: model.completed,
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
}
