//! What:
//!  Build the module tree of a Rust package from its root files,
//!  following `mod` lines to the files they load.
//! Why:
//!  The layout rules ask which folder defines the module a path names,
//!  and only the real tree knows:
//!  `mod x;` loads `x.rs` or `x/mod.rs`,
//!  `#[path = "..."]` moves the file,
//!  and an inline `mod x { ... }` is a module in the file that holds it.
//!  The tree holds production modules only:
//!  a module whose `mod` line is marked for test builds,
//!  and a module loaded from a test file (`*_tests.rs`,
//!  `tests.rs` or a file under a `tests` folder),
//!  is never read.
//!  Files are found by following `mod` lines and never by walking folders,
//!  so a `uses/` link can never add a second copy of a module.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ModuleNode = { file: Path; parent?: number; children: Map<string, number>; references: FoundReference[]; globTargets: number[] };
//! type ModuleTree = { modules: ModuleNode[]; roots: number[]; problems: TreeProblem[] };
//! function buildModuleTree(roots: Path[]): ModuleTree;
//! ```

/// Import the path reader's reference type,
///  the module-body reader and its declaration type.
use crate::{
    layout_reference_paths::RawReference,
    layout_source_facts::{BodyFacts, ModuleDeclaration, collect_body_facts},
    markdown_lfs_target::lexical_normal,
    rust_source::RustSource,
    rust_tests_in_sidecars::is_sidecar_path,
};
/// Import the syntax node,
///  shared ownership of a parsed file and native paths.
use ra_ap_syntax::SyntaxNode;
/// Import an ordered map,
///  native paths and shared ownership.
use std::{collections::BTreeMap, path::PathBuf, rc::Rc};

/// What:
///  A path a module's code names,
///  with the line it is on.
/// Why:
///  Every finding names a file and line as proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FoundReference = { raw: RawReference; line: number };
/// ```
#[derive(Clone, Debug)]
pub struct FoundReference {
    /// The path as written.
    pub raw: RawReference,
    /// One-based line of the path's first token.
    pub line: usize,
}

/// What:
///  One production module.
/// Why:
///  A file module and each inline module are separate nodes,
///  because `super::` from inside an inline module names the module around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ModuleNode = { file: Path; parent?: number; children: Map<string, number>; references: FoundReference[]; globTargets: number[] };
/// ```
#[derive(Clone, Debug)]
pub struct ModuleNode {
    /// The file that holds the module's code;
    ///  an inline module shares the file of the module around it.
    pub file: PathBuf,
    /// The module around this one.
    pub parent: Option<usize>,
    /// Modules declared inside this one.
    pub children: BTreeMap<String, usize>,
    /// The paths this module's own code names.
    pub references: Vec<FoundReference>,
    /// Modules that this module re-exports with `pub use path::*`,
    ///  filled in by the resolver.
    pub glob_targets: Vec<usize>,
}

/// What:
///  A reason the tree could not be completed.
/// Why:
///  A missing or unreadable module file means the answers would be incomplete,
///  so the caller fails the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TreeProblem = { file: Path; line: number; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeProblem {
    /// The file whose `mod` line could not be followed.
    pub file: PathBuf,
    /// One-based line of the `mod` line.
    pub line: usize,
    /// What went wrong and what was tried.
    pub message: String,
}

/// What:
///  The tree of a package.
/// Why:
///  A package with a library and executables has several roots;
///  the modules of all of them live in one list so an index names a module everywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ModuleTree = { modules: ModuleNode[]; roots: number[]; problems: TreeProblem[] };
/// ```
#[derive(Clone, Debug, Default)]
pub struct ModuleTree {
    /// Every module,
    ///  indexed by position.
    pub modules: Vec<ModuleNode>,
    /// The index of each root module.
    pub roots: Vec<usize>,
    /// Module lines that could not be followed.
    pub problems: Vec<TreeProblem>,
}

