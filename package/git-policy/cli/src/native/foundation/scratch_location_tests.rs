//! Controls for the scratch locations,
//!  each in its own disposable directory under the target directory.

use super::*;
use crate::test_support::{fixture, remove};
use crate::time_spent_outside_the_phases::outside_the_phases;
use std::os::unix::fs::PermissionsExt;

/// What:
///  The names `directory` holds,
///  sorted.
/// Why:
///   What a sweep did is the entries it left,
///  so every control of it states that as a list of names.
fn names_in(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .expect("list")
        .map(|entry| { return entry.expect("entry").file_name().to_string_lossy().into_owned(); })
        .collect();
    names.sort();
    return names;
}

/// What:
///  Sweep `directory` until it holds nothing but `wanted`,
///  at most twice per entry that may still go,
///  and answer how many removals finished inside those calls.
/// Why:
///   A sweep removes every stale entry it can and leaves the one it could not to a later sweep:
///  a removal that outlasts its ceiling,
///  a place the process cannot take because a stalled removal holds it,
///  a thread the machine refuses.
///  So how many entries one call removes is not a promise the product makes,
///  and a control that asserted it failed in a gate whose disk was jammed
///  (`doc/planning/cli-git-scratch-locations.md`,
///  "What one sweep promises").
///  What the product promises is which entries remain,
///  which is `wanted`,
///  and the figure this answers is reported in that assertion's message and never asserted,
///  because a removal cut short still finishes on its thread while the later sweeps run.
fn sweep_until(directory: &Path, now: SystemTime, ceiling: Duration, wanted: &[&str]) -> usize {
    let mut removed: usize = 0;
    let attempts: usize = names_in(directory).len().saturating_mul(2).saturating_add(1);
    for attempt in 0..attempts {
        if names_in(directory) == wanted {
            break;
        }
        let finished_before: usize = removed;
        removed += sweep_stale(directory, now, SCRATCH_STALE_AFTER, ceiling);
        if removed == finished_before && attempt + 1 < attempts {
            // Nothing finished inside that call, so a removal it started is still on its thread: the next sweep waits for it.
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    return removed;
}

/// An environment of the given pairs.
fn environment(pairs: &[(&str, &Path)]) -> Vec<(OsString, OsString)> {
    return pairs
        .iter()
        .map(|(name, value)| { return (OsString::from(name), value.as_os_str().to_os_string()); })
        .collect();
}

/// The permission bits of a path without following a final link.
fn mode_of(path: &Path) -> u32 {
    return std::fs::symlink_metadata(path).expect("metadata").permissions().mode() & 0o7777;
}

/// Set a file or directory's modification time to `back` before now.
fn age(path: &Path, back: Duration) {
    std::fs::File::open(path)
        .expect("open")
        .set_modified(SystemTime::now() - back)
        .expect("set modification time");
}

/// An absolute `XDG_STATE_HOME` wins;
/// a relative one is ignored as the XDG rules say;
/// `HOME` is the fallback.
#[test]
fn the_state_directory_follows_the_xdg_rules() {
    assert_eq!(
        state_directory(&environment(&[("XDG_STATE_HOME", Path::new("/state")), ("HOME", Path::new("/home/u"))])),
        Some(PathBuf::from("/state/monochromatic-git-policy-cli"))
    );
    assert_eq!(
        state_directory(&environment(&[("XDG_STATE_HOME", Path::new("relative")), ("HOME", Path::new("/home/u"))])),
        Some(PathBuf::from("/home/u/.local/state/monochromatic-git-policy-cli"))
    );
    assert_eq!(
        state_directory(&environment(&[("HOME", Path::new("/home/u"))])),
        Some(PathBuf::from("/home/u/.local/state/monochromatic-git-policy-cli"))
    );
}

/// Without an absolute `XDG_STATE_HOME` and without a non-empty `HOME` there is no directory,
/// and the sentence names
/// both variables and says the system temporary directory is not a fallback.
#[test]
fn a_missing_home_is_named_and_never_falls_back_to_the_system_directory() {
    assert_eq!(state_directory(&[]), None);
    assert_eq!(state_directory(&environment(&[("HOME", Path::new(""))])), None);
    let reason: String = user_scratch_path(&[]).expect_err("no home");
    assert!(reason.contains("XDG_STATE_HOME") && reason.contains("HOME"), "{reason}");
    assert!(reason.contains("never in the system temporary directory"), "{reason}");
    assert_eq!(prepare_user_scratch(&[]).expect_err("no home"), reason);
}

/// The per-user scratch directory is made with every missing parent at mode `0700`,
/// and a second preparation accepts it.
#[test]
fn the_user_scratch_directory_is_created_private_with_its_parents() {
    let root: PathBuf = fixture("scratch-user-create");
    let home: PathBuf = root.join("home");
    std::fs::create_dir(&home).expect("home");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    let scratch: PathBuf = prepare_user_scratch(env.as_slice()).expect("first preparation");
    assert_eq!(scratch, home.join(".local/state/monochromatic-git-policy-cli/scratch"));
    for directory in [".local", ".local/state", ".local/state/monochromatic-git-policy-cli", ".local/state/monochromatic-git-policy-cli/scratch"] {
        assert_eq!(mode_of(home.join(directory).as_path()), 0o700, "{directory}");
    }
    assert_eq!(prepare_user_scratch(env.as_slice()).expect("second preparation"), scratch);
    remove(root.as_path());
}

/// A scratch path that is a link is refused and never followed,
/// so no write can be redirected through it.
#[test]
fn a_linked_scratch_directory_is_refused() {
    let root: PathBuf = fixture("scratch-user-link");
    let home: PathBuf = root.join("home");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(home.join(".local/state/monochromatic-git-policy-cli")).expect("state");
    std::fs::create_dir(&elsewhere).expect("elsewhere");
    std::os::unix::fs::symlink(&elsewhere, home.join(".local/state/monochromatic-git-policy-cli/scratch")).expect("plant link");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    let reason: String = prepare_user_scratch(env.as_slice()).expect_err("a link is refused");
    assert!(reason.contains("not a directory"), "{reason}");
    let repository_link: PathBuf = root.join("common");
    std::fs::create_dir(&repository_link).expect("common");
    std::os::unix::fs::symlink(&elsewhere, repository_link.join(REPOSITORY_SCRATCH_NAME)).expect("plant link");
    assert!(prepare_repository_scratch(repository_link.as_path()).is_err());
    assert_eq!(std::fs::read_dir(&elsewhere).expect("elsewhere").count(), 0);
    remove(root.as_path());
}

/// An existing directory of this user with a looser mode is tightened to `0700`.
#[test]
fn a_loose_scratch_directory_of_this_user_is_tightened() {
    let root: PathBuf = fixture("scratch-user-loose");
    let home: PathBuf = root.join("home");
    let scratch: PathBuf = home.join(".local/state/monochromatic-git-policy-cli/scratch");
    std::fs::create_dir_all(&scratch).expect("scratch");
    std::fs::set_permissions(&scratch, std::fs::Permissions::from_mode(0o755)).expect("loosen");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    prepare_user_scratch(env.as_slice()).expect("preparation");
    assert_eq!(mode_of(scratch.as_path()), 0o700);
    remove(root.as_path());
}

/// The repository scratch directory is made inside the common directory at mode `0700` and accepted again.
#[test]
fn the_repository_scratch_directory_is_created_private_beside_the_common_directory_entries() {
    let root: PathBuf = fixture("scratch-repository-create");
    let first: PathBuf = prepare_repository_scratch(root.as_path()).expect("first preparation");
    assert_eq!(first, root.join("monochromatic-git-policy-cli-scratch"));
    assert_eq!(mode_of(first.as_path()), 0o700);
    assert_eq!(prepare_repository_scratch(root.as_path()).expect("second preparation"), first);
    let missing: PathBuf = root.join("no-such-common-directory");
    assert!(prepare_repository_scratch(missing.as_path()).is_err());
    remove(root.as_path());
}

/// The sweep removes files,
/// directories with contents and links older than the bound,
/// never follows a link,
/// and keeps
/// fresh entries and entries dated in the future.
#[test]
fn the_sweep_removes_only_stale_entries_and_never_follows_links() {
    let root: PathBuf = fixture("scratch-sweep");
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    let outside: PathBuf = root.join("outside.txt");
    std::fs::write(&outside, b"outside").expect("outside file");
    age(outside.as_path(), SCRATCH_STALE_AFTER * 3);
    std::fs::write(scratch.join("cli-git-old-file"), b"old").expect("old file");
    std::fs::create_dir(scratch.join("cli-git-old-directory")).expect("old directory");
    std::fs::write(scratch.join("cli-git-old-directory/inner"), b"inner").expect("inner");
    std::fs::write(scratch.join("cli-git-fresh"), b"fresh").expect("fresh file");
    std::fs::write(scratch.join("cli-git-future"), b"future").expect("future file");
    std::os::unix::fs::symlink(&outside, scratch.join("cli-git-old-link")).expect("link");
    let long_ago: SystemTime = SystemTime::now() - SCRATCH_STALE_AFTER * 2;
    std::fs::File::open(scratch.join("cli-git-old-file")).expect("open").set_modified(long_ago).expect("age");
    std::fs::File::open(scratch.join("cli-git-old-directory/inner")).expect("open").set_modified(long_ago).expect("age");
    std::fs::File::open(scratch.join("cli-git-old-directory")).expect("open").set_modified(long_ago).expect("age");
    std::fs::File::open(scratch.join("cli-git-future"))
        .expect("open")
        .set_modified(SystemTime::now() + SCRATCH_STALE_AFTER * 2)
        .expect("future");
    // The link itself is fresh, so the first sweep keeps it: only its own time counts, never its target's.
    let removed: usize = sweep_until(scratch.as_path(), SystemTime::now(), SCRATCH_REMOVAL_CEILING, &["cli-git-fresh", "cli-git-future", "cli-git-old-link"]);
    assert_eq!(names_in(scratch.as_path()), ["cli-git-fresh", "cli-git-future", "cli-git-old-link"], "the stale file and the stale directory went, and the fresh entry, the entry dated in the future and the link stayed ({removed} removals finished inside a sweep)");
    // Two days later the fresh file and the link are stale: both go, and the file the link points at stays.
    let later: SystemTime = SystemTime::now() + SCRATCH_STALE_AFTER * 2;
    let removed_later: usize = sweep_until(scratch.as_path(), later, SCRATCH_REMOVAL_CEILING, &["cli-git-future"]);
    assert_eq!(names_in(scratch.as_path()), ["cli-git-future"], "the file that was fresh and the link went, and the entry dated in the future stayed ({removed_later} removals finished inside a sweep)");
    assert!(std::fs::symlink_metadata(scratch.join("cli-git-old-link")).is_err(), "the link is gone");
    assert_eq!(std::fs::read(&outside).expect("target"), b"outside");
    remove(root.as_path());
}

/// A sweep keeps a commit tree that has not been abandoned,
/// however its owner looks,
/// because a landing that a killed client left queued is still reading it;
/// and a tree nothing has written for the abandonment age goes,
/// long before the sweep's own day has passed.
#[test]
fn a_sweep_keeps_a_tree_that_may_still_be_read_and_takes_one_that_is_abandoned() {
    let root: PathBuf = fixture("scratch-sweep-owner");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let scratch: PathBuf = common.join(REPOSITORY_SCRATCH_NAME);
    std::fs::create_dir_all(&scratch).expect("scratch directory");
    let read_by_a_landing: PathBuf = scratch.join("monochromatic-git-policy-cli-service-recent1");
    std::fs::create_dir_all(&read_by_a_landing).expect("a tree a landing is reading");
    std::fs::write(read_by_a_landing.join("commit.index"), b"index").expect("a file in the tree");
    // The client that made this tree was killed a moment ago, and nothing a marker says can take the tree from its landing.
    let gone: String = format!(
        "{}{}0 0 1.1:0\n",
        crate::service_index_marker::LOCK_MARKER,
        crate::service_index_marker::MARKER_VERSION
    );
    std::fs::write(read_by_a_landing.join("owner"), gone.as_bytes()).expect("a marker naming a process that is gone");
    age(read_by_a_landing.as_path(), Duration::from_secs(60 * 60));
    let abandoned: PathBuf = scratch.join("monochromatic-git-policy-cli-service-gone1");
    std::fs::create_dir_all(&abandoned).expect("a tree no command writes any more");
    age(abandoned.as_path(), crate::service_scratch_abandonment::SCRATCH_TREE_ABANDONED_AFTER + Duration::from_secs(60));
    let removed: usize = sweep_until(scratch.as_path(), SystemTime::now(), SCRATCH_REMOVAL_CEILING, &["monochromatic-git-policy-cli-service-recent1"]);
    assert!(read_by_a_landing.exists(), "a tree written an hour ago may still be read by a landing");
    assert!(!abandoned.exists(), "a tree nothing has written for the abandonment age goes, long before the day, and a sweep that could not remove it inside its ceiling leaves it to the next one ({removed} removals finished inside a sweep)");
    remove(root.as_path());
}

/// A missing directory sweeps nothing and does not fail.
#[test]
fn sweeping_a_missing_directory_removes_nothing() {
    let root: PathBuf = fixture("scratch-sweep-missing");
    assert_eq!(sweep_stale(root.join("missing").as_path(), SystemTime::now(), SCRATCH_STALE_AFTER, SCRATCH_REMOVAL_CEILING), 0);
    remove(root.as_path());
}

/// What:
///  A commit's own scratch tree is made without collecting the repository's stale entries.
/// Why:
///   A commit never reads the answer:
/// it makes its tree and goes,
/// so a listing,
/// one question to the file system per
///  entry and a removal are free work on its path,
/// and on a volume whose calls stall they were 13,482 ms of the 48
///  commits of 2026-11-11T12:33:40Z to 13:15:27Z and 4,775 ms in one
///  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
/// "The causes,
/// ranked by the time they cost").
///  The resident service collects beside its landing loop instead
///  (`service_repository_scratch_sweep.rs`),
/// so nothing a killed command left stays.
#[test]
fn a_commits_scratch_tree_collects_nothing_on_the_commits_own_path() {
    let root: PathBuf = fixture("scratch-commit-without-a-sweep");
    let common: PathBuf = root.join("common");
    let scratch: PathBuf = common.join(REPOSITORY_SCRATCH_NAME);
    std::fs::create_dir_all(&scratch).expect("repository scratch");
    protect_path(scratch.as_path(), PRIVATE_DIRECTORY_MODE).expect("private");
    std::fs::write(scratch.join("cli-git-input-stale"), b"x").expect("a stale entry a killed command left");
    age(scratch.join("cli-git-input-stale").as_path(), SCRATCH_STALE_AFTER * 2);
    let tree: PathBuf = prepare_commit_scratch(common.as_path(), common.as_path(), "monochromatic-git-policy-cli-service-0a1b2c3d-0", SCRATCH_REMOVAL_CEILING)
        .expect("a commit's scratch tree");
    assert!(tree.is_dir(), "the tree the commit writes into is made");
    assert!(
        scratch.join("cli-git-input-stale").exists(),
        "the commit's own path collects nothing, so the stale entry is still there for the service's collection"
    );
    remove(root.as_path());
}

/// Both preparations collect what a killed process left,
/// so stale files cannot accumulate without bound.
#[test]
fn preparing_a_directory_collects_what_a_killed_process_left() {
    let root: PathBuf = fixture("scratch-prepare-sweep");
    let home: PathBuf = root.join("home");
    let scratch: PathBuf = home.join(".local/state/monochromatic-git-policy-cli/scratch");
    std::fs::create_dir_all(&scratch).expect("scratch");
    std::fs::write(scratch.join("cli-git-markdown-stale.jsonc"), b"[]").expect("stale");
    age(scratch.join("cli-git-markdown-stale.jsonc").as_path(), SCRATCH_STALE_AFTER * 2);
    std::fs::write(scratch.join("cli-git-markdown-live.jsonc"), b"[]").expect("live");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    prepare_user_scratch(env.as_slice()).expect("preparation");
    assert!(!scratch.join("cli-git-markdown-stale.jsonc").exists());
    assert!(scratch.join("cli-git-markdown-live.jsonc").exists());
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.join(REPOSITORY_SCRATCH_NAME)).expect("repository scratch");
    std::fs::write(common.join("monochromatic-git-policy-cli-scratch/cli-git-input-stale"), b"x").expect("stale");
    age(common.join("monochromatic-git-policy-cli-scratch/cli-git-input-stale").as_path(), SCRATCH_STALE_AFTER * 2);
    prepare_repository_scratch(common.as_path()).expect("preparation");
    assert!(!common.join("monochromatic-git-policy-cli-scratch/cli-git-input-stale").exists());
    remove(root.as_path());
}

/// The scratch folder an earlier release made in a Git directory (`cli-git-scratch`) is recognized and swept under the same rules
///  as this release's own:
/// entries older than the bound go (files and directories with their contents),
/// fresh entries and entries dated in the future stay,
/// and the folder itself stays.
#[test]
fn an_earlier_releases_repository_scratch_is_swept_by_the_rules_of_its_own() {
    let root: PathBuf = fixture("scratch-earlier-sweep");
    let common: PathBuf = root.join("common");
    let earlier: PathBuf = common.join(REPOSITORY_SCRATCH.earlier);
    std::fs::create_dir_all(&earlier).expect("the earlier scratch");
    protect_path(earlier.as_path(), PRIVATE_DIRECTORY_MODE).expect("private");
    std::fs::write(earlier.join("cli-git-input-stale"), b"x").expect("stale file");
    std::fs::create_dir_all(earlier.join("cli-git-service-stale/inner")).expect("stale directory");
    std::fs::write(earlier.join("cli-git-service-stale/inner/file"), b"x").expect("inner file");
    std::fs::write(earlier.join("cli-git-input-fresh"), b"x").expect("fresh file");
    std::fs::write(earlier.join("cli-git-input-future"), b"x").expect("future file");
    for stale in ["cli-git-input-stale", "cli-git-service-stale"] {
        age(earlier.join(stale).as_path(), SCRATCH_STALE_AFTER * 2);
    }
    std::fs::File::open(earlier.join("cli-git-input-future")).expect("open").set_modified(SystemTime::now() + SCRATCH_STALE_AFTER * 2).expect("future time");
    prepare_repository_scratch(common.as_path()).expect("preparation");
    assert!(!earlier.join("cli-git-input-stale").exists(), "a stale file of the earlier release goes");
    assert!(!earlier.join("cli-git-service-stale").exists(), "and a stale directory with its contents");
    assert!(earlier.join("cli-git-input-fresh").exists(), "a fresh entry stays");
    assert!(earlier.join("cli-git-input-future").exists(), "an entry dated in the future stays");
    assert!(earlier.is_dir(), "the folder itself stays");
    assert!(prepare_repository_scratch(common.as_path()).expect("second preparation").ends_with(REPOSITORY_SCRATCH.current));
    remove(root.as_path());
}

/// The earlier folder is not made when it is not there,
/// and is kept where this release's own would be refused or kept:
/// a link in
///  its place is not followed and nothing behind it is touched,
/// and a loose mode of a folder this user owns is tightened.
#[test]
fn an_earlier_releases_repository_scratch_is_never_made_and_is_kept_where_the_rules_keep_their_own() {
    let root: PathBuf = fixture("scratch-earlier-kept");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(&common).expect("common directory");
    prepare_repository_scratch(common.as_path()).expect("preparation");
    assert!(!common.join(REPOSITORY_SCRATCH.earlier).exists(), "nothing is made under the earlier name");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    std::fs::write(elsewhere.join("old-file"), b"keep").expect("file");
    age(elsewhere.join("old-file").as_path(), SCRATCH_STALE_AFTER * 2);
    std::os::unix::fs::symlink(&elsewhere, common.join(REPOSITORY_SCRATCH.earlier)).expect("a link in place of the folder");
    prepare_repository_scratch(common.as_path()).expect("preparation");
    assert!(elsewhere.join("old-file").exists(), "nothing behind a link is swept");
    std::fs::remove_file(common.join(REPOSITORY_SCRATCH.earlier)).expect("link removed");
    let loose: PathBuf = common.join(REPOSITORY_SCRATCH.earlier);
    std::fs::create_dir_all(&loose).expect("loose folder");
    std::fs::set_permissions(&loose, std::fs::Permissions::from_mode(0o755)).expect("loose mode");
    std::fs::write(loose.join("stale"), b"x").expect("stale");
    age(loose.join("stale").as_path(), SCRATCH_STALE_AFTER * 2);
    prepare_repository_scratch(common.as_path()).expect("preparation");
    assert_eq!(mode_of(loose.as_path()), 0o700, "tightened as the own folder is");
    assert!(!loose.join("stale").exists(), "and swept");
    remove(root.as_path());
}

/// An anonymous scratch file is read and written,
/// has no name in the per-user scratch directory,
/// and without a home
/// the failure names the variables instead of falling back to the system temporary directory.
#[test]
fn a_scratch_file_has_no_name_and_needs_a_home() {
    use std::io::{Read, Seek, SeekFrom, Write};
    let root: PathBuf = fixture("scratch-file");
    let home: PathBuf = root.join("home");
    std::fs::create_dir(&home).expect("home");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    let mut file: std::fs::File = scratch_file(env.as_slice()).expect("scratch file");
    file.write_all(b"bytes").expect("write");
    file.seek(SeekFrom::Start(0)).expect("seek");
    let mut read: Vec<u8> = Vec::new();
    file.read_to_end(&mut read).expect("read");
    assert_eq!(read, b"bytes");
    let scratch: PathBuf = home.join(".local/state/monochromatic-git-policy-cli/scratch");
    assert_eq!(std::fs::read_dir(&scratch).expect("scratch directory").count(), 0, "the file has no name");
    let error: std::io::Error = scratch_file(&[]).expect_err("no home");
    assert!(error.to_string().contains("never in the system temporary directory"), "{error}");
    remove(root.as_path());
}

/// Every write to a scratch file lands at its end,
/// whatever offset the writers think they have:
/// the child that gets a clone of the
/// file and a helper it starts share one offset,
/// and a write at a stale position overwrites a line the other wrote (measured with
/// a leader and a helper writing to one shared file:
/// the leader's line was lost in 5 of 150 runs at a 20% CPU quota,
/// in none of 300
/// once the file was in append mode).
/// The control makes the stale position certain:
/// a child writes,
/// the file is emptied,
/// the
/// child writes again,
/// and the second write is the whole of what is there,
/// not a line after a hole of zero bytes.
#[test]
fn a_write_to_a_scratch_file_after_it_was_emptied_lands_at_the_start_and_leaves_no_hole() {
    use std::io::{Read, Seek, SeekFrom};
    let root: PathBuf = fixture("scratch-file-append");
    let home: PathBuf = root.join("home");
    std::fs::create_dir(&home).expect("home");
    let go: PathBuf = root.join("go");
    let env: Vec<(OsString, OsString)> = environment(&[("HOME", home.as_path())]);
    let mut file: std::fs::File = scratch_file(env.as_slice()).expect("scratch file");
    let for_child: std::fs::File = file.try_clone().expect("a clone for the child");
    // The wait ends when the fixture goes or this test process ends, and the child's own group is ended with the test thread:
    //  a loop that waits for a release a failed test never writes would otherwise run on in a removed fixture.
    let script: String = format!(
        "echo first >&2; while [ ! -e '{}' ] && {}; do sleep 0.01; done; echo second >&2",
        go.display(),
        crate::test_support::while_test_lives(root.as_path())
    );
    let mut command: std::process::Command = std::process::Command::new("sh");
    command.args(["-c", script.as_str()]).current_dir(root.as_path()).stderr(for_child);
    let mut child = crate::test_support::spawn_owned(&mut command).expect("the child");
    // The child writes to its standard error, which is the file; wait until its first line is there.
    let began: std::time::Instant = std::time::Instant::now();
    while file.metadata().expect("metadata").len() == 0 {
        assert!(began.elapsed() < Duration::from_secs(60), "the child never wrote its first line");
        std::thread::sleep(Duration::from_millis(5));
    }
    file.set_len(0).expect("empty the file");
    std::fs::write(&go, b"").expect("the signal to go on");
    assert!(child.wait().expect("the child ends").success());
    file.seek(SeekFrom::Start(0)).expect("seek");
    let mut read: Vec<u8> = Vec::new();
    file.read_to_end(&mut read).expect("read");
    assert_eq!(read, b"second\n", "the second write is the whole file");
    remove(root.as_path());
}

/// The staleness bound outlasts every Git child step the wrapper runs:
/// a live process must never lose its file.
#[test]
fn the_staleness_bound_outlasts_the_longest_wait() {
    for class in [crate::child_class::ChildClass::Plumbing, crate::child_class::ChildClass::Object] {
        assert!(SCRATCH_STALE_AFTER > class.default_limit(), "{class:?}");
    }
}

/// An entry exactly as old as the bound is kept;
/// one a second older goes.
#[test]
fn the_sweep_keeps_an_entry_exactly_at_the_bound_and_removes_one_past_it() {
    let root: PathBuf = fixture("scratch-sweep-boundary");
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    let entry: PathBuf = scratch.join("cli-git-boundary");
    std::fs::write(&entry, b"x").expect("entry");
    let modified: SystemTime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    std::fs::File::open(&entry).expect("open").set_modified(modified).expect("set modification time");
    let at_bound: SystemTime = modified + SCRATCH_STALE_AFTER;
    assert_eq!(sweep_stale(scratch.as_path(), at_bound, SCRATCH_STALE_AFTER, SCRATCH_REMOVAL_CEILING), 0);
    assert!(entry.exists(), "an entry exactly at the bound is kept");
    let past_bound: SystemTime = at_bound + Duration::from_secs(1);
    let removed: usize = sweep_until(scratch.as_path(), past_bound, SCRATCH_REMOVAL_CEILING, &[]);
    assert!(!entry.exists(), "an entry past the bound is removed ({removed} removals finished inside a sweep)");
    remove(root.as_path());
}

/// A sweep removes a stale entry within the ceiling it is given,
/// and one whose removal cannot finish inside that ceiling is reported and left for a later sweep,
/// never waited for:
/// the caller is a commit,
/// and an entry another command left must not hold it.
#[test]
fn a_sweep_removes_within_its_ceiling_and_leaves_what_it_cannot() {
    let root: PathBuf = fixture("scratch-sweep-bounded");
    let scratch: PathBuf = root.join("scratch");
    let tree: PathBuf = scratch.join("cli-git-many");
    std::fs::create_dir_all(&tree).expect("tree");
    for at in 0..500 {
        std::fs::write(tree.join(format!("entry-{at}")), b"x").expect("an entry");
    }
    age(tree.as_path(), SCRATCH_STALE_AFTER * 2);
    let before: u64 = outside_the_phases().removals_that_did_not_finish;
    let removed: usize = sweep_stale(scratch.as_path(), SystemTime::now(), SCRATCH_STALE_AFTER, Duration::from_micros(1));
    assert_eq!(removed, 0, "an entry whose removal outlasts the ceiling is not counted as removed");
    assert_eq!(outside_the_phases().removals_that_did_not_finish, before + 1, "the removal went through the bounded one and said so");
    assert!(tree.exists(), "what the ceiling cut short stays for a later sweep");
    // A later sweep, given the ceiling a caller really gives it, removes an entry it finds stale.
    let second: PathBuf = root.join("second-scratch");
    let later: PathBuf = second.join("cli-git-later");
    std::fs::create_dir_all(&later).expect("later tree");
    std::fs::write(later.join("entry"), b"x").expect("an entry");
    age(later.as_path(), SCRATCH_STALE_AFTER * 2);
    let later_removed: usize = sweep_until(second.as_path(), SystemTime::now(), SCRATCH_REMOVAL_CEILING, &[]);
    assert!(!later.exists(), "a stale entry goes within the ceiling ({later_removed} removals finished inside a sweep)");
    remove(root.as_path());
}

/// A sweep whose every call is cut short at its ceiling still empties the stale entries over later sweeps:
/// what it promises is which entries remain,
/// never how many one call removes,
/// so a control that required that figure would fail whenever the device stalls
/// or a stalled removal holds the process's place for removals.
#[test]
fn a_sweep_cut_short_empties_the_stale_entries_over_later_sweeps() {
    let root: PathBuf = fixture("scratch-sweep-cut-short");
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    for at in 0..3 {
        let tree: PathBuf = scratch.join(format!("cli-git-tree-{at}"));
        std::fs::create_dir(&tree).expect("tree");
        std::fs::write(tree.join("entry"), b"x").expect("an entry");
        age(tree.as_path(), SCRATCH_STALE_AFTER * 2);
    }
    let before: u64 = outside_the_phases().removals_that_did_not_finish;
    let removed: usize = sweep_until(scratch.as_path(), SystemTime::now(), Duration::from_micros(1), &[]);
    assert!(names_in(scratch.as_path()).is_empty(), "every stale tree went over the later sweeps, though only {removed} of their removals finished inside a sweep");
    assert!(outside_the_phases().removals_that_did_not_finish > before, "the removals a one-microsecond ceiling cut short were reported as not finished");
    remove(root.as_path());
}

/// A creation failure other than "already exists" is returned as it is,
/// not hidden behind the later privacy check.
#[test]
fn a_refused_creation_surfaces_its_own_error() {
    let root: PathBuf = fixture("scratch-repository-refused");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(&common).expect("common");
    std::fs::set_permissions(&common, std::fs::Permissions::from_mode(0o500)).expect("seal");
    let writable_anyway: bool = std::fs::create_dir(common.join("probe")).is_ok();
    let outcome: std::io::Result<PathBuf> = prepare_repository_scratch(common.as_path());
    std::fs::set_permissions(&common, std::fs::Permissions::from_mode(0o700)).expect("unseal");
    // A process that ignores permission bits (root) cannot be refused, so only a refusing process proves the error.
    if !writable_anyway {
        let error: std::io::Error = outcome.expect_err("creation is refused");
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied, "{error}");
    }
    remove(root.as_path());
}

