# monochromatic-git-policy-cli

A Git wrapper that enforces repository-owned policies.
It is installed as `git` ahead of the real Git on `PATH`,
forwards every command to the real Git,
and checks the commands and the commits that pass through it against policies the repository configures as data.

The executable is `cli-git-native`.
The library inside the crate is named `git_policy_cli`.

How the owner wants this package designed and judged is in [DESIGN.md](DESIGN.md).

## What it does

Every Git command goes through the wrapper first.
Read-only commands,
 and commands the wrapper has no rule for,
 run the real Git unchanged.

- Rules before forwarding.
  `require-root` demands the repository top level as the working directory.
  `linked-worktree-only` keeps every command form that can discard uncommitted work out of the main worktree,
   and out of a linked worktree while it holds uncommitted work (see "Discarding commands").
  `branch-worktree-only` sends branch creation through `git worktree add`.
  `add-explicit` rejects bulk staging patterns such as `git add .`.
- Commits landed by a service.
  A `git commit` is prepared in a private index,
   the content policies check the exact bytes,
   and only then does the commit service of the worktree land it on the real index and branch
   by compare-and-swap.
  The service is started by the operating system on the first commit (socket activation);
   the wrapper never starts it,
   and on a machine where no service can be reached or started a commit stops with a plain message and changes nothing.
  When another commit lands first,
   the wrapper prepares its own commit again on top of the new state.
  Concurrent commits from several worktrees or sessions therefore never overwrite each other.
  The wrapper takes no lock of its own: a commit meets only Git's `index.lock`.
  A commit the service was landing when it died is settled by the service from its journal when it starts again.
- Merges beside other sessions' staged work.
  Native Git refuses every `git merge` that creates a commit while the index holds any staged change,
   even when the merge touches none of the staged files.
  When that is the only obstacle,
   the wrapper computes the merge from Git objects,
   commits it with the parents, message and signing native Git would use,
   and lets Git's own fast-forward update move the checkout to it.
  Every staged entry, unstaged edit and untracked file that the merge does not change stays byte for byte as it was.
  A merge that changes a path someone else has staged or modified, or that conflicts,
   is refused with the paths named and the ways forward, and changes nothing.
  Shapes the wrapper cannot make exact (`--squash`, `--no-commit`, `-s`, `-X`, several branches, an editor) run in
   native Git unchanged; see "Merging with staged changes" for the full list.
- Hooks.
  cli-git supports no external hook program: not a traditional hook (an executable script in `.git/hooks` or in a
   `core.hooksPath` directory) and not a hook configured with `hook.<name>.event` and `hook.<name>.command`.
  Every real Git the wrapper starts, forwarded commands and its own children alike, runs with
   `-c core.hooksPath=/dev/null/cli-git-no-hooks`, a directory that cannot exist, and `-c hook.<event>.enabled=false`
   for every hook event Git knows, after your global options and before the subcommand, so no `-c` of yours brings a
   hook back.
  The wrapper injects no hook of its own, so `pre-push` is switched off like every other event.
  A script that starts `/usr/bin/git` directly, outside any Git the wrapper started, still runs hooks.
  The wrapper runs no program that any configuration names: what a hook checked is a cli-git policy under `policies`
   in `cli-git.config.jsonc`, and there is no setting that brings a hook back.
  A `postCommit` key in `cli-git.config.jsonc` is a configuration error that says so.
  A commit in a repository that configures hooks prints one non-blocking `configuration-warning` event
   (code `external-hooks-unsupported`) naming the keys, the events and the ways forward,
   and `git cli-git doctor` lists the same hooks.
  A commit in a repository that sets `core.hooksPath` prints one such event too (code `traditional-hooks-unsupported`).
  The shims `git lfs install` writes are the expected case: see "Git LFS" for what replaces them.
- Git LFS.
  Every push the wrapper starts uploads the LFS objects it publishes pointers for, before any ref moves,
   in a repository whose own Git configuration (local, worktree or `-c`; not the global file) has a `filter.lfs.*` or
   `lfs.*` key.
  When the pushed commits add pointers the step is `git-lfs push`, which the wrapper runs itself as a child before it starts the push;
   the push carries no hook, the wrapper never starts itself and writes no shell text.
  The remote and refs come from the push's arguments (everything for `--all`, `--tags`, `--mirror` and glob
   refspecs; the branch Git would send for a bare push), and a failed upload stops the push before it starts.
  The wrapper, as the parent that ran the upload, prints the explanation.
  The wrapper first reads the pushed commits itself, with its in-process readers, and finds the pointer files they add
   (compared with each parent, so a pointer added and removed again inside the push still counts).
  A push that adds no pointer starts no `git-lfs` and sends no request to the LFS server; a push that adds some runs
   `git-lfs push --object-id <remote> <oid>...` for exactly those objects.
  The remote's tracking refs say what the destination already has only for a configured remote that is fetched from
   and pushed to one place; a push URL, a `pushInsteadOf` rewrite, a URL or path destination, a mirror push or an
   unconfigured remote makes the wrapper read the whole history of the pushed refs instead.
  A repository the readers decline, or an object missing from `.git/lfs`, falls back to `git-lfs push <remote> <refs>`
   (with `--all` in the untrusted cases), which scans itself.
  When `git-lfs push` fails with a server error (an HTTP 5xx answer other than 501, 507 and 509, which `git-lfs` reports
   on a line starting `Fatal error: `), the wrapper runs it again after 2 seconds and, if needed, after 5 seconds;
   a refused credential, a missing local object or any other failure is not retried.
  `CLI_GIT_LFS_RETRY_DELAYS_MILLISECONDS` (a comma-separated list, at most five entries; empty for none) replaces the delays.
  The wrapper reads the standard error of `git-lfs` through a private temporary file and relays it with the user and
   password of every URL replaced by `<credentials>`.
  It covers a typed `git push` (including the merge push of a lost race), the auto-push after a commit and its
   reconcile pushes.
  `--dry-run` pushes and the wrapper's own probes never upload.
  `--no-verify`, `GIT_LFS_SKIP_PUSH` and `-c hook.cli-git-lfs-pre-push.enabled=false` skip it.
  Without `git-lfs` on `PATH` the push stops before anything starts and the message says to install it, remove the
   LFS configuration or use `git push --no-verify`.
  The other three hooks `git lfs install` writes (`post-checkout`, `post-commit`, `post-merge`) only keep lockable
   files read-only, do nothing without the `lockable` attribute, and are not run.
  See the SPEC section "Git LFS upload before a push" for the evidence and the limits.
- Content policies.
  `final-newline`,
   `markdown/autofix`,
   `mono/forbidden-root-context`,
   `mono/dependent-version-bump`,
   `mono/catalog-specifiers` and
  `security/forbidden-strings` check or rewrite what a commit contains.
  The same content policies scan what a manual `git push` newly publishes.
