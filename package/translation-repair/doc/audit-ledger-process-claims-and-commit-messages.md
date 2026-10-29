# Audit ledger: process mistakes, claims and commit messages

Claims written without their evidence,
in commit messages,
the ledger and notes.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M9: a ledger claim written from a summary rather than measured

Status:
corrected in P3.
P3 said rounds ran to the 360 s deadline;
the two runs it cited had no round past 181 s.
Prevention:
a timing or count in a finding is read off the log it cites before the finding is written.

## M16: a commit message claiming records not yet written, and a hash typed rather than resolved

Status:
corrected by a commit comment on `5ab33539f`.
Its message said A13,
A14 and K5 were already in the ledger;
they were recorded in the next commit.
The first attempt at that comment named a full hash typed out by hand,
which GitHub refused as no commit.
Prevention:
a commit message states only what `git show --stat` of that commit shows;
every hash is resolved with `git rev-parse` in the same command that uses it.
Once more on 2026-09-27,
in the page-agreement docs:
a hash no command had produced was written into this ledger as a red guard's,
and caught on reading the edit back,
before any commit.
Once more on 2026-09-28:
`c1dd889b5` said the preparation hook now takes `BenchSeating`
before any preparation hook existed (it came in `aa635056a`);
a commit comment on it corrects the message.
The same day `d00123df3` was typed `test` while it also changed production code
(`attestPassReferences`,
the injected reference reader,
`keepBench`);
a commit comment names the right type.
Prevention for the type:
read `git diff --cached --stat` before choosing it;
any file outside tests makes the commit more than `test`.
Once more on 2026-09-28:
`67243edae` wrote the audit's task-list number in its subject,
which on GitHub is an unrelated file-enforcer issue;
commit comment 202472839 corrects it.
Prevention for references:
a commit message names a GitHub issue only after `gh issue view` shows it is the one meant,
and never a task-list number.
Within the hour `3f29feb30` wrote the prevention doc's task-list number in its subject,
which on GitHub is an unrelated kwin-key-helper issue;
commit comment 202478725 corrects it.
A rule written an hour before did not stop it,
so the prevention doc now asks for a read of the message
for `#` followed by digits before every commit.
Once more on 2026-09-28:
`d01d359d1` gave its counts (44 places and five) from a tally kept while editing,
before the commit existed to count;
`git show --unified=0` of it counts 40 and 6,
and commit comment 202514782 corrects both.
Prevention for counts:
a number in a commit message comes from a command run on the staged diff,
in the same call that commits,
or the message says no number.
Hours later,
with that rule written,
`dd519a3b8` said its guard found seven log times with seconds;
the guard's output before the fix held 14,
fixed by 7 edits,
so the message counted edits and called them times.
Commit comment 202516691 corrects it.
The rule holds as written:
the count came from the edit script's "applied 7 edits" line,
not from a command on the staged diff.
A third time on 2026-09-29:
`c3abb83e4` said the first `--fix` run split "187 headings and a list"
where the block comparison it came from held 187 damaged places,
186 of them headings;
commit comment 202522217 corrects it.
The count was a total read as a subtotal,
which a count taken from a command still allows:
the message has to name what the command counted.
A fourth time on 2026-09-29:
`d090a0f0a` said the first five of its red guard cases fail,
where six fail against the build it was committed on;
commit comment 202537574 corrects it.
The five came from no command's output,
so the rule held and was not followed.
The same day,
three more:
`11d029fde`'s message lost an apostrophe to shell quoting
("The guard headers claim"),
and commit comment 202542820 gives the wording meant;
`edac4d209` said a mutant survived without its new case,
which was inferred from reading the tests rather than run,
and commit comment 202543210 says so;
and two hashes were typed into ledger B23 rather than resolved,
caught on reading the edit back,
before any commit.
Prevention:
a message with an apostrophe goes through `git commit --file` from a written file,
never a single-quoted shell argument;
a claim in a message is either a command's output or labelled an inference;
and every hash in a doc comes from `git log` output in the same step that writes it.
Once more on 2026-09-29,
caught before the commit:
the message file for `e22373347` named the commit that added `build:coverage` by a hash typed from memory,
and `git log -S build:coverage` gave `043697797` before the commit was made.
The same day `37e1dd8e8` said its cases covered every branch of the census,
which no census had measured (M66);
a commit comment corrects it.

## M22: a comment's claim carried into the ledger without reading the measurement it described

