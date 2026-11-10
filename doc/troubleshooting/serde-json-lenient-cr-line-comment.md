# serde_json_lenient 0.2.4: a bare CR inside `//` swallows the next JSONC member

Status:
 **source diagnosis and bounded red/green prototype verified**.
 Published 0.2.4 still has the behavior;
 a local checked patch is not an upstream release.
 No upstream issue or patch was sent.

## Symptom

`serde_json_lenient::from_str::<Value>` accepts a line comment followed by LF or CRLF,
 but rejects the same object when its `//` comment ends in a bare carriage return:

```text
{"a":1,// c\r"b":2}
EOF while parsing an object at line 1 column 18
```

The `\r` in the displayed source denotes **one actual carriage-return byte**.
 The ordinary member `"b":2` is treated as part of the line comment,
 so the parser reaches EOF instead of reading the member or closing brace.
 This is separate from this repository's TypeScript CR-line-comment defect;
 the parser implementations have different source paths.

## Root cause

The audited upstream checkout `google/serde_json_lenient` at
 `111dd4522f5989efe43449e715a96e1aee533894` declares 0.2.4 in `Cargo.toml:3`.
 Its `src/de.rs` SHA-256 matches the published 0.2.4 archive:
 `cef85a62110299dee3f217a0e84ec1263f3bcb105f1d87b374780dfba62199ef`.
 The disposable consumer's Cargo.lock pins that release with checksum
 `0e033097bf0d2b59a62b42c18ebbb797503839b26afdda2c4e1415cb6c813540`.

1.  `src/de.rs:2824-2829` routes public `from_str` through `from_trait`,
    which constructs `Deserializer::new` (`:2612-2622`).
    The constructor enables comments by default (`:53-68`):

    ```rust
    // src/de.rs:2824-2829
    pub fn from_str<'a, T>(s: &'a str) -> Result<T>
    where
        T: de::Deserialize<'a>,
    {
        from_trait(read::StrRead::new(s))
    }
    ```

    ```rust
    // src/de.rs:60-67
    remaining_depth: 128,
    ignore_trailing_commas: true,
    allow_comments: true,
    ```

2.  The general whitespace branch recognizes CR and LF,
    but `//` handling ends only on LF (`src/de.rs:302-333`):

    ```rust
    // src/de.rs:303-326
    // Consume comments as if they were whitespace.
    loop {
        match tri!(self.peek()) {
            Some(b' ' | b'\n' | b'\t' | b'\r') => {
                self.eat_char();
            }
            Some(b'/') if self.allow_comments => {
                self.eat_char();
                match tri!(self.peek()) {
                    Some(b'/') => {
                        // TODO: Read until newline.
                        loop {
                            match tri!(self.peek()) {
                                Some(b'\n') => {
                                    self.eat_char();
                                    break;
                                }
                                Some(_) => {
                                    self.eat_char();
                                }
    ```

    A CR-only separator enters `Some(_)` and the scanner consumes the rest of the source as comment text.
    A CRLF separator eventually reaches LF and terminates,
    which explains the positive control.

3.  When `parse_whitespace` returns `None` at EOF,
    object-member access reports `ErrorCode::EofWhileParsingObject`
    (`src/de.rs:2100-2108`):

    ```rust
    // src/de.rs:2100-2108
    let peek = match tri!(map.de.parse_whitespace()) {
        Some(b) => b,
        None => {
            return Err(map.de.peek_error(ErrorCode::EofWhileParsingObject));
        }
    };
    ```

The README describes human-authored JSON with both `/*` and `//` comments
 (`README.md:9-17`),
 but does not state a special LF-only restriction.
 The fix needs no number,
 string or comment-ownership redesign for this parse-only behavior.

## Verification

- `~/temp/agent/serde-json-lenient-cr-probe/` is an independent Rust consumer of
   `serde_json_lenient = "=0.2.4"`.
   Its `mise run test` ran inside `podman run --memory=2g --cpus=2 --rm`
   using the locally inspected Rust 1.97.1 image and only that disposable consumer mounted.
   It printed the exact EOF error quoted in the Symptom section.
- A fresh private prototype clone at
   `~/temp/agent/upstream-prototype.7k2y07VI/serde_json_lenient/`
   was checked against upstream commit `111dd4522f5989efe43449e715a96e1aee533894`
   and the published source hash **before editing**.
   The added `tests/cr_comment.rs` initially failed both bare-CR controls under
   `mise run test:serde-json-lenient:control`:
   **zero passed,
   two failed** (exit 101).
   The LF and CRLF inputs in the first test reached the later failing CR input,
   so the red test included accepted controls.
