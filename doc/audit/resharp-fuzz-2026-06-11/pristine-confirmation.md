# Pristine-engine confirmation of the oracle-only findings

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

The five soundness bugs that were found only through the oracle harness (bug-02,
bug-03,
 bug-07,
 bug-08,
 bug-10) were originally reproduced through `repro-simd`,
which links the instrumented engine `tools/resharp-instr` (the `has_simd()`
atomic override plus prefilter counters).
 At `override = 0` that patch is meant
to be behaviour-neutral,
 but "meant to be" is exactly the assumption this
campaign exists to reject.
 arm-bug-01,
 bug-04,
 bug-05,
 and bug-06 never needed
this step (arm-bug-01 is a line read in pristine source;
 bug-04/05/06 come from
the in-tree fuzzers,
 which link the unmodified `resharp-v0612`).
 The five
oracle-only bugs did.

## Closure 1: byte-identical on the unmodified crate

`tools/pristine-repro` is a throwaway binary depending on the stock
`resharp-v0612/resharp-engine` (no override,
 no counters),
 built with the same
`debug-assertions + overflow-checks` profile.
 It runs the documented minimal
reproducer for each of the five and prints the raw outputs.
 Every one reproduces
identically to the instrumented run:

```txt
bug-02  "(?<=a)" on "b": im=false fa=[] fan=Some((0, 0))    CONFIRMED
bug-02  "\BU"    on "U": im=false fa=[] fan=Some((0, 1))    CONFIRMED
bug-03  "(?=c)"  on "c":  find_all=[(0,0)]        stream=[(1,1)]        CONFIRMED
bug-03  "\b"     on "ab": find_all=[(0,0),(2,2)]  stream=[(1,1),(2,2)]  CONFIRMED
bug-03  "(?!\A)" on "ab": find_all=[(1,1),(2,2)]  stream=[(0,0),(2,2)]  CONFIRMED
bug-07  "~(\A|\n+){2}" on "\n\n": default=[(1,1),(2,2)] hardened=[(2,2)] CONFIRMED
bug-08  "[0-9]{2}~(\z{1,3}|^{2}\W{0})+" on "00" (flags): is_match=false find_all=[(0,2)] CONFIRMED
bug-10  "~(.{1,3}\z){2,4}" on "ab": find_all=[(0,2),(2,2)] find_anchored=Some((0,1)) CONFIRMED
```

The instrumentation is therefore not the cause of any of these.
 The rust
snippets in the individual bug files (which call the stock `resharp` crate) are
now backed by an actual stock-crate run,
 not only by the instrumented harness.

## Closure 2: orthogonality (the "10 distinct" count is defensible)

`code-quality.md` notes that several of these are "the same conceptual mistake
(zero-width / complement / end-anchor handling) rediscovered in different
drivers".
 The obvious pushback is that bug-02/03/07/08/10 are one bug seen
through five queries.
 They are counted separately because each is separately
reproducible (distinct minimal pattern,
 distinct query) and separately fixable.
To show "separately fixable" concretely rather than assert it:

`tools/pristine-repro-fixtest` is the same five reproducers run against
`resharp-fixtest`,
 which is v0.6.12 with exactly one change:
 the arm-bug-01 fix
at `fwd.rs:123` (`search_start = if max_end == 0 { 1 }` -> `{ 0 }`).
 That fix
repairs arm-bug-01 in isolation (verified separately:
 `^$` on `"\n\n"` then
returns `[0:0, 1:1, 2:2]` and the SIMD differential goes quiet).
 With that fix
in place,
 all five oracle bugs still fire,
 byte-identical to above.
 A fix to one
driver does not touch the others;
 they live in different drivers
(`find_anchored`'s context-free fast path,
 the stream DFA,
 the hardened scan
selector,
 the flags-config `is_match` gate,
 the `find_anchored` longest-span
gate).
 The shared **theme** is real and is recorded in `code-quality.md` as a
systemic signal;
 the **count** of separately-reproducible,
 separately-fixable
defects is ten.

This mirrors the 2026-06-04 campaign,
 which counted BUG-7 / BUG-26 / BUG-27 as
distinct despite a shared nullability theme.

## Tooling

- `tools/pristine-repro` (`/tmp/agent/pristine-repro`):
   stock-engine reproducer.
- `tools/pristine-repro-fixtest` (`/tmp/agent/pristine-repro-fixtest`):
   same five
  against the arm-bug-01-fixed engine.

Both are mirrored under the M1 `tools/` tree alongside the rest of the harness.
