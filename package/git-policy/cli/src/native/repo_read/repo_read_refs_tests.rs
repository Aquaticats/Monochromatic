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
    remove(root.as_path());
}

/// A name Git accepts and the reader takes is the same name:
/// Git's own `check-ref-format` is asked about every candidate,
/// and the reading of an absent name answers `Missing` exactly when the reader took the name,
/// so the alphabet is Git's and no wider.
#[test]
fn the_names_the_reader_takes_are_the_names_git_accepts() {
    let root = fixture("repo-read-refs-alphabet");
    let repo = repository(root.as_path(), "repo");
    let refs = Refs::new(repo.join(".git").as_path(), 40);
    let components = [
        "a+b",
        "@scope",
        "@",
        "a@b",
        "a{b",
        "a}b",
        "a@b{",
        "a@",
        "{",
        "a!b",
        "a#b",
        "a$b",
        "a%b",
        "a&b",
        "a'b",
        "a(b",
        "a)b",
        "a,b",
        "a;b",
        "a<b",
        "a=b",
        "a>b",
        "a]b",
        "a|b",
        "a`b",
        "v1.",
        "a.b",
        "a_b-c.d",
        "a b",
        "a*b",
        "a:b",
        "a?b",
        "a[b",
        "a\\b",
        "a^b",
        "a~b",
        "a..b",
        "a@{b",
        ".hidden",
        "x.lock",
        "..",
        ".",
    ];
    for component in components {
        let name: String = format!("refs/heads/{component}");
        let accepted: bool = git_output(repo.as_path(), &["check-ref-format", name.as_str()]).status.success();
        // A name the reader takes is looked for and found absent; one it refuses is `Unsure`, whatever exists.
        let taken: bool = matches!(refs.lookup(name.as_str()), Lookup::Missing);
        assert_eq!(taken, accepted, "Git's verdict on {name:?} is {accepted}");
    }
    assert_eq!(refs.lookup("heads/x"), Lookup::Unsure, "a name without the refs/ prefix is not a ref name");
    remove(root.as_path());
}

/// A scoped tag and a branch with a plus are listed and resolved as Git lists and resolves them,
/// loose,
/// packed and in a mix,
/// and a damaged line still ends the read.
#[test]
fn scoped_and_plus_names_are_listed_as_git_lists_them() {
    let root = fixture("repo-read-refs-scoped-tag");
    let repo = repository(root.as_path(), "repo");
    git(repo.as_path(), &["tag", "-a", "-m", "m", "@scope/package@1.0.0"]);
    git(repo.as_path(), &["tag", "v1+meta"]);
    git(repo.as_path(), &["branch", "a+b"]);
    let refs = Refs::new(repo.join(".git").as_path(), 40);
    let head: String = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8").trim().to_string();
    let tag: String = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "refs/tags/@scope/package@1.0.0"]).stdout).expect("UTF-8").trim().to_string();
    assert_eq!(refs.resolve("refs/tags/@scope/package@1.0.0"), Some(tag), "the scoped tag resolves to the object Git names");
    assert_eq!(refs.resolve("refs/heads/a+b"), Some(head), "the branch beside it resolves");
    for packed in [false, true] {
        if packed {
            git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
        }
        let mine: Vec<(String, String)> = refs.list().expect("every name is one Git lists").into_iter().map(|(name, value)| {
            let object = match value {
                RefValue::Direct(object) => object,
                RefValue::Symbolic(target) => refs.resolve(target.as_str()).expect("resolves"),
            };
            return (name, object);
        }).collect();
        assert_eq!(mine, git_list(repo.as_path()), "packed: {packed}");
    }
    let packed = repo.join(".git/packed-refs");
    let original = std::fs::read_to_string(&packed).expect("packed refs");
    std::fs::write(&packed, format!("{original}not-an-object refs/heads/other\n")).expect("damaged");
    assert_eq!(refs.lookup("refs/heads/missing"), Lookup::Unsure, "a damaged line still ends the read");
    remove(root.as_path());
}

/// The packed count is the number of refs `packed-refs` names:
/// none for a repository Git has not packed,
/// and Git's own listing after a pack that includes a peeled line.
#[test]
fn the_packed_ref_count_is_the_packed_refs_git_writes() {
    let root = fixture("repo-read-refs-count");
    let repo = repository(root.as_path(), "repo");
    let refs = Refs::new(repo.join(".git").as_path(), 40);
    assert_eq!(refs.packed_ref_count(), Some(0), "a repository Git has not packed has no packed refs");
    git(repo.as_path(), &["tag", "-a", "-m", "m", "v1"]);
    git(repo.as_path(), &["branch", "other"]);
    git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
    let packed = std::fs::read_to_string(repo.join(".git/packed-refs")).expect("packed refs");
    assert!(packed.lines().any(|line| { return line.starts_with('^'); }), "the annotated tag's peeled line is in the file");
    assert_eq!(refs.packed_ref_count(), Some(git_list(repo.as_path()).len()), "the count is the listing Git prints");
    remove(root.as_path());
}
