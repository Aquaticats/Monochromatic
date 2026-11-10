# Handover: resharp rust fuzz campaign (2026-06-04)

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Living handover for the resharp fuzzing effort.
 Context compacts repeatedly,
 so
this file is the single source of truth for resuming with zero rediscovery.
 The
per-bug writeups live in `doc/audit/resharp-fuzz-2026-06-04/`.
 Update this file
as state changes.

## Mission and goal

Find as many real bugs as possible in `ieviev/resharp` (the Rust engine) at
current `main` (the version with the two recently merged fuzzing-enablement PRs).
The user maintains a fork (`Aquaticat/resharp`).
 resharp is a derivative-based
automaton regex engine with intersection `&`,
 complement `~`,
 lookarounds,
 and
leftmost-longest (POSIX) semantics,
 multiline on by default.

## Current status (2026-06-04 18:55)

23 distinct root causes and 39 numbered reproducers committed under
`doc/audit/resharp-fuzz-2026-06-04/`:
 BUG-1,
 2,
 3,
 4,
 7,
 8,
 9,
 10,
 12,
 13,
 14,
 15,
16,
 17,
 18,
 19,
 20,
 21,
 22,
 23,
 25,
 26,
 27 (numbers 5 and 6 folded;
 BUG-11 CONFIRMED
identical to BUG-17,
 the bracketed `[\w]`;
 BUG-24 CONFIRMED identical to BUG-7,
 the
ascii negated-shorthand language complement,
 so each counts once;
 there is no
BUG-24).
 Plus one regression,
 REG-1,
 introduced by the v0.6.9 cleanup commit (see
the v0.6.9 re-run section below).
 A full source-level code-quality audit is in