- After a landed commit.
  The wrapper checks the committed bytes again,
   then pushes the branch.
  A branch without an upstream is pushed to `origin` with `--set-upstream` when that remote exists.
  A branch with an upstream is pushed to it,
   including a branch whose upstream has another name, such as a linked worktree branch created from `origin/main`:
   Git itself refuses that push under its default `push.default=simple`, so the wrapper names the tracked branch.
  The resident service pushes the branch its record names and never reads `HEAD`.
   Where the settings leave the destination to Git, it asks Git:
   a `remote.<name>.push` refspec is resolved by `git for-each-ref` with the `%(push:remoteref)` atom,
   and `push.default=matching` pushes the same name only when the branch's `@{push}` remote-tracking branch exists.
   When Git names no destination (`push.default=nothing`, a triangular `upstream`, a refspec that does not cover the branch),
   nothing is pushed and the note says the branch tracks no remote branch that can be named.
  When the remote refuses that push because its branch moved after the commit landed,
   the wrapper merges the remote tip into the branch tip from Git objects alone and pushes the merge without force,
   at most ten times.
  While the merge is computed and pushed, the local branch, the index, the worktree and untracked files are not read
   or changed, so another session's uncommitted files cannot block the push.
  Afterwards the local branch follows the published merge by a fast-forward when nothing can be lost:
   a worktree with uncommitted work follows too, as long as the move writes none of the files that work touches,
   and the report says the work was left as it is.
   Otherwise the branch stays behind and the report names the overlapping files.
  A conflict is reported with the conflicting paths and publishes nothing.
  An amended or rebased branch is not merged, and its rejection is shown as before.
  A failed push never fails the commit,
   and no configuration key turns the push off.
  Automatic maintenance,
   as native Git runs it after a commit,
   runs after each landing as work of the commit service,
  so a commit returns before it.
- A manual push that lost a race.
  When another session pushes to the same branch between your commit and your `git push`,
   Git rejects the push as non-fast-forward.
  The wrapper runs a plain push of one branch as a child process with your terminal,
   so Git's own output is unchanged.
  When that push fails and the remote branch has commits yours lacks,
   the wrapper merges them in from Git objects alone and pushes the merge by running itself,
   so every push policy reads the integrated result,
   at most ten times.
  A conflict is reported with the conflicting paths and publishes nothing.
  The checked-out branch follows the published merge under the same conditions as after a commit;
   a detached HEAD stays where it is, and the report says so.
  Forced, deleting, mirroring, tag, `--all`, `--set-upstream`, `--porcelain` and `--dry-run` pushes,
   and pushes of a `+refspec`, are never integrated,
   and a failure that is not a divergence (authentication, offline, a refusing hook) keeps Git's own report.
- Linked worktrees.
  When a Git command creates a linked worktree,
   the wrapper copies the ignored state of the source worktree into it
  under one settlement lock per repository (this lock remains until the copy moves into the commit service).
- Signals.
  The wrapper catches `SIGINT`,
   `SIGTERM`,
   `SIGHUP` and `SIGQUIT` while it waits for Git.
  A signal sent by another process is relayed to the Git child,
   and the wrapper then ends without leaving state of its own behind.

## Requirements

