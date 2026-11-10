//! Controls for the checks of a commit Git built:
//! the parents and the objects it adds are read from the object store,
//! no
//!  `rev-list` is started for an ordinary signed commit,
//! and every verdict equals the one the `rev-list` gives.

use super::tests::{World, built_request, request, world};
use super::*;
use crate::test_support::{git, git_text, remove};

/// How many times the landing's Git children named `command`,
/// from Git's trace2 start events.
fn starts_of(trace: &Path, command: &str) -> usize {
    let events: String = std::fs::read_to_string(trace).unwrap_or_default();
    let quoted: String = format!("\"{command}\"");
    let found: Vec<&str> = events.lines().filter(|line| { return line.contains("\"event\":\"start\"") && line.contains(quoted.as_str()); }).collect();
    for line in &found {
        eprintln!("started: {}", line.split("\"argv\":").nth(1).unwrap_or(line));
    }
    return found.len();
}

/// A world whose Git children write their trace2 events to the returned file.
fn traced(name: &str) -> (World, PathBuf) {
    let mut world: World = world(name);
    let trace: PathBuf = world.root.join("trace.json");
    world.context.git.overlay.push((std::ffi::OsString::from("GIT_TRACE2_EVENT"), trace.clone().into_os_string()));
    return (world, trace);
}

/// A signed commit has the branch tip as its parent and adds a blob:
/// the landing reads the commit and checks the blob in the
/// object store,
/// so it starts no `rev-list`,
/// and the commit lands as it did.
#[test]
fn an_ordinary_commit_git_built_is_checked_without_starting_rev_list() {
    let (world, trace) = traced("service-built-no-rev-list");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "dir/deeper/typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().unwrap_or_default();
    let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
    assert_eq!(outcomes, vec![Outcome::Landed { commit: commit.clone() }]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), commit);
    assert_eq!(starts_of(trace.as_path(), "rev-list"), 0, "the object store answered both questions");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// Positive control:
/// where the object store cannot show the history (a graft file makes `rev-list` show another one),
/// the
/// landing still starts the `rev-list` it always ran,
/// once for the parents and once for the objects.
#[test]
fn a_repository_with_grafts_is_left_to_rev_list() {
    let (world, trace) = traced("service-built-grafts");
    std::fs::create_dir_all(world.repo.join(".git/info")).expect("info");
    std::fs::write(world.repo.join(".git/info/grafts"), "").expect("an empty graft file");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
    assert!(matches!(outcomes.as_slice(), [Outcome::Landed { .. }]), "{outcomes:?}");
    assert_eq!(starts_of(trace.as_path(), "rev-list"), 2, "the parents and the objects were asked of Git");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// The verdict for each way an object can be gone equals the `rev-list` verdict:
/// the landing is refused as unavailable and the
/// branch stays,
/// whether the missing object is a blob,
/// a tree,
/// a tree below a tree,
/// or is removed after the commit was read.
#[test]
fn every_object_a_built_commit_adds_must_be_there_as_rev_list_demands() {
    let remove_loose = |world: &World, name: &str| {
        std::fs::remove_file(world.repo.join(".git/objects").join(&name[..2]).join(&name[2..])).expect("the object is loose");
    };
    // A blob, a tree and a tree below a tree, each removed before the landing.
    for (label, path, victim) in [("blob", "typed.txt", "typed.txt"), ("tree", "one/typed.txt", "one"), ("nested", "one/two/typed.txt", "one/two")] {
        let (world, trace) = traced(format!("service-built-missing-{label}").as_str());
        let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
        let built: CommitRequest = built_request(&world, "typed", path, &[base.as_str()]);
        let commit: String = built.commit.clone().unwrap_or_default();
        let name: String = git_text(world.repo.as_path(), &["rev-parse", "--verify", format!("{commit}:{victim}").as_str()]);
        remove_loose(&world, name.as_str());
        assert_eq!(land_batch(&world.context, 1, &[built]), Err(FailureCode::Unavailable), "{label}");
        assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "{label}: the branch stays");
        // The blob the request names is found missing before any check of the commit; a tree is found by the check, and what the
        //  object store cannot show there is decided by Git.
        if label != "blob" {
            assert!(starts_of(trace.as_path(), "rev-list") >= 1, "{label}: what the object store cannot show is decided by Git");
        }
        remove(world.root.as_path());
    }
    // An object that goes between the read of the commit and the check of its objects.
    let world: World = world("service-built-between");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().unwrap_or_default();
    let blob: String = git_text(world.repo.as_path(), &["rev-parse", "--verify", format!("{commit}:typed.txt").as_str()]);
    let repo: PathBuf = world.repo.clone();
    crate::service_fault::hooks::set("before-connectivity", std::rc::Rc::new(move || {
        std::fs::remove_file(repo.join(".git/objects").join(&blob[..2]).join(&blob[2..])).expect("the blob is loose");
    }));
    assert_eq!(land_batch(&world.context, 1, &[built]), Err(FailureCode::Unavailable), "between the reads");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch stays");
    remove(world.root.as_path());
}

