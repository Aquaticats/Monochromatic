# systemd 259.10 and 262.1: StartTransientUnit with a PIDs entry for an exiting process leaves the scope active (running) with an empty cgroup forever

A transient scope started through `StartTransientUnit` with a `PIDs` entry that names a process which is already
exiting (a zombie its parent has not reaped) ends up `active (running)` with an empty cgroup,
and nothing in systemd ever ends it.
A process that is reaped between the bus call and the start job fails the scope instead,
with `No PIDs left to attach to the scope's control group, refusing.`,
and that failed unit stays too.
The repo's `wg-quicker-exempt` watcher
(`package/wg/cli-quicker-exempt/src/scope_adoption.rs`,
function `adopt_arguments`)
adopts short-lived processes this way and hit both outcomes at scale.
This file records the source trace,
the reproduction,
a prototype fix verified against it,
the workarounds,
and the upstream filing decision:
do not file as written,
because the target's contribution policy requires the poster to write issues in their own words.

## Symptom

The call under test has the same shape as `adopt_arguments` builds:

```sh
# StartTransientUnit(name, mode, properties, aux): two properties, then the empty aux array "0"
busctl call org.freedesktop.systemd1 /org/freedesktop/systemd1 org.freedesktop.systemd1.Manager \
  StartTransientUnit 'ssa(sv)a(sa(sv))' repro.scope fail 2 PIDs au 1 "$pid" Description s repro 0
```

What happens depends on the state of `$pid` when the manager handles the call.

- live process:
  the scope is `active (running)`,
  `cgroup.events` reads `populated 1`,
  and the scope goes `inactive` and is collected when the process ends.
  This is the case the design expects.
- zombie,
  meaning the process has exited and its parent has not reaped it:
  the call succeeds and the scope is `ActiveState=active SubState=running Result=success`
  with `cgroup.events` reading `populated 0` and an empty `cgroup.procs`.
  The debug journal says `1 process added to scope's control group.`
  The scope never ends,
  not when the zombie is later reaped and not after minutes.
- process already reaped before the call:
  the bus call fails and no unit is created (`LoadState=not-found`).
  Two error texts occur:
  `Call failed: Failed to set unit properties: No such process`
  and `Call failed: Process with ID 211 does not exist.`
- process reaped after the manager accepted the property and before the start job runs:
  the scope is `ActiveState=failed SubState=failed Result=resources`,
  the journal says `PID 386 vanished before we could move it to target cgroup '/system.slice/c3b-gated.scope',
  skipping:
  No such process` and then `No PIDs left to attach to the scope's control group,
  refusing.`,
  and the failed unit stays listed until something resets or collects it.

Host evidence from the `wg-quicker-exempt` watcher (read-only,
host systemd 259.10-1.fc44,
kernel 7.2.11-ogc4.1.fc44.x86_64,
user manager).
The census program counted `wg-quicker-update-*.scope` units by `list-units` state
and read `cgroup.events` and `cgroup.procs` of each scope directory:

- measured at 2026-11-01T01:17:04-05:00:
  3696 scope units,
  2047 `active/running` (1988 with `populated 0` and an empty `cgroup.procs`,
  59 populated with a live process),
  and 1649 `failed/failed`.
  A sample,
  `wg-quicker-update-1000703-15319883.scope`,
  was `active running` since `Sun 2026-11-01 01:50:30 EDT` with `Result=success`.
- the user journal (`journalctl --user -g 'No PIDs left to attach'`) held 1650 matching lines from
  2026-10-31T22:27:54-04:00 to 2026-11-01T01:18:02-05:00,
  one per failed unit plus one that arrived after the census.
- the coordinator measured about an hour before that:
  1293 running with `populated 0`,
  18 running and populated,
  982 failed,
  980 journal lines.
  The counts grow with the watcher's adoption rate.
  Stuck to failed was 1293 to 982 in the coordinator's counts and 1988 to 1649 in the census,
  and 28 to 46 in the throwaway tally (stock binary,
  four runs,
  in "Verification"):
  the same order of magnitude,
  not the same rate.

## Root cause

Source citations are at systemd tag `v259.10`,
commit `cc6062f23cb595c172d9a007ed99d925852bfd60`
(tag object `940355dd2367d9f27de9a13fe1254d22c5c93a47`),
the build the host runs.
Kernel citations are at `gregkh/linux` tag `v7.2.11`,
commit `a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea`
(tag object `8316c24ec1b77dba19ba072ca4fe251b36dc65d3`).
Every local copy was compared byte for byte with the blob at that commit,
and every line-range link below was loaded in `agent-browser`
and checked to highlight exactly the quoted lines (method in "Verification").
Kernel excerpts show tabs as eight spaces.

### The manager accepts any process it can open a pidfd for

