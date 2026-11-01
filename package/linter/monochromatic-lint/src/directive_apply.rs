//! What:
//!  Apply inline disable directives to the findings of one host file.
//! Why:
//!  Rules know nothing about directives.
//!  One filter runs after every rule,
//! the semantic engine and the embedded-source processors
//!  have produced host-position findings,
//!  so a suppressed finding is gone before `--fix` ever sees its fix,
//!  and the same semantics hold for every rule and language.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function applyDirectives(context, findings): Diagnostic[];
//! ```

/// Import the finding model,
/// the directive stages and the registry that declares suppressibility.
use crate::{
    diagnostic::{Diagnostic, Severity, Span},
    directive_findings::Reporter,
    directive_options::DirectiveOptions,
    directive_parse::{DIRECTIVE_PREFIX, DirectiveKind},
    directive_resolve::{Directive, Targets, resolve_directives},
    directive_scan::{ScannedComments, scan_comments},
    run_modes::{RULES, RuleInfo, rule_info},
    run_paths::Language,
};
/// Import ordered containers for the line index and the replayed disabled set.
use std::collections::{BTreeMap, BTreeSet};

/// What:
///  Everything the filter needs besides the findings.
/// Why:
///  The host's language and source select the parser,
///  and the display name and options shape the directive findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirectiveContext = { language: Language; filename: string; source: string; options: DirectiveOptions };
/// ```
pub struct DirectiveContext<'file> {
    /// Language of the host file.
    pub language: Language,
    /// Display name used in directive findings.
    pub filename: &'file str,
    /// Exact snapshot the findings were produced from.
    pub source: &'file str,
    /// Run-wide unused-directive settings.
    pub options: DirectiveOptions,
}

/// What:
///  Lookup tables over the active directives.
/// Why:
///  Range directives are searched by position and line directives by line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Index = { ranges: number[]; lines: Map<number, number[]> };
/// ```
struct Index {
    /// Indexes of disable and enable directives in document order.
    ranges: Vec<usize>,
    /// Indexes of line directives by the one-based line they cover.
    lines: BTreeMap<usize, Vec<usize>>,
}

/// What:
///  Sort the active directives into the two lookup tables.
/// Why:
///  Document order is the order comments were scanned in,
///  and later directives win over earlier ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildIndex(directives: Directive[]): Index;
/// ```
fn build_index(directives: &[Directive]) -> Index {
    let mut ranges: Vec<usize> = Vec::<usize>::new();
    let mut lines: BTreeMap<usize, Vec<usize>> = BTreeMap::<usize, Vec<usize>>::new();
    for (position, directive) in directives.iter().enumerate() {
        if directive.kind.is_line_scoped() {
            lines
                .entry(directive.covered_line)
                .or_default()
                .push(position);
        } else {
            ranges.push(position);
        }
    }
    return Index { ranges, lines };
}

/// What:
///  Find the directive entry that suppresses a finding.
/// Why:
///  A line directive is more specific than a range and is tried first,
///  the latest one winning.
///  Otherwise the latest disable or enable that covers the rule and begins at or before the finding decides:
///  a disable suppresses and an enable does not.
///  The entry that decides is the one credited with the use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function suppressing(directives, index, finding): [number, number] | undefined;
/// ```
fn suppressing(
    directives: &[Directive],
    index: &Index,
    code: &str,
    offset: usize,
    line: usize,
) -> Option<(usize, usize)> {
    if let Some(candidates) = index.lines.get(&line) {
        for position in candidates.iter().rev() {
            if let Some(slot) = directives[*position].slot_for(code) {
                return Some((*position, slot));
            }
        }
    }
    // `ranges` keeps its positions in document order,
    // so the directives that start after the finding form one unbroken stretch at this reversed walk's start;
    // `continue` steps over that stretch exactly as the boundary slice left it out,
    // without a closure borrowing `directives` and `offset`.
    for position in index.ranges.iter().rev() {
        let directive: &Directive = &directives[*position];
        if directive.site.start > offset {
            continue;
        }
        if let Some(slot) = directive.slot_for(code) {
            if directive.kind == DirectiveKind::Disable {
                return Some((*position, slot));
            }
            return None;
        }
    }
    return None;
}

/// What:
///  Whether the registry lets one rule's findings be suppressed at all.
/// Why:
///  `is_some_and` takes a plain function path,
///  so the suppressibility test stays a named predicate instead of a capture-free closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSuppressibleRule(rule: RuleInfo): boolean;
/// ```
fn is_suppressible_rule(rule: &RuleInfo) -> bool {
    return rule.suppressible;
}

/// What:
///  Whether a finding is a rule's finding that its rule lets directives suppress.
/// Why:
///  Processing failures,
/// refused fixes and directive findings are not rules,
///  and a rule that declares itself unsuppressible keeps its findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSuppressible(finding: Diagnostic): boolean;
/// ```
fn is_suppressible(finding: &Diagnostic) -> bool {
    if finding.processing_failure || finding.labels.is_empty() {
        return false;
    }
    return rule_info(finding.code.as_str()).is_some_and(is_suppressible_rule);
}

