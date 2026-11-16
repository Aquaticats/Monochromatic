//! What:
//!  What a removal does with a subvolume or a mount point it did not make,
//!  through the built executable,
//!  in a fixture of its own:
//!  it removes everything else,
//!  forgets the worktree in Git,
//!  leaves what it did not make exactly as it stood,
//!  and says what frees the path.
//! Why:
//!  The kernel cannot destroy a subvolume that contains another one,
//!  and Git's own removal descends into a mount point:
//!  so a worktree holding what a person put there had no removal at all,
//!  and the path stayed taken with nothing said.
//!  Every control here compares what stands in the volume before and after the removal,
//!  so a step that changed a mode, an owner or a byte fails it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = treeOf(`${root}/topic/snap`);
//! const removed = runWrapped(fixture, repo, ['worktree', 'remove', '../topic']);
//! expect(removed.code).toBe(0); expect(treeOf(`${root}/topic/snap`)).toEqual(before);
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, TIME_BOUND_SECONDS, fixture, git, hermetic, observe, on_btrfs_or_acknowledged, remove, repository, run_direct, run_wrapped, stderr_of, wrapper_first_path};
use git_policy_cli::fs_btrfs_ioctl::subvolume_create_at;
use git_policy_cli::fs_kind::is_subvolume_root;
use git_policy_cli::shell_quote::shell_quote;
use git_policy_cli::worktree_remove::DESTROY_NOTICE;
use std::ffi::OsStr;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The variable that states this run has no user namespace for a mount control.
/// The gate image sets it where unprivileged mounts are refused;
/// the host runs every one of these controls.
const NO_USER_NAMESPACE_VARIABLE: &str = "CLI_GIT_NO_USER_NAMESPACE_ACKNOWLEDGED";

