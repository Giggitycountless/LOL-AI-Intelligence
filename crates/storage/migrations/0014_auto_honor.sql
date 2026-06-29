ALTER TABLE app_settings ADD COLUMN auto_honor_enabled INTEGER NOT NULL DEFAULT 0 CHECK (auto_honor_enabled IN (0, 1));

UPDATE app_metadata
SET value = '14',
    updated_at = CURRENT_TIMESTAMP
WHERE key = 'schema_version';
