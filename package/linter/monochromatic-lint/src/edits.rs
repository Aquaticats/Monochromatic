//! What:
//!  Atomic application of grouped,
//!  byte-addressed source fixes.
//! Why:
//!  Competing findings may overlap,
//!  but a multi-edit fix must never be applied only partly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // applyFixes(source, fixes) chooses whole compatible fixes before producing any output.
//! ```

/// Import the ordered interval lookup used to detect conflicts.
use std::collections::BTreeMap;

/// What:
///  One replacement of a half-open UTF-8 byte range.
/// Why:
///  The original and replacement strings stay owned by their callers;
///  offsets never use UTF-16 indexing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Edit = { start: number; end: number; replacement: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Edit {
    /// Inclusive byte offset.
    pub start: usize,
    /// Exclusive byte offset;
    ///  equal to start for insertion.
    pub end: usize,
    /// Owned replacement source.
    pub replacement: String,
}

/// What:
///  The class of one fix,
///  in the terms Oxlint's `--help` and its `FixKind` use.
/// Why:
///  The owner settled on 2026-11-15 that this command follows Oxlint's `--fix`,
///  and Oxlint's own source at tag `oxlint_v1.93.0`
///  (`crates/oxc_linter/src/fixer/fix.rs`)
///  defines `Fix`,
///  `Suggestion` and `Dangerous`,
///  with `can_apply` asking whether the run's allowed set contains the rule's class.
///  A rule declares the class beside the edit it builds,
///  so what a fix may change is read where the fix is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FixClass = 'fix' | 'suggestion' | 'dangerous_fix' | 'dangerous_suggestion';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FixClass {
    /// A safe fix:
    ///  it changes no word a reader reads and no link or image a reader follows.
    ///  Plain `--fix` applies it.
    Fix,
    /// A suggestion:
    ///  it changes what a reader sees (a heading's words,
    ///  a listing's lines,
    ///  a fence's label,
    ///  an image's destination)
    ///  while keeping the document valid.
    ///  `--fix-suggestions` applies it.
    Suggestion,
    /// A dangerous fix:
    ///  it deletes authored source,
    ///  so it may break the document if the analysis behind it is wrong.
    ///  `--fix-dangerously` applies it.
    DangerousFix,
    /// A dangerous suggestion:
    ///  it deletes authored source and changes what a reader sees.
    ///  `--fix-dangerously` applies it.
    DangerousSuggestion,
}

/// The `Fix` bit of Oxlint's `FixKind`.
const FIX: u8 = 1;
/// The `Suggestion` bit of Oxlint's `FixKind`.
const SUGGESTION: u8 = 2;
/// The `Dangerous` bit of Oxlint's `FixKind`,
///  which always qualifies a fix or a suggestion.
const DANGEROUS: u8 = 4;

/// What:
///  The bits this class sets,
///  spelled as Oxlint's `FixKind` composites spell them.
/// Why:
///  A scope's `allows` is a set containment,
///  so the bits are what decides,
///  not a list of names kept in two places.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bits = { fix: 1, suggestion: 2, dangerousFix: 5, dangerousSuggestion: 6 };
/// ```
impl FixClass {
    /// The bit set of this class.
    pub const fn bits(self) -> u8 {
        return match self {
            Self::Fix => FIX,
            Self::Suggestion => SUGGESTION,
            Self::DangerousFix => DANGEROUS | FIX,
            Self::DangerousSuggestion => DANGEROUS | SUGGESTION,
        };
    }
}

/// What:
///  The classes a run may apply,
///  built from the three fix flags as Oxlint builds them.
/// Why:
///  `--fix` allows a safe fix,
///  `--fix-suggestions` allows a suggestion alone,
///  and `--fix-dangerously` allows every class,
///  so a flag combination means here what it means in Oxlint 1.93.0.
///  A finding whose fix is outside the scope is still reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FixScope { allows(fixClass: FixClass): boolean }
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FixScope {
    /// The union of the classes the fix flags selected,
    ///  with the dangerous bit only when `--fix-dangerously` was given.
    bits: u8,
}

