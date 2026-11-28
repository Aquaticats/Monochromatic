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
