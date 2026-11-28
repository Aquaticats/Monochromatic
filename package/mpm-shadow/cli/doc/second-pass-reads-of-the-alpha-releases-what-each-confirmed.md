# Second-pass reads of the alpha releases: what each confirmed

What the second pass over each alpha release of this package confirmed with the
released binaries,
one section per release,
and what this machine showed of the one condition a Medium finding needed.

Status:
**unapproved and non-final**.
Written by the coordinating session from the second passes' reports.
Each pass was another agent of the same vendor,
so a second pass and not an independent review,
and each is named by the release it read.
Every figure below is one that pass measured.
What each confirmation says about the tool's own behaviour stands in this
package's `README.md`;
this file holds only what the released binaries themselves showed,
on fixtures with real kills.

## The alpha.14 pass: what held as claimed

The pass compared `0.2.7-alpha.14` with `0.2.7-alpha.13`.
Right as claimed:

- the exchange leaves no absence:
  none in 25 rounds on ZFS and on tmpfs,
  where `0.2.7-alpha.13` showed one in 3 and in 1 of 25,
  and 4 of 4 with its first rename slowed;
- an unfinished copy and another version's finished copy were never published;
- whether a refused start is repeated did not change;
- 53 command lines are identical between the two binaries.

## The alpha.14 pass: what this machine shows of `hidepid`

This machine ran `0.2.7-alpha.14` until `0.2.7-alpha.15` was installed at
2026-11-27T13:29.
`findmnt /proc` showed no `hidepid` here,
so the reading a live installer the reader cannot see gets was never met on
this host,
and the race a Medium finding needs is the one that needs a killed install's
leftovers and two installs at once.

## The alpha.15 read: what the released binaries showed

The pass compared `0.2.7-alpha.15` with `0.2.7-alpha.14`.
Shown with both released binaries on fixtures with real kills:

- a claimed finished copy that could not be published stays for the next
  install,
  which publishes it with no download;
- the copy a kill moved aside is not put back beside a live claim or a live
  staging name;
- a recovery never publishes an older version beside a newer one in either
  toolchain root;
- each of the launcher's five start messages names the file that is missing or
  busy.

With no killed install behind a run,
nothing differs between the two releases but the version and the marker's text.

## The alpha.16 read: what the released binaries showed on fixtures with real kills

The pass compared `0.2.7-alpha.16` with `0.2.7-alpha.15`.
Shown with both released binaries on fixtures with real kills:

- the order of a recovery's last two steps,
  where a start in the window exits 127 on `0.2.7-alpha.15` and runs the newer
  version on `0.2.7-alpha.16`;
- the raced publish,
  where `0.2.7-alpha.15` ends 1 with both hidden copies kept and
  `0.2.7-alpha.16` ends 0 with the other install's copy as the version;
- every message of the round on a hidden name nobody can judge;
- all twelve text blocks of the README's section,
  byte for byte.

On input with nothing left behind,
38 commands compared between the two binaries differ in the version string
alone.

## The alpha.17 read: what the released binaries showed

The pass compared `0.2.7-alpha.17` with `0.2.7-alpha.16`.
It confirmed with the released binaries that:

- 38 commands on an input with nothing left behind differ in the version string
  alone,
  as the alpha.16 read found of its pair;
- a move of `current` that fails once and a put-back behave in
  `0.2.7-alpha.17` as in `0.2.7-alpha.16`;
- the README's blocks for a kill match what `0.2.7-alpha.17` prints.

## The alpha.19 pass: four sentences and one name length

Status:
**unapproved and non-final**.
The pass compared `0.2.7-alpha.19`
(release commit `1cd66ac8df`)
with `0.2.7-alpha.18`
(release commit `39e60f48cc`).
It was another agent of the same vendor as some of the authors,
so a second pass and not an independent review,
and it named its findings M-1 and L-1 to L-9.
The pass's own report and transcripts are scratch and expire a day after their
last change,
so what this round leaned on from them is restated here.

The round that answered it owned the sentences an install prints and one name
length:
L-4,
L-5,
L-6,
L-7 and the part of L-8 about those sentences.
The exposure folder's removal
(M-1 and L-1,
L-2,
L-3 and L-9)
belongs to the round
`mpm-shadow-alpha-19-read-two-installs-at-once-fail-neither-and-the-removal-names-what-it-keeps`,
and this record says nothing about it.

Each finding was reproduced before anything changed,
by a control that fails on `main` for the reason the finding gives,
and each control was then proved to fail again by planting the old behaviour
back.
The released binaries the pass used are not in this round's hands,
so the released-binary evidence for every finding below is the pass's own
transcripts,
quoted where they matter,
and every red named here is a control run on this branch.

### L-5: the added target's sentence named a listing this release never asks

`0.2.7-alpha.18` asked `rustup target list --installed` for a target,
and `0.2.7-alpha.19` asks `rustup component list --installed` and reads the
target from its `rust-std-<triple>` line.
The sentence an install prints for a target it added was left as it was,
so it named a command the run never made:

