-- Recompute all ELO ratings from scratch with proper breakdown of personal vs teammate contributions
-- This migration processes all races chronologically and rebuilds all ELO data

BEGIN;

-- Step 1: Reset all ELO ratings to default (1200)
UPDATE players SET elo_rating = 1200;
UPDATE player_tournament_scores SET elo_rating = 1200, updated_at = NOW();

-- Step 2: Clear all calculated ELO data
UPDATE player_race_scores
SET all_time_elo_change = NULL,
    all_time_elo_after = NULL,
    tournament_elo_change = NULL,
    tournament_elo_after = NULL;

DELETE FROM player_teammate_elo_contributions;

UPDATE player_match_scores
SET elo_change = 0,
    tournament_elo_change = 0,
    tournament_elo_from_races = 0,
    tournament_elo_from_contributions = 0;

-- Step 3: Create temporary function to calculate ELO change
-- Based on the Rust implementation in backend/src/services/elo.rs
CREATE OR REPLACE FUNCTION calculate_elo_change(
    player_elo INTEGER,
    player_position INTEGER
) RETURNS INTEGER AS $$
DECLARE
    k_factor CONSTANT NUMERIC := 100.0;
    total_race_size CONSTANT INTEGER := 24;
    max_cpu_elo CONSTANT INTEGER := 1400;
    min_cpu_elo CONSTANT INTEGER := 600;
    cpu_elo_decrease CONSTANT INTEGER := 100;

    expected_score NUMERIC := 0.0;
    actual_score NUMERIC;
    cpu_elo INTEGER;
    position INTEGER;
    expected_vs_opponent NUMERIC;
    elo_change INTEGER;
BEGIN
    -- Calculate actual score from position (normalized to 0-1)
    -- 1st place = 1.0, 24th place = 0.0
    actual_score := (total_race_size - player_position)::NUMERIC / (total_race_size - 1)::NUMERIC;

    -- Calculate expected score vs CPU opponents (positions 3-24)
    FOR position IN 3..total_race_size LOOP
        -- CPU ELO decreases 100 points per position
        cpu_elo := GREATEST(min_cpu_elo, max_cpu_elo - ((position - 1) * cpu_elo_decrease));

        -- Expected score vs this CPU opponent
        expected_vs_opponent := 1.0 / (1.0 + POWER(10, (cpu_elo - player_elo)::NUMERIC / 400.0));
        expected_score := expected_score + expected_vs_opponent;
    END LOOP;

    -- Average expected score across all opponents
    expected_score := expected_score / (total_race_size - 1)::NUMERIC;

    -- Calculate ELO change
    elo_change := ROUND(k_factor * (actual_score - expected_score));

    RETURN elo_change;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Step 4: Process all races in chronological order and recalculate ELO
DO $$
DECLARE
    race_record RECORD;
    player_record RECORD;
    v_current_all_time_elo INTEGER;
    v_current_tournament_elo INTEGER;
    v_all_time_elo_change INTEGER;
    v_tournament_elo_change INTEGER;
    v_teammate_contribution INTEGER;
    v_teammate_id UUID;
