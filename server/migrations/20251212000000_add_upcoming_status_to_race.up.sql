-- Add UPCOMING status to race_status enum
-- Note: IF NOT EXISTS is not supported for ALTER TYPE ADD VALUE in older PostgreSQL versions
-- This will fail if the value already exists, which is expected behavior for migrations
DO $$ BEGIN
    ALTER TYPE race_status ADD VALUE 'UPCOMING';
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

