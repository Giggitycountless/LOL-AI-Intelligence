use super::*;
use domain::{
    AdvisorPlayerTagKind, LeagueClientConnection, LeagueClientPhase, LeagueDataSection,
    LeagueDataWarning, MatchResult, RunePage,
};
use std::{cell::RefCell, sync::Mutex};

#[test]
fn save_settings_does_not_log_activity_when_values_are_unchanged() {
    let store = FakeStore::new(default_settings());

    let result = save_settings(
        &store,
        SettingsInput {
            startup_page: "dashboard".to_string(),
            language: "system".to_string(),
            theme: "light".to_string(),
            compact_mode: false,
            activity_limit: 100,
            auto_accept_enabled: true,
            auto_pick_enabled: false,
            auto_pick_champion_id: None,
            auto_pick_delay_seconds: 0.0,
            auto_ban_enabled: false,
            auto_ban_champion_id: None,
            auto_ban_delay_seconds: 0.0,
            auto_honor_enabled: false,
            ai_base_url: None,
            ai_api_key: None,
            ai_model: None,
        },
    )
    .expect("settings save succeeds");

    assert_eq!(result.startup_page, StartupPage::Dashboard);
    assert_eq!(store.created_entries.borrow().len(), 0);
}

#[test]
fn save_settings_logs_activity_when_values_change() {
    let store = FakeStore::new(default_settings());

    let result = save_settings(
        &store,
        SettingsInput {
            startup_page: "profile".to_string(),
            language: "zh".to_string(),
            theme: "dark".to_string(),
            compact_mode: true,
            activity_limit: 50,
            auto_accept_enabled: false,
            auto_pick_enabled: true,
            auto_pick_champion_id: Some(103),
            auto_pick_delay_seconds: 1.5,
            auto_ban_enabled: true,
            auto_ban_champion_id: Some(122),
            auto_ban_delay_seconds: 0.5,
            auto_honor_enabled: false,
            ai_base_url: None,
            ai_api_key: None,
            ai_model: None,
        },
    )
    .expect("settings save succeeds");

    assert_eq!(result.startup_page, StartupPage::Profile);
    assert_eq!(result.language, AppLanguagePreference::Zh);
    assert_eq!(result.activity_limit, 50);
    assert_eq!(store.created_entries.borrow().len(), 1);
    assert_eq!(
        store.created_entries.borrow()[0].kind,
        ActivityKind::Settings
    );
}

#[test]
fn ready_check_automation_respects_auto_accept_setting() {
    let mut settings = default_settings();
    settings.auto_accept_enabled = false;
    let store = FakeStore::new(settings);
    let reader = FakeLeagueClientReader::new(Vec::new()).with_ready_check_phase();

    run_ready_check_automation(&store, &reader).expect("automation runs");

    assert_eq!(reader.accept_ready_check_count(), 0);
}

#[test]
fn ready_check_automation_calls_reader_when_enabled_and_ready_check_is_active() {
    let store = FakeStore::new(default_settings());
    let reader = FakeLeagueClientReader::new(Vec::new())
        .with_phase_transition_after_accepts(1, "ChampSelect");

    run_ready_check_automation(&store, &reader).expect("automation runs");

    assert_eq!(reader.accept_ready_check_count(), 1);
}

#[test]
fn ready_check_automation_retries_until_phase_changes() {
    let store = FakeStore::new(default_settings());
    let reader = FakeLeagueClientReader::new(Vec::new())
        .with_phase_transition_after_accepts(3, "ChampSelect");

    run_ready_check_automation(&store, &reader).expect("automation runs");

    assert_eq!(reader.accept_ready_check_count(), 3);
}

#[test]
fn ready_check_automation_records_system_activity_when_ready_check_stays_active() {
    let store = FakeStore::new(default_settings());
    let reader = FakeLeagueClientReader::new(Vec::new()).with_ready_check_phase();

    let error = run_ready_check_automation(&store, &reader).expect_err("automation should fail");

    assert_eq!(
        error.to_string(),
        "Auto-accept did not move the client out of ReadyCheck after multiple verification attempts"
    );
    assert_eq!(reader.accept_ready_check_count(), 4);
    assert_eq!(store.created_entries.borrow().len(), 1);
    assert_eq!(store.created_entries.borrow()[0].kind, ActivityKind::System);
}

#[test]
fn ready_check_automation_records_system_activity_when_accept_call_errors() {
    let store = FakeStore::new(default_settings());
    let reader = FakeLeagueClientReader::new(Vec::new()).with_ready_check_accept_error(
        LeagueClientReadError::ClientUnavailable("League Client unavailable".to_string()),
    );

    let error = run_ready_check_automation(&store, &reader).expect_err("automation should fail");

    assert_eq!(error.code(), "clientUnavailable");
    assert_eq!(reader.accept_ready_check_count(), 1);
    assert_eq!(store.created_entries.borrow().len(), 1);
    assert_eq!(store.created_entries.borrow()[0].kind, ActivityKind::System);
}

#[test]
fn ready_check_automation_treats_accept_error_as_success_when_phase_moves() {
    let store = FakeStore::new(default_settings());
    let reader = FakeLeagueClientReader::new(Vec::new())
        .with_phase_transition_after_accepts(1, "ChampSelect")
        .with_ready_check_accept_error(LeagueClientReadError::Integration(
            "Ready check response was unavailable".to_string(),
        ));

    run_ready_check_automation(&store, &reader).expect("phase movement confirms accept");

    assert_eq!(reader.accept_ready_check_count(), 1);
    assert!(store.created_entries.borrow().is_empty());
}

#[test]
fn set_chat_status_forwards_message_and_availability_to_reader() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    set_chat_status(&reader, Some("gl hf".to_string()), Some("away".to_string()))
        .expect("valid status update");

    assert_eq!(
        *reader.last_chat_status.lock().unwrap(),
        Some((Some("gl hf".to_string()), Some("away".to_string())))
    );
}

#[test]
fn set_chat_status_rejects_unknown_availability() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let error = set_chat_status(&reader, None, Some("invisible".to_string()))
        .expect_err("unknown availability should be rejected");

    assert_eq!(error.code(), "validation");
    assert!(reader.last_chat_status.lock().unwrap().is_none());
}

#[test]
fn set_chat_status_requires_at_least_one_field() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let error =
        set_chat_status(&reader, None, None).expect_err("empty update should be rejected");

    assert_eq!(error.code(), "validation");
}

#[test]
fn champ_select_automation_calls_reader_when_settings_are_enabled() {
    let mut settings = default_settings();
    settings.auto_pick_enabled = true;
    settings.auto_pick_champion_id = Some(103);
    settings.auto_ban_enabled = true;
    settings.auto_ban_champion_id = Some(122);
    let store = FakeStore::new(settings);
    let reader = FakeLeagueClientReader::new(Vec::new());

    run_champ_select_automation(&store, &reader).expect("automation executes safely");

    assert_eq!(reader.champ_select_preference_call_count(), 1);
}

#[test]
fn champ_select_automation_no_ops_when_both_disabled() {
    let store = FakeStore::new(default_settings());
    let reader = FakeLeagueClientReader::new(Vec::new());

    run_champ_select_automation(&store, &reader).expect("automation no-ops when disabled");

    assert_eq!(reader.champ_select_preference_call_count(), 0);
}

#[test]
fn create_activity_note_trims_input() {
    let store = FakeStore::new(default_settings());

    let result = create_activity_note(
        &store,
        ActivityNoteInput {
            title: "  First note  ".to_string(),
            body: Some("  Body  ".to_string()),
        },
    )
    .expect("activity note is created");

    assert_eq!(result.title, "First note");
    assert_eq!(result.body.as_deref(), Some("Body"));
}

#[test]
fn list_activity_entries_passes_filter_to_store() {
    let store = FakeStore::new(default_settings());

    let _ = list_activity_entries(
        &store,
        ActivityListInput {
            limit: Some(25),
            kind: Some(ActivityKind::Note),
        },
    )
    .expect("activity entries list");

    assert_eq!(
        *store.last_activity_query.borrow(),
        Some((25, Some(ActivityKind::Note)))
    );
}

#[test]
fn export_local_data_includes_defaults_shape() {
    let store = FakeStore::new(default_settings());
    store.activity_entries.borrow_mut().push(sample_activity(1));

    let data = export_local_data(&store).expect("local data export");

    assert_eq!(data.format_version, 1);
    assert_eq!(data.settings.activity_limit, 100);
    assert_eq!(data.activity_entries.len(), 1);
    assert_eq!(data.activity_entries[0].created_at, "2026-04-18 00:00:00");
}

#[test]
fn import_local_data_rejects_invalid_json_without_writing() {
    let store = FakeStore::new(default_settings());

    let result = import_local_data(
        &store,
        r#"{"formatVersion":1,"settings":{"startupPage":"dashboard","compactMode":false,"activityLimit":999},"activityEntries":[]}"#,
    );

    assert!(matches!(result, Err(ApplicationError::Validation(_))));
    assert_eq!(*store.import_count.borrow(), 0);
}

#[test]
fn import_local_data_validates_then_writes() {
    let store = FakeStore::new(default_settings());

    let result = import_local_data(
            &store,
            r#"{"formatVersion":1,"settings":{"startupPage":"profile","compactMode":true,"activityLimit":50},"activityEntries":[{"kind":"note","title":"Imported","body":null,"createdAt":"2026-04-19 00:00:00"}]}"#,
        )
        .expect("local data import");

    assert_eq!(result.imported_activity_count, 1);
    assert_eq!(result.settings.startup_page, StartupPage::Profile);
    assert_eq!(*store.import_count.borrow(), 1);
}

#[test]
fn clear_activity_requires_confirmation() {
    let store = FakeStore::new(default_settings());

    let result = clear_activity_entries(&store, false);

    assert!(matches!(result, Err(ApplicationError::Validation(_))));
    assert_eq!(*store.clear_count.borrow(), 0);
}

#[test]
fn clear_activity_returns_deleted_count() {
    let store = FakeStore::new(default_settings());
    store.activity_entries.borrow_mut().push(sample_activity(1));
    store.activity_entries.borrow_mut().push(sample_activity(2));

    let result = clear_activity_entries(&store, true).expect("activity clears");

    assert_eq!(result.deleted_count, 2);
    assert_eq!(*store.clear_count.borrow(), 1);
}

