//! The unit tests of the line a refused start prints:
//! what the system's own answer for a program that could not be opened says,
//! and what the sentence about a shadow says.
//! The test program itself links this crate,
//! so it carries the identity and is the shadow-shaped fixture.

/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the two shapes a refusal comes in,
/// and the check that answers with one.
use crate::refused_start::describe_refused_start;
/// Imports the check of the chosen program.
use crate::shadow_identity::{
    VetRefusal,
    vet,
};
/// Imports the scratch directories.
use crate::test_scratch::scratch;

/// The running test program,
/// which carries the identity because it links this crate.
fn test_program() -> PathBuf {
    return std::env::current_exe().expect("the test program's own path");
}

#[test]
fn a_refused_start_that_is_a_shadow_says_which_shadow_it_found() {
    let shadow: PathBuf = test_program();
    let refusal = vet(&shadow).err().expect("the test program is a shadow");
    // What:     `to_string()` is the refusal's one line of text, which is the whole explanation for a shadow.
    // Why:      The line under test is built from it, so the test reads the same words the start prints.
    let cause: String = refusal.to_string();

    let line: String = describe_refused_start(&shadow, &cause);

    // What:     The line names the program once, the shadow the file is, and that it was not started.
    // Why:      A person reading it must be able to tell a shadow from a program fault without the source.
    assert!(
        line.starts_with("search-shadow: could not start "),
        "{line}"
    );
    assert!(line.contains(&shadow.display().to_string()), "{line}");
    assert!(line.contains("is or leads to"), "{line}");
    assert!(line.contains("a shadow"), "{line}");
    assert!(line.contains("so it was not started"), "{line}");
}

#[test]
fn a_refused_start_that_could_not_be_opened_keeps_the_systems_own_answer_and_its_kind() {
    let folder: tempfile::TempDir = scratch();
    let missing: PathBuf = folder.path().join("no-such-rg");
    let refusal = vet(&missing).err().expect("a missing program is refused");

    // What:     `VetRefusal::NotOpened { error }` is the arm for a program the system would not open.
    // Why:      The test reads the system's error itself rather than a text that lost it.
    let error: std::io::Error = match refusal {
        VetRefusal::NotOpened {
            error,
        } => error,
        VetRefusal::LeadsToShadow {
            sentence,
        } => panic!("a missing program is not a shadow: {sentence}"),
    };
    // What:     `error.kind()` is the system's own classification, and `NotFound` is the one a program that is gone
    //           answers.
    // Why:      A caller chooses a message or a retry by it; a text with no kind cannot be told from any other refusal.
    assert_eq!(std::io::ErrorKind::NotFound, error.kind());
    // What:     The line names the program and the system's words, so the reader knows which file and what happened.
    // Why:      This is the commonest refusal: an install or an upgrade moved the program aside, and the next start
    //           finds nothing.
    let line: String = describe_refused_start(&missing, &error.to_string());
    assert_eq!(
        line,
        format!(
            "search-shadow: could not start {}: No such file or directory (os error 2)",
            missing.display()
        ),
        "{line}"
    );
}

#[test]
fn a_program_the_check_accepts_is_started_and_prints_no_line() {
    let real: &Path = Path::new("/usr/bin/true");

    // What:     `is_ok()` is the check's "nothing refused".
    // Why:      The line is only for a refusal; a program the check accepts is started and prints nothing of this.
    assert!(vet(real).is_ok(), "a real program was refused");
}
