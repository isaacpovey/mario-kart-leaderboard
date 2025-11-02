use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct TeamMatchScore {
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub team_id: Uuid,
    pub score: f64,
}