/// A directory with no group or other access is accepted as it is:
/// only a looser mode is rewritten,
/// so its sticky bit
/// stays,
/// which a mode rewrite would clear.
#[test]
fn a_private_directory_is_left_untouched() {
    let root: PathBuf = fixture("scratch-private-untouched");
    let common: PathBuf = root.join("common");
    let scratch: PathBuf = common.join(REPOSITORY_SCRATCH_NAME);
    std::fs::create_dir_all(&scratch).expect("scratch");
    std::fs::set_permissions(&scratch, std::fs::Permissions::from_mode(0o1700)).expect("sticky");
    assert_eq!(mode_of(scratch.as_path()), 0o1700, "the fixture is sticky");
    prepare_repository_scratch(common.as_path()).expect("preparation");
    assert_eq!(mode_of(scratch.as_path()), 0o1700);
    remove(root.as_path());
}

/// A directory whose modification time is renewed is not swept however old it was,
/// and one that is not renewed is.
#[test]
fn a_renewed_entry_survives_the_sweep_and_an_unrenewed_one_does_not() {
    let root: std::path::PathBuf = fixture("scratch-renew");
    let kept: std::path::PathBuf = root.join("kept");
    let left: std::path::PathBuf = root.join("left");
    std::fs::create_dir_all(&kept).expect("kept");
    std::fs::create_dir_all(&left).expect("left");
    let long_ago: SystemTime = SystemTime::now() - SCRATCH_STALE_AFTER * 2;
    for directory in [&kept, &left] {
        std::fs::File::open(directory).expect("open").set_modified(long_ago).expect("age");
    }
    renew(kept.as_path());
    let removed: usize = sweep_until(root.as_path(), SystemTime::now(), SCRATCH_REMOVAL_CEILING, &["kept"]);
    assert!(kept.exists() && !left.exists(), "the renewed entry stays and the one nobody renewed goes ({removed} removals finished inside a sweep)");
    remove(root.as_path());
}

