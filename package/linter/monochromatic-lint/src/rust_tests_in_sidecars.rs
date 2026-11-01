//! What:
//!  Require Rust unit tests to live in sidecar files.
//! Why:
//!  A sidecar keeps tests out of the source file's line budget and mutation scope
//!  while remaining a child module that reaches private items.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Report inline test-configured modules and test functions found outside a sidecar file name.
//! ```

/// Import the shared finding model and retained Rust parse.
use crate::diagnostic::{Diagnostic, Severity, Span};
/// Import the owned source whose syntax tree the rule walks.
use crate::rust_source::RustSource;
/// Import attribute and name access on parsed items,
/// not a text-pattern approximation.
use ra_ap_syntax::ast::{HasAttrs, HasName};
/// Import parser node casts and the syntax types the walk passes around.
use ra_ap_syntax::{AstNode, SyntaxNode, SyntaxToken, ast};

/// Split a path into its directory parts and file name,
///  accepting either separator so a Windows path is read like a Unix one.
fn path_parts(filename: &str) -> (Vec<String>, String) {
    let normalized: String = filename.replace('\\', "/");
    let mut parts: Vec<String> = Vec::<String>::new();
    for part in normalized.split('/') {
        parts.push(String::from(part));
    }
    let file: String = parts.pop().unwrap_or_default();
    return (parts, file);
}

/// The file name without its extension,
///  which names the sidecar a finding asks for.
fn file_stem(file: &str) -> &str {
    return file.strip_suffix(".rs").unwrap_or(file);
}

/// Whether a path is already a sidecar:
///  a `*_tests.rs` file,
/// a `tests.rs` file or anything under a `tests` directory.
fn is_sidecar_path(filename: &str) -> bool {
    let (directories, file): (Vec<String>, String) = path_parts(filename);
    for directory in &directories {
        if directory == "tests" {
            return true;
        }
    }
    let stem: &str = file_stem(file.as_str());
    return stem == "tests" || stem.ends_with("_tests");
}

/// Whether a `cfg` predicate selects test builds.
///  `not(...)` flips the sense,
/// and a key-value atom such as `feature = "test"` is not the `test` flag.
fn predicate_enables_test(predicate: &ast::CfgPredicate, negated: bool) -> bool {
    match predicate {
        ast::CfgPredicate::CfgAtom(atom) => {
            let Some(identifier): Option<SyntaxToken> = atom.ident_token() else {
                return false;
            };
            return !negated && atom.eq_token().is_none() && identifier.text() == "test";
        }
        ast::CfgPredicate::CfgComposite(composite) => {
            let is_not: bool = match composite.keyword() {
                Some(keyword) => keyword.text() == "not",
                None => false,
            };
            let inner_negated: bool = if is_not { !negated } else { negated };
            for child in composite.cfg_predicates() {
                if predicate_enables_test(&child, inner_negated) {
                    return true;
                }
            }
            return false;
        }
    }
}

/// Whether an item carries a `#[cfg(...)]` that selects test builds.
fn has_test_cfg(item: &impl HasAttrs) -> bool {
    for attribute in item.attrs() {
        let Some(ast::Meta::CfgMeta(cfg)): Option<ast::Meta> = attribute.meta() else {
            continue;
        };
        let Some(predicate): Option<ast::CfgPredicate> = cfg.cfg_predicate() else {
            continue;
        };
        if predicate_enables_test(&predicate, false) {
            return true;
        }
    }
    return false;
}

/// Whether a function carries a test attribute:
///  `#[test]` or a runtime's `#[tokio::test]`,
/// which share the last path segment `test`.
fn has_test_attribute(function: &ast::Fn) -> bool {
    for attribute in function.attrs() {
        let Some(path): Option<ast::Path> = attribute.path() else {
            continue;
        };
        let Some(segment): Option<ast::PathSegment> = path.segment() else {
            continue;
        };
        if segment.syntax().text() == "test" {
            return true;
        }
    }
    return false;
}

/// Whether a module has an inline body rather than loading a file.
fn is_inline_test_module(module: &ast::Module) -> bool {
    return module.item_list().is_some() && has_test_cfg(module);
}

/// Whether a node sits inside an inline test module,
///  which already carries its own finding.
fn inside_inline_test_module(node: &SyntaxNode) -> bool {
    for ancestor in node.ancestors().skip(1) {
        let Some(module): Option<ast::Module> = ast::Module::cast(ancestor) else {
            continue;
        };
        if is_inline_test_module(&module) {
            return true;
        }
    }
    return false;
}

/// Build a finding anchored on a node's name,
///  or on the node itself when it has none.
fn finding_for(
    context: &RustSource,
    severity: Severity,
    anchor: &SyntaxNode,
    message: String,
) -> Diagnostic {
    let offset: usize = usize::from(anchor.text_range().start());
    let length: usize = usize::from(anchor.text_range().len());
    let span: Span = context.span(offset, length);
    let (_, file): (Vec<String>, String) = path_parts(context.filename.as_str());
    let stem: &str = file_stem(file.as_str());
    let mut finding: Diagnostic = Diagnostic::new(
        "rust/tests-in-sidecars",
        severity,
        message,
        context.filename.clone(),
        span,
    );
    finding.help = Some(format!(
        "Move the tests to `{stem}_tests.rs` beside this file and declare them with `#[cfg(test)] #[path = \"{stem}_tests.rs\"] mod tests;`."
    ));
    return finding;
}

/// What:
///  Report each inline test module and each test function outside a sidecar file.
/// Why:
///  A sidecar file is recognised by its name,
///  because a rule sees one parsed file and cannot see which file declares it.
/// Files named `*_tests.rs`,
///  `tests.rs` or under a `tests` directory are never reported.
/// A `#[cfg(test)] mod name;` declaration is not reported;
///  only a module with an inline body is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkTestsInSidecars(context: RustSource, severity: Severity): Diagnostic[];
/// ```
pub fn check_tests_in_sidecars(context: &RustSource, severity: Severity) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    if is_sidecar_path(context.filename.as_str()) {
        return findings;
    }
    for node in context.syntax().descendants() {
        if let Some(module) = ast::Module::cast(node.clone()) {
            if is_inline_test_module(&module) && !inside_inline_test_module(module.syntax()) {
                let anchor: SyntaxNode = match module.name() {
                    Some(name) => name.syntax().clone(),
                    None => module.syntax().clone(),
                };
                findings.push(finding_for(
                    context,
                    severity,
                    &anchor,
                    String::from("Inline test module; unit tests belong in a sidecar file."),
                ));
            }
            continue;
        }
        let Some(function): Option<ast::Fn> = ast::Fn::cast(node) else {
            continue;
        };
        if has_test_attribute(&function) && !inside_inline_test_module(function.syntax()) {
            let anchor: SyntaxNode = match function.name() {
                Some(name) => name.syntax().clone(),
                None => function.syntax().clone(),
            };
            findings.push(finding_for(
                context,
                severity,
                &anchor,
                String::from(
                    "Test function outside a sidecar file; unit tests belong in a sidecar file.",
                ),
            ));
        }
    }
    return findings;
}

/// Keep behavior controls out of release artifacts.
#[cfg(test)]
#[path = "rust_tests_in_sidecars_tests.rs"]
mod tests;
