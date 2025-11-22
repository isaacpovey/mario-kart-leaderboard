-- Drop the team_match_scores table as it's no longer used
-- Team scores are now only stored in the teams.score column
DROP TABLE IF EXISTS team_match_scores CASCADE;
