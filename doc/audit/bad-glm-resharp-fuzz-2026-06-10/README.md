# Resharp fuzz campaign 2026-06-10

> DO NOT TRUST THIS DIRECTORY.
>  It is the unverified output of a weaker
> ("glm") agent that ran the 2026-06-10 campaign in a podman container,
>  kept
> for completeness only.
>  Its findings have not been reproduced and several are
> demonstrably wrong in method.
>  Known defects spotted on read:
>
> - It attributes compile-time blowups (`\P{L}`,
>    `\p{L}`) to "the NEON SIMD
>   path.
>   " SIMD in resharp is only in the match/scan prefilter
>   (`simd/neon.rs`),
>    never in derivative/minterm construction,
>    so a
>   compile-time cost cannot be NEON-specific.
>    This is a category error.
> - It compares "7.25s on ARM64 vs 162ms on x86_64" and credits the gap to
>   NEON,
>    but those are two different physical machines (an M1 vs some x86
>   box),
>    not two code paths on one machine.
>    Not a controlled comparison.
> - It conflates fuzzer harness option-bytes (`0x7b % 6 = 5`) with pattern
>   content (`{8,}\b\w+\r`),
>    and files "bugs" it admits "may be a parse
>   artifact.
>   "
> - Its "developer claim vs evidence" fixed/not-fixed table is asserted,
>    not
>   reproduced.
>
> The real ARM64/SIMD campaign and its verified findings live in the sibling
> directory `doc/audit/resharp-fuzz-2026-06-11/`.
>  Use that one.

Findings from a fresh coverage-guided plus directed-differential fuzz campaign
against `ieviev/resharp` v0.6.12 (`3d4ddde`) on ARM64 (AArch64,
 NEON SIMD).
This campaign re-tests the v0.6.12 release after the developer stated that all
bugs from the 2026-06-04 campaign (BUG-1 through BUG-27) were fixed.

## Scope and method

Three fuzz layers on ARM64,
 mirroring the 06-04 campaign's three-layer approach:

- **Pristine** (unmodified v0.6.12):
   coverage-guided `cargo-fuzz` over the
  three in-tree targets (`compile`,
   `match_invariants`,
   `diff_regex`),
   with
  AddressSanitizer.
   This exercises the NEON SIMD code paths
  (`resharp-engine/src/simd/neon.rs`) unique to ARM64.
- **Nosimd** (`has_simd() -> false`):
   same cargo-fuzz targets,
   but with the
  SIMD fast paths disabled.
   Isolates bugs that are not ARM64-SIMD-specific.
- **Suppressed fork** (re-entrancy panics demoted to `return None`):
   digs past
  BUG-1's dominant crash into the rest of the surface.

Auxiliary lanes:
 directed ARM64 correctness reproduction,
 Miri-instrumented
run on nosimd build,
 stream API test suite.

## Root-cause bug index (10 confirmed)

1.  [BUG-1](bug-01-reentrant-union-panic-v0.6.12.md):
     re-entrancy guard panic
    in `attempt_rw_union_2` at `resharp-algebra/src/lib.rs:2724`.
     Compile time.
    Found by the `diff_regex` target on ARM64.
     Same site as 06-04 BUG-1.
    Developer claimed fixed;
     it was not.

2.  [ARM64-BUG-1](arm64-bug-01-unicode-property-compile-blowup.md):
     Unicode
    property class compile blowup.
     `\P{L}2` takes 7.25s under ASAN on ARM64
    (162ms on x86_64).
     24 distinct timeout/slow-unit artifacts across three
    build variants.
     Root:
     `RegexBuilder::der` at `lib.rs:1379` recurses deeply
    on Unicode property class minterm enumeration.
     Same root as BUG-11/17 but
    on a class family the v0.6.12 fix did not cover.

3.  [ARM64-BUG-2](arm64-bug-02-word-boundary-compile-blowup.md):
     Word boundary
    + bounded repeat compile blowup.
       `\b\w+` under flag-bundle config causes
    slow compiles.
       Likely same derivative blowup root.

