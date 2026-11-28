# What the wrapper's release passes found

The first part is the round of 2026-11-27 over `0.6.1-alpha.48`.
The part about the alpha.52 second pass is the round of 2026-11-28
over `0.6.1-alpha.52`.
Each part is unapproved and non-final.

## The alpha.48 release passes

Round of 2026-11-27,
unapproved and non-final.

Two passes read the release `0.6.1-alpha.48`:
a behaviour pass,
which exercised alpha.47 and alpha.48 side by side,
and a splits pass,
which read what the 344 refactor commits of the range did.
This record keeps the findings and the measurements that no other file holds,
in the passes' own words where they had them.
Every figure is the pass's own measurement,
taken on 2026-11-27;
nothing here was re-measured for this record.

The behaviour the release rests on is the code,
the gate,
and the service-manager install proof named under
"No change of behaviour is not demonstrated" below.

## The behaviour pass found no High finding

Neither pass raised a High finding.
The behaviour pass's two Mediums and six Lows were all fixed by rounds merged on 2026-11-27,
each with a control that fails without the fix;
what each said is history,
and the controls pin the fixed words.

The splits pass's Mediums and Lows are below.

## The behaviour pass: what held

These held,
exercised with both binaries:

- the slot refusals,
  for the spellings they name;
- the merge pass's union of deletions;
- the self-check's memory,
  62.7 to 66.1 MB in alpha.47 and 9.3 to 11.0 MB in alpha.48;
- named pipes,
  answered and not waited on;
- the typed ssh relay at a terminal,
  which ended about 100 ms sooner.

## The behaviour pass: read and not run

Three surfaces were read only,
because nothing outside the process can start them:
the prune race,
the conversion environment's precedence,
and the service-only changes.

## The splits pass: no change of behaviour is not demonstrated

Its verdict is that "no change of behaviour" is not demonstrated.
Of the items compared at both ends,
97.9 percent have identical text,
no test-only switch is in either release binary,
no static became two and no trait impl was lost;
but 318 of the 344 refactor commits carry changed lines that are not moves,
and the pass read three of them in full.

Neither pass ran `git commit` in a release binary,
because without a session bus every commit form stops at one refusal.
So for the service paths the evidence is the reading,
the gate,
and the service-manager install proof,
which issue 6587's second round ran to its end (`fdd9ce0e83`):
alpha.48 installed over alpha.47 in a container of the gate image,
and the proof printed

```text
THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once
```

## The splits pass: reach

At least 31 definitions became nameable from outside the crate,
seven of them private helpers a split made bare `pub`,
one of which removes directories.
The list of published names is
`src/native/source_guard/items_nameable_from_outside_release.txt`,
and its control is
`src/native/source_guard/nothing_becomes_nameable_from_outside_tests.rs`,
which fails on a published name that has no line there.

## The splits pass: refactor commits read and not read

The extract-function refactors are unread but for ten commits.
A second read of the seven the pass named first,
all on the client's and the service's commit paths,
found each equivalent by reading
(`e90482f100`,
`dde5c4be3b`,
`bad45d0420`,
`01c2f32e2c`,
`6a24c6342b`,
`f4e8d2fb9b`,
`8af8f56255`).
One guard covers less than its name says:
`src/native/source_guard/hook_self_invocation_tests.rs` reads 263 of the 699 code lines
the commit preparation spans,
which the list-guards round took first.
About three hundred refactor commits stay unread.

## The splits pass: guards that name moved files

The pass found 71 pairs of a source guard and a file it names,
where code left the file in a split and the guard does not name where it went.
Two were checked,
and neither was a hole.
The list-guards round holds the pairs now.

## The splits pass: low findings

Eight points,
as the pass put them:

- the sweep's warnings carry four new log tags;
- a commit labelled `docs` changed the `shutdown-unmount` help row
  (`066e33a47d`,
  noted on the commit);
- the panic notice's source locations moved;
- 34 items lost a platform gate on split-off files,
  with no effect on x86_64 Linux;
- tag constants are copied into more files;
- the README's allow-list counts are stale;
- two commits in the range cannot build alone;
- the linter the binary carries moved four alphas.

The platform gates are fixed:
a split-off file keeps the platform gate of the file it left (`4d6f09f1f2`).

## The splits pass: a split that was not a pure move

Not every split is a pure move.
The landing split's last commit (`3b9ada357b`) boxed the commit inside the service's answers
and made the commit command pass its own copy of its client configuration,
both for Clippy after the move,
which the release pass that reads the range reads as a change.

## The read of `0.6.1-alpha.52`: the surface control and three source guards

Round of 2026-11-28,
unapproved and non-final.
The sections above are the `0.6.1-alpha.48` passes;
this one is the read of `0.6.1-alpha.52`
that the session then weighed,
and the rounds that answered two of its Low findings.
Only the owner declares it complete.

The read is a second pass by another agent of the same vendor as some of the authors,
so it is no independent review.
Its limits are the ones it states:
it did not build the crate,
it ran the reader functions copied byte for byte in two tiny `rustc` programs,
and it searched for each shape in the tree rather than planting one.

### What the read found on the surface control

`nameable_items_of_a_source_tree_tests.rs` compared the whole of an attribute with the text
`#[macro_export]`,
took every line starting with `#[` for an attribute,
and read an `impl`'s type from the `impl` line alone.
So it answered nothing for three shapes,
none of them in the crate on 2026-11-28,
and its own doc,
which says what the reading cannot see,
named none of them:

- `#[macro_export(local_inner_macros)] macro_rules! name`;
- an attribute and a `pub` item on one line,
  `#[inline] pub fn f() {}`;
- an `impl<` whose generics run over several lines,
  which is the header a long generic list is
  formatted with.

Each shape published a name with no line in `items_nameable_from_outside_release.txt`,
which is what that list exists to stop.
The read's suggestion was to plant each shape in a fixture and see the control red,
or to list the three in the reader's doc.

### What was planted

Commit `359aaf32e1` adds one control beside `surface_shapes_a_split_writes_tests.rs` per shape,
each writing a fixture tree and asking for the name the shape publishes.
Threaded,
with the guide's flags,
they failed on that commit:

```text
a_macro_export_with_parentheses_is_a_name
  left: ["outer macro_rules plain"]
 right: ["outer macro_rules local", "outer macro_rules plain"]
an_item_on_its_attributes_line_is_a_name
  left: ["outer fn plain"]
 right: ["outer fn inlined", "outer fn plain", "outer macro_rules exported", "outer struct Held"]
an_impl_whose_generics_run_over_several_lines_publishes_its_members
  left: ["outer field Published::held", "outer struct Published"]
 right: ["outer field Published::held", "outer fn Published::seen", "outer struct Published"]
```

`test result: FAILED. 18 passed; 3 failed`.

### What the reading reads now

Commit `819bc743ee` reads all three,
so the reader's doc lists exactly the shapes it still
cannot see and none of these.
An attribute on the declaration's line is split from it by `leading_attributes`,
which counts square brackets and skips string literals;
`is_a_macro_export` reads the attribute with or without its parentheses;
and an `impl` whose own generics run over several lines is held as one header text until its
`{` arrives,
and the type is read from all of it.
`nothing_becomes_nameable_from_outside_tests` compares the whole real surface against the list
green,
so the widening published nothing and the list needs no new line.

The same split moved into `source_scan_declaration_tests.rs` in commit `57d381c4cc`,
because that file owns the attributes a declaration line carries.
It found a hole of the same shape one layer down:
`#[cfg(test)] pub mod support;` was swallowed as an attribute,
so the module and every name it publishes were outside the module tree,
and outside every guard whose scope that tree decides,
with nothing said.
The control in `source_scan_checks_tests.rs` was shown red without the split:
`left` held no `gated_on_its_attribute_line` and no `renamed_on_its_attribute_line`,
`right` held both,
and the same run failed `declarations_refuse_what_they_cannot_place` on a `mod` visibility the
reading does not place,
written on its attribute's line.

### What the read found on the three guards

The read showed six spellings the three guards answer nothing for,
by running the guards' own reading functions over planted text:

- the link guard,
  `hook_self_invocation_tests.rs`,
  let `std::fs::soft_link` through:
  the deprecated standard-library spelling of a symlink was in no list of link names;
- the note guard,
  `the_commit_path_writes_its_notes_off_its_thread_tests.rs`,
  let `eprintln!`,
  a file opened at `/dev/stderr`,
  and a write to `/dev/fd/2` through:
  it read the call,
  the type,
  the rename and the descriptor constant,
  and no macro or path;
- the login guard,
  `no_start_at_login_tests.rs`,
  let a `Command` imported under another name
  (`use std::process::Command as Cmd;` then `Cmd::new("systemctl")`) and a `Command::new(`
  whose literal stands on the next line through:
  it read one line at a time,
  for the text `Command::new`.

### The green that had to be red

Planted in covered release files of the tree,
in a scope,
with the switches the guide's flags pass,
each guard's own cell stayed green:

```text
hook_self_invocation_tests                           test result: ok. 8 passed; 0 failed
the_commit_path_writes_its_notes_off_its_thread_tests test result: ok. 6 passed; 0 failed
no_start_at_login_tests                              test result: ok. 10 passed; 0 failed
```

The plants were a `std::fs::soft_link(..)` call in `commit/commit_command.rs`,
an `eprintln!`,
a `File::create("/dev/stderr")` and a `std::fs::write("/dev/fd/2", ..)`
in `commit/commit_command_words.rs`,
and the rename plus the next-line literal in `service/service_instance.rs`.
They were reverted and never committed.

