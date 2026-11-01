# Vetting: Rust QA trio for S3-gateway translation logic

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Scope:
 proptest (property testing),
 cargo-mutants (mutation testing),
 cargo-fuzz
(coverage-guided fuzzing).
 Serves both the Slint+Rust and Tauri v2 stacks.
Standard:
 FULL-VERIFICATION (choosing-technology skill).
 Container only,
 no device.
Date:
 2026-06-07.

## 1. Source and ecosystem audit

Clones (shallow,
 `/tmp/agent/`):

- `/tmp/agent/proptest-vet` (proptest-rs/proptest),
   last commit 2026-05-26.
- `/tmp/agent/cargo-mutants-vet` (sourcefrog/cargo-mutants),
   last commit 2026-06-06.

### proptest: strategy + shrinking model

Two core traits in `proptest/src/strategy/traits.rs`:

- `Strategy` (line 37):
   `type Value`,
   `type Tree: ValueTree`,
   and
  `fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self>` (line 60).
  A strategy is a value generator parameterised by the runner PRNG,
   composable via
  `prop_map`,
   `prop_filter`,
   `prop_flat_map`,
   `prop_recursive`,
   unions,
   etc.
- `ValueTree` (line 580):
   `current()`,
   `fn simplify(&mut self) -> bool` (line 597),
  `fn complicate(&mut self) -> bool` (line 620).

This is integrated,
 type-directed shrinking:
 each generated value carries its own
shrink tree.
 On failure proptest repeatedly calls `simplify()` to walk toward a
minimal failing input,
 then `complicate()` to backtrack when a simplification step
overshoots past the failure boundary (doc comment at `traits.rs`:
578-43 spells out the
simplify-then-complicate contract).
 Failures are persisted as regression seeds
(`proptest-regressions/`).
 This integrated model is proptest's defining advantage over
quickcheck's separate `Arbitrary::shrink` iterator.

Self-tests use `proptest!` across many modules (sugar,
 array,
 char,
 num,
 sample,
arbitrary,
 failure_persistence).
 CI:
 `.github/workflows/rust.yml`.
 MSRV 1.86,
edition 2021,
 dual Apache-2.0 / MIT.

### cargo-mutants: mutator set and Rust handling

`Genre` enum in `src/mutant.rs` (line 23):
 `FnValue`,
 `BinaryOperator`,
`UnaryOperator`,
 `MatchArm`,
 `MatchArmGuard`,
 `StructField`.

Operators are AST-level,
 not byte-level.
 cargo-mutants parses each source file with
`syn` and emits replacements with `quote`,
 then recompiles + reruns the test suite per
mutant.

- Binary/unary operator table,
   `src/visit.rs:585-610`:
   `==`<->`!=`,
   `&&`<->`||`,
  `<` -> `==`/`>`/`<=`,
   `+` -> `-`/`*`,
   `-` -> `+`/`/`,
   bit/shift/assign variants,
   and
  `UnOp::Not`/`Neg` removal.
- FnValue type-directed body replacement,
   `src/fnvalue.rs:return_type_replacements`
  (line 21):
   inspects the return type and substitutes type-appropriate values:
  `Default::default()`,
   `Ok(Default::default())` / `Ok(#rep)`,
   `Err(#error_expr)`,
  `None` / `Some(#rep)`,
   `vec![]`,
   `""` / `"xyzzy"`,
   NonZero,
   Option/Vec/Cow,
  known container/collection/map types,
   and even `HttpResponse::Ok().finish()`.

Rust-awareness:
 it deliberately avoids deleting `_ =>` catch-all arms and guarded arms
(`visit.rs`:
660-672) because those would not compile,
 reducing unviable-mutant noise.
Known limitation (confirmed in run below):
 it mutates code regardless of `#[cfg(...)]`
gating;
 the maintainer notes cfg handling will change (book note,
 commit f7082f5).
Self-tests live in `tests/` (integration_util,
 snapshots);
 CI `.github/workflows/tests.yml`.
MIT licence.

### crates.io availability and versions

- proptest 1.11.0 (updated 2026-03-24),
   134,664,817 downloads.
- cargo-mutants 27.1.0 (updated 2026-06-02),
   336,215 downloads.
- cargo-fuzz 0.13.1 (updated 2025-06-27),
   3,164,362 downloads.
   Matches repo pin
  (`mise.toml:51` `"cargo:cargo-fuzz" = "0.13.1"`).