#[test]
fn league_self_snapshot_defaults_to_six_matches_and_summarizes_performance() {
    let reader = FakeLeagueClientReader::new((1..=7).map(high_kda_match).collect());

    let result = get_league_self_snapshot(&reader, LeagueSelfSnapshotInput { match_limit: None })
        .expect("league self snapshot");

    assert_eq!(*reader.last_match_limit.lock().unwrap(), Some(6));
    assert_eq!(result.recent_matches.len(), 6);
    assert_eq!(result.recent_performance.match_count, 6);
    assert_eq!(result.recent_performance.average_kda, Some(10.0));
    assert_eq!(result.recent_performance.kda_tag, KdaTag::High);
    assert_eq!(result.recent_performance.recent_champions.len(), 6);
    assert_eq!(result.recent_performance.top_champions.len(), 3);
}

#[test]
fn search_player_profile_returns_found_snapshot_with_derived_summaries() {
    let reader = FakeLeagueClientReader::new((1..=7).map(high_kda_match).collect());

    let result = search_player_profile(
        &reader,
        PlayerSearchInput {
            query: "  Faker  ".to_string(),
            match_limit: Some(7),
        },
    )
    .expect("player search profile");

    assert!(result.found);
    assert_eq!(result.recent_matches.len(), 7);
    // Performance summary caps at PERFORMANCE_MATCH_COUNT (6), same as the self snapshot.
    assert_eq!(result.recent_performance.match_count, 6);
    assert_eq!(result.recent_performance.kda_tag, KdaTag::High);
    // No playstyle matches supplied by the fake → empty fingerprint, not an error.
    assert_eq!(result.playstyle.games_analyzed, 0);
    assert!(result.playstyle.tags.is_empty());
}

#[test]
fn search_player_profile_reports_not_found_for_missing_summoner() {
    let reader = FakeLeagueClientReader::new((1..=3).map(high_kda_match).collect());

    let result = search_player_profile(
        &reader,
        PlayerSearchInput {
            query: "missing".to_string(),
            match_limit: None,
        },
    )
    .expect("player search profile");

    assert!(!result.found);
    assert!(result.summoner.is_none());
    assert!(result.recent_matches.is_empty());
    assert!(result.ranked_queues.is_empty());
}

#[test]
fn search_player_profile_rejects_empty_query() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let result = search_player_profile(
        &reader,
        PlayerSearchInput {
            query: "   ".to_string(),
            match_limit: None,
        },
    );

    assert!(matches!(result, Err(ApplicationError::Validation(_))));
}

#[test]
fn champ_select_snapshot_batches_recent_stats() {
    let mut champion_selections = HashMap::new();
    champion_selections.insert(1, 103);
    champion_selections.insert(2, 222);
    let reader = FakeLeagueClientReader::with_champ_select_data(
        ChampSelectSessionData {
            ally_ids: vec![1, 2],
            enemy_ids: Vec::new(),
            champion_selections,
            ally_names: Vec::new(),
            enemy_names: Vec::new(),
            champion_selections_by_name: HashMap::new(),
            source: ChampSelectSessionSource::ChampSelect,
            players: Vec::new(),
        },
        vec![
            SummonerBatchEntry {
                summoner_id: 1,
                puuid: "puuid-1".to_string(),
                display_name: "Player One".to_string(),
                summoner_level: None,
            },
            SummonerBatchEntry {
                summoner_id: 2,
                puuid: "puuid-2".to_string(),
                display_name: "Player Two".to_string(),
                summoner_level: None,
            },
        ],
        Vec::new(),
    );

    let snapshot = get_champ_select_snapshot(&reader, 6).expect("champ select snapshot reads");

    assert_eq!(snapshot.players.len(), 2);
    assert!(
        snapshot
            .players
            .iter()
            .all(|player| player.recent_stats.is_some())
    );
    assert_eq!(
        reader.recent_stats_batch_calls(),
        vec![vec!["puuid-1".to_string(), "puuid-2".to_string()]]
    );
}

#[test]
fn champ_select_recent_stats_failure_keeps_other_players() {
    let reader = FakeLeagueClientReader::with_champ_select_data(
        ChampSelectSessionData {
            ally_ids: vec![1, 2],
            enemy_ids: Vec::new(),
            champion_selections: HashMap::new(),
            ally_names: Vec::new(),
            enemy_names: Vec::new(),
            champion_selections_by_name: HashMap::new(),
            source: ChampSelectSessionSource::ChampSelect,
            players: Vec::new(),
        },
        vec![
            SummonerBatchEntry {
                summoner_id: 1,
                puuid: "puuid-1".to_string(),
                display_name: "Player One".to_string(),
                summoner_level: None,
            },
            SummonerBatchEntry {
                summoner_id: 2,
                puuid: "puuid-2".to_string(),
                display_name: "Player Two".to_string(),
                summoner_level: None,
            },
        ],
        vec!["puuid-2".to_string()],
    );

    let snapshot = get_champ_select_snapshot(&reader, 6).expect("champ select snapshot reads");
    let player_one = snapshot
        .players
        .iter()
        .find(|player| player.display_name == "Player One")
        .expect("player one is present");
    let player_two = snapshot
        .players
        .iter()
        .find(|player| player.display_name == "Player Two")
        .expect("player two is present");

    assert!(player_one.recent_stats.is_some());
    assert!(player_two.recent_stats.is_none());
    assert_eq!(
        player_two.recent_stats_status,
        ChampSelectRecentStatsStatus::Unavailable
    );
}

#[test]
fn champ_select_snapshot_matches_bare_names_to_riot_id_display_names() {
    let mut reader = FakeLeagueClientReader::with_champ_select_data(
        ChampSelectSessionData {
            ally_ids: Vec::new(),
            enemy_ids: Vec::new(),
            champion_selections: HashMap::new(),
            ally_names: vec!["Player One".to_string()],
            enemy_names: Vec::new(),
            champion_selections_by_name: HashMap::new(),
            source: ChampSelectSessionSource::LiveClient,
            players: Vec::new(),
        },
        Vec::new(),
        Vec::new(),
    );
    reader.summoners_by_name = vec![SummonerBatchEntry {
        summoner_id: 10,
        puuid: "puuid-10".to_string(),
        display_name: "Player One#NA1".to_string(),
        summoner_level: None,
    }];

    let snapshot = get_champ_select_snapshot(&reader, 6).expect("champ select snapshot reads");

    assert_eq!(snapshot.players.len(), 1);
    assert_eq!(snapshot.players[0].display_name, "Player One#NA1");
    assert_eq!(
        snapshot.players[0].recent_stats_status,
        ChampSelectRecentStatsStatus::Loaded
    );
    assert_eq!(
        reader.recent_stats_batch_calls(),
        vec![vec!["puuid-10".to_string()]]
    );
}

#[test]
fn champ_select_snapshot_marks_missing_identity() {
    let reader = FakeLeagueClientReader::with_champ_select_data(
        ChampSelectSessionData {
            ally_ids: Vec::new(),
            enemy_ids: Vec::new(),
            champion_selections: HashMap::new(),
            ally_names: vec!["Unknown Player".to_string()],
            enemy_names: Vec::new(),
            champion_selections_by_name: HashMap::new(),
            source: ChampSelectSessionSource::LiveClient,
            players: Vec::new(),
        },
        Vec::new(),
        Vec::new(),
    );

    let snapshot = get_champ_select_snapshot(&reader, 6).expect("champ select snapshot reads");

    assert_eq!(snapshot.players.len(), 1);
    assert!(snapshot.players[0].recent_stats.is_none());
    assert_eq!(
        snapshot.players[0].recent_stats_status,
        ChampSelectRecentStatsStatus::MissingIdentity
    );
    assert!(reader.recent_stats_batch_calls().is_empty());
}

#[test]
fn ranked_champion_stats_filters_lane_and_sorts_by_win_rate() {
    let response = get_ranked_champion_stats(RankedChampionStatsInput {
        lane: Some(RankedChampionLane::Jungle),
        sort_by: Some(RankedChampionSort::WinRate),
    });

    assert_eq!(response.lane, Some(RankedChampionLane::Jungle));
    assert_eq!(response.sort_by, RankedChampionSort::WinRate);
    assert!(
        response
            .records
            .iter()
            .all(|record| record.lane == RankedChampionLane::Jungle)
    );
    assert!(
        response
            .records
            .windows(2)
            .all(|records| records[0].win_rate >= records[1].win_rate)
    );
}

#[test]
fn ranked_champion_stats_supports_all_sort_modes() {
    for sort_by in [
        RankedChampionSort::Overall,
        RankedChampionSort::WinRate,
        RankedChampionSort::BanRate,
        RankedChampionSort::PickRate,
    ] {
        let response = get_ranked_champion_stats(RankedChampionStatsInput {
            lane: None,
            sort_by: Some(sort_by),
        });

        assert_eq!(response.sort_by, sort_by);
        assert_eq!(response.records.len(), 25);
        assert!(response.records.windows(2).all(|records| {
            ranked_sort_value(&records[0], sort_by) >= ranked_sort_value(&records[1], sort_by)
        }));
    }
}

#[test]
fn ranked_champion_stats_reads_cached_snapshot_when_available() {
    let store = FakeStore::new(default_settings());
    store
        .ranked_snapshot
        .replace(Some(sample_ranked_snapshot("cached-json")));

    let response = get_ranked_champion_stats_from_store(
        &store,
        RankedChampionStatsInput {
            lane: Some(RankedChampionLane::Middle),
            sort_by: Some(RankedChampionSort::Overall),
        },
    )
    .expect("ranked champion stats reads");

    assert_eq!(response.source, "cached-json");
    assert_eq!(response.patch.as_deref(), Some("26.08"));
    assert!(response.is_cached);
    assert_eq!(response.data_status, RankedChampionDataStatus::Cached);
    assert_eq!(response.records.len(), 1);
    assert_eq!(response.records[0].champion_name, "Ahri");
}