Status:
corrected,
2026-09-27 (E6).
The audit copied "a malformed archive competes as though it parsed" from a TSDoc paragraph into the ledger as E6,
and no one read `measurePatchedCandidate`,
where integrity is relative to the archive,
so the finding asked for a behaviour change that would have changed nothing on the archive's own row.
Prevention:
a finding drawn from a comment is confirmed against the code the comment describes before it is filed,
and a field's doc says what the code measures,
relative or absolute,
with the function that measures it named.

## M31: replay refusals classified as damage from claim category and tag shape

Status:
caught the same day,
2026-09-28,
before the owner question was answered;
the L4 entry is corrected.
The L4 replay's refusals were sorted into damage and fixes by their claim category and markup shape,
and the `DottedNumber` prop rewrites on number-format claims went down as damage,
in the ledger and in the owner question.
Compared with the corpus source page,
five of the six restore the source's own props.
Prevention:
a gate replay's refusals are classified against the source they are meant to protect
(is the lost atom the source's,
and is what the edit wrote the source's?)
before any is called damage;
a category name says what a critic claimed,
not what the edit did.

## M32: a fix that supplies context a model lacks, built without reading the sheet it goes on

Status:
caught the same day,
2026-09-28,
by a wire guard's surviving mutants;
reverted in `80a18dd53`.
L5's footnote item said no sheet shows footnote definitions,
and seven commits over 21 files threaded
whole-document footnote definitions into every lane's window on that premise.
One rendered sheet refutes it:
every footnote definition of both documents has been on every sheet since `12ed82cee`.
The wire guard's fixture case checked the window,
not the sheet,
so it certified the wrong object,
and `l5-render.mjs`,
the pattern that would have shown it,
already existed in the same session.
Prevention:
a fix that gives a model context it "lacks" starts by rendering the actual sheet for the case
and searching it for that content;
only an absence seen on the rendered sheet licenses the fix.

## M49: a commit message that named a cause before it was looked up

Status:
happened 2026-09-28 (B18);
corrected by a commit comment on `1c3286271` the same hour.
The message said the blank first lines were left when imports were written or removed.
`git blame` of each file's first line,
run after the push,
showed every one came in with its file's first commit,
and no import removal left one.
Prevention:
a commit message states only what its diff and the checks run show;
a cause goes in only after the command that shows it (blame,
log,
a probe) has run,
and otherwise the message says what changed and nothing about why it was there.

## M66: a commit message claiming every branch covered before any coverage run

Status:
happened 2026-09-29 in `37e1dd8e8`,
caught by the first whole-suite coverage census,
closed in `7253426c8`,
and the commit carries a correcting comment.
The message said its seven test files "cover the census modules branch by branch".
The census at `e22373347` named ten stretches in four of those modules
(five in `coverage-tally.ts`,
three in `coverage-lines.ts`,
one each in `coverage-file.ts` and `build-entries.ts`):
five branches no case reached,
four `??` fallbacks on indexes always in range,
and a test of a prefix every entry carries.
The claim came from reading the tests,
the very reading T8 exists to replace.
Prevention:
a claim that tests cover a module's branches is written only from a census of the claimed sources
(`mise run coverage-census -- <test files>`),
each loaded and holding no stretch,
in a census of format 2 or later (M67).

## M72: a commit message file written without the check that its name is unused

Status:
happened 2026-09-29 in B30's last group,
harmless.
The commit protocol writes each message to a file whose name no earlier message used,
checked with `ls` first;
`b30-consolidation-message.txt` was written without that check.
The Write tool refuses to overwrite a file it has not read,
which is what made it safe,
not the protocol.
Prevention:
the `ls` of the scratch folder runs before any message file is written,
whatever tool writes it.

## M74: describe blocks counted as test cases

Status:
happened 2026-09-29 in B30's record (`c48f674bc`),
found the same hour by B31's suite comparison,
corrected here and by a comment on that commit.
B30's entry and its commit message said the whole suite passed 1,384 cases,
reconciled against the census's 1,397 case by case.
The number is the log's PASS lines:
`module-test`'s `describe` logs one info line per block,
naming the block's passing cases,
and each case's own PASS record goes to debug level (`describe.ts`,
`it.ts`).
B31's run showed it:
two cases left a block that kept a third,
and neither the count nor the pairing of lines moved.
Earlier entries had it right ("1,309 PASS lines").
Prevention:
a suite count is written as describe blocks or PASS lines,
never as cases,
and a claim about removed cases is read off the diff of the test files,
not off the log's line count.

## M76: an inference about a census written as its output

