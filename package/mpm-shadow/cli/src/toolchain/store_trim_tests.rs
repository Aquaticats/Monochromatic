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
use std::path::{Path, PathBuf};
/// Imports `Duration`,
/// `Instant`,
/// `SystemTime` from `std::time`.
use std::time::{Duration, Instant, SystemTime};

/// Imports the module under test.
use crate::toolchain::store_trim::{
    DEFAULT_SIZE_BOUND_BYTES, TrimBounds, TrimCandidate, TrimPlan, TrimRun, TrimStep, apply_trim, candidates_of,
    DEFAULT_AGE_FLOOR_SECONDS, TrimOutcome, finish_removals, format_age, has_unfinished_removal, parse_duration,
    parse_size, plan_trim, read_named_versions, settle_markers, unfinished_marker,
};
use crate::toolchain::http::NoNetwork;
use crate::toolchain::published_archive::is_finished;
use crate::toolchain::store_trim::{MAX_AGE_FLOOR, MAX_WORK_LIMIT, format_span, trim_after_install_with};
use crate::toolchain::store_path_holders::OpaqueProcess;
use crate::toolchain::store_trim::finish_removals_with;
use crate::toolchain::store_versions::{TOMBSTONE_PREFIX, judged_of_tree, record_of, removal_markers, tombstone_of, write_judged};
use crate::toolchain::context::{Context, Host};
use crate::toolchain::fakes::{FakeHttp, FakeRunner};
use crate::toolchain::fixtures::{sample_lock, sample_scope_on_disk};
use crate::toolchain::runner::Ceiling;
use crate::toolchain::scope::{EnvMap, Scope, global_scope};
use crate::toolchain::store_path_holders::{StoreHolders, held_reason, holders_of};
use crate::test_scratch::scratch;

/// What the process table says about this fixture, without the machine's undescribed processes.
/// Why:
/// The rule keeps every version while any process of this user cannot be described, and a desktop always has such
/// processes (a password manager, an ssh-agent, a compositor); a control that is about something else hands in a scan
/// of its own, and the control about that rule hands in the undescribed processes by name.
fn holders_of_fixture(roots: &[PathBuf]) -> StoreHolders {
    let mut found: StoreHolders = holders_of(roots, Instant::now() + Duration::from_secs(60));
    found.opaque.clear();
    found.incomplete = None;
    return found;
}


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
        downloads_facts: None,
        bytes,
        tree_bytes: bytes,
        written: SystemTime::now() - Duration::from_secs(written_seconds_ago),
        inode: None,
        files: 1,
        newest: SystemTime::now() - Duration::from_secs(written_seconds_ago),
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
        return candidates_of(std::slice::from_ref(&self.root), &tools, Instant::now() + Duration::from_secs(60), Duration::ZERO);
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
            holders: Some(holders_of_fixture(std::slice::from_ref(&self.root))),
            locks: Vec::new(),
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
    // The fault point: exactly what a kill between the rename and the delete leaves, record and all.
    let judged = crate::toolchain::store_versions::judged_of_tree(&version, Instant::now() + Duration::from_secs(60))
        .expect("judge the whole version before it is moved aside");
    fs::rename(&version, &marker).expect("move the version aside");
    crate::toolchain::store_versions::write_judged(&version, &judged).expect("write the removal's record");
    let files_before: usize = fs::read_dir(&marker).expect("list the marker").count();
    assert_eq!(files_before, 1);
    let tools: BTreeSet<String> = ["node".to_string(), "aws".to_string()].into_iter().collect();

    assert!(!tools.is_empty());
    let finished = finish_removals(&fixture.run());
    let lines: Vec<String> = finished.lines;

    // What: the version comes back under its own name, and the marker goes.
    // Why: no pass that finds a marker can know which entry the run that left it judged,
    // so it puts the copy back and lets a pass with the identity and the locks in hand judge it again.
    assert!(finished.unfinished.is_empty(), "{:?}", finished.unfinished);
    assert!(!marker.exists(), "the marker is still there");
    assert!(version.join("program").exists(), "the version was not put back: {lines:?}");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains(&format!("/node/26.11.0 (")), "the line names the version's own path: {lines:?}");
    assert!(lines[0].contains("undone"), "{lines:?}");

    // What: the same pass's plan then removes it for good, on the identity it judges.
    // Why: the reclaim has to happen somewhere, and it happens where a pass can prove what it deletes.
    let plan: TrimPlan = plan_trim(&fixture.candidates(), &fixture.run().bounds, SystemTime::now(), &|_found| {
        return None;
    });
    let outcome: TrimOutcome = apply_trim(&plan, &fixture.run());
    assert!(outcome.removed > 0, "{:?}", outcome.lines);
    assert!(!version.exists(), "{:?}", outcome.lines);
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
    assert!(
        outcome.lines.iter().any(|line| {
            return line.contains("another run wrote it") || line.contains("something was written in it after it was judged");
        }),
        "{:?}",
        outcome.lines
    );
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
    {
        let judged = crate::toolchain::store_versions::judged_of_tree(
            &version_marker,
            Instant::now() + Duration::from_secs(60),
        )
        .expect("judge the whole tree under its removal name");
        crate::toolchain::store_versions::write_judged(&tool.join("1.0.0"), &judged).expect("write the removal's record");
    }
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

    crate::toolchain::store_trim::trim_after_install_with(&context, &[], Some(holders_of_fixture(std::slice::from_ref(&store))));

    assert!(!version_marker.exists(), "the removal an earlier run started was still a marker");
    // A marker is put back under its own name: only a pass that judged the entry may delete it.
    assert!(store.join("t/1.0.0/program").exists(), "the version was not put back under its own name");
    assert!(!has_unfinished_removal(&store), "the marker outlived the removal it stood for");
    // A later install that installs nothing must find nothing left to do, and the store must stay settled.
    crate::toolchain::store_trim::trim_after_install_with(&context, &[], Some(holders_of_fixture(std::slice::from_ref(&store))));
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

    // What: the scan the pass takes after the moves is taken here, after the child started, because a control hands
    // in the scan it wants the pass to judge on.
    let run_after: TrimRun = TrimRun { holders: Some(holders_of_fixture(std::slice::from_ref(&fixture.root))), ..run.clone() };
    let outcome: TrimOutcome = apply_trim(&plan, &run_after);
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
    // A finished archive install writes this marker last, and only a finished copy displaces a held one.
    fs::write(version.join(crate::toolchain::archive_staging::COMPLETE_MARKER), b"node 26.11.0\n")
        .expect("mark the newer copy finished");
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
    // What: the copy a process uses is neither deleted nor restored over the newer one.
    // Why: deleting it breaks the running process, and restoring it would write over the install.
    assert!(marker.join("program").exists(), "the copy a process uses was deleted: {:?}", finished.lines);
    // The copy waits aside until the process using it ends, so the store root is left unfinished.
    assert!(finished.unfinished.contains(&fixture.root), "the copy aside was not left for a later pass: {:?}", finished.lines);
    assert!(
        finished
            .lines
            .iter()
            .any(|line| return line.contains("has not finished") || line.contains("newer copy") || line.contains("no record")),
        "{:?}",
        finished.lines
    );
    // The copy stays aside under its removal name until the process using it ends; it is not deleted.
    let aside: Vec<_> = crate::toolchain::store_versions::removal_markers(&fixture.root);
    assert_eq!(aside.len(), 1, "{aside:?}");
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
        holders: Some(holders_of_fixture(std::slice::from_ref(&store))),
        locks: Vec::new(),
    };
    let look: StoreHolders = run.look();
    let held: Option<String> = held_reason(&look, &victim);

    let tools: std::collections::BTreeSet<String> = ["node".to_string()].into_iter().collect();
    let plan: TrimPlan = plan_trim(&candidates_of(std::slice::from_ref(&store), &tools, Instant::now() + Duration::from_secs(60), Duration::ZERO), &run.bounds, SystemTime::now(), &|found| {
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

    let run: TrimRun = TrimRun { bounds, started: Instant::now(), roots: vec![PathBuf::from("/store")], holders: None, locks: Vec::new() };

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

    trim_after_install_with(&context, &[], Some(holders_of_fixture(std::slice::from_ref(&store))));
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
    // The state a killed removal leaves after it recorded what it held.
    let judged = judged_of_tree(&version, Instant::now() + Duration::from_secs(60)).expect("judge the version");
    fs::rename(&downloads, &downloads_marker).expect("move the downloads aside");
    fs::rename(&version, &version_marker).expect("move the version aside");
    write_judged(&version, &judged).expect("write the removal's record");
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the unfinished-removal marker");
    let mut child = holder_of(&downloads_marker.join("node.tar.gz"));

    let finished = finish_removals(&fixture.run());
    let _ = child.kill();
    let _ = child.wait();

    assert!(version.join("program").exists(), "the version was not put back: {:?}", finished.lines);
    assert!(
        downloads_marker.join("node.tar.gz").exists(),
        "the archives a live process holds were moved: {:?}",
        finished.lines
    );
    assert!(!version_marker.exists(), "{:?}", finished.lines);
    assert!(
        finished.lines.iter().any(|line| return line.contains("it is in use")),
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
    // The state a killed removal leaves after it recorded what it held.
    let judged = judged_of_tree(&version, Instant::now() + Duration::from_secs(60)).expect("judge the version");
    fs::rename(&version, &marker).expect("move the version aside");
    write_judged(&version, &judged).expect("write the removal's record");
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

//region the entry a removal holds, an unfinished name, and a deferred marker

/// What:
/// A fresh install that lands between the removal's look and its move is kept, under its own name.
/// Why:
/// `move_aside` looks at a version, then renames it several system calls later,
/// and `publish_staged` replaces a version with two renames:
/// a copy that lands in between is a finished install, and deleting it discards the install.
/// The order is held by the two halves of a removal, not by two threads racing.
#[test]
fn keeps_a_fresh_install_that_lands_between_the_look_and_the_move() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let candidates = fixture.candidates();
    let judged = candidates
        .iter()
        .find(|found| return found.version == "26.11.0")
        .expect("the oldest version is scanned")
        .clone();

    // What: an install's atomic replace, exactly as `publish_staged` does it: the old copy aside, the new one in.
    // Why: this is what lands between the look and the move.
    let fresh: PathBuf = fixture.root.join("fresh");
    fs::create_dir_all(&fresh).expect("make the fresh copy");
    fs::write(fresh.join("program"), b"fresh copy").expect("write the fresh copy");
    fs::write(fresh.join(crate::toolchain::archive_staging::COMPLETE_MARKER), b"node 26.11.0\n")
        .expect("mark the fresh copy finished");
    fs::rename(&version, fixture.root.join("node/.26.11.0.replaced")).expect("move the old copy aside");
    fs::rename(&fresh, &version).expect("put the fresh copy in place");

    // What: the removal's own move, which reads the entry again at the instant it renames it.
    // Why: the fresh install landed between the plan's look and this call, and the call is what must refuse it.
    let moved = crate::toolchain::store_trim::move_aside(&judged, &StoreHolders::default(), Instant::now() + Duration::from_secs(60));

    assert!(moved.is_err(), "the removal took a copy it did not judge");
    assert_eq!(
        fs::read(version.join("program")).expect("read the fresh copy"),
        b"fresh copy",
        "the fresh install is not under its own name any more"
    );
    assert!(
        !crate::toolchain::store_versions::removal_markers(&fixture.root).iter().any(|marker| return marker.version == "26.11.0"),
        "a marker was left holding a copy the removal did not judge"
    );
}

/// What:
/// A held, finished copy stays aside when the version's name holds an install that has not finished.
/// Why:
/// A `cargo binstall` crate installs in place, so the name can be held by a half-written directory;
/// deleting the held, finished copy for it would leave the only working copy gone,
/// and putting it back would write over the install in flight.
#[test]
fn keeps_a_held_finished_copy_when_the_name_holds_an_unfinished_install() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("cargo/keep/1.0.0");
    fs::create_dir_all(&version).expect("make the version");
    fs::write(version.join("program"), b"old good copy").expect("write the old good copy");
    // The state a killed removal leaves after it recorded what it held.
    let judged = judged_of_tree(&version, Instant::now() + Duration::from_secs(60)).expect("judge the copy");
    let marker: PathBuf = tombstone_of(&version);
    fs::rename(&version, &marker).expect("move the old copy aside"); // a removal an earlier run cut short
    write_judged(&version, &judged).expect("write the removal's record");
    // An install in place has taken the name and has not finished: no crate record yet.
    fs::create_dir_all(&version).expect("start the install in place");
    fs::write(version.join("partial"), b"half-written").expect("write the half-written file");
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the unfinished-removal marker");
    let mut child = std::process::Command::new("sleep")
        .arg("60")
        .current_dir(&marker)
        .spawn()
        .expect("start a process holding the old copy");

    let finished = finish_removals(&fixture.run());
    let _ = child.kill();
    let _ = child.wait();

    assert!(marker.join("program").exists(), "the held, finished copy was deleted: {:?}", finished.lines);
    assert!(version.join("partial").exists(), "the install in place was written over: {:?}", finished.lines);
    // The store-root marker stays, so a later pass decides once the install in place has finished or failed.
    assert!(has_unfinished_removal(&fixture.root), "the marker was settled while a copy is still aside: {:?}", finished.lines);
    assert!(
        finished.lines.iter().any(|line| {
            return line.contains("no record proves this one whole") || line.contains("has not finished");
        }),
        "{:?}",
        finished.lines
    );
}

/// What:
/// A root whose removals were all deferred still gets the marker the next install reads.
/// Why:
/// The marker is what makes an install that installs nothing run the pass,
/// and only a move wrote it, so a pass that moved nothing left no trace and the store kept what it should reclaim.
#[test]
fn writes_the_marker_for_a_root_whose_removals_were_all_deferred() {
    let fixture = Fixture::new();
    let run: TrimRun = TrimRun {
        bounds: TrimBounds { size_bound_bytes: 0, age_floor: Duration::ZERO, work_limit: Duration::ZERO },
        started: Instant::now(),
        roots: vec![fixture.root.clone()],
        holders: Some(holders_of_fixture(std::slice::from_ref(&fixture.root))),
        locks: Vec::new(),
    };
    let plan: TrimPlan = plan_trim(&fixture.candidates(), &run.bounds, SystemTime::now(), &|_found| return None);

    let outcome: TrimOutcome = apply_trim(&plan, &run);

    assert_eq!(outcome.removed, 0, "{:?}", outcome.lines);
    assert!(outcome.unfinished.contains(&fixture.root), "{:?}", outcome.lines);
    assert!(has_unfinished_removal(&fixture.root), "the next install would not run the pass: {:?}", outcome.lines);
    assert!(fixture.root.join("node/26.11.0/program").exists());
}

//endregion the entry a removal holds, an unfinished name, and a deferred marker

//region the copy a marker holds, and the file a leftover name holds

/// What:
/// A copy under a removal name that no record proved whole goes back unfinished, and the pass that finds it keeps it.
/// Why:
/// A kill between a removal's rename and its record write leaves a copy the next pass cannot tell from one a deletion
/// began inside, and a name proves nothing about what stands under it.
/// Leaving it aside never ends (the version is absent, the tree is nobody's, every run prints the same line); putting
/// it back as it was would hand a possibly partial copy out as a finished version.
/// It goes back under its own name with the completion marker taken away, so the lock-named copy is kept, nothing is
/// deleted, and the next install replaces it if it is not whole.
#[test]
fn puts_back_a_copy_no_record_proved_whole_as_unfinished() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let store: PathBuf = scope.toolchain_root.clone();
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"20.0.0","route":"archive"}}}"#)
        .expect("write a lock naming the version");
    // The post-kill state: the finished copy under the marker the program's own rename produces, the name free.
    let marker: PathBuf = store.join("node").join(format!("{TOMBSTONE_PREFIX}20.0.0"));
    make_finished_copy(&marker, b"#!/bin/sh\n");
    written_ago(&marker, 2 * 24 * 3600);
    assert!(is_finished(&marker, "node"), "the fixture must prove the copy carries what a finished one carries");
    fs::write(unfinished_marker(&store), b"").expect("write the unfinished-removal marker");
    let runner = FakeRunner::new();
    runner.answer("git -C", "", 128);
    let host = Host::current();
    let env_holding: EnvMap = env
        .with("MPM_SHADOW_STORE_SIZE_BOUND", "0")
        .with("MPM_SHADOW_STORE_AGE_FLOOR", "0");
    let context = Context { scope: &scope, env: &env_holding, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };

    trim_after_install_with(&context, &[], Some(holders_of_fixture(std::slice::from_ref(&store))));

    let version: PathBuf = store.join("node/20.0.0");
    assert!(version.join("bin/node").exists(), "the lock-named copy was deleted");
    assert!(!marker.exists(), "the marker outlived the copy it held");
    assert!(
        !is_finished(&version, "node"),
        "a copy no record proved whole was left where an install reads it as a finished version"
    );
}

