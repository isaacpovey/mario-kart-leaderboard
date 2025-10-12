CREATE TABLE groups (
    id uuid PRIMARY KEY DEFAULT(gen_random_uuid()),
    name varchar(255) UNIQUE NOT NULL,
    password varchar(255) NOT NULL
);

CREATE TABLE players (
    id uuid PRIMARY KEY DEFAULT(gen_random_uuid()),
    group_id uuid NOT NULL,
    name varchar(255) NOT NULL,
    UNIQUE (group_id, name),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE
);


CREATE TABLE tournaments (
    id uuid PRIMARY KEY DEFAULT(gen_random_uuid()),
    group_id uuid NOT NULL,
    start_date date,
    end_date date,
    winner uuid,
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (winner) REFERENCES players (id) ON DELETE CASCADE ON UPDATE CASCADE
);


CREATE TABLE matches (
    id uuid PRIMARY KEY DEFAULT(gen_random_uuid()),
    group_id uuid NOT NULL,
    tournament_id uuid NOT NULL,
    time timestamptz NOT NULL,
    rounds integer NOT NULL,
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (tournament_id) REFERENCES tournaments (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE match_players (
    group_id uuid NOT NULL,
    match_id uuid,
    player_id uuid,
    PRIMARY KEY (match_id, player_id),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (match_id) REFERENCES matches (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE rounds (
    match_id uuid,
    round_number integer,
    PRIMARY KEY (match_id, round_number),
    FOREIGN KEY (match_id) REFERENCES matches (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE teams (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id uuid NOT NULL,
    match_id uuid NOT NULL,
    team_num integer NOT NULL,
    score integer,
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (match_id) REFERENCES matches (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE round_players (
    group_id uuid NOT NULL,
    match_id uuid,
    round_number integer,
    player_id uuid,
    team_id uuid NOT NULL,
    player_position integer NOT NULL,
    PRIMARY KEY (match_id, round_number, player_id),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (match_id) REFERENCES matches (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (match_id, round_number) REFERENCES rounds (match_id, round_number) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE team_players (
    group_id uuid NOT NULL,
    team_id uuid,
    player_id uuid,
    rank integer NOT NULL,
    PRIMARY KEY (team_id, player_id),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE team_match_scores (
    group_id uuid NOT NULL,
    match_id uuid,
    team_id uuid,
    score float NOT NULL,
    PRIMARY KEY (match_id, team_id),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (match_id) REFERENCES matches (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE player_match_scores (
    group_id uuid NOT NULL,
    match_id uuid,
    player_id uuid,
    score float NOT NULL,
    PRIMARY KEY (match_id, player_id),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (match_id) REFERENCES matches (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE player_race_scores (
    group_id uuid NOT NULL,
    match_id uuid,
    round_number integer,
    player_id uuid,
    score integer NOT NULL,
    PRIMARY KEY (match_id, round_number, player_id),
    FOREIGN KEY (group_id) REFERENCES groups (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (match_id) REFERENCES matches (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (match_id, round_number) REFERENCES rounds (match_id, round_number) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (player_id) REFERENCES players (id) ON DELETE CASCADE ON UPDATE CASCADE
);