Status:
happened 2026-09-30 (UTC) in `5e27208c7`'s message,
found the same hour on review,
and corrected by a comment on that commit.
The message said `census-wNMuym` listed the placement file as carried only by `coverage-census.mjs`.
`census.json` records no carrying bundle for an unloaded source;
the claim was inferred from that bundle standing among the unloaded ones
and from `coverage-census.ts` being the module's only importer at `6e631988b`.
The rule that a claim in a message is a command's output or labelled an inference was already written,
and the message did not apply it.
Prevention:
a census-derived claim names the `census.json` field or the report line it comes from before it is written;
a claim no field or line holds is labelled an inference.

## M77: a red commit's message summing its own run wrongly

Status:
happened 2026-09-30 (UTC) in `7360aae58`'s message,
found within the hour while reading the red run's assertion output,
and corrected by a comment on that commit.
The message said the run found "the listed three forwarding sites,
the parseArgs and JSON offset sites,
and four unlisted ones",
which sums to nine against the eight found;
the run's `actual` array held the comparison reader unnarrowed,
one of the four,
so only two listed forwarding sites were found as listed.
The listed entry also named `LaneComparisonError` for a reader whose callee throws `ArtifactComparisonError`,
a class name written from the plan rather than read from `compareLanes`.
Prevention:
a message describing a run's findings is written from the run's printed arrays,
each group counted against them,
and a class a guard lists for a site is read from the throw sites the site calls.

## M78: a batch record counting its edits as its stretches

Status:
happened 2026-09-30 (UTC) in `32427c9c8`'s message and the ledger paragraph it added,
found on review the same hour,
corrected by a comment on that commit and in the next docs commit.
The fifth batch's record said its 12 stretches were nine reachable and three removed;
`census-oRsK9f`'s rows give ten and two.
The count was taken from the list of edits,
which held a `nonNullishOrThrow` narrowing the census never listed as a stretch,
and counted two stretches on one line of the digest refusal as one case.
M77's rule,
to count against what the run printed,
was an hour old and not applied.
Prevention:
a batch's stretch counts are read from the baseline census's rows,
file by file,
and each stretch is placed in exactly one of reachable or removed;
an edit the census did not list is named apart.

## M82: commit scopes written as `translation-repair`, not the package's name

Status:
happened throughout the audit,
caught 2026-09-30 (UTC) while writing the ninth batch's record.
The repository's commit rule takes the scope from the package name less `@monochromatic-dev/`,
and `package.json` names this package `@monochromatic-dev/module-translation-repair`,
so the scope is `module-translation-repair`.
Of the last 400 subjects on the branch,
245 used `translation-repair` and 131 `module-translation-repair`
(`git log --format=%s --max-count=400`,
counted by type and scope);
the ninth batch's first eleven commits,
`8536b28e1` to `9e7469896`,
all used the short form,
and its last seven the full one.
No doc in the package or the handover prescribes the short form.
Nothing reads the scope,
so no commit is wrong in what it says,
and none is amended.
Prevention:
a commit's scope is read off `package.json`'s `name` before the first commit of a session,
and a message file is checked for it with the `#` and task-number checks.

## M84: a value quoted from memory in a message, and again in its correction

Status:
happened 2026-09-30 (UTC) in `65deb5167`'s message,
found the same day while drafting B66's entry,
and corrected by a comment on that commit,
whose first wording was wrong as well and was edited within minutes.
The message quoted the lane contest's finding as "below the 3 needed";
`LANE_CONTEST_QUORUM` is 2 (`lane-contest-stage.ts` line 49).
The comment first said the 3 belonged to the polish gate's twin of the finding;
`CONSOLIDATION_POLISH_GATE_QUORUM` is 2 as well (`consolidation-polish-gate-stage.ts` line 31),
and it was read only after the comment was posted.
M77's rule,
that a class a message names is read from the code,
covers a constant too,
and neither text applied it.
Prevention:
a constant a message or a comment names is read from its declaration in a command before the text is written,
and a correction is held to the rule it corrects.

## M91: a veto-open call about a writer's output made from the fixtures

Status:
happened 2026-10-01 (UTC) in `65dd1e6a8`,
found 13 minutes later measuring the stored listings,
and reversed in `ee2d57372` and `0ebf573a3`.
`65dd1e6a8` refused an exponent in a listing price,
calling it a veto-open call on the ground that the fixtures copy listings that write plain decimals.
The fixtures are this package's copies,
not the providers' output:
over the 28 stored listings,
LLM Gateway's catalogue writes `0.042e-6` twice,
a real price the rule left unlisted (ledger B73).
Prevention:
a call about what a writer writes is measured over that writer's stored output before it is made,
with a control the measurement must flag,
and the commit cites the measurement,
never the fixtures.

## M94: a commit message draft naming a lint finding at a commit that did not raise it

