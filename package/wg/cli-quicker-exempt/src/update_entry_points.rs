//! Recognizes the processes whose network traffic the wg tools keep outside the tunnel.
//! This list is owned by the wg packages,
//! and the toolchain knows nothing about it.
//! A test reads the toolchain's task names and `mpm-shadow` verbs and fails when this list falls behind.

/// Raw Unix path bytes for argument comparison.
use std::os::unix::ffi::OsStrExt;
/// Executable paths and install roots.
use std::path::{Path, PathBuf};

/// `mise` task names that fetch from the network.
/// The `:global` forms and the aliases that run a fetching task are listed too.
/// The names `install` and `outdated` are the post-cutover spellings of today's `sync` and `status`.
pub const MISE_TASKS: &[&str] = &[
    "toolchain:sync",
    "toolchain:sync:global",
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
/// including the short alias of `install`.
pub const MPM_SHADOW_VERBS: &[&str] = &["install", "i", "update", "outdated"];

/// Path tails of the TypeScript toolchain scripts that fetch from the network.
pub const TOOLCHAIN_SCRIPT_TAILS: &[&str] = &[
    "/file-enforcer/core/src/toolchain/sync.ts",
    "/file-enforcer/core/src/toolchain/update.ts",
];

/// Why a process is kept outside the tunnel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchReason {
    /// A `mise run` of a fetching toolchain task.
    MiseTask,
    /// An `mpm-shadow` fetching verb.
    MpmShadowVerb,
    /// The `mpm` shadow.
    MpmShim,
    /// A TypeScript toolchain script run by `node`.
    ToolchainScript,
    /// An executable under the toolchain install roots.
    ToolchainPath,
}

/// Wording for a reason.
impl MatchReason {
    /// Plain wording for log lines.
    pub fn describe(self) -> &'static str {
        return match self {
            MatchReason::MiseTask => "a mise toolchain task that fetches from the network",
            MatchReason::MpmShadowVerb => "an mpm-shadow command that fetches from the network",
            MatchReason::MpmShim => "the mpm shadow",
            MatchReason::ToolchainScript => "a TypeScript toolchain script that fetches from the network",
            MatchReason::ToolchainPath => "an executable under the toolchain install roots",
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

/// Decides whether a process is an update entry point or runs from the toolchain install roots.
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
    return None;
}

/// Most names listed from the install roots in a notice.
const MAX_LISTED_NAMES: usize = 64;

/// The names of the entries directly inside the install roots that exist,
/// sorted and without duplicates,
/// so a notice can say what is installed on this machine now.
pub fn installed_tool_names(roots: &[PathBuf]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for root in roots {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for listed in entries.flatten() {
            let name = listed.file_name().to_string_lossy().into_owned();
            if !name.starts_with('.') && !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names.sort();
    names.truncate(MAX_LISTED_NAMES);
    return names;
}

/// The toolchain install roots for one home directory,
/// in every spelling a resolved executable path can take.
/// `home` is the passwd home of the watched user,
/// and `canonical_home` its resolved path when the directory exists.
pub fn install_roots(home: &Path, canonical_home: Option<&Path>) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut homes: Vec<&Path> = vec![home];
    if let Some(canonical) = canonical_home {
        homes.push(canonical);
    }
    for candidate in homes {
        for directory in ["toolchain", "toolchain-global"] {
            let root = candidate.join(".local/share/monochromatic-dev").join(directory);
            if !roots.contains(&root) {
                roots.push(root);
            }
        }
    }
    return roots;
}
