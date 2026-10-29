//! Bounded structured `Arbitrary` generators for the scanner's format-driven fuzz
//! targets.
//!
//! The engine swap (#383/#384/#385) reshaped the scanner's rule surface into the
//! two-form file format (`compile_from_text` in `rule/frx`):
//!  a line is a bare literal,
//! a `/PATTERN/FLAGS` regex whose trailing run is all ASCII-lowercase (with `m`/`x`
//! accepted as no-ops and every other letter a hard load error),
//!  a `#` comment,
//!  or a
//! blank.
//!  This module produces that shape plus a content buffer seeded from the rules'
//! own match bytes,
//!  so `fuzz_ruleset_scan_invariants` and `fuzz_scan_format` exercise
//! the strict loader and the columnless scan path on inputs that actually match instead
//! of rejecting almost every iteration.
//!
//! Bounds keep the search space small enough for coverage-guided fuzzing (an unbounded
//! `derive(Arbitrary)` would burn the byte budget on length and never reach scan
//! coverage):
//!  rules per file,
//!  literal/body byte widths,
//!  content lines,
//!  and total
//! content size are all capped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type RuleFileAndContent = { rules: RuleLine[]; content: Uint8Array };
//! // type RuleLine =
//! //   | { kind: "literal"; bytes: number[] }
//! //   | { kind: "regex"; body: number[]; flags: FlagRun }
//! //   | { kind: "comment"; bytes: number[] }
//! //   | { kind: "blank" };
//! ```

/// Imports the `Arbitrary` trait,
///  its `Result` alias,
///  and the byte-cursor `Unstructured`.
// What:     `use arbitrary::{Arbitrary, Result, Unstructured};` pulls the three names
//           every manual `Arbitrary` impl needs: the trait libFuzzer calls per input,
//           the crate's `Result<T> = Result<T, arbitrary::Error>` alias, and the cursor
//           over the fuzzer-supplied bytes.
// Why:      Manual impls (not derive) so recursion and lengths stay bounded.
use arbitrary::{Arbitrary, Result, Unstructured};

/// Imports the SHA-256 hasher for the redacted crash-fingerprint helper.
use sha2::{Digest, Sha256};

//region Bounds

/// Maximum rule lines in one generated file.
pub const MAX_RULES: usize = 6;

/// Maximum bytes in one bare-literal rule before rendering.
pub const MAX_LITERAL_BYTES: usize = 24;

/// Maximum bytes in one `/PATTERN/FLAGS` regex body.
pub const MAX_BODY_BYTES: usize = 16;

/// Maximum newline-delimited lines in one generated content buffer.
pub const MAX_CONTENT_LINES: usize = 24;

/// Maximum bytes in one generated content buffer.
pub const MAX_CONTENT_BYTES: usize = 4096;

//endregion Bounds

//region Flag run

/// The trailing flag run after the closing slash of a `/body/flags` regex line.
///
/// The strict loader accepts an empty run and `m`/`x` (both engine no-ops) and fails
/// closed on any other lowercase letter.
///  `Bad` carries one such letter so a target can
/// exercise the hard-error path;
///  the letter stays ASCII-lowercase so the line still
/// classifies as a regex (a non-lowercase trailing run would reclassify as a literal).
#[derive(Debug)]
pub enum FlagRun {
    /// No flags:
    ///  `/body/`.
    None,
    /// The multiline no-op:
    ///  `/body/m`.
    Multiline,
    /// The verbose no-op:
    ///  `/body/x`.
    Verbose,
    /// Both no-ops:
    ///  `/body/mx`.
    Both,
    /// One lowercase letter outside `{m, x}`:
    ///  a hard-error flag such as `i`.
    Bad(BadFlag),
}

/// One ASCII-lowercase flag letter the strict loader rejects.
///
/// A closed set drawn from letters that are neither `m` nor `x`,
///  so rendering one always
/// produces a genuine `UnsupportedFlag`.
///  `derive(Arbitrary)` is sound:
///  finite,
///  no
/// recursion,
///  one byte per value.
#[derive(Debug, Arbitrary)]
pub enum BadFlag {
    /// Case-insensitive:
    ///  silently dropping it would change semantics,
    ///  so it hard-errors.
    I,
    /// Dot-matches-newline.
    S,
    /// Unicode mode.
    U,
    /// Global.
    G,
    /// Ungreedy.
    Us,
}

/// Renders one flag letter for a [`BadFlag`].
impl BadFlag {
    /// Returns the ASCII-lowercase letter for this rejected flag.
    fn letter(&self) -> char {
        return match self {
            BadFlag::I => 'i',
            BadFlag::S => 's',
            BadFlag::U => 'u',
            BadFlag::G => 'g',
            BadFlag::Us => 'r',
        }
    }
}

