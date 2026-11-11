//! What:
//! End-to-end tests of the commands `mpm-shadow` prints for a person to run:
//! the sentence in the older-lock warning names a program and the verb that moves one row of the lock,
//! and that command,
//! run exactly as printed in a disposable home laid out the way this machine's is,
//! must reach mpm-shadow and move that home's machine-global lock,
//! and every sentence that names `mpm` or `mpm-shadow` must name a program the scope's own install puts on `PATH`.
//! Why:
//! Measured on 2026-11-08:
//! the machine-wide `mpm` is the real mpm (a uv tool),
//! which answers `update -g` with `Error: No such option '-g'.`,
//! and nothing but a worktree's `PATH` ever makes `mpm` the shadow.
//! Measured on 2026-11-11 in a home laid out like this machine's:
//! the machine-global plan declares no `mpm-shadow` entry,
//! so every sentence naming `mpm-shadow` (`update -g <tool id>`, `install -g`, `--help`) failed with
//! `/bin/sh: line 1: mpm-shadow: command not found`, exit status 127.
//! So the program the sentence names decides whether the printed command can work at all.
//! Nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mpm_shadow::toolchain::archive_staging::COMPLETE_MARKER;
use mpm_shadow::toolchain::cargo_installs::older_lock_message_for;
use mpm_shadow::toolchain::launcher_current_link::{MissingProgram, missing_program_advice};
use mpm_shadow::toolchain::lock::{parse_lock, render_lock};
use mpm_shadow::toolchain::scope::{LOCK_FILE_NAME, PLAN_FILE_NAME, ScopeKind};

/// The tool the fixture locks:
/// a crate of this repository,
/// so the older-lock warning applies to it.
const TOOL_ID: &str = "cargo:monochromatic-demo-tool";

/// The version the fixture's global lock pins.
const LOCKED: &str = "0.0.1";

/// The version the fixture's global toolchain root holds,
/// which is newer than the pin and so raises the warning.
const INSTALLED: &str = "0.0.2";

/// The bin name the plan adds to the tool and the first lock does not hold yet.
const ADDED_BIN: &str = "tool-alias";

/// The machine-global plan:
/// one repository-published crate at a pin,
/// plus one bin the first lock does not name,
/// so the printed `update` rewrites the lock without a network lookup
/// (an exact request whose lock row already holds it is kept).
const PLAN: &str = r#"{
  "schemaVersion": 1,
  "tools": {
    "cargo:monochromatic-demo-tool": {
      "request": "cargo:monochromatic-demo-tool@0.0.1",
      "route": "cargo-binstall",
      "bins": ["tool", "tool-alias"]
    }
  }
}
"#;

/// The machine-global lock as the fixture starts with it:
/// the same pin,
/// without the bin the plan adds.
const LOCK: &str = r#"{"schemaVersion":1,"tools":{"cargo:monochromatic-demo-tool":{"request":"cargo:monochromatic-demo-tool@0.0.1","version":"0.0.1","route":"cargo-binstall","bins":["tool"],"env":{},"options":{},"artifacts":{}}}}"#;

/// The real `mpm` a shell finds on this machine,
/// as measured on 2026-11-08:
/// `meta-package-manager` installed by `uv tool install`,
/// which reads `update` as its own `upgrade` verb and refuses `-g`.
/// PATH candidates are tried in order;
/// this name must lose to the shadow.
const REAL_MPM: &str = "#!/bin/sh\necho 'Usage: mpm upgrade [OPTIONS] [PACKAGES_SPECS]...' >&2\necho \"Error: No such option '-g'.\" >&2\nexit 2\n";

/// What:
/// A disposable home holding a machine-global plan and lock,
/// a newer installed copy of the fixture's tool,
/// and the two programs a shell in that home finds:
/// the real `mpm` under its own name,
/// and this build of the shadow under `mpm-shadow`.
/// Why:
/// The warning is raised by the state of the global lock,
/// and the printed command must be run against that state,
/// with the same names a machine-wide `PATH` offers.
struct Home {
    directory: tempfile::TempDir,
}

impl Home {
    fn new() -> Home {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        let home = Home { directory };
        fs::create_dir_all(home.config_dir()).expect("make the config directory");
        fs::create_dir_all(home.bin_dir()).expect("make the fixture's PATH directory");
        fs::write(home.config_dir().join(PLAN_FILE_NAME), PLAN).expect("write the global plan");
        let unsigned = parse_lock(LOCK, "fixture").expect("the fixture lock parses");
        fs::write(home.lock_file(), render_lock(&unsigned.lock)).expect("write the global lock");
        home.install_demo_tool();
        write_program(&home.bin_dir().join("mpm"), REAL_MPM);
        fs::copy(env!("CARGO_BIN_EXE_mpm-shadow"), home.bin_dir().join("mpm-shadow")).expect("expose the shadow as mpm-shadow");
        return home;
    }

