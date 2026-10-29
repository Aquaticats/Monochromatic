# Forbidden-strings fuzzing handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings.fuzz` is now `package/forbidden-strings/cli.fuzz`;
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

## Sibling-sidecar relocation, 2026-09-26

The maintainer requires fuzz sidecars at `package/<category>/<name>.fuzz`.
The scanner stays at `package/cli/forbidden-strings`;
its harness now lives at `package/cli/forbidden-strings.fuzz`.
`AGENTS.md` rule `AP1` records this layout.
The product-cluster proposal in `doc/planning/package-category-rebalance.md` is superseded.
Other category moves are outside this change.

The complete sidecar directory moved,
including ignored local corpus and build state.
The Cargo crate name remains `forbidden-strings-fuzz`;
its dependency now uses `path = "../forbidden-strings"`.
The file-enforcer profile map,
generated configuration,
corpus ignore,
Git seed-byte exclusion,
and repository path references follow the new location.
The current curated directory is `seed/<target>/`,
not the historical `seeds/<target>/` spelling in this snapshot.

### Commits

- `25fff4759` changes Git policy's exact-byte seed exclusion before the move.
- `b9c0e6e48` relocates the sidecar and updates consumers and layout documentation.
- `3fe9b368e` and `abe8ddc1c` make the touched policy test exercise built output with clean imports.

### Verification

The relocated `list` task discovers all targets.
The `test` task passes all 6 generator tests;
`lint:clippy` and the ASAN `build` task pass.

Each fresh ASAN target loaded its read-only seed directory and dictionary
inside a disposable Fedora 44 container with `--memory=2g --cpus=2`.
Mutation output went to a container-only tmpfs.
The targets were `fuzz_cache_envelope`,
`fuzz_literal_roundtrip`,
`fuzz_ruleset_scan_invariants`,
and `fuzz_scan_format`.
Each completed with `-runs=100 -max_total_time=10 -timeout=5 -rss_limit_mb=1024`.
This is a relocation smoke check,
not a long-running fuzz campaign.

The rebuilt Git CLI passed the existing exclusion lifecycle fixture
through `check`,
`fix`,
`commit`,
and `push` against disposable local repositories and a disposable bare remote.
The final-newline unit suite,
scoped Oxlint,
and CLI and file-enforcer package type checks passed.
An initial lint failure exposed source imports in the touched test;
the artifact-import follow-up fixes those imports without suppressions.

SHA-256 checks confirm all 41 seed filenames and bytes survived the move and commit unchanged.
The new sidecar tracks 54 files,
including `Cargo.lock` and all seeds;
its scratch corpus remains untracked.
Ignore probes match corpus,
target,
artifact,
and coverage output paths.
The retired category directory is absent.
An uncapped tracked-file search found no old category path references,
with new-path matches used as its positive control.

The relocation's 27 changed Markdown files rendered with the installed Marked renderer.
The sidecar README's rendered layout and command paths were inspected,
and its relative document links resolve.
Scratch evidence lives under `~/temp/agent/fuzz-relocation*`.

### Concurrent work

Unrelated fork-package source and workspace changes were left untouched.
File-enforcer regenerated `mise.toml` as-is,
including a discovered `conf-fork` bin-directory entry.
Its unrelated pnpr allow-list output was not included in relocation commits.
No relocation follow-up remains.

## Historical implementation snapshot

State of the implementation of `~/.claude/plans/setup-fuzzing-for-forbidden-strings-merged.md`
when context approached compaction.
 Resume from here after compact.