### What the guards read now

Commit `342d3d21fd` makes each guard read its spellings,
and each guard's own self-test plants them:

- the link guard reads `soft_link` as one more name of a link,
  where no longer name touches it;
- the note guard reads `eprintln!` and `eprint!` as the terse ways to write standard error,
  and `/dev/stderr`,
  `/dev/fd/2` and `/proc/self/fd/2` as the paths that open the same handle;
- the login guard refuses the import that renames the type of a start,
  and reads a start whose literal stands on the code line after its `(`.

The same plants,
in the same files,
are findings afterwards:

```text
commit/commit_command.rs:109 (soft_link) let _ = std::fs::soft_link("planted-source", "planted-target");
commit/commit_command_words.rs:69 (eprintln!) eprintln!("planted");
commit/commit_command_words.rs:70 (/dev/stderr) let _ = std::fs::File::create("/dev/stderr");
commit/commit_command_words.rs:71 (/dev/fd/2) let _ = std::fs::write("/dev/fd/2", b"planted");
service/service_instance.rs:118: Command as
service/service_instance.rs:120: a start of the manager's own program: let _ = std::process::Command::new( "systemctl",
```

The last two are quoted with their runs of spaces closed up,
since the finding prints the line as the file writes it,
indentation included,
and the start's finding is the call's line joined with the literal's line.
The `Command as` finding's banned text ends with the space between the two words.

One cell failed in each module,
`7 passed; 1 failed`,
`5 passed; 1 failed` and
`11 passed; 1 failed` in that order.
With the plants reverted the same three modules answer
`ok. 8 passed`,
`ok. 6 passed` and
`ok. 12 passed`.

### Which other rule refuses a spelling

Only `eprintln!` is refused elsewhere:
`source_guard/diagnostic_door_tests.rs` fails any release file whose code carries `eprintln!(`,
because `eprintln!` is the one terse way to say a diagnostic without reaching the door.
The note guard's top doc says so and reads it in the note lane besides,
so a note file that spells it fails there too.

Nothing else refuses any of the six.
`clippy.toml` bans `std::env::temp_dir`,
four `tempfile` constructors and `Result::unwrap`;
the crate's `[lints.clippy]` turns on `disallowed_methods`,
`implicit_return`,
`shadow_reuse`,
`shadow_same` and `shadow_unrelated` and no group,
so `clippy::print_stderr` stays as allow-by-default as it ships;
and the repository linter's Rust rules are
`avoidable-closure`,
`file-order-in-folder`,
`folder-uses-links`,
`max-lines`,
`no-anonymous-functions`,
`require-explicit-types`,
`require-rustdoc` and
`tests-in-sidecars`,
none of which names a link,
a stream,
a path or a start.

### What is still unread

The reader's own top doc keeps the list,
and each line of it is still a shape no line covers:

- a `pub` field in a tuple-like struct's parentheses;
- a member of a type a facade names through an inline `mod` body or a module the tree cannot
  resolve;
- a `pub trait`'s methods and a `pub enum`'s variants;
- an item inside an inline `mod name { ... }`;
- the names a glob carries out of a module no `pub mod` chain reaches
  (the guide said "a glob's own names",
  which is broader than the reading,
  and now says what the
  reading does);
- the second path a re-export adds when it keeps the name it re-exports;
- a signature or a value that changes under a name that stays.

The guards' own top docs keep theirs:
a call a macro of another file expands to,
a name a glob import brings into scope,
a call whose name and `(` a line break separates,
a start whose program is a value rather than a literal,
a `member:` whose literal a line break separates from its key,
a write to the plain descriptor number (`libc::write(2, ..)`),
a path built from parts or held in a variable,
and,
for the login guard,
a literal further than the next code line from its call's `(`.

The guide's section on the published surface and its section on a guard's scope,
excuses and anchor now say the same things as those docs.

## The alpha.52 second pass: the `final-newline` hints

Round of 2026-11-28,
unapproved and non-final.

A second pass by another agent read `0.6.1-alpha.52` (`bf46e60f6d`)
against `0.6.1-alpha.51` (`81b06993bd`).
That agent is of the same vendor as some of the authors,
so the pass is not an independent review.
It reported four things about the hint a `final-newline` finding carries.
This round reproduced each one against the released binaries and against the built executable,
changed what the records and the runs support,
and kept each sentence that was true.
Everything below is this round's own measurement unless it names the pass.

The pass's other findings about the reading of a `!` alias
(M1,
L1 and L2 in its report)
belong to the round that holds that subject.
Nothing here says anything about them.

### The answers a finding can carry about a commit

`src/native/policy/event_hints.rs` answers what a commit does with the bytes a finding names.
Three answers are left:

- `CorrectsUnlessTheSelectionIsReadOnly`,
  for an `add` and for a direct check,
  whose bytes a commit that takes them corrects unless its selection is read-only;
- `NothingCorrectsWhatTheIndexHolds`,
  for a commit whose selection is read-only and which has not landed;
- `NothingCorrectsWhatACommitHolds`,
  for a landed commit and for a pushed tree.

A fourth answer said the lifecycle corrects the bytes itself.
It is gone with the arm that printed it,
because no path of the built executable printed it.
`finding_hint` no longer takes whether the finding offers a patch,
so no caller can choose a wording by a fact a reported pass never holds.

### Low 3: the `add` hint named a commit form this wrapper refuses

The pass read `git commit --message K1` inside a container
and saw it refused with `commit-only/pathspec-required`,
where the hint said a plain commit also corrects the ending.
This wrapper refuses a commit that names no path,
so those words named a form a person cannot use.

Measured again through this worktree's build:
`git commit --message=plain` in a fixture whose index holds the file exits 1,
and the same commit with `-- a.txt` lands bytes ending with one line feed.
The control `binary::final_newline_hint_steps::the_commit_the_hint_names_is_the_form_that_corrects_the_ending`
runs both and reads the landed blob.

The sentence changed inside the `add` hint.

Before:

```text
a plain commit, one that takes the whole file, also corrects the ending while this repository's settings keep
`final-newline` on
```

After:

```text
a commit that names the file, as in `git commit -- <path>`, also corrects the ending while this repository's
settings keep `final-newline` on
```

The same change turned the hint's "then stage the file again" into "then stage the file",
because a direct check now gives this hint and a checked file need not have been staged before.

### Low 6: the direct check promised a correction a read-only selection does not make

The pass's row K8 ran `check -- a.txt`
and then `git commit --include --message K8 -- a.txt`,
which landed the bytes with no line feed although the check had promised a correction.
`commit --patch` does the same in its row K3.

Two shapes were open,
as the pass put them:
carry the `add` hint's carve-out into the direct check's hint,
or map a direct check to the answer an `add` gives.
This round took the second,
because the two lifecycles answer the same fact.
Both report worktree bytes that a commit corrects only by taking them,
and neither records anything itself.
The `add` hint's clause about an index that already holds the bytes is true of a checked file too,
and a check may have staged nothing,
so one text serves both and the two cannot drift apart again.
`policy_final_newline::commit_correction` maps `CandidateRequest::Add` and `CandidateRequest::Direct`
to `CorrectsUnlessTheSelectionIsReadOnly`.

The sentence at the direct check.

Before:

```text
Hint: if you are committing this file, a commit that takes it corrects the ending for you while this repository's
settings keep `final-newline` on; to correct it now without committing, run `git monochromatic-git-policy-cli fix --
<path>`.
```

After,
and this is the `add` text too:

```text
Hint: to correct the ending now, run `git monochromatic-git-policy-cli fix -- <path>` and then stage the file with
`git add -- <path>`, because an index that already holds these bytes keeps them until the file is staged once more;
a commit that names the file, as in `git commit -- <path>`, also corrects the ending while this repository's settings
keep `final-newline` on, but a commit whose selection is read-only (`-p`, `-i` or `--include`) records the bytes it
takes exactly as they are.
```

Note of 2026-11-28T09:29:00-05:00:
this choice is superseded.
The read of `0.6.1-alpha.53` found the clause about an index that already holds the bytes false
where a direct check prints it,
and that round gave the check its own sentence
(`CommitCorrection::CorrectsWhatNothingHasStagedUnlessTheSelectionIsReadOnly` in
`src/native/policy/event_hints.rs`,
found in the section "The alpha.53 second pass" of this file):

```text
Hint: this check staged nothing, so to correct the ending now, run `git monochromatic-git-policy-cli fix --
<path>`, and then either stage the file with `git add -- <path>` or commit it, as in `git commit -- <path>`; a
commit that names the file corrects the ending while this repository's settings keep `final-newline` on, but a
commit whose selection is read-only (`-p`, `-i` or `--include`) records the bytes it takes exactly as they are.
```

The `add`'s text above is unchanged.

The controls are
`policy::final_newline_hint_of_each_lifecycle_tests::a_direct_check_says_what_corrects_the_bytes_it_read`,
which reads the rendered finding in-process,
and `binary::final_newline_hint_steps::a_direct_check_gives_the_hint_whose_commit_corrects_the_bytes_it_read`,
which reads the hint the built executable prints and then lands the commit it names.

### Low 4: at `warn`, a commit that landed printed a sentence the landing had made stale

A commit whose selection is read-only reads the index before the commit is made.
Its finding says the index holds the bytes and that staging them again is the next step.
At `warn` that commit then lands,
and the post-commit gate reports the same file with the landed sentence,
so one run printed both for one file:

