//! Controls for `git monochromatic-git-policy-cli doctor`:
//! which entries count as traditional hooks,
//! which directories are scanned,
//! and the exit status and output of the command.

use super::*;
use crate::test_support::{executable, fixture, git, git_context, remove, repository};

/// Names of the paths in a list,
/// for compact comparison.
fn names(paths: &[PathBuf]) -> Vec<String> {
    return paths
        .iter()
        .map(|path| { return path.file_name().expect("name").to_string_lossy().into_owned(); })
        .collect();
}

/// Unpack the exit status,
/// standard output as bytes and standard error of an exit action.
fn exit_bytes_parts(action: Action) -> (i32, Vec<u8>, String) {
    return match action {
        Action::Exit { code, stdout, stderr } => (code, stdout.into_bytes(), stderr),
        Action::ExitBytes { code, stdout, stderr } => (code, stdout, stderr),
        other => panic!("doctor must end in an exit action, not {other:?}"),
    };
}

/// Unpack the exit status,
/// standard output and standard error of an exit action that reports text.
fn exit_parts(action: Action) -> (i32, String, String) {
    let (code, stdout, stderr) = exit_bytes_parts(action);
    return (code, String::from_utf8(stdout).expect("a report of text paths"), stderr);
}

/// The hooks location of `cwd`,
/// located by a task of the runtime,
/// because locating it asks Git.
fn located(context: &GitContext, cwd: &Path) -> Result<HooksLocation, String> {
    return crate::lazy_runtime::run_on_runtime(locate_hooks(context, cwd)).expect("the runtime");
}

/// Executable regular files and symbolic links are hooks;
/// samples,
/// plain files and directories are not.
#[cfg(unix)]
#[test]
fn only_executable_files_and_links_are_hooks() {
    let root: PathBuf = fixture("doctor-scan");
    let hooks: PathBuf = root.join("hooks");
    std::fs::create_dir(&hooks).expect("hooks");
    executable(hooks.join("pre-commit").as_path(), b"#!/bin/sh\n");
    executable(hooks.join("commit-msg.sample").as_path(), b"#!/bin/sh\n");
    std::fs::write(hooks.join("post-commit"), b"#!/bin/sh\n").expect("plain file");
    std::fs::create_dir(hooks.join("pre-push")).expect("directory");
    std::os::unix::fs::symlink(root.join("nowhere"), hooks.join("update")).expect("dangling link");
    std::os::unix::fs::symlink(hooks.join("pre-commit"), hooks.join("pre-rebase")).expect("working link");
    std::os::unix::fs::symlink(hooks.join("pre-commit"), hooks.join("applypatch-msg.sample")).expect("sample link");
    let found: Vec<PathBuf> = scan_hooks_directory(hooks.as_path()).expect("scan");
    assert_eq!(names(found.as_slice()), vec!["pre-commit", "pre-rebase", "update"]);
    assert!(found.iter().all(|path| { return path.starts_with(&hooks); }));
    remove(root.as_path());
}

/// A directory that does not exist,
/// or is a file,
/// holds no hooks;
/// nothing is created.
#[test]
fn a_missing_or_non_directory_path_holds_no_hooks() {
    let root: PathBuf = fixture("doctor-missing");
    assert_eq!(scan_hooks_directory(root.join("absent").as_path()).expect("scan"), Vec::<PathBuf>::new());
    std::fs::write(root.join("file"), b"x").expect("file");
    assert_eq!(scan_hooks_directory(root.join("file").as_path()).expect("scan"), Vec::<PathBuf>::new());
    assert!(!root.join("absent").exists());
    remove(root.as_path());
}

/// A directory that cannot be read is an error,
/// not silence.
#[cfg(unix)]
#[test]
fn an_unreadable_directory_is_an_error() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("doctor-unreadable");
    let hooks: PathBuf = root.join("hooks");
    std::fs::create_dir(&hooks).expect("hooks");
    std::fs::set_permissions(&hooks, std::fs::Permissions::from_mode(0o000)).expect("seal");
    let readable_anyway: bool = std::fs::read_dir(&hooks).is_ok();
    let outcome = scan_hooks_directory(hooks.as_path());
    std::fs::set_permissions(&hooks, std::fs::Permissions::from_mode(0o700)).expect("unseal");
    // A process that ignores permission bits (root) cannot be refused, so only a refusing process proves the error.
    if !readable_anyway {
        assert!(outcome.is_err());
    }
    remove(root.as_path());
}

