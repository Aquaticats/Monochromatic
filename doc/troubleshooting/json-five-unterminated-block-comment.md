# json-five 0.3.1 round-trip parser drops every block comment's closing slash and panics on multibyte line-comment ends

## Symptom

[`json-five`][json-five-rs] 0.3.1 advertises
"Supports round-trip use cases with preservation/editing of whitespace and comments"
(`README.md:10`).
Its round-trip model (`json_five::rt::parser::from_str` plus `to_string`)
writes every block comment without its closing `/`,
even when nothing was edited.
The output is no longer valid JSON5 or JSONC.

The structured-edit research for `meow`
(`doc/planning/monorepo-manager-route-research/rust-structured-edits.md`,
section "Probe results by candidate",
 subsection "JSONC")
found it through case `j08`,
which sets `compilerOptions.strict` inside a `tsconfig`-like object:

```text
input:  {\n  "compilerOptions": {\n    /* strictness */\n    "strict": false, // todo\n  },\n}\n
output: {\n  "compilerOptions": {\n    /* strictness *\n    "strict": true, // todo\n  },\n}\n
```

Parsing that output again fails.
`json-five` itself reports:

```text
ParsingError { index: 29, message: "Unexpected end of input while processing block comment", lineno: 3, colno: 5, char_index: 29 }
```

The repository's `module-jsonc-edit` parser
(`package/jsonc-edit/core-ts/src/scan.ts:403`)
reports:

```text
JsoncParseError: unterminated block comment (at offset 29)
```

The same loss happens without any edit.
The research's conformance corpus round-tripped 9 of 10 valid inputs;
the failure was `{ /* block */ "a": 1 }`,
which comes back as `{ /* block * "a": 1 }`.

Surface patterns and the failure each produces:

- Any `/* ... */` comment anywhere in the document
  (leading,
   trailing,
   between members,
   inside arrays,
   `/** ... **/` style):
  - `rt::parser::from_str(src)?.to_string()` returns the text with the comment's final `/` removed;
    reparsing fails with "Unexpected end of input while processing block comment".
  - `json_five::source_to_tokens(src)` returns a `BlockComment` token whose `lexeme` lacks the `/`,
    shifts every later lexeme by one byte,
    and `tokens_to_source` drops the document's last byte instead
    (for example `{ /* block */ "a": 1 }` becomes `{ /* block */ "a": 1 `).
- A `//` comment whose last character before end of input is multibyte
  (`{"a": 1} // café`),
  or a `//` comment terminated by U+2028 or U+2029:
  - `rt::parser::from_str` panics with
    `end byte index 16 is not a char boundary; it is inside 'é' (bytes 15..17 of string)`
    at `src/rt/parser.rs:371:21`.
  - `source_to_tokens` panics with
    `internal error: entered unreachable code: Unexpected end of document`
    at `src/rt/tokenize.rs:155:5`.

`json_five::from_str` (serde) and `json_five::model_from_str` (the non-round-trip model) accept all of these inputs;
they discard comments,
so they never slice comment text.

## Root cause

Line numbers refer to the published `json-five` 0.3.1 crate.
`src/tokenize.rs` and everything under `src/rt/` are byte-identical at upstream `main` (`650be6a`,
 2026-05-13);
`diff --recursive --brief` between the crate and that checkout reports only `src/ser.rs`.

### Step 1: token spans are documented as end-exclusive

`src/tokenize.rs:36-37`:

```rust
// Start byte offset, token type, end byte offset (noninclusive)
pub(crate) type TokenSpan = (usize, TokType, usize);
```

Every single-byte token follows that contract,
for example `src/tokenize.rs:565`:

```rust
'{' => Ok((next_idx, TokType:: LeftBrace, next_idx + 1)),
```

### Step 2: `process_comment` returns the index of the closing `/` as the end

`src/tokenize.rs:529-532`,
inside the block-comment loop of `process_comment`:

```rust
'/' => {
    (last_idx, _) = self.advance().unwrap();
    return Ok((start_idx, TokType::BlockComment, last_idx))
}
```

`self.advance()` returns the `(byte index, char)` of the `/` it just consumed,
so `last_idx` is the start of the `/`,
and the span ends one byte early.
The tokenizer has already consumed the `/`,
so the next token starts after it and nothing else covers that byte.

The line-comment branch at `src/tokenize.rs:489-509` adds `1` instead,
which is right only when the last consumed character is one byte long:

```rust
'/' => {
    // line comment
    loop {
        match self.chars.peek() {
            None => {
                return Ok((start_idx, TokType::LineComment, last_idx+1))
            },
            Some((peeked_idx, peeked_char)) => {
                match peeked_char {
                    '\n' | '\r' | '\u{2028}' | '\u{2029}' => {
                        (last_idx, _) = self.advance().unwrap();
                        return Ok((start_idx, TokType::LineComment, last_idx+1))
                    }
                    _ => {
                        last_idx = *peeked_idx;
                        self.advance();
                    }
                }
            }
        }
    }
},
```

`last_idx` is the start of the last character.
For `é` (2 bytes) at end of input,
or for a U+2028 or U+2029 terminator (3 bytes each),
`last_idx+1` falls inside that character.

### Step 3: only the round-trip tokenizer keeps comment spans

`src/tokenize.rs:592-602` discards the span unless comments are requested:

```rust
'/' => {
    let (_, next_next) = self.chars.peek().unwrap_or(&(usize::MAX, '!'));
    match next_next {
        '/' | '*' => {
            if self.configuration.include_comments {
                self.process_comment()
            } else {
                self.process_comment()?;
                self.next_token()
            }
        },
```

`tokenize_rt_str` (`src/tokenize.rs:658-661`) is the only public entry point that sets `include_comments: true`:

```rust
pub fn tokenize_rt_str(text: &'_ str) -> Result<Tokens<'_>, TokenizationError> {
    let config = TokenizerConfig{include_comments: true, include_whitespace: true, allow_octal: false};
    Tokenizer::with_configuration(text, config).tokenize()
}
```

That is why `json_five::from_str` and `model_from_str` are unaffected,
while both round-trip APIs are.

### Step 4: the model stores comment text sliced from the short span

`rt::parser::from_str` (`src/rt/parser.rs:714-724`) calls `tokenize_rt_str`.
`consume_whitespace_and_comments` (`src/rt/parser.rs:690-706`) collects `BlockComment`,
`LineComment`,
and `Whitespace` spans,
and `collect_wsc_vec_to_string` (`src/rt/parser.rs:523-540`) turns them into the context strings the model serializes:

```rust
fn collect_wsc_vec_to_string(&self, wsc: &Vec<&'toks TokenSpan>) -> String {
    if wsc.is_empty() {
        return String::with_capacity(0);
    }

    let first = wsc.first().unwrap();
    if wsc.len() == 1 {
        self.get_tok_source(first).to_string()
    } else {
        let last = wsc.last().unwrap();
        let mut buff = String::with_capacity(last.2 - first.0);
        for span in wsc {
            let src = self.get_tok_source(span);
            buff.push_str(src);
        }
        buff
    }
}
```

`get_tok_source` (`src/rt/parser.rs:370-372`) slices by the span:

```rust
fn get_tok_source(&self, span: &'toks TokenSpan) -> &'input str {
    &self.source[span.0 .. span.2]
}
```

So a block comment is stored as `/* strictness *`,
and `to_string` writes it back that way.
With a multibyte last character the same slice is not on a character boundary,
which is the panic at `src/rt/parser.rs:371:21`.

### Step 5: the owned-token API sizes its buffers from the same spans

`source_to_tokens` (`src/rt/tokenize.rs:62-156`) walks the source character by character
and flushes a token when its buffer reaches the span length,
for example `src/rt/tokenize.rs:127-131`:

```rust
if current_token_buffer.len() == current_token_buffer.capacity() {

    let context = TokenContext{start_byte_offset: current_span.0, start_lineno, start_colno, start_char_index, end_byte_offset: current_span.2};
    let token = Token{lexeme: current_token_buffer, tok_type: current_span.1.clone(), context: Some(context)};
    source_tokens.push(token);
```

A span one byte short flushes the comment without its `/`,
the `/` starts the next token's buffer,
and every later lexeme is shifted by one byte;
in the harness runs the document's final byte was never flushed and was lost.
When a multibyte character pushes a buffer past its span length,
the `String` reallocates instead of reaching the flush condition,
and in both multibyte cases in "Verification" the loop ended at `unreachable!("Unexpected end of document")`
(`src/rt/tokenize.rs:155`).

### Why the existing tests pass

The round-trip parser's comment tests only check that parsing succeeds,
for example `src/rt/parser.rs:872-880`:

```rust
    #[test]
    fn test_block_comment_preceding_top_level_value() {
        let sample = r#"/*
    Some non-comment top-level value is needed;
    we use null below.
*/
null"#;
        let _res = from_str(sample).unwrap();
    }
```

`test_rt` in `src/rt/tokenize.rs:204-217` asserts a byte-identical round trip,
but its sample has only `//` comments.