/// What:
/// A flat leftover download whose file was replaced after the scan is not deleted.
/// Why:
/// The plan judges a file's identity, and the direct deletion is the third place a name could have come to hold
/// something else; the file's own inode and write time are what the deletion proves.
#[test]
fn keeps_a_leftover_whose_file_was_replaced_after_the_scan() {
    let fixture = Fixture::new();
    let leftovers: PathBuf = fixture.root.join("downloads");
    fs::create_dir_all(&leftovers).expect("make the downloads directory");
    let stale: PathBuf = leftovers.join("node-26.12.0-node-v26.12.0-linux-x64.tar.gz");
    fs::write(&stale, vec![b'z'; 40]).expect("write the leftover");
    written_ago(&stale, 2 * 24 * 3600);
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();
    let plan: TrimPlan = plan_trim(&candidates_of(std::slice::from_ref(&fixture.root), &tools, Instant::now() + Duration::from_secs(60), Duration::ZERO), &fixture.run().bounds, SystemTime::now(), &|_found| {
        return None;
    });
    let planned: bool = plan.steps.iter().any(|step| {
        return match step {
            TrimStep::Remove(found) => found.directory == stale,
            TrimStep::Keep(_, _) => false,
        };
    });
    assert!(planned, "the leftover is not planned for removal: {:?}", plan.steps);
    // What: another run puts a file of the same name there, after the scan.
    // Why: the plan's entry is gone, and this file is not the one it judged.
    fs::remove_file(&stale).expect("remove the judged file");
    fs::write(&stale, vec![b'f'; 40]).expect("write a newer file of the same name");

    let outcome: TrimOutcome = apply_trim(&plan, &fixture.run());

    assert!(stale.exists(), "the file the removal did not judge was deleted: {:?}", outcome.lines);
}

//endregion the copy a marker holds, and the file a leftover name holds

//region the five rules of the state model

/// Makes a finished archive copy of `node` in `directory`.
fn make_finished_copy(directory: &std::path::Path, payload: &[u8]) {
    fs::create_dir_all(directory.join("bin")).expect("make the copy");
    fs::write(directory.join("bin/node"), payload).expect("write its program");
    let mut permissions = fs::metadata(directory.join("bin/node")).expect("read its mode").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(directory.join("bin/node"), permissions).expect("make it runnable");
    fs::write(directory.join(crate::toolchain::archive_staging::COMPLETE_MARKER), b"node 26.0.0\n")
        .expect("mark it finished");
}

