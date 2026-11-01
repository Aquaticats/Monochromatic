//! The caret stops of a line:
//!  they equal a plain walk over the line for short and long lines alike,
//!  and the work of finding them grows with the length of the line,
//! not with its square.
//! The work is counted as the bytes a lookup has to scan,
//! never timed,
//! so the checks do not depend on the host's load.

/// The walk under test,
/// the rope view the lookups run on,
/// and the flat walk that serves as the reference.
use super::{FLAT_LINE_BYTES, StopVisitor, caret_stops, caret_stops_visiting, project_line};
/// Helix's grapheme walk and rope view,
/// used by the reference walk.
use helix_core::{RopeSlice, graphemes::next_grapheme_boundary};

/// The stops a flat walk over the whole line finds:
/// the walk every line took before long lines were chunked.
fn reference_stops(source: &str, visible: usize) -> Vec<usize> {
    let slice = RopeSlice::from(source);
    let mut stops = vec![0];
    let mut position = 0;
    while position < visible {
        position = next_grapheme_boundary(slice, position).min(visible);
        stops.push(position);
    }
    return stops;
}

/// Text that stresses grapheme boundaries:
/// combining marks,
/// emoji sequences,
/// flags,
/// Hangul,
/// CJK,
/// tabs,
/// and a CRLF.
const TRICKY: &str = "ae\u{301}x \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} \u{1F1E9}\u{1F1EA}\u{1F1FA}\u{1F1F8} \u{1112}\u{1161}\u{11AB} 猫の目\tz\u{308}\u{308}!\r\n";

/// A line of `length` characters made of tricky text and plain words,
/// without a terminator.
fn long_line(length: usize) -> String {
    let mut text = String::new();
    let mut number = 0;
    while text.chars().count() < length {
        text.push_str(&TRICKY.replace("\r\n", ""));
        text.push_str(&format!(" word{number} "));
        number += 1;
    }
    return text.chars().take(length).collect();
}

/// Short and long lines get the stops a flat walk finds,
/// whichever side of the chunking limit they are on.
#[test]
fn stops_equal_the_flat_walk_on_either_side_of_the_chunking_limit() {
    let mut samples = vec![
        String::new(),
        "plain".to_string(),
        TRICKY.to_string(),
        TRICKY.repeat(3),
    ];
    for length in [
        FLAT_LINE_BYTES - 1,
        FLAT_LINE_BYTES,
        FLAT_LINE_BYTES + 1,
        4 * FLAT_LINE_BYTES + 7,
    ] {
        samples.push(long_line(length));
        samples.push("a".repeat(length));
        samples.push("猫".repeat(length));
    }
    for sample in samples {
        let visible = sample.trim_end_matches(['\r', '\n']).chars().count();
        assert_eq!(
            caret_stops(&sample, visible),
            reference_stops(&sample, visible),
            "stops differ for a line of {} bytes",
            sample.len()
        );
    }
    // The projection of a long line carries the same stops.
    let long = long_line(3 * FLAT_LINE_BYTES);
    let projection = project_line(&long);
    assert_eq!(
        projection.stops,
        reference_stops(&long, projection.source_to_byte.len() - 1)
    );
}

/// The bytes the lookups of one walk have to scan,
/// summed over the walk.
fn scanned(source: &str) -> (u64, usize) {
    let visible = source.chars().count();
    let mut counter = ScanBytes { total: 0 };
    let stops = caret_stops_visiting(source, visible, &mut counter);
    return (counter.total, stops.len());
}

/// Adds up the bytes of the chunk each lookup starts in.
struct ScanBytes {
    /// The bytes scanned so far.
    total: u64,
}

impl StopVisitor for &mut ScanBytes {
    fn visit(&mut self, slice: RopeSlice<'_>, position: usize) {
        self.total += slice.chunk_at_char(position).0.len() as u64;
    }
}

/// A lookup scans the chunk it lands in;
/// no chunk of a long line is longer than this many bytes.
const CHUNK_BYTES: u64 = 2048;

/// Finding the stops of a line of a hundred thousand characters scans a bounded number of bytes for each stop,
///  and doubling the line doubles the total instead of quadrupling it.
#[test]
fn the_work_of_finding_stops_grows_with_the_line_not_its_square() {
    let (small_total, small_stops) = scanned(&long_line(50_000));
    let (large_total, large_stops) = scanned(&long_line(100_000));
    assert!(large_stops > 60_000, "the line has stops: {large_stops}");
    assert!(
        large_total <= large_stops as u64 * CHUNK_BYTES,
        "the lookups scanned {large_total} bytes for {large_stops} stops: more than {CHUNK_BYTES} for each"
    );
    // A flat walk would scan the whole line for every stop: 100,000 stops of about 100,000 bytes each.
    assert!(
        large_total < 100_000 * 100_000 / 20,
        "the work of {large_total} bytes is near the square of the length"
    );
    let growth = large_total as f64 / small_total as f64;
    assert!(
        growth < 3.0,
        "doubling the line multiplied the scanned bytes by {growth} ({small_total} for {small_stops} stops, \
         {large_total} for {large_stops})"
    );
}
