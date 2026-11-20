//! Controls of the shared source-scan helpers,
//! on filesystem fixtures:
//! a nested layout is walked and resolved,
//! test-only
//! support is told from release code by the module tree,
//! and a tree that cannot be trusted is an error.

use super::declaration::{Declaration, declarations, module_name};
use super::{SourceFile, classified_sources, manifest_roots, module_tree, real_roots, release_root, source_root, write_tree};
use crate::source_scan_tests::walk::discover;
use crate::test_support::{fixture, remove};

/// The classification of one path in a listing,
/// or a panic naming the path when the listing lacks it.
fn test_only_of(listing: &[SourceFile], path: &str) -> bool {
    return listing.iter().find(|file| return file.path == path).unwrap_or_else(|| panic!("{path} is listed")).test_only;
}

/// The declarations of a text,
/// or a panic when the text cannot be read.
fn declared(text: &str) -> Vec<Declaration> {
    return declarations(text).expect("readable declarations");
}

/// A declaration reads with its `path` attribute and with every spelling of a test gate,
/// and only a gate that holds
/// solely when testing marks it test-only.
#[test]
fn declarations_read_path_attributes_and_test_gates() {
    let text: &str = "\
        pub mod plain;\n\
        /// Doc line between the attribute and the module does not detach it.\n\
        #[cfg(test)]\n\
        /// A doc line.\n\
        mod gated;\n\
        #[cfg(all(test, target_os = \"linux\"))]\n\
        pub(crate) mod linux_only_in_tests;\n\
        #[cfg(any(test, unix))]\n\
        mod any_of;\n\
        #[cfg(not(test))]\n\
        mod release_only;\n\
        #[cfg(test)]\n\
        #[path = \"other_name.rs\"]\n\
        mod renamed;\n";
    let found: Vec<Declaration> = declared(text);
    let summary: Vec<(String, Option<String>, bool)> =
        found.iter().map(|d| return (d.name.clone(), d.path_attribute.clone(), d.test_only)).collect();
    assert_eq!(
        summary,
        vec![
            (String::from("plain"), None, false),
            (String::from("gated"), None, true),
            (String::from("linux_only_in_tests"), None, true),
            (String::from("any_of"), None, false),
            (String::from("release_only"), None, false),
            (String::from("renamed"), Some(String::from("other_name.rs")), true),
        ]
    );
}

/// An item between an attribute and a module detaches the attribute,
/// so a gate on a function never gates the module
/// after it;
/// an indented declaration and an unreadable `path` attribute are errors,
/// not guesses.
#[test]
fn declarations_refuse_what_they_cannot_place() {
    let detached: Vec<Declaration> = declared("#[cfg(test)]\nfn helper() {}\nmod after;\n");
    assert!(!detached[0].test_only, "the gate belongs to the function");
    assert!(declarations("mod outer {\n    mod inner;\n}\n").is_err(), "an indented declaration");
    assert!(declarations("#[path=\"x.rs\"]\nmod x;\n").is_err(), "a path attribute in another spelling");
    assert!(declarations("#[cfg_attr(unix, path = \"x.rs\")]\nmod x;\n").is_err(), "a path chosen by condition");
    assert_eq!(module_name("pub(crate) mod a_b1;"), Some("a_b1"));
    assert_eq!(module_name("mod a { }"), None);
    assert_eq!(module_name("use a::mod_b;"), None);
}

/// A nested layout resolves the way the compiler resolves it:
/// a `mod.rs` directory,
/// a sibling directory named after
/// a file,
/// and a `path` attribute,
/// with test gates inherited by every child.
#[test]
fn the_tree_resolves_nested_modules_and_inherits_test_gates() {
    let root = fixture("scan-tree-nested");
    write_tree(
        &root,
        &[
            ("lib.rs", "mod engine;\nmod sibling;\n#[cfg(test)]\nmod support;\n#[path = \"named_elsewhere.rs\"]\nmod renamed;\n"),
            ("engine/mod.rs", "mod launch;\n"),
            ("engine/launch.rs", "// release\n"),
            ("sibling.rs", "mod child;\n"),
            ("sibling/child.rs", "// release\n"),
            ("support/mod.rs", "mod inner;\n"),
            ("support/inner.rs", "// test only through support\n"),
            ("named_elsewhere.rs", "mod after_path;\n"),
            ("after_path.rs", "// a file loaded through a path attribute keeps its children beside it\n"),
        ],
    );
    let listing: Vec<SourceFile> = module_tree(&root, &[release_root("lib.rs")]).expect("a resolvable tree");
    let paths: Vec<&str> = listing.iter().map(|file| return file.path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "after_path.rs",
            "engine/launch.rs",
            "engine/mod.rs",
            "lib.rs",
            "named_elsewhere.rs",
            "sibling.rs",
            "sibling/child.rs",
            "support/inner.rs",
            "support/mod.rs",
        ]
    );
    assert!(test_only_of(&listing, "support/inner.rs"), "the child of test support is test support");
    assert!(test_only_of(&listing, "support/mod.rs"));
    assert!(!test_only_of(&listing, "engine/launch.rs"));
    assert!(!test_only_of(&listing, "after_path.rs"));
}

