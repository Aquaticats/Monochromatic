//! What:
//!  Decode a JSON value into the typed `Config`, checking ids, scope and cross-rule facts.
//! Why:
//!  The decoder is hand-written so every message names the pointer and the remedy; the committed JSON Schema is
//!  checked against it by a corpus test, so the two cannot drift unnoticed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function decodeConfig(value: unknown, scope: ConfigScope, displayPath: string): Config; // throws Diagnostic[]
//! ```

use crate::config_decode_rules::decode_rule;
use crate::config_reader::Reader;
use crate::config_types::{Config, RuleConfig};
use crate::diagnostic::Diagnostic;
use serde_json::Value;
use std::collections::BTreeSet;

/// What:
///  Which config file is being decoded: the repository's or the per-user one.
/// Why:
///  Rules that write outside the repository belong only in the per-user config; the repository config rejects them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigScope = 'repository' | 'user';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigScope {
    /// `file-enforcer.config.jsonc` at the worktree root.
    Repository,
    /// The per-user config under the user's configuration directory.
    User,
}

/// What:
///  Check that a repo-relative path is relative, normalized and stays inside the root.
///  `&str` is a borrowed text view.
/// Why:
///  Every output path is checked again at mutation time, but a bad path should fail at load time with its pointer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isRepoRelative(path: string): boolean
/// ```
pub fn is_repo_relative(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return false;
    }
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return false;
        }
    }
    return true;
}

/// What:
///  Check that a mode string is three or four octal digits.
/// Why:
///  JSON has no octal literals, so modes are strings such as `0600`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isOctalMode(mode: string): boolean { return /^[0-7]{3,4}$/.test(mode); }
/// ```
pub fn is_octal_mode(mode: &str) -> bool {
    if mode.len() < 3 || mode.len() > 4 {
        return false;
    }
    return mode.bytes().all(|byte| return (b'0'..=b'7').contains(&byte));
}

/// What:
///  Decode the whole config, collecting every problem.
///  `Result<Config, Vec<Diagnostic>>` is TS's "value or thrown list".
/// Why:
///  People fix all the mistakes in one pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeConfig(value: unknown, scope: ConfigScope, displayPath: string): Config
/// ```
pub fn decode_config(value: &Value, scope: ConfigScope, display_path: &str) -> Result<Config, Vec<Diagnostic>> {
    let mut reader = Reader::new(display_path);
    let mut rules: Vec<RuleConfig> = Vec::new();
    let mut schema: Option<String> = None;
    let mut seen_ids: BTreeSet<String> = BTreeSet::new();
    if let Some(object) = reader.object(value, "") {
        reader.reject_unknown(object, "", &["$schema", "rules"]);
        schema = reader.optional_text(object, "$schema", "");
        let items = reader.array(object, "rules", "");
        for (index, item) in items.iter().enumerate() {
            let pointer = format!("/rules/{index}");
            if let Some(rule) = decode_rule(&mut reader, item, &pointer) {
                check_scope(&mut reader, &rule, scope, &pointer);
                if !seen_ids.insert(rule.id().to_string()) {
                    let message = format!("the id {:?} is used by an earlier rule", rule.id());
                    reader.error(&format!("{pointer}/id"), &message);
                }
                rules.push(rule);
            }
        }
    }
    if !reader.errors.is_empty() {
        return Err(reader.errors);
    }
    return Ok(Config { schema, rules });
}

/// What:
///  Reject a rule kind that does not belong in this config file.
/// Why:
///  The per-user config holds only rules that write outside the repository, and the repository config holds none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkScope(rule: RuleConfig, scope: ConfigScope, pointer: string): void
/// ```
fn check_scope(reader: &mut Reader, rule: &RuleConfig, scope: ConfigScope, pointer: &str) {
    let is_user_kind = matches!(rule, RuleConfig::JetbrainsHarper { .. });
    if scope == ConfigScope::Repository && is_user_kind {
        let message = format!("the kind {} writes outside the repository; put it in the per-user config", rule.kind());
        reader.error(pointer, &message);
    }
    if scope == ConfigScope::User && !is_user_kind {
        let message = format!("the kind {} belongs in the repository config, not the per-user config", rule.kind());
        reader.error(pointer, &message);
    }
}

/// What:
///  Convert a validated octal mode string such as `0600` into permission bits.
/// Why:
///  Rules store the text the config holds; publishing needs the number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMode(mode: string): number { return parseInt(mode, 8); }
/// ```
pub fn parse_mode(mode: &str) -> u32 {
    return u32::from_str_radix(mode, 8).unwrap_or(0o644);
}
