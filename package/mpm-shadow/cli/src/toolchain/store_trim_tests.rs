//! What:
//! Unit tests of the size- and age-bounded cleanup in `store_trim.rs`:
//! the rule,
//! the bounds it reads and the marks it prints,
//! the locks whose pins protect a version,
//! and the removal itself,
//! including one cut short and finished by the next pass.
//! Why:
//! The pass deletes installed toolchains,
//! so each thing it must never remove (a version a lock names, one this run installed, one a running process holds,
//! one written inside the age floor, one the `current` link names) and each thing it must remove is a test,
//! and each runs on a store of the test's own under a disposable home.

/// Imports `BTreeMap` and `BTreeSet` from `std::collections`.
use std::collections::BTreeSet;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `FileTimes` from `std::fs`.
use std::fs::FileTimes;
/// Imports what these tests use from `std::path`.
use std::path::PathBuf;
/// Imports `Duration`,
/// `Instant`,
/// `SystemTime` from `std::time`.
use std::time::{Duration, Instant, SystemTime};

/// Imports the module under test.
use crate::toolchain::store_path_holders::{StoreHolders, held_reason};
use crate::toolchain::store_trim::{
    DEFAULT_SIZE_BOUND_BYTES, TrimBounds, TrimCandidate, TrimPlan, TrimRun, TrimStep, apply_trim, candidates_of,
    DEFAULT_AGE_FLOOR_SECONDS, TrimOutcome, finish_removals, format_age, has_unfinished_removal, parse_duration,
    parse_size, plan_trim, read_named_versions, settle_markers, unfinished_marker,
};
use crate::toolchain::context::{Context, Host};
use crate::toolchain::fakes::FakeRunner;
use crate::toolchain::fixtures::sample_scope_on_disk;
use crate::toolchain::http::NoNetwork;
use crate::toolchain::scope::EnvMap;
use crate::toolchain::store_trim::trim_after_install;
use crate::toolchain::store_trim::{MAX_AGE_FLOOR, MAX_WORK_LIMIT, format_span};
use crate::toolchain::store_versions::{removal_markers, TOMBSTONE_PREFIX, tombstone_of};
use crate::test_scratch::scratch;

/// A mebibyte, so a test's bound reads as a fixture size.
const MIB: u64 = 1024 * 1024;

/// A bound that keeps nothing and removes nothing inside an hour,
/// for the pure-rule tests.
fn tight_bounds(size_bound_bytes: u64) -> TrimBounds {
    return TrimBounds { size_bound_bytes, age_floor: Duration::ZERO, work_limit: Duration::from_secs(60) };
}

/// One candidate with a made-up path, for the pure-rule tests.
fn candidate(tool: &str, version: &str, bytes: u64, written_seconds_ago: u64) -> TrimCandidate {
    return TrimCandidate {
        root: PathBuf::from("/store"),
        tool: tool.to_string(),
        version: version.to_string(),
        directory: PathBuf::from(format!("/store/{tool}/{version}")),
        downloads: None,
        bytes,
        written: SystemTime::now() - Duration::from_secs(written_seconds_ago),
        leftover: false,
    };
}

/// The versions a plan removes,
/// oldest write first.
fn removed_by(plan: &TrimPlan) -> Vec<String> {
    let mut removed: Vec<String> = Vec::new();
    for step in &plan.steps {
        if let TrimStep::Remove(found) = step {
            removed.push(format!("{}-{}", found.tool, found.version));
        }
    }
    return removed;
}

/// The versions a plan keeps, with why.
fn kept_by(plan: &TrimPlan) -> Vec<(String, String)> {
    let mut kept: Vec<(String, String)> = Vec::new();
    for step in &plan.steps {
        if let TrimStep::Keep(found, reason) = step {
            kept.push((format!("{}-{}", found.tool, found.version), reason.clone()));
        }
    }
    return kept;
}

//region the rule

/// While the stores keep no more than the bound, nothing is removed and nothing is said.
#[test]
fn removes_nothing_while_under_the_bound() {
    let candidates = vec![candidate("node", "26.12.0", MIB, 3600), candidate("node", "26.13.1", MIB, 60)];
    let plan: TrimPlan = plan_trim(&candidates, &tight_bounds(2 * MIB), SystemTime::now(), &|_found| return None);

    assert!(!plan.over_bound);
    assert!(plan.steps.is_empty(), "{:?}", plan.steps);
}

/// Over the bound, the versions written longest ago go until the space is inside it,
/// and the newest one stays because the space is inside the bound by then.
#[test]
fn removes_the_oldest_writes_until_inside_the_bound() {
    let candidates = vec![
        candidate("aws", "2.38.2", MIB, 40 * 3600),
        candidate("aws", "2.39.2", MIB, 30 * 3600),
        candidate("aws", "2.40.2", MIB, 60),
    ];
    let plan: TrimPlan = plan_trim(&candidates, &tight_bounds(MIB), SystemTime::now(), &|_found| return None);

    assert!(plan.over_bound);
    assert_eq!(removed_by(&plan), vec!["aws-2.38.2", "aws-2.39.2"]);
    assert!(kept_by(&plan).is_empty());
}

/// A version a lock names is skipped and the pass moves to the next oldest.
#[test]
fn skips_a_version_a_lock_names_and_goes_on() {
    let candidates = vec![
        candidate("node", "26.12.0", MIB, 40 * 3600),
        candidate("node", "26.12.1", MIB, 30 * 3600),
        candidate("node", "26.13.1", MIB, 60),
    ];
    let pinned: TrimCandidate = candidates[0].clone();
    // Three mebibytes over a bound of two: removing the second-oldest is enough once the oldest is kept.
    let plan: TrimPlan = plan_trim(&candidates, &tight_bounds(2 * MIB), SystemTime::now(), &|found| {
        if *found == pinned {
            return Some("the lock /work/toolchain.lock.json names it".to_string());
        }
        return None;
    });

    assert_eq!(removed_by(&plan), vec!["node-26.12.1"]);
    assert_eq!(
        kept_by(&plan),
        vec![("node-26.12.0".to_string(), "the lock /work/toolchain.lock.json names it".to_string())]
    );
}

