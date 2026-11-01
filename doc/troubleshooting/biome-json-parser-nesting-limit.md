# Biome JSON parser 0.5.7: depth 513 parses despite the editor's 512-container boundary

## Symptom

The native `jsonc-edit` port needs to accept 512 nested JSONC containers and reject a 513th opener.
 In a bounded probe,
 `biome_json_parser::parse_json` returned `has_errors() == false` for both array depths.
 It also returned no syntax error for 512 nested object members.
 An editor that treats the upstream syntax result as its whole acceptance contract would admit the unsupported depth.
 The probe did **not** observe a stack abort at these depths.

## Root cause

The published `biome_json_parser-0.5.7/src/parser.rs:16-30` exposes only comment and trailing-comma options:

```rust
// biome_json_parser-0.5.7/src/parser.rs:16-29
pub struct JsonParserOptions {
    pub allow_comments: bool,
    pub allow_trailing_commas: bool,
}
```

The published `biome_json_parser-0.5.7/src/syntax.rs:152-230` drives nested arrays and objects with an explicit stack.
 It suspends the parent and resumes it after the child,
 with no 512-depth rejection in this path:

```rust
// biome_json_parser-0.5.7/src/syntax.rs:152-155,210-229
let mut stack = Vec::new();
let mut current = start_sequence(p, root_kind);
// ...
SequenceItem::Recurse(kind, marker) => {
    current.state = SequenceState::Suspended(marker);
    stack.push(current);
    current = start_sequence(p, kind);
    continue 'sequence;
}
// ...
match stack.pop() {
    None => return Present(node),
    Some(next) => current = next,
};
```

A search over the published parser's `src/` for a maximum nesting option or limit found `parse_sequence` calls,
 but no depth-setting branch.
 This is a **contract difference**,
 not evidence of a defect in Biome.
 The editor's existing structured parser uses a 512-depth boundary at
 `package/jsonc-edit/core-ts/src/parse.ts:21,54`.
 The upstream clone `~/temp/agent/biome-source-2026-09-24` at `2f629c5` still has the explicit-stack path in
 `crates/biome_json_parser/src/syntax.rs:159-238`;
 the published archive remains the authority for version 0.5.7.

## Verification

The published parser archive SHA-256 is
 `9c6d23fb9b683e6356c094b4a0cb38f8aa0acee60ce9c3ef24628d21a204de4d`,
 matching the scratch Cargo.lock checksum.
 Run `mise run test:isolated` from `~/temp/agent/jsonc-regex-audit/biome/` to repeat the public-API tests
 under the offline 2 GiB/2 CPU execution manifest `~/temp/agent/biome-execution-manifest.md`.
 Bounded debug and optimized release suites passed all focused tests after the consumer guard was added.

- Accepted case:
   `"[".repeat(512) + "0" + "]".repeat(512)` parses upstream and through the editor guard.
- Accepted case:
   the corresponding 512-record chain parses,
   its syntax text can be read,
   and the syntax tree drops without a stack abort in the bounded run.
- Upstream acceptance outside the editor contract:
   the 513-array chain parses without diagnostics;
   `parse_for_editor` returns `JSONC nesting too deep`.
- Another editor-contract check:
   upstream accepts scalar root `42` as JSON,
   while `parse_for_editor` returns `JSONC root must be an array or object`.
- False-positive control:
   `{"[":"{"} /*[*/` passes the guard;
   bracket characters in strings and comments do not count as containers.

## Verified workaround

The disposable adapter parses with Biome,
 rejects upstream diagnostics and scalar roots,
 then walks the grammar tokens without recursion.
 Its relevant depth branch is:

```rust
// ~/temp/agent/jsonc-regex-audit/biome/lib.rs
let mut depth: usize = 0;
for token in parsed.syntax().descendants_tokens(biome_rowan::Direction::Next) {
    if token.text_trimmed() == "[" || token.text_trimmed() == "{" {
        depth += 1;
        if depth > 512 {
            return Err(String::from("JSONC nesting too deep"));
        }
    } else if token.text_trimmed() == "]" || token.text_trimmed() == "}" {
        depth = depth.checked_sub(1).ok_or("unmatched JSONC close delimiter")?;
    }
}
```

The syntax-error check runs **before** the token walk,
 so skipped malformed tokens do not become trusted delimiters.
 `text_trimmed()` excludes comment trivia and keeps quoted strings quoted;
 the false-positive control above passed.
 This is a consumer-side acceptance gate,
 not an upstream parser configuration.

**Tradeoff:**
 This gate runs after Biome has allocated the parsed syntax tree.
 It establishes the editor's accepted/rejected boundary at the tested depths,
 but is not an early resource limit for arbitrary adversarial nesting.
 A separately validated lexical preflight or an input-size policy would be needed
if early resource rejection is required.
 It also does not implement attached comments,
 exact numeric values,
 or immutable edits.

## What does not work

- Setting `JsonParserOptions` cannot select a maximum depth in version 0.5.7;
   its published fields are limited to comments and trailing commas (`biome_json_parser-0.5.7/src/parser.rs:16-30`).
- Counting raw `[` and `{` bytes treats delimiters inside quoted strings or comments as containers.
   The verified guard uses grammar-token text instead.
- Recursing through every nested syntax node for a postparse guard could reintroduce a stack risk at the accepted depth.
   The verified guard uses an iterator.

## Upstream filing decision

`.out-of-scope/` has no Biome or JSON nesting exemption.
 `gh search issues 'json parser maximum nesting depth' --repo biomejs/biome --include-prs --limit 20`
 returned no results.
 No upstream report is proposed:

- **Upstream fault:**
   No.
   Biome does not promise this editor's 512-container limit.
- **Can upstream change it:**
   Yes.
   An optional depth policy could be added,
   but no upstream correctness fix is necessary for the observed behavior.
- **Supported use case:**
   Parsing JSON and JSONC is supported;
   this consumer's exact depth boundary is not an advertised option.
- **Contribution policy:**
   Not evaluated for a proposed patch because the consumer guard meets the measured acceptance need.
- **Likelihood of an upstream fix:**
   Not evaluated without a requested upstream feature.
- **Minimal upstream prototype:**
   Not applicable.
   The tested prototype is at the consumer boundary.

## Upstream filing artifact

Do not file as-is.
 A future request for an optional parser limit would be a feature request,
 not a report that Biome is broken:

~~~md
# Optional maximum nesting policy for JSON parsing

A consumer currently checks its depth limit after `parse_json`, because `JsonParserOptions` has no maximum-depth field.
 An optional limit enforced when `parse_sequence` pushes a new frame could allow early rejection for callers with a strict
 accepted-depth contract. The current unbounded behavior is valid for other consumers; no default change is requested.
~~~
