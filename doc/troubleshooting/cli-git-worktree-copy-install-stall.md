# cli-git 0.0.1: linked-worktree copy stalls, blocks other worktrees, and leaves an unrecoverable journal

## Symptom

On 2026-09-26,
from a linked worktree of this repository,
the wrapped command

```sh
git worktree add --detach /var/home/user/worktrees/ccc-premerge 5f513eb64
```

staged about 2 GB of ignored files beside the destination
(`/var/home/user/worktrees/.cli-git-worktree-copy-EwlgHt`)
and then made no visible progress and did not exit.

While it ran,
wrapped Git commands in other linked worktrees failed with exit `2` and:

```text
cli-git: timed out waiting for active worktree-copy settlement under "/var/home/user/Monochromatic/.git".
```

The contention record in `doc/troubleshooting/cli-git-linked-worktree-settlement-contention.md` (on `main`)
names the same live owner:
PID `970928` running this `worktree add`.

After the process was stopped,
every later wrapped command in a linked worktree still failed,
now in startup recovery of the leftover journal
`.git/cli-git-worktree-copy/v1/006ecda1-….json`.
Only removing the worktree by hand and deleting the journal,
its `.tmp` sibling,
and the stage directory restored wrapped Git.

Three failure modes are separate defects with separate evidence:

- **Stall:**
  installation after staging is quadratic in the ignored entry count.
- **Blocking while live:**
  commands in other linked worktrees wait on the owner's settlement lock and give up after about one second.
- **Blocking after death:**
  recovery rejects the dead owner's own partial work,
  and a failed installation never removes its journal,
  so every later command replays the failure.

## Root cause

All citations in this section are at pre-fix commit `255de52b4`
(branch `feat/cli-git-concurrent-commits`).
The installed wrapper at incident time was `main`'s build,
which contains the same journaling code:
`git show main:package/git-policy/cli/src/worktree-copy/install.ts` has the same three per-entry
`recordEntryIntent` and `recordCreatedEntry` calls.

### The stall: every installed entry rewrote and synchronized the whole journal twice

`package/git-policy/cli/src/worktree-copy/install.ts:355-386` installs entries one at a time
and records each one twice:

```ts
// package/git-policy/cli/src/worktree-copy/install.ts (255de52b4)
      await recordEntryIntent({
        state: journalState,
        relativePath: entry.relativePath,
      },);
      await createSelectedEntry({
        snapshot,
        destinationRoot,
        entry,
      },);
      // ...
      await recordCreatedEntry({
        state: journalState,
        entry: installed,
      },);
```

`package/git-policy/cli/src/worktree-copy/transaction-journal.ts:72-104` builds a new record holding every intent so far
and writes all of it:

```ts
// package/git-policy/cli/src/worktree-copy/transaction-journal.ts (255de52b4)
  const record: WorktreeCopyJournal = {
    ...state.pending
      .record,
    intendedEntries: [
      ...state.pending
        .record
        .intendedEntries,
      relativePath,
    ],
    phase: 'installing',
  };
  await writeJournal({
```

`recordCreatedEntry` at `transaction-journal.ts:124-160` does the same for `createdEntries`.
`package/git-policy/cli/src/worktree-copy/journal.ts:236-281` serializes the complete record to a temporary file,
synchronizes it,
renames it,
and synchronizes the directory:

```ts
// package/git-policy/cli/src/worktree-copy/journal.ts (255de52b4)
      await handle.writeFile(
        `${JSON.stringify(serializedRecord,)}\n`,
        'utf8',
      );
      await handle.sync();
    }
    await rename(
      temporaryPath,
      path,
    );
    await syncDirectory(dirname(path,),);
```

With `n` selected entries,
entry `k` writes two records of size proportional to `k`,
so installation writes on the order of `n²` bytes and performs `4n` synchronizations.
The stage had already been fully copied,
so the destination grew only as fast as the journal rewrites allowed:
the reported idle state.
The leftover `.tmp` file matches a process stopped inside `writeJournal`,
where nearly all installation time is spent.

### Blocking while live: a pending journal sent unrelated commands into the settlement lock

`package/git-policy/cli/src/worktree-copy/pending-recovery.ts:59-66` takes the lock whenever any journal is pending:

```ts
// package/git-policy/cli/src/worktree-copy/pending-recovery.ts (255de52b4)
  if ((await readPendingWorktreeCopyJournals(commonDir,)).length === 0) {
    // ...
    return;
  }
  await using _settlementLock = await acquireWorktreeCopyLock(commonDir,);
```

A live owner's journal exists for its whole installation phase,
and the owner holds the lock throughout.
`package/git-policy/cli/src/worktree-copy/journal-lock.ts:25-30` bounds the wait to 100 attempts 10 ms apart,
and `journal-lock.ts:514` then throws the quoted timeout.
The feature branch's narrowing of the lock to worktree-creating commands therefore protected unrelated commands
only during staging.
On `main`'s build every forwarded linked-worktree command took the lock,
so they failed during staging too.

The same bounded wait made a second `git worktree add` from a linked worktree fail after about one second
whenever a live copy held settlement,
although `doc/decision/cli-git-concurrent-commits.md` "Locks" gives a proven-live owner a wait with no time limit.

### Blocking after death: recovery rejected the owner's own directories

Installation creates selected directories with a private mode and applies source modes only after every entry exists.
`package/git-policy/cli/src/worktree-copy/install.ts:39,267,387`:

```ts
// package/git-policy/cli/src/worktree-copy/install.ts (255de52b4)
const PRIVATE_DIRECTORY_MODE = 0o700;
// ...
    await mkdir(
      destinationPath,
      { mode: PRIVATE_DIRECTORY_MODE, },
    );
// ...
    await applyEntryModes({
      root: destinationRoot,
      entries: snapshot.entries,
    },);
```

Recovery resumed through the same preflight,
which requires an exact mode match for every existing entry,
`package/git-policy/cli/src/worktree-copy/entry-compare.ts:215-217`:

```ts
// package/git-policy/cli/src/worktree-copy/entry-compare.ts (255de52b4)
  const actualMode = actualStats.mode & PERMISSION_BITS;
  if (actualMode !== entry.mode)
    return false;
```

so `install.ts:121-127` rejected the first directory the dead owner had created,
even when the journal recorded its identity:

```text
cli-git: ignored-state installation failed for "/work/wt/dest".
Rollback retained: "node_modules/.pnpm/pkg-82@1.0.0/node_modules/pkg-82/lib", ... "node_modules".
Cause: cli-git: ignored-state copy would overwrite differing destination entry "node_modules" in "/work/wt/dest".
```

(One stderr line,
wrapped at its sentence boundaries,
with the retained list elided.)

Rollback retained every directory for the same mode reason,
and `package/git-policy/cli/src/worktree-copy/transaction.ts:103-124` removed the journal only after a successful
installation,
so the journal stayed pending and the next command repeated the same failure.
Removing the worktree did not help either:
`package/git-policy/cli/src/worktree-copy/journal-validation.ts:336` resolves the destination with `realpath`,
which fails once it is gone,
and `journal-validation.ts:439` turns that into a recovery failure.

The same missing terminal state affects an ordinary documented collision
(a destination branch that tracks a path the source ignores with different bytes):
the first command exits `2` as specified,
but its journal and stage stay behind and every later linked-worktree command exits `2` with the same message.

### Contributing factor: the copied tree includes cli-git's own logs

The logger writes `node_modules/.monochromatic/<timestamp>.log.jsonl` under the nearest `node_modules` of the
invoking directory
(`package/logger/core/src/sink/file.ts:120-215`),
which is inside the ignored tree the copy selects.
Measured on 2026-09-26 with `du --summarize --bytes` and `find -type f | wc --lines`:

- `/var/home/user/Monochromatic/node_modules/.monochromatic`:
  8,800,332,401 bytes in 499,257 files;
- `/var/home/user/worktrees/translation-repair/node_modules/.monochromatic`:
  713,625,306 bytes in 34,399 files.

