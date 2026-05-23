CREATE TABLE chat_presets (
    slot INTEGER PRIMARY KEY CHECK (slot BETWEEN 1 AND 9),
    label TEXT NOT NULL,
    message TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

UPDATE app_metadata
SET value = '12',
    updated_at = CURRENT_TIMESTAMP
WHERE key = 'schema_version';
