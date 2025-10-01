-- Create account_lockouts table for brute force protection
CREATE TABLE account_lockouts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    locked_until TIMESTAMPTZ NOT NULL,
    failed_attempts INTEGER NOT NULL,
    locked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    unlocked_at TIMESTAMPTZ NULL
);

-- Create indexes for efficient lockout checks
CREATE INDEX idx_account_lockouts_user_id_active ON account_lockouts(user_id) WHERE unlocked_at IS NULL;
CREATE INDEX idx_account_lockouts_locked_until ON account_lockouts(locked_until);
CREATE INDEX idx_account_lockouts_locked_at ON account_lockouts(locked_at);