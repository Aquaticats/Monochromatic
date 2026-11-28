//! What:
//!  The state plan step 21's dependency arms prepare in the checkout:
//!  the packed dependency the pnpm arm installs,
//!  the Cargo workspace whose lock disagrees with its manifest,
//!  and the launcher record that names another checkout's lock.
//! Why:
//!  Each arm's subject is what the shipped repair does to one kind of state,
//!  so the state is made in one place and the arms,
//!  which are `binary_zfs_step_21_pnpm_and_cargo_arms.rs`'s and
//!  `binary_zfs_step_21_launcher_arm.rs`'s,
//!  only run the adds and read what came of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! theNodeProject(fixture); theCargoWorkspace(fixture); theLauncherState(fixture);
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_guest_fixture::Fixture;
use std::path::{
    Path,
    PathBuf,
};

/// The pnpm the guest driver installed,
/// which the repair finds on the fixture's search path.
pub(super) const PNPM: &str = "/usr/local/bin/pnpm";

/// The Node the pnpm arm resolves one dependency through,
/// which is what a relink that did not happen fails at.
pub(super) const NODE: &str = "/usr/bin/node";

/// The guest's Cargo,
/// which the repair finds on the fixture's search path.
pub(super) const CARGO: &str = "/root/.cargo/bin/cargo";

/// The tar a control round extracts,
/// taken once from the checkout's own `node_modules`,
/// so both arms of a round start from the same bytes.
pub(super) const PRISTINE_MODULES: &str = "/root/step21-node-modules.tar";

/// Where the fixture writes the argument lines of every stand-in `mpm-shadow`
/// start.
pub(super) const MPM_START_LOG: &str = "/root/step21-mpm-start.log";

/// The folder a lock of another checkout hangs under,
/// which the launcher arm's record names.
pub(super) const ELSEWHERE: &str = "/root/step21-elsewhere";

/// The Node project the pnpm arm adds a worktree of.
/// Its one dependency is a tarball the fixture makes,
/// because this guest has no package registry the arm may rely on and because pnpm 12 refuses a store it cannot
/// hard link into a project on another file system,
/// which the checkout's dataset and the clone's are.
const PACKAGE: &str = "{\n  \"name\": \"step21-fixture\",\n  \"private\": true,\n  \"version\": \"0.0.0\",\n  \"dependencies\": {\n    \"is-odd-local\": \"file:vendor/is-odd-local-1.0.0.tgz\"\n  }\n}\n";

/// The manifest of the dependency the fixture packs into that tarball.
const VENDOR_MANIFEST: &str =
    "{\"name\":\"is-odd-local\",\"version\":\"1.0.0\",\"main\":\"index.js\"}\n";

/// The one source file of that dependency.
const VENDOR_SOURCE: &str =
    "module.exports = function isOdd(n) {\n  return Math.abs(n % 2) === 1;\n};\n";

/// The folder inside the project the tarball is packed from and recorded in.
pub(super) const VENDOR: &str = "vendor";

/// The tarball's name inside that folder,
/// which the lockfile names and both arms install from.
pub(super) const VENDOR_TARBALL: &str = "is-odd-local-1.0.0.tgz";

/// Where the packing writes before the tarball is copied into the project,
/// so the archive never sits inside the tree it reads.
pub(super) const VENDOR_PACKED: &str = "/root/step21-is-odd-local.tgz";

/// The one dependency the pnpm and queued arms resolve after their install,
/// so a relink that never ran is a failure and not a silence.
pub(super) const RESOLVED: &str = "is-odd-local";

/// The workspace manifest the Cargo arm's fixture holds.
const WORKSPACE_MANIFEST: &str =
    "[workspace]\nmembers = [\"crates/one\", \"crates/two\"]\nresolver = \"2\"\n";

/// The root member's manifest,
/// depending on its sibling through the workspace.
const ONE_MANIFEST: &str = "[package]\nname = \"one\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ntwo = { path = \"../two\" }\n";

/// The sibling the lock does not name,
/// which is what makes the clean rewrite the lock.
const TWO_MANIFEST: &str = "[package]\nname = \"two\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// The one source file of the root member.
const ONE_SOURCE: &str = "/// One fixture crate.\npub fn one() -> u32 {\n    return 2;\n}\n";

