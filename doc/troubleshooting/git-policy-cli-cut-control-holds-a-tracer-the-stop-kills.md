# strace 7.3 detaches a tracee when its tracer dies, so a stop's kill ended the cut a control held it in

`a_watched_kill_between_the_pid_file_and_the_lock_leaves_the_lock_it_cannot_prove` in
`package/git-policy/cli/src/native/lock/git_stop_lock_tests.rs` was red on `main` when run alone,
and kept passing or failing at random on the branch that starts at `5686c7bb29`,
whose copy of that file is the same as `main`'s (`git diff main..HEAD -- <the file>` is empty).

## Symptom

Run `wrapper-issues-655-1135-1` (2026-11-10) recorded,
at `5686c7bb29`'s lineage:

```text
thread 'lock::git_stop_lock_tests::a_watched_kill_between_the_pid_file_and_the_lock_leaves_the_lock_it_cannot_prove' (1585230) panicked at src/native/lock/git_stop_lock_tests.rs:478:9:
watched step HeldPastTheStopGrace: no lock was left, so this control no longer tests a lock nothing proves gone
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6211 filtered out; finished in 21.93s
```
A run of the same control on this machine,
`mise run //package/git-policy/cli:native:test:host -- a_watched_kill_between_the_pid_file_and_the_lock_leaves_the_lock_it_cannot_prove`,
failed once in 33 runs with the same line in 8.351 s;
the other 32 runs passed.

The failure is the control's own checkpoint:
it found no `.lock` file at all under the fixture's Git directory,
which is what a holder that cleaned up after itself leaves,
not what a kill between Git's two removals leaves.

## What the control held

The holder is Git's real `update-ref --stdin` told to `prepare`,
so it holds the lock of
`refs/remotes/origin/main` and the pid file `refs/remotes/origin/main~pid.lock` beside it
(`core.lockfilePid` is on),
started under

```text
strace -f -o /dev/null -e trace=unlink -e inject=unlink:delay_exit=10000000:when=1 /usr/bin/git update-ref --stdin
```

Git's `SIGTERM` handler removes the pid file and then the lock,
130 µs apart on a quiet disk (`doc/troubleshooting/git-policy-cli-stop-lock-controls-wait-on-order-not-time.md`),
so the injected delay is what keeps Git inside that cut for 10 s,
and the watched stop's kill at the end of its 5 s grace (`STOP_GRACE`) is what should land inside it.

Two facts about where the hold really sits were not checked when the control was written.

## Root cause

### 1. The hold is in the tracer, and the stop kills the tracer

The delay runs in the `strace` process,
and the tracee waits for it.
The watched stop signals every process below the step
(`package/git-policy/cli/src/native/push/child_watch_clock_marks_and_stopping.rs`,
`stop_process`,
`terminate_members` then `kill_members`),
and `strace` is one of them.
The manual names the option that keeps a tracee from outliving its tracer
([strace.1.in][strace-1-kill-on-exit]):

> Applies the PTRACE_O_EXITKILL ptrace option to all tracees,
> which sends a SIGKILL signal to a tracee if the tracer exits.
> This prevents tracees from being left running after the tracer exits,
> as they will not be detached on cleanup.

It also says what a traced command's descendants do when nothing holds them
([strace.1.in][strace-1-left-running]):

> When tracing a command,
> its descendant processes may be left running after `strace` is terminated by an
> interrupt signal (such as CTRL-C ).
> This can be prevented by using the `--kill-on-exit` option

Measured on this machine for the signal this stop sends,
`SIGKILL`:
`after_two_seconds_after_killing_the_tracer=yes` without the option and `=no` with it.

So when the stop's `SIGKILL` reaches the `strace` and not (or before) its tracee,
Git resumes the handler it was stopped in,
removes the lock as well,
and the control finds nothing at all.
The `doc/troubleshooting/git-policy-cli-stop-lock-controls-wait-on-order-not-time.md` record already measured the
same shape from the tree read's side:
"Git was forked by `strace` after that read,
so `kill_members` never signalled it,
and it went on to finish its cleanup 5.9 s after the push had been reported stopped".

