//! What:
//!  The unified linter's existing Rust rule behavior.
//! Why:
//!  Rule logic is ported without the old plugin,
//!  category or suppression machinery.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkMaxLines(source, limit, severity), checkRustdoc(source, severity)
//! ```

/// Import the new core's source and diagnostic interfaces.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the Rust source model that rule checks read syntax and spans from.
use crate::rust_source::RustSource;
/// Import the syntax kinds and typed impl/doc-comment views used by the incumbent.
use ra_ap_syntax::ast::{DocCommentIter, Impl};
/// Import the syntax node and doc-comment types that the documentation rule walks.
use ra_ap_syntax::{AstNode, NodeOrToken, SyntaxKind, SyntaxNode};

/// What:
///  Documentable node kinds paired with their established diagnostic labels.
/// Why:
///  One fixed table owns both selection and wording;
///  macros and extern blocks remain excluded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const documentedKinds: readonly [SyntaxKind, string][] = [...];
/// ```
const DOCUMENTED_KINDS: &[(SyntaxKind, &str)] = &[
    (SyntaxKind::FN, "function"),
    (SyntaxKind::STRUCT, "struct"),
    (SyntaxKind::ENUM, "enum"),
    (SyntaxKind::UNION, "union"),
    (SyntaxKind::TRAIT, "trait"),
    (SyntaxKind::TYPE_ALIAS, "type alias"),
    (SyntaxKind::CONST, "constant"),
    (SyntaxKind::STATIC, "static"),
    (SyntaxKind::MODULE, "module"),
    (SyntaxKind::EXTERN_CRATE, "extern crate"),
    (SyntaxKind::USE, "use"),
    (SyntaxKind::IMPL, "impl block"),
    (SyntaxKind::VARIANT, "enum variant"),
    (SyntaxKind::RECORD_FIELD, "field"),
    (SyntaxKind::TUPLE_FIELD, "field"),
    (SyntaxKind::SOURCE_FILE, "file"),
];

/// What:
///  Look up a borrowed program-lifetime label for a documentable syntax kind.
/// Why:
///  Static literals need no owned allocation for every visited node.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindLabel(kind: SyntaxKind): string | undefined;
/// ```
fn kind_label(kind: SyntaxKind) -> Option<&'static str> {
    for (candidate, label) in DOCUMENTED_KINDS {
        if *candidate == kind {
            return Some(*label);
        }
    }
    return None;
}

/// What:
///  Detect cxx-qt usage from identifier tokens,
///  not comments or string contents.
/// Why:
///  Only the established bridge carve-out may exempt imports and trait-impl members.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function usesCxxQt(root: SyntaxNode): boolean;
/// ```
fn uses_cxx_qt(root: &SyntaxNode) -> bool {
    for element in root.descendants_with_tokens() {
        let NodeOrToken::Token(token) = element else {
            continue;
        };
        if token.kind() == SyntaxKind::IDENT
            && (token.text() == "cxx_qt" || token.text() == "cxx_qt_lib")
        {
            return true;
        }
    }
    return false;
}

/// What:
///  Distinguish a trait implementation's associated item from a free or inherent item.
/// Why:
///  The bridge carve-out must not exempt inherent methods or trait declarations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isTraitImplMember(node: SyntaxNode): boolean;
/// ```
fn is_trait_impl_member(node: &SyntaxNode) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != SyntaxKind::ASSOC_ITEM_LIST {
        return false;
    }
    let Some(ancestor) = parent.parent() else {
        return false;
    };
    let Some(implementation) = Impl::cast(ancestor) else {
        return false;
    };
    return implementation.trait_().is_some();
}

/// What:
///  Build the existing named or unnamed missing-documentation message.
/// Why:
///  The new rule ID changes the namespace,
///  not what the rule tells the user.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingMessage(node: SyntaxNode, label: string): string;
/// ```
fn missing_message(node: &SyntaxNode, label: &str) -> String {
    for child in node.children() {
        if child.kind() == SyntaxKind::NAME {
            return format!("Missing rustdoc on {label} \"{}\".", child.text());
        }
    }
    return format!("Missing rustdoc on {label}.");
}

/// What:
///  Find the declaration's first non-trivia byte,
///  including any attributes.
/// Why:
///  Attached ordinary comments must not move a finding away from the declaration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function declarationOffset(node: SyntaxNode): number;
/// ```
fn declaration_offset(node: &SyntaxNode) -> usize {
    for element in node.descendants_with_tokens() {
        let NodeOrToken::Token(token) = element else {
            continue;
        };
        if token.kind() != SyntaxKind::COMMENT && token.kind() != SyntaxKind::WHITESPACE {
            return usize::from(token.text_range().start());
        }
    }
    return usize::from(node.text_range().start());
}

/// What:
///  Report a line-budget finding at the first code line beyond the configured limit.
/// Why:
///  Blank and comment-only lines were excluded once by the source context.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkMaxLines(context: RustSource, limit: number, severity: Severity): Diagnostic[];
/// ```
pub fn check_max_lines(context: &RustSource, limit: usize, severity: Severity) -> Vec<Diagnostic> {
    let count = context.code_line_count();
    if count <= limit {
        return Vec::new();
    }
    let line = context
        .code_line_at(limit)
        .expect("an exceeded limit has an offending code line");
    let span = context
        .line_span(line)
        .expect("a classified code line has a source span");
    let message =
        format!("file has {count} code lines, limit is {limit} (blank and comment lines excluded)");
    return vec![Diagnostic::new(
        "rust/max-lines",
        severity,
        message,
        context.filename.clone(),
        span,
    )];
}

/// What:
///  Report missing docs on all documentable nodes,
///  including private items and the file.
/// Why:
///  Visibility and suppression comments do not change this rule's accepted behavior.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRustdoc(context: RustSource, severity: Severity): Diagnostic[];
/// ```
pub fn check_rustdoc(context: &RustSource, severity: Severity) -> Vec<Diagnostic> {
    let bridge = uses_cxx_qt(context.syntax());
    let mut findings = Vec::new();
    for node in context.syntax().descendants() {
        let Some(label) = kind_label(node.kind()) else {
            continue;
        };
        if bridge && (node.kind() == SyntaxKind::USE || is_trait_impl_member(&node)) {
            continue;
        }
        if DocCommentIter::from_syntax_node(&node).next().is_some() {
            continue;
        }
        let offset = declaration_offset(&node);
        let length = usize::from(node.text_range().len());
        let span = context.span(offset, length);
        let message = missing_message(&node, label);
        findings.push(Diagnostic::new(
            "rust/require-rustdoc",
            severity,
            message,
            context.filename.clone(),
            span,
        ));
    }
    return findings;
}

/// Keep behavior fixtures out of release artifacts.
#[cfg(test)]
#[path = "rust_rules_tests.rs"]
mod tests;
