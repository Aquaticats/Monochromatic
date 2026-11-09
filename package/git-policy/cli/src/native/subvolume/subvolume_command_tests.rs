//! Controls for the command:
//! finding the main checkout without Git from every kind of starting directory,
//! the refusal
//!  outside a repository,
//! the help,
//! and the preview through the real entry point (a preview changes nothing,
//! so it
//!  may run on any fixture).

use super::*;
use crate::subvolume_test_support::{PlainPlatform, make_checkout, migrate_arguments, run_command};
use crate::test_support::{fixture, git, remove, repository};

/// The parts of an exit action.
fn exit_parts(action: Action) -> (i32, String, String) {
    let Action::Exit { code, stdout, stderr } = action else { panic!("the command ends in an exit action") };
    return (code, stdout, stderr);
}

/// The main checkout is found from itself,
/// from a directory inside it,
/// and from a linked worktree and its
///  subdirectory,
/// each time as the same real path.
#[test]
fn the_main_checkout_is_found_from_every_starting_point() {
    let parent: PathBuf = fixture("command-root-discovery");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "../linked"]);
    std::fs::create_dir_all(parent.join("linked/sub")).expect("subdirectory");
    for start in [root.clone(), root.join("src/deep"), parent.join("linked"), parent.join("linked/sub")] {
        assert_eq!(main_checkout_root(start.as_path()), Ok(root.clone()), "{}", start.display());
    }
    remove(parent.as_path());
}

/// A start that does not exist,
/// one that is not in a repository,
/// and a linked worktree of a bare repository (whose
///  common directory is not called `.git`) are each refused with their own reason.
#[test]
fn starts_without_a_main_checkout_are_refused_with_the_reason() {
    let parent: PathBuf = fixture("command-root-refused");
    let reason: String = main_checkout_root(parent.join("absent").as_path()).expect_err("missing");
    assert!(reason.contains("cannot resolve"), "{reason}");
    let reason2: String = main_checkout_root(parent.as_path()).expect_err("no repository above");
    assert!(reason2.contains("not inside a Git working tree"), "{reason2}");
    let seed: PathBuf = repository(parent.as_path(), "seed");
    git(parent.as_path(), &["clone", "--quiet", "--bare", seed.to_str().expect("path"), "store.git"]);
    git(parent.join("store.git").as_path(), &["worktree", "add", "--quiet", "--detach", "../from-bare"]);
    let reason3: String = main_checkout_root(parent.join("from-bare").as_path()).expect_err("bare common directory");
    assert!(reason3.contains("is not named .git"), "{reason3}");
    remove(parent.as_path());
}

/// What:
///  A `.git` that is a named pipe with no writer ends discovery at once with a reason.
/// Why:
///   Discovery used a blocking read,
/// so a FIFO at `.git` waited for a writer that never comes,
/// with
///  nothing said and no end.
#[test]
fn a_named_pipe_where_the_gitfile_belongs_is_refused_at_once() {
    let parent: PathBuf = fixture("command-gitfile-pipe");
    let start: PathBuf = parent.join("checkout");
    std::fs::create_dir_all(start.as_path()).expect("directory");
    crate::subvolume_sync_attrs::make_fifo(start.join(".git").as_path(), 0o600).expect("named pipe");
    let discovery = crate::test_support::finished_within(move || { return main_checkout_root(start.as_path()); });
    let Some(outcome) = discovery else { panic!("discovery did not finish within five seconds") };
    let reason: String = outcome.expect_err("refused");
    assert!(reason.contains("not a plain file") && reason.contains(".git"), "{reason}");
    remove(parent.as_path());
}

/// What:
///  A `commondir` that is a named pipe with no writer ends discovery at once with a reason.
/// Why:
///   A linked worktree's discovery reads its administrative directory's `commondir`,
/// and a blocking read there
///  waits with nothing said and no end.
#[test]
fn a_named_pipe_where_the_commondir_belongs_is_refused_at_once() {
    let parent: PathBuf = fixture("command-commondir-pipe");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "../linked"]);
    let common: PathBuf = root.join(".git/worktrees/linked/commondir");
    std::fs::remove_file(common.as_path()).expect("remove the commondir file");
    crate::subvolume_sync_attrs::make_fifo(common.as_path(), 0o600).expect("named pipe");
    let start: PathBuf = parent.join("linked");
    let discovery = crate::test_support::finished_within(move || { return main_checkout_root(start.as_path()); });
    let Some(outcome) = discovery else { panic!("discovery did not finish within five seconds") };
    let reason: String = outcome.expect_err("refused");
    assert!(reason.contains("not a plain file") && reason.contains("commondir"), "{reason}");
    remove(parent.as_path());
}

