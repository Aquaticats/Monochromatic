# Translation repair history, first handover document: passes 4 and 5 with the false blocks and the lockstep slicing fix, 2026-07-24

The false non-translation block on AkiraComplex and the vote threshold that fixed it,
the runs of pass 5,
and the diagnosis of the Arita block as a slice misalignment,
with the lockstep slicing fix and a survey of how many entries it touched.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Pass 4 run 002: false block on AkiraComplex and the vote threshold fix

PASS 4 RUN 002 (2026-07-24,
   tip `92f7b2c55`,
   2110s wall,
   soft budget hit):
processed 2 of 92,
   artifacts 2/92.
   `AmbeR_the_anpa` repaired (61 issues,
   58
accepted,
   58 resolved,
   0 findings,
   2028s ~34min ALONE,
   consuming the whole soft
budget;
   the top-of-loop soft check stopped new entries after it).
   `AkiraComplex`
blocked-non-translation (0 issues,
   4 findings,
   62s) is a VERIFIED FALSE BLOCK:
its en page is a faithful translation of the zh (checked directly against the
pinned corpus).
   Findings were `empty-quote (source)` x2,
   `non-translation votes
stand (2/7 heard); slice unchanged`,
   `non-translation dominance (561 of 590
target chars)`.
   Root cause:
   only 2 of 7 critics were HEARD on the dominant
slice and both voted non-translation,
   meeting the ABSOLUTE
`NON_TRANSLATION_BLOCK_VOTES=2` threshold (`non-translation-evidence.ts`);
   5
silent critics gave no counter-signal,
   so a bare 2 votes blocked despite no real
ensemble agreement.
   Likely trigger:
   the page opens with an English epigraph that
is English in BOTH zh and en source,
   so a slice reads as "source == target,
   not
a translation" to a critic.
   HIGH-CONFIDENCE ISSUE,
   fix deferred for careful
calibration (do not just lower/raise the constant blindly):
   the severe block
(discards all repair,
   returns input) must require genuine ensemble agreement,
not a bare count a low-participation slice can satisfy.
   Candidate fixes,
   each
with a tension to resolve against the KNOWN TRUE-POSITIVE case (zh cat story vs
"Meow meow meow":
   GLM + gpt-oss + Qwen,
   i.e. 3 of 7,
   all flagged
non-translation,
   one failing to anchor):
   (1) require a MINIMUM critics-heard
count on the slice before any block (2/7 heard is too few to make a severe call;
degrade instead) -- cleanest,
   targets the failure mode directly,
   needs the
minimum chosen so the 3/7 true case still blocks when those 3 are among the
heard;
   (2) require non-translation votes as a fraction of the FULL roster
treating silent critics as not-non-translation (e.g. >= 3) -- must not exceed 3
or it breaks the true case;
   (3) both.
   The English-epigraph-in-both trigger is a
second,
   orthogonal seam (a slice whose source and target are identical English
should never count as non-translation evidence).
   NEXT ACTION:
   design and land
this with full context,
   add unit tests over the participation cases,
   validate on
`AkiraComplex` (expect:
   no longer blocked) via `sentinel-probe -- AkiraComplex`
plus the true-positive fixture,
   then restart the pass.
   Accumulation is PAUSED
(run 003 not launched) because a fix+restart discards further runs;
   resume only
if choosing progress-under-current-pipeline over the fix.
USER PICK (2026-07-24):
   "Always land the fix now then restart."
   This is a
STANDING refinement of the improve-and-restart directive:
   context pressure is
NOT a reason to defer a verified high-confidence fix;
   land it,
   do not park it for
a fresh session.
   Recorded so future sessions do not re-offer "defer".
FIX LANDED (2026-07-24,
   commit `342f9caa5`):
   `NON_TRANSLATION_BLOCK_VOTES` raised
2 -> 3 in `non-translation-evidence.ts`.
   Three wire votes is genuine ensemble
agreement and,
   because three votes cannot come from fewer than three critics
heard,
   folds a participation floor into the count so a low-participation slice
