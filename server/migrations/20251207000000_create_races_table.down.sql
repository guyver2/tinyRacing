-- Drop trigger
DROP TRIGGER IF EXISTS update_race_updated_at ON race;

-- Drop indexes
DROP INDEX IF EXISTS idx_race_start_datetime;
DROP INDEX IF EXISTS idx_race_status;
DROP INDEX IF EXISTS idx_race_creator_id;
DROP INDEX IF EXISTS idx_race_track_id;

-- Drop races table
DROP TABLE IF EXISTS race;

-- Drop enum type
DROP TYPE IF EXISTS race_status;

