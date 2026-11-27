# The notice of a stopped step keeps promising a stop

## Purpose

What a person reads from the notice of the `git commit` the wrapper runs in its scratch repository,
once the kernel was left holding that step.
It was found while making issue 6589's never-released control hold under a hold of its own,
and the owner has not chosen anything about it.
Nothing here is built:
no release behaviour changed with this proposal,
and the control that found it asserts what the release promises.

Unapproved and non-final.

## What was read

Two runs.

A gate run on 2026-11-27 failed that control
(`gpc-gate-issue-6562-clippy-bound-20261127T045928-0500`,
logged at `~/temp/agent/issue-6562-gate-clippy-container-gets-a-measured-memory-bound-scratch/keep/`,
which is scratch and expires):

```text
FAIL [  33.315s] (  776/12096) monochromatic-git-policy-cli commit::commit_preparation::tests::a_scratch_commit_the_kernel_never_releases_is_ended_with_a_message
```

Its standard error, in the order the lines were written,
with the three figures the record names:

```text
monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository (process 12876) was sent
 `SIGKILL` when its ceiling came after 1 seconds, and the kernel has not released it: it is in uninterruptible sleep
 (state `D`), so it ends when the call it is inside returns. Nothing was committed, and nothing of yours is at
 risk: your index and files are as they were. The terminal stays here until the kernel lets go.
monochromatic-git-policy-cli: still waiting for the `git commit` this commit runs in its scratch repository, which
 signs the commit because `commit.gpgsign` or `-S` asks for it; `gpg` (process 12880) runs below it, 10 seconds so
 far; it is stopped if it lasts 1 seconds in all. Hint: it is still running; if it looks stuck, check it with `ps`
 and report it.
monochromatic-git-policy-cli: still waiting for the `git commit` this commit runs in its scratch repository, which
 signs the commit because `commit.gpgsign` or `-S` asks for it; `gpg` (process 12880) runs below it, 20 seconds so
 far; it is stopped if it lasts 1 seconds in all.
[warn] [2026-11-27T10:19:40.642Z] [monochromatic-git-policy-cli] [runNativeCommit] the scratch git commit was
 stopped after 1306 ms at the bound of 1200 ms
```

The lines above were wrapped to fit this file; the log holds each as one line.
The warning is the command's own, so the `SIGKILL` was sent at 1306 ms,
and the two `still waiting` lines came 10 and 20 seconds later.

The second shape was reproduced on 2026-11-27 by a control's own plant and with no host load
(`commit::commit_preparation::tests::a_scratch_commit_the_kernel_never_releases_is_ended_with_a_message`,
the step's state and its uncollected end planted together):
the ordinary line was written at 10, 20, 30 and 40 seconds,
each saying `it is stopped if it lasts 1 seconds in all`,
while the wait's own line had already said the stop was sent and the kernel had not released it.

## What is not in question

The wait.
The release keeps waiting for the kernel after the stop, with no bound over that wait,
and that is the owner's answer of 2026-11-10,
"Keep waiting and say so",
which `package/git-policy/cli/SPEC.md` states:

```text
and the child is reaped there,
and a child the kernel holds in an uninterruptible call is waited for with no bound after that,
so the kernel's release of it is what ends the command
```

The stop's own words are right too:
they name the longer bound,
and they say
"The kernel held it in a disk wait, so it was waited for to that bound before it was stopped."
The kernel side is recorded in
[`doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`](../troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md).

## What is wrong

One line, written by the notice's watching thread
(`package/git-policy/cli/src/native/commit/scratch_step_notice_line.rs`,
its bound taken from the one stall decision both enforcers share).
After the ceiling stood aside (2026-11-12) that thread never breaks its loop:
only a stop its own ceiling sent ends it,
and the stop here was sent by the wait that owns the child.
So it goes on printing
"it is stopped if it lasts N seconds in all"
long after N, and after the stop it names was sent.
At 40 seconds a person reads a sentence that promises, in the future tense,
a stop that happened 38.7 seconds ago,
three lines under the command's own line saying it already happened.

The record of the kernel behavior says the notice's lines stop
(`doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`, "Symptom").
That was the shape before the stand-aside,
when the notice's thread sent the stop itself and ended.

## Options

### A. The line says the stop once the bound has come

Once the shared decision says the ceiling stood aside and the bound has come,
the ordinary line names the stop instead of promising it:
`it was stopped at N seconds in all, and this commit waits for the kernel to let it go`.

Pros:
the person keeps one line per interval through the longest wait this command has,
so a command that is still working never looks like one that hung;
the line stops contradicting the wait's own line, which is two lines above it;
the promise was already said once, in the line that said the ceiling stood aside.

Cons:
the notice's thread then decides that the bound has come from its own clock,
which counts from the first look that found the step and can differ by one look from the wait's own start;
two threads describe one wait, and the wording has to stay true whichever of them is ahead;
`notice_line` and its controls gain a case.

### B. The notice's thread says nothing more once the ceiling stood aside

Pros:
one voice for that wait, which is the wait that owns the child;
no second clock, and the least code;
the line can never say something the wait's own line does not.

Cons:
the wait that is longest is the one with no periodic line,
and the troubleshooting doc records that silence beside a killed process is what a person read as a hang
before 2026-11-10;
a wait of five minutes (the shipped bound) with nothing said after it starts is the shape this release
spent effort removing.

### C. Change the promise's tense everywhere

Every line says `it is stopped at N seconds in all`,
before and after the bound.

Pros:
one wording and no branch;
the hedge `if it lasts` goes, which reads oddly in a line that also says how long the step has already lasted.

Cons:
at 40 seconds a person still reads `it is stopped at 1 seconds in all`,
which names a stop that did not end the step, so the contradiction moves rather than goes;
the line still ends on the wrong news.

### D. Change nothing

Pros:
no release change at all;
the promise was kept, because the stop was sent at that bound;
the command's own line already says what the kernel did.

Cons:
two lines in one terminal contradict each other;
the ordinary line's hint
(`it is still running; if it looks stuck, check it with ps and report it`)
is said about a step that was already signalled;
the person cannot tell whether the command is still waiting or the terminal is done.

## Ranking

Ranked by what a person reading a terminal gets while the command cannot answer.
Effort is a cost stated in each option, never a reason for a place here.

Ranking:
A > B > C > D.

- **A over B**:
  A keeps the news that the command is alive and makes it true,
  while B keeps the truth and drops the news;
  the longest wait is exactly where a person needs to know the command has not hung,
  and B's silence at 40 seconds is the shape the troubleshooting doc records as a hang.
- **B over C**:
  B removes the contradiction,
  and C only moves it:
  a line saying the step "is stopped at 1 seconds" beside "40 seconds so far" still reads as a command
  contradicting itself.
- **C over D**:
  C at least stops the notice from promising a stop in the future tense after it was sent,
  and D leaves that promise standing,
  with the hint to check `ps` for a step that was already killed.

## What this proposal does not decide

- The exact words of any line, and which of the two threads says the last thing about the wait.
- Whether the notice's thread should keep looking at a step the wait now owns at all after the bound.
- Whether the record should note the moment the stop was sent beside `stoppedAfterMs`,
  which is the moment the bound came.