- After applying the linked [patch](serde-json-lenient-cr-line-comment.patch),
   the same bounded focused command passed both new tests (exit 0).
   `mise run test:serde-json-lenient:upstream` then ran `cargo test --all-targets`
   under the same container bounds;
   every test group passed,
   with one existing ignored test.
   The patch contains the source change and integration harness;
   in this session the bounded commands were:

   ```bash
   # doc/troubleshooting/serde-json-lenient-cr-line-comment.md
   cd -- "$HOME/temp/agent"
   mise run test:serde-json-lenient:control
   mise run test:serde-json-lenient:upstream
   ```

- Accepted controls after the patch:
   `// c\n`,
   `// c\r\n`,
   `// c\r`,
   and consecutive CR-separated `//` comments all leave member `"b":2` readable.
- Failure before the patch:
   CR-only and consecutive CR-separated line comments returned the
   object-EOF error instead of a `Value` containing `b`.
   LF and CRLF did not produce that failure in the consumer.

The published crate was executed only on these bounded inputs inside the disposable container;
 no user configuration or repository state was used as a test fixture.

## Verified workarounds

- Apply the [checked source patch](serde-json-lenient-cr-line-comment.patch)
   in a disposable fork or future published version before using CR-only JSONC comments.
   It treats either `\r` or `\n` as a line-comment terminator;
   an LF following CR is consumed by the normal whitespace loop.
   The red/green controls and upstream suite passed.
   Tradeoff:
   using an unpublished fork adds source and release maintenance,
   and this patch still does **not** add separately queryable comments,
   raw mathematical number equality or a proven 512-depth lifecycle for this port.
- Use the repository-owned iterative scratch parser for this domain during foundation evaluation.
   Its bounded CR,
   CRLF,
   emission/reparse and external-consumer controls passed in the parser vet.
   Tradeoff:
   that prototype remains unpublished and must still complete the overall foundation gates.

## What does not work

- Switching only from `from_str` to an ordinary `serde_json_lenient::Value` projection:
   the actual consumer already used that entry and failed on CR-only input.
- Treating a CRLF passing case as evidence that CR-only is supported:
   the LF in CRLF stops the source loop,
   while a bare CR takes its `Some(_)` branch.
- Calling a comment discard operation after parsing:
   the parser has already consumed the following member before any value is available.

## Upstream filing decision

No entry under this repository's `.out-of-scope/` list exempts this crate or comment-boundary issue.
 Searches of open and closed upstream issues and PRs for carriage-return comments and CRLF
 found no duplicate.
 The upstream issue list includes an unrelated trailing-comma-in-ignored-fields report
 ([issue 34][issue-34]),
 not this line-comment case.
 No external filing was sent.

1.  Upstream fault:
    yes.
    `src/de.rs:303-326` consumes bare CR inside a supported `//` comment,
    although the same parser treats CR as JSON whitespace.
2.  Fixability:
    yes.
    The minimal scanner branch change and tests passed under the existing architecture.
3.  Supported use case:
    yes for human-authored JSON with `//` comments (`README.md:9-17`).
    The README does not advertise a special LF-only grammar.
4.  Contribution welcome:
    `README.md:44-50` gives contribution licensing terms.
    The inspected `.github/` has only a CI workflow,
    with no contribution prohibition or AI-assistance ban found.
5.  Maintainer disposition:
    no searched issue,
    PR or repository policy declines this correction.
    Silence is not evidence of a response timetable.
6.  Prototype:
    yes.
    [The patch](serde-json-lenient-cr-line-comment.patch) contains the small source change and
    consumer-boundary tests;
    they failed prepatch and passed with the source change,
    and the patched upstream suite passed.

These technical filing conditions hold,
 but sending an external issue or patch requires separate authorization.
 The following draft is local only:

~~~md
Title: Bare CR does not terminate `//` comments in serde_json_lenient 0.2.4

`serde_json_lenient::from_str::<Value>("{\"a\":1,// c\r\"b\":2}")`
returns `EOF while parsing an object at line 1 column 18`,
while LF and CRLF variants return both members.
The README describes support for `//` comments in human-authored JSON.

At `src/de.rs:303-326`,
`parse_whitespace` recognizes CR as whitespace,
but the inner `//` loop breaks only on `Some(b'\n')`.
It consumes a bare CR and the following member through EOF.
`MapAccess` then reports `EofWhileParsingObject` (`src/de.rs:2100-2108`).

Changing the comment-loop arm to `Some(b'\n' | b'\r')` and leaving the
outer whitespace scan to consume an optional LF fixed the two new
carriage-return tests in a disposable clone of v0.2.4
(`111dd4522f5989efe43449e715a96e1aee533894`).
The tests failed before the change and passed after it;
`cargo test --all-targets` passed in a 2 GiB/2 CPU container.
A patch and integration fixture are available at
`doc/troubleshooting/serde-json-lenient-cr-line-comment.patch`
in the Monochromatic repository.
~~~

[issue-34]: https://github.com/google/serde_json_lenient/issues/34
