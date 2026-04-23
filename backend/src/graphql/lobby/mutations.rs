use crate::graphql::context::GraphQLContext;
use crate::graphql::players::types::Player;
use crate::models;
use crate::services::notification_manager::LobbyNotification;
use async_graphql::*;
use uuid::Uuid;

#[derive(Default)]
pub struct LobbyMutation;

#[Object]
impl LobbyMutation {
    /// Check a player into their group's lobby.
    ///
    /// Idempotent: calling twice with the same player is a no-op.
    /// Returns the updated lobby.
    async fn check_in_player(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The player ID to check in")] player_id: ID,
    ) -> Result<Vec<Player>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let player_uuid =
            Uuid::parse_str(&player_id).map_err(|_| Error::new("Invalid player ID"))?;

        let player = models::Player::find_by_id(&gql_ctx.pool, player_uuid)
            .await?
            .ok_or_else(|| Error::new("Player not found"))?;

        if player.group_id != group_id {
            return Err(Error::new("Player not found"));
        }

        models::LobbyEntry::check_in(&gql_ctx.pool, group_id, player_uuid).await?;

        gql_ctx
            .notification_manager
            .notify_lobby(LobbyNotification { group_id });

        let lobby_players = load_lobby_players(&gql_ctx.pool, group_id).await?;
        Ok(lobby_players)
    }
}

async fn load_lobby_players(
    pool: &crate::db::DbPool,
    group_id: Uuid,
) -> Result<Vec<Player>> {
    let entries = models::LobbyEntry::find_by_group_id(pool, group_id).await?;
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    let player_ids: Vec<Uuid> = entries.iter().map(|e| e.player_id).collect();
    let players_by_id = models::Player::find_by_ids(pool, &player_ids).await?;

    // Preserve check-in order
    let ordered: Vec<Player> = entries
        .iter()
        .filter_map(|e| {
            players_by_id
                .iter()
                .find(|p| p.id == e.player_id)
                .cloned()
                .map(Player::from)
        })
        .collect();

    Ok(ordered)
}
