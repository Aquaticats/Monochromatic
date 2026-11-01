//! What: Controls proving the Markdown generators reach every outcome and the invariants
//!       hold on fixed hard cases.
//! Why: An invariant that is never reached proves nothing; these controls count what the
//!      generators produce and run the same checks the fuzz targets run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(corruptionsReachedBy(generatedRecords)).toEqual(CORRUPTIONS);
//! ```

/// Import the generators and invariants under control.
use super::{
    CORRUPTIONS, check_event_path, check_generated_records, check_linter_config, check_linter_run,
    decode_base64, generated_patterns, generated_records,
};
use std::collections::BTreeSet;

/// A deterministic sample of fuzz inputs: the empty one, every two-byte input, and longer mixes.
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

/// Every corruption kind, and clean runs of none, one and several records, are reached.
#[test]
fn generated_records_reach_every_outcome() {
    let mut kinds: BTreeSet<&'static str> = BTreeSet::new();
    let mut clean_counts: BTreeSet<usize> = BTreeSet::new();
    for input in inputs() {
        let (_stderr, findings, corrupted) = generated_records(input.as_slice());
        match corrupted {
            Some((_, kind)) => {
                kinds.insert(kind);
            }
            None => {
                clean_counts.insert(findings.len());
            }
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

/// Raw runs reach both verdicts, and the hard cases hold.
#[test]
fn raw_runs_hold_the_invariants() {
    for input in inputs() {
        check_linter_run(input.as_slice());
    }
    let record: &[u8] = b"{\"message\":\"m\",\"code\":\"markdown/lfs-image-url\",\"severity\":\"warn\",\"labels\":[{\"span\":{\"line\":1,\"column\":2}}]}\n";
    for stderr in [&b""[..], record, b"\n", b"x", b"{}\n", b"\xff\n"] {
        let mut data: Vec<u8> = vec![0, 1, b'x'];
        data.extend_from_slice(stderr);
        check_linter_run(data.as_slice());
        data[0] = 210;
        check_linter_run(data.as_slice());
    }
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
