# Translation repair history, first handover document: pass 6 first runs, per-entry cap and slice resumability, 2026-07-24

The restart of the pass under lockstep slicing,
the runs that hit the hard cap,
the per-entry budget fix,
the long tail of heavily sliced entries,
and the decision and landing of slice-level resumability so that a large entry can finish over several runs.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Pass 6 restart note, prior-art survey and the decision to keep full deadlines

PASS 6 (2026-07-24):
  pipeline behavior changed (slicing),
  so the restarted
pass is a NEW pass;
  prior pass-5 artifacts and attempts.json discarded.
Note lessons banked while landing this:
  run package tasks ONLY by scoped
name (`//package/module/translation-repair:<task>` (moved on 2026-10-18 to
`//package/translation-repair/<package>:<task>`)) -- a bare `mise run
build`/`test` from the worktree root fans out to the whole monorepo
(cargo/podman/rust),
  spikes load,
  and OOM-kills scoped work;
  and a lint
`no-regex` rule blocks inline `String#match` regex (used substring checks
instead).
  PASS 6 RUN 001 launched on tip after the handover commit under
the lockstep slicing fix.
  Loop continues per task 30:
  record each run's
tallies content-free,
  commit,
  launch the next,
  until all 92 settle,
landing any further verified high-confidence fix immediately (restarting)
per the standing rule.
The user's concurrent
prior-art survey landed as doc/research/translation-repair-prior-art.md
(commits `650fc5827`,
  `059ce44e8`):
  closest precedents MQM-APE and
TEaR;
  the guarded-envelope composition is the unusual part;
  its
cautions (ensemble-checked is not independently-verified,
  seeded
numbers need human-graded evaluation for real-world claims) match
milestone three's human-graded gate.
  USER PICK (2026-07-23):
  keep full deadlines and retry-to-quorum as
is;
  no adjudication-quality change before the precision gate,
  the
accumulation loop absorbs the wall time.
  Racing to quorum and
deadline shortening stay recorded as rejected-for-now options,
revisitable if graded evidence changes the tradeoff.

## Pass 6 runs 001 to 005 under lockstep slicing

PASS 6 RUN 001 (2026-07-24,
   tip `973ca8235`,
   1556s wall,
   soft budget hit):
1 dispatched,
   1 completed,
   0 failed.
   Acheron repaired (46 issues,
   45
accepted,
   44 resolved,
   4 findings,
   1556s).
   Data point on the corrected
slicing:
   Acheron is in the drifted set,
   and its numbers moved from pass-5
old-slicing (56 issues,
   55 accepted,
   54 resolved,
   18 findings) to 46/45/44
with findings down 18->4.
   Factual:
   the issue set changed and finding-noise
dropped under 1:1 alignment;
   whether that is higher quality is for the
grade,
   not this delta.
   1/92 settled.
   Run 002 launched on tip `6a58ababf`.
PASS 6 RUN 002 (2026-07-24,
   tip `edc7959bf`,
   2620s wall,
   soft budget hit):
2 dispatched,
   2 completed,
   0 failed.
   AkiraComplex repaired (32 issues,
   28
accepted,
   28 resolved,
   1 finding,
   698s);
   AmbeR_the_anpa repaired (44
issues,
   42 accepted,
   42 resolved,
   4 findings,
   1923s).
   Both in earlier
passes too;
   no blocks under lockstep.
   3/92 settled.
   Run 003 launched.
PASS 6 RUN 003 (2026-07-24,
   tip `8459fd92d`,
   1764s wall,
   soft budget hit):
1 dispatched,
   1 completed,
   0 failed.
   Aniloviraw repaired (52 issues,
   52
accepted,
   all 52 resolved,
   10 findings,
   1764s) -- the original
contradiction-screen false-block entry,
   repairing cleanly again.
   4/92
settled.
   Run 004 launched.
PASS 6 RUN 004 (2026-07-24,
   tip `f08bd3996`,
   hard cap hit):
   Anilovr
status=ERROR,
   aborted at 2700002ms (the 45-min HARD_CAP),
   0 processed.
Diagnosis (NOT a regression,
   NOT a fix trigger):
   the log shows transport
