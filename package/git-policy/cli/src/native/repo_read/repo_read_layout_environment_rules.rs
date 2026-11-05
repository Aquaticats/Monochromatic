//! What:
//!  The rules about the environment of a Git command: which `GIT_` variables can move repository discovery, which
//!  configuration entries the environment injects, and the environment a child would see with an overlay applied.
//! Why:
//!  `repo_read_layout.rs` finds and checks the repository layout; these rules read only variable names and values, so they sit
//!  in a file of their own that does not use it, and `repo_read_layout.rs` re-exports the public ones.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function environmentIsPlain(variables: [string, string][]): boolean; // no GIT_ variable moves discovery
//! function effectiveEnvironment(overlay: [string, string][]): [string, string][];
//! ```

use crate::child_environment::{COUNT_VARIABLE, environment_value, parse_config_count};
use std::ffi::OsString;

/// The environment variables a plain read tolerates, by prefix; any other `GIT_` variable can move a path or a key.
///  The first group is the original list. The second group was added after reading what Git 2.56.0 does with each name
///  (`environment.h`, and the file named for each); none can change which configuration entry, attribute tree, repository,
///  index, object, ref or pathspec meaning an answer rests on, since each only traces, words output, reaches a network or a tool,
///  or sets a limit or a write-time choice:
///  Tracing and output: `GIT_TRACE` and every `GIT_TRACE2`, `GIT_TRACE_*` name (`trace.c` line 33, `trace2/tr2_sysenv.c`
///   lines 32 and 34), `GIT_FLUSH` (`write-or-die.c` line 25), `GIT_PROGRESS_DELAY` (`progress.c` line 286),
///   `GIT_PRINT_SHA1_ELLIPSIS` (`environment.c` line 242), `GIT_MERGE_VERBOSITY` (`merge-ort.c` line 5516), `GIT_ADVICE`
///   (`advice.c` line 139), `GIT_REFLOG_ACTION` (`sequencer.c` line 53), `GIT_CHERRY_PICK_HELP` (`sequencer.c` line 527);
///  Network and transport: `GIT_CURL_*` (`http.c` line 1494), `GIT_HTTP_*` (`http.c` lines 1475 to 1477), `GIT_SSL_*`
///   (`http.c` line 1465), `GIT_PROXY_*` (`connect.c` line 1046), `GIT_TCP_KEEP*` (`http.c` line 1507), `GIT_USER_AGENT`
///   (`version.c` line 33), `GIT_OVERRIDE_VIRTUAL_HOST` (`connect.c` line 1276), `GIT_PROTOCOL_FROM_USER` (`transport.c` line 1182);
///  Tools: `GIT_EXTERNAL_DIFF` (`diff.c` line 565), `GIT_DIFF_*` (`diff.c` line 4045, `builtin/difftool.c` lines 792 and 799),
///   `GIT_MERGETOOL_GUI` (`builtin/difftool.c` line 786), `GIT_MAN_VIEWER` (`builtin/help.c` line 521), `GIT_SHELL_PATH`
///   (`builtin/var.c` line 149);
///  Limits and write-time choices: `GIT_ALLOC_LIMIT` (`wrapper.c` line 25), `GIT_MMAP_LIMIT` (`wrapper.c` line 947),
///   `GIT_INDEX_VERSION` (`read-cache.c` line 1646, the version Git writes; the index reader takes versions 2 to 4),
///   `GIT_BASENAME_FACTOR` (`diffcore-rename.c` line 1497), `GIT_COMMIT_GRAPH_PARANOIA` (`commit-graph.c` line 1028),
///   `GIT_DISABLE_UNTRACKED_CACHE` and `GIT_FORCE_UNTRACKED_CACHE` (`dir.c` lines 2988 and 3174),
///   `GIT_AUTO_COMMENT_CHAR_CONFIG_WARNING_GIVEN` (`config.c` line 2260, only suppresses a warning).
///  Left out because they can move an answer: `GIT_DIR`, `GIT_WORK_TREE`, `GIT_COMMON_DIR`, `GIT_PREFIX`, `GIT_IMPLICIT_WORK_TREE`,
///  `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_CEILING_DIRECTORIES`,
///  `GIT_DISCOVERY_ACROSS_FILESYSTEM`, `GIT_NAMESPACE`, `GIT_CONFIG`, `GIT_CONFIG_PARAMETERS`, `GIT_ATTR_*`, `GIT_NO_REPLACE_OBJECTS`,
///  `GIT_REPLACE_REF_BASE`, `GIT_GRAFT_FILE`, `GIT_SHALLOW_FILE`, `GIT_QUARANTINE_PATH`, `GIT_REFERENCE_BACKEND`, `GIT_REF_PARANOIA`,
///  the four `GIT_*_PATHSPECS`, `GIT_TEMPLATE_DIR`, `GIT_TEXTDOMAINDIR` (the wording the synthesized failures assume),
///  `GIT_NOTES_*`, `GIT_NO_LAZY_FETCH`, `GIT_DEFAULT_*` and every `GIT_TEST_*`.
pub(crate) const PLAIN_ENVIRONMENT_PREFIXES: &[&str] = &[
    "GIT_AUTHOR_",
    "GIT_COMMITTER_",
    "GIT_TERMINAL_PROMPT",
    "GIT_ASKPASS",
    "GIT_PAGER",
    "GIT_EDITOR",
    "GIT_SEQUENCE_EDITOR",
    "GIT_EXEC_PATH",
    "GIT_SSH",
    "GIT_ALLOW_PROTOCOL",
    "GIT_LFS_",
    "GIT_CONFIG_NOSYSTEM",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_KEY_",
    "GIT_CONFIG_VALUE_",
    "GIT_MERGE_AUTOEDIT",
    "GIT_OPTIONAL_LOCKS",
    "GIT_TRACE",
    "GIT_FLUSH",
    "GIT_PROGRESS_DELAY",
    "GIT_PRINT_SHA1_ELLIPSIS",
    "GIT_MERGE_VERBOSITY",
    "GIT_ADVICE",
    "GIT_REFLOG_ACTION",
    "GIT_CHERRY_PICK_HELP",
    "GIT_CURL_",
    "GIT_HTTP_",
    "GIT_SSL_",
    "GIT_PROXY_",
    "GIT_TCP_KEEP",
    "GIT_USER_AGENT",
    "GIT_OVERRIDE_VIRTUAL_HOST",
    "GIT_PROTOCOL_FROM_USER",
    "GIT_EXTERNAL_DIFF",
    "GIT_DIFF_",
    "GIT_DIFFTOOL_",
    "GIT_MERGETOOL_GUI",
    "GIT_MAN_VIEWER",
    "GIT_SHELL_PATH",
    "GIT_ALLOC_LIMIT",
    "GIT_MMAP_LIMIT",
    "GIT_INDEX_VERSION",
    "GIT_BASENAME_FACTOR",
    "GIT_COMMIT_GRAPH_PARANOIA",
    "GIT_DISABLE_UNTRACKED_CACHE",
    "GIT_FORCE_UNTRACKED_CACHE",
    "GIT_AUTO_COMMENT_CHAR_CONFIG_WARNING_GIVEN",
];

