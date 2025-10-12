-- Remove team_mode column from matches table
ALTER TABLE matches DROP COLUMN team_mode;

-- Drop the team_creation_mode enum type
DROP TYPE team_creation_mode;
