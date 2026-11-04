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
 `mono/dependent-version-bump`,
 `mono/catalog-specifiers` and
`security/forbidden-strings`) run only where `cli-git.config.jsonc` lists them.
A no-config invocation therefore runs the built-ins only.

The forbidden-strings scanner is linked into the executable and needs no separate installation.
`mono/catalog-specifiers` links the `monochromatic-catalog-specifier` crate the same way
 (`package/catalog-specifier/core-rust`,
 pinned to an exact version in `Cargo.toml`).
The crate owns the grammar and the finding wording,
 and the policy re-implements none of it:
 it gives the crate each candidate `package.json`'s bytes from the candidate store
 (never the worktree's),
 and gives it only the path of any other candidate,
 so the crate refuses `package.json5`,
 `package.yaml` and `package.yml` by file name without a read.
Candidates are matched on the last segment of the raw path bytes,
 so a manifest below a directory whose name is not UTF-8 is still checked
 and its finding carries the exact bytes.
The policy runs on `pre-forward`,
 `post-commit`,
 `manual-push` and `direct-check`,
 and proposes no correction.
An executable that predates the policy refuses a configuration that lists it,
 as for any unknown policy ID,
 so a repository lists the policy only after every checkout runs an executable that ships it.
`markdown/autofix` runs the `monochromatic-lint` fix engine in-process;
 the executable links the linter library without its semantic engine.
A panic or a stall past 60 seconds in the linter ends that candidate's check, never the command;
the abandoned thread stays until the command ends, and the command starts no second linter thread.
A stack overflow or an abort in the linter is not contained and ends the command, so the guard is a hang and
panic boundary and not a crash boundary.
A run whose fixed source passes 64 MiB, or whose findings pass 4 MiB, is refused as unchecked.

## Policy re-run after a retried landing

After a commit is prepared again over a branch that moved,
the policies converge again over the merged tree,
and start no hook.
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
A commit adds no paths of its own: only the paths it selected reach the landing.

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
     `indexLock`,
     `landing` and `discardedCopies` exist.
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
    `security/forbidden-strings` accepts `builtinRules` and `rulesFile`;
     `rulesFile` is one path string (a required file) or a non-empty array of path strings and
     `{"path": ..., "optional": true}` objects, with no path named twice and every path checked as before.
6.  Validate the concurrency keys (see "Concurrency configuration").
7.  Resolve the rules files of `security/forbidden-strings`:
     `rulesFile`,
     else the environment variable
    `FORBIDDEN_STRINGS_RULES`,
     else `forbidden-strings.local.txt` at the repository top level,
     which may be absent.
    The variable is the scanner's comma-separated rules list, where a trailing `?` marks an optional entry.
    A file named by `rulesFile` or by the variable must exist unless it is optional;
     an optional file that is absent is skipped and logged.
    Every listed file is excluded from the scan.
8.  Emit configuration warnings for explicit unsafe `warn` settings.

Unknown top-level keys,
unknown or mistyped concurrency keys,
unknown policy IDs,
duplicate keys,
invalid severities,
unknown or mistyped options,
and an unreadable or oversized file exit `2` before any policy runs.

### Concurrency configuration

Three nested keys once tuned concurrent commits.
None of them disables concurrent commits;
no configuration key or environment variable opts out of them.
All three are inert:
nothing reads their values.
An older file that sets one still loads, and the value is checked and ignored.

- `hooks.concurrentCommits`:
  boolean,
  default `false`.
  It tuned the hook lock, and the wrapper runs no external hook, so no lock exists.
  `git cli-git doctor` says so.
- `indexLock.unprovenOwnerTimeoutMs`:
  non-negative safe integer,
  default `1000`.
  It was the backoff budget of the wrapper's own wait for a foreign `index.lock` whose owner is dead or unproven.
  The service's bounded wait for a foreign `index.lock` (`CLI_GIT_LOCK_WAIT_SECONDS`) replaced it.
  `git cli-git doctor` names the key when a file sets it and says to remove it.
- `landing.reserveAfterLostRaces`:
  positive safe integer,
  default `1`.
  It was the number of lost landing races after which a transaction asked for a landing reservation.
  The service lands in arrival order and bounds the retries of a commit (see "Retry after a moved branch").
  `git cli-git doctor` names the key when a file sets it and says to remove it.

Unknown nested keys,
wrong types,
fractional values,
and out-of-range values are config failures.
An absent config uses the defaults.

### Discarded copies configuration

`discardedCopies` is not a concurrency key and is not inert.

- `discardedCopies.maxAgeDays`:
  safe integer of at least 1,
  default `30`,
  the days after which a saved copy of a conflicted file is removed.
- `discardedCopies.maxSize`:
  safe integer of at least 1 (bytes) or text such as `"256m"`,
  default one gibibyte,
  the size the folder of saved copies is kept under.

The pass that applies them is described under "Conflicted files during a merge, rebase, cherry-pick or revert".

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
So does a `--policy` selection that names a policy whose effective severity is `off`,
 because that run would check nothing and exit `0` with no output.
The message names every such policy once,
 in selection order,
 with the `cli-git.config.jsonc` key and the severity that turns it on
 (the policy's own default severity),
 and the check happens after the configuration loads and before any candidate is prepared.
Lifecycle runs
 (a commit,
 an add,
 a push,
 the post-commit check)
 never refuse:
 they skip policies that are off.
A selected policy that is on but does not run for the command
 (a policy with no `direct-fix` trigger selected by `fix`,
 or a command policy such as `require-root` selected by `fix`)
 is refused the same way,
 because the run would check nothing and exit `0` with no output.
The message names the command that does run it,
 or, for a policy that runs for neither direct command,
 says that it runs only when the Git command it guards is forwarded.
A selection that mixes policies that run with ones that do not is refused for the ones that do not.
The escape `--no-enforce-<policy>` written for a selected policy is not refused.

### Doctor

`git cli-git doctor` lists the traditional hooks a repository holds.
Traditional hooks are unsupported (see "Commit hooks"),
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
- A hook file that is byte for byte the shim git-lfs writes for its own name
  (`pre-push`, `post-checkout`, `post-commit` or `post-merge`; the text of git-lfs 3.8.0, `lfs/hook.go`, with its final
  line feed) is a Git LFS shim.
  Only a regular file counts; a link, a file that differs by one byte and a shim under another hook's name do not.
  The comparison reads at most 1024 bytes of those four names and nothing else.
- It prints one path per line on stdout for every hook that is not a Git LFS shim,
  the default directory first and each directory in name order,
  and nothing else on stdout.
  When it finds any it also explains on stderr that traditional hooks are unsupported.
- Git LFS shims are reported on stderr only, apart from the other hooks, with their paths and an explanation:
  git-lfs writes these files again whenever it cleans a file in a Git that the wrapper did not start,
  so they can come back after deletion, and this is normal;
  the wrapper does the LFS upload itself and starts Git with traditional hooks off, so the files never run through it;
  nothing needs doing, and the files may be deleted or left.
- Exit status: `0` when no hook is found or only Git LFS shims are found (they are expected and harmless),
  `1` when any other hook is found, also when Git LFS shims are present beside it,
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
a command that creates or moves a worktree copies the main checkout's ignored state, under the leave-out rule below,
into the new worktree, under the settlement lock and the journal.
The main checkout is an applicable source for that command only; its other commands cost no filesystem reads.
After real Git and its hooks return for an applicable source,
cli-git compares the administrative set again.
Every new identity is a synchronization destination,
regardless of whether argv named `worktree add`,
an ordinary Git alias expanded to it,
or real Git returned nonzero after retaining registration.
A missing or stale pre-existing worktree root does not hide a newly created identity.

For an applicable non-bare invocation,
the source is the invoking linked worktree root, or the main checkout root for the exception above,
after Git and `post-checkout` settle.
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
No repository configuration touches this behavior.
The wrapper-only flag `--no-worktree-copy` skips synchronization for one invocation.
It is recognized in flag position after the subcommand and before Git's `--` pathspec separator,
and, like the other wrapper controls, among Git's global options before the subcommand,
is stripped before real Git runs,
and leaves value-position tokens with the same bytes untouched.
An opted-out invocation performs no journal recovery;
pending journals wait for the next repository invocation without the flag.

### Cheap worktree creation and removal on btrfs, XFS and overlayfs

Design, measurements and the reasons for each choice: `doc/planning/cli-git-btrfs-worktree.md`.
The ladder applies to a `git worktree add` that names one path and at most one commit-ish and uses only `-b <branch>`,
`--detach`, `-d`, `--quiet`, `-q`, `--lock`, `--reason` and `--checkout`, run through the wrapper from a linked worktree,
a bare repository or a main checkout, without `--no-worktree-copy`, without a global option other than `-C`, `--no-pager`,
`-P`, `-p`, `--paginate`, `--no-advice` and `--no-lazy-fetch`, and with an existing parent directory of the destination.
Any other form is Git's own, unchanged.
The ladder runs inside the settlement lock, and every rung that cannot run falls to the next without a message to the
person and with one diagnostic line (`MONOCHROMATIC_VERBOSE=true`) that names the rung and the reason.

A source is the worktree the command runs in, the main checkout included.
It is clean when no merge, cherry-pick, revert, bisect, rebase, sequencer or `index.lock` state exists in its Git
directory, its index equals the tree of its `HEAD` (no staged change, intent-to-add, skip-worktree, unmerged or
submodule entry), every tracked file has the size, modification time, change time, inode, owner, type and executable
bit its index entry recorded and is older than the index file, and `git ls-files --others --exclude-standard` lists
nothing.
A source is also refused for a sparse checkout, for any worktree-scope configuration entry, and for `core.sparseCheckout`.

Rung 1 needs a clean source that is a btrfs subvolume root on the destination's filesystem and, for a source that is a
linked worktree, no nested linked worktree that is an ordinary directory; a snapshot of the main checkout sets such a
worktree aside instead (see "Leaving out of a snapshot of the main checkout").
Real Git runs with `--no-checkout` added after `add`.
The wrapper then reads the new `HEAD`, refuses a subject Git would have to print or `.gitattributes` files that differ
between the source's tree and the new tree, claims a stage name beside the destination in the settlement journal root,
snapshots the source there (`BTRFS_IOC_SNAP_CREATE_V2`), makes the stage private, removes the stub of every nested
linked worktree and of the destination, replaces `.git` with the bytes Git wrote, compares every tracked file of the
snapshot with the copied index, installs the index with `rewrite_index_stat_with` (extensions `UNTR` and `FSMN`
dropped), applies the tree difference with `git read-tree -m -u <source commit> <new commit>` under Git's
`GIT_DIR` and `GIT_WORK_TREE`, gives the stage the source's mode and swaps it with the destination
(`RENAME_EXCHANGE`).
The directory real Git made, now at the stage name, is settled without a recursive removal: Git's pointer and the empty
directory go, and anything another process added to it meanwhile stays there and is named in a warning.
Git's line `HEAD is now at <abbreviation> <subject>` is printed unless `--quiet` was given.
The snapshot carries the whole ignored state of the source, so the ignored-state copy skips that destination.

Leaving out of a snapshot of the main checkout:
before the stage is made private and its stubs removed, the wrapper prunes a snapshot whose source is the main checkout
with the main-checkout rule of `src/native/worktree_copy_prune.rs`.
First it works out what the source's index tracks: every directory above a tracked file, and every directory-valued
index entry (a gitlink) at its own path. None of those is ever moved, whatever its name.
A registered nested worktree that is an ordinary directory and holds or is a tracked path cannot be left out, so the
snapshot is declined, before anything is touched, with a reason that names the worktree and the tracked path.
Then it unlinks the `.git` file of every other nested linked worktree that is an ordinary directory and of every other
directory that holds a `.git` file, because a copied pointer names another worktree's administrative directory.
Then it moves each such directory, and each directory the rule names as a cache or a run record, into a trash directory
`.claude/cli-git-trash/<forty hexadecimal digits>` inside the stage, by rename.
Names are kept as bytes: a name that is not UTF-8 is moved or kept by its bytes and never taken for another entry.
Every directory is opened by one plain name below the handle of the one above, without following a link, so a link at
`.claude` or at the trash parent in the stage is no way out of the stage.
When the trash cannot be made that way, or Git does not ignore the place (one `git check-ignore --no-index` on the
source), the directories are removed in the foreground, in place, by the same held-handle walk.
The leave-out writes down a manifest: the identity of the trash directory (device, inode and birth time, taken when it was
made) and the identity of each directory moved into it.
After the swap, the trash is handed to the commit service as a work item of kind `trash` in the new worktree's Git
directory, carrying the manifest. A new worktree has a service instance of its own whose socket activation does not exist
yet, so the hand-over first provisions it exactly as the worktree's first commit does, and then wakes the service through
its socket.
The service empties it only if all of these hold: the worktree's `.git` file, read without following a link, points at
the item's Git directory; every directory from the worktree down to the trash is a real directory on the worktree's
device; the trash has the recorded identity; and each entry removed is the recorded object (a replacement, a link or
anything unrecorded is left alone and named in the failure). Nothing is found by scanning the trash parent for names, so a
directory the main checkout carried there is never touched.
When no route to the service exists, provisioning fails, or the wake fails, the wrapper empties the trash itself by the same
rules before the command returns, after one plain line on standard error that names the worktree and says why the service
could not take it over. The work item stays until the trash is gone: it is withdrawn only after a successful removal.
A destination or Git directory whose path is not valid UTF-8 cannot be spelled in a record, so no work item is written and
the trash is emptied in the foreground by the bytes of the path.
A trash that cannot be emptied is reported on standard error as a warning that names the path and the reason and says
the work item stays for the service; it does not fail the add, because the worktree is already in place and complete.
The race with other processes is a risk the person accepts by using the command: a directory of the new worktree that another
process replaces, moves or links elsewhere while the leave-out or the disposal runs is guarded only by the cheap checks above
(opening without following links, comparing identities when an object is opened); the wrapper builds nothing further against
it and does not promise to survive it.
A trash that no one has emptied yet lies under an ignored path, so it is no untracked file.
The wake reaches only a service of this wrapper's release: a service of another release is asked to retire first and is
replaced, and when it does not retire within the start wait the wake fails, so the trash is removed in the foreground and
its work item is withdrawn once it is gone.
A snapshot of the main checkout does not print the copy summary, because no copy ran; unless `--quiet` is given it writes one
line to standard error saying the worktree was made as a snapshot of the main checkout and that nested worktrees, caches and
run records were left out.

Telling why an add was not made as a snapshot:
when a `git worktree add` that the ladder serves succeeds without being made as a snapshot, and the person could change
the cause, the wrapper writes one line to standard error unless `--quiet` is given, after Git's own words and before the
copy summary.
The line starts with `cli-git: this worktree was not made as a snapshot, so it took longer, because `, then the cause in
plain words, then what makes the next add quick, and it names no internal step.
The causes it tells are:
uncommitted changes in the source (the number of paths `git --no-optional-locks status --porcelain=v1 --untracked-files=normal`
lists, and the first of them, with control characters shown as `?`; the remedy is to commit or stash);
a running Git command in the source (its `index.lock` exists; the remedy is to add again when it has finished);
an unfinished operation (a merge, cherry-pick, revert, bisect or rebase mark exists; the remedy is to finish or abort it);
an index that cannot yet vouch for its files although Git lists no change (a file as new as the index, or a size, time or
inode that differs; the remedy is `git status` once);
a btrfs source that is no subvolume (the remedy is `git cli-git migrate-to-subvolume` for the main checkout and
`git cli-git convert-worktree-to-subvolume` for a linked worktree);
a destination on another btrfs filesystem than the source;
a feature a snapshot does not reproduce (a sparse checkout, a worktree-specific configuration entry, a nested worktree
reached through a link), told in the check's own words;
and a snapshot that started and could not be finished, told with the reason, after which Git's own checkout made the worktree.
A source or destination that is not on btrfs gets no line, because nothing the person does makes the add a snapshot there.
A failed add gets no line either; Git's refusal is the thing to read.
The same words go to the journal as one entry of priority `warning` under the `SYSLOG_IDENTIFIER` `cli-git`, with the
fields `CLI_GIT_WORKTREE` (the destination), `CLI_GIT_SLOW_CAUSE` (`uncommitted-changes`, `git-running`,
`operation-unfinished`, `stale-index`, `unproven`, `not-subvolume`, `other-filesystem`, `unsupported` or `snapshot-failed`)
and `CLI_GIT_SLOW_DETAIL` (the check's reason as the debug log prints it), also under `--quiet`.
The entry is one datagram on the journal's native socket, sent without waiting; a journal that is absent or full costs the
command nothing, and `journalctl --identifier=cli-git` lists the entries.
The debug line with the reasons of every step stays (`MONOCHROMATIC_VERBOSE=true`).

Converting a nested worktree:
`git cli-git convert-worktree-to-subvolume [--dry-run | --abandon] (--all | <worktree>...)` converts a registered linked worktree
that is an ordinary directory into a subvolume at the same path, one at a time.
It refuses when the worktree is locked, has an `index.lock`, has an operation in progress, has a modified tracked file, has a
process with its working directory or an open file inside it, has a worktree registered inside it, or when its converting
name exists without a journal.
A tracked file counts as unmodified when its index entry's size, times and inode match it.
An entry those fields cannot vouch for (as new as the index file, or a stale stat in a copied or cloned worktree) is settled
by content in process, without a Git process: the file is hashed as a blob (a symbolic link by its target) and compared with the
entry's object name.
Content cannot settle a path that a `filter`, `ident` or `working-tree-encoding` attribute may apply to, a file with a carriage
return, a file over 32 MiB, or a difference of type, executable bit or assume-unchanged mark; each is refused with its reason,
naming the path.
The attribute sources are the tracked `.gitattributes` files from the root down to the file, `info/attributes`, the file
`core.attributesFile` names or the default global file, and the system file, which is where the Git executable says it is
(`git var GIT_ATTR_SYSTEM`, learned once per executable and cached) and is off under `GIT_ATTR_NOSYSTEM`; it is never derived from
the system configuration path, and when its place cannot be established the worktree is refused.
Attribute lines, patterns and paths are read as bytes: only space, tab, carriage return and line feed separate tokens, a NUL ends
the text of a line, a byte order mark on the first line is skipped, `?` is one byte and only ASCII letters fold; a spelling the
matcher does not model counts as applying.
Every tracked file and every `.gitattributes` below the worktree is opened below a held root without following a link (a
directory replaced by a link is refused); the type and mode of a file are judged on its open handle, its state is compared
before and after the read, and the read is bounded at the limit plus one byte, so the bytes that were hashed are the bytes of
the one file that was judged.
The rungs of the add keep the stat-only check.
A real run holds an exclusive kernel lock (`cli-git-convert.lock` in the administrative directory, left in place) from before it
reads the journal until it ends, so two commands never read, write or resume one journal; a command that finds the lock held
skips the worktree and says so.
It creates the journal `cli-git-convert` exclusively, through a temporary file created exclusively under a unique name, takes
`index.lock` with a token, creates `<path>.cli-git-converting` as a subvolume and records its identity (device, inode and birth
time) in the journal right after.
A directory at the converting name that the journal has no identity for is foreign: it is refused, never synced into,
exchanged or destroyed, and the abandon leaves it.
It copies the worktree into the stage with cloned data and runs a second pass that must find nothing to copy; `Synced` only
records that this once held, so every resume before the swap copies and proves again.
It exchanges the two directories (`RENAME_EXCHANGE`), checks that the path is a subvolume root, is the subvolume whose identity
the journal recorded and holds the same `.git` bytes, and rewrites the index stat fields; a refresh that did not install the new
index stops the conversion with the old tree kept.
Before it removes the old tree it copies anything newer from the old tree into the worktree (an update-only pass, which never
deletes), reads the process table again for anything holding the old tree, and keeps the old tree for that run, with a message,
when it carried anything over or found a holder.
It removes the old tree only when the directory at the name is the one whose device and inode the journal recorded, pinned by a
descriptor opened below its parent, deleting beneath that descriptor and removing the name last only while it still names that
object.
Then it drops the journal and the lock.
Nested registrations are read from the common directory whatever it is called (a bare repository included), and the refusals have
kinds: only the `index.lock` that holds this command's token and the stage whose identity the journal recorded are exempt, never a
refusal because of its words.
A rerun completes an unfinished conversion in place whatever phase it stopped at, including a stop between the exchange and
its journal line; `--abandon` destroys only the subvolume whose identity the journal recorded, and only before the exchange.

What keeps a writer out between the proof and the deletion: nothing excludes one while the old tree is still the worktree.
The proof, the copy and the second pass see the tree at their instants; the process table is read before the swap and again before
the removal, a write that lands in between is found by the update-only pass after the exchange (the old tree is then reachable by
no name anyone uses), and the old tree is kept, never removed, whenever that pass carried anything over or a process still holds it.
A process that wrote and exited between the second pass and the exchange is the case the update-only pass exists for; a write
that keeps the size and modification time of the copy is the case it cannot see.

Rung 2 needs a clean source on btrfs, XFS or overlayfs, with the source and the destination in one clone domain.
The same `--no-checkout` run is followed by `FICLONE` of every tracked file (mode and nanosecond times kept; links
recreated), a second comparison of the source with its own index, the installation of the index, and the same tree
difference.
A refusal removes nothing, because a name cannot be checked and unlinked in one step: the entries the clones created stay,
`git reset --hard` overwrites the tracked ones, a message counts them and names `git -C <path> status` for any other, and a
file another process put into the destination is never touched. The index records the stat fields of the regular files the
wrapper made, taken from their descriptors; a cloned link keeps the source's stat fields, which Git compares by target.
The ignored-state copy then runs as for Git's checkout.

Rung 3 is Git's checkout and the ignored-state copy.
Where the destination's parent is on btrfs and the rung is 2 or 3, the wrapper first creates the destination as an
empty subvolume (`BTRFS_IOC_SUBVOL_CREATE`), which Git accepts as an empty directory, and removes it again if Git leaves
it empty.
No rung runs a hook: every real-Git start of the wrapper switches hooks off.
The wrapper never deletes or unregisters a worktree, a registration or a stage it did not make in the running process, and
the only entry it removes by name is its own lock file, while the name still holds the file it made.
After Git's `--no-checkout` run it binds the add to the two directories Git made (the destination and its administrative
directory, proven to name each other below this repository's `worktrees`, the pointer resolving to the registration without
any symbolic link; a pointer through a link ends the ladder for the add with a message that names the pointer and says to make
the worktree with a canonical path), records their identities, and checks the
binding again before every write into the worktree and before the fallback checkout; a worktree that was moved, replaced
or edited meanwhile is not written to, the add fails with one report, and nothing is changed.
When the fallback checkout fails, the worktree and its registration stay as they are, the command fails with Git's
status, and one report names the path, `git -C <path> reset --hard` (retry) and `git worktree remove --force <path>`.
A process killed between Git's run and the end of the add leaves the claim and a record; the next recovery changes
nothing, prints that report once for a bound add (or says that nothing proves a leftover is the add's when the add was
never bound), prints the stage it did not remove, and drops the record. A record or a completion marker that cannot be
looked up keeps the record and says that the state is unknown. The cost: an interrupted or failed add can leave an
incomplete registered worktree for the person to complete or remove.

`git worktree remove <directory>` with no option, for a registered, unlocked, clean linked worktree that is a subvolume
root, on a mount that lists `user_subvol_rm_allowed`, destroys the subvolume (`BTRFS_IOC_SNAP_DESTROY`) and deletes the
administrative directory.
Every other case, and a refusal by the kernel, is Git's own removal.

Before staging,
exclude every registered worktree root strictly nested under the source.
A main-checkout source also applies a leave-out rule of directory names, the same everywhere below the source root,
implemented in `src/native/worktree_copy_prune.rs`:
a directory that holds a `.git` file or directory is another checkout and is left out whole;
a cache is left out (`.pnpm-store`, `.gradle`, `.kotlin`, `.cache` anywhere but the repository root,
and `incremental` and `zig-global-cache` below a `target` directory);
and a run record is left out while `COPY_RUN_RECORDS` is false, which it is
(`.monochromatic` below a `node_modules` directory, `verification` directly under `target`, `mutants.out`,
`mutants.out.old`, `playwright-report`, `test-results`, and `cache` directly under `.serena`).
The repository root's own `.cache` is kept, because tasks read the tools and generated inputs it holds.
Dependencies, builds and local files are copied.
A linked or bare source copies everything it ignores except its nested registered worktrees.
A file or directory the running user cannot read is left out of every source's copy instead of failing the copy,
and one stderr line names the count and the first three paths:
`cli-git: left out <n> unreadable ignored entries from <source>: <paths>`.
Private stage names are reserved and excluded at every path component.
Stage each destination separately in a mode-`0700` sibling directory so file cloning stays
 on the destination filesystem.
Copy regular files into the stage with an exclusive copy-on-write request whose unsupported-filesystem behavior falls
back to a full copy.
A manifest of at least 4096 entries is copied by up to 8 threads, bounded by the CPUs the process may use,
over contiguous slices after every directory exists; the first failure stops the others.
Preserve directory and regular-file permission bits plus exact symbolic-link target text.
Never follow a symbolic link while classifying source or destination entries.

Build a deterministic parent-first manifest before copying.
After the private stage is complete and modes are applied,
rebuild the source and staged manifests and require exact final equality of selected roots,
entry paths,
types,
modes,
regular-file content,
and symbolic-link targets.
A regular file that a copy-on-write request cloned holds the source's data blocks as of the clone,
so its content is proved by the source's stamp (size, modification time, change time and inode) instead of by reading
both files: the stamp is read before and after the clone, must be equal, and must be at least 20 ms older than the clock,
and the final comparison requires the source's stamp to be unchanged.
No write can leave the change time unchanged, so a writer to the source during the copy fails the comparison.
A file copied byte by byte, or whose stamp was too recent to prove anything, is compared byte for byte.
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
after writing a stderr line naming the owner's PID, the lock and the limit, and repeating it with the time waited
(see "Wait notices"),
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
and the landing is complete
(see "Commit protocol"),
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

After a commit lands,
the push is a work item of the commit service,
coalesced per branch (see "Commit protocol").
The service pushes the branch its record names and never reads `HEAD`,
because the worktree may switch branches at any instant.
It is the only pusher of the repositories it serves and runs one push per branch at a time,
so the push takes no lock.
A failed push never fails the commit:
the service leaves a notice that the next command in the repository shows.

#### Commands that add commits

A command other than `git commit` hands the push over too (owner decision, 2026-11-02):
`merge`, `pull`, `cherry-pick`, `revert` and `am` (including the merge the wrapper completes from objects when the index holds
staged changes, and the `--continue` that ends a stopped sequence).
The wrapper stays alive around such a command instead of replacing itself with Git, reads the branch `HEAD` names and its tip
from the files before the command, and after a command that exited `0` hands over one push record (not the review or the
upkeep of a commit) when all of these hold:
- `HEAD` still names the same branch under `refs/heads` and its tip moved;
- the old tip is an ancestor of the new tip, which leaves out every rewrite of the branch (a rebase, `pull --rebase`,
  `pull.rebase`, an amend, a reset);
- the branch has exactly one `branch.<name>.remote` and one `branch.<name>.merge`, the remote is not `.`, and the fetch
  refspec maps the merge ref to a local tracking ref that resolves;
- the new tip is not that tracking ref's tip and the tracking ref's tip is an ancestor of it (ahead, not diverged).
Ancestry is a walk over commit parents from the new tip, at most 4096 commits per question; a walk that cannot tell hands
nothing over. No Git process runs for the check. A service that cannot be reached is said plainly, and the command's exit
status is unchanged. `git rebase` is never handed over.

The files are read as for a caller that follows a Git run that succeeded:
the structure checks of the in-process reader apply
(owner,
object format,
`core.bare`,
`core.worktree`,
reftable,
a valid `HEAD`),
and the proof that every configuration value is
one Git accepts does not.
That proof guards the wording of a failure Git would give for a malformed value,
and a command that already succeeded has none to give;
it is not a certificate that every value is valid,
because the files are read again and not the snapshot the command saw,
and a successful command types only the keys it consults.
A user-level `core.fsmonitor = true` once made the reader
decline the whole repository
(the proof then accepted only false spellings of the key),
so no merge was ever handed over for that user and nothing said why.
The proof now accepts `true`,
`yes` and `on` for that key
(and types `fsmonitor.allowRemote` and `fsmonitor.socketDir`,
which Git reads once the monitor is on),
and a hook path still declines it,
as do the bare name and the numbers other than zero,
which Git reads as true.
A same-user listener on the monitor's socket that answers with a malformed reply is an accepted limit,
by the owner's decision that the wrapper leaves Git's monitor alone.
An active `trace2.normalTarget`,
`trace2.eventTarget` or `trace2.perfTarget` in any configuration file declines the readers,
because Git then writes trace output they do not.
The trigger keeps reading as a caller that follows a successful Git run,
because any other value the proof cannot vouch for
would stop it the same way.

Every ending of the trigger leaves one line in `cli-git-push-trigger.jsonl` in the common Git directory (size-capped like
the commit timing file,
moved aside to `.1`),
and a debug line under `MONOCHROMATIC_VERBOSE=true`;
the terminal is told
nothing except a service that cannot be reached.
The line has `schemaVersion`,
`time`,
`pid`,
`release`,
`command`,
`outcome` (`handed-over` or `declined`),
`reason`,
`detail` and `branch`,
and since the file is shared by every worktree,
`worktree`,
`gitDir` and `instance` name the worktree that decided,
and `level` is `warn` for a decision the wrapper could not make or a hand-over that failed
(`layout-declined`,
`head-unreadable`,
`branch-has-no-tip`,
`object-database-unopened`,
`upstream-config-unreadable`,
`walk-undecided-growth`,
`walk-undecided-ahead`,
`no-route`,
`handover-failed`) and `info` otherwise.
`command-failed` names the branch and the tip it was at before the command in `detail`.
The userinfo of any URL in `detail` is masked as `<credentials>`.
The lines of a watch that could not start name the worktree but no branch.
The reasons are stable words,
one for each step:
`layout-declined`,
`head-unreadable`,
`head-detached`,
`head-not-a-branch`,
`branch-has-no-tip`,
`branch-changed`,
`tip-unchanged`,
`upstream-no-branch-name`,
`upstream-config-unreadable`,
`upstream-not-single`,
`upstream-is-local`,
`upstream-no-tracking-ref`,
`upstream-tracking-ref-missing`,
`upstream-equals-tip`,
`object-database-unopened`,
`walk-undecided-growth`,
`not-grown`,
`walk-undecided-ahead`,
`not-ahead`,
`no-route`,
`handover-failed` and `command-failed`;
a handed-over push has the reason `ahead-of-upstream`.

#### Destination

The push names both sides of its refspec, `<branch>:<ref>`, and the destination follows what a plain `git push`
of that branch would do if it were checked out (`setup_default_push_refspecs` in Git's `builtin/push.c`
and `match_push_refs` in `remote.c`).
A setting under which Git refuses, or pushes the branch nowhere, is a skip, never a push:
the push result is `Skipped` with exit code `0`, and its `shown` text is a note that names the branch,
the setting and the way forward, printed to the same stream and starting with `cli-git: auto-push: `.
One read of the configuration (`config --list -z --show-scope`, answered without starting Git where the
in-process reader can) gives every value of the multi-valued keys.

- A branch without an upstream is pushed to `origin` with `--set-upstream` when that remote exists,
  and skipped silently when it does not.
- The push remote follows Git's order
  (`branch.<name>.pushRemote`, then `remote.pushDefault`, then `branch.<name>.remote`).
  The remote `.` (a branch that tracks a local branch) has nowhere to push and is a skip.
- A push remote with `remote.<name>.mirror` true is a skip:
  Git refuses a branch pushed by name to a mirror,
  and a plain `git push` there mirrors every ref and deletes the others, which the service never does.
- Any `remote.<push remote>.push` value decides, whatever `push.default` says.
  Every refspec must be a full ref name or hold one `*` on each side, optionally forced (`+`) or negative (`^`).
  The refspecs are applied to the local refs the way `match_push_refs` applies them:
  explicit refspecs first, in order, where a source that is no local ref or two sources for one destination
  refuse the whole push;
  then each local ref, in name order, takes the first matching pattern unless an earlier pairing already took its
  destination;
  then a negative refspec removes the destinations it matches.
  The branch is pushed to each destination it receives.
  A branch that receives none is a skip.
  A refspec of another form (`HEAD`, a short name, an object name, a deletion, the `:` of matching)
  depends on more than local ref names, so the push is a skip that names the refspec.
  The `%(push:remoteref)` atom is not used, because it maps one branch alone and so misses all three of these rules.
- Otherwise `push.default` decides:
  `current`, and `simple` with a push remote other than the tracked remote, use `refs/heads/<name>`.
  `upstream` (with its deprecated synonym `tracking`), and `simple` on the tracked remote,
  use the one `branch.<name>.merge` value;
  a differently named one is pushed to as well, because that is where the person tracked the branch
  (a linked worktree branch created from `origin/main`), although Git itself refuses it under `simple`.
  Several `branch.<name>.merge` values, a triangular `upstream` and a tracked ref outside `refs/heads` are skips,
  as Git refuses them ("multiple upstream branches, refusing to push").
  `nothing` and a value Git does not accept are skips.
  `matching` runs the plain `git push <remote>` with no refspec and leaves the matching to Git
  (the owner chose this over a push of the recorded branch alone):
  every local branch whose name the remote's push URL advertises is pushed, siblings included,
  a branch the remote lacks is left alone, and Git's own failure ("No refs in common and none specified")
  is a failed push reported with Git's words.

#### Reconcile and bounds

- A push of a branch with an upstream that the remote refuses as stale
  (its output holds `[rejected]`, the marker Git does not translate)
  is reconciled by the same push
  whenever its destination is one ref of one remote that the settings name (see "Destination").
  The remote `.`, a `remote.<name>.push` refspec, `matching`, `nothing`, several upstream branches
  and a ref outside `refs/heads` name no such ref, and the rejection is returned unchanged.
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
- No step of the push waits without a bound.
  The push runs under a deadline,
  `CLI_GIT_PUSH_DEADLINE_SECONDS` whole seconds,
  30 by default;
  a zero,
  negative,
  fractional or non-numeric value is ignored.
  Past the deadline the push is asked to end,
  killed after a five second grace,
  and reported as failed with exit code 124 whose output ends with a line naming the deadline.
  A push whose output grows past 4 MiB is stopped the same way, and only the first 4 MiB are kept.
  The commit stays saved.
  Git's low-speed abort is added to the push's environment
  (`GIT_HTTP_LOW_SPEED_LIMIT=1000`, `GIT_HTTP_LOW_SPEED_TIME=60`)
  for each variable the invocation's environment does not set,
  which also overrides `http.lowSpeedLimit` and `http.lowSpeedTime` from Git configuration.
  Each read-only Git query before the push
  (`config`,
  `for-each-ref`,
  `rev-parse`,
  `remote get-url`,
  `merge-base`)
  stops at the push deadline or 120 seconds,
  whichever is shorter,
  and a stopped query is a failure that names it.
- A push that lost a race on a pending reference lock (`cannot lock ref` in a push that otherwise delivered)
  is repeated up to three times with growing pauses, because the two worktrees' services share the references.
- A failed push is reported with its complete output and the note
  ``cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready.``
  unless the reconcile already said why the commit is unpublished.
  A successful push shows only the `remote:` lines and the lines of the reconcile.

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

### Real Git starts with traditional hooks off

External hooks are not only unsupported: no start of the real Git by the wrapper runs one.
Every such start, forwarded commands as well as the wrapper's own children (queries, the commit's native steps,
the auto-push, the reconcile and integration pushes, maintenance), carries
`-c core.hooksPath=/dev/null/cli-git-no-hooks` after the caller's global options and before the subcommand.
The directory is below a device, so nothing can create it, and Git finds no hook file there.
The same prefix carries `-c hook.<event>.enabled=false` for every hook event Git knows, `pre-push` included, so a hook
defined in the configuration (`hook.<name>.event` with `hook.<name>.command`) does not run either.
`core.fsmonitor` is left as configured, like the other programs Git starts for its own features (filters, editors,
signing programs, credential helpers): the program it names is not reached by the hooks path or by any event switch
(measured on Git 2.56.0, including with `hook.fsmonitor-watchman.enabled=false`), and the built-in monitor daemon keeps
working.
This is the owner's decision ("To stop the fsmonitor program - we don't need to.",
recorded in `doc/handover/cli-git-rust-implementation.md`):
Git runs its monitor as the configuration says,
and the wrapper neither stops it, starts it, nor works around it.
The wrapper injects no hook of its own, so no event is an exception.

