//! Controls of the service's relinking:
//! which tools run,
//! in which order,
//! and what a tool that fails leaves behind.

use super::relink_with;
use crate::package_repair_relink::{RelinkEntry, RelinkTool, SearchPathOfTheAdd, item_of};
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
    // The two records Git writes beside a registration: the pointer's reciprocal back-link and the common directory
    //  the registration belongs to. A copied pointer in a replacement directory has neither of the first.
    std::fs::write(admin.join("gitdir"), format!("{}\n", worktree.join(".git").display())).expect("the registration's back-link");
    std::fs::write(admin.join("commondir"), b"../..\n").expect("the registration's common directory");
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
    let entry: RelinkEntry = RelinkEntry { root: scene.worktree.clone(), search_path: SearchPathOfTheAdd::Set(scene.environment.search_path.clone().expect("the scene's search path")), tools };
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

/// How long a control waits for a relink that must answer without waiting for anything,
/// before it releases whatever the relink is waiting for and reports.
const ANSWER_BOUND: Duration = Duration::from_secs(5);

/// Make a named pipe at `path`,
/// which blocks whoever opens it for reading until a writer comes.
fn make_a_named_pipe(path: &std::path::Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(made, 0, "mkfifo {}", path.display());
}

/// A fixture tool that names its start and then replaces the worktree directory with an unrelated one,
/// keeping a copy of the registration's pointer so only the directory's own identity tells them apart.
fn replacing_tool(scene: &Scene) -> String {
    let admin: PathBuf = scene.common.join("worktrees/topic");
    return format!(
        "#!/bin/sh\necho \"pnpm $* in $(pwd -P)\" >> '{started}'\ncd /\n/bin/rm -rf '{worktree}' 2>>'{started}'\n/bin/mkdir -p '{worktree}/node_modules' '{worktree}/.cache/toolchain-launchers' 2>>'{started}'\nprintf \"lockfileVersion: '9.0'\\n\" > '{worktree}/pnpm-lock.yaml'\nprintf '{{\"lockFile\":\"{elsewhere}\"}}\\n' > '{worktree}/.cache/toolchain-launchers/mpm.launch'\nprintf 'gitdir: %s\\n' '{admin}' > '{worktree}/.git'\nexit 0\n",
        started = scene.record.display(),
        worktree = scene.worktree.display(),
        admin = admin.display(),
        elsewhere = ELSEWHERE,
    );
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
/// the next tool still runs,
/// and the record keeps its claim for another try.
/// Why:
///   One tool's failure must not leave the other tool's state unrelinked,
///   the person is told which tool failed and how to finish it,
///   and a failure this machine's ordinary conditions produce (two installers in one `node_modules`)
///   passes when the tool runs alone.
#[test]
fn a_tool_that_fails_is_named_the_next_still_runs_and_the_record_is_tried_again() {
    let scene: Scene = scene("relink-job-fails");
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 3).as_bytes());
    executable(scene.bin.join("mpm-shadow").as_path(), installing_toolchain(&scene, 0).as_bytes());
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm, RelinkTool::Toolchain]);
    let verdict: Verdict = relink(&scene, &item);
    let Verdict::Retry(reason) = verdict else { panic!("a failing tool is a failure that can pass: {verdict:?}") };
    assert!(reason.starts_with("pnpm: "), "{reason}");
    assert!(reason.contains("pnpm --dir "), "the words do not name the command that finishes it: {reason}");
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
///  A replacement directory that keeps a copy of the old pointer is left alone,
///  because the registration it names does not point back at it.
/// Why:
///   A queued record outlives the worktree it was written for,
/// and a person may put their own directory at that path with a copy of the pointer the old worktree had.
/// The registration Git writes carries a `gitdir` back-link naming the worktree's own pointer,
/// so a directory that has the pointer but not the back-link is no worktree of this repository,
/// and its package state is not rewritten.
#[test]
fn a_replacement_carrying_the_old_pointer_is_left_alone() {
    let scene: Scene = scene("relink-job-stale-pointer");
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 0).as_bytes());
    executable(scene.bin.join("mpm-shadow").as_path(), installing_toolchain(&scene, 0).as_bytes());
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm, RelinkTool::Toolchain]);
    let old_pointer: Vec<u8> = std::fs::read(scene.worktree.join(".git")).expect("the old pointer");
    std::fs::remove_dir_all(scene.worktree.as_path()).expect("the worktree is removed");
    std::fs::create_dir_all(scene.worktree.join("node_modules")).expect("a replacement node_modules");
    std::fs::write(scene.worktree.join("pnpm-lock.yaml"), b"lockfileVersion: '9.0'\n").expect("a replacement lockfile");
    std::fs::write(scene.worktree.join(".git"), old_pointer.as_slice()).expect("a copy of the old pointer");
    std::fs::remove_file(scene.common.join("worktrees/topic/gitdir")).expect("the registration points back at nothing");
    let verdict: Verdict = relink(&scene, &item);
    assert!(matches!(verdict, Verdict::Skipped(_)), "a copied pointer is no registration: {verdict:?}; it started {:?}", starts(&scene));
    assert_eq!(starts(&scene), "", "a tool was started in a directory the record was not written for");
    remove(scene.root.as_path());
}

