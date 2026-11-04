//! What:
//!  Shared fixtures for the layout tests.
//! Why:
//!  Every layout test writes a small Cargo package into a disposable folder,
//!  builds its module tree and asks one question of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = packageWith([['src/lib.rs', 'mod a;'], ['src/a.rs', '']]);
//! const tree = treeOf(fixture.path);
//! ```

/// Import the units under test and the fixture writers.
use crate::{
    layout_crate_roots::crate_root_files,
    layout_module_tree::{ModuleTree, build_module_tree},
    layout_path_resolution::link_glob_targets,
    layout_reference_paths::RawReference,
    layout_source_facts::collect_body_facts,
    run_test_support::write,
    rust_source::RustSource,
    test_fs::Fixture,
};
/// Import native paths.
use std::path::{Path, PathBuf};

/// Write a package with the given source files and a minimal `Cargo.toml`.
pub(crate) fn package_with(files: &[(&str, &str)]) -> Fixture {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, "Cargo.toml", "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n");
    for (relative, contents) in files {
        write(&fixture.path, relative, contents);
    }
    return fixture;
}

/// Build the module tree of a fixture package from its discovered roots,
///  with glob targets linked as a lint run does.
pub(crate) fn tree_of(root: &Path) -> ModuleTree {
    let roots: Vec<PathBuf> =
        crate_root_files(&root.join("Cargo.toml")).expect("fixture roots are readable");
    let mut tree: ModuleTree = build_module_tree(roots.as_slice());
    link_glob_targets(&mut tree);
    return tree;
}

/// The index of the first module held by the file at a path relative to the fixture.
pub(crate) fn module_in(tree: &ModuleTree, root: &Path, relative: &str) -> usize {
    let wanted: PathBuf = root.join(relative);
    for (index, module) in tree.modules.iter().enumerate() {
        if module.file == wanted {
            return index;
        }
    }
    panic!("no module in {relative}");
}

/// The references of a source text as `a::b::c` strings,
///  with `::*` for a glob and a leading `pub ` for a re-export.
pub(crate) fn references_of(source: &str) -> Vec<String> {
    let parsed: RustSource = RustSource::new(String::from("fixture.rs"), String::from(source));
    let mut shown: Vec<String> = Vec::<String>::new();
    for reference in collect_body_facts(parsed.syntax()).references {
        shown.push(describe(&reference));
    }
    return shown;
}

/// One reference as text.
pub(crate) fn describe(reference: &RawReference) -> String {
    let mut text: String = reference.segments.join("::");
    if reference.glob {
        text.push_str("::*");
    }
    if reference.reexport {
        text.insert_str(0, "pub ");
    }
    return text;
}

/// Make a symbolic link to a folder or file,
///  where `target` is the text stored in the link.
#[cfg(unix)]
pub(crate) fn link(root: &Path, entry: &str, target: &str) {
    let path: PathBuf = root.join(entry);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("link parent");
    }
    std::os::unix::fs::symlink(target, &path).expect("symbolic link");
}