#[test]
fn ranked_champion_refresh_persists_provider_snapshot() {
    let store = FakeStore::new(default_settings());
    let provider = FakeRankedChampionProvider {
        snapshot: sample_ranked_snapshot("remote-json"),
    };

    let response = refresh_ranked_champion_stats(
        &store,
        &provider,
        RankedChampionRefreshInput::default(),
        RankedChampionStatsInput {
            lane: Some(RankedChampionLane::Middle),
            sort_by: Some(RankedChampionSort::WinRate),
        },
    )
    .expect("ranked champion stats refreshes");

    assert_eq!(response.source, "remote-json");
    assert!(response.is_cached);
    assert_eq!(response.data_status, RankedChampionDataStatus::Fresh);
    assert!(store.ranked_snapshot.borrow().is_some());
    assert_eq!(
        store.ranked_snapshot.borrow().as_ref().unwrap().source,
        "remote-json"
    );
}

#[test]
fn ranked_champion_refresh_returns_stale_cache_when_remote_fails() {
    let store = FakeStore::new(default_settings());
    store
        .ranked_snapshot
        .replace(Some(sample_ranked_snapshot("cached-json")));
    let provider = FailingRankedChampionProvider;

    let response = refresh_ranked_champion_stats(
        &store,
        &provider,
        RankedChampionRefreshInput::default(),
        RankedChampionStatsInput {
            lane: Some(RankedChampionLane::Middle),
            sort_by: Some(RankedChampionSort::Overall),
        },
    )
    .expect("stale cache is returned");

    assert_eq!(response.source, "cached-json");
    assert_eq!(response.data_status, RankedChampionDataStatus::StaleCache);
    assert_eq!(response.records.len(), 1);
    assert!(response.status_message.unwrap().contains("could not be refreshed"));
}

#[test]
fn ranked_champion_refresh_errors_without_cache_when_remote_fails() {
    let store = FakeStore::new(default_settings());
    let provider = FailingRankedChampionProvider;

    let error = refresh_ranked_champion_stats(
        &store,
        &provider,
        RankedChampionRefreshInput::default(),
        RankedChampionStatsInput {
            lane: None,
            sort_by: None,
        },
    )
    .expect_err("refresh fails without cache");

    assert_eq!(error.code(), "integration");
}

#[test]
fn advisor_data_filters_lane_and_champion() {
    let store = FakeStore::new(default_settings());
    store
        .advisor_snapshot
        .replace(Some(sample_advisor_fixture("cached-advisor")));

    let response = get_advisor_data_from_store(
        &store,
        AdvisorDataInput {
            lane: Some(RankedChampionLane::Top),
            champion_id: Some(86),
        },
    )
    .expect("advisor data reads");

    assert_eq!(response.source, "cached-advisor");
    assert_eq!(response.records.len(), 1);
    assert_eq!(response.records[0].champion_name, "Garen");
    assert_eq!(response.records[0].lane, RankedChampionLane::Top);
}

#[test]
fn advisor_refresh_persists_provider_snapshot() {
    let store = FakeStore::new(default_settings());
    let provider = FakeAdvisorProvider {
        snapshot: sample_advisor_fixture("remote-advisor"),
    };

    let response = refresh_advisor_data(
        &store,
        &provider,
        AdvisorDataRefreshInput { url: None },
        AdvisorDataInput {
            lane: Some(RankedChampionLane::Top),
            champion_id: None,
        },
    )
    .expect("advisor data refreshes");

    assert_eq!(response.source, "remote-advisor");
    assert_eq!(response.data_status, RankedChampionDataStatus::Fresh);
    assert!(store.advisor_snapshot.borrow().is_some());
}

#[test]
fn champ_select_advisor_snapshot_adds_tags_and_matchup_advice() {
    let store = FakeStore::new(default_settings());
    store
        .advisor_snapshot
        .replace(Some(sample_advisor_fixture("cached-advisor")));
    let reader = FakeLeagueClientReader::with_champ_select_data(
        ChampSelectSessionData {
            ally_ids: vec![1],
            enemy_ids: vec![2],
            champion_selections: HashMap::from([(1, 86), (2, 122)]),
            ally_names: Vec::new(),
            enemy_names: Vec::new(),
            champion_selections_by_name: HashMap::new(),
            source: ChampSelectSessionSource::ChampSelect,
            players: Vec::new(),
        },
        vec![
            SummonerBatchEntry {
                summoner_id: 1,
                puuid: "puuid-1".to_string(),
                display_name: "Ally".to_string(),
                summoner_level: None,
            },
            SummonerBatchEntry {
                summoner_id: 2,
                puuid: "puuid-2".to_string(),
                display_name: "Enemy".to_string(),
                summoner_level: None,
            },
        ],
        Vec::new(),
    );

    let snapshot =
        get_champ_select_advisor_snapshot(&store, &reader, 6).expect("advisor snapshot reads");
    let ally = snapshot
        .players
        .iter()
        .find(|player| player.display_name == "Ally")
        .expect("ally player exists");

    assert!(ally.tags.iter().any(|tag| tag.kind == AdvisorPlayerTagKind::StrongPick));
    assert!(ally.tags.iter().any(|tag| tag.kind == AdvisorPlayerTagKind::Spike));
    assert!(
        ally.matchup_advice
            .as_deref()
            .unwrap_or_default()
            .contains("Darius")
    );
}

#[test]
fn player_advisor_tags_detect_one_trick_and_loss_streak() {
    let mut recent_matches = Vec::new();
    for game_id in 1..=4 {
        let mut match_summary = sample_match(game_id, "Garen", 1, 6, 2);
        match_summary.result = MatchResult::Loss;
        recent_matches.push(match_summary);
    }
    let stats = ParticipantRecentStats {
        match_count: 4,
        average_kda: Some(0.5),
        recent_champions: vec!["Garen".to_string()],
        recent_matches,
    };
    let advisor = sample_advisor_record_fixture(
        86,
        "Garen",
        RankedChampionLane::Top,
        52.5,
        Vec::new(),
        Vec::new(),
        "Trade when Q is ready.",
    );

    let tags = player_advisor_tags(&Some(stats), Some(&advisor));

    assert!(tags.iter().any(|tag| tag.kind == AdvisorPlayerTagKind::OneTrick));
    assert!(tags.iter().any(|tag| tag.kind == AdvisorPlayerTagKind::LossStreak
        && tag.value.as_deref() == Some("4")));
    assert!(tags.iter().any(|tag| tag.kind == AdvisorPlayerTagKind::StrongPick));
}

#[test]
fn live_overlay_snapshot_delegates_without_hidden_timer_fields() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let snapshot = get_live_overlay_snapshot(&reader).expect("live overlay reads");

    assert_eq!(snapshot.game_time_seconds, Some(300.0));
    assert_eq!(snapshot.gold.item_value_diff, 500);
    assert!(snapshot.events.is_empty());
}

#[test]
fn league_self_snapshot_rejects_invalid_match_limit() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let result = get_league_self_snapshot(
        &reader,
        LeagueSelfSnapshotInput {
            match_limit: Some(0),
        },
    );

    assert!(matches!(result, Err(ApplicationError::Validation(_))));
    assert_eq!(*reader.last_match_limit.lock().unwrap(), None);
}

#[test]
fn league_self_snapshot_handles_zero_death_matches() {
    let reader = FakeLeagueClientReader::new(vec![sample_match(1, "Ahri", 7, 0, 5)]);

    let result = get_league_self_snapshot(
        &reader,
        LeagueSelfSnapshotInput {
            match_limit: Some(1),
        },
    )
    .expect("league self snapshot");

    assert_eq!(result.recent_performance.average_kda, Some(12.0));
    assert_eq!(result.recent_performance.kda_tag, KdaTag::High);
}

#[test]
fn league_self_snapshot_marks_empty_performance_unavailable() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let result = get_league_self_snapshot(
        &reader,
        LeagueSelfSnapshotInput {
            match_limit: Some(6),
        },
    )
    .expect("league self snapshot");

    assert_eq!(result.recent_performance.match_count, 0);
    assert_eq!(result.recent_performance.average_kda, None);
    assert_eq!(result.recent_performance.kda_tag, KdaTag::Unavailable);
}

#[test]
fn league_self_snapshot_preserves_unavailable_status() {
    let reader = FakeLeagueClientReader::with_data(LeagueSelfData {
        status: LeagueClientStatus {
            is_running: false,
            lockfile_found: false,
            connection: LeagueClientConnection::Unavailable,
            phase: LeagueClientPhase::NotRunning,
            message: Some("League Client is not running".to_string()),
        },
        summoner: None,
        ranked_queues: Vec::new(),
        recent_matches: Vec::new(),
        data_warnings: Vec::new(),
    });

    let result = get_league_self_snapshot(
        &reader,
        LeagueSelfSnapshotInput {
            match_limit: Some(6),
        },
    )
    .expect("league self snapshot");

    assert_eq!(result.status.phase, LeagueClientPhase::NotRunning);
    assert!(result.summoner.is_none());
    assert!(result.data_warnings.is_empty());
}

#[test]
fn league_self_snapshot_accepts_partial_data_without_error() {
    let reader = FakeLeagueClientReader::with_data(LeagueSelfData {
        status: LeagueClientStatus {
            is_running: true,
            lockfile_found: true,
            connection: LeagueClientConnection::Connected,
            phase: LeagueClientPhase::PartialData,
            message: Some("League Client connected with partial data".to_string()),
        },
        summoner: None,
        ranked_queues: Vec::new(),
        recent_matches: vec![sample_match(1, "Ahri", 1, 1, 1)],
        data_warnings: vec![LeagueDataWarning {
            section: LeagueDataSection::Ranked,
            message: "Ranked data is temporarily unavailable".to_string(),
        }],
    });

    let result = get_league_self_snapshot(
        &reader,
        LeagueSelfSnapshotInput {
            match_limit: Some(6),
        },
    )
    .expect("league self snapshot");

    assert_eq!(result.status.phase, LeagueClientPhase::PartialData);
    assert_eq!(result.data_warnings.len(), 1);
    assert_eq!(result.data_warnings[0].section, LeagueDataSection::Ranked);
}

