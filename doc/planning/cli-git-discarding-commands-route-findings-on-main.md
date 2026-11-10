# Discarding-command route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
`main` at `9d4e5a9970`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-discarding-commands-route-findings-on-main`.
No production code was changed and no test was committed.
The scratch scripts are in
`${HOME}/temp/agent/pi-discarding-route`,
and each result below names the script that produced it.
Every measurement used a debug binary built from the audited revision
(`mise exec -- cargo build --locked --offline --bin monochromatic-git-policy-cli`,
3 minutes 1 second),
started by path with `PATH=/usr/bin:/bin`,
in disposable repositories under that folder.
The route's decisions do not depend on the build profile.
Scope:
the code that decides whether a discarding command runs
(`src/native/command/`,
the deciding parts of `src/native/invocation/` and `src/native/policy/`,
and `src/native/conflict/`),
on Linux only.
The owner's pause covers Windows and macOS,
which this page does not describe.

Purpose:
answer one question.
Can any spelling of a command reach real Git and lose a person's uncommitted work
with the wrapper's approval or without its notice?
The answer is yes,
in six reproduced sequences,
plus a window that is read from the code alone.
Nothing in this page is recorded in the handover's `## Open review findings` section:
that section's open audit-history question is about the timing logs and the push trigger log,
which keep one generation,
while the audit file itself keeps numbered generations on `main`
and this page confirms that.

## What the route decides

Two decisions stand between a command line and real Git,
plus one carve-out that runs before both.

### The argument reading

`decide_linked_worktree`
(`src/native/command/rule_linked_worktree.rs:318`)
takes the command word and the region after it from
`command_tokens`
(`src/native/invocation/global_arguments.rs:196`),
which walks Git's own global options,
and returns one of three answers:

- `Pass`,
  which means no policy decision and no audit line at all;
- `NeedsEffectiveTarget(command)`,
  for a form that overwrites or deletes by itself;
- `WhenDirty(command)`,
  for a form that only moves `HEAD`.

Each subcommand word has its own reader,
all of them built on one tokenizer that follows Git 2.56.0 `parse-options.c`
(`src/native/command/command_options.rs:421`):

- `stash`:
  `parse_stash_region` (`command_stash.rs:236`) reads the subcommand word and its own table.
  Only `list`,
  `show` and `create` read
  (`command_stash.rs:328`);
  every other form,
  and a region the stash parser refuses,
  is `NeedsEffectiveTarget(Stash)`.
- `clean`:
  `parse_clean_region` (`command_clean.rs:109`) and `clean_changes_worktree` (`:150`),
  where only the last `--dry-run`/`--no-dry-run` decides,
  so `-ndx` is `Pass` and `-fdx` is `NeedsEffectiveTarget(Clean)`.
- `reset`:
  `parse_reset_region` (`command_reset.rs:190`),
  where `--soft` is `Pass`,
  `--hard`,
  `--merge` and `--keep` are `NeedsEffectiveTarget(ResetHard)`,
  and the default and `--mixed` are `NeedsEffectiveTarget(ResetMixed)`.
- `checkout`:
  `checkout_discard` (`command_discard.rs:85`),
  where a force,
  patch,
  stage (`--ours`,
  `--theirs`),
  index read from a file,
  a `--` boundary or two or more positionals are `Overwrites`,
  one positional is `MovesHead`,
  and none is `Harmless`.
- `switch`:
  `switch_discard` (`command_discard.rs:113`),
  where force and `--discard-changes` are `Overwrites`,
  a named target is `MovesHead`,
  and nothing is `Harmless`.
- `restore`:
  `restore_discard` and `restores_index_only` (`command_discard.rs:192`),
  where every form is `Overwrites` except a request for usage text,
  and `--staged` without `--worktree` is reported as `RestoreStaged`.
- `rm`,
  `mv`,
  `checkout-index`:
  `force_discard` (`command_discard.rs:272`),
  a byte scan of the tokens before the first `--`
  for `-<cluster with f>` or `--force` in any abbreviation of four characters.
- `read-tree`:
  `read_tree_discard` (`command_discard.rs:294`),
  `--reset` from five characters.
- `merge`,
  `rebase`,
  `cherry-pick`,
  `revert`,
  `am`:
  `abort_discard` (`command_discard.rs:319`),
  `--abort` from four characters.

Two facts about that reading carry every finding on this page:

