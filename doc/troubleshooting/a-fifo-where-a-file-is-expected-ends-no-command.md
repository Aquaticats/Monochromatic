# A named pipe where a rules file or the index lock is expected holds a wrapped Git command for ever

What:
a `git add`,
`git commit`,
`git checkout`,
`git merge`,
`git pull`,
`git switch`,
`git rebase` or `git reset` run through
`monochromatic-git-policy-cli 0.6.1-alpha.5` never ends when a named pipe (a FIFO) stands at a path the wrapper
reads as a file:
`.git/index.lock` or a forbidden-strings rules file.
Why:
`open(2)` on a FIFO that no writer has opened waits for a partner before it returns,
so no read ever starts and no bound the wrapper sets elsewhere is reached.

## Symptom

With a FIFO at the rules file's name,
a wrapped command that moves the worktree prints nothing and ends nothing,
while the same command under `/usr/bin/git` answers at once.
Measured on 2026-11-09 with the branch's debug build
(`package/git-policy/cli/target/debug/monochromatic-git-policy-cli`) and native Git in the same fixture:

- Wrapped `git checkout other` with a FIFO at `forbidden-strings.local.txt`:
  `rc=124` at the 30 s limit,
  branch
  unchanged (`main`),
  the process in `S` with `/proc/<pid>/wchan` = `wait_for_partner`.
- Wrapped `git checkout other` with no FIFO:
  `rc=0` in 0.1 s.
  Native Git with the FIFO present:
  `rc=0` in 0.0 s.
- Wrapped `git add`,
  `git commit` and `check --all` with a FIFO at the `rulesFile` the configuration names:
  `rc=124` at 30 s each.
- Wrapped `git add` with a FIFO at `.git/index.lock`:
  still running after 130 s (the landing lock ceiling is 60 s),
  `wchan` = `wait_for_partner`.
  A wrapped `git commit` started beside it also outlived 130 s,
  its main thread in
  `unix_stream_read_generic` waiting on the service's answer.
  Native Git with the same FIFO:
  `rc=128` in 0.0 s with
  `fatal: Unable to create '<git dir>/index.lock': File exists.`
- The same FIFO at `.git/index.lock` also stops the commit service's own worker:
  its thread waits in
  `wait_for_partner`,
  so every later commit of that worktree queues behind it.

`wait_for_partner` is the kernel function `open` of a FIFO waits in (`fs/pipe.c`);
it names the moment before any read,
which is why no read-side bound applies.

## Root cause

`open(2)` of a FIFO read-only without `O_NONBLOCK` blocks until some writer opens the other end.
The standard library's `File::open` and the `std::fs::read` built on it do exactly that,
so a path whose holder is a FIFO hangs the caller before a byte is read.
Four sites did that,
all on the commit,
merge and worktree paths.

### The index lock's holder and marker (git-policy-cli)

`package/git-policy/cli/src/native/landing/service_index_wait.rs` looked at the lock with a plain open:

```rust
let mut file = match std::fs::File::open(lock) {
    Ok(file) => file,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Holder::Nobody,
    Err(_) => return Holder::Foreign,
};
```

`package/git-policy/cli/src/native/landing/service_index_marker.rs` read the lock's marker the same way:

```rust
std::fs::File::open(path).ok()?.take(MARKER_LIMIT).read_to_string(&mut text).ok()?;
```

The sibling readers of the same lock already opened without blocking
(`O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC`):
`package/git-policy/cli/src/native/landing/service_index_claim.rs:282`,
`package/git-policy/cli/src/native/landing/service_index_claim.rs:316` and
`package/git-policy/cli/src/native/landing/service_index_reclaim.rs:243`,
which `service_index_unheld_removal.rs` reaches through `open_to_judge`.
The two plain opens were the only ones left.

### Which of the two the service thread was in

