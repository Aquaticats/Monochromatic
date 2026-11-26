//! What:
//!  Controls of the mode's verdict line and its source readings:
//!  each verdict is one sentence naming what a person must look at,
//!  a refused slot is told its route,
//!  and a timed-out record names both steps.
//! Why:
//!  The line is the journal entry a boot failure is read from,
//!  and it must name the arm or the step rather than only that something
//!  failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(verdictLine({ kind: 'failed', step: 'move' })).toContain('move');
//! ```

use super::{
    refusal_sentence,
    refusal_word,
    source_of,
    source_refusal_word,
    verdict_line,
};

/// An empty report,
/// which every verdict line reads without a step.
fn report() -> ProbeReport {
    return ProbeReport::new();
}
use super::super::probe_arms::{
    ProbeArm,
    ProbeStep,
};
use super::super::probe_report::{
    ProbeReport,
    ProbeVerdict,
};
use super::super::probe_status_line::StepLine;

/// What:
///  The sentence the mode prints for a refusal the worker reported,
///  and the words it is asked about.
/// Why:
///  The project's second answer of 2026-11-22 prints the rows for none and for
///  several,
///  and the fourth gives the empty file its sentence;
///  both bind the probe,
///  whose record carries only a closed word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(refusalSentence('several-mounted-checkouts', file, twoInstances)).toContain('/home/user/second');
/// ```
#[test]
fn the_refusal_words_ask_for_sentences_that_name_the_rows_and_the_key() {
    let mut several: ProbeReport = ProbeReport::new();
    several.record(StepLine::not_exercised(
        ProbeStep::CheckoutRoot,
        "several-mounted-checkouts",
        None,
    ));
    assert_eq!(refusal_word(&several), Some("several-mounted-checkouts"));
    let mut empty: ProbeReport = ProbeReport::new();
    empty.record(StepLine::not_exercised(
        ProbeStep::ReadSourceName,
        "configuration-empty",
        None,
    ));
    assert_eq!(refusal_word(&empty), Some("configuration-empty"));
    let mut other: ProbeReport = ProbeReport::new();
    other.record(StepLine::not_exercised(
        ProbeStep::CheckoutRoot,
        "no-checkout-path",
        None,
    ));
    assert_eq!(refusal_word(&other), None);
    assert_eq!(refusal_word(&ProbeReport::new()), None);

    let fixture: std::path::PathBuf = crate::test_support::fixture("probe-refusal-sentence");
    let missing: std::path::PathBuf = fixture.join("nothing.jsonc");
    let rows: Vec<String> = vec![
        crate::testing::mounted_dataset_table::row(
            40,
            1,
            "/",
            "/home/user/Monochromatic",
            "zfs",
            "pool/home/monochromatic-git-checkout",
        ),
        crate::testing::mounted_dataset_table::row(
            41,
            1,
            "/",
            "/home/user/second",
            "zfs",
            "other/home/monochromatic-git-checkout",
        ),
    ];
    let table = crate::testing::mounted_dataset_table::ScriptedMountTable::holding(&rows);
    let several_sentence: String =
        refusal_sentence("several-mounted-checkouts", missing.as_path(), &table)
            .expect("two instances are no single checkout");
    assert!(
        several_sentence.contains("/home/user/Monochromatic")
            && several_sentence.contains("/home/user/second"),
        "the sentence names both rows with their paths: {several_sentence}"
    );
    assert!(
        several_sentence.contains("will not choose between them"),
        "the sentence says why nothing was taken: {several_sentence}"
    );
    assert!(
        !several_sentence.contains("cloneMounts"),
        "the probe's remedy names only its own key, because the sweep's keys do not settle it: {several_sentence}"
    );
    assert!(
        several_sentence.contains("startProbe.checkout"),
        "the probe's own key is what it names: {several_sentence}"
    );

    // The empty file's sentence is the one the project's fourth answer asks for, and it is the probe's own words.
    let empty_file: std::path::PathBuf = fixture.join("empty.jsonc");
    std::fs::write(empty_file.as_path(), "").expect("the empty fixture file is written");
    let empty_sentence: String =
        refusal_sentence("configuration-empty", empty_file.as_path(), &table)
            .expect("the empty file has a sentence");
    assert!(
        empty_sentence.contains("an empty file is not the same as no file"),
        "{empty_sentence}"
    );
    assert!(
        !empty_sentence.contains("cloneMounts"),
        "the probe's empty-file sentence names its own key only: {empty_sentence}"
    );
    assert_eq!(
        refusal_sentence("several-mounted-checkouts", empty_file.as_path(), &table),
        None,
        "a word the file does not produce prints nothing"
    );

    // Every file refusal the probe can meet has a sentence, and none of them borrows the sweep's remedy.
    let single: Vec<String> = vec![crate::testing::mounted_dataset_table::row(
        40,
        1,
        "/",
        "/home/user/Monochromatic",
        "zfs",
        "pool/home/monochromatic-git-checkout",
    )];
    let one = crate::testing::mounted_dataset_table::ScriptedMountTable::holding(&single);
    let no_rows: Vec<String> = vec![crate::testing::mounted_dataset_table::row(
        1,
        0,
        "/",
        "/",
        "zfs",
        "pool/ROOT/cos/root",
    )];
    let empty_table = crate::testing::mounted_dataset_table::ScriptedMountTable::holding(&no_rows);
    let shapes: [(
        &str,
        &str,
        &crate::testing::mounted_dataset_table::ScriptedMountTable,
    ); 5] = [
        ("configuration-unparsable", "{\"startProbe\": ", &one),
        ("name-not-text", "{\"startProbe\":{\"dataset\": 7}}", &one),
        (
            "name-refused",
            "{\"startProbe\":{\"dataset\": \"pool/ds@snap\"}}",
            &one,
        ),
        (
            "no-mounted-checkout",
            "{\"startProbe\":{\"checkout\": \"/home/nowhere\"}}",
            &empty_table,
        ),
        ("several-mounted-checkouts", "{}", &table),
    ];
    for (word, document, mounts) in shapes {
        let file: std::path::PathBuf = fixture.join(format!("{word}.jsonc"));
        std::fs::write(file.as_path(), document).expect("the fixture file is written");
        let sentence: String = refusal_sentence(word, file.as_path(), mounts)
            .unwrap_or_else(|| panic!("{word} has a sentence of its own"));
        assert!(
            !sentence.contains("cloneMounts"),
            "the {word} sentence names the probe's own key only: {sentence}"
        );
        assert!(
            sentence.contains("the probe's "),
            "the {word} sentence says what could not be told: {sentence}"
        );
        // The name rule's sentence names the name as written, which is what the person has to change; every other one
        // names the file.
        assert!(
            sentence.contains(file.display().to_string().as_str())
                || sentence.contains("`pool/ds@snap`"),
            "the {word} sentence names the file a person opens, or the name to change: {sentence}"
        );
    }
}

