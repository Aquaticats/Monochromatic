# Translation repair history, first handover document: milestone two derivability probe, editor A/B and closure, 2026-07-18

The declaration of 2026-07-18 that milestone two was complete,
with its final numbers,
the probe that separates misses the source cannot support from real editor shortfalls,
the A/B of the clause-enumeration rule,
the last runs,
the promotion of the rule and the closing run that ended the accumulation loop,
and the notes left at the compaction after run 23.
Its last item says a mirror of the current state is in the task state section;
that section is now `translation-repair-history-first-handover-to-2026-08-15-task-state-checklist-of-the-early-tasks.md`.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Milestone two declared complete

MILESTONE TWO DECLARED COMPLETE (2026-07-18,
user directive
"Promote the clause-enumeration rule and declare milestone two").
Final accumulated numbers over 20 judge-graded runs (22 to 41),
repairable universe:
judge strict 80/96 (0.83);
PROBE-ADJUSTED
EFFECTIVE 94/96 (0.98);
lenient 92/96 (0.96);
detection 166/174
(0.95,
the four falses being correct refusals of not-derivable
content);
lexical (retired comparison) 59/96 (0.61).
Every miss is
attributed:
embellishment-capped partials,
correct refusals of
unfounded content,
and TLL1122's two derivable seeds,
one of which
the now-promoted clause-enumeration rule (commit `b6967cbc9`)
reproducibly fixes.
The accumulation loop is CLOSED;
run 42 (in
flight at declaration,
old baseline prompt) gets recorded as the
final baseline-era run when it lands,
and no run 43 follows.

## Derivability probe and editor rule addendum built

USER DIRECTION (2026-07-18,
  interactive):
  pursue BOTH the
derivability probe AND the editor calibration A/B,
  probe first.
Built and committed while run 37 was in flight:
- Derivability probe (commit `2951e9b42`):
  derivability-wire.ts +
  derivability-probe.ts ask the judge ensemble whether each deleted
  sentence is fully derivable from the zh source
  (derivable/partially-derivable/not-derivable).
  UPPER-median
  resolution (opposite of the restoration judge) rounds splits
  toward derivable:
  the probe can only EXCUSE a partial,
  so the
  excuse carries the burden of proof.
  Unjudged defaults derivable.
  Driver `derivability-probe-run.ts` (scratchpad) probes all five
  stable-partial entries with restored siblings as controls;
  run it
  BETWEEN benchmark runs (one stream per model is fastest).
- Editor rule addendum (commit `c767a550a`):
  RepairModels.editorRuleAddendum threads one extra rule line into
  the editor system prompt (composed from named blocks,
  never
  string surgery).
  A/B plan:
  baseline vs clause-enumeration rule on
  the stable-partial entries,
  judge verdicts on target seeds
  compared;
  only meaningful for seeds the probe rules derivable.

## Run 37, probe results and miss attribution

run 37 (2026-07-18,
  1531 s):
  judge 3/6 (0.50,
  three partials),
lexical 3/6,
  detection 6/6;
  TLL1122,
  DarlinChit,
  keyword233
repaired (three dispatched,
  three budget-skipped).
  TLL1122 (one of
the two original long-needle miss entries) re-drew at last:
  BOTH
seeds judged partial.
  keyword233 seed 0 reproduced its run 34
partial (0.40 then 0.47 coverage across independent runs).
DarlinChit fully restored.
  Stable-partial set now SIX seeds over
five entries:
  Acheron 0,
  CuspariaKLSY 1,
  Anilovr 0,
  keyword233 0,
TLL1122 0 and 1 (akasa_musha's two included makes eight probe
targets).
  Enriched analysis over all six shows the one signature:
scattered within-sentence paraphrase,
  0.40 to 0.69 coverage,
  no
sentence dropped whole.
JUDGE ACCUMULATED (repairable universe):
  62/76 (0.82) over 16 runs
(22 to 37).
  Lexical over the same runs 48/76 (0.63).
  Detection
148/154 (0.96).
DERIVABILITY PROBE RESULTS (2026-07-18,
  144 s total,
  ~14 quota
units,
  every quorum 3/3,
  every vote unanimous):
- Six of eight stable-partial seeds are PARTIALLY-DERIVABLE
  (Acheron 0,
  CuspariaKLSY 1,
  Anilovr 0,
  keyword233 0,
  akasa_musha
  0 and 1):
  the original EN translation embellished beyond the zh
  source,
  so partial restoration is those seeds' correct CEILING.
- TLL1122 seeds 0 and 1 are DERIVABLE yet only partially restored:
  the only genuine editor shortfall;
  the A/B target.
