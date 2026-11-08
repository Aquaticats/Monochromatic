//! What:
//!  The line that tells the person why `git worktree add` took the slow way,
//! through the built executable:
//! a source that
//!  cannot be snapshotted gets a plain line on standard error naming the cause and what makes the next add quick,
//! `--quiet`
//!  silences it,
//! a clean source and a failed add print none,
//! and the journal receives an entry either way.
//! Why:
//!  Two adds from the main checkout once took minutes through Git's checkout and the ignored-state copy and said nothing,
//! so
//!  nobody could tell the cause afterwards.
//! The sign of the slow way is the ignored-state copy summary (a snapshot prints a
//!  different note),
//! and the line under test is found by the words that say the worktree was not made as a snapshot.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(stderr).toContain('uncommitted changes in 1 path (for example shared.txt)');
//! ```

/// Import the shared fixtures,
/// and the helpers of the snapshot controls.
use super::support::{Fixture, Observed, fixture, git, observe, remove, run_wrapped, stderr_of, wrapped};
use super::support::on_btrfs_or_acknowledged;
use super::worktree_snapshot::subvolume_repository;
use std::path::{Path, PathBuf};

/// The words that mark the line under test:
/// the worktree was not made as a snapshot.
const SLOW_MARK: &str = "not made as a snapshot";

/// The lines of standard error that say the add took the slow way.
fn slow_lines(observed: &Observed) -> Vec<String> {
    return stderr_of(observed).lines().filter(|line: &&str| { return line.contains(SLOW_MARK); }).map(str::to_string).collect();
}

