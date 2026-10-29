# cli-git policy platform implementation specification

## Status and authority

This file is the canonical implementation interface for
`doc/decision/cli-git-policies-platform.md`
and `doc/decision/cli-git-concurrent-commits.md`.
The concurrent-commit code map and slice order live in
`package/git-policy/cli/doc/concurrent-commits-implementation-plan.md`.
Runtime code,
public declarations,
tests,
and user documentation must conform to it.
Changing a settled product behavior requires a new decision update rather than an implementation shortcut.

The implementation is the native Rust executable `cli-git-native` (`src/native/`),
 published as the crate
`monochromatic-git-policy-cli`.
The TypeScript implementation (authoring API,
 plugins,
 trust registry and the bundled MJS artifact) is retired.
Its last commit is `f4fc1fa213ece65672e4314983a75266837892f9`,
 and the sections that specified it were removed from
this file;
 they remain readable at that commit.

## Module seams

The executable has these external seams:

- the shadowing `git` executable;
- the repository-root configuration file `cli-git.config.jsonc`;
- JSONL policy events.

It has internal seams for filesystem identity,
real-Git execution,
clock,
process liveness and birth identity,
lock-holder evidence,
and prompts.
Production adapters own operating-system effects.
Disposable tests replace those adapters without production environment overrides.

## Single executable and shipped policies

The release artifact is one executable,
 `cli-git-native`.
Every policy is compiled into it;
 no configuration loads code,
 and there are no plugins.
The built-in policies run at their default severity without being listed.
The optional policies (`markdown/autofix`,
 `mono/forbidden-root-context`,
 `mono/dependent-version-bump` and
`security/forbidden-strings`) run only where `cli-git.config.jsonc` lists them.
A no-config invocation therefore runs the built-ins only.

The forbidden-strings scanner is linked into the executable and needs no separate installation.
`markdown/autofix` runs the `monochromatic-lint` executable,
 which must be on an absolute `PATH` entry.

## Policy re-run after replay

After a clean replay,
 revalidation converges all policies again over the replayed tree and the shadow `HEAD` at the
replay parent,
 and re-runs `pre-commit` whenever the tree differs from the one it last approved.
No recorded policy run is reused:
 the executable keeps no per-policy read sets or input fingerprints.
The retired TypeScript wrapper reused a run when its declared inputs and recorded reads were unchanged;
that design (the `inputs` declaration and the read-set validation) is specified in this file at the last incumbent
commit `f4fc1fa213ece65672e4314983a75266837892f9`.
Re-run findings and patches follow the ordinary whole-sequence convergence rules
(see "Whole-sequence fixing").

## Finding and patch validation

`code` is a stable policy-local kebab-case identifier.
The public JSONL code becomes `<policy-id>/<code>`.
`message` is non-empty and contains no terminal-control requirement.
The renderer JSON-escapes it.

A location is a half-open byte range.
It requires `path` and must satisfy
`0 <= byteStart <= byteEnd <= candidateByteLength`.
Character,
line,
and column coordinates are intentionally absent because policies inspect exact bytes.

A fix is a correction and not a diff:
the repository path,
the file mode (which the fix keeps),
the bytes the policy read,
and the complete bytes the file should hold.
The retired TypeScript wrapper carried a textual unified Git diff and applied it with `git apply --cached --3way`;
the native engine already holds the corrected bytes, so it writes them directly.
A commit transaction stores them with `git hash-object -w --stdin` and points the private index entry at the blob with
`git update-index --add --cacheinfo`;
a direct fix writes the worktree file.
The crate has no patch parser or applier for policy corrections.
A correction that the private index cannot take is a failure of the pass and names its path.
The engine does not compare a correction's bytes with the candidate's:
a pass counts as changed only when its resulting state differs from the preceding state
(see "Whole-sequence fixing").

A correction for a tracked file that the fix did not select (an added path) exists only for a direct fix.
It is valid only when `HEAD` holds the file as an ordinary file with the blob the correction was computed against,
the real index holds that same blob with `HEAD`'s mode,
and the worktree copy is a regular file whose bytes and executable bit match that blob.
Otherwise the engine exits `2` with `patch-conflict`,
naming the path,
which state differs,
and the remedies:
include the path in the fix pathspecs so the fix applies to its worktree copy,
or restore it to match `HEAD` (`git restore --staged --worktree -- <path>`).
A commit transaction journals only the paths it selected:
its records carry an empty list of added paths.

## Configuration validation

The wrapper reads `cli-git.config.jsonc` at the repository top level.
The file is JSON with comments,
 at most 1 MiB,
 and data only:
 it never names code to run.
Without the file the defaults apply.
Legacy executable configuration files (`cli-git.config.mjs`,
 `cli-git.config.ts`) are never read or executed:
beside a JSONC file `git cli-git check` reports a `legacy-config-ignored` configuration warning (other commands
 and `fix` stay silent),
and alone they are a configuration failure that asks for the translation.

Config parsing validates the following.
It visits the top-level keys in document order and reports the first failure it meets,
so the numbered items are the checks,
 not a fixed sequence:

1.  Parse the JSONC text;
     a syntax error names its byte offset.
2.  Validate that the top level is an object with no duplicate keys.
3.  Validate each top-level key:
     only `policies`,
     `hooks`,
     `indexLock` and `landing` exist.
    `plugins` and `trust` are refused with an explanation of the retirement.
4.  Validate `policies`:
     each key is a known policy ID,
     and each value is a severity (`off`,
     `warn` or `error`),
    for a policy that has options a `[severity, options]` array,
    or an options object alone,
     which names the policy at its default severity.
5.  Validate each policy's options against its fixed schema;
     a policy without options takes none.
    `markdown/autofix` accepts `rules` (only `lfs-image-url`;
     a repeated rule is refused) and `exclude`;
    `security/forbidden-strings` accepts `builtinRules` and `rulesFile`.
6.  Validate the concurrency keys (see "Concurrency configuration").
7.  Resolve the rules file of `security/forbidden-strings`:
     `rulesFile`,
     else the environment variable
    `FORBIDDEN_STRINGS_RULES`,
     else `forbidden-strings.local.txt` at the repository top level,
     which may be absent.
    A file named by `rulesFile` or by the variable must exist.
8.  Emit configuration warnings for explicit unsafe `warn` settings.

Unknown top-level keys,
unknown or mistyped concurrency keys,
unknown policy IDs,
duplicate keys,
invalid severities,
unknown or mistyped options,
and an unreadable or oversized file exit `2` before any policy runs.

### Concurrency configuration

Three nested keys tune concurrent commits.
None of them disables the concurrent transaction;
no configuration key or environment variable opts out of it.

- `hooks.concurrentCommits`:
  boolean,
  default `false`.
  `false` serializes preparation hooks and the post-landing `post-commit` hook through the hook lock;
  `true` lets them overlap.
- `indexLock.unprovenOwnerTimeoutMs`:
  non-negative safe integer,
  default `1000`.
  The backoff budget for a foreign `index.lock` whose owner is dead or unproven.
- `landing.reserveAfterLostRaces`:
  positive safe integer,
  default `1`
  (see "Benchmark method").
  Lost landing races after which a transaction asks for the landing reservation.

Unknown nested keys,
wrong types,
fractional values,
and out-of-range values are config failures.
An absent config uses the defaults.
Startup recovery runs before config loading and always uses the defaults.

## Command facts and classification

`rawArgs` are the exact wrapper arguments after the script name.
`transformedArgs` contain fixed cli-git transforms but no management-only or escape-hatch tokens.
Both arrays are immutable copies.

`subcommand` is the argument-aware parsed Git subcommand after global options and `-C` chains.
`effectiveCwd` is canonicalized after applying Git's ordered `-C` semantics.
`repositoryRoot` is the canonical real-Git toplevel for that effective directory.

Known read-only forms skip config loading.
The classifier source must enumerate its Git documentation version and fixtures.
Mixed commands classify from arguments.
Any unknown alias,
external command,
future command,
or ambiguous form loads config.

Short-circuit global version and help forms preserve real Git behavior and do not load config.
Management dispatch occurs only when the parsed subcommand is exactly `cli-git`.
A pathspec named `cli-git` does not dispatch management.

## Management grammar

The accepted grammar is:

```text
git [<git-global-options>] cli-git (--help | -h)
git [<git-global-options>] cli-git check [--policy <id>]... --all
git [<git-global-options>] cli-git check [--policy <id>]... -- <pathspec>...
git [<git-global-options>] cli-git fix [--policy <id>]... --all
git [<git-global-options>] cli-git fix [--policy <id>]... -- <pathspec>...
git [<git-global-options>] cli-git doctor
```

The retired forms `trust [--yes]`,
 `untrust` and `status` still parse.
They print an explanation to stderr,
 change nothing and exit `0`;
`trust --help` sends the explanation to stdout,
while `untrust --help` and `status --help` are usage errors that exit `2`.
They read no configuration and need no Git.

Rules:

- Management parsing uses the internal argv region parser.
- Namespace help writes human-readable text to stdout and exits `0`.
- Help returns before resolving real Git,
  recovering transactions,
  or discovering repository config.
- `--help` and `-h` are valid at the namespace.
- `--policy` is repeatable only for `check` and `fix`.
- Repeated identical policy IDs are deduplicated while preserving first occurrence order.
- `--all` and `-- <pathspec>...` are mutually exclusive and one is required.
- The pathspec form requires at least one token after `--`.
- Tokens after `--` are passed to real Git pathspec expansion without wrapper reinterpretation.
- Unknown management flags and extra positionals are usage errors with exit `2`.
- A usage error never loads repository config.
- `doctor` takes no options, scope or policy; any extra token is a usage error with exit `2`.
- `check` and `fix` load config after grammar and scope validation.
- `fix` changes selected worktree files and verifies every real index blob is unchanged.

Direct `check` and `fix` emit policy events on stdout.
A configuration failure of `check` or `fix` emits one schema-version-one `engine-failure` event with code
`config-invalid` on stdout and exits `2`.

### Doctor

`git cli-git doctor` lists the traditional hooks a repository holds.
Traditional hooks are unsupported (see "Hook dispatcher shim"),
and this command is where they are found:
the commit path only warns when `core.hooksPath` is configured and never scans a directory.

- It asks real Git, with the caller's global options, for the common Git directory
  (`git rev-parse --path-format=absolute --git-common-dir`) and for `core.hooksPath`
  (`git config --type=path --get core.hooksPath`).
- It scans `<git-common-dir>/hooks` and, when `core.hooksPath` is set, the directory it names.
  A relative value resolves against the worktree root, or against the Git directory of a repository without a worktree.
  A directory that two spellings name is scanned once.
- A hook is any entry of a scanned directory that is an executable regular file or a symbolic link
  (a dangling link included), except names ending in `.sample`.
  Plain files without an execute bit, directories and samples are not hooks.
- It prints one hook path per line on stdout,
  the default directory first and each directory in name order,
  and nothing else on stdout.
  When it finds any it also explains on stderr that traditional hooks are unsupported.
- Exit status: `0` when no hook is found, `1` when any is found,
  and `2` when Git cannot locate a repository or a scanned directory cannot be read.
  A directory that does not exist holds no hooks.
- It does not load configuration, recover transactions or take a lock, and it changes nothing.

## Lifecycle

### Pre-forward

Built-in policies run against raw semantic command facts.
Fixed transforms then produce `transformedArgs`.
Optional policies run against the predicted candidate set for the transformed command.
`canApplyPatches` authorizes only engine-owned policy patch application,
not arbitrary repository mutation.
It is true only when a private commit transaction or direct-fix operation owns safe patch application.
Applicable normalizer patches then run through whole-sequence convergence.
Other pre-forward commands emit findings without proposing unavailable patches.
Remaining errors block before real Git.

### Linked-worktree ignored-state synchronization

`resolveGitWorktreeIdentity` is the package-wide seam for replaying effective Git selection and classifying canonical
outside,
bare,
main,
and linked identity.
Policy adapters add their own allowlisting after this shared classification.

An applicable source is a forwarded invocation whose shared identity selects a linked worktree or bare repository
and whose subcommand,
after ordinary Git alias resolution,
creates or moves worktrees.
Only applicable sources hold the common-directory settlement lock across real Git and its hooks.
Every other forwarded command,
including every commit preparation,
runs without that lock,
so concurrent commits in linked worktrees never contend on it.
A worktree created by Git invoked through an absolute path from a hook bypasses the wrapper
and therefore gets no ignored-state copy,
matching the documented bypass of everything else.
A hook that runs `git worktree add` through the wrapper is itself an applicable source.

Before forwarding an applicable source,
cli-git captures the common Git directory and linked-worktree administrative identity set.
An invocation targeting the main worktree bypasses administrative observation,
journal recovery,
settlement locking,
and synchronization,
even when real Git creates a linked worktree,
with one exception:
a command that creates or moves a worktree copies the configured `rulesFile` of the `security/forbidden-strings`
policy from the main checkout into the new worktree, under the settlement lock and the journal.
After real Git and its hooks return for an applicable source,
cli-git compares the administrative set again.
Every new identity is a synchronization destination,
regardless of whether argv named `worktree add`,
an ordinary Git alias expanded to it,
or real Git returned nonzero after retaining registration.
A missing or stale pre-existing worktree root does not hide a newly created identity.

For an applicable non-bare invocation,
the source is the invoking linked worktree root after Git and `post-checkout` settle.
Git's standard exclusion stack selects the source paths:
per-directory `.gitignore`,
repository exclude state,
and configured excludes files.
The source set includes ignored regular files,
directories,
and symbolic links.
Ignored sockets,
FIFOs,
and device nodes are selected only to produce a copy failure without opening them.
A bare repository has no source root and synchronizes an empty set.
The only repository configuration that touches this behavior is that `rulesFile` path.
The wrapper-only flag `--no-worktree-copy` skips synchronization for one invocation.
It is recognized in flag position after the subcommand and before Git's `--` pathspec separator,
and, like the other wrapper controls, among Git's global options before the subcommand,
is stripped before real Git runs,
and leaves value-position tokens with the same bytes untouched.
An opted-out invocation performs no journal recovery;
pending journals wait for the next repository invocation without the flag.

Before staging,
exclude every registered worktree root strictly nested under the source.
Private stage names are reserved and excluded at every path component.
Stage each destination separately in a mode-`0700` sibling directory so file cloning stays
 on the destination filesystem.
Copy regular files into the stage with an exclusive copy-on-write request whose unsupported-filesystem behavior falls
back to a full copy.
Preserve directory and regular-file permission bits plus exact symbolic-link target text.
Never follow a symbolic link while classifying source or destination entries.

Build a deterministic parent-first manifest before copying.
After the private stage is complete and modes are applied,
rebuild the source and staged manifests and require exact final equality of selected roots,
entry paths,
types,
modes,
regular-file bytes,
and symbolic-link targets.
Transient source mutations that revert before final comparison are outside the guarantee.
Any remaining difference is source instability and blocks destination mutation.

Preflight every existing destination manifest entry before creating any path.
Accept an exact match even when the destination does not ignore that path.
Never overwrite or remove a differing destination entry.
Create absent entries exclusively and retain the Git-created worktree on any failure.
Install a regular file as a hard link to its staged copy,
so it appears atomically with its final bytes and mode;
when the filesystem refuses the link,
fall back to the exclusive copy-on-write request.
The staged copy is independent of the source,
so a destination file never shares storage with its source file.
Immediate rollback visits only transaction-owned paths in child-first order;
it removes a selected file or symbolic link only when it still exactly matches the private stage,
and removes a proven directory,
selected or scaffold,
only when it is empty.
Every path whose ownership cannot be proved remains in place and is named in the diagnostic.
A failed installation ends its transaction after rollback:
it removes its journal and private stage,
so no later invocation replays the same failure.

Each destination uses a schema-versioned journal under
`<git-common-dir>/cli-git-worktree-copy/v1`.
Journal writes use a private no-follow temporary file,
file synchronization,
atomic rename,
and parent-directory synchronization where supported.
The journal record is rewritten only when the transaction changes phase.
Selected-entry intents and proven post-creation identities are appended to a private install log inside the stage,
one synchronized line per bounded batch of manifest entries:
the batch's absent paths are claimed before any of them is created,
and their identities are recorded after.
Installation cost therefore grows linearly with the entry count.
A trailing log fragment without a line terminator is an unfinished append;
recovery ignores it and truncates it before appending.
A completion phase makes stage removal and journal removal recoverable in either crash order.
Staging takes minutes and gigabytes, and the journal is written only after it,
so the stage's name is recorded first:
a claim file `<id>.stage` beside the journals names the stage container before the container is created,
with the same temporary file, synchronization and rename as a journal write.
The claim is removed once a journal names the stage, or once a failed copy has removed the stage.
A process-birth-identity lock serializes installation and recovery.
An applicable source waits for an owner proven alive for at most `CLI_GIT_LOCK_WAIT_SECONDS` (default 1800),
after writing one stderr line naming the owner's PID and the lock,
and then fails naming the owner, leaving the lock in place;
while that owner lives the waiter only reads the owner record,
and it attempts publication again once the owner releases,
replaces,
or no longer runs.
A dead owner's lock,
including PID reuse or an exited but unreaped process,
is retired after re-reading that it still names that owner.
An owner record that is unreadable or invalid proves nothing,
so the waiter uses Git's jittered quadratic backoff for up to 1000 ms without a proven owner,
then fails with a diagnostic listing that evidence and leaves the lock in place.
The lock is released and retired by renaming it to a unique name before deleting it,
so a concurrent publication is never emptied in place.

