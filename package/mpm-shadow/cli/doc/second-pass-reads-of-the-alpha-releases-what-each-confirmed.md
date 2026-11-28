# Second-pass reads of the alpha releases: what each confirmed

What the second pass over each alpha release of this package confirmed with the
released binaries,
one section per release,
what a read found where a round acted on its findings,
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

## The alpha.19 read: what it found, and what the round after it changed

The pass compared `0.2.7-alpha.19` (release commit `1cd66ac8df`) with
`0.2.7-alpha.18` (release commit `39e60f48cc`),
with the two released binaries,
and by reading the code at both commits;
it built nothing.
Its report and transcripts lived in a scratch folder that expires a day after its last change,
so every figure below is restated from them here,
and stands without them.
The pass is another agent of the same vendor as some of this package's authors,
so it is a second pass,
not an independent review.

The round after it holds one control per change in
`src/toolchain/the_farm_removal_spares_and_names_tests.rs`
(the branch `fix/mpm-shadow-cli-mpm-shadow-alpha-19-read-two-installs-at-once-fail-neither-and-the-removal-names-what-it-keeps`).
The red each control shows is the released code's behaviour,
not the pass's:
every one was run with the release's own code in the tree and the fix taken out,
and eight of that file's controls failed then.
Each part below says which.

Status:
**unapproved and non-final**.
The owner declares completion.

### M-1: two installs at once still failed each other at two of the removal's three calls

The pass held one install with `strace` at a call of the removal while the other ran,
on `0.2.7-alpha.19` and on `0.2.7-alpha.18`.
Where the held install was stopped at its `lstat` of the other install's temporary link,
the other install renamed that name away,
and the held install exited 1 with
`toolchain install: exposure directory failed: cannot read <farm>/.thirdbin.tmp.<pid>.<start>.<ns>.<boot>: No such file or directory (os error 2)`
and `1 failed`.
Stopped at its `readlink` of an orphan,
it exited 1 with `cannot read the link <farm>/orphan: No such file or directory`;
stopped at its `lstat` of the same orphan,
with `cannot read <farm>/orphan: No such file or directory`.
The two races the earlier round had built,
at the unlink of an orphan and at the rename that gives a temporary link its name,
both ended 0 for both installs in the same runs,
so the release held its claim at one call of three.
The folder was right after each failure,
and a rerun passed.

Only the removal's own `unlink` read `NotFound` as "the other run took it";
the two reads before it failed the whole rebuild.

Changed:
the read of an entry's own metadata and the read of the link's target treat `NotFound` the way the
unlink does,
so the name leaves the pass unnamed and unremarked,
and the run that took it is the one that named it.

Controls:
`a_link_another_run_took_before_this_run_read_it_is_not_a_failure` and
`a_link_another_run_took_before_this_run_read_its_target_is_not_a_failure`,
each with a plant at the call the pass held.
On the release's code they fail with
`exposure directory failed: cannot read /.../orphan: No such file or directory (os error 2)` and
`exposure directory failed: cannot read the link /.../orphan: No such file or directory (os error 2)`;
with the fix they pass.
The unlink's control,
`a_removal_of_a_link_another_run_took_first_is_not_a_failure`,
already existed and still holds,
so all three calls now have one.
The plant is compiled out of every build that is not a test,
as the link write's own plant is
(`doc/decision/cli-git-test-only-switches-compiled-out-of-release.md`).

### L-1: the removed temporary link's sentence claimed an author it cannot know

The pass planted a person's own dotted links of the temporary shape.
A link named `.notes.4190000.tmp` (a free number) and one named
`.mine.tmp.4190001.100.<this namespace>.<this boot>` were removed and named
`removed the temporary link ... which an install that ended left before it gave the link its name`,
which the tool cannot know:
nothing in the folder tells a person's link from an install's.

Changed:
the sentence says what the name is and what the run could show of the process it records:

```text
removed the temporary link <name> (it pointed at <target>), a name of the shape an install writes while it holds a link, and the process that name records is not running
```