/// A repository whose first commit carries the ignore rules, so that a volume a control puts inside a worktree is
/// either covered by them or listed by Git as untracked.
fn a_repository_with_rules(fixture: &Fixture, rules: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("a tracked file");
    git(fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    if !rules.is_empty() {
        std::fs::write(repo.join(".gitignore"), rules.as_bytes()).expect("the ignore rules");
        git(fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    }
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    return repo;
}

/// What one directory holds, as text:
///  every path below it, its kind, its mode, its owner, and a digest of a file's bytes,
///  sorted so two readings compare whatever order the kernel lists them in.
fn tree_of(root: &Path) -> Vec<String> {
    let mut shown: Vec<String> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let listed = match std::fs::read_dir(directory.as_path()) {
            Ok(listed) => listed,
            Err(error) => {
                shown.push(format!("{} cannot be listed ({error})", directory.display()));
                continue;
            }
        };
        for found in listed.flatten() {
            let path: PathBuf = found.path();
            let facts = match std::fs::symlink_metadata(path.as_path()) {
                Ok(facts) => facts,
                Err(error) => {
                    shown.push(format!("{} cannot be examined ({error})", path.display()));
                    continue;
                }
            };
            let kind: &str = match facts.file_type().is_symlink() {
                true => "link",
                false => match facts.is_dir() {
                    true => "dir",
                    false => "file",
                },
            };
            let relative: String = path.strip_prefix(root).unwrap_or(path.as_path()).display().to_string();
            if facts.is_dir() {
                shown.push(format!("{relative} {kind} {:o} {}", facts.mode() & 0o7777, facts.uid()));
                stack.push(path);
                continue;
            }
            let bytes: Vec<u8> = std::fs::read(path.as_path()).unwrap_or_default();
            let mut digest: u64 = 0xcbf2_9ce4_8422_2325;
            for byte in &bytes {
                digest ^= u64::from(*byte);
                digest = digest.wrapping_mul(0x100_0000_01b3);
            }
            shown.push(format!("{relative} {kind} {:o} {} {digest:016x} {}", facts.mode() & 0o7777, facts.uid(), bytes.len()));
        }
    }
    shown.sort();
    return shown;
}

/// The registration directories the repository holds now, by name.
fn registrations_of_the_repository(repo: &Path) -> Vec<String> {
    let Ok(listed) = std::fs::read_dir(repo.join(".git/worktrees")) else { return Vec::new() };
    let mut names: Vec<String> = listed.flatten().map(|found| return found.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    return names;
}

/// The porcelain `worktree list` of a repository as text.
fn listing(fixture: &Fixture, repo: &Path) -> String {
    return String::from_utf8_lossy(&git(fixture, repo, &["worktree", "list", "--porcelain"]).stdout).into_owned();
}

/// What:
///  A subvolume the wrapper did not make, inside a worktree,
///  with a file a person put there.
/// Why:
///  Every control needs the same volume,
///  and it is made with the kernel call the wrapper itself uses for a worktree,
///  which an unprivileged user may run on this filesystem.
fn a_volume_at(worktree: &Path, inside: &str, what: &str) -> PathBuf {
    let volume: PathBuf = worktree.join(inside);
    if let Some(parent) = volume.parent() {
        std::fs::create_dir_all(parent).expect("the directories the volume stands in");
    }
    subvolume_create_at(volume.as_path()).expect("a subvolume the wrapper did not make");
    std::fs::write(volume.join("precious.txt"), format!("{what}\n").as_bytes()).expect("a file the person put there");
    return volume;
}

/// What:
///  Age the tracked files of a worktree the control added,
///  and refresh the index,
///  so the removal's cleanliness proof judges their stat data.
/// Why:
///  A worktree the wrapper adds holds files whose times are the moment of the add,
///  and a file as new as the index
///  proves nothing by its stat fields.
fn age_tracked_files(fixture: &Fixture, worktree: &Path) {
    for name in ["tracked.txt", ".gitignore"] {
        let Ok(file) = std::fs::OpenOptions::new().write(true).open(worktree.join(name)) else { continue };
        file.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(120)).expect("age the tracked file");
    }
    git(fixture, worktree, &["update-index", "--refresh"]);
}

/// The report of a removal that went around one volume,
/// as every control checks it.
fn assert_the_report_of_a_removal_that_went_around(observed: &Observed, target: &Path, left: &[&Path]) {
    let said: String = stderr_of(observed);
    assert_eq!(observed.code, Some(0), "the removal finished: {said}");
    for volume in left {
        assert!(said.contains(volume.to_str().expect("a UTF-8 fixture path")), "the report names what was left: {said}");
    }
    assert!(said.contains(target.to_str().expect("a UTF-8 fixture path")), "and the worktree it removed: {said}");
    assert!(said.contains("left where it stands"), "and says that it stayed: {said}");
    assert!(said.contains("Git no longer lists it"), "and that Git forgot the worktree: {said}");
    assert!(said.contains("free for `git worktree add`"), "and what frees the path: {said}");
    assert!(said.contains("Nothing of what is left was moved, unmounted or deleted"), "and that nothing of it was touched: {said}");
}

/// Whether an unprivileged mount can be made in a namespace of this run, or the reason the control did not run.
fn a_private_mount_namespace_is_available(control: &str) -> bool {
    let mut probe: Command = Command::new("/usr/bin/unshare");
    probe.args(["--user", "--map-root-user", "--mount", "--propagation", "private", "/bin/true"]);
    let ran: Observed = observe(&mut probe, b"");
    if ran.code == Some(0) {
        return true;
    }
    let acknowledged: bool = std::env::var_os(NO_USER_NAMESPACE_VARIABLE).is_some_and(|value| { return !value.is_empty() });
    assert!(
        acknowledged,
        "the control {control} needs an unprivileged mount in a private namespace, and `unshare --user --map-root-user \
         --mount` answered {:?}: {}; run it where user namespaces are allowed, or set {NO_USER_NAMESPACE_VARIABLE}=1 to \
         state that this run has none",
        ran.code,
        stderr_of(&ran)
    );
    eprintln!("USER NAMESPACE CONTROL NOT RUN: {control} (unprivileged mounts are refused here)");
    return false;
}

/// What:
///  Run one script in a private mount namespace of this command's own,
///  with the wrapper first on `PATH`.
/// Why:
///  A mount a control makes needs a namespace nobody else shares,
///  and the rule that bounds every probe applies to it:
///  the script runs under `timeout`,
///  and the namespace ends with it.
fn inside_a_private_mount_namespace(fixture: &Fixture, directory: &Path, script: &str) -> Observed {
    let mut command: Command = Command::new("/usr/bin/unshare");
    command.args(["--user", "--map-root-user", "--mount", "--propagation", "private", "--", "/usr/bin/timeout", TIME_BOUND_SECONDS, "/bin/sh", "-c", script]);
    hermetic(&mut command, fixture, wrapper_first_path(fixture).as_os_str());
    command.current_dir(directory);
    return observe(&mut command, b"");
}

/// `path` as a word a shell script takes.
fn word(path: &Path) -> String {
    return shell_quote(path.as_os_str().as_encoded_bytes());
}

/// What:
///  A volume at the top of a worktree stays with the directories that lead to it,
///  every other entry of the worktree goes,
///  Git forgets the worktree,
///  and what stands in the volume is the same after as before.
/// Why:
///  This is the case the decision of 2026-11-12 answers ("Remove around it"):
///  the kernel cannot destroy a subvolume that holds another one,
///  so the worktree had no removal at all
///  and the person was sent to move the volume out first.
#[test]
fn a_volume_at_the_top_of_a_worktree_is_left_and_the_worktree_is_otherwise_removed() {
    let fixture: Fixture = fixture("removal-around-top");
    if !on_btrfs_or_acknowledged(&fixture, "removal-around-top") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, "snap/\n");
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let target: PathBuf = fixture.root.join("topic");
    assert!(is_subvolume_root(target.as_path()).expect("probe"), "the add made a subvolume");
    age_tracked_files(&fixture, target.as_path());
    let volume: PathBuf = a_volume_at(target.as_path(), "snap", "what I put there");
    let mode_before: u32 = std::fs::metadata(volume.as_path()).expect("the volume").mode() & 0o7777;
    let before: Vec<String> = tree_of(volume.as_path());
    let registrations_before: Vec<String> = registrations_of_the_repository(repo.as_path());

    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    assert_the_report_of_a_removal_that_went_around(&removed, target.as_path(), &[volume.as_path()]);

    assert_eq!(tree_of(volume.as_path()), before, "what stands in the volume is untouched");
    assert_eq!(std::fs::metadata(volume.as_path()).expect("the volume").mode() & 0o7777, mode_before, "and its mode is what it was");
    assert!(volume.is_dir(), "the volume is where it stood");
    assert!(target.is_dir(), "the worktree root stays because the volume stands in it");
    assert_eq!(
        std::fs::read_dir(target.as_path()).expect("the worktree root").flatten().map(|found| return found.file_name()).collect::<Vec<_>>(),
        vec![std::ffi::OsString::from("snap")],
        "nothing but the volume is left in the worktree"
    );
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "Git no longer lists the worktree");
    assert_eq!(registrations_of_the_repository(repo.as_path()).len(), registrations_before.len().saturating_sub(1), "its registration is gone");
    let _: &Path = volume.as_path();
    remove(&fixture);
}

