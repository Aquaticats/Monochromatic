//! What:
//! Unit tests of the size- and age-bounded cleanup in `store_trim.rs`:
//! the rule,
//! the bounds it reads and the marks it prints,
//! the locks whose pins protect a version,
//! and the removal itself,
//! including one cut short and finished by the next pass.
//! Why:
//! The pass deletes installed toolchains,
//! so each thing it must never remove (a version a lock names,
//! one this run installed,
//! one a running process holds,
//! one written inside the age floor,
//! one the `current` link names) and each thing it must remove is a test,
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
use crate::toolchain::store_trim::{
    ClaimedVersion, DEFAULT_SIZE_BOUND_BYTES, TrimBounds, TrimCandidate, TrimPlan, TrimRun, TrimStep, apply_trim,
    candidates_of, DEFAULT_AGE_FLOOR_SECONDS, TrimOutcome, finish_removals, format_age, has_unfinished_removal,
    parse_duration, parse_size, plan_trim, read_named_versions, settle_markers, trim_after_install, unfinished_marker,
};
use crate::toolchain::util::{ProcessIdentity, this_process};
use crate::toolchain::store_versions::{TOMBSTONE_PREFIX, tombstone_of};
use crate::test_scratch::scratch;

/// A deadline no test reaches,
/// for an inventory a test asks for in full.
fn far_deadline() -> Instant {
    return Instant::now() + Duration::from_secs(600);
}

/// A mebibyte,
/// so a test's bound reads as a fixture size.
const MIB: u64 = 1024 * 1024;

/// A bound that keeps nothing and removes nothing inside an hour,
/// for the pure-rule tests.
fn tight_bounds(size_bound_bytes: u64) -> TrimBounds {
    return TrimBounds { size_bound_bytes, age_floor: Duration::ZERO, work_limit: Duration::from_secs(60) };
}

/// One candidate with a made-up path,
/// for the pure-rule tests.
fn candidate(tool: &str, version: &str, bytes: u64, written_seconds_ago: u64) -> TrimCandidate {
    return TrimCandidate {
        root: PathBuf::from("/store"),
        tool: tool.to_string(),
        version: version.to_string(),
        directory: PathBuf::from(format!("/store/{tool}/{version}")),
        downloads: None,
        bytes,
        written: SystemTime::now() - Duration::from_secs(written_seconds_ago),
        naming: crate::toolchain::store_versions::EntryNaming::default(),
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

/// The versions a plan keeps,
/// with why.
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

/// While the stores keep no more than the bound,
/// nothing is removed and nothing is said.
#[test]
fn removes_nothing_while_under_the_bound() {
    let candidates = vec![candidate("node", "26.12.0", MIB, 3600), candidate("node", "26.13.1", MIB, 60)];
    let plan: TrimPlan = plan_trim(&candidates, &tight_bounds(2 * MIB), SystemTime::now(), &|_found| return None);

    assert!(!plan.over_bound);
    assert!(plan.steps.is_empty(), "{:?}", plan.steps);
}

/// Over the bound,
/// the versions written longest ago go until the space is inside it,
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

/// When everything over the bound is protected,
/// the plan removes nothing and says why for each.
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

/// The age floor and the work limit are read from seconds,
/// suffixes or milliseconds.
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
        return candidates_of(std::slice::from_ref(&self.root), &tools, far_deadline()).0;
    }

    fn run(&self) -> TrimRun {
        return TrimRun {
            bounds: TrimBounds {
                size_bound_bytes: 0,
                age_floor: Duration::ZERO,
                work_limit: Duration::from_secs(60),
            },
            started: Instant::now(),
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

    let (lines, unfinished) = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());

    assert!(unfinished.is_empty(), "{unfinished:?}");
    assert!(!marker.exists(), "the marker is still there");
    assert!(!version.exists());
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains(&format!("{TOMBSTONE_PREFIX}26.11.0")), "{lines:?}");
    assert!(lines[0].contains("finished a removal an earlier run started"), "{lines:?}");
}