The rule that removes such a link is unchanged;
only the sentence moved to what was seen.
Control:
`a_sync_says_of_a_removed_temporary_link_only_what_it_could_show`,
which fails on the release's sentence and passes on this one.

### L-2: a temporary link no run can judge stayed for ever and no line named it

The pass left `.ddd.tmp.<pid>.<start>.<other namespace>.<boot>` and
`.hhh.tmp.<pid>.0.<namespace>.<boot>` in a farm and ran an install,
`list` and `prune --toolchain --dry-run`:
the links stayed,
and no line named them.
A kill at the rename of a `current` move left
`.current.tmp.<pid>.<start>.<ns>.<boot>` in `<root>/fixtool/`,
and the next install,
a second one,
and a real `prune --toolchain` all left it too,
because the exposure folder is the only directory with a removal pass.

Changed:
the rebuild names each temporary link it cannot judge once per run,
and says what deleting it by hand would risk,
as `prune --toolchain` names a hidden copy:

```text
toolchain install: left the temporary link <name>: this run cannot judge the install its name records, so it is not removed, and deleting it by hand may make that install fail where it gives the link its name
```

Controls:
`a_sync_keeps_and_names_a_temporary_link_an_install_in_another_process_namespace_holds`,
`a_sync_keeps_and_names_the_earlier_builds_temporary_link`,
`a_sync_keeps_and_names_the_earlier_builds_temporary_link_while_its_number_is_held`,
`a_sync_keeps_and_names_the_temporary_link_of_a_run_that_could_not_read_its_own_place`;
all four fail on the release,
which left the names and said nothing.

Left:
the tool folders have no removal pass.
The README's part on the upgrade from a release that removes temporary links by number,
and this file's last section,
say what such a rule would have to prove.
`list` and `prune --toolchain` still say nothing about a temporary name in the exposure folder;
only `install` names one.

### L-3: the program's own link was kept for any install of any tool

The pass built a farm in which no `mpm-shadow` was installed,
and the link named `mpm-shadow -> <root>/fixtool/1.0.0/mpm-shadow`,
a file of that name inside another tool's install directory.
The rebuild kept it,
where the README says a program installed nowhere has its link removed.

Changed:
the link is kept only while it resolves into an install of `mpm-shadow` itself,
which is the program's own install directory below either toolchain root:
`<root>/mpm-shadow` for a release archive,
or `<root>/cargo/monochromatic-mpm-shadow` for the cargo route
(`env!("CARGO_PKG_NAME")`).

Controls:
`a_sync_removes_and_names_the_farms_mpm_shadow_link_into_another_tools_install` (red on the
release),
and four that hold the rest of the rule:
`..._that_resolves_nowhere`,
`..._to_a_program_that_belongs_to_no_install`,
`..._that_resolves_into_the_other_toolchain_roots_install`,
`..._that_resolves_into_the_cargo_routes_install`.
The pass found no test that held a removal of this link at all;
the first three of these now do.

### L-9: mixed releases, and an earlier name that records no namespace

The pass ran one install of each release against the other,
and one in a real second process namespace through a container that shared the farm:

- Released `0.2.7-alpha.18` held at the rename that gives its temporary link its name while
  `0.2.7-alpha.19` rebuilt the farm:
  the earlier remover took the live link and named it,
  and the held install exited 1 with
  `cannot place the link <farm>/thirdbin: No such file or directory`.
- Released `0.2.7-alpha.18` held at an orphan's unlink while `0.2.7-alpha.19` removed the same
  orphan:
  the earlier install exited 1.
- `0.2.7-alpha.19` in the container,
  its number free in that namespace,
  took the host's live `0.2.7-alpha.18` link `.thirdbin.<pid>.tmp` and named it as one an install
  that ended had left,
  which was false,
  while the host's install exited 1.

Changed,
in two parts:

- The README now says what an earlier release's remover still does,
  and that only a farm every session rebuilds with this release is past the window.
  This repository's lock has named `0.2.7-alpha.19` alone since 2026-11-27
  (commit `ba118ea127`),
  so this machine's own scopes are past it.
- An earlier release's name,
  and this build's shorter name for a run that cannot read its own boot or process namespace,
  record no process namespace at all,
  so no run can show that the process they name wrote them,
  or was ever in this namespace.
  Both now stay and are named,
  and the process-number rule that judged them is gone with `the_number_is_held`.
  That is the rule a hidden copy's unjudgeable name already followed,
  which `README.md` states under "What a kill leaves and what the next install does about it".

### What a removal rule for the tool folders would have to prove

The pass's L-2 says a temporary link of any other atomic link write survives every run,
because the farm is the only directory with a removal pass:
`src/toolchain/install_support/the_atomic_symlink.rs` writes the same shape for the
`current` links (`src/toolchain/install_archive.rs`,
`src/toolchain/published_archive.rs`,
`src/toolchain/cargo_installs/which_version_runs.rs`),
the postinstall links (`src/toolchain/exposure/the_postinstall_steps.rs`),
the old-copy links (`src/toolchain/old_copy_apply.rs`) and the cargo links.
It is not this round's,
because a rule that removes them has to prove,
for each name:

- that the name is this tool's and not a person's:
  the shape alone cannot,
  which L-1 shows for the farm;
- that the install the name records has ended,
  by the four facts the name carries,
  and never by a guess,
  since a name from another process namespace or with no namespace at all names an install that
  may still be running;
- that the name holds no link a later step reads:
  `.current.tmp.<facts>` beside `<tool id>/current` is read by every launcher record and every
  version move,
  and the folder it sits in is a version directory's parent;
- which directories it may touch at all.
  `<root>/<tool id>/` and `<root>/cargo/<crate>/` are this tool's;
  `~/.local/bin`,
  `$CARGO_HOME/bin` and every checkout's `.cache/*-bin` are not,
  and a removal there would take a name this tool cannot show it wrote;
- that the two commands that read those folders agree:
  `prune --toolchain` already names what it leaves,
  so an install's pass would have to name the same names in the same words,
  or one of the two would hide the other's.

## The alpha.19 pass: the four sentences and the one name length this round took

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
The section above holds the exposure folder's removal,
which another round took;
this one holds the sentences an install prints and one name length:
L-4,
L-5,
L-6,
L-7 and the part of L-8 about those sentences.
The exposure folder's removal
(M-1 and L-1,
L-2,
L-3 and L-9)
belongs to that other round,
`mpm-shadow-alpha-19-read-two-installs-at-once-fail-neither-and-the-removal-names-what-it-keeps`,
and this section says nothing about it.

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

Every program a toolchain command runs through the runner is now waited on
through one function,
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
That is the failure this check replaces;
the length at which the check refuses is a different number,
written below with the commands it reaches.

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
The edge is 160 and 161:
measured on 2026-11-28T06:46:53-05:00 with the released `0.2.7-alpha.20`
on a fixture whose plan and lock declare one `rust` row,
a bin of 160 bytes installs and every command run exits 0,
and a bin of 161 bytes is refused.
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
which was measured on 2026-11-28T06:46:53-05:00 with the released
`0.2.7-alpha.20` against a plan declaring a 161-byte bin:
`install`,
`install --lockfile-only`,
`list`,
`update`,
`outdated` and
`verify-archive` exit 2 with the refusal and nothing on standard output,
and `bin` (plain and `--json`),
`prune`,
`prune --toolchain`,
`--version` and `--help`
run as before.
`bin` and `prune --toolchain` never read the plan,
so a person whose plan holds such a bin can still print the exposure folder and the IDE file;
the commands that cannot run are the ones that read the plan,
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

## The alpha.20 read: what it found, what this round changed, what stays open (read begun 2026-11-28T06:18:35-05:00)