/// A version written inside the age floor stays whatever else is true,
/// and the pass goes on to an older one.
#[test]
fn keeps_a_version_written_inside_the_age_floor() {
    let candidates = vec![candidate("node", "26.12.0", MIB, 60), candidate("node", "26.11.0", MIB, 5 * 3600)];
    let bounds = TrimBounds { size_bound_bytes: MIB, age_floor: Duration::from_secs(3600), work_limit: Duration::from_secs(60) };
    let plan: TrimPlan = plan_trim(&candidates, &bounds, SystemTime::now(), &|_found| return None);

    assert_eq!(removed_by(&plan), vec!["node-26.11.0"]);
    let kept: Vec<String> = kept_by(&plan).into_iter().map(|(name, _reason)| return name).collect();
    assert_eq!(kept, vec!["node-26.12.0"]);
}

/// When everything over the bound is protected, the plan removes nothing and says why for each.
#[test]
fn keeps_everything_when_every_candidate_is_in_use() {
    let candidates = vec![candidate("node", "26.12.0", MIB, 40 * 3600), candidate("node", "26.12.1", MIB, 30 * 3600)];
    let plan: TrimPlan = plan_trim(&candidates, &tight_bounds(0), SystemTime::now(), &|_found| {
        return Some("it is in use: pid 7 runs it".to_string());
    });

    assert!(removed_by(&plan).is_empty());
    assert_eq!(kept_by(&plan).len(), 2);
}

//endregion the rule

//region the bounds and the words

/// The size bound is read from bytes or from a binary suffix.
#[test]
fn reads_a_size_bound() {
    assert_eq!(parse_size("1024"), Some(1024));
    assert_eq!(parse_size("4KiB"), Some(4 * 1024));
    assert_eq!(parse_size("24GiB"), Some(24 * 1024 * 1024 * 1024));
    assert_eq!(parse_size(" 2 Gib "), Some(2 * 1024 * 1024 * 1024));
    assert_eq!(parse_size("1TiB"), Some(1024 * 1024 * 1024 * 1024));
    assert_eq!(parse_size("plenty"), None);
    assert_eq!(parse_size("12 potatoes"), None);
}

/// The age floor and the work limit are read from seconds, suffixes or milliseconds.
#[test]
fn reads_a_span() {
    assert_eq!(parse_duration("3600"), Some(Duration::from_secs(3600)));
    assert_eq!(parse_duration("90s"), Some(Duration::from_secs(90)));
    assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
    assert_eq!(parse_duration("2h"), Some(Duration::from_secs(7200)));
    assert_eq!(parse_duration("3d"), Some(Duration::from_secs(3 * 86400)));
    assert_eq!(parse_duration("2000ms"), Some(Duration::from_millis(2000)));
    assert_eq!(parse_duration("soon"), None);
}

/// A span reads in the unit a person compares,
/// and a version written a minute ago is never "0 h".
#[test]
fn reads_a_span_as_words() {
    let now: SystemTime = SystemTime::now();
    assert_eq!(format_age(now, now - Duration::from_secs(5)), "5 s");
    assert_eq!(format_age(now, now - Duration::from_secs(90)), "1 min");
    assert_eq!(format_age(now, now - Duration::from_secs(3600 * 20 + 600)), "20 h");
    assert_eq!(format_age(now, now - Duration::from_secs(86400 * 3)), "3 d");
}

//endregion the bounds and the words

//region the locks

/// Every route's directory name comes from the lock rows,
/// and a row without a version makes the pass keep everything.
#[test]
fn reads_the_pinned_versions_of_a_lock() {
    let directory = scratch();
    let lock: PathBuf = directory.path().join("toolchain.lock.json");
    fs::write(
        &lock,
        r#"{"tools":{
            "node":{"version":"26.13.1","route":"archive"},
            "conda:bubblewrap":{"version":"0.13.1","route":"archive"},
            "cargo:fd-find":{"version":"10.5.2","route":"cargo-binstall"},
            "npm:typescript":{"version":"6.1.0","route":"mpm-pnpm"}
        }}"#,
    )
    .expect("write the lock");

    let named = read_named_versions(std::slice::from_ref(&lock));

    assert_eq!(named.unreadable, None);
    assert_eq!(
        named.tool_directories(),
        ["node".to_string(), "conda-bubblewrap".to_string(), "cargo/fd-find".to_string()].into_iter().collect::<BTreeSet<String>>()
    );
    assert_eq!(named.naming_lock("node", "26.13.1"), Some(lock.display().to_string()).as_deref());
    assert_eq!(named.naming_lock("node", "26.12.0"), None);
    assert_eq!(named.naming_lock("cargo/fd-find", "10.5.2"), Some(lock.display().to_string()).as_deref());
}

/// A lock row without a version is what makes the pass keep everything,
/// because that row might pin any version.
#[test]
fn refuses_to_judge_a_lock_whose_row_has_no_version() {
    let directory = scratch();
    let lock: PathBuf = directory.path().join("toolchain.lock.json");
    fs::write(&lock, r#"{"tools":{"node":{"route":"archive"}}}"#).expect("write the lock");

    let named = read_named_versions(&[lock]);

    assert!(named.unreadable.is_some(), "{named:?}");
}

/// Another worktree's lock that is gone names nothing and is not a reason to keep everything.
#[test]
fn skips_a_lock_that_is_gone() {
    let directory = scratch();
    let named = read_named_versions(&[directory.path().join("gone.lock.json")]);

    assert_eq!(named.unreadable, None);
    assert!(named.tool_directories().is_empty());
}

//endregion the locks

//region the removal

/// What the control's store looks like:
/// one root with three node versions and one aws version,
/// the oldest node version written two days ago.
struct Fixture {
    /// Held so the disposable directory lives as long as the test.
    #[expect(dead_code)]
    directory: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let directory = scratch();
        let root: PathBuf = directory.path().join("home/.local/share/monochromatic-dev/toolchain");
        for (tool, version, age) in [
            ("node", "26.11.0", 2 * 24 * 3600),
            ("node", "26.12.0", 24 * 3600),
            ("node", "26.13.1", 60),
            ("aws", "2.40.2", 48 * 3600),
        ] {
            let version_directory: PathBuf = root.join(tool).join(version);
            fs::create_dir_all(&version_directory).expect("make a version directory");
            fs::write(version_directory.join("program"), vec![b'a'; 1000]).expect("write the program");
            let when: SystemTime = SystemTime::now() - Duration::from_secs(age);
            let file = fs::File::open(&version_directory).expect("open the version directory");
            file.set_times(FileTimes::new().set_modified(when)).expect("set its write time");
        }
        return Fixture { directory, root };
    }

    fn candidates(&self) -> Vec<TrimCandidate> {
        let tools: BTreeSet<String> = ["node".to_string(), "aws".to_string()].into_iter().collect();
        return candidates_of(std::slice::from_ref(&self.root), &tools);
    }

    fn run(&self) -> TrimRun {
        return TrimRun {
            bounds: TrimBounds {
                size_bound_bytes: 0,
                age_floor: Duration::ZERO,
                work_limit: Duration::from_secs(60),
            },
            started: Instant::now(),
            roots: vec![self.root.clone()],
        };
    }
}