/// What:
///  Credit every enable entry that re-enabled something.
/// Why:
///  Replaying the range directives in order tracks which rules are disabled,
///  so an enable that follows no matching disable is reported as unused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function creditEnables(directives, index, used): void;
/// ```
fn credit_enables(directives: &[Directive], index: &Index, used: &mut [Vec<bool>]) {
    let mut disabled: BTreeSet<&'static str> = BTreeSet::<&'static str>::new();
    for position in &index.ranges {
        let directive: &Directive = &directives[*position];
        match (&directive.targets, directive.kind) {
            (Targets::All, DirectiveKind::Disable) => {
                for rule in RULES {
                    if rule.suppressible {
                        disabled.insert(rule.id);
                    }
                }
            }
            (Targets::Listed(listed), DirectiveKind::Disable) => {
                for target in listed {
                    disabled.insert(target.rule);
                }
            }
            (Targets::All, _) => {
                used[*position][0] = !disabled.is_empty();
                disabled.clear();
            }
            (Targets::Listed(listed), _) => {
                for (slot, target) in listed.iter().enumerate() {
                    used[*position][slot] = disabled.remove(target.rule);
                }
            }
        }
    }
}

/// What:
///  Report every directive entry that nothing used.
/// Why:
///  A listed entry is judged on its own,
///  and a directive without a list is judged as a whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unusedFindings(reporter, directives, used, options): Diagnostic[];
/// ```
fn unused_findings(
    reporter: &Reporter<'_>,
    directives: &[Directive],
    used: &[Vec<bool>],
    severity: Severity,
) -> Vec<Diagnostic> {
    let mut found: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for (position, directive) in directives.iter().enumerate() {
        for (slot, was_used) in used[position].iter().enumerate() {
            if *was_used {
                continue;
            }
            match &directive.targets {
                Targets::All => found.push(reporter.unused(
                    severity,
                    directive.kind,
                    (directive.site.start, directive.site.end),
                    None,
                )),
                Targets::Listed(listed) => found.push(reporter.unused(
                    severity,
                    directive.kind,
                    (listed[slot].start, listed[slot].end),
                    Some(listed[slot].rule),
                )),
            }
        }
    }
    return found;
}

/// What:
///  Whether one finding marks a rule run that never finished.
/// Why:
///  `any` takes a plain function path,
///  so the completeness test stays a named predicate instead of a capture-free closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isProcessingFailure(finding: Diagnostic): boolean;
/// ```
fn is_processing_failure(finding: &Diagnostic) -> bool {
    return finding.processing_failure;
}

/// What:
///  Suppress the findings that directives cover,
///  and add the findings about the directives themselves.
/// Why:
///  A file whose text lacks the directive prefix cannot hold a directive,
///  so it is returned untouched without a second parse.
///  Unused directives are judged only when the rules ran completely:
///  after a processing failure some rule's findings are missing,
///  and reporting a directive as unused would be a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyDirectives(context: DirectiveContext, findings: Diagnostic[]): Diagnostic[];
/// ```
pub fn apply_directives(
    context: &DirectiveContext<'_>,
    findings: Vec<Diagnostic>,
) -> Vec<Diagnostic> {
    if !context.source.contains(DIRECTIVE_PREFIX) {
        return findings;
    }
    let Some(scanned): Option<ScannedComments> =
        scan_comments(context.language, context.filename, context.source)
    else {
        return findings;
    };
    let reporter: Reporter<'_> = Reporter {
        filename: context.filename,
        scanned: &scanned,
    };
    let mut problems: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    let directives: Vec<Directive> = resolve_directives(&reporter, context.source, &mut problems);
    let index: Index = build_index(directives.as_slice());
    let mut used: Vec<Vec<bool>> = Vec::<Vec<bool>>::new();
    for directive in &directives {
        used.push(vec![false; directive.slot_count()]);
    }
    let incomplete: bool = findings.iter().any(is_processing_failure);
    let mut kept: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for finding in findings {
        if !is_suppressible(&finding) {
            kept.push(finding);
            continue;
        }
        let span: &Span = &finding.labels[0].span;
        match suppressing(
            directives.as_slice(),
            &index,
            finding.code.as_str(),
            span.offset,
            span.line,
        ) {
            Some((position, slot)) => used[position][slot] = true,
            None => kept.push(finding),
        }
    }
    if let Some(severity) = context.options.unused
        && !incomplete
    {
        credit_enables(directives.as_slice(), &index, &mut used);
        problems.extend(unused_findings(
            &reporter,
            directives.as_slice(),
            &used,
            severity,
        ));
    }
    kept.extend(problems);
    return kept;
}

/// Filter and unused-directive controls stay outside release artifacts.
#[cfg(test)]
#[path = "directive_apply_tests.rs"]
mod tests;
