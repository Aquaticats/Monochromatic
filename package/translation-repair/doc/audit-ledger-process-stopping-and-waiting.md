# Audit ledger: process mistakes, stopping and waiting

Turns that ended with work still listed,
and waits spent polling.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M101: a turn ended on a status report while the batch was still open

Status:
happened 2026-10-01 (UTC) at the close of T8's nineteenth batch,
caught by the owner asking whether the work would go on.
With the fix,
the frame case,
the ledger record
and the prevention section committed (`729312bb4`),
the turn ended on a summary that listed the open steps itself:
`status.md`,
the handover,
the full suite
and the twentieth batch.
It gave a shrinking context window as the reason,
though the harness summarizes a long session and work continues after it.
The same stop happened once before in this audit,
and the owner's answer then was to tighten the rule against it.
Prevention:
a turn ends only when the tracked queue is empty or a step needs the owner;
a shrinking context is handled by committing and going on,
and a summary that names a next step the agent could take itself is the sign it must not stop.

## M149: an agent read the session's task output files while it waited

Status:
happened on 2026-10-11 (UTC),
in the agent that built B425;
disclosed in its own report.
While its suite ran,
it read one background task's output file under the harness's task folder,
then stopped and read its own scratch logs instead.
Its brief forbids reading those files,
since they hold whole transcripts and other tasks' output.
No key or environment was read or printed.

Prevention:
the rule stays in every brief and is restated in every continuation brief;
an agent waits on its own scratch logs.
`mistake-prevention.md`,
"Tasks,
builds and bulk output".

## M162: a brief forbade the ways of waiting and named none, so its agent polled

Status:
happened on 2026-10-12 (UTC) in the consistency review's fixes.
The brief and the shared facts it cites forbade `sleep` and shell loops
and said to run the whole unit suite once,
but named no way to wait for a command longer than the Bash timeout.
The agent started the suite with the Bash tool's background mode,
armed three Monitors (the first matched its own `ps` filter and was stopped),
and counted the suite log's lines in about twenty calls;
one earlier call ended in `; echo done`.
The owner saw it and asked for better briefs.
The lead sent the agent the unit form and a blocking wait the same day;
the agent ran its last suite as a unit and waited on it with `wait-unit.ts`.
Its hand-back disclosed four more slips:
a file read with `sed --quiet`,
a backslash-u escape typed once in a probe,
eight throwaway probe files made and removed under `src`
while it looked for a lint panic's trigger,
and a build of the git-policy CLI's dist so one test file could import it.
None reached the patch.

Prevention:
the shared facts now give the waiting rule:
foreground with the Bash timeout at 600000 for a command inside ten minutes,
otherwise a `systemd-run --user` unit and one blocking call to `wait-unit.ts`,
and no Monitor,
process loop
or line count.
`doc/mistake-prevention.md` (moved on 2026-10-18 to `package/translation-repair/doc/mistake-prevention.md`) holds it
under "How an agent waits".

## M296: a branch was held for a clean re-check, and there was no rule for when it lands

Status:
recorded on 2026-10-29 (UTC) from the lead's notes.
The owner's rule is dated 2026-10-25 ("Land on real impact").

What happened:
the Claude route took 20 re-checks and 22 fix rounds,
and the lead's count of what they found is that only re-checks 13 and 14 found a major issue.
The cause the lead recorded has four parts:
no stopping rule,
landing only on a clean re-check,
guard evasions that only adversarial inputs reach treated as mistakes of the package,
and a round for each nit.

The rule as the owner gave it,
checked against the wording that the ledger already held:
the record of the plain reading with tables and the containers states the first half
(a branch lands once a re-check finds no major issue and nothing that changes behaviour on real sources or the real
corpus,
and gaps that only invented inputs show become stated limits and a later item)
and lacked three parts.
They are that behaviour is judged on runs as well as sources and the corpus,
that nits go into the same docs round and not a round of their own,
and the measure of what the missing rule cost.
They are written here and in the prevention.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs",
beside the rule of M219 that a review loop stops when the checks that measure what ships stay identical.

Recurrence:
M218 and M219,
ten fix rounds without a commit and a review loop that was not stopped.

## M301: the output of a background command was read before the harness reported that it had finished

Status:
recorded on 2026-10-29 (UTC) from the lead's notes.

