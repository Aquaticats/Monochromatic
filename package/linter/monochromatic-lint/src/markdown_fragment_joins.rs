//! What:
//!  The fragment joins that put a word the old hard wrap cut back with the clause it belongs to.
//! Why:
//!  A break inserted after a break-point character leaves the rest of the line the author wrote
//!  as a line of its own,
//!  so the word that ended that line can stand alone at no boundary;
//!  a join changes only one line ending and the spaces around it,
//!  and never a word or the order of words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // For each fragment this pass's breaks cut, answer the line to join it with, or nothing.
//! ```

/// Import the block-start guard,
///  so a join never moves a line whose text opens a block into the middle of prose.
use crate::markdown_block_start::starts_block_construct;
/// Import the rule's own break-point ending test,
///  so a fragment the rule wants to end a line stays where it is.
use crate::markdown_break_points::ends_at_break_point;
/// Import the paragraph line-ending lookup every fragment starts from.
use crate::markdown_prose_context::line_ending;

/// What:
///  The most characters a joined line may hold,
///  counted after the indent the line is written with.
/// Why:
///  `AGENTS.md` rule MD1 asks for lines under this length;
///  the rule itself holds no limit,
///  so this bounds what one join may add and nothing else.
const JOINED_LINE_LIMIT: usize = 120;

/// The most words a fragment may hold and still be one the old hard wrap cut.
const FRAGMENT_WORDS: usize = 2;

/// The trailing spaces at which Markdown reads a hard break,
///  which a join would delete.
const HARD_BREAK_SPACES: usize = 2;

/// What:
///  The byte range one join replaces with a single space.
/// Why:
///  The range holds the fragment's trailing spacing,
///  its line ending and the next line's indentation,
///  and nothing a reader sees as a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FragmentJoin = { start: number; end: number };
/// ```
pub(crate) struct FragmentJoin {
    /// First byte of the line ending and indentation the join removes.
    pub start: usize,
    /// First byte of the following line's text.
    pub end: usize,
}

/// What:
///  Whether the line at `from` ends with a break Markdown reads as anything but a space.
/// Why:
///  A hard break and a blank line are the author's own line structure;
///  a join would delete what they said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdsOwnBreak(source: string, from: number, ending: number): boolean;
/// ```
fn holds_own_break(
    source: &str,
    from: usize,
    ending: usize,
) -> bool {
    let written: &str = &source[from..ending];
    let trimmed: &str = written.trim_end_matches([' ', '\t']);
    if trimmed.is_empty() {
        return true;
    }
    // Two trailing spaces and a trailing backslash are both hard breaks, and the spacing before either.
    return written.len() - trimmed.len() >= HARD_BREAK_SPACES || trimmed.ends_with('\\');
}

/// What:
///  Answer each fragment with the join that puts it back with the line that continues its clause.
/// Why:
///  A fragment alone on its line is the shape the old hard wrap left behind;
///  the answer holds `None` wherever the rule's own line break has to stay,
///  so a caller applies exactly one answer per break.
///  Answers arrive in the fragments' own order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fragmentJoins(source, paragraphEnd, fragments, indentWidth): (FragmentJoin | undefined)[];
/// ```
pub(crate) fn fragment_joins(
    source: &str,
    paragraph_end: usize,
    fragments: &[usize],
    indent_width: usize,
) -> Vec<Option<FragmentJoin>> {
    let mut answers: Vec<Option<FragmentJoin>> = Vec::<Option<FragmentJoin>>::new();
    for fragment in fragments {
        answers.push(fragment_join(
            source,
            paragraph_end,
            *fragment,
            indent_width,
        ));
    }
    return answers;
}

/// What:
///  One fragment's join, or nothing where its line has to stay as it is.
/// Why:
///  Every declining reason is a token the rule owns,
///  not a preference:
///  a fragment the rule would break again,
///  a line ending the author marked,
///  and a following line that opens a block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fragmentJoin(source, paragraphEnd, fragment, indentWidth): FragmentJoin | undefined;
/// ```
fn fragment_join(
    source: &str,
    paragraph_end: usize,
    fragment: usize,
    indent_width: usize,
) -> Option<FragmentJoin> {
    if fragment >= paragraph_end {
        return None;
    }
    let (ending, ending_length): (usize, usize) = line_ending(source, fragment, paragraph_end)?;
    if holds_own_break(source, fragment, ending) {
        return None;
    }
    // The fragment's last visible byte is where the join starts removing bytes.
    let written: &str = source[fragment..ending].trim_end_matches([' ', '\t']);
    let start: usize = fragment + written.len();
    let written: &str = written.trim_start_matches([' ', '\t']);
    if written.split_whitespace().count() > FRAGMENT_WORDS {
        return None;
    }
    // A fragment ending at a break point is where the rule puts a line break, so joining it would break it again.
    if ends_at_break_point(written) {
        return None;
    }
    let next: usize = ending + ending_length;
    if next >= paragraph_end {
        return None;
    }
    let rest: &str = &source[next..paragraph_end];
    let indentation: usize = rest.len() - rest.trim_start_matches([' ', '\t']).len();
    let text_start: usize = next + indentation;
    if text_start >= paragraph_end {
        return None;
    }
    // A following line that opens a quote, a list item, a fence or any other block
    // carries a marker a whitespace-only fix may not delete.
    if starts_block_construct(source, text_start) {
        return None;
    }
    let following_end: usize = match line_ending(source, text_start, paragraph_end) {
        Some((index, _)) => index.min(paragraph_end),
        None => paragraph_end,
    };
    let following: &str = source[text_start..following_end].trim_end_matches([' ', '\t']);
    let width: usize = indent_width + written.chars().count() + 1 + following.chars().count();
    if width >= JOINED_LINE_LIMIT {
        return None;
    }
    return Some(FragmentJoin {
        start,
        end: text_start,
    });
}

/// Fragment-join controls are test-only.
#[cfg(test)]
#[path = "markdown_fragment_joins_tests.rs"]
mod tests;