/// What:
/// A tombstone whose deletion has begun is finished, never put back under the version's name.
/// Why: a kill inside a recursive deletion leaves a tree with files already gone.
/// Restoring it would show a broken copy as whole, and its executable and completion marker can survive the kill.
#[test]
fn never_puts_a_partly_deleted_tombstone_back_as_a_version() {
    let fixture = Fixture::new();
    // A version the fixture does not hold, so "the version's name is free" is the fixture's own state too.
    let version: PathBuf = fixture.root.join("node/20.0.0");
    // The name a removal's deletion leaves: built from the removal name, which is what every removal this module
    // deletes writes.
    let deleting: PathBuf = crate::toolchain::store_versions::deleting_of(&tombstone_of(&version));
    // The record of the removal a later pass must find from this name.
    make_finished_copy(&version, b"a whole copy before the removal began");
    let judged = judged_of_tree(&version, Instant::now() + Duration::from_secs(60)).expect("judge the whole copy");
    write_judged(&version, &judged).expect("write the removal's record");
    // The tree the removal held leaves its name; the deletion of it had begun when the kill landed.
    fs::remove_dir_all(&version).expect("take the tree out of its name");
    // The state a kill inside the deletion leaves: a deletion name holding a tree a file short.
    make_finished_copy(&deleting, b"surviving program");
    fs::create_dir_all(deleting.join("lib")).expect("make the library directory");
    fs::write(deleting.join("lib/libnode.so"), b"gone soon").expect("write the library");
    fs::remove_file(deleting.join("lib/libnode.so")).expect("model a file the deletion already took");
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the unfinished-removal marker");

    let finished = finish_removals(&fixture.run());

    assert!(!version.exists(), "a partly deleted copy was put back as the version: {:?}", finished.lines);
    assert!(!deleting.exists(), "the deletion was not finished: {:?}", finished.lines);
    assert!(removal_markers(&fixture.root).is_empty());
    assert!(!record_of(&version).exists(), "the record outlived the tree it described: {:?}", finished.lines);
}

/// What:
/// A copy a live process works inside is neither deleted nor replaced when another install takes the version's name.
/// Why: the process keeps working only while its copy is there.
#[test]
fn keeps_a_copy_a_process_uses_even_when_the_name_is_taken() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let marker: PathBuf = tombstone_of(&version);
    fs::rename(&version, &marker).expect("move the version aside");
    // The name is taken by a finished copy of the same version, and a process works inside the moved-aside one.
    make_finished_copy(&version, b"fresh copy");
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the unfinished-removal marker");
    let mut child = std::process::Command::new("sleep")
        .arg("60")
        .current_dir(&marker)
        .spawn()
        .expect("start a process inside the moved-aside copy");

    let finished = finish_removals(&fixture.run());
    let cwd: PathBuf = fs::read_link(format!("/proc/{}/cwd", child.id())).expect("read the child's folder");
    let used_copy_survived: bool = marker.join("program").exists();
    let _ = child.kill();
    let _ = child.wait();

    assert!(used_copy_survived, "the copy a process uses was deleted; cwd={} lines={:?}", cwd.display(), finished.lines);
}

/// What:
/// A write into a folder that already existed, after the scan, keeps the version.
/// Why: it changes neither the version root's inode nor its own write time,
/// and only the whole-tree facts (the file count, the bytes, the newest write inside) show it.
#[test]
fn keeps_a_write_into_a_nested_folder_that_lands_after_the_scan() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    fs::create_dir_all(version.join("nested")).expect("make a nested folder");
    fs::write(version.join("nested/old"), b"old").expect("write a nested file");
    written_ago(&version, 2 * 24 * 3600);
    let judged: TrimCandidate = fixture
        .candidates()
        .into_iter()
        .find(|found| return found.directory == version)
        .expect("the version is scanned");
    // What: another command writes into the folder that already existed.
    // Why: the entry that changes is inside the tree, not the root.
    fs::write(version.join("nested/fresh"), b"fresh user data").expect("write after the scan");
    assert_eq!(
        fs::symlink_metadata(&version).and_then(|metadata| return metadata.modified()).ok(),
        Some(judged.written),
        "the fixture must prove the root's own write time cannot see this write"
    );
    let plan: TrimPlan = TrimPlan { total_bytes: judged.bytes, over_bound: true, steps: vec![TrimStep::Remove(judged)] };

    let outcome: TrimOutcome = apply_trim(&plan, &fixture.run());

    assert!(version.join("nested/fresh").exists(), "the write after the scan was deleted: {:?}", outcome.lines);
}

/// What:
/// A pass finds a tombstone and recovers it even when it installed nothing and the store-root marker is gone.
/// Why: two passes settling the marker at once can clear it after one wrote it, and recovery must not wait on an
/// unrelated install.
#[test]
fn recovers_a_tombstone_when_nothing_was_installed_and_the_marker_is_gone() {
    // What: the pass runs in a store whose only trace of a killed removal is the tombstone and its record.
    // Why: the store-root marker is a hint, and two passes settling it at once can clear it after one wrote it.
    let (_directory, scope, env) = sample_scope_on_disk();
    let store: PathBuf = scope.toolchain_root.clone();
    let marker: PathBuf = store.join("node").join(format!("{TOMBSTONE_PREFIX}26.11.0"));
    fs::create_dir_all(&marker).expect("make the tombstone");
    fs::write(marker.join("program"), b"a whole copy").expect("write its payload");
    let judged = judged_of_tree(&marker, Instant::now() + Duration::from_secs(60)).expect("judge the whole copy");
    write_judged(&store.join("node/26.11.0"), &judged).expect("write the removal's record");
    assert!(!has_unfinished_removal(&store), "this case is about a store root whose marker is gone");
    let runner = FakeRunner::new();
    let host = Host::current();
    let context = Context { scope: &scope, env: &env, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };

    trim_after_install_with(&context, &[], Some(holders_of_fixture(std::slice::from_ref(&store))));

    assert!(
        store.join("node/26.11.0/program").exists(),
        "the no-op install left the tombstone unrecovered; marker present={}",
        has_unfinished_removal(&store)
    );
    assert!(!marker.exists(), "the marker outlived the copy it held");
    assert!(!record_of(&store.join("node/26.11.0")).exists(), "the record outlived the tree it described");
}

/// What:
/// A deletion that runs out of its time limit leaves the tree under its deletion name, and the next pass finishes it.
/// Why: a begun deletion is finished and never undone; the work limit bounds the walk, not only the loop around it.
#[test]
fn a_deletion_that_runs_out_of_time_leaves_its_tree_under_the_deletion_name() {
    let fixture = Fixture::new();
    // A version the fixture does not hold, so "the version's name is free" is the fixture's own state too.
    let version: PathBuf = fixture.root.join("node/20.0.0");
    make_finished_copy(&version, b"a tree to delete");
    let deleting: PathBuf = crate::toolchain::store_versions::deleting_of(&version);
    // What: the deletion's own seam, with a deadline already gone.
    // Why: no large tree and no wait are needed to show what the walk does at its limit.
    let refusal: Option<String> =
        crate::toolchain::store_trim::delete_tree_within(&version, Instant::now() - Duration::from_secs(1));

    assert!(refusal.is_some(), "the deletion reported it finished a tree it could not reach the end of");
    assert!(deleting.exists(), "the tree was not left under its deletion name: {refusal:?} version_exists={}", version.exists());
    assert!(!version.exists(), "the version's name still holds a tree whose deletion began");
    // The next pass finishes it without asking what it is.
    let finished = finish_removals(&fixture.run());
    assert!(!deleting.exists(), "the next pass did not finish the deletion: {:?}", finished.lines);
}

/// What:
/// A partly deleted copy under a removal name and no record goes back unfinished, and not one of its files is deleted.
/// Why:
/// A build of an earlier commit of this line deleted a removal name's tree in place, so a kill inside that deletion
/// left exactly this: a tree that kept its executable and its completion marker with payload files gone.
/// Nothing here can prove it whole or prove it short, so the pass may neither hand it out as a version nor delete it;
/// it goes back with nothing left saying it finished, which ends the state without losing a file.
#[test]
fn puts_back_a_partly_deleted_copy_as_unfinished_and_deletes_nothing() {
    let fixture = Fixture::new();
    // A version the fixture does not hold, so "the version's name is free" is the fixture's own state too.
    let version: PathBuf = fixture.root.join("node/20.0.0");
    let marker: PathBuf = tombstone_of(&version);
    make_finished_copy(&marker, b"surviving program");
    fs::create_dir_all(marker.join("lib")).expect("make the library directory");
    fs::write(marker.join("lib/libnode.so"), b"gone soon").expect("write the library");
    fs::remove_file(marker.join("lib/libnode.so")).expect("model a file the deletion already took");
    fs::write(marker.join("kept-by-a-person"), b"notes a person left here").expect("write a file of a person's");
    assert!(is_finished(&marker, "node"), "the fixture must prove the surviving marker would pass for finished");

    let finished = finish_removals(&fixture.run());

    assert!(version.exists(), "a partly deleted copy was deleted instead of put back: {:?}", finished.lines);
    assert!(
        !is_finished(&version, "node"),
        "a partly deleted copy was left where an install reads it as a finished version"
    );
    assert!(version.join("kept-by-a-person").exists(), "a file inside the tree was deleted: {:?}", finished.lines);
    assert!(!marker.exists(), "the marker outlived the copy it held");
    assert!(
        finished.lines.iter().any(|line| return line.contains("unfinished")),
        "the line does not say the copy went back unfinished: {:?}",
        finished.lines
    );
}

