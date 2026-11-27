# Claude Code 2.1.332 restarted a session unasked, killed the background command it tracked, and left systemd units of their own running

## Status

Incomplete and unapproved.
Written 2026-11-27T14:15:26-05:00 by the coordinating Claude Code session of the Git wrapper work,
from one restart it lived through that afternoon.
The cause is not established,
no reproduction exists,
and the section "What is not known" lists what nobody checked.
Corrected 2026-11-27T14:19:55-05:00:
the first version said a subagent the agent list no longer names is briefed afresh,
and a message to such an agent's id then resumed it,
so rule `RZG` and this record now say that.
The owner declares this record complete,
not its writer.

The owner's instruction that this record backs,
given in that session after the restart:

```text
Add a rule to CLAUDE.md to say Claude Code may restart whenever it feels like it and the implications.
```

The rules are `RZN`,
`RZU`,
`RZW`,
`RZH` and `RZG` in `.config/file-enforcer/claude-preamble.md`,
which `file-enforcer` writes into `CLAUDE.md`.

## Symptom

A session that was waiting on a background command comes back with its conversation and its task list,
and with two messages from the harness.

The first names the background command:

```text
<status>stopped</status>
<summary>Background shell command didn't finish before the previous session ended</summary>
<note>No completion record was found for it in the previous session. It may have been stopped (via the UI, Monitor timeout, or agent teardown — these leave no transcript marker), or it may have been running when the previous Claude Code process exited. Check the output file for partial results before assuming it completed.</note>
```

The second replaces the context the harness attaches to a session:

```text
The session context was re-read when this session started; these values replace the earlier ones:
```

The owner's own words on it were:

```text
My limits have been reset but Claude Code restarted. Continue.
```

Nothing announces the restart beforehand,
and nothing says why it happened.

## What the restart of 2026-11-27 showed

Each line names the command that showed it.
All times are from the machine's clock.

### When

The harness wrote the last bytes of the stopped command's output file at 2026-11-27T14:02:39-05:00
(`stat` of that file),
and the shell the resumed session uses was set up at 2026-11-27T14:03:13-05:00
(`ls --time-style=full-iso` of the newest file under `~/.claude/shell-snapshots/`).
The session's first command after the restart printed 2026-11-27T14:03:13-05:00.

### What stopped

The session had one background command the harness tracked,
a `bash` loop started with `run_in_background` at 2026-11-27T13:57:34-05:00 with a deadline of ninety minutes.
After the restart its output file held ten bytes and no result:

```text
[killed]
```

`ps -eo pid=,ppid=,etimes=,args=` at 14:06 showed no process of that loop.
The harness reported it once,
as stopped,
and no completion for it ever came.

### What ran on

Eight writer rounds,
each in a `pi-<name>.service` unit of the account's service manager started with `systemd-run --user`,
were alive at 14:03:15,
their driver processes between 1574 and 4223 seconds old
(`ps -eo etimes=,args=` and `systemctl --user list-units 'pi-*.service'`).
None noticed the restart.

### What the resumed session was handed

- The conversation and the task list,
  whole.
- A git status older than the repository.
  The re-attached context named `e937565658` (2026-11-27T13:03:17-05:00) as the newest commit,
  while `git log` put `main` at `2f5cc0aeb7` (2026-11-27T14:00:13-05:00),
  which is 54 commits later
  (`git rev-list --count e937565658..2f5cc0aeb7`).
- A new folder for task output.
  Commands started before the restart wrote under
  `/tmp/claude-1000/-home-user-Monochromatic/076ac3af-0142-4237-abe8-c0d641b35841/tasks/`,
  and commands started after it write under
  `/tmp/claude-1000/-home-user-Monochromatic/b990a53d-bf22-4522-98cf-71a43a562d9c/tasks/`.
  The earlier folder and its files stayed readable.
- An agent list with no subagent of its own in it.
  `ListAgents` named one other session and nothing else.
  The subagent was still reachable:
  `SendMessage` to its agent id,
  which the conversation held from the result of starting it,
  answered `Resuming agent`,
  and the agent replied from its own transcript without running a command.

### A separate incident at the same time

The session's one native subagent,
a Claude Sonnet read of a release,
had ended on a usage limit;
its notice arrived together with the restart's.
Its ending is that limit's doing,
by its own notice,
and not shown to be the restart's.
A listener it had started was still running 1419 seconds after its start and was stopped by its process id.
A second agent was briefed from the twenty transcripts the first had saved as it went,
before the session knew the first could be resumed;
resuming the first would have kept its whole context.

## Root cause

Not established.
Claude Code is distributed as one executable,
so the only source read is the text inside it.

The executable under `~/.local/share/claude/versions/2.1.332` holds the notice's words and one more sentence
(`rg --text --only-matching` over that file):

```text
didn't finish before the previous session ended
Orphaned by a previous Claude Code process exit and reported in an aggregate summary.
```