/// A `.git` file that is not a gitfile,
/// one whose administrative directory has no `commondir`,
/// and one that cannot be
///  read are each refused;
/// converting the wrong directory would be harmful.
#[test]
fn a_gitfile_that_does_not_lead_to_a_main_checkout_is_refused() {
    let parent: PathBuf = fixture("command-gitfile");
    std::fs::create_dir(parent.join("not-gitfile")).expect("directory");
    std::fs::write(parent.join("not-gitfile/.git"), b"something else\n").expect("file");
    let reason: String = main_checkout_root(parent.join("not-gitfile").as_path()).expect_err("no gitdir line");
    assert!(reason.contains("not a gitfile"), "{reason}");
    std::fs::create_dir_all(parent.join("submodule/admin")).expect("directories");
    std::fs::write(parent.join("submodule/.git"), b"gitdir: admin\n").expect("file");
    let reason2: String = main_checkout_root(parent.join("submodule").as_path()).expect_err("no commondir");
    assert!(reason2.contains("has no commondir"), "{reason2}");
    std::fs::create_dir_all(parent.join("dangling/admin")).expect("directories");
    std::fs::write(parent.join("dangling/.git"), b"gitdir: admin\n").expect("file");
    std::fs::write(parent.join("dangling/admin/commondir"), b"../nowhere\n").expect("commondir");
    let reason3: String = main_checkout_root(parent.join("dangling").as_path()).expect_err("common directory missing");
    assert!(reason3.contains("cannot resolve the common directory"), "{reason3}");
    std::fs::create_dir(parent.join("unreadable")).expect("directory");
    std::os::unix::fs::symlink("nowhere", parent.join("unreadable/.git")).expect("dangling link");
    let reason4: String = main_checkout_root(parent.join("unreadable").as_path()).expect_err("unreadable gitfile");
    assert!(reason4.contains("cannot read"), "{reason4}");
    remove(parent.as_path());
}

/// The help is printed on standard output with status 0,
/// whatever directory the command runs in.
#[test]
fn the_help_needs_no_repository() {
    let parent: PathBuf = fixture("command-help");
    let arguments: MigrateArguments = MigrateArguments { help: true, ..migrate_arguments() };
    assert_eq!(exit_parts(migrate_action(&arguments, parent.as_path())), (0, String::from(MIGRATE_HELP), String::new()));
    remove(parent.as_path());
}

/// Outside a repository the command says so with status 1 and the instruction,
/// and changes nothing.
#[test]
fn outside_a_repository_the_command_refuses_with_status_one() {
    let parent: PathBuf = fixture("command-outside");
    let (code, stdout, stderr) = exit_parts(migrate_action(&migrate_arguments(), parent.as_path()));
    assert_eq!((code, stdout.as_str()), (REFUSED_EXIT_CODE, ""));
    assert!(stderr.contains("refused:") && stderr.contains("What to do:") && stderr.ends_with("Nothing was changed.\n"), "{stderr}");
    remove(parent.as_path());
}

/// The preview through the real entry point (the Linux platform) ends with status 0,
/// creates nothing beside the
///  checkout,
/// and is the same from a linked worktree.
/// On a filesystem without btrfs it is refused instead,
/// with status 1
///  and the name of the filesystem;
/// both are the entry point's correct answers,
/// and the test checks the one that applies.
#[test]
fn the_preview_through_the_entry_point_changes_nothing() {
    let parent: PathBuf = fixture("command-preview");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "../linked"]);
    let before: Vec<String> = names(parent.as_path());
    let arguments: MigrateArguments = MigrateArguments { dry_run: true, ..migrate_arguments() };
    let on_btrfs: bool = crate::fs_kind::filesystem_kind(root.as_path()) == Ok(crate::fs_kind::FilesystemKind::Btrfs);
    for start in [root.clone(), parent.join("linked")] {
        let (code, _stdout, _stderr) = exit_parts(migrate_action(&arguments, start.as_path()));
        // A checkout that is already a subvolume (inode 256) is refused as "nothing to do" with status 1.
        let already: bool = std::os::unix::fs::MetadataExt::ino(&std::fs::metadata(root.as_path()).expect("metadata")) == 256;
        let expected: i32 = if on_btrfs && !already { 0 } else { REFUSED_EXIT_CODE };
        assert_eq!(code, expected, "{}", start.display());
    }
    assert_eq!(names(parent.as_path()), before, "a preview creates nothing beside the checkout");
    remove(parent.as_path());
}

