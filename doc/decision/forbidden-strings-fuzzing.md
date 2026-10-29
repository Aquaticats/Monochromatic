# forbidden-strings fuzzing tooling

## Status

Accepted.
 Plan:
 `~/.claude/plans/setup-fuzzing-for-forbidden-strings-merged.md`.

## Context

`package/forbidden-strings/cli` is a deny-list scanner that mixes a custom
regex prefix extractor,
 Aho-Corasick gates,
 a hybrid `regex` + `resharp`
engine dispatcher,
 and rayon-parallel scans.
 The recent regression history
(`e49d8694`,
 `e100659f`,
 `9b41fca0`,
 `1463c59b`,
 `0479371a`,
 `4289cdb3`)
shows that the bug class is "the extractor or dispatcher constructs a
gate that drops legitimate matches" rather than "the fast path is too
slow".
 Random-input testing has missed every one of those bugs;
 what we
need is coverage-guided fuzzing that walks structural branches in the
extractor and engine dispatcher.

Update (#386,
 engine swap):
 the extractor,
 Aho-Corasick gates,
 and hybrid dispatcher above were deleted in the #385 teardown;
 the scanner now runs on the in-house `forbidden-regex` engine.
 The scanner-level fuzz suite was retargeted onto the surviving surfaces:
 the literal-to-verbose-dialect escaper (`fuzz_literal_roundtrip`),
 the strict two-form rule loader (`fuzz_ruleset_scan_invariants`),
 and the columnless `PATH:LINE rule=N` scan output (`fuzz_scan_format`).
 The three removed-machinery targets (aho-corasick gate soundness,
 residual shards,
 engine dispatch) and the regex-syntax-walker target were removed with the
code they fuzzed,
 as was the test-literal seeder.
 Engine-level fuzzing (the `RegexSet` compile,
 serialize,
 and `line_matches`
paths) moved to the engine's own sidecar at
`package/forbidden-strings/core.fuzz` and is not duplicated here.

## Decision

Use `cargo-fuzz` (libFuzzer backend) as the primary fuzzer for this
package.
 Pin `cargo-fuzz` and a nightly Rust toolchain via mise (root
`mise.no-env.toml`).
Express the fuzz surface through a `fuzz_api` module behind a Cargo
`fuzzing` feature;
 do not widen the production `pub` surface.

CI integration is deferred.
 Targets are exercised locally and on demand
only,
 inside a bounded container.

## Rejected alternatives

- `proptest`:
   a property-based testing framework using uniform random
  generation per type.
   It misses adversarial input shapes the way
  coverage-guided fuzzing finds them,
   and the regression class above is
  exactly the kind of branch a uniform sampler will miss.
   We may revisit
  if cargo-fuzz surfaces a fast deterministic invariant worth pulling
  into a normal `cargo test` run;
   until then `proptest` adds a second
  generator surface for no fuzzing gain.
- `afl.rs`:
   AFL bindings for Rust.
   AFL has a different corpus format
  from libFuzzer and weaker integration with the `arbitrary` derive
  macro that the structured generator depends on.
   No measurable
  advantage on a libFuzzer-friendly codebase like this one.
- `honggfuzz-rs`:
   Honggfuzz bindings for Rust.
   Requires the `honggfuzz`
  binary as a separate toolchain,
   with little to no advantage over
  libFuzzer for this code shape.
   Adds an install path without a
  matching capability gain.
- `bolero`:
   a unifying frontend over cargo-fuzz,
   kani,
   and honggfuzz.
  Useful if we eventually want symbolic execution (e.g. `kani` for
  parser proofs) and a second random backend in the same harness,
   but
  today we have one backend (`cargo-fuzz`) and adding the abstraction
  layer just adds a dependency on bolero's API surface.
   Revisit if we
  add a second backend.

## Redaction rule

Fuzz reproducers and panic messages never echo raw bytes from the
content slice or the rule source.
 Every reproducer field is one of:

- input length in bytes;
- a SHA-256 hex digest of the input (the `redacted_fingerprint` helper);
- a one-line label naming the invariant that failed.

The columnless output contract makes redaction structural:
 a finding is exactly `PATH:LINE rule=N`,
 interpolating only the fixed path and two integers,
 so no rule byte or content byte can reach it.

Seed corpora are hand-curated (there is no automated seeder anymore).
 Contents must never contain a raw secret-shaped string,
 because the tracked `seed/` files are scanned by the repo commit gate;
 use the crate's byte-escape convention if a fixture ever needs one.
 The local deny-list (`forbidden-strings.local.txt` and its append sibling)
must never enter a seed,
 the dictionary,
 or a reproducer.

## CI integration

Deferred to a later plan.
 The Done criteria for this work do **not**
include CI wiring.
 Reasons:

- libFuzzer + nightly Rust both need extra build steps that change
  cache shape on GitHub Actions;
   doing that in the same PR as the
  scaffold mixes two unrelated concerns.
- The fuzz smoke target uses a bounded container wrapper (Resource-
  exhaustion isolation rule in `AGENTS.md`);
   the wrapper needs an
  image we have not yet pinned for CI.

The follow-up CI plan should pick a Rust + LLVM container image,
decide a fuzz-time budget per target,
 and add a separate workflow file
(do not piggyback onto `.github/workflows/forbidden-strings.yml`,
 which
runs on every push and would balloon the per-push cost).