/// The control the brief asks for:
/// over the bound the oldest unpinned version goes,
/// and the version this run installed stays.
#[test]
fn removes_the_oldest_unpinned_version_and_keeps_the_installed_one() {
    let fixture = Fixture::new();
    let candidates: Vec<TrimCandidate> = fixture.candidates();
    let installed: TrimCandidate = candidates
        .iter()
        .find(|found| return found.version == "26.13.1")
        .expect("the installed version is scanned")
        .clone();
    let plan: TrimPlan = plan_trim(&candidates, &fixture.run().bounds, SystemTime::now(), &|found| {
        if *found == installed {
            return Some("this install put it there".to_string());
        }
        return None;
    });

    let outcome = apply_trim(&plan, &fixture.run());

    assert_eq!(outcome.removed, 3, "{:?}", outcome.lines);
    assert!(!fixture.root.join("node/26.11.0").exists());
    assert!(!fixture.root.join("node/26.12.0").exists());
    assert!(!fixture.root.join("aws/2.40.2").exists());
    assert!(fixture.root.join("node/26.13.1/program").exists());
    assert!(outcome.lines.iter().any(|line| return line.contains("removed") && line.contains("node/26.11.0")));
    assert!(outcome.lines.iter().any(|line| return line.contains("kept") && line.contains("this install put it there")));
}

/// The control the brief asks for:
/// under the bound nothing goes and no line is printed.
#[test]
fn removes_nothing_and_says_nothing_under_the_bound() {
    let fixture = Fixture::new();
    let plan: TrimPlan = plan_trim(
        &fixture.candidates(),
        &TrimBounds { size_bound_bytes: 6 * MIB, age_floor: Duration::ZERO, work_limit: Duration::from_secs(60) },
        SystemTime::now(),
        &|_found| return None,
    );

    let outcome = apply_trim(&plan, &fixture.run());

    assert_eq!(outcome.removed, 0);
    assert!(outcome.lines.is_empty(), "{:?}", outcome.lines);
    assert!(fixture.root.join("node/26.11.0").exists());
    assert!(fixture.root.join("aws/2.40.2").exists());
}

/// The control the brief asks for:
/// a version a running process of the test holds open stays.
#[test]
fn keeps_a_version_a_running_process_holds_open() {
    let fixture = Fixture::new();
    let held_version: PathBuf = fixture.root.join("node/26.11.0");
    let opened: fs::File = fs::File::open(held_version.join("program")).expect("open a file of the oldest version");
    let holders = crate::toolchain::store_path_holders::holders_of(std::slice::from_ref(&fixture.root), Instant::now() + Duration::from_secs(60));
    // Control: the scan really sees this process's open file, so the test cannot pass on an empty scan.
    let reason = crate::toolchain::store_path_holders::held_reason(&holders, &held_version);
    assert!(reason.is_some(), "the scan found no holder of the held version: {holders:?}");
    let plan: TrimPlan = plan_trim(&fixture.candidates(), &fixture.run().bounds, SystemTime::now(), &|found| {
        return crate::toolchain::store_path_holders::held_reason(&holders, &found.directory)
            .map(|held| return format!("it is in use: {held}"));
    });

    let outcome = apply_trim(&plan, &fixture.run());

    assert!(held_version.join("program").exists(), "the held version was removed");
    assert!(outcome.lines.iter().any(|line| return line.contains("node/26.11.0") && line.contains("it is in use")));
    assert!(!fixture.root.join("node/26.12.0").exists());
    drop(opened);
}

/// The control the brief asks for:
/// a removal cut short after the version was moved out of its name is finished by the next pass,
/// and the version is never there half removed.
#[test]
fn finishes_a_removal_cut_short() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let marker: PathBuf = tombstone_of(&version);
    // The fault point: exactly what a kill between the rename and the delete leaves.
    fs::rename(&version, &marker).expect("move the version aside");
    let files_before: usize = fs::read_dir(&marker).expect("list the marker").count();
    assert_eq!(files_before, 1);
    let tools: BTreeSet<String> = ["node".to_string(), "aws".to_string()].into_iter().collect();

    assert!(!tools.is_empty());
    let finished = finish_removals(&fixture.run());
    let lines: Vec<String> = finished.lines;

    assert!(finished.unfinished.is_empty(), "{:?}", finished.unfinished);
    assert!(!marker.exists(), "the marker is still there");
    assert!(!version.exists());
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains(&format!("/node/26.11.0 (")), "the line names the version's own path: {lines:?}");
    assert!(lines[0].contains("finished a removal an earlier run started"), "{lines:?}");
}

/// The removal marker is what makes the next install run the pass,
/// and it goes once nothing is left to remove.
#[test]
fn settles_the_marker_of_an_unfinished_removal() {
    let fixture = Fixture::new();
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the marker");
    assert!(has_unfinished_removal(&fixture.root));

    settle_markers(std::slice::from_ref(&fixture.root), &BTreeSet::new());

    assert!(!has_unfinished_removal(&fixture.root));
}

/// A marker is not cleared while a removal is still pending,
/// so the cleanup is not forgotten with a version half removed.
#[test]
fn keeps_the_marker_while_a_removal_is_pending() {
    let fixture = Fixture::new();
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the marker");
    let pending: BTreeSet<PathBuf> = [fixture.root.clone()].into_iter().collect();

    settle_markers(std::slice::from_ref(&fixture.root), &pending);

    assert!(has_unfinished_removal(&fixture.root));
}

/// A version another run wrote after the scan is not the one that was judged, so it stays.
#[test]
fn keeps_a_version_another_run_wrote_after_the_scan() {
    let fixture = Fixture::new();
    let candidates: Vec<TrimCandidate> = fixture.candidates();
    let judged: TrimCandidate = candidates
        .iter()
        .find(|found| return found.version == "26.11.0")
        .expect("the oldest version is scanned")
        .clone();
    // What: another run adds a file to the version, which is what moving a new build in looks like.
    // Why: a version directory's write time changes when an entry arrives, and that is the change this guard reads.
    fs::write(fixture.root.join("node/26.11.0/extra"), vec![b'b'; 10]).expect("add a file to the version");
    let plan: TrimPlan = TrimPlan { total_bytes: judged.bytes, over_bound: true, steps: vec![TrimStep::Remove(judged)] };

    let outcome = apply_trim(&plan, &fixture.run());

    assert_eq!(outcome.removed, 0);
    assert!(fixture.root.join("node/26.11.0/program").exists());
    assert!(outcome.lines.iter().any(|line| return line.contains("another run wrote it")));
}

