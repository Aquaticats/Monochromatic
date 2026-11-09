//! Several worktrees converted side by side:
//! they overlap in time,
//! the report keeps input order,
//! a failure in one leaves the
//!  others whole,
//! the budget and `--jobs` bound the overlap,
//! and the output equals the one-at-a-time output.

use super::*;
use crate::fs_primitive_error::PrimitiveError;
use crate::parallel_test_support::{PATIENCE, Rendezvous, hold_the_only_place};
use crate::subvolume_test_support::{PlainPlatform, backdate, make_checkout};
use crate::test_support::{fixture, git, remove};
use crate::worktree_convert::CONVERTING_SUFFIX;
use crate::worktree_convert_journal::JOURNAL_NAME;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// How many worktrees the controls convert.
const WORKTREES: usize = 4;

/// A host large enough that neither the CPUs nor the memory bound a run of `WORKTREES` worktrees.
const LARGE_HOST: HostResources = HostResources { parallelism: WORKTREES * 8, memory_available: None };

/// How long the worktree that must finish last waits for the others before the control reports that they did not.
const LAST_TO_FINISH_PATIENCE: Duration = Duration::from_secs(3);

/// The name of the worktree at `index`;
/// the names sort in index order.
fn name_of(index: usize) -> String {
    return format!("w{index}");
}

/// A platform on plain directories that calls hooks before it creates a subvolume and before it exchanges two directories.
struct Watched {
    /// The platform that does the work.
    /// It is shared because a test hands the same one to a second run,
    /// and because a conversion is a job of the blocking pool,
    /// which must be `'static`.
    inner: Arc<PlainPlatform>,
    /// Called with the path of the converting subvolume before it is created;
    /// an error fails the creation.
    /// The hooks own what they read, because a conversion is a job of the blocking pool,
    /// which must be `'static`.
    on_create: Box<dyn Fn(&Path) -> Result<(), PrimitiveError> + Send + Sync>,
    /// Called before the swap of a worktree with its converting sibling.
    on_exchange: Box<dyn Fn(&Path) + Send + Sync>,
}

impl Watched {
    /// A platform with hooks that do nothing.
    fn quiet() -> Watched {
        return Watched { inner: Arc::new(PlainPlatform::new()), on_create: Box::new(|_path: &Path| { return Ok(()); }), on_exchange: Box::new(|_path: &Path| {}) };
    }
}

/// Only creation and the swap are watched.
impl Platform for Watched {
    fn filesystem_facts(&self, path: &Path) -> Result<(crate::fs_kind::FilesystemKind, u64), PrimitiveError> {
        return self.inner.filesystem_facts(path);
    }
    fn is_subvolume_root(&self, path: &Path) -> Result<bool, PrimitiveError> {
        return self.inner.is_subvolume_root(path);
    }
    fn create_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        (self.on_create)(path)?;
        return self.inner.create_subvolume(path);
    }
    fn destroy_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.destroy_subvolume(path);
    }
    fn removal_allowed(&self, path: &Path) -> bool {
        return self.inner.removal_allowed(path);
    }
    fn is_mount_point(&self, path: &Path) -> bool {
        return self.inner.is_mount_point(path);
    }
    fn clone_file(&self, source: &std::fs::File, destination: &std::fs::File, path: &Path) -> Result<(), PrimitiveError> {
        return self.inner.clone_file(source, destination, path);
    }
    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError> {
        (self.on_exchange)(first);
        return self.inner.exchange(first, second);
    }
    fn conversion_environment(&self) -> crate::worktree_clean_content::ConversionEnvironment {
        return self.inner.conversion_environment();
    }
}