/// What:
///  A volume several directories deep and a second one beside it both stay,
///  with exactly the directories that lead to them,
///  and their modes are the modes they had,
///  a directory the person made read-only included.
/// Why:
///  The chain is what a person has to move the volume through,
///  and a pass that widened a directory to empty it and never gave the mode back would change
///  state nobody asked it to touch.
#[test]
fn a_volume_several_directories_deep_and_a_second_one_are_both_left_with_their_modes() {
    let fixture: Fixture = fixture("removal-around-deep");
    if !on_btrfs_or_acknowledged(&fixture, "removal-around-deep") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, "a/\nc/\n");
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let target: PathBuf = fixture.root.join("topic");
    age_tracked_files(&fixture, target.as_path());
    let deep: PathBuf = a_volume_at(target.as_path(), "a/b/deep", "deep");
    let second: PathBuf = a_volume_at(target.as_path(), "c/two", "two");
    // The directories that lead to them are emptied of everything else, and one of them is read-only to its owner.
    std::fs::write(target.join("a/b/beside.txt"), b"a file beside the volume\n").expect("a file the removal takes");
    std::fs::write(target.join("c/other.txt"), b"another file the removal takes\n").expect("a file the removal takes");
    std::fs::set_permissions(target.join("a/b").as_path(), std::fs::Permissions::from_mode(0o500)).expect("a read-only directory");
    let deep_before: Vec<String> = tree_of(deep.as_path());
    let second_before: Vec<String> = tree_of(second.as_path());
    let chain_mode: u32 = std::fs::metadata(target.join("a/b").as_path()).expect("the chain").mode() & 0o7777;

    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    assert_the_report_of_a_removal_that_went_around(&removed, target.as_path(), &[deep.as_path(), second.as_path()]);
    assert_eq!(tree_of(deep.as_path()), deep_before, "the deep volume is untouched");
    assert_eq!(tree_of(second.as_path()), second_before, "the second volume is untouched");
    assert_eq!(std::fs::metadata(target.join("a/b").as_path()).expect("the chain").mode() & 0o7777, chain_mode, "the read-only directory keeps its mode");
    assert!(!target.join("a/b/beside.txt").exists(), "the file beside the deep volume was removed");
    assert!(!target.join("c/other.txt").exists(), "the file beside the second volume was removed");
    assert!(target.join("a/b").is_dir() && target.join("a").is_dir() && target.join("c").is_dir(), "the directories that lead to them stay");
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "Git no longer lists the worktree");
    // The control gave that directory its read-only mode, and the fixture is removed with the owner's access to it.
    std::fs::set_permissions(target.join("a/b").as_path(), std::fs::Permissions::from_mode(0o700)).expect("the fixture's removal needs this directory");
    remove(&fixture);
}