```text
toolchain install: rust: nightly-2026-11-16 was already installed; added the declared target aarch64-linux-android, which `rustup target list --installed` did not show
```

A person who read that line and ran the named command found a different answer
from the one the sentence said it would give.
The sentence now names the listing the run asked and the line of it the target
was looked for in:

```text
toolchain install: rust: nightly-2026-11-16 was already installed; added the declared target aarch64-linux-android, which the `rust-std-aarch64-linux-android` line of `rustup component list --installed` did not show
```

A component's sentence was already right and is unchanged.
Control:
`the_sentence_for_an_added_target_names_the_listing_this_release_reads`,
which also holds that no sentence names `rustup target list --installed` and
that the run asks no such listing.
Proved to fail by putting the old sentence back:
it failed with the two texts above side by side.
Run at the user boundary on a build of this branch:
`install --frozen-lockfile` in a throwaway scope with a stand-in `rustup` printed
the new sentence,
exit 0.

### L-6: two sentences promised more than every state keeps

The first is the README's line about the copy a failed publish keeps:
"that copy is the one the next install publishes".
It does not hold for a copy whose hidden name belongs to another process
namespace,
which no install there judges,
nor once a newer version of this repository's tool is installed,
which leaves the copy hidden.
The README now says which run publishes the copy and names both conditions,
and the sentence itself (quoted in full in the README's block) already said
both.

The second is the warning a failed recovery becomes.
That warning is printed by the run whose own install then put the version at its
name,
and it ended with the sentence the six failed-publish messages end with,
whose first half offers a publish of the kept copy while the version's name
holds no finished copy.
That run had already made that half impossible:
what a later install does from there is remove the copy as a second one.
The warning's sentence before:

```text
A later install of that version publishes that copy without downloading when the version's name holds no finished copy by then, and removes it as a second finished copy when that name holds one; it acts on that name in this process namespace and after a reboot, but an install in another process namespace does not judge that name and leaves the copy where it is, and prune --toolchain prints that name and what deleting it costs; a newer version of a tool this repository publishes, installed by then, leaves that copy hidden, because an older repository tool is never kept
```

and after:

```text
A later install of that version removes that copy as a second finished copy, because the version's name holds a finished copy by then; it acts on that name in this process namespace and after a reboot, but an install in another process namespace does not judge that name and leaves the copy where it is, and prune --toolchain prints that name and what deleting it costs; a newer version of a tool this repository publishes, installed by then, leaves that copy hidden, because an older repository tool is never kept
```

The six failed-publish messages keep the longer sentence,
because both of its halves are true of them.
Control:
`a_failed_claim_says_what_a_later_install_does_with_the_finished_copy_it_named`
holds the warning's words and that they do not offer a publish that state has
ruled out;
`a_run_that_publishes_after_a_failed_recovery_says_what_it_then_did` and the
five controls of the messages hold the rest.
Proved to fail by putting the longer sentence back in the warning:
it failed with both texts above.

### L-7: an install waited on `rustup` with no word, and the README named the wait for `list` only

No deadline ends a wait for a program's answer,
and that is on purpose:
a cut-off would tell a person a toolchain's parts could not be read when
`rustup` was merely slow.
What was missing was any word while it waited,
so a run that was stuck and a run that was working looked alike,
and the README named the wait for `list` alone
although `install` waits the same way and the release added one more question
per Rust row.

Every program a command starts is now waited on through one function,
and when a wait passes 2 s it prints one line on standard error,
once,
however long the wait lasts:

```text
mpm-shadow: still waiting for `rustup component list --installed --toolchain nightly-2026-11-16` to answer, 2.0 s after this command started it; no deadline ends that wait
```

The bound is not a deadline and ends nothing.
It comes from what an ordinary answer costs,
measured on 2026-11-28 with three arms interleaved in each of 40 rounds:

- this machine's `rustup 1.29.3` answering `--version`,
  which is the read-only call class the real program was timed on:
  4.39 ms at the median,
  45.61 ms at its slowest,
  and its slowest sample was 10.4 times its median,
  which is the spread this
  machine showed;
- a stand-in answering `which --toolchain <channel> rustc`:
  12.85 ms at the median,
  26.58 ms at its slowest;
- the same stand-in answering `component list --installed`:
  12.63 ms at the
  median,
  26.28 ms at its slowest.

So 2 s is 75 times the slowest ordinary question and 157 times the median of the
two questions,
and it is above the whole `list` command the README's own earlier measurement
took (20 to 116 ms).
The stand-in stands for `rustup`'s answer and says so;
no claim about a real `rustup` rests on it.

Controls:
`a_wait_that_passes_the_bound_says_what_is_being_waited_for`,
`a_wait_that_outlasts_the_bound_twice_is_still_named_once`,
`a_wait_that_ends_within_the_bound_is_silent`,
`a_programs_whole_output_is_read_even_when_it_is_larger_than_a_pipe`,
`the_shipped_bound_is_far_above_an_ordinary_answer`,
and `the_shipped_runner_names_a_program_whose_answer_passes_the_bound`,
which starts the shipped runner in a process of its own and reads the line from
its standard error.
That last one was added because the others pass their own bound:
with `A_WAIT_LONGER_THAN_ORDINARY` raised to an hour every earlier control stayed
green,
and only that one failed.
At the user boundary,
a run of this branch's build against a stand-in `rustup` that did not answer
printed the line once and then the answer.

Whether a deadline should end such a wait,
and at what bound,
is not decided here:
the owner decides,
and what each answer costs is in the report of the round that wrote this
section.

### L-4: a declared bin of 186 bytes or more failed every install

Every exposure link is written under
`.<bin>.tmp.<process id>.<start time>.<namespace>.<boot>`
and renamed over the bin's name,
so the temporary name carries about seventy bytes more than the bin.
`0.2.7-alpha.18` wrote `.<bin>.<process id>.tmp` and linked a 240-byte bin;
`0.2.7-alpha.19` fails such a link with
`cannot link <the temporary name> to <the target>: File name too long (os error 36)`,
naming a name no plan holds and no person wrote.
The pass measured the edge at 185 bytes linked and 186 bytes failed.

The name is now judged where the plan is read,
before any download or link write,
against the longest name asked of the file system the exposure folder is on
(`statvfs`,
so a file system holding 143 bytes and one holding 255 each answer
for themselves).
The reserve a temporary name adds is the widest each of its fields can be under
its own type
(a `u32` process number,
a `u64` start time,
a `u64` namespace and the boot
identifier's 36 bytes),
95 bytes,
so on a 255-byte file system a bin name is at most 160 bytes.
The refusal names the plan file,
the row,
the bin,
the limit the file system
gave,
the temporary name it cannot hold,
and the two ways out:

```text
mpm-shadow: the toolchain plan <plan file> declares the bin "<bin>" of rust, 186 bytes long, and the file system of the exposure folder <folder> holds names of at most 255 bytes, so the `.<bin>.tmp.<process id>.<start time>.<namespace>.<boot>` name its link is written under before it is renamed (at most 281 bytes for that bin) cannot exist there: a bin name that folder can hold is at most 160 bytes, so declare a shorter exposed name in `bins` and the program's own name in `options.binAliases`, or run this scope from a file system whose names hold more bytes
```

The command exits 2,
the code for a plan that cannot be used.
The name is refused wherever the plan is read,
so `list`,
`update`,
`install --lockfile-only` and `bin` refuse such a plan too,
which is the cost of putting the check there;
the way not taken is a shorter temporary name,
which gives up the identity the name carries:
which process,
which start of it,
which namespace and which boot wrote the link,
the facts a later rebuild needs to tell a link an install still holds from one an
install that ended left.
The owner has that veto open.

Controls:
`a_bin_the_folder_cannot_hold_is_refused_naming_the_row_the_bin_and_the_limit`,
`reading_the_plan_refuses_a_bin_the_exposure_folder_cannot_hold`,
`the_question_is_put_to_the_nearest_directory_that_exists`,
`a_plan_whose_bins_fit_the_exposure_folder_is_accepted`,
and the positive control of the failure this replaces,
`a_name_at_the_limit_is_written_and_one_byte_more_fails_with_the_systems_own_error`,
which writes a real link whose temporary name is exactly the longest name the
file system holds and one byte past it,
and reads `File name too long` with the temporary name in it from the system.
Refused at the user boundary:
`install --frozen-lockfile` on a scope whose plan declares a 186-byte bin
printed the refusal and exited 2.

### L-8, the part about these sentences: three controls that did not exist

The other three parts of L-8 belong to the round that owns the removal.
Here the finding was that no control held the sixth failure an install's publish
can reach
(the two-rename fallback whose second rename fails and whose put-back fails after
it),
and that nothing drove the kept copy into the two states the sentence names
beside the next install's own namespace:
another boot,
and a newer version of this repository's tool installed by then.

The plant that answers a rename now has a field of its own for the put-back that
fails after the refused rename,
which is what that sixth arm needs,
and three controls were added:

- `both_renames_failing_says_what_a_later_install_does_with_the_kept_copy`,
  which reads the sixth message's own words and then runs the install that
  follows,
  which publishes the copy without downloading;
- `a_copy_under_a_name_from_an_earlier_boot_is_published_by_the_next_install`,
  which puts the kept copy under a name carrying a boot identifier this machine
  is not running,
  and shows the next install publishes it;
- `a_newer_version_installed_leaves_the_kept_copy_hidden`,
  which finishes a newer version in the same root and shows the next install
  says `left the finished copy ... because 0.2.2 is installed and an older
  repository tool is never kept`,
  downloads nothing and leaves the copy.

Each was proved to fail by planting the behaviour it guards:
the sixth message's text replaced by the fifth's,
a name of another boot answered as unjudgeable,
and the recovery allowed to keep an older version.
