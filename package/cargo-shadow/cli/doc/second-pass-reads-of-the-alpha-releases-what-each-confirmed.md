# Second-pass reads of the alpha releases: what each confirmed

What the second pass over each alpha release of this package confirmed with the
released binaries,
one section per release.

Status:
**unapproved and non-final**.
Written by the coordinating session from the second passes' reports.
Each pass was another agent of the same vendor,
so a second pass and not an independent review,
and each is named by the release it read.
Every figure below is one that pass measured;
the behaviour it confirmed is stated where this package's `README.md` states
it,
and this file holds only what the released binaries themselves showed.

## The alpha.13 pass: cost of a trivial call against alpha.11

The pass compared `0.1.5-alpha.13` with `0.1.5-alpha.11`,
and with `0.1.5-alpha.12` beside them.
In six interleaved rounds a trivial shadow call took 1.085 times alpha.11's time
at the median (0.667 to 2.205),
against 1.097 (0.776 to 2.364) for a same-binary control and 1.982 for alpha.12.
It asked the manager two questions a call where alpha.12 asked ten,
and no stop in 24 scoped runs.

## The alpha.14 read: cost of a call against alpha.11, and what the harness could not tell apart

The read compared `0.1.5-alpha.14` with `0.1.5-alpha.11`,
and found no decision or output that differs from `0.1.5-alpha.13` beyond what
the merges name.
The manager is asked two questions a call in alpha.11,
alpha.13 and alpha.14 alike,
over 260 timed calls.
The cost of a call against alpha.11 had a median ratio of 1.021 for alpha.14
(0.409 to 2.072 over 20 rounds),
with a byte-identical copy of alpha.11 as the control at 0.907
(0.457 to 2.019),
so the harness tells alpha.14 from neither alpha.13 nor alpha.11.

## The alpha.15 read: what the released binaries showed, and the resolution of a timing here

The read compared `0.1.5-alpha.15` with `0.1.5-alpha.14`.
Between the two released binaries nothing differs but the version and the
sentences the merges name:
226 of 232 compared outputs were identical,
the rest version lines,
and alpha.12 as the case that must differ did,
in 21.
A trivial call asks the manager two questions in alpha.11,
alpha.14 and alpha.15 against six to ten in alpha.12.
Timing cannot see a small change on this host:
a byte-identical copy's own ratio ranged from 0.414 to 1.500.

## The alpha.16 read: what the released binaries showed across a reexec

The read compared `0.1.5-alpha.16` with `0.1.5-alpha.15`.
Across real reexecs of a container's own manager,
13 of 13 calls per arm that began inside a reexec took the weaker path under
`0.1.5-alpha.15` and the delegated route under `0.1.5-alpha.16`.
The kinds of later run after a kept journal,
and the fast path,
behave the same on both releases.
Where the manager answers at once the two differ in 12 of 240 compared files:
the version lines,
and the help of `install` and `update`,
which now prints the long layout for every option,
a change no record names.

## The alpha.18 read: controls that could not fail, and sentences that said more than their evidence

Written 2026-11-28T07:11:50-05:00 by the round on branch
`fix/cargo-shadow-cli-cargo-shadow-alpha-18-read-controls-that-can-fail-and-sentences-that-say-what-runs`.
**Unapproved and non-final**:
the owner declares completion.

The read was a second pass by another agent of the same vendor over `0.1.5-alpha.18`,
so a second pass and not an independent review.
It raised no High finding,
two Medium ones and six Low ones.
The two Medium findings are answered in `src/control_group_route_discovery.rs` by the branch
`fix/cargo-shadow-cli-cargo-shadow-alpha-18-read-a-stop-past-the-bound-is-refused-and-a-reported-path-is-checked`,
whose release code this round changed nothing in.
The six Low findings are this round's,
and this section holds what each one was,
what changed for it,
and what was left.

### Low 1: six edits, each now held by a control that goes red under it

Each edit was applied to the release code of this round's worktree,
the controls named here ran in one nextest invocation,
and the edit was then removed and the file restored.
Under every edit the controls that held the record before stayed green,
which is the finding:
a green suite did not show these edits.
Each pair below is one run of 2026-11-28,
with `cargo nextest run --release --no-fail-fast` and one filter per control.

#### The question's `--property=Job`

Dropped from the question in `src/control_group_route_one_ask.rs`.

Green:
the four controls that read the manager's answers,
`a_path_the_ask_after_a_failed_start_reports_is_the_route`,
`a_start_that_never_reached_the_manager_is_put_again`,
`a_start_that_failed_and_the_ask_after_it_failing_too_is_put_again` and
`a_stop_job_queued_on_the_slice_is_asked_through_before_the_route_is_taken`.

Red:
`control_group_route_one_ask::one_ask_tests::the_question_asks_for_the_slice_s_control_group_and_the_job_queued_on_it`,
with:

```text
the question does not ask for the slice's control group and the job queued on it in one program start: "--user show app.slice --property=ControlGroup\n"
```

#### The middle of the reading of a failed start's words

