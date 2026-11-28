//! What:
//! The controls of what a rebuild of the exposure farm removes,
//! spares and names:
//! the temporary link of an install that still runs,
//! the temporary link an install that ended left,
//! a temporary link nothing can be judged about,
//! the temporary names an earlier build wrote,
//! a dotted name that is no temporary link,
//! the farm's own `mpm-shadow` link,
//! a declared name a partial sync does not link,
//! a link another run removed first,
//! at each of the three calls a removal makes,
//! and the removals a run names before a later one fails.
//! Why:
//! The rebuild is the one thing in this tool that deletes from a folder a person sees,
//! and every state below is one the released `0.2.7-alpha.18` got wrong or never named:
//! it removed another install's temporary link (its rename then failed with `ENOENT`),
//! it removed the farm's `mpm-shadow` link although an installed copy resolved,
//! and a removal that failed after earlier ones left those unnamed.
//! Every name and line asserted here is the one a run prints,
//! and two states stand in for ones no disposable fixture can reach on this machine:
//! the facts `the_owner_of_an_install_that_ended` reads belong to a real process the kernel no longer lists,
//! so the temporary name it builds is the one a killed install left,
//! and `the_farm_shadow_link` writes an installed copy by hand,
//! below the toolchain root with a `current` link,
//! which is the layout an install of it leaves.
//! A stand-in says so where it is written,
//! and no claim about a real program rests on one.

/// Imports `PathBuf` from `std::path`,
/// which is the owned path type (its sibling `Path` borrows a string the caller keeps).
use std::path::PathBuf;
/// Imports `Child` from `std::process`,
/// which is the handle a program the test starts answers through.
use std::process::Child;

/// Imports `symlink` from `std::os::unix::fs`,
/// which makes a symbolic link where the tool's own writes make one.
use std::os::unix::fs::symlink;

/// Imports the facts a temporary link's name records about the install holding it.
use super::archive_staging::Owner;

/// Imports the moment a running process started,
/// which the name of a link an install wrote carries.
use super::archive_staging::start_time_of_a_process;
/// Imports the rebuild whose removal pass these controls are about.
use super::exposure::rebuild_exposure;
/// Imports the plants a control makes at one removal.
use super::exposure::{
    What,
    planting_at_the_removal_of,
};
/// Imports the disposable scope,
/// the lock and the fakes a control runs against.
use super::fakes::{
    Rig,
    write_executable,
};
/// Imports the program of a process the kernel no longer lists,
/// which is what a killed run's name carries.
use super::fixtures::pid_of_a_process_that_ended;
/// Imports the name a held link carries,
/// which the rebuild reads to tell a leftover from a link an install still holds.
use super::install_support::the_temporary_link_of;
/// Imports `sync_tools`,
/// the one command that runs the rebuild after its installs.
use super::sync::sync_tools;
/// Imports the builder of the lines `install` prints.
use super::sync_report_text::format_sync_report;

/// The id the sample plan gives `mpm-shadow`,
/// which a control that is about a scope without that row removes from its lock.
const SHADOW_TOOL_ID: &str = "cargo:monochromatic-mpm-shadow";

/// The name the farm's link to an installed `mpm-shadow` carries,
/// which is that program's own file name.
const SHADOW_PROGRAM: &str = "mpm-shadow";

/// The version the control installs `mpm-shadow` at,
/// which no lock in a fixture pins.
const AN_INSTALLED_VERSION: &str = "0.2.7-alpha.99";

/// The bin a fixture row declares and an install links,
/// which is the name an install holds a temporary link for.
const A_DECLARED_BIN: &str = "thirdbin";

/// The name of a link no row declares,
/// which an earlier install left in the farm.
const AN_ORPHAN: &str = "orphan";

/// The name of a second link no row declares,
/// which the removal that fails is about.
const A_SECOND_ORPHAN: &str = "orphan-d";

/// The version a stand-in install of another tool sits at,
/// which a `current` link beside it names.
const ANOTHER_TOOLS_VERSION: &str = "1.0.0";

/// The directory another tool's install sits in,
/// below the toolchain root,
/// which is what tells it from this program's own install.
const ANOTHER_TOOLS_DIRECTORY: &str = "fixtool";

/// The crate this program is published as,
/// which names the directory a cargo-route install of it sits in.
const THE_PROGRAMS_CRATE: &str = env!("CARGO_PKG_NAME");

/// A process namespace's inode number that is not this one,
/// which no reader on this machine is ever inside.
const ANOTHER_PROCESS_NAMESPACE: u64 = 1;

/// What:
/// A rig whose lock declares no tool,
/// so a sync does nothing but the steps after its installs,
/// and the rebuild runs over an empty list.
/// Why:
/// A control of the removal must see the removal's own lines alone,
/// and a fixture tool that cannot install would put its failure beside them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aRigWithNoTools(): Rig { const rig = Rig.new(); rig.lock.tools.clear(); return rig; }
/// ```
fn a_rig_with_no_tools() -> Rig {
    let mut rig: Rig = Rig::new();
    rig.lock.tools.clear();
    return rig;
}

/// What:
/// A program beside the farm that a farm link points at,
/// written through the fixture's own executable writer.
/// Why:
/// A link whose target is there is the ordinary shape of a link an install made,
/// and the removal must judge the link,
/// never whether its target lives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aProgramBesideTheFarm(rig: Rig, name: string): string
/// ```
fn a_program_beside_the_farm(
    rig: &Rig,
    name: &str,
) -> PathBuf {
    let program: PathBuf = rig.scope.root.join("build").join(name);
    write_executable(&program);
    return program;
}