/// The one source file of the sibling.
const TWO_SOURCE: &str =
    "/// The sibling the lock does not name.\npub fn two() -> u32 {\n    return 3;\n}\n";

/// The lock file the toolchain arm declares a launcher for.
const TOOLCHAIN_LOCK: &str =
    "{\"schemaVersion\":1,\"tools\":{\"step21:own\":{\"bins\":[\"own\"]}}}\n";

/// The launcher name the toolchain arm's lock declares.
pub(super) const LAUNCHER: &str = "own";

/// A stand-in `mpm-shadow`,
/// which is no real installer:
/// it records its arguments,
/// prints the summary line a real install prints,
/// and writes the record of the name its own lock declares.
/// It is a stand-in,
/// and nothing here claims what a real `mpm-shadow` does beyond that summary
/// line,
/// which `package_repair_toolchain_tests.rs` copies from a real run.
const MPM_STAND_IN: &str = "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$*\" \"$(pwd -P)\" \"$MONOCHROMATIC_WORKTREE_ROOT\" >> /root/step21-mpm-start.log\necho 'toolchain install: 1 installed, 2 unchanged, 0 failed'\nprintf '{\"toolId\":\"step21:own\",\"lockFile\":\"%s/toolchain.lock.json\",\"real\":\"/tools/own\",\"scope\":\"repo\"}\\n' \"$MONOCHROMATIC_WORKTREE_ROOT\" > \"$MONOCHROMATIC_WORKTREE_ROOT/.cache/toolchain-launchers/own.launch\"\nexit 0\n";

/// What:
///  The node project the pnpm and queued arms prepare:
///  one dependency,
///  its lockfile committed,
///  its installed `node_modules` kept as the tar both arms of a round start
///  from.
/// Why:
///  The repair relinks a `node_modules` the clone carried,
///  so the store is warmed in the checkout and the bytes a control starts from
///  are the same bytes,
///  not a second install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// commit({ 'package.json': manifest }); pnpmInstall(); commit({ 'pnpm-lock.yaml': ... }); tar('node_modules');
/// ```
pub(super) fn the_node_project(fixture: &Fixture) {
    let packaged: PathBuf = fixture.root().join(VENDOR).join("package");
    std::fs::create_dir_all(packaged.as_path()).expect("the vendored package's folder");
    std::fs::write(packaged.join("package.json"), VENDOR_MANIFEST.as_bytes())
        .expect("the vendored package's manifest");
    std::fs::write(packaged.join("index.js"), VENDOR_SOURCE.as_bytes())
        .expect("the vendored package's source");
    fixture.run(
        Path::new("/usr/bin/tar"),
        &[
            "--create",
            "--gzip",
            "--file",
            VENDOR_PACKED,
            "--directory",
            fixture.root().join(VENDOR).to_str().expect("a text path"),
            "package",
        ],
        fixture.root(),
    );
    std::fs::copy(
        Path::new(VENDOR_PACKED),
        fixture.root().join(VENDOR).join(VENDOR_TARBALL),
    )
    .expect("the tarball the lockfile names");
    fixture.commit(
        &[
            ("package.json", PACKAGE),
            ("src/main.txt", "the node project\n"),
        ],
        "the node project",
    );
    // The store is warmed once, in the checkout, so both arms of every round install offline from it.
    fixture.run(Path::new(PNPM), &["install"], fixture.root());
    fixture.git(&["add", "--all"]);
    fixture.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "--no-verify",
        "-m",
        "the lockfile and the tarball it names",
    ]);
    fixture.run(
        Path::new("/usr/bin/tar"),
        &["--create", "--file", PRISTINE_MODULES, "node_modules"],
        fixture.root(),
    );
}