Status:
happened 2026-10-01 (UTC) during B75,
caught before the commit.
The draft of `1f37aa07d`'s message said max-lines fired at `97aeee2ca`;
the reader passed max-lines' 300 counted lines only once `1f37aa07d` itself laid it out one element per line,
and the message was rewritten to say so.
Prevention:
a message's claim about what a tool reported at a commit names the run that printed it,
as every claim in a message is command output or a labelled inference.

## M102: a commit message saying a case reached an arm before any census showed it

Status:
happened 2026-10-01 (UTC) in T8's twenty-first batch,
caught by the next census,
corrected by a comment on the commit and by `9244d67d3`.
`c17feba5b`'s message said its tail case reached both `sharedPrefixLength` running to the end of the shorter text
and `sharedSuffixLength` using all its room.
`census-Yh15Hq` at that commit still listed the second as cold:
the case's break sits in the shared prefix,
so `assembledOffsetOf` returns before the suffix is measured.
The claim came from reading the case against the code,
not from a run.
Prevention:
a message or record says a case reaches a stretch only after a census
(or a coverage read of that file) shows the stretch ran;
before that it says which stretch the case is written for.

Recurred the same day in the commit recording the batch's full suite:
its message file,
stating the source scans' result of 23 PASS lines and no FAIL line,
was written in the same parallel call that counted them.
The counts matched before the commit ran,
but the claim was written ahead of its evidence.
The prevention covers any count a message states:
the message is written after the call that shows the count returns.

## M106: a commit message reporting a clean Markdown lint the committed ledger failed

