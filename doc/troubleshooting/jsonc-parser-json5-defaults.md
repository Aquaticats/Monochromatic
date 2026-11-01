# jsonc-parser 0.33.2 parses loose syntax by default, ignores `allow_comments: false` in the CST parser, and `set_value` rewrites equal numbers

## Symptom

The structured-edit research for `meow`
(`doc/planning/monorepo-manager-route-research/rust-structured-edits.md`,
sections "Probe results by candidate",
"JSONC options",
and "Documentation problems recorded")
recorded `jsonc-parser` 0.33.2 surprises covered here:
loose default parse options,
accepted scalar roots,
and an equal-value set that rewrote `1.0`.
This investigation found one more while checking them.
The research's `j07` finding (a deleted member's leading comment stays behind) is not covered.

### Default options accept syntax that is not JSONC

The crates.io description is `JSONC parser.`,
but `ParseOptions::default()` accepts all of these through `CstRootNode::parse`:

- unquoted property names:
  `{ a: 1 }`
- single-quoted names and strings:
  `{ 'a': 1 }`,
  `{ "a": 'x' }`
- hexadecimal and unary-plus numbers:
  `{ "a": 0x1F }`,
  `{ "a": +1 }`
- missing commas:
  `{ "a": 1 "b": 2 }`,
  `[1 2]`
- number and hyphenated words as property names:
  `{ 1: 2 }`,
  `{ a-b: 1 }`

The research described this as accepting JSON5.
It is not JSON5:
the same defaults reject JSON5's `.5`,
`5.`,
`Infinity`,
`NaN`,
and escaped line continuations,
and accept missing commas and `a-b` names that JSON5 rejects.

Scalar roots (`42`,
`"bare string"`),
empty input,
and comment-only input parse successfully under every option set,
including the JSONC-only and strict-JSON sets.

### `allow_comments: false` does not reject comments in the CST parser

With `ParseOptions { allow_comments: false, ..Default::default() }`:

```text
parse_to_value           "[ \"test\" ] // 1" -> ERR(Comments are not allowed on line 1 column 12)
parse_to_ast Separate    "[ \"test\" ] // 1" -> ERR(Comments are not allowed on line 1 column 12)
parse_to_ast AsTokens    "[ \"test\" ] // 1" -> ok
CstRootNode::parse       "[ \"test\" ] // 1" -> ok
```

So the rustdoc's "Parse Strictly as JSON" recipe (`src/lib.rs:99-121`)
still accepts `// c\n{ "a": 1 }` and `{ /* b */ "a": 1 }`
when the text goes through `CstRootNode::parse`,
the entry point for editing.

### An equal-value set rewrites `1.0` to `1`

Research case `j11` sets `a` to `1` in:

```text
{\n  "a": 1.0, // raw\n  "s": "x\u0041"\n}\n
```

The probe skipped the write when `prop.value().and_then(|v| v.to_serde_value()) == Some(wanted)`.
For `s` the guard held and no byte changed;
for `a` the guard failed and `set_value` produced:

```text
{\n  "a": 1, // raw\n  "s": "x\u0041"\n}\n
```

## Root cause

Line numbers refer to the published `jsonc-parser` 0.33.2 crate.
Upstream `main` is the `0.33.2` release commit (`e6e3837`),
and `diff --recursive --brief` between the crate's `src/` and that checkout reports no differences.

### Loose defaults are a documented choice

`src/parse_to_ast.rs:49-80` documents each option's default and sets every one to `true`:

```rust
/// Options for parsing.
#[derive(Clone)]
pub struct ParseOptions {
  /// Allow comments (defaults to `true`).
  pub allow_comments: bool,
  /// Allow words and numbers as object property names (defaults to `true`).
  pub allow_loose_object_property_names: bool,
  /// Allow trailing commas on object literal and array literal values (defaults to `true`).
  pub allow_trailing_commas: bool,
  /// Allow missing commas between object properties or array elements (defaults to `true`).
  pub allow_missing_commas: bool,
  /// Allow single-quoted strings (defaults to `true`).
  pub allow_single_quoted_strings: bool,
  /// Allow hexadecimal numbers like 0xFF (defaults to `true`).
  pub allow_hexadecimal_numbers: bool,
  /// Allow unary plus sign on numbers like +42 (defaults to `true`).
  pub allow_unary_plus_numbers: bool,
}

impl Default for ParseOptions {
  fn default() -> Self {
    Self {
      allow_comments: true,
      allow_loose_object_property_names: true,
      allow_trailing_commas: true,
      allow_missing_commas: true,
      allow_single_quoted_strings: true,
      allow_hexadecimal_numbers: true,
      allow_unary_plus_numbers: true,
    }
  }
}
```

The crate documentation says so directly (`src/lib.rs:99-102`):

```rust
//! ## Parse Strictly as JSON
//!
//! By default this library is extremely loose in what it allows parsing. To be strict,
//! provide `ParseOptions` and set all the options to false:
```

`README.md:6` and `src/lib.rs:3` describe the crate as
"A JSON parser and manipulator that supports comments and other JSON extensions";
only the one-line crates.io description (`Cargo.toml.orig:7`) says `JSONC parser.`.
The CST examples in `src/lib.rs:52` and `src/cst/mod.rs:1130` pass `&ParseOptions::default()`,
so code copied from them parses loosely.

### Why the loose set differs from JSON5

Object property names accept any word or number token (`src/parse_to_ast.rs:327-329`):

```rust
Some(Token::Word(prop_name)) | Some(Token::Number(prop_name)) => {
  properties.push(parse_object_property(context, PropName::Word(prop_name))?);
}
```

and only `allow_loose_object_property_names` gates them (`src/parse_to_ast.rs:372-378`).
Missing commas are rejected only when the option is off,
for objects at `src/parse_to_ast.rs:345` and for arrays at `:427`.
A bare word in value position is always an error (`src/parse_to_ast.rs:307`):

```rust
Token::Word(_) => Err(context.create_error(ParseErrorKind::UnexpectedWord)),
```