/// Objects that only a pack holds are present too:
/// a repack that left nothing loose does not make the commit unavailable.
#[test]
fn objects_held_only_by_a_pack_count_as_present() {
    let (world, trace) = traced("service-built-packed");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "one/typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().unwrap_or_default();
    git(world.repo.as_path(), &["update-ref", "refs/keep/typed", commit.as_str()]);
    git(world.repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
    git(world.repo.as_path(), &["prune-packed"]);
    let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
    assert!(matches!(outcomes.as_slice(), [Outcome::Landed { .. }]), "{outcomes:?}");
    assert_eq!(starts_of(trace.as_path(), "rev-list"), 0, "the packs answered");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// A commit with the tip's tree (nothing added) and one that merges another branch are not shapes the store proves:
/// the first is
/// present by construction,
/// the second is left to Git and lands as before.
#[test]
fn an_empty_change_and_a_merge_land_as_before() {
    let (world, trace) = traced("service-built-shapes");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let _unused: CommitRequest = request(&world, "side-file", "side.txt", "side\n");
    git(world.repo.as_path(), &["add", "side.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=side commit"]);
    let side: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["reset", "--quiet", "--hard", base.as_str()]);
    let merge: CommitRequest = built_request(&world, "merge", "merged.txt", &[base.as_str(), side.as_str()]);
    let outcomes = land_batch(&world.context, 1, &[merge]).expect("landed");
    assert!(matches!(outcomes.as_slice(), [Outcome::Landed { .. }]), "{outcomes:?}");
    assert_eq!(starts_of(trace.as_path(), "rev-list"), 1, "the parents were read from the store and the objects of a merge went to Git");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// The proof of a wide directory costs work in proportion to the directory,
/// not to its square:
/// a commit that adds one file to a
/// directory of thousands of files is checked with a few steps per entry.
/// The steps are counted (name comparisons),
/// so the control
/// does not depend on how fast the host is.
#[test]
fn the_proof_of_a_wide_directory_is_linear_in_its_width() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (world, trace) = traced("service-built-wide");
        let width: usize = 3000;
        std::fs::create_dir_all(world.repo.join("wide")).expect("wide");
        for number in 0..width {
            std::fs::write(world.repo.join(format!("wide/file-{number:05}.txt")), format!("{number}\n")).expect("file");
        }
        git(world.repo.as_path(), &["add", "--all"]);
        git(world.repo.as_path(), &["commit", "--quiet", "--message=wide"]);
        let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
        let built: CommitRequest = built_request(&world, "typed", "wide/zz-added.txt", &[base.as_str()]);
        let before: u64 = crate::service_built_checks::name_steps();
        let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
        assert!(matches!(outcomes.as_slice(), [Outcome::Landed { .. }]), "{outcomes:?}");
        let steps: u64 = crate::service_built_checks::name_steps() - before;
        assert_eq!(starts_of(trace.as_path(), "rev-list"), 0, "the object store answered");
        let allowed: u64 = 8 * width as u64;
        assert!(steps > 0, "the proof looked at the names");
        assert!(steps <= allowed, "{steps} name steps for a directory of {width} files; a linear proof needs at most a few per entry ({allowed})");
        crate::service_durable::drain();
        remove(world.root.as_path());
    });
}