- wangzihao980's two ABSENT seeds (run 22) are NOT-DERIVABLE:
  the
  deleted sentences have no zh support,
  so the editor rule "Never
  introduce content the ORIGINAL does not support" makes refusal
  CORRECT;
  run 22's "genuine repair failure" reading is retracted.
- All four restored-sibling controls probed derivable:
  the probe
  discriminates rather than excuses.
MISS ATTRIBUTION through run 37 (76 judged seed-results):
  62
restored;
  10 partials on partially-derivable seeds (ceiling met);
2 partials on derivable TLL1122 seeds (real shortfall);
  2 absents
on not-derivable wangzihao980 seeds (correct refusal).
PROBE-ADJUSTED EFFECTIVE RATE:
  74/76 (0.97).
  Strict rate over
winnable (derivable) seeds:
  62/64 (0.97).
ERRATUM:
  run 29 to 37 notes said accumulated lenient stayed 1.0;
wrong,
  run 22's two wangzihao980 absents make lenient through run
37 74/76 (0.97).
  Per-run lenient scorecards were correct;
  only the
accumulated claim in these notes was wrong.

## Editor A/B on the clause-enumeration rule

EDITOR A/B,
  VARIANT ARM 1 (2026-07-18,
  937 s):
  TLL1122 with the
clause-enumeration editorRuleAddendum.
  Seed 1 FLIPPED partial ->
RESTORED (unanimous,
  on 5/12 lexical words:
  meaning-complete
rewording only the zh-anchored judge credits).
  Seed 0 stayed
partial.
  Word coverage identical across arms (0.47 and 0.50),
  so
the flip is purely semantic.
  Retry-to-quorum recovered a 4-critic
forfeit and a whole-panel 7-voice forfeit inside this run.
The rule text lives in scratchpad `editor-ab-run.ts` as
CLAUSE_ENUMERATION_RULE.
  Variant arm 2 (replicate) launched to
confirm the n=1 flip before drawing conclusions.
EDITOR A/B,
  VARIANT ARM 2 (2026-07-18,
  591 s):
  reproduces arm 1
EXACTLY:
  seed 1 restored (unanimous),
  seed 0 partial,
  identical
lexical counts (6/15,
  5/12).
A/B VERDICT:
  the clause-enumeration rule reliably flips TLL1122
seed 1 (2/2 variant arms vs partial at baseline);
  TLL1122 seed 0 is
now the accumulation's ONLY unresolved derivable seed,
  resisting
both arms at 0.47 coverage.
  Why it resists is open;
  candidate
probes:
  per-clause derivability of that one sentence,
  or a
different editor model on that entry.
DECISION FOR USER:
  promote CLAUSE_ENUMERATION_RULE into the
baseline editor prompt?
  Evidence:
  2/2 reproducible win on the one
targeted seed,
  no observed regression (arm seeds elsewhere weren't
run).
  Promotion changes the measured pipeline mid-accumulation,
  so
it awaits explicit direction;
  the loop continues on the BASELINE
prompt meanwhile for measurement continuity.

## Runs 38 to 41

run 38 (2026-07-18,
  1598 s):
  judge 4/4 (1.0),
  lexical 3/4,
detection 4/4;
  xixi_yuexi and homoyamakaze (both first-time
entries) fully repaired (two dispatched,
  four budget-skipped).
homoyamakaze includes a 16-word needle restored at 11/16 and
another under-credit (3/10 words,
  unanimous judge=restored).
JUDGE ACCUMULATED (repairable universe):
  66/80 (0.83) over 17 runs
(22 to 38).
  Lexical 51/80 (0.64).
  Detection 152/158 (0.96).
Lenient 78/80 (0.98).
  Probe-adjusted effective 78/80 (0.98).
run 39 (2026-07-18,
  2281 s):
  judge 6/6 (1.0),
  lexical 5/6,
detection 6/6;
  Katerina (first draw),
  homoyamakaze,
  a2581911655
all fully repaired (three dispatched,
  three budget-skipped).
  Both
repeats reproduced earlier verdicts;
  a2581911655 seed 0 repeated
its under-credit (6/18,
  unanimous judge=restored).
JUDGE ACCUMULATED (repairable universe):
  72/86 (0.84) over 18 runs
(22 to 39).
  Lexical 56/86 (0.65).
  Detection 158/164 (0.96).
Lenient and probe-adjusted effective both 84/86 (0.98).
run 40 (2026-07-18,
  1504 s):
  judge 6/6 (1.0),
  lexical 2/6,
