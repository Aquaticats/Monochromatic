# AppImageKit type2 runtime `effcebc`: terminating the FUSE server before the payload app wedges that app in uninterruptible `D` state

Sending `SIGTERM` to an AppImage's FUSE server process while the payload application is still running removes the
mount out from under that application.
The application then sits in uninterruptible sleep,
ignores further `SIGTERM` deliveries,
and leaves only `SIGKILL`,
which itself takes effect only once the blocked operation completes.
Terminating the payload application first retires the server cleanly through the runtime's keepalive pipe and needs
no escalation.

Recorded while terminating JetBrains Qure so the `wg-quicker` exemption watcher could be refreshed without marking a
shared cgroup.
See `doc/handover/wg-quicker.md`,
 section "Qure deployment on 2026-10-08".

## Symptom

One `kill -TERM` naming both the payload application and the AppImage process left this state:

```text
$ pgrep -af "qure" | cut -c1-80
3862000 /tmp/.mount_qure.anGojH1/qure-ai-assistant --no-sandbox
3862010 /tmp/.mount_qure.anGojH1/qure-ai-assistant --type=zygote --no-zygote-sandbox --no-sandb
3862011 /tmp/.mount_qure.anGojH1/qure-ai-assistant --type=zygote --no-sandbox

$ grep -E "^(State|Threads)" /proc/3862000/status
State:  D (disk sleep)
Threads:        38

$ grep -c mount_qure /proc/mounts
0
```

Three properties distinguish this from an application that is merely slow to quit:

- the AppImage's own process was already gone,
   so its FUSE mount was already removed;
- the payload application reported `D`,
   uninterruptible sleep,
   not `S` or `T`;
- a second `SIGTERM` ten seconds later produced no state change at all.

`SIGKILL` to the payload application and its zygote children removed all of them within four seconds.

The reverse order on a second launch of the same AppImage terminated cleanly:

```text
$ kill -TERM 3888911   # payload application only
$ sleep 8; pgrep -af "qure" | rg -v "pgrep|bash -c"
(no output)
```

The AppImage process had already exited on its own by the time it was named,
which is why the second `kill` reported `No such process`,
and `/proc/mounts` held no `mount_qure` entry.

## Root cause

The runtime is a two-process design,
 and only one of the two is safe to signal.

AppImageKit's type2 runtime forks once.
The child becomes the FUSE server,
 and the parent becomes the application:

```c
// src/runtime.c:826-845 at effcebc1d81c5e174a48b870cb420f490fb5fb4d
    if (pid == 0) {
        /* in child */
        ...
        /* close read pipe */
        close (keepalive_pipe[0]);
        ...
        if(0 != fusefs_main (5, child_argv, fuse_mounted)){
```

```c
// src/runtime.c:855-866 at effcebc1d81c5e174a48b870cb420f490fb5fb4d
    } else {
        /* in parent, child is $pid */
        int c;

        /* close write pipe */
        close (keepalive_pipe[1]);

        /* Pause until mounted */
        read (keepalive_pipe[0], &c, 1);

        /* Fuse process has now daemonized, reap our child */
        waitpid(pid, NULL, 0);
```

The parent then replaces itself with the payload,
 so the application keeps the runtime's own lower process id:

```c
// src/runtime.c:936-941 at effcebc1d81c5e174a48b870cb420f490fb5fb4d
        char filename[mount_dir_size + 8]; /* enough for mount_dir + "/AppRun" */
        strcpy (filename, mount_dir);
        strcat (filename, "/AppRun");

        /* TODO: Find a way to get the exit status and/or output of this */
        execv (filename, real_argv);
```

That matches the observed process pair exactly:
process `3862000` resolved to `/tmp/.mount_qure.anGojH1/qure-ai-assistant` and was the payload,
while process `3862005` resolved to `/home/user/AppImages/qure.appimage`,
was a session leader,
and was reparented to `systemd --user` after the daemonizing fork was reaped,
which identifies it as the FUSE server.

Retirement runs in the other direction,
 from the payload to the server.
The FUSE server keeps a thread blocked writing into a pipe whose read end the payload holds:

```c
// src/runtime.c:134-153 at effcebc1d81c5e174a48b870cb420f490fb5fb4d
static pid_t fuse_pid;
static int keepalive_pipe[2];

static void *
write_pipe_thread (void *arg)
{
    char c[32];
    int res;
    memset (c, 'x', sizeof (c));
    while (1) {
        /* Write until we block, on broken pipe, exit */
        res = write (keepalive_pipe[1], c, sizeof (c));
        if (res == -1) {
            kill (fuse_pid, SIGTERM);
            break;
        }
    }
    return NULL;
}
```

So the supported shutdown is:
 the payload exits,
its read end of `keepalive_pipe` closes,
the server's write fails,
and the server signals itself,
taking the mount with it.

Signalling the server directly inverts that order.
The server dies while the payload still executes from the mount,
the FUSE connection is gone,
and the payload's next page-in or file read inside that mount blocks in the kernel.
A task in uninterruptible sleep cannot run a signal handler,
so the pending `SIGTERM` stays queued and the process appears to ignore it.
`SIGKILL` is likewise only acted on once the blocked operation returns.

Measured versus inferred:
the process states,
the disappearing mount,
the ineffective second `SIGTERM`,
and the clean reverse order are all directly observed on this host.
That the blocking operation was a FUSE page-in against the removed mount is inference from the payload's
`/proc/<pid>/exe` living inside that mount plus the mount's disappearance in the same window;
no `wchan` or stack trace was captured before the process was killed.

