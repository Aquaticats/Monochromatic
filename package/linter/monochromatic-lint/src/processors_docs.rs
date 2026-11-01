//! What:
//!  Authored Rustdoc tokens grouped into exact virtual Markdown inputs.
//! Why:
//!  String literals and #[doc] attributes must not become comment processors.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Visit native comment tokens, group adjacent matching line prefixes, copy block bodies.
//! ```

/// Retain original physical lines and mapping records.
use crate::processors_lines::{copy_line, physical_lines};
/// Keep extraction failures explicit and snapshots immutable.
use crate::processors_model::{Guard, Mapping, ProcessorError, ProcessorLanguage};
/// Import the parsed Rust context and authored comment token interface.
use crate::rust_source::RustSource;
/// Comment classification distinguishes /// from //// and /** from /***.
use ra_ap_syntax::ast::Comment;
/// AstToken casts only real comment tokens,
///  not comment-looking strings.
use ra_ap_syntax::{AstToken, NodeOrToken, SyntaxKind};
/// Share the parent snapshot between documented items.
use std::sync::Arc;

/// Native comment coordinates,
///  owned so no parser borrow escapes extraction.
struct DocToken {
    /// Inclusive byte start of the comment token.
    start: usize,
    /// Exclusive byte end of the comment token.
    end: usize,
    /// Native comment prefix,
    ///  including inner/outer placement.
    prefix: String,
}

/// Discover real doc comments from recoverable Rust syntax.
fn tokens(parent: &Mapping) -> Vec<DocToken> {
    // Parse the exact supplied snapshot, not a filesystem path.
    let parsed: RustSource = RustSource::new(parent.filename.clone(), parent.text.clone());
    let mut result: Vec<DocToken> = Vec::new();
    for element in parsed.syntax().descendants_with_tokens() {
        let NodeOrToken::Token(token) = element else {
            continue;
        };
        if token.kind() != SyntaxKind::COMMENT {
            continue;
        }
        let Some(comment) = Comment::cast(token) else {
            continue;
        };
        if !comment.is_doc() {
            continue;
        }
        result.push(DocToken {
            // Convert the parser's compact address into platform-sized string indexes.
            start: usize::from(comment.syntax().text_range().start()),
            end: usize::from(comment.syntax().text_range().end()),
            prefix: String::from(comment.prefix()),
        });
    }
    return result;
}

/// Borrow the physical start containing a comment token.
fn line_start(source: &str, offset: usize) -> usize {
    // A backward search is bounded to the current physical line.
    if let Some(index) = source[..offset].rfind(['\r', '\n']) {
        return index + 1;
    }
    return 0;
}

/// Copy a line-doc token,
///  excluding conventional spacing and its run's common margin.
fn line_doc(child: &mut Mapping, source: &str, token: &DocToken, margin: usize) {
    let physical: usize = line_start(source, token.start);
    let mut payload: usize = token.start + 3;
    if source[payload..token.end].starts_with(' ') {
        payload += 1;
    }
    let body: &str = &source[payload..token.end];
    let indent: usize = body.len() - body.trim_start_matches([' ', '\t']).len();
    payload += indent.min(margin);
    let mut end: usize = token.end;
    // The native lexer ends a line comment at LF or EOF, so a CRLF's carriage return is already inside the token.
    if source[end..].starts_with('\n') {
        end += 1;
    }
    // Preserve indentation and the exact outer/inner prefix spelling.
    let prefix: String = String::from(&source[physical..payload]);
    copy_line(child, source, physical, payload, end, prefix);
}

/// Compute the common indentation of nonempty authored line-doc bodies.
fn run_margin(source: &str, comments: &[DocToken]) -> usize {
    let mut margin: usize = usize::MAX;
    for comment in comments {
        let mut body: &str = &source[comment.start + 3..comment.end];
        if body.starts_with(' ') {
            body = &body[1..];
        }
        if body.trim().is_empty() {
            continue;
        }
        margin = margin.min(body.len() - body.trim_start_matches([' ', '\t']).len());
    }
    return if margin == usize::MAX { 0 } else { margin };
}

/// Compute a block's common continuation margin and consistent decorative-star convention.
fn block_margin(body: &str) -> (usize, bool) {
    let mut margin: usize = usize::MAX;
    let mut stars: bool = true;
    let mut nonempty: bool = false;
    for (index, line) in physical_lines(body).iter().enumerate() {
        let written: &str = &body[line.start..line.content_end];
        if written.trim().is_empty() {
            continue;
        }
        let trimmed: &str = written.trim_start_matches([' ', '\t']);
        if index == 0 {
            // Inline opening prose already establishes a zero common continuation margin.
            margin = 0;
            continue;
        }
        nonempty = true;
        margin = margin.min(written.len() - trimmed.len());
        stars = stars && trimmed.starts_with('*');
    }
    return (
        if margin == usize::MAX { 0 } else { margin },
        stars && nonempty,
    );
}

