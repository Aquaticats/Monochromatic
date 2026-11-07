//! What:
//! Round 10 tests:
//! the unsettled scope (9a D1),
//! the strict boot identity (9a D2),
//! the discovery's shared deadline (9a D3),
//! the
//! authority of the boot id (9a section 4),
//! and the false greens of part 9b.
//! Why:
//! The review of round 9 (`cargo-shadow-sol-9a.txt`,
//! `-9b.txt`) named schedules inside existing rows of the acceptance list;
//! each test here is one of them,
//! with a handshake at the boundary and not an elapsed sleep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round ten', () => { it('an unsettled empty scope is not a run that ended', ...) })
//! ```

use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::cgroup_test_support::{delegated_route, start_in_scope};
use crate::failpoints::{Point, arm};
use crate::control_group::{RouteSettings, detect_route, ControlGroup, Identity, Route, current_identity, read_current_identity, use_identity_for_test, view_of};
use crate::crates::LockSnapshot;
use crate::journal::recover;
use crate::journal_recover::survivor_note;
use crate::journal_support::plant;
use crate::journal_unready::after_query_for_test;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The live text of the lockfile a killed run left.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds the output of a killed run.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), LEFT_BY_THE_KILLED_RUN).expect("lock");
    return root;
}

/// A judge that refuses the live lockfile,
/// so that a recovery which goes ahead restores the snapshot.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// Plants a journal whose `groups` file holds `records`.
fn journal_with_records(root: &Path, records: &str) -> PathBuf {
    let journal: PathBuf = plant(root, &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), records).expect("write the records");
    return journal;
}

/// The text of the live lockfile.
fn live_lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("lock");
}

/// A route whose scopes are made inside a delegated holder scope,
/// which the user may write to,
/// so a test can create and remove a control group by hand exactly where a manager would.
fn route_inside(holder: &Path, route: &Route) -> Route {
    return Route { slice_dir: holder.to_path_buf(), slice_view: view_of(holder).expect("the view of the holder"), ..route.clone() };
}

/// Acceptance rows A1,
/// V2,
/// V3/V4 (9a D1,
/// creation before placement):
/// the manager has made the scope's control group and not yet attached the launcher,
/// so the scope exists with `populated 0`.
/// The run did not record its end,
/// so a recovery must not read that empty scope as a run that ended.
#[test]
fn an_unsettled_scope_that_exists_empty_is_not_read_as_a_run_that_ended() {
    let Some(route) = delegated_route() else {
        return;
    };
    let Some(holder) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let inside: Route = route_inside(holder.group.directory(), &route);
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    fs::create_dir(planned.directory()).expect("make the scope's control group the way the manager does, before it attaches a process");
    assert_eq!(fs::read_to_string(planned.directory().join("cgroup.events")).expect("events").lines().find(|line| { return line.starts_with("populated"); }), Some("populated 0"));
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\n", planned.record()));

    let note: Option<String> = survivor_note(&journal);
    let messages: Vec<String> = recover(root.path(), &refusing);
    let lock: String = live_lock(root.path());
    // Fourth certification pass: while the empty scope still exists, the recovery keeps the journal and says why, and a new
    // transaction is not admitted beside it (the journal set is unchanged by the refusal).
    let retained: bool = journal.exists() && journal.join("0.lock").exists();
    let admitted = crate::journal::begin(root.path(), &[snapshot("original")], false);
    let journals_after: usize = crate::journal_support::journal_dirs(root.path()).len();
    let _ = fs::remove_dir(planned.directory());

    let blocked: String = note.expect("an unsettled, empty scope blocks");
    assert!(blocked.contains("did not record its end") && blocked.contains("cargo-shadow acknowledge"), "{blocked}");
    assert_eq!(lock, LEFT_BY_THE_KILLED_RUN, "a recovery restored files while a program could still be attached: {messages:?}");
    assert!(messages.iter().any(|message| { return message.contains("did not record its end"); }), "the recovery did not refuse for the unsettled scope: {messages:?}");
    assert!(retained, "the journal or its snapshot was removed by the refusing recovery: {messages:?}");
    assert!(admitted.is_err_and(|error| { return error.to_string().contains("did not record its end"); }), "a transaction was admitted beside an unsettled run, or the refusal does not name it");
    assert_eq!(journals_after, 1, "the refused admission left a journal behind");
}

