# Commit-check route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
`main` at `cf63cacad8`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-commit-checks-route-findings-on-main`
(fast-forwarded to `main` before writing;
the route files are byte-identical to the tip).
No production code was changed and no test was committed.
The scratch fixtures and scripts are in `${HOME}/temp/agent/pi-commit-checks-audit`,
and every quoted result names the fixture that produced it.
Linux only.

Scope:
everything that runs before a commit lands in the check path:

- `package/git-policy/cli/src/native/policy/` (22 production files,
  6,864 lines),
- `candidate/` (23,
  6,841),
- `config/` (13,
  3,863),
- `markdown/` (7,
  1,715),
- `standards_review/` (12,
  2,859),
- `source_guard/` (the runtime file declares only test modules),
- the rules cache and its warm-up (`landing/service_rules_precompile.rs`,
  `service/service_warm_request.rs`,
  `package/forbidden-strings/cli/src/runtime_cache/`),
- the glue that calls them:
  `invocation/wrapped_command.rs`,
  `commit/commit_convergence.rs`,
  `commit/private_index_stage.rs`.

Landing a commit is already audited in
`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
so this page stops where the prepared commit is handed to the service.
The route's closing line is
`landing/service_commit_requests_and_landing_report.rs:169`,
which installs a correction into the worktree after the landing.

Purpose:
the owner requires that commits never fail and that nothing waits forever,
and the checks can also rewrite files.
This page says,
state change by state change,
what a killed process leaves,
what notices it later,
which waits have no ceiling,
which locks exist and whether they could go,
and what two concurrent commands can do to one repository.
Each finding is a sequence and a wrong result,
marked **reproduced** when a fixture produced it and
**read from the code** when only the code establishes it.

## The route, step by step

1.  `invocation/entry.rs:271-294`
    decides the invocation:
    the self-invocation refusal,
    the real Git,
    and one `ShippedChecks` value that reads nothing yet.
2.  `wrapped_command.rs:246-316` prepares a guarded command:
    the repository location (in-process layout or one Git query),
    `recover_for_command` (the landing recovery,
    whose state is the landing route's),
    the configuration file (`config/invocation_config.rs:46-67`),
    the scanner and Markdown options,
    and,
    for `git add` only,
    the candidate request
    (`wrapped_command.rs:306-310`).
    No candidate byte is read here.
3.  A real commit (`wrapped_command.rs:337-381`)
    goes to `commit/commit_command.rs`,
    which captures the invocation:
    the real index is copied beside itself,
    one `git config --list` snapshot is taken
    (`config/config_snapshot.rs:632-640`),
    and the base is recorded.
    `landing/service_prepare.rs:289-306` then runs the whole ordered policy pass
    (`policy/policy_pass.rs:139-197`) over the commit's private index,
    with each correction written into that index
    (`commit/private_index_stage.rs:74-90`),
    until the loop settles (`policy/policy_convergence.rs:182-211`).
    The candidate listing is `CandidateRequest::StagedAtBase`
    (`commit/commit_transaction.rs:310`),
    so content policies read the private index,
    never the worktree.
4.  Every other non-dry-run command runs one pre-forward pass
    (`wrapped_command.rs:474-489`).
    For `git add`,
    the first content policy that reads candidates pays for the prediction
    (`policy/policy_content.rs:307-366`):
    a private index directory is created and filled
    (`candidate/candidate_prediction.rs:431`),
    the caller's `git add` is replayed on it
    (`candidate/candidate_prediction.rs:279-300`),
    and the changed entries become the candidates.
    A real push also discovers its scope and runs the manual-push gate
    (`wrapped_command.rs:490-530`).
5.  Policies read candidate bytes through one long-lived
    `git cat-file --batch` process
    (`candidate/candidate_reader.rs:199-260`,
    `candidate/candidate_store.rs:542-575`),
    scan them through the linked forbidden-strings scanner
    (`candidate/scanner_run.rs:169-215`),
    and run Markdown autofix through the linked linter on a guarded thread
    (`markdown/markdown_linter.rs:258-317`).
6.  When the pass ends with no stop,
    the checks are dropped and the wrapper replaces itself with real Git
    (`invocation/entry.rs:337-347`,
    `invocation/forwarding.rs:230-252`).
    The drop is what removes the private index directory
    (`candidate/candidate_private_index.rs:285-288`).

The commit path's policy loop never writes the real index or the worktree.
The direct `fix` command does write the worktree:
`policy/direct_fix.rs:280-305` installs the converged corrections through
`policy/direct_fix_install.rs`.

## What a process killed at each step leaves behind

The check path has six durable state changes.
Landing's own states (the transaction journal,
the scratch repository,
the side ref,
the index lock) are the landing route's and are not repeated here.

