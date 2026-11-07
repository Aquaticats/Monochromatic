//! Controls for the files ref reader:
//! lookups and the full list equal Git's,
//! loose and packed,
//! and odd spellings are unsure.

use super::*;
use crate::test_support::{fixture, git, git_output, remove, repository};

/// The refs Git lists,
/// as `(name, object name)`.
fn git_list(repo: &Path) -> Vec<(String, String)> {
    let output = git_output(repo, &["for-each-ref", "--format=%(refname) %(objectname)"]);
    return String::from_utf8(output.stdout).expect("UTF-8").lines().map(|line| {
        let (name, object) = line.split_once(' ').expect("two words");
        return (name.to_string(), object.to_string());
    }).collect();
}

/// The list and every lookup equal Git's with loose refs,
/// then with the same refs packed,
/// then with a loose override.
#[test]
fn lists_and_lookups_equal_git() {
    let root = fixture("repo-read-refs-unit");
    let repo = repository(root.as_path(), "repo");
    git(repo.as_path(), &["branch", "a/b/c"]);
    git(repo.as_path(), &["tag", "-a", "-m", "m", "v1"]);
    let refs = Refs::new(repo.join(".git").as_path(), 40);
    for packed in [false, true] {
        if packed {
            git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
        }
        let mine: Vec<(String, String)> = refs.list().expect("plain").into_iter().map(|(name, value)| {
            let object = match value {
                RefValue::Direct(object) => object,
                RefValue::Symbolic(target) => refs.resolve(target.as_str()).expect("resolves"),
            };
            return (name, object);
        }).collect();
        assert_eq!(mine, git_list(repo.as_path()), "packed: {packed}");
        for (name, object) in git_list(repo.as_path()) {
            assert_eq!(refs.resolve(name.as_str()), Some(object), "{name}");
        }
    }
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=newer"]);
    assert_eq!(refs.resolve("refs/heads/main"), Some(String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8").trim().to_string()), "a loose file overrides the packed line");
    assert_eq!(refs.lookup("refs/heads/missing"), Lookup::Missing);
    remove(root.as_path());
}

/// A peeled line that does not follow a ref line makes the packed file unsure.
#[test]
fn a_misplaced_peeled_line_is_unsure() {
    let root = fixture("repo-read-refs-peeled");
    let repo = repository(root.as_path(), "repo");
    git(repo.as_path(), &["tag", "-a", "-m", "m", "v1"]);
    git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
    let refs = Refs::new(repo.join(".git").as_path(), 40);
    assert!(refs.list().is_some(), "Git's own packed file is read");
    let packed = repo.join(".git/packed-refs");
    let original = std::fs::read_to_string(&packed).expect("packed refs");
    std::fs::write(&packed, format!("^{}\n{original}", "1".repeat(40))).expect("misplaced");
    assert!(refs.list().is_none(), "a peeled line before any ref");
    remove(root.as_path());
}

/// A name outside the plain alphabet,
/// a link,
/// a damaged value and a symbolic loop are unsure or unresolved.
#[test]
fn odd_spellings_are_unsure() {
    let root = fixture("repo-read-refs-unit-odd");
    let repo = repository(root.as_path(), "repo");
    let refs = Refs::new(repo.join(".git").as_path(), 40);
    assert_eq!(refs.lookup("refs/heads/we ird"), Lookup::Unsure);
    assert_eq!(refs.lookup("HEAD"), Lookup::Unsure);
    std::os::unix::fs::symlink("main", repo.join(".git/refs/heads/linked")).expect("link");
    assert_eq!(refs.lookup("refs/heads/linked"), Lookup::Unsure);
    std::fs::write(repo.join(".git/refs/heads/damaged"), "not a name\n").expect("damaged");
    assert_eq!(refs.lookup("refs/heads/damaged"), Lookup::Unsure);
    assert!(refs.list().is_none(), "a damaged file makes the whole list unsure");
    std::fs::remove_file(repo.join(".git/refs/heads/damaged")).expect("remove");
    std::fs::remove_file(repo.join(".git/refs/heads/linked")).expect("remove");
    std::fs::write(repo.join(".git/refs/heads/one"), "ref: refs/heads/two\n").expect("one");
    std::fs::write(repo.join(".git/refs/heads/two"), "ref: refs/heads/one\n").expect("two");
    assert_eq!(refs.resolve("refs/heads/one"), None, "a loop ends");
    assert!(is_plain_ref_name("refs/heads/a.b-c_d/e"));
    assert!(!is_plain_ref_name("refs/heads/a..b"));
    assert!(!is_plain_ref_name("refs/heads/x.lock"));
    assert!(!is_plain_ref_name("heads/x"));
    remove(root.as_path());
}

/// A packed ref with a name outside the plain alphabet does not make a lookup of another ref unsure,
/// the list still is,
/// and
///  the odd name itself is never answered;
/// a damaged line still ends the read.
#[test]
fn an_unusual_packed_name_does_not_hide_a_plain_branch() {
    let root = fixture("repo-read-refs-scoped-tag");
    let repo = repository(root.as_path(), "repo");
    git(repo.as_path(), &["tag", "-a", "-m", "m", "@scope/package@1.0.0"]);
    git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
    let refs = Refs::new(repo.join(".git").as_path(), 40);
    let head: String = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8").trim().to_string();
    assert_eq!(refs.resolve("refs/heads/main"), Some(head), "the branch beside the scoped tag resolves");
    assert_eq!(refs.lookup("refs/heads/missing"), Lookup::Missing, "and an absent one is absent");
    assert_eq!(refs.lookup("refs/tags/@scope/package@1.0.0"), Lookup::Unsure, "the unusual name itself is not answered");
    assert!(refs.list().is_none(), "the full list needs every name");
    let packed = repo.join(".git/packed-refs");
    let original = std::fs::read_to_string(&packed).expect("packed refs");
    std::fs::write(&packed, format!("{original}not-an-object refs/heads/other\n")).expect("damaged");
    assert_eq!(refs.lookup("refs/heads/missing"), Lookup::Unsure, "a damaged line still ends the read");
    remove(root.as_path());
}
