# HANDOVER.resharp-panic-fix

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

State of the fix for two upstream `resharp` panics surfaced by
`fuzz_extract_gate_soundness`.
 Originally written against resharp 0.5.
x;
post-compact verification against resharp 0.6.0 (2026-05-16) confirmed
**both panic shapes still exist unchanged at the same source lines**.
The pre-validators and `catch_unwind` net both remain load-bearing.

## Post-compact verification (resharp 0.6.0, 2026-05-16)

- Bumped `resharp = "0.5.2"` → `resharp = "0.6"` in
  `package/cli/forbidden-strings/Cargo.toml`.
   Clean build (19s),
  no API breakage;
   `Regex::new` / `find_all` / `is_match` / `Match`
  signatures unchanged across the 0.5.
  x → 0.6.0 bump.
- 121 unit + 19 integration tests all pass against 0.6.0.
- Direct probe at `/tmp/probe-resharp-06` (resharp 0.6,
   RUSTFLAGS
  matching cargo-fuzz defaults) reproduces BOTH crashes at the SAME
  source lines:
  - Shape 1 (compile,
     `attempt to add with overflow`):
    `resharp-algebra/src/lib.rs:2470`
  - Shape 2 (runtime,
     `unexpected end 0 > N`):
    `resharp/src/engine.rs:1020`
- Discovered during 0.6.0 probing:
   shape 2's panic is behind a
  `debug_assert!`.
   With `debug_assertions` OFF (release default,
  which is our build config),
   the path does NOT panic;
   instead it
  returns corrupted matches.
   The probe shows 62 spurious matches on
  a 64-byte input.
   **This means `catch_unwind` cannot help in
  release for shape 2;
   the pre-validator is the only defense.
  **
- Decision:
   KEEP both layers (pre-validators + catch_unwind +
  profile settings).
   All three are load-bearing for at least one
  shape;
   none are now-redundant.
- Updated `TROUBLESHOOTING.resharp.md` to add Bug B (intersection
  with lookbehind,
   silent-corruption-in-release) and Bug C
  (intersection with `\w` and `$`,
   overflow-checks-load-bearing).
- Version-pinned comments in
  `package/cli/forbidden-strings/src/{rules.rs,rules/engine.rs,
  rules/engine_tests.rs,lib.rs}` updated to read "resharp 0.5.
  x
  through 0.6.
  x" so future readers know the bugs are not fixed.

Verified post-fix against the preserved crash artifacts:

- `crash-aba4ef4e00b9a1cde5962347c98cccd0b29b9174` (shape 1,
   runtime
  panic in 0.5.
  x):
   runs in 0ms against the 0.6.0+fix binary;
   no crash.
- `crash-ecaf28b2bdba9b7ae1f7b465ea6ff2bc77b8458a` (shape 2,
   compile
  panic in 0.5.
  x):
   runs in 0ms against the 0.6.0+fix binary;
   no crash.

Both artifacts now exit cleanly.
 Combined with the direct probe showing
0.6.0 still panics on the same source lines when invoked without our
guards,
 this is the proof that the pre-validators + catch_unwind +
profile settings handle the 0.5.
x crash inputs correctly against 0.6.0.

Bounded fresh fuzz run:
 4714 iterations of `fuzz_extract_gate_soundness`
in ~45 seconds against resharp 0.6.0,
 zero new crashes.

cargo-fuzz infrastructure note:
 cargo-fuzz 0.13.1 defaults to the musl
target which is incompatible with the ASAN flags it sets via RUSTFLAGS
(`sanitizer is incompatible with statically linked libc`).
 The
workaround is `--target x86_64-unknown-linux-gnu` explicitly on
`cargo fuzz build` / `cargo fuzz run`.
 Threaded through the mise
tasks (`fuzz:build`,
 `fuzz:smoke`,
 `fuzz:run`) in commit `7b2caf88`.

### Late-2026-05-16 session update: 5 pre-validators landed, fuzz hardened