/// A version's downloads go with it,
/// and the tool's downloads directory goes once its last version's archives are gone.
#[test]
fn removes_the_downloads_of_the_version_it_removes() {
    let fixture = Fixture::new();
    let downloads: PathBuf = fixture.root.join("downloads/node/26.11.0");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    fs::write(downloads.join("node.tar.gz"), vec![b'z'; 40]).expect("write the archive");
    let candidates: Vec<TrimCandidate> = fixture.candidates();
    let judged: TrimCandidate = candidates
        .iter()
        .find(|found| return found.version == "26.11.0")
        .expect("the oldest version is scanned")
        .clone();
    assert!(judged.downloads.is_some(), "the scan did not find the downloads");
    let plan: TrimPlan = TrimPlan { total_bytes: judged.bytes, over_bound: true, steps: vec![TrimStep::Remove(judged)] };

    let outcome = apply_trim(&plan, &fixture.run());

    assert_eq!(outcome.removed, 1);
    assert!(!downloads.exists());
    assert!(!fixture.root.join("downloads/node").exists());
}

//endregion the removal

//region the removals an earlier run left, and a process that starts mid-pass

/// What:
/// A removal an earlier run cut short is finished even when the worktrees' locks cannot be read.
/// Why:
/// The version is condemned already, and the marker is the only record that its space is not back:
/// a pass that cannot see any tool name must still find the marker in the store and finish it,
/// or the marker goes and the space is never taken back.
#[test]
fn finishes_a_removal_cut_short_even_when_the_worktrees_cannot_be_listed() {
    let (_directory, scope, env) = crate::toolchain::fixtures::sample_scope_on_disk();
    let store: PathBuf = scope.toolchain_root.clone();
    let tool: PathBuf = store.join("t");
    let version_marker: PathBuf = tool.join(format!("{TOMBSTONE_PREFIX}1.0.0"));
    fs::create_dir_all(&version_marker).expect("make the removal marker");
    fs::write(version_marker.join("program"), vec![b'a'; 1000]).expect("write the version's file");
    fs::write(unfinished_marker(&store), b"").expect("write the unfinished-removal marker");
    // The fault this control exists for: the listing of worktrees fails, so no lock name is known.
    let runner = crate::toolchain::fakes::FakeRunner::new();
    runner.answer("git -C", "", 128);
    let host = crate::toolchain::context::Host::current();
    let context = crate::toolchain::context::Context {
        scope: &scope,
        env: &env,
        runner: &runner,
        http: &crate::toolchain::http::NoNetwork,
        host: &host,
        tools: &[],
    };

    crate::toolchain::store_trim::trim_after_install(&context, &[]);

    assert!(!version_marker.exists(), "the removal an earlier run started was not finished");
    assert!(!store.join("t/1.0.0").exists(), "the version came back although nothing holds it");
    assert!(!has_unfinished_removal(&store), "the marker outlived the removal it stood for");
    // A later install that installs nothing must find nothing left to do, and the store must stay settled.
    crate::toolchain::store_trim::trim_after_install(&context, &[]);
    assert!(!has_unfinished_removal(&store));
    assert!(crate::toolchain::store_versions::removal_markers(&store).is_empty());
}

/// What:
/// A version a process starts using after the pass's look at the process table and before its removal stays,
/// and it comes back under its own name.
/// Why:
/// The version is moved out of its name before anything is deleted, and the process table is asked again once it
/// is: nothing can start using a version after its rename, so that second look is the moment the removal becomes
/// final, and a version something holds is put back.
#[test]
fn keeps_a_version_a_process_starts_in_between_the_look_and_the_removal() {
    let fixture = Fixture::new();
    let run: TrimRun = fixture.run();
    let victim: PathBuf = fixture.root.join("node/26.11.0");
    // What: the pass's own first look, taken before any candidate is judged.
    // Why: this is the seam the defect lives in, so the control drives the production function and then starts the
    // process exactly between the look and the removal.
    let first_look: StoreHolders = run.look();
    assert!(held_reason(&first_look, &victim).is_none(), "the fixture must be unused when the look is taken");
    let plan: TrimPlan = plan_trim(&fixture.candidates(), &run.bounds, SystemTime::now(), &|found| {
        return held_reason(&first_look, &found.directory).map(|held| return format!("it is in use: {held}"));
    });
    let planned: Vec<String> = removed_by(&plan);
    assert!(planned.iter().any(|name| return name == "node-26.11.0"), "the victim must be planned for removal: {planned:?}");

    let mut child = std::process::Command::new("sleep")
        .arg("60")
        .current_dir(&victim)
        .spawn()
        .expect("start a process working inside the version");

    let outcome: TrimOutcome = apply_trim(&plan, &run);
    // What: the child is ended before anything is asserted.
    // Why: a failing assertion must not leak a process that outlives the test.
    let cwd: PathBuf = fs::read_link(format!("/proc/{}/cwd", child.id())).expect("read the child's working directory");
    let _ = child.kill();
    let _ = child.wait();

    // What: the version is back under its own name and the line says why.
    // Why: a kept version that stays a removal marker is not the same store state, and a dangling cwd is the harm
    // this control is about.
    assert!(victim.join("program").exists(), "the version a process started in was removed: {:?}", outcome.lines);
    assert!(!cwd.to_string_lossy().contains("(deleted)"), "the child's working directory was deleted: {}", cwd.display());
    assert!(
        outcome.lines.iter().any(|line| return line.contains("node/26.11.0") && line.contains("it is in use")),
        "{:?}",
        outcome.lines
    );
    assert_eq!(outcome.removed, 3, "{:?}", outcome.lines);
}

//endregion the removals an earlier run left, and a process that starts mid-pass

