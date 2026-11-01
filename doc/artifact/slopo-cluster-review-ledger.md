# Review ledger for the 2026-10-07 slopo duplicate-code report

Companion to `slopo-cluster-review-ledger-2026-10-07.jsonl` in this directory.

## What it is

One JSON row per cluster in the report that slopo `0.8.0` generated on 2026-10-07 22:36:47 from the
repository index,
1008 rows covering 4377 code units.
It is the per-cluster rationale behind the 50 issues numbered #611 through #660,
and behind the four dismissal issues #656,
#657,
#658 and #659 in particular.
Before this copy it existed only in `~/temp/agent/slopo-issues/review/ledger.jsonl`,
outside the repository,
so the reasoning behind 147 proposed suppressions had no durable home.

## Fields

- `cluster`:
   the cluster ordinal in that one report.
   Ordinals come from `enumerate(clusters, 1)` at report time
   (`slopo/result/report/markdown/analyze.py:22-23`),
   so C187 means something different after any regeneration.
   The `hash` field is the durable identifier.
- `hash`:
   the cluster key,
   sha256 over the sorted `(file_path, body_hash)` pairs of its units truncated to 12 hex
   characters (`slopo/result/identity.py:24-30`).
- `theme` and `bucket`:
   the grouping used to review related clusters together.
- `units`,
   `score`:
   unit count and the report's rounded similarity range.
- `verdict`:
   one of `EXTRACT-SHARED`,
   `DEDUPE-ARTIFACT`,
   `GENERATED-MIRROR`,
   `TEST-OR-GENERATED`,
   `UNCERTAIN`,
   `STRUCTURAL-IDIOM`,
   `INTENTIONAL-VARIANT`,
   `BOILERPLATE-TRIVIAL`,
   `REMOVE-MIRROR`,
   `COINCIDENTAL-SHAPE`,
   `CROSS-LANGUAGE-DRIFT`,
   `GENERATOR-SIDE`.
   The definitions are in `doc/handover/slopo-cluster-issue-triage.md`.
- `concept`:
   the pattern name,
   which is how clusters were assigned to issues.
   86 concepts cover the 147 clusters the four dismissal issues proposed.
- `mode`:
   how the verdict was reached.
   `full-read` means the reported bodies were read in full and nothing else was inspected;
   `file-inspected` means the surrounding files were opened;
   `pattern-verified` means a mechanical invariant was checked per cluster.
- `evidence`:
   the reasoning,
   naming both sides by path,
   symbol and line range.
- `issue`:
   the issue that owns the cluster.

## Distribution, counted from this file

- Verdicts:
   405 `EXTRACT-SHARED`,
   159 `DEDUPE-ARTIFACT`,
   99 `GENERATED-MIRROR`,
   92 `TEST-OR-GENERATED`,
   66 `UNCERTAIN`,
   54 `STRUCTURAL-IDIOM`,
   48 `INTENTIONAL-VARIANT`,
   35 `BOILERPLATE-TRIVIAL`,
   28 `REMOVE-MIRROR`,
   20 `COINCIDENTAL-SHAPE`,
   and 1 each of `CROSS-LANGUAGE-DRIFT` and `GENERATOR-SIDE`.
- Modes:
   622 `full-read`,
   246 `pattern-verified`,
   87 `full-read+file-inspected`,
   42 `pattern-verified+file-inspected`,
   11 `pattern-verified+hand-read`.
- Issues:
   50 distinct.

## Limits a reader must apply

- 622 of 1008 verdicts are `full-read`,
   meaning the reviewer read what the report showed and did not open callers,
   check for an existing owner,
   or count the family the pair belongs to.
   Every dismissal that rested on "extraction costs more than it saves" came from that mode,
   and `doc/planning/slopo-dismissed-cluster-actionability.md` records what happened as a result:
   twelve clusters of one error-naming idiom were dismissed as unrelated instances of trivial
   boilerplate while the idiom sat at 210 sites across 146 files.
- Concept-level assignment put clusters into issues wholesale,
   which is why nine clusters landed under a heading their own verdict contradicts.
   That is corrected in the four issue bodies but not in this file,
   which is a snapshot of the review as it ran.
- The hashes are valid against the 2026-10-07 index.
   A regeneration reassigns every ordinal and can change any hash whose bodies or paths moved.

## What to do with it

Read it as the evidence layer under the four dismissal issues and the 46 refactor issues.
To check one cluster,
find its row by `hash`,
read `evidence`,
then open the source,
because `mode` says whether the reviewer did.
To re-run the family measurements that this file's verdicts lacked,
use the recipes in `doc/troubleshooting/slopo-threshold-tuning.md` under
"Measuring a family instead of a pair".