/// Acceptance rows A1,
/// V2,
/// V3/V4 (9a D1,
/// the race between two observations):
/// the check finds the scope absent,
/// and the launcher's request then makes it before the check ends.
/// Nothing may decide from a second look that the run is complete.
#[test]
fn a_scope_that_appears_between_the_query_and_the_judgement_is_not_read_as_a_run_that_ended() {
    let Some(route) = delegated_route() else {
        return;
    };
    let Some(holder) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let inside: Route = route_inside(holder.group.directory(), &route);
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    assert!(!planned.directory().exists());
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\n", planned.record()));
    let directory: PathBuf = planned.directory().to_path_buf();
    after_query_for_test(Some(Rc::new(move || { fs::create_dir(&directory).expect("the launcher's request makes the scope"); })));

    let note: Option<String> = survivor_note(&journal);
    after_query_for_test(None);
    let created: bool = planned.directory().exists();
    let _ = fs::remove_dir(planned.directory());

    assert!(created, "the hook did not run between the query and the judgement");
    let blocked: String = note.expect("a scope that appeared after the query blocks");
    assert!(blocked.contains("did not record its end"), "{blocked}");
}

/// Acceptance rows A1,
/// V2 (certification,
/// third pass):
/// the same race,
/// driven through the real recovery.
/// The scope appears after the
/// query inside the recovery's own check;
/// the recovery must refuse,
/// restore nothing and keep the journal,
/// whatever the second
/// look at the scope shows.
#[test]
fn a_scope_that_appears_inside_the_real_recovery_is_not_read_as_a_run_that_ended() {
    let Some(route) = delegated_route() else {
        return;
    };
    let Some(holder) = start_in_scope(Command::new("sleep").arg("600")) else {
        return;
    };
    let inside: Route = route_inside(holder.group.directory(), &route);
    let planned: ControlGroup = ControlGroup::plan(&inside).expect("a scope name");
    assert!(!planned.directory().exists());
    let root = repository();
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\n", planned.record()));
    let directory: PathBuf = planned.directory().to_path_buf();
    after_query_for_test(Some(Rc::new(move || { fs::create_dir(&directory).expect("the launcher's request makes the scope"); })));

    let messages: Vec<String> = recover(root.path(), &refusing);
    after_query_for_test(None);
    let created: bool = planned.directory().exists();
    let _ = fs::remove_dir(planned.directory());

    assert!(created, "the hook did not run inside the recovery's check");
    assert!(messages.iter().any(|message| { return message.contains("did not record its end"); }), "the recovery did not refuse for the unsettled scope: {messages:?}");
    assert_eq!(live_lock(root.path()), "unchecked output of the killed run", "a recovery went ahead on a scope that appeared during its check: {messages:?}");
    assert!(journal.exists(), "the journal was removed: {messages:?}");
}

/// A process in a process group of its own,
/// which is alive until the test ends,
/// however it ends.
struct LiveGroup(
    /// The process that leads the group.
    Child,
);

/// The methods of the guard.
impl LiveGroup {
    /// The process id,
    /// which is the group's number.
    fn id(&self) -> u32 {
        return self.0.id();
    }
}