/// What:
///  A volume Git lists as untracked keeps a typed removal from happening without `--force`,
///  as real Git refuses a worktree with an untracked file,
///  and `--force` removes everything around it and leaves it exactly as it stood.
/// Why:
///  A typed removal never does more than real Git does without `--force`,
///  and the decision spares an untracked file that is not ignored.
///  The volume is not a file that would go: `--force` keeps it,
///  which is what the refusal says.
#[test]
fn a_volume_the_ignore_rules_do_not_cover_refuses_without_force_and_is_left_with_it() {
    let fixture: Fixture = fixture("removal-around-untracked");
    if !on_btrfs_or_acknowledged(&fixture, "removal-around-untracked") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, "");
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let target: PathBuf = fixture.root.join("topic");
    age_tracked_files(&fixture, target.as_path());
    let volume: PathBuf = a_volume_at(target.as_path(), "snap", "untracked and mine");
    let before: Vec<String> = tree_of(volume.as_path());

    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "a refusal changes nothing: {said}");
    assert!(said.contains("this removal changed nothing"), "the refusal says what the command did: {said}");
    assert!(said.contains(volume.to_str().expect("a UTF-8 fixture path")), "and names the volume: {said}");
    assert!(said.contains("--force"), "and gives the way on: {said}");
    assert!(target.join("tracked.txt").is_file(), "nothing of the worktree was removed");
    assert_eq!(tree_of(volume.as_path()), before, "and nothing of the volume changed");
    assert!(listing(&fixture, repo.as_path()).contains("/topic\n"), "Git still lists the worktree");

    let forced: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "--force", "../topic"]);
    assert_the_report_of_a_removal_that_went_around(&forced, target.as_path(), &[volume.as_path()]);
    assert_eq!(tree_of(volume.as_path()), before, "the volume is exactly what it was");
    assert!(!target.join("tracked.txt").exists(), "every other entry of the worktree was removed");
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "Git no longer lists the worktree");
    remove(&fixture);
}