/// A process number no machine reaches,
/// for the name of a claim a killed removal left.
const A_PROCESS_NUMBER_NO_MACHINE_HAS: u32 = u32::MAX - 1;

/// The claimed name a removal killed at `pid` left for `version` of a tool,
/// spelled the way the review wrote it out:
/// the version,
/// the process's number and start time,
/// and which claim of that process it is.
/// The spelling is deliberate here rather than taken from the code,
/// so a change to the shape has to change this line too.
fn claimed_name(version: &str, pid: u32, started: u64) -> String {
    return format!(".mpm-shadow-claimed.{version}.{pid}.{started}.0.claimed");
}

/// What:
/// The name a removal writes and the name the recovery pass reads are one definition,
/// and a version comes back out of it exactly as it went in,
/// whatever it holds.
/// Why:
/// The two halves disagreed once:
/// the writer fell back to a name with one field fewer than the reader expected,
/// so a killed removal's dotted version came back under a made-up name (`26.11.0` as `11.0`)
/// and a plain number was never given back at all.
/// A version can hold dots,
/// dashes,
/// plus signs,
/// underscores and the word `claimed`,
/// so the encoding has to be exact for every one of them.
#[test]
fn a_claim_name_reads_back_as_the_version_it_was_written_with() {
    let awkward: [&str; 13] = [
        "26.11.0",
        "1",
        "123",
        "v1.2.3",
        "0.4.5-alpha.9",
        "1.0.0+build.7",
        "2024.01.01-rc.1+build-5",
        "9.9.9_1",
        "a.b.c",
        "x",
        "1.2.3.4.5.6.7",
        "26.11.0.claimed",
        "nightly-2026-11-14",
    ];
    let maker: ProcessIdentity = ProcessIdentity { pid: A_PROCESS_NUMBER_NO_MACHINE_HAS, started: 123456 };
    for version in awkward {
        for attempt in [0u32, 7, u32::MAX] {
            let written: ClaimedVersion = ClaimedVersion { version: version.to_string(), maker, attempt };
            let name: String = written.to_name();
            assert_eq!(ClaimedVersion::parse(&name), Some(written.clone()), "{name} did not read back");
            assert!(!name.contains('/'), "{name} is not one directory name");
        }
    }
}

/// What:
/// A claim another build of this branch wrote under its two-field name is not read as one version or another:
/// the pass leaves it where it is.
/// Why:
/// That name is `.mpm-shadow-claimed.<process number>.<version>`,
/// which cannot be told from a version that begins with a number and a dot
/// (`.mpm-shadow-claimed.26.11.0` is version `26.11.0` or process `26` and version `11.0`),
/// and restoring a version under a name it did not have is worse than leaving it for a person to move back.
#[test]
fn a_claim_an_older_build_wrote_is_left_where_it_is() {
    let older_shapes: [&str; 2] = [".mpm-shadow-claimed.4294967294.26.11.0", ".mpm-shadow-claimed.4294967294.0.26.11.0"];
    for name in older_shapes {
        assert_eq!(ClaimedVersion::parse(name), None, "{name} was read as a claim of this build");
    }
}

/// What:
/// A version a killed removal took out of its name and never judged is given back by the next pass,
/// and nothing is deleted on the way.
/// Why:
/// The claim is what lets a removal compare what it took with what it judged,
/// and a run killed between the claim and that comparison would otherwise leave the version under a hidden name
/// no scan lists and no tombstone pass knows,
/// until a person moved it back by hand.
#[test]
fn gives_back_a_version_a_killed_removal_took_out_of_its_name() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let claimed: PathBuf = fixture
        .root
        .join("node")
        .join(claimed_name("26.11.0", A_PROCESS_NUMBER_NO_MACHINE_HAS, 987_654));
    fs::rename(&version, &claimed).expect("leave the claim a kill leaves");
    let tools: BTreeSet<String> = ["node".to_string(), "aws".to_string()].into_iter().collect();

    let (lines, unfinished) = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());

    assert!(unfinished.is_empty(), "{unfinished:?}");
    assert!(!claimed.exists(), "the claim is still there");
    assert!(version.join("program").is_file(), "the version is back with what it held");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("gave back 26.11.0"), "{lines:?}");
    assert!(lines[0].contains("taken out of its name"), "{lines:?}");
}