/// How the guard cleans up.
impl Drop for LiveGroup {
    /// What:
    /// `fn drop(&mut self)` kills and reaps the process.
    /// Why:
    /// A failed assertion must not leave a `sleep` behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Starts a live process group.
fn live_group() -> LiveGroup {
    return LiveGroup(Command::new("sleep").arg("600").process_group(0).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("a live group"));
}

/// This machine's identity with its boot id changed in one hexadecimal digit,
/// as a real reboot would change it.
fn after_a_reboot() -> Identity {
    let now: Identity = current_identity().expect("this machine has an identity");
    let mut boot: Vec<u8> = now.boot.into_bytes();
    boot[0] = if boot[0] == b'0' { b'1' } else { b'0' };
    return Identity { boot: String::from_utf8(boot).expect("ascii"), machine: now.machine };
}

/// Rewrites the identity lines of the journal's view record and keeps its first line.
fn set_identity_lines(journal: &Path, lines: &str) {
    let recorded: String = fs::read_to_string(journal.join("view")).expect("the view record");
    let first: &str = recorded.lines().next().expect("a view line");
    fs::write(journal.join("view"), format!("{first}\n{lines}")).expect("rewrite the view");
}

/// Acceptance rows V2 and the weaker path (9a D2,
/// recorded side):
/// a recorded boot or machine field that is not in the format of an id is not a different boot,
/// a repeated field or an unknown line makes the record unusable,
/// and each of them keeps a live process group blocking.
/// The control is a well-formed record of another boot,
/// which does clear it (so the probe can move).
#[test]
fn a_malformed_recorded_identity_is_not_a_reboot() {
    let root = repository();
    let group = live_group();
    let journal: PathBuf = journal_with_records(root.path(), &format!("pgroup {}\n", group.id()));
    let now: Identity = current_identity().expect("an identity");
    let other: Identity = after_a_reboot();
    let boot_line = |boot: &str| -> String { return format!("boot {boot}\nmachine {}\n", now.machine); };
    let cases: Vec<(&str, String)> = vec![
        ("a boot field that is not an id", boot_line("not-a-boot-id")),
        ("an upper-case boot id", boot_line(&other.boot.to_uppercase())),
        ("a boot id with a trailing space", boot_line(&format!("{} ", other.boot))),
        ("an all-zero boot id", boot_line("00000000-0000-0000-0000-000000000000")),
        ("a machine field that is not an id", format!("boot {}\nmachine not-a-machine-id\n", other.boot)),
        // Both orders of the repeated field (fourth certification pass): a reader that keeps the last value, or the first, is wrong in
        // exactly one of them, and the order that would clear the live group is the one the other boot comes last.
        ("a repeated boot field, the other boot first", format!("boot {}\nboot {}\nmachine {}\n", other.boot, now.boot, now.machine)),
        ("a repeated boot field, the other boot last", format!("boot {}\nboot {}\nmachine {}\n", now.boot, other.boot, now.machine)),
        ("a repeated machine field", format!("boot {}\nmachine {}\nmachine {}\n", other.boot, now.machine, now.machine)),
        ("an unknown line", format!("boot {}\nmachine {}\nthree {}\n", other.boot, now.machine, now.machine)),
    ];
    for (label, lines) in &cases {
        set_identity_lines(&journal, lines);
        let note: Option<String> = survivor_note(&journal);
        assert!(note.is_some_and(|text| { return text.contains("still running"); }), "{label}: a malformed identity cleared a live process group");
    }
    // The record says another boot than the one running now: the machine is the same, so a reboot ended the group.
    set_identity_lines(&journal, &boot_line(&other.boot));
    let cleared: Option<String> = survivor_note(&journal);
    assert_eq!(cleared, None, "the control: a well-formed record of another boot did not clear the group, so the probe cannot move");
}

/// Acceptance rows V2 and the weaker path (9a D2,
/// current side):
/// a current identity that is not well formed disables the shortcut instead of showing a reboot.
#[test]
fn a_malformed_current_identity_is_not_a_reboot() {
    let root = repository();
    let group = live_group();
    let journal: PathBuf = journal_with_records(root.path(), &format!("pgroup {}\n", group.id()));
    let now: Identity = current_identity().expect("an identity");
    let cases: Vec<(&str, Identity)> = vec![
        ("a current boot field that is not an id", Identity { boot: "not-a-boot-id".to_string(), machine: now.machine.clone() }),
        ("a current machine field that is not an id", Identity { boot: after_a_reboot().boot, machine: "not-a-machine-id".to_string() }),
    ];
    for (label, identity) in cases {
        use_identity_for_test(Some(Ok(identity)));
        let note: Option<String> = survivor_note(&journal);
        use_identity_for_test(None);
        assert!(note.is_some_and(|text| { return text.contains("still running"); }), "{label}: cleared a live process group");
    }
}

/// The files the current identity is read from must have the format of an id (9a D2):
/// each malformed file is an error.
#[test]
fn the_current_identity_is_read_strictly_from_its_files() {
    let directory = scratch();
    let machine: PathBuf = directory.path().join("machine-id");
    let real_boot: &Path = Path::new(crate::control_group::BOOT_ID_PATH);
    for (label, text) in [("empty", ""), ("too short", "abcdef\n"), ("upper case", "ABCDEF0123456789ABCDEF0123456789\n"), ("a UUID with hyphens", "0123456789abcdef-0123456789abcdef\n"), ("all zero", "00000000000000000000000000000000\n")] {
        fs::write(&machine, text).expect("write");
        assert!(read_current_identity(real_boot, &machine).is_err(), "{label}: accepted as a machine id");
    }
    fs::write(&machine, "0123456789abcdef0123456789abcdef\n").expect("write");
    let read: Identity = read_current_identity(real_boot, &machine).expect("a well-formed machine id and this host's boot id");
    assert_eq!(read.machine, "0123456789abcdef0123456789abcdef");
}

/// The environment variable that gives the helper process its workspace.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_ROUND_TEN_FIXTURE_ROOT";

/// The environment variable that tells the helper process which replacement of the boot id file to make.
const FIXTURE_MODE: &str = "CARGO_SHADOW_ROUND_TEN_FIXTURE_MODE";

/// Runs `mount` with the arguments inside the helper's own mount namespace and panics when it fails.
fn mount_in_namespace(arguments: &[&str]) {
    let status = Command::new("mount").args(arguments).status().expect("mount must be installed");
    assert!(status.success(), "mount {arguments:?} failed: {status:?}");
}

/// The helper process of the boot authority test:
/// it runs in a new user and mount namespace (the host's `/proc` is not touched),
/// puts a replacement over the boot id file in the way
/// its mode says,
/// then reads the identity and asks the liveness check (`survivor_note`,
/// not the whole `recover`) about the journal the
/// test planted,
/// and writes both results to files.
/// It shows the identity rejection and the check's refusal,
/// not an unchanged output or a
/// retained journal under those replacements.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_boot_id_put_over_the_kernels_file_is_not_an_identity"]
fn boot_authority_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let mode: String = std::env::var(FIXTURE_MODE).expect("a mode");
    let fake: PathBuf = root.join("fake-boot-id");
    let boot_file: &str = crate::control_group::BOOT_ID_PATH;
    match mode.as_str() {
        "baseline" => {}
        "file" => mount_in_namespace(&["--bind", &fake.display().to_string(), boot_file]),
        "uuid" => mount_in_namespace(&["--bind", "/proc/sys/kernel/random/uuid", boot_file]),
        "masked-directory" => {
            mount_in_namespace(&["--bind", "/proc/sys/kernel/random", "/proc/sys/kernel/random"]);
            mount_in_namespace(&["-o", "remount,bind,ro", "/proc/sys/kernel/random"]);
        }
        "symlink" => {
            // The genuine directory is saved at another path, and a replacement directory whose `boot_id` is a symbolic link to the
            // genuine `uuid` file is put over it (11a: a path check that follows links finds procfs and no mount root).
            let saved: PathBuf = root.join("saved");
            let replacement: PathBuf = root.join("replacement");
            fs::create_dir_all(&saved).expect("saved");
            fs::create_dir_all(&replacement).expect("replacement");
            mount_in_namespace(&["--bind", "/proc/sys/kernel/random", &saved.display().to_string()]);
            std::os::unix::fs::symlink(saved.join("uuid"), replacement.join("boot_id")).expect("the link");
            mount_in_namespace(&["--bind", &replacement.display().to_string(), "/proc/sys/kernel/random"]);
        }
        "tmpfs-directory" => {
            mount_in_namespace(&["-t", "tmpfs", "tmpfs", "/proc/sys/kernel/random"]);
            fs::write(boot_file, fs::read_to_string(&fake).expect("fake")).expect("a boot id file on the tmpfs");
        }
        other => panic!("unknown mode {other}"),
    }
    let identity: String = match current_identity() {
        Ok(found) => format!("ok {}", found.boot),
        Err(reason) => format!("err {reason}"),
    };
    // The journal was made with the real identity; the helper gives it the view this namespace has of it, so only the identity matters.
    for journal in crate::journal_support::journal_dirs(&root) {
        let recorded: String = fs::read_to_string(journal.join("view")).expect("the view record");
        let rest: String = recorded.lines().skip(1).map(|line| { return format!("{line}\n"); }).collect();
        fs::write(journal.join("view"), format!("view {}\n{rest}", view_of(&journal).expect("views"))).expect("rewrite the view");
    }
    let note: Option<String> = crate::journal_recover::survivor_note(&crate::journal_support::journal_dirs(&root).into_iter().next().expect("a journal"));
    fs::write(root.join("fixture-result"), format!("{identity}\n{}\n", note.unwrap_or_else(|| { return "no survivor".to_string(); }))).expect("write the result");
}

