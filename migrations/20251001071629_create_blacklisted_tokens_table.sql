-- Create blacklisted_tokens table for JWT token management
CREATE TABLE blacklisted_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jti TEXT NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_type TEXT NOT NULL CHECK (token_type IN ('access', 'refresh')),
    expires_at TIMESTAMPTZ NOT NULL,
    blacklisted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for efficient lookups
CREATE INDEX idx_blacklisted_tokens_jti ON blacklisted_tokens(jti);
CREATE INDEX idx_blacklisted_tokens_user_id ON blacklisted_tokens(user_id);
CREATE INDEX idx_blacklisted_tokens_expires_at ON blacklisted_tokens(expires_at);
CREATE INDEX idx_blacklisted_tokens_blacklisted_at ON blacklisted_tokens(blacklisted_at);

-- Add trigger for updated_at if needed
-- (Since this table mainly deals with immutable blacklist entries, we might not need updated_at)