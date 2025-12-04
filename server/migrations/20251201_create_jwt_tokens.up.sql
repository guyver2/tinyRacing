-- Create JWT tokens table to store active tokens with expiry dates
CREATE TABLE jwt_token (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    player_id UUID NOT NULL REFERENCES player(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on player_id for faster lookups
CREATE INDEX idx_jwt_token_player_id ON jwt_token(player_id);

-- Create index on token for faster validation
CREATE INDEX idx_jwt_token_token ON jwt_token(token);

-- Create index on expires_at for cleanup queries
CREATE INDEX idx_jwt_token_expires_at ON jwt_token(expires_at);

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_jwt_token_updated_at BEFORE UPDATE ON jwt_token
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

