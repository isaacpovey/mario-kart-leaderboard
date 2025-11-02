use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PlayerRaceScore {
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub round_number: i32,
    pub player_id: Uuid,
    pub position: i32,
}
