use tauri::{Emitter, Manager, State};

const SELF_HISTORY_OVERLAY_WINDOW_LABEL: &str = "self-history-overlay";

#[tauri::command]
fn healthcheck(state: State<'_, platform::AppState>) -> domain::HealthReport {
    platform::healthcheck(state.inner())
}

#[tauri::command]
fn get_app_state(
    state: State<'_, platform::AppState>,
) -> Result<domain::AppSnapshot, platform::CommandError> {
    platform::get_app_state(state.inner())
}

#[tauri::command]
fn get_settings(
    state: State<'_, platform::AppState>,
) -> Result<domain::AppSettings, platform::CommandError> {
    platform::get_settings(state.inner())
}

#[tauri::command]
fn get_settings_defaults() -> domain::SettingsValues {
    platform::get_settings_defaults()
}

#[tauri::command]
fn save_settings(
    state: State<'_, platform::AppState>,
    input: platform::SaveSettingsCommand,
) -> Result<domain::AppSettings, platform::CommandError> {
    platform::save_settings(state.inner(), input)
}

#[tauri::command]
fn list_activity_entries(
    state: State<'_, platform::AppState>,
    input: platform::ListActivityEntriesCommand,
) -> Result<platform::ActivityEntriesResponse, platform::CommandError> {
    platform::list_activity_entries(state.inner(), input)
}

#[tauri::command]
fn create_activity_note(
    state: State<'_, platform::AppState>,
    input: platform::CreateActivityNoteCommand,
) -> Result<domain::ActivityEntry, platform::CommandError> {
    platform::create_activity_note(state.inner(), input)
}

#[tauri::command]
fn export_local_data(
    state: State<'_, platform::AppState>,
) -> Result<domain::LocalDataExport, platform::CommandError> {
    platform::export_local_data(state.inner())
}

#[tauri::command]
fn import_local_data(
    state: State<'_, platform::AppState>,
    input: platform::ImportLocalDataCommand,
) -> Result<domain::ImportLocalDataResult, platform::CommandError> {
    platform::import_local_data(state.inner(), input)
}

#[tauri::command]
fn clear_activity_entries(
    state: State<'_, platform::AppState>,
    input: platform::ClearActivityEntriesCommand,
) -> Result<domain::ClearActivityResult, platform::CommandError> {
    platform::clear_activity_entries(state.inner(), input)
}

#[tauri::command]
fn get_league_client_status(
    state: State<'_, platform::AppState>,
) -> Result<domain::LeagueClientStatus, platform::CommandError> {
    platform::get_league_client_status(state.inner())
}

#[tauri::command]
fn get_auto_accept_status(state: State<'_, platform::AppState>) -> domain::AutoAcceptStatus {
    platform::get_auto_accept_status(state.inner())
}

#[tauri::command]
fn can_open_self_history_overlay(state: State<'_, platform::AppState>) -> bool {
    platform::can_open_self_history_overlay(state.inner())
}

#[tauri::command]
fn destroy_self_history_overlay_window(app: tauri::AppHandle) {
    platform::destroy_self_history_overlay_window(&app);
}

#[tauri::command]
fn get_league_champion_catalog(
    state: State<'_, platform::AppState>,
) -> Result<Vec<domain::LeagueChampionSummary>, platform::CommandError> {
    platform::get_league_champion_catalog(state.inner())
}

#[tauri::command]
fn get_league_self_snapshot(
    state: State<'_, platform::AppState>,
    input: platform::LeagueSelfSnapshotCommand,
) -> Result<domain::LeagueSelfSnapshot, platform::CommandError> {
    platform::get_league_self_snapshot(state.inner(), input)
}

#[tauri::command]
fn get_champ_select_snapshot(
    state: State<'_, platform::AppState>,
    input: platform::ChampSelectSnapshotCommand,
) -> Result<domain::ChampSelectSnapshot, platform::CommandError> {
    platform::get_champ_select_snapshot(state.inner(), input)
}

#[tauri::command]
fn get_ranked_champion_stats(
    state: State<'_, platform::AppState>,
    input: platform::RankedChampionStatsCommand,
) -> Result<domain::RankedChampionStatsResponse, platform::CommandError> {
    platform::get_ranked_champion_stats(state.inner(), input)
}

#[tauri::command]
fn refresh_ranked_champion_stats(
    state: State<'_, platform::AppState>,
    input: platform::RefreshRankedChampionStatsCommand,
) -> Result<domain::RankedChampionStatsResponse, platform::CommandError> {
    platform::refresh_ranked_champion_stats(state.inner(), input)
}

#[tauri::command]
fn get_advisor_data(
    state: State<'_, platform::AppState>,
    input: platform::AdvisorDataCommand,
) -> Result<domain::AdvisorDataResponse, platform::CommandError> {
    platform::get_advisor_data(state.inner(), input)
}

#[tauri::command]
fn refresh_advisor_data(
    state: State<'_, platform::AppState>,
    input: platform::RefreshAdvisorDataCommand,
) -> Result<domain::AdvisorDataResponse, platform::CommandError> {
    platform::refresh_advisor_data(state.inner(), input)
}

#[tauri::command]
fn get_champ_select_advisor_snapshot(
    state: State<'_, platform::AppState>,
    input: platform::ChampSelectSnapshotCommand,
) -> Result<domain::ChampSelectAdvisorSnapshot, platform::CommandError> {
    platform::get_champ_select_advisor_snapshot(state.inner(), input)
}

#[tauri::command]
fn get_live_overlay_snapshot(
    state: State<'_, platform::AppState>,
) -> Result<domain::LiveOverlaySnapshot, platform::CommandError> {
    platform::get_live_overlay_snapshot(state.inner())
}