```text
{"type":"finding","trigger":"pre-forward","policyId":"final-newline","path":"a.txt","hint":"Hint: if you are committing this file, edit it so it ends with exactly one newline, then stage it again."}
{"type":"finding","trigger":"post-commit","policyId":"final-newline","path":"a.txt","hint":"Hint: this file's bytes are already in a commit, so nothing here corrects them; edit the file so it ends with exactly one newline, then commit the correction."}
```

`commit::commit_command::run_timed` now drops a finding whose answer is `NothingCorrectsWhatTheIndexHolds`
when the service answers with a landed commit.
The pass ran before the landing,
and the gate reports the bytes the landing holds.
Nothing else of that pass is dropped,
and a commit that does not land keeps the staged sentence.

The control
`binary::final_newline_hint_steps::a_read_only_selection_records_the_bytes_it_takes_until_the_file_is_staged_again`
asks for one finding line and reads the landed sentence.
On `main`'s release code it answers two lines.
The same control reads the bytes the selection recorded,
so the carve-out the hints now name is pinned by a run:
`git commit --include -- a.txt` lands the bytes with no line feed,
which is what the words say.

At `error` the cost is nothing.
The commit does not land,
so the staged sentence is the only one printed and it is true.
`binary::final_newline_hint_steps::the_read_only_selection_that_does_not_land_says_to_edit_and_stage_again`
reads that sentence,
the untouched `HEAD`,
and the corrected bytes the person's edit and staging then land.

One narrow path reports nothing at all.
A commit that lands and whose gate cannot read the new commit reports the gate's engine failure and no finding,
and the staged sentence would have been the only word about the file.

### Low 5: the records described a finding no run prints

The `README.md` and `SPEC.md` sentences said that a commit which may correct the bytes,
and `git monochromatic-git-policy-cli fix`,
say the correction is that command's own.
No run printed such a finding,
and none can.
`fix` runs the bounded convergence of `policy_convergence`.
A patch-capable commit runs the same convergence (`commit::commit_convergence`).
A pass that proposes a correction is followed by the pass that applies it,
so the pass a run reports holds no finding at all.

Measured here with the released alpha.52 binary,
`git monochromatic-git-policy-cli fix -- a.txt` at `warn` and again at `error` printed one line and no finding:

```text
{"schemaVersion":1,"sequence":0,"type":"fix-summary","trigger":"direct-fix","passes":1,"changedPaths":["a.txt"]}
```

The same holds through this worktree's build for `fix` and for a commit that names the file.
`binary::final_newline_hint_steps::a_direct_fix_reports_a_summary_and_leaves_one_line_feed` reads the first,
and `binary::final_newline_hint_steps::the_commit_the_hint_names_is_the_form_that_corrects_the_ending` the second.
Both are green on `main`'s release code as well,
and that is the finding:
the arm was unreachable before this round touched it.

What went:

- the hint arm for a finding that offers a patch;
- the `finding_hint` parameter that chose it;
- the two in-process controls that drove one stage with a patch-capable commit and with a direct fix;
- the two records' sentences.

`README.md` and `SPEC.md` now say what a run prints.
The correction appears as a `fix-summary`,
and a correcting lifecycle reports no finding.

### The controls that run the step each hint names

Each hint was pinned by an exact-text control,
and no control ran the step its hint names,
so a change that stopped a commit from correcting the ending would have left every one of them green.
`binary::final_newline_hint_steps` holds one control per hint.
Each runs the named step through the built executable in a fixture and reads the bytes that result:

- `the_add_hint_s_two_steps_leave_one_line_feed_in_the_worktree_and_the_index`:
  `add`,
  then `fix -- a.txt`,
  then `git add -- a.txt`,
  and the worktree and the index both end with exactly one line feed;
- `a_direct_check_gives_the_hint_whose_commit_corrects_the_bytes_it_read`:
  `check -- a.txt`,
  then `git commit -- a.txt`,
  and the landed blob ends with one line feed;
- `the_commit_the_hint_names_is_the_form_that_corrects_the_ending`:
  a commit with no path is refused,
  and `git commit -- a.txt` lands the corrected bytes;
- `a_read_only_selection_records_the_bytes_it_takes_until_the_file_is_staged_again`:
  `git commit --include -- a.txt` lands the uncorrected bytes and one landed sentence,
  then the edit,
  the staging and the commit land the corrected bytes;
- `the_read_only_selection_that_does_not_land_says_to_edit_and_stage_again`:
  the same selection at `error` prints the staged sentence and leaves `HEAD` alone,
  and the edit and staging after it land the corrected bytes;
- `a_direct_fix_reports_a_summary_and_leaves_one_line_feed`:
  `fix -- a.txt` prints the summary alone and leaves the corrected worktree.

The fixtures' `observe` helper removes every `hint` field,
so a control can compare an event's bytes with the retired wrapper's.
These controls read `observe_raw` instead and drop only the lines a wait prints.

### What is open

- The `warn` shape of Low 4 is this round's choice and stands open to the owner's veto.
  The way not taken is to print both sentences.
  That costs a person a sentence the landing had already made stale.
  What is taken costs the file its only word when a landed commit's gate cannot read the new commit.
- `CommitCorrection` keeps one variant for the two lifecycles that read worktree bytes and drops the one that
  told a direct check apart.
  A reader who wants the answer named after the bytes rather than after the commit has an easy rename here.
  Nothing in this round's findings needs it.

## The read of `0.6.1-alpha.52`: the guest drivers' shared steps

Round of 2026-11-28,
unapproved and non-final.
This file also holds the passes over `0.6.1-alpha.48`;
the section here is one read of `0.6.1-alpha.52` that the session weighed,
and the round that answered four of its findings and checked the fifth.
Only the owner declares it complete.

The read is a second pass by another agent of the same vendor as some of the authors,
so it is no independent review.
Its limits are the ones it states:
it started no guest,
it built no binary,
and it drove the drivers of the package against a stand-in `mvm`,
`node` and `tar`.
The round below kept to the same ground:
every reproduction runs a stand-in for `mvm`,
and no guest of this machine was used.

The read rated two Mediums and three Lows on the shared steps that every guest driver runs
(`bin/mvm-guest-step-command.ts`,
`bin/mvm-guest-detached-script.ts`,
`bin/mvm-guest-command-line.ts`,
`bin/mvm-guest-whole-run.ts`).
The Mediums were both consequences of the merge `98996296fb`,
which replaced three sets of guest helpers with one,
and the release said of that merge only that what every driver sends its guest is unchanged.
Two of those claims were about the drivers' own process and about the status of a command,
and both were false.

### What a real `mvm exec` answers for a command that exits nonzero

The refusal of a refused launcher rests on this,
so it was taken from the producer's source and its controls,
not from memory.
`package/mvm/cli/src/cli.ts` answers the guest command's own status:
`const result = await backend.exec({ command, name })`,
then `if (result.exitCode !== 0) process.exitCode = result.exitCode`.
The status itself is the guest agent's answer,
`exitCodeOf` in `package/mvm/cli/src/guest-exec-status.ts`,
which is `exitcode` when the answer has one and `128 + signal` when a signal ended the command.
The controls beside it agree:
`guest-exec.unit.test.ts` returns `exitCode: 3` for a command ending `exit 3`,
and `137` for one that ended `kill -KILL $$`.

So a launcher whose first line is `test "$(hostname)" = NAME || exit 1`
ends `mvm exec` with status 1 on another machine's guest,
and a status other than zero is exactly "the launcher was refused".

### M1: an interrupted driver left the step it had started running

Every step command is started in a process group of its own (`detached: true`),
which is what lets a step's bound stop everything the step started.
The same separation is what a terminal's interrupt key cannot reach:
SIGINT goes to the driver's own group,
so the driver ended and the step went on working with nobody reading it,
bounded at the two hours a guest build may take (7,200,000 ms),
and no driver handled a signal.

Reproduced before the change with the real `forced-act-guest.ts create`
and a stand-in `node` for the recipe step that records its own process number and sleeps,
SIGINT sent to the driver's group as a terminal sends it:

```text
before-main e2e/forced-act-guest.ts create: driver pid 1146505 leads group 1146505; the step it started is pid 1146521
before-main: before the signal, the step is alive=true
before-main: after SIGINT to the driver's group: driver exit code null signal SIGINT; step alive=true
before-main: the step outlived the driver, and this reading killed it
```

`01cb2ed4c9` holds the step commands this process is still waiting for in
`bin/mvm-guest-live-steps.ts`,
and a signal that ends the driver stops each through the group stop its own deadline uses
(`stopProcessGroup`),
says one line naming the command and its process group,
and then ends the driver the way the signal would have:
the handlers go first,
so the re-raised signal has its default action.
SIGHUP,
SIGINT and SIGTERM are the three signals,
because a terminal that went away is the same
hazard as one that was interrupted.
The same driver,
the same signal,
after the change:

```text
after e2e/forced-act-guest.ts create: driver pid 1257141 leads group 1257141; the step it started is pid 1257237
after: after SIGINT to the driver's group: driver exit code null signal SIGINT; step alive=false
git-policy guest step: SIGINT, so this run stopped every step command it had started and still had running: node .../e2e/run-fixture.ts create --name m1-after-guest --log-dir .../logs (process group 1257237)
```

The controls are three cases of `bin/mvm-guest-step-command.unit.test.ts`,
each running the driver as a program of its own,
because a signal's disposition belongs to the whole process:
SIGINT and SIGTERM each stop the step and name it,
and a step that had already ended is neither stopped again nor named.
Shown red by taking the hold out of `awaitStepCommand`:

```text
[FAIL] stops the step it started when SIGINT ends the driver, and names what it stopped
  AssertionError: expected false to equal true  -- interrupted.stepIsGone
[FAIL] stops the step it started when SIGTERM ends the driver, and names what it stopped
[PASS] stops and names no step that had already ended
```

