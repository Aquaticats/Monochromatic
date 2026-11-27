# A minimum-rate control measured the host's scheduling, not the traffic its socket carried

`push::push_minimum_rate_socket_tests::a_transfer_that_meets_the_floor_is_work_however_slow`
in `package/git-policy/cli/src/native/push/push_minimum_rate_socket_tests.rs`
failed once in a whole gate on 2026-11-27,
while the host's load average stood between 48 and 96,
and passed in the four gates beside it.
It failed alone,
in a process of its own,
so no other test of the run shared its state.

This record is about the defect behind that:
a control whose transfer moved at a rate only the host could keep,
and a judgement that therefore measured how busy the host was rather than what the socket carried.

Status:
unapproved and non-final.
The owner declares completion.

## Symptom

The gate `gpc-gate-issue-6562-clippy-bound-20261127T045928-0500`,
on tree `2a3bf031f8`:

```text
FAIL [   2.450s] monochromatic-git-policy-cli push::push_minimum_rate_socket_tests::a_transfer_that_meets_the_floor_is_work_however_slow
thread 'push::push_minimum_rate_socket_tests::a_transfer_that_meets_the_floor_is_work_however_slow' (77336) panicked at src/native/push/push_minimum_rate_socket_tests.rs:320:5:
a transfer above the floor is work: a stop by Connected after 2 seconds, carrying 3072 bytes
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10777 filtered out; finished in 2.28s
```

The three other controls of that module passed in the same gate,
in 2.438 s,
2.399 s and 4.317 s.

The gate starts each control in a process of its own,
inside a container the gate caps,
so nothing else in that run competed with it:
what starved it was the rest of the host.

## What the control assumed

The control started a shell that opened `/dev/tcp` to a loopback listener,
slept one second,
and then wrote 1024 bytes every 0.2 seconds:
about 5 KB/s,
against a floor of 4096 bytes in a connected window the control shortened to two seconds.

That is 10240 bytes for every window the clock judges,
two and a half times the floor,
and it holds only while the host schedules two processes for every chunk in time
(`head`,
and its own `sleep`).

The floor's own record says what the window really counted:
3072 bytes,
three chunks.

## What the window counted against what the sender meant to send

Measured 2026-11-27 with a temporary arm of that same file,
built from the control's own drive and a sender that records every chunk it writes in a file of its own,
so the number the sender meant is the fixture's own record and not the control's belief.
Two arms,
one run each:
the sender's own 0.2 seconds between chunks,
and 0.55 seconds,
which is what the loaded host made of those 0.2 seconds.

```text
PACED pause=0.2 elapsed_ms=4009 offered_chunks=16 bytes_the_sender_meant=16384 stop=no stop
PACED pause=0.55 elapsed_ms=2072 offered_chunks=3 bytes_the_sender_meant=3072 stop=a stop by Connected after 2 seconds, carrying 3072 bytes
PACED pause=0.55 counted_in_the_window=3072 floor=4096 window=Connected limit=2s elapsed=2.056301131s
```

The arm at the sender's intended pace offered 16 chunks in 4.009 s and was not stopped.
The arm with the planted delay offered three chunks when the window was judged,
and the window counted exactly what the sender had offered:
3072 bytes by the sender's record,
3072 bytes by the socket's counters.

So nothing undercounted and nothing lagged:
the window counted the traffic the connection carried,
and the sender carried a third of what the control had assumed it would.
The number the gate saw under load,
3072,
is the number the planted delay produces.

## Root cause

The rule itself is not at fault.
The release judges the transport's bytes over the connected window,
which is wall time by design,
and that is the bound a person's line is held to
(`SPEC.md`,
"The least traffic that counts as progress").

The control was at fault in two ways:

- Its premise was a rate the host keeps,
  not a rate the control keeps.
The sender's pace was the fixture's,
but the window that judged it was the host's,
so the control measured the scheduling of processes it did not own.
- Its record could not tell the two apart.
Nothing in the control said what the sender had offered when the window was judged,
so a stop with 3072 bytes read equally well as a meter that undercounted the transport.

## Reproduction

The failure is shown by planting what the host did,
and never by loading the host:
a sender whose chunks are 0.55 seconds apart,
about what the two tenths of a second became under that gate's load,
against the control's own two-second window and 4096 byte floor.
The window then ends the transfer at 2.056 s,
carrying the three chunks the sender managed,
which is `a stop by Connected after 2 seconds, carrying 3072 bytes`.

