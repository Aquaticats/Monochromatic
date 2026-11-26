//! What:
//!  Find the code that an index file holds.
//!  An index file is the `mod.rs` of a folder,
//!  or the `lib.rs` (else `main.rs`) of the folder of a crate root.
//! Why:
//!  The file rule leaves the index out of its loops because every sibling may name it with `super::`;
//!  that is safe only while the index holds no code that can take part in a loop.
//!  So an index may hold its top doc,
//!  `mod` lines (with their attributes and docs) and `pub use` re-exports,
//!  and nothing else;
//!  anything else is a finding that says to move it into a file of the folder.
//!  Decisions:
//!  an inline module with a body is code and is a finding;
//!  an entry point (`main.rs` that is the index of its folder) may also hold `fn main`
//!  and the plain `use` lines that feed it,
//!  because the compiler pins `fn main` to that file and no code can name `main`,
//!  so it can never close a loop;
//!  everything else an entry point holds is a finding too;
//!  code marked for test builds is not read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type IndexRole = 'index' | 'entry-point';
//! function checkIndexItems(file: Path, role: IndexRole, base: Path): LayoutFinding[];
//! ```

/// Import the finding shape and the way findings show a path,
///  the test-build detector,
///  and the parsed source.
use crate::{
    layout_finding::{
        LayoutFinding,
        key_path,
        shown,
        unique_keys,
    },
    layout_source_facts::is_test_only,
    rust_source::RustSource,
};
/// Import the syntax tree types.
use ra_ap_syntax::{
    AstNode,
    NodeOrToken,
    SyntaxKind,
    SyntaxNode,
    ast::{
        self,
        HasName,
        HasVisibility,
    },
};
/// Import native paths.
use std::path::Path;

/// What:
///  Which kind of index a file is.
/// Why:
///  An entry point may hold `fn main` and plain `use` lines beside the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexRole = 'index' | 'entry-point';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexRole {
    /// A `mod.rs`,
    ///  or a `lib.rs` that is a folder's index.
    Index,
    /// A `main.rs` that is a folder's index.
    EntryPoint,
    /// The `mod.rs` of a group folder,
    ///  which holds its top doc and `mod` lines only.
    Group,
}

/// What:
///  The text that names an item for a reader,
///  such as "a function `f`".
/// Why:
///  A finding names the kind and the name so the reader finds the item without opening an editor first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describe(kind: string, name: string | undefined): string;
/// ```
fn describe(
    kind: &str,
    name: Option<ast::Name>,
) -> String {
    return match name {
        Some(found) => format!("{kind} `{}`", found.text()),
        None => String::from(kind),
    };
}

/// What:
///  The text that names a kind of item that an index may not hold.
/// Why:
///  `mod` lines and `use` lines are decided by the caller,
///  so they are not here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatItHolds(item: Item): string;
/// ```
fn what_it_holds(item: &ast::Item) -> String {
    return match item {
        ast::Item::Fn(found) => describe("a function", found.name()),
        ast::Item::Struct(found) => describe("a struct", found.name()),
        ast::Item::Enum(found) => describe("an enum", found.name()),
        ast::Item::Union(found) => describe("a union", found.name()),
        ast::Item::Trait(found) => describe("a trait", found.name()),
        ast::Item::Const(found) => describe("a constant", found.name()),
        ast::Item::Static(found) => describe("a static", found.name()),
        ast::Item::TypeAlias(found) => describe("a type alias", found.name()),
        ast::Item::MacroRules(found) => describe("a `macro_rules!` definition", found.name()),
        ast::Item::MacroDef(found) => describe("a macro definition", found.name()),
        ast::Item::Module(found) => describe("an inline module", found.name()),
        ast::Item::Impl(_) => String::from("an `impl` block"),
        ast::Item::Use(_) => String::from("a `use` that is not public"),
        ast::Item::ExternCrate(_) => String::from("an `extern crate` line"),
        ast::Item::ExternBlock(_) => String::from("an `extern` block"),
        ast::Item::AsmExpr(_) => String::from("an assembly block"),
        ast::Item::MacroCall(found) => match found.path() {
            Some(path) => format!("a call of the macro `{}!`", path.syntax().text()),
            None => String::from("a macro call"),
        },
    };
}

