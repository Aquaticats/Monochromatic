//! What:
//!  The typed settings a validated `cli-git.config.jsonc` produces,
//!  with defaults.
//! Why:
//!  The policy engine and transactions read plain typed values,
//!  never JSON.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type CliGitConfig = { policies: PolicyConfig; concurrency: ConcurrencyConfig };
//! ```

/// What:
///  Import the compiled-in registry and its identity and severity types.
/// Why:
///   Defaults are derived from the registry so the two can never disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { POLICY_REGISTRY, type PolicyId, type Severity } from './policy-registry.ts';
/// ```
use super::policy_registry::{
    POLICY_REGISTRY, PolicyDescriptor, PolicyId, Severity, policy_descriptor,
};

/// What:
///  `u64` constant,
///  an unsigned 64-bit integer (siblings `u32`,
///  `usize`,
///  `i64`).
/// Why:
///   Milliseconds up to 2^53 - 1 are accepted,
///  which `u32` cannot hold.
///       This is the incumbent's default patience for an unproven `index.lock` owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS = 1_000;
/// ```
pub const DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS: u64 = 1_000;

/// Incumbent default:
///  reserve the next landing slot after one lost landing race.
pub const DEFAULT_RESERVE_AFTER_LOST_RACES: u64 = 1;

/// What:
///  The effective severity of one shipped policy.
///       `#[derive(...)]` generates copy,
///  debug printing and `==` for the struct.
/// Why:
///   `explicit` separates "the repository chose this" from "the default applied";
///       only an explicit unsafe `warn` earns a configuration warning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicySetting = { id: PolicyId; severity: Severity; explicit: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicySetting {
    /// Which shipped policy this row configures.
    pub id: PolicyId,
    /// Severity in effect for this invocation.
    pub severity: Severity,
    /// Whether the repository configuration named this policy.
    pub explicit: bool,
}

/// What:
///  The closed set of Markdown rules the commit-time policy may run.
/// Why:
///   The shipped policy runs the owned linter with a fixed rule list;
///  an unknown
///       rule name is a configuration error,
///  never a pass-through argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MarkdownRule = 'lfs-image-url';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarkdownRule {
    /// `lfs-image-url`:
    ///  rewrite image links to LFS-tracked files to immutable object URLs.
    LfsImageUrl,
}

/// What:
///  Validated options of `markdown/autofix`.
///       `Vec<T>` is an owned growable list (siblings:
///  `&[T]` borrowed,
///  `[T; N]` fixed).
///       `String` is owned text (sibling:
///  `&str` borrowed).
/// Why:
///   The settings outlive the parsed document,
///  so they own their lists and text.
///       There is deliberately no command or executable field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MarkdownAutofixOptions = { rules: MarkdownRule[]; exclude: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownAutofixOptions {
    /// Rules to run,
    ///  without repeats;
    ///  defaults to the LFS image rewrite alone.
    pub rules: Vec<MarkdownRule>,
    /// gitignore-syntax patterns,
    ///  relative to the repository root,
    ///  left untouched.
    pub exclude: Vec<String>,
}

/// What:
///  One rules file named by the `rulesFile` option:
///  its path inside the repository and whether it may be absent.
/// Why:
///   The configuration names several files as an array,
///  and a gitignored local file must be allowed to be missing
///       on a fresh clone while a committed shared file must not.
///       The path is the configuration's own text, with no comma or `?` syntax,
///  because JSON can already say which file is meant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RulesFileEntry = { path: string; optional: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RulesFileEntry {
    /// The file,
    ///  relative to the repository's top level and already checked to stay inside it.
    pub path: String,
    /// Whether the file may be absent;
    ///  an absent optional file is skipped and an absent required file stops the policy.
    pub optional: bool,
}

/// What:
///  Validated options of `security/forbidden-strings`.
///  `Option<Vec<RulesFileEntry>>` is "a
///       configured list of files or nothing".
/// Why:
///   The scanner is linked into cli-git,
///  so the remaining choices are whether its
///       embedded baseline rules run and which repository files hold the private rules.
///       There is deliberately no executable path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForbiddenStringsOptions = { builtinRules: boolean; rulesFile?: string | (string | { path: string; optional?: boolean })[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForbiddenStringsOptions {
    /// Whether the scanner's embedded baseline rules are loaded;
    ///  default `true`.
    pub builtin_rules: bool,
    /// The private rules files in the order they are read,
    ///  each relative to the repository's top level and already checked to
    /// stay inside it;
    ///  never an empty list;
    ///  when absent,
    ///  `FORBIDDEN_STRINGS_RULES` and then the default file apply.
    pub rules_files: Option<Vec<RulesFileEntry>>,
}