/// Picks a flag run,
///  biased toward the accepted forms so most rulesets load.
impl<'a> Arbitrary<'a> for FlagRun {
    /// Reads one tag byte and maps it to a flag run;
    ///  five of eight land on accepted forms.
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let tag = u.int_in_range(0u8..=7)?;
        return Ok(match tag {
            0 | 1 => FlagRun::None,
            2 => FlagRun::Multiline,
            3 => FlagRun::Verbose,
            4 => FlagRun::Both,
            _ => FlagRun::Bad(BadFlag::arbitrary(u)?),
        })
    }
}

/// Reads a flag run for rendering and load-outcome prediction.
impl FlagRun {
    /// Appends the flag letters after a regex line's closing slash.
    fn render(&self, out: &mut String) {
        match self {
            FlagRun::None => {}
            FlagRun::Multiline => out.push('m'),
            FlagRun::Verbose => out.push('x'),
            FlagRun::Both => out.push_str("mx"),
            FlagRun::Bad(flag) => out.push(flag.letter()),
        }
    }

    /// Reports whether this run makes the strict loader fail closed.
    fn is_bad(&self) -> bool {
        return matches!(self, FlagRun::Bad(_))
    }
}

//endregion Flag run

//region Rule line

/// One line of the two-form rule file.
///
/// Each variant renders to exactly one line the loader classifies deterministically:
/// `Literal` to an escaped literal rule,
///  `Regex` to a `/body/flags` rule,
///  `Comment` to a
/// skipped `#` line,
///  `Blank` to a skipped empty line.
#[derive(Debug)]
pub enum RuleLine {
    /// A bare literal line whose bytes the loader escapes into the verbose dialect.
    Literal(SafeBytes),
    /// A `/body/flags` regex line.
    Regex {
        /// ASCII-alphanumeric body,
        ///  matched literally by the verbose-mode engine.
        body: AlnumBytes,
        /// Trailing flag run controlling the load outcome.
        flags: FlagRun,
    },
    /// A `#`-prefixed comment line the loader skips.
    Comment(SafeBytes),
    /// An empty line the loader skips.
    Blank,
}

/// Picks a rule-line shape,
///  biased toward literals and regexes that carry scan work.
impl<'a> Arbitrary<'a> for RuleLine {
    /// Reads one tag byte,
    ///  then the variant payload.
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let tag = u.int_in_range(0u8..=7)?;
        return Ok(match tag {
            0..=2 => RuleLine::Literal(SafeBytes::arbitrary(u)?),
            3..=5 => RuleLine::Regex {
                body: AlnumBytes::arbitrary(u)?,
                flags: FlagRun::arbitrary(u)?,
            },
            6 => RuleLine::Comment(SafeBytes::arbitrary(u)?),
            _ => RuleLine::Blank,
        })
    }
}

/// Renders a rule line and reports the bytes it can match.
impl RuleLine {
    /// Appends this line's rendered text (no trailing newline) to `out`.
    fn render(&self, out: &mut String) {
        match self {
            RuleLine::Literal(bytes) => bytes.render(out),
            RuleLine::Regex { body, flags } => {
                out.push('/');
                body.render(out);
                out.push('/');
                flags.render(out);
            }
            RuleLine::Comment(bytes) => {
                out.push('#');
                bytes.render(out);
            }
            RuleLine::Blank => {}
        }
    }

    /// Returns the bytes this line matches when it loads,
    ///  for seeding content.
    ///
    /// A literal matches its own bytes;
    ///  a regex matches its alphanumeric body verbatim
    /// (verbose mode leaves alphanumerics untouched);
    ///  a comment or blank matches nothing.
    /// A bad-flag regex never loads,
    ///  so it contributes no match bytes.
    fn match_bytes(&self) -> Option<Vec<u8>> {
        return match self {
            RuleLine::Literal(bytes) => Some(bytes.0.clone()),
            RuleLine::Regex { body, flags } if !flags.is_bad() => Some(body.0.clone()),
            _ => None,
        }
    }
}

//endregion Rule line

//region Byte alphabets

/// A non-empty ASCII byte run safe to render as a literal or comment body.
///
/// The alphabet is alphanumerics,
///  spaces,
///  and the escapable metacharacters,
///  so the
/// literal escaper has real work (escaping `.`/`#`/space/backslash and friends) while
/// the bytes stay valid UTF-8 (all ASCII) and free of `\n`/`\r` (which would split the
/// rendered rule across lines).
///  The first and last bytes are forced alphanumeric so a
/// literal never begins with `#` (comment),
///  `/` (regex),
///  or whitespace (trimmed away),
/// keeping its classification and its match bytes stable.
#[derive(Debug)]
pub struct SafeBytes(pub Vec<u8>);

