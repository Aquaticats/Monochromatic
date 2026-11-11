//! Controls of the service's relinking:
//! which tools run,
//! in which order,
//! and what a tool that fails leaves behind.

use super::relink_with;
use crate::package_repair_relink::{RelinkEntry, RelinkTool, item_of};
use crate::package_repair_request::RepairEnvironment;
use crate::service_work_handlers_replanning_and_acceptance::Verdict;
use crate::test_support::{executable, fixture, remove};
use crate::work_item::WorkItem;
use std::path::PathBuf;
use std::time::Duration;

/// A worktree whose copy needs both tools,
/// a directory of fixture tools,
/// and where the tools name their starts.
struct Scene {
    /// The fixture root,
    /// removed at the end.
    root: PathBuf,
    /// The new worktree.
    worktree: PathBuf,
    /// The repository's common directory,
    /// which the record names.
    common: PathBuf,
    /// The directory of fixture tools.
    bin: PathBuf,
    /// Where fixture tools append one line per start.
    record: PathBuf,
    /// The environment that searches only the fixture directory.
    environment: RepairEnvironment,
}

/// What a launcher record names when the worktree's own lock is another checkout's:
/// the stale state a copy leaves behind.
const ELSEWHERE: &str = "/elsewhere/toolchain.lock.json";

/// Build a scene whose worktree is a linked worktree of the repository the record names,
/// and holds a pnpm layout and a stale launcher record.
fn scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let worktree: PathBuf = root.join("worktree");
    let common: PathBuf = root.join("common");
    let bin: PathBuf = root.join("bin");
    for directory in [&worktree.join("node_modules"), &worktree.join(".cache/toolchain-launchers"), &common, &bin] {
        std::fs::create_dir_all(directory).expect("directory");
    }
    std::fs::write(worktree.join("pnpm-lock.yaml"), b"lockfileVersion: '9.0'\n").expect("lockfile");
    std::fs::write(worktree.join("toolchain.lock.json"), b"{}\n").expect("the worktree's own lock");
    std::fs::write(worktree.join(".cache/toolchain-launchers/mpm.launch"), format!("{{\"lockFile\":\"{ELSEWHERE}\"}}\n").as_bytes()).expect("launcher");
    // The registration `git worktree add` leaves: a `.git` file naming the worktree's own directory under
    // the repository's `worktrees`, which is what makes this directory the record's own worktree.
    let admin: PathBuf = common.join("worktrees/topic");
    std::fs::create_dir_all(admin.as_path()).expect("the worktree's administrative directory");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", admin.display())).expect("the .git file");
    let record: PathBuf = root.join("record");
    let environment: RepairEnvironment =
        RepairEnvironment { search_path: Some(bin.clone().into_os_string()), quiet_window: Duration::from_secs(60), ceiling: Duration::from_secs(600), workers: 1 };
    return Scene { root, worktree, common, bin, record, environment };
}

/// What:
///  A fixture tool that names its start,
///  the search path it was given,
///  and then ends with `status`.
/// Why:
///   The record of starts is what proves which tool ran where,
///   and the search path is what proves the tool sees the path the add resolved it on;
/// a tool that must not touch the worktree's state leaves it as the copy left it.
fn recording_tool(scene: &Scene, program: &str, status: u8) -> String {
    return format!(
        "#!/bin/sh\necho \"{program} $* in $(pwd -P)\" >> '{started}'\nprintf '%s\\n' \"$PATH\" >> '{paths}'\nexit {status}\n",
        started = scene.record.display(),
        paths = search_paths(scene).display()
    );
}

/// Where the fixture tools name the search path they ran with.
fn search_paths(scene: &Scene) -> PathBuf {
    return scene.root.join("paths");
}

/// The search paths the fixture tools named.
fn paths(scene: &Scene) -> String {
    return std::fs::read_to_string(search_paths(scene).as_path()).unwrap_or_default();
}

/// What:
///  A fixture mpm-shadow that names its start and then rewrites the launcher record as a real install does.
/// Why:
///   The toolchain repair reads the records again after the install,
/// so a control whose install left them stale would report a failed toolchain,
/// and the record of starts must still show which tool ran where.
fn installing_toolchain(scene: &Scene, status: u8) -> String {
    let launcher: PathBuf = scene.worktree.join(".cache/toolchain-launchers/mpm.launch");
    let own: PathBuf = scene.worktree.join("toolchain.lock.json");
    return format!(
        "#!/bin/sh\necho \"mpm-shadow $* in $(pwd -P)\" >> '{started}'\nprintf '{{\"lockFile\":\"%s\"}}\\n' '{own}' > '{launcher}'\nexit {status}\n",
        started = scene.record.display(),
        own = own.display(),
        launcher = launcher.display()
    );
}

/// The record one scene's relink is queued as,
/// naming both tools.
fn item_of_the_scene(scene: &Scene, tools: Vec<RelinkTool>) -> WorkItem {
    let entry: RelinkEntry =
        RelinkEntry { root: scene.worktree.clone(), search_path: scene.environment.search_path.clone(), search_from: None, tools };
    return item_of(scene.common.as_path(), scene.worktree.as_path(), &entry).expect("the record");
}

