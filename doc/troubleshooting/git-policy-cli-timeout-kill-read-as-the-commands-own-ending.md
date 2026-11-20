# A bounded line's own `timeout --signal=KILL` death is read as the command ending by itself

GNU coreutils `timeout` 9.12,
which the wrapper's test harness puts in front of every command as that command's bound,
dies by `SIGKILL` itself when its timer fires.
`wait_under_bound` (`package/git-policy/cli/src/native/binary/binary_support_observation_bound.rs`)
answered that death as the command's own ending whenever the exit reached the wait before the wait's
deadline ran out,
so a control read `code: None` with no bound verdict instead of the verdict it asserts (issue 6324).
What host load does to the same harness bound is
[native binary tests hit the shared bound under host load](native-binary-tests-hit-the-shared-bound-under-host-load.md).

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

`src/timeout.c` at commit `24a027b103704f6c11079e15917aac4a88a54c50`,
a commit on master that is 312 commits past the `v9.12` tag
(`gh api repos/coreutils/coreutils/compare/v9.12...24a027b103704f6c11079e15917aac4a88a54c50`),
which is the version the host's `timeout --version` answers
(`timeout (GNU coreutils) 9.12`).
Every quote below is that commit's,
whose git blob hash is `e2d1ccdcafa892fb93316859ef9271499f2fc25b`;
the `v9.12` tag's own `src/timeout.c` is a different blob
(`94d2c41cb2f0f72d3cc815bbb297c821182df037`),
whose `send_sig` says the same thing in three lines
(`if (where == 0) signal (sig, SIG_IGN); return kill (where, sig);`),
and whose group kill,
its ignore of the signal,
and its re-raise behave the same.
The two files differ in ptrace handling,
in the order `cleanup` sends signals and prints diagnostics,
and in `send_sig`'s shape,
and no conclusion below rests on those.
The harness names `/usr/bin/timeout` by its absolute path,
so the `timeout-cli` that can come first on `PATH` never runs here
([timeout-cli consumes flags right after the command](timeout-cli-consumes-flags-right-after-command.md)).

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
and this wait's deadline is the earlier of the two:
the start is floored to a whole clock tick,
and the line's `timeout` arms its timer only after its own `exec`,
so the line's bound is reached a tick and a start-up later.
So a `timeout` line that never answers is normally ended by this wait's own kill.
When the waiting thread is scheduled late,
both the deadline and the exit are ready at once,
and the standard library's `recv_timeout` answers a message that is already in hand
instead of reporting the timeout
(a message queued before the call is returned by `recv_timeout(Duration::ZERO)`,
and by `recv_timeout(Duration::from_millis(1))` whose deadline is already spent;
the program under "Verification" prints both,
and reports the timeout only when the channel is empty).
That arm then reported the line's own `timeout` death as the command's own,
with no exit code and no verdict.

## Verification

Versions:
GNU coreutils 9.12 (`timeout (GNU coreutils) 9.12`),
whose source was read at the master commit `24a027b103704f6c11079e15917aac4a88a54c50`,
and `package/git-policy/cli` at `8d07db9940`,
which carries the red control,
and `338bf97cf6`,
which carries the fix.

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

The standard library's `recv_timeout` is checked by a program of its own,
which queues a message before it asks
(`/usr/bin/rustc --edition 2021 recv-timeout-queued-message.rs --out-dir . && ./recv_timeout_queued_message`):

```rust
// What: whether std's mpsc `recv_timeout` answers a queued message once its deadline has passed.
use std::sync::mpsc;
use std::time::Duration;

fn main() {
    // A message queued before the call, with no time left: what a wait reached after its deadline sees.
    let (sender, receiver) = mpsc::channel::<u8>();
    sender.send(1).expect("send");
    std::thread::sleep(Duration::from_millis(50));
    println!("zero timeout, message queued: {:?}", receiver.recv_timeout(Duration::ZERO).is_ok());

    // A message queued before the call, with a deadline a millisecond away and already spent.
    let (sender, receiver) = mpsc::channel::<u8>();
    sender.send(1).expect("send");
    std::thread::sleep(Duration::from_millis(50));
    let began = std::time::Instant::now();
    let seen = receiver.recv_timeout(Duration::from_millis(1));
    println!("1ms timeout, message queued: ok={:?} after {:?}", seen.is_ok(), began.elapsed());

    // An empty channel whose deadline has passed: the timeout arm.
    let (_sender, receiver) = mpsc::channel::<u8>();
    println!("1ms timeout, empty: {:?}", receiver.recv_timeout(Duration::from_millis(1)).is_err());
}
```

```text
zero timeout, message queued: true
1ms timeout, message queued: ok=true after 2.41µs
1ms timeout, empty: true
```

The first two lines are the queued message winning over a deadline that had already passed.
The third is the positive control:
the same program does report a timeout when the channel is empty,
so the first two lines say what the channel held and not that the call cannot time out.
The duration on the second line is scheduling noise.

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
the wait is told the two bounds one reading of the command's own line gives (`line_bound_of`),
and an exit read in the `Ok` arm counts as the bound's when that line is the harness's bounded form,
the status is death by `SIGKILL`,
and that line's own bound has been reached by this wait's clock
(`ending_of_an_exit_read_first`).
Tradeoff:
the status alone cannot say who sent a `SIGKILL`,
so the rule leans on the clock:
a command that kills itself with `SIGKILL` at or after its line's bound is read as the bound's,
which is how a late-scheduled `timeout` death is caught and also how a command's own late death is misread,
and the truncation of the command's start to whole clock ticks widens "reached" upward by less than one tick.
That window is as wide as the wait's lateness past its line's bound,
and a wait that is on time never enters it.
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
  as `/usr/bin/timeout --signal=KILL 5 /bin/sh -c 'kill -KILL $$'` shows.
- Waiting for the deadline to decide first.
  A late-scheduled thread reads a message already in hand instead of the timeout,
  and `recv_timeout(0)` answers the same way,
  so the arm that reads the exit cannot be avoided.
- Reading the bounded form back from the child's `/proc/<pid>/cmdline`.
  Between `fork` and `execve` that file still shows the parent's command line,
  so the line's bound cannot be read there without racing the command's own start.
- Trusting `--foreground`'s exit status 137 as the harness's ending.
  It is an exit code,
  and the wrapper's bounded form keeps the group kill,
  because the group is what ends a command's descendants.

## Upstream filing decision

Nothing is filed,
and nothing is added to an upstream thread.

1.  Is it really upstream's fault:
    no.
    `timeout` documenting and implementing "send the signal to the command's group,
    and die by it" is behaviour,
    and the group kill is the reason the program exists in that shape.
    The defect is this repository's classification of the ending.
2.  Can upstream fix it:
    not applicable,
    since there is nothing to fix there.
    The `SIGKILL` cannot be caught,
    and `send_sig`'s own comment explains why the group signal must be sent.
3.  Are they supporting this use case:
    the manual documents the group semantics under `--foreground`,
    which is the supported way to avoid the group signal.
4.  Would the repository welcome our contribution:
    not checked,
    because constraints 1 to 3 already stop a filing.
5.  Will they likely fix it:
    not applicable.
6.  Have we prototyped a minimal fix compatible with their architecture:
    not applicable,
    and the fix for the observed failure is in this repository
    (`ending_of_an_exit_read_first` in `binary_support_observation_bound.rs`).

No duplicate search,
trust-list search or reproduction repository was made,
because the filing decision was reached at constraint 1.
