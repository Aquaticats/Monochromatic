# agent-branch-landing-cli

Lands agents' branches on `main` in the main checkout, and keeps the two things a landing exists to prove: the tree that lands was
compiled with what `main` gained since the agent checked its own branch, and the merge went through the Git wrapper.

The commands replace the scratch scripts in `${HOME}/temp/agent/pi-runs` that did this many times a day:
`land-branch-on-main.ts`,
`check-branch-after-merging-main.ts`,
`check-package-branch-on-main.ts`,
`check-package-branch-and-land.ts`,
`check-branch-with-tasks.ts`,
`check-and-land-shell-analyzer.ts`,
`check-and-land-wrapper-lint-branch.ts`,
`retry-open-merge-commit-on-main.ts`,
`retry-merge-commit-in-worktree.ts`,
`keep-branch-side-of-conflict-markers.ts`,
`keep-head-side-of-conflicts.ts`,
`resolve-worktree-conflicts-keeping-main-side.ts`,
`show-merge-conflicts-of-worktree.ts`,
`list-worktrees-and-unmerged-pi-branches.ts`,
`stale-detached-worktrees.ts`,
`stale-worktrees-dry-run.ts`,
`remove-landed-agent-worktrees.ts`,
`remove-stopped-codex-audit-worktrees.ts`,
and every `land-branch-on-main` round: `land-wrapper-branches-round-*.ts`,
`land-package-branches-round-*.ts` and its `.json` lists,
`land-doc-only-branches-round-*.ts`,
`land-doc-branches-round-*.ts`,
`land-branches-round-*.ts`,
`land-package-branches-in-order.ts`,
`land-two-wrapper-fixes-after-alpha-6.ts`,
`land-wrapper-branches-before-the-next-gate.ts`,
`land-claude-code-plugin-branch.ts`,
`land-push-delivery-then-gate-failures.ts`,
`land-tokio-then-round-43.ts`,
`land-round-100-then-gate.ts` and `land-round-107-then-gate.ts` (their landing half; they also start the release gate, which is
not this package's job).

`check-branches-landed-and-reflow-is-whitespace.ts` is replaced by `worktrees` for its counts and by the landing's own reflow
lines; the one-off word-stream dump it printed is not a command.

## What a landing proves, and what it deliberately does not

Every agent ran its own suites on its own tree. What it could not run is its work combined with what `main` gained since, and what
breaks there first is the compile: `367c52c390` on top of `e590c0554d` gave 10 type errors with each branch green alone.

So a landing runs the compile, or the cheapest type check a changed package declares, and never the tests another agent already
ran. The owner, 2026-11-10: "I don't think you'd need to rerun the same lints or tests another agent did, do you?"

Both wrapper test targets are compiled, because `b9000c1929` broke only the binary target with the unit target and every other
check green, and no landing noticed until a release gate did.

A branch that changes `package/git-policy/` or a `Cargo.toml` or `Cargo.lock` is reported as changing a path a release gate
depends on: a release refuses a dispatch when such a path changed after its gate ran (`package/crate-release/cli/README.md`).

## The command

```
mise run //package/agent-branch-landing/cli:land -- --branch=<name> --message=<text>
```

Run without arguments it prints its usage. Every subcommand takes `--main-checkout=<folder>` (by default the checkout the command
runs in), `--git-timeout-seconds`, `--commit-tries`, `--commit-pause-seconds` and `--commit-timeout-seconds`.

`land`
: lands one branch. It merges the branch without committing (`git merge --no-ff --no-commit`), refuses when a merge is already
open, compiles the merged tree when the merge touches the wrapper, commits through the wrapper while its scratch commit ceiling
refuses, and reports the last commit of `main` and how many commits of the branch are still not on it. When Git refuses because
the main checkout holds uncommitted edits the merge would overwrite, an edit that only moves whitespace is committed as it stands
(`style(*): commit the reflow a format run left in the main checkout`) and the merge is tried again; an edit that changes a word
stays, and the landing says which files it left.

`check`
: `--worktree=<name> --check=wrapper|package|tasks|none` merges `main` into the agent's worktree and runs only what the branch
asks for. `wrapper` compiles both wrapper targets with `--no-run`; `package` runs the cheapest declared task of each package the
branch changed; `tasks` runs exactly the `--task` names given; `none` runs nothing and reports that nothing would check it.
`--filter` is accepted from older callers and ignored.

`list`
: `--list=<file>` lands each entry of a JSON list, one at a time, after the check its entry asks for. A branch the check refuses is
skipped and leaves `main` alone, so the run continues; a landing that does not finish stops the run, because later branches must
not land on a broken `main`. `--only=<key>` runs just the entries with those keys.

`commit-open-merge`
: commits the merge left open in a checkout, trying again while the wrapper refuses, and stopping at once when Git says there is
nothing to commit. `--worktree=<name>` names an agent's worktree instead of the main checkout, and `--path=<path>` limits the
commit to those paths.

`resolve-conflicts`
: `--worktree=<name> [--path=<path>[=<side>] ...]` keeps one side of each conflict region, stages the files and commits the merge.
With no `--path` it only describes the conflicts and whether a merge is open. The sides are named by role: `current` is the side
the checkout holds (Git's `<<<<<<<`), `incoming` is the side merged in (`>>>>>>>`), and `current-then-incoming` and
`incoming-then-current` keep both. In a merge of `main` into an agent's worktree, `current` is the branch and `incoming` is
`main`.

`worktrees`
: reports every worktree with its branch, how many commits `main` lacks, whether a `pi-` unit still runs it, and whether
anything would be lost by removing it.

`remove-worktrees`
: with `--apply`, removes the worktrees nothing would be lost by removing, after repeating every check in the same run. Without
it, it only reports. A worktree stays when Git holds it locked, a `pi-` unit still runs it, a process works inside it, it was
touched within a day, it holds a changed or untracked path that is not a build output, its branch has commits `main` lacks (or a
detached HEAD's commit is on no branch), or it holds an ignored entry that is not a build output.

## The list file

```json
[
  {
    "worktree": "pi-module-pipe-ready-bugs",
    "check": "package",
    "message": "merge(pipe-core): the sync and async ladders share one prologue"
  },
  {
    "branch": "fix/git-policy-cli-issues-4357-6274",
    "check": "none",
    "message": "merge(*): the record of the wrapper issue group 4357 to 6274"
  },
  {
    "worktree": "pi-commit-needs-no-shadow-when-it-can-avoid-one",
    "check": "wrapper",
    "key": "shadow",
    "message": "merge(git-policy-cli): the shadow pool lives in its own folder"
  }
]
```

- `worktree`: the folder under `.claude/worktrees/`, without the folder itself. Older lists name it `agent` or `name`, which are
  still read.
- `branch`: the branch to land, read from the worktree when it is absent.
- `check`: `wrapper`, `package`, `tasks` or `none`.
- `tasks`: the tasks a `package` check runs beside the derived ones, and the whole set a `tasks` check runs. Older lists name it
  `consumerTasks`.
- `message`: the merge commit's message. It is required.
- `key`: a name `--only` selects the entry by.
- `note`: why the check is enough, printed with the entry. The older key `lintEqualToMain: true` is read as a note saying the
  branch's lint findings are the same on `main`.

An entry that states no `check` gets the one its own fields mean, so the lists the scratch rounds wrote keep working: `tasks` means
`tasks`, `consumerTasks` means `package`, `agent` means `wrapper`, `name` or `worktree` means `package`, and a branch alone means
`none`.

## Exit codes

`doc/decision/cli-output-and-exit-codes.md` owns these: `0` when the command did what it was asked, `1` when it tried and did not
finish, `2` when the caller or the checkout is wrong (an unknown option, a list entry with no message, a folder that is not a
repository, a worktree on a detached HEAD, a merge already open before the command started). The scratch scripts used four more
codes for the kinds of failure they could meet; each of those is now a sentence on standard output.

## Bounds

Every command a landing starts runs in its own cgroup, sized to the work (`WRAPPER_COMPILE_LIMITS`, `WORKTREE_COMPILE_LIMITS`,
`PACKAGE_TASK_LIMITS`, `OFFLINE_INSTALL_LIMITS` in `src/constants.ts`), and every one has a bound, so nothing waits forever. The
numbers and the incident behind each one are beside the constant.

## Tests

`mise run //package/agent-branch-landing/cli:buildAndTest`.

This machine's Git wrapper refuses a commit outside its own commit service, so a test answers Git through the seam the sequences
already ask through (`src/fixtures-of-a-fake-repository.ts`), and reads the command lines a landing built from what was asked.
