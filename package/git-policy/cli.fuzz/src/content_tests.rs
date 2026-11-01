//! What:
//!  Controls proving the content generators reach every outcome and the invariants
//!       hold on fixed hard cases.
//! Why:
//!  An invariant that is never reached proves nothing;
//!  these controls count what the
//!      generators produce and run the same checks the fuzz targets run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(refusalsReachedBy(generatedRulesFile)).toEqual(allRefusals);
//! ```

/// Import the generators and invariants under control.
use super::{
    check_final_newline, check_generated_listings, check_rules_file_value, check_stage_records,
    generated_listings, generated_rules_file, generated_text, render_stage_records,
};
use git_policy_cli::candidate_error::CandidateFailure;
use git_policy_cli::candidate_stage::{StageRecord, parse_stage_records, staged_delta};
use git_policy_cli::config_rules_file::{RulesFileRefusal, check_rules_file};
use git_policy_cli::policy_final_newline::normalized_final_newline;

/// Every input of two bytes,
///  then a sample of longer ones,
///  as fuzz inputs.
fn inputs() -> Vec<Vec<u8>> {
    let mut all: Vec<Vec<u8>> = Vec::new();
    for first in 0..=255_u8 {
        for second in [0_u8, 1, 2, 3, 5, 8, 13, 64, 127, 128, 200, 255] {
            all.push(vec![first, second]);
            all.push(vec![
                first,
                second,
                first ^ second,
                second,
                7,
                first,
                3,
                9,
                1,
                0,
                255,
            ]);
            let mut long: Vec<u8> = Vec::new();
            for index in 0..40_u8 {
                long.push(
                    first
                        .wrapping_mul(31)
                        .wrapping_add(index.wrapping_mul(second)),
                );
            }
            all.push(long);
        }
    }
    return all;
}

/// Generated listings reach equal,
///  changed,
///  removed and new paths,
///  conflict stages and
/// every mode,
///  and the delta invariant holds on all of them.
#[test]
fn generated_listings_reach_every_delta_kind() {
    let mut changed: usize = 0;
    let mut removed: usize = 0;
    let mut added: usize = 0;
    let mut unchanged: usize = 0;
    let mut conflicts: usize = 0;
    for data in inputs() {
        check_generated_listings(data.as_slice());
        let (before, after): (Vec<StageRecord>, Vec<StageRecord>) =
            generated_listings(data.as_slice());
        for record in before.iter().chain(after.iter()) {
            if record.stage != 0 {
                conflicts += 1;
            }
        }
        let delta_paths: Vec<Vec<u8>> = staged_delta(before.as_slice(), after.as_slice())
            .into_iter()
            .map(changed_path)
            .collect();
        for record in &before {
            let present_after: bool = holds_path(after.as_slice(), record.path.as_slice());
            let listed: bool = delta_paths.contains(&record.path);
            match (present_after, listed) {
                (false, true) => removed += 1,
                (true, true) => changed += 1,
                (true, false) => unchanged += 1,
                (false, false) => panic!("a removed path was not in the delta"),
            }
        }
        for record in &after {
            if !holds_path(before.as_slice(), record.path.as_slice()) {
                added += 1;
            }
        }
    }
    assert!(
        changed > 0 && removed > 0 && added > 0 && unchanged > 0 && conflicts > 0,
        "changed {changed}, removed {removed}, added {added}, unchanged {unchanged}, conflicts {conflicts}"
    );
}

/// Whether any record of a listing names `path`.
fn holds_path(records: &[StageRecord], path: &[u8]) -> bool {
    for record in records {
        if record.path == path {
            return true;
        }
    }
    return false;
}

/// The pathname of a changed path.
fn changed_path(path: git_policy_cli::candidate_stage::ChangedPath) -> Vec<u8> {
    return path.path;
}

