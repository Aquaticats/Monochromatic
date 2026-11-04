//! Controls for the commit facts: rules read the real repository, candidates read the private index.

use super::*;
use crate::candidate_prediction::CandidateRequest;
use crate::repository_facts::git_facts;
use crate::test_support::{REAL_GIT, fixture, git, remove, repository};
use std::path::PathBuf;

/// The names of the candidates a request lists.
fn names(facts: &mut PrivateIndexFacts<crate::repository_facts::GitFacts>, scope: &str) -> Vec<String> {
    let request: CandidateRequest = CandidateRequest::StagedAtBase {
        base: None,
        pathspecs: vec![OsString::from(format!(":(top,literal){scope}"))],
    };
    let prepared = facts.candidates(&request).expect("candidates");
    return prepared
        .version
        .candidates()
        .iter()
        .map(|candidate| { return String::from_utf8_lossy(candidate.path.as_slice()).into_owned(); })
        .collect();
}

/// Candidates come from the named private index; naming another index changes them; the real index is not read.
#[test]
fn candidates_come_from_the_private_index_and_follow_a_replacement() {
    let root: PathBuf = fixture("commit-facts");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("one.txt"), b"one\n").expect("one");
    std::fs::write(repo.join("two.txt"), b"two\n").expect("two");
    // Private index A holds one.txt; private index B holds two.txt; the real index holds neither.
    for (index, file) in [("a.index", "one.txt"), ("b.index", "two.txt")] {
        let command = std::process::Command::new(REAL_GIT)
            .current_dir(&repo)
            .env("GIT_INDEX_FILE", repo.join(".git").join(index))
            .args(["add", "--", file])
            .env("GIT_CONFIG_GLOBAL", "/nonexistent")
            .status()
            .expect("add");
        assert!(command.success());
    }
    git(repo.as_path(), &["status", "--short"]);
    let inner = git_facts(
        Path::new(REAL_GIT),
        &[OsString::from("-C"), repo.as_os_str().to_os_string()],
        &[],
    );
    let mut facts = PrivateIndexFacts::new(inner, repo.join(".git/a.index").as_path(), repo.join(".git/objects").as_path());
    assert_eq!(names(&mut facts, "one.txt"), vec!["one.txt"]);
    assert_eq!(names(&mut facts, "two.txt"), Vec::<String>::new());
    facts.use_index(repo.join(".git/b.index").as_path(), repo.join(".git/objects").as_path());
    assert_eq!(names(&mut facts, "one.txt"), Vec::<String>::new());
    assert_eq!(names(&mut facts, "two.txt"), vec!["two.txt"]);
    remove(root.as_path());
}

/// The rule facts answer from the wrapped provider: the location is asked once and remembered.
#[test]
fn rule_facts_are_the_wrapped_providers() {
    let root: PathBuf = fixture("commit-facts-rules");
    let repo: PathBuf = repository(root.as_path(), "repo");
    // A remote that has a branch the local repository lacks: checking out its name would create the branch.
    let upstream: PathBuf = repository(root.as_path(), "upstream");
    git(upstream.as_path(), &["branch", "feature"]);
    git(repo.as_path(), &["remote", "add", "origin", upstream.to_str().expect("path")]);
    git(repo.as_path(), &["fetch", "--quiet", "origin"]);
    let inner = git_facts(
        Path::new(REAL_GIT),
        &[OsString::from("-C"), repo.as_os_str().to_os_string()],
        &[],
    );
    let mut facts = PrivateIndexFacts::new(inner, repo.join(".git/index").as_path(), repo.join(".git/objects").as_path());
    assert!(facts.asked_location().is_none());
    let location = facts.location().expect("location");
    assert!(facts.asked_location().is_some());
    assert_eq!(facts.location().expect("again"), location);
    assert!(facts.index_vs_head().is_ok());
    assert!(facts.sequencer_state().is_ok());
    assert_eq!(facts.remote_guess_creates_branch(OsStr::new("feature")), Ok(true));
    assert_eq!(facts.remote_guess_creates_branch(OsStr::new("nothing")), Ok(false));
    assert_eq!(facts.transaction_context().real_git, PathBuf::from(REAL_GIT));
    // A mutable borrow answers like the provider it borrows; the trait is named so that the impl for the borrow runs.
    let mut borrowed = &mut facts;
    assert_eq!(RepositoryFacts::location(&mut borrowed).expect("borrowed"), location);
    assert!(RepositoryFacts::asked_location(&borrowed).is_some());
    assert_eq!(RepositoryFacts::transaction_context(&borrowed).real_git, PathBuf::from(REAL_GIT));
    assert!(RepositoryFacts::index_vs_head(&mut borrowed).is_ok());
    assert!(RepositoryFacts::sequencer_state(&mut borrowed).is_ok());
    assert_eq!(RepositoryFacts::remote_guess_creates_branch(&mut borrowed, OsStr::new("feature")), Ok(true));
    assert_eq!(RepositoryFacts::remote_guess_creates_branch(&mut borrowed, OsStr::new("nothing")), Ok(false));
    let request: CandidateRequest = CandidateRequest::StagedAtBase { base: None, pathspecs: Vec::new() };
    assert!(RepositoryFacts::candidates(&mut borrowed, &request).is_ok());
    remove(root.as_path());
}