/// Acceptance rows V2,
/// amended c,
/// and the weaker path (9a section 4,
/// the authority of the boot rule):
/// a boot id file that was put over the kernel's,
/// by a bind mount of a regular file,
/// of another procfs file,
/// or of a file system over
/// its directory,
/// is not an identity,
/// and a live process group of the journal keeps blocking on the same running machine.
/// A read-only bind of the directory that holds the file (the way container runtimes mask paths) is not a replacement.
#[test]
fn a_boot_id_put_over_the_kernels_file_is_not_an_identity() {
    let now: Identity = current_identity().expect("this host's boot id is the kernel's, which the test needs");
    let replacement: String = format!("{}\n", after_a_reboot().boot);
    for (mode, replaced) in [("baseline", false), ("file", true), ("uuid", true), ("symlink", true), ("tmpfs-directory", true), ("masked-directory", false)] {
        let root = repository();
        fs::write(root.path().join("fake-boot-id"), &replacement).expect("write the replacement");
        let group = live_group();
        let _journal: PathBuf = journal_with_records(root.path(), &format!("pgroup {}\n", group.id()));
        let status = Command::new("unshare")
            .args(["--user", "--map-root-user", "--mount", "--"])
            .arg(std::env::current_exe().expect("this test executable"))
            .args(["--ignored", "--exact", "round_ten_tests::boot_authority_fixture", "--nocapture"])
            .env(FIXTURE_ROOT, root.path())
            .env(FIXTURE_MODE, mode)
            .status()
            .expect("unshare must be installed and allowed to make a user and mount namespace for this test");

        assert!(status.success(), "{mode}: the helper process failed: {status:?}");
        let said: String = fs::read_to_string(root.path().join("fixture-result")).expect("the helper's result");
        let (identity, note): (&str, &str) = said.split_once('\n').expect("two lines");
        if replaced {
            assert!(identity.starts_with("err "), "{mode}: a replaced boot id file was read as an identity: {identity}");
            assert!(note.contains("still running"), "{mode}: a replaced boot id cleared a live process group: {note}");
        } else {
            assert_eq!(identity, format!("ok {}", now.boot), "{mode}: the real boot id was refused");
            assert!(note.contains("still running"), "{mode}: the group is live and the boot is the same: {note}");
        }
    }
}