/// A file reached once through a gate and once without is release,
/// whichever is followed first.
#[test]
fn a_file_reached_without_a_gate_is_release_code() {
    let root = fixture("scan-tree-two-routes");
    write_tree(
        &root,
        &[
            ("lib.rs", "#[cfg(test)]\nmod gated_route;\nmod plain_route;\n"),
            ("gated_route.rs", "#[path = \"shared.rs\"]\nmod shared;\n"),
            ("plain_route.rs", "#[path = \"shared.rs\"]\nmod shared;\n"),
            ("shared.rs", "// reached both ways\n"),
        ],
    );
    let listing: Vec<SourceFile> = module_tree(&root, &[release_root("lib.rs")]).expect("a resolvable tree");
    assert!(!test_only_of(&listing, "shared.rs"));
    assert!(test_only_of(&listing, "gated_route.rs"));
}

/// A tree that is not what the compiler reads is an error:
/// a declaration with no file,
/// a declaration with two files,
/// a source file nothing declares,
/// and a root that is missing.
#[test]
fn an_untrustworthy_tree_is_an_error() {
    let missing = fixture("scan-tree-missing-file");
    write_tree(&missing, &[("lib.rs", "mod absent;\n")]);
    assert!(module_tree(&missing, &[release_root("lib.rs")]).is_err(), "a declared module without a file");
    let twice = fixture("scan-tree-two-files");
    write_tree(&twice, &[("lib.rs", "mod both;\n"), ("both.rs", ""), ("both/mod.rs", "")]);
    assert!(module_tree(&twice, &[release_root("lib.rs")]).is_err(), "a declared module with two files");
    let orphan = fixture("scan-tree-orphan");
    write_tree(&orphan, &[("lib.rs", ""), ("nested/forgotten.rs", "")]);
    let error: String = classified_sources(&orphan, &[release_root("lib.rs")]).expect_err("an unreached file fails");
    assert!(error.contains("nested/forgotten.rs"), "{error}");
    let rootless = fixture("scan-tree-no-root");
    write_tree(&rootless, &[("other.rs", "")]);
    assert!(module_tree(&rootless, &[release_root("lib.rs")]).is_err(), "a root that is not there");
}

/// The walk lists nested files by relative name,
/// and an empty tree and an absent root are errors.
#[test]
fn the_walk_lists_everything_or_fails() {
    let nested = fixture("scan-walk-nested");
    write_tree(&nested, &[("b.rs", ""), ("a/deep/c.rs", ""), ("a/readme.md", "not source")]);
    assert_eq!(discover(&nested).expect("a listing"), vec!["a/deep/c.rs".to_owned(), "b.rs".to_owned()]);
    let empty = fixture("scan-walk-empty");
    write_tree(&empty, &[("notes.md", "no source here")]);
    assert!(discover(&empty).is_err(), "an empty listing proves nothing");
    assert!(discover(&nested.join("absent")).is_err(), "a root that is not there");
}

/// A link in the tree may point outside it or nowhere,
/// so the walk is an error and not a listing.
#[cfg(unix)]
#[test]
fn the_walk_refuses_a_link() {
    let linked = fixture("scan-walk-link");
    write_tree(&linked, &[("real.rs", "")]);
    std::os::unix::fs::symlink(linked.join("nowhere.rs"), linked.join("dangling.rs")).expect("a dangling link");
    assert!(discover(&linked).is_err(), "a link may point outside the tree or nowhere");
}

/// What:
///  A link inside a `uses` folder is the repository's own record of which source folders a folder's code uses,
///  so the walk reads nothing through it and lists the folder's `.rs` files as it lists the others,
///  while a link anywhere else still stops the walk.
/// Why:
///  Rule `LYD` asks every source folder for that `uses` folder,
///  and the wrapper's own guards failed on those links before this control existed:
///  a listing that refused them could not exist beside the layout the repository asks for,
///  and a listing that skipped every link would let a link hide sources a scan must read.
#[cfg(unix)]
#[test]
fn the_walk_reads_nothing_through_a_uses_folder_and_still_refuses_any_other_link() {
    let tree = fixture("scan-walk-uses");
    write_tree(&tree, &[("folder/mod.rs", "// a source file\n"), ("folder/uses/stray.rs", "// a source file beside the links\n")]);
    std::fs::create_dir_all(tree.join("folder/uses")).expect("the uses folder");
    std::os::unix::fs::symlink("../..", tree.join("folder/uses/other")).expect("a link to a folder beside this one");
    assert_eq!(
        discover(&tree).expect("the links of a uses folder are not read"),
        vec!["folder/mod.rs".to_owned(), "folder/uses/stray.rs".to_owned()],
        "the folder's own sources are listed and the link is not"
    );
    std::os::unix::fs::symlink(tree.join("folder/mod.rs"), tree.join("folder/link.rs")).expect("a link beside the sources");
    let refused: String = discover(&tree).expect_err("a link anywhere else still stops the walk");
    assert!(refused.contains("folder/link.rs"), "the link is named: {refused}");
    assert!(refused.contains("symbolic link"), "the reason is given: {refused}");
    remove(tree.as_path());
}

