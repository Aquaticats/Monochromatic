//! What:
//! The controls of what a rebuild of the exposure farm removes,
//! spares and names:
//! the temporary link of an install that still runs,
//! the temporary link an install that ended left,
//! the farm's own `mpm-shadow` link,
//! a declared name a partial sync does not link,
//! a link another run removed first,
//! and the removals a run names before a later one fails.
//! Why:
//! The rebuild is the one thing in this tool that deletes from a folder a person sees,
//! and every state below is one the released `0.2.7-alpha.18` got wrong or never named:
//! it removed another install's temporary link (its rename then failed with `ENOENT`),
//! it removed the farm's `mpm-shadow` link although an installed copy resolved,
//! and a removal that failed after earlier ones left those unnamed.
//! Every name and line asserted here is the one a run prints,
//! and three states stand in for the ones no disposable fixture can reach on this machine:
//! the temporary name `the_name_the_release_writes` builds is the one the shipped rebuild wrote
//! (`.<name>.<pid>.tmp`),
//! the number `pid_of_a_process_that_ended` answers is a real process the kernel no longer lists,
//! so the name is the one a killed install left,
//! and `the_farm_shadow_link` writes an installed copy by hand,
//! below the toolchain root with a `current` link,
//! which is the layout an install of it leaves.
//! A stand-in says so where it is written,
//! and no claim about a real program rests on one.

/// Imports `PathBuf` from `std::path`,
/// which is the owned path type its sibling `Path` borrows.
use std::path::PathBuf;

/// Imports `symlink` from `std::os::unix::fs`,
/// which makes a symbolic link where the tool's own writes make one.
use std::os::unix::fs::symlink;

/// Imports the disposable scope, the lock and the fakes a control runs against.
use super::fakes::{
    Rig,
    write_executable,
};
/// Imports the rebuild whose removal pass these controls are about.
use super::exposure::rebuild_exposure;
/// Imports the plants a control makes at one removal.
use super::exposure::{
    What,
    planting_at_the_removal_of,
};
/// Imports the program of a process the kernel no longer lists,
/// which is what a killed run's name carries.
use super::fixtures::pid_of_a_process_that_ended;
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
/// which is the name a held install is between its two steps for.
const A_DECLARED_BIN: &str = "thirdbin";

/// The name of a link no row declares,
/// which an earlier install left in the farm.
const AN_ORPHAN: &str = "orphan";

/// The name of a second link no row declares,
/// which the removal that fails is about.
const A_SECOND_ORPHAN: &str = "orphan-d";

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
/// and the removal must judge the link, never whether its target lives.
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
/// The name a temporary link of `name` carries in the released `0.2.7-alpha.18`,
/// `.<name>.<pid>.tmp`,
/// with `pid` read from the end.
/// Why:
/// That is the name the shipped rebuild wrote before this round,
/// so a control can plant the state a killed or a held install left;
/// `a_stand_in_for_the_state` says what stands in for what.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theNameTheReleaseWrites(linkName: string, pid: number): string
/// ```
fn the_name_the_release_writes(
    link_name: &str,
    pid: u32,
) -> String {
    return format!(".{link_name}.{pid}.tmp");
}

/// What:
/// A farm holding one link for `A_DECLARED_BIN` whose installer stands between the two steps of its write,
/// and the name that link has while the installer holds it.
/// Why:
/// A second rebuild that runs in that instant is what the released release failed:
/// it removed the temporary link, and the installer's rename then had no name to move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aFarmWithAHeldInstall(rig: Rig): [PathBuf, PathBuf]
/// ```
fn a_farm_with_a_held_install(rig: &Rig) -> (PathBuf, PathBuf) {
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let target: PathBuf = a_program_beside_the_farm(rig, A_DECLARED_BIN);
    let final_name: PathBuf = rig.scope.exposure_dir.join(A_DECLARED_BIN);
    // What: the dotted name the installer made, which no row declares.
    // Why: it is the state the installer holds between its `symlink` and its `rename`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const temporary = pathJoin(farm, theNameTheReleaseWrites(A_DECLARED_BIN, process.pid));
    // ```
    let temporary: PathBuf = rig
        .scope
        .exposure_dir
        .join(the_name_the_release_writes(
            A_DECLARED_BIN,
            std::process::id(),
        ));
    symlink(&target, &temporary).expect("stand in for the installer's held link");
    return (temporary, final_name);
}

/// What:
/// The farm's `mpm-shadow` link, pointing at an installed copy through that install's `current` link.
/// Why:
/// That is the link every run by an installed copy makes,
/// and the one a shell outside a checkout reaches the program through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aFarmWithTheInstalledShadowLink(rig: Rig): PathBuf
/// ```
fn the_farm_shadow_link(rig: &Rig) -> PathBuf {
    std::fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let install: PathBuf = rig.scope.toolchain_root.join(SHADOW_PROGRAM);
    let version: PathBuf = install.join(AN_INSTALLED_VERSION);
    let program: PathBuf = version.join(SHADOW_PROGRAM);
    write_executable(&program);
    symlink(AN_INSTALLED_VERSION, install.join("current")).expect("link current");
    let link: PathBuf = rig.scope.exposure_dir.join(SHADOW_PROGRAM);
    symlink(
        install.join("current").join(SHADOW_PROGRAM),
        &link,
    )
    .expect("link the installed program under the farm's own name");
    return link;
}