The fixture is the control's own:
a loopback listener of the control's,
a `sh` that writes `/dev/tcp`,
and the push's clock over that shell.
Two arms of one run are enough to show both sides,
since the arm at the intended pace offers sixteen chunks over the same span.

With the same 0.55 second pause planted in its sender,
the control as this round leaves it passes,
in 9.32 s,
because it waits for the chunks instead of judging the seconds they took:

```text
test push::push_minimum_rate_socket_tests::a_transfer_that_meets_the_floor_is_work_however_slow ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10795 filtered out; finished in 9.32s
```

## What the module's other controls rest on

Each control that asks whether something is stopped is safe on a loaded host,
because a host that is late can make the stop it waits for late and never early.
Each control that asks whether something is not stopped needs a rate it keeps itself,
and these are the ones that have one:

- `a_transfer_that_meets_the_floor_is_work_however_slow` and
`the_same_trickle_without_a_floor_is_progress`,
  in this module:
their clock is the transfer's own,
one slice of the window for every slice's worth of bytes the connection carried.
- `push_progress_tests::a_tree_that_keeps_moving_is_never_stopped_for_time`:
rests on a shell of its own writing two bytes every tenth of a second,
fifteen chunks inside its 1.5 second stall window.
- `push_patience_tests::a_push_that_keeps_moving_data_past_the_stall_window_is_not_stopped`:
rests on a real push whose hook prints five bytes every 250 ms,
against a floor it lowers to 8 bytes and a 3 second stall window,
sixty bytes offered for every window.
- `binary_push_minimum_rate_tests::an_honest_slow_push_above_the_floor_lands`:
rests on a relay that reads 8 KB every 50 ms,
  about 160 KB/s,
against 512 bytes in a six second window,
  some nineteen hundred times the floor.
- `push_wait_tests::the_bytes_of_a_socket_are_progress_when_nothing_else_moves`:
waits for the first byte a probe sends,
  with `PATIENCE` around the wait,
so a delay lengthens the wait and never ends it.

The controls whose subject is a stop read the host's clock and want it to:
`a_trickle_below_the_floor_is_ended_at_the_connected_window` and
`a_window_with_no_bytes_is_bounded_by_the_connected_window_and_no_sooner` in this module,
`push_progress_tests::a_silent_open_connection_is_ended_at_the_connected_window`,
`push_progress_tests::a_connected_tree_that_only_uses_processor_time_is_waiting`,
`push_progress_tests::a_connected_tree_that_only_writes_its_own_meter_is_waiting`,
and the silent-server controls of `push_slow_server_tests`.
Their upper bounds carry the host's lateness in full.

## What the control does now

The control states the clock its transfer is judged by.
It reads the connection until it has carried the bytes one slice of the window means,
states that slice has passed,
and asks the clock,
through `Clock::judged_after`,
the judgement `Clock::check` makes at the time it is given.
Two slices' worth of the floor go into every slice,
so a window the control states carries twice the floor and a lost slice is not a window under it.

The release is unchanged:
`check` is `judged_at(Instant::now())`,
and `judged_after` is compiled out of every build that is not a test.

The holder is now `spawn_owned`'s child,
so its process group ends with the test thread's fixtures however the control ends,
and the ledger line the site held in
`source_guard/test_children_not_yet_owned.txt` is gone.

The control also says the transfer really was slow,
by asserting that its bytes arrived over more seconds than one window holds,
which a host that is late can only lengthen.

## Open

- The arm at the sender's intended pace offered 16 chunks over 4.009 s,
which is about 4 KB/s,
below the 5 KB/s its own script names.
The gap is the cost of starting `head` and `sleep` for every chunk;
nobody has measured how it moves with the host's load.
- `the_same_trickle_without_a_floor_is_progress` now states a clock slower than the trickle it offers,
so it no longer reads a real-time window at all.
That is the price of a rate it keeps itself;
the wall-clock side of the rule is pinned by the two controls whose subject is a stop.
- No control of this module has been run on a host at the load average the failing gate met,
because measuring that would mean generating the load,
which no run on this machine may do.
The planted delay is the reproduction.
