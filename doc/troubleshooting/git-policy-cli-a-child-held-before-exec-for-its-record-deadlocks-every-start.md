# Git policy cli a child held before exec for its record deadlocks every start

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 8070 to 8087 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The lock-holders folder**,
  its author's fourth run,
  ended 08:09:23 at `3ac1254a73`:
  the four findings answered,
  each control failing with its fix out
  (the fourth hangs until an outer bound of 90 s without its fix and answers in 48 ms with it).
  A Git child whose record cannot be written is stopped and reaped before it can take a lock;
  a child that already died is still named,
  from `/proc/<pid>/stat`,
  which keeps its start tick until its parent reaps it;
  this holds at every place the wrapper starts a Git that may take a lock.
  The walk goes over exact lock names;
  every record is opened as a plain file;
  above 4,096 records the proof answers "unknown".
  The author first built the other shape
  (the child held in `pre_exec` until the record is written)
  and found that it deadlocks every start,
  because `Command::spawn` waits for the child to reach `execve`.
