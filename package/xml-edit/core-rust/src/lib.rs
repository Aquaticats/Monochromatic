//! What:
//!  `monochromatic-xml-edit` parses XML into a span-annotated element arena and edits it by splicing text
//!  into the recorded byte ranges,
//! so everything outside an edit keeps its exact bytes.
//!  A Rust crate is closest to a TypeScript package:
//! private implementation files plus one public API
//!  surface re-exported here,
//! so callers write `monochromatic_xml_edit::parse_xml`.
//! Why:
//!  The file-enforcer rule that writes the JetBrains Harper LSP4IJ policy must change a few attributes in
//!  IDE settings XML without reformatting the rest or dropping comments,
//! and must refuse malformed XML.
//!  The user chose an owned parser over a third-party one,
//! the way `monochromatic-jsonc-edit` was built;
//!  this crate has no dependencies and no `unsafe` block.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { parseXml } from '@monochromatic-dev/xml-edit-core-rust';
//! const edited = parseXml(source).setAttribute(id, 'value', '{"a": 1}');
//! ```

/// What:
/// Character classes from the XML 1.0 grammar.
/// Why:
/// Validation and name checks share one definition.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: isXmlChar, isXmlName
/// ```
mod chars;

/// What:
/// Entity and character-reference encoding and decoding.
/// Why:
/// Attribute values must round-trip exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { decodeAttributeValue, encodeAttributeValue } from './coding';
/// ```
pub mod coding;

/// What:
/// The public failure types.
/// Why:
/// Parse positions and edit refusals answer different questions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './errors';
/// ```
pub mod error;

/// What:
/// The document model and handles.
/// Why:
/// Edits and queries are methods on one parsed document.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type { XmlDocument, XmlElementId, XmlSpan } from './document';
/// ```
pub mod document;

/// What:
/// The linear scanner for names,
/// comments,
/// CDATA and start tags.
/// Why:
/// One pass per construct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: the scanner the parser drives.
/// ```
mod scan;

/// What:
/// The XML declaration scanner.
/// Why:
/// Unsupported versions and encodings are refused up front.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: scanDeclaration
/// ```
mod declaration;

/// What:
/// The iterative parser.
/// Why:
/// Constant stack depth whatever the nesting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { parseXml } from './parse';
/// ```
mod parse;

/// What:
/// Read-only queries on a parsed document.
/// Why:
/// Rules find elements before editing them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // methods on XmlDocument: attribute, childrenNamed, findChild
/// ```
mod query;

/// What:
/// Line ending,
/// indentation and line-ownership probes.
/// Why:
/// Inserted text should match its neighbours.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: detectEol, lineIndent
/// ```
mod layout;

/// What:
/// The element description type and the iterative emitter.
/// Why:
/// New markup is data,
/// never string glue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { XmlElementSpec } from './render';
/// ```
mod render;

/// What:
/// Attribute edits (set,
/// remove).
/// Why:
/// The consumer's main operation,
/// with the smallest possible splice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // methods on XmlDocument: setAttribute, removeAttribute
/// ```
mod edit_attribute;

/// What:
/// Element edits (replace,
/// append,
/// remove,
/// set text).
/// Why:
/// Whole-entry management for the consumer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // methods on XmlDocument: replaceElement, appendChild, removeElement, setText
/// ```
mod edit_content;

/// What:
/// Re-export the element description types.
/// Why:
/// Callers build the elements they append or replace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { XmlElementSpec, XmlNodeSpec } from './render';
/// ```
pub use render::{XmlElementSpec, XmlNodeSpec};

/// What:
/// Re-export the parser entry point at the crate root.
/// Why:
/// It is the crate's front door.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { parseXml } from './parse';
/// ```
pub use parse::parse_xml;

/// What:
/// Re-export the failure types at the crate root.
/// Why:
/// Callers name them in signatures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { XmlParseError, XmlEditError } from './errors';
/// ```
pub use error::{XmlEditError, XmlParseError};

/// What:
/// Re-export the document types at the crate root.
/// Why:
/// Callers hold handles and spans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { XmlDocument, XmlElementId, XmlSpan } from './document';
/// ```
pub use document::{XmlDocument, XmlElementId, XmlSpan};

/// What:
/// Synthetic JetBrains-shaped fixtures.
/// Why:
/// Tests never touch the owner's real settings files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './fixtures';
/// ```
#[cfg(test)]
mod fixtures;

/// What:
/// Parser and query tests.
/// Why:
/// The grammar and span bookkeeping are the crate's foundation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './parse.test';
/// ```
#[cfg(test)]
mod parse_tests;

/// What:
/// Attribute edit tests.
/// Why:
/// Byte-for-byte preservation outside the edited value is the contract.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './editAttribute.test';
/// ```
#[cfg(test)]
mod edit_attribute_tests;

/// What:
/// Element edit tests.
/// Why:
/// Inserted text must match its neighbours' layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './editContent.test';
/// ```
#[cfg(test)]
mod edit_content_tests;

/// What:
/// Adversarial boundary tests.
/// Why:
/// Quotes,
/// entities,
/// CDATA,
/// comments,
/// BOM and size limits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './boundary.test';
/// ```
#[cfg(test)]
mod boundary_tests;

/// What:
/// Public-surface tests for classes,
/// layout,
/// builders and errors.
/// Why:
/// Every export needs coverage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './api.test';
/// ```
#[cfg(test)]
mod api_tests;

/// What:
/// End-to-end consumer flow on the JetBrains fixtures.
/// Why:
/// Proves the surface fits the real rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './consumer.test';
/// ```
#[cfg(test)]
mod consumer_tests;
