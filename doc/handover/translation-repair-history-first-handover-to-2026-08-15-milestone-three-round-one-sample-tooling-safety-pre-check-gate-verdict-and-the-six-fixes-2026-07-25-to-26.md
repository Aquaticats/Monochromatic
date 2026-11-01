# Translation repair history, first handover document: milestone three round one: the sample tooling and sheet, the safety pre-check, the gate verdict and six precision fixes, 2026-07-25 to 26

The precision sample tooling and the sheet drawn on 2026-07-25,
the safety-invariant pre-check,
the user-graded verdict that failed the gate on 2026-07-26,
the diagnosis of its false positives,
and the six fixes that followed,
kept here in the order the record has them,
newest first.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The six precision fixes landed and round two begun

M3 FIXES A-F ALL LANDED (2026-07-26),
   and PASS 7 is the re-measure.
   Commits:
`ef6b75052` (A identity),
   `1790ec037` (B1 container unwrapping),
   `1aa8a0904`
(B2 monotone alignment),
   `7c4502580` (D source-not-golden),
   `a76aacae6` (E
context + community usage),
   `f6aee711a` (C self-contradiction),
   `f0821647f` (F
anchor naming).
   Every one:
   build/format/lint 0-0,
   types clean,
   full suite 0
FAIL.
   What each addresses in the graded sample:
   A=3 FPs,
   B=5 FPs (the largest
cluster),
   C=1,
   D=5,
   E=3,
   F=1 ungradable item.
FIX C and D and E are PROMPT changes,
   not gates.
   C was explicitly NOT built as
a deterministic check (advisor-confirmed):
   the case is cross-language semantic
overlap that no deterministic check catches without translating,
   and a lexical
approximation would silently reject real omissions,
   trading recall for
precision.
   Same reasoning for F:
   the unanchored-claim case is SURFACED on the
sheet,
   not gated,
   since rejecting such claims would trade recall for precision
before any measurement shows the trade is worth it.
ROUND-TWO SEED CHANGED:
   `DEFAULT_SAMPLE_SEED` is now
`milestone-three-precision-round-two`.
   Re-drawing under the round-one seed
would partially re-select the fifty items the user already graded,
   so the next
measurement would be scored partly on its own calibration set and read better
than the pipeline is.
   The constant's TSDoc now states this rule so a future
session cannot reuse a burned seed by accident.
ROUND-ONE EVIDENCE ARCHIVED,
   NOT DELETED:
   `artifacts/` (6.4M),
   `attempts.json`
and `slice-cache/` moved to
`node_modules/.monochromatic/translation-repair-runs/round-one-archive/`,
   with
`gate-verdict.md` and `grading-sheet.md` copied in beside them.
   The segmentation
fixes change slicing,
   so every cached slice is stale and a fresh pass is
required;
   the archive keeps the graded pool and its verdict auditable.
   All of
it stays OUTSIDE git (UNLICENSED corpus content).

## Declared identity given to the critics

M3 FIX A LANDED (2026-07-26,
   commit `ef6b75052`,
   task 36):
   critics now receive
the identity BOTH pages declare.
   `parse-document.ts` had been putting
`frontMatter` on `RepairDocument` since forever,
   and a grep proved NO downstream