1.  The private index directory
    (`candidate/candidate_private_index.rs:217-240`):
    `mkdir` of `.git/cli-git-add-policy-<pid>-<nanoseconds>-<count>`,
    then a copy of the real index with its access and modification times.
    Killed before the copy:
    an empty directory.
    Killed after:
    the directory with a full index copy.
    The only remover is `Drop`.
    Nothing else notices:
    no journal,
    no notice,
    no reaper
    (`rg 'cli-git-add-policy'` finds the prefix,
    its tests,
    and documents).
    Work is not lost,
    but the copy stays for the life of the worktree.
2.  Corrections written into the commit's private index
    (`commit/private_index_stage.rs:74-90`):
    one `git hash-object -w` and one `git update-index` per corrected path.
    Killed between the two leaves an unreferenced blob in the private object
    store;
    killed after leaves the entry as well.
    Both live in the transaction's own directory and object store,
    which the landing route's recovery removes or reports.
    The real index and the worktree are untouched.
3.  The rules cache artifact
    (`package/forbidden-strings/cli/src/runtime_cache/publish.rs:131-149`):
    a private same-directory temporary,
    `write_all`,
    `sync_all`,
    `rename`.
    Killed before the rename:
    `.rules.bin.<pid>.<sequence>.tmp` in the digest's directory.
    No reader ever opens it,
    and the next publish uses a new name;
    it is inert,
    but nothing removes it.
    Killed after the rename:
    a complete artifact that every later reader validates against the
    source digest and the scanner version,
    so a half-written artifact can never be read.
4.  The service's remembered repository for the warm
    (`landing/service_rules_precompile.rs:229-240`):
    write `rules-repository.new`,
    then rename over `rules-repository`.
    Killed before the rename leaves the `.new` file;
    the next write replaces it whole.
    Killed after leaves a complete name.
    A wrong name only costs a warm after a restart.
5.  The direct fix's prepared replacements
    (`policy/direct_fix_install.rs:250-320`):
    `read(destination)`,
    compare with the bytes the fix read,
    copy the destination to `<prefix>.old`,
    write the corrected bytes to `<prefix>.new`,
    then rename `.new` over the destination
    (`:328-345`),
    then `cleanup` (`:152-163`).
    Killed before any rename:
    the worktree is unchanged and two siblings per selected file stay.
    Killed during the renames:
    some files corrected,
    some not,
    and every prepared file keeps both siblings.
    Nothing notices.
    The next `fix` reads the current bytes and carries on;
    the siblings are ordinary worktree files to every later scope.
6.  The standards-review log entry
    (`standards_review/standards_review_append.rs:60-99`):
    one `write_all` on an `O_APPEND` descriptor,
    after a read that checks whether the commit's short name is already there.
    Killed before the write:
    the item stays unsettled and the next attempt writes it.
    Killed after:
    the next attempt finds the short name and writes nothing.
    The read and the append are not one step,
    but only one service reviews one landed commit,
    so no second writer is reachable in this reading.

## Waits and their ceilings

- The Markdown linter:
  a 60 s quiet window (`markdown/markdown_linter.rs:81`),
  a 240 s ceiling (`service/wait_ceilings.rs:484`,
  `LINT_CEILING`),
  and a 50 ms poll (`markdown/markdown_linter_guard.rs`,
  `NOTICE_POLL`).
  A linter that is abandoned stops later candidates in the same command
  and the policy fails closed with `policy-incomplete`;
  the thread stays until the process ends.
