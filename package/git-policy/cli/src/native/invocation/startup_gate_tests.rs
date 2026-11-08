//! Controls for the startup step of a command.

use super::*;
use crate::test_support::{fixture, git, git_context, remove};

/// A canonical transaction name an earlier release gave its commit directories.
const TRANSACTION: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// The identity of the main worktree of the repository at `root`.
fn identity(root: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: root.join(".git"),
        git_dir: root.join(".git"),
        worktree_root: root.to_path_buf(),
    };
}

/// A repository with nothing left by an earlier release goes on,
/// and so does a command outside a worktree.
#[test]
fn a_clean_repository_and_a_place_outside_a_worktree_go_on() {
    let root: PathBuf = fixture("startup-gate-clean");
    git(&root, &["init", "--quiet"]);
    assert_eq!(recover_for_command(&git_context(), &identity(root.as_path()), &[]), Gate::Clear);
    assert_eq!(recover_for_command(&git_context(), &WorktreeIdentity::OutsideWorktree, &[]), Gate::Clear);
    remove(root.as_path());
}

/// The single-journal directory of the previous executable stops the command with the way to recover it,
/// and nothing is touched.
#[test]
fn the_previous_executables_journal_directory_stops_the_command() {
    let root: PathBuf = fixture("startup-gate-legacy");
    git(&root, &["init", "--quiet"]);
    let legacy: PathBuf = root.join(".git").join(LEGACY_JOURNAL_DIRECTORY_NAME);
    std::fs::create_dir(&legacy).expect("legacy directory");
    assert_eq!(
        recover_for_command(&git_context(), &identity(root.as_path()), &[]),
        Gate::Stop(legacy_notice(legacy.as_path()))
    );
    assert!(legacy.is_dir(), "nothing was removed");
    remove(root.as_path());
}

/// A registry that holds only locks and staging names has nothing to look at;
/// a transaction directory in it is named.
#[test]
fn a_registry_note_appears_only_for_a_transaction_directory() {
    let root: PathBuf = fixture("startup-gate-registry");
    assert_eq!(registry_note(root.join("absent").as_path()), None, "no registry, no note");
    let registry: PathBuf = root.join(REGISTRY_DIRECTORY_NAME);
    std::fs::create_dir_all(registry.join("landing.lock")).expect("lock directory");
    std::fs::create_dir_all(registry.join(format!("{TRANSACTION}.pending"))).expect("staging directory");
    assert_eq!(registry_note(registry.as_path()), None, "locks and staging names are not commits");
    std::fs::create_dir_all(registry.join(TRANSACTION)).expect("transaction directory");
    let note: String = registry_note(registry.as_path()).expect("a note");
    assert!(note.contains(registry.to_string_lossy().as_ref()), "{note}");
    assert!(note.contains("git status"), "the way to see whether the commit was made: {note}");
    assert!(note.contains("no longer uses that directory"), "it says the directory is unused: {note}");
    assert!(registry.join(TRANSACTION).is_dir(), "nothing was removed");
    remove(root.as_path());
}

/// The command the note ends with,
/// which is everything after its last colon-and-space marker,
/// without the line break.
fn removal_command(note: &str) -> String {
    let (_, command) = note.rsplit_once("directory with: ").expect("the note ends with the removal command");
    return command.trim_end_matches('\n').to_string();
}

/// Running a note's removal command in a shell removes the registry it names and nothing beside it,
/// whatever the name holds.
#[test]
fn the_removal_command_in_the_note_removes_exactly_the_registry() {
    let root: PathBuf = fixture("startup-gate-removal");
    for name in ["plain", "with space", "it's quoted", "$HOME `date` \"x\" \\ ; echo", "line\nbreak", "-dash"] {
        let git_dir: PathBuf = root.join(name);
        let registry: PathBuf = git_dir.join(REGISTRY_DIRECTORY_NAME);
        std::fs::create_dir_all(registry.join(TRANSACTION)).expect("transaction directory");
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("a sibling file that must stay");
        let note: String = registry_note(registry.as_path()).expect("a note");
        assert!(note.starts_with("monochromatic-git-policy-cli: "), "{note}");
        let command: String = removal_command(note.as_str());
        let status = std::process::Command::new("sh").arg("-c").arg(command.as_str()).status().expect("sh runs");
        assert!(status.success(), "{name:?}: {command}");
        assert!(!registry.exists(), "{name:?}: the command removed the registry: {command}");
        assert!(git_dir.join("HEAD").is_file(), "{name:?}: the command left the sibling file: {command}");
    }
    remove(root.as_path());
}

