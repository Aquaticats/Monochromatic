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

/// What:
///  A hook whose every event the author switched off is not reported,
///  and one that still has an event switched on is.
/// Why:
///   `hook.<event>.enabled=false` is Git's own way of switching every hook of an event off (`git hook list` calls it
/// `event-disabled`),
/// and the wrapper passes the same switch on every start,
/// so warning about such a hook names keys that ask for
///  nothing and makes a `doctor` a script gates on fail for a repository that has already disabled what it is told about.
#[test]
fn a_hook_whose_every_event_the_author_switched_off_is_not_reported() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.control.event\npre-commit"),
        ("local", "hook.control.command\nrun it"),
        ("local", "hook.pre-commit.enabled\nfalse"),
        ("local", "hook.half.event\npre-commit"),
        ("local", "hook.half.event\npost-commit"),
        ("local", "hook.half.command\nrun it"),
        ("global", "hook.pre-commit.enabled\nfalse"),
        ("global", "hook.on.event\npost-checkout"),
        ("global", "hook.on.command\nrun it"),
    ]);
    let hooks: Vec<ConfiguredHook> = configured_hooks(&snapshot);
    let names: Vec<&str> = hooks.iter().map(|hook| { return hook.name.as_str(); }).collect();
    assert_eq!(names, vec!["half", "on"], "the hook of the disabled event is gone, the others stay");
    let line: String = external_hooks_warning(Some(&snapshot)).expect("a warning about the hooks that remain");
    assert!(!line.contains("hook.control"), "{line}");
    assert!(line.contains("hook.half") && line.contains("hook.on"), "{line}");
}

/// What:
///  A switch the wrapper supplied does not hide a hook the person configured.
/// Why:
///   Every real-Git start of the wrapper carries `-c hook.<event>.enabled=false`,
/// and a Git child inherits the same values
///  through `GIT_CONFIG_PARAMETERS`,
/// so a wrapped command inside one reads them in the `command` scope.
/// Those are the wrapper's
///  suppression and not the person's choice,
/// so a hook dropped because of them is a configured hook that doctor never names and a
///  commit never warns about.
#[test]
fn a_switch_the_wrapper_supplied_does_not_hide_the_persons_hook() {
    let suppressed: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.control.event\npre-commit"),
        ("local", "hook.control.command\nrun the linter"),
        ("command", "hook.pre-commit.enabled\nfalse"),
        ("command", "core.hooksPath\n/dev/null/cli-git-no-hooks"),
    ]);
    let names: Vec<String> = configured_hooks(&suppressed).into_iter().map(|hook| { return hook.name; }).collect();
    assert_eq!(names, vec![String::from("control")], "the person's hook is still reported");
    assert!(external_hooks_warning(Some(&suppressed)).is_some(), "and the commit still warns about it");
    // The person's own switch, written in a file, still hides the hook: that one is their choice.
    let chosen: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.control.event\npre-commit"),
        ("local", "hook.pre-commit.enabled\nfalse"),
    ]);
    assert!(configured_hooks(&chosen).is_empty(), "a switch in the person's configuration is theirs");
}

/// What:
///  A hook's own switch is read the way Git reads it:
/// the command scope comes last and wins.
/// Why:
///   The wrapper supplies no `hook.<name>.enabled` of its own,
/// so a value in the command scope is the caller's own choice,
///  written with `-c` or inherited from a command line:
/// reading only the files dropped a hook the caller had just enabled,
///  and kept one the caller had just switched off,
///  in doctor and in the commit warning alike.
#[test]
fn a_switch_the_caller_wrote_wins_over_a_file_as_it_does_in_git() {
    let enabled_by_the_caller: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.control.event\npre-commit"),
        ("local", "hook.control.command\nrun the linter"),
        ("local", "hook.control.enabled\nfalse"),
        ("command", "hook.control.enabled\ntrue"),
    ]);
    let names: Vec<String> = configured_hooks(&enabled_by_the_caller).into_iter().map(|hook| { return hook.name; }).collect();
    assert_eq!(names, vec![String::from("control")], "the caller's own switch enabled the hook");
    assert!(external_hooks_warning(Some(&enabled_by_the_caller)).is_some(), "and the commit warns about it again");
    // The other direction: a caller who switches the hook off is not told about it.
    let disabled_by_the_caller: ConfigSnapshot = listing_snapshot(&[
        ("local", "hook.control.event\npre-commit"),
        ("local", "hook.control.command\nrun the linter"),
        ("command", "hook.control.enabled\nfalse"),
    ]);
    assert!(configured_hooks(&disabled_by_the_caller).is_empty(), "the caller switched the hook off for this command");
}
