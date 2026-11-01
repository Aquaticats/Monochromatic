//! What:
//!  One parsed Rust source and its reusable code-line and position indexes.
//! Why:
//!  Both Rust rules inspect the same syntax tree without reparsing or counting comments as code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class RustSource { source: string; syntax: SyntaxNode; codeLines: number[] }
//! ```

/// Import the common diagnostic wire range.
use crate::diagnostic::Span;
/// Import the exact rust-analyzer syntax interface already used by the incumbent.
use ra_ap_syntax::{Edition, NodeOrToken, SourceFile, SyntaxKind, SyntaxNode};

/// What:
///  Owned per-file source with a parser-owned syntax handle.
/// Why:
///  The context outlives individual rule calls while lending read-only views.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RustSource { filename: string; source: string; syntax: SyntaxNode }
/// ```
pub struct RustSource {
    /// Real or virtual input name;
    ///  processors later map findings to their host.
    pub filename: String,
    /// Exact text parsed by the lexer.
    pub source: String,
    /// Parser handle retained for documentable-item traversal.
    syntax: SyntaxNode,
    /// Byte offset of every LF-delimited line start,
    ///  including an empty final line.
    line_starts: Vec<usize>,
    /// One-based lines touched by non-comment,
    ///  non-whitespace tokens.
    code_lines: Vec<usize>,
}

/// What:
///  Build the incumbent Rust line-index convention in a single byte walk.
/// Why:
///  Rust findings retain their existing LF and byte-column behavior.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineStarts(bytes: Uint8Array): number[];
/// ```
fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (offset, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(offset + 1);
        }
    }
    return starts;
}

/// What:
///  Find the containing line by binary search over sorted byte starts.
/// Why:
///  Tokens already carry byte positions and do not need text rescanning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineIndex(offset: number, starts: number[]): number;
/// ```
fn line_index(offset: usize, starts: &[usize]) -> usize {
    // What: `count` is how many line starts lie at or before `offset`:
    //  `binary_search` reports `Ok(found)` for an exact line start,
    //  which counts itself and everything before it,
    //  while `Err(insertion)` is already the count of earlier starts.
    // Why: the named fold replaces `partition_point`,
    //  whose closure would borrow `offset`,
    //  and `count - 1` gives the containing line exactly as before.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const count = binarySearchCount(starts, offset);
    // ```
    let count: usize = match starts.binary_search(&offset) {
        Ok(found) => found + 1,
        Err(insertion) => insertion,
    };
    return count - 1;
}

/// What:
///  Classify code lines using the real lexer instead of comment-like substrings.
/// Why:
///  Comments inside string literals remain code and multiline tokens cover all their lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function codeLines(syntax: SyntaxNode, starts: number[]): number[];
/// ```
fn code_lines(syntax: &SyntaxNode, starts: &[usize]) -> Vec<usize> {
    let mut marked = vec![false; starts.len()];
    for element in syntax.descendants_with_tokens() {
        let NodeOrToken::Token(token) = element else {
            continue;
        };
        let kind = token.kind();
        if kind == SyntaxKind::COMMENT || kind == SyntaxKind::WHITESPACE {
            continue;
        }
        let range = token.text_range();
        let start = usize::from(range.start());
        let end = usize::from(range.end());
        if end <= start {
            continue;
        }
        let first = line_index(start, starts);
        let last = line_index(end - 1, starts);
        for line in &mut marked[first..=last] {
            *line = true;
        }
    }
    let mut result = Vec::new();
    for (index, is_code) in marked.iter().enumerate() {
        if *is_code {
            result.push(index + 1);
        }
    }
    return result;
}

/// What:
///  Construct and query one Rust parse.
/// Why:
///  Parser errors are deliberately not findings,
///  preserving compile_fail-snippet behavior.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Parse once; rule code uses indexed source and the recoverable syntax tree.
/// ```
impl RustSource {
    /// Index one already associated text/tree pair without another parse.
    fn indexed(filename: String, source: String, syntax: SyntaxNode) -> RustSource {
        let starts: Vec<usize> = line_starts(source.as_str());
        let lines: Vec<usize> = code_lines(&syntax, starts.as_slice());
        return RustSource {
            filename,
            source,
            syntax,
            line_starts: starts,
            code_lines: lines,
        };
    }

    /// Parse under the already selected current Rust edition.
    pub fn new(filename: String, source: String) -> RustSource {
        let parsed = SourceFile::parse(source.as_str(), Edition::CURRENT);
        let syntax: SyntaxNode = parsed.syntax_node();
        return RustSource::indexed(filename, source, syntax);
    }

    /// What:
    ///  Reuse a semantic database's registered syntax root and its exact lossless text.
    /// Why:
    ///  Semantic queries require nodes belonging to that database;
    ///  a second standalone parse loses that identity.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromSyntax(filename: string, syntax: SyntaxNode): RustSource;
    /// ```
    pub fn from_syntax(filename: String, syntax: SyntaxNode) -> RustSource {
        // Reconstruct owned source bytes from lossless tokens, not a formatter or diagnostic display.
        let source: String = syntax.text().to_string();
        return RustSource::indexed(filename, source, syntax);
    }

    /// Borrow the retained syntax handle for rule traversal.
    pub fn syntax(&self) -> &SyntaxNode {
        return &self.syntax;
    }

    /// Count distinct code lines after token classification.
    pub fn code_line_count(&self) -> usize {
        return self.code_lines.len();
    }

    /// Read the one-based line at a zero-based code-line index.
    pub fn code_line_at(&self, index: usize) -> Option<usize> {
        return self.code_lines.get(index).copied();
    }

    /// Return a whole LF-delimited line span,
    ///  excluding the final LF exactly as the incumbent does.
    pub fn line_span(&self, line: usize) -> Option<Span> {
        if line == 0 {
            return None;
        }
        let start = *self.line_starts.get(line - 1)?;
        let next = self
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(self.source.len());
        // An empty slice cannot end in LF, so a separate next > start guard is redundant.
        let end = if self.source[start..next].ends_with('\n') {
            next - 1
        } else {
            next
        };
        return Some(Span {
            offset: start,
            length: end - start,
            line,
            column: 1,
        });
    }

    /// Resolve a byte range and clamp its underline to the first line,
    ///  preserving Rust byte columns.
    pub fn span(&self, offset: usize, length: usize) -> Span {
        let index = line_index(offset, self.line_starts.as_slice());
        let start = self.line_starts[index];
        let next = self
            .line_starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.source.len());
        // ends_with is false for the empty final line as well as every non-LF ending.
        let end = if self.source[start..next].ends_with('\n') {
            next - 1
        } else {
            next
        };
        return Span {
            offset,
            length: length.min(end.saturating_sub(offset)),
            line: index + 1,
            column: offset.saturating_sub(start).saturating_add(1),
        };
    }
}

/// Keep source-index controls outside release artifacts.
#[cfg(test)]
#[path = "rust_source_tests.rs"]
mod tests;
