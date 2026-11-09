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

Findings 1 and 2 are the measured consequences of an item the planning document already
lists as needing the owner's decision
(`doc/planning/cli-git-merge-auto-resolution.md`,
2026-11-03T03:39:36-05:00 entry,
"Needs the owner's decision" item 3):

- "The attribute source is the first side's tip;
  a merged `.gitattributes` of both sides is not built,
  and `attr.tree` is overridden."

The choice is recorded;
what its consequence costs is not,
and that is what the two findings add.

Already recorded,
and therefore not reported here:
the pass's install window (`doc/handover/cli-git-rust-implementation.md`,
"Open review findings",
which quotes the merge and repair audit's item 7 about a kill between the staging and the file write),
and the two limits this audit's inputs do not reach (a stage over 8 MiB,
and the cost bound).
Neither that section nor the planning document names findings 1 to 4;
the attribute-source item quoted above is the closest,
and it states the choice,
not the consequence measured here.

## What the route does

1. Git stops with an unmerged index.
   For a command a person types,
   `watch_before` and `resolve_after`
   (`merge_second_pass_worktree.rs:239`)
   keep only the paths that were not unmerged before the command,
   so a file being resolved by hand is never replaced.
2. `second_pass` (`merge_second_pass.rs:301`) keeps a path only when all three stages exist
   with one plain-file mode,
   no stage holds a NUL in its first 8000 bytes,
   no stage is over 8 MiB,
   and the `merge` attribute is `unspecified`,
   `set` or `text`
   (`merge_second_pass.rs:109`,
   `:224`).
3. `merge_by_word` (`word_merge.rs:386`) merges the line hunks Git left,
   then each hunk both sides changed by word,
   then the bytes between words.
   A hunk is taken only when one side equals the base there,
   or both sides equal each other
   (`word_merge.rs:266`);
   otherwise the whole file stays as Git left it.
4. The merged bytes become a blob
   (`hash-object -w --no-filters`),
   are staged as resolved
   (`update-index --index-info`),
   and are written over the worktree file
   (`checkout-index --force`),
   with the conflict stages put back if the write fails
   (`merge_second_pass_worktree.rs:193`).
5. One line per merged part goes to standard error,
   or one line for a file whose every region took one side,
   and one journal entry per merged file
   (`merge_second_pass.rs:463`,
   `:484`;
   `merge_second_pass_tree.rs:296`).
   When every conflicted file resolved,
   `finish` ends the operation the way a clean run does
   (`merge_second_pass_finish.rs`).
6. A merge computed from objects
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

Two comparisons say this is the wrapper's own choice and not Git's:

- The same input with the line in the base commit (`attributes-in-base-isolated`)
  is refused instead:
  exit `2`,
  core finding `merge-isolation/merge-conflict`,
  "merging it into the current branch conflicts in data.lock",
  nothing written.
  So the guard works when the first side's tree carries the line,
  and not otherwise.
- Real Git never produces that content for the pair:
  in a clean checkout (`attributes-added-by-theirs`)
  it stops with exit `1`,
  `data.lock` keeps Git's conflict markers,
  and `.gitattributes` is staged as a new file.

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
(`auto_push_reconcile_merge_commit.rs:111`,
`:114`)
and the commit replay
(`service_replay.rs:347`),
so the published and replayed merges have the same hole.

Suggestion:
ask `check-attr` against the tree the merge will produce
(`git check-attr --source=<tree>` with the merge tree)
rather than the first side's tip,
and treat a `.gitattributes` that is itself conflicted in that tree as unknown,
which keeps the file's conflict instead of merging it.

### 2. A merge in a worktree loses the `merge` exclusion when the exclusion line is itself in conflict

Reproduced (`attributes-guard-lost`).
Read from the code:
`merge_second_pass_worktree.rs:264` runs the pass's Git from the top of the worktree
with no attribute source,
so `git check-attr` (`merge_second_pass.rs:233`) reads the worktree copy of `.gitattributes`,
which during a conflicted merge is the copy holding Git's markers.

Input,
three commits:

