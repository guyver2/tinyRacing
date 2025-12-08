-- Add player_id column to team table to track team ownership
ALTER TABLE team ADD COLUMN player_id UUID REFERENCES player(id) DEFAULT NULL;

-- Create index for better query performance on player_id
CREATE INDEX idx_team_player_id ON team(player_id);

