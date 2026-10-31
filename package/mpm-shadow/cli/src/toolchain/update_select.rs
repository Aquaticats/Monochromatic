//! What:
//! Which plan tools a named `update` resolves:
//! the patterns typed after `update`,
//! the tools they select,
//! and the tools the
//!           lock cannot hold that the names left out.
//! Why:
//! `pnpm update <pkg>...` moves only the named dependencies and keeps every other lockfile entry,
//! and `update <tool>...` mirrors it;
//! the same selection backs `update --check <tool>...`.
//! An unknown name is refused (pnpm 12.16.1 silently reports "Already up to date" for it,
//! which hides a typo) with the valid ids listed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-select.ts
//! export function selectTools(plan: Plan, patterns: string[]): Set<string>  // throws on an unknown pattern
//! export function unnamedNeedingResolution(plan: Plan, lock: Lock | undefined, selected: Set<string>): string[]
//! ```

/// Imports `BTreeSet`,
/// a sorted set.
use std::collections::BTreeSet;

/// Imports what this module uses from `super::agree`.
use super::agree::Disagreement;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `Plan` from `super::plan`.
use super::plan::Plan;

/// What:
/// Whether `text` matches `pattern`,
/// where `*` stands for any run of characters (including none) and every other character stands for itself.
/// Why:
/// pnpm selects packages with patterns such as `@babel/*`;
/// tool ids contain `:` and `/`,
/// which `*` crosses.
/// One pass over each text with a single remembered `*` position,
/// so the work is bounded by the lengths of both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function matchesPattern(pattern: string, text: string): boolean
/// ```
pub fn matches_pattern(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    let mut pattern_index: usize = 0;
    let mut text_index: usize = 0;
    let mut star_pattern_index: Option<usize> = None;
    let mut star_text_index: usize = 0;
    while text_index < text_chars.len() {
        if pattern_index < pattern_chars.len() && pattern_chars[pattern_index] == '*' {
            star_pattern_index = Some(pattern_index);
            star_text_index = text_index;
            pattern_index += 1;
        } else if pattern_index < pattern_chars.len() && pattern_chars[pattern_index] == text_chars[text_index] {
            pattern_index += 1;
            text_index += 1;
        } else if let Some(star) = star_pattern_index {
            // The last `*` absorbs one more character and the match resumes after it.
            pattern_index = star + 1;
            star_text_index += 1;
            text_index = star_text_index;
        } else {
            return false;
        }
    }
    while pattern_index < pattern_chars.len() && pattern_chars[pattern_index] == '*' {
        pattern_index += 1;
    }
    return pattern_index == pattern_chars.len();
}

/// The ids of the plan's tools that match one pattern,
/// in plan order.
fn ids_matching(plan: &Plan, pattern: &str) -> Vec<String> {
    let mut matched: Vec<String> = Vec::new();
    for declaration in &plan.tools {
        if matches_pattern(pattern, &declaration.id) {
            matched.push(declaration.id.clone());
        }
    }
    return matched;
}

/// What:
/// The plan tools the `update` arguments select.
/// A pattern selects the tools whose id it matches;
/// a pattern with a leading `!` removes the tools it matches from the selection
/// (only exclusions select everything else);
/// no pattern at all selects every tool.
/// Why:
/// This is pnpm's `pnpm update "@babel/*" "!@babel/core"`.
/// A pattern that matches no plan tool is refused before any lookup,
/// naming it and listing every valid id,
/// because a misspelled name would otherwise update nothing and look like success.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function selectTools(plan: Plan, patterns: string[]): Set<string>
/// ```
pub fn select_tools(plan: &Plan, patterns: &[String]) -> Result<BTreeSet<String>, ToolchainError> {
    let mut included: BTreeSet<String> = BTreeSet::new();
    let mut excluded: BTreeSet<String> = BTreeSet::new();
    let mut has_inclusion: bool = false;
    let mut unmatched: Vec<String> = Vec::new();
    for pattern in patterns {
        let (negated, body): (bool, &str) = match pattern.strip_prefix('!') {
            Some(rest) => (true, rest),
            None => (false, pattern.as_str()),
        };
        let matched: Vec<String> = ids_matching(plan, body);
        if matched.is_empty() {
            unmatched.push(pattern.clone());
            continue;
        }
        if negated {
            excluded.extend(matched);
        } else {
            has_inclusion = true;
            included.extend(matched);
        }
    }
    if !unmatched.is_empty() {
        let mut valid: Vec<String> = Vec::new();
        for declaration in &plan.tools {
            valid.push(declaration.id.clone());
        }
        return Err(ToolchainError::config(format!(
            "update names no tool of the plan: {}; the plan's tool ids are: {}",
            unmatched.join(", "),
            valid.join(", ")
        )));
    }
    if !has_inclusion {
        for declaration in &plan.tools {
            included.insert(declaration.id.clone());
        }
    }
    for id in &excluded {
        included.remove(id);
    }
    return Ok(included);
}

/// What:
/// The plan tools `update <tool>...` did not select that the lock cannot hold as they are:
/// no lock row (new, or the lock does not exist),
/// or a resolution-affecting edit in the plan.
/// Why:
/// Keeping their rows would write a lock that fails `lock --check`,
/// and resolving them unasked would move what the caller did not name,
/// so the command refuses and names them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function unnamedNeedingResolution(plan: Plan, disagreement: Disagreement | undefined, selected: Set<string>): string[]
/// ```
pub fn unnamed_needing_resolution(plan: &Plan, disagreement: Option<&Disagreement>, selected: &BTreeSet<String>) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for declaration in &plan.tools {
        if selected.contains(&declaration.id) {
            continue;
        }
        let needs: bool = match disagreement {
            None => true,
            Some(found) => found.added.contains(&declaration.id) || found.resolution_edits.iter().any(|(id, _)| return *id == declaration.id),
        };
        if needs {
            ids.push(declaration.id.clone());
        }
    }
    return ids;
}

/// What:
/// The refusal for a named `update` that left out tools the lock cannot hold.
/// Why:
/// It tells the caller both ways forward:
/// name those tools too,
/// or update everything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function leftOutRefusal(ids: string[]): ToolchainError
/// ```
pub fn left_out_refusal(ids: &[String]) -> ToolchainError {
    return ToolchainError::drift(format!(
        "update was given tool names but the plan also has entries the lock cannot hold as they are (new, or edited in a way \
         that moves their version or artifacts): {}; add them to the names (`mpm-shadow update ... {}`) or run `mpm-shadow update` \
         with no names",
        ids.join(", "),
        ids.join(" ")
    ));
}