- base:
  `.gitattributes` is `data.lock merge=binary`,
  and `data.lock` is `port=1 host=a zone=z`.
- ours:
  `.gitattributes` becomes `data.lck merge=binary`,
  and `data.lock` becomes `port=9 host=a zone=z`.
- theirs:
  `.gitattributes` becomes `data.lok merge=binary`,
  and `data.lock` becomes `port=1 host=b zone=z`.

Both sides change the guard line itself,
so `.gitattributes` conflicts,
and the worktree copy is then the marker file,
in which neither `*.lck` nor `*.lok` nor `data.lock merge=binary` is a whole line.

Real Git,
exit `1`:

```text
.gitattributes="<<<<<<< HEAD\ndata.lck merge=binary\n=======\ndata.lok merge=binary\n>>>>>>> topic\n"
data.lock="<<<<<<< HEAD\nport=9 host=a zone=z\n=======\nport=1 host=b zone=z\n>>>>>>> topic\n"
unmerged=[".gitattributes", "data.lock"]
```

The wrapper,
exit `1`:

```text
stderr="monochromatic-git-policy-cli: word-level merge: data.lock: part 1 of 1:
 base line 1, ours line 1, theirs line 1 became \"port=9 host=b zone=z\\n\"\n
 monochromatic-git-policy-cli: word-level merge: staged as resolved by the second merge pass: data.lock.
 Review a merged file with `git diff --cached -- <file>`; resolve the files Git still lists as conflicted,
 then finish the merge the way Git said above.\n"
.gitattributes="<<<<<<< HEAD\ndata.lck merge=binary\n=======\ndata.lok merge=binary\n>>>>>>> topic\n"
data.lock="port=9 host=b zone=z\n"
unmerged=[".gitattributes"]
status=["UU .gitattributes", "M  data.lock"]
```

The control `attributes-guard-intact` is the same merge with the guard line outside the conflict:
there Git prints `warning: Cannot merge binary files: data.lock (HEAD vs. topic)`,
the wrapper leaves `data.lock` alone,
and both arms agree on `UU` for both paths.
So the guard is lost exactly when the rule is in the conflicted hunk,
which is the case the person is about to resolve.

What it risks:
the pass removes the conflict stages of a file the base commit declared unmergeable,
while the person is still resolving the file that declares it.
If they resolve `.gitattributes` and commit,
the hybrid is committed with nothing else said.
The two versions survive in `HEAD` and `MERGE_HEAD`,
so no content is unrecoverable,
but the choice is gone:
`git checkout --merge -- data.lock` cannot rebuild the conflict once the stages are replaced
(`merge_second_pass_worktree.rs:201`).

Suggestion:
fail closed for the whole pass when any attribute file that can apply to a candidate
is itself unmerged
(`git ls-files -u` names `.gitattributes`),
since a rule that cannot be read is not a rule that says "text".

### 3. The part list gives coordinates the same command destroys

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

### 4. A finish that cannot commit says nothing in the repository changed, and drops the line that says what it did

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
and the same command on that machine (`git commit`) ends with the same sentence,
so the way out is not in the output either.
No content is lost,
which is why this is ranked last.

Suggestion:
print the closing line whenever the finish does not print a line of its own about the staged files,
or render the commit failure's last sentence from what the command as a whole changed.

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
  see finding 3.)
- Very long lines (`long-line`,
  one 400 kB line):
  left as Git's conflict.
- A file whose text already holds conflict markers (`markers-in-text`):
  Git merges it cleanly,
  so the pass is not involved,
  and both arms agree.
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
  leaves `MERGE_HEAD` and `MERGE_MSG` or `SQUASH_MSG` as a clean run leaves them,
  and keeps `MERGE_MSG` cleaned of the conflict hint.
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
  so findings 1 and 3 reach them by the same functions;
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
- Whether a `merge` exclusion added by the *first* side and removed by the second should keep guarding the file.
  This audit measured the two cases it can state
  (findings 1 and 2);
  the general question of which `.gitattributes` a merge should read is the open item quoted in
  "Decisions this route answers to".
- Windows and macOS,
  which the owner's pause leaves out.
