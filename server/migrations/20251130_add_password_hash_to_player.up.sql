-- Add password_hash column to player table
ALTER TABLE player
ADD COLUMN password_hash VARCHAR(255) NOT NULL DEFAULT '';