/// The claim of a run that is still working is that run's own:
/// the pass reads the process's number and start time in the name and leaves it alone.
#[test]
fn leaves_the_claim_of_a_run_that_is_still_working() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.12.0");
    let maker: ProcessIdentity = this_process().expect("this process's start time is readable");
    let claimed: PathBuf = fixture.root.join("node").join(claimed_name("26.12.0", maker.pid, maker.started));
    fs::rename(&version, &claimed).expect("move the version to the claim of this process");
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();

    let (lines, unfinished) = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());

    assert!(lines.is_empty(), "this pass says nothing about a claim another run holds: {lines:?}");
    assert!(unfinished.is_empty(), "{unfinished:?}");
    assert!(claimed.exists(), "the claim of a live run was touched");
    assert!(!version.exists());
}

/// What:
/// A claim whose process number a later process took is given back:
/// the claim names the run that made it by its start time too,
/// so a stranger holding the dead run's number does not
/// hold its claim.
/// Why:
/// The review of this change noted that a reused number postpones the recovery for as long as the new process lives,
/// and a service that keeps a number makes that indefinite.
#[test]
fn gives_back_a_claim_whose_process_number_a_later_process_took() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let maker: ProcessIdentity = this_process().expect("this process's start time is readable");
    // This process's number, and a start time no process with that number has: the number was reused after the kill.
    let claimed: PathBuf = fixture
        .root
        .join("node")
        .join(claimed_name("26.11.0", maker.pid, maker.started.saturating_add(1)));
    fs::rename(&version, &claimed).expect("leave the claim a killed removal left");
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();

    let (lines, unfinished) = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());

    assert!(unfinished.is_empty(), "{unfinished:?}");
    assert!(version.join("program").is_file(), "the version was not given back");
    assert!(!claimed.exists(), "the claim is still there");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("gave back 26.11.0"), "{lines:?}");
}

/// What:
/// Two passes that recover one claim at once say nothing false and leave no unfinished marker:
/// the pass that loses the rename finds the claim gone,
/// which is the state it wanted.
/// Why:
/// The review of this change ran this race 300 times in 300 rounds and every round printed
/// "kept ... cannot be given back:
/// No such file or directory" from the losing pass and marked the root unfinished.
/// The store is shared by every worktree,
/// so two installs reach this pass together.
#[test]
fn two_recoveries_of_one_claim_at_once_say_nothing_false() {
    let fixture = Fixture::new();
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();
    let rounds: u32 = 20;
    let claims: u32 = 40;
    for round in 0..rounds {
        let root: PathBuf = fixture.root.join(format!("node-round-{round}"));
        let tool_directory: PathBuf = root.join("node");
        fs::create_dir_all(&tool_directory).expect("make the tool directory");
        for claim in 0..claims {
            let version: String = format!("1.0.{claim}");
            let claimed: PathBuf = tool_directory.join(claimed_name(&version, A_PROCESS_NUMBER_NO_MACHINE_HAS, 987_654));
            fs::create_dir_all(&claimed).expect("leave a claim");
            fs::write(claimed.join("program"), b"bytes").expect("write the version's program");
        }
        let barrier: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(2));
        let mut passes: Vec<std::thread::JoinHandle<(Vec<String>, BTreeSet<PathBuf>)>> = Vec::new();
        for _pass in 0..2 {
            let pass_root: PathBuf = root.clone();
            let pass_tools: BTreeSet<String> = tools.clone();
            let pass_barrier: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::clone(&barrier);
            passes.push(std::thread::spawn(move || {
                pass_barrier.wait();
                return finish_removals(std::slice::from_ref(&pass_root), &pass_tools, &fixture_run());
            }));
        }
        for pass in passes {
            let (lines, unfinished) = pass.join().expect("the pass finished");
            for line in &lines {
                assert!(!line.contains("cannot be given back"), "a pass reported work the other pass did: {line}");
            }
            assert!(unfinished.is_empty(), "a pass marked {} unfinished, with nothing left to do", unfinished.len());
        }
        // Control: every claim was given back exactly once, so the rounds are not passing on an empty pass.
        for claim in 0..claims {
            assert!(tool_directory.join(format!("1.0.{claim}")).join("program").is_file(), "version 1.0.{claim} is not back");
        }
    }
}

