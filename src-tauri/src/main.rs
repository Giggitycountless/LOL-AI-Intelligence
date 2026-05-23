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

#[tauri::command]
fn get_rune_recommendations(
    state: State<'_, platform::AppState>,
    input: platform::GetRuneRecommendationsCommand,
) -> Vec<domain::RuneRecommendation> {
    platform::get_rune_recommendations(state.inner(), input)
}

#[tauri::command]
fn apply_rune_page(
    state: State<'_, platform::AppState>,
    input: platform::ApplyRunePageCommand,
) -> Result<(), platform::CommandError> {
    platform::apply_rune_page(state.inner(), input)
}

#[tauri::command]
fn save_champion_rune_config(
    state: State<'_, platform::AppState>,
    input: platform::SaveRuneConfigCommand,
) -> Result<domain::ChampionRuneConfig, platform::CommandError> {
    platform::save_champion_rune_config(state.inner(), input)
}

#[tauri::command]
fn get_champion_rune_config(
    state: State<'_, platform::AppState>,
    input: platform::GetRuneConfigCommand,
) -> Result<Option<domain::ChampionRuneConfig>, platform::CommandError> {
    platform::get_champion_rune_config(state.inner(), input)
}

#[tauri::command]
fn delete_champion_rune_config(
    state: State<'_, platform::AppState>,
    input: platform::GetRuneConfigCommand,
) -> Result<bool, platform::CommandError> {
    platform::delete_champion_rune_config(state.inner(), input)
}

static HOTKEY_LISTENER_STARTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Called from the frontend after the window loads.
/// Guards against multiple calls — only one rdev listener is ever started per process.
#[tauri::command]
fn run_ai_analysis(
    app: tauri::AppHandle,
    state: State<'_, platform::AppState>,
    scope: String,
    tone: String,
) -> Result<(), platform::CommandError> {
    platform::run_ai_analysis(&app, state.inner(), scope, tone)
}

#[tauri::command]
fn run_match_recap_analysis(
    app: tauri::AppHandle,
    state: State<'_, platform::AppState>,
    game_id: i64,
    tone: String,
) -> Result<(), platform::CommandError> {
    platform::run_match_recap_analysis(&app, state.inner(), game_id, tone)
}

#[tauri::command]
fn list_chat_presets(
    state: State<'_, platform::AppState>,
) -> Result<Vec<domain::ChatPreset>, platform::CommandError> {
    platform::list_chat_presets(state.inner())
}

#[tauri::command]
fn save_chat_preset(
    state: State<'_, platform::AppState>,
    slot: i64,
    label: String,
    message: String,
) -> Result<domain::ChatPreset, platform::CommandError> {
    platform::save_chat_preset(state.inner(), slot, label, message)
}

#[tauri::command]
fn delete_chat_preset(
    state: State<'_, platform::AppState>,
    slot: i64,
) -> Result<bool, platform::CommandError> {
    platform::delete_chat_preset(state.inner(), slot)
}

#[tauri::command]
fn get_ai_config(
    state: State<'_, platform::AppState>,
) -> Result<domain::AiConfig, platform::CommandError> {
    platform::get_ai_config(state.inner())
}

#[tauri::command]
fn save_ai_analysis(
    state: State<'_, platform::AppState>,
    input: platform::SaveAiAnalysisCommand,
) -> Result<(), platform::CommandError> {
    platform::save_ai_analysis(state.inner(), input)
}

#[tauri::command]
fn get_ai_analysis(
    state: State<'_, platform::AppState>,
    scope: String,
) -> Result<Option<domain::AiAnalysisCache>, platform::CommandError> {
    platform::get_ai_analysis(state.inner(), &scope)
}

#[tauri::command]
fn init_overlay_hotkey(app: tauri::AppHandle) {
    if HOTKEY_LISTENER_STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        listen_for_overlay_hotkey(app);
    });
}

