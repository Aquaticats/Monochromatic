//! What:
//!  Controls of one status line:
//!  the round trip through the pipe's text,
//!  every answer's shape,
//!  each refusal,
//!  and the feed that keeps whole lines from arbitrary reads.
//! Why:
//!  The mode writes the record from these lines,
//!  so a line that parses into another answer would misname an arm,
//!  and a half line read as a line would fail a child that answered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parse(encode({ step: 'teardown', answer: 'passed' })).step).toBe('teardown');
//! ```

use super::super::probe_arms::ProbeStep;
use super::super::probe_line_refusal::LineRefusal;
use super::{
    LineFeed,
    StepLine,
};

/// A passing line names its arm and step and carries no source.
#[test]
fn a_passing_line_round_trips() {
    let line: StepLine = StepLine::passed(ProbeStep::Teardown);
    let text: String = line.encode();
    assert!(text.starts_with("probe-step arm=teardown step=teardown answer=passed errno=none"));
    assert_eq!(StepLine::parse(text.as_str()), Ok(line));
}

/// A failed line keeps the errno's name,
/// and a comparison's failure says none.
#[test]
fn a_failed_line_keeps_its_errno() {
    let with_errno: StepLine = StepLine::failed(ProbeStep::Move, Some("EBUSY"));
    assert_eq!(
        StepLine::parse(with_errno.encode().as_str()),
        Ok(with_errno)
    );
    let comparison: StepLine = StepLine::failed(ProbeStep::WalkSlot, None);
    assert!(comparison.encode().contains("errno=none"));
    assert_eq!(
        StepLine::parse(comparison.encode().as_str()),
        Ok(comparison)
    );
}

/// A number this build does not name stays the kernel's number,
/// so a failed call never reads as a comparison.
#[test]
fn an_unnamed_errno_keeps_its_number() {
    let line: StepLine = StepLine::failed_number(ProbeStep::AttachSource, Some(4242));
    assert!(line.encode().contains("errno=4242"));
    assert_eq!(StepLine::parse(line.encode().as_str()), Ok(line));
}

/// A step that did not run carries the word saying why.
#[test]
fn a_not_exercised_line_carries_its_reason() {
    let line: StepLine = StepLine::not_exercised(
        ProbeStep::AttachExclusiveCreate,
        "source-held",
        Some("EBUSY"),
    );
    assert!(line.encode().contains("reason=source-held"));
    assert_eq!(StepLine::parse(line.encode().as_str()), Ok(line));
}

/// The source the read-source-name step read travels on its line.
#[test]
fn the_source_travels_on_its_line() {
    let mut line: StepLine = StepLine::passed(ProbeStep::ReadSourceName);
    line.source = Some(String::from("zpcachyos/nosnap/git-probe@ready"));
    assert_eq!(StepLine::parse(line.encode().as_str()), Ok(line));
}

/// A line of another kind is refused,
/// never read as a step.
#[test]
fn a_foreign_line_is_refused() {
    match StepLine::parse("hello there") {
        Err(LineRefusal::NotAStatusLine {
            ..
        }) => {}
        other => panic!("{other:?}"),
    }
}

/// An unknown step or answer is refused.
#[test]
fn unknown_words_are_refused() {
    let line: String = String::from("probe-step arm=nothing step=nothing answer=passed errno=none");
    assert!(matches!(
        StepLine::parse(line.as_str()),
        Err(LineRefusal::UnknownStep { .. })
    ));
    let answer: String =
        String::from("probe-step arm=teardown step=teardown answer=maybe errno=none");
    assert!(matches!(
        StepLine::parse(answer.as_str()),
        Err(LineRefusal::UnknownAnswer { .. })
    ));
    let errno: String =
        String::from("probe-step arm=move-unmount step=move answer=failed errno=ENOPE");
    assert!(matches!(
        StepLine::parse(errno.as_str()),
        Err(LineRefusal::UnknownErrno { .. })
    ));
}

/// A field given twice,
/// a field missing,
/// and an arm that disagrees with its step are each refused.
#[test]
fn repeated_missing_and_mismatched_fields_are_refused() {
    let repeated: String =
        String::from("probe-step arm=teardown arm=teardown step=teardown answer=passed errno=none");
    assert!(matches!(
        StepLine::parse(repeated.as_str()),
        Err(LineRefusal::RepeatedKey { .. })
    ));
    let missing: String = String::from("probe-step arm=teardown answer=passed errno=none");
    assert!(matches!(
        StepLine::parse(missing.as_str()),
        Err(LineRefusal::MissingKey { .. })
    ));
    let wrong_arm: String =
        String::from("probe-step arm=move step=teardown answer=passed errno=none");
    assert!(matches!(
        StepLine::parse(wrong_arm.as_str()),
        Err(LineRefusal::WrongArm { .. })
    ));
    let no_reason: String = String::from(
        "probe-step arm=attach step=attach-exclusive-create answer=not-exercised errno=none",
    );
    assert!(matches!(
        StepLine::parse(no_reason.as_str()),
        Err(LineRefusal::MissingKey { .. })
    ));
}

/// The feed reports only whole lines,
/// keeping the tail a later read completes.
#[test]
fn the_feed_keeps_half_a_line() {
    let mut feed: LineFeed = LineFeed::new();
    let whole: String = StepLine::passed(ProbeStep::Teardown).encode();
    let first: Vec<Result<StepLine, LineRefusal>> = feed.push(&whole.as_bytes()[..10]);
    assert!(first.is_empty());
    let rest: Vec<Result<StepLine, LineRefusal>> = feed.push(&whole.as_bytes()[10..]);
    assert_eq!(rest.len(), 1);
    assert_eq!(
        rest.into_iter().next(),
        Some(Ok(StepLine::passed(ProbeStep::Teardown)))
    );
}