/// The default directory comes first,
/// the configured one follows,
/// and one directory named two ways is scanned once,
/// by the identity the file system gives it.
#[test]
fn directories_are_the_default_then_the_configured_one_once() {
    let root: PathBuf = fixture("doctor-directories");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let hooks: PathBuf = repo.join(".git/hooks");
    std::fs::create_dir(repo.join("custom")).expect("custom hooks directory");
    let plain: HooksLocation = HooksLocation {
        common_directory: repo.join(".git"),
        git_directory: repo.join(".git"),
        configured: None,
    };
    assert_eq!(hook_directories(&plain), vec![hooks.clone()]);
    let configured: HooksLocation = HooksLocation {
        common_directory: repo.join(".git"),
        git_directory: repo.join(".git"),
        configured: Some(repo.join("custom")),
    };
    assert_eq!(hook_directories(&configured), vec![hooks.clone(), repo.join("custom")]);
    // The same directory through another spelling of a component that exists is scanned once.
    let same: HooksLocation = HooksLocation {
        common_directory: repo.join(".git"),
        git_directory: repo.join(".git"),
        configured: Some(repo.join("./custom")),
    };
    assert_eq!(hook_directories(&same), vec![hooks.clone(), repo.join("./custom")]);
    // The default hooks directory named again by the configured path is one directory too.
    let itself: HooksLocation = HooksLocation {
        common_directory: repo.join(".git"),
        git_directory: repo.join(".git"),
        configured: Some(repo.join(".git/./hooks")),
    };
    assert_eq!(hook_directories(&itself), vec![hooks.clone()]);
    // A spelling whose directory does not exist is a directory that holds no hooks either way, so it is kept as given.
    let absent: HooksLocation = HooksLocation {
        common_directory: repo.join(".git"),
        git_directory: repo.join(".git"),
        configured: Some(PathBuf::from("/repo/./sub/../absent")),
    };
    assert_eq!(hook_directories(&absent), vec![hooks, PathBuf::from("/repo/./sub/../absent")]);
    remove(root.as_path());
}

/// The report is one pathname per line and nothing else,
/// as bytes.
#[test]
fn the_report_is_one_path_per_line() {
    assert_eq!(render_hooks(&[]), Vec::<u8>::new());
    assert_eq!(
        render_hooks(&[PathBuf::from("/a/hooks/pre-commit"), PathBuf::from("/b/update")]),
        b"/a/hooks/pre-commit\n/b/update\n".to_vec()
    );
}

/// The locator asks Git,
/// so a relative `core.hooksPath` resolves against the worktree root,
/// not the current directory.
#[test]
fn a_relative_hooks_path_resolves_against_the_worktree_root() {
    let root: PathBuf = fixture("doctor-locate");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir(repo.join("sub")).expect("sub");
    let context: GitContext = git_context();
    let plain: HooksLocation = located(&context, repo.join("sub").as_path()).expect("locate");
    assert_eq!(plain.common_directory, repo.join(".git"));
    assert_eq!(plain.configured, None);
    git(repo.as_path(), &["config", "core.hooksPath", "custom-hooks"]);
    let relative: HooksLocation = located(&context, repo.join("sub").as_path()).expect("locate");
    assert_eq!(relative.configured, Some(repo.join("custom-hooks")));
    git(repo.as_path(), &["config", "core.hooksPath", "/elsewhere/hooks"]);
    let absolute: HooksLocation = located(&context, repo.as_path()).expect("locate");
    assert_eq!(absolute.configured, Some(PathBuf::from("/elsewhere/hooks")));
    remove(root.as_path());
}

/// A linked worktree shares the main repository's hooks directory.
#[test]
fn a_linked_worktree_reports_the_shared_common_directory() {
    let root: PathBuf = fixture("doctor-linked");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("UTF-8")]);
    let location: HooksLocation = located(&git_context(), linked.as_path()).expect("locate");
    assert_eq!(location.common_directory, repo.join(".git"));
    remove(root.as_path());
}