- A region the reader refuses is `Pass`,
  because `guarded_decision` returns `OptionError` and line 325 unwraps it to
  `LinkedWorktreeDecision::Pass`
  (`src/native/command/rule_linked_worktree.rs:325`).
  Measured:
  `git checkout --no-such-option -- tracked.txt` exits 129 from Git,
  writes no audit file,
  and leaves the worktree file alone
  (`refuse.ts`).
  The reading therefore protects exactly the regions its tables model.
- The word it reads is the word on the command line,
  never the command Git ends up running.
  `resolve_forwarded_command`
  (`src/native/invocation/forwarded_command.rs:370`)
  expands aliases for the copy route
  (`src/native/invocation/entry_forward_action_planning.rs:77`)
  and for pushes,
  and the policies never ask it.

### The measurement

`check_linked_worktree`
(`src/native/policy/policy_checks.rs:337`)
turns one `GuardedCommand` into one outcome:

- the conflict carve-out first,
  for `checkout` with paths and for `restore` only,
  while `MERGE_HEAD`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`,
  `rebase-merge` or `rebase-apply/rebasing` exists
  (`src/native/policy/policy_checks.rs:359`):
  `saved-conflict` after the copies are written and synced,
  or a rejection naming the paths that are not conflicted;
- an overwrite in the main worktree or outside every worktree is rejected at once
  (`src/native/policy/policy_checks.rs:389`,
  `resolve_linked_worktree` at `rule_linked_worktree.rs:341`);
- a repository under an allowlisted tool cache is `exempt`
  (`src/native/policy/policy_checks.rs:393`);
- a `HEAD` move in a Git directory with no index file is `allowed`
  (`:399`),
  which is Git's own initial checkout;
- everything else is decided by one `git status`
  (`src/native/policy/policy_checks.rs:403`),
  run with `--no-optional-locks`,
  `--porcelain=v1`,
  and `--untracked-files=no`,
  or `--untracked-files=normal` for `clean`
  (`src/native/command/rule_worktree_dirty.rs:56`,
  `dirty_scope` at `src/native/command/rule_linked_worktree.rs:180`).

So the inputs reach these outcomes:

- main worktree,
  any overwriting form:
  rejected,
  `dirty` null,
  before any status runs.
- outside every worktree,
  any overwriting form:
  rejected.
- bare repository:
  `EffectiveTarget::OutsideWorktree`,
  so rejected.
- allowlisted cache,
  any form:
  exempt,
  runs.
- linked worktree,
  overwriting form,
  tracked change present:
  rejected with `dirty` true.
- linked worktree,
  overwriting form,
  no tracked change:
  allowed with `dirty` false.
- main or linked worktree,
  `HEAD` move,
  tracked change present:
  rejected.
- main or linked worktree,
  `HEAD` move,
  no tracked change:
  allowed.
- a form that reads only,
  or a region the tables refuse:
  no decision,
  no audit line.
- a status Git cannot read:
  an engine failure,
  exit 2,
  never "clean"
  (`src/native/policy/policy_checks.rs:412`).

### What is written, and what is kept

The audit log is one JSON line per decision,
appended before Git runs,
at `<common-git-dir>/monochromatic-git-policy-cli-audit.jsonl`
(`src/native/conflict/discard_audit.rs:362`).
It carries the form,
the outcome,
the worktree kind,
the Git directory,
the directory the command ran in,
the arguments as Git will see them,
the dirtiness when it was measured,
and,
for a carve-out,
the copies.
A torn line never swallows the next record,
because a file whose last byte is not a newline gets one first
(`src/native/conflict/discard_audit.rs:384`).
The line is written with one `write`
(`src/native/conflict/discard_audit.rs:405`)
and is never synced.

Saved copies exist in exactly one place:
the conflict carve-out
(`src/native/conflict/conflict_side_guard.rs:259`,
`save_copies` at `src/native/conflict/conflict_sides.rs:113`).
They go to
`<common-git-dir>/monochromatic-git-policy-cli/discarded/<YYYY-MM-DDTHH-MM-SS>/<path>`,
each file is written and every directory above it is synced before the caller lets Git run,
and a save whose directory was just made is never chosen by the prune that follows
(`src/native/conflict/conflict_side_guard.rs:152`).
The limits are the owner's:
"Prune by both age and size.",
recorded as standing over "never discard user data" for saved copies
(`doc/handover/cli-git-rust-implementation.md`,
`### Picks made under the owner's delegation`).

No other allowed form saves anything.
A `clean`,
a `reset --hard`,
a `checkout -f`,
a `stash drop` and a `worktree remove`
that the route allows leave no copy behind.

## The spellings I ran, and what each reached

