# Conflict resolution route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
the code of this worktree at `9d4e5a9970`,
which differs from `main` (`bb50f746b5`) in one file,
`doc/handover/questions-for-the-owner.md`,
which this route does not read.
No production code was changed and no test was committed.

Scope:
`package/git-policy/cli/src/native/conflict/`,
the word pass in `src/native/merge/`
(`word_merge.rs`,
`merge_second_pass.rs`,
`merge_second_pass_conflicts.rs`,
`merge_second_pass_tree.rs`,
`merge_second_pass_worktree.rs`,
`merge_second_pass_finish.rs`,
`merge_isolation_apply.rs`),
the callers that hand it a conflict
(the forwarded merge,
rebase,
cherry-pick,
revert and pull,
the isolated merge,
the auto-push reconcile merge,
the commit replay),
and the sections of `package/git-policy/cli/SPEC.md` that bind them.
Linux only,
which is what the owner's pause leaves in scope.

Purpose:
the audits of the merge and repair route
(`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`)
and of the copy route
(`doc/planning/cli-git-copy-route-findings-on-main.md`)
judged recovery,
locks,
records and kills,
and neither judged the text an automatic resolution produces.
This page asks one question:
can the word-level second pass silently drop a change one side made,
or produce text neither side wrote,
and leave a commit at exit `0` with no line saying so.
It records what each answer rests on,
so nobody re-audits it.

Every finding names the input,
the exact result,
and whether it was reproduced.
A suggested fix is marked as a suggestion.
The harness and its output are in `${HOME}/temp/agent/pi-conflict-audit`:

- `mergeprobe.ts` builds one template repository per input
  (base on `main`,
  `ours` on `main`,
  `theirs` on `topic`),
  copies it twice,
  and runs the same merge once through `/usr/bin/git` 2.56.0
  and once through this worktree's built executable
  (`package/git-policy/cli/target/debug/monochromatic-git-policy-cli`),
  under the hermetic fixture environment the binary-level controls use
  (`PATH` with a `git`-named link first,
  a private `HOME`,
  `GIT_CONFIG_NOSYSTEM`,
  no journal socket).
  It prints the exit status,
  the standard output and error,
  the unmerged paths,
  the worktree bytes,
  the staged diff,
  `HEAD`,
  and the merge state files of both arms.
  Every case name in this page is a case there,
  and `report.txt` is its output for all of them.
- `wordprobe/src/main.rs` includes `src/native/merge/word_merge.rs` unchanged by path,
  so it runs the route's own source,
  and checks properties over generated inputs,
  with no repository and no Git.

## Decisions this route answers to

Owner words,
quoted from `package/git-policy/cli/SPEC.md`
("Merges and commits never fail"):

- "Auto merging should never fail."
- "Manual merge should never fail too."
- "never discard user data"

The decision of 2026-11-03
(`doc/planning/cli-git-merge-auto-resolution.md`,
"Merges a person runs"),
quoted as the coordinator relayed it:

- "Word-level second pass (Recommended)".
- "after Git stops,
  re-merge only the conflicted parts word by word (the method of Jujutsu);
  accept a file only when every part resolves;
  list each merged part on stderr and in the journal."
- "Second pass there too" for the wrapper's own unattended merges.
- "Finish like a clean merge",
  after every conflicted file resolved.

The rules of the specification that the findings answer to,
quoted with their lines:

- `SPEC.md:7214`:
  "accepts a file only when every part resolves,
  and lists each merged part on standard error and in the journal".
- `SPEC.md:7234`:
  "it never drops a byte that either side added or kept".
- `SPEC.md:7246`:
  "the `merge` attribute is unspecified,
  set or `text`
  (a driver,
  `-merge` and `merge=binary` keep the merge to their setting)".
- `SPEC.md:7258`:
  "Anything else is left exactly as Git left it,
  with the reason in the debug log."
- `SPEC.md:7506`:
  "The `doc/handover/*.md merge=union` line of `.gitattributes` therefore applies in them."