/// Every verdict's line names what the reader must act on.
#[test]
fn every_verdict_line_names_what_matters() {
    let mut failed: ProbeReport = ProbeReport::new();
    failed.record(StepLine::failed(ProbeStep::Move, Some("EBUSY")));
    assert!(verdict_line(ProbeVerdict::Passed, &report()).contains("enabled"));
    let failure: String = verdict_line(
        ProbeVerdict::Failed {
            step: ProbeStep::Move,
        },
        &failed,
    );
    assert!(
        failure.contains("move") && failure.contains("EBUSY") && failure.contains("move-unmount"),
        "{failure}"
    );
    assert!(
        verdict_line(
            ProbeVerdict::Blocked {
                arm: ProbeArm::Source
            },
            &report()
        )
        .contains("source")
    );
    assert!(
        verdict_line(
            ProbeVerdict::Blocked {
                arm: ProbeArm::Source
            },
            &report()
        )
        .contains("reflink")
    );
    let timed_out: String = verdict_line(
        ProbeVerdict::TimedOut {
            in_flight: Some(ProbeStep::Unmount),
            last_finished: Some(ProbeStep::Move),
        },
        &report(),
    );
    assert!(
        timed_out.contains("unmount") && timed_out.contains("move"),
        "{timed_out}"
    );
    assert!(
        verdict_line(
            ProbeVerdict::Skipped {
                gate_step: "member"
            },
            &report()
        )
        .contains("member")
    );
}

/// The source and its refusal are read from the lines the worker sent.
#[test]
fn the_source_and_its_refusal_come_from_the_lines() {
    let mut report: ProbeReport = ProbeReport::new();
    assert_eq!(source_of(&report), None);
    assert_eq!(source_refusal_word(&report), None);
    let mut line: StepLine = StepLine::passed(ProbeStep::ReadSourceName);
    line.source = Some(String::from("pool/ds@ready"));
    report.record(line);
    assert_eq!(source_of(&report), Some(String::from("pool/ds@ready")));
    let mut missing: ProbeReport = ProbeReport::new();
    missing.record(StepLine::not_exercised(
        ProbeStep::ReadSourceName,
        "name-absent",
        None,
    ));
    assert_eq!(source_refusal_word(&missing), Some("name-absent"));
}