The `PIDs` handler opens a pidfd for each entry,
checks the process is not a kernel thread or the manager itself,
and starts watching it
([src/core/dbus-scope.c:116-131](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/dbus-scope.c#L116-L131)):

```c
/* src/core/dbus-scope.c:116-131 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
                                r = pidref_set_pid(&pidref, upid);
                                if (r < 0)
                                        return r;

                                p = &pidref;
                        }

                        r = unit_pid_attachable(u, p, reterr_error);
                        if (r < 0)
                                return r;

                        if (!UNIT_WRITE_FLAGS_NOOP(flags)) {
                                r = unit_watch_pidref(u, p, /* exclusive= */ false);
                                if (r < 0)
                                        return r;
                        }
```

`pidref_set_pid` calls `pidfd_open`
([src/basic/pidref.c:103](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/basic/pidref.c#L103)):

```c
/* src/basic/pidref.c:103 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        fd = pidfd_open(pid, 0);
```

A process that is already reaped fails there with `ESRCH`,
which is the `Failed to set unit properties: No such process` refusal.
The `Process with ID N does not exist.` refusal is the text `unit_pid_attachable` produces when its kernel-thread probe
returns `-ESRCH`
([src/core/unit.c:6135-6137](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/unit.c#L6135-L6137)):

```c
/* src/core/unit.c:6135-6137 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        r = pidref_is_kernel_thread(pid);
        if (r == -ESRCH)
                return sd_bus_error_setf(reterr_error, SD_BUS_ERROR_UNIX_PROCESS_ID_UNKNOWN, "Process with ID " PID_FMT " does not exist.", pid->pid);
```

The observed text matches that string;
the likely trigger is a process reaped between `pidfd_open` and that probe (the window was not isolated).
A zombie passes both checks:
`pidfd_open` accepts it and it is not a kernel thread.
Neither refusal is the `No PIDs left` message the consumer's journal is full of.

### The start job counts a successful write as an attached process

The start job runs `scope_enter_running`
([src/core/scope.c:403-422](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/scope.c#L403-L422)):

```c
/* src/core/scope.c:403-422 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        r = unit_attach_pids_to_cgroup(u, u->pids, NULL);
        if (r < 0) {
                log_unit_warning_errno(u, r, "Failed to add PIDs to scope's control group: %m");
                goto fail;
        }
        if (r == 0) {
                r = log_unit_warning_errno(u, SYNTHETIC_ERRNO(ECHILD), "No PIDs left to attach to the scope's control group, refusing.");
                goto fail;
        }
        log_unit_debug(u, "%i %s added to scope's control group.", r, r == 1 ? "process" : "processes");

        s->result = SCOPE_SUCCESS;

        scope_set_state(s, SCOPE_RUNNING);

        /* Set the maximum runtime timeout. */
        scope_arm_timer(s, /* relative= */ false, scope_running_timeout(s));

        /* Unwatch all pids we've just added to cgroup. We rely on empty notifications there. */
        unit_unwatch_all_pids(u);
```

A positive count means the scope is running,
and the last statement stops watching the pids because an empty-cgroup notification is expected to end the scope later.
The count comes from `unit_attach_pids_to_cgroup`.
It verifies the pidfd,
writes the pid to `cgroup.procs`,
and counts every write that returned success
([src/core/cgroup.c:2218-2224](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/cgroup.c#L2218-L2224)
and
[src/core/cgroup.c:2252-2257](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/cgroup.c#L2252-L2257)):

```c
/* src/core/cgroup.c:2218-2224 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
                r = pidref_verify(pid);
                if (r < 0) {
                        log_unit_info_errno(u, r, "PID " PID_FMT " vanished before we could move it to target cgroup '%s', skipping: %m", pid->pid, p);
                        continue;
                }

                r = cg_attach(p, pid->pid);
```

```c
/* src/core/cgroup.c:2252-2257 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
                /* the cgroup is definitely not empty now. in case the unit was in the cgroup empty queue,
                 * drop it from there */
                unit_remove_from_cgroup_empty_queue(u);

                if (ret >= 0)
                        ret++; /* Count successful additions */
```

The comment states the assumption that fails here:
a successful write is taken to mean the cgroup is populated.
`cg_attach` is a plain write
([src/shared/cgroup-setup.c:150-152](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/shared/cgroup-setup.c#L150-L152)):

```c
/* src/shared/cgroup-setup.c:150-152 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        xsprintf(c, PID_FMT "\n", pid);

        r = write_string_file(fs, c, WRITE_STRING_FILE_DISABLE_BUFFER);
```

### The pidfd check passes for a zombie and fails only for a reaped process

`pidref_verify` returns an error only when its pidfd check does
([src/basic/pidref.c:417-424](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/basic/pidref.c#L417-L424)):

```c
/* src/basic/pidref.c:417-424 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        if (pidref->fd < 0)
                return 0; /* If we don't have a pidfd we cannot validate it, hence we assume it's all OK → return 0 */

        r = pidfd_verify_pid(pidref->fd, pidref->pid);
        if (r < 0)
                return r;

        return 1; /* We have a pidfd and it still points to the PID we have, hence all is *really* OK → return 1 */
```

and the pidfd helpers document `-ESRCH` as "already reaped"
([src/basic/pidfd-util.c:145-146](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/basic/pidfd-util.c#L145-L146),
[src/basic/pidfd-util.c:158-162](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/basic/pidfd-util.c#L158-L162),
[src/basic/pidfd-util.c:172-176](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/basic/pidfd-util.c#L172-L176)):

```c
/* src/basic/pidfd-util.c:145-146 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
         *    -EBADF   → fd invalid
         *    -ESRCH   → fd valid, but process is already reaped
```

```c
/* src/basic/pidfd-util.c:158-162 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        r = pidfd_get_pid_ioctl(fd, ret);
        if (r != -EOPNOTSUPP)
                return r;

        return pidfd_get_pid_fdinfo(fd, ret);
```

```c
/* src/basic/pidfd-util.c:172-176 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        r = pidfd_get_pid(pidfd, &current_pid);
        if (r < 0)
                return r;

        return current_pid != pid ? -ESRCH : 0;
```

A zombie still has its pid,
so the check passes;
a reaped process makes it fail,
`unit_attach_pids_to_cgroup` logs `vanished before we could move it` and `continue`s,
the count stays 0,
and `scope_enter_running` takes the `No PIDs left` branch.
That is the failed-unit outcome.
The zombie case is the one that goes on to the kernel write.

### The kernel reports success for a write that moves nothing

A task that entered `do_exit` has `PF_EXITING` set by `exit_signals`
([kernel/exit.c:929](https://github.com/gregkh/linux/blob/a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea/kernel/exit.c#L929)
and
[kernel/signal.c:3217](https://github.com/gregkh/linux/blob/a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea/kernel/signal.c#L3217)):

```c
/* kernel/exit.c:929 at a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea */
        exit_signals(tsk);  /* sets PF_EXITING */
```

```c
/* kernel/signal.c:3217 at a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea */
                tsk->flags |= PF_EXITING;
```

The `cgroup.procs` write still finds a zombie by pid
([kernel/cgroup/cgroup.c:3056-3061](https://github.com/gregkh/linux/blob/a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea/kernel/cgroup/cgroup.c#L3056-L3061)):

```c
/* kernel/cgroup/cgroup.c:3056-3061 at a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea */
        if (pid) {
                tsk = find_task_by_vpid(pid);
                if (!tsk) {
                        tsk = ERR_PTR(-ESRCH);
                        goto out_unlock_rcu;
                }
```

then `cgroup_attach_task` calls `cgroup_migrate`
([kernel/cgroup/cgroup.c:3033-3035](https://github.com/gregkh/linux/blob/a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea/kernel/cgroup/cgroup.c#L3033-L3035)),
which calls `cgroup_migrate_add_task` for the task,
and that function returns without adding an exiting task
([kernel/cgroup/cgroup.c:2594-2596](https://github.com/gregkh/linux/blob/a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea/kernel/cgroup/cgroup.c#L2594-L2596)):

```c
/* kernel/cgroup/cgroup.c:3033-3035 at a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea */
        ret = cgroup_migrate_prepare_dst(&mgctx);
        if (!ret)
                ret = cgroup_migrate(leader, threadgroup, &mgctx);
```

```c
/* kernel/cgroup/cgroup.c:2594-2596 at a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea */
        /* @task either already exited or can't exit until the end */
        if (task->flags & PF_EXITING)
                return;
```

With no task in the taskset,
`cgroup_migrate_execute` skips the permission and attach callbacks
(both are guarded by `if (tset->nr_tasks)`)
and returns 0
([kernel/cgroup/cgroup.c:2753-2754](https://github.com/gregkh/linux/blob/a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea/kernel/cgroup/cgroup.c#L2753-L2754)):

```c
/* kernel/cgroup/cgroup.c:2753-2754 at a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea */
        ret = 0;
        goto out_release_tset;
```

The write handler passes that 0 up and returns the byte count
([kernel/cgroup/cgroup.c:5441](https://github.com/gregkh/linux/blob/a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea/kernel/cgroup/cgroup.c#L5441)
and
[kernel/cgroup/cgroup.c:5454](https://github.com/gregkh/linux/blob/a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea/kernel/cgroup/cgroup.c#L5454)):

```c
/* kernel/cgroup/cgroup.c:5441 at a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea */
        ret = cgroup_attach_task(dst_cgrp, task, threadgroup);
```

```c
/* kernel/cgroup/cgroup.c:5454 at a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea */
        return __cgroup_procs_write(of, buf, true) ?: nbytes;
```

So the write succeeds and the cgroup stays empty.
The zombie was not counted as populating its old cgroup either:
a dead task is unlinked from its `css_set` when it dies
([kernel/cgroup/cgroup.c:7098-7101](https://github.com/gregkh/linux/blob/a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea/kernel/cgroup/cgroup.c#L7098-L7101)):

```c
/* kernel/cgroup/cgroup.c:7098-7101 at a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea */
        WARN_ON_ONCE(list_empty(&tsk->cg_list));
        cset = task_css_set(tsk);
        css_set_move_task(tsk, cset, NULL, false);
        cset->nr_tasks--;
```

The kernel comment on the early return,
"@task either already exited or can't exit until the end",
shows the no-op is intended kernel behavior,
so the kernel is not the component to change.

### Nothing in systemd re-checks a scope that never became populated

The manager watches `cgroup.events` with inotify for `IN_MODIFY`
([src/core/cgroup.c:1972](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/cgroup.c#L1972)
and
[src/core/cgroup.c:1993](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/cgroup.c#L1993)):

```c
/* src/core/cgroup.c:1972 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        /* Watches the "cgroups.events" attribute of this unit's cgroup for "empty" events. */
```

```c
/* src/core/cgroup.c:1993 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        crt->cgroup_control_inotify_wd = inotify_add_watch(u->manager->cgroup_inotify_fd, events, IN_MODIFY);
```

The kernel modifies `cgroup.events` when `populated` changes.
A cgroup that is created empty and never receives a task never changes it,
so no event arrives.
The only way from an event to the "cgroup is empty" action is `unit_check_cgroup_events`,
which reads `populated` and queues the empty handling
([src/core/cgroup.c:3151-3165](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/cgroup.c#L3151-L3165)):

```c
/* src/core/cgroup.c:3151-3165 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        r = cg_get_keyed_attribute(
                        crt->cgroup_path,
                        "cgroup.events",
                        STRV_MAKE("populated", "frozen"),
                        values);
        if (r < 0)
                return r;

        /* The cgroup.events notifications can be merged together so act as we saw the given state for the
         * first time. The functions we call to handle given state are idempotent, which makes them
         * effectively remember the previous state. */
        if (streq(values[0], "1"))
                unit_remove_from_cgroup_empty_queue(u);
        else
                unit_add_to_cgroup_empty_queue(u);
```

A search of `src/core/*.c` and `src/core/*.h` for the callers (`rg` over `unit_add_to_cgroup_empty_queue`,
`unit_check_cgroup_events` and `unit_cgroup_catchup`) found exactly two callers of `unit_check_cgroup_events`:
the inotify handler
([src/core/cgroup.c:3209-3211](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/cgroup.c#L3209-L3211))
and `unit_cgroup_catchup`
([src/core/cgroup.c:3986-3991](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/cgroup.c#L3986-L3991)):

```c
/* src/core/cgroup.c:3209-3211 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
                        u = hashmap_get(m->cgroup_control_inotify_wd_unit, INT_TO_PTR(e->wd));
                        if (u)
                                unit_check_cgroup_events(u);
```

```c
/* src/core/cgroup.c:3986-3991 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        /* We dropped the inotify watch during reexec/reload, so we need to
         * check these as they may have changed.
         * Note that (currently) the kernel doesn't actually update cgroup
         * file modification times, so we can't just serialize and then check
         * the mtime for file(s) we are interested in. */
        (void) unit_check_cgroup_events(u);
```

`unit_cgroup_catchup` has one caller,
`unit_catchup`
([src/core/unit.c:3816-3823](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/unit.c#L3816-L3823)),
which `manager_catchup` runs for every unit when the manager becomes ready after a reload or re-execution
([src/core/manager.c:1966-1967](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/manager.c#L1966-L1967)):

```c
/* src/core/unit.c:3816-3823 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
void unit_catchup(Unit *u) {
        assert(u);

        if (UNIT_VTABLE(u)->catchup)
                UNIT_VTABLE(u)->catchup(u);

        unit_cgroup_catchup(u);
}
```

```c
/* src/core/manager.c:1966-1967 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        /* Let's finally catch up with any changes that took place while we were reloading/reexecing */
        manager_catchup(m);
```

The process-exit path that does exist for other units is gone too:
`scope_enter_running` called `unit_unwatch_all_pids`,
and the queue's own comment says why scopes depend on the empty notification
([src/core/cgroup.c:2932-2935](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/cgroup.c#L2932-L2935)):

```c
/* src/core/cgroup.c:2932-2935 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
        /* Note that cgroup empty events are dispatched in a separate queue with a lower priority than
         * the SIGCHLD handler, so that we always use SIGCHLD if we can get it first, and only use
         * the cgroup empty notifications if there's no SIGCHLD pending (which might happen if the cgroup
         * doesn't contain processes that are our own child, which is typically the case for scope units). */
```

A running scope ends only through `scope_notify_cgroup_empty_event`
([src/core/scope.c:568-575](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/scope.c#L568-L575)):

```c
/* src/core/scope.c:568-575 at cc6062f23cb595c172d9a007ed99d925852bfd60 */
static void scope_notify_cgroup_empty_event(Unit *u) {
        Scope *s = ASSERT_PTR(SCOPE(u));

        log_unit_debug(u, "cgroup is empty");

        if (IN_SET(s->state, SCOPE_RUNNING, SCOPE_ABANDONED, SCOPE_STOP_SIGTERM, SCOPE_STOP_SIGKILL))
                scope_enter_dead(s, SCOPE_SUCCESS);
}
```

Putting it together:
the zombie write succeeds and moves nothing,
`scope_enter_running` counts it and stops watching the pid,
the cgroup never becomes populated so the kernel never notifies,
and the one function that reads `populated` on demand runs only after a reload or re-execution.
This is also why `systemctl daemon-reload` clears existing stuck scopes (see "Verified workarounds").

### The same code is in every maintained line checked

A function-body comparison through the GitHub contents API (`fn-diff` run on 2026-11-01)
found `scope_enter_running`,
`scope_notify_cgroup_empty_event`,
`unit_cgroup_catchup` and `pidref_verify`
text-identical in `v258.12`,
`v259.10`,
`v260.6`,
`v261.4` and `v262.1`,
and `unit_attach_pids_to_cgroup` and `unit_check_cgroup_events` identical in `v259.10`,
`v260.6`,
`v261.4` and `v262.1`
(`v258.12` differs in those two by a few lines that were not analyzed).
The default branch (`main`,
commit `fad40b4fb1a18925e886c8b1a93e2681591b16a8`,
read 2026-11-01)
has the same statements in `scope_enter_running` and `unit_attach_pids_to_cgroup`;
the only textual difference is the `/* suffix_path= */` comment from commit `ef69c63a7b` (2026-10-30).
The `v262.1` copies
([src/core/scope.c:399-418](https://github.com/systemd/systemd/blob/73612ea502d53baf9b0df4336cfc296a7ee0d524/src/core/scope.c#L399-L418)
and
[src/core/cgroup.c:2265-2271](https://github.com/systemd/systemd/blob/73612ea502d53baf9b0df4336cfc296a7ee0d524/src/core/cgroup.c#L2265-L2271),
commit `73612ea502d53baf9b0df4336cfc296a7ee0d524`)
are the ones the prototype patches.

### Readings that were wrong

- The brief expected a PID that is fully gone at call time to produce the `No PIDs left` refusal and a kept failed unit.
  It does not.
  The bus call fails (`Failed to set unit properties: No such process`) and no unit exists.
  `No PIDs left` needs a process that was alive or a zombie when the manager accepted the property and was reaped before
  the start job ran.
  The natural race window is small;
  the reproduction widens it deterministically with a start job that waits for another unit
  (case "reaped before the start job").
- The kernel "commit" `8316c24ec1b77dba19ba072ca4fe251b36dc65d3` for `v7.2.11` is the annotated tag object.
  `gh api repos/gregkh/linux/git/ref/tags/v7.2.11` returns type `tag` with that sha,
  and `gh api repos/gregkh/linux/git/tags/<that sha>` dereferences it to commit
  `a2de78ee6ed8bf8cd99d177c74865a6e8c3cb6ea`.
  The same applies to systemd `v259.10`
  (tag object `940355dd...` to commit `cc6062f23cb595c172d9a007ed99d925852bfd60`).
  Permalinks use the commits.
- Open questions from the brief,
  answered:
  `cgroup_procs_write` does return success for an exiting task (the chain above),
  and `pidref_verify` accepts a zombie (reproduced:
  the debug journal says `1 process added`).

## Verification

### Environment

- host,
  read-only census only:
  systemd 259.10-1.fc44 (`rpm -q systemd`),
  kernel 7.2.11-ogc4.1.fc44.x86_64,
  user manager.
  No transient unit was created in the host's managers.
- throwaway probe:
  rootless podman 5.8.9 on that host kernel,
  image `docker.io/cachyos/cachyos-v4:latest` (the repo's default base image) with systemd `262 (262.1-1-arch)` as
  PID 1,
  so the system manager runs as root inside the container's cgroup namespace.
  The container ran with `--memory=2g --cpus=2 --pids-limit=1024 --network=none --systemd=always
  --security-opt label=disable`.
  The kernel is the host's 7.2.11,
  so the kernel citations apply to the observed runs.
- the 259.10 line itself was not reproduced in a throwaway.
  The Fedora 44 base image has no `systemd` package,
  and `dnf install` stalled on a mirror reset (`Curl error (56)` on `updates-released-f44`) and was stopped.
  The 259.10 coverage is the host census,
  plus the function-body comparison that shows its code equals `v262.1`'s for the functions on the path.
- why not the VM the brief asked for:
  the `mvm` MCP server failed on every call
  (`Cannot find module '.../package/mcp/mvm/dist/final/node/libvirt-C2IRPclg.mjs'`;
  the directory holds `libvirt-B8wDpRpJ.mjs`,
  so the running server predates a rebuild).
  The `mvm` CLI worked,
  and a Fedora 43 VM booted (systemd 258-1.fc43,
  kernel 6.17.1-300.fc43.x86_64),
  but its guest agent runs as SELinux domain `virt_qemu_ga_t`:
  `systemctl is-system-running` answered `unknown` and `systemctl` failed with `Access denied`,
  so no `StartTransientUnit` could be issued through `mvm exec`.
  The serial-console route returned no output.
  The VM was destroyed (`mvm list` prints `no VMs found`).
- the probe image needed one drop-in:
  `dbus-broker-launch` fails in this rootless container (`launcher_run_child ... Package not installed`),
  and `busctl` cannot use the manager's private socket because it sends a `Hello` the socket rejects
  (`Invalid request descriptor`),
  so `dbus-broker.service` is overridden to run the reference `dbus-daemon`.

### Harness

The probe image is the CachyOS image plus the `dbus-daemon` drop-in:

```dockerfile
# Containerfile.run, built with: podman build --network=none --tag localhost/systemd-scope-probe:run --file Containerfile.run .
ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos-v4:latest
FROM ${CACHYOS_IMAGE}
RUN mkdir --parents /etc/systemd/system/dbus-broker.service.d
RUN printf '[Service]\nType=notify\nExecStart=\nExecStart=/usr/bin/dbus-daemon --system --address=systemd: --nofork --nopidfile --systemd-activation --syslog-only\n' > /etc/systemd/system/dbus-broker.service.d/probe.conf
STOPSIGNAL SIGRTMIN+3
CMD ["/usr/lib/systemd/systemd"]
```

Start a container from it:

```sh
# host: throwaway system manager, capped
podman run --detach --name scope-stock --memory=2g --cpus=2 --pids-limit=1024 --network=none \
  --systemd=always --security-opt label=disable localhost/systemd-scope-probe:run
# inside the container, run once: debug logging so the journal shows the attach path
podman exec scope-stock systemd-analyze log-level debug
```

Each case is one `podman exec scope-stock bash -c '<commands>'` and uses a shell helper that wraps the `busctl` call
shown in "Symptom".
Background processes are started with their output redirected to `/dev/null`,
because `podman exec` otherwise waits for them.
The cases (the `adopt` helper is `busctl call ... StartTransientUnit` with `PIDs au 1 <pid>` and `Description`):

```sh
# control: a live process, then its death
sleep 30 >/dev/null 2>&1 & P=$!
adopt c1-control.scope $P
systemctl show c1-control.scope -p ActiveState -p SubState; cat /sys/fs/cgroup/system.slice/c1-control.scope/cgroup.events
kill $P; sleep 1.5; systemctl show c1-control.scope -p ActiveState -p LoadState

# zombie: the child of an exec'd sleep is a zombie because the new sleep never waits for it
sh -c 'sleep 0.1 & exec sleep 60' >/dev/null 2>&1 & PARENT=$!
sleep 0.5; ZOMBIE=$(ps --ppid "$PARENT" -o pid= | tr -d ' ')
adopt c2-zombie.scope "$ZOMBIE"
kill "$PARENT"; sleep 1.5      # the zombie is reparented to PID 1 and reaped
systemctl show c2-zombie.scope -p ActiveState -p SubState; cat /sys/fs/cgroup/system.slice/c2-zombie.scope/cgroup.events

# gone: reaped before the call
sleep 0.1 >/dev/null 2>&1 & P=$!; wait "$P"
adopt c3a-gone.scope "$P"

# reaped before the start job: hold the scope's start job behind a 4 s oneshot, let the process die and be reaped
systemd-run --no-block --unit=gate-a --service-type=oneshot sleep 4
sleep 0.3 >/dev/null 2>&1 & P=$!
adopt c3b-gated.scope "$P" 2 After as 1 gate-a.service Wants as 1 gate-a.service
sleep 6; systemctl show c3b-gated.scope -p ActiveState -p Result
```

### Results on systemd 262.1-1-arch, kernel 7.2.11-ogc4.1.fc44.x86_64

Control,
live process:

```text
-- after start (live sleep 122):
LoadState=loaded ActiveState=active SubState=running Result=success
  cgroup.events: populated 1 frozen 0  cgroup.procs: [122]
-- after killing the sleep:
LoadState=not-found ActiveState=inactive SubState=dead Result=success
  cgroup dir: absent
c1-control.scope: cgroup is empty
c1-control.scope: Deactivated successfully.
```

Zombie (stuck running):

```text
-- parent (exec'd sleep 60) 134, child 136:
    PID    PPID STAT COMMAND
    134     133 S    sleep
    136     134 Z    sleep <defunct>
-- after start (zombie 136 adopted):
LoadState=loaded ActiveState=active SubState=running Result=success
  cgroup.events: populated 0 frozen 0  cgroup.procs: []
-- after the parent died and the zombie was reaped:
LoadState=loaded ActiveState=active SubState=running Result=success
  cgroup.events: populated 0 frozen 0  cgroup.procs: []
c2-zombie.scope: 1 process added to scope's control group.
c2-zombie.scope: Changed dead -> running
```

Reaped before the call:

```text
Call failed: Failed to set unit properties: No such process
exit=1
LoadState=not-found ActiveState=inactive
```

Reaped before the start job (failed,
stays without `CollectMode`,
collected with it):

```text
-- after the gate released the scope:
LoadState=loaded ActiveState=failed SubState=failed Result=resources
c3b-gated.scope: PID 162 vanished before we could move it to target cgroup '/system.slice/c3b-gated.scope', skipping: No such process
c3b-gated.scope: No PIDs left to attach to the scope's control group, refusing.
c3b-gated.scope: Failed with result 'resources'.
-- still listed, without CollectMode:
● c3b-gated.scope loaded failed failed repro
-- with CollectMode=inactive-or-failed:
(empty above means collected)
c3c-gated-collect.scope: Collecting.
```

Natural race tally,
bounded:
64 adoptions per run (8 repetitions of process lifetimes 1,
2,
3,
5,
8,
12,
20 and 40 ms,
one `sleep` each,
sequential),
counted per run as refused at call time,
stuck running with `populated 0`,
and failed with `No PIDs left`.
The remainder of the accepted calls completed normally and were collected.

- stock `262.1-1-arch`,
  four runs
  (the first in a hand-configured container from the same image and drop-in,
  the other three in the baked image):
  stuck running 6,
  5,
  9 and 8 (28 of 256 adoptions);
  failed 16,
  11,
  13 and 6;
  refused 20,
  21,
  24 and 22.
- built from tag `v262.1` unpatched,
  one run:
  stuck running 2,
  failed 10,
  refused 14.

The tally shows the shape of the host's census (stuck and failed in the same order of magnitude),
not its rate:
the watcher's timing differs,
and the run-to-run spread is wide (stuck 5 to 9 on the same binary).

### Catalogs

Works cleanly:

- a live process:
  running,
  `populated 1`,
  ends when the process ends (control).
- a process reaped before the call:
  a bus error,
  no unit,
  nothing to clean up.
- `CollectMode=inactive-or-failed` collects a scope that failed with `No PIDs left`.
- `systemctl stop NAME.scope` ends a stuck scope (exit 0).
- `systemctl daemon-reload` ends every stuck scope (see "Verified workarounds").

Fails,
by variant:

- stuck running:
  a zombie at attach time gives `active (running)`,
  `populated 0`,
  empty `cgroup.procs`,
  no end.
- failed `resources`:
  a process reaped between the property being accepted and the start job gives `No PIDs left`,
  and the failed unit stays without `CollectMode`.
- bus refusal:
  `Failed to set unit properties: No such process` or `Process with ID N does not exist.`
  Not a leak,
  but a caller that adopts without handling the error loses the adoption.

### Link verification method

`verify-links` (scratch,
not in the repo) opened each commit-pinned line-range link in `agent-browser`
(dedicated session) and checked that the highlighted rows' `data-line-number` values were exactly the cited range
and that each row's text equalled the same line of the local file the excerpt was copied from.
All 36 line-range links passed on 2026-11-01
(systemd `cc6062f2` and `73612ea5`,
kernel `a2de78ee`,
`docs/CONTRIBUTING.md` and the bug form at `fad40b4f`).
Issue and pull-request links were loaded the same way and checked for the quoted text on the page
(and inside the anchored comment for the two comment permalinks).
The text-fragment links could not be confirmed to scroll to the quote in the headless browser
(GitHub renders those pages client-side and the fragment did not scroll),
so for them only the presence of the exact quoted text on the loaded page is confirmed.

## Prototype fix

Constraints 1 to 5 hold or soft-hold (see "Upstream filing decision"),
so constraint 6 was prototyped.

### Setup

- disposable clone:
  `gh repo clone systemd/systemd <fresh mktemp directory> -- --depth 1 --branch v262.1`;
  the clone's `origin` is `https://github.com/systemd/systemd.git` and `HEAD` is
  `73612ea502d53baf9b0df4336cfc296a7ee0d524`,
  the commit the `v262.1` tag dereferences to.
- build image:
  the repo's default CachyOS image plus `pacman -Sy --noconfirm --needed meson ninja gperf python-jinja
  python-pyelftools`.
- build,
  a stated real job in a capped container with no network:
  `podman run --rm --memory=8g --cpus=4 --network=none --security-opt label=disable --volume <clone>:/src:rw
  localhost/systemd-scope-probe:build meson setup build -Dprefix=/usr -Dmode=release -Dman=disabled -Dhtml=disabled
  -Dtranslations=false -Dtests=false -Dfuzz-tests=false -Defi=false -Dbootloader=disabled -Dukify=disabled
  -Dkernel-install=false`
  then `meson compile -C build -j 4 ./systemd:executable`
  (the bare target name `systemd` is ambiguous between an executable and a library).
  The unpatched build ran 644 ninja steps;
  the patched rebuild recompiled only `scope.c` and relinked `libsystemd-core-262.so`.
- run:
  a container from the probe image whose PID 1 is the built binary
  (`--volume <clone>/build:/probe-build:ro ... /probe-build/systemd`),
  which resolves its own `libsystemd-core-262.so` and `libsystemd-shared-262.so` through `$ORIGIN`
  (the image's tools keep using the packaged `...-262.1-1.so` libraries).
  `readlink /proc/1/exe` printed `/probe-build/systemd` and `/proc/1/maps` listed the built `libsystemd-core-262.so`.
- the unpatched build is the control for the build and mount method:
  it reproduces the stuck scope exactly like the packaged binary.

### The patch

One call,
inside `scope_enter_running` after the pids are unwatched,
reusing the existing `unit_cgroup_catchup` that the reload path uses
(it is declared in `src/core/cgroup.h`,
so the change touches one file):

```diff
--- a/src/core/scope.c
+++ b/src/core/scope.c
@@ -417,6 +417,11 @@
         /* Unwatch all pids we've just added to cgroup. We rely on empty notifications there. */
         unit_unwatch_all_pids(u);
 
+        /* Writing a PID to cgroup.procs succeeds without moving anything if the process is already
+         * exiting (a zombie nobody reaped yet), so the cgroup might be empty by now and then the kernel
+         * will never send an empty notification for it. Check explicitly once. */
+        unit_cgroup_catchup(u);
+
         return 1;
 
 fail:
```

Why it is sound:
the kernel updates `populated` synchronously inside the migration that the write performs,
so after a write that moved a live task `populated` is already 1 and the check removes the scope from the empty queue;
after a write that moved nothing it is 0 and the check queues the normal empty handling,
which ends the scope through `scope_notify_cgroup_empty_event` exactly as when a process exits right after attaching.
The queue is dispatched from the event loop,
so the start job still completes and the scope then stops.
`unit_cgroup_catchup` also queues an out-of-memory check
(`unit_add_to_cgroup_oom_queue`,
harmless but broader than needed);
a narrower alternative is to export `unit_check_cgroup_events` and call that,
at the cost of touching `cgroup.c` and `cgroup.h` as well.

### Results, before and after

- zombie case,
  unpatched build:
  stuck,
  identical to the packaged binary.
- zombie case,
  patched build:

```text
-- after start (zombie 120 adopted):
LoadState=not-found ActiveState=inactive SubState=dead Result=success
  cgroup dir: absent
Started repro.
c2-zombie.scope: cgroup is empty
c2-zombie.scope: Deactivated successfully.
c2-zombie.scope: Changed running -> dead
c2-zombie.scope: Collecting.
```

- control,
  patched build:
  a live process still gives `populated 1` and the scope ends when the process ends.
- natural tally,
  patched build,
  three runs:
  stuck running 0,
  0 and 0 (0 of 192 adoptions);
  failed 12,
  18 and 12;
  refused 18,
  23 and 26.
  The stock binary left 28 of 256 stuck.
  The `No PIDs left` failures are unchanged in kind,
  as expected:
  that is the reaped-before-start path,
  which this patch does not touch and which fails the scope correctly.
- reaped-before-start case,
  patched build:
  still `failed`,
  `Result=resources`,
  `No PIDs left`.

Limits of this prototype:
upstream's own test suite was not run (its integration tests need `mkosi` virtual machines);
only the system manager as root was exercised,
not the user manager the host uses (the code path is the same function);
scopes with `User=` (the `SCOPE_START_CHOWN` route into the same function) and `Delegate=yes` were not exercised;
one distribution build and one kernel were used.

## Verified workarounds

All five were run in the stock `262.1-1-arch` container (system manager) on 2026-11-01.
None was run against the host's real user manager.

- `systemctl daemon-reload` ends every stuck scope.
  After adopting a zombie,
  `daemon-reload` left the scope `inactive`,
  `LoadState=not-found`,
  with
  `Collecting.` in the journal.
  Mechanism:
  the reload path runs `unit_cgroup_catchup` for every unit (see "Nothing in systemd re-checks").
  Tradeoffs:
  it is a cleanup,
  not prevention,
  and stuck scopes come back at the adoption rate;
  a reload re-reads unit files and briefly interrupts the manager's bus
  (for the user manager,
  use `systemctl --user daemon-reload`);
  it does not clear failed `No PIDs left` units (those stay `failed`).
  A first probe that ran `daemon-reload` before the `stop` and `RuntimeMaxUSec` probes collected all three test scopes at
  once and invalidated the other two;
  each probe now uses its own scope.
- `systemctl stop NAME.scope` on a scope whose `cgroup.events` says `populated 0` ends it (exit 0,
  `inactive`).
  Tradeoffs:
  the consumer must check `populated` after the start job completes (a live adoption has `populated 1`),
  and a check-then-stop pair races with a process that is being moved in;
  re-read `populated` immediately before the stop.
- `CollectMode=inactive-or-failed` as a property of the adopted scope
  (`CollectMode s inactive-or-failed` in the property array;
  the property count rises by one)
  makes the failed `No PIDs left` units disappear on their own (`Collecting.` in the journal).
  Tradeoffs:
  it does not help the stuck-running scopes,
  and it removes the failed record that is otherwise the only per-process evidence besides the journal.
- `RuntimeMaxUSec t 4000000` bounds the stuck state:
  after 4 s the scope was `failed`,
  `Result=timeout`.
  Tradeoffs:
  the bound applies to legitimate adoptions as well,
  so a long-lived adopted process is stopped at the limit (the manager stops the scope's processes),
  the stuck scope still lives for the whole bound,
  and the result is a `failed` unit that needs `CollectMode` to disappear.
- handle the bus refusals (`No such process` and `Process with ID N does not exist.`) as "the process is gone" and
  skip the adoption.
  Tradeoff:
  that loses nothing,
  but it does not address stuck or failed scopes.

## What does not work

- The `mvm` MCP tools and the Fedora VM,
  for the reasons in "Environment".
- `busctl` against `/run/systemd/private`:
  the private socket rejects `Hello`.
- `dbus-broker` in this rootless container.
- Checking that the process is not a zombie (`/proc/<pid>/stat` state `Z`) immediately before adopting
  narrows the window but cannot close it:
  the process can exit between the check and the kernel write.
  It was not measured here and is not a verified workaround.
- Expecting the scope to end when the zombie is reaped:
  the pid watch was dropped at `unit_unwatch_all_pids`,
  and the reaping produces no cgroup event.
- Expecting `RuntimeMaxUSec` or `CollectMode` to prevent the stuck state:
  they only bound or tidy it.

## Upstream filing decision

Default policy is do not file.
The six-constraint walk:

1.
    Is it really upstream's fault?
    Partly,
    which is a soft yes.
    The kernel behavior is intentional and documented by its own comment (see "The kernel reports success").
    systemd's contribution is the assumption that a successful `cgroup.procs` write means a populated cgroup,
    and the lack of any path that ends a running scope whose cgroup is empty and never changes.
    A maintainer could answer that the caller handed PID 1 a process that was already exiting,
    and a maintainer stated a related principle:
    "clients need to wait for the "Job" object returned by StartUnit() to signal completion before they can assume the
    processes that shall be part of the scope are actually part of it"
    ([pull request 42910, comment](https://github.com/systemd/systemd/pull/42910#issuecomment-4913164922),
    on a different bug:
    a stop request during `SCOPE_START_CHOWN`).
    That statement does not cover a scope left running forever after the job completed.
2.
    Can upstream fix it?
    Yes.
    The prototype is one call and the code path is not architecturally constrained.
3.
    Are they supporting this use case?
    Soft yes.
    The implemented `PIDs` setting is used by systemd's own components:
    a pull request in this area describes libvirt,
    machined and PID 1 creating a scope for a leader process
    ([pull request 20577](https://github.com/systemd/systemd/pull/20577#:~:text=now%20the%20scope%20and%20machine%20is%20in%20active%20but%20no%20leader%20process%20exist)),
    and a logind session scope hit the refusal in a test
    ([issue 24040](https://github.com/systemd/systemd/issues/24040#:~:text=No%20PIDs%20left%20to%20attach%20to%20the%20scope)).
    The maintainer's stated rule for the same symptom class was
    "this should be fixed in pid1,
    not worked around in machined"
    ([pull request 20577, comment](https://github.com/systemd/systemd/pull/20577#issuecomment-930276498)).
    Against that,
    `PIDs` is not listed for scopes in `man/org.freedesktop.systemd1.xml` ("Scope Unit Objects") or in
    `docs/TRANSIENT-SETTINGS.md` ("Scope Unit Settings") at `v259.10`,
    so a maintainer could call it internal API
    (`rg` for the word `PIDs` in both files found only an unrelated sentence).
4.
    Would the repo welcome the contribution?
    Not as an agent-written filing,
    so this constraint fails for the draft in "Draft".
    `docs/CONTRIBUTING.md` at `fad40b4fb1a18925e886c8b1a93e2681591b16a8` allows AI help but says
    ([lines 69-71](https://github.com/systemd/systemd/blob/fad40b4fb1a18925e886c8b1a93e2681591b16a8/docs/CONTRIBUTING.md?plain=1#L69-L71)):
    "You must speak for yourself.
    Do not let an AI agent open pull requests or post for you.
    Write issues,
    review
    comments,
    and posts to the systemd-devel mailing list yourself."
    and
    ([lines 82-84](https://github.com/systemd/systemd/blob/fad40b4fb1a18925e886c8b1a93e2681591b16a8/docs/CONTRIBUTING.md?plain=1#L82-L84)):
    "The first time maintainers find that you let AI communicate for you,
    or that you posted AI-written text without
    reviewing it,
    you get a warning.
    The second time,
    you are banned from the project."
    So an agent must not open the issue or the pull request,
    and a human who files must write the text in their own words;
    the draft is a fact sheet for that human,
    not text to paste.
    Other intake facts:
    blank issues are enabled;
    the bug form asks for the systemd version (required),
    distribution,
    kernel,
    architecture,
    component,
    and the expected,
    unexpected and reproduction texts;
    bugs are tracked only for the two most recently released major versions
    ([`docs/CONTRIBUTING.md` lines 16-17](https://github.com/systemd/systemd/blob/fad40b4fb1a18925e886c8b1a93e2681591b16a8/docs/CONTRIBUTING.md?plain=1#L16-L17)
    and the form's
    [line 15](https://github.com/systemd/systemd/blob/fad40b4fb1a18925e886c8b1a93e2681591b16a8/.github/ISSUE_TEMPLATE/bug_report.yml?plain=1#L15)),
    which on 2026-11-01 are 262 and 261
    (`v262.1` was published 2026-10-16;
    `v261.4`,
    `v260.6`,
    `v259.10` and `v258.12` on 2026-10-17).
    The host's 259.10 is outside that window,
    so a filing would cite 262.1.
5.
    Will they likely fix it?
    Soft yes.
    The maintainers added the `No PIDs left` refusal for the sibling symptom
    ([pull request 21163](https://github.com/systemd/systemd/pull/21163),
    commit `8d3e4ac7cd37200d1431411a4b98925a24b7d9b3`)
    after the lead maintainer wrote that a scope left active with no leader process
    ([comment](https://github.com/systemd/systemd/pull/20577#issuecomment-929481551))
    "looks like a bug in PID1's scope unit management".
    The follow-up fix for the refusal's regression recorded the limitation that a process which is gone after a delegated
    attach cannot be told apart
    ([pull request 21298](https://github.com/systemd/systemd/pull/21298#:~:text=all%20processes%20to%20attach%20are%20gone%20in%20the%20meantime)).
    No documented won't-fix or stated non-goal was found,
    and no maintainer declined this specific request.
6.
    Has a minimal fix been prototyped?
    Yes (section "Prototype fix"):
    one call,
    verified against the zombie reproduction before and after
    (stuck running before,
    ended and collected after)
    and against a natural-race tally (28 of 256 stuck on the stock binary,
    0 of 192 patched).
    Not run:
    upstream's test suite.

Decision:
do not file as written.
Constraint 4 blocks an agent-authored issue or pull request under the project's written policy,
and filing is the poster's decision either way.
If the owner chooses to report it,
they would write the issue themselves from the facts in this file,
cite `v262.1`,
and consider attaching the prototype as a patch they have read and can explain
(the policy requires that they understand and own it).

### Duplicate and prior-art searches

All run on 2026-11-01 with `gh search issues` and `gh search prs`,
`--repo systemd/systemd --limit 15`,
open and closed state,
through a script that printed number,
state,
update date and title per result.

- issues:
  `No PIDs left to attach to the scope's control group` (2 results,
  both closed:
  #24040 and #21297);
  `unit_attach_pids_to_cgroup` (3 closed results:
  #21297,
  #6045 and #6875);
  no results for `scope zombie PIDs StartTransientUnit`,
  `StartTransientUnit exiting process scope`,
  `scope running populated 0`,
  `transient scope never garbage collected`,
  `scope stays active no processes empty cgroup`,
  `scope_enter_running`,
  `AttachProcessesToUnit zombie`,
  `scope empty cgroup notification missed`,
  `cgroup.procs write exiting task success` and
  `PIDs property scope already exited process`.
- pull requests,
  the same twelve queries:
  #24039 (merged,
  a stray `%m` in the refusal message),
  #21309 (closed),
  #42910 (closed),
  #29232 (merged),
  #23639 (merged),
  #21298 (merged),
  #20577 (closed),
  #6932,
  #8125 and #12419 (merged).
  The other queries returned no results.
- read in full:
  pull request #21163 (adds the `No PIDs left` refusal;
  its only comment asks a reporter to test),
  #20577 (the machined symptom and the maintainer's "fix it in pid1" comments),
  #21298 (a regression of the refusal for delegated user-manager attaches),
  #42910 (a stop request during the chown helper),
  and issues #24040 (a logind test that hit the refusal in a race with a `getty` restart,
  closed 2022-07-30 after two comments about the test),
  #21297 (the regression #21298 fixed),
  #6045 and #6875 (2017 reports about processes migrating to a PAM session scope).
  None describes a scope left running by a zombie,
  and none proposes a post-attach `populated` check.
  Nothing there adds to a report beyond this file's reproduction,
  source trace and prototype.
- the exact symptom (a scope `active (running)` with `populated 0` after a successful attach) has no existing issue,
  so a human filing would open a new one.

### Work in progress

Run on 2026-11-01:

- open pull requests matching `scope`,
  `cgroup empty`,
  `transient scope`,
  `StartTransientUnit`,
  `zombie` or `PIDs scope`:
  104 distinct candidates
  (the `scope` search is capped at 100 results,
  so it can miss some);
  seven touch the files named in the root cause
  (#44504,
  #43844,
  #43885,
  #43866,
  #43461,
  #43692,
  #37180);
  their diffs mention the root-cause functions only in unrelated ways
  (#43885 uses `pidref_verify` in a new component,
  #43866 calls `cg_attach` in tests),
  and none changes `scope_enter_running`,
  `unit_attach_pids_to_cgroup` or the empty-cgroup path.
- commits since 2026-10-01 through `gh api repos/systemd/systemd/commits?path=<file>&since=...`:
  on `src/core/scope.c`,
  `ef69c63a7b` (comment style) only;
  on `src/core/cgroup.c`,
  `ef69c63a7b` and `e530bd0368` (an invocation-id helper);
  on `src/basic/pidref.c`,
  `8fed4ce43f` (an array free helper);
  on `src/core/dbus-scope.c`,
  `ef69c63a7b`;
  on `src/basic/pidfd-util.c`,
  none.
  There is recent maintainer activity in `src/core/cgroup.c`,
  but not on this path.
- branches on `systemd/systemd`:
  9,
  none matching `scope|cgroup|zombie|empty|pidref|transient`.
  Forks were not searched.
- claims of work:
  none in the issues and pull requests read;
  #24040 is closed.

### Out-of-scope check

`.out-of-scope/` held 11 files on 2026-11-01
(`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
`typescript-project-references.md`);
`rg --ignore-case --count 'systemd|cgroup'` over the directory printed nothing,
so no exemption applies.

### Risk to the poster

This file makes no recommendation to file.
Filing is the poster's decision,
and the written policy makes it a personal one:
a first violation (text an AI wrote,
or an unreviewed AI post) is a warning,
a second is a ban from the project,
and the account is the poster's.
A reproduction,
a source trace and a prototype lower the risk of a low-effort judgment and do not remove it.

Trust and blocklist search,
repository `systemd/systemd`,
default branch commit
`fad40b4fb1a18925e886c8b1a93e2681591b16a8`,
2026-11-01:

- the repository tree (8503 entries) has no path matching `vouch`,
  `denounce`,
  `blocklist`,
  `allowlist` or `trusted-list`.
  It does have `.github/workflows/*`,
  `.github/CODEOWNERS`,
  `docs/CODE_OF_CONDUCT.md`,
  `AGENTS.md` and `CLAUDE.md`.
- `rg` over all 16 workflow YAML files and `CODEOWNERS` for
  `vouch|denounce|blocklist|blacklist|banned|lock-threads|stale|close-issue|auto-close`
  matched only label-handling variable names in `labeler.yml`.
- the only workflow triggered by `issues:` is `issue-labeler.yml`,
  which labels by the form's Component field;
  `claude-review.yml` triggers on pull request events,
  not on issues.
- not searched:
  organization-level lists outside the repository,
  and forks.

The filer must check the live repository again on the day of posting.

### Readiness check

Run on 2026-11-01 against the commits named in each item.
An item that did not run says so.

1.
    Latest release and other maintained lines:
    reproduced on `262.1-1-arch` (release `v262.1`,
    published 2026-10-16) and on a build of the `v262.1` tag;
    `v259.10` only through the host census (no throwaway run);
    `v261.4`,
    `v260.6` and `v258.12` were not run.
    For `v261.4`,
    `v260.6` and `v259.10` the four functions on the path are text-identical to `v262.1`'s
    (the function-body comparison in "Root cause"),
    which is code evidence,
    not a reproduction.
2.
    Prior art in the touched code:
    done (see "Duplicate and prior-art searches").
    The stated reason for the current refusal is that a scope must not be activated when its PIDs are gone
    (pull request 21163);
    this report extends that principle to a process that is gone but still has a pid.
3.
    Intake:
    read the bug form,
    the template configuration,
    `docs/CONTRIBUTING.md` and its AI policy (constraint 4).
    Fields only the poster can answer:
    none are invented here.
    There is no "I am working on a fix" checkbox in the bug form.
    No pull request template exists (`.github/pull_request_template.md` returned 404 and the tree search found none);
    the policy requires a human to open any pull request.
4.
    Automation on a new filing:
    `issue-labeler.yml` labels by component;
    no trust,
    auto-close or lock workflow was found (search recorded in "Risk to the poster").
5.
    Day-of re-runs:
    the duplicate search,
    the work-in-progress check and the trust search were run on 2026-11-01
    and must be run again on the day of any posting.
    The cited code exists on the default branch (`fad40b4f`,
    2026-11-01).
6.
    The draft's reproduction was run as written:
    the block was extracted from this file by a script (which also checks that the fact sheet in "Draft" contains the
    identical block) and run unmodified in a fresh container;
    the output is recorded in "Draft".
7.
    Runnable by others (a minimal repository with a CI workflow):
    not run.
    Blocker:
    the target's policy forbids an agent posting,
    and the repository would carry the poster's account name and
    need GitHub Actions under the owner's account;
    it is the next step only if the owner decides to report.
8.
    Verdict:
    not ready to file as written,
    and no recommendation to file.
    The blocking item is constraint 4 (the policy requires the poster to write the report),
    which an agent cannot clear.
    What clears it:
    the owner writes the issue.

### Draft

Kept as a record and a fact sheet;
marked do not file as-is.
The reproduction block was extracted from this file and run unmodified in a fresh container.

```sh
# draft-reproduction: as root on a booted systemd system (tested: 262.1, system manager, kernel 7.2.11)
sh -c 'sleep 0.1 & exec sleep 60' >/dev/null 2>&1 & parent=$!
sleep 0.5
zombie=$(ps --ppid "$parent" -o pid= | tr -d ' ')
ps -o pid,ppid,stat,comm -p "$zombie"
busctl call org.freedesktop.systemd1 /org/freedesktop/systemd1 org.freedesktop.systemd1.Manager \
  StartTransientUnit 'ssa(sv)a(sa(sv))' repro.scope fail 2 PIDs au 1 "$zombie" Description s repro 0
kill "$parent"; sleep 1.5
systemctl show repro.scope -p ActiveState -p SubState -p Result
cat /sys/fs/cgroup/system.slice/repro.scope/cgroup.events
```

Output of that block in a fresh `localhost/systemd-scope-probe:run` container (2026-11-01):

```text
    PID    PPID STAT COMMAND
     99      97 Z    sleep <defunct>
o "/org/freedesktop/systemd1/job/262"
ActiveState=active
SubState=running
Result=success
populated 0
frozen 0
```

~~~md
Do not file as-is: an AI agent wrote this fact sheet, and the project's policy requires the poster to write the issue.

Title: A transient scope started with PIDs naming an exiting process stays active (running) forever with an empty cgroup

Labels: set by the bug form

systemd version: 262.1 (the host runs 259.10-1.fc44; the functions on the path are identical in 259.10, 260.6,
261.4 and 262.1)
Distribution and kernel: CachyOS container with systemd 262.1-1 as PID 1 on kernel 7.2.11-ogc4.1.fc44.x86_64
Architecture: x86_64
Component: systemd

Expected: the scope ends when its processes are gone, as it does when a process exits right after being attached.

Seen: StartTransientUnit with PIDs naming a process that is already a zombie succeeds. The scope is
active (running), cgroup.events says "populated 0", cgroup.procs is empty, and the scope never ends. The debug
journal says "1 process added to scope's control group."

Cause: cg_attach() writes the pid to cgroup.procs. The kernel returns success without moving a task that has
PF_EXITING set (kernel/cgroup/cgroup.c, cgroup_migrate_add_task). unit_attach_pids_to_cgroup() counts the write as
an attached process, and scope_enter_running() sets SCOPE_RUNNING and calls unit_unwatch_all_pids() ("We rely on
empty notifications there."). The cgroup never becomes populated, so cgroup.events never changes and the inotify
watch never fires. unit_check_cgroup_events() is the only reader of "populated" and runs only from the inotify
handler and from unit_cgroup_catchup() (reload and re-execution), so "systemctl daemon-reload" ends such scopes.

Reproduction (root, system manager):

```sh
# draft-reproduction: as root on a booted systemd system (tested: 262.1, system manager, kernel 7.2.11)
sh -c 'sleep 0.1 & exec sleep 60' >/dev/null 2>&1 & parent=$!
sleep 0.5
zombie=$(ps --ppid "$parent" -o pid= | tr -d ' ')
ps -o pid,ppid,stat,comm -p "$zombie"
busctl call org.freedesktop.systemd1 /org/freedesktop/systemd1 org.freedesktop.systemd1.Manager \
  StartTransientUnit 'ssa(sv)a(sa(sv))' repro.scope fail 2 PIDs au 1 "$zombie" Description s repro 0
kill "$parent"; sleep 1.5
systemctl show repro.scope -p ActiveState -p SubState -p Result
cat /sys/fs/cgroup/system.slice/repro.scope/cgroup.events
```

Observed: ActiveState=active, SubState=running, Result=success, populated 0.

Suggested fix: after unit_unwatch_all_pids() in scope_enter_running() (src/core/scope.c), check once whether the
cgroup is populated and queue the empty handling if it is not, for example by calling unit_cgroup_catchup(u). Tested
on a v262.1 build: the reproduction then shows the scope ended and collected, a live process still keeps it
running, and the "No PIDs left" path for a process reaped before the start job is unchanged.

Related: a process reaped between StartTransientUnit and the start job fails the scope with "No PIDs left to attach
to the scope's control group, refusing." (added by commit 8d3e4ac7cd37, pull request 21163). That is correct and
unchanged.
~~~