    /// The fixture's PATH directory:
    /// where a machine-wide install links the programs a shell types.
    fn bin_dir(&self) -> PathBuf {
        return self.directory.path().join("bin");
    }

    /// `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain`.
    fn config_dir(&self) -> PathBuf {
        return self.directory.path().join("home").join(".config").join("monochromatic-dev").join("toolchain");
    }

    /// The machine-global toolchain root,
    /// `<data home>/monochromatic-dev/toolchain-global`.
    fn toolchain_root(&self) -> PathBuf {
        return self.directory.path().join("home").join(".local").join("share").join("monochromatic-dev").join("toolchain-global");
    }

    fn lock_file(&self) -> PathBuf {
        return self.config_dir().join(LOCK_FILE_NAME);
    }

    /// A directory with no plan anywhere above it,
    /// so only a `-g` command can find a scope.
    fn lonely_dir(&self) -> PathBuf {
        let lonely = self.directory.path().join("lonely");
        fs::create_dir_all(&lonely).expect("make the directory");
        return lonely;
    }

    /// Puts `INSTALLED` in the global toolchain root as a finished cargo install,
    /// the way `install` records one.
    fn install_demo_tool(&self) {
        let crate_dir = self.toolchain_root().join("cargo").join("monochromatic-demo-tool");
        let version_dir = crate_dir.join(INSTALLED);
        fs::create_dir_all(version_dir.join("bin")).expect("make the install");
        write_program(&version_dir.join("bin").join("tool"), "#!/bin/sh\nexit 0\n");
        fs::write(
            version_dir.join(".crates.toml"),
            format!("[v1]\n\"monochromatic-demo-tool {INSTALLED} (registry+https://example.invalid/index)\" = [\"tool\"]\n"),
        )
        .expect("write cargo's record");
        symlink(INSTALLED, crate_dir.join("current")).expect("link current");
    }

    /// Runs `program` the way a person in this home would:
    /// no other environment,
    /// and a `PATH` of the fixture's own directory plus the system ones.
    fn run(&self, program: &Path, arguments: &[&str]) -> Output {
        return Command::new(program)
            .args(arguments)
            .current_dir(self.lonely_dir())
            .env_clear()
            .env("HOME", self.directory.path().join("home"))
            .env("PATH", format!("{}:/usr/bin:/bin", self.bin_dir().display()))
            .output()
            .expect("run the program");
    }

    /// Runs one printed command line through a shell,
    /// so the command is read exactly as the warning printed it.
    fn run_text(&self, line: &str) -> Output {
        return Command::new("/bin/sh")
            .arg("-c")
            .arg(line)
            .current_dir(self.lonely_dir())
            .env_clear()
            .env("HOME", self.directory.path().join("home"))
            .env("PATH", format!("{}:/usr/bin:/bin", self.bin_dir().display()))
            .output()
            .expect("run the printed command");
    }
}

/// Writes `program` with the executable bit set.
fn write_program(path: &Path, program: &str) {
    fs::write(path, program).expect("write the program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod");
}

/// The text of a stream of a finished command.
fn text(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).to_string();
}

/// The command a warning prints,
/// which the sentence holds between backticks.
fn printed_command(warning: &str) -> String {
    let Some(open) = warning.find('`') else {
        panic!("the warning prints no command: {warning}");
    };
    let rest: &str = &warning[open + 1..];
    let Some(close) = rest.find('`') else {
        panic!("the warning's command has no closing backtick: {warning}");
    };
    return rest[..close].to_string();
}

