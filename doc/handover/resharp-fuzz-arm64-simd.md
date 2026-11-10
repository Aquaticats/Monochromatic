# Handover: resharp ARM64 / NEON SIMD fuzz campaign (2026-06-11)

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Living handover for the ARM64 (Apple M1,
 NEON SIMD) fuzz campaign against
`ieviev/resharp` v0.6.12.
 Context compacts;
 this file is the single source of
truth for resuming with zero rediscovery.
 Per-bug writeups go under
`doc/audit/resharp-fuzz-2026-06-11/`.
 Update this file as state changes.

The prior x86 campaign's handover is `doc/handover/resharp-fuzz.md`
(2026-06-04,
 23 root causes).
 Read it for the engine semantics,
 the Lean oracle
pipeline,
 and the `repro` mode list;
 this file only adds the ARM64/SIMD layer
and the footguns specific to it.

## Mission and goal

Fuzz `ieviev/resharp` v0.6.12 (`3d4ddde`,
 tag `v0.6.12`) on an ARM Mac (M1)
because resharp has different SIMD code for ARM (NEON) vs x86 (AVX2).
 Match or
exceed the depth of the 2026-06-04 campaign (27+ bugs),
 maximal yield.
 Hard
goal:
 10 distinct root causes.
 The user wants maximum yield ("honeycomb").

## CRITICAL: do not trust the dumb agent (bad-glm-...)

A weaker agent ("glm") is running this same task in a podman container on the
Mac.
 Its output landed at `doc/audit/resharp-fuzz-2026-06-10/`,
 now renamed to
`doc/audit/bad-glm-resharp-fuzz-2026-06-10/` with a warning banner in its
README.
 The user explicitly said:
 do not trust anything that agent wrote.
Verified-by-reading defects in it:

- It blames compile-time blowups (`\P{L}`,
   `\p{L}`) on "the NEON SIMD path.
  "
  SIMD in resharp is ONLY in the match/scan prefilter (`simd/neon.rs`),
   never
  in derivative/minterm construction.
   A compile-time cost cannot be
  NEON-specific.
   Category error.
- It compares "7.25s on ARM64 vs 162ms on x86_64" and credits NEON,
   but those
  are two different physical machines,
   not two code paths on one machine.
- It conflates fuzzer harness option-bytes (`0x7b % 6 = 5`) with pattern
  content,
   and files "bugs" it admits "may be a parse artifact.
  "

Use it only as a (very weak) lead generator.
 Every finding in MY campaign must
rest on a reproduction I ran myself:
 a same-machine NEON-on vs NEON-off
divergence,
 a self-consistency violation,
 Lean,
 or the `regex` crate.
 The user
is letting glm run for completeness;
 ignore its conclusions.

## Environment

- Host:
   `ssh m1` (key auth works,
   BatchMode ok).
   Apple M1 (`arm64`,
   T8103),
  macOS 26.5.1 (build 25F80),
   8 cores,
   16 GiB RAM.
- MacData partition:
   `/Volumes/MacData` (477 GiB,
   ~340 GiB free).
   This is where
  the campaign must run (user directive).
- Campaign scaffold (already created,
   mostly empty):
  `/Volumes/MacData/resharp-fuzz-v0.6.12-arm64-20260610/` with `src/`
  (`resharp-pristine`,
   `resharp-nosimd` both at `3d4ddde`),
   `artifacts/`,
  `corpus/`,
   `logs/`,
   `oracles/`,
   `scripts/`,
   `tools/`,
   `build-context/`
  (`Containerfile`).
   The glm agent uses the podman container built from that
  Containerfile.
- Local pristine clone for fast reading (no `.git`):
  `/tmp/agent/resharp-v0612` (rsynced from the Mac scaffold).
   Use `rg`/Read here
  rather than ssh round-trips.
- Reference materials in `~/Downloads`:
   Lean formalization
  (`extended-regexes/`,
   the ground-truth oracle),
   papers
  (`3704837.pdf`,
   `lean formalization ...pdf`),
   dotnet impl
  (`resharp-dotnet/`,
   secondary only,
   NOT an arbiter),
   plus
  `ereq-derivatives`,
   `finiteness-derivatives`,
   `RLTL-derivatives`,
  `re-sharp-smt`.

## TOOLCHAIN FOOTGUN (open issue)

The M1 HOST has NO rust toolchain in PATH:
 `cargo`,
 `rustc`,
 `rustup`,
`cargo-fuzz` are all "command not found" over `ssh m1` (login shell is zsh).
Only the podman container (glm's) has nightly + cargo-fuzz.
 To run anything on
the host I must either install rustup on the host,
 or run my differential
inside a container/VM I control.
 Decide and record the choice here.
`sysctl hw.ncpu = 8`,
 `hw.memsize = 16 GiB`.

## The SIMD differential oracle (the headline method)

`has_simd()` (`resharp-engine/src/simd/mod.rs:7`) is hardcoded `true` on
aarch64 (x86 checks `is_x86_feature_detected!("avx2")`,
 wasm checks simd128).
So on the M1,
 the scalar fallback inside every dispatch site is NEVER taken in
a stock build.
 There are exactly FOUR `has_simd()` consult sites in the engine,
and at each one the SIMD branch is an ACCELERATION whose scalar fallback is "no
acceleration" (it does not reimplement the search,
 it just declines to build a
prefilter / skip table):

- `resharp-engine/src/prefix.rs:430` `build_fwd_prefix`:
   `!has_simd()` returns
  `Ok(None)` (no forward literal/rare-byte prefilter).
- `resharp-engine/src/prefix.rs:796` `select_prefix`:
   `!has_simd()` returns
  `Ok((None, None))` (no reverse Teddy / anchored prefix).
- `resharp-engine/src/ldfa.rs:458`:
   `if has_simd()` then
  `try_build_skip_simd` (a lazy-DFA skip/acceleration table).
   Built LAZILY
  during matching,
   so `has_simd()` must stay constant for a regex's whole
  lifetime (build AND match).
- `resharp-engine/src/bdfa.rs:80` `build_prefix`:
   `!has_simd()` returns
  `Ok(())` (bounded DFA gets no prefix).

Consequence:
 for the same (pattern,
 haystack,
 config),
 `is_match`,
 `find_all`,
and `find_anchored` MUST be byte-identical between `has_simd() == true` (NEON)
and `has_simd() == false` (scalar) on the same M1.
 Any divergence is an
unambiguous correctness bug in the accelerated NEON path (it skipped past a
real match start,
 landed on a wrong position,
 or mis-handled a chunk
boundary/tail).
 This is the gold oracle,
 and it needs no external reference.