### Readings this investigation corrected

- The research listed the `j08` edit output and the corpus round-trip failure as two findings.
  They are one parse-time span bug:
  the unedited `j08` source already loses the `/`
  (harness line `MODEL CHANGED` for the `j08` source in "Verification").
- "No parser accepts" the output was measured for two parsers only:
  `json-five` itself and `module-jsonc-edit`.

## Verification

Versions under test:

- `json-five` 0.3.1 from crates.io,
  checksum `865f2d01a4549c1fd8c60640c03ae5249eb374cd8cde8b905628d4b1af95c87c`,
  published 2026-01-08,
  and upstream `main` at `650be6a39503371b2ea5bbc69d635f79ba977241` (the "Prototype fix" base).
- `rustc 1.97.1 (8bab26f4f 2026-07-14)` in `docker.io/library/rust:1.97-bookworm`
  (`sha256:389c1ae98c20fbcadca68a685482749267cec3c90893ae4671c5a37cc894c416`).
- Run on 2026-09-17 with the command in "Harness command".

### Harness

```toml
# json-five-block-comment-repro/Cargo.toml
[package]
name = "json_five_block_comment_repro"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
json-five = "=0.3.1"
serde = "1"
```

```rust
// json-five-block-comment-repro/src/main.rs
use json_five::rt::parser::{JSONValue, from_str};
use json_five::tokenize::TokType;
use json_five::{source_to_tokens, tokens_to_source};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Round-trips `src` through the model and reports byte identity and whether the output reparses.
fn model(src: &str) {
    let result = catch_unwind(AssertUnwindSafe(|| from_str(src).map(|text| text.to_string())));
    match result {
        Err(_) => println!("MODEL PANIC      {src:?}"),
        Ok(Err(e)) => println!("MODEL PARSE-ERR  {src:?} -> {}", e.message),
        Ok(Ok(out)) if out == src => println!("MODEL OK         {src:?}"),
        Ok(Ok(out)) => match from_str(&out) {
            Ok(_) => println!("MODEL CHANGED    {src:?} -> {out:?} (reparses)"),
            Err(e) => println!("MODEL CHANGED    {src:?} -> {out:?} (reparse: {} at index {})", e.message, e.index),
        },
    }
}

/// Round-trips `src` through the owned-token API and prints each comment lexeme.
fn tokens(src: &str) {
    match catch_unwind(AssertUnwindSafe(|| source_to_tokens(src))) {
        Err(_) => println!("TOKENS PANIC     {src:?}"),
        Ok(Err(e)) => println!("TOKENS ERR       {src:?} -> {}", e.message),
        Ok(Ok(toks)) => {
            let same = tokens_to_source(&toks) == src;
            let comments: Vec<(&TokType, &str)> = toks
                .iter()
                .filter(|t| t.tok_type == TokType::BlockComment || t.tok_type == TokType::LineComment)
                .map(|t| (&t.tok_type, t.lexeme.as_str()))
                .collect();
            println!("TOKENS same={same:<5} {src:?} comments={comments:?}");
            if !same {
                println!("       tokens_to_source -> {:?}", tokens_to_source(&toks));
            }
        }
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| eprintln!("panic: {info}")));
    let cases = [
        "{\"a\": 1}",
        "{ \"a\": 1, }",
        "{ \"a\": 1 } // trailing line comment",
        "// leading line comment\n{ \"a\": 1 }",
        "{\n  \"a\": 1, // inline\n  \"b\": 2\n}",
        "{\n  \"a\": 1, // crlf\r\n  \"b\": 2\r\n}",
        "{ /* block */ \"a\": 1 }",
        "/* top */ {\"a\": 1}",
        "{\"a\": 1} /* tail */",
        "[1, /* x */ 2]",
        "/** doc **/ {}",
        "{\n  \"compilerOptions\": {\n    /* strictness */\n    \"strict\": false, // todo\n  },\n}\n",
        "{\"a\": 1} // caf\u{e9}",
        "{\"a\": 1 // x\u{2028}}",
    ];
    for src in cases {
        model(src);
    }
    for src in cases {
        tokens(src);
    }
    // Non-round-trip entry points discard comment spans; check whether they are affected.
    for src in cases {
        let serde = catch_unwind(AssertUnwindSafe(|| json_five::from_str::<serde::de::IgnoredAny>(src).is_ok()));
        let plain = catch_unwind(AssertUnwindSafe(|| json_five::model_from_str(src).map(|t| t.to_string())));
        println!("PLAIN serde_ok={:?} model={:?} {src:?}", serde, plain.map(|r| r.map_err(|e| e.message)));
    }

    // j08: set compilerOptions.strict to true, then serialize.
    let src = "{\n  \"compilerOptions\": {\n    /* strictness */\n    \"strict\": false, // todo\n  },\n}\n";
    let mut text = from_str(src).expect("parse");
    if let JSONValue::JSONObject { key_value_pairs, .. } = &mut text.value {
        if let JSONValue::JSONObject { key_value_pairs: inner, .. } = &mut key_value_pairs[0].value {
            inner[0].value = JSONValue::Bool(true);
        }
    }
    let out = text.to_string();
    println!("EDIT output {out:?}");
    match from_str(&out) {
        Ok(_) => println!("EDIT reparse ok"),
        Err(e) => println!("EDIT reparse err: {e:?}"),
    }
}
```