The audit that found this could not say which site the service's thread waited in,
because both open the same
path with the same flags.
It was isolated on 2026-11-09 with a stack of every thread while a FIFO stood at `.git/index.lock`
(`${HOME}/temp/agent/pi-fifo-fixes/repro/isolate-index-lock.sh`,
`eu-stack -p <pid>`,
taken 8 s after the FIFO
appeared).
`ptrace_scope` is `0` here,
so the stacks were readable.

The wrapped `git add`'s only thread:

```text
#5  std::fs::File::open::<&std::path::Path>
#6  git_policy_cli::landing::service_index_wait::holder_after_pauses
#7  git_policy_cli::landing::service_index_wait::holder_of
#8  git_policy_cli::landing::service_index_wait::wait_with_watches
#9  git_policy_cli::landing::service_index_wait::wait_for_service_hold_within
#10 git_policy_cli::landing::service_index_wait::wait_for_service_hold
#11 git_policy_cli::landing::service_index_wait::wait_for_service_hold_here
#12 git_policy_cli::lock::index_writer_forward::run_coordinated_forward
#13 git_policy_cli::invocation::entry_run_forwarded_actions::run_coordinated_action
```

The service's worker thread:

```text
#5  std::fs::File::open::<&std::path::Path>
#6  git_policy_cli::landing::service_index_marker::marker_of
#7  git_policy_cli::landing::service_index_found::describe_lock
#8  git_policy_cli::landing::service_index_attempts::wait_and_place
#9  git_policy_cli::landing::service_index_lock::acquire_placing
#10 git_policy_cli::landing::service_index_lock::acquire_noting_wait_within
#11 git_policy_cli::landing::service_land_take_index_lock::take_index_lock
#12 git_policy_cli::landing::service_land_batch_body::land_batch_body
#13 git_policy_cli::landing::service_land::land_batch_timed
#14 git_policy_cli::service::service_server_land_group::land_group
```

So the service was in `marker_of` (through `describe_lock`,
before it places a lock of its own),
not in the wait's `holder_after_pauses`.

### The rules text marked before and after a worktree-moving command (git-policy-cli)

`package/git-policy/cli/src/native/invocation/entry_rules_text_change_warm.rs` read each rules file whole to
build the digest it compares before and after `merge`,
`pull`,
`checkout`,
`switch`,
`rebase` and `reset`:

```rust
match std::fs::read(entry.path.as_path()) {
    Ok(bytes) => { 1_u8.hash(&mut hasher); bytes.hash(&mut hasher); }
    Err(_absent) => 0_u8.hash(&mut hasher),
}
```

The path is `worktree_root.join(...)` of the `rulesFile` the configuration names,
or the default
`forbidden-strings.local.txt` when no source names one,
so a person or any process that may write the worktree
controls the name.

### The rules files the scanner loads (forbidden-strings)

`package/forbidden-strings/cli/src/rules_snapshot.rs` read every entry the same way,
in `RulesSnapshot::read`
and again in `holds_what_was_read` (the check that an artifact is published only while the files still hold the
text that was compiled):

```rust
match std::fs::read(&entry.path) {
```

This is the site a commit or `check --all` reaches,
and the one whose silent failure would let a commit pass a
check it could not run.

## The fix

Every site now reads through an open that cannot wait,
and takes the file's type from the open handle,
which is
what `package/git-policy/cli/src/native/foundation/plain_file.rs` already owned.

- `holder_after_pauses` and `marker_of` open the lock with `crate::plain_file::open_plain_file`,
  which opens with
  `O_NONBLOCK | O_NOFOLLOW | O_CLOEXEC` and refuses anything that is not a plain file.
  A lock that is a pipe,
  a
  link or a directory is `Holder::Foreign` at once,
  and names no marker.
- `digest_of_rules_text` reads each rules file with
  `crate::plain_file::read_plain_file_through_links(path, SMALL_FILE_LIMIT)`.
  That variant was added to the same module beside `read_plain_file`,
  because a rules file is named in a
  configuration by a person who may keep it behind a link of their own,
  and following the link is the path that
  configuration means.
  `SMALL_FILE_LIMIT` (1 MiB) is past the largest rules text here:
  the scanner's builtin baseline is 80,820
  bytes and this repository's two appendixes are 10,238 and 5,417.
