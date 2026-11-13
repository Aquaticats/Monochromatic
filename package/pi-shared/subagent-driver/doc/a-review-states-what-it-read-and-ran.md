# A review states what it read and ran

A review of a change is read afterwards by a program,
not by a model.
That program can tell whether the review file is there,
whether the review says what it read and ran,
whether its session shows the commands it names,
and whether any command ran a program the review built.
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
  "diff": { "lines": 0, "linesRead": 0 },
  "files": [ { "path": "<a file the diff touches>", "read": "whole" } ],
  "commands": [
    { "command": "<a command this review ran>", "exit": 0, "ranBuiltProgram": "<a path under target/ or dist/>" }
  ],
  "findings": [ { "finding": "<its number or short name>", "shownBy": "run" } ]
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
- A `ranBuiltProgram` path against the command that ran it,
  or against a script that command names:
  a path under `target/` or `dist/` of the review's worktree,
  named in the command itself or written inside a script the command runs.
- `files` and `diff.lines` against the diff file the brief wrote.
- The `commit` the statement names against the commit `what-the-review-read.json` recorded.
- The run's `endReason` in its summary,
  which says whether the run ended by itself or was cut.

### What it cannot see

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
- Anything written into the session after the run ended.
- Whether the review is right about what it found,
  and whether a file it says it read whole really was read.

### The verdicts

- **complete**:
  the review file is there,
  its statement matches its session,
  at least one command ran a
  program built from the reviewed commit,
  and the run ended by itself.
  Spelling out the files it did not read whole does not make a review incomplete.
- **ran nothing**:
  the statement and the session agree,
  and no command ran a program built from the commit.
- **statement missing or not matching its session**:
  no statement section,
  a statement the reader refuses,
  a command the session does not record,
  an exit code that is not the one recorded,
  a file or line count that does not match the diff,
  or a commit other than the one recorded.
- **no review file**:
  the run ended by itself and wrote no `review.md`.
- **cut**:
  the run ended on a turn cap,
  a wall clock,
  an abort or a model error,
  or no run of the review finished at all.

The command exits `0` only when every review it was given is `complete`,
and names what it could not see on its last line.

## Starting a review again

For a review whose verdict is `ran nothing`,
`no review file` or `cut`,
the command writes the entry that starts it again.
It starts nothing itself unless the caller adds `--start`.

- The restart is the review's own name with `-restart-<n>`:
  the run `review-<change>-restart-2`,
  on branch `test/<change>-restart-2`,
  in the folder `<reviews folder>/<change>-restart-2`.
  The numbers count the review itself as the first run.
- Its worktree starts at the commit the earlier review recorded it read.
- Its brief is the review brief,
  opened with a paragraph saying what the earlier run did:
  why it ended,
  the verdict,
  how much it read and what it ran,
  and where its own review file and
  reproduction scripts are,
  so the second run does not repeat a reading the first left off.
- Its diff is the whole earlier diff when the earlier run left no complete reading,
  and only the files the earlier review did not read whole when it did.
- The batch list is `<reviews folder>/<change>/restart-<n>-batch.json`,
  one entry,
  so the launcher takes it as any other batch.

Nothing is written over.
A brief,
a diff,
a reading or a batch list that already holds other text refuses the whole restart
and names the file;
a file that already holds exactly the restart's own text is left alone.
A restart whose review file is already there is refused as one that already ran.

A restart that has been written and has not ended (its brief is there,
its review file is not,
no run of it finished) holds back the next one:
the command says which run is still pending instead of writing a third.
Once a restart ends,
the next number is written,
and its brief says it is the third run.
So a review is never started a third time without the output saying so.

## What this is not

The check books evidence and coverage,
not quality:
it never says a finding is right,
a reading was careful,
or a review is worth acting on.
A review that states it read part of the diff is `complete`,
and what the package does with the rest is name it.
