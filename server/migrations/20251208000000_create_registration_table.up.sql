-- Create registration table
CREATE TABLE registration (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    race_id UUID NOT NULL REFERENCES race(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES team(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Ensure a team can only register once per race
    UNIQUE(race_id, team_id)
);

-- Create indexes for better query performance
CREATE INDEX idx_registration_race_id ON registration(race_id);
CREATE INDEX idx_registration_team_id ON registration(team_id);

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_registration_updated_at BEFORE UPDATE ON registration
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

