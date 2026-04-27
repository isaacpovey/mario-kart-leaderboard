use crate::graphql::context::GraphQLContext;
use crate::graphql::lobby::fetch_lobby;
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

        if let Err(e) = gql_ctx
            .notification_manager
            .publish_lobby(&gql_ctx.pool, LobbyNotification { group_id })
            .await
        {
            tracing::error!(
                group_id = %group_id,
                "lobby pg_notify failed (data is already committed; live update will be missed): {}",
                e
            );
        }

        fetch_lobby(&gql_ctx.pool, group_id).await
    }

    /// Check a player out of their group's lobby.
    ///
    /// Idempotent: checking out a player who isn't in the lobby — including a
    /// player who no longer exists or belongs to a different group — is a
    /// no-op. The DELETE is bounded by `group_id`, so cross-group writes are
    /// not possible. Returns the updated lobby.
    async fn check_out_player(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "The player ID to check out")] player_id: ID,
    ) -> Result<Vec<Player>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let group_id = gql_ctx.authenticated_group_id()?;

        let player_uuid =
            Uuid::parse_str(&player_id).map_err(|_| Error::new("Invalid player ID"))?;

        models::LobbyEntry::check_out(&gql_ctx.pool, group_id, player_uuid).await?;

        if let Err(e) = gql_ctx
            .notification_manager
            .publish_lobby(&gql_ctx.pool, LobbyNotification { group_id })
            .await
        {
            tracing::error!(
                group_id = %group_id,
                "lobby pg_notify failed (data is already committed; live update will be missed): {}",
                e
            );
        }

        fetch_lobby(&gql_ctx.pool, group_id).await
    }
}