Status:
**unapproved and non-final**.
Written by the round that answered the read,
not by the read itself.
The read was a second pass by Claude Sonnet,
the same vendor as some of this package's authors,
so it is not an independent review;
its report and transcripts lived in a scratch folder that expires a day after
its last change,
so every figure taken from it is restated here as a fact,
and this section is what a later reader has.

The read compared the released `0.2.7-alpha.20` with `0.2.7-alpha.19`
(`monochromatic-mpm-shadow-v0.2.7-alpha.20` and `...-alpha.19`,
`x86_64-unknown-linux-gnu`):
the binaries' SHA-256 values are
`4269ffeaca702bd4eb63204b7d5ef963ae690aa1ca4eec0ee0fc00b9b9ddc58d` and
`3305ddff09ec99775d1db7d22e05cc9f96316421073a7ea294bf49e86ffdfe12`,
and the alpha.20 archive's is
`7a12d5df3173bb45f713f9f673db7621b7af3c4c5bb5d49e26930bf239d41a46`,
the value the lock's `linux-x64` row recorded.
This round downloaded the same two archives and checked both hashes before
running anything,
so its fixtures and the read's used the same binaries.

Every run of either released binary,
and of this branch's build,
went through `systemd-run --user --scope --slice=agent-work.slice` with
`MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight=1` and `RuntimeMaxSec`,
with a throwaway `HOME`,
`XDG_CONFIG_HOME`,
`XDG_DATA_HOME` and `PATH`,
no `DBUS_SESSION_BUS_ADDRESS`,
and a stand-in `rustup` (a Node script with the
bare name `rustup` on `PATH`,
which logs every call it is given).
Every run that was expected to abort was started through
`prlimit --core=0`,
so no core dump was stored.
No network was reached and no real `rustup`,
`cargo`,
`pnpm` or `go` was
started:
the stand-in stands for each,
and no claim below about a real
`rustup` rests on it.

### F-M1: the wait line ended the whole install when standard error refused it

Changed:
`src/toolchain/runner.rs` now writes the wait line through one new writer,
`src/notices.rs` `say`,
which drops whatever the write answered,
instead of `eprintln!`,
which panics on a write error;
this crate's release profile sets `panic = "abort"`,
so the panic was `SIGABRT`.
The same writer now carries every other notice this crate says about its own
work (the list is below).

Reproduced with both released binaries on a fixture whose plan and lock hold one
`rust` row whose toolchain the stand-in answers,
with the stand-in's `which` taking 3 s,
so the wait passes the shipped 2 s bound:

- standard error a file (the control arm):
  both releases exit 0,
  both call the stand-in twice,
  and alpha.20 prints the wait line twice (314 bytes on standard error) while
  alpha.19 prints nothing;
  standard error `/dev/full` (open,
  answering `ENOSPC`):
  alpha.19 exits 0 after 2 calls;
  alpha.20 ends with `SIGABRT` 2.048 s after the start,
  having called the stand-in once,
  with nothing on standard output;
- standard error a pipe whose read end this side closed at 0.5 s (`EPIPE`):
  alpha.19 exits 0 after 2 calls;
  alpha.20 ends with `SIGABRT` 2.057 s after the start after one call;
- standard output and standard error one pipe whose reader left at 0.5 s:
  both releases end with `SIGABRT`,
  alpha.19 at 6.1 s after both calls and alpha.20 at 2.056 s after one,
  which is a second,
  older defect (see "What stays open",
  the last item).

This branch's build in the same four states:
0 in the first three (twice two calls,
the wait line written in the control arm)
and `SIGABRT` only in the fourth,
where it is the report on standard output rather than a notice that the stream
refuses.

Controls:
`a_wait_notice_a_full_standard_error_refuses_does_not_end_the_command` and
`a_wait_notice_a_departed_reader_refuses_does_not_end_the_command`,
each of which starts the shipped runner in a process of its own with a standard
error of one of those two kinds and asserts the fixture ran and passed.
Both are green now;
with `notices::say` put back to `eprintln!` both fail,
the fixture's own process ending 101 with the panic
that the shipped release build turns into `SIGABRT`.