/// What:
/// A removal put back because something holds it is deleted when the version's name holds another copy by then,
/// and no marker is left behind for the next pass to retry forever.
/// Why:
/// Another run may install the same version while a pass holds the old copy out of its name;
/// restoring the old copy over the new one would discard the install,
/// and leaving the marker would make every later pass retry a restore that cannot succeed.
#[test]
fn deletes_a_held_copy_whose_name_another_run_took() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let marker: PathBuf = tombstone_of(&version);
    // What: the state a pass leaves when the version was moved aside and the run was killed before the delete,
    // and a newer copy of the same version was installed afterwards.
    fs::rename(&version, &marker).expect("move the version aside");
    fs::create_dir_all(&version).expect("install the newer copy");
    fs::write(version.join("program"), b"fresh copy").expect("write the newer copy");
    let mut held = std::process::Command::new("sleep")
        .arg("60")
        .current_dir(&marker)
        .spawn()
        .expect("start a process working inside the moved-aside copy");

    let finished = finish_removals(&fixture.run());
    let _ = held.kill();
    let _ = held.wait();

    assert!(version.join("program").exists(), "{:?}", finished.lines);
    assert_eq!(fs::read(version.join("program")).expect("read the newer copy"), b"fresh copy");
    assert!(!marker.exists(), "the held copy was restored over the newer one: {:?}", finished.lines);
    assert!(finished.unfinished.is_empty(), "a removal is still pending: {:?}", finished.lines);
    assert!(
        finished.lines.iter().any(|line| return line.contains("a newer copy holds")),
        "{:?}",
        finished.lines
    );
    assert!(crate::toolchain::store_versions::removal_markers(&fixture.root).is_empty());
}

//region the path spelling, a live version's downloads, and a bound at the largest number

/// Gives `path` a write time `seconds` before now.
fn written_ago(path: &std::path::Path, seconds: u64) {
    let when: SystemTime = SystemTime::now() - Duration::from_secs(seconds);
    fs::File::open(path)
        .expect("open the version directory")
        .set_times(FileTimes::new().set_modified(when))
        .expect("set the version's write time");
}

/// What:
/// A version a live process uses is kept when the store's path has a symbolic link in it.
/// Why:
/// `HOME` spells the store, and on this machine `/home` is a link to `var/home`,
/// while the process table reports resolved paths:
/// a comparison of the two spellings keeps nothing,
/// and a version a process runs was removed under it.
#[test]
fn keeps_a_version_a_process_uses_when_the_store_path_is_a_symlink() {
    let directory = scratch();
    let real: PathBuf = directory.path().join("real");
    for (version, age) in [("26.11.0", 2 * 24 * 3600), ("26.13.1", 60)] {
        let version_directory: PathBuf = real.join("store/node").join(version);
        fs::create_dir_all(&version_directory).expect("make a version directory");
        fs::write(version_directory.join("program"), vec![b'a'; 1000]).expect("write the program");
        written_ago(&version_directory, age);
    }
    let link: PathBuf = directory.path().join("link");
    std::os::unix::fs::symlink(&real, &link).expect("link the real directory");
    // What: the store is reached through a link, the way `HOME` reaches it on this machine.
    // Why: the candidate paths the rule builds then carry the link's spelling.
    let store: PathBuf = link.join("store");
    let victim: PathBuf = store.join("node/26.11.0");
    let mut child = std::process::Command::new("sleep")
        .arg("60")
        .current_dir(&victim)
        .spawn()
        .expect("start a process working inside the version");
    let run: TrimRun = TrimRun {
        bounds: TrimBounds { size_bound_bytes: 0, age_floor: Duration::ZERO, work_limit: Duration::from_secs(60) },
        started: Instant::now(),
        roots: vec![store.clone()],
    };
    let look: StoreHolders = run.look();
    let held: Option<String> = held_reason(&look, &victim);

    let tools: std::collections::BTreeSet<String> = ["node".to_string()].into_iter().collect();
    let plan: TrimPlan = plan_trim(&candidates_of(std::slice::from_ref(&store), &tools), &run.bounds, SystemTime::now(), &|found| {
        return held_reason(&look, &found.directory).map(|reason| return format!("it is in use: {reason}"));
    });
    let outcome: TrimOutcome = apply_trim(&plan, &run);
    let _ = child.kill();
    let _ = child.wait();

    assert!(held.is_some(), "the look found no holder of a version a process runs: {look:?}");
    assert!(victim.join("program").exists(), "the version a process runs was removed: {:?}", outcome.lines);
    assert!(
        outcome.lines.iter().any(|line| return line.contains("node/26.11.0") && line.contains("it is in use")),
        "{:?}",
        outcome.lines
    );
}

/// What:
/// The archives of a version that is still there are put back when a removal was cut short after moving them aside.
/// Why:
/// `move_aside` moves the downloads aside before the version and `put_back` restores the version before them,
/// so a kill between the two leaves a downloads marker and a live version;
/// reading that as "condemned" deleted the archives of a version every protection keeps.
#[test]
fn puts_back_the_downloads_of_a_version_that_is_still_there() {
    let fixture = Fixture::new();
    let downloads: PathBuf = fixture.root.join("downloads/node/26.11.0");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    fs::write(downloads.join("node.tar.gz"), vec![b'z'; 40]).expect("write the archive");
    let marker: PathBuf = tombstone_of(&downloads);
    // The state a kill between the two renames leaves: downloads moved aside, version still under its name.
    fs::rename(&downloads, &marker).expect("move the downloads aside");
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the unfinished-removal marker");

    let finished = finish_removals(&fixture.run());

    assert!(
        downloads.join("node.tar.gz").exists(),
        "the archives of a version that is still there were deleted: {:?}",
        finished.lines
    );
    assert!(!marker.exists(), "the downloads marker outlived the removal it stood for: {:?}", finished.lines);
    assert!(finished.unfinished.is_empty(), "{:?}", finished.lines);
    assert!(
        finished.lines.iter().any(|line| return line.contains("kept") && line.contains("still there")),
        "{:?}",
        finished.lines
    );
    assert!(removal_markers(&fixture.root).is_empty());
}