Every later linked-worktree or bare-repository invocation checks for pending journals and stage claims before forwarding
without taking the settlement lock,
and takes it only to recover when one exists.
An invocation that neither creates nor moves worktrees does not wait for the lock:
while another live process owns it,
that owner is still writing its own journal or already recovering,
so the invocation forwards without recovery.
Recovery validates that journal paths remain canonical,
the stage is a private owned directory beside the destination,
and every intended entry exists in the private manifest.
Every pending transaction then reaches an end:

- A destination that no longer resolves to a linked registration under the same common directory,
  because it was removed,
  moved,
  pruned,
  or replaced,
  is discarded:
  recovery removes the private stage and journal,
  never touches the destination path,
  and writes one notice line.
- A transaction whose private stage is gone is discarded the same way,
  with a notice that ignored files in the destination may be incomplete.
- Otherwise installation resumes.
  An existing directory at a selected path the transaction claimed or created is accepted whatever its mode,
  because an interrupted installation applies modes after every entry exists;
  every other existing entry still requires an exact match.
  A resumed installation that fails ends its transaction as a first attempt would,
  and the recovering invocation writes the failure as a notice and continues.
- Completed cleanup is resumed without reinstalling entries.

After the journals, recovery removes every stage container that a claim names and no pending journal names:
the lock proves that no live copy is between claiming and journaling,
so a copy that died while staging (a signal, the out-of-memory killer, a power loss) left it.
Recovery requires the claim to be a private file naming an absolute path whose last component starts with the reserved
stage prefix, and the container to be a private directory owned by the user,
removes it with its read-only subdirectories,
writes one notice line naming it,
and removes the claim.
A claim whose container was never created is removed without a notice.

Malformed or unsafe journal,
install-log,
stage claim,
or stage state fails closed without deleting the journal or a path outside the validated private stage.

After all newly registered destinations settle successfully,
write exactly one human summary line to stderr.
Copy-only failure exits `2`.
If real Git returned nonzero and synchronization also fails,
preserve Git's numeric status;
use `1` when Git ended by signal without a numeric status.
If Git failed but synchronization succeeds,
preserve its status after writing the successful copy summary.

### Post-commit

After a successful non-dry-run commit lands
and the `post-commit` hook has run
(see "Post-landing"),
use the OID the landing wrote by compare-and-swap.
Never re-read live `HEAD`:
after concurrent landings it can name another invocation's commit.
Run applicable post-commit policies against committed ground truth.
`landedCommitOid()` returns the exact resolved commit;
`candidates()` enumerates the commit's changed entries (its delta against each parent, every entry of a root commit)
and reads blobs by object ID rather than worktree path.
Only a clean or warning-only result allows auto-push.
A policy or engine failure leaves the commit intact,
blocks push,
returns `2`,
and emits `commit-landed` after the causal events.
Once the landed OID is known,
repository-root or candidate-fact setup failure emits `content-unavailable` plus the same explicit landed state.

### Auto-push

Auto-push is single-flight per branch.
A per-branch owner lock at `<git-common-dir>/cli-git/push/<encoded-ref>.lock`
and a `last-pushed` record at `<git-common-dir>/cli-git/push/<encoded-ref>.last-pushed.json`
coordinate pushes from concurrent landings.
The record names the most recent attempt:
the branch tip it resolved,
its outcome,
the owner-lock token and PID of the attempt,
its exit code,
and,
for a failed attempt,
its complete push output.
A record that an earlier release wrote with a `publishedTip` field still reads, and the field is dropped on the next write.
It is published by rename,
and an absent or unreadable record reads as no attempt,
which only costs an extra push.
The encoded ref is the branch ref name's UTF-8 bytes
with every byte outside lowercase ASCII letters,
digits,
`-`,
`_`,
and `.`
written as `%XX` in uppercase hexadecimal,
so it is reversible,
flat,
and distinct on case-insensitive filesystems.

- A landed commit whose OID is an ancestor of the recorded successful tip has already been delivered
  and finishes without taking the lock.
- Otherwise it waits for the lock,
  which a pusher holds for its whole push,
  so it waits for any in-flight push.
  Holding the lock,
  it re-reads the record:
  a successful attempt whose tip contains its OID is a joined success;
  a failed attempt that finished after the commit's first read and whose tip contains its OID is a joined failure.
  A failed attempt recorded before the commit's first read is retried rather than reported.
- Otherwise it resolves the current branch tip,
  pushes with the existing argument selection
  (plain `git push` with an upstream,
  `git push --set-upstream origin HEAD` without one),
  and records the attempt,
  successful or failed.
- A push of a branch with an upstream that the remote refuses as stale
  (its output holds `[rejected]`, the marker Git does not translate)
  is reconciled by the same attempt, under the same lock,
  whenever the push destination can be named.
  The destination remote follows Git's order
  (`branch.<name>.pushRemote`, then `remote.pushDefault`, then `branch.<name>.remote`),
  and `push.default` picks the ref:
  `current` and a triangular `simple` (push remote other than the tracked remote) use `refs/heads/<name>`,
  `upstream`, `tracking` and a non-triangular `simple` use the tracked ref.
  `@{push}` is not used, because Git refuses to resolve it for a push remote other than the tracked one
  although the push itself works.
  The remote `.`, a `remote.<name>.push` refspec, `matching`, `nothing` and a ref outside `refs/heads`
  name no destination, and the rejection is returned unchanged.
  An amended or rebased branch is reconciled like any other:
  the merge brings the published commits it replaced into the remote history beside the new ones,
  and only a real overlap between the two is a conflict.
  Histories that share no commit are merged too (`--allow-unrelated-histories`).
  The attempt fetches the remote branch into the private ref `refs/cli-git/auto-push/fetch-<pid>`
  (never `FETCH_HEAD`, which every fetch shares),
  deletes that ref,
  and branches on the two ancestor questions.
  A remote tip that the branch contains means the rejection was not a divergence and is returned unchanged.
  A remote tip that contains the branch means the commit is delivered, and nothing is pushed.
  Otherwise `git merge-tree --write-tree --no-messages --name-only` computes the merge from objects alone
  and `git commit-tree` records it with the branch tip first and the remote tip second,
  signed (`-S`) when `commit.gpgsign` is true because `commit-tree` never reads that setting;
  a signer that fails publishes nothing.
  The merge is pushed with a plain push to `<remote> <merge>:<merge ref>`, never forced.
  A conflict (merge-tree exit `1`) publishes nothing and names the conflicting paths.
  A push that is refused as stale again repeats the fetch, the merge and the push,
  at most `MAX_RECONCILE_ATTEMPTS` (10) times in all,
  because sibling worktrees that keep pushing are contention and not a conflict.
  The merge is computed without reading or writing the local branch, the index, the worktree or the untracked files,
  so another consumer's uncommitted files cannot block it or be changed by it.
  After the merge is published, the branch and its worktree are fast-forwarded to it
  (`git merge --ff-only <merge>`) only when all of these hold:
  `HEAD` is the pushed branch at the tip that was pushed,
  and no path of `git --no-optional-locks status --porcelain=v1 -z --untracked-files=no`
  (both names of a staged rename or copy included)
  is a path of `git diff --name-only -z --no-renames <pushed tip> <merge>`.
  An empty status passes trivially; uncommitted work in files the move does not write is left exactly as it was,
  and the report says so.
  Git itself refuses, without changing anything, a move that would overwrite an untracked file.
  Any other state leaves the branch behind the remote, and the report names the reason, with the overlapping paths
  (five by name, then a count).
  The merge commit skips commit-time hooks and policies because it is made from objects.
  Every line the reconcile adds begins with `cli-git: auto-push: `;
  a line that begins with `cli-git: auto-push: not published: `
  explains a failure and replaces the generic "run `git push`" note,
  which would repeat a push that cannot succeed.
  When the branch could not be fast-forwarded it stays behind the remote,
  and the note names the reason and `git merge --ff-only <remote>/<branch>` for when the worktree allows it.
- An invocation finishes auto-push once a successful push covers its OID,
  or once the push that would cover it has failed.
  Covering is `git merge-base --is-ancestor <landed OID> <tip>`
  against the tip the deciding push resolved before it pushed.
  When the pushed tip no longer contains the landed OID,
  because the branch was reset or rewritten,
  the invocation prints a note naming both.
- A dead pusher's lock is retired through the owner-liveness check,
  and a waiting joiner takes over.
  A dead pusher records nothing,
  so the joiner pushes the current tip itself.
  The dead pusher's orphaned `git push` child may still be running.
  The killed-pusher fixture observed the takeover push succeed while that child waited in its `pre-push` hook,
  and the child's later update of an older tip left the remote at the takeover tip.
  A takeover push that races the child on the remote ref can fail,
  and it is then surfaced like any other failed push.
- No step of auto-push waits without a bound.
  The push runs under a deadline,
  `CLI_GIT_PUSH_DEADLINE_SECONDS` whole seconds,
  600 by default;
  a zero,
  negative,
  fractional or non-numeric value is ignored.
  Past the deadline the push is asked to end,
  killed after a five second grace,
  and recorded as a failed attempt with exit code 124 whose output ends with a line naming the deadline.
  A push whose output grows past 4 MiB is stopped the same way, and only the first 4 MiB are kept.
  The commit stays saved and the exit contract below holds.
  Git's low-speed abort is added to the push's environment
  (`GIT_HTTP_LOW_SPEED_LIMIT=1000`, `GIT_HTTP_LOW_SPEED_TIME=60`)
  for each variable the invocation's environment does not set,
  which also overrides `http.lowSpeedLimit` and `http.lowSpeedTime` from Git configuration.
  Each read-only Git query before the push
  (`rev-parse`,
  `remote get-url`,
  `symbolic-ref`,
  `merge-base`)
  stops at the push deadline or 120 seconds,
  whichever is shorter,
  and a stopped query is a coordination failure that names it.
- A waiter gives up on the lock after the push deadline plus the grace plus 60 seconds,
  or after `CLI_GIT_PUSH_LOCK_WAIT_SECONDS` when that is set.
  It never takes a live holder's lock.
  The commit is reported with the push-failed note and a message naming the lock and the holder's PID.
  A waiter prints one line naming the pushing process,
  the branch and the limit as soon as it starts waiting.
- A joined commit's own `pre-push` hook does not run separately;
  the joined push ran it once for its tip.
- The existing exit contract holds:
  a failed push,
  whether owned or joined,
  is surfaced with its complete output by every affected invocation,
  leaves the commit local,
  and preserves exit `0`.
  A failure of the coordination itself,
  such as an unwritable `cli-git/push` directory,
  is surfaced the same way.
- Skips for a detached `HEAD` or a missing remote are unchanged
  and create no coordination files.
- The landing lock is never held while waiting for or running a push.

### Manual push

Resolve actual local and remote updates with Git-native information.
Scan only content the push newly publishes to its destination remote.
A pure deletion has no content target.
An indeterminate content-bearing range is `content-unavailable` and exits `2`.
Manual push never applies policy patches.

A pushed commit's already-published set is every commit reachable from a commit Git knows the destination remote has:
the commits at that remote's remote-tracking refs (`refs/remotes/<remote name>/`,
 matched as a literal prefix),
plus the peeled authoritative prior value of every update in the same push to that remote,
deletions included,
when that value exists locally.
A prior value Git never fetched is unknown and bounds nothing.
When the published set is non-empty,
scan each commit reachable from the pushed commit but not from the published set.
A tag or branch whose target is already published scans nothing.
When the published set is empty
(first push to an empty remote,
a remote with no remote-tracking refs,
or a push to a URL rather than a named remote),
scan the pushed commit's final tree once as complete added content;
never walk every commit in history.
Intermediate content added and removed before that first push is not scanned.

Remote-tracking refs are a cached view,
not destination authority.
A tracking ref left stale by a remote history rewrite can exclude commits the remote no longer has;
Git updates tracking refs on fetch and on push to that named remote.
Pushing history unrelated to every published commit still scans that whole unrelated range.

Process creation is bounded independently of history length and update count:
one `git cat-file --batch-check` peels every pushed and prior object,
one `git for-each-ref` lists remote-tracking refs,
at most four per-update `git rev-list --stdin` or `git ls-tree` processes run concurrently,
one `git diff-tree --stdin` reads every range commit's delta,
and one `git cat-file --batch` reads every blob.
Never spawn one process per commit.
Not yet met by the native executable: it handles updates one after another,
 runs `git rev-list <local> --not <published>`
with arguments, and reads each range commit's delta with its own `git diff-tree` process; the handover lists this as an
open divergence.

### Direct check

Use worktree bytes selected by explicit pathspecs or `--all`.
Run applicable direct-check policies read-only.
Emit final findings to stdout.

### Direct fix

Use worktree bytes selected by explicit pathspecs or `--all`.
Apply eligible patches to private candidate state through whole-sequence convergence,
then atomically replace only changed selected and added worktree files.
Snapshot the complete real index before and after and fail if any index blob changes.
Hold the landing lock
(see "Locks")
from the first snapshot through worktree installation and the final snapshot,
so a concurrent landing cannot falsify the byte-identical real index check.
Emit final findings and one fix summary to stdout.
A patch targeting an unselected tracked file adds it under the "Added paths" precondition;
its original worktree bytes for the concurrent-change check are the verified `HEAD` blob.

## Policy order and stopping

Each pass runs:

1. built-in policies in fixed order;
2. fixed transforms in fixed order;
3. optional policies in registry order.

The built-in order is
`require-root`,
`linked-worktree-only`,
`branch-worktree-only`,
`add-explicit`,
then `final-newline`.
The transform order is atomic push,
commit only,
then status hints off.

Without `--cli-git-keep-going`,
the first unpatched error stops the pass.
With the wrapper flag,
later policies run after findings,
but any final error still blocks real Git.
Engine failures always stop immediately.
The wrapper recognizes its flag only before Git's `--` separator and removes it before forwarding.

## Whole-sequence fixing

A pass begins with one exact candidate state.
Later policies see earlier patches.
Any changed candidate makes every finding in that pass provisional and restarts the complete order.

The engine retains the initial candidate state and up to eight changed states in private temporary storage.
States serialize as ordered entries of repository path,
mode,
revision identity,
and exact bytes.
No content digest participates.

After each pass:

- exact equality with the preceding state means stable;
- exact equality with any non-adjacent retained state means cycle failure;
- after eight changed passes the engine runs one more pass:
  it settles when that pass proposes nothing and is a pass-limit failure when it still proposes a change;
- otherwise start the next pass at the first built-in policy.

Stable warning-only or clean state exits `0` when Git does not run.
Stable state with errors exits `1`.
Cycle and pass-limit failures exit `2`.
Only events derived from the stable pass are emitted as findings.

## JSONL schema version 1

Each event is one compact JSON object followed by LF.
Fields not listed for an event are absent rather than carrying a JSON `null` value.
Unknown fields may be added only in a backward-compatible schema revision.
New event types,
new optional fields,
new `coreId` values,
and new finding or failure codes are backward-compatible additions under `schemaVersion: 1`.
Consumers must ignore event types,
fields,
and codes they do not recognize.
Removing,
renaming,
or changing field meaning requires a new integer `schemaVersion`.

All policy events share:

```ts
export type EventBase = {
  readonly schemaVersion: 1;
  readonly sequence: number;
  readonly type: string;
};
```

`sequence` starts at `0` for each batch of events and increments by one in emission order.
A commit invocation can emit up to three batches (the traditional-hooks warning, the pre-commit run and the
post-commit run), each starting at `0`.

### Finding event

```ts
export type FindingEvent = EventBase & {
  readonly type: 'finding';
  readonly trigger: PolicyTrigger;
  readonly policyId: string;
  readonly severity: ActivePolicySeverity;
  readonly code: string;
  readonly message: string;
  readonly path?: RepositoryPath;
  readonly pathBytes?: string;
  readonly location?: FindingLocation;
  readonly fix: 'none' | 'available';
};
```

`path` is text.
A Git pathname that is not valid UTF-8 appears in `path` with each invalid sequence replaced by U+FFFD,
and `pathBytes` then carries the exact pathname bytes as standard padded base64 (RFC 4648 section 4).
`pathBytes` is absent whenever `path` is the exact name,
including a name that holds U+FFFD itself and a name that holds a backslash.
A reader can therefore rely on this rule:
when an event has no `pathBytes`, its `path` is the exact name,
with one exception.
A finding of `security/forbidden-strings` reports the scanner's masked display of the name in `path`
(a byte that is not UTF-8 shown as `\xNN`, and a matched segment masked),
and never has `pathBytes`,
because the exact bytes would undo the masking.
Every other event that names a file reports the exact name this way,
on `git add`, on a commit (landed, blocked, normalized, replayed or recovered), and on direct `check` and `fix`.
A `message` may also name a file; there the name is readable text with each invalid sequence replaced by U+FFFD,
and only the `path` fields above carry the exact bytes.
The retired TypeScript wrapper stopped on such a name and never emitted the field.

