-- Drop trigger
DROP TRIGGER IF EXISTS update_event_updated_at ON event;

-- Drop indexes
DROP INDEX IF EXISTS idx_event_time_offset;
DROP INDEX IF EXISTS idx_event_event_type;
DROP INDEX IF EXISTS idx_event_driver_id;
DROP INDEX IF EXISTS idx_event_team_id;
DROP INDEX IF EXISTS idx_event_car_id;
DROP INDEX IF EXISTS idx_event_race_id;

-- Drop event table
DROP TABLE IF EXISTS event;

-- Drop event type enum
DROP TYPE IF EXISTS event_type;