#[test]
fn league_client_error_codes_are_stable() {
    let unavailable = ApplicationError::from(LeagueClientReadError::ClientUnavailable(
        "League Client is not running".to_string(),
    ));
    let access = ApplicationError::from(LeagueClientReadError::ClientAccess(
        "League Client rejected local authentication".to_string(),
    ));
    let integration = ApplicationError::from(LeagueClientReadError::Integration(
        "League Client returned an unexpected response".to_string(),
    ));

    assert_eq!(unavailable.code(), "clientUnavailable");
    assert_eq!(access.code(), "clientAccess");
    assert_eq!(integration.code(), "integration");
}

#[test]
fn league_profile_icon_validates_id_before_reader_call() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let result = get_league_profile_icon(&reader, LeagueProfileIconInput { profile_icon_id: 0 });

    assert!(matches!(result, Err(ApplicationError::Validation(_))));
}

#[test]
fn league_champion_icon_returns_image_bytes() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let result = get_league_champion_icon(&reader, LeagueChampionIconInput { champion_id: 103 })
        .expect("champion icon reads");

    assert_eq!(result.mime_type, "image/png");
    assert_eq!(result.bytes, vec![103]);
}

#[test]
fn league_game_asset_validates_id_before_reader_call() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let result = get_league_game_asset(
        &reader,
        LeagueGameAssetInput {
            kind: LeagueGameAssetKind::Item,
            asset_id: 0,
        },
    );

    assert!(matches!(result, Err(ApplicationError::Validation(_))));
}

#[test]
fn league_game_asset_returns_metadata_and_image_bytes() {
    let reader = FakeLeagueClientReader::new(Vec::new());

    let result = get_league_game_asset(
        &reader,
        LeagueGameAssetInput {
            kind: LeagueGameAssetKind::Spell,
            asset_id: 4,
        },
    )
    .expect("game asset reads");

    assert_eq!(result.kind, LeagueGameAssetKind::Spell);
    assert_eq!(result.asset_id, 4);
    assert_eq!(result.name, "Spell 4");
    assert_eq!(result.image.bytes, vec![4]);
}

#[test]
fn player_note_validation_trims_and_deduplicates() {
    let store = FakeStore::new(default_settings());

    let result = save_player_note_for_resolved_player(
        &store,
        SavePlayerNoteInput {
            game_id: 10,
            participant_id: 2,
            note: Some("  Watch roams  ".to_string()),
            tags: vec![
                " mid ".to_string(),
                "mid".to_string(),
                "shotcaller".to_string(),
            ],
        },
        "internal-puuid".to_string(),
        "Visible Player".to_string(),
    )
    .expect("player note saves");

    assert_eq!(result.note.as_deref(), Some("Watch roams"));
    assert_eq!(result.tags, vec!["mid", "shotcaller"]);
    assert_eq!(result.game_id, 10);
    assert_eq!(result.participant_id, 2);
}

#[test]
fn player_note_summary_does_not_require_puuid() {
    let store = FakeStore::new(default_settings());

    let summary = player_note_summary(&store, None).expect("summary reads");

    assert!(!summary.has_note);
    assert!(summary.tags.is_empty());
}

#[test]
fn post_match_detail_groups_teams_and_hydrates_notes() {
    let store = FakeStore::new(default_settings());
    store
        .save_player_note(StoredPlayerNoteInput {
            player_puuid: "self-puuid".to_string(),
            last_display_name: "Player One".to_string(),
            note: Some("Played well".to_string()),
            tags: vec!["carry".to_string()],
        })
        .expect("note saves");
    let reader = FakeLeagueClientReader::with_completed_match(sample_completed_match());

    let detail = get_post_match_detail(&store, &reader, PostMatchDetailInput { game_id: 10 })
        .expect("post-match detail reads");

    assert_eq!(detail.teams.len(), 2);
    assert_eq!(detail.teams[0].participants.len(), 1);
    assert_eq!(detail.teams[0].totals.kills, 7);
    assert_eq!(detail.comparison.most_damage.unwrap().participant_id, 2);
    assert!(detail.teams[0].participants[0].performance_score > 8.0);
    assert!(detail.teams[0].participants[0].note_summary.has_note);
    assert_eq!(
        detail.teams[0].participants[0].note_summary.tags,
        vec!["carry"]
    );
}

#[test]
fn post_match_detail_scores_participants_from_available_stats() {
    let store = FakeStore::new(default_settings());
    let reader = FakeLeagueClientReader::with_completed_match(sample_completed_match());

    let detail = get_post_match_detail(&store, &reader, PostMatchDetailInput { game_id: 10 })
        .expect("post-match detail reads");
    let first_score = detail.teams[0].participants[0].performance_score;
    let second_score = detail.teams[1].participants[0].performance_score;

    assert!((0.0..=10.0).contains(&first_score));
    assert!((0.0..=10.0).contains(&second_score));
    assert!(first_score > second_score);
}

#[test]
fn post_match_detail_warns_when_only_partial_participants_are_available() {
    let store = FakeStore::new(default_settings());
    let mut completed_match = sample_completed_match();
    completed_match.participants.truncate(1);
    let reader = FakeLeagueClientReader::with_completed_match(completed_match);

    let detail = get_post_match_detail(&store, &reader, PostMatchDetailInput { game_id: 10 })
        .expect("post-match detail reads");

    assert_eq!(detail.teams.len(), 1);
    assert_eq!(detail.warnings.len(), 1);
    assert_eq!(detail.warnings[0].section, LeagueDataSection::Participants);
}

#[test]
fn participant_profile_uses_completed_match_context_without_exposing_puuid() {
    let store = FakeStore::new(default_settings());
    let reader = FakeLeagueClientReader::with_completed_match(sample_completed_match());

    let profile = get_post_match_participant_profile(
        &store,
        &reader,
        ParticipantPublicProfileInput {
            game_id: 10,
            participant_id: 2,
            recent_limit: Some(3),
        },
    )
    .expect("participant profile reads");

    assert_eq!(profile.display_name, "Player Two");
    assert_eq!(profile.recent_stats.as_ref().unwrap().match_count, 3);
    assert_eq!(
        profile.recent_stats.as_ref().unwrap().recent_matches.len(),
        3
    );
    assert!(format!("{profile:?}").contains("Player Two"));
    assert!(!format!("{profile:?}").contains("enemy-puuid"));
}

struct FakeStore {
    settings: RefCell<AppSettings>,
    activity_entries: RefCell<Vec<ActivityEntry>>,
    created_entries: RefCell<Vec<NewActivityEntry>>,
    imported_entries: RefCell<Vec<LocalActivityEntry>>,
    player_notes: RefCell<Vec<StoredPlayerNote>>,
    ranked_snapshot: RefCell<Option<RankedChampionDataSnapshot>>,
    advisor_snapshot: RefCell<Option<AdvisorDataSnapshot>>,
    last_activity_query: RefCell<Option<(i64, Option<ActivityKind>)>>,
    import_count: RefCell<usize>,
    clear_count: RefCell<usize>,
}

impl FakeStore {
    fn new(settings: AppSettings) -> Self {
        Self {
            settings: RefCell::new(settings),
            activity_entries: RefCell::new(Vec::new()),
            created_entries: RefCell::new(Vec::new()),
            imported_entries: RefCell::new(Vec::new()),
            player_notes: RefCell::new(Vec::new()),
            ranked_snapshot: RefCell::new(None),
            advisor_snapshot: RefCell::new(None),
            last_activity_query: RefCell::new(None),
            import_count: RefCell::new(0),
            clear_count: RefCell::new(0),
        }
    }
}

impl AppStore for FakeStore {
    fn schema_version(&self) -> Result<i64, String> {
        Ok(2)
    }

    fn get_settings(&self) -> Result<AppSettings, String> {
        Ok(self.settings.borrow().clone())
    }

    fn save_settings(&self, settings: SettingsValues) -> Result<AppSettings, String> {
        let updated = AppSettings {
            startup_page: settings.startup_page,
            language: settings.language,
            theme: settings.theme,
            compact_mode: settings.compact_mode,
            activity_limit: settings.activity_limit,
            auto_accept_enabled: settings.auto_accept_enabled,
            auto_pick_enabled: settings.auto_pick_enabled,
            auto_pick_champion_id: settings.auto_pick_champion_id,
            auto_pick_delay_seconds: settings.auto_pick_delay_seconds,
            auto_ban_enabled: settings.auto_ban_enabled,
            auto_ban_champion_id: settings.auto_ban_champion_id,
            auto_ban_delay_seconds: settings.auto_ban_delay_seconds,
            auto_honor_enabled: settings.auto_honor_enabled,
            ai_base_url: settings.ai_base_url,
            ai_api_key: settings.ai_api_key,
            ai_model: settings.ai_model,
            updated_at: "2026-04-18 00:00:00".to_string(),
        };

        self.settings.replace(updated.clone());
        Ok(updated)
    }

    fn list_activity_entries(
        &self,
        limit: i64,
        kind: Option<ActivityKind>,
    ) -> Result<Vec<ActivityEntry>, String> {
        self.last_activity_query.replace(Some((limit, kind)));

        Ok(self
            .activity_entries
            .borrow()
            .iter()
            .filter(|entry| kind.is_none_or(|value| entry.kind == value))
            .take(limit as usize)
            .cloned()
            .collect())
    }

    fn list_all_activity_entries(&self) -> Result<Vec<ActivityEntry>, String> {
        Ok(self.activity_entries.borrow().clone())
    }

    fn create_activity_entry(&self, entry: NewActivityEntry) -> Result<ActivityEntry, String> {
        self.created_entries.borrow_mut().push(entry.clone());

        Ok(ActivityEntry {
            id: self.created_entries.borrow().len() as i64,
            kind: entry.kind,
            title: entry.title,
            body: entry.body,
            created_at: "2026-04-18 00:00:00".to_string(),
        })
    }

