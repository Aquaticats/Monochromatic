# Audit ledger: process mistakes, agents and briefs in the review, docs, archive and empty link rounds

What went wrong in the rounds that reviewed the deferred findings and the quoted tense work (n162),
wrote the docs of that work (n163),
read the history archive and fixed what the readings found (n164,
n165 and n167),
fixed the findings of that review in the sheets and the empty link reading (n166 and n168),
finished the reading of empty links as their plain-text twins (n169),
and wrote the ledger and prevention docs for the mistakes of those rounds (n172):
the rule breaches of each round,
as its author or reviewer disclosed them,
and three mistakes of the lead's briefs:
the brief for the archive fixes (M225),
and two in the brief for the docs round (M236 and M237).
The lead's two other mistakes of the same days are filed with their subjects:
M224,
the worktree added without `--no-worktree-copy`,
in
[Audit ledger: process mistakes, shell commands and tasks](audit-ledger-process-shell-commands-and-tasks.md),
and M226,
the read that took a stage's outcome from one log line,
in
[Audit ledger: process mistakes, claims and commit messages](audit-ledger-process-claims-and-commit-messages.md).
n170,
the consistency pass read of TianqiChen66625,
has no entry:
no account of its rule breaches is among the sources,
and the refusal of its Write of its report by the tool is no breach,
since it returned the report as text.
The entries before these are filed in
[Audit ledger: process mistakes, agents and briefs in the reference, edge and prose mask rounds](audit-ledger-process-agents-and-briefs-reference-edge-and-prose-mask-rounds.md),
which stopped at M223 because it would otherwise pass the line limit.
Later entries are filed in a file named for its subject,
which [Audit ledger: where each numbered entry is filed](audit-ledger-entries-by-id.md) lists by id.

The findings of the review of n162 are in its report (`s233/review.md`).
Only the residue that it traced is filed in the ledger,
as B619 in
[Audit ledger: the consistency pass, what the word test reads as shown](audit-ledger-consistency-pass-what-the-word-test-reads-as-shown.md),
which says that n166,
n168,
n169 and n173 fixed it and that it landed on 2026-10-18.
The archive that n164,
n165 and n167 read and fixed is the history archive,
`doc/handover/translation-repair-history*.md` at the repository root:
the fixes of its second reading landed in `730c503608`,
and those of its third reading in `d6f7f9ae12`.
The entries that n163 wrote for the work of n159 to n161 are filed in
[Audit ledger: process mistakes, agents and briefs in the reference, edge and prose mask rounds](audit-ledger-process-agents-and-briefs-reference-edge-and-prose-mask-rounds.md)
(M221 to M223)
and in the consistency pass files that list B616 to B628.