/// What:
///  A `.git` that is a named pipe is refused instead of waited for.
/// Why:
///   Reading the pointer with `std::fs::read` waits inside `open(2)` for a writer of a pipe,
/// and that read happens before any tool starts,
/// so no quiet window and no ceiling applies to it:
/// one malformed pointer would hold the service's round for ever
/// (`SPEC.md`,
/// "Every wait has a bound").
#[test]
fn a_pointer_that_is_a_named_pipe_is_refused_without_waiting() {
    use std::io::Write as _;
    let scene: Scene = scene("relink-job-pointer-pipe");
    executable(scene.bin.join("pnpm").as_path(), recording_tool(&scene, "pnpm", 0).as_bytes());
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm]);
    let pointer: PathBuf = scene.worktree.join(".git");
    let old_pointer: Vec<u8> = std::fs::read(pointer.as_path()).expect("the old pointer");
    std::fs::remove_file(pointer.as_path()).expect("the regular pointer is removed");
    make_a_named_pipe(pointer.as_path());
    let (sender, receiver) = std::sync::mpsc::channel::<Verdict>();
    let environment: RepairEnvironment = scene.environment.clone();
    let worker_item: WorkItem = item.clone();
    let worker: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let verdict: Verdict = crate::lazy_runtime::run_on_runtime(relink_with(&worker_item, &environment)).expect("the relinking runs");
        let _ = sender.send(verdict);
    });
    let answered: Result<Verdict, std::sync::mpsc::RecvTimeoutError> = receiver.recv_timeout(ANSWER_BOUND);
    if answered.is_err() {
        // Release the reader so this control reports what it found instead of leaving a thread waiting for ever.
        let mut writer: std::fs::File = std::fs::OpenOptions::new().write(true).open(pointer.as_path()).expect("release the reader");
        writer.write_all(old_pointer.as_slice()).expect("write the pointer");
        drop(writer);
    }
    let verdict: Verdict = match answered {
        Ok(found) => found,
        Err(_) => receiver.recv_timeout(ANSWER_BOUND).expect("the released reader lets the relinking finish"),
    };
    worker.join().expect("join the relinking");
    assert!(matches!(verdict, Verdict::Skipped(_)), "a pointer that is no regular file is no registration: {verdict:?}");
    assert_eq!(starts(&scene), "", "a tool was started for a directory whose pointer cannot be read as a file");
    remove(scene.root.as_path());
}

/// What:
///  A worktree replaced between two tools of one record starts no further tool.
/// Why:
///   The tools of one record run one after another and the first can take a minute,
/// so the directory that was proved to be the record's worktree can be gone by the time the next tool starts.
/// The replacement keeps a copy of the pointer and the registration still points back at that path,
/// so only the directory's own identity tells it from the worktree the run began in.
#[test]
fn a_worktree_replaced_between_two_tools_starts_no_further_tool() {
    let scene: Scene = scene("relink-job-replaced-between-tools");
    executable(scene.bin.join("pnpm").as_path(), replacing_tool(&scene).as_bytes());
    executable(scene.bin.join("mpm-shadow").as_path(), installing_toolchain(&scene, 0).as_bytes());
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm, RelinkTool::Toolchain]);
    let verdict: Verdict = relink(&scene, &item);
    let named: String = starts(&scene);
    assert!(!named.contains("mpm-shadow"), "the second tool ran in a directory the record was not written for: {named}");
    assert!(matches!(verdict, Verdict::Skipped(_)), "a worktree replaced between two tools is named as left alone: {verdict:?}");
    remove(scene.root.as_path());
}

/// What:
///  A toolchain tool that names itself,
///  its start,
///  and then rewrites the launcher record as a real install does.
/// Why:
///   Two programs can stand for one tool,
/// the add's own link and one the service's search path holds,
/// and only the name each records tells which one the service started.
fn toolchain_tool(scene: &Scene, label: &str) -> String {
    let launcher: PathBuf = scene.worktree.join(".cache/toolchain-launchers/mpm.launch");
    let own: PathBuf = scene.worktree.join("toolchain.lock.json");
    return format!(
        "#!/bin/sh\necho \"{label} $* in $(pwd -P)\" >> '{started}'\nprintf '{{\"lockFile\":\"%s\"}}\\n' '{own}' > '{launcher}'\nexit 0\n",
        started = scene.record.display(),
        own = own.display(),
        launcher = launcher.display()
    );
}