/// What:
///  The scope's question about one fix,
///  and the two scopes that do not come from a flag.
/// Why:
///  A checker consults the scope and nothing else,
///  so the flags are read once per run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scope.allows(fix.class);
/// ```
impl FixScope {
    /// No fix flag was given:
    ///  every finding is reported and nothing is applied.
    pub const NONE: Self = Self { bits: 0 };

    /// Every class,
    ///  which `--fix-dangerously` selects on its own and which the in-process entry always uses.
    pub const ALL: Self = Self {
        bits: DANGEROUS | FIX | SUGGESTION,
    };

    /// Build the scope the three fix flags select,
    ///  as `FixOptions::fix_kind` does at tag `oxlint_v1.93.0`
    ///  (`apps/oxlint/src/command/lint.rs`).
    pub const fn from_flags(fix: bool, fix_suggestions: bool, fix_dangerously: bool) -> Self {
        let mut bits: u8 = 0;
        if fix {
            bits |= FIX;
        }
        if fix_suggestions {
            bits |= SUGGESTION;
        }
        if fix_dangerously {
            bits |= DANGEROUS | FIX | SUGGESTION;
        }
        return Self { bits };
    }

    /// Whether a fix of this class may be applied in this scope.
    pub const fn allows(self, class: FixClass) -> bool {
        return self.bits & class.bits() == class.bits();
    }
}

/// What:
///  All edits supplied by one finding,
///  with the class of change they make.
/// Why:
///  This group is the unit accepted or rejected when edits conflict,
///  and the class decides whether a run may apply it at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fix = { class: FixClass; edits: Edit[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fix {
    /// The class of change the edits make,
    ///  which decides whether a run may apply them.
    pub class: FixClass,
    /// Related replacements that must apply together.
    pub edits: Vec<Edit>,
}

/// What:
///  An invalid edit plan or a refused empty rewrite.
/// Why:
///  Rule failures become processing findings rather than partially changed files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FixError extends Error { readonly message: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixError {
    /// Operational reason that no output was accepted.
    pub message: String,
}

/// What:
///  Result of selecting compatible whole fixes.
/// Why:
///  The caller can distinguish conflicts from actual changes before writing atomically.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AppliedFixes = { source: string; applied: number[]; rejected: number[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedFixes {
    /// New source;
    ///  the input string itself is never modified.
    pub source: String,
    /// Original finding indexes whose entire fix was selected.
    pub applied: Vec<usize>,
    /// Original finding indexes rejected for conflict with an earlier selected fix.
    pub rejected: Vec<usize>,
}

/// Render edit failures through the ordinary error interface.
impl std::fmt::Display for FixError {
    /// Borrow the formatter only while writing the explanation.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.message.as_str());
    }
}

/// Integrate processing failures with application error handling.
impl std::error::Error for FixError {}

/// What:
///  Compare a candidate interval with its nearest earlier selected interval.
/// Why:
///  Ordered neighbors are sufficient;
///  no scan over every earlier fix is necessary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conflictsBefore(edit: Edit, accepted: OrderedMap<number, Edit>): boolean;
/// ```
fn conflicts_before(edit: &Edit, accepted: &BTreeMap<usize, &Edit>) -> bool {
    let Some((start, previous)) = accepted.range(..=edit.start).next_back() else {
        return false;
    };
    // Equal starts also reject competing zero-width insertions whose order would otherwise be ambiguous.
    return *start == edit.start || previous.end > edit.start;
}

/// What:
///  Compare a candidate interval with its nearest later selected interval.
/// Why:
///  A replacement must not cover an already accepted insertion or replacement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conflictsAfter(edit: Edit, accepted: OrderedMap<number, Edit>): boolean;
/// ```
fn conflicts_after(edit: &Edit, accepted: &BTreeMap<usize, &Edit>) -> bool {
    let Some((start, _)) = accepted.range(edit.start..).next() else {
        return false;
    };
    // Equal-start conflicts were already rejected by conflicts_before.
    return *start < edit.end;
}