Two halves of `the_manager_answered_a_start`.

The transport's words deleted:

- green:
  `a_refusal_the_manager_answers_ends_the_asking` and
  `a_start_that_never_reached_the_manager_is_put_again`;
- red:
  `one_ask_tests::the_manager_answered_a_start_tells_its_refusal_from_a_moment_and_from_other_words`,
  with
  `a start whose reply was lost was read as the manager's refusal, so a call that really started the slice would be refused`.

The slice's name after the verb no longer required:

- green:
  the same two controls;
- red:
  the same control,
  with
  `words that name the slice only in the verb's opening were read as the manager's answer about it, so a refusal that names no unit would end the asking`.

#### `spent` after a start

`!spent &&` dropped from `ask_once`'s failed-start answer in `src/control_group_route_discovery.rs`.

Green:
`a_manager_that_answers_nothing_is_asked_once_and_ends_the_discovery_with_its_own_words`,
which is the `show` that spends the bound,
so no control but this round's makes a start spend one.

Red:
`asking_again_tests::a_start_that_spent_its_bound_is_not_put_again`,
which counted six asks where three are expected:
a first attempt of `show`,
`start` and `show`,
then a second attempt after the question was put again.

#### A sleep longer than the pause

`std::thread::sleep(ASKING_AGAIN_PAUSE * 2)` at the call site in `ask_where_the_slice_is`.

Green:
`reexec_tests::the_gap_between_two_asks_is_the_pause_the_call_site_sleeps`,
whose four gaps were 406.5 ms to 407.5 ms.

Red:
`reexec_tests::the_pauses_of_the_asking_again_are_no_longer_than_the_pause_the_call_site_sleeps`,
with:

```text
the shortest pause between two asks was 404.084351ms, longer than the 200ms the call site sleeps plus the 150ms the work around it is measured to take, so the span is not the 800ms the records name
```

#### The kind of file an address names

`is_a_socket` replaced by a check that the path exists in `src/control_group_route_manager_reach.rs`.

Green:
`a_manager_socket_at_the_runtime_directory_keeps_the_asking_again`,
`a_bus_address_naming_a_socket_keeps_the_asking_again_where_no_runtime_directory_is_set` and
`a_bus_address_naming_a_socket_keeps_the_asking_again_where_a_runtime_directory_is_set`.

Red:
`a_plain_file_at_a_bus_address_is_not_a_transport_the_ask_takes` and
`a_directory_at_a_bus_address_is_not_a_transport_the_ask_takes`,
each with five asks where one is required.

#### A path beside a queued job after a successful start

The final ask's path refused when a job stands on the slice.

Green:
`a_stop_job_queued_on_the_slice_is_asked_through_before_the_route_is_taken`,
whose last ask is the real manager's and answers no job.

Red:
`asking_again_tests::a_path_beside_a_job_after_a_successful_start_is_taken_as_the_route`,
with:

```text
a path the manager reports beside a job, after a start that ran, is the route: GroupUnavailable { reason: "a job is queued on app.slice after the start" }
```

#### What the green arm measured

The controls that stay green in the green arm of every pair are the ones named above,
and each was run in the same invocation as the control that reddened.
The pause's own gaps with the pause in place were 208.0 ms,
207.0 ms,
206.2 ms and 207.9 ms,
so the work around the sleep cost at most 8.0 ms there.
`THE_MOST_A_GAP_OUTLASTS_THE_PAUSE` (150 ms) is sized from that and from the earlier run the package document records,
and it does not catch a pause that grew by less than that.

One more control came with them,
for a branch no record read before:
`what_the_manager_reported_reads_the_two_named_lines_and_one_value_alone`.
It reads the answer's two named lines,
the one value a stand-in prints,
an empty path,
a relative one,
a value spanning lines and a name that is not the slice's.

One trap of the method is worth recording.
Each edit was removed by copying the file back with its timestamps preserved,
so the restored file carried its original modification time,
which is older than the build the edit had just made.
Cargo reads that as a source it has already compiled,
so the first run of the restored tree re-ran the edited binary and reddened the very control the removal was meant to green.
The restored files were touched after the last removal,
and the green run is the one made after that.
A round that removes an edit must make the next build see the source again.

What the read named and this round leaves uncovered:

- an address naming an existing socket beside a runtime directory that holds the bus socket:
  the answer is the same either way,
  because the bus socket alone already keeps the asking again,
  so no count of asks can tell an edit that skips the address there;
  only the words of a refusal that appears when nothing is there could,
  and no state prints them;
- the reading of a failed start's words is held for the four shapes this round wrote;
  a refusal of a shape nobody has captured is still read as a moment,
  which is the safe direction and is said in the source.

### Low 5: three sentences, corrected against their own records

#### Every discovery was done before the window opened

The sentence stood in the README's Development section and in the package document's "What the span does not cover".
The record it cites is the alpha.17 read's own log of 24 calls whose reexec was fired 0 to 110 ms after each began,
and it shows the opposite for half of them:

