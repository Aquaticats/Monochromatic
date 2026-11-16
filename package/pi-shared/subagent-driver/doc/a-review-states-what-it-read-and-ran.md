# A review states what it read and ran

A review of a change is read afterwards by a program,
not by a model.
That program can tell whether the review file is there,
whether the review says what it read and ran,
whether its session shows the commands it names,
and whether any command ran a program a build wrote.
It cannot tell whether the review is right.

The project decided on 2026-11-12
(`doc/handover/cli-git-rust-implementation.md`,
the section dated `2026-11-12T12:26:24-05:00`,
"Who reviews a change"):
reviews stay with `qwen4-max-preview` alone,
every review brief demands that the review states how much of the diff it read
and what it ran through a built wrapper,
and a review that ran nothing is started again.
What was seen before that decision:
a review read about two fifths of a 3927-line diff,
ran nothing through a built wrapper,
and reported no defect;
another ended with no review file although a review was expected;
another ended in the middle of a tool call.
A second reading of each session found those hours later.

The project decided on 2026-11-15
(the same handover,
"Owner decisions in force",
"Answers of the evening,
the long sitting")
that a thin review is started again **once,
by itself,
on a different model**:
"The other of the pair;
Sonnet while Qwen is dry".
So a first review that read less than it was given,
or ran nothing a build wrote,
is answered by one second review on the other reviewer of the pair,
and a second review that is still thin is reported and left for whoever coordinates,
never started a third time.

## The statement

Every review brief written by `write-a-brief-for-a-review`
asks for one section,
first after the review file's title,
holding one JSON object in a fenced block tagged `review-statement`:

````markdown
# Review: a change

## What this review read and ran

```review-statement
{
  "commit": "<the commit this review reads>",
  "diff": {
    "lines": "<how many lines the diff file holds>",
    "linesRead": "<how many of those lines this review read>"
  },
  "files": [
    { "path": "<a file the diff touches, from the top of the checkout>", "read": "whole" }
  ],
  "commands": [
    {
      "command": "<a command this review ran, as it ran it>",
      "exit": 0,
      "ranBuiltProgram": "<the path of a program a build wrote>"
    }
  ],
  "findings": [
    { "finding": "<the finding's number or short name>", "shownBy": "run" }
  ]
}
```

## Findings
````

Through `briefs/review-of-a-change-template.ts`,
the brief asks for it and says what will be compared.
The shape itself lives in `src/review-statement-shape.ts` (`REVIEW_STATEMENT_HEADING`,
`REVIEW_STATEMENT_LANGUAGE`,
the types,
and `REVIEW_STATEMENT_SKELETON`,
which the brief carries),
the reader that holds a block to it in `src/review-statement-fields.ts`,
and the reader that finds the section in `src/review-statement.ts`.

How a review says how far it read a file:

- `whole`:
  it read the file as the commit holds it.
- `part`:
  it read some of it.
- `none`:
  it read none of it.

A review that read part of the diff and says so is worth more than one that claims all of it:
the check names the files not read whole as work left for a second review,
never as a defect.

## The check

`check-a-review-that-ended` reads each review's review file,
its newest finished run's
`summary.json`,
the session file that summary names,
the `change.diff` beside the review,
and `what-the-review-read.json`.
It answers one verdict per review,
with its reasons on the same line.

### What is compared

- Each `commands` entry's text against the `bash` calls and `process` starts the session records,
  with the shell's line breaks and runs of whitespace collapsed.
- The exit code a statement gives with the code the matching `bash` call's result ends with.
  Only a result Pi marked as an error carries the harness's `Command exited with code` line:
  a result it did not mark is a command that exited zero,
  so a command that printed such a line itself is read as zero,
  not as the line it printed.
- A `ranBuiltProgram` path against the command that ran it:
  a program in a build output folder,
  which is one whose path holds a `target` segment,
  a `dist` segment,
  or a `debug` or `release` segment of a Cargo target directory,
  **in the review's worktree or in its own review folder**.
  A program from the main checkout,
  or from a build folder tied to no worktree,
  is not confirmed:
  nothing this check reads ties it to the reviewed commit.
  That is a stated limit, not a mismatch the review caused;
  a review that relied on one says where it was built.
  Folders are compared after every existing symbolic link is resolved,
  so the aliases `/home/user`,
  `/var/home/user` and `/var/mnt/encrypted` still decide nothing.
- A program is counted only when the command **runs** it:
  a word in a segment's program position,
  the script an interpreter or shell runs,
  or a program after `sudo`,
  `env`,
  `timeout`,
  `nice`,
  `xargs` or `systemd-run`.
  `ls package/a/dist/x.js` names a path and runs `ls`;
  `node package/a/dist/x.js` and `dist/x.js --version` run it.
- A relative program word is compared by its tail,
  because a command that moved to another folder (`cd package/a && ./target/debug/a`)
  is not followed;
  a word naming a shell variable is compared by its file name.
- A command that is a script the command runs:
  its text is read,
  and a program path it holds confirms the statement's `ranBuiltProgram`.
  A script the command only names,
  as `cat` does,
  is not read as run.
- `files` and `diff.lines` against the diff file the brief wrote,
  whose files are read from its `+++ b/` and `--- a/` lines when the header alone is ambiguous
  (a path holding ` b/`,
  or a rename).
- The `commit` the statement names against the commit `what-the-review-read.json` recorded.
- The run's `endReason` in its summary,
  which says whether the run ended by itself or was cut.
- Whether the session records a build command,
  which the verdict says:
  a program that ran from a build folder is confirmed either way,
  and the tie from that program to the reviewed commit is the statement's claim.

### What it cannot see

- A build's provenance:
  it reads a program's path and the session,
  not a build's records,
  so it confirms that a program from a build folder ran and leaves "from this commit" to the statement,
  whose commit it compares with the one the review recorded.
- A command run inside a `codemode` call:
  the session records the call,
  not the commands it made.
  A session holding one says so in the verdict.
- A `process` start's exit code:
  its result records none,
  because the process is reported on later.
  Such a claim is reported as unconfirmed,
  never as a match.
- A script that builds a built program's path from a variable rather than holding it.
- Text inside a here-document or a quoted shell string of a stated command,
  which this reader does not separate from the command itself.
- Anything written into the session after the run ended.
- Whether the review is right about what it found,
  and whether a file it says it read whole really was read.

### The verdicts

- **complete**:
  the review file is there,
  its statement matches its session,
  at least one command ran a program a build wrote,
  and the run ended by itself.
- **ran nothing**:
  the statement and the session agree,
  and no command ran a program a build wrote.
- **statement missing or not matching its session**:
  no statement section,
  a statement the reader refuses,
  a command the session does not record,
  an exit code that is not the one recorded,
  a file or line count that does not match the diff,
  a commit other than the one recorded,
  or a built program the session shows and the statement does not name
  (when the statement confirms none).
- **no review file**:
  the run ended by itself and wrote no `review.md`.
- **cut**:
  the run ended on a turn cap,
  a wall clock,
  an abort or a model error,
  or no run of the review finished at all.

A review is **thin** when its statement is readable and either it did not read every file it was given whole,
or no command it lists ran a program a build wrote.
A `complete` review can be thin;
a `ran nothing` review is thin by definition.
A thin review does not make a review file wrong:
the check names what is left and leaves the judgement to whoever coordinates.

The command exits `0` only when every review it was given is `complete` **and** not thin,
and names what it could not see on its last line.

## Starting a second review

A first review that is thin or incomplete is answered by one second review,
started by the command itself;
a person needs no flag.
A second review is the review's own name with `-restart-<n>`:
`review-<change>-restart-2`,
on branch `test/review-<change>-restart-2`,
in the folder `<reviews folder>/<change>-restart-2`.
The numbers count the review itself as the first run.
The command starts it by itself;
`--start` is accepted for a caller that already passes it and changes nothing.

- Its worktree starts at the commit the earlier review recorded it read.
- Its brief is the review brief,
  opened with a paragraph saying what the earlier run did:
  why it ended,
  the verdict,
  how much it read and what it ran,
  and where its own review file and reproduction scripts are.
- Its diff is the whole earlier diff when the earlier run left no complete reading,
  and only the files the earlier review did not read whole when it did.
- It runs on **the other reviewer of the pair**:
  the model the first run's session names,
  read from its `model_change` record,
  decides which;
  a session that names no model is read as one that ran on `qwen4-max-preview`,
  so its second review goes to Claude Sonnet.
- The model of each start is recorded twice:
  in the second review's `what-the-review-read.json`,
  under `reviewer`,
  and in its own run's session,
  so a third start (which never happens) would still know both.

### A Pi model, or a note for a harness

Pi on this machine lists no Claude Sonnet model
(`pi --list-models`,
2026-11-15:
one Claude model,
`claude-haiku-5-5` on `opencode-go`).
When the other reviewer has a Pi provider and model,
the command writes the batch list and starts it.
When it has none,
the command writes the second review's brief,
diff and reading,
and a note named `<reviews folder>/<change>/restart-<n>-second-review-owed.md`,
and starts nothing.
The note names the first review's own line,
the reviewer owed,
the commit,
and the brief a harness with that model reads,
so whoever coordinates can hand it on without reading the command's output.
A later check says the same line and writes no third review.
When Pi lists that model,
one edit to `REVIEWING_PEER` in `src/reviewer-of-a-change.ts`
gives it a provider and model,
and the command starts its next second review instead.

### Once only

- A second review is written for a first review only:
  the review the check reads carries no `-restart-` in its name.
- A second review that is still thin or incomplete is reported with what it read and ran,
  and never started a third time.
- A second review that already wrote its review file is reported by name,
  and nothing is started.
- A second review whose run has started and not ended is reported,
  and nothing is started.

### What guards a second review's files and its start

- **No file is replaced.**
  Each file is created with an exclusive create (`wx`),
  so a text that appeared after the refusal check makes the create fail and is left where it is.
  A file that already holds the plan's own text is left;
  a file holding the beginning of it,
  which a kill cut short,
  is removed and created again,
  and nothing else is written over.
- **A plan written and never started is started by a later check,
  and a brief a kill cut short is repaired.**
  The plan is the same number,
  its texts are written again,
  and the start marker decides whether a start already began.
- **One start per second review.**
  The start marker is created with `wx` just before the launcher is asked,
  and names its process.
  A marker whose process is alive is somebody else's start and is not taken;
  a marker whose process is gone and left no run folder is a start that died,
  and the next check takes it over once.
  Two concurrent checks therefore write one second review and start it once.

## What this is not

The check books evidence and coverage,
not quality:
it never says a finding is right,
a reading was careful,
or a review is worth acting on.
It never decides that a thin review is wrong;
it names what is left,
and a second reviewer reads it once.