/// A pass's run for the concurrency control,
/// with a work limit that fits every claim of one round.
fn fixture_run() -> TrimRun {
    return TrimRun {
        bounds: TrimBounds { size_bound_bytes: 0, age_floor: Duration::ZERO, work_limit: Duration::from_secs(60) },
        started: Instant::now(),
    };
}

/// The removal marker is what makes the next install run the pass,
/// and it goes once nothing is left to remove.
#[test]
fn settles_the_marker_of_an_unfinished_removal() {
    let fixture = Fixture::new();
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the marker");
    assert!(has_unfinished_removal(&fixture.root));

    settle_markers(std::slice::from_ref(&fixture.root), &tools, &BTreeSet::new());

    assert!(!has_unfinished_removal(&fixture.root));
}

/// A marker is not cleared while a removal is still pending,
/// so the cleanup is not forgotten with a version half removed.
#[test]
fn keeps_the_marker_while_a_removal_is_pending() {
    let fixture = Fixture::new();
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();
    fs::write(unfinished_marker(&fixture.root), b"").expect("write the marker");
    let pending: BTreeSet<PathBuf> = [fixture.root.clone()].into_iter().collect();

    settle_markers(std::slice::from_ref(&fixture.root), &tools, &pending);

    assert!(has_unfinished_removal(&fixture.root));
}

/// A version another run wrote after the scan is not the one that was judged,
/// so it stays.
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
    // Older than the in-use floor: no install is unpacking it, so the removal may take it.
    crate::toolchain::fixtures::age_file(&downloads.join("node.tar.gz"), Duration::from_secs(2 * 60 * 60));
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

/// The listing `MPM_SHADOW_STORE_LISTING` names,
/// decoded,
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
    let candidates: Vec<TrimCandidate> = candidates_of(&replayed_roots, &tools, far_deadline()).0;
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
    let outcome: TrimOutcome = apply_trim(&plan, &TrimRun { bounds, started: Instant::now() });
    lines.push(format!(
        "removed from the replayed store: {} version(s), {} MiB",
        outcome.removed,
        outcome.removed_bytes / MIB
    ));
    let report: String = std::env::var("MPM_SHADOW_STORE_LISTING_REPORT").expect("the report path");
    fs::write(&report, format!("{}\n", lines.join("\n"))).expect("write the replay report");
}

//endregion replaying a real store's listing

/// What:
/// A version whose downloads an install is unpacking stays,
/// with the file the install is reading,
/// and the line says why.
/// Why:
/// The size- and write-time cleanup judges a version from a snapshot of the locks and the process table,
/// and a command in another worktree may start installing that very version after the snapshot was read;
/// moving its downloads aside fails that install.
#[test]
fn keeps_a_version_whose_downloads_an_install_is_unpacking() {
    let fixture = Fixture::new();
    let downloads: PathBuf = fixture.root.join("downloads/node/26.11.0");
    fs::create_dir_all(&downloads).expect("make the downloads directory");
    let in_flight: PathBuf = downloads.join("node-v26.11.0.tar.gz");
    fs::write(&in_flight, vec![b'z'; 40]).expect("write the archive an install is unpacking");
    let held: fs::File = fs::File::open(&in_flight).expect("hold the archive open, as the install does");
    let candidates: Vec<TrimCandidate> = fixture.candidates();
    let judged: TrimCandidate = candidates
        .iter()
        .find(|found| return found.version == "26.11.0")
        .expect("the oldest version is scanned")
        .clone();
    let plan: TrimPlan = TrimPlan { total_bytes: judged.bytes, over_bound: true, steps: vec![TrimStep::Remove(judged)] };

    let outcome = apply_trim(&plan, &fixture.run());

    assert_eq!(outcome.removed, 0, "{:?}", outcome.lines);
    assert!(fixture.root.join("node/26.11.0").join("program").is_file(), "the version was removed under an install");
    assert!(in_flight.is_file(), "the archive an install is unpacking was taken");
    assert_eq!(fs::read(&in_flight).expect("read it back, as the install does next"), vec![b'z'; 40]);
    assert!(
        outcome.lines.iter().any(|line| return line.contains("an install is using its downloads")),
        "{:?}",
        outcome.lines
    );
    drop(held);
}

