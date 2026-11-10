# cli-git printed remedies

## Status

Audited 2026-11-10T06:31:54-05:00 on branch `fix/git-policy-cli-printed-remedies-run-as-printed`.
Every command the native wrapper prints as a way forward was run exactly as printed,
through the built executable,
in a disposable repository put into the state that prints it.
Four printed commands were refused by the wrapper's own policies,
and the text of those messages is corrected.
Three of them have a control that runs the corrected text;
the fourth,
Git's own wording inside the wrapper's failed-pull report,
has the reproduction recipe below and no control yet.

The message texts live in
`package/git-policy/cli/src/native/command/rule_linked_worktree_messages.rs`
(the discarding-command rejections),
`package/git-policy/cli/src/native/policy/event_hints.rs`
(the `hint` field of every policy finding),
`package/git-policy/cli/src/native/worktree/worktree_finish_commands.rs`
(the finish-or-remove commands of an interrupted add,
which one owner now holds for both messages that print them),
and the notice modules under
`package/git-policy/cli/src/native/worktree/`,
`package/git-policy/cli/src/native/worktree_copying/`,
`package/git-policy/cli/src/native/landing/`,
`package/git-policy/cli/src/native/push/`
and
`package/git-policy/cli/src/native/merge/`.

## Why this exists

The wrapper refuses commands that can discard uncommitted work,
and its messages name other commands as the way to reach the same goal.
A message is part of the interface:
a person who runs what it prints must not be refused by the same program that printed it.
One was:
the interrupted add's report printed `git -C <path> reset --hard`,
and `linked-worktree-only` refused it with
`this worktree has uncommitted changes to tracked files`,
because a worktree whose checkout never ran reads exactly like one holding uncommitted changes
(`doc/handover/cli-git-rust-implementation.md`,
section "fix/git-policy-cli-worktree-add-keeps-ignored-files-and-names-interrupted-adds",
recorded the refusal and left the choice open).
This audit ran every printed command instead of trusting that one report.

## Method

1. Every string literal in
   `package/git-policy/cli/src/native`
   outside `*_tests.rs` and `binary/`
   was extracted with a reader that joins Rust's escaped line breaks,
   and the literals naming a `git` command or telling the reader to run one were listed.
   The reader and its output are not committed:
   the list below is the durable record,
   and the controls in
   `package/git-policy/cli/src/native/binary/binary_printed_remedies_tests.rs`
   keep the corrected texts checked.
2. Each message was put in front of a disposable repository in the state that prints it,
   under a disposable `HOME`,
   `XDG_RUNTIME_DIR`,
   `GIT_CONFIG_NOSYSTEM`,
   and a real Git at `/usr/bin/git` (2.56.0),
   with the built executable linked as `git` first on `PATH`.
3. The command was read out of the printed text and run with the words a shell reads back,
   not retyped,
   so a message that changes keeps being checked.
4. A run counted as working when the wrapper printed no `policyId` finding for it and
   Git itself did not refuse the command.
   A refusal by Git's own rules (a plain `git rm` of a modified file) is not a defect of the message
   when the message says so.

## What was refused, and what was corrected

### The interrupted add's finish command

The report is
`monochromatic-git-policy-cli: an interrupted git worktree add left <path> registered and possibly incomplete; ...`
from
`package/git-policy/cli/src/native/worktree_copying/worktree_copy_interrupted_add_settlement.rs`
and, for a ladder that could not complete a checkout,
`package/git-policy/cli/src/native/worktree/worktree_ladder_finish.rs`.

Printed before the correction:

```text
`git -C <path> reset --hard` completes its checkout, and `git worktree remove --force <path>` removes it.
```

Run in a registered worktree whose checkout never ran,
`git -C <path> reset --hard` was refused by `linked-worktree-only`
(`code: linked-worktree-only/linked-worktree-required`,
`this worktree has uncommitted changes to tracked files`, exit `1`).
`git worktree remove --force <path>` runs (it is audited, never refused).

Corrected:
both messages now print one sentence,
built by
`finish_or_remove_sentence` in
`package/git-policy/cli/src/native/worktree/worktree_finish_commands.rs`:

```text
`git -C <path> reset --no-enforce-worktree --hard` completes its checkout, discarding whatever that
worktree holds (the wrapper's worktree guard refuses a hard reset in a worktree that holds uncommitted
changes, which a checkout that never ran reads as, unless --no-enforce-worktree is given), and
`git worktree remove --force <path>` removes it.
```

The policy is not what was wrong.
The wrapper cannot tell a half-written checkout from a person's edits in the same directory,
and "never discard user data" makes the guard the right answer;
what was wrong is a message that asked for a command the guard refuses without saying so.
The hatch form runs,
and the sentence says what the reset discards.

