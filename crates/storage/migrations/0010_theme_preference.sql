ALTER TABLE app_settings ADD COLUMN theme TEXT NOT NULL DEFAULT 'light' CHECK (theme IN ('light', 'dark'));

UPDATE app_metadata
SET value = '10',
    updated_at = CURRENT_TIMESTAMP
WHERE key = 'schema_version';