- The standards-review gateway:
  a 10 s quiet window in the service (`service/service_review.rs:52`),
  30 s for the script's default (`standards_review/standards_review_client.rs:69`),
  a 60 s call ceiling (`wait_ceilings.rs:502`,
  `REVIEW_CALL_CEILING`),
  a 120 s whole-review ceiling (`wait_ceilings.rs:520`,
  `REVIEW_CEILING`),
  and a 7 s handoff wait after a job that released the runner
  (`wait_ceilings.rs:553`,
  `SHORT_EXTRA_WAIT`;
  `standards_review/standards_review_run.rs:533-547`).
- The candidate object reader:
  one `ChildWatch` per request at `OBJECT_STEP_CEILING` (600 s,
  `candidate/candidate_reader.rs:252`,
  `wait_ceilings.rs:395`);
  a request that crosses it is ended and reported.
- Service-side Git steps:
  120 s for plumbing,
  600 s for objects
  (`wait_ceilings.rs:378`,
  `:395`),
  with the branch write's rules in
  `transaction/transaction_git_step_bounds.rs`.
- **No ceiling at all** on the check path's own Git reads:
  `repo_read/git_metadata.rs:77-96` starts the child with
  `Command::output()` and waits for it.
  Every candidate listing (`candidate/candidate_store.rs:247-...`),
  the `git add` replay (`candidate/candidate_prediction.rs:290`,
  `:330`),
  the real-index query (`candidate/candidate_prediction_git_runs.rs:78-115`),
  and the rule facts (`git status`,
  the sequencer and branch-name queries) go through it.
  No watch,
  no step bound,
  no notice.
  The configuration snapshot is not one of these:
  it uses `transaction_git::run_git`
  (`config/config_snapshot.rs:612-614`)
  and therefore the step ceilings.
- **No ceiling** on the warm request:
  `service/service_warm_request.rs:107-110` calls `UnixStream::connect`
  and `write_all` with no timeout.
  A connect only blocks while a listener's backlog is full,
  and the line fits the socket buffer,
  so this is bounded in practice by the kernel;
  it is named here because the code sets no bound.
- **No ceiling** on the local reads and writes:
  the configuration file,
  the rules files,
  the standard-review log's read and append.
  The review run's own comment says so
  (`standards_review/standards_review_run.rs` module doc:
  "The one step with no judge is the local write of the log entry").
- The policy loop is bounded by structure,
  not by time:
  at most eight changed passes and a repeat of an earlier state
  (`policy/policy_convergence.rs:30`,
  `:182-211`).
- The private-index prediction's chunking bounds argument counts,
  not time:
  2,048 paths per listing (`candidate/candidate_pathspec_chunk_size.rs`).

## Locks and why each exists

There is no file lock,
`flock`,
`RwLock` or condition variable in the
check path.
The one production lock is in the candidate layer:

- `candidate/candidate_store_commit_listings.rs:55`,
  `static LISTED: Mutex<Option<(ObjectId, Arc<Vec<TreeRecord>>)>>`.
  It caches the last commit-tree listing for the whole process,
  because the policies before a commit and the gate after it list the same
  base commit through two stores.
  It is taken twice per call (`:119`,
  `:129`);
  the listing itself runs outside the lock.
  Its reason is written but not in the `Lock kept:` form the inventory guard
  looks for.
  A per-store cache gives the same benefit with no lock,
  and a process-global cache can be an `Arc` swapped with one atomic store,
  since a listing is immutable for the commit name it is keyed by.
  **No lock is needed for correctness here**;
  the lock only protects a cache.

Test-only locks:
`standards_review/standards_review_run.rs:392` (`handoff_gate`,
`#[cfg(test)]`),
`standards_review/standards_review_mock.rs` (the mock server),
and the repository's other test support.
They do not ship.