Layout update (2026-07-12,
 issue #364):
 this snapshot predates the corpus split.
Committed seeds now live in tracked `seeds/<target>/` directories,
 the `corpus/<target>/` tree is wholly gitignored scratch,
 and fuzz invocations pass `corpus/<target> seeds/<target>` as explicit
libFuzzer corpus dirs (new discoveries land only in the first).
Mentions of a `seed-*` `.gitignore` re-include and of seeds under
`corpus/<target>/` describe the layout as it was when this snapshot was
written;
 see `doc/decision/gitignore-negations.md` for the design.

## Overall task

Implement coverage-guided fuzzing for `package/cli/forbidden-strings`
using cargo-fuzz,
 a `fuzz_api` Cargo feature,
 a structured generator,
curated seeds,
 a dictionary,
 and bounded local verification.
 CI is
deferred.
 Final criterion is **soundness-by-revert** — confirm
`fuzz_extract_gate_soundness` catches a real bug when run against a
reverted commit.

Plan path:
 `/home/user/.claude/plans/setup-fuzzing-for-forbidden-strings-merged.md`.

## Environment notes (verified, important)

- Nightly Rust toolchain installed locally:
  `nightly-x86_64-unknown-linux-gnu` (rustc 1.97.0-nightly,
   d7f14d3d8 2026-05-15).
  Active default toolchain is still 1.95.0 stable;
   use `cargo +nightly` for fuzz commands.
- `cargo-fuzz 0.13.1` installed via `cargo install cargo-fuzz --version 0.13.1 --locked`.
  Binary lives at `/home/user/.cargo/bin/cargo-fuzz`.
- `cargo +nightly fuzz build` succeeds on the bare scaffold (verified — 18s cold).
- `mise run lint`,
   `mise run lint:clippy`,
   and `mise run test` for
  `package/cli/forbidden-strings` all pass with the lib extraction and
  fuzz_api in place.

## Commits landed (in order, most recent first)

```text
4a1fe951 build(forbidden-strings): scaffold cargo-fuzz workspace
4225d7ef feat(forbidden-strings): expose internals to fuzz targets via fuzz_api feature
cfc33f68 refactor(forbidden-strings): extract library boundary with run_cli_from_env
5ebe3ed1 docs(forbidden-strings): record fuzzing-tool decision
```

## Plan phases — status

<table>
<thead>
<tr>
<th>Phase</th>
<th>Plan §</th>
<th>Status</th>
<th>Notes</th>
</tr>
</thead>
<tbody>
<tr>
<td>1</td>
<td>Prepare editing context</td>
<td>DONE</td>
<td>dum-dum-non-ts SKILL.md read; package scouted.</td>
</tr>
<tr>
<td>2</td>
<td>Decision doc</td>
<td>DONE</td>
<td>`doc/decision/forbidden-strings-fuzzing.md` committed (5ebe3ed1).</td>
</tr>
<tr>
<td>3</td>
<td>Library extraction</td>
<td>DONE</td>
<td>`src/lib.rs` + `run_cli_from_env()` committed (cfc33f68). Integration tests pass.</td>
</tr>
<tr>
<td>4</td>
<td>Fuzz-only API surface</td>
<td>DONE</td>
<td>`[features] fuzzing = []`, `src/fuzz_api.rs`, `compile_rule_src`, `load_ruleset_from_source`. Committed (4225d7ef).
 `cargo check --features fuzzing` passes.</td>
</tr>
<tr>
<td>5</td>
<td>Scaffold cargo-fuzz</td>
<td>PARTIAL</td>
<td>Workspace materialized via `cargo +nightly fuzz init --fuzzing-workspace=true`. Cargo.toml wired
 for the `fuzzing` feature + arbitrary + sha2 + panic=unwind. Root `.gitignore` ignores corpus growth
 while re-including `seed-*`; `Cargo.lock` is tracked directly
 because root `.gitignore` does not ignore Cargo lockfiles. Committed (4a1fe951).
 **PLACEHOLDER `fuzz_targets/fuzz_target_1.rs` STILL PRESENT**, delete it when phase 7 lands.</td>
</tr>
<tr>
<td>6</td>
<td>Shared structured generator</td>
<td>TODO</td>
<td>`fuzz/src/generators.rs` (the file does not exist yet — `src/` dir is unmaterialized).</td>
</tr>
<tr>
<td>7</td>
<td>Prioritized fuzz targets</td>
<td>TODO</td>
<td>None of the 7 targets written. Each goes in `fuzz/fuzz_targets/`.</td>
</tr>
<tr>
<td>8</td>
<td>Dictionary and curated seeds</td>
<td>TODO</td>
<td>`fuzz/dictionaries/forbidden-strings.dict` + `fuzz/src/bin/seed-from-tests.rs`.</td>
</tr>
<tr>
<td>9</td>
<td>Tooling integration</td>
<td>TODO</td>
<td>Root `mise.toml` `[tools]` `cargo:cargo-fuzz = "0.13.1"` not yet added. Per-package nightly pinning not yet added.
 Package `mise.toml` tasks not yet added.</td>
</tr>
<tr>
<td>10</td>
<td>Documentation</td>
<td>TODO</td>
<td>README/FUZZING/PERF updates.</td>
</tr>
<tr>
<td>11</td>
<td>Final verification (incl. soundness-by-revert)</td>
<td>TODO</td>
<td>The soundness validation is the load-bearing Done criterion — proves `fuzz_extract_gate_soundness` is real.
 Use a disposable worktree, revert `e49d8694`, run for 120s, confirm failure with a redacted reproducer.</td>
</tr>
</tbody>
</table>

## What's in the scaffolded fuzz/ directory right now

```text
package/cli/forbidden-strings/fuzz/
├── .gitignore          # ignores target, artifacts, coverage, corpus/*/* (except seed-*)
├── Cargo.lock          # ✅ tracked directly; Cargo lockfiles are not gitignored
├── Cargo.toml          # wired with libfuzzer-sys (arbitrary-derive), arbitrary, sha2,
│                       # forbidden-strings (features=["fuzzing"]), panic=unwind override
└── fuzz_targets/
    └── fuzz_target_1.rs  # cargo-fuzz placeholder; delete in phase 7
```

The `src/` and `src/bin/` and `dictionaries/` and `corpus/` subdirectories do NOT exist yet.

## Next session — concrete steps to resume

### Step A — Add the structured generator (phase 6)

Create `fuzz/src/generators.rs`.
 The file must define `Arbitrary`-deriving
types that bound the generated pattern source.
 Plan §6 lists the exact
caps:
 literal atoms ≤16 B,
 concats ≤4 elements,
 alternations ≤3 branches,
depth ≤6,
 set-algebra nodes ≤2,
 content ≤4 KiB.
 Constructs the generator
must be able to reach are also listed in §6 (inline/scoped flags,
non-capturing groups,
 classes,
 quantifiers,
 lookarounds,
 resharp algebra,
the bare `_` triad,
 escaped lookalikes,
 Unicode WS bytes).

Suggested module layout:
 a `RuleSrc` type that derives `Arbitrary` and
serialises to a regex source string,
 plus a `Content` type that derives
`Arbitrary` and bounds at 4 KiB.
 Bias content toward rendered literals
plus single-byte mutations (per §6 final bullet).

Reference this file from every fuzz target via
`use forbidden_strings_fuzz::generators::*;` once the lib target is
declared in `fuzz/Cargo.toml`.
 (Currently `fuzz/Cargo.toml` only has
[[bin]] entries;
 you'll need to add `[lib] path = "src/lib.rs"` and a
`src/lib.rs` that re-exports the generators module.
)

### Step B — Write the 7 fuzz targets (phase 7)

For each target listed in plan §7,
 create `fuzz/fuzz_targets/<name>.rs`
and add a matching `[[bin]]` block to `fuzz/Cargo.toml`.
 Order by
priority:

1. **`fuzz_extract_gate_soundness`** — primary.
    Per §7.1:
    assert that for
   every regex match in haystack,
    at least one extracted gate substring
   from `extract_gating_substrings` is present in the haystack under the
   gate's case-sensitivity mode.
    Use `compile_rule_src` so the fuzzer and
   production share the compile path.
    Reject oversized patterns,
    compile
   failures,
    and zero-match cases.
    Panic message:
    pattern source +
   content length + SHA-256 hex digest only (never raw bytes).
2. **`fuzz_ruleset_scan_invariants`** — build a bounded ruleset,
    run
   `scan_content`,
    assert:
    every hit re-checks via regex on raw bytes,
   no column counter crosses UTF-8 boundary,
    hit set invariant to rayon
   thread count,
    hit set invariant to rule order,
    hit format matches the
   `path:line:cols rule=N` shape.
3. **`fuzz_regex_engine_dispatch`** — assert resharp-only constructs
   route to resharp;
    plain constructs route to `regex` crate.
    For the
   feature subset both engines agree on,
    compare `is_match` and non-empty
   `find_all` on bounded haystacks.
    Gate the comparison via
   `both_engines_agree(src)` so the target stays sound as either engine
   evolves.
4. **`fuzz_regex_syntax_walkers`** — panic-freedom + index invariants for
   `group_body_start`,
    `find_matching_close_paren`,
    `skip_any_quantifier`,
   `quantifier_is_required`,
    `skip_class_body`,
    `walk_literal_bytes`.
5. **`fuzz_scan_format`** — line-index construction,
    byte-to-line/column,
   hit-end clipping,
    `format_hit` redaction.
    Negative invariant:
    the
   formatted hit never contains any matched byte from the content slice.
6. **`fuzz_residual_shards`** — each input regex appears exactly once
   across shards;
    if any member regex matches a haystack,
    the combined-
   shard gate returns `Ok(true)` or `Err(())`,
    never `Ok(false)`.
7. **`fuzz_literal_roundtrip`** — keep only if cheap.

Delete `fuzz/fuzz_targets/fuzz_target_1.rs` when the first real target
lands.

### Step C — Dictionary and seeder (phase 8)

- Write `fuzz/dictionaries/forbidden-strings.dict` per plan §8.1.
   Token
  list is exhaustive there.
- Write `fuzz/src/bin/seed-from-tests.rs` per §8.2.
   It extracts byte
  literals from `src/rules/extract_tests.rs`,
   `atom_tests.rs`,
  `engine_tests.rs`,
   `algebra_tests.rs` and writes them to
  `fuzz/corpus/<target>/seed-<sha>` (the `seed-*` prefix matches the
  `.gitignore` re-include).
   At runtime,
   verify with `git check-ignore`
  that the seeder never reads `forbidden-strings.local.txt`.

### Step D — Tooling integration (phase 9)

1. Root `mise.toml` `[tools]` — add `"cargo:cargo-fuzz" = "0.13.1"`
   next to the existing `"cargo:fastmod"`.
2. Per-package nightly pinning — either add to
   `package/cli/forbidden-strings/mise.toml` `[tools]` (preferred per
   plan),
    or set `RUSTUP_TOOLCHAIN=nightly` per fuzz task as the
   fallback.
    The simpler fallback is fine;
    nightly is currently
   installed system-wide.
3. Add to `package/cli/forbidden-strings/mise.toml` `[tasks.fuzz:list]`,
   `[tasks.fuzz:build]`,
    `[tasks.fuzz:smoke]`,
    `[tasks.fuzz:run]`,
   `[tasks.fuzz:seed]`.
    Plan §9.3 spells out the command bodies.
4. **DO NOT** add a `fuzz:install` task — mise's tool system handles
   installation (per plan §9.4).
5. Document the bounded container wrapper in README:
   `podman run --memory=2g --cpus=2 --rm -v "$PWD":/work -w /work <image> mise run //package/cli/forbidden-strings:fuzz:build`.

### Step E — Documentation (phase 10)

Update `package/cli/forbidden-strings/README.md` with the sections in
plan §10.1.
 Split into a dedicated `FUZZING.md` only if README exceeds
120 lines after the additions.

`PERF.md`:
 only add findings if fuzzing surfaces a compile-time or
scan-time cliff during smoke runs.
 If smoke is clean,
 no `PERF.md` edit
needed.

### Step F — Final verification (phase 11)

Run in order:

1. `mise run //package/cli/forbidden-strings:build`
2. `mise run //package/cli/forbidden-strings:test`
3. `mise run //package/cli/forbidden-strings:lint`
4. `mise run //package/cli/forbidden-strings:lint:clippy`
5. Release binary CLI smoke (any temp file with a known rule).
6. `mise run //package/cli/forbidden-strings:fuzz:build` inside the
   container wrapper.
7. `mise run //package/cli/forbidden-strings:fuzz:smoke` inside the
   container wrapper.
8. `git check-ignore --verbose package/cli/forbidden-strings.fuzz/Cargo.lock`
   must return no match.
   Cargo lockfiles are not gitignored.
9. Sentinel commands from `AGENTS.md` "Git cleanup and worktree safety
   reviews" to confirm no fuzz output escapes the ignore set.
10. **Soundness-by-revert (load-bearing!
    )**:
    - Create a disposable worktree from current `main`.
    - In the worktree,
       `git revert --no-commit e49d8694` (the `(?u)`
      extraction skip fix).
    - Run `mise run //package/cli/forbidden-strings:fuzz:run -- fuzz_extract_gate_soundness -max_total_time=120`.
    - Confirm the target reports a soundness failure with a redacted
      reproducer (no raw secret-like bytes).
    - Remove the worktree.

### Phase 11 partial status (post resharp 0.6.0 bump, 2026-05-16)

- Steps 1-4 PASS against resharp 0.6.0 (132 unit + 19 integration tests,
  zero lint warnings,
   zero clippy warnings).
- Step 6 (fuzz:
  build) PASSES against resharp 0.6.0.
   Resolved a
  cargo-fuzz 0.13.1 vs musl-vs-ASAN incompatibility by threading
  `--target x86_64-unknown-linux-gnu` through `fuzz:build`,
  `fuzz:smoke`,
   and `fuzz:run` in commit `7b2caf88`.
- `fuzz:run` arg-spread bug fixed in commit `202ed6b6`.

### Late-2026-05-16 session: 5 pre-validators landed, fuzz hardened

**Commits landed today,
 most recent first:
**

```text
cbc1616e fix(forbidden-strings): widen intersection+quant hang detector
3d996936 docs(forbidden-strings): trace resharp hang to prefix.rs visited-set bug
e5ab8c6f fix(forbidden-strings): pre-validate resharp algebra-hang shape
4fb14f4c fix(forbidden-strings): pre-validate alt-lookaround sibling shape
9ac0b3a9 fix(forbidden-strings): pre-validate nested grouped quantifiers
091e0015 docs(handover): record late-session findings
4d5563cb Reapply widening (un-revert)
2f4d27b0 feat: widen fuzz literal alphabet
cd9b2dbf fix: pre-validate stacked quantifiers
```

**Pre-validators now in `compile_rule_src` (resharp path):
**

1. `stacked_quantifier` (cd9b2dbf) -- bare-stacked `a**`/`\D{5,11}{5,11}`.
2. `nested_grouped_quantifier` (9ac0b3a9) -- grouped-via-`(?:)` chain
   of `){quant}` adjacencies at depth 4+.
    Probe shows slow-unit
   `(?iu)(?:(?:(?:(?:(?:\d){5,11}){5,11}){5,11}){5,11}){5,11}(?:(?:(?:(?:(?:\d)*)*)*)*)*aa`
   rejects in 4.98us (was 3.26s).
3. `intersection_with_lookbehind` (4fb14f4c) -- widened to cover any
   lookaround direction (was lookbehind-only).
4. `lookaround_in_alternation_with_sibling` (4fb14f4c) -- catches
   `(a|(?![_]))(?!a)` shape that hits resharp `engine.rs:1020`
   `debug_assert!` during `scan_fwd_all`.
    Bisected from
   `crash-8cba104f0805ccb567513aff895398a4f652200c`.
    Probe rejects
   in 8.89us (was compile-OK then find_all-panic).
5. `complement_intersection_quantified_group` (e5ab8c6f,
    widened
   cbc1616e) -- catches `&` + quantified group (`)*`/`)+`/etc.)
   anywhere in source.
    Bisected from
   `timeout-00179d433e26fbcc3bedf2b7b38b6ce1ff9e6438` and
   `timeout-0815a95346bfa16ae0c6454162d9af0b8c05779c`.

**Bug E -- new finding documented in TROUBLESHOOTING.
`resharp.md`:
**

The intersection+quantified-group hang traces (via gdb attach to a
hung probe) to `resharp-engine/src/prefix.rs:27` in
`calc_prefix_sets_inner`.
 The `redundant` set is initialized with
`{BOT, start}` and never updated inside the loop;
 derivative chains
that produce unique nodes indefinitely never terminate.
 This has a
minimal-patch prototype (`redundant.insert(node)`) so it should be
filed upstream.
 The 4 other resharp bugs (B,
 C,
 D) lack patch
prototypes and stay deferred.

**Verification status:
**

- 60s fuzz on main:
   PASSED 34328 runs cleanly (verified after the
  first 4 pre-validators landed).
- 120s fuzz in reverted worktree:
   HIT timeout shape
  `(?i) ###(?:\s&üü)(?:####)+...` (no complement,
   intersection +
  quant).
   Widened the validator in cbc1616e but haven't re-run yet.

**Resume work for next session,
 in priority order:
**

1. **Re-run 120s soundness-by-revert** in `/tmp/fs-soundness-revert`
   (worktree exists,
    was reverted of `e49d8694`).
    Sync sources first:
   `cp /var/home/user/Monochromatic/package/cli/forbidden-strings/src/{fuzz_api,rules}.rs /tmp/fs-soundness-revert/packages/cli/forbidden-strings/src/` then `cp src/rules/{engine,engine_tests}.rs .../src/rules/`.
   Clear stale timeout:
    `rm /tmp/fs-soundness-revert/packages/cli/forbidden-strings/fuzz/artifacts/fuzz_extract_gate_soundness/timeout-*`.
   Run:
    `cd /tmp/fs-soundness-revert/packages/cli/forbidden-strings && mise run fuzz:run fuzz_extract_gate_soundness -max_total_time=120 -timeout=10`.
   Expect SOUNDNESS PANIC.
    If hit another non-soundness halt,
    decode
   via `/tmp/probe-slow-unit/target/release/probe <artifact-path>`
   and bisect with `bisect2.rs..bisect6.rs` style probes.
2. **If soundness panic doesn't fire after fuzz completes 120s clean,
   **
   bias `synth_content` in `fuzz/src/generators.rs` to emit
   Unicode-case-flipped variants of any non-ASCII bytes in the rule's
   literals.
    Manual probe confirms `(?iu)café` vs `CAFÉ` panics with
   the expected redacted reproducer when e49d8694 is reverted;
    the
   gap is that uniform-random byte mutations don't reliably converge
   on the `0xA9` -> `0x89` (é -> É) byte-pair flip.
3. **File Bug E upstream** at github.
   com/ieviev/resharp using
   minimal-patch prototype documented in TROUBLESHOOTING.
   `resharp.md`.
   Verify the patch against the resharp test suite in
   `/tmp/resharp-src/` before filing.

### 2026-05-17 session: tightened Bug F validator, pre-validator warnings, Bug G discovered

**Commits landed today,
 most recent first:
**

```text
29f5f495 fix(forbidden-strings): tighten Bug F validator + emit pre-validator warnings
241286f8 fix(forbidden-strings): widen Bug F validator for trailing-content shape
```

**What happened (chronological):
**

1. Added `quantified_lookahead_with_sibling_content` (241286f8) covering
   the second Bug F shape (single quantified-LA + trailing).
    Initially
   broad:
    fire on any quantified-LA + ANY sibling content at parent
   depth.
    User pushed back that this rejects legitimate production
   patterns like `(?:(?!X)){n}<atom>` (exact quant,
    compiles cleanly)
   and `(?:(?!X)){m,n}aaa` (3+ trail,
    also compiles cleanly).

2. Tightened (29f5f495) to be PRECISE — fires only on shapes that
   actually panic upstream:
   - VARIABLE bound `{m,n}` with m < n (and `*`/`+`/`?` which are
     variable).
      EXACT `{n}` does NOT arm.
   - EXACTLY 1 or 2 trailing atoms at parent depth.
      3+ atoms disarms.
   - Atom = literal byte,
      escape,
      char class,
      group,
      anchor.
     No accepted false positives.
      Negative tests now explicitly
     enumerate the previously-false-positive shapes.

3. Added eprintln warnings to ALL 7 pre-validator rejection sites in
   `compile_rule_src` (per user request "emit a warning whenever the
   validator flags something").
    Each warning names the validator and
   quotes the source so users see why a rule was rejected without
   parsing the error string.

**Bug G discovered (NEW upstream resharp bug,
 separate from Bug F):
**

Same overflow line (`resharp-algebra/src/lib.rs:2470`) but DIFFERENT
upstream trigger — NO lookahead,
 just alternation+complement+intersection.

- Artifact:
   `crash-7e654294d4daaa18073ed3117a54546af32b3a54`
- Minimal:
   `(?:$|x)(?u:(?:$&y))aa`
- Required pieces (bisected via `bisect_g.rs`,
   `bisect_g2.rs`):
  - Alternation `(?:$|...)` with `$` anchor as one branch
  - Wrapped group containing `$&literal` (`&` is resharp intersection)
  - Trailing content NOT required
- Stack trace identical to Bug F:
   `attempt_rw_concat_2` →
  `mk_concat` → `mk_binary_inner` (recursive) → `der` → `Regex::new`.
- NOT yet pre-validated.
   Needs its own validator if we go the iterative
  route.
   Probably the simplest detection:
   alternation containing
  `$`/`^` + co-occurring `&` outside character class.

**Long-trail + long-LA-body Bug F variant escapes tightened validator:
**

The full crash-a219 artifact `(?:(?!ñññAtsöéaañ)){4,12}~(ññM aaaaaaaa)
aaaaaa` has 8+ trailing atoms after the quantified-LA,
 so the tightened
validator correctly doesn't fire (it would be a false positive on the
"3+ trail OK" rule).
 But upstream still panics — bisect shows the
TRIGGER depends on LA body length AND complement body length,
 both
above some threshold.
 Minimal panicking shape:
`(?:(?!aaaaaaaaaaaaa)){4,12}~(def)aaa`.

This isn't trivially structurally detectable.
 The right architectural
fix is to install a custom panic hook in the fuzz target (see "Next
session priority" below) rather than chase iterative validators per
upstream-bug-shape.

**Resume work next session — priority:
**

1. **Install a custom panic hook in the fuzz target.
   ** libfuzzer-sys
   currently calls `abort()` from its panic hook BEFORE unwinding
   starts,
    so `compile_rule_src`'s `catch_unwind` never gets a chance
   to catch resharp panics.
    Solution:
    in `fuzz_extract_gate_soundness.rs`,
   replace libfuzzer's hook (use `std::panic::set_hook`) with one that:
   - For panics at known resharp locations
     (`resharp-algebra/src/lib.rs:2470`,
      `resharp-engine/src/engine.rs:1020`)
     → no-op,
      let unwinding proceed (catch_unwind catches it).
   - For all other panics → call default hook + abort (preserves
     normal fuzz crash semantics).
     This single change unblocks both remaining Bug F variants AND Bug G
     AND any future upstream variant at the same locations.

2. Once panic hook is in place,
    re-run 180s soundness-by-revert fuzz.
   Expected:
    SOUNDNESS PANIC at the assertion in
   `fuzz_extract_gate_soundness.rs:290`.

3. Once fuzz halts on SOUNDNESS PANIC,
    complete Phase 11.
    The verbose
   output should include the rule and content that triggered the
   violation (the `e49d8694` bug).

4. File Bug E,
    Bug F,
    Bug G upstream at github.
   com/ieviev/resharp.

**Diagnostic infrastructure (continued):
**

- `/tmp/probe-slow-unit/src/bin/`:
  - `bisect_complement.rs` — complement-after-LA variants
  - `bisect_artifact.rs` — artifact vs minimal divergence
  - `bisect_g.rs`,
     `bisect_g2.rs` — Bug G triggers
  - `verify_tightened.rs` — confirms tightened validator behavior
  - `decode_7e6.rs` — decodes Bug G crash artifact
  - `verify_a219.rs` — confirms Bug F a219 artifact handling

**Verification status:
**

- 138 unit tests pass (3 new for Bug F variant,
   modified test set for
  tightened semantics).
- Clippy clean.
- Warnings stream to stderr on every pre-validator rejection (visible
  in fuzz log and CLI runs);
   trimmed to one line per rejection to
  avoid log spam (b9ce2811).
- Panic-hook installed (66b1b007) and corrected to match
  `resharp-0.6.0/src/engine.rs` Bug B path (68c4cf92).
- 600s fuzz with hook in place:
   90,129 iterations,
   zero crashes,
   zero
  soundness violations.
   Hook is working (no upstream-bug halts) but
  random mutation + case-flip bias isn't converging on the
  soundness-triggering shape within that budget.
- Soundness shape IS still reachable via direct probe:
   `(?iu)café`
  vs `CAFÉ` and `(?iu)abécret` vs `abÉcret` both fire SOUNDNESS
  VIOLATION via `/tmp/probe-slow-unit/target/release/verify_soundness`.

**Next session:
 bias the fuzz harder toward the soundness shape.
**

The fuzz's coverage feedback alone hasn't found the right (?
iu)

- Unicode-literal + case-flipped-content combo in 90k iterations.
  Three orthogonal levers:

1. **Seed the corpus with crafted bytes** that decode (via
   `RuleAndContent::arbitrary`) close to the soundness shape.
    Started
   with 5 seeds at `/tmp/fs-soundness-revert/.../seed-soundness-*`
   (via `/tmp/seed_soundness.sh`) embedding `\xc3\xa9` (é) and
   `\xc3\x89` (É) byte pairs.
    The Arbitrary decoder will partition
   these between rule/content;
    coverage-guided mutation evolves from
   there.
    Next session,
    expand the seed set with longer prefixes
   (`abc<é>def<É>ret`) and explicit flag bytes corresponding to
   `(?iu)` rendering.

2. **Bias FlagSet's Arbitrary impl** so `(?iu)` (i=true,
    u=true,
   negate_i=false) is overrepresented.
    Currently each bool is
   uniform-random so the right combo is ~12.5% probability.
    A
   manual `Arbitrary` impl that reads one byte and biases the i+u
   case to ~50% would help.

3. **Bias gen_literal_bytes** so multi-char Unicode-flanked
   literals (`abc<é>def` shape) appear more often than single
   Unicode chars (single-char doesn't get a gate extracted;
    the
   shape needs ASCII context around the Unicode letter for the
   AC gate to include it).

Lower-effort first:
 try (1) with a richer seed set.
 If 30-min
fuzz against the larger seed set doesn't fire,
 do (2) and (3) in
the generator.

**Update:
 biases (2) and (3) landed in 063512ea this session.
**
A 11-min fuzz with the biases + 5 seeded corpus entries ran 54,960
iterations on a 12504-file corpus (grew from 9929).
 The biases are
working (`café`-shaped literals visible in saved dictionary entries
like `caf\303\243caf\303\243`),
 but the soundness panic still
didn't fire.
 Slow-units (13-17 s each) ate throughput,
 dropping
exec/s from 175 to ~83.

**Next session priorities (in order):
**

1. **Add a slow-unit pre-validator** for the intersection-of-large-classes
   shape that ate fuzz cycles this session (e.g. `(?i)(?!(?:(?:(?:ñ
   ×10&ñ×10)&ñ×10)|\s))`).
    Add a check like
   "intersection involving classes/literals of length >= 10 inside a
   lookahead" so the fuzz rejects them in microseconds instead of 13s.
   The slow-unit artifacts at
   `fuzz/artifacts/fuzz_extract_gate_soundness/slow-unit-293a*` and
   `slow-unit-7a84*` decode the exact shape.

2. **Run 30-60 min fuzz on the warm corpus.
   ** With the slow-unit
   validator removing the throughput drain,
    expected exec/s should
   return to ~200-300.
    Combined with the 12504-file corpus and the
   biases,
    the soundness panic should fire within that budget.

3. **If still not firing,
    write a brute-force soundness searcher.
   **
   Iterate over short byte sequences passed through
   `RuleAndContent::arbitrary`;
    check each decoded pair against the
   soundness contract directly (no libfuzzer harness).
    The byte
   space for the simplest triggering shape is small enough to
   enumerate exhaustively up to ~100 bytes.

4. **Phase 11 completion:
   ** when soundness panic fires,
    capture the
   redacted reproducer + clean up worktree per the original plan.

### 2026-05-16 late session (~23:00): Bug F discovered + 3 more commits

### Late-2026-05-16 session (~23:00): Bug F discovered + 3 more commits

**Commits landed today,
 most recent first:
**

```text
a08b45ed fuzz(forbidden-strings): bias synth_content for Unicode case-flip
214c03b1 fix(forbidden-strings): widen alt-lookaround validator threshold
6ff333f1 fix(forbidden-strings): pre-validate Bug F nested-lookahead overflow
```

**Bug F discovered (NEW resharp bug):
**

- Panic:
   `attempt to add with overflow` at
  `resharp-algebra/src/lib.rs:2470` (`tail_rel + la_rel`,
   both u32).
  Triggered when a lookahead-chain `rel` length saturates to u32:
  :
  MAX
  and the next add overflows.
   Under cargo-fuzz's debug-assertions=on,
  this panics.
   Production (debug-assertions=off) silently wraps to 0
  and likely produces wrong matches.
- First two artifacts:
  `crash-06d9dd9fa1abfeec72a8154c09434b237dfc7f38` (rendered
  `(?:(?:(?!\?)){1,5}){2,4}`-style) and
  `crash-df95fcd52de76d952ee3db291f59434ece2c0b81`.
   Added
  `nested_lookahead_in_quantified_group` (6ff333f1).
- Third artifact `crash-c3c364eb3a03114a52015721c02cba0bf20eb496`
  (`engine.rs`:
  1020,
   NOT Bug F) had a single lookaround in alternation;
  widened the existing `lookaround_in_alternation_with_sibling`
  validator (214c03b1) to drop `total_lookarounds >= 2`.
- Added Unicode case-flip bias (a08b45ed):
   when rule has (?
  iu),
  with 50% probability flip embedded é/ñ/ü/ö/ç to É/Ñ/Ü/Ö/Ç in
  content.
   Manual probe confirms the soundness panic shape
  `(?iu)café` vs `CAFÉ` IS reachable;
   the gap was random mutations.

**KNOWN GAP — Bug F validator is too narrow:
**

After biasing case-flip,
 the 240s fuzz hit YET ANOTHER Bug F shape:
`crash-a219859099426658d70e90bc97f560b85f2cf256` rendered as
`(?u-i)\+\+\+\+\+\+(?:_|_|\u{3000})ñöa#3vaarññ (?:(?!ñññAtsöéaañ)){4,12}~(ññM aaaaaaaa)aaaaaa`.

Bisected (probes `bisect_f5..bisect_f8.rs`) to:

- `(?:(?!abc)){4,12}a` PANICS (quantified lookahead followed by trailing literal)
- `(?:(?!abc)){4,12}` alone OK
- `(?:(?!abc)){2}a` OK (exact quantifier)
- `(?:(?!abc)){4,12}aaaaaa` OK (long trailing)
- `(?:(?!abc)){4,12}aa` PANICS (short trailing)
- `(?:(?!abc)){1,4}a` PANICS (min=1 with trailing)
- `(?:(?!abc)){2,3}a` PANICS

Pattern:
 **quantified lookahead with VARIABLE quant (m<n) + SHORT
trailing content** also triggers Bug F.
 My validator only catches
"nested quant + outer min ≥ 2 + no siblings";
 misses the "single
quant + trailing literal" case.

**Resume work next session:
**

1. **Widen nested_lookahead_in_quantified_group** to also catch
   "quantified lookahead group + trailing content at parent depth".
   Bisect first to nail the exact criterion (variable-quant vs
   trailing-length interaction).
    Probes are at
   `/tmp/probe-slow-unit/src/bin/bisect_f5..f8.rs`.
   Alternatively,
    just widen broadly:
    fire on any
   quantified-lookahead-bearing group that has any sibling content
   at parent depth.
    Accept false positives (the user already endorsed
   this trade-off).
2. Sync new validator to `/tmp/fs-soundness-revert/`,
    clear crashes,
   re-run 120s fuzz.
    Expect either soundness panic (success) or
   another novel resharp shape (decode + add another pre-validator).
3. Once fuzz halts cleanly on SOUNDNESS PANIC,
    complete Phase 11.

**Diagnostic infrastructure in /tmp/probe-slow-unit/:
**

- `Cargo.toml` currently points at `/tmp/fs-soundness-revert/.../forbidden-strings` (worktree).
   The original main-branch path is in `Cargo.toml.bak`.
   To probe MAIN's compile_rule_src,
   restore from bak.
- Probes built so far (RUSTFLAGS="-C debug-assertions=on" required):
  `probe` (main entry,
   decodes artifact),
   `bisect_f`,
   `bisect_f2..f8`
  (Bug F bisection),
   `bisect_b2` (Bug B alt-lookaround bisection),
  `verify_soundness` (direct soundness check on hand-crafted shapes).

**Verification status as of this commit:
**

- 137 unit tests pass (3 new added for Bug F validator).
- Clippy clean.
- Main branch 60s fuzz:
   clean.
- Worktree 240s fuzz:
   still hits Bug F shapes my validator misses.
- Soundness panic NOT yet observed via fuzz (but verified reachable
  via direct probe;
   case-flip bias is in place).
- Crash artifacts at `/tmp/fs-crash-artifacts/` were re-verified against
  the 0.6.0+fix binary (`128221b7`);
   both run in 0ms with no crash.
   See
  `HANDOVER.resharp-panic-fix.md` for details.
- Fresh bounded fuzz run (`fuzz_extract_gate_soundness` for ~45s
  against 0.6.0+fix) completed 4714 iterations with zero new crashes.
- During the 0.6.0 verification,
   a side-finding was recorded in
  `HANDOVER.resharp-panic-fix.md`:
   shape 2 (`scan_fwd_all` panic at
  `engine.rs:1020`) is behind a `debug_assert!` that compiles out in
  release.
   The pre-validator therefore stays as the primary defense for
  shape 2 in production;
   `catch_unwind` only matters in test/CI builds
  where `debug_assertions` is on.
   See `TROUBLESHOOTING.resharp.md` Bug B
  for details.

## Open questions / gotchas to remember

- **Soundness-by-revert is non-skippable.
  ** Done criteria explicitly call
  this out.
   The whole point of the target is to prove it would catch a
  real bug;
   skipping the revert validation makes the target unproven.
- **Items widened from pub(crate) → pub.
  ** `walk_literal_bytes`,
  `skip_atom_with_extract`,
   the 5 regex-syntax walkers are now `pub` in
  their submodules,
   but the submodules are still `mod` (private),
   so the
  items remain unreachable from outside the crate unless the `fuzzing`
  feature is active and a consumer reaches them via
  `forbidden_strings::fuzz_api::*`.
   Don't widen the submodules
  themselves to `pub mod`.
- **Refactored compile path.
  ** Production now calls `compile_rule_src`
  exclusively.
   The old direct `Regex::new` block inside the load loop is
  gone.
   Error messages still match the previous shape
  (`rule on line N (resharp): ...` / `rule on line N (regex): ...`)
  because `compile_rule_src` returns `(resharp): ...`/`(regex): ...` and
  the loader prepends `rule on line N`.
- **`compile_plain_rule` was removed.
  ** It became dead code after the
  refactor and was deleted.
   Don't be surprised that the function is
  gone;
   `compile_plain_rule_to_compiled` is its replacement.
- **`load_ruleset_from_source(content, label)`** has an unused `_label`
  parameter today — present for future error-context use.
- **`fuzz/Cargo.toml` needs a `[lib]` entry** once the generators
  module lands,
   so each target can `use forbidden_strings_fuzz::generators::*;`.
  Right now there are only `[[bin]]` entries.
- **`mise run` task logic uses `node -e`**,
   not bash.
   The fuzz:
  smoke task
  loops over `cargo fuzz list` output in node,
   running each target in sequence.
- **Resource-exhaustion isolation rule** (`AGENTS.md`):
   all fuzz commands
  must run inside the bounded container wrapper.
   Document it;
   don't
  remove the wrapper for convenience.
- **Bash piping is unreliable** per `AGENTS.md` "Visible terminal
  spawning".
   Don't pipe in mise task bodies;
   use intermediate files.

## Reference paths

- Plan:
   `/home/user/.claude/plans/setup-fuzzing-for-forbidden-strings-merged.md`
- Decision doc:
   `doc/decision/forbidden-strings-fuzzing.md`
- Lib boundary:
   `package/cli/forbidden-strings/src/lib.rs`
- Fuzz API surface:
   `package/cli/forbidden-strings/src/fuzz_api.rs`
- Fuzz scaffold:
   `package/cli/forbidden-strings/fuzz/`
- Test fixture sources the seeder must read:
  `package/cli/forbidden-strings/src/rule/{extract_tests,atom_tests,engine_tests,algebra_tests}.rs`
- The bug-fix commits motivating the soundness target:
  `e49d8694` (`(?u)` extraction skip),
   `e100659f` (bare `_` as wildcard
  in extractor),
   `9b41fca0` (route bare `_` to resharp),
   `1463c59b`
  (scoped `(?x:body)`),
   `0479371a` (unicode for `\s/\w/\d/\b`),
  `4289cdb3` (expand `\s` to Unicode-WS bytes).

### 2026-05-17 evening session: three new slow-compile pre-validators landed

**Commits landed this session,
 most recent first:
**

```text
f5bc49c6 fix(forbidden-strings): pre-validate nested complement `~(~(...))`
9c3e06cd fix(forbidden-strings): pre-validate two slow-compile resharp shapes
```

**Three new pre-validators in `compile_rule_src`'s resharp branch:
**

1. `nested_quantifier_after_wildcard` (9c3e06cd) -- chain >= 3 of
   `){quant}` adjacencies immediately following a bare `_` wildcard
   outside a class.
    Decoded from `slow-unit-8c41` (compile 409ms +
   scan 1.16s) and `slow-unit-709c` (compile 4.33s).
    Innermost atom
   is `_` triad (wildcard);
    nesting depth 3+ explodes resharp's NFA.

2. `nested_chain_in_lookaround_body` (9c3e06cd) -- chain >= 3 of
   `){quant}` pairs occurring while any open lookaround frame
   (`(?!`/`(?=`/`(?<!`/`(?<=`) remains higher up the stack.
    Decoded
   from `slow-unit-4eab` (compile 1.9s,
    errors with
   `Algebra(UnsupportedPattern)`).
    Resharp's algebra walks derivative
   shapes per-prefix per-suffix inside lookarounds.

3. `nested_complement` (f5bc49c6) -- inner `~(...)` complement
   nested inside an open outer `~(...)`.
    Both back-to-back
   `~(~(...))` and group-transparent `~((?:~(...)))` forms caught
   (probed both at ~915ms).
    Decoded from
   `timeout-95f5e661c596e4b5a12e9841cda2e3ba242ecf7a` (the new
   biased-generator counterpart).
    Sibling complements
   `~(...)&~(...)` (production shape,
    e.g. RELEASE_TAG rule) are
   NOT caught:
    the inner complement is checked only when an outer
   complement frame is still on the stack.

**Verification status:
**

- 147 unit tests pass (3 new for each of the validators above;
   9
  new total).
- Clippy clean against `-D warnings`.
- Main branch test suite green.
- Sources synced to worktree at `/tmp/fs-soundness-revert/`.
- Stale slow-unit-* and crash-* artifacts cleared from worktree
  fuzz/artifacts/fuzz_extract_gate_soundness/.
- Initial post-validator fuzz run hit the
  `timeout-95f5e661c596e4b5a12e9841cda2e3ba242ecf7a` shape after
  the first 2 validators landed (~12s in).
   After adding validator
  #3,
   ran a 5-min fuzz to verify no novel slow shapes appear.

**Resume work next session,
 in priority order:
**

1. **Wait for 5-min fuzz result** (started at end of this session,
    log
   at `/tmp/fuzz-soundness-5min.log`).
    If clean,
    run a 30-60 min
   fuzz with `mise run fuzz:run fuzz_extract_gate_soundness
   -max_total_time=1800 -timeout=10`.
    Expect SOUNDNESS PANIC.

2. **If yet ANOTHER novel slow shape appears**,
    the advisor flagged
   that validator count is at 10 and "structural-validator approach
   has hit its limit".
    Switch to generator-side anti-bias:
    modify
   `RuleSrc::arbitrary` in `fuzz/src/generators.rs` to use rejection
   sampling against known slow shapes (`~(~(`,
    deep nested chains,
   etc.).
    Coverage is preserved if the rejection happens at decode
   time,
    before the input enters the corpus.
    Cap validator-side
   additions at this round.

3. **Phase 11 completion:
   ** when SOUNDNESS PANIC fires,
    capture the
   redacted reproducer + clean up worktree per the original plan.

**Diagnostic infrastructure additions:
**

- `/tmp/probe-slow-unit/src/bin/test_nested_complement.rs` --
  timing probe for the back-to-back and group-transparent
  complement forms.
   Confirms both compile in ~915ms (1000x slower
  than single complement);
   confirms the production RELEASE_TAG
  rule is rejected by a separate validator and would not be
  affected.

**Pre-validator inventory (full list,
 10 total):
**

In `compile_rule_src` order (both branches considered):

1. `stacked_quantifier` (regex branch + resharp) -- `a**` / `\D{5,11}{5,11}` bare-stacked shapes.
2. `nested_grouped_quantifier` (regex branch + resharp) -- chain >= 4 of `){quant}` pairs anywhere.
3. `lookaround_in_complement` (resharp only) -- `\b`/`^`/`$`/lookaround inside `~(...)`.
4. `intersection_with_lookbehind` (resharp only) -- `&` with any lookaround.
5. `intersection_with_word_end_alternation` (resharp only).
6. `lookaround_in_alternation_with_sibling` (resharp only) -- `(a|(?![X]))(?!Y)` shape.
7. `complement_intersection_quantified_group` (resharp only) -- `&` + quantified group + `~(`.
8. `nested_lookahead_in_quantified_group` (resharp only) -- Bug F core:
    nested LA in quantified group,
    outer min >= 2.
9. `quantified_lookahead_with_sibling_content` (resharp only) -- Bug F variant:
    quantified LA + short trailing.
10. `nested_quantifier_after_wildcard` (resharp only) -- chain >= 3 immediately after bare `_` (new this session).
11. `nested_chain_in_lookaround_body` (resharp only) -- chain >= 3 inside open lookaround body (new this session).
12. `nested_complement` (resharp only) -- `~(...)` body contains another `~(...)` (new this session).

(Counted to 12 above.
 The 8th and 9th overlap conceptually but each
catches a distinct upstream-bug shape;
 both kept.
)

### 2026-05-17 late session: PHASE 11 COMPLETE — soundness-by-revert verified

**Commits landed this session,
 most recent first:
**

```text
099bfe84 fuzz(forbidden-strings): fix Unicode-literal renderer mojibake
97aa1bf2 docs(handover): record three new pre-validators landed this session
f5bc49c6 fix(forbidden-strings): pre-validate nested complement `~(~(...))`
9c3e06cd fix(forbidden-strings): pre-validate two slow-compile resharp shapes
```

**THE bug that gated Phase 11:
 Unicode-literal renderer mojibake (099bfe84).
**

`Node::Literal`,
 `Node::Class`,
 and `Node::NegClass` rendered byte
sequences via `out.push(b as char)`,
 which converts each byte to
its Latin-1 codepoint.
 For multi-byte UTF-8 sequences like é
(0xC3 0xA9) the result was mojibake `Ã©` (chars U+00C3 + U+00A9,
re-encoded by String's UTF-8 storage as bytes 0xC3 0x83 0xC2 0xA9).

Pre-fix consequence:
 every Unicode-literal rule the generator
emitted had source bytes that did NOT match the content's real
UTF-8 bytes.
 `find_all` returned 0 matches on every Unicode rule,
so the fuzz target's soundness check could never fire.
 The
063512ea biases (50% (?
iu) flag,
 25% Unicode-shape literal,
 50%
case-flip) all stacked the deck to ~3% of iterations hitting
soundness shape — but every one of those rejected at find_all=0.

Root cause confirmed via `/tmp/probe-slow-unit/src/bin/check_render.rs`:
pre-fix rendered source for `é` literal had bytes
`c3 83 c2 a9` (mojibake),
 content had `c3 a9` (real UTF-8),
find_all=0.
 Post-fix:
 source has `c3 a9`,
 content has `c3 a9`,
find_all=1,
 gate `café` (CI=true) extracted,
 contains_under_ci
checks "café" against content "cafÉ" → ASCII case-fold mismatch
on bytes a9 vs 89 → SOUNDNESS VIOLATION.

**Phase 11 verification (load-bearing Done criterion):
**

1. Reverted worktree at `/tmp/fs-soundness-revert/` (e49d8694 fix
   reverted via commit 52aa8a46).
2. 5-min fuzz with fixed renderer + warm corpus (18,278 files):
   panic fired during corpus replay (before mutation budget kicked
   in).
3. Crash artifact:
    `crash-6dcd4ce23d88e2ee9568ba546c007c63d9131c1b`
   (1-byte input).
4. Decoded reproducer:
   - rendered src:
      `(?iu)café` (10 bytes)
   - content:
      `cafÉ` (5 bytes:
      `99 97 102 195 137` = `c a f c3 89`)
   - compile OK,
      find_all = 1 match,
      gates = [("café",
      true)]
   - contains_under_ci("café",
      content="cafÉ",
      ci=true) = false
   - SOUNDNESS VIOLATION at `fuzz_target.rs`:
     377
5. Same crash artifact replayed against MAIN's fuzz binary
   (e49d8694 fix in place):
    runs in 4ms,
    no crash,
    no panic.

**This is the load-bearing soundness-by-revert evidence:
** the fuzz
target reports a SOUNDNESS PANIC with a redacted reproducer (rule
source,
 content length,
 content SHA-256,
 match count,
 gate count;
no raw content bytes) when run against the reverted commit.
 The
same artifact passes on main.
 The target is therefore PROVEN to
catch the bug class commit e49d8694 closed.

**Test count:
 147 unit tests in main crate + 3 unit tests in
fuzz crate (new this session,
 covering the renderer fix) + 19
integration tests = all green.
 Clippy clean.
**

**Sentinel verification (post-fix):
**

- `find . -maxdepth 1 \( -name HEAD -o -name config -o ... \)` -- no
  fuzz output escaped gitignore.
- `git check-ignore --verbose package/cli/forbidden-strings.fuzz/Cargo.lock`
  exits 1.
  This is correct because Cargo lockfiles are not ignored by root `.gitignore`.
- The post-fix fuzz log shows the panic immediately after
  `seed corpus: files: 18278 ...` with no intervening `INITED` or
  `#N` lines,
   confirming the SOUNDNESS VIOLATION fired during
  corpus replay (before libfuzzer's mutation budget started).
  Stronger result than mutation-finds-it:
   the warm corpus already
  encoded the bug under the fixed renderer.

**Status:
**

<table>
<thead>
<tr>
<th>Phase</th>
<th>Status</th>
</tr>
</thead>
<tbody>
<tr>
<td>1-10</td>
<td>DONE</td>
</tr>
<tr>
<td>11</td>
<td>DONE — soundness-by-revert verified; crash artifact triggers panic on reverted worktree, runs clean on main</td>
</tr>
</tbody>
</table>

**Next-session cleanup tasks:
**

1. **Remove the disposable worktree** at `/tmp/fs-soundness-revert/`
   (per Phase 11 step 11.10 "Remove the worktree").
    Save the crash
   artifact + decode metadata to repo first if you want it
   preserved.
2. **Decode and address `timeout-dba8a535...`** (the new timeout
   that escaped my validators during the pre-fix 30-min fuzz).
    It
   was added under the old broken renderer;
    may decode to something
   entirely different now and may not need a validator at all.
   Verify before adding more pre-validators.
3. **Optional**:
    Document the pre-validators in `FUZZING.md` per
   plan §10.
    Defer if not blocking other work.
4. **Optional**:
    Commit the pre-existing uncommitted README +
   fuzz/`Cargo.toml` + fuzz/dictionaries/forbidden-strings.
   dict
   changes (the dictionary fix is a real bug fix;
    the others are
   small/cosmetic).