**Status:
** 60s fuzz on main now PASSES cleanly (34328 iterations).
120s fuzz in reverted worktree still in progress -- hit a second
timeout shape (`(?i) ###(?:\s&üü)(?:####)+...`) without complement,
fixed by widening `complement_intersection_quantified_group` in
cbc1616e (drop the complement requirement;
 intersection + quantified
group anywhere is enough to trigger the hang).

**Pre-validators added/widened today (newest first):
**

```text
cbc1616e fix: widen intersection+quant hang detector
3d996936 docs: trace resharp hang to prefix.rs visited-set bug
e5ab8c6f fix: pre-validate resharp algebra-hang shape
4fb14f4c fix: pre-validate alt-lookaround sibling shape
9ac0b3a9 fix: pre-validate nested grouped quantifiers
```

**Bug E (new) -- root cause traced via gdb:
**

The intersection+quantified-group hang traces to
`resharp-engine/src/prefix.rs:27` in `calc_prefix_sets_inner`.
 The
`redundant` set is initialized with `{BOT, start}` and never updated
inside the loop;
 derivative chains producing unique nodes
indefinitely never terminate.
 Minimal-patch prototype:
`redundant.insert(node);` at end of each loop iteration.
 This is
the first resharp bug with a viable upstream patch (see
TROUBLESHOOTING.
`resharp.md` Bug E for the full writeup).

**Resume work next session:
**

1. Re-run 120s soundness-by-revert in `/tmp/fs-soundness-revert`.
   Sync src first,
    clear stale timeout artifacts:
   ```text
   cp /var/home/user/Monochromatic/package/cli/forbidden-strings/src/{fuzz_api,rules}.rs /tmp/fs-soundness-revert/packages/cli/forbidden-strings/src/
   cp /var/home/user/Monochromatic/package/cli/forbidden-strings/src/rule/{engine,engine_tests}.rs /tmp/fs-soundness-revert/packages/cli/forbidden-strings/src/rule/
   rm /tmp/fs-soundness-revert/packages/cli/forbidden-strings/fuzz/artifacts/fuzz_extract_gate_soundness/timeout-*
   cd /tmp/fs-soundness-revert/packages/cli/forbidden-strings
   mise run fuzz:run fuzz_extract_gate_soundness -max_total_time=120 -timeout=10
   ```
   Expect SOUNDNESS PANIC.
    If still hitting non-soundness halts,
   decode via `/tmp/probe-slow-unit/target/release/probe <path>` and
   add another pre-validator.
2. If panic still doesn't fire after fuzz runs 120s clean,
    bias
   `synth_content` to produce Unicode case-flipped variants.
3. File Bug E upstream at github.
   com/ieviev/resharp.

### Late-2026-05-16 session (~23:00): Bug F + case-flip bias + widened B

**Commits landed since previous handover:
**

```text
a08b45ed fuzz(forbidden-strings): bias synth_content for Unicode case-flip
214c03b1 fix(forbidden-strings): widen alt-lookaround validator threshold
6ff333f1 fix(forbidden-strings): pre-validate Bug F nested-lookahead overflow
```

**Bug F discovered (NEW resharp bug):
**

- `attempt to add with overflow` at `resharp-algebra/src/lib.rs:2470`
  (`tail_rel + la_rel`,
   both u32).
   Lookahead-chain `rel` length
  saturates to u32:
  :
  MAX and the next add overflows under
  debug-assertions.
   Production (debug-assertions=off) silently wraps
  to 0;
   likely produces wrong matches but won't visibly panic.
- Added `nested_lookahead_in_quantified_group` pre-validator catching
  the nested-quant + outer-min≥2 shape (e.g. `(?:(?:(?!X)){1,5}){2,4}`).
- Documented in TROUBLESHOOTING.
  `resharp.md` Bug F (see next-session
  todo to add).
- **Status:
  ** validator is too narrow;
   the broader trigger class
  (quantified lookahead followed by short trailing literal) is not
  yet caught.
   See "KNOWN GAP" in HANDOVER.
  `forbidden-strings-fuzzing.md`.

**Bug B widened (`lookaround_in_alternation_with_sibling`):
**

Dropped the `total_lookarounds >= 2` requirement.
 Bisected from
