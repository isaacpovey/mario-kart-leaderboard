use async_graphql::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum TeamCreationMode {
    Balanced,
    Full,
}

impl From<crate::models::TeamCreationMode> for TeamCreationMode {
    fn from(model: crate::models::TeamCreationMode) -> Self {
        match model {
            crate::models::TeamCreationMode::Balanced => TeamCreationMode::Balanced,
            crate::models::TeamCreationMode::Full => TeamCreationMode::Full,
        }
    }
}

#[derive(Clone)]
pub struct Match {
    pub id: Uuid,
    pub group_id: Uuid,
    pub tournament_id: Uuid,
    pub time: DateTime<Utc>,
    pub rounds: i32,
    pub team_mode: TeamCreationMode,
    pub completed: bool,
}

impl From<crate::models::Match> for Match {
    fn from(model: crate::models::Match) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            tournament_id: model.tournament_id,
            time: model.time,
            rounds: model.rounds,
            team_mode: TeamCreationMode::from(model.team_mode),
            completed: model.completed,
        }
    }
}

#[Object]
impl Match {
    async fn id(&self) -> ID {
        ID(self.id.to_string())
    }

    async fn tournament_id(&self) -> ID {
        ID(self.tournament_id.to_string())
    }

    async fn time(&self) -> String {
        self.time.to_rfc3339()
    }

    async fn rounds(&self) -> i32 {
        self.rounds
    }

    async fn team_mode(&self) -> TeamCreationMode {
        self.team_mode
    }

    async fn completed(&self) -> bool {
        self.completed
    }
}
