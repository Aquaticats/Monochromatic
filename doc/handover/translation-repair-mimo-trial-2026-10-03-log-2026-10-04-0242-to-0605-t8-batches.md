# Translation repair MiMo trial: log of 2026-10-04 from 02:42 to 06:05 UTC

The trial log from 02:42 to 06:05 UTC on 2026-10-04:
T8 batches on clusters of seven,
eight and nine uncovered stretches,
each batch a closing line and the baseline census that follows it.

Part of [Translation repair: trial handover to MiMo, 2026-10-03](translation-repair-mimo-trial-2026-10-03.md),
which lists every file of this record.

## 2026-10-04, 02:42 to 06:05 UTC: T8 batches

- 2026-10-04,
  06:05 UTC:
  the next batch's baseline census taken at commit `75d19f5c3`:
  `census written to ~/.cache/translation-repair/coverage/census-koUGtg/census.json`
  (`~/temp/agent/mimo-trial/census-18.log`).
  `library source: 162 files, 325 stretches over 794 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/window` leads with 7 stretches over 8 lines in 3 files,
  `corpus-run/contributor` with 7 over 7 in 2,
  `introduced` with 7 over 7 in 2.
  This line lands in the trial-log commit that follows `75d19f5c3`.
  Next:
  the `corpus-run/window` batch.

- 2026-10-04,
  05:55 UTC:
  the T8 `refine` batch closed with commits `d8aea82b3`,
  `0ddd86249`,
  `7710497e2`,
  `45f0eae68` and `2e827df9c`:
  7 stretches over four files.
  Five carry cases
  (the eligibility boundary's non-body zone through its document
  parameter,
  a rewrite entry that is no record,
  a rewrite naming no envelope and one writing the base back,
  the neighbour and context spreads read off the asked sheets of a fully
  scripted flow,
  and the checker tally with its worse-verdict control).
  Two came out behind their callers' own contracts
  (the resolver binds operations only to envelopes it found;
  the checker stage builds a tally per issue it asked),
  both now `nonNullishOrThrow`.
  The reach census reads `ran 2, still cold 0, cold since then 0, not loaded 0`
  and the two edited sources "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-refine4.log`).
  Counts at the close:
  the full suite 1,552 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-refine-suite2.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-refine-scans2.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `2e827df9c`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/window` cluster
  (the ranking's next at `census-UCyK9F`).

- 2026-10-04,
  05:10 UTC:
  the next batch's baseline census taken at commit `f0d26bb64`:
  `census written to ~/.cache/translation-repair/coverage/census-UCyK9F/census.json`
  (`~/temp/agent/mimo-trial/census-17.log`).
  `library source: 166 files, 333 stretches over 806 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `refine` leads with 7 stretches over 11 lines in 4 files,
  `corpus-run/window` with 7 over 8 in 3,
  `corpus-run/contributor` with 7 over 7 in 2.
  This line lands in the trial-log commit that follows `f0d26bb64`.
  Next:
  the `refine` batch.

