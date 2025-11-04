-- Add created_at timestamp to player_race_scores
ALTER TABLE player_race_scores
ADD COLUMN created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW();

-- Add created_at timestamp to player_match_scores
ALTER TABLE player_match_scores
ADD COLUMN created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW();

-- Backfill player_race_scores with match times for more accurate historical data
UPDATE player_race_scores prs
SET created_at = m.time
FROM matches m
WHERE prs.match_id = m.id;

-- Backfill player_match_scores with match times for more accurate historical data
UPDATE player_match_scores pms
SET created_at = m.time
FROM matches m
WHERE pms.match_id = m.id;