/// Run the relinking as the service's round does.
fn relink(scene: &Scene, item: &WorkItem) -> Verdict {
    return crate::lazy_runtime::run_on_runtime(relink_with(item, &scene.environment)).expect("the relinking runs");
}

/// The starts the fixture tools named.
fn starts(scene: &Scene) -> String {
    return std::fs::read_to_string(scene.record.as_path()).unwrap_or_default();
}

/// What:
///  Both tools run in the worktree,
/// in the order the record names them,
/// and the relink is done.
/// Why:
///   Moving the work to the service must change nothing about what the tools do:
/// pnpm's install and mpm-shadow's install still run in the new worktree,
/// with the switches each repair owns.
#[test]
fn both_tools_run_in_the_worktree_in_the_records_order() {
    let scene: Scene = scene("relink-job-both");
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 0).as_bytes());
    executable(scene.bin.join("mpm-shadow").as_path(), installing_toolchain(&scene, 0).as_bytes());
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm, RelinkTool::Toolchain]);
    assert_eq!(relink(&scene, &item), Verdict::Done);
    let named: String = starts(&scene);
    let lines: Vec<&str> = named.lines().collect();
    assert_eq!(lines.len(), 2, "{named}");
    assert!(lines[0].starts_with("pnpm install --offline --frozen-lockfile in "), "{named}");
    assert!(lines[1].starts_with("mpm-shadow install --frozen-lockfile in "), "{named}");
    for line in lines {
        assert!(line.ends_with(scene.worktree.to_str().expect("utf8")), "{named}");
    }
    let launcher: String = std::fs::read_to_string(scene.worktree.join(".cache/toolchain-launchers/mpm.launch")).expect("the launcher record");
    assert!(!launcher.contains(ELSEWHERE), "the install left the record stale: {launcher}");
    remove(scene.root.as_path());
}

/// What:
///  A tool that fails is named in the verdict,
/// and the next tool still runs.
/// Why:
///   One tool's failure must not leave the other tool's state unrelinked,
/// and the person is told which tool failed:
/// the same two guarantees the add gave.
#[test]
fn a_tool_that_fails_is_named_and_the_next_still_runs() {
    let scene: Scene = scene("relink-job-fails");
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 3).as_bytes());
    executable(scene.bin.join("mpm-shadow").as_path(), installing_toolchain(&scene, 0).as_bytes());
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm, RelinkTool::Toolchain]);
    let verdict: Verdict = relink(&scene, &item);
    let Verdict::Failed(reason) = verdict else { panic!("a failing tool is a failure: {verdict:?}") };
    assert!(reason.starts_with("pnpm: "), "{reason}");
    let named: String = starts(&scene);
    assert!(named.contains("mpm-shadow"), "the toolchain install did not run: {named}");
    remove(scene.root.as_path());
}

/// What:
///  A record naming a tool this release does not know fails,
/// and starts nothing.
/// Why:
///   A record a person or another release wrote must be reported rather than acted on,
/// and a tool the service cannot name is no work it may guess at.
#[test]
fn a_record_naming_no_known_tool_fails_and_starts_nothing() {
    let scene: Scene = scene("relink-job-unknown");
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 0).as_bytes());
    let mut item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm]);
    item.parent = String::from("{\"root\":\"/w\",\"tools\":[\"npm\"]}");
    let verdict: Verdict = relink(&scene, &item);
    assert!(matches!(verdict, Verdict::Failed(_)), "{verdict:?}");
    assert_eq!(starts(&scene), "", "a tool was started for a record this release cannot read");
    remove(scene.root.as_path());
}

/// What:
///  A worktree whose state a person's own command already relinked starts no tool and is done.
/// Why:
///   The record is written before the wake and may be drained after the person ran the install by hand,
/// so a tool with nothing to do is not an error.
#[test]
fn a_worktree_that_needs_nothing_starts_no_tool() {
    let scene: Scene = scene("relink-job-nothing");
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 0).as_bytes());
    executable(scene.bin.join("mpm-shadow").as_path(), installing_toolchain(&scene, 0).as_bytes());
    std::fs::remove_dir(scene.worktree.join("node_modules")).expect("remove node_modules");
    std::fs::remove_file(scene.worktree.join("pnpm-lock.yaml")).expect("remove the lockfile");
    std::fs::remove_file(scene.worktree.join(".cache/toolchain-launchers/mpm.launch")).expect("remove the launcher record");
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm, RelinkTool::Toolchain]);
    assert_eq!(relink(&scene, &item), Verdict::Done);
    assert_eq!(starts(&scene), "");
    remove(scene.root.as_path());
}

