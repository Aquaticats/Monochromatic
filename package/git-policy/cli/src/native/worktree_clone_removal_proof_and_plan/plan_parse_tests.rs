//! What:
//!  The controls of the parse:
//!  every malformed form the design lists,
//!  the two order rules,
//!  and the name a record must agree with.
//! Why:
//!  Nothing is read as a partial record,
//!  so each of these must refuse the whole file rather than skip the line it did not understand
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The grammar" and section 8's list of malformed forms).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parsePlan(header, attempts, name).cause).toBe('repeated-key');
//! ```

use crate::worktree_clone_removal_proof_and_plan::fixtures::{
    SAMPLE_CLONE_GUID,
    SAMPLE_OWN_GUID,
    SAMPLE_SANOID_GUID,
    guid,
    replaced,
    sample_bytes,
};
use crate::worktree_clone_removal_proof_and_plan::plan_parse::parse_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::{
    HEADER_LINE_COUNT,
    split_the_header,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{
    PlanRefusal,
    PlanRefusalCause,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;

/// The name the sample is filed under.
fn filed_under() -> String {
    return SAMPLE_CLONE_GUID.to_string();
}

/// Parse one file's bytes as its own name says it is filed.
fn parse(bytes: &[u8]) -> Result<DestructionPlan, PlanRefusal> {
    let (header, attempts) = split_the_header(bytes)?;
    return parse_plan(header, attempts, Some(filed_under().as_str()));
}

/// The refusal one file's bytes earn.
fn refusal_of(bytes: &[u8]) -> PlanRefusal {
    return parse(bytes).expect_err("the planted form is not a record");
}

/// One sample plan whose attempt region a control writes itself,
/// with the declared count made to match it.
fn with_attempts(lines: &[String]) -> Vec<u8> {
    let sample: Vec<u8> = sample_bytes();
    let (header, _) = split_the_header(sample.as_slice()).expect("the sample has a header");
    let declared: Vec<u8> = replaced(
        header,
        "attempts=2",
        format!("attempts={}", lines.len()).as_str(),
    );
    let mut bytes: Vec<u8> = declared;
    for line in lines {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    return bytes;
}

/// One attempt line of the sample's shape.
fn attempt_line(
    attempt: &str,
    outcome: &str,
    guid_text: &str,
    createtxg: &str,
    errno: &str,
    name: &str,
) -> String {
    return format!(
        "tuple=snapshot {attempt} {outcome} {guid_text} {createtxg} {errno} name={name}"
    );
}

#[test]
fn the_sample_reads_back_with_its_own_values() {
    let plan: DestructionPlan = parse(sample_bytes().as_slice()).expect("the sample is a record");
    assert_eq!(plan.head, HeadWord::Present);
    assert_eq!(plan.facts.clone_guid.value(), SAMPLE_CLONE_GUID);
    assert_eq!(plan.attempts.len(), 2);
    assert_eq!(plan.named_snapshots().len(), 2);
}

#[test]
fn a_missing_extra_repeated_or_out_of_order_key_refuses() {
    let cases: [(&str, &str, PlanRefusalCause); 4] = [
        (
            "\nhead=present\n",
            "\nhead=present\nhead=present\n",
            PlanRefusalCause::RepeatedKey,
        ),
        (
            "\nhead=present\n",
            "\nhead=absent\n",
            PlanRefusalCause::UnknownHead,
        ),
        (
            "\ncloneGuid=",
            "\ncloneGuidX=",
            PlanRefusalCause::UnknownKey,
        ),
        (
            "head=present\nattempts=2\n",
            "attempts=2\nhead=present\n",
            PlanRefusalCause::OutOfOrderKey,
        ),
    ];
    for (from, to, cause) in cases {
        let refusal: PlanRefusal =
            refusal_of(replaced(sample_bytes().as_slice(), from, to).as_slice());
        assert_eq!(
            refusal.cause, cause,
            "planting `{from}` as `{to}` refuses with {cause:?}, not {refusal:?}"
        );
    }
}

#[test]
fn a_header_of_another_record_or_another_version_refuses() {
    let other: Vec<u8> = replaced(
        sample_bytes().as_slice(),
        "zfs worktree clone destruction plan",
        "zfs worktree clone binding record",
    );
    assert_eq!(
        refusal_of(other.as_slice()).cause,
        PlanRefusalCause::NotThisRecord
    );
    let later: Vec<u8> = replaced(
        sample_bytes().as_slice(),
        "plan, version 1",
        "plan, version 2",
    );
    let refusal: PlanRefusal = refusal_of(later.as_slice());
    assert_eq!(refusal.cause, PlanRefusalCause::UnknownVersion);
    assert_eq!(refusal.found, "2");
}

#[test]
fn a_declared_count_above_or_below_the_lines_refuses() {
    assert_eq!(
        refusal_of(replaced(sample_bytes().as_slice(), "attempts=2", "attempts=3").as_slice())
            .cause,
        PlanRefusalCause::CountDisagrees
    );
    assert_eq!(
        refusal_of(replaced(sample_bytes().as_slice(), "attempts=2", "attempts=1").as_slice())
            .cause,
        PlanRefusalCause::CountDisagrees
    );
    assert_eq!(
        refusal_of(replaced(sample_bytes().as_slice(), "attempts=2", "attempts=02").as_slice())
            .cause,
        PlanRefusalCause::NotCanonical
    );
    assert_eq!(
        refusal_of(
            replaced(sample_bytes().as_slice(), "attempts=2", "attempts=12345678").as_slice()
        )
        .cause,
        PlanRefusalCause::NotCanonical
    );
}

#[test]
fn a_line_of_another_kind_or_an_unknown_outcome_refuses() {
    let other_kind: Vec<u8> = replaced(
        sample_bytes().as_slice(),
        "tuple=snapshot 0 planned",
        "tuple=bookmark 0 planned",
    );
    assert_eq!(
        refusal_of(other_kind.as_slice()).cause,
        PlanRefusalCause::WrongAttemptKind
    );
    let unknown: Vec<u8> = replaced(sample_bytes().as_slice(), "0 planned", "0 maybe");
    assert_eq!(
        refusal_of(unknown.as_slice()).cause,
        PlanRefusalCause::UnknownOutcome
    );
}

#[test]
fn two_lines_of_one_name_and_one_number_refuse() {
    let name: String = String::from("tank/wt/fix-flaky-test@own");
    let line: String = attempt_line(
        "0",
        "planned",
        &guid(SAMPLE_OWN_GUID).file_name(),
        "18",
        "-",
        name.as_str(),
    );
    let bytes: Vec<u8> = with_attempts(&[line.clone(), line]);
    assert_eq!(
        refusal_of(bytes.as_slice()).cause,
        PlanRefusalCause::DuplicateAttempt
    );
}

#[test]
fn one_names_lines_out_of_number_order_refuse() {
    let name: String = String::from("tank/wt/fix-flaky-test@own");
    let guid_text: String = guid(SAMPLE_OWN_GUID).file_name();
    let bytes: Vec<u8> = with_attempts(&[
        attempt_line("2", "intact", guid_text.as_str(), "18", "-", name.as_str()),
        attempt_line(
            "1",
            "refused",
            guid_text.as_str(),
            "18",
            "16",
            name.as_str(),
        ),
    ]);
    assert_eq!(
        refusal_of(bytes.as_slice()).cause,
        PlanRefusalCause::AttemptsOutOfOrder
    );
}

#[test]
fn a_non_canonical_number_or_guid_refuses() {
    for guid_text in [
        "+7828944690444299989",
        "007",
        "1A2B",
        "100000000000000000000",
    ] {
        let refusal: PlanRefusal = refusal_of(
            replaced(
                sample_bytes().as_slice(),
                "cloneGuid=7828944690444299989",
                format!("cloneGuid={guid_text}").as_str(),
            )
            .as_slice(),
        );
        assert_eq!(
            refusal.cause,
            PlanRefusalCause::NotCanonical,
            "`{guid_text}` is no GUID"
        );
    }
    let refusal: PlanRefusal = refusal_of(
        replaced(
            sample_bytes().as_slice(),
            "cloneCreatetxg=17",
            "cloneCreatetxg=017",
        )
        .as_slice(),
    );
    assert_eq!(refusal.cause, PlanRefusalCause::NotCanonical);
    let attempt: PlanRefusal = refusal_of(
        replaced(
            sample_bytes().as_slice(),
            "tuple=snapshot 0 planned",
            "tuple=snapshot 00 planned",
        )
        .as_slice(),
    );
    assert_eq!(attempt.cause, PlanRefusalCause::NotCanonical);
}

#[test]
fn an_errno_on_a_line_that_is_not_a_refusal_and_a_missing_one_on_a_refusal_both_refuse() {
    let planned_with_errno: Vec<u8> = replaced(
        sample_bytes().as_slice(),
        "planned 9332110456720018274 20 -",
        "planned 9332110456720018274 20 16",
    );
    assert_eq!(
        refusal_of(planned_with_errno.as_slice()).cause,
        PlanRefusalCause::ErrnoOnAnotherLine
    );
    let name: String = String::from("tank/wt/fix-flaky-test@own");
    let refused_without: Vec<u8> = with_attempts(&[attempt_line(
        "1",
        "refused",
        guid(SAMPLE_OWN_GUID).file_name().as_str(),
        "18",
        "-",
        name.as_str(),
    )]);
    assert_eq!(
        refusal_of(refused_without.as_slice()).cause,
        PlanRefusalCause::MissingErrno
    );
    let refused_with: Vec<u8> = with_attempts(&[attempt_line(
        "1",
        "refused",
        guid(SAMPLE_OWN_GUID).file_name().as_str(),
        "18",
        "16",
        name.as_str(),
    )]);
    let plan: DestructionPlan =
        parse(refused_with.as_slice()).expect("a refusal carries its errno");
    assert_eq!(plan.attempts[0].errno, Some(16));
}

#[test]
fn a_line_whose_name_is_missing_or_of_another_dataset_refuses() {
    let no_name: Vec<u8> = replaced(
        sample_bytes().as_slice(),
        "name=tank/wt/fix-flaky-test@sanoid-hourly",
        "named=tank/wt/fix-flaky-test@sanoid-hourly",
    );
    assert_eq!(
        refusal_of(no_name.as_slice()).cause,
        PlanRefusalCause::NotCanonical
    );
    let foreign: Vec<u8> = replaced(
        sample_bytes().as_slice(),
        "name=tank/wt/fix-flaky-test@sanoid-hourly",
        "name=tank/other@sanoid-hourly",
    );
    assert_eq!(
        refusal_of(foreign.as_slice()).cause,
        PlanRefusalCause::ForeignName
    );
}

#[test]
fn a_file_whose_name_disagrees_with_its_own_clone_guid_refuses() {
    let bytes: Vec<u8> = sample_bytes();
    let (header, attempts) = split_the_header(bytes.as_slice()).expect("the sample has a header");
    let other: String = guid(SAMPLE_SANOID_GUID).file_name();
    let refusal: PlanRefusal = parse_plan(header, attempts, Some(other.as_str()))
        .expect_err("a record is filed under its own GUID");
    assert_eq!(refusal.cause, PlanRefusalCause::NameDisagreesWithTheRecord);
    let foreign: PlanRefusal =
        parse_plan(header, attempts, Some("plan")).expect_err("a plan is named by a GUID");
    assert_eq!(foreign.cause, PlanRefusalCause::ForeignName);
}

#[test]
fn a_record_truncated_at_any_header_line_refuses_and_changes_nothing() {
    let bytes: Vec<u8> = sample_bytes();
    let mut boundaries: Vec<usize> = Vec::new();
    for (place, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            boundaries.push(place + 1);
        }
    }
    assert!(
        boundaries.len() > HEADER_LINE_COUNT,
        "the sample holds its header and its attempt lines"
    );
    for at in boundaries.iter().take(HEADER_LINE_COUNT) {
        let truncated: &[u8] = &bytes[..*at];
        let refusal: PlanRefusal = refusal_of(truncated);
        assert!(
            matches!(
                refusal.cause,
                PlanRefusalCause::EndsEarly
                    | PlanRefusalCause::MissingAttemptLines
                    | PlanRefusalCause::CountDisagrees
            ),
            "a record cut at {at} refuses, and refused with {refusal:?}"
        );
    }
}

#[test]
fn a_record_that_stops_inside_its_last_attempt_line_refuses() {
    let bytes: Vec<u8> = sample_bytes();
    let cut: &[u8] = &bytes[..bytes.len() - 1];
    let refusal: PlanRefusal = refusal_of(cut);
    assert_eq!(refusal.cause, PlanRefusalCause::MissingAttemptLines);
}