/// The store answers only where the history it reads is the history Git shows:
/// every setting that changes what Git shows (a
/// shallow file or graft file elsewhere,
/// another replacement namespace,
/// replacements switched off,
/// `core.useReplaceRefs`,
/// and
/// the global option that switches replacements off) sends the checks to `rev-list`,
/// which the landing always ran.
/// The commit
/// still lands.
/// Each case lands one ordinary commit Git built and counts the `rev-list` starts:
/// two,
/// where an ordinary
/// repository starts none.
#[test]
fn every_setting_that_changes_the_history_git_shows_sends_the_checks_to_rev_list() {
    let variables: [(&str, &str); 4] = [("GIT_SHALLOW_FILE", "elsewhere-shallow"), ("GIT_GRAFT_FILE", "elsewhere-grafts"), ("GIT_REPLACE_REF_BASE", "refs/other-replace/"), ("GIT_NO_REPLACE_OBJECTS", "1")];
    for (name, value) in variables {
        let (mut world, trace) = traced(format!("service-built-env-{}", name.to_lowercase()).as_str());
        world.context.git.overlay.push((std::ffi::OsString::from(name), std::ffi::OsString::from(value)));
        let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
        let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
        let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
        assert!(matches!(outcomes.as_slice(), [Outcome::Landed { .. }]), "{name}: {outcomes:?}");
        assert_eq!(starts_of(trace.as_path(), "rev-list"), 2, "{name}: the checks went to rev-list");
        crate::service_durable::drain();
        remove(world.root.as_path());
    }
}

