# Audit ledger: process mistakes, agents and briefs in the rounds after the split

What went wrong in the briefs and hand-backs of the rounds that ran after the split of the package into 36
on 2026-10-18 (UTC):
two briefs whose search text was never run against the thing it searches (M259 and M260),
and a report that the tool refused to write (M261).
The first two are the lead's own.
The third is the tool's behaviour and no agent's breach,
and is filed because it came again.

M304 was added on 2026-10-29 (UTC) from the hand-back of the agent of round 8,
which listed the rules it broke.
M312 was added the same day from the lead's notes on the caps round,
whose brief named a census root that a live pass writes to.

Written on 2026-10-19 (UTC) from the lead's notes after the split (`s245/ledger-notes-after-split.md`).
M270 was added on the same day from the notes of the seat round (`s245/w191-notes.md`).
The entries before these are filed in
[Audit ledger: process mistakes, agents and briefs in the review, docs, archive and empty link rounds](audit-ledger-process-agents-and-briefs-review-docs-archive-and-empty-link-rounds.md).
Paths here are written in the layout after the split.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M259: a brief gave a proof pattern that matches nothing in the bundle it was meant to prove

Status:
happened in the brief for the comparison of the lint of every package under the baseline bundle and under the hardened
bundle of the shared analysis (`w177`).
The brief gave the proof that a bundle is the one in use as a count of matches of `update\('9'\)`.
The built bundle writes that call with the digit between backticks,
``update(`9`)``,
so the pattern with single quotes matches nothing in either bundle.
Every `proof-*.json` therefore records a zero.
The agent used `update\(.9.\)` instead,
where each dot stands for any one character.

Prevention:
a pattern that proves which bundle is live is run against both bundles before it is written into a brief,
and it is shown to find the mark in the bundle that holds it
and to find nothing in the bundle that does not,
the positive and the negative control.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs".

## M260: a brief gave a pathspec that matches no file below the directory it names

Status:
happened in the brief for the account of the cache versions since the round n171 (`w185`).
The brief gave the pathspec `'package/translation-repair/*/src'`,
which matches no file below `src`.
It returned 10 commits,
all of them before the split,
and would have left out 40 production commits.
The agent caught it and used `'package/translation-repair/*/src/*'`.

Prevention:
a pathspec in a brief carries a positive control:
a commit known to touch the path must appear in its result before the result is used.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs".

## M261: the tool refused an agent's Write of its report, as it had refused n170's

Status:
happened in the guard demonstrations for n168,
n169,
n171 and n173 (`w181`).
The harness refused the agent's Write of `report.md`,
and the hand-back text was the report.
The agent's tables of results are in `w181-notes/report-tests.md` and `w181-notes/report-tables.md`.
The same refusal met n170's Write of its report on 2026-10-15
(M226 and the introduction of
[the file for the review, docs, archive and empty link rounds](audit-ledger-process-agents-and-briefs-review-docs-archive-and-empty-link-rounds.md)),
which also returned the report as text,
and the file for those rounds does not count that refusal as a breach.
The notes record this one as the harness's refusal and not as a slip of the agent.

Prevention:
the notes name none.
A brief for a review or a demonstration that produces a report names the hand-back text as the report,
and a report file as optional,
so that a refused Write loses nothing.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs".

## M270: the report that began the seat round counted the cut Qwen replies as usable and gave them no answer text

Status:
happened in the round `w191` for the pipeline defects of the pass `TianqiChen66626`.
The report that the brief was written from held two premises about the Qwen3.8-27B replies that the rounds cut:
that the census counted them as usable,
and that they had no answer characters and 30,000 to 63,000 characters of reasoning.
The agent checked both against the log and corrected them:
the census counted the cut replies in no column at all (B687),
and 3 of the 58 cut replies had answer text,
with reasoning that starts at 18,470 characters.
The agent also checked that the log path named in the brief is the real file.
The notes do not say who wrote the report.

