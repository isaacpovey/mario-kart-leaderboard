pub mod loaders;
pub mod mutations;
pub mod types;

pub use loaders::LobbyByGroupLoader;
pub use mutations::LobbyMutation;

use crate::db::DbPool;
use crate::graphql::players::types::Player;
use crate::models;
use async_graphql::Result;
use uuid::Uuid;

/// Load the current lobby for a group, ordered by check-in time.
///
/// Shared by `checkInPlayer` / `checkOutPlayer` mutations (which return the
/// updated lobby) and the `lobbyUpdated` subscription (which yields it on
/// every notification). The `Group.lobby` field uses `LobbyByGroupLoader` for
/// DataLoader batching — this helper is for single-group reads outside a
/// request's DataLoader scope.
pub async fn fetch_lobby(pool: &DbPool, group_id: Uuid) -> Result<Vec<Player>> {
    let entries = models::LobbyEntry::find_by_group_id(pool, group_id).await?;
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    let player_ids: Vec<Uuid> = entries.iter().map(|e| e.player_id).collect();
    let players = models::Player::find_by_ids(pool, &player_ids).await?;

    Ok(entries
        .iter()
        .filter_map(|e| {
            players
                .iter()
                .find(|p| p.id == e.player_id && !p.disabled)
                .cloned()
                .map(Player::from)
        })
        .collect())
}
