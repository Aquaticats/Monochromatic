//! What:
//!  Controls for the POSIX-family and fish scanner.
//! Why:
//!  Quotes,
//!  comments,
//!  redirections,
//!  heredocs and GitHub expressions each decide whether a byte is an operator.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('posix scan', () => { /* operators, quoting, redirections, heredocs */ });
//! ```

/// Import the scanner and the construct names.
use super::scan;
use crate::shell_dialect::Construct;

/// The constructs found,
///  in source order.
fn kinds(text: &str, fish: bool) -> Vec<Construct> {
    let mut found: Vec<Construct> = Vec::new();
    for hit in scan(text, fish) {
        found.push(hit.construct);
    }
    return found;
}

/// Each operator is found once,
///  with the longest match winning.
#[test]
fn operators_are_found() {
    assert_eq!(kinds("a | b", false), [Construct::Pipe]);
    assert_eq!(kinds("a && b || c", false), [Construct::AndChain, Construct::OrChain]);
    assert_eq!(kinds("a |& b", false), [Construct::PipeStderr]);
    assert_eq!(kinds("a; b", false), [Construct::Semicolon]);
    assert_eq!(kinds("case x in a) b;; esac", false), [Construct::Semicolon]);
    assert_eq!(kinds("a &", false), [Construct::Ampersand]);
    assert_eq!(kinds("echo $(date)", false), [Construct::DollarParen]);
    assert_eq!(kinds("echo `date`", false), [Construct::Backtick]);
    assert_eq!(kinds("echo plain words", false), []);
}

/// Hits carry the exact byte range.
#[test]
fn hit_ranges_are_exact() {
    let hits = scan("é && b", false);
    assert_eq!(hits.len(), 1);
    assert_eq!((hits[0].start, hits[0].end), (3, 5));
}

/// Redirections that contain `&` or `|` are not operators,
///  and arithmetic is not a command substitution.
#[test]
fn redirections_and_arithmetic_are_not_operators() {
    for text in ["cmd 2>&1", "cmd &> out", "cmd &>> out", "cmd >| out", "cmd <&3", "echo $((1 + 2))"] {
        assert_eq!(kinds(text, false), [], "{text}");
    }
}

/// Single-quoted text is inert,
///  and double-quoted text runs only substitutions.
#[test]
fn quotes_hide_operators_but_not_substitutions() {
    assert_eq!(kinds("echo 'a | b && c; $(x) `y`'", false), []);
    assert_eq!(kinds("echo \"a | b && c; d\"", false), []);
    assert_eq!(kinds("echo \"$(date)\"", false), [Construct::DollarParen]);
    assert_eq!(kinds("echo \"`date`\"", false), [Construct::Backtick]);
    assert_eq!(kinds("echo \"\\$(date)\"", false), []);
    assert_eq!(kinds("echo $'a\\'b | c'", false), []);
    assert_eq!(kinds("echo unterminated 'a | b", false), []);
    assert_eq!(kinds("echo unterminated \"a | b", false), []);
}

/// A substitution's contents are code,
///  including nested quotes.
#[test]
fn substitutions_nest() {
    assert_eq!(
        kinds("echo $(a | b)", false),
        [Construct::DollarParen, Construct::Pipe]
    );
    assert_eq!(
        kinds("echo \"$(echo \"x | y\")\" | z", false),
        [Construct::DollarParen, Construct::Pipe]
    );
    assert_eq!(kinds("echo $(a (b) | c) | d", false).len(), 3);
}

/// Comments start only at the start of a word.
#[test]
fn comments_end_the_line() {
    assert_eq!(kinds("echo a # b | c", false), []);
    assert_eq!(kinds("echo a#b|c", false), [Construct::Pipe]);
    assert_eq!(kinds("echo ${#x} | c", false), [Construct::Pipe]);
    assert_eq!(kinds("# c\na | b", false), [Construct::Pipe]);
}

/// Escapes hide the next byte,
///  and a continuation joins lines.
#[test]
fn escapes_and_continuations() {
    assert_eq!(kinds("a \\| b", false), []);
    assert_eq!(kinds("a \\\n && b", false), [Construct::AndChain]);
}

/// GitHub expressions are not shell.
#[test]
fn expressions_are_skipped() {
    assert_eq!(kinds("echo ${{ a && b || c }} | x", false), [Construct::Pipe]);
    assert_eq!(kinds("echo ${{ unterminated | x", false), []);
    assert_eq!(kinds("echo \"${{ a && b }}\"", false), []);
}

/// A here-document body is data up to its end word.
#[test]
fn heredoc_bodies_are_skipped() {
    assert_eq!(kinds("cat <<EOF\na | b; c\nEOF\necho x | y", false), [Construct::Pipe]);
    assert_eq!(kinds("cat <<-'EOF'\n\tx | y\n\tEOF\n", false), []);
    assert_eq!(kinds("cat <<\"EOF\"\nx | y\nEOF\n", false), []);
    assert_eq!(kinds("cat <<EOF | tr a b\nx; y\nEOF\n", false), [Construct::Pipe]);
    assert_eq!(kinds("cat <<A <<B\nx | y\nA\np | q\nB\n", false), []);
    assert_eq!(kinds("cat <<EOF\r\nx | y\r\nEOF\r\n", false), []);
    assert_eq!(kinds("cat <<< $x | y", false), [Construct::Pipe]);
    assert_eq!(kinds("cat <<EOF\nnever ends | x", false), []);
}

/// fish has no backtick substitution or heredoc,
///  and its single quotes take backslash escapes.
#[test]
fn fish_differs_in_three_places() {
    assert_eq!(kinds("echo it\\'s | x", true), [Construct::Pipe]);
    assert_eq!(kinds("echo 'it\\'s | x'", true), []);
    assert_eq!(kinds("echo 'it\\'s | x'", false), [Construct::Pipe]);
    assert_eq!(kinds("echo `x`", true), []);
    assert_eq!(kinds("cat <<EOF | x", true), [Construct::Pipe]);
}
