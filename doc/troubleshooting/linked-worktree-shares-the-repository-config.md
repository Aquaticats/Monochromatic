# A Git config write in a linked worktree changes the repository every worktree shares

MADE 2026-11-10 by an agent,
after `3e75a49852` (2026-11-10T16:00:05-05:00).
FOUND and repaired the same day by the owner,
after the landing of `e5e689c8a5` on `main`:
from that landing on,
every push of `main` failed.

## Symptom

Every push of `main` failed with:

```text
fatal: 'DISABLED' does not appear to be a git repository
```

Nothing in the pushed commits named `DISABLED`.
The repository's own configuration held a push URL of `DISABLED`,
which a person had not set.

## Root cause

An agent wanted a throwaway checkout of an older commit to test a claim.
It made one with `git worktree add` inside this repository's worktrees and then ran this in it,
as the auto-push rule tells an agent to do in a clone of its own:

```console
# ${HOME}/temp/agent/base-6ad29d302d, a worktree this repository registered
git remote set-url --push origin DISABLED
```

A path `git worktree add` registers is a linked worktree and not a copy of the repository:
its `.git` is a file naming the repository's `.git/worktrees/<name>`,
and the configuration a plain Git command reads and writes there is the repository's one
common file.
`git-worktree(1)`,
section CONFIGURATION FILE:

```text
By default, the repository config file is shared across all worktrees.
```

So the command set the push URL for the whole repository,
the main checkout and every other agent's worktree included.
The agent wanted a scratch copy and reached for a worktree rather than a clone,
which `AGENTS.md` rule `APQ` names as the one place never to run that command,
and the agent did not apply that clause.

## Detection

Ask for the stored push URLs without printing them,
from any worktree of the repository:

```console
git config --get-all remote.origin.pushurl | wc --lines
```

A count of zero means no push URL is overridden.
`git remote -v` also shows the push URL beside `origin` when one is set,
and a push URL that is not a repository is the one to remove.

## Recovery

Remove the entry from the repository's common configuration,
from any worktree or from the main checkout:

```console
git config --unset-all remote.origin.pushurl
```

`git remote set-url --push origin <the real push URL>` sets it back to a URL instead.
A push of a branch then works again,
and that push is the proof the repair took.

## What does not work

- Removing the worktree.
  `git worktree remove` deletes its directory and its registration and leaves the common
  configuration exactly as the write left it.
- Reading `git worktree add` as a copy.
  It gives a working tree with its own `HEAD` and index and no configuration of its own.
- Relying on `extensions.worktreeConfig`.
  With that extension on,
  per-worktree settings live in `config.worktree`,
  but only `git config --worktree` writes there;
  `git remote set-url` and every plain `git config` write still reach the common file.
- Making the copy as a worktree wherever its directory sits.
  `git worktree add` registers the worktree in this repository,
  so a path under `${HOME}/temp/agent` shares the one configuration exactly as a path beside the
  checkout does,
  and only a real clone has a `.git` directory of its own to write in.

## The rule that follows

A copy to experiment in is a real clone,
made under the run folder,
and the push URL is disabled inside it:

```console
git clone --no-hardlinks <path to this repository> ${HOME}/temp/agent/<name>
git -C ${HOME}/temp/agent/<name> remote set-url --push origin DISABLED
```

`AGENTS.md` states it twice,
and both rules apply to every worktree of this repository:

- `GCW`:
  never write Git config in any worktree of this repository,
  because every linked worktree shares one `.git/config`,
  so one write changes the main checkout and every agent's commands.
- `APQ`:
  disable auto-push in a `git clone --no-hardlinks` of your own,
  never in a path `git worktree add` made.
