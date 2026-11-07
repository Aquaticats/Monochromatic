//! What:
//!  The table of every shipped rule's identifier and documented capabilities,
//!  with the lookup by identifier.
//! Why:
//!  `--rules` prints it,
//!  and the directive engine and the fixer ask it whether a rule may be suppressed and which class its fix has,
//!  so the capabilities are data that a test checks against the configuration registry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const RULES: RuleInfo[]; export function ruleInfo(id: string): RuleInfo | undefined;
//! ```

/// Import the class a rule's fix declares.
use crate::edits::FixClass;
/// Import the serializer that prints one rule per JSON line.
use serde::Serialize;

/// What:
///  One shipped rule's identifier and documented capabilities.
/// Why:
///  `--rules` output is one JSON object per rule,
///  like findings are one per line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuleInfo = { id: string; fixable: boolean; fixClass: 'whitespace' | 'content' | null; options: string[]; suppressible: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RuleInfo {
    /// Rule identifier as written in configuration and in a finding's `code`.
    pub id: &'static str,
    /// Whether the rule can attach a fix that `--fix-content` applies.
    pub fixable: bool,
    /// What the rule's fix does to the reader's text:
    ///  `whitespace` applies under plain `--fix`,
    ///  `content` only under `--fix-content`,
    ///  and a rule without a fix has none.
    #[serde(rename = "fixClass")]
    pub fix_class: Option<FixClass>,
    /// Option names accepted besides `severity`.
    pub options: &'static [&'static str],
    /// Whether an inline `monochromatic-lint-disable` family directive may suppress this rule's findings.
    ///  The rule declares this here;
    ///  no configuration key,
    ///  override or nested configuration can change it.
    pub suppressible: bool,
}

/// What:
///  The listing entry of a `shell/` rule.
/// Why:
///  All nine take only `max`,
///  have no fix,
///  and refuse inline directives:
///  their exemptions are configuration ratchets,
///  so one constructor keeps the nine entries identical.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shellRule = (id: string): RuleInfo => ({ id, fixable: false, fixClass: null, options: ['max'], suppressible: false });
/// ```
const fn shell_rule(id: &'static str) -> RuleInfo {
    return RuleInfo {
        id,
        fixable: false,
        fix_class: None,
        options: &["max"],
        suppressible: false,
    };
}

/// What:
///  Capabilities of every shipped rule,
///  in registry order.
/// Why:
///  The listing is data,
///  checked against the configuration registry by a test,
///  so a rule
/// cannot be accepted by configuration yet missing from `--rules`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RULES: RuleInfo[] = [/* ... */];
/// ```
pub const RULES: &[RuleInfo] = &[
    RuleInfo {
        id: "rust/max-lines",
        fixable: false,
        fix_class: None,
        options: &["max"],
        suppressible: false,
    },
    RuleInfo {
        id: "rust/require-rustdoc",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: false,
    },
    RuleInfo {
        id: "rust/require-explicit-types",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "rust/tests-in-sidecars",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "rust/avoidable-closure",
        fixable: false,
        fix_class: None,
        options: &["library"],
        suppressible: false,
    },
    RuleInfo {
        id: "rust/folder-uses-links",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: false,
    },
    RuleInfo {
        id: "rust/file-order-in-folder",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: false,
    },
    RuleInfo {
        id: "markdown/heading-increment",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/commands-show-output",
        fixable: true,
        fix_class: Some(FixClass::Content),
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-duplicate-heading",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/single-h1",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-trailing-punctuation",
        fixable: true,
        fix_class: Some(FixClass::Content),
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-bare-urls",
        fixable: true,
        fix_class: Some(FixClass::Content),
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-emphasis-as-heading",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/fenced-code-language",
        fixable: true,
        fix_class: Some(FixClass::Content),
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/link-image-reference-definitions",
        fixable: true,
        fix_class: Some(FixClass::Content),
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/link-image-style",
        fixable: true,
        fix_class: Some(FixClass::Content),
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-pipe-tables",
        fixable: false,
        fix_class: None,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/semantic-line-breaks",
        fixable: true,
        fix_class: Some(FixClass::Whitespace),
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/lfs-image-url",
        fixable: true,
        fix_class: Some(FixClass::Content),
        options: &["exclude"],
        suppressible: true,
    },
    shell_rule("shell/max-lines"),
    shell_rule("shell/no-pipe"),
    shell_rule("shell/no-and-chain"),
    shell_rule("shell/no-or-chain"),
    shell_rule("shell/no-pipe-stderr"),
    shell_rule("shell/no-semicolon"),
    shell_rule("shell/no-dollar-paren"),
    shell_rule("shell/no-backtick"),
    shell_rule("shell/no-ampersand"),
];

/// What:
///  Find one shipped rule's description by the identifier a finding carries in `code`.
/// Why:
///  The directive engine asks the rule's own declaration whether it may be suppressed,
///  and treats an identifier outside the registry as unknown,
///  which keeps `core/` and `directive/` findings unsuppressible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ruleInfo(id: string): RuleInfo | undefined;
/// ```
pub fn rule_info(id: &str) -> Option<&'static RuleInfo> {
    for rule in RULES {
        if rule.id == id {
            return Some(rule);
        }
    }
    return None;
}