#[test]
fn the_command_the_older_lock_warning_prints_reaches_the_shadow_and_moves_the_global_lock() {
    let home = Home::new();
    let before = fs::read_to_string(home.lock_file()).expect("read the global lock");
    let warned = home.run(Path::new(env!("CARGO_BIN_EXE_mpm-shadow")), &["bin", "--global"]);
    assert_eq!(warned.status.code(), Some(0), "{}", text(&warned.stderr));
    let warning = text(&warned.stderr);
    assert!(warning.contains(TOOL_ID), "the warning names the pinned tool: {warning}");
    assert!(warning.contains(LOCKED) && warning.contains(INSTALLED), "the warning names both versions: {warning}");
    let remedy = printed_command(&warning);

    let ran = home.run_text(&remedy);
    assert_eq!(
        ran.status.code(),
        Some(0),
        "the printed command `{remedy}` must run; it printed\n{}\n{}",
        text(&ran.stdout),
        text(&ran.stderr)
    );
    assert!(
        !text(&ran.stderr).contains("No such option"),
        "`{remedy}` reached the real mpm, not the shadow: {}",
        text(&ran.stderr)
    );
    assert!(
        text(&ran.stdout).contains(&format!("{TOOL_ID}")),
        "`{remedy}` reported on the tool: {}",
        text(&ran.stdout)
    );

    let after = fs::read_to_string(home.lock_file()).expect("read the global lock");
    assert_ne!(after, before, "`{remedy}` moved the lock this home owns");
    let lock: serde_json::Value = serde_json::from_str(&after).expect("the lock is JSON");
    let bins = &lock["tools"][TOOL_ID]["bins"];
    assert_eq!(bins[0], "tool", "{after}");
    assert_eq!(bins[1], ADDED_BIN, "the lock took the plan's new bin: {after}");
    assert_eq!(lock["tools"][TOOL_ID]["version"], LOCKED, "the pin did not move: {after}");
}

/// The bin the fixture's demo tool declares.
const DEMO_BIN: &str = "tool";

/// The repository-published tool the fixture's machine-global plan declares,
/// a crate of this repository so the older-lock warning applies to it.
const DEMO_CRATE: &str = "monochromatic-demo-tool";

/// The id the repository plan gives `mpm-shadow`,
/// the row whose postinstall link makes `mpm` this tool in a worktree.
const SHADOW_ID: &str = "mpm-shadow";

/// The postinstall directory of the repository plan's `mpm-shadow` row:
/// the one a worktree's `PATH` holds, so `mpm` there is the shadow.
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
/// the demo tool is installed at `INSTALLED`, and the lock pins `LOCKED`,
/// so the warning and the command that clears it are the real ones.
struct Machine {
    directory: tempfile::TempDir,
}

/// Which scope's directories a fresh shell in the fixture has on `PATH`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Place {
    /// The worktree:
    /// the exposure farm and the postinstall link directory the plan declares.
    Repo,
    /// The machine:
    /// the global exposure farm and the global postinstall link directory.
    Global,
}