- 2026-10-04,
  05:00 UTC:
  the T8 `markup` batch closed with commits `1136d5be0`,
  `2fe92afcd` and `27a8745bb`:
  7 stretches over `markup-atom-scan.ts` and `markup-atom-preservation.ts`.
  Four scanner refusals carry cases
  (the escaped close inside a matched region,
  the link destination unclosed at the newline,
  the unterminated comment,
  the unclosed attribute expression),
  two `?? ''` fallbacks and one `?.unexcused ?? []` came out dead behind
  their own callers' guarantees,
  and the preservation module's new test file
  (it sat on the own-unit-tests allowlist)
  grew to eight cases across the reach runs.
  The reach census reads `ran 0, still cold 0, cold since then 0, not loaded 0`
  and the two edited sources "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-markup3.log`).
  Counts at the close:
  the full suite 1,552 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-markup-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-markup-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `27a8745bb`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `refine` cluster
  (the ranking's next at `census-uyG0kG`).

- 2026-10-04,
  04:38 UTC:
  the next batch's baseline census taken at commit `411e705a1`:
  `census written to ~/.cache/translation-repair/coverage/census-uyG0kG/census.json`
  (`~/temp/agent/mimo-trial/census-16.log`).
  `library source: 168 files, 340 stretches over 817 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `markup` leads with 7 stretches over 11 lines in 2 files,
  `refine` with 7 over 11 in 4,
  `corpus-run/window` with 7 over 8 in 3.
  This line lands in the trial-log commit that follows `411e705a1`.
  Next:
  the `markup` batch.

- 2026-10-04,
  04:28 UTC:
  the T8 `coverage` batch closed with commits `c9b1940fe`
  and `0f65e1f55`:
  8 stretches over four files.
  Two `unreachable:` index guards came out for `nonNullishOrThrow`,
  the wire's four refusals read through one case,
  the stage's three conditional spreads each carry its case and control
  (the follow-up evidence's JSON on the asked sheet,
  the identity context where handed,
  the misattributed quote named in the verdict),
  and `targetSize`'s insertion guard came out dead behind `isPaired`'s
  own filter.
  The reach census's `cold since then` reading named the identity and
  foreign-region spreads along the way,
  cased with `0f65e1f55`.
  The reach census reads `ran 5, still cold 0, cold since then 0, not loaded 0`
  and the two edited sources "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-coverage3.log`).
  Counts at the close:
  the full suite 1,549 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-coverage-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-coverage-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `0f65e1f55`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `markup` cluster
  (the ranking's next at `census-MBzNhP`).

- 2026-10-04,
  04:12 UTC:
  the next batch's baseline census taken at commit `c651bea68`:
  `census written to ~/.cache/translation-repair/coverage/census-MBzNhP/census.json`
  (`~/temp/agent/mimo-trial/census-15.log`).
  `library source: 172 files, 348 stretches over 826 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `coverage` leads with 8 stretches over 9 lines in 4 files,
  `markup` with 7 over 11 in 2,
  `refine` with 7 over 11 in 4.
  This line lands in the trial-log commit that follows `c651bea68`.
  Next:
  the `coverage` batch.

- 2026-10-04,
  03:58 UTC:
  the T8 `corpus` batch closed with commits `33fc7a459`,
  `6046af7be` and `0c0b80bd9`:
  8 stretches over `corpus-source.ts` and `corpus-name-index.ts`.
  Five are cased through the public surface
  (the failure classifier's cause shapes,
  the stderr reader's string and unreadable arms,
  the name index's no-renderings skip and length gate,
  and its catch rethrow through the injected reader).
  One was the `gitOutput` catch rethrow,
  dead behind nano-spawn's error wrapping:
  the catch now wraps every throwable
  as the sibling `readCorpusBytes` catch already did,
  and the tradeoff is recorded under "Decisions for the owner to review"
  for the owner to veto.
  The reach census's whole-file reading named the byte reader's body and
  its failure wrap along the way,
  both cased (`6046af7be`,
  `0c0b80bd9`).
  The reach census reads `ran 3, still cold 0, cold since then 0, not loaded 0`
  and the edited source "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-corpus3.log`).
  Counts at the close:
  the full suite 1,549 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-corpus-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-corpus-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `0c0b80bd9`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `coverage` cluster
  (the ranking's next at `census-6x3Blt`).

- 2026-10-04,
  03:32 UTC:
  the next batch's baseline census taken at commit `5fe00bd6e`:
  `census written to ~/.cache/translation-repair/coverage/census-6x3Blt/census.json`
  (`~/temp/agent/mimo-trial/census-14.log`).
  `library source: 174 files, 356 stretches over 840 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus` leads with 8 stretches over 14 lines in 2 files,
  `coverage` with 8 over 9 in 4,
  `markup` with 7 over 11 in 2.
  This line lands in the trial-log commit that follows `5fe00bd6e`.
  Next:
  the `corpus` batch.

- 2026-10-04,
  03:22 UTC:
  the T8 `corpus-run/tag` batch closed with commits `a0d13daf7`
  and `71c58938b`:
  8 stretches over `corpus-run/tag-attributes.ts`,
  cased in six in a new test file beside the module
  (it sat on the own-unit-tests allowlist and now leaves it,
  `readTags` exported through `corpus-barrel`).
  The refusals read as:
  a name cut off at the text's end,
  a bare attribute with no equals staying out of the reading,
  an attribute starting where no name may,
  an unquoted value,
  an unterminated quote,
  and a bracket that opens no letter,
  every refusal case keeping the readable tag before it as its control.
  The first reach census named one `cold since then` stretch
  (the first-letter refusal),
  closed with `71c58938b`.
  The reach census reads `ran 8, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-tag2.log`).
  Counts at the close:
  the full suite 1,549 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-tag-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-tag-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `71c58938b`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus` cluster
  (the ranking's next at `census-E0TzdR`).

- 2026-10-04,
  03:12 UTC:
  the next batch's baseline census taken at commit `63c8f25b0`:
  `census written to ~/.cache/translation-repair/coverage/census-E0TzdR/census.json`
  (`~/temp/agent/mimo-trial/census-13.log`).
  `library source: 175 files, 364 stretches over 856 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `corpus-run/tag` leads with 8 stretches over 16 lines in 1 file,
  `corpus` with 8 over 14 in 2,
  `coverage` with 8 over 9 in 4.
  This line lands in the trial-log commit that follows `63c8f25b0`.
  Next:
  the `corpus-run/tag` batch.

