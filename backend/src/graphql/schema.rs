use async_graphql::*;

// Import all feature modules
use crate::graphql::{auth, groups, matches, players, rounds, tournaments, tracks};

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
    rounds::RoundsMutation,
);

pub type Schema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema() -> Schema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .limit_depth(10)
        .limit_complexity(100)
        .finish()
}
