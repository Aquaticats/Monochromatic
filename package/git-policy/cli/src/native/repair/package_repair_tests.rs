//! The switches read from a command line,
//! and the orchestration with fixture tools:
//! nothing starts for `--no-checkout`,
//!  the tools run side by side,
//! a failure is reported and never raised,
//! and `--quiet` silences only success.

use super::*;
use crate::test_support::executable;
use crate::package_repair_report::{Outcome, Tool};
use crate::test_support::{SharedSink, fixture, git_context, remove};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

/// The arguments of a command line as the wrapper receives them.
fn words(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(OsString::from).collect();
}

/// `--quiet` and `-q` after `worktree add` are quiet;
/// so is a bundle that holds `q`.
#[test]
fn reads_quiet_in_each_spelling() {
    for spelling in ["--quiet", "-q", "-dq"] {
        let found: RepairSwitch = RepairSwitch::from_arguments(&words(&["worktree", "add", spelling, "../topic"]));
        assert!(found.quiet, "{spelling}");
        assert!(!found.no_checkout, "{spelling}");
    }
}

/// `--no-checkout` is read,
/// and global options before the subcommand do not hide it.
#[test]
fn reads_no_checkout_behind_global_options() {
    let found: RepairSwitch = RepairSwitch::from_arguments(&words(&["-C", "repo", "-c", "a.b=c", "worktree", "add", "--no-checkout", "../topic"]));
    assert!(found.no_checkout);
    assert!(!found.quiet);
}

/// Neither switch is read from a value,
/// an operand after `--`,
/// or another command.
#[test]
fn ignores_look_alikes() {
    assert_eq!(RepairSwitch::from_arguments(&words(&["worktree", "add", "-b", "--quiet", "../topic"])), RepairSwitch::default());
    assert_eq!(RepairSwitch::from_arguments(&words(&["worktree", "add", "--", "--no-checkout"])), RepairSwitch::default());
    assert_eq!(RepairSwitch::from_arguments(&words(&["status", "--quiet", "--no-checkout"])), RepairSwitch::default());
    assert_eq!(RepairSwitch::from_arguments(&[]), RepairSwitch::default());
}

/// A destination holding a lockfile and a `node_modules`,
/// a source beside it,
/// and a search path of one fixture directory.
struct Scene {
    /// The fixture root,
    /// removed at the end.
    root: PathBuf,
    /// The source checkout.
    source: PathBuf,
    /// The new worktree.
    destination: PathBuf,
    /// The directory of fixture tools.
    bin: PathBuf,
    /// Where fixture tools append one line per start.
    record: PathBuf,
    /// The repair environment,
    /// searching only the fixture directory.
    environment: RepairEnvironment,
}

/// Build a scene whose destination has pnpm state and no Cargo manifest.
fn scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let source: PathBuf = root.join("source");
    let destination: PathBuf = root.join("destination");
    let bin: PathBuf = root.join("bin");
    for directory in [&source, &destination.join("node_modules"), &bin] {
        std::fs::create_dir_all(directory).expect("directory");
    }
    std::fs::write(destination.join("pnpm-lock.yaml"), b"lockfileVersion: '9.0'\n").expect("lockfile");
    let record: PathBuf = root.join("record");
    let environment: RepairEnvironment = RepairEnvironment { search_path: Some(bin.clone().into_os_string()), quiet_window: Duration::from_secs(60), ceiling: Duration::from_secs(600), workers: 2 };
    return Scene { root, source, destination, bin, record, environment };
}

/// A fixture `pnpm` that records its start and exits with `status`.
fn fake_pnpm(scene: &Scene, status: i32) {
    executable(
        scene.bin.join("pnpm").as_path(),
        format!("#!/bin/sh\necho \"pnpm $* in $(pwd -P)\" >> '{}'\nexit {status}\n", scene.record.display()).as_bytes(),
    );
}

/// Run a repair of the scene with the switches and collect what it printed.
fn repair(scene: &Scene, switch: RepairSwitch) -> (Vec<Finding>, String) {
    let context = git_context();
    let request: RepairRequest<'_> = RepairRequest { git: &context, source: scene.source.as_path(), destination: scene.destination.as_path(), switch, environment: &scene.environment };
    let mut sink: SharedSink = SharedSink::default();
    let findings: Vec<Finding> = repair_worktree(&request, &mut sink);
    return (findings, sink.text());
}