`crash-c3c364eb3a03114a52015721c02cba0bf20eb496` which has only
ONE lookaround inside the alternation but trips `engine.rs`:
1020 at
find_all time.
 The compile succeeds;
 the panic is input-dependent.

**Case-flip bias landed:
**

When rule has both `(?i)` and `(?u)` flags,
 `synth_content` with 50%
probability flips embedded é/ñ/ü/ö/ç to É/Ñ/Ü/Ö/Ç in the content
buffer.
 This bias closes the gap between "soundness panic IS reachable
via direct probe" and "fuzz never discovers it via random mutation".

**TODO for next session:
**

1. Widen `nested_lookahead_in_quantified_group` to also catch
   "quantified lookahead group + trailing content" (Bug F broader
   class).
    Bisect probes in `/tmp/probe-slow-unit/src/bin/bisect_f7,
   bisect_f8.rs` document the exact triggers.
2. Sync to worktree,
    re-fuzz.
    Expect SOUNDNESS PANIC.
3. Add Bug F section to TROUBLESHOOTING.
   `resharp.md`.
4. File Bug E upstream at github.
   com/ieviev/resharp (still pending).

---

Original (pre-late-session) outstanding section preserved below for
historical context only:

- Soundness-by-revert phase 11 validation:
   **partial progress;
   60s
  gate now broken;
   multiple follow-up pre-validators needed.
  **
  Findings late-2026-05-16,
   in the order they crystallised:

  1. **`cd9b2dbf` (stacked-quantifier pre-validator) was solving the
     wrong shape.
     ** The pre-validator catches bare-stacked
     quantifiers like `a**` and `\D{5,11}{5,11}`.
      But the fuzz
     target's `Node::Quant` renderer at
     `fuzz/src/generators.rs:1292-1300` ALWAYS wraps the quantified
     atom in `(?:...)` non-capturing group:
     ```rust
     Node::Quant(body, kind) => {
         out.push_str("(?:");
         body.render(out);
         out.push(')');
         kind.render(out);
     }
     ```
     So the generator NEVER emits bare-stacked source.
      A deeply-
     nested Quant chain renders as
     `(?:(?:(?:(?:(?:atom){N,M}){N,M}){N,M}){N,M}){N,M}` (grouped
     via `(?:...)`),
      and my pre-validator's "two quantifier suffixes
     back-to-back" logic does not fire because the `)` resets the
     `just_consumed_quant` state.
      The pre-validator is a no-op for
     the fuzz target;
      its unit tests pass because they use a
     hand-written bare-stacked source the fuzz never produces.

     Decoded slow-unit `slow-unit-0cfbc4b8b9945074fe5214a96c503f6e994e3b97`
     rendered source via the probe at `/tmp/probe-slow-unit/`:
     ```text
     (?iu)(?:(?:(?:(?:(?:\d){5,11}){5,11}){5,11}){5,11}){5,11}(?:(?:(?:(?:(?:\d)*)*)*)*)*aa
     ```
     `compile_rule_src` on this source takes ~3.26s and errors with
     `CompiledTooBig(268_435_456)`.
      The shape that needs to be
     caught is **grouped-via-`(?:)` nested quantifiers** at depth
     4+,
      not bare-stacked.
      The pre-validator either needs a new
     algorithm (count chains of `){quant}` adjacency) or a sibling
     that walks paren-depth-tracked quantified-group nesting.

     The pre-validator is still correct on its own terms (it catches
     bare-stacked patterns IF anyone hand-writes one in a rules
     file).
      It is just not the load-bearing defence the fuzz target
     needs.
      Keep it,
      add the grouped sibling.

  2. **`2f4d27b0` (Unicode literal alphabet widening) exposed two
     pre-existing resharp panic shapes that the fuzz now reaches
     within ~30s.
     ** The new pick=5 branch in `gen_literal_bytes`
     emits one of `é`/`ñ`/`ü`/`ö`/`ç` as a 2-byte UTF-8 pair.
     This is necessary for the soundness-by-revert path (the
     e49d8694 case-fold bug requires non-ASCII letters in the
     gate).
      But the wider literal alphabet also lets the
     generator construct rule sources combining `&` (intersection)
     with `(?=`/`(?!` (lookahead) -- a shape neither
     `intersection_with_lookbehind` (covers lookBEHIND only) nor
     `intersection_with_word_end_alternation` (covers `&`+`\w`+`$`)
     catches.
      Resharp's `engine.rs:1020` `debug_assert!` then
     fires in debug builds and the libfuzzer panic hook aborts
     before our `catch_unwind` net can intercept.

     Crash artifacts left over from the failed 60s gate on main
     (post-widening,
      with stacked-quant fix in place):
     - `crash-8cba104f0805ccb567513aff895398a4f652200c` — Alt
       containing `Lookaround(NotAhead, ...)` siblings with
       intersection.
        The reproducer panics on
       `resharp-algebra/0.6.0/src/lib.rs:2470` (algebra
       overflow shape),
        but the rule shape is lookahead+intersect.

     - `slow-unit-0cfbc4b8b9945074fe5214a96c503f6e994e3b97` —
       same hash as the original slow-unit,
        but the bytes now
       represent the grouped-quantifier shape under (?
       iu);
       compile takes ~3s,
        which combined with ASAN overhead
       crosses libfuzzer's 10s slow-unit threshold.

     Both artifacts must remain on disk for the next session's
     diagnosis;
      do not `rm` them.
      The artifact-permission denial
     in the prior turn was correct.

  3. **The 60s fuzz on main is now BROKEN.
     ** Pre-widening (commit
     order through `cd9b2dbf`),
      two consecutive 60s `fuzz:run
     fuzz_extract_gate_soundness` invocations completed cleanly
     (9561 + 3858 iterations).
      Post-widening (`2f4d27b0`),
      the
     same command exits with libfuzzer status 77 within ~60s,
     hitting the lookahead+intersect resharp panic and the
     grouped-quantifier slow-unit.
      The user instructed
     "Don't revert it" -- the widening stays in HEAD as commit
     `4d5563cb` (reapplied via revert-of-revert;
      `1976d0b9` is
     the discarded revert).

  4. **Manually probed soundness IS reachable** in the reverted
     worktree:
      `(?iu)café` vs content `CAFÉ` triggers the
     expected soundness violation (probe at
     `/tmp/probe-slow-unit/`).
      The bug class fixed by `e49d8694`
     is real.
      The fuzz target cannot reach it because the
     generator's `synth_content` does not produce
     Unicode-case-flipped variants of the rule's non-ASCII
     bytes -- mutations there are uniform random 0..=255 byte
     writes,
      so converging on the `0xA9` -> `0x89` flip is
     stochastic.
      A targeted bias in `synth_content`,
      OR a
     hand-crafted seed in `fuzz/corpus/fuzz_extract_gate_soundness/`
     that decodes to the violating shape,
      would let phase 11
     panic deterministically.

  **Follow-ups,
   in priority order:
  **
  1. Add a `nested_grouped_quantifier` pre-validator that flags
     depth-4+ chains of `(?:body){quant}` nesting.
      Or extend
     `stacked_quantifier` with a paren-depth counter that
     re-arms `just_consumed_quant` across `)` when the inner
     body was itself quantified.
      Algorithm sketch:
      walk paren
     depth;
      when a group close `)` is followed by a quantifier
     start,
      increment a per-depth `quantified_closes` counter;
     flag when 4+ such closes appear in adjacent positions
     (separated only by quantifier suffixes).
  2. Generalise `intersection_with_lookbehind` to also fire on
     `&` + lookahead (`(?=`,
      `(?!`).
      Or add a sibling
     `intersection_with_lookaround`.
      The reproducer for the
     post-widening crash is in
     `fuzz/artifacts/fuzz_extract_gate_soundness/crash-8cba104f...`.
  3. Re-run 60 s fuzz on main:
      must complete clean before
     declaring the load-bearing verification gate green again.
  4. Re-run 120 s soundness-by-revert:
      expect either the
     grouped-quantifier slow-unit and lookahead+intersect to
     fire FIRST (if the new pre-validators are still
     incomplete),
      or the SOUNDNESS PANIC on a non-ASCII
     case-fold shape (if the pre-validators are thorough and
     libfuzzer's mutations reach the case-flip).
  5. If (4) keeps missing the soundness panic,
      bias
     `synth_content` (in `fuzz/src/generators.rs` near
     line 1383) to emit Unicode-case-flipped variants of any
     non-ASCII letter bytes in the rule's literals -- not
     uniform 0..=255 mutations.
      Or add a deterministic seed.

- `/tmp/fs-fuzz-validate`,
   `/tmp/fs-soundness-revert`,
  `/tmp/fs-crash-artifacts/`,
   and `/tmp/probe-resharp-06/` all
  removed during cleanup.
   `/tmp/probe-slow-unit/` retained for
  the soundness-shape probe,
   the bare-stacked compile-timing
  measurements (1.4s unicode-off + 1.5s unicode-on retry =
  2.9s per `compile_rule_src` call for the BARE form),
   and the
  grouped-form probe (`compile_rule_src` Err in 3.26s on the
  actual fuzz-rendered source).

- Commit log on `main` (newest first):
  - `4d5563cb` — Reapply widening (un-revert per user
    instruction).
  - `1976d0b9` — Revert of widening (discarded;
     superseded by
    `4d5563cb`).
  - `903d973f` — docs(handover):
     record stacked-quantifier
    slow-unit fix and remaining phase 11 blockers (this file's
    PREVIOUS state;
     the current update supersedes it).
  - `2f4d27b0` — feat(forbidden-strings):
     widen fuzz literal
    alphabet to include Unicode lowercase letters.
  - `cd9b2dbf` — fix(forbidden-strings):
     pre-validate stacked
    quantifiers to skip regex-crate compile blowup.
- `fuzz:run` arg-spread fixed in commit `202ed6b6`:
   mise's `usage_args`
  arrives shell-quoted (`'-a' '-b'`);
   the previous `split row ' '`
  kept the surrounding quotes as part of each element,
   so libfuzzer
  treated `'-max_total_time=10'` as a corpus directory.
   Now splits on
  the inter-arg `"' '"` boundary and trims the lone outer quotes.

---

## Original session notes (against resharp 0.5.x, pre-compact)

## What the user asked for

1. Fix two upstream resharp panics by **both** mechanisms:
   - `std::panic::catch_unwind` wrapping (defense in depth)
   - Pre-validator pre-flight rejection of known-bad shapes
2. **Do NOT** file upstream issues yet -- only update
   `TROUBLESHOOTING.resharp.md`.
3. Per their later note:
    do not skip when probing fails to reproduce;
   document any cargo-fuzz-vs-`cargo run` discrepancies in a separate
   TROUBLESHOOTING doc.
4. Per their final note:
    resharp 6.0 was released mid-task;
    finish
   current logical unit,
    write handover,
    commit,
    defer everything else.

## Status of the original task list

- [x] Identify exact regex source strings from crash inputs
- [x] Wrap `Regex::new` in `catch_unwind` inside `compile_rule_src`
- [x] Wrap resharp `find_all`/`is_match` in `catch_unwind` inside
      `CompiledRegex` methods
- [x] Add pre-validation for the two specific shapes
- [x] Add tests for both crash shapes (6 new tests in
      `src/rules/engine_tests.rs`)
- [x] Verify with mise tasks (build,
       test,
       lint,
       lint:
      clippy -- all pass)
- [x] Commit fixes (committed as `23ca7a1f` -- single commit,
       not the
      two-commit split originally planned,
       due to time pressure)
- [x] Bump to resharp 0.6.0 and re-verify (post-compact 2026-05-16)
- [x] Update `TROUBLESHOOTING.resharp.md` with 0.6.0 verification
      and the two new Bug B / Bug C sections
- [ ] Re-run both fuzz crash artifacts against the fix
      **(deferred to post-bump cleanup pass)**
- [ ] Soundness-by-revert validation **(deferred)**
- [ ] Update `HANDOVER.forbidden-strings-fuzzing.md` and clean up
      worktree/artifacts **(deferred)**

## What's landed in the working tree (uncommitted)

### `package/cli/forbidden-strings/Cargo.toml`

`[profile.release]` changed:

- `panic = "abort"` → `panic = "unwind"`.
   `catch_unwind` cannot
  intercept under "abort" -- the process aborts before the unwind
  barrier runs.
- Added `overflow-checks = true`.
   Rust's default release profile
  silences integer-overflow panics,
   but resharp-algebra's
  `attempt_rw_concat_2` panic IS an overflow panic.
   Without
  overflow-checks the `add` wraps silently in production,
   building
  the wrong DFA → fail-open.
   With it the wrap becomes a real panic
  the wrapper intercepts.
- Long header comment above the profile block explains both
  load-bearing reasons.

This is the **critical** part of the fix.
 If only the catch_unwind
wrappers landed,
 crash 2 would still silently corrupt rules in
production builds.

### `package/cli/forbidden-strings/src/rule.rs`

- Added `use std::panic::{catch_unwind, AssertUnwindSafe};` import
  with the rationale block above it.
- Re-export bundle widened:
  `pub use engine::{intersection_with_lookbehind,
   intersection_with_word_end_alternation, lookaround_in_complement,
   requires_resharp, CompiledRegex};`
- `compile_rule_src` now runs the two new pre-validators after
  `lookaround_in_complement`,
   and wraps the `Regex::new` call in
  `catch_unwind(AssertUnwindSafe(|| Regex::new(src)))`.
   Outer-`Err`
  arm of `catch_unwind` formats as:
  `"(resharp): panic during compile (upstream resharp 0.5.x bug). See TROUBLESHOOTING.resharp.md."`
- The crate-wide error shape is preserved:
   every error is still
  prefixed with `rule on line N (resharp): ...` by the outer loader.

### `package/cli/forbidden-strings/src/rule/engine.rs`

- Added `use std::panic::{catch_unwind, AssertUnwindSafe};`.
   The
  import-site comment is the long-form primer explaining
  `UnwindSafe`,
   `AssertUnwindSafe`,
   `RefUnwindSafe`,
   mutex
  poisoning,
   soundness rationale.
- `CompiledRegex::find_all` and `CompiledRegex::is_match` both
  wrap their resharp branch in
  `catch_unwind(AssertUnwindSafe(|| re.<method>(content)))`.
   The
  match flattens `Result<Result<T, resharp::Error>, Panic>` into
  the existing `Result<T, ()>` contract.
   `Plain` branches are
  unchanged.
- Two new public pre-validators alongside `lookaround_in_complement`:
  - `intersection_with_lookbehind(src: &str) -> Option<String>`:
    fires when intersection `&` (outside class) AND a lookbehind
    `(?<=` or `(?<!` both appear in the source.
     Returns an
    actionable error message ending in the standard
    `TROUBLESHOOT_REF` ("See TROUBLESHOOTING.
    `resharp.md` ...").
  - `intersection_with_word_end_alternation(src: &str) -> Option<String>`:
    fires when intersection `&`,
     `\w` shorthand,
     and `$` end-anchor
    all appear in the source (all outside character classes).

Both detectors are single-pass byte walkers,
 mirror the style of
`lookaround_in_complement`,
 and skip escaped sequences and
character-class interiors.

### `package/cli/forbidden-strings/src/rule/engine_tests.rs`

6 new tests at the end of the file:

- `intersection_with_lookbehind_fires_on_minimal_shape`
- `intersection_with_lookbehind_skips_safe_shapes`
- `intersection_with_word_end_alternation_fires_on_minimal_shape`
- `intersection_with_word_end_alternation_skips_safe_shapes`
- `compile_rule_src_does_not_panic_on_known_bad_shapes`
  (end-to-end -- drives `compile_rule_src` on every minimum
  reproducer;
   asserts `Err` rather than process panic)
- `find_all_catches_runtime_panic_via_catch_unwind`
  (bypasses pre-validators by calling `resharp::Regex::new` then
  `CompiledRegex::find_all`;
   asserts no panic propagates)

`mise run //package/cli/forbidden-strings:test` reports 121 unit
tests passing (was 115 pre-patch).
 `cargo check` + `clippy --release
-- -D warnings` both clean.

## Bisection notes (worth saving even after resharp 6.0)

Reproducing requires `RUSTFLAGS="-C overflow-checks=on -C
debug-assertions=on"` (`cargo fuzz` sets these by default;
 plain
`cargo run --release` does NOT).
 This was the load-bearing clue --
without it both shapes silently return `Ok` instead of panicking
during ad-hoc probing.

### Crash 1 (runtime, `resharp/src/engine.rs:1020` "unexpected end")

Minimum reproducer:
 `(?:(?=a)&(?<=_))` with content ≥ 64 bytes.
Trigger:
 intersection where one side has a lookahead and the other
has a lookbehind (specifically `(?<=_)` -- bare `_` wildcard inside
the lookbehind matters;
 `(?<=b)` does NOT crash).

### Crash 2 (compile, `resharp-algebra/src/lib.rs:2470` "attempt to add with overflow")

Minimum reproducer:
 `(?:\w|$)(?:(?![1g]\_X)& a)` (no scoped flag
required).
 Trigger combination:
 alternation containing both `\w`
and `$`,
 concatenated with intersection `&` whose operand contains
a negative lookahead with a character class followed by additional
literal bytes.

Many minor variations PANIC;
 many superficially similar shapes
return `Algebra(UnsupportedPattern)` (clean Err,
 no panic):

- `(?:(?=a)&(?=b))` → Err (two lookaheads,
   not panic)
- `(?:(?<=a)&b)` → Err (lookbehind on left,
   plain on right)
- `(?:\w|$)&a` → Err (no neg-lookahead-with-class on either side)
- `(?u:~(\_))` → Ok (scoped-flag + escaped-_ + complement is fine
  in isolation -- the issue is the COMBINATION above)

So pre-validators 1 & 2 are **conservative supersets** of the
actual panic surface.
 They will reject some patterns that would
have returned `Err(UnsupportedPattern)` upstream -- which we
consider a wash,
 because both routes refuse the rule.

## What remains in `/tmp/`

- `/tmp/fs-crash-artifacts/crash-aba4ef4e...` (crash 1 artifact)
- `/tmp/fs-crash-artifacts/crash-ecaf28b2...` (crash 2 artifact)
- `/tmp/fs-fuzz-validate/` — disposable worktree at commit
  `c6792310` (the pre-fix state).
   The probe-shapes helper binary
  has been cleaned up.
   The worktree is no longer load-bearing;
  remove with `git worktree remove /tmp/fs-fuzz-validate --force`
  if the resharp-6.0 bump invalidates the 0.5.
  x crash artifacts.

## Resume after compact

**Step 1 (highest priority):
 bump resharp to 6.0.
**

```bash
cd package/cli/forbidden-strings
# Read upstream changelog before bumping; the API surface may have
# changed (Regex::new signature, find_all return shape, error type).
cargo update -p resharp --precise 6.0.0   # or `cargo add resharp@6` if needed
mise run //package/cli/forbidden-strings:build
mise run //package/cli/forbidden-strings:lint:clippy
mise run //package/cli/forbidden-strings:test
```

Likely fallout to expect:

- `resharp::Error` variants may have moved or been renamed
  (used in `engine.rs` via `.map_err(|_| ())` -- which is
  variant-agnostic,
   so probably fine).
- `Regex::new` API stayed put in 0.5.
  x → 6.0 transitions is the
  typical Rust crate norm;
   verify before assuming.
- The two panic shapes MAY be fixed upstream.
   If `cargo +nightly
  fuzz run fuzz_extract_gate_soundness <artifact>` against the
  preserved crash artifacts no longer reproduces against
  resharp 6.0,
   the pre-validators are now unnecessary churn:
  consider removing them or keeping as defense-in-depth.

**Step 2:
 re-fuzz with resharp 6.0.
**

Run the existing fuzz target for 120s (or 600s if no panic at 120s).
The artifacts under `/tmp/fs-crash-artifacts/` may no longer
reproduce;
 if they don't,
 generate fresh ones via a clean fuzz run
and document any NEW panic shapes.

**Step 3:
 TROUBLESHOOTING.
`resharp.md`.
**

Write against **resharp 6.0** shapes,
 not 0.5.
x.
 If resharp 6.0
genuinely fixed both shapes,
 the doc shrinks to a one-line
"defense-in-depth catch_unwind + overflow-checks were added in
[commit] in case any future regression reintroduces this class.
"
If new panics appear,
 document each per the `troubleshooting-doc`
skill.

**Step 4:
 keep the load-bearing infrastructure.
**

Regardless of resharp 6.0's bug fixes,
 **do NOT revert**:

- The `panic = "unwind"` + `overflow-checks = true` profile
  settings.
   These cost ~5% binary size for the safety net every
  future resharp release deserves.
- The `catch_unwind` wrappers in `engine.rs` and `rules.rs`.
  These are version-agnostic defense in depth.
- The 6 new unit tests (some will need their case lists trimmed
  if 6.0 changes which shapes are accepted vs error).

The pre-validators (`intersection_with_lookbehind`,
`intersection_with_word_end_alternation`) are the 0.5.
x-specific
parts.
 Decide their fate based on the resharp 6.0 fuzz run:

- If 6.0 fixes both shapes → consider removing the pre-validators
  (smaller surface area,
   less to maintain).
   Keep their tests as
  "shapes that used to crash 0.5.
  x and now compile/error cleanly".
- If 6.0 leaves either shape broken → keep the pre-validator that
  matches the still-broken shape;
   remove the other.

**Step 5:
 soundness-by-revert (deferred from the original task).
**

Once resharp 6.0 is wired up and the test suite passes,
 run the
soundness-by-revert validation per the plan (revert e49d8694,
 run
fuzz_extract_gate_soundness for 120s,
 expect the ORIGINAL soundness
panic).
 The point is unchanged:
 prove the primary fuzz target's
load-bearing claim.

## Verification commands that pass right now (against resharp 0.5.x)

```bash
mise run //package/cli/forbidden-strings:build          # 18s clean compile
mise run //package/cli/forbidden-strings:lint           # 0.2s
mise run //package/cli/forbidden-strings:lint:clippy    # 0.2s, no warnings
mise run //package/cli/forbidden-strings:test           # 121 unit + 19 integration, 0 fail
```

## Files touched (uncommitted)

```text
modified:   package/cli/forbidden-strings/Cargo.toml
modified:   package/cli/forbidden-strings/src/rule.rs
modified:   package/cli/forbidden-strings/src/rule/engine.rs
modified:   package/cli/forbidden-strings/src/rule/engine_tests.rs
new file:   HANDOVER.resharp-panic-fix.md
```

## Recommended commit split (before compact, per CLAUDE.md eager-commit)

Two commits feel right:

1. `fix(forbidden-strings): make scanner panic-safe against resharp engine`
   - `Cargo.toml` profile (panic=unwind,
      overflow-checks=true)
   - `src/rules.rs` catch_unwind wrap on `Regex::new`
   - `src/rules/engine.rs` catch_unwind wraps on `find_all`/`is_match`
   - Subset of the new tests covering the catch_unwind paths
2. `fix(forbidden-strings): pre-validate resharp 0.5.x panic shapes`
   - `src/rules/engine.rs` two new pre-validators + their re-exports
   - `src/rules.rs` wiring `compile_rule_src` to call them
   - Remaining tests (pre-validator units + end-to-end
     `compile_rule_src_does_not_panic_on_known_bad_shapes`)

If the resharp 6.0 bump in the next session ends up removing the
pre-validators entirely,
 splitting the commits like this means a
single `git revert <commit 2>` cleans up without disturbing the
defense-in-depth changes.
 If they land as one commit,
 the bump
becomes a manual surgery.

## Open question / deferred decision

User said "do both fixes" (= catch_unwind + pre-validate).
 With
resharp 6.0 fixing the bugs upstream,
 the pre-validators become
0.5.
x-only complexity.
 Should they be reverted as part of the
resharp bump,
 or kept as a belt-and-braces guard?

My read:
 revert if 6.0 demonstrably fixes both shapes.
 Keep the
catch_unwind + Cargo profile work always.
 But this is the user's
call -- surface it in the resharp-6.0-bump session.