    fn import_local_data(
        &self,
        settings: SettingsValues,
        activity_entries: Vec<LocalActivityEntry>,
    ) -> Result<ImportLocalDataResult, String> {
        *self.import_count.borrow_mut() += 1;
        let imported_activity_count = activity_entries.len();
        self.imported_entries.borrow_mut().extend(activity_entries);

        let settings = self.save_settings(settings)?;

        Ok(ImportLocalDataResult {
            settings,
            imported_activity_count,
        })
    }

    fn clear_activity_entries(&self) -> Result<i64, String> {
        *self.clear_count.borrow_mut() += 1;
        let deleted_count = self.activity_entries.borrow().len() as i64;
        self.activity_entries.borrow_mut().clear();
        Ok(deleted_count)
    }

    fn get_player_note(&self, player_puuid: &str) -> Result<Option<StoredPlayerNote>, String> {
        Ok(self
            .player_notes
            .borrow()
            .iter()
            .find(|note| note.player_puuid == player_puuid)
            .cloned())
    }

    fn save_player_note(&self, note: StoredPlayerNoteInput) -> Result<StoredPlayerNote, String> {
        let saved = StoredPlayerNote {
            player_puuid: note.player_puuid,
            last_display_name: note.last_display_name,
            note: note.note,
            tags: note.tags,
            updated_at: "2026-04-20 00:00:00".to_string(),
        };
        let mut notes = self.player_notes.borrow_mut();

        if let Some(existing) = notes
            .iter_mut()
            .find(|note| note.player_puuid == saved.player_puuid)
        {
            *existing = saved.clone();
        } else {
            notes.push(saved.clone());
        }

        Ok(saved)
    }

    fn clear_player_note(&self, player_puuid: &str) -> Result<bool, String> {
        let mut notes = self.player_notes.borrow_mut();
        let before = notes.len();
        notes.retain(|note| note.player_puuid != player_puuid);

        Ok(before != notes.len())
    }

    fn latest_ranked_champion_snapshot(
        &self,
    ) -> Result<Option<RankedChampionDataSnapshot>, String> {
        Ok(self.ranked_snapshot.borrow().clone())
    }

    fn replace_ranked_champion_snapshot(
        &self,
        snapshot: RankedChampionDataSnapshot,
    ) -> Result<RankedChampionDataSnapshot, String> {
        self.ranked_snapshot.replace(Some(snapshot.clone()));
        Ok(snapshot)
    }

    fn latest_advisor_snapshot(&self) -> Result<Option<AdvisorDataSnapshot>, String> {
        Ok(self.advisor_snapshot.borrow().clone())
    }

    fn replace_advisor_snapshot(
        &self,
        snapshot: AdvisorDataSnapshot,
    ) -> Result<AdvisorDataSnapshot, String> {
        self.advisor_snapshot.replace(Some(snapshot.clone()));
        Ok(snapshot)
    }

    fn get_champion_rune_config(
        &self,
        _champion_id: i64,
    ) -> Result<Option<ChampionRuneConfig>, String> {
        Ok(None)
    }

    fn save_champion_rune_config(
        &self,
        champion_id: i64,
        page: RunePage,
    ) -> Result<ChampionRuneConfig, String> {
        Ok(ChampionRuneConfig {
            champion_id,
            page,
            saved_at: "2026-05-22 00:00:00".to_string(),
        })
    }

    fn delete_champion_rune_config(&self, _champion_id: i64) -> Result<bool, String> {
        Ok(false)
    }

    fn get_ai_analysis(&self, _scope: &str) -> Result<Option<domain::AiAnalysisCache>, String> {
        Ok(None)
    }

    fn save_ai_analysis(&self, _scope: &str, _result_text: &str, _game_count: i64) -> Result<(), String> {
        Ok(())
    }

    fn list_chat_presets(&self) -> Result<Vec<domain::ChatPreset>, String> {
        Ok(Vec::new())
    }

    fn save_chat_preset(
        &self,
        slot: i64,
        label: &str,
        message: &str,
    ) -> Result<domain::ChatPreset, String> {
        Ok(domain::ChatPreset {
            slot,
            label: label.to_string(),
            message: message.to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        })
    }

    fn delete_chat_preset(&self, _slot: i64) -> Result<bool, String> {
        Ok(true)
    }
}

struct FakeRankedChampionProvider {
    snapshot: RankedChampionDataSnapshot,
}

impl RankedChampionDataProvider for FakeRankedChampionProvider {
    fn fetch_ranked_champion_snapshot(
        &self,
        _input: RankedChampionRefreshInput,
    ) -> Result<RankedChampionDataSnapshot, RankedChampionDataError> {
        Ok(self.snapshot.clone())
    }
}

struct FakeAdvisorProvider {
    snapshot: AdvisorDataSnapshot,
}

impl AdvisorDataProvider for FakeAdvisorProvider {
    fn fetch_advisor_snapshot(
        &self,
        _input: AdvisorDataRefreshInput,
    ) -> Result<AdvisorDataSnapshot, RankedChampionDataError> {
        Ok(self.snapshot.clone())
    }
}

struct FailingRankedChampionProvider;

impl RankedChampionDataProvider for FailingRankedChampionProvider {
    fn fetch_ranked_champion_snapshot(
        &self,
        _input: RankedChampionRefreshInput,
    ) -> Result<RankedChampionDataSnapshot, RankedChampionDataError> {
        Err(RankedChampionDataError::Unavailable(
            "remote unavailable".to_string(),
        ))
    }
}

fn default_settings() -> AppSettings {
    AppSettings {
        startup_page: StartupPage::Dashboard,
        language: AppLanguagePreference::System,
        theme: domain::AppThemePreference::Light,
        compact_mode: false,
        activity_limit: 100,
        auto_accept_enabled: true,
        auto_pick_enabled: false,
        auto_pick_champion_id: None,
        auto_pick_delay_seconds: 0.0,
        auto_ban_enabled: false,
        auto_ban_champion_id: None,
        auto_ban_delay_seconds: 0.0,
        auto_honor_enabled: false,
        ai_base_url: None,
        ai_api_key: None,
        ai_model: None,
        updated_at: "2026-04-18 00:00:00".to_string(),
    }
}

fn sample_activity(id: i64) -> ActivityEntry {
    ActivityEntry {
        id,
        kind: ActivityKind::Note,
        title: format!("Activity {id}"),
        body: None,
        created_at: "2026-04-18 00:00:00".to_string(),
    }
}

fn sample_ranked_snapshot(source: &str) -> RankedChampionDataSnapshot {
    RankedChampionDataSnapshot {
        source: source.to_string(),
        patch: Some("26.08".to_string()),
        region: Some("KR".to_string()),
        queue: Some("RANKED_SOLO_5X5".to_string()),
        tier: Some("EMERALD_PLUS".to_string()),
        generated_at: Some("2026-04-25T00:00:00Z".to_string()),
        imported_at: "2026-04-25 00:00:00".to_string(),
        records: vec![
            RankedChampionStat {
                champion_id: 103,
                champion_name: "Ahri".to_string(),
                champion_alias: Some("Ahri".to_string()),
                lane: RankedChampionLane::Middle,
                win_rate: 51.4,
                pick_rate: 10.0,
                ban_rate: 8.0,
                overall_score: 90.0,
                games: 1000,
                wins: 514,
                picks: 1000,
                bans: 80,
            },
            RankedChampionStat {
                champion_id: 222,
                champion_name: "Jinx".to_string(),
                champion_alias: Some("Jinx".to_string()),
                lane: RankedChampionLane::Bottom,
                win_rate: 52.1,
                pick_rate: 12.0,
                ban_rate: 6.0,
                overall_score: 88.0,
                games: 1200,
                wins: 625,
                picks: 1200,
                bans: 72,
            },
        ],
    }
}

fn sample_advisor_fixture(source: &str) -> AdvisorDataSnapshot {
    AdvisorDataSnapshot {
        source: source.to_string(),
        patch: Some("26.08".to_string()),
        region: Some("KR".to_string()),
        queue: Some("RANKED_SOLO_5X5".to_string()),
        tier: Some("EMERALD_PLUS".to_string()),
        generated_at: Some("2026-04-25T00:00:00Z".to_string()),
        imported_at: "2026-04-25 00:00:00".to_string(),
        records: vec![
            sample_advisor_record_fixture(
                86,
                "Garen",
                RankedChampionLane::Top,
                52.5,
                vec![122],
                Vec::new(),
                "Trade when Q is ready.",
            ),
            sample_advisor_record_fixture(
                122,
                "Darius",
                RankedChampionLane::Top,
                49.0,
                Vec::new(),
                vec![86],
                "Punish short trades.",
            ),
        ],
    }
}

fn sample_advisor_record_fixture(
    champion_id: i64,
    champion_name: &str,
    lane: RankedChampionLane,
    win_rate: f64,
    strong_against: Vec<i64>,
    weak_against: Vec<i64>,
    lane_advice: &str,
) -> AdvisorRecord {
    AdvisorRecord {
        champion_id,
        champion_name: champion_name.to_string(),
        champion_alias: Some(champion_name.to_string()),
        lane,
        win_rate,
        pick_rate: 8.0,
        ban_rate: 4.0,
        overall_score: win_rate,
        games: 10_000,
        runes: AdvisorRunePage {
            primary_style: "Precision".to_string(),
            primary_runes: vec![AdvisorNamedRef {
                id: Some(8010),
                name: "Conqueror".to_string(),
            }],
            secondary_style: "Resolve".to_string(),
            secondary_runes: vec![AdvisorNamedRef {
                id: Some(8444),
                name: "Second Wind".to_string(),
            }],
            stat_shards: vec!["Adaptive Force".to_string()],
        },
        summoner_spells: vec![
            AdvisorNamedRef {
                id: Some(4),
                name: "Flash".to_string(),
            },
            AdvisorNamedRef {
                id: Some(14),
                name: "Ignite".to_string(),
            },
        ],
        skill_order: AdvisorSkillOrder {
            max_order: vec!["Q".to_string(), "E".to_string(), "W".to_string()],
            early_order: vec!["Q".to_string(), "E".to_string(), "W".to_string()],
        },
        item_build: AdvisorItemBuild {
            starter: vec![AdvisorNamedRef {
                id: Some(1055),
                name: "Doran's Blade".to_string(),
            }],
            core: vec![AdvisorNamedRef {
                id: Some(6631),
                name: "Stridebreaker".to_string(),
            }],
            boots: vec![AdvisorNamedRef {
                id: Some(3047),
                name: "Plated Steelcaps".to_string(),
            }],
            late: vec![AdvisorNamedRef {
                id: Some(3053),
                name: "Sterak's Gage".to_string(),
            }],
            situational: vec![AdvisorNamedRef {
                id: Some(3156),
                name: "Maw of Malmortius".to_string(),
            }],
        },
        strong_against: strong_against
            .into_iter()
            .map(|champion_id| AdvisorMatchup {
                champion_id,
                champion_name: if champion_id == 122 {
                    "Darius"
                } else {
                    "Garen"
                }
                .to_string(),
                note: "Punish cooldowns.".to_string(),
                win_rate_delta: Some(2.0),
            })
            .collect(),
        weak_against: weak_against
            .into_iter()
            .map(|champion_id| AdvisorMatchup {
                champion_id,
                champion_name: if champion_id == 122 {
                    "Darius"
                } else {
                    "Garen"
                }
                .to_string(),
                note: "Respect early all-in.".to_string(),
                win_rate_delta: Some(-2.0),
            })
            .collect(),
        power_spikes: vec![AdvisorPowerSpike {
            timing: "6".to_string(),
            label: "All-in".to_string(),
            description: "Look for ultimate windows.".to_string(),
        }],
        lane_advice: lane_advice.to_string(),
        teamfight_advice: "Play front to back.".to_string(),
    }
}