Run through the real binary in a disposable repository with one modified tracked file
(`evidence.ts`,
section `S1`).
Every line exited 1 with a rejection,
and each wrote its audit line:

- `["reset","--hard"]`,
  `["reset","--har"]`,
  `["reset","--hard","--hard"]`:
  `ResetHard`/`rejected`.
- `["checkout","--","tracked.txt"]`,
  `["checkout","-f","tracked.txt"]`,
  `["checkout","--force","."]`,
  `["checkout","--ours","--","tracked.txt"]`:
  `CheckoutPaths`/`rejected`.
- `["restore","tracked.txt"]`:
  `RestoreWorktree`/`rejected`.
  `["restore","--staged","tracked.txt"]`:
  `RestoreStaged`/`rejected`.
- `["switch","--discard-changes","main"]`,
  `["switch","-f","main"]`:
  `SwitchForce`/`rejected`.
- `["clean","-fdx"]`:
  `Clean`/`rejected`.
- `["read-tree","--reset","-u","HEAD"]`:
  `ReadTree`/`rejected`.
- `["rm","-f","tracked.txt"]`:
  `Remove`/`rejected`.
- `["mv","-f","tracked.txt","other.txt"]`:
  `Move`/`rejected`.
- `["checkout-index","-f","-a"]`:
  `CheckoutIndex`/`rejected`.
- `["rebase","--abort"]`,
  `["merge","--abort"]`,
  `["am","--abort"]`,
  `["cherry-pick","--abort"]`:
  `AbortRebase`,
  `AbortMerge`,
  `AbortAm`,
  `AbortPick`,
  each `rejected`.
- `["stash","drop"]`:
  `Stash`/`rejected`.

Repository selection was tried from every direction the brief names,
with a modified tracked file and the same command `["reset","--hard"]`
(`spellings-bypass.ts`):

- `["-C", "<the repository>", "reset", "--hard"]` from the home directory:
  rejected.
- `["--git-dir", "<the repository>/.git", "--work-tree", "<the repository>", ...]`:
  rejected.
- `GIT_DIR` and `GIT_WORK_TREE` set with the command run from the home directory:
  rejected.
- a repository whose own `config` sets `core.worktree`:
  rejected,
  and audited.
  The in-process layout reader declines any repository with `core.worktree` set
  (`src/native/repo_read/repo_read_layout.rs:16`),
  so Git answers the identity question itself.

Repository selection is sound.
What it does not do is protect the forms nothing models,
which is the subject of the findings.

## Findings ranked by what they risk losing

### 1. `git worktree remove` deletes a whole worktree's uncommitted and ignored work, unguarded

Ranked first because it destroys everything in the worktree at once,
in any worktree,
and it does so without a rejection,
an audit line or a copy.

Where:
`guarded_decision` (`src/native/command/rule_linked_worktree.rs:222`)
has no arm for the word `worktree`,
and `command_worktree.rs:44` reads only `add` and `move`.
The only work the wrapper does before a removal is to copy the linked worktrees' unshown
push-failure records
(`src/native/invocation/wrapped_command_notices_snapshots_and_manual_push.rs:190`),
which keeps notices and not files.

Sequence,
reproduced (`evidence.ts`,
`F1`):
a linked worktree of a repository holds one modified tracked file
and an ignored directory with a file in it;
from the main worktree the person runs
`git worktree remove --force <path>`.

Wrong result:
the command exits 0 with empty standard error,
no audit line is written (there is no audit file at all),
and the whole directory is gone,
including the tracked edit that was never committed.

Measured output:

```text
F1  git worktree remove --force, a linked worktree holding uncommitted and ignored work
    before: tracked.txt="uncommitted tracked work\n" .env="SECRET=2\n" ignored/data.bin="uncommitted ignored work\n"
    wrapper exit 0, stderr "", audit "no audit file"
    after: the linked worktree directory exists = false
```

The second sequence is worse,
because it needs no option at all:
a linked worktree whose only uncommitted state is an ignored file,
and `git worktree remove <path>` with no `--force`.
Git removes it,
because an ignored file is not an untracked file to Git's own refusal.
Measured:

```text
    and without --force, an ignored file only: .env="SECRET=3\n"
    wrapper exit 0, stderr "", audit "no audit file"
    after: the linked worktree directory exists = false
```

Real Git's own two answers,
for comparison
(`ground-truth.ts`):
`git worktree remove` on a worktree with a modified tracked file exits 128 with
"contains modified or untracked files,
use --force to delete it",
and the same command with `--force` exits 0.
So the habit the wrapper should catch is exactly the one `--force` names.

