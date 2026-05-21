CREATE TABLE champion_rune_configs (
    champion_id INTEGER PRIMARY KEY CHECK (champion_id > 0),
    primary_style_id INTEGER NOT NULL,
    sub_style_id INTEGER NOT NULL,
    selected_perk_ids_json TEXT NOT NULL,
    saved_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

UPDATE app_metadata
SET value = '9',
    updated_at = CURRENT_TIMESTAMP
WHERE key = 'schema_version';