The guard that should keep it this way,
`source_guard/service_lock_inventory_tests.rs:36-50`,
covers file names that start with
`service_`,
`claim_time`,
`discarded_`,
`index_lock_`,
`journald_native`,
`git_version`,
`config_`,
`process_identity`,
`private_storage`,
`recovery_files`,
`work_item`.
It does not cover `candidate_`,
`policy_`,
`markdown_`,
`standards_review_`,
`commit_`,
`foundation_`,
`transaction_`,
`worktree_`,
or
`worktree_copying_`,
which is where the route lives.

## Races between two concurrent commands

- Two `fix` commands in one worktree:
  both read the files,
  both prepare replacements,
  and both rename.
  The later rename wins,
  each verifies only the real index (which neither
  writes),
  and both report success.
  With the same rule set the results agree on the bytes each read;
  with different rule sets,
  or with a person's save between the read and the
  rename,
  the later rename silently replaces bytes the other writer wrote
  (finding 1).
- A `fix` and a commit in one worktree:
  the commit captures the worktree for `-a` and explicit paths while the fix
  renames files one by one.
  The capture can hold a mix of corrected and uncorrected files,
  which is what any two concurrent worktree writers can produce.
  The fix never writes the index,
  so it cannot change what is staged.
- Two `git add` commands in one worktree:
  each prediction gets its own directory
  (`pid` + nanoseconds + a per-process counter,
  `candidate/candidate_private_index.rs:53-77`),
  and the forwarded adds serialize on Git's index lock
  (`doc/decision/cli-git-concurrent-commits.md`,
  "Locks").
  The predictions can disagree with the final index,
  but each add's candidates are its own delta and the commit re-checks the
  index it commits.
- Two commits in one worktree:
  preparation copies the real index read-only and reads only objects;
  the service serializes landing.
  The static `LISTED` cache is keyed by complete commit name only,
  so two repositories in one service process cannot confuse a tree:
  a name denotes one content at one hash algorithm.
- Two rules-cache writers:
  different digests are different directories;
  the same digest compiles to the same bytes and the rename publishes one
  complete file.
  A killed temporary is ignored by both.
- The standards-review append's read-then-append is not atomic,
  but one landed commit is reviewed by one service,
  so no second appender for the same commit is reachable here.
- A commit's configuration snapshot is one read;
  a person saving the settings file during a commit is not seen by that
  commit.
  A truncate-in-place save that a commit catches yields a parse failure
  (exit 2,
  `config-invalid`),
  not a silently different policy set;
  a save that leaves a complete older document would be seen as that
  document.
  This was read,
  not reproduced.

## Reproductions

The wrapper probes used the debug binary built from this revision
(`mise run //package/git-policy/cli:native:build`,
1 m 22 s),
started by path through a `git` symlink in
`${HOME}/temp/agent/pi-commit-checks-audit/bin`,
with the real Git found at `/usr/bin/git`.
The seam tests were added temporarily to the crate's own sidecars,
run through `native:test:host`,
and removed before this page was written
(`git status` is clean apart from this page).

### A hanging fsmonitor hangs the check, and the kill leaves the private index

`fixture-git-hang`:
one tracked file and a `core.fsmonitor` script that
sleeps.
The positive control,
the same add with the hook unset:

```text
== positive control: add with no fsmonitor
exit=0
private index dirs now: 0
```

With the hook set,
the wrapper's `git add` never returns and prints nothing;
`timeout 10` is what ended it:

```text
== add with a fsmonitor that never returns
wrapper add exit=124 (124 is the timeout's kill)
--- stdout
--- stderr
private index dirs after the kill:
cli-git-add-policy-2836597-1794194466735565194-0
== take the hook away and add again
second add exit=0
private index dirs after the second add:
cli-git-add-policy-2836597-1794194466735565194-0
```

Plain Git blocks on the same hook,
which is where the wrapper's listing runs:

```text
add exit=124
ls-files exit=124
diff-index exit=124
```

The leftover directory is `0700` and holds the index copy:

```text
700 .git/cli-git-add-policy-2836597-1794194466735565194-0
index copy equals the real index: no
```

### A 256 MiB candidate costs 534 MiB

