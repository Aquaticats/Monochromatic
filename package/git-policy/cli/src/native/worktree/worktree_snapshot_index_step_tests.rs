//! What:
//!  Controls for the index step of the snapshot rung:
//! the index a source with changes installs in the new
//!  worktree keeps the stat fields its own index recorded,
//! so Git hashes a tracked file instead of trusting fields
//!  taken from the file itself.
//! Why:
//!  The restore of such a snapshot settles by content the files the stat data cannot vouch for.
//! Fields
//!  refreshed from the copy of a file an edit changed would vouch for the edited bytes,
//! and the reset
//!  after it would leave them where Git reports the file unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! installIndex({ source, stage, admin, ownIndex: seed, environment, proven: undefined });
//! expect(matchesStored(statOf(stage, 'a.txt'), installedEntry('a.txt'))).toBe(false);
//! ```

use super::*;
use crate::config_snapshot::read_config_snapshot;
use crate::fixture_content_decision::content_environment;
use crate::subvolume_index_files::{fields_of, matches_stored};
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_clean::CleanSource;
use crate::worktree_ladder_test_support::hide_an_edit;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// The bytes the fixture commits and the same-length bytes an edit puts in their place.
const COMMITTED: &[u8] = b"first\n";
const EDITED: &[u8] = b"other\n";

/// What:
///  A main checkout whose one tracked file an edit Git cannot see replaced,
/// and the index that recorded it before
///  that edit.
/// Why:
///   This is the state a snapshot of a checkout with changes carries:
/// its own index,
/// one instant with the
///  files it recorded,
/// and files an edit changed afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(root, 'source'); ...; const seed = readFileSync(join(repo, '.git/index'));
/// ```
fn source_with_a_hidden_edit(root: &std::path::Path) -> (PathBuf, Vec<u8>) {
    let repo: PathBuf = repository(root, "source");
    std::fs::write(repo.join("a.txt"), COMMITTED).expect("the tracked file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    assert!(hide_an_edit(repo.as_path(), "a.txt", COMMITTED, EDITED), "the fixture hides its edit from Git's stat check");
    let seed: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("the index the snapshot carries");
    return (repo, seed);
}

/// The source the index step is told about:
/// what `HEAD` names and the hash length of the repository.
fn source_of(root: &std::path::Path, repo: &std::path::Path) -> SnapshotSource {
    let head: String = String::from_utf8_lossy(&git(repo, &["rev-parse", "HEAD"]).stdout).trim().to_string();
    let elsewhere: PathBuf = root.join("elsewhere");
    return SnapshotSource {
        root: elsewhere.clone(),
        git_dir: elsewhere.join(".git"),
        common_dir: elsewhere.join(".git"),
        clean: CleanSource { head_commit: head, head_tree: Vec::new(), tracked: 1, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: Vec::new() },
        nested: Vec::new(),
        config: read_config_snapshot(&git_context(), repo).expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
}

/// What:
///  The installed index never records the edited file's own stat fields,
/// so Git hashes that file instead of taking the
///  edit for the entry's content.
/// Why:
///   The fields of a file an edit changed would vouch for bytes the entry does not name;
/// keeping the recorded
///  fields is what makes the restore settle the file by content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(matchesStored(fieldsOf(statOf(stage, 'a.txt')), installedEntry('a.txt').stored)).toBe(false);
/// ```
#[test]
fn the_index_a_source_with_changes_installs_keeps_the_fields_that_prove_an_edit() {
    let root: PathBuf = fixture("index-step-unseen-edit");
    let (repo, seed) = source_with_a_hidden_edit(root.as_path());
    let source: SnapshotSource = source_of(root.as_path(), repo.as_path());
    let admin: PathBuf = root.join("admin");
    std::fs::create_dir(admin.as_path()).expect("administrative directory");
    let held: File = open_root(admin.as_path()).expect("the administrative directory opens");
    let state: CopyState = install_index(&source, repo.as_path(), &held, Some(seed.as_slice()), &content_environment(), None).expect("the index installs");
    assert_eq!(state, CopyState::Agrees, "a source with changes installs its own index as it is");
    let installed: Vec<u8> = std::fs::read(admin.join("index")).expect("the installed index");
    let entries: Vec<StatEntry> = stored_stats(installed.as_slice(), source.clean.hash_length).expect("the entries are plain");
    let entry: &StatEntry = entries.iter().find(|found: &&StatEntry| { return found.path == b"a.txt"; }).expect("the entry of the edited file");
    let file: std::fs::Metadata = std::fs::metadata(repo.join("a.txt")).expect("the file's stat");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("the edit"), EDITED);
    assert_eq!((file.size(), file.mtime()), (EDITED.len() as u64, crate::worktree_ladder_test_support::HIDDEN_EDIT_SECONDS as i64), "the edit keeps the length and the modification time the index recorded");
    assert!(
        !matches_stored(&fields_of(&file), &entry.stored),
        "the installed index records the edited file's own stat fields (ctime {}:{}, recorded {}:{}), so Git takes the edit for the entry's content",
        file.ctime(),
        file.ctime_nsec(),
        entry.stored.ctime_seconds,
        entry.stored.ctime_nanos
    );
    remove(root.as_path());
}
