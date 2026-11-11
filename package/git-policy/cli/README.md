# monochromatic-git-policy-cli

EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE

A Git wrapper that enforces repository-owned policies.
It is installed as `git` ahead of the real Git on `PATH`,
forwards every command to the real Git,
and checks the commands and the commits that pass through it against policies the repository configures as data.

The executable is `monochromatic-git-policy-cli`.
The library inside the crate is named `git_policy_cli`.

How the owner wants this package designed and judged is in [DESIGN.md](DESIGN.md).

## Names before the rename of 2026-11-04T22:23:58-05:00

The owner renamed the program,
and the rename is recorded in `doc/planning/git-policy-cli-program-rename.md`.
The program was `cli-git-native` and is `monochromatic-git-policy-cli`,
the crate's name.
Every earlier name keeps working in this release:

- The command word.
  `git cli-git <command>` is answered like `git monochromatic-git-policy-cli <command>`,
   with one line on standard error before the output that names the new spelling.
  A unit file that an older program wrote starts the commit service with the earlier word,
  and the service answers it.
- The settings file.
  `monochromatic-git-policy-cli.config.jsonc` is read when it exists.
  A repository that has only `cli-git.config.jsonc` is read from it,
  with one line on standard error that names the new file;
   the wrapper never renames a committed file.
  When both exist,
  the new file is read,
  and one line says the earlier one is ignored.
- The folders and files the wrapper keeps in the Git directory (the data folder with saved copies,
  notices and work records,
   the discard audit log with its numbered older copies,
  the commit timing,
  worktree-add timing and push trigger logs,
   the merge records and the ignored-state copy records).
  The first command that needs one after the rename finds only the earlier name,
  and renames it in place with one atomic
   rename that never replaces;
  nothing is copied,
  nothing is left behind,
  and a process that loses a race finds the result.
  When both names exist (a development build of the renamed wrapper and an earlier release ran side by side),
  the new one is used
   and the earlier one is folded into it,
  losslessly,
  by the first command that finds no earlier program running:
   a log gets the earlier log's bytes added after its own lines,
  in the earlier log's order,
  and the earlier log is removed;
   a folder gets every entry of the earlier folder (folders that exist in both are merged one level at a time),
  and an entry
   whose name exists in both is kept twice,
  the earlier one as `<name>.from-earlier-release`.
  Nothing prints after a fold.
  The fold waits while a process has the start arguments of the earlier commit service of the Git directory's worktree
   (a socket that only listens does not hold it),
  says one line per process what to do in plain words,
  and folds on a later command.
  A kill at any instruction leaves each byte under one of the two names,
  and the next command finishes the fold.
  A program of an earlier release that opened a log before the fold moved it may still write a line to it,
  so the fold removes
   the moved log only when no process on the machine has it open for writing (a read lease that is taken and given back at once
   answers that,
  for any user and any namespace,
  and takes nothing from anyone).
  While one does,
  the fold leaves the moved log and a journal beside the current log,
  says nothing,
  and each later command adds
   only the lines written since and tries again,
  until that program has closed the log;
  no line is lost or added twice.
   Each such command also looks at the running processes once (about 10 ms) while the moved log waits.
  Logs are not interleaved by time stamp:
  the current log is never rewritten,
  because other programs append to it without a lock.
  If a rename fails (a read-only Git directory),
  the earlier name is used for that run and the next command tries again.
- The commit service units.
  The units this release writes are `monochromatic-git-policy-cli@.socket` and `monochromatic-git-policy-cli@.service`,
   and the socket sits under `$XDG_RUNTIME_DIR/monochromatic-git-policy-cli/`.
  A service that an older program started under `cli-git@<instance>` keeps running from its own install folder
   and lands the commits that older programs send it;
  this release neither stops it nor removes the old units.

These names are not renamed,
because an older program or an interrupted run may still need them:
 `--cli-git-keep-going`,
the Git configuration keys `hook.cli-git-lfs-pre-push.enabled` and `user.cli-git-test`,
 the reference names under `refs/cli-git/` and `refs/worktree/cli-git-service/`,
 the temporary file and folder prefixes that start with `.cli-git-` or `cli-git-`,
 the index lock marker `cli-git-service`,
the conversion and migration journals,
 the trash folder `.claude/cli-git-trash`,
and thread names.
The link folder `.cache/cli-git-bin` is `.cache/monochromatic-git-policy-cli-bin` from the release that carries the new name,
and the toolchain lock names it.

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
  `add-explicit` refuses a `git add` whose pathspec selects every path of the repository or of the
   directory it runs in,
   however that pathspec is spelled (`git add .`,
  `git add *`,
  `git add .//`,
  `git add sub/..`),
   and says how many paths it selected.
  A `git add` that names its paths starts no process for this.
- Commits landed by a service.
  A `git commit` is prepared in a private index,
   the content policies check the exact bytes,
   and only then does the commit service of the worktree land it on the real index and branch
   by compare-and-swap.
  That private index is a copy of the real index,
  so the `git commit` built from it reads no file whose entry it copied;
   an intent-to-add entry copies as it is and carries no stat data to lose,
   and an assume-unchanged or skip-worktree entry sends the build to the preparation base instead
   (`SPEC.md`,
  "Explicit-path commit").
  The service is started by the operating system on the first commit (socket activation);
   the wrapper never starts it,
   and on a machine where no service can be reached or started a commit stops with a plain message and changes nothing.
  When another commit lands first,
   the wrapper prepares its own commit again on top of the new state.
  The commit that concludes a merge,
   a cherry-pick or a revert is made again the same way on the branch as it is now,
   with the parents its state names,
   and an amend whose branch moved becomes a new commit on top of the branch,
  with the amended message and authorship,
   leaving the commit it named and the commit that moved the branch as they are,
  and a line says what it became
   (`SPEC.md`,
  "Retry after a moved branch").
  Concurrent commits from several worktrees or sessions therefore never overwrite each other,
   and none waits for another.
  The wrapper takes no lock of its own:
  a commit meets only Git's `index.lock`,
   which the service holds for the branch move and the index build only (a median of 5 ms in the measurement in `SPEC.md`).
  A commit the service was landing when it died is settled by the service from its journal when it starts again.
  A stale `index.lock` that a killed service left (its marker names a process that is gone) is removed by one reclaimer at a time,
   on every file system:
  reclaimers take their place in a chain of numbered claim files (`cli-reclaim-*`) beside the lock and check that
   the name still refers to the file they judged before they unlink,
  so no `flock` is involved.
  A killed reclaimer can leave inert
   claim files,
  which `SPEC.md` ("The claim chain") says how to remove.
  On a file system that supports neither a rename that does not replace nor a hard link (a FUSE server such as `s3fs-fuse`),
  the service
   takes `index.lock` the way Git does,
  by exclusive creation,
  and writes its marker into the file afterwards,
  so a commit lands there as
   it does with plain Git.
  A kill between the two steps leaves a lock without its marker,
  as plain Git leaves one:
  the wrapper never removes
   it (it may be a Git that is still writing),
  a wrapped `git add` meets it as Git does,
  and a commit waits for it until the lock ceiling and
   then says which file it is and the command that removes it (`SPEC.md`,
  "Placement and its refusals").
  A killed service's lock with its whole
   marker is removed by the next landing only where a claim file can be placed;
  on such a file system the notice names the file to remove.
  A marker carries the machine ID of its host:
  a reboot of the same machine ends the holder,
  and a lock or claim of a service on another
   machine (a repository on a shared file system) is never taken for stale.
  Neither is a marker of an earlier release,
  which names no
   machine and is reclaimed by that release's own exclusion:
  such a lock,
  or a claim that cannot be judged,
  stays,
  the commit waits as for
   any held lock,
  and the notice names the file and gives `rm -- '<path>'` for a person who knows the service is gone.
  A stale lock that a
   service of an earlier release left is removed by hand once after the upgrade;
  every lock this release writes reads
   `cli-git-service v2 ...`,
  which an earlier release leaves alone.
  A lock that cannot be proven stale is never removed (a running Git's lock,
  or one held by a process of another PID namespace),
  with one exception the owner chose (2026-11-07,
  "Service removes it"):
  an empty `index.lock` with no marker,
  no Git lock PID file beside it and no process that a complete scan of every readable process finds with it open
  is removed by the service that waits for it,
  and the commit lands.
  The scan runs on Linux only,
  and its bound is `SERVICE_SCAN_BUDGET`;
  a scan that did not finish removes nothing.
  The removal is journaled with the lock's device,
  inode and size,
  and the person is told in the waiting notices.
  Limit:
  a process in another user namespace,
  or one the scan may not read,
  cannot be seen.
  A lock with content is left,
  because a native `git commit --all`,
  `--only` or with paths keeps `index.lock` with the index written into it through its editor,
  with no descriptor open and no PID file unless `core.lockfilePid` is on
  (measured with Git 2.56.0,
  `src/native/landing/service_index_unheld_removal.rs`);
  widening the removal to such locks is the owner's to decide.
- A record from another PID namespace that is older than two minutes (`FOREIGN_OWNER_PATIENCE`) is never taken over (the owner,
  2026-11-04:
  "Refuse and name it (Recommended)").
  The owner chose the two minutes (2026-11-10:
  "Two minutes (Recommended)"),
  which is twice the one minute a merge waits for another program's `index.lock`;
  the record's age is
   the whole hold of its owner,
  so a hold that runs longer is refused as this paragraph says instead of being waited for.
  A merge recovery,
  a conversion claim or a worktree add that finds one stops with an
   error that names the record file,
  its owner's process number and namespace,
  its age,
  that the owner may be a live command in
   another container,
  and the exact `rm -- '<path>'` to run when no monochromatic-git-policy-cli command and no Git that monochromatic-git-policy-cli left running is working
   on the repository in any container;
  it changes nothing.
  Commits are outside this and never fail on it.
- Merges that do not fail (the owner's rules of 2026-11-03:
  "Auto merging should never fail."
  and
   "Manual merge should never fail too.").
  Native Git's merge stashes first and dies when another process holds `index.lock`,
   changes the checkout before it moves the branch,
  and refuses every `git merge` that creates a commit
   while the index holds any staged change,
  even when the merge touches none of the staged files.
  The wrapper waits for the lock,
  lands the merge on a branch that moved meanwhile,
   and computes the merge from Git objects,
   commits it with the parents,
  message and signing native Git would use,
   and lets Git's own fast-forward update move the checkout to it.
  Every staged entry,
  unstaged edit and untracked file that the merge does not change stays byte for byte as it was.
  A merge that changes a path someone else has staged or modified is completed,
  and that work is carried across it (see "Carrying the person's own work").
  A merge that conflicts over that work is refused,
  with the conflicting paths named and the ways forward,
  only when no resolution settles every one of them;
  over a clean checkout a conflict is native Git's,
  which stops and leaves its markers in the worktree.
  Shapes the wrapper cannot make exact (`--squash`,
  `--no-commit`,
  `-s`,
  `-X`,
  several branches,
  an editor) run in
   native Git unchanged;
  see "Merging" for the full list.
- Package-manager state in a new worktree.
  After `git worktree add` has filled a worktree and copied its ignored files,
   the wrapper repairs that state at its new location:
   a clone of the source's external Cargo build directory followed by `cargo clean --package <member>` for each workspace member,
   and,
   where the wrapper can reach a commit service,
   one record that hands the two tools rewriting link layouts in place to that service:
   `pnpm install --offline --frozen-lockfile` for the copied `node_modules`
   and `mpm-shadow install --frozen-lockfile` for toolchain launchers that still name another checkout's lock
   run there after the add returns.
  Where no commit service can be reached,
   the add runs those two tools itself,
   because a worktree whose copied links were never relinked holds the source's links for your next command there.
  A repair never fails the add:
  when a tool is missing,
  offline data is incomplete or a lock disagrees,
   one line names the command to run (see "Repairing package-manager state in a new worktree").
- Hooks.
  monochromatic-git-policy-cli supports no external hook program:
  not a traditional hook (an executable script in `.git/hooks` or in a
   `core.hooksPath` directory) and not a hook configured with `hook.<name>.event` and `hook.<name>.command`.
  Every real Git the wrapper starts,
  forwarded commands and its own children alike,
  runs with
   `-c core.hooksPath=/dev/null/cli-git-no-hooks`,
  a directory that cannot exist,
  and `-c hook.<event>.enabled=false`
   for every hook event Git knows,
  after your global options and before the subcommand,
  so no `-c` of yours brings a
   hook back.
  One kind of start goes without them:
  a `git config` that only reports or changes configuration,
  because it must report what the repository configures,
  and the wrapper reads the repository's `core.hooksPath` that way for the commit warning and for `doctor`
   (`SPEC.md`,
  "External hooks").
  `git config --edit` starts an editor,
  and the Git that editor starts inherits the switches,
  so a Git edited in runs with hooks off too.
  The wrapper injects no hook of its own,
  so `pre-push` is switched off like every other event.
  A script that starts `/usr/bin/git` directly,
  outside any Git the wrapper started,
  still runs hooks.
  The wrapper runs no program that any configuration names:
  what a hook checked is a monochromatic-git-policy-cli policy under `policies`
   in `monochromatic-git-policy-cli.config.jsonc`,
  and there is no setting that brings a hook back.
  A `postCommit` key in `monochromatic-git-policy-cli.config.jsonc` is a configuration error that says so.
  A commit in a repository that configures hooks prints one non-blocking `configuration-warning` event
   (code `external-hooks-unsupported`) naming the keys,
  the events and the ways forward,
   and `git monochromatic-git-policy-cli doctor` lists the same hooks.
  A commit in a repository that sets `core.hooksPath` prints one such event too (code `traditional-hooks-unsupported`).
  The shims `git lfs install` writes are the expected case:
  see "Git LFS" for what replaces them.
