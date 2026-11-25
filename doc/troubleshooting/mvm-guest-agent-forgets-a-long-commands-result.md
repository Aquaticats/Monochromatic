# A long command in an mvm guest loses its result when the kernel ends a process in the agent's unit

## What you see

`mvm exec` or `mvm run` of a long,
memory-hungry command ends with a warning,
no result,
and an error naming the pid.
A release build is the case this was found on.

```text
# mvm's standard error
[info] [mvm] [attributedResult] the guest agent in mvm-<name> did not answer a status request; the command keeps running and is asked about again
mvm: No result in mvm-<name> can be tied to the command started as guest process <pid>.
The guest agent no longer knows that process ID and reported no finished result for it; the result was read by something else or the agent restarted.
Agent answer: The guest agent in mvm-<name> answered guest-exec-status with an error: error: guest agent command failed: unable to execute QEMU agent command 'guest-exec-status': PID ld does not exist
```

`ld` is qemu-ga's own formatting of the signed 64-bit pid (`%` with `PRId64`),
not a different pid.

## Why it happens

Two programs,
each behaving as its own source says.

- The QEMU guest agent keeps every started command in one process-wide table in memory:
  `qga/commands.c`,
  `guest_exec_state.processes`,
  filled by `guest_exec_info_add`.
  `qmp_guest_exec_status` answers `error_setg(errp, "PID %" PRId64 " does not exist")` for a pid
  the table does not hold,
  and it removes a finished command's entry the first time it reports `exited`.
  `qga/main.c` creates that state once in `main()` and frees it at exit;
  nothing resets it on a channel connect or disconnect.
  The table therefore lives and dies with the agent process.
- Every command `mvm exec` starts is a child of `qemu-ga`,
  so it runs in the agent's own systemd unit,
  `qemu-guest-agent.service`.
  systemd learns about an out-of-memory kill inside a unit from that cgroup's `memory.events`
  `oom_kill` counter (`src/core/cgroup.c`,
  `unit_check_oom`),
  and for a service its default `OOMPolicy=stop` then stops the whole unit
  (`src/core/service.c`,
  `service_notify_cgroup_oom_event`,
  `service_enter_stop` with `SERVICE_FAILURE_OOM_KILL`).
  Stopping the unit kills the agent,
  the command,
  and everything the command started,
  and the distro unit's `Restart=always` then starts a new agent whose table is empty.

A build that fills the guest loses its result that way:
the kernel ends the largest process,
systemd ends the agent,
and mvm's next status request reaches a new agent that has never heard of the pid.
mvm's five-minute retry for an unanswered request cannot help,
because the new agent answers at once:
it answers that the pid does not exist.

## The evidence

Read on 2026-11-25 in one guest created by `mvm create`:
Ubuntu 24.04.5,
kernel `6.8.0-147-generic`,
systemd 255.4-1ubuntu8.18,
`qemu-guest-agent` 1:8.2.2+ds-0ubuntu1.18,
8 GiB of memory and four vCPUs,
no swap.
The command was this repository's own `CARGO_BUILD_JOBS=2 cargo build --release --tests --locked`
for `package/git-policy/cli`,
started through one `mvm exec` as guest process 6912.
Times are the guest's clock.

The kernel,
first (`dmesg`,
which the journal also holds):

```text
# inside the guest
2026-11-25T10:36:35,991112+00:00 rustc invoked oom-killer: gfp_mask=0x140dca(...), order=0, oom_score_adj=0
2026-11-25T10:36:35,992376+00:00 oom-kill:constraint=CONSTRAINT_NONE,nodemask=(null),cpuset=/,mems_allowed=0,global_oom,task_memcg=/system.slice/qemu-guest-agent.service,task=rustc,pid=13080,uid=0
2026-11-25T10:36:35,992421+00:00 Out of memory: Killed process 13080 (rustc) total-vm:8274424kB, anon-rss:5404824kB, file-rss:2256kB, shmem-rss:0kB, UID:0 pgtables:13028kB oom_score_adj:0
```

`MemAvailable` had fallen to 2388 kB one two-second sample earlier,
and the victim was a `rustc` of the build,
5.15 GiB resident,
in the agent's cgroup.

systemd,
in the same second:

```text
# inside the guest, journalctl --unit=qemu-guest-agent.service
qemu-guest-agent.service: A process of this unit has been killed by the OOM killer.
qemu-guest-agent.service: Failed with result 'oom-kill'.
qemu-guest-agent.service: Consumed 12min 12.289s CPU time, 7.4G memory peak, 0B memory swap peak.
qemu-guest-agent.service: Scheduled restart job, restart counter is at 1.
Started qemu-guest-agent.service - QEMU Guest Agent.
```

mvm,
as the unit stopped,
and 1.4 seconds later:

```text
# the host running mvm
[info] ... [mvm] [attributedResult] the guest agent in mvm-agent-6458 did not answer a status request; the command keeps running and is asked about again
mvm: No result in mvm-agent-6458 can be tied to the command started as guest process 6912.
... PID ld does not exist
```

The command did not keep running:
the unit stop killed it,
so the build's own end marker was never written and no `cargo` or `rustc` process was left.

The same guest,
the same request for more memory than the guest has,
from a small program started through `mvm exec`,
before and after the drop-in below:

- With the distro's `OOMPolicy=stop`,
  `mvm exec` answered `PID ld does not exist`,
  the agent's `MainPID` changed,
  and `NRestarts` grew by one.
- With `OOMPolicy=continue` written and `systemctl daemon-reload` run,
  with no agent restart,
  `systemctl show --property=OOMPolicy` reported `continue` with the same `MainPID`,
  the command reported exit status 137 (128 plus 9),
  and `mvm exec` returned that result.
  The journal shows `A process of this unit has been killed by the OOM killer.`
  and no stop and no restart.

## The fix, and what it cannot do

mvm's Linux template writes `/etc/systemd/system/qemu-guest-agent.service.d/mvm-command-oom.conf`
with `OOMPolicy=continue` before the unit's first start.
The kernel then ends the one process rather than the whole unit,
the agent stays,
and a command the kernel ended reports 128 plus the signal number
while every other command's result is still there.
The file lands when an image's template is baked.

An Alpine guest runs its agent under OpenRC,
which has no such setting and no such stop:
a kernel kill of a command there leaves the agent running already,
and only a kernel that picks the agent itself loses the table.

- A template baked by an earlier mvm keeps the distro's default.
  Bake it again with `mvm update`,
  or delete the template file once no running guest's disk is backed by it,
  as [mvm-fedora-guest-confined-by-selinux.md](./mvm-fedora-guest-confined-by-selinux.md) also
  says for its own drop-in.
  `systemctl daemon-reload` alone reaches a unit that is already running:
  measured,
  with the same `MainPID`.
- The setting does not give the guest more memory.
  A build that does not fit still fails,
  now with its own error and exit status instead of silence;
  give it fewer jobs,
  a swap file,
  or more memory.
  In the run above,
  two `rustc` processes at Cargo's default parallelism reached 5.15 GiB each in a
  guest of 8 GiB.
- Any other end of the agent,
  a crash,
  a guest reboot,
  or a person restarting the unit,
  loses every started command's result the same way,
  and the error then names the pid.
  The agent keeps that table in memory only,
  which is qemu-ga's design and not something mvm can change.

## What to read in the guest after such a loss

```sh
# inside the guest, or through mvm exec
dmesg | grep -e 'Out of memory' -e 'oom-kill'
journalctl --unit=qemu-guest-agent.service --since '-10 min'
systemctl show --property=MainPID,NRestarts,Result,OOMPolicy qemu-guest-agent.service
```

An `oom-kill` line whose `task_memcg` is `/system.slice/qemu-guest-agent.service`,
followed by `Failed with result 'oom-kill'`,
names this cause.
`NRestarts` alone is not enough:
a person's `systemctl restart` and an OOM stop both raise it,
and the journal line distinguishes them.

## Limits of this reading

- The qemu-ga source read is upstream tag `v8.2.0`,
  the release the guest's `8.2.2` package is built from,
  and the systemd source is tag `v255`,
  the release the guest's `255.4` package is built from.
  Both were read on 2026-11-25,
  not from the packages' patched sources.
- One host,
  one guest image,
  one kernel release,
  and the guest's own clock.
  No other distribution was measured.
