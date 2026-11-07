//! What:
//!  Disposable-directory controls for the move of an earlier name to the current name.
//! Why:
//!  The move must keep every byte of what the earlier name held,
//!  lose nothing when two processes race to do it,
//!  finish after a crash between the look and the rename,
//!  and never fail the command that asked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parent = await mkdtemp(...); await mkdir(join(parent, 'cli-git')); expect(await placeKeptName(parent, DATA_FOLDER)).toBeRenamed();
//! ```

/// Import the functions under test and the pair of names they work on.
use super::{Look, Placement, carry_out, existing_kept_name, look, place_kept_name};
use crate::rename_that_never_replaces::rename_noreplace;
use crate::wrapper_names_old_and_new::{DATA_FOLDER, DISCARD_AUDIT_LOG, RenamedName};
use std::path::{Path, PathBuf};

/// Create one empty fixture directory owned by this test process and test name.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-earlier-name", name);
}

/// Make the earlier folder of a pair with one file in it.
fn earlier_folder_with_file(parent: &Path, name: RenamedName, content: &str) {
    let folder: PathBuf = parent.join(name.earlier);
    std::fs::create_dir_all(folder.as_path()).expect("create the earlier folder");
    std::fs::write(folder.join("saved"), content).expect("write the saved file");
}

/// Make the current folder of a pair with one file in it.
fn current_folder_with_file(parent: &Path, name: RenamedName, content: &str) {
    let folder: PathBuf = parent.join(name.current);
    std::fs::create_dir_all(folder.as_path()).expect("create the current folder");
    std::fs::write(folder.join("saved"), content).expect("write the saved file");
}

/// A rename that always fails the way a read-only Git directory does.
fn refusing_rename(_from: &Path, _to: &Path) -> std::io::Result<()> {
    return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
}