/// What:
/// The facts a temporary link's name records for an install that has ended:
/// the number of a real process the kernel no longer lists,
/// the moment that process started,
/// and this boot and process namespace.
/// Why:
/// A kill between a link's creation and its rename leaves exactly such a name,
/// so a control builds the state instead of holding a system call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theOwnerOfAnInstallThatEnded(): Owner
/// ```
fn the_owner_of_an_install_that_ended() -> Owner {
    let mut child: Child = std::process::Command::new("true")
        .spawn()
        .expect("start a program that ends at once");
    let pid: u32 = child.id();
    let started: u64 =
        start_time_of_a_process(pid).expect("the kernel reports when the program started");
    child.wait().expect("wait for the program");
    let here: Owner = Owner::here();
    return Owner {
        pid,
        started,
        boot: here.boot,
        namespace: here.namespace,
    };
}

/// What:
/// The name the build before this round wrote a temporary link under,
/// `.<name>.<pid>.tmp`.
/// Why:
/// A farm written by `0.2.7-alpha.18` still holds such names,
/// and a control has to plant one to show what the rebuild does with it.
/// STAND-IN:
/// the shape is copied from that build's write,
/// and no claim about a real program rests on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theNameTheEarlierBuildWrote(linkName: string, pid: number): string
/// ```
fn the_name_the_earlier_build_wrote(
    link_name: &str,
    pid: u32,
) -> String {
    return format!(".{link_name}.{pid}.tmp");
}

/// What:
/// A farm holding one link for `A_DECLARED_BIN` whose install stands between the two steps of its write,
/// and the names that link has while the install holds it and after.
/// Why:
/// A second rebuild that runs in that instant is what the release failed:
/// it removed the temporary link,
/// and the install's rename then had no name to move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aFarmWithAHeldInstall(rig: Rig): [string, string, string]
/// ```
fn a_farm_with_a_held_install(rig: &Rig) -> (PathBuf, PathBuf, PathBuf) {
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(rig, A_DECLARED_BIN);
    let final_name: PathBuf = rig.scope.exposure_dir.join(A_DECLARED_BIN);
    // What: the dotted name the install holds the link under, built for the program running now.
    // Why: this process stands in for the install that is between its two steps.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const temporary = theTemporaryLinkOf(pathJoin(farm, A_DECLARED_BIN), ownerOfThisProcess());
    // ```
    let temporary: PathBuf = the_temporary_link_of(&final_name, Owner::here());
    symlink(&target, &temporary).expect("stand in for the install's held link");
    return (temporary, final_name, target);
}

/// What:
/// The farm's `mpm-shadow` link,
/// pointing at an installed copy through that install's `current` link.
/// Why:
/// That is the link every run by an installed copy makes,
/// and the one a shell outside a checkout reaches the program through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFarmShadowLink(rig: Rig): string
/// ```
fn the_farm_shadow_link(rig: &Rig) -> PathBuf {
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let install: PathBuf = rig.scope.toolchain_root.join(SHADOW_PROGRAM);
    let version: PathBuf = install.join(AN_INSTALLED_VERSION);
    let program: PathBuf = version.join(SHADOW_PROGRAM);
    write_executable(&program);
    symlink(AN_INSTALLED_VERSION, install.join("current")).expect("link current");
    let link: PathBuf = rig.scope.exposure_dir.join(SHADOW_PROGRAM);
    symlink(install.join("current").join(SHADOW_PROGRAM), &link)
        .expect("link the installed program under the farm's own name");
    return link;
}

/// What:
/// A rig whose lock declares no `mpm-shadow` row,
/// so the farm's own link for the program is a name no row declares.
/// Why:
/// The row would own the name,
/// and the rebuild would take it for the plan's rather than the program's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aRigWhoseLockDeclaresNoShadow(): Rig
/// ```
fn a_rig_whose_lock_declares_no_shadow() -> Rig {
    let rig: Rig = a_rig_with_no_tools();
    assert!(
        !rig.lock
            .tools
            .iter()
            .any(|tool| return tool.declaration.id == SHADOW_TOOL_ID),
        "the lock declares no {SHADOW_TOOL_ID}"
    );
    return rig;
}

/// What:
/// A program a farm link points at inside another tool's install directory.
/// Why:
/// The farm's own-name link is kept only while it resolves into an install of this program,
/// and this is the file an install of another tool could hold under that name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anInstallOfAnotherTool(rig: Rig): string
/// ```
fn an_install_of_another_tool(rig: &Rig) -> PathBuf {
    let install: PathBuf = rig.scope.toolchain_root.join(ANOTHER_TOOLS_DIRECTORY);
    let version: PathBuf = install.join(ANOTHER_TOOLS_VERSION);
    let program: PathBuf = version.join(SHADOW_PROGRAM);
    write_executable(&program);
    symlink(ANOTHER_TOOLS_VERSION, install.join("current")).expect("link current");
    return program;
}

/// What:
/// A program of this program's own name inside the directory a cargo-route install of it sits in,
/// `<toolchain root>/cargo/<crate>/<version>/bin/mpm-shadow`.
/// Why:
/// The repository bootstraps this tool with `cargo install` as well as with a release archive,
/// so the farm's own-name link has to be kept for that install of it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aCargoInstallOfTheShadow(rig: Rig): string
/// ```
fn a_cargo_install_of_the_shadow(rig: &Rig) -> PathBuf {
    let install: PathBuf = rig
        .scope
        .toolchain_root
        .join("cargo")
        .join(THE_PROGRAMS_CRATE);
    let version: PathBuf = install.join(AN_INSTALLED_VERSION);
    let program: PathBuf = version.join("bin").join(SHADOW_PROGRAM);
    write_executable(&program);
    symlink(AN_INSTALLED_VERSION, install.join("current")).expect("link current");
    return program;
}

