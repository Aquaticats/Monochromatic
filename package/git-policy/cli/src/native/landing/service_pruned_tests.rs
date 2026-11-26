//! Controls for recognizing content that was removed before a commit landed.

use super::*;
use crate::service_protocol::{
    ALLOWED_MODES,
    Change,
};
use crate::test_support::{
    fixture,
    git_context,
    git_output,
    remove,
};
use std::path::{
    Path,
    PathBuf,
};

const NAME: &str = "0123456789abcdef0123456789abcdef01234567";

fn failure(
    code: EngineFailureCode,
    message: &str,
) -> PolicyEvent {
    return PolicyEvent::EngineFailure {
        code,
        hint: crate::policy_events::EngineFailureHint::OfCode,
        message: message.to_string(),
        trigger: None,
        policy: None,
        path: None,
    };
}

/// The failure a policy raises for a missing object is the removed-content signal;
/// the same code with another cause,
/// and
///  other codes,
/// are not.
#[test]
fn only_a_missing_object_is_removed_content() {
    let missing: String =
        format!("{OBJECT_MISSING_PREFIX}{NAME}. Run `git fsck` to check the object store.");
    assert!(content_was_removed(&[failure(
        EngineFailureCode::ContentUnavailable,
        missing.as_str()
    )]));
    assert!(!content_was_removed(&[failure(
        EngineFailureCode::ContentUnavailable,
        "monochromatic-git-policy-cli could not start Git"
    )]));
    assert!(!content_was_removed(&[failure(
        EngineFailureCode::PolicyIncomplete,
        missing.as_str()
    )]));
    assert!(!content_was_removed(&[]));
}