/// `--no-checkout` starts nothing,
/// even though the lockfile and `node_modules` are there.
#[test]
fn no_checkout_starts_nothing() {
    let scene: Scene = scene("repair-no-checkout");
    fake_pnpm(&scene, 0);
    let (findings, printed) = repair(&scene, RepairSwitch { quiet: false, no_checkout: true });
    assert!(findings.is_empty());
    assert_eq!(printed, "");
    assert!(!scene.record.exists(), "a tool was started for a worktree with no files");
    remove(&scene.root);
}

/// A repair that works is one success finding and one printed line;
/// `--quiet` prints nothing.
#[test]
fn a_working_repair_reports_one_line_unless_quiet() {
    let scene: Scene = scene("repair-works");
    fake_pnpm(&scene, 0);
    let (findings, printed) = repair(&scene, RepairSwitch::default());
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].tool, Tool::Pnpm);
    assert!(matches!(findings[0].outcome, Outcome::Repaired { .. }));
    assert_eq!(printed.lines().count(), 1, "{printed}");
    let (_, quiet_printed) = repair(&scene, RepairSwitch { quiet: true, no_checkout: false });
    assert_eq!(quiet_printed, "");
    remove(&scene.root);
}

/// A failing tool is a failure finding and a printed line naming the command,
/// even under `--quiet`;
/// nothing is raised.
#[test]
fn a_failing_repair_names_its_command_even_when_quiet() {
    let scene: Scene = scene("repair-fails");
    fake_pnpm(&scene, 1);
    let (findings, printed) = repair(&scene, RepairSwitch { quiet: true, no_checkout: false });
    assert!(matches!(findings[0].outcome, Outcome::Failed { .. }), "{findings:?}");
    assert!(printed.contains("pnpm") && printed.contains(scene.destination.to_str().expect("utf8")), "{printed}");
    remove(&scene.root);
}

/// A tool the environment does not provide is a failure finding that names the command,
/// not an error.
#[test]
fn a_missing_tool_names_the_command_to_run() {
    let scene: Scene = scene("repair-missing");
    let (findings, printed) = repair(&scene, RepairSwitch::default());
    let Outcome::Failed { reason, run_instead } = &findings[0].outcome else { panic!("expected a failure: {findings:?}") };
    assert!(reason.contains("pnpm"), "{reason}");
    assert!(printed.contains(run_instead.as_str()), "{printed}");
    remove(&scene.root);
}

/// A destination with no pnpm state starts no tool and prints nothing.
#[test]
fn nothing_to_repair_prints_nothing() {
    let scene: Scene = scene("repair-nothing");
    fake_pnpm(&scene, 0);
    std::fs::remove_file(scene.destination.join("pnpm-lock.yaml")).expect("remove lockfile");
    let (findings, printed) = repair(&scene, RepairSwitch::default());
    assert!(findings.is_empty());
    assert_eq!(printed, "");
    assert!(!scene.record.exists());
    remove(&scene.root);
}

/// The repairs may use as many threads as the host offers,
/// not a fixed few:
/// the work is independent per workspace and per tool.
#[test]
fn the_workers_are_what_the_host_offers() {
    let large_host: usize = 64;
    assert_eq!(workers_for(large_host), large_host);
    assert_eq!(workers_for(1), 1);
    assert_eq!(workers_for(0), 1, "a host that reports nothing still runs one worker");
}

/// Give the scene's destination a launcher record that names another checkout's lock,
/// which is what a copy leaves behind,
/// and a fixture mpm-shadow that rewrites it as an install does.
fn with_a_stale_launcher(scene: &Scene) {
    let launchers: PathBuf = scene.destination.join(".cache/toolchain-launchers");
    std::fs::create_dir_all(&launchers).expect("launchers");
    std::fs::write(launchers.join("mpm.launch"), b"{\"lockFile\":\"/elsewhere/toolchain.lock.json\"}\n").expect("record");
    let record: PathBuf = launchers.join("mpm.launch");
    let own: PathBuf = scene.destination.join("toolchain.lock.json");
    executable(
        scene.bin.join("mpm-shadow").as_path(),
        format!(
            "#!/bin/sh\ncase \"$1\" in\n  install) printf '{{\"lockFile\":\"%s\"}}\\n' '{own}' > '{record}' ;;\nesac\nexit 0\n",
            own = own.display(),
            record = record.display()
        )
        .as_bytes(),
    );
}

