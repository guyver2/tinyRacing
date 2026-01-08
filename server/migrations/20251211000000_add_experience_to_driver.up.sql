-- Add total_exp and spent_exp columns to driver table with default value of 0
ALTER TABLE driver ADD COLUMN total_exp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE driver ADD COLUMN spent_exp INTEGER NOT NULL DEFAULT 0;

