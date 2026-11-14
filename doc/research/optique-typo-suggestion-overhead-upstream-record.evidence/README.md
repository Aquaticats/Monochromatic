# Members kept out of the optique evidence archive

A note beside the kept files,
written by the removal of 2026-11-13;
it is the only file here that was not a member of the archive.

- Archive:
  `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`,
  631659 bytes,
  SHA-256 `0f2a70bd4504b1c8f58cb5234feb5354e5fa490fa7933019106195517d892352`.
- Last tracked at commit `a6783a44ae7bdd43f9cd6995fcb6416eea1250d1`.
  Any member prints with `git show a6783a44ae7bdd43f9cd6995fcb6416eea1250d1:doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`,
  then `unzip -p`.
- The archive held 266 members:
  240 files and the rest directory entries.
  160 files are kept here,
  as they stood in the archive at that commit,
  at their archive-relative paths.
- Each kept file's bytes and SHA-256 are in
  `doc/audit/removed-archives-and-where-history-holds-each.md`,
  which also gives the archive's commit and print command.
- The documents that cite this folder,
  and what each one cites:
  `doc/research/optique-typo-suggestion-overhead-upstream-record.md` cites `results/`,
  the three `results-scaling/` files `fit.json`,
  `fit-scaling.txt` and
  `summarise-search.txt`,
  `coordinator-check/paired-statistics.ts` with its `*-rounds.jsonl`
  files,
  `agent-scaling-report.md`,
  and the cost model with the paths it reads;
  `doc/troubleshooting/deno-run-with-an-npm-import-map-fails-beside-a-package-json.md` cites
  `measurements-2026-11-13/cross-runtime/deno-variants.log` and
  `measurements-2026-11-13/close-review-2-gaps.jsonl`;
  `doc/troubleshooting/optique-1-4-short-option-scoping-slows-every-option-probe.md` cites the
  `pull-request-1121-on-node-and-bun-2026-11-13` folder;
  `doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.md` cites the
  `measurements-2026-11-13` and `two-remaining-copies-recheck-2026-11-13` folders.
- Every kept file is byte for byte the member it is named after,
  except seven that this repository's own guards changed:
  five markdown members were re-wrapped at semantic line breaks and their bare fences given a
  language label by `monochromatic-lint --fix-content`,
  with no word changed;
  `data/real-paths.json` was re-wrapped by `dprint fmt`,
  with no value changed;
  and `results/report-tables.txt` lost its extra final newline to the repository's
  `final-newline` policy.
  The record carries both digests of each.
  A patch stays applicable and a JSONL stays parseable,
  so nothing else was touched.
- The rest left the tree on 2026-11-13,
  when the owner ordered that no archive stays tracked;
  Git history still holds the archive whole at the commit above.