### Harness command

Third-party code was built and run only in a container with no credentials,
no repository mount,
and a 2 GiB memory cap:

```sh
podman run --rm --memory=2g --cpus=2 \
  --volume "${HARNESS}:/work:Z" --workdir /work \
  docker.io/library/rust:1.97-bookworm cargo run --quiet --jobs 2
```

### Patterns that round-trip cleanly

Every entry printed `MODEL OK` and `TOKENS same=true`:

- `{"a": 1}` and `{ "a": 1, }` (no comments)
- `{ "a": 1 } // trailing line comment` (ASCII line comment ending at end of input)
- `// leading line comment\n{ "a": 1 }`
- `{\n  "a": 1, // inline\n  "b": 2\n}`
- `{\n  "a": 1, // crlf\r\n  "b": 2\r\n}`
  (the `LineComment` lexeme is `// crlf\r`;
  the `\n` becomes whitespace,
  and the round trip is still byte-identical)

`json_five::from_str::<serde::de::IgnoredAny>` returned `Ok` and `model_from_str` parsed every case in the harness,
including the failing ones,
because neither keeps comments
(`model_from_str` printed `{"compilerOptions": {"strict": false}}` for the `j08` source).

### Patterns that lose the closing slash

Model output,
each followed by the reparse error `Unexpected end of input while processing block comment`:

```text
MODEL CHANGED    "{ /* block */ \"a\": 1 }" -> "{ /* block * \"a\": 1 }" (reparse: ... at index 2)
MODEL CHANGED    "/* top */ {\"a\": 1}" -> "/* top * {\"a\": 1}" (reparse: ... at index 0)
MODEL CHANGED    "{\"a\": 1} /* tail */" -> "{\"a\": 1} /* tail *" (reparse: ... at index 9)
MODEL CHANGED    "[1, /* x */ 2]" -> "[1, /* x * 2]" (reparse: ... at index 4)
MODEL CHANGED    "/** doc **/ {}" -> "/** doc ** {}" (reparse: ... at index 0)
```

The `j08` source changed the same way without an edit
(`/* strictness */` became `/* strictness *`,
reparse error at index 29),
and the edit path printed:

```text
EDIT output "{\n  \"compilerOptions\": {\n    /* strictness *\n    \"strict\": true, // todo\n  },\n}\n"
EDIT reparse err: ParsingError { index: 29, message: "Unexpected end of input while processing block comment", ... }
```

Owned-token API output for the same inputs:

```text
TOKENS same=false "{ /* block */ \"a\": 1 }" comments=[(BlockComment, "/* block *")]
       tokens_to_source -> "{ /* block */ \"a\": 1 "
TOKENS same=false "{\"a\": 1} /* tail */" comments=[(BlockComment, "/* tail *")]
       tokens_to_source -> "{\"a\": 1} /* tail *"
TOKENS same=false "/** doc **/ {}" comments=[(BlockComment, "/** doc **")]
       tokens_to_source -> "/** doc **/ {"
```

For the `j08` source the token list also shows the shift:
`comments=[(BlockComment, "/* strictness *"), (LineComment, " // todo")]`.

### Patterns that panic

```text
MODEL PANIC      "{\"a\": 1} // café"
MODEL PANIC      "{\"a\": 1 // x\u{2028}}"
TOKENS PANIC     "{\"a\": 1} // café"
TOKENS PANIC     "{\"a\": 1 // x\u{2028}}"
panic: panicked at .../json-five-0.3.1/src/rt/parser.rs:371:21:
end byte index 16 is not a char boundary; it is inside 'é' (bytes 15..17 of string)
panic: panicked at .../json-five-0.3.1/src/rt/parser.rs:371:21:
end byte index 13 is not a char boundary; it is inside '\u{2028}' (bytes 12..15 of string)
panic: panicked at .../json-five-0.3.1/src/rt/tokenize.rs:155:5:
internal error: entered unreachable code: Unexpected end of document
```

