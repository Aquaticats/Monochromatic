# Translation repair history 2026-08-05 to 2026-08-07: the naturalness lane built

The naturalness lane from its first shape to its completion.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The naturalness lane begins

- NATURALNESS LANE,
  DETERMINISTIC HALF BUILT (task 46,
  still in progress).
  `refine-eligibility.ts` (`84e8fc380`) decides which paragraphs of a REPAIRED
  slice the lane may touch.
  It is named a FILTER,
  never a verse detector,
  because nothing in the parsed model identifies poetry:
  an mdast `break`,
  a
  soft source wrap inside node text,
  and an HTML or MDX break element are three
  different things and none means verse.
  It admits only single-line prose,
  so
  single-line poetry still passes and wrapped prose is still skipped.
  It reads the repaired slice,
  never the original target:
  accuracy edits shift
  offsets and can change block structure.
  Every block gets a verdict and skips carry their reason,
  so lane yield is
  explainable.
  A degraded parse disqualifies the WHOLE slice,
  since a downgrade
  or a masked region changes how every block was read.
  `protected-atom.ts` plus `inspect-paragraph.ts` (`b87753ce9`) are the
  structural gate.
  Atoms compare as an ORDERED SEQUENCE:
  a multiset would pass
  "3 cats and 5 dogs" becoming "5 cats and 3 dogs",
  two links exchanging
  destinations,
  and two names exchanging positions,
  all of which are now tests.
  TWO DEFECTS FOUND BY RUNNING THE CODE,
  not by reading it,
  both worth keeping
  in mind because both were silent:
  A paragraph parsed in isolation does not resolve references.
  GFM only yields
  a `footnoteReference` when a matching definition is in scope,
  so `[^1]` came
  back as literal text:
  the digit was protected as a number while the marker
  around it was not,
  and a rewrite turning `[^1]` into `1` would have passed.
  Fixed by parsing twice,
  alone for structure and with the document definitions
  for references.
  The first version walked code points correctly but stopped its foreign ranges
  at U+FAFF,
  so a given name in Han Extension B produced NO atom and could be
  deleted silently.
  That is the character most likely to be a person's name.
  Both now have regression tests.

## The naturalness lane complete

- NATURALNESS LANE COMPLETE (task 46,
  commits `b3aee385a`,
  `acc5022e5`,
  `6d695fe4e`).
  One rewriter call per slice returns paragraph rewrites,
  each
  gated on the ordered atoms,
  applied through the SAME deterministic gate the
  editor uses,
  and judged as whole slices by models that wrote none of them.
  Batched per slice rather than per paragraph for correctness as much as wall
  clock:
  paragraphs rewritten in separate calls are chosen against each other
  by nobody,
  so the slice reads as stitched fragments.
  Same problem whole-chunk
  judging solves for the editor.
  BOTH decline dispositions keep `T1`,
  unlike the editor stage,
  and the
  asymmetry is the point.
  The editor works from panel-accepted issues with
  checkers proving each one gone,
  so shipping on indecision is safe because a
  later gate still tests it.
  Nothing here claimed the text was wrong,
  and on a
  slice with no accepted issues nothing downstream re-examines a refusal.
  The lane is a SECOND per-slice phase in `repairTranslation`,
  not inside
  `repairChunk`,
  and the first phase test is the reason:
  `repairChunk` returns
  early when no claim validates,
  so text with no accuracy defect never reaches
  its bottom,
  and that text is the lane's primary target.
  A failed recheck rolls back the WHOLE slice,
  with the regressed issue named.
  The recheck is skipped when the slice had no confirmed issue,
  which is the
  common case.
  Definitions come from the assembled `T1`,
  since a paragraph may reference a
  footnote defined in another slice and an out-of-scope reference does not parse
  as a reference at all.
  `spliceSlices` was extracted because the driver now
  assembles twice.
  ON for corpus runs:
  `refinerModelIds: ['hf:moonshotai/Kimi-K3']`.
  It also
  edits,
  which nothing forbids (judges exclude producers;
  checkers exclude
  editors AND refiners),
  but a model that just wrote a paragraph judges its own
  awkwardness poorly.
  The only strong-enough model that neither edits nor checks
  is GLM-4.7-Flash,
  the one that most often loses its voice to schema mismatch,
  so strength won.
  Revisit if the `refine-` findings show little change.
  NOT YET MEASURED:
  the lane has never run against the real provider.
  Every test
  is over a scripted client,
  so the prompt's "leave it alone unless the
  improvement is clear" instruction is unvalidated against real model behavior,
  and that instruction is the main guard on a slice with no accepted issues.
