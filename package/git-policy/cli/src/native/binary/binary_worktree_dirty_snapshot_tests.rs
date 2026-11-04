//! What:
//!  A main checkout with uncommitted changes is still snapshotted, and the new worktree equals what a clean `git worktree add`
//!  makes, through the built executable.
//! Why:
//!  The owner works in the main checkout, which is dirty most of the time; refusing the snapshot then costs minutes. The new
//!  worktree must hold the committed bytes, no staged change, no untracked file that Git shows and all of the ignored state,
//!  and the main checkout must not change by one byte. Every control compares with a worktree real Git made from the same
//!  commit, and the sign that a snapshot served the command is the ignored state only a snapshot carries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(trackedContent('../topic')).toEqual(trackedContent('../control'));
//! // expect(snapshotOf('repo')).toEqual(snapshotBefore); // the main checkout is untouched
//! ```

/// Import the shared fixtures, and the helpers of the snapshot controls.
use super::support::{Fixture, Observed, fixture, git, observe, remove, run_wrapped, stderr_of, wrapped};
use super::worktree_snapshot::{add_pair, carries_ignored_state, commit_all, on_btrfs_or_acknowledged, subvolume_repository, tracked, tracked_content};
use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The words that mark the line saying an add was not made as a snapshot.
const SLOW_MARK: &str = "not made as a snapshot";

/// What the person is told after a snapshot of a dirty main checkout: the changes were left out.
const LEFT_OUT_MARK: &str = "uncommitted changes";

/// A main checkout that is a subvolume, with a third commit that adds the files the changes below need, then every kind of
///  change: an unstaged edit, a file staged and then edited again, a staged deletion, an unstaged deletion, a staged rename,
///  a mode change, untracked files, an untracked directory with an ignored file in it, an empty untracked directory, and
///  installed dependencies in an ignored `node_modules`.
fn dirty_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = subvolume_repository(fixture);
    tracked(repo.as_path(), ".gitignore", b"ignored/\nnode_modules/\n");
    tracked(repo.as_path(), "extra/rename_me.txt", b"rename\n");
    tracked(repo.as_path(), "extra/gone.txt", b"gone\n");
    tracked(repo.as_path(), "extra/script.sh", b"#!/bin/sh\n");
    commit_all(fixture, repo.as_path(), "third");
    std::fs::create_dir_all(repo.join("node_modules/dep")).expect("dependencies");
    std::fs::write(repo.join("node_modules/dep/index.js"), b"module.exports = 2;\n").expect("dependency");
    // An unstaged edit.
    std::fs::write(repo.join("shared.txt"), b"unstaged edit\n").expect("edit");
    // Staged, then edited again.
    std::fs::write(repo.join("src/deep/leaf.txt"), b"staged edit\n").expect("edit");
    git(fixture, repo.as_path(), &["add", "src/deep/leaf.txt"]);
    std::fs::write(repo.join("src/deep/leaf.txt"), b"staged edit, then edited again\n").expect("edit");
    // A staged deletion, an unstaged deletion, a staged rename, a mode change.
    git(fixture, repo.as_path(), &["rm", "--quiet", "new/added.txt"]);
    std::fs::remove_file(repo.join("extra/gone.txt")).expect("delete");
    git(fixture, repo.as_path(), &["mv", "extra/rename_me.txt", "extra/renamed.txt"]);
    std::fs::set_permissions(repo.join("extra/script.sh"), std::fs::Permissions::from_mode(0o755)).expect("chmod");
    // Untracked: a file, a directory, a directory that also holds an ignored file, and an empty directory.
    std::fs::write(repo.join("notes.txt"), b"untracked\n").expect("untracked");
    std::fs::create_dir_all(repo.join("scratch/ignored")).expect("untracked directory");
    std::fs::write(repo.join("scratch/draft.txt"), b"draft\n").expect("untracked");
    std::fs::write(repo.join("scratch/ignored/keep.txt"), b"ignored\n").expect("ignored inside untracked");
    std::fs::create_dir_all(repo.join("emptydir")).expect("empty directory");
    return repo;
}

