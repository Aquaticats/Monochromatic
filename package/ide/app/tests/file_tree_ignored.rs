//! Rows carry what the rule files said about their listing,
//!  and an answer is accepted or dropped together with the listing it belongs to.

/// The tree and the answer type it stores.
use ide_app::{
    file_tree::FileTree,
    git_ignore::{IgnoreAnswer, RuleStamps, Stamp},
    workspace::DirectoryEntry,
};
/// Synthetic roots keep these tests independent of filesystem timing.
use std::{collections::BTreeSet, ffi::OsString, path::Path};

/// Build a synthetic child without performing filesystem I/O.
fn child(parent: &str, name: &str, is_directory: bool) -> DirectoryEntry {
    return DirectoryEntry {
        name: name.into(),
        path: Path::new(parent).join(name),
        is_directory,
    };
}

/// An answer naming the given entries,
///  with stamps of a work tree whose rules never change.
fn answer(names: &[&str]) -> IgnoreAnswer {
    let mut owned: BTreeSet<OsString> = BTreeSet::new();
    for name in names {
        owned.insert(OsString::from(name));
    }
    return IgnoreAnswer {
        names: owned,
        stamps: RuleStamps {
            top: Some("/project".into()),
            ..RuleStamps::default()
        },
    };
}

/// The names of the rows that are marked ignored.
fn dimmed(tree: &FileTree) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for row in tree.rows() {
        if row.ignored {
            names.push(row.entry.name.to_string_lossy().into_owned());
        }
    }
    return names;
}

/// A snapshot without an answer marks nothing,
///  and an answer marks exactly the entries it names in its own directory.
#[test]
fn rows_are_marked_from_the_answer_of_their_own_directory() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    tree.apply_listing(
        root,
        vec![
            child("/project", "build", true),
            child("/project", "src", true),
        ],
    )
    .expect("root without an answer");
    assert!(dimmed(&tree).is_empty());
    tree.apply_listing_with(
        root,
        vec![
            child("/project", "build", true),
            child("/project", "src", true),
        ],
        answer(&["build"]),
    )
    .expect("root with an answer");
    tree.toggle(Path::new("/project/build"))
        .expect("expand build");
    tree.toggle(Path::new("/project/src")).expect("expand src");
    // The same name in two directories is judged by its own directory's answer.
    tree.apply_listing_with(
        Path::new("/project/build"),
        vec![child("/project/build", "out.o", false)],
        answer(&["out.o"]),
    )
    .expect("build listing");
    tree.apply_listing_with(
        Path::new("/project/src"),
        vec![
            child("/project/src", "out.o", false),
            child("/project/src", "build", false),
        ],
        answer(&[]),
    )
    .expect("src listing");
    assert_eq!(dimmed(&tree), vec!["build", "out.o"]);
    let rows = tree.rows();
    assert!(
        rows[0].ignored && rows[0].entry.is_directory,
        "the ignored directory's own row is marked"
    );
    assert!(rows[1].ignored, "build/out.o");
    assert!(
        !rows[3].ignored && !rows[4].ignored,
        "src/out.o and src/build keep their own answer"
    );
}

/// A reply that was superseded is dropped with its answer:
///  the current request's answer is the one shown.
#[test]
fn a_stale_reply_cannot_install_its_answer() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    let old = tree.begin_listing(root).expect("first request");
    let current = tree.begin_listing(root).expect("replacement request");
    let both = vec![
        child("/project", "a.log", false),
        child("/project", "b.txt", false),
    ];
    assert!(
        !tree
            .complete_listing_with(&old, Ok(both.clone()), answer(&["a.log"]))
            .expect("stale reply"),
        "the superseded reply is reported as not applied"
    );
    assert!(tree.rows().is_empty(), "a stale listing shows nothing");
    assert!(
        tree.complete_listing_with(&current, Ok(both), answer(&["b.txt"]))
            .expect("current reply")
    );
    assert_eq!(
        dimmed(&tree),
        vec!["b.txt"],
        "only the current answer is shown"
    );
    // A late stale reply after the current one changes nothing either.
    assert!(
        !tree
            .complete_listing_with(
                &old,
                Ok(vec![child("/project", "a.log", false)]),
                answer(&["a.log"])
            )
            .expect("late stale reply")
    );
    assert_eq!(dimmed(&tree), vec!["b.txt"]);
}

/// A failed read keeps the last snapshot and its answer.
#[test]
fn a_failed_read_keeps_the_previous_answer() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    tree.apply_listing_with(
        root,
        vec![child("/project", "a.log", false)],
        answer(&["a.log"]),
    )
    .expect("first listing");
    let retry = tree.begin_listing(root).expect("retry");
    assert!(
        tree.complete_listing_with(
            &retry,
            Err(anyhow::anyhow!("read failed")),
            IgnoreAnswer::default()
        )
        .is_err()
    );
    assert_eq!(dimmed(&tree), vec!["a.log"]);
}

/// Changed rule stamps ask for a reread of every shown directory;
///  a first listing,
///  a changed set of names,
///  and an unchanged answer do not.
#[test]
fn only_changed_rule_stamps_raise_the_reread_flag() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    let entries = vec![child("/project", "a.log", false)];
    tree.apply_listing_with(root, entries.clone(), answer(&["a.log"]))
        .expect("first");
    assert!(
        !tree.take_ignore_sources_changed(),
        "a first listing has nothing to differ from"
    );
    tree.apply_listing_with(root, entries.clone(), answer(&[]))
        .expect("same stamps, other names");
    assert!(
        !tree.take_ignore_sources_changed(),
        "names alone do not raise the flag"
    );
    let mut moved = answer(&["a.log"]);
    moved.stamps.own = Stamp::of(Path::new("/proc/self/stat"));
    tree.apply_listing_with(root, entries.clone(), moved.clone())
        .expect("moved stamp");
    assert!(
        tree.take_ignore_sources_changed(),
        "a different .gitignore stamp raises it"
    );
    assert!(!tree.take_ignore_sources_changed(), "taking it clears it");
    tree.apply_listing_with(root, entries, moved)
        .expect("unchanged");
    assert!(!tree.take_ignore_sources_changed());
}

/// A directory removed from its parent takes its answer with it,
///  so a recreated directory starts without a mark.
#[test]
fn a_removed_directory_forgets_its_answer() {
    let root = Path::new("/project");
    let mut tree = FileTree::new(root);
    tree.apply_listing(root, vec![child("/project", "dist", true)])
        .expect("root");
    tree.toggle(Path::new("/project/dist")).expect("expand");
    tree.apply_listing_with(
        Path::new("/project/dist"),
        vec![child("/project/dist", "a.js", false)],
        answer(&["a.js"]),
    )
    .expect("dist");
    assert_eq!(dimmed(&tree), vec!["a.js"]);
    tree.apply_listing(root, vec![]).expect("dist removed");
    tree.apply_listing(root, vec![child("/project", "dist", true)])
        .expect("dist recreated");
    tree.toggle(Path::new("/project/dist"))
        .expect("expand again");
    tree.apply_listing(
        Path::new("/project/dist"),
        vec![child("/project/dist", "a.js", false)],
    )
    .expect("dist again");
    assert!(dimmed(&tree).is_empty());
}