/// Acceptance row V6 (9a D3):
/// a discovery program that does not end is killed at the deadline and its reaping spends only what is left of that deadline.
/// The child here cannot be reaped (the failpoint),
/// so the return must come at the deadline and say that the child was not reaped,
/// not a fresh budget later.
#[test]
fn a_discovery_that_cannot_reap_its_child_returns_at_the_deadline_and_says_so() {
    // The bound is long enough that a second budget of the same length (or the old two seconds) cannot be mistaken for scheduling
    // overhead: one deadline returns at about the bound, two budgets at about twice the bound (11b 2.1).
    let bound: Duration = Duration::from_millis(1500);
    let settings: RouteSettings = RouteSettings {
        systemctl: "sh".to_string(),
        question: Some(vec!["-c".to_string(), "exec sleep 30".to_string()]),
        discovery_bound: bound,
        ..RouteSettings::default()
    };
    arm(Point::ReapStuck);
    let started = Instant::now();

    let refused = detect_route(&settings).expect_err("a manager that does not answer is a missing capability");
    let elapsed: Duration = started.elapsed();

    assert!(!crate::failpoints::fires(Point::ReapStuck), "the reaping fault was never reached, so the report did not come from it");
    assert!(refused.to_string().contains("did not answer"), "{refused}");
    assert!(refused.to_string().contains("could not be reaped"), "the unreaped child was not reported: {refused}");
    assert!(elapsed >= bound, "the discovery returned after {elapsed:?}, before its deadline of {bound:?}");
    assert!(elapsed < bound + Duration::from_millis(700), "the discovery returned after {elapsed:?}: a second budget was spent after its deadline of {bound:?}");
}

