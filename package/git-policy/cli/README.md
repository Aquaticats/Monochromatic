# monochromatic-git-policy-cli

A Git wrapper that enforces repository-owned policies.
It is installed as `git` ahead of the real Git on `PATH`,
forwards every command to the real Git,
and checks the commands and the commits that pass through it against policies the repository configures as data.

The executable is `cli-git-native`.
The library inside the crate is named `git_policy_cli`.

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
- Commits as transactions.
  A `git commit` is prepared in a private index and a shadow repository.
  Hooks run there,
   the content policies check the exact bytes,
   and only then is the commit landed on the real index and
  branch.
  When another commit lands first,
   the wrapper replays its own commit on top of the new state.
  Concurrent commits from several worktrees or sessions therefore never overwrite each other.
  After an explicit-path commit lands, the wrapper compares the real index before and after the landing.
  Any staged entry outside the committed paths that differs is named in a warning,
   so a loss of staged work is reported instead of silent.
  A transaction that an interrupted process left behind is recovered by the next invocation.
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
  Only hooks configured through Git config (`hook.<name>.event` with `hook.<name>.command`) are supported.
  Traditional hooks, executable scripts in `.git/hooks` or in a `core.hooksPath` directory, are not supported,
   and they are never run through the wrapper:
   every real Git it starts, forwarded commands and its own children alike, runs with
   `-c core.hooksPath=/dev/null/cli-git-no-hooks` first, a directory that cannot exist.
  Configured hooks still run, and a configured `pre-push` hook runs once per push that publishes:
   the wrapper's own dry-run probes switch configured `pre-push` hooks off with `-c hook.pre-push.enabled=false`.
  A `-c core.hooksPath=<directory>` you write after the subcommand's global options wins for that command.
  A script that starts `/usr/bin/git` directly, outside any Git the wrapper started, still runs them.
  A commit in a repository that sets `core.hooksPath` prints one non-blocking `configuration-warning` event
   (code `traditional-hooks-unsupported`) that says so; `git cli-git doctor` lists the scripts.
  The shims `git lfs install` writes are the expected case: see "Git LFS" for what replaces them.
- Git LFS.
  Every push the wrapper starts uploads the LFS objects it publishes pointers for, before any ref moves,
   in a repository whose own Git configuration (local, worktree or `-c`; not the global file) has a `filter.lfs.*` or
   `lfs.*` key.
  The step is Git's own `pre-push` hook, put in effect by the wrapper for that one `git push` with
   `-c hook.cli-git-lfs-pre-push.event=pre-push` and a command that runs `git-lfs pre-push`;
   Git hands it the remote, the URL and the ref-update lines, and a failed upload stops the push.
  It covers a typed `git push` (including the merge push of a lost race), the auto-push after a commit and its
   reconcile pushes.
  `--dry-run` pushes and the wrapper's own probes never upload.
  Without `git-lfs` on `PATH` the push stops and the message says to install it, remove the LFS configuration
   or use `git push --no-verify`.
  The other three hooks `git lfs install` writes (`post-checkout`, `post-commit`, `post-merge`) only keep lockable
   files read-only, do nothing without the `lockable` attribute, and are not run.
  See the SPEC section "Git LFS upload before a push" for the evidence and the limits.
- Content policies.
  `final-newline`,
   `markdown/autofix`,
   `mono/forbidden-root-context`,
   `mono/dependent-version-bump` and
  `security/forbidden-strings` check or rewrite what a commit contains.
  The same content policies scan what a manual `git push` newly publishes.
- After a landed commit.
  The wrapper checks the committed bytes again,
   then pushes the branch.
  A branch without an upstream is pushed to `origin` with `--set-upstream` when that remote exists.
  A branch with an upstream is pushed to it,
   including a branch whose upstream has another name, such as a linked worktree branch created from `origin/main`:
   Git itself refuses that push under its default `push.default=simple`, so the wrapper names the tracked branch.
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
   runs after each landing,
  because a landing stores its objects in one new pack.
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
  under one settlement lock per repository.
