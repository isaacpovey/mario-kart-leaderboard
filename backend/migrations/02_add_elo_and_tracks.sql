-- Add ELO rating to players
ALTER TABLE players ADD COLUMN elo_rating INTEGER NOT NULL DEFAULT 1200;

-- Add tracks table
CREATE TABLE tracks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE
);

-- Insert some default Mario Kart tracks
INSERT INTO tracks (name) VALUES
    ('Acorn Heights'),
    ('Boo Cinema'),
    ('Mario Circuit'),
    ('Starview Peak'),
    ('Sky-High Sundae'),
    ('DK Pass'),
    ('Dandelion Depths'),
    ('Cheep Cheep Falls'),
    ('Wario''s Galleon'),
    ('Salty Salty Speedway'),
    ('Peach Beach'),
    ('Great ? Block Ruins'),
    ('Dino Dino Jungle'),
    ('Faraway Oasis'),
    ('Peach Stadium'),
    ('Moo Moo Meadows'),
    ('Choco Mountain'),
    ('Toad''s Factory'),
    ('Crown City'),
    ('Koopa Troopa Beach'),
    ('DK Spaceport'),
    ('Whistlestop Summit'),
    ('Desert Hills'),
    ('Mario Bros. Circuit'),
    ('Shy Guy Bazaar'),
    ('Wario Stadium'),
    ('Airship Fortress'),
    ('Bowser''s Castle'),
    ('Dry Bones Burnout'),
    ('Rainbow Road');

-- Add track_id to rounds table
ALTER TABLE rounds ADD COLUMN track_id uuid REFERENCES tracks(id);

-- Update team creation mode enum
CREATE TYPE team_creation_mode AS ENUM ('balanced', 'full');
ALTER TABLE matches ADD COLUMN team_mode team_creation_mode NOT NULL DEFAULT 'balanced';

-- Change score columns to use integer for positions instead of float
ALTER TABLE team_match_scores ALTER COLUMN score TYPE INTEGER;
ALTER TABLE player_match_scores ALTER COLUMN score TYPE INTEGER;

-- Add completed status to matches
ALTER TABLE matches ADD COLUMN completed BOOLEAN NOT NULL DEFAULT FALSE;

-- Add indexes for performance
CREATE INDEX idx_players_elo_rating ON players(elo_rating DESC);
CREATE INDEX idx_matches_completed ON matches(completed);
CREATE INDEX idx_rounds_track_id ON rounds(track_id);