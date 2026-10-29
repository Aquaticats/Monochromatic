//! What:
//!  The command modes that do not lint:
//!  `--rules`,
//!  `--init` and `--print-config`.
//! Why:
//!  Each answers a question about the tool or its configuration and must work without
//! reading or rewriting any source file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // rulesListing(); initConfiguration(cwd); printConfiguration(store, cwd, path)
//! ```

/// Import configuration matching,
///  the lookup store and the registry of shipped rule identifiers.
use crate::{
    config_error::ConfigError,
    config_lookup::CONFIG_NAME,
    config_match::{FileConfiguration, PreparedConfiguration},
    configuration_rules::RULE_IDS,
    run_json::strict_json,
    run_paths::{absolute_normal, display_name, relative_from},
    run_plan::ConfigStore,
};
/// Import the JSONC value model that carries merged rules into the printed document.
use monochromatic_jsonc_edit::{JsoncEntry, JsoncKey, JsoncValue};
/// Import the field encoder that renders one rule description as a JSON object.
use serde::Serialize;
/// Import exclusive file creation and native paths.
use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

/// What:
///  A failure that stops the run before or instead of linting,
///  reported with exit status 2.
/// Why:
///  Usage,
///  configuration and discovery errors are not findings about a source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SetupError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetupError {
    /// Explanation printed after the program name on standard error.
    pub message: String,
}

/// What:
///  Construct a setup failure from any displayable cause.
/// Why:
///  Typed configuration and discovery errors keep their own text;
///  this only changes the exit path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SetupError.from(error)
/// ```
impl SetupError {
    /// Copy the cause's rendered message into owned storage.
    pub fn from_display(cause: &dyn std::fmt::Display) -> SetupError {
        return SetupError {
            message: cause.to_string(),
        };
    }
}