- `read_rules_file` in `package/forbidden-strings/cli/src/rules_snapshot.rs` opens with
  `O_NONBLOCK | O_CLOEXEC`,
  follows a link,
  and refuses a descriptor that is not a regular file with
  `it is not a plain file (a named pipe, a device or a directory)`.
  It needs `libc` for the flag,
  which that crate's lockfile already carried through the git index reader.

What each site does with a file it cannot read,
and why:

- The index lock:
  a shape the service can never have written is a foreign program's.
  The command runs on into
  Git's own refusal,
  which names the lock,
  and the service's landing stops waiting with the message it already
  had for a lock nothing proves alive.
  Nothing is dropped:
  the lock is still there and still blocks the index.
- The warm mark:
  a rules file that cannot be read is not marked,
  and the command says so on standard error,
  naming the path and saying the next commit reads it.
  Only a cache warm is lost.
- The scanner's own read:
  the read fails naming the path,
  so the policy decides nothing and the command exits 2
  with `policy-incomplete`.
  A commit is refused before Git runs,
  its staged change untouched.
  This is the one
  place where refusing loudly is the requirement:
  a commit must not pass a check it could not run.

## Verification

Wrapper under test:
`monochromatic-git-policy-cli 0.6.1-alpha.5`,
debug build of branch
`fix/git-policy-cli-fifo-where-a-file-is-expected-ends-no-command`,
native Git `2.56.0` as `/usr/bin/git`.
Reproductions and helpers:
`${HOME}/temp/agent/pi-fifo-fixes/repro/`,
copied from the audit's
`${HOME}/temp/agent/sonnet-audits/never-stuck/reproductions/`,
which ran against `a56d51b311`.
Every run below was made on 2026-11-09 in a fixture of its own (`fx_new`),
under its own cgroup,
with a
`timeout`.