/// What:
/// The pass an install runs ends by removing the archives of a version this root no longer has,
/// so the archives a removal kept for an install are not left on the disk for ever.
/// Why:
/// The removal keeps them,
/// and nothing else reads a downloads directory whose version is gone,
/// so a call that is not made here means a leak nobody clears.
#[test]
fn the_install_pass_removes_the_archives_of_a_version_that_is_gone() {
    let rig = crate::toolchain::fakes::Rig::new();
    let root = rig.scope.toolchain_root.clone();
    // The sample lock names `bin`, whose versions live under `<root>/bin`.
    let orphan: PathBuf = crate::toolchain::store_versions::downloads_of(&root, "bin", "0.1.0");
    fs::create_dir_all(&orphan).expect("make the orphan's downloads");
    fs::write(orphan.join("bin-0.1.0.tar.gz"), b"old").expect("write the archive");
    crate::toolchain::fixtures::age_file(&orphan.join("bin-0.1.0.tar.gz"), Duration::from_secs(2 * 60 * 60));
    rig.runner.answer(
        "git -C",
        &format!("worktree {}\nHEAD 0000\n\n", rig.scope.root.display()),
        0,
    );

    crate::toolchain::store_trim::trim_after_install(&rig.context(), &[("bin".to_string(), "1.2.3".to_string())]);

    assert!(!orphan.exists(), "the pass left the archives of a version this root does not have");
}

//region what a claim under a root keeps, and what the inventory's deadline does

/// What:
/// The recovery marker stays while any claim is under the root,
/// alive or dead,
/// and goes once the last claim is given back.
/// Why:
/// The review of this change ran a second sync while a live removal owned a hidden claim:
/// the pass left the claim (its owner was alive),
/// did not mark the root unfinished,
/// and `settle_markers` deleted the
/// marker because no tombstone was left.
/// The owner was then killed,
/// and every later sync that installed nothing returned early,
/// so the version stayed hidden
/// under its claimed name for good.
#[test]
fn the_recovery_marker_stays_while_a_claim_is_under_the_root() {
    let fixture = Fixture::new();
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();
    let version: PathBuf = fixture.root.join("node/26.12.0");
    let maker: ProcessIdentity = this_process().expect("this process's start time is readable");
    let claimed: PathBuf = fixture
        .root
        .join("node")
        .join(claimed_name("26.12.0", maker.pid, maker.started));
    fs::rename(&version, &claimed).expect("model a removal after its claim");
    fs::write(unfinished_marker(&fixture.root), b"").expect("model its recovery marker");

    let (lines, unfinished) = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());
    assert!(lines.is_empty(), "the claim of a live removal is not this pass's to move: {lines:?}");
    settle_markers(std::slice::from_ref(&fixture.root), &tools, &unfinished);

    assert!(
        has_unfinished_removal(&fixture.root),
        "the marker was cleared while a claim was still under the root"
    );

    // Control: once the claim is gone, a pass settles the marker, so it is not kept for ever.
    fs::rename(&claimed, &version).expect("give the version back by hand");
    let (_, second_pass_unfinished) = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());
    settle_markers(std::slice::from_ref(&fixture.root), &tools, &second_pass_unfinished);
    assert!(!has_unfinished_removal(&fixture.root), "the marker stayed with nothing left to do");
}

