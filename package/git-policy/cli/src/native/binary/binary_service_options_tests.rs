//! What:
//!  The command lines that only shape the message or the identity,
//! landed through the service,
//! compared field by field
//!  with the commit native `git commit` makes of the same input.
//! Why:
//!  A service commit that differs from the native one in any stored field (tree,
//! message,
//! author,
//! committer) would be a
//!  silent change of what a person's `-F`,
//! `--author`,
//! `--date`,
//! `--trailer` or `--cleanup` means.

use super::commit::{git_text, remove_settled};
use super::service_commit::{Running, query, start, through_service};
use super::support::{Fixture, direct, fixture, git, repository};
use git_policy_cli::service_journal::DONE_EXTENSION;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Output;

/// The stored fields that must agree,
/// with the author's date (the one date a command line can set).
fn stored(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let formats: [&str; 6] = ["%T", "%B", "%an <%ae>", "%ad", "%cn <%ce>", "%P"];
    // Untrimmed, so trailing whitespace and line breaks of a verbatim message are compared too.
    return formats.map(|format| { return git_text(fixture, repo, &["log", "-1", "--date=iso-strict", format!("--format={format}").as_str()]); }).to_vec();
}

fn finished(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries
            .flatten()
            .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
            .count();
    });
}

/// One command line to compare:
/// the arguments before the separator,
/// and whether the author's date is set by it.
struct Case {
    name: &'static str,
    arguments: Vec<String>,
    date_set: bool,
    edits: bool,
}

fn case(name: &'static str, arguments: &[&str], date_set: bool) -> Case {
    return Case { name, arguments: arguments.iter().map(|argument| { return (*argument).to_string(); }).collect(), date_set, edits: true };
}

/// Every covered option gives the commit native Git gives,
/// and the service (not the transaction) landed it.
#[test]
fn message_and_identity_options_through_the_service_equal_the_native_commit() {
    let fixture: Fixture = fixture("service-options");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let start_commit: String = query(&fixture, &repo, &["rev-parse", "HEAD"]);
    let message_file: PathBuf = fixture.root.join("message.txt");
    std::fs::write(&message_file, "subject  \n\n\n\nbody with trailing   \n\n").expect("message file");
    let message_path: String = message_file.display().to_string();
    let cases: Vec<Case> = vec![
        case("file", &["-F", message_path.as_str()], false),
        case("file attached", &[format!("-F{message_path}").as_str()], false),
        case("author", &["-m", "s", "--author=Other One <other@example.invalid>"], false),
        case("date", &["-m", "s", "--date=2020-01-02T03:04:05Z"], true),
        case("author and date", &["-m", "s", "-m", "b", "--author=A <a@b.invalid>", "--date=2021-05-06T07:08:09+0200"], true),
        case("trailers", &["-m", "s", "--trailer", "Fixes: 12", "--trailer", "Reviewed-by: R <r@example.invalid>"], false),
        case("trailer on a body", &["-m", "s", "-m", "body text", "--trailer=Cc: x <x@example.invalid>"], false),
        case("verbatim", &["-m", "line  \n\n\n\ntrailing  ", "--cleanup=verbatim"], false),
        case("empty message", &["-m", "", "--allow-empty-message"], false),
        case("sign-off", &["-m", "s", "-s"], false),
        case("sign-off with a body", &["-m", "s", "-m", "body", "--signoff"], false),
        case("sign-off after a trailer block", &["-m", "s\n\nCo-authored-by: X <x@example.invalid>", "-s"], false),
        case("sign-off already last", &["-m", "s\n\nSigned-off-by: Fixture <fixture@example.invalid>", "-s"], false),
        case("sign-off of another person last", &["-m", "s\n\nSigned-off-by: Other <o@example.invalid>", "-s"], false),
        case("sign-off and a trailer", &["-m", "s", "-s", "--trailer", "Fixes: 3"], false),
        case("sign-off on a message with a divider", &["-m", "s\n\nbody\n---\nnot a trailer"], false),
        case("strip", &["-m", "# comment\nsubject\n# more\n\nbody", "--cleanup=strip"], false),
        case("fixup", &["--fixup=HEAD"], false),
        case("reuse", &["-C", "HEAD"], true),
        Case { edits: false, ..case("allow empty", &["-m", "nothing changed", "--allow-empty"], false) },
    ];
    let name: String = format!("options-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in &cases {
        git(&fixture, &repo, &["reset", "--hard", "--quiet", start_commit.as_str()]);
        if entry.edits {
            std::fs::write(repo.join("a.txt"), format!("base\nchanged for {}\n", entry.name)).expect("edit");
        }
        let served_before: usize = finished(&fixture);
        let mut arguments: Vec<&str> = vec!["commit"];
        arguments.extend(entry.arguments.iter().map(String::as_str));
        arguments.extend(["--", "a.txt"]);
        let served: Output = through_service(&fixture, &repo, name.as_str(), arguments.as_slice());
        assert!(served.status.success(), "{}: {}", entry.name, String::from_utf8_lossy(&served.stderr));
        assert_eq!(finished(&fixture), served_before + 1, "{}: the service landed it, not the transaction", entry.name);
        let through: Vec<String> = stored(&fixture, &repo);
        git(&fixture, &repo, &["reset", "--hard", "--quiet", start_commit.as_str()]);
        if entry.edits {
            std::fs::write(repo.join("a.txt"), format!("base\nchanged for {}\n", entry.name)).expect("edit again");
        }
        let mut native_arguments: Vec<&str> = vec!["commit", "--quiet"];
        native_arguments.extend(entry.arguments.iter().map(String::as_str));
        native_arguments.extend(["--", "a.txt"]);
        let native: Output = direct(&fixture).current_dir(&repo).args(native_arguments.as_slice()).output().expect("native");
        assert!(native.status.success(), "{}: {}", entry.name, String::from_utf8_lossy(&native.stderr));
        let natively: Vec<String> = stored(&fixture, &repo);
        for (index, (left, right)) in through.iter().zip(&natively).enumerate() {
            // The author date is the commit time unless a command line set it, and the commit time differs between runs.
            if index == 3 && !entry.date_set {
                continue;
            }
            assert_eq!(left, right, "{}: field {index}", entry.name);
        }
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A repository whose configuration changes what native `git commit` does (here a clean-up mode) is never built by the
///  service,
/// which would ignore the setting;
/// real Git builds it,
/// the service lands it,
/// and the setting holds.
#[test]
fn a_configuration_the_service_would_ignore_has_git_build_the_commit() {
    let fixture: Fixture = fixture("service-options-config");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["config", "commit.cleanup", "verbatim"]);
    let name: String = format!("options-config-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    std::fs::write(repo.join("a.txt"), "base\nchanged\n").expect("edit");
    let before: usize = finished(&fixture);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "-m", "kept   \n\n\n", "--", "a.txt"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(finished(&fixture), before + 1, "Git built it and the service landed it");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%B"]), "kept   \n\n\n\n", "the setting held");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