/// Raw listings:
///  accepted renderings,
///  and each malformed shape refused with its cause.
#[test]
fn raw_listings_hold_on_fixed_hard_cases() {
    let name: &str = "0123456789abcdef0123456789abcdef01234567";
    let valid: Vec<u8> = format!("100644 {name} 0\ta.txt\0160000 {name} 2\tt\tab\0").into_bytes();
    check_stage_records(valid.as_slice());
    let records: Vec<StageRecord> = parse_stage_records(valid.as_slice()).expect("valid");
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].path, b"t\tab");
    assert_eq!(render_stage_records(records.as_slice()), valid);
    check_stage_records(b"");
    for (bytes, failure) in [
        (
            format!("100644 {name} 0\ta.txt"),
            CandidateFailure::ListingMalformed,
        ),
        (
            format!("100644 {name} 0 a.txt\0"),
            CandidateFailure::ListingMalformed,
        ),
        (
            format!("100644 {name} 0\t\0"),
            CandidateFailure::ListingMalformed,
        ),
        (
            format!("040000 {name} 0\ta\0"),
            CandidateFailure::UnsupportedMode,
        ),
        (
            format!("100644 {} 0\ta\0", &name[..39]),
            CandidateFailure::ListingMalformed,
        ),
        (
            format!("100644 {name} 4\ta\0"),
            CandidateFailure::ListingMalformed,
        ),
        (
            format!("100644  {name} 0\ta\0"),
            CandidateFailure::ListingMalformed,
        ),
        (
            format!("100644 {} 0\ta\0", name.to_uppercase()),
            CandidateFailure::ListingMalformed,
        ),
    ] {
        check_stage_records(bytes.as_bytes());
        match parse_stage_records(bytes.as_bytes()) {
            Ok(parsed) => panic!("{bytes:?} was accepted as {parsed:?}"),
            Err(error) => assert_eq!(error.failure, failure, "{bytes:?}"),
        }
    }
    for data in inputs() {
        check_stage_records(data.as_slice());
    }
}

/// Generated `rulesFile` values reach acceptance and every refusal.
#[test]
fn generated_rules_files_reach_every_refusal() {
    let mut accepted: usize = 0;
    let mut refusals: Vec<RulesFileRefusal> = Vec::new();
    for data in inputs() {
        let value: String = generated_rules_file(data.as_slice());
        check_rules_file_value(value.as_str());
        match check_rules_file(value.as_str()) {
            Ok(()) => accepted += 1,
            Err(refusal) => {
                if !refusals.contains(&refusal) {
                    refusals.push(refusal);
                }
            }
        }
        // Raw bytes that happen to be text are checked as written too.
        if let Ok(text) = std::str::from_utf8(data.as_slice()) {
            check_rules_file_value(text);
        }
    }
    assert!(accepted > 0);
    for refusal in [
        RulesFileRefusal::Empty,
        RulesFileRefusal::Absolute,
        RulesFileRefusal::Drive,
        RulesFileRefusal::Backslash,
        RulesFileRefusal::Nul,
        RulesFileRefusal::EmptyComponent,
        RulesFileRefusal::DotComponent,
    ] {
        assert!(
            refusals.contains(&refusal),
            "{refusal:?} was never generated"
        );
    }
    for value in [
        "rules.txt",
        "a/../b",
        "../x",
        "./x",
        "x/",
        "C:x",
        "...",
        "a/.../b",
    ] {
        check_rules_file_value(value);
    }
}

/// Generated text reaches every final-newline outcome,
///  and raw bytes reach the others.
#[test]
fn generated_text_reaches_every_final_newline_outcome() {
    let mut missing: usize = 0;
    let mut extra: usize = 0;
    let mut canonical: usize = 0;
    let mut left_alone: usize = 0;
    for data in inputs() {
        check_final_newline(data.as_slice());
        let text: Vec<u8> = generated_text(data.as_slice());
        check_final_newline(text.as_slice());
        match normalized_final_newline(text.as_slice()) {
            Some(_) if text.last() == Some(&b'\n') => extra += 1,
            Some(_) => missing += 1,
            None if text.last() == Some(&b'\n') => canonical += 1,
            None => left_alone += 1,
        }
    }
    assert!(
        missing > 0 && extra > 0 && canonical > 0 && left_alone > 0,
        "missing {missing}, extra {extra}, canonical {canonical}, left alone {left_alone}"
    );
    for bytes in [
        b"".as_slice(),
        b"\n",
        b"\n\n",
        b"a\r\n",
        b"a\n\r",
        b"\0\n\n",
        b"\xff\n\n",
        "\u{e9}".as_bytes(),
    ] {
        check_final_newline(bytes);
    }
}