Written on 2026-10-15 (UTC) from the accounts of the rounds,
each of which lists its own breaches
(the reviews `s233/review.md` and `s236/review.md`,
the hand-backs `s234/handback-n163.md`,
`s235/handback-n164.md`,
`s237/handback-n166.md`,
`s238/handback-n167.md` and `s239/handback-n168.md`,
and the log `s234/breaches.md`),
the briefs of the rounds (`prompts/prompt-n162-review-of-n159-and-n161.md` to `prompts/prompt-n168-z9-follow-up.md`),
and the lead's notes of the day (`s228/next-docs-batch-notes.md` and `s241/notes-n170.md`).
M234 to M237 were written on 2026-10-18 from the hand-backs of n169 (`s240/handback-n169.md`)
and n172 (`s243/handback-n172.md`),
the briefs of n172 and n173 (`prompts/prompt-n172-docs-mistakes-n162-to-n170.md` and
`prompts/prompt-n173-labelled-links-read-as-shown.md`),
and the lead's notes (`s228/next-docs-batch-notes.md`).
n171 and n173 returned no hand-back file,
so their rule breaches are not filed,
and n170 stays without an entry.
The lead's slip in merging n173 onto n171 is M238,
filed in
[Audit ledger: process mistakes, the lead's slips by batch of work](audit-ledger-process-the-leads-slips-by-batch.md).
The rounds are named as the neighbouring entries name them,
and a round name is not a task number.
The sizes of reads and outputs are the authors' own figures.
The labels Z1 to Z11 are the review's own and are no ledger ids.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M225: a brief asked for a check to be reported where every mistake found is fixed

Status:
happened on 2026-10-15 (UTC) in the brief for n167 (`prompts/prompt-n167-history-third-reading-fixes.md`);
found when n167 read the verb after it had applied fixes.
The brief asked n167 to check every other top index entry of the history archive against its group's files
and to "report each entry's result".
The owner's standard is that every mistake found is fixed,
so "report" named the wrong act.

- n167 applied fixes to six more entries
  (the first handover,
  2026-08-05 to 2026-08-07,
  2026-08-12 to 2026-08-15,
  2026-08-21 to 2026-08-23,
  2026-08-23 to 2026-08-24
  and 2026-08-26)
  and rewrote one position pointer.
  It then read the verb,
  reverted all of it in the top index to the text of `c3c0c8dda7`,
  and recorded the clauses as proposals,
  one proposal per entry,
  marked as proposed.
- The lead applied the six proposals
  after checking that each subject they name is the title of a file (`s238/survey-before.txt`),
  and fixed the one position pointer.
- Those clauses had no reading by an agent that wrote none of them.
  The scripted checks of the index,
  sections,
  links,
  body,
  headings,
  guards
  and lint passed after them.

Prevention:
a brief's verb matches the ruling the lead will make:
a check whose findings will be fixed says "fix",
not "report".
Clauses that the lead writes itself get the reading that an agent's change gets.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs".

## M227: breaches of the Read, output, batch and quoting rules by the review of the quoted tense rounds (n162)

Status:
happened on 2026-10-15 in the review that found Z1 to Z11 and traced the residue filed as B619;
every one disclosed by its author in the review (`s233/review.md`,
"Rules I broke").

- Reads with a limit over 50:
  `pass-consistency-decide.ts` (100),
  `pass-consistency-unasked.ts` (100),
  `consistency-shown-kinds.ts` (100 twice),
  `pass-consistency.ts` (120),
  `fragment-landing-error.ts` (89),
  the test of the group line (96),
  `pass-consistency-carry-choose.ts` (80),
  `pass-consistency-settle.ts` (80),
  `consistency-shown-edge.ts` (70 and 100),
  `consistency-closing-marks.ts` (70 and 60),
  `pass-consistency-group-line.ts` (66),
  `pass-consistency-apply-record.ts` (60)
  and `diff-cache.txt` (70).
- Reads with no limit of several small files:
  the `.lines.txt` extracts,
  `y8-mutate.out`
  and `t23.txt`.
- A search for `groupLine` in the minified `dist/final/node/index.mjs` that sent 116 KB to the tool output;
  the harness kept the output outside the scratch folder,
  and the author did not open it.
- Two commands that wrote a file and one that read it in one batch
  (`q3-edges` run with a count of its output);
  the author reran `q3` after the notice.
- One `rg` with backticks in a shell string failed to parse;
  the author reran it with an escape.
- Four Edits to the review in one batch,
  for heading and count corrections.
- Sources read through `git -C <repository> show 277145a862:<path>` into the scratch folder
  while the trees were missing,
  and copies (`y8`,
  `y8b`) with links to the `node_modules` of another worktree made in the scratch folder;
  the author counted these reads of git and of a link as no write.
  The copy `y8` was mutated before the notice and `y8b` after it.

A cause named from reading alone:
the review traced Z9 to the `]` and `(` branch of `closingJointsOf` in `src/consistency-closing-marks.ts` (moved on
2026-10-18 to `package/translation-repair/consistency/src/consistency-closing-marks.ts`)
from reading the loop and from `cutsShownWord` being false on the rows,
and its own list of what it did not check says it did not instrument `closingJointsOf`.
The lead's brief for n166 carried the pointer and told the author to confirm the cause by instrumenting a copy first,
and n166 showed on an instrumented copy that refusing an empty label in that branch changes nothing:
the cause is the last return of `apostropheJoinsAt` (B619).

Prevention:
the same as M223 for the limits on Reads and outputs and for the batches of Edits.
A pattern that holds a backtick or a dollar sign is written in single quotes.
A brief that passes on a cause from a report says whether the report instrumented it.
The brief of n166 told the author to confirm the cause on an instrumented copy first,
and that instruction is what found the pointer wrong.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Briefs and hand-backs".

## M228: breaches of the Read, output, batch, quoting and control-file rules by the docs round (n163)

Status:
happened on 2026-10-15 in the writing of B616 to B628 and M221 to M223;
every one disclosed by its author
(`s234/breaches.md` and the hand-back `s234/handback-n163.md`,
section 5).

- Terminal output over 50 lines not sent to a file:
  an `ls` of the doc directory (about 123 lines)
  and an `rg` of the index by id for candidate file names (about 105).
- Reads with a limit over 50:
  the prose mask ledger file at offset 509 (74) and at offset 878 (64),
  the span edges ledger file at offset 555 (64)
  and the tests prevention file at offset 716 (58).
- An Edit and a Read of one file in one batch,
  four times,
  in four files:
  the span edges ledger file,
  the elisions ledger file,
  the prose mask ledger file
  and `audit-ledger.md`.
- Two Edits to one file in one batch,
  once:
  the labels and counts ledger file (the group line format and the `splice-cannot-reach-place` line).
- Double-quoted `rg` patterns holding backticks,
  which the shell interpreted,
  three times:
  two failed to parse,
  and one searched the wrong thing before it failed.
  One `rg` with an invented flag and a `2>/dev/null` redirect ended with status 2 and no output.
  No file was touched by these.
- A control file `zz-lint-control.md` written in the docs folder to show that the linter can fail,
  then removed with `rm`,
  which the brief does not name.
  A second control file in the scratch folder was refused by the linter for lack of a config.
- Two scripts written in the scratch folder and run with `node`:
  `render.mjs`,
  the copy that the brief asked for,
  and `check-render.mjs`,
  the author's own read-only checker of links,
  anchors,
  source paths and stray markers.
  Neither wrote under the docs.
- The entries B620 to B622 were cut out of the prose mask ledger file,
  which had reached 1,032 lines,
  by Edits whose `old_string` was a whole entry,
  and written to a new file with one Write,
  with no script.
- The source scans task ran three times;
  it builds in the worktree and writes outside the docs.
  The first run found two position words that the author had written.
- Hand-back structure:
  one Write and one Edit per appended section,
  and two small Edits of text already written in the file
  (a count of inline wrappers from 17 to 14,
  and the sentence on long lines).
- M221 carries one item that the review of the round found and the author did not disclose
  (the description of the harness copies),
  beyond the brief's "from its own account".

Prevention:
the same as M210,
M214 and M216,
and a brief that asks for a lint control names where the control goes and who removes it,
since a control in the scratch folder was refused for lack of the linter's config
and one in the docs folder needed a shell delete that the brief did not name.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Lint and edits".

## M229: a brief premise that did not hold, and a required read left unfinished, in the archive fix round (n164)

Status:
happened on 2026-10-15 in the round that was to fix the second reading of the history archive;
the round made no edit,
and the hand-back (`s235/handback-n164.md`) states both facts.
The brief said that the archive had not changed since `33d0c875ae`
and gave the check for it:
`git -C <worktree> log --oneline 33d0c875ae..HEAD -- doc/handover/translation-repair-history*`.
The check printed one commit,
`730c503608`,
"history archive,
the second reading's findings fixed" (author date 2026-10-13),
an ancestor of the branch head `5f6c8593d7`.
That commit already held the fixes of the second reading:
155 files,
none past 731 lines,
and 49 files changed.
The line numbers of the reading no longer held,
since it read `57678add44`.
n164 stopped before any edit,
reproduced only the index check (every one of the 12 group indexes with 0 problems),
and asked whether the brief was stale or a third pass was wanted.
n165 then read the landed tree a third time.

- The lead's mistake:
  the premise of the brief,
  that the archive was unchanged since `33d0c875ae`,
  was false on the tree that the brief named.
  The check that the brief gave the agent found it at once.
- The agent's breach:
  the brief said to read the fixer's account (`s158/handback-n107.md`) first and whole,
  and n164 read it to line 150 of 362.
  Its list of rules broken says "None that I know of" and gives the partial read as a fact,
  judging the rest moot once the precondition failed.

Prevention:
before a brief states that a tree has not changed since a commit,
the lead runs the check that the brief gives the agent.
A round that stops at a precondition names each read that the brief required and it did not finish.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs".

## M230: breaches of the Read, output, quoting, copy and hand-back rules by the third reading of the archive (n165)

Status:
happened on 2026-10-15 in the review of the fixes of `730c503608`
that found seven low findings,
all ruled fix and given to n167;
every one disclosed by its author in the review (`s236/review.md`,
"Rules I broke")
and in the lead's notes.

- A Read with a limit of 51,
  on the 2026-08-23 to 2026-08-24 writer calibration coverage report file.
- Output that may pass 50 lines sent to the terminal,
  three times:
  an `rg` with a backticked pound sign and number in a double-quoted pattern,
  where the shell substituted an empty string,
  the pattern matched every line,
  and 251 lines of the block grouper and pairing measurement file printed
  (which also read a file outside the Read tool and past 50 lines);
  an `rg --files-with-matches` for names of history files (about 150 lines);
  and an `rg` over the zero-overlap survey (about 85).
- The copies of `conserve.mjs` and `addedbody.mjs` were edited to take the old ref as an argument,
  where the brief said to copy and point them;
  the logic did not change.
- An `rg --replace "$0"` in double quotes expanded `$0` to `/bin/bash`
  and mangled the file names of `survey-h1.txt`;
  the author read the mangled file and worked from it.
- The brief said one Write and one Edit per appended section;
  after the appends the author made about fifteen more Edits that corrected or replaced text already in the review,
  and added sections out of order by replacing a heading line.
- The reads of the heads and tails of the touched files,
  which the brief's item 2 asks for,
  were done late,
  after the first version of the review,
  and only after the advisor pointed out that the first version claimed them.
  The first version overstated what had been read.
- The advisor was called after the review was written and not before work began
  (a harness rule,
  not the brief's).

The review did not run the Markdown lint or the guards that the commit cites,
since the brief allowed no `mise`.
That is no breach.

Prevention:
the same as M210,
M214 and M216,
and the rule for patterns in single quotes.
A hand-back states a read or a check as done only when it was done,
and later work is appended as new text and not written over what the report already says.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Briefs and hand-backs".

## M231: breaches of the Read, output, listing, batch and test-order rules by the fix round of the review (n166)

Status:
happened on 2026-10-15 in the fixes of Z1 to Z11 of the review of n162;
every one disclosed by its author (`s237/handback-n166.md`,
section 9).
The round ended green on its final source
(the suite at 2,445 passes and 0 failures on its second whole run;
the first was red on seven tests that pinned cache keys and seat order which the sheet text of Z11 moved),
and it is not landed.

- Reads with a limit over 50,
  in orientation:
  `group-line.ts` at 196 (70),
  `audit-sensitivity-print.unit.test.ts` at 1 (60),
  `audit-sensitivity-print.ts` at 50 (60),
  `editor-calibrate-lane.ts` at 50 (60),
  `stage-fanout-window.ts` at 100 (80),
  `diff-tests-a.txt` of the review at 132 (86)
  and `stage-settings.ts` at 1 (66);
  and one Read with no limit,
  of a file of 5 lines (`parser-lists.out`).
- Directory listings sent to the tool output and not to a file:
  the scratch folder of the review and its `probes`,
  and in the same stretch the scratch folder of the round,
  its `z9copy`,
  the probes again,
  and an `ls --almost-all` of a `node_modules`.
- Searches over 50 lines or with no cap:
  the test search for `ConsistencyInvariantError` (about 50),
  the search for `FanOutMode` (about 60),
  the search for the pairing readers (about 100)
  and a search for `export function`;
  the probe `z3-lane-starts.ts` printed about 65 KB to the tool output.
- Three test runs (judge,
  verify and deep) issued in one parallel batch,
  all reading `dist`.
- Several Edits to one file in one batch:
  `house-policy.ts`,
  `consistency-shown-kinds.ts`,
  `fragment-landing-error.ts`,
  `present-tense-text.ts`,
  `consistency-word-joints.ts`,
  four of `stage-quorum.ts`,
  others in the same round,
  and in the last stretch `translate-slice-attempt.unit.test.ts` (the test body and the import)
  and `probes/measure-n166.ts` (two edits).
  An `rg` of `present-tense-text.ts` ran beside Edits to other files.
- A nonsense Edit call to a file that does not exist (it failed),
  and four Edit calls to four probe copies that had not been read first (refused by the tool,
  then read and repeated).
- The helper `consistency-link-label.ts` and its unit test were written
  after the reds of `locateWords` and before the build,
  so the test of the helper was red at the start only because its export did not exist.
- The narrowed test of Z11 was not run against the start build,
  which no longer existed;
  its first,
  broader version was red at the start.
- Commands beyond the brief's list,
  all writing only in the scratch folder or the worktree of the round and read-only toward git:
  `cp` (backups and restores of mutated files,
  and three probe copies),
  `diff`,
  `tar` and `git archive` (the start source into `startcopy`),
  `ln --symbolic` (two links) and `sha256sum`.
- The replays ran with the brief's literal shell command and not as a systemd unit,
  since they finished in seconds.
- The probes `measure-n166.ts`,
  `q5c` and `q5d` are new files that the brief did not name.

Prevention:
the same as M210,
M214 and M216.
A new test of a new helper fails for the reason it was written for,
and not because the export it calls does not exist.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Tasks,
builds and bulk output",
and `mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## M232: breaches of the Read, output and batch rules, and a fix beyond the verb, by the archive fix round (n167)

Status:
happened on 2026-10-15 in the fixes of the seven findings of the third reading,
landed in `d6f7f9ae12`;
every one disclosed by its author (`s238/handback-n167.md`,
"Rules I broke").

- Reads with no limit,
  each of a file under 50 lines:
  `sections.mjs`,
  `conserve.mjs`,
  `addedbody.mjs`,
  `heads.mjs`,
  `pickaxe.txt`,
  `e17-stat.txt`,
  `sweep-count.txt`,
  `sweep-lines.txt`,
  `sweep-lines2.txt`,
  `corpus-lines.txt`
  and `pass-lines.txt`.
- Reads with a limit over 50:
  the ratio guard file with 56
  and `survey-before.txt` with 54.
- One `ls` of the handover folder with ignore patterns,
  badly filtered,
  sent about 70 lines to the terminal.
- A `rg` call with `--output=/dev/null` failed as an unrecognised flag,
  and a first `links.mjs` call passed `AGENTS.md` as a directory and crashed with `ENOTDIR`.
  Neither changed a file.
- `node --input-type=module-typescript --version` ran twice as a version probe;
  it was not `--eval`,
  and no code was evaluated.
- Fixes beyond the verb of the brief:
  the author applied clauses to six top index entries and rewrote one position pointer
  where the item's verb for them was "report" (M225),
  then reverted it all to the text of `c3c0c8dda7` and recorded the clauses as proposals.
  Before the revert the checks ran against the broader edit,
  and the author reran all of them after it.
- Hand-back handling:
  two Edits to the hand-back in one batch
  (the "Files changed" append and the "Hits" fix),
  an Edit of the hand-back and an `rg` read of it in one batch,
  and several sections appended per Edit where the brief said one Edit per section.
- The new holds paragraphs and index lines that carry full file names are over 120 characters,
  since the names alone are 100 or more;
  the archive already does the same.
  One other overlong line in the top index was left as it was.

Prevention:
the same as M210,
M214 and M216,
and the rule for the verb of a brief (M225).
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Tasks,
builds and bulk output" and "Briefs and hand-backs".

## M233: breaches of the Read, output, batch and foreground rules by the empty link round (n168)

Status:
happened on 2026-10-15 in the round that finished the apostrophe after an empty link (B619);
every one disclosed by its author (`s239/handback-n168.md`,
section 7,
"Every rule I broke").
The round ended green on its final source
(the suite at 2,446 passes and 0 failures,
lint,
types and the source scans clean,
the replays as expected),
and it is not landed.

- Two Edits to one file in one batch,
  twice:
  `consistency-fold.unit.test.ts` (two simplifications of a test body)
  and `consistency-word-joints.ts` (the two call sites of `apostropheJoinsAt` in `cutsWord`).
  Both applied without overlap.
- An Edit of `consistency-link-label.unit.test.ts` in one batch with an `rg` that read the file.
- A Read with no limit,
  twice (`lint-types1.log` and `lint-types1.err.log`,
  one line each),
  and seven Reads with a limit over 50
  (62,
  62,
  56,
  54,
  51,
  56 and 62 lines:
  `consistency-markup-ranges.unit.test.ts`,
  `consistency-prose-text.ts`,
  `consistency-markup-ranges.ts`,
  `consistency-fold.unit.test.ts`,
  `consistency-word-joints.ts`,
  and `consistency-folded-links.ts` twice).
- One `rg --line-number` over `consistency-fold.unit.test.ts` printed about 95 lines to the terminal and not to a file.
  A Read of three lines of `final-unit-2.log` was refused by the tool for token size,
  since those lines are very long,
  and the author searched the log with `rg --only-matching` instead.
- Short runs (a single test file,
  a probe of seconds,
  the copy of the dist of n166,
  `diff`) ran in the foreground and not as units;
  the builds,
  the checks,
  the suite,
  the measure and the large probes were units.
- A few independent commands ran side by side
  (two probes that read `src` and not `dist`);
  no two commands read or built `dist` at once.
- Not a rule breach:
  the author found and fixed a defect of its own at review.
  The first version of `linkLabelKindOf` read the children of a label in reverse,
  since its work stack was filled in child order and popped,
  so a label of several nodes was judged by its first shown character and not its last.
  Two rows showed it against the build that had it,
  the fix reversed the pushes,
  a guard was shown to fail,
  and every check was rerun on the fixed source.

Prevention:
the same as M210,
M214 and M216.

## M234: breaches of the Read, output, batch and search rules by the twin reading of empty links (n169)

Status:
happened on 2026-10-15 in the round that read an empty link as its plain-text twin in every reader (B619);
every one disclosed by its author (`s240/handback-n169.md`,
"Rules I broke"),
and the lead's notes list the same forms.
The round ended green on its final source
(the suite at 2,450 passes and 0 failures,
lint,
types and the source scans clean,
the replays as expected,
no real change on any of the 92 pages),
and landed on 2026-10-18 with n166,
n168 and n171 in `555bb74aa0` and `29d6064708`.

- Seven Reads with a limit over 50
  (61,
  62,
  70,
  85,
  52,
  53 and 70 lines:
  `consistency-folded-links.ts`,
  `consistency-markup-ranges.ts`,
  `consistency-unread-units.ts`,
  `consistency-prose-text.ts`,
  `consistency-word-joints.ts`,
  `consistency-shown-edge.ts`
  and `consistency-count.ts`).
- One `rg --context` over `mise.toml` printed far more than 50 lines to the terminal and not to a file.
- Commands that read a file in a batch with the command that wrote it:
  a file copy beside a checksum of the same files,
  earlier in the work,
  and,
  in the guard stretch,
  a `sha256sum` written to `sums-now.txt` in one batch with the `diff` that read it.
  The author checked afterwards with `sha256sum --check`.
- Two Edits to the one hand-back file in one batch.
- Two probes (`p2` and `p3`) ran at once as units;
  they read `src` and not `dist`,
  so no two commands read or built `dist` together.
- Short `diff`,
  `cp` and `sha256sum` calls ran in the foreground;
  the builds,
  the tests,
  the probes and the checks were units.
- The author searched its own session transcript under `/home/user/.claude/projects/` with `rg`
  to recover the exact unit commands of the earlier rounds.
  The lead's notes list it as a breach.
- A deviation the hand-back names under the replays:
  it ran each as a unit with `TRANSLATION_REPAIR_RUNS_DIR` set,
  and not in the literal form of the brief,
  with the same command.

The reaction is in the brief of the next round (`prompts/prompt-n173-labelled-links-read-as-shown.md`):
it opens its rules with "the last author broke the first three",
asks for a `limit` of 50 or less on every Read "even when you need a longer stretch",
counts a checksum or a `diff` of a file as a read of it,
and adds "Never search session transcripts".

Prevention:
the same as M210,
M214 and M216,
with the transcript search added to the forms a brief names:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs".

## M235: breaches of the output, Read, batch and search rules by the docs round for the mistakes of n162 to n170 (n172)

Status:
happened on 2026-10-15 in the round that wrote M224 to M233 and brought B619 up to date;
every one disclosed by its author (`s243/handback-n172.md`,
section 3,
"every rule I broke"),
and the lead's notes list the same forms.
The round landed in `3fdfa7422e` after the advisor's review,
and the fixes that followed (`c25e8f6b04` and `72789cd5f9`) corrected M224.

- Terminal output over 50 lines that was not sent to a file,
  twice:
  an `rg --files` listing of the `doc/` folder (about 130 lines)
  and an `rg '^#+ '` over five hand-backs (about 70 lines).
- One Read with a limit over 50:
  `mistake-prevention-shell-edits-commits-and-agents.md` at offset 100 with a limit of 60.
- An Edit and an `rg` that read the same file in one batch,
  twice:
  the word-test ledger file (a search for "labels of marks only")
  and the shell-commands ledger file (a search for a phrase of its opening).
- A command that wrote a log and a command that counted it in one batch,
  once:
  the count exited 1 and the log was read afterwards,
  and it held no finding.
- An `rg` with an invented flag (`--output-file /dev/null`),
  which exited 2 and touched no file.
- Two scratch outputs (`round-hits.txt` and `brief-template-hits.txt`) were written
  and read only after the advisor pointed it out;
  the zero was then checked with a positive control.
  Two Reads of `scans.log` printed very long single lines (test names).
- Three Edits of its own hand-back replaced text already in the file
  (two sentences of section 2,
  and an append of section 3 that replaced the last line of section 2),
  against the form the same round filed as M230 and as the rule "written once and then appended to".
  The author said so,
  and appended section 4 only.
- A gap in its checks that the hand-back names:
  the scans and the rendered check were not rerun after the edits that followed the advisor's review,
  where the lint and the five doc guards were.
  The first scan run had failed on `task-list-numbers.unit.test.ts` for a backticked pound sign and number in M230,
  which the author fixed;
  that is the guard working and no breach.

Prevention:
the same as M210,
M214 and M216,
with the rule of M230 that a hand-back is written once and then appended to
shown broken by the author who had just filed it:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs".

## M236: a brief named the review as the source of a cause that a fix round had found

Status:
happened on 2026-10-15 in the brief for n172 (`prompts/prompt-n172-docs-mistakes-n162-to-n170.md`,
item 2),
and was corrected by the author of the round from the sources
(`s243/handback-n172.md`,
section 1,
the first of three deviations).
The brief asked for B619 to be brought up to date and said "n162 traced the cause to `apostropheJoinsAt`".

- The review of n162 (`s233/review.md`,
  finding Z9) traced the cause to `closingJointsOf`,
  through `apartStartsOf`,
  and says that it had read that branch and had not instrumented it.
- The cause was found by n166,
  which instrumented a copy and showed that refusing the case in that branch changed nothing;
  the real cause was the last return of `apostropheJoinsAt` (`s237/handback-n166.md`,
  section 3).
  The lead's notes say that the pointer to the first function in the brief of n166 was the lead's,
  from n162,
  and that the brief of n172 repeated the attribution to n162.
- n172 read the sources and filed the history as they hold it:
  the review's pointer,
  the instrumentation that refuted it,
  and the real cause (B619).
  The advisor's review of the round made it rewrite M227 and the prevention text,
  which had read as a failure of the brief of n166 about that pointer;
  that brief said "confirm the cause by instrumenting a copy first",
  which is the instruction that found the pointer wrong.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs",
whose rule that a brief carrying a cause says whether the report instrumented it (M227) this repeats
for a brief that carries a cause into docs:
the cause is quoted from the report that names it,
with the report's own limit,
and the later finding is cited beside it.

## M237: a brief named a file for the brief rules that the repository does not hold

Status:
happened on 2026-10-15 in the brief for n172 (item 4),
and was found by the author of the round
(`s243/handback-n172.md`,
section 1,
the third of three deviations).
The brief asked for a stronger rule "in the brief template,
`doc/writer-brief.md` if that is where brief rules live:
find out".

- The author searched the repository's `doc/`,
  the package's `doc/` and `AGENTS.md` for a brief template and found none.
  The brief rules live in "Briefs and hand-backs" of `mistake-prevention-shell-edits-commits-and-agents.md`,
  and the stronger rule went there.
- The scratch folder of the work holds a file of that name,
  `writer-brief.md` (192 lines,
  last changed on 2026-10-06),
  but it is a brief for another job
  (verifying and fixing forward one contributor's production changes)
  and holds no template of brief rules.
- The brief put the question to the author ("find out") and named a path nobody had listed,
  and the lead's notes count the path as a mistake of the brief.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs":
a brief names a path only after the lead has listed it,
and a question the lead can answer by one listing is answered before the brief is sent.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Tasks,
builds and bulk output".