/// What:
///  The text of a syntax node with every run of whitespace squeezed to one space and both ends trimmed.
/// Why:
///  A key is one line of a list file,
///  so an item written over several lines must not put a line break in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const compact = (node) => node.text().split(/\s+/u).filter(Boolean).join(' ')
/// ```
fn compact(node: &SyntaxNode) -> String {
    return node
        .text()
        .to_string()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
}

/// What:
///  What an `impl` block implements,
///  such as `Holder` or `Display for Holder`.
/// Why:
///  An `impl` block has no name of its own,
///  and two blocks of one file must not share a key,
///  so the key uses the type the block is written for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const implTarget = (block) => block.trait_ ? `${block.trait_} for ${block.selfTy}` : block.selfTy
/// ```
fn impl_target(found: &ast::Impl) -> String {
    let self_ty: String = match found.self_ty() {
        Some(found) => compact(found.syntax()),
        None => String::from("anonymous"),
    };
    return match found.trait_() {
        Some(found) => format!("{} for {self_ty}", compact(found.syntax())),
        None => self_ty,
    };
}

/// What:
///  The name of the item,
///  such as `function-end_after_the_log`,
///  which an allow-list entry writes to tolerate this one finding.
/// Why:
///  A key has to read the same after the item moves down a file,
///  and a line number does not.
///  A kind with no name of its own is named by its own text,
///  because a key must name one finding:
///  `impl-block` for every block of a file let one entry tolerate a block added later,
///  and so did `private-use` for every `use` line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function named(item: Item): string;
/// ```
fn named(item: &ast::Item) -> String {
    return match item {
        ast::Item::Fn(found) => format!("function-{}", name_text(found.name())),
        ast::Item::Struct(found) => format!("struct-{}", name_text(found.name())),
        ast::Item::Enum(found) => format!("enum-{}", name_text(found.name())),
        ast::Item::Union(found) => format!("union-{}", name_text(found.name())),
        ast::Item::Trait(found) => format!("trait-{}", name_text(found.name())),
        ast::Item::Const(found) => format!("constant-{}", name_text(found.name())),
        ast::Item::Static(found) => format!("static-{}", name_text(found.name())),
        ast::Item::TypeAlias(found) => format!("type-alias-{}", name_text(found.name())),
        ast::Item::MacroRules(found) => format!("macro-rules-{}", name_text(found.name())),
        ast::Item::MacroDef(found) => format!("macro-def-{}", name_text(found.name())),
        ast::Item::Module(found) => format!("inline-module-{}", name_text(found.name())),
        ast::Item::Impl(found) => format!("impl-{}", impl_target(found)),
        ast::Item::Use(found) => match found.use_tree() {
            Some(tree) => format!("private-use-{}", compact(tree.syntax())),
            None => String::from("private-use-anonymous"),
        },
        ast::Item::ExternCrate(found) => {
            let name: String = match found.name_ref() {
                Some(found) => found.text().to_string(),
                None => String::from("anonymous"),
            };
            match found.rename() {
                Some(found) => format!("extern-crate-{name}-{}", compact(found.syntax())),
                None => format!("extern-crate-{name}"),
            }
        }
        ast::Item::ExternBlock(found) => {
            match found.abi().and_then(|abi| return abi.string_token()) {
                // The ABI's own string, such as `"C"`, not the `extern` keyword the node also holds.
                Some(found) => format!("extern-block-{}", found.text()),
                None => String::from("extern-block-unstated"),
            }
        }
        ast::Item::AsmExpr(_) => String::from("assembly-block"),
        ast::Item::MacroCall(found) => match found.path() {
            // The path, the `!` that makes it a call, and the arguments as written.
            Some(path) => match found.token_tree() {
                Some(tree) => format!(
                    "macro-call-{}!{}",
                    compact(path.syntax()),
                    compact(tree.syntax())
                ),
                None => format!("macro-call-{}!", compact(path.syntax())),
            },
            None => String::from("macro-call-anonymous"),
        },
    };
}

/// What:
///  The text of an item's name,
///  and `anonymous` for an item that declares none.
/// Why:
///  A key keeps one shape per item kind,
///  so an unnamed `impl` or inline module still names itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nameText(name: Name | undefined): string;
/// ```
fn name_text(name: Option<ast::Name>) -> String {
    return match name {
        Some(found) => found.text().to_string(),
        None => String::from("anonymous"),
    };
}

/// What:
///  Whether a function is named `main`.
/// Why:
///  The compiler looks for the entry point by that name in the crate root file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isMain(found: FnItem): boolean;
/// ```
fn is_main(found: &ast::Fn) -> bool {
    return match found.name() {
        Some(name) => name.text() == "main",
        None => false,
    };
}

