//! What:
//! Unit tests of the farm link that makes the running `mpm-shadow` reachable under its own name.
//! Why:
//! Every sentence `mpm-shadow` prints names `mpm-shadow` as the program to run,
//! and a shell outside a checkout reaches only the scope's exposure farm,
//! so the farm must hold that name or the printed command cannot run as printed.

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::os::unix::fs`.
use std::os::unix::fs::symlink;
/// Imports what this module uses from `std::path`.
use std::path::PathBuf;

/// Imports `current_link` from `super::cargo_installs`.
use super::cargo_installs::current_link;
/// Imports `rebuild_exposure`,
/// `expose_running_shadow` from `super::exposure`.
use super::exposure::{
    expose_running_shadow,
    rebuild_exposure,
};
/// Imports what this module uses from `super::fakes`.
use super::fakes::{
    Rig,
    write_executable,
};
/// Imports `parse_record`,
/// `record_path` from `super::launcher`.
use super::launcher::{
    parse_record,
    record_path,
};

/// The id the sample plan gives `mpm-shadow`.
const SHADOW_TOOL_ID: &str = "cargo:monochromatic-mpm-shadow";

/// The crate the sample plan locks for `mpm-shadow`.
const SHADOW_CRATE: &str = "monochromatic-mpm-shadow";

/// The version the fixture has installed,
/// which no lock in the fixture pins.
const INSTALLED: &str = "0.2.0";

/// The name every sentence `mpm-shadow` prints gives the program,
/// and the name the farm must carry.
const SHADOW_PROGRAM: &str = "mpm-shadow";

/// What:
/// Puts a finished `cargo-binstall` install of `mpm-shadow` at `INSTALLED` in the rig's own toolchain root,
/// and hands back the program inside it,
/// the path a run of that install has as its own.
/// Why:
/// The farm link names an installed program,
/// so the fixture must be one:
/// a version directory with cargo's record,
/// the program below `bin`,
/// and the tool's `current` link naming the version.
fn installed_shadow(rig: &Rig) -> PathBuf {
    let version_dir: PathBuf = rig
        .scope
        .toolchain_root
        .join("cargo")
        .join(SHADOW_CRATE)
        .join(INSTALLED);
    let program: PathBuf = version_dir.join("bin").join(SHADOW_PROGRAM);
    write_executable(&program);
    fs::write(
        version_dir.join(".crates.toml"),
        format!("[v1]\n\"{SHADOW_CRATE} {INSTALLED} (registry+https://example.invalid/index)\" = [\"{SHADOW_PROGRAM}\"]\n"),
    )
    .expect("write cargo's record");
    symlink(
        INSTALLED,
        current_link(&rig.scope.toolchain_root, SHADOW_CRATE),
    )
    .expect("link current");
    return program;
}

/// A program of the same name that belongs to no install:
/// a build folder has no `current` link above it.
fn uninstalled_shadow(rig: &Rig) -> PathBuf {
    let program: PathBuf = rig.scope.root.join("build").join(SHADOW_PROGRAM);
    write_executable(&program);
    return program;
}

#[test]
fn the_farm_links_the_running_shadow_under_its_own_name() {
    let mut rig = Rig::new();
    rig.lock
        .tools
        .retain(|tool| return tool.declaration.id != SHADOW_TOOL_ID);
    let program: PathBuf = installed_shadow(&rig);

    let linked: Option<String> =
        expose_running_shadow(&rig.context(), &program).expect("the farm is writable");

    assert_eq!(linked, Some(SHADOW_PROGRAM.to_string()));
    let link: PathBuf = rig.scope.exposure_dir.join(SHADOW_PROGRAM);
    assert_eq!(
        fs::read_link(&link).expect("a symlink"),
        current_link(&rig.scope.toolchain_root, SHADOW_CRATE)
            .join("bin")
            .join(SHADOW_PROGRAM),
        "the link names the program through the tool's `current` link, so a later version move never leaves it dangling"
    );
    assert_eq!(
        fs::canonicalize(&link).expect("the link resolves"),
        fs::canonicalize(&program).expect("the installed program exists"),
        "a shell that types `mpm-shadow` runs the installed program"
    );
}

#[test]
fn a_shadow_that_belongs_to_no_install_links_nothing() {
    let mut rig = Rig::new();
    rig.lock
        .tools
        .retain(|tool| return tool.declaration.id != SHADOW_TOOL_ID);

    let linked: Option<String> =
        expose_running_shadow(&rig.context(), &uninstalled_shadow(&rig)).expect("nothing to do");

    assert_eq!(
        linked, None,
        "a build folder is not an install, so it never goes on the machine's PATH"
    );
    assert!(
        !rig.scope.exposure_dir.join(SHADOW_PROGRAM).exists(),
        "the farm holds no link for it"
    );
}

#[test]
fn a_plan_that_declares_the_name_keeps_its_own_link() {
    let rig = Rig::new();
    let program: PathBuf = installed_shadow(&rig);

    let linked: Option<String> =
        expose_running_shadow(&rig.context(), &program).expect("nothing to do");

    assert_eq!(
        linked, None,
        "the plan's own row owns the farm link for the bin it declares"
    );
    let tool = rig.tool(SHADOW_TOOL_ID);
    rebuild_exposure(&rig.context(), &[&tool]).expect("rebuilt");
    assert_eq!(
        fs::read_link(rig.scope.exposure_dir.join(SHADOW_PROGRAM)).expect("a symlink"),
        rig.scope.launcher_dir.join(SHADOW_PROGRAM),
        "a declared bin goes through this tree's launcher, so its record warns about that row"
    );
    let record = parse_record(
        &fs::read_to_string(record_path(&rig.scope.launcher_dir.join(SHADOW_PROGRAM)))
            .expect("a record"),
    )
    .expect("it parses");
    assert_eq!(
        record.tool_id, SHADOW_TOOL_ID,
        "the record names the row the farm link belongs to"
    );
}

// What:
// `const THE_LINK_A_ROW_NO_LONGER_DECLARES: &str = "cargo-miri"` is the name of a program rustup's
// `miri` component installs,
/// which no row of the sample plan declares.
// Its string sibling `String` is an owned buffer,
// which a constant never needs.
// Why:
// The farm keeps a name an earlier row declared,
// and the state to build is one link for a name no row of the lock declares.
// Gotcha:
// A name handed to a test this way stands for rustup's own spelling of that program;
// the fixture's program is written here and is not rustup's.
//
// In TS you'd write (pseudocode):
// ```ts
// const theLinkARowNoLongerDeclares = 'cargo-miri';
// ```
const THE_LINK_A_ROW_NO_LONGER_DECLARES: &str = "cargo-miri";

