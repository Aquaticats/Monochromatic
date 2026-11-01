# Translation repair MiMo trial: log of the B123 fix, 2026-10-04

The two trial log lines from 01:37 and 02:26 UTC on 2026-10-04 that record B123,
the footnote relabel refusing a page with an unpositioned link literal:
the red case and the design settled by a probe,
then the fix and the cache-version measurement.

Part of [Translation repair: trial handover to MiMo, 2026-10-03](translation-repair-mimo-trial-2026-10-03.md),
which lists every file of this record.

## 2026-10-04, 01:37 to 02:26 UTC: B123, the unpositioned link literal

- 2026-10-04,
  02:26 UTC:
  `B123` closed with commits `23137d784` (red),
  `539cd11b8` (the design recorded mid-batch)
  and `9d7bbbec2` (the fix):
  the marker walk now reads the raw behind each unpositioned run,
  bounded between its positioned neighbours,
  so a page holding a footnote and an autolink literal relabels
  instead of being refused whole.
  The design reads the installed tokenizer's `defined.includes` gate:
  an undefined reference stays in its text,
  rides in the run,
  and the recovery places it exactly,
  while escaped openings and malformed ones stay byte-identical
  (three contract cases beside the red one).
  No cache version moves:
  279 pinned `.md` and `.mdx` files scanned,
  68 with marker text,
  0 of those parsing into unpositioned nodes
  (`~/temp/agent/mimo-trial/b123-corpus-scan.mjs`).
  Counts at the close:
  the full suite 1,548 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/b123-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/b123-scans.log`).
  The ledger's B123 entry update lands in this line's commit.
  This line lands in the trial-log commit that follows `9d7bbbec2`.
  Next:
  the `synthetic` T8 cluster
  (9 stretches over 9 lines in 2 files at `census-xB1R6c`).

- 2026-10-04,
  01:37 UTC:
  `B123` is in progress.
  The red case through `applyFootnoteRelabel` is committed (`23137d784`):
  one page per measured trigger shape
  (a quoted `www.` literal,
  one after an unbalanced `[`,
  one beside CJK punctuation,
  one after a comma),
  each refusing today with the `position` kind
  (`~/temp/agent/mimo-trial/b123-red.log`).
  A probe of the built package settled the design
  (`~/temp/agent/mimo-trial/b123-probe.mjs`):
  the autolink transform's unpositioned runs hold `[^` text
  in the empty-label,
  spaced-label and unclosed shapes,
  and their decoded values erase escapes
  (`\[^x\]` reads as `[^x]`),
  so no check off node values can tell a marker from an escaped opening.
  The fix will bound each unpositioned run
  between its positioned neighbours
  (the parent's own bounds at either end)
  and scan that raw region with `gfmMarkerSpans`,
  which is escape-aware and reads labels as micromark does:
  no fabricated positions,
  no value checks,
  and the existing graph verification still guards the rewrite.
  This line lands in the trial-log commit that adds it.
  Next:
  the fix and its contract cases,
  then the cache-version call measured on the pinned corpus.