/// What:
/// A rig whose lock declares no `mpm-shadow` row,
/// so the farm's own link for the program is a name no row declares.
/// Why:
/// The row would own the name, and the rebuild would take it for the plan's rather than the program's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aRigWhoseLockDeclaresNoShadow(): Rig
/// ```
fn a_rig_whose_lock_declares_no_shadow() -> Rig {
    let rig: Rig = a_rig_with_no_tools();
    assert!(
        !rig
            .lock
            .tools
            .iter()
            .any(|tool| return tool.declaration.id == SHADOW_TOOL_ID),
        "the lock declares no {SHADOW_TOOL_ID}"
    );
    return rig;
}

// What:
// `#[test] fn a_sync_spares_the_temporary_link_of_an_install_that_still_runs()`
// holds a farm in the state an install stands in while its write is between its two steps,
// runs the whole sync a second install is,
// and then gives the link its final name as the held install would.
// Why:
// The released release removed that temporary link,
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
    let (temporary, final_name) = a_farm_with_a_held_install(&rig);
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&temporary).is_ok(),
        "the temporary link belongs to an install that still runs, so the sync leaves it; {text}"
    );
    assert!(
        !text.contains(&the_name_the_release_writes(
            A_DECLARED_BIN,
            std::process::id()
        )),
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
        a_program_beside_the_farm(&rig, A_DECLARED_BIN),
        "the link a person's `PATH` now reaches names the program the install found"
    );
}

// What:
// `#[test] fn a_sync_removes_and_names_the_temporary_link_of_an_install_that_ended()`
// plants a temporary link named after a process the kernel no longer lists,
// runs the whole sync,
// and reads the folder and the printed lines.
// Why:
// The release added this removal and it must stay:
// a kill between a link's creation and its rename would otherwise leave the dotted name for ever,
// and a person must be told which name left the folder.
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
    let killed: String = the_name_the_release_writes(A_DECLARED_BIN, pid_of_a_process_that_ended());
    let temporary: PathBuf = rig.scope.exposure_dir.join(&killed);
    symlink(&target, &temporary).expect("stand in for the link a killed install left");
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(&temporary).is_err(),
        "an install that ended left this temporary link, so the sync removes it; {text}"
    );
    assert!(
        text.lines().any(|line| {
            return line.starts_with("toolchain install: removed the temporary link")
                && line.contains(&killed);
        }),
        "the sync names the temporary link it removed and says what it was; {text}"
    );
}

// What:
// `#[test] fn a_sync_keeps_the_farms_mpm_shadow_link_that_resolves_into_an_install()`
// builds a farm holding the program's own link into an installed copy,
// runs the whole sync from a program that belongs to no install,
// and reads the folder.
// Why:
// The released release removed that link whenever the running binary was not an install,
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
    rebuild_exposure(&rig.context(), &[&node_row, &pnpm_row]).expect("the whole lock links");
    let pnpm_link: PathBuf = rig.scope.exposure_dir.join("pnpm");
    assert_eq!(
        std::fs::read_link(&pnpm_link).expect("the whole lock linked pnpm"),
        rig.scope.toolchain_bin_dir.join("pnpm")
    );
    rebuild_exposure(&rig.context(), &[&node_row]).expect("one row's own rebuild");
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
// Two installs meeting one orphan made the released release fail with
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
    symlink(&a_program_beside_the_farm(&rig, AN_ORPHAN), &orphan).expect("leave an orphan");
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
// The released release built its sentences from the removals only after the loop,
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
            &a_program_beside_the_farm(&rig, AN_ORPHAN),
            &rig.scope.exposure_dir.join(name),
        )
        .expect("leave an orphan");
    }
    // What: the number the kernel gives `EIO`, which no fixture can make a real `unlink` answer.
    // Why: the refusal must be the one a real disk error carries, so the message reads as one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const eio = Errno.IO.rawOsError();
    // ```
    let eio: i32 = rustix::io::Errno::IO.raw_os_error();
    let _plant = planting_at_the_removal_of(A_SECOND_ORPHAN, What::Refused {
        code: eio,
    });
    let report = sync_tools(&rig.context(), None);
    let text: String = format_sync_report(&report);
    assert!(
        std::fs::symlink_metadata(rig.scope.exposure_dir.join(AN_ORPHAN)).is_err(),
        "the first removal was made before the second failed; {text}"
    );
    assert!(
        text.lines().any(|line| {
            return line.starts_with("toolchain install: removed the exposure link")
                && line.contains(AN_ORPHAN);
        }),
        "the run names the removal it made before the failure; {text}"
    );
    assert!(
        text.lines().any(|line| {
            return line.contains("exposure directory failed") && line.contains(A_SECOND_ORPHAN);
        }),
        "the run names the removal that failed; {text}"
    );
}
