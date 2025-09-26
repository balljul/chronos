-- Migration: Fix password hash constraint issue
-- Created: 2025-09-26
-- Description: Investigate and fix password hash length constraint

-- First, let's check the current password hash lengths in the database
-- Run this to see what lengths we're dealing with:

SELECT 
    id,
    email,
    LENGTH(password_hash) as hash_length,
    LEFT(password_hash, 10) as hash_prefix
FROM users 
ORDER BY LENGTH(password_hash);

-- Check the distribution of hash lengths
SELECT 
    LENGTH(password_hash) as hash_length,
    COUNT(*) as count
FROM users 
GROUP BY LENGTH(password_hash)
ORDER BY hash_length;

-- Find any passwords that would violate our constraint
SELECT 
    id,
    email,
    LENGTH(password_hash) as hash_length,
    password_hash
FROM users 
WHERE LENGTH(password_hash) < 60;

-- ========================================
-- SOLUTION 1: Adjust constraint to match existing data
-- ========================================

-- Drop the problematic constraint if it exists
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_users_password_hash_length;

-- Add a more realistic constraint based on actual data
-- bcrypt hashes are typically 59-60 characters, but let's be more flexible
ALTER TABLE users 
ADD CONSTRAINT chk_users_password_hash_length 
CHECK (LENGTH(password_hash) >= 50); -- More realistic minimum

-- ========================================
-- SOLUTION 2: Alternative - Update short hashes
-- ========================================

-- If you want to keep the 60-character minimum, you could update short hashes
-- WARNING: This would invalidate those passwords and users would need to reset them

-- First, create a backup of users with short hashes
CREATE TABLE IF NOT EXISTS users_short_hash_backup AS
SELECT * FROM users WHERE LENGTH(password_hash) < 60;

-- Then you could either:
-- Option A: Set these users as unverified and requiring password reset
-- UPDATE users 
-- SET is_verified = FALSE,
--     password_hash = '$2b$12$' || REPEAT('x', 47) -- Placeholder hash, forces reset
-- WHERE LENGTH(password_hash) < 60;

-- Option B: Delete users with invalid hashes (DANGEROUS - only if test data)
-- DELETE FROM users WHERE LENGTH(password_hash) < 60;

-- ========================================
-- SOLUTION 3: More comprehensive password validation
-- ========================================

-- Drop the simple length constraint
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_users_password_hash_length;

-- Add a more sophisticated constraint that validates bcrypt format
ALTER TABLE users 
ADD CONSTRAINT chk_users_password_hash_format 
CHECK (
    password_hash ~ '^\$2[abxy]?\$[0-9]{2}\$.{53}$' -- bcrypt format validation
    OR 
    password_hash ~ '^\$argon2[id]?\$.*' -- argon2 format
    OR 
    LENGTH(password_hash) >= 32 -- fallback for other hash types
);

-- ========================================
-- INVESTIGATION QUERIES
-- ========================================

-- Run these queries to understand your data better:

-- 1. Show all password hash formats and lengths
-- SELECT 
--     SUBSTRING(password_hash FROM 1 FOR 7) as hash_type,
--     LENGTH(password_hash) as length,
--     COUNT(*) as count
-- FROM users 
-- GROUP BY SUBSTRING(password_hash FROM 1 FOR 7), LENGTH(password_hash)
-- ORDER BY hash_type, length;

-- 2. Validate bcrypt format for all users
-- SELECT 
--     id,
--     email,
--     CASE 
--         WHEN password_hash ~ '^\$2[abxy]?\$[0-9]{2}\$.{53}$' THEN 'Valid bcrypt'
--         WHEN LENGTH(password_hash) < 50 THEN 'Too short'
--         ELSE 'Unknown format'
--     END as validation_status,
--     LENGTH(password_hash) as hash_length
-- FROM users
-- ORDER BY validation_status, hash_length;

-- ========================================
-- RECOMMENDED APPROACH
-- ========================================

-- Based on your sample data, I recommend using the more flexible constraint:
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_users_password_hash_length;
ALTER TABLE users 
ADD CONSTRAINT chk_users_password_hash_min_length 
CHECK (LENGTH(password_hash) >= 32 AND LENGTH(TRIM(password_hash)) > 0);

-- Add a comment explaining the constraint
COMMENT ON CONSTRAINT chk_users_password_hash_min_length ON users IS 
'Ensures password hash has minimum length of 32 characters and is not empty. Accommodates various hash formats (bcrypt, argon2, etc.)';

-- ========================================
-- FUTURE IMPROVEMENTS
-- ========================================

-- Consider adding application-level validation for new passwords:
-- - Minimum 8 characters
-- - At least one uppercase, lowercase, number
-- - Common password checking
-- But enforce only basic requirements at DB level for flexibility