/// The names in a directory,
/// sorted.
fn names(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(directory).expect("listing").flatten().map(|item| { return item.file_name().to_string_lossy().into_owned(); }).collect();
    found.sort();
    return found;
}

/// Several refusals are each printed with their remedy and one closing line.
#[test]
fn every_refusal_is_printed_with_its_remedy_and_one_closing_line() {
    let (parent, root) = crate::subvolume_test_support::checkout_with_worktree("command-refusals");
    std::fs::write(root.join(".git/MERGE_HEAD"), b"x").expect("marker");
    std::fs::write(root.join(".git/CHERRY_PICK_HEAD"), b"x").expect("marker");
    let (code, lines) = run_command(&PlainPlatform::new(), &migrate_arguments(), root.as_path(), &crate::subvolume_error::Interrupt::never());
    assert_eq!(code, REFUSED_EXIT_CODE);
    assert_eq!(lines.iter().filter(|line: &&String| { return line.contains("refused:"); }).count(), 2, "{lines:#?}");
    assert_eq!(lines.iter().filter(|line: &&String| { return line.trim_start().starts_with("What to do:"); }).count(), 2);
    assert_eq!(lines.last().map(String::as_str), Some("Nothing was changed."));
    remove(parent.as_path());
}

/// What:
///  A preview runs the refusals of the mode it previews.
/// Why:
///   `--dry-run --remove-old` on a checkout with no journal said the command "would delete the
/// original tree" while the same invocation without `--dry-run` refused,
/// because the preview
/// returned before the refusals were looked at;
/// a preview that promises what the real run refuses
/// answers a question nobody asked it to check.
#[test]
fn a_preview_refuses_what_the_real_run_would_refuse() {
    let platform: PlainPlatform = PlainPlatform::new();
    let parent: PathBuf = fixture("command-preview-refusals");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    for (mode, arguments) in [
        (MigrateMode::Rollback, MigrateArguments { mode: MigrateMode::Rollback, dry_run: true, ..migrate_arguments() }),
        (MigrateMode::RemoveOld, MigrateArguments { mode: MigrateMode::RemoveOld, dry_run: true, ..migrate_arguments() }),
    ] {
        let (code, lines) = run_command(&platform, &arguments, root.as_path(), &Interrupt::never());
        let said: String = lines.join("\n");
        assert_eq!(code, REFUSED_EXIT_CODE, "{mode:?}: {said}");
        let expected: &str = if mode == MigrateMode::Rollback { "no migration journal" } else { "not recorded as finished" };
        assert!(said.contains("refused") && said.contains(expected), "{mode:?}: {said}");
        assert!(!said.contains("dry run:"), "{mode:?}: a preview promises nothing the run refuses: {said}");
        // The real run of the same mode refuses with the same lines, which is what the preview promises.
        let (real_code, real_lines) = run_command(&platform, &MigrateArguments { dry_run: false, ..arguments.clone() }, root.as_path(), &Interrupt::never());
        assert_eq!(real_code, REFUSED_EXIT_CODE, "{mode:?}: {real_lines:#?}");
        assert_eq!(refusals_said(&lines), refusals_said(&real_lines), "{mode:?}");
    }
    // The positive control: a finished migration is previewed as the deletion the real run makes.
    let (migrated, migration_lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    assert_eq!(migrated, 0, "{migration_lines:#?}");
    let preview: MigrateArguments = MigrateArguments { mode: MigrateMode::RemoveOld, dry_run: true, ..migrate_arguments() };
    let (code, lines) = run_command(&platform, &preview, root.as_path(), &Interrupt::never());
    assert_eq!(code, 0, "{lines:#?}");
    assert!(lines.join("\n").contains("would delete"), "{lines:#?}");
    remove(parent.as_path());
}

/// The refusal lines of a run,
/// which are the ones a preview must match.
fn refusals_said(lines: &[String]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in lines {
        if line.contains("refused:") || line.starts_with("  What to do:") {
            found.push(line.clone());
        }
    }
    return found;
}