/// What:
/// An age floor at the largest number is read as the longest floor the pass allows, and nothing panics.
/// Why:
/// The floor was subtracted from the current time for its line,
/// and `18446744073709551615` seconds overflowed, which panicked `install` (exit 101) out of a commit hook.
#[test]
fn keeps_a_version_when_the_age_floor_is_the_largest_number() {
    let env: EnvMap = EnvMap::from_pairs(&[("MPM_SHADOW_STORE_AGE_FLOOR", "18446744073709551615")]);
    // The size bound is taken out of the way so the age floor is what the rule reads.
    let bounds: TrimBounds = TrimBounds { size_bound_bytes: 0, ..TrimBounds::from_env(&env) };
    let candidates = vec![candidate("node", "26.12.0", MIB, 40 * 3600)];

    // What: the rule is asked first.
    // Why: subtracting this floor from the current time is what panicked, so the panic has to be what a broken run
    // meets before any assertion about the clamped value.
    let plan: TrimPlan = plan_trim(&candidates, &bounds, SystemTime::now(), &|_found| return None);

    let kept: Vec<String> = kept_by(&plan).into_iter().map(|(name, _reason)| return name).collect();
    assert_eq!(kept, vec!["node-26.12.0".to_string()]);
    assert_eq!(format_span(bounds.age_floor), "36500 d");
    assert_eq!(bounds.age_floor, MAX_AGE_FLOOR);
    // A count that does not fit the multiplier is the longest span, not a truncated one.
    assert_eq!(parse_duration("18446744073709551615ms"), Some(Duration::MAX));
    assert_eq!(
        TrimBounds::from_env(&EnvMap::from_pairs(&[("MPM_SHADOW_STORE_WORK_LIMIT", "18446744073709551615ms")]))
            .work_limit,
        MAX_WORK_LIMIT
    );
}

/// What:
/// A work limit at the largest number is read as the longest limit the pass allows, and the deadline is finite.
/// Why:
/// The limit was added to `Instant::now` and overflowed, panicking the whole install.
#[test]
fn keeps_the_deadline_when_the_work_limit_is_the_largest_number() {
    let env: EnvMap = EnvMap::from_pairs(&[("MPM_SHADOW_STORE_WORK_LIMIT", "18446744073709551615")]);
    let bounds: TrimBounds = TrimBounds::from_env(&env);

    let run: TrimRun = TrimRun { bounds, started: Instant::now(), roots: vec![PathBuf::from("/store")] };

    // What: the deadline is asked first.
    // Why: adding this limit to the start is what panicked, so the panic has to be what a broken run meets before any
    // assertion about the clamped value.
    let deadline: Instant = run.deadline();

    assert!(deadline > run.started, "the deadline is not after the start");
    assert!(deadline <= run.started + MAX_WORK_LIMIT);
    assert_eq!(run.bounds.work_limit, MAX_WORK_LIMIT);
}

//endregion the path spelling, a live version's downloads, and a bound at the largest number

//region every path a version owns, and the link that names it

/// What:
/// Starts a process that holds `path` open, and returns it.
/// Why:
/// The promise under test is about a file a live process has open,
/// and a child whose standard input is that file holds the only descriptor of it.
fn holder_of(path: &std::path::Path) -> std::process::Child {
    let file: std::fs::File = fs::File::open(path).expect("open the file a process will hold");
    return std::process::Command::new("sleep")
        .arg("60")
        .stdin(std::process::Stdio::from(file))
        .spawn()
        .expect("start a process holding the file");
}

/// Gives the fixture's tool a `current` link at `version`.
fn point_current(root: &std::path::Path, tool: &str, version: &str) {
    let link: PathBuf = root.join(tool).join("current");
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink(version, &link).expect("point the current link");
}

/// The version a tool's `current` link names, as the pass's snapshot reads it.
fn current_snapshot(root: &std::path::Path, tool: &str) -> String {
    let target: PathBuf = fs::read_link(root.join(tool).join("current")).expect("read the current link");
    return target.file_name().expect("a name").to_string_lossy().to_string();
}

/// What:
/// A version whose downloads a live process holds open is kept, through the pass a sync runs.
/// Why:
/// The cleanup promises it never removes a version a running process uses,
/// and a version owns two trees: its directory and the archives downloaded for it.
/// A hold on the archive used to be invisible to every check, and the install that was downloading it failed.
#[test]
fn keeps_a_version_whose_downloads_a_process_holds_open() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let store: PathBuf = scope.toolchain_root.clone();
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"26.13.1","route":"archive"}}}"#)
        .expect("write a lock naming the tool");
    let stale: PathBuf = store.join("node/26.12.0");
    fs::create_dir_all(&stale).expect("make the old version");
    fs::write(stale.join("program"), vec![b'a'; 1000]).expect("write the program");
    written_ago(&stale, 2 * 24 * 3600);
    let downloads: PathBuf = store.join("downloads/node/26.12.0");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    fs::write(downloads.join("node.tar.gz"), vec![b'z'; 40]).expect("write the archive");
    fs::write(unfinished_marker(&store), b"").expect("write the unfinished-removal marker");
    let mut child = holder_of(&downloads.join("node.tar.gz"));
    let runner = FakeRunner::new();
    let host = Host::current();
    // The pass reads its bounds from the environment it is given.
    let env_holding: EnvMap = env
        .with("MPM_SHADOW_STORE_SIZE_BOUND", "0")
        .with("MPM_SHADOW_STORE_AGE_FLOOR", "0");
    let context = Context { scope: &scope, env: &env_holding, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };

    trim_after_install(&context, &[]);
    let _ = child.kill();
    let _ = child.wait();

    assert!(stale.join("program").exists(), "the version a process holds its archive of was removed");
    assert!(downloads.join("node.tar.gz").exists(), "the archive a live process holds open was removed");
}

/// What:
/// A version whose downloads are held open is kept when it is already moved aside, and both come back.
/// Why:
/// The removal's second look asks about the same trees the plan does;
/// a hold on the archives used to be answered by neither, and the archive was deleted.
#[test]
fn keeps_its_downloads_when_a_moved_aside_version_is_held() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let downloads: PathBuf = fixture.root.join("downloads/node/26.11.0");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    fs::write(downloads.join("node.tar.gz"), vec![b'z'; 40]).expect("write the archive");
    let plan: TrimPlan = plan_trim(&fixture.candidates(), &fixture.run().bounds, SystemTime::now(), &|_found| {
        return None;
    });
    let mut child = holder_of(&downloads.join("node.tar.gz"));

    let outcome: TrimOutcome = apply_trim(&plan, &fixture.run());
    let _ = child.kill();
    let _ = child.wait();

    assert!(version.join("program").exists(), "the version was removed: {:?}", outcome.lines);
    assert!(downloads.join("node.tar.gz").exists(), "the archive a live process holds was removed: {:?}", outcome.lines);
    assert!(
        outcome.lines.iter().any(|line| return line.contains("node/26.11.0") && line.contains("it is in use")),
        "{:?}",
        outcome.lines
    );
}

