//! What:
//! The fixture the printed-remedy tests build:
//! a disposable home laid out the way this machine's is,
//! with a machine-global plan and lock,
//! a repository worktree with its own plan and lock,
//! and the installed copies a machine that ran both installs holds.
//! Why:
//! The printed sentences are about state a person reaches from a fresh shell,
//! so a test must produce that state rather than write half of it by hand.
//! The versions:
//! the demo tool is installed at `INSTALLED`,
//! and the lock pins `LOCKED`,
//! so the warning and the command that clears it are the real ones.
//! Nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // printed_remedy/a_machine_with_both_scopes_installed.ts
//! export class Machine {
//!   static new(): Machine;
//!   installGlobal(): Output;
//!   installRepo(): Output;
//!   shell(line: string, place: Place): Output;
//! }
//! ```

use std::fs;
use std::os::unix::fs::{
    PermissionsExt,
    symlink,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Command,
    Output,
};

use mpm_shadow::toolchain::archive_staging::COMPLETE_MARKER;
use mpm_shadow::toolchain::lock::{
    parse_lock,
    render_lock,
};
use mpm_shadow::toolchain::scope::{
    LOCK_FILE_NAME,
    PLAN_FILE_NAME,
};

use crate::{
    INSTALLED,
    LOCKED,
    TOOL_ID,
};

/// The bin the fixture's demo tool declares.
pub(super) const DEMO_BIN: &str = "tool";

/// The program every machine-global sentence names,
/// the name the scope's install links in its exposure farm.
pub(super) const SHADOW_PROGRAM: &str = "mpm-shadow";

/// The programs every shell has,
/// after the scope's own directories.
const SYSTEM_PATH: &str = "/usr/bin:/bin";

/// The repository-published tool the fixture's machine-global plan declares,
/// a crate of this repository so the older-lock warning applies to it.
const DEMO_CRATE: &str = "monochromatic-demo-tool";

/// The id the repository plan gives `mpm-shadow`,
/// the row whose postinstall link makes `mpm` this tool in a worktree.
const SHADOW_ID: &str = "mpm-shadow";

/// The postinstall directory of the repository plan's `mpm-shadow` row:
/// the one a worktree's `PATH` holds,
/// so `mpm` there is the shadow.
const SHADOW_LINK_DIR: &str = ".cache/mpm-shadow-bin";

/// What:
/// A disposable home laid out the way this machine's is:
/// a machine-global plan and lock,
/// a repository worktree with its own plan and lock,
/// and the installed copies a machine that ran both installs holds.
/// Why:
/// The printed sentences are about state a person reaches from a fresh shell,
/// so the test must produce that state rather than write half of it by hand.
/// The versions:
/// the demo tool is installed at `INSTALLED`,
/// and the lock pins `LOCKED`,
/// so the warning and the command that clears it are the real ones.
pub(super) struct Machine {
    directory: tempfile::TempDir,
}

/// Which scope's directories a fresh shell in the fixture has on `PATH`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Place {
    /// The worktree:
    /// the exposure farm and the postinstall link directory the plan declares.
    Repo,
    /// The machine:
    /// the global exposure farm and the global postinstall link directory.
    Global,
}

impl Machine {
    /// The machine with both plans,
    /// both locks and every installed copy in place.
    pub(super) fn new() -> Machine {
        let machine = Machine {
            directory: tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR"))
                .expect("make a fixture root"),
        };
        fs::create_dir_all(machine.global_config()).expect("make the global config directory");
        fs::create_dir_all(machine.repo()).expect("make the worktree");
        machine.write_global_plan(LOCKED);
        fs::write(machine.global_lock_file(), demo_tool_lock(LOCKED, LOCKED))
            .expect("write the global lock");
        machine.write_repo_plan_and_lock();
        machine.install_demo_tool();
        machine.install_shadow();
        return machine;
    }

    fn home(&self) -> PathBuf {
        return self.directory.path().join("home");
    }

    /// The worktree the fixture's repository commands run in.
    pub(super) fn repo(&self) -> PathBuf {
        return self.directory.path().join("work");
    }

    /// The directory a program started outside any worktree runs in.
    fn elsewhere(&self) -> PathBuf {
        let elsewhere: PathBuf = self.directory.path().join("away");
        fs::create_dir_all(&elsewhere).expect("make the directory");
        return elsewhere;
    }

    fn global_config(&self) -> PathBuf {
        return self
            .home()
            .join(".config")
            .join("monochromatic-dev")
            .join("toolchain");
    }

    pub(super) fn global_lock_file(&self) -> PathBuf {
        return self.global_config().join(LOCK_FILE_NAME);
    }

