//! What:
//!  The grammar of `git cli-git <command>`,
//!  the wrapper's own management namespace.
//! Why:
//!  `check` and `fix` run policies over an explicit scope;
//!  `doctor` lists the traditional hooks a repository holds;
//!  `trust`,
//!  `untrust` and
//!      `status` are retired with executable configuration and only explain that.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseManagementArgs(['check', '--all']) => { command: 'check', all: true, policies: [], pathspecs: [] }
//! ```

/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Pathspecs are file names and are kept as the exact bytes the caller gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string[] argv, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  The complete grammar shown when an invocation is refused.
///       `&str` is borrowed text compiled into the executable.
/// Why:
///   The text states the enforced rule:
///  exactly one scope per direct command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MANAGEMENT_USAGE = 'Usage: git cli-git check ...';
/// ```
pub const MANAGEMENT_USAGE: &str = "\
Usage: git cli-git check (--all | -- <pathspec>...) [--policy <id>]...
       git cli-git fix (--all | -- <pathspec>...) [--policy <id>]...
       git cli-git doctor
       git cli-git --help
";

/// Successful namespace help,
///  shown without touching any repository.
pub const MANAGEMENT_HELP: &str = "\
Usage: git cli-git <command> [options]

Run repository policies directly.

Commands:
  check    Check policies over an explicit scope.
  fix      Apply policy fixes over an explicit scope.
  doctor   List traditional hooks (unsupported). Exit 0 when none or only git-lfs shims are found,
           1 when any other hook is found, 2 outside a repository.

Scope (exactly one):
  --all             The whole repository.
  -- <pathspec>...  The named pathspecs.

Options:
  --policy <id>  Run only this policy; may be repeated.
  --help         Show this help.

Policies are configured as data in cli-git.config.jsonc at the repository top level.
The trust, untrust and status commands are retired: configuration is no longer code,
so there is nothing to approve.
";

/// What:
///  The three commands that existed only to approve executable configuration.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   Each still parses,
///  so old instructions get an explanation instead of a usage error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RetiredCommand = 'trust' | 'untrust' | 'status';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetiredCommand {
    /// `git cli-git trust [--yes]`.
    Trust,
    /// `git cli-git untrust`.
    Untrust,
    /// `git cli-git status`.
    Status,
}

/// What:
///  One understood management invocation.
///       `Vec<String>` is an owned list of owned UTF-8 text;
///  `Vec<OsString>` keeps raw bytes.
/// Why:
///   The runner acts on a typed value;
///  nothing downstream re-reads raw arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagementAction = { command: 'help' } | { command: 'retired'; ... } | { command: 'check' | 'fix'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagementAction {
    /// `--help` or `-h`:
    ///  print the namespace help.
    Help,
    /// A retired trust command;
    ///  `help` is true for `trust --help`.
    Retired {
        /// Which retired command was named.
        command: RetiredCommand,
        /// Whether help was requested,
        ///  which selects standard output.
        help: bool,
    },
    /// `doctor`:
    ///  list the traditional hooks the repository holds.
    Doctor,
    /// `check` or `fix` over exactly one scope.
    Direct {
        /// `true` for `fix`,
        ///  `false` for `check`.
        fix: bool,
        /// Whether the whole repository was selected with `--all`.
        all: bool,
        /// Selected policy IDs,
        ///  first occurrence order,
        ///  without repeats;
        ///  not yet validated.
        policies: Vec<String>,
        /// Pathspecs after `--`,
        ///  in order,
        ///  as raw bytes.
        pathspecs: Vec<OsString>,
    },
}

/// What:
///  Why a management invocation was refused.
/// Why:
///   Each refusal has its own remedy,
///  so each has its own message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagementRefusal = 'usage' | 'pathspecs-before-separator' | 'scope-required';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagementRefusal {
    /// Unknown command,
    ///  unknown option,
    ///  missing option value or stray argument.
    Usage,
    /// A pathspec appeared before `--`;
    ///  `fix` tells which direct command.
    PathspecsBeforeSeparator {
        /// `true` for `fix`,
        ///  `false` for `check`.
        fix: bool,
    },
    /// Neither or both of `--all` and pathspecs were given.
    ScopeRequired {
        /// `true` for `fix`,
        ///  `false` for `check`.
        fix: bool,
    },
}