- Linux with glibc 2.39 or newer.
  Release archives exist for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` only.
  They are built on the `ubuntu-latest` runner and link `pidfd_spawnp` and `pidfd_getpid`, which glibc added in 2.39
  (measured on the 0.2.2 x86_64 archive with `objdump -T`), so they do not start on Debian 12 or Ubuntu 22.04.
  The crate does not compile for Windows: `cargo check --target x86_64-pc-windows-gnu --lib` fails with 16 errors in
  eight files (the `tempfile` and `ignore` dependencies are unix-only and several modules use `std::os::unix`),
  so the Windows-specific branches in the sources have never run; each is marked `UNTESTED ON WINDOWS` in the source.
  A Windows port is deferred to a later version (owner decision, 2026-10-15).
- Real Git 2.55.0 or newer on `PATH` after the wrapper.
  The wrapper checks the version before work that depends on Git's behavior,
   such as a commit,
  and refuses older Git.
  The wrapper finds the real Git by walking `PATH`,
   skipping itself and any shim that leads back to it.
- `git-lfs` on `PATH`, only in a repository whose own Git configuration has `filter.lfs.*` or `lfs.*` keys.
- The scanner and the Markdown linter are linked into the executable and need no separate installation.

## Install

The crate is published on crates.io.
Install it as a `cargo:` tool of [mise][mise],
 the way the repository installs `monochromatic-lint`:

```toml
# mise.toml
[tools]
"cargo:monochromatic-git-policy-cli" = { version = "latest", os = ["linux"] }
```

`mise install` downloads the release archive of the crate's GitHub release when one exists for the host.
Otherwise it compiles the crate from source with the Rust toolchain.
`cargo install monochromatic-git-policy-cli --locked` compiles the same executable into `~/.cargo/bin`.

The installed file is `cli-git-native`,
 not `git`.
Link it as `git` in a directory that comes before the real Git on `PATH`:

```sh
# Run in the repository once per checkout, after the installation.
mkdir --parents .cache/cli-git-bin
ln --symbolic --force "$(mise which cli-git-native)" .cache/cli-git-bin/git
```

Never name a file `git` inside the tool's own `bin/` directory.
`mise` would then create a `git` shim for the whole account,
 and every Git command would run through the wrapper.

`PATH` shadowing covers only processes that find `git` through `PATH`.
An absolute path to the real Git,
 a `PATH` without the link directory,
 a GUI with an embedded Git library and direct
libgit2 consumers bypass the wrapper completely,
 including its policies.

## Commands

The wrapper adds one namespace to Git.

```sh
# Check or fix the whole repository, or the named pathspecs.
git cli-git check --all
git cli-git fix -- docs/ README.md
git cli-git check --all --policy final-newline
# List the hook files and configured hooks the repository holds; none of them runs.
git cli-git doctor
git cli-git --help
```

The wrapper never starts its own executable, never writes shell text or scripts, and explains a failure from the
parent process that owns the child (see `doc/planning/cli-git-no-self-invocation.md`).
A Clippy rule bans `std::env::current_exe` and `src/native/no_self_invocation_tests.rs` scans the source for the rest;
nothing is on its exception list that starts the wrapper.

Exactly one scope is required:
 `--all`,
 or pathspecs after `--`.
`check` and `fix` read the worktree files the scope selects,
 unstaged changes included;
 a commit or a push checks the staged or committed content instead,
 so a violation that exists only in the worktree fails `check` but not a commit that leaves it unstaged.
`--policy <id>` limits the run to one policy and may be repeated.
A selected policy that is off stops the run with exit `2`,
 because it would check nothing and read as clean:
 a `config-invalid` event on standard output names the policy
 and the `cli-git.config.jsonc` key that turns it on
 (`"policies": { "<id>": "error" }`,
 or the value the file now sets to `"off"`).
This holds for the four older optional policies as well,
 which used to exit `0` with no output in that case.
A selected policy that is on but does not run for the command is refused the same way,
 with the command that does run it:
 `fix --policy mono/catalog-specifiers` stops with exit `2` and points to `check`.
Without `--policy` an off policy is skipped without a word,
 and so it is when a commit,
 an add or a push runs.
Events are JSON lines on standard output.

`git cli-git doctor` takes no options.
It scans `<git-common-dir>/hooks` and the directory a configured `core.hooksPath` names,
 and prints the path of every traditional hook it finds, one per line, on standard output:
 each executable regular file and each symbolic link, except names ending in `.sample`.
A hook file that is byte for byte the shim git-lfs writes for its own name is not listed there:
 it is explained on standard error, with its path, as normal and harmless (git-lfs writes these files again whenever
 it cleans a file in a Git the wrapper did not start; the wrapper does the LFS upload itself and never runs them;
 delete them or leave them).
It exits 0 when it finds none or only such shims, 1 when it finds any other hook, and 2 when it cannot locate or read
 the repository.
Traditional hooks are not supported,
 so a script that still checks something should become a cli-git policy under `policies`.
It also lists the hooks configured with `hook.<name>.event` (one `hook.<name> (events: ...)` line each, exit 1) and
 notes a `hooks` key in `cli-git.config.jsonc`, which is accepted for older files and ignored.
A commit does not scan for traditional hooks: it only warns when the configuration sets `core.hooksPath` or defines
 hooks, which it already knows without any extra file system access.

The former `trust`,
 `untrust` and `status` commands are retired.
Configuration is data,
 so nothing needs approval.
They print an explanation and change nothing.

### Version and help

The wrapper runs the real Git for the top-level `--version`, `-v`, `--help` and `-h`,
 so Git's own output and exit status come first and are never changed.
Only when Git exits with status 0 does the wrapper add plain text to standard output,
 so a person can tell that a policy wrapper sits in front of Git.
When Git exits with another status, or a signal ends it, nothing is added and the wrapper returns Git's status.
Global options before the query, such as `git -C <directory> --version`, behave the same.
A subcommand's own help, such as `git commit --help`, is untouched and so is every other query
 (`--exec-path`, `--html-path`, `--man-path`, `--info-path`, `--list-cmds=<group>`):
 the wrapper replaces itself with Git and their output is byte for byte Git's.

`--version` and `-v` add exactly one line,
 where `<version>` is this crate's version and `<path>` is the real Git the wrapper resolved:

```text
monochromatic-git-policy-cli <version>, shadowing git at <path>
```

`--help` and `-h` add a blank line and a short section:

```text
monochromatic-git-policy-cli <version> sits in front of this git
It adds repository policies on commit, push and worktree commands,
the `git cli-git` management command (`git cli-git --help`),
and `git cli-git doctor`, which lists the traditional hooks a repository holds.
real git: <path>
```

A script that compares the output of `git --version` with a fixed string should read the first line only,
 or call the real Git by its path.
The wrapper's own findings stay JSON lines as described under "Output and exit status";
 this text appears only for these four spellings.

### Wrapper-only options

A wrapper-only option is removed before Git sees the arguments and applies to that one invocation.

- `--no-enforce-<policy id>` skips one policy,
   for example `--no-enforce-final-newline` or
  `--no-enforce-security/forbidden-strings`.
- `--no-enforce-worktree` skips `linked-worktree-only` for the forms listed in "Discarding commands".
  Write it directly after the subcommand,
   for example `git restore --no-enforce-worktree -- <path>`.
- `--no-enforce-worktree-branch` skips `branch-worktree-only`.
- `--no-enforce-bulk-add` skips `add-explicit`.
- `--no-worktree-copy` skips the ignored-state copy into a new linked worktree.
- `--cli-git-keep-going` reports every finding of the command in one run instead of stopping at the first blocking one.

### Hints

Every block and every divergence from real Git's behavior says what you were probably trying to do and how to proceed.
A JSONL event (a finding, an engine failure or a warn-unsafe notice) carries the hint in its own `hint` field,
 beside a `message` that is unchanged from the retired wrapper's text.
A plain notice (an internal error, a push stopped by its deadline or its output cap, a lock wait that gave up,
 a failed worktree copy, a real Git that cannot start, a commit that normalization emptied) ends with the hint.
The rejections that already carry their own alternatives (`require-root`, `linked-worktree-only`,
 `branch-worktree-only` and the commit-only rules) keep them in the message.
`src/native/event_hints.rs` holds the texts, and its tests fail when a finding code or an engine failure code has no hint.

### Discarding commands

`linked-worktree-only` rejects a command form that can overwrite or delete uncommitted work by itself.
The rule is:

- In the main worktree such a form is always rejected,
   clean or not.
- In a linked worktree it is rejected while the worktree holds uncommitted tracked changes
   (for `git clean`, also untracked files),
   and runs when the worktree is clean.
- A form that only moves `HEAD` (`git checkout <branch>`, `git switch <branch>`) is rejected in any worktree
   only while it holds uncommitted tracked changes.
- A status Git cannot read stops the command instead of reading as clean.
- Repositories under an allowlisted tool directory are exempt: uv's cache (`UV_CACHE_DIR`, else
   `XDG_CACHE_HOME/uv`, else `~/.cache/uv`) and mise's data directory (`MISE_DATA_DIR`, else `XDG_DATA_HOME/mise`,
   else `~/.local/share/mise`), where mise keeps its plugin clones and updates them with a forced checkout.

Every rejection names the form,
 what it would destroy,
 the alternative that fits the most likely intent,
 and the deliberate way to discard anyway.
The deliberate way is always the same:
 save a copy (`git diff > backup.patch`) and pass `--no-enforce-worktree` directly after the subcommand.
`git apply backup.patch` brings the edits back.

Each decision about a discarding form (rejected, run in a clean worktree, run in an exempt cache, or run with the
 escape) is appended as one JSON line to `cli-git-audit.jsonl` in the common Git directory,
 with the time, the process and its parent, the worktree kind, the form, the working directory and the arguments.
When uncommitted work disappears, `git rev-parse --git-common-dir` locates the file;
 a line that says `allowed` or `hatch` near the time of the loss names the command that was let through.
The file moves aside to `cli-git-audit.jsonl.1` at one mebibyte, and a failed write never changes a command's result.

The forms and their alternatives follow.
Each alternative is run through the built executable by `src/native/binary_discard_tests.rs`.

- `git stash` (every form except `list`, `show` and `create`):
   commit work in progress with explicit paths on a topic branch in its own linked worktree
   (`git worktree add -b <branch> <path>`).
- `git clean` without `--dry-run`:
   `git clean -n` lists what would go,
   delete specific files with `rm`,
   or use the project's own clean task.
- `git reset --hard`, `--merge`, `--keep`:
   `git reset --soft <commit>` undoes a commit and keeps its changes,
   `git diff <commit>` compares,
   `git worktree add <path> <commit>` gives a clean copy.
- `git reset` in the default or `--mixed` mode:
   `git commit --message <message> -- <paths>` commits only the named paths and ignores other staged entries,
   `git diff --cached --name-only` lists what is staged.
- `git restore --staged` (unstaging):
   the same two commands as `git reset`.
- `git restore` and `git checkout` with paths,
   `--force`,
   `--patch`,
   `--ours` or `--theirs` (reverting files):
   `git diff -- <path>` and `git show HEAD:<path>` compare,
   `git worktree add <path> <branch>` opens another branch,
   or save a patch and use the hatch.
- `git checkout <branch>` and `git switch <branch>` while the worktree is dirty,
   and `git switch --force`:
   commit with explicit paths first,
   or open the branch with `git worktree add <path> <branch>`.
- `git rm --force`:
   `git rm <path>` refuses a modified file,
   and `git rm --cached <path>` stops tracking and keeps the file.
- `git mv --force`:
   pick a destination that does not exist,
   or `git rm <destination>` first.
- `git checkout-index --force`:
   `git checkout-index --prefix=<directory>/ --all` writes copies elsewhere.
- `git read-tree --reset`:
   `git read-tree -m` merges,
   or reset in a clean linked worktree.
- `git merge --abort`,
   `git rebase --abort`,
   `git cherry-pick --abort`,
   `git revert --abort`,
   `git am --abort`:
   the same command with `--quit` stops the operation and keeps the index and files,
   `--continue` finishes it after resolving conflicts,
   and `--skip` drops the current commit or patch.

The forms are read with Git's own option grammar,
 so abbreviations,
 clustered short flags (`-rf`) and a path after `--` are handled as Git handles them.
Commands without a table of their own (`rm`,
 `mv`,
 `checkout-index`,
 `read-tree`,
 `restore` and the `--abort` commands) take the hatch directly after the subcommand.

### Merging with staged changes

Several sessions can share one checkout.
When the index differs from `HEAD`,
 native Git refuses a `git merge <branch>` that needs a merge commit with
 `error: Your local changes to the following files would be overwritten by merge`,
 whichever files are staged.
The wrapper completes that merge instead, when all of these hold:

- the command is `git merge` with one commit-ish and only the options `-m`, `--no-edit`, `--ff`, `--no-ff`,
   `-q`, `-v`, `--stat`, `--no-stat`, `-n`, `--log`, `--no-log`, `-S`, `--gpg-sign`, `--no-gpg-sign`,
   `--no-verify`, `--allow-unrelated-histories` and `--no-progress` (full spellings only);
- the index differs from `HEAD`, and `HEAD` is not already ahead of or equal to the merged commit
   (native Git fast-forwards over unrelated staged work and answers "Already up to date" by itself);
- no merge, rebase, cherry-pick or revert is in progress, and the settings `merge.autostash`,
   `merge.verifySignatures` and `branch.<name>.mergeOptions` are unset;
- the merge would not open an editor: pass `--no-edit`, or set `GIT_MERGE_AUTOEDIT=no`
   (a merge started from a terminal opens one by default, and then native Git runs it and refuses it as before).

The merge tree comes from `git merge-tree --write-tree`, the commit from `git commit-tree`, signed when `-S` is given
 or `commit.gpgSign` is true.
Its message is Git's own (`git fmt-merge-msg`, then the whitespace cleanup of a commit that opens no editor),
 and the checkout then moves with `git merge --ff-only <merge commit>`.
That is Git's two-way update:
 it writes exactly the paths the merge changes and keeps every other index entry and file.
The merge commit skips `pre-merge-commit`, `prepare-commit-msg` and `commit-msg`,
 like the merge the auto-push makes from objects;
 `post-merge` runs once.
The branch reflog reads `merge <branch>: Fast-forward`.

Two refusals change nothing:

- The merge changes a path that is staged, or modified in the worktree, by someone else.
  The finding `merge-isolation/paths-in-use` names the paths.
  The ways forward are to wait until the owner has committed or restored them,
   to merge in a clean linked worktree and bring the result back with `git merge --ff-only`,
   or, for your own files, to commit or stash them first.
- The merge conflicts textually.
  The finding `merge-isolation/merge-conflict` names the conflicting paths.
  Resolve in a clean linked worktree or checkout, or rebase the branch.

An untracked file where the merge adds one is refused by Git at the last step;
 the finding `merge-isolation/not-applied` follows Git's message, and nothing is changed.
A merge killed while it runs is finished or discarded by the next invocation (see "State on disk").
`git pull` runs its merge inside Git, out of the wrapper's reach,
 so a pull that needs a merge commit is still refused while the index holds staged changes;
 run `git fetch`, then `git merge --no-edit origin/<branch>`.

### Output and exit status

Policy events of a wrapped command are JSON lines on standard error,
 one object per line,
 with `schemaVersion` 1.
Real Git keeps the normal standard streams.

- When real Git does not run:
   `0` means clean or warnings only,
   `1` means final error findings,
   and `2` means a usage,
  configuration,
   commit or engine failure.
- When real Git runs,
   its exit status is preserved.
  A landed commit that a post-commit policy then blocks exits `2` and stays local.

## Configuration

The wrapper reads `cli-git.config.jsonc` at the repository top level.
The file is JSON with comments,
 at most 1 MiB,
 and it is data:
 it never names code to run.
Without the file,
 the defaults apply.
The former TypeScript wrapper read `cli-git.config.mjs` and `cli-git.config.ts`;
 those files are never executed or read.
With no JSONC file beside them,
 they are a configuration failure that asks for the translation.
Beside a JSONC file,
 they are reported and ignored.
The keys `plugins` and `trust` are refused.

```jsonc
// cli-git.config.jsonc
{
  "policies": {
    // A severity alone: "off", "warn" or "error".
    "final-newline": "error",
    // A severity with options.
    "markdown/autofix": ["warn", { "rules": ["lfs-image-url"], "exclude": ["package/ssg/"] }],
    "mono/forbidden-root-context": "error",
    "mono/dependent-version-bump": "error",
    "mono/catalog-specifiers": "error",
    "security/forbidden-strings": ["error", { "builtinRules": true, "rulesFile": ["forbidden-strings.append.txt", { "path": "forbidden-strings.append.local.txt", "optional": true }] }]
  }
}
```

An unknown key,
 a wrong type or an unknown policy id is a configuration failure that exits `2` before any policy runs.

### Policies

The wrapper ships these policies and no others.
The first five run at their default severity without being listed.
The last five run only where the file lists them.
Listing one with a severity is the only way to turn it on,
 and a configuration that does not name a policy behaves exactly as it did before the policy shipped.

- `require-root`:
   default `error`.
- `linked-worktree-only`:
   default `error`.
- `branch-worktree-only`:
   default `error`.
- `add-explicit`:
   default `error`.
- `final-newline`:
   default `warn`.
  Text files end with exactly one line feed.
  A file whose Git `text` attribute is unset (`-text`, which `binary` includes) keeps its exact bytes,
  so a package declares its exact-byte inputs, such as fuzz seeds, with `seed/** -text` in its own `.gitattributes`.
  The attribute is read from the commit or index being checked, never from the worktree.
  Files below `dist/final/node/` or `bundle/node/` are also exempt, as generated output.
- `markdown/autofix`:
   default `warn`.
  Runs the linked `monochromatic-lint` fix engine over staged Markdown inside the commit, in-process on a guarded thread.
  Options:
   `rules` (only `lfs-image-url` exists) and `exclude` (gitignore-style patterns).
- `mono/forbidden-root-context`:
   default `error`.
  No `CONTEXT.md` at the repository top level enters a commit.
- `mono/dependent-version-bump`:
   default `error`.
  Packages that depend on a package with a bumped version follow the bump.
- `mono/catalog-specifiers`:
   default `error`,
   no options.
  Every dependency in a tracked `package.json` is `catalog:`,
   `catalog:<name>` or exactly `workspace:*`
   (`peerDependencies` take `*`,
   `catalog:` or `catalog:<name>`).
  The rule belongs to the `monochromatic-catalog-specifier` crate,
   whose README is its contract;
   the policy hands it each candidate `package.json`'s bytes,
   never the worktree's,
   and reports its findings.
  A tracked `package.json5`,
   `package.yaml` or `package.yml` is a finding too,
   reported by name without reading it.
  The policy checks syntax only:
   it looks up no catalog entry and does not read `pnpm-workspace.yaml`.
  It runs when a commit is made,
   when `git add` stages,
   after a landed commit,
   before a push,
   and for `check`;
   it proposes no correction,
   so `fix` skips it.
  The finding codes are `unsupported-manifest-format`,
   `invalid-manifest`,
   `malformed-section`,
   `duplicate-key`,
   `non-string-specifier` and `disallowed-specifier`.
  A `warn` lets a literal version into the commit,
   so it earns the unsafe-`warn` notice.
  This repository's `cli-git.config.jsonc` does not list it yet,
   so the policy is off here.
- `security/forbidden-strings`:
   default `error`.
  Scans candidate bytes and path names with the linked scanner.
  Options:
   `builtinRules` (boolean,
   load the scanner's built-in baseline) and `rulesFile` (a rules file path relative
  to the repository top level, or an array of them).
  The rules files are `rulesFile`,
   else the environment variable `FORBIDDEN_STRINGS_RULES`,
  else `forbidden-strings.local.txt` in the repository top level,
   which may be absent.
  A file named by `rulesFile` or by the variable must exist, unless it is marked optional.
  Several files are read in order and scanned as one rule set,
  and every one of them is left out of the scan, as the single rules file always was.

The `rulesFile` option takes either form:

```jsonc
// One required file, the form that existed before lists.
{ "rulesFile": "forbidden-strings.append.txt" }
// Several files; a string is a required file, an object may mark one optional.
{ "rulesFile": [
  "forbidden-strings.append.txt",
  { "path": "forbidden-strings.append.local.txt", "optional": true }
] }
```

An optional file that is absent is skipped and recorded in the debug log.
A configured string is one path exactly as written, so a comma or a `?` in it needs no care;
an array never needs the variable's list syntax.
An empty array, an element that is neither a string nor an object with `path`, an object key other than `path` and
`optional`, and a path named twice are configuration failures.
Every path is checked the same way: relative to the repository top level, no `..`, no backslash.
`FORBIDDEN_STRINGS_RULES` uses the scanner's comma-separated list (see
`package/forbidden-strings/cli/README.md`, "Rules list"), where a trailing `?` marks an optional entry;
a value that is not valid UTF-8 is one required path.
Because the variable's entries are resolved against the repository top level too,
the same `{{config_root}}`-anchored value serves the scanner and cli-git.
An invalid variable fails the policy only when a candidate is scanned, like a missing file.
The new-worktree copy takes every configured file that Git ignores and the main checkout has.

#### Content already in history is not scanned again

`security/forbidden-strings` reads the bytes of a candidate
only when no parent commit's tree already holds that exact blob.
A pure rename, a copy of an existing file, and a merge that brings a file only one parent has therefore pass,
because a blob's object name is the hash of its bytes and the bytes are already in history.
What counts as a parent depends on the command:

- `git add`, and a commit being made: the commit the new commit is based on (the recorded base, else `HEAD`).
- The post-commit pass over a landed commit: every parent of that commit.
- The pre-push pass: every parent of a pushed commit that is not itself being pushed,
   so a parent that is part of the push vouches for nothing.
   A push that includes a tip with no known published history skips nothing.
- `git cli-git check`, a root commit and an unborn branch have no parent tree, so everything is scanned.

The comparison is by exact blob object name per path, never by path name or by similarity,
so one changed byte makes the file new and it is scanned in full.
The pathname of a skipped file is still scanned, because a rename gives the content a new name.
A file that a fix corrected during the same invocation is never skipped, because its bytes are no longer its object's.
If the parent trees cannot be listed, every candidate is scanned in full and a warning says why.

Each skipped file is recorded on standard error as a debug line when `MONOCHROMATIC_VERBOSE=true`:
the masked pathname, the blob's object name, and the commit whose tree holds it.

Tradeoffs.
Content that reached a parent through an earlier bypass, or that sits at a path the scanner never reads
(the rules file and the scanner's own rule sources), stays unscanned wherever it is copied or moved to afterwards.
That is also what lets the scanner's own package move to a new directory.
A rule added to the scanner after a file entered history is applied to that file again only when its bytes change.

A `warn` setting reports findings without blocking.
A policy whose protection `warn` would remove produces a configuration warning event when the file sets it to `warn`.

### Concurrency

- `hooks.concurrentCommits`:
   boolean.
  Retired: it tuned the hook lock, and the wrapper runs no external hook, so there is no lock.
  Older files that set it still load; the value is checked and ignored, and `git cli-git doctor` says so.
- `indexLock.unprovenOwnerTimeoutMs`:
   non-negative integer,
   default `1000`.
  Retired: it was how long the wrapper backed off for an `index.lock` whose owner is dead or unproven.
  Older files that set it still load; the value is checked and ignored.
- `landing.reserveAfterLostRaces`:
   positive integer,
   default `1`.
  Retired: it was how many lost landing races passed before a commit asked for the landing reservation.
  Older files that set it still load; the value is checked and ignored.

Concurrent commits cannot be turned off.

### Waiting and time limits

The wrapper never waits for another process without a limit,
and never waits without saying so.
The environment variables below set the limits and the pace of the notices,
each a positive whole number of seconds;
a zero,
negative,
fractional or non-numeric value is ignored.

- `CLI_GIT_LOCK_WAIT_SECONDS`,
   default `1800`.
  How long a command waits for a lock that a live process holds:
   the commit service waits for a foreign `index.lock` before it lands a commit,
   a forwarded index writer waits for the service's own hold on the index,
   and the worktree-copy settlement.
  At the limit it stops with a message that names the holder and every way forward:
   let the holder finish and run the command again,
   check with `ps` whether it is stuck,
   or raise the limit.
  The command leaves the lock in place and changes nothing else,
   so it can simply be run again.
  The commit service reads it from its own environment,
   so it is set where the service's units get their environment
   (`systemctl --user set-environment CLI_GIT_LOCK_WAIT_SECONDS=...`, then restart the service).
- `CLI_GIT_CHILD_WAIT_SECONDS`,
   default `60` for a quick query step and `300` for a bulk step.
  How long one Git process that the wrapper starts for itself may run before it is stopped.
  A quick query step only reads refs, commits and trees (`rev-parse`, `rev-list`, `ls-tree`, `cat-file`, `diff-tree`,
   `merge-base`, `write-tree`, `update-ref` and the like);
   the measured worst case is a fraction of a second whatever the worktree holds.
  A bulk step is every other step:
   it moves bytes (`pack-objects`, `index-pack`, `add`, `maintenance run`),
   or compares the index with the worktree (`reset`, `update-index`, `ls-files`, `diff`),
   or is not known to be a query.
  Its time grows with the size of what is committed and with how many files the index must read again
   (a stat-dirty 20 000-file index made `reset` take 2.9 s against 0.03 s on a clean one).
  The object reader (`git cat-file --batch`) is a bulk step too, because one request can name a large blob.
  Setting the variable gives every class the same bound.
  A process that crosses its bound is stopped,
   and the command reports which one, its class, the bound and how to go on:
   run the command again,
   or raise the bound when the step was only slow.
- `CLI_GIT_WAIT_NOTICE_SECONDS`,
   default `10`.
  How often a waiting command repeats its notice (see "What a waiting command prints"),
   and how often the commit service repeats the line that names the holder of a foreign `index.lock`.
- `CLI_GIT_PUSH_DEADLINE_SECONDS`,
   default `30`.
  How long the push after a landed commit may run before it is stopped.
  The commit stays saved,
   the failure is reported,
   and `git push` can be run by hand.
  Git's own read-only queries before the push stop at this deadline or 120 seconds,
   whichever is shorter.
- `CLI_GIT_PUSH_LOCK_WAIT_SECONDS`,
   default the push deadline plus 65.
  How long a commit waits behind another process's push before it gives up and reports its push as failed.

A command that passes through to Git unchanged,
 and the editor of a commit,
 run as long as they run and receive Ctrl-C like any Git command.

#### What a waiting command prints

Every wait of the wrapper writes to standard error,
so a queued command can be told from a hung one.
A wait that ends within one second prints nothing,
because healthy commits queue behind each other briefly all the time.
After that the first line comes,
and it repeats every `CLI_GIT_WAIT_NOTICE_SECONDS` seconds until the wait ends.
A wait whose holder is already known when it begins (a live `index.lock`,
 the worktree-copy settlement,
 another process's push) prints its first line at once.
Each line names what the command waits for,
 the process that holds it when the wrapper can tell,
 how long the wait has lasted and where it gives up.

- An `index.lock` held by a live process, which the commit service waits for before it lands a commit:
   the holder is proven by an open descriptor or Git's PID file and the proof is named,
   and an `index.lock` that nothing proves held is said to be unproven while the wait lasts.
   The first line of a wait adds a hint,
   and the repeats stay short.
- The commit service's own hold on the index, which a forwarded `git add` waits for.
- The worktree-copy settlement lock and another process's push of the same branch.
- A long run of `monochromatic-lint` for a Markdown file,
   which names the file and stops waiting for the linter after 60 seconds,
   leaving its thread to end with the command.
   The guard contains a hang and a panic, not a stack overflow or an abort in the linter.
- The object reader and the Git steps of a commit that run long,
   which name the Git command and the process,
   and stop it at the bound of its class (`CLI_GIT_CHILD_WAIT_SECONDS`).
  The first notice of a quick query step comes after 5 seconds and that of a bulk step after 10 seconds,
   so a step that only needed a moment prints nothing.

```text
// standard error of a git add that waits for the commit service to finish a commit
cli-git: git add is waiting for the commit service to finish a commit in this repository, 12 seconds so far; giving up after 1800 seconds.
```

## Reading the repository in process

The wrapper starts real Git as rarely as it can.
For read-only questions it reads the repository itself and falls back to Git for anything it does not fully understand.
The readers live in `src/native/repo_read_*.rs`; the design, the measurements and the limits are in
`doc/planning/cli-git-in-process-reads.md`.

What is answered in process, with the standard output, standard error and exit code Git itself would give:

- the repository location every wrapped command asks for (`rev-parse` with the bare, Git directory, common directory,
  top level and prefix options), and the Git paths and object format the commit path asks for;
- `ls-files --stage -z` and `ls-files --unmerged -z`;
- `diff-index --cached` (name-only and raw, with `--diff-filter` and literal pathspecs),
  `diff --cached --name-only -z`, `diff-tree --root -r -m` for a commit with at most one parent,
  and `show --no-patch --format=%s`;
- `config --list -z --show-scope`, `remote get-url`, `for-each-ref` with the wrapper's format,
  `rev-parse --verify --quiet <ref>^{commit}`, `rev-parse --abbrev-ref --symbolic-full-name @{upstream}`
  and `symbolic-ref --quiet HEAD`;
- `cat-file --batch` fed full object names, in the commit preparation.

Every reader answers only after checking what Git would check, and otherwise leaves the question to Git:
`GIT_*` variables that can move a path, global options other than `-C`, bare repositories, links, other file systems,
reftable, conditional includes, unmerged or intent-to-add entries in a diff,
a `config.worktree` that makes the repository bare, moves the worktree or holds a value the readers cannot prove
Git accepts (with `extensions.worktreeConfig` on, the file is otherwise read and listed under the scope `worktree`),
a file that became a directory, a gitlink, a possible rename, replace refs, shallow or grafted history,
split or sparse indexes, and configuration keys that can change the output.
Differential controls run real Git and the readers over the same command lines
(`src/native/repo_read_*_tests.rs`, `src/native/binary_budget_tests.rs`).

Set `CLI_GIT_IN_PROCESS_READS=off` in the wrapper's environment to start Git for every one of these questions,
for a measurement that compares both paths or to rule the readers out when output differs.

## Operating conditions

This package is one of the few that must keep working under the conditions its host lives in every day,
not only on a quiet machine.
The wrapper, the commit service and their tests are expected to work under these conditions:

- high CPU load from other sessions and builds;
- memory pressure and heavy swap use;
- slow SSD and other I/O trouble;
- a nearly full disk, so a write can fail with no space left in the middle of a commit;
- process and descriptor limits close to exhaustion;
- a process killed at any instruction, which the service's journal exists for;
- a closed terminal or a broken pipe;
- slow fsync and unusual filesystems;
- DNS or mirror trouble, where the wrapper recovers or falls back instead of wedging.

### Latency requirements

Two requirements bound the latency the wrapper adds, and `native:perf` enforces both:

- A forwarded real Git command gains strictly less than `2,000 ms` (`SPEC.md`).
- Every commit made through the wrapper must not take `5,000 ms` or more longer than the same commit made with native Git
  (each commit, not a percentile of them).

No wrapped command is required to finish within a fixed time, and there is no 5-second bound on wrapped commands.
The owner's words (2026-11-01):

> " 5-second bound on wrapped commands" - there is never a 5s bound on wrapped commands.
> The measure is always just commits vs native git should not take 5s longer.

The 5-second `timeout --signal=KILL` of the binary tests is a hang guard of the test harness
(`TIME_BOUND_SECONDS` in `src/native/binary_support.rs`), short so that a mutant that loops fails every affected control
quickly.
It says nothing about how long a correct wrapped command may take.

### What pins each condition

Each condition has permanent controls that start the built executable on real Git fixtures.
The wrapper's side of a commit no longer writes state that outlives the command, so the call-by-call fault sweeps of the
earlier landing (`strace --inject` failing, killing or delaying every write, rename, flush, descriptor and process start of a
commit, and the records they left to recover) retired with that landing.
The commit service's own faults are pinned by the kill points of its journal (`binary_service_tests.rs`: a kill at each point
of a batch, requests sent again, late and older services), the pruned-content controls (`binary_service_pruned_tests.rs`)
and the unit controls of its landing (`service_land_tests.rs`); a port of the call-by-call sweeps to the service is open.
What the faults found, and the change for each, is in
[the fault injection note](../../../doc/troubleshooting/git-wrapper-fault-injection-findings.md).

- A closed terminal or a broken pipe: `binary_fault_stream_tests.rs` closes the standard streams, gives them a pipe with no
  reader or a full device, and requires that the status matches the one Git gives when started directly.
- A Git step that stalls: `binary_child_bound_tests.rs` delays the return of every start of a Git step past its class
  bound (`strace --inject`, a `ptrace` of the child, no privilege and no load) and requires that the step is stopped,
  named with its class, and that the repository recovers.
  The control reports a skip and checks nothing on a host without `strace`.
- A process killed at any instruction elsewhere: the worktree-copy and merge flows are killed at their phase markers
  (`binary_merge_isolation_tests.rs`, `binary_worktree_copy_tests.rs`).
  A staging directory of a killed process stays on purpose and is ignored by every reader.
- DNS or mirror trouble: `binary_fault_remote_tests.rs` makes the push after a landed commit and an explicit push meet
  a refused connection, a name under `.invalid`, a server that hangs up, a server that never answers (stopped at
  `CLI_GIT_PUSH_DEADLINE_SECONDS`) and a `git-lfs` that never exits, and requires that the commit stays, the failure is
  worded, no lock is held and no process the push started is left running; `binary_push_environment_tests.rs`,
  `binary_lfs_pre_push_tests.rs` and `binary_lfs_server_tests.rs` cover the stalled push and the failing upload.
  The push is the service's work after the command returned, so each control waits for the work records to settle and reads
  the failure from the notice the next command shows.
- High CPU load, memory pressure and slow I/O: the test harness's 5-second hang guard, which loaded hosts have tripped,
  and the byte copy of the wrapper that keeps a cold start inside it (the troubleshooting note below);
  no control injects memory pressure.
  `RLIMIT_AS` is the only privilege-free candidate and was not tried: the address-space cap it sets is hit by Rust and
  Git allocations alike at points that depend on the build, so it is not expected to tell a wrapper defect from an
  allocator abort.

Known gaps: an editor that never exits is not bounded by `CLI_GIT_CHILD_WAIT_SECONDS` (it runs as long as it runs,
as documented, and is ended by the signals above); the worktree-copy and merge flows are killed at their phase markers
but not swept call by call; the Git steps of a commit that the wrapper bounds are killed without the processes below
them; no control sweeps the service's landing call by call.

Tests therefore never wait for a quiet host.
The 5-second hang guard on one wrapped command in the binary tests is deliberate,
and is not a requirement on the wrapper;
its measurements and the evidence needed to change it are in
[the troubleshooting note](../../../doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md).

## State on disk

The wrapper keeps its state beside the repository's Git data under `cli-git*` paths in the common Git directory.
Delete nothing there while a Git command is running.
A commit keeps a scratch directory `cli-git-service-<id>` in the worktree's Git directory while it is made;
 the commit service removes it when the command ends, however it ends.
The service's after-commit work records and the notices of a failed backup push are kept under `cli-git/`.
A directory `cli-git-transactions` in the worktree's Git directory belongs to an earlier release:
 this release does not read it, and a command that finds commit records in it names the directory once and goes on.
 Look at `git log -1` and `git status`, then remove it.

A merge in progress keeps a one-line record in `cli-git-merges` in the worktree's Git directory,
 from before Git starts updating the checkout until the branch has moved.
Do not remove it by hand unless a command says so.

The ignored-state copy into a new linked worktree stages its files in a `.cli-git-worktree-copy-*` directory
beside the new worktree, which can hold gigabytes.
The wrapper records that directory's name in the common Git directory before it creates it,
so a copy that was killed while staging cannot leak it:
the next invocation in the repository removes the directory and says so on stderr.
Delete a `.cli-git-worktree-copy-*` directory by hand only when no `git` command is running in that repository.

## Development

The commands below run in the repository that holds the crate,
[Aquaticat/Monochromatic][repository],
 from the repository top level.
Every task runs through `mise`.

```sh
# Run from the repository top level.
mise run //package/git-policy/cli:native:check
mise run //package/git-policy/cli:native:clippy
mise run //package/git-policy/cli:native:build
mise run //package/git-policy/cli:native:test:host -- <test name filter>
mise run //package/git-policy/cli:native:test:binary:host -- <test name filter>
mise run //package/git-policy/cli:native:test:container
```

- `native:check` type-checks every target.
  `native:clippy` lints every target with warnings denied.
  `native:build` writes the debug executable to `package/git-policy/cli/target/debug/cli-git-native`.
- `native:test:host` runs the unit tests and `native:test:binary:host` runs the tests that start the built executable,
  both against the host's Git while iterating.
- `native:test:container` is the authority.
  It builds the crate and runs the tests and Clippy in bounded containers with Git 2.56.0 and no network.
  Set `GIT_POLICY_NATIVE_IMAGE_TAG=<name>` to keep the images of parallel runs apart.
  The test run uses `cargo nextest run --no-fail-fast`, so every test runs even after a failure and one flaky test
  never hides the rest; the exit status is still non-zero, and nextest prints its summary and the list of failed tests
  last.
  The image build is layered so that a rebuild reuses everything the change did not reach, with no network and
  `--locked` throughout (`bin/native-layered-build.ts`, `bin/native-test-containerfile.ts`):
  - The dependency layer compiles every registry crate from the manifests, the lock files, the vendored sources and
    empty stand-in source files, so its cache key holds no source file.
    Only a change to a manifest or a lock file rebuilds it, which includes a version bump of the crate or of a path
    crate.
  - The path-crate layer copies the JSONC crate and the scanner with its engine and compiles them (the scanner's
    build script precompiles the embedded baseline), so it is rebuilt only when one of them changes.
  - The subject layer copies the wrapper's `src/native` and `README.md` (the tests include it) and compiles only the
    wrapper, so a test sidecar edit or a source edit recompiles one crate.
  - The linter stage follows the same split: its registry dependencies compile from stand-ins, then its own sources.
  - Each real-source layer touches the copied files in the same step, because Cargo compares file times with the
    stand-in build and would otherwise treat the stand-in output as current.
  - Build cache mounts (`RUN --mount=type=cache`) are not used: they keep `target/` outside the image, but the test
    run, the planted controls and the mutation campaigns reuse the image's own `target/`, so the compiled output has to
    stay in the image.
  - The image build keeps the same CPU limit as the run.
    In paired builds at the same time, a cold dependency layer compiled about 1.7 to 1.9 times faster with
    `--cpus 4` than with the default (47 s and 58 s against 23 s and 31 s) and the path-crate layer about 1.6 times
    faster, while the wrapper-only layer that a test or source edit rebuilds did not change (about 12 s either way).
    A cold build happens only after a manifest or lock file change, so the default is unchanged;
    pass `--cpus 4` after the `--` for that case (the test concurrency follows it too).
  - Every build step costs a fixed amount on a busy host (measured as tens of seconds, up to minutes, for a step that
    does no work), so the Containerfile keeps its step count low and puts the steps that rarely change first.
- `native:perf` builds the release executable and measures the latency it adds to forwarded Git commands and to
  commits (explicit-path and service, unsigned and SSH-signed),
  with paired direct-Git samples in disposable repositories.
  It reports the median, p95 and median absolute deviation of both sides and the added delta,
  and for each commit the median, p95 and maximum of the paired deltas (the wrapped time minus the direct time of the
  same round).
  It fails when a forwarded command's added p95 reaches the 2,000 ms bound,
  when any commit's paired delta reaches the 5,000 ms bound (the owner's measure, see "Latency requirements"),
  or when any sample fails.
  `-- --output <file>` stores the raw samples (baselines live in `perf/`; `schemaVersion` 2 adds the paired deltas and
  names each scenario's `kind`).
  The judging lives in `bin/native-perf-judge.ts`, and `bin/native-perf-judge.unit.test.ts` runs it on the recorded
  baseline, edited to break each bound.
- `native:lock:members` records a changed dependency list or version in `Cargo.lock` without moving any other locked
  version.

A release build of the executable:

```sh
# Run in package/git-policy/cli.
cargo build --release --locked --bin cli-git-native
```

The release profile keeps `panic = "unwind"` and `overflow-checks = true`.
The linked scanner turns a matcher panic into a failing finding by catching the unwind,
and the crate refuses to compile under `panic = "abort"`.

### Container limits

Every container the verification tasks start is bounded.
The defaults are 2 GiB of memory, 2 CPUs and 128 processes, defined once in `bin/container-limits.ts`.
Each task accepts overrides after the `--` of `mise run`, with these exact option names:

- `--memory <size>` is a whole number with an optional unit `b`, `k`, `m` or `g`, such as `512m` or `2g`.
- `--cpus <count>` is a number from `0.01` with at most two decimals, so `1`, `1.5` and `0.25` are valid.
  The build CPU quota is derived as `cpus * 100000` microseconds per 100000 microsecond period.
- `--pids-limit <count>` is a whole number of processes from `1`.
  It is accepted only where the container has a process cap, and rejected with a message otherwise.

A value is a separate token (`--memory 1g`); the `--memory=1g` form is not read and reaches the task's own option parser,
which rejects it as an unknown argument.
An invalid value stops the task before any container starts.
The message names the option and the accepted forms.
Options repeat with the last one winning, and the number of build jobs (`CARGO_BUILD_JOBS`, `make --jobs`) follows the
whole CPUs of the limit, never below one.

```sh
# Run from the repository top level.
mise run //package/git-policy/cli:native:test:container -- --memory 4g --cpus 4
mise run //package/git-policy/cli:native:mutation:scoped -- --memory 1g --cpus 1 --file src/native/lock.rs
mise run //package/git-policy/cli:native:interop -- --cpus 1 old-format-driver
mise run //package/git-policy/cli:test:e2e:concurrent:native --memory 1g --cpus 1
mise run //package/git-policy/cli:test:git-image -- <git checkout> --memory 4g
```

- `native:test:container` accepts all three options, and so do `native:mutation:scoped` and `test:git-image`
  (the Git checkout path comes first there).
- `native:mutation` runs the gate and then the mutation step, and mise passes the options to the mutation step only,
  so the gate runs at the defaults.
  Run `native:test:container` with its own options first when both need changing.
- `native:interop` accepts `--memory` and `--cpus` before the driver name, because everything after the driver name
  belongs to the driver.
  It has no process cap, so `--pids-limit` is rejected.
- `test:e2e:concurrent:native` takes `--memory` and `--cpus` as task flags and applies them to the run.
  The image build it depends on uses the defaults, and `test:e2e:concurrent:native:image` and
  `test:e2e:concurrent:image` take the same options after the `--`.
  The e2e containers have no process cap either.
- The mutation manifest records the limits it ran with, and the gate and image-build manifests record the container
  argument arrays.
- `native:mutation:scoped` reuses the image of the last gate, whose `CARGO_BUILD_JOBS` was fixed when it was built.
  Run the gate with the same `--cpus` for the job count to match.
- The `package/git-policy/cli.fuzz` campaign uses the defaults from the same module and has no override options yet.

### Who runs a mutation campaign

The session's main agent starts every campaign of this package and waits on it, never a subagent; the owner's reason
is in the launcher's [Who runs a campaign](../../mutation-test/cli-cargo-mutants/README.md#who-runs-a-campaign).

### Resuming a mutation campaign

`native:mutation:scoped` and `native:mutation:parallel` (and so `native:mutation`) are resumable.
A kill, a reboot, an out-of-memory stop or `systemctl --user stop <unit>` loses no scored mutant:
run the same task with the same options on the same gate image and it continues where the campaign stopped, scoring
only mutants that no earlier run scored, whether they were caught, missed, timed out or unviable.
For `native:mutation:parallel`, start the same launcher command again; each file resumes its own campaign.

The campaign's results are in `target/verification/native-mutation-<key>/`, where the key names the gate image and the
exact options.
While a campaign runs, `attempt-NNN/mutants.out/` there holds the outcomes scored so far in `caught.txt`, `missed.txt`,
`timeout.txt` and `unviable.txt`.
When it ends, `mutants.out/` holds the same four files as the union of every attempt, with `outcomes.json`, `log/`
and `diff/`, and `exit.json` records the status of the whole campaign (`attempt` and `skippedMutants` say how many
earlier mutants the last run left out).
To score everything again, delete that directory; a rebuilt gate image gets a new one by itself.
The mechanism, the choice of cargo-mutants `--iterate`, and the limits are in
`package/mutation-test/cli-cargo-mutants/README.md`.

## License

`LGPL-3.0-or-later`.
The license texts are in `LICENSES/`.

[mise]: https://mise.jdx.dev/
[repository]: https://github.com/Aquaticat/Monochromatic

## The commit service

A scoped commit (`git commit -m <message> -- <paths>`, with the message, identity, sign-off, trailer, empty-commit,
reused-message, fixup and `-a` options listed in `doc/planning/cli-git-resident-service.md`) is landed by a resident commit
service instead of by the wrapper itself. The service is a separate program, `cli-git-service`, that the operating system
starts on demand; the wrapper never starts it. Commits queue in arrival order, a batch of whatever is queued lands with one
durable write, and a commit that waits says how many commits go first. There is nothing to install and no setting.

- The first commit that needs the service sets up its socket activation itself, and every later one checks it is still
  right: it compares the two installed units with what this release would write and, only when they differ (an upgrade or
  a downgrade moved the service program), rewrites them, reloads the manager, clears failed state and starts the socket.
  When they match it opens no connection to the bus. Setting up writes two unit files into
  `$XDG_RUNTIME_DIR/systemd/user` (nothing is left in the home directory; the next first use after a login does it again)
  and asks the user's service manager, through the session bus, to start the socket. The manager starts the service when
  the first connection arrives. Units that an earlier release wrote into `~/.config/systemd/user` are recognized by their
  content and removed; a file of the same name that something else wrote is never touched, and the commit stops and names it.
- The single service of the releases before one service per worktree is stopped before the first instance starts, wherever
  its unit files are (including `~/.config/systemd/user`); the commit waits for it to be inactive, and fails naming it when
  it will not stop. Limit: going back to one of those releases starts its single service beside the per-worktree instances
  until their idle hours pass; stop `cli-git@*.service` and `cli-git@*.socket` with `systemctl --user stop` before committing
  with the older release (`doc/planning/cli-git-resident-service.md`, "Limit: a downgrade").
- A failed backup push of a linked worktree, and a push the service had not run yet, survive `git worktree remove` and
  `git worktree prune`: the wrapper copies them first, and the next command in any worktree shows them once. The copy is
  kept while the command that took it may still be removing, and until the worktree's checkout is seen intact (Git
  refused) or its Git directory is gone (Git removed it); a record that cannot be copied or kept is printed on standard
  error, and what could be copied stays. Limit: a worktree that Git's own `gc` prunes is not seen by the wrapper, so its
  records are lost with it.
- The service keeps the records of its newest 1024 finished batches in its journal and removes older ones when it starts;
  a request is answered as already landed only while its own client retries it, which is minutes at most.
- The service exits after three hours with nothing to do (Bazel's server has the same default); the next commit starts it
  again. A service started from another release of the wrapper is asked to finish its batch and exit, and the next commit
  reaches the current one; if it does not exit, the commit says so and names `systemctl --user restart cli-git.service`.
- `git cli-git service status` says whether the service answers; it is a diagnostic only.
- When the service cannot be set up (no session bus, no user service manager), the commit stops with a plain message that
  says what to do, and changes nothing: the service is the only way a commit lands, so there is no other path. Every command
  line the service does not build from the command line itself (amend with an editor, an editor, signing, templates, a
  repository whose configuration sets a template, a clean-up mode, signing or trailers) is built by real Git in a scratch
  repository and landed by the service as Git made it.
- The backup push and the automatic upkeep of a commit the service landed run in the service after the command returned.
  A push that fails does not fail the commit; the next commit through the service prints a notice once. The service pushes
  with its own environment, so a variable only your terminal has (an ssh agent address) must also be in the user manager's
  environment.
- Where a worktree holds the tracked file `doc/audit/standards-review-gate-shadow.md`, the service also runs a standards
  review of each commit it landed, after the command returned, and appends one entry to the repository's review log,
  `<common Git directory>/cli-git/standards-review-shadow.md`. That log is untracked and inside the Git directory, so no
  checkout shows it as a change; there is one per repository, shared by every worktree's service, and the service creates it
  when missing. Each entry is written in one append, so entries of concurrent instances never interleave, and no lock is
  taken. The tracked file only switches the review on and is never written by the service. The log has no rotation.
  `SHADOW_REVIEW_LOG` points it at another file; the runbook is `doc/runbook/standards-review-gate.md`.

The design, the measurements and the migration plan are in `doc/planning/cli-git-resident-service.md`.