/// The prefix of the variables this package owns (`GIT_POLICY_NATIVE_IMAGE_TAG` keeps the images of parallel runs apart);
///  Git defines none of them, so they cannot move a path or a key.
pub const WRAPPER_ENVIRONMENT_PREFIX: &str = "GIT_POLICY_";

/// How many environment-supplied configuration entries the reader inspects before it declines.
///  Exempt from a citation under `LQM` (class: reader fallback): a file, chain or count past this bound makes the reader
///  decline and real Git answer, so a Git start is added and no result changes. Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
pub(crate) const ENV_CONFIG_ENTRY_LIMIT: u64 = 1024;

/// The keys an environment-supplied configuration entry may not set without the reader declining.
pub(crate) fn env_key_moves_discovery(key: &str) -> bool {
    let lowered: String = key.to_ascii_lowercase();
    return lowered == "core.bare" || lowered == "core.worktree" || lowered.starts_with("include");
}

/// Whether the `GIT_CONFIG_COUNT` entries of `variables` (the wrapper injects some) all leave discovery alone.
pub(crate) fn env_config_is_plain(variables: &[(OsString, OsString)]) -> bool {
    return env_config_avoids(variables, &env_key_moves_discovery);
}

/// What:
///  Whether every environment-supplied configuration entry has a key `declines` does not name;
///  a malformed set of entries is not plain either.
/// Why:
///   The content queries use the same walk with their own key list, so an injected `diff.orderFile` is not missed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// envConfigKeys(variables).every((key) => !declines(key)); // false when malformed
/// ```
pub fn env_config_avoids(variables: &[(OsString, OsString)], declines: &dyn Fn(&str) -> bool) -> bool {
    let Some(count_text) = environment_value(variables, COUNT_VARIABLE) else {
        return true;
    };
    let Some(count) = parse_config_count(count_text.as_os_str()) else {
        return false;
    };
    if count > ENV_CONFIG_ENTRY_LIMIT {
        return false;
    }
    for index in 0..count {
        let Some(key) = environment_value(variables, format!("GIT_CONFIG_KEY_{index}").as_str()) else {
            return false;
        };
        match key.to_str() {
            Some(text) if !declines(text) => {}
            _ => return false,
        }
    }
    return true;
}

/// Whether every `GIT_` variable among `variables` is one that cannot move discovery:
///  a tolerated prefix, or one of this package's own.
pub(crate) fn environment_is_plain(variables: &[(OsString, OsString)]) -> bool {
    for (name, _) in variables {
        let Some(text) = name.to_str() else {
            continue;
        };
        let mut tolerated: bool = text.starts_with(WRAPPER_ENVIRONMENT_PREFIX);
        for prefix in PLAIN_ENVIRONMENT_PREFIXES {
            if text.starts_with(prefix) {
                tolerated = true;
            }
        }
        if text.starts_with("GIT_") && !tolerated {
            return false;
        }
    }
    return env_config_is_plain(variables);
}

/// The environment Git would see: the process environment with the child overlay applied over it.
pub(crate) fn effective_environment(overlay: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    let mut variables: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in std::env::vars_os() {
        let mut replaced: bool = false;
        for (added, _) in overlay {
            if *added == name {
                replaced = true;
                break;
            }
        }
        if !replaced {
            variables.push((name, value));
        }
    }
    variables.extend(overlay.iter().cloned());
    return variables;
}