/// What:
/// A removal an earlier run left, whose archives a live process holds, is undone and the archives come back.
/// Why:
/// The finish pass asks about the paths a version owns,
/// and a hold on the archives marker used to be answered by neither of its two checks.
#[test]
fn puts_back_a_held_marker_and_its_downloads() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let downloads: PathBuf = fixture.root.join("downloads/node/26.11.0");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    fs::write(downloads.join("node.tar.gz"), vec![b'z'; 40]).expect("write the archive");
    let downloads_marker: PathBuf = tombstone_of(&downloads);
    let version_marker: PathBuf = tombstone_of(&version);
    fs::rename(&downloads, &downloads_marker).expect("move the downloads aside");
    fs::rename(&version, &version_marker).expect("move the version aside");
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the unfinished-removal marker");
    let mut child = holder_of(&downloads_marker.join("node.tar.gz"));

    let finished = finish_removals(&fixture.run());
    let _ = child.kill();
    let _ = child.wait();

    assert!(version.join("program").exists(), "the version was not put back: {:?}", finished.lines);
    assert!(
        downloads.join("node.tar.gz").exists(),
        "the archives a live process holds were not put back: {:?}",
        finished.lines
    );
    assert!(!downloads_marker.exists() && !version_marker.exists(), "{:?}", finished.lines);
    assert!(
        finished.lines.iter().any(|line| return line.contains("it is in use") && line.contains("undone")),
        "{:?}",
        finished.lines
    );
}

/// What:
/// A version the tool's `current` link came to name after the pass's snapshot is kept.
/// Why:
/// Another command can repoint `current` between the pass's read and its removal,
/// and a version the link names at that moment must never be removed:
/// a link left naming a deleted version dangles every worktree's exposed link.
#[test]
fn keeps_a_version_current_came_to_name_before_its_removal() {
    let fixture = Fixture::new();
    let victim: PathBuf = fixture.root.join("node/26.11.0");
    point_current(&fixture.root, "node", "26.13.1");
    let snapshot: String = current_snapshot(&fixture.root, "node");
    let plan: TrimPlan = plan_trim(&fixture.candidates(), &fixture.run().bounds, SystemTime::now(), &|found| {
        if found.tool == "node" && found.version == snapshot {
            return Some("the tool's current link names it".to_string());
        }
        return None;
    });
    assert!(removed_by(&plan).iter().any(|name| return name == "node-26.11.0"), "{:?}", removed_by(&plan));
    // What: the stand-in for another command repointing the link between the snapshot and the removal.
    point_current(&fixture.root, "node", "26.11.0");

    let outcome: TrimOutcome = apply_trim(&plan, &fixture.run());

    assert!(victim.join("program").exists(), "the version current came to name was removed: {:?}", outcome.lines);
    assert!(
        outcome.lines.iter().any(|line| return line.contains("node/26.11.0") && line.contains("current link names it")),
        "{:?}",
        outcome.lines
    );
    let named: String = current_snapshot(&fixture.root, "node");
    assert!(fixture.root.join("node").join(named).is_dir(), "the current link names a version that is gone");
}

/// What:
/// A removal an earlier run left, whose version the `current` link came to name, is undone.
/// Why:
/// The finish pass runs before any lock is read and used not to look at the link at all,
/// so a marker the link named was deleted and the link dangled.
#[test]
fn puts_back_a_marker_whose_version_current_names() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let marker: PathBuf = tombstone_of(&version);
    fs::rename(&version, &marker).expect("move the version aside");
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the unfinished-removal marker");
    point_current(&fixture.root, "node", "26.11.0");

    let finished = finish_removals(&fixture.run());

    assert!(version.join("program").exists(), "the version current names was not put back: {:?}", finished.lines);
    assert!(!marker.exists(), "{:?}", finished.lines);
    assert!(
        finished.lines.iter().any(|line| return line.contains("current link names it") && line.contains("undone")),
        "{:?}",
        finished.lines
    );
}

//endregion every path a version owns, and the link that names it

//region replaying a real store's listing

/// One version directory of the replayed listing.
struct ListedVersion {
    /// The store's directory name for the tool.
    tool: String,
    /// The version.
    version: String,
    /// The size the version and the downloads kept for it weigh.
    bytes: u64,
    /// When the version directory was written.
    written: SystemTime,
    /// Whether a lock of the replayed machine named it.
    pinned: bool,
}

/// One download file an earlier layout left under the store's `downloads` directory.
struct ListedLeftover {
    /// The tool the file's name belongs to.
    tool: String,
    /// The file's name.
    name: String,
    /// The size it weighs.
    bytes: u64,
    /// When it was written.
    written: SystemTime,
}

/// The system time `milliseconds` after the epoch.
fn when(milliseconds: i64) -> SystemTime {
    let whole_seconds: i64 = milliseconds.div_euclid(1000);
    let remainder: i64 = milliseconds.rem_euclid(1000);
    let seconds: u64 = u64::try_from(whole_seconds).expect("a positive time");
    return SystemTime::UNIX_EPOCH + Duration::from_secs(seconds) + Duration::from_millis(remainder as u64);
}

/// One store root of the replayed listing.
struct ListedRoot {
    /// The versions of the root.
    versions: Vec<ListedVersion>,
    /// The download files an earlier layout left in the root.
    leftovers: Vec<ListedLeftover>,
}

/// The listing `MPM_SHADOW_STORE_LISTING` names, decoded,
/// one entry per store root,
/// because a tool and version can be in both roots and the two are judged separately.
fn listed(text: &str) -> Vec<ListedRoot> {
    let decoded: serde_json::Value = serde_json::from_str(text).expect("the listing is JSON");
    let mut roots: Vec<ListedRoot> = Vec::new();
    for root in decoded["roots"].as_array().expect("roots") {
        let mut versions: Vec<ListedVersion> = Vec::new();
        let mut leftovers: Vec<ListedLeftover> = Vec::new();
        for version in root["versions"].as_array().expect("versions") {
            versions.push(ListedVersion {
                tool: version["tool"].as_str().expect("tool").to_string(),
                version: version["version"].as_str().expect("version").to_string(),
                bytes: version["bytes"].as_u64().expect("bytes"),
                written: when(version["writtenMs"].as_i64().expect("writtenMs")),
                pinned: version["pinned"].as_bool().expect("pinned"),
            });
        }
        for leftover in root["leftovers"].as_array().expect("leftovers") {
            leftovers.push(ListedLeftover {
                tool: leftover["tool"].as_str().expect("tool").to_string(),
                name: leftover["name"].as_str().expect("name").to_string(),
                bytes: leftover["bytes"].as_u64().expect("bytes"),
                written: when(leftover["writtenMs"].as_i64().expect("writtenMs")),
            });
        }
        roots.push(ListedRoot { versions, leftovers });
    }
    return roots;
}