- Git LFS.
  Every push the wrapper starts uploads the LFS objects it publishes pointers for,
  before any ref moves,
   in a repository whose own Git configuration (local,
  worktree or `-c`;
  not the global file) has a `filter.lfs.*` or
   `lfs.*` key.
  When the pushed commits add pointers the step is `git-lfs push`,
  which the wrapper runs itself as a child before it starts the push;
   the push carries no hook,
  the wrapper never starts itself and writes no shell text.
  The remote and refs come from the push's arguments (everything for `--all`,
  `--tags`,
  `--mirror` and glob
   refspecs;
  the branch Git would send for a bare push),
  and a failed upload stops the push before it starts.
  The wrapper,
  as the parent that ran the upload,
  prints the explanation.
  The wrapper first reads the pushed commits itself,
  with its in-process readers,
  and finds the pointer files they add
   (compared with each parent,
  so a pointer added and removed again inside the push still counts).
  A push that adds no pointer starts no `git-lfs` and sends no request to the LFS server;
  a push that adds some runs
   `git-lfs push --object-id <remote> <oid>...` for exactly those objects.
  The remote's tracking refs say what the destination already has only for a configured remote that is fetched from
   and pushed to one place;
  a push URL,
  a `pushInsteadOf` rewrite,
  a URL or path destination,
  a mirror push or an
   unconfigured remote makes the wrapper read the whole history of the pushed refs instead.
  A repository the readers decline,
  or an object missing from `.git/lfs`,
  falls back to `git-lfs push <remote> <refs>`
   (with `--all` in the untrusted cases),
  which scans itself.
  When `git-lfs push` fails,
  the wrapper does not run it again,
  whatever the cause.
  A server error (an HTTP 5xx answer other than 501,
  507 and 509,
  which `git-lfs` reports on a line starting
   `Fatal error: `) is outside our control,
  like a network that is down.
  That is pick 12 of `doc/planning/cli-git-shipped-limits-audit.md`,
  made by the coordinator under the owner's delegation
   and open to the owner's veto.
  The commit service's push retry plan (`CLI_GIT_PUSH_RETRY_SECONDS`,
  doubling to 600 seconds) tries the auto-push again
   by itself,
  and a typed `git push` ends at the first server error,
  with `git-lfs`'s own words,
  so that you push again.
  The wrapper reads the standard error of `git-lfs` through a private temporary file and relays it with the user and
   password of every URL replaced by `<credentials>`.
  It covers a typed `git push` (including the merge push of a lost race),
  the auto-push after a commit and its
   reconcile pushes.
  The typed push's upload is bounded like the auto-push's:
  a program that shows no sign of work for the push's stall window
   (`CLI_GIT_PUSH_DEADLINE_SECONDS`,
  30 seconds by default) is stopped,
  everything it started is ended,
  and the push
   says the upload "made no progress for N seconds and was stopped" instead of waiting for an answer that never comes;
  a program whose connection to the server is open is waited for while it works,
  and one that keeps it open with nothing moving for the connected window (150 seconds)
   is stopped,
  with `git lfs push --all <remote>` named as its way forward.
  The plan's own Git questions are bounded the same
   way,
  so a Git that never answers is stopped and named.
  A push whose process has no controlling terminal uploads with `GIT_ASKPASS` set to the empty string and
   `GIT_TERMINAL_PROMPT=0`,
  so a login it does not have fails at once instead of opening a desktop password window for nobody
   to answer,
  and the refusal names the server and how to store a login in a credential helper (see "Git commands that may need a
   login").
  A push typed at a terminal keeps Git's and `git-lfs`'s own prompting.
  That refusal says which prompting the wrapper left:
  a push with a terminal is told the wrapper kept Git's and `git-lfs`'s own prompting and names a variable or setting
   that would have refused the login,
  and only a push without one is told the never-prompt pair was added,
  so no person is sent to fix a missing terminal that was there.
  `--dry-run` pushes and the wrapper's own probes never upload.
  `--no-verify`,
  `GIT_LFS_SKIP_PUSH` and `-c hook.cli-git-lfs-pre-push.enabled=false` skip it.
  Without `git-lfs` on `PATH` the push stops before anything starts and the message says to install it,
  remove the
   LFS configuration or use `git push --no-verify`.
  The other three hooks `git lfs install` writes (`post-checkout`,
  `post-commit`,
  `post-merge`) only keep lockable
   files read-only,
  do nothing without the `lockable` attribute,
  and are not run.
  See the SPEC section "Git LFS upload before a push" for the evidence and the limits.
- Content policies.
  `final-newline`,
   `markdown/autofix`,
   `mono/forbidden-root-context`,
   `mono/dependent-version-bump`,
   `mono/catalog-specifiers` and
  `security/forbidden-strings` check or rewrite what a commit contains.
  One of them adds content:
   a hand-raised workspace manifest makes `mono/dependent-version-bump` bump every dependent it reaches
   and put those manifests in the same commit,
   and the worktree copies follow the committed bytes while they still hold what the bump was computed on.
  A dependent whose copy holds anything else is left out of the commit and named on standard error,
   so the commit lands and nobody's work is overwritten.
  The same content policies scan what a manual `git push` newly publishes.
- After a landed commit.
  The wrapper checks the committed bytes again,
   then pushes the branch.
  A branch without an upstream is pushed to `origin` with `--set-upstream` when that remote exists.
  A branch with an upstream is pushed to it,
   including a branch whose upstream has another name,
  such as a linked worktree branch created from `origin/main`:
   Git itself refuses that push under its default `push.default=simple`,
  so the wrapper names the tracked branch.
  The resident service pushes the branch its record names and never reads `HEAD`.
   It sends the branch only where a plain `git push` of that branch would send it:
   a `remote.<name>.push` refspec is applied to the local refs the way Git applies it
   (an explicit refspec before a pattern,
  the first matching pattern,
  a negative refspec last),
   and `push.default` decides otherwise.
   Under `push.default=matching` the service runs the plain `git push <remote>` and lets Git match,
   so every local branch whose name exists on the remote is pushed,
  not only the recorded one,
   and a branch deleted on the remote is not recreated.
   When Git refuses the setting or would push the branch nowhere
   (several `branch.<name>.merge` values under `simple` or `upstream`,
  `push.default=nothing`,
   a triangular `upstream`,
  two refspecs that send different branches to one destination,
   a refspec that names a ref that does not exist,
  a refspec that does not cover the branch),
   nothing is pushed and the note names the branch,
  the setting and the way forward.
  The same push is handed over after a `git merge`,
  a `git pull` that merges,
  a `git cherry-pick`,
  a `git revert` and a `git am`
   that succeeded,
  when the branch tip moved forward and the branch is now ahead of its upstream and not diverged from it.
  The check reads the branch,
  the upstream tracking ref and the commit parents from the files and starts no Git.
  A rebase is not handed over (a rebased branch would need a forced push),
  `git pull --rebase` and `pull.rebase` included,
   and neither is a branch with no upstream,
  a diverged branch,
  a conflict,
  or a command that moved nothing.
   `--continue` of a stopped cherry-pick,
  revert,
  am or merge is the command that adds the commit,
  so it is handed over.
  Every decision,
   handed over or not,
   leaves a line with its reason in `monochromatic-git-policy-cli-push-trigger.jsonl`
   in the common Git directory
   (`"reason":"not-ahead"` and the like),
   so a push that did not happen can be explained from the file.
   A full file moves aside to the next free generation there too,
   and the generations are kept up to 32 MiB together like every other log.
   The file is shared by every worktree,
  so each line names its `worktree`,
  `gitDir` and `instance`,
   and the branch when it is known;
   a failed command's line carries the branch and the tip it was at before the command.
   `level` is `warn` for a step that could not decide or a hand-over that failed and `info` otherwise,
   and the userinfo of a URL in `detail` is masked.
  When the remote refuses that push because its branch moved after the commit landed,
   the wrapper merges the remote tip into the branch tip from Git objects alone and pushes the merge without force,
   at most ten times.
  The merge's message names the branch whose tip is its first parent
  (`Merge origin/main into feature`),
  as `git merge` names the branch it is on,
   and it names the ref the merge is published to when that ref's name differs from the branch's.
  While the merge is computed and pushed,
  the local branch,
  the index,
  the worktree and untracked files are not read
   or changed,
  so another session's uncommitted files cannot block the push.
  Afterwards the local branch follows the published merge by a fast-forward when nothing can be lost:
   a worktree with uncommitted work follows too,
  as long as the move writes none of the files that work touches,
   and the report says the work was left as it is.
   Otherwise the branch stays behind and the report names the overlapping files.
  A conflict that the word-level second merge pass cannot resolve in every file is reported with the conflicting paths
   and publishes nothing;
  when it resolves every conflicted file,
  the merge is published and its commit message and the report
   list each part merged by word (see "Word-level second merge pass").
  An amended or rebased branch is not merged,
  and its rejection is shown as before.
  When a post-commit policy kept one of the branch's commits local,
   the push stops at the commit before the first of them:
  it publishes the newest commit that no such verdict forbids,
  which is the merge base of the blocked commits' parents,
   and it is never reconciled,
  because the merge a reconcile publishes is made above the local tip and would carry the blocked commit.
  The branch is then tried again by itself,
   so the push resumes once the blocked commit is off the branch's way out.
  Every command in that worktree says so in one line while it holds,
   naming the branch,
  the commit and the words of its verdict,
   and the line goes once the commit is off the branch's way out or Git already holds it in the branch's remote.
  The refusal is read from the held record as well when its own verdict record could not be written,
   because the verdict goes into that record first,
   so a commit your gate refused still stops the push.
  A person's own `git push` is unchanged and still sends the branch whole,
   and a commit whose command died before its check ran holds nothing back:
  it has no verdict,
   so the next push of the branch carries it.
  The line such a record prints says exactly that:
  it says the record pushes nothing,
  that the next automatic push of the branch carries the commit if it is not published yet,
  and that a push the person makes sends it the same way.
  A record that names no command, which an earlier release wrote or a batch wrote before it recorded one,
  prints the same shape of line,
  so no line ever claims a commit stays local while nothing stops a push of its branch.
  When Git delivered a push but was refused the update of the branch's remote-tracking reference,
  because another Git held that reference's lock, the wrapper writes the reference itself once the push is over,
  with the reflog message `update by push`, the same bounded retries and the value it read, so a write never replaces
  another writer's value.
  The write names the reference's full name: `git update-ref origin/main <commit>` is accepted and changes nothing
  (measured with Git 2.56.0), so a write from the name a branch's upstream is reported as would leave the reference behind.
  A lock that refuses the write through every retry is not cleared, because nothing may clear another Git's lock
  without a proof that its holder is gone: the push then says that it delivered the commit and that the reference names
  an older commit until a push or a fetch writes it.
  The line about a commit that is not published yet reads the same reference,
  so it never claims a publication state: a reference that lags a remote Git already holds changes nothing about what the line says.
   A commit whose check is still running holds the push back the same way a refusal does,
  because that check may refuse it:
  the push stops before it,
   the line says it waits for those checks,
  and the push resumes by itself when they answer.
   A command a program reads its output from settles the held records and prints no line,
  so an approved push never waits for a command only a person would type.
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
  A conflict is reported with the conflicting paths and publishes nothing,
  after the same second pass.
  The checked-out branch follows the published merge under the same conditions as after a commit;
   a detached HEAD stays where it is,
  and the report says so.
  Forced,
  deleting,
  mirroring,
  tag,
  `--all`,
  `--set-upstream`,
  `--porcelain` and `--dry-run` pushes,
   and pushes of a `+refspec`,
  are never integrated,
   and a failure that is not a divergence (authentication,
  offline,
  a refusing hook) keeps Git's own report.
- Linked worktrees.
  When a Git command creates a linked worktree,
   the wrapper copies the ignored state of the source worktree into it
  without any lock:
  the destination comes from the command line,
  the copy is a transaction of its own whose journal and
  stage claim name the process that owns them,
  and a copy a dead process left is recovered by the next command run in the worktree the copy fills,
  because staging that state costs what the source's ignored tree costs and a command in another worktree gains nothing
  from it;
  such a command names the worktree whose files are missing instead.
  A destination,
  source or stage whose path holds bytes that are not text is named in every record by those bytes,
  so recovery fills and reaps exactly that directory and no other,
  and a record written by an earlier release is still read.
  A stage a killed `git worktree add` left beside its destination is removed by the next command
  that runs the recovery,
  with one line saying what went,
  when the record written beside the stage's claim names the object still standing at that name
  and the folder holds nothing that record cannot account for;
  a stage nothing proves is named,
  left whole,
  and keeps its claim,
  with the reason it was kept,
  so a folder holding something you added there is never removed.
  Concurrent adds in one repository never wait for one another,
  and each worktree receives exactly its own copy.
  A copy settles itself against the source:
  a file the source removed meanwhile has nothing to copy and
   leaves the stage,
  a file that changed is copied again twice at most,
   and a file that keeps changing stays untouched in its source,
  is not installed,
  and is named
   on one line.
  A file that cannot be copied at all fails the copy with its own path in the message.
  A worktree whose own ignored-state copy is still running is no source:
  a second add run from it creates its worktree,
  writes one line saying that the worktree received no ignored state and why,
  and names the commands that have the copy made once the first add has finished.
  The wrapper reads the destination of every form Git's option parser accepts (abbreviated long options resolved as Git
   resolves them,
  bundled short options,
  `--git-dir` and `--work-tree`,
  a destination below directories Git creates);
   a form Git rejects keeps Git's own error,
  and a form the wrapper cannot read is made by Git alone with one line on
   standard error saying the worktree received no copy.
  `git worktree add` in the plain forms (`-b <branch>`,
  `--detach`,
  `--quiet`,
  `--lock`,
  `--reason`,
  one path and at most
   one commit-ish) is served by the first rung of a ladder that applies and otherwise falls to the next;
  when the add
   succeeds without being a snapshot because of a cause the person can change,
  one line on standard error (unless `--quiet`) and one journal entry say why,
  in plain
   words,
  and what makes the next add quick (each step leaves a line in the diagnostic log,
  `MONOCHROMATIC_VERBOSE=true`):
  1. A btrfs snapshot of the worktree the command is run from (the main checkout included) when it is a clean subvolume
     on the destination's filesystem.
     Real Git runs with `--no-checkout`,
     the snapshot is made beside the destination,
     gets the new worktree's `.git`
     pointer and the source's index with refreshed stat fields,
     has the difference between the source's tree and the
     new `HEAD` applied by `git read-tree -m -u`,
     and is swapped into place with `RENAME_EXCHANGE`.
     The snapshot shares every data block with its source and carries the source's whole ignored state
     (`node_modules` included),
     so the ignored-state copy is skipped for it.
     A main checkout with uncommitted changes (staged,
     edited,
     deleted,
     renamed,
     mode-changed or untracked files) is
     snapshotted too:
     in the new worktree only,
     `git read-tree --reset -u` and `git clean --force -d` make it equal to what
     `git worktree add` makes from a clean checkout,
     the result is proven before the worktree is handed over,
     and the
     changes stay in the main checkout,
     which is never written.
     Ignored paths of the source that the new commit does not ignore are carried through the clean and the reset,
     and the proof accepts them:
     the same name,
     a file below a carried directory,
     or a whole directory Git collapsed into one name whose every path below it is carried.
     One path the restore did not carry refuses the rung.
     A failed proof falls to Git's own checkout and says why.
     A merge or other operation in progress,
     a held `index.lock`,
     a skip-worktree or assume-unchanged entry,
     a submodule
     and a changed `.gitattributes` file still send the add the slow way,
     and so does a dirty linked worktree.
     When the source is clean and this repository's commit service runs,
     that snapshot is the one the service made ahead
     (`worktree_snapshot_ahead.rs`):
     the wrapper keeps up to three copies of a worktree in
     `<git-dir>/monochromatic-git-policy-cli/ahead`,
     one add asks for the next one when it takes one,
     and an add
     takes the newest whose files still equal the source's index,
     which is one rename rather than the btrfs call that
     forces a transaction commit of the whole filesystem
     (5,413.0 ms at the median on this volume).
     A copy that fell behind
     the index is refused and handed to the service for deletion,
     and nothing of the source it copied is lost by
     that:
     it is a copy,
     and the source was never written.
     A source with uncommitted changes is never served this way,
     because its
     snapshot carries its own index,
     which is one instant with its files,
     so that snapshot has to be taken on the add's own path.
     When
     no copy comes within 60 s (`SNAPSHOT_AHEAD_CEILING`,
     the time the owner allows an add),
     the rung ends and Git's own
     checkout makes the worktree,
     and one line says so.
     The ignored files the new worktree carries are the source's as of when the service made the
     copy,
     which is what "the moment of the add" means for a worktree served this way:
     a file you write into an
     ignored directory between the copy and the add is not in the new worktree,
     and the source keeps it.
  2. Clones (`FICLONE`) of every tracked file of a clean worktree,
     with its index,
     on btrfs,
     XFS with reflink and
     overlayfs.
     It shares disk but takes about as long as Git's checkout.
  3. Git's own checkout,
     then the ignored-state copy.
  The `git read-tree` of a rung and the fallback `git reset --hard` are Git steps the wrapper starts itself,
     and each has a ceiling of its class (`LADDER_READ_TREE_CEILING`,
     `LADDER_CHECKOUT_CEILING` in `src/native/service/wait_ceilings.rs`):
     a smudge filter,
     a Git LFS process or a filesystem that never answers must not hold the add.
     The person's own command has no ceiling.
     Such a step keeps the process group and the terminal the wrapper has,
     which is what a Git the person starts themselves keeps,
     so a conversion filter that reads `/dev/tty` runs in the foreground and `Ctrl-C` reaches Git as usual.
     When the ceiling is reached the wrapper sends `SIGKILL` to Git and nothing else,
     which is what killing a Git the person started also leaves behind:
     the filters and helpers Git started are not signalled and may still run,
     the add returns,
     and the line says that Git ended (or that it has not exited,
     when it cannot be collected inside `STOP_GRACE`),
     that a helper may still be running,
     and that a caller reading the command's standard error to its end waits for that helper,
     because a parent cannot close a child's inherited stream.
  A source is clean when no operation is in progress,
     its index equals the tree of its `HEAD`,
     every tracked file has
   the size,
     times and inode its index entry recorded,
     and Git lists no untracked file that it does not ignore.
  Rungs 1 and 2 are declined for a sparse checkout,
     a worktree-specific configuration,
     a submodule,
     `.gitattributes`
   files that differ between the source and the new `HEAD`;
     rung 1 from a linked worktree is also declined while a nested linked worktree is
   an ordinary directory (`git monochromatic-git-policy-cli migrate-to-subvolume` or `git monochromatic-git-policy-cli convert-worktree-to-subvolume` makes them
   subvolumes).
     A snapshot of the main checkout does not decline for one:
     it sets those worktrees,
     other checkouts,
     caches
   and run records aside in a trash directory under `.claude/`,
     and the add hands that trash to the commit service and returns
   without waiting for it.
   A directory set aside whole is swept for those containers first,
     at any depth,
     a nested worktree and another checkout among them,
   so nothing of another command's reaches the trash through that route either;
     the sweep reads no file and follows no link,
     and a directory it cannot look inside
   stops the leave-out with that path named instead of counting as one with nothing to sweep.
   Another command's stage container is not put in that trash at all:
     it is removed before the worktree is installed,
     so
   those bytes never reach a worktree,
     not even an ignored trash directory
     (a container of 20,000 one-kilobyte files takes
   about half a second to remove on this host,
     against a tenth of a millisecond to rename).
   A standing trash cannot be staged,
     whatever the target commit ignores:
     before the leave-out asks Git whether the
   trash place is ignored,
     the wrapper adds `/.claude/cli-git-trash/` to the repository's common `info/exclude`,
     which
   no commit writes.
     Measured with Git 2.56.0:
     a linked worktree reads that file and never its own worktree's,
     `git add --all`
   stages nothing of a trash standing there,
     and a target commit that ignores nothing under `.claude/`,
     or that tries to
   re-include the trash parent,
     does not change that.
     A directory of that name outside the trash parent is no trash and is
   never touched.
     When that file cannot be read or written the question falls back to the rules the repository has,
     and a
   `false` answer removes the directories in the foreground instead.
   The record the hand-over writes names the new worktree,
     the common directory it is read from,
     the trash and the
   identities of everything moved into it,
     and it sits under the common directory,
     so one repository's service empties
   its trashes one at a time whichever worktree holds each of them;
     the claim the drain takes is the lease every other
   kind of work already uses,
     so no lock is added.
     Each removal runs at I/O priority class `idle`,
     the lowest Linux offers.
   Unless `--quiet` is given the add prints one line saying how many directories were set aside,
     which worktree
   they were left out of,
     and that the commit service empties them after the command returns.
     A wake that fails is a warning
   that names the worktree,
     the trash and the record,
     which still empties it when the repository's service next runs,
     and
   never the add's failure:
     the worktree is whole either way.
     A path that is not valid UTF-8 cannot be named by a record,
   so the hand-over writes none and says so,
     and only the person removes that trash.
   A work item of kind `trash` that an
   earlier release wrote is emptied by the same handler with the same checks,
     so a repository that ran one keeps its promise,
   and a trash an earlier add left in the source is moved into this add's trash and covered by its record.
   A trash the service cannot empty is reported by the service with its reason and its record settles as failed;
     every
   later command run in that worktree,
     unless its output is read by a program,
     then names each directory in
   `.claude/cli-git-trash` that no pending or claimed record accounts for,
     with its path,
     and deletes nothing.
   It does decline when a registered nested worktree holds a file the main checkout tracks.
     A copy of the ignored state
   leaves the trash parent out of its source as well,
     so no copied worktree carries another command's trash.
     Another process that
   replaces or relinks directories of the new worktree while this runs is a risk the person accepts by using the
   command:
     only the cheap checks (no followed links,
     identities compared on opening) guard against it.
  Unless `--quiet` is given the snapshot also prints one line of its own saying the worktree was made as a snapshot of the
   main checkout and that nested worktrees,
     caches,
     run records and the stage containers of other commands were left out.
  The line reads like `monochromatic-git-policy-cli: this worktree was not made as a snapshot, so it took longer, because the main checkout has
   uncommitted changes in 2 paths (for example notes.md). Commit or stash them, and the next add is a snapshot that
   takes seconds.` It tells uncommitted changes,
     a running Git command,
     an unfinished operation,
     an index that cannot yet
   vouch for its files (`git status` once fixes it),
     a source that is no subvolume,
     and a snapshot that started and failed;
   `journalctl --identifier=monochromatic-git-policy-cli` finds the same entry afterwards,
     with the fields `CLI_GIT_WORKTREE`,
   `CLI_GIT_SLOW_CAUSE` and `CLI_GIT_SLOW_DETAIL`.
     On a filesystem that is not btrfs it says nothing.
  The ladder does not read `--force`,
     `--no-checkout`,
     `-B`,
     `--orphan`,
     `--track`,
   any `-c` option,
     or `--no-worktree-copy`:
     Git makes such a worktree alone,
   as an ordinary directory,
     and `--no-worktree-copy` skips the ignored-state copy with it.
     An add run from such a worktree says it is not a subvolume and names
   `git monochromatic-git-policy-cli convert-worktree-to-subvolume`.
  Every worktree the ladder makes on btrfs is a subvolume,
     whatever rung served it,
     so rung 1 needs no migration after the first one.
  Git's `post-checkout` hook runs on no rung,
     because every real Git start of the wrapper switches hooks off.
  `git worktree remove <directory>` (no option,
     or one `-f`/`--force`) destroys an unlocked subvolume worktree with one
   ioctl and deletes its administrative directory when the mount has `user_subvol_rm_allowed`
   (the doctor prints the commands that set it);
   in every other case Git removes it.
  A forced removal is served without the cleanliness proof,
     because Git's own `--force` removal deletes a worktree that holds changes whatever it holds
     (`builtin/worktree.c`:
     `if (!force) check_clean_worktree(wt, av[0])`),
     and it words its own line on standard error because nothing was checked first.
  A second force,
     which is what Git needs for a locked worktree,
     stays Git's own.
  It does so only for a worktree that the repository you run it in registered and that Git would pick for the name you typed
   (Git matches the name against the last components of every registered path before it resolves the path);
     an empty name,
   another repository's worktree and a name that Git resolves differently go to Git,
     which words its answer.
  A subvolume destroy cannot be undone,
     and the command prints a line that says so each time it removes one:
     a write,
     a
   lock or a rename that another process makes in the moments between the last check and the destroy is destroyed with the
   worktree,
     as it is when Git deletes the files one by one.
     Do not run it on a worktree something else is still using.
  What the command no longer does itself is delete the files:
     it renames the worktree's subvolume to `cli-git-deleting-<subvolume id>`
   beside the path you typed,
     writes a record of it,
     tells the worktree's commit service,
     and returns;
     that service deletes the
   subvolume in its own time,
   normally within a second,
     so a busy volume's btrfs transaction or cleaner does not hold your
   command.
     The path you named is gone the moment the command returns;
     the files are gone when the service has run.
     A
   directory named `cli-git-deleting-<16 hexadecimal digits>` that stays in a worktrees' parent directory means the
   service did not delete it yet:
     the next command of that repository wakes the service that reads the record,
     which is every
   commit and every add,
     and the service says in `journalctl --identifier=monochromatic-git-policy-cli` why it could not delete it
   (a mount without `user_subvol_rm_allowed`,
   for one).
     Nothing of yours is in it:
   a removal without `--force` runs only
   after the wrapper proved the worktree clean and unlocked,
   and a forced removal deletes whatever the worktree held,
   which its own line says on standard error because nothing was checked first.
  If the files are destroyed but the registration cannot be removed,
     the command fails with status 2 and a report that
   names the leftover `.git/worktrees/<name>` directory and the command that finishes the removal
   (`git -C <repository> worktree prune`);
     it does not run Git afterwards.
  A worktree that still has a conversion's copy beside it is not removed,
     and what proves the copy is a conversion's
   is the record that conversion wrote beside the worktree,
     and the object at that name must still be one the record holds:
     a
   directory that no record names,
     or one the record's identities do not match,
     or a record whose stage is gone,
   is not a
     copy a conversion left,
     and the removal goes on and leaves the directory alone:
     the command
   refuses with status 1 and a report that names the copy and both commands that clear it
   (`convert-worktree-to-subvolume <path>` to finish the conversion,
     `--abandon <path>` to discard the copy),
   because the removal deletes the registration and its journal is what names that copy.
     `--force` and `prune` go to Git
   and still remove the worktree,
     leaving the copy behind,
     which `--abandon` then reaches at its recorded identity.
  The destroy is recorded beside the repository before it is made
     (`monochromatic-git-policy-cli/recorded-destroys/<name>` under the common Git directory),
     so a command killed between the destroy and the removal of the registration leaves a state the next command finishes
   by itself,
     and only after what the registration still holds is copied into the removal snapshot,
     so a push failure nobody saw yet is shown instead of dying with it.
     A destroy that failed,
     and a removal that finished,
     drop the record.
  From the main checkout the copy carries the dependencies and builds a new worktree needs to run at once
   (`node_modules`,
     `target/`,
     `dist/`,
     local configuration files and the repository's `.cache/`),
   and leaves out caches,
     the records of earlier runs and any directory that is another checkout.
- Signals.
  The wrapper catches `SIGINT`,
   `SIGTERM`,
   `SIGHUP` and `SIGQUIT` while it waits for Git.
  A signal sent by another process is relayed to the Git child,
   and the wrapper then ends without leaving state of its own behind.

## What a worktree snapshot holds, and when the wrapper removes one

A `git worktree add` on btrfs is served by a snapshot of the worktree the command runs from,
 usually the main checkout.
A snapshot shares every data block with the tree it was taken from,
 so it costs almost nothing at the moment it is made.
What that means over time surprises people,
 so this section says it plainly.

**A snapshot keeps the files the source had at the moment of the add.**
The new worktree sees its own copy of them,
 but the blocks it shares are the ones the source held then.
When the source later rewrites a file,
 the new bytes go to new blocks and the old blocks stay,
 kept alive by the snapshot,
 until that worktree is removed.
This holds for every file the add copied,
 ignored ones included:
 `node_modules/`,
 each `package/*/*/target/`,
 `dist/`,
 and the repository's `.cache/`.
So a finished worktree that nobody removes pins one generation of the source's files.
The cost is invisible in any single worktree,
 because no snapshot owns those blocks;
 it shows up only as space that no file seems to use.

**The wrapper removes a worktree by itself when the branch it holds lands.**
When a `git merge` run through this wrapper lands a branch on the branch the merge ran in,
 every linked worktree whose checked-out branch that merge brought in is removed,
 provided all of these hold:

- this program recorded that it made that worktree,
   which it does when its own `git worktree add` succeeds;
- the branch it has checked out holds a commit of its own,
   which its reflog proves by beginning at an earlier commit;
   a worktree a person has just added and not committed in is never taken
   by someone else's merge,
   because a merge of another branch brings every branch sitting on its commits along;
   a repository that keeps no reflog for that branch proves nothing about it,
   so its worktree is examined as it was before this rule;
- the worktree does not hold another worktree this repository registers,
   whose files and whose own record a removal would take with it;
- the worktree is not the one the merge runs in,
   and it is not locked;
- no merge,
   rebase,
   cherry-pick or revert is in progress in it;
- no process is working in it or in a folder below it;
- Git would remove that worktree without `--force`:
   no tracked file is modified or staged there and no untracked file exists.

On btrfs the removal destroys the worktree's subvolume in one call,
 which is what gives the shared blocks back;
 elsewhere,
 or when the mount does not allow that call,
 real `git worktree remove` runs instead.
Either way,
 **the branch is kept**,
 and so is every commit on it;
 only the checked-out copy goes.

**What goes with the worktree.**
The files Git ignores in it go too,
 exactly as they do when a person runs `git worktree remove`:
 a `node_modules/` and a `target/` are rebuilt by the next task,
 and anything else ignored that you put there yourself is gone.
That is the whole reason a worktree is removed only when this program made it:
 a worktree you created by hand is never removed by itself,
 because this program cannot know which ignored files in it are yours.
A worktree this program cannot prove it made is named instead,
 once per merge that lands its branch,
 with the `git worktree remove` line that removes it.

**What a removal keeps.**
A worktree's own Git directory also holds the push failures nobody has seen yet
 and the pushes the service had not run,
 and a removal deletes it.
So a removal copies them first,
 as `git worktree remove` does,
 and the next command shows what the worktree could no longer say:
 the failed push with Git's words,
 and an unfinished push as one that may not have finished.
A commit a reference lock held in that worktree is kept the same way,
 on `refs/cli-git/kept/<batch>` of this repository,
 before anything is removed.

**What a worktree left behind says.**
Every merge that lands a branch names each worktree it kept,
 or could not prove it made,
 giving the path,
 the branch,
 and the reason nothing was removed.
A worktree whose branch holds no commit of its own is no candidate at all,
 so a merge that brings in another branch's commits leaves it where it is.
When no kept reference can be written for a commit a reference lock holds,
 the removal is stopped and the notice names the `git worktree remove` line that finishes it:
 running the merge again removes nothing,
 because a branch that already landed makes that merge up to date.
Nothing is removed in silence,
 and nothing is removed while a person is working in it.

**Seeing what the snapshots hold.**
The commands that answer this on this repository's filesystem,
 each read-only:

```sh
# every linked worktree Git knows, with its branch
git worktree list

# the subvolumes under the worktrees folder, which is what a snapshot is
btrfs subvolume list --sort=path .claude/worktrees

# space per worktree: the total each one sees, and what only it holds
btrfs filesystem du --summarize .claude/worktrees/<name>

# what the whole set holds, and what the filesystem has left
btrfs filesystem du --summarize --human-readable .claude/worktrees
df --human-readable .
```

A worktree whose exclusive figure is small and whose total is large is one of many snapshots
 sharing the source's blocks:
 removing it alone frees little,
 and removing all of them frees the generations they pinned.
That is why the wrapper now removes them when their branches land.

**The copies a repository keeps ahead of an add.**
A repository whose commit service runs keeps up to three snapshots of a worktree waiting in
 `<git-dir>/monochromatic-git-policy-cli/ahead`,
 so that an add takes one instead of forcing a btrfs commit of the whole
 filesystem on the person's thread.
The names begin with `.cli-git-worktree-ahead-` and no Git command ever lists them,
 because they are inside
 the Git directory.
An add removes what no add claimed for ten minutes,
 and one nothing claimed yet is removed by the commit service as
 the next add asks for a copy.
To see them:

```sh
# the copies a commit service made ahead for this repository's adds
ls --almost-all .git/monochromatic-git-policy-cli/ahead

# what one of them holds, and what only it holds
btrfs filesystem du --summarize .git/monochromatic-git-policy-cli/ahead/<name>
```

A copy that waits here pins the source's rewritten blocks the way a worktree's own snapshot does,
 which is why it is
 removed after ten minutes and why a repository keeps three at most.
Removing one by hand is safe:
 it is a copy,
 and
 `btrfs subvolume delete` on it takes nothing of the source away.

## Other processes changing files

The wrapper checks files,
copies them and,
in a few commands,
removes them,
and no check or lock it takes can stop another
process from changing them in the same moment.
A rename or a replacement of a path,
a write through an open file or a shared mapping,
and an edit between a check and the step
that relies on it are outside what the wrapper guards against,
in every command and in the commit service.
Running the program is accepting that risk.
The cheap guards stay (a lock where one is free,
a second look before a removal,
an identity check on what is removed),
but
they narrow the window and do not close it.

A command that can change or delete files states the risk in its own `--help`,
where it has one,
and says it once on standard error;
a command whose own line already states it never prints a second one:
`git monochromatic-git-policy-cli migrate-to-subvolume` before it first writes,
keeps,
swaps or deletes anything (a `--dry-run` and a refusal change nothing and say nothing);
`git monochromatic-git-policy-cli convert-worktree-to-subvolume` before it touches the first worktree
(a `--dry-run`,
an `--abandon` and a run that names no worktree say nothing);
and `git worktree remove` once it has destroyed the worktree's subvolume in one step
(its `--help` is Git's own).

## Requirements

- Linux with glibc 2.39 or newer.
  Release archives exist for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` only.
  They are built on the `ubuntu-latest` runner and link `pidfd_spawnp` and `pidfd_getpid`,
  which glibc added in 2.39
  (measured on the 0.2.2 x86_64 archive with `objdump -T`),
  so they do not start on Debian 12 or Ubuntu 22.04.
  The crate does not compile for Windows:
  `cargo check --target x86_64-pc-windows-gnu --lib` fails with 16 errors in
  eight files (the `tempfile` and `ignore` dependencies are unix-only and several modules use `std::os::unix`),
  so the Windows-specific branches in the sources have never run;
  each is marked `UNTESTED ON WINDOWS` in the source.
  A Windows port is deferred to a later version (owner decision,
  2026-10-15).
- Real Git on `PATH` after the wrapper,
  tested with 2.56.0 (`MINIMUM_GIT_VERSION` in `src/native/invocation/git_version.rs`).
  The wrapper checks the version before work that depends on Git's behavior,
   such as a commit.
  A Git older than that release is not refused:
   the wrapper prints one warning that names the Git it found,
   the release it reported and the release the wrapper was tested with,
   says that the command continues,
   and runs the command as usual
   (owner decision,
  2026-11-04:
  "Raise minimum,
  drop the test,
  merely print a warning on older git and not refuse.").
  Only a Git whose release cannot be read stops the command.
  The constant follows the test image's Git:
   the gate image installs the distribution's current `git` package,
   so when `mise run //package/git-policy/cli:test:git-image` reports a newer release,
   raise the constant to it with the first gate run on that release.
  The wrapper finds the real Git by walking `PATH`,
   skipping itself and anything that leads back to it:
   a link or a copy of the wrapper,
   its toolchain launcher (recognised by the `.launch` record beside it),
   and a script that starts the TypeScript wrapper.
  When `PATH` offers none,
  it uses the Git recorded for its commit service,
  with a notice (see "Install").
- `git-lfs` on `PATH`,
  only in a repository whose own Git configuration has `filter.lfs.*` or `lfs.*` keys.
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

The installed file is `monochromatic-git-policy-cli`,
 not `git`.
Link it as `git` in a directory that comes before the real Git on `PATH`:

```sh
# Run in the repository once per checkout, after the installation.
mkdir --parents .cache/monochromatic-git-policy-cli-bin
ln --symbolic --force "$(mise which monochromatic-git-policy-cli)" .cache/monochromatic-git-policy-cli-bin/git
```

Never name a file `git` inside the tool's own `bin/` directory.
`mise` would then create a `git` shim for the whole account,
 and every Git command would run through the wrapper.

The wrapper never starts itself as Git.
If every `git` after the wrapper on `PATH` leads back to monochromatic-git-policy-cli (the link directory comes alone,
or the real Git's
directory is missing),
it falls back to the real Git that its commit service was set up with,
when the unit file of
the service (`monochromatic-git-policy-cli@.service` below `$XDG_RUNTIME_DIR/systemd/user`) names one in its `--git` option and that file
is an executable that does not lead back to monochromatic-git-policy-cli.
It then runs your command through that Git and writes one line on standard error that says so and which directory to
add to `PATH` to silence it.
This fallback is the coordinator's pick under the owner's delegation,
open to the owner's veto
(`doc/planning/cli-git-no-self-invocation.md`).
Without a usable record (no unit yet in this login,
or a unit that names the wrapper,
a launcher of it,
or a missing
file),
it starts nothing and stops with exit status 2.
The message lists the `PATH` it read and every candidate with the reason it was skipped,
names the record when one was
refused,
and says how to fix it:
install Git,
or put the real Git's directory (for example `/usr/bin`) on the `PATH` of the
program that ran the command.
Every start of the real Git sets `CLI_GIT_STARTED_AS_GIT`,
so a monochromatic-git-policy-cli executable that a wrapper started as Git
(another release,
or a launcher without a record) stops on arrival with a message saying so;
a hook,
alias or editor that the real Git runs may still run an installed wrapper.
The rule and the incident behind it are in `doc/planning/cli-git-no-self-invocation.md`.

`PATH` shadowing covers only processes that find `git` through `PATH`.
An absolute path to the real Git,
 a `PATH` without the link directory,
 a GUI with an embedded Git library and direct
libgit2 consumers bypass the wrapper completely,
 including its policies.

There is no switch that forces one command or one repository through the native route,
and no plan for one:
the owner answered a request for such a switch (2026-11-10)
with "No switch,
/usr/bin/git is it (Recommended)".
A person who wants a command to skip the wrapper's policies runs `/usr/bin/git` itself,
which is the same way out that the paragraph above describes.

## Commands

The wrapper adds one namespace to Git.

```sh
# Check or fix the whole repository, or the named pathspecs.
git monochromatic-git-policy-cli check --all
git monochromatic-git-policy-cli fix -- docs/ README.md
git monochromatic-git-policy-cli check --all --policy final-newline
# List the hook files and configured hooks the repository holds; none of them runs.
git monochromatic-git-policy-cli doctor
# Whether the commit service answers, and finish the batches an earlier release's service left open (see below).
git monochromatic-git-policy-cli service status
git monochromatic-git-policy-cli service settle-earlier-batches
# Stop the commit service units of worktrees that no longer exist (see below).
git monochromatic-git-policy-cli service stop-units-of-removed-worktrees
# Convert the main checkout into a btrfs subvolume (preview first); see the section below.
git monochromatic-git-policy-cli migrate-to-subvolume --dry-run
git monochromatic-git-policy-cli --help
```

The wrapper never starts its own executable,
never writes shell text or scripts,
and explains a failure from the
parent process that owns the child (see `doc/planning/cli-git-no-self-invocation.md`).
A Clippy rule bans `std::env::current_exe` and `src/native/source_guard/no_self_invocation_tests.rs` scans the source for the rest;
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
 and the `monochromatic-git-policy-cli.config.jsonc` key that turns it on
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

`git monochromatic-git-policy-cli service settle-earlier-batches` finishes the batches that the commit service of an
 earlier release (the program was called `cli-git-native` and its units `cli-git@<instance>`) accepted for the worktree
 it runs in and left open,
for example after a logout or a kill,
so that the person's index shows the landed commit again.
It takes no options and acts on the current worktree only.
**It is safe only when no wrapper of the earlier release is in use in that worktree at this moment**:
 such a wrapper sets its commit service up again and can start the earlier helper while the command runs,
 and two services could then settle one batch.
Before it acts,
the command checks that no socket of the earlier units listens and no earlier helper runs;
 when one does,
it settles nothing,
says which one it is,
and says what to stop for that one:
 this worktree's earlier units,
or the single service of the first releases (`cli-git.socket` and `cli-git.service`),
 which serves every worktree.
A running program whose command line carries `--journal` with a folder the check cannot look up
 (an unrelated tool can carry the option too) also holds the command back,
 because it cannot be ruled out as an earlier helper;
 the message then names that program by process ID and executable path,
gives the folder and the system's answer,
 and says how to go on:
 wait for it,
stop it,
or run the command again after it ends.
A word that is an absolute path to a folder that is not there is the one exception:
 such a folder holds none of the worktree's batches,
so the program holds nothing back unless the path names one of the worktree's earlier journals,
 which is how a starting helper is recognised before it makes that folder.
Its other arguments are never printed or logged,
because any of them can be a secret.
It offers to wait for a helper's idle exit (three hours after its last client) only when no socket is left listening,
 because a listening socket starts the helper again and does not stop by waiting.
When the checks pass it says the condition above on standard error,
 then finishes the batches that landed,
one line each on standard output,
 and leaves the intent of a batch that looks as if it never landed on disk.
A batch it could not examine (Git gave no answer,
for example because its repository is gone) is named in that output,
 stays on disk,
and makes the exit status 1.
The command also refuses,
changing nothing,
when it cannot write that condition to standard error.
Nothing runs it by itself,
and a commit never waits for it.
`git monochromatic-git-policy-cli doctor` says on standard error how many such batches the worktree has and names this command.

Each worktree has a commit service socket unit and a service unit,
named by a hash of the worktree's own Git directory (`monochromatic-git-policy-cli@<hash>.socket` and `.service`).
A socket unit has no idle exit,
and the service exits only three hours after its last client,
so units of a worktree that was removed would run on.
When the wrapper serves `git worktree remove <path>` (the subvolume destroy,
or Git's own removal run as a child),
it reads the worktree's registration first,
and once the removal ended well and the registration is gone it stops that worktree's socket unit and then its service
unit through the user manager,
never by killing a process by name.
A removal Git refused leaves the units alone,
a worktree that never committed costs the removal no question to the manager,
and a manager that cannot be reached is logged and never fails the removal.
Two forms are not seen afterwards because the process becomes Git
(`git worktree prune`,
and a `git worktree remove --force` the wrapper does not serve because the worktree is no subvolume
or its mount lacks the option),
and a worktree deleted without Git leaves its units too.
Those leftovers end at the first connection,
when the service starts and finds the worktree gone,
or when the command named next runs.
`git monochromatic-git-policy-cli service stop-units-of-removed-worktrees` stops those:
it takes no options and looks at every unit instance of the user,
not at one repository.
Each commit service setup writes the worktree's Git directory into the folder it writes its unit files to
(`$XDG_RUNTIME_DIR/systemd/user/monochromatic-git-policy-cli-worktree-notes/<hash>.git-dir`),
which the manager reads as no unit,
and which goes away with the units at logout or shutdown;
for a unit set up before that,
the `gitdir=` line of a record in the service's journal names the worktree.
Either text counts only when it hashes to the instance.
The command stops only an instance whose Git directory is missing while the repository that held it is there
(a repository on a drive that is not mounted proves nothing),
says which worktree each stopped instance was,
and names an instance that maps to nothing without touching it.
A stopped instance's note and socket file are removed,
because the manager leaves the socket file of a stopped socket unit in place,
and a file that stayed would count the instance again.
Its journal goes with them when it holds nothing else and no client of the instance can still read it:
the records of the batches that landed,
once they are older than the longest a client that lost its answer can look,
and an empty landing record are the memory of a service that will never run again,
while a batch that did not land,
a landing that was in flight,
and a file this release does not know keep the whole journal,
and the command names the file it kept it for.
A journal that stays only for the clients that may still read it,
or for a service the manager does not yet report gone,
keeps the note and the socket file too and is not named:
nothing in it is the person's business,
and the next sweep removes it once both are over,
so until that sweep runs `git monochromatic-git-policy-cli doctor` counts the instance and names this command.
A refused stop is named with the reason and makes the exit status 1;
running the command again tries it again.
It starts nothing,
nothing runs it by itself,
and no unit it concerns is enabled for login.
A service that finds its own worktree gone stops its own socket and serves nothing,
so those leftovers also go without anyone running the command.
It asks the same question of the same note and journal records,
when it starts and then once a minute while it serves
(a removal by another Git takes the worktree while its service is running,
so no start of that service ever comes),
stops its socket with the stop mode that leaves the units requiring it alone,
removes the note,
the socket file and the journal when the journal holds nothing else,
and ends with status 0.
Its own end is what stops its service unit,
so it never asks the manager to stop the service it is,
and a failure status would make `Restart=on-failure` start it again.
`git monochromatic-git-policy-cli doctor` says on standard error
how many removed worktrees still have units and names this command.

A removal through the wrapper stops the worktree's units and asks the manager whether the service's processes are gone
before it removes the journal that service writes,
because a stop answers when the job is queued:
a service that is still going keeps its journal and the files that name the instance,
and the next sweep finishes when it is gone.
The wait for that answer is two seconds (`SERVICE_GONE_AFTER_A_STOP` in `service/wait_ceilings.rs`);
a service that outlasts it costs the removal no more than the kept journal.

`git monochromatic-git-policy-cli doctor` takes no options.
It scans `<git-common-dir>/hooks` and the directory a configured `core.hooksPath` names,
 and prints the path of every traditional hook it finds,
one per line,
on standard output:
 each executable regular file and each symbolic link,
except names ending in `.sample`.
A hook file that is byte for byte the shim git-lfs writes for its own name is not listed there:
 it is explained on standard error,
with its path,
as normal and harmless (git-lfs writes these files again whenever
 it cleans a file in a Git the wrapper did not start;
the wrapper does the LFS upload itself and never runs them;
 delete them or leave them).
It exits 0 when it finds none or only such shims,
1 when it finds any other hook,
and 2 when it cannot locate or read
 the repository.
Traditional hooks are not supported,
 so a script that still checks something should become a monochromatic-git-policy-cli policy under `policies`.
It also lists the hooks configured with `hook.<name>.event` (one `hook.<name> (events: ...)` line each,
exit 1) and
 notes a `hooks` key in `monochromatic-git-policy-cli.config.jsonc`,
which is accepted for older files and ignored.
A commit does not scan for traditional hooks:
it only warns when the configuration sets `core.hooksPath` or defines
 hooks,
which it already knows without any extra file system access.

The doctor also says whether Git's own fsmonitor is off,
on standard error and before the filesystem section
 (see "Git's own fsmonitor"):
one line when `core.fsmonitor` is false or unset,
and a finding with the file,
the scope
 and the command that turns it off when it is on.
It never changes the exit status and never writes configuration.

After the filesystem section the doctor prints one line,
`repository link: <message>`,
when the file `~/.config/monochromatic-dev/repository-link.json` exists (the move of the repository to the 860 drive recorded it):
the same plain message `repository-link-check` prints (ready,
the drive is not mounted,
a real folder replaced the link,
and the others).
It is informational too and starts no program;
a machine with no such file gets no line.
Every Git command whose paths lie under that repository link,
its approved root or the renamed old copy is checked the same way first,
and a link that is not usable stops it with that message and status 128 before Git prints its own error;
with the file absent the check costs one failed open.

The doctor ends its standard error with a filesystem section for the main checkout of the repository Git selects
 (so `-C`,
`--git-dir` and a linked worktree report the main checkout;
a bare repository has none).
It is informational:
it never changes the exit status and starts no program.
It reads `statfs`,
the checkout's inode number,
`/proc/self/mountinfo`,
`/run/ostree-booted` and `/proc/cmdline`.
Every line of the section after the first is indented,
and the first begins with `git monochromatic-git-policy-cli doctor: filesystem:`.
Four cases are told apart:
 a checkout on a filesystem that is neither btrfs nor overlayfs gets one line saying that `git worktree add` uses Git's
 own checkout and no faster method applies;
 a checkout on overlayfs gets the detection,
that cloning file data inside the overlay works and a snapshot does not
 apply,
and no command;
 a checkout on btrfs that is an ordinary directory is told that making it a subvolume can help with performance
 (worktrees can then be made from btrfs snapshots,
which took a median of 0.35 s to 1.09 s in the design measurements
 against 2.1 s to 2.7 s for Git's checkout,
see `doc/planning/cli-git-btrfs-worktree.md`,
and until then are made by
 clones of the tracked files) and is given `git monochromatic-git-policy-cli migrate-to-subvolume --dry-run` and
 `git monochromatic-git-policy-cli migrate-to-subvolume`;
 a checkout that already is a subvolume gets what a worktree add and a worktree remove do there,
and the commands that
 set `user_subvol_rm_allowed` when the mount lacks it.
On btrfs it also checks the mount option `user_subvol_rm_allowed`,
which lets a regular user delete a subvolume in one
 call.
When it is absent,
the doctor prints the command to set it,
for the host it finds:
 `sudo mount --options remount,user_subvol_rm_allowed <mount point>` until the next boot;
 on an ostree or bootc host,
for the filesystem the kernel command line configures (the one mounted at `/sysroot`)
 with exactly one `rootflags=` word on the running command line,
 `sudo rpm-ostree kargs --delete=rootflags=<old> --append=rootflags=<old>,user_subvol_rm_allowed` (delete and append,
 because the value contains `=` and libostree splits each argument at the first `=`,
which makes `--replace` read the
 old value wrongly),
then a reboot;
 for any other filesystem,
or on a conventional host,
an `/etc/fstab` entry (or a systemd mount unit) for it.
Nothing is run for the person.

The former `trust`,
 `untrust` and `status` commands are retired.
Configuration is data,
 so nothing needs approval.
They print an explanation and change nothing.

### Converting the checkout to a btrfs subvolume

`git monochromatic-git-policy-cli migrate-to-subvolume` turns the repository's main checkout into a btrfs subvolume at the same path.
A subvolume is the precondition for creating worktrees from btrfs snapshots,
and `git monochromatic-git-policy-cli doctor` says when it would
 help.
It runs inside the wrapper:
it calls the btrfs ioctls,
`FICLONE`,
`renameat2` with `RENAME_EXCHANGE` and its own tree
 walk,
and it starts no program (no `cp`,
`rsync`,
`btrfs` or `mv`),
so it needs no tool besides the wrapper.
Run it inside the repository,
from the main checkout or from any linked worktree:
it finds the main checkout by reading
 `.git` itself.
It takes minutes on a large checkout and there is no time limit on it.
One command at a time works on a checkout:
it owns the checkout through a claim file beside the journal
 (`.cli-git-migrate-<name>.owner.<generation>`),
a second command is refused with the owner's process id,
and a claim a
 killed command left needs no cleanup.
The journal is replaced by writing a sibling file that is created exclusively under a name of its own (the journal's name,
 `.tmp`,
the process number and a clock reading),
synchronizing it and renaming it over the journal,
so a link or a file
 planted or left at a fixed name is never followed,
truncated or written through.
Releases up to 0.5.20 wrote one fixed sibling name and followed a link planted there.
The plain files a killed run left under the writer's own names (that fixed name included) are removed once a later write lands;
 a link,
a directory or any other name beside the journal is left alone.

```sh
git monochromatic-git-policy-cli migrate-to-subvolume --dry-run   # preflight and plan; nothing is created
git monochromatic-git-policy-cli migrate-to-subvolume             # migrate; run it again after any stop and it resumes
git monochromatic-git-policy-cli migrate-to-subvolume --rollback [--discard-unreadable] [--discard-left-out]  # undo, finished or not
git monochromatic-git-policy-cli migrate-to-subvolume --remove-old [--discard-unreadable] [--discard-left-out]  # delete the original tree
```

The steps,
each recorded in a journal beside the checkout (`.cli-git-migrate-<name>.journal.json`):
 create `<root>.subvol-new` as a subvolume;
 create each linked worktree inside the checkout as a nested subvolume (`--no-nested-worktrees` keeps them ordinary
 directories),
so that a later snapshot of the root holds an empty stub for each;
 copy the tree,
cloning file data so that the new tree shares every data block with the old one (only metadata is
 copied);
 repeat the copy as a delta that also deletes what the checkout lost meanwhile;
 hold a marked `index.lock` (Git's own lock for writing an index,
the one lock that cannot be dropped) for the main
 index and for every linked worktree's index,
in the original before the last delta and in the new tree before the swap,
run the last delta and swap the two directories in one atomic call;
 rename the original to `<root>.before-subvolume`;
 carry over files that are newer in the original (writes that landed there after the swap),
while the new checkout
 holds the locks;
 rewrite the stat fields of every Git index of the checkout and its linked worktrees from the copied files
 (`--no-refresh-indexes` leaves them;
the first `git status` then hashes every file,
and runs the Git LFS clean filter
 on LFS files);
an entry is rewritten only when Git could trust it (its time is earlier than the index file's own) and
 the original file still has the stat fields the entry records,
so a file edited behind Git's back stays dirty;
 release the locks and verify that the root is a subvolume with a readable `.git/HEAD`.
The original tree is never deleted by the migration.
One measured run,
on a synthetic checkout of 121,260 entries (forty thousand tracked files in the checkout and in one
 linked worktree,
forty thousand ignored files) under a scope of `CPUQuota=200%` and `IOWeight=20`:
 the copy took 8.34 s and 9.67 s in two runs,
each delta pass 0.81 s to 1.02 s,
the index rewrite 0.35 s and 0.40 s,
 and the whole migration 11.55 s and 14.74 s.
The copy cost about 70 to 80 microseconds per entry and a delta pass about 7 to 8 microseconds per entry;
the pass under
 the locks is the delta pass,
so they are held for that long,
for the exchange,
and now also for the catch-up and the
 index rewrite.
Each lock file holds a token that names the migration.
A lock that another Git command holds when the migration needs
 it stops the migration with the file named,
and is never taken over,
exempted or removed;
the migration's own locks are
 recognised by their token when a run resumes.
Every entry is written under a temporary name and renamed into place,
so a stop never leaves a half-written file under
 its real name,
and a file that changes while it is copied is copied again.

A process whose working directory is inside the checkout keeps that directory:
after the swap its relative writes land
 in the original tree.
The preview and the run list such processes by number;
stop agent sessions and editors first,
and run `cd "$PWD"` in
 shells afterwards.

The command refuses,
with the cause and every way forward,
and changes nothing,
when:
 the checkout is not on btrfs (overlayfs is named separately),
is a mount point,
is on another device than its parent
 or is a symbolic link;
 the parent directory has fewer free bytes than the metadata of the checkout's entries needs
 (2048 bytes per entry:
one kibibyte measured per copied entry on btrfs,
written twice by the `DUP` profile;
the command
 lists every directory once to count the entries,
and the refusal names the count and both byte figures);
 `<root>.subvol-new` or `<root>.before-subvolume` exists without a journal that explains it;
 Git is in the middle of an operation (any of `index.lock`,
`MERGE_HEAD`,
`CHERRY_PICK_HEAD`,
`REVERT_HEAD`,
 `BISECT_LOG`,
`rebase-merge`,
`rebase-apply`,
`sequencer`,
`HEAD.lock`,
`config.lock`,
`packed-refs.lock` or
 `shallow.lock` in `.git` or in a linked worktree's administrative directory);
 the commit service has unfinished work in `.git/monochromatic-git-policy-cli/work/pending` or `claimed`.
A checkout that already is a subvolume with no migration recorded is no refusal:
the command says there is nothing to do,
exits 0,
and names `--remove-old` when the migration's original tree is still beside it.
Exit status 0 means done,
already done or previewed;
1 means a refusal;
2 means a step failed after the journal began,
 and the message says to run the command again to resume or to add `--rollback`.

Files this user may not read (for example root-owned mode 600 logs written by a privileged run) stay in the original
 tree and are listed in `.cli-git-migrate-<name>.unreadable.txt`,
and the run says so loudly.
A directory this user cannot list stops the run before the swap,
because copying around it with deletion on would
 erase its subtree from the new tree.
Because the original tree is then the only copy of the unreadable files,
`--remove-old` refuses while the list is not
 empty,
unless `--discard-unreadable` is given with it;
copy the files over with the needed privilege first
 (for example `sudo cp --archive --reflink=always --parents`).

`--rollback` carries back files that are newer in the new root,
swaps the trees again (recorded in the journal before the
 call,
so a rerun after a stop never swaps twice),
rewrites the restored indexes,
removes the migration's own index
 locks and deletes the new subvolume,
nested subvolumes first.
Files that only the new subvolume holds and this user may not read cannot be carried back,
so `--rollback` refuses
 while there are any,
unless `--discard-unreadable` is given with it.
Both `--remove-old` and `--rollback` refuse when the directory at the checkout path is not the one the migration
 installed
 (its device and inode are recorded with the migration's last step),
 because a checkout moved or replaced by hand since then
 is one the migration never recorded.
The carry writes through a descriptor on the directory whose identity was checked,
 so a checkout moved aside and another
 put at its path receives nothing,
 and the old tree is kept when the name no longer leads to that directory.
Before it deletes the new subvolume,
 `--rollback` compares the identity at that name with the recorded one on every path that
 reaches the deletion,
 so a run resumed after the exchange was recorded keeps a subvolume another process put there
 and names both identities.
`--remove-old` first carries into the checkout what was written to the original after the catch-up (files newer than
 their copy,
and files the checkout lacks that are no older than the catch-up;
older ones were deleted on purpose),
then
 deletes the original.
An entry it left out that way is either a file deleted from the checkout on purpose or a file restored into the original
 tree after the migration (an unpacked backup carries old timestamps),
and no check can tell the two apart:
the command counts them,
writes their relative paths to `.cli-git-migrate-<name>.left-out.txt` beside the checkout,
names them,
and deletes nothing until `--discard-left-out` is given with `--remove-old`
 (or you delete the original tree yourself).
A rollback's carry back leaves an entry out the same way when the original tree holds another kind of entry at its
path,
and the new subvolume's directory there is the only copy of what it holds:
the rollback names those entries and
 deletes nothing until `--discard-left-out` is given with `--rollback`
 (or you delete the new subvolume yourself).
Both deletions make the directories this user owns writable on the way down,
so a mode-555 directory does not stop them.
Both refuse,
before they delete anything,
when a directory below the tree is a nested subvolume or a mount point,
and
 name it:
its content is not part of the tree.
 Move it out,
delete it with `btrfs subvolume delete`,
or unmount it,
then run the deletion again.
Deleting a subvolume uses the kernel's destroy call where the mount has `user_subvol_rm_allowed` and otherwise empties
 it and calls `rmdir`,
which the kernel allows for an empty subvolume root.

Risk you accept by running it:
it replaces or deletes files of the checkout while other programs may be changing the same files
(see "Other processes changing files"),
and a program that writes through an open file or a mapping,
or that renames or replaces a path,
between a check and the step that relies on it can lose that change.
Stop the programs working inside the checkout first.
The command says so once on standard error before it changes anything,
and in `--help`.

### Converting nested worktrees to subvolumes

`git monochromatic-git-policy-cli convert-worktree-to-subvolume [--dry-run | --abandon] [--jobs <count>] (--all | <worktree>...)` turns a linked
worktree that is an ordinary directory into a btrfs subvolume at the same path,
several worktrees side by side.
A snapshot of the main checkout shows a nested worktree that is a subvolume as an empty directory,
so converting the ones
under `.claude/worktrees/` once removes the work of setting each of them aside at every `git worktree add`.

- The worktrees share nothing but the filesystem (each has its own administrative directory,
  index,
  claim file and journal),
  so
  they are converted side by side,
  as many at once as the host allows:
  the threads this process may run less an eighth of
  them (at least one,
  none when only two),
  no more than the free memory holds at the measured peak of one conversion,
  never
  more than sixteen (the measured point past which the filesystem gains nothing),
  and no more than `--jobs <count>` when it
  is given.
  The report is the one a run of one worktree after another gives:
  each worktree's
  name and lines together,
  in the order named (with `--all`,
  sorted by path),
  whichever finishes first,
  with the lines of the
  first unfinished worktree shown as they happen.
  A worktree named twice is converted once and then found converted.
- A worktree that fails after its journal began does not stop the others.
  It keeps its journal and says so,
  every other worktree
  is converted,
  the closing line counts the failures,
  and the status is 2;
  running the same command again finishes the failed
  ones.
  Releases before this one stopped at the first failure.
- `--dry-run` lists each worktree and either `would convert` or why it is skipped:
  locked,
  an `index.lock`,
  an operation in
  progress,
  a modified tracked file,
  a process with its working directory or an open file inside it,
  a worktree registered
  inside it,
  or a converting directory that no journal owns.
  Run it first;
  it changes nothing.
- Without `--dry-run` the command takes no lock.
  It owns the worktree through a numbered claim file in the administrative
  directory (`cli-git-convert.owner.<generation>`) that names its process;
  a second command that finds a live owner skips the
  worktree and names that process,
  and a claim a killed command left needs no cleanup.
  A conversion is also refused while a
  `migrate-to-subvolume` of the main checkout runs,
  and that command is refused while a conversion runs.
  Then it creates
  `<path>.cli-git-converting` as a subvolume and records its identity,
  copies the worktree into it with cloned data,
  takes the
  administrative directory's `index.lock` with its own token only for the rest (the long copy runs without it),
  proves the
  copy equal with a second pass,
  swaps the two directories with one exchange,
  checks that the path is the subvolume it created
  and has the same `.git` bytes,
  rewrites the index stat fields so the first `git status` stays fast,
  gives the `index.lock`
  back,
  and removes the old tree only when it is the directory it recorded and the installed worktree holds all of it.
  A `cli-git-convert.lock` file that release 0.5.20 or older left is removed when no command holds it,
  and a conversion
  that release left unfinished is completed by running the command again.
  The registration never changes:
  the path and the bytes of the `.git` file are the same.
- The old tree is kept,
  and the run says why,
  when a process still holds it,
  when anything in it differs from the installed
  worktree (size,
  modification time,
  kind,
  link target) or is missing there,
  or when it contains a subvolume or a mount point,
  which this command does not delete.
  Nothing is ever copied from the old tree into the installed one,
  so an edit made to the
  installed worktree is never overwritten;
  whoever knows which version is wanted settles it by hand and runs the command again.
- A quoted line in any attributes file,
  an attribute source chosen with `GIT_ATTR_SOURCE`,
  `attr.tree` or `--attr-source`,
  or a
  filter,
  `ident` or encoding attribute that may apply to a file whose recorded size and times cannot vouch for it,
  makes the
  command refuse that worktree and name the reason,
  instead of deciding by the file's content.
- Where Git reads its system configuration file and its system attributes file from is a fact only the real Git the wrapper
  forwards to can give (`git var GIT_CONFIG_SYSTEM`,
  `git var GIT_ATTR_SYSTEM`,
  learned once per executable and cached).
  The
  command resolves that Git itself,
  as every command that forwards does,
  because the wrapper puts the variable that names it
  only into the environment of the Git processes it starts,
  never into its own.
  Release 0.5.20 and older read their own
  environment,
  found no Git named,
  and skipped every worktree whose index entries the stat data could not vouch for with
  "the Git configuration cannot be read without running Git";
  this release judges those worktrees by content.
  Where no real
  Git is found,
  such a worktree is still skipped with that reason,
  which is the safe side.
- Risk you accept by running it:
  it deletes the old directory,
  and it cannot stop another process from changing the worktree
  while it works (see "Other processes changing files").
  It says so once on standard error before it starts,
  and in `--help`.
- Run the same command again after any stop:
  it completes the conversion in place.
  `--abandon` undoes one that has not
  reached its swap,
  destroying only the subvolume its journal says it created,
  and refuses while a process has a shell or a file open inside it (the process is named).
  The scan runs again immediately before the destroy,
  so the window is one check wide;
  it is not closed,
  and a shell that enters the copy in that window is left in a `(deleted)` directory whose writes fail.
  The identity of a leftover stage,
  and the two directories a rollback exchanges,
  are read again in the step before the call
  that uses them,
  so a directory another program renames into the name inside that interval is destroyed or exchanged:
  neither call
  takes a descriptor,
  and binding them would need one.
  It also reaches the copy a conversion left behind when its worktree was removed before the
  conversion was finished (a worktree path without a `.git` that still has `<path>.cli-git-converting`
  beside it):
  the copy is destroyed,
  nothing is destroyed while a process is inside it,
  and nothing is destroyed whose identity is not one the conversion recorded beside the worktree
  (`.cli-git-convert-<name>.leftover-stage.json`),
  so a directory someone put at that name is kept and named.
- Measured on a fixture of four ordinary worktrees of 22.3 thousand entries each:
  4.2 s per worktree under a scope of
  `CPUQuota=400%`,
  load average 6 to 10.
- Stop sessions that have a working directory in the worktree first;
  the command refuses while one does.

### Repairing package-manager state in a new worktree

A copy keeps symbolic links and build state exactly as they were,
 which is wrong for tools that record where they were installed or built.
pnpm's links climb out of the tree by a fixed number of levels to reach the shared store,
 so in a worktree at another depth they are dead,
 and Node then resolves the name through a parent `node_modules` to the source's packages without any error.
Cargo bakes the source's paths into what it built (`CARGO_MANIFEST_DIR`,
build script output).
The toolchain launchers copied into `.cache/toolchain-launchers` name the source's `toolchain.lock.json`.
After the ladder (or Git's checkout) and the ignored-file copy,
 the wrapper repairs each tool's own state in the new worktree.
Cargo's build state is planned and cleaned here;
the two tools that rewrite link layouts in place are the commit service's work whenever a service answers,
 and the add's own when none does:

- pnpm:
  `pnpm install --offline --frozen-lockfile`,
  when `pnpm-lock.yaml` and `node_modules` are both there.
  The `pnpm` is the first one on your `PATH` (in this repository the pnpm-shadow,
  whose warnings are shown only if the run fails).
  The add writes this tool in the `relink` record it hands over,
  and the service runs it there after the add returns.
- Cargo:
  for each Cargo workspace,
   the source's external build directory is filled in after the add returns,
   by the commit service's clone of it into the worktree's when the worktree has none,
   then `cargo clean --offline --package <member>` runs for every member
   (a `Cargo.lock` the clean would rewrite is put back as checked out,
  so the worktree is not dirty).
  The clean runs in a worktree you may already be working in,
   and nothing read afterwards can tell Cargo's own rewrite of the lock from a newer one of yours,
   so the bytes standing there when the clean ends are kept whole under the worktree's Git directory,
   in `monochromatic-git-policy-cli-displaced-locks`,
   under a directory named by the lock's path in hexadecimal,
   and the success line names each copy.
  A lock whose bytes cannot be kept there is left as the clean found it and the worktree shows the difference.
  Registry and git dependencies stay reused;
  the worktree's own crates rebuild with their own paths.
  A file or directory in the source's build directory that you cannot read is left out of the clone and counted in the success line;
   the source is never touched.
  The workspaces are planned side by side,
  on as many threads as your machine's available parallelism.
- Toolchain:
  `mpm-shadow install --frozen-lockfile`,
  written into the same `relink` record as pnpm when the worktree's launcher records still name another checkout's lock.
  It also downloads the tools the worktree's lock names that are not installed yet,
   and can retire tool copies another checkout still links (both accepted by the owner),
   and its summary of what was installed is part of the success line.

What the hand-over costs you:
for as long as the service has not run the record,
 the new worktree still holds the copied links,
 which is exactly the state the copy left,
 so run the command from your own line below if you need that state before the service gets to it.
The record is kept in the repository's work directory,
 so a service that was not running when you added the worktree relinks it the next time it runs for that repository.
When no service can be reached at all (a session with no user service manager,
 such as a container),
 the add relinks the worktree itself,
 which is what every release before this one did.

Nothing runs for `--no-worktree-copy` or `--no-checkout`.
`--quiet` silences the one line that reports success;
a failure is always printed.
When a tool is missing,
its offline data is incomplete,
a lock does not match or the run exceeds its limit,
 the add still succeeds and prints one line that names the reason and the command to run,
 such as `pnpm --dir <worktree> install --offline --frozen-lockfile`.
Another process changing files while a worktree is copied or repaired is your risk and is not defended against,
except for the `Cargo.lock` a repair's clean rewrites:
the bytes that clean displaces are kept under the worktree's Git directory and the success line names them.
Times measured on this repository are in `doc/planning/cli-git-main-worktree-ignored-copy.md`,
 section "Package-manager state of a new worktree".

### Version and help

The wrapper runs the real Git
 for the top-level `--version`,
`-v`,
`--help` and `-h`,
 so Git's own output and exit status come first and are never changed.
Only when Git exits with status 0 does the wrapper add plain text to standard output,
 so a person can tell that a policy wrapper sits in front of Git.
When Git exits with another status,
or a signal ends it,
nothing is added and the wrapper returns Git's status.
Global options before the query,
such as `git -C <directory> --version`,
behave the same.
A subcommand's own help,
such as `git commit --help`,
is untouched and so is every other query
 (`--exec-path`,
`--html-path`,
`--man-path`,
`--info-path`,
`--list-cmds=<group>`):
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
the `git monochromatic-git-policy-cli` management command (`git monochromatic-git-policy-cli --help`),
and `git monochromatic-git-policy-cli doctor`, which lists the traditional hooks a repository holds.
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
- `--no-enforce-worktree` skips `linked-worktree-only` for the forms listed in "Discarding commands"
  and "The remote settings every worktree shares".
  Write it directly after the subcommand,
   for example `git restore --no-enforce-worktree -- <path>` or
  `git remote --no-enforce-worktree set-url origin <url>`.
- `--no-enforce-worktree-branch` skips `branch-worktree-only`.
- `--no-enforce-bulk-add` skips `add-explicit`.
- `--no-worktree-copy` skips the ignored-state copy into a new linked worktree and the whole worktree ladder:
  Git alone
  makes an ordinary directory.
- `--cli-git-keep-going` reports every finding of the command in one run instead of stopping at the first blocking one.

### Hints

Every block and every divergence from real Git's behavior says what you were probably trying to do and how to proceed.
A JSONL event (a finding,
an engine failure or a warn-unsafe notice) carries the hint in its own `hint` field,
 beside a `message` that is unchanged from the retired wrapper's text.
A plain notice (an internal error,
a push stopped by its deadline or its output cap,
a lock wait that gave up,
 a failed worktree copy,
a real Git that cannot start,
a commit that normalization emptied) ends with the hint.
The rejections that already carry their own alternatives (`require-root`,
`linked-worktree-only`,
 `branch-worktree-only` and the commit-only rules) keep them in the message.
`src/native/policy/event_hints.rs` holds the texts,
and its tests fail when a finding code or an engine failure code has no hint.

### Discarding commands

`linked-worktree-only` rejects a command form that can overwrite or delete uncommitted work by itself.
The rule is:

- In the main worktree such a form is always rejected,
   clean or not.
- In a linked worktree it is rejected while the worktree holds uncommitted tracked changes
   (for `git clean`,
  also untracked files),
   and runs when the worktree is clean.
- An ignored file is not a reason to refuse a clean in a linked worktree,
  by the owner's decision of 2026-11-10 ("Allow and log",
  because a refusal only moves the same loss to `rm -rf`):
  `git clean -x` counts tracked and untracked files only,
  and `git clean -X` deletes ignored files and nothing else,
  so that one runs whatever the worktree holds.
  Both exit with Git's own status and leave one audit line naming the command and the worktree,
  and its `dirty` field is `null` for `git clean -X`,
  because no question was asked.
  The main checkout still refuses both.
- A `git clean -fd`,
  which keeps the ignored files,
  is not rejected for an ignored file,
  though the rules above still reject it in the main worktree and in a linked worktree that holds
  uncommitted changes or untracked files.
  Its own report ends with the hatch advice,
  and the report a `git clean -x` gets names the whole form,
  `git clean --no-enforce-worktree -fd`,
  which deletes the untracked files and keeps the ignored ones.
  `git clean -nx` and `git clean -nX` are the dry runs that list what each would delete.
  A `git clean -X` that names its own exclusions (`-e`,
  `--exclude`) is measured by Git's own dry runs of that clean,
  because those patterns decide what it deletes and a status cannot see them:
  the clean as typed and the same clean without the patterns are printed by Git,
  and the command is refused while only the patterns call a path ignored,
  because that path is an untracked file the clean would lose.
- A form that only moves `HEAD` (`git checkout <branch>`,
  `git switch <branch>`) is rejected in any worktree
   only while it holds uncommitted tracked changes.
- A `git checkout` or `git switch` that changes any file is forwarded with `--no-overwrite-ignore`,
  so Git itself refuses a switch that would replace an ignored file the target branch tracks,
  and names the file instead.
  Nothing is lost when nothing would be replaced,
  and a person who means it can still write `git checkout --overwrite-ignore <branch>`.
- A forced `git checkout -f`,
  a forced `git switch -f` and a `git reset --hard` (`--merge`,
  `--keep`) override that token,
  so in a linked worktree they are rejected while the worktree holds an ignored file at a path the tree they
  write tracks,
  or where one of those paths needs a directory,
  and the rejection names the path.
  The tracked state of the worktree is still what makes the worktree clean or dirty.
- A form that only moves `HEAD` also runs where the Git directory has no `index` file,
  the state `git clone --no-checkout` leaves:
   Git sees an initial checkout there (`is_index_unborn` in `read-cache.c`,
  `initial_checkout` in `builtin/checkout.c`),
   while `git status` lists every committed file as a staged deletion.
  An `index` file that exists,
  empty or not,
  never counts as absent,
  a `GIT_INDEX_FILE` that names another index is not read as absent,
   and `git checkout -f`,
  `git reset --hard` and the other forms that overwrite stay rejected.
- A status Git cannot read stops the command instead of reading as clean.
- `git stash drop` and `git stash clear` delete entries of `refs/stash`,
  the one stash every worktree of the repository shares,
  so they are rejected in a linked worktree while the repository holds an entry,
  whatever that worktree's own status says.
  `git stash list` and `git stash show` read the stash,
  and `git stash branch <branch>` moves an entry into its own branch and keeps the work,
  which runs in a clean linked worktree and is rejected in the main worktree and in a linked
  worktree that holds uncommitted changes,
  because it is a stash form that changes state.
- Repositories under an allowlisted tool directory are exempt:
  uv's cache (`UV_CACHE_DIR`,
  else
   `XDG_CACHE_HOME/uv`,
  else `~/.cache/uv`) and mise's data directory (`MISE_DATA_DIR`,
  else `XDG_DATA_HOME/mise`,
   else `~/.local/share/mise`),
  where mise keeps its plugin clones and updates them with a forced checkout,
  and Cargo's build-script output directory (`OUT_DIR`,
  which Cargo sets for every build script and passes on to what it starts),
  where a crate that vendors another project clones it and runs Git in that clone,
  as `libghostty-vt-sys` did with Ghostty.
  An empty value of any of these variables counts as unset.
  The exemption is chosen by the environment of the command itself,
  so the wrapper says on standard error which variable exempted the repository and which directory it named,
  because a variable that happens to name a repository turns the guard off for it.

The guard reads the command Git will run,
not the word on the command line:
an ordinary Git alias is expanded as Git expands it,
up to 16 expansions,
the decision is taken on the expansion,
and the same expansion is what the fixed transforms rewrite and the wrapper forwards.
`git config --null --list` decides which value Git keeps when both spellings of one alias are set,
so the last one in config order runs,
and the value is split on space,
TAB,
LF and CR only,
keeping the empty last word a trailing whitespace character leaves.
Two ordinary aliases are left to Git as you typed them,
because the expansion the wrapper read is not the command that runs:
one whose value moves the directory or sets an environment variable,
which Git refuses and runs nothing for,
and one whose name an installed `git-<name>` program carries,
which Git runs before it looks any alias up.
A shell alias (`!...`) is a shell command,
which no reading of its text can judge,
and Git runs it with its own `git-core` directory first on `PATH`,
so the `git` inside it is real Git rather than this wrapper.
One plain form of it is read,
only so the guard can decide:
a value written `!git <words>` whose bytes hold no quote,
backslash,
`$`,
backtick,
glob,
brace or separator,
and whose words name no global option of their own.
Git appends your own words to that value,
so `-c 'alias.wipe=!git' wipe reset --hard` is refused where `git reset --hard` is,
and so is `-c 'alias.wipe=!git reset --hard' wipe`.
Every other shell alias runs as Git runs it,
with no guard:
`!sh -c 'git reset --hard'` and `!git -C <directory> reset --hard` included.
An alias the guard reads is still forwarded as you wrote it,
because the shell keeps the `GIT_DIR` and `GIT_PREFIX` Git gives it.

Two commands are never rejected and always leave one audit line (outcome `logged`,
with `writtenBeforeGitRuns` `true`,
because the line is written before Git runs and cannot say whether Git ran the deletion or refused it),
by the owner's decision,
also when `--no-enforce-worktree` is given:

- `git worktree remove`,
  because agents are as likely to run `rm -rf`;
  the line names the worktree,
  whether `--force` was given,
  the `HEAD` it was on,
  and how many tracked paths differed from it
  (and says that it counted tracked changes only).
- `git branch -d`,
  `git branch -D` and `git branch --delete` from the main worktree;
  one line per branch names the branch,
  the reference it took (`refs/heads/<name>`,
  or `refs/remotes/<name>` with `-r`/`--remotes`),
  its commit and whether it was already merged,
  so `git update-ref <reference> <commit>` brings it back.

Every rejection names the form,
 what it would destroy,
 the alternative that fits the most likely intent,
 and the deliberate way to discard anyway.
The deliberate way is always the same:
 save a copy (`git diff > backup.patch`) and pass `--no-enforce-worktree` directly after the subcommand.
`git apply backup.patch` brings the edits back.

During a merge,
a rebase,
a cherry-pick or a revert there is one exception.
`git checkout --ours|--theirs -- <path>`,
`git checkout <rev> -- <path>`,
`git checkout --merge -- <path>` and the
 `git restore` forms with `--ours`,
`--theirs`,
`--source` or `--merge` run for conflicted paths only
 (the paths with an unmerged index entry),
 in the main worktree and in a linked one,
 after the wrapper saves your current file.
The copy goes to `monochromatic-git-policy-cli/discarded/<UTC time>/<path>` in the common Git directory,
beside `monochromatic-git-policy-cli-audit.jsonl`,
 and the wrapper prints where before Git runs:

```text
monochromatic-git-policy-cli: saved your copy of a.rs to
  /path/to/repo/.git/monochromatic-git-policy-cli/discarded/2026-11-02T14-01-07/a.rs
  before this command replaces it (a.rs was conflicted).
```

A path that is not conflicted keeps the rejection,
 and a line that names conflicted and clean paths together is rejected whole with the clean paths named.
Write each conflicted file by its own name after `--`;
 a directory,
a wildcard,
pathspec magic,
a missing `--`,
`--force`,
`--patch`,
`--pathspec-from-file` and
 `restore --staged` are not covered and stay rejected.
 A token that is the value of `--source`,
 `-s` or `--conflict` is never read as `--worktree` or `--staged`,
 so `git restore --source -W --staged -- f` (a tag can be named `-W`) is an index-only restore and stays rejected,
 and the rejection names `git restore --staged`,
 not a restore of file content.
A conflicted path that is deleted in your worktree has nothing to copy,
and the command runs.
When a copy cannot be written (no space,
no permission),
the command is rejected and says why.
 A symbolic link or a file where a directory in front of the name should be (`dir` for `dir/file`) is never followed:
 Git would replace it with a real directory and it would be lost unsaved,
 so the command is rejected,
 names `dir`
 and says to move it out of the way or save it first.
 A file that changes while it is being copied (an editor's atomic save,
 an append,
 a rewrite in place) is not reported saved:
 the copy is removed,
 the command is rejected,
 and running it again once nothing writes to the file goes through.
 A file that another program changes while the save runs is your risk.
 A pathspec mode (`--icase-pathspecs`,
 `GIT_ICASE_PATHSPECS` and the glob,
 noglob and literal forms) keeps the rejection,
 and when every path you named is a conflicted file the message names the mode and says the same command without it
 is saved first and then allowed.
 The same holds for pathspec magic on a conflicted file's name (`:(icase)a.rs`,
 `:!a.rs`,
 `:/a.rs`):
 the message names the magic and the file,
 and naming the file itself is saved first and then allowed.
The folder is bounded,
because it would otherwise grow for as long as the repository lives,
 and the copies are the only remaining content of replaced files.
 When a new copy has been written and synced,
the wrapper removes the saves older than 30 days,
 then,
while the folder is over 1 GiB,
the oldest saves first.
 A save younger than 24 hours is never removed by either limit,
and neither is the save just written.
 Only directories named like the wrapper's own (`2026-11-02T14-01-07`) are touched.
 No commit waits for this,
and no lock is taken:
passes that overlap share a save's files,
and each copy is audited once.
 The folder is never followed through a link:
if `.git/monochromatic-git-policy-cli` or `.git/monochromatic-git-policy-cli/discarded` is a symbolic link,
 nothing is removed,
and the command's notice and `git monochromatic-git-policy-cli doctor` say so;
 make it a real directory to apply the limits again.
 A save that cannot be read,
a directory replaced while the pass ran,
a removal that fails part way
 and an audit file that cannot be written are all kept and named the same way,
never counted as removed.
 Each removed save leaves one line in `monochromatic-git-policy-cli-audit.jsonl`
 (`"outcome":"pruned"`,
with the time,
`path`,
`bytes`,
`reason` `age` or `size`,
 the `limit` it was judged against and the names of the copies in it).
 The limits are the `discardedCopies` section of `monochromatic-git-policy-cli.config.jsonc`.
 The defaults are not yet based on a measurement of growth:
 on 2026-11-03 no repository on the development host had a saved copy,
 and a copy can be the only copy of your work,
so the numbers are open to change
 (`doc/planning/cli-git-shipped-limits-audit.md`,
"Defaults for saved conflict copies").
 An example that keeps saves for 90 days and the folder under 256 MiB:

 ```jsonc
 // monochromatic-git-policy-cli.config.jsonc: keep saves for 90 days and the folder under 256 MiB
 { "discardedCopies": { "maxAgeDays": 90, "maxSize": "256m" } }
 ```

 A value that is no limit (zero,
 a fraction,
 text that is not a size) is a configuration failure that names the key;
 to keep copies longer,
 set a large number.
 `git monochromatic-git-policy-cli doctor` reports the folder's size,
 the number of copies and saves,
 and the oldest save,
 once it holds any.

Each decision about a discarding form (rejected,
run in a clean worktree,
run in an exempt cache,
or run with the
 escape) is appended as one JSON line to `monochromatic-git-policy-cli-audit.jsonl` in the common Git directory,
 with the time (`time` and `timeMillis`),
a `level` (`warn` for a rejection),
the release,
the process and its parent,
 the worktree kind and the worktree's Git directory (`gitDir`),
the form,
the working directory and the arguments,
 each path field and each argument written beside its text as its bytes (`gitDirHex`,
`cwdHex`,
`argumentsHex`),
 so a name that is not UTF-8 still names the command that was run
 (a decision that saved conflicted files also lists where each copy went).
When uncommitted work disappears,
`git rev-parse --git-common-dir` locates the file;
 a line that says `allowed` or `hatch` near the time of the loss names the command that was let through.
At one mebibyte the file moves aside to the next free number (`monochromatic-git-policy-cli-audit.jsonl.1`,
then `.2`,
and on;
the highest is
 the newest) and a new file starts.
The rotated files are kept up to 32 MiB together,
oldest removed first,
and
 a rotated file with a `saved-conflict` or `saved-nested-repository` line whose copy still exists is never removed,
which can take the log past 32 MiB.
 The reading of a rotated file to see whether it names a copy happens on a thread of its own,
never on the command's path.
 A rotation killed between taking the file and placing it leaves the file at a private name (`.moving-<process>-<count>`);
 the next rotation gives it the next free number,
 so every line is read by number,
 and the time of a line is in the line,
 not in its number.
 Every other log of the wrapper keeps its rotations the same way,
under the same 32 MiB,
with no copy to pin:
the commit timing log,
the worktree-add timing log,
the push trigger log and the standards-review shadow log.
The owner answered on 2026-11-10 in the question tool,
"Keep more,
capped (Recommended)":
32 MiB per log,
and an audit line naming a saved copy that still exists is never dropped.
The bound is about six days of commit timing at the pace measured on 2026-11-10
(1,143,526 bytes of 229 records in 5.4 hours),
and the whole history of an ordinary audit log
(782,685 bytes in 1,619 lines over twenty days).
 A failed write never changes a command's result.

The forms and their alternatives follow.
Each alternative is run through the built executable by `src/native/binary/binary_discard_tests.rs`.

- `git stash` (every form except `list`,
  `show` and `create`):
   commit work in progress with explicit paths on a topic branch in its own linked worktree
   (`git worktree add -b <branch> <path>`).
- `git clean` without `--dry-run`:
   `git clean -n` lists what would go,
   delete specific files with `rm`,
   or use the project's own clean task.
- `git reset --hard`,
  `--merge`,
  `--keep`:
   `git reset --soft <commit>` undoes a commit and keeps its changes,
   `git diff <commit>` compares,
   `git worktree add <path> <commit>` gives a clean copy.
- `git reset` in the default or `--mixed` mode:
   `git commit --message <message> -- <paths>` commits only the named paths and ignores other staged entries;
   a directory or a wildcard takes the tracked paths and staged additions it matches,
  as native Git does,
   while a path that names one file takes that file even when nothing staged it,
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
- `git read-tree` in any form Git writes with,
  `--reset`,
  `-m`,
  `-m -u`,
  `--empty` and a plain tree included:
   `git read-tree --dry-run` with the same arguments shows what the read would change without writing anything,
   `git ls-tree` and `git cat-file` read a tree without touching the index,
   or replace the index in a clean linked worktree.
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

### The remote settings every worktree shares

The owner chose "Refuse in worktrees (Recommended)" on 2026-11-10
 (`doc/handover/questions-for-the-owner.md`,
"A push URL changed from a worktree").
Every worktree of a repository shares one `.git/config`,
 so a remote setting changed from a linked worktree changes the main checkout and every other worktree too.
An agent had made a worktree outside the repository folder,
ran `git remote set-url --push origin DISABLED` in it believing it was a separate clone,
and every push of `main` failed from then on with
`fatal: 'DISABLED' does not appear to be a git repository` until someone removed that entry,
with nothing recording which command had made it.

In a linked worktree,
 the one whose Git directory is not the common one,
 wherever it sits on disk,
 `linked-worktree-only` rejects:

- `git remote set-url`,
  `git remote add`,
  `git remote remove`/`rm`,
  `git remote rename`,
  `git remote set-branches` and `git remote set-head`.
  `-v`/`--verbose` and its abbreviations are the only options `git remote` reads before a subcommand,
  so `git remote --verbose add beta <url>` is the same rejection as the plain form,
  and the subcommand's own options belong to the subcommand.
- a `git config` write whose key or section falls under `remote.`,
  in the plain `name value` form and with `--add`,
  `--unset`,
  `--unset-all`,
  `--replace-all`,
  `--rename-section`,
  `--remove-section`,
  `set`,
  `unset`,
  `rename-section` and `remove-section`.
  A rename into the `remote` section creates remote settings just as renaming out of it removes them.
  Section names are compared the way Git compares them,
  without case,
  so `git config Remote.Origin.URL <url>` writes the same section as `remote.origin.url`.
  An option after the key is a value Git stores,
  so `git config remote.origin.url --worktree` stores the text `--worktree`,
  and the scope is read where Git reads it.

No read is rejected:
`git remote -v`,
`git remote get-url`,
`git remote show`,
`git config --get`,
`git config get`,
`git config list`,
`git config --get-regexp`.
No write to another file is rejected either:
`--global`,
`--system`,
`--worktree`,
`--blob`,
and a `--file` whose path is not the shared file:
the path is resolved from the directory the command runs in,
so `git config --file ../repo/.git/config <key> <value>`,
an absolute spelling,
one written with `=` and one through a link are refused as the plain form is,
while `git config --file other.config <key> <value>` and `git config --file other.config --edit` run.
An editor session on the shared file is refused as well:
`git config --edit` and `git config edit` open that file when no scope option names another one,
and an editor can change any remote setting,
so `--edit` with `--global`,
`--system`,
`--worktree` or a `--file` of its own is what runs.
The `edit` subcommand takes no argument ("wrong number of arguments,
should be 0"),
so `git config edit <key>` keeps Git's own refusal.
A setting that is not a remote setting can always be named from the worktree:
`git config <key> <value>`.
`git -c remote.origin.url=<value>` is one command's override and changes no file,
so it runs.
The main checkout keeps today's behavior,
and so do a bare repository and a repository under an allowlisted tool directory.

The rejection says that every worktree of the repository shares the setting,
names the shared file and the main checkout,
and offers both ways forward:

- change the setting on purpose in the main checkout,
  which owns that file;
- make a real clone for a copy whose remote settings are its own:
  `git clone --no-hardlinks <path> <destination>`.

A rejection changes nothing,
and Git never runs.
The deliberate way through is the hatch every other rejection of this policy takes,
`--no-enforce-worktree`,
written directly after `remote` for a remote subcommand
 (the only place a command without a table of its own reads it)
 or in option position of `git config`:

```text
git remote --no-enforce-worktree set-url origin <url>
git config --no-enforce-worktree remote.origin.url <value>
git config --no-enforce-worktree --edit
```

A hatch run is not silent about the change it makes:
the decision leaves one `hatch` line in `monochromatic-git-policy-cli-audit.jsonl`,
with `form` `SharedRemote` and the working directory and arguments every audit line carries,
so a later reader can attribute the change to the command that made it.
A rejection leaves no audit line,
because nothing was written and the JSONL finding on standard error is the record.
A linked worktree is also one whose remote settings a shell alias would change:
the guard reads the `git` command a `!` alias runs,
and `git -c 'alias.su=!git remote set-url' su origin <url>` is refused like `git remote set-url`.

`src/native/binary/binary_shared_remote_tests.rs` runs every refused form in a linked worktree made inside and
 outside the repository folder,
every allowed form,
the hatch and the main checkout through the built executable,
and compares the shared configuration byte for byte after each rejection.
The decision is `src/native/command/rule_shared_remote.rs`,
the options `git remote` reads before a subcommand are `src/native/command/command_remote.rs`,
and the `git config` form,
file scope and write are `src/native/command/command_config.rs`.
The rejection text is `src/native/command/rule_shared_remote_messages.rs`.

### Merging

The owner's rules (2026-11-03):
"Auto merging should never fail."
and "Manual merge should never fail too."
Native Git's merge that creates a commit does two things that break them in a checkout many sessions share:
 it first runs `git stash create` on the index (`save_state()` in Git's `builtin/merge.c`;
no stash ref is ever stored),
 which dies with `could not write index` and `stash failed` when another process holds `index.lock`,
 and it changes the index and the worktree before it moves the branch,
 so a branch that moved meanwhile (another session's commit,
or the auto-push's own reconcile merge)
 leaves the checkout half merged with `fatal: update_ref failed for ref 'HEAD'`.
The wrapper therefore completes the merge itself whenever the command is one it understands,
 whether the index differs from `HEAD` (other sessions' staged work) or not.
Native Git also refuses,
when the index differs from `HEAD`,
 a `git merge <branch>` that needs a merge commit with
 `error: Your local changes to the following files would be overwritten by merge`,
 whichever files are staged.
The wrapper completes that merge instead,
when all of these hold:

- the command is `git merge` with one commit-ish and only the options the wrapper reads exactly,
  each in the full spelling Git documents or its short one,
  never an abbreviation:
  `-m`/`--message` (a value after it,
  or attached to it),
  `--no-edit`,
  `--ff`,
  `--no-ff`,
  `-q`/`--quiet`,
  `-S`/`--gpg-sign` (with or without a key),
  `--no-gpg-sign`,
  `--stat`/`--summary` (Git's synonym for it),
  `--no-stat`/`--no-summary`/`-n`,
  `--log`,
  `--no-log`,
  `--allow-unrelated-histories`,
  and the output-only options that leave the merge commit exactly as it would be without them,
  which the isolated merge therefore does not carry:
   `-v`/`--verbose`,
  `--no-verbose`,
  `--progress`,
  `--no-progress`,
  `--verify`,
  `--no-verify`,
  `--no-signoff`,
  `--no-autostash` and `--no-verify-signatures`
  (`is_ignorable` in `src/native/merge/merge_options.rs`;
  `--log` or `--no-log` together with `-m` is native Git's);
- `HEAD` is not already ahead of or equal to the merged commit
   (native Git fast-forwards and answers "Already up to date" by itself);
- no merge,
  rebase,
  cherry-pick or revert is in progress,
  neither `merge.autostash` nor `merge.verifySignatures` is true,
  and no `branch.<name>.mergeOptions` is set;
- the merge would not open an editor,
  unless the checkout holds the person's own work in the paths the merge changes:
  that merge is completed with the editor (see "The editor a carried merge opens").
  A merge over a clean checkout that would open an editor,
  and one started with `--edit`,
  is native Git's whole,
  as before.

The merge tree comes from `git merge-tree --write-tree`,
the commit from `git commit-tree`,
signed when `-S` is given
 or `commit.gpgSign` is true.
Its message is Git's own (`git fmt-merge-msg`,
then the whitespace cleanup of a commit that opens no editor),
 and the checkout then moves with `git merge --ff-only <merge commit>`.
That is Git's two-way update:
 it writes exactly the paths the merge changes and keeps every other index entry and file.
The merge commit skips `pre-merge-commit`,
`prepare-commit-msg` and `commit-msg`,
 like the merge the auto-push makes from objects;
no hook runs around it at all,
`post-merge` included,
for the reason "Hooks" gives:
every Git the wrapper starts runs with `core.hooksPath` at a directory that cannot exist.
The branch reflog reads `merge <branch>: Fast-forward`.

A merge that lands a branch also removes the worktrees that hold it.
After the branch moved,
 every linked worktree whose checked-out branch this merge brought in is removed by itself when
 it is recorded as one this program made,
 its branch holds a commit of its own,
 it is not the worktree the merge runs in,
 it is not locked,
 no operation is in progress in it,
 no process works inside it,
 and Git would remove it without `--force`.
Each worktree the merge removes,
 leaves,
 or cannot prove it made is named on standard error with its branch and the reason.
See "What a worktree snapshot holds,
 and when the wrapper removes one".

#### Carrying the person's own work

The owner's answer for a merge over unsaved edits is "Carry edits forward (Recommended)"
(`SPEC.md`,
"Merges and commits never fail"),
and the rules behind it are "Manual merge should never fail too."
and "never discard user data".
Native Git refuses such a merge
(`error: Your local changes to the following files would be overwritten by merge`,
exit 2),
and so did the wrapper.
Now the merge runs and the person's work is carried across it:

- Every path the merge changes that holds the person's own work in the worktree or the index has its bytes
  written as objects,
  and kept on `refs/cli-git/carried/<n>`,
  before the first byte of the checkout changes;
  the merge's record names them.
  A `SIGKILL` at any instant leaves them where the next command finds them.
- Those paths are put back to the branch,
  so Git's own two-way update takes them as it always did.
  Each path is read again just before the write that replaces it,
  and the file that was there is set aside under a name of its own and checked before it makes way,
  so a file the person saves while the merge runs is kept rather than replaced:
  the captured layers go back,
  the record and the ref go,
  and the merge is made again on what the file holds now.
- After the branch moved,
  each of the person's two layers is merged with the merge on its own:
  a staged edit stays staged and an unsaved edit stays unsaved.
  The index entries of a path are written before its file,
  and the next command takes the entries a killed merge wrote as its own,
  so a kill between the two restores the layer that was not written.
  A layer the merge does not reach is written back byte for byte.
  A text both sides changed goes through the word-level second pass;
  a region it cannot resolve keeps both sides between conflict markers,
  the file is named on standard error,
  and the command still exits 0.
- A binary file,
  a symbolic link,
  a mode the pass cannot take,
  a file the merge deletes and a file the person deleted each keep the person's own side whole and are named;
  the merge's version is in the merge commit.
  A path written again while the merge ran is left alone and named,
  and the naming is true:
  nothing was written to it before the check that found it changed.
- A merge killed after it cleared those paths puts the captured bytes back
  when it turns out the merge never landed,
  and carries them when it did.
  A layer the person saved again while the merge ran stays as they left it,
  and is named.
- What Git's own update left at each path is written beside the merge's record
  (`<transaction id>.landed`) once the checkout moved and before the carry,
  so the command that finishes a killed merge judges each path against the file Git left
  rather than against its bytes alone.
  A save that arrives after the kill and holds the merge's own bytes therefore stands,
  and is named,
  where reading by bytes alone would take it for the merge's own file
  and write the person's older layer over it.
- A file the merge set aside and could not put back is a copy beside its path,
  named `.cli-git-recovery-<key>-<process>-<count>-<moment>`,
  and it is never dropped while its bytes are the person's alone.
  A copy whose path is empty and whose merge is unfinished goes back into the checkout.
  A copy whose bytes the person's file holds again,
  or the branch's own commit holds for that path,
  is removed,
  and the line says which of the two it was.
  Any other copy stays where it is,
  and its line names the copy,
  `mv -- <copy> <path>` to put it back,
  `rm -- <copy>` to drop it,
  and the end a later command performs by itself once `<path>` holds its bytes again.
  Every copy a merge keeps is recorded under
  `.git/monochromatic-git-policy-cli-kept-copies/`,
  so every later command that runs the merge recovery decides it again,
  even when no merge touches its path.
  A copy an earlier release of this program left carries no merge key in its name,
  so its bytes decide it:
  a copy of the branch's own file,
  or of the person's own captured file,
  goes back while a merge is unfinished,
  and anything else is kept and named rather than put back on a guess.

A merge over a checkout whose own paths are clean runs exactly as it did.
The refusals that change nothing are:
- The merge conflicts textually,
  and the word-level second merge pass cannot resolve every conflicted file.
  The finding `merge-isolation/merge-conflict` names the conflicting paths.
  Resolve in a clean linked worktree or checkout,
  or rebase the branch.
  A conflict that does resolve by word completes the merge,
  and the lines of the merged parts come first on standard error.
- A path the merge changes stands as a directory,
  or the index holds an unmerged entry.
  The command is native Git's whole,
  which refuses such a checkout itself.
- A merge over that work that would open an editor ends without one:
  the editor exits with a failure
  (`there was a problem with the editor '<editor>'`),
  leaves an empty message,
  or is one this environment resolves to nothing.
  The command exits 1 and nothing in the repository changes;
  run the merge again to write the message.

A merge killed while it runs is finished or discarded by the next invocation (see "State on disk").

#### The editor a carried merge opens

A merge started from a terminal opens the person's editor for its message
(`default_edit_option` in Git's `builtin/merge.c`).
Native Git performs the merge before the editor opens,
so over the person's own work it refuses before anything is edited;
this wrapper opens the editor itself:

- The message is written to `<git-dir>/MERGE_MSG`,
  with the help lines Git writes under it.
  A comment character other than `#` (`core.commentChar`,
  `core.commentString`) opens the message alone,
  because those help lines would not be stripped then.
- The program is the one `git var GIT_EDITOR` names,
  and its arguments are the words of that command line,
  made here as a shell makes them:
  blanks separate them,
  single quotes hold their bytes,
  double quotes hold
  theirs,
  and a backslash escapes the next byte (`merge_isolation_editor_words.rs`).
  No shell is started,
  since the wrapper writes no shell text,
  so `code --wait` and `emacsclient -nw` work as under Git.
  A line that asks for an expansion,
  a tilde,
  a glob,
  a control operator or a comment is not guessed:
  that merge is native Git's whole.
  The editor gets this command's terminal,
  and no ceiling.
- What it leaves is cleaned the way a commit that opens an editor cleans it
  (`git stripspace --strip-comments`),
  and it is the merge's message.
  `--no-edit` and `-m` keep the merge away from the editor,
  as under Git.
- The message is written once,
  so a round made again because the branch moved never asks the person to type it twice.

A merge whose editor left nothing,
  or one whose editor failed,
  exits 1 with the reason and changes nothing;
  the message file goes either way.

#### A pull that merges

A `git pull` that merges is completed the same way when the command line or the settings say that it merges
 (`--no-rebase`,
`pull.rebase` or `branch.<name>.rebase` set to false,
or `--ff` or `--no-ff`) and every option
 and operand is one the wrapper reads exactly:
 the wrapper runs the pull's fetch,
 then the merge of `FETCH_HEAD` through the path of a typed `git merge`
 that carries the pull's own merge options,
 so the editor opens on a terminal exactly where `git pull` opens it.
When the pull the wrapper completes cannot move the working tree along,
the report keeps Git's own words
(`fatal: Cannot fast-forward your working tree.` and the `git reset --hard` it names)
and adds the line naming `git reset --no-enforce-worktree --hard`,
because the worktree guard refuses a hard reset in the dirty worktree that made the move fail.
Git's advice to save the `git diff <old head>` output misses the untracked file that made the move
fail,
and the reset it offers overwrites that file,
so the wrapper copies every such file beside the Git directory first
(`monochromatic-git-policy-cli/pull-kept/<n>`) and names each copy in that line;
the hatch is offered only once nothing the move would overwrite is left uncopied.
Every other pull is native Git's whole (a rebase,
`--ff-only`,
`--autostash`,
`pull.ff`,
submodules,
 a refspec with a destination,
 an abbreviated option,
a pull with nothing stated,
which Git refuses with its advice when the branches diverged),
 and it keeps native Git's failures (the held lock,
the staged index,
the late ref update);
 run `git fetch`,
then `git merge --no-edit origin/<branch>`.

#### Waiting for the index lock

Before Git's fast-forward changes anything,
the merge waits for any program's hold on `index.lock`
 (the commit service's,
a foreign `git`'s,
or a lock nothing proves held):
 until `MERGE_LOCK_CEILING` (1 minute),
whether its owner is proven alive or not
 (the line for an unproven owner says what is known and how to remove the lock if no program uses the repository);
 at the bound it names the holder and stops waiting:
 a merge the wrapper completes is saved on `refs/cli-git/kept/<n>`,
 exits 0 and changes nothing in the checkout
 (see "A merge saved for a later command"),
 and a merge native Git runs ends with Git's refusal,
non-zero,
with nothing changed.
The notice names the merge,
says it has changed nothing yet,
names the lock and the holder with its proof
 (an open descriptor or Git's PID file),
and repeats every `CLI_GIT_WAIT_NOTICE_SECONDS`.
No lock is ever removed by the wait itself.
A lock whose Git lock PID file names a process that no longer runs is released by nobody,
 so the merge clears it on the commit service's proof of disuse
 (an empty lock with no marker,
no Git lock PID file naming a process this table has,
and no process a complete scan finds holding it)
and goes on;
a lock that proof does not cover stays where it is,
and so does one whose content is not empty.
Native Git,
measured in a fixture,
never waits:
a held lock fails the merge in a few milliseconds.
A merge that native Git runs (a conflict,
an editor) waits for the lock first in the same way.

#### A merge saved for a later command

When the lock is still held after the bound,
or a lock the proof does not cover holds it at once,
the merge saves its commit on `refs/cli-git/kept/<n>`,
records that the checkout is still to be finished,
says so and exits 0.
Nothing of that merge is in the checkout,
so no later command waits for the lock:
 a command that meets the record says where the merge is saved and goes on at once,
and the first command run once the lock is free finishes the merge.
A merge that had already begun its checkout update is the other case:
a command that meets one waits for the lock,
 bounded,
because only finishing that merge lets the command read the checkout.

#### A branch that moved while the merge was prepared

The merge commit is built on the head the merge was computed on,
and Git's fast-forward moves the branch only from that head.
When the branch moved first,
Git's fast-forward refuses without changing anything,
 or,
if it lost the race in the instant between its checkout and its ref update,
 the paths it installed are put back to the moved head's content
 (only paths that hold the old,
the merged or the moved head's content;
any other content stops it
 with the paths named and the record kept).
The merge is then computed again on the moved head and lands as a merge commit whose first parent is that head,
 up to twenty times in a row;
each round says so on standard error.
No `MERGE_HEAD` is ever left behind by a merge that did not finish.

#### A conflict

With another session's staged work,
the word-level second pass (see "Word-level second merge pass") runs on the conflicts
 of the merge computed from objects,
and the merge completes when every conflicted file resolves;
 otherwise it is refused (`merge-isolation/merge-conflict`) and nothing changes.
With a clean index the merge is native Git's,
which writes the markers for the person to resolve in the worktree;
 the second pass then runs on the stopped merge.

#### What can still end a merge with a nonzero status

Each with its cause:

- A conflict that no resolution settles (exit 1 from native Git,
  or `merge-isolation/merge-conflict`,
  exit 2).
- A path the merge changes that stands as a directory,
  or an index that holds unmerged entries:
  the command is native Git's whole,
  which refuses such a checkout itself.
- A merge whose commit cannot be kept on a side ref while the lock stays held:
  the command is refused with status 2,
  changes nothing and leaves the lock.
  That is the outcome of a merge native Git runs,
  which fails on the held lock;
  a merge the wrapper completes is saved on a side ref and exits 0 instead,
  as "A merge saved for a later command" says.
- A branch that moved in twenty rounds in a row:
  the merge stops and changes nothing;
  run it again.
- A form the wrapper does not complete and native Git runs unchanged:
  `--squash`,
  `--no-commit`,
  `-s`,
  `-X`,
  more than one
   branch,
  `--abort`,
  `--continue`,
  `--ff-only`,
  abbreviated options,
  an editor,
  `merge.autostash`,
   `merge.verifySignatures`,
  `branch.<name>.mergeOptions`.
  These keep native Git's failures (a held lock is waited for first,
  but the stash of a native merge commit and the late ref
   update are native Git's).
- A merge or sequencer operation already in progress (native Git reports it).
- A signer that fails when the merge is signed (nothing is changed).
- A repository or index the wrapper cannot read (the failure names the file).
- A `git pull` that needs a merge and is not read exactly:
  it runs its merge inside Git,
  out of the wrapper's reach.

### Word-level second merge pass

Git's line merge calls two changes that touch a conflict:
a line you edited and lines the other side inserted right after it,
 for example,
in a paragraph that is hard-wrapped one sentence to a line.
When a `git merge`,
`git rebase`,
`git cherry-pick`,
`git revert` or `git pull` stops on such a conflict,
the wrapper re-merges only the
 conflicted parts word by word (the way Jujutsu does with `merge.hunk-level = "word"`),
 and accepts a file only when every part of it resolves.
A file that resolves is written to the worktree and staged as resolved.
When every conflicted file resolves,
the operation is finished the way Git finishes it after a clean run (through the wrapper's
 own command path,
so the commit policies apply):
a merge commits with its prepared message,
a cherry-pick or revert commits and
 continues,
a rebase continues,
and the exit status is the finishing command's.
`--no-commit` and `--squash` stop as they would after a clean run,
with Git's line "Automatic merge went well;
stopped before
 committing as requested" and status 0,
and `--edit` opens the editor.
`git pull` is covered,
both its merge and its rebase;
`--autostash`,
`merge.autoStash`,
`rebase.autoStash` and `branch.<name>.mergeOptions` are applied
 as Git applies them;
a command with an option the wrapper does not read exactly is left stopped with the files staged and
 Git's status.
What Git printed when it stopped (`CONFLICT` lines) is already out and stays.
An autostash (`--autostash`,
`merge.autoStash`,
or a pull's) is applied by the commit that ends the finished merge,
 as in Git.
When the stash conflicts with the merged files,
the merge commit is still made,
the stash is kept in your repository as
 `stash@{0}: autostash`,
the files it conflicted in are unmerged in your index with the markers in the worktree,
 and the exit status is the commit's,
as after Git's own apply.
The same holds when you conclude such a merge yourself with `git commit`,
 and when you have edited a file the stash touches
 since the merge stopped (Git's apply then refuses and keeps the whole stash).
Standard error lists every part that was merged,
with the file and the lines in the base,
your side and the other side,
 and the text it became:

```text
monochromatic-git-policy-cli: word-level merge: notes.md: part 1 of 1: base line 3, ours line 3, theirs lines 3 to 4 became "Flags are optional, see below.\nExit code is zero on success.\n"
```

The journal gets one entry per merged file (`journalctl --identifier=monochromatic-git-policy-cli`),
with the fields
 `CLI_GIT_WORD_MERGE_FILE`,
`CLI_GIT_WORD_MERGE_OPERATION` and `CLI_GIT_WORD_MERGE_PARTS`.
Only conflicts the command itself created are touched;
a file you are resolving by hand is never replaced.
A file that does not resolve stays exactly as Git left it.
That includes two sides that append different text at the same place,
two different edits of the same word,
 a file with a NUL byte in its first 8000 bytes (Git's own test),
a file over 8 MiB,
a path added on both sides or deleted on one,
 a link,
a path whose mode differs between the sides,
and a path whose `merge` attribute names a driver,
`binary` or `-merge`.

The merge is Git's own line merge:
`git merge-file --zdiff3` over the base,
your side and the other side,
so the lines both sides wrote identically stay out of a conflict,
and the record of each file says which line regions the word pass settled.
The owner on 2026-11-10:
"there are still ways you haven't tried to make this happen less often,
for example,
using a better algorithm or something."
Measured on 723 files that both sides of a two-parent merge changed and that plain Git conflicts on,
from 3,194 merges of this repository,
`--zdiff3` resolved 130 of them where the pass's own line merge resolved 108,
and Git's `patience`,
`histogram` and `minimal` algorithms resolved the same 130 to 129.

A region whose merge would write more than a resolution keeps both sides instead:
a word written twice in a row that neither side writes twice
(merge `ef43afe9f9` shipped `return return without_variable == *selected;` that way),
and,
where the region spans more than one line on some side,
a line neither side holds
(the shape that ran one body into the other mid-sentence).
A file with such a region stays exactly as Git left it and you resolve it,
which is what the measured set shows:
the refusals took both files that wrote a doubled word
and the 71 lines no side holds that 34 resolved files had joined across lines.

The result can be text neither side wrote:
two edits of different arguments of one call become one call with both edits.
Nothing either side added or kept is dropped,
and the parts are listed so you can check them.
The wrapper's own merges from objects (the merge of a staged-index checkout above,
the merge of an auto-push that lost a race,
 and the replay of a commit another commit beat to the branch) use the same pass and publish or commit a merge only when every
 conflicted file resolves.
They read no worktree,
so they read the `.gitattributes` of the branch tip (`GIT_ATTR_SOURCE`) unless you named an attribute tree yourself
 (`--attr-source`,
`GIT_ATTR_SOURCE` or `attr.tree`),
and the `union` line for `doc/handover/*.md` applies in them too.
Design and measurements:
`doc/planning/cli-git-merge-auto-resolution.md`;
the rules:
`SPEC.md`,
"Word-level second merge pass".

### A commit held behind a reference lock

The owner,
asked what a commit should do when a killed Git's branch lock is still there (2026-11-09),
chose
"Make it,
hold until lock goes (Recommended)":
the commit is written and kept on a side ref,
the command exits `0`,
says the branch has not moved yet and names the lock,
and the next command that finds the lock gone moves the branch.
This is the answer to "Commits should never fail."
No reference lock is removed,
by this or by anything else,
because no proof ties one to a Git that has ended.

A commit that meets a lock nothing proves gone is therefore made and kept,
on `refs/worktree/cli-git-service/<batch>` (a commit real Git built included,
so no maintenance that drops objects no reference names can take it while the lock stands),
and the command exits `0` with one line that names the branch,
the lock and the exact `rm -- <path>` to run when no Git process is using the repository.
Nothing of the person's is lost:
the captured change is the commit's,
the worktree keeps its bytes,
and the index catches up when the branch moves.

While a commit is held,
every command in that worktree says so on standard error:
one heading with the number of held commits,
then one line for each,
naming the branch,
the commit and the lock it waits for.
Held commits never collect without that line,
and a command whose output a program reads prints none of it.

A commit the service made and held whose answer was lost ends the same way,
with the same words and exit `0`,
found through the record of the hold rather than the branch,
which the lock keeps from showing it.

`git worktree remove` or `git worktree prune` deletes a linked worktree's own Git directory,
and the per-worktree reference with it,
so a held commit there is kept on `refs/cli-git/kept/<batch>` before Git runs,
and the command says then that the branch stays where it is,
because no command in a removed worktree can move it.

The first command that finds the lock gone moves the branch,
by one compare-and-swap from the value the batch was prepared on.
The batch's index entries are installed under Git's index lock,
taken before the move,
so nothing another program staged is overwritten,
and the record of the hold is removed once the branch and the index show the commit.
If the index lock is another program's at that instant,
the entries are left to the record of what the batch owes,
which the next commit in the worktree settles first (`SPEC.md`,
"Recovery").

The branch can move while the lock stands.
The commit then lands on the branch as it is now,
with the rule a commit that lost a landing race already follows
(`SPEC.md`,
"Retry after a moved branch"):
its change is merged onto the current tip from the base it was prepared on,
and the commit is written again with its own authorship,
committer and message,
its one parent the tip.
The other session's work stays.
A change that does not combine with the branch as it is now is not landed:
the record stays,
the commit stays on its side ref,
and the line names the paths that conflict,
so nothing of either session is discarded.

Three states are kept and named instead of carried over,
because a command that resumes a commit cannot do what a command that makes one can:

- a commit that concludes a merge,
  whose other parents cannot be kept on another head;
- a commit prepared for a branch that did not exist and that another session created meanwhile;
- a signed commit,
  which would have to be signed again by a program that may ask for a key,
  and no command starts a wait like that on its own.

Nothing released such a commit's push:
the command that made it had already ended,
so no post-commit gate ran,
and the line says so.
The next commit or push from that branch carries the commit,
and the held push notice says which commit waits for that.

One state still ends such a commit with `1`:
the record of the hold cannot be written,
so the commit cannot be made durable.
The message then names the lock,
what is known about its holder,
and the exact `rm -- <path>` that clears it,
and says that nothing in the repository changed.

### What can still end a commit with a nonzero status

The owner's rule (2026-11-03):
"Commits should never fail."
A named-path commit beside another session's unfinished merge or
stopped cherry-pick lands as an ordinary commit and the state is carried on (see `SPEC.md`,
"Merges and commits never fail").
Each of these still ends a commit with a nonzero status,
with its cause;
none is waited out or retried on the moved head yet:

- A Git step the client starts shows no sign of work for its quiet window (`child_class.rs`:
  60 seconds for a quick
  query step,
   300 seconds for a bulk step;
  both unmeasured),
  and is quiet again when run a second time.
  A step that works is
   never ended
   for being slow,
  and a step that may not be run twice (a reference update) is waited for,
  with no end of its own:
  only you end it.
- The commit service does not start:
  a start the manager reports as under way is waited for until `START_CEILING` (60
  seconds),
   a start it reports failed ends the wait at once ("commit service did not start"),
   and a unit it reports inactive is asked
   to start after `START_WAIT`.
  Two causes end the commit at once instead of after that wait,
  because the command knows them before it waits,
  and each message names the setting that has to change and every way to change it:
  a runtime directory that leaves no room for the service's address
  (that address takes at most 107 bytes,
  so `XDG_RUNTIME_DIR` may take 40),
  and a service the account's manager started at another address than the one this command waits at,
  which happens when `XDG_RUNTIME_DIR` is not the runtime directory the manager runs with
  (a unit,
  a container or a script with one of its own):
  run the commit from your login session,
  or set `XDG_RUNTIME_DIR` to the manager's.
  An account without user services is still told `loginctl enable-linger`,
  which is also what a session bus with no user manager behind it says.
  The commit service remains the only writer of a commit,
  so a command that cannot reach it changes nothing and stops.
- The service does not answer within `ANSWER_CEILING` (20 minutes,
  heartbeats or not):
  the client withdraws the request,
  so it
   never lands later by itself,
  and ends with "This commit waited N minutes for the commit service and was withdrawn";
   if the
   commit landed at that moment the command reports success.
   Five seconds without a word ask the service again with the same
   commit,
  and a service that does not answer then ends the command with "The commit service stopped answering".
- The branch keeps changing:
  `LANDING_ATTEMPTS` (3,
  `service_land.rs`) landings lost to other commits end with
   `BranchKeepsChanging`,
  and a commit built on a moved branch is built again for as long as the time since its first build is under
  `MOVED_BRANCH_REBUILD_CEILING` (20 minutes,
  `service/wait_ceilings.rs`).
- A commit that concludes a merge,
  an amend or a revert cannot replay onto a branch that moved (`concurrent-commit/head-moved`).
- `index.lock` is held by a program that does not let go within `LANDING_LOCK_CEILING` (1 minute;
  commits queued behind the same lock are refused at once),
  or holds a Git lock PID file naming a number this process table does not have
  (a killed Git leaves one,
  and so does a live Git in another PID namespace,
  so the commit waits and the line says which number was read).
  The commit is withdrawn,
   kept on a side ref named
   in the message (`refs/cli-git/kept/<batch>`),
  your staged changes stay staged,
  and nothing lands later by itself;
   run the
   commit again.
- The record of a held commit cannot be written,
  so a commit whose branch write a reference lock refuses cannot be made durable
  (see "A commit held behind a reference lock").
  A reference lock that nothing proves gone is otherwise no longer one of these.
- The content a commit captured was pruned from the object store before it landed (`ClientFailure::ContentPruned`).
- A named path an ignore rule matches while no index entry tracks it.
  The commit is not made,
   nothing is staged by it,
   and the finding `explicit-path-commit/named-path-ignored` names every rejected path
   with both ways forward:
   leave the ignored generated file out of the commit's paths,
   or stage it with `git add --force -- <path>` when it is meant to become tracked (`SPEC.md`,
  "Core finding event").
  The decision comes from the paths the command names,
   before any private `git add` runs,
   so it no longer depends on the wording of a Git message (issue 6278).
- A pathspec the command names that matches no path Git can stage,
   or one Git read from the pathspec file the command names.
  Native Git keeps deciding which pathspecs are valid,
   so the wrapper reports Git's own refusal instead of predicting a match:
   a directory,
   a pattern,
   a magic selector and a literal name behave exactly as before.
  The commit is not made,
   nothing is staged by it,
   the finding `explicit-path-commit/pathspec-matches-nothing` names every refused token as Git named it,
   and a token that begins with a hyphen also says that options belong before `--`
   (`git commit --message <text> -- <paths>`).
  The staging start whose words this reads runs under the C locale,
   so the finding appears whatever language the person's shell asks for,
   and a token that holds Git's closing wording is named whole.
  Every other failure of that staging start keeps its engine-failure report (`SPEC.md`,
  "Core finding event";
  issue 6283).
  The twenty commits that stopped inside that staging on 2026-11-10 are measured in
  `doc/planning/cli-git-commits-that-stopped-after-their-add-steps.md`.
- A policy finding at error severity,
  an empty message or a failing signer.
  A named path in conflict is no longer one of these:
  it is committed with the markers it holds and a warning that names it (owner,
  2026-11-03:
  "Committing a conflicted path is fine.
  Warn and commit anyway,
  even if conflict markers will be committed.").
  A plain `git commit` with no paths while files are still unmerged finishes the merge the same way,
  with the markers in those files and the same warning for each (owner,
  2026-11-03:
  "Warn and finish the merge (Recommended)");
  Git's "Committing is not possible because you have unmerged files" no longer ends it.
  `git commit -a` over unmerged files commits their markers and warns for each file in the same words,
  when the repository's commit-only policy is off:
  with it on,
  `-a` is refused before anything is committed (`commit-only/all-flag`).

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
  A landed commit that a post-commit policy then blocks exits `2` and stays local:
  it stays local past later commits of the same branch,
   because the automatic push of that branch stops before it and
   every command says which commit holds the branch back and which check refused it.

The wrapper's own diagnostic lines on standard error (`[debug]` lines under `MONOCHROMATIC_VERBOSE=true`,
 and `[warn]` lines unless `MONOCHROMATIC_WARN=false`) often carry Git's words,
which can name a remote.
 The userinfo of any URL in such a line is replaced by `<credentials>`,
and every control character by a space.

## Configuration

The wrapper reads `monochromatic-git-policy-cli.config.jsonc` at the repository top level.
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
 `git monochromatic-git-policy-cli check` reports a `legacy-config-ignored` configuration warning that names the file,
 and every other command and `fix` stay silent about it;
 either way those files are ignored.
The keys `plugins` and `trust` are refused.

```jsonc
// monochromatic-git-policy-cli.config.jsonc
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
  A `git add` must name the paths it stages.
  Bulk flags (`-A`,
  `--all`,
  `-u`,
  `--update`) are refused from the arguments alone,
  and a positional pathspec is refused when Git finds it selects every path of the repository or of
   the directory the command runs in,
   unless the spelling names a character of a name (see `SPEC.md`,
  "Bulk staging").
- `final-newline`:
   default `warn`.
  Text files end with exactly one line feed.
  A file whose Git `text` attribute is unset (`-text`,
  which `binary` includes) keeps its exact bytes,
  so a package declares its exact-byte inputs,
  such as fuzz seeds,
  with `seed/** -text` in its own `.gitattributes`.
  The attribute is read from the commit or index being checked,
  never from the worktree.
  Files below `dist/final/node/` or `bundle/node/` are also exempt,
  as generated output.
- `markdown/autofix`:
   default `warn`.
  Runs the linked `monochromatic-lint` fix engine over staged Markdown inside the commit,
  in-process on a guarded thread.
  Options:
   `rules` (only `lfs-image-url` exists) and `exclude` (gitignore-style patterns).
- `mono/forbidden-root-context`:
   default `error`.
  No `CONTEXT.md` at the repository top level enters a commit.
- `mono/dependent-version-bump`:
   default `error`.
  Packages that depend on a package with a bumped version follow the bump.
  A wrapped `git commit` of a hand-raised manifest carries its dependents' patch bumps in the same commit,
   through the whole chain of dependents,
   and brings their worktree copies to the committed bytes
   while those copies still match `HEAD`.
  A dependent whose worktree or index copy holds something else is left out of the commit and named on standard error,
   with the reason and both remedies,
   so the commit lands with the rest and nothing is overwritten.
  A publish set the commit cannot read,
   or a dependent whose version cannot be bumped automatically,
   is named the same way;
   a commit never fails over the ripple.
  A real index the commit could not list is named the same way,
   once for each dependent whose copies that listing would have proved,
   so a failed lookup never reads as an index that holds nothing.
  `git monochromatic-git-policy-cli check` and
   `git monochromatic-git-policy-cli fix` report the same ripple for the worktree.
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
  This repository's `monochromatic-git-policy-cli.config.jsonc` does not list it yet,
   so the policy is off here.
- `security/forbidden-strings`:
   default `error`.
  Scans candidate bytes and path names with the linked scanner.
  Options:
   `builtinRules` (boolean,
   load the scanner's built-in baseline) and `rulesFile` (a rules file path relative
  to the repository top level,
  or an array of them).
  The rules files are `rulesFile`,
   else the environment variable `FORBIDDEN_STRINGS_RULES`,
  else `forbidden-strings.local.txt` in the repository top level,
   which may be absent.
  A file named by `rulesFile` or by the variable must exist,
  unless it is marked optional.
  Several files are read in order and scanned as one rule set,
  and every one of them is left out of the scan,
  as the single rules file always was.

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
A configured string is one path exactly as written,
so a comma or a `?` in it needs no care;
an array never needs the variable's list syntax.
An empty array,
an element that is neither a string nor an object with `path`,
an object key other than `path` and
`optional`,
and a path named twice are configuration failures.
Every path is checked the same way:
relative to the repository top level,
no `..`,
no backslash.
`FORBIDDEN_STRINGS_RULES` uses the scanner's comma-separated list (see
`package/forbidden-strings/cli/README.md`,
"Rules list"),
where a trailing `?` marks an optional entry;
a value that is not valid UTF-8 is one required path.
Because the variable's entries are resolved against the repository top level too,
the same `{{config_root}}`-anchored value serves the scanner and monochromatic-git-policy-cli.
An invalid variable fails the policy only when a candidate is scanned,
like a missing file.
The new-worktree copy takes every configured file that Git ignores and the main checkout has.

#### Content already in history is not scanned again

`security/forbidden-strings` reads the bytes of a candidate
only when no parent commit's tree already holds that exact blob.
A pure rename,
a copy of an existing file,
and a merge that brings a file only one parent has therefore pass,
because a blob's object name is the hash of its bytes and the bytes are already in history.
What counts as a parent depends on the command:

- `git add`,
  and a commit being made:
  the commit the new commit is based on (the recorded base,
  else `HEAD`).
- The post-commit pass over a landed commit:
  every parent of that commit.
- The pre-push pass:
  every parent of a pushed commit that is not itself being pushed,
   so a parent that is part of the push vouches for nothing.
   A push that includes a tip with no known published history skips nothing.
- `git monochromatic-git-policy-cli check`,
  a root commit and an unborn branch have no parent tree,
  so everything is scanned.

The comparison is by exact blob object name per path,
never by path name or by similarity,
so one changed byte makes the file new and it is scanned in full.
The pathname of a skipped file is still scanned,
because a rename gives the content a new name.
A file that a fix corrected during the same invocation is never skipped,
because its bytes are no longer its object's.
If the parent trees cannot be listed,
every candidate is scanned in full and a warning says why.

Each skipped file is recorded on standard error as a debug line when `MONOCHROMATIC_VERBOSE=true`:
the masked pathname,
the blob's object name,
and the commit whose tree holds it.

Tradeoffs.
Content that reached a parent through an earlier bypass,
or that sits at a path the scanner never reads
(the rules file and the scanner's own rule sources),
stays unscanned wherever it is copied or moved to afterwards.
That is also what lets the scanner's own package move to a new directory.
A rule added to the scanner after a file entered history is applied to that file again only when its bytes change.

A `warn` setting reports findings without blocking.
A policy whose protection `warn` would remove produces a configuration warning event when the file sets it to `warn`.

### Discarded copies

- `discardedCopies.maxAgeDays`:
   positive integer,
   default `30`.
  Saves of conflicted files older than this many days are removed when a new save is written.
- `discardedCopies.maxSize`:
   a number of bytes above zero or a string such as `"256m"`
   (`k`,
  `m` and `g`,
  powers of 1024,
  as Git's integer settings),
   default `"1g"`.
  While the folder holds more than this,
  the oldest saves are removed.

Neither limit reaches a save younger than 24 hours.
Zero,
a negative or fractional number,
and text that is not a size are configuration failures that name the key.
See "Discarding commands" for the pass.

### Concurrency

- `hooks.concurrentCommits`:
   boolean.
  Inert:
  it tuned the hook lock,
  and the wrapper runs no external hook,
  so there is no lock.
  Older files that set it still load;
  the value is checked and ignored,
  and `git monochromatic-git-policy-cli doctor` says so.
- `indexLock.unprovenOwnerTimeoutMs`:
   non-negative integer,
   default `1000`.
  Inert:
  it was how long the wrapper backed off for an `index.lock` whose owner is dead or unproven,
   and nothing reads it now (the commit service waits for a foreign lock until `LANDING_LOCK_CEILING`,
  proven alive or not).
  Older files that set it still load;
  the value is checked and ignored,
   and `git monochromatic-git-policy-cli doctor` names the key and asks for it to be removed.
- `landing.reserveAfterLostRaces`:
   positive integer,
   default `1`.
  Inert:
  it was how many lost landing races passed before a commit asked for the landing reservation,
   and nothing reads it now (the commit service lands in arrival order).
  Older files that set it still load;
  the value is checked and ignored,
   and `git monochromatic-git-policy-cli doctor` names the key and asks for it to be removed.

Concurrent commits cannot be turned off.

### Waiting and time limits

Nothing the wrapper waits for waits for ever (owner,
2026-11-03:
"nothing should ever wait forever or indefinitely"),
and the
wrapper never waits without saying so.
A wait with no bound of the wrapper's own is a scratch `git commit` the kernel has taken:
once the ceiling's `SIGKILL` reaches a process in uninterruptible sleep it cannot act on it,
and the commit waits until the kernel releases the step,
saying so once and again while that wait lasts
(owner,
2026-11-10,
"Keep waiting and say so").
Every bound is in `src/native/service/wait_ceilings.rs` with its measurement,
and the list of
every wait is `doc/planning/cli-git-no-unbounded-waits.md`.
A process that works is waited for until its ceiling;
a quiet window ends a wait for quiet,
a ceiling ends a wait for length,
and
the one step that is never ended is a write in place that is not safe to run twice.
A Git step the wrapper starts keeps a ceiling of its class (reads 120 seconds,
bulk steps 10 minutes) exactly when it can run twice:
the owner's criterion,
decided by cutting each step at every mutating system call,
running it again and comparing the end state
(`doc/planning/cli-git-no-unbounded-waits.md`,
"The cut experiment").
The branch write (`git update-ref`),
`git merge`,
`git reset --hard`,
`git checkout`,
`git switch`,
`git stash store` and `git read-tree -m` came out different and are never ended
(the owner,
2026-11-04:
"Never cut a branch write (Recommended)",
whose option read "Steps that can safely run twice keep their
ceilings;
the branch write is never ended.
Your command still returns at its bound with "result not yet known" and the `git log -1`
and side-ref hint.
Con:
a truly hung write keeps the service busy,
and later commits fail at their own 20-minute ceiling with an
error naming it.").
A command that started such a step stops waiting at the step's class bound plus 7 seconds (`SHORT_EXTRA_WAIT`;
the owner's
"Class ceiling + 7 s (Recommended)" and "7 s (Recommended)") and exits non-zero saying the result is not known,
naming the step and
what to check;
the step is left running,
with its output in files that outlive the command.
Only the exact forms the cut experiment
ran (the table in `doc/planning/cli-git-no-unbounded-waits.md`) keep a ceiling;
any other form of a writing step has none.
No `git update-ref`
keeps one (a reflog another Git creates between a check and the write would be appended to twice),
and `git fast-import` has none.
The commit service's landing thread waits for it,
and a commit that waits behind it is told what the service is stuck in after a
minute (`NEVER_ENDED_STEP_SLOW_AFTER`) and is withdrawn at its own `ANSWER_CEILING` with an error that names the write and the way out
(`journalctl`,
and a restart of the service when no program uses the repository).
A commit whose own batch is inside a write is told sooner,
because its commit may be on the branch already:
the service sends one line,
`this commit is inside a write of the commit service`,
as soon as the write starts
(the owner chose this on 2026-11-04,
"Short extra wait").
That line is a normal part of a landing that outlasts a beat and not a failure:
the command goes on waiting and then says whether the commit was made.
The commit inside the stuck write waits 7 seconds
past its `ANSWER_CEILING` and then reports that it is not known whether it was made,
with `git log -1`,
`git status` and the write
named;
the commit lands when the write returns.
A lock holder is waited for until `LANDING_LOCK_CEILING` (1 minute),
the commit service's answer until `ANSWER_CEILING` (20 minutes),
a start until `START_CEILING` (60 seconds),
the service's hold for a forwarded `git add` until `SERVICE_HOLD_CEILING`
(10 minutes),
and a push whose connection is open and silent until `CONNECTED_PUSH_QUIET_WINDOW` (150 seconds,
then the
service's retry plan keeps it).
At a bound on the commit path the request is withdrawn,
the commit is kept on a side ref,
and the command exits non-zero naming the blocker.
A service that goes silent for five seconds is asked again.
Where no
inotify watch can be set,
the waits poll every 50 ms. These were chosen by the owner's answers of 2026-11-03 and the
coordinator's reading under the delegation,
and are open to the owner's veto.
The environment variables below set the limits and the pace of the notices,
each a positive whole number of seconds;
a zero,
negative,
fractional or non-numeric value is ignored.

- `CLI_GIT_CHILD_WAIT_SECONDS`,
   default `60` for a quick query step and `300` for a bulk step.
  How long one Git process that the wrapper starts for itself may show no sign of work before it is ended.
  A step that is slow and works (bytes moved,
  processor time,
  a task running or waiting for the disk) is never ended
  for being
   slow,
  however long it runs.
  The two defaults are not measured:
  they were wall-clock limits,
  no log held a quiet gap,
  and the timing records now
  keep the
   longest quiet gap of every step (`longestQuietMs` and `quietMs` in `monochromatic-git-policy-cli-commit-timing.jsonl`) so a measured
   number can
   replace them.
  A quick query step only reads refs,
  commits and trees (`rev-parse`,
  `rev-list`,
  `ls-tree`,
  `cat-file`,
  `diff-tree`,
   `merge-base`,
  `write-tree`,
  `update-ref` and the like).
  A bulk step is every other step:
   it moves bytes (`pack-objects`,
  `index-pack`,
  `add`,
  `maintenance run`),
   or compares the index with the worktree (`reset`,
  `update-index`,
  `ls-files`,
  `diff`),
   or is not known to be a query.
  The object reader (`git cat-file --batch`) is a bulk step too,
  because one request can name a large blob.
  Setting the variable gives every class the same window.
  A step that is safe to run twice and shows no sign of work for the window is ended and run once more;
   if it is quiet again,
  the command reports which step,
  its class,
  the window and how to go on.
  A step that cannot be run twice (a branch write above all) is announced and has no ceiling:
  the commit service waits for it,
  and
  any other command stops waiting for it at its bound plus 7 seconds and says the result is not known.
  No hook of yours runs below a step the wrapper starts,
  and the stop never reaches a process beside the step.
- The tools and checks that are not Git end on the same rule,
  with windows that are not measured:
   a package-manager tool of the worktree repair after 600 seconds with no sign of work (with its whole process group),
   the Markdown linter after 60 seconds in which its thread shows none,
   and the standards-review call after 10 seconds of silence on its connection (30 seconds for the script's default).
  Each also has a ceiling on its whole run,
  whatever it shows (owner,
  2026-11-04:
  "Silence plus a ceiling (Recommended)",
  which
   read "Keep ending on silence,
  and add a generous overall ceiling per tool,
  measured from real runs,
  after which it's ended with
   an error naming the tool.
  Con:
  an extremely slow but working run on a huge repository can hit the ceiling."):
   18 minutes for one repair tool (`REPAIR_TOOL_CEILING`,
  ended with its whole process group,
  and the line names the tool,
  what it
   was doing and the command to run by hand),
  4 minutes for the Markdown check of one file (`LINT_CEILING`,
  abandoned as a quiet one
   is),
  1 minute for the standards-review call (`REVIEW_CALL_CEILING`) and 2 minutes for the whole review of a commit
   (`REVIEW_CEILING`,
  with the command `SHADOW_REVIEW_SHA=<commit> mise run shadow-review` that reviews it again).
  Each ceiling is the smallest whole number of minutes that is at least five times the longest run measured on this host
   (`src/native/service/wait_ceilings.rs` has the figures and the method).
  The ceilings are not variables you can set.
- `CLI_GIT_WAIT_NOTICE_SECONDS`,
   default `10`.
  How often a waiting command repeats its notice (see "What a waiting command prints"),
   and how often the commit service repeats the line that names the holder of a foreign `index.lock`.
- `CLI_GIT_PUSH_DEADLINE_SECONDS`,
   default `30`.
  How long the push after a landed commit may show no sign of work and hold no open connection (the stall window)
   before it is stopped.
  The 30 seconds has no measurement or owner decision behind it (the audit keeps it as an open question).
  The push and every process it started are watched:
  one that moves bytes (on its pipes and files,
  and on its connected TCP
   sockets),
  uses processor time,
  or has a task able to run or waiting for the disk (an `fsync` on a slow volume) is working
   and is not stopped here,
  however slow;
  one that does none of these is stopped after this many seconds.
  A run whose connection to the remote is open (the push,
  the fetch that follows a refused push,
  and the Git LFS upload)
   is waiting for the remote and is ended only when it shows no sign of work for `CONNECTED_PUSH_QUIET_WINDOW` (150
   seconds,
   unmeasured);
  the service's retry plan keeps the push and tries it again.
  The owner's answer of 2026-11-03,
  "Bound it,
  keep retrying (Recommended)",
  replaced "Wait while connected (Recommended)".
  The wrapper says so once,
  in a line that starts with `monochromatic-git-policy-cli: git push is waiting for the server`
   (`git fetch` or `git-lfs push` for those runs),
  and waits until the remote answers or the connection closes,
   or until the connection stays open with nothing moving for that window,
  when the run is stopped as a stall.
  "Connected" means that a process of the run holds a TCP socket in state `ESTABLISHED`,
  read from the kernel.
  A connection that dies without a reset is noticed only when TCP gives up;
  the wrapper sets no time for it.
  The wrapper adds no low-speed abort of its own to any run (it used to set `GIT_HTTP_LOW_SPEED_LIMIT=1000` and
   `GIT_HTTP_LOW_SPEED_TIME=60`,
  which ended a silent check after 60 seconds):
  Git sets neither,
  and a limit and time you set
   in the environment or in `http.lowSpeedLimit` and `http.lowSpeedTime` still reach Git and still end the wait.
  A push to a local path has no connection to wait on and is stopped after this many seconds,
  as before.
  The name is kept from the time when this was a fixed deadline,
  so an existing setting now sets the stall window.
  Where the system cannot count bytes (no `/proc/<pid>/io`,
  as on macOS),
   it is a fixed deadline for the whole push,
  as before.
  The commit stays saved,
   the failure is reported,
   and `git push` can be run by hand.
  Git's own read-only queries before the push are ended by the same stall window when they show no sign of work;
   a query that works is never ended for being slow.
- `CLI_GIT_PUSH_RETRY_SECONDS`,
   default `10`.
  The pause before the commit service tries a stopped or failed auto-push again on its own.
  A failure that can pass is tried again,
  with the pause doubled each time up to 600 seconds,
   until the push lands;
  no new commit is needed,
  and the service's journal says each time that it will try again
   (cause,
  number of the try,
  pause).
  Being offline,
  a stall,
  an HTTP 5xx answer of the LFS server and a held lock can pass.
  The server's own protected-branch refusal passes the same way,
  for a fixed number of tries:
  GitHub answers one with
   `GH006: Protected branch update failed for <ref>`,
  and such a refusal cleared by itself three times in this repository,
  so a push answered with it is given five tries in all
   (a try a stall or an offline failure spent counts too),
  and with the default first pause the last of those tries comes two and a half minutes after the refusal.
  That is the owner's decision of 2026-11-11,
   "Make the wrapper auto retry on GH006";
  GitHub's words for a refused update are what make it that refusal
   (the code family alone also covers other refusals such as one of a deletion),
  since the reason beside it
   (`protected branch hook declined`) is where any server writes words of its own.
  A refusal that keeps coming back ends after that fifth try
   (an unsigned commit on a branch that requires signed commits is one)
  with a record that names what GitHub said,
  how many tries the service gave it and how long they covered,
  and both ways out.
   One of them is to change the commit or the rule and commit again;
  the other is to push by hand.
  Three failures stop the retry until the person's next commit
   (owner decision,
  2026-11-03:
  "Offline is retried,
  but auth and server hook declines stop until your next commit."):
   a reported conflict,
   a login the service does not have (Git's "Authentication failed",
  "terminal prompts disabled",
  an HTTP 401 or 403,
  or
   SSH's "Permission denied ("),
   and a hook on the server that declined the push (a `[remote rejected]` line whose reason ends in "hook declined",
  with no protected-branch code anywhere in the push's output).
  The failure record then starts with plain words that say which one it was,
  above Git's own words
   (for a decline,
  the message the server's hook printed),
  and for a decline it also says that a push by hand may succeed once the cause has cleared:
  a decline is the server's answer at that moment,
  never a promise about a later try.
  A rejection by the server whose reason is none of these is tried again,
  and the journal names its line.
  The service never asks anyone for a login.
  The Git and `git-lfs` children it starts (the push,
  the fetch of a reconcile,
  the LFS upload,
  upkeep,
  landing) run with
   `GIT_ASKPASS` set to the empty string and `GIT_TERMINAL_PROMPT=0`,
  because Git and `git-lfs` try `GIT_ASKPASS`,
  `core.askPass`
   and `SSH_ASKPASS` before they read `GIT_TERMINAL_PROMPT`,
  and the manager that starts the service can carry
   `SSH_ASKPASS=/usr/bin/ksshaskpass` and a display.
  A push whose credential helper gives no answer therefore fails at once with "terminal prompts disabled",
  which is the login
   failure above,
  and no desktop password dialog opens for nobody to answer.
  `SSH_ASKPASS` stays in the environment for OpenSSH's key passphrases,
  and a command typed at a terminal keeps Git's own prompting
   while one whose process has no controlling terminal gets the same pair,
  because nobody can answer it.
  The evidence is in [the askpass troubleshooting note][askpass-dialog].
  The failure record the next command shows says that the service keeps trying,
  and goes when the push lands.
  The plan survives a restart of the service:
  it is kept on disk,
  one small file per branch,
  in the service instance's own
   journal (`push-retries`),
  and the service that starts next rebuilds it and tries again with no wrapper command.
  The record the next command shows is not the plan,
  since showing it removes it.
  A pause that was running when the service stopped starts again from the restart,
  and the try number is kept.
- There is no ceiling on a push,
  and no variable sets one:
   no total time ends a push that shows progress,
  and a push that holds an open connection is ended only by the connected window,
  which no variable changes.
  The report of a stopped push names the limit that was hit:
  `limit=stall`,
  or `limit=connected` when the connection to the remote was what was held open,
  the bytes the push moved and the seconds it ran.
  A stopped Git process is first asked to end with `SIGTERM`,
  which lets Git remove its own `.lock` files,
   and is killed only if it is still running after five seconds.
  A process that was killed,
  or that could not answer in time,
  can leave a lock file behind;
   once it is gone the wrapper judges the reference locks of the repository it worked in and removes only those whose holder
   is proven gone (the rules are in the push deadline entry of `SPEC.md`),
   so a later `git fetch` or `git commit` does not fail with "Unable to create ... File exists".
  A lock whose pid file names a process of the stopped tree is removed on the wrapper's own knowledge that it stopped that
   process,
  also inside a container,
  where the general proof is not available.
  A lock it cannot prove stale,
  such as an empty lock without a pid file,
  is left,
  because a live Git of another program holds
   exactly such a lock for as long as its transaction lasts;
  the wrapper says what it found and prints the command that clears it.
  The same stop applies to a Git step that shows no sign of work for `CLI_GIT_CHILD_WAIT_SECONDS`.
- A lock that a Git killed in any other way left (a closed terminal,
  the out-of-memory killer,
  a reboot,
  a release before
  0.5.23) is judged before the wrapper's own fetch and push and before your plain `git fetch` and `git push`.
  It is removed only when its holder is proven gone:
  the process its pid file names has ended,
  or its number now belongs to a
  program that is not Git,
  and no process of a nested PID namespace has that number as its own;
  and never while a process has
  it open or a running Git is named.
  The age of a lock by the clock is not a proof,
  because a correction of the clock moves it:
  a lock left by a reboot is removed
  when it has a pid file,
  and otherwise left with the command that clears it.
  Git writes no process number into a lock,
  and a live Git holds its reference locks with no file open,
  so nothing else can
  prove a lock stale.
  A lock that is not proven is left,
  and the wrapper prints what it found and the exact `rm` command that clears it
  (`SPEC.md`,
  "Reference locks left by a Git that was killed").

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
 another process's push) prints its first line at once.
Each line names what the command waits for,
 the process that holds it when the wrapper can tell,
 how long the wait has lasted and where it gives up.

- An `index.lock` held by a live process,
  which the commit service waits for before it lands a commit:
   the holder is proven by an open descriptor or Git's PID file and the proof is named,
   and an `index.lock` that nothing proves held is said to be unproven while the wait lasts.
   The first line of a wait adds a hint,
   and the repeats stay short.
- A `git merge` that waits for the index lock of any holder before it changes anything:
   it says it has changed nothing yet,
  names the lock and the holder with its proof,
  and repeats.
- The commit service's own hold on the index,
  which a forwarded `git add` waits for.
- Another process's push of the same branch.
- A long run of `monochromatic-lint` for a Markdown file,
   which names the file and stops waiting for the linter only after 60 seconds in which its thread shows no sign of
   work,
   leaving its thread to end with the command.
   The guard contains a hang and a panic,
  not a stack overflow or an abort in the linter.
- The `git commit` a commit that Git must build runs in its scratch repository
   (a template,
  a clean-up mode,
  or `commit.gpgsign`):
   it says the command waits for that `git commit` (its process),
   says the commit is signed when `commit.gpgsign` or `-S` asks for it,
   names the signing program while one runs below it,
   and says when the step is stopped.
   When the message came with the command (`-m`,
  `-F`,
  `-C`) the step is stopped after `SCRATCH_COMMIT_STEP_CEILING` (16,332 ms,
  three times the slowest
  `git commit` step record that is not one of the three over 26 s,
  5,444 ms,
  over 907 records of releases 0.6.1-alpha.2 to 0.6.1-alpha.5;
  owner,
  2026-11-08,
  "About 16 seconds",
  after "About 3 s now" on the same day and "limit unless editor" on 2026-11-07;
  `CLI_GIT_SCRATCH_COMMIT_CEILING_SECONDS` sets another ceiling in whole seconds):
   everything below the command is ended,
   nothing is committed,
   the person's index and files are as they were,
   and the same command can be run again.
   The owner chose this knowing that on the data of 2026-11-07 it would have failed commits whose builds took 11 to 406 s;
   those builds were a retry index without stat data,
   which is fixed.
   On the log of 2026-11-08 it still refuses three commits,
   whose steps ran 26,256,
   36,651 and 171,383 ms,
   and the owner asked for each of them to be investigated.
   A step the kernel does not release is waited for with no bound:
   a process in uninterruptible sleep (state `D`,
   a stalled disk or a file system that stopped answering)
   cannot act on the `SIGKILL` until the call it is inside returns,
   so the wait that keeps waiting says,
   once and again for as long as it lasts,
   that the stop was sent and the kernel has not released the step,
   names that `git commit` and the process the kernel is holding
   (the step itself or a program it started),
   says that nothing was committed and that the person's index and files are as they were,
   and says the terminal stays there until the kernel lets go.
   It says it from the wait and never from the notice's thread,
   which a process at its thread limit never started.
   The owner chose that wait,
   over leaving after a bound and failing the commit
   (2026-11-10,
   "Keep waiting and say so"),
   as an exception to "nothing should ever wait forever or indefinitely";
   the write-up is `doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`.
   The line is said only of a process that still reports that state one look interval after the signal,
   so one that acts on its stop in that interval is never said to be held,
   a look that could not be made is made again rather than read as the kernel having let go,
   and a line the writer could not take is said again as soon as it can be.
   A commit that opens the person's editor prints nothing and has no ceiling.
- A commit whose answer takes longer than `WAITING_NOTICE_AFTER` (10 seconds,
  `service/wait_ceilings.rs`;
  twice the owner's bound of 5 s on what a commit may add to native Git,
  and reached by 26 of the 909 answer waits in this host's commit timing log of 2026-11-09):
  one line says where the commit stands,
  and it is said once for each place the service names,
  not on the repeat schedule above.
  When the service named the place of the commit's batch it reads `Waiting for N earlier commits.`,
  or `This commit is next.` for the first place;
  when it named none it reads "The commit service has not said where this commit stands yet;
  it has waited 12 seconds.",
  because the client cannot tell a request still in the queue from a landing waiting behind another
  program's `index.lock`,
  and may claim nothing about whether the landing began.
  A commit that lands within those 10 seconds prints nothing,
  and `CLI_GIT_STANDING_NOTICE_SECONDS` (a positive number of seconds) sets the wait for a control.
  The service names the place of a batch's first request only once that request has waited as long,
  so a client of an earlier release,
  which prints every place it is told,
  sees one only for a commit that waited.
  A place belongs to the connection that named it:
  a request sent again after its service went away starts with no place,
  so a stale one is never printed as this wait's.
  Every line is written by a thread of its own and dropped when the terminal stops taking lines,
  so a reader that stopped draining costs the commit no more than `WARNING_WRITE_CEILING` (250 ms) at its end
  and never its answer.
- A commit whose own request is inside the commit service's branch write,
  which the service never ends in the middle:
  one line says so,
  names the write (`git update-ref --stdin`) and what it does (the reference it moves),
  and says that a write the service does not end is why this commit may be on the branch before its answer.
  The service says it as the write begins,
  because a commit whose wait reaches its own deadline inside such a write is not withdrawn,
  and the words wait for the same 10 seconds as the place above:
  a branch write took 5 ms at the median,
  601 ms at the 99th percentile and 4,785 ms at the 99.9th of the 6,703 landings this host's service journal
  recorded from 2026-11-01 to 2026-11-11,
  so a commit whose write is a normal one lands before a line could be read and says nothing.
  The words name no moment of the write for that reason;
  a write that has stayed stuck for a minute says so in the longer line that follows,
  at once,
  because its words changed.
  `CLI_GIT_STANDING_NOTICE_SECONDS` sets that wait too.
- A commit whose branch moved while it was being built:
   a commit Git built (a signed one,
   or one with a template or clean-up mode) is built again on the new tip,
   and a captured change is merged onto it again,
   for as long as the time since the first build is under `MOVED_BRANCH_REBUILD_CEILING` (20 minutes,
   `service/wait_ceilings.rs`);
   there is no count of builds,
   because a count ended a commit on 2026-11-08 that conflicted with nothing,
   with the change left uncommitted.
   Each rebuild says so on standard error,
   with the builds so far,
   the time so far and when the rebuilding stops.
   `CLI_GIT_REBUILD_CEILING_SECONDS` (a positive number of seconds) sets the ceiling for a control.
   A real conflict is still withdrawn with a plain message,
   and an amend,
   or a commit written or signed by hand,
   is still not moved onto another tip.
   A rebuild's private index is made from a copy of the real index,
   so the scratch `git commit` keeps the cached stat data of every entry the new tree shares with it
   and does not hash the work tree again (measured:
   233,618,538 bytes of LFS content cleaned and 3.6 s with an index built by a plain `read-tree`,
   against 0.1 s with the real index's stat data).
   An intent-to-add entry copies as it is,
   and the paths it names are written as the requested tree's own entries before the reset,
   so a scratch-commit hook that restages an unrelated file cannot leave one of them out of the tree the index writes
   (native Git's reset leaves the mark,
  and its tree then omits that path);
   an index that holds an assume-unchanged or skip-worktree mark,
   or one the wrapper cannot read,
   is built from the tree alone instead.
- The object reader and the Git steps of a commit that run long,
   which name the Git command and the process,
   and end it only when it shows no sign of work for the window of its class (`CLI_GIT_CHILD_WAIT_SECONDS`).
  The first notice of a quick query step comes after 5 seconds and that of a bulk step after 10 seconds,
   so a step that only needed a moment prints nothing.

```text
// standard error of a git add that waits for the commit service to finish a commit
monochromatic-git-policy-cli: git add is waiting for the commit service to finish a commit in this repository, 12 seconds so far;
this waits for at most 600 seconds in all, and then git add runs and reports what Git says.
```

#### Compiled rules are warmed off the commit path

The forbidden-strings rules are compiled once per text of the rules files and kept in a cache shared by every worktree
(`~/.cache/forbidden-strings/v<version>/<platform>/<digest of the rules text>/rules.bin`).
A commit that finds no compiled form compiles the rules itself in its prepare phase,
between 1.2 s and 1.3 s as measured on 2026-11-07 under the host's load at the time,
and about 20 s on the first commit of a fresh cache under heavier load.
The commit service therefore compiles them on its own time,
on the blocking pool and never on the landing thread:

- after the first landing it sees for a repository,
  and after a landing that changes a path of the rules,
- right after it starts,
  for the repository it remembered in its state directory (`rules-repository`),
  so a service restarted by an upgrade fills the new release's cache at once,
- when the client asks it to:
  after `merge`,
  `pull`,
  `checkout`,
  `switch`,
  `rebase` or `reset` exits 0,
  the client compares a digest of the rules text it would load before and after,
  and when it differs sends a one-way `warm` message to the service of that worktree that already runs.
  The client never starts a service or a process for this,
  and a command whose rules text did not change reads two small files and sends nothing.

The client still compiles on a cache miss,
so a commit that starts before the warm ends compiles as well (the two compiles race and the cache keeps one result).

A commit does not write that cache itself.
When the cache holds no compiled form of the rules text a command needs,
the command asks the service of that worktree to compile and publish it,
with the same one-way `warm` message a worktree-moving command sends,
and loads the rules without writing:
the rules it compiled are the ones it scans with,
and the artifact becomes the service's write rather than the command's.
The question is asked only when the cache holds no artifact for that text,
which one `stat` answers,
so a warm cache pays nothing beside the read it always made.

Two cases keep the write on the command's path,
because nothing else can make it:

- a machine where no service runs for that worktree,
  which is the first command after a boot or a fresh home;
- a rules list the command took from `FORBIDDEN_STRINGS_RULES` in a repository whose configuration names no
  rules files,
  since the service's own environment carries no such variable and it would compile the configuration's files
  (or the default appendix) instead.
  A configuration that names files wins over the variable,
  so a repository with both lists leaves them to the service.

The artifact itself is published without a file or directory sync:
a reader validates its magic,
scanner version,
platform,
source digest,
payload digest and engine digest,
so a copy a crash tore is rejected and recompiled,
and forcing the bytes to the device made the writer wait for a file that is derived data
(6.7 s and 3.0 s of `fsync` in one measured commit,
and 1.5 s in its `rename`).

## Reading the repository in process

The wrapper starts real Git as rarely as it can.
For read-only questions it reads the repository itself and falls back to Git for anything it does not fully understand.
The readers live in `src/native/repo_read_*.rs`;
the design,
the measurements and the limits are in
`doc/planning/cli-git-in-process-reads.md`.

What is answered in process,
with the standard output,
standard error and exit code Git itself would give:

- the repository location every wrapped command asks for (`rev-parse` with the bare,
  Git directory,
  common directory,
  top level and prefix options),
  and the Git paths and object format the commit path asks for;
- `ls-files --stage -z` and `ls-files --unmerged -z`;
- `diff-index --cached` (name-only and raw,
  with `--diff-filter` and literal pathspecs),
  `diff --cached --name-only -z`,
  `diff-tree --root -r -m` for a commit with at most one parent,
  `diff-tree -r --name-only -z` between two commits or trees (with or without `--no-renames`) and
  `diff-tree --no-commit-id --name-only -r -z` for a commit with exactly one parent (the landing and the review ask for these;
  only the trees that differ are read),
  and `show --no-patch --format=%s`;
- `config --list -z --show-scope`,
  `remote get-url`,
  `for-each-ref` with the wrapper's format,
  `rev-parse --verify --quiet <ref>^{commit}`,
  `rev-parse --abbrev-ref --symbolic-full-name @{upstream}`
  and `symbolic-ref --quiet HEAD`;
- `cat-file --batch` fed full object names,
  in the commit preparation.

Every reader answers only after checking what Git would check,
and otherwise leaves the question to Git:
`GIT_*` variables that can move a path,
global options other than `-C`,
bare repositories,
links,
other file systems,
reftable,
conditional includes,
unmerged or intent-to-add entries in a diff,
a `config.worktree` that makes the repository bare,
moves the worktree or holds a value the readers cannot prove
Git accepts (with `extensions.worktreeConfig` on,
the file is otherwise read and listed under the scope `worktree`),
a file that became a directory,
a gitlink,
a possible rename,
replace refs,
shallow or grafted history,
split or sparse indexes,
an index extension other than the ones Git reads without a word,
an index offset table that does not name the entries the reader parsed,
a ref name Git's own rule for a ref name would refuse,
a ref listing past the 256 refs the listing reader answers,
and configuration keys that can change the output.
Differential controls run real Git and the readers over the same command lines
(`src/native/repo_read_*_tests.rs`,
`src/native/binary/binary_budget_tests.rs`).

Set `CLI_GIT_IN_PROCESS_READS=off` in the wrapper's environment to start Git for every one of these questions,
for a measurement that compares both paths or to rule the readers out when output differs.

## Operating conditions

This package is one of the few that must keep working under the conditions its host lives in every day,
not only on a quiet machine.
The wrapper,
the commit service and their tests are expected to work under these conditions:

- high CPU load from other sessions and builds;
- memory pressure and heavy swap use;
- slow SSD and other I/O trouble;
- a nearly full disk,
  so a write can fail with no space left in the middle of a commit;
- process and descriptor limits close to exhaustion;
- a process killed at any instruction,
  which the service's journal exists for;
- a closed terminal or a broken pipe;
- slow fsync and unusual filesystems;
- DNS or mirror trouble,
  where the wrapper recovers or falls back instead of wedging.

The wrapper's guarantees are those of one host.
It supports one host per repository,
and a checkout used from two machines is not protected:
the owner answered a request to support a repository two hosts share (2026-11-10)
with "Not supported (Recommended)".
A command of another host counts as an owner of another PID namespace,
so the wrapper judges it by the age of its record and not by its process number (see "What it does"),
and the claims and leases of the commit service are compared by machine and boot identity,
which tells a restarted host from a running one and never closes a race between two.

### Latency requirements

Two requirements bound the latency the wrapper adds,
and `native:perf` enforces both:

- A forwarded real Git command gains strictly less than `2,000 ms` (`SPEC.md`).
- Every commit made through the wrapper must not take `5,000 ms` or more longer than the same commit made with native Git
  (each commit,
  not a percentile of them).

No wrapped command is required to finish within a fixed time,
and there is no 5-second bound on wrapped commands.
The owner's words (2026-11-01):

> " 5-second bound on wrapped commands" - there is never a 5s bound on wrapped commands.
> The measure is always just commits vs native git should not take 5s longer.

The 5-second `timeout --signal=KILL` of the binary tests is a hang guard of the test harness
(`TIME_BOUND_SECONDS` in `src/native/binary/binary_support.rs`),
short so that a mutant that loops fails every affected control
quickly.
It says nothing about how long a correct wrapped command may take.

### What pins each condition

Each condition has permanent controls that start the built executable on real Git fixtures.
The wrapper's side of a commit no longer writes state that outlives the command,
so the call-by-call fault sweeps of the
earlier landing (`strace --inject` failing,
killing or delaying every write,
rename,
flush,
descriptor and process start of a
commit,
and the records they left to recover) retired with that landing.
The commit service's own faults are pinned by the kill points of its journal (`binary_service_tests.rs`:
a kill at each point
of a batch,
requests sent again,
late and older services),
the pruned-content controls (`binary_service_pruned_tests.rs`)
and the unit controls of its landing (`service_land_tests.rs`);
the call-by-call sweeps are ported to the service below.
What the faults found,
and the change for each,
is in
[the fault injection note](../../../doc/troubleshooting/git-wrapper-fault-injection-findings.md).

- A closed terminal or a broken pipe:
  `binary_fault_stream_tests.rs` closes the standard streams,
  gives them a pipe with no
  reader or a full device,
  and requires that the status matches the one Git gives when started directly.
- A Git step that stalls:
  `binary_child_bound_tests.rs` delays the return of every start of a Git step past its class
  bound (`strace --inject`,
  a `ptrace` of the child,
  no privilege and no load) and requires that the step is stopped,
  named with its class,
  and that the repository recovers.
  The control reports a skip and checks nothing on a host without `strace`.
- A process killed at any instruction elsewhere:
  the worktree-copy and merge flows are killed at their phase markers
  (`binary_merge_isolation_tests.rs`,
  `binary_worktree_copy_tests.rs`) and,
  in `binary_fault_trace_tests.rs`,
  at a spread
  of the Nth `write`,
  `fsync`,
  `rename`,
  `unlink`,
  directory and process-start calls of each flow (`strace --inject`
  `signal=SIGKILL`,
  positions counted from an uninjected run,
  first and last included),
  followed by the next commands
  and `git fsck`.
  The commit service is killed the same way at any call of itself and of the Git it starts.
  A staging directory of a killed process stays on purpose and is ignored by every reader.
- A signal to the wrapper alone:
  `binary_fault_signal_tests.rs` signals the wrapper's own process ID (no `timeout` in front,
  which would forward the signal to its whole group) and checks that a stand-in Git received exactly that signal through
  the wrapper's relay,
  that the wrapper ends with `128 + signal`,
  that nothing runs on and that the repository recovers;
  a hangup of the whole group is a separate control.
  The `SIGQUIT` core dumps of Git seen on the development host came from
  the earlier controls,
  which signalled `timeout` and so reached Git and the editor directly.
- DNS or mirror trouble:
  `binary_fault_remote_tests.rs` makes the push after a landed commit and an explicit push meet
  a refused connection,
  a name under `.invalid`,
  a server that hangs up,
  a server that never answers (stopped at
  `CLI_GIT_PUSH_DEADLINE_SECONDS`) and a `git-lfs` that never exits,
  and requires that the commit stays,
  the failure is
  worded,
  no lock is held and no process the push started is left running;
  `binary_push_environment_tests.rs`,
  `binary_lfs_pre_push_tests.rs` and `binary_lfs_server_tests.rs` cover the stalled push and the failing upload.
  The push is the service's work after the command returned,
  so each control waits for the work records to settle and reads
  the failure from the notice the next command shows.
- Process and descriptor limits,
  memory pressure and a starved processor:
  `binary_fault_limits_tests.rs` sets
  `RLIMIT_NOFILE`,
  `RLIMIT_NPROC` and `RLIMIT_AS` (`setrlimit` between `fork` and `exec`,
  no privilege) on the wrapper and on
  the commit service,
  one side at a time,
  sweeps the descriptor and address-space limits from the smallest at which the
  program starts to the smallest at which a commit lands (both found by running,
  so a new build moves them),
  lowers and raises
  the service's `RLIMIT_NPROC` in place with `prlimit`,
  and runs a commit with the wrapper,
  the service and Git on one
  processor.
  After each,
  the pressure ends,
  the next ordinary command succeeds,
  a further commit lands and nothing is left.
  A failed allocation ends a Rust program with an abort;
  the controls accept that end and require the recovery.
- Slow storage,
  slow fsync,
  a full or failing disk:
  `binary_fault_trace_tests.rs` runs the service under `strace --inject`
  with every flush returning late (100 ms),
  with `ENOSPC` and with `EIO` from the Nth storage call on (`write`,
  `fsync`,
  `rename`),
  and requires the recovery after each.
  The wrapper's own `fsync` and writes are not swept because a commit no
  longer writes state that outlives the command.
- High CPU load,
  memory pressure and slow I/O in the test harness:
  the 5-second hang guard,
  which loaded hosts have tripped,
  and the byte copy of the wrapper that keeps a cold start inside it (the troubleshooting note below).

What no control injects,
and why:

- Swap and real memory pressure:
  the kernel's reclaim needs a cgroup the test does not own;
  `RLIMIT_AS` stands in for the
  allocation failures,
  and the gate container's `--memory` cap bounds the whole run.
- A real full filesystem,
  real slow storage,
  and unusual filesystems (FUSE,
  NFS,
  a loop mount):
  mounting needs a privilege
  the gate does not give.
  `strace --inject` stands in for the errors and delays,
  and `binary_subvolume_tests.rs` and
  `fs_kind_tests.rs` cover btrfs and the filesystem classification where the host offers them.
  The gate runs on an
  overlay filesystem.
- High CPU load:
  a synthetic load is banned.
  The container gate's `--cpus` cap and the single-processor control stand in.
- `RLIMIT_NPROC` for a user who may exceed it (a privileged run) is exempt from the limit;
  the two process-limit controls say
  so and check nothing there.

Known gaps:
an editor that never exits is not judged by `CLI_GIT_CHILD_WAIT_SECONDS` (it runs as long as it runs,
as documented,
and is ended by the signals above);
the removal of a name (`unlink`) is not swept with failing errors,
because a lock file left by a failed removal needs a person with native Git as with the wrapper;
a step stopped at its
bound ends with the processes it started,
found in `/proc`,
so only Linux ends them.

Tests therefore never wait for a quiet host.
The 5-second hang guard on one wrapped command in the binary tests is deliberate,
and is not a requirement on the wrapper;
its measurements and the evidence needed to change it are in
[the troubleshooting note](../../../doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md).

## State on disk

The wrapper keeps its state beside the repository's Git data under `monochromatic-git-policy-cli*` paths in the common Git directory.
Delete nothing there while a Git command is running.
A commit keeps a scratch directory `monochromatic-git-policy-cli-service-<id>-<attempt>` in the repository scratch directory
 `monochromatic-git-policy-cli-scratch` while it is made;
 the commit service removes the tree of an attempt once that attempt's landing is over,
and a command that no service took the tree from removes it itself as it ends.
Nothing in a commit waits for that removal:
 it happens beside the landing loop,
bounded by `SCRATCH_REMOVAL_CEILING` (thirty seconds),
 so a device that stalls costs a partial removal and a line in the service journal,
never a slow commit.
A tree no request ever named,
because its command was killed before it submitted,
goes once nothing has written it for two hours,
and its command renews it every hour while it lives,
so a running command never loses it;
the trees a machine makes while no service answers are bounded by that rule (about 9 MiB each)
 and by at most eight removed per command sweep;
 other entries of the directory go once they are a day old.
The service's after-commit work records and the notices of a failed backup push are kept under `monochromatic-git-policy-cli/`.
The timing record of each wrapped commit is appended to `monochromatic-git-policy-cli-commit-timing.jsonl` (see "Splitting a slow commit").
A directory `cli-git-transactions` in the worktree's Git directory belongs to an earlier release:
 this release no longer uses or reads it.
 A command that finds commit records in it prints a note on standard error,
 names the directory,
 gives the exact `rm --recursive --force -- '<directory>'` command that removes it (with the path quoted for the shell),
 and goes on.
 Look at `git log -1` and `git status` to see whether that commit was made,
 then run the command.

A merge in progress keeps a one-line record in `monochromatic-git-policy-cli-merges` in the worktree's Git directory,
 from before Git starts updating the checkout until the branch has moved.
Do not remove it by hand unless a command says so.
The record is written and its directory flushed before Git starts,
and its removal is not flushed:
a record a power loss brings back names a merge the branch already holds,
which the next command drops as already landed,
or one whose branch moved,
which it settles as it would settle any such record.
The flush it used to make was a third of a merge's fsyncs and it sat on the path the person waits for.
What Git's own update left at each path of a merge that carries work travels beside the record as
`<transaction id>.landed`,
so a command that finishes a killed merge reads the same entries the command that moved the checkout read;
it goes with the record and is never read by a command the record is gone from.
A file a merge kept beside a path is recorded in `monochromatic-git-policy-cli-kept-copies` in the worktree's Git directory,
one small file per copy naming the copy and the path it belongs to.
It is not a record a command refuses anything for:
a copy whose bytes the person's file holds again,
or the branch's commit holds them,
is removed and the line says which,
a copy an unfinished merge names goes back,
and any other copy is named again on every command that runs the recovery until one of those holds.

The ignored-state copy into a new linked worktree stages its files in a `.cli-git-worktree-copy-*` directory
beside the new worktree,
which can hold gigabytes.
The wrapper records that directory's name in the common Git directory before it creates it,
so a copy that was killed while staging cannot leak it:
the next invocation in the repository removes the directory and says so on stderr.
A directory a `git worktree add` was snapshotting into is removed the same way once the record written beside its name
holds the identity of what stands there and the folder holds nothing that record cannot account for,
which is how a whole snapshot of a checkout with its credentials and its `.git` stops outliving the command that was
killed;
a directory the wrapper cannot prove that way stays,
is named on every command with the reason,
and is yours to look at and remove by hand,
and only when no `git` command is running in that repository.
A copy also writes the registration it was made for inside that directory before its first file,
so the next command can tell a stage its own copy left,
which it removes and fills again,
from a directory another program put at that name,
which it names and never removes.
A directory holding nothing but that record,
or the part of it a kill inside its write left,
is a stage of that copy too,
because no ignored file of the source is there yet.
Delete a `.cli-git-worktree-copy-*` directory by hand only when no `git` command is running in that repository.

Each file the copy installs reaches its destination name only when it holds the source's bytes and permission bits.
It is filled in a file the filesystem has given no name,
or in a `.monochromatic-git-policy-cli-partial-*` file of its own where the filesystem has none,
and that name is one the next run removes,
so a credential is never readable at a wider mode than its source,
not even while the copy runs,
and a copy killed at any moment leaves nothing at a file's real name for the next command to refuse.

A worktree whose ignored-state copy ended before it finished is filled by the next command run in that worktree,
from the source its add recorded,
and only a copy that truly cannot be made leaves the worktree named,
with `git worktree remove --force <path>` and the same `git worktree add` line that copies the ignored state again.
A command in another worktree never pays for that copy:
it names the worktree whose files are missing,
that the first command run there makes the copy,
and that it does not wait for it.
Nothing the worktree gained since the add is ever replaced:
an entry whose bytes differ from the source's is left alone and named,
and the same line comes again from every later command.
No command a person runs fills an existing worktree,
and none is planned:
the owner answered a request for one (2026-11-10)
with "No,
remove and add again",
so removing the worktree and adding it again stays the remedy a person can take.
The add's own record is written before real Git runs and is handed to the copy rather than dropped once the checkout is
done,
so a command killed between the checkout and the copy's own records still leaves a record that names the worktree.
The copy writes its own record before its first entry and removes it when it completes,
so a copy killed at any moment is resumed by the next command run in that worktree and a copy that truly cannot
finish leaves the worktree
named instead of forgetting what is missing there,
with what ended it in that record,
so the later lines say which entry differs,
not only that files are missing.
An add records the worktree its ignored state comes from and which of that worktree's ignored paths it takes,
so a command killed after the checkout and before the copy's own records leaves enough for the next command run in that
worktree to make
that copy itself;
a record written by an earlier release names no source and its worktree is named rather than filled.
The line goes when the worktree is removed or is replaced by another one at the same path.

A worktree this program made has one record of its own beside its registration in the common Git directory,
 `monochromatic-git-policy-cli/worktrees/<name>/monochromatic-git-policy-cli-worktree-added`,
 which names the worktree's path.
It is written once the add that Git accepted has returned,
 and it goes with the registration,
 so nothing removes it on its own.
It is the only proof a later command accepts that this program made that worktree,
 and a merge uses it to remove the worktree once its branch has landed
 (see "What a worktree snapshot holds,
and when the wrapper removes one").

A `git worktree add` killed between Git's registration of the worktree and the end of its checkout leaves a record in the
 common Git directory.
Every command that runs the recovery before it prints one line on standard error about that worktree,
with `git -C <path> reset --no-enforce-worktree --hard --no-recurse-submodules` to finish the checkout and
`git worktree remove --force <path>` to remove it,
and names the worktree's `index.lock` first when a Git killed inside the index install left one,
because that command cannot start while the lock stands.
The hatch is in that line because the worktree guard reads a checkout that never ran as a worktree
holding uncommitted changes,
so it refuses a hard reset there,
and the line says what the reset discards:
`src/native/worktree/worktree_finish_commands.rs` holds the sentence,
and the placement ladder's own incomplete-add report prints the same one.
Every path in those commands is quoted the way a shell reads a word back
(`src/native/worktree/shell_quote.rs`),
so a `$`,
a backquote,
a quote or a newline in a path expands nothing and splits nothing when the line is pasted into a shell.
The record stays until the worktree is whole or gone,
so the same line comes again from every command that runs the recovery.
A look at the destination that fails,
rather than finding no pointer,
keeps the record too:
a directory the person or another process holds closed for a moment is no evidence
 that the add left nothing.

A worktree added from the main checkout receives the main checkout's ignored state minus what a new worktree does not need.
Left out by directory name,
wherever the name sits below the main checkout:
 a directory that holds a `.git` file or directory (another checkout,
including every registered worktree);
 the caches `.pnpm-store`,
`.gradle`,
`.kotlin` and `.zig-cache`;
 a `.cache` directory other than the one in the repository root (the root `.cache` holds the tool launchers and generated
 inputs that tasks read,
so it is copied);
 `incremental` and `zig-global-cache` below a `target` directory;
 and the records of earlier runs,
`.monochromatic` below a `node_modules` directory,
`target/verification`,
`mutants.out`,
 `mutants.out.old`,
`playwright-report`,
`test-results` and `.serena/cache`.
The run records follow the constant `COPY_RUN_RECORDS` in `src/native/worktree_copying/worktree_copy_prune.rs`,
which is `false`.
A linked or bare source copies everything it ignores except its nested registered worktrees.
An ignored file or directory that the user cannot read,
such as a log written by `root`,
is left out of any copy instead of
 failing it,
and one stderr line says how many and which first paths:
`monochromatic-git-policy-cli: left out 2 unreadable ignored entries from
 "/path/to/source": "a/b", "c"`.
The copy clones files with a copy-on-write request,
in parallel on a large tree,
and trusts the clone:
it proves a cloned
 source file unchanged by its size,
modification time,
change time and inode instead of reading both files again.
A source file written to during the copy still fails the copy,
and a file the filesystem would not clone is compared byte
 for byte.

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
mise run //package/git-policy/cli:native:test:host:btrfs -- <test name filter>
mise run //package/git-policy/cli:native:release:host-controls
mise run //package/git-policy/cli:native:test:container
```

- `native:check` type-checks every target.
  `native:clippy` lints every target with warnings denied.
  `native:build` writes the debug executable to `package/git-policy/cli/target/debug/monochromatic-git-policy-cli`.
- Every process a unit test starts dies with the test thread,
  however the test ends:
  start helpers with `spawn_owned` in `src/native/testing/test_support.rs`,
  which gives the helper a process group of its own
   (so what it starts dies with it) and kills the group when the thread's fixtures are removed,
   the moment a failing or panicking test unwinds (`a_loop_started_by_a_panicking_test_is_gone_when_the_test_ends`).
  A shell loop that a failing push-progress test never ended ran for 87 minutes on 2026-11-03 before this.
- `native:test:host` runs the unit tests and `native:test:binary:host` runs the tests that start the built executable,
  both against the host's Git while iterating.
- The controls that need a btrfs filesystem cannot run in the gate container,
  which is an overlay:
  each one prints `BTRFS CONTROL NOT RUN: <control> (<what its fixture root is on>)` and passes there.
  `native:test:host:btrfs` is the other half of that bargain.
  It points `CLI_GIT_TEST_ROOT` at `${HOME}/temp/agent/cli-git-btrfs-fixtures`
  (or at the directory that variable already names),
  unsets `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED`,
  and runs every target on that root with `--success-output=final`,
  so a control that cannot run fails the run with the filesystem it found instead of passing,
  a control that had nothing to check here names itself in the log,
  and no list of control names has to stay current.
  Selected by name,
  the same controls are the `btrfs_*` tests of the library and the binary-level controls that call
  `on_btrfs_or_acknowledged` (`src/native/binary/binary_support_btrfs_controls.rs`).
- A real wrapper release waits for that run:
  `native:release` dispatches nothing until `native:release:host-controls`
  (`bin/native-release-host-controls.ts`) reports that every control passed on this host,
  and that task is what the crate's entry in `package/crate-release/cli/releasable-crates.json` names as its
  `dispatchGate`,
  so the dispatch runs it after every refusal that costs nothing and before it starts the workflow
  (`package/crate-release/cli/README.md`).
  It runs `native:test:host:btrfs` with its own deadline of three hours,
  three times the 3286 seconds a full run took on this host on 2026-11-10,
  and it refuses the release when a test failed,
  when a control printed `BTRFS CONTROL NOT RUN` for any reason but needing a filesystem that is not btrfs,
  when the deadline stopped the run,
  or when the run holds no passing status line for a control the gate requires by name
  (the ioctls,
  the migration,
  and one binary-level control).
  The refusal names what refused
  and what a person does on a host that has no btrfs filesystem to run the controls on.
  `--dry-run` skips the controls and says so,
  because a dry run runs nothing their evidence would cover.
  A control that needs a filesystem that is not btrfs does print that line on this host,
  which is the one such line a green release run holds
  (`doc/troubleshooting/nextest-hides-a-passing-tests-output.md`).
- `native:release` starts a wrapper release and `native:release:wait` follows it
  (`package/crate-release/cli/README.md`).
  A wrapper release runs two workflows:
  `crate-release.yml` bumps the version,
  checks the crate and pushes the bump to `main`,
  and the `cargo-publish.yml` run that its `publish` job dispatches publishes the crate and builds the tarballs of the
  GitHub release.
  Both are followed to their own end,
  reading what each run says about itself with `gh run view --json=createdAt,status,conclusion,jobs`:
  a release run that passed is reported as passed,
  a run that failed names the job and the step that did not pass,
  and a run that is still building says which jobs it waits for
  (`doc/troubleshooting/gh-run-watch-exit-status-conflates-failures.md`).
- `native:test:container` is the authority.
  It builds the crate and runs the tests and Clippy in bounded containers with pacman's current Git and no network.
  The test run and Clippy start together,
  one container each,
  and the gate waits for both:
  a Clippy compile behind the suite was a fifth of the gate's wall time while the two checks had nothing to say to each
  other,
  and either one still fails the gate on its own
  (`bin/native-gate-checks.ts`).
  Before it copies or builds anything it runs `lint:sidecar-locks`,
  so a sidecar lock this package's dependencies moved past stops the gate in milliseconds
  instead of surfacing later in the sidecar's own build.
  Set `GIT_POLICY_NATIVE_IMAGE_TAG=<name>` to keep the images of parallel runs apart.
  The test run uses `cargo nextest run --no-fail-fast`,
  so every test runs even after a failure and one flaky test
  never hides the rest;
  the exit status is still non-zero,
  and nextest prints its summary and the list of failed tests
  last.
  The image build is layered so that a rebuild reuses everything the change did not reach,
  with no network and
  `--locked` throughout (`bin/native-layered-build.ts`,
  `bin/native-test-containerfile.ts`):
  - The dependency layer compiles every registry crate from the manifests,
    the lock files,
    the vendored sources and
    empty stand-in source files,
    so its cache key holds no source file.
    Only a change to a manifest or a lock file rebuilds it,
    which includes a version bump of the crate or of a path
    crate.
  - The path-crate layer copies the JSONC crate and the scanner with its engine and compiles them (the scanner's
    build script precompiles the embedded baseline),
    so it is rebuilt only when one of them changes.
  - The subject layer copies the wrapper's `src/native` and `README.md` (the tests include it) and compiles only the
    wrapper,
    so a test sidecar edit or a source edit recompiles one crate.
  - The linter stage follows the same split:
    its registry dependencies compile from stand-ins,
    then its own sources.
  - Each real-source layer touches the copied files in the same step,
    because Cargo compares file times with the
    stand-in build and would otherwise treat the stand-in output as current.
  - Build cache mounts (`RUN --mount=type=cache`) are not used:
    they keep `target/` outside the image,
    but the test
    run,
    the planted controls and the mutation campaigns reuse the image's own `target/`,
    so the compiled output has to
    stay in the image.
  - The image build keeps the same CPU limit as the run.
    In paired builds at the same time,
    a cold dependency layer compiled about 1.7 to 1.9 times faster with
    `--cpus 4` than with the default (47 s and 58 s against 23 s and 31 s) and the path-crate layer about 1.6 times
    faster,
    while the wrapper-only layer that a test or source edit rebuilds did not change (about 12 s either way).
    A cold build happens only after a manifest or lock file change,
    so the default is unchanged;
    pass `--cpus 4` after the `--` for that case (the test concurrency follows it too).
  - Every build step costs a fixed amount on a busy host (measured as tens of seconds,
    up to minutes,
    for a step that
    does no work),
    so the Containerfile keeps its step count low and puts the steps that rarely change first.
- `native:perf` builds the release executable and measures the latency it adds to forwarded Git commands and to
  commits (explicit-path and service,
  unsigned and SSH-signed),
  with paired direct-Git samples in disposable repositories.
  It reports the median,
  p95 and median absolute deviation of both sides and the added delta,
  and for each commit the median,
  p95 and maximum of the paired deltas (the wrapped time minus the direct time of the
  same round).
  It fails when a forwarded command's added p95 reaches the 2,000 ms bound,
  when any commit's paired delta reaches the 5,000 ms bound (the owner's measure,
  see "Latency requirements"),
  or when any sample fails.
  The reasons are the text of a thrown error (`bin/native-perf-verdict.ts`),
  so a failed run needs no separately assigned exit status.
  `-- --output <file>` stores the raw samples (baselines live in `perf/`;
  `schemaVersion` 2 adds the paired deltas and
  names each scenario's `kind`).
  The judging lives in `bin/native-perf-judge.ts`,
  and `bin/native-perf-judge.unit.test.ts` runs it on the recorded
  baseline,
  edited to break each bound.
  Options for measuring a commit at the size the person commits at
  (`doc/planning/cli-git-commit-latency-under-disk-pressure.md`):
  `--shape <repository>` makes every fixture a reflink copy of a prepared repository without remotes (a clone of the main
  checkout,
  for example),
  refreshes its index and gives it the tracked file `README.md` to edit;
  `--files <count>` edits and commits that many tracked text files by name in every sample (the size of the commit;
  one by default),
  `--change-rules` appends a comment line to `forbidden-strings.append.local.txt` before every sample,
  so a `--shape` repository whose configuration names that file (this repository's does) meets rules text with no compiled form in the cache;
  it changes nothing a commit reads in a fixture whose configuration does not name the file,
  `--new-files <count>` also creates that many new files per sample and stages them with `git add` before the commit names them (staged additions),
  `--hooks-from <git directory>` copies that repository's policy hooks,
  `--tool <path>` puts an executable such as `git-lfs`
  on the fixtures' PATH,
  and `--only <scenario,...>` measures the named scenarios.
  `--cold` drops the pack,
  index and packed-refs pages of the fixtures from the page cache before every sample on both sides,
  and `--trace-directory <dir>` leaves an `strace` record of each sample and of each commit service for counting syncs,
  bytes and process starts.
  `--pause <ms>` waits after each sample,
  so the work a commit leaves running after its answer is over before the next one.
  With `--output`,
  the report also holds the wrapped commits' timing records and the service's `answered` lines
  (`evidence`),
  which split a slow sample into its phases.
- `native:lock:members` records a changed dependency list or version in `Cargo.lock` without moving any other locked
  version.
  `native:lock:check` asks cargo whether every manifest the gate builds still resolves against its own lockfile
  (`bin/native-lock-check.ts`,
  replacing the scratch scripts `locked-check.ts` and `lock-refresh.ts`).
  It prints one line per manifest with cargo's status,
  the directory and cargo's first error,
  and `--refresh` first re-resolves only the entries a raised floor made unsatisfiable,
  which writes a lockfile before it checks again.
  It exits nonzero when any locked check failed.

A release build of the executable:

```sh
# Run in package/git-policy/cli.
cargo build --release --locked --bin monochromatic-git-policy-cli
```

The release profile keeps `panic = "unwind"` and `overflow-checks = true`.
The linked scanner turns a matcher panic into a failing finding by catching the unwind,
and the crate refuses to compile under `panic = "abort"`.

### The release gate

Three tasks start the full gate and read its verdict.
They replace the coordinator's scratch scripts
`~/temp/agent/gate/launch-gate.ts`,
`~/temp/agent/gate/relaunch-gate-on-main-tip.ts` and
`~/temp/agent/gate/wait-for-gate-verdict.ts`,
which the wrapper's release flow ran many times a day.
The scratch copies stay where they are
until the coordinator has switched over;
nothing in this package reads them.

```sh
# Run from the repository top level.
mise run //package/git-policy/cli:native:gate -- <worktree name or full path> <label>
mise run //package/git-policy/cli:native:gate:main-tip
mise run //package/git-policy/cli:native:gate:verdict -- <log path> <deadline seconds>
```

- `native:gate` starts `native:test:container` as its own transient systemd user unit,
  which is the image build,
  the compile,
  the whole suite and Clippy,
  and prints `<unit>.service=<log path>`.
  It answers as soon as systemd has queued the unit,
  so nothing waits an hour for a run.
  The label is what the gate covers,
  in lowercase letters,
  digits and hyphens:
  it names the unit,
  the log file and the image tag of the tested image,
  all three `gpc-gate-<label>-<stamp>`
  with a second-precise timestamp
  (`bin/native-gate-unit-name.ts`).
  `--repository <path>` names the checkout the worktree belongs to
  (otherwise the nearest folder above the caller whose `.git` is a folder),
  `--log-directory <path>` names the log folder
  (otherwise `${HOME}/temp/agent/gate`,
  or `GIT_POLICY_GATE_LOG_DIRECTORY`),
  and `--unit-cpu-quota`,
  `--unit-memory-max`,
  `--unit-tasks-max`,
  `--unit-io-weight`
  and `--container-pids-limit` override the caps every recorded gate ran with,
  which are documented with what sized each one in `bin/native-gate-caps.ts`.
  A worktree that holds no `package/git-policy/cli` is refused before the unit starts,
  because systemd would otherwise fail at `CHDIR` with an empty log.
- `native:gate:main-tip` detaches the gate worktree
  (`--worktree <name>`,
  default `gate-wrapper-next-alpha`,
  or `GIT_POLICY_GATE_WORKTREE`)
  at the commit `main` points at now,
  then starts a gate there labelled `<worktree name minus gate->-on-<sha>`,
  so the log names the commit it tested.
  It prints `gate worktree at <sha>` and `launch status <n>`,
  and a worktree holding uncommitted work is not moved:
  the status is printed and the command exits 2,
  because another session's work is never thrown away (rule `EC1`).
  Run it from a folder inside the checkout,
  or name the checkout with `--repository <path>`.
- `native:gate:verdict` waits,
  bounded,
  for the gate log to hold a verdict
  and prints it with nextest's summary and every line that names a failure.
  A gate is read from its log and never from its unit:
  the unit is started with `--collect`,
  which unloads it when it ends and makes `systemctl show` report the defaults of a unit that does not exist
  (`doc/troubleshooting/systemd-run-collect-reports-success-for-a-failed-unit.md`).
  The unit therefore writes its own exit status as the log's last line,
  `gate-exit-status=<n>`,
  and that line is part of the printed verdict.
  The exit status of this command says whether a verdict arrived,
  never whether the gate passed:
  a passing run and a failing one both earn a zero,
  and the printed line and summary carry the difference.
  `--poll-milliseconds` sets the interval between two looks
  (default 20 seconds,
  shortened to what the deadline leaves).
  A wider caller can name what ends the wait and what it prints then:
  `--marker <text>` waits for another line than the exit status
  (the older waits spelled their own markers,
  `exit=`,
  a nextest tag or a sentence),
  `--unit <name>` ends the wait as soon as systemd reports that unit stopped,
  so a gate that died is reported at once instead of at the end of the deadline,
  and `--tail-lines <count>` sets how many of the log's last lines a wait without a verdict prints
  (30 by default,
  none with 0).
  A wait that ends without a verdict names its reason,
  `no verdict in <log> within <n> s` or `unit <name> stopped without a verdict in <log>`.

### Lint

```sh
# Run from the repository top level.
mise run //package/git-policy/cli:lint
mise run //package/git-policy/cli:lint:monochromatic-lint
mise run //package/git-policy/cli:lint:sidecar-locks
mise run //:format:monochromatic-lint -- package/git-policy/cli
```

- `lint` runs every `lint:*` task of the package,
  and `lint:monochromatic-lint` is one of them.
  It runs the repository linter,
  `monochromatic-lint --max-warnings 0 package/git-policy/cli`,
  under the root
  `monochromatic-lint.config.jsonc`,
  and fails on any error or any warning.
  The package's own paths are the Rust sources,
  the rustdoc inside them and every Markdown file.
- The three rules that fail most often are `markdown/semantic-line-breaks` (also for the `///` and `//!` text of Rust
  files,
  through the `**/*.rs/*.md` block of the configuration),
  `rust/max-lines` (300 code lines per Rust file,
  blank and comment lines excluded) and `rust/avoidable-closure` (a closure that a named function,
  a loop or a match
  can replace).
- `format:monochromatic-lint` is the fixer.
  It breaks lines at semantic boundaries and changes nothing but whitespace,
  line breaks and the `>` or comment marker
  that a new line needs.
  Run it over the paths you edited before every commit.
  It cannot split a file or replace a closure.
- A file over the line limit is split into sibling files whose names say what they hold,
  by what the parts use:
  the leaf goes first in the folder's `mod.rs`,
  and the parent that uses it re-exports what other files import.
- The package's own `lint` is also reached from the root `lint`,
  so the root run lints this package twice,
  once through the package task and once in the repository-wide pass,
  as `package/ide/app` does.
- `lint:sidecar-locks` fails when a sidecar crate beside this package
  (`package/git-policy/cli.fuzz`)
  locks one of this package's dependencies at another version,
  or when Cargo refuses the sidecar's `Cargo.lock`
  (`bin/native-sidecar-lock-check.ts`).
  A sidecar links this package by path,
  so a dependency this package gains,
  drops or moves makes the sidecar's lock stale;
  the sidecar's own build only meets that later,
  and nothing else in this package's checks read its lock.
  `mise run //package/git-policy/cli.fuzz:lock:members`
  records the versions again,
  and the container gate runs the same check before it builds anything.

### Merging main into a branch that edited doc comments

The line-break pass rewrote the `///` and `//!` text and the Markdown of the whole package,
so a branch that edited a
comment before the pass meets conflicts that are only about where lines break.
Resolve them this way:

1. Run `git merge main`.
   Collect the conflicted files with `git diff --name-only --diff-filter=U`.
2. In each conflicted hunk,
   keep your branch's side of the comment lines (the first side,
   `HEAD`).
   Their line breaks do not matter,
   because step 4 breaks them again.
   Where the same hunk holds code,
   merge the code by hand as for any conflict,
   taking main's code and your own.
3. When main and your branch changed the words of the same comment,
   merge the words by hand:
   the fixer does not choose between two wordings.
4. Run `mise run //:format:monochromatic-lint -- <the conflicted files>`.
   Anything it still prints is not a line break;
   fix it by hand.
5. Search the files for leftover markers (`<<<<<<<`,
   `=======`,
   `>>>>>>>`),
   run `mise run //package/git-policy/cli:lint:monochromatic-lint`,
   stage the files by name and finish the merge.

The wrapper's word-level second merge pass (see "Word-level second merge pass") may already have resolved some of the
hunks;
check them with `git diff --cached -- <file>`.

### Git commands that may need a login

NDG:
 Git or git-lfs commands this package's tests,
scripts or agents start that may reach a login set `GIT_ASKPASS` to the empty string and `GIT_TERMINAL_PROMPT=0`
 ([why][askpass-dialog]).

Set only `GIT_TERMINAL_PROMPT=0` and the desktop's password dialog still opens on a host whose `SSH_ASKPASS` names one.
In a Rust test that is `.env("GIT_ASKPASS", "").env("GIT_TERMINAL_PROMPT", "0")`.
A test that has to see the prompt program start (the controls in `src/native/landing/service_push_retry_tests.rs`) points
both `GIT_ASKPASS` and `SSH_ASKPASS` at a recorder that answers a name,
so no window opens,
and its log answers whether the
program asked on every host:
naming only `SSH_ASKPASS` would leave the host's own `GIT_ASKPASS` to be the program that
runs,
and the recorder's empty log would then prove nothing.

The wrapper adds the same pair to a forwarded command whose process has no controlling terminal
(`invocation_overlay` in `src/native/invocation/child_environment.rs`).
A control that compares a forwarded command's environment builds the pair from the same reading,
`control_terminal()`
(`expected_overlay` in `src/native/invocation/entry_tests.rs`).
A control that froze the environment without the pair passed at a person's terminal and failed under a test runner,
an agent's shell or a container started without `--tty`,
which is how the campaigns start theirs:
`podman run --network=none <campaign image> sh -c tty` printed `not a tty`,
and the `invocation::entry` group ran 36 tests with 10 of them failed on a clean tree (2026-11-10).

### Controls that depend on timing

WFR:
 A control's verdict comes from the order of events:
 release its fixture from the wait's own line,
 never from a sleep whose passing only the host's speed decides.

The same defect was fixed twice in the index-lock tests (`2b33c7bd32` and `35cd5ae7e5`).

### Git's own fsmonitor

Git's file-system monitor is off,
by the owner's decision of 2026-11-03:

- "If git's fsmonitor is fundamentally broken and cannot be easily or meaningfully worked around,
  disable git's fsmonitor globally entirely and make doctor say so."

Why:
Git 2.56.0's Linux daemon registers an inotify watch for every directory before it starts answering
(`compat/fsmonitor/fsm-listen-linux.c`,
then `ipc_server_start_async`),
and a Git process that finds no listening daemon starts one and waits for it for up to `fsmonitor.startTimeout`
(60 s by default;
`builtin/fsmonitor--daemon.c`).
On the owner's main checkout,
where linked worktrees live inside the checkout,
one daemon watched 2,269,377 directories
and spent 117.7 s of CPU and 734 MB to start,
and 261 daemons held 4,591,335 watches in all.
A commit of 7.7 s spent 6.5 s with the wrapper's Git child waiting for `git fsmonitor--daemon start`,
while a running daemon saved about 7 ms per `git status` (16 ms off,
9 ms on,
five alternating runs each).
The measurements and the cause are in `doc/troubleshooting/git-fsmonitor-daemon-start-wait.md`.

What the wrapper does now:

- The Git it starts for its own work passes `-c core.fsmonitor=false`:
  the commit's index builds and trees,
  `read-tree`,
  `diff-tree`,
  `reset`,
  the scratch commit,
  the landing and service steps,
  and its queries.
  A Git that reads the index asks the file-watching daemon first and waits for it,
  so a step that found no daemon
  took 60 s twice in one commit.
  The option is part of the prefix every real-Git start already carries,
  so it adds no process and no start.
- A command the person typed is forwarded with their configuration untouched,
  so the person's `core.fsmonitor` stays for what they run themselves.
- It never writes `core.fsmonitor` in any repository,
  and never starts,
  stops or kills `git fsmonitor--daemon`.
  The setting is the person's configuration,
  and it is off there:
  the person's global `~/.gitconfig` holds
  `core.fsmonitor = false`.
- `git monochromatic-git-policy-cli doctor` says so on standard error.
  When `core.fsmonitor` is false or unset it prints one line that Git's fsmonitor is off by the owner's decision of
  2026-11-03,
  with the file and scope of the entry that decided (or "not set"),
  the reason in one sentence,
  and the path of the document above.
  When the last entry of `core.fsmonitor` in any scope turns it on (true,
  a true word,
  a number other than zero,
  a key without a value,
  or a hook path) it prints a finding that names the file and the scope and the command that
  turns it off for that scope,
  such as `git config --global core.fsmonitor false`.
  A value from `-c` or the environment has no file,
  so the finding says to remove it there.
  Doctor reports and never writes configuration,
  and its exit status does not depend on this line.
  Doctor reads one more configuration listing with `--show-origin` for this,
  because the snapshot the commit path
  reads names scopes but not files.

The wrapper's own Git is the exception,
because the owner chose it on 2026-11-10:
asked "Should the wrapper switch fsmonitor off for its own internal Git commands only?",
the owner chose "B:
off for internal Git (Recommended)".
So `-c core.fsmonitor=false` reaches every Git the wrapper starts for its own work,
and never a command the person typed
(`doc/handover/questions-for-the-owner.md`,
"The wrapper's own Git and fsmonitor").

The owner's earlier words,
kept as history;
each was superseded on 2026-11-03 by the decision above:

- "\"To stop the fsmonitor program\" - we don't need to."
- (2026-11-02) "Why are we messing with git's fsmonitor?
  I already told you it's fine for git to run it."
- (2026-11-03) "Why are some of your agents still messing with fsmonitor?
  I've said it at least twice that we
  shouldn't mess with git's own fsmonitor s."
- (2026-11-03) ".git/config overriding it to false is wrong."

Before that decision the person's `~/.gitconfig` set `core.fsmonitor = true`.
A main checkout's `.git/config` had held `core.fsmonitor=false` until 2026-11-03,
written by no code in this repository;
it was removed at the owner's word.
Tests that reproduce `core.fsmonitor = true` inside their own temporary repositories stay:
they check that the wrapper behaves as Git does under that setting
(`repo_read_config_typed_tests.rs`,
`reader_fsmonitor_tests.rs`,
`binary_budget_tests.rs`),
and they never touch a real repository's monitor.
The controls of the doctor line are in `doctor_fsmonitor_tests.rs`,
with an injected global file.
`SPEC.md` (search "core.fsmonitor") records the decision for each code path.

### No `--quiet` on the wrapper's own Git runs

No Git run the wrapper or the service starts carries `--quiet` or `-q` where the option only hides messages or progress.
The owner's words:

- (2026-11-03) "No the wrapper should never append --quiet.
  Letting native git emit whatever it wants to is expected."
- (2026-11-03) "Drop it everywhere (Recommended)",
  about the wrapper's own internal runs:
  maintenance,
  the auto-push fetch,
  fast-forward merges,
  reset,
  clean and init.

So no other output-hiding option or redirect stands in for it either.
A `--quiet` stays on a run whose documented effect is the exit status or what is computed (`rev-parse --verify --quiet`,
`symbolic-ref --quiet`,
`diff-index --quiet`,
`diff-files --quiet`;
read in the Git 2.56.0 sources,
listed in
`doc/planning/cli-git-slow-server-and-inner-quiet.md`).
The words of a run that said something are kept in the journal (`src/native/worktree/inner_git_words.rs`):
the service writes one
`info` entry in its log,
and the wrapper writes one client entry (`journalctl --identifier=monochromatic-git-policy-cli`),
both with the field
`CLI_GIT_INNER_COMMAND`.
A test that starts such a run records its arguments with `recording_git.rs` and checks `quiet_starts`.
`git reset` without `-q` lists the tracked paths whose files differ from the index,
so the service's journal now names
the paths of other uncommitted work for every landing.

### Container base images

Every container image of the verification tasks starts from the newest CachyOS image the host CPU can run,
on its `latest` tag (`doc/decision/container-base-distro.md` at the repository top level).
The tag is never pinned to a version or a digest,
so an upstream CachyOS update that breaks a task is a bug to fix at once.
Two derived base images carry what the network-free image builds need:

- `native-test-base.Containerfile` builds `localhost/git-policy-native-test-base`:
  CachyOS,
  the distribution's Rust (Cargo and Clippy) and build tools,
  and the tools the tests start:
  Git (at
  `/usr/bin/git`,
  where the tests expect the real Git),
  Node,
  pnpm,
  `strace` and cargo-nextest.
  The gate image and the mutation images build on it.
- `e2e/concurrent-commits.Containerfile` builds `localhost/cli-git-concurrent-e2e`:
  CachyOS with the distribution's Git
  and Node and `ssh-keygen`.

Every tool in these two images is the distribution's current pacman package,
with no version pin,
no source build and no
downloaded archive (owner decisions of 2026-11-04:
"We don't need to pin git because my current system is already using
git latest.",
"We don't need to pin Node too.",
and "Pacman for everything it has").
That is an exception to the repository rule that containers install locked tools from `toolchain.lock.json` rows,
for
these test images only.
Pacman checks every package signature (`SigLevel = Required DatabaseOptional` in the CachyOS image).
`mise run //package/git-policy/cli:test:git-image` prints the release of each tool the base image got;
the wrapper's
`MINIMUM_GIT_VERSION` follows the Git release printed there (an older Git draws a warning and is not refused).

Each run of `native:test:container`,
`test:git-image`,
`test:e2e:concurrent:image` and
`test:e2e:concurrent:native:image` first pulls the `latest` tag with `podman pull --policy=newer`
(one small request when nothing changed;
a registry that cannot be reached is reported and the copy the host holds is used).
The derived base is tagged `<first digits of the pulled image ID>-<first digits of the Containerfile hash>`,
so a new CachyOS image or an edited Containerfile builds a new base (with network,
once) and every run after that finds it.
The gate image then builds from the base's ID with no network,
exactly as before,
and its layer cache is keyed on that ID.
`bin/native-test-base.ts` holds the pull,
the tag and the build.
The resolver that picks `cachyos-v4`,
`cachyos-v3` or `cachyos` is `package/test-harness/core-matrix`;
a host that is not x86-64 stops with an error that names the image,
because these images have no Alpine variant.

### Container limits

Every container the verification tasks start is bounded.
The defaults are 2 GiB of memory,
2 CPUs and 512 processes,
defined once in `bin/container-limits.ts`.
`native:test:container` is the one task that starts from its own bounds,
from `bin/native-gate-container-limits.ts`,
because a release waits on that gate and its suite is what the CPUs shorten
(`doc/planning/git-policy-cli-cachyos-test-containers.md` records the paired measurement):
- Four CPUs,
  which nextest reads as four tests at once and the image build as four compile jobs.
- A process bound of 256 for each of those tests,
  so 1024.
  The fault sweeps of the binary suite start six cases inside one test and each case
  starts a service and its Git programs,
  and at four tests at once the suite peaked at 401 processes in one run and reached
  the 512 the repository default allows in another,
  which failed three tests on `fork` with `Resource temporarily unavailable`.
  The same four tests at 1024 processes passed all 7,192.
- The 2 GiB memory bound is the repository default,
  because four tests at once peaked at 978 MiB of it.

A bound a person passes after the `--` still wins,
with one exception:
the process bound is a floor that follows the test count,
so a `--pids-limit` below it is raised and the gate prints the line that says so.
Every task accepts overrides after the `--` of `mise run`,
with these exact option names:

- `--memory <size>` is a whole number with an optional unit `b`,
  `k`,
  `m` or `g`,
  such as `512m` or `2g`.
- `--cpus <count>` is a number from `0.01` with at most two decimals,
  so `1`,
  `1.5` and `0.25` are valid.
  The build CPU quota is derived as `cpus * 100000` microseconds per 100000 microsecond period.
- `--pids-limit <count>` is a whole number of processes from `1`.
  It is accepted only where the container has a process cap,
  and rejected with a message otherwise.

Every container also starts at the lowest disk weight,
`--blkio-weight=10` and `--cgroup-conf=io.weight=10`
(`lowestDiskWeight` and `diskWeightArguments` in `bin/container-limits.ts`);
no option raises it.
Podman writes the first to `io.bfq.weight`,
which the BFQ I/O scheduler reads,
and the second to `io.weight`,
which the iocost controller reads,
and each leaves the other file at the default weight,
so passing both puts whichever file the host's disk scheduler reads at the lowest value.
`podman build` accepts neither option,
so the containers of image builds keep the default weight.

A value is a separate token (`--memory 1g`);
the `--memory=1g` form is not read and reaches the task's own option parser,
which rejects it as an unknown argument.
An invalid value stops the task before any container starts.
The message names the option and the accepted forms.
Options repeat with the last one winning,
and the number of build jobs (`CARGO_BUILD_JOBS`,
`make --jobs`) follows the
whole CPUs of the limit,
never below one.

```sh
# Run from the repository top level.
mise run //package/git-policy/cli:native:test:container -- --memory 4g --cpus 4
mise run //package/git-policy/cli:native:mutation:scoped -- --memory 1g --cpus 1 --file src/native/invocation/startup_gate.rs
mise run //package/git-policy/cli:test:e2e:concurrent:native --memory 1g --cpus 1
mise run //package/git-policy/cli:test:git-image -- --memory 4g
```

- `native:test:container` accepts all three options,
  and so do `native:mutation:scoped` and `test:git-image`.
- `native:mutation` runs the gate and then the mutation step,
  and mise passes the options to the mutation step only,
  so the gate runs at the defaults.
  Run `native:test:container` with its own options first when both need changing.
- `test:e2e:concurrent:native` takes `--memory` and `--cpus` as task flags and applies them to the run.
  The image build it depends on uses the defaults,
  and `test:e2e:concurrent:native:image` and
  `test:e2e:concurrent:image` take the same options after the `--`.
  The e2e containers have no process cap either.
- The mutation manifest records the limits it ran with,
  and the gate and image-build manifests record the container
  argument arrays.
- `native:mutation:scoped` reuses the image of the last gate,
  whose `CARGO_BUILD_JOBS` was fixed when it was built.
  Run the gate with the same `--cpus` for the job count to match.
- The `package/git-policy/cli.fuzz` campaign uses the defaults from the same module and has no override options yet.

### Who runs a mutation campaign

The session's main agent starts every campaign of this package and waits on it,
never a subagent;
the owner's reason
is in the launcher's [Who runs a campaign](../../mutation-test/cli-cargo-mutants/README.md#who-runs-a-campaign).

### Resuming a mutation campaign

`native:mutation:scoped` and `native:mutation:parallel` (and so `native:mutation`) are resumable.
A kill,
a reboot,
an out-of-memory stop or `systemctl --user stop <unit>` loses no scored mutant:
run the same task with the same options on the same gate image and it continues where the campaign stopped,
scoring
only mutants that no earlier run scored,
whether they were caught,
missed,
timed out or unviable.
For `native:mutation:parallel`,
start the same launcher command again;
each file resumes its own campaign
(see "Image tags of a parallel launch" for what the launcher does to the unit image tags first).

### Image tags of a parallel launch

Each file unit of `native:mutation:parallel` runs the image `localhost/git-policy-native-test:<label><index>`
(the shared launcher's default `--tag-template`,
one tag per file).
Those tags do not exist after a gate,
so before it starts any unit the launcher makes them
(`bin/native-mutation-parallel.ts`,
`ensureUnitImages` in the shared package;
its README,
"Image tags",
has the rules):
it runs `podman tag <full ID of the gate image> localhost/git-policy-native-test:<label><index>` for every unit,
reads every tag back,
and starts nothing if one does not name the gate image.
A plain `native:mutation:parallel -- <label> <files>` therefore works right after `native:test:container`,
with no `podman tag` by hand.

The gate image is the one the last `native:test:container` built,
`localhost/git-policy-native-test:<tag>` with the tag from `GIT_POLICY_NATIVE_IMAGE_TAG`,
or `development` when that variable is not set in the launcher's shell
(`gateImageTagOf` in `bin/native-gate-image-tag.ts`,
which the gate,
the campaign task,
the controls-only task and the launcher all use,
so they name one image).
To make the tags from another gate image,
pass `--from-tag <tag>` after the `--`:
the person's flag comes after the launcher's own,
and the later one wins.
A relaunch of a label after a new gate moves that label's tags to the new image and prints a line for each
(the old image stays in storage).
A running unit keeps the image it started with,
and a resumed campaign over a new image is another campaign directory.
If the gate image does not exist,
nothing starts and the message names the image and `--from-tag`.
The launcher exits 0 only when every file unit started:
a unit that `systemd-run` refuses is named with its error and the launcher exits 1,
while the other units keep running
(the shared launcher's README,
"Exit status and errors").
A refused launch prints only the plain message,
with no stack trace.
Before this change the launcher started units whose tags did not exist,
every one died at once with "image not known",
and the launcher exited 0 (2026-11-07).

The campaign's results are in `target/verification/native-mutation-<key>/`,
where the key names the gate image,
the exact options,
the digest of the planted controls and the identity of the code that runs and judges them,
so a changed control table or runner rescores.
While a campaign runs,
`attempt-NNN/mutants.out/` there holds the outcomes scored so far in `caught.txt`,
`missed.txt`,
`timeout.txt` and `unviable.txt`.
When it ends,
`mutants.out/` holds the same four files as the union of every attempt,
with `outcomes.json`,
`log/`
and `diff/`,
and `exit.json` records the status of the whole campaign (`attempt` and `skippedMutants` say how many
earlier mutants the last run left out).
To score everything again,
delete that directory;
a rebuilt gate image gets a new one by itself.
The mechanism,
the choice of cargo-mutants `--iterate`,
and the limits are in
`package/mutation-test/cli-cargo-mutants/README.md`.

### What a mutation campaign costs per mutant

Every mutant runs the package's tests,
so the cost of a campaign is mutants times the time the tests need
(`doc/planning/mutation-campaign-per-mutant-cost.md` has the measurements).
Two measures cut it;
both keep every kill and every result.

The baseline is skipped when the gate already passed on the same image.
`native:test:container` writes `manifest.json` (the full ID of the image it tested) when it starts and `passed.json`
(`tests`,
`clippy` and the same `image`) only after the test run and Clippy both exited zero,
in
`target/verification/native-<random>/`.
`native:mutation:scoped` resolves the ID of the image it layers cargo-mutants over and runs the unmutated baseline only
when no such directory names that ID with a passed record;
the log says which it did (`Baseline: skipped, because
<directory> proves ...` or `Baseline: run, because no gate evidence ...`) and the campaign manifest records the
directory as `baselineProof`.
A rebuilt snapshot has another image ID,
so it has no proof until its gate has passed.
The proof is the gate's pass on that image,
and only for a campaign whose `--memory` and `--pids-limit` are at least the
ones in the gate manifest's `limits`:
with fewer processes than the gate had,
the fork failures of 2026-11-01 would come
back as caught mutants,
so the log names the gate directory and the limits and the baseline runs.
A manifest that records no readable limits proves nothing.
A campaign that reuses a gate image runs at whatever `--pids-limit` it was given,
which defaults to the repository's 512 processes,
while the gate's own bound is 1024 for its four test threads:
pass `--pids-limit 1024` after the `--` of the campaign task to keep the proof,
or that campaign's baseline runs once.
Taking the gate's own bound from its manifest is the follow-up this paragraph leaves open.

The planted controls (a full suite run per guard removal,
about 24 minutes per campaign) are skipped by the same idea.
A campaign whose controls were all noticed leaves `planted-controls-passed.json` in its evidence directory
(in its `pass-1` directory for a two-pass campaign),
written to a temporary file,
flushed and renamed so a reader never sees half of it.
The record holds `plantedControls: true`,
the full `image` ID,
`limits` (the container arguments the controls ran with),
the control names in `controls`,
and `controlsSha256`,
a digest of every control's file,
replaced text,
new text and named test.
`native:mutation:scoped` looks through `target/verification/native-mutation-*` before it plants anything,
and skips the controls when a record names the same image ID,
the same controls,
and no more `--memory` or `--pids-limit` than this campaign has.
The log says `Planted controls skipped, because <record> proves ...`
or `Planted controls: run, because no campaign evidence proves ...`.
A rebuilt snapshot has another image ID,
and a control added,
removed or edited has another digest,
so either one runs the controls again.
A record made at larger limits is named in the log and not used.
A control that was not noticed,
or one whose planted source did not compile,
leaves no record.
`--cpus` is recorded but not compared,
because fewer CPUs only slow the suite.
The record also holds `harnessId`,
the `plantedHarnessId` literal in `bin/native-planted-controls-harness.ts`:
the digest of the sources that run and judge the controls
(`bin/native-planted-controls.ts`,
`bin/native-planted-controls-outcome.ts` and `bin/native-verification-process.ts`).
It is part of the loaded code,
not hashed from disk afterwards,
so an edit between loading and writing cannot publish a record for code that never ran,
and it is content-derived,
so two branches that change the judge differently cannot share an identity the way a counter would.
A unit test asserts that the literal equals the digest of the sources and prints the digest to write,
so editing any of them without updating the literal fails the test,
and a record of another identity is not proof.
Limits are not parsed as Podman parses a command line.
The runners write only `--rm`,
`--init`,
`--network=none`,
the two disk-weight arguments (a record made before they existed lacks them and is still read),
`--cpus=` and one `--memory=` and one `--pids-limit=`,
so a recorded argument list with anything else
(a repeated bound,
a split or attached short option,
a cluster,
an option whose value looks like a bound,
a trailing bare option)
was not written by a runner and gives no proof,
whatever it would mean to Podman.
A zero or unlimited value (`--pids-limit=-1`) is no bound a campaign can be compared with either.
The same reading applies to the gate's `passed.json` proof.
Campaigns take no lock for this.
Two campaigns started together over one image,
with no record yet,
both run the controls and each writes its own record;
neither waits for the other,
because waiting would serialize work that is independent.
The cost is that both pay the 24 minutes,
so start one first and the rest after its record exists.
`NATIVE_MUTATION_SKIP_PLANTED=1` still skips without any record
(a person's claim that another campaign ran them).
A resumed campaign is held to the same rule as a new one,
and a changed control table or a changed runner or judge rescores:
the digest of the controls and the runner and judge identity are part of the campaign key,
so a campaign run after either changed is another directory that keeps no scored results.

`native:mutation:parallel` runs the controls beside the file units,
never before them (`bin/native-mutation-parallel.ts`,
`bin/native-planted-parallel.ts`).
Before it starts anything it resolves the image ID of every unit's tag,
and looks for a proving record with the reader above,
at the memory and process limit the units will run with.
With a record,
no extra unit starts,
and each file unit logs the record it found.
Without one,
it starts one more capped unit per distinct image,
named `<unit prefix>-<label>-planted-<first 12 of the ID>`,
through the same slot limiter.
That unit runs only `native:mutation:planted`,
and writes the record into a `native-mutation-planted-*` evidence directory.
The file units start at the same moment and do not wait.
Each skips the controls,
because `NATIVE_MUTATION_PLANTED_UNITS` names the unit for its image,
and logs `Planted controls skipped here, without proof`,
with the unit and the image.
That skip is not a pass:
the results of those campaigns hold only once the controls unit ends with a `planted-controls-passed.json` for the image.
The launcher prints the controls unit,
its log and that rule,
and puts the unit first in the `systemctl` wait hint.
The controls unit's log ends with `Planted controls passed; recorded in <record>`,
or with `Planted controls FAILED over image <ID>`,
followed by the controls that were not noticed,
and the unit then shows as failed.
If systemd refuses the controls unit,
or a tag does not resolve to an image,
the launcher says so,
and the file units of that image plant the controls themselves,
as `native:mutation:scoped` does alone.
`--setenv NATIVE_MUTATION_SKIP_PLANTED=1` after the `--` still skips without proof,
and no controls unit starts.
The controls unit runs the full image ID the launcher inspected (`--image`),
never the tag,
which may be moved while the unit waits for a slot.
The launcher inspects with the environment a unit gets
(the tag,
the `--setenv` entries,
and its own `TMPDIR`,
`PATH` and `HOME` last)
and nothing inherited from its shell but `XDG_RUNTIME_DIR`,
so it looks in the container storage the units will use,
and it refuses a `--setenv` of the image tag variable or of `NATIVE_MUTATION_PLANTED_UNITS`,
which it sets itself,
because an override would run another image than the one inspected.
The controls unit gets the same extra environment as the file units,
with the launcher's own `TMPDIR`,
`PATH` and `HOME` last.
It succeeds only when it published a matching record or found one,
so a record that could not be written fails the unit.
A campaign that deferred keeps its obligations in its manifest as the list `plantedControlsDeferred`.
Each holds the image,
the limits the campaign ran with,
the digest and names of the controls,
the runner and judge identity,
and the controls unit with its systemd invocation ID and its log.
The launcher names the invocation it started in the coordination entry,
so a campaign defers only to that invocation and never to a later unit that took the name.
Whether an obligation is met is asked on demand and never kept:
a matching record exists,
found with the same reader and by the saved keys,
not by the asking process's own table or code,
or it does not.
Finalization and `native:mutation:reconcile` run that lookup and print the record path,
or "no record yet" with the unit,
its invocation and its log.
With no record,
an obligation is pending while that very invocation runs,
and unvalidated once it ended,
after one more look for a record published in between.
Unvalidated fails the campaign's finalization,
so a campaign whose controls failed never passes.
So does a manifest that is unreadable,
holds an incomplete obligation,
is missing after the campaign finished (`exit.json` without `manifest.json`),
or no longer holds an obligation the running process itself decided to keep.
A campaign's key includes the controls digest,
the runner and judge identity,
and the memory and process limit (the memory normalized to bytes),
and `campaign.json` records those key parts in full so that a clash of the shortened key is refused,
so a resumed campaign has the same controls,
runner and limits as the attempt that deferred,
and a proof or a local run in a resume pays that obligation by construction.
An attempt that proves or runs the controls writes a manifest without the field,
and a person's skip does the same.
The two passes of a two-pass campaign share the key and each directory is validated by its own manifest.
A pass that runs the controls itself while the first pass is still waiting for a record must publish its own,
and fails the campaign with the reason when it cannot,
because a debt in another directory is paid only by a record the lookup finds.
A directory in which a container ran
(an `attempt-*` holds `mutants.out`)
and that has no manifest is invalid,
finished or interrupted;
a controls-only directory,
an attempt that never started,
and the aggregate root of a two-pass campaign whose pass 2 never ran have none and are exempt.
The controls unit's name carries a token unique to the launch,
so no other launch's unit can be taken for it.
A file unit that starts after the controls unit is gone and left no record runs the controls itself.
`mise run //package/git-policy/cli:native:mutation:reconcile` looks up the obligation of every campaign under `target/verification` later,
and exits nonzero unless all are validated.

`--passes 2` runs each file's campaign in two passes.
Pass 1 runs only the tests that can reach the mutated file:
the tests of its module and of every module that imports
it,
directly or through other modules (the closure of importers over the crate's `use` paths under `src/native`),
plus
the test targets that refer to one of those modules,
and never the binary suite (`native_binary`).
Pass 2 runs the whole suite on the mutants pass 1 missed or timed out,
and on those only.
The report in `mutants.out` and the exit status read as one campaign:
a mutant pass 2 scored has pass 2's outcome.

```sh
# Run from the repository top level; one file, two passes, measured on its own before a queue is started.
mise run //package/git-policy/cli:native:mutation:parallel -- --passes 2 --timeout 900 m1 repo_read_query.rs
```

`--passes 2` refuses a glob `--file`,
no `--file` and a file the library does not declare,
instead of quietly running
one pass.
It also refuses to start without the gate proof above:
pass 1 runs a filtered baseline at best,
so run
`native:test:container` first,
at limits no larger than the campaign's.
A campaign directory started with `--passes 2` refuses a run without it,
and the other way round;
move the directory
away to switch.
The selection is written to the manifest (`passOne`) and to the log.
How the two passes are seeded,
composed and resumed is in
`package/mutation-test/cli-cargo-mutants/README.md` under "Two passes".

### Source layout

The crate is one library and one executable (`monochromatic-git-policy-cli`,
which is also the commit service in its service mode),
and its sources live under `src/native/`.
Each folder there is one subsystem,
so a contributor opens a folder for a concern instead of searching a flat list of
prefixed file names.
The small-file rule stays:
a folder holds many small modules,
each with its sidecar test file.

#### How a module is reached

- `lib.rs` is the map.
  It declares each folder (`pub mod command;`) and re-exports the folder's modules with a glob (`pub use command::*;`).
- Each folder's `mod.rs` declares the modules inside it with one documented line each.
- The glob re-export keeps every module at its old path,
  so `crate::command_options` and
  `git_policy_cli::command_options` still resolve.
  The fuzz crate,
  the executable and the integration tests name modules that way and needed no change.
- A glob re-export loses to an item written in `lib.rs`,
  so no folder may share its name with a module.
- `use super::` in a module that a folder declares directly names the folder,
  not the crate root.
  Write `use crate::module_name` to reach a module of another folder.
- Library test names now start with the folder:
  `command::command_options::tests::name`.
  The integration test roots (`native_binary`,
  `native_candidates`) keep their names.

#### Subsystems

- `invocation/`:
  what one invocation does from arguments to Git:
  the decision,
  real Git resolution,
  forwarding,
  startup,
  management words and the wrapper's own Git version check.
- `config/`:
  reading and validating `monochromatic-git-policy-cli.config.jsonc` and the Git configuration a command is judged by.
- `command/`:
  the option grammar of each Git command the wrapper understands and the rules that rewrite,
  refuse or hint.
- `policy/`:
  the policy engine:
  registry,
  triggers,
  events,
  passes over candidates and the direct fix.
- `markdown/`,
  `dependent_version/`,
  `standards_review/`:
  three policies large enough for a folder each.
- `candidate/`:
  what a commit or push would add,
  read from the index or objects,
  and the scanner run over it.
  Its `candidate_consumer_tests.rs` is the `native_candidates` test root.
- `commit/`,
  `shadow/`,
  `transaction/`:
  the wrapper's commit path,
  the shadow repository it prepares on and replays
  onto,
  and the durable transaction records with their recovery.
- `landing/` and `service/`:
  what the resident commit service does to land a commit,
  and the service mode's
  code around
  it:
  protocol,
  server,
  client,
  journal,
  provisioning and the unit files in `service/service_units/`.
- `push/`,
  `lfs/`,
  `merge/`,
  `conflict/`:
  pushes and their waits,
  Git LFS,
  merges with the corpus in
  `merge/word_merge_corpus/`,
  and conflict state around commits and checkouts.
  A push the remote refused as stale is reconciled by one awaited loop
  (`push/auto_push_reconcile_awaiting.rs`),
  whose plan and words are read once
  (`push/auto_push_reconcile_plan.rs`);
  a command outside the runtime reaches that loop through
  `push/auto_push_reconcile.rs`,
  which carries the merge push of a person's explicit `git push` on a plain thread of its own.
- `worktree/`,
  `worktree_copying/`,
  `subvolume/`,
  `repair/`,
  `health/`:
  worktree commands,
  the ignored-state copy,
  btrfs subvolumes and the filesystem primitives under them,
  package-manager repair,
  and the `doctor` report.
- `repo_read/`,
  `index/`,
  `lock/`,
  `process/`,
  `foundation/`:
  reads without a Git process,
  the index file,
  locks and claims,
  facts about processes,
  and shared building blocks that depend on no subsystem.
- `testing/`:
  test-only fixtures shared by many unit tests,
  never part of the release executable.
- `source_guard/`:
  the controls that read the crate's own source text.
- `binary/`:
  the consumer-level controls that start the built executable;
  `binary_tests.rs` is the `native_binary`
  test root.
- At the top:
  `lib.rs`,
  `main.rs` (`monochromatic-git-policy-cli`,
  installed as `git`;
  with the words `monochromatic-git-policy-cli service run` first it is the commit service,
  `service/service_main.rs`).

#### Where new files go

- A new module goes in the folder of the subsystem it belongs to.
  Declare it in that folder's `mod.rs` with a documented line,
  never in `lib.rs`.
- Its sidecar test file `<module>_tests.rs` sits beside it and is declared in the module with `#[path]`.
  Test support used by one subsystem stays in that folder under `#[cfg(test)]`;
  support used by many goes in `testing/`.
- A data file that `include_str!` reads sits in a subfolder beside the module that reads it,
  and its path is added to the
  `include` list in `Cargo.toml` when the published crate needs it.
- A new subsystem is a new folder with a `mod.rs`,
  a `pub mod` and `pub use` pair in `lib.rs`,
  and a line in the list
  above.
  Its name must differ from every module name,
  singular where it reads naturally.
- Do not put a module in `lib.rs` or beside it;
  the folders exist so the top level stays short.
- The source scans (`source_guard/`) follow `mod` declarations from the `Cargo.toml` targets,
  so a file that no
  declaration reaches fails them.
- Paths in tooling name the folder,
  for example `src/native/command/command_add.rs`.
  A mutation scope may name subfolders (`--file src/native/command/command_add.rs`);
  pass 1 of a two-pass campaign
  selects tests by top-level folder,
  so its first pass covers more tests than it did when every module was a top-level
  entry.
- Records under `doc/` that predate this layout name the old flat paths.
  `git log --follow -- <new path>` finds a file's history across the move.

#### The `native` directory layer

The sources sit under `src/native/` although the TypeScript implementation that the name told apart from is gone.
This is a layout decision,
picked by the coordinator's agent,
veto open.
Removing the layer needs no binary rename and no behaviour change,
because the targets are named in `Cargo.toml`.

- Keep `src/native/` for now and drop it in a follow-up.
  - Pros:
    no second mass rename while other branches still edit these files;
    nothing else changes paths.
    The shared mutation scope rules (`nativeSourcePrefix`) and their golden scenarios keep their prefix.
  - Cons:
    the redundant segment stays in every path until the follow-up.
- Move to `src/` now (`src/lib.rs`,
  `src/main.rs`,
  and the folders).
  - Pros:
    the conventional Cargo layout and shorter paths,
    in one step with the folder move.
  - Cons:
    a second rename of every file under review in other branches,
    a change of the prefix in the mutation package
    and its golden scenarios,
    and hundreds of path mentions in other packages and documents.
- Move to `src/` with the executables in `src/bin/`.
  - Pros:
    Cargo finds the executables without `[[bin]]` paths.
  - Cons:
    everything the previous option costs,
    plus the executable entry points move away from `lib.rs`.

Ranking:
keep and drop later > move to `src/` now > `src/` with `src/bin/`,
because the first avoids a second rename
while branches are in flight and loses only a path segment,
the second reaches the same layout as the third at a
smaller cost,
and the third only adds moves that nothing requires.

## License

`LGPL-3.0-or-later`.
The license texts are in `LICENSES/`.

[mise]: https://mise.jdx.dev/
[repository]: https://github.com/Aquaticat/Monochromatic
[askpass-dialog]: ../../../doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md

## The commit service

A scoped commit (`git commit -m <message> -- <paths>`,
with the message,
identity,
sign-off,
trailer,
empty-commit,
reused-message,
fixup and `-a` options listed in `doc/planning/cli-git-resident-service.md`) is landed by a resident commit
service instead of by the wrapper itself.
The service is a mode of the same executable,
`monochromatic-git-policy-cli monochromatic-git-policy-cli service run`
(options `--journal`,
`--git`,
`--instance` and,
for a development run,
`--path` or `--abstract`),
that the operating system
starts on demand;
the wrapper never starts it,
and typed by hand with no socket from the manager and no address it refuses
and starts nothing.
The release archive holds that one executable,
as cargo-binstall requires of every `[[bin]]`.

The repository is the one the command line names.
`-C` moves the wrapper,
and `--git-dir`,
`--work-tree` and `--bare` (or the `GIT_DIR` and `GIT_WORK_TREE` variables) select the repository,
read from the directory the `-C` chain reaches,
as Git reads them.
A commit whose Git directory sits beside its work tree,
or one an environment variable names,
lands in that repository,
and the one the shell happens to be in gains no object and no commit.
The wrapper's own Git starts run in the directory the `-C` chain reached,
so they receive the caller's other global options without that `-C`.

When the manager stops the service (`SIGTERM`),
the unit's `KillMode=mixed` makes it signal the service alone.
The service then ends the Git processes it waits on (pushes,
queries,
the import of a commit's objects),
except the Git that moves a branch:
that write is not signalled,
and the shutdown waits for it for at most the stop bound
(`SETTLE_CEILING` in `stop_of_the_service_noticed_by_its_children.rs`),
so a slow branch write finishes and the commit lands.
The service ends by the signal that stopped it once the work in flight has finished or the bound has passed,
and `KillMode=mixed` makes the manager kill whatever is left of the unit at that moment
(or when `TimeoutStopSec` has passed).
A branch write that outlives the bound is therefore cut by the manager,
the branch stays where it was,
the commit stays on its scratch branch `refs/worktree/cli-git-service/<seq>` when the landing made one,
but on no ref at all when only Git could build it,
which asks for a hook,
an editor,
a signature or an unusual head,
and the batch's open intent is what recovery reads.
The next commit in that worktree settles the intent before it lands:
it keeps the commit on `refs/cli-git/kept/<seq>`,
the ref this wrapper names for work it did not land,
and removes the intent,
so `git for-each-ref refs/cli-git/kept/` shows the commit from then on.
Until such a commit comes,
the commit is on the scratch branch alone,
or on no ref at all when the landing made none,
and the client that saw the stop is told only that the service stopped before it could confirm the commit.
`TimeoutStopSec` is written into the unit from the stop bounds (`STOP_TIMEOUT_SECONDS`),
above the service's last-resort alarm,
so the host's default stop timeout never decides
(`DefaultTimeoutStopSec=` in a user's manager configuration can set it shorter than the bounds).
A service whose unit was written before this setting has the old kill mode until the next commit's provisioning rewrites the
template and asks the manager to reload its unit files;
a reload makes the manager read the new settings for the stops that follow,
and a service that runs meanwhile is not restarted.
Commits queue in arrival order,
a batch of whatever is queued lands with one
durable write,
and a commit that waits says how many commits go first.
There is nothing to install and no setting.

**The service never starts at system startup or at login** (owner,
2026-11-07:
"Were you implying our wrapper or something in it
auto starts itself on system startup or user login?
It should never do that!
Follow Bazel.";
then:
"Systemd units are fine.").
It is started by the first command that needs it,
and exits after three idle hours,
like Bazel's server.
The units are runtime units with no `[Install]` section (no `WantedBy=`,
`RequiredBy=` or `Also=`),
so `systemctl --user enable`
has nothing to link;
provisioning links nothing,
writes only below `$XDG_RUNTIME_DIR/systemd/user` (the one other thing it does
there is remove files an earlier release put in `~/.config/systemd/user`),
and asks the manager only to reload,
reset a failed
unit,
start,
stop and report a state.
Guard tests keep it so:
`src/native/source_guard/no_start_at_login_tests.rs` and
`src/native/service/service_provision_never_starts_at_login_tests.rs`.

- The first commit that needs the service sets up its socket activation itself,
  and every later one checks it is still
  right:
  it compares the two installed units with what this release would write and,
  only when they differ (an upgrade or
  a downgrade moved the executable the units name),
  rewrites them,
  reloads the manager,
  clears failed state and starts the socket.
  When they match it opens no connection to the bus.
  Setting up writes two unit files into
  `$XDG_RUNTIME_DIR/systemd/user` (nothing is left in the home directory;
  the next first use after a login does it again)
  and asks the user's service manager,
  through the session bus,
  to start the socket.
  The manager starts the service when
  the first connection arrives.
  Units that an earlier release wrote into `~/.config/systemd/user` are recognized by their
  content and removed;
  a file of the same name that something else wrote is never touched,
  and the commit stops and names it.
- The single service of the releases before one service per worktree is stopped before the first instance starts,
  wherever
  its unit files are (including `~/.config/systemd/user`);
  the commit waits for it to be inactive,
  and fails naming it when
  it will not stop.
  Limit:
  going back to one of those releases starts its single service beside the per-worktree instances
  until their idle hours pass;
  stop `monochromatic-git-policy-cli@*.service` and `monochromatic-git-policy-cli@*.socket` with `systemctl --user stop` before committing
  with the older release (`doc/planning/cli-git-resident-service.md`,
  "Limit:
  a downgrade").
- A failed backup push of a linked worktree,
  and a push the service had not run yet,
  survive `git worktree remove` and
  `git worktree prune`:
  the wrapper copies them first,
  and the next command in any worktree shows them once.
  The copy is
  kept while the command that took it may still be removing,
  and until the worktree's checkout is seen intact (Git
  refused) or its Git directory is gone (Git removed it);
  a record that cannot be copied or kept is printed on standard
  error,
  and what could be copied stays.
  Limit:
  a worktree that Git's own `gc` prunes is not seen by the wrapper,
  so its
  records are lost with it.
- The service keeps the records of its newest 1024 finished batches in its journal and removes older ones when it starts;
  a request is answered as already landed only while its own client retries it,
  which is minutes at most.
- The service exits after three hours with nothing to do (Bazel's server has the same default);
  the next commit starts it
  again.
  A service started from another release of the wrapper is asked to finish its batch and exit,
  and the next commit
  reaches the current one;
  if it does not exit,
  the commit says so and names `systemctl --user restart cli-git.service`.
- `git monochromatic-git-policy-cli service status` says whether the service answers;
  it is a diagnostic only.
- When the service cannot be set up (no session bus,
  no user service manager),
  the commit stops with a plain message that
  says what to do,
  and changes nothing:
  the service is the only way a commit lands,
  so there is no other path.
  Every command
  line the service does not build from the command line itself (amend with an editor,
  an editor,
  signing,
  templates,
  a
  repository whose configuration sets a template,
  a clean-up mode,
  signing or trailers) is built by real Git in a scratch
  repository and landed by the service as Git made it.
- The backup push and the automatic upkeep of a commit the service landed run in the service after the command returned.
  A push that fails does not fail the commit;
  the next commit through the service prints a notice once.
  The service pushes
  with its own environment,
  so a variable only your terminal has (an ssh agent address) must also be in the user manager's
  environment.
- Where a worktree holds the tracked file `doc/audit/standards-review-gate-shadow.md`,
  the service also runs a standards
  review of each commit it landed,
  after the command returned,
  and appends one entry to the repository's review log,
  `<common Git directory>/monochromatic-git-policy-cli/standards-review-shadow.md`.
  The review decides nothing about publication:
  a commit is pushed as soon as its post-commit gate allows the push,
  whatever the review later says,
  and a review item that waits behind other work of the service holds no push back.
  A review is claimed one at a time and runs before every step of the slow lane that can take minutes,
  so a commit's review ends within the review's own time limit rather than behind a trash of a removed worktree.
  That log is untracked and inside the Git directory,
  so no
  checkout shows it as a change;
  there is one per repository,
  shared by every worktree's service,
  and the service creates it
  when missing.
  A linked worktree's entries go there too,
  never into that worktree's own Git directory (`.git/worktrees/<name>`),
  which Git removes with the worktree.
  Each entry is written in one append,
  so entries of concurrent instances never interleave,
  and no lock is
  taken.
  The tracked file only switches the review on and is never written by the service.
  The live log moves aside to the next free generation at one mebibyte like every other log of the wrapper,
  and its generations are kept up to 32 MiB together.
  A commit's receipt line is looked for in the log and in every generation of it,
  so an entry that rotated away is still an entry and its commit is never reviewed twice.
  `SHADOW_REVIEW_LOG` points it at another file;
  the runbook is `doc/runbook/standards-review-gate.md`.

### Splitting a slow worktree add

The wrapper appends one JSON line per `git worktree add` it serves to `<common Git directory>/monochromatic-git-policy-cli-worktree-add-timing.jsonl`,
 in one append with no flush to disk,
beside the commit timing file.
`steps` gives each step's milliseconds and calls (steps do not overlap,
and `otherMs` is the rest of `totalMs`),
 `work` gives the time spent on worker threads by name (the Cargo workspaces,
the tools' repairs and the source checks run side by
 side,
so a name's sum can exceed the step that waited for it),
and `notes.route` names the rung that served the add.

```sh
# sh: the newest add of this repository, one line
tail --lines=1 -- "$(git rev-parse --path-format=absolute --git-common-dir)/monochromatic-git-policy-cli-worktree-add-timing.jsonl"
```

### Splitting a slow commit

Two logs exist without anyone having switched anything on,
and neither is ever shown at the terminal.
Together they split the time of a wrapped commit by phase.

The wrapper appends one JSON line per commit to `<common Git directory>/monochromatic-git-policy-cli-commit-timing.jsonl`,
beside
`monochromatic-git-policy-cli-audit.jsonl`,
in one append with no flush to disk.
Every worktree of the repository writes to the one file,
and a worktree's `instance` field tells the lines apart.
The file moves aside to `monochromatic-git-policy-cli-commit-timing.jsonl.1` at one mebibyte,
then `.2`,
and on,
and the generations are kept up to 32 MiB together,
oldest removed first,
as every log of the wrapper.
At the rate measured on 2026-11-10 (1,143,526 bytes of 229 records in 5.4 hours) that is about six days of timing.
A failed write never changes the commit's result.
A commit that stops before it reaches the service,
because the machine has no way to start one,
writes no line.
Read the newest lines of the repository you committed in:

```sh
# sh: the five newest commits of this repository, one per line
tail --lines=5 -- "$(git rev-parse --path-format=absolute --git-common-dir)/monochromatic-git-policy-cli-commit-timing.jsonl"
```

The fields,
all durations in whole milliseconds taken from the monotonic clock:

- `startMillis` is the wall-clock start (milliseconds since the Unix epoch),
  and `time` is when the line was written,
  in the form of every other record of the wrapper.
  `requestId` is the identity of the commit's request,
  the same text as `request="<id>"` on the service's `answered` line and as the journal field `CLI_GIT_REQUEST_ID`,
  so the two records join on a key and not on a time;
  it is null for a commit that never made a request.
  `level` is `warn` for a command that `stopped` and `info` otherwise.
  `instance`,
  `gitDir`,
  `branch`,
  `release`,
  `pid`,
  `outcome` (`landed`,
  `ended` or `stopped`) and `exitCode` say which
  commit it was.
- `totalMs` is the whole command.
  It is the sum of `routeMs` (finding the worktree's route to a service,
  this commit's identity,
  the standard input a pathspec file is read from,
  and the client's settings,
  none of which a phase covered before),
  `prepareMs` (policies,
  hooks,
  the private index,
  the tree and the objects;
  `attempts` counts the
  preparations),
  `buildMs` (the message,
  the identities,
  the tree check,
  a commit Git builds,
  the hand-over of the scratch
  directory),
  `provisionMs` (checking and,
  after an upgrade,
  rewriting the socket-activation units),
  `submitMs` (sending the
  request and waiting for the answer),
  `afterLandingMs` (the summary and the files a policy corrected),
  `gateMs` (the
  post-commit gate),
  `handoffMs` (leaving the push and the upkeep for the service) and `otherMs`,
  which is whatever no phase
  covers.
- `indexBuilds` states how each build of the private commit index was made,
  in the order built:
  `kind` is `first` for the command's own preparation and `retry` for a rebuild after the branch moved,
  and `statDataKept` says whether that build started from a copy of the real index,
  so every entry its tree shares with the real index kept its cached stat data.
  An index whose only marks are intent-to-add entries is copied as well,
  so its build keeps the stat data too.
  A build from the base or the tree alone holds none,
  and the scratch `git commit` then hashes every file of the work tree
  (233,618,538 bytes of LFS content cleaned and 3.6 s,
  measured on 2026-11-07).
  The list is empty for a commit that built no index,
  so a reader never has to tell a missing field from a commit that stopped first.
- `submitMs` contains `reachMs`,
  the time to reach a service of this release.
  `connectTries`,
  `refusedConnects` and `startWaitMs` show the manager starting the service,
  `updateRounds` counts the
  retire requests sent to a service of another release,
  `releasesSeen` lists every release that greeted,
  in order,
  and
  `answeredByRelease` is the one that took the request.
- `submitMs` also contains `answerWaitMs`,
  the time from the request being written to the answer:
  the queue,
  another
  program's hold on the index and the landing itself.
  `positions` lists the queue positions the service reported,
  `waitingNotices` counts the notices the service sent it about its wait,
  another program's hold and the words of a write its own request is inside,
  whichever of them its wait had mattered enough to say,
  `lockHolderKind` says what the last of them said of that program
  (`proven-alive` for a process proven to hold the lock,
  `unproven` for an owner nobody could prove alive,
  null when there was none),
  `reconnects` counts connections that broke and `sends` counts requests written.
- `landing` states what the landing of this commit spent and held,
  which the service measures and the answer carries,
  and is `null` for a service of a release that sends no figures:
  `lockHoldMs` is how long the landing held the index lock,
  which is what the commits behind it queued on,
  `queuedBehind` is how many requests were still waiting for their answer when the landing finished,
  `readsMs` is the whole of the residual reads,
  `reads` names each of them
  (`config`,
  `branch`,
  `parents`,
  `changed`,
  `content`,
  `witness`,
  `conclusion`,
  `owed`,
  `follow`,
  and never more than `readsMs` together),
  and `syncs` is every file or directory sync the landing asked for,
  in the order it asked,
  each as `{"what": "built-index", "waited": true, "ms": 12}`,
  with `waited` false for one it handed to the background task and never waited for.
- `handoffMs` is one figure:
  leaving the work records and waking the service,
  which reaches the service a second time.
  It is not split into a reach and a wait.
- `scratchRemovalMs`,
  `removalsThatDidNotFinish` and `stderrMs`
  are the two costs measured wherever they fell,
  so a reading can say which of them a slow `otherMs` was:
  the time spent removing scratch trees,
  how many of those removals ended without finishing within their ceiling,
  and the time spent writing to standard error,
  which waits for whoever reads it.
  They were added for the reading in
  `doc/planning/cli-git-commit-latency-under-many-agents.md`.
- `prepareParts` splits the preparation by the stretch of work it was in,
  as `{"layout": {"ms": 12, "calls": 1}, ...}`:
  the seams are one line each in `src/native/landing/service_prepare.rs`,
  each figure is the time that stretch spent without a Git child
  (the steps are listed apart in `steps`),
  a name marked again by a retry is added together with `calls` saying how many stretches make the sum,
  and `preparePartsDropped` counts the stretches beyond the sixty-four the record lists.
  It is what lets a reading name the in-process work a slow `prepareMs` holds.
- `buildParts` is the same shape for the build phase:
  `{"branch": {"ms": 1, "calls": 1}, "shadow skeleton": {"ms": 3, "calls": 1}, ...}`,
  with a name for every stretch that held no Git child.
  The shadow repository's creation is the one this was added for:
  it is in-process filesystem work,
  so `steps` never named it and a build of 63 444 ms whose longest step was 65 ms explained itself with no figure
  (`doc/planning/cli-git-commit-latency-under-many-agents.md`
  holds that reading under what the build phase waits on).
  Its names are `branch`,
  `message` (the identity and the message),
  `amend tree`,
  `fitting request`,
  `request`,
  `shadow` (what the shadow must carry),
  `shadow pool` (taking a finished shadow instead of making one),
  `shadow skeleton`,
  `shadow config`,
  `shadow links`,
  `shadow conclusion`,
  `shadow refs`,
  `client build` (the scratch `git commit`,
  whose own figure is in `steps`),
  `commit read back` and `amend fields`,
  and `buildPartsDropped` counts the stretches beyond the sixty-four the record lists.
  A build that stops inside itself still states its time,
  because the phase is closed where the command stops and in the record's own writer,
  so no build is ever reported as time that no phase covers.
  A build that took a finished shadow from the pool names `shadow pool` and no `shadow skeleton`,
  `shadow config` or `shadow links` stretch,
  which is how a reading tells the two routes apart
  (`SPEC.md`,
  "A shadow taken from the pool").
- `steps` lists every Git step the command started,
  in the order they began,
  each as `{"name": "git commit", "offsetMs": 18, "durationMs": 1004}`:
  the program and the subcommand (never a path,
  a message,
  an option value or the environment),
  when the step began after the command's start,
  and how long it ran,
  in whole milliseconds.
  It names the step that spent a slow phase,
  which `quietMs` cannot (that holds the longest quiet gap of a step,
  not its time).
  The commit Git builds in the scratch repository is `git commit`:
  it runs with the terminal attached and no watch,
  so signing
  with `commit.gpgsign` and an editor are inside its duration,
  and `ssh-keygen`,
  which Git starts,
  is not listed apart from it.
  The list holds at most 64 steps (`MAX_STEP_RUNS`:
  the most one commit starts is 3 landing attempts times 17 Git starts,
  measured with `strace` on 2026-11-07,
  rounded up to a power of two),
  and `stepsDropped` counts the steps that came after.
  `quietMs` and `longestQuietMs` stay as they were.
  The service's `answered` line ends with `steps="git rev-parse@3+4;..."` (`name@offset+duration`,
  the offset from the request's
  acceptance) and `steps_dropped=<count>`.

The landing's own figures are on the same line and in the record's `landing`:

- `landing_ms` is the whole landing,
  and its phases are `lock_wait_ms` (another program's hold on the index),
  `reads_ms`,
  `objects_ms`,
  `journal_ms`,
  `ref_ms` and `index_update_ms`.
- `reads_ms` is named one read at a time by `reads_config_ms`,
  `reads_branch_ms`,
  `reads_parents_ms`,
  `reads_changed_ms`,
  `reads_content_ms`,
  `reads_witness_ms`,
  `reads_conclusion_ms`,
  `reads_owed_ms` and `reads_follow_ms`,
  which are parts of it and never more than it together.
- `lock_hold_ms` is how long the landing held the index,
  and `queued_behind` is how many requests were still waiting for their answer when it finished,
  the batch's own taken out.
- `syncs_waited` and `syncs_queued` count the syncs the landing waited for and the ones it handed to the background,
  `sync_held_ms` is the time it spent waiting for its own,
  and `syncs="built-index=12;intent-file=queued;..."` names each one with its milliseconds or with `queued`.
  A background batch of a landing's records says in the journal how many it made and what they cost.

The service writes to its standard error,
which the service manager keeps in the journal of its unit.
The unit is `monochromatic-git-policy-cli@<instance>.service`,
and `<instance>` is the `instance` field of the commit's line.
Without a line,
it is 32 hexadecimal digits,
the 128-bit FNV-1a hash of the real path of the worktree's own Git directory
(`.git` in the main worktree,
`.git/worktrees/<name>` in a linked one):

```sh
# sh: the unit instance of the worktree you are in
node --input-type=module -e '
import { realpathSync } from "node:fs";
let hash = 0x6c62272e07bb014262b821756295c58dn;
for (const byte of Buffer.from(realpathSync(process.argv[1]))) { hash ^= BigInt(byte); hash = (hash * 0x0000000001000000000000000000013bn) & ((1n << 128n) - 1n); }
console.log(hash.toString(16).padStart(32, "0"));
' "$(git rev-parse --absolute-git-dir)"
```

`systemctl --user list-units --all 'monochromatic-git-policy-cli@*.service'` lists the instances that exist.
Read the unit around the commit's `startMillis`:

```sh
# sh: the service's lines for one worktree on 2026-11-01 between 22:52 and 22:54
journalctl --user --unit="monochromatic-git-policy-cli@<instance>.service" --since="2026-11-01 22:52:00" --until="2026-11-01 22:54:00"
```

Or find the service's entry for one commit by the `requestId` of its timing line:

```sh
# sh: the service's entry for one request, with its fields
journalctl --user --output=json-pretty CLI_GIT_REQUEST_ID=30cbb78e-677f-41ff-b059-87d9f49d3409
# sh: only what failed or did not go as intended
journalctl --user --unit="monochromatic-git-policy-cli@<instance>.service" --priority=warning
```

The service sends each line to the journal's native socket when its standard error is the journal
(the service manager's `JOURNAL_STREAM` names the very stream the process has as standard error).
The journal then dates the line by the moment it was sent,
not by the moment it got round to reading it,
which on a loaded machine could be seconds later and out of order with the manager's own lines.
A line carries its priority (`err` for something the service needed and could not do,
`warning` for something that did not go as intended or a request that failed,
`info` for the rest) and fields to filter on:
`CLI_GIT_RELEASE`,
`CLI_GIT_INSTANCE`,
`CLI_GIT_AT_MS` (the time of the event),
and on some lines `CLI_GIT_REQUEST_ID`,
`CLI_GIT_BRANCH`,
`CLI_GIT_OUTCOME`,
`CLI_GIT_BATCH` and `CLI_GIT_OID`.
When the journal is not listening or the machine's socket queue is full,
the line goes to standard error instead,
prefixed with its priority as `<n>` for the stream to read,
and a service run by hand writes plain lines to the terminal.
A value with a newline in it reaches the journal as written (the protocol's length-prefixed form);
the line on standard error is one line.
The write to standard error never waits for room:
a line the stream took only in part is finished before the next one starts,
and a line that finds no room is dropped and counted,
and the next entry that gets through says how many
(`CLI_GIT_LOG_DROPPED` on a native entry,
a warning line on the stream).
Every line ends with `at_ms=<milliseconds since the epoch>`,
the time of its event,
on both paths.
The userinfo of any URL in a line is replaced by `<credentials>`.

The lines,
each prefixed `monochromatic-git-policy-cli-service:` on standard error and identified as `monochromatic-git-policy-cli-service` in the journal:

- `started release="<release>" protocol=<n> pid=<pid> instance=<instance>` comes first on every start,
  then `ready`.
  An `instance=any` is a development run.
- `answered request="<id>" branch="<ref>" outcome=<word> ...` comes once per request answered.
  `outcome` is `landed`,
  `already` (a retry of a request that had landed),
  `retry-prepare`,
  `branch-moved` or
  `failed:<code>`.
  `batch`,
  `size` and `position` say which requests landed together.
  The phases,
  in order of time:
  `read_ms` (the client sending its line),
  `queued_ms` (waiting for the batch to start),
  `settle_ms` (finishing a batch a killed service left open),
  `landing_ms` and `answer_ms`;
  `total_ms` is the whole stay
  from the connection being accepted.
  `landing_ms` is the sum of `lock_wait_ms` (taking the index lock:
  flushing the marker to storage,
  `lock_stage_ms`,
  with `lock_stage_kind=pre` when the marker was made while the service was idle and `cold` when the commit made it,
  and waiting for a hold on the index,
  `lock_place_ms`,
  with `lock_holder="Process <pid>"`
  when the evidence names one,
  `lock_looks` for how many times the lock was found held,
  and `lock_found` and `lock_last` for what the first and the last look found:
  `service seq=.. pid=.. state=running|gone|unknown age_ms=..` for a lock with the service's marker,
  `foreign no-marker size=.. age_ms=..` for any other,
  with `open_by=[pid (comm)]` when a process has it open:
  the program's name,
  at most fifteen bytes,
  never its arguments or environment;
  the scan for them is made once,
  with the evidence for the waiting notice,
  and stops after a quarter of a second;
  the marker's birth identity,
  boot ID and PID namespace are not in the line),
  `reads_ms` (the branch reads and comparisons),
  `objects_ms`,
  `journal_ms`,
  `ref_ms`
  (moving the branch) and `index_update_ms` (removing the conclusion state,
  then building and installing the new index);
  records written before 2026-11-04 carry the same duration as `sync_ms`,
  which was never a disk sync;
  `lock_hold_ms` is how long the index lock was held,
  from its placement to its release (the branch move and the index
  build),
  and is no phase of that sum;
  `attempts` is more than one when the branch moved under the batch.
  Each figure is cut to whole milliseconds on its own,
  so a sum can fall short of its total by less than one per term.
  The line never holds the commit's message,
  its author or its address.
- `waited for the index lock batch=<n> result=taken|refused lock_wait_ms=.. lock_stage_ms=.. ...` comes when the lock was
  found held or when flushing the marker or waiting took a second or more,
  at the moment the wait ends;
  it carries the same `lock_*` fields as the `answered` line,
  and stays when the landing fails afterwards.
  A slow commit with a long `lock_stage_ms` and no `lock_looks` lost its time to the disk and not to another program.
- `pushed branch="<ref>" oid=<oid> items=<n> outcome=done|failed|skipped push_ms=<n>` comes once for each push
  the service ran for a branch after commits
  (a failed push is a `warning` and carries Git's words,
  credentials masked,
  after a colon),
  `automatic maintenance ran maintenance_ms=<n>` and `the review of <oid> is done review_ms=<n>`
  say the other work ran,
  and `settled the batch an earlier service left open batch=<n> decision=finish|drop` (a `warning`)
  comes for each batch a restart found open.
- `retire requested by a client; this service is release "<release>" and exits after its batch` comes once for each retire
  request the service reads,
  and a client sends one in every round while the old service is still there,
  so the number of
  these lines is at most `updateRounds` of the commit's line.
  The request carries no release,
  so the client's side is in its line:
  `releasesSeen`,
  `updateRounds`.
- `exiting reason=<idle|retire|error|signal-<n>> uptime_ms=<n>` comes last.
  `idle` is three hours with nothing to do,
  `retire` is a retire request,
  `error` is the service's queue closing,
  and
  `signal-15` is the manager stopping the unit.
  Uptime counts from the start of the process for a signal and from the start of serving for the others.
  A service killed by `SIGKILL` or the kernel's out-of-memory killer writes nothing,
  which is itself the evidence.

To split the slow commit of 2026-11-01 that showed `Updating the commit service...`,
read its line in
`monochromatic-git-policy-cli-commit-timing.jsonl`:
a large `reachMs` with `updateRounds` above zero and two entries in `releasesSeen` is a service
of another release being replaced,
and the journal of the instance shows that service's `retire requested` and `exiting`
lines and the new `started` line;
a large `answerWaitMs` is the queue or a held index,
and the `answered` line of the
request in the journal says which,
and which phase of the landing.

The design,
the measurements and the migration plan are in `doc/planning/cli-git-resident-service.md`.