/// What:
/// A file written into a tree after its record was written keeps that tree, which goes back unfinished.
/// Why:
/// The record says what the tree held, so a tree that changed since is not the one judged: it may be a person's work
/// landing in a version directory while the cleanup ran, and reading the difference as "the deletion has begun"
/// deleted it.
#[test]
fn puts_back_a_tree_a_write_landed_in_after_its_record_as_unfinished() {
    let fixture = Fixture::new();
    // A version the fixture does not hold, so "the version's name is free" is the fixture's own state too.
    let version: PathBuf = fixture.root.join("node/20.0.0");
    let marker: PathBuf = tombstone_of(&version);
    make_finished_copy(&version, b"a whole copy");
    let judged = judged_of_tree(&version, Instant::now() + Duration::from_secs(60)).expect("judge the version");
    assert!(is_finished(&version, "node"), "the fixture must prove the copy carries what a finished one carries");
    fs::rename(&version, &marker).expect("move the version aside");
    write_judged(&version, &judged).expect("write the removal's record");
    fs::write(marker.join("landed-after"), b"a person's work").expect("write into the tree after its record");

    let finished = finish_removals(&fixture.run());

    assert!(
        version.join("landed-after").exists(),
        "a write that landed after the record was deleted: {:?}",
        finished.lines
    );
    assert!(!is_finished(&version, "node"), "a tree that changed after its record was read as a finished version");
    assert!(!marker.exists(), "the marker outlived the copy it held: {:?}", finished.lines);
}

/// What:
/// A tool's `current` link that names a copy put back unfinished is taken away, and the copy stays.
/// Why:
/// A launcher follows that link without asking whether the version finished, and while the copy was aside the link
/// named nothing; leaving it would turn an absent tool into a run of a copy nothing proved whole.
/// The link is mpm-shadow's own, nothing is deleted, and the next install of that version points it again.
#[test]
fn takes_away_the_current_link_that_names_a_copy_no_record_proved_whole() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let marker: PathBuf = tombstone_of(&version);
    fs::rename(&version, &marker).expect("move the version aside");
    point_current(&fixture.root, "node", "26.11.0");

    let finished = finish_removals(&fixture.run());

    assert!(version.join("program").exists(), "the copy was deleted instead of put back: {:?}", finished.lines);
    assert!(
        fs::symlink_metadata(fixture.root.join("node/current")).is_err(),
        "the link still names a copy nothing proved whole: {:?}",
        finished.lines
    );
    assert!(
        finished.lines.iter().any(|line| return line.contains("current link is taken away")),
        "the line does not say the link was taken away: {:?}",
        finished.lines
    );
}

/// What:
/// The record of a judgement goes when the tree it described leaves the removal name.
/// Why:
/// A record left beside a version's name is read by the next removal of that version, and one that describes an older
/// tree turns a kill between a rename and its record write into a proof about the wrong tree.
#[test]
fn deletes_the_record_of_a_judgement_when_the_copy_goes_back() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let marker: PathBuf = tombstone_of(&version);
    let judged = judged_of_tree(&version, Instant::now() + Duration::from_secs(60)).expect("judge the version");
    fs::rename(&version, &marker).expect("move the version aside");
    write_judged(&version, &judged).expect("write the removal's record");

    let finished = finish_removals(&fixture.run());

    assert!(version.join("program").exists(), "the version was not put back: {:?}", finished.lines);
    assert!(!record_of(&version).exists(), "the record outlived the tree it described: {:?}", finished.lines);
}

/// What:
/// The record of a judgement goes when the tree it described is deleted.
/// Why:
/// A record under a name no tree stands at is nobody's, and it is what a later removal of the same version would be
/// read through if that removal were killed before writing its own.
#[test]
fn deletes_the_record_of_a_judgement_when_the_tree_is_deleted() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let judged: TrimCandidate = fixture
        .candidates()
        .into_iter()
        .find(|found| return found.directory == version)
        .expect("the oldest version is scanned");
    let plan: TrimPlan = TrimPlan { total_bytes: judged.bytes, over_bound: true, steps: vec![TrimStep::Remove(judged)] };

    let outcome: TrimOutcome = apply_trim(&plan, &fixture.run());

    assert_eq!(outcome.removed, 1, "{:?}", outcome.lines);
    assert!(!version.exists(), "{:?}", outcome.lines);
    assert!(!record_of(&version).exists(), "the record outlived the tree it described: {:?}", outcome.lines);
}

/// What:
/// A finished copy at the version's name takes the place of a copy no record proved whole, and the record goes.
/// Why:
/// The copy under the removal name is a duplicate of a finished copy of the same version, which is the one proof this
/// step has, and the record described the duplicate, not the copy that stays.
#[test]
fn deletes_a_copy_no_record_proved_whole_when_a_finished_copy_took_the_name() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/20.0.0");
    let marker: PathBuf = tombstone_of(&version);
    make_finished_copy(&marker, b"old copy");
    let older = judged_of_tree(&marker, Instant::now() + Duration::from_secs(60)).expect("judge the whole copy");
    write_judged(&version, &older).expect("write the removal's record");
    // A later removal of the same version, killed before it replaced the older record.
    make_finished_copy(&version, b"fresh copy");

    let finished = finish_removals(&fixture.run());

    assert!(!marker.exists(), "the duplicate outlived the finished copy at its name: {:?}", finished.lines);
    assert_eq!(fs::read(version.join("bin/node")).expect("read the copy at the name"), b"fresh copy");
    assert!(!record_of(&version).exists(), "the record outlived the tree it described: {:?}", finished.lines);
}

//endregion the five rules of the state model

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
    let run: TrimRun = TrimRun { bounds, started: Instant::now(), roots: replayed_roots.clone(), holders: Some(holders_of_fixture(&replayed_roots)), locks: Vec::new() };
    let look_clock: Instant = Instant::now();
    let planning_look: StoreHolders = run.look();
    let look_ms: u128 = look_clock.elapsed().as_millis();
    let inventory_clock: Instant = Instant::now();
    let candidates: Vec<TrimCandidate> = candidates_of(&replayed_roots, &tools, Instant::now() + Duration::from_secs(60), Duration::ZERO);
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
    let removal_run: TrimRun = TrimRun { bounds, started: Instant::now(), roots: replayed_roots.clone(), holders: Some(holders_of_fixture(&replayed_roots)), locks: Vec::new() };
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

//region what protects a version, and what a read that fails means

/// A version of `node` in `root`, with one program file.
fn node_version(root: &std::path::Path, version: &str) -> PathBuf {
    let directory: PathBuf = root.join("node").join(version);
    fs::create_dir_all(&directory).expect("make the version directory");
    fs::write(directory.join("program"), version).expect("write the program");
    return directory;
}

/// What:
/// A repository's lock keeps a version in the shared global store from a global-scope pass.
/// Why:
/// Which locks can name a version in a store does not depend on which scope asked: a global pass that read only its
/// own lock deleted a global-store version a worktree's lock still named.
#[test]
fn keeps_a_global_store_version_a_worktrees_lock_names() {
    let directory = scratch();
    let home: PathBuf = directory.path().join("home");
    let work: PathBuf = directory.path().join("work");
    fs::create_dir_all(&work).expect("make the worktree");
    fs::write(work.join("toolchain.plan.jsonc"), "{}").expect("a plan is what names a repository");
    let env: EnvMap = EnvMap::from_pairs(&[
        ("HOME", home.to_str().expect("UTF-8 home")),
        ("XDG_CONFIG_HOME", directory.path().join("config").to_str().expect("UTF-8 config")),
        ("XDG_DATA_HOME", directory.path().join("data").to_str().expect("UTF-8 data")),
        ("MONOCHROMATIC_WORKTREE_ROOT", work.to_str().expect("UTF-8 worktree")),
        ("MPM_SHADOW_STORE_SIZE_BOUND", "0"),
        ("MPM_SHADOW_STORE_AGE_FLOOR", "0"),
    ]);
    let scope = crate::toolchain::scope::global_scope(&env).expect("a global scope");
    fs::create_dir_all(scope.lock_file.parent().expect("the config directory")).expect("make the config directory");
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"2.0.0","route":"archive"}}}"#)
        .expect("write the global lock");
    let worktree_lock: PathBuf = work.join("toolchain.lock.json");
    fs::write(&worktree_lock, r#"{"tools":{"node":{"version":"1.0.0","route":"archive"}}}"#)
        .expect("write the worktree lock");
    let old: PathBuf = node_version(&scope.toolchain_root, "1.0.0");
    let current: PathBuf = node_version(&scope.toolchain_root, "2.0.0");
    written_ago(&old, 2 * 24 * 3600);
    let runner = FakeRunner::new();
    runner.answer("git -C", &format!("worktree {}\nHEAD abc\n\n", work.display()), 0);
    let host = Host::current();
    let context = Context { scope: &scope, env: &env, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };
    let holders: StoreHolders = holders_of_fixture(std::slice::from_ref(&scope.toolchain_root));

    trim_after_install_with(&context, &[("unrelated".to_string(), "1.0.0".to_string())], Some(holders));

    assert!(old.join("program").exists(), "a global pass deleted a version a worktree's lock names");
    assert!(current.exists(), "the version the global lock names went");
}

