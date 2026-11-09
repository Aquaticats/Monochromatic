//! What:
//! An end-to-end test of the command the older-lock warning prints:
//! the sentence in the warning names a program and the verb that moves one row of the lock,
//! and that command, run exactly as printed in a disposable home,
//! must reach mpm-shadow and move that home's machine-global lock.
//! Why:
//! Measured on 2026-11-08:
//! the machine-wide `mpm` is the real mpm (a uv tool),
//! which answers `update -g` with `Error: No such option '-g'.`,
//! and nothing but a worktree's `PATH` ever makes `mpm` the shadow.
//! So the program the sentence names decides whether the printed command can work at all.
//! Nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mpm_shadow::toolchain::lock::{parse_lock, render_lock};
use mpm_shadow::toolchain::scope::{LOCK_FILE_NAME, PLAN_FILE_NAME};

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
/// the same pin, without the bin the plan adds.
const LOCK: &str = r#"{"schemaVersion":1,"tools":{"cargo:monochromatic-demo-tool":{"request":"cargo:monochromatic-demo-tool@0.0.1","version":"0.0.1","route":"cargo-binstall","bins":["tool"],"env":{},"options":{},"artifacts":{}}}}"#;

/// The real `mpm` a shell finds on this machine, as measured on 2026-11-08:
/// `meta-package-manager` installed by `uv tool install`,
/// which reads `update` as its own `upgrade` verb and refuses `-g`.
/// PATH candidates are tried in order; this name must lose to the shadow.
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