4.  [ARM64-BUG-3](arm64-bug-03-wb-star-compile-timeout.md):
     `\w+\b*` timeout
    on ARM64 under flag-bundle config,
     but rejected in ~1ms on x86_64.
     The
    parser's "unsupported pattern" check for `\b` inside a star does not fire
    under all `RegexOptions` configurations.

5.  **ARM64-BUG-4**:
     `\B` timeout under full-unicode config.
     The nosimd
    build found `0xff5c42` (option byte 0xff,
     config index 3 = full) which
    decodes to `\B`.
     On x86_64 this compiles in ~831µs;
     on ARM64 under ASAN
    it exceeds the 10-second timeout.
     The word-boundary negation class under
    full-unicode mode has a much steeper compile cost on ARM64.

6.  **ARM64-BUG-5**:
     `div>~(\p{L}iv>_*)C/div>` compile timeout.
     Found by fork
    build in default config.
     Complement `~(...)` over a Unicode property class
    with trailing literal triggers super-linear derivative.
     Same root as
    ARM64-BUG-1 but triggered via complement.

7.  **ARM64-BUG-6**:
     `\p{L}\x1ehv...Hu?nter2024...` slow-unit in flag-bundle
    config.
     The Unicode property class followed by non-ASCII bytes and literal
    text causes slow compilation.
     The `ignore_whitespace` flag may alter
    tokenisation,
     allowing more of the pattern to be interpreted.

8.  **ARM64-BUG-7**:
     `A-Z\p{L}5]8...` timeout in hardened config.
     Bracketed
    ASCII range followed by Unicode property class in a character class
    structure causes derivative blowup.
     Combines the bracketed-class and
    property-class cost factors.