/// What:
///  Whether an item is one an index of this role may hold.
/// Why:
///  A `mod` line without braces and a public `use` are the index itself;
///  an entry point also holds `fn main` and plain `use` lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isAllowed(item: Item, role: IndexRole): boolean;
/// ```
fn is_allowed(
    item: &ast::Item,
    role: IndexRole,
) -> bool {
    return match item {
        ast::Item::Module(found) => found.item_list().is_none(),
        ast::Item::Use(found) => match role {
            IndexRole::EntryPoint => true,
            IndexRole::Index => found.visibility().is_some(),
            IndexRole::Group => false,
        },
        ast::Item::Fn(found) => role == IndexRole::EntryPoint && is_main(found),
        _ => false,
    };
}

/// What:
///  The byte offset of the first word of an item after its attributes and docs.
/// Why:
///  A finding names the line of the item itself,
///  not the line of a `#[derive]` or a doc comment above it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstWordOffset(item: SyntaxNode): number;
/// ```
fn first_word_offset(item: &SyntaxNode) -> usize {
    for part in item.children_with_tokens() {
        let (kind, start): (SyntaxKind, usize) = match &part {
            NodeOrToken::Node(node) => (node.kind(), usize::from(node.text_range().start())),
            NodeOrToken::Token(token) => (token.kind(), usize::from(token.text_range().start())),
        };
        if kind != SyntaxKind::ATTR && kind != SyntaxKind::WHITESPACE && kind != SyntaxKind::COMMENT
        {
            return start;
        }
    }
    return usize::from(item.text_range().start());
}

/// What:
///  The findings about what one index file holds.
/// Why:
///  Only the file's own items count,
///  one finding for each,
///  so the reader can fix them one by one.
///  A file that cannot be read makes no finding here,
///  because the module tree has already failed the run for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkIndexItems(file: Path, role: IndexRole, base: Path): LayoutFinding[];
/// ```
pub fn check_index_items(
    file: &Path,
    role: IndexRole,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    let Ok(text) = std::fs::read_to_string(file) else {
        return findings;
    };
    let source: RustSource = RustSource::new(file.to_string_lossy().into_owned(), text);
    for child in source.syntax().children() {
        let Some(item) = ast::Item::cast(child.clone()) else {
            continue;
        };
        if is_test_only(&child) || is_allowed(&item, role) {
            continue;
        }
        let line: usize = source.span(first_word_offset(&child), 0).line;
        let (message, help): (String, String) = match role {
            IndexRole::Index => (
                format!(
                    "{} is the index of its folder, so it may hold only its top doc, `mod` lines and `pub use` re-exports, but it holds {}.",
                    shown(file, base),
                    what_it_holds(&item)
                ),
                String::from(
                    "Move it into a file of this folder named for what it does, add `mod` for that file here, and re-export the names other folders use with `pub use`.",
                ),
            ),
            IndexRole::Group => (
                format!(
                    "{} is the index of the group folder {}, so it may hold only its top doc and `mod` lines, but it holds {}.",
                    shown(file, base),
                    shown(file.parent().unwrap_or(file), base),
                    match &item {
                        ast::Item::Use(_) => String::from("a `use` line"),
                        other => what_it_holds(other),
                    }
                ),
                String::from(
                    "Move it into a child folder of the group named for what it does, and let other folders name that child folder directly.",
                ),
            ),
            IndexRole::EntryPoint => (
                format!(
                    "{} is the entry point of its program, so it may hold only its top doc, `mod` lines, `use` lines and `fn main`, but it holds {}.",
                    shown(file, base),
                    what_it_holds(&item)
                ),
                String::from(
                    "Move it into a file of this folder named for what it does, add `mod` for that file here, and call it from `fn main`.",
                ),
            ),
        };
        findings.push(LayoutFinding {
            file: file.to_path_buf(),
            line,
            message,
            help: Some(help),
            key: format!("index-item {} {}", key_path(file, base), named(&item)),
        });
    }
    // Two blocks of one type and two calls of one macro would otherwise share a key,
    // and one list line would tolerate whichever of them a later edit added.
    unique_keys(&mut findings);
    return findings;
}

/// Controls of the index file check stay outside release artifacts.
#[cfg(test)]
#[path = "layout_index_file_items_tests.rs"]
mod tests;
