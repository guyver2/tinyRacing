-- Drop the index first
DROP INDEX IF EXISTS idx_team_player_id;

-- Remove player_id column from team table
ALTER TABLE team DROP COLUMN player_id;