fn request(blob: &str) -> CommitRequest {
    return CommitRequest {
        id: String::from("x"),
        repository: PathBuf::from("/nowhere"),
        common_dir: PathBuf::from("/nowhere/.git"),
        git_dir: PathBuf::from("/nowhere/.git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: NAME.to_string(),
        changes: vec![Change {
            path: PathBuf::from("a.txt"),
            mode: ALLOWED_MODES[0],
            blob: Some(blob.to_string()),
        }],
        message: String::from("m\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: PathBuf::from("/nowhere/.git/index"),
        scratch: PathBuf::from(
            "/nowhere/.git/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-x-0",
        ),
        shadow: None,
        captured: 0,
        holder: String::new(),
        programs: None,
    };
}

/// An object the request names is present while its loose file is there and missing once the file is gone.
#[test]
fn a_named_object_is_missing_once_its_loose_file_is_gone() {
    let root: PathBuf = fixture("pruned-objects");
    git_output(root.as_path(), &["init", "--quiet"]);
    std::fs::write(root.join("a.txt"), "content\n").expect("file");
    let written = git_output(root.as_path(), &["hash-object", "-w", "a.txt"]);
    assert!(written.status.success());
    let name: String = String::from_utf8_lossy(&written.stdout).trim().to_string();
    let common: PathBuf = root.join(".git");
    let objects: PathBuf = common.join("objects");
    assert!(!prepared_content_missing(
        &git_context(),
        root.as_path(),
        common.as_path(),
        objects.as_path(),
        "sha1",
        &request(name.as_str())
    ));
    std::fs::remove_file(objects.join(&name[..2]).join(&name[2..])).expect("the object is loose");
    assert!(prepared_content_missing(
        &git_context(),
        root.as_path(),
        common.as_path(),
        objects.as_path(),
        "sha1",
        &request(name.as_str())
    ));
    remove(root.as_path());
}

/// Each numbered phase file answers one occurrence:
/// the marker counts how often the phase was reached.
#[test]
fn the_service_phases_are_numbered_by_occurrence() {
    let before: u64 = STAGED_REACHED.load(Ordering::SeqCst);
    // Unarmed, reaching a phase does nothing but count.
    reach_service_phase(Phase::ServiceStaged).expect("unarmed");
    reach_service_phase(Phase::ServiceStaged).expect("unarmed");
    assert_eq!(STAGED_REACHED.load(Ordering::SeqCst), before + 2);
}

/// A fixture repository holding `d/a.txt`,
/// its trees,
/// and an amendment-shaped request that names only the root tree.
struct TreeFixture {
    root: PathBuf,
    blob: String,
    subtree: String,
    request: CommitRequest,
}

fn tree_fixture(name: &str) -> TreeFixture {
    return tree_fixture_in(name, &["init"], false);
}

/// The stores the in-process object reader declines,
/// as the arms of the controls below:
/// the words an arm is named with,
/// the `git init` arguments of its format,
/// and whether it holds an unrelated replace ref.
/// Why:
///  A declined reader asks Git,
///  so a control of what the landing asks has to stand where the refusal happens.
const DECLINED_STORES: &[(&str, &[&str], bool)] = &[
    ("reftable", &["init", "--ref-format=reftable"], false),
    ("files beside a replace ref", &["init"], true),
];

/// `tree_fixture` in the store an arm names.
fn declined_tree_fixture(
    name: &str,
    (label, format, replace): (&str, &[&str], bool),
) -> TreeFixture {
    return tree_fixture_in(
        &format!("{name}-{}", label.replace(' ', "-")),
        format,
        replace,
    );
}

/// `tree_fixture` with the repository initialized by `init`,
/// holding an unrelated replace ref when `replace` asks.
fn tree_fixture_in(
    name: &str,
    init: &[&str],
    replace: bool,
) -> TreeFixture {
    let root: PathBuf = fixture(name);
    let mut opened: Vec<&str> = init.to_vec();
    opened.push("--quiet");
    git_output(root.as_path(), opened.as_slice());
    if replace {
        // A replace ref on an object nothing in this fixture names: the reader declines the store, and what Git
        // answers for the objects this fixture asks about is what it answered before.
        std::fs::write(root.join("replaced.txt"), "replaced\n").expect("file");
        std::fs::write(root.join("standing-in.txt"), "standing in\n").expect("file");
        let named = |path: &str| -> String {
            return String::from_utf8_lossy(
                &git_output(root.as_path(), &["hash-object", "-w", path]).stdout,
            )
            .trim()
            .to_string();
        };
        let replaced: String = named("replaced.txt");
        let standing_in: String = named("standing-in.txt");
        assert!(
            git_output(
                root.as_path(),
                &["replace", replaced.as_str(), standing_in.as_str()]
            )
            .status
            .success(),
            "the unrelated replace ref"
        );
    }
    std::fs::create_dir(root.join("d")).expect("directory");
    std::fs::write(root.join("d").join("a.txt"), "content\n").expect("file");
    assert!(
        git_output(root.as_path(), &["add", "d/a.txt"])
            .status
            .success()
    );
    let text = |arguments: &[&str]| -> String {
        return String::from_utf8_lossy(&git_output(root.as_path(), arguments).stdout)
            .trim()
            .to_string();
    };
    let tree: String = text(&["write-tree"]);
    let blob: String = text(&["rev-parse", ":d/a.txt"]);
    let subtree: String = text(&["rev-parse", format!("{tree}:d").as_str()]);
    let mut request: CommitRequest = request(blob.as_str());
    request.changes = Vec::new();
    request.amend = true;
    request.tree = Some(tree);
    return TreeFixture {
        root,
        blob,
        subtree,
        request,
    };
}

fn remove_loose(
    root: &Path,
    name: &str,
) {
    std::fs::remove_file(
        root.join(".git")
            .join("objects")
            .join(&name[..2])
            .join(&name[2..]),
    )
    .expect("the object is loose");
}

/// An amendment that embeds a whole tree is intact only while every blob and tree below it is:
/// a blob removed from under a
///  surviving tree is removed content,
/// found before the branch moves.
#[test]
fn a_blob_removed_from_under_an_embedded_tree_is_removed_content() {
    let fixed = tree_fixture("pruned-tree-blob");
    let common: PathBuf = fixed.root.join(".git");
    assert!(!batch_content_missing(
        &git_context(),
        fixed.root.as_path(),
        common.as_path(),
        &[&fixed.request]
    ));
    remove_loose(fixed.root.as_path(), fixed.blob.as_str());
    assert!(batch_content_missing(
        &git_context(),
        fixed.root.as_path(),
        common.as_path(),
        &[&fixed.request]
    ));
    assert!(prepared_content_missing(
        &git_context(),
        fixed.root.as_path(),
        common.as_path(),
        common.join("objects").as_path(),
        "sha1",
        &fixed.request
    ));
    remove(fixed.root.as_path());
}

/// A subtree removed from under the embedded tree is removed content too.
#[test]
fn a_subtree_removed_from_under_an_embedded_tree_is_removed_content() {
    let fixed = tree_fixture("pruned-tree-subtree");
    let common: PathBuf = fixed.root.join(".git");
    remove_loose(fixed.root.as_path(), fixed.subtree.as_str());
    assert!(batch_content_missing(
        &git_context(),
        fixed.root.as_path(),
        common.as_path(),
        &[&fixed.request]
    ));
    assert!(prepared_content_missing(
        &git_context(),
        fixed.root.as_path(),
        common.as_path(),
        common.join("objects").as_path(),
        "sha1",
        &fixed.request
    ));
    remove(fixed.root.as_path());
}

/// How many files the shared tree of the batch tests holds below its directory.
const SHARED_FILES: usize = 12;

/// A repository whose root tree holds a directory of `SHARED_FILES` files,
/// with the root tree,
/// and a second root tree that
/// shares that directory and adds one file of its own.
struct SharedTrees {
    root: PathBuf,
    first: String,
    second: String,
    own_blob: String,
}

fn shared_trees(name: &str) -> SharedTrees {
    let root: PathBuf = fixture(name);
    git_output(root.as_path(), &["init", "--quiet"]);
    std::fs::create_dir(root.join("shared")).expect("directory");
    for number in 0..SHARED_FILES {
        std::fs::write(
            root.join("shared").join(format!("file-{number}.txt")),
            format!("content {number}\n"),
        )
        .expect("file");
    }
    assert!(
        git_output(root.as_path(), &["add", "shared"])
            .status
            .success()
    );
    let text = |arguments: &[&str]| -> String {
        return String::from_utf8_lossy(&git_output(root.as_path(), arguments).stdout)
            .trim()
            .to_string();
    };
    let first: String = text(&["write-tree"]);
    std::fs::write(root.join("own.txt"), "own content\n").expect("file");
    assert!(
        git_output(root.as_path(), &["add", "own.txt"])
            .status
            .success()
    );
    let second: String = text(&["write-tree"]);
    let own_blob: String = text(&["rev-parse", ":own.txt"]);
    return SharedTrees {
        root,
        first,
        second,
        own_blob,
    };
}

fn walk_of(
    fixed: &SharedTrees,
    roots: &[&str],
    budget: usize,
) -> ClosureWalk {
    let database =
        ObjectDatabase::open(fixed.root.join(".git").as_path(), None, ObjectFormat::Sha1)
            .expect("the object store opens");
    return walk_tree_closures(&database, roots, ObjectFormat::Sha1, budget);
}

/// A tree that every request of a batch embeds is walked once:
/// the work of a batch of identical roots is the work of one.
#[test]
fn requests_that_share_a_tree_walk_it_once() {
    let fixed = shared_trees("pruned-batch-shared");
    let one: ClosureWalk = walk_of(&fixed, &[fixed.first.as_str()], usize::MAX);
    assert_eq!(one.end, ClosureEnd::Intact);
    assert!(
        one.examined > SHARED_FILES,
        "the walk looked at every file of the shared directory"
    );
    let batch: Vec<&str> = vec![fixed.first.as_str(); SHARED_FILES];
    assert_eq!(
        walk_of(&fixed, &batch, usize::MAX),
        one,
        "a batch of identical roots costs what one root costs"
    );
    remove(fixed.root.as_path());
}

/// Two roots that share a directory cost less together than apart,
/// because the shared directory is walked once.
#[test]
fn roots_that_share_a_subtree_cost_less_together_than_apart() {
    let fixed = shared_trees("pruned-batch-subtree");
    let first: ClosureWalk = walk_of(&fixed, &[fixed.first.as_str()], usize::MAX);
    let second: ClosureWalk = walk_of(&fixed, &[fixed.second.as_str()], usize::MAX);
    let both: ClosureWalk = walk_of(
        &fixed,
        &[fixed.first.as_str(), fixed.second.as_str()],
        usize::MAX,
    );
    assert_eq!(both.end, ClosureEnd::Intact);
    assert!(
        both.examined < first.examined + second.examined,
        "{both:?} {first:?} {second:?}"
    );
    remove(fixed.root.as_path());
}

/// Sharing what was examined never hides a loss:
/// a blob only the second root lists is found missing after the first root
/// was found intact.
#[test]
fn a_loss_below_the_second_root_of_a_batch_is_found() {
    let fixed = shared_trees("pruned-batch-loss");
    remove_loose(fixed.root.as_path(), fixed.own_blob.as_str());
    let common: PathBuf = fixed.root.join(".git");
    let mut first: CommitRequest = request(NAME);
    first.changes = Vec::new();
    first.tree = Some(fixed.first.clone());
    let mut second: CommitRequest = first.clone();
    second.tree = Some(fixed.second.clone());
    assert!(!batch_content_missing(
        &git_context(),
        fixed.root.as_path(),
        common.as_path(),
        &[&first]
    ));
    assert!(batch_content_missing(
        &git_context(),
        fixed.root.as_path(),
        common.as_path(),
        &[&first, &second]
    ));
    assert_eq!(
        walk_of(
            &fixed,
            &[fixed.first.as_str(), fixed.second.as_str()],
            usize::MAX
        )
        .end,
        ClosureEnd::Missing
    );
    remove(fixed.root.as_path());
}

/// The budget counts every tree and entry examined:
/// exactly the work of the walk is allowed,
/// one unit less is not,
/// and the
/// walk then says it does not know instead of answering that something is missing.
/// Past the budget it examines nothing more.
#[test]
fn a_walk_over_its_budget_says_it_does_not_know() {
    let fixed = shared_trees("pruned-batch-budget");
    let needed: usize = walk_of(&fixed, &[fixed.second.as_str()], usize::MAX).examined;
    assert_eq!(
        walk_of(&fixed, &[fixed.second.as_str()], needed).end,
        ClosureEnd::Intact,
        "the budget equal to the work is enough"
    );
    let short: ClosureWalk = walk_of(&fixed, &[fixed.second.as_str()], needed - 1);
    assert_eq!(short.end, ClosureEnd::OutOfBudget);
    assert!(short.examined < needed);
    remove(fixed.root.as_path());
}

/// A submodule commit below the embedded tree lives in another repository,
/// so its absence here is not removed content.
#[test]
fn a_submodule_entry_below_an_embedded_tree_is_not_removed_content() {
    let fixed = tree_fixture("pruned-tree-gitlink");
    let common: PathBuf = fixed.root.join(".git");
    let added = git_output(
        fixed.root.as_path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000,0123456789abcdef0123456789abcdef01234567,sub",
        ],
    );
    assert!(added.status.success());
    let written = git_output(fixed.root.as_path(), &["write-tree"]);
    let mut request: CommitRequest = fixed.request.clone();
    request.tree = Some(String::from_utf8_lossy(&written.stdout).trim().to_string());
    assert!(!batch_content_missing(
        &git_context(),
        fixed.root.as_path(),
        common.as_path(),
        &[&request]
    ));
    assert!(!prepared_content_missing(
        &git_context(),
        fixed.root.as_path(),
        common.as_path(),
        common.join("objects").as_path(),
        "sha1",
        &request
    ));
    remove(fixed.root.as_path());
}

/// What:
///  A blob a prune removed from below an amended tree is removed content where the reader declines the store,
///  so the landing asks Git about the tree instead of importing over the loss.
/// Why:
///  An amendment embeds a whole tree and no list of blobs,
///  so only the tree's own question sees a blob that did not change in the amendment.
///  The reader declines a reftable store and a files store holding a replace ref,
///  and taking that decline as an answer imported over the gone blob.
#[test]
fn a_blob_removed_from_under_an_embedded_tree_is_removed_content_where_the_reader_declines() {
    for store in DECLINED_STORES {
        let (label, _format, _replace) = store;
        let fixed = declined_tree_fixture("pruned-tree-blob-declined", *store);
        let common: PathBuf = fixed.root.join(".git");
        assert!(
            !batch_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                &[&fixed.request]
            ),
            "{label}: the tree is complete"
        );
        remove_loose(fixed.root.as_path(), fixed.blob.as_str());
        // Positive control: Git itself no longer finds the blob the tree names.
        assert_eq!(
            git_output(
                fixed.root.as_path(),
                &["cat-file", "-e", fixed.blob.as_str()]
            )
            .status
            .code(),
            Some(1),
            "{label}: the blob is gone"
        );
        assert!(
            batch_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                &[&fixed.request]
            ),
            "{label}: the amended tree names a gone blob"
        );
        assert!(
            prepared_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                common.join("objects").as_path(),
                "sha1",
                &fixed.request
            ),
            "{label}: the client's check names the tree too"
        );
        remove(fixed.root.as_path());
    }
}

