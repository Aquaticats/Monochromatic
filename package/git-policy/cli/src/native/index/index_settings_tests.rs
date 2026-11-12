//! Controls for the settings the in-process index update follows.

use super::*;
use crate::repo_read_config_text::ConfigEntry;

/// One listed entry from a global file.
fn entry(key: &str, value: Option<&str>) -> ListedEntry {
    return ListedEntry { scope: "global", entry: ConfigEntry { key: key.to_string(), value: value.map(|text| { return text.as_bytes().to_vec(); }) } };
}

/// No configuration:
/// the monitor is off,
/// the untracked cache is kept and the trailing hash is written.
#[test]
fn an_empty_configuration_keeps_the_extensions_and_has_no_monitor() {
    assert_eq!(read_settings(&[]), Some(IndexSettings { monitor: false, untracked: UntrackedMode::Keep, shared_repository: 0, skip_hash: false }));
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
    for key in ["core.splitindex", "index.sparse", "index.threads", "index.recordoffsettable", "index.recordendofindexentries", "core.hookspath", "core.worktree"] {
        assert_eq!(read_settings(&[entry(key, Some("1"))]), None, "{key}");
    }
    for key in ["core.sparsecheckout", "core.ignorecase"] {
        assert_eq!(read_settings(&[entry(key, Some("true"))]), None, "{key} true");
        assert!(read_settings(&[entry(key, Some("false"))]).is_some(), "{key} false");
    }
}

/// What:
///  `feature.manyFiles` is the three defaults Git makes of it (`git-config` of Git 2.56.0:
///  a version 4 index file,
///  the untracked cache and a skipped trailing hash),
///  and `core.untrackedCache` or `index.skipHash` beside it stands
///  only where Git's own reader replaces the default with the key's value.
/// Why:
///   A repository that adopts Git's setting for a large checkout must keep the in-process index update,
///  which declines every configuration it cannot reproduce,
///  and it must not write an index Git would have written differently.
///  `repo-settings.c` of Git 2.56.0 sets the untracked cache to add from `feature.manyFiles` first and reads
///  `core.untrackedCache` afterwards,
///  where `keep` is a value `git_parse_maybe_bool` refuses,
///  so `keep` leaves the added default in place while `true` and `false` replace it;
///  `index.skipHash` is read with the `feature.manyFiles` value as its default,
///  so an explicit boolean there always wins.
#[test]
fn feature_manyfiles_sets_git_s_defaults_and_a_boolean_beside_it_wins() {
    let defaults: IndexSettings = read_settings(&[entry("feature.manyfiles", Some("true"))]).expect("feature.manyFiles is a configuration the update reproduces");
    assert_eq!((defaults.monitor, defaults.untracked, defaults.skip_hash), (false, UntrackedMode::Write, true));
    let overridden: IndexSettings = read_settings(&[
        entry("feature.manyfiles", Some("true")),
        entry("core.untrackedcache", Some("keep")),
        entry("index.skiphash", Some("false")),
    ])
    .expect("read");
    assert_eq!((overridden.untracked, overridden.skip_hash), (UntrackedMode::Write, false));
    let off: IndexSettings = read_settings(&[entry("feature.manyfiles", Some("false"))]).expect("read");
    assert_eq!((off.untracked, off.skip_hash), (UntrackedMode::Keep, false));
}

/// What:
///  `index.skipHash` on its own decides the trailing hash,
///  a bare key is true,
///  and a value Git dies on declines.
/// Why:
///   Git writes zeros where the checksum would be when this is on,
///  so the in-process write must know which of the two files it is writing.
#[test]
fn index_skiphash_is_read_as_git_reads_it() {
    for (value, expected) in [(Some("true"), Some(true)), (Some("false"), Some(false)), (Some("1"), Some(true)), (None, Some(true)), (Some("maybe"), None)] {
        let found: Option<bool> = read_settings(&[entry("index.skiphash", value)]).map(|settings| {
            return settings.skip_hash;
        });
        assert_eq!(found, expected, "{value:?}");
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
