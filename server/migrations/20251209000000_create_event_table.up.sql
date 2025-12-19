CREATE TYPE event_type AS ENUM (
    'START_RACE',
    'END_RACE',
    'PIT_REQUEST',
    'PIT_CANCEL',
    'PIT_STOP',
    'WEATHER_CHANGE',
    'ACCIDENT',
    'CAR_FINISHED',
    'DNF',
    'OTHER'
);

-- Create event table to store race events
CREATE TABLE event (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    race_id UUID NOT NULL REFERENCES race(id) ON DELETE CASCADE,
    event_type event_type NOT NULL,
    description TEXT DEFAULT NULL,
    time_offset_seconds REAL DEFAULT 0.0,
    -- Event data fields (all optional)
    car_number INTEGER DEFAULT 0,
    car_id UUID REFERENCES car(id) ON DELETE SET NULL,
    team_id UUID REFERENCES team(id) ON DELETE SET NULL,
    driver_id UUID REFERENCES driver(id) ON DELETE SET NULL,
    tire TEXT DEFAULT NULL,
    fuel REAL DEFAULT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX idx_event_race_id ON event(race_id);
CREATE INDEX idx_event_car_id ON event(car_id);
CREATE INDEX idx_event_team_id ON event(team_id);
CREATE INDEX idx_event_driver_id ON event(driver_id);
CREATE INDEX idx_event_event_type ON event(event_type);
CREATE INDEX idx_event_time_offset ON event(time_offset_seconds);

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_event_updated_at BEFORE UPDATE ON event
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

