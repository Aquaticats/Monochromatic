//! What:
//! Reads a `cargo update` or `cargo generate-lockfile` command line and decides which pnpm command it stands for.
//! Why:
//! The owner's rule (2026-11-03, "follow pnpm semantics"): `cargo update` under the shadow behaves like `pnpm update`, which
//! is pnpm's own policy-aware command (https://pnpm.io/cli/update, read 2026-11-03), and `cargo generate-lockfile` like
//! `pnpm install --lockfile-only` (https://pnpm.io/cli/install). The mapping, flag by flag:
//!
//! - no names: `pnpm update`, every dependency within its range: the full catalog update of this workspace.
//! - `[SPEC]...` or `-p SPEC`: `pnpm update <name>`, the named dependencies within their ranges.
//! - `--recursive`: `pnpm update <name> --depth Infinity` (the dependencies of the named ones too); cargo's own resolver
//!   decides which move, then the policy checks the result. Without it, only the named ones.
//! - `--precise V`: `pnpm update <name>@<version>`, still subject to the policy; pnpm refuses it for a package that is not a
//!   direct dependency (`ERR_PNPM_UPDATE_VERSION_ON_INDIRECT_DEP`), and so does this.
//! - `--breaking`: `pnpm update --latest`, run by cargo and then checked; set aside to an older release when refused.
//! - `-n`, `--dry-run`: pnpm update has no dry run; the nearest is this tool's own: report, then restore every lockfile.
//! - `-w`, `--workspace`: `pnpm install` (relink the workspace's own packages): the install step.
//! - `--locked`, `--frozen`: `pnpm install --frozen-lockfile`: check and fail naming the lock, change nothing.
//! - `--offline`, `--ignore-rust-version`, `--config`, `-Z`, `--color`, `-q`, `-v`: cargo-only; forwarded to every cargo call.
//! - `-m`, `--manifest-path`: pnpm's `--dir`; it selects the workspace.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-plan.ts
//! export function planUpdate(args: string[]): UpdatePlan
//! ```

use std::ffi::OsString;

use crate::verb::{find_verb, words_before_double_dash};

/// What the command line stands for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdatePlan {
    /// `pnpm install --frozen-lockfile`: check only.
    FrozenCheck,
    /// `pnpm install` (and `--lockfile-only`): the install step.
    Install {
        /// Report without keeping anything.
        dry_run: bool,
    },
    /// `pnpm update`: the full catalog update.
    Full {
        /// Report without keeping anything.
        dry_run: bool,
    },
    /// `pnpm update <name>[@<version>]`: run the user's update, then the policy.
    Targeted {
        /// Report without keeping anything.
        dry_run: bool,
        /// The cargo arguments, starting with `update`, without the forwarded options.
        cargo_args: Vec<String>,
        /// The package names named (without any `@version`).
        names: Vec<String>,
        /// Whether `--precise` was given.
        precise: bool,
    },
}

/// A plan with the cargo options to forward to every cargo call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Planned {
    /// What to do.
    pub plan: UpdatePlan,
    /// The options to forward.
    pub forward: Vec<String>,
}

/// Options of `cargo update` that take the next word as their value.
const VALUE_OPTIONS: [&str; 8] = ["--precise", "-m", "--manifest-path", "--color", "--config", "-Z", "-p", "--package"];

/// Value options that are forwarded to every cargo call together with their value.
const FORWARDED_VALUE_OPTIONS: [&str; 3] = ["--color", "--config", "-Z"];

/// Flags that are forwarded to every cargo call.
const FORWARDED_FLAGS: [&str; 6] = ["--offline", "--ignore-rust-version", "-q", "--quiet", "-v", "--verbose"];

/// What:
/// `pub fn plan_update(arguments: &[OsString]) -> Result<Planned, String>` maps a `cargo update` or
/// `cargo generate-lockfile` command line to the pnpm command it stands for.
/// Why:
/// `Err` carries the message of a refusal that mirrors pnpm's own (a `--precise` without one name, or for an indirect
/// dependency is checked later, where the lockfile is known).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planUpdate(args: string[]): Planned  // throws
/// ```
pub fn plan_update(arguments: &[OsString]) -> Result<Planned, String> {
    let words: Vec<String> = words_before_double_dash(arguments);
    let verb: String = find_verb(arguments).unwrap_or_default();
    let start: usize = words.iter().position(|word| { return *word == verb; }).map(|index| { return index + 1; }).unwrap_or(words.len());
    let mut dry_run: bool = false;
    let mut frozen: bool = false;
    let mut workspace: bool = false;
    let mut precise: bool = false;
    let mut breaking: bool = false;
    let mut names: Vec<String> = Vec::new();
    let mut cargo_args: Vec<String> = vec!["update".to_string()];
    let mut forward: Vec<String> = Vec::new();
    let mut index: usize = start;
    while index < words.len() {
        let word: &str = &words[index];
        let takes_value: bool = VALUE_OPTIONS.contains(&word);
        let value: String = if takes_value { words.get(index + 1).cloned().unwrap_or_default() } else { String::new() };
        if word == "-n" || word == "--dry-run" {
            dry_run = true;
        } else if word == "--locked" || word == "--frozen" {
            frozen = true;
        } else if word == "-w" || word == "--workspace" {
            workspace = true;
        } else if word == "--precise" {
            precise = true;
            cargo_args.extend(["--precise".to_string(), value.clone()]);
        } else if let Some(attached) = word.strip_prefix("--precise=") {
            precise = true;
            cargo_args.push(format!("--precise={attached}"));
        } else if word == "-m" || word == "--manifest-path" {
            // Dropped: the working directory of each cargo call is the selected workspace.
        } else if word.starts_with("--manifest-path=") {
            // Dropped, as above.
        } else if FORWARDED_VALUE_OPTIONS.contains(&word) {
            forward.extend([word.to_string(), value.clone()]);
        } else if FORWARDED_FLAGS.contains(&word) || word.starts_with("--color=") || word.starts_with("--config=") || (word.starts_with("-Z") && word.len() > 2) {
            forward.push(word.to_string());
        } else if word == "-p" || word == "--package" {
            names.push(value.split('@').next().unwrap_or("").to_string());
            cargo_args.extend([word.to_string(), value.clone()]);
        } else if word.starts_with('-') {
            if word == "--breaking" {
                breaking = true;
            }
            cargo_args.push(word.to_string());
        } else {
            names.push(word.split('@').next().unwrap_or("").to_string());
            cargo_args.push(word.to_string());
        }
        index += if takes_value { 2 } else { 1 };
    }
    if precise && names.len() != 1 {
        return Err("--precise needs exactly one package to update (pnpm update <name>@<version> names one dependency)".to_string());
    }
    let plan: UpdatePlan = if frozen {
        UpdatePlan::FrozenCheck
    } else if verb == "generate-lockfile" || (workspace && names.is_empty()) {
        UpdatePlan::Install { dry_run }
    } else if !names.is_empty() || precise || breaking {
        UpdatePlan::Targeted { dry_run, cargo_args, names, precise }
    } else {
        UpdatePlan::Full { dry_run }
    };
    return Ok(Planned { plan, forward });
}