### 2. `when=1` is the tracee's own first `unlink`, not the pid file's

`-e inject=unlink:delay_exit=…:when=1` holds the *first* `unlink` of each traced process:

> Accounting of system calls that are subject to injection is done per system call and per tracee.

([strace.1.in][strace-1-per-tracee]).

`git update-ref --stdin` makes no other `unlink` before the handler,
so the hold happened to land on the pid file here.
A Git that unlinks anything first (a commit made 14 `unlink` calls in one probe) spends the hold elsewhere,
and the control would then be a kill at an unrelated moment.
The control's name claims the pid file's removal,
and nothing in its arrangement said so.

### 3. The checkpoint took any leftover file for the ordering

`assert_after_stop` asked only that some `.lock` file was left.
A kill before Git's handler ran leaves the lock *and* its pid file,
which is a different ordering,
and a handler that finished leaves neither,
which the control reported as "no lock was left".

## Verification

strace 7.3 (the host's `/usr/bin/strace`,
and the gate image's current pacman package),
Git 2.56.0,
Python 3.14 as the tracee where a probe needs only a syscall.

### A tracer's death lets its tracee finish

```sh
cd "${HOME}/temp/agent/git-stop-lock/toy" && touch a
strace -f -o t.trace -e trace=unlink -e inject=unlink:delay_exit=8s:when=1 -- \
  python3 -c 'import os; os.unlink("a"); open("after","w").close()' &
sleep 1; kill -KILL $!; sleep 2
[ -e after ] && echo "after exists"
```

Prints `after exists`:
the tracee resumed and finished the work the hold was in.
The same command with `--kill-on-exit` before the `-e` options prints nothing,
because the tracee dies with its tracer.

### `when=1` counts per tracee

```sh
touch a b
strace -f -o /dev/null -e trace=unlink -e inject=unlink:error=EACCES:when=1 -- \
  sh -c 'python3 -c "import os; os.unlink(\"a\")"; python3 -c "import os; os.unlink(\"b\")"'
```

Both children raise `PermissionError`,
so each one's own first `unlink` was injected.

### `-P` aims an injection at one path

```sh
strace -f -o t.trace -P "$PWD/x" -e trace=unlink,unlinkat \
  -e inject=unlink:delay_exit=3s:when=1 -e inject=unlinkat:delay_exit=3s:when=1 -- \
  python3 -c 'import os; open("x","w").close(); os.unlink("x"); open("after","w").close()'
```

Takes 3.027 s and leaves `after`,
and the trace holds one line,
`unlink("/…/x") = 0 (DELAYED)`,
for a file that did not exist when `strace` started.
The manual documents the combination ([strace.1.in][strace-1-with-path]).

### The control's hold after the fix

The same control with its holder's trace kept (`-tt -T -o <file>`) prints one traced syscall:

```text
1972399 21:26:48.894421 --- SIGTERM {si_signo=SIGTERM, si_code=SI_USER, si_pid=1965000, si_uid=1000} ---
1972399 21:26:48.894552 unlink("/…/repo/.git/refs/remotes/origin/main~pid.lock") = 0 (DELAYED) <0.000088>
```

131 µs after the request,
on the pid file and nothing else,
which is the ordering the control's name claims.

## Verified workarounds

The holder's command became
(`held_before_the_pid_file_goes` in
`package/git-policy/cli/src/native/lock/git_stop_lock_tests.rs`):

```text
strace -f -o /dev/null --kill-on-exit -P "$PID_FILE" -e trace=unlink,unlinkat \
  -e inject=unlink:delay_exit=<µs>:when=1 -e inject=unlinkat:delay_exit=<µs>:when=1 /usr/bin/git
```

- `-P "$PID_FILE"` puts the injection on the pid file's removal only.
It also narrows what is traced,
  which costs nothing here (`-o /dev/null`).
Tradeoff:
  the path is matched as the tracee spells it,
  so a Git that named the file differently would
get no hold at all,
  and the control would fail naming the file instead of proving the cut.
- `--kill-on-exit` makes the tracer end its tracee,
  so a stop that kills the tracer cannot undo the cut.
Tradeoff:
  none for a control whose holder is meant to die;
  it rules out attaching to a process a person owns,
which this fixture never does.
- The checkpoint now reads both files.

## A machine without the tracer: the control blamed the holder's speed (issue 6460)

The three controls of `src/native/lock/git_stop_lock_tests.rs` that arrange the cut were red on this host,
alone,
each in
its own process,
and green in the container gate.
This host installs no `strace`,
and the holder's shell named it bare,
so Git never started and no look could ever see the lock.

### The symptom

At `49029c6828` (the tip of `main` this record's branch starts from),
a worktree of its own,
one control per process
(`keep/red-four-controls-alone-at-the-pre-fix-commit.log` of the round
`lock-kill-controls-that-fail-alone-on-this-host-2026-11-25`):

- `a_kill_between_the_pid_file_and_the_lock_leaves_the_lock_it_cannot_prove` failed after 18.18 s with
  `bounded run SlowToFinishCleanup: no look saw the lock and its pid file within the hold of 18.095958s, so the stop was
  asked before the holder held the lock`;
- `a_push_stop_of_a_holder_slow_to_finish_its_cleanup_leaves_no_lock` failed after 18.17 s with the same line;
- `a_watched_kill_between_the_pid_file_and_the_lock_leaves_the_lock_it_cannot_prove` failed after 600.15 s with
  `watched step HeldPastTheStopGrace: the holder never announced that it held the lock, so the stop was asked before the
  lock existed`.

Both failure lines named the control's own ordering,
and neither named the program that was missing.

### What this host does differently

`pacman -Q strace` answers `error: package 'strace' was not found`,
`/usr/bin/strace` does not exist,
and
`command -v strace` under the fixture's own `PATH` (`/usr/bin:/bin:/usr/sbin`) finds nothing
(`keep/probe-the-fixture-path-without-a-tracer.log`).
The gate's image installs it:
`bin/build-git-test-image.ts` lists
it among the tools the tests start.

### The cause

Both `Holder` kinds that hold a removal back,
`SlowToFinishCleanup` and `HeldPastTheStopGrace`,
are Git wrapped in the
tracer (`held_before_the_pid_file_goes`),
and that string spelled the program `strace`.
The holder's shell runs with the
fixture's own `PATH`,
which holds no such program here,
so the shell answered

```text
sh: line 1: strace: command not found
```

and exited 127 before Git started (measured with that `PATH` alone in the environment,
`keep/probe-the-fixture-path-without-a-tracer.log`).
Nothing then wrote `refs/remotes/origin/main.lock` or its pid
file,
so `the_holder_holds_its_lock` was false at every look.

The two bounded controls fail at their announcement hold:
`ANNOUNCEMENT_HOLD` is `STOP_CEILING`,
which is
`LONGEST_DELAY_SEEN` (6031.986 ms) times the owner's margin of 3,
so 18.095958 s
(`src/native/foundation/worst_delay_seen_on_this_host.rs`),
the same number their line printed.

The watched control has no announcement hold and is slower for a second reason.
Its holder's shell waits for the lock in
a loop that starts a `sleep` every 50 ms,
and a step that starts processes reads as work,
so the watched stop's quiet
window never fired;
only the step's own ceiling ended it,
`OBJECT_STEP_CEILING` of ten minutes
(`src/native/service/wait_ceilings.rs`),
which is the 600.15 s.

### The measurement that shows the tracer is the cause

With a copy of the gate image's `strace` (7.3,
taken from a layer of that image as issue 6459's round did) on the test
process's own `PATH`,
the same three controls at the same source commit passed in 4.62 s,
7.62 s and 8.14 s
(`keep/green-lock-controls-alone-with-a-tracer.log`).
Nothing else changed:
the copy is the only difference between the
red and the green arm.

### The fix

- `src/native/testing/strace_the_controls_run_under.rs`,
  moved from the binary controls' own `binary_strace_support.rs`,
is the one copy of the answer both test crates reach (`src/native/binary/binary_tests.rs` includes it by path,
  as it
includes `real_git_lfs_the_run_requires.rs`).
- The holder runs the absolute path that answer gives,
  so the fixture's own `PATH` no longer decides whether the cut can
be made.
- A control whose machine has no tracer prints one line naming itself and passes,
  as the btrfs and real-tool controls
do,
  and `--success-output=final` brings that line into a green gate's log:

```text
STRACE CONTROL NOT RUN: stop-lock-push-slow-cleanup (no strace on PATH)
```

- A run that must have the tracer fails instead of passing with controls unchecked:
  `CLI_GIT_REQUIRE_STRACE`,
  which the
gate image sets (`bin/native-test-containerfile.ts`),
  names the variable and the program.

The three controls passed here in 0.00 s each,
printing that line
(`keep/green-lock-controls-alone-without-a-tracer.log`).

### What is not claimed

- No production code runs `strace`;
  it is a control's tool,
  so nothing a person runs changed.
- The host's AppArmor records need no reading for this cause:
  no denial is involved,
  and the checker's windows for the
runs above are in `keep/windows.txt` in case a record of those minutes is wanted for another reason.
- The 600 s wait of the watched control is not a product defect of its own:
  it needs a command that keeps starting
processes while it never takes the lock,
  which is the shape of this control's waiting loop when the program it pipes to
is missing.

## What does not work

- `-P` given the canonical path of a file reached through a symbolic link.
`strace` resolves its own `-P` argument,
  and the tracee's own spelling is what matches:
`-P /var/mnt/encrypted/…/c` traced nothing while the tracee unlinked `/home/user/…/c`,
and `-P /home/user/…/c` matched.
The fixture root is canonicalized,
  so the control passes the same spelling on both sides.
- Trusting the stop to kill the tracee.
The stop reads the process tree once,
  before it signals,
and the earlier record in
`doc/troubleshooting/git-policy-cli-stop-lock-controls-wait-on-order-not-time.md` measured a tracee that was never
signalled and finished its cleanup after the stop had returned.
- `-e inject=…:when=<n>` with an ordinal read off one Git version.
The ordinal is the tracee's own count and the command's `unlink` count is Git's business.

## What is not claimed

- The bounded-path cut control,
  `a_kill_between_the_pid_file_and_the_lock_leaves_the_lock_it_cannot_prove`,
still passes when the sweep's refusal is planted away,
because the sweep of a push whose child was not collected runs as a task
(`package/git-policy/cli/src/native/push/stop_of_a_push_and_sweep_of_its_locks.rs`,
  `settle_locks`),
after the control's checkpoint.
The watched control's sweep runs inside the stop (`stop_process`),
  which is why it catches that plant.
- Nothing here says the stop should re-read its process tree;
  that would be a production change of its own,
and the earlier record already declined to claim it.

## Upstream filing decision

Nothing to file.

1. Is it upstream's fault?
   No:
   `--inject`,
   `when`,
   `-P` and `--kill-on-exit` behave as the manual describes,
   and the arrangement relied on two of them in ways their documentation contradicts.
2. Can upstream fix it?
   There is nothing to fix;
   a tracee that outlives its tracer is ptrace's semantics until `PTRACE_O_EXITKILL`,
   which `--kill-on-exit` sets.
3. Is the use case supported?
   Yes:
   the manual documents `-P` with `--inject`,
   and `--kill-on-exit` for exactly this.
4. Would the repo welcome a report?
   Not reached:
   1 and 2 already decide it.
5. Duplicate?
   Not searched,
   for the same reason.
6. Were one promised,
   could we run it?
   Nothing to promise.

[strace-1-per-tracee]: https://github.com/strace/strace/blob/v7.3/doc/strace.1.in#L1746-L1747
[strace-1-with-path]: https://github.com/strace/strace/blob/v7.3/doc/strace.1.in#L1749-L1752
[strace-1-kill-on-exit]: https://github.com/strace/strace/blob/v7.3/doc/strace.1.in#L523-L529
[strace-1-left-running]: https://github.com/strace/strace/blob/v7.3/doc/strace.1.in#L2113-L2121