/// What:
/// Replays a machine's store listing:
/// builds a disposable store whose version directories weigh exactly what the listing recorded,
/// judges it with the bounds the pass ships,
/// and writes what it would remove and what it keeps to `MPM_SHADOW_STORE_LISTING_REPORT`.
/// Why:
/// The rule and its bounds are sized from one real store,
/// so they must be shown deciding that store's listing rather than a fixture invented for them.
/// Skipped when the variable is unset:
/// the listing is a measurement of one machine and is not part of the package.
#[test]
fn replays_a_real_store_listing() {
    let Ok(listing_path) = std::env::var("MPM_SHADOW_STORE_LISTING") else {
        return;
    };
    let text: String = fs::read_to_string(&listing_path).expect("read the listing");
    let listing: Vec<ListedRoot> = listed(&text);
    let fixture = scratch();
    let mut replayed_roots: Vec<PathBuf> = Vec::new();
    let mut tools: BTreeSet<String> = BTreeSet::new();
    for (index, listed_root) in listing.iter().enumerate() {
        let root: PathBuf = fixture.path().join(format!("toolchain-{index}"));
        replayed_roots.push(root.clone());
        for version in &listed_root.versions {
            tools.insert(version.tool.clone());
            let directory: PathBuf = root.join(&version.tool).join(&version.version);
            fs::create_dir_all(&directory).expect("make a replayed version directory");
            // Sparse: the length is what the rule measures, and the replayed listing is what decides the outcome.
            fs::File::create(directory.join("payload")).expect("make the payload").set_len(version.bytes).expect("size it");
            fs::File::open(&directory)
                .expect("open the directory")
                .set_times(FileTimes::new().set_modified(version.written))
                .expect("set the write time");
        }
        for leftover in &listed_root.leftovers {
            tools.insert(leftover.tool.clone());
            let directory: PathBuf = root.join("downloads");
            fs::create_dir_all(&directory).expect("make the downloads directory");
            let file: PathBuf = directory.join(&leftover.name);
            fs::File::create(&file).expect("make the leftover").set_len(leftover.bytes).expect("size it");
            fs::File::options()
                .write(true)
                .open(&file)
                .expect("open the leftover")
                .set_times(FileTimes::new().set_modified(leftover.written))
                .expect("set its write time");
        }
    }
    // What: the size bound can be set for a measurement, and is the shipped one otherwise.
    // Why: an arm that measures a pass with nothing to remove needs a bound above the replay's total,
    // and the same test has to be the one that measures the removals.
    let bounds: TrimBounds = TrimBounds {
        size_bound_bytes: std::env::var("MPM_SHADOW_REPLAY_SIZE_BOUND")
            .ok()
            .and_then(|given| return parse_size(&given))
            .unwrap_or(DEFAULT_SIZE_BOUND_BYTES),
        age_floor: Duration::from_secs(DEFAULT_AGE_FLOOR_SECONDS),
        work_limit: Duration::from_secs(24 * 3600),
    };
    let pinned: BTreeSet<String> = listing
        .iter()
        .flat_map(|listed_root| return listed_root.versions.iter())
        .filter(|version| return version.pinned)
        .map(|version| return format!("{}-{}", version.tool, version.version))
        .collect();
    // What: the pass is timed the way a real one runs: one look at the process table, the inventory, the judge,
    // then the removals.
    // Why: the second look the removals take is what the review's second finding bought, so its cost has to be
    // visible next to the rest of a pass.
    let run: TrimRun = TrimRun { bounds, started: Instant::now(), roots: replayed_roots.clone() };
    let look_clock: Instant = Instant::now();
    let planning_look: StoreHolders = run.look();
    let look_ms: u128 = look_clock.elapsed().as_millis();
    let inventory_clock: Instant = Instant::now();
    let candidates: Vec<TrimCandidate> = candidates_of(&replayed_roots, &tools);
    let inventory_ms: u128 = inventory_clock.elapsed().as_millis();
    let judge_clock: Instant = Instant::now();
    let plan: TrimPlan = plan_trim(&candidates, &bounds, SystemTime::now(), &|found| {
        if pinned.contains(&format!("{}-{}", found.tool, found.version)) {
            return Some("a lock of the replayed machine named it".to_string());
        }
        return held_reason(&planning_look, &found.directory).map(|held| return format!("it is in use: {held}"));
    });
    let judge_ms: u128 = judge_clock.elapsed().as_millis();
    let mut lines: Vec<String> = vec![
        format!("replayed version directories and leftover downloads: {}", candidates.len()),
        format!("their total: {} MiB; the size bound: {} MiB", plan.total_bytes / MIB, bounds.size_bound_bytes / MIB),
        format!("over the bound: {}", plan.over_bound),
    ];
    let mut planned_removal: u64 = 0;
    for step in &plan.steps {
        match step {
            TrimStep::Remove(found) => {
                planned_removal += found.bytes;
                lines.push(format!("would remove {} ({} MiB)", found.directory.display(), found.bytes / MIB));
            }
            TrimStep::Keep(found, reason) => {
                lines.push(format!("kept {} ({} MiB) because {reason}", found.directory.display(), found.bytes / MIB));
            }
        }
    }
    lines.push(format!("planned removal: {} MiB", planned_removal / MIB));
    let removal_run: TrimRun = TrimRun { bounds, started: Instant::now(), roots: replayed_roots.clone() };
    let removal_clock: Instant = Instant::now();
    let outcome: TrimOutcome = apply_trim(&plan, &removal_run);
    let remove_ms: u128 = removal_clock.elapsed().as_millis();
    lines.push(format!(
        "cost: look {look_ms} ms, inventory {inventory_ms} ms, judge {judge_ms} ms, removals {remove_ms} ms"
    ));
    lines.push(format!(
        "removed from the replayed store: {} version(s), {} MiB",
        outcome.removed,
        outcome.removed_bytes / MIB
    ));
    let report: String = std::env::var("MPM_SHADOW_STORE_LISTING_REPORT").expect("the report path");
    fs::write(&report, format!("{}\n", lines.join("\n"))).expect("write the replay report");
}

//endregion replaying a real store's listing
