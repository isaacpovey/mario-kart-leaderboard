use crate::db::DbPool;
use sqlx::FromRow;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

use super::Player;

#[derive(Debug, Clone, FromRow)]
pub struct Team {
    pub id: Uuid,
    pub group_id: Uuid,
    pub match_id: Uuid,
    pub team_num: i32,
    pub score: Option<i32>,
}

impl Team {
    #[instrument(level = "debug", skip(pool))]
    pub async fn find_by_match_id(
        pool: &DbPool,
        match_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, match_id, team_num, score
             FROM teams
             WHERE match_id = $1
             ORDER BY team_num ASC",
        )
        .bind(match_id)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool), fields(batch_size = match_ids.len()))]
    pub async fn find_by_match_ids(
        pool: &DbPool,
        match_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, group_id, match_id, team_num, score
             FROM teams
             WHERE match_id = ANY($1)
             ORDER BY match_id, team_num ASC",
        )
        .bind(match_ids)
        .fetch_all(pool)
        .await
    }

    #[instrument(level = "debug", skip(pool))]
    pub async fn get_by_match_with_players(
        pool: &DbPool,
        match_id: Uuid,
    ) -> Result<Vec<(Team, Vec<Player>)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, Uuid, i32, Option<i32>, Uuid, Uuid, String, i32, Option<String>)>(
            "SELECT
                t.id, t.group_id, t.match_id, t.team_num, t.score,
                p.id, p.group_id, p.name, p.elo_rating, p.avatar_filename
             FROM teams t
             JOIN team_players tp ON tp.team_id = t.id
             JOIN players p ON p.id = tp.player_id
             WHERE t.match_id = $1
             ORDER BY t.team_num ASC, tp.rank ASC",
        )
        .bind(match_id)
        .fetch_all(pool)
        .await?;

        let grouped = rows.into_iter().fold(
            HashMap::<Uuid, (Team, Vec<Player>)>::new(),
            |mut acc, (team_id, team_group_id, team_match_id, team_num, score,
                       player_id, player_group_id, player_name, player_elo, player_avatar)| {
                let team = Team {
                    id: team_id,
                    group_id: team_group_id,
                    match_id: team_match_id,
                    team_num,
                    score,
                };
                let player = Player {
                    id: player_id,
                    group_id: player_group_id,
                    name: player_name,
                    elo_rating: player_elo,
                    avatar_filename: player_avatar,
                };

                acc.entry(team_id)
                    .and_modify(|(_, players)| players.push(player.clone()))
                    .or_insert((team, vec![player]));
                acc
            },
        );

        let mut result: Vec<(Team, Vec<Player>)> = grouped.into_values().collect();
        result.sort_by_key(|(team, _)| team.team_num);

        Ok(result)
    }
}