Finding 1 is the measured consequence of an item the planning document already
lists as needing the owner's decision
(`doc/planning/cli-git-merge-auto-resolution.md`,
2026-11-03T03:39:36-05:00 entry,
"Needs the owner's decision" item 3):

- "The attribute source is the first side's tip;
  a merged `.gitattributes` of both sides is not built,
  and `attr.tree` is overridden."

The choice is recorded;
what its consequence costs is not,
and that is what finding 1 adds.

Already recorded,
and therefore not reported here:
the pass's install window (`doc/handover/cli-git-rust-implementation.md`,
"Open review findings",
which quotes the merge and repair audit's item 7 about a kill between the staging and the file write),
and the candidate bounds the specification already states
(8 MiB,
and Git's 8000-byte binary probe).
Neither that section nor the planning document names findings 1 to 3;
the attribute-source item quoted above is the closest,
and it states the choice,
not the consequence measured here.

## What the route does

1.  Git stops with an unmerged index.
    For a command a person types,
    `watch_before` and `resolve_after`
    (`merge_second_pass_worktree.rs:239`)
    keep only the paths that were not unmerged before the command,
    so a file being resolved by hand is never replaced.
2.  `second_pass` (`merge_second_pass.rs:301`) keeps a path only when all three stages exist
    with one plain-file mode,
    no stage holds a NUL in its first 8000 bytes,
    no stage is over 8 MiB,
    and the `merge` attribute is `unspecified`,
    `set` or `text`
    (`merge_second_pass.rs:109`,
    `:224`).
3.  `merge_by_word` (`word_merge.rs:386`) merges the line hunks Git left,
    then each hunk both sides changed by word,
    then the bytes between words.
    A hunk is taken only when one side equals the base there,
    or both sides equal each other
    (`word_merge.rs:266`);
    otherwise the whole file stays as Git left it.
4.  The merged bytes become a blob
    (`hash-object -w --no-filters`),
    are staged as resolved
    (`update-index --index-info`),
    and are written over the worktree file
    (`checkout-index --force`),
    with the conflict stages put back if the write fails
    (`merge_second_pass_worktree.rs:193`).
5.  One line per merged part goes to standard error,
    or one line for a file whose every region took one side,
    and one journal entry per merged file
    (`merge_second_pass.rs:463`,
    `:484`;
    `merge_second_pass_tree.rs:296`).
    When every conflicted file resolved,
    `finish` ends the operation the way a clean run does
    (`merge_second_pass_finish.rs`).
6.  A merge computed from objects
    (the isolated merge,
    the reconcile merge,
    the replay)
    takes the tree from `git merge-tree --write-tree -z`
    and rewrites it with the merged blobs
    (`merge_second_pass_tree.rs:193`).

## Findings

### 1. A merge from objects ignores a `merge` exclusion the merge introduces, and commits the file at exit `0`

Reproduced (`attributes-added-by-theirs-isolated`).
Read from the code:
`merge_isolation_apply.rs:134` names the first side's tip
(`attribute_tree_variables(head, &probe)`)
as the attribute tree of a merge that reads no worktree,
unless the person's own `--attr-source`,
`GIT_ATTR_SOURCE` or a resolving `attr.tree` already names one
(`merge_second_pass_tree.rs:141`),
and `merge_second_pass.rs:233` asks `git check-attr` for the `merge` attribute of each candidate
under that tree.

Input,
three commits:

- base:
  `data.lock` is the single line `port=1 host=a zone=z`.
- ours (`main`):
  `data.lock` becomes `port=9 host=a zone=z`.
- theirs (`topic`):
  adds `.gitattributes` with the line `data.lock merge=binary`,
  and changes `data.lock` to `port=1 host=b zone=z`.
- the checkout also holds a file another session staged,
  which is the documented trigger for the isolated merge
  (`merge_isolation.rs:180`).

The wrapper's merge:

```text
code=0
stdout="Merge made by the 'ort' strategy.\n .gitattributes | 1 +\n data.lock      | 2 +-\n
 2 files changed, 2 insertions(+), 1 deletion(-)\n create mode 100644 .gitattributes\n"
stderr="monochromatic-git-policy-cli: word-level merge: data.lock: part 1 of 1:
 base line 1, ours line 1, theirs line 1 became \"port=9 host=b zone=z\\n\"\n"
HEAD="af23685 Merge branch 'topic'"
data.lock="port=9 host=b zone=z\n"
.gitattributes="data.lock merge=binary\n"
```

The merge commit holds `.gitattributes` declaring `data.lock` `merge=binary`,
and the same commit holds `data.lock` textually merged from both sides,
at exit `0`.
The attribute is invisible because the first side's tree has no `.gitattributes`:
the file arrives with the side being merged.

Two comparisons bound what this shows:

- The same input with the line in the base commit (`attributes-in-base-isolated`)
  is refused instead:
  exit `2`,
  core finding `merge-isolation/merge-conflict`,
  "merging it into the current branch conflicts in data.lock",
  nothing written.
  So the code that honours the setting works;
  what reaches it is the first side's tree.
- Real Git never produces that content for the pair:
  in a clean checkout (`attributes-added-by-theirs`)
  it stops with exit `1`,
  `data.lock` keeps Git's conflict markers,
  and `.gitattributes` is staged as a new file.
  This shows Git does not word-combine the file;
  whether a rule the merge introduces should govern the pass is the open item quoted in
  "Decisions this route answers to",
  and this page measures what the current answer costs.

What it risks:
a file whose `merge` setting says a driver,
`-merge` or `merge=binary` owns its content
is word-merged and committed,
which is the class of file people exclude precisely because a hybrid is not valid
(a lockfile a tool must rewrite,
a certificate or bundle,
anything a machine regenerates).
`SPEC.md:7246` says the setting "keep[s] the merge to their setting";
`SPEC.md:7506` says the `.gitattributes` line "therefore applies in them",
which holds only when the first side's tree already has it.
The same source feeds the auto-push reconcile merge
(`auto_push_reconcile_merge_commit.rs:115`)
and the commit replay
(`service_replay.rs:347`),
so the published and replayed merges have the same hole.

Suggestion:
ask `check-attr` against the tree the merge will produce
(`git check-attr --source=<tree>`,
which Git 2.56.0 accepts with `--stdin -z`;
measured on the pair above:
`--source=HEAD^{tree}` answers `unspecified` for `data.lock`
and `--source=topic^{tree}` answers `binary`)
rather than the first side's tip,
and treat a `.gitattributes` that is itself conflicted in that tree as unknown,
which keeps the file's conflict instead of merging it.

### 2. The part list gives coordinates the same command destroys

Reproduced (`plain-resolved`).
Read from the code:
`line_span` (`word_merge.rs:362`) computes the part's line numbers in the base,
ours and theirs texts,
`part_text` (`merge_second_pass.rs:463`) prints them,
and `install` (`merge_second_pass_worktree.rs:201`)
then replaces the three index stages with the merged entry.

The line a person gets:

```text
monochromatic-git-policy-cli: word-level merge: notes.md: part 1 of 1:
 base line 1, ours line 1, theirs line 1 became "ALPHA UNO\n"
```

What the same repository answers afterwards:

```text
$ git show :2:notes.md
fatal: path 'notes.md' is in the index, but not at stage 2
hint: Did you mean ':0:notes.md'?
$ git show :1:notes.md
fatal: path 'notes.md' is in the index, but not at stage 1
```

The only coordinates given point at texts the command has just removed from the index.
For a merge the person can still find them
(`HEAD:notes.md`,
`MERGE_HEAD:notes.md`,
the merge base),
and for a rebase,
a cherry-pick,
the reconcile merge or the replay they must reconstruct
which two commits those were,
with nothing in the line or the journal entry naming them.
The journal entry carries the file,
the operation and the part count
(`merge_second_pass.rs:507`),
not the three object names.

What it risks:
the decision of 2026-11-03 is that each merged part is visible so a person can check it;
a part whose three inputs cannot be opened by the names in the line cannot be checked
without work the line gives no hint of,
and a wrong resolution then looks reviewed.

A second,
smaller effect of the same line:
its excerpt comes from `escape_debug` over lossy UTF-8,
so a part of a file that is not UTF-8 cannot be compared with the bytes.
The case `non-utf8` resolves to `caf\xe9 UNO\nna\xefve ZWEI\n`
and its line says `became "cafï¿½ UNO\nnaï¿½ve ZWEI\n"`.

Suggestion:
put the three stage object names in the part line and in the journal entry,
so `git cat-file blob <oid>` reaches each input after the stages are gone.

### 3. A finish that cannot commit says nothing in the repository changed, and drops the line that says what it did

Reproduced (`plain-resolved`),
on a machine whose account has no user service manager,
which is the wrapper's own supported state
(`SPEC.md`,
"Machines with no way to reach the service").
Read from the code:
`resolve_after` prints its closing line only when no finish is planned
(`merge_second_pass_worktree.rs:312`),
and a planned finish returns its commit's status
(`:330`),
whose failure text ends with "Nothing in the repository changed."
(`landing/service_messages.rs:156`,
`:177`).

Input:
`notes.md` conflicts,
ours changes the second word of line 1,
theirs changes the first,
which the word pass resolves to `ALPHA UNO`.

What the person sees:

```text
stderr="monochromatic-git-policy-cli: word-level merge: notes.md: part 1 of 1:
 base line 1, ours line 1, theirs line 1 became \"ALPHA UNO\\n\"\n
 This computer cannot start the commit service on demand, so this commit was not made.
 This account has no user service manager. Turn it on with `loginctl enable-linger` ...
 or commit from a system that does. Nothing in the repository changed.\n"
code=1
```

The state at the same moment:

```text
status=["M  notes.md"]
notes.md="ALPHA UNO\nbeta two\ngamma three\n"
unmerged=[]
```

The repository did change:
the merge no longer has a conflict,
and the resolved file is staged.
Because the finish was planned,
the closing line that would have said
"staged as resolved by the second merge pass"
was suppressed,
so nothing in the output says the merge's files are now resolved.

What it risks:
the person is told the command changed nothing while it left a resolved file staged,
and the same command on that machine (`git commit`)
ends with the same sentence,
so the way out is not in the output either
(verified:
the wrapped `git commit --no-edit` in this arm exits `1` with exactly that text).
No content is lost,
which is why this is ranked last.

Suggestion:
print the closing line whenever the finish does not print a line of its own about the staged files,
or render the commit failure's last sentence from what the command as a whole changed.

## Withdrawn by measurement

The first draft of this page carried a second finding:
a merge in a worktree loses a `merge` exclusion when the exclusion line is itself in conflict,
because `git check-attr` reads a worktree `.gitattributes` that holds Git's markers.
The measurement does not support it,
and it is recorded here so nobody derives it a second time.

The input that suggested it (`attributes-guard-lost`) is:

- base:
  `.gitattributes` is `data.lock merge=binary`,
  and `data.lock` is `port=1 host=a zone=z`.
- ours:
  `.gitattributes` becomes `data.lck merge=binary`,
  and `data.lock` becomes `port=9 host=a zone=z`.
- theirs:
  `.gitattributes` becomes `data.lok merge=binary`,
  and `data.lock` becomes `port=1 host=b zone=z`.

The wrapper word-merged `data.lock` where Git left it conflicted,
but the cause is not the markers:
neither side's tree,
and not the merge result,
guards `data.lock` any more,
so reading it as text is what the attribute source in force says,
and Git's own worktree read agrees.
Two inputs settle it:

- `attributes-two-attributes`:
  both sides keep `data.lock merge=binary` on the same line and differ in the third attribute
  (`merge=binary -text` against `merge=binary text`),
  so the guard line itself is inside the conflict hunk and the worktree copy is a marker file.
  Both arms print `warning: Cannot merge binary files: data.lock (HEAD vs. topic)`,
  leave both paths `UU`,
  and the wrapper runs no pass at all.
  Git's markers surround whole lines and never split one,
  so a rule both sides still carry survives in the marker file.
- `attributes-guard-intact`:
  with the guard line outside the conflict,
  both arms also leave `data.lock` alone.

What the draft got right is the direction finding 1 takes:
which `.gitattributes` a merge reads is the open item quoted in
"Decisions this route answers to",
and the worktree arm reads the copy Git left in the worktree,
markers included,
which is the source Git itself uses for a worktree merge.

## Checked and found sound

Each item names the `mergeprobe.ts` case and what the route did.
Unless an item says otherwise,
the wrapper's arm matches real Git's arm or leaves Git's conflict exactly as Git left it.

- The merge itself (`wordprobe`,
  20000 generated bases with line edits,
  word edits,
  insertions,
  deletions,
  moves and repeated identical lines):
  when one side equals the base the result is the other side's bytes exactly,
  every time;
  and swapping the two sides gives the same bytes and the same verdict,
  every time.
  No failure in 20000 inputs.
- Two sides editing one word of adjacent lines (20000 generated pairs):
  every pair resolved,
  and both edited lines survived whole.
  This is the shape Git conflicts on and the pass exists for.
- Both sides appending different text at one point (`both-append`):
  left as Git's conflict,
  which is what `SPEC.md:7237` promises.
- One side deleting a line the other edits (`delete-modify`):
  left as Git's conflict.
- Repeated identical lines,
  with one side moving a block and the other editing inside it
  (`duplicate-lines-move`,
  `repeated-block-edit`):
  resolved,
  and both changes survive,
  in the one order the base gives them.
- Whitespace-only reindentation on one side and a word edit on the other
  (`whitespace-reindent`):
  resolved to the reindented line carrying the other side's edit.
- Line-ending conversion on one side and an edit on the other
  (`line-endings`,
  `line-endings-both-sides`):
  left as Git's conflict.
  The pass does not resolve CRLF against LF,
  which is the safe side of this audit's question.
- Text that is not UTF-8 (`non-utf8`):
  resolved to exactly both sides' edits,
  byte for byte.
  (The report line is lossy;
  see finding 2.)
- Very long lines (`long-line`,
  one 400 kB line):
  left as Git's conflict.
- A file whose text already holds conflict markers (`markers-in-text`):
  Git merges it cleanly,
  so the pass is not involved,
  and both arms agree.
- A guard line that both sides still carry,
  inside the conflict hunk itself (`attributes-two-attributes`):
  the worktree `.gitattributes` is then a marker file,
  the rule still reads as `merge=binary`,
  no pass runs,
  and both arms agree on `UU` for both paths.
  (The draft finding this replaced is in "Withdrawn by measurement".)
- A guard line outside the conflict (`attributes-guard-intact`):
  the wrapper leaves the guarded file alone,
  as Git does.
- A guard line only the base carries,
  and both sides rename (`attributes-guard-lost`):
  the wrapper word-merges the file and Git leaves it conflicted,
  which is the attribute source talking,
  not the markers;
  the input is in "Withdrawn by measurement" because it does not support the finding it first suggested.
- A file whose last line has no newline on one side (`trailing-newline`):
  left as Git's conflict.
- Both sides changing the whitespace between words differently (`tab-and-space`):
  left as Git's conflict;
  the pass does not choose one side's spacing.
- A path that is a symbolic link (`symlink`):
  never a candidate,
  because the stages' modes are not a plain file's
  (`merge_second_pass.rs:194`),
  and the unrelated conflict beside it stays as Git left it.
- A mode change on one side (`mode-change`):
  never a candidate,
  for the same reason,
  and the file's conflict stays.
- Binary content with a NUL in its first 8000 bytes (`binary-nul`):
  never a candidate,
  and both arms leave Git's conflict.
- A rename on one side and an edit on the other (`rename-and-edit`):
  Git merges it cleanly,
  so the pass is not involved.
- `--no-commit` (`no-commit-resolved`)
  and `--squash` (`squash-resolved`):
  after the pass resolved the file,
  the wrapper prints Git's own sentence,
  exits `0`,
  stages the resolved file,
  keeps `MERGE_HEAD` and `SQUASH_MSG` as Git left them,
  and cuts the conflict hint from `MERGE_MSG`.
- A file whose one region cannot resolve beside another that can:
  the whole file stays as Git left it,
  which a stored control and this audit's reading of `merge_by_word` both confirm.
- No path installs a merged file without a line naming it:
  every `Verdict::Merged` reaches `announce`
  (`merge_second_pass_worktree.rs:298`,
  `merge_second_pass_tree.rs:296`),
  and the reconcile merge and the replay report the same list in the commit and in the event
  (`auto_push_reconcile_merge_commit.rs:173`,
  `service_replay.rs:305`).

## What I could not determine

- The reconcile merge and the commit replay end to end.
  Both were read,
  and both call the same `attribute_tree_variables` and `resolve_tree`,
  so findings 1 and 2 reach them by the same functions;
  exercising them needs a service,
  a remote and a staged push,
  which this audit did not build.
- A person's merge whose finish commits.
  This host's account has no user service manager,
  so the finish's `git commit` cannot start the service,
  and the completed merge was observed only through the isolated merge,
  which commits from objects and exits `0` (finding 1's arm).
  The staged state,
  the exit statuses and every message were observed.
- Whether the accepted class of "text neither side wrote" is acceptable for structured files.
  `one-line-json-object` resolves `{"alpha": 1, "beta": 2}`
  with ours changing `alpha` and theirs changing `beta`
  into `{"alpha": 3, "beta": 4}`,
  and `call-arguments` resolves two argument edits in one call into one call with both.
  The owner accepted the second shape
  (`doc/planning/cli-git-merge-auto-resolution.md`,
  "Open design points",
  "The word-level pass can produce text neither side wrote (`scale(10, 20)`);
  the guard options are in ..."),
  and the first is the same class,
  so this audit reports it as accepted rather than as a finding.
- Whether a `merge` exclusion added by the **first** side and removed by the second should keep guarding the file.
  This audit measured the case it can state (finding 1),
  and the two inputs that withdrew the worktree variant
  ("Withdrawn by measurement");
  the general question of which `.gitattributes` a merge should read is the open item quoted in
  "Decisions this route answers to".
- Windows and macOS,
  which the owner's pause leaves out.

## Fixed after this audit

Branch `fix/git-policy-cli-conflict-resolution-attributes-and-part-list`.
The page above stands as the record of the audited revision.
Each finding got a control that was written first and shown failing,
then passed with the fix,
and failed again with the fix removed.

- Finding 1,
  `cb2a9f230a`:
  the second pass reads the `merge` attribute from the tree `git merge-tree` produced
  (`git check-attr --source=<tree>`),
  so a rule the merge introduces governs the pass;
  the tree is named only where the person named no source,
  and a `.gitattributes` the merge left conflicted names no source at all,
  which keeps every candidate of that merge conflicted.
  `git merge-tree` itself still reads the first side's tip,
  which is the choice the planning record leaves to the owner.
  Controls:
  `a_merge_rule_the_merge_itself_introduces_keeps_the_isolated_merge_from_merging_the_file`,
  `a_rule_the_merge_itself_introduces_governs_the_second_pass`,
  `a_gitattributes_the_merge_left_conflicted_names_no_rule`,
  `the_merge_result_tree_is_named_only_where_the_person_named_no_source`.
- Finding 2,
  `ac7b52c406` (the module split that followed is `18eb5bfb96`):
  every part line ends with the three blobs it came from
  (`from base blob <oid>, ours blob <oid>, theirs blob <oid>`),
  one sentence names `git cat-file blob <oid>`,
  and the journal entry carries the same three object names in
  `CLI_GIT_WORD_MERGE_BLOBS`.
  The line spans stay as the coordinates of the part inside those blobs.
  Controls:
  `a_part_line_names_each_input_as_a_blob_that_survives_the_resolution`,
  the hint assertion of `an_isolated_merge_with_an_adjacent_edit_is_completed_and_listed`,
  `the_report_lines_name_the_file_and_number_the_parts_and_escape_the_text`,
  `the_journal_entry_names_the_file_the_operation_and_the_parts`,
  `the_notice_lists_each_merged_part_and_is_absent_without_one`.
- Finding 3,
  `ed47cef2af`:
  the line that names the files staged as resolved is printed whether or not a finish follows,
  and a finish that ends with a non-zero status is followed by one line naming the state
  the person will find,
  with the command that ends that operation.
  Controls:
  `a_finish_that_cannot_reach_the_commit_service_names_what_changed`,
  `the_closing_words_say_whether_a_conflict_remains`.

Still open after this work:

- The reconcile merge and the replay end to end,
  which this audit could not exercise either.
  Both pass their `AttributeProbe` to the same `resolve_tree`,
  so finding 1's fix reaches them by the same function,
  and no control starts either with a `.gitattributes` the merge introduces.
- The commit path still ends a failure it reports with "Nothing in the repository changed.",
  which is true of that commit and false of the merge that staged the files before it.
  The wrapper's own line after it is now the last word and the true one;
  suppressing the commit's sentence would mean passing the state of the index into
  the landing messages,
  which no finding required.
- Which `.gitattributes` `git merge-tree` itself should read
  stays the owner's item,
  unchanged by this work.

## The cross-vendor review of that branch, and what it found

The review of `fix/git-policy-cli-conflict-resolution-attributes-and-part-list`
(model `openai/gpt-6.1-sol`,
2026-11-09)
judged the branch and reported seven findings,
each reproduced here before it was fixed,
and each with a control that failed first and failed again with the fix removed.
The page above stands as the record of the audited revision.

- Finding 1,
  a resolution another command staged was replaced by `install`,
  `ad36b527dc`:
  the index stages of each merged path are read again just before the install,
  and the worktree file's device,
  inode,
  length and modification time are compared with what they were when the stages were read;
  a path that moved is left as it is and named.
  Controls:
  `a_resolution_another_command_staged_is_left_alone`,
  `a_worktree_file_edited_after_it_was_read_is_left_alone`,
  `a_path_that_did_not_move_is_installed`.
- Finding 2,
  a worktree `.gitattributes` holding Git's markers was read as a rule,
  `2a99eb2035`:
  a `.gitattributes` among the unmerged paths makes the source `Unknown`,
  which the tree arm already answered.
  Control:
  `a_gitattributes_the_merge_left_conflicted_names_no_rule_for_the_worktree_pass`.
- Finding 3,
  `-c attr.tree=<spec>` on the command line was not read,
  `924670342b`:
  the invocation's `-c` pairs are read before the configuration files.
  The remaining gap is `--config-env=attr.tree=<variable>`,
  which is the same mechanism and which `config_options` drops before the pass's own `check-attr`.
  Controls:
  `a_configuration_option_on_the_command_line_names_the_tree`,
  `a_configuration_option_on_the_command_line_names_the_tree_of_an_isolated_merge`.
- Finding 4,
  a merge driver that never returns held the reconcile merge for good,
  `a223881c35`:
  every command of the reconcile merge's second pass runs under the push query's bound,
  with its request on standard input and an answer ceiling of one stage per name it asks for.
  Control:
  `a_merge_driver_that_never_returns_stops_the_reconcile_merge`,
  which hung until the package's unit-test watchdog ended the run before the fix.
- Findings 5 and 6,
  the line after a finish described the pre-continuation state and named `git commit` for a rebasing pull,
  `662509bde4`:
  the index is read again after the failed step and the ending command comes from the state files.
  Controls:
  `a_rebase_that_stops_again_is_told_what_is_conflicted_now`,
  `a_rebasing_pull_that_stops_again_is_told_to_continue_the_rebase`,
  `the_ending_command_is_the_one_the_state_files_name`.
- Finding 7,
  a worktree path ending in a space became a false replay conflict,
  `4bdc760360`:
  the replay's `rev-parse --show-toplevel` output keeps every byte except its line break.
  Control:
  `the_toplevel_keeps_a_trailing_space_in_a_directory_name`.
  The reconcile merge reads the same line through `git_output`,
  whose JavaScript trim is right for every other caller of it,
  so that call still trims a trailing space;
  it is recorded here as open rather than changed with one caller in view.

The gap the audit left open is closed for both unattended merges:
`a_rule_the_remote_introduces_keeps_the_reconcile_merge_from_merging_the_file` and
`a_rule_the_captured_side_introduces_keeps_the_replay_from_merging_the_file`
start a merge whose own `.gitattributes` excludes the conflicted file,
and both fail when `merge_attribute_source` is forced to `Invocation`.

## The second cross-vendor review, and what it found

The second review of the same branch
(model `openai/gpt-6.1-sol`,
2026-11-09)
reported five findings at `[P1]` and three at `[P2]`.
Each was reproduced against the branch as it stood;
the owner ordered work on the four that were not the window,
and instructed that the window be recorded rather than closed.

- `[P1]` the baseline of a path was read after the merge was computed,
  `ede571f4af`:
  an edit made while the word merge ran became the baseline and was then overwritten.
  The baseline now precedes the merge.
  Control:
  `an_edit_made_while_the_pass_merges_is_not_replaced`,
  which waits for the pass's own `git check-attr` inside the command.
- `[P1]` a configuration key was matched case-sensitively,
  `773faf0865`:
  `git -c Attr.Tree=rules merge` reached the pass as no choice at all.
  The key is now read through `canonical_key`,
  which is the wrapper's one spelling of a configuration key and which became public for this.
  Controls:
  `a_configuration_option_in_any_letter_case_names_the_tree`,
  `a_configuration_option_in_any_letter_case_names_the_tree_of_an_isolated_merge`.
- `[P1]` a merge driver that never returns but keeps writing was unbounded,
  `ad7549e99e`:
  the push stall window cannot see it,
  because the driver moves bytes in every sample.
  A command of a commit's merge step now runs under a total ceiling of 300 seconds
  (`CLI_GIT_PUSH_MERGE_CEILING_SECONDS`),
  with `SingleFlightError::Ceiling` saying what was not changed.
  Control:
  `a_merge_driver_that_keeps_working_is_stopped_at_the_merge_ceiling`,
  which timed out at 300 seconds with the stall window restored.
- `[P1]` different bytes of the same length with a restored modification time passed the check,
  `f1ffeff185`:
  the guard compares the conflicted file's bytes now,
  read up to three times the pass's 8 MiB stage limit.
  Control:
  `a_same_length_resolution_with_a_restored_time_is_not_replaced`.
- `[P1]` a second process inside the window between the check and the install still loses its resolution.
  Recorded in `SPEC.md` with what the person sees there,
  as the owner instructed;
  no mechanism was added.
- `[P2]` a line about an index that could not be read again claimed no file was conflicted,
  `08b765358e`:
  the list is an `Option` now.
  Control:
  `a_line_about_an_unread_index_does_not_claim_a_clean_one`.
- `[P2]` the replay's toplevel parser removed a line break that belonged to a directory's name,
  `50622d0ff3`:
  exactly one terminator is removed.
  Control:
  `the_toplevel_keeps_a_line_break_that_belongs_to_a_directory_name`.
- `[P2]` the remaining count included a resolution another command had staged.
  Not reproduced:
  in a fixture of 200 conflicted files whose resolution is written and staged by another command inside the pass,
  that command's `git add` completed after the fresh index read,
  so the count was right at the instant it was computed and the merge was installed whole.
  The reviewer's sequence needs a pause inside the wrapper,
  which this work did not build;
  the one-line change (count from the fresh index) exists but has no control,
  so it was not made.
  Stays open.