- in 12 of the 24 an ask overlapped the window,
  and every one of those twelve calls exited 0;
- the 11 calls that exited 1 are the ones whose asks had all ended before the window opened,
  being 7 of `0.1.5-alpha.16`'s 12 and 4 of `0.1.5-alpha.17`'s 12;
- one more call of the 24 exited 0 without meeting the window.

Both sentences now say that.

#### 2 of its 24 calls

It stood in the package document twice,
in "What this does not cover" and in the Open list.
The 24 is `0.1.5-alpha.17`'s 12 aligned calls together with the 12 of its byte-identical copy,
and the two are `0.1.5-alpha.17`'s own:

- the state was reached in 2 of its 12 calls;
- in 0 of the copy's 12;
- and in 0 of its 9 calls against a loaded manager.

Both sentences now say that.

#### A program that outlives its run is left alive on that fallback

The sentence stood in the README under "Limits".
Measured by the alpha.18 read on this host with no manager at all:

- a program started with `setsid` was left alive in 4 of 4 calls;
- one that stays in the run's process group was killed in 4 of 4;
- and the same held in both releases.

The README now says that a program staying in the group is killed and one that leaves it is left alive.

### Low 6: what the control of issue 6650 prints when it fails

The control's clean-up wrote `cgroup.kill` and discarded the answer,
then waited twenty seconds for the group below the scope to list no process,
and its failure said only `the nested group's end did not happen in time`.
It now keeps the write's answer and reads the state of every process the group still lists.

The sentence it fails with is this one,
printed by the control of that sentence with a fifty-millisecond bound and a group holding a live process,
a zombie and an id no process of this machine can have:

```text
the nested group <group> still listed a process at the bound of 50ms: the write of <group>/cgroup.kill was refused (Is a directory (os error 21)), and every process it lists: <pid> is alive, so the kill did not take it (state "S" of /proc/<pid>/stat: <the stat line>); <pid> has exited and waits to be reaped, so it can write nothing (state "Z" of /proc/<pid>/stat: <the stat line>); 4294967295 has no process any more (/proc/4294967295/stat could not be read: No such file or directory (os error 2)), so the group's listing has not caught up with a member that left. A member that has exited or is no longer there was slow to leave; one alive in any other state is a member the kill did not take
```

The control's own bound is unchanged at twenty seconds,
so the only difference in a real failure is the bound's figure and the group's own path.
What the sentence still cannot say is why a member stayed:
a member that has exited and a listing that has not caught up look the same from here,
and the sentence names which of the two the state shows rather than guessing.

### Low 2, Low 3 and Low 4: what was left, and the figures

No code changed for these three.
Their figures are restated in the package document's Open list,
and the questions they leave are in the round's report.

#### Low 2: a refusal for good asked fifteen times

A refusal for good whose words do not name the slice after the verb is asked five times,
which is 15 asks where `0.1.5-alpha.17` made 3.
A call's time as a ratio to `0.1.5-alpha.17` in the same round,
with a byte-identical copy of that release as the control:

- 8.47 (6.32 to 10.38) for a dependency that fails every time;
- 8.55 (6.32 to 10.93) for a cyclic transaction order;
- 8.97 (7.60 to 11.69) for a masked dependency;
- 8.87 (5.28 to 9.99) for a missing one;
- 1.05 (0.98 to 1.15) for a masked slice,
  whose words do name it;
- and the copy's own ratio ranged from 0.40 to 2.37.

A dependency that sleeps eight seconds and then fails took 15 asks and 41.9 s to 49.5 s a call against 8.3 s to 10.7 s.

#### Low 3: the early ends that remain

- a dependency of `app.slice` that starts slower than the ten-second bound takes the weaker path in 9 of 9 calls of each of the three releases,
  and the slice is active a few seconds later;
- a `show` that hangs on a busy host spent the bound in one call of 15 and took the weaker path after 18.2 s;
- a slice stopped between a successful start and the final ask takes the weaker path in 9 of 9 calls;
- and a stop job queued after a successful start makes the call exit 1 in 9 of 9 calls,
  because the final ask's path is taken and the scope is then refused as destructive.

#### Low 4: a call's start replaces a person's stop

Nine seconds after a call,
`app.slice` is active under `0.1.5-alpha.18` where it was inactive under `0.1.5-alpha.17`.
That is two calls an arm,
with a service holding the slice for four seconds and the stop issued 0.7 s before the call.
The discovery's `start` uses `replace` mode,
so it cancels the person's queued stop.
The README now says so where the queued stop is described.

### Where this round's controls meet the Medium round's change

Two of the controls added here read `ask_once`,
which the Medium round changes:

- `asking_again_tests::a_start_that_spent_its_bound_is_not_put_again` requires three asks for one attempt and a refusal that names the bound the start spent;
- `...::a_path_beside_a_job_after_a_successful_start_is_taken_as_the_route` requires that the path an answer carries is the route even when the same answer carries a queued job.

A change that refuses a path beside a job,
or that asks again after a start that spent its bound,
reddens them,
which is what they are for.
