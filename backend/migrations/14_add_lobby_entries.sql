CREATE TABLE lobby_entries (
    group_id uuid NOT NULL,
    player_id uuid NOT NULL,
    checked_in_at timestamptz NOT NULL DEFAULT NOW(),
    PRIMARY KEY (group_id, player_id),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX idx_lobby_entries_group_id ON lobby_entries (group_id);
