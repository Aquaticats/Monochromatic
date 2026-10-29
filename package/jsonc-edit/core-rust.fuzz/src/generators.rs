//! What:
//!      Structured JSONC document generators driven by `arbitrary`.
//! Why:
//!       Fuzzing a parser with random bytes wastes nearly every execution on inputs that fail at
//!           the first character;
//!  generating documents that are valid,
//!  or valid except for one
//!           deliberate mutation,
//!  spends the budget on the interesting boundaries.

/// What:
///      Import the unstructured-input API the generators consume.
/// Why:
///       `Arbitrary` implementations must draw every choice from the fuzzer's byte budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arbitrary, Unstructured } from 'arbitrary';
/// ```
use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};

/// What:
///      Cap on generated container nesting for ordinary documents.
/// Why:
///       Documents stay small enough for fast execution while still exercising nesting;
///           the depth-envelope target builds exact depths separately through
///           `nested_document`.
const GEN_DEPTH_CAP: usize = 4;

/// What:
///      Line terminators the generator mixes between tokens.
/// Why:
///       Bare CR,
///  LF and CRLF each take a different branch in the scanner and in comment
///           ownership,
///  so all three must appear.
const TERMINATORS: [&str; 3] = ["\n", "\r\n", "\r"];

/// What:
///      Object key names,
///  emitted quoted.
/// Why:
///       A small closed set keeps duplicate keys out,
///  which the contract rejects,
///  while still
///           covering non-ASCII and digit-leading names.
const KEYS: [&str; 5] = ["a", "b", "key", "long-key-name", "ünïcøde"];

/// What:
///      Number token spellings,
///  emitted verbatim.
/// Why:
///       These cover the spelling-preservation and exact-identity contracts:
///           exponent forms,
///           trailing fraction zeros,
///           signed zero,
///           an integer past 2 to the 53,
///           and an exponent far beyond any binary64 range.
const NUMBERS: [&str; 9] = ["0", "-0", "1e0", "1.500", "9007199254740993", "1e999999999999999999999999", "-1.5e3", "123", "1E+2"];

/// What:
///      String values,
///  already quoted and escaped as they must appear in source.
/// Why:
///       They include an escaped lone high surrogate,
///           an escaped lone low surrogate,
///           a swapped pair,
///           and escapes that must survive emission unchanged.
const STRINGS: [&str; 8] = [
    "\"plain\"",
    "\"\"",
    "\"esc\\n\"",
    "\"\\u0041\"",
    "\"\\uD800\"",
    "\"\\uDC00\\uD800\"",
    "\"tab\\there\"",
    "\"quote\\\"inside ünïcøde\"",
];

/// What:
///      Bodies for generated line comments.
/// Why:
///       An empty body and a `region` body both take distinct paths in comment merging.
const LINE_TEXTS: [&str; 4] = [" note", " x", "region foo", ""];

/// What:
///      Bodies for generated block comments.
/// Why:
///       A multi-line body is the case that forces leading placement during emission,
///           so the generator must produce it.
const BLOCK_TEXTS: [&str; 3] = [" inner ", " why ", "multi\nline"];

/// What:
///      One generated JSONC document,
///  with the facts a target needs to assert against it.
/// Why:
///       Targets check comment preservation and the depth bound,
///           and recomputing either from the source would duplicate the implementation under test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GeneratedDocument = { source: string; commentTexts: string[]; depth: number };
/// ```
#[derive(Debug, Clone)]
pub struct GeneratedDocument {
    /// The document source,
    ///  which is always a container root and always parses.
    pub source: String,
    /// Every comment body in the source,
    ///  in the order the generator wrote it.
    pub comment_texts: Vec<String>,
    /// Container nesting depth the generator reached,
    ///  counted as open containers.
    pub depth: usize,
}