/// What:
/// A link in the farm called after this program,
/// pointing where `target` names.
/// Why:
/// One name of the farm carries the rule the controls here are about,
/// and only its target differs between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFarmLinkTo(rig: Rig, target: string): string
/// ```
fn the_farm_link_to(
    rig: &Rig,
    target: &std::path::Path,
) -> PathBuf {
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let link: PathBuf = rig.scope.exposure_dir.join(SHADOW_PROGRAM);
    symlink(target, &link).expect("link the program's own name into the farm");
    return link;
}

/// What:
/// The line one sentence of the report occupies,
/// or nothing when the report holds no such line.
/// Why:
/// A control about a sentence reads the whole line,
/// because the words a person acts on are the line's and not a substring of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theLineStartingWith(text: string, start: string): string | undefined
/// ```
fn the_line_starting_with(
    text: &str,
    start: &str,
) -> Option<String> {
    return text
        .lines()
        .find(|line| return line.starts_with(start))
        .map(|line| return line.to_string());
}

// What:
// `#[test] fn a_sync_spares_the_temporary_link_of_an_install_that_still_runs()`
// holds a farm in the state an install stands in while its write is between its two steps,
// runs the whole sync a second install is,
// and then gives the link its final name as the held install would.
// Why:
// The release removed that temporary link,
// and the held install's `rename` then failed with
// `cannot place the link <farm>/thirdbin: No such file or directory (os error 2)`;
// two installs at once must both finish.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync spares the temporary link of an install that still runs', () => { ... });
// ```
#[test]
fn a_sync_spares_the_temporary_link_of_an_install_that_still_runs() {
    let rig: Rig = a_rig_with_no_tools();
    let (temporary, final_name, target) = a_farm_with_a_held_install(&rig);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    let held: String = temporary
        .file_name()
        .expect("a file name")
        .to_string_lossy()
        .to_string();
    assert!(
        std::fs::symlink_metadata(&temporary).is_ok(),
        "the temporary link belongs to an install that still runs, so the sync leaves it; {text}"
    );
    assert!(
        !text.contains(&held),
        "the sync says nothing about the temporary link it left; {text}"
    );
    // What: the second step of the held install's write, which is the rename the release could not make.
    // Why: the control's whole point is that the held install still finishes after the other run.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // renameSync(temporary, finalName);
    // ```
    std::fs::rename(&temporary, &final_name)
        .expect("the install that held the link still gives it its name");
    assert_eq!(
        std::fs::read_link(&final_name).expect("a symlink"),
        target,
        "the link a person's `PATH` now reaches names the program the install found"
    );
}

// What:
// `#[test] fn a_sync_removes_and_names_the_temporary_link_of_an_install_that_ended()`
// plants a temporary link named after an install that has ended,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// The release added this removal and it must stay:
// a kill between a link's creation and its rename would otherwise leave the dotted name for ever,
// and a person must be told which name left the folder and that a kill left it.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync removes and names the temporary link of an install that ended', () => { ... });
// ```
#[test]
fn a_sync_removes_and_names_the_temporary_link_of_an_install_that_ended() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(&rig, A_DECLARED_BIN);
    let temporary: PathBuf = the_temporary_link_of(
        &rig.scope.exposure_dir.join(A_DECLARED_BIN),
        the_owner_of_an_install_that_ended(),
    );
    symlink(&target, &temporary).expect("stand in for the link a killed install left");
    let name: String = temporary
        .file_name()
        .expect("a file name")
        .to_string_lossy()
        .to_string();
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&temporary).is_err(),
        "an install that ended left this temporary link, so the sync removes it; {text}"
    );
    assert!(
        text.lines().any(|line| {
            return line.starts_with("toolchain install: removed the temporary link")
                && line.contains(&name);
        }),
        "the sync names the temporary link it removed and says what it was; {text}"
    );
}

// What:
// `#[test] fn a_sync_spares_a_temporary_link_an_install_in_another_process_namespace_holds()`
// plants a temporary link whose name records a process namespace that is not this one,
// and reads the reader as one inside that namespace,
// runs the whole sync,
// and reads the folder.
// Why:
// A process id means one thing only inside the namespace that gave it out,
// so this run cannot say whether the install that wrote the name still runs;
// a link that may belong to a live install stays, and a later rebuild in that namespace asks again.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync spares a temporary link an install in another process namespace holds', () => { ... });
// ```
#[test]
fn a_sync_spares_a_temporary_link_an_install_in_another_process_namespace_holds() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(&rig, A_DECLARED_BIN);
    let held: Owner = the_owner_of_an_install_that_ended();
    // What: the same name, recorded in a process namespace this reader is not inside.
    // Why: the number in it means nothing here, which is the state a container sharing the farm leaves.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const temporary = theTemporaryLinkOf(link, { ...held, namespace: ANOTHER_PROCESS_NAMESPACE });
    // ```
    let temporary: PathBuf = the_temporary_link_of(
        &rig.scope.exposure_dir.join(A_DECLARED_BIN),
        Owner {
            pid: held.pid,
            started: held.started,
            boot: held.boot,
            namespace: Some(ANOTHER_PROCESS_NAMESPACE),
        },
    );
    symlink(&target, &temporary).expect("stand in for a link another namespace's install holds");
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&temporary).is_ok(),
        "the number means nothing in this namespace, so the sync leaves the link; {text}"
    );
}