/// What:
///  Report whether a token starts an option rather than naming a path.
/// Why:
///   As in Git,
///  a lone `-` is a positional,
///  and anything else dash-led is an option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isOptionToken(token: string): boolean { return token.startsWith('-') && token !== '-'; }
/// ```
fn is_option_token(token: &OsString) -> bool {
    // `&[u8]` borrows the token's raw bytes (`u8` is one byte).
    let bytes: &[u8] = token.as_encoded_bytes();
    return bytes.starts_with(b"-") && bytes != b"-";
}

/// What:
///  Parse a retired command's remaining arguments.
///       `&[OsString]` borrows the arguments after the command name.
///       `Result<T, E>` is "value or error":
///  `Ok(...)` or `Err(...)`.
/// Why:
///   The accepted forms are the ones the commands had:
///  `trust` takes `--yes` and
///       `--help`/`-h`;
///  `untrust` and `status` take nothing.
///  A lone `--` ends options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRetired(command: RetiredCommand, rest: string[]): ManagementAction;
/// ```
fn parse_retired(
    command: RetiredCommand,
    rest: &[OsString],
) -> Result<ManagementAction, ManagementRefusal> {
    // `mut` allows the flags to be set while scanning.
    let mut help: bool = false;
    let mut terminated: bool = false;
    // `for token in rest` borrows each argument in order.
    for token in rest {
        if terminated {
            // What: `Err(...)` is the failure variant; `return` leaves early.
            // Why:  These commands never took positional arguments.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return MANAGEMENT_REFUSED;
            // ```
            return Err(ManagementRefusal::Usage);
        }
        if token == "--" {
            terminated = true;
        } else if command == RetiredCommand::Trust && (token == "--help" || token == "-h") {
            help = true;
        } else if command == RetiredCommand::Trust && token == "--yes" {
            // Noninteractive consent has nothing left to consent to; it is accepted and ignored.
            continue;
        } else {
            return Err(ManagementRefusal::Usage);
        }
    }
    // `Ok(...)` is the success variant carrying the understood action.
    return Ok(ManagementAction::Retired { command, help });
}

/// What:
///  Add one selected policy ID to the list,
///  once.
///  `&mut Vec<String>` lends the list
///       for writing;
///  `&[u8]` borrows the value's raw bytes;
///  `Result<(), ManagementRefusal>`
///       is "nothing,
///  or a refusal".
/// Why:
///   A separated value and an attached value are recorded the same way:
///  policy IDs are
///       ASCII names,
///  so other bytes can never name a policy,
///  and a repeated ID counts once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordPolicy(policies: string[], value: Uint8Array): void { const id = decodeUtf8OrRefuse(value); if (!policies.includes(id)) policies.push(id); }
/// ```
fn record_policy(policies: &mut Vec<String>, value: &[u8]) -> Result<(), ManagementRefusal> {
    // What: `let Ok(text) = ... else { ... };` keeps valid UTF-8 or exits.
    // Why:  Bytes that are not text cannot be a policy ID.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const id = decodeUtf8OrRefuse(value);
    // ```
    let Ok(id) = std::str::from_utf8(value) else {
        return Err(ManagementRefusal::Usage);
    };
    let owned: String = String::from(id);
    // `.contains(&owned)` borrows the new ID to keep only its first occurrence.
    if !policies.contains(&owned) {
        policies.push(owned);
    }
    // `Ok(())` is success carrying nothing.
    return Ok(());
}