`code` is the complete `<policy-id>/<policy-local-code>` value.
Patch bytes are never emitted.
`available` means the final stable finding still offers a patch that was not applied in this read-only or unsupported
mode.
A policy may instead omit an inapplicable patch and report `none`.
A corrected finding from a provisional pass is not emitted.
Consequently successful autofix emits only `fix-summary`,
not the corrected finding.

### Configuration warning event

```ts
export type ConfigurationWarningEvent = EventBase & {
  readonly type: 'configuration-warning';
  readonly code: 'warn-unsafe' | 'legacy-config-ignored' | 'traditional-hooks-unsupported';
  readonly message: string;
  // `warn-unsafe` carries the trigger and policy it came from:
  readonly trigger?: PolicyTrigger;
  readonly policyId?: string;
  // `legacy-config-ignored` carries the legacy file instead and has no trigger or policy:
  // `traditional-hooks-unsupported` carries the configured `core.hooksPath` value in `path`:
  readonly path?: string;
};
```

An explicitly configured warn-unsafe policy emits this non-blocking event even when its check returns no finding.
Configuration warnings share the same JSONL stream and invocation-local sequence as findings and engine failures;
they never write ad hoc prose to the machine stream.

A `git commit` in a repository whose configuration sets `core.hooksPath` emits one `traditional-hooks-unsupported`
configuration warning on standard error, before anything else the command prints.
It names the configured value in `path`,
says that traditional hooks are unsupported and that their behavior is undefined,
and says to run `git cli-git doctor`.
The warning never blocks the commit and is not a core finding.
The trigger is the configuration snapshot the commit reads once for its other needs:
the commit starts no process and makes no file system call for this warning,
so a repository without the setting pays nothing,
and the hooks directory is not scanned on the commit path.
The wrapper's own per-transaction `core.hooksPath` overrides (`<tx>/hooks` and `<tx>/no-hooks`),
which reach commands that config hooks start, never trigger it.

### Core finding event

```ts
export type CoreFindingEvent = EventBase & {
  readonly type: 'core-finding';
  readonly trigger: 'pre-forward';
  readonly coreId:
    | 'commit-only'
    | 'commit-normalization'
    | 'concurrent-commit';
  readonly code:
    | 'commit-only/all-flag'
    | 'commit-only/pathspec-required'
    | 'commit-only/staged-changes-ignored'
    | 'commit-normalization/no-change'
    | 'concurrent-commit/replay-conflict'
    | 'concurrent-commit/head-moved'
    | 'concurrent-commit/branch-switched';
  readonly message: string;
  readonly paths?: readonly RepositoryPath[];
  readonly pathsBytes?: readonly string[];
  readonly winningOid?: GitObjectId;
  readonly preparedOid?: GitObjectId;
};
```

Core findings are expected rejections from fixed non-configurable behavior.
They block with exit `1`,
share policy-event sequencing,
and cannot be disabled or assigned a severity through repository config.

- `commit-normalization/no-change`:
  the settled tree equals the preparation base after normalization,
  so no commit is created unless one was explicitly requested.
- `concurrent-commit/replay-conflict`:
  replaying the prepared commit onto the moved target conflicts.
  `paths` lists the conflicting paths in Git path byte order,
  `winningOid` names the earliest commit in `<preparation base>..<current target>` that touches any of them,
  and `preparedOid` names the prepared commit so the user can cherry-pick it.
  The message names all three.
- `concurrent-commit/head-moved`:
  an amend,
  a merge,
  cherry-pick,
  or revert conclusion,
  or a normalization found the target moved since preparation,
  or any commit found that the target no longer names a commit
  (a deleted branch),
  so there is nothing to replay onto.
- `concurrent-commit/branch-switched`:
  the symbolic `HEAD` target differs from the one recorded at invocation.

`paths`,
`winningOid`,
and `preparedOid` are present only for `concurrent-commit/replay-conflict`.
When any conflicting pathname is not valid UTF-8,
`pathsBytes` lists the base64 bytes of every path in `paths`,
entry for entry in the same order;
otherwise it is absent.
Every `concurrent-commit` finding leaves ref,
real index,
and worktree bytes unchanged by cli-git.

### Fix summary event

```ts
export type FixSummaryEvent = EventBase & {
  readonly type: 'fix-summary';
  readonly trigger: 'pre-forward' | 'direct-fix';
  readonly passes: number;
  readonly changedPaths: readonly RepositoryPath[];
  readonly changedPathBytes?: readonly string[];
};
```

`changedPaths` are unique and sorted by Git path byte order.
When any changed pathname is not valid UTF-8,
`changedPathBytes` lists the base64 bytes of every changed path,
entry for entry in the same order as `changedPaths`;
otherwise it is absent.
The event is absent when no bytes changed.

### Engine failure event

```ts
export type EngineFailureCode =
  | 'config-invalid'
  | 'core-incomplete'
  | 'policy-incomplete'
  | 'content-unavailable'
  | 'patch-invalid'
  | 'patch-conflict'
  | 'fix-cycle'
  | 'fix-pass-limit'
  | 'transaction-failed'
  | 'index-lock-unproven-owner';

export type EngineFailureEvent = EventBase & {
  readonly type: 'engine-failure';
  readonly code: EngineFailureCode;
  readonly message: string;
  readonly trigger?: PolicyTrigger;
  readonly policyId?: string;
  readonly path?: RepositoryPath;
  readonly pathBytes?: string;
};
```

`path` and `pathBytes` follow the finding event's rules.
One causal engine-failure event is emitted for an engine exit.
`core-incomplete` means a fixed transform failed unexpectedly rather than producing an expected core finding.
`index-lock-unproven-owner` means a foreign `index.lock` with a dead or unproven owner outlasted
`indexLock.unprovenOwnerTimeoutMs`;
its message lists the collected holder evidence and states that cli-git left the lock in place
(see "Foreign `index.lock` classification").
Nested exception stacks and arbitrary thrown values remain debug logs rather than schema fields.

### Commit replayed event

```ts
export type CommitReplayedEvent = EventBase & {
  readonly type: 'commit-replayed';
  readonly preparedOid: GitObjectId;
  readonly fromBase?: GitObjectId;
  readonly onto: GitObjectId;
  readonly oid: GitObjectId;
};
```

Emitted once per successful replay,
after revalidation,
when the replayed commit is written.
`fromBase` is the preparation base and is absent when the branch was unborn at preparation.
`onto` is the target value the replay used as parent,
and `oid` is the replayed commit the next landing attempt lands,
including any policy patch or hook change revalidation made.
A later lost race can replay the same transaction again with a new event.
The events of each replay follow the settled preparation pass in this order:
`landing-race-lost`,
`landing-reserved` when that race earned the reservation,
`commit-replayed`,
`replay-headers-dropped` when it applies,
and then the revalidation pass's own events,
including its `fix-summary` when a revalidation patch changed bytes.

### Replay headers dropped event

```ts
export type ReplayHeadersDroppedEvent = EventBase & {
  readonly type: 'replay-headers-dropped';
  readonly preparedOid: GitObjectId;
  readonly oid: GitObjectId;
  readonly headers: readonly string[];
  readonly message: string;
};
```

A non-blocking warning emitted immediately after the `commit-replayed` event
of a replay that rebuilt a signed commit carrying custom headers
(see "Replay" in "Transaction protocol").
`headers` lists the dropped header names in their order in the prepared commit,
each name once,
and `oid` is the re-signed commit that lacks them.
The event changes no exit code.

### Landing race lost event

```ts
export type LandingRaceLostEvent = EventBase & {
  readonly type: 'landing-race-lost';
  readonly attempt: number;
  readonly winningOid: GitObjectId;
};
```

Emitted each time a landing finds the target ref moved,
either before its compare-and-swap or through a failed compare-and-swap.
`attempt` counts this transaction's lost races from `1`,
and `winningOid` is the target value that won.

### Landing reserved event

```ts
export type LandingReservedEvent = EventBase & {
  readonly type: 'landing-reserved';
  readonly lostRaces: number;
};
```

Emitted once when a transaction is granted the landing reservation
(see "Starvation reservation" in "Transaction protocol"),
immediately after the `landing-race-lost` event of the race that earned it.
`lostRaces` is this transaction's lost races at the grant.
The event changes no exit code.

### Commit landed event

```ts
export type CommitLandedEvent = EventBase & {
  readonly type: 'commit-landed';
  readonly oid: GitObjectId;
  readonly outcome: 'post-commit-blocked';
  readonly message: string;
};
```

This event is emitted when a commit exists but a post-commit policy or engine failure makes cli-git return `2` after
real Git succeeded.
It follows any causal finding or engine-failure event and is the final policy event.
An ordinary auto-push network or remote rejection is not an engine failure:
cli-git surfaces complete push output,
leaves the commit local,
and preserves the successful commit command's exit code `0`.

### Stream and exit contract

Wrapper policy events use stderr.
Direct `check` and `fix` policy events use stdout.
Real Git inherits normal stdio.
Debug logs must not corrupt the selected JSONL event stream.

When real Git does not run:

- `0` means clean or warnings only;
- `1` means final error findings;
- `2` means usage,
  config,
  patch,
  transaction,
  or engine failure.

When real Git runs,
preserve its exit code except for a landed commit followed by a post-commit policy or engine failure,
which returns `2`.
An ordinary failed auto-push preserves the successful commit result and exits `0` after surfacing the push failure.

For a commit transaction,
the native `git commit` that runs during private preparation is the real Git run:

- a failed preparation
  (a hook,
  the editor,
  signing,
  or Git itself)
  preserves its exit code and lands nothing;
- a `pre-commit` hook that fails when re-run after a replay exits `1`,
  like native Git's hook rejection,
  and lands nothing;
- a `concurrent-commit` core finding exits `1`;
- a landing,
  replay,
  or lock failure that is not a core finding exits `2` with an engine-failure event;
- a landed commit exits `0` unless post-commit policies block,
  which returns `2` as above.

## Transaction protocol

### Scope and phases

Several `git commit` invocations against the same worktree and branch run at the same time,
and each lands as its own sequential commit.
The behavior is on in every repository the wrapper runs in;
no configuration key or environment variable opts out of it.

The production transaction supports explicit-path and `--no-only` commits,
pathspec files including stdin and NUL forms,
selected deletions and untracked files,
`commit -a`,
clean commits that no policy patches,
amend,
allow-empty,
and merge,
cherry-pick,
or revert conclusions.
Every non-dry-run commit runs through it.
The wrapper forwards native `git commit` only for dry runs and short-circuit forms,
because native `git commit` keeps `index.lock` on disk through hooks and the editor.

Each transaction passes through four phases:

1.  Capture:
    at invocation,
    record the transaction facts and snapshot the commit's content
    under the per-worktree capture lock
    (see "Capture order").
2.  Preparation:
    in parallel with other transactions and without the real index lock,
    evaluate policies and run native `git commit` against private state.
3.  Landing:
    serially under the landing lock and the real `index.lock`,
    advance the target ref by compare-and-swap and install the real index.
4.  Post-landing:
    outside both locks,
    complete worktree copies,
    run automatic maintenance,
    run `post-commit` once,
    run post-commit policies,
    and auto-push.

### Invocation capture

At invocation,
before any Git mutation,
the transaction records:

- the preparation base:
  the target's commit OID,
  or unborn;
- the symbolic `HEAD` target:
  a ref name,
  or detached;
- the target ref for compare-and-swap:
  the branch,
  or `HEAD` itself when detached;
- the mode:
  explicit-path or index;
- the conclusion kind:
  none,
  amend,
  merge,
  cherry-pick,
  or revert;
- the repository root,
  common Git directory,
  and real index path,
  honoring a caller-set `GIT_INDEX_FILE`,
  `GIT_DIR`,
  `GIT_WORK_TREE`,
  and the effective directory of `-C`;
  global `--git-dir` and `--work-tree` do not reach these queries;
- the selected paths;
- the reflog nonce;
- the invocation start time.

Explicit-path commits capture the selected worktree bytes at invocation;
index commits capture the real index at invocation.
Every later read of `HEAD` inside the transaction uses the recorded preparation base or the landed OID,
never live `HEAD`.
Cli-git never re-prepares a commit from current worktree bytes.

The facts are read in this order,
so capture order can keep every landed-capture record a transaction may replay over
(see "Capture order"):

1.  the layout and every other fact except the preparation base;
2.  the transaction directory is published;
3.  the next capture sequence number is read;
4.  the preparation base is read;
5.  `preparing.json` is written and the shadow repository created;
6.  the content is captured under the per-worktree capture lock with the next capture sequence number,
    and `captured.json` is written.