/// What:
/// An inventory whose deadline has passed is not the whole store,
/// and it says so.
/// Why:
/// The review of this change found the cleanup's work limit does not bound its filesystem work:
/// the inventory walked every version and measured every tree with no deadline,
/// so a large store made the pass overrun its bound by the whole store's cost.
#[test]
fn an_inventory_past_its_deadline_stops_and_says_so() {
    let fixture = Fixture::new();
    let tools: BTreeSet<String> = ["node".to_string(), "aws".to_string()].into_iter().collect();

    let (candidates_in_time, stopped_in_time) =
        candidates_of(std::slice::from_ref(&fixture.root), &tools, Instant::now() + Duration::from_secs(60));
    assert!(stopped_in_time.is_empty(), "the inventory in time has nothing to report: {stopped_in_time:?}");
    assert_eq!(candidates_in_time.len(), 4, "the whole fixture is in the inventory in time");

    // A deadline that has already passed: the scan stops at its first check and says why it is not the whole store.
    let (candidates_late, stopped_late) = candidates_of(std::slice::from_ref(&fixture.root), &tools, Instant::now());
    assert!(candidates_late.is_empty(), "a scan past its deadline listed versions after all: {candidates_late:?}");
    assert_eq!(stopped_late.len(), 1, "the scan says it is not the whole store: {stopped_late:?}");
    assert!(stopped_late[0].contains("ran out of its time limit"), "{stopped_late:?}");
}

//endregion

//region the table: the names a removal leaves, and who puts them right

/// What:
/// A recovery gives a claimed version back and its archives with it.
/// Why:
/// The removal moves the archives aside before the version, so a kill between the two leaves the version under its
/// claim and the archives under their tombstone;
/// the review of this change showed the version restored and the tombstone pass then deleting the archives,
/// which leaves the version without the files it was installed with.
#[test]
fn a_recovery_gives_a_version_back_with_its_downloads() {
    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.11.0");
    let downloads: PathBuf = fixture.root.join("downloads/node/26.11.0");
    fs::create_dir_all(&downloads).expect("make the downloads");
    fs::write(downloads.join("node.tar.gz"), b"archive").expect("write the archive");
    let downloads_marker: PathBuf = tombstone_of(&downloads);
    fs::rename(&downloads, &downloads_marker).expect("the removal moves the downloads aside first");
    let claimed: PathBuf = fixture.root.join("node").join(claimed_name("26.11.0", A_PROCESS_NUMBER_NO_MACHINE_HAS, 987_654));
    fs::rename(&version, &claimed).expect("a kill lands after the version claim");
    fs::write(unfinished_marker(&fixture.root), b"").expect("leave the recovery marker");
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();

    let (lines, _unfinished) = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());

    assert!(version.join("program").is_file(), "the claimed version was not given back: {lines:?}");
    assert!(
        downloads.join("node.tar.gz").is_file(),
        "the version came back without the archives the removal moved aside: {lines:?}"
    );
    assert!(!downloads_marker.exists(), "the downloads tombstone is still there");
}

