-- Allow players to be disabled manually via the database.
-- Disabled players are hidden from player selection (lobby, create match)
-- but remain visible in historical matches and tournaments.
ALTER TABLE players ADD COLUMN disabled BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX idx_players_group_active ON players (group_id) WHERE disabled = FALSE;