/// A checkout with `count` quiet linked worktrees `w0`,
/// `w1`,
/// ... inside it.
fn checkout_with_worktrees(name: &str, count: usize) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let mut exclude: String = std::fs::read_to_string(root.join(".git/info/exclude")).unwrap_or_default();
    for index in 0..count {
        git(root.as_path(), &["worktree", "add", "--quiet", "--detach", name_of(index).as_str()]);
        exclude.push_str(format!("/{}/\n", name_of(index)).as_str());
    }
    std::fs::create_dir_all(root.join(".git/info")).expect("info directory");
    std::fs::write(root.join(".git/info/exclude"), exclude).expect("exclude");
    for index in 0..count {
        backdate(root.join(name_of(index)).as_path());
        git(root.join(name_of(index)).as_path(), &["update-index", "--refresh"]);
    }
    return (parent, root);
}

/// Run `--all` (or the named worktrees) on `host` and return the status and every line.
fn run_on(platform: &Arc<dyn Platform + Send + Sync>, host: &HostResources, arguments: &ConvertArguments, cwd: &Path) -> (i32, Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let code: i32 = execute_run(&ConvertRun { platform: Arc::clone(platform), arguments, cwd, interrupt: Arc::new(Interrupt::never()), host }, &mut |text: &str| { lines.push(String::from(text)); });
    return (code, lines);
}

/// The platform of a control, as a run takes it.
fn shared(platform: &Arc<Watched>) -> Arc<dyn Platform + Send + Sync> {
    let held: Arc<Watched> = Arc::clone(platform);
    let shared: Arc<dyn Platform + Send + Sync> = held;
    return shared;
}

/// The platform of a control that hands the run a quiet one of its own.
fn shared_quiet() -> Arc<dyn Platform + Send + Sync> {
    let quiet: Arc<Watched> = Arc::new(Watched::quiet());
    return shared(&quiet);
}

/// The arguments of a real `--all` run.
fn all_arguments() -> ConvertArguments {
    return ConvertArguments { all: true, ..ConvertArguments::default() };
}

/// Whether `path` is the converting sibling of the worktree called `name`.
fn is_converting_of(path: &Path, name: &str) -> bool {
    return path.file_name().is_some_and(|last: &std::ffi::OsStr| { return last.to_string_lossy() == format!("{name}{CONVERTING_SUFFIX}"); });
}

/// The conversions of several worktrees start before any of them has finished the first step:
/// a point that needs all of them
///  at once is passed only by parallel work,
/// and a serial run waits there until its patience ends.
#[test]
fn worktrees_are_converted_side_by_side() {
    let (parent, root) = checkout_with_worktrees("convert-parallel-overlap", WORKTREES);
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::new(WORKTREES));
    let met: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let (at_the_point, met_it) = (Arc::clone(&point), Arc::clone(&met));
    let platform: Arc<Watched> = Arc::new(Watched {
        on_create: Box::new(move |_path: &Path| {
            if at_the_point.arrive() {
                met_it.fetch_add(1, Ordering::SeqCst);
            }
            return Ok(());
        }),
        ..Watched::quiet()
    });
    let (code, lines) = run_on(&shared(&platform), &LARGE_HOST, &all_arguments(), root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    assert_eq!(met.load(Ordering::SeqCst), WORKTREES, "every conversion met the others while it ran: {lines:#?}");
    remove(parent.as_path());
}