/// What:
/// A downloads tombstone a live removal holds is left for that removal,
/// and goes once the removal is done.
/// Why:
/// A removal moves the archives aside before it takes the version,
/// and in that window the downloads claim is the only record of it;
/// the review showed a concurrent pass deleting the tombstone there and the removal's restore then finding nothing
/// to put back, so the version it kept had no archives.
#[test]
fn a_downloads_tombstone_a_live_removal_holds_is_not_deleted() {
    let fixture = Fixture::new();
    let downloads: PathBuf = fixture.root.join("downloads/node/26.12.0");
    fs::create_dir_all(&downloads).expect("make the downloads");
    fs::write(downloads.join("node.tar.gz"), b"archive").expect("write the archive");
    // Removal A takes the right and moves the archives aside, which is the step before its version claim.
    let right = crate::toolchain::download_ownership::take_removal_right(&fixture.root, "node", "26.12.0")
        .expect("take the right to remove");
    let held = match right {
        crate::toolchain::download_ownership::RemovalRight::Held { claim } => claim,
        _ => panic!("a removal that asked first did not get the right"),
    };
    let downloads_marker: PathBuf = tombstone_of(&downloads);
    fs::rename(&downloads, &downloads_marker).expect("removal A moves the archives aside");
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();

    let _ = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());
    assert!(downloads_marker.is_dir(), "a pass deleted the archives a live removal had moved aside");

    // Control: with the right given up, the same pass finishes the tombstone.
    drop(held);
    let (lines, _unfinished) = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());
    assert!(!downloads_marker.exists(), "the tombstone stayed with no removal over it: {lines:?}");
}

/// What:
/// A downloads tombstone whose version a claim holds is left too,
/// because giving the version back restores the archives with it.
/// Why:
/// The claim pass runs before the tombstone pass in one root,
/// so a version it gave back has its archives back by the time the tombstones are looked at;
/// a claim it left (a live removal's) still names work the archives belong to.
#[test]
fn a_downloads_tombstone_a_version_claim_holds_is_not_deleted() {
    let fixture = Fixture::new();
    let tool_directory: PathBuf = fixture.root.join("node");
    let downloads: PathBuf = fixture.root.join("downloads/node/26.13.1");
    fs::create_dir_all(&downloads).expect("make the downloads");
    fs::write(downloads.join("node.tar.gz"), b"archive").expect("write the archive");
    let downloads_marker: PathBuf = tombstone_of(&downloads);
    fs::rename(&downloads, &downloads_marker).expect("a removal moved the archives aside");
    let maker: ProcessIdentity = this_process().expect("this process's start time is readable");
    let claimed: PathBuf = tool_directory.join(claimed_name_made_by("26.13.1", maker));
    fs::create_dir_all(&claimed).expect("a directory standing in for the live claim");
    let tools: BTreeSet<String> = ["node".to_string()].into_iter().collect();

    let _ = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());
    assert!(downloads_marker.is_dir(), "a pass deleted the archives of a version a live claim holds");

    // Control: with the claim gone, the same pass finishes the tombstone.
    fs::remove_dir_all(&claimed).expect("drop the stand-in claim");
    let (lines, _unfinished) = finish_removals(std::slice::from_ref(&fixture.root), &tools, &fixture.run());
    assert!(!downloads_marker.exists(), "the tombstone stayed with no claim over it: {lines:?}");
}

/// The claim name a removal of `version` by the process `maker` writes.
fn claimed_name_made_by(version: &str, maker: ProcessIdentity) -> String {
    return ClaimedVersion { version: version.to_string(), maker, attempt: 0 }.to_name();
}

