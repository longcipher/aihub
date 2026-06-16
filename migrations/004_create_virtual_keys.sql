-- Migration: 004_create_virtual_keys.sql

-- Virtual keys table
CREATE TABLE IF NOT EXISTS virtual_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    allowed_models TEXT[] NOT NULL DEFAULT '{}',
    denied_models TEXT[] NOT NULL DEFAULT '{}',
    rpm_limit INTEGER,
    tpm_limit INTEGER,
    monthly_budget_cents BIGINT,
    budget_mode TEXT NOT NULL DEFAULT 'hard',
    provider_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast key lookup
CREATE INDEX IF NOT EXISTS idx_virtual_keys_key_hash ON virtual_keys(key_hash);

-- Index for listing enabled keys
CREATE INDEX IF NOT EXISTS idx_virtual_keys_enabled ON virtual_keys(enabled);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to automatically update updated_at
CREATE TRIGGER update_virtual_keys_updated_at
    BEFORE UPDATE ON virtual_keys
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