Every private index copy and every index install carries the source index's access and modification times
(`copy_index_with_timestamps` in `commit_index.rs`):
Git re-hashes a cached entry only when its mtime is not older than the index file's mtime,
so a fresh timestamp would hide same-size edits made in the second Git cached their stat
(`doc/troubleshooting/git-racy-index-copy.md`,
 #544).

### Transaction directory and journal

Transactions live under `<git-dir>/cli-git-transactions/`,
where `<git-dir>` is the worktree's own Git directory.
Each transaction owns `<root>/<transaction-id>/`,
named by a random UUID.
The directory is built under a reserved staging name and published by rename
only after `owner.json` is complete,
so every published transaction names its owner.
Enumeration lists staging directories as staging entries;
recovery never recovers one whose owner is dead and leaves it in place for diagnosis.

```text
<git-dir>/cli-git-transactions/
  landing.lock/            landing owner lock
  reservation.lock/        landing reservation owner lock
  <transaction-id>/
    owner.json             PID, process-birth identity, schema version, invocation start time
    preparing.json         invocation capture facts
    prepared.json          shadow repository path, prepared OID, signed flag, intended tree
    reservation-request    JSON state record written when the transaction asks for the reservation
    index-lock-<n>.json    identity of the real index.lock attempt <n> created, written right after creating it
    landing-<n>.json       one per landing attempt inside the critical section
    ref-updated.json       exact landed OID
    index-installed        empty completion marker
    hooks/                 plan.json and one entry per preparation event, each a link to the executable
    captured.index         real index at invocation
    commit.index           private commit index native preparation commits
    pre-landing-<n>.index  exact copy of the real index attempt <n> computed against
    post-<n>.index         post-index attempt <n> installs
    install.index          post-index hard link the install renames over the real index
    replay-<r>/            replay <r>: commit.index of the replayed tree
    replay-message-<r>     exact message bytes a signed replay passes to git commit-tree -F
    captured.json          capture stamp, next sequence before the base, worktree-captured paths

<git-dir>/cli-git-captures/
  capture.lock/            capture lock, held only while a transaction captures
  worktree-id              random identity of this capture store
  sequence                 last allocated capture sequence number
  landed/<oid>.json        capture of a commit landed from this worktree, pruned when unneeded

<git-common-dir>/cli-git/shadow/
  <transaction-id>/        shadow repository (see "Private preparation")
```

The shadow repository path is derived from the transaction ID,
so recovery finds it even before `prepared.json` exists.
A shadow repository is created only after its transaction directory is published,
and is removed before its transaction directory,
so no shadow repository outlives the journal that names it.
It is created right after `preparing.json`,
before the private index exists
(see "Private preparation").

State files are created exclusively and never rewritten,
so a crash leaves the newest complete state readable.
Per-transaction state records use schema version 2;
landed-capture records and `plan.json` use schema version 1.
Directories are mode `0700`,
files are mode `0600`,
and every read uses no-follow checks that reject symbolic links,
non-regular files,
and files over the read size limit.

A path in a record is the exact pathname bytes.
Where it is a JSON string,
a pathname that is not valid UTF-8 is written as its text with each invalid sequence replaced by U+FFFD,
and the record adds the base64 of the exact bytes beside it:
`pathBytes` in each object of `addedPaths` and `selectedWorktreePaths`,
`committedPathBytes` beside `committedPaths`,
and `selectedPathspecBytes` beside `selectedPathspecs`
(the list form names every entry once any entry needs it,
the object form only its own).
A record whose paths are all valid UTF-8 has none of these fields and is byte for byte what the retired wrapper wrote,
and a reader treats a missing field as "the text is the exact name".
A record whose bytes do not read as the text beside them is malformed.
`captured.json` keeps its own form,
with every path decoded as Latin-1.

`landing-<n>.json` records the expected old OID,
the new OID
(prepared or replayed),
the exact pre-landing real index snapshot identity,
the post-index artifact identity,
the real `index.lock` device and inode,
the migrated pack name
(see "Object migration"),
and the added-path and selected-worktree records.
It ends with `preLandingDigest` and `postDigest`,
each `{ "length": <bytes>, "fnv1a64": "<16 lowercase hexadecimal digits>" }`
over the snapshot as the landing wrote it
(FNV-1a with the 64-bit offset basis `cbf29ce484222325` and prime `100000001b3`).
A record without them (written before they existed) is read as having none.

The legacy single-journal `<git-dir>/cli-git-transaction` directory is not recovered:
a command that meets one stops with exit `2` and asks for any Git command through the previous cli-git executable,
which recovers it.

### Private preparation

Preparation never mutates the real index,
the worktree,
the target ref,
any other real ref,
or shared reflogs.
It uses:

- a private commit index at `<tx>/commit.index`;
- a shadow repository at `<git-common-dir>/cli-git/shadow/<transaction-id>`
  (see "Shadow repository layout"),
  whose own `HEAD` is the private `HEAD`
  (see "Private `HEAD` shape")
  and whose own object store receives every object preparation writes.

The shadow repository is created right after `preparing.json`,
before the private index is built.
Every cli-git Git command that writes or reads the private index's objects
(`read-tree`,
`add`,
`add --patch`,
`apply --cached --3way`,
`write-tree`,
`diff --cached`,
and the `cat-file --batch` candidate reads)
runs in the owning worktree with `GIT_OBJECT_DIRECTORY=<shadow>/objects`,
whose `info/alternates` names the real object store,
so every blob and tree preparation writes lands in the shadow store.
A real `git gc --prune=now` during preparation therefore cannot delete a staged blob,
which the `gc-prune-during-commits` scenario of the container suite exercises.
Worktree completions after landing read their pre-correction or pre-hook blobs through the same store,
because those blobs never migrate.
Native Git and replay run with `--git-dir=<shadow>` instead;
cli-git's own plumbing keeps the real repository's refs and config.

The shadow repository is a separate repository rather than a registered worktree,
so `git worktree list` never shows it,
and preparation creates no ref in the real repository.
The shadow object store protects the prepared commit until landing:
a real `git prune` or `git gc` never sees the shadow's objects,
so it cannot delete them,
and the shadow reaches real objects only through `objects/info/alternates`,
so a prune run inside the shadow cannot delete real objects.

After policy evaluation settles,
preparation runs native `git commit` with inherited stdio as:

```text
git --git-dir=<shadow> --work-tree=<worktree root>
    -c core.hooksPath=<tx>/hooks
    -c hook.pre-commit.enabled=false -c hook.prepare-commit-msg.enabled=false
    -c hook.commit-msg.enabled=false -c hook.post-commit.enabled=false
    commit <private commit args>
```

with `GIT_INDEX_FILE=<tx>/commit.index`.
`--work-tree` names the same absolute worktree root as the shadow's `core.worktree`;
the command-line value takes precedence for this process,
and `core.worktree` covers a Git process that a hook starts with the shadow as `GIT_DIR`
but without `GIT_WORK_TREE`.
The child environment drops a caller-set `GIT_DIR`,
`GIT_WORK_TREE`,
`GIT_COMMON_DIR`,
and `GIT_OBJECT_DIRECTORY`,
so every object native Git writes lands in the shadow store.
The private commit args drop the user's `--git-dir` and `--work-tree`,
pathspecs,
and the internal `--only`,
because the private index is the complete intended tree.
Git therefore owns hooks,
the editor,
templates,
message cleanup,
and signing.
Native `commit -a` updates only the private index copy.
After Git succeeds,
the prepared OID is the value of the shadow `HEAD`,
and preparation records `prepared.json`.
A commit hook may change the private index while it runs,
as a lint-staged-style formatter rewrites and re-stages files;
native `git commit` commits whatever the hook staged,
and so does the transaction:
the prepared tree is accepted even when it differs from the policy-settled intended tree
(see "Hook-staged changes").
A preparation failure removes the shadow repository and then the transaction directory,
and leaves real index,
worktree,
and real ref bytes unchanged.
Do not invoke Git with a lock path as `GIT_INDEX_FILE`.

#### Shadow repository layout

The shadow repository holds these private entries:

- The ref store:
  `HEAD`,
  `refs`,
  `packed-refs`,
  `reftable`,
  and `logs`.
  It contains the private `HEAD`,
  a snapshot of every real ref taken at invocation
  (`git for-each-ref` in the owning worktree,
  symbolic refs recreated with `git symbolic-ref`),
  and a private copy of the target branch ref at the preparation base
  that replaces the snapshot's entry for that ref.
  Hooks therefore resolve tags,
  other branches,
  and remote-tracking refs as they stood at invocation.
  With the files backend,
  cli-git writes the snapshot as one `packed-refs` file
  (header `# pack-refs with: sorted `,
  with the trailing space Git's own writer emits and older readers require)
  and writes the private target ref as a loose ref through `git --git-dir=<shadow> update-ref`,
  which takes precedence over the packed entry.
  With the reftable backend,
  cli-git writes the snapshot through one `git --git-dir=<shadow> update-ref --stdin` transaction.
  Measured with the files backend
  (Git 2.55.0,
  7 runs each):
  a `packed-refs` snapshot took a median 3.5 ms for this repository's 105 refs
  and 12.2 ms for 20,001 refs,
  while creating loose refs through `update-ref --stdin` took 34.3 ms and 4166.9 ms.
- `objects`,
  a private object store whose `objects/info/alternates` names the absolute real object directory
  resolved at invocation
  (`git rev-parse --git-path objects` in the owning worktree).
- `config`,
  written by cli-git
  (see the config layout in this subsection).
- `config.worktree`,
  copied from the owning worktree's Git directory when present.
- `COMMIT_EDITMSG`,
  which native `git commit` rewrites during every commit,
  so concurrent preparations never share one message file.
- The copied conclusion state
  (see "Sequencer conclusion state").
- `cli-git`,
  which is never linked,
  because the real one contains the shadow directory itself.

Every other entry of the real common Git directory,
such as `info`,
`hooks`,
`rr-cache`,
`lfs`,
and `modules`,
is a symbolic link to the real entry by default.
A per-worktree entry that Git resolves in the owning worktree's Git directory,
such as a linked worktree's `info/sparse-checkout`,
is copied;
a directory that mixes shared and per-worktree entries becomes a private directory
whose shared entries are linked individually.
The shadow has no `index`;
`GIT_INDEX_FILE` names the private commit index.

The shadow `config` has this layout:

```ini
; <git-common-dir>/cli-git/shadow/<transaction-id>/config
[core]
  repositoryformatversion = <copied value>
[extensions]
  ; every extensions.* key of the real repository, copied with its value;
  ; the section is omitted when the repository has none
[include]
  path = <git-common-dir>/config
[core]
  worktree = <worktree root>
[gc]
  auto = 0
[maintenance]
  auto = false
```

- `core.repositoryformatversion` and every `extensions.*` key are copied explicitly,
  because Git does not apply them from an included file.
- The shadow names no hooks directory:
  traditional hooks are unsupported,
  and every command the wrapper starts against the shadow passes its own `core.hooksPath`.
- `core.worktree` follows the include,
  so the real config cannot redirect the shadow's worktree.
- `gc.auto` and `maintenance.auto` follow the include,
  so native `git commit`'s automatic maintenance
  (`run_auto_maintenance` in Git's `builtin/commit.c`)
  never runs in the shadow.

Cli-git writes the file itself,
quoting every path value under git-config value syntax at the final interpolation.

#### Private `HEAD` shape

A prototype in disposable repositories with Git 2.55.0 chose the shadow repository
("Private `HEAD` shape:
 shadow repository with alternates" in `doc/decision/cli-git-concurrent-commits.md`).
It ran a `pre-commit` hook that reads the branch name,
the upstream,
and an `includeIf "onbranch:"` value,
and rejects commits to `main`.

- For a symbolic target,
  the shadow `HEAD` is symbolic to the target branch's own ref name,
  which names the private copy at the preparation base,
  or no ref when the branch was unborn.
  Native `git commit` advances only that private copy.
- For a detached target,
  the shadow `HEAD` is detached at the preparation base.
- `--amend` and merge,
  cherry-pick,
  and revert conclusions read the preparation base through the shadow `HEAD`,
  so they produce native parents and messages.
- No window leaves a prepared commit unprotected:
  it exists only in the shadow store until landing migrates it into a kept pack
  (see "Object migration").

Hooks running during preparation observe:

- the real branch name through `git symbolic-ref HEAD`,
  `git rev-parse --abbrev-ref HEAD`,
  and `git branch --show-current`;
- the branch's upstream through `@{upstream}`,
  because the `branch.<name>.*` config arrives through the include
  and the private upstream copy holds the real value;
- `includeIf "onbranch:<pattern>"` sections matching the real branch,
  so a hook that rejects commits to a protected branch still rejects them;
- the shadow as `git rev-parse --git-dir`
  and the private commit index as `GIT_INDEX_FILE`;
- the shadow ref store's snapshot of every real ref at invocation,
  so tags,
  other branches,
  and remote-tracking refs resolve as they stood at invocation;
- a private `refs/stash`,
  so a hook's `git stash` never touches the real stash list,
  although it still rewrites shared worktree files.

Culled shapes,
with the prototype evidence:

- Detached private `HEAD` with a pending or per-worktree ref:
  hooks see no branch and no upstream,
  so the "reject `main`" hook is bypassed.
- Symbolic `HEAD` naming a pending ref under `refs/cli-git/`:
  the same hook bypass.
  It remains the fallback only if the shadow repository hits a blocker.
- `GIT_REFERENCE_BACKEND` with a private ref store:
  Git passes it into submodules,
  `git -C` from a hook sees the private refs,
  and pseudorefs must be kept in two places.
- A shadow repository using `GIT_OBJECT_DIRECTORY` instead of alternates:
  a prune inside the shadow deleted objects reachable only from real refs.

Residual risk:
a Git-directory-derived path that is neither linked nor copied points into the shadow.
The container end-to-end suite covers LFS,
submodules,
sparse checkout,
reftable,
and SHA-256 repositories
(see "Container end-to-end suite").

#### Hook dispatcher shim

Traditional hooks,
executable scripts in `.git/hooks` or in a `core.hooksPath` directory,
are not supported.
Any use of them with the wrapper is undefined behavior:
nothing here promises their order, environment, locking or exit-status handling,
and no user-facing requirement is written for them.
Only hooks configured through Git config (`hook.<name>.event` with `hook.<name>.command`) are supported.
The Git processes the wrapper starts on the commit path name a hooks directory that holds no hook
(`<tx>/no-hooks`, below), so none of them starts a traditional hook;
controls pin that because the dispatcher must never start its own entries again,
and it is an implementation property, not a promise to users.
`git cli-git doctor` lists the traditional hooks a repository holds.

Hooks run through a dispatcher shim passed with `-c core.hooksPath=<tx>/hooks`
plus `-c hook.<event>.enabled=false` for each event.
`hook.<event>.enabled=false` suppresses config-based hooks but not the hookdir hook,
and `core.hooksPath` alone does not disable config-based hooks
(`doc/troubleshooting/git-hook-disable-switches.md`),
so both are required.

- Each hook entry is the native executable itself.
  `<tx>/hooks/plan.json` holds the dispatch plan,
  encoded at the final interpolation,
  and Git starts the entry under the event's name,
  so the executable dispatches by the name it was started under.
  Nothing new ships beyond the executable.
- `<tx>/hooks/pre-commit`,
  `prepare-commit-msg`,
  and `commit-msg` are those entries (symbolic links on Unix;
  on Windows, which has not been run, hard links or copies named `<event>.exe`).
  `post-commit` is intentionally absent,
  so it never runs during preparation.
- The plan records the events the user disabled through `hook.<event>.enabled`,
  the config parameters native Git would hand its hooks or their absence,
  the real Git path,
  the absolute worktree root,
  the preparation lease,
  the hook lock path,
  and whether the hook lock is skipped.
  It records no hooks path.
  The config parameters are the caller's inherited `GIT_CONFIG_PARAMETERS` followed by the caller's global `-c` options,
  each quoted in Git's `sq_quote` form
  (`'` and `!` close and reopen the quote);
  they are absent when the caller had neither,
  so cli-git's own `core.hooksPath` and `hook.<event>.enabled` overrides never reach a hook.
- The program restores those config parameters,
  exports `GIT_WORK_TREE` as the absolute worktree root
  (Git otherwise rewrites it to `.` for hooks,
  `doc/troubleshooting/git-private-admin-dir-hook-environment.md`),
  skips user-disabled events,
  takes the hook lock
  (see "Hook lock"),
  runs
  `git -c hook.<event>.enabled=true -c core.hooksPath=<tx>/no-hooks hook run --ignore-missing <event> -- <args>`,
  releases the hook lock,
  and propagates the exit status.
- `<tx>/no-hooks` is a path inside the transaction that is never created.
  Git finds no hook in a missing hooks directory,
  so the inner run starts only the hooks the configuration defines by name,
  never a traditional hook and never the transaction's own entries again.
  The `post-commit` run in the real worktree passes the same `-c core.hooksPath=<tx>/no-hooks`.
- A configuration that defines no hook by name (`hook.<name>.event` or `hook.<name>.command`)
  gets no entry and no `post-commit` run:
  the entries and the run would find nothing to start.
  The answer comes from the configuration snapshot, so it costs no file system call.
  A `pre-commit` re-run after a replay follows the same answer.
- The program exports `CLI_GIT_PREPARATION_LEASE`.
  A nested wrapper invocation that inherits a valid lease skips startup transaction recovery and hook-lock acquisition,
  and an index writer whose `GIT_INDEX_FILE` names the transaction's private index
  never takes the landing lock.

Hooks observe the branch state listed in "Private `HEAD` shape".
A `pre-commit` hook that runs `git stash` writes its entry to the shadow's private `refs/stash`
but still transiently touches the shared worktree;
the hook lock serializes only cli-git's own hook runs.

#### Hook lock

The hook lock is an owner lock at `<git-common-dir>/cli-git/hook.lock`.
The dispatcher shim takes it around each preparation hook event,
so an open message editor never holds it.
The re-run of `pre-commit` after a replay goes through the same shim,
so the shim takes it there too,
and cli-git takes it around the post-landing `post-commit`.
It is skipped when `hooks.concurrentCommits` is `true`
and when a valid preparation lease is inherited.

### Policy evaluation during preparation

Preparation constructs candidate facts from the private index,
writes each policy's corrections into the private index,
and restarts the whole ordered policy sequence after exact candidate changes.
Only the final unchanged pass emits findings.
Revalidation after a replay runs every policy again
(see "Policy re-run after replay");
recovery never revalidates,
so the journal records no policy result.
Policy exceptions,
patch conflicts,
and failed Git hooks discard private state without changing real index,
worktree,
or shared ref bytes.

Interactive and patch selection runs through native Git once against the copied private index;
include selection stages into that private index.
Policies receive the exact chosen candidate but cannot apply automatic patches.
Warning findings allow the settled private index to commit;
error findings block with direct-fix guidance.
Unmerged indexes block automatic correction.

#### Index commit

For ordinary index semantics:

1.  Copy the real index captured at invocation.
2.  Write the selected corrections into the copy.
3.  Run native `git commit` privately with the copied index.
4.  Record `prepared.json`.

The real index is computed at landing
(see "Real index at landing").

#### Explicit-path commit

For injected commit-only semantics:

1.  Build a commit index from the preparation base.
2.  Carry into that index every selected entry that the real index stages as an addition
    (`git diff-index --cached --diff-filter=A` against the base, with the same pathspecs;
    a pathspec file is read by the wrapper and passed as arguments, and a file it cannot read is left to Git alone).
    Native Git tracks these paths through the real index and takes their worktree content without asking an ignore rule,
    so the private index must track them too (issue 1807).
3.  Add exact selected worktree paths to that index using Git pathspec semantics.
    A selected path that an ignore rule matches and that no index stages is refused,
    and the message says to stage it with `git add --force` first.
4.  Apply policy patches to the commit index.
5.  Remove pathspecs and internal `--only` before invoking native Git
    because the private index is the complete intended tree.
6.  Record `prepared.json`.

Merge,
cherry-pick,
and revert conclusions use index-commit semantics only.

#### Added paths

A commit transaction adds no paths of its own:
its prepared record carries an empty list of added paths,
and only the selected paths and the hook-staged changes below reach the landing.
The record field remains from the retired design,
in which policy patches could add tracked files to the commit.

Direct fix admits added paths under the precondition in "Finding and patch validation":
`HEAD`,
the real index,
and the worktree copy all hold the blob the correction was computed against.
It has no journal or index step:
converged bytes of added paths join the direct-fix worktree installation.
Precondition failures are `patch-conflict` with direct-fix remedies
(include the path in the pathspecs,
or restore it to `HEAD`).

#### Hook-staged changes

A `pre-commit` hook that edits files and re-stages them,
as lint-staged does,
changes the private index native Git commits.
The transaction keeps that tree,
as native Git keeps it,
and diffs the policy-settled tree against the committed tree
(`git diff-tree -r --raw --no-renames`)
to learn the hook's paths:

- Explicit-path commits reset each hook path outside the selection in the post-index to the landed entry
  only while its real index entry still equals the one captured at invocation;
  an entry restaged since then is kept,
  with a warning naming the path.
  Selected paths are reset as always.
- Index commits need nothing extra:
  the landed index is the hook's private index,
  and the per-path merge of "Real index at landing" already takes landed entries only for unchanged paths.
- A path modified in place as an ordinary file joins the worktree completions of `prepared.json`
  with its pre-hook blob as the original and its committed blob as the intended bytes,
  under the "Added paths" comparison:
  a worktree copy still holding the pre-hook bytes receives the committed bytes,
  one already holding them is left alone,
  and any other bytes are kept with a warning.

A `pre-commit` re-run after a replay is handled the same way
(see "Revalidation after replay").

### Landing

Commits land in preparation completion order.
One landing attempt runs these steps:

1.  Wait while a live reservation owned by another transaction exists
    (see "Starvation reservation").
2.  Acquire the landing lock.
    When a live reservation owned by another transaction now exists,
    release the landing lock and return to step 1.
    Then acquire the real `index.lock` under the foreign-lock rules
    (see "Foreign `index.lock` classification"),
    and write cli-git's own owner PID file in Git's `core.lockfilePid` format.
3.  Fail with `concurrent-commit/branch-switched` when `git symbolic-ref -q HEAD`
    differs from the recorded symbolic `HEAD` target.
4.  Read the target ref.
    When it still equals the expected old value
    (the preparation base,
    or the parent of the latest replay),
    the new OID is the prepared or replayed commit;
    a prepared commit keeps its exact bytes and signature.
    When it moved and the commit is an amend,
    a conclusion,
    or a normalization,
    or when the target no longer names a commit,
    fail with `concurrent-commit/head-moved`.
    When it moved otherwise,
    record a lost race,
    emit `landing-race-lost`,
    release both locks,
    take the landing reservation when this was the `landing.reserveAfterLostRaces`-th lost race
    (see "Starvation reservation"),
    replay and revalidate outside the locks
    (see "Replay" and "Revalidation after replay"),
    and restart at step 1.
5.  Migrate the new commit's shadow-only objects into the real object store as a kept pack
    (see "Object migration").
6.  Copy the current real index,
    compute the post-index against it
    (see "Real index at landing"),
    and write `landing-<n>.json`,
    which records the migrated pack name.
7.  Advance the target by compare-and-swap:
    `git update-ref -m <reflog message> <target> <new> <old>`,
    with the all-zero OID as `<old>` for an unborn target
    and `--no-deref` on `HEAD` for a detached target.
    `<target>` is the branch ref itself,
    `refs/heads/<branch>`,
    never `HEAD`,
    so a `reference-transaction` hook sees exactly one committed update of the branch ref,
    and Git still writes the `HEAD` reflog because `HEAD` is symbolic to it.
    `<old>` is the expected old value of step 4.
    A compare-and-swap failure removes the migrated pack's `.keep`,
    counts as a lost race,
    and continues as in step 4.
8.  Write `ref-updated.json`,
    remove the migrated pack's `.keep`,
    install the post-index through the held lock with an owner-preserving hard link,
    write `index-installed`,
    reproduce native conclusion-state cleanup in the owning worktree's Git directory
    (see "Sequencer conclusion state"),
    and release both locks.

Every real-repository Git command in a landing,
including the compare-and-swap,
runs in the owning worktree's context as captured at invocation:
its working directory and environment,
never `--git-dir`,
and never a `GIT_DIR` naming the common directory or the shadow.
Only then does `git update-ref` on the branch also write the real `HEAD` reflog.

The reflog message is `commit (cli-git <nonce>): <subject>`.
After a landing,
the target reflog,
and the `HEAD` reflog when `HEAD` is symbolic,
each contain the nonce entry exactly once;
a disposable fixture verifies that the chosen `update-ref` form writes both.

The landing critical section never runs hooks,
the editor,
signing,
network operations,
or the hook lock.
Every landing-lock acquisition first recovers dead transactions that hold a landing record
(see "Recovery"),
so a crashed landing is resolved before another landing moves the ref or index.

#### Real index at landing

The real index is always computed against the then-current real index,
never against the invocation-time copy,
so a landing never erases another invocation's staging
and never stages a revert of landed content.

- Explicit-path:
  the current real index with the committed paths,
  including policy-added paths,
  reset to the landed tree.
- Index mode:
  when no replay happened and the current real index is byte-identical to the one captured at invocation,
  the post-index is the private index native preparation committed,
  exactly what native Git would leave.
  Otherwise,
  for each path the landed tree changes relative to the preparation base or to the tree of the captured index,
  take the landed entry only when the current real index entry still equals the captured one;
  otherwise keep the current entry,
  because it was restaged after invocation.
  After a replay the landed entries come from the replayed private index,
  and the wholesale copy is never used,
  because an index read from a tree carries no stat data,
  skip-worktree bits,
  or intent-to-add entries.
- Hook-staged paths follow "Hook-staged changes".

Every copy and install keeps the source index timestamps.

#### Object migration

Before the compare-and-swap,
landing copies every object the new commit reaches that exists only in the shadow store
into the real object store:

```text
git --git-dir=<shadow> pack-objects --revs --local --stdout --quiet
  | git index-pack --stdin --keep=<keep message>
```

- `pack-objects` reads `<new>` and,
  unless the expected old value is unborn,
  `^<old>` on standard input,
  where `<old>` is the expected old value of "Landing" step 4:
  the preparation base,
  or the replay parent.
  `--local` skips every object borrowed through `objects/info/alternates`,
  so the pack holds only shadow-store objects.
- `index-pack` runs in the owning worktree's context,
  so it writes into the real `objects/pack`.
  `--keep` creates the `.keep` file before the pack and its index become visible
  (`final` in Git's `builtin/index-pack.c`),
  so neither `git prune` nor a concurrent repack can drop the objects
  between migration and the compare-and-swap.
  The line it prints
  (`keep`,
  a tab,
  and the pack hash)
  names the pack,
  which `landing-<n>.json` records.
- The keep message is `cli-git <transaction-id>`,
  so recovery finds a `.keep` even when a crash preceded `landing-<n>.json`.

The `.keep` is removed once `ref-updated.json` is written,
after a failed compare-and-swap,
and by recovery.
A pack whose compare-and-swap failed stays as unreachable objects until `gc` expires them;
the shadow store still holds the same objects for the next attempt.

Every landing therefore adds one pack to the real store.
Git searches every pack for an object it does not find in the most recently used one,
so unconsolidated packs make every later Git process in the repository slower;
"Post-landing" step 2 bounds their number the way native `git commit` does.

#### Replay

A lost race replays the prepared commit onto the current target outside both locks,
because signing can prompt.
Replay runs in the shadow
(`git --git-dir=<shadow>`),
so every object it writes stays in the shadow store until the next migration;
the current target's objects resolve through the alternates.
Every replay starts again from the prepared commit and the preparation base,
whichever earlier replay lost its race.

- The tree comes from
  `git merge-tree --write-tree --name-only -z --merge-base=<merge base> <current> <prepared>`,
  run with the worktree root as its working directory so conflicted paths are reported from the repository root.
  The preparation base is the target at invocation,
  or for a branch that was unborn at preparation a root commit of the empty tree
  that cli-git writes into the shadow store,
  because Git 2.40 accepts only a commit as `--merge-base`
  (`object ... is a tree, not a commit`,
   measured 2026-09-26).
  `<merge base>` is the preparation base,
  or,
  when capture order or subsumption settles any path
  (see "Capture order" and "Subsumption"),
  a synthetic root commit in the shadow store
  whose tree is the preparation base's with each subsumed path set to its landed content
  and each capture-ordered path set to the entry of the side whose change it drops.
  The output is the tree ID and a NUL;
  on a conflict,
  each conflicted path NUL-terminated follows,
  then an empty record and the informational messages.
  Exit `1` is a conflict even though a tree ID prints;
  an exit above `1` is a replay failure.
- The replayed tree is revalidated
  (see "Revalidation after replay")
  before any commit object is written,
  so a signed commit is re-signed once per replay.
- An unsigned prepared commit is rebuilt by rewriting its raw object:
  cli-git reads `git cat-file commit <prepared>` as bytes,
  replaces only the `tree` line with the revalidated tree
  and the `parent` line with `<current>`
  (inserting a `parent` line after `tree` when the preparation base was unborn),
  and writes the result with `git hash-object -t commit -w --stdin`.
  Every other header,
  continuation lines included,
  and the message bytes stay exact,
  including `encoding`,
  the author and committer identities and dates,
  and custom headers.
  `git replay` and plain `git commit-tree` both transcoded non-UTF-8 messages and dropped custom headers
   in the prototype.
- A signed prepared commit,
  one carrying a `gpgsig` or `gpgsig-sha256` header,
  is rebuilt with
  `git <caller global options> -c i18n.commitEncoding=<encoding> commit-tree <tree> -p <current> -S[<key id>] -F <tx>/replay-message-<r>`,
  where `<encoding>` is the prepared commit's `encoding` header value or `utf8` when absent,
  `<key id>` is the key the invocation passed to `-S` or `--gpg-sign` when one was given,
  `replay-message-<r>` holds the exact message bytes,
  and `GIT_AUTHOR_NAME`,
  `GIT_AUTHOR_EMAIL`,
  `GIT_AUTHOR_DATE`,
  and the committer triple come from the prepared commit's raw identity lines
  (dates as `@<seconds> <zone>`).
  No Git primitive re-signs a raw object,
  so any header other than `tree`,
  `parent`,
  `author`,
  `committer`,
  `encoding`,
  `gpgsig`,
  and `gpgsig-sha256` is dropped,
  and cli-git emits `replay-headers-dropped`.
  Native `git commit` writes no such header on an ordinary commit,
  so through the wrapper this happens only for objects a hook or another tool built.
- A successful replay emits `commit-replayed`.

A conflict fails without landing as `concurrent-commit/replay-conflict`
with exit `1`
and leaves ref,
real index,
and worktree bytes unchanged.
`winningOid` is the first line of
`git rev-list --reverse <current> ^<base> -- <conflicting paths>`
(without `^<base>` when the base was unborn),
or `<current>` when that lists nothing.
Before the shadow repository is removed,
cli-git migrates the prepared commit into the real object store as a pack without `.keep`
(the "Object migration" command without `--keep`,
with the preparation base as the excluded revision),
so the prepared commit survives until `gc` expires unreachable objects
and the user can cherry-pick it.
Any other replay or revalidation failure,
such as a failed `merge-tree` or signing,
is a `transaction-failed` engine failure with exit `2`.
Path-level replay and automatic re-preparation are not used.

#### Subsumption

Owner decision 2026-09-26
(`doc/decision/cli-git-concurrent-commits.md` "Serial landing"):
for every path both the prepared commit
(against its preparation base)
and the landed history
(preparation base to `<current>`)
changed,
and that capture order leaves undecided
(see "Capture order"),
the prepared bytes already contain the landed change when that change applies in reverse to them.
Such a path keeps the prepared entry as it is,
matching what native sequential commits produce in a shared worktree;
every other path merges three-way from the preparation base.
Under the synthetic merge base a subsumed path shows no landed content change,
so `git merge-tree` takes the prepared bytes,
while a regular file keeps the base's mode there,
so a landed mode change still merges three-way with the prepared one.

Paths are listed by two `git diff-tree -r -z --raw --no-renames` runs from the preparation base,
so a rename counts as a deletion of its old path and an addition of its new one,
each decided on its own;
`git merge-tree` still detects renames when it merges the remaining paths.
Per shared path,
with base,
landed,
and prepared entries:

- The prepared entry equals the landed one in mode and object,
  or both are absent:
  subsumed.
- Otherwise,
  when the landed or prepared entry is absent
  (a deletion on one side,
   or a deletion against a modification):
  not subsumed,
  because a reversed deletion applies only to an absent file.
  An addition on both sides goes on to the text check against an empty base.
- Otherwise,
  when any present entry is not a regular file
  (a symbolic link or a submodule `160000` entry),
  or any of the three blobs has a NUL byte in its first 8000 bytes
  (Git's `buffer_is_binary`,
   so attributes never decide it):
  not subsumed.
- Otherwise the path is subsumed when either text check holds:
  - strict reverse application:
    the landed hunks with three context lines apply in reverse to the prepared bytes,
    an exact emulation of `git apply --reverse --check`
    (no context reduction,
     no whitespace fixing,
     no overlapping hunks,
     anchoring at the file's start and end as Git's `apply.c` does);
  - one-sided extension:
    in zero-context hunks from the preparation base,
    every landed hunk lies inside one prepared hunk,
    and in each prepared hunk the landed hunks it covers,
    joined by the base lines between them,
    are a prefix of the prepared text when both hunks start on the same base line,
    or a suffix when both end on the same base line.
    When both hunks cover exactly the same base range,
    the landed lines may also appear in order with own lines inserted between them,
    as long as the prepared text starts with the first landed line or ends with the last;
    an addition on both sides,
    whose zero-context hunks both cover the empty base,
    is this case,
    so a prepared file holding every line the landed addition added,
    in order,
    is subsumed,
    and one that dropped or rewrote a landed line is not.
    A prepared hunk that replaces exactly the lines a landed deletion removed is a modify/delete conflict
     and is not subsumed.

Strict reverse application alone never subsumes the case the decision exists for:
an own edit directly next to the landed edit sits in the landed hunk's context lines,
so `git apply --reverse --check` and `-C1` reject it,
and `-C0` accepts it only by dropping all context,
which also accepts a landed deletion the prepared bytes never made
(checked 2026-09-26 with Git 2.55.0).
The one-sided extension accepts it and still rejects own edits on both sides of a landed hunk.

Process count is fixed per replay:
two `diff-tree` runs;
the two history processes of "Capture order" when a shared path is worktree-captured;
when text candidates exist,
one `cat-file --batch`,
one `hash-object` writing the empty blob when an addition on both sides needs an empty base,
three `mktree` runs writing single-level trees whose entries are named by index,
so no patch header carries a user path,
and three `diff-tree -p --text` runs;
and when a path is subsumed,
`read-tree`,
`update-index -z --index-info`,
`write-tree`,
and `hash-object` into `<tx>/replay-base-<r>.index` and the shadow store.
File bytes and paths are decoded as Latin-1,
so comparisons are byte-exact and paths encode back unchanged.

#### Revalidation after replay

After a clean merge,
revalidation runs outside both locks
in a private directory `<tx>/replay-<r>/`:

1.  `git read-tree <merged tree>` into `replay-<r>/commit.index`,
    in the shadow store.
2.  Move the shadow `HEAD` target
    (the branch ref,
     or detached `HEAD`)
    to `<current>`,
    so policies and hooks see the parent the commit will have.
3.  Re-run cli-git policies against the paths the replayed tree changes relative to `<current>`:
    every policy runs
    (see "Policy re-run after replay").
    Patches converge under the ordinary pass-limit and cycle rules,
    and policy-added paths follow "Added paths",
    checked against the real index captured at invocation.
    A blocking finding or engine failure lands nothing and returns that result.
4.  When the tree after policies differs from the tree `pre-commit` last approved
    (the prepared tree at the first replay)
    and the invocation did not pass `--no-verify` or `-n`,
    re-run `pre-commit` against `replay-<r>/commit.index`:
    `git <caller global options> --git-dir=<shadow> --work-tree=<worktree root> -c core.hooksPath=<tx>/hooks`
    with every commit event's `hook.<event>.enabled=false`,
    `hook run --ignore-missing pre-commit`,
    `GIT_INDEX_FILE` naming the replayed index,
    and `GIT_EDITOR=:`,
    as native Git runs `pre-commit` when no editor is used.
    The dispatcher shim takes the hook lock and runs the repository's hooks.
    `prepare-commit-msg` and `commit-msg` do not re-run,
    because the message is fixed.
    A failing `pre-commit` lands nothing,
    removes the shadow repository and the transaction,
    and exits `1` without a JSONL event,
    like native Git's hook rejection.
    Changes the hook staged are kept
    (see "Hook-staged changes").
5.  Build the replayed commit from the prepared commit on the resulting tree
    (see "Replay").

Worktree completions of preparation are pointed at the blobs the replayed tree holds;
a path the replayed tree no longer holds as the same ordinary file drops its completion.
Landing then retries from step 1 of "Landing"
with the replayed commit,
`<current>` as the expected old value,
the union of the selected and revalidated paths as the committed paths,
and `replay-<r>/commit.index` as the landed index.
The loop ends when the commit lands,
its replay conflicts,
revalidation rejects it,
or the target cannot take it.
The reservation after `landing.reserveAfterLostRaces` lost races is taken before the replay that follows that lost race
(see "Starvation reservation").

#### Amend, conclusions, and branch switches

- `--amend`,
  merge,
  cherry-pick,
  and revert conclusions fail with `concurrent-commit/head-moved` when the target moved.
- Every commit fails with `concurrent-commit/branch-switched`
  when the symbolic `HEAD` target changed since invocation.
- A detached `HEAD` lands by compare-and-swap on `HEAD` itself.
- Concurrency covers one worktree and branch;
  Git refuses to check out one branch in two worktrees.

#### Sequencer conclusion state

Merge,
cherry-pick,
and revert conclusions run under private preparation.
In the prototype,
copying the per-worktree state into the shadow produced commits byte-identical to native Git
("Conclusions and replay" in `doc/decision/cli-git-concurrent-commits.md`).

At preparation,
cli-git copies each present entry of the owning worktree's conclusion state into the shadow:
`MERGE_HEAD`,
`MERGE_MSG`,
`MERGE_MODE`,
`SQUASH_MSG`,
`AUTO_MERGE`,
`CHERRY_PICK_HEAD`,
`REVERT_HEAD`,
`MERGE_RR`,
and `sequencer/`.
In a reftable repository,
`AUTO_MERGE`,
`CHERRY_PICK_HEAD`,
and `REVERT_HEAD` live in the ref store rather than as files
(only `FETCH_HEAD` and `MERGE_HEAD` stay files,
`is_pseudo_ref` in Git's `refs.c`),
so cli-git reads them with `git rev-parse --verify --quiet`
and writes them into the shadow with `git update-ref`.

Native `git commit` then cleans up inside the shadow.
Its `rerere` step writes the recorded resolution's postimage into `rr-cache` during preparation,
through the linked real `rr-cache`,
and updates the shadow's private `MERGE_RR`.

At landing step 8,
cli-git reproduces native cleanup in the owning worktree's Git directory:

- It removes each of `AUTO_MERGE`,
  `MERGE_HEAD`,
  `MERGE_MODE`,
  `MERGE_MSG`,
  `SQUASH_MSG`,
  `CHERRY_PICK_HEAD`,
  and `REVERT_HEAD` that native Git removed from the shadow.
  `SQUASH_MSG` joins the list the prototype observed because native `git commit` unlinks it too
  (`builtin/commit.c`).
  A reftable repository deletes store-held entries with `git update-ref -d <name> <copied value>`
  instead of removing files.
- An entry is removed only while it still holds the bytes or value copied at preparation;
  a changed entry was written by another command after invocation and is kept.
- It copies the shadow's `MERGE_RR` back.
- It keeps `ORIG_HEAD`.
- It leaves `sequencer/` as native Git leaves it:
  removed only when native Git removed the shadow copy after the last pick
  (`sequencer_post_commit_cleanup` in Git's `sequencer.c`),
  otherwise untouched.

### Capture order

Owner decision 2026-09-26
(`doc/decision/cli-git-concurrent-commits.md` "Implementation-time decisions"):
every capture from a worktree takes a short per-worktree capture lock
and records a monotonically increasing capture sequence number,
giving a total order of the captured disk states of that worktree.
For a path that both the prepared commit and a commit landed since its preparation base captured from the same worktree,
the later capture's bytes land,
which records what native sequential commits would record from the shared disk.
Evidence:
the container `concurrent-trace-replay` scenario still conflicted under subsumption alone
wherever a later capture rewrote lines an earlier in-flight commit had just added.
Accepted cost:
a later capture from a stale editor buffer reverts the earlier edit,
exactly as native Git would.

#### Capture store and lock

The store is `<git-dir>/cli-git-captures/`,
in the worktree's own Git directory
(see "Transaction directory and journal").

- The capture lock `capture.lock` is an owner lock
  (see "Locks"):
  a wait while its owner lives, bounded by `CLI_GIT_LOCK_WAIT_SECONDS` (default 1800),
  and a dead or zombie owner's lock is retired by the next acquirer.
  A transaction holds it only while it allocates its sequence number and captures:
  the private index built from the preparation base and the selected worktree paths,
  or the copy of the real index,
  including `commit -a` staging.
  It is never held across hooks,
  the editor,
  policies,
  or landing,
  and no other lock is taken while it is held.
- `worktree-id` is a random identity written once under the lock.
  A deleted store starts again with a new identity,
  so sequence numbers of two store generations are never compared.
- `sequence` holds the last allocated number as decimal text and a newline.
  Allocation reads it under the lock,
  writes the next number to a private temporary file,
  and renames it over `sequence`,
  so a reader outside the lock sees the old or the new number,
  and a crash leaves at most a gap.
  Numbers start at 1.

#### Worktree-captured paths

`captured.json` in the transaction directory records the stamp
(`worktreeId`,
 `sequence`),
`nextSequenceBeforeBase`,
and the paths whose committed bytes the capture read from the worktree:

- explicit-path commits:
  every path the private index changes relative to the preparation base,
  which are the selected paths;
- index commits that stage worktree content at capture
  (`commit -a`,
   `--include`):
  every path whose private index entry differs from the captured real index;
- other index commits:
  none,
  because their staged bytes come from an unknown earlier time,
  so a later capture of the index is not a later disk state.

Paths are Latin-1 decoded Git path bytes,
as replay's shared-path listing decodes them.
A staged path whose entry already equals the disk is left out,
which only sends it through subsumption.

#### Landed-capture records

Right after the compare-and-swap succeeds
(after `ref-updated.json` and the `ref-updated` phase marker),
a landing writes `landed/<landed oid>.json`:
the landed commit,
the transaction ID,
the stamp,
the worktree-captured paths,
and `nextSequenceAfterLanding`,
the next capture sequence number read after the compare-and-swap.
A failure to write it is reported and never fails the landed commit;
the commit's paths then replay without capture order.
Recovery of a dead transaction whose commit landed writes the same record from its `captured.json`
before any other landing can take the landing lock,
so a commit on the branch lacks its record only when it landed without capture order.
An existing record is kept.

#### Decision per path

Replay decides every shared path
(see "Subsumption" for how shared paths are listed)
before subsumption,
and runs no process when no shared path is among the transaction's worktree-captured paths.
Otherwise it lists the first-parent history since the preparation base once
(`git rev-list --first-parent <current> ^<base>`,
without `^<base>` for an unborn base,
and one `git diff-tree --stdin --root -r -z --raw --no-abbrev --no-renames -m --first-parent`),
and reads the record of each listed commit.
For a shared path:

- when the transaction did not capture the path from the worktree,
  or any landed commit that changed the path has no record,
  a record of another store identity,
  or a record whose capture did not read the path from the worktree,
  capture order does not apply:
  subsumption,
  then the three-way merge,
  decide the path;
- otherwise,
  when the transaction's sequence number is larger than every such record's,
  the prepared entry lands:
  the synthetic merge base takes the landed entry exactly,
  mode included;
- otherwise the landed entry stays:
  the synthetic merge base takes the prepared entry exactly.

Paths changed by a commit from another worktree or clone,
or by a native commit that bypassed cli-git,
therefore keep subsumption and the three-way merge.
A replayed commit whose capture kept a landed entry does not change that path,
so the path's landed content is always the bytes of the latest capture among the commits that changed it.

#### Pruning

A transaction reads `nextSequenceBeforeBase` after publishing its directory and before reading its preparation base,
and a landing reads `nextSequenceAfterLanding` after its compare-and-swap.
A commit that landed after a transaction read its base therefore recorded a number no smaller than the transaction's,
and a transaction not yet published has not read its base,
so every commit already landed is in its base.
Pruning lists the records first and the registry second,
then removes each record that some published transaction may not need:

- every published transaction's `captured.json` of this store identity names a larger `nextSequenceBeforeBase`
  than the record's `nextSequenceAfterLanding`,
  or no published transaction remains;
- the record names another store identity,
  or is malformed.

A published transaction without `captured.json` keeps every record.
Pruning runs after every transaction removes its directory,
whatever its outcome,
and after startup recovery recovered a dead transaction,
so the last transaction to finish leaves no record behind.
It reads only files and never fails the invocation that runs it.

### Post-landing

After both locks are released:

1.  Complete added-path worktree copies
    (see "Added paths"),
    remove the shadow repository,
    and then remove the transaction directory.
2.  Run automatic maintenance in the real worktree with the caller's kept global options,
    as native `git commit` runs it after updating `HEAD` and before `post-commit`
    (`run_auto_maintenance` in Git's `builtin/commit.c`):
    `git maintenance run --auto --quiet` with `--detach` or `--no-detach`.
    The decision mirrors `prepare_auto_maintenance` in Git's `run-command.c`:
    an explicit `maintenance.auto` decides alone;
    otherwise a positive `gc.auto` enables it,
    and neither key set enables it as well;
    `maintenance.autoDetach`,
    falling back to `gc.autoDetach`,
    then to `true`,
    selects detaching.
    Git before 2.47 rejects `--detach` with exit status 129,
    so the step retries without it,
    the form native `git commit` used there
    (unreachable under the 2.55.0 gate).
    `--quiet` is fixed,
    so the background notice never interleaves with JSONL output.
    Its exit status is ignored,
    as in native Git.
    This bounds the real pack count
    (see "Object migration"):
    since Git 2.54 the default `geometric` maintenance strategy merges packs whose sizes break a geometric progression,
    and under the `gc` strategy or an older Git,
    `gc --auto` repacks once the count exceeds `gc.autoPackLimit`.
3.  Run `post-commit` once through `git -c core.hooksPath=<tx>/no-hooks hook run post-commit` in the real worktree,
    when the configuration defines a hook by name,
    under the hook lock unless `hooks.concurrentCommits` is `true`,
    with native-equivalent `GIT_INDEX_FILE`,
    `GIT_AUTHOR_NAME`,
    `GIT_AUTHOR_EMAIL`,
    `GIT_AUTHOR_DATE`,
    and `GIT_EDITOR=:`.
    Its exit status is ignored,
    as in native Git.
4.  Run post-commit policies with the landed OID
    (see "Post-commit").
5.  Auto-push
    (see "Auto-push").

### Starvation reservation

A transaction that has lost `landing.reserveAfterLostRaces` races
writes `reservation-request`
(`{"schemaVersion":2,"state":"reservation-request","lostRaces":<n>}`)
into its transaction directory
and waits for the landing reservation before it replays.
The reservation is an owner lock at `<git-dir>/cli-git-transactions/reservation.lock`
whose owner record also names the transaction ID.

- Reservations are granted oldest invocation first:
  a free reservation goes to the live requesting transaction with the earliest recorded invocation start time
  (`createdAt` in `owner.json`),
  with ties broken by transaction ID byte order.
  A requester takes it only when it is first in that order,
  re-read on every check.
- While a live reservation owned by another transaction exists,
  a transaction may prepare,
  replay,
  and revalidate,
  but waits before landing,
  and a landing that took the landing lock after a reservation was granted releases it and waits again
  ("Landing" step 2).
  So after the grant at most the landing already inside the critical section lands before the holder,
  and a holder loses at most `landing.reserveAfterLostRaces` + 1 races.
- The holder releases the reservation and removes its request when its landing loop ends:
  landed,
  conflicted,
  rejected by revalidation,
  or failed.
- A dead owner's reservation no longer counts:
  the owner-liveness check,
  which counts a Linux zombie as exited,
  treats it as free,
  the next requester retires it,
  and startup recovery retires it as well.
  A dead requester's request goes with its transaction directory.
- An invocation nested under a forwarded Git that holds the landing lock
  (an inherited landing lease,
   see "Index-writer coordination")
  neither waits for nor takes the reservation,
  because its ancestor holds the landing lock the reservation holder waits for.
- A granted reservation emits `landing-reserved` after that race's `landing-race-lost`.

### Locks

Cli-git's own locks are rename-published owner-lock directories carrying process-birth identity,
the mechanism the worktree-copy settlement lock already uses.
A lock whose owner is dead,
including PID reuse,
is retired by the next acquirer.

- Landing lock,
  `<git-dir>/cli-git-transactions/landing.lock`:
  a wait while its owner lives, bounded by `CLI_GIT_LOCK_WAIT_SECONDS` (default 1800).
- Reservation lock,
  `<git-dir>/cli-git-transactions/reservation.lock`:
  a wait while its owner lives, bounded by `CLI_GIT_LOCK_WAIT_SECONDS` (default 1800).
- Hook lock,
  `<git-common-dir>/cli-git/hook.lock`:
  a wait while its owner lives, bounded by `CLI_GIT_LOCK_WAIT_SECONDS` (default 1800).
- Capture lock,
  `<git-dir>/cli-git-captures/capture.lock`:
  a wait while its owner lives, bounded by `CLI_GIT_LOCK_WAIT_SECONDS` (default 1800),
  held only while one transaction captures
  (see "Capture order").
- Push locks,
  `<git-common-dir>/cli-git/push/<encoded-ref>.lock`:
  a wait while their owner lives, bounded by the push deadline plus the grace plus 60 seconds
  or by `CLI_GIT_PUSH_LOCK_WAIT_SECONDS` when that is set.
- Worktree-copy settlement lock:
  a wait while its owner lives, bounded by `CLI_GIT_LOCK_WAIT_SECONDS` (default 1800),
  bounded wait for an owner record without evidence,
  held only by applicable sources
  (see "Linked-worktree ignored-state synchronization").

An owner is alive only while its PID names a running process with the recorded birth identity.
On Linux a process in state `Z`
(exited but not reaped)
or `X` counts as exited:
a zombie keeps its PID and start time,
and inside a container whose PID 1 does not reap orphans,
such as a Node process,
the zombies a `SIGKILL`ed process group leaves are never reaped,
so treating them as alive made every later acquirer wait forever.
The hook dispatcher shim applies the same rule to the hook lock.
A hook process orphaned by a killed wrapper but still running keeps the hook lock until it exits:
it may still rewrite worktree files,
which is what the lock serializes.

Lock order is reservation check,
landing lock,
then real `index.lock`.
No process takes the hook lock or a push lock while holding the landing lock,
and no process takes any other lock while holding the capture lock.

Cli-git never deletes a foreign lock.
Recovery removes a real `index.lock` only when a journal proves a dead transaction owner created it,
by the recorded device and inode.

#### Lock PID injection

Cli-git injects `core.lockfilePid=true` into every forwarded and spawned Git
by appending it through `GIT_CONFIG_COUNT`,
`GIT_CONFIG_KEY_<n>`,
and `GIT_CONFIG_VALUE_<n>` while preserving existing entries,
so native Git leaves an owner PID file beside each lock
and the shim's restoration of `GIT_CONFIG_PARAMETERS` does not remove the setting.
The wrapper adds the entry to the environment of each child it forwards or spawns,
not to its own environment.
Nothing is appended when the last numbered `core.lockfilePid` entry already reads as true,
or when `GIT_CONFIG_COUNT` is malformed,
so Git still reports the caller's error.
Git reads the numbered entries before `GIT_CONFIG_PARAMETERS`,
so a caller's explicit `-c core.lockfilePid=false` still wins.
`core.lockfilePid` first shipped in Git 2.54.0;
an older Git ignores the unknown key and produces no PID evidence.

#### Foreign `index.lock` classification

Before cli-git creates the real `index.lock`,
and before it forwards an index writer,
it classifies an existing lock from evidence re-read on every attempt:

- the lock's device,
  inode,
  and ctime;
- the Git PID file and whether that process is alive and started no later than the lock's ctime,
  allowing for the start time's clock resolution
  (20 ms on Linux,
  from start ticks and `/proc/uptime`;
  1 s on macOS,
  from `ps -o lstart=`);
  a zombie counts as exited,
  and a process started later means the PID was reused;
- open holders matched by device and inode,
  never by path:
  `/proc/<pid>/fd` on Linux,
  `lsof` on macOS,
  and a Restart Manager query on Windows,
  where the `DELETE`-access probe is never used because it can delay Git's own rename.
  `lsof` and Restart Manager are spawned only while a foreign lock is present.
  Unreadable processes are recorded as partial evidence.

The verdict is proven alive,
dead,
or evidence-free:

- A proven-alive owner gets a wait bounded by `CLI_GIT_LOCK_WAIT_SECONDS` (default 1800)
 with one human-readable stderr line naming the holder.
- A dead or evidence-free owner gets Git-style quadratic backoff with jitter up to
  `indexLock.unprovenOwnerTimeoutMs`,
  then `index-lock-unproven-owner` with exit `2`,
  listing the evidence,
  leaving the lock in place,
  and forwarding nothing.

An absent open holder does not prove abandonment:
native `git commit` keeps `index.lock` on disk without an open descriptor through its hooks and editor.

The open-holder scan runs only when the PID file does not already prove a live owner.
The unproven budget counts all time spent in attempts without a proven owner,
evidence gathering included,
because a Linux `/proc` scan over about 1000 processes took 59 to 106 ms on the development host.
While a proven-alive owner holds the lock,
polls are capped at 100 ms when a PID file proves it
and at 500 ms when only an open descriptor does,
because each of those polls repeats the scan.

#### Index-writer coordination

Forwarded index writers coordinate with landings through the landing lock.
Cli-git classifies them from parsed arguments:
`add`,
`rm`,
`mv`,
`restore --staged`,
`reset` except `--soft`,
`stash`,
`checkout`,
`switch`,
`merge`,
`rebase`,
`cherry-pick`,
`revert`,
`apply --cached` and `apply --index`,
`update-index`,
`read-tree`,
`am`,
`pull`,
and `sparse-checkout`.
Classification uses the command after ordinary alias resolution,
and long options match in full or as the abbreviations Git accepts for them.
A writer is against the real index when `git rev-parse --git-path index` names `<git-dir>/index`,
so a writer redirected by `GIT_INDEX_FILE`,
such as a hook of a private preparation,
is not coordinated.
For a classified writer against the real index,
cli-git takes the landing lock,
pre-waits for a foreign `index.lock` under the classification rules,
forwards the command,
and releases the landing lock after real Git returns.
Cli-git does not capture Git's stderr to detect a lock failure and re-forward,
because capturing stderr changes Git's color and progress output.
A residual race remains only with processes that bypass the wrapper.

The forwarded Git receives `CLI_GIT_LANDING_LEASE` naming the held landing lock and its owner token.
Hooks and `rebase --exec` commands of that Git can invoke the wrapper again;
a nested invocation whose inherited lease names the same,
still-held landing lock proceeds without taking the landing lock or pre-waiting for `index.lock`,
exactly as native Git would run,
instead of waiting for its own ancestor.
A nested commit transaction lands under the ancestor's landing lock the same way.

`git cli-git fix` holds the landing lock as described in "Direct fix".

### Recovery

At wrapper startup,
before config loading or forwarding,
recovery enumerates every published transaction under `<git-dir>/cli-git-transactions/`.
A legacy `<git-dir>/cli-git-transaction` directory is not enumerated:
the command stops with exit `2` and instructions.
Management help returns before recovery.

- An owner that is alive with a matching birth identity is skipped at debug log level;
  its transaction is not an error.
  PID reuse counts as a dead owner.
- A dead owner without a landing record and without the record of a real `index.lock` it created:
  remove every `.keep` in the real `objects/pack` whose message is `cli-git <transaction-id>`,
  retire any reservation or reservation request it owns,
  remove the shadow repository,
  and then remove the transaction directory.
  The real index and real refs were never touched.
- A dead owner with a landing record, or with the record of a real `index.lock` it created
  (it was killed holding the landing lock and that lock, before any landing record),
  is recovered only while holding the landing lock,
  so recovery never races a live lander and never releases an `index.lock` that a later lander created.
  Recovery removes the transaction's `.keep` files as for an owner without a landing record,
  then either discards an unlanded attempt,
  installs the recorded post-index for a landed commit whose index install was interrupted,
  or recognizes a completed install.
  For a landed commit it writes the commit's landed-capture record when missing
  (see "Capture order"),
  and it also completes the conclusion-state cleanup from the shadow
  (see "Sequencer conclusion state")
  and added-path worktree copies with the "Added paths" comparison.
  It then removes the shadow repository and the transaction.
- A published directory without a valid owner record,
  or with malformed state,
  fails closed with the path named and preserves its contents.
- A reservation lock whose owner is dead is retired after the transactions are recovered.
- After a dead transaction was recovered,
  landed-capture records are pruned
  (see "Capture order").

Recovery validates the expected old OID,
the landed OID,
the pre-landing and intended index snapshots,
and the recorded artifact and lock identities.
Neither index snapshot is synced when it is written,
so before installing a post-index, and before comparing the real index with the pre-landing snapshot,
recovery recomputes the length and hash of each snapshot and compares them with the landing record's digests.
A snapshot that differs is never installed: recovery fails closed,
names the snapshot, the landing record and the transaction directory,
and leaves the branch, the real index and the owned `index.lock` as they were.
A landing whose install already happened (the `index-installed` marker, or a real index equal to the post-index)
needs neither snapshot and skips the comparison.
Before the exact `ref-updated.json` marker exists,
a landing counts as landed only when the target ref's reflog contains the transaction's nonce entry
with the recorded new OID;
recovery searches the whole reflog,
because other landings can follow a crashed one.
Missing nonce evidence,
or a real index that no longer matches the recorded pre-landing snapshot,
fails closed rather than guessing.
Recovery stabilizes exact artifacts through verified same-filesystem hard links,
installs only through an owner-preserving hard link rather than the mutable lock pathname,
refuses unsafe or replaced filesystem artifacts,
and preserves conflicting evidence after unrelated ref or index movement.

### Compatibility and degradation

The executable requires real Git 2.55.0 or newer for commits,
 hooks and the locks that coordinate with them.
It asks Git for its version once per invocation before that work,
and on an older Git it runs nothing and emits an `engine-failure` event with code `content-unavailable` and exit `2`,
whose message names the real Git it found,
the version Git reported and the remedy (upgrade Git,
 then run the command again).
Read-only commands are never checked.
Features that Git introduced after 2.55.0 are not assumed.

The retired TypeScript wrapper had no version gate and degraded one feature at a time on Git 2.39.5 and 2.40.0:
a probe of `git merge-tree --write-tree --merge-base` (fail fast with `concurrent-commit/head-moved` without it),
the `core.lockfilePid` key of Git 2.54.0,
 and config-based hooks of Git 2.54.0.
That design and the observations of 2026-09-26 are in this section at commit
`f4fc1fa213ece65672e4314983a75266837892f9`.

### Required disposable fixtures

Existing transaction behavior,
each run through private preparation:

- ordinary staged commit;
- explicit-path commit;
- explicit `--no-only`;
- clean commit that no policy patches;
- `commit -a` with `--no-enforce-only`;
- partial staging with unstaged tail;
- unrelated staged paths;
- deletion;
- untracked selected path;
- `--pathspec-from-file` from ordinary files,
  standard input,
  and NUL form;
- amend;
- allow-empty;
- merge conclusion;
- cherry-pick conclusion;
- revert conclusion;
- commit hook failure before ref update;
- real-Git failure;
- patch conflict;
- invalid patch;
- read-only administrative filesystem failure and healthy next invocation;
- hk duplicate-separator regression bytes;
- policy-added path in explicit-path,
  `--no-only`,
  and amend commits,
  with committed,
  indexed,
  and worktree bytes and clean status;
- policy-added path blocked by unstaged worktree changes and by staged changes;
- policy-added path refused under `--include` selection;
- policy-added path in merge,
  cherry-pick,
  and revert conclusions;
- racily clean same-size edit kept visible through direct fix,
  explicit-path and `--no-only` commits that apply a fix,
  a landing index install,
  and a recovery index install;
- policy-added path in direct fix:
  clean unselected path rewritten in the worktree with exact real index bytes,
  dirty unselected path refused,
  and the same path fixed once selected.

Private preparation:

- config-based hooks each run once, and a traditional hook in `.git/hooks` or a `core.hooksPath` directory does not run;
- a hook that changes into a subdirectory sees the correct top level and an absolute `GIT_WORK_TREE`;
- `post-commit` does not run during preparation;
- shared `HEAD`,
  branch,
  every other real ref,
  reflogs,
  and real index bytes stay unchanged during preparation;
- a `pre-commit` hook sees the real branch name,
  the upstream,
  and a matching `includeIf "onbranch:"` value,
  and a hook that rejects commits to `main` rejects one;
- a real `git prune --expire=now` and `gc --prune=now` during preparation keep the prepared commit,
  and `git prune --expire=now` inside the shadow keeps objects reachable only from real refs;
- a real `core.hooksPath` runs no traditional hook and prints the `traditional-hooks-unsupported` warning,
  and `extensions.worktreeConfig` with `config.worktree`,
  and a real `gc.auto` or `maintenance.auto` setting each take the effect "Shadow repository layout" states;
- two concurrent preparations never share a `COMMIT_EDITMSG`;
- preparation in LFS,
  submodule,
  sparse-checkout,
  reftable,
  and SHA-256 repositories,
  and in a linked worktree;
- SSH-signed preparation stays signed;
- `--amend`,
  `--allow-empty`,
  and merge,
  cherry-pick,
  and revert conclusions produce native parents and messages,
  byte-identical to native Git for the conclusions;
- shim plan with user-disabled events,
  caller `GIT_CONFIG_PARAMETERS` present and absent,
  and adversarial paths with quotes,
  newlines,
  and spaces;
- `git worktree list` output unchanged by the shadow repository;
- a preparation failure leaves no shadow repository.

Landing and replay:

- disjoint explicit-path commits land in completion order with native parents;
- non-overlapping hunks in one file replay cleanly;
- subsumption,
  each on real repositories:
  prepared bytes holding the landed edit plus an adjacent own edit land as they are;
  a far-apart edit is not subsumed and merges;
  an adjacent edit without the landed one conflicts;
  identical additions and additions keeping every landed line are subsumed while one
   that rewrote a landed line conflicts;
  delete against modify conflicts either way round and a deletion on both sides is subsumed;
  binary blobs are subsumed only when identical;
  a landed mode change merges three-way even when the content is subsumed;
  an identical gitlink is subsumed and a different one merges without being read as a blob;
  the strict reverse check agrees with `git apply --reverse --check` on seeded cases;
  and an adjacent edit captured on top of another commit's edit replays through the wrapper as captured;
- overlapping hunks fail with `concurrent-commit/replay-conflict` and leave ref,
  real index,
  and worktree exact,
  and the named prepared commit can be cherry-picked after the shadow repository is gone;
- the migrated pack keeps its `.keep` until the compare-and-swap succeeds or fails,
  a real `git prune --expire=now` and `git repack -a -d` between migration and compare-and-swap keep the objects,
  and no `cli-git` `.keep` remains afterwards;
- the real `HEAD` reflog receives the landing entry when `update-ref` runs in the owning worktree's context,
  checked in the main worktree and a linked worktree;
- a signed commit landing without replay keeps its exact bytes;
  a replayed signed commit is re-signed with an SSH key generated in the fixture;
- an unsigned replayed commit keeps its `encoding` header,
  exact identity and date lines,
  and a custom header;
- a signed replayed commit with a custom header drops it and emits `replay-headers-dropped`;
- merge,
  cherry-pick,
  and revert conclusions leave the owning worktree's Git directory as native Git does:
  the removed state files,
  `MERGE_RR`,
  `ORIG_HEAD`,
  and `sequencer/`
  after the last pick and mid-sequence,
  in files and reftable repositories;
- a conclusion-state entry rewritten by another command after invocation survives landing;
- another invocation's staged path survives a landing;
- amend,
  merge,
  cherry-pick,
  and revert with a moved target fail with `concurrent-commit/head-moved`;
- a branch switch between preparation and landing fails with `concurrent-commit/branch-switched`;
- a detached `HEAD` lands by compare-and-swap on `HEAD`;
- two concurrent initial commits on an unborn branch;
- the target reflog and the `HEAD` reflog each contain the nonce entry exactly once;
- a non-UTF-8 `i18n.commitEncoding` commit replays with its encoding header and exact message bytes,
  signed and unsigned;
- `post-commit` runs once with `GIT_INDEX_FILE`,
  `GIT_AUTHOR_*`,
  and `GIT_EDITOR=:`;
- `pre-commit` re-runs against the replayed tree,
  is skipped under `--no-verify`,
  and a failing re-run lands nothing;
- policies re-run against the replayed tree and a revalidation patch lands;
- 2,
  3,
  and 8 commits started together each land exactly once with their captured bytes;
- a `pre-commit` hook that stages another path,
  rewrites and re-stages a selected file,
  or stages a formatted blob without touching the worktree lands what it staged,
  with the real index and worktree reconciled,
  and a worktree edit or real-index restage made after the hook ran is kept;
- a staged blob is absent from the real object store while the commit is held in `pre-commit`,
  and a real `gc --prune=now` then keeps the commit whole;

Capture order:

- a commit captured after another commit's capture of the same line lands its own bytes after a replay,
  and one captured before a commit that landed first keeps the landed bytes of that path and lands its other paths;
- a change landed from another worktree,
  which has no capture in this worktree,
  conflicts when it overlaps and merges three-way when it does not;
- a landing killed after its compare-and-swap gets its landed-capture record from recovery,
  so a commit captured before it keeps the landed bytes;
- a capture waits while another holds the capture lock,
  a holder killed inside the lock is retired by the next capture,
  and captures started together in one process never overlap;
- sequence numbers are consecutive,
  persist in the store across invocations under one identity,
  and a malformed sequence file fails the capture;
- the per-path decision:
  both directions,
  the latest of several landed captures,
  and the fallbacks for a commit without a record,
  another store identity,
  a record that did not capture the path,
  and a path the transaction did not capture from the worktree;
- worktree-captured paths of explicit-path,
  `commit -a`,
  and plain index captures;
- the first-parent history listing from a commit and from an unborn base;
- pruning keeps every record while a published transaction has not captured,
  keeps a record a published transaction may replay over,
  removes a record of another store identity or a malformed one,
  and removes every record once no transaction remains,
  so no landing fixture leaves a record behind.

Reservation,
locks,
and configuration:

- the reservation is requested after the configured lost races,
  granted oldest invocation first,
  blocks other landings while held,
  including one that took the landing lock after the grant,
  and is released when its holder lands,
  when its replay conflicts,
  when the holder is killed,
  and when the holder is a zombie;
- two preparations serialize their hooks by default and overlap with `hooks.concurrentCommits: true`;
- an open message editor does not hold the hook lock;
- concurrency config defaults,
  each valid value,
  zero,
  negative,
  fractional,
  string,
  and unknown nested keys;
- `core.lockfilePid` injection preserves an existing `GIT_CONFIG_COUNT`,
  and a real `git add` blocked in a hook leaves its PID file;
- foreign `index.lock` with a live PID-file owner,
  a PID file naming an exited process,
  a PID file naming a process started after the lock's ctime,
  no PID file,
  and a lock held open by a child process;
- a lock whose owner is a zombie is retired by the owner-lock acquirer,
  the hook dispatcher shim,
  and the transaction-owner liveness check;
- a wait released when the holder exits (and abandoned after `CLI_GIT_LOCK_WAIT_SECONDS`),
  and a leftover lock from a killed Git producing `index-lock-unproven-owner` after the timeout with the lock retained;
- a concurrent `git add` during a landing waits and then succeeds;
- `git cli-git fix` concurrent with a landing keeps its real index check valid;
- concurrent commits in a linked worktree never contend on the settlement lock,
  and `git worktree add` through the wrapper from a hook still synchronizes ignored state.

Recovery:

- `git status` succeeds while another commit's hook runs;
- two prepared and one landing transaction with dead owners recover in one startup;
- a live owner is skipped while a dead one beside it recovers;
- a crashed landing followed by another landing is recognized through the reflog nonce search;
- interruption at every journal state:
  before native Git,
  after preparation,
  after the reservation request,
  after object migration and before `landing-<n>.json`,
  inside the landing before and after compare-and-swap,
  before conclusion-state cleanup,
  after the index install,
  and before added-path worktree completion,
  each leaving no shadow repository and no `.keep` carrying the transaction's keep message after recovery;
- a legacy `cli-git-transaction` directory stops the command with exit `2` and instructions instead of recovering.

Auto-push:

- several landings produce fewer pushes than commits and the remote contains every landed OID;
- a joined push failure is reported by every joiner with exit `0`;
- a killed pusher is taken over after the liveness check;
- detached `HEAD` and missing-remote skips are unchanged.

Each fixture asserts exact ref,
reflog,
real index,
and worktree bytes before and after.
Deterministic interleaving uses Node hook programs that write a readiness marker and wait for a release file;
shell hooks are not used.
State-mutating verification uses disposable repositories only.

### Container end-to-end suite

Concurrent mutation of shared Git state is verified only by running it concurrently against real repositories.
Unit and packed shadow-bin fixtures cannot show interleavings,
crash recovery,
or lock contention,
so this suite is an inherent part of the transaction protocol.

- A mise task (`test:e2e:concurrent:native`) builds the release executable into an image and runs a consumer script
  inside `podman` with stated memory and CPU bounds.
- The image provides Git 2.55.0 only.
  The native executable requires Git 2.55.0
  (see "Compatibility and degradation"),
  so the run covers the whole catalog on that version.
  Distribution images that ship an older Git do not qualify as the only Git.
- Each run creates new repositories and a local bare remote inside the container,
  never touching host repositories.
- Workloads come from two sources:
  - commit-shape traces mined from this repository's own history
    (files per commit,
    path overlap,
    sizes,
    additions,
    deletions,
    renames,
    and binary files)
    with synthesized content;
  - a scenario catalog of concurrent agent behavior:
    shared-file edits with overlapping and non-overlapping hunks,
    interleaved index writers,
    config-based hooks,
    lint-staged-style stash hooks,
    `commit-msg` and `post-commit` hooks,
    SSH signing,
    amend attempts,
    branch switches,
    foreign `index.lock` holders,
    `gc --prune=now` during preparation,
    and `SIGKILL` injected at every transaction phase followed by recovery.
- Runs are seeded;
  a failing seed replays deterministically.
- Landing phases that have no hook are reached through a test-only phase marker:
  `CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill[:<directory>]` makes the wrapper `SIGKILL` itself at the phase,
  and `<phase>:pause:<directory>` writes `<directory>/<phase>.reached` and waits for `<directory>/<phase>.release`.
  The phases are `capture-locked`
  (inside the capture lock,
  before the sequence number is allocated),
  `preparation-done`,
  `landing-locked`,
  `objects-migrated`,
  `ref-updated`,
  `index-installed`,
  and `race-lost`,
  reached after each lost landing race outside both locks,
  after any reservation it earned and before the replay.
  `race-lost` can be reached more than once,
  so its files carry the occurrence:
  `race-lost-<n>.reached` and `race-lost-<n>.release`.
  Only this explicitly test-named variable arms a marker;
  a malformed value fails the invocation.
- Every run checks these invariants:
  - each commit that exited `0` appears exactly once with exactly its captured bytes;
  - no worktree edit is lost;
  - the real index never stages a revert of landed content;
  - the remote contains every landed OID,
    except a commit landed on top of an amend of an already-published commit,
    which instead must surface the non-fast-forward auto-push rejection with exit `0`
    (`doc/decision/cli-git-concurrent-commits.md` "Amending published history");
  - no shadow repository,
    transaction directory,
    `cli-git` `.keep` file,
    lock,
    landed-capture record,
    or temporary capture-store file remains;
  - `git fsck` is clean;
  - exit codes match the JSONL events.

## Policy-specific parity

### Built-ins

The configurable core policies preserve every accepted and rejected command fixture through the unified policy engine.
Their fixed order is `require-root`,
`linked-worktree-only`,
`branch-worktree-only`,
`add-explicit`,
then `final-newline`.
`final-newline` defaults to `warn`;
the other built-ins default to `error`.
`branch-worktree-only` and `final-newline` are warn-safe.
Unsafe warn configuration emits a `configuration-warning` JSONL event but still forwards when only warning findings
remain.
Generic `--no-enforce-<policy-id>` escapes and the legacy safeguard aliases skip the complete policy lifecycle and are
stripped before real Git.
Escape-looking option values and pathspecs remain ordinary Git arguments.
The legacy fixed-rule implementations no longer participate in dispatch.

### Hints

A finding, an engine failure and a warn-unsafe notice may carry an additive `hint` string field after `message`.
The hint guesses what the person was trying to do and names the next step; it never changes `message`, `code` or `fix`.
Every finding code of a policy whose message has no alternative of its own, and every engine failure code,
has a hint, and the test suite fails when one is added without it.
Plain stderr notices that report a divergence from real Git's behavior end with a hint in the same voice.
A reader that ignores unknown fields is unaffected.

### Discarding commands

`linked-worktree-only` guards every command form that can overwrite or delete uncommitted work:
`stash` (except `list`, `show` and `create`),
`clean` without a dry run,
`reset` in the hard, merge, keep, mixed and default modes,
`restore`,
`checkout` with paths or a force option,
`checkout` and `switch` that move `HEAD`,
`switch` with a force or discard option,
`rm` and `mv` with force,
`checkout-index` with force,
`read-tree` with reset,
and the abort forms of `merge`, `rebase`, `cherry-pick`, `revert` and `am`.
A form is rejected in the main worktree and outside any worktree always,
and in a linked worktree while `git status --porcelain=v1 --untracked-files=no` is non-empty
(`clean` also counts untracked files).
A `HEAD`-moving form is rejected only while tracked changes exist.
A status Git cannot read is a `fact-unavailable` finding, never a clean worktree.
Repositories under an allowlisted tool cache are exempt.
The rejection names the form, what it destroys,
an alternative that fits the likely intent,
and the deliberate override (`--no-enforce-worktree` directly after the subcommand).

Every decision about a guarded form is appended as one JSON line to `<common-git-dir>/cli-git-audit.jsonl`
with `schemaVersion`, `timeMillis`, `pid`, `parentPid`, `worktree`, `form`, `outcome`
(`rejected`, `allowed`, `exempt` or `hatch`), `dirty`, `cwd` and `arguments`.
The file moves to `cli-git-audit.jsonl.1` at one mebibyte.
A failed write never changes the command's result.

After an explicit-path landing computes its post-index,
the wrapper lists the stage entries of the pre-landing and post indexes,
and warns with the differing paths outside the committed and guarded paths.
The warning never changes the landing.

### Forbidden root context

Reject root `CONTEXT.md` for commit candidates and direct check.
Do not reject nested files with that basename.
Deleted root candidates are clean.

### Forbidden strings

Use candidate bytes rather than incidental worktree bytes.
Post-commit scan uses landed commit ground truth.
Manual push runs a private Git `--dry-run --verify` pre-push probe,
parses Git's pre-push update records,
and validates negotiated remote OIDs with `git ls-remote --refs` before policy evaluation.
Do not infer destination ref values from cached tracking refs,
push output,
or hand-written refspec interpretation;
tracking refs only bound which commits are already published,
as the "Manual push" lifecycle defines.
Scan each newly published commit's own delta against its parents:
every parent for merges,
the whole tree only for parentless commits,
and deletions publish no content and are skipped.
Annotated tags peel to their target.
A pushed commit with no known published commit on its destination scans its final tree.
Directly pushed tree and blob targets scan their complete content.
Deduplicate exact candidate identities across updates.
Skip the bytes of a candidate whose blob a parent's tree already holds, as "Content already in history" defines.
Load unique Git blobs through one `git cat-file --batch` process per evaluation.
The scanner reads candidate content in process, and no scanner file is materialized.
Wrapper-added latency for every forwarded real Git command must remain strictly less than `2,000 ms`.
Explicit dry runs bypass manual-push policies.
Pure ref deletion is clean.
Indeterminate required content exits `2` with `content-unavailable`.

The scanner is linked into the executable and runs in-process;
 no scanner executable is resolved or started.
Before scanning,
apply the scanner's path-anchored `--all` exclusions for its configured rules file and canonical self-match sources.
Do not exclude unrelated nested files sharing those basenames.
Findings are redacted;
the scanner must not expose matched bytes.
A missing rules file named by configuration,
and a scanner failure are engine failures that exit `2`;
an incomplete policy result emits `policy-incomplete`.
The policy defaults to error and is warn-unsafe.
Preserve the independent SLSA-attested CI invocation.

#### Content already in history

A scanned candidate is an added or modified path, so a rename lists its file as an addition,
and a merge lists every file that one parent lacks as an addition,
although the bytes are already in history.
A blob's object name is the hash of its bytes, so a candidate whose blob a parent's tree holds carries no new byte:
rescanning it only refuses moves of content that was accepted when it entered history.
The policy therefore reads no bytes for such a candidate.

The reference trees depend on the request that produced the candidates:

- `Add`, `Staged` and `StagedAtBase`: the tree of the baseline commit,
   which is the recorded base of a commit transaction, else the commit `HEAD` names.
   An unborn branch has none.
- `Landed`: the tree of every parent of the landed commit.
   A root commit has none.
- `Pushed`: the tree of every parent of a pushed commit that is not itself a pushed commit,
   so only history the remote already had vouches.
   A push that carries a tip tree (a tip with no known published commit) has no such parents and skips nothing,
   because its candidates share one version with the commits' candidates.
- `Direct`: none.
   A direct check reports what the selected files hold now.

The rule is exact and local:

- The comparison is the blob object name of the candidate against the blob object names of the reference trees,
   per candidate.
   It never compares pathnames and never uses rename or similarity detection.
   A changed byte changes the object name and the candidate is scanned in full.
- Only scanned blobs qualify.
   A deleted path, a submodule entry and the rules file and canonical self-match sources the scanner excludes
   keep their existing handling, and a gitlink in a reference tree never vouches for a blob.
- A candidate that this invocation's fix corrected never qualifies,
   because its bytes are no longer its object's.
- A known candidate is still given to the scanner with empty content,
   so a pathname that matches a rule is reported as before, masked.
- Each reference tree is listed at most once, in reference order, and listing stops when every candidate is placed.
   No Git process starts for a version without a scannable candidate or without a reference.
- Failing to list the reference trees never allows a candidate:
   every candidate is scanned in full and a warning names the failure.
- Each skip is a decision record on the debug channel (`MONOCHROMATIC_VERBOSE=true`):
   the scanner's masked pathname, the blob's object name, and the commit whose tree holds it.

Security reasoning.
The policy exists to keep secrets from entering history and from being published.
A blob a parent already holds is not new exposure: it is already in the tree the new commit builds on,
and in a push it is already on the remote.
Scanning it again blocks legitimate moves without protecting anything new.
A push counts only parents the remote already had,
so a secret introduced by a pushed commit is scanned at that commit even when a later pushed commit copies it.

Tradeoffs.
Content that entered history through an earlier bypass stays unscanned when it is copied or moved,
and so does content at the paths the scanner never reads, such as the rules file and its own rule sources,
which is what lets the scanner package move.
A rule added after a file entered history applies to the file again only when its bytes change.
The skip can only be as strong as the parent trees: it is never taken from the worktree or the index.

### Final newline

`final-newline` is a core policy enabled at warning severity by default.
An explicit error override restores blocking enforcement.
Selected non-empty text ends with exactly one LF.
Remove every terminal LF before adding one.
Empty and binary-looking bytes stay unchanged.
A file with CRLF content receives one terminal LF without normalizing interior line endings.

Preserve these generated-output exclusion families exactly:

```text
**/dist/final/node/**
**/bundle/node/**
```

A file whose Git `text` attribute is unset (`-text`, which the `binary` macro includes) keeps its exact bytes.
Only the value `unset` preserves a file:
`set`, `auto`, an unspecified attribute, and any other value leave the file checked.
A package declares its exact-byte inputs in its own `.gitattributes`,
for example `seed/** -text` beside a fuzz target,
so the exemption moves with the package and no path list in this policy needs to follow it.
No package path is listed:
fuzz seeds (`forbidden-strings/cli.fuzz`, `jsonc-edit/core-rust.fuzz`)
and the toml-edit fixture sources (`package/test-fixture/toml-edit/.gitattributes`)
are exempt through their own `-text` declarations.

The attribute is read from the state the candidates were listed from, never from the worktree:

- Candidates of `git add`, of a direct command, and of a commit transaction are read from the private index,
  so a `.gitattributes` staged by the same command or commit applies to it,
  and an unstaged edit of one does not.
- Candidates of a landed commit are read from that commit's tree.
- Candidates of a manual push are read from the tree of the commit,
  or the pushed tip tree,
  that first lists each path.

Git resolves the attribute, so nested `.gitattributes` files, macros,
the repository's `info/attributes` and the configured global attributes file apply as they do for Git itself.
The attribute of a file is asked only when its bytes are not canonical,
in one batch for the whole lifecycle,
so a lifecycle whose files all end with one LF starts no attribute query.
A lookup that fails fails the policy: the file is never assumed to be text or binary.

Patch-capable commit transactions automatically normalize would-be-committed bytes.
At default warning severity,
read-only commit selection and other pre-forward commands,
including `git add`,
warn and continue without offering an inapplicable patch.
An explicit error override makes those findings blocking.
Direct fix affects selected worktree bytes only.
Every real index entry remains exact during direct fix.

## Benchmark method

Issue #356 records measured budgets;
this specification does not invent numeric thresholds.

The benchmark harness that produced the records in `perf/` and its `perf:*` task no longer exist;
a new measurement needs a new harness.
A harness uses built production artifacts and disposable repositories.
Each scenario has a paired direct-real-Git baseline using the same executable,
repository,
command,
filesystem cache state,
and stdio sink.
Network operations use a local bare remote.
Scanner fixtures are deterministic and local.

Measure these scenarios separately:

- no config forwarded command;
- known read-only command;
- built-in validator;
- linked scanner;
- normalizer clean path;
- normalizer changed path;
- post-commit policy and local auto-push;
- concurrent commits at concurrency levels 1,
  2,
  4,
  and 8 with disjoint paths;
- same-file non-overlapping replays;
- conflicting pairs;
- a slow hook with `hooks.concurrentCommits` set to `false` and to `true`;
- a sweep of `landing.reserveAfterLostRaces` that confirms or replaces the default.

The sweep decides the default by the owner's rule,
most correct first,
then most performant:
every setting lands the same commits with the same bytes
and bounds a holder at `landing.reserveAfterLostRaces` + 1 lost races,
so performance decides.
When the per-commit p95 of the best settings differ by less than the run-to-run band,
the lower per-commit median wins;
otherwise the lower per-commit p95 wins.
The run-to-run band of a setting is the spread of its per-commit p95 across four sweeps of one build
with the setting order rotated as a Latin square
(orders 1-2-4-8,
8-4-2-1,
2-8-1-4,
and 4-1-8-2).

Default changed to `1` on 2026-09-26
(veto open):
four sweeps of the build that runs automatic maintenance after landing
(`5f79404b1`,
Git 2.47.3,
concurrency 8,
30 recorded batches per setting,
`perf/reserve-sweep-2026-09-26-maintained-run-<n>.json`;
the last three sweeps recorded `35db9ea0a`,
a commit that changed only Markdown)
gave per-commit p95 of 2630 to 2774 ms for `1`
(band 144 ms),
2823 to 2899 ms and one 3145 ms for `2`
(band 322 ms),
3128 to 3520 ms for `4`,
and 3185 to 3458 ms for `8`.
The mean p95 of `1` is 232 ms below that of `2`:
more than the band of `1`,
less than the band of `2`.
Both branches of the rule pick `1`:
its per-commit median of 2009 to 2062 ms sits below the 2278 to 2544 ms of `2` in every run,
and its p95 was the lowest of all settings in every run,
with no overlap between the p95 ranges of `1` and `2`.
Lost races per commit were 1.02 to 1.09 for `1`,
1.63 to 1.65 for `2`,
2.53 to 2.59 for `4`,
and 3.00 to 3.08 for `8`;
batch wall time medians were 2581 to 2635 ms,
2768 to 3095 ms,
3086 to 3450 ms,
and 3154 to 3309 ms.

Earlier sweeps,
superseded because every landing then left one more pack in the real store
(see "Post-landing"),
so later batches in a sweep ran against more packs than earlier ones:
four sweeps (the first at revision `2ec229081`, the other three at `d42a4fab3`,
Git 2.47.3,
concurrency 8,
30 recorded batches per setting,
setting order rotated as a Latin square,
`perf/reserve-sweep-2026-09-26-run-<n>.json`)
gave per-commit p95 of 4342 to 5616 ms for `1`
and 4379 to 5530 ms for `2`,
so the run-to-run band of either setting
(about 1150 to 1270 ms)
is more than ten times the 91 ms between their means,
and `1` had the lower p95 in only two of the four runs.
`8` (5615 to 5806 ms) was slower than both in every run,
and `4` (5257 to 5392 ms) was slower than both in three runs;
in the second run its p95 (5318 ms) was below both.
Per-commit medians do separate:
`1` gave 2586 to 2767 ms and `2` gave 3024 to 3106 ms in every run,
with 1.16 to 1.24 against 1.63 to 1.71 lost races per commit.
Batch wall time medians were 3765 to 3965 ms for `1`,
3897 to 4045 ms for `2`,
4511 to 4749 ms for `4`,
and 4844 to 5035 ms for `8`;
batch wall p95 was 5037 to 9401 ms,
4927 to 8871 ms,
5750 to 5975 ms,
and 6162 to 6472 ms,
where the two highest values came from one run's last two positions.
Those sweeps kept `2`,
because the 91 ms p95 difference of `1` and `2` sat far inside either band.

Concurrent scenarios also report per-commit completion time,
landing lock hold time,
real `index.lock` hold time,
lost races per commit,
and a paired serialized baseline.
Because every non-dry-run commit now prepares privately,
the lifecycle baseline is re-measured and stored as a new dated `perf/lifecycle-latency-<date>.json`.
Timing comparisons first measure the run-to-run band on one unchanged build.

Repeated wrapper commits in one repository measure a moving state,
not one fixed scenario:
before automatic maintenance ran after landing
(see "Post-landing"),
every landing left one more pack in the real store,
so per-commit time grew with the number of earlier wrapper commits while direct Git stayed flat.
With synthetic one-commit packs on Git 2.55.0,
30 serialized commits had a median of 435 ms at 250 packs,
521 ms at 500,
716 ms at 1000,
and 1062 ms at 2000,
against a 339 to 344 ms band at none;
with maintenance the medians stayed between 343 and 351 ms.
Commit history length alone,
2000 commits in one pack,
did not move the median.
`doc/troubleshooting/git-plumbing-commit-auto-maintenance.md` holds the trace and source citations.

For each scenario:

1. Record platform,
   Node,
   Git,
   filesystem,
   CPU,
   and build revision.
2. Run isolated warm-up samples until the harness-defined stable warm-up rule is met.
3. Collect at least 30 successful measured samples.
4. Report median,
   p95,
   median absolute deviation,
   direct-Git baseline,
   and wrapper-added delta.
5. Keep stdout and stderr destinations identical between paired runs.
6. Exclude failed samples only with a recorded failure reason;
   any systematic failure fails the benchmark.
7. Store raw samples as CI artifacts.

A performance budget is accepted only after at least one measured baseline on each enforced operating system.
CI compares like-for-like scenarios and reports both absolute and relative regression.

## Package and user-boundary verification

Before release readiness:

- the package README exists and matches this interface;
- Clippy reports no warning and the crate's tests pass in the bounded container
  (`mise run //package/git-policy/cli:native:test:container`);
- the release executable builds with `cargo build --release --locked --bin cli-git-native`;
- the `git` that `PATH` resolves first leads to `cli-git-native`
  (`readlink --canonicalize "$(command -v git)"`);
- the wrapper and all management commands run through the installed executable;
- direct fix proves index preservation;
- the container end-to-end suite
  (see "Container end-to-end suite")
  passes;
- a signed commit lands and auto-pushes with the user's real credential helper,
   and a manual push succeeds.

The crate is published to crates.io by `.github/workflows/cargo-publish.yml`;
 no step uploads to npm.