/// What:
/// A global pass that can name no repository removes nothing.
/// Why:
/// The locks of the worktrees that install into the shared store can name any version in it, and a pass that cannot
/// list them cannot tell a pinned version from one nobody wants; reading "no lock names it" into locks it never opened
/// is what finding one of the sixth review did.
#[test]
fn removes_no_global_store_version_when_no_repository_can_be_named() {
    let directory = scratch();
    let home: PathBuf = directory.path().join("home");
    let env: EnvMap = EnvMap::from_pairs(&[
        ("HOME", home.to_str().expect("UTF-8 home")),
        ("XDG_CONFIG_HOME", directory.path().join("config").to_str().expect("UTF-8 config")),
        ("XDG_DATA_HOME", directory.path().join("data").to_str().expect("UTF-8 data")),
        ("MPM_SHADOW_STORE_SIZE_BOUND", "0"),
        ("MPM_SHADOW_STORE_AGE_FLOOR", "0"),
    ]);
    let scope = crate::toolchain::scope::global_scope(&env).expect("a global scope");
    fs::create_dir_all(scope.lock_file.parent().expect("the config directory")).expect("make the config directory");
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"2.0.0","route":"archive"}}}"#)
        .expect("write the global lock");
    let old: PathBuf = node_version(&scope.toolchain_root, "1.0.0");
    written_ago(&old, 2 * 24 * 3600);
    let runner = FakeRunner::new();
    let host = Host::current();
    let context = Context { scope: &scope, env: &env, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };
    let holders: StoreHolders = holders_of_fixture(std::slice::from_ref(&scope.toolchain_root));

    trim_after_install_with(&context, &[("unrelated".to_string(), "1.0.0".to_string())], Some(holders));

    assert!(old.join("program").exists(), "a global pass removed a version while it could see no worktree lock");
}

/// What:
/// A lock row whose route this release cannot read keeps its version.
/// Why:
/// Such a row names a version this pass cannot place, so it may pin any version; reading it as "pins nothing" deleted a
/// version a lock still named.
#[test]
fn keeps_a_version_a_lock_with_a_route_this_release_cannot_read_names() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let store: PathBuf = scope.toolchain_root.clone();
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"2.0.0","route":"archive"}}}"#)
        .expect("write this scope's lock");
    let global = crate::toolchain::scope::global_scope(&env).expect("a global scope");
    fs::create_dir_all(global.lock_file.parent().expect("the config directory")).expect("make the config directory");
    fs::write(&global.lock_file, r#"{"tools":{"node":{"version":"1.0.0","route":"archive-v2"}}}"#)
        .expect("write a lock with a route this release cannot read");
    let old: PathBuf = node_version(&store, "1.0.0");
    let current: PathBuf = node_version(&store, "2.0.0");
    written_ago(&old, 2 * 24 * 3600);
    let runner = FakeRunner::new();
    runner.answer("git -C", &format!("worktree {}\nHEAD abc\n\n", scope.root.display()), 0);
    let host = Host::current();
    let env_holding: EnvMap = env.with("MPM_SHADOW_STORE_SIZE_BOUND", "0").with("MPM_SHADOW_STORE_AGE_FLOOR", "0");
    let context =
        Context { scope: &scope, env: &env_holding, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };
    let holders: StoreHolders = holders_of_fixture(std::slice::from_ref(&store));

    trim_after_install_with(&context, &[("unrelated".to_string(), "1.0.0".to_string())], Some(holders));

    assert!(old.join("program").exists(), "a version a lock with an unreadable route names was deleted");
    assert!(current.exists(), "the version the readable lock names went");
}

/// What:
/// A global lock that cannot be found keeps every version.
/// Why:
/// It may name any version; a pass that cannot find it must not read "no lock names it" into it.
#[test]
fn keeps_every_version_when_the_global_lock_cannot_be_found() {
    let fixture = Fixture::new();
    let old: PathBuf = fixture.root.join("node/26.11.0");
    // What: a worktree of its own, and a pass handed an environment with no home at all.
    // Why: the missing home is how the machine-global lock cannot be found, and the stores the pass trims must still be
    // the fixture's.
    let home: &std::path::Path = fixture.root.ancestors().nth(4).expect("the fixture's home directory");
    let work: PathBuf = home.join("work");
    fs::create_dir_all(&work).expect("make the worktree");
    let scope = crate::toolchain::scope::repo_scope(&work, &EnvMap::from_pairs(&[("HOME", home.to_str().expect("UTF-8 home"))]))
        .expect("a scope");
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"26.12.0","route":"archive"}}}"#)
        .expect("write the lock");
    assert_eq!(scope.toolchain_root, fixture.root, "the scope must trim the fixture's own store");
    let env: EnvMap = EnvMap::from_pairs(&[("MPM_SHADOW_STORE_SIZE_BOUND", "0"), ("MPM_SHADOW_STORE_AGE_FLOOR", "0")]);
    let runner = FakeRunner::new();
    runner.answer("git -C", &format!("worktree {}\nHEAD abc\n\n", work.display()), 0);
    let host = Host::current();
    let context = Context { scope: &scope, env: &env, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };
    let holders: StoreHolders = holders_of_fixture(std::slice::from_ref(&fixture.root));

    trim_after_install_with(&context, &[("unrelated".to_string(), "1.0.0".to_string())], Some(holders));

    assert!(old.join("program").exists(), "a pass removed a version while the global lock could not be found");
}

/// What:
/// A version whose nested file was written a moment ago stays.
/// Why:
/// A write into a folder that already existed leaves the version root's own write time alone, and the age of the newest
/// write inside the tree is the age the floor is about.
#[test]
fn keeps_a_version_a_file_was_written_into_a_moment_ago() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    fs::create_dir_all(version.join("nested")).expect("make a nested folder");
    fs::write(version.join("nested/fresh-user-data"), b"a person's work").expect("write a file a moment ago");
    written_ago(&version, 2 * 24 * 3600);

    let bounds: TrimBounds = TrimBounds { size_bound_bytes: 0, age_floor: Duration::from_secs(3600), work_limit: Duration::from_secs(60) };
    let plan: TrimPlan = plan_trim(&fixture.candidates(), &bounds, SystemTime::now(), &|_found| return None);

    let kept: Vec<String> = kept_by(&plan).into_iter().map(|(name, _reason)| return name).collect();
    assert!(kept.iter().any(|name| return name == "node-26.11.0"), "a file written a moment ago was called old: {kept:?}");
}

/// What:
/// A version whose tree the pass could not read in full is not judged.
/// Why:
/// A directory the pass cannot open leaves a file count and a newest write that say nothing about what is inside, and a
/// version judged by a partial tree is removed on a proof it does not have.
#[test]
fn keeps_a_version_whose_tree_the_pass_could_not_read_in_full() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let hidden: PathBuf = version.join("invisible");
    fs::create_dir_all(&hidden).expect("make a folder the pass cannot open");
    fs::write(hidden.join("a-person's-file"), b"kept").expect("write a file inside it");
    written_ago(&version, 2 * 24 * 3600);
    let mut permissions = fs::metadata(&hidden).expect("read the folder's mode").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o000);
    fs::set_permissions(&hidden, permissions).expect("make the folder unreadable");

    let candidates: Vec<TrimCandidate> = fixture.candidates();
    let run: TrimRun = fixture.run();
    let plan: TrimPlan = plan_trim(&candidates, &run.bounds, SystemTime::now(), &|_found| return None);
    let outcome: TrimOutcome = apply_trim(&plan, &run);

    assert!(version.join("program").exists(), "a version read in part was removed: {:?} {}", outcome.lines, candidates.len());
    let mut restored = fs::metadata(&hidden).expect("read the folder's mode").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut restored, 0o700);
    fs::set_permissions(&hidden, restored).expect("let the scratch directory go");
}

/// What:
/// A copy a removal holds is neither put back nor deleted when the process table could not be read in full,
/// and the line names the process.
/// Why:
/// A process the kernel refuses to describe may run it, so the one place that renames or unlinks anything answers kept;
/// the copy stays where it is, and the line is what a person needs to end the process or decide.
#[test]
fn keeps_a_copy_aside_when_the_process_table_could_not_be_read_in_full() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let marker: PathBuf = tombstone_of(&version);
    fs::rename(&version, &marker).expect("move the version aside");
    let mut holders: StoreHolders = holders_of_fixture(std::slice::from_ref(&fixture.root));
    holders.opaque.push(OpaqueProcess { pid: 4242, comm: "fd".to_string() });
    holders.incomplete = crate::toolchain::store_path_holders::undescribed_reason(&holders.opaque);

    let finished = finish_removals(&TrimRun { bounds: fixture.run().bounds, started: Instant::now(), roots: vec![fixture.root.clone()], holders: Some(holders), locks: Vec::new() });

    assert!(!version.exists(), "a copy a process may run was put back while the scan could not tell: {:?}", finished.lines);
    assert!(marker.join("program").exists(), "a copy a process may run was deleted: {:?}", finished.lines);
    assert!(
        finished.lines.iter().any(|line| return line.contains("pid 4242 (fd)") && line.contains("stays out of its name")),
        "the line does not name the process: {:?}",
        finished.lines
    );
}

/// What:
/// A pass whose process table could not be read in full removes nothing.
/// Why:
/// A process the kernel refuses to describe may run any version, so no version can be told from one in use; the pass
/// says so once instead of guessing.
#[test]
fn removes_nothing_when_a_process_cannot_be_described() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let store: PathBuf = scope.toolchain_root.clone();
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"2.0.0","route":"archive"}}}"#)
        .expect("write the lock");
    let old: PathBuf = node_version(&store, "1.0.0");
    let current: PathBuf = node_version(&store, "2.0.0");
    written_ago(&old, 2 * 24 * 3600);
    let runner = FakeRunner::new();
    runner.answer("git -C", &format!("worktree {}\nHEAD abc\n\n", scope.root.display()), 0);
    let host = Host::current();
    let env_holding: EnvMap = env.with("MPM_SHADOW_STORE_SIZE_BOUND", "0").with("MPM_SHADOW_STORE_AGE_FLOOR", "0");
    let context =
        Context { scope: &scope, env: &env_holding, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };
    let mut holders: StoreHolders = holders_of_fixture(std::slice::from_ref(&store));
    holders.opaque.push(OpaqueProcess { pid: 4242, comm: "fd".to_string() });
    holders.incomplete = crate::toolchain::store_path_holders::undescribed_reason(&holders.opaque);

    trim_after_install_with(&context, &[("unrelated".to_string(), "1.0.0".to_string())], Some(holders));

    assert!(old.join("program").exists(), "a version was removed while a process that may run it could not be described");
    assert!(current.exists());
}

