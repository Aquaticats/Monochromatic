//! Differential controls for the typed-value proof:
//!  a value Git rejects or warns about makes the readers decline, whichever key and command carries it;
//!  every spelling the proof accepts is one real Git runs silently with; and every key Git documents, set to each
//!  kind of malformed value, either makes the readers decline or leaves them equal to Git on all eleven query shapes.

use super::*;
use crate::repo_read_layout::read_layout_with;
use crate::repo_read_query::{Answer, Query, answer_with};
use crate::repo_read_query::tests::scope;
use crate::test_support::{REAL_GIT, fixture, git, git_output, remove, repository};
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;

/// The malformed spellings the sweep tries for every documented key: a word no type reads, a bare name, an empty value,
/// a negative number, a small number, an overflowing number and a `~user` path of a user that does not exist.
const MALFORMED_VARIANTS: [Option<&str>; 7] = [Some("maybe"), None, Some(""), Some("-1"), Some("3"), Some("99999999999999999999"), Some("~no-such-user-here/x")];

/// Sections whose text values name remotes, branches and rewrites that the query readers interpret.
const SEMANTIC_SECTIONS: [&str; 3] = ["branch", "remote", "url"];

/// How many keys one sweep batch holds; a batch that shows a difference is split until the key is found.
const BATCH_SIZE: usize = 24;