which is why `Infinity` and `NaN` fail under every option set.

The array half of `allow_missing_commas: false` was ignored before 0.33.1
([dprint/jsonc-parser#84][jsonc-84],
fixed by [#87][jsonc-87],
released in 0.33.1 on 2026-07-26),
so strict parsing needs at least 0.33.1.

### Scalar and empty roots have no option

`parse_value` (`src/parse_to_ast.rs:289-312`) returns `Ok(None)` for no token and accepts any scalar token at the root:

```rust
match context.token() {
  None => Ok(None),
  Some(token) => match token {
    Token::OpenBrace => Ok(Some(Value::Object(parse_object(context)?))),
    Token::OpenBracket => Ok(Some(Value::Array(parse_array(context)?))),
    Token::String(value) => Ok(Some(Value::StringLit(create_string_lit(context, value)))),
    Token::Boolean(value) => Ok(Some(Value::BooleanLit(create_boolean_lit(context, value)))),
    Token::Number(value) => Ok(Some(Value::NumberLit(create_number_lit(context, value)))),
    Token::Null => Ok(Some(Value::NullKeyword(create_null_keyword(context)))),
```

A scalar root is valid JSON (RFC 8259),
so this is not a defect;
the repository's JSONC definition rejects it
(`package/jsonc-edit/core-ts.conformance/src/jsonc.conformance.unit.test.ts`),
and the consumer has to check.

### The comment check is skipped when comments are collected as tokens

`CstRootNode::parse` (`src/cst/mod.rs:1146-1163`) always asks `parse_to_ast` for comments as tokens:

```rust
pub fn parse(text: &str, parse_options: &ParseOptions) -> Result<Self, ParseError> {
  let parse_result = parse_to_ast(
    text,
    &crate::CollectOptions {
      comments: crate::CommentCollectionStrategy::AsTokens,
      tokens: true,
    },
    parse_options,
  )?;
```

In `scan_handling_comments` (`src/parse_to_ast.rs:177-199`)
the `AsTokens` arm captures the comment and loops
before the arms that reach `handle_comment`:

```rust
fn scan_handling_comments(&mut self) -> Result<Option<Token<'a>>, ParseError> {
  loop {
    let token = self.scanner.scan()?;
    match token {
      Some(token @ Token::CommentLine(_) | token @ Token::CommentBlock(_)) if self.collect_comments_as_tokens => {
        self.capture_token(token);
      }
      Some(Token::CommentLine(text)) => {
        self.handle_comment(Comment::Line(CommentLine {
          range: self.create_range_from_last_token(),
          text,
        }))?;
      }
```

`handle_comment` (`src/parse_to_ast.rs:211-214`) is the only place this parser enforces the option:

```rust
fn handle_comment(&mut self, comment: Comment<'a>) -> Result<(), ParseError> {
  if !self.allow_comments {
    return Err(self.create_error(ParseErrorKind::CommentsNotAllowed));
  }
```

`parse_to_value` and `parse_to_serde_value` use the separate `JsoncParser`,
whose `scan` checks the option on every comment (`src/parser.rs:64-83`),
so they reject correctly.

The strict tests do not cover this path:
`assert_has_strict_error` (`src/parse_to_ast.rs:595-596`)
calls `parse_to_ast(text, &Default::default(), &strict_options())`,
and `CollectOptions::default()` collects no comments
(`src/parse_to_ast.rs:22-26`).

### `set_value` always replaces, and `serde_json` numbers compare by representation

`CstObjectProp::set_value` (`src/cst/mod.rs:1964-1983`) has no doc comment and no equality check;
it removes the current value node and inserts the replacement:

```rust
pub fn set_value(&self, replacement: CstInputValue) {
  let maybe_value = self.value();
  let mut value_index = maybe_value
    .as_ref()
    .map(|v| v.child_index())
    .unwrap_or_else(|| self.children().len());
  let container: CstContainerNode = self.clone().into();
  let indents = compute_indents(&container.clone().into());
  let style_info = &StyleInfo {
    newline_kind: container.root_node().map(|v| v.newline_kind()).unwrap_or_default(),
    uses_trailing_commas: uses_trailing_commas(maybe_value.unwrap_or_else(|| container.clone().into())),
  };
  self.remove_child_set_no_parent(value_index);
  container.raw_insert_value_with_internal_indent(
    Some(&mut value_index),
    InsertValue::Value(replacement),
    style_info,
    &indents,
  );
}
```

A number is inserted as raw text (`src/cst/input.rs:3-6`):

```rust
pub enum CstInputValue {
  Null,
  Bool(bool),
  Number(String),
```

`CstNumberLit::to_serde_value` (`src/cst/mod.rs:1521-1523`) parses the raw token with `serde_json::Number::from_str`,
so `1.0` becomes a float:

```rust
let num_for_parsing = raw.trim_start_matches('+');
match serde_json::Number::from_str(num_for_parsing) {
  Ok(number) => Some(serde_json::Value::Number(number)),
```

`serde_json` 1.0.151 compares numbers by internal variant (`src/number.rs:37-46`):

```rust
impl PartialEq for N {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (N::PosInt(a), N::PosInt(b)) => a == b,
            (N::NegInt(a), N::NegInt(b)) => a == b,
            (N::Float(a), N::Float(b)) => a == b,
            _ => false,
        }
    }
}
```

`Float(1.0)` against `PosInt(1)` is `false`,
the probe's guard called `set_value`,
and the probe's input conversion wrote `CstInputValue::Number("1")`.
Strings compare equal after decoding (`"x\u0041"` against `"xA"`),
which is why only `a` changed.

### Classification

- Loose defaults:
  documented default,
  not a bug.
  The field docs,
  the crate docs,
  and a maintainer reply
  ("Use `parseStrict` and then provide only what you want to turn on",
  [dsherret/jsonc-morph#7][jsonc-morph-7])
  all say the caller opts into strictness.
- The crates.io description `JSONC parser.`:
  wording.
  The README and crate docs already say "comments and other JSON extensions".
- Scalar and empty roots:
  valid JSON,
  not a bug;
  a consumer policy.
- `allow_comments: false` ignored by `CstRootNode::parse` and by `parse_to_ast` with `AsTokens`:
  bug.
  The option's documentation makes no exception for the collection strategy,
  and the other parser path enforces it.
- The `1.0` rewrite:
  consumer-side.
  `set_value` promises no equality check,
  and `serde_json` equality is representation-sensitive by design.

### Readings this investigation corrected

- "Default options accept JSON5" (`rust-structured-edits.md`,
  "J2" cons):
  the defaults accept a different loose set,
  per "Why the loose set differs from JSON5".
- "`jsonc-parser`:
  the crate description is 'JSONC parser.'
   while default options accept JSON5-style syntax"
  (`rust-structured-edits.md`,
  "Documentation problems recorded"):
  only the one-line description is terse;
  the rustdoc and field docs document the loose defaults.
- "J2 rewrites `1.0` on an equal-value set":
  the probe's `serde_json` equality guard caused the write,
  not `jsonc-parser`.
- The research wrapper's `same` compared every number with `as_f64`
  (`PR/rust/jsonc-parser-wrapper/src/main.rs` in that research's scratchpad).
  That guard skips a real change between integers above 2^53
  (harness line `f64-only big int` in "Equality and write catalog").

## Verification

Versions under test:

- `jsonc-parser` 0.33.2 from crates.io with features `cst`,
  `serde`,
  and `serde_json`,
  checksum `9ff5a48f48971be8e762a6ff955725a0802b6e46c441057992da5a673db9fd3a`,
  published 2026-09-12;
  upstream tag `0.33.2` is `e6e383704fd9a5b5e14d2de84cab0e3fcf187ecb`.
- `serde_json` 1.0.151.
- `rustc 1.97.1 (8bab26f4f 2026-07-14)` in `docker.io/library/rust:1.97-bookworm`
  (`sha256:389c1ae98c20fbcadca68a685482749267cec3c90893ae4671c5a37cc894c416`).
- Run on 2026-09-17.

### Harness

```toml
# jsonc-parser-defaults-repro/Cargo.toml
[package]
name = "jsonc_parser_defaults_repro"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
jsonc-parser = { version = "=0.33.2", features = ["cst", "serde", "serde_json"] }
serde_json = "=1.0.151"
```

```rust
// jsonc-parser-defaults-repro/src/main.rs
use jsonc_parser::ParseOptions;
use jsonc_parser::cst::{CstInputValue, CstRootNode};
use serde_json::{Number, Value, json};

/// JSONC as VS Code defines it: comments and trailing commas only.
fn jsonc_options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

/// The crate rustdoc's "Parse Strictly as JSON" recipe: every option false.
fn strict_json_options() -> ParseOptions {
    ParseOptions {
        allow_comments: false,
        allow_loose_object_property_names: false,
        allow_trailing_commas: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

/// Describes one parse outcome.
fn outcome(src: &str, options: &ParseOptions) -> String {
    match CstRootNode::parse(src, options) {
        Err(e) => format!("ERR({e})"),
        Ok(root) => {
            let kind = match root.value() {
                None => "no-value",
                Some(v) if v.as_object().is_some() => "object",
                Some(v) if v.as_array().is_some() => "array",
                Some(_) => "scalar",
            };
            format!("ok {kind}")
        }
    }
}

/// Parses a managed JSONC file: JSONC-only options, and the root must be an object or array.
fn parse_managed(src: &str) -> Result<CstRootNode, String> {
    let root = CstRootNode::parse(src, &jsonc_options()).map_err(|e| e.to_string())?;
    match root.value() {
        Some(value) if value.as_object().is_some() || value.as_array().is_some() => Ok(root),
        Some(_) => Err("root must be an object or array".to_owned()),
        None => Err("document has no value".to_owned()),
    }
}

/// Exact integer comparison when both sides are integers, otherwise f64 comparison.
fn same_number(a: &Number, b: &Number) -> bool {
    let a_int = a.is_i64() || a.is_u64();
    let b_int = b.is_i64() || b.is_u64();
    if a_int && b_int {
        return a == b;
    }
    a.as_f64() == b.as_f64()
}

/// JSON equality that treats `1` and `1.0` as the same number, recursively.
fn same(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => same_number(a, b),
        (Value::Array(a), Value::Array(b)) => a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same(x, y)),
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|w| same(v, w)))
        }
        (a, b) => a == b,
    }
}

/// Converts serde_json values into CST input, keeping number spelling from serde_json's Display.
fn input_of(value: &Value) -> CstInputValue {
    match value {
        Value::Null => CstInputValue::Null,
        Value::Bool(b) => CstInputValue::Bool(*b),
        Value::Number(n) => CstInputValue::Number(n.to_string()),
        Value::String(s) => CstInputValue::String(s.clone()),
        Value::Array(items) => CstInputValue::Array(items.iter().map(input_of).collect()),
        Value::Object(map) => CstInputValue::Object(map.iter().map(|(k, v)| (k.clone(), input_of(v))).collect()),
    }
}

/// Sets top-level key `key` to `wanted`, guarded by `guard`, and prints the result.
fn set_case(label: &str, src: &str, key: &str, wanted: Value, guard: &dyn Fn(&Value, &Value) -> bool) {
    let root = CstRootNode::parse(src, &jsonc_options()).expect("parse");
    let prop = root.object_value().expect("object").get(key).expect("key");
    let current = prop.value().and_then(|v| v.to_serde_value()).expect("value");
    let skipped = guard(&current, &wanted);
    if !skipped {
        prop.set_value(input_of(&wanted));
    }
    let out = root.to_string();
    println!("{label}: current={current} wanted={wanted} skipped={skipped} identical={} out={out:?}", out == src);
}

fn main() {
    let cases = [
        "{}", "{ \"a\": 1, }", "// c\n{ \"a\": 1 }", "{ /* b */ \"a\": 1 }",
        "{ a: 1 }", "{ 'a': 1 }", "{ \"a\": 'x' }", "{ \"a\": 0x1F }", "{ \"a\": +1 }",
        "{ \"a\": .5 }", "{ \"a\": 5. }", "{ \"a\": Infinity }", "{ \"a\": NaN }", "{ \"a\": \"line\\\ncontinued\" }",
        "{ \"a\": 1 \"b\": 2 }", "[1 2]", "{ 1: 2 }", "{ a-b: 1 }",
        "42", "\"bare string\"", "", "// only a comment",
        "{ \"a\": 1, , }", "{ \"a\": }", "{ \"a\": 1 ",
    ];
    for src in cases {
        println!(
            "{:<32} default={:<48} jsonc={}",
            format!("{src:?}"),
            outcome(src, &ParseOptions::default()),
            outcome(src, &jsonc_options())
        );
    }
    for src in cases {
        println!("strict-json {:<32} {}", format!("{src:?}"), outcome(src, &strict_json_options()));
    }
    for src in cases {
        let managed = parse_managed(src).map(|root| root.to_string() == src);
        println!("managed {:<32} {:?}", format!("{src:?}"), managed);
    }
    set_case("numeric long decimal", "{ \"a\": 0.1000000000000000055511151231257827 }", "a", json!(0.1), &same);

    println!("serde_json: json!(1) == json!(1.0) -> {}", json!(1) == json!(1.0));
    let src = "{\n  \"a\": 1.0, // raw\n  \"s\": \"x\\u0041\"\n}\n";
    let eq = |a: &Value, b: &Value| a == b;
    set_case("serde-eq a", src, "a", json!(1), &eq);
    set_case("serde-eq s", src, "s", json!("xA"), &eq);
    set_case("numeric a", src, "a", json!(1), &same);
    set_case("numeric a changed", src, "a", json!(2), &same);
    set_case("numeric exp", "{ \"a\": 1e2 }", "a", json!(100), &same);
    set_case("numeric big int", "{ \"a\": 9007199254740993 }", "a", json!(9007199254740992_u64), &same);
    set_case("numeric big float", "{ \"a\": 9007199254740993.0 }", "a", json!(9007199254740992_u64), &same);
    set_case("numeric neg zero", "{ \"a\": -0 }", "a", json!(0), &same);
    let f64_only = |a: &Value, b: &Value| match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (x, y) => x == y,
    };
    set_case("f64-only big int", "{ \"a\": 9007199254740993 }", "a", json!(9007199254740992_u64), &f64_only);
    let never = |_: &Value, _: &Value| false;
    set_case("unguarded same raw", "{ \"a\": 1.0 }", "a", serde_json::from_str("1.0").unwrap(), &never);
    let object_src = "{\n  \"o\": {\n    \"x\": 1,\n    // keep\n    \"y\": 2\n  }\n}\n";
    set_case("unguarded object", object_src, "o", json!({"x": 1, "y": 2}), &never);
    set_case("guarded object", object_src, "o", json!({"x": 1, "y": 2}), &same);
}
```

A second binary checks `allow_comments: false` per entry point and the two-pass workaround:

```rust
// jsonc-parser-defaults-repro/src/bin/comments.rs
use jsonc_parser::cst::CstRootNode;
use jsonc_parser::{CollectOptions, CommentCollectionStrategy, ParseOptions, parse_to_ast, parse_to_serde_value, parse_to_value};

/// Prints `ok` or the error for one entry point.
fn show<T, E: std::fmt::Display>(label: &str, src: &str, result: Result<T, E>) {
    match result {
        Ok(_) => println!("{label:<24} {src:?} -> ok"),
        Err(e) => println!("{label:<24} {src:?} -> ERR({e})"),
    }
}

/// Strict JSON for CST editing: validate with `parse_to_value`, which enforces `allow_comments`, then build the CST.
fn parse_strict_json_cst(src: &str, options: &ParseOptions) -> Result<CstRootNode, jsonc_parser::errors::ParseError> {
    parse_to_value(src, options)?;
    CstRootNode::parse(src, options)
}

fn main() {
    let strict_json = ParseOptions {
        allow_comments: false,
        allow_loose_object_property_names: false,
        allow_trailing_commas: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    };
    for src in ["{ \"a\": 1 }", "{ \"a\": 1 } // c", "{ /* c */ \"a\": 1 }", "{ \"a\": 1, }"] {
        show("two-pass strict CST", src, parse_strict_json_cst(src, &strict_json).map(|root| root.to_string()));
    }
    let no_comments = ParseOptions { allow_comments: false, ..Default::default() };
    let collect = |comments: CommentCollectionStrategy, tokens: bool| CollectOptions { comments, tokens };
    for src in ["[ \"test\" ] // 1", "[ \"test\" /* 1 */]", "// c\n{ \"a\": 1 }"] {
        show("parse_to_value", src, parse_to_value(src, &no_comments));
        show("parse_to_serde_value", src, parse_to_serde_value::<serde_json::Value>(src, &no_comments));
        show("parse_to_ast Off", src, parse_to_ast(src, &collect(CommentCollectionStrategy::Off, false), &no_comments));
        show("parse_to_ast Separate", src, parse_to_ast(src, &collect(CommentCollectionStrategy::Separate, false), &no_comments));
        show("parse_to_ast AsTokens", src, parse_to_ast(src, &collect(CommentCollectionStrategy::AsTokens, true), &no_comments));
        show("CstRootNode::parse", src, CstRootNode::parse(src, &no_comments));
    }
}
```

### Harness command

```sh
podman run --rm --memory=2g --cpus=2 \
  --volume "${HARNESS}:/work:Z" --workdir /work \
  docker.io/library/rust:1.97-bookworm cargo run --quiet --jobs 2 --bin jsonc_parser_defaults_repro

podman run --rm --memory=2g --cpus=2 \
  --volume "${HARNESS}:/work:Z" --workdir /work \
  docker.io/library/rust:1.97-bookworm cargo run --quiet --jobs 2 --bin comments
```

### Accepted by every option set

Default,
JSONC-only,
and strict-JSON options all returned `ok`:

- `{}`
- `42` and `"bare string"` (`ok scalar`)
- empty input (`ok no-value`)

Accepted by default and JSONC-only options,
and also by strict-JSON options because of the `allow_comments` bug:

- `// c\n{ "a": 1 }`
- `{ /* b */ "a": 1 }`
- `// only a comment` (`ok no-value`)

Accepted by default and JSONC-only options,
rejected by strict-JSON options:

- `{ "a": 1, }`:
  `ERR(Trailing commas are not allowed on line 1 column 9)`

### Accepted only by default options

Each row is the default result,
then the error under both JSONC-only and strict-JSON options:

- `{ a: 1 }`:
  `ok object`,
  `Expected string for object property on line 1 column 3`
- `{ 1: 2 }` and `{ a-b: 1 }`:
  `ok object`,
  `Expected string for object property on line 1 column 3`
- `{ 'a': 1 }`:
  `ok object`,
  `Single-quoted strings are not allowed on line 1 column 3`
- `{ "a": 'x' }`:
  `ok object`,
  `Single-quoted strings are not allowed on line 1 column 8`
- `{ "a": 0x1F }`:
  `ok object`,
  `Hexadecimal numbers are not allowed on line 1 column 8`
- `{ "a": +1 }`:
  `ok object`,
  `Unary plus on numbers is not allowed on line 1 column 8`
- `{ "a": 1 "b": 2 }`:
  `ok object`,
  `Expected comma on line 1 column 9`
- `[1 2]`:
  `ok array`,
  `Expected comma on line 1 column 3`

### Rejected by every option set

- `{ "a": .5 }`:
  `Unexpected token on line 1 column 8`
- `{ "a": 5. }`:
  `Expected digit on line 1 column 10`
- `{ "a": Infinity }` and `{ "a": NaN }`:
  `Unexpected word on line 1 column 8`
- `{ "a": "line\` followed by a newline and `continued" }`:
  `Invalid escape on line 1 column 13`
- `{ "a": 1, , }`:
  `Unexpected token in object on line 1 column 11`
- `{ "a": }`:
  `Unexpected close brace on line 1 column 8`
- `{ "a": 1 ` (unterminated):
  `Unterminated object on line 1 column 1`

### `allow_comments: false` by entry point

For each of `[ "test" ] // 1`,
`[ "test" /* 1 */]`,
and `// c\n{ "a": 1 }`,
`parse_to_value`,
`parse_to_serde_value`,
`parse_to_ast` with `Off`,
and `parse_to_ast` with `Separate` returned `Comments are not allowed`
(columns 12,
10,
and 1),
while `parse_to_ast` with `AsTokens` and `CstRootNode::parse` returned `ok`:

```text
parse_to_value           "[ \"test\" /* 1 */]" -> ERR(Comments are not allowed on line 1 column 10)
parse_to_ast Off         "[ \"test\" /* 1 */]" -> ERR(Comments are not allowed on line 1 column 10)
parse_to_ast AsTokens    "[ \"test\" /* 1 */]" -> ok
CstRootNode::parse       "[ \"test\" /* 1 */]" -> ok
parse_to_ast Separate    "// c\n{ \"a\": 1 }" -> ERR(Comments are not allowed on line 1 column 1)
parse_to_ast AsTokens    "// c\n{ \"a\": 1 }" -> ok
CstRootNode::parse       "// c\n{ \"a\": 1 }" -> ok
```

### Equality and write catalog

Writes that change bytes:

```text
serde-eq a: current=1.0 wanted=1 skipped=false identical=false out="{\n  \"a\": 1, // raw\n  \"s\": \"x\\u0041\"\n}\n"
numeric a changed: current=1.0 wanted=2 skipped=false identical=false out="{\n  \"a\": 2, // raw\n  \"s\": \"x\\u0041\"\n}\n"
numeric big int: current=9007199254740993 wanted=9007199254740992 skipped=false identical=false out="{ \"a\": 9007199254740992 }"
unguarded object: current={"x":1,"y":2} wanted={"x":1,"y":2} skipped=false identical=false out="{\n  \"o\": {\n    \"x\": 1,\n    \"y\": 2\n  }\n}\n"
```

The `unguarded object` line shows `set_value` dropping `// keep` when the replacement object has the same content.

Writes skipped or byte-identical:

```text
serde_json: json!(1) == json!(1.0) -> false
serde-eq s: current="xA" wanted="xA" skipped=true identical=true out=...
numeric a: current=1.0 wanted=1 skipped=true identical=true out=...
numeric exp: current=100.0 wanted=100 skipped=true identical=true out="{ \"a\": 1e2 }"
numeric neg zero: current=-0.0 wanted=0 skipped=true identical=true out="{ \"a\": -0 }"
numeric long decimal: current=0.1 wanted=0.1 skipped=true identical=true out="{ \"a\": 0.1000000000000000055511151231257827 }"
unguarded same raw: current=1.0 wanted=1.0 skipped=false identical=true out="{ \"a\": 1.0 }"
guarded object: current={"x":1,"y":2} wanted={"x":1,"y":2} skipped=true identical=true out=...
```

`unguarded same raw` shows `set_value` itself preserves bytes when given the same raw spelling.

Edge cases that name the guard's limits:

```text
numeric big float: current=9007199254740994.0 wanted=9007199254740992 skipped=false identical=false out="{ \"a\": 9007199254740992 }"
f64-only big int: current=9007199254740993 wanted=9007199254740992 skipped=true identical=true out="{ \"a\": 9007199254740993 }"
```

`9007199254740993.0` parsed as `9007199254740994.0`,
one representable double away from the correctly rounded `9007199254740992.0`;
`serde_json` documents its default float parsing as best-effort unless the `float_roundtrip` feature is enabled
(`Cargo.toml.orig:67-74` of `serde_json` 1.0.151).

## Verified workarounds

### Parse managed JSONC through one entry point with a root check

Use `parse_managed` from "Harness" for every managed file.
Its `managed` lines accepted exactly the JSONC set
(`{}`,
`{ "a": 1, }`,
`// c\n{ "a": 1 }`,
`{ /* b */ "a": 1 }`,
each `Ok(true)`,
meaning the CST round-trips byte for byte),
and rejected every other case in "Harness",
including `42`,
`"bare string"`,
empty input,
and `// only a comment`
(`Err("root must be an object or array")` or `Err("document has no value")`).

Tradeoffs:

- Files other tools accept as JSONC with loose syntax
  (unquoted keys,
  missing commas)
  fail with a diagnostic instead of being edited.
- Trailing commas are allowed,
  so this is JSONC as `tsconfig.json` uses it,
  not strict JSON.
- It depends on 0.33.1 or later for arrays with missing commas ([#84][jsonc-84]);
  pin the minimum version.
- The option set must live in one function;
  any direct `ParseOptions::default()` call reintroduces the loose set.

### Guard every write with numeric-aware equality

Call `set_value` only when `same(&current, &wanted)` is false,
with `same` and `same_number` from "Harness".
`numeric a`,
`numeric exp`,
and `guarded object` stayed byte-identical,
while `numeric a changed` and `numeric big int` still wrote.

Tradeoffs:

- Numbers with a fraction or exponent compare as `f64` after `serde_json`'s best-effort parse:
  `-0` equals `0`,
  a long decimal equal to `0.1` in `f64` is skipped,
  and a fractional spelling of an integer above 2^53 may compare unequal to the intended value.
- Object equality ignores key order,
  so reordering keys alone is not written.
- A guarded write of a container that did change still replaces the whole subtree through `set_value`,
  dropping comments inside it;
  set leaf paths instead of whole objects when comments inside matter.

### Validate strict JSON before building the CST

When strict JSON is required from `CstRootNode::parse`,
parse once with `parse_to_value` first;
it enforces `allow_comments: false`.
`parse_strict_json_cst` in the `comments` binary printed:

```text
two-pass strict CST      "{ \"a\": 1 }" -> ok
two-pass strict CST      "{ \"a\": 1 } // c" -> ERR(Comments are not allowed on line 1 column 12)
two-pass strict CST      "{ /* c */ \"a\": 1 }" -> ERR(Comments are not allowed on line 1 column 3)
two-pass strict CST      "{ \"a\": 1, }" -> ERR(Trailing commas are not allowed on line 1 column 9)
```

Tradeoffs:

- Every file is parsed twice,
  once by each of the crate's two parser implementations.
- If the implementations disagree on some other input,
  the first parse decides acceptance while the CST decides editing.
- Not needed once the "Prototype fix" ships.

## What does not work

- **`ParseOptions::default()` as the CST examples use it.**
  It accepts unquoted and numeric names,
  single quotes,
  hexadecimal and unary-plus numbers,
  and missing commas ("Accepted only by default options").
- **The rustdoc's "Parse Strictly as JSON" recipe for JSONC files.**
  It rejects trailing commas (`{ "a": 1, }`),
  which JSONC files such as `tsconfig.json` use,
  and through `CstRootNode::parse` it still accepts comments.
- **Relying on any `allow_*` option to reject scalar or empty documents.**
  `42`,
  `""`,
  and comment-only input parse under every option set.
- **Comparing `to_serde_value()` results with `==`.**
  `serde-eq a` wrote `1` over `1.0`.
- **Comparing every number with `as_f64`.**
  `f64-only big int` skipped a real change from `9007199254740993` to `9007199254740992`.
- **Calling `set_value` with an equal object to "refresh" a value.**
  `unguarded object` dropped the `// keep` comment inside it.

## Upstream filing decision

`.out-of-scope/` was checked:
 it has no entry for `jsonc-parser`,
 dprint,
 or JSONC tooling.
Nothing was filed or commented upstream from this session.

### Duplicate search

Searched 2026-09-17 with `gh search issues --repo dprint/jsonc-parser --include-prs` and `gh search prs`,
 open and closed,
 for an empty query (all 88 issues and pull requests listed and read by title),
 `json5`,
 `strict`,
 `default options`,
 `set_value`,
 `number`,
 `description`,
 `allow_comments`,
 `comments not allowed`,
 and `comments`.
Threads read in full:

- [#18][jsonc-18] (closed,
   2021):
   "Change the repository name to json5-parser".
  Closed by its author with no comment;
   no maintainer response.
- [#38][jsonc-38] (closed):
   asks whether the crate is "more of a JSON5 parser than a JSONC one" while reporting `+42` failing;
   the response was to support unary plus ([#58][jsonc-58]),
   later made optional ([#61][jsonc-61]).
- [#25][jsonc-25] (merged):
   "feat:
   ability to parse strictly as JSON",
   which added the `allow_*` switches.
- [#63][jsonc-63] (merged):
   added `allow_missing_commas`,
   referencing [dsherret/jsonc-morph#7][jsonc-morph-7],
   where the maintainer answered a "too lenient" report with
   "Use `parseStrict` and then provide only what you want to turn on".
- [#84][jsonc-84] and [#87][jsonc-87] (closed and merged the same day,
   2026-07-26):
   `allow_missing_commas: false` ignored for array elements,
   a report with the same shape as the comment bug here;
   the maintainer fixed both parser implementations and released 0.33.1.

No issue or pull request reports `allow_comments: false` being ignored by `CstRootNode::parse` or `AsTokens`,
 asks for stricter defaults,
 or discusses `set_value` equality.

### Loose defaults, `set_value` equality, and the crate description

Decision:
 do not file.
Constraint 1 fails for every item,
 and constraint 5 fails for a default change.

1.  **Upstream's fault:**
    no.
    The defaults are documented behavior (`src/parse_to_ast.rs:49-80`,
     `src/lib.rs:99-102`);
     the `1.0` rewrite comes from the caller's `serde_json` equality;
     scalar roots are valid JSON.
    Only the crates.io description is terse,
     and that is wording the README and crate docs already correct.
2.  **Upstream can fix it:**
    yes,
     technically:
     changing defaults,
     the description,
     or adding an equality short-circuit to `set_value` is possible.
3.  **Supported use case:**
    yes.
    CST editing and strict parsing are both documented.
4.  **Contribution welcome:**
    yes.
    The repository has no `CONTRIBUTING.md`,
     no issue or pull request templates,
     and no AI-assistance policy
     (`.github/` holds `CODE_OF_CONDUCT.md`,
     `SECURITY.md`,
     and `workflows/`;
     the community profile API reports `contributing: null`);
     no ban was found.
5.  **Likely to fix:**
    no for stricter defaults:
     the maintainer's direct guidance in jsonc-morph#7 is to opt in to strictness,
     and #38 led to more loose syntax,
     not less.
    No signal either way for the description.
6.  **Prototype:**
    not triggered,
     because constraints 1 and 5 fail.

### Draft (do not file as-is)

Kept as an auditable record in case upstream signals interest in a clearer crates.io description.

~~~md
Title: crates.io description "JSONC parser." hides that default ParseOptions accept non-JSONC syntax
Labels: documentation

The crates.io description (`Cargo.toml`: `description = "JSONC parser."`) is the first thing search results show,
but `ParseOptions::default()` accepts unquoted and numeric property names, single-quoted strings, hexadecimal and
unary-plus numbers, and missing commas (`src/parse_to_ast.rs:68-80`). The README and crate docs say "supports comments
and other JSON extensions", and `src/lib.rs:99-102` explains how to parse strictly, but the CST examples
(`src/lib.rs:52`, `src/cst/mod.rs:1130`) use `&ParseOptions::default()`, so code copied from them parses loosely.

Suggested wording: "JSON parser and CST manipulator with comments and optional JSON extensions (loose by default)",
plus a JSONC-only `ParseOptions` example (comments and trailing commas on, everything else off) next to the strict
JSON one.
~~~

### `allow_comments: false` ignored by `CstRootNode::parse`

Decision:
 all six constraints hold,
 so the draft in "Draft issue" is fileable once a human completes "Before filing".

1.  **Upstream's fault:**
    yes.
    This is behavior:
     `scan_handling_comments` captures comment tokens before the only `allow_comments` check
     (`src/parse_to_ast.rs:181-183` against `:211-214`),
     while the documented option (`src/parse_to_ast.rs:52-53`) has no exception
     and the other parser implementation enforces it (`src/parser.rs:70-78`).
2.  **Upstream can fix it:**
    yes;
    "Prototype fix" adds the check to the `AsTokens` arm.
3.  **Supported use case:**
    yes.
    `CstRootNode::parse` takes `ParseOptions`,
     the crate docs present every-option-false as strict JSON (`src/lib.rs:99-121`),
     and `src/cst/mod.rs:4792-4805` already tests another strict option (`allow_missing_commas`) through the CST.
4.  **Contribution welcome:**
    yes;
    same evidence as the "Loose defaults" item:
     no contributing guide,
     templates,
     or AI policy,
     so no ban was found.
5.  **Likely to fix:**
    yes.
    [#84][jsonc-84],
     the same class for `allow_missing_commas`,
     was reported on 2026-07-25 and fixed by the maintainer in #87,
     which merged and shipped as 0.33.1 on 2026-07-26.
6.  **Prototype:**
    yes;
    see "Prototype fix".

### Prototype fix

Patch:
 [`jsonc-parser-json5-defaults.patch`](jsonc-parser-json5-defaults.patch),
 against `dprint/jsonc-parser` `main` at `e6e383704fd9a5b5e14d2de84cab0e3fcf187ecb` (tag `0.33.2`;
 `git apply --check` passes on a clean clone of that commit).
It adds `parse_to_ast::tests::strict_should_error_comments_collected_as_tokens`,
 `cst::test::comments_when_not_allowed`,
 and this check:

```diff
--- a/src/parse_to_ast.rs
+++ b/src/parse_to_ast.rs
@@ -179,6 +179,9 @@ impl<'a> Context<'a> {
       let token = self.scanner.scan()?;
       match token {
         Some(token @ Token::CommentLine(_) | token @ Token::CommentBlock(_)) if self.collect_comments_as_tokens => {
+          if !self.allow_comments {
+            return Err(self.create_error(ParseErrorKind::CommentsNotAllowed));
+          }
           self.capture_token(token);
         }
         Some(Token::CommentLine(text)) => {
```

Build bounds:
 a fresh `mktemp --directory "${HOME}/temp/agent/upstream-prototype.XXXXXXXX"` clone
 (origin `https://github.com/dprint/jsonc-parser.git`,
 push URL disabled,
 HEAD verified as `e6e3837`),
 built and tested only in `podman run --rm --memory=2g --cpus=2` with `docker.io/library/rust:1.97-bookworm`,
 only the prototype directory mounted (clone,
 a throwaway `CARGO_HOME`,
 and target directory),
 no credentials in the environment,
 and `RUSTUP_TOOLCHAIN=1.97.1`
 (upstream's `rust-toolchain.toml` pins `1.94.0`).

Test command,
 run once with only the tests added (without `CARGO_INCREMENTAL=0`) and again with the fix applied:

```sh
podman run --rm --memory=2g --cpus=2 \
  --volume "${PROTOTYPE}:/proto:Z" --workdir /proto/jsonc-parser \
  --env CARGO_HOME=/proto/cargo-home --env CARGO_TARGET_DIR=/proto/target \
  --env RUSTUP_TOOLCHAIN=1.97.1 --env CARGO_INCREMENTAL=0 \
  docker.io/library/rust:1.97-bookworm cargo test --all-features --jobs 2
```

Tests only:

```text
test cst::test::comments_when_not_allowed ... FAILED
test parse_to_ast::tests::strict_should_error_comments_collected_as_tokens ... FAILED
thread 'cst::test::comments_when_not_allowed' (1063) panicked at src/cst/mod.rs:4817:54:
called `Option::unwrap()` on a `None` value
thread 'parse_to_ast::tests::strict_should_error_comments_collected_as_tokens' (1134) panicked at src/parse_to_ast.rs:573:18:
Expected error, but did not find one.
test result: FAILED. 160 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Patched,
 every command in upstream's `.github/workflows/ci.yml` test jobs:

- `cargo test --all-features`:
   162 unit,
   1 `sort_fuzz`,
   2 `tests/test.rs`,
   14 doctests passed.
- `cargo test --release --all-features`:
   162,
   1,
   2,
   and 14 passed.
- `cargo test --features serde`:
   114 unit,
   1 `tests/test.rs`,
   7 doctests passed.
- `cargo test --features preserve_order`:
   94 unit,
   1 `tests/test.rs`,
   7 doctests passed.

User-boundary check:
 the "Harness" crate with `[patch.crates-io] jsonc-parser = { path = "/patched" }`,
 the patched clone mounted read-only,
 printed `Comments are not allowed` for all 18 entry-point and input combinations of the `comments` binary.
The main binary's output differed from the unpatched run only in the three `strict-json` comment lines:

```text
strict-json "// c\n{ \"a\": 1 }"             ERR(Comments are not allowed on line 1 column 1)
strict-json "{ /* b */ \"a\": 1 }"           ERR(Comments are not allowed on line 1 column 3)
strict-json "// only a comment"              ERR(Comments are not allowed on line 1 column 1)
```

The reproducer in "Draft issue" also ran as written against unpatched 0.33.2
 (as a third binary of the "Harness" crate)
 and failed only at the second assert:

```text
thread 'main' (1) panicked at src/bin/draft.rs:11:5:
assertion failed: CstRootNode::parse("[1] // 1", &options).is_err()
```

Not run:
 `cargo fmt` and `cargo clippy`
 (the image has no `rustfmt` or `clippy` component),
 and the benchmark job.

### Before filing

A human must personally:

- rerun the reproduction in "Draft issue" against the latest `jsonc-parser` release;
- open `src/parse_to_ast.rs` on current `main` and confirm the `AsTokens` arm still skips `handle_comment`;
- apply the patch,
   run `cargo fmt --check` and the four `cargo test` commands from `.github/workflows/ci.yml`;
- remove any clause of the draft's footer describing a check they did not do.

### Draft issue

~~~md
Title: `allow_comments: false` is ignored by `CstRootNode::parse` and `parse_to_ast` with `CommentCollectionStrategy::AsTokens`
Labels: bug

## Reproducer

jsonc-parser 0.33.2 with the `cst` feature.

```rust
use jsonc_parser::cst::CstRootNode;
use jsonc_parser::{parse_to_value, ParseOptions};

fn main() {
    let options = ParseOptions { allow_comments: false, ..Default::default() };

    // Rejects correctly.
    assert!(parse_to_value("[1] // 1", &options).is_err());

    // Does not. This assert fails.
    assert!(CstRootNode::parse("[1] // 1", &options).is_err());
}
```

`parse_to_value`, `parse_to_serde_value`, and `parse_to_ast` with `CommentCollectionStrategy::Off` or `Separate`
return a "Comments are not allowed" error. `parse_to_ast` with `AsTokens` and `CstRootNode::parse` return `Ok`, for
line and block comments alike, so the "Parse Strictly as JSON" options in the crate docs still accept comments
through the CST.

## Cause

`Context::scan_handling_comments` (`src/parse_to_ast.rs:181-183`) captures comment tokens when
`collect_comments_as_tokens` is set and loops, before the arms that call `handle_comment`, which is the only place
this parser checks `allow_comments` (`:211-214`). `CstRootNode::parse` always uses `AsTokens`
(`src/cst/mod.rs:1150`). `JsoncParser::scan` checks the option on every comment (`src/parser.rs:70-78`), so the
value and serde paths are fine. `assert_has_strict_error` uses `CollectOptions::default()`, so no strict test
covers `AsTokens`.

## Suggested fix

```diff
--- a/src/parse_to_ast.rs
+++ b/src/parse_to_ast.rs
@@ -179,6 +179,9 @@ impl<'a> Context<'a> {
       let token = self.scanner.scan()?;
       match token {
         Some(token @ Token::CommentLine(_) | token @ Token::CommentBlock(_)) if self.collect_comments_as_tokens => {
+          if !self.allow_comments {
+            return Err(self.create_error(ParseErrorKind::CommentsNotAllowed));
+          }
           self.capture_token(token);
         }
         Some(Token::CommentLine(text)) => {
```

With `strict_should_error_comments_collected_as_tokens` in `src/parse_to_ast.rs` and `comments_when_not_allowed`
in `src/cst/mod.rs` (modeled on `missing_comma_between_array_elements`), both tests fail before this change and pass
after it, and the four `cargo test` commands from `ci.yml` pass. Same class as #84. I can open this as a pull request.

---

Written by an agent (Claude Code, claude-opus-5).
A human reran the reproduction, checked the cited source lines, and ran the patched test suite before filing.
~~~

[jsonc-18]: https://github.com/dprint/jsonc-parser/issues/18
[jsonc-25]: https://github.com/dprint/jsonc-parser/pull/25
[jsonc-38]: https://github.com/dprint/jsonc-parser/issues/38
[jsonc-58]: https://github.com/dprint/jsonc-parser/pull/58
[jsonc-61]: https://github.com/dprint/jsonc-parser/pull/61
[jsonc-63]: https://github.com/dprint/jsonc-parser/pull/63
[jsonc-84]: https://github.com/dprint/jsonc-parser/issues/84
[jsonc-87]: https://github.com/dprint/jsonc-parser/pull/87
[jsonc-morph-7]: https://github.com/dsherret/jsonc-morph/issues/7