`fixture-big-blob`:
a 256 MiB sparse file added with the wrapper,
measured by `/usr/bin/time -v`:

```text
Elapsed (wall clock) time (h:mm:ss or m:ss): 0:01.53
Maximum resident set size (kbytes): 546816
Exit status: 0
```

The read is `read_content`'s `take(size).read_to_end`
(`candidate/candidate_batch.rs:334-337`)
plus the `Rc` copy the store keeps,
so the peak is about twice the object's size and grows with it.
The OOM kill itself was not reproduced:
this host's user scopes did not enforce the limit
(`systemd-run --user --scope --property MemoryMax=128M -- python3 -c
'x=bytearray(300*1024*1024)'` exited 0).

### A write that arrives after preparation is overwritten

Seam:
`prepare_replacements`,
then a write to the destination,
then
`install_prepared`.
Both functions are public and the index is unchanged.

```text
AUDIT late-write install result: Ok(())
AUDIT a.txt after install: [97, 10]
```

The fix read `a` and proposed `a\n`;
the newer `newer edit\n` is gone,
`fix` reported success,
and the `.old` file that held the pre-fix bytes is removed by `cleanup`.

### A fix turns a 0600 file into 0644

```text
AUDIT mode install result: Ok(()); a.txt mode now 644
```

The fixture set `a.txt` to `0600` before
`install_corrections`.

### A killed fix leaves its siblings

Seam:
`prepare_replacements`,
then no drop and no cleanup
(`mem::forget`),
which is what a kill leaves.

```text
AUDIT siblings after a simulated kill: [".cli-git-direct-fix-2851007-1794194610430881284-1.old", ".cli-git-direct-fix-2851007-1794194610430881284-1.new", ".cli-git-direct-fix-2851007-1794194610430683494-0.old", ".cli-git-direct-fix-2851007-1794194610430683494-0.new"]
AUDIT a.txt after the kill: [97]
```

### A killed private index stays

Seam:
create a private index and forget it.

```text
AUDIT private index left after a simulated kill: /var/mnt/encrypted/cargo-build/toolchain/nightly-2026-11-08/87/633e19a8a16765/debug/build/monochromatic-git-policy-cli/test-fixtures/native-unit-2851004-audit-private-index-kill/cli-git-add-policy-2851004-1794194610416528966-0 exists=true
```

### The lock guard does not see the check route

Planted a real `static ... Mutex` in `candidate/candidate_store.rs`.
All four inventory tests passed:

```text
PASS source_guard::service_lock_inventory_tests::the_scan_finds_a_lock_in_text_that_has_one
PASS source_guard::service_lock_inventory_tests::every_covered_prefix_reaches_a_source_in_some_folder
PASS source_guard::service_lock_inventory_tests::a_lock_stands_only_in_a_file_on_the_inventory
PASS source_guard::service_lock_inventory_tests::every_kept_lock_has_its_reason_beside_it_and_the_inventory_has_no_stale_entry
Summary [1.199s] 4 tests run: 4 passed
```

The positive control,
the same line in `service/service_reply.rs`:

```text
locks outside the inventory (remove them, or keep them with the reason and list the file): ["service/service_reply.rs"]
```

## Findings ranked by what they risk losing

### 1. A fix overwrites a write that arrived after it read the file

`policy/direct_fix_install.rs:250-292`,
`:328-345`,
`:396-415`.
Sequence:

1.  `fix` reads each selected file and compares it with the bytes the policy
    read (`:263-275`),
    then writes `.old` and `.new` beside it.
2.  The person's editor,
    a formatter,
    or another agent saves the file.
3.  `install_prepared` renames `.new` over the destination (`:336`)
    without reading it again,
    and verifies only the real index
    (`:341-348`).
4.  `cleanup` removes `.old` (`:152-163`),
    so the overwritten bytes are gone
    by the time the command returns,
    and the summary names the file as
    corrected.

