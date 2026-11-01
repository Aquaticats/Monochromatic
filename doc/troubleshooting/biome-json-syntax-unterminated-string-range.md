# Biome JSON 0.5.7: an unterminated `"` makes the public `inner_string_text` helper panic

Status:
 **source-proven panic,
 reproduced through a consuming crate,
 and closed by a verified minimal patch prototype**.
 Upstream already fixed the cause in its lexer
 ([PR 2621](https://github.com/biomejs/biome/pull/2621)),
 but no published `biome_json_parser` release contains that fix.
 Nothing was filed or posted upstream.

## Symptom

A Rust consumer that parses the one-byte source `"` with published `biome_json_parser` 0.5.7
 and then calls the public `JsonStringValue::inner_string_text` helper aborts the current thread with:

```text
thread 'tests::unterminated_quote_does_not_panic' panicked at biome_text_size-0.5.8/src/range.rs:66:9:
assertion failed: start.raw <= end.raw
```

The parse itself succeeds and returns a tree;
 the panic happens later,
 inside the helper that strips a string literal's quotes.
 The same tree carries a `Missing closing quote` diagnostic,
 so callers that check diagnostics first never reach the helper.
 This is a Rust panic,
 **not** memory unsafety:
 the inverted range is rejected by an assertion rather than used to index memory.

## Root cause

All line numbers refer to the published crates.io archives used by the bounded probes:
 `biome_json_parser` 0.5.7 (archive SHA-256
 `9c6d23fb9b683e6356c094b4a0cb38f8aa0acee60ce9c3ef24628d21a204de4d`),
 `biome_json_syntax` 0.5.7 (lockfile checksum
 `f2645ca57f75680d3d390b2482c35db5850b1d849e1f96151a12f15f4abdb097`),
 and `biome_text_size` 0.5.8 (lockfile checksum
 `672627531edd258f1a9ecdd9bd5ff3ea6c36768622ce2cedc12dc03cb51605c9`).

1. The lexer treats an unterminated string as an ordinary string literal token and only records a diagnostic
   (`biome_json_parser-0.5.7/src/lexer/mod.rs:619-629`):

   ```rust
   // biome_json_parser-0.5.7/src/lexer/mod.rs:619-629
   LexStringState::InString => {
       let unterminated =
           ParseDiagnostic::new("Missing closing quote", start..self.text_position())
               .with_detail(
                   self.source.text_len()..self.source.text_len(),
                   "file ends here",
               );
       self.diagnostics.push(unterminated);

       JSON_STRING_LITERAL
   }
   ```

2. The parser bumps that token into a completed,
   well-typed string value node rather than a bogus one
   (`biome_json_parser-0.5.7/src/syntax.rs:55-59`):

   ```rust
   // biome_json_parser-0.5.7/src/syntax.rs:55-59
   JSON_STRING_LITERAL => {
       let m = p.start();
       p.bump(JSON_STRING_LITERAL);
       Present(m.complete(p, JSON_STRING_VALUE))
   }
   ```

3. The public helper assumes both delimiters exist and subtracts one byte from each end
   (`biome_json_syntax-0.5.7/src/string_ext.rs:4-8`
   and `biome_json_syntax-0.5.7/src/lib.rs:116-125`):

   ```rust
   // biome_json_syntax-0.5.7/src/lib.rs:116-125
   pub fn inner_string_text(token: &JsonSyntaxToken) -> TokenText {
       let mut text = token.token_text_trimmed();
       if token.kind() == JsonSyntaxKind::JSON_STRING_LITERAL {
           // remove string delimiters
           // SAFETY: string literal token have a delimiters at the start and the end of the string
           let range = TextRange::new(1.into(), text.len() - TextSize::from(1));
           text = text.slice(range);
       }
       text
   }
   ```

   For the source `"`,
   the trimmed token text is one byte long,
   so the range becomes start 1 and end 0.

4. `TextRange::new` asserts its ordering
   (`biome_text_size-0.5.8/src/range.rs:64-68`):

   ```rust
   // biome_text_size-0.5.8/src/range.rs:64-68
   pub const fn new(start: TextSize, end: TextSize) -> TextRange {
       assert!(start.raw <= end.raw);
       TextRange { start, end }
   }
   ```

The step-3 comment states the invariant that step 1 breaks:
 the same crate family produces a one-byte `JSON_STRING_LITERAL` whose helper then requires at least two bytes.

Upstream status,
 verified from the tracker and a read-only upstream clone:

- [Issue 2357][issue-2357] reports this exact panic,
   is labeled `S-Bug-confirmed`,
   and was closed on 2024-04-28 by [PR 2621][pr-2621].
- That PR changed the lexer,
   not the helper:
   an unterminated string now becomes `ERROR_TOKEN`.
   The upstream clone at `~/temp/agent/biome-source-2026-09-24` (`2f629c5`,
   workspace version 0.7.0) shows the fixed branch
   (`crates/biome_json_parser/src/lexer/mod.rs:670-679`).
- `cargo info biome_json_parser` still resolves **0.5.7** as the latest published version,
   which predates the fix,
   so a consumer pinning published crates keeps the panicking behavior.
- The newest published `biome_json_syntax` 0.7.0 retains the same unguarded helper
   (`biome_json_syntax-0.7.0/src/lib.rs:110-120`),
   and `biome_json_parser` 0.5.7 requires `biome_json_syntax` `0.5.7`
   (`biome_json_parser-0.5.7/Cargo.toml:44-51`),
   so the helper cannot be upgraded independently of the parser.

## Verification

- Library-boundary probe:
   `~/temp/agent/jsonc-regex-audit/biome/` gained
   `unterminated_string_boundary_and_raw_helper_panic`,
   which asserts the disposable editor adapter rejects `"` **and** that the raw upstream helper panics under
   `catch_unwind`.
   `mise run test:isolated` and `mise run test:isolated:release` each passed 18 tests
   (1 ignored test runs in its own bounded process).
   The caught panic named `biome_text_size-0.5.8/src/range.rs:66` and
   `assertion failed: start.raw <= end.raw`.
- Red/green consumer probes:
   two disposable crates share one test file and differ only by a
   `[patch.crates-io]` entry pointing at a copy of the published syntax crate.
   Both ran in `podman run --memory=2g --memory-swap=2g --cpus=2 --pids-limit=128`
   with `--network=none --read-only`,
   a 1 GiB `/target` tmpfs,
   the registry mounted read-only,
   and the pinned local Rust image
   `docker.io/library/rust@sha256:77fac8b98f9f46062bb680b6d25d5bcaabfc400143952ebc572e924bcbedc3fa`.

   ```bash
   # doc/troubleshooting/biome-json-syntax-unterminated-string-range.md
   cd -- "$HOME/temp/agent/biome-string-panic-unpatched" && mise run lock:update && mise run test
   cd -- "$HOME/temp/agent/biome-string-panic-patched" && mise run lock:update && mise run test
   ```

   Unpatched (published 0.5.7):
   `1 passed; 1 failed`,
   with `left: Err("inner_string_text panicked")` against `right: Ok("")`.
   Patched:
   `2 passed; 0 failed`.
- Both configurations keep the well-formed controls identical:
   `"bc"` yields `bc`,
   `""` yields an empty string,
   and `"\\u0041"` and `"\\\"q\\\""` keep their raw escaped spelling.
- The prototype patch is
  [biome-json-syntax-unterminated-string-range.patch](biome-json-syntax-unterminated-string-range.patch).
   `git apply --check` against a pristine copy of the published 0.5.7 `src/lib.rs`
   exited 0.

Patterns that panic in published 0.5.7:

- source `"` read through `JsonStringValue::inner_string_text`;
- the same source read through the free function `biome_json_syntax::inner_string_text`.

Patterns that do not panic:

- any terminated string literal,
   including the empty literal `""`;
- the same unterminated source when the caller checks `JsonParse::has_errors` first,
   which the disposable adapter does;
- an unterminated string in a **member-name** position reached through the adapter,
   because the adapter rejects the tree before projecting keys.

## Verified workarounds

- Reject any tree with diagnostics before projecting values.
   The disposable adapter's `has_errors` gate runs before the root,
   depth and value projection steps,
   and its bounded suites pass in both debug and optimized profiles.
   Tradeoff:
   the protection lives entirely in the consumer,
   so any other call site that ignores diagnostics is still exposed,
   and the gate makes error-recovery features unavailable on those inputs.
- Carry the minimal clamp through a patched copy of `biome_json_syntax` 0.5.7:

   ```rust
   // doc/troubleshooting/biome-json-syntax-unterminated-string-range.patch
   let end = text.len().checked_sub(TextSize::from(1)).unwrap_or_default();
   let start = TextSize::from(1).min(end);
   let range = TextRange::new(start, end);
   ```

   The green probe shows a one-byte literal then yields empty inner text while every terminated literal is unchanged.
   Tradeoffs:
   it forks a published crate whose parser companion is pinned at 0.5.7,
   it diverges from upstream's chosen lexer-side fix,
   and the prototype was verified as a patched registry copy rather than inside the upstream workspace,
   so a real pull request needs re-verification there.

## What does not work

- Relying on the helper's `SyntaxResult`:
   `value_token()?` succeeds for this tree,
   and the panic happens in the range arithmetic that follows.
- Assuming error trees expose bogus nodes:
   published 0.5.7 completes a normal `JSON_STRING_VALUE` for an unterminated literal,
   so typed narrowing does not filter it out.
- Upgrading `biome_json_syntax` alone to 0.7.0:
   the parser crate requires the 0.5.7 line,
   and 0.7.0's helper still subtracts one byte from each end unguarded.
- Waiting for the helper to be hardened upstream:
   the merged fix changed the lexer instead,
   and no published parser release contains it yet.

## Upstream filing decision

`.out-of-scope/` has no Biome or JSON-parser exemption.
 A duplicate search found [issue 2357][issue-2357],
 which reports this panic,
 is confirmed,
 and was closed by [PR 2621][pr-2621];
 searches for `inner_string_text`,
 `unterminated string panic json`,
 and `recursive drop` returned no other matching report.
 Because a matching thread exists and is already fixed in upstream source,
 **no new issue is drafted**.

1. **Upstream fault**:
   yes for published 0.5.7,
   where the lexer violates the helper's documented delimiter invariant.
   Upstream agrees:
   the issue carries `S-Bug-confirmed`.
2. **Upstream fixability**:
   yes,
   and already fixed in source by emitting `ERROR_TOKEN` for unterminated strings.
3. **Supported use case**:
   yes;
   the helper is public API on typed string values,
   and the tree it receives comes from upstream's own parser.
4. **Contribution welcome**:
   yes with disclosure.
   `CONTRIBUTING.md:47-69` in the upstream clone requires disclosing any AI assistance in a pull request
   and gives an example disclosure;
   it does not ban outside or AI-assisted contributions.
5. **Likely to fix**:
   already merged upstream.
   The remaining gap is release timing for the published Rust crates,
   which upstream documents as on-demand
   (`RELEASES.md:35-36` in the same clone).
6. **Prototype**:
   yes for the helper-side clamp,
   red/green verified in bounded containers.
   It is now redundant with upstream's lexer fix,
   so it is kept only as a fallback for consumers pinned to published 0.5.7.

Nothing was posted upstream.
 The only content this audit could add to the closed thread is the published-crate exposure,
 which is a release-cadence request rather than a new defect.
 Sending it would be an external communication and needs separate authorization,
 so the draft comment below is **do not post as-is**:

~~~md
The lexer fix from #2621 is not in any published `biome_json_parser` release yet:
crates.io still resolves 0.5.7 (published before the fix),
so Rust consumers that pin published crates can still reach the
`inner_string_text` panic on a one-byte `"` source.
`biome_json_syntax` 0.7.0 is published but keeps the same unguarded range
arithmetic, and `biome_json_parser` 0.5.7 requires the 0.5.7 syntax line,
so the helper cannot be upgraded independently.

Measured with published 0.5.7 in a bounded container:
parsing `"` succeeds with a `Missing closing quote` diagnostic,
the tree completes a normal `JSON_STRING_VALUE`,
and `inner_string_text` panics with
`assertion failed: start.raw <= end.raw` at `biome_text_size-0.5.8/src/range.rs:66`.
Callers that check `has_errors()` first are unaffected.

Would publishing a `biome_json_parser` release that includes #2621 be
feasible, or should downstream consumers treat 0.5.7 as requiring a
diagnostics check before using `inner_string_text`?
~~~

[issue-2357]: https://github.com/biomejs/biome/issues/2357
[pr-2621]: https://github.com/biomejs/biome/pull/2621