/// A byte from the literal alphabet:
///  alphanumeric,
///  space,
///  or an escapable metacharacter.
fn safe_byte(u: &mut Unstructured<'_>) -> Result<u8> {
    /// Escapable metacharacters plus space,
    ///  mirroring the escaper's escape set.
    const METAS: &[u8] = b" .[](){}?|&~^$\\#-/*+";
    let pick = u.int_in_range(0u8..=3)?;
    return Ok(match pick {
        0 => b'a' + u.int_in_range(0u8..=25)?,
        1 => b'A' + u.int_in_range(0u8..=25)?,
        2 => b'0' + u.int_in_range(0u8..=9)?,
        _ => *u.choose(METAS)?,
    })
}

/// An ASCII alphanumeric byte,
///  forced so a literal's ends never reclassify the line.
fn alnum_byte(u: &mut Unstructured<'_>) -> Result<u8> {
    let pick = u.int_in_range(0u8..=2)?;
    return Ok(match pick {
        0 => b'a' + u.int_in_range(0u8..=25)?,
        1 => b'A' + u.int_in_range(0u8..=25)?,
        _ => b'0' + u.int_in_range(0u8..=9)?,
    })
}

/// Builds a `SafeBytes` with alphanumeric ends and a safe-alphabet interior.
impl<'a> Arbitrary<'a> for SafeBytes {
    /// Reads a bounded length,
    ///  then bytes,
    ///  forcing the first and last to be alphanumeric.
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let len = u.int_in_range(1usize..=MAX_LITERAL_BYTES)?;
        let mut bytes: Vec<u8> = Vec::with_capacity(len);
        for index in 0..len {
            // Alphanumeric ends keep a literal off the comment/regex/blank classification
            // and keep its trimmed form equal to its rendered form.
            let byte = if index == 0 || index == len - 1 {
                alnum_byte(u)?
            } else {
                safe_byte(u)?
            };
            bytes.push(byte);
        }
        return Ok(SafeBytes(bytes))
    }
}

/// Appends a `SafeBytes` value as UTF-8 text (its bytes are ASCII by construction).
impl SafeBytes {
    /// Writes the bytes into `out`;
    ///  every byte is ASCII,
    ///  so the string stays valid UTF-8.
    fn render(&self, out: &mut String) {
        for &byte in &self.0 {
            out.push(byte as char);
        }
    }
}

/// A non-empty ASCII-alphanumeric run used as a regex body.
///
/// Alphanumerics compile to a plain literal pattern that matches themselves,
///  so a regex
/// rule always compiles (never an empty-matchable or dialect error) and its match bytes
/// are predictable for content seeding.
#[derive(Debug)]
pub struct AlnumBytes(pub Vec<u8>);

/// Builds an `AlnumBytes` of bounded length.
impl<'a> Arbitrary<'a> for AlnumBytes {
    /// Reads a bounded length,
    ///  then that many alphanumeric bytes.
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let len = u.int_in_range(1usize..=MAX_BODY_BYTES)?;
        let mut bytes: Vec<u8> = Vec::with_capacity(len);
        for _ in 0..len {
            bytes.push(alnum_byte(u)?);
        }
        return Ok(AlnumBytes(bytes))
    }
}

/// Appends an `AlnumBytes` value as UTF-8 text.
impl AlnumBytes {
    /// Writes the bytes into `out`;
    ///  every byte is ASCII alphanumeric.
    fn render(&self, out: &mut String) {
        for &byte in &self.0 {
            out.push(byte as char);
        }
    }
}

//endregion Byte alphabets

//region Rule file

/// A bounded set of rule lines forming one two-form rule file.
#[derive(Debug)]
pub struct RuleFile {
    /// The lines in file order.
    pub lines: Vec<RuleLine>,
}

/// Builds a `RuleFile` of one to `MAX_RULES` lines.
impl<'a> Arbitrary<'a> for RuleFile {
    /// Reads a bounded line count,
    ///  then that many rule lines.
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let count = u.int_in_range(1usize..=MAX_RULES)?;
        let mut lines: Vec<RuleLine> = Vec::with_capacity(count);
        for _ in 0..count {
            lines.push(RuleLine::arbitrary(u)?);
        }
        return Ok(RuleFile { lines })
    }
}