Wrong result:
the newest bytes of a file the person was editing are lost silently.
This is the owner's "never discard user data" broken by the route itself.
Reproduced above.
Suggestion (suggestion):
re-read the destination immediately before its rename and refuse when it no
longer equals `original`,
restoring the files already replaced as a failed
index check already does;
a per-file lock taken only for prepare-and-rename is the owner's
"smallest unit" exception if a re-read is not enough.

### 2. A fix replaces a private file with a world-readable one

`policy/direct_fix_install.rs:210-231`.
`write_new` creates `.new` with `0755` or `0644` before the umask,
and the rename replaces the destination inode,
so a `0600` file becomes `0644`.
Sequence:

1.  `fix` selects an untracked,
    non-ignored file the person keeps at `0600`
    (a token file,
    a key,
    a local configuration).
2.  It prepares the correction and writes `.new` as a regular file.
3.  The rename installs `.new`,
    with its own mode.

Wrong result:
a file kept private becomes readable by every local user,
and the command reports success.
Reproduced above.
The installed TypeScript wrapper did the same,
so this is not a parity regression,
but it is a confidentiality loss all the same.
Suggestion (suggestion):
read the destination's mode and apply it to `.new` before the rename
(masked to permission bits),
or create `.new` with `0600` and `fchmod` the
destination's mode after writing the content.

### 3. A check waits on Git with no ceiling

`repo_read/git_metadata.rs:77-96`.
Every candidate listing,
the `git add` replay,
the real-index query,
and the rule facts call `Command::output()` and wait for the child.
Nothing watches the child,
so nothing notices a Git that never returns,
and there is no waiting notice.
Sequence:

1.  A repository configures `core.fsmonitor` to a program that hangs,
    or a Git read blocks on an unresponsive mount or index.