#### Every other notice this crate says while it works

Each line below was an `eprintln!` in `0.2.7-alpha.20`
(line numbers are the release's),
and each now goes through `notices::say`:

  `src/toolchain/runner.rs:339-341`,
  `say_on_standard_error`:
  the wait line.
  It ended an install in the two stream states above.
  `src/toolchain/commands/what_every_verb_prints.rs:112`,
  `warn_older_locks`:
  the older-lock warning that `install --lockfile-only`,
  `update`,
  `outdated`
  and `bin` print and then succeed.
  Measured at the user boundary on a machine-global scope whose lock pins
  `0.2.7-alpha.19` while `0.2.7-alpha.20` is installed:
  `bin --global` on alpha.20 wrote its 215 bytes of JSON on standard output,
  then met `ENOSPC` on the warning and ended `SIGABRT`;
  this branch's build exits 0 with the same JSON and no warning.
- `src/toolchain/archive_staging.rs:493`:
  the warning that a policy refused the one-exchange publish,
  after which the install goes on in two renames and succeeds.
- `src/toolchain/prune_cargo/the_prune_itself/the_hidden_installs_it_names.rs:278`:
  the line naming a hidden copy `prune --toolchain` cannot judge and leaves,
  after which the prune goes on and ends 0.
- `src/prune.rs:198`:
  the line for a cached copy that could not be deleted,
  which the function's own doc says must not change the run's exit code.
- `src/shim.rs:334`:
  the line that says a start was refused for lack of processes and then
  succeeded,
  printed before the child is waited for.
- `src/shim.rs:397`:
  the post-run cleanup summary,
  printed before the shadow returns mpm's own
  status.
  `src/toolchain/launcher/starting_it.rs:179`,
  `:187`:
  the two lines saying why a lock could not be read,
  printed before the tool is
  started.
- `src/toolchain/launcher/starting_it.rs:280`:
  the older-lock warning a launcher prints before it replaces itself with the
  program.
- `src/toolchain/launcher/starting_it.rs:288`:
  the warning that a launcher record could not be rewritten,
  printed before the
  start.

Left as `eprintln!`,
because the command is already failing and the line is its
failure report rather than a courtesy:
`src/toolchain/commands/what_every_verb_prints.rs:68` (`report`),
`src/cli.rs:256` (neither `CARGO_HOME` nor `HOME` is set),
`src/shim.rs:476`,
`:497`,
`:504`,
`:536` and `:542` (the shadow's start
refusals and failures,
which end 126 or 127),
`src/toolchain/launcher/starting_it.rs:294`,
`:312`,
`:316`,
`:326`,
`:330`,
`:333`,
`:374` and `:378` (a launcher's refusals and missing program,
which end
126 or 127),
and `src/toolchain/prune_cargo/the_prune_command.rs:53` and `:80`.
In those states a stream that refuses the line turns a documented failure into
`SIGABRT`,
which is a change of the reported status and not of the outcome;
no record promises otherwise,
and this round changed none of them.

### F-L2: the refused bin names and the commands that refuse them

Changed:
the record,
not the code.
The release's behaviour is right and the record named the wrong commands.

Measured on 2026-11-28T06:46:53-05:00 with the released `0.2.7-alpha.20`,
on a fixture whose plan and lock declare one `rust` row:

  a bin of 160 bytes:
  `install --lockfile-only`,
  `install --frozen-lockfile`,
  `list`,
  `update`,
  `outdated` and `bin` exit 0;
  `verify-archive` exits 2 for an absent artifact,
  which is its own refusal.
  a bin of 161 bytes:
  `install --lockfile-only`,
  `install --frozen-lockfile`,
  `list`,
  `update`,
  `outdated` and `verify-archive` exit 2 with the plan
  refusal and nothing on standard output;
  `bin` exits 0 and prints the exposure folder,
  and `prune --toolchain --dry-run` fails on the fixture's absent `.git`
  rather than on the plan,
  so neither of those two reads the plan.

The record's L-4 section said `list`,
`update`,
`install --lockfile-only` and
`bin` refuse,
and left out `outdated` and `verify-archive`;
it now names the measured set and says that `bin` and `prune --toolchain` never
read the plan.
The same section said the pass measured the edge at 185 bytes linked and 186
failed without saying that 186 is the failure the check replaces rather than the
length the check refuses;
it now says both,
with 161 as the first refused length.

### F-L4: "every rebuild names it once" and a rebuild that stops

Changed:
the record,
not the code.

The rebuild's pass stops at the first removal that fails,
and a control holds that:
`a_run_names_the_removals_it_made_before_a_later_one_failed` plants two orphan
links,
refuses the second removal with `EIO`,
and asserts that the second link is still there afterwards,
which can only be true if the pass stopped.
So the sentence "every rebuild names it once" was the side that was wrong.
`README.md` and the module doc of
`src/toolchain/exposure/the_links_no_row_declares.rs` now say that a rebuild
names the names it reached before that removal and that the next install names
the ones after it.

The kept names a rebuild did reach survive a later failure:
`report.exposure_links_left` is assigned before the failure is recorded,
so a run that fails after judging a name still prints its line.

### F-L5: the doc comment of `the_link_the_farm_keeps`

Changed:
the code's comment,
not its behaviour.

The comment said a link into another tool's install directory,
a superseded version,
or a dead installer's staging directory is not kept;
the code keeps the last two and removes the first,
because the question it asks is which install directory the resolved file sits
in and never which version it is.
`README.md` already said exactly that ("A rebuild keeps it while it resolves
into an install of `mpm-shadow` in either toolchain root"),
and the test
`a_sync_keeps_the_farms_mpm_shadow_link_that_resolves_into_the_cargo_routes_install`
holds the two install directories the code compares.
The comment now says what the code does and why the version does not matter.
The read's two planted states (a link onto an older version directory,
and one
onto a `.staging` copy inside the program's own install directory) are still
kept,
and no test holds either;
that is in "What stays open".

### F-L10: "every program a command starts" is wider than the code

Changed:
the record,
and one sentence of `README.md`.

Three starts do not go through the runner and print no wait line:
the shadow mode's start of the real `mpm` (`src/shim.rs:265`,
standard streams
inherited),
a launcher's replacement of itself with the program its record names
(`src/toolchain/launcher/starting_it.rs:91` and `:106`),
and the repository-link check's own program under its own deadline
(`src/toolchain/repository_link_cause.rs:242`).
Each is right:
a pass-through has nothing to report,
and the link check ends the program it started itself.
The record's L-7 section and the README's `list` bullet now say
"every program a toolchain command runs through the runner",
and the README names the three starts that do not.

### F-L11: three controls the records claimed and no test held

Added:
`a_sync_names_each_kept_temporary_link_once_in_name_order_after_the_removals_it_made`,
`a_bin_of_the_longest_fitting_length_is_accepted_and_one_byte_more_is_refused`,
and `the_shipped_bound_is_the_two_seconds_the_record_promises`.

Each was green with the code unchanged and red under the edit the earlier
controls passed:

- count and order of the kept lines:
  a second `left.push(name.clone())` fails the new control's count while the
  seven keep-and-name controls and the removal-order control stay green;
  printing the kept lines before the removal lines fails its order while the
  same eight stay green.
- the 160 and 161 byte edge:
  `bytes <= most` changed to `bytes < most` fails the new control while the five
  earlier controls of this check stay green.
  The control derives its edge from the file system's `f_namemax` and the
  check's own reserve,
  asserts the reserve covers the temporary name this machine writes,
  and refuses a two-byte-character name whose characters fit and whose bytes do
  not.
- the 2 s bound:
  `A_WAIT_LONGER_THAN_ORDINARY` set to 600 ms fails only the new control;
  `the_shipped_bound_is_far_above_an_ordinary_answer` (twenty times an ordinary
  answer) and
  `the_shipped_runner_names_a_program_whose_answer_passes_the_bound` both stay
  green.

Removing `left.sort()` alone still leaves the count-and-order control green on
this machine,
because its file system lists the two planted names in name order already;
the control's order assertion covers the sort only where a listing is unsorted,
and the report of this round says so.

## What stays open after the alpha.20 read

Seven items,
each with what the released binaries showed and what each answer would cost.
None is fixed in this round,
and the owner decides.

### A temporary link an earlier build wrote, or this build's shorter name, is named on every install for ever

Evidence:
a farm holding `.<name>.<pid>.tmp` (the shape `0.2.7-alpha.18` wrote) and
`.<name>.tmp.<pid>.<start>` (this build's shape for a run that cannot read its
own boot or process namespace) keeps those names through ten installs in a row,
each install printing one line per name;
`list` and `prune --toolchain` print nothing about them.
A reboot clears neither:
the name carries no boot,
so no run can show its
installer ended.
`0.2.7-alpha.19` removed such a name once no process held its number,
and said nothing about the ones it kept.
The writing of the shorter shape is not hypothetical:
a run whose read of `/proc/sys/kernel/random/boot_id` is refused writes
`.thirdbin.tmp.<pid>.<start>`,
and every install after it names it.
A person's own `.<name>.<n>.tmp` link joins the list.

Two answers:
remove such a name once the link's own modification time is older than this
boot's start time (`/proc/stat` `btime`),
which clears them at the first install after a reboot and keeps every name a
live install of another namespace may hold,
at the cost of one `btime` read and a rule that a link with no readable time
stays for ever;
or keep the shipped rule,
which never deletes what it cannot judge,
at the cost of one line per name on every install for ever,
a reboot included.
Ranking:
the `btime` rule > shipped keep-and-name > removal by free number,
because the first ends the lines without deleting anything a live install
holds,
and alpha.19's rule deleted a live install's link in another namespace.

### A bin name that is long only together with its folder is not refused

Evidence:
a scope directory 3907 bytes deep with a 100-byte bin (its link path 4029 bytes,
its temporary link's about 4099,
`PATH_MAX` 4096) fails the install with exit 1
and `cannot link <...>/.bbb...x.tmp.<...>: File name too long (os error 36)`,
which is the failure the check was written to replace;
`list` and `bin` exit 0.
The check asks `statvfs` for the folder's name limit and never the path limit.
Both releases behave the same way.

Two answers:
ask the kernel about the whole path as well (a link write's own `ENAMETOOLONG`
is the only question that covers every component,
so a dry construction of the
temporary path's length against `PATH_MAX` and the folder's own spelling would
be the check),
at the cost of a second rule that can refuse a scope whose folder is later
shortened;
or say in `README.md` that the check is of the name alone,
at the cost of leaving the deep-checkout failure where it is.
Ranking:
the README sentence > the path check,
because a path check that refuses a plan which links today is
what the read objected to at the name limit.

### The 2 s line also prints for programs that routinely run longer

Evidence:
a `rustup component add`/`target add` that takes 3 s prints one line per add
(on a fixture,
three adds printed at 2.09 s,
5.17 s and 8.20 s);
the same `rustup which ...` called twice by one install prints the identical
line twice,
each saying "2.0 s after this command started it".
The 2 s bound was measured on read-only questions (this machine's
`rustup --version` and a stand-in's `which` and `component list`);
the installers the same runner starts (`rustup component add`,
`target add`,
`cargo binstall`,
`pnpm add`) download for longer than that in an ordinary run.

Two answers:
one bound for read-only questions and a larger one for installers,
at the cost of two constants and a second measurement;
or one bound above the slowest ordinary installer,
at the cost of a wait on a stuck read-only question going unnamed for longer.
Ranking:
the two bounds > one larger bound,
because the questions differ by an order of magnitude and one bound must be
wrong for one of them.

### An install by `0.2.7-alpha.19` still fails beside one by `0.2.7-alpha.20`

Evidence:
with alpha.19 as the install that reads while alpha.20 runs,
alpha.19 exits 1 with
`cannot read <farm>/.thirdbin.tmp...: No such file or directory` (held at its
`lstat` of alpha.20's temporary link),
and at its `readlink` and its `lstat` of a killed install's leftover that
alpha.20 removes;
the held calls answer `ENOENT`.
Two alpha.20 installs fail neither,
and two alpha.19 installs fail each other,
which is the failure alpha.19's own read reported.
The repository's lock named `0.2.7-alpha.19` until
2026-11-28T04:44:29-05:00 and names alpha.20 since.

Two answers:
one sentence in `README.md` naming the alpha.19 window and that a session on
alpha.19 fails against one on alpha.20,
at the cost of nothing but words;
or a release note and a lock bump everywhere alpha.19 still runs,
at the cost of a release for a binary that no change here can fix.
Ranking:
the README sentence > the release,
because nothing in this crate can change what alpha.19's remover does.

### Two locks in one folder: an install removes a link the other just made

Evidence:
install A is held before its removal pass while install B,
whose plan and lock
declare one more tool,
links that tool;
A,
released,
removes the link as one "no row of the lock declares";
both exit 0,
and B's tool is unreachable until its next install.
The same in all four pairs of the releases.
The claim the removal's rule makes is about one lock,
so this is outside it,
and it needs two locks in one scope (a branch switch during an install,
or a
folder shared by trees with different locks).

Two answers:
say in `README.md` that two locks in one exposure folder are not supported,
at the cost of a state a person can reach without knowing;
or judge a name against every lock `git worktree list` reports,
at the cost of a `git` call in every install and a new failure mode when git
cannot answer.
Ranking:
the README sentence > the second lock set,
because the second adds a slow call to the common path for a state only two
concurrent sessions with different locks reach.

### A killed move of `current` leaves a temporary link nothing removes or names

Evidence:
`.current.tmp.<pid>.<start>.<namespace>.<boot>` in a tool folder stays through
the next install,
a second install,
`prune --toolchain --dry-run` and `list`;
no line names it,
in either release.
The record says the tool folders have no removal pass;
`README.md` says nothing about it.

Two answers:
one `README.md` sentence saying what such a name is and that it is left,
at the cost of nothing but words;
or a removal pass for the tool folders,
at the cost of the four proofs the record's earlier section lists
(is it this tool's name,
has its installer ended,
does a later step read it,
which directories may it touch).
Ranking:
the sentence > the pass,
because the pass has to answer four questions for names this tool's own writes
leave in folders that also hold other things.

### The report on standard output aborts when that stream refuses it

Evidence:
with standard output and standard error one pipe whose reader left 0.5 s after
the start,
`0.2.7-alpha.19`,
`0.2.7-alpha.20` and this branch's build all end
with `SIGABRT`;
the report lines go out with `println!`,
which panics on a write error as
`eprintln!` did.
The work the report describes has already finished there,
so the abort changes
what a caller learns,
not what happened.

Two answers:
write the report through a writer that drops the refused answer,
at the cost of a caller that cannot tell a report was refused;
or keep `println!`,
and say that a caller which stops reading its output ends
the command,
at the cost of a state a pipeline reaches without meaning to.
Ranking:
the sentences above are unasked for by the read;
the round left `println!` alone and put the choice here.

## What this round did not run

- The six open items above were not re-reproduced:
  their states need a real second process namespace,
  a reboot,
  a deep checkout
  or two released binaries held at a call,
  and the read's own fixtures for them are the evidence restated above.
  `cargo binstall`,
  `pnpm add`,
  `uv tool install`,
  `go install` and a real
  `rustup`:
  every program the runner starts was a stand-in.
  A file system whose name limit is below 255 bytes:
  that needs a mount.
- The three quarters of the notice sites checked only by reading and by the
  shared writer's two controls:
  the older-lock warning was run at the user boundary,
  and the other seven sites were not.
