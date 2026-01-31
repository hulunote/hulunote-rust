-- =====================================================
-- Migration: Add Registration Codes System
-- =====================================================

-- Create registration_codes table
CREATE TABLE IF NOT EXISTS registration_codes (
    id BIGSERIAL PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    validity_days INTEGER NOT NULL,  -- How many days the user account is valid for
    is_used BOOLEAN NOT NULL DEFAULT false,
    used_by_account_id BIGINT,
    used_at TIMESTAMP(6) WITH TIME ZONE,
    created_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT now(),
    FOREIGN KEY (used_by_account_id) REFERENCES accounts(id)
);

-- Add index on code for fast lookups
CREATE INDEX IF NOT EXISTS idx_registration_codes_code ON registration_codes(code);
CREATE INDEX IF NOT EXISTS idx_registration_codes_is_used ON registration_codes(is_used);

-- Add expires_at column to accounts table
ALTER TABLE accounts
ADD COLUMN IF NOT EXISTS expires_at TIMESTAMP(6) WITH TIME ZONE;

-- Add registration_code column to accounts table to track which code was used
ALTER TABLE accounts
ADD COLUMN IF NOT EXISTS registration_code TEXT;

-- =====================================================
-- Comment on tables and columns
-- =====================================================
COMMENT ON TABLE registration_codes IS 'Stores registration codes with validity periods';
COMMENT ON COLUMN registration_codes.code IS 'Unique registration code string';
COMMENT ON COLUMN registration_codes.validity_days IS 'Number of days the account is valid after registration';
COMMENT ON COLUMN registration_codes.is_used IS 'Whether this code has been used';
COMMENT ON COLUMN registration_codes.used_by_account_id IS 'ID of the account that used this code';
COMMENT ON COLUMN registration_codes.used_at IS 'When the code was used';

COMMENT ON COLUMN accounts.expires_at IS 'When the user account expires (NULL means never expires)';
COMMENT ON COLUMN accounts.registration_code IS 'The registration code used to create this account';