/// A registry whose name is not UTF-8 gets a command in the `$'...'` form,
/// which bash reads back as the exact bytes.
#[cfg(unix)]
#[test]
fn the_removal_command_names_a_registry_that_is_not_utf8_by_its_bytes() {
    /// The Unix conversion from raw bytes.
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("startup-gate-removal-bytes");
    let git_dir: PathBuf = root.join(std::ffi::OsString::from_vec(b"bytes\xff'\\\n\x01".to_vec()));
    let registry: PathBuf = git_dir.join(REGISTRY_DIRECTORY_NAME);
    std::fs::create_dir_all(registry.join(TRANSACTION)).expect("transaction directory");
    std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("a sibling file that must stay");
    let note: String = registry_note(registry.as_path()).expect("a note");
    let command: String = removal_command(note.as_str());
    assert!(command.contains("$'"), "bytes that are not UTF-8 use the ANSI-C form: {command}");
    let status = std::process::Command::new("bash").arg("-c").arg(command.as_str()).status().expect("bash runs");
    assert!(status.success(), "{command}");
    assert!(!registry.exists(), "the command removed the registry: {command}");
    assert!(git_dir.join("HEAD").is_file(), "the command left the sibling file: {command}");
    remove(root.as_path());
}

/// A word quotes text with the quote spelling a shell reads back,
/// and leaves nothing for the shell to expand.
#[test]
fn a_shell_word_quotes_a_single_quote_and_leaves_nothing_to_expand() {
    assert_eq!(shell_word(Path::new("/a b/c")), "'/a b/c'");
    assert_eq!(shell_word(Path::new("/it's")), "'/it'\\''s'");
    assert_eq!(shell_word(Path::new("$HOME")), "'$HOME'");
}

/// A transaction directory an earlier release left does not stop the command:
/// the note goes to standard error and the command goes on.
#[test]
fn a_registry_of_an_earlier_release_does_not_stop_the_command() {
    let root: PathBuf = fixture("startup-gate-registry-goes-on");
    git(&root, &["init", "--quiet"]);
    std::fs::create_dir_all(root.join(".git").join(REGISTRY_DIRECTORY_NAME).join(TRANSACTION)).expect("transaction directory");
    assert_eq!(recover_for_command(&git_context(), &identity(root.as_path()), &[]), Gate::Clear);
    remove(root.as_path());
}

/// What:
///  A canonical root,
///  a home below it whose identity file is in phase `after`,
///  a link to a folder on a drive that is not mounted,
///  and the environment that names that home.
/// Why:
///  The link gate reads one file,
///  so the control needs nothing else of a real move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = { root, home, link, environment: [['HOME', home]] };
/// ```
#[cfg(unix)]
struct InvocationHome {
    root: PathBuf,
    link: PathBuf,
    environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

/// A world whose recorded link dangles, with the environment that names its home.
#[cfg(unix)]
fn invocation_home(name: &str) -> InvocationHome {
    let root: PathBuf = std::fs::canonicalize(fixture(name)).expect("root");
    let home: PathBuf = root.join("home");
    let mount_point: PathBuf = root.join("not-a-mount");
    let link: PathBuf = root.join("repolink");
    let target: PathBuf = mount_point.join("repository/Monochromatic");
    std::fs::create_dir_all(mount_point.as_path()).expect("mount point stand-in");
    std::os::unix::fs::symlink(target.as_path(), link.as_path()).expect("dangling link");
    let identity: PathBuf = home.join(crate::repository_link_identity::IDENTITY_RELATIVE_PATH);
    std::fs::create_dir_all(identity.parent().expect("parent")).expect("config folder");
    std::fs::write(
        identity,
        format!(
            "{{\"linkPath\":\"{}\",\"linkText\":\"{}\",\"canonicalRoot\":\"{}\",\"mountPoint\":\"{}\",\"driveUuid\":\"D\",\"nvmeUuid\":\"N\",\"rootCommit\":\"abc\",\"phase\":\"after\"}}",
            link.display(),
            target.display(),
            target.display(),
            mount_point.display()
        ),
    )
    .expect("identity file");
    return InvocationHome { root, link, environment: vec![(std::ffi::OsString::from("HOME"), home.into_os_string())] };
}

/// The link gate reads the home the invocation names,
/// not this process's:
/// a worktree under a recorded link that dangles stops the command,
/// as the entry gate already stops it.
#[cfg(unix)]
#[test]
fn the_link_gate_reads_the_home_the_invocation_names() {
    let world: InvocationHome = invocation_home("startup-gate-invocation-home");
    let worktree: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: world.link.join(".git"),
        git_dir: world.link.join(".git"),
        worktree_root: world.link.clone(),
    };
    match recover_for_command(&git_context(), &worktree, world.environment.as_slice()) {
        Gate::RepositoryUnreachable(stderr) => {
            assert!(stderr.starts_with("fatal: the 860 drive is not mounted yet ("), "{stderr}");
            assert!(stderr.contains(world.link.to_string_lossy().as_ref()), "{stderr}");
        }
        other => panic!("the invocation's home records a dangling link, so the command must stop: {other:?}"),
    }
    remove(world.root.as_path());
}