/// Run `git worktree add` through the wrapper from `repo`,
/// with extra words before the destination,
/// and the journal pointed at
///  nobody so no run reaches the host's journal.
fn add(fixture: &Fixture, repo: &Path, name: &str, extra: &[&str]) -> Observed {
    let mut arguments: Vec<String> = vec![String::from("worktree"), String::from("add")];
    arguments.extend(extra.iter().map(|word: &&str| { return (*word).to_string(); }));
    arguments.extend([String::from("-b"), name.to_string(), format!("../{name}")]);
    return observe(
        wrapped(fixture)
            .env(git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@cli-git-test-journal-nobody-{}", std::process::id()))
            .current_dir(repo)
            .args(arguments.iter().map(String::as_str)),
        b"",
    );
}

/// A linked worktree that a snapshot made from the main checkout (so it is a subvolume) with one file edited:
/// a source whose
///  changes are not restored by a snapshot,
/// because only the main checkout is.
fn dirty_linked_worktree(fixture: &Fixture, repo: &Path) -> PathBuf {
    let made: Observed = run_wrapped(fixture, repo, &["worktree", "add", "--quiet", "-b", "linked", "../linked"]);
    assert_eq!(made.code, Some(0), "{}", stderr_of(&made));
    let linked: PathBuf = fixture.root.join("linked");
    std::fs::write(linked.join("shared.txt"), b"dirty edit\n").expect("modify");
    return linked;
}

/// A source with an uncommitted change says so:
/// one line,
/// the cause with the count and a path,
/// and what makes the next add quick.
#[test]
fn a_dirty_source_says_why_the_add_was_not_a_snapshot() {
    let fixture: Fixture = fixture("slow-route-dirty");
    if !on_btrfs_or_acknowledged(&fixture, "slow-route-dirty") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    let linked: PathBuf = dirty_linked_worktree(&fixture, repo.as_path());
    let created: Observed = add(&fixture, linked.as_path(), "topic", &[]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let lines: Vec<String> = slow_lines(&created);
    assert_eq!(lines.len(), 1, "one plain line: {}", stderr_of(&created));
    let line: &str = lines[0].as_str();
    assert!(line.starts_with("monochromatic-git-policy-cli: "), "{line}");
    assert!(line.contains("this worktree has uncommitted changes in 1 path (for example shared.txt)"), "the cause: {line}");
    assert!(line.contains("Commit or stash"), "what makes the next add quick: {line}");
    assert!(!line.contains('[') && !line.to_lowercase().contains("rung"), "plain words, no log prefix, no internal names: {line}");
    remove(&fixture);
}

/// `--quiet` silences the line,
/// and the add still succeeds the slow way.
#[test]
fn quiet_silences_the_slow_route_line() {
    let fixture: Fixture = fixture("slow-route-quiet");
    if !on_btrfs_or_acknowledged(&fixture, "slow-route-quiet") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    let linked: PathBuf = dirty_linked_worktree(&fixture, repo.as_path());
    let created: Observed = add(&fixture, linked.as_path(), "topic", &["--quiet"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(slow_lines(&created), Vec::<String>::new(), "{}", stderr_of(&created));
    assert!(!stderr_of(&created).contains("shared.txt"), "{}", stderr_of(&created));
    assert_eq!(std::fs::read(fixture.root.join("topic/shared.txt")).expect("checked out"), b"second, longer\n", "the add completed");
    remove(&fixture);
}

/// A Git command running in the source (its `index.lock`) is named as the cause,
/// with the wait as the remedy.
#[test]
fn a_running_git_command_in_the_source_is_named() {
    let fixture: Fixture = fixture("slow-route-lock");
    if !on_btrfs_or_acknowledged(&fixture, "slow-route-lock") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    std::fs::write(repo.join(".git/index.lock"), b"").expect("a lock");
    let created: Observed = add(&fixture, repo.as_path(), "topic", &[]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let lines: Vec<String> = slow_lines(&created);
    assert_eq!(lines.len(), 1, "{}", stderr_of(&created));
    assert!(lines[0].contains("another Git command is running"), "{}", lines[0]);
    assert!(lines[0].contains("index.lock"), "{}", lines[0]);
    remove(&fixture);
}

/// A clean source is snapshotted and says nothing about a slow route;
/// an add Git refuses says nothing either.
#[test]
fn a_snapshot_and_a_failed_add_print_no_slow_route_line() {
    let fixture: Fixture = fixture("slow-route-silent");
    if !on_btrfs_or_acknowledged(&fixture, "slow-route-silent") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    let snapshotted: Observed = add(&fixture, repo.as_path(), "topic", &[]);
    assert_eq!(snapshotted.code, Some(0), "{}", stderr_of(&snapshotted));
    assert_eq!(slow_lines(&snapshotted), Vec::<String>::new(), "{}", stderr_of(&snapshotted));
    assert!(stderr_of(&snapshotted).contains(git_policy_cli::worktree_trash::MAIN_SNAPSHOT_NOTE), "positive control: a snapshot served it: {}", stderr_of(&snapshotted));
    std::fs::write(repo.join("shared.txt"), b"dirty edit\n").expect("modify");
    git(&fixture, repo.as_path(), &["branch", "taken"]);
    let failed: Observed = add(&fixture, repo.as_path(), "taken", &[]);
    assert_ne!(failed.code, Some(0), "{}", stderr_of(&failed));
    assert_eq!(slow_lines(&failed), Vec::<String>::new(), "a failed add is not told as a slow one: {}", stderr_of(&failed));
    remove(&fixture);
}

/// The journal receives one warning entry with the same words,
/// the worktree and the cause as fields,
/// also under `--quiet`.
#[test]
fn the_journal_receives_the_slow_route_entry_even_when_quiet() {
    use std::os::linux::net::SocketAddrExt;
    let fixture: Fixture = fixture("slow-route-journal");
    if !on_btrfs_or_acknowledged(&fixture, "slow-route-journal") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    let linked: PathBuf = dirty_linked_worktree(&fixture, repo.as_path());
    let name: String = format!("cli-git-test-journal-slow-route-{}", std::process::id());
    let address = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("an abstract address");
    let receiver = std::os::unix::net::UnixDatagram::bind_addr(&address).expect("bind the journal stand-in");
    receiver.set_read_timeout(Some(std::time::Duration::from_secs(5))).expect("a read timeout");
    let created: Observed = observe(
        wrapped(&fixture)
            .env(git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@{name}"))
            .current_dir(linked.as_path())
            .args(["worktree", "add", "--quiet", "-b", "topic", "../topic"]),
        b"",
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let mut buffer: Vec<u8> = vec![0; 65_536];
    let length: usize = receiver.recv(buffer.as_mut_slice()).expect("an entry reached the journal stand-in");
    let entry: std::collections::BTreeMap<String, String> = String::from_utf8_lossy(&buffer[..length])
        .lines()
        .filter_map(|line: &str| { return line.split_once('=').map(|(field, value): (&str, &str)| { return (field.to_string(), value.to_string()); }); })
        .collect();
    assert_eq!(entry.get("PRIORITY").map(String::as_str), Some("4"), "a warning: {entry:?}");
    assert_eq!(entry.get("SYSLOG_IDENTIFIER").map(String::as_str), Some("monochromatic-git-policy-cli"), "{entry:?}");
    assert!(entry.get("MESSAGE").is_some_and(|message: &String| { return message.contains(SLOW_MARK) && message.contains("shared.txt"); }), "{entry:?}");
    assert_eq!(entry.get("CLI_GIT_WORKTREE").map(String::as_str), Some(fixture.root.join("topic").to_string_lossy().as_ref()), "{entry:?}");
    assert_eq!(entry.get("CLI_GIT_SLOW_CAUSE").map(String::as_str), Some("uncommitted-changes"), "{entry:?}");
    remove(&fixture);
}