fn sample_live_overlay_snapshot() -> LiveOverlaySnapshot {
    LiveOverlaySnapshot {
        game_time_seconds: Some(300.0),
        game_mode: Some("CLASSIC".to_string()),
        map_name: Some("Summoner's Rift".to_string()),
        active_player: Some(domain::LiveOverlayActivePlayer {
            display_name: "Player One".to_string(),
            level: Some(6),
            current_gold: Some(750.0),
            resource_type: Some("MANA".to_string()),
            resource_value: Some(100.0),
            resource_max: Some(300.0),
        }),
        players: Vec::new(),
        events: Vec::new(),
        gold: domain::LiveOverlayGoldSummary {
            ally_item_value: 3000,
            enemy_item_value: 2500,
            item_value_diff: 500,
        },
        refreshed_at: "1".to_string(),
    }
}

struct FakeLeagueClientReader {
    champ_select_session: ChampSelectSessionData,
    data: LeagueSelfData,
    completed_match: Mutex<Option<LeagueCompletedMatch>>,
    failed_recent_puuids: Vec<String>,
    gameflow_phase: Mutex<String>,
    last_match_limit: Mutex<Option<i64>>,
    ready_check_accepts: Mutex<i64>,
    ready_check_clears_after: Option<i64>,
    ready_check_next_phase: String,
    ready_check_accept_error: Option<LeagueClientReadError>,
    champ_select_preference_calls: Mutex<i64>,
    recent_stats_batch_calls: Mutex<Vec<Vec<String>>>,
    summoners_by_id: Vec<SummonerBatchEntry>,
    summoners_by_name: Vec<SummonerBatchEntry>,
    last_chat_status: Mutex<Option<(Option<String>, Option<String>)>>,
}

impl FakeLeagueClientReader {
    fn new(recent_matches: Vec<RecentMatchSummary>) -> Self {
        Self::with_data(LeagueSelfData {
            status: connected_status(),
            summoner: None,
            ranked_queues: Vec::new(),
            recent_matches,
            data_warnings: Vec::new(),
        })
    }

    fn with_data(data: LeagueSelfData) -> Self {
        Self {
            champ_select_session: ChampSelectSessionData {
                ally_ids: Vec::new(),
                enemy_ids: Vec::new(),
                champion_selections: HashMap::new(),
                ally_names: Vec::new(),
                enemy_names: Vec::new(),
                champion_selections_by_name: HashMap::new(),
                source: ChampSelectSessionSource::ChampSelect,
                players: Vec::new(),
            },
            data,
            completed_match: Mutex::new(None),
            failed_recent_puuids: Vec::new(),
            gameflow_phase: Mutex::new("None".to_string()),
            last_match_limit: Mutex::new(None),
            ready_check_accepts: Mutex::new(0),
            ready_check_clears_after: None,
            ready_check_next_phase: "ChampSelect".to_string(),
            ready_check_accept_error: None,
            champ_select_preference_calls: Mutex::new(0),
            recent_stats_batch_calls: Mutex::new(Vec::new()),
            summoners_by_id: Vec::new(),
            summoners_by_name: Vec::new(),
            last_chat_status: Mutex::new(None),
        }
    }

    fn with_completed_match(completed_match: LeagueCompletedMatch) -> Self {
        Self {
            champ_select_session: ChampSelectSessionData {
                ally_ids: Vec::new(),
                enemy_ids: Vec::new(),
                champion_selections: HashMap::new(),
                ally_names: Vec::new(),
                enemy_names: Vec::new(),
                champion_selections_by_name: HashMap::new(),
                source: ChampSelectSessionSource::ChampSelect,
                players: Vec::new(),
            },
            data: LeagueSelfData {
                status: connected_status(),
                summoner: None,
                ranked_queues: Vec::new(),
                recent_matches: Vec::new(),
                data_warnings: Vec::new(),
            },
            completed_match: Mutex::new(Some(completed_match)),
            failed_recent_puuids: Vec::new(),
            gameflow_phase: Mutex::new("None".to_string()),
            last_match_limit: Mutex::new(None),
            ready_check_accepts: Mutex::new(0),
            ready_check_clears_after: None,
            ready_check_next_phase: "ChampSelect".to_string(),
            ready_check_accept_error: None,
            champ_select_preference_calls: Mutex::new(0),
            recent_stats_batch_calls: Mutex::new(Vec::new()),
            summoners_by_id: Vec::new(),
            summoners_by_name: Vec::new(),
            last_chat_status: Mutex::new(None),
        }
    }

    fn with_champ_select_data(
        champ_select_session: ChampSelectSessionData,
        summoners_by_id: Vec<SummonerBatchEntry>,
        failed_recent_puuids: Vec<String>,
    ) -> Self {
        let mut reader = Self::new(Vec::new());
        reader.champ_select_session = champ_select_session;
        reader.summoners_by_id = summoners_by_id;
        reader.failed_recent_puuids = failed_recent_puuids;
        reader
    }

    fn with_ready_check_phase(self) -> Self {
        *self.gameflow_phase.lock().unwrap() = "ReadyCheck".to_string();
        self
    }

    fn with_phase_transition_after_accepts(mut self, accepts: i64, next_phase: &str) -> Self {
        *self.gameflow_phase.lock().unwrap() = "ReadyCheck".to_string();
        self.ready_check_clears_after = Some(accepts);
        self.ready_check_next_phase = next_phase.to_string();
        if accepts <= 0 {
            *self.gameflow_phase.lock().unwrap() = next_phase.to_string();
        }
        self
    }

    fn with_ready_check_accept_error(mut self, error: LeagueClientReadError) -> Self {
        *self.gameflow_phase.lock().unwrap() = "ReadyCheck".to_string();
        self.ready_check_accept_error = Some(error);
        self
    }

    fn accept_ready_check_count(&self) -> i64 {
        *self.ready_check_accepts.lock().unwrap()
    }

    fn champ_select_preference_call_count(&self) -> i64 {
        *self.champ_select_preference_calls.lock().unwrap()
    }

    fn recent_stats_batch_calls(&self) -> Vec<Vec<String>> {
        self.recent_stats_batch_calls.lock().unwrap().clone()
    }
}

impl LeagueClientReader for FakeLeagueClientReader {
    fn status(&self) -> Result<LeagueClientStatus, LeagueClientReadError> {
        Ok(self.data.status.clone())
    }

    fn gameflow_phase(&self) -> Result<String, LeagueClientReadError> {
        Ok(self.gameflow_phase.lock().unwrap().clone())
    }

    fn self_data(&self, match_limit: i64) -> Result<LeagueSelfData, LeagueClientReadError> {
        *self.last_match_limit.lock().unwrap() = Some(match_limit);

        Ok(LeagueSelfData {
            status: self.data.status.clone(),
            summoner: self.data.summoner.clone(),
            ranked_queues: self.data.ranked_queues.clone(),
            recent_matches: self
                .data
                .recent_matches
                .iter()
                .take(match_limit as usize)
                .cloned()
                .collect(),
            data_warnings: self.data.data_warnings.clone(),
        })
    }

    fn search_player(
        &self,
        query: &str,
        match_limit: i64,
    ) -> Result<Option<domain::PlayerSearchData>, LeagueClientReadError> {
        // Sentinel for the "client reachable but no such summoner" path.
        if query == "missing" {
            return Ok(None);
        }
        Ok(Some(domain::PlayerSearchData {
            self_data: self.self_data(match_limit)?,
            playstyle_matches: Vec::new(),
        }))
    }

    fn profile_icon(
        &self,
        profile_icon_id: i64,
    ) -> Result<LeagueImageAsset, LeagueClientReadError> {
        Ok(LeagueImageAsset {
            mime_type: "image/jpeg".to_string(),
            bytes: vec![profile_icon_id as u8],
        })
    }

    fn champion_icon(&self, champion_id: i64) -> Result<LeagueImageAsset, LeagueClientReadError> {
        Ok(LeagueImageAsset {
            mime_type: "image/png".to_string(),
            bytes: vec![champion_id as u8],
        })
    }

    fn game_asset(
        &self,
        kind: LeagueGameAssetKind,
        asset_id: i64,
    ) -> Result<LeagueGameAsset, LeagueClientReadError> {
        Ok(LeagueGameAsset {
            kind,
            asset_id,
            name: format!("{kind:?} {asset_id}"),
            description: Some("Local game data asset".to_string()),
            image: LeagueImageAsset {
                mime_type: "image/png".to_string(),
                bytes: vec![asset_id as u8],
            },
        })
    }

