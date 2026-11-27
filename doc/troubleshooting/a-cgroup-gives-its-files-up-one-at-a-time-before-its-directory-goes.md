# A cgroup gives its files up one at a time before its directory goes

A reader that reads a control group's files while the group is being removed meets a group that is partly there:
`package/git-policy/cli`'s check scope accounting read a scope's `pids.peak` and then answered no
`pids.events` (issue 6610,
2026-11-27),
so `native:ts-checks` printed no line about the task bound the scope had refused forks at,
which is what issue 6605 added those counters for.
The next section is the kernel's order,
which is what the reading met;
the last sections are what was measured here and what the reader does about it.

## What the kernel does

`cgroup_destroy_locked` clears a dying group's files first and removes its directory after them
(`kernel/cgroup/cgroup.c:6398-6400`:
`css_clear_dir(&cgrp->self)`,
then `kernfs_remove(cgrp->kn)`).
`css_clear_dir` removes one file at a time,
each through `cgroup_rm_file`'s `kernfs_remove_by_name`,
so between the first and the last file the directory is still there and holds some of its files.
A reader therefore meets a group losing its files in the controller's own order,
never a group that vanishes whole:
the pids controller's order is `max`,
`current`,
`peak`,
`events`,
`events.local`
(`kernel/cgroup/pids.c:397-419`),
so `pids.peak` can still answer when `pids.events` is already gone.

A file the kernel has deactivated and not yet unlinked answers `ENODEV`,
not `ENOENT`:
`kernfs_fop_open` refuses the open at `fs/kernfs/file.c:626`,
and a read of an open file that was deactivated the same way answers it at `fs/kernfs/file.c:469`.
Both windows are open while `kernfs_drain` waits for the node's last user
(`fs/kernfs/dir.c:539`) and until it is unlinked,
after which the same lookup answers `ENOENT`.

An open descriptor does not hold the removal:
`kernfs_fop_open` takes an active reference only for the length of the open and puts it once the open succeeds
(`fs/kernfs/file.c:737`).
So a process cannot stop a group's removal by keeping one of its files open,
and a probe that tried to do so saw the removal complete and every file answer `ENOENT` afterwards.

## What was measured

All three readings are from this host on 2026-11-27,
in capped scopes of their own.

- One reader read a real check scope's twelve files,
  one after the other,
  in a 337-round loop while the scope died.
  In one of the 144 rounds that found the folder there,
  `pids.current`,
  `pids.peak`,
  `pids.max`,
  `pids.events` and `pids.events.local` all answered,
  and `memory.current`,
  `memory.peak`,
  `memory.max`,
  `memory.events`,
  `memory.events.local`,
  `cpu.max` and `cpu.stat`
  all answered `ENOENT`.
  That is the shape a reading meets:
  a group that is partly gone,
  with the folder still there.
- Four hundred and fifty-eight real control groups were made and removed under this account's delegated place while
  three readers read their ten files,
  76,860 file reads in all:
  69,742 answered a figure,
  6,649 answered `ENOENT`,
  and 469 answered `ENODEV`.
  `ENODEV` is not a rare answer for a reader that meets a removal,
  which is why a reader that rethrows it dies where a check's scope ends.
- The error Node builds for that answer,
  captured from one of those reads:

  ```json
  {
    "code": "ENODEV",
    "errno": -19,
    "syscall": "read",
    "message": "ENODEV: no such device, read '/sys/fs/cgroup/.../pids.max'"
  }
  ```

## The rule

A check scope's figures are read while the check runs,
and a reading that answers less than the whole reading before it is a reading of the moment the group went,
never a figure this host's kernel does not write:
`answersEveryPartOf` in `package/git-policy/cli/bin/native-check-scope-accounting.ts` refuses such a reading,
and both callers pass the reading it would replace
(`bin/native-check-scope-watch-accounting.ts` for the watch,
`bin/native-capped-scope-command.ts` for its look after the check has ended).
A file the kernel is removing answers no figure at all (`isRemoved` in the same file),
so the removal ends a reading instead of throwing out of it.

What `native:ts-checks` prints for a scope whose group loses `pids.events` and `cpu.stat` before the runner looks again,
before and after that rule:
before,
its line about the scope's refused forks was absent and the scope's `cpu.stat` was empty;
after,
the reading taken while the whole group was there is what the line is made of.

## What this leaves

- `bin/native-gate-container-cgroup-memory.ts` reads a gate container's cgroup the same way and has neither the rule
  nor the removal's answer:
  it keeps a partial sample,
  and a removal throws out of the read.
  Its watchers (`bin/native-gate-container-memory.ts`,
  `bin/native-gate-container-peaks.ts`) keep that sample as the
  last figures,
  so a container killed for memory whose `memory.events` went with its scope is reported as one that met no kill.
  A container's cgroup goes the same way a check scope's does,
  so the same rule belongs there;
  issue 6610 changed the check scope's reader only.
- A check whose first reading is taken while its group goes still under-reports it:
  a reading with nothing before it replaces nothing,
  and no better reading of that group exists.
