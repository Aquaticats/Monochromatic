//! What:
//!  Duplicate heading checks within the incumbent's textual ancestor scope.
//! Why:
//!  Equal text at different depths or under different parent headings is not a duplicate sibling.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Maintain the heading ancestor stack and compare [ancestor path, depth, text] identities.
//! ```

/// Import the diagnostic and severity types that duplicate-heading findings use.
use crate::diagnostic::{Diagnostic, Severity};
/// Import node findings and the processing failure reported when the document's structure cannot be walked.
use crate::markdown_finding::{finding, structure_failure};
/// Import the parsed Markdown source whose headings are compared for duplicates.
use crate::markdown_source::MarkdownSource;
/// Import native heading data and shared finding construction.
use satteri_ast::mdast::{MdastNodeType, decode_heading_data};
/// Use ordered structural keys rather than encoding relationships into delimiter strings.
use std::collections::BTreeSet;

/// Report repeated depth/text pairs under the same textual ancestor heading path.
pub fn no_duplicate_heading(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    let mut ancestors: Vec<(u8, String)> = Vec::<(u8, String)>::new();
    let mut seen: BTreeSet<Vec<(u8, String)>> = BTreeSet::<Vec<(u8, String)>>::new();
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Heading {
            continue;
        }
        let depth: u8 = decode_heading_data(context.data(*id)).depth;
        // What: `match` unwraps `Ok(text)`, the heading's text from the bounded descendant walk, or handles
        // `Err(error)`, which means the document's child index has a cycle.
        // Why: A heading whose text cannot be read was not checked, so the rule reports one processing
        // failure that names it and stops, instead of comparing headings by partial text.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let text: string;
        // try { text = context.textContent(id); }
        // catch (error) { findings.push(structureFailure(context, rule, error)); return findings; }
        // ```
        let text: String = match context.text_content(*id) {
            Ok(collected) => collected,
            Err(error) => {
                findings.push(structure_failure(
                    context,
                    "markdown/no-duplicate-heading",
                    error,
                ));
                return findings;
            }
        };
        while let Some((previous_depth, _)) = ancestors.last() {
            if *previous_depth < depth {
                break;
            }
            ancestors.pop();
        }
        // Markdown heading depth bounds this structural path; no unbounded text-spine recursion is introduced.
        ancestors.push((depth, text.clone()));
        if !seen.insert(ancestors.clone()) {
            findings.push(finding(
                context,
                *id,
                "markdown/no-duplicate-heading",
                severity,
                format!("Duplicate heading \"{text}\" among sibling headings."),
                None,
            ));
        }
    }
    return findings;
}

/// Keep source-level rule controls separate from release artifacts.
#[cfg(test)]
#[path = "markdown_duplicate_headings_tests.rs"]
mod tests;
