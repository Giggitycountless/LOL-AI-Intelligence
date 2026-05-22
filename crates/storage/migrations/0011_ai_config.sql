ALTER TABLE app_settings ADD COLUMN ai_base_url TEXT;
ALTER TABLE app_settings ADD COLUMN ai_api_key TEXT;
ALTER TABLE app_settings ADD COLUMN ai_model TEXT;

CREATE TABLE ai_analysis_cache (
    scope TEXT PRIMARY KEY CHECK (scope IN ('all', 'top', 'jungle', 'middle', 'bottom', 'support')),
    result_text TEXT NOT NULL,
    game_count_at_analysis INTEGER NOT NULL DEFAULT 0,
    analyzed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

UPDATE app_metadata
SET value = '11',
    updated_at = CURRENT_TIMESTAMP
WHERE key = 'schema_version';