/// What:
///  A module waiting to be read.
/// Why:
///  The tree is built with a work list,
///  not recursion,
///  because the nesting depth comes from the files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Pending = { name: string; file: Path; source?: RustSource; body?: SyntaxNode; parent?: number; childrenFolder: Path; pathBase: Path };
/// ```
struct Pending {
    /// The module's name.
    name: String,
    /// The file that holds it.
    file: PathBuf,
    /// The parsed host file and the braces of an inline module;
    ///  both absent for a file module that is not read yet.
    inline: Option<(Rc<RustSource>, SyntaxNode)>,
    /// The module around it.
    parent: Option<usize>,
    /// The folder where `mod x;` lines inside this module look for files.
    children_folder: PathBuf,
    /// The folder that a `#[path]` on a `mod` line inside this module is relative to.
    path_base: PathBuf,
}

/// What:
///  Whether a file module is known to be test-only by its place.
/// Why:
///  A sidecar file is never part of production code,
///  even when its `mod` line forgot the `cfg` attribute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isTestFile(file: Path): boolean;
/// ```
fn is_test_file(file: &std::path::Path) -> bool {
    return is_sidecar_path(file.to_string_lossy().as_ref());
}

/// What:
///  Whether an earlier module on the way up already is this file.
/// Why:
///  A file that loads itself through `#[path]` would loop forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loadsItself(tree: ModuleTree, from: number | undefined, file: Path): boolean;
/// ```
fn loads_itself(tree: &ModuleTree, from: Option<usize>, file: &std::path::Path) -> bool {
    let mut current: Option<usize> = from;
    while let Some(index) = current {
        if tree.modules[index].file == file {
            return true;
        }
        current = tree.modules[index].parent;
    }
    return false;
}

/// What:
///  Pick the file a `mod x;` line loads,
///  and the folder its own `mod` lines look in.
/// Why:
///  A `#[path]` file is read like a `mod.rs` file;
///  without the attribute `x.rs` and `x/mod.rs` are the two places,
///  and having both is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function locate(parent: Pending, declaration: ModuleDeclaration): { file: Path; childrenFolder: Path } | string;
/// ```
fn locate(parent: &Pending, declaration: &ModuleDeclaration) -> Result<(PathBuf, PathBuf), String> {
    if let Some(attribute) = &declaration.path_attribute {
        let file: PathBuf = lexical_normal(&parent.path_base.join(attribute));
        let folder: PathBuf = file
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_default();
        return Ok((file, folder));
    }
    let plain: PathBuf = parent
        .children_folder
        .join(format!("{}.rs", declaration.name));
    let folder: PathBuf = parent.children_folder.join(&declaration.name);
    let nested: PathBuf = folder.join("mod.rs");
    let has_plain: bool = plain.is_file();
    let has_nested: bool = nested.is_file();
    if has_plain && has_nested {
        return Err(format!(
            "module `{}` has both {} and {}; Cargo refuses that",
            declaration.name,
            plain.display(),
            nested.display()
        ));
    }
    if has_plain {
        return Ok((plain, folder));
    }
    if has_nested {
        let nested_folder: PathBuf = folder.clone();
        return Ok((nested, nested_folder));
    }
    // Neither exists: a test-only name is skipped by the caller before this message is used.
    return Err(format!(
        "module `{}` has no file: looked for {} and {}",
        declaration.name,
        plain.display(),
        nested.display()
    ));
}

/// What:
///  Read the module a pending item describes and queue the modules it declares.
/// Why:
///  A file module is read and parsed here;
///  an inline module reuses its host's parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPending(tree: ModuleTree, item: Pending, queue: Pending[]): void;
/// ```
fn read_pending(tree: &mut ModuleTree, item: Pending, queue: &mut Vec<Pending>) {
    let (source, body): (Rc<RustSource>, SyntaxNode) = match &item.inline {
        Some((host, braces)) => (Rc::clone(host), braces.clone()),
        None => {
            let text: String = match std::fs::read_to_string(&item.file) {
                Ok(read) => read,
                Err(error) => {
                    tree.problems.push(TreeProblem {
                        file: item.file.clone(),
                        line: 1,
                        message: format!("cannot read {}: {error}", item.file.display()),
                    });
                    return;
                }
            };
            let parsed: RustSource =
                RustSource::new(item.file.to_string_lossy().into_owned(), text);
            let syntax: SyntaxNode = parsed.syntax().clone();
            (Rc::new(parsed), syntax)
        }
    };
    let facts: BodyFacts = collect_body_facts(&body);
    let index: usize = tree.modules.len();
    let mut references: Vec<FoundReference> = Vec::<FoundReference>::new();
    for raw in facts.references {
        let line: usize = source.span(raw.offset, 0).line;
        references.push(FoundReference { raw, line });
    }
    tree.modules.push(ModuleNode {
        file: item.file.clone(),
        parent: item.parent,
        children: BTreeMap::<String, usize>::new(),
        references,
        glob_targets: Vec::<usize>::new(),
    });
    match item.parent {
        Some(parent) => {
            tree.modules[parent]
                .children
                .insert(item.name.clone(), index);
        }
        None => tree.roots.push(index),
    }
    for declaration in facts.declarations {
        queue_declaration(tree, &item, index, &source, declaration, queue);
    }
}