/// What:
///  One shipped rule's identifier and documented capabilities.
/// Why:
///  `--rules` output is one JSON object per rule,
///  like findings are one per line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuleInfo = { id: string; fixable: boolean; options: string[]; suppressible: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RuleInfo {
    /// Rule identifier as written in configuration and in a finding's `code`.
    pub id: &'static str,
    /// Whether the rule can attach a fix that `--fix` applies.
    pub fixable: bool,
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
/// const shellRule = (id: string): RuleInfo => ({ id, fixable: false, options: ['max'], suppressible: false });
/// ```
const fn shell_rule(id: &'static str) -> RuleInfo {
    return RuleInfo {
        id,
        fixable: false,
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
        options: &["max"],
        suppressible: false,
    },
    RuleInfo {
        id: "rust/require-rustdoc",
        fixable: false,
        options: &[],
        suppressible: false,
    },
    RuleInfo {
        id: "rust/require-explicit-types",
        fixable: false,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "rust/tests-in-sidecars",
        fixable: false,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "rust/avoidable-closure",
        fixable: false,
        options: &["library"],
        suppressible: false,
    },
    RuleInfo {
        id: "markdown/heading-increment",
        fixable: false,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/commands-show-output",
        fixable: true,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-duplicate-heading",
        fixable: false,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/single-h1",
        fixable: false,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-trailing-punctuation",
        fixable: true,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-bare-urls",
        fixable: true,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-emphasis-as-heading",
        fixable: false,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/fenced-code-language",
        fixable: true,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/link-image-reference-definitions",
        fixable: true,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/link-image-style",
        fixable: true,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/no-pipe-tables",
        fixable: true,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/semantic-line-breaks",
        fixable: true,
        options: &[],
        suppressible: true,
    },
    RuleInfo {
        id: "markdown/lfs-image-url",
        fixable: true,
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

/// What:
///  Render the rule listing as JSON Lines.
/// Why:
///  The registry order is kept,
///  and an identifier the registry accepts but this table lacks is
/// an error instead of a silently shorter listing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rulesListing(): string;
/// ```
pub fn rules_listing() -> Result<String, SetupError> {
    let mut output: String = String::new();
    for id in RULE_IDS {
        let mut described: Option<&RuleInfo> = None;
        for rule in RULES {
            if rule.id == *id {
                described = Some(rule);
            }
        }
        let Some(rule): Option<&RuleInfo> = described else {
            return Err(SetupError {
                message: format!("Rule {id} is registered but has no --rules description."),
            });
        };
        match serde_json::to_string(rule) {
            Ok(line) => output.push_str(line.as_str()),
            Err(error) => return Err(SetupError::from_display(&error)),
        }
        output.push('\n');
    }
    return Ok(output);
}

/// What:
///  The configuration `--init` writes.
/// Why:
///  It selects the syntax-only Rust rules and every Markdown rule for all files,
///  as a
/// starting point;
///  `rust/require-explicit-types` is left out because it needs a Cargo workspace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STARTER_CONFIGURATION = '[ { "files": ["**\/*.rs"], "rules": { ... } }, ... ]';
/// ```
pub const STARTER_CONFIGURATION: &str = r#"// monochromatic-lint.config.jsonc
// Blocks apply in order. A rule runs only where a block turns it on.
[
  {
    "name": "rust",
    "files": ["**/*.rs"],
    "rules": {
      "rust/max-lines": { "severity": "error", "max": 300 },
      "rust/require-rustdoc": { "severity": "error" },
      "rust/tests-in-sidecars": { "severity": "error" }
    }
  },
  {
    "name": "markdown",
    "files": ["**/*.md", "**/*.mdx"],
    "rules": {
      "markdown/heading-increment": { "severity": "error" },
      "markdown/commands-show-output": { "severity": "error" },
      "markdown/no-duplicate-heading": { "severity": "error" },
      "markdown/single-h1": { "severity": "error" },
      "markdown/no-trailing-punctuation": { "severity": "error" },
      "markdown/no-bare-urls": { "severity": "error" },
      "markdown/no-emphasis-as-heading": { "severity": "error" },
      "markdown/fenced-code-language": { "severity": "error" },
      "markdown/link-image-reference-definitions": { "severity": "error" },
      "markdown/link-image-style": { "severity": "error" },
      "markdown/no-pipe-tables": { "severity": "error" },
      "markdown/semantic-line-breaks": { "severity": "error" },
      "markdown/lfs-image-url": { "severity": "error" }
    }
  }
]
"#;

/// What:
///  Create the starter configuration in the working directory,
///  refusing to overwrite.
/// Why:
///  `create_new` fails when the file exists,
///  so an existing configuration is never replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initConfiguration(cwd: Path): Path; // throws when the file exists
/// ```
pub fn init_configuration(cwd: &Path) -> Result<PathBuf, SetupError> {
    let path: PathBuf = cwd.join(CONFIG_NAME);
    let mut file: std::fs::File = match OpenOptions::new().write(true).create_new(true).open(&path)
    {
        Ok(handle) => handle,
        Err(error) => {
            return Err(SetupError {
                message: format!(
                    "Cannot create {}: {error}. --init never overwrites an existing configuration.",
                    path.display()
                ),
            });
        }
    };
    if let Err(error) = file.write_all(STARTER_CONFIGURATION.as_bytes()) {
        return Err(SetupError {
            message: format!("Cannot write {}: {error}.", path.display()),
        });
    }
    return Ok(path);
}

/// What:
///  Build one string-valued record member.
/// Why:
///  The emitter quotes and escapes the text,
///  so paths with quotes or control characters stay valid JSONC.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function member(key: string, value: string): JsoncEntry;
/// ```
fn member(key: &str, value: &str) -> JsoncEntry {
    return JsoncEntry {
        key: JsoncKey::from_text(key),
        value: JsoncValue::text_from_units(value.encode_utf16().collect::<Vec<u16>>()),
    };
}

/// What:
///  Print the effective configuration for one real or virtual path,
///  without linting it.
/// Why:
///  The answer distinguishes four states:
///  no configuration file,
///  ignored,
///  matched by no
/// block,
///  and configured with the merged rules shown.
///  The document is strict JSON,
///  so tools that
/// reject trailing commas and comments can read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function printConfiguration(store: ConfigStore, cwd: Path, path: Path): string;
/// ```
pub fn print_configuration(
    store: &mut ConfigStore,
    cwd: &Path,
    path: &Path,
) -> Result<String, ConfigError> {
    let absolute: PathBuf = absolute_normal(path, cwd);
    let mut entries: Vec<JsoncEntry> = vec![member("file", display_name(&absolute, cwd).as_str())];
    let found: Option<Arc<PreparedConfiguration>> = store.configuration_for(&absolute)?;
    if let Some(config) = found {
        entries.push(member(
            "configuration",
            config.path.to_string_lossy().as_ref(),
        ));
        entries.push(member("base", config.base.to_string_lossy().as_ref()));
        match config.resolve(&relative_from(&config.base, &absolute))? {
            FileConfiguration::Ignored => entries.push(member("state", "ignored")),
            FileConfiguration::Unconfigured => entries.push(member("state", "unconfigured")),
            FileConfiguration::Configured { rules } => {
                entries.push(member("state", "configured"));
                entries.push(JsoncEntry {
                    key: JsoncKey::from_text("rules"),
                    value: rules,
                });
            }
        }
    } else {
        entries.push(member("state", "no-configuration"));
    }
    return Ok(strict_json(&JsoncValue::record(entries)));
}

/// Listing,
///  starter and effective-configuration controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_modes_tests.rs"]
mod tests;