A freshly installed linked worktree
(this investigation's worktree after `mise run prepare:pnpm:install` and a Rust build)
held 48,457 ignored entries
(6,311 directories,
40,100 files,
2,046 symbolic links)
totaling 1,164,023,181 bytes.
Accumulated logs multiply both the entry count that the quadratic journal amplified and the bytes staged.
This is recorded,
not changed:
excluding the log directory would add repository-specific copy configuration that the specification rules out,
so it needs a separate decision.

## Verification

Verified on 2026-09-26 with:

- `@monochromatic-dev/git-policy-cli` `0.0.1` built from `255de52b4` (pre-fix) and from the fix commits;
- real Git `2.55.0` on the host and Debian's Git `2.39.5` in the reproduction image;
- Node `v24.18.0` in the image;
- a bounded container:
  `podman run --memory=2g --cpus=2 --tmpfs /work` on `localhost/cli-git-worktree-copy-stall:20260926`
  (`docker.io/library/node:24-slim` plus `git`,
  `strace`,
  and `procps`),
  with the worktree mounted read-only.
  Using `tmpfs` kept the multi-gigabyte journal rewrites off the host SSD.

### Harness

The reproduction script
(kept outside the repository)
creates a main repository,
a linked source worktree whose `node_modules/` holds pnpm-shaped packages
(a `.pnpm/<name>@1.0.0/node_modules/<name>/lib` tree,
hard links from a shared content store,
and a top-level symbolic link per package),
and a second linked worktree.
It runs the built wrapper's `worktree add --detach <destination> HEAD` from the source,
samples the journal directory every 100 ms,
reads the wrapper's `/proc/<pid>/io`,
optionally runs wrapped `git status --short` from the second linked worktree during staging and during
installation,
and optionally `SIGKILL`s the wrapper mid-installation and runs later wrapped commands.

The regression tests are the durable harness:

- `package/git-policy/cli/src/worktree-copy-scale.unit.test.ts`
  bounds the wrapper's own `wchar` for a 2,401-entry copy at 16 MiB.
- `package/git-policy/cli/src/worktree-copy-recovery.unit.test.ts`
  covers recorded and claimed interrupted directories,
  failed installations,
  directory rollback,
  a removed destination,
  and the live-owner case.
- `package/git-policy/cli/src/worktree-copy-interruption.unit.test.ts`
  `SIGKILL`s a process-group-owned creation after installation began,
  and covers torn,
  truncated,
  and corrupt install logs and a removed stage.

### Pre-fix measurements

Installation time grew with the square of the entry count
(staging stayed linear):

- 1,103 entries:
  staging 508 ms,
  installation 1,337 ms;
- 4,403 entries:
  staging 1,308 ms,
  installation 13,008 ms;
- 8,803 entries:
  staging 2,510 ms,
  installation 49,658 ms.

For 4,403 entries the wrapper wrote 3,461,327,796 bytes (`wchar`) in 197,991 write calls,
and a CPU profile attributed 8,938 ms of 15,951 ms to `writeJournal`,
with most of the remainder idle in I/O waits.
Extrapolating the measured square law to the 48,457-entry worktree gives about 25 minutes of installation on
`tmpfs` and about 420 GB of journal writes;
on the btrfs home volume each rewrite also pays two `fsync` calls
(an append plus `fsync` measured 1.02 ms there).

The regression tests failed at commit `beb3ea51c` for these reasons:

- the scale test measured 861,766,361 bytes against the 16 MiB budget;
- both interrupted-directory tests,
  the removed-destination test,
  and the failed-installation test saw the next `git status` exit `2`;
- the live-owner test saw `git status` exit `2` with the settlement timeout.

### Behaviors that now work

Post-fix measurements in the same container:

- 4,403 entries:
  staging 1,653 ms,
  installation 908 ms,
  3,125,875 bytes written;
- 48,403 entries:
  staging 14,256 ms,
  installation 9,528 ms,
  35,172,823 bytes written;
- wrapped `git status --short` in another linked worktree during the owner's installation phase:
  exit `0` in 76 ms,
  owner still running
  (pre-fix:
  exit `2` after 1,184 ms);
- a creation killed after installation began is finished by the next wrapped command,
  and the destination tree equals the source tree;
- a destination on its own mount point
  (`--tmpfs /work/wt/dest`)
  refuses the hard link with `EXDEV`,
  falls back to copying,
  and still completes;
  the destination file has one link and a different inode from the source.

Each fix guard was removed in turn,
the bundle rebuilt,
and the suites run;
every removal failed at least one named test,
and the tree was restored from Git afterwards:

- accepting the transaction's own directories:
  both interrupted-directory tests,
  the killed-creation test,
  and the torn-append test;
- skipping recovery while a live owner holds settlement:
  the live-owner test;
- ending a failed installation:
  the failed-installation and directory-rollback tests;
- discarding an unregistered destination:
  the removed-destination test;
- discarding a missing stage:
  the removed-stage test;
- truncating an unfinished append:
  the truncation test;
- removing empty proven directories on rollback:
  the directory-rollback test.

### Behaviors that still fail closed

- A complete install-log line that is not a valid record:
  exit `2` with `install log is corrupt`,
  journal kept.
- A journal whose stage path escapes the destination's parent or is not private:
  exit `2`,
  journal kept.
- Journal intents the private stage no longer holds:
  exit `2` with `journal intent is absent from private stage`,
  journal kept.

## Verified workarounds

### Upgrade or rebuild cli-git with the fix

This is the complete correction
(commits `725c6df76` through `ef865a280` on `fix/cli-git-worktree-copy-stalls-and-blocks-linked-worktrees`).
Installation appends intents and identities to a log inside the private stage once per 512-entry batch,
hard-links staged files into the destination,
accepts the transaction's own directories on recovery,
ends failed and orphaned transactions,
and lets unrelated commands skip recovery while an owner is live.
Commit `bb91a9f26` applies the decision's lock model to settlement:
a worktree-creating command waits for a proven-live owner without a time limit after one stderr line naming its PID,
retires a dead owner's lock after a re-read,
and gives an owner record without evidence a bounded wait and a diagnostic.
The test
`package/git-policy/cli/src/worktree-copy-settlement-wait.unit.test.ts`
keeps a second creation waiting 2.5 s behind a live holder and then requires both to succeed;
with the live-owner branch replaced by the old immediate timeout,
that test fails.

Tradeoffs:

- A crash inside a batch leaves up to 512 created paths without recorded identities;
  recovery accepts them when they match the stage or are directories the batch claimed,
  but a later rollback retains them instead of removing them,
  and names them.
- A failed installation no longer leaves its journal for a later retry;
  its stage is removed,
  so rerun `git worktree add` after fixing the cause.
- A journal whose worktree was removed or whose stage was deleted is discarded with a notice;
  ignored files in that worktree stay as they were.
- A second worktree creation now waits as long as the first copy runs
  (about 24 s for 48,403 entries in the bounded container).
  A nested wrapped `git worktree add` from a hook that strips `CLI_GIT_WORKTREE_COPY_LEASE` would wait for its own
  ancestor forever;
  the inherited lease is what prevents that,
  as with the landing lock.

### Skip copying for one creation on an older build

```sh
git worktree add --no-worktree-copy --detach <path> <commit>
```

Tradeoff:
the new worktree gets no ignored state,
and this invocation performs no journal recovery.
On an older build,
other linked-worktree commands can use the same flag to skip recovery of an existing leftover journal,
at the cost of leaving it pending.

## What does not work

- **Raising the lock timeout alone.**
  The owner's installation would still take time proportional to the square of the entry count,
  and unrelated commands would wait longer instead of failing.
  The accepted design instead removes the timeout only for proven-live owners,
  after making installation linear and letting unrelated commands skip recovery.
- **Deleting only `settlement.lock`.**
  It races a live owner;
  after the owner is dead the lock is already reclaimed as stale,
  and recovery still fails on the journal.
- **Removing the destination worktree on the pre-fix build.**
  Recovery then fails in `realpath` of the missing destination instead.
- **Batching the full-record rewrites.**
  Rewriting the whole journal once per batch still writes on the order of `n²/b` bytes;
  only appending keeps the cost linear.
- **An earlier kill test that fired as soon as the journal appeared.**
  It usually landed before any destination entry existed,
  so it passed even with recovery's directory acceptance removed;
  the test now waits for the first installed entry.

## Upstream filing artifact

### Upstream filing decision

No external upstream filing applies.
Cli-git is owned by this repository,
`.out-of-scope/` has no matching exemption,
and the fix lands here.

1. **Is it really upstream's fault?**
   No external upstream exists;
   the defects are in this repository's worktree-copy journaling,
   recovery,
   and lock usage.
2. **Can upstream fix it?**
   The owned implementation was fixed.
3. **Are they supporting this use case?**
   Yes;
   linked-worktree ignored-state copying is a documented feature in `package/git-policy/cli/README.md`.
4. **Would the repo welcome our contribution?**
   This is the owning repository.
5. **Will they likely fix it?**
   The fix and regression tests are committed.
6. **Have we prototyped a minimal compatible fix?**
   Yes;
   the regression tests fail before it and pass after,
   and each guard was shown to fail its tests when removed.

Nothing should be posted externally.
The repository commits and this record are the filing artifact.
