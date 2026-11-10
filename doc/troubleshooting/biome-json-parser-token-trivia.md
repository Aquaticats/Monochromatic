# Biome JSON parser 0.5.7: trailing JSONC comment is absent from token-only lookups

## Symptom

In a bounded consumer probe of `biome_json_parser` 0.5.7,
 parsing `{"s":"\uD800","n":1e0,} // c` with comments and trailing commas enabled returned no error.
 A search for a syntax token whose text was exactly `// c` nevertheless failed:

```text
# ~/temp/agent/jsonc-regex-audit/biome/lib.rs
comment token missing: ["{", "\"s\"", ":", "\"\\uD800\"", ",", "\"n\"", ":", "1e0", ",", "} // c", ""]
```

The comment was **not lost**.
 The closing-brace token's `text()` included its trailing trivia.
 This is an API-usage correction,
 not a demonstrated parser defect.

## Root cause

The published `biome_json_parser-0.5.7/src/token_source.rs:31-60` recognizes comments as trivia when allowed,
 instead of returning them as regular syntax tokens:

```rust
// biome_json_parser-0.5.7/src/token_source.rs:35-57
let trivia_kind = TriviaPieceKind::try_from(token.kind());
// ...
Ok(trivia_kind) => {
    if trivia_kind.is_newline() {
        trailing = false;
        self.preceding_line_break = true;
    }
    self.trivia.push(Trivia::new(trivia_kind, token.range(), trailing));
}
```

The published `biome_parser-0.5.7/src/tree_sink.rs:123-145` attaches the collected trivia to a syntax token:

```rust
// biome_parser-0.5.7/src/tree_sink.rs:129-145
self.eat_trivia(false);
let trailing_start = self.trivia_pieces.len();
self.text_pos = token_end;
self.eat_trivia(true);
// ...
self.inner.token_with_trivia(kind, text, leading, trailing);
```

The published `biome_rowan-0.5.7/src/syntax/token.rs:77-94,467-493` documents that `text()` includes trivia
 and exposes `leading_trivia()` and `trailing_trivia()` separately:

```rust
// biome_rowan-0.5.7/src/syntax/token.rs:77-94,467-493
/// Returns the text of the token,
/// including all trivia.
pub fn text(&self) -> &str {
    self.raw.text()
}
pub fn trailing_trivia(&self) -> SyntaxTrivia<L> {
    SyntaxTrivia::new(self.raw.trailing_trivia())
}
```

`biome_rowan-0.5.7/src/syntax/trivia.rs:231-269,603-609` supplies each piece's kind and text through
 `SyntaxTrivia::pieces()`.
 The same token/trivia accessors were checked in the upstream clone
 `~/temp/agent/biome-source-2026-09-24` at `2f629c5` under `crates/biome_rowan/src/syntax/`;
 the published archive is the authority for version 0.5.7 behavior.

## Verification

The published `biome_json_parser` archive SHA-256 is
 `9c6d23fb9b683e6356c094b4a0cb38f8aa0acee60ce9c3ef24628d21a204de4d`,
 matching its Cargo.lock checksum in the disposable probe.
 Its source and execution manifest are at `~/temp/agent/jsonc-regex-audit/biome/` and
 `~/temp/agent/biome-execution-manifest.md`.
 Run `mise run test:isolated` from that scratch package to repeat the probe in an offline,
 read-only 2 GiB/2 CPU container.
 The first bounded run failed the token-only assertion while its deep-array and JSON5-rejection controls passed.
 After changing only the comment lookup,
 bounded debug and optimized release runs passed the focused tests.

- Working pattern:
   `parsed.syntax().text().to_string() == source` preserved the complete input,
   including `// c`.
- Working pattern:
   `token.trailing_trivia().pieces()` yielded a piece with `kind().is_comment()` and `text() == "// c"`.
- Working pattern:
   `comment_trivia_positions` parsed `{/*key*/"a":/*value*/1, //inline\n"b":2}` and found comments on
   the trailing trivia of `{`,
   `:`,
   and `,` respectively.
   The raw trivia location is not the editor's key/value owner.
- Failing pattern:
   `descendants_tokens(...).any(|token| token.text() == "// c")` did not find an independent comment token.
- Failing pattern:
   searching the trimmed syntax-token text for `// c` is not a comment lookup;
   `text_trimmed()` excludes trivia (`biome_rowan-0.5.7/src/syntax/token.rs:126-149`).

The parse and syntax-text checks also passed for an escaped unpaired surrogate value and an unedited `1e0` token.
 Those checks do not establish that a Biome adapter implements the editor's attached-comment semantics.

## Verified workaround

Read comment pieces from the trivia on both sides of each syntax token.
 The disposable Rust probe uses:

```rust
// ~/temp/agent/jsonc-regex-audit/biome/lib.rs
let comment_found = parsed.syntax().descendants_tokens(Direction::Next).any(|token| {
    token.leading_trivia().pieces().chain(token.trailing_trivia().pieces())
        .any(|piece| piece.kind().is_comment() && piece.text() == "// c")
});
assert!(comment_found);
```

This reads all comment-bearing trivia without losing the raw source body.
 It does **not** determine whether a comment belongs to an object key,
 a value,
 a container,
 or a document.
 A consumer must implement and test that attachment rule separately;
 it must also inspect delimiters to distinguish line and block comments.

## What does not work

- Treating each `SyntaxToken::text()` as one grammar token treats `} // c` as one text string.
   It obscures the trivia boundary even though the comment survives in the syntax tree.
- Calling `text_trimmed()` and expecting a comment to become its own token cannot work;
   the method removes trivia rather than producing a separate comment node.
- Comparing the full syntax text with the input proves byte preservation,
   not queryable key/value ownership after canonical emission or edits.

## Upstream filing decision

`.out-of-scope/` has no Biome or JSON token-trivia exemption.
 `gh search issues 'trivia token comments' --repo biomejs/biome --include-prs --limit 20` returned no results.
 No upstream filing is proposed:

- Upstream fault:
   No.
   The published token API documents that `text()` includes trivia and exposes trivia accessors.
- Can upstream change it:
   Yes,
   but no correctness change is required for this observed behavior.
- Supported use case:
   Yes,
   lossless syntax and comment trivia are part of the parser's API.
- Contribution policy:
   Not evaluated for a proposed patch because there is no upstream fault.
- Likelihood of an upstream fix:
   Not applicable without a defect or requested enhancement.
- Minimal upstream prototype:
   Not applicable.
   The tested change is in the consumer's comment lookup,
   not in upstream source.

## Upstream filing artifact

Do not file as-is.
 The only possible request would be an optional documentation example,
 not a bug report:

~~~md
# Optional example for JSON syntax comment trivia

`SyntaxToken::text()` includes trailing trivia, so a caller searching for a separate `//` syntax token will not find one.
 A short example of `leading_trivia().pieces()` and `trailing_trivia().pieces()` in the JSON parser docs could help consumers
 locate comment pieces. The API already supports this; no code change is requested.
~~~