/// What:
///  A volume that holds a linked worktree of this repository stays,
///  its registration stays,
///  and its branch and its files are still there to work in.
/// Why:
///  A nested worktree of this repository is a subvolume the removal did not make for itself,
///  and the kernel cannot destroy the outer one through it;
///  a removal that deleted the nested worktree's files would take work of the person's with it.
#[test]
fn a_volume_that_holds_a_worktree_of_this_repository_stays_registered_and_usable() {
    let fixture: Fixture = fixture("removal-around-nested-worktree");
    if !on_btrfs_or_acknowledged(&fixture, "removal-around-nested-worktree") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, ".claude/\n");
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "outer", "../outer"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let outer: PathBuf = fixture.root.join("outer");
    age_tracked_files(&fixture, outer.as_path());
    let inner: PathBuf = outer.join(".claude/worktrees/inner");
    std::fs::create_dir_all(inner.parent().expect("the directories the worktree stands in")).expect("directories");
    let made: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "inner", inner.to_str().expect("a UTF-8 fixture path")]);
    assert_eq!(made.code, Some(0), "{}", stderr_of(&made));
    assert!(is_subvolume_root(inner.as_path()).expect("probe"), "the inner worktree is a subvolume");
    std::fs::write(inner.join("notes.txt"), b"what I was in the middle of\n").expect("a file the person wrote");
    let before: Vec<String> = tree_of(inner.as_path());

    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../outer"]);
    assert_the_report_of_a_removal_that_went_around(&removed, outer.as_path(), &[inner.as_path()]);
    assert_eq!(tree_of(inner.as_path()), before, "the nested worktree is untouched");
    let listed: String = listing(&fixture, repo.as_path());
    assert!(listed.contains(inner.to_str().expect("a UTF-8 fixture path")), "Git still lists the nested worktree: {listed}");
    assert!(!listed.contains("/outer\n"), "and no longer lists the worktree that was removed: {listed}");
    let asked: Observed = run_direct(&fixture, inner.as_path(), &["status", "--porcelain"]);
    assert_eq!(asked.code, Some(0), "the nested worktree works: {}", stderr_of(&asked));
    assert!(String::from_utf8_lossy(&asked.stdout).contains("notes.txt"), "and holds the file the person wrote there");
    remove(&fixture);
}

/// What:
///  A mount point inside a worktree is left,
///  what is mounted there is not read and not deleted,
///  and the worktree is removed around it.
/// Why:
///  Git's own removal descends into a mount point and deletes what is mounted there,
///  and nothing in the tree is evidence that the mount is the removal's:
///  this is the case a device comparison alone cannot catch (a bind mount keeps the device).
///  The worktree and the mount are made inside the private namespace,
///  because a removal's cleanliness proof compares each file's owner with the index,
///  and a file written outside the namespace has another owner inside it.
#[test]
fn a_mount_point_inside_a_worktree_is_left_and_what_is_mounted_there_is_untouched() {
    let fixture: Fixture = fixture("removal-around-mount");
    if !a_private_mount_namespace_is_available("removal-around-mount") {
        remove(&fixture);
        return;
    }
    // The mount point is covered by an ignore rule, so Git sees a clean worktree: a mount point Git lists as untracked keeps
    // an unforced removal from happening, as any other untracked entry does.
    let repo: PathBuf = a_repository_with_rules(&fixture, "mnt/\n");
    let target: PathBuf = fixture.root.join("topic");
    let mounted: PathBuf = fixture.root.join("mounted");
    std::fs::create_dir_all(mounted.as_path()).expect("the directory the mount shows");
    std::fs::write(mounted.join("precious.txt"), b"what is mounted there\n").expect("a file the mount shows");
    let before: Vec<String> = tree_of(mounted.as_path());
    let script: String = format!(
        "{real} -C {repo} worktree add --quiet -b topic {target} && mkdir {at} && touch -d '120 seconds ago' {tracked} {rules} && {real} -C {target} update-index --refresh && \
         mount --bind {mounted} {at} && git -C {repo} worktree remove {target}; status=$?; cd / && umount -l {at}; exit $status",
        real = "/usr/bin/git",
        repo = word(repo.as_path()),
        target = word(target.as_path()),
        at = word(target.join("mnt").as_path()),
        tracked = word(target.join("tracked.txt").as_path()),
        rules = word(target.join(".gitignore").as_path()),
        mounted = word(mounted.as_path())
    );
    let removed: Observed = inside_a_private_mount_namespace(&fixture, repo.as_path(), script.as_str());
    assert_the_report_of_a_removal_that_went_around(&removed, target.as_path(), &[target.join("mnt").as_path()]);
    assert_eq!(tree_of(mounted.as_path()), before, "what is mounted there is untouched");
    assert!(target.join("mnt").is_dir(), "the directory the mount stood at is left");
    assert!(!target.join("tracked.txt").exists(), "every other entry of the worktree was removed");
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "Git no longer lists the worktree");
    remove(&fixture);
}