/// `core.useReplaceRefs` set to anything sends the checks to `rev-list`.
#[test]
fn a_use_replace_refs_setting_sends_the_checks_to_rev_list() {
    let (world, trace) = traced("service-built-use-replace-refs");
    git(world.repo.as_path(), &["config", "core.useReplaceRefs", "false"]);
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    land_batch(&world.context, 1, &[built]).expect("landed");
    assert_eq!(starts_of(trace.as_path(), "rev-list"), 2, "core.useReplaceRefs: the checks went to rev-list");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// The global option that switches replacements off sends the checks to `rev-list`.
#[test]
fn the_no_replace_objects_option_sends_the_checks_to_rev_list() {
    let (mut world, trace) = traced("service-built-no-replace-option");
    world.context.git.global_prefix.push(std::ffi::OsString::from("--no-replace-objects"));
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    land_batch(&world.context, 1, &[built]).expect("landed");
    assert_eq!(starts_of(trace.as_path(), "rev-list"), 2, "--no-replace-objects: the checks went to rev-list");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// A shallow boundary at the default place is left to `rev-list` as well.
#[test]
fn a_shallow_file_sends_the_checks_to_rev_list() {
    let (world, trace) = traced("service-built-shallow");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(world.repo.join(".git/shallow"), format!("{base}\n")).expect("a shallow boundary at the tip");
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
    assert!(matches!(outcomes.as_slice(), [Outcome::Landed { .. }]), "{outcomes:?}");
    assert_eq!(starts_of(trace.as_path(), "rev-list"), 2, "shallow: the checks went to rev-list");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// A replacement object anywhere in the repository is left to `rev-list` too.
#[test]
fn a_replacement_object_sends_the_checks_to_rev_list() {
    let (world, trace) = traced("service-built-replaced");
    let empty: String = git_text(world.repo.as_path(), &["hash-object", "-t", "tree", "-w", "--stdin"]);
    let first: String = git_text(world.repo.as_path(), &["commit-tree", empty.as_str(), "-m", "unrelated one"]);
    let second: String = git_text(world.repo.as_path(), &["commit-tree", empty.as_str(), "-m", "unrelated two"]);
    git(world.repo.as_path(), &["replace", first.as_str(), second.as_str()]);
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
    assert!(matches!(outcomes.as_slice(), [Outcome::Landed { .. }]), "{outcomes:?}");
    assert_eq!(starts_of(trace.as_path(), "rev-list"), 2, "replacement: the checks went to rev-list");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// A world over a SHA-256 repository,
/// which the object names of 64 hexadecimal digits tell from a SHA-1 one.
fn sha256_world(name: &str) -> (World, PathBuf) {
    let root: PathBuf = crate::test_support::fixture(name);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", "--object-format=sha256"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("journal");
    let mut context: LandContext = LandContext { git: crate::test_support::git_context(), journal, on_objects_written: None, lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING };
    let trace: PathBuf = root.join("trace.json");
    context.git.overlay.push((std::ffi::OsString::from("GIT_TRACE2_EVENT"), trace.clone().into_os_string()));
    return (World { root, repo, context }, trace);
}

/// The checks answer from the object store in a SHA-256 repository too,
/// with the objects loose and with them only in a pack,
/// and
/// a missing object is refused there as it is in a SHA-1 one.
#[test]
fn the_checks_of_a_commit_git_built_hold_in_a_sha256_repository() {
    for packed in [false, true] {
        let (world, trace) = sha256_world(format!("service-built-sha256-{packed}").as_str());
        let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
        assert_eq!(base.len(), 64, "a SHA-256 name");
        let built: CommitRequest = built_request(&world, "typed", "one/typed.txt", &[base.as_str()]);
        let commit: String = built.commit.clone().unwrap_or_default();
        if packed {
            git(world.repo.as_path(), &["update-ref", "refs/keep/typed", commit.as_str()]);
            git(world.repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
            git(world.repo.as_path(), &["prune-packed"]);
        }
        let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
        assert_eq!(outcomes, vec![Outcome::Landed { commit: commit.clone() }], "packed: {packed}");
        assert_eq!(starts_of(trace.as_path(), "rev-list"), 0, "packed: {packed}: the object store answered");
        crate::service_durable::drain();
        remove(world.root.as_path());
    }
    let (world, _trace) = sha256_world("service-built-sha256-missing");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "one/typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().unwrap_or_default();
    let tree: String = git_text(world.repo.as_path(), &["rev-parse", "--verify", format!("{commit}:one").as_str()]);
    std::fs::remove_file(world.repo.join(".git/objects").join(&tree[..2]).join(&tree[2..])).expect("the tree is loose");
    assert_eq!(land_batch(&world.context, 1, &[built]), Err(FailureCode::Unavailable));
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch stays");
    remove(world.root.as_path());
}

/// The commit Git built is looked up again when the proof runs:
/// one removed between the read of its parents and the check of its
/// objects is refused as unavailable,
/// the branch stays,
/// and nothing of it is put on the branch.
#[test]
fn a_built_commit_removed_between_the_checks_is_refused() {
    let world: World = world("service-built-commit-between");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().unwrap_or_default();
    let repo: PathBuf = world.repo.clone();
    crate::service_fault::hooks::set("before-connectivity", std::rc::Rc::new(move || {
        std::fs::remove_file(repo.join(".git/objects").join(&commit[..2]).join(&commit[2..])).expect("the commit is loose");
    }));
    assert_eq!(land_batch(&world.context, 1, &[built]), Err(FailureCode::Unavailable));
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch stays");
    remove(world.root.as_path());
}

/// An amend whose commit Git built reads the replaced commit's parents from the store,
/// and its objects are checked by Git (the
/// tip is not its parent):
/// one `rev-list` start,
/// and it lands with the parents of the commit it replaces.
#[test]
fn an_amend_git_built_lands_with_the_parents_of_the_commit_it_replaces() {
    let (world, trace) = traced("service-built-amend");
    let parent: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let _second: CommitRequest = request(&world, "second", "second.txt", "second\n");
    git(world.repo.as_path(), &["add", "second.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let replaced: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let mut built: CommitRequest = built_request(&world, "amended", "amended.txt", &[parent.as_str()]);
    built.amend = true;
    let commit: String = built.commit.clone().unwrap_or_default();
    let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
    assert_eq!(outcomes, vec![Outcome::Landed { commit: commit.clone() }]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), commit);
    assert_ne!(commit, replaced);
    assert_eq!(starts_of(trace.as_path(), "rev-list"), 1, "the replaced commit's parents came from the store; the objects went to Git");
    crate::service_durable::drain();
    remove(world.root.as_path());
}