detection 6/6;
  SU5ZI2MO1,
  luxuanwen3,
  coin all fully repaired
(three dispatched,
  three budget-skipped).
  LUXUANWEN3 at last:
  one
of the two original pre-judge long-needle miss entries dispatched
for the first time in the judge era,
  and BOTH seeds judged restored
on extreme under-credits (2/13 and 2/8 lexical words,
  unanimous).
Its historic lexical-era "miss" was faithful rewording;
  with
TLL1122 probed and A/B'd,
  every pre-judge question is now closed.
SU5ZI2MO1's 20-word needle restored again (third time).
JUDGE ACCUMULATED (repairable universe):
  78/92 (0.85) over 19 runs
(22 to 40).
  Lexical 58/92 (0.63).
  Detection 164/170 (0.96).
Lenient and probe-adjusted effective both 90/92 (0.98).
run 41 (2026-07-18,
  1919 s):
  judge 2/4,
  lexical 1/4,
  detection 2/4;
wangzihao980 and a2581911655 repaired (two dispatched,
  four
budget-skipped).
  wangzihao980's first re-draw since run 22
REPRODUCES it exactly:
  both seeds absent,
  detection false.
  With the
probe's not-derivable verdict this is the correct-refusal case
confirming across independent runs,
  and the detection "misses" are
the same coin:
  nothing is actually missing relative to zh,
  so the
panel rightly accepts no issue there.
  a2581911655 third draw,
  both
restored again.
JUDGE ACCUMULATED (repairable universe):
  80/96 (0.83) over 20 runs
(22 to 41).
  Lexical 59/96 (0.61).
  Detection 166/174 (0.95,
  the four
wangzihao980 falses being correct refusals in disguise).
  Lenient
92/96 (0.96).
  Probe-adjusted effective 94/96 (0.98).

## Promotion of the clause-enumeration rule, declaration and closing run

PROMOTION AND DECLARATION (2026-07-18,
  user directive):
  the
clause-enumeration rule is now a baseline editor prompt rule
(commit `b6967cbc9`;
  the editorRuleAddendum plumbing stays for
future calibration experiments),
  and MILESTONE TWO IS DECLARED
COMPLETE with the final numbers recorded at the top of the resume
block.
  Run 42,
  launched before the promotion,
  ran the old baseline
and closes the baseline era;
  no run 43.
run 42,
  CLOSING RUN (2026-07-18,
  1523 s,
  pre-promotion prompt):
judge 2/4,
  lexical 2/4,
  detection 4/4;
  chunchun_yudong (first
draw) both restored;
  SS3B_0016 (first draw) both partial,
  and an
immediate derivability probe ruled both seeds PARTIALLY-DERIVABLE
(unanimous,
  19 s):
  embellishment-capped,
  misses attributed,
  the
dominant pattern to the end.
WHOLE-ACCUMULATION TOTALS including the closing run,
  21 runs (22
to 42),
  exactly 100 seed-results:
  judge strict 82/100 (0.82);
PROBE-ADJUSTED EFFECTIVE 98/100 (0.98);
  lenient 96/100 (0.96);
detection 170/178 (0.96);
  retired lexical 61/100 (0.61).
  LOOP
CLOSED.
NEXT AFTER CLOSURE:
  package completeness per PKG (README,
  exported
API surface review,
  test coverage over every exported path) awaits
user direction,
  as does any milestone-three scoping.

## Compaction notes taken after run 23: seed-detection grading, the longest-needle pattern and the go/no-go number

The point after run 23 is where the user chose to compact.
Seed-detection grading (commit `a5c368a8a`) is active from run 5:
it splits panel detection misses from editor under-restoration
per seed,
  which TLL1122 and luxuanwen3 need.
PATTERN:
  every missed seed so far is the LONGEST needle of its
entry,
  restored only partially (5/18 and 5/14 words returned);
long omissions come back compressed.
  Calibration candidates when
data accumulates:
  widen omission envelopes,
  or grade long needles
by clause.
Follow-ups beyond that,
  none yet requested:
  canary calibration
feeding panel weights,
  per-model editor comparison in candidate
slates.
  MILESTONE-TWO GO/NO-GO NUMBER:
  seeded
repair rate,
  the fraction of seeded omissions whose repaired
candidate restores content matching the known deleted needle
(normalized similarity;
  we planted it,
  so ground truth is exact)
with ZERO out-of-envelope diffs.
  Reuses the seeded harness and the
25-minute budget discipline.
- Task list lives in the session task tool;
  mirror of current state is in "Task state" below.
