# Page shape: semantic wrap stays per slice

Decided by the owner on 2026-09-03 ("Keep per-slice wrap"),
asked with three options after the XIEPT2
reading (`doc/planning/translation-repair-xiept2-reading-2026-09-03.md`,
"Shape") found the page
alternating between one-clause-per-line paragraphs and single-line paragraphs section by section.

## The rule

- Semantic wrap is applied per slice,
  and a line-structured slice (poems,
  scheduled-message blocks,
  anything
    whose line breaks carry meaning) is left as it stands.
- A page whose slices differ in kind therefore differs in shape across sections,
  and that is accepted.

## Options rejected

- Wrap the whole page one way at assembly,
  with fenced and line-structured blocks exempt:
  one convention per
    page,
  at the cost of touching the assembly step every page passes through,
  a new guard,
  and a re-read.
- No semantic wrap,
  single-line paragraphs:
  the archive's own convention is unmeasured,
  and every diff
    against the archive would become a whole-paragraph diff.

## What this changes

Nothing in code.
The reading's "Shape" finding is closed by decision,
and a later reading does not report it
as a defect.