What the change does not do,
and the record of it says so:
a guest the interrupted run had made is left as it was,
because taking a guest away is a step of the run's own,
so the guest's name and `mvm destroy` are what a person needs next.

### M2: a refused launcher was waited out instead of refused

`startDetachedGuestScript` called `runQuietStepCommand` for the `mvm exec`
that starts the launcher and threw the status away,
while its own doc promised a throw "when the guest could not be asked to start the script".
The adoption and start-probe drivers then waited for a log no guest would write,
for `crateBuildTimeoutMilliseconds + buildFinishMarginMilliseconds`
(2,400,000 + 180,000 ms,
43 minutes,
`e2e/adoption-guest-arms/crate-build.ts:88` and `e2e/start-probe-arms/crate-build.ts:200`),
and the step 22 driver for its own crate-build bound.
Before the merge the three drivers failed at once,
with the guest's name,
through `e2e/guest-host/guest-side-scripts.ts:187`.

Reproduced against a stand-in `mvm` that accepts both pushes and exits 1 for the launcher,
the driver program then doing what every driver does next:

```text
before-main: the driver program exited with 0
the start returned after 129 ms, although the launcher exited 1
the wait refused after 15205 ms: build.log: the guest script did not finish inside 3000 ms; see build.log
```

The wait's bound was 3,000 ms and it answered after 15,205 ms,
because a look that finds no status line waits one `POLL_INTERVAL_MS` (15,000 ms) first:
the real drivers' wait would be the 43 minutes of their own bound.

`09e47a5285` reads the status and refuses a nonzero one at once,
naming the launcher's command,
its status and the guest's name:

```text
after: the start refused after 66 ms: mvm --backend libvirt exec guest-a -- sh /mnt/shared/build.log.launch.sh exited 1: the launcher was refused, so 'guest-a' started none of build.log.step.sh
```

The controls are two cases of `bin/mvm-guest-detached-script.unit.test.ts`:
one reads the refusal,
the guest's name,
the status,
the three calls the stand-in was given and
that nothing came after the launcher,
and the other is the case where the launcher exits zero,
which must reach the wait,
so the probe is shown able to tell the two apart.
Shown red by dropping the status again:

```text
[FAIL] refuses at once when the launcher exited nonzero, naming the guest and the status
  expected 'the start returned\nthe wait refused:…' to include 'the start refused:'
[PASS] waits for the script when the launcher exited zero
```

The removal-lineage and settling drivers carried `nameCheck: 'none'` and ignored the launcher's
status before the merge too,
so what changes for them is new behaviour and not restored behaviour:
a launcher that `mvm` could not run,
or a guest the command could not reach,
refuses at once with the launcher's status and the guest's name
instead of leaving them to wait out the build's bound for a log that would never appear.
`e2e/README.md` says that,
and says which drivers carry no name check at all.

### L3: the step-shaped name check guarded nothing

`bin/mvm-guest-command-line.ts` wrote the `step` shape as `test "$(hostname)" = NAME`
with no `|| exit` and no `set -e`,
so on a machine of another name the step file ran the script anyway,
while the shape's doc said the detached script refuses itself.
`e2e/step-22-arms/detached-step.ts` is the driver that sends this shape,
and its check was the only one its scripts carried.

Reproduced by running the package's own step text with `sh` and a stand-in `hostname`:

```text
## before-main: the step-shaped check, run with sh on a machine of another name
sh exit status: 0
the log the step file left: "the script itself ran\nDONE=0\n"
## before-main: the same file on the guest it names, which is the positive control
sh exit status: 0
the log the step file left: "the script itself ran\nDONE=0\n"
```

`85213b9623` makes the shape refuse,
and writes the status line the host already waits for before it exits,
because a step file that exited without one would leave the host asking until the script's bound
passed:

```text
test "$(hostname)" = guest-a || { echo "DONE=1" >> /mnt/shared/build.log; exit 1; }
...
## after: the step-shaped check, run with sh on a machine of another name
sh exit status: 1
the log the step file left: "DONE=1\n"
## after: the same file on the guest it names, which is the positive control
sh exit status: 0
the log the step file left: "the script itself ran\nDONE=0\n"
```

So the host's next look finds `DONE=1` and refuses with the status the step wrote,
at once rather than at the end of the script's bound.
The shape was kept rather than dropped,
because the step 22 driver's record says the check has always sat in its step file,
and because the one place the three helper sets disagreed is worth keeping as a distinction.
Its controls are two cases of `bin/mvm-guest-command-line.unit.test.ts`,
which run the step file with `sh` on another machine's name and on the guest's own,
and the two text cases beside them.
Shown red by writing the bare check again:

```text
[FAIL] puts a check that refuses, and its status line, in the step when the launcher carries none
[FAIL] bounds the script, sends both streams to the shared log and appends its status
[FAIL] refuses at once, and writes the status line the host waits for, on a machine of another name
  expected +0 to equal 1
[PASS] runs the script on the guest it names, which is what the refusal is told apart from
```

### L4: the sentences whose wording the merge changed, checked and left

