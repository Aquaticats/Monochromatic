# monochromatic-xml-edit

Formatting-preserving XML parse and edit for Rust.

This crate parses an XML document into a span-annotated element arena,
and edits it by splicing replacement text into the recorded byte ranges.
Everything outside an edit keeps its exact bytes:
comments,
indentation,
line endings,
quote style,
attribute order and a leading byte order mark.
Malformed XML,
and XML outside the supported grammar,
is rejected with a positioned diagnostic instead of being guessed at.

Its first consumer is the Rust file-enforcer rule that writes the JetBrains Harper LSP4IJ policy into
`LanguageServersSettings.xml` and `UserDefinedLanguageServerSettings.xml`.
Those files embed JSON in attribute values.
This crate exposes the decoded attribute value so the caller can edit the JSON with `serde_json`,
and writes the new JSON back with correct attribute escaping.
The crate itself knows nothing about JSON.

## Model

- `parse_xml` is the only way to build an `XmlDocument`,
   so holding one proves the text passed every well-formedness check.
- Elements are addressed by `XmlElementId`,
   a plain number in document order that borrows nothing.
   A handle is only meaningful for the document that issued it;
   an edit returns a new document and leaves the old one usable.
- Every edit is a single splice followed by a re-parse of the result.
   An edit can therefore never return text this crate would reject.
- New elements are described as data with `XmlElementSpec` and rendered by the crate's emitter,
   which validates names,
   escapes values,
   and copies the host document's line ending and indentation unit.
   Callers never concatenate markup.

## Supported grammar

Accepted:

- one root element,
   with optional comments,
   processing instructions and whitespace around it
- an optional UTF-8 byte order mark at the very start
- an optional XML declaration with `version="1.0"`,
   an optional `encoding` of UTF-8 and an optional `standalone`
- start,
   end and empty-element tags,
   with single- or double-quoted attributes
- comments,
   CDATA sections,
   processing instructions,
   the five predefined entities and numeric character references
- namespace prefixes as part of a name (they are not resolved)

Rejected with a diagnostic:

- `DOCTYPE` and any other markup declaration,
   so no internal subset and no user-defined entities
- XML 1.1,
   and any declared encoding other than UTF-8
- a mismatched,
   missing or extra end tag,
   a second root element,
   and text outside the root
- duplicate attributes,
   attributes without whitespace between them,
   and an unquoted value
- a literal `<` in an attribute value,
   a bare `&`,
   an undefined entity,
   and a character reference to a code point XML 1.0 forbids
- `--` inside a comment,
   `]]>` in text,
   and unterminated comments,
   CDATA sections,
   processing instructions,
   tags and values
- characters XML 1.0 forbids anywhere,
   such as most control characters and U+FFFE

## Reading attribute values

Values follow XML 1.0 section 3.3.3:
a literal tab,
line feed or carriage return in a value reads as one space,
while a reference such as `&#10;` reads as the real character.
The encoder writes tab,
line feed and carriage return as numeric references,
so a value written by this crate reads back unchanged.
Libraries that write a literal tab or newline into an attribute do not round-trip,
which `doc/troubleshooting/xml-attribute-whitespace-serialization.md` records.

## Edits

- `set_attribute` replaces a value in place,
   keeping the quote style already used,
   or appends a new attribute after the last one.
- `remove_attribute` removes an attribute and the whitespace before it.
- `append_child` adds an element after the parent's last content,
   on its own line when the parent spans lines,
   expanding an empty or self-closing parent when needed.
   It refuses a parent that holds text.
- `replace_element` replaces a whole element,
   comments inside it included.
   Prefer attribute edits when comments inside a managed element must survive.
- `remove_element` removes an element and,
   when it owns its lines,
   its indentation and line break.
- `set_text` replaces the text of an element that has no child elements,
   comments or instructions.

```rust
//! Rewrite one option value of a settings file in place.
// example/set-policy.rs

/// Import the parser entry point.
use monochromatic_xml_edit::parse_xml;

let document = parse_xml(source)?;
let settings = document.descendants_named(document.root(), "LanguageServerDefinitionSettings")[0];
let option = document
    .find_child(settings, "option", "name", "configurationContent")
    .expect("option exists");
let json = document.attribute(option, "value").expect("value attribute");
// Edit `json` with serde_json, then write it back.
let edited = document.set_attribute(option, "value", &new_json)?;
std::fs::write(path, edited.source())?;
```

## Cost model

Parsing makes one linear pass over the source with an explicit stack of open elements,
so call-stack use does not grow with nesting depth.
Each construct is scanned once from its opening delimiter to its closing delimiter.
A reference scan looks at a fixed number of bytes ahead,
so a run of bare `&` characters does not turn quadratic.
Every edit copies the source once and re-parses it,
which is linear in the document size.

## Dependencies

None.
The standard library is the only dependency,
and the crate contains no `unsafe` block.
`doc/decision/file-enforcer-rust-rewrite.md` records the user's decision to write this editor in the repository
the way `monochromatic-jsonc-edit` was built,
with an owned scanner,
parser and emitter and no third-party parser.

## Status

Version 0.1.0.
The supported grammar is scoped to what the file-enforcer consumer needs plus well-formedness checking,
and anything outside it is refused rather than mis-edited.

## License

LGPL-3.0-or-later.
The license texts are in `LICENSES/`.
