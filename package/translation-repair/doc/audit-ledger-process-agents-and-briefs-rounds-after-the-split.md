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