/// The pnpm and the toolchain repairs really run beside each other:
/// each tool names its start and then waits,
/// bounded,
/// for the other's name,
/// which only an overlap can bring about.
/// A repair that ran its tools one after another would let that wait run out and fail the tool.
#[test]
fn the_tools_run_beside_each_other() {
    /// How many times a fixture tool looks for the other's name before it gives up:
    /// three seconds at ten milliseconds a look.
    const LOOKS: usize = 300;
    let scene: Scene = scene("repair-overlap");
    with_a_stale_launcher(&scene);
    let pnpm_marked: PathBuf = scene.root.join("pnpm-started");
    let toolchain_marked: PathBuf = scene.root.join("toolchain-started");
    let waiting_tool = |mine: &Path, theirs: &Path| {
        return format!(
            "#!/bin/sh\ntouch '{mine}'\nlooked=0\nwhile [ ! -f '{theirs}' ]; do looked=$((looked + 1)); [ \"$looked\" -gt {LOOKS} ] && exit 1; sleep 0.01; done\nexit 0\n",
            mine = mine.display(),
            theirs = theirs.display()
        );
    };
    executable(scene.bin.join("pnpm").as_path(), waiting_tool(pnpm_marked.as_path(), toolchain_marked.as_path()).as_bytes());
    // The install names its start, waits for pnpm's, then rewrites the launcher record as a real install does.
    let record: PathBuf = scene.destination.join(".cache/toolchain-launchers/mpm.launch");
    let own_lock: PathBuf = scene.destination.join("toolchain.lock.json");
    let installing: String = format!(
        "#!/bin/sh\ntouch '{mine}'\nlooked=0\nwhile [ ! -f '{theirs}' ]; do looked=$((looked + 1)); [ \"$looked\" -gt {LOOKS} ] && exit 1; sleep 0.01; done\nprintf '{{\"lockFile\":\"%s\"}}\\n' '{own}' > '{record}'\nexit 0\n",
        mine = toolchain_marked.display(),
        theirs = pnpm_marked.display(),
        own = own_lock.display(),
        record = record.display()
    );
    executable(scene.bin.join("mpm-shadow").as_path(), installing.as_bytes());
    let (findings, printed) = repair(&scene, RepairSwitch::default());
    assert!(pnpm_marked.exists() && toolchain_marked.exists(), "both tools were not started: {printed}");
    assert_eq!(findings.len(), 2, "{findings:?} {printed}");
    for finding in &findings {
        assert!(matches!(finding.outcome, Outcome::Repaired { .. }), "{finding:?} {printed}");
    }
    remove(&scene.root);
}

/// Where the system gives no thread and no process,
/// the repair reports each tool in plain words,
/// changes nothing,
/// and does not panic.
/// The body runs in a child that is refused every thread.
#[test]
fn a_refused_system_is_reported_in_plain_words_and_changes_nothing() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_refused_system_is_reported_in_plain_words_and_changes_nothing") {
        return;
    }
    // The fixture tools are started as processes of their own, so the limit is lifted while the scene is made and put
    //  back for the part this control measures.
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises");
    let scene: Scene = scene("repair-refused");
    fake_pnpm(&scene, 0);
    let lockfile: PathBuf = scene.destination.join("pnpm-lock.yaml");
    let before: Vec<u8> = std::fs::read(lockfile.as_path()).expect("the lockfile");
    crate::no_threads_child_test_support::refuse_threads_again().expect("the process limit falls");
    let (findings, printed) = repair(&scene, RepairSwitch::default());
    let Outcome::Failed { reason, run_instead } = &findings[0].outcome else { panic!("expected a failure: {findings:?} {printed}") };
    assert_eq!(findings[0].tool, Tool::Pnpm, "{findings:?}");
    assert!(reason.contains("pnpm"), "{reason}");
    assert!(printed.contains(run_instead.as_str()), "{printed}");
    assert_eq!(std::fs::read(lockfile.as_path()).expect("the lockfile"), before, "the repair changed the worktree");
    remove(&scene.root);
}