SIMD prefilter bugs classically hide at NEON 16-byte chunk boundaries and tail
handling.
 Construct haystacks that place candidate/rare bytes at offsets 15,16,
17,
 31,32,33,
 63,64,65,
 at the very end,
 and at lengths that are not multiples
of 16.
 The NEON hotspots to read first:
 `neon_movemask` (`simd/neon.rs:6`,
the sign-bit gather),
 `find_fwd_neon`,
 `find_rev_neon`,
 `search_neon`,
 the
Teddy masks,
 and `read_partial_u64` (the sub-8-byte tail read in `simd/mod.rs`).

### Planned harness (in-process differential)

Patch the engine's `has_simd()` (in a path-dependency copy used only by the
repro crate) to consult a process-global `AtomicBool` (default:
 real
detection),
 so ONE binary can build a regex with SIMD off,
 fully run it,
 then
build the same regex with SIMD on and run it,
 and compare in-process.
 Sequential
runs keep `has_simd()` constant across each regex's build+match lifetime (needed
for the lazy `ldfa` skip).
 Emit `SIMDDIFF|op|cfg|pat|hay|simdoff=..|simdon=..`
on mismatch.
 Also build a libFuzzer `simd_diff` cargo-fuzz target with the same
assertion so coverage guidance hunts divergences on the host natively.

Keep the 06-04 self-consistency oracles too (BOUNDS,
 OVERLAP,
 INCONSIST,
find_anchored-vs-find_all,
 STREAM*),
 and the `regex`-crate ascii differential,
since v0.6.12 may still carry non-SIMD bugs.
 The nosimd src tree in the MacData
scaffold is currently IDENTICAL to pristine (`diff -rq` empty);
 it has NOT been
patched yet,
 so do not assume it disables SIMD.

## Reference is Lean, not dotnet (carried from 06-04)

The Lean formalization (`~/Downloads/extended-regexes`) is THE verified ground
truth.
 The dotnet engine is immature and is a secondary "another implementation
to look at,
" never an arbiter.
 Every filed bug must rest on Lean plus a
rust-internal inconsistency or the `regex` crate,
 never on dotnet alone.
 See the
06-04 handover for the full Lean pipeline (`re2lean.py`,
 chunked `lake env lean`
recipe,
 `adj_full.py`,
 the lookbehind-of-anchor unfaithfulness caveat).

## v0.6.12 vs the 06-04 baseline

v0.6.12 is heavily refactored from the 06-04 baseline.
 The engine is no longer a
single `engine.rs`;
 it is split into `scan.rs`,
 `fwd.rs`,
 `ldfa.rs`,
 `bdfa.rs`,
`fas.rs`,
 `accel.rs`,
 `minterms.rs`,
 `prefix.rs`,
 `dump.rs`,
 `stream.rs`,
 plus
`simd/{mod,neon,wasm,byte_freq}.rs`.
 Old `engine.rs:NNN` line references from
the 06-04 docs do NOT map;
 re-locate every site.
 Several 06-04 bugs were
addressed by making the PARSER reject the pattern (e.g. lookaround/`\b`/`^`/`$`
inside a complement now "unsupported pattern"),
 which avoids the defect rather
than fixing the engine;
 note that distinction when re-testing.

## Conventions and process

- Repo prose rules:
   no em-dashes/en-dashes as em-dashes,
   sentence-case ATX
  headings (max 4 levels),
   fenced code with language tags,
   lines under 120,
   no
  tables,
   no emojis,
   `-` bullets.
   Each bug file is self-contained:
   reproducer
  (rust snippet plus exact command),
   observed vs expected,
   affected configs,
  source location,
   relationship to other bugs.
- Commit eagerly,
   `docs(resharp-fuzz): ...`,
   EXPLICIT scoped pathspecs.
   The
  working tree carries concurrent external "unbash" changes in `package/` and
  `AGENTS.md`;
   never stage those.
   `cli-git` rejects `-A`/`.`/pathspec-less
  commits by design.
- Current scratch lives under `~/temp/agent`
  (`mkdir --parents "${HOME}/temp/agent"; chmod 700 "${HOME}/temp/agent"` if missing).
   The user cleans up;
   do not delete audit artifacts.

## Toolchain resolved (2026-06-11)

The M1 host now has a working PATH:
 `~/.zshenv` was created (it is sourced by
non-interactive ssh;
 `~/.zprofile` is not) prepending `~/.cargo/bin`,
 the
`nightly-aarch64-apple-darwin` toolchain bin,
 `/opt/homebrew/bin`,
 and the mise
shims.
 `cargo`,
 `rustc`,
 and `cargo-fuzz` (installed via `cargo install`) are
reachable over `ssh m1`.
 macOS has no GNU `timeout` (use chunking,
 not a timeout
wrapper).
 The x86 box (where this repo lives) is `x86_64` (AVX2) with its own
nightly + cargo-fuzz + cargo-miri;
 the SIMD differential reproduces on AVX2 too,
so x86 is a valid second SIMD lane and runs the correctness/fuzz lanes.

## Findings (2026-06-11 01:30): 8 root causes, committed

All in `doc/audit/resharp-fuzz-2026-06-11/` (README + one file per bug +
`verification-2026-06-04.md` + `code-quality.md` + `test-coverage.md`).
 The
developer's "all 27 fixed,
 plus more" claim is FALSE in part:
 BUG-1,
 BUG-20,
BUG-8,
 BUG-3 are all still live on new triggers.

- arm-bug-01 (SIMD soundness,
   fix-verified):
   `fwd_lb_prefix_impl` `fwd.rs:123`
  drops the offset-1 match after a leading zero-width match.
   `^$` on `"\n\n"` =>
  `[0:0,2:2]` SIMD-on,
   `[0:0,1:1,2:2]` SIMD-off.
   Identical NEON and AVX2 (defect
  is in the arch-independent driver).
   Found by `simd_diff` fuzzer + the on/off
  differential.
   Fix `{1}`->`{0}` verified.
- bug-02 (soundness):
   `find_anchored` phantom on leading zero-width assertion
  (`(?<=a)` on `"b"`,
   `\BU` on `"U"`):
   `fan=Some` while `im=false`.
   122 triggers.
  BUG-20 partial-fix-still-live.
- bug-03 (soundness):
   `stream` phantom/mislocated zero-width (`(?=c)`->1:1,
  `\b`->shifted,
   `(?!\A)`->0:0).
