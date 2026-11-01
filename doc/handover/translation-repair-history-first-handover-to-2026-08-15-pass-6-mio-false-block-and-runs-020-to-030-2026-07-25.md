# Translation repair history, first handover document: pass 6 false block on Mio and runs 020 to 030, 2026-07-25

The fourth false non-translation block,
on Mio,
and the anchor veto that removed it,
followed by the run records from 021 to 030 that filled the size bands,
with the advisor call and the decision point on declaring the bar met.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Pass 6 run 020: false block on Mio and the anchor veto

PASS 6 RUN 020 (2026-07-25,
>tip `55b0dd444`,
>2257s wall ~38 min):
>1 settled,
Mio status=blocked-non-translation (93 accepted issues,
>0 resolved).
>FALSE
BLOCK discovered and fixed (commit `398007d4c`).
>Mio's en page is a faithful
translation that ALSO translates its embedded images -- WeChat chat logs,
Twitter posts,
>and a final chat that the zh page.md carries only as
`<PhotoScroll>` photos,
>so that translated-image text has NO zh-markdown
counterpart.
>Its slices correctly drew non-translation votes (chunks
5,6,8-12,
>all 7/7),
>and at 5342 of 7778 target chars (69%) they tripped the
bare char-majority dominance rule and discarded the whole document,
>even
though chunks 0,1,2,7 are clean repaired translations.
>VERIFIED SYSTEMIC via
a scan of all 22 artifacts:
>Futajuhuacha and Huasheng each carry 8 standing
slices too (same as Mio) and only escaped because their standing chars
stayed under half;
>every entry with standing slices also has clean chunks
(Mio 5,
>Futajuhuacha 11,
>Huasheng 13,
>Dethelly 22,
>Kotori 10).
>This is the
4TH real-corpus non-translation block this session (Aniloviraw,
>AkiraComplex,
Arita,
>Mio) and ALL FOUR WERE FALSE;
>the only true positive ever is the
invented cat/"meow" pair.
>Discriminator:
>every false block held confirmed
good-translation content;
>the true positive held none.
>FIX (advisor-guided,
document-dominance layer ONLY):
>added `sliceAnchorsTranslation` (a
non-standing slice carrying an accepted target-anchored content critique) and
`assessNonTranslationDominance` now vetoes the block whenever ANY slice
anchors translation -- a threshold-free discriminator from 4-false-vs-1-true,
keeping the calibrated err-toward-not-blocking direction.
>Because the change
only flips the block branch and only toward NOT blocking,
>every "repaired"
entry is provably unchanged;
>ONLY Mio is stale (no full wipe,
>slicing
untouched so caches stay valid).
>Deterministic tests pass (Mio-shape
dominance regression + six anchor-probe cases);
>format/lint/types/build 0.
Live Mio re-validation CONFIRMED (sentinel-probe `bcgzdo82q`,
>~47 min):
PROBE Mio status=repaired (84 issues,
>82 accepted,
>45 findings) -- flipped
blocked->repaired exactly like Arita.
>The log shows the correct shape:
>chunks
0,1,2,3,4,7 repaired (clean anchors),
>chunks 5,6,8-15 still ship unchanged
per-slice (10 standing,
>even MORE than run 020's 8,
>yet no document block
because the anchors veto it).
>Stale Mio.json deleted and Mio's attempts entry
reset to 0 so it re-settles as a fair medium candidate during accumulation;
settled dropped to 21/92 (9 small / 6 medium / 6 large).
>Accumulation resumed
at run 021 (NO pass restart or wipe -- only Mio was stale).
>Surface the
4/4-false-block pattern in the milestone writeup -- it is a real finding about
this feature's value on this all-real-translation corpus.

## Pass 6 runs 021 to 030

