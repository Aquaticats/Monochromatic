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
use crate::toolchain::store_versions::{TOMBSTONE_PREFIX, tombstone_of};
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
    assert!(planned.contains(&"node-26.11.0".to_string()), "the victim must be planned for removal: {planned:?}");

    let mut child = std::process::Command::new("sleep")
        .arg("60")
        .current_dir(&victim)
        .spawn()
        .expect("start a process working inside the version");

    let outcome: TrimOutcome = apply_trim(&plan, &run);

    // What: the version is back under its own name and the line says why.
    // Why: a kept version that stays a removal marker is not the same store state, and a dangling cwd is the harm
    // this control is about.
    assert!(victim.join("program").exists(), "the version a process started in was removed: {:?}", outcome.lines);
    let cwd: PathBuf = fs::read_link(format!("/proc/{}/cwd", child.id())).expect("read the child's working directory");
    assert!(!cwd.to_string_lossy().contains("(deleted)"), "the child's working directory was deleted: {}", cwd.display());
    assert!(
        outcome.lines.iter().any(|line| return line.contains("node/26.11.0") && line.contains("it is in use")),
        "{:?}",
        outcome.lines
    );
    assert_eq!(outcome.removed, 3, "{:?}", outcome.lines);
    let _ = child.kill();
    let _ = child.wait();
}

//endregion the removals an earlier run left, and a process that starts mid-pass

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
    let bounds: TrimBounds = TrimBounds {
        // The bounds the pass ships, with the work limit out of the way: the replay asks what the rule decides, while
        // an install only does as much of it as its time limit allows.
        size_bound_bytes: DEFAULT_SIZE_BOUND_BYTES,
        age_floor: Duration::from_secs(DEFAULT_AGE_FLOOR_SECONDS),
        work_limit: Duration::from_secs(24 * 3600),
    };
    let pinned: BTreeSet<String> = listing
        .iter()
        .flat_map(|listed_root| return listed_root.versions.iter())
        .filter(|version| return version.pinned)
        .map(|version| return format!("{}-{}", version.tool, version.version))
        .collect();
    let candidates: Vec<TrimCandidate> = candidates_of(&replayed_roots, &tools);
    let plan: TrimPlan = plan_trim(&candidates, &bounds, SystemTime::now(), &|found| {
        if pinned.contains(&format!("{}-{}", found.tool, found.version)) {
            return Some("a lock of the replayed machine named it".to_string());
        }
        return None;
    });
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
    let outcome: TrimOutcome = apply_trim(&plan, &TrimRun { bounds, started: Instant::now(), roots: replayed_roots.clone() });
    lines.push(format!(
        "removed from the replayed store: {} version(s), {} MiB",
        outcome.removed,
        outcome.removed_bytes / MIB
    ));
    let report: String = std::env::var("MPM_SHADOW_STORE_LISTING_REPORT").expect("the report path");
    fs::write(&report, format!("{}\n", lines.join("\n"))).expect("write the replay report");
}

//endregion replaying a real store's listing