// What:
// `#[test] fn a_sync_keeps_and_names_the_temporary_link_of_a_run_that_could_not_read_its_own_place()`
// plants a temporary link whose name carries no boot and no process namespace,
// as the write of a run that could not read its own leaves,
// and whose number belongs to no process,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// The name records no process namespace,
// so nothing here can show that the install that wrote it was in this one:
// a number free here says nothing about a name written in another namespace,
// where the install that holds it may still run.
// The link stays, and the report says which name it left and why.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync keeps and names the temporary link of a run that could not read its own place', () => { ... });
// ```
#[test]
fn a_sync_keeps_and_names_the_temporary_link_of_a_run_that_could_not_read_its_own_place() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(&rig, A_DECLARED_BIN);
    let ended: Owner = the_owner_of_an_install_that_ended();
    // What: the same name with the boot and the namespace left out, as a run that could not read its own writes it.
    // Why: the name then records no place, so no run can say which install holds the link.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const temporary = theTemporaryLinkOf(link, { ...ended, boot: undefined, namespace: undefined });
    // ```
    let temporary: PathBuf = the_temporary_link_of(
        &rig.scope.exposure_dir.join(A_DECLARED_BIN),
        Owner {
            pid: ended.pid,
            started: ended.started,
            boot: None,
            namespace: None,
        },
    );
    symlink(&target, &temporary)
        .expect("stand in for a link a run that could not place itself left");
    let name: String = temporary
        .file_name()
        .expect("a file name")
        .to_string_lossy()
        .to_string();
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&temporary).is_ok(),
        "the name records no place, so the sync leaves the link; {text}"
    );
    let start: String = format!("toolchain install: left the temporary link {name}: ");
    assert_eq!(
        the_line_starting_with(&text, &start),
        Some(format!(
            "{start}this run cannot judge the install its name records, so it is not removed, and deleting it by hand may make that install fail where it gives the link its name"
        )),
        "the sync names the link it left and what deleting it by hand would risk; {text}"
    );
}

// What:
// `#[test] fn a_sync_keeps_and_names_the_earlier_builds_temporary_link()`
// plants the name `0.2.7-alpha.18` wrote for a process that has ended,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// A farm written by that release still holds those names,
// and the name records a process number and nothing else:
// no run can say whether the number's holder is the install that wrote it,
// or whether that install was ever in this process namespace,
// so the link stays and the report names it
// rather than claiming an install has ended.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync keeps and names the earlier builds temporary link', () => { ... });
// ```
#[test]
fn a_sync_keeps_and_names_the_earlier_builds_temporary_link() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(&rig, A_DECLARED_BIN);
    let name: String =
        the_name_the_earlier_build_wrote(A_DECLARED_BIN, pid_of_a_process_that_ended());
    let temporary: PathBuf = rig.scope.exposure_dir.join(&name);
    symlink(&target, &temporary)
        .expect("stand in for the link a killed install of the release left");
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&temporary).is_ok(),
        "the name records one process number and no place, so nothing here can judge it; {text}"
    );
    let start: String = format!("toolchain install: left the temporary link {name}: ");
    assert!(
        the_line_starting_with(&text, &start).is_some(),
        "the sync names the link it left; {text}"
    );
}

// What:
// `#[test] fn a_sync_keeps_and_names_the_earlier_builds_temporary_link_while_its_number_is_held()`
// plants the name `0.2.7-alpha.18` wrote, carrying the number of a process that runs now,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// The same rule covers both states of that name:
// a number some process holds may be the install that wrote the name,
// a number no process holds here may belong to an install in another process namespace,
// and the name cannot tell either from a program that took the number over,
// so the link stays and the report names it.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync keeps and names the earlier builds temporary link while its number is held', () => { ... });
// ```
#[test]
fn a_sync_keeps_and_names_the_earlier_builds_temporary_link_while_its_number_is_held() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(&rig, A_DECLARED_BIN);
    let name: String = the_name_the_earlier_build_wrote(A_DECLARED_BIN, std::process::id());
    let temporary: PathBuf = rig.scope.exposure_dir.join(&name);
    symlink(&target, &temporary)
        .expect("stand in for a link a running install of the release holds");
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&temporary).is_ok(),
        "a process holds this number, so the install that wrote the name may still run; {text}"
    );
    let start: String = format!("toolchain install: left the temporary link {name}: ");
    assert!(
        the_line_starting_with(&text, &start).is_some(),
        "the sync names the link it left; {text}"
    );
}