Prevention:
the notes name none.
Written in this filing and not given by the notes:
a premise in a brief about a count or about the size of a text is recounted
from the log it cites before the brief is sent,
or the brief marks it as unchecked and the agent measures it first,
as this agent did.
`mistake-prevention-guards-searches-and-claims.md`,
"Claims without their evidence".

## M304: the agent of round 8 broke the shell, edit, output and worktree rules and said so in its hand-back

Status:
recorded on 2026-10-29 (UTC) from the agent's hand-back
(`report.md` of `w357-dx1r8`,
outside the repository).
The work is in the file of the footnote readings,
round 8 (B970 to B979).

The agent listed these breaches:
- Repository files were edited through `node --eval` and `sed` several times,
  and not through the Edit and Write tools.
- One command was chained with `;`,
  and another git call was left outside the keyless wrapper.
- Very large outputs were read in full:
  a list of docs from a scan failure,
  a listing of the `altfuzz` runs,
  and the whole `git worktree list` at the end.
- Evidence was capped with `rg --max-count` and `rg --context`.
- Placeholder commands were run to see what a script prints:
  `search.ts --help` and `pair.ts --help`.
- Scratch worktrees `base`,
  `red`
  and `red2` were added to the shared `.git`
  (removed at the end),
  and `git -C base checkout --detach` was used to bisect.
- A red case was made by temporarily reverting a fix file (`a9b77b764e`,
  `4a70e75162`).
- The growth case of the placing (B977) was not red first.
- `mise run` was run through a wrapper script that spawned `systemd-run`.
- One `rg` call had a stray empty pattern argument;
  it failed with no effect.
- A directory with one symbolic link to the `monochromatic-lint` binary of `.cargo/bin` (version 0.3.0,
  which is the pinned version) was added to the keyless `PATH`,
  because `cli-git` refused to stage a Markdown file that the policy lints when it found no such program.
  Nothing was installed and no guard was bypassed.

Cause:
the tree of the agent predated the cutover of the toolchain and had never been synced (B980),
so `monochromatic-lint` was missing from its `PATH`,
and the symbolic link stood in for the tool that `toolchain:sync` would have made.

