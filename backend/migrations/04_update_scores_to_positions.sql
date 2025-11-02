ALTER TABLE player_race_scores RENAME COLUMN score TO position;

ALTER TABLE player_match_scores RENAME COLUMN score TO position;

ALTER TABLE player_match_scores ADD COLUMN elo_change INTEGER NOT NULL DEFAULT 0;

ALTER TABLE rounds ADD COLUMN completed BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE team_match_scores ALTER COLUMN score TYPE DOUBLE PRECISION;

CREATE INDEX idx_rounds_completed ON rounds(match_id, completed);