/// What:
///  Every policy's severity plus the option records of option-bearing policies.
/// Why:
///   One value answers "does this policy run,
///  how strictly,
///  with what options".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicyConfig = { settings: PolicySetting[];
///   forbiddenStrings: ForbiddenStringsOptions; markdownAutofix: MarkdownAutofixOptions };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyConfig {
    /// One row per registry policy,
    ///  in registry order.
    pub settings: Vec<PolicySetting>,
    /// Options of `security/forbidden-strings`,
    ///  defaults applied.
    pub forbidden_strings: ForbiddenStringsOptions,
    /// Options of `markdown/autofix`,
    ///  defaults applied.
    pub markdown_autofix: MarkdownAutofixOptions,
}

/// What:
///  Hook serialization tuning (`hooks` key).
/// Why:
///   `false` serializes preparation and post-landing hooks through the hook lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HooksConfig = { concurrentCommits: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HooksConfig {
    /// Whether hooks of concurrent commits may overlap;
    ///  default `false`.
    pub concurrent_commits: bool,
}

/// What:
///  Foreign `index.lock` patience (`indexLock` key).
/// Why:
///   A lock whose owner cannot be proven alive is waited on only this long.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLockConfig = { unprovenOwnerTimeoutMs: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexLockConfig {
    /// Backoff budget in milliseconds;
    ///  whole number of at least 0.
    pub unproven_owner_timeout_ms: u64,
}

/// What:
///  Landing starvation tuning (`landing` key).
/// Why:
///   A commit that keeps losing landing races eventually reserves the next slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingConfig = { reserveAfterLostRaces: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LandingConfig {
    /// Lost races before reserving;
    ///  whole number of at least 1.
    pub reserve_after_lost_races: u64,
}

/// What:
///  The three concurrent-commit tuning groups together.
/// Why:
///   None of them disables the commit transaction;
///  they only tune it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConcurrencyConfig = { hooks: HooksConfig; indexLock: IndexLockConfig; landing: LandingConfig };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcurrencyConfig {
    /// Hook serialization.
    pub hooks: HooksConfig,
    /// Foreign index-lock patience.
    pub index_lock: IndexLockConfig,
    /// Landing starvation protection.
    pub landing: LandingConfig,
}

/// What:
///  The complete validated configuration of one repository.
/// Why:
///   A repository without a file and a repository with an empty `{}` file both yield
///       `CliGitConfig::defaults()`:
///  a policy that is off unless listed stays off either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CliGitConfig = { policies: PolicyConfig; concurrency: ConcurrencyConfig };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliGitConfig {
    /// Policy severities and options.
    pub policies: PolicyConfig,
    /// Concurrent-commit tuning.
    pub concurrency: ConcurrencyConfig,
}

/// What:
///  `impl ConcurrencyConfig { ... }` attaches functions to the struct.
/// Why:
///   Startup recovery runs before configuration loads and always uses these values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFAULT_CONCURRENCY_CONFIG: ConcurrencyConfig = { ... };
/// ```
impl ConcurrencyConfig {
    /// What:
    ///  Build the incumbent's default tuning.
    /// Why:
    ///   One function is the single source of the defaults for every caller.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static defaults(): ConcurrencyConfig;
    /// ```
    pub fn defaults() -> ConcurrencyConfig {
        return ConcurrencyConfig {
            hooks: HooksConfig {
                concurrent_commits: false,
            },
            index_lock: IndexLockConfig {
                unproven_owner_timeout_ms: DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS,
            },
            landing: LandingConfig {
                reserve_after_lost_races: DEFAULT_RESERVE_AFTER_LOST_RACES,
            },
        };
    }
}

/// What:
///  The severity of a policy the repository configuration does not name.
///  `&PolicyDescriptor`
///       borrows the policy's registry row.
/// Why:
///   The five built-in policies run everywhere at their default severity.
///  The four
///       policies that plugins used to provide run only where `cli-git.config.jsonc` names
///       them,
///  whether or not a file exists,
///  so an empty file and no file behave alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unlistedSeverity = descriptor.offUnlessListed ? 'off' : descriptor.defaultSeverity;
/// ```
pub fn unlisted_severity(descriptor: &PolicyDescriptor) -> Severity {
    if descriptor.off_unless_listed {
        return Severity::Off;
    }
    return descriptor.default_severity;
}