/// What:
///  A record whose worktree was removed and replaced by an unrelated directory touches nothing there.
/// Why:
///   A queued record outlives the worktree it was written for:
/// a service that died holds a claim that runs out and requeues it,
/// and a person may remove the worktree and put their own directory at that path meanwhile.
/// The install rewrites state in place,
/// so starting it in whatever directory now holds the recorded path would modify package state this record was
/// never written for.
#[test]
fn a_delayed_relink_leaves_a_replacement_directory_alone() {
    let scene: Scene = scene("relink-job-replaced-path");
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 0).as_bytes());
    executable(scene.bin.join("mpm-shadow").as_path(), installing_toolchain(&scene, 0).as_bytes());
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm, RelinkTool::Toolchain]);
    std::fs::remove_dir_all(scene.worktree.as_path()).expect("the worktree is removed");
    std::fs::create_dir_all(scene.worktree.join("node_modules")).expect("a replacement node_modules");
    std::fs::write(scene.worktree.join("pnpm-lock.yaml"), b"lockfileVersion: '9.0'\n").expect("a replacement lockfile");
    assert!(!scene.worktree.join(".git").exists(), "the replacement is no worktree of the repository");
    let verdict: Verdict = relink(&scene, &item);
    assert!(matches!(verdict, Verdict::Skipped(_)), "a replacement directory is left alone: {verdict:?}; it started {:?}", starts(&scene));
    assert_eq!(starts(&scene), "", "a tool was started in a directory the record was not written for");
    assert_eq!(paths(&scene), "", "a tool was started in a directory the record was not written for");
    remove(scene.root.as_path());
}

/// What:
///  The install is the program the add resolved,
/// on the add's search path,
/// whatever path the service process was started with.
/// Why:
///   The owner decided that the repair starts whatever pnpm is first on the person's path,
/// and the service is a resident process of another session,
/// so its own path may hold no pnpm or another one.
/// The record is the only thing the service reads,
/// so the path the add resolved on has to travel with it.
#[test]
fn the_service_starts_the_program_the_add_resolved() {
    let scene: Scene = scene("relink-job-service-path");
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 0).as_bytes());
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm]);
    let empty: PathBuf = scene.root.join("service-search-path");
    std::fs::create_dir_all(empty.as_path()).expect("an empty search path for the service");
    let service_environment: RepairEnvironment = RepairEnvironment { search_path: Some(empty.clone().into_os_string()), ..scene.environment.clone() };
    let verdict: Verdict = crate::lazy_runtime::run_on_runtime(relink_with(&item, &service_environment)).expect("the relinking runs");
    assert_eq!(verdict, Verdict::Done, "the service could not start the pnpm the add found");
    let named: String = starts(&scene);
    assert!(named.trim_end().ends_with(scene.worktree.to_str().expect("utf8")), "the install did not run in the worktree: {named}");
    let path: String = paths(&scene);
    assert!(path.contains(scene.bin.to_str().expect("utf8")), "the install did not see the add's search path: {path}");
    remove(scene.root.as_path());
}

/// What:
///  A record whose search path names a directory relatively starts the program the add resolved,
/// and the tool sees that search path spelled from the directory the add resolved it in.
/// Why:
///   An entry of a search path without a leading slash means a directory of the working directory,
/// so a shell that puts `caller-bin` on `PATH` means its own `caller-bin`,
/// and the add found the pnpm there and handed the work over.
/// The service is a resident process of the session and works in its own directory,
/// so the record carries the directory the add resolved it in,
/// and the service reads the entry from that one.
/// Without it the service finds no program,
/// settles the record failed,
/// and the worktree keeps the links the copy left while the add has already said the service will relink it.
///
/// The record here is the payload text a queue holds,
/// so the control asks the reader the question it asks of the add's own record.
#[test]
fn a_relative_search_entry_starts_the_program_the_add_resolved() {
    let scene: Scene = scene("relink-job-relative-search-entry");
    // The add's own `PATH` names this directory relatively: `bin`, resolved from the fixture root, which is `scene.bin`.
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 0).as_bytes());
    let payload: String = format!(
        r#"{{"root":"{root}","searchPath":"bin","searchFrom":"{from}","tools":["pnpm"]}}"#,
        root = scene.worktree.display(),
        from = scene.root.display()
    );
    let item: WorkItem = WorkItem {
        kind: crate::work_item::WorkKind::Relink,
        repository: scene.worktree.clone(),
        git_dir: scene.common.clone(),
        branch_ref: String::from(crate::package_repair_relink::RELINK_REFERENCE),
        oid: String::from("0000000000000000000000000000000000000000"),
        parent: payload,
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    // The service's own search path holds no pnpm, as a service started by the manager has none of the person's.
    let elsewhere: PathBuf = scene.root.join("service-search-path");
    std::fs::create_dir_all(elsewhere.as_path()).expect("an empty search path for the service");
    let service_environment: RepairEnvironment = RepairEnvironment { search_path: Some(elsewhere.clone().into_os_string()), ..scene.environment.clone() };
    let verdict: Verdict = crate::lazy_runtime::run_on_runtime(relink_with(&item, &service_environment)).expect("the relinking runs");
    assert_eq!(verdict, Verdict::Done, "the service could not start the pnpm the add found through a relative search path entry");
    let named: String = starts(&scene);
    assert!(named.trim_end().ends_with(scene.worktree.to_str().expect("utf8")), "the install did not run in the worktree: {named}");
    let path: String = paths(&scene);
    assert!(path.contains(scene.bin.to_str().expect("utf8")), "the install did not see the add's search path: {path}");
    remove(scene.root.as_path());
}