Suggestion,
not a decision:
give `worktree` a guarded form in the same reading the other commands use.
A `remove` or `prune` whose subcommand words and `--force` count are read from Git's own
`builtin/worktree.c` table,
judged by the *removed worktree's* status rather than the running worktree's,
with the hatch and the audit line the other forms have,
and with the conflict-free copy of the tracked changes the owner's "never discard user data" asks for.
Whether that copy is a patch,
a `discarded/` save of the worktree's tracked and ignored files,
or a refusal with a named remedy is the owner's call;
the audit is not the place to take it.

### 2. The dirtiness measurement cannot see ignored files, so the most common `HEAD` move replaces them

Ranked second because the command is `git checkout <branch>` or `git switch <branch>`,
the most common command in the family,
it works in the main worktree,
and what it can replace is precisely an environment file.

Where:
`dirty_scope` gives `Tracked` to every command but `clean`
(`src/native/command/rule_linked_worktree.rs:180`),
`dirty_query_arguments` asks Git with `--untracked-files=no`
(`src/native/command/rule_worktree_dirty.rs:56`),
and `SPEC.md:8120` states the rule as
"A `HEAD`-moving form is rejected only while tracked changes exist."
Git's own `git checkout -h` lists `--[no-]overwrite-ignore` as
"update ignored files (default)",
so a `HEAD` move replaces an ignored file whenever the branch it moves to tracks that path.

Sequence,
reproduced (`evidence.ts`,
`F2a`):
on `main`,
`.gitignore` names `.env`,
and `main` has no `.env` in its tree;
on `other`,
`.env` is tracked;
the person keeps a real `.env` on `main` with a secret in it;
the worktree is otherwise clean,
so `git status --porcelain=v1 --untracked-files=no` prints nothing;
the person runs `git checkout other` in the main worktree.

Wrong result:
the command is allowed with `dirty` false,
Git switches branches,
and the ignored file's content is replaced by the branch's committed copy.
Nothing is saved first,
and nothing says the file was overwritten.

Measured output:

```text
F2a git checkout <branch> replaces an ignored file the branch tracks
    before: status --untracked-files=no "" .env="SECRET=the person value\n"
    wrapper exit 0, stderr "Switched to branch 'other'"
    after: .env="the tracked version\n"
    audit: worktree="main" form="CheckoutMove" outcome="allowed" dirty=false
```

The same root cause has a second instance that needs no tracked/ignored asymmetry at all,
because `clean -x` and `clean -X` delete ignored files by definition,
while `clean`'s own measurement asks for `--untracked-files=normal`,
which lists untracked files and never ignored ones
(`src/native/command/rule_linked_worktree.rs:182`,
`src/native/command/rule_worktree_dirty.rs:64`).
`clean_changes_worktree` decides from the dry-run state alone
(`src/native/command/command_clean.rs:150`),
so nothing in the route reads which of `-d`,
`-x` and `-X` was given.

Sequence,
reproduced (`evidence.ts`,
`F2b`):
a linked worktree holds `.env` and `ignored/` and no tracked change;
the person runs `git clean -fdx` there.
`git clean -fdX` and `git clean --force --force -d -x` behave identically
(`spellings-bypass.ts`).

Wrong result:
exit 0,
Git prints "Removing .env" and "Removing ignored/",
the audit line says `Clean`/`allowed` with `dirty` false,
and both files are gone with no copy.

Measured output:

```text
F2b git clean -fdx in a clean linked worktree deletes ignored files
    before: status --untracked-files=no "" .env="SECRET=1\n" ignored/data.bin="ignored work\n"
    wrapper exit 0, stderr "", stdout "Removing .env\nRemoving ignored/"
    after: .env=ABSENT ignored/data.bin=ABSENT
    audit: worktree="linked" form="Clean" outcome="allowed" dirty=false
```

The wrapper calls the same files precious everywhere else:
the copy route exists to carry a worktree's ignored state into new worktrees,
and the carve-out saves a conflicted file before it is replaced.
Here the guard's own measurement cannot see them.

Suggestion,
not a decision:
measure ignored files too.
For `clean`,
add `--ignored=matching` to the status query when the form deletes ignored files,
or refuse `-x` and `-X` in a linked worktree with a remedy that names
`git clean -fd` (which keeps them).
For a `HEAD` move,
the cheap fix is to pass Git `--no-overwrite-ignore`,
which makes Git itself
refuse rather than replace;
the alternative,
refusing the move while any ignored file would be replaced,
needs the
target tree read first and is slower.
`SPEC.md:8120` would need the same sentence amended.

