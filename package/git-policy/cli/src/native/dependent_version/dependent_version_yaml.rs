//! What:
//! Read a block YAML document into the three node kinds a mapping,
//! a sequence and a
//!       scalar need.
//! Why:
//! The publish set sits four levels down the generated registry configuration
//!      (`auth.oidc[].workloads[].packages`),
//! and a line scan cannot tell it from another
//!      `packages:` key above it.
//! Descent needs the nesting,
//! so this reader builds it
//!      from indentation alone:
//! a key line and the deeper lines below it are one
//!      mapping entry,
//! and a deeper `- ` list is one sequence.
//! Flow collections,
//! block scalars,
//! anchors,
//! aliases and
//!      tags are refused,
//! because the generator writes none of them and a reader that
//!      guessed at one would report a value it never read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const document = parseYamlDocument(text); // a Map/List tree of the block subset
//! ```

/// The line-level reader this file builds on.
use crate::dependent_version_yaml_lines::{
    YamlLine, YamlProblem, is_item, item_content, read_lines, read_scalar, split_key,
};

/// What:
/// How far the keys of an item's compact mapping sit from the item's dash:
/// the dash and
///       the space after it.
/// Why:
///  `- key: value` opens a mapping whose later keys line up under `key`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ITEM_KEY_INDENT = 2;
/// ```
const ITEM_KEY_INDENT: usize = 2;

/// What:
/// One node of the block structure.
/// Why:
///  Descent asks three questions:
/// which entry has this key,
/// which items does this
///       sequence hold,
/// and is this leaf a scalar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type YamlNode = { kind: 'scalar'; text: string } | { kind: 'sequence'; items: YamlNode[] } | { kind: 'mapping'; entries: [string, YamlNode][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum YamlNode {
    /// A leaf value,
    /// with its quotes and escapes read.
    Scalar(String),
    /// A block of `- ` items.
    Sequence(Vec<YamlNode>),
    /// A block of `key: value` entries,
    /// in document order.
    Mapping(Vec<(String, YamlNode)>),
}

/// What:
/// The three questions descent asks,
/// answered by kind.
/// Why:
///  A node of another kind answers "no" rather than panicking,
/// so a wrong shape
///       becomes a diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// node.entry(key); node.items(); node.text();
/// ```
impl YamlNode {
    /// The value of the entry named `key`,
    /// and nothing when this node is not a mapping
    /// or
    /// names no such key.
    pub fn entry(&self, key: &str) -> Option<&YamlNode> {
        let YamlNode::Mapping(entries) = self else {
            return None;
        };
        for entry in entries {
            if entry.0 == key {
                return Some(&entry.1);
            }
        }
        return None;
    }

    /// The items of a sequence,
    /// and nothing when this node is not one.
    pub fn items(&self) -> Option<&[YamlNode]> {
        let YamlNode::Sequence(items) = self else {
            return None;
        };
        return Some(items.as_slice());
    }

    /// The text of a scalar,
    /// and nothing when this node is not one.
    pub fn text(&self) -> Option<&str> {
        let YamlNode::Scalar(text) = self else {
            return None;
        };
        return Some(text.as_str());
    }
}

/// What:
/// The document one text holds.
/// Why:
///  The registry configuration is one block document;
/// text this reader cannot read
///       is a problem naming its line,
/// never an empty document.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseYamlDocument(text: string): YamlNode;
/// ```
pub fn parse_yaml_document(text: &str) -> Result<YamlNode, YamlProblem> {
    let lines: Vec<YamlLine<'_>> = read_lines(text)?;
    let Some(first) = lines.first() else {
        return Ok(YamlNode::Mapping(Vec::new()));
    };
    if first.indent != 0 {
        return Err(YamlProblem::new(
            first.number,
            "starts the document indented, where no block can open",
        ));
    }
    let (node, at): (YamlNode, usize) = parse_block(&lines, 0)?;
    if at < lines.len() {
        return Err(YamlProblem::new(
            lines[at].number,
            "is indented deeper than the block it follows",
        ));
    }
    return Ok(node);
}