2.  A `git add` (or the commit's first listing) starts its check.
3.  The listing never returns;
    the wrapper prints nothing and never exits.
4.  The person kills it,
    which skips every `Drop` and leaves the private index (finding 6).

Wrong result:
the command never returns,
against the owner's newest wait rule,
and a kill is the only way out.
Reproduced at the Git level and at the wrapper level above.
Owner decision touched:
`package/git-policy/cli/SPEC.md`,
"Every wait has a bound"
(owner,
verbatim:
"nothing should ever wait forever or indefinitely";
at a bound:
"Return with an error",
naming what blocked it,
and "No,
keep it for a retry" for saved work),
which overrides "Commits should never fail" for a command that hits a bound.
Suggestion (suggestion):
run these reads through the same bounded step runner the service uses
(or a `ChildWatch` at the plumbing/object ceiling),
and at the bound print the waiting notice and exit 2 naming the command and
the repository;
the staged index and worktree need no change.

### 4. Candidate bytes are read whole with no size cap

`candidate/candidate_batch.rs:334-337`,
`candidate/candidate_store.rs:542-575`.
`read_content` reads the object's declared size into a growing `Vec`,
and the store keeps it for the invocation;
nothing caps it,
and the object's size comes from Git.
Sequence:

1.  The person stages a large asset (a video,
    a dataset,
    a database dump).
2.  Final-newline (a `warn` policy that runs on every trigger,
    `policy/policy_final_newline.rs:250-280`) reads every candidate's bytes;
    the forbidden-strings and Markdown policies read them too.
3.  Peak memory is about twice the object's size
    (measured:
    546,816 KiB for 256 MiB).
4.  Under a memory cap or on a full host the process is killed
    (the OOM killer is on the owner's outside-our-control list,
    but reading the whole object is ours),
    with no policy message,
    and the kill leaves the private index (finding 6).

Wrong result:
a commit of large files fails by being killed,
or pushes the host into memory pressure,
instead of naming the file and refusing the check.
Reproduced as a measurement above;
the kill was not reproduced because the host did not enforce the cap.
Suggestion (suggestion):
cap the bytes a content policy reads and report a named
`policy-incomplete` finding above the cap
(with the remedy to commit with that policy off),
or stream a check that needs only a prefix.

### 5. A killed fix leaves sibling backups that a later pass treats as content

`policy/direct_fix_install.rs:283-300`,
`:152-163`.
Sequence:

1.  `fix` prepares `.old` and `.new` beside a selected file.
2.  The process is killed before the renames or during them.
3.  Both siblings stay;
    the next `fix`'s scope is the whole worktree
    (`CandidateRequest::Direct`,
    which stages `:/` with `git add --all`),
    so the siblings become candidates and the `.old` copy of the file is
    "corrected" and reported.
4.  A later `git add` of such a name publishes a duplicate of the file under
    a name no ignore rule is likely to match.

Wrong result:
the worktree gains files the person did not create,
and a copy of a private file can be published under a new name.
Reproduced above (the siblings);
the later selection and publication are read from the code.
Suggestion (suggestion):
put the siblings under the Git directory with the private index,
or record prepared replacements in a durable journal,
so the next run removes or resumes them instead of reading them as content.

### 6. A killed check leaves its private index copy for ever

`candidate/candidate_private_index.rs:217-240`,
`:285-288`.
Sequence:

1.  Any `git add` (or commit preparation) creates
    `.git/cli-git-add-policy-<pid>-<nanoseconds>-<count>` and copies the
    index.
2.  The process is killed (SIGTERM,
    SIGKILL,
    the OOM killer,
    a closed
    terminal) before `Drop`.
3.  The directory and its index copy stay;
    no later command removes them
    (`rg 'cli-git-add-policy'` finds no reaper).

Wrong result:
unbounded growth in every worktree's Git directory;
the copy holds the repository's index (path names and stat data),
and enough of them fill the disk,
after which commits fail.
The handover's native-cutover measurement already counted 39 such
directories from July to October.
Reproduced above.
Suggestion (suggestion):
reap a directory whose owner is dead at the start of the next candidate
read,
as the copy route already reaps its journals,
or use one directory per process that is removed on start and on exit.

### 7. The lock inventory guard does not cover the check route

`source_guard/service_lock_inventory_tests.rs:36-50`,
`:86-88`,
`candidate/candidate_store_commit_listings.rs:55`.
Sequence:

1.  A developer adds a lock to any `candidate_`,
    `policy_`,
    `markdown_`,
    `standards_review_`,
    `commit_`,
    `foundation_`,
    `transaction_`,
    `worktree_` or `worktree_copying_` file.
2.  `a_lock_stands_only_in_a_file_on_the_inventory` reads the source and
    passes,
    because the file name is outside `COVERED_PREFIXES`.
3.  The lock ships without the reason the inventory exists to demand.

Wrong result:
the owner's "Use as few locks as possible" is enforced only for part of the
crate,
and the control that promises it cannot reports green.
The route's existing production lock
(`LISTED`,
a process-global tree-listing cache) is itself outside the guard;
a per-store cache or an atomic `Arc` swap removes it.
Reproduced above with a planted lock and its positive control.
Suggestion (suggestion):
derive the covered set from the strict module tree (the guard already has
`classified_sources`) rather than from file-name prefixes,
and keep an explicit test-only exception list.

## What I checked and found sound

- The commit path never writes the real index or the worktree:
  corrections go to the private commit index
  (`commit/private_index_stage.rs:74-90`),
  and the post-landing install is the landing route
  (`landing/service_commit_requests_and_landing_report.rs:169`).
  The direct fix is the only worktree writer in this route,
  which is why findings 1,
  2 and 5 are there.
- Candidate bytes come from objects,
  never the live worktree,
  so an edit after staging cannot change what the policies read
  (`candidate/candidate_store.rs:542-575`).
- A candidate version invalidated by a fix or replay is refused,
  not read
  (`candidate/candidate_store.rs:542-550`),
  so a later read cannot use bytes from before a correction.
- The known-content skip compares blob object names only
  (`candidate/candidate_known_content.rs`),
  so one changed byte is scanned;
  a corrected file is excluded from the
  skip.
- The rules cache key is the SHA-256 of the joined rules text;
  the artifact is a private temporary,
  `sync_all`,
  then rename;
  readers validate the embedded digest,
  scanner version and platform;
  a missing,
  unreadable,
  corrupt or mismatched artifact recompiles and
  repairs,
  and a failed revalidation refuses to publish
  (`package/forbidden-strings/cli/src/runtime_cache/mod.rs:75-120`,
  `publish.rs:131-149`).
  A killed compile leaves a temporary that no reader opens.
- The policy loop cannot run away:
  eight changed passes,
  a repeated-state check,
  and a pass limit and cycle
  reported as engine failures
  (`policy/policy_convergence.rs:30`,
  `:182-211`).
- The Markdown linter is bounded and contains panics:
  quiet window,
  ceiling,
  output-size bounds,
  and an abandoned run that stops
  later candidates and fails the policy closed
  (`markdown/markdown_linter.rs:258-317`,
  `markdown/markdown_linter_guard.rs`).
- The standards review is contained:
  it runs on its own thread with a whole-run ceiling,
  a panicking or timed-out job leaves the item unsettled rather than claimed,
  and the log entry is one append that a retry recognises by the commit's
  short name
  (`standards_review/standards_review_run.rs:470-556`,
  `standards_review/standards_review_append.rs:60-99`).
- Scan failures are fail-closed:
  missing rules,
  an unrepresentable pathname,
  a matcher failure and a
  pathname with a line break all end the policy as an engine failure,
  never as clean (`policy/policy_forbidden_strings.rs`,
  `candidate/scanner_adapter.rs`).
- The configuration snapshot either reproduces Git's own answer for the
  keys it models or returns `Unhandled` and lets Git answer
  (`config/config_snapshot.rs:66-110`);
  no guessed value reaches a policy.
- The private index is `0700`,
  keeps the real index's access and
  modification times,
  and lives on the same filesystem as the real index
  (`candidate/candidate_private_index.rs:118-200`).
- The source guard's scans are textual,
  but they read the module tree from
  the manifest and fail when any `.rs` file under the source root is
  unreachable from a declared module
  (`source_guard/source_scan_tests.rs`,
  `source_scan_tree_tests.rs:330-360`),
  and no rule fires on this revision's release code.
- The lock inventory has no stale entries and its positive controls fire
  (`source_guard/service_lock_inventory_tests.rs:150-190`).

## What I could not determine

- The OOM kill of finding 4.
  The measured read is linear and uncapped,
  but this host's user scopes did not enforce `MemoryMax`
  (the python control above survived a 128 MiB cap),
  so the kill is inferred from the measurement,
  not observed.
- Whether a `fix` and a commit in one worktree ever capture a sibling file.
  `-a` stages tracked paths only and this repository's `add-explicit`
  rejects `git add -A`;
  a repository without that policy can stage the siblings,
  which is read,
  not reproduced.
- What a kill during `install_prepared`'s rollback leaves.
  The tests exercise a failed restore and a lost backup
  (`direct_fix_install_tests.rs:290-340`),
  not a kill in that loop.
- Whether the static `LISTED` cache can answer across two object stores
  whose replace refs differ inside one service process.
  A commit name fixes its tree and the store never passes
  `--no-replace-objects`,
  so a concurrent `git replace` is the only way this could differ;
  it was not exercised.
- Whether two services of one repository can both review one landed commit.
  In this reading one commit lands through one service and one item is
  reviewed once;
  the append's read-then-write would duplicate only if that
  changed.
- Whether the installed release has the same code.
  Every probe used the debug binary built from this revision;
  the installed wrapper and its running service were not touched.

## What to inspect and how to respond

Inspect,
in this order:

1.  the two direct-fix sequences (findings 1 and 2) against
    `policy/direct_fix_install.rs:250-345`;
2.  the unbounded reader `repo_read/git_metadata.rs:77-96`,
    with the fsmonitor fixture in
    `${HOME}/temp/agent/pi-commit-checks-audit/fixture-git-hang`;
3.  the cap-free read `candidate/candidate_batch.rs:334-337`,
    with the 534 MiB measurement in `fixture-big-blob`;
4.  the two leak points (findings 5 and 6) and the guard gap (finding 7).

Respond with which findings to fix first.
This audit changed no production code,
so no other action is pending.
