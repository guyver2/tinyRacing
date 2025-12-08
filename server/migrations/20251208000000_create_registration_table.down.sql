-- Drop trigger
DROP TRIGGER IF EXISTS update_registration_updated_at ON registration;

-- Drop indexes
DROP INDEX IF EXISTS idx_registration_team_id;
DROP INDEX IF EXISTS idx_registration_race_id;

-- Drop registration table
DROP TABLE IF EXISTS registration;

