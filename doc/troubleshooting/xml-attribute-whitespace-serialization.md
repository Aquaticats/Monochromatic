# xot 0.31.2, xml-rs 1.4.0, xmltree 0.12.0: literal tab or newline in attribute values reparses as spaces

Found while probing Rust XML editors for the `meow` managed-file design
(`doc/planning/monorepo-manager-route-research/rust-structured-edits.md`,
 sections "XML" and "XML options").
No package in this repository depends on these crates today,
so there is no consumer-side guard to maintain;
the finding constrains which XML write path `meow` may use.

Status (2026-09-17):
the defect is live in the newest published release of each crate
and on each repository's default branch.
Upstream artifacts:
a fileable new-issue draft for `faassen/xot`,
an additive comment draft for the existing
[kornelski/xml-rs#88][xmlrs-88],
and nothing for `eminence/xmltree-rs`,
whose attribute escaping belongs to xml-rs.
Nothing was filed or posted.

## Symptom

A string attribute value that contains a tab (U+0009),
a newline (U+000A),
or a carriage return (U+000D)
is serialized with the character written literally between the quotes.
No error is raised.
The document is well-formed,
but every XML 1.0 reader applies attribute-value normalization on the next parse,
so the value silently comes back with spaces.
This happens on edited values and on untouched values that the source spelled as character references,
because a tree or event copy decodes `&#9;` to a tab before the serializer sees it.

Which characters each serializer writes literally:

- Literal tab variant:
   `xot` 0.31.2,
   `xml` 1.4.0 (xml-rs),
   and `xmltree` 0.12.0 all write a raw tab.
- Literal newline and carriage return variant:
   only `xot` 0.31.2.
   xml-rs and xmltree write `&#xA;` and `&#xD;`.

Probe output from the `meow` research harness,
case `x04` (`setOption` with value `new "cmd" & <x>` plus a tab and `tab`),
as JSON strings so the raw tab shows as `\t`:

```text
xot:     <option name=\"commandLine\" value=\"new &quot;cmd&quot; &amp; &lt;x>\ttab\"/>
xml-rs:  <option name=\"commandLine\" value=\"new &quot;cmd&quot; &amp; &lt;x&gt;\ttab\"></option>
xmltree: <option name=\"commandLine\" value=\"new &quot;cmd&quot; &amp; &lt;x&gt;\ttab\" />
```

Case `x05` (LSP4IJ `configurationContent` holding `JSON.stringify(value, null, 2)`):

```text
xot:     value=\"{\n  &quot;a&quot;: 1,\n  &quot;b&quot;: [\n    &quot;x&quot;\n  ]\n}\"/>
xml-rs:  value=\"{&#xA;  &quot;a&quot;: 1,&#xA;  &quot;b&quot;: [&#xA;    &quot;x&quot;&#xA;  ]&#xA;}\"></option>
xmltree: value=\"{&#xA;  &quot;a&quot;: 1,&#xA;  &quot;b&quot;: [&#xA;    &quot;x&quot;&#xA;  ]&#xA;}\" />
```

The minimal harness in "Verification" shows the read-back:
`xot` reads its own output of `a\tb\nc\rd` as `a b c d`,
and xml-rs and xmltree read theirs as `a b\nc\rd`;
`roxmltree` 0.21.1 reads each output the same way.

### Other rewrites the probe recorded (context, not part of this defect)

These made every tree or event serializer fail the byte-preservation cases,
but each is a documented choice or a lossy data model,
not an escaping error:

- `xot` rewrites `" />"` to `"/>"`,
   single-quoted attributes to double quotes,
   and `'` inside values to `&apos;`
   (source in "Root cause",
   "xot 0.31.2"),
   and rejects documents with a DOCTYPE ("DTDs are not supported.",
   `README.md:46`).
- `xmltree` drops whitespace-only text
   (`src/lib.rs:235` and `:313` discard `XmlEvent::Whitespace`)
   and stores attributes in `std::collections::HashMap` by default (`src/lib.rs:44-47`),
   which is randomly seeded per instance
   ("each `HashMap` instance uses a different seed",
   `library/std/src/collections/hash/map.rs:214` in the local nightly toolchain source),
   so attribute order changes between runs and repeated writes are not byte-identical.
   The `attribute-order` feature switches to `IndexMap` (`src/lib.rs:34`);
   the owner kept `HashMap` as the default in [eminence/xmltree-rs#21][xmltree-21].
- xml-rs event copies add `<?xml version="1.0" encoding="UTF-8"?>`
   because the reader synthesizes a `StartDocument` event when the source has no declaration
   (`src/reader/parser.rs:191-197`)
   and `XmlEvent::as_writer_event` forwards it (`src/reader/events.rs:241-248`).
   Reader `StartElement` events carry no self-closing flag (`src/reader/events.rs:62-73`),
   so `normalize_empty_elements(false)` expands every `<a/>` to `<a></a>` (`src/writer/emitter.rs:317-319`)
   and the default `true` collapses every `<a></a>` to `<a />` (`src/writer/emitter.rs:379`).
   Attribute values are always double-quoted (`src/writer/emitter.rs:349`).

## Root cause

### What the reader must do

XML 1.0 (Fifth Edition),
 [section 3.3.3, "Attribute-Value Normalization"][xml10-avn],
applies to every attribute value before it reaches the application:

```text
For a character reference, append the referenced character to the normalized value.
For a white space character (#x20, #xD, #xA, #x9), append a space character (#x20) to the normalized value.
```

A literal tab,
 newline,
 or carriage return therefore becomes a space,
while `&#9;`,
 `&#10;`,
 and `&#13;` survive as the characters they name.
A serializer that wants the value to survive a parse must write character references.
[XSLT and XQuery Serialization 3.1, section 5 "XML Output Method"][ser31-xml] states the same rule for serializers:

```text
CR, NL, TAB, NEL and LINE SEPARATOR characters in attribute nodes MUST be output respectively
as "&#xD;", "&#xA;", "&#x9;", "&#x85;", and "&#x2028;", or their equivalents.
```

NEL and LINE SEPARATOR are only normalized by XML 1.1 readers;
the XML 1.0 round trip in this document depends on tab,
 newline,
 and carriage return.

### xot 0.31.2

Source:
`faassen/xot` tag `v0.31.2` (commit `4a44505`),
identical to the crates.io 0.31.2 sources (`diff --recursive` of `src/` printed nothing).

`Xot::to_string` (`src/serialize.rs:149-151`) calls `serialize_xml_string` (`:225-231`),
which reaches `XmlSerializer` through `serialize_xml_write_with_normalizer` (`:308`).
The attribute arm writes a double-quoted value through `serialize_attribute`
(`src/output/xml_serializer.rs:154-163`):

```rust
Attribute(name_id, value) => {
    let fullname = self.fullname_serializer.attribute_fullname(*name_id)?;
    OutputToken {
        space: true,
        text: format!(
            "{}=\"{}\"",
            fullname,
            serialize_attribute((*value).into(), &self.normalizer)
        ),
    }
}
```

`serialize_attribute` (`src/entity.rs:218-252`) escapes four characters and copies everything else:

```rust
for c in normalized_content.chars() {
    match c {
        '&' => {
            change = true;
            result.push_str("&amp;")
        }
        '<' => {
            change = true;
            result.push_str("&lt;")
        }
        '\'' => {
            change = true;
            result.push_str("&apos;")
        }
        '"' => {
            change = true;
            result.push_str("&quot;")
        }
        _ => result.push(c),
    }
}
```

`CHANGES.md:868-869` records the choice in the 0.5.0 entry:

```text
- Made built in escaping rules less aggressive for serialization; text values
  now only escape `<` and `&`, and attributes escape those and `'` and '"`.
```

The parser applies section 3.3.3.
`src/parse.rs:107` decodes every attribute through `parse_attribute`,
which is `parse_content(content, true, ...)` (`src/entity.rs:11-16`).
Carriage returns become spaces at `src/entity.rs:28-40`:

```rust
if c == '\r' {
    if let Some((_, peeked)) = chars.peek() {
        if peeked == &'\n' {
            // consume next char
            chars.next();
        }
    }
    if !attribute {
        result.push('\n');
    } else {
        // https://www.w3.org/TR/xml/#AVNormalize
        result.push(' ');
    }
    change = true;
```

Tabs and newlines become spaces at `src/entity.rs:99-103`,
while character references push the referenced character at `:59-83`:

```rust
} else if attribute && (c == '\t' || c == '\n') {
    // https://www.w3.org/TR/xml/#AVNormalize
    // \r and \r\n already handled earlier
    result.push(' ');
    change = true;
```

So `Xot::parse` of `Xot::to_string` output changes the value inside xot itself.
The serializer claims the serialization rules
(`src/output/xml.rs:9-13`,
"This follows the rules <https://www.w3.org/TR/xslt-xquery-serialization/#xml-output>"),
and its divergence list (`src/output/xml.rs:15-30`) does not mention attribute whitespace.
The property test that serializes and re-parses random documents
generates attribute values from `XML_STRING_WITHOUT_WHITESPACE`
(`src/proptest.rs:28`,
 used at `:34`)
and only asserts that parsing succeeds (`src/proptest.rs:228`),
so no test exercised the combination.

The empty-element and quote rewrites in "Symptom" come from the same serializer:
`StartTagClose` writes `"/>"` for a childless element (`src/output/xml_serializer.rs:100-104`),
and the attribute arm always formats `{}=\"{}\"`.

### xml-rs 1.4.0

Source:
`kornelski/xml-rs` tag `1.4.0` (commit `41949a7`),
identical to the crates.io `xml` 1.4.0 sources.
`main` at `e2b391a` has no commits touching `src/escape.rs` or `src/writer/` after the tag
(`git log 1.4.0..main -- src/escape.rs src/writer` printed nothing).

`Emitter::emit_attributes` (`src/writer/emitter.rs:347-358`) always double-quotes
and escapes through `AttributeEscapes`:

```rust
pub fn emit_attributes<W: Write>(&self, target: &mut W, attributes: &[Attribute<'_>]) -> Result<()> {
    for attr in attributes {
        write!(target, " {}=\"", attr.name.repr_display())?;
        if self.config.perform_escaping {
            write!(target, "{}", Escaped::<AttributeEscapes>::new(attr.value))?;
        } else {
            write!(target, "{}", attr.value)?;
        }
        write!(target, "\"")?;
    }
    Ok(())
}
```

`AttributeEscapes` (`src/escape.rs:81-90`) covers newline and carriage return but not tab:

```rust
escapes!(
    AttributeEscapes,
    b'<'  => "&lt;",
    b'>'  => "&gt;",
    b'"'  => "&quot;",
    b'\'' => "&apos;",
    b'&'  => "&amp;",
    b'\n' => "&#xA;",
    b'\r' => "&#xD;",
);
```

Its doc comment gives layout as the reason (`src/escape.rs:110-113`,
"The following characters are escaped so that attributes are printed on a single line").
The newline and carriage-return arms came from [netvl/xml-rs#154][netvl-154] (2017),
which answered [netvl/xml-rs#153][netvl-153];
the owner's review comment on the pull request said the writer behavior
"is not covered by any spec"
and that bare newlines "just have to be parsed as spaces".

The reader started normalizing in commit `4e76ad5`
("Normalize whitespace in attribute values",
[kornelski/xml-rs#80][xmlrs-80],
fixing [kornelski/xml-rs#79][xmlrs-79]),
first released in 1.4.0
(`git tag --contains 4e76ad5` prints only `1.4.0`;
1.3.1 was cut from a branch without it).
`read_attribute_value` now pushes a space for each literal whitespace character
(`src/reader/parser.rs:661-670`):

```rust
_ if self.data.quote.is_some() => {
    if self.buf.len() > self.config.max_attribute_length {
        return Some(self.error(SyntaxError::ExceededConfiguredLimit));
    }
    match t {
        Token::Character(c) if is_whitespace_char(c) => self.buf.push(' '),
        _ => t.push_to_string(&mut self.buf),
    }
    None
},
```

`is_whitespace_char` matches `'\x20' | '\x0a' | '\x09' | '\x0d'` (`src/common.rs:132-134`),
and the test added with the change asserts that references are not normalized
(`tests/event_reader.rs:1063-1076`).
Since 1.4.0 the xml-rs writer and reader disagree about tab.
With xml-rs 1.3.1 the harness shows xml-rs reading its own tab back unchanged
while `roxmltree` reads a space,
so the tab defect already broke interoperability before 1.4.0
and only became visible inside xml-rs with #80.

### xmltree 0.12.0

Source:
`eminence/xmltree-rs` tag `v0.12.0` (commit `a4b85b4`),
identical to the crates.io 0.12.0 `src/lib.rs`.

xmltree does no escaping of its own.
`Element::write_with_config` (`src/lib.rs:408-423`) creates an xml-rs `EventWriter`
from the caller's `EmitterConfig` (`:414`),
and `Element::_write` (`src/lib.rs:348-400`) hands each attribute to it unchanged
(`src/lib.rs:361-367` and `:376-380`):

```rust
let mut attributes = Vec::with_capacity(self.attributes.len());
for (k, v) in &self.attributes {
    attributes.push(Attribute {
        name: Name::local(k),
        value: v,
    });
}
```

```rust
emitter.write(XmlEvent::StartElement {
    name,
    attributes: Cow::Owned(attributes),
    namespace,
})?;
```

The dependency is `xml = "1"`,
so xmltree inherits the xml-rs escape table exactly,
and an xml-rs fix reaches xmltree without an xmltree change
(verified in "Prototype:
 xml-rs").

### Prior reading corrected

A summary of the probe said all three crates write raw tab and newline characters.
Only `xot` writes a raw newline or carriage return.
xml-rs escapes both through `src/escape.rs:88-89`,
and the `&#xA;` seen in probe case `x05` was not a one-off position:
it is how xml-rs and xmltree write every newline in an attribute value,
as the escape table in "Verification" shows.

## Verification

Verified 2026-09-17.
Versions under test,
 with crates.io checksums from the harness `Cargo.lock`:

- `xot` 0.31.2,
   `fd6d2012838b97104fc3e8d2e46c53f3d1ca98706941ff4aae37811adaa60f4e`,
   published 2025-04-09,
   tag `v0.31.2` (`4a44505`).
- `xml` (xml-rs) 1.4.0,
   `2f45bb2c13fec6a6cb4c0f76a7e94839e110a14ec803ec2940777a94c347bc52`,
   published 2026-08-06,
   tag `1.4.0` (`41949a7`).
- `xmltree` 0.12.0,
   `cbc04313cab124e498ab1724e739720807b6dc405b9ed0edc5860164d2e4ff70`,
   published 2025-11-10,
   tag `v0.12.0` (`a4b85b4`).
- `roxmltree` 0.21.1,
   `f1964b10c76125c36f8afe190065a4bf9a87bf324842c05701330bba9f1cacbb`,
   used as an independent reader.

Each is the newest version on crates.io (`max_stable_version` from `https://crates.io/api/v1/crates/<name>`).
Builds ran in `docker.io/library/rust:1.97-bookworm`
(`sha256:389c1ae98c20fbcadca68a685482749267cec3c90893ae4671c5a37cc894c416`,
 rustc 1.97.1)
with `--memory=2g --cpus=2`,
no repository mount,
and a fresh `CARGO_HOME` with no credentials.

### Harness

```toml
# ${HARNESS}/Cargo.toml
[package]
name = "xml-attr-ws-harness"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
roxmltree = "=0.21.1"
xml = "=1.4.0"
xmltree = "=0.12.0"
xot = "=0.31.2"
```

```rust
// ${HARNESS}/src/main.rs
#![forbid(unsafe_code)]
//! Writes attribute values holding TAB, LF, and CR with xot, xml-rs, and xmltree,
//! then reads them back with the same crate and with roxmltree.

use xml::reader::{EventReader, XmlEvent as ReaderEvent};
use xml::writer::{EmitterConfig, XmlEvent as WriterEvent};

/// Attribute value whose whitespace must survive serialize, then parse.
const VALUE: &str = "a\tb\nc\rd";

/// Unedited source document that spells the same value with character references.
const SOURCE: &str = "<r v=\"a&#9;b&#10;c&#13;d\"/>";

/// Characters whose attribute escaping is traced.
const TRACED: [char; 8] = ['\t', '\n', '\r', '<', '>', '&', '"', '\''];

/// Reads attribute `v` of the document element with roxmltree (XML 1.0 section 3.3.3 reader).
fn roxmltree_read(xml: &str) -> String {
    let document = roxmltree::Document::parse(xml).expect("roxmltree parse");
    let element = document.root_element();
    return element.attribute("v").expect("attribute v").to_owned();
}

/// Sets attribute `v` on a fresh xot document and serializes it.
fn xot_write(value: &str) -> String {
    let mut xot = xot::Xot::new();
    let root = xot.parse("<r v=\"\"/>").expect("xot parse");
    let element = xot.document_element(root).expect("document element");
    let name = xot.add_name("v");
    xot.set_attribute(element, name, value);
    return xot.to_string(root).expect("xot serialize");
}

/// Reads attribute `v` of the document element with xot.
fn xot_read(xml: &str) -> String {
    let mut xot = xot::Xot::new();
    let root = xot.parse(xml).expect("xot parse");
    let element = xot.document_element(root).expect("document element");
    let name = xot.name("v").expect("name v");
    return xot.get_attribute(element, name).expect("attribute v").to_owned();
}

/// Parses and serializes with xot without any edit.
fn xot_copy(xml: &str) -> String {
    let mut xot = xot::Xot::new();
    let root = xot.parse(xml).expect("xot parse");
    return xot.to_string(root).expect("xot serialize");
}

/// Writes one element with attribute `v` through an xml-rs event writer.
fn xmlrs_write_with(config: EmitterConfig, value: &str) -> String {
    let mut out = Vec::new();
    let mut writer = config.create_writer(&mut out);
    let start = WriterEvent::start_element("r").attr("v", value);
    writer.write(start).expect("write start");
    writer.write(WriterEvent::end_element()).expect("write end");
    return String::from_utf8(out).expect("utf-8");
}

/// Writes one element with attribute `v` through the default xml-rs event writer.
fn xmlrs_write(value: &str) -> String {
    let config = EmitterConfig::new().write_document_declaration(false);
    return xmlrs_write_with(config, value);
}

/// Reads attribute `v` of the first element with the xml-rs event reader.
fn xmlrs_read(xml: &str) -> String {
    for event in EventReader::from_str(xml) {
        if let ReaderEvent::StartElement { attributes, .. } = event.expect("xml-rs read") {
            let attribute = attributes.into_iter().find(|a| a.name.local_name == "v");
            return attribute.expect("attribute v").value;
        }
    }
    panic!("no element");
}

/// Copies reader events to the writer without any edit, skipping the synthesized document start.
fn xmlrs_copy(xml: &str) -> String {
    let mut out = Vec::new();
    let config = EmitterConfig::new().write_document_declaration(false);
    let mut writer = config.create_writer(&mut out);
    for event in EventReader::from_str(xml) {
        let event = event.expect("xml-rs read");
        if matches!(event, ReaderEvent::StartDocument { .. }) {
            continue;
        }
        if let Some(writer_event) = event.as_writer_event() {
            writer.write(writer_event).expect("xml-rs write");
        }
    }
    return String::from_utf8(out).expect("utf-8");
}

/// Writes an xmltree element without a declaration.
fn xmltree_emit(element: &xmltree::Element) -> String {
    let mut out = Vec::new();
    let config = xmltree::EmitterConfig::new().write_document_declaration(false);
    element.write_with_config(&mut out, config).expect("xmltree write");
    return String::from_utf8(out).expect("utf-8");
}

/// Writes one element with attribute `v` through xmltree.
fn xmltree_write(value: &str) -> String {
    let mut element = xmltree::Element::new("r");
    element.attributes.insert("v".to_owned(), value.to_owned());
    return xmltree_emit(&element);
}

/// Reads attribute `v` of the root element with xmltree.
fn xmltree_read(xml: &str) -> String {
    let element = xmltree::Element::parse(xml.as_bytes()).expect("xmltree parse");
    return element.attributes["v"].clone();
}

/// Parses and writes with xmltree without any edit.
fn xmltree_copy(xml: &str) -> String {
    let element = xmltree::Element::parse(xml.as_bytes()).expect("xmltree parse");
    return xmltree_emit(&element);
}

/// Encodes attribute text for its own quote character, as file-enforcer `escapeXmlAttribute` does.
fn encode_attribute(value: &str, quote: char) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' if quote == '"' => out.push_str("&quot;"),
            '\'' if quote == '\'' => out.push_str("&apos;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            '\t' => out.push_str("&#9;"),
            other => out.push(other),
        }
    }
    return out;
}

/// Replaces only the bytes of attribute `v` on the document element, then re-parses the result.
fn roxmltree_splice(source: &str, value: &str) -> String {
    let document = roxmltree::Document::parse(source).expect("roxmltree parse");
    let attribute = document.root_element().attribute_node("v").expect("attribute v");
    let range = attribute.range_value();
    let quote = char::from(source.as_bytes()[range.start - 1]);
    let mut out = source.to_owned();
    out.replace_range(range, &encode_attribute(value, quote));
    roxmltree::Document::parse(&out).expect("spliced output parses");
    return out;
}

/// Writes pre-encoded attribute text through xml-rs with its own escaping turned off.
fn xmlrs_write_unescaped(encoded: &str) -> String {
    let mut config = EmitterConfig::new().write_document_declaration(false);
    config.perform_escaping = false;
    return xmlrs_write_with(config, encoded);
}

/// Returns the attribute text between `v="` and the next double quote.
fn attribute_text(xml: &str) -> String {
    let start = xml.find("v=\"").expect("v=\"") + 3;
    let end = start + xml[start..].find('"').expect("closing quote");
    return xml[start..end].to_owned();
}

/// Prints one round trip with both read-backs.
fn report(label: &str, xml: &str, same_crate: &str) {
    let independent = roxmltree_read(xml);
    println!("{label}");
    println!("  serialized:        {xml:?}");
    println!("  same crate reads:  {same_crate:?} preserved={}", same_crate == VALUE);
    println!("  roxmltree reads:   {independent:?} preserved={}", independent == VALUE);
}

fn main() {
    println!("source parse check: xot={:?} xml-rs={:?}", xot_read(SOURCE), xmlrs_read(SOURCE));
    println!("  xmltree={:?} roxmltree={:?}", xmltree_read(SOURCE), roxmltree_read(SOURCE));
    println!();
    println!("escape table (character -> attribute text): xot | xml-rs | xmltree");
    for c in TRACED {
        let value = c.to_string();
        let xot_text = attribute_text(&xot_write(&value));
        let xmlrs_text = attribute_text(&xmlrs_write(&value));
        let xmltree_text = attribute_text(&xmltree_write(&value));
        println!("  {c:?} -> {xot_text:?} | {xmlrs_text:?} | {xmltree_text:?}");
    }
    println!();
    let out = xot_write(VALUE);
    report("xot set_attribute then to_string", &out, &xot_read(&out));
    let out = xot_copy(SOURCE);
    report("xot parse then to_string, no edit", &out, &xot_read(&out));
    let out = xmlrs_write(VALUE);
    report("xml-rs EventWriter start_element attr", &out, &xmlrs_read(&out));
    let out = xmlrs_copy(SOURCE);
    report("xml-rs reader events copied to writer, no edit", &out, &xmlrs_read(&out));
    let out = xmltree_write(VALUE);
    report("xmltree attributes.insert then write_with_config", &out, &xmltree_read(&out));
    let out = xmltree_copy(SOURCE);
    report("xmltree parse then write_with_config, no edit", &out, &xmltree_read(&out));
    println!();
    println!("workarounds and rejected approaches");
    let out = roxmltree_splice("<r  v='old'   w=\"keep\" />", VALUE);
    report("roxmltree range_value splice with encoder", &out, &roxmltree_read(&out));
    let out = xmlrs_write_unescaped(&encode_attribute(VALUE, '"'));
    report("xml-rs perform_escaping = false, pre-encoded value", &out, &xmlrs_read(&out));
    let out = xmlrs_write(&encode_attribute(VALUE, '"'));
    report("xml-rs pre-encoded value, escaping on (rejected)", &out, &xmlrs_read(&out));
    let out = xot_write(&encode_attribute(VALUE, '"'));
    report("xot pre-encoded value (rejected)", &out, &xot_read(&out));
}
```

Run it in a bounded container:

```bash
mkdir --parents "${HOME}/temp/agent"
chmod 700 "${HOME}/temp/agent"
HARNESS=$(mktemp --directory "${HOME}/temp/agent/xml-attr-ws-harness.XXXXXXXX")
CARGO_CACHE=$(mktemp --directory "${HOME}/temp/agent/xml-attr-ws-cargo-home.XXXXXXXX")
# write the Cargo.toml and src/main.rs blocks of the "Harness" section into "${HARNESS}"
podman run --memory=2g --cpus=2 --rm \
  --volume "${HARNESS}:/work:Z" \
  --volume "${CARGO_CACHE}:/cargo-home:z" \
  --env CARGO_HOME=/cargo-home \
  --workdir /work \
  docker.io/library/rust:1.97-bookworm \
  cargo run --quiet --jobs 4
```

Output against the published crates:

```text
source parse check: xot="a\tb\nc\rd" xml-rs="a\tb\nc\rd"
  xmltree="a\tb\nc\rd" roxmltree="a\tb\nc\rd"

escape table (character -> attribute text): xot | xml-rs | xmltree
  '\t' -> "\t" | "\t" | "\t"
  '\n' -> "\n" | "&#xA;" | "&#xA;"
  '\r' -> "\r" | "&#xD;" | "&#xD;"
  '<' -> "&lt;" | "&lt;" | "&lt;"
  '>' -> ">" | "&gt;" | "&gt;"
  '&' -> "&amp;" | "&amp;" | "&amp;"
  '"' -> "&quot;" | "&quot;" | "&quot;"
  '\'' -> "&apos;" | "&apos;" | "&apos;"

xot set_attribute then to_string
  serialized:        "<r v=\"a\tb\nc\rd\"/>"
  same crate reads:  "a b c d" preserved=false
  roxmltree reads:   "a b c d" preserved=false
xot parse then to_string, no edit
  serialized:        "<r v=\"a\tb\nc\rd\"/>"
  same crate reads:  "a b c d" preserved=false
  roxmltree reads:   "a b c d" preserved=false
xml-rs EventWriter start_element attr
  serialized:        "<r v=\"a\tb&#xA;c&#xD;d\" />"
  same crate reads:  "a b\nc\rd" preserved=false
  roxmltree reads:   "a b\nc\rd" preserved=false
xml-rs reader events copied to writer, no edit
  serialized:        "<r v=\"a\tb&#xA;c&#xD;d\" />"
  same crate reads:  "a b\nc\rd" preserved=false
  roxmltree reads:   "a b\nc\rd" preserved=false
xmltree attributes.insert then write_with_config
  serialized:        "<r v=\"a\tb&#xA;c&#xD;d\" />"
  same crate reads:  "a b\nc\rd" preserved=false
  roxmltree reads:   "a b\nc\rd" preserved=false
xmltree parse then write_with_config, no edit
  serialized:        "<r v=\"a\tb&#xA;c&#xD;d\" />"
  same crate reads:  "a b\nc\rd" preserved=false
  roxmltree reads:   "a b\nc\rd" preserved=false

workarounds and rejected approaches
roxmltree range_value splice with encoder
  serialized:        "<r  v='a&#9;b&#10;c&#13;d'   w=\"keep\" />"
  same crate reads:  "a\tb\nc\rd" preserved=true
  roxmltree reads:   "a\tb\nc\rd" preserved=true
xml-rs perform_escaping = false, pre-encoded value
  serialized:        "<r v=\"a&#9;b&#10;c&#13;d\" />"
  same crate reads:  "a\tb\nc\rd" preserved=true
  roxmltree reads:   "a\tb\nc\rd" preserved=true
xml-rs pre-encoded value, escaping on (rejected)
  serialized:        "<r v=\"a&amp;#9;b&amp;#10;c&amp;#13;d\" />"
  same crate reads:  "a&#9;b&#10;c&#13;d" preserved=false
  roxmltree reads:   "a&#9;b&#10;c&#13;d" preserved=false
xot pre-encoded value (rejected)
  serialized:        "<r v=\"a&amp;#9;b&amp;#10;c&amp;#13;d\"/>"
  same crate reads:  "a&#9;b&#10;c&#13;d" preserved=false
  roxmltree reads:   "a&#9;b&#10;c&#13;d" preserved=false
```

The `preserved` check is not blind:
the same harness prints `preserved=true` for every serializer row
when built against the prototype patches ("Prototype:
 xot" and "Prototype:
 xml-rs").

With `xml = "=1.3.1"` in `Cargo.toml` (all else unchanged),
the xml-rs rows serialize identically,
but xml-rs 1.3.1 reads its own output back as `"a\tb\nc\rd" preserved=true`
while `roxmltree` still reads `"a b\nc\rd" preserved=false`.

### Values that round-trip cleanly

- Any attribute value without tab,
   newline,
   or carriage return,
   in all three crates,
   including `<`,
   `>`,
   `&`,
   `"`,
   and `'` (escape table rows).
- Newline and carriage return through xml-rs 1.4.0 and xmltree 0.12.0,
   written as `&#xA;` and `&#xD;`.
- Tab through xml-rs 1.3.1 when xml-rs 1.3.1 also reads it back
   (xml-rs only;
   conforming readers still see a space).

### Values that fail: literal tab (xot, xml-rs, xmltree)

- `set_attribute`,
   `attr`,
   or `attributes.insert` with a value containing U+0009.
- Unedited copies of a source attribute spelled with a character reference such as `&#9;`,
   through `Xot::parse` then `Xot::to_string`,
   xml-rs `EventReader` events written back through `EventWriter`,
   or `xmltree::Element::parse` then `write_with_config`.

### Values that fail: literal newline or carriage return (xot only)

- `Xot::set_attribute` with a value containing U+000A or U+000D,
   for example LSP4IJ `configurationContent` JSON (probe case `x05`).
- `Xot::parse` then `Xot::to_string` of a source attribute spelled with character references `&#10;` or `&#13;`.

## Verified workarounds

### Splice attribute bytes after a strict parse, encoding with a repository encoder

Parse with `roxmltree`,
take the value's byte range from `Attribute::range_value`,
replace only those bytes with text encoded for the attribute's own quote character,
and re-parse the result.
`roxmltree_splice` and `encode_attribute` in the harness are the minimal form;
the encoder mirrors file-enforcer `escapeXmlAttribute`
(`package/file-enforcer/core/src/pipeline/xml-coding.ts:177-193`),
plus `&apos;` for single-quoted values.
The harness row "roxmltree range_value splice with encoder" shows a single-quoted source keeping its quote,
its spacing,
 and the untouched `w` attribute,
with the value preserved by both readers.
The `meow` research probe (`xml-rox`,
 option X1 in the research document) extends this to entry replacement,
insertion before `</map>`,
 and attribute-level `setOption`,
and passed all of its XML cases with byte-identical untouched spans and idempotent reruns.

Tradeoffs:

- Every byte outside the replaced range is kept,
   so formatting and comments survive,
   but new elements must be formatted by repository code (indentation,
   quote style).
- The parse and the splice are separate steps;
   each edit must re-parse before the next range is used,
   and the final text must re-parse to catch splices that break well-formedness.
- `range_value` is wrong for qualified names longer than `u16::MAX` bytes
   or more than `u8::MAX` spaces around `=`
   (`roxmltree` 0.21.1 `src/lib.rs:620-622`);
   the `positions` feature it needs is on by default.
- Comparing a decoded value against the wanted value needs the parsed tree,
   so an equal-value set does not rewrite the escaping the source chose.

### Same splice with a `quick-xml` event scan

The research probe `xml-quick` indexes the same byte ranges from `quick-xml` 0.42.0 borrowed events
and reuses the splice editor,
passing the same XML cases.
Tradeoffs beyond the roxmltree form:
the repository has to check what `quick-xml` leaves to the caller
(unclosed elements at end of input,
 a single root,
 text outside the root,
 DOCTYPE entity declarations),
and attribute error positions are relative to the attribute value.
Its own `normalized_value` documents the same whitespace translation
(`quick-xml` 0.42.0 `src/events/attributes.rs:94-103`),
so the encoder is still required.

### xml-rs or xmltree writers with `perform_escaping = false` and caller-side encoding

When a program must keep an xml-rs or xmltree writer,
set the public field `EmitterConfig::perform_escaping` to `false`
(the `gen_setters!` builder list in 1.4.0 omits it,
 `src/writer/config.rs:147-157`)
and pass values already encoded by a complete encoder,
as `xmlrs_write_unescaped` does.
The harness row "xml-rs perform_escaping = false,
 pre-encoded value" preserves the value in both readers.

Tradeoffs:

- The flag also disables escaping of character data (`src/writer/emitter.rs:425-429`),
   so every text event must be encoded by the caller too,
   and the config documentation warns the writer "may produce non-well-formed documents"
   (`src/writer/config.rs:37-38`).
- For xmltree the tree must hold encoded strings at write time,
   so in-memory values and written values differ,
   and values read with `Element::parse` must be re-encoded before writing.
- None of the other rewrites in "Symptom" go away.

## What does not work

- **Pre-encoding the value while the serializer still escapes.**
   xot and xml-rs escape `&`,
   so `&#9;` becomes `&amp;#9;` and reads back as the literal text `&#9;`
   (harness rows marked "rejected").
- **Any tree or event round trip through these crates when untouched bytes must stay identical.**
   Even with the escaping fixed,
   `xot` rewrites quotes and `" />"`,
   `xmltree` drops whitespace and reorders attributes,
   and xml-rs event copies change declarations and empty-element spelling
   ("Other rewrites the probe recorded").
   The research harness scored each of them 2 of 9 XML cases,
   passing only the malformed-input cases.
- **A custom `xot` `Normalizer`.**
   Rejected from source,
   not run:
   `serialize_attribute` calls `normalizer.normalize(content)` before its escape loop
   (`src/entity.rs:224-225`),
   so a normalizer that inserts `&#9;` has its `&` escaped like any other.

## Upstream filing artifact

### Out-of-scope check

`.out-of-scope/` holds `bun-install.md`,
 `cargo-workspace.md`,
 `claude-code-upstream-bugs.md`,
 `codex-harness.md`,
`jsr.md`,
 `lightningcss.md`,
 `low-impact-typescript-formatting.md`,
 `module-es-monolith.md`,
`pi-gpt55-long-context.md`,
 `terminal-title-fork-parity-tests.md`,
 and `typescript-project-references.md`.
`rg --ignore-case 'xml|xot|serializ|crates|rust' .out-of-scope/` matched only a `lightningcss` deserialize error
and the Cargo workspace decision;
no exemption covers XML serializers or these crates.

### Duplicate search

Searched on 2026-09-17 with `gh search issues` and `gh search prs` (no `--state`,
 so open and closed)
for `tab`,
 `attribute escape`,
 `attribute whitespace`,
 `normalization`,
 `newline`,
 `&#9`,
 `round trip`,
and `escape_str_attribute`,
plus `gh issue list --state all` and `gh pr list --state all` title listings,
in `faassen/xot`,
 `eminence/xmltree-rs`,
 `kornelski/xml-rs`,
and `netvl/xml-rs` (the repository `kornelski/xml-rs` is a GitHub fork of).

- `faassen/xot`:
   no duplicate.
   Related:
   [faassen/xot#43][xot-43] (open,
   Canonical XML feature request),
   where the owner answered the C14N item
   "Special characters in attribute values and character content are replaced by character references"
   with "I don't know;
   it doesn't seem to define what "special characters" are?
   Might need a serialization parameter."
- `kornelski/xml-rs`:
   duplicate [kornelski/xml-rs#88][xmlrs-88]
   ("`\r` in character data and `\t` in attribute values do not survive a write/read round-trip",
   open since 2026-09-04,
   no comments,
   no linked pull request).
   Its body already has a writer-to-reader reproduction on 1.4.0 and `main`,
   the observation that the writer escapes CR and LF but not TAB,
   the link to #79,
   and an offer to contribute tests.
   Related,
   closed:
   [kornelski/xml-rs#79][xmlrs-79] and [kornelski/xml-rs#80][xmlrs-80] (reader normalization).
- `netvl/xml-rs`:
   no open duplicate;
   [netvl/xml-rs#153][netvl-153] and [netvl/xml-rs#154][netvl-154] (2017) added the newline and carriage-return escapes.
- `eminence/xmltree-rs`:
   no duplicate.
   In [eminence/xmltree-rs#47][xmltree-47] (closed) a reporter concluded that escaping behavior
   "is caused by crate xml-rs and not by the xmltree-rs crate".

No open pull request touches the fix locations:
`gh api repos/faassen/xot/pulls/<n>/files` for open #44,
 #45,
 and #46 lists no `src/entity.rs`,
and open [kornelski/xml-rs#90][xmlrs-90] does not touch `src/escape.rs` or `tests/event_writer.rs`.
xot #44 (lint fixes) does edit `tests/roundtrip.rs`,
where the xot prototype adds a test case,
so one of the two would need a rebase.

### Upstream filing decision: xot

1.  **Is it really upstream's fault?**
    Yes,
     behavior.
    `Xot::parse` applies section 3.3.3 (`src/entity.rs:28-40`,
     `:99-103`),
    so `Xot::to_string` output that `Xot::parse` reads differently is a serializer defect
    (`src/entity.rs:218-252`),
    and the serializer states it follows the serialization rules that require character references
    (`src/output/xml.rs:9-13`).
2.  **Can upstream fix it?**
    Yes.
    Three match arms in `serialize_attribute`;
    the prototype passes the full test suite.
3.  **Are they supporting this use case?**
    Yes.
    `README.md:31` lists "Parse XML into a tree,
     and serialize back to XML.",
    `tests/roundtrip.rs` asserts parse-then-serialize equality,
    and `src/output/xml.rs:9-30` adopts the serialization spec with a divergence list that omits this.
    The property test excluding whitespace from attribute values (`src/proptest.rs:28`,
     `:34`)
    left the combination untested rather than declaring it unsupported.
4.  **Would the repo welcome our contribution?**
    Yes.
    The clone has no `CONTRIBUTING.md`,
     issue or pull request templates,
     or `.github/` directory;
    `gh api repos/faassen/xot/community/profile` lists none;
    `repos/faassen/.github` does not exist (HTTP 404);
    tracker searches for `AI`,
     `LLM`,
     `Claude`,
     `Copilot`,
     `ChatGPT`,
     and `generated` found no policy.
    No ban was found.
    The owner wrote "PRs are welcome!"
     on #43 (2025-04-03)
    and merged outside pull requests #33,
     #39,
     and #40.
5.  **Will they likely fix it?**
    Plausible,
     a soft yes.
    Nothing documents escaping as a non-goal.
    The last owner activity is the 0.31.2 release and the #43 comment (April 2025),
    and outside pull requests #44 to #46 (February to May 2026) have no review,
    which is silence,
     not a decline.
    The #43 answer suggests the owner may frame escaping as a serialization parameter;
    the draft cites the MUST in the rule xot adopts.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Yes,
     "Prototype:
     xot":
    the fix stays inside the existing escape match,
    adds a unit test and a `tests/roundtrip.rs` case,
    fails before the change and passes after,
    and the whole suite passes.

All six hold.
The draft for faassen/xot is fileable as written;
whether to file stays with the user.

### Upstream filing decision: xml-rs

1.  **Is it really upstream's fault?**
    Yes,
     behavior.
    Since 1.4.0 the xml-rs reader applies section 3.3.3 (`src/reader/parser.rs:666`)
    while `AttributeEscapes` omits tab (`src/escape.rs:81-90`),
    so xml-rs cannot read its own output back;
    before 1.4.0 the same output already changed in conforming readers.
2.  **Can upstream fix it?**
    Yes.
    One escape arm.
3.  **Are they supporting this use case?**
    Yes.
    `README.md:18` claims "XML spec conformance better than other pure-Rust libraries",
    `README.md:32-34` presents reader-to-writer transformation chains,
    and the project escaped newline and carriage return on purpose (#154) and normalized the reader (#80).
4.  **Would the repo welcome our contribution?**
    Yes.
    `.github/` holds only `FUNDING.yml`,
     `dependabot.yml`,
     and `workflows/main.yml`;
    there is no `CONTRIBUTING.md` or template;
    the community-profile API returns 404 for this fork and `repos/kornelski/.github` does not exist;
    tracker searches for `AI`,
     `LLM`,
     `Claude`,
     `Copilot`,
     `ChatGPT`,
     and `generated` found no policy.
    No ban was found.
    The owner merged outside pull requests #76,
     #78,
     #80,
     #81,
     #83,
     #85,
     and #87,
    each within five days of opening.
5.  **Will they likely fix it?**
    Yes.
    The project is active (1.4.0 on 2026-08-06,
     #87 merged 2026-08-11),
    and #80,
     the fix for #79,
     was merged the day #79 was opened.
    #88 has had no response for 13 days,
     which is not a signal against.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Yes,
     "Prototype:
     xml-rs".

All six hold,
 but [kornelski/xml-rs#88][xmlrs-88] already reports the attribute-tab behavior,
so the artifact is an additive comment,
 not a new issue.
Additive content,
 checked against the thread:
the prototype diff with pre-patch and post-patch test results,
the version history (1.3.1 self round trip passes while conforming readers already changed the value),
and the downstream effect on xmltree with the fix reaching it unchanged.
The thread's other half (carriage return in character data) is not addressed by this prototype.

### Upstream filing decision: xmltree

1.  **Is it really upstream's fault?**
    No.
    xmltree passes attribute values unchanged to the xml-rs `EventWriter` (`src/lib.rs:361-380`,
     `:414`);
    the escape table is xml-rs code.
    The `HashMap` attribute order in "Symptom" is the owner-decided default (#21),
    and the whitespace dropping is how its tree builder handles `Whitespace` events (`src/lib.rs:235`),
    not an escaping choice.
2.  **Can upstream fix it?**
    Not in xmltree without duplicating xml-rs escaping;
    the xml-rs prototype fixes xmltree output with no xmltree change,
    through the existing `xml = "1"` requirement.
3.  **Are they supporting this use case?**
    Writing is supported (`Element::write`,
     `write_with_config`),
    with escaping delegated to xml-rs.
4.  **Would the repo welcome our contribution?**
    No `CONTRIBUTING.md`,
     templates,
     or policy were found (community profile and clone);
    open [eminence/xmltree-rs#59][xmltree-59] discloses LLM assistance and has no maintainer response,
    so no ban was found.
5.  **Will they likely fix it?**
    Not assessed further;
     constraint 1 fails.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Not applicable;
    the auto-prototype step needs constraints 1 to 5 to hold,
     and constraint 1 fails.

Decision:
do not file with xmltree.
There is nothing to add there;
the xml-rs comment covers the xmltree effect.

### Prototype: xot

Disposable clone:
`mktemp --directory "${HOME}/temp/agent/upstream-prototype.XXXXXXXX"`,
then `gh repo clone faassen/xot "<dir>/xot" -- --branch v0.31.2 --depth 1`.
`origin` was `https://github.com/faassen/xot.git` (push URL set to `DISABLED`),
`HEAD` `4a44505f23844d6101a8da7397e434562b1ac513`,
`git describe --tags` `v0.31.2`.

```diff
diff --git a/src/entity.rs b/src/entity.rs
index 3c0c37b..22cdd0f 100644
--- a/src/entity.rs
+++ b/src/entity.rs
@@ -240,6 +240,23 @@ pub(crate) fn serialize_attribute<'a, N: Normalizer>(
                 change = true;
                 result.push_str("&quot;")
             }
+            // A parser replaces literal tab, newline, and carriage return in
+            // attribute values with spaces, so they must be character references
+            // to survive a round trip.
+            // https://www.w3.org/TR/xml/#AVNormalize
+            // https://www.w3.org/TR/xslt-xquery-serialization-31/#xml-output
+            '\t' => {
+                change = true;
+                result.push_str("&#x9;")
+            }
+            '\n' => {
+                change = true;
+                result.push_str("&#xA;")
+            }
+            '\r' => {
+                change = true;
+                result.push_str("&#xD;")
+            }
             _ => result.push(c),
         }
     }
@@ -457,6 +474,15 @@ mod tests {
         );
     }
 
+    #[test]
+    fn test_serialize_attribute_whitespace() {
+        let text = "a\tb\nc\rd";
+        assert_eq!(
+            serialize_attribute(text.into(), &NoopNormalizer),
+            "a&#x9;b&#xA;c&#xD;d"
+        );
+    }
+
     #[test]
     fn test_serialize_attribute_no_entities() {
         let text = "hello";
diff --git a/tests/roundtrip.rs b/tests/roundtrip.rs
index 8addcb2..bdaa275 100644
--- a/tests/roundtrip.rs
+++ b/tests/roundtrip.rs
@@ -67,6 +67,10 @@ fn roundtrip(#[values(
   (
     "prefix stability",
     r#"<root xmlns:foo="http://example.com" xmlns:bar="http://example.com/bar"/>"#,
+  ),
+  (
+    "whitespace character references in attribute",
+    r#"<root foo="a&#x9;b&#xA;c&#xD;d"/>"#,
   )
 )] value: RoundTripEntry) {
     let (name, xml) = value;
```

`serialize_attribute` is also used by the HTML5 serializer for attributes in a namespace
(`src/output/html5_serializer.rs:245-246`);
character references are valid there too,
 and the HTML5 serializer tests pass.

Verification (container as in "Verification",
 source mounted at `/src`):

```bash
PROTO=$(mktemp --directory "${HOME}/temp/agent/upstream-prototype.XXXXXXXX")
gh repo clone faassen/xot "${PROTO}/xot" -- --branch v0.31.2 --depth 1
git -C "${PROTO}/xot" remote set-url --push origin DISABLED
# save the diff block of this section as "${PROTO}/xot-fix.patch"
git -C "${PROTO}/xot" apply ../xot-fix.patch
podman run --memory=2g --cpus=2 --rm \
  --volume "${PROTO}:/src:Z" \
  --volume "${CARGO_CACHE}:/cargo-home:z" \
  --env CARGO_HOME=/cargo-home \
  --workdir /src/xot \
  docker.io/library/rust:1.97-bookworm \
  cargo test --jobs 4
```

Post-patch:
every test binary and the doctests report `ok`,
485 passed and 0 failed across 16 result lines,
including `entity::tests::test_serialize_attribute_whitespace ... ok`
and `roundtrip::value_18____whitespacecharacterreferencesinattribute___r__rootfoo__a_x9b_ ... ok`.

Pre-patch
(first hunk reversed with `git apply --reverse`,
 tests kept,
`cargo test --jobs 4 --no-fail-fast --lib --test roundtrip`;
the roundtrip failure's `thread ... panicked at tests/roundtrip.rs:80:5:` line is omitted for width):

```text
test entity::tests::test_serialize_attribute_whitespace ... FAILED
thread 'entity::tests::test_serialize_attribute_whitespace' (531) panicked at src/entity.rs:463:9:
  left: "a\tb\nc\rd"
 right: "a&#x9;b&#xA;c&#xD;d"
test result: FAILED. 138 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test roundtrip::value_18____whitespacecharacterreferencesinattribute___r__rootfoo__a_x9b_ ... FAILED
  left: "<root foo=\"a&#x9;b&#xA;c&#xD;d\"/>"
 right: "<root foo=\"a\tb\nc\rd\"/>"
test result: FAILED. 17 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Consumer boundary:
the harness built with
`--config 'patch.crates-io.xot.path="/xot-proto/xot"'`
prints `"<r v=\"a&#x9;b&#xA;c&#xD;d\"/>"` and `preserved=true` in both xot rows.

### Prototype: xml-rs

Disposable clone:
`gh repo clone kornelski/xml-rs "<dir>/xml-rs" -- --branch 1.4.0 --depth 1` in a fresh `mktemp` directory.
`origin` was `https://github.com/kornelski/xml-rs.git` (push URLs set to `DISABLED`),
`HEAD` `41949a759dada9493b29e0661b318f1960d1f849`,
`git describe --tags` `1.4.0`.

```diff
diff --git a/src/escape.rs b/src/escape.rs
index c1e4dd1..bfce122 100644
--- a/src/escape.rs
+++ b/src/escape.rs
@@ -87,6 +87,7 @@ escapes!(
     b'&'  => "&amp;",
     b'\n' => "&#xA;",
     b'\r' => "&#xD;",
+    b'\t' => "&#x9;",
 );
 
 escapes!(
@@ -107,10 +108,11 @@ escapes!(
 /// * `'` → `&apos;`
 /// * `&` → `&amp;`
 ///
-/// The following characters are escaped so that attributes are printed on
-/// a single line:
+/// The following whitespace characters are escaped, because a parser replaces
+/// literal ones in attribute values with spaces (attribute-value normalization):
 /// * `\n` → `&#xA;`
 /// * `\r` → `&#xD;`
+/// * `\t` → `&#x9;`
 ///
 /// The resulting string is safe to use inside XML attribute values or in PCDATA sections.
 ///
@@ -144,7 +146,7 @@ mod tests {
 
     #[test]
     fn test_escape_str_attribute() {
-        assert_eq!(escape_str_attribute("<>'\"&\n\r"), "&lt;&gt;&apos;&quot;&amp;&#xA;&#xD;");
+        assert_eq!(escape_str_attribute("<>'\"&\n\r\t"), "&lt;&gt;&apos;&quot;&amp;&#xA;&#xD;&#x9;");
         assert_eq!(escape_str_attribute("no_escapes"), "no_escapes");
     }
 
diff --git a/tests/event_writer.rs b/tests/event_writer.rs
index 40a6da2..10db312 100755
--- a/tests/event_writer.rs
+++ b/tests/event_writer.rs
@@ -278,6 +278,34 @@ fn attribute_escaping() {
     );
 }
 
+#[test]
+fn attribute_whitespace_survives_reading_back() {
+    use xml::reader::XmlEvent as ReaderEvent;
+    use xml::writer::XmlEvent;
+
+    let value = "a\tb\nc\rd";
+    let mut b = Vec::new();
+    {
+        let mut w = EmitterConfig::new()
+            .write_document_declaration(false)
+            .create_writer(&mut b);
+        unwrap_all! {
+            w.write(XmlEvent::start_element("hello").attr("test", value));
+            w.write(XmlEvent::end_element())
+        }
+    }
+    assert_eq!(str::from_utf8(&b).unwrap(), "<hello test=\"a&#x9;b&#xA;c&#xD;d\" />");
+
+    let read_back = EventReader::new(b.as_slice())
+        .into_iter()
+        .find_map(|e| match e.unwrap() {
+            ReaderEvent::StartElement { attributes, .. } => Some(attributes[0].value.clone()),
+            _ => None,
+        })
+        .unwrap();
+    assert_eq!(read_back, value);
+}
+
 #[test]
 fn accidental_cdata_suffix_in_characters_is_escaped() {
     let mut b = Vec::new();
```

Verification:
the same commands as "Prototype:
 xot" with `kornelski/xml-rs`,
 `--branch 1.4.0`,
 this section's diff,
and `--workdir /src/xml-rs`,
 running `cargo test --jobs 4`.

Post-patch:
157 passed,
 0 failed,
 2 ignored (the `rust,ignore` examples at `README.md:111` and `:190`,
 which `src/lib.rs:12` includes as doctests),
across the unit tests,
 `eol_normalization`,
 `event_reader`,
 `event_writer`,
 `streaming`,
 `version`,
`xmlconf`,
 and the doctests,
including `escape::tests::test_escape_str_attribute ... ok`
and `attribute_whitespace_survives_reading_back ... ok`.

Pre-patch
(the `b'\t'` arm reversed,
 tests kept,
`cargo test --jobs 4 --no-fail-fast --lib --test event_writer`):

```text
test escape::tests::test_escape_str_attribute ... FAILED
thread 'escape::tests::test_escape_str_attribute' (504) panicked at src/escape.rs:148:9:
  left: "&lt;&gt;&apos;&quot;&amp;&#xA;&#xD;\t"
 right: "&lt;&gt;&apos;&quot;&amp;&#xA;&#xD;&#x9;"
test result: FAILED. 37 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test attribute_whitespace_survives_reading_back ... FAILED
thread 'attribute_whitespace_survives_reading_back' (543) panicked at tests/event_writer.rs:297:5:
  left: "<hello test=\"a\tb&#xA;c&#xD;d\" />"
 right: "<hello test=\"a&#x9;b&#xA;c&#xD;d\" />"
test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Consumer boundary:
the harness built with both
`--config 'patch.crates-io.xot.path="/xot-proto/xot"'` and
`--config 'patch.crates-io.xml.path="/xmlrs-proto/xml-rs"'`
prints `&#x9;`,
 `&#xA;`,
 and `&#xD;` in every escape-table column
and `preserved=true` in every xot,
 xml-rs,
 and xmltree row,
with no change to xmltree.

### Draft new issue for faassen/xot (fileable)

The user chose to file this personally;
tracked in issue #546 (2026-09-17).

Title and labels:

```text
Title: Attribute values with tab, newline, or carriage return come back as spaces after to_string and parse
Labels: bug
```

~~~md
## Description

`Xot::to_string` escapes `&`, `<`, `'`, and `"` in attribute values,
but writes tab, newline, and carriage return literally.
XML 1.0 attribute-value normalization (https://www.w3.org/TR/xml/#AVNormalize)
turns each literal one into a space on the next parse,
and `Xot::parse` implements that normalization,
so these values do not survive `to_string` followed by `parse`.
No edit is needed to hit it:
parsing `<r v="a&#9;b"/>` and serializing it writes a literal tab.

Source trace at v0.31.2 (`4a44505`):

- `src/output/xml_serializer.rs:154-163` writes each attribute as `name="..."` through `serialize_attribute`.
- `src/entity.rs:218-252` (`serialize_attribute`) has match arms for `&`, `<`, `'`, and `"` only.
- `src/entity.rs:28-40` and `:99-103` (`parse_content` with `attribute: true`, from `src/parse.rs:107`)
  replace literal carriage return, tab, and newline in attribute values with spaces.
- `src/output/xml.rs:9-13` says serialization follows the XML output method of XSLT and XQuery Serialization.
  Section 5 of Serialization 3.1 (https://www.w3.org/TR/xslt-xquery-serialization-31/#xml-output) says:
  "CR, NL, TAB, NEL and LINE SEPARATOR characters in attribute nodes MUST be output respectively
  as "&#xD;", "&#xA;", "&#x9;", "&#x85;", and "&#x2028;", or their equivalents."
  The divergence list at `src/output/xml.rs:15-30` does not mention this.

The serialize-then-parse property test generates attribute values without whitespace
(`src/proptest.rs:28`, `:34`), so it does not cover this.

## Reproduction

```rust
// Cargo.toml: xot = "=0.31.2"
fn main() {
    let mut xot = xot::Xot::new();
    let root = xot.parse(r#"<r v="a&#9;b&#10;c&#13;d"/>"#).unwrap();
    let element = xot.document_element(root).unwrap();
    let v = xot.name("v").unwrap();
    assert_eq!(xot.get_attribute(element, v), Some("a\tb\nc\rd"));

    let out = xot.to_string(root).unwrap();
    println!("{out:?}");

    let mut again = xot::Xot::new();
    let root = again.parse(&out).unwrap();
    let element = again.document_element(root).unwrap();
    let v = again.name("v").unwrap();
    println!("{:?}", again.get_attribute(element, v));
}
```

Output:

```text
"<r v=\"a\tb\nc\rd\"/>"
Some("a b c d")
```

Expected: the second line is `Some("a\tb\nc\rd")`.

## Suggested fix

Add three arms to the match in `serialize_attribute` (`src/entity.rs:226-244`):

```rust
'\t' => {
    change = true;
    result.push_str("&#x9;")
}
'\n' => {
    change = true;
    result.push_str("&#xA;")
}
'\r' => {
    change = true;
    result.push_str("&#xD;")
}
```

With a unit test for `serialize_attribute`
and a `tests/roundtrip.rs` case `<root foo="a&#x9;b&#xA;c&#xD;d"/>`,
`cargo test` passes in full (485 passed, 0 failed, Rust 1.97.1),
and the reproduction prints `"<r v=\"a&#x9;b&#xA;c&#xD;d\"/>"` and `Some("a\tb\nc\rd")`.
Without the three arms the new unit test and roundtrip case fail.
The HTML5 serializer reuses `serialize_attribute` for attributes in a namespace
(`src/output/html5_serializer.rs:245-246`); its tests pass unchanged.

NEL and LINE SEPARATOR are left alone, since xot parses XML 1.0 only.

Related: #43 (Canonical XML).

Prepared with AI assistance;
the reproduction and the test runs in this report were executed against the v0.31.2 tag.
~~~

### Draft comment for kornelski/xml-rs#88 (additive)

The user chose to post this personally;
tracked in issue #547 (2026-09-17).

~~~md
The attribute half has a one-line fix:
`AttributeEscapes` (`src/escape.rs:81-90`) has arms for `\n` and `\r` but not `\t`.

```diff
     b'\n' => "&#xA;",
     b'\r' => "&#xD;",
+    b'\t' => "&#x9;",
 );
```

Tested on the `1.4.0` tag with the doc comment on `escape_str_attribute` updated,
`test_escape_str_attribute` extended with `\t`,
and this new test in `tests/event_writer.rs`:

```rust
#[test]
fn attribute_whitespace_survives_reading_back() {
    use xml::reader::XmlEvent as ReaderEvent;
    use xml::writer::XmlEvent;

    let value = "a\tb\nc\rd";
    let mut b = Vec::new();
    {
        let mut w = EmitterConfig::new()
            .write_document_declaration(false)
            .create_writer(&mut b);
        unwrap_all! {
            w.write(XmlEvent::start_element("hello").attr("test", value));
            w.write(XmlEvent::end_element())
        }
    }
    assert_eq!(str::from_utf8(&b).unwrap(), "<hello test=\"a&#x9;b&#xA;c&#xD;d\" />");

    let read_back = EventReader::new(b.as_slice())
        .into_iter()
        .find_map(|e| match e.unwrap() {
            ReaderEvent::StartElement { attributes, .. } => Some(attributes[0].value.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(read_back, value);
}
```

`cargo test` (Rust 1.97.1): 157 passed, 0 failed, 2 ignored doctests, including `xmlconf`.
Without the new arm, `test_escape_str_attribute` and the new test fail with the literal tab in the output.
This does not touch the character-data `\r` half of this issue.

Version history, from the same writer-to-reader check:
the escape table is unchanged from 1.3.1, where xml-rs reads its own tab back unchanged,
but `roxmltree` 0.21.1 (and any reader applying XML 1.0 section 3.3.3) already reads it as a space.
#80 (`4e76ad5`, first released in 1.4.0) made the mismatch visible inside xml-rs.

Downstream: `xmltree` 0.12.0 writes attributes through `EventWriter` unchanged,
so its `write_with_config` output has the same literal tab;
with this patch applied through `[patch.crates-io]`, xmltree output round-trips with no xmltree change.

The rule for serializers is spelled out in XSLT and XQuery Serialization 3.1, section 5
(https://www.w3.org/TR/xslt-xquery-serialization-31/#xml-output):
"CR, NL, TAB, NEL and LINE SEPARATOR characters in attribute nodes MUST be output respectively
as "&#xD;", "&#xA;", "&#x9;", "&#x85;", and "&#x2028;", or their equivalents."

Prepared with AI assistance;
the test runs in this comment were executed against the 1.4.0 tag.
~~~

[xml10-avn]: https://www.w3.org/TR/xml/#AVNormalize
[ser31-xml]: https://www.w3.org/TR/xslt-xquery-serialization-31/#xml-output
[xmlrs-79]: https://github.com/kornelski/xml-rs/issues/79
[xmlrs-80]: https://github.com/kornelski/xml-rs/pull/80
[xmlrs-88]: https://github.com/kornelski/xml-rs/issues/88
[xmlrs-90]: https://github.com/kornelski/xml-rs/pull/90
[netvl-153]: https://github.com/netvl/xml-rs/issues/153
[netvl-154]: https://github.com/netvl/xml-rs/pull/154
[xot-43]: https://github.com/faassen/xot/issues/43
[xmltree-21]: https://github.com/eminence/xmltree-rs/issues/21
[xmltree-47]: https://github.com/eminence/xmltree-rs/issues/47
[xmltree-59]: https://github.com/eminence/xmltree-rs/pull/59