// What:
// `#[test] fn a_sync_names_each_kept_temporary_link_once_in_name_order_after_the_removals_it_made()`
// plants two names no run can judge and two orphans a run removes,
// runs the whole sync,
// and reads the lines.
// Why:
// The release left these names silent,
// and a report that printed one of them twice,
// or printed the kept names before the removals and in the directory's own order,
// would read differently on two runs of the same folder:
// the count, the name order and the place of the kept lines are what make the report
// the same text every time.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync names each kept temporary link once in name order after the removals it made', () => { ... });
// ```
#[test]
fn a_sync_names_each_kept_temporary_link_once_in_name_order_after_the_removals_it_made() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(&rig, A_DECLARED_BIN);
    // What: the two link names an earlier build's temporary name is built on, so that the name order of the
    //  kept lines is not the order they are planted in.
    // Why: the control has to name an order the report can disagree with.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keptNames = ['zzz-earlier', 'aaa-earlier'];
    // ```
    let kept_link_names: [&str; 2] = ["zzz-earlier", "aaa-earlier"];
    let ended: u32 = pid_of_a_process_that_ended();
    let mut kept_names: Vec<String> = Vec::new();
    for link_name in kept_link_names {
        let name: String = the_name_the_earlier_build_wrote(link_name, ended);
        symlink(&target, rig.scope.exposure_dir.join(&name))
            .expect("stand in for a link a killed install of the release left");
        kept_names.push(name);
    }
    for orphan in [AN_ORPHAN, A_SECOND_ORPHAN] {
        symlink(&target, rig.scope.exposure_dir.join(orphan)).expect("leave an orphan");
    }
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    // What: the report's kept lines, in the order the report holds them.
    // Why: a count and an order are read from the lines themselves, not from the report's fields.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keptLines = text.split('\n').filter((line) => line.startsWith('toolchain install: left the temporary link '));
    // ```
    let kept_lines: Vec<&str> = text
        .lines()
        .filter(|line| {
            return line.starts_with("toolchain install: left the temporary link ");
        })
        .collect();
    assert_eq!(
        kept_lines.len(),
        kept_names.len(),
        "one line for each name kept, and no name twice: {text}"
    );
    // What: the name each kept line carries, read out of the sentence's own prefix and separator.
    // Why: a line naming one name twice would keep the count right and the report wrong.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const said = keptLines.map((line) => line.slice('toolchain install: left the temporary link '.length).split(': this run')[0]);
    // ```
    let said_names: Vec<String> = kept_lines
        .iter()
        .map(|line| {
            let rest: &str = line
                .strip_prefix("toolchain install: left the temporary link ")
                .expect("the line is the kept sentence");
            return rest
                .split_once(": this run")
                .expect("the sentence names the name and then why it stays")
                .0
                .to_string();
        })
        .collect();
    let mut kept_names_in_name_order: Vec<String> = kept_names.clone();
    kept_names_in_name_order.sort();
    assert_eq!(
        said_names, kept_names_in_name_order,
        "the kept lines come in name order, whichever order the directory listed the names in: {text}"
    );
    // What: where the last removal line sits and where the first kept line sits.
    // Why: the report's own order is what a person reads, so a kept line printed before the removals
    //  would move on every run of the same folder.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lastRemoval = lines.findLastIndex((line) => line.startsWith('toolchain install: removed the exposure link '));
    // const firstKept = lines.findIndex((line) => line.startsWith('toolchain install: left the temporary link '));
    // ```
    let lines: Vec<&str> = text.lines().collect();
    let last_removal: usize = lines
        .iter()
        .rposition(|line| {
            return line.starts_with("toolchain install: removed the exposure link ");
        })
        .expect("the two orphans were removed and named");
    let first_kept: usize = lines
        .iter()
        .position(|line| {
            return line.starts_with("toolchain install: left the temporary link ");
        })
        .expect("the two kept names were named");
    assert!(
        last_removal < first_kept,
        "the kept lines follow every removal line, at {first_kept} after the last removal at {last_removal}: {text}"
    );
}

// What:
// `#[test] fn a_sync_removes_a_dotted_name_that_is_no_temporary_link()`
// plants a dotted link whose name is not one a temporary link carries,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// Only a name this tool's own write produces is judged as a held link;
// every other undeclared name is an ordinary orphan,
// and the report has to say that rather than call a person's name a kill's.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync removes a dotted name that is no temporary link', () => { ... });
// ```
#[test]
fn a_sync_removes_a_dotted_name_that_is_no_temporary_link() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(&rig, A_DECLARED_BIN);
    let name: String = format!(".{A_DECLARED_BIN}.not-a-number.tmp");
    let dotted: PathBuf = rig.scope.exposure_dir.join(&name);
    symlink(&target, &dotted).expect("leave a dotted link a person made");
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&dotted).is_err(),
        "the name is no temporary link's, and no row declares it, so it is removed; {text}"
    );
    assert!(
        text.lines().any(|line| {
            return line.starts_with("toolchain install: removed the exposure link")
                && line.contains(&name);
        }),
        "the sync names it as the exposure link it was; {text}"
    );
}

// What:
// `#[test] fn a_sync_keeps_the_farms_mpm_shadow_link_that_resolves_into_an_install()`
// builds a farm holding the program's own link into an installed copy,
// runs the whole sync from a program that belongs to no install,
// and reads the folder.
// Why:
// The release removed that link whenever the running binary was not an install,
// although an installed copy existed and the link resolved,
// so the name came and went with whichever binary ran `install`.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync keeps the farms mpm-shadow link that resolves into an install', () => { ... });
// ```
#[test]
fn a_sync_keeps_the_farms_mpm_shadow_link_that_resolves_into_an_install() {
    let rig: Rig = a_rig_whose_lock_declares_no_shadow();
    let link: PathBuf = the_farm_shadow_link(&rig);
    let target: PathBuf = std::fs::read_link(&link).expect("a symlink");
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert_eq!(
        std::fs::read_link(&link).expect("the farm still holds the program's own link"),
        target,
        "the link resolves into an installed {SHADOW_PROGRAM}, so a rebuild leaves it; {text}"
    );
    assert!(
        !text.contains(&format!("removed the exposure link {SHADOW_PROGRAM}")),
        "the rebuild says nothing about removing the program's own link; {text}"
    );
}