- The option is part of the argument vector, so it adds no system call and no process start.
  The three commit controls of `binary_budget_tests.rs` pass with their counts unchanged.
- It follows every global option of the caller and precedes the subcommand, so no `-c core.hooksPath=<directory>`,
  `-c hook.<event>.enabled=true` or `--config-env` of the caller can bring a hook back, and the native commit repeats
  the options after the caller's (see "External hooks").
- A command that reaches Git through an alias gets Git's own expansion from the wrapper (resolved from the alias
  values it already reads for classification), so the options follow the `-c` options an alias value may carry;
  put before the alias word they would lose to them (measured on Git 2.56.0).
  A shell alias (`!...`) is run by Git's shell, and a `git` it starts through `PATH` is the wrapper again.
- `git config` starts that only report or change configuration go without the options, because they report what the
  repository configures and the wrapper reads `core.hooksPath` that way (the commit warning, `doctor`).
  `git config --edit` (or `-e`) starts an editor, and a Git the editor starts inherits the options, so it gets them.
- A child of Git inherits the settings through `GIT_CONFIG_PARAMETERS`, so a program Git starts (a filter, an editor) that
  starts `/usr/bin/git` runs it with hooks off as well; a local-path remote's `receive-pack` does not inherit it (measured by
  `a_local_remotes_server_side_hook_still_runs`), so server-side hooks of such a remote still run.
  A script that starts Git outside any wrapper-started Git still runs traditional hooks.