### The stash drop report's branch command

Printed before the correction,
from
`package/git-policy/cli/src/native/command/rule_linked_worktree_messages.rs`
(the `git stash drop` and `git stash clear` text):

```text
To keep the work, move it into its own branch with git stash branch <branch> (allowed), which applies
the entry and keeps it in the branch.
```

Run in the main worktree,
where the report is printed,
`git stash branch <branch>` was refused by `linked-worktree-only`,
because `git stash branch` is a stash form that changes state.
Run in a dirty linked worktree,
it was refused there too.
It runs only in a clean linked worktree,
which is what the corrected text now says:

```text
To keep the work, move it into its own branch with git stash branch <branch> in a clean linked
worktree, which applies the entry and keeps it in the branch; a stash form that changes state is
refused in the main worktree, and in a linked worktree that holds uncommitted changes.
```

### The `git clean -x` report's untracked-files command

Printed before the correction:

```text
To delete only untracked files and keep the ignored ones, use git clean -fd (allowed while no
untracked file is precious).
```

Run in the main worktree,
`git clean -fd` was refused by `linked-worktree-only`,
which refuses every deleting `git clean` in the main worktree.
Run in a linked worktree holding one untracked file,
it was refused as well,
so the command was refused in every state where it would delete anything,
and the parenthetical described a rule the wrapper does not have.
The corrected text names the hatch form,
which runs and keeps the ignored files:

```text
To delete only untracked files and keep the ignored ones, pass --no-enforce-worktree directly after
clean (git clean --no-enforce-worktree -fd), because a clean that deletes files is refused in the
main worktree, and in a linked worktree while it holds uncommitted changes or untracked files.
```

### The failed pull's recovery command

`package/git-policy/cli/src/native/merge/pull_isolation.rs`
prints,
when its own fast-forward step fails:

```text
fatal: Cannot fast-forward your working tree.
After making sure that you saved anything precious from
$ git diff <orig_head>
output, run
$ git reset --hard
to recover.
```

That text is Git's own,
copied from `builtin/pull.c` in 2.56.0,
and `git reset --hard` is refused by `linked-worktree-only` in the dirty worktree that made the
fast-forward fail.

Corrected:
Git's words are kept,
and one line is appended,
so the person who pastes the recovery command is not refused:

```text
monochromatic-git-policy-cli: that reset discards whatever this worktree holds, and the wrapper's
worktree guard refuses a hard reset here without --no-enforce-worktree, which is what made this
fast-forward fail; run git reset --no-enforce-worktree --hard if you mean it.
```

This is the one correction without a control.
The state needs a pull whose fetch moves the checked-out branch and whose `git read-tree -u -m` then
fails against local changes,
which no fixture here builds.
To add one:
give a repository a remote whose fetch refspec writes `refs/heads/<branch>`,
commit in the remote,
change the same file in the local checkout,
and run `git pull`;
the report is on standard error and the printed hatch command is the one to run.

## What runs as printed

Every command below was run through the built executable in the state that prints it,
and none was refused by a policy.
The policy and message are named first,
then the commands.

### `linked-worktree-only`, in the main worktree

- `git restore`, `git checkout -- <path>`: `git diff -- <path>`,
  `git show HEAD:<path>`,
  `git worktree add <path> HEAD`,
  `git worktree add <path> <branch>`.
- `git reset --hard`: `git reset --soft <commit>`,
  `git diff <commit>`,
  `git show <commit>:<path>`,
  `git worktree add <path> <commit>`.
- `git reset`: `git diff --cached --name-only`, `git reset --soft <commit>`.
- `git restore --staged`: `git diff --cached --name-only`, `git worktree add <path> HEAD`.
- `git clean`: `git clean -n`, `git worktree add <path> HEAD`.
- `git clean -x`: `git clean -nx`, `git worktree add <path> HEAD`,
  and the corrected `git clean --no-enforce-worktree -fd`.
- `git clean -X`: `git clean -nX`, `git worktree add <path> HEAD`.
- `git checkout <branch>`, `git switch --force`, `git switch <branch>`:
  `git worktree add <path> <branch>`.
- `git read-tree --reset`: `git read-tree -m`, `git worktree add <path> HEAD`.
- `git checkout-index --force`: `git checkout-index --prefix=<directory>/ --all`.
- `git rm --force`: `git rm --cached <path>`.
  A plain `git rm <path>` of a modified file is refused by Git itself,
  which is what the message says it does.
- `git mv --force`: `git mv` without `--force` runs;
  `git rm <destination>` before it is the message's own precondition.
- `git stash drop`: `git stash list`, `git stash show`,
  and the corrected `git stash branch <branch>` in a clean linked worktree.
