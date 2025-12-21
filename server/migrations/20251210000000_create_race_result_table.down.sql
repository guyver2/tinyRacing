-- Drop trigger
DROP TRIGGER IF EXISTS update_race_result_updated_at ON race_result;

-- Drop indexes
DROP INDEX IF EXISTS idx_race_result_final_position;
DROP INDEX IF EXISTS idx_race_result_team_id;
DROP INDEX IF EXISTS idx_race_result_driver_id;
DROP INDEX IF EXISTS idx_race_result_car_id;
DROP INDEX IF EXISTS idx_race_result_race_id;

-- Drop table
DROP TABLE IF EXISTS race_result;

-- Drop enum type
DROP TYPE IF EXISTS race_result_status;

