-- Expand the startup-page options (allow profile/matches/advisor) and drop the
-- now-buried 'activity' option. The original CHECK constraint on startup_page can't be
-- altered in place in SQLite, so rebuild app_settings with the new allowed set and fold
-- any stored 'activity' value back to the default 'dashboard'.

CREATE TABLE app_settings_new (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    startup_page TEXT NOT NULL CHECK (startup_page IN ('dashboard', 'profile', 'matches', 'advisor', 'settings')),
    compact_mode INTEGER NOT NULL DEFAULT 0 CHECK (compact_mode IN (0, 1)),
    activity_limit INTEGER NOT NULL DEFAULT 100 CHECK (activity_limit BETWEEN 1 AND 500),
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    auto_accept_enabled INTEGER NOT NULL DEFAULT 1 CHECK (auto_accept_enabled IN (0, 1)),
    auto_pick_enabled INTEGER NOT NULL DEFAULT 0 CHECK (auto_pick_enabled IN (0, 1)),
    auto_pick_champion_id INTEGER CHECK (auto_pick_champion_id IS NULL OR auto_pick_champion_id > 0),
    auto_ban_enabled INTEGER NOT NULL DEFAULT 0 CHECK (auto_ban_enabled IN (0, 1)),
    auto_ban_champion_id INTEGER CHECK (auto_ban_champion_id IS NULL OR auto_ban_champion_id > 0),
    language TEXT NOT NULL DEFAULT 'system' CHECK (language IN ('system', 'zh', 'en')),
    auto_pick_delay_seconds REAL NOT NULL DEFAULT 0.0 CHECK (auto_pick_delay_seconds >= 0.0 AND auto_pick_delay_seconds <= 5.0),
    auto_ban_delay_seconds REAL NOT NULL DEFAULT 0.0 CHECK (auto_ban_delay_seconds >= 0.0 AND auto_ban_delay_seconds <= 5.0),
    theme TEXT NOT NULL DEFAULT 'light' CHECK (theme IN ('light', 'dark')),
    ai_base_url TEXT,
    ai_api_key TEXT,
    ai_model TEXT
);

INSERT INTO app_settings_new (
    id, startup_page, compact_mode, activity_limit, updated_at,
    auto_accept_enabled, auto_pick_enabled, auto_pick_champion_id, auto_ban_enabled, auto_ban_champion_id,
    language, auto_pick_delay_seconds, auto_ban_delay_seconds, theme, ai_base_url, ai_api_key, ai_model
)
SELECT
    id,
    CASE WHEN startup_page = 'activity' THEN 'dashboard' ELSE startup_page END,
    compact_mode, activity_limit, updated_at,
    auto_accept_enabled, auto_pick_enabled, auto_pick_champion_id, auto_ban_enabled, auto_ban_champion_id,
    language, auto_pick_delay_seconds, auto_ban_delay_seconds, theme, ai_base_url, ai_api_key, ai_model
FROM app_settings;

DROP TABLE app_settings;
ALTER TABLE app_settings_new RENAME TO app_settings;

UPDATE app_metadata
SET value = '13',
    updated_at = CURRENT_TIMESTAMP
WHERE key = 'schema_version';