/// The helper process of the bind-mount substitution test (9b 2.4):
/// in a new user and mount namespace (the cgroup namespace is the one it inherited) it gives the journal the views this namespace has,
/// so that nothing differs,
/// asks for the verdict,
/// then binds a different cgroup directory over the directory that held the recorded
/// scope,
/// and asks again.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_bind_mount_over_the_directory_of_a_recorded_scope_is_refused_in_the_same_cgroup_namespace"]
fn bind_substitution_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let journal: PathBuf = crate::journal_support::journal_dirs(&root).into_iter().next().expect("a journal");
    let groups: String = fs::read_to_string(journal.join("groups")).expect("the records");
    let mut recorded_slice: Option<PathBuf> = None;
    let mut rewritten: String = String::new();
    for line in groups.lines() {
        match crate::control_group::parse_record(line) {
            Ok(crate::control_group::Record::Scope { namespace, path, .. }) => {
                let parent: PathBuf = path.parent().expect("a parent").to_path_buf();
                let here: crate::control_group::View = view_of(&parent).expect("the view of the parent here");
                rewritten.push_str(&format!("{}\n", crate::control_group::Record::Scope { namespace, parent_view: here, path }));
                recorded_slice = Some(parent);
            }
            _ => rewritten.push_str(&format!("{line}\n")),
        }
    }
    fs::write(journal.join("groups"), rewritten).expect("rewrite the records as this namespace sees them");
    let recorded: String = fs::read_to_string(journal.join("view")).expect("the view record");
    let rest: String = recorded.lines().skip(1).map(|line| { return format!("{line}\n"); }).collect();
    fs::write(journal.join("view"), format!("view {}\n{rest}", view_of(&journal).expect("views"))).expect("rewrite the view");

    let baseline: Option<String> = crate::journal_recover::survivor_note(&journal);
    let slice_dir: PathBuf = recorded_slice.expect("a scope record");
    let substitute: PathBuf = slice_dir.parent().expect("a directory above the slice").to_path_buf();
    mount_in_namespace(&["--bind", &substitute.display().to_string(), &slice_dir.display().to_string()]);
    let substituted: Option<String> = crate::journal_recover::survivor_note(&journal);
    // The real recovery too (certification group 1): it must refuse the substituted directory, not only classify it.
    let recovery: Vec<String> = crate::journal::recover(&root, &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });
    fs::write(root.join("fixture-recovery"), recovery.join("\n")).expect("write what the recovery said");
    fs::write(
        root.join("fixture-result"),
        format!("{}\n{}\n", baseline.unwrap_or_else(|| { return "no survivor".to_string(); }), substituted.unwrap_or_else(|| { return "no survivor".to_string(); })),
    )
    .expect("write the result");
}