Before the fix,
on this branch (so the audit's findings still reproduced):

- `repro-01-fifo-at-rules-file-hangs-worktree-moving-commands.sh`:
  the FIFO case `rc=124` at 30 s.
- `repro-02-fifo-at-configured-rules-file-hangs-add-commit-check.sh`,
  variant `configured`:
  `add`,
  `commit`
  and `check --all` each `rc=124` at 30 s.
  Variant `default` ended at once (`add` 1.1 s,
  `commit` 1.8 s,
  `check` 0.0 s),
  as the audit said.
- `repro-05-fifo-at-index-lock-holds-add-and-commit.sh`:
  `add` and `commit` still running at 130 s;
  the service
  with a thread in `wait_for_partner`.

After the fix,
same scripts:

- `repro-01`:
  wrapped `checkout` `rc=0` in 0.0 s,
  the branch moved to `other`,
  and the wrapper printed
  `monochromatic-git-policy-cli: the forbidden-strings rules file <path> is not text this command can read (it is
  not a plain file (a named pipe, a device or a directory)), so the rules text is not marked here; the next commit
  reads it and reports what it finds.`
- `repro-02`,
  variant `configured`:
  `add` `rc=2` in 0.2 s,
  `commit` `rc=2` in 0.0 s,
  `check --all` `rc=2` in
  0.0 s,
  each with
  `{"type":"engine-failure","code":"policy-incomplete","message":"monochromatic-git-policy-cli could not load the
  forbidden-strings rules, so no file was scanned: read rules <path>: it is not a plain file (a named pipe, a
  device or a directory)."}`.
  Variant `default`:
  `add` 0.2 s,
  `commit` 0.1 s,
  `check` 0.0 s.
- `repro-05`:
  wrapped `add` `rc=128` in 0.3 s with native Git's own
  `fatal: Unable to create '<git dir>/index.lock': File exists.`,
  and no service thread in `wait_for_partner`.
- `fifo-index-lock-timing.sh`:
  with the FIFO still in place,
  a wrapped `commit` ended in 79.5 s with `rc=1`,
  naming the lock,
  saying the commit was not made,
  and keeping it at `refs/cli-git/kept/<batch>`;
  the next
  wrapped command ended in 1.7 s,
  so nothing queued behind it.
  With the FIFO removed,
  the next commit landed
  (`rc=0`,
  16.1 s).

The bound is in the harness as well as the code:
each control runs the read inside `finished_within` (five seconds) so a regression fails instead of hanging the
suite.
With the old reads put back,
all four controls failed at their own bound:

- `git-policy-cli`:
  `4 tests run: 0 passed, 4 failed` in 6.26 s,
  among them
  `reading a marker out of a named pipe did not finish within five seconds` and
  `marking a rules text whose file is a named pipe did not finish within five seconds`.
- `forbidden-strings`:
  `3 tests run: 0 passed, 3 failed` in 5.02 s,
  among them
  `reading a rules list that names a named pipe did not finish within five seconds` and
  `comparing against a named pipe did not finish within five seconds`.

## What does not work

- Checking the name with `metadata` before a plain read.
  The standard library has no non-blocking open,
  so the
  check is a separate look at a name another process may change,
  and the read that follows still blocks.
  `plain_file`'s own doc says the type is taken from the open handle for that reason.
- Refusing links everywhere,
  by adding `O_NOFOLLOW` to the rules file's open.
  A dotfile manager keeps a
  configuration's file behind a link;
  that would turn a working setup into a failed commit,
  against the owner's
  "Commits should never fail".
- A timer or a thread around the read.
  It leaves the descriptor and the thread waiting,
  and the command still
  has to decide what the unread file means.

## The class, and what was checked beside these four sites

Reads of a path a person or another process controls,
on the commit,
merge and worktree paths
(`src/native/{landing,invocation,command,merge,worktree,lock,policy,repo_read,push,repair,service,binary,process}`,
tests excluded):
154 plain `File::open`,
`std::fs::read` and `std::fs::read_to_string` calls.
They fall into these groups:

- `/proc` and `/sys` reads (process tables,
  `statm`,
  cgroup limits):
  a procfs entry cannot be a named pipe a
  person plants.
  Left as they are.
- Opens of a directory for `fsync` (`service_land_copy.rs`,
  `worktree_subvolume.rs`,
  `service_durable.rs`):
  `open` of a directory returns at once.
  Left as they are.
- The wrapper's own state and journal records under `XDG_RUNTIME_DIR` and the journal:
  a FIFO planted there
  would still hold the wrapper.
  Not changed here;
  it is the next group to convert,
  and the audit's full and
  read-only journal runs ended within seconds because those fail on the first write.
- Reads of Git's own files (`.git/config`,
  `commondir`,
  `HEAD`,
  `FETCH_HEAD`,
  `sequencer/opts`,
  `rebase-merge/*`,
  the index itself,
  pack files):
  a FIFO at one of these holds native Git too,
  since it reads
  the same files.
  Not changed here,
  so the wrapper is not slower to refuse than the Git it forwards to.
- The real Git executable and its version cache (`invocation/real_git_candidate.rs`,
  `invocation/git_version_cache.rs`):
  a FIFO replacing the Git binary would hold the wrapper.
  Not changed here.
- Worktree registration reads (`worktree_remove_registry.rs`,
  `worktree_removal_rescue*.rs`,
  `worktree_snapshot_source.rs`,
  `worktree_ladder_target.rs`):
  some already use `read_plain_file`,
  the rest are
  the next group for the class work.

## Upstream filing decision

Nothing to file upstream.
Every site is code this repository owns,
so the six-constraint check has no upstream to address,
and there is
no `.out-of-scope/` entry to consult.
The reproductions are in `${HOME}/temp/agent/pi-fifo-fixes/repro/`,
outside the repository,
and are not a
filing artifact.