//endregion what protects a version, and what a read that fails means


//region what the one function proves, and what the seventh review found

/// A run of these controls: a store of its own and a scan with nothing in it.
fn control_run(root: &Path) -> TrimRun {
    return TrimRun {
        bounds: TrimBounds { size_bound_bytes: 0, age_floor: Duration::ZERO, work_limit: Duration::from_secs(60) },
        started: Instant::now(),
        roots: vec![root.to_path_buf()],
        holders: Some(StoreHolders::default()),
        locks: Vec::new(),
    };
}

/// What:
/// A downloads tree another command put at the judged name after the inventory is not moved or deleted.
/// Why:
/// The archives are an object of their own: the plan judged one tree, and a replacement under that name is a different
/// object whose bytes belong to whoever wrote them.
#[test]
fn keeps_a_replacement_download_tree_it_never_judged() {
    let fixture = Fixture::new();
    let version = fixture.root.join("node/26.11.0");
    let downloads = fixture.root.join("downloads/node/26.11.0");
    fs::create_dir_all(&downloads).expect("make the judged downloads");
    fs::write(downloads.join("old-archive"), b"old").expect("write the judged archive");
    let judged = fixture
        .candidates()
        .into_iter()
        .find(|candidate| return candidate.directory == version)
        .expect("scan the version");
    assert!(judged.downloads.is_some(), "the scan did not find the archives");

    // Another command replaces only the archives after the inventory; the version tree is untouched.
    fs::remove_dir_all(&downloads).expect("remove the judged archives");
    fs::create_dir_all(&downloads).expect("make the replacement archives");
    fs::write(downloads.join("fresh-user-data"), b"new data, not judged").expect("write replacement data");
    let plan = TrimPlan { total_bytes: judged.bytes, over_bound: true, steps: vec![TrimStep::Remove(judged)] };

    let outcome = apply_trim(&plan, &control_run(&fixture.root));

    assert_eq!(outcome.removed, 0, "a replacement archive tree was removed: {:?}", outcome.lines);
    assert!(
        downloads.join("fresh-user-data").exists(),
        "the cleanup deleted a replacement download tree it never judged: {:?}",
        outcome.lines
    );
    assert!(version.join("program").exists(), "the version went with archives it never had: {:?}", outcome.lines);
}

/// What:
/// A copy a removal holds that changed after its record is put back unfinished when the version's name holds a
/// finished copy, and its later file is kept.
/// Why:
/// The record no longer proves the held copy whole, so it is not a duplicate of the copy at the name, whatever the name
/// holds.
#[test]
fn keeps_a_changed_copy_when_a_finished_copy_takes_the_name() {
    let fixture = Fixture::new();
    let version = fixture.root.join("node/20.0.0");
    let marker = tombstone_of(&version);
    make_finished_copy(&version, b"old copy");
    let judged = judged_of_tree(&version, Instant::now() + Duration::from_secs(60)).expect("judge the old copy");
    fs::rename(&version, &marker).expect("move the old copy aside");
    write_judged(&version, &judged).expect("record the old copy");

    // This write makes the record stale, exactly as a person's write during a killed removal would.
    fs::write(marker.join("user-note"), b"do not discard").expect("write later user data");
    make_finished_copy(&version, b"fresh copy");

    let finished = finish_removals(&control_run(&fixture.root));

    assert!(
        marker.join("user-note").exists(),
        "the changed old copy was treated as a duplicate and its later user file was deleted: {:?}",
        finished.lines
    );
    assert!(version.join("bin/node").exists(), "the finished copy at the name was touched: {:?}", finished.lines);
}

/// What:
/// A deletion a killed pass left is not finished while a live process works inside the tree.
/// Why:
/// The one function asks the scan it was given about the exact path before it renames or unlinks anything.
#[test]
fn keeps_a_deletion_tree_a_live_process_works_inside() {
    let fixture = Fixture::new();
    let version = fixture.root.join("node/20.0.0");
    let deleting: PathBuf = crate::toolchain::store_versions::deleting_of(&version);
    make_finished_copy(&deleting, b"surviving program");
    let mut child = std::process::Command::new("sleep")
        .arg("60")
        .current_dir(&deleting)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("start a process working inside the deletion tree");
    let mut run: TrimRun = control_run(&fixture.root);
    run.holders = Some(StoreHolders {
        held: vec![crate::toolchain::store_path_holders::HeldPath {
            path: deleting.clone(),
            kind: crate::toolchain::store_path_holders::HoldKind::WorksInside,
            pid: child.id(),
        }],
        ..StoreHolders::default()
    });

    let finished = finish_removals(&run);
    let _ = child.kill();
    let _ = child.wait();

    assert!(
        deleting.join("bin/node").exists(),
        "the recovery deleted the rest of a tree a live process works inside: {:?}",
        finished.lines
    );
    assert!(
        finished.lines.iter().any(|line| return line.contains("it is in use")),
        "the line does not say a process holds it: {:?}",
        finished.lines
    );
}

/// What:
/// A flat leftover download a process has open is kept, because the pass asks about the file's own path.
/// Why:
/// A leftover has no version, so a question about the tool's folder is a question about another object.
#[test]
fn keeps_an_open_flat_leftover_download() {
    let (_directory, scope, env) = sample_scope_on_disk();
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"2.0.0","route":"archive"}}}"#)
        .expect("write the lock");
    let leftovers = scope.toolchain_root.join("downloads");
    fs::create_dir_all(&leftovers).expect("make the flat downloads directory");
    let leftover = leftovers.join("node-1.0.0-node-v1.0.0-linux-x64.tar.gz");
    fs::write(&leftover, b"archive a reader still has open").expect("write the leftover");
    written_ago(&leftover, 2 * 24 * 3600);
    let _opened = fs::File::open(&leftover).expect("hold the leftover open");
    let bounds_env = env.with("MPM_SHADOW_STORE_SIZE_BOUND", "0").with("MPM_SHADOW_STORE_AGE_FLOOR", "0");
    let runner = FakeRunner::new();
    runner.answer("git -C", &format!("worktree {}\nHEAD abc\n\n", scope.root.display()), 0);
    let host = Host::current();
    let context = Context { scope: &scope, env: &bounds_env, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };
    let holders = StoreHolders {
        held: vec![crate::toolchain::store_path_holders::HeldPath {
            path: leftover.clone(),
            kind: crate::toolchain::store_path_holders::HoldKind::HasOpen,
            pid: std::process::id(),
        }],
        ..StoreHolders::default()
    };

    trim_after_install_with(&context, &[("node".to_string(), "2.0.0".to_string())], Some(holders));

    assert!(leftover.exists(), "the open flat leftover was deleted because its empty version hid the holder");
}

/// What:
/// A partly deleted downloads tree beside a replacement version is finished, never put back as the version's archives.
/// Why:
/// A deletion name is a name only the deletion step writes, and what stands under it is condemned whatever the
/// version's name holds now.
#[test]
fn finishes_a_partly_deleted_downloads_tree_instead_of_restoring_it() {
    let fixture = Fixture::new();
    let version = fixture.root.join("node/20.0.0");
    make_finished_copy(&version, b"replacement version");
    let downloads = fixture.root.join("downloads/node/20.0.0");
    let deleting: PathBuf = crate::toolchain::store_versions::deleting_of(&downloads);
    fs::create_dir_all(&deleting).expect("make the archives deletion tree");
    fs::write(deleting.join("partial-archive"), b"only the part not yet deleted").expect("write surviving data");

    let finished = finish_removals(&control_run(&fixture.root));

    assert!(
        !downloads.join("partial-archive").exists(),
        "a deletion-name downloads tree was restored beside the replacement version: {:?}",
        finished.lines
    );
}

/// What:
/// Archives belonging to a version that cannot be read are kept, because a failed read is not proof of absence.
/// Why:
/// Only `NotFound` says a path is absent; a permission or I/O failure says nothing about it.
#[test]
fn keeps_archives_while_the_version_cannot_be_statted() {
    let fixture = Fixture::new();
    let tool = fixture.root.join("node");
    let version = tool.join("20.0.0");
    make_finished_copy(&version, b"version still present");
    let downloads_marker = fixture.root.join("downloads/node/.mpm-shadow-removing.20.0.0");
    fs::create_dir_all(&downloads_marker).expect("make the archives removal tree");
    fs::write(downloads_marker.join("archive"), b"still needed").expect("write the archive");
    let mut hidden = fs::metadata(&tool).expect("read the tool directory mode").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut hidden, 0o000);
    fs::set_permissions(&tool, hidden).expect("make the version unreadable");

    let finished = finish_removals(&control_run(&fixture.root));

    let mut restored = fs::metadata(&tool).expect("read the tool directory mode again").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut restored, 0o700);
    fs::set_permissions(&tool, restored).expect("restore the tool directory mode");
    assert!(version.join("bin/node").exists(), "the fixture must keep the version itself");
    assert!(
        downloads_marker.join("archive").exists(),
        "permission errors were treated as proof that the version was gone: {:?}",
        finished.lines
    );
}