/// What:
///  Validate a fix and return its edit indexes in source order.
/// Why:
///  Owned indexes avoid tying a returned reference to either borrowed input.
/// Invalid ranges or internal overlap must not reach unchecked string slicing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function orderedEditIndexes(source: string, fix: Fix): number[];
/// ```
fn ordered_edit_indexes(source: &str, fix: &Fix) -> Result<Vec<usize>, FixError> {
    // What: `bounds` is a growable list of triples,
    //  one per edit: its start offset,
    //  its end offset and its position in `fix.edits`.
    // Why: sorting the triples orders the edits by source bounds with no closure borrowing `fix`,
    //  and the trailing position keeps equal bounds in their original order,
    //  exactly as the earlier stable sort by `(start, end)` did.
    let mut bounds: Vec<(usize, usize, usize)> = Vec::new();
    for (index, edit) in fix.edits.iter().enumerate() {
        // What: `(edit.start, edit.end, index)` is one triple of the sort.
        bounds.push((edit.start, edit.end, index));
    }
    // What: `sort()` orders the triples lexicographically,
    //  so start first,
    //  then end,
    //  then position.
    bounds.sort();
    // What: `ordered` keeps just the third element of every triple:
    //  the edit indexes in source order.
    let mut ordered: Vec<usize> = Vec::new();
    for entry in bounds {
        ordered.push(entry.2);
    }
    for edit in &fix.edits {
        if edit.start > edit.end
            || edit.end > source.len()
            || !source.is_char_boundary(edit.start)
            || !source.is_char_boundary(edit.end)
        {
            return Err(FixError {
                message: String::from(
                    "A fix supplied an out-of-range or non-UTF-8-boundary source edit.",
                ),
            });
        }
    }
    for pair in ordered.windows(2) {
        let previous = &fix.edits[pair[0]];
        let current = &fix.edits[pair[1]];
        if previous.end > current.start || previous.start == current.start {
            return Err(FixError {
                message: String::from("A fix supplied overlapping edits within its own group."),
            });
        }
    }
    return Ok(ordered);
}

/// What:
///  Select whole fixes in finding order,
///  then construct output in one forward source walk.
/// Why:
///  Selection is independent from writing,
///  so a late conflict cannot leave half a fix applied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyFixes(source: string, fixes: readonly Fix[]): AppliedFixes;
/// ```
pub fn apply_fixes(source: &str, fixes: &[Fix]) -> Result<AppliedFixes, FixError> {
    let mut accepted: BTreeMap<usize, &Edit> = BTreeMap::new();
    let mut applied = Vec::new();
    let mut rejected = Vec::new();
    for (index, fix) in fixes.iter().enumerate() {
        let ordered = ordered_edit_indexes(source, fix)?;
        let mut conflict = false;
        for edit_index in &ordered {
            let edit = &fix.edits[*edit_index];
            if conflicts_before(edit, &accepted) || conflicts_after(edit, &accepted) {
                conflict = true;
                break;
            }
        }
        if conflict {
            rejected.push(index);
            continue;
        }
        // Insert only after checking the complete group against previously selected fixes.
        for edit_index in ordered {
            let edit = &fix.edits[edit_index];
            accepted.insert(edit.start, edit);
        }
        applied.push(index);
    }
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for edit in accepted.values() {
        output.push_str(&source[cursor..edit.start]);
        output.push_str(edit.replacement.as_str());
        cursor = edit.end;
    }
    output.push_str(&source[cursor..]);
    if !source.is_empty() && output.is_empty() {
        return Err(FixError {
            message: String::from(
                "Autofix would replace non-empty file with empty output; leaving file unchanged.",
            ),
        });
    }
    return Ok(AppliedFixes {
        source: output,
        applied,
        rejected,
    });
}

/// Keep atomic-fix regressions out of release artifacts.
#[cfg(test)]
#[path = "edits_tests.rs"]
mod tests;
