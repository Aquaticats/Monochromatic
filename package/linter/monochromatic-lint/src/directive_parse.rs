//! What:
//!  Read one comment's text as a suppression directive.
//! Why:
//!  The grammar is the ESLint family's:
//!  a keyword that starts the comment,
//!  optional comma-separated rule identifiers,
//!  and a justification after a run of two or more dashes set off by whitespace.
//!  One parser serves Rust and Markdown comments,
//!  so both languages accept exactly the same text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function parseComment(content: string, contentStart: number): NotDirective | Malformed | Directive;
//! ```

/// What:
///  The text every directive keyword starts with.
/// Why:
///  A comment that starts with this prefix but is not one of the four keywords is a mistake worth reporting,
///  while a comment that merely mentions the tool name is prose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DIRECTIVE_PREFIX = 'monochromatic-lint-';
/// ```
pub const DIRECTIVE_PREFIX: &str = "monochromatic-lint-";

/// What:
///  The four directive kinds.
/// Why:
///  `Disable` and `Enable` bound a range of the file,
///  while the other two cover one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirectiveKind = 'disable' | 'enable' | 'disable-line' | 'disable-next-line';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectiveKind {
    /// Suppress from the comment to the matching enable or the end of the file.
    Disable,
    /// End the suppression that a disable began.
    Enable,
    /// Suppress on the comment's own line.
    DisableLine,
    /// Suppress on the line after the comment.
    DisableNextLine,
}

/// What:
///  Name and classify directive kinds.
/// Why:
///  Messages quote the exact keyword,
///  and line-scoped kinds must sit on a single line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// kind.keyword(); kind.isLineScoped();
/// ```
impl DirectiveKind {
    /// The complete keyword as written in a comment.
    pub fn keyword(self) -> &'static str {
        match self {
            DirectiveKind::Disable => return "monochromatic-lint-disable",
            DirectiveKind::Enable => return "monochromatic-lint-enable",
            DirectiveKind::DisableLine => return "monochromatic-lint-disable-line",
            DirectiveKind::DisableNextLine => return "monochromatic-lint-disable-next-line",
        }
    }

    /// Whether the kind covers one line instead of a range.
    pub fn is_line_scoped(self) -> bool {
        return matches!(
            self,
            DirectiveKind::DisableLine | DirectiveKind::DisableNextLine
        );
    }

    /// Match a whole word against the four keywords.
    fn from_keyword(word: &str) -> Option<DirectiveKind> {
        for kind in [
            DirectiveKind::Disable,
            DirectiveKind::Enable,
            DirectiveKind::DisableLine,
            DirectiveKind::DisableNextLine,
        ] {
            if kind.keyword() == word {
                return Some(kind);
            }
        }
        return None;
    }
}

/// What:
///  One rule identifier as written,
///  with its place in the host source.
/// Why:
///  A finding about one identifier underlines that identifier,
///  not the whole comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuleToken = { text: string; start: number; end: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleToken {
    /// The identifier with surrounding blanks removed.
    pub text: String,
    /// Host byte offset of its first byte.
    pub start: usize,
    /// Host byte offset after its last byte.
    pub end: usize,
}

/// What:
///  A well-formed directive before its identifiers are checked against the rule registry.
/// Why:
///  Parsing is text only;
///  which identifiers exist and which may be suppressed is decided later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParsedDirective = { kind: DirectiveKind; rules: RuleToken[]; justification?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedDirective {
    /// Which of the four keywords began the comment.
    pub kind: DirectiveKind,
    /// The listed identifiers;
    ///  empty means every suppressible rule.
    pub rules: Vec<RuleToken>,
    /// The reason after the dashes,
    ///  or `None` when the separator is absent or nothing follows it.
    pub justification: Option<String>,
}

/// What:
///  The outcome of reading one comment.
/// Why:
///  A comment that is not a directive is silently ignored,
///  while a broken one must be reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParsedComment = { kind: 'none' } | { kind: 'malformed'; reason: string } | { kind: 'directive'; directive };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParsedComment {
    /// Ordinary comment text.
    NotDirective,
    /// Starts like a directive but cannot be read as one;
    ///  the reason names what is wrong.
    Malformed(
        /// What is wrong with the comment,
        ///  phrased to follow "This comment cannot be read as a directive:".
        String,
    ),
    /// A readable directive.
    Directive(
        /// The parsed keyword,
        ///  identifiers and justification.
        ParsedDirective,
    ),
}