## 2. Maintenance signals (gh)

### proptest

- 2156 stars,
   214 forks.
   Latest publish 1.11.0 (2026-03).
- Recent issue triage (sampled #592-#646,
   12 months):
   members cameron1024,
   rexmas,
  tzemanovic,
   wojciech-graj respond within days,
   label,
   close,
   and link PRs.
   Example:
  #642 (feature request) answered by cameron1024 in 1 day with a concrete workaround;
  #630 (regression) accepted,
   PR #640 landed in ~5 weeks;
   #601 regression acknowledged
  and fixed in 6 days.
- State:
   active releases,
   responsive maintainers,
   small triaged backlog.
   Healthy.

### cargo-mutants

- 1185 stars,
   41 forks.
   Latest release v27.1.0 (2026-06-02),
   automated via release-plz.
- Owner sourcefrog (Martin Pool) triages personally within hours to days:
   #619 closed
  same day;
   #611 (macOS temp-reaper bug) diagnosed and fix accepted in a week;
   #614
  (advisory on `atty`) triaged with reasoned risk assessment;
   #610 dup closed same day.
- State:
   single highly-active maintainer,
   fast triage,
   frequent releases.
   Healthy,
   with
  the usual bus-factor caveat of a one-maintainer project (mitigated by MIT licence and
  a clean,
   syn-based codebase that is forkable).

## 3. Full verification (podman, bounded)

Container:
 `docker.io/library/rust:1-bookworm`,
 `--memory=6g --memory-swap=6g
--cpus=4`,
 volume `/tmp/agent/s3vet:/work:Z` (SELinux relabel needed on the Fedora
host),
 build dirs on `/var/tmp` (not tmpfs) via `CARGO_TARGET_DIR`/`TMPDIR`.
Toolchain:
 rustc 1.96.0 stable;
 nightly 1.98.0-nightly (61d7280f3) installed for
cargo-fuzz.
 Memory cap confirmed:
 `memory.max = 6442450944`.

Fixture crate `s3vet`:
 a real S3 ranged-GET translator
`parse_byte_range(header, size) -> Result<(u64,u64), RangeError>` resolving HTTP
`Range: bytes=` specs (explicit `a-b`,
 open `a-`,
 suffix `-n`,
 multi-range reject,
416 unsatisfiable,
 clamping) into inclusive offsets.
 A `plantbug`-gated twin
`parse_byte_range_buggy` returns `size` instead of `size-1` in the suffix branch.

### (a) proptest

Build pulled proptest 1.11.0.
 Baseline suite (`cargo test`):
 11 unit tests + the
holding property `prop_valid_range_in_bounds` all pass.

```text
running 12 tests
test tests::prop_valid_range_in_bounds ... ok
test result: ok. 12 passed; 0 failed; ...
```

Planted-bug property (`cargo test --features plantbug prop_buggy_suffix_violates_bound`)
fails and shrinks to the minimal counterexample,
 persisting a regression seed:

```text
test tests::prop_buggy_suffix_violates_bound ... FAILED
proptest: Saving this and future failures in /work/proptest-regressions/lib.txt
cc c464e09d26d786dee7ed7565c139e814b63098ac28882c4a271d1df3c69c6b47
thread '...' panicked at src/lib.rs:161:5:
Test failed: end 1 not < size 1 at src/lib.rs:172.
minimal failing input: size = 1, suffix = 1
```

Result:
 pass.
 Integrated shrinking reduced random inputs (size,
 suffix up to 100k) to
`size = 1, suffix = 1` and wrote a reproducible seed.

### (b) cargo-mutants

Installed cargo-mutants v27.1.0 (`cargo install --locked`,
 49s).
 First full run found
33 mutants but also mutated the cfg-gated buggy twin (cfg-blind,
 see limitation above),
so all 15 of its mutants showed trivially MISSED.
 Re-ran scoped to the real parser:

```text
cargo mutants --no-shuffle --exclude-re "parse_byte_range_buggy" -v
Found 18 mutants to test
ok       Unmutated baseline ...
caught   src/lib.rs:28:5: replace parse_byte_range -> ... with Ok((0, 0))
caught   src/lib.rs:36:24: replace || with && in parse_byte_range
caught   src/lib.rs:44:14: replace >= with < in parse_byte_range
caught   src/lib.rs:51:22: replace < with == in parse_byte_range
caught   src/lib.rs:51:22: replace < with > in parse_byte_range
MISSED   src/lib.rs:51:22: replace < with <= in parse_byte_range
... (16 total caught)
18 mutants tested in 4s: 1 missed, 16 caught, 1 unviable
```

