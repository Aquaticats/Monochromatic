//! What:
//!  Turn the candidate comments of a host file into active directives and the findings about broken ones.
//! Why:
//!  Reading,
//! justification,
//! identifier checks and suppressibility are one pass over the comments,
//!  so a directive is either active or reported,
//!  and an active directive names only rules that may be suppressed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function resolveDirectives(reporter, source, problems): Directive[];
//! ```

/// Import the finding model,
/// the comment reader and the reporter that builds the findings.
use crate::{
    diagnostic::Diagnostic,
    directive_findings::Reporter,
    directive_parse::{DirectiveKind, ParsedComment, ParsedDirective, parse_comment},
    directive_scan::CommentSite,
    run_modes::{RuleInfo, rule_info},
};

/// What:
///  One rule an active directive covers,
///  as the registry spells it.
/// Why:
///  The position of the written entry lets an unused entry be underlined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { rule: string; start: number; end: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Target {
    /// The registry identifier.
    pub rule: &'static str,
    /// Host byte offset of the written entry.
    pub start: usize,
    /// Host byte offset after the written entry.
    pub end: usize,
}

/// What:
///  The rules an active directive covers.
/// Why:
///  A directive with no list covers every suppressible rule,
///  while a list whose entries were all refused covers nothing:
///  the two must never be confused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Targets = { kind: 'all' } | { kind: 'listed'; targets: Target[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Targets {
    /// Every suppressible rule.
    All,
    /// Exactly these rules,
    ///  possibly none.
    Listed(
        /// The accepted entries in written order.
        Vec<Target>,
    ),
}

/// What:
///  A directive that passed every check.
/// Why:
///  Matching findings against directives reads only these fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Directive = { kind: DirectiveKind; targets: Targets; site: CommentSite; coveredLine: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Directive {
    /// The keyword.
    pub kind: DirectiveKind,
    /// The covered rules.
    pub targets: Targets,
    /// The comment's place in the host.
    pub site: CommentSite,
    /// For the line kinds,
    ///  the one-based host line they cover;
    ///  zero for the range kinds.
    pub covered_line: usize,
}

/// What:
///  Slot arithmetic for usage tracking.
/// Why:
///  A directive with a list is used per listed rule,
///  and one without a list is used or unused as a whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// directive.slotCount(); directive.slotFor(code)
/// ```
impl Directive {
    /// How many independently usable entries the directive has.
    pub fn slot_count(&self) -> usize {
        match &self.targets {
            Targets::All => return 1,
            Targets::Listed(listed) => return listed.len(),
        }
    }

    /// The entry that covers a finding's rule,
    ///  if any.
    pub fn slot_for(&self, code: &str) -> Option<usize> {
        match &self.targets {
            Targets::All => return Some(0),
            Targets::Listed(listed) => {
                // A plain walk returns the slot without the `position` closure,
                //  which would borrow `code` for a callee that keeps nothing.
                for (slot, target) in listed.iter().enumerate() {
                    if target.rule == code {
                        return Some(slot);
                    }
                }
                return None;
            }
        }
    }
}

/// What:
///  Check a readable directive's identifiers against the registry.
/// Why:
///  Each refused entry is its own finding,
///  and the accepted entries still apply,
///  so one typo does not hide the rest of the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRules(reporter, parsed, problems): Targets;
/// ```
fn check_rules(
    reporter: &Reporter<'_>,
    parsed: &ParsedDirective,
    problems: &mut Vec<Diagnostic>,
) -> Targets {
    if parsed.rules.is_empty() {
        return Targets::All;
    }
    let mut accepted: Vec<Target> = Vec::<Target>::new();
    for token in &parsed.rules {
        let Some(info): Option<&'static RuleInfo> = rule_info(token.text.as_str()) else {
            problems.push(reporter.unknown_rule(token));
            continue;
        };
        if !info.suppressible {
            problems.push(reporter.non_suppressible(token));
            continue;
        }
        accepted.push(Target {
            rule: info.id,
            start: token.start,
            end: token.end,
        });
    }
    return Targets::Listed(accepted);
}

/// What:
///  Resolve one readable directive,
///  or report why it is void.
/// Why:
///  A directive without a justification suppresses nothing,
///  and a line directive spread over several lines is ambiguous about the line it covers.
///  Identifier problems are reported either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveOne(reporter, site, parsed, problems): Directive | undefined;
/// ```
fn resolve_one(
    reporter: &Reporter<'_>,
    site: CommentSite,
    parsed: &ParsedDirective,
    problems: &mut Vec<Diagnostic>,
) -> Option<Directive> {
    let targets: Targets = check_rules(reporter, parsed, problems);
    if parsed.justification.is_none() {
        problems.push(reporter.missing_justification(site, parsed.kind));
        return None;
    }
    let first_line: usize = reporter.scanned.span(site.start, 0).line;
    let last_line: usize = reporter.scanned.span(site.end - 1, 0).line;
    if parsed.kind.is_line_scoped() && first_line != last_line {
        problems.push(
            reporter.malformed(
                site,
                format!(
                    "{} covers one line, so it must be written on one line",
                    parsed.kind.keyword()
                )
                .as_str(),
            ),
        );
        return None;
    }
    let mut covered_line: usize = 0;
    if parsed.kind == DirectiveKind::DisableLine {
        covered_line = first_line;
    }
    if parsed.kind == DirectiveKind::DisableNextLine {
        covered_line = last_line + 1;
    }
    return Some(Directive {
        kind: parsed.kind,
        targets,
        site,
        covered_line,
    });
}

/// What:
///  Resolve every candidate comment in document order.
/// Why:
///  Active directives come back for matching,
///  and everything wrong with the others lands in `problems`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveDirectives(reporter, source, problems): Directive[];
/// ```
pub fn resolve_directives(
    reporter: &Reporter<'_>,
    source: &str,
    problems: &mut Vec<Diagnostic>,
) -> Vec<Directive> {
    let mut directives: Vec<Directive> = Vec::<Directive>::new();
    for site in &reporter.scanned.sites {
        let content: &str = &source[site.content_start..site.content_end];
        match parse_comment(content, site.content_start) {
            ParsedComment::NotDirective => {}
            ParsedComment::Malformed(reason) => {
                problems.push(reporter.malformed(*site, reason.as_str()));
            }
            ParsedComment::Directive(parsed) => {
                if let Some(active) = resolve_one(reporter, *site, &parsed, problems) {
                    directives.push(active);
                }
            }
        }
    }
    return directives;
}

/// Resolution controls stay outside release artifacts.
#[cfg(test)]
#[path = "directive_resolve_tests.rs"]
mod tests;
