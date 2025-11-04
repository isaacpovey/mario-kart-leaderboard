-- Create table to track teammate ELO contributions per race
CREATE TABLE player_teammate_elo_contributions (
    match_id uuid NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    round_number INTEGER NOT NULL,
    source_player_id uuid NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    beneficiary_player_id uuid NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    source_tournament_elo_change INTEGER NOT NULL,
    contribution_amount INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (match_id, round_number, source_player_id, beneficiary_player_id),
    FOREIGN KEY (match_id, round_number) REFERENCES rounds(match_id, round_number) ON DELETE CASCADE
);

-- Index for querying contributions received by a player in a match
CREATE INDEX idx_teammate_contributions_beneficiary ON player_teammate_elo_contributions(beneficiary_player_id, match_id);

-- Index for querying contributions given by a player
CREATE INDEX idx_teammate_contributions_source ON player_teammate_elo_contributions(source_player_id, match_id);
