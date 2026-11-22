# libvirt 12.9 session daemon runs in the capped scope of whichever command starts it, so one command's memory cap kills every session's guests

Status:
diagnosis by the coordinator session of 2026-11-21,
unapproved and non-final.
On 2026-11-21 the project chose that `mvm` starts the daemon itself in a user unit of its own
("A:
mvm starts it",
issue 6338),
and that first workaround is built.
One real guest has since run inside the unit (see "One real guest in the unit,
2026-11-21"),
and a gate review the same day found the branch not fit for three reasons this workaround now answers:
guests dying whenever the daemon exits,
a caller that finds the unit name still loaded and waits thirty seconds for nothing,
and a cap that rested on an assumed guest count.
See "Workarounds" for what is built and what is still open.

## Symptom

Plasma shows a persistent notification titled "Memory Shortage Avoided",
with the text
"The background service `<unit>` has been terminated by the Linux kernel because the system is low on memory.
Consider closing unused applications or browser tabs."
On 2026-11-21 the units were transient scopes named like `run-p444198-i75857716.scope`.

At the same moments a guest started through `mvm`'s `libvirt` backend dies,
and so does every other guest running at the time,
whichever session started it.
The user journal shows the scope's end:

```text
# journalctl --user, 2026-11-21
21:23:58 run-p444198-i75857716.scope: The kernel OOM killer killed some processes in this unit.
21:24:08 run-p444198-i75857716.scope: Stopping timed out. Killing.
21:24:08 run-p444198-i75857716.scope: Killing process 444288 (virtqemud) with signal SIGKILL.
21:24:08 run-p444198-i75857716.scope: Killing process 447756 (qemu-system-x86) with signal SIGKILL.
21:24:08 run-p444198-i75857716.scope: Killing process 1006385 (qemu-system-x86) with signal SIGKILL.
```

The machine as a whole was not short of memory.
The kernel's own line names a cgroup limit:

```text
# journalctl --dmesg, 2026-11-21
21:23:58 kernel: oom-kill:constraint=CONSTRAINT_MEMCG,...,oom_memcg=/user.slice/user-1000.slice/user@1000.service/app.slice/run-p444198-i75857716.scope,...
21:23:58 kernel: memory: usage 2101496kB, limit 2097152kB, failcnt 37179
21:23:58 kernel: shmem 2064969728
21:23:58 kernel: Memory cgroup out of memory: Killed process 447756 (qemu-system-x86) ...
```

A second scope ended the same way at 21:47:52
(`run-p1196543-i76658584.scope`,
"2G memory peak,
2.9G memory swap peak",
its `virtqemud` and one guest killed).

## Root cause

### The daemon is started by whichever client runs first

`mvm` does not start libvirt's session daemon;
`package/mvm/cli/README.md`,
section "Session daemon":

```text
mvm does not start libvirt's session daemon.
`virsh` starts `virtqemud` on demand,
and the daemon keeps running while a domain runs and exits after two idle minutes.
```

On 2026-11-21,
after this diagnosis,
that section changed:
`mvm` now starts the daemon itself,
as described under "Workarounds".

A process started by another inherits its cgroup.
So when no daemon is running,
the first `virsh` call puts `virtqemud`,
and with it `virtlogd`,
into the scope that call runs in.
Agent commands here run in capped scopes
(`AGENTS.md` rules `RXI` and `RXS`:
"`systemd-run --user --scope` with `CPUQuota`,
`MemoryMax`,
`TasksMax`,
`IOWeight=1`").

### Every later guest joins that scope

Each guest's `qemu-system-x86` and `virtiofsd` are started by the daemon,
so they join the daemon's scope,
whichever session asked for the guest.
The kernel's task table for the first kill lists the daemon and two guests of different start times in one scope
(PIDs 447756 and 1006385;
the full table is in the kernel journal at 21:23:58).

### Guest memory is charged to that scope

`mvm` backs guest memory with a shared `memfd`,
because `virtiofsd` maps it;
`package/mvm/cli/src/domain-xml-builders.ts:321` to `:336`:

```ts
// package/mvm/cli/src/domain-xml-builders.ts
// guest memory in a memfd that `virtiofsd` can map.
// sharedMemoryBacking(); // => '<memoryBacking><source type="memfd" /><access mode="shared" /></memoryBacking>'
```

That memory is counted as `shmem` in the scope:
2,064,969,728 of the 2,101,496 kB in use at the first kill.
A scope sized for one command's needs (2 GiB in both kills) cannot hold a guest.

### systemd then stops the whole scope

The user manager's default is `DefaultOOMPolicy=stop`
(`systemctl --user show --property=DefaultOOMPolicy`,
read 2026-11-21),
and `systemd.service(5)` (systemd 262),
under `OOMPolicy=`,
says:
"If set to stop the event is logged and the unit's processes are terminated cleanly by the service manager."
So after the kernel kills one guest,
systemd stops the scope and kills the daemon and every other guest in it.
The same happens when the scope's `RuntimeMaxSec=` runs out,
or when its starter stops it by hand,
which `AGENTS.md` rule `NLD` asks every agent to do before it ends.

### Why the notification blames system memory

Plasma's `oom_notifier` (plasma-workspace 6.8.3) shows the notification for any user scope or service whose `Result` becomes `oom-kill`;
[`oom_notifier/module.cpp` lines 54 to 78 at v6.8.3](https://github.com/KDE/plasma-workspace/blob/4302f699329ebb155097beaab2d5aabbb21e33b3/oom_notifier/module.cpp#L54-L78):

```cpp
// oom_notifier/module.cpp at v6.8.3 (4302f699329ebb155097beaab2d5aabbb21e33b3)
if (interface != "org.freedesktop.systemd1.Scope"_L1 && interface != "org.freedesktop.systemd1.Service"_L1) {
    return;
}
if (changed.value(u"Result"_s, QString()) != "oom-kill"_L1) {
    return;
}
...
const QString title = i18nc("@title", "Memory Shortage Avoided");
...
"the system is low on memory. Consider closing unused applications or browser tabs.",
```

It does not look at whether the kill was a cgroup's own limit (`CONSTRAINT_MEMCG`) or the machine running out,
so its wording is wrong for these kills.
It ignores slices,
so the `app.slice: The kernel OOM killer killed some processes in this unit.` lines that follow each kill raise no notification.

The same notification has another recorded cause,
an agent's own 2-GiB slice:
`doc/troubleshooting/pi-worker-memory-budget.md`.

## Verification

### Versions

- plasma-workspace 6.8.3-1.1,
  libvirt 1:12.9.0-1.1,
  systemd 262.2-1 (`pacman -Q`,
  2026-11-21);
- the running daemon is `/usr/bin/virtqemud`.

### Evidence read

- The kernel journal's two dumps at 21:23:58 and 21:47:52,
  both `CONSTRAINT_MEMCG`,
  both killing `qemu-system-x86`.
- The user journal's scope endings quoted in "Symptom".
- `ps -o pid=,lstart=,cgroup= -C virtqemud,qemu-system-x86` at 21:58:
  the daemon (started 21:49:24) and a guest (21:49:42) both in `app.slice/integ-store-guest-arms.scope`,
  an agent's capped command scope
  (`systemctl --user show`:
  `MemoryMax=25769803776`,
  `RuntimeMaxUSec=1h`,
  `OOMPolicy=stop`,
  started 21:35:55).

### Run on 2026-11-21 after the build

- `mvm list` against the session daemon another session had running:
  it asked with `LIBVIRT_AUTOSTART=0`,
  found that daemon answering,
  used it as it is,
  and listed both VMs in 191 ms,
  the question itself taking 37 ms.
- A `virtqemud --timeout=120` started in a user unit of its own named for this measurement,
  with a private `XDG_RUNTIME_DIR` so it could not reach the running daemon's socket:
  it answered 203 ms after `systemd-run` returned,
  its unit held 11.6 MiB (peak 14 MiB),
  the unit's journal held its log lines,
  and stopping the unit left no process.

### One real guest in the unit, 2026-11-21

An integration pass on 2026-11-21 ran `mvm create` through the built CLI with the unit in place,
from a command scope capped at 24 GiB.
`virtqemud`,
`virtlogd`,
`virtiofsd` and
`qemu-system-x86` all sat in `app.slice/mvm-libvirt-session.service`,
not in that command's scope,
and they stayed there after the scope ended:
that scope's cap can no longer kill them.
Unit properties with the guest inside:
`Type=simple`,
`KillMode=control-group`,
`CollectMode=inactive-or-failed`,
`OOMPolicy=continue`,
`MemoryMax=38654705664` (the 36 GiB cap this doc had then),
and `MemoryPeak=993636352`.
The guest answered its agent and was destroyed through `mvm destroy`.
Within two minutes a second session's 8 GiB guest joined the same unit and reached a peak of 2725593088 B,
so one unit with every session's guests was measured,
not assumed.
The unit then ended by itself two idle minutes after the last guest was removed,
but another session's guest had been in it until 2026-11-21T23:01:37-05:00,
so an idle exit with nobody else's guest in the way is still unobserved.

### Not run

- A guest charged for the memory its domain asks for:
  an 8 GiB guest measured 990 MB peak inside the unit on 2026-11-21,
  so the cap bounds what guests touch,
  not what they are configured for.
- An overshoot of the memory cap,
  `OOMPolicy=continue` sparing the daemon when the kernel picks another process,
  and `systemd-oomd` acting on the unit:
  the synthetic-load ban rules these out.
- A minimal reproduction
  (start `virsh -c qemu:///session list` inside a fresh capped scope while no daemon runs,
  then read the daemon's cgroup)
  was not run:
  a daemon was running for other sessions,
  and stopping it would have killed their guests.
- The retained journals start at 19:53:18 (user) and 21:07:07 (kernel) on 2026-11-21,
  so kills before then are not counted here.
- Which commands created the two 2-GiB scopes is not known;
  their start lines were outside the retained journal.
- A daemon that is alive but slow for more than ten seconds is probed as dead
  (`PROBE_DEADLINE_MS` in `package/mvm/cli/src/session-daemon.ts`),
  which then reads as "not answering":
  the measured probe takes 37 ms,
  so only heavy host pressure would show this.
  Recorded,
  not changed.

## Workarounds

The project chose the first candidate on 2026-11-21;
it is built and one real guest has run inside it.
The other candidates are not built.

### Built: `mvm` starts the daemon itself

Before its first `virsh` call in a process,
`mvm` runs `virsh list --all` with `LIBVIRT_AUTOSTART=0`,
which asks libvirt whether a session daemon answers without letting libvirt start one.
A daemon that answers is used as it is,
in whatever cgroup it runs.
When none answers,
`mvm` starts `virtqemud --timeout=120` itself,
as the transient user unit `mvm-libvirt-session.service`,
with `--collect`,
a `MemoryMax` of half this machine's total memory,
`OOMPolicy=continue` and `ExitType=cgroup`:

```sh
# half of MemTotal, read from /proc/meminfo when the unit starts; 31576M on this 61.7 GiB machine
systemd-run --user --unit=mvm-libvirt-session --collect \
  --property=MemoryMax=31576M --property=OOMPolicy=continue \
  --property=ExitType=cgroup \
  virtqemud --timeout=120
```

Each setting,
with its basis:

- `MemoryMax`:
  half of `MemTotal` from `/proc/meminfo`,
  read when the unit starts.
  The project chose "half of total" on 2026-11-21,
  so every guest together may use half the machine (rule `LQM`).

- `OOMPolicy=continue`:
  the kernel kills a process it picks inside the unit,
  usually the largest guest,
  and `continue` keeps systemd from stopping the unit over that kill.
  It holds against that per-process kill only;
  `systemd-oomd` acts on the whole cgroup,
  which one unit holding every session's guests makes a large candidate.
  Nothing here says the kernel must pick a guest rather than the daemon.

- `ExitType=cgroup`:
  `systemd.service(5)` on systemd 262 says a `cgroup` exit type keeps the service
  "running as long as at least one process in the cgroup has not exited",
  so the unit and its guests stay up when the daemon exits,
  as a guest outlived the daemon it had on `main`.
  `KillMode` stays the default `control-group`,
  so a stop still kills every process in the unit.
  Probed on 2026-11-21 with a `sleep` child standing in for a guest,
  the unit stayed `active` with the child after its main process exited,
  and a `systemctl --user stop` left the unit `inactive` with the child gone.

- `virtqemud --timeout=120`:
  libvirt's own idle limit,
  so the daemon exits two idle minutes after the last guest stops,
  and `--collect` then removes the unit.
  A running guest keeps the daemon up.

The unit name is the single owner:
a second starter gets systemd's "already loaded" answer
and waits for the daemon the first starter began.
A start that finds the name still loaded,
as it is while the previous daemon exits,
is tried again until a daemon answers or thirty seconds pass;
measured on 2026-11-21,
a probe asking for one unit's name every 25 ms was refused for as long as the unit ran
and accepted again 2.054 s after the start,
54 ms after that unit's two-second main process was due to exit.
The same "already loaded or has a fragment file" text also comes from a failed unit that was never collected,
and `systemctl --user reset-failed mvm-libvirt-session.service` frees that name:
probed on 2026-11-21,
a refused start of a failed unit's name was accepted after `reset-failed`.
The error says both.
`mvm` also runs every `virsh` call with `LIBVIRT_AUTOSTART=0`,
so no call ever lets `virsh` start a daemon in `mvm`'s own cgroup.

Never stop `mvm-libvirt-session.service` while a guest runs:
the unit is one for every session,
so a stop kills every session's guests,
and it needs no stop because the daemon exits by itself and `--collect` removes the unit.
A cleanup that stops the units it started must leave this one alone.

Where it lives:
`package/mvm/cli/src/session-daemon.ts` (the start,
the question,
the wait and the retries),
`package/mvm/cli/src/session-daemon-memory.ts` (half of `MemTotal`),
`package/mvm/cli/src/config.ts` (the unit name and the idle limit),
and the "Session daemon" section of `package/mvm/cli/README.md` (for users).
Measured on 2026-11-21:
the unit with `virtqemud` and `virtlogd` held 11.6 MiB (peak 14 MiB),
a started daemon answered 203 ms after `systemd-run` returned,
and `systemd-run --user` returned in 8 to 29 ms.

When `mvm` has no user manager to start a unit in,
it falls back to the old on-demand start and warns,
naming this defect and this document:
`systemd-run` missing from `PATH`,
or `systemd-run --user` unable to reach the user bus,
which a caller whose `XDG_RUNTIME_DIR` is not the user manager's gets
(recorded 2026-11-21:
"Failed to connect to user scope bus via local transport:
No such file or directory").
A caller with its own runtime directory also cannot see the daemon a unit would start,
so an isolated run needs no unit.

Tradeoffs:
`mvm` takes over a choice its README leaves to `virsh`;
every guest is still charged to one cap,
now half the machine,
so guests that together exceed it get the process the kernel picks killed,
usually the largest guest;
a `virsh` run outside `mvm` can still start a daemon in its own scope when none runs,
and `package/vm-builder/cli` still runs bare `virsh` without `LIBVIRT_AUTOSTART=0`,
so it forks daemons into its caller's cgroup:
the same defect in another package.
That package has no dependency on the mvm CLI,
its `virsh` calls go through its own `exec` runner (`src/exec.ts`,
a nano-spawn wrapper),
and its tasks run in whatever cgroup the caller has rather than a capped scope of their own,
so setting the variable there would leave it without a daemon to talk to
and no measured loss stands behind the change:
left for a change of its own,
which either calls mvm's start first or carries one;
and the Flatpak path is untested,
where the unit runs `flatpak run --command=virtqemud`,
because no Flatpak installation was available.
Still open:
an idle exit with nobody else's guest in the unit,
and how `systemd-oomd` treats this unit.

### Not built: socket-activated user units for the daemon

Socket-activated user units would let systemd start the daemon in its own service.
No libvirt user units are installed here (`/usr/lib/systemd/user` holds none),
so this means writing and enabling units in the owner's user configuration,
which is the owner's act.

Until that happens,
a command that may be the first to start a guest still needs a cap sized for the guests that may join it,
and its starter must not stop that scope by hand while a daemon is inside;
the scope ends when the daemon exits after two idle minutes.
With this workaround in place,
the daemon and the guests live in `mvm-libvirt-session.service` instead,
so that advice now applies only to a host where `mvm` had to fall back.

## What does not work

- Raising one command's cap:
  the daemon and every later guest still live in that scope,
  so its `RuntimeMaxSec=` or its starter's cleanup still kills them all.
- Reading the notification's text as a memory shortage:
  the kernel journal's `constraint=` field is the deciding evidence.

## Upstream filing decision

Checked 2026-11-21.
`.out-of-scope/` lists no libvirt or KDE entry.

### libvirt

1.  Upstream's fault:
    no.
    A daemon a client starts inheriting the client's cgroup is ordinary process creation;
    running that client inside a capped scope is this repository's choice.

The remaining constraints were not walked.
Nothing is filed.

### Plasma's `oom_notifier`

1.  Upstream's fault:
    yes for the wording.
    The text says "the system is low on memory" for a kill the kernel reports as a cgroup's own limit.
2.  Upstream can fix it:
    not checked
    (whether systemd's properties or the cgroup's `memory.events` let the module tell the two apart was not read).
3.  Supported use case:
    not checked.
4.  Contribution policy and AI-assisted reports:
    not checked (KDE files bugs on `bugs.kde.org`).
5.  Likely to fix:
    not checked;
    no duplicate search was run.
6.  Prototype:
    not attempted.

The audit is incomplete,
so the draft is kept for a later session and must not be filed as it is.
Filing would put the poster's account in front of KDE's maintainers;
whether to file is the poster's decision,
and nothing here recommends it.

~~~md
Title: oom_notifier says "the system is low on memory" when a unit hit its own MemoryMax

(Do not file as-is: constraints 2 to 6 unchecked, no duplicate search, no prototype.)

plasma-workspace 6.8.3. A user scope with `MemoryMax=2G` had a process killed by the kernel with
`constraint=CONSTRAINT_MEMCG`. oom_notifier showed "Memory Shortage Avoided ... has been terminated by the Linux kernel
because the system is low on memory", though the machine had memory free. The module reacts to `Result=oom-kill`
(oom_notifier/module.cpp lines 54 to 78 at v6.8.3) and does not distinguish a unit's own limit from a machine-wide
shortage.
~~~