### Independent token-consumer control, 2026-09-24

A separate scratch Rust consumer at `~/temp/agent/json-five-token-probe/`
 imported published `json-five = "=0.3.1"` and ran `mise run test` inside
 `podman run --memory=2g --cpus=2 --rm` with only that fixture mounted.
 Its initially optimistic check of the string following a block comment **failed**.
 The printed token stream contained a block-comment lexeme `"/*value\ncomment*"`
 and the following `DoubleQuotedString` lexeme `"/\"\\uD800"`.
 The comment span's reported end omitted the `/`,
 and the owned-token constructor then treated that slash as the next string's first byte.
 The subsequent number token began with `:` rather than its own first digit.
 This is the same off-by-one `process_comment` incident traced in the Root cause section,
 not a separate string-decoding defect.

The final bounded consumer suite passed two explicit controls:
 without a block comment,
 `source_to_tokens` retained the raw `"\\uD800"` escape and `1.000e+99` spelling and round-tripped byte-identically;
 with an inserted multi-line block comment,
 it asserted the truncated comment lexeme and non-identical round trip.
 A CR-ending line comment was tokenized without absorbing the following array,
 and JSON5-only single quotes were still admitted lexically.
 Passing these negative and positive controls does **not** make unpatched 0.3.1 an editor foundation:
 every supported multi-line block comment still corrupts the public round-trip token stream.

## Verified workarounds

### Fail closed at the consumer boundary

Treat any round-trip mismatch as an error before writing a file:
parse under `catch_unwind`,
serialize before editing,
and refuse the file when the serialization is not byte-identical to the input.
The harness's `model` function is the same check,
and it flagged every failing pattern in "Verification" while passing every clean one.

```rust
// consumer-side guard around json_five::rt
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Parses `src` for editing only when the round-trip model reproduces it exactly.
fn parse_for_edit(src: &str) -> Result<json_five::rt::parser::JSONText, String> {
    let parsed = catch_unwind(AssertUnwindSafe(|| json_five::rt::parser::from_str(src)))
        .map_err(|_| "json-five panicked while parsing".to_owned())?
        .map_err(|e| e.to_string())?;
    if parsed.to_string() != src {
        return Err("json-five 0.3.1 cannot round-trip this file (block comment or multibyte comment end)".to_owned());
    }
    Ok(parsed)
}
```

This function ran as a second binary of the harness crate against 0.3.1
(same `podman run` bounds,
`cargo run --bin guard`,
with a `main` that prints `parse_for_edit(src).map(|_| "editable")` per input):

```text
"{ \"a\": 1 } // trailing line comment" -> Ok("editable")
"{\n  \"a\": 1, // crlf\r\n  \"b\": 2\r\n}" -> Ok("editable")
"{ /* block */ \"a\": 1 }" -> Err("json-five 0.3.1 cannot round-trip this file (block comment or multibyte comment end)")
"{\"a\": 1} // café" -> Err("json-five panicked while parsing")
"{ 'a': 1 " -> Err("ParsingError: Expecting '}' at end of object: line 1 column 10 (char 9)")
```

Tradeoffs:

- It detects the defect and never repairs it:
  every file with a block comment,
  and every file whose `//` comment ends in a multibyte character,
  becomes uneditable.
- `catch_unwind` does not catch aborts;
  a binary built with `panic = "abort"` still terminates.
- The check doubles serialization work per file.

### Carry the prototype fix through `[patch.crates-io]`

Point `json-five` at a fork under our own account with
[`json-five-unterminated-block-comment.patch`](json-five-unterminated-block-comment.patch) applied:

```toml
# consumer Cargo.toml
[patch.crates-io]
json-five = { git = "https://github.com/<our-account>/json-five-rs", rev = "<patched commit>" }
```

The harness run against the patched source in "Prototype fix" is this mechanism
(`json-five = { path = "/proto" }` under `[patch.crates-io]`),
and every case round-tripped.
No fork was created in this session.

Tradeoffs:

- A git dependency that must be rebased on each upstream release until upstream ships the fix.
- The patch also changes `LineComment` span ends,
  so any consumer that read the old spans directly sees different offsets.
- Only `src/tokenize.rs` changes;
  other round-trip gaps in `json_five::rt` (for example inserting members needs hand-built whitespace contexts,
  per `rust-structured-edits.md` "J5") remain.

### Use `jsonc-parser` for JSONC edits