/// The renewal comes many times in the stale age,
/// so a sweep at any moment finds a waiter's entry well inside it.
#[test]
fn the_renewal_pace_is_far_inside_the_stale_age() {
    assert!(SCRATCH_REFRESH_EVERY * 2 < SCRATCH_STALE_AFTER, "{SCRATCH_REFRESH_EVERY:?}");
}

/// A commit's scratch tree is made inside the repository scratch directory,
/// which is the place the sweep covers,
/// and it is private.
#[test]
fn a_commits_scratch_tree_is_made_where_the_sweep_covers_it() {
    let root: PathBuf = fixture("scratch-commit-tree");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let named: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let tree: PathBuf = prepare_commit_scratch(common.as_path(), common.as_path(), named, SCRATCH_REMOVAL_CEILING).expect("the scratch tree");
    assert_eq!(tree, common.join(REPOSITORY_SCRATCH_NAME).join(named));
    assert_eq!(mode_of(tree.as_path()), 0o700);
    remove(root.as_path());
}

/// A leftover of the command's own earlier attempt whose removal its ceiling cut short does not fail the next attempt:
///  the leftover is moved aside under a name no commit tree has,
///  the attempt starts from an empty tree,
///  and the sweep of a later command collects what was moved.
///  The removal of the moved tree runs beside this call,
///  so whether any of it is still there when the assertion below runs is the device's to say,
///  and what is asserted is that nothing but such a moved tree is left beside the attempt's own.
#[test]
fn a_leftover_its_removal_cut_short_is_moved_aside_and_the_attempt_goes_on() {
    let root: PathBuf = fixture("scratch-leftover-aside");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let first: PathBuf = prepare_commit_scratch(common.as_path(), common.as_path(), name, SCRATCH_REMOVAL_CEILING).expect("the first attempt's tree");
    for at in 0..500 {
        std::fs::write(first.join(format!("entry-{at}")), b"x").expect("a file of the first attempt");
    }
    // The ceiling a stalled device gives the removal: the leftover is still there when the next attempt begins.
    let again: PathBuf = prepare_commit_scratch(common.as_path(), common.as_path(), name, Duration::from_micros(1)).expect("the next attempt's tree");
    assert_eq!(again, first, "the attempt keeps the name of its own tree");
    assert!(!again.join("entry-0").exists(), "the attempt starts from an empty tree");
    let scratch: PathBuf = common.join(REPOSITORY_SCRATCH_NAME);
    let beside: Vec<PathBuf> = std::fs::read_dir(&scratch)
        .expect("scratch directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path != &again; })
        .collect();
    for leftover in &beside {
        let named: String = leftover.file_name().map(|file| { return file.to_string_lossy().into_owned(); }).unwrap_or_default();
        assert!(named.contains(".removing-"), "the leftover is under a name no commit tree has: {}", leftover.display());
    }
    remove(root.as_path());
}

/// What:
///  The name a preparation hands to the removal of its earlier attempt's tree is the name that tree was moved to,
///  never the name the attempt now beginning makes.
/// Why:
///  A removal that outlives its ceiling keeps the pathname it was given and never learns that the tree moved;
///  one holding this attempt's name would examine that name later,
///  after this attempt created its tree,
///  and empty or remove the tree the attempt is writing.
///  The attempt's own writes are here a private index,
///  which is the first thing a commit puts in its tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const again = prepareCommitScratch(common, common, name, 1);
/// expect(removals.named()).not.toContain(again);
/// expect(await read(again + '/commit.index')).toBe('this attempt');
/// ```
#[test]
fn a_removal_of_a_leftover_is_never_given_the_name_the_next_attempt_makes() {
    let root: PathBuf = fixture("scratch-leftover-target");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d-0";
    let first: PathBuf = prepare_commit_scratch(common.as_path(), common.as_path(), name, SCRATCH_REMOVAL_CEILING).expect("the first attempt's tree");
    std::fs::write(first.join("commit.index"), b"the earlier attempt's index").expect("a private index of the earlier attempt");
    // The ceiling of a removal that outlives it: the leftover is still there when the next attempt begins.
    test_targets::forget();
    let again: PathBuf = prepare_commit_scratch(common.as_path(), common.as_path(), name, Duration::from_micros(1)).expect("the next attempt's tree");
    let named: Vec<PathBuf> = test_targets::named();
    assert_eq!(named.len(), 1, "the leftover's removal is started once: {named:?}");
    assert_ne!(named[0], again, "the removal is given the name the old tree was moved to, not the name this attempt makes");
    assert!(named[0] != first, "the removal is not given the name this attempt uses either: {}", named[0].display());
    // What this attempt writes after it is given its tree is still there once a removal that outlived its ceiling has run.
    std::fs::write(again.join("commit.index"), b"this attempt's index").expect("this attempt's private index");
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(
        std::fs::read(again.join("commit.index")).ok().as_deref(),
        Some(&b"this attempt's index"[..]),
        "the tree this attempt made is still its own: {}",
        again.display()
    );
    remove(root.as_path());
}

/// A tree of a command that was killed at any instruction is collected by a later command's preparation of the same
/// directory,
/// because nothing else would ever remove it.
#[test]
fn a_scratch_tree_of_a_killed_command_is_swept() {
    let root: PathBuf = fixture("scratch-commit-killed");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let tree: PathBuf = prepare_commit_scratch(common.as_path(), common.as_path(), "monochromatic-git-policy-cli-service-0a1b2c3d", SCRATCH_REMOVAL_CEILING).expect("the scratch tree");
    std::fs::write(tree.join("commit.index"), b"index").expect("a file in the tree");
    // The command is killed here: nothing removes the tree, and no service ever held it.
    age(tree.as_path(), SCRATCH_STALE_AFTER * 2);
    let _prepared: PathBuf = prepare_repository_scratch(common.as_path()).expect("a later preparation");
    assert!(!tree.exists(), "the sweep of the place the tree is in removes it: {}", tree.display());
    remove(root.as_path());
}