BEGIN
    -- Loop through all completed races in chronological order
    FOR race_record IN
        SELECT DISTINCT
            prs.match_id,
            prs.round_number,
            prs.group_id,
            m.tournament_id,
            m.time
        FROM player_race_scores prs
        JOIN matches m ON prs.match_id = m.id
        WHERE prs.position IS NOT NULL
        ORDER BY m.time, prs.round_number
    LOOP
        -- Process each player in this race
        FOR player_record IN
            SELECT
                prs.player_id,
                prs.position,
                p.elo_rating as current_all_time_elo
            FROM player_race_scores prs
            JOIN players p ON prs.player_id = p.id
            WHERE prs.match_id = race_record.match_id
                AND prs.round_number = race_record.round_number
            ORDER BY prs.position
        LOOP
            -- Get current ELO ratings
            v_current_all_time_elo := player_record.current_all_time_elo;

            -- Get or create tournament score record
            INSERT INTO player_tournament_scores (player_id, tournament_id, group_id, elo_rating, created_at, updated_at)
            VALUES (player_record.player_id, race_record.tournament_id, race_record.group_id, 1200, NOW(), NOW())
            ON CONFLICT (player_id, tournament_id) DO NOTHING;

            SELECT elo_rating INTO v_current_tournament_elo
            FROM player_tournament_scores
            WHERE player_id = player_record.player_id
                AND tournament_id = race_record.tournament_id;

            -- Calculate ELO changes based on position
            v_all_time_elo_change := calculate_elo_change(v_current_all_time_elo, player_record.position);
            v_tournament_elo_change := calculate_elo_change(v_current_tournament_elo, player_record.position);

            -- Update player's all-time ELO
            UPDATE players
            SET elo_rating = elo_rating + v_all_time_elo_change
            WHERE id = player_record.player_id;

            -- Update player's tournament ELO
            UPDATE player_tournament_scores
            SET elo_rating = elo_rating + v_tournament_elo_change,
                updated_at = NOW()
            WHERE player_id = player_record.player_id
                AND tournament_id = race_record.tournament_id;

            -- Update race score record
            UPDATE player_race_scores
            SET all_time_elo_change = v_all_time_elo_change,
                all_time_elo_after = v_current_all_time_elo + v_all_time_elo_change,
                tournament_elo_change = v_tournament_elo_change,
                tournament_elo_after = v_current_tournament_elo + v_tournament_elo_change
            WHERE match_id = race_record.match_id
                AND round_number = race_record.round_number
                AND player_id = player_record.player_id;
        END LOOP;

        -- Calculate 20% teammate contributions for this race
        FOR player_record IN
            SELECT
                prs.player_id,
                prs.tournament_elo_change,
                tp.team_id
            FROM player_race_scores prs
            JOIN team_players tp ON tp.player_id = prs.player_id
            JOIN teams t ON t.id = tp.team_id AND t.match_id = race_record.match_id
            WHERE prs.match_id = race_record.match_id
                AND prs.round_number = race_record.round_number
                AND prs.tournament_elo_change IS NOT NULL
        LOOP
            -- Find teammates (players on same team, different player)
            FOR v_teammate_id IN
                SELECT DISTINCT tp2.player_id
                FROM team_players tp1
                JOIN teams t ON tp1.team_id = t.id
                JOIN team_players tp2 ON tp1.team_id = tp2.team_id
                WHERE t.match_id = race_record.match_id
                    AND tp1.player_id = player_record.player_id
                    AND tp2.player_id != player_record.player_id
            LOOP
                -- 20% of tournament ELO change goes to teammates
                v_teammate_contribution := ROUND(player_record.tournament_elo_change * 0.2);

                -- Record the contribution
                INSERT INTO player_teammate_elo_contributions (
                    match_id,
                    round_number,
                    source_player_id,
                    beneficiary_player_id,
                    source_tournament_elo_change,
                    contribution_amount,
                    created_at
                ) VALUES (
                    race_record.match_id,
                    race_record.round_number,
                    player_record.player_id,
                    v_teammate_id,
                    player_record.tournament_elo_change,
                    v_teammate_contribution,
                    NOW()
                );

                -- Apply contribution to teammate's tournament ELO
                UPDATE player_tournament_scores
                SET elo_rating = elo_rating + v_teammate_contribution,
                    updated_at = NOW()
                WHERE player_id = v_teammate_id
                    AND tournament_id = race_record.tournament_id;
            END LOOP;
        END LOOP;
    END LOOP;
END $$;

-- Step 5: Recalculate player_match_scores with proper breakdown
DO $$
DECLARE
    match_record RECORD;
    player_record RECORD;
    v_total_all_time_elo_change INTEGER;
    v_total_tournament_elo_from_races INTEGER;
    v_total_tournament_elo_from_contributions INTEGER;
    v_total_tournament_elo_change INTEGER;
    v_avg_position NUMERIC;
BEGIN
    FOR match_record IN
        SELECT DISTINCT match_id FROM player_race_scores
    LOOP
        FOR player_record IN
            SELECT DISTINCT player_id
            FROM player_match_scores
            WHERE match_id = match_record.match_id
        LOOP
            -- Calculate total all-time ELO change for this match
            SELECT COALESCE(SUM(all_time_elo_change), 0)
            INTO v_total_all_time_elo_change
            FROM player_race_scores
            WHERE match_id = match_record.match_id
                AND player_id = player_record.player_id;

            -- Calculate total tournament ELO change from races
            SELECT COALESCE(SUM(tournament_elo_change), 0)
            INTO v_total_tournament_elo_from_races
            FROM player_race_scores
            WHERE match_id = match_record.match_id
                AND player_id = player_record.player_id;

            -- Calculate total teammate contributions received
            SELECT COALESCE(SUM(contribution_amount), 0)
            INTO v_total_tournament_elo_from_contributions
            FROM player_teammate_elo_contributions
            WHERE match_id = match_record.match_id
                AND beneficiary_player_id = player_record.player_id;

            -- Calculate total tournament ELO change
            v_total_tournament_elo_change := v_total_tournament_elo_from_races + v_total_tournament_elo_from_contributions;

            -- Calculate average position (or 0 if player didn't race)
            SELECT COALESCE(AVG(position), 0)
            INTO v_avg_position
            FROM player_race_scores
            WHERE match_id = match_record.match_id
                AND player_id = player_record.player_id;

            -- Update match score record with breakdown
            UPDATE player_match_scores
            SET elo_change = v_total_all_time_elo_change,
                tournament_elo_from_races = v_total_tournament_elo_from_races,
                tournament_elo_from_contributions = v_total_tournament_elo_from_contributions,
                tournament_elo_change = v_total_tournament_elo_change,
                position = ROUND(v_avg_position)
            WHERE match_id = match_record.match_id
                AND player_id = player_record.player_id;
        END LOOP;
    END LOOP;
END $$;

-- Step 6: Clean up temporary function
DROP FUNCTION calculate_elo_change(INTEGER, INTEGER);

COMMIT;