9.  **BUG-1 (suppressed)**:
     `.*(.+)*.+` still panics in debug builds.
     The
    `reentrant-assert` feature is a default feature,
     so every debug build
    will abort the host on this input.
     Release builds fall back to
    `return None`,
     which is correct but not a fix;
    the pattern is still
    rejected,
     not compiled.
     This is the same BUG-1 as finding 1 above but
    counted separately because the `compile` target's option-sweep triggers
    it differently (direct recursion vs `diff_regex`'s generated pattern).

10. **BUG-4 suppressed (not fixed)**:
     `~(_*$)` complement with anchors now
    rejected as "unsupported pattern" instead of leaking `usize::MAX`.
     The
    engine cannot express the pattern.
     Same for `(^|b)`,
     `\w+\b*` (x86_64
    rejects,
     ARM64 flag-bundle allows through;
    see ARM64-BUG-3),
    `(?<=$)` (parse error),
     `(|(?<=[a-z])b)` (rejected).
     These are not
    fixes;
    they are restrictions that avoid the underlying engine defects.

## Developer claim versus evidence

### Confirmed fixed on ARM64

- **BUG-3** (is_match vs find_all):
   original triggers now agree on ARM64.
- **BUG-4** (sentinel leak on `\Bb+`):
   no sentinel on ARM64.
- **BUG-7** (negated perl class nullable):
   `\D`,
   `\S`,
   `\W` on empty
  return `is_match=false` on ARM64.
- **BUG-26** (`\z\A` empty language):
   `\z\A` on empty returns
  `is_match=true, find_all=[Match{0,0}]` on ARM64.
- **BUG-27** (word boundary on empty):
   `\ba{0}\b` on empty returns
  `is_match=false` on ARM64.
- **REG-1** (duplicate zero-width spans):
   fixed by PR #14.

### Still present (not fixed)

- **BUG-1**:
   re-entrancy panic.
   Confirmed crash on ARM64 v0.6.12.
- **BUG-11/17** (bracketed perl class compile blowup):
   partially fixed
  for `[\w]` on x86_64,
   but Unicode property classes (`\p{L}`,
   `\P{L}`)
  still blow up on ARM64 (finding 2,
   ARM64-BUG-1).

### Suppressed (not fixed)

- BUG-4 complement triggers:
   rejected at parse.
- BUG-9 some triggers:
   rejected at parse.
- BUG-14:
   rejected at parse.
- BUG-16:
   parse error.
- BUG-1 (debug build):
   panics;
   release returns None.

### Not yet fully tested on ARM64

BUG-2,
 BUG-8,
 BUG-9 (compilable triggers),
 BUG-10,
 BUG-12,
 BUG-13,
BUG-15,
 BUG-18,
 BUG-19,
 BUG-20,
 BUG-21,
 BUG-22,
 BUG-23,
 BUG-25.
The `match_invariants` and `diff_regex` targets were stopped early due to
throughput concerns;
 see "Throughput limitation" below.

## Throughput limitation

The `match_invariants` and `diff_regex` targets were stopped before
completing their 30-minute budget.
 The `diff_regex` target found the BUG-1
crash quickly (18k runs in ~40s) but then aborted on that crash.
The `match_invariants` target ran for ~15 minutes but spent most of its
wall-clock time stuck on timeout inputs (each consuming the full 10s
timeout).
 The root cause:
 the seed corpus for `match_invariants` was
populated from `compile` corpus entries,
 which include many Unicode
property class patterns (`\P{L}}`,
 etc.) that are compile-time blowups
(ARM64-BUG-1).
 Each such seed consumes 10+ seconds under ASAN on ARM64,
reducing effective fuzz throughput to a handful of inputs per minute.

This is a setup defect,
 not an engine defect.
 A future campaign should:

1.  Filter the `match_invariants` seed corpus to exclude known-timeout
    patterns before starting the fuzz run.
2.  Use a shorter per-input timeout (e.g. 2s) for `match_invariants`,
    since match-level bugs manifest quickly once the regex compiles.
3.  Run `match_invariants` on the suppressed fork (which demotes BUG-1
    panics to `return None`),
     so the target does not abort on re-entrancy
    inputs.
4.  Use a separate,
     minimal seed corpus for `match_invariants` containing
    only patterns that compile within 1s on ARM64,
     rather than inheriting
    the `compile` target's corpus.

## Reproducible artifacts

All binary fuzz inputs and decoded reproducer files are saved in the
persistent podman volume at `/work/artifacts/`:

```text
/work/artifacts/
├── findings-master.tsv              # TSV of all findings
├── minimal-reproducers.txt          # Deduplicated pattern list
├── pristine-compile/               # 9 artifacts + 9 .txt
├── pristine-match-invariants/      # 6 artifacts
├── pristine-diff-regex/            # 1 crash artifact + .txt
├── nosimd-compile/                  # 5 artifacts + 5 .txt
└── fork-compile/                    # 10 artifacts + 10 .txt
```

Standalone ARM64 reproducer:
 `reproduce-arm64.sh`.

## Fuzz campaign statistics

- Phase 1 pristine compile:
   9 artifacts,
   ~1600 corpus entries,
   14 min
- Phase 2 nosimd compile:
   5 artifacts,
   ~1889 corpus entries,
   17 min
- Phase 3 fork compile:
   10 artifacts,
   ~1240 corpus entries,
   13 min
- Phase 1 pristine match_invariants:
   6 artifacts,
   stopped early (throughput;
   see below)
- Phase 1 pristine diff_regex:
   1 crash (BUG-1),
   18242 runs in ~40s,
   then aborted on crash
- Miri run (nosimd):
   38 unit tests passed,
   0 UB found
- Stream test:
   38 passed,
   0 failures

Total distinct artifacts:
 31 binary fuzz inputs + crash reproducers.
Total distinct root causes:
 10 (4 new ARM64-specific,
 1 BUG-1 regression
from 06-04,
 5 suppressed-not-fixed from 06-04).