#[tauri::command]
fn get_league_profile_icon(
    state: State<'_, platform::AppState>,
    input: platform::LeagueProfileIconCommand,
) -> Result<domain::LeagueImageAsset, platform::CommandError> {
    platform::get_league_profile_icon(state.inner(), input)
}

#[tauri::command]
fn get_league_champion_icon(
    state: State<'_, platform::AppState>,
    input: platform::LeagueChampionIconCommand,
) -> Result<domain::LeagueImageAsset, platform::CommandError> {
    platform::get_league_champion_icon(state.inner(), input)
}

#[tauri::command]
fn get_league_champion_details(
    state: State<'_, platform::AppState>,
    input: platform::LeagueChampionDetailsCommand,
) -> Result<domain::LeagueChampionDetails, platform::CommandError> {
    platform::get_league_champion_details(state.inner(), input)
}

#[tauri::command]
fn get_league_game_asset(
    state: State<'_, platform::AppState>,
    input: platform::LeagueGameAssetCommand,
) -> Result<domain::LeagueGameAsset, platform::CommandError> {
    platform::get_league_game_asset(state.inner(), input)
}

#[tauri::command]
fn get_post_match_detail(
    state: State<'_, platform::AppState>,
    input: platform::PostMatchDetailCommand,
) -> Result<domain::PostMatchDetail, platform::CommandError> {
    platform::get_post_match_detail(state.inner(), input)
}

#[tauri::command]
fn get_post_match_participant_profile(
    state: State<'_, platform::AppState>,
    input: platform::ParticipantPublicProfileCommand,
) -> Result<domain::ParticipantPublicProfile, platform::CommandError> {
    platform::get_post_match_participant_profile(state.inner(), input)
}

#[tauri::command]
fn save_player_note(
    state: State<'_, platform::AppState>,
    input: platform::SavePlayerNoteCommand,
) -> Result<domain::PlayerNoteView, platform::CommandError> {
    platform::save_player_note(state.inner(), input)
}

#[tauri::command]
fn clear_player_note(
    state: State<'_, platform::AppState>,
    input: platform::ClearPlayerNoteCommand,
) -> Result<domain::ClearPlayerNoteResult, platform::CommandError> {
    platform::clear_player_note(state.inner(), input)
}

/// Called from the frontend after the window loads (mirrors Frank's init_keyboard command).
/// Spawns a blocking thread for rdev so the hook is registered after Tauri's message
/// loop is fully running — same timing as Frank's approach.
#[tauri::command]
fn init_overlay_hotkey(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        listen_for_overlay_hotkey(app);
    });
}

fn listen_for_overlay_hotkey(app: tauri::AppHandle) {
    use rdev::{listen, Event, EventType, Key};

    let mut shift_down = false;

    if let Err(e) = listen(move |event: Event| {
        match event.event_type {
            EventType::KeyPress(Key::ShiftLeft | Key::ShiftRight) => {
                shift_down = true;
            }
            EventType::KeyRelease(Key::ShiftLeft | Key::ShiftRight) => {
                shift_down = false;
            }
            // Trigger on KeyRelease, not KeyPress — prevents key-repeat from
            // toggling the overlay twice (show then immediately hide).
            EventType::KeyRelease(Key::Tab) if shift_down => {
                toggle_overlay(&app);
            }
            _ => {}
        }
    }) {
        eprintln!("[overlay-hotkey] rdev listen error: {e:?}");
    }
}

fn toggle_overlay(app: &tauri::AppHandle) {
    let can_open = app
        .try_state::<platform::AppState>()
        .map(|s| platform::can_open_self_history_overlay(s.inner()))
        .unwrap_or(false);

    match app.get_webview_window(SELF_HISTORY_OVERLAY_WINDOW_LABEL) {
        None => {
            // Window not yet created — ask the frontend to open it.
            // The frontend checks can_open itself before constructing the window.
            let _ = app.emit("open-self-history-overlay", ());
        }
        Some(window) => {
            if !can_open {
                let _ = window.destroy();
                return;
            }
            match window.is_visible() {
                Ok(true) => { let _ = window.hide(); }
                Ok(false) | Err(_) => {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
    }
}

fn main() {
    if let Err(error) = tauri::Builder::default()
        .setup(|app| {
            platform::setup_app(app)?;

            let app_handle = app.handle().clone();
            let state = app.state::<platform::AppState>().inner().clone();
            platform::start_league_event_service(app_handle, state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            healthcheck,
            get_app_state,
            get_settings,
            get_settings_defaults,
            save_settings,
            list_activity_entries,
            create_activity_note,
            export_local_data,
            import_local_data,
            clear_activity_entries,
            get_league_client_status,
            get_auto_accept_status,
            can_open_self_history_overlay,
            destroy_self_history_overlay_window,
            get_league_champion_catalog,
            get_league_self_snapshot,
            get_champ_select_snapshot,
            get_ranked_champion_stats,
            refresh_ranked_champion_stats,
            get_advisor_data,
            refresh_advisor_data,
            get_champ_select_advisor_snapshot,
            get_live_overlay_snapshot,
            get_league_profile_icon,
            get_league_champion_icon,
            get_league_champion_details,
            get_league_game_asset,
            get_post_match_detail,
            get_post_match_participant_profile,
            save_player_note,
            clear_player_note,
            init_overlay_hotkey
        ])
        .run(tauri::generate_context!())
    {
        eprintln!("failed to run LoL Desktop Assistant: {error}");
    }
}