/// What:
/// The removal order is the newest write inside each tree, which is the age the decision uses.
/// Why:
/// A tree whose root is old and whose contents are newer is newer than one written before it, whatever the roots say.
#[test]
fn orders_removal_by_the_newest_write_inside_the_tree() {
    let fixture = Fixture::new();
    let old_root_newer_contents = node_version(&fixture.root, "20.0.0");
    let newer_root_older_contents = node_version(&fixture.root, "21.0.0");
    written_ago(&old_root_newer_contents.join("program"), 5 * 24 * 3600);
    written_ago(&old_root_newer_contents, 10 * 24 * 3600);
    written_ago(&newer_root_older_contents.join("program"), 7 * 24 * 3600);
    written_ago(&newer_root_older_contents, 7 * 24 * 3600);
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();
    let candidates = candidates_of(
        std::slice::from_ref(&fixture.root),
        &tools,
        Instant::now() + Duration::from_secs(60),
        Duration::ZERO,
    )
    .into_iter()
    .filter(|candidate| return candidate.version == "20.0.0" || candidate.version == "21.0.0")
    .collect::<Vec<_>>();
    let bounds = TrimBounds { size_bound_bytes: candidates[0].bytes, age_floor: Duration::ZERO, work_limit: Duration::from_secs(60) };

    let plan = plan_trim(&candidates, &bounds, SystemTime::now(), &|_candidate| return None);

    assert_eq!(
        removed_by(&plan),
        vec!["node-21.0.0"],
        "the tree last written seven days ago should go before the tree last written five days ago: {candidates:?}"
    );
}

/// A runner that records the deadline of every request it is given, and answers as a small Git would.
struct RecordedGit {
    deadlines: std::cell::RefCell<Vec<Ceiling>>,
}

impl crate::toolchain::runner::Runner for RecordedGit {
    fn run(&self, request: &crate::toolchain::runner::RunRequest) -> crate::toolchain::runner::RunOutcome {
        self.deadlines.borrow_mut().push(request.deadline);
        return crate::toolchain::runner::RunOutcome { stdout: String::new(), stderr: String::new(), code: 0, cut_off: false };
    }
}

/// What:
/// The Git the cleanup starts is handed the pass's work limit as its deadline.
/// Why:
/// A program the pass starts that never ends held an install for good; the limit has to reach the child.
#[test]
fn hands_the_cleanup_work_limit_to_the_git_it_starts() {
    let (_directory, scope, env) = sample_scope_on_disk();
    fs::create_dir_all(&scope.toolchain_root).expect("make the store the cleanup must visit");
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"2.0.0","route":"archive"}}}"#)
        .expect("write the lock");
    let limit_env = env.with("MPM_SHADOW_STORE_WORK_LIMIT", "5s");
    let runner = RecordedGit { deadlines: std::cell::RefCell::new(Vec::new()) };
    let host = Host::current();
    let context = Context { scope: &scope, env: &limit_env, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };

    trim_after_install_with(&context, &[("node".to_string(), "2.0.0".to_string())], Some(StoreHolders::default()));

    let deadlines: Vec<Ceiling> = runner.deadlines.borrow().clone();
    assert!(!deadlines.is_empty(), "the cleanup did not start Git at all");
    assert!(
        deadlines.iter().all(|deadline| return !deadline.why.is_empty()),
        "Git was started with no bound while the pass had {} of work limit: {deadlines:?}",
        format_span(Duration::from_secs(5))
    );
}

//endregion what the one function proves, and what the seventh review found


//region the ninth review: the names on disk, the locks read before recovery, and the archives in flight

/// What:
/// A sync that recovers a killed deletion reads the lock first, so a version the current lock names keeps its tree.
/// Why:
/// Recovery ran before any lock was read, so a tree a killed cleanup left under a deletion name was deleted though the
/// lock named its version.
#[test]
fn keeps_a_deletion_tree_the_lock_names_when_a_sync_recovers() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let store: PathBuf = scope.toolchain_root.clone();
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"20.0.0","route":"archive"}}}"#)
        .expect("write the lock naming the interrupted version");
    let version: PathBuf = store.join("node/20.0.0");
    let deleting: PathBuf = crate::toolchain::store_versions::deleting_of(&tombstone_of(&version));
    fs::create_dir_all(&deleting).expect("make the interrupted deletion tree");
    fs::write(deleting.join("surviving-user-data"), b"must remain").expect("write surviving data");
    fs::write(unfinished_marker(&store), b"").expect("make recovery run");
    let runner = FakeRunner::new();
    runner.answer("git -C", &format!("worktree {}\nHEAD abc\n\n", scope.root.display()), 0);
    let host = Host::current();
    let context = Context { scope: &scope, env: &env, runner: &runner, http: &NoNetwork, host: &host, tools: &[] };

    trim_after_install_with(&context, &[], Some(StoreHolders::default()));

    assert!(
        deleting.join("surviving-user-data").exists(),
        "the lock named node 20.0.0, but a sync's recovery deleted the surviving data before reading that lock"
    );
}

/// What:
/// A deletion name carries the version last, however many removal prefixes stand in front of it.
/// Why:
/// A normal removal renames a version to its removal name and the deletion renames that again, so a killed deletion
/// leaves a name two prefixes deep; reading the version as the remainder after one prefix made a lock that names that
/// version match nothing.
#[test]
fn keeps_a_double_prefixed_deletion_tree_a_lock_names() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/20.0.0");
    let deleting: PathBuf = crate::toolchain::store_versions::deleting_of(&tombstone_of(&version));
    fs::create_dir_all(&deleting).expect("make the normal interrupted deletion tree");
    fs::write(deleting.join("surviving-user-data"), b"must remain").expect("write surviving data");
    let lock: PathBuf = fixture.root.join("toolchain.lock.json");
    fs::write(&lock, r#"{"tools":{"node":{"version":"20.0.0","route":"archive"}}}"#)
        .expect("write the lock naming the interrupted version");
    let named = read_named_versions(std::slice::from_ref(&lock));
    let parsed = crate::toolchain::store_versions::removal_markers(&fixture.root);
    assert_eq!(parsed.len(), 1, "{parsed:?}");
    assert_eq!(parsed[0].version, "20.0.0", "the version is the name's last part, after every prefix: {parsed:?}");

    let finished = finish_removals_with(&fixture.run(), &named);

    assert!(
        deleting.join("surviving-user-data").exists(),
        "the lock-aware pass deleted data although the lock names 20.0.0; lines={:?}",
        finished.lines
    );
}

/// What:
/// An unreadable lock keeps a killed deletion's tree, because it may name that version.
/// Why:
/// What the pass could not read it takes for present.
#[test]
fn keeps_a_deletion_tree_when_a_lock_cannot_be_read() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/20.0.0");
    let deleting: PathBuf = crate::toolchain::store_versions::deleting_of(&tombstone_of(&version));
    fs::create_dir_all(&deleting).expect("make the interrupted deletion tree");
    fs::write(deleting.join("surviving-user-data"), b"must remain").expect("write surviving data");
    let named = crate::toolchain::store_trim::NamedVersions {
        by_tool: std::collections::BTreeMap::new(),
        unreadable: Some("the lock cannot be read".to_string()),
    };

    let finished = finish_removals_with(&fixture.run(), &named);

    assert!(
        deleting.join("surviving-user-data").exists(),
        "a tree was deleted while a lock that may name it could not be read: {:?}",
        finished.lines
    );
}

/// What:
/// The only complete archive is kept while the replacement download is still in flight.
/// Why:
/// A live downloads directory that holds only a partial download is not a finished replacement; deleting the
/// moved-aside archives loses the only complete copy.
#[test]
fn keeps_the_only_complete_archive_while_a_replacement_downloads() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    assert!(version.join("program").exists(), "the live version must exist");
    let downloads: PathBuf = fixture.root.join("downloads/node/26.11.0");
    let marker: PathBuf = tombstone_of(&downloads);
    fs::create_dir_all(&downloads).expect("make the complete archives");
    fs::write(downloads.join("complete-archive"), b"the only complete archive").expect("write the complete archive");
    fs::rename(&downloads, &marker).expect("model the interrupted archive move");
    fs::create_dir_all(&downloads).expect("start replacement downloads");
    fs::write(downloads.join("replacement.tar.download"), b"partial replacement").expect("write an in-flight download");

    let finished = finish_removals(&fixture.run());

    assert!(
        marker.join("complete-archive").exists(),
        "recovery deleted the only complete archive and left just an in-flight download: {:?}",
        finished.lines
    );
    assert!(
        finished.lines.iter().any(|line| return line.contains("in flight")),
        "the line does not say why the archives stayed: {:?}",
        finished.lines
    );
}

//endregion the ninth review: the names on disk, the locks read before recovery, and the archives in flight

fn immediate_trim_env(env: &EnvMap) -> EnvMap {
    return env
        .with("MPM_SHADOW_STORE_SIZE_BOUND", "0")
        .with("MPM_SHADOW_STORE_AGE_FLOOR", "0s")
        .with("MPM_SHADOW_STORE_WORK_LIMIT", "5s");
}

fn run_trim(scope: &Scope, env: &EnvMap, runner: &FakeRunner) {
    let locked = sample_lock();
    let http = FakeHttp::new();
    let host = Host::current();
    let context = Context { scope, env, runner, http: &http, host: &host, tools: &locked.tools };
    trim_after_install_with(&context, &[("unrelated".to_string(), "1".to_string())], Some(StoreHolders::default()));
}

#[test]
fn reproduction_cargo_package_id_cannot_escape_the_store() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let bounds_env = immediate_trim_env(&env);
    let runner = FakeRunner::new();
    let outside = scope.toolchain_root.parent().expect("store parent").join("outside");
    let user_data = outside.join("old").join("user-data.txt");
    std::fs::create_dir_all(scope.toolchain_root.join("cargo").join("safe")).expect("make traversal prefix");
    std::fs::create_dir_all(user_data.parent().expect("data parent")).expect("make outside version");
    std::fs::write(&user_data, b"not owned by the toolchain store").expect("write outside data");
    std::fs::write(
        &scope.lock_file,
        r#"{"tools":{"cargo:review":{"version":"locked","route":"cargo-binstall","packageId":"safe/../../../outside"}}}"#,
    )
    .expect("write lock with escaping package id");

    run_trim(&scope, &bounds_env, &runner);

    assert!(
        user_data.exists(),
        "cleanup left the store through packageId and deleted {}",
        user_data.display()
    );
}

