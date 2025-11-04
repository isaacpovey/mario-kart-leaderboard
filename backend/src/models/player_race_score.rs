use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PlayerRaceScore {
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub round_number: i32,
    pub player_id: Uuid,
    pub position: i32,
    pub all_time_elo_change: Option<i32>,
    pub all_time_elo_after: Option<i32>,
    pub tournament_elo_change: Option<i32>,
    pub tournament_elo_after: Option<i32>,
    pub created_at: DateTime<Utc>,
}