impl Machine {
    /// The machine with both plans, both locks and every installed copy in place.
    fn new() -> Machine {
        let machine = Machine { directory: tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root") };
        fs::create_dir_all(machine.global_config()).expect("make the global config directory");
        fs::create_dir_all(machine.repo()).expect("make the worktree");
        machine.write_global_plan(LOCKED);
        fs::write(machine.global_lock_file(), demo_tool_lock(LOCKED, LOCKED)).expect("write the global lock");
        machine.write_repo_plan_and_lock();
        machine.install_demo_tool();
        machine.install_shadow();
        return machine;
    }

    fn home(&self) -> PathBuf {
        return self.directory.path().join("home");
    }

    /// The worktree the fixture's repository commands run in.
    fn repo(&self) -> PathBuf {
        return self.directory.path().join("work");
    }

    /// The directory a program started outside any worktree runs in.
    fn elsewhere(&self) -> PathBuf {
        let elsewhere: PathBuf = self.directory.path().join("away");
        fs::create_dir_all(&elsewhere).expect("make the directory");
        return elsewhere;
    }

    fn global_config(&self) -> PathBuf {
        return self.home().join(".config").join("monochromatic-dev").join("toolchain");
    }

    fn global_lock_file(&self) -> PathBuf {
        return self.global_config().join(LOCK_FILE_NAME);
    }

    /// `<data home>/monochromatic-dev/toolchain-global`, which both scopes' tools share.
    fn global_root(&self) -> PathBuf {
        return self.home().join(".local").join("share").join("monochromatic-dev").join("toolchain-global");
    }

    /// The machine-global exposure farm: the directory this machine's `mise` configuration puts on `PATH`.
    fn farm(&self) -> PathBuf {
        return self.global_root().join("exposure");
    }

    /// The program of the fixture's `mpm-shadow` install,
    /// the path a run of that install has as its own.
    fn shadow_program(&self) -> PathBuf {
        let version: &str = env!("CARGO_PKG_VERSION");
        let folder: String = format!("monochromatic-mpm-shadow-{version}-x86_64-unknown-linux-gnu");
        return self.global_root().join(SHADOW_ID).join(version).join(folder).join("mpm-shadow");
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
        fs::create_dir_all(program.parent().expect("a parent")).expect("make the version directory");
        fs::copy(env!("CARGO_BIN_EXE_mpm-shadow"), &program).expect("copy the shadow into the install");
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
        fs::write(version_dir.join(COMPLETE_MARKER), format!("{version}\n")).expect("mark the install finished");
        let folder: String = program.parent().expect("a parent").file_name().expect("a folder name").to_string_lossy().to_string();
        symlink(format!("{folder}/mpm-shadow"), version_dir.join("mpm-shadow")).expect("link the program beside its folder");
        symlink(version, self.global_root().join(SHADOW_ID).join("current")).expect("link current");
    }

    /// Puts the demo tool in the machine-global toolchain root at `INSTALLED`, the way `cargo binstall` records one.
    fn install_demo_tool(&self) {
        let version_dir: PathBuf = self.global_root().join("cargo").join(DEMO_CRATE).join(INSTALLED);
        let program: PathBuf = version_dir.join("bin").join(DEMO_BIN);
        fs::create_dir_all(program.parent().expect("a parent")).expect("make the install");
        fs::write(&program, "#!/bin/sh\necho the demo tool ran\n").expect("write the program");
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
        fs::write(
            version_dir.join(".crates.toml"),
            format!("[v1]\n\"{DEMO_CRATE} {INSTALLED} (registry+https://example.invalid/index)\" = [\"{DEMO_BIN}\"]\n"),
        )
        .expect("write cargo's record");
        symlink(INSTALLED, self.global_root().join("cargo").join(DEMO_CRATE).join("current")).expect("link current");
    }

    /// The machine-global plan, asking for the demo tool at `request`.
    fn write_global_plan(&self, request: &str) {
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
        fs::write(self.repo().join(LOCK_FILE_NAME), lock_with(&row)).expect("write the worktree's lock");
    }

    /// The `PATH` a fresh shell in the fixture has:
    /// the scope's own directories first, then the system programs.
    fn path(&self, place: Place) -> String {
        let directories: Vec<PathBuf> = match place {
            Place::Global => vec![self.farm(), self.home().join(".local").join("bin")],
            Place::Repo => vec![self.repo().join(".cache").join("toolchain-bin"), self.repo().join(SHADOW_LINK_DIR)],
        };
        let joined: Vec<String> = directories.iter().map(|directory| return directory.display().to_string()).collect();
        return format!("{}:/usr/bin:/bin", joined.join(":"));
    }

    /// Runs `program` the way a person in this home would:
    /// no other environment,
    /// and the `PATH` of the scope the command belongs to.
    fn run(&self, program: &Path, arguments: &[&str], place: Place) -> Output {
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
    fn shell(&self, line: &str, place: Place) -> Output {
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

    /// Runs the machine-global install from the installed copy, as a person on that machine would.
    fn install_global(&self) -> Output {
        let program: PathBuf = self.shadow_program();
        return self.run(&program, &["install", "--global"], Place::Global);
    }

    /// Runs the worktree's install,
    /// which links `mpm` and the farm of that tree.
    fn install_repo(&self) -> Output {
        let program: PathBuf = self.global_root().join(SHADOW_ID).join("current").join("mpm-shadow");
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
/// `install` refuses a lock it did not write, so the fixture's files must be the real thing.
fn lock_with(row: &str) -> String {
    let text: String = format!("{{\"schemaVersion\":1,\"tools\":{{{row}}}}}");
    let parsed = parse_lock(&text, "fixture").expect("the fixture lock parses");
    return render_lock(&parsed.lock);
}

/// The machine-global lock row for the demo tool:
/// the plan asks `request` and the lock names `version`.
fn demo_tool_lock(request: &str, version: &str) -> String {
    let row: String = format!(
        "\"{TOOL_ID}\":{{\"request\":\"cargo:{DEMO_CRATE}@{request}\",\"version\":\"{version}\",\"route\":\"cargo-binstall\",\"manager\":\"cargo\",\"packageId\":\"{DEMO_CRATE}\",\"bins\":[\"{DEMO_BIN}\"],\"env\":{{}},\"options\":{{}},\"artifacts\":{{}}}}"
    );
    return lock_with(&row);
}

/// What:
/// The command a sentence prints that starts with `program`.
/// Why:
/// A sentence may print other backticked words (`current`, a lock file)
/// before the command it tells a person to run,
/// so the program is what identifies the command,
/// and a sentence that names a different program than the one expected fails here.
fn command_naming(sentence: &str, program: &str) -> String {
    let mut rest: &str = sentence;
    while let Some(open) = rest.find('`') {
        let after: &str = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            break;
        };
        let span: &str = &after[..close];
        let first: &str = span.split(' ').next().unwrap_or_default();
        if first == program {
            return span.to_string();
        }
        rest = &after[close + 1..];
    }
    panic!("the sentence prints no command starting with {program}: {sentence}");
}

#[test]
fn every_printed_remedy_names_a_program_its_own_install_puts_on_path() {
    let machine = Machine::new();
    let installed = machine.install_global();
    assert_eq!(installed.status.code(), Some(0), "the machine-global install failed:\n{}", text(&installed.stderr));
    let linked = machine.install_repo();
    assert_eq!(linked.status.code(), Some(0), "the worktree's install failed:\n{}", text(&linked.stderr));

    let global_lock: PathBuf = machine.global_lock_file();
    let missing_global: PathBuf = machine.farm().join("mpm-shadow-nowhere");
    let global_sentences: [String; 2] = [
        older_lock_message_for(ScopeKind::Global, TOOL_ID, LOCKED, INSTALLED, &global_lock),
        missing_program_advice(MissingProgram {
            tool_id: TOOL_ID,
            scope: ScopeKind::Global,
            lock_file: &global_lock,
            real: &missing_global,
            lock_text: None,
        }),
    ];
    for sentence in &global_sentences {
        let command: String = command_naming(sentence, "mpm-shadow");
        let found = machine.shell(&format!("command -v {command}"), Place::Global);
        assert!(
            found.status.success(),
            "the machine-global install puts `{command}` on the machine's PATH, so the sentence runs as printed: {sentence}"
        );
    }

    let repo_lock: PathBuf = machine.repo().join(LOCK_FILE_NAME);
    let missing_repo: PathBuf = machine.repo().join(".cache").join("toolchain-bin").join("nothing-here");
    let repo_sentences: [String; 2] = [
        older_lock_message_for(ScopeKind::Repo, TOOL_ID, LOCKED, INSTALLED, &repo_lock),
        missing_program_advice(MissingProgram {
            tool_id: TOOL_ID,
            scope: ScopeKind::Repo,
            lock_file: &repo_lock,
            real: &missing_repo,
            lock_text: None,
        }),
    ];
    for sentence in &repo_sentences {
        let command: String = command_naming(sentence, "mpm");
        let found = machine.shell(&format!("command -v {command}"), Place::Repo);
        assert!(found.status.success(), "the worktree's install puts `{command}` on that worktree's PATH: {sentence}");
    }
}

#[test]
fn the_global_remedy_runs_as_printed_and_clears_the_warning() {
    let machine = Machine::new();
    let installed = machine.install_global();
    assert_eq!(installed.status.code(), Some(0), "the machine-global install failed:\n{}", text(&installed.stderr));
    let program: PathBuf = machine.farm().join("mpm-shadow");
    assert!(
        program.exists(),
        "the install puts the program the printed command starts in the farm a fresh shell reads: {}",
        program.display()
    );

    // The plan now asks for the version that runs, which is the state the warning's own command repairs.
    machine.write_global_plan(INSTALLED);
    let before = machine.shell(DEMO_BIN, Place::Global);
    assert_eq!(before.status.code(), Some(0), "the demo tool runs:\n{}", text(&before.stderr));
    let warning: String = text(&before.stderr);
    assert!(warning.contains(TOOL_ID) && warning.contains(LOCKED) && warning.contains(INSTALLED), "{warning}");
    let remedy: String = printed_command(&warning);
    assert_eq!(remedy, format!("mpm-shadow update -g {TOOL_ID}"), "the sentence names the shadow and the verb that moves one row");

    let ran = machine.shell(&remedy, Place::Global);
    assert_eq!(
        ran.status.code(),
        Some(0),
        "`{remedy}` must run as printed in a fresh shell; it printed\n{}\n{}",
        text(&ran.stdout),
        text(&ran.stderr)
    );

    let lock: String = fs::read_to_string(machine.global_lock_file()).expect("read the global lock");
    let parsed = parse_lock(&lock, "the lock the remedy wrote").expect("the lock parses");
    let tool = parsed.lock.tool(TOOL_ID).expect("the lock still holds the tool");
    assert_eq!(tool.version, INSTALLED, "the lock now names the version that runs: {lock}");
    assert_eq!(tool.declaration.request.range, INSTALLED, "and the request that asks for it: {lock}");

    let after = machine.shell(DEMO_BIN, Place::Global);
    assert_eq!(after.status.code(), Some(0), "the demo tool still runs:\n{}", text(&after.stderr));
    assert_eq!(text(&after.stderr), "", "the older-lock warning is gone, because the lock names the version that runs");
}
