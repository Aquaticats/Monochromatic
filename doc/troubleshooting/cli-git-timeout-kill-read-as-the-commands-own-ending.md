# A bounded line's own `timeout --signal=KILL` death is read as the command ending by itself

GNU coreutils `timeout` 9.12,
which the wrapper's test harness puts in front of every command as that command's bound,
dies by `SIGKILL` itself when its timer fires.
`wait_under_bound` (`package/git-policy/cli/src/native/binary/binary_support_observation_bound.rs`)
answered that death as the command's own ending whenever the exit reached the wait before the wait's
deadline ran out,
so a control read `code: None` with no bound verdict instead of the verdict it asserts (issue 6324).

## Symptom

One control in a loaded container gate failed,
and the same control passed alone in the same image:

```text
FAIL [   1.228s] monochromatic-git-policy-cli::native_binary resolution::a_version_query_that_never_answers_ends_at_its_bound
panicked at src/native/binary/binary_resolution_tests.rs:54:5:
assertion `left == right` failed: the `git --version` query failed: ""
  left: None
 right: Some(0)
```

The control runs `/usr/bin/timeout --signal=KILL 1 <a script that sleeps 5 s> --version` through
`observe` and expects the wait to end it and panic with a verdict naming `1s wall-clock bound`.
The observed answer instead was an ordinary observation with no exit code and empty standard error,
which is what `Ending::Itself` with a `SIGKILL` status looks like to the control.

## Root cause

Two facts meet,
one in coreutils and one in this repository's wait.

### coreutils `timeout` kills its own group and dies with it

`src/timeout.c` at commit `24a027b103704f6c11079e15917aac4a88a54c50`
(the file the host's `timeout --version` answers `timeout (GNU coreutils) 9.12` for;
the copy read here is byte-identical to that blob,
whose git blob hash is `e2d1ccdcafa892fb93316859ef9271499f2fc25b`):