Status:
happened 2026-10-01 (UTC) in `cd10f83f6` (B99),
found by the Markdown lint run for B100,
fixed in the commit adding this entry
and corrected by a comment on `cd10f83f6`.
The message reported that Markdown lint found nothing in the ledger,
and the ledger it committed carries one semantic-line-breaks finding in B99's own entry
(two clauses on the line naming the root suite's children).
Which edit followed the lint run that came back clean was not established.
Prevention:
a commit message reports the lint of the bytes it commits:
the Markdown lint runs after a doc's last edit,
as the commit message's own check runs after the message's last write.

## M119: a Markdown lint started in one checkout that read another

Status:
happened 2026-10-05 (UTC) in `441de9d2b`,
found by the agent writing the next docs commit,
fixed in `5f4c38d96`
and corrected by a comment on `441de9d2b`.
That docs commit was written by an agent in a side checkout,
and its message reported that the Markdown lint found nothing in either doc.
The lint command the agent was given runs a script from the audit's scratch,
which holds the main checkout's path as a constant and reads the named files under it,
whatever directory it is started from.
Started in the side checkout,
it linted the branch as it stood before the agent's edits.
On the committed text the lint reports one `semantic-line-breaks` finding in each doc,
on the owner's quoted ruling.
M106 already says a message reports the lint of the bytes it commits;
here the bytes linted were another checkout's,
and the script's output named the files by their path under the root without naming the root.
Prevention:
a check run for a side checkout takes that checkout's root as an argument,
and its verdict is repeated only beside the root it read;
a script that holds a root as a constant is not handed to an agent working elsewhere.
A copy of the lint that takes the root first was written for the next docs commit,
and prints the linter's exit beside each file.
`mistake-prevention.md`,
"Commit messages".

## M125: the count of pass lines read as if it could show a failing case

Status:
recorded 2026-10-06 (UTC) from the lead's directive for this docs batch,
which lists a gate result read from a pass line while a fail line stood.
The lead's notes hold the fact behind it and no account of the reading,
so which result was read so is not established here.
The fact:
a suite with failing cases still prints a `[PASS]` line listing the cases that passed,
so the count of pass lines does not move when a case fails.
The whole unit suite printed 1,823 pass lines on the tree of `cebfcc122` with two red test files in it,
beside 10 fail lines,
and 1,823 on `040656c51`,
beside none (B291).
B277 had already shown that the count is of suites that print and not of cases.
Prevention:
a run is green when its count of `[FAIL]` lines is zero and its exit says so;
the count of `[PASS]` lines is quoted beside that and never in place of it.
A background command that ends in an `echo` of its exit code reports success whatever ran,
so its log is read.
`mistake-prevention.md`,
"Claims without their evidence".

## M129: a notes file for agents written before the fixtures it described were read

Status:
happened 2026-10-06 (UTC),
caught before any agent was launched on it.
The lead wrote a notes file for the next runner agents
saying three keyless fixtures removed the package's own variables from a child's environment.
One search then showed two of them removed the keys alone.
The file was corrected before launch.
Prevention:
a brief states of a file only what a read of that file showed in the same sitting.
`mistake-prevention.md`,
"Claims without their evidence".

## M130: a commit-message check whose match stopped the commit without a word

Status:
happened 2026-10-06 (UTC);
nothing was lost.
The lead's check of a commit message for task-list numbers matched its own sentence about the `source-scans` task,
where the word "task" stood directly before a count of pass lines.
The commit did not run,
and the only sign was the matching line in the output,
which the lead had to notice.
Prevention:
a check that stops a commit says that it stopped it,
and the commit's own result is read before the next step.
`mistake-prevention.md`,
"Commit messages".

## M131: two accounts of a check, one an agent's of a scan and one a commit message's of code unread

Status:
happened 2026-10-06 (UTC),
the first found by the lead's own scan run before `53e25b458`,
the second standing in `da8a963cc`.
The agent of the cluster of B288 reported that the source scans failed only on inventory lines its move changed.
On the lead's tree two scans failed on two lines:
the held `caught-value-text` key named another function than the report did,
since the agent moved the print after writing the paragraph;
and a local whose name was a position word failed the position-reference scan,
which the agent had read as the moved exemption alone.
Both were mended before the commit (B288).
The message of `da8a963cc` says `coverage-control-probe` builds its client before it reads `--only`,
"which a case pins".
That is the agent's account;
the lead had not read that code when the message was written,
and the message does not label it.
Prevention:
an agent lists each scan finding by its text,
never as a kind;
and a commit message labels as the agent's account whatever the lead has not read.
`mistake-prevention.md`,
"Claims without their evidence".

## M151: a lint log with no verdict was read as clean

Status:
happened on 2026-10-11 (UTC);
the commit stood after a rerun.
The docs commit `38780d0830` was recorded as Markdown-lint clean from a log
that held only the logger's sink timeout (T9) and no line from the linter.
The rerun on the same bytes exited 0 with no finding,
so the claim had no evidence when it was made.

Prevention:
a check's log counts only with its exit status and its own verdict line;
a log holding neither is no result.
`mistake-prevention.md`,
"Claims without their evidence".

## M168: a cost was sized by the one shape that prompted the question

Status:
happened on 2026-10-13,
found by the fifth review and withdrawn the same day (B463).
The lead's note recorded the cost of the span rule
as one apostrophe left beside a replacement at a dropped-letter opening,
and sized it by the one dropped opening in the pinned English.
Every shape that puts a single mark at the edge of the words reaches the same code,
and the single-mark reader's own header counted 33 plural possessives.

Prevention:
a cost is sized by counting every shape that reaches the code path,
never by the example that prompted the question.
`mistake-prevention.md`,
"Claims without their evidence".

## M169: a guarantee was stated for the shapes tried

Status:
happened on 2026-10-13,
found by the sixth review (B478).
The mark rule compared the marks beside and inside a span and never what stands beyond them.
Headers said the result pairs words and marks the way both places did.
The reviewer's grids,
with emphasis,
brackets and quotation marks at the edges,
found 7,472 of 148,408 plans breaking it.

Prevention:
a sentence of the form "so the result pairs X the way both places did" is backed by a property probe
over a grid that includes markup and punctuation edges before it is written in a header.
`mistake-prevention.md`,
"Claims without their evidence".

## M172: a replay was cited as evidence for a reader though its output never showed what the reader read

Status:
found on 2026-10-13 by the sixth review.
`verify-published` was cited round after round as showing that old artifacts still read.
Its output never said whether the artifact held a consistency section with groups,
so it did not show that a generation 15 section with groups reads as before.
The line now ends `consistency-groups=2 read-at-generation=15` for the stored run (B487).

Prevention:
a replay cited as evidence for a reader prints the thing the reader read.
`mistake-prevention.md`,
"Claims without their evidence".

## M175: a hand-back called the work green from 64 test files

Status:
happened on 2026-10-13 in the first round of fixes to the sixth review;
found by the second round.
The hand-back's checks ran 64 test files and not the package suite.
The whole suite then showed four failing cases in `verify-published.unit.test.ts`,
whose expected replay lines had not been updated for the new column of B487.
The second round fixed them,
and its whole suite ended with 2,259 `[PASS]` lines and no `[FAIL]` line.

Prevention:
"green" in a hand-back means the whole package suite ran in that tree;
a list of files is not green.
`mistake-prevention.md`,
"Briefs and hand-backs".

## M180: a limit was accepted from the author's description with no probe

Status:
happened on 2026-10-13 in the entry B488,
found by the seventh review on 2026-10-14 (B494).
The lead recorded as an accepted limit that a proven apostrophe is carried as the chosen place wrote it,
so that a page can mix straight and curly forms,
and compared it to the apostrophe in `don't`.
The acceptance rested on the author's description of the behaviour.
No probe showed what a reader of the page would see.
The review measured 6,239 replaces out of 61,846 on its grid that put both forms on one page,
and the lead reversed the acceptance on 2026-10-14.

Prevention:
a limit is accepted only with a probe that shows what a reader of the page sees,
and the entry that records the acceptance names the probe.
`mistake-prevention.md`,
"Claims without their evidence".

## M220: a page read said the tense held without reading the quoted speech

Status:
happened on 2026-10-12 (UTC) in the read of TianqiChen66623,
found on 2026-10-15 in the read of TianqiChen66624.
The read recorded that the passage shipped in the present now read in the past,
and that the tense held.
It read the narration only.
Her last message (source lines 71 to 75),
which she wrote in her present,
shipped in the past on that page ("I felt like I might not make it to that day",
"There were still so many kids who needed help"),
while the human archive and her other quoted words on the page keep the present.
TianqiChen66624 shipped the same.
The house rule on tense lists what the person said among the life told in the past
and makes no exception for her own words quoted;
the lead decided on 2026-10-15 that quoted words keep their speaker's tense,
and the rule says so in the round that follows the read.

Prevention:
a page read that says a rule holds names the lines it read,
and reads quoted speech against the source and the human archive apart from the narration.
`mistake-prevention.md`,
"Claims without their evidence".

## M226: a read stated a stage's outcome from one log line and described a count it had not traced

Status:
happened on 2026-10-15 (UTC) in the lead's read of the consistency pass on TianqiChen66625 (commit `de8f6a079a`);
corrected by n170 the same day,
fixed in the docs by `e50786900d`,
and answered by the lead's corrective comment on `de8f6a079a` (commit comment 205137104).
The read made three claims from less than the record held:

- It said that the run-off of the refrain reached no quorum.
  Its only source was the "no quorum" line of the first window (log line 9551),
  read without the retry lines and without the `runOff` of the artifact.
  The first ask split its 3 usable ballots one to each of three places.
  The first window of the run-off heard 2 of 4 seats (the "no quorum" line),
  the retry asked the other two seats (log line 9552)
  and heard 2 of 2 (log line 9562),
  and the recovery round re-asked the two refused seats whose votes were surplus.
  The `runOff` of the artifact holds Qwen,
  Kimi and minimax for OCCURRENCE 3 and deepseek for OCCURRENCE 2,
  so OCCURRENCE 3 won with 3 of 4.
  The log prints no line for the final result of a run-off,
  so the line that the read took for the outcome was a stage on the way to it.
- It called the 1.3 million "delivered chars" of Qwen the volume of the reply.
  The count is the `partialText.length` of the raw stream including its framing
  (`src/abandon-kind.ts` (moved on 2026-10-18 to `package/translation-repair/provider-stream/src/abandon-kind.ts`) and
  `src/stream-drain.ts` (moved on 2026-10-18 to `package/translation-repair/provider-stream/src/stream-drain.ts`)),
  and the three streams cut at the straggler deadline held no content
  and 49,293,
  36,301 and 37,784 characters of reasoning,
  so the count was about 27 times the reasoning.
- It said that most voices were lost without counting them.
  n170 counted the replies the shape rule refused,
  the streams that were cut
  and the truncations that came back empty,
  each in its own list (`s241/notes-n170.md`,
  Q1,
  Q4 and Q5).

n170 was given the log,
the artifact and the sources,
and corrected all three.
Its own Write of the report was refused by the tool and it returned the report as text;
the lead wrote the notes from that text.

Prevention:
a read states the outcome of a stage from the record the artifact keeps of that stage,
never from one round's log line,
and describes a count that a log line prints only after reading the code that computes it.
A listing that is named by a cause gets the type of each entry first (M224).
`mistake-prevention-guards-searches-and-claims.md`,
"Claims without their evidence".

## M257: an agent's hand-back counted 6,751 hits where its own counter files total 6,628

Status:
happened after the split of 2026-10-18 (UTC),
in the hand-back of the agent that prototyped the shared analysis for the lint plugin in `w177`.
The summary said "6751 cross-project shared hits".
The lead summed the agent's own counter files
(`w177-notes/sum-verify.ts`,
family `vfinal-family`)
and they total 6,628.
The gap is 123,
and the notes do not say which count the summary used.

The notes record that it was caught by recounting from the raw files before the figure was recorded.

Prevention:
a number in an agent's hand-back is recounted from the raw files the agent names before it is recorded,
and the recount is named beside the figure.
`mistake-prevention-guards-searches-and-claims.md`,
"Claims without their evidence".

## M265: a claim that no file of a generation was stored failed once a live pass wrote one

Status:
happened on 2026-10-18 (UTC) in the round of the sol review (`w187`).
Its notes said that `ARTIFACT_SCHEMA_VERSION_V18` was still open:
both stored artifacts under the corpus-pass runs were generation 17,
and no file of 18 was stored,
so the stricter recounts of B675 and B677 could ride inside generation 18.
The notes of the same round say,
under the cache versions,
that a pass was writing into `corpus-pass-TianqiChen66626`
(`pass.lock` held,
and the count of slice-cache records rose from 62 to 74 during the work).
That pass was writing generation 18 files,
and the port found one there.
The notes had anticipated the case:
they said that if a file of generation 18 turned out to exist elsewhere,
the readings would need a gate on the generation,
or a generation 19.
A recount stricter than the pass that wrote the file counted would have refused it,
so the recount is gated by a new generation instead (B682).

Prevention:
the notes name none.
Written in this filing and not given by the notes:
before a generation or a version is called unused or unstored,
the run directories are searched for a pass that is writing now (a held `pass.lock`),
the claim names the date and time of the search,
and a change that makes a reader stricter is gated by a generation and does not rely on a directory being static.
`mistake-prevention-guards-searches-and-claims.md`,
"Claims without their evidence".

## M267: the body of a test commit said six guards for the nine mutations it pins

Status:
happened in `5a0fe85281` (`77823411c4` in `w188`),
whose body says "A mutation run left six guards of the artifact reader unpinned."
The commit pins nine mutations across two files:
five in the run-off recount
(the refusal beside a majority,
the run-off quorum,
the counting against the finalists only,
and the finalists count and labels checks,
each with a fixture that only that check refuses)
and four in the reader of an unlocated reason
(the old reading of the ballots,
candidates only,
any span,
and readable).
The commit was not amended.
The correction is commit comment 205562500 on `5a0fe85281`,
posted at 00:44 UTC on 2026-10-19:
the test changes are as the body describes,
and only the count is wrong.
The notes of `w188` record the same correction.

Prevention:
the notes name none.
Written in this filing and not given by the notes:
a count in a commit body is read off the proof log it summarises,
or the body leaves the count out.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Commit messages".

## M271: the report of the seat fix round quoted a log line with a space the source does not have

Status:
happened in the report of the agent of `w191`.
It quoted the line `SEAT CREDENTIAL REFUSED <model>: ...` with a space before a comma that the source lacks.
In `stage/src/seat-tally.ts` the failure text is followed directly by the comma,
and the line goes on "after N usable answers this run".
The notes say no more than that the quotation was inexact.

Prevention:
the notes name none.
Written in this filing and not given by the notes:
a log line quoted in a report is copied from the source or from the test that pins it,
and a test that pins a line pins its exact text.
`mistake-prevention-guards-searches-and-claims.md`,
"Claims without their evidence".

## M274: an agent reported its own correct commit message as wrong and asked for a correction

Status:
happened in the fix-mode fixture round (`w195`),
for the message of `a8f9296668` (landed as `e1debca690`).
The message says the autofix cases of `oxlint-stylistic.unit.test.ts` ran oxlint with the fix flag "in nine places",
and names four of the cases.
The agent took the count for wrong because the message names four,
and asked for a corrective commit comment.
The notes record that the diff was counted before any comment was posted:
nine `--fix` spawns are removed,
so the message is accurate and the agent misread it.
This filing counted the removals again in `git show a8f9296668`:
one inside `fixUntilStable`,
six single spawns,
and one inside each of two loops of up to eight passes that now call `fixUntilStable`,
which are nine spawns in all.
No comment was posted.

Prevention:
a claimed error in a commit message is checked against the diff of that commit before a corrective comment is posted
(the notes give this rule).
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Commit messages".

## M275: the body of a held fix counted the cases it changed without naming them

Status:
happened on 2026-10-18 (UTC) in the spread-bridge pair of the request memo round (`w190`).
The pair `50ccee0440` (red cases) and `5dce27bb59` (the fix) was held and not landed.
The notes give three faults of the pair:
two valid-fixture cases passed only because `runOxlintFixture` hid a plugin's throw (B688),
one ambiguous-spread case newly failed,
and the body counted failures without naming them.
The body of `5dce27bb59` says that "two cases of oxlint-no-restricted-syntax.unit.test.ts that passed on degraded
reports now fail on the error",
and gives no case name.
The redo (`w194`) landed `249c3d2d0b`,
whose body lists the failing cases of the test task by name,
once for the base,
once for the throw without the load-time start,
and once for the result after the change,
and names the three cases that passed on the base only because the degraded report matched them (B689).

Prevention:
a fixture runner fails on a diagnostic with no rule code,
which holds in lint mode since `w194` and in fix mode since `w195` (the notes give this),
and a commit body names each failing case it counts.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Commit messages",
and `mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## M276: the lead wrote that the stylistic package had the same findings as before, without the redo

Status:
happened in the lead's account of the lint of the plugin packages for the redo of the spread bridge (`w194`).
The lead first wrote that the stylistic package had "the same findings" without the redo.
It had three shared findings,
and type-aware warnings from dependencies that were not built.
The notes say it was corrected before the commit,
and do not say which summary line the first reading came from.

Prevention:
lint logs are compared by finding,
and not by the summary line.
`mistake-prevention-guards-searches-and-claims.md`,
"Claims without their evidence".

## M279: the lead called a work's English title invented after reading one store page, against the run's own lookup

Status:
happened on 2026-10-19 (UTC),
in the reading of `zheermao14`;
retracted in the same reading,
before anything was recorded or changed.

The page names the galgame 《恋爱绮谭·不存在的真相》 as *Mystery Lover 2:
Forgotten Truth*,
where the archive writes *Mystery Lover 2*.
The lead searched the web,
read the game's Steam page,
which gives no English title and says "English language not supported",
and told the owner the subtitle matched no source.
The run's own title lookup,
cached at `~/.cache/translation-repair/lookup/1c88e047….json` since 2026-09-19,
holds the VNDB entry `v36674` listing one English release under that title,
two indienova pages giving the same title,
and the Nintendo Hong Kong eShop page listing English among its languages.
Steam's release is the one without English.
The judges who cited "the web lookup" were right.

Prevention:
`mistake-prevention-guards-searches-and-claims.md`,
"Claims without their evidence":
a fact the run's evidence states is checked against the sources that evidence cites
before it is called wrong.

## M281: a commit body described a README change another commit had already made

Status:
happened on 2026-10-19 (UTC),
in the docs commit `79e96f63b6` of the bundler guard;
corrected by a commit comment,
`commitcomment-205673566`,
since commits are never amended.

The body of `79e96f63b6` opened with a `docs(config-rolldown)` paragraph
saying the package README's behaviour notes now name the refusal.
That README change was in the fix commit `0fa3120174`,
whose pathspec listed `README.md`;
the docs commit touched no file of `config-rolldown`.
The lead wrote the body from the plan of the change,
not from what was staged,
and caught it afterwards when the commit's file count
was one short of the paths the body named.

Prevention:
before a commit lands,
read `git show --stat` of it beside its body;
every package paragraph of the body names a file the stat lists,
and every package the stat lists has its paragraph.

## M283: the lead called "so her enemies could never take her" invented after reading one line of the original

Status:
happened on 2026-10-20 (UTC),
in the main session,
while it wrote the reader-protection scope of B697.
`7071a8eb5b` (00:54 UTC) removed the words,
`abf7fdae89` (01:15 UTC) restored them.
The lead told the owner the wrong thing and retracted it.
A corrective commit comment on `7071a8eb5b8f585a9bdf3ac6b88fd4a94d51ddd7` is owed,
since commits are never amended.

The comment over the rule's scope described her opening words as keeping a capsule "so her enemies could never take
her",
and the rule's own example carried the same motive to every sheet.
The lead read line 9 of the original page alone,
which says living is her right and not her duty,
so she could even do things with a potassium cyanide capsule in her mouth,
and that she was preparing to.
The words are not there,
and `7071a8eb5b` called the motive invented.
Line 10 of the original is the rest of the quote:
in dangerous places,
where she might be caught,
she would do it,
and she would never let her enemies get her (我绝对不会让我的敌人得到我).
The words are hers.

`abf7fdae89` gives both lines in the comment.
The rule's example keeps the generic wording of `7071a8eb5b`,
"what they might do or carry",
which is accurate and ties no motive to a case.
The message of `7071a8eb5b` still says the motive was invented;
the corrective comment says line 10 of the original holds it,
and names `abf7fdae89`.

Prevention:
`mistake-prevention-guards-searches-and-claims.md`,
"Claims without their evidence":
before text is called invented,
the whole quoted passage is read in the original,
not the line the quote started on.
The slip has the shape of M279,
a fact called wrong after one source was read,
and a quote is the unit that can run over a line.