- git-lfs reads the same setting when it picks the directory for its shims: a clean filter started inside a
  wrapper-started Git finds no directory it can write and writes none (measured with git-lfs 3.8.0;
  `binary_hooks_off_tests.rs` has the positive control, a direct `git add` that writes all four).
- The wrapper's dry-run push probes (the scope probe of a manual push, the integration probe after a rejected push, and
  the stacked push's own scope probe) carry `-c hook.pre-push.enabled=false` and a hooks directory that never exists,
  and a `--dry-run --porcelain` push reports its updates without any hook.
  No push runs a configured `pre-push` hook: the event is switched off like every other, and the wrapper injects no
  hook of its own.
- Measured with Git 2.56.0 only; Git 2.39 was not available on the host.
  A missing `core.hooksPath` directory is the case the shadow commands already rely on (`<shadow>/no-hooks`),
  and the container end-to-end suite runs that on Git 2.39.5.

### Git LFS upload before a push

A push publishes pointer files; the objects they name must reach the LFS server first, or collaborators fetch pointers
whose content does not exist.
Git LFS does this in its `pre-push` hook (`git lfs pre-push`),
which `git lfs install` writes as a traditional hook file.
Traditional hooks are unsupported, so the wrapper provides the same step itself, as a child it runs before the push.

