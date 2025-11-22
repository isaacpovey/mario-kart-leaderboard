use crate::db::DbPool;
use sqlx::FromRow;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Player {
    pub id: Uuid,
    pub group_id: Uuid,
    pub name: String,
    pub elo_rating: i32,
    pub avatar_filename: Option<String>,
}

impl Player {
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, name, elo_rating, avatar_filename FROM players WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = ids.len()))]
    pub async fn find_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, name, elo_rating, avatar_filename FROM players WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_group_id(
        pool: &DbPool,
        group_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, name, elo_rating, avatar_filename FROM players WHERE group_id = $1 ORDER BY elo_rating DESC"
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = group_ids.len()))]
    pub async fn find_by_group_ids(
        pool: &DbPool,
        group_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, name, elo_rating, avatar_filename FROM players WHERE group_id = ANY($1) ORDER BY elo_rating DESC"
        )
        .bind(group_ids)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn create(
        pool: &DbPool,
        group_id: Uuid,
        name: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO players (group_id, name) VALUES ($1, $2) RETURNING id, group_id, name, elo_rating, avatar_filename"
        )
        .bind(group_id)
        .bind(name)
        .fetch_one(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = team_ids.len()))]
    pub async fn find_by_team_ids(
        pool: &DbPool,
        team_ids: &[Uuid],
    ) -> Result<Vec<(Uuid, Self)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, Uuid, String, i32, Option<String>)>(
            "SELECT tp.team_id, p.id, p.group_id, p.name, p.elo_rating, p.avatar_filename
             FROM team_players tp
             JOIN players p ON tp.player_id = p.id
             WHERE tp.team_id = ANY($1)
             ORDER BY tp.team_id, tp.rank ASC",
        )
        .bind(team_ids)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(team_id, id, group_id, name, elo_rating, avatar_filename)| {
                (
                    team_id,
                    Player {
                        id,
                        group_id,
                        name,
                        elo_rating,
                        avatar_filename,
                    },
                )
            })
            .collect())
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = round_keys.len()))]
    pub async fn find_by_round_keys(
        pool: &DbPool,
        round_keys: &[(Uuid, i32)],
    ) -> Result<Vec<((Uuid, i32), Self)>, sqlx::Error> {
        let match_ids: Vec<Uuid> = round_keys.iter().map(|(match_id, _)| *match_id).collect();
        let round_numbers: Vec<i32> = round_keys
            .iter()
            .map(|(_, round_number)| *round_number)
            .collect();

        let rows = sqlx::query_as::<_, (Uuid, i32, Uuid, Uuid, String, i32, Option<String>)>(
            "SELECT rp.match_id, rp.round_number, p.id, p.group_id, p.name, p.elo_rating, p.avatar_filename
             FROM round_players rp
             JOIN players p ON rp.player_id = p.id
             WHERE rp.match_id = ANY($1) AND rp.round_number = ANY($2)
             ORDER BY rp.match_id, rp.round_number, rp.player_position ASC",
        )
        .bind(&match_ids)
        .bind(&round_numbers)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(match_id, round_number, id, group_id, name, elo_rating, avatar_filename)| {
                (
                    (match_id, round_number),
                    Player {
                        id,
                        group_id,
                        name,
                        elo_rating,
                        avatar_filename,
                    },
                )
            })
            .collect())
    }
}
