//! Controls for the settings the in-process index update follows.

use super::*;
use crate::repo_read_config_text::ConfigEntry;

/// One listed entry from a global file.
fn entry(key: &str, value: Option<&str>) -> ListedEntry {
    return ListedEntry { scope: "global", entry: ConfigEntry { key: key.to_string(), value: value.map(|text| { return text.as_bytes().to_vec(); }) } };
}

/// No configuration:
/// the monitor is off and the untracked cache is kept.
#[test]
fn an_empty_configuration_keeps_the_extensions_and_has_no_monitor() {
    assert_eq!(read_settings(&[]), Some(IndexSettings { monitor: false, untracked: UntrackedMode::Keep, shared_repository: 0 }));
}

/// `core.fsmonitor` as a true value,
/// a bare name and a hook path turns the monitor on;
/// false and empty leave it off;
/// the last
/// entry wins.
#[test]
fn core_fsmonitor_is_read_as_git_reads_it() {
    for (value, expected) in [(Some("true"), true), (None, true), (Some("/hooks/watchman"), true), (Some("false"), false), (Some(""), false), (Some("no"), false)] {
        let settings: IndexSettings = read_settings(&[entry("core.fsmonitor", value)]).expect("read");
        assert_eq!(settings.monitor, expected, "{value:?}");
    }
    let last: IndexSettings = read_settings(&[entry("core.fsmonitor", Some("true")), entry("core.fsmonitor", Some("false"))]).expect("read");
    assert!(!last.monitor);
}

/// `core.untrackedCache`:
/// true adds,
/// false removes,
/// `keep` and no value keep,
/// and anything else is left to Git.
#[test]
fn core_untrackedcache_is_read_as_git_reads_it() {
    for (value, expected) in [("true", Some(UntrackedMode::Write)), ("false", Some(UntrackedMode::Remove)), ("keep", Some(UntrackedMode::Keep)), ("maybe", None)] {
        let found: Option<UntrackedMode> = read_settings(&[entry("core.untrackedcache", Some(value))]).map(|settings| { return settings.untracked; });
        assert_eq!(found, expected, "{value}");
    }
}

/// Keys that change the file layout or the checks decline;
/// boolean ones decline only when true.
#[test]
fn keys_the_update_does_not_reproduce_decline() {
    for key in ["core.splitindex", "index.sparse", "index.skiphash", "index.threads", "index.recordoffsettable", "index.recordendofindexentries", "core.hookspath", "core.worktree"] {
        assert_eq!(read_settings(&[entry(key, Some("1"))]), None, "{key}");
    }
    for key in ["core.sparsecheckout", "core.ignorecase", "feature.manyfiles"] {
        assert_eq!(read_settings(&[entry(key, Some("true"))]), None, "{key} true");
        assert!(read_settings(&[entry(key, Some("false"))]).is_some(), "{key} false");
    }
}

/// `core.sharedRepository` is read as `git_config_perm` reads it,
/// and the values Git dies on or cannot be classified decline.
#[test]
fn core_sharedrepository_is_read_as_git_reads_it() {
    for (value, expected) in [
        (None, Some(0o660)),
        (Some("group"), Some(0o660)),
        (Some("all"), Some(0o664)),
        (Some("world"), Some(0o664)),
        (Some("everybody"), Some(0o664)),
        (Some("umask"), Some(0)),
        (Some("false"), Some(0)),
        (Some("true"), Some(0o660)),
        (Some("0"), Some(0)),
        (Some("1"), Some(0o660)),
        (Some("2"), Some(0o664)),
        (Some("0640"), Some(-0o640)),
        (Some("0666"), Some(-0o666)),
        (Some("0477"), None),
        (Some("sometimes"), None),
    ] {
        assert_eq!(shared_permission(value.map(str::as_bytes)), expected, "{value:?}");
    }
    let settings: IndexSettings = read_settings(&[entry("core.sharedrepository", Some("group"))]).expect("read");
    assert_eq!(settings.shared_repository, 0o660);
}
