-- Create enum type for tournament stat categories
CREATE TYPE tournament_stat_type AS ENUM (
    'best_teammate',
    'worst_teammate',
    'best_race',
    'worst_race',
    'biggest_swing',
    'most_helped',
    'most_hurt',
    'best_match',
    'worst_match'
);

-- Create tournament_stats table (normalized: multiple rows per tournament)
CREATE TABLE tournament_stats (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tournament_id uuid NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    stat_type tournament_stat_type NOT NULL,
    player_id uuid NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    value INTEGER NOT NULL,
    extra_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (tournament_id, stat_type)
);

-- Index for querying stats by tournament
CREATE INDEX idx_tournament_stats_tournament_id ON tournament_stats(tournament_id);

-- Index for querying stats by player
CREATE INDEX idx_tournament_stats_player_id ON tournament_stats(player_id);
