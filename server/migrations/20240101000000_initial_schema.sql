-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Teams table
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    number INTEGER NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    logo TEXT NOT NULL,
    color VARCHAR(50) NOT NULL,
    pit_efficiency REAL NOT NULL CHECK (pit_efficiency >= 0.0 AND pit_efficiency <= 1.0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Drivers table
CREATE TABLE drivers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    skill_level REAL NOT NULL CHECK (skill_level >= 0.0 AND skill_level <= 1.0),
    stamina REAL NOT NULL CHECK (stamina >= 0.0 AND stamina <= 1.0),
    weather_tolerance REAL NOT NULL CHECK (weather_tolerance >= 0.0 AND weather_tolerance <= 1.0),
    experience REAL NOT NULL CHECK (experience >= 0.0 AND experience <= 1.0),
    consistency REAL NOT NULL CHECK (consistency >= 0.0 AND consistency <= 1.0),
    focus REAL NOT NULL CHECK (focus >= 0.0 AND focus <= 1.0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tracks table
CREATE TABLE tracks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    track_id VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    laps INTEGER NOT NULL CHECK (laps > 0),
    lap_length_km REAL NOT NULL CHECK (lap_length_km > 0),
    svg_start_offset REAL NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Players table
CREATE TABLE players (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cars table
CREATE TABLE cars (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    number INTEGER NOT NULL UNIQUE,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    driver_id UUID NOT NULL REFERENCES drivers(id) ON DELETE CASCADE,
    handling REAL NOT NULL CHECK (handling >= 0.0 AND handling <= 1.0),
    acceleration REAL NOT NULL CHECK (acceleration >= 0.0 AND acceleration <= 1.0),
    top_speed REAL NOT NULL CHECK (top_speed >= 0.0 AND top_speed <= 1.0),
    reliability REAL NOT NULL CHECK (reliability >= 0.0 AND reliability <= 1.0),
    fuel_consumption REAL NOT NULL CHECK (fuel_consumption >= 0.0 AND fuel_consumption <= 1.0),
    tire_wear REAL NOT NULL CHECK (tire_wear >= 0.0 AND tire_wear <= 1.0),
    base_performance REAL NOT NULL CHECK (base_performance >= 0.9 AND base_performance <= 1.1),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX idx_cars_team_id ON cars(team_id);
CREATE INDEX idx_cars_driver_id ON cars(driver_id);
CREATE INDEX idx_cars_number ON cars(number);
CREATE INDEX idx_teams_number ON teams(number);
CREATE INDEX idx_drivers_name ON drivers(name);
CREATE INDEX idx_tracks_track_id ON tracks(track_id);
CREATE INDEX idx_players_username ON players(username);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers to automatically update updated_at
CREATE TRIGGER update_teams_updated_at BEFORE UPDATE ON teams
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_drivers_updated_at BEFORE UPDATE ON drivers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_cars_updated_at BEFORE UPDATE ON cars
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tracks_updated_at BEFORE UPDATE ON tracks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_players_updated_at BEFORE UPDATE ON players
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