// What:
// `#[test] fn a_partial_sync_keeps_the_link_of_a_row_it_did_not_link()`
// fills a farm from the whole lock,
// then rebuilds it with one row alone,
// as `update <id>` does.
// Why:
// A partial sync installs some rows and links only those,
// and the removal reads the whole lock:
// a name another row declares must stay,
// or `update <id>` would take every other tool off `PATH`.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a partial sync keeps the link of a row it did not link', () => { ... });
// ```
#[test]
fn a_partial_sync_keeps_the_link_of_a_row_it_did_not_link() {
    let rig: Rig = Rig::new();
    let node: PathBuf = rig.install_binary("node", "node");
    write_executable(&rig.scope.toolchain_bin_dir.join("npm"));
    write_executable(&rig.scope.toolchain_bin_dir.join("pnpm"));
    let node_row = rig.tool("node");
    let pnpm_row = rig.tool("pnpm");
    let whole = rebuild_exposure(&rig.context(), &[&node_row, &pnpm_row]);
    assert!(whole.error.is_none(), "the whole lock links");
    let pnpm_link: PathBuf = rig.scope.exposure_dir.join("pnpm");
    assert_eq!(
        std::fs::read_link(&pnpm_link).expect("the whole lock linked pnpm"),
        rig.scope.toolchain_bin_dir.join("pnpm")
    );
    let partial = rebuild_exposure(&rig.context(), &[&node_row]);
    assert!(partial.error.is_none(), "one row's own rebuild");
    assert_eq!(
        std::fs::read_link(&pnpm_link).expect("pnpm's link is another row's and stays"),
        rig.scope.toolchain_bin_dir.join("pnpm"),
        "a partial sync leaves the link of a name the lock declares"
    );
    assert_eq!(
        std::fs::read_link(rig.scope.exposure_dir.join("node")).expect("a symlink"),
        node,
        "the row the partial sync linked is linked to its own program"
    );
}

// What:
// `#[test] fn a_removal_of_a_link_another_run_took_first_is_not_a_failure()`
// plants an orphan link,
// has another run's removal take it in the instant before this run's own,
// and reads the report.
// Why:
// Two installs meeting one orphan made the release fail with
// `cannot remove the exposure link <farm>/orphan: No such file or directory (os error 2)`,
// although the folder was right and a rerun passed;
// the link is gone either way, and the run that took it named it.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a removal of a link another run took first is not a failure', () => { ... });
// ```
#[test]
fn a_removal_of_a_link_another_run_took_first_is_not_a_failure() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let orphan: PathBuf = rig.scope.exposure_dir.join(AN_ORPHAN);
    symlink(a_program_beside_the_farm(&rig, AN_ORPHAN), &orphan).expect("leave an orphan");
    let _plant = planting_at_the_removal_of(AN_ORPHAN, What::AnotherRunRemovesIt);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&orphan).is_err(),
        "the orphan is gone from the folder; {text}"
    );
    assert!(
        !text.contains("exposure directory failed"),
        "the run that found the link gone does not fail for it; {text}"
    );
    assert!(
        !text.contains(&format!("removed the exposure link {AN_ORPHAN}")),
        "the run that took the link named it, so this one does not name it again; {text}"
    );
}

// What:
// `#[test] fn a_run_names_the_removals_it_made_before_a_later_one_failed()`
// plants two orphan links,
// has the second removal refused with the kernel's `EIO`,
// and reads the report.
// Why:
// The release built its sentences from the removals only after the loop,
// so a removal that failed left every removal before it out of the report:
// a link left the folder and no run said which.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a run names the removals it made before a later one failed', () => { ... });
// ```
#[test]
fn a_run_names_the_removals_it_made_before_a_later_one_failed() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    for name in [AN_ORPHAN, A_SECOND_ORPHAN] {
        symlink(
            a_program_beside_the_farm(&rig, AN_ORPHAN),
            rig.scope.exposure_dir.join(name),
        )
        .expect("leave an orphan");
    }
    // What: the two names in the order the directory lists them, which is the order the pass removes them in.
    // Why: the removal that succeeds has to come before the one that fails, and the order is the file system's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const listed = readdirSync(farm).sort();  // the pass's own order is the listing's
    // ```
    let mut listed: Vec<String> = std::fs::read_dir(&rig.scope.exposure_dir)
        .expect("list the farm")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(listed.len(), 2, "the farm holds the two orphans alone");
    // What: the name the pass removes first, and the one whose removal is refused.
    // Why: the control's whole point is a removal that succeeded before a later one failed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [first, second] = listed;
    // ```
    let first: String = listed.remove(0);
    let second: String = listed.remove(0);
    // What: the number the kernel gives `EIO`, which no fixture can make a real `unlink` answer.
    // Why: the refusal must be the one a real disk error carries, so the message reads as one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const eio = Errno.IO.rawOsError();
    // ```
    let eio: i32 = rustix::io::Errno::IO.raw_os_error();
    let _plant = planting_at_the_removal_of(
        &second,
        What::Refused {
            code: eio,
        },
    );
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(rig.scope.exposure_dir.join(&first)).is_err(),
        "the removal of {first} was made before the removal of {second} failed; {text}"
    );
    assert!(
        std::fs::symlink_metadata(rig.scope.exposure_dir.join(&second)).is_ok(),
        "the removal of {second} failed, so its link stays; {text}"
    );
    assert!(
        text.lines().any(|line| {
            return line.starts_with("toolchain install: removed the exposure link")
                && line.contains(&first);
        }),
        "the run names the removal it made before the failure; {text}"
    );
    assert!(
        text.lines().any(|line| {
            return line.contains("exposure directory failed") && line.contains(&second);
        }),
        "the run names the removal that failed; {text}"
    );
}