`terminated` failures with retries plus two critic timeout bursts,
   and
only 6 of 7 slices finished in 45 min -- a bad API window at ~20:27Z,
   not
a workload change.
   Deterministic slice-count check (anilovr-slices.mjs)
proves it:
   Anilovr produces 7 slices under BOTH old and lockstep code
(delta=0;
   Arita +1,
   Acheron/AmbeR 0),
   and it repaired fine in pass-5 run
004 at 2540s with those same 7 slices.
   Self-heals by design:
   no artifact
written so Anilovr stays pending,
   but its attempt count went to 1,
   so the
fewest-attempts order now processes the 87 zero-attempt entries first and
retries Anilovr later (hopefully a calmer API window).
   Still 4/92 settled.
Run 005 launched.
PASS 6 RUN 005 (2026-07-24,
   tip `9c86aebf1`,
   hard cap hit):
   Arita
status=ERROR,
   aborted at 2700003ms,
   0 processed.
   This one is SYSTEMATIC,
not transient:
   the earlier live probe showed Arita legitimately takes
4110s (~68 min) for its 12 slices,
   which exceeds the 45-min cap,
   so Arita
can never settle as configured.
   Still 4/92 settled.

## Per-entry hard cap fix and the slice-count tail of the corpus

CORPUS-PASS BUDGET FIX (2026-07-24,
   commit `5b74bd7b2`):
   two problems
surfaced.
   (1) The hard ceiling was armed ONCE for the whole loop,
   so its
abort signal was shared -- an entry starting near the soft budget got only
the sliver left before the cap.
   Fixed:
   a fresh `armCallDeadline` per entry
(disposed via `using`,
   since try/finally is lint-banned),
   so each entry
gets its full budget regardless of start time.
   (2) 45 min was too tight;
raised HARD_CAP to 90 min (per entry),
   clearing every entry up to ~16
slices at the measured ~5.5 min/slice.
   Driver-only;
   repair results and the
4 settled artifacts unaffected,
   so NO restart.
   Verified:
   format/lint/types
0/0 and a `--plan` run (zero quota) shows hard=5400000ms.
LARGE-ENTRY TAIL (2026-07-24,
   deterministic slice survey
slice-distribution.mjs):
   the corpus slice-count distribution is
median 8 but long-tailed -- aiyysk 77 slices,
   hulicaijia 65,
   shihai4h 45,
interrgned 43,
   NIGHT81473140 41,
   Xu_Yushu 35,
   XingZ60 31,
   Dethelly 24;
total 1129 slices over 92 entries.
   At ~5.5 min/slice the biggest need
multiple HOURS end to end,
   so ~10 entries cannot complete in ANY bounded
single-run cap.
   The 90-min cap settles the ~80 smaller entries;
   the tail
needs slice-level RESUMABILITY (cache completed slices,
   resume across
runs) -- a change to the pure function's contract (inject a slice cache,
like the client is injected).
   Flagged to the user as a decision before
building.
   Meanwhile run 006 relaunches so the bulk keeps settling;
   huge
entries waste one 90-min attempt then deprioritize (attempts already 1 for
Anilovr/Arita).

## Run 006, the choice of slice-level resumability and its landing

PASS 6 RUN 006 (2026-07-24,
   tip `47ba504a3`,
   3034s wall):
   3 dispatched,
   3
completed,
   0 failed.
   ArtsEpiphany unchanged (0 issues,
   8s,
   the placeholder
stub again);
   BI4PBV repaired (38 issues,
   37 accepted,
   36 resolved,
   5
findings,
   1437s);
   Barron12312 repaired (49 issues,
   46 accepted,
   46
resolved,
   2 findings,
   1590s).
   The per-entry cap did its job -- three
entries packed before the soft budget instead of one blocking the run.
7/92 settled.
   Run 007 launched.
USER PICK (2026-07-24):
   the large-entry tail is solved with SLICE-LEVEL
RESUMABILITY (chosen over a very high atomic cap or reducing per-slice
cost).
   Plan:
   inject an optional slice cache into repairTranslation (keyed
by deterministic slice content hash),
   persist each completed slice,
   resume