/// Acceptance row V2,
/// amended c (9b 2.4,
/// the same-cgroup-namespace bind-mount substitution of 7a F6):
/// with the cgroup namespace unchanged and every recorded view equal,
/// a bind mount of another cgroup directory over the directory that
/// held the settled,
/// absent scope makes the recovery refuse for the cgroup parent view,
/// and the journal and the lockfile stay.
/// The first verdict (nothing substituted) is the control:
/// it clears,
/// so the refusal is caused by the bind and nothing else.
#[test]
fn a_bind_mount_over_the_directory_of_a_recorded_scope_is_refused_in_the_same_cgroup_namespace() {
    let Some(route) = delegated_route() else {
        return;
    };
    let root = repository();
    let planned: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let journal: PathBuf = journal_with_records(root.path(), &format!("{}\nsettled {}\n", planned.record(), planned.directory().display()));

    let status = Command::new("unshare")
        .args(["--user", "--map-root-user", "--mount", "--"])
        .arg(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "round_ten_tests::bind_substitution_fixture", "--nocapture"])
        .env(FIXTURE_ROOT, root.path())
        .status()
        .expect("unshare must be installed and allowed to make a user and mount namespace for this test");

    assert!(status.success(), "the helper process failed: {status:?}");
    let said: String = fs::read_to_string(root.path().join("fixture-result")).expect("the helper's result");
    let (baseline, substituted): (&str, &str) = said.trim_end().split_once('\n').expect("two verdicts");
    assert_eq!(baseline, "no survivor", "the control did not clear, so the refusal below is not caused by the bind: {baseline}");
    assert!(substituted.contains("recorded path may lead somewhere else"), "the substitution was not refused for the cgroup parent view: {substituted}");
    assert!(!substituted.contains("another mount or path"), "the journal view differed too, so the cgroup view was not isolated: {substituted}");
    let recovery: String = fs::read_to_string(root.path().join("fixture-recovery")).expect("what the recovery said");
    assert!(recovery.contains("recorded path may lead somewhere else") && recovery.contains("cargo-shadow acknowledge"), "the recovery did not refuse the substituted directory: {recovery}");
    assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "the recovery changed the live output");
    assert!(journal.exists(), "the journal was not retained by the recovery");
}

/// Acceptance row V2,
/// amended c (9b section 2,
/// "every other check"):
/// a reboot discharges only the liveness of the run's processes.
/// The completion marker of the recovery still runs and still keep the journal when they fail,
/// and the same
/// recovery settles the journal once the barrier works (the control,
/// so the probe can move).
#[test]
fn a_reboot_still_publishes_the_completion_marker_of_the_recovery_or_keeps_the_journal() {
    for fault in [Point::CompletionRename] {
        let root = repository();
        let journal: PathBuf = journal_with_records(root.path(), "pgroup starting\npgroup 2147483646\n");
        use_identity_for_test(Some(Ok(after_a_reboot())));
        arm(fault);
        let failed: Vec<String> = recover(root.path(), &refusing);
        let reached: bool = !crate::failpoints::fires(fault);
        let kept: bool = journal.exists() && !journal.to_string_lossy().ends_with(".done") && crate::journal_support::journal_dirs(root.path()).len() == 1;
        let again: Vec<String> = recover(root.path(), &refusing);
        use_identity_for_test(None);

        assert!(reached, "{fault:?}: the barrier was never reached, so the reboot skipped the finish ({failed:?})");
        assert!(kept, "{fault:?}: the journal was removed or marked done although its barrier failed ({failed:?})");
        assert!(!failed.is_empty(), "{fault:?}: the failure of the barrier was not reported");
        // The control must complete the journal, not only leave the restored text behind (11b 2.5): the journal is gone.
        assert!(crate::journal_support::journal_dirs(root.path()).is_empty(), "{fault:?}: the control recovery left a journal behind: {:?} {again:?}", crate::journal_support::journal_dirs(root.path()));
        assert_eq!(live_lock(root.path()), "original", "{fault:?}: the control did not settle the journal once the barrier worked: {again:?}");
    }
}
