use async_graphql::*;
use async_graphql::extensions::OpenTelemetry;

// Import all feature modules
use crate::graphql::{auth, groups, matches, players, rounds, subscriptions, tournaments, tracks};

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

/// Root Subscription for real-time updates
pub use subscriptions::Subscription;

pub type Schema = async_graphql::Schema<Query, Mutation, Subscription>;

pub fn build_schema() -> Schema {
    let tracer = opentelemetry::global::tracer("graphql");

    Schema::build(Query::default(), Mutation::default(), Subscription::default())
        .extension(OpenTelemetry::new(tracer))
        .limit_depth(20)
        .limit_complexity(1000)
        .finish()
}