/// What:
///  A commit Git built whose object a prune removed is removed content where the reader declines the store,
///  in the landing's question and in the client's,
///  and a present one is not.
/// Why:
///  A request for a commit Git built names that commit and nothing else,
///  so its presence is the only question either side can ask about the content it captured.
///  Taking the decline as "not gone" moved the branch to a commit no object holds.
#[test]
fn a_gone_commit_git_built_is_removed_content_where_the_reader_declines() {
    for store in DECLINED_STORES {
        let (label, _format, _replace) = store;
        let fixed = declined_tree_fixture("pruned-built-commit-declined", *store);
        let common: PathBuf = fixed.root.join(".git");
        let objects: PathBuf = common.join("objects");
        let text = |arguments: &[&str]| -> String {
            return String::from_utf8_lossy(&git_output(fixed.root.as_path(), arguments).stdout)
                .trim()
                .to_string();
        };
        let built: String = text(&[
            "commit-tree",
            fixed.request.tree.as_deref().expect("the tree"),
            "-m",
            "built",
        ]);
        let mut sent: CommitRequest = fixed.request.clone();
        sent.changes = Vec::new();
        sent.tree = None;
        sent.amend = false;
        sent.commit = Some(built.clone());
        assert!(
            !batch_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                &[&sent]
            ),
            "{label}: the commit is present"
        );
        assert!(
            !prepared_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                objects.as_path(),
                "sha1",
                &sent
            ),
            "{label}: the client's question finds it present"
        );
        remove_loose(fixed.root.as_path(), built.as_str());
        // Positive control: Git itself no longer finds the commit Git built.
        assert_eq!(
            git_output(fixed.root.as_path(), &["cat-file", "-e", built.as_str()])
                .status
                .code(),
            Some(1),
            "{label}: the commit is gone"
        );
        assert!(
            batch_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                &[&sent]
            ),
            "{label}: the landing's question names the gone commit"
        );
        assert!(
            prepared_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                objects.as_path(),
                "sha1",
                &sent
            ),
            "{label}: the client's question names the gone commit"
        );
        remove(fixed.root.as_path());
    }
}

