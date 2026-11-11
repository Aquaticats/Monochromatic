# A long command line of an unrelated program refused every settle command

The look at the earlier helper could not prove one program of the user,
so
`service settle-earlier-batches` refused,
and running it again cleared nothing while that program ran.

## Symptom

`git monochromatic-git-policy-cli service settle-earlier-batches` refused,
and the only reason it gave was:

```text
the sockets and processes of the earlier helper could not be looked at in time,
so it is not known whether one runs.
```

Running the command again did not clear it,
and nothing the person could do cleared it,
for as long as one unrelated program on the machine kept running.

The three controls that expect the command to act failed in a suite run and passed alone:

```text
FAIL [64.960s] service::the_settle_command_finishes_a_batch_the_earlier_helper_left_open
FAIL [62.652s] service::the_settle_command_is_not_held_by_a_service_another_control_is_starting
FAIL [~63s]    service::the_settle_command_names_a_batch_it_could_not_examine_and_exits_nonzero
```

That run of 2026-11-10 was 197 tests at a host load average of 68 on 8 threads
(`doc/planning/git-policy-cli-full-tokio.md`,
"the three binary failures of the first run are one group"),
and the same group is recorded three times before it as a suite-only failure whose cause was not measured
(`doc/handover/cli-git-rust-implementation.md`,
"cause not measured;
queued").
The 60 seconds those controls spend is their own patience:
each retries a refusal that changed nothing until `PATIENCE` runs out
(`package/git-policy/cli/src/native/binary/binary_service_tests.rs`,
`run_settle_past_a_refusal`).

## What the look costs, measured as the look makes it

The ceiling was never reached.
A probe repeating exactly what
`package/git-policy/cli/src/native/landing/earlier_service_process_look.rs` does
(the socket table,
the process table,
`stat` of each entry,
one read of `<pid>/cmdline`)
took 40 samples on this host on 2026-11-10 with the load average at 56 of 8 threads:

- the whole look:
  23.0 ms median,
  43.0 ms at the 90th percentile,
  54.1 ms at the slowest;
- the command-line reads:
  16.9 ms median,
  35.7 ms at the slowest;
- the socket table:
  3.5 to 6.2 ms;
- the process table listing:
  2.9 ms median,
  13.5 ms at the slowest;
- 617 to 625 command lines read of 1,336 processes listed;
- the slowest single read of one process:
  6.4 ms.

`PROCESS_LOOK_CEILING` is 10 s,
185 times the slowest whole look measured here.
So the time was not in the reads,
not in a runtime worker,
and not in a lock the controls share:
the look answered at once,
and the answer was "cannot look",
because of a process its own limit could not prove.

## Cause one: a command line longer than the limit the look read

The look read at most 65,536 bytes of each command line,
and a command line that filled it counted as one that may be the earlier helper
(`CommandLine::Unproven`),
so the whole look held.

Tens of kilobytes in one command line are ordinary:

- a `git commit -m` carries its whole message in its arguments;
- a script passed with `node -e` or a mise task's inline `run` is one argument;
- a `podman run` line carries its mounts;
- the largest command line on this host when it was measured was 44,174 bytes.

Nothing about such a program changes while it runs,
so every retry met the same answer,
and the whole machine's settle commands were refused until it ended.
The advice the message gave ("try again in a moment") was therefore an endless wait.

### Reproduction, and the same host state against the fixed product

One unrelated program of the user with a 100,041-byte argument was enough:

```sh
# a program that is not an earlier helper, and lives while the controls run
big=$(head -c 100000 /dev/zero | tr '\0' x)
python3 -c 'import time; time.sleep(600)' "$big" &
```

Before the fix,
the four settle controls on that host state:

```text
Summary [ 93.974s] 4 tests run: 1 passed, 3 failed, 920 skipped
   FAIL [89.157s] service::the_settle_command_finishes_a_batch_the_earlier_helper_left_open
   FAIL [89.329s] service::the_settle_command_is_not_held_by_a_service_another_control_is_starting
   FAIL [93.973s] service::the_settle_command_names_a_batch_it_could_not_examine_and_exits_nonzero
```

with the words above in each one's standard error.
After the fix,
the same four controls with that same program still running:

```text
Summary [ 6.567s] 4 tests run: 4 passed, 920 skipped
```

## Cause two: a command line the look may not read

A process whose `<pid>/cmdline` the kernel refuses to open
(a setuid program such as `fusermount3` or `sudo`,
a program in another user namespace,
a process held in the kernel)
was counted the same way,
and the refusal named nothing:

```text
the sockets and processes of the earlier helper could not be looked at in time, ...
To go on, try again in a moment.
```

Such a process cannot be ruled out as the earlier helper,
so refusing is right;
naming nothing is not,
because the person is the only one who can wait for it or stop it.

## The fix

- The look now reads up to the kernel's own ceiling on a command line,
  6 MiB,
  so no command line is unproven for its length.
  Linux refuses to start a program whose arguments and environment together are more than
  `_STK_LIM / 4 * 3` (`fs/exec.c`,
  `bprm_stack_limits`),
  and one argument is at most 32 pages (`MAX_ARG_STRLEN`,
  `include/uapi/linux/binfmts.h`).
  Measured on this host on 2026-11-10:
  with the default 8 MiB stack limit
  `getconf ARG_MAX` answers 2,097,152 and the kernel refused a 2 MiB argument list;
  with `ulimit -s unlimited` it answers 6,291,456,
  and a 5 MiB argument list started while 6 MiB was refused.
  The read reserves only what one process carries
  (measured:
  `take(6 MiB + 1).read_to_end` of a 12-byte command line left the buffer at 44 bytes),
  so the larger limit costs nothing for an ordinary process.
- A command line the look still cannot prove names its process:
  the process ID,
  the name the kernel keeps for its program (`/proc/<pid>/comm`,
  escaped,
  never a word of the command line,
  which can be a secret),
  what the kernel answered,
  and the two ways past it (wait for that process,
  or stop it,
  then run the command again).
- `Hold::CannotLook`'s words no longer say "in time":
  a limit or a refused read is not a deadline,
  and the sentences after the reason now say which it was.

## Controls

`package/git-policy/cli/src/native/landing/earlier_service_process_look_tests.rs`:

- `a_command_line_longer_than_a_program_can_carry_is_proved_whole`:
  a fake process table with one process whose command line is 204,800 bytes holds nothing back.
  It failed with the words above before the fix.

`package/git-policy/cli/src/native/invocation/management_service_settle_earlier_batches_tests.rs`:

- `a_program_whose_command_line_is_long_holds_nothing_back`:
  the command says its condition and examines the batch instead of refusing.
  It failed with the words above before the fix.
- `a_process_whose_command_line_cannot_be_read_is_named_with_the_answer_and_the_way_on`:
  a command line with its permissions taken away names process 77,
  the name the kernel keeps for it,
  `Permission denied`,
  the way on,
  and no word of the command line it may not read.
  It failed before the fix,
  which named nothing.

The three controls as the product stood,
before the fix:

```text
Summary [   1.227s] 3 tests run: 0 passed, 3 failed, 6289 skipped
```

and with it:

```text
Summary [   2.932s] 36 tests run: 36 passed, 6256 skipped
```

In the binary suite at its default parallelism on 2026-11-10,
with the host's load average between 85 and 125 and four other nextest suites running beside it,
all four settle controls passed:

```text
PASS [   9.091s] service::the_settle_command_changes_nothing_when_its_condition_cannot_be_written
PASS [  43.280s] service::the_settle_command_finishes_a_batch_the_earlier_helper_left_open
PASS [  43.444s] service::the_settle_command_is_not_held_by_a_service_another_control_is_starting
PASS [  67.803s] service::the_settle_command_names_a_batch_it_could_not_examine_and_exits_nonzero
```

## Limits

- A process whose command line the look may not read still holds the settle command back,
  safely,
  until that process ends;
  the message now names it.
  Reading only the processes that could be the wrapper
  (by the kernel's 15-byte program name) would remove that wait,
  and it would also miss a helper installed under any other name,
  which is the one mistake this check must never make.
- A read that waits for a process stuck in the kernel still ends at `PROCESS_LOOK_CEILING`,
  and its thread is left behind.
  No sample of this host showed such a read:
  the slowest single read in the 40 samples took 6.4 ms.
- The exact process behind the 197-test run of 2026-11-10 is not identified:
  that run's transcript kept only a truncated tail.
  What is shown here is that one ordinary program of the user reproduces that run's three failures,
  with its exact words,
  and that the fixed product passes the same controls with that program still running.
- Directory sharing is unchanged:
  the controls drive the look through their own process table where they can,
  and the one control that needs a real other process
  (`the_settle_command_is_not_held_by_a_service_another_control_is_starting`)
  still reads the machine's own,
  because that is what it is about.
