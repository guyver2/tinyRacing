-- Create enum type for race status
CREATE TYPE race_status AS ENUM (
    'REGISTRATION_OPEN',
    'REGISTRATION_CLOSED',
    'FINISHED',
    'ONGOING',
    'CANCELED'
);

-- Create races table
CREATE TABLE race (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    track_id UUID NOT NULL REFERENCES track(id),
    laps INTEGER NOT NULL CHECK (laps > 0),
    status race_status NOT NULL DEFAULT 'REGISTRATION_OPEN',
    start_datetime TIMESTAMPTZ DEFAULT NULL,
    creator_id UUID REFERENCES player(id) DEFAULT NULL,
    description TEXT DEFAULT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX idx_race_track_id ON race(track_id);
CREATE INDEX idx_race_creator_id ON race(creator_id);
CREATE INDEX idx_race_status ON race(status);
CREATE INDEX idx_race_start_datetime ON race(start_datetime);

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_race_updated_at BEFORE UPDATE ON race
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