- bug-04 (crash):
   reentrant union rewrite panic `algebra/lib.rs:2724` LIVE,
  minimal `(.*.+)*.+`,
   ~165 fuzzer triggers.
   BUG-1 narrowed not killed.
- bug-05 (crash):
   reachable `debug_assert!(false,"this path should be
  eliminated")` `engine/lib.rs:1824`,
   pattern `_*$` (`rev_trivial`).
- bug-06 (perf):
   full-mode `\w{n}` compile ~0.14s/repeat (`\w{24}`=3.3s).
- bug-07 (soundness):
   default vs hardened `find_all` differ,
   `~(\A|\n+){2}` on
  `"\n\n"`.
   BUG-8 family live.
- bug-08 (soundness):
   `is_match` vs `find_all`,
   `[0-9]{2}~(\z{1,3}|^{2}\W{0})+`
  flags cfg on `"00"` (`im=false`,
   `fa=[0:2]`).
   BUG-3 family live.

The single SIMD root cause is arm-bug-01;
 the rest are arch-independent.
 The
SIMD differential harness is solid (99.2% prefilter-active coverage,
 zero false
positives over 58k cases).
 Most 06-04 bugs ARE genuinely fixed (see verification
doc);
 do not re-file those.

## Harness (rebuilt, both machines)

- `tools/resharp-instr`:
   v0.6.12 engine + `has_simd()` atomic override
  (`instr_set_override` 0/1/2) + prefilter-built counters (`instr_counters`).
  ONLY change vs stock.
   `simd` stays `pub(crate)`;
   hooks re-exported at crate
  root (avoids `deny(missing_docs)`).
- `tools/repro-simd` (binary `repro`):
   modes `--pair`/`--probe`/`--show`/
  `--reuse`/`--compile`/`--benchrep`/`--benchcyc`/`--patbatch`/`--oraclebatch`/
  `--batch`.
   The verification + differential + oracle workhorse.
- `tools/resharp-instr/fuzz`:
   `simd_diff` libFuzzer target (on/off differential
  with `catch_unwind`,
   `panic=unwind`).
   Build with `--target` default on mac.
- Local x86 mirror:
   `/tmp/agent/{resharp-instr,repro-simd,resharp-v0612}`.
  `resharp-fixtest` is the engine copy with the arm-bug-01 fix applied (for the
  fix verification).
   Generators:
   `gen_simd_corpus.py`,
   `gen_adversarial.py`,
  `gen_big.py`;
   `verify_0604.py`.
- KNOWN HARNESS GAP:
   `repro --oraclebatch` swallows MATCH-TIME panics into
  `fa=Err` (only build-time panics print `PANIC|`).
   So bug-05-style match panics
  do not show as `PANIC` in oracle output;
   the libFuzzer `match_invariants` target
  is the one that surfaced it.

## Running jobs (overnight, both machines)

- x86:
   `compile`,
   `match_invariants`,
   `diff_regex` in-tree fuzzers,
   `-fork=3
  -ignore_crashes=1 -max_total_time=14400` (logs `/tmp/agent/fz2_*.log`,
  artifacts under `resharp-v0612/fuzz/artifacts/<tgt>/`).
   diff_regex crashes are
  dominated by bug-04 (BUG-1);
   look for NON-`lib.rs:2724` crashes for new bugs.
- M1:
   `simd_diff` `-fork=3 -ignore_crashes=1` (logs `tools/simd_diff_fuzz2.log`,
  artifacts `tools/resharp-instr/fuzz/artifacts/simd_diff/`).
   All crashes so far
  are arm-bug-01;
   a non-`^`-prefixed divergence would be a new SIMD root cause.

## Campaign log (newest first; update >= every 10 min while running)

CLOCK NOTE:
 the `08:40`-`09:30` wall-clock timestamps below were UNVERIFIED
guesses (I did not read the host clock when writing them).
 The real verified
clock at the next tick was `2026-06-11 04:03 EDT` via `date`.
 So those entries
happened BEFORE 04:03,
 not after;
 ignore their absolute times,
 trust their
order.
 The substantive error this caused is corrected in the 04:03 entry.

- 05:06 (clock-verified via `date`):
  OVERNIGHT FUZZERS COMPLETE,
   FINAL HARVEST,
  NO NEW CRASH SITE.
   The three x86 cargo-fuzz targets reached their full 4h budget
  (`time: 14363s` / `14325s` / `14367s` against `max_total_time=14400`) and are
  wrapping up.
   Final artifact counts and triage:
  - diff_regex:
     1671 artifacts,
     newest 05:06:09;
     newest 8 triaged = ALL
    `resharp-algebra/src/lib.rs:2724` (bug-04).
  - diff_regex2:
     59 artifacts (newest 01:21,
     stopped finding new long ago),
     all
    bug-04 (triaged last tick).
  - match_invariants:
     5 artifacts (was 3 at 04:03;
     +2 new at 04:24 and 04:58).
    ALL 5 triaged = `resharp-engine/src/lib.rs:1824` (bug-05);
     every input is the
    `_*$` family (`5f2a24...`).
     The two new ones are NOT a new site.
  - compile:
     0 artifacts.
  - M1 simd_diff:
     7 artifacts,
     all arm-bug-01 (finished earlier).
  NET:
     across the entire overnight run on BOTH architectures,
     the fuzzers
  reproduced ONLY bug-04 / bug-05 (x86) and arm-bug-01 (ARM).
     THREE crash sites
  exist campaign-wide:
     bug-04,
     bug-05 (fuzzer-found),
     bug-11 (Lean-found;
     the
  libFuzzer arbitrary encoding never reached the ldfa shape).
     The overnight phase
  is genuinely DONE this time (clock-verified,
     fuzzers at budget);
     no further
  harvest will produce anything new.
     Campaign findings are final:
     13 root causes,
  two-tiered,
     all ARM-confirmed,
     all documented under
  `doc/audit/resharp-fuzz-2026-06-11/`.

