-- Create player_tournament_scores table for per-tournament ELO tracking
CREATE TABLE player_tournament_scores (
    player_id uuid NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    tournament_id uuid NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    group_id uuid NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    elo_rating INTEGER NOT NULL DEFAULT 1200,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (player_id, tournament_id)
);

-- Add indexes for performance
CREATE INDEX idx_player_tournament_scores_tournament_id ON player_tournament_scores(tournament_id, elo_rating DESC);
CREATE INDEX idx_player_tournament_scores_group_id ON player_tournament_scores(group_id);

-- Add ELO tracking columns to player_race_scores
ALTER TABLE player_race_scores ADD COLUMN all_time_elo_change INTEGER;
ALTER TABLE player_race_scores ADD COLUMN all_time_elo_after INTEGER;
ALTER TABLE player_race_scores ADD COLUMN tournament_elo_change INTEGER;
ALTER TABLE player_race_scores ADD COLUMN tournament_elo_after INTEGER;

-- Add tournament ELO change tracking to player_match_scores
ALTER TABLE player_match_scores ADD COLUMN tournament_elo_change INTEGER NOT NULL DEFAULT 0;

-- Backfill player_tournament_scores for existing players who have participated in tournaments
INSERT INTO player_tournament_scores (player_id, tournament_id, group_id, elo_rating, created_at, updated_at)
SELECT DISTINCT
    p.id,
    m.tournament_id,
    p.group_id,
    p.elo_rating,
    NOW(),
    NOW()
FROM players p
JOIN player_match_scores pms ON pms.player_id = p.id
JOIN matches m ON m.id = pms.match_id
ON CONFLICT DO NOTHING;

-- Backfill race-level ELO changes by splitting match-level changes equally
WITH race_counts AS (
    SELECT match_id, player_id, COUNT(*) as race_count
    FROM player_race_scores
    GROUP BY match_id, player_id
),
race_deltas AS (
    SELECT
        prs.match_id,
        prs.round_number,
        prs.player_id,
        COALESCE(ROUND(pms.elo_change::NUMERIC / NULLIF(rc.race_count, 0)), 0) as per_race_change
    FROM player_race_scores prs
    LEFT JOIN player_match_scores pms ON pms.match_id = prs.match_id AND pms.player_id = prs.player_id
    LEFT JOIN race_counts rc ON rc.match_id = prs.match_id AND rc.player_id = prs.player_id
)
UPDATE player_race_scores prs
SET
    all_time_elo_change = rd.per_race_change::INTEGER,
    tournament_elo_change = rd.per_race_change::INTEGER
FROM race_deltas rd
WHERE prs.match_id = rd.match_id
  AND prs.round_number = rd.round_number
  AND prs.player_id = rd.player_id;

-- Backfill tournament_elo_change in player_match_scores
UPDATE player_match_scores
SET tournament_elo_change = elo_change;

-- Calculate approximate "after" values for existing race data
-- This works backwards from current player ELO through their race history
WITH player_race_history AS (
    SELECT
        prs.player_id,
        prs.match_id,
        prs.round_number,
        prs.all_time_elo_change,
        prs.tournament_elo_change,
        m.time as match_time,
        ROW_NUMBER() OVER (PARTITION BY prs.player_id ORDER BY m.time DESC, prs.round_number DESC) as race_order_desc
    FROM player_race_scores prs
    JOIN matches m ON m.id = prs.match_id
    WHERE prs.all_time_elo_change IS NOT NULL
),
elo_running_totals AS (
    SELECT
        prh.player_id,
        prh.match_id,
        prh.round_number,
        p.elo_rating - SUM(prh.all_time_elo_change) OVER (
            PARTITION BY prh.player_id
            ORDER BY prh.race_order_desc
            ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
        ) + prh.all_time_elo_change as all_time_elo_after,
        pts.elo_rating - SUM(prh.tournament_elo_change) OVER (
            PARTITION BY prh.player_id
            ORDER BY prh.race_order_desc
            ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
        ) + prh.tournament_elo_change as tournament_elo_after
    FROM player_race_history prh
    JOIN players p ON p.id = prh.player_id
    JOIN matches m ON m.id = prh.match_id
    LEFT JOIN player_tournament_scores pts ON pts.player_id = prh.player_id AND pts.tournament_id = m.tournament_id
)
UPDATE player_race_scores prs
SET
    all_time_elo_after = ert.all_time_elo_after,
    tournament_elo_after = ert.tournament_elo_after
FROM elo_running_totals ert
WHERE prs.player_id = ert.player_id
  AND prs.match_id = ert.match_id
  AND prs.round_number = ert.round_number;