fn listen_for_overlay_hotkey(app: tauri::AppHandle) {
    use rdev::{listen, Event, EventType, Key};
    use std::sync::mpsc;

    // WH_KEYBOARD_LL callbacks must return within ~300 ms or Windows removes the hook.
    // All window work is dispatched to a dedicated thread via a bounded channel so the
    // rdev callback returns immediately.  Capacity 1 drops rapid duplicate presses.
    let (overlay_tx, overlay_rx) = mpsc::sync_channel::<()>(1);
    let (chat_tx, chat_rx) = mpsc::sync_channel::<i64>(4);

    let app_for_toggle = app.clone();
    std::thread::spawn(move || {
        for () in overlay_rx {
            toggle_overlay(&app_for_toggle);
        }
    });

    let app_for_chat = app.clone();
    std::thread::spawn(move || {
        for slot in chat_rx {
            dispatch_chat_preset(&app_for_chat, slot);
        }
    });

    // Windows may silently kill the low-level keyboard hook (e.g. when a callback
    // exceeds its budget under load, or after sleep/resume). rdev::listen() returns
    // Err in that case and never recovers on its own — so we wrap it in a restart
    // loop. Each iteration re-installs a fresh hook.
    loop {
        // Modifier state is per-listener-lifetime: re-derive from scratch on every restart
        // so stale flags from a previous session can't cause spurious toggles.
        let mut shift_down = false;
        let mut ctrl_down = false;
        let overlay_tx = overlay_tx.clone();
        let chat_tx = chat_tx.clone();
        let listen_result = listen(move |event: Event| {
            match event.event_type {
                EventType::KeyPress(Key::ShiftLeft | Key::ShiftRight) => {
                    shift_down = true;
                }
                EventType::KeyRelease(Key::ShiftLeft | Key::ShiftRight) => {
                    shift_down = false;
                }
                EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => {
                    ctrl_down = true;
                }
                EventType::KeyRelease(Key::ControlLeft | Key::ControlRight) => {
                    ctrl_down = false;
                }
                // KeyRelease avoids key-repeat firing multiple times.
                // try_send: if a toggle is already queued, drop the extra press.
                EventType::KeyRelease(Key::Tab) if shift_down && !ctrl_down => {
                    let _ = overlay_tx.try_send(());
                }
                EventType::KeyRelease(key) if ctrl_down && shift_down => {
                    if let Some(slot) = number_key_to_slot(key) {
                        let _ = chat_tx.try_send(slot);
                    }
                }
                _ => {}
            }
        });

        match listen_result {
            Ok(()) => {
                eprintln!("[overlay-hotkey] rdev listener exited cleanly; not restarting.");
                break;
            }
            Err(e) => {
                eprintln!(
                    "[overlay-hotkey] rdev listener died: {e:?}. Restarting in 1s."
                );
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

fn number_key_to_slot(key: rdev::Key) -> Option<i64> {
    use rdev::Key;
    Some(match key {
        Key::Num1 => 1,
        Key::Num2 => 2,
        Key::Num3 => 3,
        Key::Num4 => 4,
        Key::Num5 => 5,
        Key::Num6 => 6,
        Key::Num7 => 7,
        Key::Num8 => 8,
        Key::Num9 => 9,
        _ => return None,
    })
}

fn dispatch_chat_preset(app: &tauri::AppHandle, slot: i64) {
    let Some(state) = app.try_state::<platform::AppState>() else {
        return;
    };
    let preset = match platform::list_chat_presets(state.inner()) {
        Ok(presets) => presets.into_iter().find(|p| p.slot == slot),
        Err(e) => {
            eprintln!("[chat-preset] failed to load presets: {e:?}");
            return;
        }
    };
    let Some(preset) = preset else {
        // No preset bound to this slot — silently ignore.
        return;
    };

    if !is_league_game_foreground() {
        // Only inject keystrokes when the League game window is focused, to avoid
        // typing chat presets into Discord/browser/IDE/etc.
        return;
    }

    if let Err(e) = type_chat_message(&preset.message) {
        eprintln!("[chat-preset] failed to type message: {e}");
    }
}

#[cfg(windows)]
fn is_league_game_foreground() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetForegroundWindow};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }
        let mut buf = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut buf);
        if len <= 0 {
            return false;
        }
        let class_name = String::from_utf16_lossy(&buf[..len as usize]);
        // League's in-game window uses class "RiotWindowClass"
        class_name == "RiotWindowClass"
    }
}

#[cfg(not(windows))]
fn is_league_game_foreground() -> bool {
    // Non-Windows builds (testing only): always allow.
    true
}

fn type_chat_message(message: &str) -> Result<(), String> {
    use enigo::{Enigo, Key, Keyboard, Settings, Direction};

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    // Open team chat (Enter)
    enigo.key(Key::Return, Direction::Click).map_err(|e| e.to_string())?;
    // League needs a moment to register the chat box is open before accepting text
    std::thread::sleep(std::time::Duration::from_millis(80));
    // Type the message (Unicode-safe; works for Chinese)
    enigo.text(message).map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(30));
    // Send
    enigo.key(Key::Return, Direction::Click).map_err(|e| e.to_string())?;
    Ok(())
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
            run_ai_analysis,
            run_match_recap_analysis,
            list_chat_presets,
            save_chat_preset,
            delete_chat_preset,
            get_ai_config,
            save_ai_analysis,
            get_ai_analysis,
            init_overlay_hotkey,
            get_rune_recommendations,
            apply_rune_page,
            save_champion_rune_config,
            get_champion_rune_config,
            delete_champion_rune_config
        ])
        .run(tauri::generate_context!())
    {
        eprintln!("failed to run LoL Desktop Assistant: {error}");
    }
}
