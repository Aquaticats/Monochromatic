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

/// Import the scanner,
///  the construct names and the deadline that ends a scan which stops advancing.
use super::scan;
use crate::shell_dialect::Construct;
use crate::shell_scan::Hit;
use crate::test_deadline::finished_within_deadline;

/// The hits of a scan that must finish before the deadline.
fn hits(text: &str, fish: bool) -> Vec<Hit> {
    let owned: String = String::from(text);
    return finished_within_deadline(move || return scan(owned.as_str(), fish));
}

/// The constructs found,
///  in source order.
fn kinds(text: &str, fish: bool) -> Vec<Construct> {
    let mut found: Vec<Construct> = Vec::new();
    for hit in hits(text, fish) {
        found.push(hit.construct);
    }
    return found;
}

/// The byte range of the `nth` occurrence of `needle` in `text`,
///  counted from zero.
fn nth_span(text: &str, needle: &str, nth: usize) -> (usize, usize) {
    let start: usize = text
        .match_indices(needle)
        .nth(nth)
        .expect("the needle occurs often enough")
        .0;
    return (start, start + needle.len());
}

/// Assert that a scan finds exactly the given constructs,
///  each over exactly the given occurrence of the given text.
fn assert_hits(text: &str, fish: bool, expected: &[(Construct, &str, usize)]) {
    let mut wanted: Vec<(Construct, usize, usize)> = Vec::new();
    for (construct, needle, nth) in expected {
        let span: (usize, usize) = nth_span(text, needle, *nth);
        wanted.push((*construct, span.0, span.1));
    }
    let mut found: Vec<(Construct, usize, usize)> = Vec::new();
    for hit in hits(text, fish) {
        found.push((hit.construct, hit.start, hit.end));
    }
    assert_eq!(found, wanted, "{text:?}");
}

/// Each operator is found once,
///  with the longest match winning.
#[test]
fn operators_are_found() {
    assert_eq!(kinds("a | b", false), [Construct::Pipe]);
    assert_eq!(
        kinds("a && b || c", false),
        [Construct::AndChain, Construct::OrChain]
    );
    assert_eq!(kinds("a |& b", false), [Construct::PipeStderr]);
    assert_eq!(kinds("a; b", false), [Construct::Semicolon]);
    assert_eq!(
        kinds("case x in a) b;; esac", false),
        [Construct::Semicolon]
    );
    assert_eq!(kinds("a &", false), [Construct::Ampersand]);
    assert_eq!(kinds("echo $(date)", false), [Construct::DollarParen]);
    assert_eq!(kinds("echo `date`", false), [Construct::Backtick]);
    assert_eq!(kinds("echo plain words", false), []);
}

/// Hits carry the exact byte range.
#[test]
fn hit_ranges_are_exact() {
    let found = hits("é && b", false);
    assert_eq!(found.len(), 1);
    assert_eq!((found[0].start, found[0].end), (3, 5));
}

