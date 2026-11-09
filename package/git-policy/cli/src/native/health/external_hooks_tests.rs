//! Controls for the configured-hook report:
//! which hooks the snapshot names,
//! what the warning says and that it never
//!  blocks.

use super::*;
use crate::test_support::SharedSink;

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

/// A `core.fsmonitor` is left as configured:
/// it is no hook of the report,
/// whatever it names.
#[test]
fn core_fsmonitor_is_not_reported_as_a_hook() {
    for value in ["/usr/local/bin/watch", "true", "false"] {
        let snapshot: ConfigSnapshot = listing_snapshot(&[("local", format!("core.fsmonitor\n{value}").as_str())]);
        assert!(configured_hooks(&snapshot).is_empty(), "{value}");
        assert_eq!(external_hooks_warning(Some(&snapshot)), None, "{value}");
    }
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

/// Hooks group by name,
/// events accumulate in order,
/// an empty event resets the list as Git does,
/// and a command alone
/// still names a hook.
#[test]
fn hooks_group_by_name_with_their_events_and_commands() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.review.event\npost-commit"),
        ("local", "hook.review.command\nrun it"),
        ("local", "hook.lint.event\npre-commit"),
        ("local", "hook.lint.event\ncommit-msg"),
        ("local", "hook.reset.event\npre-push"),
        ("local", "hook.reset.event\n"),
        ("local", "hook.reset.event\npost-merge"),
        ("local", "hook.orphan.command\ntrue"),
        ("local", "hook.a.b.event\nupdate"),
    ]);
    let hooks: Vec<ConfiguredHook> = configured_hooks(&snapshot);
    let names: Vec<&str> = hooks.iter().map(|hook| { return hook.name.as_str(); }).collect();
    assert_eq!(names, vec!["review", "lint", "reset", "orphan", "a.b"]);
    assert_eq!(hooks[0].events, vec![String::from("post-commit")]);
    assert!(hooks[0].has_command);
    assert_eq!(hooks[1].events, vec![String::from("pre-commit"), String::from("commit-msg")]);
    assert!(!hooks[1].has_command);
    assert_eq!(hooks[2].events, vec![String::from("post-merge")]);
    assert!(hooks[3].events.is_empty() && hooks[3].has_command);
    assert_eq!(describe_hook(&hooks[3]), "hook.orphan (no event)");
    assert_eq!(describe_hook(&hooks[1]), "hook.lint (events: pre-commit, commit-msg)");
}

/// Keys that only look like hooks name none:
/// event switches,
/// other hook settings and other sections.
#[test]
fn switches_and_other_keys_name_no_hook() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.pre-commit.enabled\nfalse"),
        ("local", "hook.jobs\n4"),
        ("local", "hook.mine.parallel\ntrue"),
        ("local", "core.hookspath\n/x"),
        ("local", "hooks.concurrentcommits\ntrue"),
    ]);
    assert_eq!(configured_hooks(&snapshot), Vec::new());
    assert_eq!(external_hooks_warning(Some(&snapshot)), None);
    assert_eq!(external_hooks_warning(None), None);
}

/// A hook its author switched off with `hook.<name>.enabled=false` is not reported,
/// and one switched on is.
#[test]
fn a_hook_switched_off_by_name_is_not_reported() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.off.event\npost-commit"),
        ("local", "hook.off.enabled\nfalse"),
        ("local", "hook.on.event\npost-commit"),
        ("local", "hook.on.enabled\ntrue"),
    ]);
    let hooks: Vec<ConfiguredHook> = configured_hooks(&snapshot);
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].name, "on");
}

/// The warning names every hook with its events,
/// the ways forward and the doctor,
/// as one `configuration-warning`.
#[test]
fn the_warning_names_the_hooks_and_the_ways_forward() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.review.event\npost-commit"),
        ("local", "hook.review.command\nrun it"),
        ("global", "hook.lint.event\npre-commit"),
    ]);
    let line: String = external_hooks_warning(Some(&snapshot)).expect("warning");
    assert!(line.ends_with('\n') && line.matches('\n').count() == 1, "one line: {line:?}");
    assert!(line.contains("\"type\":\"configuration-warning\"") && line.contains(EXTERNAL_HOOKS_UNSUPPORTED_CODE), "{line}");
    for expected in ["hook.review (events: post-commit)", "hook.lint (events: pre-commit)", "--unset-all", "runs no program named by any configuration", "policies", "git monochromatic-git-policy-cli doctor"] {
        assert!(line.contains(expected), "{expected} in {line}");
    }
}

/// A stream that refuses the line does not stop the command,
/// and a good stream receives exactly the line.
#[test]
fn the_writer_emits_one_line_or_nothing() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[("local", "hook.a.event\npost-commit")]);
    let written: SharedSink = SharedSink::default();
    warn_external_hooks(Some(&snapshot), written.clone());
    assert_eq!(written.text(), external_hooks_warning(Some(&snapshot)).expect("warning"));
    let nothing: SharedSink = SharedSink::default();
    warn_external_hooks(None, nothing.clone());
    assert_eq!(nothing.text(), "");
    warn_external_hooks(Some(&snapshot), RefusingStream);
}

/// What:
///  A hook disabled with an integer false is already off,
/// so the report does not name it and the warning does not fire.
/// Why:
///  Git reads `hook.<name>.enabled` with its own boolean reader,
/// which parses `00` as zero;
/// the report kept
///  every value it had not modelled,
/// and told the person to remove a hook that never runs.
#[test]
fn a_hook_disabled_with_a_numeric_false_is_not_reported() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.audit.event\npre-commit"),
        ("local", "hook.audit.command\necho EXTERNAL-RAN"),
        ("local", "hook.audit.enabled\n00"),
    ]);
    assert_eq!(configured_hooks(&snapshot), Vec::<ConfiguredHook>::new());
    assert_eq!(external_hooks_warning(Some(&snapshot)), None);
}
