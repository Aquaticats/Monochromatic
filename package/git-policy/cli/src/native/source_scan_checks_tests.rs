//! Controls of the shared source-scan helpers, on filesystem fixtures: a nested layout is walked and resolved, test-only
//! support is told from release code by the module tree, and a tree that cannot be trusted is an error.

use super::declaration::{Declaration, declarations, module_name};
use super::{SourceFile, classified_sources, manifest_roots, module_tree, real_roots, release_root, source_root, write_tree};
use crate::source_scan_tests::walk::discover;
use crate::test_support::fixture;

/// The classification of one path in a listing, or a panic naming the path when the listing lacks it.
fn test_only_of(listing: &[SourceFile], path: &str) -> bool {
    return listing.iter().find(|file| return file.path == path).unwrap_or_else(|| panic!("{path} is listed")).test_only;
}

/// The declarations of a text, or a panic when the text cannot be read.
fn declared(text: &str) -> Vec<Declaration> {
    return declarations(text).expect("readable declarations");
}

/// A declaration reads with its `path` attribute and with every spelling of a test gate, and only a gate that holds
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

/// An item between an attribute and a module detaches the attribute, so a gate on a function never gates the module
/// after it; an indented declaration and an unreadable `path` attribute are errors, not guesses.
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

/// A nested layout resolves the way the compiler resolves it: a `mod.rs` directory, a sibling directory named after
/// a file, and a `path` attribute, with test gates inherited by every child.
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

/// A file reached once through a gate and once without is release, whichever is followed first.
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

/// A tree that is not what the compiler reads is an error: a declaration with no file, a declaration with two files,
/// a source file nothing declares, and a root that is missing.
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

/// The walk lists nested files by relative name, and an empty tree and an absent root are errors.
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

/// A link in the tree may point outside it or nowhere, so the walk is an error and not a listing.
#[cfg(unix)]
#[test]
fn the_walk_refuses_a_link() {
    let linked = fixture("scan-walk-link");
    write_tree(&linked, &[("real.rs", "")]);
    std::os::unix::fs::symlink(linked.join("nowhere.rs"), linked.join("dangling.rs")).expect("a dangling link");
    assert!(discover(&linked).is_err(), "a link may point outside the tree or nowhere");
}

/// The manifest's library and binaries are release roots, its test targets are test-only roots, and a target
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

/// The real crate: every file is reached from the manifest's targets, support is test-only, and the shipped
/// entry points are release code.
#[test]
fn the_real_crate_is_fully_classified() {
    let listing: Vec<SourceFile> = classified_sources(&source_root(), &real_roots()).expect("the crate's own sources");
    assert!(!test_only_of(&listing, "lib.rs"));
    assert!(!test_only_of(&listing, "main.rs"));
    assert!(!test_only_of(&listing, "service_main.rs"));
    assert!(test_only_of(&listing, "binary_tests.rs"), "a [[test]] target is test-only");
    assert!(test_only_of(&listing, "test_support.rs"), "a `#[cfg(test)]` module of lib.rs is test-only");
    assert!(test_only_of(&listing, "no_self_invocation_rules_tests.rs"), "a child of a test-only module");
}