/// Only the earlier folder:
///  it is renamed in place,
///  its content is intact,
///  no second name is left,
///  and nothing is reported.
#[test]
fn only_the_earlier_folder_is_renamed_in_place_with_its_content() {
    let parent: PathBuf = fixture("only-earlier");
    earlier_folder_with_file(parent.as_path(), DATA_FOLDER, "kept bytes");
    let placed: Placement = place_kept_name(parent.as_path(), DATA_FOLDER);
    assert_eq!(placed, Placement { path: parent.join(DATA_FOLDER.current), notice: None });
    assert_eq!(std::fs::read_to_string(placed.path.join("saved")).expect("content moved"), "kept bytes");
    assert!(!parent.join(DATA_FOLDER.earlier).exists(), "the earlier name is gone");
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// A log file is a kind of its own:
///  the same move works on a file.
#[test]
fn an_earlier_log_file_is_renamed_in_place_with_its_bytes() {
    let parent: PathBuf = fixture("log-file");
    std::fs::write(parent.join(DISCARD_AUDIT_LOG.earlier), b"line one\nline two\n").expect("write the earlier log");
    let placed: Placement = place_kept_name(parent.as_path(), DISCARD_AUDIT_LOG);
    assert_eq!(placed.path, parent.join(DISCARD_AUDIT_LOG.current));
    assert_eq!(placed.notice, None);
    assert_eq!(std::fs::read(placed.path.as_path()).expect("bytes moved"), b"line one\nline two\n");
    assert!(!parent.join(DISCARD_AUDIT_LOG.earlier).exists());
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// A symbolic link under the earlier name is moved as a link and never followed.
#[test]
fn an_earlier_symbolic_link_is_moved_as_a_link() {
    let parent: PathBuf = fixture("symbolic-link");
    let target: PathBuf = parent.join("elsewhere");
    std::fs::create_dir_all(target.as_path()).expect("create the target");
    std::os::unix::fs::symlink(target.as_path(), parent.join(DATA_FOLDER.earlier)).expect("link");
    let placed: Placement = place_kept_name(parent.as_path(), DATA_FOLDER);
    assert_eq!(placed.notice, None);
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(placed.path.as_path()).expect("the link exists");
    assert!(metadata.file_type().is_symlink(), "the link itself was moved");
    assert!(target.is_dir(), "the target was not touched");
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// Only the current folder:
///  it is used and nothing is reported.
#[test]
fn only_the_current_folder_is_used_without_a_notice() {
    let parent: PathBuf = fixture("only-current");
    current_folder_with_file(parent.as_path(), DATA_FOLDER, "current bytes");
    assert_eq!(look(parent.as_path(), DATA_FOLDER), Look::Settled);
    let placed: Placement = place_kept_name(parent.as_path(), DATA_FOLDER);
    assert_eq!(placed, Placement { path: parent.join(DATA_FOLDER.current), notice: None });
    assert_eq!(std::fs::read_to_string(placed.path.join("saved")).expect("content"), "current bytes");
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// Neither folder:
///  the current path is returned and nothing is created or reported.
#[test]
fn neither_name_gives_the_current_path_and_creates_nothing() {
    let parent: PathBuf = fixture("neither");
    let placed: Placement = place_kept_name(parent.as_path(), DATA_FOLDER);
    assert_eq!(placed, Placement { path: parent.join(DATA_FOLDER.current), notice: None });
    assert_eq!(std::fs::read_dir(parent.as_path()).expect("list").count(), 0);
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// Both folders:
///  the current one is used,
///  the earlier one is left byte for byte by the placement,
///  and the placement says nothing because the fold (`earlier_names_folded_into_current.rs`) speaks when it waits or fails.
#[test]
fn both_names_use_the_current_and_leave_the_earlier_to_the_fold() {
    let parent: PathBuf = fixture("both");
    earlier_folder_with_file(parent.as_path(), DATA_FOLDER, "earlier bytes");
    current_folder_with_file(parent.as_path(), DATA_FOLDER, "current bytes");
    let placed: Placement = place_kept_name(parent.as_path(), DATA_FOLDER);
    assert_eq!(placed.path, parent.join(DATA_FOLDER.current));
    assert_eq!(placed.notice, None, "the placement of two existing names is silent");
    assert_eq!(std::fs::read_to_string(parent.join(DATA_FOLDER.earlier).join("saved")).expect("earlier"), "earlier bytes");
    assert_eq!(std::fs::read_to_string(parent.join(DATA_FOLDER.current).join("saved")).expect("current"), "current bytes");
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// The race:
///  this process looks and sees only the earlier name,
///  a competitor then renames it,
///  and this process's rename finds nothing to move.
///  One wins,
///  nothing is lost,
///  and the loser reports no error.
#[test]
fn a_competitor_that_renames_first_leaves_one_folder_and_no_error() {
    let parent: PathBuf = fixture("race-lost");
    earlier_folder_with_file(parent.as_path(), DATA_FOLDER, "raced bytes");
    let looked: Look = look(parent.as_path(), DATA_FOLDER);
    assert_eq!(looked, Look::OnlyEarlier);
    // The competing process does the whole move between this process's look and its rename.
    let competitor: Placement = place_kept_name(parent.as_path(), DATA_FOLDER);
    assert_eq!(competitor.notice, None);
    let loser: Placement = carry_out(looked, parent.as_path(), DATA_FOLDER, rename_noreplace);
    assert_eq!(loser, Placement { path: parent.join(DATA_FOLDER.current), notice: None });
    assert_eq!(std::fs::read_to_string(loser.path.join("saved")).expect("content"), "raced bytes");
    assert!(!parent.join(DATA_FOLDER.earlier).exists());
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// A competitor that made the current folder fresh after this process looked:
///  the rename refuses to replace it,
///  the earlier folder stays for the fold,
///  and the placement is silent.
#[test]
fn a_current_folder_that_appears_after_the_look_is_not_replaced() {
    let parent: PathBuf = fixture("race-appears");
    earlier_folder_with_file(parent.as_path(), DATA_FOLDER, "earlier bytes");
    let looked: Look = look(parent.as_path(), DATA_FOLDER);
    // An empty folder is the case a plain rename would silently replace.
    std::fs::create_dir_all(parent.join(DATA_FOLDER.current)).expect("the competitor's folder");
    let placed: Placement = carry_out(looked, parent.as_path(), DATA_FOLDER, rename_noreplace);
    assert_eq!(placed.path, parent.join(DATA_FOLDER.current));
    assert_eq!(placed.notice, None);
    assert_eq!(std::fs::read_to_string(parent.join(DATA_FOLDER.earlier).join("saved")).expect("earlier"), "earlier bytes");
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// The crash:
///  a process that looked and stopped before its rename changed nothing,
///  so the next run still sees only the earlier name and finishes the move.
#[test]
fn a_crash_between_the_look_and_the_rename_is_finished_by_the_next_run() {
    let parent: PathBuf = fixture("crash");
    earlier_folder_with_file(parent.as_path(), DATA_FOLDER, "crash bytes");
    // The crashed process: it looked, and was gone before it renamed.
    assert_eq!(look(parent.as_path(), DATA_FOLDER), Look::OnlyEarlier);
    assert!(parent.join(DATA_FOLDER.earlier).is_dir(), "nothing was moved");
    assert!(!parent.join(DATA_FOLDER.current).exists(), "no half-made current name");
    // The next run.
    let next: Placement = place_kept_name(parent.as_path(), DATA_FOLDER);
    assert_eq!(next, Placement { path: parent.join(DATA_FOLDER.current), notice: None });
    assert_eq!(std::fs::read_to_string(next.path.join("saved")).expect("content"), "crash bytes");
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// A filesystem that refuses the rename:
///  the earlier name is used for this run,
///  nothing is lost,
///  and the notice says the rename is tried again.
#[test]
fn a_refused_rename_keeps_the_earlier_name_in_use_with_a_notice() {
    let parent: PathBuf = fixture("refused");
    earlier_folder_with_file(parent.as_path(), DATA_FOLDER, "kept bytes");
    let placed: Placement = carry_out(Look::OnlyEarlier, parent.as_path(), DATA_FOLDER, refusing_rename);
    assert_eq!(placed.path, parent.join(DATA_FOLDER.earlier));
    let notice: String = placed.notice.expect("reported");
    assert!(notice.contains("tried again"), "{notice}");
    assert_eq!(std::fs::read_to_string(placed.path.join("saved")).expect("content"), "kept bytes");
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// A reader that must not move anything finds the earlier name when it is the only one,
///  and the current name otherwise.
#[test]
fn a_reader_finds_the_name_that_exists_and_moves_nothing() {
    let parent: PathBuf = fixture("reader");
    assert_eq!(existing_kept_name(parent.as_path(), DATA_FOLDER), parent.join(DATA_FOLDER.current));
    earlier_folder_with_file(parent.as_path(), DATA_FOLDER, "earlier bytes");
    assert_eq!(existing_kept_name(parent.as_path(), DATA_FOLDER), parent.join(DATA_FOLDER.earlier));
    assert!(parent.join(DATA_FOLDER.earlier).is_dir(), "nothing was renamed");
    current_folder_with_file(parent.as_path(), DATA_FOLDER, "current bytes");
    assert_eq!(existing_kept_name(parent.as_path(), DATA_FOLDER), parent.join(DATA_FOLDER.current));
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}

/// Adversarial parent paths:
///  spaces,
///  quotes,
///  a newline and a leading dash in the folder that holds the names must not change what is renamed.
#[test]
fn spaces_quotes_and_newlines_in_the_parent_path_do_not_change_the_move() {
    let root: PathBuf = fixture("adversarial");
    for (index, awkward) in ["with space", "it's", "double \"quote\"", "line\nbreak", "-dash", "$(echo)", "semi;colon"].iter().enumerate() {
        let parent: PathBuf = root.join(format!("{index}-{awkward}"));
        std::fs::create_dir_all(parent.as_path()).expect("create the awkward parent");
        earlier_folder_with_file(parent.as_path(), DATA_FOLDER, awkward);
        let placed: Placement = place_kept_name(parent.as_path(), DATA_FOLDER);
        assert_eq!(placed.notice, None, "{awkward:?}");
        assert_eq!(std::fs::read_to_string(placed.path.join("saved")).expect("content"), *awkward);
        assert!(!parent.join(DATA_FOLDER.earlier).exists(), "{awkward:?}");
    }
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Two real threads move the same name at once:
///  exactly one folder remains,
///  its content is intact,
///  and neither thread reports a failure.
#[test]
fn two_threads_moving_the_same_name_leave_one_folder_with_the_content() {
    let parent: PathBuf = fixture("two-threads");
    earlier_folder_with_file(parent.as_path(), DATA_FOLDER, "contended bytes");
    let placements: Vec<Placement> = std::thread::scope(|scope| {
        let first = scope.spawn(|| { return place_kept_name(parent.as_path(), DATA_FOLDER); });
        let second = scope.spawn(|| { return place_kept_name(parent.as_path(), DATA_FOLDER); });
        return vec![first.join().expect("first thread"), second.join().expect("second thread")];
    });
    for placed in &placements {
        assert_eq!(placed.path, parent.join(DATA_FOLDER.current));
        assert_eq!(placed.notice, None);
    }
    assert_eq!(std::fs::read_to_string(parent.join(DATA_FOLDER.current).join("saved")).expect("content"), "contended bytes");
    assert!(!parent.join(DATA_FOLDER.earlier).exists());
    std::fs::remove_dir_all(parent.as_path()).expect("remove only the fixture");
}