`code-quality.md` (definitely-rewrite tier) and `code-quality.recommendations.md`.
Distinct triggering patterns run to the thousands (BUG-15:
 28688 distinct patterns
in the full 159k hunt).
 All work is committed.
 The user wants maximum yield ("poke
resharp until it's a honeycomb");
 20 is a floor,
 not a stop.

## v0.6.9 re-run (commit 264e85b / 4ffe1cc, 2026-06-04 18:55)

Upstream landed `4ffe1cc` "cleaning up and simplifying edge cases" (on `main` as
`264e85b` "bump version",
 tag `v0.6.9`),
 a direct child of the campaign baseline
`a7ab016`,
 so it isolates exactly one diff.
 Re-ran the whole campaign against it.

- NONE of the 23 filed bugs are fixed.
   Every reproducer is byte-identical on the
  parent and v0.6.9 (harness `/tmp/agent/recheck.ts`,
   all configs).
- The commit's footprint is narrow:
   a before/after `find_all` differential over the
  6426-pattern hard corpus (96390 pairs,
   default config) shows ZERO changes.
   All
  change is in the zero-width negative-lookahead-of-anchor and `\b\B`-anchor family.
- It introduces REG-1 (filed as
  `regression-01-neg-lookahead-zerowidth-duplicate-findall-spans.md`):
   `(?!\A)`
  makes `find_all` emit the same zero-width span twice (`(?!\A)` on `"ab"` ->
  `1:1,1:1,2:2`,
   was `1:1,2:2`).
   Single-hunk bisection on a fork copy
  (`/tmp/agent/resharp-bisect`) pins it to the new `mk_neg_lookahead` zero-width
  branch (`resharp-algebra/src/lib.rs:3554`):
   `(?!body)` with a zero-width body is
  now lowered to `EPS & ~body`,
   which double-registers interior nullable positions
  in `collect_rev` (engine `debug` null dump:
   `[nulls] [3,2,2,1,1]` vs parent's
  `[3,2,1]`).
   Reverting only `nulls.rs` (the `and_id` rewrite) does NOT fix it;
  reverting only this branch does.
   Config-independent;
   regresses patterns the parent
  compiled correctly (`(?!\A)`,
   `(?!\A)|a`,
   `a|(?!\A)`).
- Same commit's improvements (all in the same family):
   `(?!\A)a`,
   `(?!\A)*`,
  `(?!\A)(?=[A-Z])`,
   `(?!\A){2}`,
   `(?!\A)(?!\A)` now compile and/or match correctly
  where the parent rejected or wrongly missed them,
   and the parent's compile PANIC
  on `\A((?<=a)B+|x)` becomes a clean `UnsupportedPattern`.
- Oracle gap:
   equal-span zero-width duplicates trip neither OVERLAP (`start<prev_end`
  is `1<1`=false) nor INCONSIST nor BOUNDS.
   REG-1 was found by a direct find_all
  before/after diff with an explicit duplicate-span check.
   Add a `DUPSPAN` oracle
  (two emitted matches with equal start and equal end) before the next re-run.
- After-engine harnesses:
   `/tmp/agent/repro-after` (oracle vs v0.6.9),
  `/tmp/agent/resharp-after-20260604` (v0.6.9 clone),
   `dbg-after`/`dbg-before`/
  `dbg-bisect` (engine `debug`-feature null dumps),
   `recheck.ts`/`recheck2.ts`/
  `dup.ts`/`classify.ts`/`genpairs.ts`/`showdiff.ts` (differentials).

Six root causes added in the 17:00 round (all committed):

- BUG-21:
   a reused `Regex` returns history-dependent wrong answers.
   Repeating an
  identical `is_match("ba")` on `\Bb` flips false->true;
   a prior query leaks the
  `usize::MAX` `NO_MATCH` sentinel into a `find_all` `Match.end`.
   Root cause:
   the
  reverse lazy-DFA cache (`rev_ts`) at `handle_rev_end` (`engine.rs:1478`) returns a
  different cached state for the same `(sid, minterm)` across calls;
   the
  begin-of-input boundary context is not part of cached state identity.
   Found by
  reusing one Regex across haystacks in `--checkbatch` (STREAMINCONSIST + BOUNDS).
- BUG-22:
   O(n^2) `is_match`/`find_all`/`stream` on a repetitive single-byte prefix
  with a failing suffix (`(a+)+b`,
   `(a|a)*b`;
   4.4s at 64KiB,
   stream ascii >25s).
   DFA
  is bounded (5 states) so not state explosion:
   `is_match_fwd_prefix`/
  `find_all_fwd_prefix` re-scan from every prefix occurrence,
   advancing `search_start`
  by one byte after a failed scan (`fwd.rs:92`,
   `:51`).
   NOT hardened-mitigated
  (distinct from BUG-18).
   Found by `--time1`.
- BUG-23:
   full-unicode `\w` bounded-repeated blows up compile super-linearly
  (`\w{16}`=15.6s,
   `\w{12}`=9.6s in full;
   ~0.03s default/ascii).
   Specific to `\w`/`\W`
  (the large multi-byte set);
   `\d`/`\s`/`.`/ASCII are instant.
   Parser unrolls `{n,m}`
  via `mk_repeat` (`parser/lib.rs:2030`,
   `algebra/lib.rs:3710`) instead of the native
  `Kind::Counted`.
   Distinct from BUG-17 (bracketed,
   slow in default too) and BUG-19
  (match phase).
- BUG-25:
   a panic inside the locked region poisons the `inner` `std::sync::Mutex`,
  so every later `.lock().unwrap()` (16 sites) panics with `PoisonError`.
   One bad
  input permanently bricks a shared compiled `Regex` (all methods,
   all threads) even
  if the caller catches the first panic.
   Minimal `\w+b` default,
  `find_all(["ab","ba"])`.
   Orthogonal amplifier of BUG-2/BUG-21 aborts.
- BUG-26:
   `\z\A` is compiled to the empty language (BOT) and misses the empty match
  it should make (`\A\z` is correct).
   `mk_concat` (`algebra/lib.rs:3232`) reduces an
  `End` head before a non-END-nullable tail to BOT,
   ignoring that on the empty input
  the end position is also begin,
   so a begin-nullable tail (`\A`) still matches "".
  Reclassifies the prior `\z\A` findings from BUG-3 (is_match and find_all actually
  AGREE = []).
   Found by `--divergebatch`.
- BUG-27:
   a word boundary composed with a nullable filler flips on the empty string:
  `\ba{0}\b` matches "" (should not) and `\Ba{0}\z` fails on "" (should match),
   all
  configs.
   `\b` lowers to boundary lookarounds over `~(\w)` (nullable),
   so on "" both
  sides are satisfied and the "word char on one side" rule is lost under composition.
  Bare `\b`/`\B` are correct.
   Found by `--divergebatch` over an anchor-composition
  corpus.
   Same complement-is-nullable theme as BUG-7.

The bracketed perl-class issue (BUG-24/BUG-7) was nearly double-counted:
 BUG-7 already
covered ascii `\W`/`\D`/`\S` matching the empty string.
 This round PINNED its exact
line (`parser/lib.rs:1373` uses `mk_compl` instead of `neg_class`;
 the js branch at
`:1309` is correct) and confirmed the scope (bracketed `[\W]` is correct;
 only the
bare shorthand breaks;
 only ascii config).
 Folded into `bug-07`;
 the `bug-24` file was
removed.
 LESSON:
 check the existing bug index before filing.

A bug is not only a panic or a wrong answer.
 ieviev's invariant:
 with the size
limits NOT disabled,
 nothing should take >= 10s.
 So under any limits-enabled config
(0..5;
 the `unbounded` config 6 disables limits and is EXEMPT),
 a single
compile/match op taking >= 10s is a bug,
 and >= 1s is suspicious and must be
documented (a clearly-pathological >= 1s case counts as a full bug).
 This produced
the timing oracle (`--time1`) and three performance bugs:

- BUG-16:
   lookbehind of a positive lookahead is ~O(n^3) at match time.
   `(?<=$)`
  find_all is 13s on 512 bytes (`$` desugars to a lookahead;
   the inner lookahead
  never reaches a fixpoint so the lazy DFA mints a state per offset).
- BUG-17:
   a perl shorthand inside a character class (`[\w]` vs bare `\w`) makes
  bounded-repeat compile super-linear.
   `([\w]{3,5}){3,3}` compiles in 15s;
   bare
  `(\w{3,5}){3,3}` is 20ms. Likely the real root cause of BUG-11.
- BUG-20:
   `find_anchored` reports a match at offset 0 that a leading zero-width
  assertion forbids there.
   `\B0` on `"00"` returns 0:1 (wrong;
   `\B` is false at the
  start) and `(?<=0)0` returns 0:1 (wrong;
   nothing precedes 0),
   while `find_all`
  correctly matches at 1:2 in both (regex-crate-corroborated for `\B`).
   find_anchored
  (`engine/src/lib.rs:1847`) calls `scan_fwd_slow(0,...)` without seeding the
  begin-of-input context find_all keys on (`pos_begin==0` / `Nullability::BEGIN`,
  `engine.rs:818`/`:921`).
   Found by the new FANDIFF oracle.
- BUG-18:
   `find_all` is O(n^2) on a nullable complement (`~(a+)`,
   `~(\w+)`) because
  `find_all_nullable_slow` restarts a forward scan per position.
   `~(a+)` is 10.5s
  on 96KB;
   hardened (different driver) is linear,
   so the quadratic is avoidable.
  Generalizes to any nullable pattern with a far per-position scan (corroborated by
  the non-complement `${0,2}([a-c]_+&((?:a)*))a{1,3}[^a]\w*`,
   same O(n^2)).
- BUG-19:
   an anchor in front of a FULL-mode word class (`$?\w`=3s,
   `$\w`=1s,
  `$?\W`=2s) is a fixed ~1-3s match-time DFA construction cost on diverse-byte
  (cyc) input.
   Full mode only (`\w` alone,
   `\d`/`\s`,
   ascii/js all <1ms);
   the anchor
  splits the huge unicode `\w` minterm set.
   The mode-independent bracketed analogue
  `$[\w]` (~1.15s,
   diverse input) is filed as the match-time face of BUG-17.

User bar (2026-06-04,
 explicit):
 the machine is an AMD 8700F;
 even WITH the user's
ffmpeg video jobs running,
 every resharp op should take <= 1s.
 So do NOT dismiss
1-6s flags as mere contention.
 Re-measure each SOLO (kill your own parallel hunts
first;
 ffmpeg may keep running,
 that is the realistic condition),
 and treat any
op still > 1s solo as a real finding.
 Contention can inflate a 0.3s op past 1s but
cannot fake a >25s hang or turn a linear op super-linear,
 so solo re-measurement is
the arbiter.
 All five performance findings (BUG-16/17/18/19 plus BUG-17's
match-time face) are solo-confirmed with clean scaling or fixed-cost curves.

Diverse-byte input matters:
 BUG-19 and BUG-17's match-time face appear ONLY on
diverse bytes (`--benchcyc`),
 not on single-byte `--benchrep`,
 because the lazy DFA
only builds the expensive states when the input exercises many byte classes.

### The reference is Lean, not dotnet (user directive, 2026-06-04)

The Lean formalization is THE reference (verified ground truth).
 The dotnet engine
(`ieviev/resharp-dotnet`) is NOT a reference;
 it is immature too.
 Use dotnet only
as a secondary "another implementation to look at,
" never as an arbiter.
 Every bug
filed rests on Lean plus a rust-internal inconsistency or the `regex` crate,
 never
on dotnet alone.

## Environments, clones, and key paths

Historical artifacts were recorded under `/tmp/agent`.
Recreate current scratch under `~/temp/agent` with
`mkdir --parents "${HOME}/temp/agent"; chmod 700 "${HOME}/temp/agent"` if missing.
Do not delete audit artifacts there;
 the user cleans up.

- Pristine rust clone (do not modify,
   all reproducers confirmed here):
  `/tmp/agent/resharp-fuzz-20260604`.
   Engine crate
  `resharp-engine/src/engine.rs`,
   parser `resharp-parser/src/lib.rs`,
   algebra
  `resharp-algebra/src/lib.rs`.
- Suppressed rust fork (two re-entrancy panics turned into `return None` to dig
  past BUG-1;
   equals release semantics):
   `/tmp/agent/resharp-fork-20260604`.
- Rust reproducer and oracle crate (depends on the pristine engine by path,
   built
  release with debug-assertions and overflow-checks on to match cargo-fuzz):
  `/tmp/agent/repro`,
   binary `/tmp/agent/repro/target/release/repro`.
   Rebuild
  after editing:
   `cd "${HOME}/temp/agent/repro" && cargo build --release` (raw `cargo` is
  correct here;
   this crate is NOT part of the Monochromatic workspace,
   so the
  no-raw-tools rule does not apply).
- Dotnet clone (secondary,
   not a reference):
   `/tmp/agent/resharp-dotnet-20260604`.
- Dotnet harness (F#),
   a NATIVE binary (not a dll):
  `/tmp/agent/dnharness/bin/Release/net10.0/dnharness`.
   Reads `<hexpat> <hexhay>`
  lines on stdin,
   prints `im=..|fa=..|le=..` or `err=UnsupportedPatternException`
  or `err=...`.
   REQUIRES `export
  DOTNET_ROOT=/home/user/.local/share/mise/installs/dotnet/10.0.300` in any fresh
  shell,
   or it prints "You must install .
  NET".
   Rebuild:
   `cd "${HOME}/temp/agent/dnharness"
  && DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_ROOT=... <dotnet> build -c Release`.
- Lean reference:
   `/var/home/user/Downloads/extended-regexes` (Zhuchko,
   Veanes,
  Ebner).
   Toolchain via elan;
   need `export PATH="$HOME/.elan/bin:$PATH"`.
   lean4
  v4.24.0-rc1,
   mathlib already cached.
   The `RE` algebra is in
  `Regex/Definitions.lean` (constructors:
   `ε Pred Alternation(⋓) Intersection(⋒)
  Concatenation(⬝) Star(*) Negation(~) Lookahead(?=) Lookbehind(?<=)
  NegLookahead(?!) NegLookbehind(?<!)`).
   `llmatch` (in
  `Regex/MatchingAlgorithm.lean`) is leftmost-longest.
   `EliminationNegLookarounds
  .lean` proves negative lookarounds are eliminable with primitive anchors (a
  rewrite roadmap for the complement-of-lookaround limit).
   Run an eval file:
  `cd /var/home/user/Downloads/extended-regexes && export PATH="$HOME/.elan/bin:$PATH"
  && lake env lean <file>.lean` (prints `#eval IO.println` lines to stdout).
- Paper:
   `/var/home/user/Downloads/3704837.pdf`,
   text at `/tmp/agent/paper.txt`.

## Tooling: exact invocations

### repro (the rust oracle)

`mk(idx)` in `/tmp/agent/repro/src/main.rs` defines SEVEN configs swept by
`--sweep` (config-affectedness matters;
 BUG-4/7/8 are config-specific):
`0 default, 1 ascii (unicode=Ascii), 2 full (unicode=Full), 3 js
(unicode=Javascript), 4 hardened, 5 flags (case_insensitive + ignore_whitespace +
dot_matches_new_line + multiline=false), 6 unbounded (unbounded_size=true)`.
 Full
`RegexOptions` surface:
 `unicode`x4,
 `multiline`,
 `hardened`,
 `unbounded_size`,
plus translator flags `dot_matches_new_line`,
 `case_insensitive`,
`ignore_whitespace`.

- `repro '<pattern>' --sweep`:
   builds the pattern in all 7 configs,
   runs
  is_match,
   find_all,
   find_anchored,
   AND `re.stream` over a built-in haystack set,
  prints one line per oracle violation.
   Oracle prefixes below.
- `repro '<pattern>' <hex> --hex`:
   single explicit haystack.
- `repro --pair <hexpat> <hexhay>`:
   DEFAULT config only;
   builds fresh,
   runs
  is_match + find_all + find_anchored (NOT stream);
   prints `<hexpat>\t<hexhay>\t
  im=<0/1>|fa=<s:e,...>|le=<n>` (or `err=compile`/`err=panic`).
   `le` is
  `find_anchored(hay).end` (anchored at offset 0),
   NOT a general longest-end;
   it
  is only comparable to find_all's first span when that span starts at 0.
   This is
  the workhorse for single-case checks.
- `repro --batch < pairs.txt`:
   stdin `<hexpat> <hexhay>` lines,
   DEFAULT config,
  prints `im=|fa=|le=` per line,
   with a 2000ms-per-pattern watchdog thread that
  prints `err=timeout` on slow-compile patterns.
   Used for the Lean-round rust side.
- `repro --panicbatch < hexpats.txt`:
   stdin one `<hexpat>` per line;
   for each
  pattern and all 7 configs,
   streams the built-in haystack set under
  catch_unwind;
   prints ONLY `PANIC|<file:line msg>|mode=..|hay=..|pat=<hexpat>`.
  This is the crash hunter;
   it MUST call `re.stream` (BUG-15 only fires via
  stream).
- `repro --stream1 <hexpat> <hexhay>`:
   builds a fresh regex,
   calls `re.stream`
  once on the one haystack,
   prints `ok` or `PANIC <loc msg>`.
   Isolates
  single-call stream panics.
- `repro --time1 <hexpat>`:
   the TIMING ORACLE.
   Times compile across configs 0..5
  (limits-enabled;
   skips 6/unbounded) and is_match/find_all/find_anchored/stream
  under default(0)+hardened(4) over a 3-haystack battery (`a1k`,
   `cyc16k`,
   `a64k`).
  Prints `SLOW|<secs>|op=..|mode=..|hay=..|pat=<hexpat>` for any op >= 1.0s.
   A
  watchdog thread fires at 25s,
   printing `TIMEOUT|>25|op=..|mode=..|hay=..|
  pat=<hexpat>` then `process::exit` (so a hard hang is attributed to the op in
  flight).
   NOTE:
   match-op timing currently runs all six configs 0..5 (the
  `if midx != 0 && midx != 4` skip was removed per the all-config request).
   Run
  the corpus with `xargs -P<n> -n1 -I{} timeout 35 repro --time1 {}`;
   lower `-P`
  to avoid contention inflating times.
- `repro --bench1 <hexpat> <hexhay> <op> [cfgidx]`:
   one config,
   one op
  (is_match/find_all/find_anchored),
   prints seconds.
   For solo confirmation.
   Hay on
  argv,
   so keep it small.
- `repro --benchrep <hexpat> <bytehex> <N> <op> [cfgidx]`:
   builds hay = byte*N
  INTERNALLY (argv cannot carry a 64KB+ hex hay;
   "Argument list too long").
   This is
  the scaling workhorse:
   vary N to read off O(n)/O(n^2)/O(n^3).
- `repro --compile1 <hexpat> [cfgidx]`:
   prints `<secs>|ok=<bool>` for compile only.
  Used to isolate BUG-17 (compile-time blowup).
- `repro --benchcyc <hexpat> <N> <op> [cfgidx]`:
   hay = bytes 0..256 cycling,
   length
  N (DIVERSE input).
   This is what surfaces BUG-19 and BUG-17's match-time face;
  many slownesses appear only on diverse bytes,
   not on single-byte `--benchrep`.

### dnharness (dotnet, secondary)

`export DOTNET_ROOT=/home/user/.local/share/mise/installs/dotnet/10.0.300` first.
`echo '<hexpat> <hexhay>' | "${HOME}/temp/agent/dnharness/bin/Release/net10.0/dnharness"`.
Throws `err=UnsupportedPatternException` on constructs it cannot parse (nested
lookarounds,
 some complements,
 alternation-with-lookbehind).
 Useful only as a hint.

### Lean oracle pipeline

- `/tmp/agent/re2lean.py`:
   translates an RE# pattern to a Lean `RE (BA Char)`
  term.
   Encodes anchors as lookarounds:
   `\A`=`(?<! _)`,
   `\z`=`(?! _)`,
  `^`=line-start lookbehind,
   `$`=line-end lookahead,
   `\b`/`\B`=word-boundary
  lookaround pairs.
   Validated against the 19 known-answer anchor cases that `/tmp/agent/val_expected.txt` and
  `/var/home/user/Downloads/extended-regexes/leanval2.lean` hold.
   KNOWN
  UNFAITHFUL for lookbehind whose body contains a lookahead or anchor
  (`(?<=(?=...)...)`,
   `(?<=$)`);
   do not trust Lean disagreements on those shapes.
- The `sp` helper used in generated files:
  `def sp (r) (s) : String := match llmatch r s.toList with | some x =>
  toString x.i ++ ":" ++ toString x.j | none => "none"`.
   Output line format is
  `R<idx> <i:j>` or `R<idx> none`.
- Generators:
   `/tmp/agent/gen_lean2.py` (non-anchor leftmost-longest position
  round),
   `/tmp/agent/gen_lean_anchor.py` (anchor round).
   Each emits a `.lean`
  file (header + 54000 `#eval` lines) plus an aligned `<...>_pairs.txt`
  (`<hexpat> <hexhay>` per line,
   index-aligned to the `R<idx>` lines).
- Encoding validator:
   `/var/home/user/Downloads/extended-regexes/leanval2.lean`
  (19 known-answer anchor cases) vs `/tmp/agent/val_expected.txt`.
   Run before
  trusting any anchor round;
   all 19 must match.
- Chunked Lean run recipe (the only way a 54k-eval round finishes in reasonable
  time;
   ~0.2s/eval non-anchor,
   slower for anchor terms;
   do not exceed ~16
  concurrent `lake env lean` since each reloads mathlib oleans):

  ```sh
  cd /var/home/user/Downloads/extended-regexes
  export PATH="$HOME/.elan/bin:$PATH"
  head -3 /tmp/agent/<round>.lean > /tmp/agent/hdr.txt
  tail -n +4 /tmp/agent/<round>.lean > /tmp/agent/ev.txt
  split -n l/16 -d --additional-suffix=.e /tmp/agent/ev.txt /tmp/agent/Chunk_
  for f in /tmp/agent/Chunk_*.e; do n=$(basename "$f" .e|sed 's/Chunk_//'); cat /tmp/agent/hdr.txt "$f" > "<round>_chunk_$n.lean"; done
  for f in <round>_chunk_*.lean; do n=$(basename "$f" .lean|sed 's/<round>_chunk_//'); timeout 1800 lake env lean "$f" 2>/dev/null | grep '^R' > "<round>_out_$n.txt" & done; wait
  ```

  The `<round>_out_*.txt` files land in the extended-regexes dir,
   NOT /tmp/agent.

- Rust side of a round (parallelize;
   single-process `--batch` is the bottleneck):
  `split -n l/16 -d --additional-suffix=.p <round>_pairs.txt "${HOME}/temp/agent/Rp_"`;
  run `repro --batch < Rp_NN > "${HOME}/temp/agent/Rr_NN"` in parallel;
   then
  `cat "${HOME}"/temp/agent/Rr_*.txt > <round>_rust.txt` (glob sorts numerically,
   preserving
  index alignment).

### Adjudication scripts

- `/tmp/agent/adj_full.py <out_prefix> <rust_file> <pairs_file>`:
   the three-way
  harvester.
   Loads Lean `<out_prefix>*.txt` from the extended-regexes dir,
   rust
  results,
   and pairs;
   for every (pattern,
  haystack) where valid rust disagrees with
  Lean,
   batches it through dnharness and buckets:
   `RUST_WRONG` (dotnet AND Lean
  contradict rust;
   strongest),
   `ENCODING_SUSPECT` (dotnet agrees with rust against
  Lean;
   translation likely unfaithful,
   discard),
   `UNADJUDICATED` (dotnet threw),
  `3WAY` (all differ).
   Treat RUST_WRONG as candidates,
   verify translation is a
  faithful shape (not lookbehind-of-lookaround),
   then minimize.
- `/tmp/agent/diff_lean_span.py <leandir> <out_prefix> <rust_file> <pairs_file>`:
  Lean-vs-rust only;
   reports POSITION_DIFF and EXIST_DIFF counts and distinct
  patterns.
- `/tmp/agent/adj.py`,
   `adj2.py`,
   `adj3.py`:
   focused minimal-case probes;
   edit the
  `CASES` list and run with `DOTNET_ROOT` exported.
   Template for new probes.

## Oracle prefixes emitted by repro --sweep

- `PANIC|<file:line msg>|mode=..|hay=..|pat=..` any panic,
   assert,
   or abort.
- `BOUNDS` find_all match with end>len or start>end.
   `OVERLAP` overlapping or
  out-of-order find_all.
   `INCONSIST` is_match disagrees with find_all
  non-emptiness.
   `ANCHOR` find_anchored start not 0.
- `HARDDIFF_FA`/`HARDDIFF_IM` default vs hardened differ.
   `HARDPANIC_*` default
  panics where hardened does not.
   `STREAMBOUNDS`/`STREAMOVERLAP`/`STREAMINCONSIST`
  the `re.stream` path is out of bounds,
   overlapping,
   or disagrees with is_match.
- `DIVERGE` resharp ascii mode vs the `regex` crate built `.unicode(false)` (trust
  only pure-ascii haystacks,
   no anchors,
   no resharp-only operators).
   The `regex`
  crate is a second independent oracle for the shared syntax subset.

Self-consistency oracles (INCONSIST,
 BOUNDS,
 OVERLAP,
 HARDDIFF,
 STREAM*,
find_anchored-vs-find_all) need no external reference:
 a single engine
contradicting itself is unambiguous.

## Source-code map (pristine clone)

- `resharp-engine/src/engine.rs:12` `NO_MATCH = usize::MAX` sentinel;
   must never
  reach a Match (BUG-2,
   BUG-4).
- `engine.rs:550` `create_state` reads `state_nodes[state_id]`;
   panics when the
  caller passed an unregistered id (BUG-15 crash site).
- `engine.rs:960` `assertion left != right: correctness issue found` (BUG-2).
- `engine.rs:1249` `scan_rev_from` reverse-scan loop calls `create_state(b, curr)`
  with NO preceding `ensure_capacity(curr)`,
   unlike `lazy_transition_slow`
  (`:414`-`:415`) and the block matchers (`:415`-`:416`,
   `:441`-`:442`).
   This is
  BUG-15's root cause;
   reached only via `stream` -> `try_emit_step`
  (`stream.rs:247`).
   Audit siblings at `engine.rs:1098` and `:1185`.
- Parser limits:
   `ensure_lookbehind_at_start` `resharp-parser/src/lib.rs:479`
  (lookbehind must be leftmost);
   `ClassRangeLiteral` `:305` (`[\d-a]`);
  `UnsupportedLazyQuantifier` `:2275`,
  `:2363`;
   `UnsupportedBackreference` `:2621`;
  swap-greed flag `:1872`;
   special word boundaries `:1942`-`:1958`.
   Algebra
  `resharp-algebra/src/lib.rs:39` complement/star + lookaround/anchor limit.
   Size
  caps `lib.rs:56`-`:59` (`DEFAULT_MAX_REPEAT=500`,
   `EXPANDED_AST_LIMIT=50_000`,
  `MAX_LIST_LEN=4_000`,
   `MAX_DEPTH=1_000`).

## The 23 bugs (files in doc/audit/resharp-fuzz-2026-06-04/)

BUG-21 through BUG-27 are detailed in the `Current status (2026-06-04 18:55)` section.
 BUG-1 through
BUG-20 follow;
 BUG-19/20 are in the status section's timing list.

- BUG-1 `bug-01-...`:
   re-entrancy guard panic in union/intersection rewrites,
  compile time.
   `.*(.+)*.+`.
- BUG-2 `bug-02-...`:
   `correctness issue found` assert at `engine.rs`:
  960 (NO_MATCH
  reaches a Match).
   `\S+b` on `b'_`.
- BUG-3 `bug-03-...`:
   is_match disagrees with find_all.
   Triggers `(\z|(?=a)\w)`,
  `\BU`,
   `(?<=\D?[a-c]+0?)b` on `ba`.
   (The `\z\A` reversed-anchor cases once filed
  here are now BUG-26:
   a compile-time empty-language reduction where is_match and
  find_all AGREE = [],
   so not a BUG-3 divergence.
  )
- BUG-4 `bug-04-...`:
   find_all emits `end=usize::MAX`.
   `~(_*$)`,
   `\Bb+`,
  `(?<=[^a])b+`.
- BUG-7 `bug-07-...`:
   negated perl classes `\D \S \W` nullable (ascii):
  they match
  EVERYTHING including word chars and the empty string.
   `\D`.
   Root cause now PINNED:
  `perl_class_node` ascii else-branch uses `tb.mk_compl(pos)` (language complement)
  instead of `neg_class` (`parser/lib.rs:1373` vs the correct js branch at `:1309`).
  Scope:
   bare shorthand only (bracketed `[\W]` is correct);
   ascii config only
  (default/full/js correct).
   The duplicate `bug-24` was merged here and removed.
- BUG-8 `bug-08-...`:
   hardened find_all differs from default;
   hardened wrong.
  `~(_a+)`,
   `~(\D+)`.
- BUG-9 `bug-09-...`:
   stream path under-reports.
   `\A\z?`,
   `(^|b)`,
   `(?<!b)`.
   707+
  STREAMINCONSIST triggers.
- BUG-10 `bug-10-...`:
   default find_all drops a trailing zero-width match.
  `(?<=^)~(0+)`.
- BUG-11 `bug-11-...`:
   super-linear compile time.
   `[\w]{3,5}[\w]([^a]&a+)` ~4s.
  CONFIRMED to be the same root cause as BUG-17:
   the bracketed `[\w]` is the whole
  cost (bare `\w` version is 0.0068s vs 2.79s;
   the intersection is incidental).
  Counted once with BUG-17.
- BUG-12 `bug-12-...`:
   negative lookahead of a class makes a non-nullable pattern
  nullable;
   spurious empty match.
   `(?!\w)0+`.
   Lean-only find (self-consistent).
- BUG-13 `bug-13-lookahead-width-leak.md`:
   a top-level lookahead leaks its body
  width into the zero-width span.
   `(?=(?=c)c{1,3})` -> `0:1`.
   Lean + find_anchored
  vs find_all internal disagreement.
- BUG-14 `bug-14-alternation-drops-lookbehind-gate.md`:
   a nullable alternation
  sibling drops a lookbehind gate in find_all.
   `(|(?<=[a-z])b)` -> `0:1`.
   Lean
  (with a longest-pref control) + rust isolation argument.
- BUG-15 `bug-15-stream-dfa-construction-panic.md`:
   broad `stream()` DFA crash at
  `engine.rs`:
  550.
   Full 159k panic hunt (DONE):
   28688 distinct patterns,
   165515 panic
  lines,
   ALL 7 configs.
   Minimal `Regex::new("a&b").unwrap().stream(b"aaa")`
  (3+-byte input;
   not via is_match/find_all/find_anchored).
   Root cause
  `engine.rs`:
  1249 (missing ensure_capacity).
   The hunt confirms only two crash sites
  total (`engine.rs`:
  550 = 165515 lines,
   `engine.rs`:
  960/BUG-2 = 137 lines).
- BUG-16 `bug-16-lookbehind-of-lookahead-superlinear-match.md`:
   lookbehind of a
  positive lookahead is ~O(n^3) at match time.
   `(?<=$)` find_all 13s on 512 bytes,
  >2min on 1KB;
  > `(?<=(?=z))` (inner lookahead that FAILS) is the general trigger.
  is_match short-circuits when an early match exists,
  > else it blows up too.
  > Root
  cause:
  > Lookbehind derivative arm `resharp-algebra/src/lib.rs:1378` re-derives the
  inner lookahead each step without fixpoint.
  > Blows up under every limits-enabled
  config except where a flag incidentally removes the trigger (flags:
  > multiline-off
  drops `$`'s newline-lookahead;
  > ignore_whitespace eats `(?= )`'s space).
  > Resolves
  the PERFORMANCE angle of the held-back `(?<=$)` cluster.
- BUG-17 `bug-17-bracketed-perl-class-repeat-compile-blowup.md`:
   a perl shorthand
  inside a character class (`[\w]` vs bare `\w`) misses the single-predicate fast
  path;
   bounded-repeat compile is super-linear.
   `[\w]{3,5}` = 1.76s,
  `([\w]{3,5}){3,3}` = 15.3s,
   bare `(\w{3,5}){3,3}` = 20ms. NOT class size
  (`[\x00-\xff]` and explicit `[A-Za-z0-9_]` are instant);
   the perl-to-union
  lowering (`resharp-parser/src/lib.rs:186`) feeding `mk_repeat`'s unroll
  (`resharp-algebra/src/lib.rs:3710`).
   Mode-independent;
   max_repeat cap does not
  bound it.
   Likely the real root cause of BUG-11 (whose trigger also brackets
  `[\w]{3,5}`).
- BUG-18 `bug-18-findall-nullable-complement-quadratic.md`:
   `find_all` is O(n^2) on
  a nullable complement (`~(a+)`,
   `~(\w+)`).
   `find_all_nullable_slow`
  (`resharp-engine/src/lib.rs:1794`) restarts `scan_fwd_slow` from every position;
  a complement that matches empty everywhere gives N positions x O(n) scan each.
  `~(a+)` = 10.5s on 96KB,
   18s on 128KB;
   is_match/find_anchored are O(1).
   Quadratic
  under every limits-enabled config except hardened,
   which uses the linear
  `find_all_dfa` driver (`:1713`),
   proving it avoidable.

The README in the audit dir has the root-cause index,
 the 30 numbered findings,
the limits inventory pointer,
 and the Lean-round writeups.
 A separate
`limits-and-recommendations.md` documents every deliberate compile-time limit
(fundamental vs implementable vs tuning) with recommendations for ieviev.

## Pattern corpora (in /tmp/agent)

- `patterns_all.txt`:
   159257 RAW RE# patterns (one per line) from the directed
  sweeps.
   `fullpats.hex`:
   same 159257 as distinct hexpats (for `--panicbatch`).
- `pairs_all.txt`,
   `sweep_combined.txt`:
   pair/sweep outputs.
- `lean2_pairs.txt`,
   `leanA_pairs.txt`:
   54000 `<hexpat> <hexhay>` each (non-anchor
  position round,
   anchor round).
   Aligned rust sides `lean2_rust.txt`,
  `leanA_rust.txt`;
   Lean outputs `lean2_out_*.txt`,
   `leanA_out_*.txt` in the
  extended-regexes dir.
   Both rounds harvested.
- Distinct-trigger lists from prior mining:
   `diverge_ascii_pats.txt`,
  `harddiff_pats.txt`,
   `inconsist_pats.txt`,
   `bounds_pats.txt`.

## Throughput gotchas (learned the hard way)

- `DOTNET_ROOT` is NOT set in fresh shells;
   export it before any dnharness call.
- Lean `<round>_out_*.txt` land in `/var/home/user/Downloads/extended-regexes`,
  not /tmp/agent.
- `le` in repro output is `find_anchored().end` (anchored at 0),
   not longest-end.
- A stray `dnharness`/`lean` process can hang an hour on one pathological pattern
  and steal a core.
   Check `ps -eo pid,pcpu,comm --sort=-pcpu`,
   kill by PID.
  `ffmpeg` in that list is the user's VideoDownloader;
   leave it.
- `pkill -f '<pattern>'` matches its own shell.
   Kill via a script reading
  `/proc/*/cmdline` (`/tmp/agent/killsweep.py`) or by explicit PID.
- Parallelize heavy sweeps 16-way with `split -n l/16 -d` + per-chunk `timeout`.
- BUG-15 panics ~20% of patterns via stream,
   so `--panicbatch` output is huge;
  grep/uniq the `engine.rs:NNN` site,
   not the full lines.

## In-flight background jobs (poll .done, do not restart)

- Full-corpus panic hunt:
   DONE (`/tmp/agent/fullpanhunt.done` exists).
   Final:
   two
  crash sites only,
   `engine.rs`:
  550 (28688 distinct patterns,
   165515 lines) and
  `engine.rs`:
  960/BUG-2 (137 lines),
   across `Fpan_*.txt`.
   BUG-15 scope updated.
- Timing hunt over the full corpus,
   output `/tmp/agent/timehunt2_all.txt` (no
  `.done` marker written by the relaunch;
   it was `&`-detached inside a bash call).
  Each line is `SLOW|<secs>|...` or `TIMEOUT|>25|...`.
   Heavily BUG-16 (lookbehind-
  of-lookahead,
   ~100+ distinct) plus BUG-17 (op=compile,
   bracketed perl class) and
  BUG-18 (op=find_all,
   `~(...)` complement).
   Distinct flagged hexpats extracted to
  `/tmp/agent/flagged_pats.hex`.
- Clean SOLO re-measurement was killed mid-run after the first ~18 lines already
  confirmed the residual cluster is REAL (e.g. `$[\w]` is_match 1s,
   `$?\w` full 3s);
  finishing it would have spent ~20min re-confirming BUG-16 hangs (25s each).
   The
  residual was then isolated directly with `--benchcyc` (diverse input):
   two new
  faces emerged,
   both now FILED (`$[\w]`->BUG-17 match-time,
   `$?\w` full->BUG-19).
  Remaining residuals checked and FOLDED:
   ` {0,2}.[^\w]` compile 1.2s is BUG-17
  (bracketed negated `[^\w]`);
   `${0,2}([a-c]_+&((?:a)*))a{1,3}[^a]\w*` find_all is
  BUG-18 (nullable,
   non-complement,
   O(n^2) 0.27/1.09/4.35).
   `flagged_pats.hex` (419
  distinct) remains if a future session wants an exhaustive solo sweep;
   run
  `--time1` over it with NO parallel hunts (ffmpeg ok) and scan for any solo op
  outside BUG-16/17/18/19.

## Held back, NOT filed

The anchor round's `(?<=$)` cluster (lookbehind containing an anchor),
 e.g.
`(_{0,1}&(?<=$))` on `\n` (rust 1:1,
 Lean 0:0) and `(?=(?<=$) *)[^a]*` on `\n`
(rust 1:1,
 Lean 0:1).
 First-principles reasoning leans toward rust being wrong,
but lookbehind-of-anchor is the translator shape known to be unfaithful,
 and these
show no internal-consistency violation.
 Need RE# lookbehind-of-anchor semantics
from the paper or source before filing the CORRECTNESS angle.
 NOTE:
 the PERFORMANCE
angle of this exact shape is now filed as BUG-16 (the `(?<=$)` superlinear match),
which needs no oracle;
 only the position-correctness question remains held back.

## New oracles and repro modes added this session (all in /tmp/agent/repro)

- `--time1`,
   `--bench1`,
   `--benchrep`,
   `--benchcyc` (DIVERSE cyc input;
   surfaces
  BUG-19 and BUG-17's match-time face),
   `--compile1`:
   the timing oracle family.
- `--fandiffbatch` (stdin hexpats):
   find_anchored vs find_all consistency.
   Built
  BUG-20.
   Output of the full-corpus run:
   `/tmp/agent/fandiff_all.txt` (~290k lines).
  Classes:
   `fa=empty|fan=0:N` and `fafirst=0:1|fan=0:0` are dominated by `\b`/`\B`
  (BUG-3) and leading-lookbehind (BUG-20);
   `fafirst|fan` span gaps are BUG-13/14.
- `--streambatch` (STREAMEXTRA:
   stream span not in find_all):
   DEAD END;
  the `Spot-checked, no bug in samples (NOT confirmed robust)` section says why.
- `--ci1 <hexpat> <hexhay>`:
   resharp case_insensitive(ascii) is_match/find_all.
- `--fa1 <hexpat> <hexhay> <cfgidx>`:
   im/fa under any config (for UTF-8 / mode probes).

Added in the 17:00 round (all in /tmp/agent/repro):

- `--checkbatch` (stdin hexpats):
   runs the full `check_one` internal-consistency
  suite (INCONSIST,
   BOUNDS,
   OVERLAP,
   ANCHOR,
   FANDIFF,
   STREAM*) over configs 0/2/4,
  REUSING one Regex across haystacks.
   The reuse is what exposed BUG-21 (cache
  contamination):
   STREAMINCONSIST (24 `\B`+intersection patterns) and BOUNDS (the
  `usize::MAX` leak).
- `--checkfresh` (stdin hexpats):
   same suite but a FRESH Regex per (pattern,
   config,
  haystack),
   so contamination cannot fire.
   Whatever still flags is a genuine
  per-input bug.
   Used to confirm BOUNDS/STREAMINCONSIST were purely BUG-21.
- `--hardbatch` (stdin hexpats):
   default vs hardened find_all/is_match over builtin +
  larger haystacks (`diff_hardened`).
   HARDDIFF = guaranteed bug (BUG-8);
   the
  HARDPANIC rows (default panics,
   hardened does not,
   then cascade) surfaced BUG-25.
- `--divergebatch` (stdin hexpats):
   resharp ascii is_match vs the regex crate
  (`unicode(false).multi_line(true)`),
   FRESH per haystack,
   existence-only (resharp is
  POSIX-longest vs regex leftmost-first,
   so spans legitimately differ).
   Built BUG-26
  and BUG-27,
   and re-found BUG-7.
   NOTE:
   regex crate is only a Lean-consistent proxy
  for the SHARED standard-feature subset;
   the user rejects it as a case-insensitivity
  oracle (Lean is the reference there).
- `--contam <hexpat> <hexhays-comma> <cfg>`:
   build one Regex,
   run is_match/find_all
  over a haystack sequence,
   then a fresh Regex on the last;
   prints REUSED vs FRESH.
  The minimal-priming tool for BUG-21/BUG-25.
- Corpora added:
   `/tmp/agent/corpus_hard.txt`,
   `corpus2.txt`,
   `corpus_plain.txt`,
  `corpus_edge.txt`,
   `corpus_anchor.txt`,
   `time_corpus.txt`;
   generators
  `gen.ts`/`gen2.ts`/`gen_plain.ts`/`gen_edge.ts`/`gen_anchor.ts` in `/tmp/agent/repro`.
  Standalone repro crate `/tmp/agent/contam-test` (depends on the pristine engine;
  has a panic-hook + `diff_hardened`-style harness;
   toggle `features=["debug"|"diag"]`
  in its `Cargo.toml` for the node trace / `dfa_stats`).

## Spot-checked, no bug in samples (NOT confirmed robust)

Standard (user directive):
 nothing is "confirmed robust" until that distinct portion
has been READ through in source,
 REASONED about,
 and UNIT-TESTED to the max.
 The
items below are only spot-checked:
 a handful of inputs passed,
 no bug surfaced.
 They
are candidates for thorough verification,
 not closed.
 Do not present them as robust;
each still needs source-read + reasoning + exhaustive adversarial unit tests before
that claim.
 The ONE exception is the stream-semantics item,
 which rests on a doc
fact,
 not a sample.

- DOC FACT (solid):
   `stream` is leftmost-SHORTEST ("Shortest matches,
   left-to-right.
  State resets after each match.
  ",
   `stream.rs:153`,
   via `scan_fwd_first_null_from`).
  So stream spans differing from find_all's leftmost-LONGEST is BY DESIGN;
   the
  STREAMEXTRA oracle's 60k hits (` *` stream 0:1 vs find_all 0:2) are NOT bugs.
   BUG-9
  (stream EMPTY when a match exists) stays valid:
   a shortest match must still exist.
  Still untested:
   whether stream's shortest spans are themselves always correct,
   and
  the `stream_first`/`stream_ends`/`stream_chunk`/`stream_with` variants.
- Case-insensitive (ascii):
   13 definitional inputs passed (`a` on `A`,
   `[a-z]` on
  `A`,
   `[^a]` on `A` false,
   `~(a)` on `A` via empty,
   `[A-Z]` on `a`).
   NOT tested:
  ci with `\b`/anchors,
   ranges crossing case (`[Z-a]`),
   `\w`/`\S` under ci,
   ci+full
  unicode case folding (Turkish i,
   sharp s),
   find_all spans under ci.
   Read the
  parser's ci lowering before claiming correctness.
- UTF-8 under full mode:
   `.` spans one codepoint (é=0:2,
   中=0:3,
   😀=0:4),
   `.{2}` on
  `éé`=0:4,
   `..` on 1 codepoint fails.
   NOT tested:
   invalid/truncated UTF-8 (lone
  continuation byte,
   overlong),
   `\w`/`\b` at codepoint boundaries,
   find_all
  splitting a codepoint,
   classes spanning the BMP,
   surrogate ranges.
- find_all zero-width word boundaries:
   `\b` on `"a"`=[0:0,1:1],
   `"a b"`=[0:0,1:1,
  2:2,3:3].
   (FANDIFF `fa=empty|fan=0:0` are BUG-20,
   not find_all drops.
  ) NOT
  exhaustively tested across boundary/anchor/lookaround combinations.
- Compile stress:
   nested complement,
   multi-intersection,
   counted `(a|b|c|d|e){200}`,
  `((a*)*)*` all instant;
   only bracketed `[\w]` blows up (BUG-17).
   Nullable
  INTERSECTIONS in find_all are linear (`(_+&a*)`),
   only nullable COMPLEMENT `~(a+)`
  is O(n^2) (BUG-18).
   This is a sample,
   not a proof of no other blowup.

## Remaining avenues (for more root causes)

- Harvest the clean solo timing re-measurement (`/tmp/agent/timeclean_all.txt`,
  marker `timeclean.done`):
   any solo op >= 10s outside BUG-16/17/18 is a new root
  cause;
   document 1-6s solo clusters as suspicious.
- Timing oracle has only sampled the directed corpus.
   Generate NEW adversarial
  patterns aimed at the three known mechanisms (lookbehind-of-lookahead nesting,
  bracketed-class repeats,
   nullable-complement find_all) to find sibling triggers,
  and run `--time1` with a longer/diverse haystack to surface match-time blowups
  that the a1k/cyc16k/a64k battery misses.
- Deepen other stream-correctness mining:
   BUG-15 blocks stream RESULTS on 28688
  patterns,
   so more stream bugs may hide behind the crash once it is fixed;
   for
  now mine STREAMINCONSIST on the non-crashing remainder.
- Per-mode Lean rounds (ascii,
   flags,
   hardened) since BUG-4/7/8 are config
  specific;
   the Lean encoding must match that config's class semantics.
- Resolve the `(?<=$)` CORRECTNESS cluster by pinning RE# lookbehind-of-anchor
  semantics (the performance angle is now BUG-16).
- find_anchored correctness vs Lean as a new oracle dimension (BUG-13 showed
  find_anchored can disagree with find_all).

## Conventions for the writeups

Repo prose rules:
 no em-dashes or en-dashes as em-dashes,
 sentence-case headings,
ATX headers,
 fenced code with language tags,
 lines under 120,
 no tables,
 no
emojis.
 Each bug file:
 self-contained reproducer (rust snippet plus `repro`
command),
 observed vs expected,
 affected configs,
 source location,
 relationship to
other bugs.
 Commit eagerly with `docs(resharp-fuzz): ...`,
 explicit pathspec (the
working tree has concurrent external "unbash" changes in `package/` and
`AGENTS.md`;
 never stage those,
 they are not ours).