### 3. An alias reaches no guard and leaves no trace

Ranked third because it is a total bypass of every form in the route,
and because aliases are a normal way to type Git.

Where:
`decide_linked_worktree` reads the word with `command_tokens`
(`src/native/command/rule_linked_worktree.rs:321`),
which returns the token on the command line.
The crate already knows how to do better:
`resolve_forwarded_command`
(`src/native/invocation/forwarded_command.rs:370`)
expands `alias.<name>` and `alias.<name>.command` the way Git does,
follows up to `MAX_ALIAS_EXPANSIONS` (`:39`),
and hands the expanded command to the copy route
(`src/native/invocation/entry_forward_action_planning.rs:77`).
No policy asks it.

Sequence,
reproduced (`evidence.ts`,
`F3`):
in the main worktree with one modified tracked file,
the person runs `git -c alias.wipe=reset --hard wipe`.

Wrong result:
the word is `wipe`,
`guarded_decision` matches no arm,
the command is forwarded,
Git expands the alias and resets the branch hard,
the edit is gone,
standard error is empty,
and no audit file is written.
A configured alias does the same:
`git config alias.wipe 'clean -fdx'` followed by `git wipe` empties `.env` and `ignored/`
in the main worktree,
which the plain `clean` can never do there because the main worktree is rejected outright.

Measured output:

```text
F3  an alias reaches no guard at all
    before: tracked.txt="the person wrote this\n"
    -c alias.wipe=reset --hard wipe: exit 0, stderr "", audit "no audit file"
    after: tracked.txt="one\n"
    before: .env="SECRET=1\n" ignored/data.bin="ignored work\n"
    wipe (alias of clean -fdx), main worktree: exit 0, stderr "", audit "no audit file"
    after: .env=ABSENT ignored/data.bin=ABSENT
```

A shell alias (`alias.wipe '!git reset --hard'`) is forwarded the same way.
Its own inner `git` is whatever `PATH` offers,
so whether that inner command is guarded depends on the person's `PATH`,
not on this route.

Suggestion,
not a decision:
run `resolve_forwarded_command` once and decide on its `region()` and `word()`,
the way the copy route already does.
A shell alias and an unresolved name have no subcommand to judge,
so the honest answer for them is a rejection naming the alias,
which is a change of user-visible behavior and belongs to the owner.

### 4. `stash drop` and `stash clear` in a clean linked worktree discard the repository's stash

Ranked fourth because the stash lives in the common directory,
so one clean linked worktree can delete work parked anywhere in the repository,
and because the form is a single word away from `stash list`.

Where:
`stash` is `NeedsEffectiveTarget(Stash)` for every state-changing subcommand
(`src/native/command/rule_linked_worktree.rs:235`),
which means the decision is taken by the *running worktree's* status
(`src/native/policy/policy_checks.rs:403`).
`stash drop` and `stash clear` delete entries of `refs/stash`,
which is shared by every worktree of the repository,
and `stash_changes_state` correctly says they change state
(`src/native/command/command_stash.rs:328`).
The measurement asks the wrong object.

Sequence,
reproduced (`evidence.ts`,
`F4`):
the person parks uncommitted work with `git stash push` in the main worktree;
a linked worktree of the same repository exists and is clean;
the person runs `git stash drop` (or `git stash clear`) in the linked worktree.

Wrong result:
the linked worktree is clean,
so the form is allowed with `dirty` false,
and the stash entry and its reflog are both gone.
The work is then only in the object database as an unreachable commit,
which Git's own expiry removes.

Measured output:

```text
F4  git stash drop and stash clear from a clean linked worktree discard the repository stash
    before: stash list "stash@{0}: On main: the person work", linked status --untracked-files=no ""
    stash drop in the linked worktree: exit 0, stderr "", audit worktree="linked" form="Stash" outcome="allowed" dirty=false
    after: stash list "", stash reflog ""
    before: stash list "stash@{0}: On main: the person work", linked status --untracked-files=no ""
    stash clear in the linked worktree: exit 0, stderr "", audit worktree="linked" form="Stash" outcome="allowed" dirty=false
    after: stash list "", stash reflog ""
```

`stash drop` in the main worktree stays rejected
(`stash-and-branch.ts`),
which is why the linked worktree is the way in.

Suggestion,
not a decision:
let `StashDrop` and `StashClear` be a form that is rejected in every worktree,
like an overwrite in the main worktree,
since they delete an object that is not the running worktree's.
A softer option treats a repository with any stash entry as dirty for those two words.

