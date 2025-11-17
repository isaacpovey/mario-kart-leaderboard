-- Fix player_match_scores by recalculating from source data
-- This migration corrects the double-counting bug where teammate contributions
-- were being accumulated incorrectly across rounds.

UPDATE player_match_scores pms
SET
    elo_change = COALESCE((
        SELECT SUM(all_time_elo_change)
        FROM player_race_scores
        WHERE match_id = pms.match_id
          AND player_id = pms.player_id
    ), 0),
    tournament_elo_change = COALESCE((
        SELECT SUM(tournament_elo_change)
        FROM player_race_scores
        WHERE match_id = pms.match_id
          AND player_id = pms.player_id
    ), 0) + COALESCE((
        SELECT SUM(contribution_amount)
        FROM player_teammate_elo_contributions
        WHERE match_id = pms.match_id
          AND beneficiary_player_id = pms.player_id
    ), 0);