Verdict per hook of the four `git lfs install` writes
(source: `commands/command_pre_push.go`, `command_post_checkout.go`, `command_post_commit.go` and
`command_post_merge.go` of git-lfs, and the man page `git-lfs-pre-push`):

- `pre-push`: handled natively, as described below.
- `post-checkout`, `post-commit` and `post-merge`: not needed.
  Each returns before doing anything unless a lockable pattern exists
  (`lfs.setlockablereadonly` is on by default, and `GetLockablePatterns()` is empty without the `lockable` attribute);
  what they do otherwise is set the read-only flag of lockable files that are not locked.
  The repository has no `lockable` attribute: no tracked `.gitattributes` file contains the word,
  and `git check-attr --all` on an LFS path lists none.
  A repository that adopts `lockable` reopens this verdict; until a hook or a wrapper step runs, its lockable files
  keep whatever write permission checkout gave them, and no content is lost.
  The wrapper runs none of the three, and a shim file for any of them never runs through the wrapper either.

The step:

- Trigger.
  The wrapper uploads before a push only when the repository's own Git configuration (local, worktree or
  command-line scope) holds a key under `filter.lfs.` or `lfs.`, and the push is not a dry run.
  A key in the global or system file does not count: `git lfs install` writes `filter.lfs.*` there for every
  repository of the account, and the filter's first run in a repository writes `lfs.repositoryformatversion` to its
  local file.
  The decision costs no extra process on the commit path:
  the configuration snapshot the commit already reads is searched in memory.
  A forwarded `git push` reads the configuration once (one `git config --list` start) before it runs;
  every other forwarded command is unchanged.
  When the configuration cannot be read, the upload is attempted.