    fn completed_match(&self, game_id: i64) -> Result<LeagueCompletedMatch, LeagueClientReadError> {
        self.completed_match
            .lock()
            .unwrap()
            .clone()
            .filter(|completed_match| completed_match.game_id == game_id)
            .ok_or_else(|| {
                LeagueClientReadError::Integration(
                    "Completed match was not found in current user's recent history".to_string(),
                )
            })
    }

    fn participant_recent_stats(
        &self,
        player_puuid: &str,
        limit: i64,
    ) -> Result<ParticipantRecentStats, LeagueClientReadError> {
        if self
            .failed_recent_puuids
            .iter()
            .any(|value| value == player_puuid)
        {
            return Err(LeagueClientReadError::Integration(
                "Recent stats unavailable".to_string(),
            ));
        }

        let recent_matches = (1..=limit)
            .map(|id| sample_match(id, format!("Recent Champion {id}").as_str(), 5, 2, 7))
            .collect();

        Ok(ParticipantRecentStats {
            match_count: limit as usize,
            average_kda: Some(3.5),
            recent_champions: vec!["Ahri".to_string()],
            recent_matches,
        })
    }

    fn participant_recent_stats_batch(
        &self,
        player_puuids: &[String],
        limit: i64,
    ) -> HashMap<String, Result<ParticipantRecentStats, LeagueClientReadError>> {
        self.recent_stats_batch_calls
            .lock()
            .unwrap()
            .push(player_puuids.to_vec());

        player_puuids
            .iter()
            .map(|player_puuid| {
                (
                    player_puuid.clone(),
                    self.participant_recent_stats(player_puuid, limit),
                )
            })
            .collect()
    }

    fn champ_select_session(&self) -> Result<ChampSelectSessionData, LeagueClientReadError> {
        Ok(self.champ_select_session.clone())
    }

    fn summoners_by_ids(&self, ids: &[i64]) -> Vec<SummonerBatchEntry> {
        self.summoners_by_id
            .iter()
            .filter(|entry| ids.contains(&entry.summoner_id))
            .cloned()
            .collect()
    }

    fn summoners_by_names(&self, names: &[String]) -> Vec<SummonerBatchEntry> {
        let normalized_names: HashSet<String> = names
            .iter()
            .map(|name| normalize_player_name(name.as_str()))
            .collect();
        self.summoners_by_name
            .iter()
            .filter(|entry| {
                summoner_name_lookup_keys(entry.display_name.as_str())
                    .iter()
                    .any(|key| normalized_names.contains(key))
            })
            .cloned()
            .collect()
    }

    fn champion_catalog(&self) -> Result<Vec<LeagueChampionSummary>, LeagueClientReadError> {
        Ok(vec![LeagueChampionSummary {
            champion_id: 103,
            champion_name: "Ahri".to_string(),
            champion_alias: Some("Ahri".to_string()),
        }])
    }

    fn champion_details(
        &self,
        champion_id: i64,
    ) -> Result<LeagueChampionDetails, LeagueClientReadError> {
        Ok(LeagueChampionDetails {
            champion_id,
            champion_name: "Ahri".to_string(),
            title: Some("the Nine-Tailed Fox".to_string()),
            square_portrait: Some(LeagueImageAsset {
                mime_type: "image/png".to_string(),
                bytes: vec![champion_id as u8],
            }),
            abilities: vec![domain::LeagueChampionAbility {
                slot: "Q".to_string(),
                name: "Orb of Deception".to_string(),
                description: "Ahri sends out and pulls back her orb.".to_string(),
                summary_description: "Ahri sends out and pulls back her orb.".to_string(),
                icon: Some(LeagueImageAsset {
                    mime_type: "image/png".to_string(),
                    bytes: vec![1],
                }),
                cooldown: Some("7".to_string()),
                cost: Some("55".to_string()),
                range: Some("880".to_string()),
                cooldown_values: vec!["7".to_string()],
                cost_values: vec!["55".to_string()],
                range_values: vec!["880".to_string()],
                stats: vec![],
            }],
        })
    }

    fn live_overlay(&self) -> Result<LiveOverlaySnapshot, LeagueClientReadError> {
        Ok(sample_live_overlay_snapshot())
    }

    fn accept_ready_check(&self) -> Result<(), LeagueClientReadError> {
        let mut accept_count = self.ready_check_accepts.lock().unwrap();
        *accept_count += 1;

        if let Some(target_accepts) = self.ready_check_clears_after {
            if *accept_count >= target_accepts {
                *self.gameflow_phase.lock().unwrap() = self.ready_check_next_phase.clone();
            }
        }

        if let Some(error) = &self.ready_check_accept_error {
            return Err(error.clone());
        }

        Ok(())
    }

    fn chat_me(&self) -> Result<domain::ChatMe, LeagueClientReadError> {
        Ok(domain::ChatMe {
            availability: Some("chat".to_string()),
            status_message: Some("hello".to_string()),
        })
    }

    fn set_chat_status(
        &self,
        status_message: Option<&str>,
        availability: Option<&str>,
    ) -> Result<(), LeagueClientReadError> {
        *self.last_chat_status.lock().unwrap() = Some((
            status_message.map(str::to_string),
            availability.map(str::to_string),
        ));
        Ok(())
    }

    fn apply_rune_page(
        &self,
        _page: &domain::RunePage,
        _champion_name: &str,
    ) -> Result<(), LeagueClientReadError> {
        Ok(())
    }

    fn apply_champ_select_preferences(
        &self,
        _pick_champion_id: Option<i64>,
        _ban_champion_id: Option<i64>,
    ) -> Result<(), LeagueClientReadError> {
        *self.champ_select_preference_calls.lock().unwrap() += 1;
        Ok(())
    }
}

fn connected_status() -> LeagueClientStatus {
    LeagueClientStatus {
        is_running: true,
        lockfile_found: true,
        connection: LeagueClientConnection::Connected,
        phase: LeagueClientPhase::Connected,
        message: None,
    }
}

fn high_kda_match(id: i64) -> RecentMatchSummary {
    sample_match(id, format!("Champion {id}").as_str(), 6, 1, 4)
}

fn sample_match(
    game_id: i64,
    champion_name: &str,
    kills: i64,
    deaths: i64,
    assists: i64,
) -> RecentMatchSummary {
    RecentMatchSummary {
        game_id,
        champion_id: Some(game_id),
        champion_name: champion_name.to_string(),
        queue_name: Some("Ranked Solo/Duo".to_string()),
        lane: None,
        result: MatchResult::Win,
        kills,
        deaths,
        assists,
        kda: None,
        played_at: Some("2026-04-19T12:00:00Z".to_string()),
        game_duration_seconds: Some(1800),
    }
}

#[test]
fn champion_records_aggregate_wins_losses_per_champion() {
    let make = |champion_id: Option<i64>, result: MatchResult| RecentMatchSummary {
        game_id: 0,
        champion_id,
        champion_name: "X".to_string(),
        queue_name: None,
        lane: None,
        result,
        kills: 0,
        deaths: 0,
        assists: 0,
        kda: None,
        played_at: None,
        game_duration_seconds: None,
    };

    let matches = vec![
        make(Some(103), MatchResult::Win),
        make(Some(103), MatchResult::Loss),
        make(Some(103), MatchResult::Win),
        make(Some(64), MatchResult::Loss),
        make(Some(64), MatchResult::Unknown),
        make(None, MatchResult::Win), // dropped: no champion id
    ];

    let records = super::summarize_champion_records(&matches);

    assert_eq!(records.len(), 2, "the match with no champion id is dropped");

    let ahri = records.iter().find(|r| r.champion_id == 103).expect("ahri record");
    assert_eq!(ahri.wins, 2);
    assert_eq!(ahri.losses, 1);
    assert_eq!(ahri.games, 3);

    let lee = records.iter().find(|r| r.champion_id == 64).expect("lee record");
    assert_eq!(lee.wins, 0);
    assert_eq!(lee.losses, 1);
    // Unknown results still count as a game played but not toward W/L.
    assert_eq!(lee.games, 2);
}

fn sample_completed_match() -> LeagueCompletedMatch {
    LeagueCompletedMatch {
        game_id: 10,
        queue_name: Some("Ranked Solo/Duo".to_string()),
        played_at: Some("2026-04-19T12:00:00Z".to_string()),
        game_duration_seconds: Some(1880),
        result: MatchResult::Win,
        self_participant_id: Some(1),
        participants: vec![
            LeagueCompletedParticipant {
                participant_id: 1,
                team_id: 100,
                display_name: "Player One".to_string(),
                player_puuid: Some("self-puuid".to_string()),
                profile_icon_id: Some(1),
                champion_id: Some(103),
                champion_name: "Ahri".to_string(),
                role: Some("SOLO".to_string()),
                lane: Some("MIDDLE".to_string()),
                result: MatchResult::Win,
                kills: 7,
                deaths: 1,
                assists: 8,
                kda: Some(15.0),
                cs: 210,
                gold_earned: 12_000,
                damage_to_champions: 22_000,
                physical_damage_to_champions: 0,
                magic_damage_to_champions: 22_000,
                true_damage_to_champions: 0,
                damage_to_objectives: 5_000,
                damage_to_turrets: 3_000,
                damage_taken: 18_000,
                vision_score: 18,
                wards_placed: 4,
                wards_killed: 2,
                control_wards_bought: 3,
                time_spent_dead_seconds: 30,
                largest_killing_spree: 4,
                largest_multi_kill: 2,
                double_kills: 2,
                triple_kills: 0,
                quadra_kills: 0,
                penta_kills: 0,
                first_blood: true,
                first_tower: false,
                items: vec![1056, 3020],
                runes: vec![8112],
                spells: vec![4, 14],
            },
            LeagueCompletedParticipant {
                participant_id: 2,
                team_id: 200,
                display_name: "Player Two".to_string(),
                player_puuid: Some("enemy-puuid".to_string()),
                profile_icon_id: Some(2),
                champion_id: Some(266),
                champion_name: "Aatrox".to_string(),
                role: Some("SOLO".to_string()),
                lane: Some("TOP".to_string()),
                result: MatchResult::Loss,
                kills: 5,
                deaths: 7,
                assists: 4,
                kda: Some(1.3),
                cs: 180,
                gold_earned: 10_000,
                damage_to_champions: 25_000,
                physical_damage_to_champions: 25_000,
                magic_damage_to_champions: 0,
                true_damage_to_champions: 0,
                damage_to_objectives: 4_000,
                damage_to_turrets: 2_000,
                damage_taken: 22_000,
                vision_score: 12,
                wards_placed: 3,
                wards_killed: 1,
                control_wards_bought: 2,
                time_spent_dead_seconds: 120,
                largest_killing_spree: 3,
                largest_multi_kill: 1,
                double_kills: 1,
                triple_kills: 0,
                quadra_kills: 0,
                penta_kills: 0,
                first_blood: false,
                first_tower: false,
                items: vec![1055, 3047],
                runes: vec![8010],
                spells: vec![4, 12],
            },
        ],
    }
}

