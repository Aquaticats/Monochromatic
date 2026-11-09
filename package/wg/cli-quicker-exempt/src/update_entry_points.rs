//! Recognizes the processes whose network traffic the wg tools keep outside the tunnel.
//! This list is owned by the wg packages,
//! and the toolchain knows nothing about it.
//! A test reads the root and package `mise.toml` tasks,
//! the `mpm-shadow` verb grammar
//! and the launcher script,
//! and fails when this list falls behind or names something that no longer exists.

/// Raw Unix path bytes for argument comparison.
use std::os::unix::ffi::OsStrExt;
/// Executable paths and install roots.
use std::path::{Path, PathBuf};

/// `mise` task names that fetch from the network.
/// The `:global` forms and the aliases that run a fetching task are listed too.
/// The tasks `toolchain:list`,
/// `toolchain:list:global` and `toolchain:lock:check` stay off the list:
/// they read the lock and the file system and open no connection.
pub const MISE_TASKS: &[&str] = &[
    "toolchain:update",
    "toolchain:update:global",
    "toolchain:install",
    "toolchain:install:global",
    "toolchain:outdated",
    "toolchain:outdated:global",
    "bootstrap",
    "prepare:android",
];

/// `mpm-shadow` verbs that fetch from the network,
/// with or without the `-g` flag,
/// including the short alias of `install`
/// and pnpm's own aliases of `update`.
pub const MPM_SHADOW_VERBS: &[&str] = &["install", "i", "update", "up", "upgrade", "outdated"];

/// Path tails of the TypeScript launcher that every root `toolchain:*` task runs to find or download `mpm-shadow`:
/// the source,
/// and the built entry that the tasks prefer when it exists.
/// Run directly,
/// outside `mise`,
/// it downloads the locked archive before any `mpm-shadow` process exists.
pub const TOOLCHAIN_SCRIPT_TAILS: &[&str] = &[
    "/monorepo-task/core-exec/src/mpm-shadow.ts",
    "/monorepo-task/core-exec/dist/final/node/mpm-shadow.mjs",
];

/// Tail of Pi's CLI entry,
/// the file the installed shim runs with the toolchain's node or another node:
/// a versioned store path whose last components name the package and the bundle.
pub const PI_CLI_TAIL: &str = "/pi-coding-agent/dist/bundle/cli.js";

/// Why a process is kept outside the tunnel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchReason {
    /// A `mise run` of a fetching toolchain task.
    MiseTask,
    /// An `mpm-shadow` fetching verb.
    MpmShadowVerb,
    /// The `mpm` shadow.
    MpmShim,
    /// The TypeScript `mpm-shadow` launcher run by `node`.
    ToolchainScript,
    /// An executable under the toolchain install roots.
    ToolchainPath,
    /// Pi,
    /// the agent harness this repository runs,
    /// whose provider traffic leaves the tunnel.
    PiAgent,
}

/// Wording for a reason.
impl MatchReason {
    /// Plain wording for log lines.
    pub fn describe(self) -> &'static str {
        return match self {
            MatchReason::MiseTask => "a mise toolchain task that fetches from the network",
            MatchReason::MpmShadowVerb => "an mpm-shadow command that fetches from the network",
            MatchReason::MpmShim => "the mpm shadow",
            MatchReason::ToolchainScript => "the TypeScript mpm-shadow launcher, which downloads from the network",
            MatchReason::ToolchainPath => "an executable under the toolchain install roots",
            MatchReason::PiAgent => "Pi, the agent harness whose provider traffic stays outside the tunnel",
        };
    }
}

/// The final path component of raw argument bytes.
fn basename(bytes: &[u8]) -> &[u8] {
    return match bytes.iter().rposition(is_slash) {
        Some(index) => &bytes[index + 1..],
        None => bytes,
    };
}

