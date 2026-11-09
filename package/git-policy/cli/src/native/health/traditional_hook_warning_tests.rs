//! Controls for the configured-`core.hooksPath` warning:
//! when it fires,
//! what it says,
//! and that it never blocks.

use super::*;
use crate::test_support::SharedSink;

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

/// Without the setting there is no warning,
/// whatever else the configuration holds,
/// and no snapshot says nothing.
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

/// A configured hooks path,
/// relative or absolute,
/// from any scope,
/// produces one `configuration-warning` line.
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

/// The warning names the finding,
/// says traditional hooks are unsupported and undefined,
/// and names the doctor command.
#[test]
fn the_warning_says_what_is_wrong_and_what_to_run() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\nmy-hooks")]);
    let line: String = traditional_hooks_warning(Some(&snapshot)).expect("a warning");
    assert!(line.contains("`core.hooksPath` is set to my-hooks"), "{line}");
    assert!(line.contains("supports no external hook program"), "{line}");
    assert!(line.contains("policy under policies"), "{line}");
    assert!(line.contains("`git monochromatic-git-policy-cli doctor`"), "{line}");
    assert!(line.contains("\"hint\":\"Hint: "), "{line}");
}

/// The decision reads the snapshot only:
/// a directory that does not exist still warns,
/// so no file system probe is made.
#[test]
fn the_warning_never_looks_at_the_directory() {
    let snapshot: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\n/no/such/directory/anywhere")]);
    assert!(!Path::new("/no/such/directory/anywhere").exists());
    assert!(traditional_hooks_warning(Some(&snapshot)).is_some());
}

/// The last entry of the key decides,
/// as it does for Git,
/// and an empty or valueless entry names no directory.
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

/// The wrapper's own hooks-off values are not the person's choice and never warn;
/// lookalikes still do.
#[test]
fn the_wrappers_own_overrides_do_not_warn() {
    for own in [
        String::from(NO_HOOKS_DIRECTORY),
        format!("/repo/.git/monochromatic-git-policy-cli/{SHADOW_ROOT_NAME}/{TRANSACTION}/{SHADOW_NO_HOOKS_NAME}"),
        format!("/repo/.git/cli-git/{SHADOW_ROOT_NAME}/{TRANSACTION}/{SHADOW_NO_HOOKS_NAME}"),
    ] {
        assert!(is_wrapper_hooks_path("command", own.as_str()), "{own}");
        let snapshot: ConfigSnapshot = listing_snapshot(&[("command", format!("core.hookspath\n{own}").as_str())]);
        assert_eq!(traditional_hooks_warning(Some(&snapshot)), None, "{own}");
    }
    for other in [
        format!("/project/{SHADOW_ROOT_NAME}/{TRANSACTION}/{SHADOW_NO_HOOKS_NAME}"),
        format!("/repo/.git/monochromatic-git-policy-cli/{SHADOW_ROOT_NAME}/{TRANSACTION}/elsewhere"),
        format!("/repo/.git/monochromatic-git-policy-cli/{SHADOW_ROOT_NAME}/{TRANSACTION}/hooks"),
        format!("/repo/.git/other-root/{TRANSACTION}/{SHADOW_NO_HOOKS_NAME}"),
        format!("/repo/.git/monochromatic-git-policy-cli/{SHADOW_ROOT_NAME}/not-a-transaction/{SHADOW_NO_HOOKS_NAME}"),
        format!("{NO_HOOKS_DIRECTORY}/more"),
        String::from("/repo/.git/hooks"),
        String::from("hooks"),
        String::new(),
    ] {
        assert!(!is_wrapper_hooks_path("command", other.as_str()), "{other}");
    }
    let lookalike: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\n/repo/.git/hooks")]);
    assert!(traditional_hooks_warning(Some(&lookalike)).is_some());
    // Even the wrapper's own spellings are the person's choice in a configuration file: the wrapper writes none.
    assert!(!is_wrapper_hooks_path("local", NO_HOOKS_DIRECTORY), "{NO_HOOKS_DIRECTORY}");
    let written: String = format!("/repo/.git/monochromatic-git-policy-cli/{SHADOW_ROOT_NAME}/{TRANSACTION}/{SHADOW_NO_HOOKS_NAME}");
    assert!(!is_wrapper_hooks_path("local", written.as_str()), "{written}");
    let file_scope: ConfigSnapshot = listing_snapshot(&[("local", format!("core.hookspath\n{written}").as_str())]);
    assert!(traditional_hooks_warning(Some(&file_scope)).is_some(), "{written}");
}

/// The writer puts exactly one line on the stream when the setting exists and nothing when it does not.
#[test]
fn the_writer_emits_one_line_or_nothing() {
    let configured: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\nmy-hooks")]);
    let warned: SharedSink = SharedSink::default();
    assert!(warn_traditional_hooks(Some(&configured), warned.clone()), "the line reached the stream");
    assert_eq!(warned.text().matches('\n').count(), 1);
    let silent: SharedSink = SharedSink::default();
    assert!(warn_traditional_hooks(Some(&listing_snapshot(&[])), silent.clone()));
    assert!(warn_traditional_hooks(None, silent.clone()));
    assert_eq!(silent.text(), "");
}

/// A stream that refuses the warning does not stop the command.
#[test]
fn a_refusing_stream_does_not_panic() {
    let configured: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\nmy-hooks")]);
    warn_traditional_hooks(Some(&configured), RefusingStream);
}