Two reads came too early.
A commit's result was read before the commit finished,
so the head looked unchanged.
A log of comment posts was read after one of three posts.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"How an agent waits":
the output of a background command is read only after the harness reports that it finished.

Recurrence:
M162,
where a wait was replaced by counting the lines of a growing log.

## M305: the output of a background command was read before the harness reported that it had finished, three times in a day

Status:
recorded on 2026-10-29 (UTC) from the lead's notes,
after M301 and its rule.

The lead's count for 2026-10-29 is three reads that came too early:
the log of a comment post,
the output of a commit (the file was empty),
and the log of a worktree retire (the file was empty).
M301 records the first two kinds;
the notes do not say whether the first two reads are the ones M301 records.
The retire log is the kind it does not record.
Two of the three files were empty when read,
and an empty file was taken to say that the command had printed nothing,
where it said that the command had not yet finished.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"How an agent waits".
The rule of M301 stands,
and its wording is tightened where this slip showed a gap:
an empty output file of a background command is the sign of a command still running,
never a result,
and the turn ends until the harness reports that the command finished.

Recurrence:
M301,
the same read made twice the same day,
and M162,
where a wait was replaced by counting the lines of a growing log.

## M313: the output of a background command was read before the harness reported that it had finished, eight more times

Status:
recorded on 2026-10-29 (UTC) from the lead's notes,
after M301 and M305 and the rule they tightened.

The notes name eight reads that came too early since those two entries.
Each is named here by the task or file the lead read:
- the output of `bvolyk6xb`,
  which the notes count as the fourth such read;
- the output of `bhde0vtf3` and the file of `b2tmdhc5v`,
  two reads of one stretch;
- the output of `bdgo1zmt4`;
- the output of `bnwokco8f`,
  the commit that fixed the kept list of the temporary-directory scan,
  which was empty when read
  (the notes call it the fourth early read of that session);
- the report of a dry run of the old-worktree retire,
  which held partial output only;
- the output of a search for the gate launch command,
  which was empty;
- the output of the unit that started the rotation sweep of round w376
  (the notes call it the third early read of that stretch).

The counts in the notes do not agree with each other
(fourth,
fourth
and third),
so this entry counts by task.
A ninth read of the same stretch,
of `bwi7tdrq6`,
waited for its notice and is not counted.
The notes say that nothing was acted on after the retire report or after the search.
They give no cause beyond the habit:
the rule was in force,
and had been tightened once the same day in M305.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"How an agent waits".
The rule of M301 and M305 stands,
and gets one step where these eight showed a gap:
the rule said when to read and gave nothing to check at the moment of reading.
Written in this filing and not given by the notes:
before an output file of a background command is opened,
the turn names the task whose finish report it has seen,
and with no such report the file stays closed,
whatever kind of command it was
(a search,
a dry run or the start line of a unit).

Recurrence:
M305,
three such reads in one day,
M301,
and M162,
where a wait was replaced by counting the lines of a growing log.

## M314: five commands of the lead's had no purpose but to pass time

Status:
recorded on 2026-10-29 (UTC) from the lead's notes.

While it waited for background work,
the lead sent commands that waited for nothing:
- a "placeholder" command that waited on its own shell's process id,
  which the notes call a useless call;
- a second one that waited on process id 1 with a bound of 120 seconds;
- a third,
  after a compaction,
  `timeout 60 tail --pid=$$ ... 2>/dev/null || true`,
  which also hid its own stderr (stopped as `bmlwwym8d`);
- a fourth (`br0bvpdop`) that found its process id with `pgrep --exact git`,
  which the rule on waiting for a process forbids (stopped);
- a fifth that found its process id with `pgrep` as well (stopped at once).

The lead's notes give the rule:
never send a command whose only purpose is to pass time,
and when waiting on notifications,
end the turn or do queued work.
Two agents made the same kind of wait in the same rounds (M321).

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"How an agent waits":
the rule banned a loop of `pgrep` or `ps` calls and a Monitor,
and did not say that one wait on a process id found by `pgrep` is the same fault,
or that a wait on the shell's own process id never ends before its bound.
Both are written there now,
with the rule that a command whose only purpose is to pass time is not sent.

Recurrence:
M162,
where a brief named no way to wait and its agent polled,
and M301 and M305,
reads of output made in place of waiting for the notice.
