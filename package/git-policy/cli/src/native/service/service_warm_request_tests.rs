//! Controls for the look at a running service's start arguments.

use super::*;

/// The words of a start command line.
fn words(text: &[&str]) -> Vec<Vec<u8>> {
    return text.iter().map(|word| { return word.as_bytes().to_vec(); }).collect();
}

/// A service is the one of an instance only by `service run` and `--instance <that instance>`.
#[test]
fn a_service_is_told_by_its_start_arguments_and_its_instance() {
    let own: Vec<Vec<u8>> = words(&["/bin/monochromatic-git-policy-cli", "monochromatic-git-policy-cli", "service", "run", "--journal", "/j", "--instance", "abc"]);
    let borrowed: Vec<&[u8]> = own.iter().map(Vec::as_slice).collect();
    assert!(is_service_of(borrowed.as_slice(), "abc"));
    assert!(!is_service_of(borrowed.as_slice(), "abd"));
    let status: Vec<Vec<u8>> = words(&["monochromatic-git-policy-cli", "service", "status", "--instance", "abc"]);
    let status_words: Vec<&[u8]> = status.iter().map(Vec::as_slice).collect();
    assert!(!is_service_of(status_words.as_slice(), "abc"), "a status command is not the service");
    let other: Vec<Vec<u8>> = words(&["git", "commit", "--instance", "abc"]);
    let other_words: Vec<&[u8]> = other.iter().map(Vec::as_slice).collect();
    assert!(!is_service_of(other_words.as_slice(), "abc"));
}

/// A process table with no service of the instance says no,
/// and one that cannot be read says no.
#[test]
fn an_empty_or_missing_process_table_says_no() {
    let root: PathBuf = crate::test_support::fixture("warm-proc-table");
    assert!(!service_process_runs("abc", root.as_path()));
    assert!(!service_process_runs("abc", root.join("nothing").as_path()));
    let own: PathBuf = root.join("4242");
    std::fs::create_dir_all(&own).expect("process directory");
    std::fs::write(own.join("cmdline"), b"x\0service\0run\0--instance\0abc\0").expect("cmdline");
    assert!(service_process_runs("abc", root.as_path()));
    crate::test_support::remove(root.as_path());
}