The research's J1 route (`rust-structured-edits.md`,
"JSONC options")
edits the same `j08` source correctly with `jsonc-parser` 0.33.2 and round-trips `{ /* block */ "a": 1 }`.

Tradeoffs:

- JSONC only,
  not JSON5;
  unquoted keys and single-quoted strings must be rejected or handled elsewhere.
- Its default parse options are loose;
  see [`jsonc-parser-json5-defaults.md`](jsonc-parser-json5-defaults.md).

## What does not work

- **Editing only values and leaving contexts alone.**
  The `/` is lost at parse time,
  so an untouched context string is already short (`MODEL CHANGED` without any edit).
- **Switching to the token API (`source_to_tokens` and `tokens_to_source`).**
  The same span feeds its buffer sizes:
  lexemes shift by one byte,
  the document's last byte is dropped,
  and multibyte comment ends hit `unreachable!`.
- **Switching to `model_from_str` or serde `from_str`.**
  Both parse,
  but neither keeps comments or whitespace,
  so writing a file back loses all of them.

## Upstream filing decision

Decision:
 all six constraints hold,
 so the draft in "Draft issue" is fileable once a human completes "Before filing".
Nothing was filed or commented upstream from this session.

`.out-of-scope/` was checked:
 it has no entry for `json-five`,
 JSON5,
 or JSONC tooling.

### Duplicate search

Searched 2026-09-17 with `gh search issues --repo spyoungtech/json-five-rs --include-prs` and `gh search prs`,
 open and closed,
 for an empty query (all 26 issues and pull requests listed and read by title),
 `comment`,
 `round trip`,
 `unterminated`,
 `slash`,
 and `tokenize`.
No issue or pull request reports block comments losing their closing slash or multibyte comment panics.
Closest threads,
 read in full:

- [spyoungtech/json-five-rs#24][json-five-24] (closed):
   the round-trip parser wrote double-quoted strings without their quotes.
  Different token,
   same class (round-trip output not reparseable);
   fixed by [#25][json-five-25] and released in 0.3.1 the same day.
- [spyoungtech/json-five-rs#7][json-five-7] (open):
   a feature request for methods that edit whitespace and comments;
   it notes contexts are plain concatenated strings but reports no corruption.
- #9 and #15 (merged):
   older multibyte fixes for the EOF token and for characters after a value;
   neither touches `process_comment`.

### Six constraints

1.  **Upstream's fault:**
    yes.
    This is behavior,
     not wording:
     the tokenizer violates its own end-exclusive span contract (`src/tokenize.rs:36`),
     and the README promises comment preservation.
    Nothing in the architecture requires it;
     the line-comment branch already adds one byte.
2.  **Upstream can fix it:**
    yes;
    "Prototype fix" changes the three return expressions in `process_comment` and tracks the last consumed character.
3.  **Supported use case:**
    yes.
    `README.md:10` and `:12` list round-trip preservation of comments and "token-based round-tripping",
     `README.md:104-107` points at round-trip examples,
     and `src/rt/parser.rs` has block-comment tests for the round-trip parser.
4.  **Contribution welcome:**
    yes.
    `README.md:159-160` says
     "Questions,
     discussions,
     and contributions are welcome"
     and suggests opening an issue.
    The repository has no `CONTRIBUTING.md`,
     no issue or pull request templates,
     and no AI-assistance policy (`.github/` holds only `workflows/`);
     no ban was found.
    An outside contributor's comparable report (#24) was fixed and released.
5.  **Likely to fix:**
    yes.
    #24 was closed by #25 and released in 0.3.1 on the day of the fix (2026-01-08),
     and #26 merged on 2026-05-13.
    No won't-fix or non-goal was found.
6.  **Prototype:**
    yes;
    see "Prototype fix".

### Prototype fix

Patch:
 [`json-five-unterminated-block-comment.patch`](json-five-unterminated-block-comment.patch),
 against `spyoungtech/json-five-rs` `main` at `650be6a39503371b2ea5bbc69d635f79ba977241`
 (`git apply --check` passes on a clean clone of that commit).
It:

- makes `process_comment` return `last_idx + 1` for block comments
   (the closing `/` is one byte);
- tracks the last consumed character in the line-comment branch and returns `last_idx + last_char.len_utf8()`
   at end of input and after a line terminator;
- adds `tokenize::test::test_comment_spans_are_noninclusive`,
   `rt::parser::tests::test_comments_round_trip`,
   and `rt::tokenize::tests::test_rt_comments`.

The tokenizer change:

```diff
--- a/src/tokenize.rs
+++ b/src/tokenize.rs
@@ -488,19 +488,21 @@ impl <'input> Tokenizer<'input> {
         match star_or_slash {
             '/' => {
                 // line comment
+                let mut last_char = star_or_slash;
                 loop {
                     match self.chars.peek() {
                         None => {
-                            return Ok((start_idx, TokType::LineComment, last_idx+1))
+                            return Ok((start_idx, TokType::LineComment, last_idx + last_char.len_utf8()))
                         },
                         Some((peeked_idx, peeked_char)) => {
                             match peeked_char {
                                 '\n' | '\r' | '\u{2028}' | '\u{2029}' => {
-                                    (last_idx, _) = self.advance().unwrap();
-                                    return Ok((start_idx, TokType::LineComment, last_idx+1))
+                                    (last_idx, last_char) = self.advance().unwrap();
+                                    return Ok((start_idx, TokType::LineComment, last_idx + last_char.len_utf8()))
                                 }
                                 _ => {
                                     last_idx = *peeked_idx;
+                                    last_char = *peeked_char;
                                     self.advance();
                                 }
                             }
@@ -528,7 +530,7 @@ impl <'input> Tokenizer<'input> {
                                             match next_peeked_char {
                                                 '/' => {
                                                     (last_idx, _) = self.advance().unwrap();
-                                                    return Ok((start_idx, TokType::BlockComment, last_idx))
+                                                    return Ok((start_idx, TokType::BlockComment, last_idx + 1))
                                                 }
                                                 _ => {
                                                     continue
```

Build bounds:
 a fresh `mktemp --directory "${HOME}/temp/agent/upstream-prototype.XXXXXXXX"` clone
 (origin `https://github.com/spyoungtech/json-five-rs.git`,
 push URL disabled,
 HEAD verified as `650be6a`),
 built and tested only in `podman run --rm --memory=2g --cpus=2` with `docker.io/library/rust:1.97-bookworm`,
 only the prototype directory mounted,
 and no credentials in the environment.

Test command,
 run once with only the three tests added and again with the fix applied:

```sh
podman run --rm --memory=2g --cpus=2 \
  --volume "${PROTOTYPE}/json-five-rs:/work:Z" --workdir /work \
  docker.io/library/rust:1.97-bookworm cargo test --jobs 2
```

Tests only (unpatched tokenizer):

```text
test rt::parser::tests::test_comments_round_trip ... FAILED
test rt::tokenize::tests::test_rt_comments ... FAILED
test tokenize::test::test_comment_spans_are_noninclusive ... FAILED
  left: "{ /* block * \"a\": 1 }"
 right: "{ /* block */ \"a\": 1 }"
  left: Tokens { tok_spans: [(0, BlockComment, 6), (7, Whitespace, 8), (8, LineComment, 13), (14, EOF, 14)], ... }
 right: Tokens { tok_spans: [(0, BlockComment, 7), (7, Whitespace, 8), (8, LineComment, 14), (14, EOF, 14)], ... }
test result: FAILED. 372 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Patched:

```text
test result: ok. 375 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests examples/json5-doublequote-fixer/src/main.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests examples/json5-trailing-comma-formatter/src/main.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests json_five
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

That is upstream's full `cargo test`,
 the command its `.github/workflows/build.yaml` runs.
The compiler warnings (`with_max_depth` never used,
 unused `err` and `context` variables) are the same set before and after the patch.

User-boundary check:
 the "Harness" crate with `[patch.crates-io] json-five = { path = "/proto" }`,
 the patched clone mounted read-only,
 printed `MODEL OK` and `TOKENS same=true` for all 14 cases,
 `comments=[(BlockComment, "/* strictness */"), (LineComment, "// todo\n")]` for the `j08` source,
 and for the edit:

```text
EDIT output "{\n  \"compilerOptions\": {\n    /* strictness */\n    \"strict\": true, // todo\n  },\n}\n"
EDIT reparse ok
```

The reproduction in "Draft issue" also ran as written against unpatched 0.3.1
 (as a third binary of the "Harness" crate):

```text
{ /* block * "a": 1 }
Err(ParsingError { index: 2, message: "Unexpected end of input while processing block comment", lineno: 1, colno: 3, char_index: 2 })
thread 'main' (1) panicked at .../json-five-0.3.1/src/rt/parser.rs:371:21:
end byte index 16 is not a char boundary; it is inside 'é' (bytes 15..17 of string)
```

### Before filing

A human must personally:

- rerun the reproduction in "Draft issue" against the latest `json-five` release;
- open `src/tokenize.rs` on current `main` and confirm `process_comment` still returns `last_idx` for block comments;
- apply the patch and run `cargo test`;
- remove any clause of the draft's footer describing a check they did not do.

### Draft issue

~~~md
Title: Round-trip tokenizer ends comment spans early: block comments lose `/`, multibyte line-comment ends panic
Labels: bug

`json_five::rt::parser::from_str(src)?.to_string()` drops the closing `/` of every block comment, even with no edits,
so the output no longer parses. The same span bug makes the round-trip parser panic on a `//` comment whose last
character is multibyte.

## Reproduction (json-five 0.3.1, also `main` at 650be6a)

```rust
use json_five::rt::parser::from_str;

fn main() {
    let src = "{ /* block */ \"a\": 1 }";
    let out = from_str(src).unwrap().to_string();
    println!("{out}"); // { /* block * "a": 1 }
    println!("{:?}", from_str(&out).map(|_| ())); // Err(... "Unexpected end of input
while processing block comment" ...)

    // panics: end byte index 16 is not a char boundary; it is inside 'é' (bytes 15..17 of string)
    let _ = from_str("{\"a\": 1} // café");
}
```

`source_to_tokens` has the same problem: the `BlockComment` lexeme is `/* block *`, later lexemes shift by one byte,
`tokens_to_source` drops the document's last byte, and `"{\"a\": 1} // café"` hits
`unreachable!("Unexpected end of document")` in `src/rt/tokenize.rs`.

## Cause

`TokenSpan` ends are noninclusive (`src/tokenize.rs:36`), but `process_comment` returns the index of the closing `/`
for block comments (`src/tokenize.rs:531`: `return Ok((start_idx, TokType::BlockComment, last_idx))`).
The line-comment branch returns `last_idx+1` (`:494`, `:500`), where `last_idx` is the start of the last character,
which is inside that character when it is multibyte (`é`, or a U+2028/U+2029 terminator).
`rt::parser` slices comment text by these spans (`get_tok_source`, `src/rt/parser.rs:371`), and
`source_to_tokens` sizes its buffers from them. `from_str` and `model_from_str` are unaffected because they discard
comment spans (`src/tokenize.rs:596-601`).

## Suggested fix

```diff
--- a/src/tokenize.rs
+++ b/src/tokenize.rs
@@ -488,19 +488,21 @@ impl <'input> Tokenizer<'input> {
         match star_or_slash {
             '/' => {
                 // line comment
+                let mut last_char = star_or_slash;
                 loop {
                     match self.chars.peek() {
                         None => {
-                            return Ok((start_idx, TokType::LineComment, last_idx+1))
+                            return Ok((start_idx, TokType::LineComment, last_idx + last_char.len_utf8()))
                         },
                         Some((peeked_idx, peeked_char)) => {
                             match peeked_char {
                                 '\n' | '\r' | '\u{2028}' | '\u{2029}' => {
-                                    (last_idx, _) = self.advance().unwrap();
-                                    return Ok((start_idx, TokType::LineComment, last_idx+1))
+                                    (last_idx, last_char) = self.advance().unwrap();
+                                    return Ok((start_idx, TokType::LineComment, last_idx + last_char.len_utf8()))
                                 }
                                 _ => {
                                     last_idx = *peeked_idx;
+                                    last_char = *peeked_char;
                                     self.advance();
                                 }
                             }
@@ -528,7 +530,7 @@ impl <'input> Tokenizer<'input> {
                                             match next_peeked_char {
                                                 '/' => {
                                                     (last_idx, _) = self.advance().unwrap();
-                                                    return Ok((start_idx, TokType::BlockComment, last_idx))
+                                                    return Ok((start_idx, TokType::BlockComment, last_idx + 1))
                                                 }
                                                 _ => {
                                                     continue
```

With a span test in `src/tokenize.rs` and byte-identical round-trip tests in `src/rt/parser.rs` and
`src/rt/tokenize.rs` (block comments in several positions, `é` at end of input, U+2028 and U+2029 terminators),
the three new tests fail before this change and pass after it, and the existing `cargo test` suite still passes
(375 unit tests, both example tests, 9 doctests). I can open this as a pull request.

---

Written by an agent (Claude Code, claude-opus-5).
A human reran the reproduction, checked the cited source lines, and ran the patched test suite before filing.
~~~

[json-five-rs]: https://github.com/spyoungtech/json-five-rs
[json-five-7]: https://github.com/spyoungtech/json-five-rs/issues/7
[json-five-24]: https://github.com/spyoungtech/json-five-rs/issues/24
[json-five-25]: https://github.com/spyoungtech/json-five-rs/pull/25
