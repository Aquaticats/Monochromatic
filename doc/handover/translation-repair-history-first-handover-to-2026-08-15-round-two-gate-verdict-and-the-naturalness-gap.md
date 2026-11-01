# Translation repair history, first handover document: the round two gate verdict and the naturalness gap

The round two verdict that failed the precision gate at 0.74,
0.787 and 0.80,
and the confirmation,
from the architecture and from the accepted issues,
that no stage visits an accurate but stilted paragraph.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The round two gate verdict and the naturalness gap

- ROUND-TWO GATE VERDICT:
  FAILED at 0.74 / 0.787 / 0.80 (strict / partials
  excluded / ceiling),
  against round one's 0.56 / 0.64 / 0.68.
  37 clear Y,
  10
  clear N,
  3 ungradable of 50.
  Bar needs 45 of 50.
  Full analysis in
  `node_modules/.monochromatic/translation-repair-runs/gate-verdict-round-two.md`
  (outside git,
  quotes UNLICENSED corpus).
  Fixes A-F did real work (+0.18 strict,
  clear false positives 16 -> 10) and the
  direction is right,
  but the gate is not close.
  The band-balanced (0.788) and pool-weighted (0.794) readings agree to within a
  point,
  so the weighting question flagged before the draw did not need the
  user's decision.
  Record that it was computed,
  not skipped.
- NATURALNESS GAP CONFIRMED,
  both architecturally and empirically.
  The user
  suspected paragraph-level rewriting for English naturalness was not
  implemented;
  it is not.
  Architecturally:
  repairs are ISSUE-DRIVEN.
  `edit-prompt.ts` already tells the
  editor that "Emotional completeness and naturalness outrank word-for-word
  correspondence" and may "recast wording,
  sentence boundaries,
  and clause order
  freely",
  but the editor only ever touches a region an accepted issue already
  covers.
  A paragraph that is accurate,
  complete,
  and grammatical but stilted is
  never visited,
  because nothing generates an issue for it.
  Empirically,
  over all 2257 accepted issues:
  accuracy 77.4 percent (omission
  31.6,
  mistranslation 23.3,
  addition 21.8),
  style 13.6 (emotional-flattening
  8.9,
  awkward-phrasing 4.5,
  register 0.1),
  fluency 4.0,
  terminology 3.0,
  policy 1.4.
  Paragraph-level naturalness work is 4.6 percent of output.
  The grading confirms the harm is active,
  not merely absent:
  3 of the 10 clear
  false positives are literalism FIGHTING fluency (poetry judged literally,
  总是 forced to "always",
  conjunctions counted as additions when they are what
  makes the English read well).
