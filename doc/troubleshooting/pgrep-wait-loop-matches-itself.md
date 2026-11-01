# A `pgrep --full` wait loop matches its own shell and never exits

## Symptom

A background wait loop meant to block until some long job finishes never finishes itself.
The job it was waiting for exits normally,
and the loop keeps spinning,
so whatever was supposed to run afterwards never runs at all.

Observed 2026-08-17 while waiting for a unit suite before rebuilding:

```bash
# Never exits, even after every matching test process is gone.
until ! pgrep --full 'translation-repair:test:unit' >/dev/null 2>&1; do sleep 5; done
mise run //package/translation-repair/<package>:build
```

The failure is silent in the worst way.
Nothing errors,
nothing is logged,
and the shell sits at 0% CPU looking exactly like a patient wait.
Two of these accumulated for 17 minutes and 7 minutes before anyone noticed,
and the rebuild they gated never happened,
so `dist/` still held a deliberately broken build while the source on disk had already been restored.

## Root cause

`pgrep --full` matches against the FULL COMMAND LINE of every process,
and the shell running the loop has the pattern in its own command line,
because the pattern is literally part of the command it was invoked with.

So the loop's own `/bin/bash -c ... until ! pgrep --full 'translation-repair:test:unit' ...` process
is itself a match.
`pgrep` finds it,
the loop concludes the job is still running,
and it waits for itself forever.

This is not specific to `pgrep`.
Any self-referential process search has it:
`ps | grep <pattern>` is the classic form,
which is why the folklore workaround `grep '[t]est'` exists.
`pgrep` without `--full` matches only the process NAME,
which is why the trap appears exactly when the pattern has to be specific enough to be useful.

## Verification

```bash
# In one shell, with no such job running anywhere:
until ! pgrep --full 'zzz-nothing-runs-this' >/dev/null 2>&1; do sleep 1; done; echo done
# Prints nothing. It is matching itself.

# Confirm by looking:
ps -eo pid,args | grep 'zzz-nothing-runs-this'
# Shows the waiting bash, whose command line contains the pattern.
```

## What to do instead

-   PREFER NOT WAITING ON A PATTERN AT ALL.
    If the job was started from this session,
    wait on the job rather than on a process name:
    start it with the command tool's own backgrounding and let the completion notification arrive.
    That is what the tool exists for,
    and it cannot match itself.
-   WAIT ON A PID,
    not a pattern,
    when a pattern is unavoidable:
    `while kill -0 "${pid}" 2>/dev/null; do sleep 5; done`.
    A pid is unambiguous and no shell can accidentally be it.
-   WAIT ON THE ARTIFACT the job produces,
    which is often the real condition anyway:
    an output file appearing,
    a lock file being released,
    an exit-code file being written.
-   EXCLUDE SELF only as a last resort,
    with `pgrep --full --older 1` or by filtering `$$`,
    since both are easy to get subtly wrong and neither is obvious to the next reader.

## Why it is worth a document

The cost is not the stuck loop,
which is free to kill.
The cost is that the loop makes a REBUILD SILENTLY NOT HAPPEN,
so a later step reads a stale or deliberately broken build and reports a result about the wrong code.
That failure looks like a real measurement,
which is the expensive kind.

## It happened again six hours later, to the agent who wrote this

2026-08-17,
same session.
A loop chaining a second corpus audit behind a first,
built to keep both runs on one build:

```bash
while pgrep --full 'node dist/final/node/rendering-audit-settled.mjs' > /dev/null; do sleep 15; done
```

Identical trap,
identical silence,
and the chained run never started.
The remedy applied was the one this document already prescribes,
which is the point:
the document was right,
and it was not read.

So the lesson is not about `pgrep`,
which is covered above.
It is that BUILDING PROCESS AUTOMATION IS A CUE TO READ THIS INDEX FIRST.
The traps that cost the most here are the silent ones,
and a silent trap cannot remind you it exists.
Two of the four incidents this file now records
were written down before they were repeated.

A second trap was hit the same afternoon
that presents identically from the outside,
a run that had genuinely finished but would not exit:
see [A finished process that will not exit](finished-process-does-not-exit.md).
Distinguishing them takes one look at the log's completion line.
