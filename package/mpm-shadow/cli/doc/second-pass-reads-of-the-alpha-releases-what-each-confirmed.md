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