/// A bare repository has no worktree top level,
/// so a relative `core.hooksPath` resolves against its Git directory.
#[test]
fn a_relative_hooks_path_in_a_bare_repository_resolves_against_the_git_directory() {
    let root: PathBuf = fixture("doctor-bare");
    let bare: PathBuf = root.join("bare.git");
    std::fs::create_dir(bare.as_path()).expect("bare directory");
    git(bare.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main"]);
    git(bare.as_path(), &["config", "core.hooksPath", "custom-hooks"]);
    let location: HooksLocation = located(&git_context(), bare.as_path()).expect("locate");
    assert_eq!(location.common_directory, bare);
    assert_eq!(location.configured, Some(bare.join("custom-hooks")));
    remove(root.as_path());
}

/// A directory with no hooks ends with status 0 and prints only the line that says Git's fsmonitor is off.
#[test]
fn a_repository_without_traditional_hooks_exits_zero_silently() {
    let root: PathBuf = fixture("doctor-clean");
    let repo: PathBuf = repository(root.as_path(), "repo");
    executable(repo.join(".git/hooks/pre-commit.sample").as_path(), b"#!/bin/sh\n");
    let (code, stdout, stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!((code, stdout.as_str(), stderr.as_str()), (0, "", crate::doctor_fsmonitor::render_fsmonitor(&[]).as_str()));
    remove(root.as_path());
}

/// A hook in the default directory ends with status 1 and is listed by path.
#[cfg(unix)]
#[test]
fn a_default_directory_hook_is_listed_with_status_one() {
    let root: PathBuf = fixture("doctor-default");
    let repo: PathBuf = repository(root.as_path(), "repo");
    executable(repo.join(".git/hooks/pre-commit").as_path(), b"#!/bin/sh\n");
    let (code, stdout, stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 1);
    assert_eq!(stdout, format!("{}\n", repo.join(".git/hooks/pre-commit").display()));
    assert!(stderr.contains("never run through monochromatic-git-policy-cli"), "{stderr}");
    remove(root.as_path());
}

/// Hooks in a configured directory and in the default one are all listed,
/// the default directory first.
#[cfg(unix)]
#[test]
fn hooks_in_both_directories_are_listed_default_first() {
    let root: PathBuf = fixture("doctor-both");
    let repo: PathBuf = repository(root.as_path(), "repo");
    executable(repo.join(".git/hooks/commit-msg").as_path(), b"#!/bin/sh\n");
    std::fs::create_dir(repo.join("custom")).expect("custom");
    executable(repo.join("custom/pre-commit").as_path(), b"#!/bin/sh\n");
    git(repo.as_path(), &["config", "core.hooksPath", "custom"]);
    let (code, stdout, _stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 1);
    assert_eq!(
        stdout,
        format!(
            "{}\n{}\n",
            repo.join(".git/hooks/commit-msg").display(),
            repo.join("custom/pre-commit").display()
        )
    );
    remove(root.as_path());
}

/// Outside a repository the command reports the failure with status 2 and lists nothing.
#[test]
fn outside_a_repository_the_command_fails_with_status_two() {
    let root: PathBuf = fixture("doctor-outside");
    let (code, stdout, stderr) = exit_parts(doctor_action(&git_context(), root.as_path()));
    assert_eq!((code, stdout.as_str()), (2, ""));
    assert!(stderr.starts_with("git monochromatic-git-policy-cli doctor: "), "{stderr}");
    remove(root.as_path());
}

/// Write the shim git-lfs writes for `name` into the default hooks directory.
#[cfg(unix)]
fn write_shim(repo: &Path, name: &str) -> PathBuf {
    let path: PathBuf = repo.join(".git/hooks").join(name);
    executable(path.as_path(), lfs_shim_text(name).as_bytes());
    return path;
}

/// The shim text is the one git-lfs 3.8.0 writes:
/// its first,
/// last and name-bearing lines.
#[test]
fn the_shim_text_carries_the_hook_name_and_ends_with_a_line_feed() {
    let text: String = lfs_shim_text("post-merge");
    assert!(
        text.starts_with(
            "#!/bin/sh\ncommand -v git-lfs >/dev/null 2>&1 || { printf >&2 \"\\n%s\\n\\n\" \"This repository is configured for Git LFS"
        ),
        "{text}"
    );
    assert!(text.contains("deleting the 'post-merge' file in the hooks directory"), "{text}");
    assert!(text.ends_with("\ngit lfs post-merge \"$@\"\n"), "{text}");
    assert!(!text.contains("{{Command}}"));
}

/// Only shims alone end with status 0,
/// nothing on standard output,
/// and a report on standard error that says what
///  is happening,
/// that it is normal,
/// and that nothing needs doing.
#[cfg(unix)]
#[test]
fn only_git_lfs_shims_exit_zero_with_the_explanation() {
    let root: PathBuf = fixture("doctor-shims-only");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let pre_push: PathBuf = write_shim(repo.as_path(), "pre-push");
    write_shim(repo.as_path(), "post-checkout");
    let (code, stdout, stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!((code, stdout.as_str()), (0, ""));
    for needle in [
        "the Git LFS shims, byte for byte, and nothing needs doing",
        pre_push.to_str().expect("UTF-8"),
        "writes these files again whenever it cleans a file in a Git that monochromatic-git-policy-cli did not start",
        "so they can come back after you delete them",
        "this is normal",
        "uploads the LFS objects itself before every push",
        "starts Git with traditional hooks off",
        "never run through monochromatic-git-policy-cli",
        "delete them or leave them",
        "/usr/bin/git",
    ] {
        assert!(stderr.contains(needle), "{needle}: {stderr}");
    }
    assert!(!stderr.contains("unsupported"), "{stderr}");
    remove(root.as_path());
}

/// A foreign hook beside the shims ends with status 1 and lists only the foreign hook on standard output;
///  the shims are explained apart.
#[cfg(unix)]
#[test]
fn a_foreign_hook_beside_shims_still_ends_with_status_one() {
    let root: PathBuf = fixture("doctor-shims-and-foreign");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let shim: PathBuf = write_shim(repo.as_path(), "post-commit");
    executable(repo.join(".git/hooks/pre-commit").as_path(), b"#!/bin/sh\n");
    let (code, stdout, stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 1);
    assert_eq!(stdout, format!("{}\n", repo.join(".git/hooks/pre-commit").display()));
    assert!(stderr.contains("never run through monochromatic-git-policy-cli"), "{stderr}");
    assert!(stderr.contains(shim.to_str().expect("UTF-8")), "{stderr}");
    assert!(stderr.contains("nothing needs doing"), "{stderr}");
    remove(root.as_path());
}

/// A shim that differs by one byte,
/// a shim under another hook's name,
/// a link,
/// and a missing final line feed are
///  ordinary unsupported hooks.
#[cfg(unix)]
#[test]
fn a_modified_misnamed_or_linked_shim_is_an_unsupported_hook() {
    let root: PathBuf = fixture("doctor-shims-lookalikes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let hooks: PathBuf = repo.join(".git/hooks");
    let mut edited: String = lfs_shim_text("pre-push");
    edited.push('#');
    executable(hooks.join("pre-push").as_path(), edited.as_bytes());
    executable(hooks.join("post-commit").as_path(), lfs_shim_text("post-merge").as_bytes());
    let mut without_line_feed: String = lfs_shim_text("post-checkout");
    without_line_feed.pop();
    executable(hooks.join("post-checkout").as_path(), without_line_feed.as_bytes());
    executable(root.join("target-shim").as_path(), lfs_shim_text("post-merge").as_bytes());
    std::os::unix::fs::symlink(root.join("target-shim"), hooks.join("post-merge")).expect("link");
    executable(hooks.join("update").as_path(), lfs_shim_text("update").as_bytes());
    let (code, stdout, stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 1);
    assert_eq!(stdout.lines().count(), 5, "{stdout}");
    assert!(!stderr.contains("the Git LFS shims, byte for byte"), "{stderr}");
    remove(root.as_path());
}

/// Every one of the four names is recognized with its own text.
#[cfg(unix)]
/// What:
///  The report through its synchronous face and through the awaited one is the same report,
/// field for field.
/// Why:
///   The conversion must change no output:
/// the bridge is the only thing the synchronous face adds,
/// so a difference here is a change of behaviour and not of threading.
///  The hook listing,
/// the fsmonitor line and the configuration notes all come from Git,
/// so all three parts are compared.
#[test]
fn the_two_faces_of_the_report_agree() {
    let root: PathBuf = fixture("doctor-two-faces");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "hook.review.event", "post-commit"]);
    git(repo.as_path(), &["config", "hook.review.command", "run it"]);
    executable(repo.join(".git/hooks/pre-commit").as_path(), b"#!/bin/sh\n");
    let context: GitContext = git_context();
    let here: (i32, String, String) = exit_parts(doctor_action(&context, repo.as_path()));
    let awaited: (i32, String, String) = crate::lazy_runtime::run_on_runtime(async {
        return exit_parts(doctor_action_with_section(&context, repo.as_path(), None, &no_filesystem_section).await);
    })
    .expect("the runtime");
    assert_eq!(here, awaited);
    assert_eq!(here.0, 1);
    remove(root.as_path());
}

/// What:
///  The synchronous face of the report,
/// asked from inside the runtime,
/// is refused and says so.
/// Why:
///   This is what the control above would pass without:
/// the awaited report could be green for a reason other than the conversion,
/// and the bridge's refusal must reach the caller as a doctor failure rather than a panic.
#[test]
fn the_synchronous_face_of_the_report_is_refused_from_inside_the_runtime() {
    let root: PathBuf = fixture("doctor-sync-refused");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let refused: (i32, String, String) = crate::lazy_runtime::run_on_runtime(async { return exit_parts(doctor_action(&context, repo.as_path())); })
        .expect("the runtime");
    assert_eq!((refused.0, refused.1.as_str()), (2, ""));
    assert!(
        refused.2.starts_with("git monochromatic-git-policy-cli doctor: the runtime could not be entered: ")
            && refused.2.contains("inside itself"),
        "{}",
        refused.2
    );
    remove(root.as_path());
}

/// What:
///  The whole report is built by a task of the runtime it is awaited on,
/// and every Git question of it is put from that task.
/// Why:
///   Every Git step of the report entered the runtime for the pipes of a step that is ended at its ceiling,
/// and that entering is refused from inside the runtime,
/// so a task could not build the report at all.
/// This control can pass only when no step of the report calls the synchronous face,
/// which is the whole of the conversion.
///  The in-process readers are switched off,
/// because one of them could answer a question of the report without any child,
/// and then the control would hold whatever face the step used.
#[test]
fn the_report_is_built_on_the_runtime_it_is_awaited_on() {
    let root: PathBuf = fixture("doctor-awaited");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let hook: PathBuf = repo.join(".git/hooks/pre-commit");
    executable(hook.as_path(), b"#!/bin/sh\n");
    let mut context: GitContext = git_context();
    context.overlay.extend(crate::repo_read_hook::switch_off());
    let (code, stdout, stderr) = crate::lazy_runtime::run_on_runtime(async {
        return exit_parts(doctor_action_with_section(&context, repo.as_path(), None, &no_filesystem_section).await);
    })
    .expect("the runtime");
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(stdout, format!("{}\n", hook.display()));
    assert!(stderr.contains("never run through monochromatic-git-policy-cli"), "{stderr}");
    remove(root.as_path());
}

#[test]
fn each_of_the_four_shims_is_recognized() {
    let root: PathBuf = fixture("doctor-shims-all");
    let repo: PathBuf = repository(root.as_path(), "repo");
    for name in ["pre-push", "post-checkout", "post-commit", "post-merge"] {
        assert!(is_lfs_shim(write_shim(repo.as_path(), name).as_path()).expect("the shim is readable"), "{name}");
    }
    let (code, stdout, _stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!((code, stdout.as_str()), (0, ""));
    remove(root.as_path());
}

/// What:
///  A newline inside a path is part of that path,
/// never the end of one answer;
/// the repository Git selected is the one examined.
/// Why:
///  The answers were read as lines,
/// so a newline in the selected repository's own name ended the answer early and
///  the doctor scanned,
/// and renamed state in,
/// an unrelated directory beside it (`STB`).
#[cfg(unix)]
#[test]
fn a_newline_in_a_path_is_no_separator() {
    let root: PathBuf = fixture("doctor-newline");
    let decoy: PathBuf = root.join("newline-decoy");
    std::fs::create_dir_all(decoy.join("hooks")).expect("decoy hooks");
    executable(decoy.join("hooks/pre-commit").as_path(), b"#!/bin/sh\n");
    std::fs::create_dir_all(decoy.join("cli-git")).expect("decoy kept folder");
    std::fs::write(decoy.join("cli-git/do-not-move.txt"), b"unrelated-content\n").expect("decoy file");
    let repo: PathBuf = root.join("newline-decoy\nactual");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let context: GitContext = git_context();
    let location: HooksLocation = located(&context, repo.as_path()).expect("locate");
    assert_eq!(location.common_directory, repo.join(".git"));
    let (code, stdout, _stderr) = exit_parts(doctor_action(&context, repo.as_path()));
    assert_eq!((code, stdout.as_str()), (0, ""));
    assert!(decoy.join("cli-git/do-not-move.txt").exists(), "the unrelated directory keeps its folder");
    assert!(!decoy.join("monochromatic-git-policy-cli").exists(), "the unrelated directory was renamed");
    remove(root.as_path());
}

/// What:
///  A `core.hooksPath` whose bytes are not UTF-8 names a directory Git runs,
/// and the doctor reads it byte for byte.
/// Why:
///  A pathname is a byte string;
/// the answer was decoded as UTF-8 and a valid pathname was refused as an error,
///  although Git runs a hook there.
#[cfg(unix)]
#[test]
fn a_hooks_path_that_is_not_utf8_is_read_as_bytes() {
    use std::ffi::{OsStr, OsString};
    use std::os::unix::ffi::OsStrExt as _;
    let root: PathBuf = fixture("doctor-non-utf8");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let hooks: PathBuf = root.join(OsString::from(OsStr::from_bytes(b"hooks-\xff")));
    std::fs::create_dir(hooks.as_path()).expect("hooks");
    executable(hooks.join("pre-commit").as_path(), b"#!/bin/sh\n");
    git(repo.as_path(), &[OsStr::new("config"), OsStr::new("core.hooksPath"), hooks.as_os_str()]);
    let location: HooksLocation = located(&git_context(), repo.as_path()).expect("locate");
    assert_eq!(location.configured, Some(hooks.clone()));
    let (code, stdout, _stderr) = exit_bytes_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 1, "the hook is found: {:?}", String::from_utf8_lossy(stdout.as_slice()));
    // The report names the inspected pathname by its own bytes, which is the whole of this control: the lossy spelling
    //  names a directory that does not exist here.
    assert_eq!(stdout, [hooks.as_os_str().as_bytes(), b"/pre-commit\n".as_slice()].concat());
    remove(root.as_path());
}

/// What:
///  A report reads the names an earlier release kept,
/// and moves none of them.
/// Why:
///  The command's contract says it changes nothing (`SPEC.md`,
/// section Doctor),
/// and a check that moves what it inspects can hide the state it was asked about.
///  The saved copies are read through the folder that exists,
/// so the earlier name is still reported.
#[test]
fn the_report_moves_none_of_the_names_it_reads() {
    let root: PathBuf = fixture("doctor-read-only");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let earlier: PathBuf = repo.join(".git/cli-git");
    let save: PathBuf = earlier.join("discarded/2026-11-02T19-33-20");
    std::fs::create_dir_all(save.as_path()).expect("earlier kept folder");
    std::fs::write(save.join("copy.txt"), b"saved-content\n").expect("kept copy");
    std::fs::write(earlier.join("kept.txt"), b"saved-content\n").expect("kept file");
    let (code, _stdout, stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 0, "{stderr}");
    assert!(earlier.join("kept.txt").exists(), "the report moved a folder it was only reading: {stderr}");
    assert!(save.join("copy.txt").exists(), "a saved copy stays where it is");
    assert!(!repo.join(".git/monochromatic-git-policy-cli").exists(), "the report wrote the current name");
    assert!(stderr.contains("discarded copies:"), "the saved copies are still reported: {stderr}");
    remove(root.as_path());
}


/// A listing that never answers,
/// as a directory on a mount whose server stopped answering does not.
///
/// The thread parks,
/// so it spends no processor time while it waits:
/// this is a stall,
/// not a load.
fn stalled_listing(_directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    loop {
        std::thread::park();
    }
}

/// What:
///  A directory listing that never answers ends at the ceiling as an error,
/// never as an empty directory.
/// Why:
///  A mount whose server stopped answering holds `readdir` and `lstat` in the kernel with no end,
/// and the owner's rule is
///  "nothing should ever wait forever or indefinitely".
/// Silence would read as "no hooks here",
/// which is the one answer a
///  report must not give when it could not look.
#[test]
fn a_stalled_directory_listing_gives_up_at_its_ceiling() {
    let root: PathBuf = fixture("doctor-stalled-listing");
    let named: PathBuf = root.clone();
    let (done, finished) = std::sync::mpsc::channel::<std::io::Result<Vec<PathBuf>>>();
    let worker = std::thread::spawn(move || {
        let outcome: std::io::Result<Vec<PathBuf>> = scan_hooks_directory_with(stalled_listing, std::time::Duration::from_millis(50), named.as_path());
        let _sent = done.send(outcome);
    });
    // The worker is left behind on purpose: a read that never answers has no end, and the test process ends it.
    let answered: Result<std::io::Result<Vec<PathBuf>>, _> = finished.recv_timeout(std::time::Duration::from_secs(10));
    let _abandoned = worker;
    let outcome: std::io::Result<Vec<PathBuf>> = answered.unwrap_or_else(|_| { panic!("the listing never returned, so a directory on a mount that stopped answering holds the report forever") });
    let error: std::io::Error = outcome.expect_err("a listing that gives no answer is an error, never an empty directory");
    assert_eq!(error.kind(), ErrorKind::TimedOut, "{error}");
    remove(root.as_path());
}

/// What:
///  A `core.hooksPath` of `redirect/../hooks`,
/// where `redirect` is a symbolic link,
/// scans the directory Git runs hooks
///  in.
/// Why:
///  A lexical rewrite removes the component before `..` without asking the file system,
/// while the kernel resolves
///  `redirect` first:
/// Git ran the hook and the doctor listed nothing (`SPEC.md`,
/// section Doctor,
///  "the directory it names").
#[cfg(unix)]
#[test]
fn a_dot_dot_after_a_symlink_scans_the_directory_git_runs() {
    let root: PathBuf = fixture("doctor-symlink-parent");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let external: PathBuf = root.join("external");
    std::fs::create_dir_all(external.join("child")).expect("the redirect's target");
    std::fs::create_dir_all(external.join("hooks")).expect("the hooks directory Git reads");
    executable(external.join("hooks/pre-commit").as_path(), b"#!/bin/sh\necho HOOK-RAN >&2\n");
    std::os::unix::fs::symlink(external.join("child"), repo.join("redirect")).expect("the redirect");
    git(repo.as_path(), &["config", "core.hooksPath", "redirect/../hooks"]);
    let (code, stdout, _stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 1, "the hook Git runs is listed: {stdout}");
    assert!(stdout.contains("hooks/pre-commit"), "{stdout}");
    assert!(stdout.trim_end().ends_with("hooks/pre-commit"), "{stdout}");
    // The file the report names is the file Git runs, resolved by the file system and not by the spelling.
    let reported: PathBuf = PathBuf::from(stdout.trim_end());
    assert_eq!(std::fs::canonicalize(reported).expect("the reported hook exists"), std::fs::canonicalize(external.join("hooks/pre-commit")).expect("the ran hook exists"));
    remove(root.as_path());
}

/// What:
///  The checkout the section reports on is the one Git names,
/// and its layout is Git's answer about which worktree the command runs in.
/// Why:
///  The section used to read the directory above the common Git directory,
/// which is a metadata directory when a
///  repository keeps its Git directory elsewhere and a bare repository's parent when it keeps none;
/// both
///  were measured and named as places to migrate.
#[test]
fn the_checkout_is_the_one_git_names_and_its_layout_is_gits_answer() {
    let root: PathBuf = fixture("doctor-checkout-identity");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("UTF-8")]);
    let context: GitContext = git_context();
    let reported = |cwd: &Path| -> (Option<PathBuf>, CheckoutLayout) {
        let location: HooksLocation = located(&context, cwd).expect("locate");
        return crate::lazy_runtime::run_on_runtime(checkout_of(&context, cwd, &location)).expect("the runtime");
    };
    assert_eq!(reported(repo.as_path()), (Some(repo.clone()), CheckoutLayout::Main));
    assert_eq!(reported(linked.as_path()), (Some(linked.clone()), CheckoutLayout::Linked));
    remove(root.as_path());
}