- 04:03 (clock-verified):
  HARVESTED FOCUS LEAN ROUND + ARM-CONFIRMED THE
  SECONDARY TIER + CORRECTED A TIMING ERROR.
  (1) CORRECTION:
   the `09:30` entry's claim that the x86 fuzzers were "CONVERGED
  / past their 4h budget / newest crash 5h+ stale / findings final" was WRONG.
   I
  never read the host clock and assumed it was ~09:30.
   Real time is 04:03;
   the
  x86 fuzzers started ~01:07,
   elapsed 02:56,
   are STILL RUNNING within their 4h
  `max_total_time=14400` budget (due ~05:07),
   and the newest diff_regex crash was
  04:03:38 (seconds before the check,
   not stale).
   The substantive conclusion that
  survives:
   every crash found so far is still bug-04 / bug-05 (newest 6
  diff_regex all algebra:
  2724;
   counts diff_regex 1266,
   diff_regex2 59,
  match_invariants 3,
   compile 0).
   The M1 simd_diff "finished clean,
   7 artifacts,
  all arm-bug-01" part stands (it did exit).
   But x86 is NOT done;
   let it run to
  ~05:07.
  (2) FOCUS LEAN HARVEST (focus.
  out.
  txt vs focus.
  rust.
  txt over focus.
  tsv,
   3199
  cases):
   agree=2067,
   rust_reject=1127,
   RUST_PANIC=0 (no new crash site),
  SPANDIFF=1 (trust1),
   RUST_PHANTOM=2 (1 trust0 + 1 trust1),
   RUST_MISS=2 (both
  trust1).
   The lone trust0 disagreement is R292
  `((\W|((?!c)))&((_&[acd])&a))` on `'a'`:
   lean=none,
   rust=0:1.
   That is bug-13
  form B (a zero-width lookaround-in-union side `&`-intersected with a consuming
  side;
   rust leaks the consuming span).
   NOT a new root cause.
   The focused round
  confirms nothing beyond the documented 13.
  (3) ARM CONFIRMATION of the secondary tier (parallels the bug-04/05 ARM run).
  armprobe on the M1:
   bug-11 `((?!a)|b)&(~((c)))`/"abca" -> PANIC@ `ldfa.rs`:
  906
  (same site as x86);
   bug-12 `((?!b)|ba)&(aa)?`/"ab" -> find_all=[(2,2)] (Lean
  0:0);
   bug-13 `a?&(?=a)?`/"ab" -> find_all=[(0,1),
  (1,1),
  (2,2)] (Lean 0:0);
   R292
  -> find_all=[(0,1)] (Lean none).
   All byte-identical to x86.
   Added ARM notes to
  bug-11/12/13 files.
   Now the WHOLE campaign (load-bearing + secondary) is
  ARM-demonstrated,
   not inferred.