### 5. `branch -D` from the main worktree deletes a branch, unguarded and unaudited

Ranked fifth because what it destroys is a ref rather than worktree files,
and the commits stay in the object store until Git's expiry.

Where:
`rule_branch_worktree.rs:137` reads only the words that *create* a branch,
and `SPEC.md:8098` to :8116 lists no branch deletion.
`guarded_decision` has no `branch` arm.

Sequence,
reproduced (`evidence.ts`,
`F5`):
a branch holds one commit that no other ref reaches;
the person runs `git branch -D gone` from the main worktree.

Wrong result:
exit 0,
"Deleted branch gone",
the commit object is still readable by its own name,
and no audit line is written.

Measured output:

```text
F5  git branch -D from the main worktree
    exit 0, stdout "Deleted branch gone (was 46341f5).", audit "no audit file"
    the commit object is still in the object store: true
```

The commit is recoverable today,
and stops being recoverable when `git gc` expires it,
which `git gc --prune=now`,
`git update-ref -d` and a forced reflog expiry reach just as unguarded.
Git refuses `git branch -D` for a branch another worktree has checked out
(`stash-and-branch.ts`),
so a worktree's checked-out work is not at risk here.

Suggestion,
not a decision:
`branch` with `-d`,
`-D`,
`--delete` or `--force` is a guarded form whose measurement is
"is the branch's commit reachable from any other ref",
reported rather than refused,
so the person learns what the deletion costs.
Whether a refusal is wanted is the owner's call.

### 6. The tool-cache exemption is chosen by the invocation's own environment

Ranked sixth because it needs one environment variable to be set for the command,
and it leaves an `exempt` audit line.

Where:
`default_allowed_worktree_dirs`
(`src/native/invocation/effective_target.rs:169`)
builds the exemption list from `UV_CACHE_DIR` or `XDG_CACHE_HOME/uv`
(`:76`) and `MISE_DATA_DIR` or `XDG_DATA_HOME/mise`
(`:125`),
read from the invocation's environment;
`is_allowed_worktree_dir` (`:200`) then exempts any repository whose Git directory lies inside one,
and `classify_effective_target` (`:231`) returns `Allowlisted`.

Sequence,
reproduced (`evidence.ts`,
`F6`):
a script that wants an isolated `mise` sets `MISE_DATA_DIR=$PWD` for the commands it runs,
which is a plausible reason to name that variable;
the person runs `git reset --hard` in the main worktree with a modified tracked file.

Wrong result:
the repository is `exempt`,
the command runs,
the edit is gone,
and the only trace is an audit line with `outcome` `exempt` that nobody reads before the loss.

Measured output:

```text
F6  MISE_DATA_DIR naming the repository exempts it
    before: tracked.txt="the person wrote this\n"
    wrapper exit 0, stderr ""
    after: tracked.txt="one\n"
    audit: worktree="main" form="ResetHard" outcome="exempt" dirty=null
```

Suggestion,
not a decision:
say the exemption in standard error when it is used,
not only in the audit file,
so a person sees "this repository is exempt from the worktree guard because
`MISE_DATA_DIR` is `<path>`" before the command runs.
The compiled-in list with environment values is otherwise the owner's choice.

### 7. The measurement is a separate process from the command it guards

Ranked last because it is a window rather than a spelling,
and it is read from the code only.

Where:
`src/native/policy/policy_checks.rs:403` asks `git status` in one process,
the policy pass returns,
and `src/native/invocation/entry.rs:367` replaces the wrapper's process with real Git,
which does the discarding.

Sequence:
a linked worktree is clean;
the wrapper's status query answers "clean";
anything else on the machine writes a tracked file,
which is exactly what a build,
a formatter
or an agent's edit does;
Git then runs `git reset --hard` or `git checkout -f` there.

Wrong result:
the write is discarded with an `allowed` audit line that says `dirty` false,
though it was true a moment later.

This is not reproduced:
the wrapper offers no seam between the two processes,
and a timing-based reproduction would prove only that the window exists,
not its size.
`SPEC.md` already makes the equivalent statement about the carve-out's own window
("A write that lands after the last comparison and before Git starts cannot be seen from outside Git"),
so the honest treatment is to say the same thing about the measurement.

Suggestion,
not a decision:
name the window in `SPEC.md` beside the dirtiness rule,
so a person reading the guard's promise knows what it does not promise.

## Waits and their ceilings

Every wait the route can take,
and what bounds it:

- the dirtiness query:
  `git status` as a child,
  waited for without a deadline
  (`src/native/repo_read/git_metadata.rs:94`).
  Its only bound is the child's own exit.
  A start the kernel refuses for lack of processes is retried on a budget of five seconds of
  pauses,
  ten milliseconds doubling to five hundred
  (`src/native/push/start_retry.rs:33`).
  A repository on a slow or network filesystem therefore makes the guard wait as long as Git does.
- the identity query:
  the same runner,
  the same bound
  (`src/native/worktree/worktree_identity.rs:247`).
- the saved copy:
  no wait.
  Each file is written,
  each directory above it is synced,
  and the prune that follows takes no lock and waits for nobody
  (`src/native/conflict/conflict_sides.rs:113`,
  `src/native/conflict/conflict_side_guard.rs:152`).
- the carve-out's index read:
  one in-process read of at most one gibibyte
  (`src/native/conflict/conflict_side_guard.rs:52`),
  so no Git process and no wait.
- the audit append:
  one `write` and no lock
  (`src/native/conflict/discard_audit.rs:362`).
  A full file is moved aside first,
  and a rotation that fails keeps the writer on the file it has,
  so the append never waits and never fails the command.
- nothing in `src/native/command/` or `src/native/conflict/` sleeps,
  polls or loops on a timeout:
  `rg 'sleep|Condvar|Mutex|RwLock|flock|lockf|F_SETLK|O_EXCL'` over both folders,
  test sidecars excluded,
  finds nothing,
  and `rg fcntl` finds only two `F_DUPFD_CLOEXEC` calls in `discarded_store.rs`
  (`:136`,
  `:318`),
  which duplicate a descriptor and take no lock.

So the route holds no lock,
and its one unbounded wait is the Git child whose status it reads.

## What a killed process leaves behind

Ordered by the sequence in `src/native/policy/policy_checks.rs:337` and
`src/native/conflict/conflict_side_guard.rs:225`:

1.  Before the decision:
    nothing is written.
    A kill here leaves the command unrun and the worktree untouched.
2.  During a carve-out copy:
    `discarded/<stamp>/` may hold some of the paths and not others,
    and the file being copied may be short.
    Git has not started,
    so nothing was replaced.
    The leftover directory is never referenced by anything and is removed by the age limit,
    which is a cost of disk and not of data.
3.  After the copies are synced,
    before the notice:
    the copies are durable and Git has not started.
    A kill here keeps the copies and leaves the command unrun.
4.  After the audit line,
    before Git:
    the record exists,
    the command did not run.
    A kill can leave a torn line only between the `write` and its page-cache flush,
    and the next append starts a fresh line
    (`src/native/conflict/discard_audit.rs:344`).
5.  While Git runs:
    Git owns whatever state it is in,
    and the copies and the audit line are already on disk.
    This is the one kill point the design handles better than the alternatives,
    because the copy is made before the process is replaced and not after.
6.  During the prune that follows a save:
    some older saves are gone,
    the save just written is out of reach by construction,
    and another pass finishes the work by name.
7.  After the command:
    nothing of the route's remains except the audit line and whatever save it names.

No kill point in the route can hide work or apply it in the wrong place:
nothing is moved,
renamed or written outside the two directories named above,
and the copies are write-once files in a directory whose name no other invocation takes
(`fresh_directory` at `src/native/conflict/conflict_sides.rs:84`).
The one state that is lost is the audit line when the machine loses power
after the `write` and before the flush,
which is the same guarantee the existing page already gives
("A failed write never changes the command's result").

## What I checked and found sound

- Every classic spelling the brief names is rejected in the main worktree,
  with the audit line the code claims:
  `reset --hard` and `--har`,
  `checkout -- <path>`,
  `checkout -f`,
  `checkout --force .`,
  `checkout --ours -- <path>`,
  `restore <path>` and `restore --staged <path>`,
  `switch --discard-changes` and `switch -f`,
  `clean -fdx`,
  `rm -f`,
  `mv -f`,
  `checkout-index -f -a`,
  `read-tree --reset -u`,
  the four abort forms,
  and `stash drop`.
- Git's own option list for `git checkout -h` matches `CHECKOUT_TABLE` row for row,
  including the arity of `--track` and `--recurse-submodules`
  (`src/native/command/command_checkout_table.rs:88`).
  `git checkout -calias.w=status w` and `git -- status` are both refused by Git 2.56.0
  (`git-options.ts`),
  so neither attached-value form can hide a subcommand from the wrapper's global reader.