consumer ever read it -- so critics judged names with the declaration withheld.
New `identity-context.ts`:
   `extractDeclaredIdentity` (reads `name` top-level,
`alias`/`location` under `info`,
   the pinned corpus's shape;
   non-string and blank
values REJECTED rather than coerced,
   since a coerced value would enter the
prompt as an authoritative correspondence) and `collectIdentityLines` (pairs
both sides per field,
   keeps one-sided declarations because "sourced metadata,
not invention" is exactly the judgment that failed).
   Returns a LIST,
   not
`string | undefined`:
   repo rule `no-restricted-syntax(no-nullish-union)` forbids
nullish unions,
   and its decision procedure picks the empty-collection branch
here.
   Prompt policy added:
   declarations are AUTHORITATIVE for naming,
   cover
transliteration across Chinese/Japanese/English readings,
   and NEVER license a
defect in surrounding prose;
   `desc` free prose deliberately excluded.
VERIFIED at the real boundary on all three failing entries:
   the block now
carries 委委-fairy/Acheron,
   岁月封华/Suigetsu Houka,
   and Toka_ls's 瞳華 alias.
12 new unit tests,
   full suite green (100 PASS,
   0 FAIL),
   lint 0/0,
   types clean.
Threading note:
   `exactOptionalPropertyTypes` rejects re-passing a destructured
optional,
   so each hop uses the codebase's conditional-spread idiom.

## Diagnosis of the segmentation cluster: index drift, container nesting and entity identity

M3 FIX B DIAGNOSIS (2026-07-26,
   task 37,
   read from the real corpus at the user's
instruction -- do NOT re-derive this from artifacts alone).
   The "segmentation"
cluster is THREE distinct causes,
   and the earlier "it's plumbing" framing was
wrong:
(1) INDEX DRIFT INSIDE A SECTION.
   `slice-pair.ts` `groupNodesLockstep` pairs by
shared index whenever both sides have equal node counts,
   and the comment at its
docblock states the assumption outright:
   "When both sides carry the same node
count their paragraphs correspond one to one ... never drifting."
   Susiethegamer
DISPROVES it:
   zh 32 nodes,
   en 32 nodes,
   yet at index 6 the en drops the zh
lead-in paragraph (a "her sister said to Susie:"
   line) and starts the blockquote
directly,
   so zh[7] blockquote is the true partner of en[6].
   Everything from
index 6 on is paired off by one;
   equal totals hid it because the en regains a
node later.
   Equal count does NOT imply correspondence.
(2) CONTAINER NESTING.
   Chinatsu_Suzuki is zh 25 nodes vs en 17:
   the en wraps its
entire trailing gallery in ONE `<details><summary>Original</summary>` element,
which is a single top-level `mdxJsxFlowElement` holding 11 blocks the zh carries
at top level.
   Inside it the en preserves the ORIGINAL CHINESE verbatim (in
traditional characters) beside the PhotoScrolls.
   Consequences:
   the 5 PhotoScroll
"omissions" are false (they are present,
   nested),
   and the preserved Chinese will
also trip the prompt's own `accuracy/untranslated` rule,
   which is a false
positive generator by design.
   Huasheng carries `<details>` on BOTH sides
(matched and translated),
   so ITS finding is misalignment,
   not convention.
(3) ENTITY IDENTITY,
   not segmentation at all.
   Susiethegamer item 41's real
defect claim is that the en attributes a game to "Nekomaki" where the zh says
姐姐 (sister) -- but Nekomaki IS the sister.
   That is fix A's territory,
   and it
shows the graded "segmenting" labels are the user's shorthand,
   not a diagnosis.
Node counts measured:
   Chinatsu_Suzuki 25/17,
   Huasheng 39/44,
   MeowBot233 55/64,
Dethelly 53/55,
   Susiethegamer 32/32.
CONSEQUENCE:
   masking non-prose MDX nodes (the cheap fix considered first) would
NOT fix this and would delete real content,
   since the en `<details>` blocks hold
prose.
   The real fix is a monotone sequence alignment tolerant of insertions and
deletions,
   plus unwrapping container elements so both sides expose comparable
top-level structure.

## Round one gate verdict: failed on precision

M3 GATE VERDICT:
   FAILED (2026-07-26,
   task 33,
   user-graded).
   The user graded all
50 items of the final sheet in place,
   with free-text rationale rather than bare
Y/N.
   Tally:
   28 clear Y,
   16 clear N,
   6 partial/ungradable.
   Bands are contiguous in
the sheet (items 1-17 small,
   18-34 medium,
   35-50 large;
   verified mechanically).
PRECISION vs the 0.9 bar -- strict (partials against) 28/50 = 0.56;
   partials
excluded 28/44 = 0.64;
   generous (both "Yes-ish" as Y) 30/46 = 0.65;
   ABSOLUTE
CEILING,
   every partial credited as a true positive,
   34/50 = 0.68.
   Per band,
partials excluded:
   small 9/15 = 0.60,
   medium 8/14 = 0.57 (10/16 = 0.63 counting
the "Yes-ish" pair,
   which enter denominator as well as numerator),
   large 11/15 =
0.73.
   The bar needs 45/50,
   so the gate fails by 22 points AT ITS CEILING:
   no
reading of the ambiguous grades can move the verdict,
   and it is not sampling
noise.
   Precision is roughly FLAT across bands,
   so entry size is not the driver
and the stratification bought a null result -- worth knowing,
   not a wasted
control.
   NOT a sheet-context artifact:
   only items 12,
   16,
   48 read as context-
starved and all three are already excluded as partials;
   each of the 16 clear N
grades carries a substantive rationale (frontmatter,
   segmentation,
   obligatory
English grammar) that more context would not overturn.
ROOT CAUSES of the 16 clear false positives,
   ranked:
   (1) SEGMENTATION/ALIGNMENT,
5 items (18,
   38,
   40,
   41, 46) -- the user named it directly ("this is a segmenting
error in our system/pipeline");
   the zh span and en span compared were never a
translation pair,
   so the model correctly reports a difference between mismatched
texts.
   This also inflates severity:
   the sample's two most severe false positives
(40 critical,
   46 major) are BOTH alignment failures.
   (2) FRONTMATTER NOT
CONSULTED,
   3 items (7,
   14, 19) -- names and aliases are declared in each page's
frontmatter but the pipeline feeds body text only,
   so a correct sourced English
name reads as an unsubstantiated substitution;
   cheap and unambiguous to fix,
since the data is in a file we already read.
   (3) LEGITIMATE CROSS-LANGUAGE
ASYMMETRY,
   5 items (2,
   10,
   17,
   23, 29) -- English obligatorily encodes what
Chinese leaves implicit (subject pronouns,
   quotation marks,
   plural address) and
the model scores that obligation as an addition or loss;
   the user's framing on
item 2 is the durable one:
   THE SOURCE TEXT IS NOT GOLDEN,
   so a translation that
repairs a source deficiency is not a defect.
   (4) DOMAIN/LOCAL CONTEXT,
   3 items
(15,
   24, 31) -- community slang the model did not know,
   and word choices judged
in isolation when the adjacent half-sentence licenses them;
   common shape is
judging a span with too little of its neighbourhood.
OTHER SIGNALS:
   addition-class claims have no gradable source context (item 16
ungradable,
   zh side rendered `(none)`);
   checked the code rather than guessing --
`sideQuotes` in `sample-grading.ts` drops empty quote strings,
   so `(none)` means
no non-empty source span existed,
   which for an `accuracy/addition` claim is
semantically CORRECT (an insertion anchors to an empty point).
   So this is a SHEET
gap,
   not an accept-gate bug:
   the grader needs a window of surrounding source text
around the insertion point.
   Affects 1/50.
   Self-contradicting claims survive
adjudication (item 48 alleges an omission its own quoted target contains).
   And
some true positives are NOT actionable (items 6,
   8,
   44:
   utterance-final
particles,
   poetic imagery) -- real precision wins that predict no repair gain,
   so
precision alone overstates deliverable value.
SAFETY INVARIANTS CHECKED CLEAN -- the gate failed on precision,
   NOT safety.
Every `repaired` entry genuinely differs from its input and the single
`unchanged` entry is byte-identical,
   measured over the artifact pool with 0
anomalies;
   0 entries blocked;
   the degrade-and-persist design in `repairChunk`
means no failure path throws,
   so a bad slice costs coverage,
   never corruption;
splice-back stays conservative (only clean-anchor chunks,
   standing slices ship
unchanged per-slice).
NON-TRANSLATION BLOCK,
   0/4 ON GENUINE INPUTS (the finding flagged at run 021 for
the milestone writeup,
   now surfaced):
   all 4 real-corpus blocks this session
(Aniloviraw,
   AkiraComplex,
   Arita,
   Mio) were FALSE;
   the only true positive the
feature ever produced is the invented cat/"meow" probe pair.
   Discriminator holds
(false blocks all carried confirmed good-translation content,
   the true positive
carried none),
   which is what the `sliceAnchorsTranslation` veto encodes.
   On an
all-real-translation corpus this is a finding about the feature's VALUE here,
   not
just its threshold.
CONSEQUENCES FOR THE RE-MEASURE:
   (a) it needs a NEW draw seed -- `DEFAULT_SAMPLE_
SEED` is the fixed constant `'milestone-three-precision'`,
   so re-drawing with it
over a changed pool would partially RE-SELECT the just-graded items and
contaminate the result;
   this sheet is burned as a calibration set.
   (b) fixing
segmentation invalidates the cached slices,
   so the re-measure needs a fresh
accumulation pass,
   not a re-draw over the current artifacts.
   TASK 31 (judge
crosscheck) STAYS DEFERRED,
   on a NEW rationale -- the old "wait until human
grades exist" expired the moment they arrived;
   the live reason is that 8 of the
16 clear false positives come from input the PIPELINE assembles,
   so a crosscheck
now would measure a pipeline about to change.
   Full quoted detail,
   which cannot be
committed (UNLICENSED corpus text),
   lives beside the sheet at
`node_modules/.monochromatic/translation-repair-runs/gate-verdict.md`.
FIX LIST,
   ranked by sample yield,
   NOT started (the user's turn was a report,
   and
scope is theirs to set):
   segmentation/alignment (5) > frontmatter (3) > accept-
gate rules for self-contradicting claims (1) > judge context widening for the
asymmetry and domain classes (8 combined,
   but these need prompt/knowledge work
rather than plumbing,
   so they are the expensive tail).

## Coverage bar met and the round one sheet drawn

TASK 30 COMPLETE + FINAL SHEET DRAWN (2026-07-25,
   advisor-confirmed).
   Advisor:
declare the bar MET at 9/11/9 -- the parity argument is decisive (small is
structurally pinned at 9 by the driver deprioritization and is already accepted
as "~10";
   by parity large=9 is "~10";
   every band is within 1 of 10,
   a face-value
satisfaction of an APPROXIMATE bar).
   Sufficiency was long past (the earlier
6-large call);
   9->10 large is a cosmetic digit whose pursuit risks a medium-
over-coverage spiral (next pick is a coin flip;
   a miss = medium 12 for zero
gain).
   STOP accumulation;
   the 29-entry pool is a frozen snapshot.
   FINAL draw run
`draw-sample -- --final` -> `grading-sheet.md` (NOT preliminary),
   pool 2871
accepted (337 small / 921 medium / 1613 large),
   50 drawn.
   Sanity checks all
pass:
   allocation exactly 17 small / 17 medium / 16 large = 50;
   reconcile clean
(every artifact's parsed accepted == its recorded acceptedCount,
   so the sample
is not short);
   large-band items render gradably across the settled large tail
(NIGHT81473140 / Jennife80677612 / Susiethegamer etc.);
   no preliminary banner.
Sheet lives OUTSIDE the repo (UNLICENSED corpus quotes) at
`node_modules/.monochromatic/translation-repair-runs/grading-sheet.md`;
   NEVER
committed.
   Task 30 (accumulation) DONE:
   29/92 settled = 9 small / 11 medium / 9
large,
   statuses 28 repaired / 1 unchanged / 0 blocked.
   Now the genuine surface-
to-user point:
   the headline gate needs the USER to grade the 50 issues Y/N (real
defect vs false positive) against the 0.9 precision bar.
   Task 31 (judge
crosscheck) stays DEFERRED until those grades exist;
   task 33 (gate verdict:
   per-
band + overall precision vs 0.9,
   plus the safety invariants pre-checked clean)
waits on the grades.
   Do NOT build ahead of grading -- the bottleneck is now
entirely the user's grading time.

## Stratified precision-sample tooling

M3 SAMPLE TOOLING BUILT + VALIDATED (2026-07-25,
   task 32,
   commits `bf4860250`
+ `be6912575`):
  the stratified precision-sample toolchain is now landed and
green (build/format 0-0/types/tests).
  Pure,
  unit-tested modules:
  `sample-
grading.ts` (band cuts 1843/3686 B matching accumulation;
  `classifyBand`;
`GradableIssue`/`GradableClaim`/`GradableSpan` -- a MINIMAL input shape a real
`AdjudicatedIssue` and an artifact-parsed issue both satisfy;
  `extractGrading-
Candidate` dedupes source/target quotes,
  primary claim first),
  `sample-draw.ts`
(`allocateBandQuota` even-split-under-availability round-robin;
  `drawStratified-
Sample` sha256-hex-keyed deterministic draw,
  round-robin ACROSS entries within
a band so one issue-heavy entry never dominates),
  `grading-sheet.ts` (Y/N grade
box per issue with zh source + en target quotes + claim),
  and `artifact-read.ts`
(`parseSettledArtifact`:
  a MEASUREMENT INSTRUMENT not a lenient deserializer --
structural-guards-only,
  THROWS `ArtifactParseError` on a malformed ACCEPTED
issue rather than skipping it,
  since a silent drop biases the precision
denominator;
  non-accepted issues excluded as out-of-denominator;
  category/
severity kept plain strings so no off-taxonomy value is ever a drop reason).
Advisor (Opus 4.8) shaped the parser-as-instrument stance + the string-not-
union call.
  Thin `corpus-run/draw-sample.ts` (mise task `draw-sample`;
  `--final`
writes the gate sheet,
  default writes a PRELIMINARY one) reconciles each
artifact's parsed accepted count against its recorded `acceptedCount` and aborts
loudly on mismatch.
  PRELIMINARY draw over the 24 settled:
  pool 2132 accepted
(337 small / 714 medium / 1081 large;
  entries 9/9/6),
  50 drawn 17/17/16.
Validated the sheet is HUMAN-GRADABLE from quotes alone across all bands and
multi-claim issues (e.g. "确认脑死亡" -> "confirmed to be dead";
  multi-claim
renders quotes joined by ` · `) -- no corpus open needed to judge Y/N.
  Sheet
lives OUTSIDE the repo (`node_modules/.monochromatic/.../grading-sheet-
preliminary.md`) since it quotes UNLICENSED corpus text;
  NEVER committed.
  FINAL
draw (`draw-sample -- --final`) runs ONCE after the large band fills,
  so the
user is never handed a sheet that shifts underneath them.
  Still pending:
  task 31
judge crosscheck (secondary machine number,
  quota-heavy,
  held until now);
  task
33 gate must report precision PER BAND (the payoff of stratifying) plus the
plain real/50>=0.9 headline.

## Safety-invariant pre-check over the settled artifacts

M3 SAFETY-INVARIANT PRE-CHECK (2026-07-25,
  task 33 prep,
  advisor-directed
"check what the artifacts actually record before building a checker;
  if they
don't,
  say so,
  don't fabricate").
  Finding:
  the artifacts record `status`,
`repairedText`,
  `findings`,
  and the issue fates -- enough to verify STATUS/TEXT
CONSISTENCY,
  but NOT the "zero deterministic-gate violations" or "zero
regression-majority selections" invariants directly (no envelope/patch-op or
candidate-comparison data is serialized;
  `findings` are model-noise diagnostics
like ambiguous-quote / quote-not-found / missing-verdict,
  not gate violations).
Those two invariants are guaranteed BY CONSTRUCTION (fail-closed patch +
resolution gates;
  `selectRepairCandidate` keeps UNCHANGED as the floor so a
repaired status means a candidate strictly beat the input) and covered by the
apply-patch / select-candidate / tally-resolution unit tests -- so task 33 will
CITE those,
  not re-measure from artifacts.
  What IS artifact+corpus-verifiable,
run now over the 24 settled (compared `repairedText` vs the pinned
`page.en.md`):
  23 repaired ALL genuinely changed (0 identical-to-input,
  i.e. no
hollow repairs),
  1 unchanged BYTE-IDENTICAL to input (correct),
  0 blocked (the
non-translation anchor-veto fix holds in the pool),
  0 anomalies.
  Status dist
23 repaired / 1 unchanged / 0 blocked;
  417 findings all model-noise shapes.
This is the "unchanged/repaired wherever the input is/ isn't beaten" invariant,
clean.
  Judge crosscheck (task 31) explicitly DEFERRED by advisor until human
grades exist to calibrate the judge against (building it now calibrates against
nothing);
  the 50-vs-2132 scope is then sequential (same-50 agreement first,
  and
the 2132-wide run is a user quota call,
  not autonomous) and the headline gate
does not depend on it.
