//! Service Layer
//!
//! This module contains business logic extracted from GraphQL resolvers.
//! Services encapsulate complex algorithms, transaction orchestration, and
//! multi-step operations, making them testable and reusable across different
//! contexts (GraphQL, CLI, background jobs, etc.).
//!
//! ## Architecture
//!
//! - **elo**: ELO rating system for Mario Kart races
//! - **scoring**: Position-to-points conversion utilities
//! - **validation**: Input validation for names, passwords, etc.
//! - **team_allocation**: Pure functions for balanced team creation
//! - **teammate_elo**: Pure functions for teammate ELO contribution calculations
//! - **race_allocation**: Algorithm for fair race distribution across players
//! - **track_selection**: Track selection with history avoidance
//! - **match_service**: High-level match creation orchestration
//! - **score_calculation**: Aggregate score calculations for players and teams
//! - **result_recording**: Race result recording and ELO update orchestration

pub mod elo;
pub mod match_service;
pub mod race_allocation;
pub mod result_recording;
pub mod score_calculation;
pub mod scoring;
pub mod team_allocation;
pub mod teammate_elo;
pub mod track_selection;
pub mod validation;