/// What:
///  A submodule entry below an embedded tree is not removed content where the reader declines either,
///  and a blob beside it that is gone is.
/// Why:
///  A submodule names a commit of another repository,
///  so its absence here says nothing about the content the request captured;
///  the positive control shows the same question sees a loss in the same tree,
///  which is what the landing would import over.
#[test]
fn a_submodule_entry_below_an_embedded_tree_is_not_removed_content_where_the_reader_declines() {
    for store in DECLINED_STORES {
        let (label, _format, _replace) = store;
        let fixed = declined_tree_fixture("pruned-tree-gitlink-declined", *store);
        let common: PathBuf = fixed.root.join(".git");
        let added = git_output(
            fixed.root.as_path(),
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                "160000,0123456789abcdef0123456789abcdef01234567,sub",
            ],
        );
        assert!(added.status.success(), "{label}: the gitlink entry");
        let written = git_output(fixed.root.as_path(), &["write-tree"]);
        let mut sent: CommitRequest = fixed.request.clone();
        sent.tree = Some(String::from_utf8_lossy(&written.stdout).trim().to_string());
        assert!(
            !batch_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                &[&sent]
            ),
            "{label}: another repository's commit is not a loss here"
        );
        assert!(
            !prepared_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                common.join("objects").as_path(),
                "sha1",
                &sent
            ),
            "{label}: the client's check agrees"
        );
        remove_loose(fixed.root.as_path(), fixed.blob.as_str());
        assert!(
            batch_content_missing(
                &git_context(),
                fixed.root.as_path(),
                common.as_path(),
                &[&sent]
            ),
            "{label}: the blob beside the gitlink is gone"
        );
        remove(fixed.root.as_path());
    }
}