/// Every file below `root` outside `.git` with its mode and bytes, and the index of `.git`, as they are now.
fn state_of(root: &Path) -> BTreeMap<String, (u32, Vec<u8>)> {
    fn walk(base: &Path, directory: &Path, found: &mut BTreeMap<String, (u32, Vec<u8>)>) {
        for entry in std::fs::read_dir(directory).expect("listing").flatten() {
            let path: PathBuf = entry.path();
            let relative: String = path.strip_prefix(base).expect("below the base").to_string_lossy().into_owned();
            if relative == ".git" {
                continue;
            }
            let metadata = std::fs::symlink_metadata(path.as_path()).expect("metadata");
            if metadata.is_dir() {
                found.insert(format!("{relative}/"), (metadata.permissions().mode(), Vec::new()));
                walk(base, path.as_path(), found);
            } else if metadata.is_file() {
                found.insert(relative, (metadata.permissions().mode(), std::fs::read(path.as_path()).expect("bytes")));
            } else {
                found.insert(relative, (0, Vec::new()));
            }
        }
    }
    let mut found: BTreeMap<String, (u32, Vec<u8>)> = BTreeMap::new();
    walk(root, root, &mut found);
    found.insert(String::from(".git/index"), (0, std::fs::read(root.join(".git/index")).expect("index")));
    return found;
}

/// The new worktree equals a worktree real Git made from the same commit, and everything else about it is as a snapshot leaves it.
fn assert_equals_gits_checkout(fixture: &Fixture, created: &Observed, topic: &Path, control: &Path, repo: &Path) {
    assert_eq!(created.code, Some(0), "{}", stderr_of(created));
    assert_eq!(tracked_content(fixture, topic), tracked_content(fixture, control), "tracked files equal Git's checkout");
    assert_eq!(git(fixture, topic, &["status", "--porcelain=v1", "--untracked-files=all"]).stdout, b"", "clean on the first status");
    assert_eq!(git(fixture, topic, &["ls-files", "--others", "--exclude-standard", "--directory", "--no-empty-directory"]).stdout, b"", "no untracked file Git shows");
    assert_eq!(git(fixture, topic, &["diff-index", "--cached", "--name-only", "HEAD"]).stdout, b"", "the index equals HEAD");
    assert_eq!(git(fixture, topic, &["rev-parse", "HEAD"]).stdout, git(fixture, control, &["rev-parse", "HEAD"]).stdout);
    assert_eq!(git(fixture, topic, &["write-tree"]).stdout, git(fixture, control, &["write-tree"]).stdout, "the same tree in the index");
    assert!(carries_ignored_state(topic), "a snapshot served the command: {}", stderr_of(created));
    assert_eq!(std::fs::read(topic.join("node_modules/dep/index.js")).expect("dependencies carried"), b"module.exports = 2;\n");
    assert_eq!(std::fs::read(topic.join("scratch/ignored/keep.txt")).expect("ignored file inside an untracked directory"), b"ignored\n");
    assert!(!topic.join("scratch/draft.txt").exists() && !topic.join("notes.txt").exists(), "untracked files are not carried");
    assert!(!topic.join("emptydir").exists(), "an empty untracked directory is not carried");
    assert!(std::fs::read(topic.join(".git")).expect("pointer").starts_with(b"gitdir: "));
    assert!(!repo.join(".git/index.lock").exists());
}

/// A dirty main checkout is snapshotted: the new worktree equals Git's own checkout, the person is told the changes were left
///  out, and the main checkout is byte for byte what it was.
#[test]
fn a_dirty_main_checkout_is_snapshotted_and_restored_to_gits_own_result() {
    let fixture: Fixture = fixture("dirty-snapshot");
    if !on_btrfs_or_acknowledged(&fixture, "dirty-snapshot") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = dirty_repository(&fixture);
    let before = state_of(repo.as_path());
    let (created, _control) = add_pair(&fixture, repo.as_path(), "topic", "control", &["-b", "NAME", "../NAME"]);
    assert_equals_gits_checkout(&fixture, &created, fixture.root.join("topic").as_path(), fixture.root.join("control").as_path(), repo.as_path());
    let text: String = stderr_of(&created);
    assert!(text.contains(git_policy_cli::worktree_trash::MAIN_SNAPSHOT_NOTE), "{text}");
    assert!(text.lines().any(|line: &str| { return line.starts_with("cli-git: ") && line.contains(LEFT_OUT_MARK) && line.contains("left out"); }), "the person is told: {text}");
    assert!(!text.contains(SLOW_MARK), "no slow-route line: {text}");
    assert_eq!(state_of(repo.as_path()), before, "the main checkout is untouched");
    remove(&fixture);
}