So the harness itself expects background commands to be orphaned when a Claude Code process exits,
and reports them when the next session starts.
The same search finds what it says of background agents,
which this incident did not show:

```text
It had no completion record after the previous Claude Code process exited, and was automatically restarted from its saved transcript. It is running in the background again; its result will arrive as a separate task notification.
It was running when the previous Claude Code process exited and did not complete. Its in-process state was lost.
```

The program was replaced in place,
in the same process.
The process that starts the session's shells has process id 35147 and started at 2026-11-23T16:14:19-05:00
(the start time in `/proc/35147/stat`),
before the 2.1.332 executable it runs was written (2026-11-25T16:56:47-05:00),
and every one of its threads beside the main one started at or after 2026-11-27T14:02:39-05:00
(the start times in `/proc/35147/task/*/stat`),
the second the stopped command's output file was last written.

What made that happen on 2026-11-27 is not known.
The journal this account can read
(`journalctl --user` and `journalctl`,
from 14:02:00 to 14:03:30)
holds no line about an out-of-memory kill and none about Claude Code.

## Verification

Version under test:
`claude --version` printed `2.1.332 (Claude Code)`,
and `sha256sum` of the executable printed
`a2df1f41bb00250992926420d0063f89b127cb81abdd284dbefb80c50ff1d026`.

No harness makes a restart happen,
so nothing here was reproduced:
every line of "What the restart of 2026-11-27 showed" is one observation.
After the next restart the same commands check it again:

```bash
# doc/troubleshooting/claude-code-restarts-a-session-unasked-and-stops-its-tracked-background-commands.md
date --iso-8601=seconds
git log --format='%h %cI %s' --max-count=5 main
ps -eo pid=,ppid=,etimes=,args=
systemctl --user list-units --no-legend --plain
ls -t --time-style=full-iso -l ~/.claude/shell-snapshots/
```

### What survived

- The conversation and the task list.
- Every process in a systemd user unit of its own.
- Files on disk,
  the earlier task output folder among them.

### What did not

- The background command the harness tracked,
  killed with no result.
- The session's knowledge of its subagent,
  which `ListAgents` no longer named.
- The freshness of the attached git status.

## Verified workarounds

Each is a rule of the preamble,
and each was what kept or recovered work on 2026-11-27.

- Run whatever has to outlive the session in a named systemd user unit that writes its result to disk
  (rule `RZU`).
  The eight rounds did,
  and lost nothing.
  The cost:
  such a unit sits outside the shared limits the harness's own commands get,
  so its starter gives it limits,
  finds it again by name,
  and stops it.
- Rebuild state from the machine before acting,
  and start waits again
  (rule `RZW`).
  The cost is a few commands after every restart.
- Inspect what a stopped command touched before repeating it
  (rule `RZH`).
  Nothing was half-applied on 2026-11-27,
  because the one stopped command only waited;
  a merge or a release stopped the same way would have been.
- Message an earlier subagent by its agent id before briefing a fresh one
  (rule `RZG`).
  The message resumes it from its transcript.
  The cost:
  the id has to be at hand,
  and the agent list does not give it.
- Have a subagent save its findings as it makes them.
  This is no rule of the preamble.
  It is what let a second agent take over the read of 2026-11-27,
  and that agent reads and checks the first one's files again before it relies on them.

## What does not work

- Telling a restart from the `claude` process's id or start time.
  Neither changed:
  "Root cause" gives the figures.
  The start times of its threads do change.
- Looking the earlier subagent up in `ListAgents`.
  It is not named there,
  and it can still be resumed.
- Waiting for the stopped command to report.
  It is reported once,
  as stopped,
  and never completes.
- Reading the stopped command's output for its result.
  The file holds the kill marker and nothing else.
- Trusting the git status the harness attaches after the restart.

## What is not known

- What ends and restarts a session.
- Whether a subagent that was running,
  and not already ended,
  is restarted by the harness itself,
  as the executable's own sentence says.
  The session's one subagent had already ended.
- Whether a command in its own `systemd-run --user --scope`,
  started from a harness command,
  survives.
  Only `.service` units were running.
- Whether scheduled wakeups and crons carry across.
  None was scheduled.
  The executable passes `carriedCrons: true` on the path that restores a session,
  which suggests they do and was not shown by a run.
- What happens to a foreground command that is running at the restart.
  None was.
- Whether other versions behave the same.

## Upstream filing decision

Nothing is filed.
`.out-of-scope/claude-code-upstream-bugs.md` puts Claude Code defects out of scope for upstream tracking:

```text
This project does not file or track Claude Code bugs as GitHub issues, and does not
invest in elaborate local workarounds for upstream Claude Code defects.
```

That file asks for this record and for rules in place of an issue,
which is what was done.
No search of the upstream tracker is recorded here:
three searches with `gh search issues` returned nothing,
and so did the control query that had to match,
so they show nothing either way.
