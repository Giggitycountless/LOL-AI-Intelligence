CREATE TABLE IF NOT EXISTS advisor_data_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,
    patch TEXT,
    region TEXT,
    queue TEXT,
    tier TEXT,
    generated_at TEXT,
    imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    payload_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_advisor_data_snapshots_imported_at
ON advisor_data_snapshots(imported_at DESC, id DESC);

UPDATE app_metadata
SET value = '7',
    updated_at = CURRENT_TIMESTAMP
WHERE key = 'schema_version';
