//! Hover content as plain text.
//!
//! editord shows hover content with the browser's `textContent`,
//!  so Markdown arrives unrendered,
//! code fences included.
//!  This view also shows plain text,
//!  but drops the fence lines,
//!  which carry
//! no information without rendering,
//!  and bounds the length so one answer cannot stall layout.

/// The reply's hover content.
use ide_app::language::reply::HoverText;

/// Longest text shown,
///  in characters;
///  the rest is replaced by an ellipsis line.
///  Chosen,
///  not measured:
/// about forty screens of a 60-character popup,
///  far beyond what a reader scrolls through.
const LIMIT: usize = 8000;

/// True for a Markdown code-fence line,
///  such as "```rust" or "~~~".
fn fence(line: &str) -> bool {
    // `trim_start` drops leading blanks; `starts_with` tests the prefix.
    let trimmed = line.trim_start();
    return trimmed.starts_with("```") || trimmed.starts_with("~~~");
}

/// What:
///  Turn hover content into the text the popup shows.
///  `&HoverText` is lent read-only;
///       the answer is an owned `String`.
/// Why:
///  Markdown fence lines are dropped,
///  runs of blank lines collapse to one,
///  the ends are
///      trimmed,
///  and very long content is cut at `LIMIT` characters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plain(hover: HoverText): string {
///   const lines = hover.text.split('\n').filter(line => !hover.markdown || !isFence(line));
///   return collapseBlankRuns(lines).join('\n').trim().slice(0, LIMIT);
/// }
/// ```
pub(super) fn plain(hover: &HoverText) -> String {
    // `Vec::new()` creates an empty growable list of borrowed lines (sibling: fixed `[T; N]`).
    let mut kept: Vec<&str> = Vec::new();
    let mut blank = false;
    for line in hover.text.lines() {
        if hover.markdown && fence(line) {
            continue;
        }
        // A run of blank lines is kept as one.
        let empty = line.trim().is_empty();
        if empty && blank {
            continue;
        }
        blank = empty;
        kept.push(line);
    }
    let joined = kept.join("\n");
    let trimmed = joined.trim();
    // `chars().count()` counts characters, not bytes, like the limit does.
    if trimmed.chars().count() <= LIMIT {
        // `to_string` copies the borrowed trimmed text into an owned `String`.
        return trimmed.to_string();
    }
    // `take` keeps the first `LIMIT` characters; `collect` gathers them into a new `String`.
    let cut: String = trimmed.chars().take(LIMIT).collect();
    return format!("{cut}\n…");
}