- Signals.
  The wrapper catches `SIGINT`,
   `SIGTERM`,
   `SIGHUP` and `SIGQUIT` while it waits for Git.
  A signal sent by another process is relayed to the Git child,
   and the wrapper then releases its locks and removes
  its transaction instead of leaving it for recovery.

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
- `monochromatic-lint` on an absolute `PATH` entry,
   only when `markdown/autofix` is configured.
- `git-lfs` on `PATH`, only in a repository whose own Git configuration has `filter.lfs.*` or `lfs.*` keys.
- The scanner is linked into the executable and needs no separate installation.

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
 including its recovery of interrupted commits and its policies.

## Commands

The wrapper adds one namespace to Git.

```sh
# Check or fix the whole repository, or the named pathspecs.
git cli-git check --all
git cli-git fix -- docs/ README.md
git cli-git check --all --policy final-newline
# List the traditional hooks the repository holds; they are unsupported.
git cli-git doctor
git cli-git --help
```

`git cli-git lfs-pre-push <remote> <url>` is the entry Git starts for the Git LFS upload described under "Git LFS";
it is not meant to be typed.
It reads the ref-update lines on standard input, runs `git-lfs pre-push` with them and returns its status.

Exactly one scope is required:
 `--all`,
 or pathspecs after `--`.
`--policy <id>` limits the run to one policy and may be repeated.
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
 so the commands that need a hook to keep working should become config hooks
 (`hook.<name>.event` with `hook.<name>.command`).
A commit does not scan for them: it only warns when the configuration sets `core.hooksPath`,
 which it already knows without any extra file system access.

The former `trust`,
 `untrust` and `status` commands are retired.
Configuration is data,
 so nothing needs approval.