/// A repository the eleven query shapes answer on: one commit, a staged file, a remote and an upstream.
fn query_repository(root: &Path) -> PathBuf {
    let repo = repository(root, "typed");
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["remote", "add", "origin", "https://example.invalid/r.git"]);
    git(repo.as_path(), &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    git(repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    return repo;
}

/// One query shape: the arguments and the standard input.
type Shape = (Vec<String>, Option<Vec<u8>>);

/// The eleven shapes the readers answer, with `head` as the object name the shapes that take one use.
fn shapes(head: &str) -> Vec<Shape> {
    let plain = |arguments: &[&str]| -> Shape { return (arguments.iter().map(|argument| { return (*argument).to_string(); }).collect(), None); };
    return vec![
        plain(&["ls-files", "--stage", "-z", "--full-name"]),
        plain(&["diff-index", "--cached", "--raw", "-z", "--no-renames", "--no-abbrev", head]),
        plain(&["diff", "--cached", "--name-only", "-z", head]),
        plain(&["diff-tree", "--root", "--no-commit-id", "-r", "-z", "-m", "--no-renames", "--no-abbrev", head]),
        plain(&["show", "--no-patch", "--format=%s", head]),
        plain(&["config", "--list", "-z", "--show-scope"]),
        plain(&["remote", "get-url", "origin"]),
        plain(&["rev-parse", "--verify", "--quiet", "HEAD^{commit}"]),
        plain(&["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]),
        plain(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"]),
        plain(&["symbolic-ref", "--quiet", "HEAD"]),
        (vec!["cat-file".to_string(), "--batch".to_string()], Some(format!("{head}\n").into_bytes())),
    ];
}

/// What real Git prints and returns for a shape.
fn real(repo: &Path, shape: &Shape) -> (Vec<u8>, Vec<u8>, Option<i32>) {
    let mut child = std::process::Command::new(REAL_GIT)
        .current_dir(repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("HOME", repo.join("no-home"))
        .args(&shape.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Git");
    let mut stdin = child.stdin.take().expect("stdin");
    if let Some(input) = &shape.1 {
        stdin.write_all(input.as_slice()).expect("input");
    }
    drop(stdin);
    let output = child.wait_with_output().expect("output");
    return (output.stdout, output.stderr, output.status.code());
}

/// What the readers answer for a shape.
fn ours(root: &Path, repo: &Path, shape: &Shape) -> Option<Answer> {
    let given: Vec<OsString> = shape.0.iter().map(OsString::from).collect();
    return answer_with(&Query { arguments: given.as_slice(), directory: Some(repo), variables: &[], input: shape.1.as_deref() }, Vec::new(), &scope(root));
}

/// The `[section "subsection"]` header and the line for one key and value, in Git's configuration syntax.
fn config_text(key: &str, value: Option<&str>) -> String {
    let (section, rest) = key.split_once('.').expect("a dotted key");
    let (header, name) = match rest.rsplit_once('.') {
        Some((subsection, name)) => (format!("[{section} \"{subsection}\"]"), name),
        None => (format!("[{section}]"), rest),
    };
    return match value {
        Some(text) => format!("{header}\n\t{name} = {text}\n"),
        None => format!("{header}\n\t{name}\n"),
    };
}

/// A fixture repository with a base configuration text that trials append to.
struct Trial {
    /// The fixture directory.
    root: PathBuf,
    /// The repository.
    repo: PathBuf,
    /// The configuration file text before any trial.
    base: String,
    /// The shapes, built once from the repository's head.
    shapes: Vec<Shape>,
}

impl Trial {
    fn new(name: &str) -> Trial {
        let root = fixture(name);
        let repo = query_repository(root.as_path());
        let head = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8").trim().to_string();
        let base = std::fs::read_to_string(repo.join(".git/config")).expect("config");
        let shapes = shapes(head.as_str());
        return Trial { root, repo, base, shapes };
    }

    /// Replace the repository's configuration with the base plus `extra`.
    fn configure(&self, extra: &str) {
        std::fs::write(self.repo.join(".git/config"), format!("{}{extra}", self.base)).expect("config");
    }

    /// Whether the readers answer the location at all under the current configuration.
    fn answers_location(&self) -> bool {
        return read_layout_with(self.repo.as_path(), &[], &scope(self.root.as_path())).is_some();
    }

    /// The shapes on which the readers answered and differ from Git, plus the location when it differs.
    fn disagreements(&self) -> Vec<String> {
        let mut found: Vec<String> = Vec::new();
        if !self.answers_location() {
            return found;
        }
        let location: Shape = (
            ["rev-parse", "--path-format=absolute", "--is-bare-repository", "--git-dir", "--git-common-dir", "--show-toplevel", "--show-prefix"].iter().map(|word| { return (*word).to_string(); }).collect(),
            None,
        );
        let (_, _, code) = real(self.repo.as_path(), &location);
        if code != Some(0) {
            found.push(format!("location: the readers answered and Git exits {code:?}"));
        }
        for shape in &self.shapes {
            let Some(answer) = ours(self.root.as_path(), self.repo.as_path(), shape) else {
                continue;
            };
            let (stdout, stderr, code) = real(self.repo.as_path(), shape);
            if answer.stdout != stdout || answer.stderr != stderr || Some(answer.code) != code {
                found.push(format!("{:?}: readers {:?} {:?} {}, Git {:?} {:?} {code:?}", shape.0, String::from_utf8_lossy(&answer.stdout), String::from_utf8_lossy(&answer.stderr), answer.code, String::from_utf8_lossy(&stdout), String::from_utf8_lossy(&stderr)));
            }
        }
        return found;
    }

    /// Whether Git prints exactly what it prints under the base configuration on every shape.
    fn git_is_unchanged_by(&self, extra: &str) -> Option<String> {
        self.configure("");
        let before: Vec<(Vec<u8>, Vec<u8>, Option<i32>)> = self.shapes.iter().map(|shape| { return real(self.repo.as_path(), shape); }).collect();
        self.configure(extra);
        for (shape, expected) in self.shapes.iter().zip(before) {
            let now = real(self.repo.as_path(), shape);
            // The listing of the configuration legitimately gains the entry.
            if shape.0[0] != "config" && now != expected {
                return Some(format!("{:?}: {:?} {:?} {:?}", shape.0, String::from_utf8_lossy(&now.0), String::from_utf8_lossy(&now.1), now.2));
            }
        }
        return None;
    }

    fn finish(self) {
        remove(self.root.as_path());
    }
}

/// Every pair in `pairs` that, set alone, makes the readers answer where Git differs; the batch is split to find them.
fn culprits(trial: &Trial, pairs: &[(String, Option<String>)]) -> Vec<String> {
    let text: String = pairs.iter().map(|(key, value)| { return config_text(key.as_str(), value.as_deref()); }).collect();
    trial.configure(text.as_str());
    // A batch the readers decline as a whole may hide a key they would answer for, so it is split down to single keys.
    let declined_as_a_whole: bool = !trial.answers_location();
    if !declined_as_a_whole && trial.disagreements().is_empty() {
        return Vec::new();
    }
    if pairs.len() == 1 {
        return if declined_as_a_whole { Vec::new() } else { vec![format!("{:?}: {}", pairs[0], trial.disagreements().join("; "))] };
    }
    let (left, right) = pairs.split_at(pairs.len() / 2);
    let mut found: Vec<String> = culprits(trial, left);
    found.extend(culprits(trial, right));
    return found;
}

/// The names `git help --config` lists, with a placeholder such as `<name>` filled in and the patterns dropped.
fn documented_keys() -> Vec<String> {
    let listing = git_output(Path::new("/"), &["help", "--config"]);
    assert!(listing.status.success(), "git help --config");
    let mut keys: Vec<String> = String::from_utf8(listing.stdout)
        .expect("UTF-8")
        .lines()
        .filter(|line| { return line.contains('.') && !line.contains('*') && !line.contains(':') && !line.contains(' '); })
        .map(|line| {
            let mut spelled: String = String::new();
            let mut rest: &str = line;
            while let Some((before, after)) = rest.split_once('<') {
                spelled.push_str(before);
                let (_, tail) = after.split_once('>').expect("a closed placeholder");
                spelled.push_str(if line.starts_with("branch.") { "main" } else if line.starts_with("remote.") { "origin" } else { "x" });
                rest = tail;
            }
            spelled.push_str(rest);
            return spelled;
        })
        .collect();
    keys.sort();
    keys.dedup();
    return keys;
}

/// Keys of each kind that real Git 2.56.0 rejected when probed, with the malformed value that it rejected.
///  Each must make the readers decline (the location included), and Git must indeed differ from the base.
#[test]
fn malformed_values_git_rejects_make_the_readers_decline() {
    let trial = Trial::new("repo-read-typed-rejected");
    let cases: [(&str, Option<&str>); 22] = [
        ("core.fileMode", Some("maybe")),
        ("core.compression", Some("maybe")),
        ("core.looseCompression", Some("")),
        ("core.checkStat", Some("maybe")),
        ("core.abbrev", Some("3")),
        ("core.commentChar", Some("")),
        ("core.fsmonitor", Some("maybe")),
        ("core.fsync", Some("maybe")),
        ("core.fsyncObjectFiles", Some("true")),
        ("core.excludesFile", Some("~no-such-user-here/x")),
        ("user.name", None),
        ("advice.statusHints", Some("maybe")),
        ("branch.autoSetupRebase", Some("maybe")),
        ("color.diff.new", Some("maybe")),
        ("color.grep.x", Some("red")),
        ("diff.context", Some("-1")),
        ("diff.algorithm", Some("maybe")),
        ("index.version", Some("99999999999999999999")),
        ("push.default", Some("maybe")),
        ("remote.origin.prune", Some("maybe")),
        ("remote.origin.url", None),
        ("transfer.credentialsInUrl", Some("maybe")),
    ];
    for (key, value) in cases {
        let extra: String = config_text(key, value);
        // The control must be able to show a difference: Git either fails or says something under this value.
        assert!(trial.git_is_unchanged_by(extra.as_str()).is_some(), "{key} = {value:?}: Git accepts it silently, so it proves nothing");
        trial.configure(extra.as_str());
        assert!(!trial.answers_location(), "{key} = {value:?}: the readers answered the location");
        assert!(trial.disagreements().is_empty(), "{key} = {value:?}");
        for shape in &trial.shapes {
            assert!(ours(trial.root.as_path(), trial.repo.as_path(), shape).is_none(), "{key} = {value:?}: the readers answered {:?}", shape.0);
        }
    }
    // The same values through a global file, an included file and the environment.
    let global = trial.root.join("global-config");
    std::fs::write(&global, config_text("core.fileMode", Some("maybe"))).expect("global");
    let through_global = ScopeEnvironment { global: Some(global.clone()), ..scope(trial.root.as_path()) };
    trial.configure("");
    assert!(read_layout_with(trial.repo.as_path(), &[], &through_global).is_none(), "a global file with a malformed value");
    let included = trial.root.join("included-config");
    std::fs::write(&included, config_text("diff.context", Some("maybe"))).expect("included");
    trial.configure(format!("[include]\n\tpath = {}\n", included.display()).as_str());
    assert!(!trial.answers_location(), "an included file with a malformed value");
    trial.configure("");
    let injected: Vec<(OsString, OsString)> = vec![
        (OsString::from("GIT_CONFIG_COUNT"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_KEY_0"), OsString::from("core.fileMode")),
        (OsString::from("GIT_CONFIG_VALUE_0"), OsString::from("maybe")),
    ];
    assert!(read_layout_with(trial.repo.as_path(), injected.as_slice(), &scope(trial.root.as_path())).is_none(), "an injected entry with a malformed value");
    trial.finish();
}

/// Every word and number the proof accepts is one real Git runs silently with on every shape, and the readers keep answering.
#[test]
fn accepted_spellings_are_silent_in_git_and_keep_the_readers_answering() {
    let trial = Trial::new("repo-read-typed-accepted");
    let mut pairs: Vec<(String, Option<String>)> = Vec::new();
    for key in documented_keys() {
        let Some(kind) = kind_of(key.as_str()) else {
            continue;
        };
        // The query readers read these values themselves and decline the ones they cannot follow, so Git changing its
        // answer for an arbitrary text value is no evidence about the proof.
        if SEMANTIC_SECTIONS.iter().any(|section| { return key.starts_with(format!("{section}.").as_str()); }) && matches!(kind, Kind::Text) {
            continue;
        }
        let mut spellings: Vec<Option<String>> = vec![None, Some(String::new())];
        match kind {
            Kind::Boolean | Kind::IntegerOrBoolean => spellings.extend(["true", "yes", "on", "false", "no", "off", "1", "0", "-1", "TRUE"].map(|word| { return Some(word.to_string()); })),
            Kind::BooleanOr(words) | Kind::OneOf { words, .. } => spellings.extend(words.iter().map(|word| { return Some((*word).to_string()); })),
            Kind::Words { words, negatable, .. } => {
                spellings.extend(words.iter().map(|word| { return Some((*word).to_string()); }));
                if negatable {
                    spellings.extend(words.iter().take(2).map(|word| { return Some(format!("-{word}")); }));
                }
                spellings.push(Some(words.iter().take(2).copied().collect::<Vec<&str>>().join(",")));
            }
            Kind::Integer { low, high } => spellings.extend([low, high, (low + high) / 2].map(|number| { return Some(number.to_string()); })),
            Kind::Color => spellings.extend(["red", "bold red", "red blue", "bold ul green", "brightblue", "12", "#00ff7f", "nobold", "no-dim", "normal", "default"].map(|word| { return Some(word.to_string()); })),
            Kind::Abbreviation => spellings.extend(["auto", "false", "4", "40"].map(|word| { return Some(word.to_string()); })),
            Kind::Refspec => spellings.extend(["refs/heads/*:refs/remotes/origin/*", "+refs/heads/*:refs/remotes/origin/*", "refs/heads/main:refs/heads/main"].map(|word| { return Some(word.to_string()); })),
            Kind::Text | Kind::Pathname | Kind::CommentText => spellings.extend(["plain", "~/x", "/abs/path"].map(|word| { return Some(word.to_string()); })),
            Kind::FalseOnly => spellings.extend(["false", "0"].map(|word| { return Some(word.to_string()); })),
            Kind::Refused => {}
        }
        for spelling in spellings {
            if is_accepted(kind, spelling.as_deref().map(str::as_bytes)) {
                pairs.push((key.clone(), spelling));
            }
        }
    }
    assert!(!pairs.is_empty(), "no accepted spelling was found to test");
    // One batch per spelling index would repeat keys; instead each key's spellings run one at a time in key-disjoint batches.
    let mut failures: Vec<String> = Vec::new();
    let mut rounds: Vec<Vec<(String, Option<String>)>> = Vec::new();
    for pair in pairs {
        match rounds.iter_mut().find(|round| { return round.len() < BATCH_SIZE && !round.iter().any(|(key, _)| { return *key == pair.0; }); }) {
            Some(round) => round.push(pair),
            None => rounds.push(vec![pair]),
        }
    }
    for round in rounds {
        let text: String = round.iter().map(|(key, value)| { return config_text(key.as_str(), value.as_deref()); }).collect();
        if trial.git_is_unchanged_by(text.as_str()).is_none() && trial.answers_location() && trial.disagreements().is_empty() {
            continue;
        }
        // Find the pairs that fail alone.
        for pair in round {
            let single: String = config_text(pair.0.as_str(), pair.1.as_deref());
            if let Some(why) = trial.git_is_unchanged_by(single.as_str()) {
                failures.push(format!("{pair:?}: Git is not silent: {why}"));
            } else if !trial.answers_location() {
                failures.push(format!("{pair:?}: the readers declined an accepted spelling"));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
    trial.finish();
}

/// Every documented key, set to each malformed spelling, either makes the readers decline or leaves them equal to Git.
#[test]
fn every_documented_key_with_a_malformed_value_is_declined_or_equals_git() {
    let trial = Trial::new("repo-read-typed-sweep");
    let keys: Vec<String> = documented_keys();
    assert!(keys.len() > 100, "git help --config listed only {} keys", keys.len());
    let mut failures: Vec<String> = Vec::new();
    for variant in MALFORMED_VARIANTS {
        for chunk in keys.chunks(BATCH_SIZE) {
            let pairs: Vec<(String, Option<String>)> = chunk.iter().map(|key| { return (key.clone(), variant.map(str::to_string)); }).collect();
            failures.extend(culprits(&trial, pairs.as_slice()));
        }
    }
    trial.configure("");
    assert!(failures.is_empty(), "{} keys make the readers answer where Git differs: {failures:#?}", failures.len());
    trial.finish();
}

/// The number reader refuses what Git reads differently: octal, hexadecimal, overflow, and a missing digit.
#[test]
fn numbers_are_read_only_when_decimal_and_small() {
    assert_eq!(number_of("12"), Some(12));
    assert_eq!(number_of("-3"), Some(-3));
    assert_eq!(number_of("2k"), Some(2 << 10));
    assert_eq!(number_of("1G"), Some(1 << 30));
    for refused in ["", "-", "k", "010", "0x10", "1.5", "1234567890", "9999999999g", "1 2", "+"] {
        assert_eq!(number_of(refused), None, "{refused:?}");
    }
}