Without `--foreground`,
the program puts itself in a process group of its own before it starts the command
([`src/timeout.c#L644-L652`](https://github.com/coreutils/coreutils/blob/24a027b103704f6c11079e15917aac4a88a54c50/src/timeout.c#L644-L652)):

```c
  if (!foreground)
    setpgid (0, 0);
```

When its timer fires,
it signals the monitored command and then the whole group,
which includes itself
([`src/timeout.c#L278-L292`](https://github.com/coreutils/coreutils/blob/24a027b103704f6c11079e15917aac4a88a54c50/src/timeout.c#L278-L292)):

```c
      send_sig (monitored_pid, sig);

      /* The normal case is the job has remained in our
         newly created process group, so send to all processes in that.  */
      if (!foreground)
        {
          send_sig (0, sig);
```

`send_sig` ignores the signal in itself first so a group signal cannot loop
([`src/timeout.c#L211-L241`](https://github.com/coreutils/coreutils/blob/24a027b103704f6c11079e15917aac4a88a54c50/src/timeout.c#L211-L241)):

```c
  bool ignore = where == 0 && sig != SIGCHLD;
  if (ignore)
    {
      sigaction (sig, NULL, &old_sa);
      signal (sig, SIG_IGN);
    }
```

`SIGKILL` cannot be ignored,
so with `--signal=KILL` the program dies by the signal it sent,
and the process the harness waits for has a death by `SIGKILL` and no exit code.

The same `SIGKILL` status also arrives when the command inside killed itself:
when the command died by a signal and the timer had not fired,
`timeout` re-raises that signal on itself
([`src/timeout.c#L746-L760`](https://github.com/coreutils/coreutils/blob/24a027b103704f6c11079e15917aac4a88a54c50/src/timeout.c#L746-L760)):

```c
              if (!timed_out && disable_core_dumps ())
                {
                  /* exit with the signal flag set.  */
                  signal (sig, SIG_DFL);
                  unblock_signal (sig);
                  raise (sig);
                }
```

So the status alone cannot say who ended the command,
and that is why the harness cannot classify an ending by the status alone either.

### The wait read the exit in the arm that meant "the command ended by itself"

`wait_under_bound` counted from the command's own start
(`remaining_of`,
reading `/proc/<pid>/stat`),
and waited with `recv_timeout(remaining)` for a waiter thread that reports the exit without reaping it.
The `Ok(Ok(()))` arm meant "the exit arrived first" and answered `Ending::Itself` unconditionally.
Only the `Err(Timeout)` arm looked at the status,
and only a `SIGKILL` death found there counted as `Ending::AtTheBound`.

Both timers keep one value from the command's start,
and this wait's deadline is the earlier of the two by up to one clock tick,
so a `timeout` line that never answers is normally ended by this wait's own kill.
When the waiting thread is scheduled late,
both the deadline and the exit are ready at once,
and the standard library's `recv_timeout` answers a message that is already in hand
instead of reporting the timeout
(measured:
a message queued before the call is returned by `recv_timeout(Duration::ZERO)`,
and by `recv_timeout(Duration::from_millis(1))` in 2.76 µs against a deadline already spent).
That arm then reported the line's own `timeout` death as the command's own,
with no exit code and no verdict.

## Verification

Versions:
GNU coreutils 9.12 (`timeout (GNU coreutils) 9.12`),
at upstream commit `24a027b103704f6c11079e15917aac4a88a54c50`,
and the wrapper at the commits this doc was written with.

A shell harness shows the three endings,
with the return code Python's `subprocess` reports
(`-9` is death by signal 9,
not an exit status):

```sh
python3 -c '
import subprocess
cases = [
  ("its own timer fires", ["/usr/bin/timeout","--signal=KILL","1","/bin/sleep","5"]),
  ("the command kills itself", ["/usr/bin/timeout","--signal=KILL","5","/bin/sh","-c","kill -KILL $$"]),
  ("foreground, its own timer fires", ["/usr/bin/timeout","--foreground","--signal=KILL","1","/bin/sleep","5"]),
]
for name, argv in cases:
    done = subprocess.run(argv)
    print(f"{name}: returncode={done.returncode}")
'
```

```text
its own timer fires: returncode=-9
the command kills itself: returncode=-9
foreground, its own timer fires: returncode=137
```

Patterns that work cleanly:

- `/usr/bin/timeout --signal=KILL 1 /bin/true` ends with status 0,
  the command's own.
- `/usr/bin/timeout --foreground --signal=KILL 1 /bin/sleep 5` ends with exit status 137,
  because `preserve_status` keeps the signal's death as an exit code when the program is not in the group it signals.

Patterns that lose the distinction:

- `/usr/bin/timeout --signal=KILL 1 /bin/sleep 5` is a death by `SIGKILL` with no exit code.
- `/usr/bin/timeout --signal=KILL 5 /bin/sh -c 'kill -KILL $$'` is the same death by `SIGKILL` with no exit code,
  though the command chose it a moment after it started.

The repository's own harness is the second catalog:
`support_observation::tests::a_death_by_the_lines_own_timeout_at_its_bound_is_the_bounds_ending`
and `support_observation::tests::a_command_that_kills_itself_before_its_bound_keeps_its_own_ending`
(`package/git-policy/cli/src/native/binary/binary_support_observation_tests.rs`) run both patterns through
`wait_under_bound` itself,
the first given a bound longer than the one its line states so the line's timer fires while the wait is still waiting,
which is the order a late-scheduled wait sees.
Before the fix the first control failed as `left: Itself, right: AtTheBound` after 1.001 s
with `status: signal: 9 (SIGKILL)`,
and after it both pass.

## Verified workarounds

The fix that shipped in this repository:
the wait is told the bound the command's own line states (`stated_line_bound`),
and an exit read in the `Ok` arm counts as the bound's when that line is the harness's bounded form,
the status is death by `SIGKILL`,
and that line's own bound has been reached by this wait's clock
(`ending_of_an_exit_read_first`).
Tradeoff:
the truncation of the command's start to whole clock ticks biases "reached" upward by less than one tick,
so a command that kills itself with `SIGKILL` within one tick of its line's bound is read as the bound's.
Anything else needs a source of the ending other than the status.

For a consumer that cannot change the wait,
running the bounded line with `--foreground` turns the death into an exit status 137,
which a status check can tell from an exit status 0.
Tradeoff:
`--foreground` stops signalling the whole group,
so a command whose descendants outlive it is no longer ended with it,
and the harness relies on the group kill (`Ending::AtTheBound` ends the group the command leads).

## What does not work

- Deciding the ending from the status alone.
  Both the bound's `SIGKILL` and a command's own `SIGKILL` reach the parent as signal 9,
  as the second catalog row shows.
- Waiting for the deadline to decide first.
  A late-scheduled thread reads a message already in hand instead of the timeout,
  and `recv_timeout(0)` answers the same way,
  so the arm that reads the exit cannot be avoided.
- Reading the bounded form back from the child's `/proc/<pid>/cmdline`.
  Between `fork` and `execve` that file still shows the parent's command line,
  so the line's bound cannot be read there without racing the command's own start.
- Trusting `--foreground`'s exit status 137 as the harness's ending.
  It is an exit code,
  and the wrapper's bounded form keeps the group kill for the reason above.

## Upstream filing decision

Nothing is filed,
and nothing is added to an upstream thread.

1. Is it really upstream's fault:
   no.
   `timeout` documenting and implementing "send the signal to the command's group,
   and die by it" is behaviour,
   and the group kill is the reason the program exists in that shape.
   The defect is this repository's classification of the ending.
2. Can upstream fix it:
   not applicable,
   since there is nothing to fix there.
   The `SIGKILL` cannot be caught,
   and `send_sig`'s own comment explains why the group signal must be sent.
3. Are they supporting this use case:
   the manual documents the group semantics under `--foreground`,
   which is the supported way to avoid the group signal.
4. Would the repository welcome our contribution:
   not checked,
   because constraints 1 to 3 already stop a filing.
5. Will they likely fix it:
   not applicable.
6. Have we prototyped a minimal fix compatible with their architecture:
   not applicable,
   and the fix for the observed failure is in this repository
   (`ending_of_an_exit_read_first` in `binary_support_observation_bound.rs`).

No duplicate search,
trust-list search or reproduction repository was made,
because the filing decision was reached at constraint 1.
