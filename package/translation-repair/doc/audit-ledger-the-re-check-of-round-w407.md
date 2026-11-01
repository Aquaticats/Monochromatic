# Audit ledger: the re-check of round w407

Round w409d (2026-11-01 UTC) answers the findings of a second-pass reader on round w407,
on the tree `tr-w409d` (detached at `dffa3ac22f`).
The reader is of the same vendor as the author of round w407,
so its findings are a second pass and not an independent review.
Each finding was checked against the tree before anything changed,
and an entry says where a finding was wrong.
The owner wants every mistake fixed:
"no mistake is too small or low impact or ambiguious to fix",
and recorded in the package docs so it does not recur.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

Ids B1395 to B1409 are used in this file only.
The logs and scripts are under `/var/home/user/temp/agent/tr-fan/w409d-fix/`.
The owner's words in this file are those the brief of the round gave:
"no mistake is too small or low impact or ambiguious to fix".

## B1395: the note of B1314 names the sentence it calls false

The finding:
B1358 says the pointers by position the round added were replaced by names,
the note of B1314 ("The last sentence") among them.
It was not replaced.
`audit-ledger-tests-read-only-what-their-package-declares.md` still opened the note with "The last sentence was false"
(found by `rg --line-number "The last sentence"` over the doc folder,
which printed that file and the line of B1358 that quotes it).

The fix:
the note quotes the sentence it calls false,
"a rewrite of it into a constant changes the expression and so fails",
and does not point at it by position.
B1358 gains a note of 2026-11-01 that says the replacement was not made in round w407.
The reader was right.

## B1396: the tags of the four TSDoc descriptions that `70ef89b0f5` changed

The finding:
B1354 and the note that round w407 added to B1342 gave the tags of the four descriptions that opened with an article.
They gave two `@param` at `nudged-reask.unit.test.ts:842` and `:844`,
and two `@returns` at `reask-reading.ts:21` and `run-config.ts:1231`.

The diff of commit `70ef89b0f580d8885bf3644322169d35ceda9490`
(`git show --format= --unified=1` on the three files)
shows what each description is:
- `nudged-reask.unit.test.ts:842` is a `@param` (`readDry`).
- `nudged-reask.unit.test.ts:844` is a `@returns`.
- `reask-reading.ts:21` is a `@returns`.
- `run-config.ts:1231` is a `@param` (`signal`).

The totals of two `@param` and two `@returns` were right,
and so were the four sites.
The tags of two sites were not.
The log of the first whole run (`source-scan-whole-final.txt` of `w406-fix`) lists the same four sites.
B1354 and the note of B1342 gain a note of 2026-11-01 with the mapping.
The reader was right.

The slip has two parts,
the lead's and the author's.
The brief of round w407 gave the tags and the sites as two lists
(two `@param` and two `@returns`,
then the four sites in the order of the log),
which reads as the first two sites being the `@param` pair.
The lead relayed that mapping from a reader without checking it against the diff.
The author of round w407 copied it into two entries,
and read the diff for the count and not for the tags.

## B1397: the originals that round w407 rewrote in place are restored, and the message of `9e7f5592ea`

The finding:
round w407 made a rule of the family:
a correction to a historical record is a dated note,
and the original sentence stays ([A correction to a historical record is a dated note][dated-note]).
Its own commit `9e7f5592ea` rewrote text of `audit-ledger-the-re-check-of-rounds-w403-to-w405.md` in place.
The diff of that commit against `0d28062c8f` (`git diff --unified=0`) shows the original text of B1327,
B1329,
B1330,
B1334 and B1337 rewritten,
and the message of the commit says "the originals of its entries stay".

Restored by round w409d from `0d28062c8f`,
each with a note headed with the date 2026-11-01,
the round that made the correction and the entry that holds its reasoning:
- The heading of B1327 ("every branch of the `package-reads` reader was switched off").
  The note says that the heading overclaims and what was done.
- The clause of B1327 that names where `count-cases.ts` is
  (the folder `w405-fix`,
  not the folder of the logs of round w406).
  The commit had written it into the sentence.
- B1330,
  "all 256 readings" of the drift case.
  The note says it builds 8.
- B1334,
  the conclusion "so no reading of the meters seats one lane and stops the other".
  The note says one reading was tried.
- B1334,
  the size of `run-config.ts` and the claim that no lint rule caps a file at 300 lines.
  The note gives the measured sizes and the rule that does.
- B1337,
  the millisecond value 1793534515113 of the reading that B1295 copies.
  The note says it is derived from the surviving timestamp.
  The derivation was run again with `date --utc --date=2026-11-01T12:01:55.113Z +%s%3N`,
  which printed 1793534515113.

Kept as round w407 left them,
because the original is a pointer by position and the position rule names a pointer as the defect.
Each original is quoted in a note or here:
- B1329,
  "the four after it" (now the five names of the lists).
  The note quotes the original.
- B1338,
  "the next bullet of this entry" (now the bullet that opens "The measurement the re-check asked for").
- B1342,
  "The targeted doc scans run earlier" (now the scans run before that whole run).

The commit message is not amended.
Corrective sentence for the comment the lead posts on
`9e7f5592ea2eaaf9ab9c9576112a6f60a78a3d57`:
its message says "the originals of its entries stay",
and its diff rewrote the heading of B1327 and the text of B1329,
B1330,
B1334 and B1337 in place in `audit-ledger-the-re-check-of-rounds-w403-to-w405.md`;
round w409d restored the original text of the heading of B1327 and of B1330,
B1334 and B1337 with dated notes,
and recorded the original of B1329 in a note (B1397).

[dated-note]: mistake-prevention-docs-and-references.md#a-correction-to-a-historical-record-is-a-dated-note