// What:
// `const THE_LINK_THAT_RESOLVES_NOWHERE: &str = "miri"` is a second name no row of the sample plan
// declares,
// whose link points at a program that is not on the disk.
// Its string sibling `String` is an owned buffer,
// which a constant never needs.
// Why:
// A name whose target went away is the state issue 6634's Low point 3 describes:
// the link dangles once the program behind it does,
// and a removal that only follows resolving links would leave it.
//
// In TS you'd write (pseudocode):
// ```ts
// const theLinkThatResolvesNowhere = 'miri';
// ```
const THE_LINK_THAT_RESOLVES_NOWHERE: &str = "miri";

// What:
// `const A_FILE_A_PERSON_PUT_IN_THE_FARM: &str = "my-rust"` is a regular file in the exposure
// directory whose name no row declares.
// Its string sibling `String` is an owned buffer,
// which a constant never needs.
// Why:
// The farm is a directory on a person's machine,
// so the removal is about the links the tool made and never about a file somebody put there.
//
// In TS you'd write (pseudocode):
// ```ts
// const aFileAPersonPutInTheFarm = 'my-rust';
// ```
const A_FILE_A_PERSON_PUT_IN_THE_FARM: &str = "my-rust";

// What:
// `fn whether_a_row_declares(rig: &Rig, name: &str) -> bool` says whether any row of the rig's lock
// declares `name` as a bin.
// Why:
// The three entries below are about names no row declares,
// and a name the sample plan declares would make the control pass for another reason.
//
// In TS you'd write (pseudocode):
// ```ts
// function whetherARowDeclares(rig: Rig, name: string): boolean
// ```
fn whether_a_row_declares(
    rig: &Rig,
    name: &str,
) -> bool {
    // What:
    // `.iter().any(...)` walks the lock's rows and answers whether one holds the name.
    // Why:
    // One pass over a small list, and no allocation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return rig.lock.tools.some((tool) => tool.declaration.bins.includes(name));
    // ```
    return rig
        .lock
        .tools
        .iter()
        .any(|tool| return tool.declaration.bins.iter().any(|bin| return bin == name));
}

