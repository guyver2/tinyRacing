-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Players table
CREATE TABLE player (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- Teams table
CREATE TABLE team (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    number INTEGER NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    logo TEXT NOT NULL,
    color VARCHAR(50) NOT NULL,
    pit_efficiency REAL NOT NULL CHECK (pit_efficiency >= 0.0 AND pit_efficiency <= 1.0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tracks table
CREATE TABLE track (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    track_id VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    laps INTEGER NOT NULL CHECK (laps > 0) DEFAULT 10,
    lap_length_km REAL NOT NULL CHECK (lap_length_km > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- Cars table
CREATE TABLE car (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    number INTEGER NOT NULL UNIQUE,
    team_id UUID REFERENCES team(id) DEFAULT NULL,
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


-- Drivers table
CREATE TABLE driver (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    date_of_birth DATE NOT NULL,
    nationality VARCHAR(255) NOT NULL,
    gender VARCHAR(255) NOT NULL,
    skill_level REAL NOT NULL CHECK (skill_level >= 0.0 AND skill_level <= 1.0),
    stamina REAL NOT NULL CHECK (stamina >= 0.0 AND stamina <= 1.0),
    weather_tolerance REAL NOT NULL CHECK (weather_tolerance >= 0.0 AND weather_tolerance <= 1.0),
    experience REAL NOT NULL CHECK (experience >= 0.0 AND experience <= 1.0),
    consistency REAL NOT NULL CHECK (consistency >= 0.0 AND consistency <= 1.0),
    focus REAL NOT NULL CHECK (focus >= 0.0 AND focus <= 1.0),
    team_id UUID REFERENCES team(id)  default NULL,
    car_id UUID REFERENCES car(id)  default NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- Create indexes for better query performance
CREATE INDEX idx_car_team_id ON car(team_id);
CREATE INDEX idx_car_number ON car(number);
CREATE INDEX idx_team_number ON team(number);
CREATE INDEX idx_track_track_id ON track(track_id);
CREATE INDEX idx_player_username ON player(username);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers to automatically update updated_at
CREATE TRIGGER update_team_updated_at BEFORE UPDATE ON team
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_driver_updated_at BEFORE UPDATE ON driver
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_car_updated_at BEFORE UPDATE ON car
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_track_updated_at BEFORE UPDATE ON track
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_player_updated_at BEFORE UPDATE ON player
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