/// The report is the one a run of one worktree after another gives,
/// whichever worktree finishes first:
/// the worktree that sorts
///  first is held back until all the others have finished,
/// and its lines still come first.
#[test]
fn the_report_keeps_input_order_whichever_worktree_finishes_first() {
    let (parent, root) = checkout_with_worktrees("convert-parallel-order", WORKTREES);
    let others_done: Arc<Rendezvous> = Arc::new(Rendezvous::impatient(WORKTREES - 1, LAST_TO_FINISH_PATIENCE));
    let first_of_all: String = name_of(0);
    let (waited_on, arrived, first_for_the_swap) = (Arc::clone(&others_done), Arc::clone(&others_done), first_of_all.clone());
    let platform: Arc<Watched> = Arc::new(Watched {
        on_create: Box::new(move |path: &Path| {
            if is_converting_of(path, first_of_all.as_str()) {
                assert!(waited_on.wait_for_target(), "the other worktrees were converted while the first waited");
            }
            return Ok(());
        }),
        on_exchange: Box::new(move |path: &Path| {
            if !path.ends_with(first_for_the_swap.as_str()) {
                arrived.arrive_without_waiting();
            }
        }),
        ..Watched::quiet()
    });
    let (code, lines) = run_on(&shared(&platform), &LARGE_HOST, &all_arguments(), root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    assert_eq!(lines.first().map(String::as_str), Some(RISK_NOTICE));
    let starts: Vec<usize> = (0..WORKTREES)
        .map(|index: usize| { return lines.iter().position(|line: &String| { return line.ends_with(format!("/{}", name_of(index)).as_str()); }).expect("the worktree is named"); })
        .collect();
    assert!(starts.windows(2).all(|pair: &[usize]| { return pair[0] < pair[1]; }), "named in input order: {lines:#?}");
    for (index, start) in starts.iter().enumerate() {
        let end: usize = starts.get(index + 1).copied().unwrap_or(lines.len() - 1);
        let own: &[String] = &lines[start + 1..end];
        assert_eq!(own.iter().filter(|line: &&String| { return line.as_str() == "  converted into a subvolume"; }).count(), 1, "{own:#?}");
        assert!(own.iter().all(|line: &String| { return line.starts_with("  "); }), "only this worktree's lines between its name and the next: {own:#?}");
    }
    assert!(lines.last().is_some_and(|line: &String| { return line.starts_with(format!("{WORKTREES} worktree(s) looked at").as_str()); }), "{lines:#?}");
    remove(parent.as_path());
}

/// A creation that fails in the first worktree leaves its journal and no other trace,
/// converts every other worktree,
/// and ends the
///  run with the failure status;
/// the same command again finishes the first.
#[test]
fn a_failed_worktree_does_not_stop_or_corrupt_the_others() {
    let (parent, root) = checkout_with_worktrees("convert-parallel-failure", WORKTREES);
    let first_of_all: String = name_of(0);
    let platform: Arc<Watched> = Arc::new(Watched {
        on_create: Box::new(move |path: &Path| {
            if is_converting_of(path, first_of_all.as_str()) {
                return Err(PrimitiveError { operation: "BTRFS_IOC_SUBVOL_CREATE", path: path.to_path_buf(), errno: libc::EIO });
            }
            return Ok(());
        }),
        ..Watched::quiet()
    });
    let (code, lines) = run_on(&shared(&platform), &LARGE_HOST, &all_arguments(), root.as_path());
    assert_eq!(code, FAILED_EXIT_CODE, "{lines:#?}");
    assert_eq!(lines.iter().filter(|line: &&String| { return line.starts_with("  failed:"); }).count(), 1, "{lines:#?}");
    assert_eq!(lines.iter().filter(|line: &&String| { return line.as_str() == "  converted into a subvolume"; }).count(), WORKTREES - 1, "{lines:#?}");
    assert!(lines.last().is_some_and(|line: &String| { return line.contains("1 failed"); }), "the summary counts the failure: {lines:#?}");
    assert!(!platform.is_subvolume_root(root.join(name_of(0)).as_path()).expect("examined"), "the failed worktree is untouched");
    assert!(root.join(".git/worktrees").join(name_of(0)).join(JOURNAL_NAME).exists(), "and journaled, so the command can finish it");
    for index in 1..WORKTREES {
        assert!(platform.is_subvolume_root(root.join(name_of(index)).as_path()).expect("examined"), "{} was converted", name_of(index));
    }
    let healthy: Arc<Watched> = Arc::new(Watched { inner: Arc::clone(&platform.inner), ..Watched::quiet() });
    let (again_code, again_lines) = run_on(&shared(&healthy), &LARGE_HOST, &all_arguments(), root.as_path());
    assert_eq!(again_code, 0, "{again_lines:#?}");
    assert!(healthy.is_subvolume_root(root.join(name_of(0)).as_path()).expect("examined"), "{again_lines:#?}");
    remove(parent.as_path());
}

/// The most conversions in flight at once,
/// from the first step to the swap,
/// for a run on `host` with `jobs` as the person's cap,
///  with a point that the first `meeting` of them must reach together.
fn busiest_of(name: &str, host: &HostResources, jobs: Option<usize>, meeting: usize) -> usize {
    let (parent, root) = checkout_with_worktrees(name, WORKTREES + WORKTREES);
    let running: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let busiest: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::impatient(meeting, LAST_TO_FINISH_PATIENCE));
    let (live, most, at_the_point, leaving) = (Arc::clone(&running), Arc::clone(&busiest), Arc::clone(&point), Arc::clone(&running));
    let platform: Arc<Watched> = Arc::new(Watched {
        on_create: Box::new(move |_path: &Path| {
            let now: usize = live.fetch_add(1, Ordering::SeqCst) + 1;
            most.fetch_max(now, Ordering::SeqCst);
            if meeting > 1 {
                at_the_point.arrive();
            }
            return Ok(());
        }),
        on_exchange: Box::new(move |_path: &Path| { leaving.fetch_sub(1, Ordering::SeqCst); }),
        ..Watched::quiet()
    });
    let (code, lines) = run_on(&shared(&platform), host, &ConvertArguments { jobs, ..all_arguments() }, root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    remove(parent.as_path());
    return busiest.load(Ordering::SeqCst);
}

/// The budget the host allows is the bound:
/// on a host that may run few threads the overlap reaches that many and no more.
#[test]
fn the_budget_of_the_host_bounds_the_overlap() {
    let host: HostResources = HostResources { parallelism: 3, memory_available: None };
    let allowed: usize = conversion_workers(usize::MAX, None, &host);
    assert!(allowed > 1 && allowed < WORKTREES, "the control needs a bound between one and the worktrees: {allowed}");
    assert_eq!(busiest_of("convert-parallel-budget", &host, None, allowed), allowed);
}

/// Memory that is free bounds the overlap too:
/// a host with room for one conversion converts one at a time.
#[test]
fn the_free_memory_of_the_host_bounds_the_overlap() {
    let host: HostResources = HostResources { parallelism: WORKTREES * 8, memory_available: Some(CONVERSION_MEMORY) };
    assert_eq!(busiest_of("convert-parallel-memory", &host, None, 1), 1);
}

/// The person's cap bounds the overlap on a host that allows more.
#[test]
fn jobs_caps_the_overlap() {
    let cap: usize = 2;
    assert_eq!(busiest_of("convert-parallel-jobs", &LARGE_HOST, Some(cap), cap), cap);
}

/// The same worktree named twice is converted once and then found converted,
/// never refused as busy with itself.
#[test]
fn a_worktree_named_twice_is_converted_once_and_then_skipped() {
    let (parent, root) = checkout_with_worktrees("convert-parallel-twice", 2);
    let platform: Arc<Watched> = Arc::new(Watched::quiet());
    let named: ConvertArguments = ConvertArguments { paths: vec![name_of(0).into(), name_of(0).into(), name_of(1).into()], ..ConvertArguments::default() };
    let (code, lines) = run_on(&shared(&platform), &LARGE_HOST, &named, root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    assert_eq!(lines.iter().filter(|line: &&String| { return line.as_str() == "  converted into a subvolume"; }).count(), 2, "{lines:#?}");
    assert_eq!(lines.iter().filter(|line: &&String| { return line.as_str() == "  already a subvolume, skipped"; }).count(), 1, "{lines:#?}");
    assert!(!lines.iter().any(|line: &String| { return line.contains("another conversion"); }), "{lines:#?}");
    remove(parent.as_path());
}

/// A dry run lists every worktree in order,
/// says what it would do,
/// and changes nothing.
#[test]
fn a_dry_run_lists_every_worktree_in_order_and_changes_nothing() {
    let (parent, root) = checkout_with_worktrees("convert-parallel-dry", WORKTREES);
    let platform: Arc<Watched> = Arc::new(Watched::quiet());
    let (code, lines) = run_on(&shared(&platform), &LARGE_HOST, &ConvertArguments { dry_run: true, ..all_arguments() }, root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    let expected: Vec<String> = (0..WORKTREES)
        .flat_map(|index: usize| { return [format!("{}/{}", root.display(), name_of(index)), String::from("  would convert")]; })
        .collect();
    assert_eq!(lines[..lines.len() - 1], expected, "{lines:#?}");
    for index in 0..WORKTREES {
        assert!(!platform.is_subvolume_root(root.join(name_of(index)).as_path()).expect("examined"));
    }
    remove(parent.as_path());
}

/// The lines of a run with the fixture's own path replaced,
/// so two fixtures' reports can be compared.
fn lines_without_root(lines: &[String], root: &Path) -> Vec<String> {
    return lines.iter().map(|line: &String| { return line.replace(root.to_string_lossy().as_ref(), "<root>"); }).collect();
}

/// What a person reads is the same whether the worktrees were converted one at a time or side by side.
#[test]
fn the_output_equals_the_one_at_a_time_output() {
    let (serial_parent, serial_root) = checkout_with_worktrees("convert-parallel-equal-serial", WORKTREES);
    let (side_parent, side_root) = checkout_with_worktrees("convert-parallel-equal-side", WORKTREES);
    let (serial_code, serial_lines) = run_on(&shared_quiet(), &LARGE_HOST, &ConvertArguments { jobs: Some(1), ..all_arguments() }, serial_root.as_path());
    let (side_code, side_lines) = run_on(&shared_quiet(), &LARGE_HOST, &all_arguments(), side_root.as_path());
    assert_eq!(serial_code, side_code);
    assert_eq!(lines_without_root(&serial_lines, serial_root.as_path()), lines_without_root(&side_lines, side_root.as_path()));
    remove(serial_parent.as_path());
    remove(side_parent.as_path());
}

/// However large the host and however many worktrees,
/// no more than the measured ceiling are converted at once;
/// the person's cap
///  still lowers it.
#[test]
fn a_large_host_does_not_pass_the_storage_ceiling() {
    let huge: HostResources = HostResources { parallelism: CONVERSION_STORAGE_CEILING * 8, memory_available: None };
    assert_eq!(conversion_workers(CONVERSION_STORAGE_CEILING * 4, None, &huge), CONVERSION_STORAGE_CEILING);
    assert_eq!(conversion_workers(CONVERSION_STORAGE_CEILING * 4, Some(2), &huge), 2);
    assert_eq!(conversion_workers(3, None, &huge), 3);
}

/// The files the fixture checkout commits,
/// which every conversion copies into its worktree.
const TRACKED_FILES_IN_THE_FIXTURE_CHECKOUT: usize = 7;

/// The changes the conversion copies into a worktree of this fixture:
/// its files and the directories that hold them,
/// measured on this fixture with `PlainPlatform`,
/// which clones no data.
const COPIED_ENTRIES: usize = 11;

/// The entries of a worktree's index that the conversion's rewrite refreshes,
/// measured on this fixture.
const INDEX_ENTRIES_REFRESHED: usize = TRACKED_FILES_IN_THE_FIXTURE_CHECKOUT - 1;

/// The report of a run over `WORKTREES` worktrees,
/// recorded from the code before this step,
/// with the fixture's own path replaced.
/// A legitimate change to the report breaks this control on purpose.
fn the_recorded_report() -> Vec<String> {
    let mut report: Vec<String> = vec![String::from(RISK_NOTICE)];
    for index in 0..WORKTREES {
        report.push(format!("<root>/{}", name_of(index)));
        report.push(format!("  copied {COPIED_ENTRIES} entries ({TRACKED_FILES_IN_THE_FIXTURE_CHECKOUT} files, 0 cloned)"));
        report.push(format!("  index: {INDEX_ENTRIES_REFRESHED} of {TRACKED_FILES_IN_THE_FIXTURE_CHECKOUT} entries refreshed"));
        report.push(String::from("  converted into a subvolume"));
    }
    report.push(format!("{WORKTREES} worktree(s) looked at, {WORKTREES} converted or completed, 0 skipped."));
    return report;
}

/// What a person reads is the report the code gave before this step,
/// line for line and in the order of the worktrees named.
#[test]
fn the_lines_of_a_run_are_the_ones_recorded_before_the_step() {
    let (parent, root) = checkout_with_worktrees("convert-parallel-lines", WORKTREES);
    let (code, lines) = run_on(&shared_quiet(), &LARGE_HOST, &all_arguments(), root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    assert_eq!(lines_without_root(&lines, root.as_path()), the_recorded_report(), "the report of a run is no longer the one recorded before this step");
    remove(parent.as_path());
}

/// What:
///  Every conversion runs inside the process's runtime,
/// as a job of its blocking pool,
/// and not on a thread of its own.
/// Why:
///   This is the whole of the conversion:
/// the conversions used to run on scoped threads,
/// where the process's runtime is not entered at all,
///  and the runtime is what the owner asked every wait to sit on.
#[test]
fn the_conversions_run_on_the_processs_runtime() {
    let (parent, root) = checkout_with_worktrees("convert-parallel-runtime", WORKTREES);
    let probe: Arc<std::sync::Mutex<Vec<bool>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let inside = Arc::clone(&probe);
    let platform: Arc<Watched> = Arc::new(Watched {
        on_create: Box::new(move |_path: &Path| {
            inside.lock().expect("lock").push(tokio::runtime::Handle::try_current().is_ok());
            return Ok(());
        }),
        ..Watched::quiet()
    });
    let (code, lines) = run_on(&shared(&platform), &LARGE_HOST, &all_arguments(), root.as_path());
    let seen: Vec<bool> = probe.lock().expect("lock").clone();
    assert_eq!(code, 0, "{lines:#?}");
    assert_eq!(seen, vec![true; WORKTREES], "a conversion ran outside the process's runtime");
    remove(parent.as_path());
}

/// A conversion waits for the one place the process's blocking pool has:
/// only a job of the pool takes one,
/// and a conversion run on a worker of the runtime instead would start at once.
#[test]
fn the_conversions_take_places_of_the_processs_bound_on_blocking_jobs() {
    let (parent, root) = checkout_with_worktrees("convert-parallel-a-place-of-the-bound", WORKTREES);
    let (given_back, finished) = hold_the_only_place();
    let started_after: Arc<std::sync::Mutex<Vec<bool>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let (began, given) = (Arc::clone(&started_after), Arc::clone(&given_back));
    let platform: Arc<Watched> = Arc::new(Watched {
        on_create: Box::new(move |_path: &Path| {
            began.lock().expect("lock").push(given.load(Ordering::SeqCst));
            return Ok(());
        }),
        ..Watched::quiet()
    });
    let (code, lines) = run_on(&shared(&platform), &LARGE_HOST, &all_arguments(), root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    let seen: Vec<bool> = started_after.lock().expect("lock").clone();
    assert_eq!(seen, vec![true; WORKTREES], "a conversion started while another job held the only place of the process's bound, so it did not take one: {lines:#?}");
    assert!(finished.recv_timeout(PATIENCE).is_ok(), "the job that held the place never gave it back");
    remove(parent.as_path());
}