/// Defaults and lookup for policy settings.
impl PolicyConfig {
    /// What:
    ///  Build the settings of a repository whose configuration names no policy,
    ///  or
    ///       that has no configuration file at all.
    /// Why:
    ///   Parsing starts from these values and replaces only the policies the file names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static defaults(): PolicyConfig;
    /// ```
    pub fn defaults() -> PolicyConfig {
        // What: `Vec::<PolicySetting>::with_capacity(n)` is an empty list with room for
        //       `n` rows; `mut` permits `push`.
        // Why:  One row per registry policy, known up front.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const settings: PolicySetting[] = [];
        // ```
        let mut settings: Vec<PolicySetting> =
            Vec::<PolicySetting>::with_capacity(POLICY_REGISTRY.len());
        // `for ... in` borrows each registry row in order.
        for descriptor in POLICY_REGISTRY {
            settings.push(PolicySetting {
                id: descriptor.id,
                severity: unlisted_severity(descriptor),
                explicit: false,
            });
        }
        return PolicyConfig {
            settings,
            forbidden_strings: ForbiddenStringsOptions {
                builtin_rules: true,
                // `None` is the "absent" variant: no file is configured.
                rules_files: None,
            },
            markdown_autofix: MarkdownAutofixOptions {
                // `vec![...]` is a macro building an owned list from literal items.
                rules: vec![MarkdownRule::LfsImageUrl],
                // `Vec::new()` is the empty owned list.
                exclude: Vec::<String>::new(),
            },
        };
    }

    /// What:
    ///  Read the effective setting of one policy.
    ///  `&self` borrows this record.
    /// Why:
    ///   Callers ask by typed identity instead of scanning the list themselves.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setting(id: PolicyId): PolicySetting;
    /// ```
    pub fn setting(&self, id: PolicyId) -> PolicySetting {
        // `&self.settings` lends the list; `*setting` copies one small row out of it.
        for setting in &self.settings {
            if setting.id == id {
                return *setting;
            }
        }
        // Unreachable while `settings` holds one row per registry policy; fall back to
        // the unlisted severity rather than aborting a Git command.
        return PolicySetting {
            id,
            severity: unlisted_severity(policy_descriptor(id)),
            explicit: false,
        };
    }
}

/// Whole-configuration defaults.
impl CliGitConfig {
    /// What:
    ///  Build the configuration of a repository without `cli-git.config.jsonc`,
    ///  which
    ///       is also what an empty `{}` file produces.
    /// Why:
    ///   Parsing starts from these values and replaces only what the file states.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static defaults(): CliGitConfig;
    /// ```
    pub fn defaults() -> CliGitConfig {
        return CliGitConfig {
            policies: PolicyConfig::defaults(),
            concurrency: ConcurrencyConfig::defaults(),
        };
    }
}

/// What:
///  Translate a configured rule name into its typed rule.
///       `Option<T>` is "value or nothing":
///  `Some(rule)` or `None`.
/// Why:
///   Only shipped rules exist;
///  the caller reports any other name with its key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markdownRuleFromName(name: string): MarkdownRule | undefined;
/// ```
pub fn markdown_rule_from_name(name: &str) -> Option<MarkdownRule> {
    if name == "lfs-image-url" {
        // `Some(...)` is the "present" variant.
        return Some(MarkdownRule::LfsImageUrl);
    }
    // `None` is the "absent" variant.
    return None;
}

/// What:
///  Translate a typed rule back to its configuration spelling.
///       `&'static str` borrows text compiled into the executable for its whole run.
/// Why:
///   Diagnostics and the later linter invocation print the accepted spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markdownRuleName(rule: MarkdownRule): string;
/// ```
pub fn markdown_rule_name(rule: MarkdownRule) -> &'static str {
    // `match` must name every variant, so a new rule cannot be forgotten here.
    match rule {
        MarkdownRule::LfsImageUrl => return "lfs-image-url",
    }
}

/// Value controls stay out of the release executable.
#[cfg(test)]
#[path = "config_schema_tests.rs"]
mod tests;
