# Fuzzing forbidden-strings

This sidecar lives at `package/forbidden-strings/cli.fuzz`,
beside its owner at `package/forbidden-strings/cli`.
Fuzz sidecars use `package/<category>/<name>.fuzz`.

Coverage-guided fuzzing for the scanner's own surfaces after the engine swap
(#383/#384/#385):
 the literal-to-verbose-dialect escaper,
 the strict two-form rule loader,
 and the columnless `PATH:LINE rule=N` scan output.
 Engine-level fuzzing (the `RegexSet` compile,
 serialize,
 and `line_matches`
paths) lives in the engine's own sidecar at
`package/forbidden-strings/core.fuzz` and is not duplicated here.
 CI integration is **deferred**;
 targets are exercised locally and on demand only.
 The rationale and rejected alternatives are recorded in
[`doc/decision/forbidden-strings-fuzzing.md`](../../../doc/decision/forbidden-strings-fuzzing.md).

## Prerequisites

- Nightly Rust toolchain.
   `cargo-fuzz` injects `-Cllvm-args=-sanitizer-coverage-*` flags that only work
  on nightly,
   and its default AddressSanitizer path emits nightly-only `-Zsanitizer`
  (see [`doc/troubleshooting/cargo-fuzz-nightly-rust.md`](../../../doc/troubleshooting/cargo-fuzz-nightly-rust.md)).
   The toolchain is managed repo-wide in the root
  [`mise.toml`](../../../mise.toml) `[tools]` entry
  (`rust = { version = "nightly", components = "rust-src,llvm-tools-preview" }`),
   so `mise install` provisions nightly (with std sources and the llvm coverage
  tools) for every crate,
   and tasks run here inherit it.
- `cargo-fuzz`.
   Installed automatically by `mise install` from the root
  [`mise.toml`](../../../mise.toml) `[tools]` entry.
- A C++ toolchain (clang or gcc).
   `libfuzzer-sys` builds a small C++ runtime that the targets link against.
- `podman` (or `docker`) for the bounded-container wrapper described below.

## Target list

Each target encodes one invariant.
 Failures are libFuzzer crashes with a redacted reproducer message
(input length plus SHA-256,
 never raw bytes).

The targets share a single tuned dictionary at
`dictionary/forbidden-strings.dict` (one dictionary,
 not per-target).
 Each target has its own per-target seed corpus under `seed/<target>/`
(tracked),
 separate from the gitignored scratch corpus under `corpus/<target>/`;
 fuzz invocations pass both as libFuzzer corpus dirs with scratch first,
 so new discoveries land only in scratch.

### `fuzz_literal_roundtrip`: syntax-boundary transformer

   Drives the real `escape_literal` over arbitrary strings and compiles its
  output as a single-pattern `RegexSet`:
   the escaped pattern must compile,
   must match the literal itself,
   and must never match empty input (the comment-swallow / empty-matchable
  regression guard).
   A fixed adversarial battery runs once per process so the always-verbose
  boundary cases (spaces,
   leading `#`,
   quotes,
   backslashes,
   metacharacters,
  escape-sequence lookalikes,
   newlines) are always exercised.
### `fuzz_scan_format`: columnless output contract

   Loads a generated two-form ruleset and scans a generated multi-line buffer,
   asserting every finding is exactly `PATH:LINE rule=N` (or the fail-closed
  `PATH: engine error`),
   the line index is 1-based and within the buffer,
   findings arrive line-ascending,
   and no content byte leaks into the finding (redaction is structural:
   the formatter interpolates only the fixed path and two integers).
### `fuzz_ruleset_scan_invariants`: strict loader plus scan invariants

   Drives `load_from_text` and `scan_file` over a generated ruleset and buffer:
   a rejected flag fails the load closed,
   `m`/`x` are no-ops,
   a file with no rule line loads nothing (a fixed loader-contract battery pins
  the flag policy every process),
   and reversing the rule order renumbers ids but never changes which positions
  match (rule-order invariance).
### `fuzz_embedding`: exact native snapshots

  Drives the public scanner over arbitrary Unix pathname bytes and content snapshots.
  Independent fixed-rule byte searches predict every content and pathname finding,
  including binary-prefix cutoffs.
  Unexpected `EngineError` records fail the oracle rather than being filtered away.
  Positive controls cover both hybrid matcher subsets,
  safe display encoding,
  and matching-component redaction.
  Committed seeds place NUL at either side of the 8192-byte boundary
  and place a literal across the cutoff.
  This target is Unix-only and deliberately does not exercise filesystem cache loading.
### `fuzz_cache_envelope`: hostile compiled-artifact framing

  Drives the scanner-owned runtime cache decoder with arbitrary artifact bytes and
  arbitrary authoritative source bytes.
  Every malformed magic value,
  length,
  identity,
  name marker,
  digest,
  and engine payload must return a verdict without panic or out-of-bounds access.

### `fuzz_rules_list`: rules-list grammar

  Drives `forbidden_strings::parse_rules_list`, the one parser behind `--rules`, `FORBIDDEN_STRINGS_RULES`,
  and `compile-rules --rules`, with raw UTF-8 text and with lists built from delimiter, marker, and path tokens.
  The oracle in `src/rules_list.rs` compares every verdict (entries, optional flags, refusal kind and position)
  with an independently written reference parse,
  requires a successful parse to render back to the exact input,
  and pins the single-path case (no comma and no trailing `?`) to one required entry equal to the whole value.
  Committed seeds cover the shared plus optional local pair, a doubled marker, an empty entry, a repeated path,
  and a plain path with a percent sign.

## Local commands (via mise)

All commands run on nightly automatically:
 the repo-root `rust = nightly` tool applies its toolchain to every task.

```bash
# List every target name.
mise run //package/forbidden-strings/cli.fuzz:list

# Type-check the shared generator library and its unit tests.
mise run //package/forbidden-strings/cli.fuzz:test

# Build every target.
mise run //package/forbidden-strings/cli.fuzz:build

# 30-second smoke run across every target (uses the dictionary).
mise run //package/forbidden-strings/cli.fuzz:smoke

# Single-target campaign with passthrough libFuzzer args.
mise run //package/forbidden-strings/cli.fuzz:run fuzz_literal_roundtrip -- -max_total_time=120
```

## Bounded embedding campaign

Run the embedding task through its owning container runner:

```sh
# package/forbidden-strings/cli.fuzz/mise.toml
mise run //package/forbidden-strings/cli.fuzz:smoke:embedding:container
mise run //package/forbidden-strings/cli.fuzz:lint:clippy:container
```

The runner bakes source,
committed seeds,
vendored locked dependencies,
the installed nightly compiler,
and `cargo-fuzz` into an immutable image.
It runs without network or host mounts as user `1000:1000`,
with 2 GiB memory,
2 CPUs,
and 128 PIDs.
AddressSanitizer is explicit.
The campaign uses a 120-second budget,
20,000-byte maximum input,
10-second per-input timeout,
and 1536 MiB libFuzzer RSS limit.
Exact commands,
source hashes,
image IDs,
transcripts,
exit status,
corpus,
and artifacts remain under `target/verification/embedding-fuzz-*`.
Failed evidence retrieval retains the disposable container.

The sidecar's all-target Clippy gate also runs in that mount-free environment.
Other existing fuzz tasks still require a bounded container or builder VM;
the embedding task does not claim coverage of every other target.
See [native scanner verification](../../../doc/handover/scanner-native-verification.md)
for exact campaign results and limitations.

## Corpus and artifact policy

- `seed/<target>/seed-*` -- curated seed files committed to the repo (tracked
  plainly;
   no `.gitignore` re-include needed).
   Hand-curated:
   the `fuzz_literal_roundtrip` seeds are the adversarial escaping cases;
   the format-driven targets carry a few small two-form-shaped byte seeds the
  `Arbitrary` decoder expands into a starting corpus.
   Seed contents must never contain a raw secret-shaped string (the tracked
  files are scanned by the repo commit gate);
   use the crate's byte-escape convention if a fixture ever needs one.
- `corpus/<target>/` -- libFuzzer's corpus growth (scratch).
   Wholly ignored.
   The smoke and run tasks pass `corpus/<target> seed/<target>` as explicit
  corpus dirs;
   libFuzzer reads both and writes new discoveries only to the first.
- `artifacts/` -- libFuzzer's crash reproducers.
   Always ignored.
   Open one with `cat artifacts/<target>/crash-*` ONLY in a private session;
   the artifact bytes are the exact fuzzer-mutated input that triggered the
  crash and may contain secret-shaped bytes.
- `coverage/` -- coverage data.
   Ignored.
- `Cargo.lock`:
   Committed so the fuzz toolchain stays reproducible.

The local deny-list (`../cli/forbidden-strings.local.txt`)
and its append sibling must NEVER enter the corpus,
 dictionary,
 or reproducer text.
 There is no automated seeder;
 hand-curated corpus / dictionary / reproducer edits are entirely the
contributor's responsibility.

## Crash reproduction and minimization

When a target reports a crash:

1. The reproducer lives at `artifacts/<target>/crash-<sha>`.
    Note its path.
2. Re-run the exact input to confirm:
   ```bash
   mise run //package/forbidden-strings/cli.fuzz:run \
     <target> -- artifacts/<target>/crash-<sha>
   ```
3. Minimize it via `cargo +nightly fuzz tmin <target> artifacts/<target>/crash-<sha>`
   -- libFuzzer finds the smallest input that still reproduces the panic.
4. Format the minimised input for readability via
   `cargo +nightly fuzz fmt <target> <path>` (decodes the bytes through the
   target's `Arbitrary` impl and prints the structured form).
5. **Redaction**:
    when filing an issue or commit message about the crash,
    NEVER paste raw reproducer bytes.
    The panic message every target prints already includes the redacted
   fingerprint (input length plus SHA-256) -- that is the reproducer shape to
   share.

## Fail-closed boundary

The scanner wraps every `RegexSet::line_matches` call in `std::panic::catch_unwind`
(`src/frx_scan.rs`);
 the engine's own `Cargo.toml` documents that it deliberately installs no
internal panic guard and expects this caller to provide the boundary.
 A caught engine panic surfaces as the redacted synthetic finding
`PATH: engine error`,
 which the format-driven targets accept as one of the two documented output
shapes.
 A genuine panic in a target's own assertion code calls the default hook and
`abort()`s,
 so libFuzzer records a crash.
 `panic = "unwind"` is pinned in this crate's `Cargo.toml` so that boundary keeps
working under the sanitizer build.