/// The same, for a target commit older than the source's `HEAD`.
#[test]
fn a_dirty_main_checkout_with_an_older_target_commit_equals_gits_checkout() {
    let fixture: Fixture = fixture("dirty-older");
    if !on_btrfs_or_acknowledged(&fixture, "dirty-older") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = dirty_repository(&fixture);
    let before = state_of(repo.as_path());
    let (created, _control) = add_pair(&fixture, repo.as_path(), "old", "oldcontrol", &["--detach", "../NAME", "HEAD~2"]);
    let old: PathBuf = fixture.root.join("old");
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(tracked_content(&fixture, old.as_path()), tracked_content(&fixture, fixture.root.join("oldcontrol").as_path()));
    // The older commit does not ignore `node_modules`, so only the carried dependencies show, and nothing else.
    let status: String = String::from_utf8_lossy(&git(&fixture, old.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]).stdout).into_owned();
    assert!(status.lines().all(|line: &str| { return line.starts_with("?? node_modules/"); }), "{status}");
    assert!(carries_ignored_state(old.as_path()), "a snapshot served the command: {}", stderr_of(&created));
    assert_eq!(state_of(repo.as_path()), before, "the main checkout is untouched");
    remove(&fixture);
}

/// `--quiet` silences the note about the left-out changes as it silences the snapshot note.
#[test]
fn quiet_silences_the_note_about_left_out_changes() {
    let fixture: Fixture = fixture("dirty-quiet");
    if !on_btrfs_or_acknowledged(&fixture, "dirty-quiet") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = dirty_repository(&fixture);
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert!(carries_ignored_state(fixture.root.join("topic").as_path()), "a snapshot served the command");
    assert!(created.stderr.is_empty(), "--quiet stays silent: {}", stderr_of(&created));
    remove(&fixture);
}

/// Every refusal that is not an ordinary change still holds: an unfinished merge, a running Git command, a skip-worktree entry
///  and an assume-unchanged entry each send the add the slow way, with the sentence that names the cause.
#[test]
fn the_refusals_that_are_no_ordinary_change_still_hold() {
    let fixture: Fixture = fixture("dirty-refusals");
    if !on_btrfs_or_acknowledged(&fixture, "dirty-refusals") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = dirty_repository(&fixture);
    let head: String = String::from_utf8_lossy(&git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout).trim().to_string();
    let journal: String = format!("@cli-git-test-journal-nobody-refusals-{}", std::process::id());
    let attempt = |name: &str| -> Observed {
        return observe(
            wrapped(&fixture).env(git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, journal.as_str()).current_dir(repo.as_path()).args(["worktree", "add", "-b", name, format!("../{name}").as_str()]),
            b"",
        );
    };
    let assert_slow = |observed: &Observed, name: &str, cause: &str| {
        assert_eq!(observed.code, Some(0), "{}", stderr_of(observed));
        assert!(stderr_of(observed).contains(SLOW_MARK), "{name}: {}", stderr_of(observed));
        assert!(stderr_of(observed).contains(cause), "{name}: {}", stderr_of(observed));
        assert!(!stderr_of(observed).contains(git_policy_cli::worktree_trash::MAIN_SNAPSHOT_NOTE), "{name}: no snapshot served it");
    };
    std::fs::write(repo.join(".git/MERGE_HEAD"), format!("{head}\n")).expect("merge mark");
    assert_slow(&attempt("merging"), "merge", "MERGE_HEAD");
    std::fs::remove_file(repo.join(".git/MERGE_HEAD")).expect("end the merge");
    std::fs::write(repo.join(".git/index.lock"), b"").expect("lock");
    assert_slow(&attempt("locked"), "lock", "index.lock");
    std::fs::remove_file(repo.join(".git/index.lock")).expect("release");
    git(&fixture, repo.as_path(), &["update-index", "--skip-worktree", "extra/script.sh"]);
    assert_slow(&attempt("skipped"), "skip-worktree", "skip-worktree");
    git(&fixture, repo.as_path(), &["update-index", "--no-skip-worktree", "extra/script.sh"]);
    git(&fixture, repo.as_path(), &["update-index", "--assume-unchanged", "extra/script.sh"]);
    assert_slow(&attempt("assumed"), "assume-unchanged", "assume-unchanged");
    git(&fixture, repo.as_path(), &["update-index", "--no-assume-unchanged", "extra/script.sh"]);
    let after: Observed = attempt("after");
    assert_eq!(after.code, Some(0), "{}", stderr_of(&after));
    assert!(stderr_of(&after).contains(git_policy_cli::worktree_trash::MAIN_SNAPSHOT_NOTE), "positive control: with the marks gone a snapshot serves it: {}", stderr_of(&after));
    remove(&fixture);
}

