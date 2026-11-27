//! What:
//! The disposable world a `tests/shim.rs` test runs in:
//! a scratch folder under the build output,
//! a link named `pnpm` to the built shadow,
//! a fake real pnpm that prints a known marker,
//! and a workspace directory to run in.
//! Why:
//! Every test needs the same arrangement and differs only in what it asks pnpm to do,
//! so one builder keeps them from drifting apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type World = { guard: TempDir; shadowDirectory: string; fakeDirectory: string; workspace: string };
//! function makeWorld(): World { /* create the three folders, link the shadow, write the fake pnpm */ }
//! ```

/// Imports the link helper and the executable-mode helper.
use std::os::unix::fs::{
    PermissionsExt,
    symlink,
};
/// Imports the path types.
use std::path::{
    Path,
    PathBuf,
};
/// Imports the child-process builder and its captured output.
use std::process::{
    Command,
    Output,
};

/// The built `pnpm-shadow` binary,
/// which Cargo builds before running these tests.
pub(crate) const SHADOW_BINARY: &str = env!("CARGO_BIN_EXE_pnpm-shadow");

/// Where integration-test scratch goes:
/// the package's build directory,
/// never the system temporary directory.
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

/// What the fake pnpm prints first,
/// so the tests can tell its output from the shadow's.
pub(crate) const FAKE_MARKER: &str = "fake-pnpm-ran";

/// The fake pnpm's exit code,
/// taken from the environment variable of this name.
pub(crate) const EXIT_VARIABLE: &str = "FAKE_PNPM_EXIT";

/// When this variable is `1` the fake pnpm kills itself with SIGTERM after printing.
pub(crate) const KILL_VARIABLE: &str = "FAKE_PNPM_KILL";

/// The number of the termination signal on Linux and macOS,
/// written once and named.
pub(crate) const SIGTERM_NUMBER: i32 = 15;

/// An exit code no default maps to,
/// so a pass-through is visible.
pub(crate) const DISTINCT_EXIT_CODE: i32 = 9;

/// One test world:
/// a directory holding the shadow linked as `pnpm`,
/// a directory holding the fake real pnpm,
/// and a
/// workspace directory to run in.
pub(crate) struct World {
    _guard: tempfile::TempDir,
    pub(crate) shadow_directory: PathBuf,
    pub(crate) fake_directory: PathBuf,
    pub(crate) workspace: PathBuf,
}

pub(crate) fn make_world() -> World {
    std::fs::create_dir_all(SCRATCH_ROOT).expect("make the scratch root");
    let guard: tempfile::TempDir =
        tempfile::tempdir_in(SCRATCH_ROOT).expect("make a scratch directory");
    let shadow_directory: PathBuf = guard.path().join("shadow");
    let fake_directory: PathBuf = guard.path().join("fake");
    let workspace: PathBuf = guard.path().join("workspace");
    for directory in [&shadow_directory, &fake_directory, &workspace] {
        std::fs::create_dir_all(directory).expect("make a directory");
    }
    symlink(SHADOW_BINARY, shadow_directory.join("pnpm")).expect("link the shadow as pnpm");

    let fake: PathBuf = fake_directory.join("pnpm");
    let script: String = format!(
        "#!/bin/sh\necho \"{FAKE_MARKER} $@\"\nif [ \"${{{KILL_VARIABLE}:-}}\" = 1 ]; then kill -TERM $$; fi\nif [ \"$1\" = nested ]; then pnpm inner; fi\nexit \"${{{EXIT_VARIABLE}:-0}}\"\n"
    );
    std::fs::write(&fake, script).expect("write the fake pnpm");
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755))
        .expect("make it executable");
    return World {
        _guard: guard,
        shadow_directory,
        fake_directory,
        workspace,
    };
}

pub(crate) fn path_of(directories: &[&Path]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for directory in directories {
        parts.push(directory.to_string_lossy().to_string());
    }
    return parts.join(":");
}

pub(crate) fn run_shadow(
    world: &World,
    path: &str,
    arguments: &[&str],
    fake_exit: i32,
) -> Output {
    return Command::new(world.shadow_directory.join("pnpm"))
        .args(arguments)
        .current_dir(&world.workspace)
        .env("PATH", path)
        .env(EXIT_VARIABLE, fake_exit.to_string())
        .output()
        .expect("start the shadow");
}

pub(crate) fn write_workspace(
    world: &World,
    manifest: &str,
) {
    std::fs::write(
        world.workspace.join("pnpm-workspace.yaml"),
        "packages: []\n",
    )
    .expect("write the marker");
    std::fs::write(world.workspace.join("package.json"), manifest).expect("write the manifest");
}

pub(crate) const LITERAL: &str = r#"{ "dependencies": { "left-pad": "1.3.0" } }"#;
pub(crate) const CLEAN: &str = r#"{ "dependencies": { "x": "catalog:" } }"#;

/// The text the notice prints for `--version`,
/// from the fake's own path.
pub(crate) fn expected_version_line(world: &World) -> String {
    return format!(
        "pnpm-shadow {}, shadowing pnpm at {}\n",
        env!("CARGO_PKG_VERSION"),
        world.fake_directory.join("pnpm").display()
    );
}