/// What:
///      Pick a key name the document has not used yet.
/// Why:
///       Duplicate keys are rejected by contract,
///  so the generator must never emit one.
///  The retry
///           count is bounded because `Unstructured::choose` returns the first choice rather than an
///           error once the byte budget is empty:
///  an unbounded retry on exhausted input would spin
///           forever,
///  which is exactly the hang this helper replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function freshKey(u: Unstructured, used: string[]): string;
/// ```
fn fresh_key<'b>(u: &mut Unstructured<'_>, used: &[&'b str]) -> ArbitraryResult<&'b str> {
    for _ in 0..KEYS.len() {
        let candidate = u.choose(&KEYS)?;
        if !used.contains(candidate) {
            return Ok(*candidate);
        }
    }
    // Unlucky draws, or an exhausted budget that keeps returning the first key: fall back to the
    // first unused name. KEYS is larger than the widest generated container, so one always exists.
    for candidate in KEYS.iter() {
        if !used.contains(candidate) {
            return Ok(*candidate);
        }
    }
    return Err(arbitrary::Error::NotEnoughData);
}

/// What:
///      Accumulate one generated document.
/// Why:
///       Emission,
///  comment bookkeeping and depth tracking have to stay in step,
///           so one type owns all three instead of passing three mutable arguments around.
struct Builder {
    /// Source written so far.
    out: String,
    /// Comment bodies written so far,
    ///  in order.
    comments: Vec<String>,
    /// Deepest open-container count reached.
    max_depth: usize,
}

/// What:
///      Generation methods for one document.
/// Why:
///       An inherent impl keeps the source buffer,
///  comment log and depth counter in step.
impl Builder {
    /// What:
    ///      Write one whitespace run.
    /// Why:
    ///       Spacing varies so emission is never accidentally identical to input formatting.
    fn whitespace(&mut self, u: &mut Unstructured<'_>) -> ArbitraryResult<()> {
        let count = u.int_in_range(0..=2)?;
        for _ in 0..count {
            self.out.push(if u.arbitrary::<bool>()? { ' ' } else { '\t' });
        }
        return Ok(());
    }

    /// What:
    ///      Write one terminator,
    ///  optionally preceded by whitespace.
    /// Why:
    ///       Comment placement and ownership depend on whether a token ends a line.
    fn terminator(&mut self, u: &mut Unstructured<'_>) -> ArbitraryResult<()> {
        self.whitespace(u)?;
        let chosen = u.choose(&TERMINATORS)?;
        self.out.push_str(chosen);
        return Ok(());
    }

    /// What:
    ///      Maybe write one comment of either style,
    ///  recording its body.
    /// Why:
    ///       Comments are the product's distinctive payload,
    ///           so they must appear often and in both styles.
    fn maybe_comment(&mut self, u: &mut Unstructured<'_>) -> ArbitraryResult<()> {
        if !u.arbitrary::<bool>()? {
            return Ok(());
        }
        if u.arbitrary::<bool>()? {
            let text = u.choose(&LINE_TEXTS)?;
            self.out.push_str("//");
            self.out.push_str(text);
            self.comments.push((*text).to_string());
            self.terminator(u)?;
            return Ok(());
        }
        let text = u.choose(&BLOCK_TEXTS)?;
        self.out.push_str("/*");
        self.out.push_str(text);
        self.out.push_str("*/");
        self.comments.push((*text).to_string());
        return self.whitespace(u);
    }

    /// What:
    ///      Write one scalar value.
    /// Why:
    ///       Scalars are the leaves every container shape needs.
    fn scalar(&mut self, u: &mut Unstructured<'_>) -> ArbitraryResult<()> {
        let choice = u.int_in_range(0..=3)?;
        if choice == 0 {
            self.out.push_str("null");
        } else if choice == 1 {
            self.out.push_str(if u.arbitrary::<bool>()? { "true" } else { "false" });
        } else if choice == 2 {
            let number = u.choose(&NUMBERS)?;
            self.out.push_str(number);
        } else {
            let text = u.choose(&STRINGS)?;
            self.out.push_str(text);
        }
        return Ok(());
    }

    /// What:
    ///      Write one value,
    ///  recursing into containers while the depth budget allows.
    /// Why:
    ///       Nesting is where the depth bound and the iterative spine live,
    ///           so generated documents must contain it.
    fn value(&mut self, u: &mut Unstructured<'_>, depth: usize) -> ArbitraryResult<()> {
        self.maybe_comment(u)?;
        let want_container = depth < GEN_DEPTH_CAP && u.arbitrary::<bool>()?;
        if !want_container {
            return self.scalar(u);
        }
        self.max_depth = self.max_depth.max(depth + 1);
        let is_record = u.arbitrary::<bool>()?;
        let member_count = u.int_in_range(0..=3)?;
        self.out.push(if is_record { '{' } else { '[' });
        let mut used_keys: Vec<&str> = Vec::new();
        for index in 0..member_count {
            if index > 0 {
                self.out.push(',');
                self.maybe_comment(u)?;
            }
            if is_record {
                // Duplicate keys are rejected by contract, so the generator never emits one.
                let key = fresh_key(u, &used_keys)?;
                used_keys.push(key);
                self.maybe_comment(u)?;
                self.out.push('"');
                self.out.push_str(key);
                self.out.push_str("\":");
                self.whitespace(u)?;
            }
            self.value(u, depth + 1)?;
        }
        // A trailing comma is accepted JSONC, so emit one often.
        if member_count > 0 && u.arbitrary::<bool>()? {
            self.out.push(',');
            self.maybe_comment(u)?;
        }
        self.out.push(if is_record { '}' } else { ']' });
        return Ok(());
    }
}

/// What:
///      The `Arbitrary` implementation libFuzzer draws documents through.
/// Why:
///       Structured generation is what makes the byte budget buy coverage instead of noise.
impl<'a> Arbitrary<'a> for GeneratedDocument {
    /// What:
    ///      Draw one document from the fuzzer's byte budget.
    /// Why:
    ///       `Arbitrary` is how libFuzzer turns coverage feedback into structured input.
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let mut builder = Builder { out: String::new(), comments: Vec::new(), max_depth: 0 };
        // The contract requires a container root, so the root never starts as a scalar.
        builder.max_depth = 1;
        builder.out.push('{');
        let member_count = u.int_in_range(0..=3)?;
        let mut used_keys: Vec<&str> = Vec::new();
        for index in 0..member_count {
            if index > 0 {
                builder.out.push(',');
                builder.maybe_comment(u)?;
            }
            let key = fresh_key(u, &used_keys)?;
            used_keys.push(key);
            builder.maybe_comment(u)?;
            builder.out.push('"');
            builder.out.push_str(key);
            builder.out.push_str("\":");
            builder.whitespace(u)?;
            builder.value(u, 1)?;
        }
        if member_count > 0 && u.arbitrary::<bool>()? {
            builder.out.push(',');
            builder.maybe_comment(u)?;
        }
        builder.out.push('}');
        if u.arbitrary::<bool>()? {
            builder.maybe_comment(u)?;
        }
        return Ok(GeneratedDocument {
            source: builder.out,
            comment_texts: builder.comments,
            depth: builder.max_depth,
        });
    }
}