- 2026-10-04,
  03:03 UTC:
  the T8 `inspect` batch closed with commit `f8cf67026`:
  8 stretches over `inspect-paragraph.ts`.
  Three `atomsOfNode` arms are cased
  (an image with a resolved reference,
  formatting spans passing the walk through to their numbers),
  three refusals are cased
  (the strict-grammar catch and its `unparseable` rejection beside an
  unclosed JSX tag,
  the definitions-parse rejection beside broken definitions),
  and `gateParagraphRewrite`'s base rejection is cased beside its
  candidate twin.
  The second `not-one-paragraph` check sat dead behind the first one's
  kind half:
  a probe over five one-paragraph texts and twenty-one definition shapes
  never fired it,
  so the first check proves only the one-block shape
  and the leading-block check keeps the kind live,
  pinned with a single heading.
  The reach census reads `still cold 0, cold since then 0, not loaded 0`
  and the edited source "loaded it and left 0 cold stretches"
  (`~/temp/agent/mimo-trial/reach-inspect.log`).
  Counts at the close:
  the full suite 1,548 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-inspect-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-inspect-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `f8cf67026`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `corpus-run/tag` cluster
  (the ranking's next at `census-8aKVkN`).

- 2026-10-04,
  02:52 UTC:
  the next batch's baseline census taken at commit `7cde68089`:
  `census written to ~/.cache/translation-repair/coverage/census-8aKVkN/census.json`
  (`~/temp/agent/mimo-trial/census-12.log`).
  `library source: 176 files, 372 stretches over 889 lines, 9 functions never called`.
  Largest clusters by stretches then lines:
  `inspect` leads with 8 stretches over 33 lines in 1 file,
  `corpus-run/tag` with 8 over 16 in 1,
  `corpus` with 8 over 14 in 2.
  This line lands in the trial-log commit that follows `7cde68089`.
  Next:
  the `inspect` batch.

- 2026-10-04,
  02:42 UTC:
  the T8 `synthetic` batch closed with commit `eca4d6e9f`:
  9 stretches over `synthetic-quota.ts` and `synthetic-transport.ts`,
  all cased and none unreachable.
  Seven quota field refusals read through four cases
  (the array-instead-of-object body,
  the string weekly block,
  each mistyped five-hour field,
  each mistyped weekly field),
  every refusal naming its own field.
  Two transport pass-throughs read through one overrun case
  and one wire-format case with its control half
  (the anthropic body unnamed completes past the same bound,
  its frames holding no `choices` key).
  The reach census reads `ran 9, still cold 0, cold since then 0, not loaded 0`
  (`~/temp/agent/mimo-trial/reach-synthetic.log`).
  Counts at the close:
  the full suite 1,548 [PASS] and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-synthetic-suite.log`),
  lint "Found 0 warnings and 0 errors.",
  35 source scans and no [FAIL]
  (`~/temp/agent/mimo-trial/t8-synthetic-scans.log`).
  The T8 paragraph for the batch lands in this line's commit.
  This line lands in the trial-log commit that follows `eca4d6e9f`.
  Next:
  the whole-suite census at this commit as the next batch's baseline,
  then the `inspect` cluster
  (the ranking's next at `census-xB1R6c`).
