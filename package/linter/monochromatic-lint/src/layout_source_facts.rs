//! What:
//!  Read what one module body says about the module tree and about the paths it names:
//!  the `mod` lines it holds and the references its code makes.
//! Why:
//!  A module body is a whole file or the braces of an inline `mod x { ... }`.
//!  Production code only is read:
//!  anything marked for test builds (`#[cfg(test)]` on an item, a `mod` line or a `use`,
//!  or `#[test]` on a function) is skipped with everything inside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ModuleDeclaration = { name: string; pathAttribute?: string; body?: SyntaxNode };
//! type BodyFacts = { declarations: ModuleDeclaration[]; references: RawReference[] };
//! function collectBodyFacts(body: SyntaxNode): BodyFacts;
//! ```

/// Import the path reader that turns tokens into references.
use crate::layout_reference_paths::{RawReference, Token, TokenKind, read_references};
/// Import the test-build detectors shared with the sidecar rule.
use crate::rust_tests_in_sidecars::{has_test_attribute, has_test_cfg};
/// Import the syntax tree types and the walk events.
use ra_ap_syntax::{
    AstNode, NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken, WalkEvent,
    ast::{self, HasAttrs, HasName, HasVisibility, LiteralKind},
};

/// What:
///  One `mod` line of a module body.
/// Why:
///  An external module (`mod x;`) has no body and is loaded from a file;
///  an inline module has the braces' node as its body.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ModuleDeclaration = { name: string; pathAttribute?: string; body?: SyntaxNode };
/// ```
#[derive(Clone, Debug)]
pub struct ModuleDeclaration {
    /// The module name with a raw prefix (`r#`) removed.
    pub name: String,
    /// The text of `#[path = "..."]`,
    ///  when the line has one.
    pub path_attribute: Option<String>,
    /// The braces of an inline module,
    ///  or nothing for `mod x;`.
    pub body: Option<SyntaxNode>,
    /// Byte offset of the `mod` item in its file,
    ///  for the line of a finding about it.
    pub offset: usize,
}

/// What:
///  Everything a module body contributes.
/// Why:
///  The tree builder follows the declarations,
///  and the use collector resolves the references.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BodyFacts = { declarations: ModuleDeclaration[]; references: RawReference[] };
/// ```
#[derive(Clone, Debug, Default)]
pub struct BodyFacts {
    /// The `mod` lines in source order.
    pub declarations: Vec<ModuleDeclaration>,
    /// The paths the code names,
    ///  in source order.
    pub references: Vec<RawReference>,
}

/// What:
///  The name of a token without a raw prefix.
/// Why:
///  `r#type` and `type` name the same module,
///  and file names never carry the prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plainName(text: string): string;
/// ```
fn plain_name(text: &str) -> String {
    return String::from(text.strip_prefix("r#").unwrap_or(text));
}

/// What:
///  The token kind the path reader needs for a syntax token.
/// Why:
///  The reader is independent of the parser's kind list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindOf(token: SyntaxToken): TokenKind;
/// ```
fn token_kind(token: &SyntaxToken) -> TokenKind {
    return match token.kind() {
        SyntaxKind::CRATE_KW => TokenKind::Crate,
        SyntaxKind::SUPER_KW => TokenKind::Super,
        SyntaxKind::SELF_KW => TokenKind::SelfKeyword,
        SyntaxKind::IDENT => TokenKind::Identifier,
        SyntaxKind::COLON2 => TokenKind::PathSeparator,
        SyntaxKind::L_CURLY => TokenKind::OpenBrace,
        SyntaxKind::R_CURLY => TokenKind::CloseBrace,
        SyntaxKind::COMMA => TokenKind::Comma,
        SyntaxKind::STAR => TokenKind::Star,
        SyntaxKind::AS_KW => TokenKind::As,
        SyntaxKind::DOLLAR => TokenKind::Dollar,
        SyntaxKind::DOT => TokenKind::Dot,
        SyntaxKind::IN_KW => TokenKind::InKeyword,
        _ => TokenKind::Other,
    };
}

/// What:
///  The text of `#[path = "..."]` on a `mod` line.
/// Why:
///  The attribute moves the module's file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathAttribute(module: Module): string | undefined;
/// ```
fn path_attribute(module: &ast::Module) -> Option<String> {
    for attribute in module.attrs() {
        let Some(ast::Meta::KeyValueMeta(meta)) = attribute.meta() else {
            continue;
        };
        let Some(path) = meta.path() else {
            continue;
        };
        if path.syntax().text() != "path" {
            continue;
        }
        let Some(ast::Expr::Literal(literal)) = meta.expr() else {
            continue;
        };
        if let LiteralKind::String(text) = literal.kind()
            && let Ok(value) = text.value()
        {
            return Some(value.into_owned());
        }
    }
    return None;
}

/// What:
///  Whether a node is marked for test builds only.
/// Why:
///  Test-only code is outside both layout rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isTestOnly(node: SyntaxNode): boolean;
/// ```
fn is_test_only(node: &SyntaxNode) -> bool {
    if let Some(function) = ast::Fn::cast(node.clone())
        && has_test_attribute(&function)
    {
        return true;
    }
    let Some(attributed) = ast::AnyHasAttrs::cast(node.clone()) else {
        return false;
    };
    return has_test_cfg(&attributed);
}

/// What:
///  Read the declarations and references of one module body.
/// Why:
///  One preorder walk:
///  an inline module is recorded and not entered,
///  because its own references belong to it and its `super::` means something else;
///  a test-only node is skipped whole;
///  every other token joins the path reader's input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collectBodyFacts(body: SyntaxNode): BodyFacts;
/// ```
pub fn collect_body_facts(body: &SyntaxNode) -> BodyFacts {
    let mut facts: BodyFacts = BodyFacts::default();
    let mut tokens: Vec<Token> = Vec::<Token>::new();
    let mut public_uses: Vec<(usize, usize)> = Vec::<(usize, usize)>::new();
    let mut walk = body.preorder_with_tokens();
    while let Some(event) = walk.next() {
        match event {
            WalkEvent::Leave(_) => {}
            WalkEvent::Enter(NodeOrToken::Token(token)) => {
                if token.kind() == SyntaxKind::WHITESPACE || token.kind() == SyntaxKind::COMMENT {
                    continue;
                }
                tokens.push(Token {
                    kind: token_kind(&token),
                    text: plain_name(token.text()),
                    offset: usize::from(token.text_range().start()),
                });
            }
            WalkEvent::Enter(NodeOrToken::Node(node)) => {
                if node == *body {
                    continue;
                }
                if is_test_only(&node) {
                    walk.skip_subtree();
                    continue;
                }
                if let Some(module) = ast::Module::cast(node.clone()) {
                    let name: String = match module.name() {
                        Some(found) => plain_name(found.text().as_str()),
                        None => String::new(),
                    };
                    facts.declarations.push(ModuleDeclaration {
                        name,
                        path_attribute: path_attribute(&module),
                        body: module.item_list().map(|list| return list.syntax().clone()),
                        offset: usize::from(module.syntax().text_range().start()),
                    });
                    walk.skip_subtree();
                    continue;
                }
                if let Some(used) = ast::Use::cast(node.clone())
                    && used.visibility().is_some()
                {
                    let range = used.syntax().text_range();
                    public_uses.push((usize::from(range.start()), usize::from(range.end())));
                }
            }
        }
    }
    facts.references = read_references(tokens.as_slice(), public_uses.as_slice());
    return facts;
}

/// Controls of the module-body reader stay outside release artifacts.
#[cfg(test)]
#[path = "layout_source_facts_tests.rs"]
mod tests;
