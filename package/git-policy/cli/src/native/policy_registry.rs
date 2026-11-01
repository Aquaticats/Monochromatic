//! What:
//!  The fixed registry of policies compiled into the native wrapper.
//! Why:
//!  Repository JSONC can only tune shipped policies;
//!  it can never name code to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const POLICY_REGISTRY = [{ name: 'require-root', defaultSeverity: 'error', ... }] as const;
//! ```

/// What:
///  `enum Severity` is a closed set of three named values.
///       `#[derive(...)]` asks the compiler to generate copying,
///  printing and `==`.
/// Why:
///   A closed set makes an unhandled severity a compile error,
///  unlike a free string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Severity = 'off' | 'warn' | 'error';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    /// The policy does not run.
    Off,
    /// Findings are reported without blocking the Git command.
    Warn,
    /// Findings block the Git command.
    Error,
}

/// What:
///  One variant per shipped policy;
///  the variant is the policy's identity in Rust.
/// Why:
///   Typed identities let options and settings be looked up without string comparison.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicyId = 'require-root' | 'linked-worktree-only' | ... | 'security/forbidden-strings';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyId {
    /// `require-root`:
    ///  repository-root working directory requirement.
    RequireRoot,
    /// `linked-worktree-only`:
    ///  destructive commands stay in linked worktrees.
    LinkedWorktreeOnly,
    /// `branch-worktree-only`:
    ///  branch creation goes through `git worktree add`.
    BranchWorktreeOnly,
    /// `add-explicit`:
    ///  bulk staging patterns are rejected.
    AddExplicit,
    /// `final-newline`:
    ///  text candidates end with one line feed.
    FinalNewline,
    /// `markdown/autofix`:
    ///  commit-time Markdown normalization.
    MarkdownAutofix,
    /// `mono/forbidden-root-context`:
    ///  no root `CONTEXT.md` enters a commit.
    ForbiddenRootContext,
    /// `mono/dependent-version-bump`:
    ///  dependent package versions follow a bumped package.
    DependentVersionBump,
    /// `security/forbidden-strings`:
    ///  candidate bytes are scanned by the bundled scanner.
    ForbiddenStrings,
}

/// What:
///  `struct PolicyDescriptor` is one registry row.
///       `&'static str` is a borrowed string baked into the executable for its whole run
///       (`'static` is that "lives forever" lifetime).
///  Sibling:
///  `String`,
///  an owned copy.
///       `bool` is `true`/`false`,
///  exactly TS `boolean`.
/// Why:
///   Registry text is compiled in and never changes,
///  so borrowing forever is free,
///       while `String` would allocate at startup for no gain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicyDescriptor = { id: PolicyId; name: string; defaultSeverity: Severity;
///   warnSafe: boolean; acceptsOptions: boolean; offUnlessListed: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyDescriptor {
    /// Typed identity used by Rust callers.
    pub id: PolicyId,
    /// Stable configuration key and JSONL `policyId`.
    pub name: &'static str,
    /// The incumbent policy definition's `defaultSeverity`.
    ///  A built-in policy the
    /// repository configuration does not name runs at this severity.
    pub default_severity: Severity,
    /// Whether `warn` keeps the policy's protection (an unsafe `warn` earns a warning event).
    pub warn_safe: bool,
    /// Whether the `["severity", { ... }]` form is accepted for this policy.
    pub accepts_options: bool,
    /// Whether the policy runs only where `cli-git.config.jsonc` names it.
    ///  The four
    /// policies that plugins used to provide are off everywhere else,
    ///  with or without
    /// a configuration file (decided 2026-10-05).
    pub off_unless_listed: bool,
}

/// What:
///  `pub const POLICY_REGISTRY: &[PolicyDescriptor]` is a compiled-in,
///  read-only list.
///       `&[T]` is a borrowed view of an array;
///  siblings are `Vec<T>` (owned,
///  growable)
///       and `[T; N]` (fixed length in the type).
/// Why:
///   Execution and configuration order must be identical on every run;
///  a borrowed
///       constant needs no allocation and cannot be reordered at runtime.
///       Every default is the incumbent policy definition's own `defaultSeverity`.
///       The four formerly plugin-provided policies are additionally off unless the
///       repository configuration names them,
///  because the incumbent only ran them in
///       repositories whose configuration registered their plugin.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const POLICY_REGISTRY: readonly PolicyDescriptor[] = [ ... ];
/// ```
pub const POLICY_REGISTRY: &[PolicyDescriptor] = &[
    PolicyDescriptor {
        id: PolicyId::RequireRoot,
        name: "require-root",
        default_severity: Severity::Error,
        warn_safe: false,
        accepts_options: false,
        off_unless_listed: false,
    },
    PolicyDescriptor {
        id: PolicyId::LinkedWorktreeOnly,
        name: "linked-worktree-only",
        default_severity: Severity::Error,
        warn_safe: false,
        accepts_options: false,
        off_unless_listed: false,
    },
    PolicyDescriptor {
        id: PolicyId::BranchWorktreeOnly,
        name: "branch-worktree-only",
        default_severity: Severity::Error,
        warn_safe: true,
        accepts_options: false,
        off_unless_listed: false,
    },
    PolicyDescriptor {
        id: PolicyId::AddExplicit,
        name: "add-explicit",
        default_severity: Severity::Error,
        warn_safe: false,
        accepts_options: false,
        off_unless_listed: false,
    },
    PolicyDescriptor {
        id: PolicyId::FinalNewline,
        name: "final-newline",
        default_severity: Severity::Warn,
        warn_safe: true,
        accepts_options: false,
        off_unless_listed: false,
    },
    PolicyDescriptor {
        id: PolicyId::MarkdownAutofix,
        name: "markdown/autofix",
        default_severity: Severity::Warn,
        warn_safe: true,
        accepts_options: true,
        off_unless_listed: true,
    },
    PolicyDescriptor {
        id: PolicyId::ForbiddenRootContext,
        name: "mono/forbidden-root-context",
        default_severity: Severity::Error,
        warn_safe: true,
        accepts_options: false,
        off_unless_listed: true,
    },
    PolicyDescriptor {
        id: PolicyId::DependentVersionBump,
        name: "mono/dependent-version-bump",
        default_severity: Severity::Error,
        warn_safe: false,
        accepts_options: false,
        off_unless_listed: true,
    },
    PolicyDescriptor {
        id: PolicyId::ForbiddenStrings,
        name: "security/forbidden-strings",
        default_severity: Severity::Error,
        warn_safe: false,
        accepts_options: true,
        off_unless_listed: true,
    },
];

