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
/// standard output and standard error of an exit action.
fn exit_parts(action: Action) -> (i32, String, String) {
    let Action::Exit { code, stdout, stderr } = action else {
        panic!("doctor must end in an exit action");
    };
    return (code, stdout, stderr);
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
/// and one directory named two ways is scanned once.
#[test]
fn directories_are_the_default_then_the_configured_one_once() {
    let plain: HooksLocation = HooksLocation {
        common_directory: PathBuf::from("/repo/.git"),
        configured: None,
    };
    assert_eq!(hook_directories(&plain), vec![PathBuf::from("/repo/.git/hooks")]);
    let configured: HooksLocation = HooksLocation {
        common_directory: PathBuf::from("/repo/.git"),
        configured: Some(PathBuf::from("/repo/custom")),
    };
    assert_eq!(
        hook_directories(&configured),
        vec![PathBuf::from("/repo/.git/hooks"), PathBuf::from("/repo/custom")]
    );
    let same: HooksLocation = HooksLocation {
        common_directory: PathBuf::from("/repo/.git"),
        configured: Some(PathBuf::from("/repo/./sub/../.git/hooks/")),
    };
    assert_eq!(hook_directories(&same), vec![PathBuf::from("/repo/.git/hooks")]);
}

/// The report is one path per line and nothing else.
#[test]
fn the_report_is_one_path_per_line() {
    assert_eq!(render_hooks(&[]), "");
    assert_eq!(
        render_hooks(&[PathBuf::from("/a/hooks/pre-commit"), PathBuf::from("/b/update")]),
        "/a/hooks/pre-commit\n/b/update\n"
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
        assert!(is_lfs_shim(write_shim(repo.as_path(), name).as_path()), "{name}");
    }
    let (code, stdout, _stderr) = exit_parts(doctor_action(&git_context(), repo.as_path()));
    assert_eq!((code, stdout.as_str()), (0, ""));
    remove(root.as_path());
}

/// What:
///  A `doctor` run inside a Git the wrapper started reports the hooks of the person's directories,
///  and never the wrapper's own hooks-off path.
/// Why:
///   The wrapper's switches travel into a child Git as command-scope configuration,
/// and a Git query that reads Git's
///  answer sees them last,
/// so a run inside wrapper-started Git would scan `/dev/null/cli-git-no-hooks`,
/// list nothing,
/// and exit 0.
#[cfg(unix)]
#[test]
fn a_nested_doctor_lists_the_persons_hooks_and_not_the_wrappers() {
    let root: PathBuf = fixture("doctor-nested-hooks-path");
    let repo: PathBuf = repository(root.as_path(), "repo");
    executable(repo.join(".git/hooks/pre-commit").as_path(), b"#!/bin/sh\n");
    std::fs::create_dir(repo.join("custom")).expect("custom");
    executable(repo.join("custom/commit-msg").as_path(), b"#!/bin/sh\n");
    let mut context: GitContext = git_context();
    context.overlay.push((
        std::ffi::OsString::from("GIT_CONFIG_PARAMETERS"),
        std::ffi::OsString::from("'core.hookspath'='/dev/null/cli-git-no-hooks'"),
    ));
    // Without a configured hooks directory, the default one is still the one scanned.
    let (code, stdout, stderr) = exit_parts(doctor_action(&context, repo.as_path()));
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(stdout, format!("{}\n", repo.join(".git/hooks/pre-commit").display()), "{stderr}");
    // A configured one is read through the wrapper's own entry rather than replaced by it.
    git(repo.as_path(), &["config", "core.hooksPath", "custom"]);
    let (code, stdout, stderr) = exit_parts(doctor_action(&context, repo.as_path()));
    assert_eq!(code, 1, "{stderr}");
    assert_eq!(
        stdout,
        format!(
            "{}\n{}\n",
            repo.join(".git/hooks/pre-commit").display(),
            repo.join("custom/commit-msg").display()
        ),
        "{stderr}"
    );
    remove(root.as_path());
}