#[test]
fn reproduction_disappearing_current_lock_keeps_the_loaded_pin() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let bounds_env = immediate_trim_env(&env);
    let runner = FakeRunner::new();
    let locked = sample_lock();
    let node = locked.tools.iter().find(|tool| return tool.declaration.id == "node").expect("sample node");
    let user_data = scope.toolchain_root.join("node").join(&node.version).join("user-data.txt");
    std::fs::create_dir_all(user_data.parent().expect("version parent")).expect("make pinned version");
    std::fs::write(&user_data, b"the version loaded from the current lock").expect("write pinned data");

    let global = global_scope(&env).expect("global scope");
    std::fs::create_dir_all(global.lock_file.parent().expect("global lock parent")).expect("make global lock parent");
    std::fs::write(
        &global.lock_file,
        r#"{"tools":{"node":{"version":"different","route":"archive"}}}"#,
    )
    .expect("write an unrelated global lock so cleanup still scans the store");

    // sync_tools has already decoded sample_lock into Context at this point.
    // A concurrent lock replacement removes the old path before cleanup re-reads it.
    std::fs::remove_file(&scope.lock_file).expect("remove the current lock after Context was loaded");
    let http = FakeHttp::new();
    let host = Host::current();
    let context = Context { scope: &scope, env: &bounds_env, runner: &runner, http: &http, host: &host, tools: &locked.tools };
    trim_after_install_with(&context, &[("unrelated".to_string(), "1".to_string())], Some(StoreHolders::default()));

    assert!(
        user_data.exists(),
        "cleanup ignored the pin already present in Context and deleted {}",
        user_data.display()
    );
}

/// Keeps an interrupted deletion when the lock snapshot did not name it but the lock does at deletion time.
#[test]
fn recovery_rereads_a_lock_before_deleting() {
    let fixture = Fixture::new();
    // Make the deletion name the real cleanup leaves after starting to unlink a version.
    let deleting = crate::toolchain::store_versions::deleting_of(&tombstone_of(&fixture.root.join("node/20.0.0")));
    let payload = deleting.join("surviving-user-data");
    // All files and the lock belong to the disposable fixture.
    fs::create_dir_all(&deleting).expect("make the interrupted deletion tree");
    fs::write(&payload, b"must remain").expect("write surviving data");
    let lock = fixture.root.join("toolchain.lock.json");
    // Read the first lock while it names a different version.
    fs::write(&lock, r#"{"tools":{"node":{"version":"21.0.0","route":"archive"}}}"#).expect("write initial lock");
    let named = read_named_versions(std::slice::from_ref(&lock));
    // Prove the test would not stop at the snapshot's protection check.
    assert!(named.unreadable.is_none());
    assert_eq!(named.naming_lock("node", "20.0.0"), None);
    // Model a replacement after the snapshot without depending on scheduling.
    fs::write(&lock, r#"{"tools":{"node":{"version":"20.0.0","route":"archive"}}}"#).expect("replace lock");
    let mut run = fixture.run();
    run.locks.push(lock);

    let finished = finish_removals_with(&run, &named);

    // Require both the retained bytes and the diagnostic from the late check.
    assert!(payload.exists(), "the late lock named node 20.0.0, but recovery deleted it: {:?}", finished.lines);
    assert!(finished.lines.iter().any(|line| return line.contains("names it now")), "{:?}", finished.lines);
}

/// Keeps the remaining bytes of a deletion when a tool's `current` link has been repointed to that version.
#[cfg(unix)]
#[test]
fn recovery_rechecks_current_before_deleting() {
    let fixture = Fixture::new();
    // A deletion can be partly complete; the remaining payload still must not be removed while named.
    let deleting = crate::toolchain::store_versions::deleting_of(&tombstone_of(&fixture.root.join("node/20.0.0")));
    let payload = deleting.join("surviving-user-data");
    // Make the interrupted deletion and the link entirely inside this test's store.
    fs::create_dir_all(&deleting).expect("make the interrupted deletion tree");
    fs::write(&payload, b"must remain").expect("write surviving data");
    let link = fixture.root.join("node").join(crate::toolchain::cargo_installs::CURRENT_LINK_NAME);
    // The link names the live path even when deletion has moved that path out of the way.
    std::os::unix::fs::symlink("20.0.0", &link).expect("point current at the interrupted version");

    let finished = finish_removals(&fixture.run());

    // Require both the retained bytes and the diagnostic from the late check.
    assert!(payload.exists(), "current named node 20.0.0, but recovery deleted it: {:?}", finished.lines);
    assert!(finished.lines.iter().any(|line| return line.contains("current link names it now")), "{:?}", finished.lines);
}

/// Retains two distinct names by newest write across the stores, plus an older pin.
#[test]
fn retains_the_two_newest_distinct_versions_plus_an_old_pin() {
    let (_directory, scope, env) = sample_scope_on_disk();
    // A lock pins an older version, independently of the two newest names.
    fs::write(&scope.lock_file, r#"{"tools":{"node":{"version":"old-pin","route":"archive"}}}"#)
        .expect("write a pin outside the newest two");
    let repo = &scope.toolchain_root;
    let global = &scope.other_toolchain_root;
    // Name ordering deliberately disagrees with write ordering, and the newest name has two copies.
    let name_by_recency = ["a-newest", "b-runner-up", "z-outdated", "old-pin"];
    let hour = DEFAULT_AGE_FLOOR_SECONDS;
    let mut paths = Vec::new();
    // Give the program as well as its directory a deterministic age.
    for (index, name) in name_by_recency.iter().enumerate() {
        let root = if index == 1 { global } else { repo };
        let path = node_version(root, name);
        let age = hour * (u64::try_from(index).expect("fixture index fits u64") + 1);
        written_ago(&path.join("program"), age);
        written_ago(&path, age);
        paths.push(path);
    }
    // The duplicate is newer than the runner-up but must not take its slot.
    let duplicate = node_version(global, name_by_recency[0]);
    let runner = FakeRunner::new();

    run_trim(&scope, &immediate_trim_env(&env), &runner);

    assert!(!paths[2].exists(), "the unpinned version outside the two newest survived the zero-byte cap");
    assert!(paths[3].exists(), "the independent pin did not keep the oldest version");
    assert!(paths[0].exists() && duplicate.exists(), "cleanup deleted a copy of the newest version");
    assert!(paths[1].exists(), "duplicate copies of the newest version used both retention slots");
}

/// A damaged lock blocks recovery of an orphaned archive and identifies the whole lock and its unreadable row.
#[test]
fn retains_recovery_archives_and_names_the_damaged_lock() {
    let fixture = Fixture::new();
    let lock = fixture.root.join("damaged-toolchain.lock.json");
    // The malformed route might name the orphaned archives. There is no installed version beside them.
    fs::write(&lock, r#"{"tools":{"node":{"version":"20.0.0","route":"archive-v-next"}}}"#)
        .expect("make the damaged lock");
    let named = read_named_versions(std::slice::from_ref(&lock));
    let aside = tombstone_of(&fixture.root.join("downloads/node/20.0.0"));
    let archive = aside.join("only-archive.tar.gz");
    fs::create_dir_all(&aside).expect("make the archive-only removal marker");
    fs::write(&archive, b"the only complete archive").expect("write the archive");
    // Repairing the lock does not make this pass's incomplete snapshot a complete one.
    fs::write(&lock, r#"{"tools":{}}"#).expect("replace the fixture lock with a readable lock");
    let mut run = fixture.run();
    run.locks.push(lock.clone());

    let finished = finish_removals_with(&run, &named);

    assert!(archive.exists(), "recovery deleted an archive the damaged lock might pin: {:?}", finished.lines);
    assert!(finished.unfinished.contains(&fixture.root), "recovery lost its retry marker: {finished:?}");
    assert!(
        finished.lines.iter().any(|line| return line.contains(&lock.display().to_string()) && line.contains("tools.node")),
        "recovery did not name the damaged lock and its row: {:?}",
        finished.lines
    );
}

/// Recognized old flat downloads do not need the stores to be over their version budget.
#[test]
fn deletes_an_obsolete_flat_download_under_the_byte_bound() {
    let (_directory, scope, env) = sample_scope_on_disk();
    // Make a pinned, complete installed copy and a finished flat archive from the old layout.
    let version = node_version(&scope.toolchain_root, "1.2.3");
    fs::write(version.join(crate::toolchain::archive_staging::COMPLETE_MARKER), b"").expect("finish the install");
    let downloads = scope.toolchain_root.join("downloads");
    fs::create_dir_all(&downloads).expect("make downloads directory");
    let flat = downloads.join("node-1.2.3-node-v1.2.3-linux-x64.tar.gz");
    let partial = downloads.join("node-1.2.3-replacement.tar.gz.download");
    fs::write(&flat, b"obsolete layout").expect("write the flat archive");
    fs::write(&partial, b"not done").expect("write an in-flight download");
    written_ago(&flat, DEFAULT_AGE_FLOOR_SECONDS * 2);
    let runner = FakeRunner::new();
    let under_bound = env.with("MPM_SHADOW_STORE_WORK_LIMIT", "5s");

    run_trim(&scope, &under_bound, &runner);

    assert!(!flat.exists(), "obsolete flat download survived under the byte bound");
    assert!(partial.exists(), "the flat cleanup removed a download in flight");
    assert!(version.join("program").exists(), "the flat cleanup removed the pinned install");
}