across runs;
   driver supplies a disk-backed cache under the runs dir.
   Build
next,
   additive so the bulk keeps running on current code meanwhile.
RESUMABILITY LANDED (2026-07-24,
   commit `bfc3f6449`):
   `SliceCache`
(`{ resumed: ReadonlyMap<hash, ChunkRepairOutcome>, persist(key,
serialized) }`) injected optionally into `repairTranslation`;
   the slice
loop keys each slice by `hashContent([chunkIndex, sourceText,
targetText])`,
   reuses a resumed outcome (zero model calls) or computes then
persists it.
   Driver helper `corpus-run/slice-cache-store.ts`
(openSliceCache / discardSliceCache) writes one JSON per slice under
`node_modules/.monochromatic/translation-repair-runs/slice-cache/<entry>/`,
guards each file (a half-write recomputes),
   and drops the dir on settle.
   A
huge entry (aiyysk 77 slices) now completes over several 90-min attempts
instead of losing all work each abort -- it deprioritizes between attempts
but caches ~16 more slices each.
   Persist takes a SERIALIZED STRING,
   not the
outcome object,
   because the repo BANS disabling
prefer-readonly-parameter-types (rule no-disable-prefer-readonly-
parameter-types);
   a string param sidesteps it,
   the pipeline owns
serialization,
   the driver writes bytes.
   Additive and result-preserving
(unit test:
   the resumed run makes 0 model calls and reproduces the fresh
result),
   so settled artifacts stay valid and NO restart.
RESTART NOTE (IMPORTANT):
   the slice-cache stores repairChunk OUTCOMES,
   so a
PIPELINE change makes it stale.
   The restart/wipe routine now clears THREE
things together:
   `artifacts/`,
   `attempts.json`,
   AND `slice-cache/`.
Content-hash keys already self-invalidate on a slicing change,
   but wipe all
three anyway.
   Verified format/lint/types/unit 0 and `--plan` runs.
   Run 007
was already in flight on pre-resumability code (per-entry cap,
   no cache);
it finishes normally,
   then run 008 launches with resumability.

## Pass 6 runs 007 to 010 with resumability live

PASS 6 RUN 007 (2026-07-24,
   tip `c4cfe103b`,
   3119s wall):
   1 dispatched,
   1
completed,
   0 failed.
   Chinatsu_Suzuki repaired (77 issues,
   77 accepted,
   all
77 resolved,
   4 findings,
   3119s ~52 min) -- an entry that would have hit
the old 45-min per-run cap but fits the 90-min per-entry budget;
   the cap
fix earned its keep.
   8/92 settled.
   Run 008 launched WITH resumability
active (tip after the resumability commits).
PASS 6 RUN 008 (2026-07-24,
   tip `ed5936126`,
   3079s wall,
   resumability
live):
   1 dispatched,
   1 completed,
   0 failed.
   Considerate_cat repaired (50
issues,
   41 accepted,
   40 resolved,
   11 findings,
   3079s).
   Resumability
cleanup verified in production:
   the slice-cache dir is empty after the run
because the settled entry's cache was discarded as designed.
   9/92 settled.
Run 009 launched.
PASS 6 RUN 009 (2026-07-24,
   tip `7c3cca379`,
   1634s wall):
   1 dispatched,
   1
completed,
   0 failed.
   CuspariaKLSY repaired (74 issues,
   70 accepted,
   all 70
resolved,
   6 findings,
   1634s).
   10/92 settled.
   Run 010 launched.
PASS 6 RUN 010 (2026-07-24,
   tip `d1d2f2c52`,
   2505s wall):
   2 dispatched,
   2
completed,
   0 failed.
   CutOceanHeyFis1 repaired (19 issues,
   19 accepted,
   18
resolved,
   5 findings,
   1182s);
   DarlinChit repaired (46 issues,
   40 accepted,
40 resolved,
   20 findings,
   1323s).
   12/92 settled.
   (Run 010's first launch
was killed by a transient harness hiccup right after the run-009 commit
landed;
   relaunched standalone.
   Going forward:
   commit foreground,
   launch
corpus-pass as its own background command.)
   Run 011 launched.