// ─── recent_stats diagnosis ──────────────────────────────────────────────────
//
// These tests pin the per-player diagnostic behavior so future refactors don't
// regress the runtime explanation of why a player's history is missing.

fn sample_recent_stats(match_count: usize) -> ParticipantRecentStats {
    use domain::{MatchResult, RecentMatchSummary};
    let matches = (0..match_count)
        .map(|i| RecentMatchSummary {
            game_id: i as i64,
            champion_id: Some(103),
            champion_name: "Ahri".to_string(),
            queue_name: None,
            lane: None,
            result: MatchResult::Win,
            kills: 5,
            deaths: 2,
            assists: 7,
            kda: Some(6.0),
            played_at: None,
            game_duration_seconds: None,
        })
        .collect();
    ParticipantRecentStats {
        match_count,
        average_kda: Some(6.0),
        recent_champions: vec!["Ahri".to_string()],
        recent_matches: matches,
    }
}

#[test]
fn diagnose_recent_stats_reports_not_requested_when_limit_is_zero() {
    let diagnosis = diagnose_recent_stats(0, "puuid-1", None);
    assert_eq!(diagnosis, RecentStatsDiagnosis::NotRequested);
    assert_eq!(
        diagnosis.public_status(),
        ChampSelectRecentStatsStatus::NotRequested
    );
    assert!(!diagnosis.should_log_to_stderr());
}

#[test]
fn diagnose_recent_stats_reports_missing_identity_when_puuid_is_blank() {
    for puuid in ["", "   ", "\t"] {
        let diagnosis = diagnose_recent_stats(20, puuid, None);
        assert_eq!(diagnosis, RecentStatsDiagnosis::MissingIdentity, "input={puuid:?}");
        assert_eq!(
            diagnosis.public_status(),
            ChampSelectRecentStatsStatus::MissingIdentity
        );
        assert!(diagnosis.should_log_to_stderr());
        assert!(diagnosis.log_phrase().contains("MISSING_IDENTITY"));
        assert!(
            diagnosis.log_phrase().contains("anti-dodge"),
            "log phrase must hint at Riot's anonymization so users know why enemies are blank"
        );
    }
}

#[test]
fn diagnose_recent_stats_reports_loaded_with_match_count() {
    let stats: Result<ParticipantRecentStats, LeagueClientReadError> = Ok(sample_recent_stats(7));
    let diagnosis = diagnose_recent_stats(20, "puuid-1", Some(&stats));
    assert_eq!(diagnosis, RecentStatsDiagnosis::Loaded(7));
    assert_eq!(
        diagnosis.public_status(),
        ChampSelectRecentStatsStatus::Loaded
    );
    assert!(!diagnosis.should_log_to_stderr());
    assert_eq!(diagnosis.log_phrase(), "OK matches=7");
}

#[test]
fn diagnose_recent_stats_reports_empty_history_when_lcu_returns_zero_games() {
    let stats: Result<ParticipantRecentStats, LeagueClientReadError> = Ok(sample_recent_stats(0));
    let diagnosis = diagnose_recent_stats(20, "puuid-1", Some(&stats));
    assert_eq!(diagnosis, RecentStatsDiagnosis::LoadedEmpty);
    assert_eq!(
        diagnosis.public_status(),
        ChampSelectRecentStatsStatus::Loaded,
        "an empty-but-successful response is still public-Loaded"
    );
    assert!(diagnosis.should_log_to_stderr());
    assert!(diagnosis.log_phrase().contains("EMPTY_HISTORY"));
}

#[test]
fn diagnose_recent_stats_reports_lcu_error_with_underlying_message() {
    let error: Result<ParticipantRecentStats, LeagueClientReadError> = Err(
        LeagueClientReadError::Integration("403 Forbidden".to_string()),
    );
    let diagnosis = diagnose_recent_stats(20, "puuid-1", Some(&error));
    assert_eq!(
        diagnosis,
        RecentStatsDiagnosis::LcuError("403 Forbidden".to_string())
    );
    assert_eq!(
        diagnosis.public_status(),
        ChampSelectRecentStatsStatus::Unavailable
    );
    assert!(diagnosis.should_log_to_stderr());
    let phrase = diagnosis.log_phrase();
    assert!(phrase.contains("LCU_ERROR"));
    assert!(
        phrase.contains("403 Forbidden"),
        "the underlying LCU error must surface in the log so the user can read it"
    );
}

#[test]
fn diagnose_recent_stats_reports_no_result_when_batch_is_missing_entry() {
    // PUUID looked valid but the batch HashMap had no entry — upstream
    // batching bug. The diagnosis must point at that case explicitly.
    let diagnosis = diagnose_recent_stats(20, "puuid-1", None);
    assert_eq!(diagnosis, RecentStatsDiagnosis::NoResult);
    assert_eq!(
        diagnosis.public_status(),
        ChampSelectRecentStatsStatus::Unavailable
    );
    assert!(diagnosis.should_log_to_stderr());
    assert!(diagnosis.log_phrase().contains("NO_RESULT"));
}

fn playstyle_match(
    champion_id: i64,
    role: &str,
    win: bool,
) -> domain::PlaystyleMatchStat {
    domain::PlaystyleMatchStat {
        champion_id: Some(champion_id),
        role: Some(role.to_string()),
        win,
        duration_seconds: 1_800, // 30 min
        kills: 5,
        deaths: 3,
        assists: 7,
        team_kills: 30,
        cs: 180,
        gold: 12_000,
        damage_to_champions: 18_000,
        damage_taken: 20_000,
        damage_to_objectives: 5_000,
        damage_to_turrets: 1_000,
        vision_score: 20,
        wards_placed: 12,
        control_wards_bought: 3,
        multikills: 0,
        largest_killing_spree: 2,
        first_blood: false,
        first_tower: false,
        time_spent_dead_seconds: 120,
    }
}

fn has_tag(profile: &domain::PlaystyleProfile, kind: domain::PlaystyleTagKind) -> bool {
    profile.tags.iter().any(|tag| tag.kind == kind)
}

#[test]
fn playstyle_profile_is_empty_below_minimum_games() {
    let matches: Vec<_> = (0..3).map(|_| playstyle_match(1, "MIDDLE", true)).collect();
    let profile = compute_playstyle_profile(&matches);
    assert_eq!(profile.games_analyzed, 3);
    assert!(profile.tags.is_empty());
}

#[test]
fn playstyle_profile_tags_a_farming_carry_mid() {
    // High CS, high damage, low deaths, one champion, on a win streak.
    let matches: Vec<_> = (0..20)
        .map(|_| {
            let mut game = playstyle_match(1, "MIDDLE", true);
            game.cs = 260; // ~8.7 cs/min
            game.damage_to_champions = 24_000; // ~800 dpm
            game.deaths = 2;
            game.largest_killing_spree = 6;
            game
        })
        .collect();
    let profile = compute_playstyle_profile(&matches);

    assert_eq!(profile.games_analyzed, 20);
    assert!(has_tag(&profile, domain::PlaystyleTagKind::DamageDealer));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::Farmer));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::SolidLaner));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::SafePlayer));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::Duelist));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::OneTrick));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::Specialist));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::OnFire));
    // A carry mid is not a frontline tank.
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::Frontline));
}

#[test]
fn playstyle_profile_tags_a_vision_support() {
    // Support: no CS, high vision, high assists, loss streak.
    let matches: Vec<_> = (0..20)
        .map(|_| {
            let mut game = playstyle_match(2, "UTILITY", false);
            game.cs = 30;
            game.vision_score = 55;
            game.kills = 1;
            game.assists = 18;
            game.deaths = 5;
            game.damage_to_champions = 8_000;
            game.largest_killing_spree = 1;
            game
        })
        .collect();
    let profile = compute_playstyle_profile(&matches);

    assert!(has_tag(&profile, domain::PlaystyleTagKind::VisionFocused));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::Playmaker));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::ColdStreak));
    // Supports never qualify as farmers or damage dealers.
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::Farmer));
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::DamageDealer));
}

#[test]
fn playstyle_profile_tags_a_struggling_mid_and_stays_consistent() {
    // Low CS, high deaths, no vision, no pinks, long time dead.
    let matches: Vec<_> = (0..20)
        .map(|_| {
            let mut game = playstyle_match(3, "MIDDLE", false);
            game.cs = 120; // 4.0 cs/min
            game.kills = 2;
            game.deaths = 8;
            game.assists = 4;
            game.damage_to_champions = 9_000; // ~300 dpm, not a damage dealer
            game.vision_score = 8;
            game.control_wards_bought = 0;
            game.time_spent_dead_seconds = 320;
            game.largest_killing_spree = 1;
            game
        })
        .collect();
    let profile = compute_playstyle_profile(&matches);

    assert!(has_tag(&profile, domain::PlaystyleTagKind::LackingLaner));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::Farmingphobia));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::BadDuelist));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::RiskTaker));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::AlreadyDead));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::Visionless));
    assert!(has_tag(&profile, domain::PlaystyleTagKind::NoEarlyPinks));

    // "要合理": a tag and its positive counterpart must never both appear.
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::Farmer));
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::SolidLaner));
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::Duelist));
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::VisionFocused));
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::SafePlayer));
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::Escapist));
    // Visionless already covers the low-vision case, so don't also stack Early Blindness.
    assert!(!has_tag(&profile, domain::PlaystyleTagKind::EarlyBlindness));
}
