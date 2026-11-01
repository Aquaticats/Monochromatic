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
which reads as the two sites of `nudged-reask.unit.test.ts` being the `@param` pair.
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
- The clause of B1327 that names the folder of `count-cases.ts`
  (`w405-fix`,
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

## B1398: the files the run after the version commit wrote, counted by kind

The finding:
B1353 said that `corpus-pass-XingZ6024` "wrote 22 files after 2026-11-01T00:12Z:
the generation markers,
13 pairing records,
1 picture record and 1 section-pair record,
and no repair slice record and no refine record".
That is true only of `slice-cache/`.
The run wrote other files after that time too.
The reader was right.

The measurement:
`b1353-kinds.ts` in the folder of this round's logs finds the runs directories the way `runsDirsUnder` of the
`slice-cache` package does:
a directory that holds a `slice-cache` directory,
at most four levels below `/var/home/user/temp/agent`,
never inside a hidden directory or `node_modules`,
links not followed.
It reads no file content,
writes nothing,
and counts by modification time (`b1353-kinds-output-2.txt`).
The cutoff is the commit time of `945af136ba`,
`2026-10-31T20:12:48-04:00` (`git show --no-patch --format=%cI`),
which is 2026-11-01T00:12:48Z.

What it printed:
- 432 runs directories,
  and 3 directories it could not list (EACCES for each of the three `pgvector-*` folders).
- 14,064 records,
  which are the `.json` files at `slice-cache/<entry>/<file>`.
  That equals the 14,064 of the audit at the end of round w407,
  so the script reads what the audit reads.
- The records by kind:
  3,300 repair slice,
  3,310 refine,
  3,023 translate slice,
  2,124 lane contest,
  946 consolidate,
  870 pairing,
  452 picture reading,
  35 section pairing
  and 4 page titles.
  The repair slice and refine counts are the figures B1353 gave.
- The newest repair slice record is 2026-10-31T09:08:05Z and the newest refine record 2026-10-31T09:20:09Z,
  both in `corpus-pass-XingZ6023`.
- `corpus-pass-XingZ6024` wrote 91 files after the cutoff:
  1 `attempts.json`,
  1 under `fixed`,
  1 under `interim`,
  1 under `log`,
  65 under `prompt-payloads`,
  and 22 under `slice-cache`
  (7 generation markers,
  13 pairing records,
  1 picture reading and 1 section pairing).
  The 22 are the figure B1353 gave as the whole.
  The other 69 files belong to other stores than the slice cache,
  so the conclusion of B1353 stands:
  no repair slice record and no refine record was written after the version commit.
- Two clones of the repository under `dr/` (`clone` and `clone-nopolicy`) also hold files after the cutoff,
  at their `package/translation-repair`.
  The audit reads them as runs directories because the package holds a folder named `slice-cache`.
  They hold no record at `slice-cache/<entry>/<file>`.

The assumptions the counts do not measure:
- A file's modification time is its write time.
  A write that is renamed into place keeps the time of the write.
  A copy that keeps times,
  a restore or a `touch` would move it.
  The times were not compared with the lines of the run log.
- The three `pgvector-*` folders hold no corpus-pass run.
  They refuse reading (EACCES),
  so their contents were not read.
  `ls -ld` shows owner 525286 and not this user's 1000,
  mode `drwx------`,
  names of the pgvector container builds of 2026-09-03,
  and a change time of 2026-09-02,
  two months before the version commit.
  A pass of this user sits in a folder this user owns.
  That is evidence of who made them,
  and not of what they hold.
- No record sits outside the roots the audit reads.
  The 14,064 records of the audit equal the records under the agent root alone,
  so the audit's other two roots held none when it ran.
  A pass started with its runs directory set elsewhere would write elsewhere,
  and nothing here reads such a place.
- A launch is made from a committed tree.
  That is a procedure (step 1 of [the launch checks][launch-checks]) and no check:
  no command here reads it.

The first run of the script counted the source files of the two clones as repair slice records (3,320 where the
audit counts 3,300),
because it read every `.json` file under `slice-cache`.
It was changed to read the layout the audit reads,
and its output stays in `b1353-kinds-output.txt` and is not cited.

A command that lists the records after a time,
for the rule in [A version stays only after the stored records are read][records-read]:
`find <runs root> -path '*/slice-cache/*/*.json' -newermt <commit time> -type f -print`
printed the 15 records of `corpus-pass-XingZ6024` that the script counts as 13 pairing records,
1 picture reading and 1 section pairing.

## B1399: the lint cap measured again, with the output saved and the recipe written down

The finding:
B1352 gave "277" counted lines for `run-config.ts` and did not keep the output.
The reader raised four points,
and all four are right:
- The output of the count is not in the folder of the round's logs
  (`maxlines/` of `w407-fix` holds `count.json`,
  `raw.json` and `positive-control.txt`).
- The positive control is not a control of `max-lines` alone.
  Its log ends "Found 0 warnings and 401 errors".
  One is `eslint(max-lines)`,
  "File has too many lines (400)",
  and 400 are `tsdoc(require-tsdoc)`,
  one for each undocumented declaration (`rg --count` over the log).
  It shows that the cap fires in that folder under the configuration of the repository,
  and the other 400 errors are noise beside it.
- "Fixtures" is too broad.
  `package/config/oxlint/src/overrides.ts` switches `eslint/max-lines` off in five places:
  `**/fixture.*` (`fixtureOverride` at lines 39 to 45),
  `**/canary-lint/**`,
  `**/*.data.ts`,
  `**/*.config.*`
  and `**/*.{test,bench}.ts`.
  A file named `*.test-fixture.ts` matches none of them.
- The counting recipe depends on `count.json`,
  which is outside the repository.

The measurement again,
under `maxlines/` of this round's logs:
- `run-config.ts` copied from `dffa3ac22f` has 1,323 lines (`wc --lines`).
  `git log 96ab9d23c6..HEAD` over the file printed nothing,
  so it is the file B1352 counted.
- `count.json` holds one line:
  `{"rules":{"eslint/max-lines":["error",{"max":1,"skipBlankLines":true,"skipComments":true}]}}`.
  The rule at a limit of one line,
  with the two skips of the repository's setting (`package/config/oxlint/src/rule/style.ts`),
  reports the count it would have compared with 300.
- `oxlint --disable-nested-config --config count.json run-config.ts`,
  run in that folder,
  printed "File has too many lines (277)" and "Found 0 warnings and 1 error"
  on 1 file with 97 rules (`count-output.txt`).
  The cap is 300,
  so the file is 23 counted lines under it.
- The whole `run` package passes the repository's configuration:
  `gate-oxlint-run.txt` of `w407-fix` ends "Found 0 warnings and 0 errors" on 240 files with 489 rules,
  which include `run-config.ts`.
  The single-file run of B1352 with 489 rules was not saved,
  and this entry rests on the package-wide log.

Whether a `*.test-fixture.ts` file is held to the cap was read from the overrides and then tried.
The two longest of the five packages `run`,
`stage`,
`provider`,
`corpus-pass` and `repair` (652 and 626 lines)
count 299 and 297 under the same recipe (`fixtures-count-output.txt`),
just under the cap,
which fits a cap that applies and does not show it.
It was not run with the repository's configuration on an over-long fixture,
and that stays unmeasured.

The note on B1352 in the ledger of round w407 carries the correction.
The rule that this entry corrects is
[A limit of a tool is measured on the file][tool-limit],
which now holds the recipe.

## B1400: the files B1355 cites are in the folder of the round before

The finding:
B1355 cites `spec-m2.json` and `red-m2-0.txt` to `red-m2-8.txt` with no folder,
and the header of that ledger says the logs of its round are in `w407-fix`.
They are in `/var/home/user/temp/agent/tr-fan/w406-fix/`
(`ls` of the ten paths there printed all ten,
and `ls` of `spec-m2.json` in `w407-fix` printed that the file does not exist).
`mutate.ts`,
which B1327 names as "in the agent folder",
is in the same folder.
The reader was right.

The fix:
B1355 gains a note of 2026-11-01 with the folder.
The rule that a cited file is named with its folder is in
[Evidence a ledger cites][evidence-cited],
where the case of `count-cases.ts` (B1359) already stands.

## B1401: the rules that rested on habit give a search or a check, or say that none exists

The finding:
the rule "A seam is held by a case at the level that wires it" ended by saying that "habit finds" any other single-line
seam of the wiring,
which gives no way to find a seam.
Five more rules of round w407 ended "habit and review" where a concrete search or check exists:
the limit of a tool,
what a check covered,
a commit message that names an item or a count,
a correction to a historical record,
and a version that stays.
The reader was right on all six.

What each rule now holds,
and how it was tried:
- The seam rule
  ([A seam is held by a case at the level that wires it][seam-level]) gives two searches and a check.
  The first lists the optional properties of the types the wiring hands objects to,
  because only an optional property can be left out and still type-check.
  Run over `prompt-uniqueness-client.ts` it printed `store`,
  `refuseSeat` and `readDry`,
  and over `chat-contract.ts` it printed the optional fields of `ModelClient`,
  `providerDryness` among them.
  The second pairs a wiring line with its case by the ledger entry both cite:
  `rg --line-number "ledger B1330"` and the same with "ledger B1162" over `run-config.ts` and `run-config.unit.test.ts`
  printed the comment above `readDry` and above `refuseSeat` and the name of the case that holds each.
  The check is a removal of the line,
  `lint:types` and the wiring test.
  It was not run,
  because this round may change no TypeScript file,
  and the rule says so.
  A search by the name of a property does not find a seam:
  no line of `run-config.unit.test.ts` names `readDry` or `refuseSeat`,
  and a first search for `store` printed nothing,
  because the case that holds it names `promptPayloadDir`.
- The tool-limit rule holds the recipe of B1399.
- The rule on what a check covered holds two checks:
  the numbers close (56 tried is 36 failed plus 20 survived,
  and 20 is 5 plus 15,
  as B1355 states),
  and `rg --word-regexp "every|all"` lists the words in the places that repeat the claim.
  Over the ledger of round w407 alone it lists every line that uses either word,
  and the work is reading each against the list tried.
  The checks cannot tell whether the list tried is the right list,
  and the rule does not claim that.
- The rule on a commit message holds a check made after the commit,
  `git show --stat --format=%B <sha>` and `git show --name-only --format= <sha>` read against the message.
  It is tried on five messages in B1406.
- The rule on a dated note holds a word diff.
  `git diff --word-diff=porcelain --unified=0 0d28062c8f 9e7f5592ea` over the ledger of round w406 printed the removed
  words `every branch`,
  `256 readings,`,
  `the four after it` and `1793534515113),`,
  which are the originals the commit rewrote (`worddiff-9e7f.txt`).
  A rewrap prints no removed word.
- The rule on a version that stays holds the `find` of B1398 with the prefixes that group its output,
  and the statement of what the counts do not measure.

Where no concrete check exists the rule says so:
the removal check of the seam rule is a procedure that was not run,
the tool-limit recipe is written for the cap on lines of a file and for no other tool,
and the checks on a covered statement do not find a missing item of the list.

## B1402: the last whole source-scan run is read at the true final head

The finding:
B1354 records the whole `source-scan` result at `293feda767`,
and says the scans that read the docs were run again on the commit that records it.
That commit is `1dd1e1af3fb2b0bfd632a751731c239a59094444`.
It is docs-only (`git show --stat` lists one file,
the ledger of round w407),
and it is a later head than the one the whole run was read at.
Six of the seven scans that read the docs were run on it:
the Markdown linter,
the line limit,
the width,
the ledger ids,
the positions and the clock times
(`final-*.txt` in `w407-fix`).
`living-doc-links` was not:
that folder holds `pre-living-doc-links.txt` and no `final-living-doc-links.txt`.
The reader was right.

What it hid is not known for that commit.
This round could not run the scan at that head,
because it may not add a worktree or switch branches.
The scan passed in this round's runs of the seven doc scans on later heads
(`docscans-3.txt`,
`docscans-4.txt`,
`docscans-6.txt` and `docscans-9.txt`),
so no failure it would have found is still in the tree.

The fix is the rule in
[Lint and edits][lint-edits]:
the last commit of a round includes the docs commit that records the round's results.
A ledger cannot hold the result of the commit that adds it,
so the whole `source-scan` `test:unit` runs after the record commit,
its counts and log go in the hand-back,
and the ledger names the head its own figures were read at.
This round does that (B1409).
B1354 gains a note of 2026-11-01 that names the later head,
and the first whole run of that entry is named by its head,
`96ab9d23c68dd4a6dc6575b648ffb2ca17648c66`,
which is the parent of `293feda767` (`git rev-parse`).

## B1403: pointers by position and line breaks inside a clause in the files of round w407

The finding:
the ledger of round w407 held six pointers by position,
and three rule files held line breaks inside a clause.
The line numbers the reader gave
(about 61,
107,
146,
415,
503 and 553)
matched the pointers by their words,
and the lines had moved.

Each pointer is replaced in place by a name,
because the position rule names a pointer as the defect,
and the original is quoted here:
- B1345,
  "the last assertion fails" is now the assertion that the exchange labels include the model
  only the reading leaves in.
  It is the assertion `expect(labels.includes(onlyInReading ?? '')).toBe(true)` that `red-a1-seam-removed.txt` prints.
- B1346,
  "with the move to 0.5.0 added after it" is now "added as a sentence of its own".
- B1348,
  "an earlier assertion (the refiner that was asked)" is now the assertion on the refiner that was asked,
  which the case makes before it compares the keys.
- B1354,
  "made one commit earlier" is now "made at" `96ab9d23c68dd4a6dc6575b648ffb2ca17648c66`.
  That is the parent of `293feda767` (`git rev-parse 293feda767^`),
  read from the commit that fixed the failures because the log prints no head.
- B1357,
  "which is the duration the earlier text gave" is now "which are the durations that B1335 gives for the stop".
- B1359,
  "the earlier text" is now the section "Lines wider than 120 columns" of
  `mistake-prevention-docs-and-references.md` at `0d28062c8f`.
  `git grep -n "at most 120" 0d28062c8f` over the package docs printed that file at line 460.

The breaks inside a clause are joined in three files:
- `mistake-prevention-commit-checks-and-figures-told-to-others.md`,
  in the sections on what was covered and on a commit message,
  where the lines the reader named broke after "and the",
  "the number that",
  "a count that a later read of the log",
  "over-width" and "or the".
  Both sections were rewritten for B1401 as well.
- `mistake-prevention-shared-code-loops-and-keyed-tables.md`,
  in the sentence on alternates ("the pool of" before "alternates is drawn")
  and in the list of tests ("hold the" before "first ask"),
  and in a place the reader did not name,
  "kept in two to" before "six files".
- `mistake-prevention-stored-files-and-cached-decisions.md`,
  where a line broke after "of" before the name of a test file,
  and "each of the last two" is now the judges and the checkers one at a time.

The search that finds them:
`rg --multiline --line-number "^[^\n]{0,95} (the|of|to|a|an|and|or|in|that|is|are|by|for|with|on|at|as|which|from|than|not|it|its|their)\n[a-z]"`
over `mistake-prevention*.md` lists the lines of 96 columns or fewer that end on a function word and continue in lower
case.
Most of the lines it lists break before "and" or "until" or "from",
which is a semantic break.
The one it lists in `mistake-prevention-shared-code-loops-and-keyed-tables.md` was a break inside "two to six".
It lists nothing in the ledgers of rounds w406 to w408 (the same pattern over those files printed nothing).
The likely origin is the linter's own `--fix`,
which B1363 records.

## B1404: the two places that round w406 rewrapped are said to be rewrapped

The finding:
the originals of B1295 and B1297 that round w407 restored were rewrapped without saying so.

What the files show:
the word diff of `audit-ledger-one-stop-delivered-twice-is-one-stop.md` against `3c2b1de5c2`
(`git diff --word-diff=porcelain --unified=0`) prints no removed word,
so only line breaks differ (`worddiff-onestop.txt`).
Two places differ:
the opening paragraph of the file,
which describes the measurement of B1295,
and a sentence in B1298 that opens "while the entry started under an already aborted signal".
Both were made by round w406 in `799dca3f0681a25b9e3427805da2ee9459b3bdc5`
(`git diff --unified=0 3c2b1de5c2 799dca3f06` lists the two hunks).
Each original line was 120 columns or more (`rg` over `git show 3c2b1de5c2:<file>`),
so the original wrapping is not restored,
and a dated note at the head of the file says that the two are rewrapped.

The reader was partly wrong:
the sentence is in B1298 and no text of B1297 differs.

## B1405: the index titles, and the sentence for `799dca3f06`

Index titles:
the reader named B1345 and B1360.
`b1405-titles.ts` compares the title of every index line from B1325 up with the heading that defines the id.
Over the docs at `dffa3ac22f` it printed three differences,
B1345,
B1360 and B1363,
of 45 lines (`b1405-titles-before-1325.txt`).
Over this tree it prints none of 53 (`b1405-titles-after-1325.txt`).
The index lines before B1325 are summaries by area,
written to another convention,
and the floor leaves them out.
B1363 was not named by the reader.
The three index titles are now the headings.

The sentence for `799dca3f0681a25b9e3427805da2ee9459b3bdc5` in B1360 described the ledger the commit adds
and not what its message claims.
`git show --no-patch --format=%B` shows the message says that B1255,
B1282 to B1288,
B1295 to B1297,
B1304,
B1313,
B1314 and B1317 were "corrected in place",
and that "the launch KillMode" was "measured".
The word diff of its change to four older ledger files prints removed words in each (`worddiff-799d.txt`),
so original sentences were rewritten (B1356).
The probe was two transient units with `TimeoutStopSec=20` that recorded no exit code,
and it left the tools the pass starts unmeasured (B1357).
B1360 gains a note of 2026-11-01 with that restatement.

Corrective sentence for the comment the lead posts on
`799dca3f0681a25b9e3427805da2ee9459b3bdc5`:
its message says that B1255,
B1282 to B1288,
B1295 to B1297,
B1304,
B1313,
B1314 and B1317 were "corrected in place" and that "the launch KillMode" was "measured",
where its change rewrote original sentences in four older ledger files (restored under dated notes in B1356),
and the probe was two transient units with `TimeoutStopSec=20` that recorded no exit code and left the tools the pass
starts unmeasured (B1357).

## B1406: commit messages that omit what they touched

No message is amended.
Each corrective sentence is for the comment the lead posts,
and each count is read from `git show --stat` and `git show --name-only` after the finding was checked
(the check that the rule [A commit message that names an item or a count][message-count] now holds).

- `3a02c1fe2d8bc4383d0a3f1dca2f3ed1d203aace` is typed `fix(repair)`.
  Its two files are `repair/src/refine-phase.ts` and `corpus-pass/src/repair-translation.unit.test.ts`,
  and the second adds 26 lines,
  the case "ASKS THE LANE'S REFINERS WHERE THE ROSTER THE HOOK RETURNS NAMES NONE".
  Corrective sentence:
  `3a02c1fe2d8bc4383d0a3f1dca2f3ed1d203aace` is typed `fix(repair)` and its message names no test,
  where it also adds the 26-line case "ASKS THE LANE'S REFINERS WHERE THE ROSTER THE HOOK RETURNS NAMES NONE" to
  `corpus-pass/src/repair-translation.unit.test.ts`.
- `0dc5b293d69dc1e732f71ee7c1688ff9b7ec4d79` touches six files.
  Its message names no change to `corpus-pass/src/corpus-run/pass-stop-signals.ts`,
  which gains a 3-line comment on why the unreachable throw is a plain error.
  Corrective sentence:
  `0dc5b293d69dc1e732f71ee7c1688ff9b7ec4d79` also adds a 3-line comment to
  `corpus-pass/src/corpus-run/pass-stop-signals.ts` that its message does not name.
- `96ab9d23c68dd4a6dc6575b648ffb2ca17648c66` is a docs commit with three changes its message omits:
  the rewrap of 18 changed lines of `doc/runbook/translation-repair-corpus-pass.md`,
  one comment line of `corpus-pass/src/repair-translation.unit.test.ts`,
  and the removal of the entry of `audit-ledger-the-re-check-of-rounds-w403-to-w405.md` from
  `source-scan/src/markdown-line-width-allowance.test-fixture.ts`.
  The third was found by this round and not by the reader.
  Corrective sentence:
  `96ab9d23c68dd4a6dc6575b648ffb2ca17648c66` is typed `docs` and its message omits the rewrap of
  `doc/runbook/translation-repair-corpus-pass.md`,
  the comment edit in `corpus-pass/src/repair-translation.unit.test.ts`
  and the removal of one entry from `source-scan/src/markdown-line-width-allowance.test-fixture.ts`.
- `8425018d71ca7b81ecd70eba8b618e5e75f39021` is typed `docs`.
  It also changes one comment line of `provider/src/provider-router.ts`
  and one comment line of `provider/src/provider-router-first-ask-hold.unit.test.ts`.
  The reader said two comment lines of the first:
  the stat shows 2 changes because one line was removed and one added,
  and the diff shows one comment line.
  Corrective sentence:
  `8425018d71ca7b81ecd70eba8b618e5e75f39021` is typed `docs` and also rewrites one comment line of
  `provider/src/provider-router.ts` and one of `provider/src/provider-router-first-ask-hold.unit.test.ts`.
- `d2fd77f3a6ed2e4ced3326750f1eb7ad5e4c8c6a` is typed `test`.
  It also adds one export line to `provider/src/index-hold.ts`,
  a production barrel.
  Corrective sentence:
  `d2fd77f3a6ed2e4ced3326750f1eb7ad5e4c8c6a` is typed `test` and also adds the export line
  `providerChosenOrRefused` to `provider/src/index-hold.ts`.

## B1407: the runbook says what systemd sends the tools the pass starts

The finding:
the runbook said that "the tools the pass starts get no SIGTERM of their own" under `KillMode=mixed`.
The probe saw mise relay the SIGTERM it received to node,
so a tool the pass starts may get a SIGTERM from the process that relays it.
The reader was right.

The fix:
the runbook says "systemd sends them no SIGTERM",
which is what `man systemd.kill` gives for the processes that are not the main process under `KillMode=mixed`.
The same words stood in B1335,
which gains a dated note.
Two other places say the same thing as reasoning of their day and keep their text:
B1296 ("would get no SIGTERM at all") is the reasoning of round w404,
and the open item of B1342 ("Tools the pass starts get no SIGTERM under `KillMode=mixed`")
is an item that says what is unmeasured.

## B1408: where the findings were wrong or partly wrong

The reader was right on every item,
with these corrections to the details:
- B1329 appears in item 4 and not in item 3.
  It was not restored,
  because its original is a pointer by position.
  The note quotes the original (B1397).
- Item 10 named B1295 and B1297 as the rewrapped entries.
  The two rewrapped places are the opening paragraph of the file and a sentence of B1298 (B1404).
- Item 10 said the commit `8425018d71ca7b81ecd70eba8b618e5e75f39021` changed two comment lines of
  `provider-router.ts`.
  It changed one (B1406).
- Item 10 named the index titles B1345 and B1360.
  B1363 differed too (B1405).
- Item 10 put a break inside a clause at about line 301 of
  `mistake-prevention-stored-files-and-cached-decisions.md`.
  The break was a few lines before it,
  after "of" and before the name of the test file.
  The "last two" at that line was a pointer and is named (B1403).

## B1409: the gates, the slips of the round and what stays open

The gates,
in the tree `tr-w409d` with the keyless environment:
- `coverage-census:build` built every family package before the first scan,
  because a fresh tree has no `dist` and the scans import it
  (`build-census.txt`).
- The seven scans that read the docs (the Markdown linter,
  the line limit,
  the width,
  the ledger ids,
  the positions,
  the clock times and the living-doc links) passed together
  at `b3a86ddee2` before this record commit (`docscans-13.txt` and `docscans-14.txt`),
  and passed again on the content of the record commit before it was made (`docscans-16.txt`).
  The width scan passed with the allowance unchanged,
  so no file of the family gained or lost a wide line,
  and the ledger of round w407 and the new ledger hold none.
- Every Markdown file changed is at or under 1,000 lines (the line-limit scan).
- The whole `source-scan` `test:unit` runs after this commit,
  which is the record commit.
  A ledger cannot hold the result of the commit that adds it (B1402),
  so the counts and the log path are in the hand-back of the round.

The slips of the round:
- The first scan run came before any build and failed at load with `ERR_MODULE_NOT_FOUND` for the markup package's
  `dist`,
  which the rules for the Markdown linter describe.
- The first run of `b1353-kinds.ts` read every `.json` under `slice-cache` and counted the source files of the two
  clones as repair slice records (3,320).
  It was corrected to the layout the audit reads and then matched the audit's total.
- The first run of `b1405-titles.ts` had no floor on the id and printed 370 differences,
  because the older index lines are summaries by area.
  It printed far more than the question needed.
- The drafts of the entries broke the Markdown linter's `semantic-line-breaks` rule in almost every entry,
  a pointer by position (`above`) in a note,
  a clock time written with an offset in place of a zone,
  and four lines of 120 columns or more.
  Each was found by the scans before a commit.
- A count of 23 lines was written in B1401 for the lines a search printed.
  The count was 21 at the time and had changed since,
  and the sentence now states no count.
- The first drafts of two notes said "above" and "the first two".
  The position scan and a re-read found them.

What stays open:
- The removal check of the seam rule (B1401) is a procedure that was not run,
  because this round may change no TypeScript file.
  The optional properties `store`,
  `refuseSeat` and `providerDryness` have not been removed one at a time to see whether the wiring-level test fails.
- Whether `living-doc-links` passed at `1dd1e1af3fb2b0bfd632a751731c239a59094444` is not known (B1402).
- The contents of the three `pgvector-*` folders are not known (B1398).
- The allowance of the width scan is not lowered:
  `markdown-line-width-allowance.test-fixture.ts` is TypeScript and belongs to the code round.
  The files this round touched keep their counts.
- The comments on the seven commits of B1397,
  B1405 and B1406 are the lead's to post.

[dated-note]: mistake-prevention-docs-and-references.md#a-correction-to-a-historical-record-is-a-dated-note
[message-count]: mistake-prevention-commit-checks-and-figures-told-to-others.md#a-commit-message-that-names-an-item-or-a-count
[lint-edits]: mistake-prevention-shell-edits-commits-and-agents.md#lint-and-edits
[seam-level]: mistake-prevention-test-cases-that-hold-a-seam-or-a-count.md#a-seam-is-held-by-a-case-at-the-level-that-wires-it
[evidence-cited]: mistake-prevention-commit-checks-and-figures-told-to-others.md#evidence-a-ledger-cites
[tool-limit]: mistake-prevention-commit-checks-and-figures-told-to-others.md#a-limit-of-a-tool-is-measured-on-the-file
[launch-checks]: mistake-prevention.md#before-a-run-launches
[records-read]: mistake-prevention-stored-files-and-cached-decisions.md#a-version-stays-only-after-the-stored-records-are-read
