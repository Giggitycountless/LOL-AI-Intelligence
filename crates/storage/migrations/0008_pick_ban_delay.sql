ALTER TABLE app_settings ADD COLUMN auto_pick_delay_seconds REAL NOT NULL DEFAULT 0.0 CHECK (auto_pick_delay_seconds >= 0.0 AND auto_pick_delay_seconds <= 5.0);
ALTER TABLE app_settings ADD COLUMN auto_ban_delay_seconds REAL NOT NULL DEFAULT 0.0 CHECK (auto_ban_delay_seconds >= 0.0 AND auto_ban_delay_seconds <= 5.0);

UPDATE app_metadata
SET value = '8',
    updated_at = CURRENT_TIMESTAMP
WHERE key = 'schema_version';
