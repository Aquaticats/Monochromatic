//! What:
//!  Event paths of a file whose name is not UTF-8,
//!  through the built executable.
//! Why:
//!  `pathBytes` is a promise to the agent or script reading the events:
//!  the readable
//!      `path` keeps its replacement characters,
//!  and the exact name comes back from the
//!      base64 field,
//!  on wrapper standard error and on `check` and `fix` standard output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const event = JSON.parse(stdout); expect(Buffer.from(event.pathBytes, 'base64')).toEqual(name);
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, fixture, git, observe, remove, repository, wrapped};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

/// The Latin-1 spelling of `café.txt`:
///  valid as a Unix name,
///  not valid UTF-8.
const LATIN_NAME: &[u8] = b"caf\xe9.txt";

/// The base64 of `LATIN_NAME`,
///  as the decision brief shows it and `base64` prints it.
const LATIN_NAME_BYTES: &str = "Y2Fm6S50eHQ=";

/// The final-newline finding about `LATIN_NAME` for one trigger,
///  with its sequence number.
fn latin_finding(sequence: u64, trigger: &str, fix: &str) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"{trigger}\",\
         \"policyId\":\"final-newline\",\"severity\":\"warn\",\"code\":\"final-newline/noncanonical-final-newline\",\
         \"message\":\"Non-empty text file must end with exactly one LF byte.\",\
         \"path\":\"caf\u{fffd}.txt\",\"pathBytes\":\"{LATIN_NAME_BYTES}\",\"fix\":\"{fix}\"}}\n"
    );
}

/// `git add`,
///  `git cli-git check` and `git cli-git fix` name a Latin-1 file by its
/// readable text and its exact bytes;
///  a UTF-8 file beside it keeps the old shape,
///  and the
/// fix summary lists the bytes of both,
///  in Git's byte order.
#[test]
fn events_carry_the_exact_bytes_of_a_name_that_is_not_utf8() {
    let fixture: Fixture = fixture("event-path");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let latin: OsString = OsString::from_vec(LATIN_NAME.to_vec());
    std::fs::write(repo.join(&latin), b"no newline").expect("Latin-1 file");
    let added: Observed = observe(
        wrapped(&fixture)
            .current_dir(&repo)
            .arg("add")
            .arg("--")
            .arg(&latin),
        b"",
    );
    assert_eq!(
        (
            added.code,
            String::from_utf8_lossy(&added.stderr).into_owned()
        ),
        (Some(0), latin_finding(0, "pre-forward", "none"))
    );
    git(&fixture, repo.as_path(), &["reset", "--quiet"]);
    std::fs::write(repo.join("b.txt"), b"utf-8 name").expect("UTF-8 file");
    let checked: Observed = observe(
        wrapped(&fixture)
            .current_dir(&repo)
            .args(["cli-git", "check", "--all"]),
        b"",
    );
    // Candidates come in Git's byte order: `b.txt` sorts before 0x63 `c`.
    let expected_check: String = format!(
        "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"direct-check\",\
         \"policyId\":\"final-newline\",\"severity\":\"warn\",\"code\":\"final-newline/noncanonical-final-newline\",\
         \"message\":\"Non-empty text file must end with exactly one LF byte.\",\"path\":\"b.txt\",\"fix\":\"none\"}}\n{}",
        latin_finding(1, "direct-check", "none")
    );
    assert_eq!(
        (
            checked.code,
            String::from_utf8_lossy(&checked.stdout).into_owned()
        ),
        (Some(0), expected_check)
    );
    let fixed: Observed = observe(
        wrapped(&fixture)
            .current_dir(&repo)
            .args(["cli-git", "fix", "--all"]),
        b"",
    );
    // The summary lists paths in the same byte order.
    assert_eq!(
        (
            fixed.code,
            String::from_utf8_lossy(&fixed.stdout).into_owned()
        ),
        (
            Some(0),
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"direct-fix\",\
                 \"passes\":1,\"changedPaths\":[\"b.txt\",\"caf\u{fffd}.txt\"],\
                 \"changedPathBytes\":[\"Yi50eHQ=\",\"{LATIN_NAME_BYTES}\"]}}\n"
            )
        )
    );
    assert_eq!(
        std::fs::read(repo.join(&latin)).expect("corrected"),
        b"no newline\n"
    );
    remove(&fixture);
}
