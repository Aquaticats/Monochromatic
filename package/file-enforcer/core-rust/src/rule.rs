//! What:
//!  The rule-kind interface:
//! what a rule plans,
//! and how a config entry becomes a rule.
//! Why:
//!  The dependency-catalog library,
//! the browserslist crate and the JetBrains XML crate are built by other
//!  agents;
//! each implements `Rule` and `RuleBuilder` and registers the builder under its kind in a `RuleRegistry`.
//!  The engine never needs to know them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface Rule { id(): string; composes(): boolean; plan(view: View): RulePlan }
//! interface RuleBuilder { build(config: RuleConfig): Rule }
//! ```

/// Imports `RuleConfig` from `crate::config_types`.
use crate::config_types::RuleConfig;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `View` from `crate::view`.
use crate::view::View;

/// What:
///  One effect a rule wants on the worktree.
///  Enum variants with named fields are tagged records (TS:
/// a discriminated union).
/// Why:
///  Rules describe effects;
/// only the publish step performs them,
/// so a failing run writes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Change = { kind: 'write'; path: string; content: Uint8Array; mode?: number } | { kind: 'remove'; path: string } | { kind: 'setMode'; path: string; mode: number } | { kind: 'removeDirectory'; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Change {
    /// Make the file hold this content;
    /// `mode` of `None` keeps an existing file's mode.
    Write {
        /// Repo-relative path.
        path: String,
        /// Content to publish.
        content: Vec<u8>,
        /// Permission bits to set,
        /// if the rule owns them.
        mode: Option<u32>,
    },
    /// Delete a file (a link is removed as a link).
    Remove {
        /// Repo-relative path.
        path: String,
    },
    /// Change a file's permission bits;
    /// a no-op when the file does not exist.
    SetMode {
        /// Repo-relative path.
        path: String,
        /// Permission bits.
        mode: u32,
    },
    /// Delete a directory that the run emptied.
    RemoveDirectory {
        /// Repo-relative path.
        path: String,
    },
}

/// What:
///  The result of planning one rule:
/// its changes and any warnings.
/// Why:
///  A rule that cannot run in this build (a kind whose library is not linked) returns no changes and a warning,
///  which is not drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RulePlan = { changes: Change[]; warnings: Diagnostic[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RulePlan {
    /// Effects,
    /// in the order they apply.
    pub changes: Vec<Change>,
    /// Warnings to report without failing the run.
    pub warnings: Vec<Diagnostic>,
}

/// What:
///  A planned rule.
///  A trait is an interface;
/// `dyn Rule` values are chosen at run time.
///  `Send + Sync` lets the server share rules across its worker threads.
/// Why:
///  Every kind,
/// built in or linked from another crate,
/// plans through the same method.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Rule { id(): string; kind(): string; composes(): boolean; plan(view: View): RulePlan }
/// ```
pub trait Rule: Send + Sync {
    /// The rule's unique id from the config.
    fn id(&self) -> &str;

    /// The kind tag from the config.
    fn kind(&self) -> &str;

    /// Whether this rule transforms a file in place and so may share an output with other composing rules.
    /// Rules that own their outputs outright return `false` (the default);
    /// two writers of one path fail the run
    /// unless both compose.
    fn composes(&self) -> bool {
        return false;
    }

    /// Compute the changes from what the view shows.
    /// Read only through the view:
    /// every read is recorded for
    /// revalidation,
    /// for `watch` and for `explain`.
    /// Return `Err` for a failure that stops the run.
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic>;
}

/// What:
///  Turns a decoded config entry into a rule.
/// Why:
///  A registry maps each kind tag to a builder,
/// so a library linked later can supply a kind without editing the engine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RuleBuilder { build(config: RuleConfig): Rule }
/// ```
pub trait RuleBuilder: Send + Sync {
    /// Build the rule for `config`,
    /// or explain why it cannot be built.
    fn build(&self, config: &RuleConfig) -> Result<Box<dyn Rule>, Diagnostic>;
}