- the hatch: `git restore --no-enforce-worktree -- <path>`.
- `git merge --abort`: `git merge --quit`.
- `git rebase --abort`: `git rebase --quit`, `git rebase --skip`.
- `git cherry-pick --abort` and `git revert --abort`: `--quit` with the same command.
- `git am --abort`: `git am --quit`.
- `git worktree add <path> HEAD` from a main worktree holding an ignored file:
  the ignored-state copy runs and the add succeeds.

### `branch-worktree-only`

- `git switch -c <branch>` is refused,
  and `git worktree add -b <branch> <path> [<start-point>]` runs.

### `require-root`

- A command run in a subdirectory is refused,
  and `-C <root>` before the subcommand runs
  (`git -C <root> status`, `git -C <root> add -- <path>`).

### Notices with commands

- `git worktree list` in the worktree-copy notices, the interrupted-add settlement and the ladder
  report: a read-only command, forwarded unchanged.
- `git worktree remove --force <path>` in the interrupted-add settlement and the ladder report:
  audited, never refused.
- `git worktree remove <path>` in the unfinished-copy notice: audited, never refused.
- `git worktree prune` in the interrupted-removal reports: forwarded unchanged.
- `git worktree unlock` in the conversion refusal: forwarded unchanged.
- `git status` in the slow-route notice: forwarded unchanged.
- `rm <path>` for a stale `index.lock`: not a Git command, and the settlement only names it while no
  Git runs there.

## What was read and not exercised

These name commands whose state needs an injection the audit did not build
(a killed service, a killed ladder step, a full disk, an unreachable remote, a btrfs-only path,
a standards-review gateway, an LFS server, a session bus, a PID namespace).
They are listed so the next reader knows what is unchecked,
not because a defect is suspected:

- `package/git-policy/cli/src/native/landing/`:
  `git log -1`, `git status`, `git show <oid>`, `git log`, `git commit -F <saved>`,
  `git push`.
  The states are a branch that moved under a commit, a service from another release, and a message
  saved to a file.
- `package/git-policy/cli/src/native/push/`:
  `git push`, `git ls-remote <remote>`, `git merge --ff-only <remote>/<branch>`.
  The states are a failed backup push, a mirror remote and an unset `push.default`.
- `package/git-policy/cli/src/native/merge/`:
  `git merge --no-edit`, `git diff --cached -- <file>`, `git log <target>`,
  `git stash list`, `git cat-file blob <oid>`, `git rebase --continue`,
  `git cherry-pick --continue`, `git revert --continue`, `git commit`.
  The states are an interrupted merge record and a merge that had to be saved on a side ref.
- `package/git-policy/cli/src/native/worktree_copying/`:
  the removal, recovery and stage-claim notices,
  whose states need a killed copy at a chosen step.
- `package/git-policy/cli/src/native/subvolume/` and the `migrate-to-subvolume` and
  `convert-worktree-to-subvolume` notices:
  they need btrfs, and the gate container runs on overlayfs.
- `package/git-policy/cli/src/native/health/`:
  `git config --global core.fsmonitor false` and
  `git config --global core.lockfilePid true` write the person's global configuration,
  so no control runs them; both are forwarded unchanged by the wrapper.
- `package/git-policy/cli/src/native/candidate/` and
  `package/git-policy/cli/src/native/invocation/`:
  `git cat-file --batch-check`, `git fsck`, `git ls-files --stage`,
  `git ls-tree -r HEAD`, `git diff --cached --raw`, `git diff-tree`,
  `ls -l <path>`.
  These are read-only commands named inside engine failures,
  and the wrapper's policy set guards no read.
- `package/git-policy/cli/src/native/policy/event_hints.rs`:
  the hints naming `git status --short`, `git add -- <path>`,
  `git monochromatic-git-policy-cli fix -- <path>`,
  `git rm --cached CONTEXT.md`, `git status`, `git log --oneline -3`,
  `git fetch`, `git push`, `git ls-remote <remote>`.
  Every one is either a read,
  a `git add` with explicit paths,
  a management subcommand the wrapper owns,
  or the `git rm --cached` that
  `binary_hint_tests::a_root_context_file_carries_an_intent_hint` already runs.
- `git monochromatic-git-policy-cli <command>` everywhere:
  these are the wrapper's own subcommands,
  and the `--help` of each is generated from the parser that enforces it.

## Reproducing

The controls that cover the three corrections are in
`package/git-policy/cli/src/native/binary/binary_printed_remedies_tests.rs`:

```text
mise run //package/git-policy/cli:native:test:binary:host -- printed_remedies
```

Each control builds the state,
reads the command out of the message the wrapper printed,
and runs it,
so the check follows the text instead of restating it.
For the rest of the list,
put the repository in the named state,
run the printed command with the built executable first on `PATH`,
and check that no `policyId` finding comes back.