/// A bare repository has no checkout at all,
/// and a repository whose Git directory lives elsewhere is reported on
///  its worktree,
/// which is what Git answers and no directory name is.
#[test]
fn a_bare_repository_has_no_checkout_and_a_separate_git_directory_keeps_its_worktree() {
    use std::ffi::OsStr;
    let root: PathBuf = fixture("doctor-checkout-layouts");
    let bare: PathBuf = root.join("bare-parent");
    std::fs::create_dir_all(bare.join(".git")).expect("bare Git directory");
    git(bare.join(".git").as_path(), &["init", "--quiet", "--bare", "--initial-branch=main"]);
    let worktree: PathBuf = root.join("real-worktree");
    let metadata: PathBuf = root.join("metadata");
    std::fs::create_dir(worktree.as_path()).expect("worktree directory");
    std::fs::create_dir(metadata.as_path()).expect("metadata directory");
    git(
        worktree.as_path(),
        &[OsStr::new("init"), OsStr::new("--quiet"), OsStr::new(format!("--separate-git-dir={}", metadata.join(".git").display()).as_str())],
    );
    let context: GitContext = git_context();
    let reported = |cwd: &Path| -> (Option<PathBuf>, CheckoutLayout) {
        let location: HooksLocation = located(&context, cwd).expect("locate");
        return crate::lazy_runtime::run_on_runtime(checkout_of(&context, cwd, &location)).expect("the runtime");
    };
    assert_eq!(reported(bare.join(".git").as_path()), (None, CheckoutLayout::Main));
    assert_eq!(reported(worktree.as_path()), (Some(worktree.clone()), CheckoutLayout::Main));
    remove(root.as_path());
}