/// What:
///  A mount point that is read-only to everybody is left as it stands.
/// Why:
///  A removal must not need to change what it does not own,
///  and a read-only mount is where a widening attempt would show.
#[test]
fn a_read_only_mount_point_is_left_standingly_untouched() {
    let fixture: Fixture = fixture("removal-around-mount-read-only");
    if !a_private_mount_namespace_is_available("removal-around-mount-read-only") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, "mnt/\n");
    let target: PathBuf = fixture.root.join("topic");
    let mounted: PathBuf = fixture.root.join("mounted");
    std::fs::create_dir_all(mounted.as_path()).expect("the directory the mount shows");
    std::fs::write(mounted.join("precious.txt"), b"read only, and mine\n").expect("a file the mount shows");
    let before: Vec<String> = tree_of(mounted.as_path());
    let script: String = format!(
        "{real} -C {repo} worktree add --quiet -b topic {target} && mkdir {at} && touch -d '120 seconds ago' {tracked} {rules} && {real} -C {target} update-index --refresh && \
         mount --bind {mounted} {at} && mount -o remount,bind,ro {at} && git -C {repo} worktree remove {target}; status=$?; cd / && umount -l {at}; exit $status",
        real = "/usr/bin/git",
        repo = word(repo.as_path()),
        target = word(target.as_path()),
        at = word(target.join("mnt").as_path()),
        tracked = word(target.join("tracked.txt").as_path()),
        rules = word(target.join(".gitignore").as_path()),
        mounted = word(mounted.as_path())
    );
    let removed: Observed = inside_a_private_mount_namespace(&fixture, repo.as_path(), script.as_str());
    assert_the_report_of_a_removal_that_went_around(&removed, target.as_path(), &[target.join("mnt").as_path()]);
    assert_eq!(tree_of(mounted.as_path()), before, "what is mounted there is untouched");
    assert!(target.join("mnt").is_dir(), "the read-only mount point is left where it stood");
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "Git no longer lists the worktree");
    remove(&fixture);
}

/// What:
///  A later `git worktree add` at a path a removal left names the volume,
///  and not only that the directory exists.
/// Why:
///  Git's own sentence for the occupied path says only that it is there,
///  and the person who removed the worktree was told what stayed:
///  the two have to meet, or the path looks broken rather than taken.
#[test]
fn a_later_add_at_the_path_names_the_volume_a_removal_left() {
    let fixture: Fixture = fixture("removal-around-later-add");
    if !on_btrfs_or_acknowledged(&fixture, "removal-around-later-add") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, "snap/\n");
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let target: PathBuf = fixture.root.join("topic");
    age_tracked_files(&fixture, target.as_path());
    let volume: PathBuf = a_volume_at(target.as_path(), "snap", "still mine");
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));

    let again: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "../topic"]);
    let said: String = stderr_of(&again);
    assert_ne!(again.code, Some(0), "the add refuses an occupied path: {said}");
    assert!(said.contains("already exists"), "Git's own sentence is there: {said}");
    assert!(said.contains(volume.to_str().expect("a UTF-8 fixture path")), "and the volume a removal left is named: {said}");
    assert!(said.contains("free for `git worktree add`"), "with what frees the path: {said}");
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "and no worktree was added");
    remove(&fixture);
}

/// What:
///  A clean worktree that holds no volume is removed exactly as it was before this change:
///  the one-call destroy, its notice, no registration and no directory.
/// Why:
///  Every control above would pass just as well if the removal had stopped removing ordinary worktrees.
#[test]
fn an_ordinary_worktree_removal_behaves_as_it_did() {
    let fixture: Fixture = fixture("removal-around-ordinary");
    if !on_btrfs_or_acknowledged(&fixture, "removal-around-ordinary") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, "");
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let target: PathBuf = fixture.root.join("topic");
    age_tracked_files(&fixture, target.as_path());
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    let said: String = stderr_of(&removed);
    assert_eq!(removed.code, Some(0), "the removal finished: {said}");
    assert!(!said.contains("left where it stands"), "nothing was left: {said}");
    assert!(!target.exists(), "the directory is gone");
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "Git no longer lists it");
    assert_eq!(registrations_of_the_repository(repo.as_path()), Vec::<String>::new(), "and its registration is gone");
    if git_policy_cli::fs_mount_options::read_mount_facts(fixture.root.as_path()).ok().flatten().is_some_and(|facts| { return facts.has_option("user_subvol_rm_allowed"); }) {
        assert!(said.contains(DESTROY_NOTICE), "the one-call destroy is stated: {said}");
    }
    remove(&fixture);
}