/// Copy a block-doc body while preserving delimiters outside editable envelopes.
fn block_doc(child: &mut Mapping, source: &str, token: &DocToken) -> Result<(), ProcessorError> {
    if !source[token.start..token.end].ends_with("*/") {
        return Err(child.error("Authored Rustdoc block is unterminated; its closing delimiter cannot be projected safely."));
    }
    let start: usize = token.start + 3;
    let end: usize = token.end - 2;
    let body: &str = &source[start..end];
    let (margin, stars): (usize, bool) = block_margin(body);
    for (index, line) in physical_lines(body).iter().enumerate() {
        let physical: usize = start + line.start;
        let written: &str = &body[line.start..line.content_end];
        let trimmed: &str = written.trim_start_matches([' ', '\t']);
        let decorated: bool = stars && index != 0 && trimmed.starts_with('*');
        let mut payload: usize = physical;
        if decorated {
            payload += written.len() - trimmed.len() + 1;
            if source[payload..start + line.content_end].starts_with(' ') {
                payload += 1;
            }
        } else if index == 0 && written.starts_with(' ') {
            payload += 1;
        } else if index != 0 {
            payload += (written.len() - trimmed.len()).min(margin);
        }
        // Block delimiters are never deleted by full virtual-line removal.
        let envelope: usize = if index == 0 { payload } else { physical };
        let continuation: String = if index != 0 {
            String::from(&source[physical..payload])
        } else {
            String::new()
        };
        copy_line(
            child,
            source,
            envelope,
            payload,
            start + line.end,
            continuation,
        );
    }
    return Ok(());
}

/// What:
///  Decide whether `next` continues the line-comment run that `previous` belongs to.
/// Why:
///  A run is one virtual Markdown file,
///  so the same prefix,
///  a whitespace-only gap and exactly one
/// line ending must all hold;
///  block comments never join anything,
///  because each block is its own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function joinsRun(source: string, previous: DocToken, next: DocToken): boolean;
/// ```
fn joins_run(source: &str, previous: &DocToken, next: &DocToken) -> bool {
    // A block's prefix starts with `/*`; equal prefixes then also rule out a block before a line comment.
    if next.prefix.starts_with("/*") || next.prefix != previous.prefix {
        return false;
    }
    // Borrow the authored bytes between the two tokens; range syntax is a checked byte slice.
    let gap: &str = &source[previous.end..next.start];
    // Code or an ordinary comment between the two doc comments splits the run.
    if !gap.trim().is_empty() {
        return false;
    }
    // Exactly one newline joins a run; blank lines split it. CRLF counts once, as one physical line ending.
    let mut endings: usize = 0;
    for line in physical_lines(gap) {
        if line.content_end != line.end {
            endings += 1;
        }
    }
    return endings == 1;
}

/// What:
///  Split the comments into extraction units:
///  each block comment alone,
///  each line-comment run together.
/// Why:
///  A fixed `for` range closes a unit wherever the next comment does not join it,
/// so no mutation of an index step can make the grouping loop forever.
/// `'tokens` says every returned slice borrows from `comments`,
///  like a view that must not outlive its array.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function units(source: string, comments: DocToken[]): DocToken[][];
/// ```
fn units<'tokens>(source: &str, comments: &'tokens [DocToken]) -> Vec<&'tokens [DocToken]> {
    // Owned list of borrowed slices; `&[DocToken]` is a view into `comments`, unlike an owned `Vec<DocToken>` copy.
    let mut result: Vec<&'tokens [DocToken]> = Vec::new();
    let mut unit_start: usize = 0;
    // `1..=len` is an inclusive range: each index is a boundary candidate between `index - 1` and `index`,
    // and the final `len` always closes the last unit. An empty list yields an empty range and no unit.
    for index in 1..=comments.len() {
        let closes: bool =
            index == comments.len() || !joins_run(source, &comments[index - 1], &comments[index]);
        if closes {
            result.push(&comments[unit_start..index]);
            unit_start = index;
        }
    }
    return result;
}

/// Extract adjacent line runs and individual authored blocks;
///  ordinary gaps split runs.
pub(crate) fn docs(parent: &Arc<Mapping>) -> Result<Vec<Mapping>, ProcessorError> {
    let comments: Vec<DocToken> = tokens(parent);
    let mut result: Vec<Mapping> = Vec::new();
    for unit in units(parent.text.as_str(), comments.as_slice()) {
        // Every unit holds at least one comment; its first comment names and anchors the virtual file.
        let token: &DocToken = &unit[0];
        let block: bool = token.prefix.starts_with("/*");
        let anchor: usize = if block {
            token.start
        } else {
            line_start(parent.text.as_str(), token.start)
        };
        let first_line: usize = physical_lines(&parent.text[..token.start]).len()
            + usize::from(token.start == 0 || parent.text[..token.start].ends_with(['\r', '\n']));
        let name: String = format!("{}/{}.md", parent.filename, first_line);
        let mut child: Mapping = crate::processors::child(
            parent,
            name,
            ProcessorLanguage::Markdown,
            Guard::Docs { anchor },
            anchor,
        );
        if block {
            block_doc(&mut child, parent.text.as_str(), token)?;
        } else {
            // Only the run's first comment can follow code: later members follow a newline-only gap.
            let before: &str = &parent.text[anchor..token.start];
            if !before.trim_matches([' ', '\t']).is_empty() {
                return Err(parent.error("A Rustdoc line comment following code on the same physical line has no safe reusable container prefix."));
            }
            let margin: usize = run_margin(parent.text.as_str(), unit);
            for comment in unit {
                line_doc(&mut child, parent.text.as_str(), comment, margin);
            }
        }
        result.push(child);
    }
    return Ok(result);
}

/// Find the same authored Rustdoc container after a projected group.
pub(crate) fn doc_text(
    parent: &Arc<Mapping>,
    anchor: usize,
) -> Result<Option<String>, ProcessorError> {
    for mapping in docs(parent)? {
        // The let-chain retains a matching variant only when its authored anchor also matches.
        if let Guard::Docs { anchor: candidate } = mapping.guard
            && candidate == anchor
        {
            return Ok(Some(mapping.text));
        }
    }
    return Ok(None);
}