- 09:20:
  DONE.
   Two closing consistency passes from the stronger-reviewer
  done-check,
   both committed:
  (1) Tier-split propagation to the supporting docs (2457bdba).
   verification-
  `2026-06-04.md` and `code-quality.md` predated the dotnet tier split and
  contradicted the narrowed README headline (they listed bug-07/08/10 as flatly
  "live" with no out-of-subset caveat,
   handing ieviev the rebuttal "those fire on
  patterns my engine rejects").
   Each still-live family is now tagged IN-SUBSET
  (bug-04,
   bug-02:
   dotnet accepts,
   refute all-fixed alone) vs OUT-OF-SUBSET
  (bug-07/08/10:
   dotnet rejects at compile,
   rust accepts and miscomputes;
  compile-rejection noted as the alternative remedy).
  (2) Airtight ARM confirmation for the two in-subset crashes (a1d7ff7e).
   Re-ran
  `armprobe "(.*.+)*.+" "aaa"` and `armprobe "_*(?!_)" "aa"` on the M1 (aarch64,
  debug-assertions on):
   both panic at the SAME sites as x86 (algebra:
  2724,
  engine:
  1824).
   These are arch-independent algebra/reverse-pass faults (not SIMD),
  so the campaign's "ARM-confirmed" claim is now demonstrated,
   not inferred,
   for
  the load-bearing crashes.
   The done-check confirmed no further passes needed:
  ship it,
   do not reopen the tier analysis or hunt a 14th bug.

- 08:55:
  CONSOLIDATION POINT.
   bug-06/09 perf confirmed rust-specific:
   dotnet
  compiles `\w{24}` in 6ms and the bug-09 `.n...n.n` unit in 21ms vs rust's ~3.3s
  / ~40-71s (1000x+);
   committed (bc480603).
   Final overnight fuzzer harvest:
   NO new
  crash site;
  diff_regex 1184 crashes ALL bug-04 (algebra:
  2724),
   match_invariants
  3 ALL bug-05 (`lib.rs`:
  1824),
   compile 0.
   THREE crash sites total across the whole
  campaign:
   bug-04,
   bug-05 (both in-subset,
   dotnet-confirmed rust-specific),
  bug-11 (`ldfa.rs`,
   out-of-subset,
   Lean-found).
   The bug-11 corpus seeding did not
  yield a fuzzer-found ldfa crash (the libFuzzer arbitrary encoding doesn't easily
  reach the precise shape;
   the Lean AST generator does).
   CAMPAIGN STATE:
   13
  findings,
   honestly two-tiered;
   load-bearing core = bug-04/05 crashes + bug-02/03
  (dotnet+self-confirmed) + bug-06/09 perf + arm-bug-01 SIMD;
   secondary =
  bug-07/08/10 (self-inconsistent,
   out-of-subset pattern) + bug-11 (crash,
   out-of-
  subset) + bug-12/13 (Lean-corroborated,
   out-of-subset).
   Full doc set committed:
  README (tiered),
   13 bug files (+banners),
   verification-2026-06-04,
   pristine-
  confirmation,
   dotnet-adjudication,
   code-quality,
   test-coverage.
   Active mining
  DONE per the stronger-reviewer steer.
   Overnight fuzzers left running on both
  machines for completeness;
   harvest periodically (only bug-04/05 expected).

- 08:40:
  MAJOR HONEST REFRAME via the dotnet reference (the integrity moment).
  Installed dotnet on Mac (`brew install dotnet`),
   built `resharp-dotnet`
  (ieviev's OTHER RE# impl),
   wrote an fsi adjudicator,
   ran EVERY bug minimal.
   The
  dotnet reference REJECTS at compile several construct classes the rust crate
  ACCEPTS:
   "lookarounds inside union",
   "nested lookarounds",
   "anchors inside
  complement",
   `\b`/`\B` away from word chars.
   Decisive guard check:
   rust HAS the
  same machinery (`ensure_supported_rec`/`Compatibility::LookaroundUnion`,
  `lib.rs`:
  762/772) but DELIBERATELY accepts a SUPERSET ("to unlock some patterns
  outside RE# fragment",
   `lib.rs`:
  803);
  so the Tier-2 defect is rust failing on
  patterns its OWN guard blessed,
   not just diverging from a stricter peer.
  TWO TIERS (`dotnet-adjudication.md`):
   * LOAD-BEARING (in-subset / self-evident):
      bug-04 & bug-05 CRASHES (dotnet
     matches the patterns cleanly -> `[(0,3)]` / `[(0,2),(2,2)]`,
      rust panics:
     CLEAN rust-specific crashes,
      lead with these);
      bug-02 `(?<=a)` (dotnet im=
     false [] confirms rust find_anchored=Some(0:0) is a phantom);
      bug-03 `(?=c)`
     (dotnet find_all=[(0,0)] confirms rust stream=1:1 is the wrong side);
     bug-06/09 perf;
      arm-bug-01 SIMD (rust-internal).
   * SECONDARY (reference rejects pattern;
      rust accepts+fails):
      bug-11 crash
     (out-of-subset but a panic is a defect regardless);
      bug-07/08/10 rust-
     INTERNAL self-inconsistencies (default-vs-hardened,
      im-vs-find_all,
      find_anch-
     vs-find_all:
     demonstrable w/o any reference,
      only caveat is rust should
     reject the pattern);
      bug-12/13 Lean-corroborated wrong spans (weakest:
     stopped treating Lean as sole authority for these out-of-subset patterns).
  Headline restructured around tiers (NOT "13 distinct");
      per-file banners added;
  committed (19c2d76a + 38615332).
      Advisor steer owned:
      the Lean lane's yield
  (bug-11/12/13) landed mostly Tier-2;
      the strong core is the fuzzer crashes +
  dotnet-confirmable consistency bugs.
      The complement-of-anchor "suspects"
  (a(~(\z))) are RESOLVED:
      dotnet rejects them too -> rust-accepts-mishandles,
  same secondary class,
      NOT a clean bug-14.
      STOPPING active mining;
      consolidating.

- 08:05:
  trust0 region EXHAUSTED + complement-of-anchor frontier opened.
  Refined `trust()` to context-aware (anchor inside complement/lookbehind ->
  trust1;
   bare `\A`/`\z` -> trust0;
   lookbehind-of-lookaround -> trust1),
   then
  re-diffed ALL harvested rounds (no new Mac evals;
  reused saved outputs).
   Every
  trust0 disagreement reduces to a KNOWN bug:
   R1612=bug-11,
   R2280=bug-12,
  R48/R292/R2739=bug-13 (R2739 `((c[ab])|(?=a))?&.` = form B again;
  nullable LHS
  matching only zero-width/width-2,
   intersected with width-1 `.`,
   leaks `.`'s
  width),
   R2513/R833/R2864=bug-05 (wider trigger family incl `\z`-in-complement).
  So the FAITHFUL region holds no 14th root cause;
   bug-13 (intersection with a
  zero-width operand) is the dominant systemic issue,
   recurring under many forms.
  NEW FRONTIER (trust1,
   unadjudicated):
   complement-of-anchor over-matches;
  `a(~(\z))` -> rust `[(0,1)]`,
   `b(~((c|\z)))` -> rust `1:2`,
   where Lean says none.
  These look like the live 06-04 BUG-4 family (`~(_*$)` sentinel leak,
   which is
  now parser-rejected but survives as `~(\z)`),
   BUT complement-of-anchor is a
  translation-faithfulness-suspect zone,
   so they need the dotnet engine as
  tie-breaker before being claimed.
   Documented as SUSPECTS in README,
   not
  findings.
   NEXT:
   attempt dotnet adjudication on the Mac (authorized) to resolve
  the complement-of-anchor suspects -> possible bug-14;
   timeboxed.

- 07:40:
  FOCUSED round + ARM parity.
   Focused Lean round (FOCUS flag,
  inter/neg/lookaround heavy,
   single-file eval = 466s/3199 cases,
   no hang,
   no
  per-chunk import reload) found 1 trust0:
   R292 `(\W|(?!c))&a` on "a" -> rust
  `[(0,1)]` phantom,
   Lean `none`.
   SAME mechanism as bug-13 (zero-width lookahead
  on one side of `&` paired with consuming on the other leaks width),
   but with the
  consuming side NOT nullable (lookahead is an alternation branch on the opposite
  side).
   Folded into bug-13 as "form B";
   corrected bug-13's too-narrow "both
  operands nullable" claim.
   NOT a new count (like R2513->bug-05).
   Committed
  (950be7fb).
   ARM PARITY CLOSED:
   rebuilt the pristine engine from source on the M1
  (aarch64) and ran bug-11/12/13 minimals -> byte-IDENTICAL to x86 (bug-11 panic
  `ldfa.rs`:
  906,
   bug-12 `[(2,2)]`,
   bug-13 `[(0,1),...]`).
   Confirms they're in the
  arch-independent `ldfa.rs` driver;
   arm-bug-01 stays the ONLY arch-specific
  (SIMD) finding.
   README parity note added.
   Campaign:
   13 root causes,
   all
  stock-confirmed,
   bug-11/12/13 also ARM-confirmed.

- 07:15:
  13TH ROOT CAUSE (bug-13),
   seed-4004 R48.
   `a?&(?=a)?` -> find_all
  `[(0,1),(1,1),(2,2)]` and find_anchored `Some((0,1))`,
   but the intersection is
  zero-width-only (`a?`'s width-1 span cannot equal the lookahead's zero-width
  span),
   so Lean leftmost = `0:0`.
   The optional lookahead,
   when SATISFIED at the
  position,
   fails to constrain the match width and the consuming `a`'s width LEAKS
  into the span -> match too LONG (opposite of bug-11/12's drops;
   an END
  over-extension in the forward match-extension path,
   not null-start collection).
  Needs both `&` operands nullable + lookahead satisfied (`a?&(?=c)?` false-look
  is correct;
   `a&(?=a)?` non-nullable-consuming is correctly empty).
   Silent in
  debug AND release,
   hits find_anchored too,
   is_match correct.
   Same width-leak
  theme as 06-04 BUG-13 (`(?=(?=c)c{1,3})`,
   was fixed on its trigger) but a
  distinct LIVE intersection trigger.
   Lean-confirmed,
   pristine,
   committed
  (a2f1f02b),
   README->13.
   Seed-4004 also re-derived bug-05 (`((_)*)+(?!b)`).
  LEAN LANE TOTAL:
   3 new root causes (bug-11/12/13),
   all intersection+lookaround;
  this subsystem is clearly fragile.
   NEXT:
   focused round (FOCUS flag biases
  inter/neg/lookaround) with single-file eval (the per-chunk import reload was the
  ~13min/seed bottleneck;
   one file = one import load).

- 06:50:
  bug-11 REFRAMED (forward-pass fault) + trust() gap fixed.
   Stronger
  reviewer caught a fresh unverified claim:
   I wrote bug-11 "over-proposes" but the
  `nulls` print only shows reverse PROPOSED the start,
   not that proposing was the
  error.
   Lean-confirmed `((?!b)|ba)&(aa)*` gives `0:0` (offset 0 IS a real match),
  so reverse proposed CORRECTLY and the FORWARD scan is at fault (returns NO_MATCH
  for a legit start = exactly the engine's own assert message).
   So:
   bug-11 =
  FORWARD-pass fault,
   bug-12 = REVERSE-pass fault.
   Different passes -> cleaner,
  harder-to-wave-off distinctness.
   Reworded all three files,
   dropped
  "over-proposes" (912056c5).
   TRUST GAP:
   seed-2002 R3641
  `((?<!(~(((?<!\d))))))` came up trust0 but is lookbehind-of-(complement-of-
  lookbehind) = lookbehind-of-lookaround = the 06-04 documented-unfaithful shape;
  my trust() only flagged anchors,
   not nested lookbehind.
   Fixed trust() to flag
  lookbehind/neglookbehind whose body contains any lookaround.
   R3641 reclassifies
  trust1 -> translator artifact,
   NOT a bug.
   bug-11/bug-12 are lookAHEAD-only (no
  lookbehind) so the gap never touched them.
   Multi-seed harvest:
   seed 1001 =
  bug-12 + bug-05-retrigger;
   seed 2002 = only R3641 (artifact);
   seed 3003 = no
  trust0 diffs;
   seed 4004 running.
   Lean lane total:
   2 new root causes (bug-11,
  bug-12).

- 06:25:
  12TH ROOT CAUSE (bug-12),
   Lean multi-seed round (seed 1001).
   R2280
  (trust0) = `find_all` SILENTLY drops the leftmost match,
   NO panic in debug or
  release.
   Minimal `((?!b)|ba)&(aa)?`:
   `"ab"` -> rust `[(2,2)]` (Lean leftmost
  `0:0`),
   `"abab"` -> `[(4,4)]` (Lean `0:0`).
   is_match stays correct.
   Stronger-
  reviewer flagged the distinct-vs-fold-into-bug-11 call (one-token `?`->`*`
  toggles silent-drop vs bug-11 panic,
   so suspiciously adjacent) and prescribed
  the decider:
   PRINT the `nulls` slice `scan_fwd_all` receives.
   Did it (one-line
  instrument in the engine copy,
   then reverted):
   bug-12 `(aa)?` -> `nulls=[4]` on
  abab (reverse NEVER proposes 0/2 = UNDER-collect -> silent drop);
   bug-11 `(aa)*`
  -> `nulls=[4,2,0]` (0 proposed,
   forward REJECTS -> panic).
   Two verified-distinct
  reverse-pass faults (under-collect vs over-propose),
   not one seen twice ->
  counted distinct (12),
   framed as siblings in the find_all reverse-null
  subsystem that may share a fix.
   Replaced the earlier INFERRED "under-collects"
  with this evidence.
   R2513 (`_*(?!_)`) was bug-05 on a new trigger (`lib.rs`:
  1824),
  not new.
   Committed `bug-12.md` + README->12 (415fe6d6) + nullsprobe tool.
   Lean
  lane now has TWO root causes (bug-11,
   bug-12),
   both find_all reverse-null faults
  the internal oracles structurally cannot see.
   Multi-seed round still finishing
  (seeds 3003/4004);
   will harvest remaining trust0 diffs.

- 05:50:
  bug-11 HARDENED + fuzzer harvest.
   CORRECTION:
   `is_match` ALSO panics
  (`ldfa.rs`:
  906 on `"abca"`,
   :
  833 on `"ca"`):
  it routes through `scan_fwd_all`
  for this intersection class,
   so the crash is NOT find_all-only (broader DoS).
  Release `is_match` stays correct (a match still exists);
   only release `find_all`
  has the soundness drop.
   Fixed the writeup + README,
   committed (193476df).
   Family
  shape sharpened:
   `(?!x)` as one branch of an ALTERNATION,
   `&` complement `~(y)`
  with `y` in haystack and `y != x`;
   drop the alternation / use positive-or-
  optional branch / make `y==x` or absent -> no fire.
   x86 OVERNIGHT FUZZER
  HARVEST:
   diff_regex 711 crashes ALL bug-04 (algebra:
  2724);
   match_invariants 1
  crash = bug-05 (`lib.rs`:
  1824);
   compile 0 crashes (52 slow/timeout = bug-06/09
  perf).
   NO new crash site from random fuzzing;
  `ldfa.rs`/bug-11 was never sampled
  (the random corpora don't produce the precise intersection+complement+neg-
  lookahead shape;
   the Lean AST generator does by construction).
   Seeded the
  match_invariants corpus with 3 bug-11-family inputs so the mutator can explore
  siblings.
   Multi-seed deeper Lean round (seeds 1001/2002/3003/4004,
   depth<=5,
  ~6000 each) RUNNING on the Mac (bkrl3ybcf).

- 05:25:
  11TH ROOT CAUSE (bug-11),
   found by the Lean lane on its FIRST run,
   a
  THIRD distinct crash site.
   1954-case differential:
   1392 agree,
   559 rust-reject,
  3 disagreements.
   Two were nested lookbehind-of-`\A` (trust1,
   the documented-
  unfaithful translator shape -> discarded).
   The third (R1612,
   trust0,
  anchor-free,
   faithful) is a rust crash:
   `find_all`'s reverse pass proposes a
  null match start the forward pass rejects,
   tripping `debug_assert_ne!(NO_MATCH,
  l_max_end, "find_all: forward scan found no end for reverse-proposed start")` at
  `ldfa.rs:833/878/906`.
   Minimal `((?!a)|b)&(~((c)))` (zero-width-nullable
  alternation & complement;
   all three ingredients required;
  `(?!a)` alone or
  `()`/`(?=a)` in its place do not fire).
   On `"abca"` -> panic at 906;
   on `"ca"`
  -> panic at 833.
   RELEASE (debug-assertions off,
   what ships):
   no panic but
  `find_all` DROPS the reverse-proposed match;
  `"ca"` returns `[(2,2)]` missing
  the leftmost `(0,0)` (Lean ground truth `0:0`),
   `"c"` returns `[(1,1)]` missing
  `(0,0)`.
   So:
   crash in debug/test builds,
   find_all soundness (dropped leftmost
  match) in release.
   Confirmed on the STOCK crate (probe + relprobe link
  `resharp-v0612`).
   Distinct from bug-04 (algebra:
  2724),
   bug-05 (`lib.rs`:
  1824),
  arm-bug-01 (`fwd.rs` prefilter).
   Written up (bug-11-...md),
   README updated to 11
  root causes / 3 crash sites,
   committed (68b65494).
   The internal oracles cannot
  reach this:
   `find_all`-only path,
   release result self-consistent but
  incomplete,
   so only the position reference exposes the soundness half (the
  crash half is engine-internal).
   NEXT:
   more Lean rounds (deeper ASTs,
   more
  seeds) to harvest further trust0 panics/span-diffs;
   consider a full-match-set
  Lean comparator (spAll) for silent completeness drops that do not hit the
  assert.

- 04:55:
  LEAN POSITION LANE STOOD UP on the Mac (the major unexplored lane).
  Installed elan via `brew install elan-init` (4.2.3;
   also pulled coreutils,
   so
  `gtimeout` now exists on the Mac).
   Lean toolchain v4.24.0-rc1 fetched.
   The
  extended-regexes formalization requires mathlib:
   `lake exe cache get` pulled
  6892 prebuilt mathlib oleans (5.4G) in minutes,
   then `lake build
  Regex.MatchingAlgorithm` built the Regex lib (NOT `lake build Regex` -- there
  is no `Regex.lean` root aggregator,
   that errors).
   `lanval.lean` has a
  pre-existing invalid-escape (`\A` in a DISPLAY string) and does not compile,
  but that never blocked the real runs.
   Reconstructed the 06-04 pipeline
  AST-FIRST to kill parser-precedence risk:
   `/tmp/agent/lean/gen_lean_ast.py`
  builds random RE ASTs and serializes each to BOTH a fully-parenthesized RE#
  string (rust) and a Lean `RE (BA Char)` term (Lean),
   so both engines get the
  same structure by construction.
   Predicate/anchor encodings fixed from the Lean
  source (`.`=`(.atom '\n')ᶜ`,
   `_`=⊤,
   `\w`/`\d` as named `def wc/dc`,
   `\b`/`\B`
  as `def bnd/nbnd`,
   `^`=`\A|(?<=\n)`,
   `$`=`\z|(?=\n)`).
   Each case tagged trust0
  (no anchor/\b,
   translator faithful) or trust1 (anchor/\b -> needs dotnet
  adjudication,
   the documented-unfaithful shapes).
   `llmatch` = leftmost-longest
  first match = resharp default `find_all(w)[0]`.
   SMOKE TEST PASSED:
   12-case file
  evaluated in Lean and 10/11 agreed with rust find_all first span;
   the 1 diff
  was a rust `builderr` (pattern rejected,
   not a real disagreement).
   Rust side
  = `/tmp/agent/lean/leanrust` (default cfg,
   prints first span,
   panic/err tokens
  distinct).
   Diff = `/tmp/agent/lean/diff_lean.py` (buckets SPANDIFF /
  RUST_PHANTOM / RUST_MISS / RUST_PANIC,
   split by trust).
   NOW RUNNING:
   1954-case
  corpus (seed 20260611),
   Lean eval in 7 gtimeout-bounded chunks on the Mac,
  rust side done (1954 lines).
   This catches the self-consistent-but-WRONG-span
  class the internal oracles structurally cannot see.

- 04:20:
  HARDENING the headline (stronger-reviewer checkpoint flagged that the
  five oracle-only bugs had only ever run through the INSTRUMENTED engine,
   so the
  "instrumentation is neutral at override=0" assumption was untested;
  the exact
  assumption this campaign rejects).
   Closed it:
   built `tools/pristine-repro`
  (depends on the STOCK `resharp-v0612/resharp-engine`,
   no override/counters) and
  re-ran the documented minimal reproducers for bug-02/03/07/08/10.
   ALL FIVE
  reproduce byte-identical on the unmodified crate.
   Then `tools/pristine-repro-
  fixtest` (same five against `resharp-fixtest` = v0.6.12 + only the arm-bug-01
  `fwd.rs:123 {1}->{0}` fix):
   all five STILL fire -> the arm-bug-01 fix is
  orthogonal,
   proving the five are separately fixable,
   not one defect in five
  drivers.
   Wrote `pristine-confirmation.md`,
   updated README headline,
   committed
  (8fb0c80b).
   Stream-variant oracle (stream_first vs stream().
  first,
   stream_ends
  vs stream() ends) over the 79k corpus:
   CLEAN NULL (0 STREAMFIRSTDIFF,
   0
  STREAMENDSDIFF),
   verified non-vacuous (stream_first/stream_ends return real
  values that mirror stream() on `a`/`\b`/`(?=c)`).
   The variant methods faithfully
  derive from stream(),
   so no 11th bug there (stream() itself is still wrong on
  zero-width = bug-03,
   but the variants don't add a defect).
   Multibyte/invalid-
  UTF-8 full-mode probe (.
  ,
  .
  +,
  .
  {2},
  \w,
  [^a],
  \b,
  ~(a) over é/中/😀/lone-cont/overlong/
  truncated):
   CLEAN NULL (no panic,
   no BOUNDS).
   NEXT:
   Lean position-level lane on
  the MAC (installs authorized there;
   the curl|sh elan install was denied only on
  THIS x86 box):
  the one major unexplored lane,
   catches self-consistent-but-
  wrong-span correctness bugs the oracles structurally cannot.

- 03:25:
  10TH ROOT CAUSE found and committed.
   The FANSPANDIFF oracle over the
  79k combined corpus fired on 30 distinct patterns:
   bug-10,
   find_anchored
  returns a strictly SHORTER span than find_all's longest at offset 0
  (`~(.{1,3}\z){2,4}` on `"ab"` -> find_all [0:2,2:2] but find_anchored
  Some(0:1)),
   config-independent,
   the 06-04 BUG-13/14 family live on
  complement-with-end-anchor patterns.
   Also surfaced HARDDIFF_IM (default
  is_match=false vs hardened true,
   `1?a~(~((1?){2,}\z+){2}){2}` on `"a"`):
  the
  existence-level face of bug-07,
   folded in (stronger witness,
   not a new root
  cause).
   GOAL MET:
   10 distinct root causes (6 soundness,
   2 crash,
   2 perf),
   all
  reproduced,
   source-located,
   and committed with the 06-04 verification,
  code-quality,
   and test-coverage docs.
   Oracle kinds are now exhaustively
  catalogued (FANDIFF/FANINCONSIST=bug-02,
   FANSPANDIFF=bug-10,
   STREAMPHANTOM/
  STREAMINCONSIST=bug-03,
   HARDDIFF_FA/IM=bug-07,
   INCONSIST=bug-08,
   PANIC=bug-04);
  no uncovered violation type remains in the 79k corpus.
   Continuing:
   stream
  variants (stream_first/stream_ends untested),
   overnight fuzzers still running.

- 03:10:
  FOOTGUN:
   the `repro-fixtest` binary (the arm-bug-01 fix copy) has NO
  per-pattern timeout,
   and its `--patbatch < adv_pats.hex` regression run hung 50
  min on a pathological pattern,
   leaving a stuck repro proc that blocked every
  `until [ repro==0 ]` waiter chain.
   Fixed by killing PIDs 699417/699687 and the
  blocked waiters (exit 144).
   LESSON:
   always wrap `repro --patbatch`/`--oraclebatch`
  with `timeout` (mac has none;
   chunk + GNU `timeout` on x86),
   and never leave a
  no-timeout batch backgrounded.
   Relaunched the FANSPANDIFF oracle cleanly over
  the 79,419-pattern combined corpus (big + big2),
   16-way,
   `timeout 600`/chunk.
  FANSPANDIFF = find_anchored-vs-find_all SPAN disagreement at offset 0 (the
  06-04 bug-13/14 width-leak/gate-drop family);
   if it fires on a fresh trigger it
  is a clean 10th root cause.
   Awaiting results.

- 02:50:
  big2 oracle reconfirms the same families (FANDIFF/FANINCONSIST =
  bug-02,
   PANIC = bug-04).
   STREAMINCONSIST (stream empty while is_match true,
   the
  06-04 BUG-9 shape) is only 3 distinct patterns,
   all the complex
  `((?<=b+){2}&...)` shape that ALSO triggers bug-02;
   folded into bug-03 (same
  stream-zero-width defect),
   not a new root cause.
   Pinned bug-02's exact source
  (find_anchored `lib.rs:1901-1908`:
   the v0.6.12 leading-lookbehind guard is
  gated on `has_lb && !rev_trivial && !always_nullable`,
   nullable lookbehinds
  slip to the context-free `scan_fwd_slow(0)` fallback).
   Added a FANSPANDIFF
  oracle (find_anchored vs find_all SPAN at offset 0,
   the 06-04 bug-13/14
  width-leak/gate-drop family) and rebuilt;
   running it next as a shot at a 10th
  distinct root cause.
   STILL 9 confirmed root causes;
   the reachable surface is
  well-characterized (DIVERGE null,
   only 2 panic sites,
   SIMD = arm-bug-01 only).

- 02:30:
  9 root causes committed (bug-09 added:
   full/js dot-literal concat
  compile blowup,
   >40s,
   fragile/non-monotonic,
   distinct from bug-06).
   Verified
  bug-05 is debug-build-only:
   `_*$` in a release (no-debug-assertions) build
  returns the CORRECT `find_all` (`[0:6,6:6]`),
   so it is a debug/test/fuzz crash
  not a release soundness bug (writeup already says so).
   DIVERGE null reconfirmed.
  Lean position-level lane:
   ABANDONED this box;
   >`curl|sh` elan install denied
  by the auto-mode classifier (external code exec;
   install authz was Mac-only),
  too heavy to reroute.
   Mining instead with a fresh differently-structured
  corpus (`gen_big2.py`,
   seed 7777,
   heavier nested lookarounds / counted repeats
  / deep complement+intersection / case-shifting classes):
   16-way `--oraclebatch`
  running on big2 to surface violation types not yet filed (a non-zero-width
  HARDDIFF,
   a distinct OVERLAP/BOUNDS,
   etc.).
   Overnight fuzzers still running on
  both machines.

- 02:05:
  DIVERGE lane (is_match resharp-ascii vs regex-crate over 20k
  shared-subset patterns x22 haystacks,
   panic-tolerant in `repro --divergebatch`)
  is a CLEAN NULL:
   resharp's basic is_match agrees with the regex crate,
   so the
  core is correct and the bugs live in the advanced features.
   M1 simd_diff (7
  crashes) are ALL arm-bug-01 (leading-`^` zero-width:
   `^$`,
  `^\0?`,
  `^\n`,
  `^\u{1}?`);
   one SIMD root cause,
   well-confirmed.
   NEW candidate root cause
  (bug-09,
   perf):
   a long `.`-and-literal concatenation blows up COMPILE in
  javascript/full mode;
  the fuzzer's `compile` timeout
  `.n.................  n...  n` (hexpat 2e6e...20206e) takes ~71s.
   Bisection by
  prefix length is NON-MONOTONIC (len 22 = 16.8s,
   len 23 = >30s,
   len 24+ =
  0.001s),
   so it is a specific minterm/derivative product blowup,
   distinct from
  bug-06's `\w` repeat cost.
   Characterizing modes/minimal now.
   Two crash sites
  total across all in-tree artifacts:
   `algebra:2724` (bug-04,
   x230) and
  `engine:1824` (bug-05,
   x1).
   diff_regex2 catch_unwind is defeated by
  libfuzzer-sys's abort-at-panic hook (use repro for the is_match diff,
   done).

- 01:45:
  8 root causes committed (README,
   8 bug files,
   verification,
   code-quality,
  test-coverage,
   handover).
   Overnight fuzzers relaunched on both machines (4h).
  Tried a BUG-1-tolerant `diff_regex2` libFuzzer target to unmask is_match
  divergences,
   but libfuzzer-sys's panic hook `abort()`s at the panic point so
  `catch_unwind` can't catch the bug-04 panic;
   pivoting the is_match-vs-regex
  differential into `repro` (where catch_unwind works) over a shared-subset
  corpus.
   simd_diff (M1) still only finds arm-bug-01 variants.
   No new crash site
  beyond `algebra:2724` (bug-04) and `engine:1824` (bug-05) so far.

## Next avenues for more yield

- A BUG-1-tolerant `diff_regex` (wrap resharp compile in `catch_unwind`,
   compare
  `is_match` only when both compile) to find resharp-vs-`regex`-crate is_match
  divergences currently masked by the bug-04 panic.
- Lean position-level correctness lane (the 06-04 pipeline;
   `~/Downloads/
  extended-regexes`,
   elan needed;
   not yet set up this campaign) for correctness
  bugs the self-consistency oracles miss.
- Per-mode oracle rounds (flags config produced bug-08;
   ascii/full may differ).
- Harvest the running fuzzers;
   minimize any new distinct crash site.