    /// `<data home>/monochromatic-dev/toolchain-global`,
    /// which both scopes' tools share.
    fn global_root(&self) -> PathBuf {
        return self
            .home()
            .join(".local")
            .join("share")
            .join("monochromatic-dev")
            .join("toolchain-global");
    }

    /// The machine-global exposure farm:
    /// the directory this machine's `mise` configuration puts on `PATH`.
    pub(super) fn farm(&self) -> PathBuf {
        return self.global_root().join("exposure");
    }

    /// The program of the fixture's `mpm-shadow` install,
    /// the path a run of that install has as its own.
    fn shadow_program(&self) -> PathBuf {
        let version: &str = env!("CARGO_PKG_VERSION");
        let folder: String = format!("monochromatic-mpm-shadow-{version}-x86_64-unknown-linux-gnu");
        return self
            .global_root()
            .join(SHADOW_ID)
            .join(version)
            .join(folder)
            .join(SHADOW_PROGRAM);
    }

    /// What:
    /// Writes a release archive's install of `mpm-shadow` into the machine-global toolchain root:
    /// a version directory with its completion marker,
    /// the program in the folder the archive unpacked into,
    /// the link named after the program,
    /// and `current` naming the version.
    /// Why:
    /// The sentences name `mpm-shadow` as the program a person runs,
    /// and only an installed copy is what the farm and the launchers may put on `PATH`.
    fn install_shadow(&self) {
        let version: &str = env!("CARGO_PKG_VERSION");
        let version_dir: PathBuf = self.global_root().join(SHADOW_ID).join(version);
        let program: PathBuf = self.shadow_program();
        fs::create_dir_all(program.parent().expect("a parent"))
            .expect("make the version directory");
        fs::copy(env!("CARGO_BIN_EXE_mpm-shadow"), &program)
            .expect("copy the shadow into the install");
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755))
            .expect("make it executable");
        fs::write(version_dir.join(COMPLETE_MARKER), format!("{version}\n"))
            .expect("mark the install finished");
        let folder: String = program
            .parent()
            .expect("a parent")
            .file_name()
            .expect("a folder name")
            .to_string_lossy()
            .to_string();
        symlink(
            format!("{folder}/{SHADOW_PROGRAM}"),
            version_dir.join(SHADOW_PROGRAM),
        )
        .expect("link the program beside its folder");
        symlink(version, self.global_root().join(SHADOW_ID).join("current")).expect("link current");
    }

    /// Puts the demo tool in the machine-global toolchain root at `INSTALLED`,
    /// the way `cargo binstall` records one.
    fn install_demo_tool(&self) {
        let version_dir: PathBuf = self
            .global_root()
            .join("cargo")
            .join(DEMO_CRATE)
            .join(INSTALLED);
        let program: PathBuf = version_dir.join("bin").join(DEMO_BIN);
        fs::create_dir_all(program.parent().expect("a parent")).expect("make the install");
        fs::write(&program, "#!/bin/sh\necho the demo tool ran\n").expect("write the program");
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755))
            .expect("make it executable");
        fs::write(
            version_dir.join(".crates.toml"),
            format!("[v1]\n\"{DEMO_CRATE} {INSTALLED} (registry+https://example.invalid/index)\" = [\"{DEMO_BIN}\"]\n"),
        )
        .expect("write cargo's record");
        symlink(
            INSTALLED,
            self.global_root()
                .join("cargo")
                .join(DEMO_CRATE)
                .join("current"),
        )
        .expect("link current");
    }

    /// The machine-global plan,
    /// asking for the demo tool at `request`.
    pub(super) fn write_global_plan(
        &self,
        request: &str,
    ) {
        let plan: String = format!(
            "{{\n  \"schemaVersion\": 1,\n  \"tools\": {{\n    \"{TOOL_ID}\": {{\n      \"request\": \"cargo:{DEMO_CRATE}@{request}\",\n      \"route\": \"cargo-binstall\",\n      \"manager\": \"cargo\",\n      \"packageId\": \"{DEMO_CRATE}\",\n      \"bins\": [\"{DEMO_BIN}\"]\n    }}\n  }}\n}}\n"
        );
        fs::write(self.global_config().join(PLAN_FILE_NAME), plan).expect("write the global plan");
    }

    /// The repository worktree's plan and lock:
    /// the row the real plan carries for `mpm-shadow`,
    /// whose postinstall link makes `mpm` this tool in a worktree.
    fn write_repo_plan_and_lock(&self) {
        let plan: String = format!(
            "{{\n  \"schemaVersion\": 1,\n  \"tools\": {{\n    \"{SHADOW_ID}\": {{\n      \"request\": \"github:Aquaticats/Monochromatic@latest\",\n      \"route\": \"archive\",\n      \"bins\": [\"mpm-shadow\"],\n      \"os\": [\"linux\"],\n      \"options\": {{\n        \"tagPrefix\": \"monochromatic-mpm-shadow-v\",\n        \"postinstall\": {{ \"when\": \"always\", \"link\": {{ \"sourceBin\": \"mpm-shadow\", \"linkName\": \"mpm\", \"linkDir\": \"{SHADOW_LINK_DIR}\" }} }}\n      }}\n    }}\n  }}\n}}\n"
        );
        fs::write(self.repo().join(PLAN_FILE_NAME), plan).expect("write the worktree's plan");
        let version: &str = env!("CARGO_PKG_VERSION");
        let row: String = format!(
            "\"{SHADOW_ID}\":{{\"request\":\"github:Aquaticats/Monochromatic@latest\",\"version\":\"{version}\",\"route\":\"archive\",\"bins\":[\"mpm-shadow\"],\"os\":[\"linux\"],\"env\":{{}},\"options\":{{\"tagPrefix\":\"monochromatic-mpm-shadow-v\"}},\"artifacts\":{{}}}}"
        );
        fs::write(self.repo().join(LOCK_FILE_NAME), lock_with(&row))
            .expect("write the worktree's lock");
    }

    /// The `PATH` a fresh shell in the fixture has:
    /// the scope's own directories first,
    /// then the system programs.
    fn path(
        &self,
        place: Place,
    ) -> String {
        let directories: Vec<PathBuf> = match place {
            Place::Global => vec![self.farm(), self.home().join(".local").join("bin")],
            Place::Repo => vec![
                self.repo().join(".cache").join("toolchain-bin"),
                self.repo().join(SHADOW_LINK_DIR),
            ],
        };
        let joined: Vec<String> = directories
            .iter()
            .map(|directory| return directory.display().to_string())
            .collect();
        return format!("{}:{SYSTEM_PATH}", joined.join(":"));
    }

    /// Runs `program` the way a person in this home would:
    /// no other environment,
    /// and the `PATH` of the scope the command belongs to.
    fn run(
        &self,
        program: &Path,
        arguments: &[&str],
        place: Place,
    ) -> Output {
        return Command::new(program)
            .args(arguments)
            .current_dir(self.elsewhere())
            .env_clear()
            .env("HOME", self.home())
            .env("PATH", self.path(place))
            .output()
            .expect("run the program");
    }

    /// Runs one command line through a shell,
    /// so the fixture reads it exactly as a sentence printed it.
    pub(super) fn shell(
        &self,
        line: &str,
        place: Place,
    ) -> Output {
        return Command::new("/bin/sh")
            .arg("-c")
            .arg(line)
            .current_dir(self.elsewhere())
            .env_clear()
            .env("HOME", self.home())
            .env("PATH", self.path(place))
            .output()
            .expect("run the printed command");
    }

    /// Runs the machine-global install from the installed copy,
    /// as a person on that machine would.
    pub(super) fn install_global(&self) -> Output {
        let program: PathBuf = self.shadow_program();
        return self.run(&program, &["install", "--global"], Place::Global);
    }

    /// Runs the worktree's install,
    /// which links `mpm` and the farm of that tree.
    pub(super) fn install_repo(&self) -> Output {
        let program: PathBuf = self
            .global_root()
            .join(SHADOW_ID)
            .join("current")
            .join(SHADOW_PROGRAM);
        return Command::new(&program)
            .args(["install"])
            .current_dir(self.repo())
            .env_clear()
            .env("HOME", self.home())
            .env("PATH", self.path(Place::Global))
            .output()
            .expect("run the worktree's install");
    }
}

/// What:
/// The text of a lock file holding one tool row,
/// with the digest a hand-written file never has.
/// Why:
/// `install` refuses a lock it did not write,
/// so the fixture's files must be the real thing.
fn lock_with(row: &str) -> String {
    let text: String = format!("{{\"schemaVersion\":1,\"tools\":{{{row}}}}}");
    let parsed = parse_lock(&text, "fixture").expect("the fixture lock parses");
    return render_lock(&parsed.lock);
}

/// The machine-global lock row for the demo tool:
/// the plan asks `request` and the lock names `version`.
fn demo_tool_lock(
    request: &str,
    version: &str,
) -> String {
    let row: String = format!(
        "\"{TOOL_ID}\":{{\"request\":\"cargo:{DEMO_CRATE}@{request}\",\"version\":\"{version}\",\"route\":\"cargo-binstall\",\"manager\":\"cargo\",\"packageId\":\"{DEMO_CRATE}\",\"bins\":[\"{DEMO_BIN}\"],\"env\":{{}},\"options\":{{}},\"artifacts\":{{}}}}"
    );
    return lock_with(&row);
}