// What:
// `#[test] fn a_link_another_run_took_before_this_run_read_it_is_not_a_failure()`
// plants an orphan link,
// has another run's removal take it in the instant before this run reads the entry's own metadata,
// and reads the report.
// Why:
// The rebuild lists the folder and reads each entry after that,
// so this is the state of an install meeting another one's removal between those two steps;
// the name is gone either way,
// and a run that reports `1 failed` for it sends a person to look at a folder that is right.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a link another run took before this run read it is not a failure', () => { ... });
// ```
#[test]
fn a_link_another_run_took_before_this_run_read_it_is_not_a_failure() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let orphan: PathBuf = rig.scope.exposure_dir.join(AN_ORPHAN);
    symlink(a_program_beside_the_farm(&rig, AN_ORPHAN), &orphan).expect("leave an orphan");
    let _plant = planting_at_the_removal_of(AN_ORPHAN, What::AnotherRunRemovesItBeforeTheLinkIsRead);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&orphan).is_err(),
        "the orphan is gone from the folder; {text}"
    );
    assert!(
        !text.contains("exposure directory failed"),
        "a name another run took before this run read it is no failure; {text}"
    );
    assert!(
        !text.contains(&format!("removed the exposure link {AN_ORPHAN}")),
        "the run that took the link named it, so this one does not name it again; {text}"
    );
}

// What:
// `#[test] fn a_link_another_run_took_before_this_run_read_its_target_is_not_a_failure()`
// plants an orphan link,
// has another run's removal take it in the instant before this run reads the link's target,
// and reads the report.
// Why:
// The metadata read and the target read are two calls,
// so a name that was there for the first can be gone for the second;
// that gap is the second of the three calls the removal's claim has to cover.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a link another run took before this run read its target is not a failure', () => { ... });
// ```
#[test]
fn a_link_another_run_took_before_this_run_read_its_target_is_not_a_failure() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let orphan: PathBuf = rig.scope.exposure_dir.join(AN_ORPHAN);
    symlink(a_program_beside_the_farm(&rig, AN_ORPHAN), &orphan).expect("leave an orphan");
    let _plant = planting_at_the_removal_of(
        AN_ORPHAN,
        What::AnotherRunRemovesItBeforeTheTargetIsRead,
    );
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&orphan).is_err(),
        "the orphan is gone from the folder; {text}"
    );
    assert!(
        !text.contains("exposure directory failed"),
        "a name another run took before this run read its target is no failure; {text}"
    );
    assert!(
        !text.contains(&format!("removed the exposure link {AN_ORPHAN}")),
        "the run that took the link named it, so this one does not name it again; {text}"
    );
}

// What:
// `#[test] fn a_sync_says_of_a_removed_temporary_link_only_what_it_could_show()`
// plants a temporary link named after an install that has ended,
// runs the whole sync,
// and reads the sentence it printed.
// Why:
// The name has the shape an install writes,
// and a person's own dotted link can have the same shape;
// nothing in the folder tells the two apart,
// so the line says what the name is and what the run could show of the process it records,
// and never that an install wrote it.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync says of a removed temporary link only what it could show', () => { ... });
// ```
#[test]
fn a_sync_says_of_a_removed_temporary_link_only_what_it_could_show() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(&rig, A_DECLARED_BIN);
    let temporary: PathBuf = the_temporary_link_of(
        &rig.scope.exposure_dir.join(A_DECLARED_BIN),
        the_owner_of_an_install_that_ended(),
    );
    symlink(&target, &temporary).expect("stand in for the link a killed install left");
    let name: String = temporary
        .file_name()
        .expect("a file name")
        .to_string_lossy()
        .to_string();
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert_eq!(
        the_line_starting_with(&text, "toolchain install: removed the temporary link"),
        Some(format!(
            "toolchain install: removed the temporary link {name} (it pointed at {}), a name of the shape an install writes while it holds a link, and the process that name records is not running",
            target.display()
        )),
        "the sentence says what the name is and what the run could show; {text}"
    );
}

// What:
// `#[test] fn a_sync_keeps_and_names_a_temporary_link_an_install_in_another_process_namespace_holds()`
// plants a temporary link whose name records a process namespace that is not this one,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// A link this run cannot judge may belong to an install that still runs,
// and leaving it silently kept a name in a folder a person reads with nothing said about it:
// the report names it once and says what deleting it by hand would risk.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync keeps and names a temporary link an install in another process namespace holds', () => { ... });
// ```
#[test]
fn a_sync_keeps_and_names_a_temporary_link_an_install_in_another_process_namespace_holds() {
    let rig: Rig = a_rig_with_no_tools();
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(&rig, A_DECLARED_BIN);
    let held: Owner = the_owner_of_an_install_that_ended();
    let temporary: PathBuf = the_temporary_link_of(
        &rig.scope.exposure_dir.join(A_DECLARED_BIN),
        Owner {
            pid: held.pid,
            started: held.started,
            boot: held.boot,
            namespace: Some(ANOTHER_PROCESS_NAMESPACE),
        },
    );
    symlink(&target, &temporary).expect("stand in for a link another namespace's install holds");
    let name: String = temporary
        .file_name()
        .expect("a file name")
        .to_string_lossy()
        .to_string();
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&temporary).is_ok(),
        "the number means nothing in this namespace, so the sync leaves the link; {text}"
    );
    let start: String = format!("toolchain install: left the temporary link {name}: ");
    assert!(
        the_line_starting_with(&text, &start).is_some(),
        "the sync names the link it left and why; {text}"
    );
}

// What:
// `#[test] fn a_sync_removes_and_names_the_farms_mpm_shadow_link_into_another_tools_install()`
// builds a farm whose `mpm-shadow` link resolves to a file of that name inside another tool's install directory,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// The name is the scope's way to the installed program,
// and an install of another tool is not that program's install;
// the link is a name no row declares, and the release kept it.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync removes and names the farms mpm-shadow link into another tools install', () => { ... });
// ```
#[test]
fn a_sync_removes_and_names_the_farms_mpm_shadow_link_into_another_tools_install() {
    let rig: Rig = a_rig_whose_lock_declares_no_shadow();
    let program: PathBuf = an_install_of_another_tool(&rig);
    let link: PathBuf = the_farm_link_to(&rig, &program);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&link).is_err(),
        "another tool's install is not an install of {SHADOW_PROGRAM}, so the link goes; {text}"
    );
    assert!(
        text.contains(&format!("removed the exposure link {SHADOW_PROGRAM}")),
        "the sync names the link it removed; {text}"
    );
}