- Mechanism.
  The wrapper first decides in process which LFS objects the pushed commits introduce (`lfs_object_plan.rs`,
  `lfs_pointer_scan.rs`, `lfs_pointer.rs`, on the in-process readers of `repo_read_*.rs`), and starts `git-lfs` only
  when there is something to upload.
  The tips are the plan's refs (a branch or tag name, `HEAD`, a full ref name or an object name; a revision expression
  is a decline). The commits to read are those reachable from the tips and not from the remote's tracking refs
  (`refs/remotes/<remote>/*`), found by a walk ordered by committer time as `git rev-list` orders it, so the history
  behind a known tip is never read; when the destination is untrusted (see below) nothing is known and every commit
  of the pushed refs is read. Each commit is compared with each of its parents, descending only into trees that
  differ, so a pointer that one commit adds and a later commit removes is still found. A blob is a pointer exactly
  when `git-lfs` 3.8.1 would take it for one (`lfs/pointer.go` `DecodePointer`, `lfs/scanner.go` `BlobSizeCutoff`,
  `commands/uploader.go` `prepareUpload`): under 1024 bytes, a header word `git-media`, `hawser` or `git-lfs`, the lines
  `version`, `oid sha256:<64 hex>`, `size <n>` in order with a known version, size above zero; the path and
  `.gitattributes` are not consulted, and a pointer with extension lines is a decline.
  No pointer found: no `git-lfs` process starts and no request reaches the LFS server. Pointers found, every one present
  in `.git/lfs/objects`: one `git-lfs push --object-id <destination> <oid>...` run per 500 objects. A missing local
  object, a repository the readers decline (replace refs, shallow, grafts, unusual environment variables or global
  options, a tree or ref in a spelling they do not read, a walk of more than 100000 commits or 3 seconds), leaves the
  next paragraph's `git-lfs` scan in charge, which also words a missing object and honours `lfs.allowincompletepush`.
  The `--object-id` form does not verify locks, as the scan form does.
  The scan form runs `git-lfs push [--all] <destination> [<ref>...]`, as one child the wrapper owns per destination,
  before it starts the `git push`; the push carries no hook and no injected `-c`.
  The wrapper never starts its own executable, never writes shell text or scripts, and explains a failure from the
  parent process that owns the child (`doc/planning/cli-git-no-self-invocation.md`).
  `git lfs push <remote> <ref>` alone skips every object that a remote-tracking ref of the remote mentions, so a push
  that adds no LFS object sends no batch request. Those refs need not describe the destination being pushed to (a fetch
  from one server and a push to another, a changed push URL), which would publish pointers without objects, so `--all`
  (everything reachable from exactly those refs, a superset and idempotent as objects are content addressed) is kept
  whenever the configuration the plan already holds says the tracking refs may not describe the destination: it is not
  a configured remote (a URL, a path, a name nobody configured), the remote has any `pushurl`, any
  `url.<base>.pushInsteadOf` rewrite exists, or the push is a mirror push. A configured remote with no tracking refs
  needs no check: the bounded form then has nothing to skip and scans the whole refs itself.
  Measured with git-lfs 3.8.1 (`commands/uploader.go`, `uploadForRefUpdates`, `uploadRangeOrAll`): without `--all` the
  scan is bounded by the remote's tracking refs (`ScanMultiRangeToRemote`, `rev-list --not --remotes=<remote>`); with
  it the whole ref is scanned (`ScanRefWithDeleted`). Both forms build the same scanner (`NewGitScannerForPush`), which
  runs `git show-ref` and `git ls-remote --heads` against the remote (`calcSkippedRefs`), so dropping `--all` adds no
  network round trip and no Git start. No tighter bound exists without one: `git lfs push` takes no remote object name,
  and the `exclude` list of `uploadForRefUpdates` is empty on this path.
  The child runs with the real Git's directory first on `PATH`, so a `git` that `git-lfs` starts is the real Git and not
  the wrapper, but the `git-lfs` it starts is the first one the caller's own `PATH` finds (as an absolute path, so a relative `PATH` entry and
  `-C` cannot make the file that was checked differ from the file that runs). It runs in the directory a `-C`
  option selects and receives the global options as Git would pass them: `-c` pairs as `GIT_CONFIG_COUNT` variables
  after any pairs already set, `--git-dir`, `--work-tree` and `--namespace` as their variables. A signal sent to the
  wrapper is relayed to it.
  The child's standard error goes to a private anonymous file; the wrapper reads it back after each run and relays it
  with the user and password of every `scheme://userinfo@host` URL replaced by `<credentials>` (standard output stays
  the caller's). When that file cannot be made, the push is refused with the ways forward.
  A run that fails with a server error is run again at most twice, after 2 seconds and after 5 seconds
  (`RETRY_DELAYS` in `lfs_server_retry.rs`; `CLI_GIT_LFS_RETRY_DELAYS_MILLISECONDS` replaces them in tests). A server
  error is a line of `git-lfs` with its `Fatal error: ` prefix, which `lfshttp/errors.go` gives only to HTTP 5xx answers
  other than 501, 507 and 509 (`git-lfs` never repeats them; the text after the prefix is the server's own JSON `message`
  or `Server error: <url>` or `Server error <url> from HTTP 5xx`), with no line naming a refused credential, a
  missing or corrupt local object, a client error or locked files. Everything else, a signal and a stop by deadline
  included, ends the step at once.
- What is uploaded.
  The destinations and refs come from the push's own arguments, read with the same option table as the rest of the
  wrapper, and from the configuration Git would read for what the arguments leave out. The configuration entries
  (`branch.<name>.pushRemote` and `.remote`, `remote.pushDefault`, `remote.<name>.push`, `.pushurl`, `.mirror` and
  `.url`, `push.default`) come from the snapshot taken when the invocation was planned, so a typed push costs no Git
  start for them; the auto-push reads them with one `config --get-regexp`.
  The first positional is the repository whatever `--repo` says; `--repo` and its abbreviations fill in only when none is
  typed. Each refspec names its source: the part before the last colon (Git splits there, so a source such as
  `:/message` survives); a deletion names nothing, `tag <name>` is `refs/tags/<name>`, and the refspec `:` (also
  `+:` and a configured `push = :`) means every local branch.
  A source with a `*` is matched as a refspec matches (the star crosses `/`) against the refs one `for-each-ref` lists.
  `--all` and `--branches` upload the local branches, `--tags` the local tags, never every ref; `--mirror` and
  `remote.<name>.mirror` upload everything.
  A bare push (or a remote with no refspec) resolves its remote the way Git does (`branch.<name>.pushRemote`,
  `remote.pushDefault`, `branch.<name>.remote`, then `origin`) and uploads what `remote.<name>.push` names, else every
  local branch with `push.default=matching`, else the current branch from the full ref `HEAD` names (`HEAD` when it is
  not a branch).
  A remote with more than one `pushurl` is uploaded to once per URL, passing each URL, because each may keep its
  objects in its own place. A destination typed as a path (starting with `/`, `./` or `../`, or naming an existing
  file or directory that is not a configured remote) is passed as a percent-encoded `file://` URL, which `git-lfs`
  accepts where it refuses a bare path.
  A push that only deletes uploads nothing. The plan fails closed: a Git query it needs that does not finish refuses the
  push with an explanation, it never reads as "nothing to upload".
  A push with `--recurse-submodules=on-demand` or `only` uploads only this repository's refs and prints a notice that
  a submodule's LFS objects must be uploaded from the submodule, because Git pushes the submodule itself.
- The explanation.
  A push of a repository with LFS keys runs as a child of the wrapper, in every case (a push that is neither stackable
  nor eligible for integration included), so a parent exists to explain.
  Before anything starts, one stat per `PATH` directory finds `git-lfs`; when it is absent the wrapper prints the
  install-or-remove explanation and exits with status 1.
  If the upload exits non-zero or is ended by a signal, the wrapper prints, after the program's own output, an
  explanation that names the status and every way forward (fix the cause and push again, `git lfs push --all <remote>`
  by hand, `git push --no-verify`, install Git LFS, or remove the LFS configuration), does not start the push, and
  exits with the upload's status (1 when it had none).
  The auto-push runs the upload as a bounded child with captured output and appends the explanation to the recorded
  output of the failed attempt, so a commit that joined that push shows it too; a stop by deadline says so.
- Paths.
  A typed `git push` is preceded by the upload, on the exec path and on the path that integrates a lost race;
  the merge push the wrapper runs for a lost race is planned and run inside the same process (`plan_invocation` with
  the stacking marker set in the planner's environment), is preceded by its own upload (its refspec source is the merge
  object name) and the wrapper does not start itself.
  The auto-push after a landed commit is preceded by it, and so is each reconcile push of that auto-push.
  The scope probe of a manual push and the integration step's probe are dry-run pushes that must not upload:
  they never run the upload.
  No `git cli-git` subcommand pushes.
- Bypass.
  `git push --no-verify` (any accepted abbreviation), `--dry-run`, `GIT_LFS_SKIP_PUSH` set to a true value, and
  `-c hook.cli-git-lfs-pre-push.enabled=false` among the global options or in `GIT_CONFIG_COUNT` pairs (the switch an
  earlier version used for its hook) skip the upload and the pre-check. A repository or account configuration file
  is not read for that switch, since reading it would cost a Git start for every push; `GIT_LFS_SKIP_PUSH` accepts
  `1`, `t`, `true`, `y`, `yes` and `on`.
- Shims.
  `git lfs filter-process` and `git lfs clean` write the four shim files into the hooks directory whenever they run
  (`setupRepository` then `installHooks(false)`), so deleting the files does not last once an LFS path is cleaned by
  a Git that the wrapper did not start.
  A shim never runs through the wrapper (see "Real Git starts with traditional hooks off"), so there is no second
  upload; `git cli-git doctor` explains the shims apart from the unsupported hooks (see "Doctor").

Verified by `src/native/binary_lfs_pre_push_tests.rs`
(a stand-in `git-lfs` that records the arguments, the input and the origin's branch at the moment it ran)
and `src/native/binary_lfs_server_tests.rs`
(the real `git-lfs` against an LFS server written inline in the test).

### Direct check

Use worktree bytes selected by explicit pathspecs or `--all`.
Run applicable direct-check policies read-only.
Emit final findings to stdout.

### Direct fix

Use worktree bytes selected by explicit pathspecs or `--all`.
Apply eligible patches to private candidate state through whole-sequence convergence,
then atomically replace only changed selected and added worktree files.
Snapshot the complete real index before and after and fail if any index blob changes.
Hold no lock:
a landing by the commit service between the first snapshot and the final one changes the real index,
and the fix then fails with "the index changed while cli-git fix ran" after restoring every file it replaced,
so a concurrent landing can never be mistaken for a fix that touched the index.
Run the fix again.
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
A commit invocation can emit up to three batches (the traditional-hooks and external-hooks warnings, the policy run and the
post-commit policy run), each starting at `0`.

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
    | 'concurrent-commit/already-committed';
  readonly message: string;
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
  the change this commit captured and the commits that landed while it was prepared cannot be combined.
  The message names the conflicting paths in Git path byte order,
  the commit that landed first,
  and the file that keeps the typed message with the command that commits again from it
  (see "Retry after a moved branch").
- `concurrent-commit/already-committed`:
  the captured change merged onto the branch equals the branch,
  because a commit that landed meanwhile already holds it.

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
  | 'transaction-failed';

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
Nested exception stacks and arbitrary thrown values remain debug logs rather than schema fields.

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
- a `concurrent-commit` core finding exits `1`;
- a landing,
  replay,
  or lock failure that is not a core finding exits `2` with an engine-failure event;
- a landed commit exits `0` unless post-commit policies block,
  which returns `2` as above.

## Commit protocol

The design, its measurements and the decisions behind it are in `doc/planning/cli-git-resident-service.md`.
This section states what a commit does.

### Scope and phases

Several `git commit` invocations against the same worktree and branch run at the same time,
and each lands as its own sequential commit.
The behavior is on in every repository the wrapper runs in;
no configuration key or environment variable opts out of it.

Every non-dry-run commit command line is landed by the resident commit service of the worktree:
one instance of the systemd template units `cli-git@.socket` and `cli-git@.service` per worktree,
started by the operating system on the first connection (socket activation).
The wrapper never starts the service itself,
and it has no other way to land a commit.
The wrapper forwards native `git commit` only for dry runs and short-circuit forms.

Each commit passes through three phases:

1.  Preparation:
    in the wrapper,
    in parallel with other commits and without the real index lock,
    capture the invocation facts,
    build a private index for the selection,
    run the pre-forward policies over it until they settle,
    and make the commit.
    The wrapper builds it from the command line (message and identity options,
    sign-off,
    trailers,
    a reused or fixup message,
    `-a`,
    an index commit,
    `--amend`).
    Every other command line,
    and every commit that is not on an ordinary branch with an ordinary state,
    is built by real Git in a scratch repository inside the commit's own scratch directory
    (editor,
    signing,
    `-F -`,
    `--include`,
    `--patch`,
    `--interactive`,
    `--pathspec-from-file`,
    conclusions of a merge,
    cherry-pick or revert,
    detached and unborn heads,
    and repository configuration that Git must read to build the message),
    and the service lands the result as Git made it.
2.  Landing:
    in the service,
    one step at a time per worktree.
    The service creates Git's own `index.lock` exclusively,
    advances the target ref by compare-and-swap from the tip the wrapper read,
    builds the new index beside the real one,
    and installs it by renaming the lock over `index`.
    A foreign holder of `index.lock` is waited for on an event watch of the Git directory,
    bounded by `CLI_GIT_LOCK_WAIT_SECONDS` (thirty minutes by default)
    with the holder's line repeated at the interval of `CLI_GIT_WAIT_NOTICE_SECONDS`;
    a holder that outlasts the bound ends the commit before anything moved,
    with a plain message naming the process.
    Both variables belong in the environment of the unit that runs the service.
3.  After landing:
    in the wrapper,
    print the one summary line,
    install the corrections a policy made into the worktree,
    run the post-commit policies against the landed commit,
    and,
    when that gate passes,
    hand the push and the upkeep to the service as work items.
    A blocked gate keeps the commit local and exits `2`.

No lock of the wrapper's own exists.
The only lock a commit meets is Git's `index.lock`,
and the only wait is the service's bounded wait for a foreign holder
and a forwarded index writer's event-driven wait for the service's own hold on the index.
A forwarded index writer that finds a foreign `index.lock` ends as native Git ends it.

### The service's hold on the index lock

`src/native/service_index_lock.rs` places `index.lock` with the service's marker in the file from the first instant.
Its hold,
ownership check,
staging files,
marker,
watch,
placement and attempts are in `service_index_marker.rs`,
`service_index_watch.rs`,
`service_index_placement.rs` and `service_index_attempts.rs`,
re-exported under the old names.
The marker names the batch,
the holding process and that process's birth identity.
A lock is reclaimed only when its marker names a process that is gone or was born again since.

#### One marker-writing taker at a time

The reclaim steps (read the marker,
unlink the lock,
place a new one) are not one atomic step,
and no lock of the service's own makes them one.
They are safe because at most one process that writes this marker takes a given `index.lock` at any moment.
Three things enforce that:

1.  Inside one service,
    `acquire` (a landing,
    `land_batch_announcing`),
    and `try_acquire` and `remove_stale` (the recovery of open batches,
    `recover_selected`),
    are called only from the serve loop and,
    before it starts,
    from `service_main.rs`.
    Connection threads only queue jobs,
    and the work thread (push and upkeep) never touches an index.
    The calls are sequential on that one thread.
2.  Across processes of one worktree,
    the worktree has one systemd unit,
    `cli-git@<instance>.service`,
    whose socket does not accept per connection.
    The manager runs at most one main process of that unit and starts it again only after the earlier one is gone.
    An upgrade sends the old release a retire,
    which it handles between batches,
    and the next connection starts the new release under the same unit name only after the old one exited.
    The result is a kill and a restart (a dead holder,
    which the marker handles) and never a live overlap.
3.  The single service of the releases before one service per worktree is asked to retire,
    stopped,
    and waited for until inactive,
    before the first instance starts (`retire_earlier_service`,
    `service_provision.rs`).

The commit route of the wrapper that took a marked lock of its own (transactions) was deleted with task 4d part 2,
and no other code writes the marker.
The tests that start `cli-git-service` directly start one service for each Git directory
(`binary_support.rs`,
`serve`,
keeps one child for each;
`binary_service_lane_tests.rs` starts one for each worktree),
and the tests that kill a service wait for its exit before they start the next one on that repository
(`binary_service_tests.rs`,
`ended_at_fault` and `stop`).
The test that runs an earlier release's single service beside a worktree's instance retires it first.

Where the invariant does not hold,
two things follow and are recorded here as limits,
not fixed:

- A downgrade to a release with one service for every worktree starts that service beside the per-worktree instances
  (`README.md`,
  "The commit service").
  Its holders write the marker without a holder (the format of those releases),
  which this release reads as a dead
  instance's,
  so a new instance can reclaim a lock that the old service still holds.
  The old service releases by pathname and cannot be changed.
  The cure is the documented one:
  stop `cli-git@*` before committing with the older release.
- A user who points two worktrees at one index with `GIT_INDEX_FILE` makes two instances take one lock.
  That is not a path the service ships.

#### A violation fails loudly

A holder keeps the staging file it made the lock from open for the whole hold
(`service_index_hold.rs`,
`service_index_ownership.rs`).
`IndexLock::release_with` and the drop of an unreleased `IndexLock` first ask whether the path still names that file
(same device and inode,
read through the open descriptor) and still holds exactly the marker it placed.
There are three answers,
and each has its own consequence:

- Owned:
  the index is installed and the lock removed,
  or the lock is removed.
- Replaced (a person deleted it,
  another taker placed one,
  or a reclaim by a taker that was paused):
  the lock is not removed,
  the index is not installed,
  the release fails with an error naming the lock,
  the service's log says so,
  and the batch stays open for the recovery that follows.
  The inode comparison also tells a replacement that carries the same text
  (the same batch taken again by the same process) from the holder's own file.
- Could not be checked (`EMFILE` and the like):
  the check is asked again a few times.
  If it still fails the lock stays in place with this process's name in it,
  the release fails (or the drop logs) and the lock is counted as unsettled.
  The serve loop ends the service after that batch,
  and the restart reclaims the lock because its marker then names a process that is gone.

The check is a preflight,
not a guard:
a replacement after it is not seen,
and the unlink that follows is by path.
The lifecycle that lets one marker-writing taker at a time take the lock stays the requirement.

#### Placement and its refusals

The marked file is first created under a name that has never existed
(`index.lock.cli-git-marker-<pid>.<pid namespace>-<batch>-<uuid>`),
with exclusive creation,
so no attempt ever opens or empties a file that an earlier attempt,
or an alias of a lock,
left.
It then becomes the lock in one step:
a rename that does not replace (`RENAME_NOREPLACE`),
else a hard link whose staging name is removed after.
A filesystem that offers neither (a vfat mount on a kernel without the rename flag,
for example) gets no lock:
the attempt fails with the code `unavailable`,
nothing stays in the directory,
and the service's log names the filesystem,
both refusals and what to do.
Creating the lock and writing the marker afterwards is not done,
because a kill between the two leaves an empty lock that may be native Git mid-write,
which no restart can take over.
A process that cannot read its own birth identity writes no marker and takes no lock,
for the same reason:
a marker without an identity reads as a dead instance's.

A staging file left by a killed attempt is removed by the next successful acquisition
when the process in its name is gone.
Its content is never consulted,
because a writer that has created the file and not yet written it leaves an empty file.
A name that carries no process is left alone,
except the fixed name of the earliest releases.

The marked file is flushed to storage (`fsync`) before it becomes the lock,
so that after a power loss the lock exists with its whole marker or not at all.
Measured on the commit path's lock step (btrfs,
host,
300 alternating cycles three times,
100% of one CPU):
create,
write and place take a median of 0.06 ms without the flush and 6.1 ms with it (95th percentile 7.2 ms).
The batch's journal write already costs a file and a directory flush,
so the lock adds one more of the same size.
In a container whose root is overlayfs the flush was not measurable (0.07 ms either way).
A lock from an earlier boot is also reclaimed by its marker (below),
but that covers only a marker that was written whole;
an empty or partial marker can be told from Git's own lock only by its content,
and the age of a file cannot be trusted across a clock step.

A stale lock that cannot be unlinked (an immutable attribute,
a directory that changed permissions)
ends the acquisition at once with `unavailable`
and a log line naming the lock and the error,
because waiting would not remove it.
The watch on the Git directory reports the end of the watch (the directory unmounted or removed)
as a wake-up,
and the wait sets a new watch before it waits again.
The wrapped index writer's wait for the service's hold (`service_index_wait.rs`) does the same.
An ended watch is dropped before the next is set,
because the new one may need the descriptor and the inotify quota the old one holds.
A directory renamed aside (`IN_MOVE_SELF`) ends the watch too.

#### Who holds a lock the marker names

The identity a new marker carries is the process's start tick,
then `#`,
then the boot ID (`/proc/sys/kernel/random/boot_id`),
then `#`,
then the PID namespace (the number in `/proc/self/ns/pid`).
A marker an earlier release wrote has no boot ID or no namespace and is judged by the parts it has.
On Linux a process that cannot read its boot ID or its PID namespace takes no lock,
and says so in the log,
because a marker without them cannot be told from a reboot's collision or from a number of another namespace.
A process also takes no lock when its `/proc` numbers it differently from the operating system
(a service that is PID 1 of a new PID namespace with an ancestor's `/proc` would read the ancestor's init):
the PID in `/proc/self/stat` must equal `getpid()`.
That record is read as bytes,
because the name of a process may hold any byte.

A reader answers one of three things about the holder a marker names:

- Gone:
  the boot is not the current one (every process died with that boot,
  whatever the reader's numbering),
  or no process runs under that number,
  or the process under it was born at another tick,
  or the marker names no holder (an earlier release's).
- Running:
  the same boot,
  the same namespace,
  and the process there is the one born then.
- Unknown:
  the marker's PID namespace is not the reader's,
  the reader's `/proc` does not agree with its own PID,
  the reader has no boot ID to compare with a marker that names one,
  or the process record cannot be read.
  Unknown keeps the lock and proves nothing.
  A wrapped command then falls back to the age of the file,
  and the service's acquisition waits for the lock like any other holder's until its limit.
  A service that was killed in one PID namespace and restarted in another leaves a lock no reader there can reclaim:
  remove it by hand,
  or restart the service in the namespace it ran in.

The wrapped commands judge a hold by its holder,
not by the age of the lock file:

- A mark that names a holder that runs now is waited for however old the file is.
- A mark that names a holder that is gone is not waited for.
  The command returns at once and says the service that held the index no longer runs and that the next commit made through the service removes the lock.
- A mark that names no holder (an earlier release's) or a holder the reader cannot ask about falls back to the age of the file,
  and the lock file's time is set to the time of the placement,
  not of the staging,
  so a hold placed after a long wait is not old.

#### Removing the lock and what is made durable

The marker of the next batch is made ahead.
When a batch has been landed and answered,
the service makes the marker of the batch after it,
which is numbered one more and lands on the same index:
it creates the file,
writes the marker and flushes it to storage,
on a thread of its own while the service is idle.
The next commit then only places that file (a rename that does not replace),
so the flush is not on its path.
The file is the service's one:
it is kept for the service thread,
named with this process and the batch number,
and replaced or removed (waiting for a flush that is still running) when the batch or the index is not the one it was made for.
It is removed when the service ends,
whichever way its loop ends,
and what a killed service leaves is removed by the next acquisition,
by these rules:
a whole marker decides alone,
and its verdict vetoes the name
(a holder that runs,
or that the cleaner cannot compare with,
keeps its file;
a holder that is gone loses it);
without a whole marker the name decides,
and only when it carries this cleaner's PID namespace
and the cleaner's `/proc` numbers it as the system does,
and the process it names is gone.
A name without a namespace (an earlier release's) or with another one,
and an unreadable `/proc`,
are no evidence of death.
An empty or partial file is never removed on its content.
A marker is a whole line:
the text up to its newline,
so a marker cut at any point reads as no marker,
from the text and from a file.
The file's pathname is checked before it is placed:
it must still name a regular file (not a link) with the device and inode of the descriptor the service holds.
A pathname that names another file is that file's,
and is not removed.
A commit that arrives while the flush is still running waits for it and starts no second.
The file the first commit of a service needs is made by that commit,
because the service learns the index only from a request.
`lock_stage_ms` is what a commit waited for in this step,
measured over the whole of it:
near zero when the marker was ready,
the rest of the flush when it was not,
a wait for a flush that failed or was for another batch plus the cold staging that followed,
the whole creation and flush when it staged its own;
`lock_stage_kind` says `pre` or `cold`.

A lock the holder owns is removed by the release after the index is installed,
or when the holder goes out of scope.
A removal that fails for any reason other than the lock being gone is said in the log and counted as unsettled,
like a check that cannot be made:
the lock names this live process,
so no later acquisition would reclaim it,
and the service ends after that batch so that the restart does.
The count is kept for the thread that holds locks (the serve loop and the recovery before it);
a lock held and dropped on another thread would not be seen by it,
and none is.

The release does not sync the directory after the unlink.
Measured (btrfs,
host,
300 alternating cycles twice,
100% of one CPU):
the removal takes 0.012 ms,
and with a directory sync 2.1 ms at the median (95th percentile 2.4 to 2.9 ms,
one cycle of 81 ms);
on the gate image's overlayfs the sync was not measurable.
Native Git syncs no directory when it releases its own lock,
and the loss it would prevent is bounded:
after a power loss the lock can come back with the marker of the boot that ended,
which every reader reads as gone,
so the next acquisition reclaims it and a wrapped command says the service no longer runs.

The built index is written by Git's own `reset` under the private index file (`service_land_follow.rs`,
`build_index`),
so Git's `core.fsync` governs it,
and the rename that installs it has the durability native Git's has,
by default and when the user opts in.
The index can be rebuilt from `HEAD`,
and the service syncs the commits and refs.

The watch on the lock's directory ends when the directory itself is renamed or removed.
A rename of an ancestor directory is not seen:
the watch stays on the old tree and the wait notices at its next timer.
That is a bounded delay and not a hang;
the wait requires the Git directory's ancestry to stay in place while it lasts.

### Machines with no way to reach the service

A commit on a machine where no socket answers and no user service manager can be asked to start one
(no runtime directory,
no session bus,
no service program beside the wrapper)
changes nothing and ends with exit `1` and a plain message for the platform:
Linux without user services names `loginctl enable-linger` and the requirement of a systemd user manager,
and Windows,
BSD and other systems say that committing through the wrapper is not supported there yet.
Run outside a repository,
a commit ends with Git's own words and exit status for that question.

### Invocation capture

At invocation,
before any Git mutation,
the wrapper records:
the repository root,
the worktree's Git directory and common directory,
the real index path and the object directory,
the ref storage format,
the symbolic `HEAD` and its target,
the conclusion in progress (`MERGE_HEAD`,
`CHERRY_PICK_HEAD`,
`REVERT_HEAD`,
or none),
the preparation base (the target's commit,
or none for an unborn branch),
and the time of invocation.
One configuration listing read at that moment serves every later decision of the commit.
A worktree whose `HEAD` leaves the recorded branch before the landing fails the commit with a plain message
and the message the person typed saved in a file.

### Private preparation

Preparation never mutates the real index,
the worktree,
the target ref,
any other real ref,
or shared reflogs.
It uses:

- a private commit index at `<git-dir>/cli-git-service-<id>/commit.index`,
  in the commit's scratch directory,
  which the service removes when the command ends,
  however it ends;
- the real object store.
  Every cli-git Git command that writes or reads the private index's objects
  (`read-tree`,
  `add`,
  `add --patch`,
  `apply --cached --3way`,
  `write-tree`,
  `diff --cached`,
  and the `cat-file --batch` candidate reads)
  runs in the owning worktree with `GIT_INDEX_FILE` naming the private index,
  and what it writes is an unreferenced loose object until the service moves the branch to a commit that names it.
  A `git gc --prune=now` between preparation and landing can remove such an object:
  the service then answers that the content is unavailable,
  the wrapper prepares once more,
  and a second loss ends the commit with a plain message that says what happened.

When real Git builds the commit,
it runs in the scratch repository after policy evaluation settles,
with inherited stdio,
as:

```text
git <caller global options> --git-dir=<shadow>
    -c core.hooksPath=/dev/null/cli-git-no-hooks
    commit <private commit args>
```

with `GIT_INDEX_FILE=<scratch directory>/commit.index`.
`--work-tree` names the same absolute worktree root as the shadow's `core.worktree`;
the command-line value takes precedence for this process,
and `core.worktree` covers a Git process that a hook starts with the shadow as `GIT_DIR`
but without `GIT_WORK_TREE`.
The child environment drops a caller-set `GIT_DIR`,
`GIT_WORK_TREE`,
and `GIT_COMMON_DIR`.
The private commit args drop the user's `--git-dir` and `--work-tree`,
pathspecs,
and the internal `--only`,
because the private index is the complete intended tree.
Git therefore owns the editor,
templates,
message cleanup,
and signing.
Native `commit -a` updates only the private index copy.
No hook runs in the shadow,
so the built commit is the policy-settled intended tree.
The service lands the commit as Git made it:
its first parent must be the tip the wrapper read,
or for an amend the replaced commit's parents.
A preparation failure removes the scratch directory and leaves real index,
worktree,
and real ref bytes unchanged.
Do not invoke Git with a lock path as `GIT_INDEX_FILE`.

#### Scratch repository layout

The scratch repository (the shadow, in the code and in this section) is made only when real Git must build the commit.
It lives in `<git-dir>/cli-git-service-<id>/shadow` and holds these private entries:

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
  an object store whose `objects/info/alternates` names the absolute real object directory
  resolved at invocation
  (`git rev-parse --git-path objects` in the owning worktree).
  Git is started with `GIT_OBJECT_DIRECTORY` naming that real directory,
  so what it writes lands in the real store.
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
  which is never linked.

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
; <git-dir>/cli-git-service-<id>/shadow/config
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
[hook "post-commit"]
  enabled = false
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
- `hook.post-commit.enabled` follows the include,
  so the native commit never runs `post-commit` in the shadow, and nothing runs it after landing.
  It is a file value, not a `-c` option, so no hook inherits it through `GIT_CONFIG_PARAMETERS`.

Cli-git writes the file itself,
quoting every path value under git-config value syntax at the final interpolation.

#### Private `HEAD` shape

A prototype in disposable repositories with Git 2.55.0 chose the shadow repository
("Private `HEAD` shape:
 shadow repository with alternates" in `doc/decision/cli-git-concurrent-commits.md`).
It ran a probe program that read the branch name,
the upstream,
and an `includeIf "onbranch:"` value,
and rejected commits to `main`.

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
- A prepared commit is an unreferenced loose object in the real object store until the service moves the branch to it;
  a `gc --prune=now` in between is answered at the landing (see "Private preparation").

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

#### External hooks

The wrapper supports no external hook program.
Traditional hooks,
executable scripts in `.git/hooks` or in a `core.hooksPath` directory,
are not supported,
and neither are hooks configured with `hook.<name>.event` and `hook.<name>.command`.
Nothing here promises their order, environment, locking or exit-status handling,
and the wrapper starts none of them.
Git never starts the wrapper for a hook:
the wrapper writes no hook entry,
passes no hooks path of a directory it made,
and the executable does not read the name it was started under.

- Every real-Git start of the wrapper carries `-c core.hooksPath=/dev/null/cli-git-no-hooks` and
  `-c hook.<event>.enabled=false` for every hook event Git 2.56.0 knows (`HOOK_EVENTS` in `hooks_off.rs`, checked
  against the host Git by a control), after the caller's global options and before the subcommand.
  `hook.<event>.enabled=false` switches off every configured hook of the event whatever its name
  (`doc/troubleshooting/git-hook-disable-switches.md`), and `core.hooksPath` switches off the hook files.
- The shadow's `config` switches every hook event off after its include, so no hook runs in the shadow whatever the
  real config defines, and the native commit repeats the hooks-off hooks path after the caller's global options.
- The native commit runs `git <caller global options> --git-dir=<shadow> -c core.hooksPath=/dev/null/cli-git-no-hooks
  commit ...` with no `--work-tree`: the shadow's `config` names the work tree.
  A caller's `-c core.worktree` cannot redirect it: with an explicit `--git-dir`, Git reads the work tree from the git
  directory's own config file only (measured on Git 2.56.0), and the wrapper removes `GIT_WORK_TREE` from the environment.
- A repository that configures hooks gets one `configuration-warning` event (code `external-hooks-unsupported`) on
  each commit, decided from the configuration snapshot the commit already holds: it names the `hook.<name>` keys, their
  events and the ways forward (remove the keys, or express what the hook checked as a cli-git policy under `policies`
  in `cli-git.config.jsonc`).
  `git cli-git doctor` lists the same hooks, one `hook.<name> (events: ...)` line each, and exits 1 while any remain.
  A push prints the same warning, from the snapshot it already reads for the LFS decision.
  A forwarded command other than a commit or a push prints no warning, because it reads no configuration snapshot.
- `hooks.concurrentCommits` is retired: an older file that sets it loads, the value is ignored, and doctor says so.

The wrapper itself executes no program that any configuration names: not a Git config key, not `cli-git.config.jsonc`,
not the git directory, not the environment.
The programs Git starts for its own features (filters, editors, signing programs, credential helpers) are Git's, not the
wrapper's, and stay, `core.fsmonitor` included; a program that is a hook event is switched off.
`cli-git.config.jsonc` is data: it has no command or executable field, and a `postCommit` key is a configuration error
(`Configuration key postCommit is not supported: cli-git runs no program named by any configuration ...`).
What a hook checked belongs in a cli-git policy.

### Policy evaluation during preparation

Preparation constructs candidate facts from the private index,
writes each policy's corrections into the private index,
and restarts the whole ordered policy sequence after exact candidate changes.
Only the final unchanged pass emits findings.
A commit prepared again after a moved branch runs every policy again
(see "Policy re-run after a retried landing").
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
3.  Build the commit:
    from the command line when the wrapper can,
    otherwise by native `git commit` run privately in the scratch repository with the copied index.

The service computes the new real index at landing,
beside the real one,
from the paths that differ between the old branch and the new commit.

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

Merge,
cherry-pick,
and revert conclusions use index-commit semantics only.

#### Added paths

A commit adds no paths of its own:
only the selected paths reach the landing,
and a rewrite a policy made is installed into the worktree by the wrapper after the landing.

Direct fix admits added paths under the precondition in "Finding and patch validation":
`HEAD`,
the real index,
and the worktree copy all hold the blob the correction was computed against.
It has no journal or index step:
converged bytes of added paths join the direct-fix worktree installation.
Precondition failures are `patch-conflict` with direct-fix remedies
(include the path in the pathspecs,
or restore it to `HEAD`).

### Sequencer conclusion state

A merge,
cherry-pick or revert conclusion,
and any commit made while that state exists,
is built by real Git in the scratch repository,
which holds a copy of the conclusion state of the worktree
(`MERGE_HEAD`,
`MERGE_MSG`,
`MERGE_MODE`,
`SQUASH_MSG`,
`AUTO_MERGE`,
`CHERRY_PICK_HEAD`,
`REVERT_HEAD`,
`MERGE_RR`,
and the `sequencer/` directory).
The landing removes the same state from the worktree exactly as native Git removed it in the scratch repository,
each entry only while it still holds the bytes copied at preparation,
and journals the cleanup with the batch.
A cleanup that fails cannot be redone once the wrapper has removed its scratch directory.

### Retry after a moved branch

A branch that moved since the wrapper read it makes the service answer `retry-prepare` with the commits that landed meanwhile.
A commit the wrapper built merges the change it captured onto the branch as it is
(`git merge-tree --write-tree`)
and runs the policies again over the merged tree;
no recorded policy run is reused.
Where a path changed in both,
the later capture wins when both captures read the same worktree
and every other path is settled by subsumption and then the three-way merge.
Paths that cannot be combined end the commit with the core finding `concurrent-commit/replay-conflict`,
exit `1`,
a message naming the paths and the commit that landed first,
and the message the person typed saved in a file with the command that reuses it.
A merged tree equal to the branch ends with `concurrent-commit/already-committed`.
A commit Git built without an editor is built again on the new tip,
signed again by Git,
in five builds at most (the first and four more);
one built after an editor keeps the typed message in a file and is built again from it within the same five;
an amend is never sent for another tip and fails with a plain message.
The retry is bounded;
a branch that keeps changing ends the commit with a plain message naming the branch
and the number of attempts it made:
"changed 5 times" for the commits Git built,
and "changed 3 times" for a commit the wrapper built (its `retry-prepare` rounds)
and for a batch the service builds again after its reference update finds the branch moved,
each three attempts at most.
The bounds are `REBUILD_ATTEMPTS` in `src/native/service_commit.rs`
and `LANDING_ATTEMPTS` in `src/native/service_land.rs`.

### Recovery

The service keeps a journal of each batch it lands and settles an unfinished batch when it starts again;
the wrapper recovers nothing for a commit.
What the wrapper still settles at the start of a command is state other code writes:
an isolated merge whose process died,
and it stops on a commit journal of the previous executable's format
(`.git/cli-git-transaction`)
with the way to recover it.
Commit records of an earlier release in `.git/cli-git-transactions`
are not read,
and the directory is no longer used:
the wrapper prints a note on standard error for every command until the directory is gone.
The note names the directory,
says that this release no longer uses it,
tells the person how to see whether that commit was made,
and gives the exact `rm --recursive --force -- '<directory>'` command that removes it,
with the path quoted for the shell.

### Leases of claimed work

The service takes a queued item by renaming its file into `claimed`,
and the file name carries the clock reading of the holder's last renewal
(`src/native/claim_time.rs`, `src/native/work_item.rs`).
The reading holds the machine ID,
the boot ID,
the monotonic clock in milliseconds,
and the wall clock in seconds,
written as `<boot>~<monotonic>~<wall>~<machine>`.
Whoever judges a claim takes a reading of its own and compares the two:

- Two readings with the same non-empty boot ID are compared by the monotonic clock,
  so a wall clock step in either direction changes nothing.
- Two readings with different non-empty boot IDs and the same non-empty machine ID are of a restart of one machine:
  the holder is dead and the claim has run out at once,
  whatever either wall clock shows.
- Every other pair is compared by the wall clock against the lease:
  a reading with no boot ID,
  a reading with no machine ID,
  and two readings of different machines.

The machine ID is the text of `/etc/machine-id`,
or of `/var/lib/dbus/machine-id` when the first holds none,
read once per process.
A file that is missing,
empty,
or not 32 hexadecimal digits (a fresh image holds `uninitialized`) counts as no machine ID.
The boot ID alone cannot say the holder is dead,
because boot IDs are random per kernel boot and two hosts that drain one work directory on a shared filesystem
have unrelated ones:
each would otherwise take the other's live claims at once.

A reading with no machine ID leaves the fourth part out,
which is the file-name form of releases up to 0.5.9.
Such a claim decodes and is judged by its lease even by a judge that has a machine ID.
The cost is in the recovery of a claim that 0.5.9 or an earlier release left across a reboot of its host.
It is recovered when the judge's wall clock passes the claim's own wall timestamp plus the lease.
That is one lease after the restart only when the clock was not set back;
a wall clock set back at boot delays the recovery by the size of the step.
A claim file name with the fourth part is not read by 0.5.9 or an earlier release;
those releases age it by the file's modification time.
A rename keeps that time,
so every claim and every renewal sets it to now before it is visible in `claimed`,
and an older release sees the last renewal as it sees it in a three-part name.
A claim is taken into `recovering` under its claim name first,
where no older release looks and the entry is the taker's alone,
stamped there,
and then renamed into `claimed`.
Setting the time is not best effort:
a claim whose time cannot be set is not taken,
and a renewal whose time cannot be set renews nothing,
because an older release would read the stale time as an expired lease.
A claim that cannot be stamped or published goes back to `pending` with the modification time it was queued at,
so a queue that cannot be drained keeps its true age for the backlog notice.
A taker killed between the take and the publication leaves the entry in `recovering`,
which runs out its lease and is recovered like a claim.
The longest claim file name the code can make is 217 bytes
(kind `maintenance`, a 64-digit object name, a 32-digit generation,
a reading with 32-digit boot and machine IDs and two 20-digit numbers),
and 223 with the suffix of a recovery's temporary file,
under the 255 bytes a file name may have.
The bound holds for every claim because the queue claims only an id that is a kind and an object name of at most 64 letters and digits.
An entry of `pending` whose name starts with the name of a kind the consumer handles,
but is not a kind and such an object name,
is moved to `failed` with a line on standard error naming it;
if the move fails the entry stays in `pending`.
A name that starts with no handled kind is left alone.
A record whose id is not the id in its file name is moved to `failed` when it is claimed or recovered,
because running it would settle the wrong work under the name.
A claim by an earlier release whose id is not a kind and an object name is moved to `failed` by its own name when its lease is found out,
since this release's names for it would not fit.

Limits:
machines cloned with one machine ID cannot be told apart,
so their claims are judged as one machine's;
a sandbox that fakes the boot ID per process but shows the host's machine ID
makes its processes see each other's claims as of an earlier boot.

### Timing logs

A slow commit is split by phase from logs that exist without verbose mode.
Neither log reaches the person's terminal,
neither starts a process or flushes to disk,
and neither holds a message,
an identity or a secret.

The wrapper appends one JSON line per commit that reached the service to `<common-git-dir>/cli-git-commit-timing.jsonl`
with `schemaVersion`,
`time`
(when the line was written),
`level` (`warn` for `stopped`),
`startMillis`,
`pid`,
`release`,
`instance`,
`gitDir`,
`requestId`
(the request's identity as the service logs it,
null when the command made none),
`branch`,
`outcome`
(`landed`,
`ended` or `stopped`),
`exitCode`,
`totalMs`,
the disjoint phases
`prepareMs`,
`buildMs`,
`provisionMs`,
`submitMs`,
`afterLandingMs`,
`gateMs`,
`handoffMs` and `otherMs`
(the remainder,
never negative),
and within `submitMs` the reach (`reachMs`,
`connectTries`,
`refusedConnects`,
`startWaitMs`,
`updateRounds`,
`releasesSeen`,
`answeredByRelease`) and the wait for the answer (`answerWaitMs`,
`positions`,
`waitingNotices`,
`lockHolderKind`,
`reconnects`,
`sends`).
Durations come from the monotonic clock in whole milliseconds,
and the start stamp from the wall clock.
The file moves to `cli-git-commit-timing.jsonl.1` at one mebibyte,
and a failed write never changes the command's result.
All three JSONL files (timing,
push trigger,
audit) are written by one appender with one `write` per line and no lock.
It moves the file aside only when the path still names the file it opened,
so two writers that both see a full file cannot put the new small file over the history in `.1`,
and a file that ends without a newline (a write a full disk or a signal cut short) gets the next line on a fresh line,
so one torn line is lost alone.
A reader skips empty lines and lines that are not JSON.

The service's lines reach the unit's journal in one of two ways.
When its standard error is the journal
(`$JOURNAL_STREAM` names the device and inode of the standard error it has),
each line is one datagram on the journal's native socket (`/run/systemd/journal/socket`):
the kernel stamps a datagram when it is sent,
so the journal dates the line by its event and not by the moment it reads it (a stream line is dated by the read,
which a loaded machine delays by seconds).
The datagram has `MESSAGE` (the line without the prefix),
`PRIORITY` (`err` 3,
`warning` 4,
`info` 6),
`SYSLOG_IDENTIFIER=cli-git-service`,
`CLI_GIT_AT_MS`,
`CLI_GIT_RELEASE`,
`CLI_GIT_INSTANCE`
and, where the line has them,
`CLI_GIT_REQUEST_ID`,
`CLI_GIT_BRANCH`,
`CLI_GIT_OUTCOME`,
`CLI_GIT_BATCH` and `CLI_GIT_OID`.
A value is carried as written: one that holds a newline goes in the protocol's length-prefixed form
(`NAME`, a newline, the byte length as eight little-endian bytes, the value, a newline),
so it starts no field, and other control bytes are kept.
The line on standard error stays one line, with every control character replaced by a space.
A field name is at most 64 characters of upper-case letters, digits and underscores, and starts with a letter:
this is systemd's `journal_field_valid` (`src/libsystemd/sd-journal/journal-file.c`), which refuses a digit first,
a leading underscore (the journal's own fields) and a longer name, and journald drops such a field without a word,
so the datagram is built without it.
The socket is unconnected and non-blocking,
no lock is taken on that path,
and a line the socket does not take (the journal restarting, a full queue) is written to standard error,
prefixed `<priority>` when standard error is the journal's stream.
That write never waits:
on a socket it is `MSG_DONTWAIT`,
on a pipe or file a test or a redirection put there it happens only after a poll shows room,
in pieces of at most a page.
A line the stream took only in part is kept:
its rest is written first, before any other line, so two lines never run together.
A line that finds no room is dropped and counted;
the next entry that gets through says how many were dropped,
as `CLI_GIT_LOG_DROPPED` on a native entry or as a warning line on the stream.
Dropped lines are lost, not spooled: a lost line is better than a stuck command.
A partial line still unfinished when the process ends stays unfinished in the stream.
The signal handler sends its line the same way, with `sendto`,
and writes the fallback line to standard error through the same non-waiting send,
using only `send`, `poll` and `write`, which `signal-safety(7)` lists:
a standard error that is a full pipe or stream loses the line, and the service still ends by the signal.
Without `$JOURNAL_STREAM` (a terminal, a file, a test) the lines are plain on standard error.
Every line ends with `at_ms=<epoch milliseconds>`,
and the userinfo of any URL in it is replaced by `<credentials>`.
The lines are:
a `started` line before `ready`,
an `answered` line for every request with the phases of its stay
(`read_ms`,
`queued_ms`,
`settle_ms`,
`lock_wait_ms` and the holder when one is named,
the split of the lock wait into the flush of the marker (`lock_stage_ms`) and the wait for a holder (`lock_place_ms`),
and when the lock was found held `lock_looks`,
`lock_found` and `lock_last`,
`reads_ms`,
`objects_ms`,
`journal_ms`,
`ref_ms`,
`sync_ms`,
`answer_ms`,
`total_ms`),
a `pushed` line for every push the service ran
(`branch`,
`oid`,
`items`,
`outcome` `done`,
`failed` or `skipped`,
`push_ms`;
a failure is a `warning` and carries Git's words),
a line when automatic maintenance ran or a review finished,
a `warning` for each batch a restart settled that an earlier service left open,
a line for a retire request,
and an `exiting` line with the reason (`idle`,
`retire`,
`error` or `signal-<n>`) and the uptime.
Text a client chose appears only in Rust's quoted form,
so it cannot end the line or hide a field.
`README.md` ("Splitting a slow commit") gives the commands that read both logs.

### Compatibility and degradation

The executable requires real Git 2.55.0 or newer for commits.
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

### Verification

Concurrent mutation of shared Git state is verified only by running it concurrently against real repositories.
The binary tests of the package start the built wrapper against a service of their own per fixture
(its runtime directory under the agent scratch directory,
never the person's `XDG_RUNTIME_DIR`)
and compare each commit line with native Git field by field.
Phases that have no hook are reached through a test-only phase marker:
`CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill[:<directory>]` makes the wrapper `SIGKILL` itself at the phase,
and `<phase>:pause:<directory>` writes `<directory>/<phase>.reached` and waits for `<directory>/<phase>.release`.
The phases that are still reached are `service-staged`
(a commit has staged its content in the private index and has not yet read it back),
`service-prepared`
(a commit is fully prepared and not yet sent;
repeatable,
so its files carry the occurrence: `service-prepared-<n>.reached` and `service-prepared-<n>.release`),
and `merge-intent` and `merge-applied` of an isolated merge
(record durable and Git not yet started; branch moved and record not yet removed).
The phases of the earlier landing no longer exist,
and a value that names one is refused like any unknown phase.
Only this explicitly test-named variable arms a marker;
a malformed value fails the invocation.
The container end-to-end suite starts one service per scenario inside the container
(`e2e/README.md`),
replays commit-shape traces mined from this repository's history and a catalog of concurrent agent behavior,
seeded so that a failing seed replays,
and checks that each commit that exited `0` appears exactly once with exactly its captured bytes,
that no worktree edit is lost,
that the real index never stages a revert of landed content,
that no scratch directory,
lock or temporary file remains,
and that `git fsck` is clean.

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

#### Conflicted files during a merge, rebase, cherry-pick or revert

The owner's decision of 2026-11-02 (choice "save, then allow") is one exception to the rule above.
While a merge, a rebase of either backend, a cherry-pick or a revert is in progress,
a `checkout` or `restore` command line that takes a side of conflicted files is let through,
in the main worktree and in a linked worktree,
after the wrapper copies the current worktree file of each path.
In progress means `MERGE_HEAD`, `CHERRY_PICK_HEAD` or `REVERT_HEAD` in the worktree's Git directory,
a `rebase-merge` directory,
or a `rebase-apply` directory that holds `rebasing`
(with `applying` it is `git am`, whose conflicts are not covered).
A conflicted path is a path with an unmerged index entry (stage 1, 2 or 3),
read from the index file in process: no Git process runs for the decision.
An index that cannot be read in process (split, sparse, unreadable, or chosen by `GIT_INDEX_FILE`)
shows no path as conflicted, and the usual rejection stays.

The forms covered are:

- `checkout [<tree-ish>] -- <path>...`, with `--ours`, `--theirs`, `-2`, `-3`, `-m`, `--merge`, `--conflict` and the quiet,
  progress and overlay options.
- `restore -- <path>...` with `--ours`, `--theirs`, `-m`, `--merge`, `--conflict`, `-s` or `--source`,
  `-W` or `--worktree` (with `-S` only beside `-W`), and the quiet, progress, overlay and ignore options.
  Abbreviated `restore` options are not read and keep the rejection.

Every other line keeps the rejection: a force, a patch, a path list read from a file, a branch option, a form without `--`
before the paths, an index-only `restore --staged`, and any `reset`, `merge --abort` or other guarded command.
A path is read by its literal spelling below the working directory.
A directory, a trailing slash, a wildcard, pathspec magic or a path outside the worktree is not a conflicted file.
A line that names a path that is not conflicted is rejected whole, with the rejection text of the form
followed by a sentence that names every such path.
A path that Git resolves in the index as the command runs (`checkout <tree-ish> -- <path>` stages it) stops being conflicted,
so a later line that names it is judged by the rules again.

The copy is written and synced before Git starts.
It goes to `<common-git-dir>/cli-git/discarded/<UTC time as YYYY-MM-DDTHH-MM-SS>/<path from the top of the worktree>`,
next to `cli-git-audit.jsonl`;
a second invocation in the same second gets `-1`, `-2` and so on after the time.
The directories are private (`0700`), and the copy has the file's content and permission bits
or the symbolic link's target.
The wrapper prints, before Git runs, for each path:

```text
cli-git: saved your copy of a.rs to
  /path/to/repo/.git/cli-git/discarded/2026-11-02T14-01-07/a.rs
  before this command replaces it (a.rs was conflicted).
```

A conflicted path with no file in the worktree has nothing to copy:
the wrapper says so in one line and allows the command.
If a copy cannot be written (a full disk, a refused permission, a place that is not a directory, a file that is neither a
regular file nor a symbolic link),
the command is rejected with a finding that names the path or place and the system's reason,
no cut-short copy is left,
and nothing is changed.

The folder is bounded after each save that wrote at least one copy,
once the copy is complete and synced (so a kill while copying costs no older copy),
with nothing waiting on it.
The folder is opened once without following a link at `cli-git` or `discarded`
(a link there, or a non-directory, keeps every copy and is said in the notice and by the doctor),
and everything is read and removed through that descriptor, never through a path walked again.
A non-blocking exclusive lock on the folder lets one pass run at a time; a pass that finds it held does nothing.
A directory is removed only when, opened by name without following a link,
it is still the directory the listing saw (device, inode and change time):
its files go through its descriptor, files of a directory before its subdirectories,
then the empty directory is removed by inode,
so a replacement under the same name is never touched.
A link inside a save is removed as a link and never entered; a tree deeper than 128 levels is kept.
The pass reads only directories whose names are a save time
(`YYYY-MM-DDTHH-MM-SS`, with `-<number>` when a second save fell in one second);
the age of a save is the time in its name, not a modification time,
a name must be a real calendar date and time (February 30 and second 60 are not names the wrapper makes),
and a name says only its second, so every age is counted from the end of that second.
It removes every save older than `discardedCopies.maxAgeDays` days (default 30),
then, while the folder's files total more than `discardedCopies.maxSize` bytes (default one gibibyte),
the oldest remaining save, until the total is within the limit or nothing is within reach.
A save younger than 24 hours (counted from the end of the second in its name) is out of reach of both limits,
and so is the save that was just written,
so a burst of saves is never cut by size and a save made with an old clock survives its own pass.
Both settings are keys of the `discardedCopies` section of `cli-git.config.jsonc`:
`maxAgeDays`, a safe integer of at least 1,
and `maxSize`, a safe integer of at least 1 (bytes) or a string of digits with an optional suffix `k`, `m` or `g`
(powers of 1024, any case).
A value that is not that is a configuration failure that names the key, as for every other key of the file
(exit `2` before any policy runs); zero never means "no limit" or "remove everything", and nothing is removed.
An unknown key under the section is refused with `maxAgeDays` and `maxSize` named.
Each removed save is followed by one audit line, written once the directory is gone:
`schemaVersion`, `time`, `level`, `timeMillis`, `release`, `pid`, `form` (`PruneDiscarded`), `outcome` (`pruned`),
`reason` (`age` or `size`), `limit` (days or bytes), `folderBytes` (the folder before the pass), `path`, `bytes`,
`copiesTotal` and `copies` (at most one hundred names).
A directory removed in part has a line whose `outcome` is `partial` and whose `copies` are the files that went.
Sizes and ages are computed in 128-bit integers, so no sum or difference wraps.
Nothing fails silently: a directory that cannot be read has no size to judge by and is kept and named,
a removal that fails stays and is named (as removed in part when some files went),
and a pass that cannot append to the audit file removes nothing and says so.
The sentences are in the notice of the command that saved the copy and in `git cli-git doctor`.
`git cli-git doctor` writes to standard error, when the folder holds any save,
its path, total bytes, number of copies and saves, the oldest save and the limits in force.

Every decision about a guarded form is appended as one JSON line to `<common-git-dir>/cli-git-audit.jsonl`
with `schemaVersion`, `timeMillis`, `pid`, `parentPid`, `worktree`, `form`, `outcome`
(`rejected`, `allowed`, `exempt`, `hatch` or `saved-conflict`), `dirty`, `cwd` and `arguments`.
A `saved-conflict` line also has `copies`, a list of `<path> => <place>` (`none` when there was nothing to copy).
The line also has `time` (the ISO form),
`level` (`warn` for `rejected`),
`release` and `gitDir` (the invocation's own Git directory, which `worktree` the kind word does not name).
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
Manual push runs a Git `git push --dry-run --porcelain` probe with every hook off and full object names
(`-c core.abbrev=no`), parses Git's own report of the updates (`manual_push_porcelain.rs`),
reads local objects the report does not name with one `git rev-parse`,
and validates the reported remote OIDs with `git ls-remote --refs` before policy evaluation;
a report that does not state the remote's value (a deletion, a refusal) takes it from `ls-remote` alone.
The probe adds `--no-quiet` to its copy of the options, because `-q` empties the report, and skips the line Git adds for
`-u`, `--set-upstream` and `push.autoSetupRemote`; any other line outside the report format stops the probe and is named.
A single destination's printed location is used as it is, which saves the query for the push URLs.
The probe starts no program of the wrapper's, writes no hook or script, and every Git child of it is bounded by the
query deadline.
When the probe's Git ends with a failure status and a message before it reports any destination, Git has refused the push
itself (a branch to delete that the remote lacks, a refspec naming nothing, an unknown remote or option): nothing could be
published, so the wrapper ends with Git's exit status and Git's standard error as Git wrote it, with no event and no hint.
A failure of the probe's own reading (Git cannot run, a failure without a message, a report outside the format, a remote that
changed during discovery) stays a `content-unavailable` engine failure and exits `2`.
That one code covers causes the person answers differently, so the failure carries its cause and the event's hint is the
cause's, not the code's: a remote that moved says to run the push again; a remote whose refs could not be read says to check
the network, the address and the credentials, and its message masks the userinfo of the address; Git that cannot start or
ends without a message says to run Git's own `--dry-run`; output the wrapper cannot read is reported as a defect with the
output of `git --version`; a pushed object that is not a commit says to push a commit; and only content missing from the
clone keeps the code's hint about fetching history. A message that already says what to do (a deadline with its setting, a
repository that cannot be located) carries no hint.
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

Every commit made through the wrapper must not take `5,000 ms` or more longer than the same commit made with native Git;
the bound holds for each commit, not for a percentile of them.
This is the only latency requirement on a commit;
no wrapped command has to finish within a fixed time.
The owner's words (2026-11-01):

> " 5-second bound on wrapped commands" - there is never a 5s bound on wrapped commands.
> The measure is always just commits vs native git should not take 5s longer.

The 5-second kill in the binary tests (`TIME_BOUND_SECONDS`) is a hang guard of the test harness,
not this requirement.
The `native:perf` task enforces both latency requirements:
the forwarded bound on the added p95 of each forwarded command,
and the commit bound on the maximum paired delta of each commit arm.

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

The harness that produced the older records in `perf/` (lifecycle, concurrent commits, manual push, forbidden strings
and the reserve sweeps) and its `perf:*` tasks no longer exist;
a new measurement of those scenarios needs a new harness.
The `native:perf` task (`bin/native-perf.ts`) exists and measures the release executable on forwarded commands and on
commits (explicit-path and service, unsigned and SSH-signed), as "Latency requirements" in the README describes;
its records are `perf/native-latency-<date>.json`.
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
(the earlier landing's migration of its objects),
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
the service's `index.lock` hold time,
lost races per commit,
and a paired serialized baseline.
Because every non-dry-run commit now prepares privately,
the lifecycle baseline is re-measured and stored as a new dated `perf/lifecycle-latency-<date>.json`.
Timing comparisons first measure the run-to-run band on one unchanged build.

Repeated wrapper commits in one repository measure a moving state,
not one fixed scenario:
before automatic maintenance ran after landing
(the earlier landing),
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