/// What:
///  The state a command killed inside the pass leaves:
///  the worktree's own tracked entries deleted,
///  the volume standing,
///  the registration kept,
///  and the record the pass wrote before it deleted anything.
/// Why:
///  The review's repro built that state by deleting the files,
/// and a control that wrote the two other parts by hand
///  is the same state without a kill the run cannot time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = killedInThePass(fixture, repo, topic); // record written, tracked files gone
/// ```
fn killed_in_the_pass(fixture: &Fixture, repo: &Path, target: &Path) -> PathBuf {
    let registration: PathBuf = the_registration_of(repo, target);
    let id: std::ffi::OsString = registration.file_name().expect("the registration has a name").to_os_string();
    let admin: String = identity_text(registration.as_path());
    let worktree: String = identity_text(target);
    let folder: PathBuf = repo.join(".git/monochromatic-git-policy-cli/around-removals-begun");
    std::fs::create_dir_all(folder.as_path()).expect("the records' folder");
    let record: PathBuf = folder.join(id);
    let hex: String = hex_of(target);
    let text: String = format!("around-removal-begun=1\nadmin={admin}\nworktree={worktree}\ntargetHex={hex}\n");
    std::fs::write(record.as_path(), text.as_bytes()).expect("the record a killed pass leaves");
    // The pass deletes the worktree's own entries before the registration goes, so these are what it left gone.
    for name in ["tracked.txt", ".gitignore"] {
        let path: PathBuf = target.join(name);
        if path.is_file() {
            std::fs::remove_file(path.as_path()).expect("the entry the pass removed");
        }
    }
    let _ = fixture;
    return record;
}

/// The registration directory of the worktree at `target` in the repository at `repo`.
fn the_registration_of(repo: &Path, target: &Path) -> PathBuf {
    let listed = std::fs::read_dir(repo.join(".git/worktrees")).expect("the registrations");
    for found in listed.flatten() {
        let text: String = std::fs::read_to_string(found.path().join("gitdir")).expect("a registration's back-link");
        if text.trim_end().starts_with(target.join(".git").to_str().expect("a UTF-8 fixture path")) {
            return found.path();
        }
    }
    panic!("the worktree is registered: {}", target.display());
}

/// The identity of a directory as the removal records it.
fn identity_text(path: &Path) -> String {
    let held = git_policy_cli::fs_beneath::open_root(path).expect("the directory opens");
    return git_policy_cli::fs_beneath::identity_of(&held).expect("the directory has an identity").encode();
}

/// A path in the hexadecimal form a record keeps it.
fn hex_of(path: &Path) -> String {
    return git_policy_cli::path_bytes_in_records::hex_of_path(path);
}

/// What:
///  A removal that was cut short inside its pass is finished by the same unforced command,
///  and what was left stands exactly as it was.
/// Why:
///   The pass deletes the worktree's own entries before it forgets the worktree,
/// so a kill in between leaves a tree Git
///  reports as changed;
/// without the record the same command refuses and only `--force` finishes it,
/// which
///  is the promise the stopped report makes and did not keep (the review's finding 2).
#[test]
fn a_removal_cut_short_in_the_pass_is_finished_by_the_same_unforced_command() {
    let fixture: Fixture = fixture("removal-around-resumed");
    if !on_btrfs_or_acknowledged(&fixture, "removal-around-resumed") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, "snap/\n");
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let target: PathBuf = fixture.root.join("topic");
    age_tracked_files(&fixture, target.as_path());
    let volume: PathBuf = a_volume_at(target.as_path(), "snap", "kept through the resume");
    let before: Vec<String> = tree_of(volume.as_path());
    let mode_before: u32 = std::fs::metadata(volume.as_path()).expect("the volume").mode() & 0o7777;
    let record: PathBuf = killed_in_the_pass(&fixture, repo.as_path(), target.as_path());
    assert!(record.is_file(), "the state a killed pass leaves names its record");

    let resumed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    assert_the_report_of_a_removal_that_went_around(&resumed, target.as_path(), &[volume.as_path()]);
    assert_eq!(tree_of(volume.as_path()), before, "the volume is untouched");
    assert_eq!(std::fs::metadata(volume.as_path()).expect("the volume").mode() & 0o7777, mode_before, "and its mode is what it was");
    assert!(!record.exists(), "the record of the pass that finished goes with the removal");
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "Git no longer lists the worktree");
    remove(&fixture);
}