// What:
// `#[test] fn a_sync_removes_and_names_the_farms_mpm_shadow_link_that_resolves_nowhere()`
// builds a farm whose `mpm-shadow` link points at a path nothing stands at,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// A scope that has the program installed nowhere removes the name:
// with nothing there it reaches nothing,
// and the README says so.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync removes and names the farms mpm-shadow link that resolves nowhere', () => { ... });
// ```
#[test]
fn a_sync_removes_and_names_the_farms_mpm_shadow_link_that_resolves_nowhere() {
    let rig: Rig = a_rig_whose_lock_declares_no_shadow();
    let gone: PathBuf = rig
        .scope
        .toolchain_root
        .join(SHADOW_PROGRAM)
        .join(AN_INSTALLED_VERSION)
        .join(SHADOW_PROGRAM);
    let link: PathBuf = the_farm_link_to(&rig, &gone);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&link).is_err(),
        "the link resolves nowhere, so the sync removes it; {text}"
    );
    assert!(
        text.contains(&format!("removed the exposure link {SHADOW_PROGRAM}")),
        "the sync names the link it removed; {text}"
    );
}

// What:
// `#[test] fn a_sync_removes_and_names_the_farms_mpm_shadow_link_to_a_program_that_belongs_to_no_install()`
// builds a farm whose `mpm-shadow` link resolves to a file of that name in a build folder,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// A build folder is no install,
// so the name reaches no installed program and is a name no row declares.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync removes and names the farms mpm-shadow link to a program that belongs to no install', () => { ... });
// ```
#[test]
fn a_sync_removes_and_names_the_farms_mpm_shadow_link_to_a_program_that_belongs_to_no_install() {
    let rig: Rig = a_rig_whose_lock_declares_no_shadow();
    let built: PathBuf = rig.scope.root.join("build").join(SHADOW_PROGRAM);
    write_executable(&built);
    let link: PathBuf = the_farm_link_to(&rig, &built);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&link).is_err(),
        "a program that belongs to no install is not the farm's program, so the link goes; {text}"
    );
    assert!(
        text.contains(&format!("removed the exposure link {SHADOW_PROGRAM}")),
        "the sync names the link it removed; {text}"
    );
}

// What:
// `#[test] fn a_sync_keeps_the_farms_mpm_shadow_link_that_resolves_into_the_other_toolchain_roots_install()`
// builds the installed copy of the program in the machine-global root,
// links the farm's own name to it,
// runs the whole sync,
// and reads the folder.
// Why:
// Either toolchain root's install of the program is one the name may reach:
// the two roots are one set for this repository's tools,
// and a rebuild must not take the name away from the root that has the program.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync keeps the farms mpm-shadow link that resolves into the other toolchain roots install', () => { ... });
// ```
#[test]
fn a_sync_keeps_the_farms_mpm_shadow_link_that_resolves_into_the_other_toolchain_roots_install() {
    let rig: Rig = a_rig_whose_lock_declares_no_shadow();
    let install: PathBuf = rig.scope.other_toolchain_root.join(SHADOW_PROGRAM);
    let version: PathBuf = install.join(AN_INSTALLED_VERSION);
    let program: PathBuf = version.join(SHADOW_PROGRAM);
    write_executable(&program);
    symlink(AN_INSTALLED_VERSION, install.join("current")).expect("link current");
    let link: PathBuf = the_farm_link_to(&rig, &install.join("current").join(SHADOW_PROGRAM));
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert_eq!(
        std::fs::read_link(&link).expect("the farm still holds the program's own link"),
        install.join("current").join(SHADOW_PROGRAM),
        "the other root's install is an install of the program, so the link stays; {text}"
    );
}

// What:
// `#[test] fn a_sync_keeps_the_farms_mpm_shadow_link_that_resolves_into_the_cargo_routes_install()`
// builds the installed copy of the program where a cargo-route install puts it,
// links the farm's own name to it,
// runs the whole sync,
// and reads the folder.
// Why:
// A machine bootstraps this tool with `cargo install` as well as with a release archive,
// and that install's directory is named after the crate:
// the rule has to keep the name for both ways of installing the program.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a sync keeps the farms mpm-shadow link that resolves into the cargo routes install', () => { ... });
// ```
#[test]
fn a_sync_keeps_the_farms_mpm_shadow_link_that_resolves_into_the_cargo_routes_install() {
    let rig: Rig = a_rig_whose_lock_declares_no_shadow();
    let program: PathBuf = a_cargo_install_of_the_shadow(&rig);
    let install: PathBuf = rig
        .scope
        .toolchain_root
        .join("cargo")
        .join(THE_PROGRAMS_CRATE);
    let link: PathBuf = the_farm_link_to(&rig, &install.join("current").join("bin").join(SHADOW_PROGRAM));
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert_eq!(
        std::fs::read_link(&link).expect("the farm still holds the program's own link"),
        install.join("current").join("bin").join(SHADOW_PROGRAM),
        "a cargo install of the program is an install of it, so the link stays; {text}"
    );
    assert!(
        std::fs::canonicalize(&link).expect("the link resolves")
            == std::fs::canonicalize(&program).expect("the program is there"),
        "the link reaches the program a cargo route installed; {text}"
    );
}
