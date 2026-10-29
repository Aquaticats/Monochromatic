//! Controls for the configured-`core.hooksPath` warning: when it fires, what it says, and that it never blocks.

use super::*;

/// A transaction identifier in the registry's spelling.
const TRANSACTION: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// A snapshot parsed from `(scope, record)` pairs in the `git config --list -z --show-scope` shape.
fn listing_snapshot(records: &[(&str, &str)]) -> ConfigSnapshot {
    let mut bytes: Vec<u8> = Vec::new();
    for (scope, record) in records {
        bytes.extend_from_slice(scope.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(record.as_bytes());
        bytes.push(0);
    }
    return ConfigSnapshot::parse(bytes.as_slice(), true).expect("a well-formed listing");
}

/// A stream that refuses every write.
struct RefusingStream;

impl Write for RefusingStream {
    /// Refuses the bytes.
    fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
        return Err(std::io::Error::other("closed"));
    }

    /// Refuses the flush.
    fn flush(&mut self) -> std::io::Result<()> {
        return Err(std::io::Error::other("closed"));
    }
}

/// Without the setting there is no warning, whatever else the configuration holds, and no snapshot says nothing.
#[test]
fn an_unset_hooks_path_has_no_warning() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[
        ("local", "core.bare\nfalse"),
        ("local", "hook.mine.event\npre-commit"),
        ("local", "hook.mine.command\ntrue"),
    ]);
    assert_eq!(traditional_hooks_warning(Some(&snapshot)), None);
    assert_eq!(traditional_hooks_warning(Some(&listing_snapshot(&[]))), None);
    assert_eq!(traditional_hooks_warning(None), None);
}

/// A configured hooks path, relative or absolute, from any scope, produces one `configuration-warning` line.
#[test]
fn a_configured_hooks_path_warns_with_a_non_blocking_event() {
    for (scope, value) in [("local", "custom-hooks"), ("global", "/shared/hooks"), ("command", "~/hooks")] {
        let snapshot: ConfigSnapshot = listing_snapshot(&[(scope, format!("core.hookspath\n{value}").as_str())]);
        let line: String = traditional_hooks_warning(Some(&snapshot)).expect("a warning");
        assert!(line.ends_with("}\n"), "{line}");
        assert_eq!(line.matches('\n').count(), 1, "one line: {line}");
        assert!(line.starts_with("{\"schemaVersion\":1,\"sequence\":0,\"type\":\"configuration-warning\","), "{line}");
        assert!(line.contains("\"code\":\"traditional-hooks-unsupported\""), "{line}");
        assert!(line.contains(format!("\"path\":\"{value}\"").as_str()), "{line}");
        assert!(!line.contains("core-finding"), "a warning is not a blocking finding: {line}");
        assert!(!line.contains("\"severity\":\"error\""), "{line}");
    }
}

/// The warning names the finding, says traditional hooks are unsupported and undefined, and names the doctor command.
#[test]
fn the_warning_says_what_is_wrong_and_what_to_run() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\nmy-hooks")]);
    let line: String = traditional_hooks_warning(Some(&snapshot)).expect("a warning");
    assert!(line.contains("`core.hooksPath` is set to my-hooks"), "{line}");
    assert!(line.contains("does not support traditional hooks"), "{line}");
    assert!(line.contains("undefined"), "{line}");
    assert!(line.contains("`git cli-git doctor`"), "{line}");
    assert!(line.contains("\"hint\":\"Hint: "), "{line}");
}

/// The decision reads the snapshot only: a directory that does not exist still warns, so no file system probe is made.
#[test]
fn the_warning_never_looks_at_the_directory() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\n/no/such/directory/anywhere")]);
    assert!(!Path::new("/no/such/directory/anywhere").exists());
    assert!(traditional_hooks_warning(Some(&snapshot)).is_some());
}

/// The last entry of the key decides, as it does for Git, and an empty or valueless entry names no directory.
#[test]
fn the_last_entry_decides_and_empty_entries_name_no_directory() {
    let overridden: ConfigSnapshot =
        listing_snapshot(&[("global", "core.hookspath\nfirst"), ("local", "core.hookspath\nsecond")]);
    assert_eq!(configured_hooks_path(&overridden), Some(String::from("second")));
    let emptied: ConfigSnapshot = listing_snapshot(&[("global", "core.hookspath\nfirst"), ("local", "core.hookspath\n")]);
    assert_eq!(configured_hooks_path(&emptied), None);
    let valueless: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath")]);
    assert_eq!(configured_hooks_path(&valueless), None);
}

/// The wrapper's own per-transaction overrides are not the user's choice and never warn; lookalikes still do.
#[test]
fn the_wrappers_own_overrides_do_not_warn() {
    for own in [
        format!("/repo/.git/cli-git-transactions/{TRANSACTION}/no-hooks"),
        format!("/repo/.git/cli-git-transactions/{TRANSACTION}/hooks"),
    ] {
        assert!(is_wrapper_hooks_path(own.as_str()), "{own}");
        let snapshot: ConfigSnapshot = listing_snapshot(&[("command", format!("core.hookspath\n{own}").as_str())]);
        assert_eq!(traditional_hooks_warning(Some(&snapshot)), None, "{own}");
    }
    for other in [
        format!("/repo/.git/cli-git-transactions/{TRANSACTION}/elsewhere"),
        format!("/repo/.git/other-root/{TRANSACTION}/no-hooks"),
        String::from("/repo/.git/cli-git-transactions/not-a-transaction/hooks"),
        String::from("/repo/.git/hooks"),
        String::from("hooks"),
        String::new(),
    ] {
        assert!(!is_wrapper_hooks_path(other.as_str()), "{other}");
    }
    let lookalike: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\n/repo/.git/hooks")]);
    assert!(traditional_hooks_warning(Some(&lookalike)).is_some());
}

/// The writer puts exactly one line on the stream when the setting exists and nothing when it does not.
#[test]
fn the_writer_emits_one_line_or_nothing() {
    let configured: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\nmy-hooks")]);
    let mut warned: Vec<u8> = Vec::new();
    warn_traditional_hooks(Some(&configured), &mut warned);
    assert_eq!(String::from_utf8(warned).expect("UTF-8").matches('\n').count(), 1);
    let mut silent: Vec<u8> = Vec::new();
    warn_traditional_hooks(Some(&listing_snapshot(&[])), &mut silent);
    warn_traditional_hooks(None, &mut silent);
    assert!(silent.is_empty());
}

/// A stream that refuses the warning does not stop the command.
#[test]
fn a_refusing_stream_does_not_panic() {
    let configured: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\nmy-hooks")]);
    warn_traditional_hooks(Some(&configured), &mut RefusingStream);
}

/// A value with quotes, newlines and backslashes stays inside one JSON line.
#[test]
fn a_hostile_value_stays_inside_one_line() {
    let snapshot: ConfigSnapshot =
        listing_snapshot(&[("local", "core.hookspath\n/x\"}\n{\"type\":\"core-finding\",\"y\":\"\\")]);
    let line: String = traditional_hooks_warning(Some(&snapshot)).expect("a warning");
    assert_eq!(line.matches('\n').count(), 1, "{line}");
    assert!(!line.contains("{\"type\":\"core-finding\""), "{line}");
}