They print an explanation and change nothing.

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
   transaction or engine failure.
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
    "security/forbidden-strings": ["error", { "builtinRules": true, "rulesFile": ".cache/forbidden-strings.rules.txt" }]
  },
  "hooks": { "concurrentCommits": false },
  "indexLock": { "unprovenOwnerTimeoutMs": 1000 },
  "landing": { "reserveAfterLostRaces": 1 }
}
```

An unknown key,
 a wrong type or an unknown policy id is a configuration failure that exits `2` before any policy runs.

### Policies

The wrapper ships these policies and no others.
The first five run at their default severity without being listed.
The last four run only where the file lists them.

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
  Runs `monochromatic-lint --fix` over staged Markdown inside the commit.
  Options:
   `rules` (only `lfs-image-url` exists) and `exclude` (gitignore-style patterns).
- `mono/forbidden-root-context`:
   default `error`.
  No `CONTEXT.md` at the repository top level enters a commit.
- `mono/dependent-version-bump`:
   default `error`.
  Packages that depend on a package with a bumped version follow the bump.
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
{ "rulesFile": ".cache/forbidden-strings.rules.txt" }
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
   boolean,
   default `false`.
  `false` serializes the hooks of concurrent commit transactions;
   `true` lets them overlap.
- `indexLock.unprovenOwnerTimeoutMs`:
   non-negative integer,
   default `1000`.
  How long the wrapper backs off for an `index.lock` whose owner is dead or unproven.
- `landing.reserveAfterLostRaces`:
   positive integer,
   default `1`.
  How many lost landing races pass before a transaction asks for the landing reservation.

The concurrent transaction itself cannot be turned off.

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
   the capture,
   landing,
   hook and registry locks,
   the landing reservation,
   the worktree-copy settlement and a live `index.lock` holder.
  At the limit it stops with a message that names the holder and every way forward:
   let the holder finish and run the command again,
   check with `ps` whether it is stuck,
   or raise the limit.
  The command leaves the lock in place and changes nothing else,
   so it can simply be run again.
- `CLI_GIT_CHILD_WAIT_SECONDS`,
   default `1800`.
  How long one Git process that the wrapper starts for itself may run before it is stopped:
   the object reader (`git cat-file --batch`) that the policy checks read candidate files through,
   and the Git steps of a commit in the shadow repository.
  A process that crosses the limit is stopped,
   and the command reports which one and how to go on.
- `CLI_GIT_WAIT_NOTICE_SECONDS`,
   default `10`.
  How often a waiting command repeats its notice (see "What a waiting command prints").
- `CLI_GIT_PUSH_DEADLINE_SECONDS`,
   default `600`.
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
 and the user's own hooks,
 run as long as they run and receive Ctrl-C like any Git command.

#### What a waiting command prints

Every wait before the push stage of a commit writes to standard error,
so a queued commit can be told from a hung one.
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

- A lock held by a live process (the landing, hook, capture and registry locks),
   as in the sample below.
  The first line of a wait adds a hint,
   and the repeats stay short.
- The landing reservation that a commit asks for after it lost landing races:
  the same line for the holder,
   and, when several commits queue for the grant,
   `This is number 2 of 3 commits in line, so 1 will be served first.`
- An `index.lock` held by a live process proven by an open descriptor or Git's PID file,
   with the proof named,
   and an `index.lock` that nothing proves held,
   which the wrapper retries on Git's backoff until `indexLock.unprovenOwnerTimeoutMs` has passed.
- The worktree-copy settlement lock and another process's push of the same branch.
- A long run of `monochromatic-lint` for a Markdown file,
   which names the file and stops the linter after 60 seconds.
- The object reader and the Git steps of a commit that run long,
   which name the Git command and the process,
   and stop it after `CLI_GIT_CHILD_WAIT_SECONDS`.

```text
// standard error of a commit that waits for the landing lock
cli-git: still waiting for process 4242 to release the landing lock (/repo/.git/cli-git-transactions/landing.lock) after 12 seconds; giving up after 1800 seconds (CLI_GIT_LOCK_WAIT_SECONDS changes this), leaving the lock in place.
```

## Operating conditions

This package is one of the few that must keep working under the conditions its host lives in every day,
not only on a quiet machine.
The wrapper, its recovery paths and its tests are expected to work under these conditions:

- high CPU load from other sessions and builds;
- memory pressure and heavy swap use;
- slow SSD and other I/O trouble;
- a nearly full disk, so a write can fail with no space left in the middle of a landing;
- process and descriptor limits close to exhaustion;
- a process killed at any instruction, which the journals and recovery exist for;
- a closed terminal or a broken pipe;
- slow fsync and unusual filesystems;
- DNS or mirror trouble, where the wrapper recovers or falls back instead of wedging.

### What pins each condition

Each condition has permanent controls that start the built executable on real Git fixtures.
The fault controls (`src/native/binary_fault_*_tests.rs`, method in `binary_fault_support.rs`) inject one fault at the
`n`th call of one system call of an explicit-path commit, for every `n`, with `strace --inject` (a `ptrace` of the
child, no privilege and no load; the wrapper's own thread alone is traced, so the `n`th call is the same call on every
run) or with a resource limit set on the child between `fork` and `exec`.
After each fault the next ordinary command must succeed without anyone repairing anything, the commit must be landed
with a clean index or not made with its file still staged, `git fsck` must pass, a further commit must land, and
nothing of the wrapper may stay behind.
The `strace` sweeps report a skip and check nothing on a host without `strace`, as `binary_fork_failure_tests.rs` does,
so the host suite is where they run.
What the faults found, and the change for each, is in
[the fault injection note](../../../doc/troubleshooting/git-wrapper-fault-injection-findings.md).

- A nearly full disk: `binary_fault_enospc_tests.rs` fails every `write`, directory creation, rename, symbolic link,
  hard link of the wrapper with `ENOSPC` (and every `write` of Git too), and every flush with `EIO`;
  `binary_fault_limits_tests.rs` caps the file size so that a write stops half way and fails with `EFBIG`.
  A cap cannot reach the records after the first, because the first record is the largest, so the sweep of calls is
  the control for those.
- A process killed at any instruction: `binary_fault_kill_tests.rs` kills the wrapper with `SIGKILL` before every
  `write`, flush, rename, directory creation and removal of a commit (and the wrapper or a Git process before every
  `write` and flush); older controls kill at the phase markers (`binary_landing_crash_tests.rs`,
  `binary_merge_isolation_tests.rs`, `binary_worktree_copy_tests.rs`) and damage the records a kill leaves
  (`binary_recovery_tearing_tests.rs`).
  A staging directory or a retired lock copy (`*.pending`, `*.stale`) of a killed process stays on purpose and is
  ignored by every reader.
- Descriptor and process limits: `binary_fault_limits_tests.rs` sets `RLIMIT_NOFILE` from 4 to 48, fails every
  `openat` and `pipe2` with `EMFILE`, and keeps `clone3` failing with `EAGAIN` from sampled starts on;
  `binary_fork_failure_tests.rs` fails a short run of starts.
  `RLIMIT_NPROC` counts the user's processes on the whole host, so it cannot be swept and is not used.
- A closed terminal or a broken pipe: `binary_fault_signal_tests.rs` sends `SIGHUP`, `SIGINT` and `SIGQUIT` to the
  wrapper while a hook runs and `SIGHUP` to the whole process group at each phase of the transaction;
  `binary_fault_stream_tests.rs` closes the standard streams, gives them a pipe with no reader or a full device, and
  requires that the status matches the one Git gives when started directly.
- Slow fsync and unusual filesystems: `binary_fault_filesystem_tests.rs` delays every flush by 30 ms, and keeps hard
  links, symbolic links and renames across devices refused and directory creation read-only from the `n`th call on.
  Directory sync on a filesystem without it is a unit control on procfs
  (`private_storage::tests::a_directory_that_cannot_be_synced_is_not_an_error`).
  Not injected: real slow storage and real filesystems with these properties, which need privilege to mount.
- DNS or mirror trouble: `binary_fault_remote_tests.rs` makes the push after a landed commit and an explicit push meet
  a refused connection, a name under `.invalid`, a server that hangs up, a server that never answers (stopped at
  `CLI_GIT_PUSH_DEADLINE_SECONDS`) and a `git-lfs` that never exits, and requires that the commit stays, the failure is
  worded, no lock is held and no process the push started is left running; `binary_push_environment_tests.rs`,
  `binary_lfs_pre_push_tests.rs` and `binary_lfs_server_tests.rs` cover the stalled push and the failing upload.
- High CPU load, memory pressure and slow I/O: the shared bound and the byte copy of the wrapper
  (the troubleshooting note below); no control injects memory pressure.
  `RLIMIT_AS` is the only privilege-free candidate and was not tried: the address-space cap it sets is hit by Rust and
  Git allocations alike at points that depend on the build, so it is not expected to tell a wrapper defect from an
  allocator abort.

Known gaps: a user hook that never exits is not bounded by `CLI_GIT_CHILD_WAIT_SECONDS` (hooks run as long as they run,
as documented, and are ended by the signals above); the worktree-copy and merge flows are killed at their phase markers
but not swept call by call; the Git steps of a commit that the wrapper bounds are killed without the processes below
them.

Tests therefore never wait for a quiet host.
The shared 5-second bound on one wrapped command is deliberate;
its measurements and the evidence needed to change it are in
[the troubleshooting note](../../../doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md).

## State on disk

The wrapper keeps its state beside the repository's Git data under `cli-git*` paths in the common Git directory.
Delete nothing there while a Git command is running.
The next invocation recovers what an interrupted one left.

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
- `native:perf` builds the release executable and measures the latency it adds to forwarded Git commands and to
  explicit-path commits, with paired direct-Git samples in disposable repositories.
  It reports the median, p95 and median absolute deviation of both sides and the added delta,
  fails when a forwarded command's added p95 reaches the 2,000 ms bound or any sample fails,
  and `-- --output <file>` stores the raw samples (baselines live in `perf/`).
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

## License

`LGPL-3.0-or-later`.
The license texts are in `LICENSES/`.

[mise]: https://mise.jdx.dev/
[repository]: https://github.com/Aquaticat/Monochromatic
