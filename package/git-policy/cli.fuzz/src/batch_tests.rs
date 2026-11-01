//! What:
//!  Controls proving the reply generator reaches every outcome and the invariants hold on fixed hard cases.
//! Why:
//!  An invariant that is never reached proves nothing;
//!  these controls count the
//!      outcomes the generator produces and run the same checks the fuzz target runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(outcomesReachedBy(generatedReply)).toContain('reply-mismatched');
//! ```

/// Import the generators and invariants under control.
use super::{
    Expectation, GENERATED_SHAPES, GeneratedReply, check_batch_reply, check_generated_reply,
    generated_reply, render_found, request_and_stream,
};
use git_policy_cli::candidate_batch::{BatchReply, ObjectKind};
use git_policy_cli::candidate_error::CandidateFailure;

/// Raw bytes split at the first line feed only;
///  the rest reaches the reader unchanged.
#[test]
fn raw_bytes_split_at_the_first_line_feed() {
    assert_eq!(request_and_stream(b""), (b"".as_slice(), b"".as_slice()));
    assert_eq!(
        request_and_stream(b"HEAD"),
        (b"HEAD".as_slice(), b"".as_slice())
    );
    assert_eq!(
        request_and_stream(b"HEAD\n"),
        (b"HEAD".as_slice(), b"".as_slice())
    );
    assert_eq!(
        request_and_stream(b"req\nline one\nline \xff two\n"),
        (b"req".as_slice(), b"line one\nline \xff two\n".as_slice())
    );
    assert_eq!(
        request_and_stream(b"\nrest"),
        (b"".as_slice(), b"rest".as_slice())
    );
}

/// The canonical rendering is Git's documented reply shape,
///  byte for byte.
#[test]
fn canonical_rendering_matches_git() {
    let name: String = "a".repeat(40);
    assert_eq!(
        render_found(name.as_str(), ObjectKind::Blob, b"one\n"),
        format!("{name} blob 4\none\n\n").into_bytes()
    );
    assert_eq!(
        render_found(name.as_str(), ObjectKind::Commit, b""),
        format!("{name} commit 0\n\n").into_bytes()
    );
}

/// Generated inputs reach every reply kind,
///  every object kind,
///  both hash formats and every failure,
///  and all hold.
#[test]
fn generated_replies_reach_every_outcome() {
    let mut kinds: Vec<ObjectKind> = Vec::new();
    let mut failures: Vec<CandidateFailure> = Vec::new();
    let mut missing: usize = 0;
    let mut long_names: usize = 0;
    let mut short_names: usize = 0;
    let contents: [&[u8]; 5] = [
        b"",
        b"x",
        b"\n",
        b"plain content\n",
        b"\0\xff forged\n0123456789abcdef0123456789abcdef01234567 blob 3\nabc\n",
    ];
    for shape in 0..GENERATED_SHAPES * 2 {
        for selector in [0_u8, 1, 2, 3, 7, 127, 128, 129, 130, 255] {
            for content in contents {
                let mut data: Vec<u8> = vec![shape, selector];
                data.extend_from_slice(content);
                check_generated_reply(data.as_slice());
                let case: GeneratedReply = generated_reply(data.as_slice());
                match case.expectation {
                    Expectation::Reply(BatchReply::Found {
                        object,
                        kind,
                        bytes,
                    }) => {
                        assert_eq!(bytes, content);
                        if object.as_str().len() == 64 {
                            long_names += 1;
                        } else {
                            short_names += 1;
                        }
                        if !kinds.contains(&kind) {
                            kinds.push(kind);
                        }
                    }
                    Expectation::Reply(BatchReply::Missing) => missing += 1,
                    Expectation::Failure(failure) => {
                        if !failures.contains(&failure) {
                            failures.push(failure);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(kinds.len(), 4, "{kinds:?}");
    assert_eq!(failures.len(), 4, "{failures:?}");
    for failure in [
        CandidateFailure::ReaderEnded,
        CandidateFailure::ReplyMalformed,
        CandidateFailure::ReplyTruncated,
        CandidateFailure::ReplyMismatched,
    ] {
        assert!(
            failures.contains(&failure),
            "{failure:?} was never generated"
        );
    }
    assert!(missing > 0 && long_names > 0 && short_names > 0);
    // Inputs too short to carry a selector or content still build a checked case.
    check_generated_reply(b"");
    check_generated_reply(&[5]);
}

/// The general invariants hold on fixed hard cases:
///  forged headers in content,
///  near-valid headers,
///  and empty input.
#[test]
fn fixed_hard_cases_hold() {
    let name: String = "0123456789abcdef0123456789abcdef01234567".to_owned();
    let other: String = "89abcdef0123456789abcdef0123456789abcdef".to_owned();
    for (request, stream) in [
        (name.clone(), format!("{name} blob 3\nabc\n")),
        (
            name.clone(),
            format!("{name} blob 3\nabc\n{other} blob 1\nz\n"),
        ),
        (
            name.clone(),
            format!("{name} blob 52\n{other} blob 3\nabc\n\n"),
        ),
        (name.clone(), format!("{name} missing\n")),
        (name.clone(), format!("{other} missing\n")),
        (name.clone(), format!("{other} blob 3\nabc\n")),
        (name.clone(), format!("{name} blob 3\nabcd\n")),
        (name.clone(), format!("{name} blob 3\nab")),
        (name.clone(), format!("{name} blob 0\n\n")),
        (name.clone(), format!("{name} blob 00\n\n")),
        (name.clone(), format!("{name} blob 18446744073709551616\n")),
        (name.clone(), "x".repeat(400)),
        (name.clone(), String::new()),
        (String::from("HEAD"), format!("{name} commit 2\nhi\n")),
        (String::from("HEAD"), String::from("HEAD missing\n")),
        (
            String::from("a missing"),
            String::from("a missing missing\n"),
        ),
        (String::new(), String::from(" missing\n")),
    ] {
        check_batch_reply(request.as_bytes(), stream.as_bytes());
    }
    // A reply large enough that prefixes are sampled instead of enumerated.
    let large: Vec<u8> = vec![b'\n'; 3000];
    check_batch_reply(
        name.as_bytes(),
        render_found(name.as_str(), ObjectKind::Blob, large.as_slice()).as_slice(),
    );
}