Result:
 pass,
 with a real finding.
 The single MISSED mutant (`< -> <=` on
`requested < start`) is a genuine test-gap:
 neither the example tests nor the property
pin the `start == end` boundary (a single-byte `bytes=5-5` range).
 This is exactly what
mutation testing is meant to surface and demonstrates the tool works end to end.

### (c) cargo-fuzz (bounded)

Installed cargo-fuzz 0.13.1 (matches repo pin).
 Bounded run,
 5 seconds:

```text
cargo +nightly fuzz run range -- -max_total_time=5 -rss_limit_mb=4096
#6908390 DONE   cov: 116 ft: 228 corp: 110/2306b exec/s: 1151398 rss: 498Mb
Done 6908390 runs in 6 second(s)
###### Recommended dictionary ######
"bytes=" / "bytes0" / "b=s|ye" ...
```

Result:
 pass.
 6,908,390 executions,
 coverage-guided (libFuzzer reverse-engineered the
`bytes=` prefix into its recommended dictionary),
 zero crashes,
 no ASan error,
 no
invariant violation.
 RSS peaked at 498 MB,
 well within the 6 GB cap.
 Confirms the
parser is panic-free on arbitrary bytes and that cargo-fuzz 0.13.1 runs cleanly in the
container context the repo already uses (`package/forbidden-strings/cli.fuzz`,
 7 targets,
corpus + dictionaries).

Nothing failed to run.
 The only friction was the SELinux `:Z` relabel on the bind
mount (Fedora host),
 resolved.

## 4. Alternatives with rejection reasons

### quickcheck vs proptest

quickcheck 1.1.0 (2026-02-10),
 56M downloads,
 still maintained.
 Reject for this repo:
shrinking is a separate `Arbitrary::shrink` method returning a `Box<dyn Iterator>`,
not integrated into generation,
 so shrinking is opt-in per type,
 easy to omit or get
wrong,
 and decoupled from the strategy that produced the value.
 quickcheck has no
first-class strategy combinators (no `prop_compose!`,
 range/regex strategies,
 weighted
unions) and no built-in failure persistence / regression seeds.
 proptest's integrated
ValueTree simplify/complicate produced a clean minimal counterexample and a reproducible
seed in the run above;
 that is the capability the S3-gateway invariants need.

### mutagen vs cargo-mutants (no real equivalent)

mutagen 0.1.2,
 last published 2018-10-10 (about 8 years stale),
 10,482 downloads.
Reject:
 abandoned,
 relies on an old nightly proc-macro source-rewrite approach that no
longer builds on current toolchains.
 There is no other maintained Rust mutation tester;
cargo-mutants is effectively the only viable option,
 and it is actively released
(v27.1.0,
 2026-06-02).
 This is a single-candidate category,
 vetted on its own merits
above rather than against a live competitor.

### afl.rs / honggfuzz vs cargo-fuzz

afl 0.18.2 (2026-05-11) and honggfuzz 0.5.60 (2026-03-28) are both maintained.
 Reject
for this workload:
 both are out-of-process fuzzers requiring a separate external binary
(`cargo afl` + the AFL++ toolchain;
 the `honggfuzz` binary) and external corpus/harness
management.
 cargo-fuzz/libFuzzer is in-process and coverage-guided with the simplest
`fuzz_target!(|data: &[u8]|)` ergonomics,
 links ASan by default,
 and is already the repo
incumbent (pinned 0.13.1,
 nightly,
 7 targets with corpora and dictionaries).
 For
fuzzing pure library functions that take structured byte input (exactly the S3 parser
case) libFuzzer is the right fit;
 switching to `afl.rs` or honggfuzz would add operational
overhead (extra binaries,
 separate instrumentation/corpus flows) with no coverage
benefit.
 Keep cargo-fuzz.

## Verdict

Adopt the trio:
 proptest 1.
x for property tests,
 cargo-mutants 27.
x for mutation
testing,
 cargo-fuzz 0.13.1 (incumbent) for fuzzing.
 All three built,
 ran,
 and produced
the expected evidence in a bounded container;
 all three are actively maintained with
responsive triage;
 alternatives each fail on a concrete incompatibility.
 The trio is
stack-agnostic (plain cargo) so it serves both the Slint+Rust and Tauri v2 sides.
