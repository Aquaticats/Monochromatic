//! What:
//!  Structured XML document generators and a single-mutation helper,
//! driven by `arbitrary`.
//! Why:
//!  Random bytes almost never form well-formed XML,
//!  so a byte-only campaign would fail at the first character.
//!  Generating valid documents (and documents one deliberate mutation away from valid)
//!  spends the budget on the grammar's boundaries:
//!  quote styles,
//!  references,
//!  CDATA,
//!  comments holding markup,
//!  and every line ending.

/// What:
/// Import the unstructured-input API the generators consume.
/// Why:
/// Every choice must come from the fuzzer's byte budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arbitrary, Unstructured } from 'arbitrary';
/// ```
use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};

/// Deepest element nesting the generator builds;
/// the parser's own stack is iterative.
const GEN_DEPTH_CAP: usize = 4;

/// Line terminators mixed into generated layout;
/// each takes a different branch in the scanner.
const TERMINATORS: [&str; 3] = ["\n", "\r\n", "\r"];

/// One indentation step,
/// including none and a tab.
const UNITS: [&str; 3] = ["  ", "\t", ""];

/// Element names,
/// including a prefixed name and a non-ASCII name.
const NAMES: [&str; 6] = ["a", "b", "entry", "ns:item", "x-y.z", "\u{e9}t\u{e9}"];

/// Attribute names;
/// attribute number `n` of a tag uses entry `n`,
/// so a tag never repeats a name.
const ATTRIBUTE_NAMES: [&str; 4] = ["k", "name", "value", "xml:lang"];

/// Raw attribute values with their quotes,
/// exactly as they appear in source.
const VALUES: [&str; 14] = [
    "\"\"",
    "\"plain\"",
    "'single'",
    "\"it's\"",
    "'say \"hi\"'",
    "\"&quot;q&quot;\"",
    "\"a&#10;b\"",
    "\"tab&#9;x\"",
    "\"&amp;&lt;&gt;\"",
    "\"/>\"",
    "\">\"",
    "\"a\tb\"",
    "\"x\ny\"",
    "\"\u{e9}\"",
];

/// Content pieces that are not elements:
/// comments holding markup,
/// CDATA,
/// text,
/// an instruction.
const PIECES: [&str; 8] = [
    "<!-- c -->",
    "<!-- <entry key=\"x\"> -->",
    "<!---->",
    "<![CDATA[</a><b/>]]>",
    "<![CDATA[]]>",
    "text &amp; more",
    " ",
    "<?pi data?>",
];

/// Tokens the mutation helper splices into a document to reach rejection paths.
const TOKENS: [&str; 22] = [
    "<", ">", "</", "/>", "&", "&amp;", "&#", ";", "\"", "'", "<!--", "-->", "--", "<![CDATA[", "]]>", "<?", "?>",
    "<!DOCTYPE", "=", "\r", "\u{0}", "\u{FEFF}",
];

/// What:
/// A generated well-formed XML source.
/// Why:
/// Targets receive documents the parser must accept,
/// then attack them with edits and mutations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GeneratedDocument = { source: string };
/// ```
#[derive(Clone, Debug)]
pub struct GeneratedDocument {
    /// The generated source text.
    pub source: String,
}

