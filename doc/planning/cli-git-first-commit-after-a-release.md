# A first commit after a release is installed

What the change of release cost a commit,
what it costs after the program link,
and what is left on the commit's wait.
Measured 2026-11-11T20:01-05:00 on this host and 2026-11-12T01:00-05:00 in a container.

## The line that started this

`0.6.1-alpha.11` was installed over `0.6.1-alpha.10` at about 20:01 and the first commit with it,
one file in the main checkout,
took 28,616 ms (`totalMs`).
Its timing line (`monochromatic-git-policy-cli-commit-timing.jsonl`,
the common Git directory,
`2026-11-12T01:02:11.470Z`) splits it:

- `routeMs` 2,
  `prepareMs` 3,580,
  `buildMs` 266;
- `provisionMs` **17,621**;
- `submitMs` 6,937,
  of which `reachMs` 1,746 and `answerWaitMs` 5,190;
- `gateMs` 204,
  `handoffMs` 2.

## The 17.6 s, by step

The service manager's own journal for the window names the client by its process number,
the same 1832701 the timing line carries:

```text
2026-11-11T20:01:46-05:00 bazzite systemd[5697]: Reload requested from client PID 1832701 ('monochromatic-g') (unit gpc-install-0-6-1-alpha-11.service)...
2026-11-11T20:01:46-05:00 bazzite systemd[5697]: Reloading...
2026-11-11T20:02:00-05:00 bazzite systemd[5697]: Reloading finished in 13324 ms.
2026-11-11T20:02:01-05:00 bazzite systemd[5697]: Reload requested from client PID 1842676 ...
2026-11-11T20:02:04-05:00 bazzite systemd[5697]: Reloading finished in 2793 ms.
```

So the 17,621 ms are:

- **13,324 ms** in one call,
  the manager reading every unit it holds because the service template changed
  (its `ExecStart` named the executable of the release that wrote it,
  at its full versioned path);
- **about 4,200 ms** in the three calls after it
  (two `reset-failed` state clearings and the start of the worktree's socket unit),
  which the manager answered only after another client's reload had run,
  the 2,793 ms one asked a second after the first finished;
- the remaining tens of milliseconds:
  the two small reads of the installed templates,
  the note of the worktree,
  the `Listen` question,
  and the write of the record of what the manager was told.

Nothing in the 17.6 s was the earlier release's service being stopped:
the retirement of the single service of the oldest releases found no file of it
(`$XDG_RUNTIME_DIR/cli-git.socket` absent,
`cli-git.socket` and `cli-git.service` absent),
and `stop_earlier_instance_socket` found no earlier per-worktree socket either.
The earlier release's *service* was stopped on the commit's own wait,
inside `submitMs`:
the journal shows four `retire requested by a client` lines
from release `0.6.1-alpha.10` between 20:02:04.365 and 20:02:05.768,
its `exiting reason=retire` at 20:02:04.366,
`Started monochromatic-git-policy-cli@f30a....service` at 20:02:05,
and `started release="0.6.1-alpha.11"` at 20:02:06.006.
That is the 1,746 ms of `reachMs`.

## What the next ten minutes showed

The coordinator's seventeen commits from 01:02 to 01:12Z (`gitDir`,
`provisionMs`):

```text
.git                          17,621 then 0 then 7,822 then 3,521
pi-kimi-...round-210-findings    387 then 20,002 (stopped) then 0 then 8,333
pi-worktree-removal-...         20,002 (stopped)
```

The hourly `provisionMs` of the 52 commits of `0.6.1-alpha.10` before the install was
median 0,
ninetieth percentile 21,
largest 2,771.

The cause of the repeated cost is the record of the unit texts the manager was told about
(`src/native/service/service_provision_loaded_unit_texts.rs`),
which is one file per runtime directory and therefore shared by every worktree.
`reach_current` dropped it the moment a service of another release answered,
so the first commit of *every* worktree that met its own still-running older service
made the next commit in *any* worktree write the units again and ask for a reload:

- worktree A's first commit met its `0.6.1-alpha.10` service → dropped the record;
- the main checkout's next commit found no record → reload (7,822 ms);
- worktree B's first commit → dropped it again;
- the main checkout's next commit → reload (3,521 ms).

A reload slower than `EXCHANGE_WAIT` (20 s) then failed the commit outright with
"This account has no user service manager",
which was false:
two commits at 20:11:32 ended with `provisionMs` 20,000 and 20,002 and exit 1.

## The two-build reproduction

