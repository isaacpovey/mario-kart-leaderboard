use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PlayerMatchScore {
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub player_id: Uuid,
    pub position: i32,
    pub elo_change: i32,
}