/// Reports a `/` byte.
fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// Reports whether the executable's own file name is `program`.
/// The file name comes from the resolved `exe` link,
/// which a process cannot rewrite after `exec`,
/// unlike the first command-line word that `exec -a` sets freely.
fn is_program(exe: &Path, program: &str) -> bool {
    return match exe.file_name() {
        Some(name) => name.as_bytes() == program.as_bytes(),
        None => false,
    };
}

/// Reports whether the process is the `mpm` shadow:
/// an executable named `mpm`,
/// or `mpm-shadow` started under the name `mpm`.
/// The command-line name counts only for `mpm-shadow`,
/// so `exec -a mpm /usr/bin/curl` does not match.
fn is_mpm_shadow(exe: &Path, args: &[Vec<u8>]) -> bool {
    if is_program(exe, "mpm") {
        return true;
    }
    return is_program(exe, "mpm-shadow")
        && match args.first() {
            Some(first) => basename(first) == b"mpm",
            None => false,
        };
}

/// Reports whether any argument after the first equals one of `names`.
fn has_argument_in(args: &[Vec<u8>], names: &[&str]) -> bool {
    for argument in args.iter().skip(1) {
        for name in names {
            if argument.as_slice() == name.as_bytes() {
                return true;
            }
        }
    }
    return false;
}

/// Reports whether the first argument that is not a flag is one of `names`.
fn first_word_in(args: &[Vec<u8>], names: &[&str]) -> bool {
    for argument in args.iter().skip(1) {
        if argument.first() == Some(&b'-') {
            continue;
        }
        for name in names {
            if argument.as_slice() == name.as_bytes() {
                return true;
            }
        }
        return false;
    }
    return false;
}

/// Reports whether any argument ends with one of the toolchain script path tails.
fn runs_toolchain_script(args: &[Vec<u8>]) -> bool {
    for argument in args.iter().skip(1) {
        for tail in TOOLCHAIN_SCRIPT_TAILS {
            if argument.ends_with(tail.as_bytes()) {
                return true;
            }
        }
    }
    return false;
}

/// Reports whether the process is Pi.
/// Pi rewrites its process title,
/// which leaves one command-line word,
/// its own name;
/// before that rewrite the exec event carries node and the entry Pi runs.
/// The first word counts only for `node`,
/// so `exec -a pi /usr/bin/curl` does not match.
fn is_pi_agent(exe: &Path, args: &[Vec<u8>]) -> bool {
    if !is_program(exe, "node") {
        return false;
    }
    if let Some(first) = args.first()
        && basename(first) == b"pi"
    {
        return true;
    }
    for argument in args.iter().skip(1) {
        if argument.ends_with(PI_CLI_TAIL.as_bytes()) {
            return true;
        }
    }
    return false;
}

/// Reports whether `exe` lies below one of `roots`,
/// comparing whole path components.
fn is_under_any(exe: &Path, roots: &[PathBuf]) -> bool {
    for root in roots {
        if exe.starts_with(root) {
            return true;
        }
    }
    return false;
}

/// Decides whether a process is an update entry point,
/// runs from the toolchain install roots,
/// or is Pi.
/// `exe` is the resolved executable,
/// `args` the raw command line with the program name first,
/// and `install_roots` the toolchain root spellings of the watched user.
pub fn classify(exe: &Path, args: &[Vec<u8>], install_roots: &[PathBuf]) -> Option<MatchReason> {
    if is_under_any(exe, install_roots) {
        return Some(MatchReason::ToolchainPath);
    }
    if is_program(exe, "mise") && has_argument_in(args, MISE_TASKS) {
        return Some(MatchReason::MiseTask);
    }
    if is_program(exe, "mpm-shadow") && first_word_in(args, MPM_SHADOW_VERBS) {
        return Some(MatchReason::MpmShadowVerb);
    }
    if is_mpm_shadow(exe, args) {
        return Some(MatchReason::MpmShim);
    }
    if is_program(exe, "node") && runs_toolchain_script(args) {
        return Some(MatchReason::ToolchainScript);
    }
    if is_pi_agent(exe, args) {
        return Some(MatchReason::PiAgent);
    }
    return None;
}