Four builds of this branch and of `main` (the released `0.6.1-alpha.11` code),
each pair differing only in `version` in `Cargo.toml`,
were built with `cargo build --release`,
and run inside a `podman` container (2 CPUs,
2 GB,
1024 processes) with a disposable `HOME`
and a disposable repository per arm:
the first commit with the first build,
the second commit through a `git` link to the second build,
so the second commit is a change of release.
The container's service manager is its *system* manager standing in for a login session's
(`XDG_RUNTIME_DIR=/run`,
the wrapper's runtime unit directory linked into the manager's own,
`DBUS_SESSION_BUS_ADDRESS` at the system bus):
`systemd --user` cannot be started there without a delegated cgroup subtree,
which this host's rootless `podman` does not give it.
That is a limit:
the manager is the same D-Bus interface and the same unit semantics,
and its reload is much faster than the host's.

Every figure below is a landed commit unless it says `stopped`.

```text
arm   code                    commit            totalMs  provisionMs  reachMs
old1  0.6.1-alpha.11 (main)   first                496          17        166
old1  0.6.1-alpha.11 (main)   change of release  15595         254      10032  stopped
old2  0.6.1-alpha.11 (main)   first                378         200         26
old2  0.6.1-alpha.11 (main)   change of release  11371         184      10020  stopped
old3  0.6.1-alpha.11 (main)   first              12863        4195        298
old3  0.6.1-alpha.11 (main)   change of release  20033         215      10052  stopped
old4  0.6.1-alpha.11 (main)   first                293         189         18
old4  0.6.1-alpha.11 (main)   change of release   4910         193       1821
new1  pointer, transition     first              11383        5822       5043
new1  pointer, transition     change of release   8448           1       7355
new2  pointer, steady         first                255           9         36
new2  pointer, steady         change of release  13061           0      10036  stopped
fix1  pointer + retire bound  first               1097         566         82
fix1  pointer + retire bound  change of release   7185          29       7035
fix2  pointer + retire bound  first                114           4         17
fix2  pointer + retire bound  change of release   6996           0       6934
```

Read:

- **the provisioning part of a change of release is gone**:
  184 to 254 ms before,
  0 to 29 ms after,
  and none of those milliseconds is a manager call
  (the two arms' `updateRounds` and `connectTries` are the reach's,
  not the provisioning's);
- the first commit of a release whose *template* changes (this release,
  the `new1` transition)
  still pays one reload:
  5,822 ms of the container's stand-in manager,
  13,324 ms of the host's;
  a release after that pays nothing (`new2`,
  `fix2`:
  `provisionMs` 9 and 4 ms on a first commit);
- **the reach part is untouched**:
  `reachMs` 6,934 to 7,355 ms on the container's stand-in,
  `startWaitMs` up to 8,580 ms on the other agent's fixture,
  and `answerWaitMs` is the new service's own first work;
- **the commit that did not land**.
  Before the bound on the retiring service,
  five of six changes of release ended `stopped`
  with `reachMs` 10,032 to 10,052 ms and `releasesSeen` naming only the old release:
  the client met the old service on all hundred tries,
  the old service never left,
  and the client gave up at its bound with
  "The commit service that is running was started from another release ...".
  Ending it once at the bound and waiting on turned those into landings
  (`fix1`,
  `fix2`,
  and `old4`,
  whose old service happened to leave in time).

## What the change does, and what is still on the wait

Moved off the commit's wait:

- the manager's reload of every unit,
  for a change of release and for a second build of one:
  the unit text names a program link the wrapper replaces by rename
  (`src/native/service/service_provision_program_pointer.rs`);
- every manager round trip when nothing about the units changed:
  a provisioning that finds the texts,
  the record,
  the link and the socket right asks nobody;
- a manager that does not answer within the exchange's time no longer fails the commit:
  the units stay written,
  the record stays unwritten so the next commit asks again,
  and one line says what the wait was for.

Still on the wait,
and why:

- **the service of the earlier release leaving.**
  The commit cannot be handed to a service of this release before the old one has gone,
  because the manager starts one service per socket unit,
  and the two must never act on one repository's commits at once.
  The bound on that wait is `start_wait`,
  after which the unit is ended once,
  said once,
  and waited for under `START_CEILING`.
- **the new service's first answer**,
  which is the service's own work:
  the warm,
  the maintenance and the landing.
- **one reload,
  once per template change**,
  which only a release that changes the unit text itself
  can need.
  This release is such a release,
  because the text it writes differs from the one
  `0.6.1-alpha.11` wrote;
  every release after it writes the same text.

## Open questions

- Should the one transition reload be paid at install time rather than by the first commit?
  `mise run //package/git-policy/cli:native:install` already runs the installed wrapper once
  (`git --version`),
  so a provisioning could be asked for there.
  It trades a commit's wait for a command's wait,
  and it is a design choice,
  not a defect.
- The container's manager is the system manager standing in for a login session's.
  A faithful `systemd --user` measurement needs a container whose cgroup subtree is delegated,
  or a throwaway VM.
