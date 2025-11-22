// Database query tracing utilities
//
// This module provides manual tracing for SQL queries since sqlx-tracing
// has API limitations (no Clone on Pool, no commit on Transaction).
//
// We create spans that wrap query execution, which will be picked up by
// the OpenTelemetry layer configured in observability.rs

/// Macro to trace a database query with a custom operation name
///
/// Usage:
/// ```ignore
/// use crate::traced_query;
///
/// let player = traced_query!("player::find_by_id",
///     sqlx::query_as::<_, Player>("SELECT * FROM players WHERE id = $1")
///         .bind(id)
///         .fetch_one(pool)
/// ).await?;
/// ```
#[macro_export]
macro_rules! traced_query {
    ($operation:expr, $query:expr) => {{
        use tracing::Instrument;

        let span = tracing::info_span!(
            "db.query",
            otel.name = $operation,
            db.system = "postgresql"
        );

        $query.instrument(span)
    }};
}

// Re-export for convenience
pub use traced_query;
