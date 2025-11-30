-- Remove password_hash column from player table
ALTER TABLE player
DROP COLUMN password_hash;