The read argued from the source that two failure sentences changed with the merge:
`git-policy guest run: destroying '<name>' also failed` (from the old lifecycle's own label)
and `exited -1` where the old helper printed `exited null` for a command a signal ended.
Nothing in the package or in this repository's records reads either sentence:
`rg 'exited -1'` and `rg 'exited null'` find nothing at all,
`rg 'also failed'` finds unrelated prose and the slot-state driver's own line,
and no test asserts either form.
So both sentences are left as they are,
and this record says they were checked.

### L5: two sentences that did not say what their own code says

`e2e/README.md` said four drivers "hold no plain long command at all"
(and five on `main` by 2026-11-28,
because another round added `cancel-and-settle-commands-guest.ts` to that list)
over a list that names three plain `guest()` commands of one of them,
each bounded at 900 s and none timed on its own
(`e2e/adoption-guest-arms/store-dataset-and-fsync.ts:115`,
`delegation-arm.ts:155`,
`snapdir-arm.ts:149`).
The sentence names those three now and says what the section's own rule owes them:
a command bounded above five minutes runs detached,
nothing has timed these three,
so a round that keeps them plain owes a measurement under five minutes or a detached script.

The package guide said the start probe's shared header "is quoted now",
which reads as one of the merges of this release.
It was quoted before the release:
`dacca205b6` (2026-11-27T11:57:11-05:00) quoted it,
and that commit is an ancestor of `0.6.1-alpha.51`.
The guide names the commit and its time now.

### What the read's own ground did not cover

No guest was started,
so none of these readings says what a real `mvm` does with a signal
sent to its own client,
and the two scripts a launcher runs were never executed inside a guest.
The step-shaped check was read with `sh` of this host,
with the guest's shared folder replaced by a folder of the round's own.
What a `mvm exec` answers for a nonzero command is the producer's source and its controls,
not a guest of this round.

## The alpha.52 behaviour pass

A second pass read `0.6.1-alpha.52` (`bf46e60f6d`) against `0.6.1-alpha.51` (`81b06993bd`) on 2026-11-28,
over 230 rows of spellings,
each row on four arms:
alpha.51,
alpha.52,
a byte-identical copy of alpha.52,
and real Git 2.56.1 as the ground truth.
It was another agent of the same vendor,
so a second pass and not an independent review.
Its scratch folder expires,
so the evidence each finding rested on is restated here,
and every fixture the round re-ran is described in full so a reader can rebuild it.

The round that acted on it is this worktree's branch,
whose commits are `ef2206f5eb`,
`503c681ea9` and
`c87f309101`.
Nothing in this section is approved or final.

### The fixture every row below was measured in

Real Git only,
`/usr/bin/git`,
never the `git` on this machine's `PATH`:

- one repository `main`
  whose commit of `a.txt` holds `a1\n`;
- `main` dirty:
  `a.txt` edited to `a1 edited\n`,
  `staged.txt` staged with `s1\n`,
  and `u.txt` untracked;
- linked worktrees `wt1` (clean tracked content),
  `wt2` (dirty,
  with an unstaged edit and a staged `s2.txt`)
  and
  `wt3` (completely clean);
- `g = !git` in the fixture's global configuration.

Every wrapper run had `env --unset=DBUS_SESSION_BUS_ADDRESS`,
`XDG_RUNTIME_DIR` a short folder of the run's own,
`HOME` and the four `XDG_*_HOME` inside the fixture,
`GIT_CONFIG_NOSYSTEM=1` and a fixture `GIT_CONFIG_GLOBAL`.

### M1: a `core.worktree` main checkout is judged as plain, and the alias resets it

The read's words:

> With `core.worktree` set to the main checkout in its own configuration (the main checkout is dirty),
> typed in `main/sub`:
> `git g -C <wt3> reset --hard`
> (also `clean --force`,
> `checkout -- a.txt`,
> `stash`,
> and the same through `git -c alias.cc=!git cc -C <wt3> reset --hard`)
> changes `main` files and index in real Git,
> and alpha.52 runs it and loses the staged `s.txt` and the edit of `a.txt`.

Reproduced on 2026-11-28,
with one change of verdict for the round:
the reading was fixed,
so the same rows are refused now.
The fixture the round used has `core.worktree = <main>` in `main/.git/config`,
`main` dirty as above and `wt3` clean.
Real Git typed in `main/sub` answers:
`rev-parse --git-dir` = `<main>/.git` (absolute),
`--absolute-git-dir` the same,
`--show-toplevel` = `<main>`,
and the fixture's own `!printf %s "$GIT_DIR"` alias prints `<main>/.git`.
Both binaries ran `g -C <wt3> reset --hard` with exit 0,
no standard error,
and afterwards `main` held no staged or unstaged work at all:
`git status --porcelain` was empty,
`a.txt` was back to `a1\n`,
and `staged.txt` was gone.
The controls in the same fixture:
the direct spelling `git -C <wt3> reset --hard` typed in `main/sub` changed nothing,
and the same effect spelled directly,
`git -C <main> reset --hard` from `wt3`,
ran and lost the same work,
which is what real Git would do.
Without `core.worktree` the same alias words change nothing.

What Git does,
from its own source at the tag this machine runs,
Git 2.56.1 (`setup.c`):

- discovery takes the explicit path when the repository's own configuration names a work tree:
  `repo_discover_implicit_gitdir` at `setup.c:1247` to `:1254` reads
  `if (getenv(GIT_WORK_TREE_ENVIRONMENT) || discovery->format.work_tree)`;
- that path stores the Git directory it found,
  and `setup_git_directory_gently` exports whatever it stored:
  `setup.c:2063` to `:2071`,
  through `apply_and_export_relative_gitdir`,
  which calls `xsetenv(GIT_DIR_ENVIRONMENT, path, 1)` at `setup.c:1102`;
- only a plain main checkout whose `.git` is the directory Git found reaches that block with nothing stored,
  which is the one shape where the nested Git finds the repository by itself;
- `core.worktree` is read from the repository's `config` (`setup.c:585` to `:597`)
  and from `config.worktree` when the extension is on (`setup.c:786` to `:793`),
  and a value in a shared configuration is ignored when the Git directory holds a `commondir`
  (`setup.c:805` to `:812`).

The reading the release shipped compared `<top>/.git` with `--absolute-git-dir`,
which with `core.worktree` set to the checkout are the same directory,
so it answered that Git gave the alias's shell nothing and passed no `--git-dir`.

What changed:
`external_git_command_the_git_dir_the_shell_reads.rs` asks Git
`git config --show-scope --get-all core.worktree`,
and counts only a value the `local` or `worktree` scope set,
because discovery reads the repository's own files and never a `-c` on the command line.
The question is asked only where the comparison has already read the shape as the plain `.git` at the top,
so the ordinary case pays nothing,
and a main checkout pays one more Git process per `!` alias whose own words name a place.
A read that fails answers nothing to add:
the shape it asks about is the one the comparison has already read,
and pinning a repository on an answer nobody read would judge a place the command may not act in.

Controls:
`external_git_command_tests.rs`'s `the_git_dir_the_alias_shell_reads_is_the_one_git_gives_it`
now holds a main checkout whose configuration names its own work tree,
at its top and in a subdirectory,
beside the seven places it held;
`entry_shell_alias_judged_where_its_words_point_tests.rs`'s
`a_checkout_naming_its_own_work_tree_is_the_state_the_shell_has`
holds the alias spelling of every guarded form to the direct spelling of the state the nested Git acts in.
With the new clause taken out,
the first control failed with
`left: false` against `right: true` at the checkout's own top,
and the second failed at its first response.

The citations these files carried before the round,
`setup.c:1126` to `:1129` and
`setup.c:1657` to `:1662`,
name other code in Git 2.56.1;
the round replaced them with the lines above in every file it touched.

### The three shapes the read did not reach

The read named three shapes as unchecked.
Each was measured with real Git 2.56.1 on 2026-11-28,
in a repository of its own,
with the same `printgitdir` alias and a row through alpha.52:

- `.git` a symbolic link to a directory.
  At the top the alias shell is given nothing,
  `--git-dir` prints `.git`,
  and `--absolute-git-dir` prints `<top>/gitdir-real`;
  in `sub/` the shell is still given nothing.
  The reading canonicalizes both sides of its comparison,
  so it answers "none" as Git does,
  and the row `g -C <clean linked worktree> reset --hard` typed in `sub/` changed nothing.
- `extensions.worktreeConfig` with `core.worktree` in `config.worktree`.
  The alias shell is given `.git` at the top and `<repo>/.git` in `sub/`,
  and `git config --show-scope --get-all core.worktree` prints `worktree\t<repo>`.
  The reading as shipped answered "none",
  and the row `g -C <clean linked worktree> reset --hard` typed in `sub/` reset the dirty checkout and lost
  its work,
  which is M1's loss by another route.
  The question added for M1 accepts that scope,
  so this shape is covered by the same change;
  no control builds it,
  so the coverage rests on the probe above and on the scope the reading accepts.
- a worktree path that holds a newline.
  `rev-parse --is-bare-repository --git-dir --absolute-git-dir --show-toplevel` then prints more lines
  than it has answers,
  and the reading takes the second and third of them:
  at the top of a main checkout at `<root>/new\nline-main` it read `.git` as the Git directory,
  `<root>/new` as the absolute one and `line-main/.git` as the top,
  and in `sub/` it read the truncated `<root>/new` as the Git directory itself.
  Both rows,
  `g -C <clean linked worktree> reset --hard` typed at the top and in `sub/`,
  were refused,
  with the outside-a-worktree refusal,
  and nothing was lost.
  `worktree_identity.rs` refuses such a path outright
  ("Repository paths containing line feeds are not supported"),
  so the shape is named and not supported,
  and the alias reading answers in the safe direction at the cost of a false refusal.

### The `git` inside an unread alias's shell, on an installed system

The read's L1 showed that an alias the reading leaves unread runs with no guard,
and its fixtures put the real Git behind the alias's `git`.
What that `git` is on an installed system was measured on 2026-11-28,
with the wrapper first on the fixture's `PATH` under the name `git`:

- here,
  `command -v git` inside the alias's shell prints `/usr/lib/git-core/git`,
  `readlink -f` of it prints `/usr/bin/git`,
  and the first `PATH` entry is `/usr/lib/git-core`:
  Git puts its own exec path in front of `PATH`,
  and this distribution's exec path holds a `git` that is the real Git,
  so the nested command is unguarded and the read's loss is real.
  Measured:
  `!git -C <dirty main> reset --hard` typed in the clean linked worktree ran with exit 0,
  and the main checkout's unstaged edit was gone;
  its staged entry survived,
  because the nested Git acted with the clean worktree's index.
- a system whose exec path holds no `git` leaves `PATH` to answer,
  and an installed wrapper is then that shell's `git`.
  Measured with `GIT_EXEC_PATH` naming an empty directory:
  `command -v git` printed the wrapper's link,
  and running it from the alias's shell failed with exit 126 and
  `<fixture>/bin/git: Permission denied`
  in the run whose log ends at 2026-11-28T03:07:24-05:00.
  This machine's AppArmor profile `git` allows executing `@{bin}/git`,
  `@{lib}/git*` and a few named programs,
  and nothing else,
  so the wrapper cannot be that shell's `git` here at all.
- what a nested wrapper judges
  when it is the one that answers,
  measured by handing it the alias shell's environment:
  the wrapper's own directory as the working directory and `GIT_DIR` the Git directory Git exported.
  It refused,
  with the linked-worktree-only finding,
  and nothing was lost;
  the same command with no `GIT_DIR` reaches the facts as the main worktree and is refused there.

The README and the SPEC now say both arms where they state the boundary,
and the boundary itself is unchanged:
whether an unread alias is refused is the owner's to decide,
and this round changed nothing about which aliases are read.

### L2: the refusal named no place

The read's words:

> the refusal text is "this worktree has uncommitted changes to tracked files" with no worktree named,
> though the dirty state is wt2's index over wt1's files.

Reproduced with real Git:
with one worktree's Git directory and another worktree's files,
`git -C <wt1> status --porcelain=v1 --untracked-files=no` answered `AD s2.txt`,
and `git -C <wt1> reset --hard` in that state dropped the staged entry of the index it kept,
so the state is dirty and the two places are real.
`rev-parse --show-toplevel` in that state prints the `-C` directory and
`--git-dir` the Git directory the shell kept,
which is why the identity the guard reads carries two places.

What changed:
`dirty_worktree_message` takes the two places and whether they are one worktree
and lives in `command/rule_linked_worktree_dirty_refusal.rs`,
the two places being read in `policy/policy_linked_worktree_the_places_it_judged.rs`,
because both files that held them would otherwise pass the linter's line limit,
and the text names the Git directory whose index and `HEAD` the form rewrites and the worktree whose files
it writes.
When they are one worktree the text says so and names that worktree;
when they are not,
it says the place is two,
names both,
and says the work that would be lost is that
index's staged entries and those files' own edits.
The function became crate-private with that signature,
and its line left
`src/native/source_guard/items_nameable_from_outside_release.txt`,
which records one name leaving and none joining.

Controls:
`rule_linked_worktree_tests.rs`'s `a_dirty_refusal_names_each_place_it_judged` holds the two shapes,
and `entry_shell_alias_judged_where_its_words_point_tests.rs`'s
`a_refusal_of_a_state_that_is_two_worktrees_names_both_of_them` builds the real two-worktree state and
asserts the refusal carries both paths.
With the places taken out of the text,
both failed;
`a_dirty_refusal_names_each_place_it_judged` failed on the path the text no longer held and the entry
control on both paths.

What the round left:
the main-worktree refusal,
`main git worktree`,
still names no path.
It is the text M1's shape gets when the state is the main checkout itself,
and naming its place is a wording choice outside L2's words,
so it is open for the owner with the cost that a person refused there must know which checkout is meant.

The three refusals issue 6641's closing comment records as open for the owner are unchanged,
and their costs are:

- `g -C <wt1> checkout -- <path>`,
  `restore -- <path>` and `checkout HEAD -- <path>` typed in a dirty
  linked worktree are refused where real Git changes nothing.
  The cost is that restoring one path from another worktree is refused,
  and the person has to copy the file or pass `--no-enforce-worktree`.
- An alias whose own `-C` names a worktree
  typed in a Git directory or in a bare repository,
  is refused with the uncommitted-changes finding where alpha.51 let Git print
  "this operation must be run in a work tree".
  The cost is a refusal that names a policy instead of Git's own reason,
  for a command Git would have failed anyway.

### The environment half of the reading

The read's sixth point was that no control gave an alias row `GIT_DIR` or `GIT_WORK_TREE` in the caller's
environment,
so the loop that reads them could be deleted with every test green.

What the round measured:
it changes nothing.
With the loop taken out,
the alias spelling `g -C <clean linked worktree> reset --hard` typed where `GIT_DIR=<the caller's Git directory>`
was in the environment is still refused,
because the facts' own Git inherits that variable and names the same state.
Measured against real Git:
an absolute `GIT_DIR` names that index over the `-C` worktree's files,
a relative `GIT_DIR=.git` is rebased by the command's own `-C` so that it acts in that worktree with no pair
at all,
and a `GIT_WORK_TREE` makes `--show-toplevel` differ from the Git directory,
which the comparison already reads as an export.

What changed:
the loop is gone,
`the_place_was_named` reads only the caller's own `--git-dir` and `--work-tree`,
and the inheritance it rests on is stated beside it,
with the gotcha that a fact request which unset either variable would make this reading name no state where
the nested command has one.
Two controls stay:
`an_alias_row_is_judged_with_the_git_dir_its_callers_environment_names` refuses the alias spelling with
`GIT_DIR` naming the caller's index over a clean worktree's files,
and `the_worktree_a_caller_names_runs_when_no_git_dir_reaches_the_facts` runs the worktree alone without it.
Both stay green without the loop,
which is the fact that took it out.

### What this round did not do

- The final-newline hints (the read's L3 to L6) belong to another round and were not touched,
  including the sentence the read says no run prints and the carve-out the direct-check hint leaves out.
- No whole container gate and no independent review ran.

## The alpha.53 second pass: the removal's resumed lock and what follows a finished one

Round of 2026-11-28,
unapproved and non-final.
Only the owner declares it complete.

A second pass by another agent of the same vendor as some of the authors read
`0.6.1-alpha.53` (`f81526484e`) against `0.6.1-alpha.52` (`bf46e60f6d`)
and reported its findings against those two released binaries.
This round took four of them,
reproduced each on the tree the force merge `7c4962988d` left,
and changed what the records and the runs support.
Everything below is this round's own measurement unless it names the pass,
and no guest ran:
every claim rests on the value layer and the package's own fixtures.
A quoted line of a program's output is wrapped for this page and keeps its
words.

### M1: a resumed removal whose clone is gone read the lock after it wrote

What the pass said:
a removal resumed from a plan whose head word is `requested`,
`unknown` or `gone`,
with the clone already gone,
took the head-gone path,
and that path answered `TheStart::Gone` without reading a person's lock.
The path then published the plan's own lines and sent the origin snapshot's
destroy,
and only the Git settle read `locked` afterward.

What was there,
on the merged tree:
`removal_resume.rs`'s `start_the_removal_with_force` returned `TheStart::Gone`
from the `CloneAbsence::Gone` arm and called nothing that reads a lock,
while every other path reads it in
`removal_entry.rs`'s `prepare_the_removal_over_a_plan`.
`removal_completion_head_gone.rs` then ran `publish_the_plan` and
`attempt_the_origin_cleanup` before `settle_the_registration`.

The control that showed it,
`removal_completion::tests::a_persons_lock_refuses_a_resumed_removal_whose_clone_is_gone_before_any_write`:
it plants the plan a killed removal leaves with each of the three head words,
plants the origin cleanup record an earlier run left so a removal that ran would
destroy the origin snapshot,
gives the pool a clone that does not answer,
writes a person's lock on the registration,
and reads what the run left.
Red on that tree,
for the `requested` plan:

```text
a person's lock refuses the resumed removal of a requested plan, so nothing finishes:
the ZFS worktree clone at `fix-flaky-test` (dataset `tank/wt/fix-flaky-test`) is destroyed,
with 2 of the snapshot(s) this removal recorded.
  destroyed snapshot tank/wt/fix-flaky-test@sanoid-hourly
  ...
  the origin cleanup destroyed the snapshot
  `tank/mono-checkout@git-worktree-clone-origin-445566778899aabbccddeeff00112233`
```

The order before and after,
in the file and the lines that hold it:

- before:
  `removal_resume.rs`'s `start_the_removal_with_force` builds
  `TheStart::Gone` from the plan and returns it;
  `removal_completion_head_gone.rs` proves the clone's absence,
  pushes a `destroyed` line for each name the plan still records,
  sets `head = HeadWord::Gone`,
  `publish_the_plan(&folder, &plan)`,
  `attempt_the_origin_cleanup(input, &plan)`,
  and only then `settle_the_registration`.
- after:
  `removal_completion_head_gone.rs` reads the lock first,
  through `the_lock_a_removal_refuses::the_locks_answer` with the run's own force
  count,
  before it proves anything and before `publish_the_plan`;
  a person's lock refuses the run there,
  and a two-force run deletes the lock as every other path does,
  with its line carried into the report's forced acts.

A second control holds the same order from the other side:
`removal_completion::tests::a_persons_lock_refuses_the_removal_before_any_request`
still refuses on the fresh path,
and both are green on the tree this round commits.

### L1: the sentence promised a removal that wrote nothing

What the pass said:
the sentence a person's lock is refused with said the removal "stops before its
first write",
and the order's first write is the slot's claim file,
which the removal takes before it reads anything.

What was there:
`removal_force_refusal_sentences.rs` printed that sentence,
and `README.md`,
`SPEC.md`,
`the_lock_a_removal_refuses.rs` and `removal_entry.rs` repeated the claim.

The sentence before,
with its middle elided:

```text
..., so a person's lock may stand there and this removal stops before its first write with no force or one. Nothing was unmounted, nothing was destroyed, no intent was filed and no plan was written. ...
```

The sentence after,
with the same elision:

```text
..., so a person's lock may stand there and this removal stops before it unmounts or destroys anything, with no force or one. The slot's claim file is the only record this run wrote, and stopping gives the slot back. Nothing was unmounted, nothing was destroyed, no intent was filed and no plan was written. ...
```

The control reads the claim rather than assuming it:
it lists the record folder's `<slot>.claim.<generation>` files,
requires exactly one,
and reads it through `single_owner_claim::decode` as a released claim,
which is what the drop a refusal takes writes.
Red on that tree at the first assertion of the new words:

```text
the sentence says before it unmounts or destroys anything: the Git registration of the slot `fix-flaky-test` holds a lock this add cannot attribute to itself (its words are `ZFS test underlay`), so a person's lock may stand there and this removal stops before its first write with no force or one. ...
```

The records were changed to say what the code does,
because the code and the tests are what the claim is about and the words were
wrong:
`SPEC.md` ("Removal of a ZFS worktree clone",
and the lock's paragraph in
the typed-network section),
`README.md`,
`the_lock_a_removal_refuses.rs` and `removal_entry.rs`.

### L3: a finished ZFS removal left its units and its shadow pool

What the pass said:
the two steps that follow a removal were keyed to `ForwardedRun::Ran(0)`,
and a bound slot's removal answers `ForwardedRun::Failed` with
`REMOVED_EXIT_CODE`,
so a finished ZFS removal stopped no unit and cleared no pool.

What was there,
at `worktree_copy_forward.rs`:

```text
if let (Some(gone), ForwardedRun::Ran(0)) = (about.as_ref(), &finished) {
```

The control that showed it,
`worktree_copy_forward::tests::a_finished_bound_slots_removal_stops_its_units_and_takes_its_shadow_pool`:
its removal step takes a real worktree the way a finished removal does and answers
`BoundSlotRemovalAnswer { code: REMOVED_EXIT_CODE }`,
and the control reads the stop's question and the pool's folder.
Red on that tree:

```text
assertion `left == right` failed: one removal, one stop
  left: 0
 right: 1
```

The change:
one reading of the run's ending now decides both steps,
`the_removal_ended_well`,
which answers a `Ran(0)` and a `Failed` whose code is `REMOVED_EXIT_CODE`.
Both steps keep their own proof that the worktree is gone
(`worktree_is_gone` over the registration the removal deletes),
so a removal that answered `0` without taking the worktree still stops nothing,
and a `Killed` or refused removal reaches neither.
The module documents of `worktree_copy_forward` and
`worktree_removal_stops_units` name the third removal,
and `README.md`'s passage about the units names it beside the subvolume destroy
and Git's own removal.

### L4: the line saying the removal runs was printed after it ran

What the pass said:
`worktree_removal_of_a_bound_clone.rs` built the line
"the removal of `<slot>` runs here:
a whole managed clone is bound to this slot,
and ... Git does not run"
into the answer's standard error before running the removal,
and returned it after the removal ended.

What was there:
the text and the slot's report were collected into a `String`,
`complete_the_bound_clones_removal` ran,
and the whole string was returned to be written by `finish_forwarded`.

The change:
`worktree_removal_said_before_it_runs.rs` holds one call,
`say_then_run_the_removal`,
which writes the line and the slot's report to the stream it is given and flushes,
then runs the removal it is given and answers what that answered;
a stream that refuses the line is dropped and the removal runs all the same.
The live arm of `the_removal_with` hands it standard error and
`complete_the_bound_clones_removal`,
and returns only the removal's own report,
so the person reads the line before the work starts and the report after it.
`SPEC.md` and `README.md` say so.

Three controls hold it.
`worktree_removal_said_before_it_runs::tests::the_line_is_out_before_the_removal_runs`
reads the stream's text at the moment the removal is entered,
so it fails when the two are swapped;
`::a_terminal_that_refuses_the_line_still_lets_the_removal_run`
is the closed terminal;
and `::each_arm_says_what_it_promises` holds what the live line and the refused
line say,
which no host control could read from the live arm before.
The order control's own guard was shown failing:
with the two statements of `say_then_run_the_removal` swapped by hand and run,
it failed with an empty text at the removal,
and the two statements were restored.

The opening lines themselves moved out of
`worktree_removal_of_a_bound_clone.rs` into the same file,
because that file's code lines passed the linter's limit of 300 with the live
line,
its force count and the resume sentence inside it,
and the linter's `rust/max-lines` finding was the one finding this round would
otherwise have added.
The words are unchanged by the move,
except that the live line's force count is now said from the struct's field.
The live arm itself is what no host control can run,
because `complete_the_bound_clones_removal` opens this machine's ZFS device,
which the pass's M2 says,
and the guest arm is where that half of the claim belongs.

### The one other path a plan's tail runs on

`worktree_clone_removal_finish_pass::finish_one_plan` runs before every
forwarded command for a plan whose head is `gone`.
It was read for the same defect and does not have it:
it takes the slot's claim,
proves the clone's absence,
settles the registration,
ends the binding's files and removes the plan,
and it destroys nothing and sends no origin cleanup,
so a person's lock met at its settle costs the claim file and no other record,
and nothing a person would lose goes before it.
No control or change was made there.

### What this round did not do

- The pass's M2 (no guest arm plants a person's lock for a removal),
  L2 (a registered slot whose folder is gone is left to Git),
  L5 (the direct check's hint carries an index reason a check does not have)
  and L6 (a guest arm's stale comment) are other rounds' or nobody's yet;
  nothing here changed them.
- No guest ran,
  and no command opened `/dev/zfs`,
  changed ZFS,
  mounted anything or started a service manager.
- No whole container gate and no independent review ran.
  The two vendor-mates make this a second pass,
  not an independent review.

### The files this round changed

Code:

- `removal_completion_head_gone.rs`:
  the person's lock is read first on the
  head-gone path,
  through `the_locks_answer` with the run's force count,
  and the line a two-force run prints about the lock it deleted goes into the report's
  forced acts.
  The module doc no longer says nothing is destroyed on this path.
- `removal_force_refusal_sentences.rs`:
  the sentence a person's lock refuses
  with.
- `worktree_copy_forward.rs`:
  `the_removal_ended_well` decides the two steps
  after a removal,
  and its doc says which three removals the wrapper serves.
- `worktree_removal_said_before_it_runs.rs`,
  new:
  the two opening lines,
  `say_then_run_the_removal`,
  and their controls in
  `worktree_removal_said_before_it_runs_tests.rs`,
  new.
- `worktree_removal_of_a_bound_clone.rs`:
  the live arm says its lines before it
  runs the removal and returns only the removal's report,
  and the opening lines
  and the resume sentence moved to the file above.
- `worktree_copying/mod.rs`:
  the folder's list of its files and the `pub mod`
  line for the new file.
- `removal_completion_tests.rs`:
  the two lock controls,
  one of them red before
  the change.
- `worktree_copy_forward_tests.rs`:
  the finished-removal control,
  red before the
  change.

Records:

- `SPEC.md`:
  the lock's paragraph in the typed-network section,
  the lock's paragraph in "Removal of a ZFS worktree clone",
  and a paragraph saying the
  removal says it runs before it runs.
- `README.md`:
  the lock's paragraph,
  the passage about a removed worktree's
  units,
  and the paragraph about what a bound slot's removal says and does.
- `the_lock_a_removal_refuses.rs` and `removal_entry.rs`:
  the two code documents
  that repeated the old wording.
- `worktree_removal_stops_units.rs`:
  the module doc names the third removal.

### The checks this round ran

Every build,
lint and test ran in its own `systemd-run --user --scope --slice=agent-work.slice`
with a memory,
CPU,
task and I/O bound and a deadline.
Every wrapper test and every run of a built wrapper carried
`env --unset=DBUS_SESSION_BUS_ADDRESS XDG_RUNTIME_DIR=<a folder of this round's
own>`.

- `cargo fmt --check`:
  no output.
- `cargo check --locked --offline --all-targets`:
  finished,
  exit 0.
- `cargo clippy --locked --offline --all-targets -- -D warnings`:
  finished,
  exit 0.
- `cargo clippy --locked --offline --all-targets --features test-only-switches
  -- -D warnings`:
  finished,
  exit 0.
- The modules this round changed,
  threaded,
  with the guide's flags:
  `worktree_clone_removal_proof_and_plan::removal_completion::tests`
  (`12 passed; 0 failed`),
  `worktree_clone_removal_proof_and_plan::removal_entry::tests`
  (`10 passed; 0 failed`),
  `worktree_copying::worktree_copy_forward::tests`
  (`11 passed; 0 failed`),
  `worktree_copying::worktree_removal_said_before_it_runs`
  (`3 passed; 0 failed`),
  and `worktree_copying::worktree_removal_of_a_bound_clone::tests`
  (`23 passed; 0 failed`).
- The whole `source_guard` module:
  `236 passed; 0 failed`.
- The repository's linter over `package/git-policy/cli`:
  no finding the tree did not already hold.
  The tree this round merged held twelve `rust/folder-uses-links` findings,
  all of them the cancel and settle wiring the 2026-11-28 amendment lets stand,
  thirteen `rust/max-lines` findings in files of `foundation/`,
  `health/`,
  `invocation/` and `worktree/`,
  and one `rust/avoidable-closure` line saying the rule was skipped on this host
  for want of its Dylint driver.
  The same findings stand after the change,
  and none of them is in a file this round touched or on a line it wrote.

### What could not be run, and why

- No guest:
  this machine's home is on a ZFS pool and no record authorizes a run
  to work on it,
  and the brief asks for no guest.
- No command opened `/dev/zfs`,
  changed ZFS,
  mounted anything,
  used `sudo` or started a service manager.
- No whole container gate,
  no mutation campaign and no independent review.

## The alpha.53 second pass: two reads, one gate read, and what each sentence now says

Round of 2026-11-28,
unapproved and non-final.
Three reads of release `0.6.1-alpha.53` (`f81526484e`,
published 2026-11-28T11:49:18Z) against `0.6.1-alpha.52` (`bf46e60f6d`)
found places where a guard or a sentence said more,
or less,
than the code does.
Two are second passes by another agent of the same vendor as some of the authors,
so they are no independent review;
the third is a gate read of a design.
This round reproduced each finding,
changed what it could,
and leaves what it could not;
the code and these records are what carry the findings now,
because the passes' own scratch expires a day after its last change.
Only the owner declares the work complete.

The behaviour pass read the release beside its predecessor with both binaries;
the no-change pass read the five merges that claim to change nothing a release build does
(`74f34f6af9`,
`039e39b885`,
`18327d05f6`,
`888d3646df`,
`6f52a414fc`);
the gate read read the two-forces design,
and its H3 is the part that concerns this package.

### The no-change pass's L1, as the record states it

The merge `74f34f6af9` was recorded as a refactor that moves code between files.
For a release build,
"moves code" is the smaller half of what it does.

It moves 23 items with their bodies,
visibility and attributes unchanged,
and it changes nine item bodies,
of which one is new indirection on the path that gives a stalled reader up:

- `QueuePolicy` gains `reader_stopped_notice: Option<fn(&'static str, Duration)>`
  (`src/native/foundation/the_queue_and_its_thread.rs`,
  the field at line 133 after the merge);
- the diagnostics' policy sets it to `say_the_reader_stopped`
  (`foundation/diagnostic_lines_off_the_calling_thread.rs`),
  and the service's log queue sets it to `None`
  (`service/service_log_lines_off_the_calling_thread.rs`);
- `wait_until_written` (`foundation/lines_written_by_a_thread.rs`)
  calls that function where its wait finds the bound passed,
  where it called `diagnostic_log::debug` itself before the merge;
- and `say_the_reader_stopped` is the new function that keeps the switch,
  the level,
  the tag and the sentence where the measurement now is.

The same merge also adds 22 items,
four of them files,
changes four visibilities from private to `pub(crate)` and the fields of two structs to `pub(super)`,
and changes `SPEC.md` in two passages (the file that says where a refused thread's line is written,
and the list of the crate's own locks).
It adds no bare `pub`,
so it publishes nothing new.

Measured 2026-11-28 by this round,
on the merge's own control,
`foundation::diagnostic_lines_off_the_calling_thread::tests::the_notice_about_a_reader_that_stopped_is_said_once_at_its_bound`:
with `reader_stopped_notice: None` on the diagnostics' policy the control fails,
`one notice is said for one reader that stopped: []` against one expected,
`test result: FAILED. 0 passed; 1 failed`;
with the field as the merge wrote it the control passes,
`test result: ok. 1 passed; 0 failed`.
So the function is on a path a control reaches,
and the indirection is what tells a person that diagnostics stopped reaching them.
The pass could not see the line in either binary,
which stands:
the queue's thread holds the stream and the command ends before the notice reaches it.

### The no-change pass's L2, as the record states it

The merge `18327d05f6` was recorded as TypeScript under `e2e/` and records,
which ships nothing new.
What it changes:

- ten files under `e2e/`;
- seven under `src/native/`:
  two release files whose four changed lines are test-gated
  (`checkout_boot_units/boot_unit_facts.rs`,
  `mod tests;` to `pub(crate) mod tests;` under `#[cfg(test)]`,
  and `checkout_underlay_entry/underlay_entry_verdict.rs`,
  a `#[cfg(test)] mod guest_tests;` and its doc)
  and five `_tests.rs` files,
  two of them new;
- `README.md` (six lines) and `SPEC.md` (32 lines) at the package root;
- and `doc/runbook/zfs-checkout-migration-and-boot-setup.md`,
  outside the package.

What a release ships of that,
by `Cargo.toml`'s `include` list (`src/native/**/*.rs`,
`/README.md`,
`/SPEC.md` and data files,
and no `doc/` and no `e2e/`):
the changed `README.md` in the crate archive and in both published binary archives,
whose copy is the repository's file at the release commit
(`sha256` `79f9d8508de236421af2c6e288aec4fdf1fe5a8a96116eb63fc6cfb7d7895024` in alpha.52's archives
and `c3430c8f92b01ffae633ca68288a02078d8574e7eb82d03fcde34cc0ec8cae82` in alpha.53's),
the changed `SPEC.md` and the five test files in the crate archive,
and nothing of it in the compiled program,
because every changed release line is behind `#[cfg(test)]`.

### What this round changed

Each finding was reproduced before anything changed,
with a control that fails on the merge for the reason the finding gives.

The no-change pass's L6,
one start of the manager's program answered three times.
`source_guard/no_start_at_login_tests.rs` checked a line for `Command::new("systemctl")` in the
whole-tree reading,
and then the next-line reading found that same start at its own line and again at the line before it,
joined with it.
A one-line start planted in `service/service_instance.rs` was answered three times:

```text
service/service_instance.rs:107: a start of the manager's own program
service/service_instance.rs:106: a start of the manager's own program: pub fn a_planted_one_line_start() {     let _ = std::process::Command::new("systemctl").arg("enable");
service/service_instance.rs:107: a start of the manager's own program:     let _ = std::process::Command::new("systemctl").arg("enable");
```

The next-line reading now joins a line only where the call's `(` ends it,
and leaves a start the next line spells on its own to that line's turn;
the whole-tree reading asks that reading alone.
The same plant is one finding,
at line 107,
and the control `a_one_line_start_of_the_manager_s_program_is_one_finding`
asserts one finding at the start's own line where the merge answered two.

The no-change pass's L3,
a literal a comment line was said to hide.
The guard's top doc and the next-line reading's doc said a literal a comment line separates from
the call is not read,
and the reading joins the next code line,
so it is read.
Both docs say what the code does now,
and a plant in `the_manager_s_program_is_read_when_its_literal_is_on_the_next_line`
holds the shape:
`Command::new(`,
a comment line,
then `"systemctl"`,
is one finding at the call's line.

The no-change pass's L5,
`dbg!` read by nothing.
`dbg!` writes this process's shared standard error on the caller's own thread,
which is the write the note guard exists to refuse,
and no guard,
lint or linter named it.
A `dbg!` planted in `commit/commit_command_words.rs` left the note guard green
(`test result: ok. 6 passed; 0 failed`) before the change
and is a finding after it:

```text
commit/commit_command_words.rs:69 (dbg!) dbg!(text);
test result: FAILED. 5 passed; 1 failed
```

The guard's terse spellings include it beside `eprintln!` and `eprint!`,
and its control's spelling list reads it,
which fails without the list entry
(`dbg!(one); asks for the shared handle: []`).
The guard's top doc now also names the macro spelling it still cannot see,
one with a space before its `!`.

The behaviour pass's L5,
a check given the `add`'s reason for staging.
The `final-newline` hint said an index already holds these bytes wherever it printed.
An `add` has just staged them,
and a direct check stages nothing.
The check's answer is now its own `CommitCorrection`
(`CorrectsWhatNothingHasStagedUnlessTheSelectionIsReadOnly`,
beside `CorrectsWhatTheIndexHoldsUnlessTheSelectionIsReadOnly` for the `add`),
and its sentence is:

```text
Hint: this check staged nothing, so to correct the ending now, run `git monochromatic-git-policy-cli fix --
<path>`, and then either stage the file with `git add -- <path>` or commit it, as in `git commit -- <path>`; a
commit that names the file corrects the ending while this repository's settings keep `final-newline` on, but a
commit whose selection is read-only (`-p`, `-i` or `--include`) records the bytes it takes exactly as they are.
```

The control `a_direct_check_says_what_corrects_the_bytes_it_read` asserts that whole sentence,
and it failed on the merge with the false reason in `left`
(`assertion left == right failed` on the rendered `direct-check` finding);
the built executable's control `a_direct_check_gives_the_hint_whose_commit_corrects_the_bytes_it_read`
reads the same four wordings through the printed finding.

The two reads' L1 and L2 are the two sections above:
what `74f34f6af9` and `18327d05f6` do is stated there in place of "moves code" and
"TypeScript and records".

### What this round left

- The no-change pass's L4,
  that a release's record by this package's folder omits its path dependency's changes,
  belongs to the release tooling and its own issue.
- The no-change pass's L7,
  three aftermath sentences the compiler folded away,
  was weighed by the session and dismissed:
  the unreachable arms are the source's own,
  and a person sees the same sentence in both releases.
- The behaviour pass's L2,
  a registered slot whose folder is gone,
  belongs to the round that reads a resumed removal.
- The behaviour pass's L6,
  a guest arm's stale comment,
  waits for issue 6669's fix,
  and this round ran no guest.

### The gate read's H3: the forced arm's two rows against `SPEC.md`

`worktree_clone_forced_removal/holder_endings.rs` gives another account's task and a unit's task
`Ending::EndBlockersThenUnmount`,
with the evidence text "no round shows this kind's end",
and its control asserts that text.
`SPEC.md` said "A kind the evidence does not show is identified and refused,
never ended".
The accepted record is
`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`
(the first answer,
"B:
root handler",
and the fourth,
the pinned signal)
and the design it accepts,
`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`:
the two kinds are kinds (b) and (c) of "Who ends each kind of holder",
and one forced removal meets them by unmounting the clone inside the holder's namespace,
sending `SIGTERM`,
waiting,
and then `SIGKILL` to the blockers.
So the ending those rows carry is the accepted answer's,
and what no round has shown is the arm that would prove it,
which the row's `owed_arm` says and `forced_arm_numbers.rs` holds back by shipping no value.

The side that moved is `SPEC.md`,
and the module's own top doc with it.
`SPEC.md` now says that a kind the evidence does not show keeps the ending the accepted answer gives it,
that the row names the arm still owed,
and that the receipt's seven refusal words name what stopped the act.
The table's rows,
their evidence text,
the control's assertion on that text,
and the row that ends a locked holder of the account's own unit are unchanged.
The control also refuses a row for either kind that is a refusal,
so the sentence cannot come back without a table change:
`every_rows_evidence_says_what_is_shown_and_what_is_owed` asserts
that neither row's ending is `Ending::Refuse`.
No command reaches this arm,
so nothing a person meets changed.

### The files this round changed

- `src/native/source_guard/no_start_at_login_tests.rs`:
  one finding per start,
  the two docs that described a comment line wrongly,
  and the two controls.
- `src/native/source_guard/the_commit_path_writes_its_notes_off_its_thread_tests.rs`:
  `dbg!` in the terse spellings,
  its control's list,
  and what the guard cannot see.
- `src/native/policy/event_hints.rs`,
  `policy/policy_final_newline.rs` and their controls:
  the check's own `CommitCorrection` and its sentence.
- `src/native/invocation/management_tests.rs`,
  `src/native/invocation/wrapped_command_tests.rs` and
  `src/native/binary/binary_final_newline_hint_steps_tests.rs`:
  the hint each place expects,
  and the words each reads.
- `src/native/worktree_clone_forced_removal/holder_endings.rs` and its control:
  the two docs and the sentence the control refuses,
  with the table unchanged.
- `SPEC.md` and this package's guide:
  the two sentences the code contradicts.
- this record.

### The checks this round ran

Each once,
in `package/git-policy/cli`,
in its own `systemd-run --user --scope --slice=agent-work.slice`
with `MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight=1` and `RuntimeMaxSec`:

- the two `source_guard` modules this round changed,
  threaded,
  13 and 6 passed;
- `policy::event_hints::tests`,
  4 passed;
- `policy::policy_final_newline::hint_of_each_lifecycle_tests`,
  5 passed;
- `invocation::management::tests`,
  27 passed;
- `invocation::wrapped_command::tests`,
  12 passed;
- `native_binary`'s `final_newline_hint_steps`,
  6 passed;
- the merge's own notice control,
  1 passed,
  and red without the policy's function.
- `cargo fmt --check`,
  `cargo check --locked --offline --all-targets`,
  Clippy with and without `test-only-switches`,
  the whole `source_guard` module,
  and the repository's linter over this package.

### What could not be run, and why

- No guest:
  this machine's home is on a ZFS pool,
  no record authorizes a run to work on it,
  and the brief asks for no guest.
- No command opened `/dev/zfs`,
  changed ZFS,
  mounted anything,
  used `sudo` or started a service manager.
- No whole container gate,
  no mutation campaign and no independent review.