/// What:
///  Queue the module one `mod` line declares,
///  or record why it cannot be followed.
/// Why:
///  Inline modules extend the folder chain by their name or `#[path]`;
///  external ones are located on disk and skipped when they are test files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function queueDeclaration(tree: ModuleTree, host: Pending, hostIndex: number, source: RustSource, declaration: ModuleDeclaration, queue: Pending[]): void;
/// ```
fn queue_declaration(
    tree: &mut ModuleTree,
    host: &Pending,
    host_index: usize,
    source: &Rc<RustSource>,
    declaration: ModuleDeclaration,
    queue: &mut Vec<Pending>,
) {
    if let Some(braces) = declaration.body {
        let component: String = match &declaration.path_attribute {
            Some(attribute) => attribute.clone(),
            None => declaration.name.clone(),
        };
        let folder: PathBuf = lexical_normal(&host.children_folder.join(component));
        queue.push(Pending {
            name: declaration.name,
            file: host.file.clone(),
            inline: Some((Rc::clone(source), braces)),
            parent: Some(host_index),
            children_folder: folder.clone(),
            path_base: folder,
        });
        return;
    }
    let (file, folder): (PathBuf, PathBuf) = match locate(host, &declaration) {
        Ok(found) => found,
        Err(message) => {
            // A name that marks a test file needs no file in production code.
            let sidecar: PathBuf = host
                .children_folder
                .join(format!("{}.rs", declaration.name));
            if !is_test_file(&sidecar) {
                tree.problems.push(TreeProblem {
                    file: host.file.clone(),
                    line: source.span(declaration.offset, 0).line,
                    message,
                });
            }
            return;
        }
    };
    if is_test_file(&file) {
        return;
    }
    if loads_itself(tree, Some(host_index), &file) {
        tree.problems.push(TreeProblem {
            file: host.file.clone(),
            line: source.span(declaration.offset, 0).line,
            message: format!(
                "module `{}` loads {}, which already contains it",
                declaration.name,
                file.display()
            ),
        });
        return;
    }
    queue.push(Pending {
        name: declaration.name,
        file: file.clone(),
        inline: None,
        parent: Some(host_index),
        children_folder: folder.clone(),
        path_base: file
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_default(),
    });
}

/// What:
///  Build the module tree that starts at the given root files.
/// Why:
///  Roots are lexically normal absolute paths.
///  The work list is a stack,
///  so the tree is filled depth first without recursion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildModuleTree(roots: Path[]): ModuleTree;
/// ```
pub fn build_module_tree(roots: &[PathBuf]) -> ModuleTree {
    let mut tree: ModuleTree = ModuleTree::default();
    for root in roots {
        let folder: PathBuf = root
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_default();
        let mut queue: Vec<Pending> = vec![Pending {
            name: String::from("crate"),
            file: root.clone(),
            inline: None,
            parent: None,
            children_folder: folder.clone(),
            path_base: folder,
        }];
        while let Some(item) = queue.pop() {
            read_pending(&mut tree, item, &mut queue);
        }
    }
    return tree;
}

/// Controls of the module tree stay outside release artifacts.
#[cfg(test)]
#[path = "layout_module_tree_tests.rs"]
mod tests;
