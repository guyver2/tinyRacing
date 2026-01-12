-- Note: PostgreSQL does not support removing enum values directly
-- This migration cannot be fully reversed without recreating the enum type
-- If rollback is needed, the enum would need to be recreated without UPCOMING
-- For now, this is a no-op as removing enum values requires more complex operations