/// What:
/// A flat download another install replaced after the plan was made is not deleted:
/// the removal moves it aside, compares its write time with the one judged, and puts it back.
/// Why:
/// The review showed a new archive published over a planned flat download being deleted,
/// because that branch removed the file without checking that it was still the one the scan measured.
#[test]
fn a_flat_download_replaced_after_the_plan_is_not_deleted() {
    let fixture = Fixture::new();
    let downloads: PathBuf = fixture.root.join("downloads");
    fs::create_dir_all(&downloads).expect("make the downloads");
    let archive: PathBuf = downloads.join("node-26.11.0-node-v26.11.0-linux-x64.tar.gz");
    fs::write(&archive, b"old archive").expect("write the old archive");
    let judged: SystemTime = SystemTime::now() - Duration::from_secs(2 * 24 * 60 * 60);
    let naming: crate::toolchain::store_versions::EntryNaming =
        crate::toolchain::store_versions::EntryNaming::of(&fs::symlink_metadata(&archive).expect("read the old archive"));
    fs::File::options()
        .write(true)
        .open(&archive)
        .expect("open the old archive")
        .set_times(FileTimes::new().set_modified(judged))
        .expect("give it the age the scan would have measured");
    let candidate = TrimCandidate {
        root: fixture.root.clone(),
        tool: "node".to_string(),
        version: String::new(),
        directory: archive.clone(),
        downloads: None,
        bytes: 11,
        written: judged,
        naming,
        leftover: true,
    };
    let plan: TrimPlan = TrimPlan { total_bytes: 11, over_bound: true, steps: vec![TrimStep::Remove(candidate)] };
    // Another install publishes its archive at that path after the plan was made.
    let replacement: PathBuf = downloads.join("replacement.download");
    fs::write(&replacement, b"new archive waiting for unpack").expect("write the new download");
    fs::rename(&replacement, &archive).expect("publish the new archive over the old name");

    let outcome = apply_trim(&plan, &fixture.run());

    assert_eq!(outcome.removed, 0, "the cleanup deleted a file it did not judge: {:?}", outcome.lines);
    assert_eq!(fs::read(&archive).expect("read the replacement"), b"new archive waiting for unpack");
}

/// What:
/// A pass that could not read the locks keeps the recovery marker, because it could look at nothing.
/// Why:
/// The review showed the worktree list failing, then no tool directory named, no claim scan and no tombstone scan,
/// and the marker deleted: a later sync with nothing to install returns early and the claimed version stays hidden.
#[test]
fn an_unreadable_lock_keeps_the_recovery_marker() {
    let rig = crate::toolchain::fakes::Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let version: PathBuf = root.join("bin").join("0.0.1");
    fs::create_dir_all(&version).expect("make the version");
    fs::write(version.join("payload"), b"user program").expect("write the payload");
    let maker: ProcessIdentity = ProcessIdentity { pid: A_PROCESS_NUMBER_NO_MACHINE_HAS, started: 12_345 };
    let claimed: PathBuf = version.with_file_name(claimed_name_made_by("0.0.1", maker));
    fs::rename(&version, &claimed).expect("a kill landsed after the claim");
    fs::write(unfinished_marker(&root), b"").expect("leave the recovery marker");
    rig.runner.answer("git -C", "", 128);

    trim_after_install(&rig.context(), &[]);

    assert!(has_unfinished_removal(&root), "a pass that could read nothing deleted the only recovery trigger");
    assert!(claimed.join("payload").is_file(), "the claimed version must stay for a later pass to give back");
}

/// What:
/// A removal that cannot write its recovery marker removes nothing and says why.
/// Why:
/// The marker is what makes a later sync look at the root, so destructive work must not start without it;
/// the review showed the version deleted although the marker could not be written.
#[test]
fn a_marker_that_cannot_be_written_stops_the_removal() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = Fixture::new();
    let version: PathBuf = fixture.root.join("node/26.13.1");
    let written: SystemTime = fs::symlink_metadata(&version).expect("read the version").modified().expect("write time");
    let candidate = TrimCandidate {
        root: fixture.root.clone(),
        tool: "node".to_string(),
        version: "26.13.1".to_string(),
        directory: version.clone(),
        downloads: None,
        bytes: 1_000,
        written,
        naming: crate::toolchain::store_versions::EntryNaming::default(),
        leftover: false,
    };
    let plan: TrimPlan = TrimPlan { total_bytes: 1_000, over_bound: true, steps: vec![TrimStep::Remove(candidate)] };
    fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o555)).expect("make the marker unwritable");

    let outcome = apply_trim(&plan, &fixture.run());

    fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o755)).expect("give the fixture its access back");
    assert_eq!(outcome.removed, 0, "the removal ran without a recovery marker: {:?}", outcome.lines);
    assert!(version.join("program").is_file(), "the version went although its recovery marker could not be written");
    assert!(
        outcome.lines.iter().any(|line| return line.contains("recovery marker") && line.contains("cannot be written")),
        "the line says why nothing was removed: {:?}",
        outcome.lines
    );
}

//endregion