/// What:
///  Find the first run of two or more dashes that whitespace precedes and whitespace or the end follows.
/// Why:
///  This is ESLint's description separator;
///  dashes inside a word or a single dash are ordinary text.
///  Returns the byte range of the dash run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findSeparator(text: string): [number, number] | undefined;
/// ```
fn find_separator(text: &str) -> Option<(usize, usize)> {
    let mut previous_blank: bool = false;
    let mut run_start: usize = 0;
    let mut run_length: usize = 0;
    let mut run_after_blank: bool = false;
    for (index, character) in text.char_indices() {
        if character == '-' {
            if run_length == 0 {
                run_start = index;
                run_after_blank = previous_blank;
            }
            run_length += 1;
            previous_blank = false;
            continue;
        }
        if run_after_blank && run_length >= 2 && character.is_whitespace() {
            return Some((run_start, index));
        }
        run_length = 0;
        run_after_blank = false;
        previous_blank = character.is_whitespace();
    }
    if run_after_blank && run_length >= 2 {
        return Some((run_start, text.len()));
    }
    return None;
}

/// What:
///  Split the identifier list on commas and place every identifier in the host source.
/// Why:
///  An empty entry such as `a,,b` or a trailing comma is a typo that must not silently widen the directive.
///  Returns the text of the empty entry's problem on failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRules(text: string, textStart: number): RuleToken[] | string;
/// ```
fn parse_rules(text: &str, text_start: usize) -> Result<Vec<RuleToken>, String> {
    let mut tokens: Vec<RuleToken> = Vec::<RuleToken>::new();
    if text.trim().is_empty() {
        return Ok(tokens);
    }
    let mut piece_start: usize = text_start;
    for piece in text.split(',') {
        let trimmed: &str = piece.trim();
        if trimmed.is_empty() {
            return Err(String::from(
                "the rule list has an empty entry; separate identifiers with single commas",
            ));
        }
        let leading: usize = piece.len() - piece.trim_start().len();
        tokens.push(RuleToken {
            text: String::from(trimmed),
            start: piece_start + leading,
            end: piece_start + leading + trimmed.len(),
        });
        piece_start += piece.len() + 1;
    }
    return Ok(tokens);
}

/// What:
///  Read the text between a comment's delimiters.
/// Why:
///  `content_start` is the host offset of `content`,
///  so identifier positions are host positions.
///  Leading blanks are ignored,
///  and only a comment whose first word starts with the prefix is considered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseComment(content: string, contentStart: number): ParsedComment;
/// ```
pub fn parse_comment(content: &str, content_start: usize) -> ParsedComment {
    let blanks: usize = content.len() - content.trim_start().len();
    let body: &str = &content[blanks..];
    if !body.starts_with(DIRECTIVE_PREFIX) {
        return ParsedComment::NotDirective;
    }
    let word_end: usize = body.find(char::is_whitespace).unwrap_or(body.len());
    let word: &str = &body[..word_end];
    let Some(kind): Option<DirectiveKind> = DirectiveKind::from_keyword(word) else {
        return ParsedComment::Malformed(format!(
            "`{word}` is not a directive; the keywords are monochromatic-lint-disable, \
             monochromatic-lint-enable, monochromatic-lint-disable-line and monochromatic-lint-disable-next-line"
        ));
    };
    let rest_start: usize = blanks + word_end;
    let rest: &str = &content[rest_start..];
    let mut rules_text: &str = rest;
    let mut justification: Option<String> = None;
    if let Some((dashes_start, dashes_end)) = find_separator(rest) {
        rules_text = &rest[..dashes_start];
        let reason: &str = rest[dashes_end..].trim();
        if !reason.is_empty() {
            justification = Some(String::from(reason));
        }
    }
    return match parse_rules(rules_text, content_start + rest_start) {
        Ok(rules) => ParsedComment::Directive(ParsedDirective {
            kind,
            rules,
            justification,
        }),
        Err(reason) => ParsedComment::Malformed(reason),
    };
}

/// Grammar controls stay outside release artifacts.
#[cfg(test)]
#[path = "directive_parse_tests.rs"]
mod tests;
