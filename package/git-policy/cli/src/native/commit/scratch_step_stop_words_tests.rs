//! Controls for the words a stopped scratch `git commit` prints.

use super::*;

/// The words of a stop say what ended,
/// after how long,
/// why this long,
/// what became of the person's work,
/// and what to run.
#[test]
fn the_words_of_a_stop_say_what_ended_what_is_kept_and_what_to_run() {
    let said: String = stop_words(Duration::from_secs(1261), Duration::from_secs(1260), true);
    assert!(said.starts_with("monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository was stopped after 1261 seconds"), "{said}");
    assert!(said.contains("1260 seconds") && said.contains("three times"), "{said}");
    assert!(said.contains("signs the commit"), "a signed commit's words name the signer: {said}");
    assert!(said.contains("nothing was committed") && said.contains("index and files are as they were"), "{said}");
    assert!(said.contains("Run the same `git commit` command again"), "{said}");
    assert!(!stop_words(Duration::from_secs(1261), Duration::from_secs(1260), false).contains("signs the commit"));
}
