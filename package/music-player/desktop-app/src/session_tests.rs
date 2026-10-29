// Session migration and current-wire tests. This file is a child of session.rs.

// Bring parent module declarations into the test module.
use super::*;

/// Current JSON round-trips every field and emits only one playback setting.
#[test]
fn current_json_round_trip_preserves_fields() {
    let original = Session {
        source_root: Some(PathBuf::from("/music/Artist")),
        selected: Some(PathBuf::from("/music/Artist/01.flac")),
        position_secs: 12.5,
        volume: 0.7,
        playback_mode: PlaybackMode::ShufflePage,
        track_rows_template: Some("$mi(title)$".to_string()),
        playing_template: None,
        auto_play_on_select: true,
        text_sizes: TextSizes::default().with(TextElement::TrackTitle, 18).with(TextElement::PaneTitle, 26),
    };
    let json = serde_json::to_string(&original).expect("current session should serialize");
    let restored = Session::from_json(&json).expect("serialized session should parse");
    assert_eq!(original, restored);
    assert!(json.contains("\"playback_mode\":\"shuffle_page\""));
    assert!(!json.contains("\"shuffle\""));
    assert!(!json.contains("repeat_track"));
}

/// Empty current input yields first-run defaults.
#[test]
fn empty_json_object_yields_defaults() {
    assert_eq!(
        Session::from_json("{}").expect("empty session object should parse"),
        Session::default(),
    );
}

/// Every former shuffle and repeat combination maps according to issue 460.
#[test]
fn every_legacy_combination_migrates() {
    let cases = [
        ("Off", false, PlaybackMode::InOrder),
        ("WithinPage", false, PlaybackMode::ShufflePage),
        ("All", false, PlaybackMode::ShuffleAll),
        ("Off", true, PlaybackMode::Repeat),
        ("WithinPage", true, PlaybackMode::Repeat),
        ("All", true, PlaybackMode::Repeat),
    ];
    for (shuffle, repeat_track, expected) in cases {
        let json = format!(
            "{{\"shuffle\":\"{shuffle}\",\"repeat_track\":{repeat_track}}}"
        );
        assert_eq!(
            Session::from_json(&json)
                .expect("legacy session should parse")
                .playback_mode,
            expected,
        );
    }
}

/// A present current field wins over stale former fields.
#[test]
fn current_playback_mode_wins_over_legacy_fields() {
    let json = r#"{"playback_mode":"shuffle_all","shuffle":"WithinPage","repeat_track":true}"#;
    assert_eq!(
        Session::from_json(json)
            .expect("current session should parse")
            .playback_mode,
        PlaybackMode::ShuffleAll,
    );
}

/// Unknown current mode degrades to In order without resurrecting stale values.
#[test]
fn unknown_current_mode_does_not_reuse_legacy_fields() {
    let json = r#"{"playback_mode":"future","shuffle":"All","repeat_track":true}"#;
    assert_eq!(
        Session::from_json(json)
            .expect("forward-version session should parse")
            .playback_mode,
        PlaybackMode::InOrder,
    );
}

/// Explicit null is a present current field and must not resurrect former state.
#[test]
fn null_current_mode_does_not_reuse_legacy_fields() {
    let json = r#"{"playback_mode":null,"shuffle":"All","repeat_track":true}"#;
    assert_eq!(
        Session::from_json(json)
            .expect("null-mode session should parse")
            .playback_mode,
        PlaybackMode::InOrder,
    );
}

/// Former track-list files retain unrelated settings while migrating playback.
#[test]
fn old_track_list_format_keeps_settings() {
    let old = r#"{"tracks":["/a.flac"],"current":0,"position_secs":5.0,"volume":0.5,"shuffle":"All","repeat_track":false}"#;
    let parsed = Session::from_json(old).expect("former track-list session should parse");
    assert_eq!(parsed.source_root, None);
    assert_eq!(parsed.selected, None);
    assert_eq!(parsed.position_secs, 5.0);
    assert_eq!(parsed.volume, 0.5);
    assert_eq!(parsed.playback_mode, PlaybackMode::ShuffleAll);
}