PASS 6 RUN 021 (2026-07-25,
>tip `5baaa37c2`,
>2829s wall ~47 min):
>1 settled,
22/92.
>Mio RE-SETTLED status=repaired (88 issues,
>87 accepted,
>ALL 87
resolved,
>56 findings) under the anchor-veto fix -- the false block is gone
in the actual artifact pool,
>not just the probe.
>Picked first because its
attempts were reset to 0.
>Medium settled 6->7.
>Bands now 9 small / 7 medium
/ 6 large (need ~1 small,
>~3 medium,
>~4 large).
>Run 022 launched.
PASS 6 RUN 022 (2026-07-25,
>tip `3e905b1f3`,
>1864s wall ~31 min):
>1 settled,
23/92.
>Mizuki_Yuuki repaired (medium,
>59 issues,
>58 accepted,
>58 resolved,
>6
findings);
>one run,
>no cap.
>Medium settled 7->8.
>Bands now 9 small / 8
medium / 6 large (need ~1 small,
>~2 medium,
>~4 large).
>Large lags because its
bigger entries (9-40KB) need multi-run resume.
>Run 023 launched.
PASS 6 RUN 023 (2026-07-25,
>tip `1d670bc17`,
>2596s wall ~43 min):
>1 settled,
24/92.
>MushroomGuuuu repaired (MEDIUM band,
>1934B zh source;
>69 issues,
>62
accepted,
>62 resolved,
>8 findings,
>7 chunks);
>one run,
>exceeded 25-min soft
budget but settled within 90-min hard cap.
>Medium settled 8->9.
>Bands now 9
small / 9 medium / 6 large (small & medium at the ~10 bar;
>large needs ~4).
Advisor call (Opus 4.8):
>do NOT add ordering to force large -- runs are free
background work (user waived quota),
>6 large already yields ample large-band
accepted issues (MushroomGuuuu alone 62 >> the ~17 a stratified 50-sample
needs),
>so large=~10 is document-diversity polish,
>not sample-sufficiency;
also `corpus-pass` takes only `--plan`,
>no entry-id targeting arg,
>so forcing
large would mean new code through the lint gauntlet for negative ROI.
>Decision:
run naturally (eligible order interleaves medium/large;
>resume-first finishes
any that abort;
>no stall risk),
>and redirect ACTIVE effort to tasks 31/32
(judge crosscheck + stratified sample tooling) which are unblocked against the
24 already-settled entries -- only the final draw waits on band fill.
>Run 024
launched.
PASS 6 RUN 024 (2026-07-25,
>tip `87c13c925`,
>hit the 90-min HARD cap):
>0
settled,
>still 24/92.
>NIGHT81473140 (LARGE,
>12301B zh,
>the 9-40KB tail)
aborted at the hard cap after completing 22 cached slices;
>status=ERROR
aborted=true,
>attempts=1.
>This is the resume-first path working as designed --
the 22 finished slices persist,
>so run 025 resume-first-picks NIGHT81473140 and
continues from slice 22+ rather than restarting (degrade-and-persist + the
cap-abort-completes->=1-slice guarantee make this monotonic).
>First entry to
exercise the hard cap this pass;
>the biggest large entries will need multi-run
resume.
>Run 025 launched to resume it.
PASS 6 RUN 025 (2026-07-25,
>tip `72128f17b`,
>4978s wall ~83 min):
>1 settled,
25/92.
>NIGHT81473140 RESUMED and settled repaired (LARGE,
>12301B;
>123 issues,
105 accepted,
>105 resolved,
>42 findings) -- run 024's 22 cached slices skipped
with zero model calls,
>the remaining 19 of 41 total processed to settlement
inside the hard cap.
>First PRODUCTION proof of the multi-run-resume path
settling a hard-cap-aborted large entry end-to-end (earlier resume validation
was Dethelly,
>which had not hit the cap).
>Large settled 6->7.
>Bands now 9 small
/ 9 medium / 7 large (large needs ~3 more for the ~10 bar).
>Run 026 launched.
PASS 6 RUN 026 (2026-07-25,
>tip `5b0f818877`,
>2731s wall ~46 min;
>a first
launch was KILLED ~2 min in on Proselyte093 with 0 slices done -- intermittent
non-resource kill,
>empty cache dir removed,
>relaunched):
>1 settled,
>26/92.
SS3B_0016 repaired (MEDIUM,
>2040B;
>139 issues,
>135 accepted,
>135 resolved,
>6
findings).
>Medium settled 9->10,
>hitting the ~10 bar.
>This is the advisor-
sanctioned natural-ordering medium over-coverage (extra medium is data,
>not
waste;
>no ordering change to force large).
>Bands now 9 small / 10 medium / 7
large (large needs ~3 more).
>Run 027 launched.
PASS 6 RUN 027 (2026-07-25,
>tip `bbaea33dd`,
>hit the 90-min HARD cap):
>0
settled,
>still 26/92.
>Susiethegamer (LARGE,
>4557B,
>heavily sliced -- 19 slices
across 2 chunk pairs) aborted at the hard cap ONE slice short:
>18 of 19 cached.
Run 028 resume-first-picks it and should finish the last slice plus document
assembly to settle quickly.
>Run 028 launched.
PASS 6 RUN 028 (2026-07-25,
>tip `0bf98e61a`,
>5605s wall):
>1 settled,
>27/92.
Susiethegamer RESUMED and settled repaired in ~3.4 min (its last 1 of 19 slices
+ document assembly;
  187 issues,
  179 accepted,
  179 resolved,
  56 findings) --
clean proof the resume path finishes a one-slice-short hard-cap abort cheaply.
Large settled 7->8.
  The run then started TianqiChen666 (LARGE,
  6746B,
  23 slices
/ 3 chunk pairs),
  which hit the hard cap at 16 of 23 cached;
  run 029 resumes it.
Bands now 9 small / 10 medium / 8 large (large needs ~2 more).
  Run 029 launched.
PASS 6 RUN 029 (2026-07-25,
  tip `d39a8b7d8`,
  1796s wall ~30 min):
  1 settled,
28/92.
  TianqiChen666 RESUMED and settled repaired (LARGE,
  6746B;
  remaining 7 of
23 slices + document assembly;
  256 issues,
  248 accepted,
  241 resolved,
  47
findings).
  Large settled 8->9.
  Bands now 9 small / 10 medium / 9 large -- all
three bands at/around the ~10 bar;
  large needs ~1 more to reach 10.
  Run 030
launched;
  once large hits ~10 the ~10/10/10 coverage bar (task 30) is met and
the FINAL draw (`draw-sample -- --final`,
  task 32) runs once against the filled
pool.
PASS 6 RUN 030 (2026-07-25,
  tip `8419cf316`,
  3121s wall ~52 min):
  1 settled,
29/92.
  Toka_ls repaired (MEDIUM,
  3660B -- 26 bytes under the 3686 large cut;
  74
issues,
  72 accepted,
  71 resolved,
  21 findings).
  Medium settled 10->11.
  Bands now
9 small / 11 medium / 9 large.
  DECISION POINT (task 30):
  small is driver-
deprioritized and pinned at 9 (it only grows once non-small is exhausted,
  so it
will NOT reach a literal 10 by natural accumulation),
  large just reached 9,
medium over-covers at 11.
  If small=9 counts as "~10" (it must,
  given the
deprioritization),
  large=9 counts equally -- so 9/11/9 is a defensible "~10/10/
10".
  Advisor consulted on whether to declare the bar met + run the FINAL draw,
or push one more for large=10.
  No run 031 launched pending that call.
