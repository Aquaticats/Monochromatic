//! What:
//!  Differential controls measured from the unchanged TypeScript lexical helpers.
//! Why:
//!  Self-authored native expectations alone do not establish incumbent parity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Compare native results with captured incumbent outputs and retain the incumbent source hashes.
//! ```

/// Import the native functions actually used by semantic-line-breaks.
use crate::markdown_block_start::starts_block_construct;
use crate::markdown_break_points::break_offsets;
/// Import the existing typed JSON decoder for the committed measured fixture.
use serde::Deserialize;

/// Captured source text and original-byte offsets from the incumbent's UTF-16 result.
#[derive(Deserialize)]
struct BreakCase {
    /// Text-node source supplied to the incumbent helper.
    slice: String,
    /// Paragraph source following the text node's resolved delimiter tail.
    trailing: String,
    /// Whether the resolved tail is the paragraph's last child.
    paragraph_tail: bool,
    /// Byte offsets converted from the incumbent's returned UTF-16 offsets.
    expected: Vec<usize>,
}

/// Captured block-start decision at one original-source offset.
#[derive(Deserialize)]
struct BlockCase {
    /// Complete source supplied to the incumbent classifier.
    source: String,
    /// Candidate insertion offset.
    at: usize,
    /// Incumbent decision.
    expected: bool,
}

/// Measured case catalogs;
///  the JSON also retains source hashes as provenance.
#[derive(Deserialize)]
struct Cases {
    /// Text-boundary cases.
    breaks: Vec<BreakCase>,
    /// Block-boundary cases.
    blocks: Vec<BlockCase>,
}

/// The actual incumbent outputs must match every captured ordinary boundary case.
#[test]
fn measured_incumbent_lexical_outputs_match_native_byte_results() {
    // Embed immutable test data, not a dependency on live repository files during a container run.
    let source: &str = include_str!("../fixtures/semantic-break-parity.json");
    let cases: Cases = serde_json::from_str::<Cases>(source).expect("measured fixture");
    // Independently counted catalogs prove neither fixture family became silently empty.
    assert_eq!(cases.breaks.len(), 44);
    assert_eq!(cases.blocks.len(), 32);
    for case in cases.breaks {
        assert_eq!(
            break_offsets(
                case.slice.as_str(),
                case.trailing.as_str(),
                case.paragraph_tail
            ),
            case.expected,
            "{}",
            case.slice
        );
    }
    for case in cases.blocks {
        assert_eq!(
            starts_block_construct(case.source.as_str(), case.at),
            case.expected,
            "{}",
            case.source
        );
    }
}

/// Unicode case expansion must not shift abbreviation guards onto unrelated original offsets.
#[test]
fn unicode_expansion_keeps_abbreviations_at_their_original_source_positions() {
    // The incumbent returned byte 13 for each of these because four capital dotted I characters expand on lowercasing.
    for source in ["İİİİ etc. Next sentence.", "İİİİ e.g. Next sentence."] {
        assert!(break_offsets(source, "", true).is_empty(), "{source}");
    }
}