Upstream documents the mount-only mode's lifetime but not this teardown order.
Its README says `--appimage-mount` "mounts the embedded filesystem image and prints the mount point,
then waits until it is killed" (`README.md:100` at the same commit),
and says nothing about signalling the server of a running payload.

## Verification

Version under test:

- AppImageKit type2 runtime reporting `Version: effcebc`,
   resolved in a full clone to
  `effcebc1d81c5e174a48b870cb420f490fb5fb4d`,
   dated 2019-05-01;
- payload application JetBrains Qure `261002.1.0`,
   Electron `40.8.5`,
   packaged as
  `~/AppImages/qure.appimage`,
   `221322431` bytes;
- host Bazzite,
   kernel `7.2.8-ogc4.1.fc44.x86_64`.

Read the runtime version without launching the payload:

```sh
~/AppImages/qure.appimage --appimage-version
```

```text
Version: effcebc
```

Resolve that abbreviation against upstream source,
 since a shallow clone cannot see it:

```sh
gh repo clone AppImage/AppImageKit "${HOME}/temp/agent/appimagekit-20261008" -- --depth 1
cd "${HOME}/temp/agent/appimagekit-20261008"
git fetch --unshallow
git log --format="%H %ad %s" -1 effcebc
git show effcebc:src/runtime.c
```

Identify which process is which before signalling either:

```sh
for pid in $(pgrep -f "qure"); do
  printf "%-9s %s\n" "${pid}" "$(readlink "/proc/${pid}/exe")"
done
ps -o pid,ppid,stat,args -p "$(pgrep -f 'qure.appimage' | head -1)"
```

The payload's `exe` points inside `/tmp/.mount_<name>.<random>/`;
the server's `exe` points at the AppImage file itself and its `STAT` carries `s` for session leader.

Catalog of teardown orders:

1. Payload first,
    server left alone:
    clean.
   Every process,
    including the server,
    exits within eight seconds;
   the mount disappears;
   no escalation needed.
2. Payload first,
    then a redundant `kill` naming the server:
    clean.
   The server is already gone,
    so the second `kill` reports `No such process`.
3. Server first,
    payload still running:
    wedges.
   The payload enters `D`,
    ignores further `SIGTERM`,
    and needs `SIGKILL`.
4. One `kill -TERM` naming both:
    wedges.
   This is the shape that produced the recorded symptom,
    because the server won the race.
5. `SIGKILL` to the payload after the mount is already gone:
    works,
    with a delay.
   All processes were gone four seconds later.

Post-teardown state checks that passed after the clean order:

```sh
grep -c mount_qure /proc/mounts
systemctl --user list-units --all --no-legend | rg -i qure
```

A stale `tmp-.mount_<name>.<random>.mount` unit in that listing means a mount is still active,
not merely remembered:
`systemctl --user status` on it reports `Active: active (mounted)` and names the live payload.

## Verified workarounds

Terminate the payload application first and let the keepalive pipe retire the server.

```sh
kill -TERM "$(pgrep -f '/tmp/\.mount_.*qure-ai-assistant' | head -1)"
```

Tradeoff:
 the payload's own shutdown path runs,
so an application that blocks in its own quit handler still needs escalation,
and the server lingers for as long as that takes.

If the server was already killed and the payload is in `D`,
 escalate and expect delayed delivery.

```sh
kill -KILL $(pgrep -f '/tmp/\.mount_')
sleep 4
grep -c mount_ /proc/mounts
```

Tradeoff:
 `SIGKILL` skips the payload's shutdown handlers,
so an Electron payload can leave a write-ahead log or a lock file behind,
and the signal takes effect only when the blocked kernel operation returns,
which makes an immediate follow-up check report the process as still alive.

## What does not work

- Repeated `SIGTERM` to a payload in `D`.
  Two deliveries ten seconds apart changed nothing,
   because a task in uninterruptible sleep runs no handler.
- One `kill` naming both processes.
  Argument order does not control which process dies first,
   and the server's death is what removes the mount.
- Waiting for the mount to come back.
  Nothing re-establishes a FUSE connection whose server exited;
  the payload's remaining lifetime is bounded by whatever it still needs from that mount.
- Reading `/proc/<pid>/exe` of the AppImage process to find the payload.
  The server's `exe` is the AppImage file,
   so the payload must be found through the mount path instead.

## Upstream filing decision

Nothing to file.
 Default policy is not to file,
 and two constraints fail outright.

1. Is it really upstream's fault?
   No.
   The trigger was signalling the process that serves the mount a live process still executes from.
   The runtime's documented direction of teardown,
   payload exit closing `keepalive_pipe[0]` and the server signalling itself at `src/runtime.c:148`,
   worked exactly as written when used.
2. Can upstream fix it?
   Plausibly,
   by having the server ignore `SIGTERM` or forward it to the payload before unmounting.
   Not reached,
   because constraint 1 already fails.
3. Are they supporting this use case?
   No.
   `README.md:100` at `effcebc` documents killing only the `--appimage-mount` inspection mode,
   which never runs a payload.
   No documented flow covers signalling the server of a running application.
4. Would the repo welcome our contribution?
   Unresolved and not needed.
   `CONTRIBUTING.md` at `effcebc` welcomes issues and pull requests,
   states no position on AI-assisted reports,
   and that commit has no issue templates.
5. Will they likely fix it?
   No signal either way.
   The runtime commit under test dates from 2019,
   and no tracker search was run because constraints 1 and 3 already decide the outcome.
6. Have we prototyped a minimal fix?
   No.
   The prototype obligation triggers only when constraints 1 through 5 hold or sorta-hold,
   and constraints 1 and 3 fail here.