/// Redirections that contain `&` or `|` are not operators,
///  and arithmetic is not a command substitution.
#[test]
fn redirections_and_arithmetic_are_not_operators() {
    for text in [
        "cmd 2>&1",
        "cmd &> out",
        "cmd &>> out",
        "cmd >| out",
        "cmd <&3",
        "echo $((1 + 2))",
    ] {
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
    assert_eq!(
        kinds("echo ${{ a && b || c }} | x", false),
        [Construct::Pipe]
    );
    assert_eq!(kinds("echo ${{ unterminated | x", false), []);
    assert_eq!(kinds("echo \"${{ a && b }}\"", false), []);
}

/// A here-document body is data up to its end word.
#[test]
fn heredoc_bodies_are_skipped() {
    assert_eq!(
        kinds("cat <<EOF\na | b; c\nEOF\necho x | y", false),
        [Construct::Pipe]
    );
    assert_eq!(kinds("cat <<-'EOF'\n\tx | y\n\tEOF\n", false), []);
    assert_eq!(kinds("cat <<\"EOF\"\nx | y\nEOF\n", false), []);
    assert_eq!(
        kinds("cat <<EOF | tr a b\nx; y\nEOF\n", false),
        [Construct::Pipe]
    );
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

/// A here-document opener ends at its word,
///  and only that body is data:
///  the code after the opener on its line and after the end word is still scanned.
#[test]
fn heredoc_openers_skip_exactly_their_own_bodies() {
    let pipe_after: &[(Construct, &str, usize)] = &[(Construct::Pipe, "|", 1)];
    assert_hits("cat <<EOF\nx | y\nEOF\na | b", false, pipe_after);
    assert_hits("cat << EOF\nx | y\nEOF\na | b", false, pipe_after);
    assert_hits("cat <<\tEOF\nx | y\nEOF\na | b", false, pipe_after);
    assert_hits("cat <<-EOF\n\tx | y\n\tEOF\na | b", false, pipe_after);
    assert_hits("cat <<- EOF\n\tx | y\n\tEOF\na | b", false, pipe_after);
    assert_hits("cat <<'EOF'\nx | y\nEOF\na | b", false, pipe_after);
    assert_hits("cat <<\"EOF\"\nx | y\nEOF\na | b", false, pipe_after);
    assert_hits("cat <<'A B'\nx | y\nA B\na | b", false, pipe_after);
    assert_hits("cat <<\\EOF\nx | y\nEOF\na | b", false, pipe_after);
    assert_hits(
        "cat <<'EOF' | tr a b\nx; y\nEOF\nc; d",
        false,
        &[(Construct::Pipe, "|", 0), (Construct::Semicolon, ";", 1)],
    );
    assert_hits(
        "cat <<EOF;ls | x\nbody | y\nEOF\n",
        false,
        &[(Construct::Semicolon, ";", 0), (Construct::Pipe, "|", 0)],
    );
    // A line that only starts with the word does not end the body.
    assert_hits("cat <<EOF\nEOF x | y\nEOF\nz | w", false, pipe_after);
    // A tab before the word ends a body only for `<<-`.
    assert_hits(
        "cat <<EOF\n\tEOF\nx | y\nEOF\nz | w",
        false,
        &[(Construct::Pipe, "|", 1)],
    );
    assert_hits(
        "cat <<A <<B\nx | y\nA\np | q\nB\nz | w",
        false,
        &[(Construct::Pipe, "|", 2)],
    );
    assert_hits("cat <<A <<B\nx | y\nA\np | q\nB", false, &[]);
}

/// Text that only looks like a here-document opener opens none,
///  so the lines after it are code.
#[test]
fn text_that_is_not_a_heredoc_opener_leaves_the_next_lines_as_code() {
    let first_pipe: &[(Construct, &str, usize)] = &[(Construct::Pipe, "|", 0)];
    assert_hits("cat <<<EOF\nx | y\nEOF\n", false, first_pipe);
    assert_hits("cat <<\n x | y", false, first_pipe);
    assert_hits("cat <<\\\n x | y", false, first_pipe);
    assert_hits("cat <<-\n x | y", false, first_pipe);
    assert_hits("cat <<EOF\nx | y", true, first_pipe);
    assert_hits("cat <<", false, &[]);
    assert_hits("cat << ", false, &[]);
    assert_hits("cat <<-", false, &[]);
    assert_hits("cat <<EOF", false, &[]);
    assert_hits("cat <<'EOF", false, &[]);
    assert_hits(
        "cat <<(x) | y\nz | w",
        false,
        &[(Construct::Pipe, "|", 0), (Construct::Pipe, "|", 1)],
    );
}

/// A scan resumes after quoted text,
///  escaped bytes and substitutions at exactly the byte after them.
#[test]
fn quoting_resumes_scanning_at_the_byte_after_the_quote() {
    let second_pipe: &[(Construct, &str, usize)] = &[(Construct::Pipe, "|", 1)];
    assert_hits("echo 'a | b' | c", false, second_pipe);
    assert_hits(
        "echo `a\\`b | c` | d",
        false,
        &[(Construct::Backtick, "`", 0), (Construct::Pipe, "|", 1)],
    );
    assert_hits("echo $'a\\'b | c' | d", false, second_pipe);
    assert_hits("echo 'it\\'s | x' | y", true, second_pipe);
    assert_hits("echo `abc | d", false, &[(Construct::Backtick, "`", 0)]);
    assert_hits("echo 'abc | d", false, &[]);
    assert_hits("echo $'abc | d", false, &[]);
    assert_hits("echo \"a \\\" | b\" | c", false, second_pipe);
    assert_hits(
        "echo \"$(a | b)\" | c",
        false,
        &[
            (Construct::DollarParen, "$(", 0),
            (Construct::Pipe, "|", 0),
            (Construct::Pipe, "|", 1),
        ],
    );
    assert_hits(
        "echo \"$((1 + 2))\" | c",
        false,
        &[(Construct::Pipe, "|", 0)],
    );
    assert_hits(
        "echo \"`a | b`\" | c",
        false,
        &[(Construct::Backtick, "`", 0), (Construct::Pipe, "|", 1)],
    );
    assert_hits("echo \"`a | b`\" | c", true, second_pipe);
    assert_hits(
        "echo \"$(date)\"",
        false,
        &[(Construct::DollarParen, "$(", 0)],
    );
}

/// Arithmetic and substitutions are skipped or entered at exact bytes.
#[test]
fn arithmetic_and_substitutions_end_where_their_parentheses_do() {
    assert_hits("echo $((1 + 2)) | c", false, &[(Construct::Pipe, "|", 0)]);
    assert_hits("echo $((1 + 2", false, &[]);
    assert_hits(
        "echo $(date) | c",
        false,
        &[(Construct::DollarParen, "$(", 0), (Construct::Pipe, "|", 0)],
    );
    assert_hits("echo $'x' | c", false, &[(Construct::Pipe, "|", 0)]);
    assert_hits(
        "echo $((a)) | b",
        true,
        &[(Construct::DollarParen, "$(", 0), (Construct::Pipe, "|", 0)],
    );
    assert_hits(
        "echo $(a (b) | c) | d",
        false,
        &[
            (Construct::DollarParen, "$(", 0),
            (Construct::Pipe, "|", 0),
            (Construct::Pipe, "|", 1),
        ],
    );
}

/// Parentheses are counted inside a substitution,
///  so it ends at its own closing parenthesis and not before or after it.
#[test]
fn a_substitution_ends_at_its_own_closing_parenthesis() {
    assert_hits(
        "echo \"$(a (b) | c)\"",
        false,
        &[(Construct::DollarParen, "$(", 0), (Construct::Pipe, "|", 0)],
    );
    assert_hits(
        "echo \"$(a) | d\" | e",
        false,
        &[(Construct::DollarParen, "$(", 0), (Construct::Pipe, "|", 1)],
    );
    assert_hits("echo $x | c", false, &[(Construct::Pipe, "|", 0)]);
    assert_hits("cmd >| out | b", false, &[(Construct::Pipe, "|", 1)]);
}

/// Arithmetic ends at its own closing parentheses,
///  after double parentheses that came before it
///  and with an operator inside it that is not a pipe.
#[test]
fn arithmetic_ends_at_its_own_closing_parentheses() {
    assert_hits(
        "echo (( x )) $((1 + 2)) | c",
        false,
        &[(Construct::Pipe, "|", 0)],
    );
    assert_hits("echo $((1|2)) | c", false, &[(Construct::Pipe, "|", 1)]);
}

/// A here-document body starts on the line after its opener,
///  also when the end word is empty or one character long.
#[test]
fn heredoc_words_that_are_empty_or_one_character_long_end_their_own_body() {
    assert_hits(
        "cat <<''\nx | y\n\na | b",
        false,
        &[(Construct::Pipe, "|", 1)],
    );
    assert_hits(
        "cat <<F\nx | y\nF\na | b",
        false,
        &[(Construct::Pipe, "|", 1)],
    );
}