/// A pathname resolution that never answers,
/// as one on a hard-mounted network file system whose server stopped answering does not.
///
/// The thread parks,
/// so it spends no processor time while it waits:
/// this is a stall,
/// not a load.
fn stalled_identity(_path: &Path) -> Option<PathBuf> {
    loop {
        std::thread::park();
    }
}

/// What:
///  A configured `core.hooksPath` is kept when deciding whether it is the default directory never answers.
/// Why:
///  Deciding that resolved both spellings before any ceiling started,
/// and pathname resolution waits in the kernel on a
///  mount whose server has stopped,
/// so the doctor waited with no end before it ever scanned.
/// The configured directory
///  is scanned when the answer does not come:
/// a directory looked at twice costs a duplicate line,
/// and a directory Git
///  runs hooks in that was never looked at costs the finding.
#[test]
fn a_directory_identity_that_never_answers_keeps_the_configured_directory() {
    let location: HooksLocation = HooksLocation {
        common_directory: PathBuf::from("/repo/.git"),
        git_directory: PathBuf::from("/repo/.git"),
        configured: Some(PathBuf::from("/mnt/gone/custom-hooks")),
    };
    let (done, finished) = std::sync::mpsc::channel::<Vec<PathBuf>>();
    let worker = std::thread::spawn(move || {
        let directories: Vec<PathBuf> = hook_directories_with(stalled_identity, std::time::Duration::from_millis(50), &location);
        let _sent = done.send(directories);
    });
    // The worker is left behind on purpose: a resolution that never answers has no end, and the test process ends it.
    let answered: Result<Vec<PathBuf>, _> = finished.recv_timeout(std::time::Duration::from_secs(10));
    let _abandoned = worker;
    let directories: Vec<PathBuf> = answered.unwrap_or_else(|_| {
        panic!("the identity check never returned, so pathname resolution on a stopped mount holds the doctor before its scan")
    });
    assert_eq!(
        directories,
        vec![PathBuf::from("/repo/.git/hooks"), PathBuf::from("/mnt/gone/custom-hooks")],
        "the configured directory is scanned when its identity is unknown"
    );
}

