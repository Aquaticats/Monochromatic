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
and a read of an open file that was deactivated the same way answers it at `fs/kernfs/file.c:167`,
where `kernfs_seq_start` returns `ERR_PTR(-ENODEV)` for the file a cgroup's read goes through.
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

A reader of a control group's figures reads them while the work runs,
and a reading that answers less than the whole reading before it is a reading of the moment the group went,
never a figure this host's kernel does not write:
`answersEveryPartOf` in `package/git-policy/cli/bin/cgroup-whole-readings.ts` refuses such a reading,
and every caller passes the reading it would replace.
A file the kernel is removing answers no figure at all (`isRemoved` in the same file),
so the removal ends a reading instead of throwing out of it.

Two readers share that rule and neither knows the other's files:
`bin/native-check-scope-accounting.ts` reads a check scope's own accounting,
and `bin/native-gate-container-cgroup-memory.ts` reads a gate container's.
The check scope's callers are `bin/native-check-scope-watch-accounting.ts` for the watch and
`bin/native-capped-scope-command.ts` for its look after the check has ended;
the container's are `bin/native-gate-container-memory.ts` for the watch the gate reports a kill from and
`bin/native-gate-container-peaks.ts` for the recorder that writes one line per sample.

What `native:ts-checks` prints for a scope whose group loses `pids.events` and `cpu.stat` before the runner looks again,
before and after that rule:
before,
its line about the scope's refused forks was absent and the scope's `cpu.stat` was empty;
after,
the reading taken while the whole group was there is what the line is made of.

## Every order a reading of a gate container's group can take

A gate container's reader asks whether the scope's directory is there (`stat`),
then reads `memory.current`,
`memory.peak`,
`memory.max`,
`memory.swap.current`,
`memory.swap.max`,
`pids.current`,
`pids.peak`,
`memory.stat` and
`memory.events`,
one after the other.
The kernel removes a dying group's files one at a time in each controller's own order
(the pids controller's is `max`,
`current`,
`peak`,
`events`,
`events.local`;
the memory controller's order is not recorded here),
and the directory only after every file.
So a reading meets one of these,
and each answers as follows
(before is the reader at `07f9232af0`,
after is issue 6614's round):

1. Every file answers.
   Both answer the whole group,
   the watch keeps it,
   and the gate names a kill its `memory.events` counted.
2. The directory is gone before the first read.
   Both answer that the cgroup is not there,
   the watch keeps the whole reading before it,
   and the gate prints what that reading says.
3. The directory is there and every file is gone,
   which is the moment between `css_clear_dir` and `kernfs_remove`.
   Before,
   the sample held no figure and an empty `stat` and `events`,
   the watch kept it,
   and the gate printed nothing for a container that met no kill.
   After,
   the reader answers `READING_NOT_WHOLE`,
   so the whole reading before it is what the gate reports,
   and a recorder writes no line for it and asks again.
4. Some files answer and `memory.events` is one of those that did not,
   whichever point of the read the file went at.
   Before,
   the sample's `events` was empty,
   the watch kept it,
   and a container killed inside its bound was reported as one that met no kill.
   After,
   the reading is refused and the whole reading before it is what the gate reports,
   with the kill in it.
   The recorder writes no line for it either,
   and a refused reading ends no role:
   a group whose files are going is still a directory that is there,
   and the role ends when a reading finds the directory gone.
5. A file the kernel has deactivated and not yet unlinked answers `ENODEV`.
   Before,
   the reader rethrew it,
   the watch's reading loop rejected,
   and the gate's container run ended on the reader instead of on its check.
   After,
   that file answers no figure,
   the reading is refused for the part it lost,
   and nothing throws.
6. A reading answers a part the one before it did not,
   which is a group still being made.
   Both keep it:
   a control group gains its files as it is made and never regains one it lost.
7. The group goes between two readings.
   Both answer that the cgroup is not there,
   and the last whole reading is what a caller is judged on.

A recorder writes one line per sample (`bin/native-gate-container-peaks.ts`):
before,
every order that answered a part of the group wrote a sample line,
and the end line repeated the last one;
after,
a reading that is no whole reading writes no line,
the recorder keeps asking,
and the end line holds the last whole reading.

Two sentinels keep those apart,
`CGROUP_NOT_THERE` for a directory the reader cannot stat and
`READING_NOT_WHOLE` for a group that answered no file of its own or lost a part of the reading before it
(`bin/native-gate-container-cgroup-memory.ts`):
the first ends a recorder's role,
the second only says the reading is not one to keep.

## What the gate prints for such a container, before and after

The fixture is a control group with a whole reading that counted one kill,
held at `pids.peak` by a named pipe while the group loses `memory.events`
(`holdStandInCgroupReading` in `package/git-policy/cli/bin/stand-in-cgroup-files-for-tests.ts`),
so the loss happens at a chosen point of a reading and no pause decides it.
The control that holds the fixture is the one in `package/git-policy/cli/bin/native-gate-container-memory.unit.test.ts`
whose name says it keeps the whole reading a killed container gave when its group loses `memory.events` part way through
the next,
and it asks the printer the gate's own container run prints with,
`containerMemoryKilledLine` in `bin/native-gate-container-memory.ts`.

- Before,
  the watch kept `{}` as the container's events,
  so the printer answered that no kill is to be named and the gate printed nothing.
- After,
  the watch keeps the whole reading,
  `{"low":0,"high":0,"max":1,"oom":1,"oom_kill":1,"oom_group_kill":0}`,
  and the gate prints:

  ```text
  The suite's container was killed for memory: its cgroup counted 1 out-of-memory kill at the 3.0 GiB memory.max bound it
  was given, with a peak of 1,516.0 MiB. That is the bound and not a failing test: raise it with --memory after the task
  name, or with --container-memory when native:gate starts the run.
  ```

## What still limits it

- A first reading taken while a group goes still under-reports it:
  a reading with nothing before it replaces nothing,
  and no better reading of that group exists.
  What changed is that a directory answering no file of its own is no whole reading,
  so a recorder starts its end line from a whole reading or writes none.
- A kill that lands after the last whole reading and before the group starts going is still lost:
  the 100 ms interval between two readings decides whether a reading is still there to take,
  and nothing can read a cgroup that is already gone.