(AkiraComplex's 2/7) can never block;
   three is the observed true-positive floor
(cat/"meow" drew three) and errs safe (a missed block attempts repair with issues
still surfaced;
   a false block discards a faithful translation whole).
   The block
decision was extracted from an inline expression in `repair-chunk.ts` into a
named,
   exported `nonTranslationVotesStand({votes, contradicted})` with regression
unit tests (2 votes below floor do not stand;
   3 uncontradicted stand;
   3
contradicted do not).
   `downgradeCount` moved to sibling `downgrade-count.ts` to
keep `repair-chunk.ts` under the 300-line budget.
   Verified:
   build,
   format 0/0,
lint,
   types,
   unit tests all green.
   Live sentinel-probe on AkiraComplex is the
final confirmation before restart.
   This makes the restarted pass a NEW pass
(pipeline behavior changed);
   prior pass-4 artifacts are discarded.
FIX CONFIRMED LIVE (2026-07-24):
   `sentinel-probe -- AkiraComplex` returned
status=repaired (21 issues,
   21 accepted,
   1 finding,
   421s),
   up from
blocked-non-translation:
   the false block is gone end-to-end.
   Artifacts and
`attempts.json` wiped for a clean restart.
   PASS 5 RUN 001 launched on tip
`b3fdf6e4c` (log `pass5-run-001.log`);
   this is the current accumulation pass
under the three-vote non-translation block.
   Loop continues per task 30:
   record
each run's tallies content-free,
   commit,
   launch the next,
   until all 92 settle,
landing any further verified high-confidence fix immediately (restarting) per
the standing rule.

## Pass 5 runs 001 to 005

PASS 5 RUN 001 (2026-07-24,
   tip `b3fdf6e4c`,
   1906s wall,
   soft budget hit):
2 dispatched,
   2 completed,
   0 failed.
   Acheron repaired (56 issues,
   55
accepted,
   54 resolved,
   18 findings,
   1464s);
   AkiraComplex repaired (13
issues,
   13 accepted,
   12 resolved,
   1 finding,
   442s).
   AkiraComplex is the
headline:
   the three-vote block holds in the full pass exactly as the probe
predicted -- the once-false-blocked slice now repairs cleanly,
   no
regression elsewhere.
   Acheron alone ate the 25-min soft budget,
   so the
top-of-loop check stopped new entries after it;
   the two artifacts persist,
attempts.json carries {Acheron:1,
   AkiraComplex:1}.
   No new high-confidence
fix surfaced;
   loop continues,
   launching run 002 on the same tip.
PASS 5 RUN 002 (2026-07-24,
   tip `94b031cae`,
   2088s wall,
   soft budget hit):
1 dispatched,
   1 completed,
   0 failed.
   AmbeR_the_anpa repaired (49 issues,
45 accepted,
   44 resolved,
   6 findings,
   2088s) -- a single large document
that overran the 25-min soft budget on its own,
   so no second entry
dispatched.
   3/92 settled.
   No new fix surfaced;
   run 003 launched on tip
`94b031cae` (same,
   since only the handover moved).
PASS 5 RUN 003 (2026-07-24,
   tip `0384097b7`,
   1806s wall,
   soft budget hit):
1 dispatched,
   1 completed,
   0 failed.
   Aniloviraw repaired (26 issues,
   26
accepted,
   25 resolved,
   13 findings,
   1806s) -- the once-false-blocked
divergence-heavy pair,
   repairing cleanly again.
   One transient event:
   five
of seven critics (Qwen3.6-27B,
   Kimi-K2.7-Code,
   MiniMax-M3,
   Nemotron-3,
gpt-oss-120b) hit the 240s deadline together on a single slice at
10:14:54Z;
   that slice heard only two critics,
   the pipeline degraded
gracefully via quorum,
   and the entry still repaired.
   Reads as an API-side
slowdown burst,
   not a code fault -- logged,
   no fix triggered.
   4/92 settled.
Run 004 launched on tip `0384097b7`.
PASS 5 RUN 004 (2026-07-24,
   tip `5f60a1b55`,
   2540s wall,
   near hard cap):
1 dispatched,
   1 completed,
   0 failed.
   Anilovr repaired (95 issues,
   95
accepted,
   ALL 95 resolved,
   34 findings,
   2540s) -- the largest document
yet by issue count,
   running ~42 min,
   just under the 45-min hard cap.
   Two
critic timeouts (Nemotron-3,
   gpt-oss-120b) on one slice,
   again absorbed
by quorum with no effect on the outcome.
   A perfect 95/95 accept-and-
resolve is a strong signal but exactly the kind of number the milestone-
three human grade exists to check,
   not to trust on its own.
   5/92 settled.
Run 005 launched on tip `5f60a1b55`.
PASS 5 RUN 005 (2026-07-24,
   tip `20a66e58b`,
   1769s wall,
   soft budget hit):
3 dispatched,
   3 completed,
   0 failed.
   BI4PBV repaired (42 issues,
   38
accepted,
   37 resolved,
   16 findings,
   1498s).
   ArtsEpiphany unchanged (0
issues,
   15s) -- correct:
   a 120-char placeholder stub whose desc is blank
by intent and whose source equals its target,
   nothing to repair.
   Arita
BLOCKED-non-translation (0 issues,
   14 findings,
   255s) -- the FIRST block
under the three-vote regime,
   and it demanded investigation before the
loop could continue.

## Arita diagnosis, lockstep slicing fix and its blast radius

ARITA DIAGNOSIS (2026-07-24):
   a FALSE block,
   but from a slice-alignment
defect,
   not the vote threshold.
   Arita is a genuine translation (rich zh
biography,
   faithful en).
   Deterministic node dump proved both sides carry
exactly 13 nodes corresponding 1:1,
   yet `subdivideChunkPair` mis-paired
them with a one-paragraph drift,
   so critics correctly read each mismatched
slice as non-translation and 6-7 of 7 voted -- genuine ensemble agreement
on genuinely mispaired input.
   Root cause:
   the slicer grouped each side
independently with different budgets (source scaled ~150,
   target 400);
small adjacent nodes merged at different indices per side,
   run counts
diverged (12 vs 11),
   and the character-fraction merge pulled an extra
source run into slice 0,
   shifting every later slice by one.
   This is the
common case (translations preserve paragraph structure),
   so the defect
likely mis-sliced many entries subtly;
   Arita was pathological because the
drift made EVERY slice a mismatch.
ARITA FIX (2026-07-24,
   commit `7a5117727`):
   `groupNodesLockstep` -- when
both sides carry equal node counts,
   group them together,
   extending a slice
to the next shared index only while BOTH sides stay within budget,
   so
slice N always holds the same node indices on both sides.
   Genuine
paragraph-count mismatch still falls back to the existing monotone merge.
Correct by construction for the equal-count case;
   surgical (unequal counts
untouched).
   Verified deterministically:
   an engineered equal-count marker
fixture drifted under the old code (src[M0] vs tgt[M0,M1]) and pairs 1:1
under the new;
   the real Arita content now slices 1:1 (简介/Introduction,
intro/intro,
   band/band ... every slice a true pair).
   Unit test
`pairs equal node counts in lockstep without off-by-one drift (Arita
regression)` added;
   format/lint/types/unit all green.
   Live confirmation:
`sentinel-probe -- Arita` returned status=repaired (123 issues,
   114
accepted,
   7 findings,
   4110s) with 0 non-translation votes on every critic
stage -- the false block gone end-to-end.
ARITA FIX BLAST RADIUS (2026-07-24,
   deterministic corpus survey,
   zero
quota):
   across the 92 usable entries,
   182 of 284 aligned chunk-pairs
(64%) carry equal node counts and so take the lockstep path;
   replaying the
old independent-budget grouping,
   68 of those 182 pairs -- spread over 46
of the 92 entries -- had divergent per-side run boundaries,
   i.e. the old
code actually mis-sliced them.
   Most drifted entries still REPAIRED before
(Acheron,
   AkiraComplex,
   BI4PBV are in the drifted set) because partial
drift only mispairs some slices;
   Arita was the pathological all-slices
case that blocked.
   So the fix corrects slicing on 68 pairs across half the
corpus,
   but the ONLY confirmed end-to-end outcome change is Arita
(block->repair);
   alignment is now provably more correct on those pairs,
while any issue-set or quality effect is unmeasured and waits on the
milestone-three human grade.
   Survey scripts in the session scratchpad
(nodecount-survey.mjs,
   drift-survey.mjs) reproduce the counts.
