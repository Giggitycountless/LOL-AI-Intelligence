pub(crate) const DATABASE_FILE_NAME: &str = "app.sqlite";
pub(crate) const MIGRATION_0001: &str = include_str!("../migrations/0001_initial.sql");
pub(crate) const MIGRATION_0002: &str = include_str!("../migrations/0002_state_foundation.sql");
pub(crate) const MIGRATION_0003: &str = include_str!("../migrations/0003_player_notes.sql");
pub(crate) const MIGRATION_0004: &str =
    include_str!("../migrations/0004_ranked_champion_cache.sql");
pub(crate) const MIGRATION_0005: &str =
    include_str!("../migrations/0005_lobby_automation_settings.sql");
pub(crate) const MIGRATION_0006: &str = include_str!("../migrations/0006_language_preference.sql");
pub(crate) const MIGRATION_0007: &str = include_str!("../migrations/0007_advisor_data_cache.sql");
pub(crate) const MIGRATION_0008: &str = include_str!("../migrations/0008_pick_ban_delay.sql");
pub(crate) const MIGRATION_0009: &str = include_str!("../migrations/0009_champion_rune_configs.sql");
pub(crate) const MIGRATION_0010: &str = include_str!("../migrations/0010_theme_preference.sql");
pub(crate) const MIGRATION_0011: &str = include_str!("../migrations/0011_ai_config.sql");
pub(crate) const MIGRATION_0012: &str = include_str!("../migrations/0012_chat_presets.sql");
pub(crate) const MIGRATION_0013: &str =
    include_str!("../migrations/0013_startup_page_options.sql");