/// What:
/// Append one generated element,
/// with its attributes and content,
/// at the given nesting level.
/// Why:
/// The depth cap bounds the recursion,
/// so this walk cannot exhaust the stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pushElement(u: Unstructured, out: string[], level: number, eol: string, unit: string): void;
/// ```
fn push_element(
    unstructured: &mut Unstructured<'_>,
    output: &mut String,
    level: usize,
    layout: (&str, &str),
) -> ArbitraryResult<()> {
    let (eol, unit): (&str, &str) = layout;
    let name: &str = *unstructured.choose(&NAMES)?;
    output.push('<');
    output.push_str(name);
    let attribute_count: usize = unstructured.int_in_range(0..=ATTRIBUTE_NAMES.len())?;
    for attribute_name in ATTRIBUTE_NAMES.iter().take(attribute_count) {
        output.push(' ');
        output.push_str(attribute_name);
        output.push_str(*unstructured.choose(&["=", " = "])?);
        output.push_str(*unstructured.choose(&VALUES)?);
    }
    let kind: u8 = unstructured.int_in_range(0..=3)?;
    if kind == 0 || (kind == 3 && level >= GEN_DEPTH_CAP) {
        output.push_str(*unstructured.choose(&["/>", " />"])?);
        return Ok(());
    }
    output.push('>');
    if kind == 3 {
        let child_count: usize = unstructured.int_in_range(1..=3)?;
        for _ in 0..child_count {
            output.push_str(eol);
            output.push_str(&unit.repeat(level + 1));
            if unstructured.ratio(2, 3)? {
                push_element(unstructured, output, level + 1, layout)?;
            } else {
                output.push_str(*unstructured.choose(&PIECES)?);
            }
        }
        output.push_str(eol);
        output.push_str(&unit.repeat(level));
    } else if kind == 2 {
        output.push_str(*unstructured.choose(&PIECES)?);
    }
    output.push_str("</");
    output.push_str(name);
    output.push('>');
    return Ok(());
}

/// What:
/// Build a whole document:
/// optional BOM,
/// declaration and comments around one generated root.
/// Why:
/// The prolog and epilog paths are part of the grammar and need the same coverage as the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generateDocument(u: Unstructured): string;
/// ```
impl Arbitrary<'_> for GeneratedDocument {
    /// What:
    /// Draw one document from the fuzzer's bytes.
    /// Why:
    /// This is the `Arbitrary` entry point targets call.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static arbitrary(u: Unstructured): GeneratedDocument;
    /// ```
    fn arbitrary(unstructured: &mut Unstructured<'_>) -> ArbitraryResult<GeneratedDocument> {
        let eol: &str = *unstructured.choose(&TERMINATORS)?;
        let unit: &str = *unstructured.choose(&UNITS)?;
        let mut source: String = String::new();
        if unstructured.ratio(1, 4)? {
            source.push('\u{FEFF}');
        }
        if unstructured.ratio(1, 3)? {
            source.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
            source.push_str(eol);
        }
        if unstructured.ratio(1, 3)? {
            source.push_str("<!-- prolog -->");
            source.push_str(eol);
        }
        push_element(unstructured, &mut source, 0, (eol, unit))?;
        if unstructured.ratio(1, 3)? {
            source.push_str(eol);
            source.push_str("<!-- epilog -->");
        }
        if unstructured.ratio(1, 2)? {
            source.push_str(eol);
        }
        return Ok(GeneratedDocument { source });
    }
}

/// What:
/// Round a byte offset up to the next UTF-8 character boundary of `text`.
/// Why:
/// Mutations splice strings,
/// and splicing inside a multi-byte character would panic the harness itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function boundary(text: string, offset: number): number;
/// ```
fn boundary(text: &str, offset: usize) -> usize {
    let mut index: usize = offset;
    while index < text.len() && !text.is_char_boundary(index) {
        index += 1;
    }
    return index;
}

/// What:
/// Apply exactly one deliberate mutation to `source`:
/// delete,
/// insert,
/// truncate or replace.
/// Why:
/// Near-valid input reaches the rejection paths that unrelated bytes almost never do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mutated(source: string, u: Unstructured): string;
/// ```
pub fn mutated(source: &str, unstructured: &mut Unstructured<'_>) -> ArbitraryResult<String> {
    let start: usize = boundary(source, unstructured.int_in_range(0..=source.len())?);
    let length: usize = unstructured.int_in_range(1..=8)?;
    let end: usize = boundary(source, std::cmp::min(source.len(), start + length));
    let token: &str = *unstructured.choose(&TOKENS)?;
    let kind: u8 = unstructured.int_in_range(0..=3)?;
    let mut result: String = String::with_capacity(source.len() + token.len());
    result.push_str(&source[..start]);
    if kind == 1 {
        result.push_str(token);
        result.push_str(&source[start..]);
    } else if kind == 3 {
        result.push_str(token);
        result.push_str(&source[end..]);
    } else if kind == 0 {
        result.push_str(&source[end..]);
    }
    return Ok(result);
}