/// What:
///  Find a registry row by its configuration key.
///       `Option<T>` is Rust's "value or nothing":
///  `Some(row)` or `None`.
///       `&'static PolicyDescriptor` borrows a row that lives as long as the program.
/// Why:
///   An unknown policy ID must be reported,
///  never treated as a new policy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function policyByName(name: string): PolicyDescriptor | undefined;
/// ```
pub fn policy_by_name(name: &str) -> Option<&'static PolicyDescriptor> {
    // What: `usize` is the index type of every Rust list (siblings `u32`, `i64`);
    //       `&POLICY_REGISTRY[index]` borrows one row without copying it.
    // Why:  Nine rows make a linear scan cheaper than building a map, and an indexed
    //       `while` loop needs no callback to stop at the first match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let index = 0; index < POLICY_REGISTRY.length; index += 1) { ... }
    // ```
    let mut index: usize = 0;
    while index < POLICY_REGISTRY.len() {
        let descriptor: &PolicyDescriptor = &POLICY_REGISTRY[index];
        index += 1;
        if descriptor.name == name {
            // What: `Some(descriptor)` wraps the found row in the "present" variant.
            // Why:  The caller must handle absence explicitly; there is no `undefined`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return descriptor;
            // ```
            return Some(descriptor);
        }
    }
    // `None` is the "absent" variant: no shipped policy has this name.
    return None;
}

/// What:
///  Find the registry row for a typed identity.
/// Why:
///   Callers holding a `PolicyId` need its name,
///  default and warn-safety.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function policyDescriptor(id: PolicyId): PolicyDescriptor;
/// ```
pub fn policy_descriptor(id: PolicyId) -> &'static PolicyDescriptor {
    // `for descriptor in POLICY_REGISTRY` borrows each row in order; no index is needed
    // because the row itself is returned, not wrapped in `Some(...)`.
    for descriptor in POLICY_REGISTRY {
        if descriptor.id == id {
            return descriptor;
        }
    }
    // What: `unreachable!` stops the program with this message if ever executed.
    //       The `!` marks a macro (compiler-expanded code), not negation.
    // Why:  Every `PolicyId` variant has a row; the registry test proves it, so
    //       reaching here means the registry itself was edited incorrectly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // throw new Error('unreachable: every PolicyId has a registry row');
    // ```
    unreachable!("every PolicyId variant has a POLICY_REGISTRY row");
}

/// What:
///  Translate a configuration word into a typed severity.
/// Why:
///   Only the three documented spellings are settings;
///  anything else is an error
///       the caller reports with the offending key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function severityFromName(name: string): Severity | undefined;
/// ```
pub fn severity_from_name(name: &str) -> Option<Severity> {
    if name == "off" {
        return Some(Severity::Off);
    }
    if name == "warn" {
        return Some(Severity::Warn);
    }
    if name == "error" {
        return Some(Severity::Error);
    }
    return None;
}

/// What:
///  Translate a typed severity back to its configuration and JSONL spelling.
/// Why:
///   Events and diagnostics print the same words the configuration accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function severityName(severity: Severity): string;
/// ```
pub fn severity_name(severity: Severity) -> &'static str {
    if severity == Severity::Off {
        return "off";
    }
    if severity == Severity::Warn {
        return "warn";
    }
    return "error";
}

/// What:
///  `#[cfg(test)]` compiles the next item only for `cargo test`;
///       `#[path = "..."]` names the sibling file holding the module body.
/// Why:
///   Registry controls stay out of the release executable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // policy_registry.unit.test.ts lives beside this file.
/// ```
#[cfg(test)]
#[path = "policy_registry_tests.rs"]
mod tests;