/// A changed `.gitattributes` file could change how the restore writes other files, so that source is left to Git's checkout.
#[test]
fn a_changed_attributes_file_sends_the_add_the_slow_way() {
    let fixture: Fixture = fixture("dirty-attributes");
    if !on_btrfs_or_acknowledged(&fixture, "dirty-attributes") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = dirty_repository(&fixture);
    std::fs::write(repo.join(".gitattributes"), b"*.txt text eol=crlf\n").expect("an untracked attributes file");
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "-b", "topic", "../topic"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert!(stderr_of(&created).contains(SLOW_MARK) && stderr_of(&created).contains(".gitattributes"), "{}", stderr_of(&created));
    assert!(!stderr_of(&created).contains(git_policy_cli::worktree_trash::MAIN_SNAPSHOT_NOTE), "{}", stderr_of(&created));
    remove(&fixture);
}

/// Move a path's modification time `seconds` into the past, so a stat comparison sees a time that Git's index never recorded.
fn set_mtime_seconds_ago(path: &Path, seconds: u64) {
    let file: std::fs::File = std::fs::File::options().write(true).open(path).expect("open to set the time");
    file.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(seconds)).expect("set the modification time");
}

/// Whether any stderr line says changes were left out of the new worktree; with none to leave out, the line would be false.
fn says_changes_were_left_out(text: &str) -> bool {
    return text.lines().any(|line: &str| { return line.starts_with("cli-git: ") && line.contains(LEFT_OUT_MARK); });
}

/// What the person sees when the main checkout holds no change at all: a snapshot, equal to Git's checkout, with nothing said about
///  changes, and the main checkout byte for byte what it was.
fn assert_snapshotted_without_changes(fixture: &Fixture, repo: &Path) {
    let before = state_of(repo);
    let (created, _control) = add_pair(fixture, repo, "topic", "control", &["-b", "NAME", "../NAME"]);
    let text: String = stderr_of(&created);
    assert!(!text.contains(SLOW_MARK), "no slow-route line: {text}");
    assert!(!text.contains("could not list"), "{text}");
    let (topic, control): (PathBuf, PathBuf) = (fixture.root.join("topic"), fixture.root.join("control"));
    assert_eq!(created.code, Some(0), "{text}");
    assert_eq!(tracked_content(fixture, topic.as_path()), tracked_content(fixture, control.as_path()), "tracked files equal Git's checkout");
    assert_eq!(git(fixture, topic.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]).stdout, b"", "clean on the first status");
    assert_eq!(git(fixture, topic.as_path(), &["write-tree"]).stdout, git(fixture, control.as_path(), &["write-tree"]).stdout, "the same tree in the index");
    assert!(carries_ignored_state(topic.as_path()), "a snapshot served the command: {text}");
    assert_eq!(std::fs::read(topic.join("ignored/pkg/index.js")).expect("ignored state carried"), b"module.exports = 1;\n");
    assert!(text.contains(git_policy_cli::worktree_trash::MAIN_SNAPSHOT_NOTE), "{text}");
    assert!(!says_changes_were_left_out(text.as_str()), "there were no changes to leave out: {text}");
    assert_eq!(state_of(repo), before, "the main checkout is untouched");
}

/// A main checkout whose index is as new as its files (a merge or a pull just wrote both) has no change, and Git says so by content;
///  the stat data cannot vouch for the files, and that alone must not cost the snapshot.
#[test]
fn a_main_checkout_whose_files_are_as_new_as_its_index_is_snapshotted() {
    let fixture: Fixture = fixture("dirty-racy-index");
    if !on_btrfs_or_acknowledged(&fixture, "dirty-racy-index") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    // The index is older than every file it records, which is what an index written in the same second as its files looks like.
    set_mtime_seconds_ago(repo.join(".git/index").as_path(), 3_600);
    assert_eq!(git(&fixture, repo.as_path(), &["--no-optional-locks", "status", "--porcelain=v1"]).stdout, b"", "Git lists no change");
    assert_snapshotted_without_changes(&fixture, repo.as_path());
    remove(&fixture);
}

/// A main checkout whose tracked file has another modification time than its index entry records, with the same bytes (a file
///  touched, or written again with the same content), has no change either.
#[test]
fn a_main_checkout_with_a_touched_file_of_the_same_bytes_is_snapshotted() {
    let fixture: Fixture = fixture("dirty-stale-index");
    if !on_btrfs_or_acknowledged(&fixture, "dirty-stale-index") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    set_mtime_seconds_ago(repo.join("shared.txt").as_path(), 60);
    assert_eq!(git(&fixture, repo.as_path(), &["--no-optional-locks", "status", "--porcelain=v1"]).stdout, b"", "Git lists no change");
    assert_snapshotted_without_changes(&fixture, repo.as_path());
    remove(&fixture);
}

