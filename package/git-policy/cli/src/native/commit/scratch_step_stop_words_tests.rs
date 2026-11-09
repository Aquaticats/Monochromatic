//! Controls for the words a stopped scratch `git commit` prints.

use super::*;

/// The words of a stop say what ended,
/// after how long,
/// why this long,
/// what became of the person's work,
/// and what to run.
#[test]
fn the_words_of_a_stop_say_what_ended_what_is_kept_and_what_to_run() {
    let said: String = stop_words(Duration::from_secs(1261), Duration::from_secs(1260), true, false);
    assert!(said.starts_with("monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository was stopped after 1261 seconds"), "{said}");
    assert!(said.contains("1260 seconds") && said.contains("three times"), "{said}");
    assert!(said.contains("signs the commit"), "a signed commit's words name the signer: {said}");
    assert!(said.contains("nothing was committed") && said.contains("index and files are as they were"), "{said}");
    assert!(said.contains("Run the same `git commit` command again"), "{said}");
    assert!(!stop_words(Duration::from_secs(1261), Duration::from_secs(1260), false, false).contains("signs the commit"));
}

/// A stop whose programs below could not be listed says one of them may still run,
/// and a stop that listed them says nothing about it.
#[test]
fn the_words_say_when_the_programs_below_could_not_be_listed() {
    let listed: String = stop_words(Duration::from_secs(1261), Duration::from_secs(1260), false, false);
    let unlisted: String = stop_words(Duration::from_secs(1261), Duration::from_secs(1260), false, true);
    assert!(!listed.contains("could not be listed"), "{listed}");
    assert!(unlisted.contains("could not be listed") && unlisted.contains("check with `ps`"), "{unlisted}");
}
