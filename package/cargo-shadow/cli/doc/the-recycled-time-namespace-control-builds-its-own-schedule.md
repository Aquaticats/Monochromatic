# The recycled time-namespace control builds its own schedule

Issue 6599,
round of 2026-11-27,
unapproved and non-final.

The control is
`round_fourteen_tests::a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone`.
It guards acceptance rows A1 and V2 of round 14 section 2 through round 15:
a recorded number that names a later,
different time namespace must not turn a live launcher's differing start time into "gone".
The record holds no time-namespace number at all,
so the question answers "unknown",
and the control's job is to build the state in which a number that *was* the writer's is now the recovery's.

## The schedule it needs

Four facts,
in this order:

1. A writer runs in a time namespace T0 with a boot-time offset of one hour,
   starts a launcher in another time namespace T1 with an offset of two hours,
   and reads the launcher's start time on T0's basis.
2. T0 loses its last process while the launcher in T1 lives on.
3. A time namespace T2 is made afterwards and is handed the number T0 had,
   so the same number names two different namespaces at two different times.
4. The recovery runs in T2 and reads the live launcher's start time on T2's basis,
   which differs from the recorded one,
   and answers "unknown",
   never "gone".

Fact 3 is the kernel's to give:
a namespace number is an identifier the kernel hands out and takes back.
A test can only make fact 3 true by being the process the kernel hands the number to.

## What it did before, and what defeated it

Until this round the test ended T0's processes and then asked for T2 with `nsenter` plus `unshare --time`.
Between the two it scanned `/proc` for T0's members,
waited for that scan to come back empty,
waited for the outer `unshare` to end,
read whether the launcher still ran,
planned a scope name,
wrote the journal's lockfile,
planted a journal (a transaction that syncs files to disk),
wrote the journal's group records,
and started `nsenter` and `unshare` as two processes.

The number was free for all of that.
The kernel hands a freed number to whoever asks next,
and the control's own unit and mount fixtures are not the only processes asking:
other test binaries of other sessions on this host create and destroy namespaces throughout.
So the control lost the number to another session,
eight attempts in a row,
and failed as if the schedule had been made and the code had got it wrong.

The round that opened issue 6599 met this twice in five runs of the whole suite,
on trees at `b6190b8ce1` and after its own change,
and passed in the three runs after `origin/main` was merged in.

The plant that shows it is kept in the test:
`CARGO_SHADOW_ROUND_FOURTEEN_PLANT_FOREIGN_NAMESPACE=1` makes the test itself allocate the one namespace another session
would,
inside the user namespace that owns T0,
and hold it for the attempt.
With the schedule this round replaced,
that plant takes T0's number every time:

```text
the freed time namespace number was not reused by the recovery's namespace in any attempt,
so the schedule was not made: the writer's time namespace was 4026533635
and the recovery's is Some("4026533637"); the planted other session took 4026533635.
```

## What the kernel does, measured

- A namespace number is the inode number `/proc/<pid>/ns/time` reports,
  and `fs/nsfs.c` sets `inode->i_ino = ns->inum` (read from `torvalds/linux` master on 2026-11-27),
  so the number a reader sees is the kernel's own namespace id.
- A destroyed namespace's number is handed to the next namespace made,
  not to a later one:
  a time namespace created inside a user namespace right after a destroyed one,
  and again after delays of one millisecond,
  five milliseconds,
  twenty milliseconds and a tenth of a second,
  was handed the same number eight times out of eight at every delay tried (measured on this host,
  2026-11-27).
  So the number waits for the next ask,
  and the ask is what a control has to own.
- The numbers *below* a long-lived namespace's number are free too on a busy host:
  the three runs of the fixed control before the reader passed over the free numbers below the writer's
  were handed numbers two,
  four and forty-four below it.
  The kernel hands out the lowest free number,
  so a reader that asks once can be handed one of those instead.
- Holding a namespace costs one open descriptor on `/proc/<pid>/ns/time`,
  and while it is open the namespace counts as used,
  so its number is not handed out (measured the same day:
  with the descriptor held,
  a namespace made next to it took the following number;
  once the descriptor was closed,
  the next namespace took the held number).

## What it does now

1. An armed reader joins the user namespace that owns T0 before T0's processes end,
   and opens T0's `/proc/<thread>/ns/time` file.
   From that moment the number cannot be handed to anyone else.
   It checks the number it opened against the one the writer recorded.
2. The test ends T0's processes,
   plants the journal and only then releases the reader with one line on its standard input.
   An end of file instead of that line means the test ended first,
   and the reader says so rather than waiting for a test that is gone.
3. The reader closes its descriptor,
   which is the kernel's last use of T0,
   and asks for a time namespace with `unshare(CLONE_NEWTIME)`.
   Each number it is handed that is not the writer's is held by a descriptor,
   which keeps it taken so the next ask moves past it,
   until the kernel hands the writer's number back.
   Its bound is `READER_ASSIGNMENTS`;
   a number it never gets is reported as a precondition this host did not give,
   never as the code under test being wrong.
4. The fixture is a child of that thread,
   so it is made in the namespace the reader was handed,
   whose offsets stay the kernel's default of zero against the writer's one hour.
   That is the differing basis the control needs,
   and it is a fact of the test's own steps rather than of a host's state.

Only a thread of a process whose other threads keep their own namespaces can do step 3:
a process cannot be moved into a new time namespace (a running process keeps its clocks),
only its children are made in one,
and the offsets of the namespace for children can be written only through `/proc/<tgid>/timens_offsets`,
which is not the unsharing thread's.
The reader therefore writes no offsets and leaves them at zero.
`rustix`'s `thread` feature is a dev-dependency feature beside `mount`,
so the release binary compiles neither.

## What changed and what it rests on

- The plant and the failing line above rest on real runs of this host on 2026-11-27,
  with the schedule this round replaced and the plant set.
- The reuse and hold measurements rest on probes of this host on the same day,
  kept in the round's logs,
  not on any claim read from the kernel's source.
- `inode->i_ino = ns->inum` rests on `fs/nsfs.c` of `torvalds/linux` master,
  fetched on 2026-11-27.
  The kernel of this host is `7.3.5-2-cachyos`,
  whose full source is not on the machine,
  so no line of *that* tree was read.
- Whether a merge between `b6190b8ce1` and `c59a4f5fd5` changed the control's outcome:
  none did.
  No commit in that range touches `src/round_fourteen_tests.rs`,
  `src/cgroup_test_support.rs`,
  `src/control_group_process_identity.rs`,
  `src/journal_support.rs`,
  `src/contain_counterexample_tests.rs` or `src/test_scratch/scratch.rs`
  (`git diff b6190b8ce1 c59a4f5fd5 --` those paths is empty),
  and the two files of the package the range does change are `src/lib.rs` (module visibility)
  and `src/test_scratch.rs` (a source walk),
  neither of which the control reads.
  The green runs after `origin/main` was merged in therefore say nothing about the failure:
  they are runs of a suite whose other tests asked for namespaces at different moments.

## How to see the red and the green

```sh
# package/cargo-shadow/cli/README.md
cargo nextest run --release -E 'test(=round_fourteen_tests::a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone)'
CARGO_SHADOW_ROUND_FOURTEEN_PLANT_FOREIGN_NAMESPACE=1 \
  cargo nextest run --release -E 'test(=round_fourteen_tests::a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone)'
```

The first command is the control as the task runs it.
The second plants the other session's allocation and must also pass;
on the schedule this round replaced it failed with the line quoted above.
