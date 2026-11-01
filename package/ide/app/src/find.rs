//! In-file find matching:
//!  plain literal,
//!  case-insensitive substrings in canonical source characters.

/// What:
///  `Result` carries either a value or an error;
///  `bail!` returns an error built from a message.
/// Why:
///  An over-long query or oversized source is reported to the find bar instead of matching nothing silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // functions below `throw new Error(message)` instead of returning a Result
/// ```
use anyhow::{Context, Result, bail};
/// What:
///  Helix reexports the `regex` crate;
///  `escape` turns every query character into a literal.
/// Why:
///  The matcher reuses an existing dependency and never interprets user input as pattern syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const escapeLiteral = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
/// ```
use helix_core::{
    Rope,
    regex::{RegexBuilder, escape},
};
/// What:
///  `Arc` is a shared pointer that may cross threads;
///  its sibling `Rc` may not.
/// Why:
///  The worker produces the match list and the native thread paints it without copying every range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = Readonly<T>; // garbage collection already shares objects
/// ```
use std::sync::Arc;

/// What:
///  `usize` is the address-sized unsigned integer;
///  siblings are `u32`,
///  `u64`,
///  and signed `i64`.
/// Why:
///  Vector lengths are `usize`,
///  and ten thousand ranges bound reply memory at 160 kB on this host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MAX_FIND_MATCHES = 10_000;
/// ```
pub const MAX_FIND_MATCHES: usize = 10_000;

/// Longer queries are rejected with a diagnostic,
///  bounding the compiled matcher and its build time.
pub const MAX_FIND_QUERY_CHARS: usize = 1_000;

/// The worker copies the source into one contiguous string;
///  larger files report a diagnostic instead.
pub const MAX_FIND_SOURCE_BYTES: usize = 64 * 1024 * 1024;

/// What:
///  A copyable record of one match;
///  `derive` asks the compiler to write copy,
///  print,
///  and equality code.
/// Why:
///  Ranges use source character positions,
///  the same unit as selection,
///  copying,
///  and reload mapping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FindRange = { start: number; end: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FindRange {
    /// Inclusive source character position,
    ///  not a UTF-8 byte or UTF-16 unit.
    pub start: usize,
    /// Exclusive source character position;
    ///  always greater than `start`.
    pub end: usize,
}

/// What:
///  A shared immutable list;
///  `[FindRange]` is a variable-length sequence,
///  unlike the growable `Vec`.
/// Why:
///  Paint identity can compare one pointer instead of every range on each caret movement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FindRanges = ReadonlyArray<FindRange>;
/// ```
pub type FindRanges = Arc<[FindRange]>;

/// Ordered,
///  non-overlapping matches plus whether the retained list stopped at its limit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindMatches {
    /// Ascending document order;
    ///  navigation and painting both binary-search this list.
    pub ranges: FindRanges,
    /// True when more matches exist than were retained.
    pub truncated: bool,
}

/// What:
///  The single matching function:
///  `&str` borrows text without owning it,
///  unlike `String`.
/// Why:
///  Replacing the matching semantics later means replacing this one function.
/// An empty query matches nothing;
///  every other query is compared literally,
///  ignoring Unicode simple case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findMatches(text: string, query: string, limit: number): FindMatches;
/// ```
pub fn find_matches(text: &str, query: &str, limit: usize) -> Result<FindMatches> {
    // What: `Vec::new()` creates an empty growable list.
    // Why: The number of matches is unknown until the scan finishes or reaches its limit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ranges: FindRange[] = [];
    // ```
    let mut ranges = Vec::new();
    if query.is_empty() {
        // What: `Ok(...)` wraps the successful value; `Arc::from` moves the list into shared storage.
        // Why: No query is a valid state with zero matches, not an error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { ranges: [], truncated: false };
        // ```
        return Ok(FindMatches {
            ranges: Arc::from(ranges),
            truncated: false,
        });
    }
    // What: `chars()` walks Unicode scalar values and `count()` counts them.
    // Why: The documented query bound is in characters, not UTF-8 bytes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const length = [...query].length;
    // ```
    let length = query.chars().count();
    if length > MAX_FIND_QUERY_CHARS {
        bail!(
            "Find text has {length} characters; use at most {MAX_FIND_QUERY_CHARS} characters to search this file"
        );
    }
    // What: `&escape(query)` lends the escaped pattern; `mut` lets the builder be configured.
    // Why: Regex punctuation in the query must match itself rather than broaden the search.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const builder = { source: escapeLiteral(query), flags: 'u' };
    // ```
    let mut builder = RegexBuilder::new(&escape(query));
    builder.case_insensitive(true);
    builder.unicode(true);
    // What: `?` returns the build error to the caller; `context` prefixes what failed.
    // Why: A matcher that cannot be built is a visible diagnostic, never an empty result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const matcher = new RegExp(builder.source, 'giu'); // throws on failure
    // ```
    let matcher = builder
        .build()
        .context("Cannot prepare this find text for literal matching")?;
    let mut byte = 0;
    let mut character = 0;
    let mut truncated = false;
    for found in matcher.find_iter(text) {
        if ranges.len() == limit {
            truncated = true;
            break;
        }
        // What: `text[byte..found.start()]` borrows the bytes between two matches.
        // Why: Counting only the gap keeps byte-to-character conversion one linear pass over the source.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // character += [...text.slice(byte, found.start)].length;
        // ```
        character += text[byte..found.start()].chars().count();
        let start = character;
        character += found.as_str().chars().count();
        byte = found.end();
        ranges.push(FindRange {
            start,
            end: character,
        });
    }
    return Ok(FindMatches {
        ranges: Arc::from(ranges),
        truncated,
    });
}

/// What:
///  `&Rope` borrows the document's chunked text;
///  `String::from` copies it into contiguous bytes.
/// Why:
///  The regex engine needs one contiguous string,
///  so the copy is bounded before it is made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findInSource(source: Rope, query: string): FindMatches;
/// ```
pub fn find_in_source(source: &Rope, query: &str) -> Result<FindMatches> {
    return find_in_bounded_source(source, query, MAX_FIND_SOURCE_BYTES, MAX_FIND_MATCHES);
}

/// Explicit bounds let tests observe the source-size and match-count limits without a 64 MiB fixture.
pub fn find_in_bounded_source(
    source: &Rope,
    query: &str,
    source_bytes: usize,
    limit: usize,
) -> Result<FindMatches> {
    if query.is_empty() {
        return find_matches("", query, limit);
    }
    let bytes = source.len_bytes();
    if bytes > source_bytes {
        bail!(
            "This file has {bytes} bytes; in-file find reads at most {source_bytes} bytes. Use project search for larger files"
        );
    }
    let text = String::from(source);
    return find_matches(&text, query, limit);
}