/// A session file written before the page style was removed still loads, and the key is not written again (VD7).
#[test]
fn the_removed_page_control_style_key_is_ignored_on_read_and_absent_from_new_writes() {
    let old = r#"{"source_root":"/music","volume":0.5,"playback_mode":"in_order","page_control_style":"Md1Tabs"}"#;
    let loaded = Session::from_json(old).expect("a file with the old key still parses");
    assert_eq!(loaded.source_root, Some(PathBuf::from("/music")));
    assert!((loaded.volume - 0.5).abs() < f32::EPSILON);
    let written = serde_json::to_string(&loaded).expect("the session serializes");
    assert!(!written.contains("page_control_style"), "new writes drop the key: {written}");
    let unknown_style = Session::from_json(r#"{"page_control_style":7,"volume":0.25}"#).expect("any value of the old key is ignored");
    assert!((unknown_style.volume - 0.25).abs() < f32::EPSILON);
}

/// A directory of its own for one test, so no test touches the user's session file. nextest runs each test in its own
/// process, and the name carries the test and the clock besides.
fn scratch_file(test: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = crate::test_scratch::scratch_root().join(format!("mp_session_{test}_{nanos}"));
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    return dir.join("session.json");
}

/// Auto-play on select is on for a first run and for an empty object (D133).
#[test]
fn auto_play_on_select_is_on_by_default() {
    assert!(Session::default().auto_play_on_select);
    assert!(Session::from_json("{}").expect("empty object parses").auto_play_on_select);
}

/// A file written before the setting existed reads on and keeps its other values.
#[test]
fn a_file_without_the_field_reads_on_and_still_loads() {
    let old = r#"{"source_root":"/music","volume":0.5,"playback_mode":"shuffle_all","track_rows_template":"$mi(title)$"}"#;
    let loaded = Session::from_json(old).expect("a file without the field parses");
    assert!(loaded.auto_play_on_select);
    assert_eq!(loaded.source_root, Some(PathBuf::from("/music")));
    assert_eq!(loaded.playback_mode, PlaybackMode::ShuffleAll);
    assert_eq!(loaded.track_rows_template.as_deref(), Some("$mi(title)$"));
}

/// The field is written and read back, in the current JSON and through a real file.
#[test]
fn auto_play_on_select_round_trips_through_json_and_a_file() {
    let on = Session { auto_play_on_select: true, ..Session::default() };
    let json = serde_json::to_string(&on).expect("the session serializes");
    assert!(json.contains("\"auto_play_on_select\":true"), "{json}");
    assert!(Session::from_json(&json).expect("it parses").auto_play_on_select);

    let path = scratch_file("round_trip");
    assert!(Session::load_at(&path).auto_play_on_select, "no file reads on");
    Session::save_auto_play_on_select_at(&path, false).expect("the setting is saved off");
    assert!(!Session::load_at(&path).auto_play_on_select, "the saved off reads off");
    Session::save_auto_play_on_select_at(&path, true).expect("the setting is saved again");
    assert!(Session::load_at(&path).auto_play_on_select, "the saved on reads on");
}

/// Saving the setting leaves the playback fields and the templates of an existing file as they were.
#[test]
fn saving_the_setting_keeps_the_rest_of_an_existing_file() {
    let path = scratch_file("keeps_rest");
    std::fs::write(
        &path,
        r#"{"source_root":"/music","selected":"/music/a.flac","position_secs":9.5,"volume":0.25,"playback_mode":"repeat","track_rows_template":"$mi(title)$","playing_template":"$mi(track)$"}"#,
    )
    .expect("the old file is written");
    Session::save_auto_play_on_select_at(&path, false).expect("the setting is saved");
    let after = Session::load_at(&path);
    assert!(!after.auto_play_on_select);
    assert_eq!(after.source_root, Some(PathBuf::from("/music")));
    assert_eq!(after.selected, Some(PathBuf::from("/music/a.flac")));
    assert!((after.position_secs - 9.5).abs() < f64::EPSILON);
    assert!((after.volume - 0.25).abs() < f32::EPSILON);
    assert_eq!(after.playback_mode, PlaybackMode::Repeat);
    assert_eq!(after.track_rows_template.as_deref(), Some("$mi(title)$"));
    assert_eq!(after.playing_template.as_deref(), Some("$mi(track)$"));
}

/// A legacy file that still needs the playback migration keeps working with the new field present or absent.
#[test]
fn the_migration_rewrite_keeps_the_setting() {
    let path = scratch_file("migration");
    std::fs::write(&path, r#"{"shuffle":"All","repeat_track":false,"auto_play_on_select":true}"#)
        .expect("the legacy file is written");
    let loaded = Session::load_at(&path);
    assert_eq!(loaded.playback_mode, PlaybackMode::ShuffleAll);
    assert!(loaded.auto_play_on_select);
    let rewritten = Session::load_at(&path);
    assert!(rewritten.auto_play_on_select, "the rewrite did not drop the setting");
    assert!(std::fs::read_to_string(&path).expect("the file exists").contains("\"playback_mode\""));
}

/// The entry points the window and the shutdown use read and write the session file of the config home (D129).
#[cfg(target_os = "linux")]
#[test]
fn the_real_entry_points_read_and_write_the_session_file_of_the_config_home() {
    let path = use_disposable_config_home("entry_points");
    assert!(Session::load_auto_play_on_select(), "no file means on");
    let before = Session { position_secs: 5.0, ..Session::default() };
    before.save().expect("the first save writes the file");
    assert!(path.exists(), "the file is in the config home");
    assert!(Session::load_auto_play_on_select(), "a file saved by the first run holds the default, on");
    Session::save_auto_play_on_select(false).expect("turning the setting off saves");
    assert!(!Session::load_auto_play_on_select(), "the saved setting loads back");
    assert!((Session::load().position_secs - 5.0).abs() < f64::EPSILON, "saving the setting keeps the other fields");
    Session::save_auto_play_on_select(true).expect("turning the setting on saves");
    assert!(Session::load_auto_play_on_select(), "and it loads back on");
}

/// What: `LogSink` is a writer that appends to a shared buffer, so a test can read what the log wrote.
/// Why: the load failures are only reported through the log, so the test needs the log's own text.
struct LogSink(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for LogSink {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("the log buffer lock is not poisoned").extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What: `logged_by` runs `action` with a log subscriber that records every level, and returns the log text with
/// `action`'s result.
/// Why: the subscriber is installed for this thread only, and `load_at` runs on the calling thread.
fn logged_by<T>(action: impl FnOnce() -> T) -> (T, String) {
    let buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer = std::sync::Arc::clone(&buffer);
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(move || return LogSink(std::sync::Arc::clone(&writer)))
        .finish();
    let result = tracing::subscriber::with_default(subscriber, action);
    let text = String::from_utf8(buffer.lock().expect("the log buffer lock is not poisoned").clone())
        .expect("the log is UTF-8");
    return (result, text);
}

/// A truncated file, the shape an interrupted in-place write leaves, loads as defaults and is reported with the parse
/// error and the path.
#[test]
fn a_truncated_session_file_loads_as_defaults_and_is_logged_with_its_path() {
    let path = scratch_file("truncated");
    std::fs::write(&path, r#"{"source_root":"/music","volume":0."#).expect("the truncated file is written");
    let (loaded, log) = logged_by(|| return Session::load_at(&path));
    assert_eq!(loaded, Session::default());
    assert!(log.contains("WARN"), "a warning is logged: {log}");
    assert!(log.contains("EOF while parsing"), "the parse error is in the log: {log}");
    assert!(log.contains("session.json"), "the path is in the log: {log}");
}

/// A session path that cannot be read as a file loads as defaults and is reported with the read error.
#[test]
fn an_unreadable_session_path_loads_as_defaults_and_is_logged_with_the_read_error() {
    let path = scratch_file("unreadable");
    std::fs::create_dir(&path).expect("a directory stands where the file would be");
    // Linux reports reading a directory as `IsADirectory`; Windows reports `PermissionDenied` ("Access is denied.").
    // The expected text is the platform's own error for the same read, so the check holds on both.
    let read_error = std::fs::read_to_string(&path).expect_err("a directory cannot be read as a file");
    let (loaded, log) = logged_by(|| return Session::load_at(&path));
    assert_eq!(loaded, Session::default());
    assert!(log.contains("WARN"), "a warning is logged: {log}");
    assert!(log.contains(&format!("{read_error:?}")), "the read error {read_error:?} is in the log: {log}");
}

/// A missing file is a first run, which is expected and stays quiet.
#[test]
fn a_missing_session_file_loads_as_defaults_without_a_warning() {
    let path = scratch_file("missing");
    let (loaded, log) = logged_by(|| return Session::load_at(&path));
    assert_eq!(loaded, Session::default());
    assert!(!log.contains("WARN"), "a first run logs no warning: {log}");
}

/// A save puts a complete new file in place of the old one, so an interrupted save leaves the old file whole. The
/// inode changes because the new bytes arrive in a different file that is renamed over the old one; an in-place
/// write keeps the inode.
#[cfg(target_os = "linux")]
#[test]
fn a_save_replaces_the_session_file_by_rename_and_leaves_no_temporary_file() {
    use std::os::unix::fs::MetadataExt;
    let path = scratch_file("atomic");
    Session::default().save_at(&path).expect("the first save writes the file");
    let before = std::fs::metadata(&path).expect("the file exists").ino();
    let second = Session { position_secs: 7.5, ..Session::default() };
    second.save_at(&path).expect("the second save replaces the file");
    let after = std::fs::metadata(&path).expect("the file exists").ino();
    assert_ne!(before, after, "the second save wrote a new file and renamed it over the old one");
    assert_eq!(Session::load_at(&path), second, "the replaced file is the complete second session");
    let names: Vec<_> = std::fs::read_dir(path.parent().expect("the file has a directory"))
        .expect("the directory lists")
        .map(|entry| return entry.expect("an entry reads").file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("session.json")], "no temporary file is left behind");
}

/// A temporary file left by an earlier interrupted save is replaced, not trusted, by the next save.
#[test]
fn a_stale_temporary_file_from_an_interrupted_save_does_not_block_the_next_save() {
    let path = scratch_file("stale_temporary");
    let directory = path.parent().expect("the file has a directory");
    std::fs::write(directory.join("session.json.tmp"), "{\"half\":").expect("the stale temporary file is written");
    let session = Session { volume: 0.25, ..Session::default() };
    session.save_at(&path).expect("the save replaces the stale temporary file");
    assert_eq!(Session::load_at(&path), session);
    assert!(!directory.join("session.json.tmp").exists(), "the temporary file was renamed away");
}

/// A save that cannot finish leaves the previous complete file in place.
#[cfg(target_os = "linux")]
#[test]
fn a_save_that_fails_leaves_the_previous_file_whole() {
    let path = scratch_file("failed_save");
    let first = Session { position_secs: 3.0, ..Session::default() };
    first.save_at(&path).expect("the first save writes the file");
    // A directory where the temporary file would go makes the write step fail before the rename.
    let directory = path.parent().expect("the file has a directory");
    std::fs::create_dir(directory.join("session.json.tmp")).expect("a directory blocks the temporary file");
    let error = Session { position_secs: 9.0, ..Session::default() }
        .save_at(&path)
        .expect_err("the save cannot create its temporary file");
    assert!(!error.to_string().is_empty());
    assert_eq!(Session::load_at(&path), first, "the previous session is still the complete file");
}

/// What: `sizes_of` gives the Text size values a test file holds, in the order of the Settings rows.
fn sizes_of(session: &Session) -> [u16; 4] {
    return TextElement::ALL.map(|element| return session.text_sizes.get(element));
}

/// A first run and an empty object draw every text at the size the app always used.
#[test]
fn text_sizes_are_the_old_sizes_by_default() {
    assert_eq!(sizes_of(&Session::default()), [20, 20, 14, 14]);
    assert_eq!(sizes_of(&Session::from_json("{}").expect("empty object parses")), [20, 20, 14, 14]);
}

/// The four sizes are written as four flat keys, not as a nested object, and read back.
#[test]
fn text_sizes_round_trip_through_json_under_four_flat_keys() {
    let session = Session {
        text_sizes: TextSizes::default()
            .with(TextElement::PaneTitle, 32)
            .with(TextElement::GroupHeading, 10)
            .with(TextElement::FolderName, 15)
            .with(TextElement::TrackTitle, 13),
        ..Session::default()
    };
    let json = serde_json::to_string(&session).expect("the session serializes");
    for expected in [
        "\"text_size_pane_title_px\":32",
        "\"text_size_group_heading_px\":10",
        "\"text_size_folder_name_px\":15",
        "\"text_size_track_title_px\":13",
    ] {
        assert!(json.contains(expected), "{expected} is in {json}");
    }
    assert!(!json.contains("text_sizes"), "no nested object: {json}");
    assert_eq!(Session::from_json(&json).expect("it parses"), session);
}

/// The sizes are saved through the one write path and load back from the file, and a missing file reads the defaults.
#[test]
fn text_sizes_round_trip_through_a_file() {
    let path = scratch_file("text_sizes_round_trip");
    assert!(Session::load_at(&path).text_sizes.is_default(), "no file reads the defaults");
    let sizes = TextSizes::default().with(TextElement::FolderName, 17).with(TextElement::TrackTitle, 22);
    Session::save_text_sizes_at(&path, sizes).expect("the sizes are saved");
    assert_eq!(Session::load_at(&path).text_sizes, sizes);
    Session::save_text_sizes_at(&path, TextSizes::default()).expect("the defaults are saved");
    assert!(Session::load_at(&path).text_sizes.is_default());
}

/// Saving the sizes leaves the playback fields, the templates and Auto-play on select of an existing file as they were.
#[test]
fn saving_the_text_sizes_keeps_the_rest_of_an_existing_file() {
    let path = scratch_file("text_sizes_keep_rest");
    std::fs::write(
        &path,
        r#"{"source_root":"/music","selected":"/music/a.flac","position_secs":9.5,"volume":0.25,"playback_mode":"repeat","track_rows_template":"$mi(title)$","playing_template":"$mi(track)$","auto_play_on_select":true}"#,
    )
    .expect("the old file is written");
    let sizes = TextSizes::default().with(TextElement::PaneTitle, 24);
    Session::save_text_sizes_at(&path, sizes).expect("the sizes are saved");
    let after = Session::load_at(&path);
    assert_eq!(after.text_sizes, sizes);
    assert_eq!(after.source_root, Some(PathBuf::from("/music")));
    assert_eq!(after.selected, Some(PathBuf::from("/music/a.flac")));
    assert!((after.position_secs - 9.5).abs() < f64::EPSILON);
    assert!((after.volume - 0.25).abs() < f32::EPSILON);
    assert_eq!(after.playback_mode, PlaybackMode::Repeat);
    assert_eq!(after.track_rows_template.as_deref(), Some("$mi(title)$"));
    assert_eq!(after.playing_template.as_deref(), Some("$mi(track)$"));
    assert!(after.auto_play_on_select);
}

/// Saving another setting keeps the sizes, so the template editor and the Auto-play switch never reset them.
#[test]
fn saving_another_setting_keeps_the_text_sizes() {
    let path = scratch_file("other_setting_keeps_sizes");
    let sizes = TextSizes::default().with(TextElement::GroupHeading, 28);
    Session::save_text_sizes_at(&path, sizes).expect("the sizes are saved");
    Session::save_auto_play_on_select_at(&path, true).expect("the other setting is saved");
    assert_eq!(Session::load_at(&path).text_sizes, sizes);
}

/// A file written before the setting existed still loads: every other field is kept and the sizes are the defaults.
#[test]
fn an_old_file_without_the_sizes_loads_and_keeps_its_fields() {
    let old = r#"{"source_root":"/music","volume":0.5,"playback_mode":"shuffle_all","track_rows_template":"$mi(title)$","auto_play_on_select":true}"#;
    let (loaded, log) = logged_by(|| return Session::from_json(old).expect("an old file parses"));
    assert!(loaded.text_sizes.is_default());
    assert_eq!(loaded.source_root, Some(PathBuf::from("/music")));
    assert!((loaded.volume - 0.5).abs() < f32::EPSILON);
    assert_eq!(loaded.playback_mode, PlaybackMode::ShuffleAll);
    assert_eq!(loaded.track_rows_template.as_deref(), Some("$mi(title)$"));
    assert!(loaded.auto_play_on_select);
    assert!(!log.contains("WARN"), "an old file is no corruption and logs no warning: {log}");
}

/// One field present and the other three missing: the present one is kept, the missing ones are their defaults, and
/// the unrelated fields are untouched.
#[test]
fn a_missing_size_field_never_resets_the_other_fields() {
    let text = r#"{"volume":0.75,"playback_mode":"shuffle_page","text_size_track_title_px":19,"auto_play_on_select":true}"#;
    let loaded = Session::from_json(text).expect("the file parses");
    assert_eq!(sizes_of(&loaded), [20, 20, 14, 19]);
    assert!((loaded.volume - 0.75).abs() < f32::EPSILON);
    assert_eq!(loaded.playback_mode, PlaybackMode::ShufflePage);
    assert!(loaded.auto_play_on_select);
}

/// A corrupt or out-of-range size falls back to the default of that element alone, is logged with its key, and costs
/// the rest of the file nothing; a good field beside it is kept.
#[test]
fn a_corrupt_size_falls_back_alone_and_is_logged() {
    let text = r#"{"volume":0.5,"playback_mode":"repeat","text_size_pane_title_px":"huge","text_size_group_heading_px":99,"text_size_folder_name_px":-3,"text_size_track_title_px":21}"#;
    let (loaded, log) = logged_by(|| return Session::from_json(text).expect("a corrupt size never fails the whole file"));
    assert_eq!(sizes_of(&loaded), [20, 20, 14, 21]);
    assert!((loaded.volume - 0.5).abs() < f32::EPSILON);
    assert_eq!(loaded.playback_mode, PlaybackMode::Repeat);
    for key in ["text_size_pane_title_px", "text_size_group_heading_px", "text_size_folder_name_px"] {
        assert!(log.contains(key), "{key} is named in the log: {log}");
    }
    assert!(!log.contains("text_size_track_title_px"), "the good field is not reported: {log}");
    assert!(log.contains("WARN"), "a warning is logged: {log}");
}

/// The edges of the range load as they are, and one pixel past each edge falls back to the default.
#[test]
fn the_limits_load_and_one_past_each_limit_falls_back() {
    let edges = r#"{"text_size_pane_title_px":10,"text_size_group_heading_px":32,"text_size_folder_name_px":9,"text_size_track_title_px":33}"#;
    assert_eq!(sizes_of(&Session::from_json(edges).expect("the file parses")), [10, 32, 14, 14]);
}

/// A corrupt size in a file on disk loads through the real read path without resetting the playback fields.
#[test]
fn a_corrupt_size_in_a_file_on_disk_keeps_the_playback_fields() {
    let path = scratch_file("corrupt_size_on_disk");
    std::fs::write(&path, r#"{"source_root":"/music","position_secs":4.5,"playback_mode":"repeat","text_size_folder_name_px":null}"#)
        .expect("the file is written");
    let loaded = Session::load_at(&path);
    assert_eq!(loaded.source_root, Some(PathBuf::from("/music")));
    assert!((loaded.position_secs - 4.5).abs() < f64::EPSILON);
    assert_eq!(loaded.playback_mode, PlaybackMode::Repeat);
    assert_eq!(sizes_of(&loaded), [20, 20, 14, 14]);
}

/// The entry points the window uses read and write the sizes of the config home, and keep the other fields.
#[cfg(target_os = "linux")]
#[test]
fn the_real_entry_points_read_and_write_the_sizes_of_the_config_home() {
    let path = use_disposable_config_home("text_size_entry_points");
    assert!(Session::load_text_sizes().is_default(), "no file means the defaults");
    Session { position_secs: 6.0, ..Session::default() }.save().expect("the first save writes the file");
    assert!(path.exists(), "the file is in the config home");
    let sizes = TextSizes::default().with(TextElement::TrackTitle, 30);
    Session::save_text_sizes(sizes).expect("saving the sizes writes the file");
    assert_eq!(Session::load_text_sizes(), sizes, "the saved sizes load back");
    assert!((Session::load().position_secs - 6.0).abs() < f64::EPSILON, "saving the sizes keeps the other fields");
}