/// What:
///  A worktree whose tracked entries are missing and which has no record of this program says so,
///  names `--force`,
///  and never says nothing was removed.
/// Why:
///   The review's repro of finding 2:
/// a tree with the shape a killed pass leaves and no record is nothing this program can
///  prove it made,
/// and the words "nothing was removed" are false about it.
#[test]
fn a_removal_over_missing_entries_without_a_record_says_what_is_true() {
    let fixture: Fixture = fixture("removal-around-unrecorded");
    if !on_btrfs_or_acknowledged(&fixture, "removal-around-unrecorded") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, "snap/\n");
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let target: PathBuf = fixture.root.join("topic");
    age_tracked_files(&fixture, target.as_path());
    let volume: PathBuf = a_volume_at(target.as_path(), "snap", "still mine");
    let before: Vec<String> = tree_of(volume.as_path());
    // The entries the pass deletes, and no record of a pass: a state this program cannot prove it made.
    for name in ["tracked.txt", ".gitignore"] {
        std::fs::remove_file(target.join(name).as_path()).expect("the entry that is gone");
    }

    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    let said: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "the removal refuses: {said}");
    assert!(said.contains("this removal changed nothing"), "the report says what the command itself did: {said}");
    assert!(!said.contains("nothing was removed"), "and never claims nothing was removed, which the tree makes false: {said}");
    assert!(said.contains("Git reports nothing about the worktree but entries missing from it"), "and names the state it found: {said}");
    assert!(said.contains("No record of this program says a removal of this worktree removed them"), "and that nothing proves the deletions are this program's: {said}");
    assert!(said.contains("git worktree remove --force ../topic"), "and the command that finishes it: {said}");
    assert!(said.contains("Moving what is named above out would not finish it"), "and that moving the volume out cannot help: {said}");
    assert_eq!(tree_of(volume.as_path()), before, "the volume is untouched");
    assert!(listing(&fixture, repo.as_path()).contains("/topic\n"), "Git still lists the worktree");

    let forced: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "--force", "../topic"]);
    assert_the_report_of_a_removal_that_went_around(&forced, target.as_path(), &[volume.as_path()]);
    assert_eq!(tree_of(volume.as_path()), before, "the volume survives the forced finish too");
    remove(&fixture);
}

/// What:
///  A worktree removed in the second it was added,
///  with a volume inside it,
///  goes around the volume:
///  its entries' stat fields prove nothing yet (Git's own racy rule),
///  and Git's own check settles which files they are.
/// Why:
///   The state every person meets who adds a worktree,
/// puts a snapshot inside it and removes it at once refused with
///  exit 1 and said entries were missing from a tree nothing was missing from
/// (an integration pass's section 8,
/// its probe `probe-removal-and-commit-timing.ts`).
///  The documented way is that an unforced removal goes around the volume,
/// so this control ages nothing and rewrites
///  no index:
///  it adds, puts the volume in, and removes.
#[test]
fn a_worktree_removed_in_the_second_it_was_added_goes_around_its_volume() {
    let fixture: Fixture = fixture("removal-around-the-same-second");
    if !on_btrfs_or_acknowledged(&fixture, "removal-around-the-same-second") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = a_repository_with_rules(&fixture, "snap/\n");
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let target: PathBuf = fixture.root.join("topic");
    // Nothing ages the tracked files and no Git command runs in the worktree: its entries' whole-second times are the index's,
    // which is the racy state Git's own rule names.
    let volume: PathBuf = a_volume_at(target.as_path(), "snap", "kept in the same second");
    let before: Vec<String> = tree_of(volume.as_path());

    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    let said: String = stderr_of(&removed);
    assert_eq!(removed.code, Some(0), "the unforced removal goes around the volume: {said}");
    assert!(said.contains("left where it stands"), "and says what it left: {said}");
    assert!(said.contains(volume.to_str().expect("a UTF-8 fixture path")), "naming the volume: {said}");
    assert_eq!(tree_of(volume.as_path()), before, "the volume is untouched");
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "Git no longer lists the worktree");
    remove(&fixture);
}