- The conflict carve-out does what the owner decided.
  Reproduced (`carve-out.ts`):
  `git checkout --theirs -- a.rs` during a merge conflict saved the hand-edited file,
  printed the notice before Git ran,
  wrote `saved-conflict` with the copy's place,
  and then let Git replace it:

  ```text
  stderr: "monochromatic-git-policy-cli: saved your copy of a.rs to
    /var/mnt/encrypted/.../carve-out-allowed/.git/monochromatic-git-policy-cli/discarded/2026-11-09T05-56-33/a.rs
    before this command replaces it (a.rs was conflicted)."
  saved copies: {"2026-11-09T05-56-33/a.rs":"\"line one\\n<<<<<<< HEAD\\nMY line two\\n=======\\nTHEIR line two\\n>>>>>>> theirs\\nline three\\nhand-resolved note\\n\""}
  ```

  A clean path named in the same line keeps the rejection and saves nothing,
  `git restore --theirs a.rs` without `--` keeps the rejection,
  and an index that cannot be read in process keeps the rejection.
- `switch --orphan` does not lose anything:
  real Git refuses it while a tracked file is modified
  (`ground-truth.ts`),
  and the tracked files it removes after that are in `HEAD`.
- A repository selected through `-C`,
  `--git-dir` with `--work-tree`,
  `GIT_DIR` with
  `GIT_WORK_TREE`,
  or a `config` that sets `core.worktree` is judged by the same rules every time,
  and every one of those four runs was rejected.
- The escape hatch is recorded in both positions it is accepted in,
  directly after the subcommand and before it,
  with `outcome` `hatch` (`carve-out.ts`).
  `SPEC.md:8098` to :8127 says "directly after the subcommand",
  and the global reader removes a control written before it
  (`src/native/invocation/wrapper_controls.rs:375`),
  so the sentence describes a narrower rule than the code applies.
  This is a documentation difference,
  not a loss.
- A torn or unlocked audit line never swallows the next record,
  and a generation that names a saved copy on disk is never removed
  (`src/native/conflict/discard_audit_generations.rs:277`).
- `restore --staged` alone leaves the worktree file alone (real Git,
  `ground-truth.ts`),
  which is why the route reports it as `RestoreStaged` rather than as a worktree restore.

## What I could not determine

- The size of the window in finding 7.
  The wrapper offers no seam between its status query and the process it becomes,
  and a timing measurement would answer a different question.
- Whether the owner wants an alias judged at all.
  A shell alias has no subcommand to read,
  and refusing one changes what a person can type.
- Whether the copy the owner wants before a `worktree remove`
  should be a patch of tracked changes,
  a save of the whole worktree,
  or a refusal with a remedy.
  The three options differ in what they cost on a large worktree,
  and this audit was told not to change production code.
- Whether a `branch -D` refusal is wanted,
  given that the commit stays recoverable and Git's own expiry is what removes it.
- Whether any spelling of a command with a ported table is accepted by Git 2.56.0 and refused
  by that table.
  The tables match `git <command> -h` for `checkout`,
  the destructive spellings of every command were rejected by the real binary,
  and the differential controls in `command_options_git_tests.rs` are the standing proof for the rest;
  I did not enumerate Git's own `parse-options` tables option by option.
  The mechanism if one exists is finding 3's neighbour:
  a refused region is `Pass` with no audit line,
  which `refuse.ts` shows.

## Reproductions

Scripts are in `${HOME}/temp/agent/pi-discarding-route`,
none of them is committed,
and each writes into disposable repositories under its own `fixtures*` or `cases*` folder.

- `ground-truth.ts`:
  what real Git 2.56.0 does,
  with no wrapper on `PATH`.
- `git-options.ts`:
  Git's own option names and the two attached-value global forms.
- `spellings-main.ts` and `spellings-bypass.ts`:
  the first pass over the classic spellings and over repository selection.
- `carve-out.ts`:
  the saved copy,
  the refusal of a clean path in the same line,
  the hatch positions,
  and the exemption.
- `stash-and-branch.ts`:
  the stash in a clean linked worktree and `branch -D`.
- `ignored-overwrite.ts`:
  the first measurement of finding 2's first instance.
- `refuse.ts`:
  the refused region that produces no audit line.
- `evidence.ts`:
  the final run whose output is quoted for findings 1 to 6 and for the spelling list;
  its output is in `evidence-output.txt` beside it.

Each script runs under `timeout` inside its own `systemd-run` scope with a CPU quota,
a memory cap and a task cap,
and none of them touches this repository,
the installed wrapper or its service.
