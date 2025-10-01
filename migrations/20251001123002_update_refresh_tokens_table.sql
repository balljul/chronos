-- Add missing fields to refresh_tokens table for better tracking
ALTER TABLE refresh_tokens
ADD COLUMN jti VARCHAR(255) NOT NULL DEFAULT gen_random_uuid()::text,
ADD COLUMN revoked_at TIMESTAMPTZ NULL,
ADD COLUMN last_used_at TIMESTAMPTZ NULL;

-- Create unique index on JTI for token lookups
CREATE UNIQUE INDEX idx_refresh_tokens_jti ON refresh_tokens(jti);

-- Create index for revoked token queries
CREATE INDEX idx_refresh_tokens_revoked_at ON refresh_tokens(revoked_at) WHERE revoked_at IS NOT NULL;

-- Create index for user token cleanup
CREATE INDEX idx_refresh_tokens_user_id_expires ON refresh_tokens(user_id, expires_at);