/// Renders the file,
///  reports its load outcome,
///  and collects its match bytes.
impl RuleFile {
    /// Renders the lines into one newline-joined two-form source string.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for (index, line) in self.lines.iter().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            line.render(&mut out);
        }
        return out
    }

    /// Renders the same lines in reverse order (rule-order-invariance oracle).
    pub fn render_reversed(&self) -> String {
        let mut out = String::new();
        for (index, line) in self.lines.iter().rev().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            line.render(&mut out);
        }
        return out
    }

    /// Reports whether any regex line carries a flag the strict loader rejects.
    ///
    /// When true,
    ///  the loader fails closed regardless of line order,
    ///  so a target can
    /// predict a load error without re-implementing the loader.
    pub fn has_bad_flag(&self) -> bool {
        return self.lines.iter().any(|line| {
            return matches!(line, RuleLine::Regex { flags, .. } if flags.is_bad())
        })
    }

    /// Collects the bytes the loading rules can match,
    ///  for content seeding.
    pub fn match_literals(&self) -> Vec<Vec<u8>> {
        let mut out: Vec<Vec<u8>> = Vec::new();
        for line in &self.lines {
            if let Some(bytes) = line.match_bytes() {
                out.push(bytes);
            }
        }
        return out
    }
}

//endregion Rule file

//region Rule file plus content

/// A rule file paired with a content buffer seeded from the rules' match bytes.
///
/// The seeded content plants each rule's match bytes on their own lines amid random
/// filler and blank lines,
///  so the scan path finds real hits (exercising the columnless
/// `PATH:LINE rule=N` format) instead of scanning noise that never matches.
#[derive(Debug)]
pub struct RuleFileAndContent {
    /// The generated rule file.
    pub rules: RuleFile,
    /// Multi-line content seeded to match some rules.
    pub content: Vec<u8>,
}

/// Builds the pair:
///  rules first,
///  then content synthesized from their match bytes.
impl<'a> Arbitrary<'a> for RuleFileAndContent {
    /// Generates the rule file,
    ///  then seeds content from its collected match bytes.
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let rules = RuleFile::arbitrary(u)?;
        let literals = rules.match_literals();
        let content = synth_content(&literals, u)?;
        return Ok(RuleFileAndContent { rules, content })
    }
}

/// Synthesizes a multi-line content buffer biased toward the given match bytes.
///
/// Each line is one of:
///  a planted match literal (optionally with filler around it),
///  a
/// random filler line,
///  or an empty line.
///  Blank and comment-skipping paths in the scan
/// stay exercised by the empty lines.
///  A few post-hoc single-byte mutations perturb the
/// buffer to reach near-miss edges.
///  The result is capped at `MAX_CONTENT_BYTES`.
fn synth_content(literals: &[Vec<u8>], u: &mut Unstructured<'_>) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(256);
    let line_count = u.int_in_range(1usize..=MAX_CONTENT_LINES)?;
    for _ in 0..line_count {
        if out.len() >= MAX_CONTENT_BYTES {
            break;
        }
        let choice = u.int_in_range(0u8..=3)?;
        if choice <= 1 && !literals.is_empty() {
            // Plant a match literal, optionally padded with filler on each side.
            let lead = u.int_in_range(0usize..=4)?;
            push_filler(&mut out, lead, u)?;
            let literal = u.choose(literals)?;
            out.extend_from_slice(literal);
            let trail = u.int_in_range(0usize..=4)?;
            push_filler(&mut out, trail, u)?;
        } else if choice == 2 {
            // A random filler line: matches nothing on its own.
            let width = u.int_in_range(0usize..=32)?;
            push_filler(&mut out, width, u)?;
        }
        // choice == 3 (or the empty branches above) leaves an empty line.
        out.push(b'\n');
    }
    // A handful of single-byte mutations reach boundary and near-miss shapes.
    let mutations = u.int_in_range(0u8..=4)?;
    for _ in 0..mutations {
        if out.is_empty() {
            break;
        }
        let index = u.int_in_range(0usize..=(out.len() - 1))?;
        out[index] = u.int_in_range(0u8..=255)?;
    }
    out.truncate(MAX_CONTENT_BYTES);
    return Ok(out)
}

/// Appends `count` lowercase filler bytes,
///  stopping at the content cap.
fn push_filler(out: &mut Vec<u8>, count: usize, u: &mut Unstructured<'_>) -> Result<()> {
    for _ in 0..count {
        if out.len() >= MAX_CONTENT_BYTES {
            break;
        }
        out.push(b'a' + u.int_in_range(0u8..=25)?);
    }
    return Ok(())
}

//endregion Rule file plus content

//region Redacted fingerprint

/// Fingerprints content as a length and SHA-256 for a redacted crash message.
///
/// A fuzz crash must never paste raw content (it can carry secret-shaped bytes);
///  this
/// returns the reproducer shape the README prescribes,
///  a length plus lowercase hex
/// digest,
///  which uniquely identifies the input without echoing it.
pub fn redacted_fingerprint(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let digest = hasher.finalize();
    return format!("len={} sha256={:x}", content.len(), digest)
}

//endregion Redacted fingerprint

/// Registers the generator unit tests (sidecar;
///  the fuzz crate does not lint them).
#[cfg(test)]
#[path = "generators_tests.rs"]
mod tests;