/// The journal receives one ordinary-level entry that a snapshot left the changes out, with the destination and the number of
///  paths, also under `--quiet`.
#[test]
fn the_journal_records_that_changes_were_left_out_even_when_quiet() {
    use std::os::linux::net::SocketAddrExt;
    let fixture: Fixture = fixture("dirty-journal");
    if !on_btrfs_or_acknowledged(&fixture, "dirty-journal") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = dirty_repository(&fixture);
    let name: String = format!("cli-git-test-journal-dirty-{}", std::process::id());
    let address = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("an abstract address");
    let receiver = std::os::unix::net::UnixDatagram::bind_addr(&address).expect("bind the journal stand-in");
    receiver.set_read_timeout(Some(std::time::Duration::from_secs(5))).expect("a read timeout");
    let created: Observed = observe(
        wrapped(&fixture).env(git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@{name}")).current_dir(repo.as_path()).args(["worktree", "add", "--quiet", "-b", "topic", "../topic"]),
        b"",
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let mut buffer: Vec<u8> = vec![0; 65_536];
    // The wrapper's `clean` of the stage also makes an entry (the words of a run that carries no `--quiet`, `inner_git_words.rs`),
    //  so the entries are read until the one about the left-out changes arrives.
    let entry: BTreeMap<String, String> = loop {
        let length: usize = receiver.recv(buffer.as_mut_slice()).expect("the entry about the left-out changes reached the journal stand-in");
        let read: BTreeMap<String, String> = String::from_utf8_lossy(&buffer[..length])
            .lines()
            .filter_map(|line: &str| { return line.split_once('=').map(|(field, value): (&str, &str)| { return (field.to_string(), value.to_string()); }); })
            .collect();
        if read.get("MESSAGE").is_some_and(|message: &String| { return message.contains(LEFT_OUT_MARK); }) {
            break read;
        }
    };
    let listed: usize = git(&fixture, repo.as_path(), &["status", "--porcelain=v1", "--no-renames"]).stdout.split(|byte: &u8| { return *byte == b'\n'; }).filter(|line: &&[u8]| { return !line.is_empty(); }).count();
    assert_eq!(entry.get("PRIORITY").map(String::as_str), Some("6"), "the ordinary level: {entry:?}");
    assert_eq!(entry.get("CLI_GIT_DIRTY_PATHS"), Some(&listed.to_string()), "{entry:?}");
    assert_eq!(entry.get("CLI_GIT_WORKTREE").map(String::as_str), Some(fixture.root.join("topic").to_string_lossy().as_ref()), "{entry:?}");
    assert!(entry.get("MESSAGE").is_some_and(|message: &String| { return message.contains(LEFT_OUT_MARK); }), "{entry:?}");
    remove(&fixture);
}

/// What:
///  What a snapshot add of a dirty main checkout prints on standard output.
/// Why:
///   The restore of the stage runs `git clean` without `--quiet` (the owner's rule), and Git then says "Removing <path>" for each
///  untracked file of the stage, a stage the person's command never names and native `git worktree add` has no such lines for.
///  Those words are kept in the journal, so the person sees only what native Git's add prints: the line of the checkout, and
///  nothing under `--quiet`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await add(['-b', 'topic', '../topic']).stdout).toMatch(/^HEAD is now at \w+ third\n$/); expect(await add(['--quiet', ...]).stdout).toBe('');
/// ```
#[test]
fn a_snapshot_add_does_not_print_what_its_clean_removes() {
    let fixture: Fixture = fixture("dirty-clean-words");
    if !on_btrfs_or_acknowledged(&fixture, "dirty-clean-words") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = dirty_repository(&fixture);
    let plain: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "-b", "topic", "../topic"]);
    assert_eq!(plain.code, Some(0), "{}", stderr_of(&plain));
    let shown: String = String::from_utf8_lossy(&plain.stdout).into_owned();
    assert!(!shown.contains("Removing"), "the clean of the stage is not the person's business: {shown}");
    assert!(shown.starts_with("HEAD is now at "), "{shown}");
    let quiet: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic2", "../topic2"]);
    assert_eq!(quiet.code, Some(0), "{}", stderr_of(&quiet));
    assert!(quiet.stdout.is_empty(), "--quiet stays silent: {}", String::from_utf8_lossy(&quiet.stdout));
    remove(&fixture);
}
