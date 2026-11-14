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
  239 files are kept here,
  as they stood in the archive at that commit,
  at their archive-relative paths.
- One sentence of the research record names four things,
  and the owner decided on 2026-11-14
  that the files they cover are kept as plain text here
  (`doc/handover/cli-git-rust-implementation.md`,
  "Owner decisions in force"):
  the harness is `scripts/` with the top-level `Containerfile`,
  the check scripts are `scripts/` and `coordinator-check/`,
  the baked inputs are `data/`,
  `src/` and `ctx/`,
  and every raw record of both rounds is `results/` and `results-scaling/`.
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
  except nine that this repository's own guards changed:
  five markdown members were re-wrapped at semantic line breaks and their bare fences given a
  language label by `monochromatic-lint --fix-content`,
  with no word changed;
  three JSON members (`data/real-paths.json`,
  `data/short-paths.json`,
  `data/issue988-paths.json`) were re-wrapped by `dprint fmt`,
  with no value changed;
  and `results/report-tables.txt` lost its extra final newline to the repository's
  `final-newline` policy.
  The record carries both digests of each.
  A patch stays applicable and a JSONL stays parseable,
  so nothing else was touched.
- One member of the archive is not kept:
  `agent-final-report.md`,
  17106 bytes.
  It is none of the four things that sentence names,
  it is not the harness,
  a check script,
  a
  baked input or a raw record of either round,
  and it is the agent's own account of the work,
  which the documents here carry.
  It stays in history alone,
  and
  `doc/audit/removed-archives-and-where-history-holds-each.md` keeps the command that prints it.
- The rest left the tree on 2026-11-13,
  when the owner ordered that no archive stays tracked;
  Git history still holds the archive whole at the commit above.