/// A value with quotes,
/// newlines and backslashes stays inside one JSON line.
#[test]
fn a_hostile_value_stays_inside_one_line() {
    let snapshot: ConfigSnapshot =
        listing_snapshot(&[("local", "core.hookspath\n/x\"}\n{\"type\":\"core-finding\",\"y\":\"\\")]);
    let line: String = traditional_hooks_warning(Some(&snapshot)).expect("a warning");
    assert_eq!(line.matches('\n').count(), 1, "{line}");
    assert!(!line.contains("{\"type\":\"core-finding\""), "{line}");
}

///  A commit run inside a Git the wrapper started warns about the person's hooks path,
///  not about the wrapper's own.
/// Why:
///   The wrapper's switches travel into a child Git as command-scope configuration,
/// which Git places last,
///  so the last entry of the key is the wrapper's and the person's path must be found before it;
///  otherwise the
///  warning says nothing about a repository whose hooks are configured.
#[test]
fn a_nested_run_warns_about_the_persons_hooks_path_and_not_the_wrappers() {
    let own: String = format!("core.hookspath\n{NO_HOOKS_DIRECTORY}");
    let nested: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\n/my/repository/hooks"), ("command", own.as_str())]);
    assert_eq!(configured_hooks_path(&nested), Some(String::from("/my/repository/hooks")));
    let line: String = traditional_hooks_warning(Some(&nested)).expect("a warning about the person's path");
    assert!(line.contains("/my/repository/hooks"), "{line}");
    assert!(!line.contains(NO_HOOKS_DIRECTORY), "{line}");
}

///  A stream whose write waits for an answer that never comes,
/// as a full pipe does.
/// Why:
///   A write to a descriptor whose reader has stopped draining waits in the kernel with no end;
/// this sink waits the same way,
///  in the process,
/// so the bound can be proved without a full pipe.
struct BlockedStream {
    /// Kept alive so the wait below has no end.
    _sender: std::sync::mpsc::Sender<()>,
    /// Waited on,
    /// never answered.
    waiting: std::sync::mpsc::Receiver<()>,
}

impl BlockedStream {
    /// A stream that blocks its first write until this value is dropped.
    fn new() -> Self {
        let (sender, waiting) = std::sync::mpsc::channel::<()>();
        return Self { _sender: sender, waiting };
    }
}

impl Write for BlockedStream {
    /// Waits for an answer that never comes.
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let _never = self.waiting.recv();
        return Ok(buffer.len());
    }

    /// Nothing is buffered.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A warning whose stream blocks returns anyway,
/// inside its ceiling.
/// Why:
///  The warning is written on the commit's preparation path,
/// and the owner's rule is "nothing should ever wait forever or indefinitely":
///  a full pipe must not hold a commit,
/// and its line is optional beside the command's own result.
#[test]
fn a_blocked_warning_write_returns_within_its_ceiling() {
    let configured: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\nmy-hooks")]);
    let (done, finished) = std::sync::mpsc::channel::<()>();
    let writer = std::thread::spawn(move || {
        let stream = BlockedStream::new();
        warn_traditional_hooks(Some(&configured), stream);
        let _sent = done.send(());
    });
    let answered: Result<(), _> = finished.recv_timeout(std::time::Duration::from_secs(30));
    // The writer is left waiting on purpose: a blocked write has no end, and the test process ends it.
    let _abandoned = writer;
    assert!(answered.is_ok(), "the warning write never returned, so a stream that stops draining holds a commit forever");
}

/// What:
///  A `core.hooksPath` the wrapper does not own warns,
/// even when it ends in `shadow/<transaction>/no-hooks`.
/// Why:
///  The wrapper's own override is written by its shadow commands into the data folder it keeps inside the
///  repository's Git directory;
/// a path a person chose that merely ends the same way is the person's choice,
/// and the disclosure
///  that traditional hooks never run must not be suppressed by a suffix.
#[test]
fn a_user_owned_shadow_shaped_path_still_warns() {
    let unrelated: String = format!("/project/{SHADOW_ROOT_NAME}/{TRANSACTION}/{SHADOW_NO_HOOKS_NAME}");
    assert!(!is_wrapper_hooks_path("command", unrelated.as_str()), "{unrelated}");
    let snapshot: ConfigSnapshot = listing_snapshot(&[("command", format!("core.hookspath\n{unrelated}").as_str())]);
    assert!(
        traditional_hooks_warning(Some(&snapshot)).is_some(),
        "a directory the wrapper does not own is the person's choice and must warn"
    );
}

/// What:
///  The warning reaches standard error while another thread holds standard error's shared lock,
/// and the shared handle is untouched by it.
/// Why:
///  The write runs on a thread of its own and is abandoned at its ceiling;
/// a worker still waiting inside a write
///  through the shared handle would keep that lock,
/// and the commit's next synchronous write would wait for it with no end.
/// This
///  control holds the lock the commit's own write takes and requires the warning to arrive anyway.
#[cfg(unix)]
#[test]
fn a_warning_reaches_standard_error_while_its_shared_lock_is_held() {
    let configured: ConfigSnapshot = listing_snapshot(&[("local", "core.hookspath\nmy-hooks")]);
    let held = std::io::stderr().lock();
    let delivered: bool = warn_traditional_hooks(Some(&configured), crate::standard_error_descriptor::StandardErrorDescriptor);
    drop(held);
    assert!(delivered, "the warning needed the lock a synchronous writer holds, so a worker left at the ceiling would keep it");
}
