-- Create login_attempts table for security logging
CREATE TABLE login_attempts (
    id UUID PRIMARY KEY,
    ip_address VARCHAR(45) NOT NULL,
    email VARCHAR(255) NOT NULL,
    user_id UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    success BOOLEAN NOT NULL,
    failure_reason TEXT NULL,
    user_agent TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient queries
CREATE INDEX idx_login_attempts_ip_success_created ON login_attempts(ip_address, success, created_at);
CREATE INDEX idx_login_attempts_email_success_created ON login_attempts(email, success, created_at);
CREATE INDEX idx_login_attempts_created_at ON login_attempts(created_at);
CREATE INDEX idx_login_attempts_user_id ON login_attempts(user_id) WHERE user_id IS NOT NULL;