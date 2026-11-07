//! What:
//! Controls proving the Markdown generators reach every outcome and the invariants
//!       hold on fixed hard cases.
//! Why:
//! An invariant that is never reached proves nothing;
//! these controls count what the
//!      generators produce and run the same checks the fuzz targets run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(corruptionsReachedBy(generatedFindings)).toEqual(CORRUPTIONS);
//! ```

/// Import the generators and invariants under control.
use super::{
    CORRUPTIONS, RunSpec, check_event_path, check_generated_records, check_linter_config,
    check_linter_run, check_oversized_runs, check_run, decode_base64, generated_finding,
    generated_findings, generated_patterns,
};
use std::collections::BTreeSet;

/// A deterministic sample of fuzz inputs:
/// the empty one,
/// every two-byte input,
/// and longer mixes.
fn inputs() -> Vec<Vec<u8>> {
    let mut all: Vec<Vec<u8>> = vec![Vec::new()];
    for first in 0..=255_u8 {
        for second in [0_u8, 1, 2, 9, 64, 127, 128, 200, 224, 231, 255] {
            all.push(vec![first, second]);
            let mut long: Vec<u8> = Vec::new();
            for index in 0..60_u8 {
                long.push(first.wrapping_mul(index).wrapping_add(second) ^ index);
            }
            all.push(long);
        }
    }
    return all;
}

/// Every corruption kind,
/// and clean runs of none,
/// one and several findings,
/// are reached.
#[test]
fn generated_findings_reach_every_outcome() {
    let mut kinds: BTreeSet<&'static str> = BTreeSet::new();
    let mut clean_counts: BTreeSet<usize> = BTreeSet::new();
    for input in inputs() {
        let generated = generated_findings(input.as_slice());
        let mut corrupted: bool = false;
        for entry in &generated {
            if let Some(kind) = entry.corruption {
                kinds.insert(kind);
                corrupted = true;
            }
        }
        if !corrupted {
            clean_counts.insert(generated.len());
        }
        check_generated_records(input.as_slice());
    }
    assert_eq!(
        kinds,
        CORRUPTIONS.iter().copied().collect::<BTreeSet<&str>>()
    );
    for count in [0, 1, 6] {
        assert!(clean_counts.contains(&count), "{clean_counts:?}");
    }
}

/// Raw runs reach every way a run ends:
/// a usable run and each failure status.
#[test]
fn raw_runs_hold_the_invariants() {
    let mut statuses: BTreeSet<Option<u8>> = BTreeSet::new();
    for input in inputs() {
        check_linter_run(input.as_slice());
        if let Some((&selector, _)) = input.split_first() {
            statuses.insert(match selector {
                0..=191 => None,
                192..=207 => Some(1),
                208..=223 => Some(2),
                other => Some(other),
            });
        }
    }
    for reached in [None, Some(1), Some(2), Some(255)] {
        assert!(statuses.contains(&reached), "{statuses:?}");
    }
}

/// The hard cases hold:
/// a run ending in each status,
/// an empty source for a file and for an empty file,
/// the
/// largest position,
/// a second label that is not read,
/// and a failure explanation past its bound.
#[test]
fn hard_runs_hold_the_invariants() {
    let warning = generated_finding(&[3, 2, 0, b'm']).finding;
    let largest = generated_finding(&[255, 1, 0]).finding;
    for failure in [None, Some(1), Some(2), Some(3), Some(255)] {
        for fixed in ["", "x"] {
            for explanation in ["", "why", &"long ".repeat(200)] {
                let spec: RunSpec = RunSpec {
                    failure,
                    explanation: String::from(explanation),
                    fixed: String::from(fixed),
                    findings: vec![warning.clone(), largest.clone()],
                };
                check_run(b"", &spec);
                check_run(b"input", &spec);
            }
        }
    }
}

/// A fixed source and findings over their size bounds are refused.
#[test]
fn oversized_runs_are_refused() {
    check_oversized_runs();
}

/// The decoder reads the RFC vectors and refuses what is not padded base64.
#[test]
fn the_decoder_reads_padded_base64_only() {
    for (text, bytes) in [
        ("", &b""[..]),
        ("Zg==", b"f"),
        ("Zm8=", b"fo"),
        ("Zm9v", b"foo"),
        ("Zm9vYmFy", b"foobar"),
        ("Y2Fm6S50eHQ=", b"caf\xe9.txt"),
    ] {
        assert_eq!(decode_base64(text).as_deref(), Some(bytes), "{text}");
    }
    for refused in ["Z", "Zg=", "Z===", "Zg=A", "Zm9*", "===="] {
        assert_eq!(decode_base64(refused), None, "{refused}");
    }
}

/// Names of every kind of byte hold the event invariants.
#[test]
fn event_paths_hold_the_invariants() {
    for input in inputs() {
        check_event_path(input.as_slice());
    }
    for name in [
        &b""[..],
        b"a.md",
        b"caf\xe9.md",
        b"\xff\xfe",
        b"\"\\\n\x01",
        "é😀".as_bytes(),
    ] {
        check_event_path(name);
    }
}

/// Patterns of every kind of character hold the configuration invariants.
#[test]
fn linter_configurations_hold_the_invariants() {
    assert_eq!(generated_patterns(b"a\xffb"), vec!["a", "b"]);
    for input in inputs() {
        check_linter_config(input.as_slice());
    }
    check_linter_config(b"\"]}\xff\\\xff\n\xff*/\xff\x00");
}
