//! What:
//! The launcher arrangement a `tests/shim.rs` test uses to stand in for the toolchain:
//! `pnpm` links to a separate launcher executable that starts the shadow under the name `pnpm`,
//! and a record beside the launcher names the shadow.
//! Why:
//! The first released shadow compared resolved paths and looped through such a launcher;
//! these helpers reproduce that arrangement so the launcher tests can show the guard bounds it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function putALauncherInFront(world: World): void { /* link pnpm to a launcher, write its .launch record */ }
//! ```

/// Imports the link helper and the executable-mode helper.
use std::os::unix::fs::{
    PermissionsExt,
    symlink,
};
/// Imports the owned path type.
use std::path::PathBuf;
/// Imports the child-process builder and its captured output.
use std::process::{
    Command,
    Output,
};

/// Imports the world the launcher is put in front of.
use crate::world::{
    EXIT_VARIABLE,
    SHADOW_BINARY,
    World,
};

/// How long a case may run before `timeout` kills it:
/// a loop is a failure,
/// not a hang.
const LIMIT_SECONDS: &str = "30";

/// `timeout` by absolute path:
/// the child's `PATH` holds only the shadow and the fake pnpm,
/// and Rust resolves a bare program
/// name against the `PATH` it is given.
const TIMEOUT_PROGRAM: &str = "/usr/bin/timeout";

/// The exit code `timeout` gives a command it had to kill.
pub(crate) const TIMED_OUT_EXIT_CODE: i32 = 124;

/// Replaces the shadow's link with the real toolchain's arrangement:
/// `pnpm` links to a launcher,
/// a separate executable
/// that starts the shadow under the name `pnpm`.
/// Its resolved path is therefore not the shadow's own executable path.
pub(crate) fn put_a_launcher_in_front(world: &World) {
    let launcher_directory: PathBuf = world.shadow_directory.join("..").join("launcher");
    std::fs::create_dir_all(&launcher_directory).expect("make the launcher directory");
    let launcher: PathBuf = launcher_directory.join("pnpm-shadow-launcher");
    std::fs::write(
        &launcher,
        format!("#!/bin/bash\nexec -a pnpm \"{SHADOW_BINARY}\" \"$@\"\n"),
    )
    .expect("write the launcher");
    std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755))
        .expect("make it executable");
    // The toolchain writes a record beside every launcher it makes; the shadow reads it to see that the launcher leads back to a shadow and
    // skips it without starting it. A launcher with no record is covered by `never_start_another_shadow.rs`.
    std::fs::write(
        launcher_directory.join("pnpm-shadow-launcher.launch"),
        format!("{{\"toolId\":\"cargo:monochromatic-pnpm-shadow\",\"lockFile\":\"/dev/null\",\"real\":\"{SHADOW_BINARY}\",\"scope\":\"repo\"}}"),
    )
    .expect("write the launcher record");
    let link: PathBuf = world.shadow_directory.join("pnpm");
    std::fs::remove_file(&link).expect("remove the direct link");
    symlink(&launcher, &link).expect("link pnpm to the launcher");
}

/// Runs the link under `timeout`,
/// so a loop fails the test instead of hanging it.
pub(crate) fn run_limited(
    world: &World,
    path: &str,
    arguments: &[&str],
) -> Output {
    // The notice handshake keeps its marker files in the per-user runtime directory; this world gets one of its own.
    let runtime: PathBuf = world.shadow_directory.join("..").join("runtime");
    std::fs::create_dir_all(&runtime).expect("make the runtime directory");
    return Command::new(TIMEOUT_PROGRAM)
        .env("XDG_RUNTIME_DIR", &runtime)
        .arg(LIMIT_SECONDS)
        .arg(world.shadow_directory.join("pnpm"))
        .args(arguments)
        .current_dir(&world.workspace)
        .env("PATH", path)
        .env(EXIT_VARIABLE, "0")
        .output()
        .expect("start timeout");
}