/// What:
///      Build a document nested to exactly `depth` open containers.
/// Why:
///       The depth envelope is a boundary,
///           and boundaries need exact inputs on both sides rather than generated approximations.
///
/// # Arguments
///
/// * `depth` - Open containers to nest,
///    where the innermost holds `0`.
/// * `malformed` - When true,
///    omit every closing bracket so the input is both over depth and
///   syntactically incomplete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nestedDocument(depth: number, malformed: boolean): string;
/// ```
pub fn nested_document(depth: usize, malformed: bool) -> String {
    let mut out = "[".repeat(depth);
    out.push('0');
    if !malformed {
        out.push_str(&"]".repeat(depth));
    }
    return out;
}

/// What:
///      Return one deliberate mutation of a generated document.
/// Why:
///       Invalid input that is one edit away from valid reaches the parser's recovery and
///           rejection paths far more often than unrelated bytes do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function mutate(source: string, u: Unstructured): string;
/// ```
pub fn mutated(source: &str, u: &mut Unstructured<'_>) -> ArbitraryResult<String> {
    if source.is_empty() {
        return Ok(String::new());
    }
    let mut bytes: Vec<u8> = source.as_bytes().to_vec();
    let position = u.int_in_range(0..=(bytes.len() - 1))?;
    let action = u.int_in_range(0..=2)?;
    if action == 0 {
        bytes.remove(position);
    } else if action == 1 {
        bytes.insert(position, b',');
    } else {
        bytes[position] = u.int_in_range(b' '..=b'~')?;
    }
    // A byte-level mutation can break UTF-8, and the parser takes &str, so fall back to the
    // unmutated source rather than panicking in the harness.
    return Ok(String::from_utf8(bytes).unwrap_or_else(|_| return source.to_string()));
}

/// What:
///      Import the crate's model and number identity for building replacement values.
/// Why:
///       The edit target needs a value to write,
///  and its spelling must come from the same
///           difficult set the document generator uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncKind, JsoncNumberIdentity, JsoncValue } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{JsoncKind, JsoncNumberIdentity, JsoncValue};

/// What:
///      Build one replacement value the edit target can write at an address.
/// Why:
///       Edits must cover every scalar kind,
///           including numbers whose spelling has to survive and text holding an unpaired surrogate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replacementValue(u: Unstructured): JsoncValue;
/// ```
pub fn replacement_value(u: &mut Unstructured<'_>) -> ArbitraryResult<JsoncValue> {
    let choice = u.int_in_range(0..=3)?;
    let kind = if choice == 0 {
        JsoncKind::Null
    } else if choice == 1 {
        JsoncKind::Boolean { value: u.arbitrary::<bool>()? }
    } else if choice == 2 {
        let token = u.choose(&NUMBERS)?;
        let identity = JsoncNumberIdentity::from_token(token)
            .unwrap_or_else(|_| return JsoncNumberIdentity::from_token("0").expect("zero is a valid token"));
        JsoncKind::Number { raw: (*token).to_string(), identity }
    } else {
        let spelling = u.choose(&STRINGS)?;
        // A decode failure here would mean a generated spelling is not the valid JSON string it
        // claims to be, so fall back to an empty string rather than panicking inside the harness.
        let units = monochromatic_jsonc_edit::decode_quoted(spelling).unwrap_or_default();
        JsoncKind::Text { units, raw: (*spelling).to_string() }
    };
    return Ok(JsoncValue { kind, comment: None });
}