/// What:
///  The doctor's report carries the saved copies of both data folders,
/// including when the current folder holds none.
/// Why:
///  The report reads without moving anything,
/// so a repository that holds both folder names keeps the earlier one's
///  copies where they are;
/// a report that named only the current folder hid a person's recoverable data from
///  the person who asked what is there.
#[test]
fn the_doctor_reports_the_saved_copies_of_both_folders() {
    let root: PathBuf = fixture("doctor-discarded-both");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let save: PathBuf = repo.join(".git/cli-git/discarded/2026-11-02T19-33-20");
    std::fs::create_dir_all(save.as_path()).expect("the earlier save");
    std::fs::write(save.join("copy.txt"), b"saved-content\n").expect("the kept copy");
    // The current folder exists and holds no saved copy, which is the state that hid the older one.
    std::fs::create_dir_all(repo.join(".git/monochromatic-git-policy-cli").as_path()).expect("the current folder");
    let (code, _stdout, stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 0, "{stderr}");
    assert!(stderr.contains("discarded copies:"), "the saved copies are reported: {stderr}");
    // The block names the folder it read and the oldest save inside it by its name.
    assert!(stderr.contains(save.parent().expect("folder").display().to_string().as_str()), "the earlier folder is named: {stderr}");
    assert!(
        stderr.contains(save.file_name().expect("save name").to_string_lossy().as_ref()),
        "the kept save is named: {stderr}"
    );
    assert!(save.join("copy.txt").exists(), "the copy stays where it is");
    remove(root.as_path());
}