/// What:
/// The block that starts at `index`,
/// and where the next block starts.
/// Why:
///  A dash opens a sequence;
/// anything else opens a mapping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseBlock(lines, index): [YamlNode, number];
/// ```
fn parse_block(lines: &[YamlLine<'_>], index: usize) -> Result<(YamlNode, usize), YamlProblem> {
    let line: &YamlLine<'_> = &lines[index];
    if is_item(line.content) {
        return parse_sequence(lines, index);
    }
    return parse_mapping(lines, index, line.indent, None, line.number);
}

/// What:
/// The sequence whose first item is at `index`,
/// and the index after its last line.
/// Why:
///  An item is a scalar,
/// a compact mapping,
/// or a block on the lines below the dash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseSequence(lines, index): [YamlNode, number];
/// ```
fn parse_sequence(lines: &[YamlLine<'_>], index: usize) -> Result<(YamlNode, usize), YamlProblem> {
    let indent: usize = lines[index].indent;
    let mut items: Vec<YamlNode> = Vec::new();
    let mut at: usize = index;
    while at < lines.len() && lines[at].indent == indent && is_item(lines[at].content) {
        let line: &YamlLine<'_> = &lines[at];
        let rest: &str = item_content(line.content);
        at += 1;
        if rest.is_empty() {
            if at < lines.len() && lines[at].indent > indent {
                let (node, next): (YamlNode, usize) = parse_block(lines, at)?;
                items.push(node);
                at = next;
            } else {
                items.push(YamlNode::Scalar(String::new()));
            }
            continue;
        }
        if is_item(rest) {
            return Err(YamlProblem::new(
                line.number,
                "opens a nested list on one line, which this reader does not read",
            ));
        }
        let opens_mapping: bool = match split_key(rest) {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(reason) => return Err(YamlProblem::new(line.number, &reason)),
        };
        if opens_mapping {
            let (node, next): (YamlNode, usize) = parse_mapping(
                lines,
                at,
                indent + ITEM_KEY_INDENT,
                Some(rest),
                line.number,
            )?;
            items.push(node);
            at = next;
            continue;
        }
        let scalar: String = match read_scalar(rest) {
            Ok(read) => read,
            Err(reason) => return Err(YamlProblem::new(line.number, &reason)),
        };
        items.push(YamlNode::Scalar(scalar));
    }
    if at < lines.len() && lines[at].indent > indent {
        return Err(YamlProblem::new(
            lines[at].number,
            "is indented deeper than the block it follows",
        ));
    }
    return Ok((YamlNode::Sequence(items), at));
}

/// What:
/// Whether a mapping already holds an entry named `key`.
/// Why:
///  A repeated key would give one name two values,
/// and the diagnostic has to name it
///       rather than keep the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.some(([name]) => name === key);
/// ```
fn already_holds(entries: &[(String, YamlNode)], key: &str) -> bool {
    for entry in entries {
        if entry.0 == key {
            return true;
        }
    }
    return false;
}

/// What:
/// The mapping whose first entry is `content` at `indent`,
/// or the one that starts at
///       `index`,
/// and the index after its last line.
/// Why:
///  An entry's children are the deeper lines below it;
/// a repeated key of one
///       mapping is refused,
/// because two values for one name would be a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMapping(lines, index, indent, first, number): [YamlNode, number];
/// ```
fn parse_mapping(
    lines: &[YamlLine<'_>],
    index: usize,
    indent: usize,
    first: Option<&str>,
    first_number: usize,
) -> Result<(YamlNode, usize), YamlProblem> {
    let mut entries: Vec<(String, YamlNode)> = Vec::new();
    let mut at: usize = index;
    let mut pending: Option<(&str, usize)> = None;
    if let Some(content) = first {
        pending = Some((content, first_number));
    }
    loop {
        let (content, number): (&str, usize) = match pending.take() {
            Some(found) => found,
            None => {
                if at >= lines.len() || lines[at].indent != indent {
                    break;
                }
                let line: &YamlLine<'_> = &lines[at];
                at += 1;
                (line.content, line.number)
            }
        };
        if is_item(content) {
            return Err(YamlProblem::new(
                number,
                "opens a list item where a key was expected",
            ));
        }
        let (key, rest) = match split_key(content) {
            Ok(Some(found)) => found,
            Ok(None) => return Err(YamlProblem::new(number, "holds no key and colon")),
            Err(reason) => return Err(YamlProblem::new(number, &reason)),
        };
        if already_holds(&entries, key.as_str()) {
            return Err(YamlProblem::new(
                number,
                &format!("names the key {key:?} a second time"),
            ));
        }
        let value: YamlNode = if rest.is_empty() || rest.starts_with('#') {
            if at < lines.len() && lines[at].indent > indent {
                let (node, next): (YamlNode, usize) = parse_block(lines, at)?;
                at = next;
                node
            } else {
                YamlNode::Scalar(String::new())
            }
        } else {
            let scalar: String = match read_scalar(rest) {
                Ok(read) => read,
                Err(reason) => return Err(YamlProblem::new(number, &reason)),
            };
            YamlNode::Scalar(scalar)
        };
        entries.push((key, value));
    }
    if at < lines.len() && lines[at].indent > indent {
        return Err(YamlProblem::new(
            lines[at].number,
            "is indented deeper than the block it follows",
        ));
    }
    return Ok((YamlNode::Mapping(entries), at));
}

/// The block reader,
/// over the forms the generator writes and the ones it refuses.
#[cfg(test)]
#[path = "dependent_version_yaml_tests.rs"]
mod tests;
