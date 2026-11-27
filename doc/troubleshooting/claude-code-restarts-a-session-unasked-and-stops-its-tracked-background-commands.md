# Claude Code 2.1.332 restarted a session unasked, killed the background command it tracked, and left systemd units of their own running

## Status

Incomplete and unapproved.
Written 2026-11-27T14:15:26-05:00 by the coordinating Claude Code session of the Git wrapper work,
from one restart it lived through that afternoon.
The cause is not established,
no reproduction exists,
and the section "What is not known" lists what nobody checked.
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
so that work did not have to be done again.

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
What made the session end and start again on 2026-11-27 is not known.

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
- Brief a fresh subagent from the files the earlier one kept,
  and have every subagent save findings as it makes them
  (rule `RZG`).
  The cost:
  the second agent reads and checks the first one's files again before it relies on them.

## What does not work

- Telling a restart from the `claude` process.
  The process that starts the resumed session's shells has process id 35147,
  and `ps -o lstart=` gave its start as Mon Nov 23 21:14:19 2026,
  while `readlink /proc/35147/exe` named the 2.1.332 executable,
  a file last changed 2026-11-25T16:56:47-05:00.
  A start time older than the executable the process runs says that neither its process id nor its start time changes
  when the program is replaced.
- Waiting for the stopped command to report.
  It is reported once,
  as stopped,
  and never completes.
- Reading the stopped command's output for its result.
  The file holds the kill marker and nothing else.
- Trusting the git status the harness attaches after the restart.

## What is not known

- What ends and restarts a session,
  and whether the process exits or is replaced in place.
- Whether a message to a subagent that `ListAgents` no longer names would have reached it.
  It was not tried:
  a second agent already worked in the same folder.
- Whether a command in its own `systemd-run --user --scope`,
  started from a harness command,
  survives.
  Only `.service` units were running.
- Whether scheduled wakeups and crons carry across.
  None was scheduled;
  the executable holds the name `carriedCrons`,
  which nobody traced.
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