// What:
// `#[test] fn a_rebuild_removes_the_links_no_row_declares_and_leaves_a_persons_file()`
// builds a farm holding a link for a name no row declares whose program is there,
// a second such link whose program is gone,
// and a regular file of such a name,
// then rebuilds the farm and reads what is left.
// Why:
// Issue 6634's Low point 3:
// a machine that installed with `0.2.7-alpha.16`'s row keeps a `rustup` link in the farm,
// which dangles once rustup's proxies go,
// and `rebuild_exposure` only made links.
// The farm holds what the lock declares:
// a symbolic link for a name no row declares is removed,
// and a file is a person's.
//
// In TS you'd write (pseudocode):
// ```ts
// test('a rebuild removes the links no row declares and leaves a persons file', () => { ... });
// ```
#[test]
fn a_rebuild_removes_the_links_no_row_declares_and_leaves_a_persons_file() {
    let rig: Rig = Rig::new();
    fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    // What:
    // the program the first link points at: a stand-in written beside the farm, not a real tool.
    // Why:
    // A link whose target is there is the ordinary state of a name an earlier row declared.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = pathJoin(rig.scope.root, 'build', theLinkARowNoLongerDeclares);
    // writeExecutable(program);
    // ```
    let program: PathBuf = rig
        .scope
        .root
        .join("build")
        .join(THE_LINK_A_ROW_NO_LONGER_DECLARES);
    write_executable(&program);
    assert!(
        !whether_a_row_declares(&rig, THE_LINK_A_ROW_NO_LONGER_DECLARES),
        "no row of the sample lock declares {THE_LINK_A_ROW_NO_LONGER_DECLARES}"
    );
    assert!(
        !whether_a_row_declares(&rig, THE_LINK_THAT_RESOLVES_NOWHERE),
        "no row of the sample lock declares {THE_LINK_THAT_RESOLVES_NOWHERE}"
    );
    assert!(
        !whether_a_row_declares(&rig, A_FILE_A_PERSON_PUT_IN_THE_FARM),
        "no row of the sample lock declares {A_FILE_A_PERSON_PUT_IN_THE_FARM}"
    );
    symlink(&program, rig.scope.exposure_dir.join(THE_LINK_A_ROW_NO_LONGER_DECLARES))
        .expect("link the program under a name no row declares");
    // What:
    // the second link, pointing at a program that is not on the disk.
    // Why:
    // That is the dangling state the finding is about.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // symlinkSync(pathJoin(rig.scope.root, 'gone', theLinkThatResolvesNowhere), pathJoin(rig.scope.exposureDir, theLinkThatResolvesNowhere));
    // ```
    symlink(
        rig.scope.root.join("gone").join(THE_LINK_THAT_RESOLVES_NOWHERE),
        rig.scope.exposure_dir.join(THE_LINK_THAT_RESOLVES_NOWHERE),
    )
    .expect("link a name whose program is gone");
    // What:
    // a regular file of a name no row declares, holding text.
    // Why:
    // The rebuild has to leave what a person put there,
    // and the text read back proves it was neither removed nor rewritten.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeFileSync(pathJoin(rig.scope.exposureDir, aFileAPersonPutInTheFarm), "a person's file\n");
    // ```
    let the_file: PathBuf = rig.scope.exposure_dir.join(A_FILE_A_PERSON_PUT_IN_THE_FARM);
    fs::write(&the_file, "a person's file\n").expect("write a person's file into the farm");
    rebuild_exposure(&rig.context(), &[]).expect("rebuild the farm");
    // What:
    // the two links, which must be gone: `symlink_metadata` reads the entry itself and never follows
    // the link, so a dangling link still answers whether it is there.
    // Why:
    // `Path::exists` follows the link and is false for a dangling one,
    // so it cannot tell a removed link from a dangling one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(lstatSync(pathJoin(rig.scope.exposureDir, theLinkARowNoLongerDeclares), { throwIfNoEntry: false })).toBeUndefined();
    // ```
    for name in [THE_LINK_A_ROW_NO_LONGER_DECLARES, THE_LINK_THAT_RESOLVES_NOWHERE] {
        assert!(
            fs::symlink_metadata(rig.scope.exposure_dir.join(name)).is_err(),
            "the farm holds no {name}, because no row of the lock declares it"
        );
    }
    // What:
    // the regular file's own content, read back.
    // Why:
    // A file of a name no row declares belongs to a person, and the rebuild leaves it untouched.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(readFileSync(theFile, 'utf8')).toBe("a person's file\n");
    // ```
    assert_eq!(
        fs::read_to_string(&the_file).expect("a person's file stays where it is"),
        "a person's file\n",
        "the rebuild leaves a regular file of a name no row declares"
    );
}
