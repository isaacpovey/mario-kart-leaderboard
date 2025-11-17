-- Add breakdown columns to player_match_scores to separate personal vs teammate contributions
-- This improves transparency and auditability of tournament ELO calculations

BEGIN;

-- Add new columns to track the breakdown of tournament ELO changes
ALTER TABLE player_match_scores
    ADD COLUMN tournament_elo_from_races INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN tournament_elo_from_contributions INTEGER NOT NULL DEFAULT 0;

-- Backfill existing data by splitting tournament_elo_change
-- For now, we'll put all existing values in tournament_elo_from_races
-- The recomputation migration will fix this properly
UPDATE player_match_scores
SET tournament_elo_from_races = tournament_elo_change,
    tournament_elo_from_contributions = 0;

-- Add a check constraint to ensure the total equals the sum of parts
-- This is added AFTER backfilling to avoid constraint violations
ALTER TABLE player_match_scores
    ADD CONSTRAINT tournament_elo_breakdown_consistent
    CHECK (tournament_elo_change = tournament_elo_from_races + tournament_elo_from_contributions);

COMMIT;
