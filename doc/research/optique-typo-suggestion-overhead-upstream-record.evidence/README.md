# Members kept out of the optique evidence archive

A note beside the kept files,
written by the removal of 2026-11-13;
it is the only file
here that was not a member of the archive.

- Archive:
  `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`,
  631659 bytes,
  SHA-256 `0f2a70bd4504b1c8f58cb5234feb5354e5fa490fa7933019106195517d892352`.
- Last tracked at commit `a6783a44ae7bdd43f9cd6995fcb6416eea1250d1`.
  Any member prints with
  `git show a6783a44ae7bdd43f9cd6995fcb6416eea1250d1:doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`,
  then `unzip -p`.
- The archive held 266 members:
  240 files and the rest directory entries.
  158 files are kept here,
  as they stood in the archive at that commit,
  at their archive-relative paths.
- Every kept file is byte for byte the member it is named after,
  except six that this repository's own guards changed.
  Five markdown members were re-wrapped at semantic line breaks and their bare fences given a
  language label by `monochromatic-lint --fix-content`;
  no word changed.
  `results/report-tables.txt` lost its extra final newline to the repository's
  `final-newline` policy.
  `doc/audit/removed-archives-and-where-history-holds-each.md` records both digests of each.
  A patch stays applicable and a JSONL stays parseable,
  so nothing else was touched.
- Which document cites which file,
  with each file's bytes and SHA-256,
  is in `doc/audit/removed-archives-and-where-history-holds-each.md`.
- The rest left the tree on 2026-11-13,
  when the owner ordered that no archive stays tracked;
  Git history still holds the archive whole at the commit above.