Prevention:
`mistake-prevention-tasks-builds-and-bulk-output.md`,
"Gate trees and their tools",
for the cause,
and the rules the agent broke stand as written in `mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands",
"Lint and edits" and "Briefs and hand-backs".
A brief names the source-scan Markdown check and not the root task (M300),
and the lead syncs the tree of an agent that merged main across a change of the toolchain.

Recurrence:
M116,
which records that writing a slip down has not stopped it.

## M312: the brief for the caps round named a census root that a live pass writes to, and four more slips followed

Status:
recorded on 2026-10-29 (UTC) from the lead's notes.
The round is the caps round (`tr-w360`,
a Sonnet agent);
its work is B990 and B991.

- The agent ran the cap census over all of `~/temp/agent`,
  as the lead's brief said.
  The census therefore read the logs of the live `corpus-pass-XingZ6022` pass.
  It only read,
  and nothing was touched.
  The brief,
  and not the agent,
  named that root.
- The agent's first placement of the ledger entries took `audit-ledger-runner-reports-and-probes.md` to 1042 lines.
  B990 and B991 were moved to `audit-ledger-the-cap-census-and-the-xiaomi-cap.md` in `e8fb13c2a3`.
- A Markdown finding needed `101f3f8c35`,
  and a pinned test that had gone stale needed `af7d6ddbf8`.
- The message of `d599b87fab` says "none at the cap".
  That was true of the old cap of 13,082 only.
  At the new cap of 5,098,
  3 of the 293 calls of Xiaomi's own endpoint reach it.
  Commit comment 207194488 corrected the message,
  and `5f9a726c61` (the lead's) corrected the comment on the card in `model-cards.ts`.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs",
for the first:
a brief for a census names run directories that no live pass writes to.
"Commit messages" for the last:
a count that a message gives for a constant is read again after the constant changes,
and the message names the constant the count is for.
`mistake-prevention-docs-and-references.md`,
"Docs that only grow",
for the second:
the lines of every Markdown file a commit touches are counted before the commit.

Recurrence:
M304,
the slips of the agent of round 8,
M291,
where blocks were added to files near a line limit,
and M300,
where a Markdown finding reached a gate.

## M321: the agents of seven rounds broke the shell, edit, output, wait and worktree rules, and the lead found it in their hand-backs

Status:
recorded on 2026-10-29 (UTC) from the lead's notes of each hand-back.
The rounds are w363 (B996),
w366 (B1005),
w369 (B1020 to B1025),
w370 (B1030),
w374 (B1045 to B1047),
w376 (B1035)
and w377 (B1040),
and the read-only measurement of B1047.

- Round w363:
  the first provider build ran without `cd` and wrote ignored build output (`dist/final/node`) in the main checkout
  for eight packages
  (markup,
  document-structure,
  shared-contract,
  typography,
  glossary,
  house-rule,
  provider-stream
  and provider);
  tracked status did not change,
  and the output was left in place because it is ignored and other sessions build there.
  The agent's case R1 made a microtask spin of about 180 seconds,
  a busy loop under the ban on synthetic load,
  and `TaskStop` ended it.
  A `timeout` wrapper went round the run of R9,
  two commands ran without `PATH`,
  and some `rg` outputs were too wide.
- Round w366:
  many parallel Edits to one file,
  an Edit and its reader in one block,
  an `ls` of the harness task folder (refused),
  `rg --max-count` caps,
  a `ps -eo` listing of all processes,
  and the source-scan unit tests run four times.
- Round w369:
  parallel Edits to one file,
  an Edit and a build in one block,
  `head -n 6`,
  `sleep 1`,
  whole large logs read,
  and three waiters of the form `timeout 3300 tail --pid=<its own shell>`,
  which sat idle for 40 to 50 minutes until the lead stopped them by process id.
- Round w370:
  one `;` chain,
  parallel Edits to one file three times,
  and polls of `git log`.
- Round w374:
  two scripts of `node` that patched files instead of the Edit tool,
  commands without the keyless prefix,
  two Edits to one file in one block,
  one extra full run of the slice-reading tests,
  and a `mise run sync:files` that rewrote the tracked `mise.toml`
  (the agent restored it by a shell write)
  and wrote a rules file of 11,064 bytes (M317).
- Round w376:
  one `;` chain,
  one `for` loop (read-only),
  two Edit and read pairs in one block,
  and the source-scan unit tests run five times to fix Markdown line breaks.
- Round w377:
  three pairs of Edits to one file,
  short flags,
  and the source-scan unit tests run three times.
- The read-only measurement of B1047:
  one wait on a process id found with `pgrep`,
  which returned at once.

Counted by round,
two Edits to one file in one block were made in five of the seven rounds,
an Edit with the command that reads it in three,
and a `;` chain in two.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands",
"How an agent waits"
and "Briefs and hand-backs":
the rules stand as written,
a brief opens with the forms broken and the number of rounds that broke each,
and a wait is on a process id its sender started.
The lead's notes say that future briefs add the last rule:
wait only on a process id the agent started,
never on one found by `pgrep`,
and never on the agent's own shell.

Recurrence:
M304,
the breaches the agent of round 8 listed,
M272 and M273,
breaks that agents listed in their own notes,
and M189 to M197,
where agents sent two or three edits to one file in one batch.

## M322: seven messages of commits said what their diffs did not, and five subjects or bodies passed the length limits

Status:
recorded on 2026-10-29 (UTC) from the lead's notes.

Messages that were wrong:
- `8bc09565ec` (w363) said 12 reads and 10 listed.
  The count is 13 reads,
  2 converted
  and 11 listed in 6 keys.
  `61b56b3602` and B996 correct it.
- `c4d4c80277` (w367) gave counts of cases that did not match.
  The file holds 8 cases,
  2 of them new,
  and 4 of the 8 fail.
  B1011 corrects it.
- `325cfb3585` (w369) claimed a fix where the commit holds tests.
  The change of verdict is in `add3f1b27e`.
- `49561dcba9` (w369) said that existing cases follow generation 22.
  Three run test files with 23 literals of 21 did not (M323).
- `65ae605fd2` (w371) said seven new cases.
  The diff adds eight.
- `1f2bb6aef6` (w371) omitted that the commit also rewrote the red test file
  (`TWO_GAPS_ENTRY`,
  `baseSignal`
  and `coverageScript`).
- `b3e4d5e2cc` (w364) said 4,030 and "one more than the lead's 4,029".
  The count is 4,031.

Corrective comments were posted for all of these once the commits were on GitHub
(207247634,
207272199,
207272204,
207272209,
207272214,
207283978
and 207298662).

Messages that were too long,
for which the lead posted no comment:
- the subject of `6253eb52bc` (w368) was 109 columns,
  and the lead's own `1787ded` was 116;
- the subjects of `65ae605fd2` and `227d4118b6` (w371) were long;
- the body lines of `e539103c5d` (w370) passed 120 columns.
The lead's ruling,
with the earlier `bd298c1d40` (114 columns) as its precedent,
is that length is style,
and a corrective comment is for a message that says something wrong.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Commit messages":
the rule that a count comes from a command run on the staged diff stands,
and the limit of 100 columns for a subject and 120 for a body line is written there,
counted by a command before the commit.

Recurrence:
M267,
a body that miscounted the guards it pins,
M274 and M281,
messages checked against their diffs,
and M312,
a count that a message gave for a constant it had replaced.

## M323: the merges of rounds dropped a line, missed the tests that print a constant, and carried a break outside the conflicts

Status:
recorded on 2026-10-29 (UTC) from the lead's notes.

- The merge in `tr-w367` removed the index line of B996 from `audit-ledger-entries-by-id.md`,
  on the ground that main had deleted it.
  Main had never had the line:
  round w363,
  which added it,
  had not landed.
  The lead caught it and restored the line in a commit on `tr-w367`.
- Round w369 moved the artifact generation to 22
  and missed three run test files that print `read-at-generation=21`
  (verify-published,
  verify-published-run
  and verify-published-entry),
  23 literals in all.
  Gate g235 was red in `run.test-unit` with 20 failing cases and 3 suite lines.
  The hand-back listed no run of the run package's tests.
  The lead moved the literals and gate g235b was green.
- Round w371 had added a case in `verify-published-run.unit.test.ts` that expected `read-at-generation=21`
  while round w369 moved the generation to 22.
  Gate g237 was red in `run.test-unit` on that one case and its suite line.
  This one was a real merge interaction,
  and the lead's fix is in `b74f010d74`.
- The merge agent of round w376 kept a `parked` hook that `702d149a74` had made dead,
  and `lint:types` of corpus-pass failed with 4 errors until `00c80c7d0d`.
  It also dropped a check of `passBoundOf` and the start of the day in `run-seats-futile.ts`
  because the pass bound was gone.
  That the behaviour stayed the same was the agent's reading and not a run,
  and gate g239 had to confirm it.

Prevention:
`mistake-prevention-merging-and-landing-rounds.md`,
"Lines a merge keeps or drops",
"Constants that tests print"
and "Behaviour a merge carries outside its conflicts".

Recurrence:
M174,
a merged tree that held three slips the gate found,
M238,
a merge of two rounds' work on one file,
B701,
two identical additions that git merged without a conflict,
and M310,
two rounds that collided on the ledger indexes.
