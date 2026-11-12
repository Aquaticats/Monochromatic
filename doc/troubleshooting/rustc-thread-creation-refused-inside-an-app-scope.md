# A Rust build refused a thread inside an application's scope, and rustc reported it as a compiler bug

## Symptom

On 2026-11-11,
`mise run //package/git-policy/cli:native:test:host` on a worktree of this repository
died in the compile phase of `cargo nextest run`,
in four different shapes,
all of them `EAGAIN` (errno 11,
`Resource temporarily unavailable`) from a thread that could not start:

```text
thread 'coordinator' panicked at compiler/rustc_codegen_ssa/src/back/write.rs:1910:10:
failed to spawn work thread: Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }
```

```text
thread 'main' panicked at library/std/src/thread/functions.rs:131:29:
failed to spawn thread: Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }
```

```text
terminate called after throwing an instance of 'std::system_error'
  what():  Resource temporarily unavailable
collect2: fatal error: ld terminated with signal 6 [Aborted], core dumped
```

```text
OS can't spawn worker thread: Resource temporarily unavailable (os error 11)
```

The first and third shapes are followed by `error: the compiler unexpectedly panicked. This is a bug`,
which is rustc reporting its own internals,
not the cause:
the cause is one `clone` the kernel refused.

## What the ceiling was

`journalctl --since -30min | grep "pids controller"` named the rejected forks,
and every one of them was inside a scope under
`app.slice/app-monochromatic-updates.slice/wg-quicker-update-app.slice-run-<pid>-<id>.scope.scope/`:

```text
cgroup: fork rejected by pids controller in /user.slice/user-1000.slice/user@1000.service/app.slice/app-monochromatic\x2dupdates.slice/wg-quicker-update-app.slice-run\x2dp2476658\x2di316681074.scope.scope/app.slice/app-monochromatic\x2dupdates.slice/wg-quicker-update-app.slice-pi\x2dprestage\x2dbuild.scope.scope
```

Read afterwards,
that scope held a task ceiling of 256 with 190 to 240 tasks in it and a lifetime `pids.events` `max` counter of 5490:

```bash
# the ceiling of an ancestor scope, and how full it was
cat /sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/app-monochromatic*/wg-quicker*/pids.max
cat /sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/app-monochromatic*/wg-quicker*/pids.current
```

The per-user ceilings were not the cause,
unlike [agent-browser-eagain.md](agent-browser-eagain.md):

```text
/sys/fs/cgroup/user.slice/user-1000.slice   pids.max=32768  pids.current=~5000
/sys/fs/cgroup/user.slice                     pids.max=max
```

A scope's own `TasksMax` is not the binding one either:
a probe inside `systemd-run --user --scope --property=TasksMax=4096` read back `pids.max=4096`,
and the build inside it still lost its threads to the 256 of the ancestor.

## How a build comes to run there

The unit that a tool call starts does not always land where it names.
On 2026-11-11 between 04:00 and 08:00 local,
units started from an agent's tool calls appeared in three shapes:

- under `app.slice`,
  as named (`/sys/fs/cgroup/.../app.slice/pi-prestage-build.scope`);
- under `pi.slice/pi-kimi.slice/`,
  the agent harness's own slices;
- inside the `wg-quicker-update-app` scope's subtree,
  whose unit name carries the requested slice and unit
  (`wg-quicker-update-pi.slice-pi-kimi.slice-pi-kimi-prestage.slice-pi-prestage-head.scope`).

The last shape is the one that refuses threads.
Passing `--slice=` to `systemd-run` made it more likely,
not less:
the same command landed under `pi.slice/pi-kimi.slice/` once and inside the `wg-quicker` scope the next time.

What rule the adopting tool applies was not measured.
The unit names and
`systemctl --user list-units --type=scope | grep quicker`
show that `wg-quicker` keeps one scope per application or unit it carries,
and `app-monochromatic-updates.slice` says these are its update-traffic scopes.

## What worked

An auto-named scope,
with `Delegate=yes` and no `--slice=` or `--unit=`,
survived six of seven runs on that morning:

```bash
systemd-run --user --scope \
  --property=CPUQuota=500% --property=MemoryMax=20G \
  --property=TasksMax=4096 --property=Delegate=yes --property=IOWeight=1 \
  mise run //package/git-policy/cli:native:test:host -- service_index_prestage
```

A run that is refused fails in three to four seconds,
and one that is not runs the whole build,
so the retry that costs nothing is to start it again.

## The user manager restarting under the same window

In the same window,
`systemd-run` failed once with:

```text
Failed to connect to user scope bus via local transport: Connection refused
```

`systemctl --user is-system-running` answered `running` a minute later,
and `/run/user/1000/systemd/private` had been created minutes before,
so the user manager had restarted.
Commits through the wrapper failed while it was away:

```text
The commit service could not be set up, so this commit was not made: the service manager refused the request (Unit monochromatic-git-policy-cli@a782fee3dd7087ec0dd69552f9cc5db8.socket not found.). Fix that and run the commit again. Nothing in the repository changed.
```

Starting that instance socket directly succeeded,
and the next commit landed:

```bash
systemctl --user start 'monochromatic-git-policy-cli@a782fee3dd7087ec0dd69552f9cc5db8.socket'
```

Retrying the commit after the manager answered again was enough;
nothing in the repository had to be repaired.

## Limits

- The tool that adopts the scope was not identified,
  only the shapes it leaves.
- The adoption rate was not measured;
  the counts above are one agent's attempts in one morning.
- Starting the build in a container (`native:test:container`) was not tried as an escape,
  because the container's own readiness on this host was not measured either.
