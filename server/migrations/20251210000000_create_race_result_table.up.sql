-- Create enum type for race result status
CREATE TYPE race_result_status AS ENUM (
    'FINISHED',
    'DNF'
);

-- Create race_result table to store final race results for each car
CREATE TABLE race_result (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    race_id UUID NOT NULL REFERENCES race(id) ON DELETE CASCADE,
    car_id UUID NOT NULL REFERENCES car(id) ON DELETE CASCADE,
    driver_id UUID NOT NULL REFERENCES driver(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES team(id) ON DELETE CASCADE,
    car_number INTEGER NOT NULL,
    final_position INTEGER NOT NULL,
    race_time_seconds REAL NOT NULL,
    status race_result_status NOT NULL,
    laps_completed INTEGER NOT NULL,
    total_distance_km REAL NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Ensure one result per car per race
    UNIQUE(race_id, car_id)
);

-- Create indexes for better query performance
CREATE INDEX idx_race_result_race_id ON race_result(race_id);
CREATE INDEX idx_race_result_car_id ON race_result(car_id);
CREATE INDEX idx_race_result_driver_id ON race_result(driver_id);
CREATE INDEX idx_race_result_team_id ON race_result(team_id);
CREATE INDEX idx_race_result_final_position ON race_result(race_id, final_position);

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_race_result_updated_at BEFORE UPDATE ON race_result
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