/// What:
///  A hook whose pathname is not UTF-8 is named by its own bytes,
/// never by the pathname its lossy spelling names.
/// Why:
///  `Path::display()` replaces each byte that is not UTF-8 with U+FFFD,
/// and a directory whose name is exactly that
///  replacement is a different directory:
/// the report named a hook in an unrelated folder,
/// and a person who
///  deleted what it named would delete somebody else's script.
#[cfg(unix)]
#[test]
fn a_hook_whose_path_is_not_utf8_is_named_by_its_bytes() {
    use std::ffi::{OsStr, OsString};
    use std::os::unix::ffi::OsStrExt as _;
    let root: PathBuf = fixture("doctor-non-utf8-exact");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let inspected: PathBuf = root.join(OsString::from(OsStr::from_bytes(b"hooks-\xff")));
    let replacement: PathBuf = root.join(OsString::from(OsStr::from_bytes(b"hooks-\xef\xbf\xbd")));
    for directory in [inspected.as_path(), replacement.as_path()] {
        std::fs::create_dir_all(directory).expect("hooks directory");
        executable(directory.join("pre-commit").as_path(), b"#!/bin/sh\n");
    }
    git(repo.as_path(), &[OsStr::new("config"), OsStr::new("core.hooksPath"), inspected.as_os_str()]);
    let (code, stdout, _stderr) = exit_bytes_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!(code, 1);
    assert_eq!(
        stdout,
        [inspected.as_os_str().as_bytes(), b"/pre-commit\n".as_slice()].concat(),
        "the inspected hook is named by its own bytes"
    );
    assert_ne!(
        stdout,
        [replacement.as_os_str().as_bytes(), b"/pre-commit\n".as_slice()].concat(),
        "the lossy spelling names another directory"
    );
    remove(root.as_path());
}