/// What:
///  The Cargo workspace the Cargo and queued arms prepare:
///  a member depending on its sibling,
///  a built build directory the clone's snapshot carries,
///  and a lock the manifest no longer matches.
/// Why:
///  `cargo clean` rewrites a lock that disagrees with its manifest unless it is
///  given `--locked`,
///  and the add's own clean is not,
///  so every round has a lock for the repair to put back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// commit(manifests); cargo build --offline; stripTheSiblingFromTheLock(); commit(lock);
/// ```
pub(super) fn the_cargo_workspace(fixture: &Fixture) -> String {
    fixture.commit(
        &[
            ("Cargo.toml", WORKSPACE_MANIFEST),
            ("crates/one/Cargo.toml", ONE_MANIFEST),
            ("crates/one/src/lib.rs", ONE_SOURCE),
            ("crates/two/Cargo.toml", TWO_MANIFEST),
            ("crates/two/src/lib.rs", TWO_SOURCE),
            ("src/main.txt", "the cargo workspace\n"),
        ],
        "the cargo workspace",
    );
    fixture.run(Path::new(CARGO), &["build", "--offline"], fixture.root());
    let complete: String =
        std::fs::read_to_string(fixture.root().join("Cargo.lock")).expect("the lock Cargo wrote");
    let stripped: String = without_the_sibling(complete.as_str());
    assert_ne!(
        stripped, complete,
        "the lock Cargo wrote names no sibling to strip"
    );
    std::fs::write(fixture.root().join("Cargo.lock"), stripped.as_bytes())
        .expect("the lock without the sibling");
    fixture.git(&["add", "--all"]);
    fixture.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "--no-verify",
        "-m",
        "the lock the manifest no longer matches",
    ]);
    return stripped;
}

/// What:
///  A lock whose bytes do not name the sibling,
///  which the clean of a lockless manifest rewrites.
/// Why:
///  `cargo clean` refuses a stale lock only with `--locked`,
///  and the add's own clean does not pass it,
///  so the lock it rewrites is the one the arm has to displace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = text.split('[[package]]').filter((part) => !part.includes('name = "two"')).join('[[package]]');
/// ```
pub(super) fn without_the_sibling(text: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for part in text.split("[[package]]") {
        if part.contains("name = \"two\"") {
            continue;
        }
        kept.push(part);
    }
    return kept.join("[[package]]");
}

/// What:
///  The launcher arm's state in the checkout:
///  a lock that declares one launcher,
///  a record naming another checkout's lock,
///  and the worktree's own stand-in `mpm-shadow`.
/// Why:
///  The repair rewrites the links of a worktree whose records name another
///  lock,
///  and the program is the one the worktree's own link names when the search
///  path holds none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeLock(); writeRecord(ELSEWHERE); writeStandIn('.cache/toolchain-bin/mpm-shadow');
/// ```
pub(super) fn the_launcher_state(fixture: &Fixture) {
    fixture.commit(
        &[
            ("toolchain.lock.json", TOOLCHAIN_LOCK),
            ("src/main.txt", "the toolchain\n"),
        ],
        "the toolchain lock",
    );
    std::fs::create_dir_all(ELSEWHERE).expect("the other checkout's folder");
    std::fs::write(
        Path::new(ELSEWHERE).join("toolchain.lock.json"),
        TOOLCHAIN_LOCK.as_bytes(),
    )
    .expect("the other checkout's lock");
    let launchers: PathBuf = fixture.root().join(".cache").join("toolchain-launchers");
    std::fs::create_dir_all(launchers.as_path()).expect("the launcher folder");
    std::fs::write(
        launchers.join(format!("{LAUNCHER}.launch")),
        format!(
            "{{\"toolId\":\"step21:own\",\"lockFile\":\"{ELSEWHERE}/toolchain.lock.json\",\"real\":\"/tools/own\",\"scope\":\"repo\"}}\n"
        )
        .as_bytes(),
    )
    .expect("the record naming another lock");
    let bin: PathBuf = fixture.root().join(".cache").join("toolchain-bin");
    std::fs::create_dir_all(bin.as_path()).expect("the toolchain bin folder");
    let stand_in: PathBuf = bin.join("mpm-shadow");
    std::fs::write(stand_in.as_path(), MPM_STAND_IN.as_bytes()).expect("the stand-in");
    std::fs::set_permissions(
        stand_in.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )
    .expect("the stand-in's mode");
}