/// What:
///  Parse `check` or `fix` arguments in one forward pass.
///       `&[OsString]` borrows the arguments after the command word.
/// Why:
///   `--policy` takes its next argument (or an attached `=value`) even when that
///       value starts with a dash;
///  everything after `--` is a pathspec.
///  A pathspec
///       before `--` is refused so a mistyped option can never become a path,
///  and the
///       two scopes are mutually exclusive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseDirect(fix: boolean, rest: string[]): ManagementAction;
/// ```
fn parse_direct(fix: bool, rest: &[OsString]) -> Result<ManagementAction, ManagementRefusal> {
    let mut all: bool = false;
    // `Vec::<String>::new()` is an empty owned list; `mut` allows pushing.
    let mut policies: Vec<String> = Vec::<String>::new();
    let mut pathspecs: Vec<OsString> = Vec::<OsString>::new();
    let mut terminated: bool = false;
    let mut pathspec_before_separator: bool = false;
    // True while the token being visited is the value of the `--policy` before it.
    let mut is_policy_value: bool = false;
    // `for token in rest` borrows each argument once, in order, so the scan always ends.
    for token in rest {
        if is_policy_value {
            is_policy_value = false;
            // A trailing `?` returns the refusal to our caller, or continues.
            record_policy(&mut policies, token.as_encoded_bytes())?;
            continue;
        }
        if terminated {
            // `.clone()` copies the pathspec bytes into the owned result.
            pathspecs.push(token.clone());
            continue;
        }
        if token == "--" {
            terminated = true;
            continue;
        }
        if !is_option_token(token) {
            pathspec_before_separator = true;
            continue;
        }
        if token == "--all" {
            all = true;
            continue;
        }
        // The policy value is the next argument, or attached after `--policy=`.
        if token == "--policy" {
            is_policy_value = true;
            continue;
        }
        // `let Some(x) = ... else { ... };` unwraps the attached value or refuses the option.
        let Some(attached) = token.as_encoded_bytes().strip_prefix(b"--policy=") else {
            return Err(ManagementRefusal::Usage);
        };
        record_policy(&mut policies, attached)?;
    }
    // `--policy` was the last argument: its value is missing.
    if is_policy_value {
        return Err(ManagementRefusal::Usage);
    }
    if pathspec_before_separator {
        return Err(ManagementRefusal::PathspecsBeforeSeparator { fix });
    }
    if all == !pathspecs.is_empty() {
        return Err(ManagementRefusal::ScopeRequired { fix });
    }
    return Ok(ManagementAction::Direct {
        fix,
        all,
        policies,
        pathspecs,
    });
}

/// What:
///  Parse the arguments after `git cli-git` into one action or refusal.
/// Why:
///   The leading command name selects one grammar,
///  so an invocation naming two
///       commands is refused for its stray argument rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseManagementArguments(args: string[]): ManagementAction;
/// ```
pub fn parse_management_arguments(
    arguments: &[OsString],
) -> Result<ManagementAction, ManagementRefusal> {
    // What: `let Some(name) = arguments.first() else { ... };` unwraps the command name.
    // Why:  A bare `git cli-git` names no command.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [name, ...rest] = args; if (name === undefined) return MANAGEMENT_REFUSED;
    // ```
    let Some(name) = arguments.first() else {
        return Err(ManagementRefusal::Usage);
    };
    // `&arguments[1..]` borrows every argument after the first.
    let rest: &[OsString] = &arguments[1..];
    if name == "--help" || name == "-h" {
        if !rest.is_empty() {
            return Err(ManagementRefusal::Usage);
        }
        return Ok(ManagementAction::Help);
    }
    if name == "trust" {
        return parse_retired(RetiredCommand::Trust, rest);
    }
    if name == "untrust" {
        return parse_retired(RetiredCommand::Untrust, rest);
    }
    if name == "status" {
        return parse_retired(RetiredCommand::Status, rest);
    }
    // `doctor` takes no options or scope, so any stray argument is a usage error.
    if name == "doctor" {
        if !rest.is_empty() {
            return Err(ManagementRefusal::Usage);
        }
        return Ok(ManagementAction::Doctor);
    }
    if name == "check" {
        return parse_direct(false, rest);
    }
    if name == "fix" {
        return parse_direct(true, rest);
    }
    return Err(ManagementRefusal::Usage);
}

/// Grammar controls stay out of the release executable.
#[cfg(test)]
#[path = "management_arguments_tests.rs"]
mod tests;
