use async_graphql::*;

// Import all feature modules
use crate::graphql::{auth, groups, matches, players, tournaments, tracks};

/// Root Query combining all feature queries
#[derive(MergedObject, Default)]
pub struct Query(
    auth::AuthQuery,
    groups::GroupsQuery,
    players::PlayersQuery,
    tournaments::TournamentsQuery,
    matches::MatchesQuery,
    tracks::TracksQuery,
);

/// Root Mutation combining all feature mutations
#[derive(MergedObject, Default)]
pub struct Mutation(
    auth::AuthMutation,
    players::PlayersMutation,
    tournaments::TournamentsMutation,
    matches::MatchesMutation,
);

pub type Schema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema() -> Schema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription).finish()
}