/// What:
///  A link in a `uses` folder whose target is not a folder below the walk's root is refused,
///  whether it names a file,
///  a folder outside the tree or nothing,
///  while the layout's own link to a folder in the tree is read past.
/// Why:
///  The skip serves a `uses` folder's links to this tree's own source folders
///  (`AGENTS.md`,
///  rule `LYD`);
///  a link to a file or to a folder outside it is not that record,
///  and skipping it would let it hide a source the scan must read.
#[cfg(unix)]
#[test]
fn a_uses_link_to_anything_but_a_folder_below_the_root_is_refused() {
    let tree = fixture("scan-walk-uses-file-link");
    write_tree(&tree, &[("folder/mod.rs", "// a source file\n"), ("folder/other/inner.rs", "// a source folder beside it\n")]);
    std::fs::create_dir_all(tree.join("folder/uses")).expect("the uses folder");
    std::os::unix::fs::symlink("../other", tree.join("folder/uses/other")).expect("the layout's own link");
    assert_eq!(
        discover(&tree).expect("a link to a source folder of this tree is the layout's own record"),
        vec!["folder/mod.rs".to_owned(), "folder/other/inner.rs".to_owned()],
        "the linked folder is listed under its own name"
    );
    std::os::unix::fs::symlink(tree.join("folder/mod.rs"), tree.join("folder/uses/a-file")).expect("a link to a file");
    let refused: String = discover(&tree).expect_err("a uses link to a file is not the layout's record");
    assert!(refused.contains("folder/uses/a-file"), "the link is named: {refused}");
    assert!(refused.contains("not a folder below"), "the reason is given: {refused}");
    std::fs::remove_file(tree.join("folder/uses/a-file")).expect("link away");
    let elsewhere = fixture("scan-walk-uses-outside");
    write_tree(&elsewhere, &[("folder/mod.rs", "// a source file\n")]);
    std::fs::create_dir_all(elsewhere.join("folder/uses")).expect("the uses folder");
    std::fs::create_dir_all(tree.join("outside/other")).expect("a folder outside the walk");
    std::os::unix::fs::symlink(tree.join("outside/other"), elsewhere.join("folder/uses/outside")).expect("a link outside the tree");
    let refused_2: String = discover(&elsewhere).expect_err("a uses link to a folder outside the tree");
    assert!(refused_2.contains("folder/uses/outside"), "the link is named: {refused_2}");
    std::fs::remove_file(elsewhere.join("folder/uses/outside")).expect("link away");
    std::os::unix::fs::symlink(elsewhere.join("folder/uses/nowhere"), elsewhere.join("folder/uses/dangling")).expect("a dangling link");
    let refused_3: String = discover(&elsewhere).expect_err("a uses link that names nothing");
    assert!(refused_3.contains("folder/uses/dangling"), "the link is named: {refused_3}");
    remove(tree.as_path());
    remove(elsewhere.as_path());
}

/// The manifest's library and binaries are release roots,
/// its test targets are test-only roots,
/// and a target
/// outside the source prefix is an error rather than a silent gap.
#[test]
fn the_manifest_gives_the_roots() {
    let manifest: &str = "[package]\nname = \"x\"\n\n[lib]\npath = \"src/native/lib.rs\"\n\n[[bin]]\nname = \"b\"\n\
        path = \"src/native/main.rs\"\n\n[[test]]\nname = \"t\"\npath = \"src/native/t_tests.rs\"\n\n\
        [dependencies]\nother = { path = \"../elsewhere\" }\n";
    let roots = manifest_roots(manifest, "src/native/").expect("roots");
    let summary: Vec<(&str, bool)> = roots.iter().map(|root| return (root.path.as_str(), root.test_only)).collect();
    assert_eq!(summary, vec![("lib.rs", false), ("main.rs", false), ("t_tests.rs", true)]);
    assert!(manifest_roots("[[bin]]\npath = \"src/other/main.rs\"\n", "src/native/").is_err());
    assert!(manifest_roots("[package]\n", "src/native/").is_err(), "no target at all");
}

/// The real crate:
/// every file is reached from the manifest's targets,
/// support is test-only,
/// and the shipped
/// entry points are release code.
#[test]
fn the_real_crate_is_fully_classified() {
    let listing: Vec<SourceFile> = classified_sources(&source_root(), &real_roots()).expect("the crate's own sources");
    assert!(!test_only_of(&listing, "lib.rs"));
    assert!(!test_only_of(&listing, "main.rs"));
    assert!(!test_only_of(&listing, "service/service_main.rs"), "the service mode is release code of the library, not a target root");
    assert!(test_only_of(&listing, "binary/binary_tests.rs"), "a [[test]] target is test-only");
    assert!(test_only_of(&listing, "testing/test_support.rs"), "a `#[cfg(test)]` module of lib.rs is test-only");
    assert!(test_only_of(&listing, "source_guard/no_self_invocation_rules_tests.rs"), "a child of a test-only module");
}