/// What:
///  An add whose environment held no search path does not let the service's own search path choose the tool.
/// Why:
///   With no `PATH` the add selects the worktree's own `.cache/toolchain-bin/mpm-shadow`,
/// and the service is a resident process of another session whose own path may hold a different one.
/// A record that cannot say "the add had no search path" makes the service start its own mpm-shadow instead
/// of the program the add resolved.
#[test]
fn an_add_with_no_search_path_does_not_borrow_the_services_path() {
    let scene: Scene = scene("relink-job-no-add-search-path");
    let own_link: PathBuf = scene.worktree.join(".cache/toolchain-bin/mpm-shadow");
    std::fs::create_dir_all(own_link.parent().expect("the link's directory")).expect("the worktree's toolchain link directory");
    executable(own_link.as_path(), toolchain_tool(&scene, "own-link").as_bytes());
    let service_bin: PathBuf = scene.root.join("service-bin");
    std::fs::create_dir_all(service_bin.as_path()).expect("the service's own search path");
    executable(service_bin.join("mpm-shadow").as_path(), toolchain_tool(&scene, "service-path").as_bytes());
    let add_environment: RepairEnvironment = RepairEnvironment { search_path: None, ..scene.environment.clone() };
    let needed: crate::package_repair_toolchain::ToolchainNeed =
        crate::lazy_runtime::run_on_runtime(crate::package_repair_toolchain::need_within_the_budget(scene.worktree.as_path(), &add_environment))
            .expect("the need is decided")
            .expect("the launcher records are read");
    assert!(matches!(needed, crate::package_repair_toolchain::ToolchainNeed::Needed), "the worktree's own link is the program the add resolved");
    let entry: RelinkEntry = RelinkEntry { root: scene.worktree.clone(), search_path: SearchPathOfTheAdd::Unset, tools: vec![RelinkTool::Toolchain] };
    let item: WorkItem = item_of(scene.common.as_path(), scene.worktree.as_path(), &entry).expect("the record");
    let service_environment: RepairEnvironment = RepairEnvironment { search_path: Some(service_bin.clone().into_os_string()), ..scene.environment.clone() };
    let verdict: Verdict = crate::lazy_runtime::run_on_runtime(relink_with(&item, &service_environment)).expect("the relinking runs");
    let named: String = starts(&scene);
    assert!(!named.contains("service-path"), "the service started the mpm-shadow of its own search path for an add that had none: {named}");
    assert!(named.contains("own-link"), "the worktree's own link is the program the add selected: {named}");
    assert_eq!(verdict, Verdict::Done, "{verdict:?}");
    remove(scene.root.as_path());
}

/// What:
///  A registration that cannot be read now keeps the record for another try.
/// Why:
///   A permission or an input-output error may pass,
/// and the worktree may still need its relink,
/// so answering "left as it is" settles a repair nobody did
/// and no record is left to try again.
/// The control makes the read answer the way a refused read does,
/// because the gate runs this suite as root,
/// where no mode makes a file unreadable.
#[test]
fn a_registration_that_cannot_be_read_now_is_tried_again() {
    let scene: Scene = scene("relink-job-undecided-read");
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm]);
    let entry: RelinkEntry = crate::package_repair_relink::decode(item.parent.as_str()).expect("the record's entry");
    let held: crate::package_repair_relink_job_worktree_proof::HeldWorktree =
        crate::package_repair_relink_job_worktree_proof::HeldWorktree::open(scene.worktree.as_path()).expect("the worktree is held open");
    let answered: Option<Verdict> = {
        crate::worktree_registration_proof::THE_RECORD_READ_ANSWERS_UNDECIDED.with(|answer| answer.set(true));
        let refusal: Option<Verdict> = crate::package_repair_relink_job_worktree_proof::refusal_of_a_replaced_or_unregistered_worktree(&item, &entry, &held);
        crate::worktree_registration_proof::THE_RECORD_READ_ANSWERS_UNDECIDED.with(|answer| answer.set(false));
        refusal
    };
    let Some(verdict) = answered else { panic!("a registration that cannot be read is a refusal") };
    assert!(matches!(verdict, Verdict::Retry(_)), "a read that may pass must keep the record: {verdict:?}");
    remove(scene.root.as_path());
}

/// What:
///  The words a refusal is settled with name the worktree the record carries,
///  not a descriptor the run reads it through.
/// Why:
///  the pointer is read through the directory the run holds open,
///  so the path inside the read is `/proc/self/fd/<descriptor>/.git`,
///  which is nothing the person typed and nothing they could look for;
///  the service's journal is what they read,
///  and it has to name the directory the record names.
#[test]
fn a_refusal_names_the_worktree_the_record_carries() {
    let scene: Scene = scene("relink-job-refusal-words");
    let item: WorkItem = item_of_the_scene(&scene, vec![RelinkTool::Pnpm]);
    // A directory at the recorded path that is no Git worktree at all.
    std::fs::remove_file(scene.worktree.join(".git")).expect("the pointer is removed");
    let verdict: Verdict = relink(&scene, &item);
    let Verdict::Skipped(words) = verdict else { panic!("a directory with no pointer is left as it is: {verdict:?}") };
    assert!(words.contains(scene.worktree.to_str().expect("utf8")), "the words name the worktree the record carries: {words}");
    assert!(!words.contains("/proc/self/fd"), "the words name no descriptor the person cannot look for: {words}");
    remove(scene.root.as_path());
}
