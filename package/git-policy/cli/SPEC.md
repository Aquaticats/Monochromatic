# monochromatic-git-policy-cli policy platform implementation specification

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

The implementation is the native Rust executable `monochromatic-git-policy-cli` (`src/native/`),
 published as the crate
`monochromatic-git-policy-cli`.
The TypeScript implementation (authoring API,
 plugins,
 trust registry and the bundled MJS artifact) is retired.
Its last commit is `f4fc1fa213ece65672e4314983a75266837892f9`,
 and the sections that specified it were removed from
this file;
 they remain readable at that commit.

The wrapper is the only route this product offers:
there is no switch that forces a command or a repository through the native one,
and the owner answered a request for such a switch (2026-11-10) with
"No switch,
/usr/bin/git is it (Recommended)".
A person who needs one command to skip the wrapper's policies runs `/usr/bin/git` itself,
the bypass the install steps already name (`README.md`,
"Install").
A Git a person starts as `/usr/bin/git` is outside every policy,
every audit line and every wait of this file.

Local durability follows what the remote already holds.
The owner,
2026-11-10:
"We don't care about data protection on the local drive if it's pushed."
Correcting the coordinator's first reading,
the owner added the same day:
"I only say things that are already on remote doesn't deserve strong data protection against power outage and stuff."
So a sync (`fsync`,
`fdatasync`,
`syncfs`,
a btrfs sync ioctl,
or a call that forces a btrfs commit) is not needed when everything it protects is already on the remote,
such as the files a worktree add checks out from a pushed commit
or objects fetched from the remote.
For everything else
(an index that holds staged changes,
a person's commits and merges before their push lands,
build trees and caches,
and the wrapper's own records)
the owner handed the choice to the coordinator,
2026-11-10:
"I think you can decide yourself on which fsync to remove and not have to ask me.
Or do it with the advisor."
The coordinator's rule,
made with the advisor on 2026-11-10 and open to the owner's veto:

- The baseline is what native Git syncs for the same data.
  Git 2.56.0's default `core.fsync` syncs packs,
  pack indexes and the commit graph,
  and not loose objects,
  references or the index:
  `write-or-die.h:20-22` at tag `v2.56.0` defines `FSYNC_COMPONENTS_DEFAULT` as the objects and derived metadata
  without `FSYNC_COMPONENT_LOOSE_OBJECT`,
  and this machine sets no `core.fsync` (read 2026-11-10).
  A plain `git commit` here therefore makes no sync,
  and the wrapper's answer to a commit waits on no sync of its own.
- Where the wrapper moves,
  replaces or deletes a person's uncommitted bytes and native Git has no matching operation
  (the merge carry,
  the aside renames,
  the trash,
  the stage,
  subvolume conversion),
  the sync that makes the new copy durable before the old one goes stays,
  unless kernel source shows that a power cut cannot leave the old copy gone without the new one.
- Removing a sync needs a test that leaves the file it guarded empty,
  torn or absent,
  as a power cut can,
  and shows that the next command proceeds at once and loses nothing this rule protects.
  Kill tests cannot show it,
  because the page cache survives `SIGKILL`.
- Where moving a sync off the waiting path does not stop the waiting path stalling behind it,
  the coordinator chooses between removing the sync and batching it into one.

## Module seams

The executable has these external seams:

- the shadowing `git` executable;
- the repository-root configuration file `monochromatic-git-policy-cli.config.jsonc`;
- JSONL policy events.

It has internal seams for filesystem identity,
real-Git execution,
clock,
process liveness and birth identity,
lock-holder evidence,
and prompts.
Production adapters own operating-system effects.
Disposable tests replace those adapters without production environment overrides.

## Other processes changing files

No check or lock the wrapper takes excludes another program from changing the same files in the same moment,
and the wrapper builds nothing further against it:
a rename or a replacement of a path,
a write through an open file or a shared mapping,
and an edit between a check and the step that relies on it are accepted risk in every command and in the commit service.
The owner's decision of 2026-11-03T00:15:33-05:00 is that a race another program causes is not the wrapper's to close,
and that using a command or the program means accepting the risk;
the wrapper states the risk plainly and keeps fixing its own
 deterministic bugs (`doc/handover/cli-git-rust-implementation.md`,
"Worktree add and package managers").

The cheap guards stay,
and each narrows the window without closing it:
a lock where one is free,
a second look before a removal,
an identity compared when an object is opened,
and a descriptor held on what a removal names.

`README.md` states the risk once for the whole program ("Other processes changing files").
A command that can change or delete files states it in its own `--help`,
where it has one,
and says it once on standard error;
a command whose own notice states it prints only that one,
never a second line.
The commands are:

- `git monochromatic-git-policy-cli migrate-to-subvolume`,
in a run,
a `--rollback` and a `--remove-old`:
the notice comes before the first step that writes,
keeps,
swaps or deletes anything (`MIGRATE_RISK_NOTICE`,
`src/native/subvolume/subvolume_arguments.rs`);
a `--dry-run` and a refusal change nothing,
so neither says it.
- `git monochromatic-git-policy-cli convert-worktree-to-subvolume`,
in a conversion:
the notice comes before the first worktree is touched (`RISK_NOTICE`,
`src/native/worktree/worktree_convert_arguments.rs`);
a `--dry-run`,
an `--abandon` and a run that names no worktree say nothing.
- `git worktree remove`,
when the wrapper destroys the worktree's subvolume in one step:
the notice comes after the destroy,
which cannot be undone,
and that command's `--help` is Git's own (`DESTROY_NOTICE`,
`src/native/worktree/worktree_remove.rs`).
A removal the person forced words its own line (`DESTROY_FORCED_NOTICE` in the same module),
because nothing was checked for changes before that destroy.

The sections that describe those commands carry the same statement beside their guards:
"Linked-worktree ignored-state synchronization" for the leave-out and the trash of a snapshot,
and "Cheap worktree creation and removal on btrfs,
XFS and overlayfs" for the conversion,
the removal and the migration.

## Single executable and shipped policies

The release artifact is one executable,
 `monochromatic-git-policy-cli`.
Every policy is compiled into it;
 no configuration loads code,
 and there are no plugins.
The built-in policies run at their default severity without being listed.
The optional policies (`markdown/autofix`,
 `mono/forbidden-root-context`,
 `mono/dependent-version-bump`,
 `mono/catalog-specifiers` and
`security/forbidden-strings`) run only where `monochromatic-git-policy-cli.config.jsonc` lists them.
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
A candidate the engine would rewrite is reported,
 and the correction is proposed wherever the caller may apply patches:
 on a direct fix,
 and in the commit's own patch-enabled convergence,
 the same way `final-newline` corrects a commit;
 a read-only selection or a direct check reports the finding and proposes none.
A panic,
or 60 seconds in which the linter thread shows no sign of work,
ends that candidate's check,
never the command
(see "Git steps,
tools and checks end only on no progress");
the abandoned thread stays until the command ends,
and the command starts no second linter thread.
A stack overflow or an abort in the linter is not contained and ends the command,
so the guard is a hang and
panic boundary and not a crash boundary.
A run whose fixed source passes 64 MiB,
or whose findings pass 4 MiB,
is refused as unchecked.

The `exclude` option is gitignore syntax,
read by the wrapper's own matcher and again by the linked linter,
and both compile every pattern through `monochromatic_lint::gitignore_pattern::for_matcher`
(`package/linter/monochromatic-lint/src/gitignore_pattern.rs`):
a brace is ordinary text to Git,
while the glob compiler beneath the matcher reads `{a,b}` as an alternate group,
so a literal brace is escaped for that compiler and a brace inside a character class is left as it is.
A control beside the matcher asks native Git's `git check-ignore` the same questions for every pattern
shape it covers.
A character range that runs backwards (`[z-a]`) keeps only the character that opens it,
which is what Git matches there,
and the glob compiler refuses the range itself.
A pattern the compiler still refuses,
a trailing escape,
fails the policy with a message naming the pattern.

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
the native engine already holds the corrected bytes,
so it writes them directly.
A commit transaction stores them with `git hash-object -w --stdin` and points the private index entry at the blob with
`git update-index --add --cacheinfo`;
a direct fix writes the worktree file.
The crate has no patch parser or applier for policy corrections.
A correction that the private index cannot take is a failure of the pass and names its path.
The engine does not compare a correction's bytes with the candidate's:
a pass counts as changed only when its resulting state differs from the preceding state
(see "Whole-sequence fixing").

A correction for a tracked file that the lifecycle did not select (an added path) is valid only when `HEAD` holds the file as an ordinary file with the blob the correction was computed against,
the real index holds that same blob with `HEAD`'s mode,
the lifecycle's own index holds that same entry,
and the worktree copy is a regular file whose bytes and executable bit match that blob.
A direct fix applies such a correction to the worktree file and refuses one that fails the precondition:
the engine exits `2` with `patch-conflict`,
naming the path,
which state differs,
and the remedies:
include the path in the fix pathspecs so the fix applies to its worktree copy,
or restore it to match `HEAD` (`git restore --staged --worktree -- <path>`).
A commit adds no paths of its own:
only the paths it selected reach the landing,
with one exception,
named by the owner on 2026-11-10 in the question tool as the answer
"A commit adds the dependents' manifests and updates them in the worktree when that copy still matches `HEAD`",
to the question recorded in
`doc/handover/questions-for-the-owner.md`,
and decided on 2026-09-15
(`doc/decision/private-npm-registry.md:158-177`):
a hand bump of a workspace manifest carries its dependents' patch bumps in the same commit,
and every one of those manifests is a path the commit adds
(see "Dependent version bump").
That exception is the only added path a commit takes,
and it keeps the precondition above rather than the fix's refusal:
a dependent whose copies hold anything else is left out of the commit and named in a note,
so the commit lands with the rest of the ripple instead of failing.

## Configuration validation

The wrapper reads `monochromatic-git-policy-cli.config.jsonc` at the repository top level.
The file is JSON with comments,
 at most 1 MiB,
 and data only:
 it never names code to run.
Without the file the defaults apply.
Legacy executable configuration files (`cli-git.config.mjs`,
 `cli-git.config.ts`) are never read or executed:
beside a JSONC file `git monochromatic-git-policy-cli check` reports a `legacy-config-ignored` configuration warning (other commands
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
     `{"path": ..., "optional": true}` objects,
    with no path named twice and every path checked as before.
6.  Validate the concurrency keys (see "Concurrency configuration").
7.  Resolve the rules files of `security/forbidden-strings`:
     `rulesFile`,
     else the environment variable
    `FORBIDDEN_STRINGS_RULES`,
     else `forbidden-strings.local.txt` at the repository top level,
     which may be absent.
    The variable is the scanner's comma-separated rules list,
    where a trailing `?` marks an optional entry.
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
An older file that sets one still loads,
and the value is checked and ignored.

- `hooks.concurrentCommits`:
  boolean,
  default `false`.
  It tuned the hook lock,
  and the wrapper runs no external hook,
  so no lock exists.
  `git monochromatic-git-policy-cli doctor` says so.
- `indexLock.unprovenOwnerTimeoutMs`:
  non-negative safe integer,
  default `1000`.
  It was the backoff budget of the wrapper's own wait for a foreign `index.lock` whose owner is dead or unproven.
  The service's wait for a foreign `index.lock` replaced it:
  a holder proven alive and an owner nothing proves alive are both
  waited for until the lock ceiling ("Every wait has a bound",
  and a descriptor whose record names the lock and which the scan could not identify keeps a PID file's absent number from
  declaring the owner dead:
  that holder may be alive,
  so the wait is the one a live holder gets).
  except an empty lock with no marker,
  no PID file and no holder a complete scan finds,
  which the service removes (owner,
  2026-11-07,
  "Service removes it").
  The window of item 3 is open here too:
  a holder that puts another file on the descriptor between the gate and the read is read as not holding the lock,
  so the removal may take a lock that is in fact held,
  and the journal line says what it acted on.
  The proof counts the service's own descriptors as the files the kernel has open under their numbers:
  `statx(2)` with `AT_EMPTY_PATH` takes the descriptor's own number,
  walks no path,
  and asks no file system to revalidate a directory entry,
  so the count is the same on every mount type
  (`src/native/process/open_file_of_a_descriptor.rs`,
  `opened_file_of_our_own`).
  A descriptor of its own that names no file of a repository matches no device and inode and is not counted,
  so a pipe,
  a socket,
  a memory or device file and a file under `/proc` or `/sys` need no name test of their own.
  A descriptor that ended while it was being counted names nothing.
  A mount whose type this process cannot classify no longer makes that count unknown,
  which on a container's overlay filesystem it did for every descriptor:
  an abandoned lock stayed where it was and every commit behind it failed at its ceiling.
  and this key no longer sets anything.
  `git monochromatic-git-policy-cli doctor` names the key when a file sets it and says to remove it.
- `landing.reserveAfterLostRaces`:
  positive safe integer,
  default `1`.
  It was the number of lost landing races after which a transaction asked for a landing reservation.
  The service lands in arrival order and bounds the retries of a commit (see "Retry after a moved branch").
  `git monochromatic-git-policy-cli doctor` names the key when a file sets it and says to remove it.

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

Neither default rests on a measurement of growth yet.
On 2026-11-03 the folder existed in no repository on the development host and the audit file held no `saved-conflict`
or `pruned` line,
a window of about one day since the feature landed,
so the defaults stay at the coordinator's earlier values,
and the coordinator's choice among the options in `doc/planning/cli-git-shipped-limits-audit.md`
("Defaults for saved conflict copies",
open to the owner's veto) is pending.
A saved copy can be the only copy of a person's work (the worktree file as it was before a side replaced it),
which is why that choice is open.

The pass that applies them is described under "Conflicted files during a merge,
rebase,
cherry-pick or revert".

## Command facts and classification

`rawArgs` are the exact wrapper arguments after the script name.
`transformedArgs` contain fixed monochromatic-git-policy-cli transforms but no management-only or escape-hatch tokens.
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
Management dispatch occurs only when the parsed subcommand is exactly `monochromatic-git-policy-cli`.
A pathspec named `monochromatic-git-policy-cli` does not dispatch management.

## Management grammar

The accepted grammar is:

```text
git [<git-global-options>] monochromatic-git-policy-cli (--help | -h)
git [<git-global-options>] monochromatic-git-policy-cli check [--policy <id>]... --all
git [<git-global-options>] monochromatic-git-policy-cli check [--policy <id>]... -- <pathspec>...
git [<git-global-options>] monochromatic-git-policy-cli fix [--policy <id>]... --all
git [<git-global-options>] monochromatic-git-policy-cli fix [--policy <id>]... -- <pathspec>...
git [<git-global-options>] monochromatic-git-policy-cli doctor
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
- `doctor` takes no options,
  scope or policy;
  any extra token is a usage error with exit `2`.
- `check` and `fix` load config after grammar and scope validation.
- `fix` changes selected worktree files and verifies every real index blob is unchanged.

Direct `check` and `fix` emit policy events on stdout.
A configuration failure of `check` or `fix` emits one schema-version-one `engine-failure` event with code
`config-invalid` on stdout and exits `2`.
So does a `--policy` selection that names a policy whose effective severity is `off`,
 because that run would check nothing and exit `0` with no output.
The message names every such policy once,
 in selection order,
 with the `monochromatic-git-policy-cli.config.jsonc` key and the severity that turns it on
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
 or,
for a policy that runs for neither direct command,
 says that it runs only when the Git command it guards is forwarded.
A selection that mixes policies that run with ones that do not is refused for the ones that do not.
The escape `--no-enforce-<policy>` written for a selected policy is not refused.

### Doctor

`git monochromatic-git-policy-cli doctor` lists the traditional hooks a repository holds.
Traditional hooks are unsupported (see "Commit hooks"),
and this command is where they are found:
the commit path only warns when `core.hooksPath` is configured and never scans a directory.

- It asks real Git,
  with the caller's global options,
  for the common Git directory,
  for the Git directory itself
  (`git rev-parse --path-format=absolute --git-common-dir` and `--absolute-git-dir`),
  and for `core.hooksPath`
  (`git config --type=path --null --show-scope --get-all core.hooksPath`).
  Each pathname is asked for on its own,
  and each answer is read as bytes with only its one terminator removed,
  so a pathname that holds a line feed or bytes that are not UTF-8 is read exactly as Git printed it.
  The hooks path is every value of the key with its scope,
  NUL-delimited:
  a configured directory whose name holds a line feed is one value,
  and splitting the listing on line feeds read it as two and scanned a directory the person never named.
  The configured directory is the last value that is not one of the wrapper's own hooks-off
  directories:
  a run inside a Git the wrapper started carries the wrapper's value last as command-scope configuration,
  and must scan the person's directory and not `/dev/null/cli-git-no-hooks`.
  A value counts as the wrapper's own only when Git reported it in the `command` scope and it names the
  wrapper's own directory (see "Commit hooks"),
  so a configuration file that merely spells such a path stays the person's and is scanned
  (`src/native/health/doctor.rs`).
- It scans `<git-common-dir>/hooks` and,
  when `core.hooksPath` is set,
  the directory it names.
  A relative value resolves against the worktree root,
  or against the Git directory of a repository without a worktree.
  Each directory is scanned under the spelling Git was given,
  and a directory that two spellings name is scanned once,
  judged by the identity the file system gives it:
  a symbol link in the way decides what `..` means,
  so a lexical rewrite of the path would scan a directory Git runs no hook in.
  That identity is decided within the report's ceiling,
  because pathname resolution waits on a mount whose server has stopped answering;
  a resolution that gives no answer keeps the configured directory,
  which is then scanned beside the default one.
- A hook is any entry of a scanned directory that is an executable regular file or a symbolic link
  (a dangling link included),
  except names ending in `.sample`.
  Plain files without an execute bit,
  directories and samples are not hooks.
  An entry that is gone by the time it is looked at,
  deleted or replaced by another terminal or by a Git LFS upgrade after the directory was listed,
  is not a hook and does not end the scan:
  the report lists the hooks that are there.
  A directory that cannot be read,
  and an entry that fails to be looked at for any other reason,
  is still an error with status 2.
- A hook file that is byte for byte the shim git-lfs writes for its own name
  (`pre-push`,
  `post-checkout`,
  `post-commit` or `post-merge`;
  the text of git-lfs 3.8.0,
  `lfs/hook.go`,
  with its final
  line feed) is a Git LFS shim.
  Only a regular file counts;
  a link,
  a file that differs by one byte and a shim under another hook's name do not.
  The comparison reads at most 1024 bytes of those four names and nothing else.
- It prints one pathname per line on stdout for every hook that is not a Git LFS shim,
  the default directory first and each directory in name order,
  and nothing else on stdout.
  Each pathname is printed as its own bytes,
  because a pathname is a byte string:
  its text spelling with replacement characters names a different directory,
  so a text report could name a hook the command never inspected.
  When it finds any it also explains on stderr that traditional hooks are unsupported.
- Git LFS shims are reported on stderr only,
  apart from the other hooks,
  with their paths and an explanation:
  git-lfs writes these files again whenever it cleans a file in a Git that the wrapper did not start,
  so they can come back after deletion,
  and this is normal;
  the wrapper does the LFS upload itself and starts Git with traditional hooks off,
  so the files never run through it;
  nothing needs doing,
  and the files may be deleted or left.
- Exit status:
  `0` when no hook is found or only Git LFS shims are found (they are expected and harmless),
  `1` when any other hook is found,
  also when Git LFS shims are present beside it,
  and `2` when Git cannot locate a repository or a scanned directory cannot be read.
  A directory that does not exist holds no hooks.
- It says on stderr whether Git's own fsmonitor is off (see "Git's own fsmonitor" in `README.md`).
  It reads one listing,
  `git config --list --show-origin --show-scope -z`,
  with the caller's global options,
  because the configuration snapshot of the commit path names scopes but not files.
  The last `core.fsmonitor` entry decides,
  as for `git config --get`.
  When it is false (`false`,
  `no`,
  `off`,
  empty,
  or any text Git's own boolean reader reads as zero,
  which it does as a word or as an integer in `strtoimax` spellings such as `0`,
  `00`,
  `0x0`,
  `-0` or `0k`) or there is no entry,
  one line says that Git's fsmonitor is off by the owner's decision of 2026-11-03,
  with the file and scope of the deciding entry (or "not set"),
  the reason in one sentence,
  and `doc/troubleshooting/git-fsmonitor-daemon-start-wait.md`.
  When it is on (a true word,
  a number other than zero,
  a key without a value,
  or a hook path),
  one finding names the file,
  the scope,
  the same reason and document,
  and the command that turns it off for that scope
  (`git config --global core.fsmonitor false` for the global file;
  `--system`,
  `--local` and `--worktree` likewise);
  a value from `-c` or the environment has no file,
  so the finding says to remove it there.
  The line or finding comes after the other notes and before the filesystem section.
  It is about the person's own Git:
  the Git the wrapper starts for its own work passes `-c core.fsmonitor=false`
  (owner decision,
  2026-11-10;
  see "Real Git starts with traditional hooks off"),
  so a scope that turns the monitor on costs the person's own commands and a forwarded command,
  never a commit or a landing.
  The exit status never depends on it,
  and doctor never writes configuration.
  A listing that cannot be read or parsed is said plainly in place of the line.
- It loads `monochromatic-git-policy-cli.config.jsonc` only to name the retired `hooks`,
  `indexLock` and `landing` keys and the limits of the folder of saved copies;
  a file that cannot be loaded is not its to report,
  and every other command names the problem.
  It recovers no transaction,
  takes no lock,
  and it changes nothing:
  a name an earlier release kept in the common Git directory stays where it is,
  and the report reads a kept folder under whichever spelling stands there
  (`kept_names_in_the_git_directories.rs`,
  `path_for_reading`).
  The program that owns those names settles them,
  and that is not this command.
  Each file system read of it (the hook directories,
  the shim files and the host facts of the filesystem section)
  is awaited for at most the `REPORT_SCAN_CEILING` of `service/wait_ceilings.rs`,
  so a mount that stops answering cannot hold the command:
  a read that gives no answer makes the command say which read it was and carries on,
  with status `2` for a hooks directory and one line of standard error for the host facts and for the folder of saved copies.
- The saved copies are read under every data folder the repository holds,
  the current `monochromatic-git-policy-cli` and the earlier `cli-git`,
  and each folder that holds a copy gets its own block with its size,
  count and oldest save.
  A repository can hold both names at once,
  and the earlier folder can hold the only copy,
  so a report that named only one folder would hide a person's recoverable data from the person who asked what is there.
  The filesystem section describes the checkout Git reports for where the command runs
  (`git rev-parse --path-format=absolute --show-toplevel`),
  and says nothing about a file system when Git reports none,
  which is the case in a bare repository.
  A directory name is not a layout proof:
  the directory above the common Git directory is a metadata directory when the Git directory lives elsewhere.
  The conversion command it prints follows the layout Git reports:
  `migrate-to-subvolume` in the repository's main worktree and `convert-worktree-to-subvolume` in a linked one.

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
creates a worktree (`git worktree add`).
A `git worktree move` registers nothing new and is forwarded like any other command.
No lock stands around the command (no per-repository lock,
and no lock of any kind on this path):
the owner's rule is lock-free first,
and concurrent adds in one repository never wait for one another.
Each job the settlement lock used to do is done without one:

- The new worktree is attributed to the invocation by the destination its command line names (see "Destination of an add"),
  and its registration is found by following the pointer in its `.git` file to the administrative directory that names
  the destination back,
  so a worktree another add registers at the same time is never taken for this one's.
- A command that may copy records the destination and the stage name before real Git runs,
  in the claim the cheap-worktree rungs reuse,
  so a process killed after Git registered the worktree leaves a record the next command reports by name,
  with the commands that finish or remove it.
- The registration real Git made is bound when Git returns,
  carried beside the staged payload before the journal names the stage,
  and checked before staging and again before the first entry is written,
  by the creating command and by a recovery alike,
  so a worktree removed and replaced at the same path receives nothing.
  The administrative directory of no other worktree is read for this.
- Every item of a copy transaction (journal,
  stage claim,
  temporary file) names its owner,
  a process identified by its
  id,
  birth identity and PID namespace.
  An item is touched by another command only after its owner is proven dead.
  The copy of a live add is never recovered or reaped;
  the copy of a dead one is.
- Taking over a dead owner's journal is one atomic rename of the journal to a name that carries the new owner.
  Of several commands that find it,
  exactly one succeeds;
  the others see the old name gone and move on without waiting.
  No lock file exists,
  so none can go stale.
- A command that creates a worktree and every other forwarded command in a linked worktree or bare repository recover
  dead owners' items first and never wait for a live owner's.
- Two windows stay open,
  and neither is closed without a lock or a wider change.
  A destination created by Git after a reservation that could not be made (its parent did not exist) has no identity
  recorded before real Git runs,
  so a command that registers a worktree there and one that replaces it in the same instant cannot be told apart:
  the copy can be written into the replacement worktree.
  A destination that is replaced after the identity check and before the installation opens it can receive the copy
  the same way:
  the installer resolves every entry by path,
  so a check closer to the call narrows that window without
  closing it,
  and closing it needs the destination held open and every entry created through that handle.
  Both need a second program to move or replace a worktree inside a window measured in microseconds while this command
  runs,
  and the person sees a copy written into a worktree the notice does not name;
  `git worktree list` shows which worktree stands where.
  A destination this command reserved is removed again after a refused add while it is still the object this command
  made and still empty,
  with `rmdir`,
  so no empty folder is left inside the enclosing checkout for a later Git command run
  there to act on the enclosing checkout instead of a worktree
  (owner report 2026-11-10:
  `git worktree add -b <branch> <path> main`
  with a branch that already exists left `<path>` empty).
  The removal is guarded three ways:
  only a destination this command made is considered,
  its identity is read again immediately before the
  removal,
  and the removal itself refuses a directory that holds anything,
  so a destination that stood before the command,
  a folder the person already had above it,
  and a destination another program wrote into all stay;
  the last one is named on the command's output.
  Git's own status and text are what the caller sees either way.
  A replacement that lands between the creation and the identity read is not told apart from the directory this
  command made;
  that window needs a second program to move a directory into the reserved name in microseconds,
  and removing nothing but an
  empty directory loses no data.
- Every other forwarded command,
including every commit preparation,
runs without any of this,
so concurrent commits in linked worktrees never contend on it.
A worktree created by Git invoked through an absolute path from a hook bypasses the wrapper
and therefore gets no ignored-state copy,
matching the documented bypass of everything else.
A hook that runs `git worktree add` through the wrapper is itself an applicable source:
  it copies its own source's ignored state
into its own worktree,
  and the add that started the hook copies into its own worktree only.

Destination of an add:
the wrapper reads the destination from the resolved command line (aliases expanded) exactly as Git's own option parser would
(`parse_long_opt` and `parse_short_opt` of `parse-options.c`,
Git 2.56.0).
One table lists the options of `git worktree add` (`-f`,
`-b`,
`-B`,
`--orphan`,
`-d`,
`--checkout`,
`--lock`,
`--reason`,
`-q`,
`--track`,
`--guess-remote`,
`--relative-paths`,
each long option also as its `--no-` form),
and a control compares it with
what `git worktree add -h` prints,
so it cannot drift from Git.
A long option is its exact name,
which wins over any prefix;
otherwise the unique prefix of one name,
with or without `--no-`;
a prefix of two names is Git's "ambiguous option" error,
and the wrapper does not guess.
A value on a flag or on a negated option,
a missing value,
an unknown option and a double negation are Git's errors too,
so
their destination stays unknown.
Short options bundle,
and `-b` and `-B` take the rest of the token or the next token.
`--` and `--end-of-options` end the options.
The global options before `worktree` are `-C` (the only one that moves a relative destination),
`-c`,
`--config-env`,
`--git-dir`,
`--work-tree`,
`--namespace`,
`--attr-source` and Git's neutral flags:
a relative destination lies against the
working directory whatever `--git-dir` and `--work-tree` say,
because Git changes into the work tree and prefixes the operand
when the working directory is inside it,
and leaves both alone when it is outside (`setup.c`).
Git accepts no option between `worktree` and `add`,
so such a command is Git's error and no copy is made.
The parent directories of the destination need not exist:
Git creates them,
and the destination is the real path of the nearest
existing ancestor with the rest as written.
A form the reading does not know or Git rejects (an unknown option,
an ambiguous prefix,
a value on a flag,
`--exec-path`
without a value) leaves the destination unknown:
real Git runs as it would without the wrapper,
no copy is made,
and when Git succeeded one line on standard error says that the worktree received no ignored-state copy and how to name it so
that it does.
This is the answer for the forms the wrapper cannot know a destination for;
a comparison of the repository's registrations before and after Git would need a lock to be safe against a concurrent add,
and copying into a worktree that is not the command's own is worse than copying nothing.

Before forwarding an applicable source,
monochromatic-git-policy-cli captures the common Git directory and linked-worktree administrative identity set.
An invocation targeting the main worktree bypasses journal recovery
and synchronization,
even when real Git creates a linked worktree,
with one exception:
a command that creates a worktree copies the main checkout's ignored state,
under the leave-out rule below,
into the new worktree,
under the journal.
The main checkout is an applicable source for that command only;
its other commands cost no filesystem reads.
Before real Git runs,
the wrapper records whether the destination is already a registered worktree.
After real Git and its hooks return for an applicable source,
the destination is a synchronization destination when it is registered now and was not before,
whether argv named `worktree add` or an ordinary Git alias expanded to it,
and also when real Git returned nonzero after retaining registration.
A destination that was registered before Git ran (Git refused it) receives no copy.
A registration is confirmed when the pointer in the destination's `.git` file names an administrative directory of this
repository whose `gitdir` file names the destination back,
written absolute or,
under `--relative-paths` and
`worktree.useRelativePaths`,
relative to the administrative directory.
A destination Git made,
with Git succeeding,
whose pair cannot be confirmed receives no copy,
and one line on standard error
says so.

For an applicable non-bare invocation,
the source is the invoking linked worktree root,
or the main checkout root for the exception above,
after Git and `post-checkout` settle.
Git's standard exclusion stack selects the source paths:
per-directory `.gitignore`,
repository exclude state,
and configured excludes files.
The source set includes ignored regular files,
directories,
and symbolic links.
Ignored sockets,
FIFOs and device nodes cannot be copied into another worktree and mean nothing there,
so the walk leaves each of them out before anything is opened,
one stderr line names them,
and the rest of the ignored state is copied.
The traversal that finds those entries never enters a directory whose name begins `.cli-git-worktree-copy-`,
because that directory is another command's stage container:
its payload is another worktree's ignored state,
and a container its owner removes while a walk runs would otherwise end this command with a filesystem error.
A selection that names repository paths asks about each one as `:(top,literal)<path>`,
and that listing runs real Git with `--no-literal-pathspecs`,
so a global pathspec mode (`GIT_LITERAL_PATHSPECS`,
`--literal-pathspecs`)
cannot make the name match nothing and leave the file in the source uncopied.
A bare repository has no source root and synchronizes an empty set.
The copy settles itself against the source in bounded passes,
because a live checkout removes and rewrites its
 ignored files while the copy runs:
a source entry that is gone when its turn comes has nothing left to copy and
 leaves the manifest,
whether the walk that lists the entries or the copy that fills them finds it gone,
and a directory the source removed takes the staged entries below it with it,
an entry that changed under the copy is copied again,
twice at most,
and a directory whose permission bits changed has those bits applied instead,
because its content belongs to the entries below it,
and an entry that is
 still changing after that is left where it is,
untouched,
in its source,
is not installed,
and is
 named on one line beside the entries the person could not read.
The stage equals the source for every entry it holds,
and a copy
 that cannot finish names the path of the entry it failed on rather than only the checkout it came from.
The number of
 passes is bounded (`worktree_copy_settling.rs`),
so a source that churns faster than the copy can prove it never
 holds a command indefinitely.
The readiness of a source is decided in `src/native/worktree_copying/worktree_copy_source_still_being_filled.rs`:
a source that is itself the destination of a copy still in progress holds only part of its own ignored files,
so nothing is staged from it.
The wrapper decides this from the durable items it already judges:
a journal whose record names the source,
and an add's stage claim whose record beside it names the source.
A live owner of either is a copy into the source that has not finished;
a dead owner's item is not,
and neither is a copy's own claim,
whose record names no destination.
A creating command that finds such a source still runs real Git,
creates its worktree,
copies nothing into it,
and writes one line on standard error saying that it received no ignored-state copy,
why,
and the commands that have the copy made once the other add has finished:
`git worktree remove <destination>` and then the same `git worktree add` again,
which copies the source's ignored state once it is whole,
with `git worktree remove` leaving a worktree that holds changes alone.
The package-manager repair does not run for such a worktree,
as it does not after a failed copy.
Nothing waits for the other add and no lock is taken.
A copy that is still staging its source has no journal yet,
and a copy's own stage claim records no destination,
so that window alone is not covered.
No repository configuration touches this behavior.
The wrapper-only flag `--no-worktree-copy` skips synchronization for one invocation.
It is recognized in flag position after the subcommand and before Git's `--` pathspec separator,
and,
like the other wrapper controls,
among Git's global options before the subcommand,
is stripped before real Git runs,
and leaves value-position tokens with the same bytes untouched.
An opted-out invocation performs no journal recovery;
pending journals wait for the next repository invocation without the flag.

### Cheap worktree creation and removal on btrfs, XFS and overlayfs

Design,
measurements and the reasons for each choice:
`doc/planning/cli-git-btrfs-worktree.md`.
The ladder applies to a `git worktree add` that names one path and at most one commit-ish and uses only `-b <branch>`,
`--detach`,
`-d`,
`--quiet`,
`-q`,
`--lock`,
`--reason` and `--checkout`,
run through the wrapper from a linked worktree,
a bare repository or a main checkout,
without `--no-worktree-copy`,
without a global option other than `-C`,
`--no-pager`,
`-P`,
`-p`,
`--paginate`,
`--no-advice` and `--no-lazy-fetch`,
and with an existing parent directory of the destination.
Any other form is Git's own,
unchanged.
The ladder takes no lock:
every name it uses (the stage,
the claim,
the destination) belongs to
this add alone,
and every rung that cannot run falls to the next without a message to the
person and with one diagnostic line (`MONOCHROMATIC_VERBOSE=true`) that names the rung and the reason.

A source is the worktree the command runs in,
the main checkout included.
It is clean when no merge,
cherry-pick,
revert,
bisect,
rebase,
sequencer or `index.lock` state exists in its Git
directory,
its index equals the tree of its `HEAD` (no staged change,
intent-to-add,
skip-worktree,
unmerged or
submodule entry),
every tracked file has the size,
modification time,
change time,
inode,
owner,
type and executable
bit its index entry recorded and is older than the index file,
and `git ls-files --others --exclude-standard` lists
nothing.
A source is also refused for a sparse checkout,
for any worktree-scope configuration entry,
and for `core.sparseCheckout`.
The clean check reports why it refused as a kind and a reason:
an operation in progress (with the name of the mark),
an
index or object database the readers cannot read,
an entry no reset can undo (unmerged,
intent-to-add,
skip-worktree,
assume-unchanged,
submodule),
a staged change,
a changed tracked file,
an index entry the stat data cannot vouch for,
or an
untracked file.
Callers and the sentence for the person decide by the kind,
never by the words of the reason.

The two halves of that check are waits of the process's runtime:
the listing of untracked files is a Git run awaited there,
and the proof of the index and of every tracked file is a job of the blocking
pool beside it,
so a filesystem that stops answering the proof holds no bounded Git wait open.
Where the system refuses to start a pool thread (`ulimit -u`,
the control group's `pids.max`),
`run_a_job_or_in_turn` runs the proof on the awaiting task itself,
under a place of the process's bound,
because a thread is only a way to do the work sooner;
the person then sees the add wait until that filesystem answers,
with no ceiling firing,
and raising the limit and running the command again is the remedy.
Where no runtime can be built at all,
the listing and the proof run one after another on the calling thread and every verdict
is the one the runtime gives.

Snapshot of a main checkout with uncommitted changes:
a main checkout whose only trouble is an ordinary change (a staged change,
an edited,
deleted,
renamed or mode-changed
tracked file,
an untracked file that no ignore rule covers,
a tracked file the stat data cannot vouch for) is snapshotted
all the same,
and the new worktree is restored to what `git worktree add` makes from a clean checkout.
Git's status listing nothing is an answer,
not a failure:
a main checkout whose index was written in the same second as its
files (a merge,
a pull or a landed commit just did that) or whose tracked file was touched without a change is snapshotted
and restored like any other,
and says nothing about left-out changes because there are none.
Only a status run that ends with a nonzero exit code or cannot start refuses,
and the line names the run,
its exit code and what Git said.
Every other refusal stays:
an operation in progress or a held `index.lock`;
any unmerged,
intent-to-add,
skip-worktree or
assume-unchanged entry,
or a submodule,
found anywhere in the index (not only at the first refusal);
a `.gitattributes`
file among the changes or untracked anywhere;
that untracked listing (`:(glob)**/.gitattributes`)
runs real Git with `--no-literal-pathspecs`,
so a global pathspec mode cannot hide a deep attributes file from the refusal;
a sparse checkout;
a source that is a linked worktree,
whose index lies outside
its directory and so is not in the snapshot;
and the clone rung,
which is only for clean sources.
The restore reads the index inside the snapshot before the snapshot's `.git` is replaced by the new worktree's pointer,
and
installs that index as it is (stat data kept,
`UNTR` and `FSMN` dropped),
so the index and the files are one instant and the
stat data still marks every file that was edited.
An entry no walk proved keeps the fields its own index recorded,
never fields taken from the file:
fields
of the file itself would vouch for its bytes,
and Git would take an edit for the entry's content.
Then,
in the stage only and before it is swapped in,
three real-Git runs under the new worktree's Git directory and the stage as work tree,
in this order:
`git ls-files --others --ignored --exclude-standard --directory --no-empty-directory` (the ignored paths the
stage holds now,
under the source's own rules,
each whole ignored directory as one name),
`git clean --force -d` (untracked files that no ignore rule covers,
and empty untracked directories,
go;
ignored files stay),
and `git read-tree --reset -u <new commit>` (the index and the tracked files become the new commit's tree,
rewriting exactly the paths whose files or entries differ).
The reset rewrites a path whose file Git's own stat check can see changed,
and an edit of the same length,
made
in the same second as the entry that recorded the file,
keeps every field that check compares.
Between the `clean` and the reset,
in the stage only and with no Git run of its own,
the restore settles those files by content:
it reads the index the new worktree received,
and for every entry whose stat data cannot vouch for the file it reads the file and compares
it with the blob the entry names (the same content decision the clean check and the snapshot
proof use,
with the same refusals for a path a conversion attribute could change).
Every path whose bytes differ,
and every path the comparison cannot decide,
is removed from the stage,
and the reset then writes the commit's bytes there.
That removal costs a write of the commit's content and never publishes other bytes.
Only the files the stat data cannot vouch for are read:
a file whose recorded size,
inode,
modification time and change time (to the nanosecond) are the file's own is believed,
so
an ordinary checkout and an add from a touched but unchanged file read nothing.
The stage is the wrapper's own snapshot and is published only once the proof below holds,
so
the main checkout's files are never written.
The clean runs before the reset,
so the rules that decide what goes are the source's,
and a
new commit that ignores less than the source (a branch that predates an ignore rule,
or one that removed it)
cannot turn a person's ignored cache,
local configuration or environment file into a file the clean removes.
Those paths are then carried
into the new worktree,
which is the state the ignored-state copy gives the worktree on every filesystem
that cannot snapshot ("Linked-worktree ignored-state synchronization"),
and the proof below counts them as carried.
The `clean` carries no `--quiet` (see "Wrapper output and options",
the inner runs);
it is run captured,
in the same stage and
under the same Git directory as the proof below,
and what it says ("Removing <path>" for each untracked file of the stage) is
kept in the journal and not printed,
because those paths belong to a stage the person's command never names.
These three runs and nothing else are run for the restore.
The `read-tree` is started as a real Git child by the lifecycle's runner,
the same path as the `git read-tree -m -u` of a clean
snapshot,
and the `clean` and the listing the same way as the proof's reads,
so the wrapper's own guards of a person's `reset` and `clean` do not apply to them:
they act on the wrapper's own
new snapshot,
never on a checkout of the person,
and they refuse to start when the stage is the source's directory or the
Git directory is the source's.
The stage is then proven:
`git diff-index --cached --quiet <new commit>` (the index equals the commit's tree),
`git ls-files --others --exclude-standard --directory --no-empty-directory` lists nothing besides the carried
the listing named (the same name,
a file below a carried directory,
or a whole directory Git collapsed into one name whose every path below it is carried,
which the same query without `--directory` decides),
and `git diff-files --quiet`
finds no tracked file different from its entry.
A collapsed directory holding even one path the restore did not carry is refused,
and the refusal names that path.
A failed step or a failed proof ends the rung:
the stage is destroyed and Git's own checkout makes the worktree,
with the
sentence of "Telling why an add was not made as a snapshot" naming the reason.
The main checkout is read to be snapshotted and listed (`git --no-optional-locks status` and `ls-files`),
and is never written:
no step runs in it,
and its files and index are what they were.
Unless `--quiet` is given,
a second line follows the snapshot note:
the uncommitted changes of the main checkout (their number
of paths) were left out of the new worktree,
which holds the committed files and none of the changes.
The same words go to the journal as one entry of priority `info` under the `SYSLOG_IDENTIFIER` `monochromatic-git-policy-cli`,
with the fields
`CLI_GIT_WORKTREE`,
`CLI_GIT_DIRTY_PATHS` (the number of paths) and `CLI_GIT_DIRTY_FIRST` (the first of them),
also under
`--quiet`.
Changes made to the main checkout while the snapshot is taken or restored are out of scope,
as for every rung:
using the
command is accepting that risk,
and no step defends against it.
The wrapper passes no `core.fsmonitor` setting to the `git worktree add` it forwards;
Git's fsmonitor is off in the person's global configuration
(owner decision,
2026-11-03),
and the Git the ladder starts for its own steps passes `-c core.fsmonitor=false`
(owner decision,
2026-11-10;
see "Real Git starts with traditional hooks off" and "Git's own fsmonitor" in `README.md`).

Rung 1 needs a clean source that is a btrfs subvolume root on the destination's filesystem and,
for a source that is a
linked worktree,
no nested linked worktree that is an ordinary directory;
a snapshot of the main checkout sets such a
worktree aside instead (see "Leaving out of a snapshot").
Real Git runs with `--no-checkout` added after `add`.
The wrapper then reads the new `HEAD`,
refuses a subject Git would have to print or `.gitattributes` files that differ
between the source's tree and the new tree,
claims a stage name beside the destination in the journal root,
snapshots the source there (`BTRFS_IOC_SNAP_CREATE_V2`),
makes the stage private,
removes the stub of every nested
linked worktree and of the destination,
replaces `.git` with the bytes Git wrote,
compares every tracked file of the
snapshot with the copied index,
installs the index with `rewrite_index_stat_with` (extensions `UNTR` and `FSMN`
dropped),
applies the tree difference with `git read-tree -m -u <source commit> <new commit>` under Git's
`GIT_DIR` and `GIT_WORK_TREE`,
gives the stage the source's mode and swaps it with the destination
(`RENAME_EXCHANGE`).
The directory real Git made,
now at the stage name,
is settled without a recursive removal:
Git's pointer and the empty
directory go,
and anything another process added to it meanwhile stays there and is named in a warning.
Git's line `HEAD is now at <abbreviation> <subject>` is printed unless `--quiet` was given.
The snapshot carries the whole ignored state of the source,
so the ignored-state copy skips that destination.
The swap this command made itself proves the worktree is its own,
so the skip comes before any question about the reservation the destination answered for before the swap:
no line says the worktree received no copy,
and no record describes it as unfilled.
A command killed after the swap and before the directory is settled leaves it at the stage name,
and recovery proves that swap from the two identities the add recorded before it ran:
the destination's record names the directory Git made and the stage's record names the snapshot,
so the object at the stage name is the first of them and the object at the destination is the second.
It settles the directory the way the creating command does,
and settles the add's records without a copy,
because the snapshot carried the ignored state;
no command then says a copy is still owed.
A stage name whose recorded destination is gone or replaced is cleared only when the folder holds
nothing but the pointer Git wrote for that add,
and a folder holding anything else keeps its claim and its records and is named.
The stage's record may name no identity,
because a command writes it before its stage exists
(`src/native/worktree_copying/worktree_copy_stage_made_object.rs`):
the object at the stage name still proves the swap,
since only the exchange puts it there,
and the destination is compared with nothing.
Such a folder is then cleared only when it holds nothing but the pointer Git wrote,
a person's entry is never renamed into a folder no record proves is the worktree it belongs to,
and the add's records are settled the same way,
because no copy of the source's ignored state serves a worktree that either holds it or no longer stands;
the line says that the two were not compared.

The snapshot an add takes is its own,
made on the add's own path:
`BTRFS_IOC_SNAP_CREATE_V2` forces a btrfs transaction commit of the whole filesystem on the thread that calls it and
waits for the source's writeback first,
which this repository's own timing log measured at 5,413.0 ms at the median over the 402 adds that recorded the step
(`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
"Step 2" and "Step 3"),
and which the owner accepted at his bound of 60 s for an add.
An earlier release installed a copy the repository's commit service had made ahead of the add and then named every
ignored entry that had changed after that copy;
that behavior was open to the owner's veto,
and when he was asked on 2026-11-11 to keep it or to take a fresh snapshot on every add,
measured at 5.4 s at the median and 26 s at worst in the fixture that asked him against his
"60s is fine for adding worktrees",
he answered "Fresh snapshot every add".
So no add claims a copy from the pool,
no add asks the service for one,
and nothing of the pool is filled by an add.
The pool's modules (`src/native/worktree/worktree_snapshot_ahead.rs` and its claim and job) are still in the tree,
so a `snapshot-ahead` record an earlier release queued is still drained by the service;
no add reads what it holds.
The ignored files the new worktree carries are the source's as of the instant this add's snapshot was taken,
which is what a snapshot has always meant,
the source keeps every file written after it,
and because the copy is never older than the add that installs it,
nothing is behind it and no walk names entries that changed after it.
Leaving out of a snapshot:
before the stage is made private and its stubs removed,
the wrapper prunes a snapshot with the rule of `src/native/worktree_copying/worktree_copy_prune.rs`.
A trash directory that an earlier add left under `.claude/cli-git-trash` in the source is not carried:
the walk
 skips the trash parent so that this add's own trash is not moved into itself,
so every other directory the
 parent holds is moved into this add's trash first,
whichever add made it,
and the emptying that runs before this
 command returns covers it.
Without that,
a kill before its emptying left a trash that the next snapshot carried into a
 new worktree,
where nothing would ever dispose of it because the record names only the add that ran,
and a target
 commit that stops ignoring the path makes another command's stage container stageable there.
A directory the
 index tracks under that parent is never moved.
A directory of that name outside the trash parent is no trash and is never
 touched.
A source whose Git directory is its common directory,
the main checkout,
gets the main-checkout rule below.
A linked or bare source has no cache or run-record rule,
so there the leave-out moves only another command's stage containers:
everything else such a source ignores is carried,
as it is by the ignored-state copy.
First it works out what the source's index tracks:
every directory above a tracked file,
and every directory-valued
index entry (a gitlink) at its own path.
None of those is ever moved,
whatever its name.
A registered nested worktree that is an ordinary directory and holds or is a tracked path cannot be left out,
so the
snapshot is declined,
before anything is touched,
with a reason that names the worktree and the tracked path.
Then it unlinks the `.git` file of every other nested linked worktree that is an ordinary directory and of every other
directory that holds a `.git` file,
because a copied pointer names another worktree's administrative directory.
Then it moves each such directory,
each directory of a linked or bare source whose name begins `.cli-git-worktree-copy-` (a stage container of another
command),
and,
for the main checkout,
each directory the rule names as a cache or a run record,
into a trash directory
`.claude/cli-git-trash/<forty hexadecimal digits>` inside the stage,
by rename.
A stand-in directory the snapshot shows where the source holds a nested subvolume (inode 2,
btrfs's stand-in for a
subvolume a snapshot did not carry) is never moved by the leave-out:
it holds nothing,
so it carries no ignored state,
and btrfs refuses to rename it.
It stays for the stand-in sweep,
which reads the folders that hold this repository's worktrees once each and takes a folder whole when every entry
of it is such a stand-in:
one rename into the snapshot's trash,
and a fresh empty folder with the mode of the one it replaced takes
its place.
The stand-ins are deleted after the add returns by the same commit service and in the same work item as the rest
of the trash,
so an add pays one rename and one directory made again however many worktrees the repository has,
and
not one btrfs metadata change for each of them
(`doc/planning/cli-git-worktree-add-stand-ins-off-the-waiting-path.md` measures the
counts).
When the leave-out set nothing aside the sweep makes that trash itself;
a source whose Git does not ignore
the trash place gets none,
and its stand-ins are removed as empty directories where they are,
because a trash the
new worktree's Git does not ignore can be staged.
A folder that holds one entry that is not such a stand-in stays where it
is,
because every other entry may be someone's data,
and only the stand-ins in it are removed one by one.
A folder that is taken whole is looked at again in the trash,
because a directory somebody made between the check and
 the rename is never deleted with the stand-ins it replaced:
it goes back to the name it came from,
and its stand-ins
 are then removed one by one.
When a directory took that name meanwhile,
the folder cannot go back,
so it is kept whole
 under `.cli-git-worktree-kept-<hexadecimal digits>` beside the trash of the new worktree,
one rename inside
 the snapshot,
because a folder that holds a stand-in of a nested subvolume cannot be renamed out of it at all.
The warning names the path that folder will have in the new worktree,
and the cleanup that removes a stage the add never served leaves the stage in place when it holds such a
 folder,
naming every path it kept (`never discard user data`).
A stand-in
whose subvolume no worktree names stays,
since the snapshot never carried its files and their absence must stay visible.
The service's emptying allows a stand-in through the device check that stops it at a nested subvolume or another
filesystem:
a stand-in reports a device of its own and belongs to the tree being removed.
The snapshot's own `.git` copy goes into that trash behind one rename,
after the index is installed and before the
new worktree's pointer is written:
it is the whole common directory,
with one administrative directory for every worktree of the
repository,
and removing it inside the add costs one btrfs metadata change for each of them.
A source whose `.git` is a pointer
file has one file there,
and it is unlinked as before.
Names are kept as bytes:
a name that is not UTF-8 is moved or kept by its bytes and never taken for another entry.
Every directory is opened by one plain name below the handle of the one above,
without following a link,
so a link at
`.claude` or at the trash parent in the stage is no way out of the stage.
When the trash cannot be made that way,
or Git does not ignore the place after the repository's own exclude line was added (one `git check-ignore
--no-index` in the source,
asked of every kind of source),
the directories are removed in the foreground,
in place,
by the same held-handle walk.
The leave-out writes down a manifest:
the identity of the trash directory (device,
inode and birth time,
taken when it was
made) and the identity of each directory moved into it.
That file is written at a name opened without following a link and without blocking:
a link another process put there is refused instead of written through,
which would replace the file it points at,
and a named pipe there is refused at once instead of waited on,
so no add waits for a reader that may never come.
A directory the leave-out set aside as another command's stage container is removed in the foreground,
before the
 snapshot is installed,
and never renamed into the trash:
a container holds another worktree's ignored state,
credentials included,
and removing it first
 means those bytes never reach a worktree at all,
not even the ignored trash of one,
so no ignore rule and no failure of the service can leave them anywhere a
 later command could read them.
A directory that is set aside whole,
a registered nested worktree or another checkout,
is swept for the same
 containers first,
because a container stands beside the destination of one add and such a directory is another place
 one can stand:
every entry named with the stage prefix,
at any depth,
is removed with the directory's own files
 still in it,
so nothing of another command's reaches the trash through that route either.
The sweep walks the
 directory it is moving,
`.git` included,
because that directory is itself another checkout and a container can stand
 beside that marker,
reads no file and
follows no link.
A directory the sweep cannot look inside is never taken for one with nothing to
 sweep:
the leave-out refuses and names it,
so nothing is set aside on a guess.
Everything else the leave-out sets aside,
the caches,
the other checkouts and the run records,
goes into the trash by rename only,
and the command that made it never empties it.
Measured on this host,
a container of 20,000 one-kilobyte files takes 460 ms to remove,
against 0.1 ms to rename into the trash.

Handing the trash over:
the add writes one work item for the trash and wakes the commit service,
then returns.
A trash emptied inside the add made ten concurrent adds spend more than ten minutes each on this host on
2026-11-10,
with 2,182 of 2,796 blocked samples inside `unlink` or `rmdir` and the drive 99% busy
(`doc/planning/cli-git-worktree-add-cost-of-its-trash.md`,
"Measured on 2026-11-11").
The record sits in the repository's common work directory,
`<common>/monochromatic-git-policy-cli/work/pending/trash-<the trash's name>`,
and not in the new worktree's:
the drain of one work directory empties one worktree's trashes one at a time and different worktrees' side by side,
so concurrent adds never delete beside each other,
and the claim that drain takes is the lease every other kind of work already uses,
so no lock is added for this.
The item names the new worktree,
the common directory the record is read from,
the reference `refs/cli-git/trash`,
the trash's forty hexadecimal digits as its object,
and the manifest as its parent.
The add prints one line on standard error unless `--quiet` was given:
it names the worktree,
says how many directories were set aside,
and says that the commit service empties them after the command returns,
one worktree's trashes at a time and at the lowest I/O priority.
A wake that fails is reported as a warning that names the worktree,
the trash,
and the record,
which still empties the trash when the repository's service next runs,
and never fails the add,
because the worktree is whole either way.
The wake waits `HAND_OVER_GREETING_WAIT` for the service's greeting and never the commit's `START_WAIT`:
the record is written before the wake and the add needs no answer to it,
so a service that greets nothing costs one later wake and never the add's work
(`src/native/worktree/worktree_trash_hand_over.rs`).
A path that is not valid UTF-8 cannot be named by a work item,
so the hand-over writes none,
says so,
and only the person removes that trash.

A standing trash cannot be staged,
whatever the target commit ignores:
before the leave-out asks Git whether the trash place is ignored,
the wrapper adds `/.claude/cli-git-trash/` to the repository's common `info/exclude`,
which no commit writes.
Measured with Git 2.56.0 on 2026-11-11:
a linked worktree reads the common directory's `info/exclude` and never its own worktree's,
`git check-ignore` names that file,
`git add --all` stages nothing of a trash standing there,
and neither a target commit that ignores nothing under `.claude/` nor one that tries to re-include the trash parent
changes that.
The line is added once and never removed,
the person's own patterns are kept as they are,
and a directory of that name outside the trash parent is no trash and is never touched.
When that file cannot be read or written,
the question is asked of the rules the repository already has,
and a `false` answer removes the directories in the foreground,
which is the shape an add had before the hand-over existed.

The emptying happens only if all of these hold:
the worktree's `.git` file,
read without following a link,
points into the Git directory the record names
(that directory itself,
or one of the per-worktree directories below its `worktrees` folder);
every directory from the worktree down to the trash is a real directory on the
worktree's device;
the trash has the recorded identity (device,
inode and birth time,
taken when it was made);
the file at the manifest's name is that trash's own only when the identity it records is the directory's,
and it is unlinked only while the name still holds the object whose bytes were read there,
so a file moved onto that name is left alone and named in the failure;
and each
 entry removed is the recorded object (a replacement,
a link or anything unrecorded is left alone and named in the
failure).
A worktree that is no longer there is done rather than failed:
a removal of it took the trash with it.
Nothing is found by scanning the trash parent for names,
so a directory the main checkout carried there is never
 touched.
The recorded entries are separate directories that share only the trash handle,
so they are removed side by side
within the host budget (the threads the process may run less a margin,
and the free memory over the estimated peak of
one removal,
128 MiB,
the measured peak of removing a directory of a million names rounded up);
each is still removed only
when it is the recorded object,
a reason for one entry never stops the others,
and the failure names the reasons in entry
order.
Every step of that removal,
each entry and the trash directory itself,
runs at I/O priority class `idle`,
the lowest Linux offers:
the class is set on the thread that unlinks and put back after it,
so the service's own push and review work keeps its priority,
and a thread whose class cannot be read or set is left as it is.
That value belongs to the thread that lowered it:
it cannot be carried to another thread,
because a drop writes the
priority of the thread that drops it,
and a value that travelled would overwrite the receiving thread's own priority while the lowered
thread stayed at the idle class.

A trash that cannot be emptied is reported by the service as a line that names the trash and the reason,
and a refusal settles its record as failed,
which is where the promise of a later attempt stops;
a failure the emptying's own words call retryable
(a busy volume,
an I/O error)
instead keeps its claim,
and the claim's lease running out queues the record again with one more try,
up to `MAX_TRIES`
(`src/native/landing/work_item_recovery.rs`),
so a later round tries it again instead of dropping the work.
Every command later run in that worktree,
unless its output is read by a program,
therefore names every directory
in the worktree's trash parent that no pending or claimed record accounts for,
on standard error,
with its path:
a trash whose emptying failed has a failed record that is never tried again,
a record still pending after one claim lease is one no service ever looked at,
and a directory nothing recorded cannot be proven to be one the leave-out moved;
nothing there is deleted,
and a worktree with no trash parent costs the command one lookup.
A trash that says what it is,
because the snapshot wrote its manifest inside it before the swap published it,
is offered to the queue again when no record the service still works on accounts for it,
and only a record that command wrote is reported as recorded now:
a record the queue already holds under the trash's name is reported as already there,
because the id of a trash's record is the trash's name,
and one the service settled as failed is named as a trash nothing will empty,
with the same instruction to look at it,
so no command promises an emptying no service will claim.
A record that command writes is work only a woken service looks at,
so the scan tells the repository's service once,
only when it wrote one,
exactly as a hand-over does.
A work item of kind `trash` that an earlier release wrote is emptied by the same handler
(`empty_trash`),
with the same checks and the same identities,
so a repository that ran one keeps its promise,
and a trash an earlier add left in the source is still moved into this add's trash and covered by its record,
whichever add made it.
A copy of the ignored state leaves the trash parent out of the source as well,
so no copied worktree carries another command's trash either.
The race with other processes is a risk the person accepts by using the command:
a directory of the new worktree that another
process replaces,
moves or links elsewhere while the leave-out or the emptying runs is guarded only by the cheap checks above
(opening without following links,
comparing identities when an object is opened);
the wrapper builds nothing further against
it and does not promise to survive it.
A snapshot of the main checkout does not print the copy summary,
because no copy ran;
unless `--quiet` is given it writes one
line to standard error saying the worktree was made as a snapshot of the main checkout and that nested worktrees,
caches,
run records and the stage containers of other commands were left out.

Parallel checkout of the new worktree:
when Git's own checkout makes the new worktree (the person's `git worktree add` run as Git's checkout,
and the `git reset --hard`
that completes a worktree after a snapshot or clone attempt failed),
the wrapper puts `-c checkout.workers=0` (one worker per
logical CPU) before the subcommand,
after the global options already there,
only when nothing the person controls sets
`checkout.workers`.
Anything counts as setting it:
an entry of any scope in the configuration Git reads (system,
global,
local,
worktree,
`include`d
files),
a `GIT_CONFIG_KEY_<n>` or `GIT_CONFIG_PARAMETERS` variable that names the key,
and a `-c` or `--config-env` the person
typed before the subcommand,
whatever the value.
Then nothing is added and Git uses the person's value.
The files are read through the one configuration read the wrapper makes for the add (answered in process where the readers
know the command line);
a configuration that cannot be read adds nothing.
The snapshot and clone rungs run no Git checkout of the whole tree and are not changed.
Owner decision,
2026-11-03 (question tool,
chosen against the coordinator's recommendation):
"Pass only if unset".
Basis:
Git's parallel checkout made a 20,000-file checkout faster in all three paired runs another agent measured
(36.9 s to 4.5 s,
3.1 to 2.3,
7.2 to 1.75).
The wrapper passes no `core.fsmonitor` setting to the add it forwards and does not touch its daemon;
Git's fsmonitor is off in the person's global configuration (owner decision,
2026-11-03),
and the ladder's own steps run with `-c core.fsmonitor=false` (owner decision,
2026-11-10:
asked "Should the wrapper switch fsmonitor off for its own internal Git commands only?",
the owner chose "B:
off for internal Git (Recommended)";
see "Real Git starts with traditional hooks off").

Telling why an add was not made as a snapshot:
when a `git worktree add` that the ladder serves succeeds without being made as a snapshot,
and the person could change
the cause,
the wrapper writes one line to standard error unless `--quiet` is given,
after Git's own words and before the
copy summary.
The line starts with `monochromatic-git-policy-cli: this worktree was not made as a snapshot, so it took longer, because `,
then the cause in
plain words,
then what makes the next add quick,
and it names no internal step.
The causes it tells are:
uncommitted changes in a source that is not restored by a snapshot (a linked worktree;
the number of paths `git --no-optional-locks status --porcelain=v1 --untracked-files=normal`
lists,
and the first of them,
with control characters shown as `?`;
the remedy is to commit or stash);
a running Git command in the source (its `index.lock` exists;
the remedy is to add again when it has finished);
an unfinished operation (a merge,
cherry-pick,
revert,
bisect or rebase mark exists;
the remedy is to finish or abort it);
an index entry no reset can undo (skip-worktree,
assume-unchanged,
intent-to-add,
unmerged or a submodule);
an index that cannot yet vouch for its files although Git lists no change,
in a source that is not restored by a snapshot
(a file as new as the index,
or a size,
time or inode that differs;
the remedy is `git status` once);
a Git status that failed (the run,
its exit code and what Git said;
the remedies are to run `git -C <source> status`
and fix what it names,
or to wait for a Git command that is running there);
a btrfs source that is no subvolume (the remedy is `git monochromatic-git-policy-cli migrate-to-subvolume` for the main checkout and
`git monochromatic-git-policy-cli convert-worktree-to-subvolume` for a linked worktree);
a destination on another btrfs filesystem than the source;
a feature a snapshot does not reproduce (a sparse checkout,
a worktree-specific configuration entry,
a nested worktree
reached through a link),
told in the check's own words;
and a snapshot that started and could not be finished,
told with the reason,
after which Git's own checkout made the worktree.
A source or destination that is not on btrfs gets no line,
because nothing the person does makes the add a snapshot there.
A failed add gets no line either;
Git's refusal is the thing to read.
The same words go to the journal as one entry of priority `warning` under the `SYSLOG_IDENTIFIER` `monochromatic-git-policy-cli`,
with the
fields `CLI_GIT_WORKTREE` (the destination),
`CLI_GIT_SLOW_CAUSE` (`uncommitted-changes`,
`git-running`,
`operation-unfinished`,
`not-restorable`,
`stale-index`,
`status-failed`,
`unproven`,
`not-subvolume`,
`other-filesystem`,
`unsupported` or `snapshot-failed`)
and `CLI_GIT_SLOW_DETAIL` (the check's reason as the debug log prints it),
also under `--quiet`.
The entry is one datagram on the journal's native socket,
sent without waiting;
a journal that is absent or full costs the
command nothing,
and `journalctl --identifier=monochromatic-git-policy-cli` lists the entries.
The debug line with the reasons of every step stays (`MONOCHROMATIC_VERBOSE=true`).

Converting a nested worktree:
What a run of it covers is found on the blocking pool and never on the thread that polls the runtime:
`--all` starts Git and scans the linked worktrees,
and grouping the targets into lanes resolves each of them with a synchronous path lookup,
which a filesystem whose lookup stalls would hold before the future's first await,
where no timeout can expire.

`git monochromatic-git-policy-cli convert-worktree-to-subvolume [--dry-run | --abandon] [--jobs <count>] (--all | <worktree>...)` converts a registered
linked worktree that is an ordinary directory into a subvolume at the same path,
several worktrees side by side.
Each worktree has its own administrative directory,
index,
claim and journal,
so the conversions share nothing but the
filesystem and run concurrently within a budget:
the threads the process may run less an eighth of them (at least one,
none
when only two),
no more than the free memory (the system's available memory and the room left in the process's control
groups,
halved) holds at the peak of one conversion,
never more than sixteen (past which the measured speed-up stopped),
and no more than `--jobs` when given.
Output is deterministic:
each worktree's name,
progress lines and verdict come together in input order (`--all` sorts by
path),
the lines of the first unfinished worktree are printed as they happen,
and those of the others are held until every
worktree before them is complete.
Targets that resolve to one worktree are converted one after another by one worker,
so the second finds the first's result.
A conversion that fails after its journal began keeps the journal and does not stop or alter the others;
the closing line
counts the failures and the status is 2.
It refuses when the worktree is locked,
has an `index.lock`,
has an operation in progress,
has a modified tracked file,
has a
process with its working directory or an open file inside it,
has a worktree registered inside it,
or when its converting
name exists without a journal.
A tracked file counts as unmodified when its index entry's size,
times and inode match it.
An entry those fields cannot vouch for (as new as the index file,
or a stale stat in a copied or cloned worktree) is settled
by content in process,
without a Git process:
the file is hashed as a blob (a symbolic link by its target) and compared with the
entry's object name.
Content cannot settle a path that a `filter`,
`ident` or `working-tree-encoding` attribute may apply to,
a file with a carriage
return,
a file over 32 MiB,
or a difference of type,
executable bit or assume-unchanged mark;
each is refused with its reason,
naming the path.
The attribute sources are the tracked `.gitattributes` files from the root down to the file,
`info/attributes`,
the file
`core.attributesFile` names or the default global file,
and the system file,
which is where the Git executable says it is
(`git var GIT_ATTR_SYSTEM`,
learned once per executable and cached) and is off under `GIT_ATTR_NOSYSTEM`;
it is never derived from
the system configuration path,
and when its place cannot be established the worktree is refused.
The same holds for the system configuration file:
both are learned from the real Git the wrapper forwards to,
which the command
resolves itself (the forward-target variable is in the environment of the Git processes the wrapper starts,
never in its own;
see "The wrapper never starts itself as Git"),
and the configuration listing reads its environment entries from the same variables the facts were read from.
With no real Git
found the worktree is refused,
never guessed.
Attribute lines,
patterns and paths are read as bytes:
only space,
tab,
carriage return and line feed separate tokens,
a NUL ends
the text of a line,
a byte order mark on the first line is skipped,
`?` is one byte and only ASCII letters fold;
a spelling the
matcher does not model counts as applying,
and so does a comparison past a fixed work budget.
Every quoted line in an attributes file refuses the paths below the file's directory,
whatever it says,
at every source level.
An attribute source chosen by `GIT_ATTR_SOURCE`,
`attr.tree` or a command-line `--attr-source` or `-c attr.tree` refuses the content
shortcut,
because Git then reads attributes from a tree.
An attributes file that is a named pipe or another file that is not plain is
refused without being opened for waiting.
Macro definitions are followed with reverse edges and a queue.
The stat-only proof is read below a held root as well.
Every tracked file and every `.gitattributes` below the worktree is opened below a held root without following a link (a
directory replaced by a link is refused);
the type and mode of a file are judged on its open handle,
its state is compared
before and after the read,
and the read is bounded at the limit plus one byte,
so the bytes that were hashed are the bytes of
the one file that was judged.
The directories on the way are kept open from one tracked path to the next,
at most 32 by `MOST_DIRECTORIES_HELD` in `src/native/subvolume/fs_beneath_directory_chain.rs`,
and a path deeper than that is walked one directory at a time,
with the kept directories released before that walk starts,
so the proof holds no more descriptors than that bound whatever the tree's depth,
and a path past the bound is answered at a descriptor limit that leaves room for the walk alone
(`src/native/subvolume/fs_beneath_directory_chain.rs`).
The rungs of the add keep the stat-only check.
A real run takes no lock.
It owns the conversion through a numbered claim file in the administrative directory
(`cli-git-convert.owner.<generation>`,
`src/native/lock/single_owner_claim.rs`) that it takes before it reads the journal and gives
back when it ends,
so two commands never read,
write or resume one journal.
The file is written whole in a private temporary
file and published with one exclusive `link` under the next generation number,
so the kernel lets exactly one command create a
generation;
the newest generation is the claim and records the owner's process id,
process birth identity and PID namespace.
A newer generation replaces an older one only when the older records a released claim or an owner that no longer runs with
that birth (a reused PID does not count as alive);
an owner in another PID namespace counts as alive until its file is older
than `FOREIGN_OWNER_PATIENCE`,
and a command that finds it older stops with an error and takes nothing over (the owner's "Refuse and
name it (Recommended)",
"Foreign-owner expiry" below).
The newest generation is never removed (a released one is rewritten by a rename),
older ones are
removed by the command that took the newer,
and a generation published by a command whose listing went stale is withdrawn
when a newer one exists.
A command that finds a live owner skips the worktree,
names the process and the file,
waits for
nothing and writes nothing.
A killed command leaves a claim file that needs no cleanup.
A conversion is refused while a migration of the main checkout that holds the worktree is running,
and a migration is refused
while a conversion of any linked worktree registered in it is running,
each naming the other command's process;
the claims
are read without being taken,
so the pair stays exclusive although the conversion holds `index.lock` only for its last phases.
Release 0.5.20 and older serialized conversions with an advisory kernel lock on `cli-git-convert.lock`,
left in place.
This
release never takes that lock for its own work.
Where the file exists it probes it once without waiting,
skips the worktree
when an older command holds it (two commands must never convert one worktree at once,
whichever release they are),
and removes
the file when nothing holds it,
so the leftover retires itself after the upgrade.
A conversion an older release left
unfinished (its journal in the same format,
and `index.lock` with the journal's token from the start) is finished by this
release by running the command again.
It creates the journal `cli-git-convert` exclusively,
through a temporary file created exclusively under a unique name,
creates
`<path>.cli-git-converting` as a subvolume and records its identity (device,
inode and birth time) in the journal right after.
A kill between those two steps leaves an empty stage the journal does not name yet:
the next run,
and `--abandon`,
take it as this conversion's own because the journal is at the phase that creates the stage and the name holds nothing,
record it or destroy it,
and go on.
An entry at that name is somebody's copy and is still left alone.
Git's `index.lock`,
which is Git's own protocol for writing an index and so cannot be dropped,
is taken with a token only for
the proving pass:
the long first copy runs without it.
It stays through the exchange and the index rewrite and is given back
as soon as the rewrite is journaled,
before the old tree is removed;
a proof that fails gives it back at once,
and a run that
resumes after the exchange takes it again before the rewrite.
A directory at the converting name that the journal has no identity for is foreign:
it is refused,
never synced into,
exchanged or destroyed,
and the abandon leaves it.
It copies the worktree into the stage with cloned data and,
holding `index.lock`,
runs a second pass that must find nothing to
copy;
`Synced` only records that this once held,
so every resume before the swap copies and proves again.
Each of those passes says what it could not carry into the stage,
in the words the migration uses for its
own passes (`subvolume_pass_report.rs`):
every socket or device node it did not recreate,
the owner,
the
permission bits,
the times and the extended attributes the kernel refused,
the destination entries it kept,
and any
file that changed while it was copied and was copied again.
A pass that could not carry an entry refuses before the swap:
a file this user cannot read,
and a
Unix socket or a device node,
which the copy never recreates,
each end the run with the entries named and
"nothing was swapped".
That is the state `--abandon` can still undo,
and a person finishes by stopping
whatever holds the socket,
deleting it and running the command again;
a socket that reaches the old tree after the swap is
named with the file to delete rather than a version to choose,
so no worktree is left in a state no
command can finish.
It reads the stage claims of the repository immediately before that exchange and refuses while one names a
container still being filled below the worktree:
the exchange moves the whole tree,
so the command filling one would lose the place it writes to,
and the refusal names the container,
the process and the rerun once that command has finished.
It exchanges the two directories (`RENAME_EXCHANGE`),
checks that the path is a subvolume root,
is the subvolume whose identity
the journal recorded and holds the same `.git` bytes,
and rewrites the index stat fields;
a refresh that did not install the new
index stops the conversion with the old tree kept.
The rewrite of the indexes of a whole checkout (the migration's,
the rollback's) covers the checkout's index and each linked
worktree's;
each is its own file under its own lock describing its own tree,
so they are rewritten side by side within the host
budget and reported in the order of the checkout's index first and then each worktree's by path.
What the run reads before its first rewrite (the repository's configuration with a synchronous read,
the linked worktrees and the size of each index) is one job of the blocking pool,
never a read on the thread that polls the runtime:
a named pipe where that configuration file is would otherwise block that thread before the future reaches
an await,
where no timeout can expire and every other task of the runtime stops with it.
An index whose rewrite was given up on before it could start is reported as unchanged,
and that is a fact rather than a hope:
the job takes a one-shot claim on its work before its first effect and the report takes the same claim
when it gives up,
so a job the report gave up on does nothing,
while one that took the claim first makes the report say the result is not known.
Before it removes the old tree it reads the process table again for anything holding the old tree and compares the old tree
with the installed one without changing either:
every entry of the old tree must be in the installed tree in the same kind,
with
the same size and modification time for a file,
the same target for a link,
and the same permission bits,
owner,
group and carried
extended attributes (user attributes and POSIX ACLs) for every entry,
and none may be on another device (a nested
subvolume or a mount point,
which is not deleted).
The old tree's own top directory is compared like any entry,
and so is a named pipe or a socket,
whose kind decides only that nothing else about it is compared.
The last four are compared because a change of one of them leaves the size and the
modification time alone,
so nothing else in this comparison can see it,
and the tree that holds it is the one deleted.
The permission bits,
owner and group cost nothing over the walk:
both sides' metadata is already read.
The carried extended attributes cost two attribute listings per entry,
measured on a 24,345-entry tree extracted from this repository's `HEAD` into a disposable
directory:
12.60 µs per entry for the walk as it was,
12.70 with the permission bits,
owner and group,
and 38.02 with the attributes as well,
which is 0.62 s more on that tree,
paid once per conversion.
One entry of the old tree is not demanded:
another command's stage container,
a directory whose own name begins `.cli-git-worktree-copy-`,
which no copy carries and whose payload is that command's work rather than this tree's.
A folder that held nothing else is not demanded either,
because the copy left it out whole,
and neither is a folder another command emptied by removing only its container,
which carries nothing a copy would have carried.
An entry of any other kind whose name merely begins the prefix is the old tree's own,
a tracked file included,
and is compared like any other entry:
skipping such an entry approved deleting it.
A container another command is still filling is not settled by this comparison at all:
the removal phase keeps the old tree before it asks,
and the swap phase refuses to exchange a worktree while one stands below it,
because the exchange would move the container away from the command writing into it.
A regular file whose size and modification time agree is also compared by contents when the change
time of either side is at or after the instant the proving pass began,
both sides being read because either can hold bytes the other does not,
which the journal records,
the instant being taken before the pass rather than after it,
because an edit made while the pass runs carries a change time before the pass ends:
an edit can keep the length and put the modification time back,
and the change time cannot be put back by an ordinary user.
A journal written by an earlier release records no such instant,
and then every regular file's contents are compared.
Only the files the change time flags are read,
so a conversion nothing was changed in reads no contents at all.
Both sides are opened by the plain-file reader,
which waits for no writer and follows no link:
a name
  swapped for a named pipe or a link after the walk read its metadata ends the comparison with the
file named
  instead of holding the conversion's claim for ever.
The measurement behind that choice,
on this repository's largest worktree (28,569,216,018 bytes in 115,067 files,
144,633 entries,
2026-11-09 under the host's build load):
reading one side's contents took 125.7 s for 2.0 GiB at 5.46 ms a file,
so reading both sides of that whole tree would take about 56 minutes,
while the walk that reads no contents took 43.0 s.
It copies nothing from the old tree to the installed one;
a holder or a difference keeps the old tree,
the journal stays,
and the
run says which entry or process it is.
It removes the old tree only when the directory at the name is the one whose identity (device,
inode and birth time) the journal
recorded,
pinned by a descriptor opened below its parent,
deleting beneath that descriptor and removing the name last only while
it still names that object;
a name that holds anything else keeps the journal and ends the run as incomplete.
Then it drops the journal (and `index.lock` if a run that stopped left it).
Nested registrations are read from the common directory whatever it is called (a bare repository included),
and the refusals have
kinds:
only the `index.lock` that holds this command's token and the stage whose identity the journal recorded are exempt,
never a
refusal because of its words.
A rerun completes an unfinished conversion in place whatever phase it stopped at,
including a stop between the exchange and
its journal line;
`--abandon` destroys only the subvolume whose identity the journal recorded,
only before the exchange,
and only while no process has a working directory or an open file inside that stage,
which it reports by
process number,
as the removal phase of the same conversion does.
The scan is taken twice,
the second immediately before the kernel call,
so the window between the
check and the destroy is one check wide;
it cannot be closed without a lock that excludes every process from
the stage,
and a process that changes its working directory into the stage inside that window loses it:
its
shell reads `(deleted)` and its writes there fail with `ENOENT`.
A position a person can be in
it is a shell that runs `cd` into the copy while the command destroys it;
the command cannot tell,
and it
reports the abandonment it made.
A worktree removed after its conversion stopped past the swap leaves the original tree at
`<path>.cli-git-converting`,
a full copy nobody can name once the registration and its journal are gone,
so `--abandon` reaches a path without a registration too:
a subvolume root at `<path>.cli-git-converting` is destroyed and any other directory there is removed as a tree,
which is the original directory a stop after the swap leaves.
The name proves nothing by itself,
so the two identities a conversion may leave there (the original worktree directory and
the subvolume it created)
are recorded beside the worktree in `.cli-git-convert-<name>.leftover-stage.json`,
and the abandon destroys only an object whose identity is one of them:
a directory put at that name instead is kept and named,
and so is a stage whose conversion left no record.
It says what it found before it destroys it,
and refuses while a process is inside that directory.
That identity is read again in the step immediately before the destroy,
so a directory another program renames into that name in that interval is destroyed as if it were the stage.
Binding it needs a
destroy that takes a descriptor,
which the platform does not offer,
so the window stays one check wide.
The person sees the directory they put there gone,
and the stage they moved aside still where they moved it,
under a name
no record names.
The rollback's two directories are read the same way,
immediately before and immediately after its exchange,
and a pair that did not exchange is exchanged back and reported,
so nothing is deleted;
a directory renamed into
the checkout's name inside the last interval is exchanged with the original tree and is back at the old tree's name when
the read after the exchange refuses,
and a kill between the exchange and that read leaves the original tree at the checkout's
name and the person's directory at `<root>.before-subvolume`,
where the next `--rollback` reports the pair it did not
expect and changes nothing.
Both windows need a second program to rename a directory inside a window measured in
microseconds,
which is the accepted risk of other processes changing files,
and neither is a defect of this command alone.

Other processes:
nothing in the command or the program excludes another process from changing the worktree while a command
works on it.
A rename or replacement mid-operation,
a write through an open file or a shared mapping,
and an edit between a
check and the step that relies on it are accepted risk,
stated in the help text,
in one notice on standard error before the
conversion runs,
and in the README;
"Other processes changing files" states it for the whole program.
The guards that cost little stay:
the claim,
the identity checks,
the process table read and the stage claims read
before the swap and before the removal,
and the read-only comparison before the removal.

The journal names the path the conversion began at:
a worktree moved while a conversion is unfinished is refused with both paths.
A journal,
a registration file or a `.git` that is a named pipe or any other file that is not plain ends the run with a reason
instead of blocking while the lock is held.
The same holds for every record either command reads:
the migration reads its journal,
its list of unreadable files,
an index,
the repository's configuration and every registration's `gitdir` through the one plain-file
reader (`foundation/plain_file.rs`),
which opens without waiting and without following a link and takes the type from the open
handle,
so a named pipe at any of them ends that run with the file named instead of waiting for a writer
that never comes while the claim of the checkout is held.
The same reader is what finds the main checkout before the claim is taken:
a `.git` file and an administrative directory's `commondir` are read through it,
so a named pipe at either ends discovery with a reason instead of waiting.
An index is read up to a gigabyte,
the largest index this repository held at 3,097,973 bytes (measured 2026-11-09) and the bound the
merge pass uses;
the record files the migration reads (its journal,
a registration's `gitdir` and the repository's configuration) are read up to a mebibyte,
and a longer one is refused with its path named
rather than read whole;
its list of unreadable files is read line by line from a handle opened the same way,
so a list longer than any fixed bound is still read and the writer and the reader agree on what
it can hold,
because the paths the run keeps are what the read holds.
The conversion reads a worktree's `.git` file and a registration's `gitdir` up to
`GIT_FILE_LIMIT` (64 KiB),
as it did before this work.
A dry run reconciles an exchange the journal has not recorded in memory,
as a real run does,
and writes nothing.
The registrations are listed from the common directory;
a listing or a registration that cannot be read is a refusal.

The migration's journal (`.cli-git-migrate-<name>.journal.json`) is replaced through a sibling file created exclusively
(`O_EXCL`,
mode owner-only) under a name no other file has (the journal's name,
`.tmp`,
the process number and a clock reading),
synchronized,
renamed over the journal,
and the directory synchronized;
an existing path of any kind is refused and never
truncated or followed.
Releases up to 0.5.20 wrote one fixed sibling name with a plain create,
which follows a link planted there.
After a write lands,
the plain files beside the journal under the writer's own names (that fixed name included) are removed,
which is safe because the numbered claim file `.cli-git-migrate-<name>.owner.<generation>` leaves one writer per checkout;
a link,
a directory or any other name stays.
The conversion's journal also records when its proving pass finished,
in milliseconds after the epoch,
so the removal can tell a file changed after the proof from one the copy already carried;
a journal written by an earlier release has no such field.
A rollback deletes the new subvolume only while the name holds a subvolume root,
which is what the migration created there:
a directory that replaced it is refused with both paths and nothing is deleted,
because the removal's fallback would otherwise delete it and whatever it holds.
The journal also records the identity of the subvolume the migration made at that name (device,
inode and,
where the filesystem gives one,
birth time),
in the same map of named values that holds the checkout's identity,
before anything is copied into it,
and the rollback deletes the stage only while that identity still answers for the directory it deletes:
a subvolume another process put at that name in the meantime is refused with the path and the
two identities,
and nothing is deleted.
A journal that records no identity for the stage names no subvolume to delete,
so the rollback is refused there too,
with the path and both ways out (delete the migration's own stage,
or move a foreign one away and clear the journal),
instead of deleting what it cannot name.
The identity of the checkout the migration installed is compared as well,
before the deletion on every path that reaches it,
including a run resumed after the exchange was recorded,
which skips the swap and with it the pair check that guards the exchange;
a directory or a subvolume that replaced it is refused with both paths and both identities.
One window stays open and is written here rather than closed:
another process that puts a subvolume at that name in the moment between the identity check and the
destruction has that subvolume destroyed;
a person sees the rollback finish and the directory they made gone,
and nothing names it,
because the name is one only this command may hold while it runs.
The migration's list of files this user may not read is published the same way:
staged in a sibling created exclusively and mode owner-only,
synchronized,
renamed over the list and the directory synchronized,
so a kill leaves either the earlier list or the complete new one,
never an empty record that the next run would read as nothing recorded.

Migrating the main checkout (`git monochromatic-git-policy-cli migrate-to-subvolume`) is owned the same way,
in every mode (migrate,
`--rollback`,
`--remove-old`):
the command takes a numbered claim file `.cli-git-migrate-<name>.owner.<generation>` in the checkout's parent
directory before it reads the journal and gives it back when it returns,
so a second command on the checkout is refused with
exit status 1,
the owner's process id and the file named,
and nothing changed;
a preview takes no claim.
A preview runs the refusals of the mode it previews and prints them in the same words the real run
uses (`refusals_of_the_mode` in `subvolume_rollback.rs`),
exiting with status 1 when any stands;
only a mode whose refusals are all clear says what it would do.
So `--dry-run --remove-old` on a checkout with no finished migration refuses with the lines the
real run gives instead of promising a deletion the real run refuses.
The refusals a preview cannot reach are the ones a later read of the tree adds while the
carry copies it:
a file that turns out unreadable,
and an entry the carry-back keeps because the destination holds another kind of entry at its path,
because finding either writes files.
Before this release
the migration had no such guard:
a second run resumed the first one's journal with the first one's token.
The migration
still holds Git's own `index.lock` for the main index and for each linked worktree's index (one `O_EXCL` file per index,
never waited for,
a foreign one stops the migration with the file named),
from before its last delta pass to the end of the
index rewrite,
because only Git's lock excludes a Git writer from the exchange;
that reason is written beside the code in
`subvolume_lock.rs`,
`subvolume_index_files.rs` and `subvolume_migrate.rs`.
The rewrite of the index stat fields that runs by
itself (the worktree add's index,
a rollback) takes `index.lock` for one index from the read to the rename.
The identity of the installed subvolume is recorded with the `exchange` step itself,
when the tree is renamed into the
checkout's place,
and the first record wins:
every step between the exchange and the verification is then protected by it,
and a directory that
took the checkout's place later is never blessed with its own identity.
The migration records the identity of the subvolume it installed at the checkout (device and inode,
in the journal's `values`
under `installed-root`,
written with the `verify` step),
and `--remove-old` and `--rollback` refuse while the
directory at the checkout path has another identity:
a checkout moved or replaced by hand since the migration is one
the migration never recorded,
and the carry would write into it and the rollback would exchange it.
A journal written by an earlier release records no identity,
and a command that has none does not refuse on a guess.
A window stays open between the exchange and that record,
and is written here rather than closed:
a kill after the exchange or the rename and before `record_installed` and the `exchange` mark
leaves a journal that records `exchange` as started and holds no `installed-root`,
so the next command has nothing to compare.
If the person then moves the installed subvolume aside and puts another checkout,
also a subvolume,
at the checkout's path,
with a version of a file whose modification time is older than the original tree's copy,
`--rollback` reads the journal as a run with only its bookkeeping left:
it marks the exchange done,
carries the replacement's files into the original tree,
keeping the original tree's newer copy of that file,
exchanges the two directories,
and deletes what stands at the new subvolume's name,
which is the replacement,
so the replacement's version of that file is gone.
A kill alone reaches the state;
a second process replacing the checkout in that moment is what turns it into a loss.
Recording the identity of `<root>.subvol-new` before the exchange would name the same object the exchange installs,
because an exchange moves names and not the object;
that change is not made in this round,
so the window is recorded here as a known limit.
A rollback resumed after its exchange renames what the old tree's name holds to the new subvolume's name and then deletes
it,
so that object's identity is compared with the recorded installed checkout before anything is renamed,
and so is the
object at the new subvolume's name once the rename has happened:
an object that is
not the recorded one is reported with both identities and nothing is renamed or deleted,
whichever of the two names holds it,
a name with nothing at it is not
a refusal (there is nothing to rename or delete),
and a journal with no recorded installed identity has nothing to compare.
A kill
between the exchange and the rename is enough to reach that state;
no second process is needed.
`--remove-old` opens the checkout below its parent without following a link,
compares the identity it reads on that
descriptor with the recorded one,
and carries the old tree's newer files through the descriptor,
so a checkout moved aside
after the check receives nothing;
the name is compared with the held identity again before the old tree is deleted,
and a
difference keeps the old tree,
names both identities and changes nothing.
`--remove-old` carries into the checkout what was written to the old tree after the catch-up,
and that carry leaves out an entry the checkout lacks whose modification time is older than the catch-up,
because that is also the shape of a file the person deleted from the checkout on purpose.
A path where the checkout holds another kind of entry than the old tree does is not carried either:
the pass keeps
the checkout's entry as the newer one and records the path,
so the old tree's directory there,
and the only copy of what it holds,
holds back the deletion the same way,
and the refusal names it.
The command counts those entries,
writes their relative paths to `<parent>/.cli-git-migrate-<name>.left-out.txt` beside the checkout
(through a sibling created exclusively and renamed over the name,
so a link planted there is replaced rather than followed;
the unreadable-file list is written the same way),
and deletes nothing while the list is not empty:
the refusal names the entries of both kinds (up to ten each,
then how many more)
and stops with exit status 1 until the person passes `--discard-left-out`
beside `--remove-old`,
or deletes the old tree themselves.
The flag applies to both deleting modes.
A rollback's carry-back leaves the same kind of entry out when the original tree holds another kind of entry at its
path,
and the new subvolume's directory there,
and the only copy of what it holds,
would go with the subvolume the rollback deletes;
the rollback therefore counts those entries,
names them,
and deletes nothing until the person passes `--discard-left-out` beside `--rollback`,
or deletes the new subvolume themselves.
A rollback writes no list file for them:
the paths are relative to the new subvolume,
and the run refuses before it exchanges anything.
The mutation campaign's evidence directories and the host-wide gate slots (`package/mutation-test/cli-cargo-mutants`) are owned
by claim files:
a second run on a directory is refused,
and a slot is free only when its owner is proven gone,
never by age.
Each also holds the kernel lock that releases before 2026-11-03 waited on,
taken once without waiting after the claim and
released just before it,
only so that a gate from an older checkout still sees the slot as taken;
the reason is written in
`gate-slot.ts`,
`gate-slot-run.ts`,
`legacy-flock.ts` and `campaign-lock.ts`,
and the package README names the gap left by
gates that ran the claim-only source.

Rung 2 needs a clean source on btrfs,
XFS or overlayfs,
with the source and the destination in one clone domain.
The same `--no-checkout` run is followed by `FICLONE` of every tracked file (mode and nanosecond times kept;
links
recreated),
a second comparison of the source with its own index,
the installation of the index,
and the same tree
difference.
A refusal removes nothing,
because a name cannot be checked and unlinked in one step:
the entries the clones created stay,
`git reset --hard` overwrites the tracked ones,
a message counts them and names `git -C <path> status` for any other,
and a
file another process put into the destination is never touched.
The index records the stat fields of the regular files the
wrapper made,
taken from their descriptors;
a cloned link keeps the source's stat fields,
which Git compares by target.
The ignored-state copy then runs as for Git's checkout.

Rung 3 is Git's checkout and the ignored-state copy.
Where the destination's parent is on btrfs and the rung is 2 or 3,
the wrapper first creates the destination as an
empty subvolume (`BTRFS_IOC_SUBVOL_CREATE`),
which Git accepts as an empty directory,
and removes it again if Git leaves
it empty.
No rung runs a hook:
every real-Git start of the wrapper switches hooks off.
The wrapper never deletes or unregisters a worktree,
a registration or a stage it did not make in the running process,
and
the only entry it removes by name is its own lock file,
while the name still holds the file it made.
After Git's `--no-checkout` run it binds the add to the two directories Git made (the destination and its administrative
directory,
proven to name each other below this repository's `worktrees`,
the pointer resolving to the registration without
any symbolic link;
a pointer through a link ends the ladder for the add with a message that names the pointer and says to make
the worktree with a canonical path),
records their identities,
and checks the
binding again before every write into the worktree and before the fallback checkout;
a worktree that was moved,
replaced
or edited meanwhile is not written to,
the add fails with one report,
and nothing is changed.
When the fallback checkout fails,
the worktree and its registration stay as they are,
the command fails with Git's
status,
and one report names the path,
`git -C <path> reset --no-enforce-worktree --hard --no-recurse-submodules` (retry) and
`git worktree remove --force <path>`.
The reset carries `--no-recurse-submodules` exactly as the ladder's own checkout step does
(`worktree_ladder_finish.rs`),
because a person's `submodule.recurse` setting makes Git's reset of a worktree whose checkout never
ran stop at `fatal: could not reset submodule index`
and leave staged deletions behind.
The hatch is in that command because the worktree guard reads a checkout that never ran as a worktree
holding uncommitted changes and refuses a hard reset there,
and the report says what the reset discards;
the sentence comes from `worktree_finish_commands`,
which the interrupted-add report prints too.
Every Git step the ladder starts itself (`git read-tree` of a rung,
the fallback `git reset --hard`) has a
ceiling and is ended with a `SIGKILL` to Git when the ceiling is reached
(`service/wait_ceilings.rs`:
`LADDER_READ_TREE_CEILING`,
the class of a bulk step that can be run twice,
and
`LADDER_CHECKOUT_CEILING`,
five times the longest measured whole checkout),
because those steps run the checkout's
conversion filters and a smudge filter,
a Git LFS process or a filesystem that never answers must not hold the add.
The person's own forwarded command has no ceiling,
whatever word it names:
native Git waits as long as its own work takes,
so a `git read-tree --reset -u` the person typed is never ended,
and the ceiling is chosen only for a step the ladder starts itself.
A step ended at
its ceiling loses nothing of the person's:
it writes the wrapper's own stage or the new worktree,
the source is
never written,
and a lock it held is named by the report of an interrupted `git worktree add`.
The ending is bounded too:
`SIGKILL` goes to Git and to nothing else,
and the wrapper waits `STOP_GRACE` for it to exit and be collected
(`src/native/invocation/forwarding.rs`,
a few seconds,
which only a process the kernel cannot stop uses up);
a Git still inside an uninterruptible call after that is left where it is,
the add fails with the step's words and says that it has not exited,
and nothing waits for the call the ceiling exists to end.
The step keeps this wrapper's process group and terminal,
which a Git the person starts themselves keeps:
a conversion filter that reads `/dev/tty` is then a foreground reader,
not one the line discipline stops with `SIGTTIN`,
and `Ctrl-C` reaches Git as it always does.
The price is the one native Git pays when it is killed:
the filters and helpers Git started are not signalled,
so one of them may still run after the add returned,
and the report says so instead of claiming they ended.
A helper inherits the streams Git had,
and this wrapper cannot close another process's descriptor,
so a caller that captures this command's standard error and waits for its end of
 file keeps waiting while that helper lives,
however bounded the step was;
`Command::output` and the wrapper's own forwarded child are such callers,
and the line a ceiling writes says so and says which of the two things it knows about Git,
naming the helper nowhere because nothing reports which process a filter started.
The record of an add that owes an ignored-state copy lives from before real Git runs until that copy's journal names
its stage:
the rung that took the claim hands it to the copy instead of releasing it,
and no completion marker is written while a copy is still owed,
so a command killed between the end of the checkout and the copy's journal leaves a claim whose owner is dead,
which the next recovery reports by name.
The completion marker is written when an add is settled with no copy left to make,
and a claim released then says nothing more.
A process killed between Git's run and the end of the add leaves the claim and a record;
the next recovery changes
nothing,
prints that report for an add whose destination Git registered (or says that nothing proves a leftover is the
add's when nothing at the recorded path names a registration),
prints the stage it did not remove,
and keeps the record,
the marker and the claim while the worktree is
not proven whole and while that stage still stands:
so every later command that runs the recovery prints the same report again,
on standard error at the ordinary level
(no `MONOCHROMATIC_VERBOSE` is needed,
and `--quiet` does not suppress it),
and the recovery runs before every command that takes the linked-worktree lifecycle:
any command in a linked worktree or a bare repository,
and a creating or served `git worktree` command in the main checkout,
and the records go in the command that finds the worktree whole or gone.
The proof is what the two directories say,
without running Git:
the destination is gone,
its administrative directory holds an index,
no `index.lock` stands there,
and the index decides what a whole checkout holds beside Git's pointer:
an index with entries belongs to a tree with files,
so a destination holding only its `.git` file is not whole,
while an index with no entries belongs to a tree with nothing to write,
so a destination holding only its `.git` file is whole,
which is what a worktree at a commit with an empty tree looks like after every step finished.
A record written before the add was bound carries the destination,
the source worktree and the selection its copy takes,
so the `gitdir:` pointer of the destination's own `.git` file names the registration the proof reads,
and an add killed between Git's registration and that record is reported and kept like a bound one.
A record written before those two fields existed carries the destination alone,
and its worktree is named rather than filled,
because nothing says where its ignored state would come from.
The pointer is read as Git reads it
(`read_gitfile_gently`,
`setup.c`):
on bytes,
everything after `gitdir: ` with only the newlines at its end removed,
so a path that holds a newline is whole and the printed remedy names the
registration it means;
a reader that took the first line would truncate such a path at the newline,
and the
 truncated name can be a directory that exists,
another repository's,
whose lock the printed step would then clear.
The proof cannot see a process filling the destination beside an index that was written before it started
and it compares no file's contents;
Git's own `read-tree -u` and `reset --hard` hold `index.lock` for as long as they write,
so the lock refuses those while they run.
The sequence it cannot judge,
written down here because nothing in the destination reports another process's progress:
a separate process runs `git read-tree HEAD` in the interrupted add's destination,
which installs the index without writing a file,
and then copies one of the target's files and stops,
one the target commit has two of;
the index stands,
no `index.lock` stands,
the destination holds a file beside Git's pointer,
so the proof reads the checkout as whole and the record goes,
although one file the commit names was never written.
No reading of the destination can tell that from a finished checkout,
so the wrapper adds no mechanism for it:
the process that restored the destination partly is the one that knows it was
partial,
and the person reads the same shape here.
The same limit covers an index written by such a process and filled by a copy of
every file,
which the proof also cannot separate from a checkout Git finished.
A stage such a kill left is removed by the next command that runs the recovery,
because it can be a whole snapshot of the checkout with its credentials and a copy of `.git`:
the owner chose "Next command deletes it (Recommended)",
with the words
"The next command deletes a stage only when the wrapper's own record proves it made that stage and its owner is dead,
and prints what it removed."
The two proofs are the record written beside the stage's claim
(`<identifier>.made`,
`worktree_copy_stage_made_object.rs`)
and that record's own accounting of the folder
(`worktree_copy_stage_entries_the_record_accounts_for.rs`).
A copy's own container is not held to that record,
because a copy creates one and records the registration it fills as the step after it:
a container whose binding names the destination it fills is the copy's own and goes,
and so does a container that holds nothing at all,
which is the state a kill between the container's creation and that record leaves,
because an empty folder holds nothing that can be lost
and only the copy's own private claim chose its name;
a container holding anything else is reported and kept.
The claim goes with the stage it names,
and with the records of the add once the destination settles.
A stage name can hold the directory Git made and not a staged copy at all:
the snapshot rung swaps its finished stage with the destination,
and a command killed between that exchange and the removal of the displaced directory
(`settle_displaced`) leaves it there.
Both records the add wrote before that exchange prove what happened
(`worktree_copy_swapped_add_settlement.rs`):
the destination's record holds the identity of the directory Git made and the pointer Git wrote into it,
and the record of the stage's creation holds the snapshot,
so the object at the stage name is the first of them and the object at the destination is the second.
The sweep settles that folder the way the creating command settles it:
Git's pointer goes when it is the one the record holds,
an entry that arrived there meanwhile moves into the worktree it belongs to,
never replacing an entry standing there,
and the folder goes once it is empty;
a folder that keeps anything stays with its claim and its records and is named,
so nothing that could be a person's is removed.
When the worktree's administrative directory holds `index.lock`
(the shape a kill inside the index install leaves),
the command run in that worktree finishes the checkout itself
(`src/native/worktree_copying/worktree_copy_interrupted_checkout.rs`),
because nothing else ever would:
the person's only remedy was to remove the worktree and add it again.
It acts only when every entry standing there holds the very bytes the source holds at that name,
which is what the interrupted add's own copies are,
so Git's own `reset --hard` replaces only those and no file of the person's;
it takes the lock away first,
and only where that file is the published index's own object,
or where the add that made it is proven dead and no process of this machine holds the lock open.
The destination that holds anything else is left exactly as it is,
and its lock is named first,
with the condition for removing it (no Git command running in the worktree),
because `git -C <path> reset --no-enforce-worktree --hard --no-recurse-submodules` cannot start while it stands
(`fatal: Unable to create '.../index.lock': File exists`)
and `git status` in the worktree still works,
so nothing else would say why every write to it fails.
Every path this report prints stands inside a command the person copies,
so it is quoted the way a shell reads a word back
(`crate::shell_quote`,
the one owner of that spelling,
`src/native/worktree/shell_quote.rs`):
bare when every byte is one a shell leaves alone,
single quotes around the rest,
and the `$'...'` form for a control byte or a byte that is not UTF-8.
A path holding `$`,
a backquote,
a quote,
a newline or a leading dash therefore reaches the person's shell as one word and expands nothing,
which JSON quoting did not do:
`rm "/work/$bucket/repo/.git/worktrees/topic/index.lock"` removed the lock of another repository
beneath `/work/other` when `bucket=other` stood in the person's shell.
A record or a completion marker that cannot be
looked up keeps the record and says that the state is unknown.
The same holds for the destination's own pointer,
which is asked three things and not two:
it is there,
it is not,
or the
 directory cannot be searched,
and only its absence says the add left nothing to finish.
A refusal read as absence
 deleted the record and the claim of a worktree that is still registered,
and no later command named it.
A worktree whose destination holds the object the record of the stage's creation made is not sent to rebuild anything:
its snapshot is in place,
so the ignored state the record still names as owed is already there.
The settlement of that add answers no notice of its own
(`settle_interrupted_add`),
and the sweep that proves the exchange says the one line that the snapshot carried the ignored state
(`worktree_copy_swapped_add_settlement.rs`),
rather than naming files that are present.
The cost:
an interrupted or failed add keeps naming the same worktree and stage in every command that
runs the recovery,
until the person finishes or removes it.

`git worktree remove <directory>` with at most one `-f`/`--force` and a non-empty directory name,
for an unlocked linked worktree
that is a subvolume root,
clean unless the person forced the removal,
on a mount that lists `user_subvol_rm_allowed`,
destroys the subvolume and deletes the
administrative directory.
The destroy names the subvolume by its id (`BTRFS_IOC_SNAP_DESTROY_V2` with `BTRFS_SUBVOL_SPEC_BY_ID`,
which an
unprivileged user may call under the same mount option),
and by its name below the held parent directory
(`BTRFS_IOC_SNAP_DESTROY`) on a kernel without that call.
Every other case,
and a refusal by the kernel,
is Git's own removal,
which words its own refusal.

The destroy is not run in the person's command:
it waits for whatever snapshot another add is committing and for the running btrfs transaction,
so the removal renames the worktree's directory out of the way inside its own parent in one `rename(2)`,
writes a durable work item that names the renamed subvolume,
and wakes the commit service,
which deletes it later
(`src/native/worktree/worktree_remove_hand_over.rs`).
The path the person named is gone at once and every file of it is still under a name this wrapper chose,
and the worktree was proven clean and unlocked before either step.
The wake waits `HAND_OVER_GREETING_WAIT` for the service's greeting and never the commit's `START_WAIT`:
the record is written before the wake,
and a person removing a worktree waits for none of what the service does with it,
so a service that greets nothing costs one later wake and never the removal.

The snapshots the wrapper made ahead of an add are taken before that hand-over
(`src/native/worktree/worktree_removal_clears_the_ahead_pool.rs`).
A worktree that was the source of one `git worktree add` holds them in
`<admin>/monochromatic-git-policy-cli/ahead`,
and their members are subvolumes of that worktree,
which the emptying of the
registration refuses,
so a removal that left them behind ended with the files gone and the registration kept,
and the sweep
after a merge reported that state as a worktree left in place.
The pool is this program's own cache of the worktree
being removed:
a member is one copy of it,
and an add that finds no pool makes its own,
exactly as with the shadow pool of a
removed worktree (`worktree_removal_clears_the_pool.rs`).
It is taken out of its own name with one rename,
so a fill
that comes afterwards makes a pool of its own rather than writing into the tree being deleted,
and then member by
member.
A pool that cannot be taken stops the removal while nothing has happened:
the command refuses with exit status 1
and one report that names the worktree,
the folder,
the reason,
and the command that deletes a member
(`btrfs subvolume delete <entry>`),
and says that nothing of the worktree changed.

The wrapper serves the removal only when real Git would remove the same directory,
with the same force:
- The worktree is registered in the repository the command runs in:
  its administrative directory is one of that
  repository's `worktrees/<id>`,
  its `.git` file and the `gitdir` file there name each other,
  and it is not the main
  checkout.
  A worktree that another repository registered is Git's:
  it answers that the name is not a working tree.
- Git resolves the operand first by the last path components of every registered worktree path,
  and by the real path only
  when that gives no unique match (`find_worktree` in Git's `worktree.c`).
  That suffix comparison is byte for byte unless `core.ignorecase` folds the case (`fspathcmp` in Git's `dir.c`),
  and the wrapper picks the worktree the operand names only by those bytes:
  a name that differs from every registered path only by case names no worktree here,
  so it is Git's,
  which applies its own setting.
  The wrapper serves the removal only when no
  registration other than the one this removal deletes ends in the operand at a directory boundary,
  ignoring ASCII case,
  which is at least every match Git's comparison can make under either setting.
  Git's match counts registrations,
  not directories,
  so a second registration that reaches the same directory through a symbolic link is a second answer to Git and the
  name stays Git's.
  An empty operand names no worktree (Git:
  "'' is not a working tree"),
  so it is Git's too.
- An operand that reaches the worktree through a symbolic link names the real directory,
  as Git resolves it:
  Git removes
  the real worktree and leaves the link dangling,
  and so does the wrapper.
  No refusal is needed for a link.
- The `locked` entry of the administrative directory is looked up,
  and only a certain absence (`NotFound`) means unlocked;
  an entry,
  or any other result of the lookup,
  leaves the removal to Git.
- The worktree is clean by the proof of `worktree_clean.rs`,
  unless the person passed `-f` or `--force`.
  Git's own removal makes that proof only when the force is zero
  (`builtin/worktree.c`,
  `remove_worktree`:
  `if (!force) check_clean_worktree(wt, av[0])`),
  and a forced removal deletes the worktree it was given whatever it
  holds,
  so the destroy answers for the same set of files.
  The proof is also the only place Git refuses a populated submodule
  (`validate_no_submodules`,
  called from
  `check_clean_worktree`),
  so a forced removal leaves a populated submodule to the destroy exactly as Git leaves it to
  `remove_dir_recursively`.
  A second force stays Git's own:
  `if (force < 2) reason = worktree_lock_reason(wt)` is what lets `-f -f` remove a locked worktree,
  and the wrapper
  reads one force only.
- No subvolume and no mount point stands below the worktree's root
  (`src/native/worktree/worktree_removal_refuses_a_nested_volume.rs`).
  The kernel refuses `BTRFS_IOC_SNAP_DESTROY` for a subvolume that contains another one,
  so such a worktree can never go
  this way:
  the refusal comes before anything is renamed,
  it names the volume it found and the branch the linked worktree
  there has checked out,
  and it gives the one command that removes that worktree while keeping its branch
  (`git worktree remove <inner>`),
  which refuses while anything there is uncommitted.
  Git's own removal is no answer here:
  it unlinks the
  inner worktree's files and leaves its empty subvolume,
  so the wrapper never hands it such a worktree.
  The proof is the walk the
  emptying makes (`subvolume_remove_walk.rs`),
  which stops at the first boundary and deletes nothing;
  it opens and reads every directory below the
  worktree,
  the ignored ones included,
  because a subvolume can stand anywhere.
  A tree the walk cannot read proves nothing either
  way,
  and the removal is then Git's,
  which is what it was before this proof existed.
  A forced removal makes the same proof:
  the
  destroy cannot take such a worktree whatever the person asks,
  and Git's removal would take the files inside it.

Accepted risk,
stated to the person who runs the command:
the check and the destroy are not one step,
and neither are
Git's check and its deletion.
A write,
a `git worktree lock`,
a rename or a replacement of the worktree or of its administrative directory made by another
process between the last check and the destroy is destroyed or deleted with the worktree,
by this command as by Git,
and
nothing can undo the destroy of a subvolume.
The owner's decision is that the person who runs `git worktree remove` accepts that risk,
and that no machinery is built to
close races that other processes cause.
The command says so on standard error whenever it destroys a subvolume.
Cheap guards that already exist are kept and described here without a promise:
the parent directory,
the worktree directory
and the administrative directory are held open from before the proof of cleanliness (opened below their parents without
following a link or crossing a mount),
the lock,
the identity of the worktree directory,
of its parent and of the
registration,
and the `gitdir` text are looked at again immediately before the destroy (any difference leaves the removal
to Git with nothing changed),
the destroy names the subvolume by the id read from the held directory,
and the registration is
emptied through its held descriptor and removed by name from the held `worktrees` directory,
which succeeds only for an empty
directory.
These narrow the window to a few system calls;
they do not close it.

The remedy the refusal names depends on how far that conversion got:
a copy whose journal is at or past the exchange is finished
by the conversion itself (which resumes,
rewrites the index,
removes the copy and drops the journal),
and `--abandon`
cannot help there because it refuses every conversion at or past its exchange;
a copy whose journal is earlier may be finished
or discarded with `--abandon`,
and a copy with no journal at all is the one `--abandon` discards.
A worktree that has a conversion's copy beside it is not removed when a conversion's record beside the worktree
(`.cli-git-convert-<name>.leftover-stage.json`) proves the copy is a conversion's;
a directory at that name that no record
names is the person's own,
and the removal goes on and leaves it where it is,
because refusing on the suffix alone would block a
legitimate removal with neither command able to clear it.
The record must also still be true:
the object at the stage's name is
compared with the identities the record holds,
so a record whose stage is gone,
or whose name holds another object,
does not stop
the removal,
because the record says what a conversion may have left and not what is there now,
and nothing clears a
record a killed `--abandon` left behind.
The report of the refusal:
the wrapper refuses with
exit status 1 and one report that names the copy,
says nothing was removed,
and gives both commands
(`convert-worktree-to-subvolume <path>` to finish the conversion,
`convert-worktree-to-subvolume --abandon <path>` to
discard the copy),
because the removal deletes the registration and the journal in it is what names that copy.
`git worktree prune` is not seen by the wrapper,
and neither is a `git worktree remove --force` of a worktree that is not a subvolume or whose mount lacks the option,
so those still remove the
worktree and leave the copy behind;
a forced removal the wrapper serves refuses like the option-free one,
because the destroy deletes the registration that names the copy;
`--abandon` reaches such a copy at its recorded identity,
and a copy whose
conversion left no record is kept and named.
The destroy is recorded before the kernel call,
in `<common>/monochromatic-git-policy-cli/recorded-destroys/<id>`,
holding the identity of the registration and
the path of the worktree;
the record is dropped by a removal that finished and by a destroy that failed,
so a command killed in the window
between the destroy and the removal of the registration leaves one decidable state.
The record is marked once the kernel call returned,
and only that mark proves the files are gone.
The mark is a line of
its own at the very start of the record,
so the pathname's bytes,
which are the person's and may hold anything,
can never be read as
one;
a record written by an earlier build starts with the identity instead and is read as one whose kernel call is
not proven,
which leaves the registration alone and is reported.
an absent pathname is not proof,
because a person can move an intact worktree,
and emptying its registration would
leave its `.git` file pointing at nothing.
Both writers of that record replace it the way the migration's lists are written:
a sibling created exclusively under a
name of its own,
then one rename,
so a link planted at a temporary name is never written through.
A directory entry in that
folder that is not a plain file is left alone:
the folder is read on every wrapped command,
so a
named pipe,
a device or a link there would otherwise make a commit wait for a writer that never comes.
A record names the process that wrote it,
with the identity that tells it from a later process of the same number,
and an
unmarked record whose owner may still be running is left untouched:
that owner destroys the tree and marks the record,
and a
second command that read it in that time and forgot it would take away the only place the first can mark its success.
A
marked record is finished even while its owner lives,
because the kernel call it names has returned.
Before a registration is emptied,
its `gitdir` is compared with the path the record holds:
a registration a repair
has pointed at a restored checkout names that checkout,
so it is kept (with its index,
reflog and per-worktree refs) and
the record is dropped,
and the person is told both paths.
A `gitdir` that cannot be read is treated the same way,
because
nothing then proves the registration is this record's,
and a registration whose directory is writable can be emptied through it even while the file cannot be read;
the report names the registration and `git worktree prune`.
A `gitdir` that is not there at all is not:
a registration no back-pointer points at is the state this command's own removal leaves when it empties a registration
and then cannot remove its directory,
so finishing it discards nothing.
Every wrapped command then finishes a marked record:
the snapshots this program made ahead of an add go first
(`src/native/worktree/worktree_removal_clears_the_ahead_pool.rs`),
because they are its own and the walk below refuses every nested subvolume,
the registration is emptied and
removed through descriptors opened below `worktrees`,
only while it is the identity the record names and only when
nothing below it is a nested subvolume or a mount point,
and only after the records it holds are copied into the removal
snapshot the rescue route settles,
so an unshown push failure in it is shown rather than destroyed with it.
A record whose worktree
directory is still there is dropped with nothing changed,
because the destroy did not happen,
and an unmarked record whose
pathname is empty is reported once and dropped with the registration untouched,
naming `git worktree repair <new path>`
for a worktree that was moved and `git worktree prune` for one that is gone.
What still fails is reported with its path and with `git worktree prune` as the way forward.

When the subvolume is destroyed and the registration cannot be removed (an unwritable `worktrees` directory,
a registration
replaced meanwhile),
the command exits with status 2 and prints one report:
the removed worktree,
the leftover
administrative directory,
the reason,
`git -C <repository> worktree prune` (`git --git-dir=<directory> worktree prune` for a
repository without a worktree of its own) with its `--dry-run --verbose` form,
and the manual removal.
It never reports success and never hands the command to Git afterwards,
whose removal would find the directory gone.
Git behaves alike (it carries on after the files and exits nonzero),
and removes the empty `worktrees` directory when the
last registration goes,
as the wrapper does.

The unshown push-failure notices and the unfinished push records of a linked worktree live in its own Git directory,
which the removal deletes,
so before `git worktree remove` or `git worktree prune` runs,
and before the landed sweep removes a worktree by itself,
the wrapper copies them into a snapshot under `<common>/monochromatic-git-policy-cli/removal-snapshots/<instance>`,
last writing the Git directory it was taken from and first recording the process that took it
(`src/native/worktree/worktree_removal_rescue.rs`).
Every wrapped command settles each snapshot once the command that took it has ended:
a Git directory that is gone,
and one that is still registered while its checkout is gone,
promote the records to the notices every worktree shows,
because a worktree without a directory can run no command that would show them;
a checkout that is intact is a removal Git refused,
and the snapshot is discarded;
a command that may still be removing keeps the snapshot,
and so does evidence that cannot be read.
The registration of a worktree whose checkout is gone is left to the person,
which `git worktree prune` removes.
That settle cannot tell a checkout a person moved aside from one a removal destroyed,
because a removal that was cut short
leaves the same two things:
a registration and no checkout.
A removal of one worktree snapshots every worktree that has records,
so a
worktree whose checkout is absent while another worktree's removal runs is settled as removed on the next command:
its
records are promoted and its snapshot deleted.
Nothing the person wrote is lost (the snapshot is a copy,
and the registration keeps
its own records),
and the person sees two things:
after `git worktree repair` the same notice appears again,
because the
registration still holds it,
and an unfinished push is reported as one that may not have finished even though the worktree
still exists and the service still runs it.
Repairing a worktree before another removal runs avoids both.

A commit a reference lock holds in a linked worktree lives in that worktree's own Git directory,
and on the per-worktree reference `refs/worktree/cli-git-service/<batch>`,
which the removal deletes as well,
and the wrapper hands a `--force` removal to Git and cannot look afterwards:
before `git worktree remove` or `git worktree prune` runs,
and before the landed sweep removes a worktree by itself,
every held commit of every linked worktree is kept on `refs/cli-git/kept/<batch>` of the common Git directory,
and the person is told,
before Git runs,
that the branch stays where it is because no command in a removed worktree can move it
(`src/native/worktree/worktree_removal_held_commits.rs`).
The question about that reference and its write run in the common Git directory,
not in the worktree,
because `git worktree prune` exists for the state a person leaves by deleting a checkout,
and Git cannot answer a question asked in a directory that is not there.
A kept reference that already names another commit is left alone and the person is told,
and the commit is kept on `refs/cli-git/kept/<batch>-rescue` beside it,
because a name that holds another commit keeps nothing of this one;
a removal Git then refuses leaves that reference naming the commit the batch still holds.
When `refs/cli-git/kept/<batch>` cannot be written at all (a lock standing on it,
or a reference directory that refuses the write),
the commit is kept on `refs/cli-git/kept/<batch>-rescue` and the person is told that name,
so the command still finishes;
every such write is a create-only one,
so no reference another command wrote is ever replaced.
When neither name can be written,
the command stops before Git runs,
with status 2,
saying which commit and branch are still held and where,
that nothing was removed,
that a `git gc --prune=now` run from another checkout would take those commits until a reference of this repository holds them,
which `git worktree remove` of the worktree that holds one does,
and which lock file stands there with the command that removes a lock nothing holds.
A merge run again never finishes that removal:
a branch that already landed makes the merge up to date,
so it looks at no worktree.

A removal of a directory tree by the migration,
the rollback and the worktree trash (`remove_tree`) opens every directory
below the tree's parent without following a link or crossing a mount,
gives the owner access to a directory that forbids
it,
and refuses before it deletes anything when a directory below the tree is a nested subvolume (its device differs
from the top directory's) or a mount point;
the report names that path and says to move it out,
to delete it with
`btrfs subvolume delete`,
or to unmount it.
A boundary that appears after that first pass is refused again at the directory where it is met,
named with the same ways forward,
and a directory whose listing was removed and that the kernel then refuses as not empty is
looked at again through the descriptor the removal still holds,
so the reason names that boundary or the entries that appeared in it,
which are left where they are because the removal deletes only what it listed.
The settlement of a destination the wrapper made as an empty subvolume removes it only while the name still leads to the
object created (identity recorded right after the creation) and the object is empty;
a replacement is kept with a warning
that names it.
The last `rmdir` names the entry in the held parent and removes only an empty directory,
so a swap by another process in
that instant can remove an empty directory at most,
never data;
that residual is an accepted risk like the one above.

Before staging,
exclude every registered worktree root strictly nested under the source.
A main-checkout source also applies a leave-out rule of directory names,
the same everywhere below the source root,
implemented in `src/native/worktree_copying/worktree_copy_prune.rs`:
a directory that holds a `.git` file or directory is another checkout and is left out whole;
a cache is left out (`.pnpm-store`,
`.gradle`,
`.kotlin`,
`.cache` anywhere but the repository root,
and `incremental` and `zig-global-cache` below a `target` directory);
and a run record is left out while `COPY_RUN_RECORDS` is false,
which it is
(`.monochromatic` below a `node_modules` directory,
`verification` directly under `target`,
`mutants.out`,
`mutants.out.old`,
`playwright-report`,
`test-results`,
and `cache` directly under `.serena`).
A stage container of another command (a directory whose name begins `.cli-git-worktree-copy-`) is left out of every
source's copy,
by that name at any depth,
because it holds the whole ignored state another worktree's copy is filling,
credentials included,
and nothing in the new worktree would name it.
The ignored-state copy a worktree-creating command makes takes that leave-out by name,
while the copy and the comparison of a conversion ask the entry's kind as well,
so a file or a link whose name begins that prefix is the source's own ignored state there.
A folder that holds nothing but such containers is no state of the source either,
so a conversion's copy leaves it out whole instead of handing the new worktree an empty folder nothing names,
and neither is a folder above it that holds nothing else made,
so a conversion of a worktree that holds a stage container and no other ignored state leaves no `.claude/worktrees`
behind for state it never carried;
a folder of the source that holds no entry at all is still copied as it is.
A deletion pass leaves a stage container in the destination where it stands,
because it belongs to the command filling it.
The repository root's own `.cache` is kept,
because tasks read the tools and generated inputs it holds.
Dependencies,
builds and local files are copied.
A linked or bare source copies everything it ignores except its nested registered worktrees.
A file or directory the running user cannot read is left out of every source's copy instead of failing the copy,
and one stderr line names the count and the first three paths:
`monochromatic-git-policy-cli: left out <n> unreadable ignored entries from <source>: <paths>`.
A socket,
a named pipe or a device node is left out of every source's copy instead of failing it,
and one stderr line names the count and the first three paths:
`monochromatic-git-policy-cli: left out <n> ignored entries that cannot be copied into a worktree from <source>: <paths>`,
followed by the one sentence that says a socket,
a named pipe or a device node belongs to one running system and means nothing in a copy,
so the rest of the ignored state was copied without it.
Private stage names are reserved and excluded at every path component.
Stage each destination separately in a mode-`0700` sibling directory so file cloning stays
 on the destination filesystem.
Copy regular files into the stage with an exclusive copy-on-write request whose unsupported-filesystem behavior falls
back to a full copy.
Open every source file with the plain-file reader,
which waits for no writer and follows no link:
a name the walk listed as a file that a build replaced with a named pipe between the two passes is answered at once,
never by an open that waits for a writer that may never come.
Open both sides of a byte comparison and the destination directory an entry is installed into the same way.
A source entry that a build replaced with a named pipe,
a socket or a device node between the walk and its copy holds none of the bytes the listing promised,
so it leaves the manifest and is named with the entries no worktree can hold,
because the new worktree lacks it and nothing else says which entry became one;
a source entry that is no longer there goes the same way without a name.
The settling pass that follows leaves either out by its path,
so a dev server's socket or named pipe never fails an add.
A manifest of at least 16 entries (`PARALLEL_BREAK_EVEN_ENTRIES`) is copied by up to 8 workers of the process's
blocking pool,
bounded by the CPUs the process may use,
over contiguous slices after every directory exists;
the first failure stops the others.
Both numbers are measured:
the copy stage forced to 1,
2,
4 and 8 workers took 1.1,
1.0,
0.7 and 0.7 ms for 30 entries and 397.6,
222.2,
139.4 and 104.4 ms for 9,301,
and cloning 22,718 entries took 1549,
880,
555,
397 and 415 ms at 1,
2,
4,
8 and 16 workers
(`doc/planning/cli-git-shipped-limits-audit.md`,
"Measurement record").
The install and the clone of a package repair use the same break-even.
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
so its content is proved by the source's stamp (size,
modification time,
change time and inode) instead of by reading
both files:
the stamp is read before and after the clone,
must be equal,
and must be at least 20 ms older than the clock,
and the final comparison requires the source's stamp to be unchanged.
No write can leave the change time unchanged,
so a writer to the source during the copy fails the comparison.
A file copied byte by byte,
or whose stamp was too recent to prove anything,
is compared byte for byte.
Transient source mutations that revert before final comparison are outside the guarantee.
Any remaining difference is source instability and blocks destination mutation.

Preflight every existing destination manifest entry before creating any path.
Accept an exact match even when the destination does not ignore that path.
Never overwrite or remove a differing destination entry.
Create absent entries exclusively and retain the Git-created worktree on any failure.
Install a regular file as a hard link to its staged copy,
so it appears atomically with its final bytes and mode;
when the filesystem refuses the link,
fill a file the filesystem has given no name (`O_TMPFILE`) in the destination's own directory,
give it the entry's exact mode before it can be read,
and link it to the destination name only once it holds every byte.
A filesystem without unnamed files is filled under a private name in that directory,
`.monochromatic-git-policy-cli-partial-` followed by a digest of the stage and the entry,
which a resumed copy of the same stage computes again and removes before filling it again;
the name reaches the destination by a rename that does not replace,
else by a hard link whose staging name is removed after.
No file therefore ever holds a destination name before its bytes and mode are whole,
so a copy killed at any instruction leaves nothing at that name,
and the next command installs the entry instead of refusing the destination it finds.
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
`<git-common-dir>/monochromatic-git-policy-cli-worktree-copy/v1`.
Journal writes use a private no-follow temporary file and an atomic rename,
and no synchronization
(see "The records an add writes for itself").
The journal record is rewritten only when the transaction changes phase.
Selected-entry intents and proven post-creation identities are appended to a private install log inside the stage,
one line per bounded batch of manifest entries:
the batch's absent paths are claimed before any of them is created,
and their identities are recorded after.
Installation cost therefore grows linearly with the entry count.
A trailing log fragment without a line terminator is an unfinished append;
An installation writes one record of its destination and that destination's identity before its first entry,
`<identifier>.unfilled` beside the journal,
naming the worktree,
the source and the owner that wrote it,
and removes it when the installation completes.
An installation that ends without finishing rewrites that record's reason with what ended it,
so every later command prints which entry differs or which step failed,
not only that files are missing.
Every command that runs the recovery names a worktree whose record's owner is dead and whose destination is still that
object,
with the `git worktree remove --force` and same `git worktree add` commands that settle it,
and removes the record once the worktree is gone or a different object stands at the path the record wrote.
`git worktree move` renames a worktree's directory and keeps its object,
so a moved worktree is still the one the record names:
the record is kept and its line and commands name the path the registration gives it now.
A readiness check holds a copy back from a source that is itself the destination of a record that still stands,
whether that worktree was moved or not.
A registration that cannot be read for a moment proves nothing about the worktree it registered,
so the record it might name is kept and named by a later pass,
never settled on a guess.
No command a person runs fills an existing worktree,
and none is planned:
the owner answered a request for one (2026-11-10) with "No,
remove and add again",
so removing the worktree and adding it again stays the only remedy a person has.
What recovery makes is the copy a killed add never made,
from the source and the selection that add recorded,
and nothing else.
An add whose own Git was refused at a destination that carries another command's registration proof settles its own
record and says so,
instead of promising a later command's report of a record nothing keeps.
recovery ignores it and truncates it before appending.
A complete line whose bytes are not UTF-8 is corruption:
reading it with replacement characters would claim a path the writer never named,
so the log is refused by name and kept.
A completion phase makes stage removal and journal removal recoverable in either crash order.
Staging takes minutes and gigabytes,
and the journal is written only after it,
so the stage's name is recorded first:
a claim file `<id>.stage` beside the journals names the stage container before the container is created,
with the same temporary file and atomic rename as a journal write,
and no synchronization of its own
(see "The records an add writes for itself" below).
The claim is removed once a journal names the stage,
or once a failed copy has removed the stage.
Every item names its owner,
and nothing serializes the commands that read them.
A journal is `<id>.<owner>.json`,
where `<owner>` is the owner's process id,
birth identity and PID namespace as hexadecimal
text.
A stage claim holds the container path on its first line,
`containerHex=<the path's bytes in lowercase hexadecimal>` on the next,
and `owner=<owner>` last;
the first line is written only when text spells the whole path and it fits one line,
so a claim whose path is not text refuses in a reader that knows only that line.
The first line is what the release installed before `containerHex` reads.
A reader takes the bytes line only as the first line or the line after the text line,
and requires those two lines to name one path when both are there,
because the release installed before the bytes line wrote a container path verbatim:
a destination parent holding a line break then spelled lines of the record itself.
A claim of any other shape is refused and kept,
never acted on.
The record of the object a stage's creation made is `<identifier>.made` beside the claim:
the container's bytes,
the object's identity and the instant after which nothing of the wrapper's is written into that folder,
one `key=value` field per line,
written atomically like a claim.
A record of any other shape proves nothing,
and a folder it names is kept and reported rather than removed.
The identity field is left out of the record a command writes before the container exists,
because an object's identity cannot be read before the object does;
such a record names the container and the instant,
and it is the whole evidence a kill in the instant between the creation of a snapshot and the record of its identity
leaves.
That record also names the source the snapshot is taken from,
in `snapshotParent=<the source subvolume's UUID in lowercase hexadecimal>`,
and the transaction its creation must follow,
in `after=<the generation of the subvolume holding the journal directory>`,
read from the last flushed write before this record,
which is the binding record the add writes immediately before the snapshot rung,
because the filesystem writes both facts into every subvolume itself
(`BTRFS_IOC_GET_SUBVOL_INFO`):
the subvolume a snapshot was taken from,
and the transaction that created it.
A later command removes that folder only when it is a btrfs subvolume root,
taken from the recorded source,
and created in a transaction later than the recorded one,
so a plain directory another party put at that name,
and a snapshot another party took before the record,
is kept and named.
The owner answered the choice between naming such a folder for ever and keeping the automatic removal with
"Keep removing,
fix the check" (2026-11-11):
nothing at the name is deleted on the shape of a folder alone.
What this proves is when the folder was made and which subvolume it was taken from,
never who made it:
a snapshot another party makes between that read and this command's own creation carries a later transaction too,
and it must then also be renamed onto the reserved stage name before the reaping runs.
Every temporary file is `<item>.<owner>.<unique>.tmp`,
with the suffix appended to the item's own path bytes and never to a spelling of them,
so a killed write of this record is a file of that shape and is swept when its writer is dead.

### The records an add writes for itself

An add's own bookkeeping is published whole and flushed by nobody.
That is the stage claim (`<id>.stage`),
the destination record beside it (`<id>.add`,
written planned before real Git runs,
rewritten when the add names its Git and when the binding is recorded),
the proof in the new registration's administrative directory of which command made it
(`monochromatic-git-policy-cli-copy-owner`),
and the removals the recovery makes when it settles them.
Each is a private file created exclusively beside its final name and renamed over it,
so a reader sees the whole record or none,
and the filesystem makes it durable with its next commit.
The reason is the rule above,
"Local durability follows what the remote already holds":
what these records guard is a command that was killed,
whose writes the page cache keeps,
and native Git flushes nothing for the worktree add they serve
(measured 2026-11-11:
a native `git worktree add` of this repository's tree makes no `fsync` at all,
while the same add through the wrapper made twelve,
43 s of its 82 s under a load average of 95,
`doc/planning/cli-git-worktree-add-records-flushed-by-nobody.md`).
A flush of any of them is a commit of the whole btrfs filesystem,
which waits behind every other writer's data,
and it stands on the add's waiting path.
What a power cut can therefore cost is the record,
never a byte of a person's:
a claim whose record is absent leaves the wrapper's own stage to the recovery's copy-stage rules and the destination
unnamed,
and the worktree Git registered,
with everything a person put in it,
stands as it was (`worktree_copy_stage_claim_tests.rs`).
The completion marker keeps its flush:
recovery removes a registered worktree whose add has no such record,
so a lost marker would remove a worktree a person was told about
(`worktree_copy_stage_claim.rs`).
The same holds for a copy's own bookkeeping:
its journal,
the journal's adoption by a command that takes over a dead owner's,
its stage binding,
each batch line of its install log,
the record of an unfilled worktree,
and the removals that settle them.
What they prove is what the copy wrote into a worktree Git has just made,
whose every byte the source still holds,
so a power cut costs the rollback of a partial installation and the notice that names it,
never a byte of a person's.
An install log that is cut by a power cut is read as its complete lines and truncated before the next append,
so the record stays usable.

Every root a copy record holds is written as its bytes.
The journal carries `destinationRootHex`,
`sourceRootHex`,
`stageContainerHex` and `stageRootHex`,
each the root's bytes in lowercase hexadecimal,
under the `Hex` suffix of `foundation/path_bytes_in_records.rs`;
a reader takes the bytes field when it is there and the text field when it is not.
A path that text spells with no replacement character is written both ways,
so the release installed before the bytes fields still reads the journal.
A path that is not text has no text field,
so that release refuses the journal,
changes nothing and keeps it,
instead of acting on the replacement spelling.
A bytes field that is present and does not decode is damage,
never absence.
An owner is alive when a process with its id and its birth identity exists;
one in a PID namespace the reader cannot see is
alive until its item is older than `FOREIGN_OWNER_PATIENCE` (twice `MERGE_LOCK_CEILING`,
two minutes;
the owner,
2026-11-10,
"Two minutes (Recommended)"),
and past it,
or as soon as the item's own time cannot be measured against this host's clock,
the command that finds the item stops with an
error that names the item and changes nothing (the owner's "Refuse and name it (Recommended)",
"Foreign-owner expiry" below);
one the platform cannot report on is not proven dead,
so its item is kept.
An item that names no owner (a name an earlier release wrote,
or a stranger's file) has none to wait for and counts as dead.
A command that finds a pending journal or stage claim whose owner is alive leaves it whole and goes on:
it never waits for
the owner,
because the owner is an add or a copy in progress.
A command that finds a journal whose owner is dead takes it over by renaming it to `<id>.<its own owner>.json`;
the rename is the whole claim.
The command that loses the race finds the old name gone and leaves the journal to the winner.
If the winner dies,
the journal's name shows a dead owner again and the next command takes it over.
The temporaries of dead writers are swept;
a temporary of a live writer is never removed,
because its writer renames it.

Every later linked-worktree or bare-repository invocation checks for pending journals and stage claims before forwarding
and recovers those of dead owners.
It never waits.
The copy itself is finished only by a command run in the worktree it fills:
staging or installing a source's ignored state costs what that tree costs,
so a command in another worktree never pays for it and never waits while another worktree is filled.
Such a command writes one line naming the worktree whose files are missing,
that the first command run in that worktree makes the copy,
and that this command does not wait for it.
The discards and the keeps stay open to every command,
because they read a few records and remove the files of a transaction nothing may resume;
a worktree that is gone has no command left to run in it.
Recovery validates that journal paths remain canonical,
reading their bytes rather than a spelling of them,
the stage is a private owned directory beside the destination,
and every intended entry exists in the private manifest.
Every pending transaction then reaches an end:

- The repository's own registrations decide whether a destination is still a linked worktree of it:
  one registration under `<common>/worktrees` names the destination by its `gitdir` file,
  and it names it whatever a command is doing to the destination's `.git` file at the time.
  A destination no registration names,
  because it was removed,
  moved,
  pruned,
  or replaced,
  is discarded:
  recovery removes the private stage and journal,
  never touches the destination path,
  and writes one notice line.
- A `gitdir` file that was not read whole proves nothing,
  and neither does a recorded path that cannot be resolved,
  because a command that unlinks,
  truncates or rewrites that file leaves it empty,
  cut short,
  gone or unreadable for as long as it runs,
  and a directory the recorded path crosses can lose its search permission while the registration stays.
  The recorded path is compared as bytes,
  with only the one line feed that ends the file removed,
  because the first byte of a worktree's path can be a space that belongs to it.
  Recovery keeps the stage and the journal and writes one notice line saying where they stay.
- A destination a registration names whose `.git` file does not name that registration is the answer that proves
  nothing,
  because a command that unlinks,
  truncates or rewrites the pointer leaves it missing,
  empty,
  cut short,
  in a symbolic link loop,
  naming another directory,
  a named pipe,
  a device,
  a directory or a file past `GIT_FILE_LIMIT`,
  while the worktree stays registered.
  Recovery keeps the stage and the journal,
  writes one notice line saying that the repository still registers that worktree but the file does not name that
  registration,
  and leaves the copy to the next command that finds the file naming it again.
  The pointer itself is read as bytes,
  as a plain file without blocking and without following a link,
  and never past `GIT_FILE_LIMIT`,
  with every trailing line feed and carriage return stripped as Git's `read_gitfile_raw` strips them,
  so a pointer a Windows tool ended with a carriage return and a line feed still names the directory it holds;
  a named pipe at that name would stop its reader until a writer comes,
  and the pending check runs before every forwarded command,
  so one worktree's pipe could stick a command in an unrelated worktree.
  No state of that file and no directory that cannot be listed makes the check fail:
  it answers,
  and the copy stays.
- A transaction whose private stage is gone is discarded the same way,
  with a notice that ignored files in the destination may be incomplete.
- Otherwise installation resumes.
  An existing directory at a selected path the transaction claimed or created is accepted whatever its mode,
  because an interrupted installation applies modes after every entry exists;
  every other existing entry still requires an exact match.
  A resumed installation that fails ends its transaction as a first attempt would,
  and the recovering invocation writes the failure as a notice and continues.
- Completed cleanup is resumed without reinstalling entries.

Before the journals,
recovery makes the copy an interrupted add still owes
(`src/native/worktree_copying/worktree_copy_owed_copy.rs`):
a claim whose owner is dead,
whose record names a source and a selection,
whose destination is still a registration of this repository with a whole checkout,
and whose stage no pending journal names,
is filled from that source by the same staging,
journaling and installation a creating command makes.
A destination that holds the object the record of the stage's creation made holds the add's own snapshot,
which carries the source's ignored state with the files:
nothing is owed there and nothing is missing,
so the claim and its records settle in the sweep that follows this pass,
which says the one line that the snapshot carried the ignored state,
removes the directory Git made at the stage name,
and no command stages the source again.
That copy is the worktree's own work:
only a command run in that worktree makes it,
and it says so before the first entry is read,
with one line naming the worktree,
the source,
and that the line stops at the ceiling below.
A command in another worktree writes the line that names the worktree and makes no copy.
Nothing is overwritten,
because the installation accepts an exact entry and refuses a differing one,
so a worktree that gained anything since the add is named instead of written into.
One command copies into one destination:
before it stages,
it creates a marker in the journal root whose name carries its owner,
one exclusive create of `<identifier>.<owner>.taking`,
and a marker whose owner is dead is removed and taken over,
so two commands that find the same dead claim never stage into one name.
The command that makes a copy it did not start holds a ceiling on it
(`src/native/worktree_copying/worktree_copy_ceiling.rs`):
six minutes,
five times the longest copy measured at the scale the wait was found with
(15,151 ignored entries and 150 MB in 72 s),
rounded up to a whole minute.
The walk of the source,
every entry the copy writes,
every batch the installation writes,
and each step between them ask the ceiling,
and the copy a creating command makes for the worktree it just registered holds none,
because the owner allows that add its own time.
A copy that reaches the ceiling ends as a copy that could not be finished:
the installation rolls back what it wrote,
the worktree is named by the record of an unfilled worktree with the sentence that names the ceiling,
and `git worktree remove --force` with the same `git worktree add` line copies the same state again,
with no ceiling on the add.
So no command waits without a bound for state another command left,
and no checkout's size decides how long one does.
A copy that cannot be made ends with the record of an unfilled worktree,
naming the reason the failure gave,
so every later command prints that reason instead of staging the source again;
the worktree is named with `git worktree remove --force <path>` and the same `git worktree add` line,
and the record goes when the worktree is removed or replaced.
That record is written before the claim is settled,
so a command killed between the two leaves the record and the claim that names it,
and a record that cannot be written leaves the claim too:
every later command still names the worktree.
A source that is no longer a worktree of this repository ends the copy the same way,
and so does a source whose directory is gone or is no longer a directory,
which the registration query cannot even be started in;
a refusal that could change leaves the record and the next command asks again,
and a source that is itself being filled leaves the record for the next command.

After the journals,
recovery removes every stage container that a dead owner's claim names and no pending journal names:
a claim whose owner lives is a copy between claiming and journaling and is left whole,
so a copy that died while staging (a signal,
the out-of-memory killer,
a power loss) left it.
For a claim no add recorded,
recovery requires the claim to be a private file naming an absolute path whose last component starts with the reserved
stage prefix,
and the container to be a private directory owned by the user,
removes it with its read-only subdirectories,
writes one notice line naming it,
and removes the claim.
A claim with a destination record beside it was made by an add,
and its stage is removed only with both proofs:
the record `<identifier>.made` beside the claim names the container whose creation it recorded,
its lines being `stage=<the container's bytes in lowercase hexadecimal>`,
`object=<device:inode:birth seconds:birth nanoseconds>` when the identity is recorded,
and
`written=<seconds>.<nanoseconds>`,
with `snapshotParent=<the source's UUID>` and `after=<the transaction the creation must follow>` on a record written
before the snapshot exists,
and the object at the container's name is still that identity,
and no entry of the folder is born after the record's `written` instant
except the ones the wrapper itself writes there after that instant:
the trash parent `.claude/cli-git-trash` with everything below it,
`.claude` itself,
and `.git` with everything below it.
A record that is absent,
that does not read as this release writes it,
that names another container,
that names no object and no source and transaction,
that names one of the pair without the other,
or that is not the identity at the name keeps the stage:
the notice names the stage,
the reason nothing in it was changed,
and the commands that remove it (`btrfs subvolume delete` for a snapshot,
`rm -r` otherwise).
A folder holding an entry born after the recorded instant that is not one of the wrapper's own is kept the same way and
named by the entry,
because nothing there says whether the wrapper's own interrupted write or a person's copy put it there,
and only a person can tell;
the wrapper's own writes after the record are the trash the leave-out makes and Git's index and pointer,
and a write of the difference between the trees (`read-tree -u`) creates entries the record cannot account for,
so a kill during that step leaves a stage the next command reports rather than removes.
The identity is recorded right after the snapshot exists (`take_snapshot`,
`worktree_snapshot.rs`),
because a kill can come at any moment after that,
and raised once the ladder's last write into the stage is done,
right before the swap,
so a kill after the wrapper's own writes removes the folder cleanly.
The accounting reads every entry of the folder before the removal and opens no file and follows no link,
so a stage of a whole checkout costs one lookup per entry on the command that finds the dead claim,
and never a wait;
the removal it decides is the destroy of a subvolume where the mount allows it.
A record that cannot be written never fails the add:
the add is whole,
and the stage a kill leaves is reported instead of removed.
A release removes the record first,
then the destination record,
the completion marker and the claim,
so a command killed in between leaves a claim whose stage is reported,
never a record nothing names.
A record whose claim nothing keeps is removed by the same sweep,
because it proves nothing about any folder once no claim names one.
Two commands that sweep one dead claim at the same moment remove nothing twice and neither fails:
a step that finds an entry another command already removed treats it as removed,
a claim or a record that vanished while it was read is skipped,
and the words that say what went are written by every sweep that still found the folder.
A claim whose container was never created is removed without a notice.
A container that holds the registration its copy was made for,
in the stage binding a copy writes before the first payload entry
(`worktree_copy_stage_binding.rs`),
or one that holds nothing but the parts of that record,
which a kill inside its write leaves,
holds none of the source's ignored files:
the pass before the journals removes it and makes the copy again into that name,
so no claim names it here.
A container that proves neither keeps its claim and is reported,
and a container of an add that holds no binding is judged by the record beside its claim.

Three sites remove a stage container by the path a record names:
finishing a transaction (`remove_pending_journal`,
`worktree_copy_journal_removal.rs`),
reaping an abandoned copy's stage,
and reaping the stage an add left (`reap_judged_stages`,
`worktree_copy_stage_claim_reaping.rs`).
The first two check the recorded path before the removal:
the transaction's stage was validated as a private directory owned by the user with its payload present,
and the abandoned copy's stage is put through `assert_private_path` immediately before `remove_stage`.
Neither check establishes that the directory at that path is the one the copy created.
Another process that renames the original stage elsewhere and puts a private directory of its own at the same path
inside that window makes recovery delete its data,
while the original stage survives at its new location,
and neither ownership nor private permissions tell the two apart.

The stage an add left is the one site where the object itself is proven (`provenance_of`,
`worktree_copy_stage_claim_reaping.rs`):
the record beside the claim names the container's device,
inode and birth time (`worktree_copy_stage_made_object.rs`,
the identity `worktree_copy_rollback.rs` compares for entries the install log holds),
and the reaping compares it with what stands at the name before the removal.
That record is the second durable record this specification said a claim would need,
written right after the snapshot exists rather than before it,
and it is written only where the object is one a snapshot made,
because a copy's stage is a private directory its own claim names and nothing else could be.
Nothing records the object of a copy's stage:
the journal records the container's path,
its payload path,
the destination,
the source,
and the identities of the entries it installed there (`created_entries`),
and a claim records the container's path alone,
written before the container exists.
Recording it would need a new journal field,
read only by this release onward.
Even a recorded identity narrows rather than closes the window,
because the removal resolves the recorded path again after any check.
The removal of a copy's stage is therefore left as it is,
and that window is recorded here as a known limit.

A removal that closed the window would take the directory from a descriptor instead:
opened once with `O_DIRECTORY` and `O_NOFOLLOW`,
its entries deleted through that descriptor,
and the container removed last by name,
where a directory another process put there makes that last step fail
instead of deleting anything inside it.
That is a redesign of `remove_stage`,
of the btrfs subvolume removal it uses,
and of the two callers,
not a check beside them,
and it is not done in this round.

Malformed or unsafe journal,
install-log,
stage claim,
or stage state fails closed without deleting the journal or a path outside the validated private stage.

After the destination settles successfully,
write exactly one human summary line to stderr.
Copy-only failure exits `2`.
If real Git returned nonzero and synchronization also fails,
preserve Git's numeric status;
use `1` when Git ended by signal without a numeric status and the copy failed too.
If Git failed but synchronization succeeds,
preserve its status after writing the successful copy summary.
A command Git was ended by a signal ends the wrapper by that same signal,
which `forwarding::end_as_git_ended` does by restoring the signal's default action and raising it:
the caller's `wait` then sees `WIFSIGNALED` with Git's signal,
and a shell reports 128 + the signal,
exactly as the plain forward reports it by replacing its process with Git.
Where that signal cannot end this process the exit status is 128 + the signal.
So `git log -p | head -1` ends by `SIGPIPE` on this route as it does on native Git
(`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
finding 3).
The coordinated route keeps a process around Git for every index writer,
and a Git it started that a signal ended
ends the wrapper the same way:
`run_forwarded_git`,
the hold retry that runs the command again while the service holds the index,
its fallback when no place for the kept output could be made,
and `from_forwarded` keep the signal,
and `run_coordinated_action` ends by it through `end_as_git_ended`.
A signal this process blocks is unblocked before it is raised,
in the calling thread,
which is the thread `raise` signals:
a parent that blocked the signal before the wrapper ran left it pending and the wrapper exited
normally with the status a shell prints for the death (finding 5 of the round-3 review).
Test:
`a_blocked_terminating_signal_still_ends_by_it`.

The plain child route is one of those:
a command the wrapper may not run again (`git add --patch`) starts its Git once,
through `run_plain`,
and that route kept the death as a status until 2026-11-09T05:40:00-05:00
(`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
finding 3 of the round-2 review).
Before that the death reached the caller as a normal exit with status 128 + the signal,
so `git add` under `kill -TERM <git child>` told a script that maps signals that nothing was signalled.
What still ends with a status on that route is the fetch of a pull,
which ends with `1` as native Git's `cmd_pull` does.
Tests:
`a_git_child_that_a_signal_ended_keeps_its_signal_through_the_coordinated_forward`,
`a_git_child_that_a_signal_ended_keeps_its_signal_when_no_scratch_place_is_given`,
and
`a_git_child_of_a_command_that_is_not_run_again_keeps_its_signal`,
and
`a_signal_to_the_git_of_an_index_writer_ends_the_wrapper_by_that_signal`.

### Package-manager repair of a new worktree

Design,
survey and measurements:
`doc/planning/cli-git-main-worktree-ignored-copy.md`,
section "Package-manager state of a new worktree".
Code:
`src/native/package_repair*.rs`;
the call is in `repair_created` of `src/native/worktree_copying/worktree_copy.rs`.

A copy,
a clone and a snapshot keep links and build state exactly as the source had them,
which is wrong for tools that record the
place they were installed or built at.
After every rung has filled a new worktree and the ignored files are in place,
the wrapper starts each such tool's own offline
repair in the new worktree,
or writes the record that has the commit service run it there after this command returns,
with no lock around it (the repairs touch only the new worktree and read
the source,
so another add never waits for them),
for exactly the destination whose registration the binding confirmed,
and never edits the tool's files itself.
The owner allowed this on 2026-11-03:
"The wrapper may launch pnpm itself for worktree copying",
widened to "all pkg managers".

Which worktrees:
every worktree the command created,
including one a snapshot served and one whose ignored files the copy filled.
Nothing runs for `--no-worktree-copy`,
for `--no-checkout` (the switches are read from the arguments,
so a form the ladder does
not serve is read too),
for a bare repository (nothing was copied),
or after a copy that failed.
The switches are read only before `--` or `--end-of-options`,
the two tokens that end Git's own `parse-options`:
a destination literally named `--no-checkout` stands after `--end-of-options`,
where Git checks the worktree out,
and a repair of that worktree runs as for any other checkout
(`src/native/repair/package_repair_request.rs`,
`RepairSwitch::from_arguments`).

The repairs,
of which the two that rewrite link layouts in place are the service's work whenever a service answers:

- pnpm,
  when the worktree holds `pnpm-lock.yaml` and a copied `node_modules`:
  `pnpm install --offline --frozen-lockfile` in the worktree,
  with the `pnpm` the person's `PATH` resolves.
  In this repository that is the pnpm-shadow,
  which scans manifests,
  warns on standard error,
  and runs the real pnpm;
  its text is
  captured and shown only inside a failure.
  Scripts are not suppressed:
  the repository's own `allowBuilds` decides.
  The add writes this tool in the `relink` record it hands to the commit service,
  which runs it after the add returns,
  and runs it here when no service took the record,
  because a worktree whose copied links were never relinked resolves the source's packages without an error.
  A hand-over no service took keeps its record queued until the add's own run has finished,
  and takes it back then:
  a record removed first would leave nothing to recover the repair from after a kill,
  and a service that drains it meanwhile runs the same install,
  which the proof below confines to the worktree the record names.
- Cargo,
  for every Cargo workspace among the worktree's tracked `Cargo.toml` files (found with `git ls-files`,
  so no walk reads
  `node_modules` or build output):
  `cargo metadata --locked --offline --no-deps` names the workspace root,
  its members,
  its `build_directory` and its
  `target_directory`;
  the same command in the source's counterpart workspace names the source's build directory.
  When Cargo keeps the build directory outside the worktree and the worktree has none,
  the source's is cloned
  (`FICLONE` per file,
  modes,
  times and links kept) to the worktree's;
  under the default layout it is inside `target/`,
  which the
  copy already carried.
  An entry of the source's build directory that cannot be read (a file or directory a test left without read permission) is left
  out of the clone,
  never altered in the source,
  and counted in the success line;
  the workspace is then cleaned as any other,
  and
  Cargo rebuilds what it needs of what is missing.
  Then `cargo clean --offline --package <member>` for every member,
  so registry and git dependencies are reused and
  the worktree's own crates rebuild with the worktree's own paths.
  The workspaces are repaired side by side on as many threads as the host's available parallelism (the standard library reads the
  CPU quota of the control group,
  so a capped session gets its own share),
  each workspace's clone and clean in order,
  because a
  clean needs its own clone first and two workspaces share no build directory.
  The add's clean does not pass `--locked`,
  because a committed `Cargo.lock` that disagrees with its manifest (ten workspaces of this
  repository do) makes Cargo refuse;
  the lock is read before and put back byte for byte after,
  so the new worktree stays as
  checked out.
  That clean runs in a worktree another command may already be working in,
  and nothing read after it can tell Cargo's own rewrite of that lock from a command's or person's newer write,
  so the bytes standing there when the clean ends are kept whole before the checked-out ones go back:
  under the repository's Git directory,
  in `monochromatic-git-policy-cli-displaced-locks`,
  under a directory named by the lock's own path in hexadecimal,
  one file per displacement,
  written and synced before the write that displaces them.
  A lock whose bytes cannot be kept,
  and a repair handed no Git directory to keep them in,
  is left exactly as the clean found it,
  because the checked-out bytes are in the branch and those are not;
  the worktree then shows the difference.
  The add's Cargo finding names every kept copy.
  The prewarm job's clean,
  which runs in the service after the add returned and when the person may be working in the worktree,
  never writes that lock:
  it passes `--locked`,
  a lock Cargo refuses is resolved first by `cargo metadata --offline`,
  and the checked-out bytes go back only when the file still holds exactly what that resolution wrote,
  so a write of the person's lands in their own file and stays there.
  A lock that is not the resolution's and not the checked-out one is left exactly as it is found,
  and the job's diagnostic log says which file it was.
  A clean that stops anyway names the lock,
  the workspace,
  `cargo update --offline --workspace` there and the clean the add would have run,
  in Cargo's own words for whatever stopped it.
  Nothing is cloned or cleaned when the source and the worktree share one build directory,
  when the source has no such
  workspace,
  when the worktree's build directory already holds state,
  or when a clean would reach a directory outside the worktree:
  a copied `target/` (or `target_directory`) that is a link to a directory elsewhere,
  and a copied `node_modules` that is one,
  are the person's own links,
  and a clean or an install through them would delete or rewrite the files of the checkout they name,
  so the repair leaves them alone and the worktree's first build recompiles what it needs.
- Toolchain:
  when a launcher record under `.cache/toolchain-launchers` names a `toolchain.lock.json` other than the worktree's,
  `mpm-shadow install --frozen-lockfile`,
  with `MONOCHROMATIC_WORKTREE_ROOT` set to the worktree.
  The install is lock-constrained:
  it relinks what is installed and downloads the tools the worktree's lock names that are not
  installed yet.
  The owner chose this on 2026-11-03 ("Install online during the add"),
  told that an add can then download and can retire tool
  copies another checkout still links;
  both are accepted consequences.
  The success line keeps mpm-shadow's own summary of what was installed.
  The add writes this tool in the same `relink` record,
  so the install is the service's work when a service answers and the add's own when none does.
  A failed install,
  a timeout or a missing `mpm-shadow` never fails the add:
  the line names the reason and
  `mise run toolchain:install`.
  A service that fails the install leaves the record failed,
  and its journal names the worktree and the reason.
  The worktree keeps the copied launchers until the install runs,
  which is what the add itself left between the copy and the install.
- The `pnpm` the repair starts is the first one on the person's `PATH`,
  which in this repository is the pnpm-shadow.
  The owner decided this on 2026-11-03:
  "The repair should start whatever pnpm is first on path."
  The `relink` record carries that search path,
  so the service starts the program the add resolved,
  and the program's own children read the same path;
  the service's own path may hold no pnpm or another one.
  A relative entry of that path is recorded against the directory the add ran in,
  because the service runs in the directory of the session it was started in,
  and an add whose environment held no path at all records that it held none,
  so the service searches nowhere instead of searching its own:
  with no path,
  the add selects the worktree's own `.cache/toolchain-bin/mpm-shadow`.
  An empty entry is recorded as it stands,
  because it means the working directory of whatever runs,
  which no record can name for another process,
  and no repair starts a program found there.
  A search path whose absolute spelling cannot be written back as one search path
  (a colon separates the entries of a search path and cannot be escaped in one,
  so an entry of a directory whose absolute name holds a colon has no such spelling)
  names no record:
  the add relinks the worktree itself,
  exactly as it does when no service answers,
  and the line the add prints is that relink's own.
  The program a repair starts is the file its own lookup found,
  named by its absolute path,
  so a relative entry of the add's own path starts the add's own file and never the worktree's.
  A record written before this change carries a relative entry as it was written,
  so the service reads that entry from its own directory,
  which is what the release that wrote the record did with it.
  The record's object name is a digest of the worktree's path and the moment the add asked,
  so a worktree removed and added again at the same path never takes an earlier record's answer,
  whether that record settled or failed there.

What the service proves before it relinks:
the record's payload and its repository field must name one directory,
that directory's `.git` must name a registration under the repository's `worktrees`,
that registration's `gitdir` back-link must name the directory's own `.git` file,
and its `commondir` must be this repository.
The pointer is read without waiting for a writer,
so a `.git` that is no regular file is refused at once instead of holding the round,
and the directory is held open for the whole run,
with the proof repeated before every tool,
so a directory replaced at the recorded path between two tools starts no further tool there.
A registration that cannot be read for a reason that may pass (a permission,
  an input-output error)
keeps the record's claim,
which queues it again when the claim runs out,
and a record whose last try cannot read it either is failed.
A record outlives the worktree it was written for
(a service that died holds a claim until the claim runs out),
and its tools rewrite link layouts in place,
so a directory that is no longer this repository's worktree is left exactly as it is,
and the service's journal says so.
A worktree removed and added again at the same path before that record was drained is a registration of the
repository too,
and each tool decides from the directory it finds whether it has anything to relink there,
so the run is the one the new worktree's own record asks for.

Each tool ends in a finding.
A start that does not answer is one of those findings:
the repair waits for the start no longer than `REPAIR_TOOL_START_CEILING`
(`src/native/service/wait_ceilings.rs`,
one minute,
the wait a client grants the service's own start)
because `Command::spawn` returns only after the child's own `exec`,
which can wait in the kernel with no end on a mount that stopped answering,
and the quiet window,
the ceiling and the notice below are all read from a clock the run has not reached while that wait stands
(`src/native/repair/package_repair_start.rs`,
`start_within`).
The start runs on a job of the blocking pool,
so a fork that does not answer holds no worker of the runtime,
and a child that appears after the bound is stopped with its whole process group and collected by a reaper,
because the finding has already said the tool did not start and named the command to run by hand.
A tool that shows no sign of work for the quiet window (600 seconds;
the longest quiet gap measured is 30.3 s,
see "The quiet
windows are labelled unmeasured") is stopped with its whole process group,
and so is a tool that has run `REPAIR_TOOL_CEILING`
(18 minutes) whatever it shows (owner,
2026-11-04,
"Silence plus a ceiling (Recommended)");
the line names the tool,
what it was
doing,
the seconds it ran and the command to run by hand.
A tool still running `REPAIR_TOOL_NOTICE_AFTER` (two seconds) after it started
says once on standard error which program it is,
its arguments,
the worktree it runs in,
and both bounds that end it
(`src/native/repair/package_repair_command.rs`).
That line is a notice and not a bound:
nothing ends at it,
and the quiet window and the ceiling above it are what end a tool.
It exists because an add whose `pnpm` never ended printed its last line at two seconds
and nothing for the next 613 s,
which a person cannot tell from a hang
(`doc/../sonnet-audits/integration-tip-never-stuck/report.md`,
finding 5 of the after-alpha.9 audit).
The quiet window itself is the owner's number and is left as it was.
A repair never fails the add:
the exit status stays Git's.
When all repairs worked,
one line on standard error names the worktree,
each tool's detail and its time,
unless `--quiet` was given.
A repair that did not finish,
including a missing tool,
prints one line per failure,
also under `--quiet`,
that names the reason
and the command to run,
for example `pnpm --dir <worktree> install --offline --frozen-lockfile`.

The three repairs are waits of one task of the runtime,
and neither the type check of each tracked manifest nor the read of the launcher records runs there:
both are jobs of the process's blocking pool,
so a filesystem that stops answering them holds no tool's quiet window or ceiling.
The record that hands the relink over is written and woken by a job of that pool for the same reason:
the task that polls the Cargo repair never waits on a socket or on a write.
That has one limit:
where the system refuses to start a pool thread (`ulimit -u`,
the control group's `pids.max`),
`run_a_job_or_in_turn` runs the job's work on the shared task itself,
because a thread is only a way to do the work sooner.
The person then sees the add wait,
with no tool line printed and no ceiling firing,
until the filesystem behind that job answers;
raising the limit and running the command again is the remedy,
and nothing of the worktree is discarded either way.

Limits:
another process changing files while a worktree is copied or repaired is the person's risk and is not defended against.
Bun,
npm,
uv,
Gradle and Zig are not repaired:
the survey found no state of theirs in this repository that a copy breaks,
except
a Zig project cache (`.zig-cache`,
left out of a copy as a cache) and uv virtual environments (no project of this repository has
one).

### Post-commit

After a successful non-dry-run commit lands
and the landing is complete
(see "Commit protocol"),
use the OID the landing wrote by compare-and-swap.
Never re-read live `HEAD`:
after concurrent landings it can name another invocation's commit.
Run applicable post-commit policies against committed ground truth.
`landedCommitOid()` returns the exact resolved commit;
`candidates()` enumerates the commit's changed entries (its delta against each parent,
every entry of a root commit)
and reads blobs by object ID rather than worktree path.
Only a clean or warning-only result allows auto-push.
A policy or engine failure leaves the commit intact,
blocks push,
returns `2`,
and emits `commit-landed` after the causal events.
Once the landed OID is known,
repository-root or candidate-fact setup failure emits `content-unavailable` plus the same explicit landed state.
The refusal is kept as a verdict in the work queue's `blocked` state,
and it holds four things:
the branch reference the commit landed on,
the commit,
the blocking event's own words
(the policy's configuration name and its message,
a fixed transform's name and message,
or an engine failure's message,
as one line),
and the durable fact that this commit may not be published.
A verdict outlives the command that wrote it,
because the queue pushes a branch and not one commit (see "Auto-push").
A refusal reaches the disk before the command answers:
its record and the `blocked` verdict that settles it are flushed with their directories,
because a machine crash that lost them would leave a refused commit with nothing holding its branch back,
and the next push of the branch would publish it.
The verdict's words are written into the held record before that record is given up,
and a verdict record that cannot be written leaves the held record standing and is said,
because a refusal a held record carries is itself one of the verdicts a push of the branch stops before:
a command killed between the two,
and a storage failure on the second,
then still keep that commit local.
A pass is not flushed:
a lost pass leaves the record the gate found,
which reads as no verdict,
so the commit stays local and every command names the push a person can make
(`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
A verdict that cannot be written leaves the held record where it is,
which still holds the gate's words,
and the command says what it kept;
a push of the branch by the person still sends that commit,
as a person's own push is forwarded unchanged (see "A blocked commit is never published").

### Auto-push

After a commit lands,
the push is a work item of the commit service,
coalesced per branch (see "Commit protocol").
The service pushes the branch its record names and never reads `HEAD`,
because the worktree may switch branches at any instant.
It is the only pusher of the repositories it serves and runs one push per branch at a time,
so the push takes no lock.

A push that delivered can leave the branch's remote-tracking reference behind.
While another Git holds that reference's lock,
Git delivers the push,
prints `cannot lock ref '...'`,
and exits `0`,
so the service would record a push as made while every later question about the branch's remote is answered from a reference that names an older commit (`src/native/push/push_tracking_ref_repair.rs`).
The push repeats itself while that refusal is in its output,
and the repeat writes the reference as a side effect of finding nothing to deliver:
`PUSH_LOCK_RETRIES` is 3 and `PUSH_LOCK_PAUSE` is 250 ms.
A lock that outlives those repeats is the case the repair is for:
the wrapper writes the reference itself after the push answered,
with the reflog message `update by push`,
the same retries,
and the value it read as the expected old value,
so a write never replaces another writer's value.
The write names the reference's full name,
because `git update-ref origin/main <commit>` is accepted and changes nothing
(measured with Git 2.56.0 on 2026-11-11),
and a name that is not under `refs/` is never written.
A lock that refuses the write through every retry is not cleared,
since nothing may clear another Git's lock without a proof that its holder is gone (see "Reference locks left by a Git that was killed");
the push then says that it delivered the commit and that the reference names an older commit until a push or a fetch writes it,
and the state reaches the service's log.

A failed push never fails the commit:
the service leaves a notice that the next command in the repository shows.
A command takes a record under a name only it holds,
by the one atomic step the file system gives,
a rename (`src/native/push/push_failure_notice_claimed_by_one_command.rs`):
the command whose rename wins shows the words,
so two commands that read the record before either showed it print it once,
and its removal can take only its own name,
never a record the service wrote meanwhile.
Every claim takes a name of its own,
and every move is a no-replace rename (`renameat2` with `RENAME_NOREPLACE`),
so a name that is already taken keeps whatever holds it:
taking over an abandoned claim cannot replace a record the same command claimed first,
and a record whose words could not be written goes back to its own name for the next command only when that name is
free,
staying under its claim otherwise,
where a later command finds it once this one is gone
(`Notice::show`).

#### A blocked commit is never published

Owner answer (question tool,
verbatim):
"Hold the push at A (Recommended)",
for the option
"The automatic push stops at the commit before the blocked one,
and every command says which commit is holding the branch back and why.
A person's own git push is unchanged."
So a branch that holds a verdict (see "Post-commit") is pushed only up to the commit before the blocked one:
the refspec's source is the merge base of the blocked commits' first parents,
never the branch's tip.
Which verdicts are on the branch's way out is asked of the repository and never remembered:
a verdict's commit must be an ancestor of the branch,
and must not be an ancestor of the branch's upstream tracking ref,
because Git already holding it in the branch's remote means no push of this branch publishes it.
A verdict on another branch decides nothing,
and one whose commit this repository no longer holds decides nothing either,
since no push can publish an object it does not have.
The base is the newest commit no verdict forbids,
so the commits below the deepest blocked one still reach GitHub in the same push.

The commit a push sends is fixed before that push reads the worktree's verdicts.
A commit's record is written before its branch moves,
so every commit of the branch at that reading has a verdict in it,
while a commit that lands after it has none.
A push that no verdict holds back therefore names that commit and never the branch's reference,
so a commit whose own gate is still running is published by its own push alone,
after its gate has answered.
The one push that gives a branch its upstream names the branch,
because Git sets an upstream only from a refspec that names a branch
(measured with Git 2.56.0:
`git push --set-upstream origin <commit>:refs/heads/new` delivers the commit and leaves `branch.new.merge` unset),
and it may name the branch only while the branch still names the commit that push may send:
a push a verdict stops before,
or one whose branch moved after the reading,
sends that commit alone and leaves the upstream to the first push that can name the branch.
Naming the branch whatever the verdicts said published the refused commit on a branch's first push,
while the result still said the branch was held back.
A reconcile publishes a merge made from the same commit:
a commit that lands while a reconcile runs goes out with its own push,
and the branch that moved is left where it is with the report saying why.

Limits.
A blocked commit that is a merge publishes only the first-parent side below it until it is fixed,
because one refspec names one commit and no commit is invented for the other side;
a blocked commit with no parent has nothing below it,
so that push sends nothing.
A push under `push.default=matching` names no refspec at all,
so a stop cannot be expressed there and the push sends nothing;
such a push also runs only while the branch still names the commit the push read,
because Git resolves the branch itself,
and a branch that moved sends nothing and is tried again.
A commit that lands while Git resolves the branch to match is published by it,
which no refspec of this push can prevent.
A first push that gives a branch its upstream names the branch for the same reason,
so a commit that lands between the question above and Git's own resolution of the branch is published by it;
Git sets an upstream from no refspec that names a commit.
The upstream tracking ref can lag the remote,
so a verdict already on the remote but not yet fetched still holds the branch back;
that costs a push that could have run and publishes nothing it should not.

A held-back push is never reconciled:
the merge a reconcile publishes is made above the commit this push read the branch at,
which still holds the blocked commit,
so it would carry that commit past the stop.
The push therefore stays a plain fast-forward of the base,
and the service keeps trying the branch on its own retry plan,
so the branch resumes without waiting for the next commit once the blocked commit is off its way out.

Every command reads the same verdicts and asks the same questions,
so the line it prints and the push that happened cannot disagree.
For one verdict the line is
"<branch> is held back by the commit <OID>,
because <the verdict's words>.
The automatic push stops before it,
a person's own git push still sends it,
and the push resumes once it is off <branch>'s way out.",
and for several it names each commit with its words and says the push publishes nothing past them.
A question Git cannot answer sends nothing and says so instead of naming a commit it could not place.
A commit whose command died before its post-commit gate ran has no verdict,
so it holds nothing back and the next push of the branch carries it,
as it did before this rule.
A commit whose gate is still running is the other case.
Its held record names a command that is provably running,
so the answer that may keep the commit local is on its way,
and the automatic push stops before that commit as it stops before a refusal,
because sending it would publish a commit the running gate may refuse.
The base is then the merge base of every parent below a refused or a still-running commit,
the line says that the push waits for those checks,
and the push resumes by itself when they answer:
a pass promotes the record and wakes the service,
and a refusal settles the verdict this section already stops before.
The held line every command prints for a live command names the same wait,
because the record is the only place that wait exists.

#### Merges and commits never fail

Owner rules (2026-11-03,
verbatim):

- "Auto merging should never fail."
- "Manual merge should never fail too."
- "Commits should never fail."

Owner correction (2026-11-03,
verbatim):
"never fail doesn't primarily mean retries!"
A failure that can happen is a design defect to remove by construction (one owner,
durable hand-over,
nothing to wait for).
A retry is at most the last resort for a cause outside the system,
and then the cause is named and why it cannot be
designed out is said.

What is outside our control (owner,
question tool,
2026-11-03;
binding):
the wrapper may stop with a clear message,
or retry
where the cause can clear by itself,
and work already handed over stays on disk and the next run finishes it.
- network down (no route,
  DNS,
  reset,
  refused);
- the remote refusing (login or auth,
  a server hook,
  GitHub's GH009,
  branch protection);
- disk full or quota;
- disk I/O errors or a read-only remount;
- a memory or process-count cap set by someone else;
- a kill (the OOM killer,
  a person),
  power loss or a reboot;
- a corrupt repository (missing objects,
  a broken index) caused by a disk fault or another tool;
- the person's signer failing (gpg or ssh-agent);
- the person's editor failing (it exits nonzero),
  leaving an empty message,
  or resolving to no program at all.

Not outside,
so designed out:
a slow remote;
other programs' locks (an IDE or `/usr/bin/git` holding `index.lock` or a ref
lock);
Git's own crashes or bugs;
everything not listed above.

Refusals that may still end a commit or merge with an error:
policy errors at error severity,
an empty message,
a named path an ignore rule matches while no index entry tracks it
(the `explicit-path-commit/named-path-ignored` finding,
"Core finding event"),
a pathspec Git itself refuses for matching nothing
(the `explicit-path-commit/pathspec-matches-nothing` finding,
"Core finding event",
issue 6283),
and a content
conflict no resolver settles (the merge stops with the conflict recorded).
A commit that names a conflicted path is not a refusal:
owner,
verbatim,
"Committing a conflicted path is fine.
Warn and commit
anyway,
even if conflict markers will be committed."
When the owner decides how an amend,
a conclusion or a revert behaves on a head that other commits moved,
the result is stacked
on the new head (owner,
2026-11-03:
"Stack on the new head (Recommended)").

Further owner answers of 2026-11-03 (question tool),
quoted by their chosen labels:
- a commit that meets a live foreign ref lock:
  "Save on a side ref (Recommended)";
- a live foreign `index.lock`:
  "Same as the ref lock (Recommended)":
  a commit lands on the branch and exits 0,
  the index catches up when the lock is free,
  and the command says so;
  a merge saves its result on a side ref,
  exits 0,
  and finishes the checkout when the lock frees,
  carrying edits made meanwhile forward;
- the person's hooks and required filters count as allowed refusals:
  "Allowed refusals (Recommended)";
- a merge over unsaved edits:
  "Carry edits forward (Recommended)";
- a plain `git commit` with unmerged files lands as the merge commit with the markers and warns per file:
  "Warn and finish the merge (Recommended)".

A branch lock a killed Git left behind (owner,
question tool,
2026-11-09,
verbatim)
"Make it,
hold until lock goes (Recommended)":
the commit is written and kept on a side ref,
the command exits `0`,
says the branch has not moved yet and names the lock,
and the next command that finds the lock gone moves the branch
(see "A commit held behind a reference lock").

The acceptance bar for the merge the auto-push makes to reconcile a rejected push,
for a `git merge` the person types,
and for a `git commit`:
nothing the machine or other processes do ends them with a nonzero status.
That covers a held `index.lock`,
a branch that moved underneath the command (the auto-push's own reconcile merge included),
a slow disk,
a stopped child and a deadline.
Such a command waits,
bounded and naming what it waits for,
retries on the new head,
or lands as a merge on the moved head.
It never leaves a half-applied result:
the index and the worktree hold either what they held before the command
or the whole result,
and no `MERGE_HEAD` is left behind by a merge that did not finish.
A content conflict goes through the wrapper's conflict resolution and stops,
as Git stops on a conflict
with the conflicting paths named,
only where no resolution exists.
The ways each of these can still exit nonzero are listed under "Merging" in `README.md`.

The owner on 2026-11-10:
"Dozens of agents at once is normal operation."
That is the load every command is built for,
not an exception to plan around.
A `git worktree add`,
commit,
merge or push that slows down,
waits on another command,
or fails because dozens of agents run Git in this repository at once has a defect in the wrapper;
the number of agents is never the explanation.
An add or a commit whose own work grows with the number of other worktrees,
adds or commits is such a defect.
The owner said the same day that unblocking these jams means fixing the wrapper:
"By "prioritize on unblocking jams" I also meant fix the wrapper."

A `git pull` that is stated to merge (`--no-rebase`,
`pull.rebase` or `branch.<name>.rebase` false,
or `--ff`
 or `--no-ff`
with nothing else stated) and that carries only the options and operands the wrapper reads exactly
(`src/native/merge/pull_isolation.rs`) is split the way Git splits it (`builtin/pull.c`,
`run_fetch` and `run_merge`):
 the wrapper runs
`git fetch --update-head-ok` with the pull's fetch options and operands,
 moves the working tree along when the fetch moved the
checked-out branch (`git read-tree -u -m`,
as `checkout_fast_forward` does),
 and then runs `git merge <merge options> FETCH_HEAD`
through the path of a typed merge,
so the lock wait,
the isolated merge over staged work,
the moved-branch rounds
 and the word-level pass
all apply to it.
The isolated merge takes the one head `FETCH_HEAD` lists for merging (`handle_fetch_head`,
 `builtin/merge.c`)
and words the message from the whole file,
as `git merge FETCH_HEAD` does.
What the settings state is Git's own reading of them,
not the raw text of the configuration:
 the wrapper asks `git config --bool --get` for `branch.<name>.rebase` and then for `pull.rebase`
(`config_get_rebase`),
so the last of several values decides,
as it does for Git,
and every spelling Git reads as false (`00`,
`0x0`,
`0k`,
the empty value)
selects the split,
while a value Git reads as a rebase,
or one it refuses,
leaves the whole pull native.
That start runs only when the command line does not already state a merge,
which `--no-rebase` always does.
Everything else is native Git's whole pull:
a rebase,
`--ff-only`,
`--autostash` and `pull.autostash`,
`pull.ff`,
submodules,
a refspec with a destination,
 an option that is not
read by its exact spelling,
a repository that is mid-operation or has unmerged entries,
an unborn or detached `HEAD`,
 and a
`FETCH_HEAD` that lists no head or several (the pull then runs again from its fetch,
which is idempotent).
When the working tree cannot be moved along,
the report keeps Git's own words
(`fatal: Cannot fast-forward your working tree.` and the `git reset --hard` it names)
and appends the line naming the hatch that lets that reset run here,
because the worktree guard refuses a hard reset in the dirty worktree that made the move fail.
Git's own words say to save the `git diff <old head>` output,
which holds no file Git does not track,
and the reset they offer overwrites exactly the untracked files that made the move fail,
so the wrapper copies every such file beside the Git directory first
(`monochromatic-git-policy-cli/pull-kept/<n>` in the Git directory's data folder,
`pull_untracked_paths_in_the_way.rs`),
names each copy in that line,
and offers the hatch only after what it could not copy
(a directory holding files standing where the new commit writes a file,
a path it could not read)
is named for the person to move out of the worktree first.
A file or a link Git does not track standing on the way to an added path,
where the move needs a folder,
is copied the same way,
because the reset replaces it with that folder;
a component the previous commit tracked is not copied,
since `git diff <old head>` holds its bytes.
The listing names the repository by its absolute paths rather than keeping the command line's global options,
so a relative `-C` or `--git-dir` is read once,
from the directory the person typed the command in.
The copies are never replaced:
a second pull that keeps files makes the next numbered folder.
A pull with nothing stated keeps Git's refusal of divergent branches and its advice.

A merge,
the person's or the auto-push's,
never runs native Git's stash and never trusts the late ref update:
the merge tree comes from `git merge-tree --write-tree`,
the commit from `git commit-tree`,
the checkout moves with
`git merge --ff-only`,
and the whole is preceded by a bounded wait for `index.lock`.

A merge whose paths hold the person's own work carries it forward
(owner's answer "Carry edits forward (Recommended)",
"Merges and commits never fail";
rules "Manual merge should never fail too."
and "never discard user data"):

- The bytes of every path the merge changes that differs from `HEAD` in the worktree or the index,
  untracked files the merge adds included,
  and the ignored untracked files of the person's that stand at those paths
  (`git status` never lists them,
  so they are read from `git ls-files --others --ignored --exclude-standard`
  in `src/native/merge/merge_isolation_ignored_paths.rs`),
  are written as objects with `git hash-object -w`,
  the clean-filtered form beside the exact bytes,
  and one `git commit-tree` holds them all under slot names on `refs/cli-git/carried/<n>`,
  written with an all-zero old value;
  the merge's record names the commit and every path with its two layers.
  The ref goes only once no record that is still there names its commit
  (`a_record_still_names_the_capture` in `src/native/merge/merge_isolation_carry.rs`):
  a merge that saves its result for a later finish writes a second record beside the first,
  both name one capture,
  and the record settled first must not take the ref the other one still needs for the person's bytes.
  The objects are written and the ref exists before the first byte of the checkout changes.
  That set includes the paths whose index entry carries the `assume-unchanged` or `skip-worktree` mark:
  Git's own status,
  `git diff-files`,
  `git diff-index` and `git ls-files --modified` read either mark as "do not compare the
  worktree copy",
  so a marked path is compared with its index entry here,
  by the clean-filtered object of the bytes and
  the mode the worktree entry has
  (`src/native/merge/merge_isolation_marks_the_status_hides.rs`).
  A marked path whose worktree file is gone is no edit of the person's:
  a `skip-worktree` entry with no file is a sparse checkout's own state,
  and a missing worktree file is one Git's own update writes back.
  A tree or a temporary index is not written for this read:
  it runs `git ls-files -v --stage -z` over the merge's own paths and
  `git hash-object --path` for the entries that carry a mark.
  That hash is the clean-filtered form without `-w`,
  so a question is not answered by writing an object,
  and bytes that clean to the object the index records are no edit at all:
  a path marked `text` whose worktree copy holds CRLF endings is not carried,
  measured with Git 2.56.0,
  where `git hash-object --path a.txt` of the CRLF copy equals the committed blob.
  The mark itself is not carried:
  the index records this merge writes hold the mode and the object of a layer and no flags,
  so a path whose entry carried `assume-unchanged` or `skip-worktree` is left with a plain entry,
  and `git status` then lists the person's edit where the mark had hidden it.
  That is a deliberate divergence from Git's own update,
  which clears an `assume-unchanged` mark of a path it writes and keeps a `skip-worktree` one
  (measured with Git 2.56.0 on a fast-forward that writes the marked path:
  `h` becomes `H` and `S` stays `S`);
  the file the merge wrote is materialised either way,
  so a kept `skip-worktree` mark would say "do not check out" about a path the merge just checked out,
  and a flag is not the person's bytes.
- Those paths are put back to `HEAD`'s entries and bytes in one `git update-index` call and per-path renames,
  so `git merge --ff-only` takes them as it always did.
  Every one of their identities is read again before that call and again before each rename,
  so a path written after the capture stops the clear with nothing written to it;
  the captured layers are put back,
  the record and the ref go,
  and the merge is made again on what the file holds now
  (`a_clear_that_met_newer_bytes` in `src/native/merge/merge_isolation_apply.rs`).
  A save is therefore never a refusal and never replaced.
- Each worktree write takes the entry it judged aside under a private name (`RENAME_NOREPLACE`),
  checks the identity and the bytes of what the take caught,
  and places the new entry with a second `RENAME_NOREPLACE`;
  an entry that is not the one that was judged goes back where it was found,
  so the bytes that are checked are exactly the bytes that make way for the merge
  (`src/native/merge/merge_isolation_carry_place.rs`,
  the step the second pass's install already takes).
- After the branch moved,
  each layer is merged with the merge's own:
  the worktree's file form and the index's blob separately,
  so a staged edit stays staged and an unsaved one stays unsaved.
  The index records of that step are written in one `update-index` call and the files after it,
  and the next command accepts the records a killed step wrote as its own,
  so a kill between the two writes the layers it had not written yet.
  A side unchanged on either side takes the other side;
  the bytes of a layer nothing of the merge reaches are written back exactly;
  a text both sides changed goes through `merge_second_pass`,
  the pass this package already runs,
  and a region it cannot settle keeps both sides between `word_merge::keep_both`'s markers.
  A binary file,
  a link,
  a mode the pass cannot take,
  a path the merge deletes and a path the person deleted keep the
  person's side whole and are named on standard error with what to do;
  two sides of different kinds,
  a symbolic link on one and a file on the other,
  cannot be merged as text at all,
  so they keep the person's side whole and are named too;
  a path written after the capture is left alone and named,
  and nothing is written to it before that is decided,
  so the line that names it is true.
  A mode the person set without touching the bytes is theirs as the bytes are:
  the merge's own content is written with their mode,
  and a staged mode stays staged over the merge's object,
  where every permission bit Git does not record (the read and write bits of each of the owner,
  the group and
  everyone,
  and the setuid,
  setgid and sticky bits above them) is kept from the entry the write replaces and
  only the execute bits come from Git's entry
  (`keep_the_permissions_of_the_entry_it_replaces` in `src/native/merge/merge_isolation_carry_place.rs`,
  and `plan_worktree_awaiting` and `plan_index_awaiting` in `src/native/merge/merge_isolation_carry_plans.rs`
  for the execute bits and the staged mode).
  A saved merge's install gives the files it wrote the permission bits the command judged before the carry
  writes over them
  (`give_back_the_permissions_of_the_files_the_install_wrote` in
  `src/native/merge/merge_isolation_recovery_install.rs`),
  because `checkout-index` writes the mode Git's entry implies and the chmod the judgement saw is nowhere else.
  A mode at a path the merge deletes keeps the person's side whole and is named with the others,
  because nothing of the merge's stands there to take it
  (`the_mode_a_deletion_would_leave` in `src/native/merge/merge_isolation_carry_layers.rs`).
  A layer another program wrote keeps its bytes while the other layer of the same path is still carried.
  Every such finding exits 0:
  the merge is made,
  and the person's work is theirs to judge.
- The branch itself may move while the carry runs,
  because another command lands a commit on it.
  The carry then finds the merge's paths holding that commit's content,
  and the record and the ref that keep the person's layers are the only copy left of them.
  So the branch is read once more after the carry,
  and a commit built on this merge is what the captured layers are merged with and written to the files,
  exactly as the recovery of a killed merge carries them onto a moved head
  (`carry_onto_awaiting` in `src/native/merge/merge_isolation_recovery_carry.rs`,
  asked by `src/native/merge/merge_isolation_apply_moved.rs`):
  the later commit stands,
  the person's work is merged with it,
  and the record and the ref go only once that write finished.
  A branch that moved away from the merge is never written over:
  the record and the ref stay,
  the command names them,
  and the next git command in the worktree puts the person's work back.
  A branch that could not be read leaves the same record and ref.
  Every one of these endings exits 0,
  and the line that names the moved commit is printed before the per-path lines of the carry onto it.
- A kill after the clear and before the branch move restores the captured layers,
  because the record says so and the merge did not land;
  a layer written after the capture holds bytes newer than the capture and is left as it is and named,
  the other layer of that path still going back.
  A kill after the move carries them before the record and the ref go (`src/native/merge/merge_isolation_carry_apply.rs`).
  The carry is a function of the captured blobs and the merge commit,
  so recovery converges on the same bytes from any step.

The lines a carried merge prints about a path it kept whole are what the person has to act on,
so every command in them is one they can copy and run as it stands,
and every one of them is true of the checkout the command leaves
(`src/native/merge/merge_isolation_carry_kept_path.rs`,
`src/native/merge/merge_isolation_carry_notice.rs`):

- A path inside a command is quoted the way a shell reads a word back
  (`crate::shell_quote`,
  the one owner of that spelling).
  Unquoted,
  `rm -- gone with space.txt` reached `rm` as three paths and deleted files the merge never named
  (`merge-notice-for-a-deleted-file` round 133,
  finding 1).
- A command that writes the index is named only where it replaces nothing of the person's there.
  `git add` puts the bytes at the path in the index,
  so it is named only when the index holds no entry or the bytes at the path;
  where the index holds a version of theirs that differs from those bytes,
  the line says both layers stand and no command is offered.
  A `git restore` of the merge's version names `--staged` only where the index holds nothing of theirs either,
  and otherwise writes the checkout alone and says that the version in the index stays
  (finding 2 of the same review,
  which measured the staged version dropped from the index).
- A path another program wrote while the carry ran keeps one line,
  the one that says the merge wrote nothing to it.
  The line the carry built from what the path held when it judged it goes,
  because the checkout now holds the newer bytes and two lines about one path would disagree
  (`Report::a_path_written_again`;
  finding 3 of the same review).

A directory that stands where the merge puts a file is not work a carry can hold:
a directory and a file cannot share one path.
Git's own update replaces such a directory,
and everything below it goes with it:
measured,
an untracked `added.txt/nested/file.txt` was gone after a merge that added `added.txt`,
and so was a staged one,
with its index entry,
after a merge the wrapper reported as made.
So the merge stops before the first byte of the worktree,
the index or the branch changes,
when that directory holds anything the merge does not delete itself,
and the line names the path,
the files below it,
and that nothing was changed
(`src/native/merge/merge_isolation_a_directory_the_merge_replaces.rs`).
A directory is the branch's own only when the merge deletes every file below it and every one of those files
still holds what `HEAD` holds,
in the worktree and in the index,
read with one `git diff-index --quiet HEAD -- <path>`;
an answer that read could not give counts as a change,
so a refused read never deletes a file either.
A file the branch deletes whose bytes or index entry or mode the person changed is the person's own,
and no layer of the carry can hold a path the merge puts a file over,
so the merge stops for it too and names the files below it
(a cross-vendor review of round 133,
its first finding:
a staged edit below such a directory went with it,
its index entry included,
while the command exited 0).
The branch's own directory is emptied by the merge's own deletions and the merge's file takes its place,
which is how the finish of a killed merge already judges one (see "Recovery");
the finish reaches a file of the person's there through that path's own standing,
which is never the merge's entry.

A merge over that work that would open an editor opens it here
(`default_edit_option`,
`builtin/merge.c`;
`src/native/merge/merge_isolation_editor.rs`):
the message Git writes is put in `<git-dir>/MERGE_MSG` with the help lines Git adds under it,
the program `git var GIT_EDITOR` names runs on it with this command's terminal and no ceiling,
and its arguments are the words of that command line made here as a shell makes them,
because the wrapper starts no shell and writes no shell text (`merge_isolation_editor_words.rs`);
a line that asks for an expansion,
a tilde,
a glob,
a control operator or a comment is not guessed and stays native Git's whole,
and what it leaves is cleaned with `git stripspace --strip-comments`,
which is `COMMIT_MSG_CLEANUP_ALL`.
A comment character other than `#` opens the message alone,
because the help lines would not be stripped then.
The message is written once per command,
so a round made again on a moved branch keeps what the person typed.
`--no-edit` and `-m` keep the editor shut,
as in Git,
and a merge over a checkout whose own paths are clean that would open an editor stays
native Git's whole,
as does one started with `--edit`.
A message left empty and an editor that fails are refusals that change nothing and exit 1.

A path whose worktree entry is a directory,
and an index that holds unmerged entries,
are not one edit anybody can merge:
the command is native Git's whole,
which refuses such a checkout itself.
A named pipe,
a socket or a device at a path the merge writes or deletes holds no bytes a merge can carry either,
and reading one is worse than refusing it:
opening a named pipe for reading waits until a writer appears,
and nothing promises one,
so the merge would wait without an end while it holds the checkout
(owner's rule "nothing should ever wait forever or indefinitely";
measured with the built wrapper,
a tracked path whose worktree copy was a named pipe kept the merge waiting until a five-second deadline killed it).
Git's own update of the checkout would also remove such an entry without a copy of it,
which takes away a pipe another process still writes into.
So the merge stops before the merge commit is created and before the first byte of the worktree,
the index or the branch changes,
and the line names every such path,
says what stands there,
that nothing was changed,
and that the entries are still in the checkout
(`src/native/merge/merge_isolation_a_special_file_the_merge_changes.rs`).
No worktree read ever opens one:
the bytes of a captured path come from a handle that can only be a plain file,
and a name swapped for a pipe between the type check and the open is refused
(`crate::plain_file`,
`src/native/merge/merge_isolation_carry.rs`).
The move the auto-push makes after a published reconcile merge names its branch instead,
because `git merge --ff-only` moves whichever branch `HEAD` names at the instant it runs:
an ancestry question,
then `git read-tree -u -m <old tip> <merge>` for the index and the worktree,
then `git update-ref -m <message> <branch> <merge> <old tip>` (`src/native/push/auto_push_reconcile_branch_advance.rs`).
A `git switch` that finishes in the window between the check of the branch and the move therefore moves no branch the
reconcile never checked,
and the message it passes is the branch's reflog entry for the move.
A fast-forward that finds the branch moved leaves the checkout as it was or puts the merge's paths back to the moved head's
content,
and the merge is computed again on the moved head (`src/native/merge/merge_isolation_moved.rs`).
A `git switch` that lands between that look and the `read-tree` is put back to the tree this fast-forward started from,
never to the branch the reconcile checked and never to the checkout's whole tree:
`read-tree -u -m` changes only the paths where that tree and the merge's differ,
which are exactly the paths the move wrote,
so a staged edit of the person's on any other path stays as it was.
Nothing is written back when the checkout already shows the merge's tree
(`move_the_named_branch` in `src/native/push/auto_push_reconcile_branch_advance.rs`).
The put-back can still take a file the switched-to branch itself commits,
and only a second process inside that window can make it:
a branch that already holds the merge's content on a path the merge adds or changes,
and an unrelated commit of its own on top,
switched to in the few instructions after the look at the checkout and before the write.
The forward `read-tree -u -m <local> <merge>` then leaves that path alone,
because the index already holds the merge's entry for it,
and the put-back,
whose paths are the ones where `<local>` and `<merge>` differ,
writes the tree the reconcile started from onto it:
the switched-to checkout gets a staged deletion,
or the older content,
of a file its own branch commits.
The branch then moves and the intent record goes,
so nothing in the record says this happened.
No lock and no third look is added for the window,
because a kill alone cannot reach it and a process cannot be kept from switching branches between two of this
command's steps;
a worktree that shows such a staged deletion puts the file back with `git restore -- <path>`.
A push the service stopped or that failed is tried again by the service on its own,
with doubling pauses from
`CLI_GIT_PUSH_RETRY_SECONDS` to 600 seconds,
until it lands or the person must act (`src/native/push/push_retry.rs`,
and what a failure's own words mean in `src/native/push/push_retry_failure.rs`).
One cause is tried a fixed number of times instead of until it passes:
a refusal GitHub answers with `GH006: Protected branch update failed for <ref>`
is the hosting service's own protected-branch rule,
which cleared by itself three times in this repository and can also refuse every push,
so a push answered with it is given `PROTECTED_BRANCH_TRIES` tries in all,
which the default first pause spreads over two and a half minutes,
and is then reported as failed (owner decision,
2026-11-11:
"Make the wrapper auto retry on GH006").
A try the push spent on a stall or an offline failure counts too,
since the plan names a try and not a cause.
A hook decline with no such code anywhere in the push's output stays final.
Its plan is kept in a file whose name is a hash of the branch and of the bytes of the worktree's Git directory
(`src/native/push/push_retry_store.rs`),
so two directories whose paths differ only in a byte that is not text keep a plan each.
A file is removed by a cleanup only when its record was read and names that directory and that branch,
because the key an earlier release wrote was built from the text of the path,
and the text of a name that holds a byte which is not text shows the replacement character:
a directory named with that character itself has the same text,
and its plan is neither done nor cancelled,
so it stays.
A file that cannot be read,
and one that holds no plan,
are kept by the cleanup too;
`load` judges them,
says what it found,
and removes what is no plan,
since deleting a file needs permission on its directory alone.
A branch the person deleted is settled as done with one notice that names the commit and every ref that still holds it:
nothing is sent,
every later attempt would fail the same way,
and when no ref holds the commit any more the notice says so.
The repository decides this,
never the words of the push's output,
which any program a push runs can print:
`rev-parse --verify --quiet <branch>` answers whether the branch is there
(`branch_exists` and `branch_is_gone_words` in `src/native/landing/service_work.rs`).
A search for the refs that hold the commit that fails is not an empty answer:
the notice says that it could not be told and names the commit still.
The person must act for a conflict,
for a login the service does not have,
for a hook on the server that declined the push without a protected-branch code,
and for a protected-branch refusal whose last try the server refused as well
(owner decisions,
2026-11-03 and 2026-11-11:
"Offline is retried,
but auth and server hook declines stop until your next commit.",
"Make the wrapper auto retry on GH006"):
these stop the retry until the person's next commit,
and the failure record says in plain words which one it was,
and for a decline that a push by hand may succeed if the cause has cleared,
since a decline says what the server answered then and nothing about a later try;
for a refusal that outlasted its tries the record also names how many tries the service gave it and how long they
covered,
and the two ways out of a rule that refuses every push.
The service never asks anyone for a login (owner decision,
2026-11-07:
"Never prompt (Recommended)"):
every Git and `git-lfs`
child it starts gets `GIT_ASKPASS` set to the empty string and `GIT_TERMINAL_PROMPT=0` from one list
(`service_child_overlay` in `src/native/invocation/child_environment.rs`,
used for the service's landing and upkeep context and for
each push's environment in `src/native/landing/service_work.rs`).
Git and `git-lfs` try `GIT_ASKPASS`,
`core.askPass` and `SSH_ASKPASS` before they read `GIT_TERMINAL_PROMPT`,
so the terminal
setting alone left the desktop's password dialog open for nobody to answer;
a push with no credential from a helper fails at once
with "terminal prompts disabled",
which is the login failure above.
`SSH_ASKPASS` stays in the environment for OpenSSH.
A command the person typed is forwarded with the prompting Git has.
Evidence and the source trace:
`doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`.
Being offline,
a stall,
an HTTP 5xx answer of the LFS server and a held lock are tried again.
A rejection by the server with a reason no rule names is tried again,
and the journal names its line.
The classification reads Git's own words,
which are cited beside the rules in `src/native/push/push_retry.rs`.
The schedule of the plan is owned by the service's work loop alone:
a round takes what is due for each repository out of it before the
repositories are drained side by side,
hands it to that repository's worker as a value,
and applies the plans and forgets the
workers return,
so the workers share it with no lock (`src/native/landing/service_work.rs`,
`src/native/landing/service_round.rs`).
The plan survives a restart of the service:
it is kept on disk,
a file per branch,
in the instance's own journal
(`src/native/push/push_retry_store.rs`),
rebuilt when the service starts,
and the loop wakes for it with no wrapper command.
A worker keeps its branch's file there itself,
before the words that promise the retry reach the person,
although the loop alone decides when the try is due
(`Replanning::keep_the_plan`):
the loop applies the workers' changes only once every repository of the round is drained,
and a kill before that would leave a notice that says the service keeps trying and nothing that keeps it.
A plan that could not be written makes no promise:
the notice then carries only Git's own words
(`planned_retry`).
The file of a branch that needs no new try goes the same way:
the worker that decided it removes that branch's file as it decides
(`Replanning::forget`),
so the loop writes no plan's file at all and replays nothing onto the journal
(`PushRetries::apply` changes only the list it owns).
Each statement about a branch is therefore made once,
when its worker decides it,
so a round that reaches the journal late cannot take a later round's plan of the same branch away,
and a round whose change never reached a thread cannot leave the plan of a push that already landed behind.
The one forget that rests on a reading of the filesystem,
the proof that a repository is gone,
goes only while the directory is still missing:
the worker asks once more before it removes the file,
and the loop asks again before it drops the branch off its list
(`Replanning::forget_when_unreachable`,
`PushRetries::apply`),
because a repository renamed back by then is reachable and keeps the only scheduled backup of its commits.
The forget carries the plan the reading ended,
which the round took out of the list before it drained the directory,
so a repository that is back when the change lands has that try put back and runs it without waiting for a start
that reads the file;
the two readings are taken at different moments,
so a repository that returns between them keeps its plan on the list even though the worker may already have
removed the file,
and the next drain that plans a try writes the file again.
A file whose record cannot be read is kept and said,
because a read that failed proves nothing about the plan inside it,
while a file that was read and holds no plan is removed
(`push_retry_store.rs`,
`load_reporting`).
A removal that fails is said in the service's own log,
and the file stays:
no later round tries that removal again,
so a service started later finds the plan of a push that has landed and tries it once more.
A start whose read of that journal gave no answer within its ceiling keeps both directories,
writes the plans and notes it makes there,
and reads both again after a pause,
so the pushes an earlier service promised are tried without waiting for a start nothing requires to arrive.
That pause doubles with each attempt whose job outlasted its ceiling,
because such a job still holds a place of the blocking bound until the journal answers,
while an attempt the pool gave no thread for promised that nothing ran and is made again at the same pause.

#### Git steps, tools and checks end only on no progress

The owner decided (2026-11-03,
question tool):
"End Git,
hold hooks (Recommended)" for a stuck step,
"Stop only on no
progress (Recommended)" for a push,
and said "never fail doesn't primarily mean retries!",
"You should (almost) never be
waiting."
and "Invoke real git as rarely as possible."
Picks 4 ("Time limits on Git child steps of a commit") and 9 ("Deadlines for tools and checks that are not Git") of
`doc/planning/cli-git-shipped-limits-audit.md` were made by the coordinator under the owner's delegation ("Pick
options as if
I had picked them and invite veto when I say I'm back") and are open to the owner's veto.
The coordinator added the read-only queries of the auto-push to the same rule.

The rule:
no run ends because it has taken long.
A run ends when it has shown no sign of work for its quiet window.
The signs are the ones the push reads (`src/native/push/push_progress.rs`):
bytes moved by the process and everything it
started,
processor time,
a task able to run or waiting for the disk,
and,
for a run that holds a connection to a remote,
the open
connection itself.
A step that is slow and works is waited for until its ceiling (the owner's later rule,
"nothing should ever wait forever or
indefinitely",
and `doc/planning/cli-git-no-unbounded-waits.md`,
"Branch write and tool ceilings,
2026-11-04");
a write in place
that is not safe to run twice has no ceiling.
The windows are in whole seconds and are not measured (see "The quiet windows are labelled unmeasured").
Linux only:
where `/proc/<pid>/io` cannot be read nothing is measured,
so no quiet window can end a run,
and only the ceilings do
(the ceiling of a step that can be run twice,
of a repair tool,
of the Markdown check,
of the review call and of the whole review).

- A Git step of a commit (`src/native/push/child_watch.rs`,
  `src/native/transaction/transaction_git.rs`) is judged by the push meter on
  the
  process the wrapper started.
  A step that has shown no sign of work for the window,
  60 seconds for a quick query step and 300 seconds for a bulk
  step or
  the seconds of `CLI_GIT_CHILD_WAIT_SECONDS`,
  is asked to end with `SIGTERM`,
  killed after the grace of a stopped
  push,
  and run
  once more,
  which is the coordinator's reading of "its step redone" (open to veto).
  A step that is quiet again in its second run fails with the sentence `<step> showed no sign of work for <n> seconds
  and was
  stopped`,
  which names the variable and what to do.
  Only a step that is safe to run twice is ended:
  the reads,
  the writers of content-addressed objects and the changes
  of a
  private index (`may_be_rerun`).
  A step keeps a ceiling only when `may_be_ended` in `src/native/transaction/transaction_git.rs` says so.
  The owner decided it for
  the branch write on 2026-11-04 ("Never cut a branch write (Recommended)"),
  and the coordinator's follow-up made the owner's
  criterion the rule for every step:
  a step keeps its ceiling exactly when it can run twice,
  decided by cutting it at every
  mutating system call with `SIGKILL` and `SIGTERM`,
  running it again,
  and comparing the end state with an uncut run's
  (`doc/planning/cli-git-no-unbounded-waits.md`,
  "The cut experiment").
  The rule is a whitelist of the exact forms the experiment
  ran,
  parsed by their options and words,
  so a form it did not run has no ceiling:
  `stash list
  --format=%H`,
  `clean --force -d`,
  `read-tree --reset -u <tree>`,
  `update-index` with only `--refresh`,
  only `--index-info` (and
  `-z`),
  or `--add --cacheinfo <triple>`,
  `add` with `--update` or `--all`,
  `checkout-index --all --force`,
  `restore` with
  `--staged`,
  `--worktree`,
  `--source=` and paths after `--`,
  `reset [--quiet] <commit> -- <paths>`,
  and `config [--local] <key>
  <value>`;
  and the steps `RERUNNABLE` names,
  and the readers and writers of unreferenced objects (`rev-list`,
  `merge-base`,
  `log`,
  `ls-tree`,
  `status`,
  `var`,
  `diff-files`,
  `check-attr`,
  `check-ignore`,
  `stripspace`,
  `fmt-merge-msg`,
  `show-ref`,
  `name-rev`,
  `commit-tree`,
  `mktree`).
  `fast-import` is not among them:
  its input can update and checkpoint branches,
  and the experiment ran the
  wrapper's own stream and found a marks lock a cut left that the second run could not pass.
  What bounds it now is the rule for every
  step that cannot run twice:
  on the landing thread it is waited for as long as it lives,
  with the clients behind it told what it is
  stuck in;
  a person's command that started it stops waiting at the class ceiling plus `SHORT_EXTRA_WAIT` (7 seconds,
  the owner's
  "Class ceiling + 7 s (Recommended)") and says the result is not known.
  `RERUNNABLE` names neither `symbolic-ref`,
  `merge-file` nor `init`:
  `symbolic-ref` may be run again as a read (`--quiet`,
  `--short`,
  one name) or,
  on a shadow repository the wrapper made,
  as a write
  of one name to one target with no option (identical in the experiment).
  `merge-file` may be run again and ended only in the form the second pass runs it,
  `--stdout --object-id` over three object names with the conflict style,
  the labels,
  the marker width and the quiet flag:
  that form writes nothing,
  while `merge-file` without `--stdout` writes the merged bytes into its first file,
  a file of the person's.
  Measured on Git 2.56.0 in a throwaway repository on 2026-11-11 under
  `strace --trace=openat,creat,truncate,unlink,rename,mkdir,link,symlink`:
  it opens nothing for writing (only `/dev/null`),
  removes and renames nothing,
  and leaves the repository byte-identical after a second run
  (reflog,
  refs,
  index,
  worktree,
  objects and the Git directory compared).
  `init` is never run again in place (a cut leaves reftable
  tables the second run does not replace:
  50 of 104 cuts differed);
  its one form,
  the argv `shadow_create.rs` runs on a shadow
  request,
  keeps its ceiling because `init_with_git` discards the partial directory and starts a fresh one when the start was cut,
  refused as a finished command,
  or refused before the command ran (`EAGAIN`;
  other I/O errors are not retried),
  at most
  `SHADOW_CREATION_ATTEMPTS` (2) times,
  which the experiment found identical to an uncut run in 104 of 104 cuts;
  a shadow whose `init`
  is cut every time fails its transaction.
  No `update-ref` keeps a ceiling,
  the write of a side ref included
  (`refs/cli-git/kept/<number>`,
  `refs/worktree/cli-git-service/<number>`,
  written by `keep_unlanded` and the scratch deletions in
  `src/native/landing/service_land.rs`):
  a reflog that another Git creates between any check and the write is appended to twice by a
  cut write and its rerun,
  `core.logAllRefUpdates=false` does not stop an append to a log that exists,
  and nothing the wrapper holds
  excludes an ordinary Git.
  The wrapper still passes `--no-deref` and that setting at those call sites,
  as hygiene and not as a proof.
  A write of a side ref is never ended,
  but it is not the branch write that the owner's "Never cut a branch write (Recommended)" covers,
  and "nothing should ever wait forever or indefinitely" holds for it:
  even the landing thread stops waiting at the class ceiling plus `SHORT_EXTRA_WAIT`,
  gets the not-known error,
  and the step is left running (`is_side_ref_write`,
  `leave_running_bound`).
  A `keep_unlanded` that gets it tells the person the commit could not be confirmed on its side ref;
  a scratch deletion ignores it.
  Everything else,
  every other `update-ref`,
  `stash`,
  `merge`,
  `reset`,
  `read-tree`,
  `checkout`,
  `switch`,
  `config --add`,
  `gc` and the rest,
  is never ended.
  On the commit service's landing thread such a step is
  announced at the usual interval and waited for as long as it lives.
  Any other command stops waiting at the bound of the step's
  class plus `SHORT_EXTRA_WAIT` (7 seconds;
  the owner's "Class ceiling + 7 s (Recommended)") and exits non-zero with an error that
  names the step and its command line and says it was left running,
  so the result is not known,
  with `git status`,
  `git log -1`
  and `ps` to check.
  The step writes its output into anonymous files,
  not pipes (`src/native/process/output_capture.rs`),
  so it
  does not die of `SIGPIPE` when the command exits,
  and the system's init or subreaper reaps it.
- The person's hook is never ended by a step the wrapper starts for itself.
  No hook of the person runs below such a step,
  because every Git the wrapper starts carries
  `core.hooksPath` below `/dev/null` and the switch of every hook event (`src/native/invocation/hooks_off.rs`),
  so the watch
  needs no way
  to tell a hook from Git.
  The stop signals the step and the processes it started and never a process beside it.
  A push to a local path or a `file://` remote is the one place a hook of the person runs below a watched step:
  the setting does not follow the push into the remote,
  so the remote's own `receive-pack` runs its `pre-receive`,
  `update` and
  `post-receive` hooks in the push's tree,
  and the stop of a stalled push ends them with it (`src/native/push/bounded_push.rs`).
  Such a stop is reported as what is known and nothing more:
  that the push was stopped,
  that the remote may or may not
  have taken it,
  and whether a retry is planned.
  Nothing asks the remote what it holds,
  because no question can
  settle it:
  the wrapper cannot know which object Git really sent,
  which of a remote's push URLs went first,
  or
  which configuration shaped the attempt,
  and a claim of delivery is one a person stops pushing on.
  The retry settles the
  question instead:
  a push of something the remote already has succeeds,
  and that success clears the record of the
  stopped push for the person (`src/native/push/bounded_push.rs`,
  `service_push_retry_tests.rs`,
  "a_stopped_push_whose_reference_landed_is_settled_by_its_retry_and_claims_nothing").
- The thread that gives a second-pass command its input is bounded by the same window applied to the input:
  the pipe's
  writing end is non-blocking,
  so a full pipe is reported rather than waited in,
  and the writer gives up once the command has taken
  none of its input for the window
  (`write_within_the_window` in `src/native/push/bounded_query.rs`).
  The writer reports what it could not write,
  and a query whose request could not be written whole is a failure that names it,
  never a success made from the prefix the command read:
  a command can keep its own quiet meter busy on another stream while it postpones reading its input.
- The reconcile's own Git children (`git merge-tree --write-tree`,
  every command of the word-level second pass,
  and the
  thread that gives a command its input) are judged like the queries:
  the same meter,
  the push's stall window,
  the service's stop arm and `STOP_GRACE`
  (`run_bounded_git` in `src/native/push/bounded_query.rs`).
  They are not a person's commands,
  and one that never returns held the push worker,
  its renewed claim and the drain of that repository for good.
- The long-lived object reader (`git cat-file --batch`) is judged the same way per request;
  a request that stalls ends
  the
  reader and fails with the same sentence.
  Starting the reader again is not built.
- A read-only Git query before the push (`src/native/push/bounded_query.rs`) has no wall-clock ceiling of its own:
  it is judged by the
  same
  meter with the push's stall window (`CLI_GIT_PUSH_DEADLINE_SECONDS`),
  and `QUERY_CEILING` (120 seconds) no longer
  exists.
  Where nothing can be measured its stall window is the whole time it may run (`query_clock`),
  because a query is read-only and stopping one is safe;
  without that fallback a hung query was waited for for good,
  and the push worker and its repository's drain behind it.
- A package-manager tool of the worktree repair (`src/native/repair/package_repair_command.rs`) is judged on its process
  group's
  leader with a window of 600 seconds,
  and ends with its whole process group;
  it also ends,
  with its whole process group,
  when it
  has run `REPAIR_TOOL_CEILING` (18 minutes),
  whatever it shows.
  An install on a huge repository that works for longer than that is ended and reported with the command to run by hand.
  The tool's output goes to anonymous files,
  so the ceiling covers the draining of it:
  no thread waits for a stream to close,
  a
  helper that keeps an inherited stream open after the leader has exited,
  or that left the process group while holding it,
  holds
  nothing up,
  and a helper still running when the leader has exited is left alone.
- The Markdown linter (`src/native/markdown/markdown_linter_guard.rs`) runs on a thread of the wrapper,
  so its signs are those
  of that
  thread (`src/native/process/thread_quiet.rs`):
  the bytes it read and wrote,
  its processor ticks and its state,
  from
  `/proc/self/task/<tid>/`,
  with a window of 60 seconds.
  A linter thread that shows none is abandoned as before,
  and so is one
  that has been checked one file for `LINT_CEILING` (4 minutes),
  with a reason that names the tool,
  the file and the ceiling.
  The still-waiting notices that long check prints go out on a thread of their own,
  one at a time,
  so a standard error that nobody drains after the pipe filled
  holds neither the window nor the ceiling:
  the waiting thread judges every deadline before it starts a notice,
  never after one,
  and a write that never returns is abandoned with the process while later notices of that command stop,
  which an advisory line may be.
- The standards-review gateway call (`src/native/standards_review/standards_review_client.rs`) has one ceiling over the whole
  call,
  `REVIEW_CALL_CEILING` (1 minute).
  Resolving,
  connecting and
  each read and write of the connection may also be silent for the window,
  10 seconds in the service (`CALL_QUIET_WINDOW`)
  and 30
  seconds for the script's own default (`SCRIPT_QUIET_WINDOW`);
  the transport ends the call itself,
  and its global timeout ends
  it at the ceiling with a sentence that names the call and the command that reviews the commit again.
  The answer is read against `MAX_RESPONSE_BYTES` (four times the model input's cap) frame by frame:
  a body past it ends the call as a failed one while it still arrives,
  and the review's entry records the failure,
  so no answer is ever held whole and the display excerpt of a failure message is not a memory bound.
  The whole review of a commit is waited for `REVIEW_CEILING` (2 minutes) by the service (`run_contained`);
  a review it stopped
  waiting for is counted as outstanding until its thread ends and no other review starts meanwhile (the refused item fails with the
  command that reviews it again),
  because it may still hold the review log;
  its Git reads end on
  no progress or at their class's ceiling,
  its call as above,
  and the local write of the log entry,
  which has no judge of its
  own,
  is covered by the review's ceiling:
  the service stops waiting and the thread ends with the process.
- Not changed:
  the refused-start retry (5 seconds,
  a process-count cap is on the owner's outside list and a retry is
  the allowed
  last resort),
  the wait for the signal guard (library tests only),
  and the notice graces (they time a message,
  not an
  end).

The longest quiet gap of every Git step and tool a command ran is kept (`src/native/process/quiet_ledger.rs`) and written into
the
timing record the command already writes:
`longestQuietMs` (the longest among the steps that ended by themselves) and
`quietMs` (by step name,
in milliseconds,
a step that was ended for being quiet listed apart under a name that ends in
` [ended for being quiet]`) in `monochromatic-git-policy-cli-commit-timing.jsonl`,
and `quiet` in `monochromatic-git-policy-cli-worktree-add-timing.jsonl`.
The resolution of a gap is the sampling interval of the meter,
500 milliseconds,
and a window below that interval can
end a
step that works between two samples,
so no window is set below it outside the controls.
The commit service runs the landing's Git steps in its own process,
so its figures go to its journal:
the `answered`
line of
each request ends with `quiet_ms=<longest among the steps that ended by themselves>` and `quiet_steps="<step>=<ms>;..."`
(`src/native/service/service_timing.rs`).
The landings of one service share the ledger,
so the first answer after a batch's steps holds their figures and the
others of
that batch hold none;
a figure is a gap of some step of the service since the previous answer,
not of one request.
Every Git step a command or a landing starts is also listed with its duration (`src/native/transaction/transaction_step_durations.rs`):
`steps` in the commit timing record (`name`,
and the read-only queries the in-process readers decline count here too,
because a commit that waits on one must name it (`src/native/repo_read/git_metadata.rs`);
the program and the subcommand only;
`offsetMs` from the command's start;
`durationMs`) and `stepsDropped`,
and `steps="<name>@<offset>+<duration>;..."` with `steps_dropped=<count>` at the end of the
service's `answered` line (offsets from the request's acceptance,
the steps since the previous answer,
as with `quiet_steps`).
The list is bounded by `MAX_STEP_RUNS` (64):
the most one commit starts is `LANDING_ATTEMPTS` (3) times 17 Git starts (10 in the
client and 7 in the service,
counted with `strace` over ten commits on 2026-11-07),
rounded up to a power of two.
The commit Git builds in the scratch repository runs with the terminal attached and no watch,
so its entry (`git commit`) is the
only record of the time Git itself took,
signing and the work Git does included,
and the notice's own time and the time a program below the step ran are in `scratchSteps` beside it,
never inside it.
The gateway call's total deadline is `ureq`'s `global` timeout,
set to `REVIEW_CALL_CEILING`;
the transport's other timeouts
(`per_call`,
`send_body`,
`recv_response`,
`recv_body`) default to none in the locked `ureq` 3.5.0 (`Timeouts::default`,
`src/config.rs`;
only the one second wait for a `100 Continue`,
which the call never sends).

The commit service warms the compiled forbidden-strings rules (`src/native/landing/service_rules_precompile.rs`):
after the first landing it sees for a repository,
after a landing that changes a path of the rules,
right after it starts for the repository named in `rules-repository` in its state directory,
and on a `warm` message (`{"type":"warm","version":<n>,"gitDir":<absolute>,"repository":<absolute>}`,
never answered).
The work is a job of the blocking pool and never runs on the landing thread,
and one warm runs at a time.
The client sends `warm` only after `merge`,
`pull`,
`checkout`,
`switch`,
`rebase` or `reset` exits 0 and the digest of the rules text
it would load changed,
only to a service of that worktree that already runs (found by its start arguments),
and never starts a process.
The client keeps compiling on a cache miss.
`src/native/policy/policy_forbidden_strings.rs` decides whether that miss writes the shared cache
(`CachePublisher`,
`cache_access`):
when the cache holds no artifact for the rules text
(`forbidden_strings::compiled_rules_artifact_exists`,
one `stat` of the path the text's digest names)
the command asks a service of that worktree that already runs to compile and publish it,
with the same one-way `warm` message,
and loads the rules read-only
(`forbidden_strings::CacheAccess::ReadOnly`,
`CandidateScanner::load_writing_cache`),
so the rules it compiled are only ever a scan's input.
An artifact too damaged to decode is not a miss here:
the load reads it,
rejects it,
and repairs it in this process.
The command publishes the artifact itself in two cases:
a worktree with no running service,
and a rules list the command took from `FORBIDDEN_STRINGS_RULES` in a repository whose configuration names no
rules files,
which the service's environment does not carry and would not compile.
A configuration that names rules files wins over the variable,
so such a repository leaves its list to the service.
A service that declines the warm (its address space leaves too little room,
`warm_fits`)
also leaves the artifact for the next command that publishes,
so such a machine compiles the rules once per command:
correct,
and slower than a cache it cannot fill.
The artifact's publication (`package/forbidden-strings/cli/src/runtime_cache/publish.rs`)
writes a private temporary and renames it over the artifact,
syncing neither the file nor its directory:
every read validates magic,
schema,
scanner version,
platform,
source digest,
payload digest,
engine digest and trailing bytes,
so a torn artifact is rejected and recompiled,
and the file is derived data that may be deleted at any time.
The waiting notice of the commit Git builds in the scratch repository names that `git commit`,
signing,
the signing program,
and when the step is stopped:
a commit whose message came with the command (`-m`,
`-F`,
`-C`) is stopped at `SCRATCH_COMMIT_STEP_CEILING` (`service/wait_ceilings.rs`,
60,000 ms:
three times the slowest `git commit` step that ended by itself,
12,735 ms,
in the 1,433 steps of the eight generations the timing log held on 2026-11-12,
raised to the 60,000 ms floor `SCRATCH_COMMIT_STEP_CEILING_FLOOR`
because a ceiling read from one generation swings with the rotation,
which is what the 16,332 ms of three times a 5,444 ms record did on 2026-11-11,
when an ordinary commit reached it inside a disk wait and failed:
its step ran 47,978 ms,
and a second commit failed the same way at 37,153 ms),
and a step that reaches that ceiling while the kernel holds it in a call a signal cannot end is slow and not stuck,
so the ceiling stands aside once for it:
the step is waited for to `SCRATCH_COMMIT_STEP_STALL_CEILING` (`service/wait_ceilings.rs`,
300,000 ms:
6.2 times the slower of those two steps,
and 23 times the slowest step that ended by itself),
one line says the ceiling stood aside,
how long the step has lasted,
the process the kernel is holding and the bound it now runs under,
the wait that owns the step is the one that says it,
the record keeps the moment (`heldAfterCeilingMs`) and the bound (`stallCeilingMs`),
and a step the kernel holds past that bound is ended as any step at its ceiling is,
with words that say it was already waited for;
that `git commit` and the processes it started ended with `SIGKILL` and no other child of the command touched
(the notice holds the pidfd the step was given the moment it started and signals through it and nothing else,
reads the programs below the step first and asks that descriptor afterwards whether the step is still there,
so a walk taken while the step was alive is discarded when it ended in the meantime,
and signals the step whatever that walk came to,
recording `belowUnlisted` and saying in the stop's words that the programs the step started could not be listed,
so one of them may still be running),
and a signal the kernel refused is recorded as `belowUnended` and said in the same words,
because a program that was listed and could not be ended may still be running too,
and the person is the one who can end it
a stop recorded only when a signal was really taken for the step,
only `ESRCH` from the descriptor's signal read as the step being gone,
with every other refusal,
and a look that could not be made,
leaving the ceiling armed rather than standing down,
and the command failing with a message that says nothing was committed and to run the same command again;
and a step whose start the kernel refused a descriptor for is not signalled by the notice at all:
finding a number and opening a descriptor for it are two operations,
so a step that ended and was reaped between them leaves a number another process holds,
and nothing here reads a number
(the wait that owns the step ends it instead,
below);
and a step the kernel has not released is said,
once,
in a line of its own:
a process in uninterruptible sleep (`D`) cannot act on `SIGKILL` until the call it is inside returns,
so the commit waits the kernel's time,
and the line names the commit that was stopped and the process the kernel is holding
(the step itself,
or a program it started,
which the line says in its own words),
says the stop was sent and the kernel has not released it,
says what state the kernel reports for that process,
and says that nothing was committed and that the person's index and files are as they were
(owner,
2026-11-10,
"Keep waiting and say so",
chosen over leaving after a bound and failing the commit;
the wait is an exception to his rule that nothing waits for ever,
and `doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md` holds the kernel behaviour behind it and the options he did not choose);
the line names only a process that two looks,
one look interval apart after the signal,
both find in uninterruptible sleep,
so a process that acts on its stop in that interval is never said to be held;
the wait that owns the step says that line and not the notice's thread,
which a process at its thread limit never started,
and it says that line once:
a person who met the same sentence three times in one failed commit read nothing new the second time
(2026-11-11,
the release 0.6.1-alpha.11 that failed a commit in a disk wait);
a look that could not be made decides nothing and is made again,
so a `/proc` that refused one read never silences the explanation of a wait that lasts;
a line the writer could not take is handed over again as soon as it can take one,
so a reader that stopped reading delays that line and never loses it;
and that reader holds the writer thread,
which every line of this notice goes through,
and never the wait;
its lines are written by a thread of their own,
handed over a queue of one line,
so a reader that stopped reading holds that thread and never the ceiling,
a line that cannot be handed over is dropped and counted (`linesDropped`),
`finish` waits for the notice's answer and for the line already handed over no longer than `NOTICE_BOUND`
(twice `LOOK_INTERVAL`),
and a command that ended prints nothing more:
the writer stops at its next line,
so at most the one line already inside a write can still reach the reader after the command is over,
and the writer ends there rather than living on beside a reader that stopped reading;
a member the kernel refuses a pidfd for is never signalled by its number,
and its silence is never read as an end;
a commit that may open the editor has no ceiling (`src/native/commit/native_commit_waiting_notice.rs`).

##### The wait that owns the step enforces the ceiling

One ceiling has two enforcers,
and the second needs no thread and no name a reuse can reach.

The notice thread ends the step through the pidfd its start gave it,
which names that step for good,
and that is where the waiting lines come from.
The wait that owns the child ends it too
(`src/native/process/child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`):
the ceiling is counted from the step's own start,
which is that wait's own moment,
so no look that ran late can lose it;
the step is killed through the child that wait owns and has not reaped,
whose number the kernel keeps until it is reaped,
so no process that took the number can be reached;
the programs below it are read before the signal,
while the step still holds its number,
and ended through descriptors pinned the same way,
with every program it could not pin or not signal counted as `belowUnended`,
so the stop says what it could not end instead of reporting a walk that ended everything it listed;
and the ceiling comes only after that wait asks one look whether the kernel holds the step,
which is what tells a slow step from a stuck one,
because a held step is waited for to `SCRATCH_COMMIT_STEP_STALL_CEILING` and never ended at the ceiling it reached;
and the child is reaped there,
and a child the kernel holds in an uninterruptible call is waited for with no bound after that,
so the kernel's release of it is what ends the command
(the owner's answer of 2026-11-10,
"Keep waiting and say so",
and the notice's own line says so once while that wait runs),
and the status that reap collects decides what ended the step:
a step that finished by itself between the ceiling and the signal is answered with its own exit and no stop,
because a kill cannot turn an exit into a signal,
and only a status that is the `SIGKILL` this wait sent is a ceiling stop.

That second enforcer is why the ceiling survives both failures the notice cannot:
a system that refuses its thread,
and a start that gave no descriptor to signal through.
A refused thread leaves no waiting lines,
so the person sees nothing during the wait and the stop's words after it,
which name the signer when the commit was signed and say nothing was committed.
A stop is written once,
by whichever enforcer ended the step first,
and the wait's own record sets `armed` for a step whose notice thread never ran.
There is no state in which a commit under a ceiling waits with nothing to end its step:
a step the kernel holds is waited for to the bound this package measured,
and one it still holds past that bound is signalled,
so the only wait left is the kernel's release of it,
which the owner chose to wait for and the wait's one line names.

##### What a stop reads about a step, and what it sends

A look at a process's record (`/proc/<pid>/stat`,
`src/native/process/process_tree.rs`)
answers one of three things,
never two:
the process is there and is the one that was found,
it is gone,
or its record could not be read.
A refusal (a descriptor limit,
a permission,
an I/O error,
a record that cannot be parsed)
is not an exit:
nothing is concluded from it,
and nothing is signalled through it,
while the grace and the ceiling end the step as they would have.
Reading a refusal as "the process is gone" signalled nothing,
so a ceiling ended nothing,
the wait outlasted it,
and the lock sweep ran against a live tree
(`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
finding 4).
A refusal is not a proof of which process the number names either,
so nothing is signalled through it,
and one line says that the step it names was not sent that signal:
the member was found a moment ago and may have ended and been reaped since,
and its number may belong to an unrelated process.
The root of a step whose record could not be read is therefore never signalled:
`member_read` answers that it is no member of the tree
(`src/native/process/process_tree.rs`,
`src/native/push/child_watch_clock_marks_and_stopping.rs`),
so no number is sent a signal,
one line says which signal the step was not sent,
and the stop's waits are bounded whether the root is in the tree or not.
A member whose record reads as gone is never signalled:
its number may be another process's by now.
Every wait of the stop is bounded:
the wait after the kill ends with a grace of its own,
extended only while a member is in uninterruptible sleep and up to the
stop's ceiling,
so a root whose record cannot be read is waited for no longer than that rather than for
as long as the process lives
(`the_wait_after_the_kill`).
The watch pins its root once,
when the child starts and is still this process's own unreaped child,
and both signals of the stop go through that pin (`pin_at_start`,
`send_to_the_root`).
Reading the record again at the signal would establish only which process holds the number *then*:
the root can have ended and been reaped during the grace,
and an unrelated process that received its number would be the one signalled
(the third review's finding 1).
That pin also answers whether the root is still there (`has_ended`),
and the stop reads no tree and arms no ceiling for a root that has ended:
the children of a number that was reused are not this step's,
and a status the wait already collected is the answer rather than a ceiling the stop reached
(the fourth review's findings 1 and 2).

One window is left open there,
and this file records it rather than closing it.
The pin answers whether the root has ended (`the_root_is_still_there`),
and the tree is walked after that answer (`descendants_of_a_root_that_still_runs`,
`src/native/push/child_watch_clock_marks_and_stopping.rs`).
For a process that is not the step's to receive a signal there,
the root would have to end,
be reaped,
and have its number handed to another process that starts children,
all within the few instructions between that answer and the walk
(the fifth review's finding 1).
Everything after the walk closes its own interval:
a member is signalled through a descriptor opened while its number was proven to still name it,
so the proof covers the moment of the signal,
and the walk itself is what the proof cannot cover.
That window is open in the same way where nothing proved which process the root's number names:
the walk goes on there (`Pinned::Unproven`),
so that a bound is never lost to a record that could not be read,
and a member that cannot be proven at its signal is not signalled at all.

A signal is sent through a descriptor that names the process,
never to a number read a moment earlier.
Reading `/proc/<pid>/stat` and then calling `kill` leaves the window between the two in which the
number can be reused by another process of this account,
and the cargo-shadow review called that shape a P1.
Every member and the root of a step are therefore held by a pidfd where the kernel gives one
(`src/native/process/pinned_member_signalled_through_a_pidfd.rs`),
the start time is read before and after the descriptor is opened to show the number named one process
throughout,
and the signal goes through the descriptor with `pidfd_send_signal`.
This is the pattern `binary/binary_supervisor_pin.rs` already uses for the hang guard of the binary
tests,
in production form.
A pin always is its descriptor,
and a host that gives none has no pin at all:
nothing is signalled there,
and the sentence says what was not done.
The fallback that read the start time again right before the number was signalled is gone,
because the window between that read and the call is exactly the one a descriptor closes
(the third review's finding 3).
One signal stays by number on purpose:
the whole process group of the repair tool,
whose leader is unreaped at that moment,
so the kernel cannot have given its number to anything else,
and a group is not a thing a descriptor can name.
The rule is one decision everywhere:
a number is a safe address only while its process is this process's own unreaped child,
and anything else goes through a descriptor taken while that identity was proven.
A second look at a record that fails after the descriptor was opened holds nothing and signals
nothing:
between the two looks the process the first one found can have ended and been reaped,
and the number can name an unrelated process,
which the descriptor then names
(the third review's finding 2).

A one-shot query's ceiling is a deadline kept by the thread that owns the child.
`output_of_within_bound` (`src/native/process/captured_child_run_to_its_end.rs`) waits for its child
with `wait_within` (`src/native/process/child_waited_for_within_a_ceiling.rs`),
and its output goes to anonymous files,
so it needs no reader at all.
While that thread waits it has not reaped the child,
so the kill at the ceiling reaches the child and nothing else:
the pin a caller used to signal at its ceiling,
and the fallbacks that went with it,
are deleted rather than patched again
(the second and third reviews' findings about signalling by a reused number).
The caller no longer signals anything.

A child the kernel will not release is not waited for past the stop's grace.
Past the grace `wait_within` hands it to
`src/native/process/stuck_children_reaped_when_the_kernel_lets_go.rs`,
which gives it to a thread that collects it when the kernel lets go,
and keeps it in a list the next reaper drains when the system refuses that thread,
so the child is never lost and never a zombie for this process's life.
The command stops waiting and says the query was left running,
not that it was ended
(the fourth review's findings 4,
5 and 9).
`OwnedChild::end_and_reap` takes the same grace before it hands a leader on,
so the step route has the bound this file states for a leader that does not end in `STOP_GRACE`,
and it answers whether that leader was left running,
which is what the sentence of a ceiling the wait carried says (`AtTheCeiling`).

A step left running is owned by a thread that waits for it,
and that thread is given the child through a channel rather than through its closure:
a refused start drops its closure,
so a child moved into it would be neither ended nor reaped,
and would stay a zombie of this process once it ended.
The child the refused start hands back goes to the reaper above
(the fifth review's finding 3).
Where no task of the runtime can run at all,
the child a step left running is handed to that reaper too
(the fifth review's finding 4).
Which of the two ways awaits that child is decided by whether a task can run now
(`tasks_can_run`),
not by how the runtime was first built:
a build that missed its deadline can have arrived since,
and its workers take the child.

A batch query of the manual push is read by the runtime,
not by threads of its own.
The two pipes are made and registered before the start
(`src/native/process/step_output_read_whole_through_pipes.rs`),
both streams are read in one select that also holds the end of the wait,
and the request list goes in from one thread whose write carries the query's own ceiling:
there is no queue between the query and its answer,
no reader thread that a dropped receiver cannot end,
and one drain deadline after the query's end rather than one per chunk
(the fourth review's findings 3,
6 and 8).
Its answer is bounded by the request list this command wrote,
and a list that was cut short is not an
answer:
the query fails and says the list was cut short
(finding 7).
The thread that writes a batch query's request list writes with the query's own ceiling as its
deadline (`write_within`),
one pipe buffer at a time,
and gives up when the pipe takes nothing within it,
so a program the query started that holds the read end of that list without reading it no longer
leaves the thread blocked in the kernel for that program's life
(the third review's finding 6).
It is not joined either:
what it writes is the query's request,
and the answer is taken from the query.

A signal the wrapper catches is relayed to the child or it ends the wrapper;
it is never swallowed.
`SignalGuard` (`src/native/process/signal_relay.rs`) installs the handlers of `SIGINT`,
`SIGTERM`,
`SIGHUP` and `SIGQUIT` and points one never-closed descriptor number at a pidfd for its child,
so a signal reaches that child and never a reused number.
When the child cannot be published to that descriptor (a `pidfd_open` a descriptor limit refuses,
a `/dev/null` that cannot be opened,
a `dup3` that failed),
no handler is left installed:
the dispositions the guard replaced go back for the rest of its life,
so the next of those signals ends the command as it would without a guard,
and the warning says so and names the limit to check.
Keeping the handlers with nothing published meant the signal reached neither the child nor the wrapper,
and only `SIGKILL` ended the command
(`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
finding 6).

A step whose watcher cannot exist says so,
and still ends at its ceiling.
The watcher is a task of the runtime,
and a task needs a worker thread of its own
(`src/native/foundation/lazy_runtime.rs`).
When the runtime has none (the current-thread stand-in,
or a build the system refused),
nothing announces the wait,
nothing ends the step for being quiet,
and nothing ends it at its ceiling.
The refusal is now said once in the command's diagnostics,
the watch records it,
and the wait carries the step's own ceiling:
a step that has one is ended at it,
with the whole process tree `StepBound::EndAt` names,
and fails with a sentence naming the refused thread and the limit to check.
A step with no ceiling is still waited for as long as it lives,
which the owner decided for a write that cannot be run twice,
and the sentence that says nothing watches it is what tells the person
(`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
finding 8).
The line goes straight to the diagnostics rather than through the blocking pool,
because the refusal means no task may be reachable;
it is one line per step.

The read of a step that is ended at its ceiling ends with the step,
not with the last program that holds its pipes open.
A step's output is read whole through two pipes
(`src/native/process/step_output_read_whole_through_pipes.rs`),
and a pipe ends at end of file,
which needs every write end closed.
Hooks are off below a watched step,
so the programs that can hold one are the ones Git runs from configuration:
a smudge or clean filter,
a merge driver,
or a signing program.
A program that outlives the step keeps the read open for as long as it lives,
so the command waited past the step's ceiling with its locks and its transaction held
(`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
finding 2).
Each read therefore ends `OUTPUT_DRAIN_AFTER_EXIT` (1 second,
labelled unmeasured) after the step has ended,
and the step fails with a sentence that names the program still holding the output,
which the mount and inode of the pipe find (`src/native/process/process_use.rs`),
and says to check that program with `ps` and to run the command again.
An end of file wins over that drain:
the read is asked first at every poll,
and where the reactor has not yet heard that the last writer closed,
the descriptor itself is asked,
whose non-blocking read answers end of file at once.
So a step that wrote its whole answer and ended is answered,
not reported as one whose output is held open
(the fifth review's finding 5).
What the read got before that moment is not returned as the step's answer:
a step whose output may be incomplete is reported,
never guessed at.

The step's answer is its exit status and its standard output.
A program that still holds the step's standard error open one second after the step ended therefore
does not fail the step:
the exit status and the whole of standard output come back,
the bytes of standard error read before the drain ended come back with them,
and one line names the program and says where that stream stops
(`src/native/process/step_output_read_whole_through_pipes.rs`).
A standard output a program still holds open stays a failure,
because the answer itself may be incomplete.

A read-only query the wrapper asks of Git ends at a ceiling of its own.
The metadata questions (`src/native/repo_read/git_metadata.rs`) and the two one-shot questions of the
invocation (`src/native/repo_read/git_system_config.rs`,
`src/native/invocation/management_service_repository_selection.rs`) run with their output in anonymous files,
so no pipe can fill and no reader is needed,
and their end is awaited on a thread with a deadline of `PLUMBING_STEP_CEILING`.
A query past the deadline is ended with `SIGKILL` and reported with a sentence that names the program,
the ceiling and what to look at;
a query that answers is unchanged,
and such a query changes nothing of the repository,
so ending it costs nothing
(`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
finding 5).
A metadata answer has to be whole,
though,
and it is:
a read bounded at its ceiling is drained through pipes,
every byte of it
(`an_answer_larger_than_the_capture_keeps_comes_back_whole`),
so an answer larger than the `STEP_OUTPUT_LIMIT` bytes the captured run keeps comes back whole and no
refusal of a cut one is needed on that route.
A forwarded command (`src/native/invocation/forwarding.rs`,
`status_retrying`) keeps no ceiling on purpose:
there the person's Git owns the terminal,
the wrapper only carries its exit code,
and ending it would take the terminal away from the person mid-command.

A batch query of the manual push writes its request list while it reads the answers.
`cat-file --batch-check` and `rev-list --stdin` read one request and write one reply in a loop
(`src/native/push/manual_push_range.rs`),
and the wrapper wrote the whole list before it read anything,
so a reply pipe that filled stopped Git reading the list and the write waited for the reply for good
(`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
finding 9).
The list goes in from a thread of its own,
the answers are read by the runtime,
and the wait carries `PLUMBING_STEP_CEILING`,
so a query that never answers is ended and reported with the command line and the ceiling,
and a query the kernel would not let end is reported as one left running rather than as one ended
(the fifth review's finding 6).
A list that was cut short at that ceiling is reported rather than answered,
because what Git answered
covers only the part of the list that went in.

The two waits of the LFS upload a typed push runs are bounded,
each through the twin the auto-push's upload already used:
the interactive `git-lfs push` (`src/native/lfs/lfs_upload.rs`)
through the bounded push's own wait (`src/native/push/bounded_push.rs`,
`wait_until_done_noting`),
and the upload plan's Git questions (`src/native/lfs/lfs_interactive.rs`)
through the bounded query (`src/native/push/bounded_query.rs`),
which is the meter with no wall clock that the push's own queries use.
The window is the push's stall window in both,
so a program or a question that shows no sign of work for it is stopped and named,
and one that holds an open connection to a server is waited for while it works,
as the push itself is,
and ended when the connection stays open with nothing moving for the connected window
(`CONNECTED_PUSH_QUIET_WINDOW`;
owner,
2026-11-03:
"Stop only on no progress" and "Bound it,
keep retrying").
A stopped upload's program is ended with the grace every stopped child gets,
and what it started goes with it,
so the password program a login would have started cannot outlive the push either.
The connected window bounds that shape too:
a server that accepts the connection and never answers holds an interactive push for that window and no longer,
and the upload is then stopped and named,
with `git lfs push --all <remote>` as its way forward because a typed push has no retry plan.
What closes that shape's login case is the never-prompt pair:
a missing login fails at once instead of leaving the connection's wait with nothing to answer it.
The refusal that follows a login failure says what the wrapper did about prompting,
read from the pair the child was given (`prompting_disabled` in `src/native/invocation/child_environment.rs`):
a command at a terminal is told that the wrapper left Git's and `git-lfs`'s own prompting,
and no refusal claims a terminal was absent when it was there.
The upload still owns its child through `SignalGuard::start_relayed`,
which hands it over still published,
so a signal a process sends to the wrapper reaches the program for the whole wait,
as it did when the guard owned the wait.

##### The quiet windows are labelled unmeasured

No log held a quiet gap before this change:
the timing records held whole times,
and a whole time is an upper bound of
a gap,
never the gap.
The class numbers (60,
300,
600,
60,
10 and 30 seconds) therefore stay as windows with no citation,
labelled so beside
each
constant,
and now judge quiet time.
What the existing logs bound is recorded in the audit ("Quiet gaps in the existing logs").
A measured interval replaces a number only after the records above hold healthy runs,
with an owner-approved margin
recorded
beside the value;
no margin is chosen here.

#### Every wait has a bound

A bound is only a bound where the wait can be reached by the timer that is meant to end it,
so:

- no synchronous storage read runs on the thread that polls the runtime
  (a named pipe would block it before the future's first await,
  and no timeout could expire);
- a wait that races a wake it has already missed never switches off the branch that reads wakes:
  the count of jobs still running ends it,
  and a sending end a refused job keeps open must not be waited for;
- the process's bound on blocking jobs covers every job that runs,
  so a job the pool queued before its frame stopped waiting either gives up its work or keeps its place
  until the work ends,
  and one of the two is decided once by a claim rather than guessed;
- a caller that stops waiting revokes that claim as its future is dropped,
  so a step the pool still holds queued does nothing:
  no conversion changes a worktree after its caller reported a timeout,
  and no result is produced for a reader that is gone;
- a refused thread start says which of the two happened,
  because the claim decides it:
  either the step will not run and changed nothing,
  or the pool ran it on another thread and whether it finished is not known.
- a storage step of a drain awaits its place of that bound,
  and its job,
  no longer than the after-commit work's own state,
  because a place is held by the job that took it until that job ends:
  a read left behind in a job would otherwise leave the round waiting for a place nothing gives back,
  so no later push or upkeep round would run and the service would never exit idle
  (`a_storage_step_awaited_no_longer_than`);
  the step that found no place ran nothing,
  and it says so.
- the read of the retry plan and of the watch at a service's start is bounded,
  and a read that gave no answer is made again rather than given up on:
  the plan's directory and the watch's directory are kept either way,
  so a retry planned after the timeout is still kept for a restart,
  and the pause before each new read grows
  (`restored_within`,
  `PushRetries::restore_came_to_nothing`).


The owner,
said in the session at about 2026-11-03T12:37-05:00 (quoted):
"nothing should ever wait forever or indefinitely".
It is newer than,
and overrides,
the unbounded waits of the delegated picks 2 and 6 and their follow-ups,
and the owner's
earlier "Wait while connected (Recommended)" for pushes;
the coordinator flags that conflict for the owner.
The owner keeps:
"Commits should never fail.";
"never fail doesn't primarily mean retries!";
"never discard user data";
"You should (almost) never be waiting."

The owner answered through the question tool at about 2026-11-03T12:45-05:00,
quoted by the options chosen:

- at a wait's bound:
  "Return with an error" ("The command stops waiting and exits non-zero,
  naming what blocked it;
  the saved
  work stays saved.");
- whether the saved commit still lands by itself later:
  "No,
  keep it for a retry (Recommended)" ("Nothing lands later by
  itself.
  Your staged changes stay staged,
  and the commit is also kept on a side ref.
  The error names the blocker and the
  command to run again.");
- pushes:
  "Bound it,
  keep retrying (Recommended)" ("A connected push that stops making progress for a measured time is
  ended;
  the service's retry plan keeps the push and tries again;
  the command returns with a line saying the push is
  queued.").

A wait with no bound of the wrapper's own is the scratch `git commit` a commit builds,
once the kernel has taken it:
a process in uninterruptible sleep cannot act on the ceiling's `SIGKILL` until the call it is inside returns,
so that wait ends when the kernel releases the step and never before.
The owner chose it (2026-11-10,
"Keep waiting and say so")
over leaving after a bound and failing the commit,
and the wait says once and again,
for as long as it lasts,
that the stop was sent and the kernel has not released the step,
so the wait is never silent
(`doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`).

The inventory of every wait,
with the end behavior of each and the options for the ones not decided,
is
`doc/planning/cli-git-no-unbounded-waits.md`.
The bounds,
in `src/native/service/wait_ceilings.rs` with the measurement beside each:

- `ANSWER_CEILING` 20 minutes:
  the client's wait for the service's answer,
  heartbeats or not (3.9 times the longest answer wait
  in the timing log,
  308 s;
  1.6 times the longest whole commit,
  751 s).
- `WITHDRAW_CEILING` 10 minutes:
  the client's wait for the answer to its withdrawal (five times the longest observed
  landing).
- `START_CEILING` 60 seconds:
  a start the service manager reports as under way (58 times the slowest observed start,
  1.04 s).
- `SERVICE_GREETING_SILENCE` 3 seconds:
  how long a commit lets a unit the service manager reports `active` stay silent before it asks the manager to
  restart it.
  A service greets a connection before it restores anything
  (`service_server_greets_before_the_journal_is_restored_tests.rs`),
  so a running unit that greets nothing is one whose program stopped inside its own process,
  and a start of a unit the manager believes runs brings nothing back.
  Nearly three times the slowest of the 52 measured cold starts (1.04 s),
  and under the owner's 5,000 ms bound on what a commit may add to native Git.
- `HAND_OVER_GREETING_WAIT` 2 seconds:
  the longest a command waits for the greeting of a service it only has to tell about work whose record it has
  already written (a snapshot's trash,
  a removed worktree's subvolume,
  a push record,
  and the work after a landing).
  The record is durable and every later command of the repository wakes the same service,
  so an answer that does not come costs one later wake and never the work.
  Twice the slowest of the 52 measured cold starts,
  where `START_WAIT` (ten seconds) is what a command whose next step needs the answer keeps.
- `LANDING_LOCK_CEILING` 1 minute:
  the landing waits for a foreign `index.lock` this long.
  It is five times the longest wait at a holder's place that ended when the lock was released (`LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT`,
  8 s,
  the largest of the 71 landings that waited at a place at all in the 4,386 answers since 2026-11-02T04:01:38-05:00;
  the longest whole lock acquisition is 11,490 ms once the wrapper's own staging of a cold index is counted,
  which keeps the one minute at 5.2 times it),
  in whole
  minutes.
  The longer waits in that journal were one lock that a live or killed `git commit` left for 62 minutes (2026-11-07),
  one 497,803 ms wait that ended when a person removed the lock,
  and five refusals at 600,090 to 600,718 ms.
  The lock that outlasts one commit's wait is remembered by its file identity,
  and the commits queued behind it are refused at once
  instead of each waiting a ceiling (`src/native/landing/service_index_refused_lock.rs`).
- `MERGE_LOCK_CEILING` and `RECONCILE_LOCK_CEILING` 1 minute each:
  the merge's wait for a foreign `index.lock` before it changes
  anything,
  and the wait of the index reconciliation after a commit's corrections removed every change.
  Neither wait is logged
  (they run in the client,
  on the person's terminal),
  so each takes the landing's measured ceiling:
  the same kind of holder,
  the only
  sample there is.
  The commit timing log read on 2026-11-08 (1,608 records,
  2026-11-05T03:13:38Z to 2026-11-08T23:49:56Z) adds no sample of its own:
  it holds no record of a
  merge,
  though 351 merge commits were made in the window,
  and no `git reset` step,
  so the reconciliation never ran in it.
  What it shows is
  that no wait a holder released came near a minute:
  the longest was under 23,783 ms of answer time,
  and the longest wait at a holder's place that ended when the lock was released was 8,000 ms.
  The memory of a lock that outlasted a wait is per process and the service lands every batch of a worktree on one
  thread,
  so only the landing has queued commits to spare it for;
  each client command waits at most this long on its own.
- `SETTLE_LOCK_CEILING` 1 minute:
  the settling of an earlier service's open batches.
  Five times the longest settle in the journal
  since 2026-11-01 (`LONGEST_SETTLE_WAIT`,
  172 ms of 4,386 settles read on 2026-11-08,
  99th percentile 17 ms;
  the read of 2026-11-07 had 3,630 settles and 20 ms),
  in whole minutes.
  The settle's whole time
  holds any wait it made,
  so the figure can only overstate it.
  The commit timing log holds no settle at all (the settling is the management command
  `service settle-earlier-batches`),
  so the journal stays its only sample.
- `SERVICE_HOLD_CEILING` 10 minutes:
  a forwarded `git add`,
  `rm` or `mv` waits for the service's hold,
  and its whole series of
  reruns ends,
  at this (five times the longest of 715 logged holds,
  119.7 s).
- `CONNECTED_PUSH_QUIET_WINDOW` 150 seconds,
  unmeasured,
  labelled as the other quiet windows (twice the longest whole push in the
  journal,
  72.4 s):
  a push whose connection is open and that shows no sign of work for this long is ended as a stall.
- `PHASE_PATIENCE` 5 minutes:
  the test marker compiled into the executable ends its pause at the limit of any test.
- `READ_CEILING` 120 seconds:
  every read-only Git query of the check path ends at this bound:
  the candidate listings,
  the `git add` replay,
  the real-index query and the rule facts run through the same watch as the service's steps
  (`repo_read/git_metadata.rs`).
  It is `PLUMBING_STEP_CEILING`:
  the longest read step of the service's landings over 1,651 landings was
  9,663 ms,
  so two minutes is twelve times it,
  and the longest client preparation of the commit timing log was 250,280 ms,
  which is the whole of every read inside it.
  It is below `ANSWER_CEILING`,
  so the person hears what blocked the check before the service's own bound.
- `CHANGED_PATHS_CEILING` 2 seconds:
  the one `git status` a `git worktree remove` reads to record how many tracked paths differed from the
  `HEAD` the removal took,
  and the count is a best-effort fact about a loss that is already attributable from the rest of the line,
  so this read ends there and the line leaves the count out
  (`src/native/worktree/worktree_audit.rs`).
  A linked worktree of this repository's size (23,194 tracked paths,
  3.1 MB index) answered in 0.29 to 0.43 s with a warm page cache,
  in six runs over two fixtures on 2026-11-11,
  and took more than 5 s under the load other sessions put on the disk,
  which is the wait notice a person saw during a removal.
  Two seconds is 4.6 times the warm figure,
  and below the 5 seconds after which a wait is announced,
  so this read is never announced as a wait.
- `PLUMBING_STEP_CEILING` 120 seconds and `OBJECT_STEP_CEILING` 10 minutes:
  the longest one Git
  step runs,
  whatever it shows and whether or not it may be ended for being quiet (the longest read step of the
  service's
  landings,
  9.7 s;
  the longest client preparation,
  250 s).
  Both are below `ANSWER_CEILING`.
  `REFERENCE_STEP_CEILING` (60 seconds) is gone:
  the owner's answer of 2026-11-04,
  "Never cut a
  branch write (Recommended)",
  means no ceiling for the branch write or for any other step that cannot be run twice
  (`may_be_ended`).
  `NEVER_ENDED_STEP_SLOW_AFTER` (60 seconds,
  17 times the longest of 2,093 reference updates,
  3.4 s) is
  not a bound:
  it is when the service starts telling the clients queued behind such a write what it is stuck in.
- `REPAIR_TOOL_CEILING` 18 minutes,
  `LINT_CEILING` 4 minutes,
  `REVIEW_CALL_CEILING` 1 minute and `REVIEW_CEILING` 2 minutes:
  the
  longest one repair tool,
  one Markdown check,
  the standards-review call and the whole review of a commit run,
  whatever they show
  (owner,
  2026-11-04,
  "Silence plus a ceiling (Recommended)").
  Each is the smallest whole number of minutes that is at least five
  times the longest measured run;
  the figures,
  the method and the sample's limits are beside each constant and in
  `doc/planning/cli-git-no-unbounded-waits.md`,
  "Branch write and tool ceilings,
  2026-11-04".
- `SHORT_EXTRA_WAIT` 7 seconds:
  how much longer a command waits for a write it started,
  after its own bound,
  before it says the
  result is not known (twice the longest branch write measured,
  3.4 s).
- `LOOKING_FOR_A_LANDED_COMMIT` 10 seconds:
  how long a client looks for the commit a request it already sent made,
  after its answer was lost,
  before it says the result is not known.
  Twice the owner's 5 s bound on what a commit may add to native Git,
  the derivation `WAITING_NOTICE_AFTER` uses;
  it covers the 5 to 9 s inside which the audit of service restarts and version skew (2026-11-09) saw the client give up
  while the commit landed seconds later.
  The look waits only while an open batch of the journal names the request,
  so a request nothing is landing costs one look and no wait.
- `FOREIGN_OWNER_PATIENCE` two minutes:
  twice `MERGE_LOCK_CEILING`,
  derived in code (`src/native/process/pid_namespace.rs`),
  the longest a merge waits for a foreign `index.lock` plus the work around it.
  The owner chose the value (2026-11-10,
  "Two minutes (Recommended)");
  it was 20 minutes,
  twice a ten minute lock ceiling that no wait took and that this release deleted.
  Measured:
  the longest hold of a lock the service journal records is the `lock_hold_ms` of 119,721 ms (4,386 answers,
  2026-11-02T04:01:38-05:00 to 2026-11-08T18:49:55-05:00),
  which the patience clears by 239 ms,
  and it is the closest proxy there is,
  since no log keeps the age of a record of another namespace;
  the longer figures of the same logs (a 751 s commit,
  a 540 s worktree add,
  a 1,228,797 ms command) time whole commands of this host's own PID namespace and not a record of another one.
  It is the age after which a record of an owner in another PID namespace is refused,
  not
  reclaimed:
  see "Foreign-owner expiry".
  A hold that does run past it is refused and named,
  never taken over,
  so nothing it holds is lost,
  and a worktree add of another namespace whose item path stands still that long meets that refusal.
  A record whose own time cannot be measured against this host's clock is refused at once,
  whatever its age,
  and never counted as a live owner's.

What happens at the bound,
on the commit path:

- A check-path Git read that reaches `READ_CEILING` is ended,
  and the check that needed it fails with `content-unavailable`,
  naming the read (the Git program and its subcommand,
  never a pathname),
  the bound it reached (the quiet window before the ceiling,
  or the ceiling itself) and that the index,
  the worktree and the staged changes are as they were,
  with the hint of a stopped read rather than the hint of content that is missing from the clone
  (`STOPPED_READ_HINT`,
  `policy/policy_events.rs`):
  the bound's "Return with an error",
  not a rejected commit,
  and the person runs the command again.
  A read that answers is untouched:
  the watch ends nothing that shows work,
  and it ends nothing before the ceiling.

- The landing waits for a foreign `index.lock` until `LANDING_LOCK_CEILING`.
  At the bound the batch is withdrawn before the branch
  moved:
  its intent is removed from the journal,
  so a restart of the service cannot land it;
  the commit it built is kept on
  `refs/cli-git/kept/<batch>`;
  the person's index and worktree are as they were;
  the client
  prints the holder (a process proven alive,
  or an owner nothing proves alive with what was looked at:
  no process with the lock
  open,
  no Git lock PID file),
  the side ref,
  and ends with the existing "Another program is using this repository's index"
  failure;
  the command to run again is the commit.
- A client whose wait passes `ANSWER_CEILING` asks the service,
  on a new connection,
  to withdraw its request (protocol version 7:
  `withdraw` and `withdrawn`).
  The service answers between two landings,
  on its one landing thread,
  so a landing in progress
  finishes first:
  if the commit landed,
  the answer is `landed` and the command reports success;
  if not,
  the request is dropped
  and its identity is remembered,
  so it cannot land even if it reaches the queue after the withdrawal,
  and the command ends
  non-zero with "This commit waited N minutes for the commit service and was withdrawn,
  so it will not land later by itself."
  A service that does not answer within `WITHDRAW_CEILING` leaves the outcome unknown,
  and the message says so and gives
  `git log -1` and `git status`.
- A commit the person interrupts while it waits is not made.
  The client watches the two signals a waiting commit answers,
  the interrupt a terminal sends and a stop's `SIGTERM`,
  notes which one came,
  and wakes the read it is inside even when the kernel delivers the signal to another thread of the process
  (`src/native/service/service_client_interrupted.rs`).
  It then withdraws its request exactly as a wait that reached its ceiling does,
  and the service's answer decides the ending:
  a request that landed before the signal came is reported as the commit it made,
  and the command succeeds;
  one the service gave up ends the command with the words that the commit was not made and that nothing lands later by itself,
  by the signal's own status,
  which a shell reports as `128 + the signal`,
  and the timing record names the signal (`client-signal`).
  A second signal ends the command at once.
  A kill that no process can catch is withdrawn by nobody,
  and the service's own rule about a request whose client is gone decides it:
  the request makes no commit when its connection closed before its batch landed.
- A request whose client is gone before its commit lands is dropped,
  and its commit is not made.
  The service asks this where the client's answer is written,
  when the group of requests is formed,
  and the landing asks it again where the branch has still not moved:
  before the objects of its attempt are written and with the index lock held,
  which is where the interrupt of a commit that waits for another program's lock lands
  (`src/native/landing/service_land_batch_body.rs`).
  A request whose client went away is answered nothing and committed nothing,
  and the rest of the batch is built again without it,
  so one command's interrupt costs no other command's commit.
  The tree such a request named stays where it is,
  because a client that reconnects sends the same request again and reads it,
  and the sweep takes it by age otherwise (`SCRATCH_TREE_ABANDONED_AFTER`).
- A Git step of the landing that reaches its ceiling is ended,
  unless it is a step that cannot be run twice,
  which
  has none.
  The batch is withdrawn as at `LANDING_LOCK_CEILING` (intent removed,
  commit kept on `refs/cli-git/kept/<batch>`,
  lock given up,
  failure `step-too-slow` with the step named),
  and an open
  intent finishes a batch only when its branch has moved,
  so nothing lands later.
  A step of the landing never ends a hook of the person:
  every Git it starts carries the hooks-off options,
  and the one place a hook of the person runs below a step of the wrapper is a push to a local path
  (see "The person's hook is never ended by a step the wrapper starts for itself").
- A landing that ends before its intent is in the journal,
  and a restart that drops an open batch whose branch has not moved,
  keep the same two things:
  the commit the batch built is kept on `refs/cli-git/kept/<batch>`,
  and the scratch reference of the batch (`refs/worktree/cli-git-service/<batch>`),
  which held the batch's objects and nothing else,
  is removed once that commit is kept,
  so a batch that will not land leaves no reference behind.
  A keep Git refused,
  a kept reference that names another commit,
  and a reference Git cannot read all leave the scratch reference where it is
  (the landing sweeps its own batch when it fails,
  the restart when it drops one;
  `src/native/landing/service_scratch_ref_of_a_dropped_batch.rs`).
- A service killed between the import of a batch's objects and the write of its intent leaves the scratch reference with no
  intent to settle it,
  and a service instance is a hash of its worktree's Git directory,
  which no path can be made from,
  so the
  landing writes `<batch>.scratch` in its own journal before the import,
  naming the worktree root and the worktree's own Git
  directory,
  and a restart's recovery does what the drop does for a named batch:
  it keeps the commit the reference holds on
  `refs/cli-git/kept/<batch>`,
  removes the reference,
  and removes the record once the reference is gone.
  A record that cannot be read,
  a keep Git refused,
  a kept reference that names another commit and a reference Git cannot read all leave the
  reference and the record where they are,
  and the journal keeps that batch number out of use while the record stays,
  because the next
  import of that number would move the reference
  (`src/native/landing/service_scratch_ref_marker_of_a_batch.rs`).
  The record is written without a flush,
  so a power loss
  rather than a kill can lose it and leave the reference where it was.
- The branch write carries the reflog message native Git writes for the commit it moves the branch to,
  as one argument of `git update-ref -m`:
  the commit's kind in Git's own words (`commit`,
  `commit (initial)`,
  `commit (amend)`,
  `commit (merge)`,
  `commit (cherry-pick)`,
  `commit (revert)`) and the first line of the commit's message after it
  (`src/native/landing/service_land_reflog.rs`).
  The first line is the message's own bytes,
  so a body line,
  a carriage return or a byte that is not UTF-8 cannot reach the row,
  and the message never enters the transaction Git reads on standard input,
  so no byte of it can change that grammar.
  The row names the commit the batch landed last,
  which is the value the branch now holds.
- The branch write (`git update-ref --stdin`) is never ended (owner,
  2026-11-04,
  "Never cut a branch write (Recommended)").
  While
  the one landing thread is inside such a write for longer than `NEVER_ENDED_STEP_SLOW_AFTER`,
  each waiting connection is sent a
  `waiting` message naming it (the command,
  the repository,
  the branch and commit it moves,
  and the scratch ref that holds the
  commit until the update ends;
  a whole new minute count each minute).
  The landing thread's record
  (`src/native/service/service_busy.rs`) lets the thread that reads a connection answer a `withdraw` itself for a request that has
  not begun to land.
  Admission and cancellation are one decision per request:
  the file `<id>.decision` in the journal's `busy`
  directory,
  made with `link(2)` of a whole file,
  which exactly one side can create (`landing` by the landing thread before it
  lands the request,
  `withdrawn:<token>` by a connection thread before it answers,
  the token being the maker's own).
  A `withdrawn`
  decision is provisional until its maker has checked the journal once more and created the file `<id>.<token>.cancelled` with the
  token the decision carries;
  only a decision whose own confirmation exists is acknowledged to a client or kept,
  and a maker that
  finds the request landed,
  open or unknown removes its decision,
  which nobody has acknowledged,
  so no observer reports a
  cancellation that is later undone.
  A confirmation belongs to one decision:
  one that outlives its decision confirms nothing of the
  decision that replaces it.
  A `withdrawn` decision with its confirmation stays as a tombstone,
  so every copy of the request that
  reaches a group is dropped (kept for as many withdrawals as the journal keeps finished batches,
  forgotten oldest first by the landing
  thread alone,
  between two groups,
  and while it is stuck in a write the record holds at most twice that number:
  past it the connection
  threads leave withdrawals to the landing thread,
  which removes a decision only while it still carries the token of the confirmation it removes;
  the
  memory is cleared when the service starts like the service's own memory of withdrawals);
  a `landing` decision is removed when its group is
  done,
  after the batch is in the journal,
  and a withdrawal that wins afterwards asks the journal again and answers `landed` for a
  request the journal names.
  A request that a batch still open names is left to the landing thread.
  Whatever cannot be established
  (a file that cannot be made or read,
  a journal that cannot be read whole) is never evidence of a cancellation:
  the withdrawal goes
  to the landing thread,
  and a request that cannot be admitted is answered with a failure and not landed.
  So a commit queued behind
  the write ends at its own `ANSWER_CEILING` and not after `WITHDRAW_CEILING`,
  with the write named in the error and the log and
  restart commands as the way out.
  A request that is the landing thread's is not withdrawn that way,
  because its commit may be made
  by the write:
  the service sends its client,
  when the write starts and on the connection it opens to withdraw too,
  the same
  `waiting` line with one more field,
  `"inside_write": true` (a client that does not know the field reads a plain `waiting`
  notice,
  a service that does not send it leaves a client that does on the older path,
  and the protocol version stays 7),
  and that
  client,
  at its original `ANSWER_CEILING` deadline plus `SHORT_EXTRA_WAIT` (7 seconds:
  the owner's "7 s (Recommended)",
  twice the
  longest of 2,093 reference updates,
  3,438 ms),
  does not ask to be withdrawn but listens for the landing and then ends with the
  "not known whether the commit was made" failure,
  which names the write and the scratch ref and says `git log -1` and `git status`;
  the write is left running and the commit lands when it returns.
  The client holds those words as the blocker of that failure and says them on standard error only once its own
  wait has passed `WAITING_NOTICE_AFTER` (10 seconds),
  the wait the place of a waiting commit waits for:
  a branch write took 5 ms at the median and 4,785 ms at the 99.9th percentile of the 6,703 landings this
  host's service journal recorded from 2026-11-01 to 2026-11-11,
  so a commit whose write is a normal one says nothing of it,
  and words that differ from the ones already said are said at once,
  so a write that has stayed stuck a new minute is told at once.
  The words name no moment of the write,
  because the moment they are read is the client's and not the write's.
  No line the client says about a long wait is said while the service's next message is already here:
  the client reads what has arrived first,
  because a message it has not read yet can be the landing,
  and a commit the service had landed then prints nothing of its write or of its place
  (`a_message_is_already_here`,
  `src/native/service/service_client_listen_and_ask_to_retire.rs`,
  said of the loop in `src/native/service/service_client.rs`).
  A line the service sends itself,
  such as one naming a lock's holder,
  is said when it arrives.
  The asking is one `MSG_DONTWAIT` peek on the connection the client already holds,
  which waits for nothing.
  A client that is never told takes the older path and waits
  `WITHDRAW_CEILING`.
  The record holds no lock (the owner:
  "Use as few locks as possible."),
  and the directory is emptied when the
  service starts.
  Protocol messages are unchanged.
  A client that waits behind a hung write keeps the service busy only as long as the write lasts;
  nothing in the wrapper ends it.
- A merge waits for a foreign `index.lock` until `MERGE_LOCK_CEILING`,
  naming the holder while it waits.
  A lock whose Git lock PID file holds a number this process table does not have
  (an empty lock beside it,
  which is what a killed Git leaves and also what a live Git of another PID namespace leaves,
  cross-vendor review of 2026-11-09,
  finding 2)
  is removed on the commit service's proof of disuse (`service_index_unheld_removal.rs`,
  `remove_if_the_owner_is_gone`:
  the file is opened and judged through that descriptor,
  no readable process holds it open,
  the name still refers to it,
  and only then is it unlinked),
  and the merge goes on.
  A holder that runs,
  one nothing proves alive,
  and a lock the proof of disuse does not cover (one that holds content,
  one behind the service's marker,
  one a process has open) are not removed.
  Such a merge is saved instead (`SPEC.md:2187`,
  the owner's answer):
  its commit is written to `refs/cli-git/kept/<number>` (a number no other command kept,
  by an `update-ref` whose old value is all zeros),
  a second record naming that ref is written beside the first,
  the command says where the merge is and what finishes it,
  and it exits 0.
  The next git command in that worktree finishes the checkout once the lock is free:
  the merge's own paths are installed,
  the branch moves by compare-and-swap against the recorded old commit,
  Settling installs only into the checkout of the branch the merge was computed on:
  when `HEAD` names another branch,
  nothing is written,
  a record kept on a side ref stays with it,
  and the line names the branch the merge is for;
  a record with no side ref that has nothing of the merge in the checkout goes instead,
  because another branch's checkout has nothing for it to finish and nothing of it was saved;
  `HEAD` is read again immediately before the branch moves,
  and a worktree that changed branch by then keeps the record as well.
  The record and the side ref go when the merge lands,
  and the side ref is deleted only while it still holds the merge's commit.
  and everything the merge does not change is left as it is,
  so an edit made meanwhile to such a file is carried forward.
  Staged content is never written over:
  an index entry for a path the merge replaces that is neither the branch's nor the merge's stops the finish,
  names the paths,
  keeps the record and leaves the staged content in the index,
  because a staged version has no other name.
  That check is read before the write and cannot be made part of it:
  `git update-index --index-info` has no way to
  write an entry only when the one it replaces is the one that was read,
  and `git read-tree -m -u`,
  which refuses a path that is not up to date,
  would replace the whole install this route is built on.
  So a staged entry that arrives after the check and before the write replaces the merge's entry,
  and the merge lands over it;
  the window runs from the reading that judged the entries to the `git update-index` call that writes them,
  and the take of every file the merge writes sits inside it.
  Closing it needs another command to stage inside that window,
  and the person would see the merge land and their staged version gone from the index,
  with nothing in the output saying which one it was.
  The same shape is a second window:
  `HEAD` is read to learn whose checkout this is,
  and another command can switch the worktree to a branch that names the same commit between that read and the install,
  so the merge's files and index land in a checkout that now belongs to the other branch.
  The read is made once more immediately before the branch moves,
  and a `HEAD` that differs keeps the record,
  moves nothing and says which branch the merge is for.
  The branch write verifies the branch and not `HEAD`,
  so a switch that lands in the instant between that look and the write is met by one more read straight after it:
  a `HEAD` that differs then puts the branch back where it was by compare-and-swap against the merge commit,
  keeps the record,
  and says which branch the worktree now has,
  that the merge's index entries and files are still in it,
  and how to finish the merge there or undo what is staged.
  The entries and files are not removed:
  the record is what finishes the merge later,
  so the person is told what is in the checkout rather than that it is clean.
  A switch that lands after that third read is not covered,
  and closing it would need the write itself to check `HEAD`,
  which `git update-ref` cannot do:
  a `symref-verify HEAD` in the same transaction as the update of the branch `HEAD` names is refused as two updates of
  `HEAD`.
  Closing that one needs another command to switch inside the same window,
  and the person would see the merge's staged content in the `git status` of the branch they had switched to,
  that branch's files otherwise unchanged,
  and the merged branch moved,
  with nothing in the output saying which command did it.
  An edit made meanwhile to one of the merge's own paths is never overwritten:
  the finish stops and names the file.
  A merge whose commit cannot be kept on a side ref is refused with nothing changed,
  never promised for a finish it may not reach.
  A branch that advances after the merge was saved makes that merge unable to land as it was saved:
  when nothing of it was installed its record goes,
  so no later command is stopped for it,
  the side ref keeps the merge commit,
  and the line names the ref,
  the commit and `git merge <commit>` as the way to land it;
  when the index or a file of it is already installed,
  the record stays,
  nothing is removed,
  and the command stops with the line naming the record and the moved branch,
  because those entries and that file have nothing else that ever settles them.
  The index reconciliation of a commit with no change left ends the same
  way.
  A forwarded writer waits for the service's hold until `SERVICE_HOLD_CEILING`,
  then runs and reports what Git says.
  Before the wait it learns which index it writes from two queries,
  `git rev-parse --path-format=absolute --absolute-git-dir` and then `--git-path index`,
  each asked on its own and each answer read as bytes with only its one terminating line feed removed,
  because a pathname may hold a line feed,
  the reading `src/native/landing/landing_location.rs` and `health/doctor.rs` both make.
  The in-process reader answers first,
  so a command in a plain checkout starts no query for it.
- A service that is still starting at `START_CEILING` ends the command with "The commit service did not start" and the
  journal
  command.
  A unit the manager reports `inactive` is asked to start at the first such answer,
  again after each further `START_WAIT`,
  because a stopped unit keeps its socket file and provisioning counts that as in place;
  asking only
  after `START_WAIT` made that commit ten seconds slower than native Git.
  A unit the manager reports `failed` is asked to have
  that state forgotten,
  for both of the worktree's units,
  and then to start,
  once;
  a second `failed` answer still ends the wait,
  so a unit that cannot start fails the commit in seconds.
  A unit the manager reports `active` whose service has greeted nothing for `SERVICE_GREETING_SILENCE` is asked to
  restart,
  once,
  and the wait goes on:
  a service greets a connection before it restores anything,
  so a running unit that greets nothing is one whose program stopped inside its own process
  (a frozen control group,
  a debugger,
  a suspended machine,
  a wait in the kernel that nothing ends),
  and no start of a unit the manager believes runs brings it back.
  The service survives being ended by design
  (its journal,
  its held commits and its work records are durable),
  so the commit gets its answer instead of failing
  (`src/native/service/systemd_dbus.rs`,
  `RestartUnit`).
- A wait for the service that lasts longer than `START_NOTICE_GRACE` says once on standard error what it waits for,
  on whom,
  and what ends it:
  the address the service is asked at,
  and the state the service manager reports about the worktree's unit
  (`starting_notice` in `src/native/landing/service_messages_of_a_wait_for_the_service.rs`,
  which the failure file reads).
  A wait that says nothing is a wait a person cannot tell from a hang,
  and the state the manager reports is what tells a service that is starting from one that stopped inside its own
  process.
  Where no manager can be asked,
  one connection attempt waits no longer than that same grace,
  and never past what is left of the start wait
  (`greeting_patience_without_a_manager` in `src/native/service/service_client_reach_service.rs`):
  an attempt that lasted the whole start wait spent the entire wait inside one read of the socket,
  so the line could only be said at the bound that ended it,
  which is the ten seconds of silence the audit measured.
- Where no manager can be asked and something took the connection but nothing answered,
  the wait ends at `START_WAIT` with a message that says what the wait proved:
  the address,
  how long it waited,
  and that a service stopped inside its own process looks like that
  (`ClientFailure::NothingAnswered`).
  "The commit service did not start" is said only where nothing listened at all,
  because the two states have different next steps.
  What provisioning already knows before that wait ends it at once instead,
  and the message names the cause and every way to change it
  (`src/native/service/service_provision_refusals_before_the_wait.rs`):
  an address longer than one socket address can hold
  (the runtime directory,
  the folder of the sockets,
  the instance and `.sock` take at most 107 bytes,
  so the runtime directory may take 40),
  which is counted on the spelling the commit's own connect uses and is asked before the fast path reads a thing,
  and a service the manager made listen at another address than the one this command waits at,
  which happens when the command's `XDG_RUNTIME_DIR` is not the one the account's service manager runs with,
  because the unit names its address with `%t` and `%t` is the manager's runtime directory.
  A socket unit the manager already holds at another address is asked about before this command starts anything,
  so a runtime directory of its own leaves no unit listening for a worktree it never looks at.
  Two spellings of one folder are one address there:
  a link,
  a `..` part,
  and a link already on the way,
  which this account has in `/var/run` reaching `/run`,
  all reach the one socket,
  so the file system decides that question and never the two texts.
  The manager's answer about the address it made is the fact that decides the second;
  an answer that cannot be read proves nothing and leaves the wait as it was,
  so a manager of another release never fails a commit that would have landed.
  A session bus that carries no user service manager answers with the remedy for an account without user services,
  not with the bus's own words,
  and a runtime directory that cannot be written loses the operating system's error number and gains that remedy.
  None of this is a second way to commit:
  the service is the only writer,
  so a command that cannot reach it changes nothing and stops.
- A push that shows no sign of work with an open connection for `CONNECTED_PUSH_QUIET_WINDOW` is ended as a stall and
  left to the
  service's retry plan,
  which keeps the push and tries it again.

#### Windows that need another process

Three instants need another command to act inside a few instructions of this one,
and no mechanism is added for them;
each is named with what the person would see.

- A blob staged between recovery's index read and its own index write.
  Recovery reads the index and classifies a path as holding the branch's entry;
  another command stages a different blob for that path without touching the worktree file;
  recovery then writes the merge's entry over it with `git update-index --index-info`.
  What the person sees:
  the merge lands and their staged version is gone from the index,
  with nothing in the output saying which version it was.
- An edit a clean filter hides.
  A path has a clean filter that removes `NOTE:` lines;
  recovery judges its content and takes the file aside;
  another program changes `NOTE: old` to `NOTE: new`,
  keeping the length and restoring the modification time;
  every field of the identity and the filtered object are then unchanged,
  so the private name is removed at the finish.
  What the person sees:
  the changed `NOTE:` bytes are gone,
  and neither the Git object nor the merged file holds them.
- Two recoveries of the same dead record.
  The first takes a file aside and writes the merge's index entry for it;
  the second,
  starting on the same dead owner's record,
  finds the private name and renames the file back to its path,
  because the repair does not ask whether the process that left it is still running.
  The first then finishes:
  its private name is gone,
  so its removal step says so and reports the merge complete.
  What the person sees:
  the branch moves onto the merge while the file the merge deletes is back in the checkout as an untracked file,
  and both commands report success.
- A detached `HEAD` made symbolic after the last check.
  The record names no branch and recovery starts with a detached `HEAD`;
  another command makes `HEAD` symbolic to a branch at the same commit after the last read;
  recovery's `update-ref --no-deref HEAD` writes the merge commit as a detached `HEAD` over that symbolic one.
  The read after the write then finds no branch,
  which is what the record expects,
  so the merge is reported complete.
  What the person sees:
  their worktree is detached at the merge commit instead of on the branch they switched to,
  and that branch itself did not move.

Decided by the coordinator under the owner's delegation,
veto open (inventory,
"Picks"):
every step class has a ceiling;
the person's
own Git and hooks are exempt,
as with native Git;
the connected push window stays 150 seconds;
the withdrawal at the lock ceiling
stays.
The withdrawal waits for the landing in progress,
so it is answered within the landing's own bounds,
and every step of a
landing now has one.

Kept from the delegated picks,
now inside these bounds:

- A holder proven alive and an owner nothing proves alive are waited for the same way until the ceiling,
  with a line that names
  what is known (the person's own `git commit` with its editor open holds the lock with no proof).
  A PID file holding a number this process table does not have is waited for the same way by a commit,
  and its line says which number it read and that a Git of another PID namespace writes one this table has
  never had:
  the number proves no death,
  so no commit ends early on it
  (cross-vendor review of 2026-11-09,
  finding 2).
  A merge is the command that acts on that evidence:
  it removes such a lock on its own proof of disuse (`remove_if_the_owner_is_gone`) and lands,
  and a lock that proof does not cover is saved for a later finish instead,
  with the line saying which number was read and what a person can do.
- An empty `index.lock` with no service marker and no Git lock PID file,
  that no process of a complete scan (Linux,
  every readable process,
  within `SERVICE_SCAN_BUDGET`) has open and this process has no other descriptor on,
  is removed by the service and the commit lands (owner,
  2026-11-07,
  "Service removes it").
  The file is opened first and judged through that descriptor,
  it is judged empty again after the scan and immediately before the unlink
  (a Git that writes its index into the lock and closes its descriptor during the scan is the editor-phase
  holder the module measured,
  cross-vendor review of 2026-11-09,
  finding 1),
  the name is looked at again before the unlink,
  and a lock replaced meanwhile is never removed.
  The journal keeps the lock's device,
  inode and size and the number of unreadable processes,
  and the waiting notices tell the person.
  A descriptor is matched to the lock by the device number and the inode the lock has,
  so a holder that reached it through a bind mount is seen,
  and a file of another btrfs subvolume that shares the inode number is not a holder (audit of 2026-11-09,
  finding 4).
  A lock with content is not removed:
  a native `git commit --all`,
  `--only` or with paths keeps `index.lock` with the index written into it,
  with no descriptor open,
  through its editor.
  Limit:
  processes in other user namespaces,
  or not readable,
  cannot be seen.
  The key `indexLock.unprovenOwnerTimeoutMs` stays
  retired.
- Where no inotify watch can be set the waits poll every `POLL_PAUSE` (50 ms,
  classed as a poll in the audit).
- A hold of the service that cannot be asked about is taken for a leftover at `UNASKABLE_HOLD_AGE` (150 seconds,
  from the longest
  of 715 measured holds);
  this number awaits the owner's approval.
- The service sends a heartbeat every second (`HEARTBEAT_EVERY`) to a client whose request is queued or landing;
  a client that
  hears nothing for `HEARTBEAT_SILENCE` (5 seconds) asks again with the same request identity,
  and ends with the "stopped
  answering" failure when the service greets nothing.
  The beat is written before the note it carries,
  and a note still unanswered one beat later is left for the next beat
  (`service_reply.rs`),
  because that note is a read of the journal through the blocking pool,
  and a round of the after-commit work holds that pool while it removes a trash:
  a beat written after its note waited 27 to 32 s in the record of 2026-11-10
  (`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`),
  and the client gave up on a commit that had landed.
  A trash's entry removals take every place of that pool but one (`places_a_round_may_take`,
  `foundation/blocking_jobs_within_the_budget.rs`),
  so the steps that answer a client run beside a round:
  the check of which instance a commit belongs to,
  a
  withdrawal's decision,
  and that note.
  The connection keeps that read (`StuckNotice`,
  `service_server_connection_handling.rs`):
  the beat behind a slow read waits for the same read and the words it answers with are written then,
  so a read that outlives many beats is made once and never queued behind itself,
  and every record that note reads is opened only when it is a regular file
  (`open_regular_file`,
  `service_busy.rs`):
  a named pipe at a record's name is answered instead of waited on.
  One round of the after-commit work starts housekeeping (upkeep,
  prewarm,
  subvolume deletes and trashes) for at most
  `HOUSEKEEPING_OF_ONE_ROUND` (30 seconds,
  `landing/housekeeping_of_one_round.rs`);
  the rest of it goes back to `pending`,
  is named in one journal line,
  and the lane that runs those kinds looks at that repository again after `LOOK_AGAIN_AFTER_DEFERRING` (5 seconds,
  `landing/service_work_slow_lane.rs`),
  so the part of a backlog one look could not start is taken by the look after it.
  An item a round has started is never interrupted,
  because a removal stopped halfway would need a resume the trash does not keep;
  the bound decides what a round starts,
  and the first housekeeping item a round finds starts whatever its slice has spent,
  so a round cannot leave every item to a later round that would spend its slice the same way.
  The place the removals leave free and the beat are what serve a client while a started item lasts.
  The connection that carries the beats is accepted before the recovery of the batch an earlier service left open,
  so a service that is recovering still greets and still beats,
  and nothing is landed before that recovery is over because only the landing thread lands and it reads the queue after it
  (finding C of the audit `commit-kills-and-concurrency`,
  2026-11-09:
  every client exited 1 after about 5.5 s with "not known whether the commit was made"
  while the recovering service landed their commits).
  After a commit was prepared,
  that failure reads the branch once (`git merge-base --is-ancestor <prepared> <reference>`)
  and states what it shows:
  that the branch holds the commit this command prepared,
  at its tip or under a later commit,
  so the commit was made and the command returns the landed commit with exit 0,
  reporting the landing and handing the after-commit work over as the answer that landed does,
  through the one function both paths share,
  so that a refusal from the work after a landed commit is a warning in either
  path and never the command's failure;
  that it does not hold it,
  so the commit was not made by this command,
  though the service may still land it;
  or that the branch could not be read,
  which keeps the words "not known whether the commit was made".
  Before that read,
  and for every failure that means an answer was lost after the request was sent,
  the client looks in the service's own journal for the commit the request made,
  for at most `LOOKING_FOR_A_LANDED_COMMIT` (10 seconds):
  a finished batch (`<seq>.done`) names the commit each of its requests became,
  and an open batch (`<seq>.intent`) names the commit it is making,
  which the branch then says is made or not yet;
  a commit either of them shows is a landed commit and the command returns it with exit 0 through the same function.
  A request whose open batch is held behind a reference lock is the state the branch cannot show:
  its record (`monochromatic-git-policy-cli/held-commits/<seq>.held`) is found from that open batch,
  and the command ends exactly as the `held` answer it never received would have ended,
  with the words about the lock and exit 0 (`service_landed_aftermath.rs`),
  because the commit is made and kept and no branch names it yet.
  The look waits only while an open batch names the request,
  so a request nothing is landing costs one look and no wait.
  A failure to reach a service again after a request was sent is never the "this commit was not made" text,
  whatever it was (a start that failed,
  a service of another release that would not stop,
  a service of another protocol):
  a request that was sent may have landed,
  so that failure is the "stopped answering" one,
  which says the result is not known and names the branch reads and the restart
  (the audit of service restarts and version skew,
  2026-11-09,
  printed "this commit was not made ... Nothing in the repository changed" while the commit was on the branch).
  A commit the service builds itself from the request is the case the journal answers where the branch alone could not:
  the open batch names the commit it is making,
  and the finished one names the commit it made.
- A command that waits renews the modification time of its scratch directory (`SCRATCH_REFRESH_EVERY`),
  so the sweep never
  removes the directory of a command that is still waiting;
  now that every wait ends,
  no wait outlasts `SCRATCH_STALE_AFTER`
  either.
- `INDEX_ATTEMPTS` (5) and `INDEX_PAUSE` (100 ms) cover only a lock that was gone by the time it was looked at.

#### Foreign-owner expiry

The owner,
2026-11-04,
"Refuse and name it (Recommended)" (option B of "Foreign-owner patience:
what its expiry authorizes"):
when
`FOREIGN_OWNER_PATIENCE` expires on a record from another PID namespace,
the command stops and changes nothing.
An age is not proof
that an owner is dead:
a merge or a copy that cannot run twice may legitimately outlive any number.
The error names the record file,
the owner's process number and namespace,
the record's age,
that the owner may be a live command in another container,
and the exact
`rm -- '<path>'` to run when no monochromatic-git-policy-cli command is running on the repository in any container and no Git process that monochromatic-git-policy-cli left
running (a step it stopped waiting for) is either.
It covers the merge intent recovery (`merge_isolation_recovery.rs`),
the takeover
of a conversion claim (`single_owner_claim.rs`),
and the journals and stage claims of a worktree add (`worktree_copy_owner.rs`,
`worktree_copy_journal.rs`,
`worktree_copy_recovery.rs`,
`worktree_copy_stage_claim.rs`),
and the legacy transaction owner records
(`transaction_owner.rs`).
A commit is outside it:
commits never wait for these records and never fail on them.
The age is the modification time of the record's file read against this reader's clock,
so it means something only while the two agree.
An age that cannot be measured,
which is a record whose own time is later than this reader's clock (a clock that was set back here,
a record copied with its time,
another machine's clock) and one whose time cannot be read at all,
is refused the same way:
the answer says the time cannot be trusted,
names the file,
the owner and its namespace,
and gives the `rm`,
and the record is never taken for a live owner's.
Taking it for one is what keeps a dead owner's record in the way for as long as its time stays ahead of the clock,
with no age and no `rm` (audit of 2026-11-09,
finding 2).
A record that is not there any more is finished and not unreadable:
its owner did the work and removed it,
so nothing holds,
nothing is refused,
and no command stops over a file that no longer exists with an `rm` for it
(cross-vendor review of 2026-11-09,
finding 3).
A measured age is this host's reading of the record,
so a clock that was set forward here,
or one that is behind where the record was written,
makes it read older than the hold;
that is why the message says so and why what follows from it is a person's decision and not the wrapper's.
Tests,
each red first:
`an_old_record_of_another_namespace_is_refused_and_named_and_nothing_changes`,
`a_claim_of_another_namespace_older_than_the_patience_is_refused_and_named_and_nothing_changes`,
`an_owner_of_another_namespace_with_an_old_item_is_refused_and_named`,
`a_journal_of_another_namespace_older_than_the_patience_stops_recovery_and_changes_nothing`,
`a_foreign_record_whose_time_is_later_than_the_clock_is_refused_and_named`,
`a_foreign_record_whose_time_cannot_be_read_is_refused_and_a_gone_one_is_finished`,
`a_record_with_no_measurable_time_says_which_case_it_is_and_a_gone_record_is_gone`,
`a_foreign_record_that_is_gone_is_finished_and_not_refused`,
`an_owner_of_another_namespace_whose_record_is_gone_is_dead_and_not_refused`,
and
`a_claim_of_another_namespace_whose_time_is_ahead_is_refused_and_named_and_nothing_changes`.

#### Output of a step, the review runner, and releases that meet

- The output of a step the wrapper started (`src/native/process/output_capture.rs`) is read once,
  when the step has ended or the
  command has stopped waiting for it,
  as a snapshot of the last `STEP_OUTPUT_LIMIT` (8 MiB) bytes of each stream as the files were
  sized when the read began.
  What was left out is said in the text the person sees ("N bytes at the start ... are left out"),
  and
  a read that fails is said too and is never taken for the end of the output.
  The notes reach the person with a failure of a repair tool
  and are kept in the tool's result,
  where a reader of its output (the `cargo metadata` of a repair) refuses output that was not taken
  whole.
  The files are emptied after the read on every path that ends a step,
  a stop included.
  A descendant that keeps appending after its leader
  has exited therefore holds nothing up.
  The files are emptied (`ftruncate`) once nobody will read them,
  which frees what was
  written until then;
  a step left running that keeps writing is held in memory by the kernel (it is anonymous memory of the step's
  own output file) until the step ends or closes its output,
  which nothing in the wrapper bounds,
  and a very verbose step left
  running can use memory in proportion to what it writes after the command has gone.
  A step the wrapper stops at its ceiling is
  stopped with its whole process group,
  and the leader is reaped within `STOP_GRACE`:
  a leader that does not end then (an
  uninterruptible wait for the disk) is handed to a reaper thread,
  and the command goes on.
- The review of the commit service (`standards_review_run.rs`) admits one job at a time in one atomic step:
  a state of a
  generation and a status,
  changed by compare-and-swap,
  so two callers that arrive together are never admitted both,
  and the
  release of the state runs from a guard when the job's thread unwinds,
  so a review that panics after its timeout leaves nothing
  outstanding.
  The look at the review's log runs inside the contained job.
  A caller whose wait ran out and who finds the job released waits for
  its value at most `SHORT_EXTRA_WAIT` (7 seconds):
  the release does not prove the value was sent.
- Threads that cannot start.
  A thread is only a way to do work sooner,
  so no shipped site panics when the system gives none
  (`Scope::spawn` did,
  since 0.5.28:
  on the service's thread it took the service down and the commit could not land).
  A scoped start
  (`src/native/foundation/scoped_start.rs`,
  `start_or_run`) runs the work on the calling thread when the thread is refused,
  and logs
  it once for the syncs of a landing;
  the second-pass input thread of a reconcile reports and stops its command instead,
  because
  writing the input in turn could block on a full pipe.
  The sites and their controls are listed in
  `doc/planning/cli-git-no-unbounded-waits.md`,
  "Round 5".
- Releases that meet.
  The `landed` answer of this release carries an optional `landing` object:
  the figures of the landing,
  which are `lockHoldMs`,
  `queuedBehind`,
  `readsMs`,
  `reads` and `syncs`,
  and which the client's timing record states under the same name.
  A service of an older release sends none and a client of this release states `null`,
  and a client of an older release ignores the field,
  so no new protocol version is needed and neither side has to know the other.
  The protocol version stays 7 and no other message changes:
  `waiting` with `"inside_write": true`,
  `withdraw` and
  `withdrawn` are the messages of 0.5.31 and 0.5.33.
  A service of 0.5.31 or 0.5.32 (0.5.32 changed the lock claims only) keeps what
  it was released with:
  a classifier by the name of the step,
  a decision race between landing and withdrawal that this release
  closes,
  a step left running that dies of `SIGPIPE` when its command exits,
  and no snapshot bound on output.
  A client of those
  releases sends its withdrawal on a connection that the service of this release answers,
  but it does not take the shortened
  deadline a late `inside_write` notice gives,
  and this release's client against a service of those releases may get no timely
  notice and then keeps to the longer withdrawal path (`WITHDRAW_CEILING`).
  Installing a new executable does not repair a service
  that is running:
  the first connection of the new release sends it a `retire`,
  which it handles between batches,
  and waits for it
  to leave for `START_WAIT`;
  a service in the middle of a write is never stopped by the wrapper,
  and the command that waited ends
  with "The commit service that is running was started from another release of this git wrapper and did not stop when ..." and can be run again.
  The single service of the releases before one service per
  worktree is asked to retire and given `retirement_wait` to leave before the manager is asked to stop it,
  because the manager's
  stop ends the whole unit and with it a Git step that cannot run twice;
  one that is still there fails the commit with the stop
  named for the person to run when no commit is running.
  The owner chose this on 2026-11-04 ("A:
  fail,
  retry (Recommended)").
  A git that is older than the service that answers it,
  which is what a rollback to an earlier release leaves running,
  cannot ask it to retire at all,
  and that message names both ways out:
  update this git,
  or install the release it belongs to and stop the newer service with `STOP_COMMAND`
  (`systemctl --user stop 'monochromatic-git-policy-cli@*.service'`),
  because a restart starts the program the manager's unit still names.
  Test:
  `an_earlier_service_that_is_still_working_is_asked_to_retire_and_never_stopped_by_the_wrapper`,
  and `an_older_git_is_told_how_to_get_a_service_it_can_work_with`.
- Provisioning asks the manager for the reload its own writes need,
  and a kill between writing a template and that reload is repaired
  by the next commit:
  the record of the unit texts the manager was told about
  (`service_provision_loaded_unit_texts.rs`,
  in the notes folder of the runtime unit directory)
  is written only after the manager accepted the reload,
  so a commit that finds the files matching and the record older
  provisions again and asks for the reload,
  and the commits after it find the record and take the fast path.
  Without it the killed upgrade's files look like an installation the manager runs,
  every later commit reaches the service of the release that was replaced,
  and ends with "The commit service that is running was started from another release of this git wrapper ...",
  however often it is run.
  Test:
  `an_upgrade_killed_before_the_reload_is_repaired_by_the_next_commit`.
  The record alone cannot tell a state the manager disagrees with:
  an interrupted downgrade writes and reloads the
    older release and is killed before it records that,
  and an interrupted upgrade then writes the newer files without a reload,
  which leaves the files and the record agreeing while the manager runs the release they replaced.
  The client knows:
  another release answering at all is proof that the unit the manager loaded and the files on disk
  disagree,
  so `reach_current` drops the record
  (`units_record_dir` in the client's settings) the moment it first sees that answer,
  not at the retirement deadline:
  a service that takes the retire and leaves can be followed by a manager trying a cached executable
  an upgrade removed,
  which ends the wait with `DidNotStart` and never reaches the deadline
  (finding 3 of the round-3 review).
  The next commit then writes the units and asks for the reload.
  The message says so,
  and names the restart for a service that still will not stop.
  Tests:
  `a_service_of_another_release_that_will_not_stop_drops_the_record_of_the_units`,
  and
  `a_service_of_another_release_that_takes_the_retire_and_never_returns_drops_the_record`.


#### Commands that add commits

A command other than `git commit` hands the push over too (owner decision,
2026-11-02):
`merge`,
`pull`,
`cherry-pick`,
`revert` and `am` (including the merge the wrapper completes from objects when the index holds
staged changes,
and the `--continue` that ends a stopped sequence).
The wrapper stays alive around such a command instead of replacing itself with Git,
reads the branch `HEAD` names and its tip
from the files before the command,
and after a command that exited `0` hands over one push record (not the review or the
upkeep of a commit) when all of these hold:
- `HEAD` still names the same branch under `refs/heads` and its tip moved;
- the old tip is an ancestor of the new tip,
  which leaves out every rewrite of the branch (a rebase,
  `pull --rebase`,
  `pull.rebase`,
  an amend,
  a reset);
- the branch has exactly one `branch.<name>.remote` and one `branch.<name>.merge`,
  the remote is not `.`,
  and the fetch
  refspec maps the merge ref to a local tracking ref that resolves;
- the new tip is not that tracking ref's tip and the tracking ref's tip is an ancestor of it (ahead,
  not diverged).
Ancestry is a walk over commit parents from the new tip,
  at most 4096 commits per question;
  a walk that cannot tell hands
nothing over.
  No Git process runs for the check.
  The push record is written before any route to the service is read,
  because it is a file of this worktree's own Git directory:
  a context with no route (a non-systemd session,
  a container,
  another `XDG_RUNTIME_DIR`) therefore leaves the push on disk for the next wake instead of losing it with the message,
  and the line the person reads says the push is queued.
  A service that cannot be reached is said plainly,
  and the command's exit
status is unchanged.
  The wake waits `HAND_OVER_GREETING_WAIT` for the service's greeting and never the commit's `START_WAIT`,
  because the record is written first and the command's own next steps need no answer
  (`src/native/push/auto_push_trigger.rs`).
  `git rebase` is never handed over.

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

Every ending of the trigger leaves one line in `monochromatic-git-policy-cli-push-trigger.jsonl` in the common Git directory (size-capped like
the commit timing file,
its generations kept under the next free number),
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
The secrets of any URL in `detail` are masked as `<credentials>` (the rule is stated in "An LFS upload",
above).
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

The push names both sides of its refspec,
`<branch>:<ref>`,
and the destination follows what a plain `git push`
of that branch would do if it were checked out (`setup_default_push_refspecs` in Git's `builtin/push.c`
and `match_push_refs` in `remote.c`).
A setting under which Git refuses,
or pushes the branch nowhere,
is a skip,
never a push:
the push result is `Skipped` with exit code `0`,
and its `shown` text is a note that names the branch,
the setting and the way forward,
printed to the same stream and starting with `monochromatic-git-policy-cli: auto-push: `.
One read of the configuration (`config --list -z --show-scope`,
answered without starting Git where the
in-process reader can) gives every value of the multi-valued keys.

- A branch without an upstream is pushed to `origin` with `--set-upstream` when that remote exists,
  and skipped silently when it does not.
- The push remote follows Git's order
  (`branch.<name>.pushRemote`,
  then `remote.pushDefault`,
  then `branch.<name>.remote`).
  The remote `.` (a branch that tracks a local branch) has nowhere to push and is a skip.
- A push remote with `remote.<name>.mirror` true is a skip:
  Git refuses a branch pushed by name to a mirror,
  and a plain `git push` there mirrors every ref and deletes the others,
  which the service never does.
- Any `remote.<push remote>.push` value decides,
  whatever `push.default` says.
  Every refspec must be a full ref name or hold one `*` on each side,
  optionally forced (`+`) or negative (`^`).
  The refspecs are applied to the local refs the way `match_push_refs` applies them:
  explicit refspecs first,
  in order,
  where a source that is no local ref or two sources for one destination
  refuse the whole push;
  then each local ref,
  in name order,
  takes the first matching pattern unless an earlier pairing already took its
  destination;
  then a negative refspec removes the destinations it matches.
  The branch is pushed to each destination it receives.
  A branch that receives none is a skip.
  A refspec of another form (`HEAD`,
  a short name,
  an object name,
  a deletion,
  the `:` of matching)
  depends on more than local ref names,
  so the push is a skip that names the refspec.
  The `%(push:remoteref)` atom is not used,
  because it maps one branch alone and so misses all three of these rules.
- Otherwise `push.default` decides:
  `current`,
  and `simple` with a push remote other than the tracked remote,
  use `refs/heads/<name>`.
  `upstream` (with its deprecated synonym `tracking`),
  and `simple` on the tracked remote,
  use the one `branch.<name>.merge` value;
  a differently named one is pushed to as well,
  because that is where the person tracked the branch
  (a linked worktree branch created from `origin/main`),
  although Git itself refuses it under `simple`.
  Several `branch.<name>.merge` values,
  a triangular `upstream` and a tracked ref outside `refs/heads` are skips,
  as Git refuses them ("multiple upstream branches,
  refusing to push").
  `nothing` and a value Git does not accept are skips.
  `matching` runs the plain `git push <remote>` with no refspec and leaves the matching to Git
  (the owner chose this over a push of the recorded branch alone):
  every local branch whose name the remote's push URL advertises is pushed,
  siblings included,
  a branch the remote lacks is left alone,
  and Git's own failure ("No refs in common and none specified")
  is a failed push reported with Git's words.
  One such push carries several branches,
  so Git's exit status is the worst fate among them:
  a sibling the remote refused makes it non-zero while the recorded branch arrived.
  The recorded branch's fate is therefore asked of the remote,
  never read from that status:
  after a matching push that failed,
  `git remote get-url --push --all <remote>` names the URLs the push went to,
  one `git ls-remote --refs <url> refs/heads/<name>` per URL names the object that ref holds there,
  and a push is reported as published only when every one of them names the commit it sent,
  while the service journal names the refused sibling from Git's own line for it,
  and the report a later command shows carries that line as well,
  because a push whose recorded branch arrived keeps every line Git wrote.
  A refused sibling is then no failure of this branch and plans no retry.
  A matching push names no refspec,
  so the commit every URL is asked for is the one the branch named before the push read the worktree's verdicts;
  a branch that moves while such a push runs leaves a URL naming another commit,
  which proves nothing and leaves the push the failure it was,
  so the retry that publishes the branch is still planned.
  Neither the remote-tracking ref nor the remote's fetch URL can answer this:
  the tracking ref lags the remote,
  so it still names a commit whose branch the remote deleted and reads a push to an unreachable
  remote as delivered,
  and `remote.<name>.pushurl` can name a repository that never received the push.
  A push remote that is a URL rather than a name the repository holds
  (`branch.<name>.pushRemote` or `remote.pushDefault`)
  is asked as that URL,
  since `git remote get-url` answers nothing for one
  and the setting is where it is written down.
  A push URL that cannot be asked,
  or that names another object,
  proves nothing and leaves the push the failure it was,
  so the retry that publishes the branch is still planned.
  It runs only while the recorded branch still names the commit the push read;
  a branch that moved sends nothing and its retry plan tries it again.

#### Reconcile and bounds

- A push of a branch with an upstream that the remote refuses as stale
  (its output holds `[rejected]`,
  the marker Git does not translate)
  is reconciled by the same push
  whenever its destination is one ref of one remote that the settings name (see "Destination").
  The remote `.`,
  a `remote.<name>.push` refspec,
  `matching`,
  `nothing`,
  several upstream branches
  and a ref outside `refs/heads` name no such ref,
  and the rejection is returned unchanged.
  An amended or rebased branch is reconciled like any other:
  the merge brings the published commits it replaced into the remote history beside the new ones,
  and only a real overlap between the two is a conflict.
  Histories that share no commit are merged too (`--allow-unrelated-histories`).
  The attempt fetches the remote branch into the private ref `refs/cli-git/auto-push/fetch-<pid>`
  (never `FETCH_HEAD`,
  which every fetch shares),
  deletes that ref,
  and branches on the two ancestor questions.
  Before it fetches it sweeps that namespace
  (`sweep_private_fetch_refs`):
  a ref named for a process that is gone is deleted,
  because a later release writes another number and would overwrite it never,
  and a ref whose process still runs may be another worktree of the repository reconciling at the same time and is left
  alone.
  A remote tip that the branch contains means the rejection was not a divergence and is returned unchanged.
  A remote tip that contains the branch means the commit is delivered,
  and nothing is pushed.
  Otherwise `git merge-tree --write-tree --no-messages -z` computes the merge from objects alone
  (with `GIT_ATTR_SOURCE` naming the branch tip,
  so its `.gitattributes` applies,
  see "Word-level second merge pass"),
  the conflicted files go through that second pass,
  and `git commit-tree` records the merge with the branch tip first and the remote tip second,
  signed (`-S`) when `commit.gpgsign` is true because `commit-tree` never reads that setting;
  a signer that fails publishes nothing.
  The message reaches `commit-tree` on its standard input (`-F -`),
  never as a `-m` argument:
  the kernel bounds one argument at about 128 KiB,
  and a merge whose conflicted files all resolved appends one line per merged part,
  so a merge of many parts failed its commit with `Argument list too long` instead of publishing
  (cross-vendor review of round 198,
  finding 2).
  Its message reads `Merge <remote>/<tracked ref> into <branch>`,
  naming the branch whose tip is that first parent,
  the way `git merge` names the branch it is on,
  measured on this host at 2026-11-09T09:33:10-05:00 with native Git 2.56.0
  (`feature` tracking `origin/main` wrote
  `Merge remote-tracking branch 'origin/main' into feature`),
  and a branch whose name differs from the tracked ref's short name also gets
  `Published to <remote>/<tracked ref>.`
  as a further line of its body,
  because the commit lands on a ref its subject does not name.
  That ref is the destination of the rejected push,
  which is not always the branch's upstream
  (an explicit `feature:main` push sends a branch that tracks `origin/feature`),
  so the line states where the merge was published and nothing about what the branch tracks.
  A detached `HEAD` has no branch to name and keeps the tracked ref's short name
  (`wording_of` in `src/native/push/auto_push_reconcile_plan.rs`).
  The merge is pushed with a plain push to `<remote> <merge>:<merge ref>`,
  never forced.
  A conflict (merge-tree exit `1`) that the second pass cannot resolve in every file publishes nothing and names the conflicting paths,
  in a sentence that names the branch the merge was into,
  the same name the merge's subject carries and never the destination ref.
  A merge whose conflicted files all resolved is published with the list of merged parts appended to the merge commit's message
  (the lines `word-level merge: <path>: part <n> of <m>: ...`),
  the same lines in the report after the push (each begins with `monochromatic-git-policy-cli: auto-push: `),
  and one journal entry per merged file after the push succeeded.
  A push that is refused as stale again repeats the fetch,
  the merge and the push,
  at most `MAX_RECONCILE_ATTEMPTS` (10) times in all,
  because sibling worktrees that keep pushing are contention and not a conflict.
  The merge is computed without reading or writing the local branch,
  the index,
  the worktree or the untracked files,
  so another consumer's uncommitted files cannot block it or be changed by it.
  After the merge is published,
  the branch and its worktree are fast-forwarded to it
  (`git merge --ff-only <merge>`) only when all of these hold:
  `HEAD` is the pushed branch at the tip that was pushed,
  and no path of `git --no-optional-locks status --porcelain=v1 -z --untracked-files=no`
  (both names of a staged rename or copy included)
  is a path of `git diff --name-only -z --no-renames <pushed tip> <merge>`.
  An empty status passes trivially;
  uncommitted work in files the move does not write is left exactly as it was,
  and the report says so.
  Git itself refuses,
  without changing anything,
  a move that would overwrite an untracked file.
  Any other state leaves the branch behind the remote,
  and the report names the reason,
  with the overlapping paths
  (five by name,
  then a count).
  The merge commit skips commit-time hooks and policies because it is made from objects.
  The record that guards the move is written before Git starts and removed once it ended,
  in the Git directory `rev-parse --absolute-git-dir` printed,
  read as the bytes Git printed,
  so a kill between Git's checkout and its ref update leaves an intent recovery finds
  whatever bytes the worktree's path holds.
  Every line the reconcile adds begins with `monochromatic-git-policy-cli: auto-push: `;
  a line that begins with `monochromatic-git-policy-cli: auto-push: not published: `
  explains a failure and replaces the generic "run `git push`" note,
  which would repeat a push that cannot succeed.
  When the branch could not be fast-forwarded it stays behind the remote,
  and the note names the reason and `git merge --ff-only <remote>/<branch>` for when the worktree allows it.
  The report reaches the person through a record of the worktree's notice directory,
  shown once by the next command (`record_push_report` in `push_failure_notice.rs`),
  because the command that ran the push returned before the push ended:
  it holds every line the push chose to show
  (`filter_push_output` in `src/native/push/auto_push.rs`),
  which for a push that exits `0` is the `remote:` lines a server printed,
  the lines the reconcile wrote,
  which begin with `monochromatic-git-policy-cli: auto-push: `,
  and Git's refusal to update a remote-tracking reference it could not lock,
  which is the only sign that the local view of the remote stayed behind,
  and it is written whatever the move came to,
  so the account of a move the worktree would not allow is read as readily as the account of one that followed.
  Its name carries the report's own prefix,
  so a later successful push of the branch removes the failure records of that branch and not a report nobody has read.
- The reconcile is one loop,
  and it is awaited
  (`integrate_and_push_awaiting` in `src/native/push/auto_push_reconcile_awaiting.rs`);
  the plan it integrates and every word of its report are read once
  (`src/native/push/auto_push_reconcile_plan.rs`).
  A command that is not a task of the runtime reaches that one loop by running it on the runtime
  (`integrate_and_push` in `src/native/push/auto_push_reconcile.rs`),
  and the merge push such a command supplies runs on a plain thread of its own beside the loop.
  That push plans and runs a real `git push` synchronously
  (`gated_merge_push`,
  the merge push of a person's explicit `git push`),
  and a start of Git from inside the runtime is refused;
  the thread is a plain one and never a job of the blocking pool,
  which is inside the runtime and would refuse the same start.
  A thread the system refuses is not worked around:
  no merge is published,
  the branch and the worktree stay as the failed push left them,
  and the push's own report is what the person reads.
- No push without a connection waits without a bound,
  and no push with an open connection is stopped for time.
  The push is stopped on no progress and no open connection,
  and under no ceiling.
  A push is at work,
  and not stalled,
  when the push or any process it started shows one of three signs,
  each read from `/proc` twice a second (`SAMPLE_INTERVAL` in `push_progress.rs`):
  it moved bytes,
  which is `rchar` plus `wchar` of `/proc/<pid>/io` growing;
  it used processor time,
  which is `utime` plus `stime` of `/proc/<pid>/stat` growing;
  or a task of it is able to run (state `R`) or waiting for the disk (state `D`) at the look,
  read per task in `/proc/<pid>/task/<tid>/stat`,
  because a thread waits in an `fsync` while the process itself sleeps.
  Only a tree that shows none of them for the stall window,
  and holds no open connection to a remote,
  is stalled.
  The three were chosen by measurement on 2026-11-03:
  a 30 MB push to a local bare remote moved 94 MB of `rchar` and 156 MB of `wchar` over its four processes,
  while `read_bytes` was 64 KB and `write_bytes` arrived late and in bursts from the receiver's disk writes,
  so the block-layer counters do not see the transfer;
  `rchar` and `wchar` count `read` and `write` calls,
  not `send` and `recv`
  (measured:
  102 bytes for 100000 bytes sent and received,
  100102 through `read` and `write`),
  so a transfer that curl alone performs is seen through the pipe that feeds or drains it;
  a `dd` of 64 MB with `conv=fsync` was in state `D` in six of seven looks and used no processor tick (a tick is 10 ms),
  so a wait for the disk shows in neither bytes nor processor time;
  a process reading a silent socket and a `sleep` stay in state `S` with a constant tick count.
  The signs cost this:
  bytes alone stop a push that is busy on a slow disk,
  as a gate saw (an `fsync` of 5 seconds after the hook,
  13 seconds in all);
  processor time makes a process that spins or polls look alive,
  at a resolution of one tick;
  state `D` makes a process wedged for ever in the kernel look alive,
  and state `R` one that only waits for a processor;
  no total time bounds them (see "Wait while connected" below).
  A server that checks the push after the pack arrived (GitHub's pre-receive hooks) is in state `S` with nothing moving.
  Wait while connected (replaced,
  same day,
  by "Bound it,
  keep retrying (Recommended)":
  see "Every wait has a bound").
  Owner decision (2026-11-03),
  verbatim:
  "Wait while connected (Recommended)",
  meaning that bytes on the push's network
  socket count as progress,
  and that the push keeps waiting while the connection to the remote is open,
  with a line naming the wait;
  the owner accepted that a connection that dies without a reset is noticed only when
  TCP gives up.
  Pick 5 of `doc/planning/cli-git-shipped-limits-audit.md`,
  "Push stall window,
  ceiling and Git's low-speed abort",
  option A,
  "Wait while connected" whole,
  is the coordinator's under the owner's delegation
  ("Pick options as if I had picked them and invite veto when I say I'm back",
  2026-11-03)
  and is open to the owner's veto:
  no stall window for a connected push,
  no ceiling,
  no low-speed abort.
  The sources are the kernel's socket diagnostics,
  read in the process (`tcp_sockets.rs`,
  `NETLINK_SOCK_DIAG`,
  `man 7 sock_diag`):
  for each connected socket of the machine its state,
  its inode,
  and `tcpi_bytes_acked` plus
  `tcpi_bytes_received` of its `struct tcp_info` (`include/uapi/linux/tcp.h`);
  the push's sockets are the `socket:[<inode>]` links in `/proc/<pid>/fd` of the push and of each process below it.
  Growth of those bytes is a fourth sign of work,
  and counts in `bytes_moved`.
  What the code observes as "connected" is exactly this:
  a process of the tree holds a socket in state `ESTABLISHED`.
  The phase of the push is not consulted
  (making the pack,
  pack sent,
  the server's checks,
  no answer yet to the first request),
  because an open connection is a remote that has not closed it,
  whatever it is doing.
  What the code cannot observe:
  a remote that died,
  or a network that dropped,
  without a reset or a close leaves the
  socket `ESTABLISHED` until TCP gives up.
  TCP gives up by retransmission for unacknowledged data and by keepalive probes for an idle connection;
  Git's HTTP client turns the probes on (`CURLOPT_TCP_KEEPALIVE`,
  `http.c` of Git 2.56.0),
  and `ssh` and `git://` leave them to the kernel's default.
  No number is invented to tell a dead remote from a slow one.
  Where the kernel's socket list cannot be read,
  the connection cannot be seen,
  and the stall window applies.
  The wait begins once a tree with an open connection has shown nothing for the stall window,
  and is said once,
  as a line that starts with `monochromatic-git-policy-cli: <program> <subcommand> is waiting for the server`
  (`git push`,
  `git fetch` or `git-lfs push`) in the run's output and in the diagnostic log.
  Silence during the wait is not counted against the window,
  so when the connection closes
  (no process of the tree holds an established socket) the stall window runs again from that moment,
  and the run is stopped as before.
  The rule covers every network run of the auto-push:
  the push,
  the reconcile fetch and the Git LFS upload.
  That scope is the choice of the agent that built the pick,
  not the coordinator's and not the owner's:
  the owner's answer named the push,
  the audit's low-speed entry names the fetch,
  and the upload runs through the same clock.
  It is flagged for the coordinator;
  a second patience without the connected branch,
  handed to the fetch and the upload,
  undoes it in a few lines.
  Measured on 2026-11-03 over HTTP to Git's own `http-backend` with `receive.keepAlive` at 0 ("set to 0 to disable keepalives
  entirely") and a `pre-receive` hook that sleeps four stall windows:
  the push is stopped at the first window before this change
  and lands on its first try after it.
  Git 2.56.0 sources the reading rests on:
  `builtin/receive-pack.c` `copy_to_sideband` sends `0005\1` (a sideband packet with no
  payload) every `receive.keepAlive` seconds,
  default 5,
  once the hook phase begins;
  `send-pack.c` waits in `receive_status`
  with the demultiplexer reading the sideband after `pack_objects` has returned;
  `http.c` sets `CURLOPT_TCP_KEEPALIVE` always.
  Whether GitHub sends keepalives is unknown.
  Over HTTP the wrapper used to set `GIT_HTTP_LOW_SPEED_LIMIT` (1000 bytes per second)
  and `GIT_HTTP_LOW_SPEED_TIME` (60 seconds) for the push and for the reconcile fetch unless the person set them.
  libcurl's `Curl_speedcheck` fails a transfer "Operation too slow" when the speed stays under the limit for the time,
  which keepalives of five bytes every five seconds do not prevent:
  a silent check longer than 60 seconds ended in a failed push
  (measured on 2026-11-03:
  "curl 28 Operation too slow.
  Less than 1000 bytes/sec transferred the last 60 seconds").
  The wrapper now sets neither variable for any run it starts.
  Git sets neither itself (`http.c`,
  Git 2.56.0:
  configuration at lines 453 to 460,
  environment at 1477 and 1478,
  libcurl only when both are positive at 1218 to 1223),
  so the wait ends when the remote answers,
  the connection closes or TCP gives up.
  A limit and time the person set in the environment or in `http.lowSpeedLimit` and `http.lowSpeedTime`
  reach Git unchanged and still apply (`push_slow_server_tests.rs` shows a 2 second setting ending a silent wait).
  A push to a local path or `file://` has no connection (its hook is a child of the push) and is stopped on a stall as before;
  a stop says that the remote may or may not have taken the push and whether a retry is planned,
  and the retry
  is what settles it (`src/native/push/bounded_push.rs`).
  There is no ceiling:
  no total time ends a run that shows progress,
  and a run that holds an open connection is ended only by the connected window (pick 5,
  above).
  What a tree with an open connection shows as work was measured on 2026-11-11,
  and that measurement decided the rule:
  `git-remote-http` was charged one processor tick every three seconds while it polled a server
  that had taken the connection and never answered,
  and moved not one byte,
  so a clock that counts that tick never reaches its window (the audit's F8:
  290 to 330 seconds of a service push with a 150 second window).
  A second measurement the same day decided the rest:
  `git-lfs` writes fourteen bytes of its upload meter to its standard error about every ten seconds
  while it waits on a server that never answers,
  the file belongs to the wrapper,
  so each line is a byte the tree wrote,
  and the window never fired in more than 260 seconds.
  While a connection is open,
  neither the processor time nor the bytes of the process that holds it are therefore work:
  that process polls the socket and prints its own account of waiting.
  What it really sends is on the connection,
  whose carried bytes are counted,
  and the work of every process that holds no connection is still work,
  which is where a push keeps working while its connection stands open:
  the pack it makes,
  the delta search,
  the listing of what to send.
  A socket carries no progress on the look that first sees it,
  because its counters then stand for this side's handshake and request,
  and that is also why a client that reconnects on its own is bounded:
  each new connection starts from nothing and its client is its holder,
  so silence across reconnections still reaches the window (measured:
  six connections of `git-lfs` thirty seconds apart ended one upload at 150 seconds).
  The cost,
  which the pick accepts:
  a process that sits in state `D` for good shows a sign of work for ever and holds the service's push of its
  repository;
  nothing the code can observe tells that from work.
  The connected window is fixed at 150 seconds for every shipped run;
  a control that must see it fire cannot wait that long,
  so `CLI_GIT_TEST_ONLY_CONNECTED_WINDOW_SECONDS`,
  a positive whole number of seconds,
  shortens it for every run of the process,
  in the manner of the crate's other control-only variables.
  One limit applies,
  in whole seconds and ignored when zero,
  negative,
  fractional or non-numeric:
  the stall window,
  `CLI_GIT_PUSH_DEADLINE_SECONDS`,
  30 by default (`DEFAULT_PUSH_DEADLINE`),
  is how long a tree with no open connection may show no sign of work;
  the variable keeps its name,
  so a setting made for the old fixed deadline now sets the stall window,
  and it is not an alias of anything else.
  Citation of the 30 seconds:
  none determines it.
  The owner decided the rule ("Stop only on no progress (Recommended)"),
  not the number.
  The journal of the commit service (read-only,
  2026-10-20 to 2026-11-03:
  794 pushes that landed)
  holds whole-push times of p50 3.5 s,
  p99 26.4 s and maximum 72.4 s,
  which include connected time and so do not measure the silence of a push that has no connection;
  no such silence is logged.
  It stays an open question for the owner,
  recorded in the audit.
  The variable `CLI_GIT_PUSH_CEILING_SECONDS` no longer exists.
  A stop reports the limit that was hit:
  `limit=stall` with `made no progress for <window> seconds`,
  or `limit=connected` when the connection to the remote was what was held open,
  and `bytes_moved=<n>` and `elapsed_seconds=<n>` in the line that ends the push's output,
  which the service journals.
  A connected stop names the server it waited for,
  as its host and port with any userinfo left out,
  which the run reads from its destination only when a stop needs it:
  a destination written as a URL answers with its own host,
  and a remote name with one bounded `git remote get-url --push` query.
  A push that lands reads nothing for the name,
  and the wait line a live run prints says "the remote".
  The stall setting is named only where the stall window ended the run:
  the connected window is fixed,
  so a connected stop says that the variable does not change it and names the retry that settles it.
  Where `/proc/<pid>/io` cannot be read (a system without `/proc`,
  such as macOS,
  or a kernel without task I/O accounting)
  nothing is measured:
  the stall window is then a fixed deadline for the whole push,
  as it was before,
  an open connection cannot be waited for,
  and the line says `limit=deadline`.
  Read-only Git queries are judged by the same signs and have no wall-clock bound.
  Past a limit the push is asked to end with `SIGTERM` (its helpers and hooks too),
  killed after a five second grace (`STOP_GRACE`;
  citation,
  partial:
  the helpers of a stopped push were measured on 2026-11-03
  to outlive it for 60 seconds,
  a `git-lfs` that never answers for good,
  and empty tracking-ref locks appeared about 37
  seconds after a stop,
  which justify the kill and the sweep and not five seconds;
  an open question for the owner),
  and reported with exit code 124.
  The report names what was cut short,
  says that the remote may or may not have taken the push,
  and says whether a retry is planned;
  it never claims a delivery,
  because no question of the remote can settle one
  (see "Commit hooks",
  `src/native/push/bounded_push.rs`).
  The same stop (a catchable request,
  five seconds,
  then the kill) ends every Git child the wrapper stops:
  the push,
  its read-only queries,
  the reconcile fetch,
  the Git LFS upload and a Git step of a commit
  that shows no sign of work for `CLI_GIT_CHILD_WAIT_SECONDS`.
  A stopped Git child leaves none of Git's lock files behind.
  Git removes its own `.lock` files when it ends or is asked to end with a signal it catches,
  so a child that answers the request cleans up after itself.
  A child that does not answer in the grace (it sits in an uninterruptible call such as a stalled disk,
  where the request waits and the kill wins) cannot,
  and neither can one whose signal arrives between creating a lock and registering it for removal
  (`tempfile.c` of Git 2.56.0,
  `create_tempfile_mode` and `activate_tempfile`).
  Once the child and everything it started are gone,
  the wrapper judges the reference locks of the child's repository
  with the judgement of "Reference locks left by a Git that was killed" below:
  a lock is removed only when its holder is
  proven gone,
  and otherwise the wrapper says what it found and prints the command that clears it.
  Releases 0.5.23 to 0.5.26 also removed an empty lock with no pid file that was born after the child started and was
  still empty after a settle time of one second,
  and a lock whose pid file named an ended member of the stopped tree.
  Both are gone:
  a cross-vendor review (2026-11-03T11:33:50-05:00,
  finding S-1) showed that a live writer of another Git,
  which writes no pid file,
  holds an empty lock with no descriptor open for as long as its transaction lasts,
  so neither
  the lifetime of the stopped child,
  the birth time of the lock nor its being empty ties the lock to that child,
  and a pid
  file older than the lock names an earlier holder.
  A lock whose pid file names a process of the stopped tree that has ended is **not** removed on that number:
  the number is one the wrapper's own PID namespace gave that process,
  and Git records no namespace,
  so a Git in a sibling namespace that shares the repository may have the same number,
  and the lock may be that Git's
  (cross-vendor review of 2026-11-09,
  finding 2).
  A matching number,
  a lock born during the child's life and an empty file fit both writers.
  The sweep that follows a stop adds no proof of its own,
  not even for a lock it saw a member of the stopped tree hold:
  what the wrapper sees of its own child is the child's process tree,
  and none of it says whose file a lock at a given path is
  (cross-vendor review of 2026-11-09,
  finding 1).
  A claim read before the stop from the pid file beside the lock rests on the same number as the rule above,
  and a Git of a sibling namespace that shares the repository makes a lock at the same path look the same,
  so no claim is made and nothing is removed on one.
  A lock of a stopped child therefore stays,
  with the command that clears it,
  and so does one whose pid file went first (Git removes a lock's pid file and then the lock,
  so a kill between the two leaves a lock nothing names).
  The cost is measured below:
  one incident in the journal this entry was written from,
  in which two stopped pushes left their tracking-ref locks and every later push of that ref was refused.
  A lock whose pid file names any other process is judged by the proofs below,
  once it is older than the margin;
  a younger
  one is judged by the next fetch or push of the repository,
  which judges first.
  A lock without a pid file is left,
  and the notice names it with the command that clears it.
  Only reference locks are looked at:
  `index.lock`,
  `shallow.lock`,
  object and pack locks are neither removed nor mentioned.
  The repository is the one of the child's working directory or its `GIT_DIR`;
  a step that names its repository with
  `-C` or `--git-dir` arguments,
  or a lock outside the Git directories (a private `GIT_INDEX_FILE`),
  is not looked at.
  A child that the kernel does not release within the grace after the kill,
  or a stopped tree that has not all ended by
  then (a loaded or stalled host has not yet run the killed processes),
  is waited for on a thread of its own,
  and its locks are judged when the whole tree has ended;
  the push is reported at its deadline and does not wait for it.
  A child that thread gives up on is kept,
  and the service's next round collects it with a non-blocking wait once the kernel has
  released it,
  so an abandoned stop leaves no zombie for the life of the service
  (`reap_abandoned_children` in `src/native/push/stop_of_a_push_and_sweep_of_its_locks.rs`).
  The judgement takes no lock of its own.
  A push whose output grows past 4 MiB (`MAX_PUSH_OUTPUT`) is stopped the same way,
  and only the first 4 MiB are kept;
  a push that ended between two looks with more printed than that keeps its own status,
  and the attempt says that only the head of its output is kept
  (`shortened_output_line` in `src/native/push/bounded_push.rs`).
  The check of the output's size and the read of the output are two steps,
  so a helper that keeps writing after the push exited can still pass the cap between them,
  and that attempt then misses the line that says its output was cut.
  It is left as it is:
  reaching it needs a background helper writing inside that window,
  and what it can cost is that line and nothing else
  (`run_bounded_push_awaiting` in `src/native/push/bounded_push.rs`).
  Citation of the 4 MiB:
  none;
  the owner decided "Keep all lines" for push output and no
  measurement of the size of a push's output is logged;
  an open question for the owner.
  The commit stays saved.
  No low-speed abort of the wrapper's own is added to any run it starts (see "Wait while connected" above).
  Each read-only Git query before the push
  (`config`,
  `for-each-ref`,
  `rev-parse`,
  `remote get-url`,
  `merge-base`)
  stops when it has shown no sign of work for the push's stall window,
  with no wall-clock ceiling of its own (delegated pick 4,
  scope added by the coordinator,
  open to the owner's veto;
  the 120 seconds of `QUERY_CEILING` are gone,
  and the journal's `git status` stopped at 2026-11-03T08:28 with
  `outcome=retrying push_ms=97398` is the case it ends),
  and a stopped query is a failure that names it.
  Where `/proc/<pid>/io` cannot be read the window is the whole time the query may run,
  since a query writes nothing and stopping it loses nothing.
  Its longest quiet gap is kept in the timing record.
- A push that lost a race on a pending reference lock (`cannot lock ref` in a push that otherwise delivered)
  is repeated up to three times with growing pauses,
  because the two worktrees' services share the references.
  A refusal that survives the repetitions exits `0` because the commits were delivered,
  and its lines are shown beside the `remote:` ones:
  they are the only sign that the local view of the remote stayed behind,
  and without them `git status` would report the branch ahead of a remote that already had the commits and nothing would
  say why.
- A failed push is reported with its complete output and the note
  ``monochromatic-git-policy-cli: commit saved locally, but auto-push to origin failed; run `git push` when ready.``
  unless the reconcile already said why the commit is unpublished.
  A successful push shows only the `remote:` lines,
  the lines of the reconcile and that refusal,
  and the record a later command shows carries them.
- The record a later command shows carries what the push came to,
  as the first line of the record file,
  and the notice is
  worded from that:
  a push Git refused delivered nothing,
  so the note says the commit exists only on this computer;
  a push
  that was stopped or failed for a reason that can pass proves nothing about the remote,
  so the note says it did not
  finish,
  that the remote may or may not have taken the commit,
  and what happens next,
  which follows the retry plan that still exists:
  the service keeps the push in its retry plan,
  or,
  when the round that wrote the
  record kept none,
  it is not trying the push again by itself and the person pushes by hand
  (`src/native/push/push_failure_notice_wording.rs`).
  A round says so as it writes the record,
  because it is the round that plans the next try or forgets the
  branch's plan,
  and a notice promising a try that was forgotten sends the person to wait for nothing.
  No wording is ever chosen from the
  text of Git's output or a remote hook's,
  because a remote's hook controls that text:
  a hook that prints the wrapper's own
  delivery sentence is shown as what Git said and changes no framing
  (`src/native/push/push_failure_notice.rs`).
  The notice claims no delivery at all,
  in any framing:
  a stopped or unfinished push says only that it did not finish and that the remote may or may
  not have taken the commit,
  and the retry is what settles it.

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
at most eight per-update local `git rev-list --stdin`,
`git cat-file` or `git ls-tree` processes run concurrently
(`MOST_CONCURRENT_LOCAL_QUERIES`:
`git ls-tree -r` took 80 and 50 ms at four and eight at once,
`git rev-list` 231 and 190 ms,
and no less at sixteen) and at most four `git ls-remote` queries,
which open a connection to a remote,
at a time
(`MOST_CONCURRENT_REMOTE_QUERIES`,
not measured),
one `git diff-tree --stdin` reads every range commit's delta,
and one `git cat-file --batch` reads every blob.
Never spawn one process per commit.
Met by the native executable:
the `ls-remote` confirmation of each destination (one per push URL of the remote) and the
 `cat-file` and `rev-list` of each update run side by side,
at most eight at once and within the host budget,
and their answers are
used in the order Git printed the destinations and negotiated the updates,
the first that fails deciding the error.
Not yet met by the native executable:
it peels each update's object with its own `git cat-file` process,
 runs `git rev-list <local> --not <published>`
with arguments,
and reads each range commit's delta with its own `git diff-tree` process;
the handover lists this as an
open divergence.

### Reference locks left by a Git that was killed

Release 0.5.23 judged the lock files of a Git child that the wrapper itself stopped (the push deadline entry of
"Auto-push" holds those rules;
since the fix of finding S-1 it uses the judgement below and removes nothing of its own).
A lock that any other killed Git leaves (a terminal closed mid-fetch,
the out-of-memory killer,
a reboot,
a release before
0.5.23) blocked every later `git fetch` and `git push` of the same reference with "Unable to create ... File exists"
until someone deleted the file by hand.
This entry extends the same code (`src/native/lock/stopped_git_locks.rs` for the pid-file reader and the sweep after a stop,
`src/native/repo_read/git_directory_roots.rs`
for the Git directories it shares,
`src/native/lock/stale_ref_locks.rs` and the `stale_ref_lock_*` modules for the judgement) to those
locks,
with one rule:
a lock is removed only when its holder is
**proven** gone,
and otherwise the wrapper says what it found and prints the command that clears it.

#### Owner requirements

- "never discard user data"
- "Use as few locks as possible."
- "You should (almost) never be waiting."
- "Invoke real git as rarely as possible."
- "Fix even pre-existing issues of the pkgs you own."
- Git's own file-system monitor is off in the person's global configuration,
  by the owner's decision of 2026-11-03
  (see the README section "Git's own fsmonitor";
  `git monochromatic-git-policy-cli doctor` says so),
  and the Git the wrapper starts for its own work passes `-c core.fsmonitor=false`
  (owner decision,
  2026-11-10;
  see "Real Git starts with traditional hooks off").
  Nothing here sets,
  unsets or pins `core.fsmonitor`,
  and no tier of the judgement counts the daemon's descriptors
  or working directory as evidence of anything.

#### What Git guarantees about who holds a lock

The source read is Git 2.56.0 (commit `a018953688`,
the installed version).
Git guarantees only that a lock is a file created with `O_CREAT | O_EXCL` (`tempfile.c:142`) and removed by its holder when
the holder ends or catches `SIGINT`,
`SIGHUP`,
`SIGTERM`,
`SIGQUIT` or `SIGPIPE` (`tempfile.c:93-98`,
`tempfile.c:112-120`,
`sigchain.c:45-52`).
It does not tell a reader who the holder is,
and it never removes a lock it did not create:

- The file holds no process number.
  The optional companion `<name>~pid.lock` (`core.lockfilePid`,
  `lockfile.c:112-139`) is created **after** the lock
  (`lockfile.c:182-184`),
  so a Git killed between the two leaves a lock with no pid file.
- The pid file is created with `O_EXCL` and a failure to create it is silent (`lockfile.c:121-123`).
  A holder that finds an older pid file in the way therefore writes none,
  and the older file names a process that is not the
  holder.
  A pid file is evidence only when it was born at or after the lock.
- Git itself reads the pid file and probes the process with `kill(pid, 0)` only to word its refusal
  ("appears to be stale",
  `lockfile.c:254-300`,
  with the caveat "PIDs can be reused" in the text),
  and then removes nothing.
- The pid file is removed before the rename of the lock (`lockfile.c:352-358`) or its rollback (`lockfile.c:368-371`),
  so
  a holder in the middle of a commit may have no pid file either.
- Reading the descriptors of other processes does **not** find a live holder:
  - `packed-refs.lock` is taken and the descriptor is closed at once,
    because the new content goes to a separate
    temporary file (`refs/packed-backend.c:1254-1261`).
    A live `packed-refs.lock` is therefore always empty and held by no open descriptor,
    for as long as the operation lasts.
  - A reference lock whose reference already has the wanted value,
    or that is being deleted,
    is created,
    never written,
    and
    closed to free the descriptor while the transaction goes on (`refs/files-backend.c:2888-2897`).
  - A reference lock that was written is closed straight after the write (`refs/files-backend.c:2058-2073`) and is
    renamed into place only in the commit phase,
    with the `reference-transaction` hook of the user in between
    (`refs.c:2676`).
    Its content is the new object name and it has no open descriptor for the whole of that window.
  - The waits for a held lock are short:
    `core.filesRefLockTimeout` defaults to 100 ms (`refs.c:992-1005`) and
    `core.packedRefsTimeout` to 1000 ms (`refs/packed-backend.c:1241-1245`),
    after which Git gives up.

#### Rejected rule: nothing has the lock open

The rule "a lock is stale when no live process has it open,
past a margin after its modification time" is rejected.
By the facts above it removes a live `packed-refs.lock`,
a live lock of a ref that is only being deleted,
and a live lock
that is waiting for its commit phase.
A live Git whose lock was deleted does not fail safely:
its rename moves the lock path onto the reference,
and if a second
Git created a new lock there in the meantime,
the first one installs the second one's half-written file as the reference.
The handover proposal of the 0.5.23 entry (an empty lock with no pid file,
past a settle time,
that no process has open)
is rejected for the same reason.
The open descriptor is kept as one **veto** (a lock that a process has open is held,
whatever else is known),
never as proof.

#### The rule: a lock is removed only on a proof about its holder

Candidates are reference locks of the Git directory and the common Git directory:
`packed-refs.lock`,
`HEAD.lock`,
`reftable/tables.list.lock` and every
`*.lock` under `refs/`.
`index.lock`,
`config.lock`,
`shallow.lock`,
object and pack locks and every `~pid.lock` file are never candidates (the index
has its own protocol in this document).
A candidate is judged without starting Git,
from the file system and `/proc`,
in this order,
and the first answer wins:

1. A candidate that is not a regular file,
   or that is younger than the margin (`MARGIN`,
   seven seconds:
   twice the 3,438 ms
   longest live reference step of a service journal,
   rounded up to a whole second),
   is left and not mentioned.
   The margin covers a lock that a live Git has just created and not yet filled,
   whose pid file is not yet written.
2. A candidate on a file system that is not local (anything not on a fixed list of disk and memory file systems:
   `btrfs`,
   `ext4`,
   `xfs`,
   `f2fs`,
   `tmpfs`,
   `overlay`,
   `zfs`,
   `bcachefs`) is unproven,
   because a Git on another machine is
   invisible here.
3. A process of the account that has the candidate open makes it **held**.
   A descriptor is matched to the file by the device number and the inode the file has,
   which is the pair that names one file on a machine,
   and the record's inode number is only the cheap test that comes first,
   before the file the descriptor has open is read.
   That read walks the path `/proc/<pid>/fd/<n>`,
   whose last step asks the file's own file system to revalidate its directory entry before any attribute is read,
   so it is taken in this process only where it cannot reach a server:
   the `mnt_id` line of the descriptor's record is looked up in that process's own mount table,
   and the file is read only when the mount is on the fixed list of local file systems above.
   A `statx(2)` with `AT_STATX_DONT_SYNC` then answers from the memory of the kernel:
   a file system whose server has stopped answers neither that walk nor its own flush,
   and `SIGTERM` does not end either wait
   (cross-vendor reviews of 2026-11-09,
   findings 3 and 4 of the first and 2 and 4 of the second;
   measured on this host with a stopped `fuse-overlayfs` server,
   `doc/troubleshooting/fuse-descriptor-attributes.md`).
   A kernel `overlay` mount is the one type that does not settle that question,
   because its own attributes are the attributes of the file below it,
   and whether a read reaches a server is decided by the layer's own file system:
   overlay revalidates a layer only when that layer sets a revalidation of its own
   (`fs/overlayfs/super.c`,
   `ovl_revalidate_real`).
   The layers are named in the mount's own options,
   and in a container they are the host's directories,
   which no process inside the container can reach:
   measured on 2026-11-09 in this package's gate container,
   every `lowerdir` and `upperdir` of its root mount is `ENOENT` there while the layers are the host's local disk.
   So that one read is made by a child of this process,
   and the answer is waited for only until a ceiling
   (300 ms;
   measured 0.5 ms mean and 2.1 ms worst over two hundred reads on this host):
   a child that has not answered is ended,
   and one the kernel does not let go is collected by a reaper whenever it does,
   which is what a command's own stuck child gets
   (`src/native/process/open_file_of_a_descriptor_in_a_child.rs`).
   The ceiling is a process the scan could not identify,
   and none of these is a wait the caller keeps:
   `fuse-overlayfs` is another type,
   served by a process of the machine,
   and stays refused with the types above.
   A copy of the descriptor answers a stalled server from its cached inode,
   but closing the copy runs that file system's flush,
   which a stopped server holds as it holds a lookup,
   and the copy is refused outright for a process this one may not attach to (`kernel.yama.ptrace_scope=1`),
   which would read as no holder at all.
   A descriptor whose record names the lock and which the scan then cannot identify
   (a mount it will not read,
   or a mount its process's table does not name)
   is a process that **may hold** the file:
   the candidate is unproven and nothing is removed,
   and the same holds for the lock removal of the index
   (`remove_if_unheld` keeps the lock when the scan reported one of these).
   A descriptor whose *record* cannot be read at all is the older case,
   a process the scan could not look at,
   which is tolerated as it always was.
   The gate and the read are two looks at the same number,
   and a program that puts a different file on that number between them is not making the read wait on a lock file:
   measured on this host on 2026-11-09 with the delay in the window,
   the record named the first file's inode and the read that followed answered with the second file's device and inode.
   Two things follow,
   and neither is a wait the scan can bound:
   the read identifies the file that is there now,
   so a holder whose descriptor was replaced is missed and a lock it holds can read as not held;
   or the file that is there now sits on a file system this gate would have refused,
   and the read asks that file system for attributes that `AT_STATX_DONT_SYNC` does not make it answer from memory,
   so the read can wait there.
   Closing that window means naming the file and reading it in one step,
   which this kernel offers only through the descriptor copy above,
   whose refusals and whose close are the two reasons it is not used;
   the coordinator's decision of 2026-11-09 is to keep this gate and document the window,
   because closing it needs a second process to act inside an instruction-sized gap:
   the holder of the descriptor calls `dup2` (or offers the new file through a `SCM_RIGHTS` message and `recvmsg`) on
   that very number,
   which is a few instructions of the holder's own code (`src/native/process/open_file_of_a_descriptor.rs`,
   the gate and the read of one descriptor),
   while this scan runs between its record read and its `statx(2)`.
   Nobody in this repository has done that,
   and no program the wrapper starts does it,
   so the window is a defect of this gate and not a case the wrapper meets.
   A lock that reads as not held for that reason is the case the words of every notice name:
   nothing here proves the holder gone,
   and the person is given the command that clears the file.
   The mount number the kernel writes for a descriptor is not an identity (audit of 2026-11-09,
   finding 4):
   a bind mount gives the same file another mount number,
   so a holder that reached the lock that way was not found at all,
   and two files of two btrfs subvolumes reached through one mount share the mount number and the inode number,
   so a process holding one of them made the other **held**.
   A process list that cannot be read,
   or a Git process of the account whose descriptors cannot be read,
   makes it unproven.
   A process of the account that is not Git and cannot be read is skipped:
   on the measured host `systemd --user` and
   `ssh-agent` are never readable (`CapEff` and set-group-id),
   so refusing on them would make every proof unreachable,
   and a
   program that is not Git does not create Git locks.
4. A pid file beside the lock authorizes nothing.
   It holds a process number,
   Git records no namespace with it,
   and the only fact that could tie that number to this lock is the birth order of the two files,
   which a correction of the clock turns around:
   after a backward step an old pid file looks newer than a live writer's new lock,
   and the number it names is a process that has ended
   (cross-vendor review of 2026-11-09,
   finding 3).
   The pid file is therefore read for the words of the notice and decides nothing:
   the notice says which number it names and whether that process runs,
   with the command that clears the file.
   The proofs it used to carry are rejected here:
   - a gone process does not prove the writer of *this* lock gone,
     because the pid file may be an earlier holder's,
     left in front of a later lock by a Git that writes none (`core.lockfilePid` off);
   - the namespace searches that confirmed a gone number
     (the initial PID namespace without `hidepid`,
     and a scan of the `NSpid` lines of `/proc/<pid>/status` for a process carrying the number in a nested namespace)
     are removed with it,
     since they answered a question no longer asked;
   - a running Git named by the pid file is not taken for the holder either:
     the number may be a Git's in a sibling namespace.
   What is left of this protocol is words,
   and the removal of a lock rests on the staging-file proof alone (item 6 below,
   `Proof::OrphanStaging`).
5. A lock whose times are earlier than the boot of the kernel is **not** a proof.
   `btime` of `/proc/stat` is the wall clock minus the uptime,
   so a forward correction of the clock after the boot moves
   `btime` and the times of every file alike;
   a lock that a live Git made before the correction then looks older than the
   boot,
   and a margin only moves the size of the correction that fools it.
   No second sign that is independent of the clock exists (no file carries a boot identifier),
   so this proof,
   which
   releases 0.5.23 to 0.5.26 had,
   is removed.
   The notice for a lock that looks older than the boot says so and says why it does not prove anything,
   beside the
   command that clears it.
6. Anything else is unproven:
   the lock is left,
   and the wrapper says what it found.

The removal itself takes no lock of the wrapper's own (`Use as few locks as possible.`),
and no file is ever removed by
its path after a separate look at it,
because the path may name a different file by the time of the removal.
Each file goes through one step:
the identity it had while it was judged (device,
inode,
size,
birth time and modification time) is compared once more
just before the move,
and a file that already differs is not touched;
then one `renameat2` with `RENAME_NOREPLACE` moves it to a private name in the Git directory itself (a name under `refs/`
would be read as a reference),
so the private name is never written over;
the identity of what was moved is compared with the judged one;
a match is removed,
and a difference is put back with a hard link,
which fails instead of replacing a lock that has been
created at the path since (that file is then left under its private name and the wrapper says so).
Two wrappers that judge the same lock at the same time therefore cannot delete a lock that a live Git created between them.
The step does not make the removal safe against every schedule,
and the residue is open for the owner (the ninth design
review,
finding R9-S,
2026-11-03).
If,
between the judgement and the move,
another cleaner removes the judged lock and a live Git creates a new one at the path,
the move takes that live lock away from its path for the moment before the put back.
A Git that tries to rename that lock into place in that moment fails loudly and loses nothing.
But if a third Git creates a lock at the path in that moment,
the put back fails (it never replaces),
two Gits then each
believe they hold the lock of one reference,
and the wrapper only reports the displaced file.
The second identity check just before the move leaves a window of microseconds instead of the length of the judgement,
and
the schedule needs three writers and another cleaner inside it,
but it is not closed:
Linux has no removal of a path
conditional on the file that stands there.
The choice,
made for now under the owner's delegation,
is to keep removing on a proof;
the owner is to be asked whether
to leave all removal to the person,
with the command,
instead.
The fix of 2026-11-03 for the container (the wrapper's own proof above) does not change the step,
so each removal has the
same residue as before;
it changes which locks are removed:
a container now removes a lock it saw its own stopped child hold (the claim),
and a
claimed lock of that child is judged without waiting out the margin,
while a lock whose pid file merely names an ended child's number is left and named
  (cross-vendor review of 2026-11-09,
  finding 2).
The occasions it adds have the fewest chances of the schedule:
the lock is the freshest,
and the sweep that judges it is
the first cleaner to look.
The pid file,
and for `packed-refs.lock` the staging file `packed-refs.new` that Git creates with `O_EXCL` while it holds that
lock (`refs/packed-backend.c:1398`),
are looked at before the lock is judged and go through the same step after the lock has
been removed,
each against that earlier look,
so a pid file or staging file that a live Git made at the same paths in
between is left.
A killed Git leaves the staging file too,
and a later packed-references update fails on it ("unable to create file
packed-refs.new:
File exists") exactly as on the lock,
so a lock is cleared together with it,
and the command given for a lock
that is not cleared names it.
A staging file with no lock beside it (the lock was removed by hand,
or by an older release) is an orphan:
Git makes it only
while it holds the lock and renames it away before it releases the lock,
so a file older than the margin that is still the same
file after the lock was looked for and found absent has no live maker,
and it is removed by the same set-aside step
(`Proof::OrphanStaging`).

#### Rejected rule: a reused number proven by a scan of the open descriptors

The cross-vendor review of 2026-11-09 (finding 1) proposed a narrower version of the removed proof:
keep the removal for a number that another program holds when a scan of every process's descriptors could read all of them and
found none holding the lock,
and keep the lock when a scan could not read a process.
The rule is rejected as well,
because a live Git holds every one of its reference locks with **no descriptor open**
(the measurement under "Measured while this entry was written" above),
so a scan that finds no holder says nothing about a holder that closed its descriptor and has not renamed its lock yet,
which is the state a `reference-transaction` hook leaves a lock in for as long as the hook runs.
Under that rule the clock would still be the only sign that separates a Git under another name from a later program that
took the number,
so a step of the clock would still delete a live lock,
and the review's own scenario (a holder of another account,
whose descriptors no unprivileged scan can read) would leave every such lock in place anyway.
The cost of removing the proof is one stale lock left and named,
with the command that clears it,
for the rare case in which a killed holder's number was taken again before the sweep looked.

#### The reftable backend's own lock

A repository keeps its references in one of two backends,
and each locks its own file.
The files backend keeps `packed-refs.lock`,
`HEAD.lock` and `refs/**/*.lock`;
the reftable backend keeps its references and its list of tables in the `reftable` directory of the Git directory,
and every writer holds `reftable/tables.list.lock` for the whole transaction
(`reftable/stack.c`,
`reftable_stack_init_addition`,
`reftable_addition_commit`).
A reftable repository holds none of the files-backend locks,
so before 2026-11-09 a killed Git's `tables.list.lock` was not a candidate:
a forwarded fetch or push did not clear it and did not name it,
and every later `git update-ref` or `git commit` failed with "cannot lock references" (audit of 2026-11-09,
finding 1).
Both backends' locks are therefore candidates,
whatever the repository's own format says,
because a repository that switched backends may hold either.
The reftable lock needs no new proof:
Git says which process holds it in `tables.list~pid.lock`,
which it creates after the lock under `core.lockfilePid` exactly as it does for the other locks,
so the pid file,
the margin,
the local-file-system rule,
the boot comparison and the set-aside step are the ones above.
The tables list is written into the lock itself and renamed,
so the lock is empty while it is held and a stale one is removed whole.
There is no staging file to go with it:
temporary table files (`*.temp.*`) and tables that no `tables.list` names are not locks,
they block no writer,
and they stay where they are.
Tests,
red first:
`a_reftable_tables_list_lock_left_by_a_killed_git_stays_and_is_named`,
with `a_live_reftable_gits_tables_list_lock_stays` and
`a_reftable_lock_that_nothing_proves_gone_is_left_and_named`.

#### A landing whose branch write a lock refuses

Git answers a refused reference update the same way whether the branch moved or a lock file stands,
and the service told them apart by asking the branch again after the refusal:
a branch that still names the commit the batch was built on did not move,
so a lock is what refused the write
(`src/native/landing/service_land_batch_body.rs`,
`src/native/landing/service_land_reference_lock.rs`).
A refusal that is not a moved branch spends no landing attempt,
so the sentence "the branch changed N times" is never said for a lock:
that sentence is said only for a branch that really changed.
A live holder lets go on its own within its step,
so the landing waits for the standing locks,
bounded by `REFERENCE_LOCK_WAIT`
(`src/native/service/wait_ceilings.rs`),
which is the margin a lock is judged at:
twice the 3,438 ms longest live reference step of the service journal.
The index lock is given up before the wait,
so another program's commit is not held behind a lock this landing only waits for.
The locks are judged and cleared by the judgement above,
through the same proof,
notice and set-aside removal,
and the landing tries its write again as soon as nothing stands.
No lock is removed on a claim about this service's own killed child,
for the reasons above.
The words of the judgement also go to the journal,
with the lock's size,
age and the evidence it was judged by.

##### A commit held behind a reference lock

A lock that is still there at the bound,
or one that a proof does not reach,
no longer ends the commit.
The owner chose,
verbatim,
"Make it,
hold until lock goes (Recommended)"
(2026-11-09),
and the rule it answers is "Commits should never fail."
The landing keeps what the batch already has:
its intent,
the scratch reference `refs/worktree/cli-git-service/<batch>` that holds its commits,
a commit real Git built included
(`src/native/landing/service_land_objects_of_batch.rs` keeps it there,
so no maintenance that drops objects no reference names can take it while the lock stands),
and the witness of the index file it saw,
and it publishes a record of the hold,
whole or absent,
before the client is answered
(`src/native/landing/held_commit_record.rs`).
The record names the two branch values,
the commits,
the journal,
the index,
the message the branch move writes in the reflog,
the locks that stood and the instant the commit was held,
in `monochromatic-git-policy-cli/held-commits/<batch>.held` below the worktree's own Git directory.
The batch's held push records and the record of the index update the batch would owe are given back,
because no branch names the commit yet and a batch whose branch has not moved owes nothing;
the settlement that moves the branch takes both again before that move,
so a kill between the move and its index install still leaves what the next command reads
(`src/native/service/service_server_land_group_owed_pushes.rs`,
`src/native/landing/held_commit_settle.rs`).
The answer is `ServiceMessage::Held` with the words,
the client prints them and exits `0`,
and the commit is on the side reference.
The service's line about the hold says what the answer says:
the commit is kept,
the branch moves when the lock is gone,
on this command or on the next one in that worktree,
and the line names the repository the batch belongs to as every line about a batch does.
A line a landing writes at the moment it is held says the branch has not moved yet,
because that landing had just read the branch;
a line a settlement writes from the locks alone says only that the move of the branch waits for them,
and says nothing about where the branch is,
because that settlement read nothing but the locks.
No state a settlement decides between calls the commit refused,
and `src/native/landing/held_commit_settle_outcomes.rs` words all six of them.

A kill at any instruction leaves a state the next command settles again,
and the commit is never lost and never applied twice:
a kill before the record leaves the intent and the side reference,
which a restart drops with the commit kept on `refs/cli-git/kept/<batch>`;
a kill after it leaves a record whose batch is settled from the branch.

Every command in the worktree reads the records and says what each held commit waits for:
a heading with the count,
then one line per record naming the branch,
the commit and the lock,
with the exact `rm --` that clears it when no Git process uses the repository.
A command whose output a program reads prints nothing and settles the same.
The first command that finds the locks gone settles them,
and so does the service before its next batch and at its start
(`src/native/landing/held_commit_settle.rs`).

The settlement reads the branch and moves it by one compare-and-swap,
with the side reference removed in the same transaction.
Every round reads the batch's record again after it reads the branch,
because another settlement of the same record rewrites that record before it moves the branch:
a decision taken from the value read before that rewrite would put one held change on the branch twice
(`src/native/landing/held_commit_settle.rs`).
A record a kill leaves after its batch landed on a branch that moved names the commit the batch started with,
while the branch names the commit made from it:
the finished record (`<seq>.done`) names the commit that landed,
so the record is recognized as landed,
said and removed instead of being reported as never landed for ever.
The index entries are installed under Git's index lock,
which the settlement takes before the move and records the witness of under it,
so nothing another program staged is overwritten and the entries are not left to the
owed record when the locked commands of a worktree have touched the index since the landing.
A branch that moved while the lock stood takes the change merged onto the tip from the base the
batch was prepared on
(`git merge-tree --write-tree --merge-base`,
the primitive "Retry after a moved branch" uses),
and the commit is written again with its own authorship,
committer and message,
each commit of a batch on the one before,
its one parent the tip
(`src/native/landing/held_commit_replay.rs`).
A path that conflicts is not landed:
the record stays,
the commit stays on its side reference and the paths are named.
A commit that concludes a merge,
one prepared for a branch that did not exist and that another session created,
and a signed commit are kept and named the same way,
because a command that resumes a commit runs no policy,
no hook and no signing program,
and no command starts a wait for a key on its own.

Nothing released the commit's push:
the command that made it had ended,
so no post-commit gate ran.
The next commit or push from the branch carries the commit,
which the held push notice says.

Tests,
red first:
`a_standing_lock_keeps_the_branch_without_losing_the_commit`,
`a_held_commit_lands_once_the_lock_is_gone`,
`a_held_commit_lands_on_the_branch_another_session_moved`,
`a_held_commit_that_conflicts_keeps_its_record_and_names_the_paths`,
`a_record_reads_back_as_it_was_written`,
`holding_writes_the_record_the_intent_describes`,
and the binary controls
`a_commit_that_meets_a_lock_nothing_proves_gone_is_held_and_moves_the_branch_when_it_goes`,
`a_held_commit_lands_on_the_branch_another_session_moved`,
`two_held_commits_wait_beside_each_other_and_both_land_in_order` and
`a_held_commit_real_git_built_is_kept_on_the_side_reference_and_survives_a_prune`,
with `a_second_settlement_of_one_record_lands_the_change_once`,
`a_record_left_by_a_kill_after_its_replayed_landing_is_recognized_and_removed`,
`the_held_record_of_the_open_batch_of_a_request_is_found`,
`a_held_commit_ends_the_command_with_success_and_no_landed_commit` and
`a_held_commit_of_a_removed_linked_worktree_survives_on_the_kept_reference`.

#### Races, and how each is handled

- A holder that wrote its lock,
  closed the file and has not yet renamed it:
  its pid file names a live Git,
  which the notice says and which decides nothing,
  and the lock is unproven,
  never stale,
  because the descriptor scan sees nothing.
- A holder that is killed between creating the lock and its pid file:
  there is no pid file,
  so the lock is unproven.
  Such a lock is reported with the command that clears it,
  and a live foreign writer's lock cannot be told from it.
- An older pid file in front of a live holder:
  said in the notice and nothing else,
  since the birth order of the two files is the only thing that could tie them and a clock step turns it around.
- A process number reused by another process:
  Git records only a number,
  so nothing tells a running Git under another name from a later
  program that took the number,
  and nothing tells either from the writer of this lock.
  The lock is unproven,
  with the command that clears it
  (cross-vendor reviews of 2026-11-09,
  findings 1 and 3).
- Another account's process:
  it cannot be read,
  but a lock it holds has its pid file naming that process,
  which a running
  number never proves stale.
  A lock with no pid file stays unproven,
  and the descriptors it cannot see are said in the message.
- A Git in another PID namespace (a container with the repository mounted in,
  or a nested namespace on the host):
  the pid file holds a number of that namespace.
  Git records no namespace,
  so the number says nothing about the writer of this lock whichever namespace the wrapper runs in,
  and no lock is removed on it.
  A number that matches a member of a tree the wrapper stopped says nothing either:
  a sibling namespace may have given the same one to a Git that shares the repository
  (cross-vendor reviews of 2026-11-09,
  findings 1 and 2),
  so every such lock is named instead.
  The veto of an open descriptor and the margin are the only judgement beyond that.
- A Git on NFS or another network or user-space file system (including a virtual machine's shared directory):
  the file
  system must be on the fixed local list.
  A lock on a local disk is still matched to a descriptor that holds a file on a stalled file system,
  and nothing there is waited for:
  a descriptor on a mount that is not on the local list is refused,
  and the lock is unproven (item 3).
- A clock that steps:
  the margin and the start time of a process read file times against the current clock,
  and the boot
  comparison is only worded,
  never a proof (item 5).
  A forward step makes a young lock look old to the margin,
  which only lets the judgement look at it;
  what it concludes
  rests on the pid file and the process table,
  not on any time.
  The derived start of a process moves with the clock,
  which is why no verdict rests on it either (item 4).
- Two wrappers removing the same stale lock:
  handled by setting aside with a rename and checking identity (above).
- A removal that is raced by a new lock at the same path:
  the file that was set aside is checked,
  and put back if it is not
  the judged one;
  if the path has been taken again by then,
  it is left set aside under its private name and the wrapper
  says so,
  because deleting it could lose a live holder's update.
  The same holds for the pid file and the staging file,
  which release 0.5.26 removed by their paths.
- A foreign writer that writes no pid file and keeps its lock empty with the descriptor closed (finding S-1):
  nothing
  proves it gone,
  so its lock is left and reported,
  whether it was born before or during the life of a stopped child.

#### Where it applies

- The wrapper's own pushes and fetches (the commit service's auto-push and its reconcile fetch,
  the stacked push,
  the Git LFS
  upload's pushes) all start through `run_bounded_push`,
  which judges and clears first (`src/native/lock/push_lock_clearing.rs`).
  Every repetition of a push starts through it too,
  so the service's repetitions after a refused tracking reference
  (`PUSH_LOCK_RETRIES`) clear again before each one;
  no repetition of its own was added,
  and nothing waits.
  These are the commands that run unattended,
  so a lock that blocks them is repaired with nobody present.
- The sweep after a stopped child (the push deadline entry of "Auto-push") judges the same locks once the child's tree has
  ended,
  removes the provably stale ones,
  and logs the notice of the rest;
  it removes nothing the judgement does not prove.
  Its wait for the rest of the tree is bounded by `STOP_CEILING`,
  the bound every other wait of a stop has:
  a descendant wedged in a dead file system,
  or one that left the process group the signal reached,
  then leaves its locks to that judgement instead of leaving a task waking every ten milliseconds for the life of the
  service.
- A plain forwarded `git fetch` or `git push` replaces the wrapper process with Git,
  so there is no "after" to look at:
  the
  wrapper judges and clears before it hands over,
  and the notice (standard error) precedes Git's own words
  (`src/native/lock/forwarded_lock_notice.rs`,
  called from `stack_if_push` in `entry.rs`).
  The repository is the one the wrapper's `-C` options and `GIT_DIR` name.
  A push that the wrapper runs as a child (stacked or with an LFS upload) goes through the first item.
- The cost on a command with no lock is one walk of `refs/` of the Git directory and the common Git directory plus two file
  lookups:
  measured while this entry was written at a median 1.9 ms (25 runs,
  1.85 to 2.15 ms) for 5000 loose references in 50 directories,
  and no change with an unproven lock younger than the margin.
  A lock old enough to be judged adds one scan of the descriptors of every process:
  median 53 ms (25 runs,
  47 to 63 ms) with
  1182 processes on the host,
  paid only when a lock exists.
  No Git is started.
- Not covered:
  `git pull`,
  `git merge` and every other command (they belong to the merge and index-writer paths);
  a command that
  names its repository with `--git-dir` or any global option other than `-C`;
  the wrapper's own `update-ref` and other ref writes
  of a commit;
  locks younger than the margin,
  which a command meets as Git's own refusal (the next push clears them once they have
  aged);
  and the notice of a push that Git delivers and exits 0 for after being refused the tracking
  reference (`cannot lock ref` in output with exit 0):
  the lock itself is judged before the push starts and never after it,
  so a lock taken during the push leaves the reference behind,
  the retry loop of `auto_push.rs` being left as it is;
  what the person reads is Git's own refusal,
  carried by the report the service's next record shows,
  so the local view staying behind is never silent.

#### Messages as printed

Every line starts with `monochromatic-git-policy-cli: `,
and every path is quoted for a shell (a plain name as it is,
other printable names in single
quotes with `'\''` for a quote,
names with a control character or a byte that is not UTF-8 as `$'...'`),
so the command can be
pasted and a terminal escape in a file name is never printed raw.

```text
# A lock whose pid file names a process that has ended: said, and nothing removed.
monochromatic-git-policy-cli: left the reference lock /r/.git/refs/remotes/origin/gone.lock alone: a pid file beside it names process 2629996, which has ended, and nothing ties that number to this lock: Git writes a number and no namespace, and only the times of the two files could tie them, which a correction of the clock turns around.
# A lock a running Git holds.
monochromatic-git-policy-cli: the reference lock /r/.git/packed-refs.lock is in use: process 2630147 (git), which the pid file beside it names, is running; it was left alone.
# A lock nothing proves stale.
monochromatic-git-policy-cli: left the reference lock /r/.git/refs/remotes/origin/gone.lock alone: no pid file beside it names a holder (Git writes one only under `core.lockfilePid`), and no process has it open.
monochromatic-git-policy-cli: it is 0 bytes, made 2026-11-03T11:57:55.882Z (2 seconds ago). If no Git process is running in this repository, clear it with:
monochromatic-git-policy-cli:   rm -- /r/.git/refs/remotes/origin/gone.lock
# A lock whose times are earlier than the boot, with no pid file: the times are said, and prove nothing.
monochromatic-git-policy-cli: left the reference lock /r/.git/refs/heads/c.lock alone: no pid file beside it names a holder (Git writes one only under `core.lockfilePid`), and no process has it open; its times are earlier than the start of this boot, which a reboot would explain, but so would a correction of the clock, so the times alone prove nothing.
monochromatic-git-policy-cli: Git writes a process number beside its locks only when `core.lockfilePid` is on. The wrapper turns it on for the Git it starts; `git config --global core.lockfilePid true` does it for the rest, so the next lock can be proven.
```

For `packed-refs.lock` the `rm` line also names `packed-refs.new` and the pid file when they exist.
Git's own refusal follows these lines unchanged,
because the wrapper becomes Git.
In a process that is not in the initial PID namespace,
or whose `/proc` hides processes,
the reason of a lock with a pid file is "the
pid file beside it names process N,
but this process runs in a PID namespace of its own,
or cannot see every process of the machine,
so it cannot tell whether N is gone".
Where a process of a nested namespace has the number,
it is "the pid file beside it names process N,
and process M is process N of a
PID namespace of its own,
so the writer may be that process".
The lock is left with the same `rm` line in each case.

#### Measured while this entry was written (see the commit time of the entry)

With real Git 2.56.0 (`git update-ref --stdin` told `start`,
three `delete`/`update` lines,
`prepare`,
then held,
and `git fetch
--prune` held by a `reference-transaction` hook at `prepared`):

- A live Git held four locks (`packed-refs.lock`,
  three reference locks) and their pid files with **no lock file descriptor open**:
  its descriptors were its three standard streams and nothing else.
  Every reference lock was empty,
  including the one whose reference already had the wanted value.
- A Git killed with `SIGKILL` in that state left all of them,
  and `packed-refs.new`.
- A push that delivers and then cannot lock the tracking reference prints "error:
  update_ref failed for ref ...:
  cannot lock ref"
  and exits 0.


### Real Git starts with traditional hooks off

External hooks are not only unsupported:
no start of the real Git by the wrapper runs one.
Every such start,
forwarded commands as well as the wrapper's own children (queries,
the commit's native steps,
the auto-push,
the reconcile and integration pushes,
maintenance),
carries
`-c core.hooksPath=/dev/null/cli-git-no-hooks` after the caller's global options and before the subcommand.
The directory is below a device,
so nothing can create it,
and Git finds no hook file there.
The same prefix carries `-c hook.<event>.enabled=false` for every hook event Git knows,
`pre-push` included,
so a hook
defined in the configuration (`hook.<name>.event` with `hook.<name>.command`) does not run either.
The same prefix carries `-c core.fsmonitor=false` for the Git the wrapper starts for its own work:
the commit's index builds and trees,
`read-tree`,
`diff-tree`,
`reset`,
the scratch commit,
the landing and service steps,
and its queries.
A command the person typed is forwarded and keeps the person's setting.
Git's own file-watching daemon registers a watch on every directory of the work tree before it answers,
and a Git that reads the index waits for it for up to `fsmonitor.startTimeout` (60 s by default),
so a commit step must not depend on the daemon being there or starting;
two steps of one commit waited 60 s each for it
(`doc/troubleshooting/git-fsmonitor-daemon-start-wait.md`).
A program `core.fsmonitor` names is not reached by the hooks path or by any event switch
(measured on Git 2.56.0,
including with `hook.fsmonitor-watchman.enabled=false`),
which is why the setting itself is passed.
Owner decisions,
in order,
quoted with the date each was superseded:
"To stop the fsmonitor program - we don't need to."
(kept the wrapper from stopping the monitor;
superseded 2026-11-03),
"If git's fsmonitor is fundamentally broken and cannot be easily or meaningfully worked around,
disable git's fsmonitor globally entirely and make doctor say so."
(2026-11-03;
recorded in `README.md`,
"Git's own fsmonitor"),
and,
asked "Should the wrapper switch fsmonitor off for its own internal Git commands only?",
"B:
off for internal Git (Recommended)"
(2026-11-10;
`doc/handover/questions-for-the-owner.md`,
"The wrapper's own Git and fsmonitor";
this amends the decision of 2026-11-03,
which stands for the person's own Git).
So the setting is off in the person's global configuration,
`git monochromatic-git-policy-cli doctor` says so,
and the wrapper still neither starts nor stops a monitor daemon.
The wrapper injects no hook of its own,
so no event is an exception.

- The option is part of the argument vector,
  so it adds no system call and no process start.
  The three commit controls of `binary_budget_tests.rs` pass with their counts unchanged.
- It follows every global option of the caller and precedes the subcommand,
  so no `-c core.hooksPath=<directory>`,
  `-c hook.<event>.enabled=true` or `--config-env` of the caller can bring a hook back,
  and the native commit repeats
  the options after the caller's (see "External hooks").
- A command that reaches Git through an alias gets Git's own expansion from the wrapper (resolved from the alias
  values it already reads for classification),
  so the options follow the `-c` options an alias value may carry;
  put before the alias word they would lose to them (measured on Git 2.56.0).
  An alias Git refuses or does not select is forwarded as the caller typed it instead,
  so those options no longer reach a command Git will not run (see "Discarding commands").
  A shell alias (`!...`) is run by Git's shell,
  which finds real Git's own `git` first on `PATH`,
  so the command inside it runs with the options Git gives that shell and not with these.
- `git config` starts that only report or change configuration go without the options,
  because they report what the
  repository configures and the wrapper reads `core.hooksPath` and `core.fsmonitor` that way (the commit warning,
  `doctor`).
  `git config --edit` (or `-e`) starts an editor,
  and a Git the editor starts inherits the options,
  so it gets them.
- A child of Git inherits the settings through `GIT_CONFIG_PARAMETERS`,
  so a program Git starts (a filter,
  an editor) that
  starts `/usr/bin/git` runs it with hooks off as well;
  a local-path remote's `receive-pack` does not inherit it (measured by
  `a_local_remotes_server_side_hook_still_runs`),
  so server-side hooks of such a remote still run.
  A script that starts Git outside any wrapper-started Git still runs traditional hooks.
- git-lfs reads the same setting when it picks the directory for its shims:
  a clean filter started inside a
  wrapper-started Git finds no directory it can write and writes none (measured with git-lfs 3.8.0;
  `binary_hooks_off_tests.rs` has the positive control,
  a direct `git add` that writes all four).
- The wrapper's dry-run push probes (the scope probe of a manual push,
  the integration probe after a rejected push,
  and
  the stacked push's own scope probe) carry `-c hook.pre-push.enabled=false` and a hooks directory that never exists,
  and a `--dry-run --porcelain` push reports its updates without any hook.
  No push runs a configured `pre-push` hook:
  the event is switched off like every other,
  and the wrapper injects no
  hook of its own.
- Measured with Git 2.56.0 only;
  Git 2.39 was not available on the host.
  A missing `core.hooksPath` directory is the case the shadow commands already rely on (`<shadow>/no-hooks`),
  and the container end-to-end suite runs that on Git 2.39.5.

### The wrapper never starts itself as Git

The wrapper looks for the real Git on `PATH` and starts it;
nothing it starts as Git may be a monochromatic-git-policy-cli wrapper.
The owner's rule:
"We should never invoke ourselves recursively."
(`doc/planning/cli-git-no-self-invocation.md`,
which
holds the incident of release 0.5.26,
2026-11-03,
the cause,
the options weighed and the controls.)

- Selection skips a candidate that leads back to monochromatic-git-policy-cli:
  the running executable through any link,
  spelling or
  byte-identical copy,
  a script that starts the TypeScript wrapper,
  and a toolchain launcher of the wrapper.
  A launcher is recognised by the record `<name>.launch` that sits beside it (or beside the file its links reach):
  the record's `toolId` ends in `monochromatic-git-policy-cli`,
  or its `real` is this executable.
  The record is read as JSON,
  up to 64 KiB,
  with no process started.
- Every start of the real Git sets `CLI_GIT_STARTED_AS_GIT` in the environment of the start:
  the starter's process number,
  its birth identity and the mode,
  `replace` when the starter replaces itself with the
  program (a forwarded command) and `child` for every other start (queries,
  commit steps,
  pushes,
  maintenance,
  the
  service's Git work).
  The builders that set it are `git_command` in `forwarding.rs`,
  `replace_process_with_real_git`,
  `git_command` in
  `auto_push_single_flight.rs`,
  and the two starts with a cleared environment,
  `ask_git_var` and `status_git_dir`.
- The marker also counts the starts of one chain:
  the fifth field of `CLI_GIT_STARTED_AS_GIT` is one more than the number the marker of
  the shell that started this process carried,
  and a marker written without it,
  as releases before this one wrote,
  reads as the first start.
- A monochromatic-git-policy-cli executable stops before it plans anything when the marker says it is the process of a `replace` starter
  (same process number,
  same birth),
  or the child of a `child` starter (the marker names its parent,
  same birth),
  or when one chain holds more than `MOST_STARTS_IN_ONE_CHAIN` starts
  (`src/native/invocation/started_as_git.rs`).
  A hook,
  an alias or an editor that the real Git starts has another process in between,
  so running an installed
  wrapper there is not stopped,
  and such a nesting costs one real Git run per level,
  far below the bound.
  An identity that cannot be read decides by the numbers.
- The stop is exit status 2 with one message that names this executable,
  the starter and how it started this process,
  the file the starter chose as Git (`CLI_GIT_NATIVE_FORWARD_TARGET`),
  the `PATH` it saw,
  why it stops,
  and what to do:
  put the directory of the real Git (for example `/usr/bin`) on the `PATH` of the program that started the first
  monochromatic-git-policy-cli,
  or remove the entry that leads back to monochromatic-git-policy-cli from that `PATH`.
- No real Git on `PATH` falls back to the Git the commit service was set up with,
  when a record names one that passes
  the same checks (below).
  Without a usable record it is a stop with exit status 2.
  The message gives the counts,
  the
  `PATH` value,
  every candidate with its verdict (skipped because it leads back to monochromatic-git-policy-cli,
  or missing,
  not executable
  or unreadable),
  the record when a unit named one that was refused,
  and the ways out:
  install Git,
  add the real Git's
  directory to the `PATH` of the program that ran the command,
  or remove an entry that should not be used.
  The wrapper
  does not search outside `PATH` except for the promotion of a conventional location that `PATH` already exposes and
  this record.
- The service mode (`monochromatic-git-policy-cli monochromatic-git-policy-cli service run`) is never started by the wrapper and refuses a `--git` that is the
  wrapper itself,
  a link or copy of it,
  or a launcher of it (a unit written by a release that had the flaw holds such a path);
  the unit text includes `--git`,
  so the next wrapper that
  provisions the unit writes it again.
  Its Git starts go through the marked builders.
- Covered since 2026-11-09,
  where releases before it were not:
  a user script named `git` that starts the wrapper as a child,
  with no launch record and no marker
  string.
  The parent of the arriving wrapper is the script,
  not the starter,
  so the two checks above it match nothing;
  such a chain starts one wrapper per level,
  and the count of the marker ends it at
  `MOST_STARTS_IN_ONE_CHAIN`,
  whatever number of processes stands between two starts
  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
  finding 1,
  measured there at sixteen levels and no end).
  The bound is 32 starts,
  and the count means what it meant before the environment of the process above is read:
  one start of real Git per level.
  It was 8,
  which refused a finite chain:
  nine `!` aliases that each run the next through `git` reach the wrapper once per alias on an install
  whose Git exec directory offers the wrapper,
  and the command itself is a tenth start,
  so the refusal ended them at start 9
  (the shape the round-1 review reported).
  On an install where `git` inside an alias is the system Git the chain never reaches the wrapper at all
  (measured:
  the marker's count is 1 at every one of nine levels,
  and the wrapped `git a` answers with real Git's own output),
  so this count is the only stop only on the install whose `git-core` leads back.
  It now allows a chain three times the measured one,
  and a chain that leads back without a real Git of the person's between its levels ends at the 33rd start,
  measured at 1.3 s from the first wrapper to the refusal.
  Test:
  `a_finite_chain_of_aliases_reaches_the_command_the_person_asked_for`.
  The stop is the same exit status 2 and the same message:
  it says this is start N of one chain,
  names the file the previous start chose as Git
  (`CLI_GIT_NATIVE_FORWARD_TARGET`,
  which is the script),
  the `PATH`,
  and both ways out.
- Covered since 2026-11-09T03:00:00-05:00,
  where the count alone did not:
  a launcher that clears the environment of the wrapper it starts
  (`env --ignore-environment`).
  The marker lives in the environment of the start,
  so a clearer removes it from the arriving wrapper and every level looked like a first start.
  What survives the wipe is what the wrapper reads about the process above it:
  this process's own environment first,
  then the environment of the process that started it and of the one above that
  (`ANCESTORS_ASKED_FOR_THE_CHAIN`,
  `chain_marker_and_where`),
  which the clearer cannot change,
  and the count continues from there,
  so the chain ends at `MOST_STARTS_IN_ONE_CHAIN`.
  The message is written from the environment the marker was read in,
  so it names that Git and that `PATH`
  even though the arriving wrapper's own environment held neither.
  The walk costs 0.18 ms of a forwarded command,
  measured by alternating two builds that differ only in it
  (median 4.76 ms without it,
  300 pairs);
  it is made only when this process's own environment carries no marker.
  Test:
  `a_launcher_that_clears_the_environment_ends_the_chain`.
- Covered since 2026-11-09T05:00:00-05:00,
  where neither the marker nor the environment of the process above could see it:
  a launcher that clears the environment **and replaces itself with the wrapper**
  (`exec env --ignore-environment ... wrapper`),
  so one process number runs the launcher,
  the clearing and the wrapper again
  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
  finding 1 of the round-2 review).
  What an `exec` keeps is the open descriptors,
  so the chain record is one:
  the first wrapper of a chain makes a memory file with `memfd_create`
  (`invocation/chain_starts_record.rs`,
  no close-on-exec flag),
  every start of real Git writes the marker it gives that Git into the file,
  and the file records the path it chose,
  and every child inherits the descriptor.
  A wrapper reads that record after any marker that does not refuse it,
  including one an ancestor still holds:
  a launcher that replaces itself with the wrapper leaves its marker only in the record,
  and the older marker above names the outer start and matches nothing,
  so reading the record only when no marker exists let the process start itself without end
  (finding 1 of the round-3 review).
  The record continues the chain when its start is this process or any live process above it,
  up to the first process of the host,
  which covers a launcher and however many relays it starts (finding 2 of the round-3 review and
  finding 1 of the round-4 review);
  the walk costs one read of the kernel's process record per level and is made only when a record
  exists,
  so a first command pays nothing and a start that follows one paid 1.05 ms on this host,
  measured against the same start without the walk
  (paired median +33.7 ms over the 32 starts of one chain,
  inside the 126 to 143 ms run-to-run spread
  of either arm);
  a record naming any other process is a command that has ended,
  so its count starts again at one and a long session of one command after another is never refused.
  A read of the wrapper's own continues the chain from any process of it,
  as the marker path refuses it from any process,
  or the branch a relay starts from a recorded read would be uncounted.
  A tree the walk cannot read above some level starts the count again there;
  no chain of finishable depth was found that the walk misses.
  A memory file has no directory entry and no name to sweep:
  the kernel frees it with the last process that holds it,
  so a marked start costs nothing.
  What the record must never be is one of the three descriptors a child reads and writes as its
  streams:
  `memfd_create` takes the lowest free number,
  which is `0` when standard input is closed,
  and the wrapper then wrote its marker into it and a child hashed the marker as its input
  (finding 4 of the round-3 review);
  the record is moved to `F_DUPFD 3` and the number the kernel gave is closed,
  so a stream that was closed stays closed.
  The record costs nothing a forwarded command can feel:
  measured by alternating two builds that differ only in it,
  300 pairs,
  paired median −0.02 ms,
  mean −0.04 ms,
  against a 6.3 ms median and a spread of 4.4 to 10.3 ms.
  Tests:
  `a_launcher_that_clears_the_environment_and_replaces_itself_ends_the_chain`,
  `a_record_of_this_process_is_read_even_when_an_older_marker_sits_above`,
  `a_record_of_a_live_process_above_continues_the_chain_through_relays`,
  `a_chain_of_relays_between_two_starts_still_ends`,
  and
  `a_record_never_takes_the_descriptor_a_child_reads_as_standard_input`.
- What a cleared environment still hides:
  a launcher that closes every descriptor it did not open and then `exec`s the wrapper.
  The record is the descriptor,
  so a launcher that closes it leaves the next level with no marker and no record,
  which is a first start again;
  the forking shape is covered by the environment of the process above,
  and the `exec` shape only by the record.

### The wrapper falls back to the Git its commit service was set up with

Recorded on 2026-11-03T11:26:52-05:00;
the pick is the coordinator's,
made before the build under the owner's
delegation ("Pick options as if I had picked them and invite veto when I say I'm back."),
open to the owner's veto:
when no real Git follows the wrapper on `PATH`,
the wrapper uses the real Git it recorded for its commit service,
with a notice,
instead of stopping.
The reason:
native Git in that `PATH` would not be found at all,
but once the wrapper is found the person expects `git` to work;
the owner's rules "Commits should never fail."
and
"never leak internal things that could be resolved to the user"
(`package/git-policy/cli/DESIGN.md`,
2026-11-01).
The deliberation is in `doc/planning/cli-git-no-self-invocation.md`,
section "Falling back to the Git the commit service was set up with".

- The record is the `--git` option on the `ExecStart` line of the unit `monochromatic-git-policy-cli@.service` that the wrapper writes below
  `$XDG_RUNTIME_DIR/systemd/user` (`service_provision.rs`) when it sets up the commit service.
  It is read only when
  `PATH` offers no real Git,
  only from an absolute `XDG_RUNTIME_DIR`,
  as text up to 64 KiB,
  with no process started
  (`real_git_record.rs`).
  No other file is read and no location is guessed:
  with no unit,
  no `--git` word,
  a relative
  path,
  or an unset runtime directory,
  there is no record and the stop stays.
- The recorded path passes the checks of a `PATH` candidate (`classify_candidate`):
  it must exist and be an executable
  file,
  and it is refused when it is this executable,
  a link or a byte copy of it,
  a toolchain launcher of it,
  or a
  script that starts the TypeScript wrapper.
  Another release of the wrapper passes the check and is stopped on arrival
  by the marker.
  A refusal keeps the stop and adds one line naming the record,
  the unit and the verdict.
- The notice is one line on standard error,
  ahead of anything else the invocation writes,
  once per planned invocation
  (the merge push planned inside a stacked push does not repeat it),
  in this form;
  an empty or unset `PATH` is printed
  as `empty or unset`:

  ```text
  monochromatic-git-policy-cli: no real Git follows this wrapper on PATH (<PATH>); used <path>, the Git recorded in <unit> as the one the commit service starts; to silence this notice, add <directory of path> to the PATH of the program that ran this command.
  ```

- The same resolution serves every start of the real Git:
  forwarded commands,
  the announced queries,
  the commit steps,
  the push work,
  `git monochromatic-git-policy-cli doctor`,
  the direct commands,
  `service status`,
  `service settle-earlier-batches` and `convert-worktree-to-subvolume`.
  The service takes the resolved path in the `--git` of its unit as before;
  a fallback path is therefore written into
  the unit it came from,
  unchanged.
- Limits:
  the unit is in the runtime directory,
  so it exists only after a commit has provisioned the service in this
  login and is gone after the account's last logout;
  before that,
  the stop stays.
  A service started with no
  `--git` still defaults to `/usr/bin/git`;
  that is a development path,
  since the wrapper always writes `--git`.

### Git LFS upload before a push

A push publishes pointer files;
the objects they name must reach the LFS server first,
or collaborators fetch pointers
whose content does not exist.
Git LFS does this in its `pre-push` hook (`git lfs pre-push`),
which `git lfs install` writes as a traditional hook file.
Traditional hooks are unsupported,
so the wrapper provides the same step itself,
as a child it runs before the push.

Verdict per hook of the four `git lfs install` writes
(source:
`commands/command_pre_push.go`,
`command_post_checkout.go`,
`command_post_commit.go` and
`command_post_merge.go` of git-lfs,
and the man page `git-lfs-pre-push`):

- `pre-push`:
  handled natively,
  as described below.
- `post-checkout`,
  `post-commit` and `post-merge`:
  not needed.
  Each returns before doing anything unless a lockable pattern exists
  (`lfs.setlockablereadonly` is on by default,
  and `GetLockablePatterns()` is empty without the `lockable` attribute);
  what they do otherwise is set the read-only flag of lockable files that are not locked.
  The repository has no `lockable` attribute:
  no tracked `.gitattributes` file contains the word,
  and `git check-attr --all` on an LFS path lists none.
  A repository that adopts `lockable` reopens this verdict;
  until a hook or a wrapper step runs,
  its lockable files
  keep whatever write permission checkout gave them,
  and no content is lost.
  The wrapper runs none of the three,
  and a shim file for any of them never runs through the wrapper either.

The step:

- Trigger.
  The wrapper uploads before a push only when the repository's own Git configuration (local,
  worktree or
  command-line scope) holds a key under `filter.lfs.` or `lfs.`,
  and the push is not a dry run.
  A key in the global or system file does not count:
  `git lfs install` writes `filter.lfs.*` there for every
  repository of the account,
  and the filter's first run in a repository writes `lfs.repositoryformatversion` to its
  local file.
  The decision costs no extra process on the commit path:
  the configuration snapshot the commit already reads is searched in memory.
  A forwarded `git push` reads the configuration once (one `git config --list` start) before it runs;
  every other forwarded command is unchanged.
  When the configuration cannot be read,
  the upload is attempted.
- Mechanism.
  The wrapper first decides in process which LFS objects the pushed commits introduce (`lfs_object_plan.rs`,
  `lfs_pointer_scan.rs`,
  `lfs_pointer.rs`,
  on the in-process readers of `repo_read_*.rs`),
  and starts `git-lfs` only
  when there is something to upload.
  The tips are the plan's refs (a branch or tag name,
  `HEAD`,
  a full ref name or an object name;
  a revision expression
  is a decline).
  The commits to read are those reachable from the tips and not from the remote's tracking refs
  (`refs/remotes/<remote>/*`),
  found by a walk ordered by committer time as `git rev-list` orders it,
  so the history
  behind a known tip is never read;
  when the destination is untrusted (see below) nothing is known and every commit
  of the pushed refs is read.
  Each commit is compared with each of its parents,
  descending only into trees that
  differ,
  so a pointer that one commit adds and a later commit removes is still found.
  A blob is a pointer exactly
  when `git-lfs` 3.8.1 would take it for one (`lfs/pointer.go` `DecodePointer`,
  `lfs/scanner.go` `BlobSizeCutoff`,
  `commands/uploader.go` `prepareUpload`):
  under 1024 bytes,
  a header word `git-media`,
  `hawser` or `git-lfs`,
  the lines
  `version`,
  `oid sha256:<64 hex>`,
  `size <n>` in order with a known version,
  size above zero;
  the path and
  `.gitattributes` are not consulted,
  and a pointer with extension lines is a decline.
  No pointer found:
  no `git-lfs` process starts and no request reaches the LFS server.
  Pointers found,
  every one present
  in `.git/lfs/objects`:
  one `git-lfs push --object-id <destination> <oid>...` run per 500 objects.
  A missing local
  object,
  a repository the readers decline (replace refs,
  shallow,
  grafts,
  unusual environment variables or global
  options,
  a tree or ref in a spelling they do not read,
  a walk of more than 100000 commits or 3 seconds),
  leaves the
  next paragraph's `git-lfs` scan in charge,
  which also words a missing object and honours `lfs.allowincompletepush`.
  The `--object-id` form does not verify locks,
  as the scan form does.
  The scan form runs `git-lfs push [--all] <destination> [<ref>...]`,
  as one child the wrapper owns per destination,
  before it starts the `git push`;
  the push carries no hook and no injected `-c`.
  The wrapper never starts its own executable,
  never writes shell text or scripts,
  and explains a failure from the
  parent process that owns the child (`doc/planning/cli-git-no-self-invocation.md`).
  `git lfs push <remote> <ref>` alone skips every object that a remote-tracking ref of the remote mentions,
  so a push
  that adds no LFS object sends no batch request.
  Those refs need not describe the destination being pushed to (a fetch
  from one server and a push to another,
  a changed push URL),
  which would publish pointers without objects,
  so `--all`
  (everything reachable from exactly those refs,
  a superset and idempotent as objects are content addressed) is kept
  whenever the configuration the plan already holds says the tracking refs may not describe the destination:
  it is not
  a configured remote (a URL,
  a path,
  a name nobody configured),
  the remote has any `pushurl`,
  any
  `url.<base>.pushInsteadOf` rewrite exists,
  or the push is a mirror push.
  A configured remote with no tracking refs
  needs no check:
  the bounded form then has nothing to skip and scans the whole refs itself.
  Measured with git-lfs 3.8.1 (`commands/uploader.go`,
  `uploadForRefUpdates`,
  `uploadRangeOrAll`):
  without `--all` the
  scan is bounded by the remote's tracking refs (`ScanMultiRangeToRemote`,
  `rev-list --not --remotes=<remote>`);
  with
  it the whole ref is scanned (`ScanRefWithDeleted`).
  Both forms build the same scanner (`NewGitScannerForPush`),
  which
  runs `git show-ref` and `git ls-remote --heads` against the remote (`calcSkippedRefs`),
  so dropping `--all` adds no
  network round trip and no Git start.
  No tighter bound exists without one:
  `git lfs push` takes no remote object name,
  and the `exclude` list of `uploadForRefUpdates` is empty on this path.
  The child runs with the real Git's directory first on `PATH`,
  so a `git` that `git-lfs` starts is the real Git and not
  the wrapper,
  but the `git-lfs` it starts is the first one the caller's own `PATH` finds (as an absolute path,
  so a relative `PATH` entry and
  `-C` cannot make the file that was checked differ from the file that runs).
  It runs in the directory a `-C`
  option selects and receives the global options as Git would pass them:
  `-c` pairs and `--config-env` pairs as `GIT_CONFIG_COUNT` variables
  after any pairs already set and after the pair the wrapper adds for its own lock ownership,
  so a caller's setting of the same key still wins,
  `--git-dir`,
  `--work-tree` and `--namespace` as their variables.
  A `--config-env` whose variable is unset,
  or that names none,
  is a command line Git refuses,
  so the push never runs and the upload does not start:
  an object sent now would travel to whichever server the setting that could not be read does not name.
  A signal sent to the
  wrapper is relayed to it.
  The child's standard error goes to a private anonymous file;
  the wrapper reads it back after each run and relays it
  with the secrets of every URL replaced by `<credentials>` (standard output stays
  the caller's).
  What counts as a secret is one rule for every line the wrapper prints or journals,
  in `src/native/lfs/lfs_output_mask.rs`:
  the userinfo of the URL,
  which ends at the last `@` of its authority;
  every path segment that is not a plain label,
  a label being the letters `a` to `z` and the separators `-`,
  `.`,
  `_` and `~` alone,
  which is what `lfs`,
  `objects`,
  `batch` and `info` are,
  so a capital,
  a digit,
  a percent encoding or another punctuation mark is hidden;
  and every value of its query and of its fragment,
  with a key kept only while it is a plain label,
  as the incident's `lfs.<url>.locksverify` key name was not.
  The punctuation that ends a URL in a sentence,
  a comma or a closing bracket,
  is printed after the mask,
  so a URL written in parentheses keeps its bracket.
  The owner's rule,
  quoted:
  "Never print or commit secrets".
  A token made only of lowercase letters reads as a label and is not hidden;
  whether the whole path of an LFS URL should be hidden instead is an open question for the owner.
  When that file cannot be made,
  the push is refused with the ways forward.
  A run that fails is never run again by the wrapper,
  whatever the cause:
  the step ends at once with the program's status and the explanation.
  An HTTP 5xx answer of the LFS server is classed as outside our control,
  a server fault like "Network down"
  (pick 12 of `doc/planning/cli-git-shipped-limits-audit.md`,
  "LFS server-error retry",
  the coordinator's under the owner's delegation and open to the owner's veto),
  and is retried by the commit service's push retry plan alone.
  A 5xx answer is a line of `git-lfs` with its `Fatal error: ` prefix,
  which `lfshttp/errors.go` gives only to 5xx answers other than 501,
  507 and 509;
  the text after the prefix is the server's own JSON `message`,
  or `Server error: <url>`,
  or `Server error <url> from HTTP 5xx`.
  The plan tries again every push whose failure it cannot read as final,
  and nothing in the words of a 5xx is final
  (`push_retry.rs`,
  `final_failure`;
  `an_lfs_server_error_is_tried_again_by_the_plan`).
  The earlier in-place retries,
  after 2 seconds and after 5 seconds,
  and the variable `CLI_GIT_LFS_RETRY_DELAYS_MILLISECONDS` that replaced them in tests,
  are gone,
  so there is one retry mechanism.
  A typed `git push` has no plan:
  it ends at the first 5xx with `git-lfs`'s own words and the explanation,
  and the person pushes again.
- What is uploaded.
  The destinations and refs come from the push's own arguments,
  read with the same option table as the rest of the
  wrapper,
  and from the configuration Git would read for what the arguments leave out.
  The configuration entries
  (`branch.<name>.pushRemote` and `.remote`,
  `remote.pushDefault`,
  `remote.<name>.push`,
  `.pushurl`,
  `.mirror` and
  `.url`,
  `push.default`) come from the snapshot taken when the invocation was planned,
  so a typed push costs no Git
  start for them;
  the auto-push reads them with one `config --get-regexp`.
  The first positional is the repository whatever `--repo` says;
  `--repo` and its abbreviations fill in only when none is
  typed.
  Each refspec names its source:
  the part before the last colon (Git splits there,
  so a source such as
  `:/message` survives);
  a deletion names nothing,
  `tag <name>` is `refs/tags/<name>`,
  and the refspec `:` (also
  `+:` and a configured `push = :`) means every local branch.
  A source with a `*` is matched as a refspec matches (the star crosses `/`) against the refs one `for-each-ref` lists.
  `--all` and `--branches` upload the local branches,
  `--tags` the local tags,
  never every ref;
  `--mirror` and
  `remote.<name>.mirror` upload everything.
  A bare push (or a remote with no refspec) resolves its remote the way Git does (`branch.<name>.pushRemote`,
  `remote.pushDefault`,
  `branch.<name>.remote`,
  then `origin`) and uploads what `remote.<name>.push` names,
  else every
  local branch with `push.default=matching`,
  else the current branch from the full ref `HEAD` names (`HEAD` when it is
  not a branch).
  A remote with more than one `pushurl` is uploaded to once per URL,
  passing each URL,
  because each may keep its
  objects in its own place.
  A destination typed as a path (starting with `/`,
  `./` or `../`,
  or naming an existing
  file or directory that is not a configured remote) is passed as a percent-encoded `file://` URL,
  which `git-lfs`
  accepts where it refuses a bare path.
  A push that only deletes uploads nothing.
  The plan fails closed:
  a Git query it needs that does not finish refuses the
  push with an explanation,
  it never reads as "nothing to upload".
  A push with `--recurse-submodules=on-demand` or `only` uploads only this repository's refs and prints a notice that
  a submodule's LFS objects must be uploaded from the submodule,
  because Git pushes the submodule itself.
- The explanation.
  A push of a repository with LFS keys runs as a child of the wrapper,
  in every case (a push that is neither stackable
  nor eligible for integration included),
  so a parent exists to explain.
  Before anything starts,
  one stat per `PATH` directory finds `git-lfs`;
  when it is absent the wrapper prints the
  install-or-remove explanation and exits with status 1.
  If the upload exits non-zero or is ended by a signal,
  the wrapper prints,
  after the program's own output,
  an
  explanation that names the status and every way forward (fix the cause and push again,
  `git lfs push --all <remote>`
  by hand,
  `git push --no-verify`,
  install Git LFS,
  or remove the LFS configuration),
  does not start the push,
  and
  exits with the upload's status (1 when it had none).
  The auto-push runs the upload as a bounded child with captured output and appends the explanation to the recorded
  output of the failed attempt,
  so a commit that joined that push shows it too;
  a stop by deadline says so.
- Paths.
  A typed `git push` is preceded by the upload,
  on the exec path and on the path that integrates a lost race;
  the merge push the wrapper runs for a lost race is planned and run inside the same process (`plan_invocation` with
  the stacking marker set in the planner's environment),
  is preceded by its own upload (its refspec source is the merge
  object name) and the wrapper does not start itself.
  The auto-push after a landed commit is preceded by it,
  and so is each reconcile push of that auto-push.
  The scope probe of a manual push and the integration step's probe are dry-run pushes that must not upload:
  they never run the upload.
  No `git monochromatic-git-policy-cli` subcommand pushes.
- Bypass.
  `git push --no-verify` (any accepted abbreviation),
  `--dry-run`,
  `GIT_LFS_SKIP_PUSH` set to a true value,
  and
  `-c hook.cli-git-lfs-pre-push.enabled=false` among the global options or in `GIT_CONFIG_COUNT` pairs (the switch an
  earlier version used for its hook) skip the upload and the pre-check.
  A repository or account configuration file
  is not read for that switch,
  since reading it would cost a Git start for every push;
  `GIT_LFS_SKIP_PUSH` accepts
  `1`,
  `t`,
  `true`,
  `y`,
  `yes` and `on`.
- Shims.
  `git lfs filter-process` and `git lfs clean` write the four shim files into the hooks directory whenever they run
  (`setupRepository` then `installHooks(false)`),
  so deleting the files does not last once an LFS path is cleaned by
  a Git that the wrapper did not start.
  A shim never runs through the wrapper (see "Real Git starts with traditional hooks off"),
  so there is no second
  upload;
  `git monochromatic-git-policy-cli doctor` explains the shims apart from the unsupported hooks (see "Doctor").

Verified by `src/native/binary/binary_lfs_pre_push_tests.rs`
(a stand-in `git-lfs` that records the arguments,
the input and the origin's branch at the moment it ran)
and `src/native/binary/binary_lfs_server_tests.rs`
(the real `git-lfs` against an LFS server written inline in the test).

### Candidate content

A policy reads a candidate's bytes from the object the index names,
never from the worktree,
so an edit after staging cannot change what it checks.
One object is read whole only up to 32 MiB (`CONTENT_CEILING`,
with its measurement beside it in
`candidate/candidate_batch.rs`):
above that the reply is consumed through a 64 KiB buffer and thrown away,
the object is never held,
and the reader stays in step for the next request.
The check then reports `policy-incomplete`,
an engine failure naming the file,
the object and both sizes,
and the pass goes on:
such a finding never blocks the command,
so a large asset costs a skipped check that names itself,
never a commit that fails.
The ceiling is 1.57 times the largest file this repository tracks (20,453,084 bytes),
so no content committed here is skipped.
It bounds the bytes a check reads whole,
never a reply's type field:
a `HEAD` commit larger than
the ceiling still names its commit,
so a staged listing compares against the real `HEAD` and not the empty tree.

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
and the fix then fails with "the index changed while monochromatic-git-policy-cli fix ran" after restoring every file it replaced,
so a concurrent landing can never be mistaken for a fix that touched the index.
Run the fix again.
Emit final findings and one fix summary to stdout.
A patch targeting an unselected tracked file adds it under the "Added paths" precondition;
its original worktree bytes for the concurrent-change check are the verified `HEAD` blob.

#### What an install keeps

Each corrected file is written into a journal inside the Git directory and then swapped with the
file through `renameat2(RENAME_EXCHANGE)`,
so the corrected bytes go in while the file they displaced stays
readable:

- the corrected bytes are created with the destination's own permission bits,
  owner and extended attributes,
  so a file kept at `0600` stays private,
  the set-user-id,
  set-group-id and sticky bits survive,
  and the access-control list that decides who may read it is the one the person
  wrote;
  the owner is
  given before the bits,
  because changing a file's owner clears those bits even when the owner
  and group stay the same,
  and the attributes are given before the bits,
  because setting an access-control list sets the group's bits from its mask;
  an attribute this process may not set refuses the whole fix and names the
  attribute;
  an owner this process may not give is refused rather than handed over,
  and the comparison is against what the
  replacement actually carries,
  read from its own descriptor,
  never against the process's credentials:
  a directory with its set-group-ID bit gives every new file that directory's group,
  so a
  `0640` file would otherwise change which group may read it;
- the bytes the swap displaced are read after the swap,
  when nothing can overwrite them any more,
  and compared with what the fix read,
  permission bits and owner included;
  a difference swaps them
  back and fails the fix,
  so a write or a permission change that arrived while the fix ran is kept
  rather than replaced;
- a selected path that is not a regular file - a symbolic link,
  a directory,
  a device - is refused
  instead of replaced,
  and a path that is gone is refused rather than created;
- rollback swaps first and then judges what the swap took out of the file,
  which is now at a name this
  command owns:
  bytes,
  permission bits,
  owner,
  extended attributes and file kind,
  never through a symbolic link.
  Only bytes that are
  this fix's replacement let the journal go;
  anything else - a save that landed beside the swap,
  a file the
  person narrowed,
  a link they put at the path - stays in the journal and is named,
  and the file is left
  holding the bytes the fix displaced;
- a site whose displaced file no longer holds what the fix read is read once more before its
  journal is removed,
  so a write that arrived through an already-open descriptor is kept and named,
  and a journal the fix
  cannot remove is named rather than reported as a success that says nothing.

A replacement carries every extended attribute the file lists,
as the bytes it holds,
the access-control list included,
and an attribute this process may not read or may not set refuses the whole fix
with the attribute's name,
leaving the original file in place:
that data exists in no commit,
so a replacement that dropped it would delete its only copy.
An attribute the replacement carries and the file never had is taken off for the
other half of the same rule:
the corrected file would otherwise grant an account access the original never gave,
and comparing the displaced file cannot see it,
because the unexpected attribute belongs to the replacement.
A security label is the one exception on both sides:
the kernel and its policy assign it,
a person does not write it,
and a container without the label's module answers "operation not supported" for
every one of them,
so a label the replacement cannot be given or cannot be rid of is a warning that
names it,
and the correction goes ahead.
The labels are `security.selinux`,
`security.apparmor` and Smack's `security.smack64` names;
`security.capability` and `security.ima` decide what the file may do or are a
signer's data,
so they are carried or refused like any other attribute,
and a substituted label is never counted as a difference when a journal's bytes
are judged.
A namespace the system does not let this process list,
such as another user's `trusted.*`,
is invisible to the fix and is neither carried nor counted.
A journal record written before the attributes were recorded names none,
so the sweep keeps and names such a journal whose bytes carry one,
rather than removing a copy it cannot prove is only a correction.

What a replacement does not carry is its inode number and birth time,
and its share of a hard link,
whose other name keeps the bytes it had.

The window that remains is two file system calls wide,
not the length of a pass:
a descriptor write that arrives after the last reading of the displaced file and
before its journal is removed is lost,
which every atomic replacement has,
and a write to the path after the swap
replaces the corrected bytes,
which is the newest state the person asked for.
When the first is hit the person
loses that one write and nothing of the file's earlier state;
when a write is caught before that reading,
it is on
disk in the journal the message names.
A rollback has no such window of its own:
it decides nothing by reading the destination.

##### What the person sees when that window is hit

The window needs a second process writing inside a few instructions.
The comparison of the displaced file and the removal of its journal are two system calls,
and a write has to land between them through a descriptor opened before the exchange,
which is what an editor holds when it saves a buffer for a file it opened before the fix ran.
The person then sees this:

- their editor still shows what they typed,
  because the buffer is the editor's and not the file's;
- the fix ends with no notice for that file,
  and says nothing about a journal,
  because the comparison it made succeeded and the journal was removed with the write inside it;
- the file on disk holds the corrected bytes,
  or,
  after a rollback,
  the bytes the fix found,
  so a look at it shows anything but their save;
- nothing of the file's earlier state is lost,
  and no journal is left behind,
  so there is nothing on disk to recover the write from.

A caught write is the recognisable case:
it is named with the journal that holds it,
and this one is not named at all,
which is how a person tells the two apart.
The only copy of the lost text is then the editor's buffer,
and the fix can be run again for the file.
No mechanism removes this window:
closing it needs the comparison and the removal to be one operation over a file another process may hold open,
which no file system offers.

Git loses that same write on its own file replacement,
so this is not a cost the wrapper adds.
Read in `git/git` at `master` on 2026-11-09:
`entry.c`'s `checkout_entry_ca()` takes
`changed = ie_match_stat(...)` of the file it would write,
and with `state->force` set - which
`git checkout -- <path>` and `git checkout <commit> -- <path>` use to discard local changes - it
runs `unlink(path.buf)` under the comment "We unlink the old file,
to get the new one with the right
permissions (including umask,
...)";
`write_entry()` then opens the name with `create_file()`,
which is
`open(path, O_WRONLY | O_CREAT | O_EXCL, mode)`.
A writer holding the old inode therefore writes
into an unlinked file there too,
and loses that write when its descriptor closes.
The path that takes
an explicit output name (`to_tempfile`,
as `git checkout-index --temp` uses) writes `mkstemp` and
renames,
which orphans the old inode in the same way.

#### The journal, and what a later fix does with it

One journal directory per corrected file,
`<git dir>/cli-git-direct-fix-<suffix>/`,
`0700`,
holding the corrected bytes (`content`) and
a record of the file (`record`:
its pathname,
permission bits,
owner,
the extended attributes it carries,
the writer's owner token,
and the
digest of the corrected bytes).
The record is written before the file is touched.
The Git directory is the
worktree's own,
from `known_git_dir`,
never the parent of an alternate index a caller named:
`GIT_INDEX_FILE`
can point inside the worktree,
and its parent is then a directory a later `git add --all` stages.
A fix that
cannot learn that directory refuses with `NO_GIT_DIR_MESSAGE` instead of guessing,
and a worktree whose
Git directory is on another file system is refused before anything is written,
because the swap needs
both names on one file system.
Nothing of a fix lives in the worktree,
so no `gitignore` rule can
expose a leftover and no `git add` can stage one.

Every settled `fix` sweeps the journals of that Git directory,
before it decides whether anything
needs correcting,
and acts only on a proof that the writer is gone
(the crate's existing owner proof).
The
sweep never exchanges two files:
a recovery that swapped would need a proof that the exchange it is undoing
happened,
and the destination's bytes are not that proof,
a reading of the destination is not a licence
to swap,
and two recoveries can exchange one journal twice.
So the one removal is the one the record
proves,
by the same judgement the rollback makes of a file it took out of a swap:
`content` is a regular file,
read without following a symbolic link,
whose bytes hash to the recorded replacement and whose permission bits,
owner and extended attributes are the ones the record names.
A digest alone would take a file the person saved with the correction's own bytes,
or the same
bytes narrowed to `0600`,
for the fix's own and discard what they chose.
A name that is not a regular
file is never opened at all:
the swap moves whatever stood at the destination into the journal,
and opening a
FIFO that no process writes to waits for ever,
so the type is read first - here,
in `read_record`,
and in the
judgement every other read of this route goes through.
Everything else stays and is named on every later
`fix`:
the person's bytes wait in the named journal until they act on it,
and no command decides for them
which of two versions belongs at the path.

What the sweep found reaches the person on every path out of a settled fix,
not only on the one that
installs:
a fix whose installation fails,
and one that cannot find the worktree or the Git directory,
report the
notices beside their own failure,
because the path in a notice is the only place the person finds the edit an
interrupted fix displaced.

No lock stands in this route:
the lock inventory guard covers `candidate/`,
`policy/`,
`markdown/`,
`standards_review/` and `commit/`,
and a lock there must be listed with its reason beside it.

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
status hints off,
ignored files kept,
then the one force an interactive clean runs with when its menu would offer a nested repository.

Without `--cli-git-keep-going`,
the first unpatched error stops the pass.
With the wrapper flag,
later policies run after findings,
but any final error still blocks real Git.
Engine failures always stop immediately.
The wrapper recognizes its flag only before Git's `--` separator and removes it before forwarding.

### Bulk staging

`add-explicit` refuses a `git add` whose pathspec selects every path it could reach,
however that pathspec is spelled,
and the finding says how many paths it selected.

`-A`,
`--all`,
`-u`
and `--update` are decided from the arguments alone,
with the option spellings Git accepts.
They stage every changed path whatever the tree holds,
so they are refused without asking the repository anything,
and the finding says whether every changed path or every changed tracked path is swept up.

A positional pathspec is judged by what Git says it selects.
An ordinary named path,
meaning one with no pathspec magic,
no glob character and no `.` or `..` segment
(`file.ts`,
`dir/`,
`.hidden`),
stages what its name holds:
it is let through and starts no process.
Every other spelling is measured,
with two listings after the caller's own global options:

- `ls-files -z --cached --others --exclude-standard` with the caller's own pathspecs,
  in the caller's own pathspec mode,
  so `--literal-pathspecs` and `GIT_LITERAL_PATHSPECS` answer for that caller.
  `--cached` and `--others` together are the paths `git add` can stage,
  and `--exclude-standard` leaves out the ignored ones it would skip.
- when that listing holds at least one path,
  `--no-literal-pathspecs ls-files -z --cached --others --exclude-standard --full-name -- :/`
  is every path the worktree gives.
  The wrapper wrote that pathspec,
  so no caller mode may read it as a name,
  and the paths under the caller's own directory prefix are the set that directory gives.

`require-root` refuses a command run below the worktree's top level before this check,
so the directory set decides only where that policy is off,
or where `-C` names a directory below the top with the command itself still at the root.
It keeps the guard as strict as it was for such callers,
who had `.` and `./` refused before.

A selection is always a subset of those paths,
so equal counts are equal sets,
and the policy refuses a pathspec when its selection is the repository's whole set
or the set of the directory the command runs in,
and only then when the spelling names no character of a name:
a pattern of wildcards alone (`*`,
`**`,
`:(glob)**`),
a spelling that resolves to a directory (`sub/..`,
`.//`,
`././`),
an exclusion (`:!x`,
`:^x`,
`:(exclude)x`,
`:(attr:!nope)`),
or a filter such as case-insensitive matching with no pattern that names anything.
A spelling that names a character of a name is let through even where the tree happens to fit it,
because `*.txt` in a repository whose every path is a `.txt` file stages by name and not by reach.
Several pathspecs of one command are measured together,
and only the reach-only ones are named.

The finding names every refused token as the caller wrote it,
says what it selects
(`It selects all 5 paths of this repository.`,
`It selects all 2 paths of this directory.`),
and offers `--no-enforce-bulk-add` for one invocation.
A flag alone says the set it reaches instead,
which needs no measurement:
`It selects every changed path in this repository.` or
`It selects every changed tracked path in this repository.`

Under `--literal-pathspecs` or `GIT_LITERAL_PATHSPECS=1` the pattern of a spelling is a name,
so only a spelling that resolves to a directory is reach-only:
`.`,
`./`
and `sub/..` are refused there,
while `**` stages the one file of that name.
The mode is the last of `--literal-pathspecs` and `--no-literal-pathspecs` among the caller's global options,
and otherwise the variable's own boolean reading;
a value Git would refuse to read as a boolean is no mode.

A listing that cannot start,
fails or reaches the read ceiling never blocks the command:
the add runs unchanged,
and one line on standard error says the check could not be made with the reason Git gave.
The read is bounded by `READ_CEILING` in `repo_read/git_metadata.rs`,
which is `PLUMBING_STEP_CEILING` in `service/wait_ceilings.rs`,
two minutes,
and the person is never held longer.
A spelling Git itself cannot prepare,
such as `:(top)/*` (Git 2.56.0 fails in its own `prep_exclude`),
gets that line and then Git's own refusal,
unless an enabled content policy reads the command's candidates first:
the candidate prediction runs that same `git add` on a private index and stops on its failure,
which names Git's refusal and no line of this check.

The two listings run before Git does,
so a path that appears between them and the forwarded command is not counted:
a spelling that covered every path up to then is still refused,
and one that covered all but that new path is let through.
The window is the policy pass's own,
and no file is read twice.

A pattern written only in wildcard syntax,
such as a class holding `*` or an escaped `*`,
is read as naming no character of a name,
because Git's character classes and escapes are not modelled here:
a repository whose entire content is one file of that name is refused,
and nothing else is,
since such a spelling names no path a person means.

An ordinary name costs nothing;
a spelling that is no ordinary name costs two listing reads,
the second only when the first found a path.
Measured on 2026-11-10 in a worktree of 22,993 paths,
five samples of each:
the selection listing took 8.9 to 9.7 ms and the worktree listing 46.8 to 49.2 ms,
so the guard adds under 60 ms to a `git add` whose pathspec is no ordinary name,
and the whole worktree listing is the larger part.

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

### Notes a check leaves

A check answers with findings,
and sometimes with notes beside them
(`PolicyOutcome::FindingsWithNotes`,
`policy_engine.rs`).
A note names one piece of work the check left alone and why,
carries the file in its own `path` field,
and never stops the command:
the engine writes it as an engine-failure event with code `policy-incomplete` and a chosen hint saying the command
is not stopped and nothing was discarded (`LEFT_ALONE_HINT`),
whatever severity the policy holds,
and the pass goes on.
A policy uses a note for work it must not do rather than for work it could not do:
a commit that may not overwrite a dependent's manifest keeps its landing and names the file,
which is what the owner's "commits should never fail" and "never discard user data" ask of it together.
The notes of the stable pass are the ones emitted,
so a check that proposes a correction and is asked again reports each left-alone file once.

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
A commit invocation can emit up to three batches (the traditional-hooks and external-hooks warnings,
the policy run and the
post-commit policy run),
each starting at `0`.

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
when an event has no `pathBytes`,
its `path` is the exact name,
with one exception.
A finding of `security/forbidden-strings` reports the scanner's masked display of the name in `path`
(a byte that is not UTF-8 shown as `\xNN`,
and a matched segment masked),
and never has `pathBytes`,
because the exact bytes would undo the masking.
Every other event that names a file reports the exact name this way,
on `git add`,
on a commit (landed,
blocked,
normalized,
replayed or recovered),
and on direct `check` and `fix`.
A `message` may also name a file;
there the name is readable text with each invalid sequence replaced by U+FFFD,
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
configuration warning on standard error,
before anything else the command prints.
It names the configured value in `path`,
says that traditional hooks are unsupported and that their behavior is undefined,
and says to run `git monochromatic-git-policy-cli doctor`.
The warning never blocks the commit and is not a core finding.
The trigger is the configuration snapshot the commit reads once for its other needs:
the commit starts no process and makes no file system call for this warning,
so a repository without the setting pays nothing,
and the hooks directory is not scanned on the commit path.
The line is delivered on a thread of its own and awaited for at most the `WARNING_WRITE_CEILING` of
`service/wait_ceilings.rs`,
so a standard error whose reader has stopped draining cannot hold the commit:
a line still waiting at that ceiling is abandoned and logged.
It is written to the standard error descriptor itself and not through the standard library's shared handle,
whose lock every synchronous write to standard error takes,
so a worker left at the ceiling holds no lock the commit's own writes would wait for.
The notices a preparation prints (an obligation of the worktree that cannot be read,
and the entries a landing left out of this commit)
are given to the same descriptor under the same ceiling,
and one that cannot be said in time is left unsaid rather than holding the commit.
The wrapper's own per-transaction `core.hooksPath` overrides (`<tx>/hooks` and `<tx>/no-hooks`),
which reach commands that config hooks start,
never trigger it.
The value it names is the last entry of the key that is not one of those overrides:
an entry a
wrapper-started Git put into a child as command-scope configuration is last in that child,
and the warning is
about the path the person configured,
not about the one the wrapper injected.
An override is recognized only as a `command` scope value
(the only place the wrapper's overrides come from)
that names `<git-common-dir>/<data folder>/shadow/<transaction>/no-hooks`,
where the data folder is `monochromatic-git-policy-cli` or its earlier spelling:
a path that merely ends the same way belongs to whoever made it and still warns,
and so does such a path written into a configuration file.

### Core finding event

```ts
export type CoreFindingEvent = EventBase & {
  readonly type: 'core-finding';
  readonly trigger: 'pre-forward';
  readonly coreId:
    | 'commit-only'
    | 'commit-normalization'
    | 'concurrent-commit'
    | 'explicit-path-commit';
  readonly code:
    | 'commit-only/all-flag'
    | 'commit-only/pathspec-required'
    | 'commit-only/staged-changes-ignored'
    | 'commit-only/staged-deletion-worktree-present'
    | 'commit-normalization/no-change'
    | 'concurrent-commit/replay-conflict'
    | 'concurrent-commit/already-committed'
    | 'explicit-path-commit/named-path-ignored'
    | 'explicit-path-commit/pathspec-matches-nothing';
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
- `commit-only/staged-deletion-worktree-present`:
  an explicit-path commit names a path whose deletion the real index stages while an entry still exists at that
  worktree path,
  so the pathspec cannot record the deletion.
  The paths it names are the selection's own,
  read from a pathspec file exactly as from the command line,
  and the question is asked of Git under the pathspec modes the caller set,
  so `--literal-pathspecs commit -- 'literal*.txt'` is asked about the file of that name and never about a glob,
  and a selection of nothing but subtracting elements keeps native Git's reading of it.
  One Git start carries at most 64 KiB of elements and every subtracting element rides in each of them,
  so a pathspec file past one command line is answered in bounded starts
  (`commit_staged_deletions.rs`,
  `selection_elements`).
  The message names the paths and both ways forward
  (`commit_staged_deletions.rs`).
- `explicit-path-commit/named-path-ignored`:
  an explicit-path commit names one file of the worktree that an ignore rule matches while no index entry tracks it,
  so the private staging would refuse it.
  It is decided from the selection's own elements before any `git add` runs,
  and the index it asks about is the real one,
  read in process:
  a commit whose named paths the index all holds asks Git nothing.
  The commit is not made,
  nothing is staged by it,
  and the message names every rejected path with both ways forward:
  leave the file out of the commit's paths,
  or stage it with `git add --force -- <path>` when it is meant to become tracked
  (`commit_named_ignored_files.rs`;
  issue 6278).
- `explicit-path-commit/pathspec-matches-nothing`:
  a pathspec the command line names,
  or one Git read from the pathspec file the command line named,
  matches no path Git can stage,
  so the private staging's own `git add` refused it.
  Native Git keeps deciding which pathspecs are valid,
  so this classification reports its refusal instead of predicting a match:
  a directory,
  a pattern,
  a magic selector and a literal name behave exactly as they did before.
  The commit is not made,
  nothing is staged by it,
  and the message names every refused token as Git named it;
  a token that begins with a hyphen also says that options belong before `--`
  (`git commit --message <text> -- <paths>`).
  The staging start whose words this reads runs under the C locale,
  so the finding appears whatever language the person's shell asks for,
  and a token that itself holds Git's closing wording is named whole
  (`commit_pathspec_matching_nothing.rs`,
  `private_index_request.rs`).
  Every other failure of that staging start keeps its engine-failure report
  (issue 6283).
  The twenty commits that stopped inside that staging on 2026-11-10,
  before this finding existed,
  are measured in `doc/planning/cli-git-commits-that-stopped-after-their-add-steps.md`.

Every `concurrent-commit` finding leaves ref,
real index,
and worktree bytes unchanged by monochromatic-git-policy-cli.

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

This event is emitted when a commit exists but a post-commit policy or engine failure makes monochromatic-git-policy-cli return `2` after
real Git succeeded.
It follows any causal finding or engine-failure event and is the final policy event.
An ordinary auto-push network or remote rejection is not an engine failure:
monochromatic-git-policy-cli surfaces complete push output,
leaves the commit local,
and preserves the successful commit command's exit code `0`.

### Word merged event

```ts
export type WordMergedEvent = EventBase & {
  readonly type: 'word-merged';
  readonly outcome: 'word-merged';
  readonly message: string;
};
```

This event is emitted when a commit replayed over a branch that moved had files merged by word where Git's line merge
reported a conflict.
`message` holds one line per merged part,
separated by newlines,
each naming the file and the part.
It does not block the commit and does not change the exit code.

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

The design,
its measurements and the decisions behind it are in `doc/planning/cli-git-resident-service.md`.
This section states what a commit does.

### Scope and phases

Several `git commit` invocations against the same worktree and branch run at the same time,
and each lands as its own sequential commit.
The behavior is on in every repository the wrapper runs in;
no configuration key or environment variable opts out of it.

Every non-dry-run commit command line is landed by the resident commit service of the worktree:
one instance of the systemd template units `monochromatic-git-policy-cli@.socket` and `monochromatic-git-policy-cli@.service` per worktree,
started by the operating system on the first connection (socket activation).
The wrapper never starts the service itself,
and it has no other way to land a commit.
The service unit sets no resource limits of its own
(no `MemoryMax`,
`CPUQuota`,
`TasksMax` or `IOWeight`),
so the service and the Git processes it starts get systemd's defaults,
the same resources plain Git gets.
The limits it once carried had no measurement behind them,
and the 1 GiB memory cap made the service's Git read the same pack and index pages from disk again
(on the main checkout 274,736 cap hits and 16,125,924 major page faults by 2026-11-03T08:40:51-05:00).
Owner,
2026-11-03 (question tool):
"Remove all three (Recommended)" for the CPU,
task and disk limits,
after asking why the service was capped "to the magic number of 1GB of memory w/o evidence".
A service that finds the worktree it serves gone stops its own socket and serves nothing,
and it asks when it starts and once a minute while it serves:
the administrative directory is missing and the repository that held it is proven gone
(the nearest directory above it that exists is the root of a mounted filesystem or holds other entries;
a repository on a drive that is not mounted,
whose mount point holds nothing,
proves nothing),
and the note provisioning wrote beside the socket,
or the `gitdir=` records of the service's own journal,
names that directory and hashes to the instance.
It asks this before the after-commit work counts itself busy,
so a service that keeps a push waiting for a new try is asked too,
and the pushes it kept for the gone repository end with it:
one journal line names the branches no try will reach again,
while a plan of a repository that is still there stays.
It stops its socket unit with the job mode that leaves the units requiring it alone,
because its own end is what stops its service unit,
and a stop of that unit from inside the service would signal it before it could stop its own socket and leave its files as the sweep finds them.
It never removes its own journal,
because it is still one of that journal's writers:
its after-commit task writes the note of a repository to look at and the plan of a push to try again,
on a thread of its own and from jobs of the blocking pool,
so a write that lands between the listing that says the journal holds nothing unresolved and the removal of the whole tree
would be taken with it.
It keeps the journal,
the note and the socket file instead,
names the journal in its own log when it holds work a person may want,
and leaves the removal to the sweep that asks the manager whether its service is gone
(`service stop-units-of-removed-worktrees`),
which finds the instance by the note it kept.
It ends with status 0,
not a refusal,
because `Restart=on-failure` would start a refused unit again,
and a socket that could not be stopped is left to `service stop-units-of-removed-worktrees` and to `doctor`'s count.
The sweep removes the instance's service journal with the socket and the note when it holds nothing else:
the records of the batches that landed,
once no client of the instance can still read them,
and an empty landing record are the memory of a service that is never served again,
while a batch that did not land,
a batch whose objects a scratch reference may hold,
a record that was written and not published,
a landing or withdrawal record,
the repositories a service was still to look at and the pushes it was still to try again,
and a file or folder this release does not know keep the whole journal and are logged,
never discarded.
A finished batch's record is kept for `A_FINISHED_RECORD_IS_READ` in `service/wait_ceilings.rs`
(`ANSWER_CEILING` plus `SHORT_EXTRA_WAIT` plus `LOOKING_FOR_A_LANDED_COMMIT`:
the whole wait a client may spend on an answer,
the short extra wait inside it,
and the look for its commit),
because a client whose answer was lost reads that record to learn which commit its request became,
and sends the same request again to be told it already landed.
That age is read from the clock reading the record carries,
written with the record itself (`service/service_journal_record_age.rs`):
the boot and machine IDs,
the monotonic clock and the wall clock,
the reading a claim's lease is judged by (`service/claim_time.rs`),
so two records of one boot are compared on a clock that never steps and does not count a suspended host's sleep,
a record of an earlier boot of the same machine is past at once because a restart ended every client of it,
and only a record of an earlier release,
which carries no reading,
is judged by its modification time.
A wall clock that stepped forward is therefore no reason to take a record a client of this boot may still read.
A journal kept only for those clients,
or for a service the manager does not yet report gone,
keeps the note and the socket file instead of being logged:
nothing in it is a person's business yet,
and the next sweep removes it when both are over,
so until that sweep runs `doctor` counts the instance and names the command that stops it.
A stop of a removed worktree's units answers when the manager queued the job,
so the sweep and the removal ask for the service unit's state and wait `SERVICE_GONE_AFTER_A_STOP`
(two seconds) for the manager to report no process,
and a unit that outlasts that keeps the journal and the files that name the instance for the next sweep.
Nothing starts a unit.
The wrapper forwards native `git commit` only for dry runs and short-circuit forms.

The repository is the one the command line names.
`-C` moves the wrapper,
and `--git-dir`,
`--work-tree` and `--bare` (or `GIT_DIR` and `GIT_WORK_TREE`) select the repository,
resolved from the directory the `-C` chain reaches,
as Git resolves them
(`src/native/invocation/repository_selection.rs`).
Every Git of the commit keeps that selection:
the capture's own reports,
each private-index start,
and the landing,
which names the Git directory and work tree the request or the journal record holds.
A commit therefore lands in the repository the command named,
and the repository its process directory belongs to gains no object,
no scratch directory and no commit.
The wrapper's own Git starts run in the directory the `-C` chain reached,
so they receive the caller's other global options without that `-C`.

Each commit passes through three phases:

1.  Preparation:
    in the wrapper,
    in parallel with other commits and without the real index lock,
    capture the invocation facts,
    build a private index for the selection,
    refresh the stat data of that private index when it no longer describes the worktree,
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
    The service reads the branch,
    makes the commits' objects and writes the batch's intent to the journal first,
    none of which needs the index,
    then creates Git's own `index.lock` exclusively,
    advances the target ref by compare-and-swap from the tip it read,
    builds the new index beside the real one,
    records the worktree's stat data in it
    (`git update-index --refresh` with `GIT_INDEX_FILE` set to the copy,
    asked for only when the copy no longer describes the worktree,
    and left out for the entries the reset just wrote),
    and installs it by renaming the lock over `index`.
    The copy carries the access and modification times the live index had,
    before and after each step that writes it,
    because Git re-reads the stat data of an entry only while that entry is as new as the index file
    (`is_racy_stat`):
    a stat check compares the fields it reads to the second,
    so an edit made in the same second as the stat the live index records differs from it in nothing Git reads,
    and a copy written a moment ago would make Git answer from that recorded stat,
    hiding the edit from `git status` and the next commit.
    The commit's private copies keep the same times for the same reason (issue #544).
    The lock is therefore held for the branch move and the index build only
    (`lock_hold_ms` in the service's log line for the request),
    and a foreign holder is waited for after the objects and the intent exist,
    not before them.
    Measured (btrfs,
    host,
    release build,
    the repository's own `native:perf` task,
    three alternating runs of each arm,
    180 landings each,
    service log of the request):
    the hold had a median of 24 ms and a 95th percentile of 68 ms when the lock was taken first
    (the sum of every phase after the wait),
    and a median of 5 ms and a 95th percentile of 6 ms as `lock_hold_ms` when it is taken after the intent.
    The median paired delta of the commit scenario against native Git was 39.5 to 40.2 ms for the unchanged release
    (six runs,
    a spread of 0.7 ms) and 38.9 to 39.8 ms for the new one (three runs,
    a spread of 0.9 ms);
    the commit path was not made slower.
    A foreign commit that moves the branch while the service waits for the lock makes the compare-and-swap fail,
    and the batch is built again on the new tip (the attempts a branch that moves under a batch already had).
    A foreign holder of `index.lock` is waited for on an event watch of the Git directory,
    with the holder's line repeated at the interval of `CLI_GIT_WAIT_NOTICE_SECONDS`.
    What a foreign lock's evidence proves rests on two facts and never on the wall clock (audit of 2026-11-09,
    finding 3):
    a process of the account that has the lock open,
    matched to it by the device number and the inode the lock has,
    and a Git lock PID file (`core.lockfilePid`) that names a running process whose command name
    starts with `git`,
    which is the holder because Git writes its own number there.
    A running program that is not named like Git may be that Git under another name,
    or a later program that took the number,
    and no start time tells them apart:
    a process's start is derived from the wall clock while the lock's change time is stored,
    so a step of the clock moves a live holder's derived start past the lock,
    and a step back moves a later program's before it.
    The start is read and worded beside the evidence,
    and it decides nothing;
    a number that is reused therefore reads as an owner nothing proves alive,
    which the ceiling ends,
    and a running Git is the holder whatever the clock says.
    A holder proven alive,
    an owner nothing proves alive,
    and a PID file naming a number this process table does not have are all waited for until
    `LANDING_LOCK_CEILING`,
    because such a number may be a live Git's of another PID namespace
    (cross-vendor review of 2026-11-09,
    finding 2);
    the wait ends the commit before the branch moved,
    removing the batch's intent,
    keeping the built commit on a side ref,
    and naming the blocker
    ("Every wait has a bound",
    in "Merges and commits never fail").
    The variable belongs in the environment of the unit that runs the service.
3.  After landing:
    in the wrapper,
    print the one summary line,
    install the corrections a policy made into the worktree,
    run the post-commit policies against the landed commit,
    and,
    when that gate passes,
    hand the push and the upkeep to the service as work items.
    A blocked gate keeps the commit local and exits `2`.

The landing holds the push work item of every commit a batch would land,
in the work queue's `held` state,
before the branch moves,
and recovery holds them for a batch it finds landed;
every path on which a batch does not land forgets them.
Nothing drains a held item,
so no round can publish a commit whose post-commit gate has not decided.
Nothing releases a held record by judging a process:
a push whose command died before its checks ran stays
held,
and the question of whether it should ever be pushed is open with the owner
(`doc/handover/questions-for-the-owner.md`).
A held commit is not stranded by that:
the queue pushes a branch,
`git push <remote> <branch>:<target>`,
so the next push from that branch carries it.
Every command in the worktree says so in one line,
read from the held records themselves:
which commit is held,
that its push has not been released yet,
and that it goes out with the next push from its branch.
The record is the only place the state exists,
so the line cannot disagree with it:
a kill between the record and a second file cannot lose the line,
a release cannot leave a stale one,
and two commands cannot interleave into either.
Measured on 2026-11-12 on this host,
one look at the held records of a worktree that holds none costs 27.8 microseconds
(2,000 looks through the command's notice desk,
`push_notices_for`),
which is nothing beside the owner's limit of five seconds per commit.
The gate itself promotes its own record into `pending` with one rename when it passes and unlinks it when it
blocks,
so a blocked gate keeps the commit local,
and a promotion creates every state directory a claim passes through (see "What a landed batch leaves
behind").
A blocked record is not deleted:
it is moved into the queue's `blocked` state with the branch and the gate's words,
and it is what makes the next push of that branch stop before that commit (see "A blocked commit is never
published").
A held record whose own words say its gate refused the commit is read as a blocked verdict as well,
because the verdict is written into that record before the record is promoted or forgotten:
a command killed in that gap,
and a verdict record that could not be written at all,
then still keep the commit local and still stop the branch's push.
A record an earlier release wrote holds the word `blocked` alone,
so its commit,
its branch and its reason are read from the file's name and from nothing else;
the commit is judged for every branch,
which can stop a push that verdict never concerned,
and no line names it,
because no branch can be named.

A record whose command still runs decides the other way:
that command's checks may refuse the commit,
so the automatic push stops before it until they answer
(see "A blocked commit is never published").
A command whose output a program reads settles the held records too and shows no line,
because the settlement is not a notice:
the approval a dead command recorded must not wait for a command a person types,
and a workflow of scripts types no other kind.
It takes no push-failure notice and no held line,
as it takes none of the other notices.
A held record is settled when Git proves its commit reached the remote.
The question is put to the local repository alone,
because a command asks no remote:
`<branch>@{upstream}` names the ref the branch's pushes update,
and `git merge-base --is-ancestor <commit> <that ref>` says whether the commit is in it
(`held_push_commit_on_the_remote.rs`).
A `merge-base` that exits other than `0` or `1`,
a branch with no upstream,
a branch outside `refs/heads` and a Git that cannot be started all answer "nothing is known",
and the line stands.
That record is moved to `done`,
where the queue keeps a push that was made,
so it is known to the queue and no round may run it,
and no command says the commit is held while `git status` shows it on the remote.
The proof is Git's record and not the remote's:
a remote-tracking ref can lag the remote
(a push whose reference update was refused still delivered the commits),
and a lagging record leaves the held line standing,
which costs a line that is still true of the record and loses nothing,
while settling on anything weaker would drop the only word about a commit that is not backed up.
A `blocked` record is not asked about at all:
the question of whether a commit the gate refused should ever be pushed is open with the owner.

A push record in `pending` is a push no service has taken yet.
When the wrapper is killed between promoting its record and waking the service,
or a service goes away before it looks,
the command that would have said "the push is queued" is gone,
so every command says it from the record:
the oldest such record is named,
the rest are counted,
and the words claim only that the push has not been made and that the service makes it the next time it
looks at the repository,
which needs no further commit.
A record younger than `PUSH_QUEUED_AFTER` (15 seconds) is a push in flight and is not named;
the wait measured on this host at 2026-11-09T16:40-05:00 from a landing to the claim of its push record
was 15 ms to 3.9 s with the service idle between commits and up to 10.7 s with commits made back to back
(debug build,
local bare remote),
so a record that outlasts the bound has outlasted any service that was merely busy.
The line is the record's,
so it goes when a claim moves the record.

The service writes the repository it lands for into the journal's claim watch before it answers
(`note_for_the_next_service` in `service_claim_watch.rs`),
so a service that starts looks at it even though no client ever woke it for that repository,
and a kill is followed by the work being run with no commit needed
(`service_server_work_loop.rs`).

No lock of the wrapper's own exists.
The only lock a commit meets is Git's `index.lock`,
and the only wait is the service's bounded wait for a foreign holder
and a forwarded index writer's event-driven wait for the service's own hold on the index.
A forwarded index writer that finds a foreign `index.lock` ends as native Git ends it.

### Locks and waits of the service, by item

Concurrency in the service is lock-free first:
atomic rename,
exclusive creation,
one owner per item.
Every lock that remains covers the smallest unit and has its reason written beside it in the code.
`src/native/source_guard/service_lock_inventory_tests.rs` fails when a lock primitive (a mutex,
a read-write lock,
a condition variable,
an `flock`) appears in the service and support sources outside its inventory,
or without the reason above its declaration.

Removed:

- The `flock` on the discarded copies' folder (`discarded_store.rs`):
  each file is owned by the pass whose `unlinkat` of it succeeded (see "Discarded copies").
- The mutex-guarded table of landed captures (`service_captures.rs`):
  the serve loop that lands commits and answers clients owns it,
  and passes it down.
- The mutex on the review credentials (`service_work.rs`):
  the work thread that remembers,
  reads and forgets them owned the map (commit `52f06fbe12`);
  since the work consumer drains
  repositories side by side,
  a round makes a read-only copy of the handlers that holds the round's credentials as values,
  the
  workers only read it,
  and it is dropped when the round ends,
  so the map has no owner to wait for and no lock.
- The mutexes and the condition variable of the claim renewal (`service_claim_renewal.rs`):
  see "Leases of claimed work".
- The long hold of Git's `index.lock`:
  the service asks for it after the objects and the intent exist,
  and holds it for the branch move and the index build only.

Kept,
each with its reason in the code and here:

- Git's own `index.lock`:
  Git's protocol for writing the index,
  which the service writes.
  It is held for the branch move and the index build (a median of 5 ms),
  and waited for on an event watch.
- The mutex on the pending bytes of the journal's stream (`journald_native.rs`):
  a stream may take a line in parts and two lines must never run together,
  so the unfinished rest of a line and the write that
  finishes it are one critical section of one non-blocking write;
  the waiter waits for one system call,
  never for the journal.
- The reclaim of a stale `index.lock`:
  one reclaimer at a time,
  by the claim chain of placed files and process-identity judgements on every file system
  (`service_index_claim.rs`,
  reason in `take_claim`);
  the earlier `flock` on the stale lock's own file is retired.
  The claim chain has no lock primitive for the scan above to find,
  so it is listed here and nowhere else (see "One reclaimer per stale lock").
- The claim and its lease on work items (`work_item.rs`,
  `service_claim_renewal.rs`):
  rename-based,
  no lock file;
  a lease is how a later instance learns that a killed holder abandoned a claim (see "Leases of claimed work").

Init-once cells (`OnceLock`) hold values read at first use (a boot ID,
the machine ID,
the Git release,
the process identity) and wait for nothing after.

### What a landed batch leaves behind

A batch's branch moves before its index is installed,
so the two can be apart after a kill,
a failing step,
or a restart that declines to replay the update.
Three things outlive the batch's landing for that reason.

The index update the landing still owes is recorded in the worktree's own Git directory,
at `<git dir>/monochromatic-git-policy-cli/work/owed/<batch>.owed`,
before anything tries to install the index.
The record names the index file,
the branch values,
the landed commit and the paths,
and it is a small record of its own so that it does not depend on the journal's intent,
which holds every field a batch needs and which no reader finishes a batch from when it cannot be parsed.
The record goes when the index update is installed,
and with a batch that recovery drops,
because a batch that never landed owes nothing.

An intent is written as a temporary file that is flushed,
renamed into place,
and followed by a flush of the journal's directory,
so a process killed between any two of those steps leaves either no intent or a whole one.
What makes an intent unreadable is therefore not a kill:
it is a release whose format this build does not read,
a write whose tail a file system lost after the flush that promised it,
or a person's edit.

Recovery never removes an open intent it cannot read.
It names it once per pass,
with its batch number and its path,
and keeps it:
the intent is the only record of a landed commit's branch values,
paths and request identities,
so removing it forgets the index update and lands a resent request twice.
A record that cannot be read is therefore a person's to remove,
or to open,
once what its batch owed has been settled from the record above.

Which entries are the unfinished landing's own cannot be decided from what they hold:
a person can stage the batch's old content back,
another branch can hold the same bytes,
a staged child of an added path leaves no entry at that path,
an unmerged path is represented by its stages,
and a program can stage between any comparison and any write.
A repair that wrote the live index could therefore discard staging that is the person's,
which is the worse outcome,
so no commit writes the live index for this reason.

What a commit does instead is record nothing about those paths.
For every path of a record whose entry in the commit's own index is still exactly the entry the landing
replaced,
and whose worktree does not hold that same content (a worktree that does is the person's own statement,
so it is committed),
and which the commit did not select by name,
that index is set to the worktree's head,
and the commit prints one line naming the paths and both ways on:
`git reset -- <paths>` to catch the index up with the branch,
or `git commit -- <paths>` to take the worktree's content for them.
A path with unmerged stages is left alone,
because a merge in progress is the person's own act,
and an explicit-path commit names its own selection and is never changed.
A path is left out only on evidence,
and absence is not evidence:
a worktree file that is not there with no entry in the copied index is what a landing whose index update
failed leaves,
so the landed entry is kept and the path is said.
A deletion the person staged in that window therefore waits for the next commit,
which records it once the index and the branch are read together.
A link is compared by what Git stores for a link,
the hash of its target text with mode `120000`,
never by hashing through it.
That comparison cannot tell a restoration the person staged from one they made without staging it:
a dangling link whose target text was put back leaves the same index entry either way,
and nothing is lost in either case,
because the entry the repair compares is the one the person's own index holds.
The repair leaves such a path out when the index still holds the text the landing replaced and the
worktree holds different text,
and the notice names it,
so the person's next step is the one the notice gives.
Two interleavings need a second process inside a few instructions and are not covered:
settling the last owed record between a commit's copy and its exclusion,
where the copy still holds a stale entry that no record names any more,
so nothing is compared,
nothing is named,
and the commit records the landed path as a deletion,
and a concurrent staging of the same content a `git commit --all` selected for a path a record names,
where that content is left out of that commit and named among the paths left out.
In both the next commit records what the index holds.
The comparison is made against the commit's own index,
because that snapshot is what the commit records:
a program that catches the live index up between the copy and the comparison leaves the snapshot's stale
entry,
and the commit still records nothing about it.
A commit never removes a record:
only the service settles one,
when the installation is recorded,
by a
rename to a settled name,
and recovery settles one it finds already installed,
so a kill between the
installation and the landing's own settling leaves either an owed record or none.
A record that matches
nothing costs a comparison.

The notice names two commands that work:
`git reset -- <paths>` to take the branch's content,
and
`git add -- <paths>` followed by a plain commit to take the worktree's.

A restart that finds the index file is not the one the landing saw finishes the batch,
leaves the index as its writers left it,
and keeps the record,
so the commit above is what protects the next command.

A landing that concludes a merge,
a cherry-pick or a revert keeps a copy of the conclusion state the commit removes
in its journal at `<batch>.cleanup`,
made beside that name and renamed onto it before the branch moves,
so a copy that failed half way never takes the name the cleanup reads.
The state files are removed only while they still hold the bytes copied at preparation,
and the copies the comparison reads are the client's own,
which the service removes when that client is gone.
With the scratch directory gone the comparison is made against the journal's copy,
so a cleanup that failed once is finished by the next landing and the worktree's next commit does not conclude
the same merge again.
A cleanup whose copy and whose scratch directory are both gone removes nothing and said so:
no copy of what it compares exists.

### Repositories of a round run side by side

The failure notice of a push is worded before its record is removed,
so a command killed while it names the commit leaves the only durable statement that the commit is not backed up for
the next command.
Two commands that start together can both show it,
which costs a repeat and loses nothing
(`src/native/push/push_failure_notice.rs`).
A push whose holder died on every try is told about before its record moves to `failed`,
because nothing scans `failed`:
a kill in between would abandon the branch in silence
(`requeue_expired_reporting_telling`).

A push record names its branch in text,
so a commit on a branch whose name is not text leaves none:
a spelling with replacement characters would push another branch.
The commit lands as any other,
and the command says once that the automatic push was not queued,
naming the branch in the `$'...'` form and saying that a push the person makes sends it
(`src/native/landing/service_handoff.rs`,
`src/native/service/service_server_land_group_owed_pushes.rs`).

A push record's id is its name and its idempotency key:
the kind,
the landed object and,
for a push,
a digest of the branch reference (`src/native/landing/work_item.rs`).
A commit can sit on two branches,
and each must reach its own remote,
so the commit alone does not name the push;
a record an earlier release named by the kind and the object alone keeps its name and is run as it is.
The other kinds keep the commit as their key,
because their work is of the commit and not of a branch.

The after-commit work of the service (pushes,
upkeep,
reviews,
trash) is drained one repository at a time in releases before this
one,
by the single work thread that commit `52f06fbe12` made the owner of the review credentials.
The repository owner's answer of 2026-11-03 to whether the service's independent work should run side by side was
"Parallelize,
no locks (Recommended)",
so a round of wakes now drains its repositories on workers within the host budget
(`src/native/landing/service_round.rs`):
the threads the process may run less a margin,
and the memory free in its control group.
The commit service's unit sets no CPU,
memory,
task or disk limit (the owner's decisions,
`doc/handover/cli-git-rust-implementation.md`,
lines 148 to 160),
so the width
of a round is the host budget:
an eighth of the threads left free,
and half of the memory available over `REPOSITORY_MEMORY`,
128 MiB,
which is the 90th percentile of the peaks of 28 services rounded up to a power of two
(`doc/planning/cli-git-shipped-limits-audit.md`,
"Measurement record").
The retry list is reported in the order the directories were named,
whichever drain finishes first.
The directories of one repository share a lane,
so a main checkout and its linked worktrees have their work directories drained one after another there,
because those directories share one upkeep and one object store.
The items inside one work directory run beside each other,
and which of them may is the next section
(`src/native/landing/service_work_drain_pending_claims.rs`,
`src/native/landing/service_work_items_side_by_side.rs`).
Nothing is shared mutably:
the handlers are read-only,
every claim is still made by an atomic rename,
each drain keeps its own
renewal of its own claims,
and `service_lock_inventory_tests.rs` still forbids a lock in these sources.

#### What each kind of work runs beside, and what it must not

One work directory holds the records of every worktree of its repository,
and a queue of them is what a person's push must never wait for:
on 2026-11-11 the installed `0.6.1-alpha.10` held two reviews and four trashes claimed from 10:24
while a fresh commit's push sat pending from 10:29,
and `main` was pushed by hand.
So every kind names the thing two items of it may stand in each other's way over,
and the drain runs at most one item of each such thing at a time,
with different things side by side.
The two keys are the worktree an item names (its `repository`) and the branch a push names.
Nothing else is shared:
the handlers are read-only values,
and each item's claim stays in the drain's own set and is settled by the task that took it.

The width of a wave of slow work is the least of the items waiting,
the host's threads less an eighth,
the free memory over one repository's measured peak,
and `SLOW_ITEMS_CEILING`
(`service_work_items_side_by_side.rs`):
4,
the number of blocking places the service has (`SERVICE_WORKER_CAP`),
because beyond them a wider wave only adds tasks that wait for a place.
Measured on 2026-11-11 on this host under its own load
(average 68 to 100),
the same binary asked for each width,
alternating,
over eight worktrees' trashes of one entry of 8,000 files each:
1,599 and 1,389 ms at one item at once,
9,935 and 948 ms at two,
1,129 and 1,090 ms at four,
1,916 and 853 ms at eight,
so the spread of one width covered the differences between widths and no width was faster than another;
the removals run at I/O priority class `idle`,
and the drive was the limit.
What the width buys is in the controls and not in that wall time:
two worktrees' trashes are inside their handler at one moment after this change,
which the release before it never did.
A push's width is the host's threads and memory alone (`pushes_side_by_side`),
because a push waits on the network and holds no place of the blocking pool.

##### Push

A push sends one branch and updates that branch's remote-tracking reference,
so two pushes of one branch must not run beside each other:
the same branch's records are one group and reach one push.
Two branches of one repository do run beside each other:
they share the object store,
which only grows,
and each names its own branch and its own reference.
A push runs beside every slow kind,
which is what the two claim lists are for;
the round that answers a wake claims `PUSH_KINDS` alone (`landing/service_work_slow_lane.rs`).
The push's own steps never queue behind slow work either:
one place of the process's bound on blocking jobs stays free of every slow kind
(`SLOW_WORK_RESERVE`,
`run_a_slow_job_within_the_budget` in `blocking_jobs_within_the_budget.rs`),
so a queue of trashes cannot hold the read of a push's child,
which is the shape of the wait measured on the installed `0.6.1-alpha.10` at 2026-11-11T10:38:00-05:00.

##### Upkeep (`maintenance`)

`git maintenance run --auto` takes the repository's own packs and locks over the common object store,
so one repository has one upkeep at a time;
the drain already runs one upkeep per repository,
keyed by the repository.
Upkeeps of different repositories run beside each other.

##### Prewarm

Filling a new worktree's Cargo build directories writes inside that worktree
(the trees it clones and the Cargo children it starts),
so it must not run beside the relink of the same worktree,
nor beside that worktree's trash emptying,
its snapshot ahead,
or the delete of its subvolume.
The prewarm of another worktree runs beside it.

##### Relink

Rewriting the copied package state of a new worktree runs each tool's install inside that worktree,
so it too holds its worktree alone.
The relink of another worktree runs beside it.

##### Trash

Emptying a trash removes the entries the leave-out moved and then removes the trash's parent directory
when nothing else is in it (`worktree_trash_dispose.rs`),
so two trashes of one worktree would race over that parent:
one worktree empties one trash at a time.
Two worktrees' trashes share nothing but the disk,
each is proved to be the directory its own snapshot made,
and each removes its own parent,
so they run beside each other.
Every entry removal and both of the emptying's lookups ask through the gate that
keeps a blocking place free of slow work (`run_a_slow_job_within_the_budget`),
so a queue of trashes never stands in front of the work a person waits for,
and the removals run at I/O priority class `idle`.

##### Subvolume delete

Destroying a removed worktree's subvolume is one kernel call on a directory nothing else names,
but the record names the pool it is removed from as its worktree,
so the deletes of one pool run one at a time and different pools' run beside each other.

##### Snapshot ahead

A snapshot ahead copies one worktree's tree,
which an add is waiting for,
and it runs before the upkeep and before every kind that frees space in the same round.
It holds its worktree alone:
a trash emptied,
a prewarm filling or a relink writing inside that tree would be captured halfway.
Its record names the source worktree,
so that is the key it takes.

##### Review

The shadow review of a landed commit reads the commit and appends one entry to the repository's one log,
and the append is one whole-entry write on an append-mode descriptor with no lock
(`log_lines_appended_without_a_lock.rs`),
so two reviews of one repository may run beside each other and every other kind.
Reviews go last in a round because a push is the person's backup and a review is a shadow record.

##### What the waves keep of the order

Between the kinds the order of a round is unchanged,
because each step of it answers a priority and no conflict:
the snapshot a person waits for,
then the upkeep,
then what a new worktree needs (relink,
then prewarm),
then what frees space (subvolume delete,
then trash),
then the reviews.
Inside one kind,
each worktree's records keep the order they were claimed in,
and a worktree that has more records than the width waits for the next wave rather than for the whole queue:
the turn moves by one key each wave.

### The service's hold on the index lock

`src/native/landing/service_index_lock.rs` places `index.lock` with the service's marker in the file from the first instant.
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
place a new one) are not one atomic step.
The removal of a stale lock is kept to one reclaimer per stale file by the claim chain beside the lock
(`service_index_reclaim.rs`,
`service_index_claim.rs`;
below,
"One reclaimer per stale lock");
placing the new lock is by exclusive creation.
The rest is safe because at most one process that writes this marker takes a given `index.lock` at any moment.
Three things enforce that:

1.  Inside one service,
    `acquire` (a landing,
    `land_batch_announcing`),
    and `try_acquire` and `remove_stale` (the recovery of open batches,
    `recover_selected`),
    are called only from the serve loop and,
    before it starts,
    from `service/service_main.rs`.
    Connection threads only queue jobs,
    and the work thread (push and upkeep) never touches an index.
    The calls are sequential on that one thread.
2.  Across processes of one worktree,
    the worktree has one systemd unit,
    `monochromatic-git-policy-cli@<instance>.service`,
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
The tests that start the service mode directly start one service for each Git directory
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
  stop `monochromatic-git-policy-cli@*` before committing with the older release.
- A user who points two worktrees at one index with `GIT_INDEX_FILE` makes two instances take one lock.
  That is not a path the service ships;
  their reclaims of a stale lock of it are kept apart by the claim chain below,
  and
  their placements by exclusive creation.

#### One reclaimer per stale lock

Every path that removes a stale `index.lock` of the service (`remove_stale` and `try_acquire` in the recovery of open
batches,
and the wait of a landing) goes through `reclaim_if`,
which opens the lock,
reads the marker from that open file,
takes its
place in the claim chain beside the lock (below),
checks that the name still refers to that file (device and inode),
and only then
unlinks the name.
Nothing is moved aside,
so a live lock is never off its name.
The chain is used on every file system and the `flock` on the stale lock's file is not taken any more (owner,
2026-11-04:
"Claim files
always"):
a reclaimer on the `flock` path and one on the claim path of one file system would not see each other,
and a lock manager
can answer one call and refuse the next.
A placement of a claim that is interrupted or finds no memory (`EINTR`,
`ENOMEM`,
`EWOULDBLOCK`)
is a back-off;
any other refusal of the placement is Unguarded.
There are eight answers,
and each has its own consequence:

- Removed:
  the stale file was removed,
  or was already gone;
  the placement is tried again.
- Changed:
  the name refers to another file than the one judged (the stale lock was reclaimed and a live one placed);
  nothing is
  removed,
  and the placement is tried again.
- Busy:
  another reclaimer holds a claim on the stale file (or a placement was interrupted);
  a landing waits as for any held lock and looks
  again after the poll pause,
  because the end of a hold leaves no event in the directory,
  and the ceiling ends the wait;
  a
  restart leaves the batch open.
- Unguarded:
  no claim file can be placed beside the lock (the file system refuses the placement,
  or this process cannot name
  itself in a claim).
  Nothing is removed,
  because an unlink without exclusion could remove a live lock.
  A landing waits as for any
  held lock to its ceiling and withdraws the commit to its side ref;
  its notice is written from the marker of the file that was judged
  (never a second read of the name,
  which a live landing may hold by then),
  names the lock,
  the batch and the dead process,
  and says to
  remove the lock by hand once no git command and no commit of the service is running and the file's first line still reads as
  quoted.
  A restart leaves the batch open.
- Foreign:
  the file (the lock,
  or a claim in the way) is not proven gone on this machine:
  its marker names a machine ID that is not
  this reader's (or this reader has none),
  or names none and a boot other than this one while this reader has a machine ID,
  or names no
  process at all,
  or the lock is in the format of an earlier release (see "The marker format and earlier releases" below).
  It stays.
  A landing
  waits as for any held lock to its ceiling and withdraws the commit;
  its notice,
  which is also written to the log on every path
  (the restart's too),
  names the file,
  the batch and the process,
  says why it is not removed,
  and gives `rm -- '<path>'` (the path quoted
  for a shell) for a person who knows that service is not running.
  A restart leaves the batch open.
- Unjudged:
  a claim in the way cannot be judged:
  its maker is in another PID namespace or this reader's `/proc` does not agree,
  or the
  claim cannot be read.
  It is neither passed over nor removed,
  and it is not passing contention (Busy is a claimer proven to run):
  the
  notice names the claim and gives the `rm -- '<path>'`;
  a landing waits as for any held lock to its ceiling,
  and a restart leaves the
  batch open.
- Kept:
  the file is not a stale lock of the service (a live holder's,
  Git's,
  or unreadable);
  it is left where it is.
- Stuck:
  the file is stale and cannot be removed;
  the log says why,
  and the landing fails as before.

#### The claim chain

Used by every reclaim,
on every file system,
and it needs no `flock` anywhere.
A claim is a file named `cli-reclaim-<digest>-<slot>` in the lock's directory,
where the digest is the first sixteen bytes of the SHA-256 of
the lock's file name and a key made from the marker of the stale lock (its batch,
process and birth identity),
so that every reclaimer
reaching the file by any mount derives one name,
whatever the device number of that mount,
and the name has a fixed length however long the
lock's own name is.
The process that marker names is gone and places no further lock with that marker,
so a claim left under a key is
inert.
A marker that names no process is never judged stale (below),
so no claim is ever made for one.
A claim is mode 0644 whatever the umask,
so that a reclaimer of another account can read it (one that cannot read a claim takes its
maker to run).
It holds the service marker of its maker with a random token in the batch field,
is written to a staging file and placed
whole by a rename that does not replace (else a hard link),
and is judged as a lock holder is (`holder_state_in`):
a maker that is gone
is passed over;
a maker that is proven to run makes the reclaimer back off (`Busy`);
a maker on another machine makes it leave the claim and
answer Foreign;
one that cannot be judged,
or a claim that cannot be read,
makes it answer Unjudged.
A placement that answers with an error
although it took effect (a network file system can lose the answer) is recognised by the claim's text and is the reclaimer's own;
when
the claim cannot be read back at that moment,
the thread remembers the claim it may have made and its next attempt removes it if it is
there with exactly that text (per thread,
so another thread's attempt in the same process is never taken for it).
The reclaimer places its claim at the first slot not taken by a running claimer,
then reads every slot below its own again:
all
must still exist and name a gone maker,
or it removes its own claim and backs off.
It then runs the same name check and unlink as
and removes the claims from zero to its own once the name no longer refers to the stale file;
when the unlink
fails it removes only its own.
A claim of a dead maker is never removed while the stale file stands at its name,
so a slot read as gone stays gone and in
place,
and no reclaimer can be placed below it;
two holders would need one slot to be read as gone by the other,
which its own
claim or the exclusive placement prevents.
The rule needs no birth time of the file,
because a claim left by an earlier file with
the same key is a gone claim at a low slot and is passed over.
A file at a claim's name that holds no whole marker is passed
over too,
since a placed claim is whole.
Limits:
a claimer killed after the unlink and before its cleanup leaves inert `<lock>.cli-reclaim-*` files,
which the next reclaim of
a file with that key removes and which a person may remove with `rm -- "$(git rev-parse --absolute-git-dir)"/cli-reclaim-*`
once no commit of the service runs (an alternate index:
the directory of that index;
`rm -- "$(git rev-parse --absolute-git-dir)"/cli-reclaim-*`);
a claim cannot be placed on a full disk,
where the landing's own lock cannot be either;
and two machines with the same machine ID (cloned images) are judged as one machine.

#### The marker format and earlier releases

A marker of this release reads `cli-git-service v2 <batch> <process> <identity>`.
An earlier release's reader takes the word after the
prefix for a batch number,
finds `v2`,
and reads no marker in the file:
it leaves the lock as a lock it does not know,
so no earlier
release's reclaimer (which holds a `flock` on the file) ever acts on a lock of this release.
In the other direction this release does not
remove a stale lock in the earlier format (no `v2`),
because that release's reclaimers and this one's claims cannot see each other and both
could unlink the file:
such a lock is left,
named,
with the `rm` (answer Foreign),
and the first line the notice quotes is the file's own.
A marker that names no process is never read as gone (it could be a live service on another machine),
and neither is one that names no
machine ID and another boot when this reader has a machine ID:
it cannot be told from a service on another machine.
After an upgrade,
a
stale lock an older release's killed service left is therefore removed by hand once;
every lock this release writes is reclaimed on its own.
After a downgrade,
the locks of this release are left to the person by the older release (stop `monochromatic-git-policy-cli@*` first,
as for any downgrade).

The exclusion is the one that `doc/planning/cli-git-one-reclaimer-per-index.md` argues for:
the unit is the one file being
reclaimed,
the hold lasts a few system calls,
and a dead claimer's claim is passed over,
so no leftover can block a reclaim.
It holds no lock primitive the inventory scan looks for (`source_guard/service_lock_inventory_tests.rs`),
and its reason stands beside
`take_claim`.
Two reclaimers,
and a reclaimer against a landing,
are kept apart;
Git takes no part and never reclaims.
What stays open:
a person's `rm index.lock` between the check of the name and the unlink,
when Git then places its own lock in
the two system calls between,
because no system call unlinks a name only if it still names a given file;
a file system that
takes no claim file (the stale lock is then left for the person,
and the log says so);
and a service on another machine whose marker
carries no machine ID (an earlier release's),
which is read as before:
another boot ID reads as gone.
Release 0.5.28's reclaim could leave a file at a taken-aside name (`<lock>.aside-<process>-<time>`) beside the lock,
when a third
program took the name while it was off it.
This release makes none,
and nothing removes the ones already there:
a scan of the
visible processes does not prove that no process holds one,
and the file may hold a native Git's pending index.
Once no git
command and no commit of this service is running,
list them with `ls -l -- "$(git rev-parse --absolute-git-dir)"/index.lock.aside-*`
(an alternate index:
the directory of that index),
keep any whose first line is not `cli-git-service `,
and remove the rest with
`rm` (`doc/planning/cli-git-one-reclaimer-per-index.md`,
"Aside files an earlier release left").

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
A filesystem that offers neither (a FUSE server such as `s3fs-fuse`,
or a vfat mount on a kernel without the rename flag,
for example) gets the lock the way Git takes its own:
`open` with `O_CREAT | O_EXCL` creates `index.lock`
(Git 2.56.0,
`tempfile.c` line 142,
called from `lock_file` in `lockfile.c` lines 168 to 190),
the marker is written into that file and flushed,
and the holder keeps that file's descriptor,
so the ownership check at release compares the lock's own inode.
The fallback is taken only when the rename that does not replace and the hard link were both refused
(`EINVAL`,
`ENOSYS`,
`EOPNOTSUPP` for the rename;
`EPERM`,
`EOPNOTSUPP` for the link),
never when the lock exists or storage failed,
and never as the first choice.
Reason:
"Commits should never fail."
and the owner's pick "Keep the fallback (Recommended)" for such filesystems;
real Git commits there and the fallback is never less safe than plain Git on the same filesystem.
A marker that cannot be written or flushed removes the lock the attempt created and fails the attempt with the storage error.
The service's log says once per acquisition that the lock was taken this way and why.
What a kill can leave there:
the lock created and the marker not yet written leaves an empty lock,
as plain Git leaves one
(`unable_to_lock_message`,
`lockfile.c` lines 254 to 267,
tells the person to remove it).
It may be a Git that is mid-write,
so nothing here removes it:
a wrapped `git add` meets it as Git does,
a landing waits for it until `LANDING_LOCK_CEILING` and ends with `index_in_use`,
and the words at the ceiling name the file and the command that clears it.
A kill after the marker is written leaves the whole marker of a gone process,
which the next landing removes where a claim file can be placed (any filesystem that offers a rename that does not replace or a hard link);
on a filesystem that offers neither no claim can be placed,
the lock stays (`Unguarded`),
and the notice names the file,
the batch and the command that removes it.
Self-clearing of an empty lock is not shipped.
The one proof considered was a file lock held on the lock file (`flock`),
and it cannot close the gap:
Git never takes a file lock on `index.lock`,
and the lock file must exist before anything can lock it,
so a kill between the creation and the file lock leaves a file that is indistinguishable from a live Git's empty lock.
The claim chain cannot be used for it either,
because a claim must be placed whole and that is the call this filesystem refuses.
The claim chain stays on the whole-file placement alone (`place_marked`);
only the index lock has the fallback (`place_lock`).
The kill controls are `binary/binary_fault_trace_lock_without_links_tests.rs`
(kills at `lock-created-unmarked`,
`lock-marked-unsynced` and each batch point under `strace --inject` of `EOPNOTSUPP` on `link`,
`linkat` and `renameat2`);
the kill controls recover on a working filesystem after the kill (the injection ends with the faulted service),
so the `Unguarded` case above is covered by the unit control `a_stale_marked_lock_stays_where_no_claim_can_be_placed` and not end to end;
a commit through a new service during the kill control also proves the service removes no lock without its marker;
the end-to-end control with real Git as the control is `a_filesystem_with_neither_no_replace_rename_nor_links_does_not_stop_a_commit`.
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
then the PID namespace (the number in `/proc/self/ns/pid`),
then `#` and the machine ID when the host has one (the text of `/etc/machine-id`,
or of `/var/lib/dbus/machine-id` when the first holds none,
32 lowercase hexadecimal digits that are not all zero;
the same reading as for claims of work,
`claim_time.rs`).
A reader of an earlier release sees a namespace it does not know in such a marker and answers unknown,
which keeps the lock.
A marker an earlier release wrote has no boot ID,
no namespace or no machine ID and is judged by the parts it has.
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
  the boot is not the current one,
  on the same machine (every process died with that boot,
  whatever the reader's numbering;
  a marker that names a machine ID is read so only when the reader's machine ID is the same),
  or no process runs under that number,
  or the process under it was born at another tick
  (a marker that names no holder is not gone:
  it is unknown;
  and a marker that names no machine ID is read by its boot only by a reader
  that has none either).
- Running:
  the same boot,
  the same namespace,
  and the process there is the one born then.
- Unknown:
  the marker names no process (an earlier release's),
  it names no machine ID and a boot other than this one while the reader has a machine ID,
  the marker names a machine ID that is not the reader's,
  is not valid,
  or the reader has none (a boot ID that differs there says
  nothing about the service,
  which may run on its machine;
  the lock or claim stays,
  a landing waits to its ceiling and the notice
  gives the `rm`),
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

The markers of the next batches are made ahead.
A batch that takes the index lock for batch `n` starts the markers of batches `n + 1` and `n + 2`,
which land on the same index:
it creates each file and writes the marker there,
and the flush is the service's background task's,
while that landing moves the branch and builds the index.
Started there rather than after the landing ended,
a flush overlaps a landing's own work instead of being the next commit's wait,
and the second marker is the spare that leaves one finished while another is being made.
The next commit then only places the file for its own batch (a rename that does not replace),
so the flush is not on its path.
Starting them there matters because the flush is seconds long on a loaded disk:
a measured landing waited 9 to 13 seconds for a marker whose flush was still running
(`lock_stage_kind=pre`,
2026-11-10),
against the owner's rule that a commit through the wrapper is not five seconds slower than native Git.
A landing that finds a marker still being made never waits for it:
it stages its own marker the ordinary way,
as one does when nothing was made ahead,
and a marker that will not be used is let go rather than waited for:
one whose file exists has it removed at once,
and one whose file does not exist yet makes none,
so no thread on a commit's path waits for a flush.
Two files at most are the service's:
they are kept for the service thread,
named with this process and the batch number,
and a marker is replaced or removed without waiting for its making,
when its batch is passed or the index is not the one it was made for.
They are removed when the service ends,
whichever way its loop ends:
a marker's file is created and recorded as one step that the service's last sweep excludes itself from,
so a marker is either recorded before that sweep and removed by it,
or never made.
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
A commit that arrives while the flush is still running starts no second flush of its own and waits for none.
The file the first commit of a service needs is made by that commit,
because the service learns the index only from a request.
`lock_stage_ms` is what a commit waited for in this step,
measured over the whole of it:
near zero when a marker made ahead was finished for it,
the removals of the markers it cannot use,
the whole creation when it staged its own,
and never the making of a marker another batch will place,
nor the flush of a marker,
which the background task makes;
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

The built index is written without a Git process (`index_reset_file.rs`,
called from `build_index` in `service_land_follow.rs`;
the ported rules are in `doc/planning/cli-git-main-commit-within-native-plus-five.md`,
"The landing's index update and the
index check in process").
It runs only for the landing that holds the live index lock,
and the type system says so:
`reset_index_file` and `build_index`
take a borrow of the held `IndexLock`,
so only the holder can call them,
and the in-process update takes a `PrivateCopy`,
a directory this landing created for itself beside the live index (`service_land_copy.rs`),
owner-only from its creation,
named
`cli-git-private-` and a random identifier,
and holding the copy,
a marker that names the owning process,
and whatever Git or
the in-process update makes for the copy (Git's lock for it,
the temporary file of the write).
No name beside the live index is
shared,
so the copy is never the live index and never another client's file,
a foreign lock of the same name does not stop a
landing whose live index is absent,
the name does not grow with the live index's,
and a pile of abandoned copies cannot use up
names.
The copy is created owner-only before its first byte,
filled from the live index while the lock is held,
and given the
live index's own mode afterwards (the mode Git gave that file,
which an index installed without a change keeps).
The directory
goes with the value;
a landing sweeps the directories of a service that died,
recognised by the name,
the marker and the death
of the process the marker names,
never by the name alone.
The lock's release renames the copy over the live index
(`IndexLock::release_with_record`),
after syncing the index's directory.
The only caller is `follow_up` in `service_land.rs`,
which owns the lock;
it is reached from the landing with the lock that
`acquire_noting_wait` gave,
and from the restart with the lock that `try_acquire` gave.
So a foreign `git add` finds the lock taken from the copy to the installation,
and what it staged before is in the copy.
The installed index is recorded in the journal (`<batch>.indexed`,
holding the commit,
synced) after the rename and before the
lock is removed,
and a record that cannot be made (or an installation whose directory cannot be synced,
or a panic that unwinds
through the record) keeps the lock for the restart to reclaim.
The finished batch is made durable (its journal directory synced)
before its completion record is removed.
A completion record that exists and cannot be read stops the restart,
which leaves the
batch open and the index alone,
because only a missing record means the update was not installed.
A restart that finds the record does not replay the index update,
which would overwrite what another program staged after the
lock was given up;
it retries only the conclusion state,
so a cleanup that failed never makes the index update replayable.
The checked-out branch is read from the Git directory of the worktree the request recorded (not from the directory the
index
sits in),
again once the lock is held and before the ref moves (a worktree can switch while the lock is awaited),
and once more
after the ref moved,
because Git refuses `symref-verify HEAD` beside an update of the branch `HEAD` names ("multiple
updates
for 'HEAD'"),
so the move cannot verify it in its own transaction;
a worktree that left the branch in that window keeps its own
index.
A restart that finds the worktree on another branch finishes the batch without touching the worktree's own
index,
still
finishes an index kept in another file (`GIT_INDEX_FILE`),
and keeps the batch open until the conclusion state is removed.
The landing records the entries of its index paths under the lock before the branch moves (`<batch>.start`,
synced),
and a
restart that has to replay an update that was never installed resets only the paths still at those entries;
a path staged since
keeps its staging and a one-line notice says so.
A batch with no record (a commit Git built,
an older batch,
an index form the
entries are not read from) replays every path as before;
a record that cannot be read stops the replay.
The result goes to a temporary file made exclusively beside the copy,
given Git's index file mode before any byte is written
(created as Git creates its lock file,
so the mask and any default ACL of the directory apply,
then `adjust_shared_perm`'s
calculation from the mode the file really has),
flushed to the disk,
and renamed over the copy;
an interrupted write is
tried again,
and a run removes only the file it made.
An entry another session staged in the same second as the copy is racily clean to Git,
so the update leaves that landing to Git.
At most one service removes the stale lock of a physical index at a time,
whatever the number of worktrees that share it
(a shared alternate index included):
the reclaim takes its place in the claim chain beside the stale lock and checks that the name
still refers to it before it unlinks (`reclaim_if`;
"One reclaimer per stale lock").
A reclaimer that finds a running claimer ahead
backs off,
and a person's `rm index.lock` in the two system calls between the check and the unlink stays open.
Whatever the update does not reproduce (the list is in the planning section) is left to `git reset --no-refresh` on the same
private file,
so Git's `core.fsync` governs only that fallback;
the in-process write always flushes the file,
as Git's default does.
A `reset` the service starts (that fallback and the reconciliation's own index update) carries `--no-literal-pathspecs`,
because every spec it is given is `:(top,literal)<path>`,
and `reset` accepts a spec that matches nothing without a word:
under a global literal pathspec mode,
inherited from the command that first started the service,
it would install no entry and report success.
The entries the update makes from the tree record the worktree's stat data where the file provably holds the tree's bytes
(`index_reset_worktree_stat.rs`),
and the all-zero value `git reset --no-refresh` writes everywhere else.
A path's file proves it when all of these hold:

- It is a regular file whose every directory on the way is a real one on the worktree's device,
  reached below a root held open without following a link.
  A symbolic link is left to Git:
  it is rare,
  and a link read across a swap of its parent directory cannot be proven to be the one examined.
- Its raw bytes hash to the entry's object name read as a blob.
- No attribute Git applies converts these bytes,
  and the bytes hold no carriage return.
  A clean filter,
  `ident` or a `working-tree-encoding` can leave raw bytes equal to the blob while Git's view of the file is another blob,
  and a line-ending rule may strip a carriage return on the way into the index.
  The attributes are read in process,
  from the same files Git would read (`ContentJudge::conversion_is_the_identity`),
  and a fact that cannot be established refuses the record.
- The state of the one inode read is the same before,
  during and after the read and the path still names it,
  and the file is older than the index file being written,
  compared to the nanosecond as Git's `is_racy_stat` compares them.

An entry that is as new as the index is racily clean to Git,
so it keeps the zero value and both Git and the wrapper's own staleness question hash it;
a path of another kind,
a file above `CONTENT_LIMIT`,
a file that changed while it was read,
and a path no file holds keep the zero value too.
The proof never waits:
 the open does not block (`O_NONBLOCK`),
so a fifo renamed over the path is refused by its type instead of holding the landing's index lock for a writer.
One landing reads at most `CONTENT_LIMIT` bytes in all,
and every byte read is charged,
including a read the stability checks later refuse,
so a commit that lands a whole worktree never reads it again inside the landing's index lock.
The attribute facts cost one `git var GIT_ATTR_SYSTEM` the first time a given real Git has no cache entry for them,
and nothing after that;
the check itself took 0.094 to 0.104 ms for a landing of 200 paths in five directories,
and the whole proof 2.9 to 3.1 ms,
on the release build of 2026-11-09.
The attribute files the proof reads are charged to the same bound,
each at most `ATTRIBUTES_LIMIT` and never more than the room that is left,
so a path whose attributes do not fit keeps the zero stat data and costs a refresh.
A directory whose `.gitattributes` the index holds while the worktree's copy is missing refuses the record:
Git reads the index's copy there (`attr.c`,
measured on Git 2.56.0),
and a sparse checkout leaves the file outside its cone the same way.
A read has no time bound,
which is native Git's own behaviour for the same files under `index.lock`;
what bounds the person is the client's answer ceiling,
after which it withdraws the request and the commit is reported as not landed.
A read the system interrupts (`EINTR`) is tried again,
because an interruption says nothing about the file,
and the tries are counted (`INTERRUPTED_TRIES`,
one try and three retries,
the same count the attributes reader takes):
a file system that interrupts every read for ever ends in the same refusal a hard read error does,
so that path keeps the all-zero stat data,
the landing goes on,
and the next command that needs the file hashes it through Git.
Nothing in the landing waits for an interruption to stop.
The 4,000 newest non-merge commits of this repository change a median of 27 KB
(p90 209 KB,
p99 3.3 MB,
largest 23.1 MB,
`package/git-policy/cli/e2e/commit-shape-trace.json`),
so no commit there spends the bound,
and 32 MiB of SHA-1 costs 10 to 20 ms on this host.
The next commit's staleness question finds the recorded entries clean and starts no Git for them,
where the zero value cost one Git start and one hash of each file the previous commit landed.
That question reads the index's bytes and `lstat`s each entry in contiguous slices on several threads above
`STAT_WALK_BREAK_EVEN_ENTRIES` (`commit_index_stat_staleness.rs`),
with the worker count of `parallel_budget.rs`,
and it takes no lock:
it asks about the file the index names and never writes the index.
The rename that installs the copy has the durability native Git's has,
by default and when the user opts in.
The index can be rebuilt from `HEAD`,
and the service syncs the commits and refs.

The watch on the lock's directory ends when the directory itself is renamed or removed.
A rename of an ancestor directory is not seen:
the watch stays on the old tree and the wait notices at its next timer.
That is a bounded delay and not a hang;
the wait requires the Git directory's ancestry to stay in place while it lasts.

### Records of a worktree that is removed

A worktree's unshown push failures and unfinished pushes live in its own Git directory,
which Git deletes with the worktree,
so before `git worktree remove` and `git worktree prune` run,
the wrapper copies them into a snapshot under the common Git directory
(`src/native/worktree/worktree_removal_rescue.rs`),
and a later wrapper command settles each snapshot:
a snapshot whose Git directory is gone is promoted into notices any worktree shows,
one whose Git directory is still registered with an intact checkout and whose taker has ended is discarded,
and anything else is kept for a later command.
The snapshot names its worktree in a `gitdir` file,
by the bytes of that directory under `gitdirHex=`,
so a path that is not text still names the worktree it was taken for;
a snapshot that an earlier release completed with the path's plain text is still read,
and its text holds every byte even when its last one is a line break.

The registration Git writes beside a linked worktree is read as bytes too,
and only the one line break Git ends it with is removed,
so a checkout whose path holds a line break is looked for as the whole path.
A name that does not decode keeps the snapshot for the next command;
it never settles it on a guess.

The note that names an instance's worktree,
written beside the socket,
holds the directory the same way.

### Machines with no way to reach the service

A commit on a machine where no socket answers and no user service manager can be asked to start one
(no runtime directory,
no session bus,
no executable of the wrapper's own to name in a unit)
changes nothing and ends with exit `1` and a plain message for the platform:
Linux without user services names `loginctl enable-linger` and the requirement of a systemd user manager,
and Windows,
BSD and other systems say that committing through the wrapper is not supported there yet.
Run outside a repository,
a commit ends with Git's own words and exit status for that question.
A socket file nothing listens on and one that takes the connection and answers nothing are told apart,
because only the second means a service stopped inside its own process:
the message names the address and the seconds it waited
(`ClientFailure::NothingAnswered`,
and `ClientFailure::DidNotStart` where nothing listened at all).
There is no other writer to fall back on:
the service is the only writer,
so a commit with no way to reach it changes nothing and stops.

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
Every name is bytes:
the symbolic `HEAD` and its target are read from Git's answer unchanged,
so a branch whose name holds a byte that is not UTF-8 is captured,
recorded,
prepared and moved as that name,
and every message shows it in the `$'...'` form a shell reads back
(`src/native/foundation/ref_name.rs`).
The same holds for the arguments of the command:
a message,
a message file's name or a `--pathspec-from-file` name reaches real Git with the bytes the person typed
(`src/native/commit/commit_args.rs`,
`src/native/landing/service_prepare_changes_config_and_pathspec_steps.rs`).

The capture's own `rev-parse` report names the repository the caller selected
(`--git-dir`,
`--work-tree`,
`--bare`,
or the `GIT_DIR` and `GIT_WORK_TREE` variables),
and the in-process layout reader answers only a plain layout,
which is the one found from its own directory:
a command line that names a repository is answered by Git,
because a Git directory beside its work tree is not the one the reader would find.

### Private preparation

Preparation never mutates the real index,
the worktree,
the target ref,
any other real ref,
or shared reflogs.
It uses:

- a private commit index at
  `<common-dir>/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-<id>/commit.index`,
  in the commit's scratch directory,
  which the service removes when the command's connection closes,
  however the command ends,
  and which a command that never handed it to a service leaves where the sweep of the repository scratch directory
  removes it
  (see "The commit's scratch directory",
  below);
- the real object store.
  Every monochromatic-git-policy-cli Git command that writes or reads the private index's objects
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

The `git add` prediction copies the real index into
`<git-dir>/cli-git-add-policy-<pid>-<nanoseconds>-<count>`,
`0700`,
on the same file system as the real index,
with the real index's access and modification times.
The copy is built under a staging name,
`cli-git-add-policy-<suffix>.making`,
with its `owner` record already inside it - the writer's process id,
the platform's identity of its birth
and its PID namespace - and one `rename` publishes it under the name a sweep visits,
so a copy this
release publishes always names its writer.
The command removes the directory on every path it ends by;
every copy made later sweeps the directory it is created in and removes a copy only when its
record proves the owner gone
(a dead process id in this PID namespace,
a reused id kept,
an owner of another namespace alive until its record outlives the foreign-owner patience).
A copy with no record is kept,
whatever its name says,
because a process id means something only inside its own PID namespace:
another namespace's live command looks exactly like a dead one from here,
and removing its copy makes it fail while it copies or reads the index.
No lock protects the sweep:
a copy has one writer and only a proven-dead writer's copy is removed.

#### Maintenance that may run while a commit is made

The objects of a commit that has not landed are protected by Git's grace period,
and from the import on also by a reference.
What this protects against,
and what it does not (the supported concurrency,
written 2026-11-04):

- Supported while a commit is prepared and landed:
  any maintenance that keeps
  Git's default grace period for unreferenced
  objects,
  which are `git gc`,
  `git gc --auto`,
  `git maintenance run` (including the
  `maintenance run --auto` the landing starts
  after a landing) and `git prune` with the default `gc.pruneExpire`.
  They
  remove no unreferenced object younger than the
  grace period (two weeks by default).
  `git gc` repacks with `-A`,
  so unreachable
  objects of a pack that the import kept become
  loose objects first and are then subject to the same grace period.
- Not supported while a commit is prepared or landed:
  maintenance that drops unreferenced objects at once.
  That is
  `git prune --expire=now`,
  `git gc --prune=now`,
  any setting of
  `gc.pruneExpire=now`,
  and `git repack -a -d` (which discards the
  unreachable objects of the packs it replaces).
  Git's own manual says the same of
  `git gc` run while another process writes
  ("when git gc runs concurrently with another process,
  there is a risk of it
  deleting an object that the other process is using
  but hasn't created a reference to";
  the installed 2.56.0 manual,
  section NOTES of git-gc).
- What the wrapper does about the unsupported case is detect it where it can and
  say so,
  not prevent it.
  Before the landing
  imports anything or moves the branch,
  the objects the request names are looked up
  (a blob,
  a tree,
  the commit Git built,
  and
  for a commit Git built the objects it adds to the branch);
  a missing one answers that the content is unavailable,
  the
  wrapper prepares the commit once more,
  and a second loss ends the commit with a
  plain message.
  An object removed after that
  look and before the branch moves is not prevented.
  The reference
  `refs/worktree/cli-git-service/<batch>` that the import creates
  is a reachability root for any collector that reads references after it exists;
  a collector that took its snapshot of the
  references before it was created and sweeps afterwards is not stopped by it.
  The landing writes that reference itself for a commit Git built
  (`src/native/landing/side_reference_written_here.rs`):
  the name is the wrapper's own,
  a number below a directory the wrapper makes,
  never packed,
  never symbolic,
  and never given a reflog,
  because the write this replaces passes `core.logAllRefUpdates=false`,
  so the write is a directory,
  an exclusive lock file,
  the object name and a rename,
  with the mark `core.lockfilePid` asks for beside the lock,
  all in the worktree's own Git directory,
  where a per-worktree reference lives
  (measured on Git 2.56.0:
  in a linked worktree `update-ref` writes
  `.git/worktrees/<id>/refs/worktree/cli-git-service/<batch>`,
  and `git rev-parse --git-path` names the same file).
  Git writes it instead wherever its write would differ:
  a reference store `core.fsync` flushes,
  a `core.sharedRepository` repository,
  a `core.lockfilePid` value that is not a plain boolean,
  a storage that is not plain files,
  and a configuration the landing read with the caller's global options,
  which proves nothing about the store.
  A write this landing makes has no step to end and no ceiling to keep,
  and a reference it cannot write is Git's,
  worded by Git.

When real Git builds the commit,
it runs in the scratch repository after policy evaluation settles,
with inherited stdio,
as:

```text
git <caller global options, without a directory change or a repository selection> --git-dir=<shadow>
    -c core.hooksPath=/dev/null/cli-git-no-hooks
    commit <private commit args>
```

with `GIT_INDEX_FILE=<scratch directory>/commit.index`.
The globals the shadow command keeps are the caller's own except `-C`,
`--git-dir`,
`--work-tree` and `--bare`:
the wrapper has already applied every directory change,
and the shadow names its own repository,
so forwarding one of them would move Git away from the shadow or name the real repository beside it.
The shadow's `core.worktree` is the absolute worktree root,
which covers a Git process that a hook starts with the shadow as `GIT_DIR`
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
A preparation failure leaves the scratch directory to the sweep and leaves real index,
worktree,
and real ref bytes unchanged.
Do not invoke Git with a lock path as `GIT_INDEX_FILE`.

#### Wrapper output and options

The wrapper never appends `--quiet` or `-q` to a Git command it builds for the user's command line.
Owner decision (2026-11-03),
verbatim:
"No the wrapper should never append --quiet.
Letting native git emit whatever it wants to is expected."
A `--quiet` or `-q` that the user typed reaches Git untouched,
and Git's own standard output and standard error
reach the user as Git emits them.
The same holds for every Git run the wrapper or the service starts on its own.
Owner decision (2026-11-03),
verbatim:
"Drop it everywhere (Recommended)",
answering whether the wrapper's own internal runs
(maintenance,
the auto-push fetch,
fast-forward merges,
reset,
clean,
init) should drop `--quiet` too.
No such run carries `--quiet` or `-q` where the option only hides messages or progress.
Git's words are not discarded with the capture of the run:
the service records them as one entry at priority `info` in
its log,
which the service manager keeps in the journal of its unit,
and the wrapper records the words of a run it made while
carrying out the person's command as one client entry in the journal (`journalctl --identifier=monochromatic-git-policy-cli`);
both carry the field
`CLI_GIT_INNER_COMMAND`,
mask the secrets of URLs,
and are made only when the run said something.
The person's own streams carry what native Git's command prints:
the isolated merge's fast-forward says "Updating a..b" and
"Fast-forward" on standard output,
and that goes to the journal,
while a warning on its standard error still reaches the person.
`git reset` without `-q` lists as "Unstaged changes after reset:"
every tracked path whose file differs from the index
(measured on 2026-11-03),
so the service's journal now names the paths of other uncommitted work for every landing.
A `--quiet` stays on a run whose documented effect is the exit status or what is computed,
read in the Git 2.56.0 sources:
`rev-parse --verify --quiet` (`builtin/rev-parse.c`,
`die_no_single_rev` exits 1 where it dies with 128),
`symbolic-ref --quiet` (`builtin/symbolic-ref.c`,
returns 1 where it dies),
and `diff-index --quiet` and `diff-files --quiet`
(`diff.c`:
exit with status,
no renames,
an early quit).
`doc/planning/cli-git-slow-server-and-inner-quiet.md` lists every use and the line that decided it.
Consequently a commit that real Git builds prints Git's own summary (`[<branch> <name>] <subject>` and the change lines)
from the scratch repository,
and the wrapper prints no summary of its own for it.
A commit the service builds itself,
with no `git commit` run,
still prints the one summary line the wrapper writes
once the branch has moved.
A commit that Git builds again because the branch moved prints Git's summary for each build,
so the line of a build that did not land can precede the line of the commit that did.
Owner decision (2026-11-03),
verbatim:
"Add a wrapper note (Recommended)".
Git's output stays untouched;
once the commit has landed,
the wrapper adds one line to standard error
that names every build that did not land and the commit that did,
by the names Git's summary lines carry:
`monochromatic-git-policy-cli: the branch moved during the commit, so it was built again; 1a2b3c4 did not land, 5d6e7f8 did.`
Several builds that did not land are joined with commas and `and`.
A commit that was not built again prints no note.

Every option the wrapper itself adds to a command line precedes the user's `--`,
because Git reads every token after it as a path.
A `--` that is the value of an option (`git commit -m --`) is a value and not the separator.
A pathspec commit during a revert or a cherry-pick,
which Git accepts,
behaves as real Git does,
including paths named like options (`--quiet`,
`-o`,
`-dash.txt`) and `--pathspec-from-file`.
During a merge or a stopped cherry-pick,
which Git refuses with `cannot do a partial commit`,
the owner's rule "Commits should never fail."
(2026-11-03) applies instead:
the commit of the named paths is built in a scratch repository that holds none of the conclusion state,
so it is an ordinary commit of those paths on `HEAD`;
the worktree's `MERGE_HEAD` (or `CHERRY_PICK_HEAD`),
message and staged result stay,
and the commit without paths that concludes the merge sits on the new head.
A named path in conflict is committed too (owner,
2026-11-03,
verbatim:
"Committing a conflicted path is fine.
Warn and commit anyway,
even if conflict markers will be committed."):
the commit takes the bytes the worktree holds,
conflict markers included,
and once it has landed standard error says in plain words that the path still has conflict markers,
that they were committed,
and how to fix them in a follow-up commit.
That claim is made only for a path whose committed blob holds a marker line as Git writes it
(`conflict_markers.rs`:
the size of `conflict-marker-size` from the attributes,
seven by default,
equal `<`
 or `>` characters
followed by white space,
as Git's `is_conflict_marker` reads a line);
a path the person resolved by editing,
 with no marker line
left,
is counted as resolved and nothing is said about markers for it.
Whenever a commit of named paths leaves a merge or a cherry-pick in progress,
with markers committed or not,
 standard error
carries one more line that says so and names how to finish it:
stage any file still unmerged,
 then `git commit` with no paths,
or `git merge --continue` (`git cherry-pick --continue`);
 a follow-up commit of the path alone never finishes the operation.
The merge stays unfinished,
and the index entry of that path is the committed one.
A plain `git commit` that names no paths while files are still unmerged finishes the merge (owner,
2026-11-03,
question tool:
"Warn and finish the merge (Recommended)"):
Git refuses it (`builtin/commit.c:348`,
`refresh_cache_or_die`:
"Committing is not possible because you have unmerged files"),
and the service instead stages the working copy of each unmerged file in the commit's private index,
as `git add --all` stages it,
and builds the merge commit Git would build for a resolved merge (the parents,
the message and the removal of the merge state
are Git's).
The markers in those files are committed,
and the same warning as for a named path names each file and says how to fix it in a
follow-up commit.
`git commit -a` over unmerged files does the same:
it stages the working copy of every tracked file,
so it commits their markers and warns for each
such file in the same words (owner:
"Committing a conflicted path is fine.
Warn and commit anyway").
Options that select the whole index (`--all`,
`--no-only`,
`--include`,
`--interactive`,
`--patch`) keep concluding it.

#### The commit's scratch directory

The commit's scratch directory holds the private indexes and,
when real Git must build the commit,
the scratch repository.
It is made at
`<common-dir>/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-<id>-<attempt>`,
inside the repository scratch directory,
and never in the worktree's own Git directory as releases before this one made it.
The attempt's number is in the name because the service removes the tree of an attempt once that attempt's landing is over,
and the next attempt of the same command must not be the directory that removal is taking.
Code:
`src/native/foundation/scratch_location.rs`,
`src/native/landing/service_prepare_ready_shapes.rs`,
`src/native/landing/service_scratch_abandonment.rs`,
`src/native/landing/service_scratch_renewal.rs`,
`src/native/service/service_scratch_release_removal.rs`,
`src/native/service/service_scratch_trees_of_queued_requests.rs`.

Why there:
a command killed at any instruction,
and a machine with no service,
leave trees behind,
and only the directory the sweep already covers collects them.
The place is also swept of entries older than `SCRATCH_STALE_AFTER` at every preparation of it,
which a preparation of a commit now is.

Who removes it,
and how long anyone waits:

- The service collects the repository scratch directory when it begins a landing,
  on a thread of its own beside the landing loop and never on a commit's path
  (`src/native/service/service_repository_scratch_sweep.rs`).
  One collection runs at a time for each directory:
  a second landing of the same directory starts nothing and leaves the collection to the run already under way,
  and a landing of another directory starts its own,
  because the development service serves several common Git directories at once
  and nothing retries a collection one directory's run skipped.
- A request names its tree in the request's own `scratch` field,
  which the decoder refuses unless it is one of the wrapper's trees of that worktree,
  so the service never works the tree out from another path the request carries
  (`index` is the worktree's real index,
  which is not a tree).
  The resident service removes that tree once the landing of that request is over,
  in a task beside its landing loop and bounded by `SCRATCH_REMOVAL_CEILING`,
  with a line at warning priority in the service journal when the removal cannot finish.
  Neither the client nor the landing loop waits for it.
  A request the service did not land,
  one it sent back to be prepared again (`retry-prepare`) or one whose branch moved (`branch-moved`),
  is not one whose landing read the tree:
  the tree goes back to its client,
  which reads the change it captured from the tree's private index to make the commit again
  (`retry_of` in `src/native/landing/service_replay.rs`)
  and releases it beside its own command,
  in a task and under the same ceiling.
  Removing it in the service took that read from under every rebuild,
  which failed with a private index that was not there.
  Nothing else in the service removes a tree a request named:
  the tree holds the private index and the conclusion copies the landing reads,
  and a client killed while its request waits is not evidence that the landing is over.
- A command that never submitted its tree,
  because the service refused it or because the preparation stopped where it was,
  removes it itself as it goes out of scope,
  with the same ceiling,
  so a declined or abandoned commit leaves nothing.
  A command whose submission got a definite answer that the service never took the request
  removes its tree the same way:
  the service confirmed the withdrawal (`StillWithService`),
  refused the request before queueing it (`Refused`),
  or could not be reached at all
  (`the_service_never_took` in `src/native/landing/service_commit.rs`).
  Every other failure leaves the tree where it is,
  because an unanswered request is not proof that nothing is running:
  an unconfirmed withdrawal,
  a silence and a broken connection can all leave a branch write inside the landing,
  which reads the private index and the conclusion copies in that tree.
  Such a tree is the service's to remove when it lands,
  and otherwise goes by the age rule that follows.
- A tree a request this service still holds goes when the service is done with that request,
  never by its age alone:
  the service records the tree every request it queues names,
  and the one decision that removes a commit tree,
  `may_go`,
  asks that record.
  A request whose client was killed while it waited renews nothing,
  so without the record a queue longer than `SCRATCH_TREE_ABANDONED_AFTER` made the collection a landing starts remove
  the private index and the conclusion copies that same landing was about to read,
  and the cleanup after its branch moved found neither the tree nor the journal's copy of it.
  The record is released when that request's group is over,
  whether the request landed,
  was handed back to its client,
  or was withdrawn,
  so what is left of its tree is collected by the age rule below.
  The record lives in one service,
  so it covers the requests that service holds;
  the loss the subsection "A command stopped for longer than the tree's age" describes
  needs another service or a command of another worktree to sweep the same repository scratch directory.
- A tree no request ever named,
  because the command was killed before it submitted or before it made the tree at all,
  goes when nothing has written it for `SCRATCH_TREE_ABANDONED_AFTER` (two hours,
  six times the longest wait a commit can have).
  Its command renews the tree's modification time every `SCRATCH_REFRESH_EVERY` (an hour) while it lives,
  so the age a sweep sees is the liveness of the command that made the tree,
  not how long its work takes.
  Age is the only evidence a collection has of a tree no request of this service names,
  and it is enough here because no landing can still be reading a tree nothing has written for two hours.
- One decision answers which entry may go,
  for the sweep of a scratch place and for every other collection
  (`may_go`,
  `service_scratch_abandonment.rs`):
  a commit tree at `SCRATCH_TREE_ABANDONED_AFTER`,
  unless a request this service holds still names it,
  and every other entry of a scratch place at the age its caller gives,
  which is `SCRATCH_STALE_AFTER` (a day) for the sweep.
- At most `SCRATCH_TREES_REMOVED_PER_COMMAND` (eight) entries go per sweep,
  oldest first,
  so a backlog another command left delays this one by that many removals and never by the backlog's length,
  and the next sweep takes the next eight.
  That is what bounds the trees a kill leaves:
  one tree of about 9 MiB per killed command,
  gone at the first sweep after two hours.
- A leftover of a command's own earlier attempt whose removal its ceiling cut short is moved aside under a name no
  commit tree has,
  where the sweep collects it by age,
  and the attempt creates its own tree as it always does:
  a stalled device costs a removal and a debug line,
  never a failed commit.
- Every removal of a scratch entry runs on a thread of its own and stops being waited for at
  `SCRATCH_REMOVAL_CEILING` (thirty seconds,
  twelve times the slowest single `unlinkat` the timing record holds),
  so a stalled device costs a partial removal and a reported line,
  never a command that waits.
  A removal that outlasted its ceiling holds one of the process's places for blocking work until its thread ends,
  and a removal that finds every place held takes none and leaves its entry to a later sweep,
  so stalled removals cannot gather without bound.
  A name that is not one of the wrapper's scratch trees,
  and a name that holds a link or a file,
  is never removed by that rule.

Two linked worktrees of one repository share that repository scratch directory,
so a client that names **another worktree's** tree is accepted,
and the tree is removed once the landing of the request that named it is over,
even though the worktree that owns it still runs.
The wrapper never sends such a request:
each one names the index inside the tree the command itself made,
whose name carries that command's identity and its attempt's number.
A client that sent one anyway,
by a defect or on purpose,
would take another worktree's private index and typed message with it,
and the service cannot tell the two apart:
the announced Git directory is the one the client says it serves,
and the tree's name is one the wrapper makes.

A cleanup a request carries,
which names the directory of a commit built elsewhere,
is refused at the decoder unless it is exactly one of those two places,
holding the wrapper's prefix and an identity of letters,
digits and dashes,
with no relative part that could lead elsewhere
(`is_scratch_path`,
`service_protocol_messages_and_name_checks.rs`),
so no request can ask the service to remove a directory the wrapper did not make.

A service of an earlier release does not know the message a client of this one used to announce a tree,
and the client does not read that refusal;
such a tree carries nothing a later command reads,
so it goes at `SCRATCH_TREE_ABANDONED_AFTER`,
which costs one tree of about 9 MiB per worktree for two hours while an installation switches releases:
the next commit retires the service of the other release and starts this one.

##### A command stopped for longer than the tree's age

A command that stops running,
by a stopped process,
a frozen cgroup,
or a machine that is not running it,
stops renewing its tree,
so the tree's age keeps growing while its command still lives.
Once the tree is older than `SCRATCH_TREE_ABANDONED_AFTER`,
the sweep of another command removes it,
and the stopped command resumes without the private index it was writing:
its next Git child fails,
and the commit reports that failure.
No rule distinguishes a stopped command from a killed one,
because the only fact that would (a process that still exists)
is exactly the fact that cannot be acted on:
the tree of a killed client whose request is queued belongs to a process that is gone,
and the landing that read it may still be running.
The window is therefore the whole of a stop longer than two hours,
and not only the moment between creating the tree and writing anything into it.
The wrapper accepts losing that tree.
What keeps every other live command's tree is the renewal,
which needs no marker file and no reader of `/proc`.

A queued request loses more than its private index in the same window,
and the sequence needs a **service** stopped,
not only a command:
a client prepares a conclusion commit and the service queues the request;
the service is frozen before that landing and stays frozen for longer than
`SCRATCH_TREE_ABANDONED_AFTER`,
so its queue survives;
the client is killed,
so nothing renews the tree and nothing can withdraw the request;
a commit in another linked worktree then prepares the shared repository scratch directory,
and the sweep takes the queued request's tree on its age alone,
because that process's record of the trees its own requests hold
(`service_scratch_trees_of_queued_requests.rs`)
names no request of the stopped service,
and the tree carries nothing that would show whose it is.
When the service resumes it does not land the request:
a request whose client is gone before its commit lands makes no commit,
so the landing of that group finds the connection closed and drops the request
(`src/native/service/service_server_land_group.rs`),
and no conclusion cleanup runs at all.
What the person sees:
the commit in the other worktree is unaffected,
nothing on the killed command's terminal says anything because that command is gone,
and the worktree that prepared the commit still reports its merge or cherry-pick in progress,
because nothing landed to remove that state.
An operating service lands a queued request long before its tree reaches that age,
and the collection that service itself starts spares the tree it has still to read,
so the sequence needs a service stopped for longer than that age,
which is why the age rule stays.

#### Scratch repository layout

The scratch repository (the shadow,
in the code and in this section) is made only when real Git must build the commit.
It lives in `<common-dir>/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-<id>/shadow` and holds
these private entries:

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
  monochromatic-git-policy-cli writes the snapshot as one `packed-refs` file
  (header `# pack-refs with: sorted `,
  with the trailing space Git's own writer emits and older readers require)
  and writes the private target ref as a loose ref through `git --git-dir=<shadow> update-ref`,
  which takes precedence over the packed entry.
  With the reftable backend,
  monochromatic-git-policy-cli writes the snapshot through one `git --git-dir=<shadow> update-ref --stdin` transaction.
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
  written by monochromatic-git-policy-cli
  (see the config layout in this subsection).
- `config.worktree`,
  copied from the owning worktree's Git directory when present.
- `COMMIT_EDITMSG`,
  which native `git commit` rewrites during every commit,
  so concurrent preparations never share one message file.
- The copied conclusion state,
  which is made only when the worktree holds some
  (see "Sequencer conclusion state").
- `monochromatic-git-policy-cli`,
  which is never linked.

The entries a commit reads are symbolic links to the real ones,
and they are named (`shadow_links.rs`,
`SHARED_ENTRY_NAMES`):
`info`,
`hooks`,
`lfs`,
`modules`,
`rr-cache`,
`shallow`,
`svn`,
`branches`,
and `remotes`.
`gitrepository-layout` (Git 2.56.0) names the entries Git resolves from `$GIT_COMMON_DIR`,
which is the same question a shadow asks:
the ones the shadow keeps its own (`config`,
`logs`,
`objects`,
`packed-refs`,
`refs`) are left out of it,
and so is `worktrees`,
which no commit reads and through which a shadow would claim the real repository's linked worktrees.
`lfs` is the folder `git-lfs` keeps its objects and logs in,
and a clean filter a commit runs writes there.
The list is named because the rest of what a common directory collects is either the shadow's own
or state no commit reads:
the wrapper's own logs and folders,
other tools' files,
`FETCH_HEAD`,
`ORIG_HEAD`,
`description`,
and `lost-found`.
A lock,
and the file a command stages a lock in
(`<lock name>.cli-git-marker-<pid>.<pid namespace>-<batch>-<token>`),
is never linked:
a shadow Git that wrote one would collide with the real holder,
and every command of a repository stages a marker of its own while it runs.
The wrapper's own state never is either:
the repository scratch root,
which holds the shadow itself,
and the pool of finished shadows beside it,
which holds the private trees of other commits.
A link to either would let one shadow reach another's tree,
and whether the pool exists would decide whether a claim trusts a member.
A per-worktree entry that Git resolves in the owning worktree's Git directory,
such as a linked worktree's `info/sparse-checkout`,
is copied;
a directory that mixes shared and per-worktree entries becomes a private directory
whose shared entries are linked individually.
`lfs`,
where `git-lfs` keeps its objects and logs,
is linked whether the repository holds it or not:
the creation makes it first when it is absent,
because a clean filter of the commit writes its object there,
and a filter that finds nothing to write into leaves the object in the private tree,
which is released with that tree.
The shadow has no `index`;
`GIT_INDEX_FILE` names the private commit index.

The shadow `config` has this layout:

```ini
; <git-common-dir>/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-<id>/shadow/config
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
  so the native commit never runs `post-commit` in the shadow,
  and nothing runs it after landing.
  It is a file value,
  not a `-c` option,
  so no hook inherits it through `GIT_CONFIG_PARAMETERS`.

Cli-git writes the file itself,
quoting every path value under git-config value syntax at the final interpolation.

#### A shadow taken from the pool

The creation of a shadow is about forty calls that create a directory entry on the volume a commit already waits on,
and the owner chose to stop paying it on every commit
(`doc/handover/questions-for-the-owner.md`,
"May a commit take its private repository from a pool made ahead?",
"Pool made ahead (Recommended),
We don't need to build a shadow for every commit.
Be smarter.").
So a commit takes a finished shadow from a pool when the pool holds one this invocation can use,
and makes its creation exactly as it did before when it does not:
an empty pool is not a failure,
and nothing about a claim can fail a commit.

The pool lives in `<common-dir>/monochromatic-git-policy-cli-shadow-pool/<worktree>`,
one folder per worktree because a member's `config` names the worktree root the commit runs in.
`<worktree>` is `main` for the main checkout,
whose own Git directory is the common directory,
and `worktree-<name>` for a linked one,
named after its own Git directory,
and one rule names it on both sides (`pool_key` in `shadow_pool.rs`):
the fill and a claim that would look at one folder,
and the sweep that removes it.
It is its own folder beside the repository scratch directory and never an entry of it:
a member is standing state the owner asked for,
not scratch,
so the check that a commit leaves no scratch behind keeps meaning what it says,
and neither the sweep a command that takes scratch runs
(`prepare_repository_scratch`),
nor the release of a commit's scratch tree after its landing,
nor the reclamation of a removed worktree's scratch treats a pooled shadow as a leftover.
The folder is the wrapper's own state of the same kind as the scratch root,
so no shadow links it
(`shadow_links.rs`,
`is_shared_shadow_entry`).
A member is everything the creation makes that does not depend on the commit:
the skeleton,
the private mode,
the `config`,
`objects/info/alternates`,
the links and the copied `config.worktree`.
It holds no ref store and no conclusion state,
which a commit adds after it takes one,
from this invocation's own capture.

- A claim is one `rename` of a member onto the commit's `<scratch>/shadow`,
  which fails for the second claimant,
  so no lock is taken and no commit waits for another
  (`AGENTS.md`,
  `NLK`).
- A claimed member is checked before it is used,
  and everything the check reads is a read:
  its `config` must be the text this invocation's creation would write,
  its alternates must name this invocation's object directory,
  its `HEAD` must be the creation's own placeholder,
  every directory of the skeleton must be there,
  every entry a commit reads that the repository holds now must be a link to this repository's own entry,
  the store a clean filter writes into must be a link there whether the repository held it at the fill or not,
  every entry the member holds must be one a creation makes or a link at one of those names,
  and a link at one of those names must reach the entry it names,
  and no pack and no ref.
  An entry a commit reads that the repository has gained since the fill is a refusal,
  because the member would give the commit the view of the moment it was made;
  an entry it has lost is a refusal as well when the member still links it,
  because a creation of this moment links no name the repository does not hold,
  and makes the store a filter writes into,
  and a filter or a hook that writes into the directory a link names fails on one that resolves to nothing
  (`git-lfs`'s clean filter writes its object into `<git-dir>/lfs`).
  The check follows each link it accepts once,
  so a member filled while `lfs` or `rr-cache` was there and claimed after it went is removed rather than trusted.
  A member that fails the check is removed and the next one is tried,
  and a pool with none left leaves the commit making its creation.
- A refusal is the wrapper's own cache being repaired,
  which the person can neither cause nor fix:
  it leaves one debug line under `MONOCHROMATIC_VERBOSE=true`
  and one build part named `shadow pool refused` in the commit's timing record,
  beside the creation the refusal forced,
  and nothing on the terminal.
- A reftable repository and a linked worktree that keeps its own `info` are never pooled:
  Git writes those shadows' own tables and entries,
  so their commit makes its creation,
  and a fill makes no member either.
- The resident service fills the pool beside the landing loop,
  never on a committing person's path:
  it reads a repository's format facts and object directory the way an invocation's capture does,
  makes one member into a folder named as a fill,
  publishes it whole with one `rename`,
  and stops at two members for a worktree.
- A fill that a stop interrupts leaves a folder named as a fill,
  which no claim takes and the next fill of that pool removes once it is fifteen minutes old.
  Nothing a sweep of scratch removes reaches a member,
  because the pool is not scratch:
  no age a sweep uses reaches one,
  and a pool of a worktree Git still lists keeps its members until the next landing fills it again
  or a person removes the folder.
- A worktree Git no longer lists has no pool:
  every wrapped command removes the folder of a worktree whose own Git directory is gone
  (`sweep_gone_worktrees` in `shadow_pool.rs`).
  `git worktree prune` deletes the registration alone,
  and a registration deleted by hand leaves even less,
  so the members of that folder would otherwise keep the shape of a checkout that is gone for ever.
  The question is asked of the repository's own `worktrees` folder and never of Git,
  so a command whose repository has no pool pays one `read_dir` that finds nothing.
  A worktree Git is adding keeps its folder,
  because Git makes a registration before it makes the checkout,
  and a folder the wrapper did not name stays,
  because only a folder named for a worktree can be proven to belong to one.
  A removal that fails is named in the diagnostic log and left for the next command,
  and nothing it removes is followed through a member's links into this repository.
  A removal that succeeds prints nothing:
  no member holds a ref or a commit of the person's,
  so there is nothing for them to do about it,
  and a line on every command that happened to run next would say nothing that changes it.
- A removal that takes a worktree also takes that worktree's pool
  (`worktree_removal_clears_the_pool.rs`),
  and the pool is renamed to a name no worktree's key has before anything of it is deleted.
  A deletion keeps unlinking through the directory it already opened,
  so a claim that lands under the pool's own name after the deletion opened the member would lose the ref store the commit had just written there,
  and the commit would report a failure real Git did not have
  (`doc/troubleshooting/remove-dir-all-empties-a-directory-renamed-out-of-its-tree.md`).
  A claim under the pool's own name either came before the rename,
  and then the member it took is not in the tree the deletion was given,
  or it finds no pool and the commit makes its own shadow.

`buildParts` names the claim `shadow pool`:
a build that took a member names no `shadow skeleton`,
`shadow config` or `shadow links` stretch,
so a reading of the record can tell the one route from the other,
and a claim that refused a member names `shadow pool refused` beside them.

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
Nothing here promises their order,
environment,
locking or exit-status handling,
and the wrapper starts none of them.
Git never starts the wrapper for a hook:
the wrapper writes no hook entry,
passes no hooks path of a directory it made,
and the executable does not read the name it was started under.

- Every real-Git start of the wrapper carries `-c core.hooksPath=/dev/null/cli-git-no-hooks` and
  `-c hook.<event>.enabled=false` for every hook event Git 2.56.0 knows (`HOOK_EVENTS` in `hooks_off.rs`,
  checked
  against the host Git by a control),
  after the caller's global options and before the subcommand.
  `hook.<event>.enabled=false` switches off every configured hook of the event whatever its name
  (`doc/troubleshooting/git-hook-disable-switches.md`),
  and `core.hooksPath` switches off the hook files.
- The shadow's `config` switches every hook event off after its include,
  so no hook runs in the shadow whatever the
  real config defines,
  and the native commit repeats the hooks-off hooks path after the caller's global options.
- The native commit runs `git <caller global options> --git-dir=<shadow> -c include.path=<scratch>/native-commit.config
  -c core.hooksPath=/dev/null/cli-git-no-hooks commit ...` with no `--work-tree`:
  the shadow's `config` names the work tree.
  A caller's `-c core.worktree` cannot redirect it:
  with an explicit `--git-dir`,
  Git reads the work tree from the git
  directory's own config file only (measured on Git 2.56.0),
  and the wrapper removes `GIT_WORK_TREE` from the environment.
- `native-commit.config` in the commit's scratch directory states every entry of the keys native `git commit`
  reads that the invocation resolved,
  in the order Git resolved them,
  because an `includeIf` condition that names the real Git directory no longer matches in the shadow.
  The entries travel in one included file and not as `-c` settings:
  a `-c key=value` argument stops a key at its first equals sign,
  so a subsection such as
  `[trailer "review=bot"]` would lose its configuration,
  and an argument stops at the platform's single-argument limit,
  which a configured value can exceed (measured on Git 2.56.0:
  262144 bytes is
  `Argument list too long` as one argument,
  and reads back from the file unchanged).
- A repository that configures hooks gets one `configuration-warning` event (code `external-hooks-unsupported`) on
  each commit,
  decided from the configuration snapshot the commit already holds:
  it names the `hook.<name>` keys,
  their
  events and the ways forward (remove the keys,
  or express what the hook checked as a monochromatic-git-policy-cli policy under `policies`
  in `monochromatic-git-policy-cli.config.jsonc`).
  A hook the author switched off is not named:
  neither by its own `hook.<name>.enabled=false` nor by every event it asks for having
  `hook.<event>.enabled=false`,
  which is Git's own switch for the whole event
  (`git hook list` reports such a hook as `event-disabled`).
  The two switches are read from different places,
  because the wrapper supplies only one of them:
  - `hook.<event>.enabled` is read from the configuration files,
    the `command` scope left out,
    because the
    wrapper itself supplies `false` for every event on every Git it starts,
    a Git child passes those values on through
    `GIT_CONFIG_PARAMETERS`,
    and reading them here let the wrapper's own suppression hide a hook the person
    configured,
    from doctor and from the commit warning alike.
  - `hook.<name>.enabled` is read the way Git reads it,
    every scope,
    the `command` one last and
    winning,
    because the wrapper writes no such switch at all:
    a `-c hook.<name>.enabled=true` is the caller's own
    choice,
    and a `-c ...=false` is too,
    so neither is discarded.
  A hook that asks for no event,
  or has an event and no command,
  is named:
  the first never runs and the second makes Git refuse the event,
  and the warning says neither runs through the wrapper.
  `git monochromatic-git-policy-cli doctor` lists the same hooks,
  one `hook.<name> (events: ...)` line each,
  and exits 1 while any remain.
  A push prints the same warning,
  from the snapshot it already reads for the LFS decision.
  A forwarded command other than a commit or a push prints no warning,
  because it reads no configuration snapshot.
- `hooks.concurrentCommits` is retired:
  an older file that sets it loads,
  the value is ignored,
  and doctor says so.

The wrapper itself executes no program that any configuration names:
not a Git config key,
not `monochromatic-git-policy-cli.config.jsonc`,
not the git directory,
not the environment.
The programs Git starts for its own features (filters,
editors,
signing programs,
credential helpers) are Git's,
not the
wrapper's,
and stay;
a program that is a hook event is switched off.
`core.fsmonitor` is among them,
and the wrapper's own Git passes `-c core.fsmonitor=false` rather than relying on the person's setting;
a command the person typed keeps it:
it is off in the person's global configuration
(owner decisions,
2026-11-03 and 2026-11-10,
the later one asking "Should the wrapper switch fsmonitor off for its own internal Git commands only?"
and answering "B:
off for internal Git (Recommended)"),
and `git monochromatic-git-policy-cli doctor` says so.
`monochromatic-git-policy-cli.config.jsonc` is data:
it has no command or executable field,
and a `postCommit` key is a configuration error
(`Configuration key postCommit is not supported: monochromatic-git-policy-cli runs no program named by any configuration ...`).
What a hook checked belongs in a monochromatic-git-policy-cli policy.

### Policy evaluation during preparation

The candidate listing reads its pathspecs with magic whatever global pathspec mode the caller set:
a commit's paths are listed as `:(top,literal)<path>`,
and `GIT_LITERAL_PATHSPECS` or `--literal-pathspecs` would make each of those names match nothing.
Every listing therefore runs real Git with `--no-literal-pathspecs` after the caller's global options,
so no content policy can be given an empty candidate list by a global mode.
Three further private Git starts carry the same switch:
the whole-index staging `commit -a` does in the private index (`:/`),
the staging of each unmerged worktree copy that finishes a merge,
and the settled-entry lookup that installs a landed correction into the worktree copy.
That lookup also compares the pathname of every entry Git lists with the path it asked about:
`GIT_ICASE_PATHSPECS` makes `:(top,literal)<path>` list every name that differs only in case,
and the first of them is not necessarily the corrected path.
Only the replay of a caller's `git add` keeps the caller's own mode.
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

1.  Build a commit index from the preparation base:
    copy the real index captured at invocation and reset the copy to the base,
    so every entry the base shares with the real index keeps its cached stat data
    and the scratch `git commit` does not read and hash the whole work tree
    (measured:
    233,618,538 bytes of LFS content cleaned and 3.6 s for an index built from the base alone,
    against 0.1 s for a copy).
    Intent-to-add entries stay in the copy,
    because they carry no cached stat data and copy as they are,
    and the paths they name are cleared before the reset
    (`write_index_for_reset`,
    `commit_index_copying.rs`):
    the reset to the base then writes the base's own entry at each of those paths,
    replaces one whose mode or object name differs,
    and drops one the base does not hold.
    A mark whose mode and object name match the base's entry survives native Git's own reset,
    and a tree built from an index that holds it leaves the path out as soon as anything invalidates the cached tree
    (measured with Git 2.56.0:
    staging the selected path does,
    and so does a scratch-commit hook),
    so the wrapper keeps a tracked file nobody named.
    The selected additions are carried afterwards
    (`git diff-index --cached --diff-filter=A` against the base,
    step 2),
    and that step is never skipped for an index holding such a mark:
    a force-added ignored file the real index holds as intent-to-add is an addition the selection names,
    and native Git records it with its worktree content (measured against Git 2.56.0).
    An index that holds an assume-unchanged or skip-worktree entry,
    or one the in-process readers do not understand,
    is built from the base alone instead:
    a copied mark makes `git add --all` skip or refuse a file the commit named,
    where native Git stages it.
    A rebuild after the branch moved builds its index the same way (`initialize_retry_index`),
    from the same copy and through the same clearing,
    so it holds exactly the requested tree,
    with the same answers,
    and neither preparation keeps a rule of its own.
    One window stays open in that rebuild and is written here rather than closed
    (`initialize_retry_index`,
    `commit_index.rs`):
    the real index is classified at the start of the build and copied at once,
    and a command that replaces it between the two
    leaves the copy holding intent-to-add entries the classification never named,
    so the clearing above does not reach them.
    A second program inside those few instructions can therefore make a retry record a deletion the person did not make:
    the reset keeps a mark whose mode and object name match the requested tree's entry,
    and a scratch-commit hook that restages a file the commit names invalidates the cached tree,
    after which the tree written from that index leaves the tracked file out
    (measured with Git 2.56.0:
    the mark survives the reset,
    the tree still holds the path while the cached tree stands,
    and the restage drops it).
    No lock covers it:
    the real index belongs to the person and their other commands,
    and a lock held across the copy would stop them to protect a path nothing named.
    The stat refresh of the private index runs after it,
    on that index alone (`commit_index_stat_refresh.rs`).
2.  Carry into that index every selected entry that the real index stages as an addition
    (`git diff-index --cached --diff-filter=A` against the base,
    with the same pathspecs;
    a pathspec file is read by the wrapper and passed as arguments,
    and a file it cannot read is left to Git alone).
    Native Git tracks these paths through the real index and takes their worktree content without asking an ignore rule,
    so the private index must track them too (issue 1807).
3.  Add selected worktree paths to that index using Git pathspec semantics,
    and only the paths native Git takes for the same pathspecs.
    A selected element that names one file of the worktree is an exact path:
    it carries no pathspec magic,
    no pattern character of the mode in effect (`*`,
    `?` and `[`;
    they are ordinary in a literal or a noglob mode),
    and a file or a link is at the pathname it gives.
    That pathname is read from the directory Git reads a relative pathspec from (`pathspec_base`):
    the directory the command runs in when it is inside the worktree,
    and the worktree's own top when it is outside it,
    which is where a command line that names another worktree with `--git-dir` and `--work-tree` is read
    (`rev-parse --show-prefix` answers nothing there).
    Which worktree that is comes from Git's own answer for the invocation
    (`rev-parse --show-toplevel`,
    `InvocationLayout::repository_root`),
    so every way of naming one reaches it:
    the two options,
    `GIT_DIR` with `GIT_WORK_TREE` in a command's environment,
    `core.worktree`,
    and a linked worktree's own `.git` file.
    A reader that derived the worktree from the command line's own `--work-tree` again found none of the other shapes,
    took the invocation directory for the base,
    and left a commit from outside the named worktree with the staging of the wrong directory.
    It is staged with `git add -- <element>`,
    so `git commit -- <file>` records a file nothing staged,
    where native Git refuses the pathspec,
    and a path that an ignore rule matches and that no index stages is refused before that staging runs,
    with the finding `explicit-path-commit/named-path-ignored` (see "Core finding event"):
    the real index is read in process,
    and the selection's own elements that name one file and that the index holds no entry at are asked of
    `git check-ignore -z --stdin` under the real index in one start,
    so nothing depends on the words of a Git message and a path the real index tracks is left out,
    as `git add` leaves it out.
    A commit whose named paths the index all holds starts no Git for this question,
    which the spawn budget of `binary_budget_tests.rs` counts;
    a reader that declines the index leaves every name to Git,
    as before.
    The commit is not made,
    nothing is staged by it,
    and the message names every rejected path with both ways forward:
    leave the file out of the commit's paths,
    or stage it with `git add --force -- <path>` when it is meant to become tracked.
    Real Git 2.56.0 on a disposable repository refuses the same command line either way:
    `git commit -- <untracked ignored path>` exits 1,
    its pathspec matching no file Git knows,
    and `git add -- <the same path>` exits 1 with
    "The following paths are ignored by one of your .gitignore files"
    and the force-add hint.
    A backslash is Git's escape and not a pattern character:
    `a\b.txt` names the file whose pathname spells it when one is there.
    Every other element is a directory,
    a pattern or pathspec magic,
    and it stages the tracked paths the private index holds and the carried additions,
    never a file nothing staged,
    with `git add --update -- <element>`:
    native Git commits the tracked and staged paths a directory or a pattern matches,
    so a commit of `.`,
    `*` or a directory leaves every untracked file untracked.
    The elements that subtract paths (`:(exclude)`,
    `:!`,
    `:^`) are part of the whole selection,
    so they ride in the exact-file start as well:
    native Git reads `git commit -- a.txt ':(exclude)a.txt'` as no path at all,
    and one `git add` per element would stage the file the same selection excludes.
    A list that holds only subtracting elements reads as every path when the caller wrote nothing else,
    which is native Git's reading of it;
    when the caller named exact files instead,
    that start is left out,
    because those elements are already the positives of the selection.
    A pathspec whose elements match no entry of the private index is refused as native Git refuses it:
    the `git add --update` start fails before anything lands,
    and its "pathspec ... did not match any files" sentence is reported as
    the `explicit-path-commit/pathspec-matches-nothing` finding (see "Core finding event"),
    which names the token the caller wrote and,
    when that token begins with a hyphen,
    says that options belong before `--` (`git commit --message <text> -- <paths>`).
    A token that itself holds Git's closing wording
    (`x' did not match any files`) is named whole,
    because only a wording that ends Git's line closes the refusal.
    Every other failure of that start stays an engine failure (`commit_pathspec_matching_nothing.rs`;
    issue 6283).
    The start whose sentence this is runs under `LC_ALL=C`,
    so the reading holds under any locale the person's shell asks for
    (`private_index_request.rs`,
    `caller_pathspec_request`).
    An empty pathspec file stages nothing,
    as native Git commits nothing for one.
    A pathspec file with more bytes than one command line may carry is passed as a file with
    `git add --update`,
    so the commit takes only the tracked paths it matches and a file nothing staged stays out
    (`commit_selection_staging.rs`,
    `stage_the_callers_pathspec_file`).
    Git refuses that whole start when one path in the file matches nothing,
    and that refusal is the same finding,
    naming the path Git named;
    a pathspec file this wrapper cannot read is left to Git alone.
4.  Apply policy patches to the commit index.
5.  Remove pathspecs and internal `--only` before invoking native Git
    because the private index is the complete intended tree.
    Only a token in option position is removed:
    the value of an option is the person's text,
    so `git commit a.txt -m a.txt` lands the path `a.txt` with the message `a.txt`
    and the message is never taken for the path it spells,
    and long options are read the way Git reads them,
    every unambiguous prefix included,
    so the value of `--mess` stays
    (`commit_args.rs`,
    `command_options_long_resolution.rs`).

Merge,
cherry-pick,
and revert conclusions use index-commit semantics only.

#### Added paths

A commit adds no paths of its own but the dependents of a hand bump,
and every rewrite a policy made is installed into the worktree by the wrapper after the landing,
only while the worktree copy still holds the bytes the correction replaced.

Direct fix admits added paths under the precondition in "Finding and patch validation":
`HEAD`,
the real index,
and the worktree copy all hold the blob the correction was computed against.
It has no journal or index step:
converged bytes of added paths join the direct-fix worktree installation.
Precondition failures are `patch-conflict` with direct-fix remedies
(include the path in the pathspecs,
or restore it to `HEAD`).

The commit path admits them under the same precondition,
checked against the commit's own index as well
(`policy_added_path::commit_refusal`),
and the run of dependents is planned from that index,
so a correction is computed against the bytes the commit holds.
It differs from the fix in what a failed precondition costs:
nothing is refused,
the dependent is left out of the commit and named in a note
(see "Dependent version bump"),
and the commit lands with the dependents it could take.
The landing then writes the commit's entries into the real index,
which is why the real index must already hold them.
The listing that reads the real index names the repository the command line does,
from the location Git answered for the invocation
(`repository_selection::of_identity`),
which already holds the Git directory and the work tree those options selected,
because a Git directory beside its work tree has no `.git` in the work tree to find
and a relative `--git-dir` read again from a discovered directory would apply the caller's `-C`
a second time and name no repository at all.
A listing that cannot run is not a verdict that the index holds nothing:
each dependent it would have proved is left out of the commit and named with Git's own reason,
and the commit lands with the rest.

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
The copies also name the conclusion values a reftable repository keeps in its ref store
(`conclusion/store-held.json`,
`store_names_conclusions` in `conclusion_cleanup.rs`):
a backend whose store can hold such a value writes the record and flushes it,
a files-backend repository can hold none,
so its copy writes no record,
an absent record names nothing there,
and no commit waits for the flush of bytes that would say nothing
(`doc/planning/cli-git-commit-latency-under-many-agents.md`).
The landing removes the same state from the worktree exactly as native Git removed it in the scratch repository,
each entry only while it still holds the bytes copied at preparation,
and journals the cleanup with the batch.
`MERGE_RR` is the exception among those entries:
the landing writes the scratch repository's copy back instead of removing it,
because `git rerere` rewrites that file while a merge waits,
and the commit made in the scratch repository leaves it as the same commit leaves it natively.
A linked worktree's own cache therefore ends with the bytes the same conclusion leaves with real Git,
which `binary_merge_rr_cleanup_tests.rs` compares byte for byte,
and a merge the word-level second pass finishes itself is compared the same way.
Where `rerere.enabled` is false,
`git merge` writes no cache at all
(measured with Git 2.56.0 on a conflicted merge),
and a merge the second pass finishes leaves none either.
A cleanup that fails cannot be redone once the wrapper has removed its scratch directory.

The copies folder is made only when the worktree holds conclusion state:
a commit that concludes nothing has nothing the folder could hold,
the landing answers an absent folder as no conclusion state at all
(`reproduce_conclusion_cleanup`),
and the one `mkdir` it used to make was the largest stretch of a build in the records
(`doc/planning/cli-git-commit-latency-under-many-agents.md`,
"What each part of a commit's private repository is for,
and how many commits need it").

### The amend a moved branch makes

An amend whose branch is no longer at the commit it replaces lands its changes as a new commit on the branch as it is now
(the owner's choice,
2026-11-09:
"A:
land as a new commit (Recommended)";
the refusal and a rewrite of the commit another session landed were both rejected).
The result is the replaced commit,
then the commit that moved the branch,
then the amendment,
and neither earlier commit is changed.
The commit is made from:

- its tree:
  the amendment's own tree merged onto the branch as it is now by the replay rule above
  (`git merge-tree --write-tree` from the replaced commit as the merge base);
- its one parent:
  the branch as it is now;
- its author,
  committer and message:
  those of the amend,
  kept as stored,
  so the author's date survives the rebuild;
- its header lines and its signature:
  written whole by `git hash-object -t commit -w` under the identities and message the amend made,
  or,
  when the amend was signed,
  signed again by `git commit-tree -S` with the `-S` option the command line named,
  whose other header lines a signature cannot carry are dropped with a warning.

The rule holds for every build,
not only the first:
a branch that moves again while the new commit is being made makes that commit again on the branch as it is now,
and the rebuilding ceiling is what ends an amend whose branch never stops moving,
naming the builds it made and landing nothing
(`after_a_moved_branch` in `src/native/landing/service_commit_after_a_moved_branch.rs`,
which decides what each kind of commit the service answered `Moved` becomes).
Nothing waits for another command and nothing is refused for it.
The person is told what the amend became after it lands:
a line naming the branch,
the commit that landed,
and that the commits already there were left as they are
(`amend_lands_as_new_note` in `src/native/landing/landed_summary.rs`).
Code:
`src/native/landing/service_amend_replayed.rs`.

### Word-level second merge pass

Git's line merge reports a conflict where two changes touch (one ends where the other begins):
a line edited on one side and lines inserted right after it on the other.
The owner decided on 2026-11-03 (`doc/planning/cli-git-merge-auto-resolution.md`) that the wrapper re-merges only the conflicted
parts word by word,
the way Jujutsu merges with `merge.hunk-level = "word"`,
accepts a file only when every part resolves,
and lists each merged part on standard error and in the journal;
and that the wrapper's own unattended merges
(the auto-push reconcile merge,
the commit replay merge) and the isolated merge use the same pass.

The merge is `src/native/merge/merge_second_pass_git_lines.rs`
(Git's line merge and the word pass over the regions it leaves),
`src/native/merge/merge_second_pass_line_regions.rs`
(that answer cut into the text Git settled and the regions it left)
and `src/native/merge/word_merge.rs`
(the word pass):
the line merge is Git's own,
`git merge-file --stdout --object-id --zdiff3` over the three stages,
whose exit status is the number of conflicts;
the line merge is a read:
it writes nothing
(`src/native/transaction/transaction_git_step_rerun_and_end_rules.rs`,
`merge_file_reads_only`),
so the wrapper may run it again after a refusal and end it at the ceiling of its class,
and its whole answer is read rather than the tail a step that is never ended keeps;
the regions it leaves are then diffed by word
and by the bytes between words (the `similar` crate,
Myers algorithm,
no other feature of it),
and kept only when every part of
the region resolves;
a file is resolved only when every region resolves.
The conflict style is the decision here,
not the diff algorithm:
`zdiff3` keeps the lines both sides wrote identically out of a conflict
(`xdl_refine_conflicts` and `XDL_MERGE_ZEALOUS_DIFF3` in Git 2.56.0's `xdiff/xmerge.c`),
so the word pass sees only the lines that are a choice.
Measured on 723 files that both sides of a two-parent merge changed and that plain line merging conflicts on,
from 3,194 merges of this history (`doc/planning/cli-git-merge-auto-resolution.md`,
"Measured again on 2026-11-10"):
this resolved 130 of them where the pass's own line merge resolved 108 under the same refusals,
with the same refusals and the same 21 lines no side holds,
and Git's `patience`,
`histogram` and `minimal` algorithms resolved 130 and 129 of them.
The word level is still held to the stored results of Jujutsu 0.46.0 on a corpus of real and measured inputs
(`src/native/merge/word_merge_corpus/`):
the same verdict on every input and the same bytes on every resolved one,
and on those 41 inputs Git's `--zdiff3` line merge gives the same verdict and the same bytes as the pass's own line merge.
A region is refused rather than merged when its answer would be more than a resolution of two edits;
see the refusal below.

A `--zdiff3` answer is cut into the text Git settled and the regions it left,
and a region's line numbers for the report and the journal are found by searching its text forward from the end
of the region before it.
The lines above each region are counted from the place the region before it reached,
never from the top of its stage,
so each stage's lines are counted once:
while every part counted its stage from the top,
the pass over a hundred and three thousand one-line regions against stages of 7.6 MB took about seven hundred seconds
of a push that took 889 seconds,
whose merge-tree took about 190
(measured 2026-11-11 on this host with the release build).
The markers of that answer carry labels made unguessable for every merge
(`-L ours-<random> -L base-<random> -L theirs-<random>`):
Git does not escape the markers it writes,
so a block the file already held that looks exactly like a conflict would otherwise be taken for a region
and replaced by the word pass.
A marker of a file whose lines end with a carriage return and a line feed ends with both bytes,
as Git writes that file's markers,
so both endings are a marker's own and such a file is still merged.
Those three labels do not reach the split marker,
which Git writes as seven equals signs and a line end with no room for a label,
so a line of the person's file that is exactly that cannot be told from Git's own by its bytes.
A region is therefore cut with its split-shaped lines still unchosen,
and its split is chosen from them:
behind the common ancestor marker only one such line is Git's,
because Git writes one split per region,
and where a region holds more than one behind that marker,
the split is the only one whose three sides are found in their own stages,
at or after the place the region before it ended.
A region no single split explains stays exactly as Git left it,
as does a region that holds no text on any side.
The searches are bounded at 256 candidate lines per file,
which is about six hundred passes over the stages at the largest size the pass takes,
so a file whose answer holds more split-shaped lines than that inside its regions is left as well.
An answer that ends inside a region,
that nests a region inside another,
or whose region count is other than the conflict count Git reported is refused,
and its file stays exactly as Git left it:
a region read wrongly would lose a side.
Git's exit status stops counting at 127,
so at that count the answer may hold more regions than Git counted and never fewer,
and a shortfall is refused as any other difference is.

Owner decision (2026-11-10),
verbatim:
"there are still ways you haven't tried to make this happen less often,
for example,
using a better algorithm or something."
That followed merge `b971df44f5`,
which carried a record two sides had written,
and merge `ef43afe9f9`,
which shipped `return return without_variable == *selected;`
in `package/git-policy/cli/src/native/policy/policy_forbidden_strings.rs`.
So the pass keeps the decision of 2026-11-03 and refuses a region whose word merge would write more than
a resolution of two edits:
a word written twice in a row that none of the three texts writes twice in a row,
and,
in a region that spans more than one line on some side,
a line that none of the three texts holds
(`refuses_word_merge` in `src/native/merge/word_merge_invented_text.rs`).
A refused region keeps both sides,
so its file stays exactly as Git left it and the person resolves it.
A resolved file therefore holds only lines one of the three texts holds,
or the merge of a line all three held,
which is the one shape the owner accepted on 2026-11-03.
On the measured set the refusals took both triples that wrote a doubled word
and the 71 lines no side holds that 34 resolved files had joined across lines,
leaving the 21 lines that are each the merge of a single line all three held.
The controls are the three stages of merge `ef43afe9f9`
and a reduced replica of the two blocks merge `31f72ecf12` left in
`doc/handover/questions-for-the-owner.md`,
in `src/native/merge/word_merge_refusals/`;
that replica is the shape the pass declines,
and its file stayed a conflict in the merge,
so the text merge `b971df44f5` carried was written by whoever resolved that conflict by hand.

It can produce text neither side wrote (two edits of different arguments of one call become one call with both),
which the owner accepted;
it never drops a line both sides kept,
and it never orders two insertions at one point,
so both sides appending different text at one place stays a conflict.

A `.gitattributes` that a merge left conflicted is no rule at all:
the worktree copy holds Git's markers and a copy in a merge tree the same,
so a rule read from between them is neither side's rule,
and every candidate of that merge keeps its conflict
(`src/native/merge/merge_second_pass_worktree.rs`,
`src/native/merge/merge_second_pass_tree.rs`).

A conflicted file is a candidate only when all three stages exist with one mode that is a plain file,
none holds a NUL byte in its
first 8000 bytes (Git's `FIRST_FEW_BYTES`,
`xdiff-interface.c` line 197),
none is larger than 8 MiB,
and the `merge` attribute is unspecified,
set or `text`
(a driver,
`-merge` and `merge=binary` keep the merge to their setting,
read from the worktree for a merge a person runs and from the tree the merge produced for a merge from objects).
Git's own limit is `MAX_XDIFF_SIZE`,
1023 MiB (`xdiff-interface.h` line 14),
above which its merge calls a file binary
(`merge-ll.c` lines 117 to 122);
the pass takes the smaller of that and 8 MiB,
which is measured:
texts that share no line took
0.08 s and 45 MB at 1 MiB,
0.73 s and 352 MB at 8 MiB,
4.12 s and 1320 MB at 32 MiB,
so Git's limit would not bound the cost.
Anything else is left exactly as Git left it,
with the reason in the debug log.

Merges a person runs (`merge`,
`rebase`,
`cherry-pick`,
`revert` and `pull` forwarded to Git):
a conflicted merge computed from objects with another session's staged work gets the pass from objects
 (`merge_isolation.rs`,
`merge_tree`);
with a clean index the conflict is native Git's,
which stops and writes the markers,
 and the pass then runs on the
stopped merge here,
because the ending commit then goes through the wrapper's commit path and its policies (the control
`the_ending_commit_is_judged_by_the_repositorys_policies` pins that for the stopped path;
 whether a merge completed from objects is
judged by the same policies was not measured).
The index is read in process before Git starts,
and again when Git exits with a status other than 0.
Only the paths that were not unmerged before the command and are unmerged after it are candidates,
so a file a person is
resolving by hand is never replaced by a later command that fails on it.
The stages are read with one `git cat-file --batch`,
the attributes with one `git check-attr`;
each file that resolves is written as a blob (`git hash-object -w`),
staged as resolved
(`git update-index --index-info`,
which replaces the conflict stages) and written to the worktree with `git checkout-index --force`,
so the repository's filters apply;
if the checkout fails the conflict stages are put back.
Standard error gets one line per part,
`monochromatic-git-policy-cli: word-level merge: <path>: part <n> of <m>: base <lines>, ours <lines>, theirs <lines> became "<text>" from base blob <oid>, ours blob <oid>, theirs blob <oid>`,
and one sentence for the reader:
the part lines end with the three blobs they came from,
and `git cat-file blob <oid>` shows one.
The spans locate the part in the three texts,
and the object names are the whole stage blobs those spans index into:
the pass replaces the stages when it stages the file as resolved,
so `git show :1:<path>`,
`:2:` and `:3:` answer nothing afterwards,
while the blobs stay readable,
which is the handle a person needs to check what the merge did,
for a merge a person finishes and for the merges from objects that leave no stage at all.
Each merged file is one journal entry of priority `warning` under `SYSLOG_IDENTIFIER` `monochromatic-git-policy-cli`,
with the fields
`CLI_GIT_WORD_MERGE_FILE`,
`CLI_GIT_WORD_MERGE_OPERATION`,
`CLI_GIT_WORD_MERGE_PARTS` and
`CLI_GIT_WORD_MERGE_BLOBS`
(`base=<oid>,ours=<oid>,theirs=<oid>`,
with `none` for a stage the merge had not).

The list ends with one line that names what the pass staged
(`monochromatic-git-policy-cli: word-level merge: staged as resolved by the second merge pass: <paths>. ...`),
whether or not a finish follows it:
the pass has already changed the index and the worktree,
so no later step can be the only account of that change.
A file the pass does not install moved while it computed the merge
(another command resolved it and
 `git add`ed it,
or a person edited it),
and one line names each such path:
what the worktree held at each of those paths is read the moment the command left them,
before the merge looks at any stage,
so an edit made while the merge runs is never the baseline it is compared with;
the bytes of each are what the comparison uses,
not their length and modification time,
because a writer can write different bytes of the same length and put the saved time back
(`src/native/merge/merge_second_pass_worktree_unchanged.rs`),
and a conflicted file is one the pass has already read:
measuring the read on the development host on 2026-11-09 took 1.56 ms for 16 MiB against 1.23 ms for 64 bytes,
medians of five runs each,
so the two reads of a 14 MiB conflicted file cost about 0.6 ms beside the 396 ms and 403 ms the whole merge step
of a 7 MiB conflict was measured at;
a file larger than three times the pass's 8 MiB stage limit is never compared by its bytes;
the index stages are read again just before the install,
and a path whose stages or bytes are not the ones the merge read keeps the version that is there now
and is never replaced by the pass.
Where every merged file moved,
nothing is installed and that line alone is printed.

Not being able to compare a path is not finding it unchanged:
a file past that limit,
one that cannot be read,
and one that is no plain file when opened
(a named pipe,
a device,
a directory,
a link)
are recorded as nothing to compare,
two nothings are never equal,
and such a path keeps what is there now,
with a line of its own naming it
(`src/native/merge/merge_second_pass_worktree_unchanged.rs`).
The type is judged on the open handle,
which is opened with `O_NONBLOCK` and `O_NOFOLLOW`
(`src/native/foundation/plain_file.rs`),
because opening a named pipe for reading waits for a writer that may never come,
and a writer that holds one open without writing moves that wait into the read,
which no byte limit bounds either.

Two windows are recorded here rather than closed,
each a few instructions wide,
and each needs another process to land inside it.
The first is between the pass reading the worktree for the last time and the step that installs:
a save written to the worktree file then is replaced by `git checkout-index --force`,
and a resolution another process stages then is replaced by `git update-index --index-info`.
The owner's instruction of 2026-11-09 is to record that window rather than close it with more mechanism
(`doc/planning/cli-git-conflict-resolution-route-findings-on-main.md`),
and what the person sees if they are in it is their resolution replaced:
the file holds the pass's merged bytes,
the line names the path among those staged as resolved,
`git diff --cached -- <path>` and the file itself show the merge rather than what they wrote,
and with every other conflicted file resolved the finish commits that merge.
Nothing else is lost:
the index stages,
the merge's own inputs and the commit are untouched.

The second window is between Git writing the conflict and the pass reading what the worktree holds:
the snapshot is taken when Git exits,
so an edit saved in between is what it records,
the comparison before the install matches that edit,
the pass stages its own merge over it,
and the line names that path among those staged as resolved rather than among those left alone.
The person therefore loses the resolution saved between Git stopping and the pass starting,
and sees the pass's bytes in the file and in the commit that follows,
with nothing said about what they wrote.
The worktree is read before the merge is computed,
so that no edit made during the pass's own work can become that baseline;
nothing inside the pass can tell an edit saved after Git wrote the conflict from the conflict Git wrote
(`resolve_after` in `src/native/merge/merge_second_pass_worktree.rs`).
When a finish is planned the line says the wrapper finishes the operation below;
otherwise it says what is left to do.
A finish that does not end the operation (a commit the commit service cannot be reached for,
a commit a policy refuses,
any other failure of the ending step)
is followed by one line naming the state the person will find,
read from the index and from the Git directory as it is after that step:
the operation did not finish,
the files the pass staged as resolved are named,
and either no file is conflicted or the files that are,
or that the index could not be read again and which files are conflicted now cannot be said,
with the sentence that Git stopped again there,
because a continuation commits the pick it resolved and then stops on the next change's conflict
(`git rebase --continue` does exactly that).
The command that ends the operation is read from the state files Git writes
(`rebase-merge`,
`rebase-apply`,
`CHERRY_PICK_HEAD`,
`REVERT_HEAD`,
`MERGE_HEAD`),
so a `git pull` that runs a rebase is told `git rebase --continue` and never `git commit`;
the operation's own word is only the fallback for a state file that is gone.
The failing step's own words describe that step alone,
and a commit's plain failure ends with "Nothing in the repository changed.",
which is true of the commit and false of the merge that staged the files before it;
the wrapper's line after it is the last word,
and it is the true one.

The merge stays in progress in that case (`MERGE_HEAD`,
`CHERRY_PICK_HEAD`,
`REVERT_HEAD` or the rebase state files as Git left them),
the resolved files stay staged,
and the exit status is the failing step's,
so a script that tests it still stops.

When every conflicted file of the command resolved,
the operation is finished the way Git finishes it after a clean run
(owner decision of 2026-11-03,
"Finish like a clean merge"),
through the wrapper's own command path:
the continuing step is
`run_process` of this process,
so the commit policies apply to it and a later pick of a rebase that conflicts gets the pass again
(nested no deeper than the budget the outermost finish takes from its input:
a rebase's remaining todo length plus one,
a
cherry-pick or revert sequence's todo length,
one for a merge;
past it the operation stays stopped).
The exit status and the output are the finishing command's;
what Git printed when it stopped (`CONFLICT` lines,
"Automatic merge
failed") is already out and stays.
What Git does after a clean run,
from the Git 2.56.0 source:
- `merge` and the merge of a `pull`:
  a clean merge that wants a commit commits (`builtin/merge.c` line 1828,
  `prepare_to_commit`
  line 905);
  with `--no-commit` or `--squash` it prints "Automatic merge went well;
  stopped before committing as requested" to
  standard error and exits 0 (lines 1867 to 1873).
  The wrapper does the same for those two options,
  and otherwise runs
  `git commit` through its own path with `--no-edit --cleanup=whitespace` (the prepared message as it is,
  with the conflict hint
  cut from `MERGE_MSG`,
  `append_conflicts_hint`,
  `sequencer.c` line 722) or,
  when the editor is wanted,
  plain `git commit`.
  The editor is wanted by `--edit`/`-e`,
  or by default when no `-m`/`-F` was given,
  `GIT_MERGE_AUTOEDIT` is unset and standard
  input and output are one terminal (`default_edit_option`,
  lines 1158 to 1180).
  `-S`,
  `--gpg-sign`,
  `--no-gpg-sign` and
  `--signoff` are handed to the commit.
  `--autostash` and `merge.autoStash` need nothing:
  the ending `git commit` applies
  `MERGE_AUTOSTASH` after it concludes the merge (`builtin/commit.c` line 1982),
  as a clean merge's `finish` does
  (`builtin/merge.c` line 1848).
  The wrapper's commit builds the commit in a shadow repository,
  which now holds a private copy of
  `MERGE_AUTOSTASH`;
  landing removes the real pseudoref when the shadow consumed it (`conclusion_cleanup.rs`).
  A stash whose apply conflicts when the commit concludes the merge (the finish's,
  or the person's own `git commit`)
  is stored by Git in the
  scratch repository's private `refs/stash`,
  with its conflict stages in the commit's private index.
  The landing carries both back
  (`autostash_carry.rs`):
  the stash is stored in the repository with `git stash store -m autostash` (once;
  the objects are
  already in the repository's store,
  because the scratch repository writes there),
  by `remove_conclusion_state` before it removes
  the pseudoref,
  and the stages are put into the index the landing installs,
  under the lock it holds (`follow_up` in
  `service_land.rs`).
  The finish therefore ends a conflicting autostash as Git does and declines nothing (the dry run of the apply that once
  declined it was retired).
  The controls `a_merge_autostash_that_conflicts_on_apply_keeps_the_work_as_git_keeps_it`,
  `a_pull_autostash_that_conflicts_on_apply_keeps_the_work_as_git_keeps_it`,
  `a_commit_that_applies_a_conflicting_autostash_keeps_the_stash_as_git_keeps_it` and
  `a_commit_over_unsaved_changes_to_a_stash_file_keeps_the_stash_as_git_keeps_it` compare the stash list,
  the status,
  the files and
  the exit status with a twin repository that real Git runs.
  A
  `branch.<name>.mergeoptions` value of the checked-out branch is split as Git splits it (`alias.c` lines 126 to 180) and read
  before the command line (`merge.c` lines 1407 to 1409),
  so `--no-commit`,
  `--squash`,
  `--edit` or `-m` set there is honoured.
  Hooks:
  the wrapper switches every hook event off in every Git it starts,
  so a user's hooks see nothing on a clean merge and
  nothing on a finished one (a control compares the two,
  with real Git run directly as the positive control).
  Were hooks run,
  a
  clean merge would run `pre-merge-commit`,
  `prepare-commit-msg`,
  `commit-msg` and `post-merge` (`merge.c` lines 931,
  966,
  975
  and 537),
  a concluding `git commit` `pre-commit`,
  `prepare-commit-msg`,
  `commit-msg` and `post-commit` (`builtin/commit.c`
  lines 780,
  1116,
  1133 and 1966),
  and a clean pick `prepare-commit-msg` and `post-commit` (`sequencer.c` lines 1367 and 1699);
  that difference would then need a decision.
- `cherry-pick` and `revert`:
  `git commit --no-edit --cleanup=strip`,
  which is what Git's own `--continue` runs for a conflicted
  pick (`continue_single_pick`,
  `sequencer.c` lines 5354 to 5378),
  edited when `--edit` is given or,
  for `revert`,
  when standard
  input is a terminal (`should_edit`,
  line 2217);
  then `<command> --continue` when a sequencer todo file exists,
  which picks the
  rest and ends the sequence (`sequencer_continue`,
  lines 5617 to 5630).
  `--no-commit` stops with the result staged and exit 0;
  a range applied with `--no-commit` is not finished (`error_dirty_index`),
  and neither is it here.
  `--continue` and `--skip`
  forms take their options from `sequencer/opts` (`save_opts`,
  line 3734).
- `rebase` and the rebase of a `pull`:
  `git rebase --continue`,
  which edits the message of the commit it makes after a conflict
  (`commit_staged_changes`,
  `sequencer.c` line 5381,
  flag `EDIT_MSG`),
  where a clean pick does not;
  the continuing step is run
  with `GIT_EDITOR=:` for its Git children (the in-process variable `CLI_GIT_CHILD_SET_GIT_EDITOR`,
  `child_environment.rs`),
  and a
  rebase whose remaining todo list holds `reword`,
  `squash` or `fixup -c`/`-C` is not finished,
  because the same setting would
  switch off the editor those steps want.
  The original author is kept because `--continue` reads `rebase-merge/author-script`
  (a plain `git commit` would not).
  `--autostash` and `rebase.autoStash` need nothing:
  the continuing step applies the autostash
  when the rebase ends (`sequencer.c` lines 4980 and 5330).
- `pull` runs `git merge` or `git rebase` as a child of Git (`builtin/pull.c` lines 525 and 750),
  out of the wrapper's reach,
  except
  for a pull that is stated to merge and read exactly,
  which the wrapper splits into its fetch
  and a `git merge FETCH_HEAD` of its own
  (see "Merges and commits never fail");
  for every other pull the wrapper sees the pull's exit and its state files:
  `rebase-merge` or `rebase-apply` means the rebase,
  `MERGE_HEAD` (or
  `--squash`) the merge.
  Pull's own options are read like merge's plus the fetch and rebase options it passes on,
  and the
  child merge applies `branch.<name>.mergeoptions` and `--autostash` as a merge does;
  the finish is not made when an option is
  unknown.
Options are read by exact spelling,
  since Git accepts unambiguous abbreviations;
  a command line with an option this reader does not know
leaves the operation stopped with the files staged and Git's exit status,
  as before the finish existed.
A file that does not resolve keeps the stop of Git for every operation.
A merge that Git completes costs one in-process read of the index before Git starts and nothing after it;
  the Git starts above happen only after a conflict,
and they run from the top of the worktree with the invocation's configuration options only
  (`-c`,
  `--config-env`,
  and `--attr-source`,
  each in the forms Git accepts),
  because `check-attr` and `checkout-index` read paths relative to the directory they run in,
  and because every one of those options can name the attribute tree the person chose:
  a pass that dropped one read attributes from another source and merged a file their rules exclude.

Merges from objects (the isolated merge,
the auto-push reconcile merge,
the commit replay merge):
`git merge-tree --write-tree -z` prints the stages of every conflicted path.
These merges accept the result only when every conflicted file resolves;
otherwise they behave as they did,
naming the paths.
The merged blobs are written,
a scratch index in the repository's `monochromatic-git-policy-cli-scratch` directory takes the merge tree and the merged blobs
(`read-tree`,
`update-index --index-info`,
`write-tree`),
and the scratch file is removed.
No worktree is read by these merges,
so the attribute tree is named where Git would read none.
Git takes the tree from
`--attr-source`,
else `GIT_ATTR_SOURCE`,
else the configuration `attr.tree`,
and ignores an `attr.tree` that is no tree
(`attr.c` lines 1202 to 1231).
A `-c attr.tree=<spec>` on the command line is a configuration value Git takes above every file,
in any letter case the key allows,
because a Git configuration key's letters are not significant and `canonical_key` is the wrapper's one spelling of them
(`git_config_from_parameters`),
so a resolving one there is the person's choice even where a file sets the key,
and one that is no tree is ignored as a configured one is;
`-c attr.tree` with no value sets `true`,
which no tree resolves to.
A `--config-env=attr.tree=<variable>` in either of its forms sets the same key,
with the value the environment the Git children get holds under that name,
read after the last `=` of the entry as Git splits it;
an empty value there is no value and is ignored as a configured one is,
and a variable the environment does not hold makes Git stop before it reads anything,
so no tree is named over that either.
`-cattr.tree=<spec>`,
with no space after `-c`,
is no form Git accepts,
so there is none to read.
The wrapper sets `GIT_ATTR_SOURCE` to the first side's tip (the isolated merge's `HEAD`,
the
reconcile merge's branch tip,
the landed tip of a replay) only when none of the three applies:
a person's choice is never
overridden.
`attr.tree` is read in process,
or from Git when the in-process readers decline the environment (a `GIT_*` variable
such as `GIT_NAMESPACE` makes them decline;
the variables that only trace,
word output,
reach a network or a tool,
or set a limit
do not,
each checked against Git 2.56.0 in `repo_read_layout.rs`),
which costs one Git start in that case.
The `doc/handover/*.md merge=union` line of `.gitattributes` therefore applies in them.
A `.gitattributes` that only the second side has is not read by `git merge-tree` itself (Git takes one tree),
but the second pass reads the `merge` attribute from the tree that merge produced,
so a rule the merge introduces governs the pass:
the candidates are asked of one `git check-attr --source=<tree>` start,
and a `--source` outranks `--attr-source` and `GIT_ATTR_SOURCE` in Git 2.56.0.
The tree is named as that source only where the person named no source above;
where a person's choice cannot be read,
the pass names none.
A `.gitattributes` that the merge itself left conflicted names no source at all:
its file in the merge tree holds Git's markers,
no side's rule can be read from between them,
and every candidate therefore keeps its conflict,
so the merge is refused as one that does not resolve.
A merge whose own `.gitattributes` is settled reads its rules there,
including one only the second side carries:
a path its settings exclude (a driver,
`-merge`,
`merge=binary`) is left exactly as Git left it.
Every Git command of the reconcile merge's second pass (`merge-tree`,
`cat-file --batch`,
`check-attr`,
`hash-object`,
the scratch index's `read-tree`,
`update-index` and `write-tree`)
runs under the push's stall window and,
beside it,
under a total ceiling,
300 seconds by default and settable with `CLI_GIT_PUSH_MERGE_CEILING_SECONDS`:
the window ends a command that shows no sign of work,
and only the ceiling ends one that keeps working without ever returning,
because a merge driver that rewrites a scratch file for ever shows progress in every sample.
The measured cost of the step is 396 ms and 403 ms for a merge of one 7 MiB file the pass resolves
(load average 66 to 75 on 8 CPUs),
and the pass takes no file past 8 MiB,
so the ceiling is hundreds of times the measured work.
When it fires,
the merge is not made and nothing was written:
the commit is kept,
no reference moved and the worktree was not touched,
the failure says so in those words,
and the service retries the push on its own,
naming that text in the notice on the person's next command.
The rest of the bound is the one a push query has:
the push's stall window,
the service's stop and the grace a stopped query gets,
with its request on the standard input and an answer ceiling of one stage per name it asks for
(`src/native/push/auto_push_reconcile_merge_commit.rs`),
and the line merge's answer ceiling is its content and its marker lines:
four stages for the three stages it is given and the text Git settled,
and eight stages for the marker lines,
which is about four hundred thousand regions at four marker lines each,
because a merge of three stages is larger than each of them and the report's cap alone refused merges
that resolve,
and because the number of regions is bounded by the lines of the stages and not by a margin of their bytes:
three stages of 7,605,777 bytes with a conflict every other line hold a hundred and three thousand regions
and print a 37,871,557-byte answer although its content is under three stages
(measured 2026-11-11 with Git 2.56.0;
cross-vendor review of round 198,
finding 2),
and the earlier four stages plus the push report's own cap was 122,821 bytes short of it.
An answer is held whole and then cut,
so a merge at this ceiling costs its own bytes twice,
about 190 MB,
plus the merged file and its parts,
under the 352 MB the pass's own word merge of three 8 MiB texts costs;
the ceiling is a memory backstop and not the bound of what eligible stages can print,
because marker bytes grow with the lines of the stages and not with their bytes,
so an answer larger than the ceiling is refused and leaves the path as Git left it,
for the person to resolve;
the total ceiling of 300 seconds,
not this one,
is what ends the slowest merges this one admits,
because `merge-tree` took 192 seconds on that fixture in one run,
under a 3-CPU scope on a host at load 66 to 75,
before the pass read any of it;
so a merge driver that never returns ends the merge and is named as a stopped query,
and the push worker,
its claim and the drain of that repository are not held for good.
The isolated merge prints the part lines before Git's own words and sends the journal entries after the merge is in the checkout;
the reconcile merge appends the part lines to the merge commit's message,
prints them with the push report and sends the entries after the push succeeded,
under `SYSLOG_IDENTIFIER` `monochromatic-git-policy-cli-service` for the auto-push,
which runs inside the service,
and `monochromatic-git-policy-cli` for a push a person typed;
the replay merge reports the parts in a `word-merged` event and sends the entries when the merge is used.

### Retry after a moved branch

A branch that moved since the wrapper read it makes the service answer `retry-prepare` with the commits that landed meanwhile.
A commit the wrapper built merges the change it captured onto the branch as it is
(`git merge-tree --write-tree`)
and runs the policies again over the merged tree;
no recorded policy run is reused.
Where a path changed in both,
the later capture wins when both captures read the same worktree
and every other path is settled by subsumption and then the three-way merge.
A path that Git's line merge still reports as a conflict after the subsumption rules is merged by word
(see "Word-level second merge pass");
when every conflicted file resolves,
the merged tree is used,
and a `word-merged` event lists each merged part with the commit's other events.
Paths that cannot be combined end the commit with the core finding `concurrent-commit/replay-conflict`,
exit `1`,
a message naming the paths and the commit that landed first,
and the message the person typed saved in a file with the command that reuses it.
A merged tree equal to the branch ends with `concurrent-commit/already-committed`.
A commit sent back to be prepared again reads the private index of the attempt that captured its change,
so that attempt's scratch tree belongs to the command until it has read it:
the service removes the tree of a request it answered as landed,
records the identity of every request it answered without landing it,
and leaves those trees where they are
(`handed_back` in `src/native/service/service_server_land_group.rs`,
`remove_the_trees` in `src/native/service/service_server.rs`).
The command releases such a tree beside itself once the answer's arm has read the change it captured
(`HandedBackTree` in `src/native/landing/service_prepare_ready_shapes.rs`),
so the recursive removal never makes the command wait,
and the removal starts only after that read,
never before it.
The tree is named after the request and the attempt,
so the next attempt never shares a directory with one a removal is taking,
and a tree whose removal never ran is collected by the age sweep of the repository scratch directory.
Before the capture reads the index it checks that the file is there:
`git write-tree` answers with the empty tree for an index that is gone,
which would rebuild the commit as a deletion of every file and report a conflict naming paths it never changed.
A commit Git built without an editor is built again on the new tip,
signed again by Git,
while the time since its first build is under the ceiling;
one built after an editor keeps the typed message in a file and is built again from it under the same ceiling.
A line whose editor ran keeps everything else it borrowed as well,
so a rebuild asks the terminal nothing and invents nothing
(`reissued_arguments` in `service_commit_line_reissued.rs`):
the message is read from the saved file (`-F`),
the authorship `-c`,
`-C`,
`--reuse-message` or `--reedit-message` took from another commit is read back from the commit Git built and passed as
`--author` and `--date`,
and a `--fixup` or `--squash` option goes because the message it built is in the file.
A selection the person made with `--patch`,
`--interactive` or `--include` is already in the private index,
so the rebuild keeps that tree and carries no selection option at all.
A `--fixup=reword:` commit lands on the branch as it is now,
carrying that branch's own tree:
Git throws the index away for that option,
so its commit is the branch's tree and the change this preparation captured is not the change it would make.
The commit is therefore written from the fields of the commit Git built
(`AmendedCommit`,
`stored_commit_fields` and `rebuilt_onto` in `service_amend_replayed.rs`)
on the tree the replay settles,
which is the branch's tree unchanged,
and its one parent is the branch as it is now.
`rewords_a_commit` in `service_commit_line_reissued.rs` tells the commit loop that an empty commit is what the line asked for,
so the attempt's preparation holds that empty change instead of ending as `already-committed`,
and the person's staged change stays staged,
as it does after Git's own commit for that option.
The person is told the landed commit's summary line and nothing more:
no message file is saved,
because running the same command makes that message again,
and the amend note is not printed because no amendment was made.
A line a rebuild cannot read back at all,
such as an abbreviated borrowing option the wrapper does not guess at,
is still refused with its message saved and the command to run again.
A commit prepared on a branch that did not exist yet has no parent it must keep:
the change it captured is a whole root tree,
and its base is the empty tree (`Captured::unborn` in `service_replay.rs`),
which the replay merges from and leaves out of the histories it reads,
so such a commit lands on the branch another session created in the window.
A commit that concludes a merge,
a cherry-pick or a revert depends on the state the worktree holds and not on the tip,
so it is built again the same way:
its parents come from `MERGE_HEAD`,
`CHERRY_PICK_HEAD` or `REVERT_HEAD`,
the tree is the replay above,
and the preparation refuses with `ConclusionStateChanged` when that state is no longer the one the commit was
prepared over,
because another command either finished the operation or abandoned it
(`conclusion_state_text` in `conclusion_copy.rs`).
No commit waits for another,
and no command is refused for another session's commit or merge
(the owner,
2026-11-09:
"I would consider this to be a defect in our wrapper.
Agents should never wait on each other on commits or merges.").
An amend whose branch moved is not made as an amend again:
it lands as a new commit on the branch as it is now
(see "The amend a moved branch makes").
The retry is bounded by time,
not by a count of builds:
`MOVED_BRANCH_REBUILD_CEILING` in `src/native/service/wait_ceilings.rs`,
which `CLI_GIT_REBUILD_CEILING_SECONDS` sets for a control,
and every rebuild says how many builds were made,
how long it has been,
and when the rebuilding stops.
A branch that keeps changing past the ceiling ends the commit with a plain message naming the branch
and the number of attempts it made;
the attempts of one landing are `LANDING_ATTEMPTS` in `src/native/landing/service_land.rs`.

### Recovery

The service keeps a journal of each batch it lands and settles an unfinished batch before its next batch;
a service's start settles only an earlier release's shared journal,
and the wrapper recovers nothing for a commit.
A batch whose branch does not show its commit is dropped,
unless a record of a hold names it:
a held batch is settled first
(`held_commit_settle.rs`),
and one whose lock still stands is left with `Handled::Held` and `Recovery::Hold`,
because dropping it would take a commit the person made and kept.
A held batch whose settlement finished every obligation of it
is reported as `Handled::Finished` with `Recovery::Finish`,
so the person who asked what became of it reads that it finished,
in the service's log and in what `service settle-earlier-batches` prints;
a batch reported nowhere would leave that answer empty after the worktree was changed.
Otherwise it is dropped:
its commit is written to `refs/cli-git/kept/<batch>` before the intent goes
(`keep_what_the_branch_does_not_show` in `src/native/landing/service_land_finish.rs`),
so the commit of a branch write that a stop left running,
including one Git built,
which has no scratch branch,
reaches a ref the wrapper names instead of an unnamed one,
or none at all.
The kept write runs only on this drop,
never during a landing,
and it is a side-ref write,
so the stop's own bounds cut it as they cut every other Git of a landing.
It writes a kept reference only where nothing is kept under that number yet:
a kept reference that already names another commit is left as it is,
and so is the batch's scratch reference,
whose removal waits for the keep
(`service_scratch_ref_of_a_dropped_batch.rs`).
A drop whose kept write fails is logged with Git's words and the commit's name,
and it still drops:
the drop is what releases the worktree's index lock and forgets the batch,
so a ref that cannot be written holds up neither the next commit nor the stop.
What the wrapper still settles at the start of a command is state other code writes:
an isolated merge whose process died,
and it stops on a commit journal of the previous executable's format
(`.git/cli-git-transaction`)
with the way to recover it.
A merge that settling finishes says so on standard error,
naming the ref it landed on and the merge commit,
because the command may have been run only to look at the worktree;
the same line names the ref a merge was abandoned on when the branch moved while it was prepared.
Settling writes the index only after deciding what another program's `index.lock` allows,
so a program that holds the index does not end every command in that worktree.
A merge that already installed part of its checkout waits for the lock,
bounded and naming the holder,
and a lock that outlasts `MERGE_LOCK_CEILING` writes nothing:
the record and the checkout stay as they were,
the command is stopped and told the lock,
the record and what is installed,
because a command that went on would read the half-installed merge as another session's staged work.
A merge that installed nothing is settled from one look at the lock instead
(`merge_isolation_recovery_index_lock.rs`):
nothing of it is in the checkout,
so a command says where the merge is saved and goes on at once,
and the next command finishes the merge once the lock is free,
because the owner's rule is "nothing should ever wait forever or indefinitely".
A lock whose Git lock PID file names a process that no longer runs is cleared on the commit service's proof of disuse,
exactly as the merge's own path clears it,
and the merge goes on rather than every command exiting 2 until the person deletes the file.
A staged entry for a path the merge changes is judged before the take and written after it,
so another command that stages one of those paths inside that window replaces a version this command never read:
the standings are read once,
every file the merge writes is taken aside,
and only then does `git update-index --index-info` write the merge's entries,
which replace whatever entry is there.
Closing it needs a write that lands only while the entry it replaces is the one that was read,
which `--index-info` does not offer;
the window is the length of the take and one `git update-index` call.
The person would see the merge land,
the version they had staged in that window gone from the index with the merge's content in its place,
and nothing in the output saying which content was overwritten.
A path is judged against the two sides of its change by the object its bytes hash to and by the mode Git records,
and the old side wins a tie,
so a change that alters neither is not evidence that a merge began.
The mode is read as part of the file because Git records it in the index entry:
a file whose bytes are the branch's and whose executable bit is the person's is the person's,
and the merge stops and names the path rather than erase the bit.
A repository whose `core.fileMode` is false records no such bit,
so no mode difference is read there,
as Git reads none;
a key Git does not answer for is read as true.
Settling takes every file it must write aside by one rename that never replaces,
and checks what it took against the device,
inode,
size,
mode and modification time the judgement read,
and then against the bytes:
the taken file is hashed with the attributes of the path it belongs to and must be the object the judgement read,
so the bytes that make way for the merge are exactly the bytes that were judged.
A program that keeps its descriptor across the rename can write the same length into the file and put its modification time back,
which leaves every part of the identity as it was;
the bytes are what decides there.
a file saved in between is put back,
nothing of the merge is written,
the paths are named on standard error and the record is kept.
The files are checked out without `--force`,
so a file that appears in the instant after the take is skipped and Git's own words are reported.
The carry after the merge writes a path only while the file standing there is still the one Git's own update wrote,
whose entry the apply reads as soon as the branch has moved:
the device,
the inode,
the size,
the mode and the change time are compared,
so a save whose bytes accept the whole merge,
and which would otherwise be read as the merge's own file,
stands and is named as a path that changed under the merge.
That entry is read once,
right after Git's update,
and a save that lands in the instant between Git's own write and that read is read as Git's own:
no evidence this command can read tells the two apart,
because the save leaves the same bytes in the same entry,
only newer.
The carry then merges the person's older layer into the file,
so both sides stand in it and the path is named;
nothing either side wrote is discarded,
and the window is that one read wide.
A command that stops between a take and the write leaves the file under a private name beside its path,
`.cli-git-recovery-<name-key>-<merge-key>-<pid>-<count>-<nanos>`,
where each key is a 64-bit FNV-1a hash in 16 hex digits,
of the file's name and of the merge's transaction id,
so a name at the file system's component limit still has a private name that can exist;
the bytes the carry writes wait under the same shape with `.cli-git-carry` in place of `.cli-git-recovery`,
so the command that finishes the merge can tell whose temporary a name is,
and the settle of a merge removes the names its own transaction left,
because those bytes are made again by the next carry and are in the object store either way;
the next command puts such a file back before it judges the path and says on standard error that it did,
and where the path holds something else keeps both,
names the private file,
and records the copy so a later command ends it.
A file written into a taken one after the take checked it is kept rather than removed,
and its private name is said.
The private name is removed only on a proof that nothing can write into the file any more,
asked in this order:
a scan of every process this command can read finds none with the file open,
which is the proof the commit service's lock removal acts on,
and then,
as the last step before the unlink,
the file's **raw bytes** still hash to the branch's own file form for that path,
which is the judged object with the path's clean filter applied and those bytes hashed with no filter.
A clean filter can hide bytes a person saved from the object Git records,
so the filtered object is not the identity of an aside:
the raw bytes are,
and a file that carries bytes the branch's form does not have is kept with its private name and named.
A symbolic link's object is the text of its target,
so a taken link is hashed from that text and the file it points at is never read.
A file whose raw bytes are not the branch's own form is not taken either,
so the merge stops and names the path rather than replace it.
A writer that still holds the file is seen by the scan,
and a writer that let go has already written,
so the bytes then show it;
a write that arrives after both would have to open the private name,
which nothing outside this command knows.
The guarantee rests on that,
not on a check made just before the unlink.
That look and the unlink are not one step:
the raw bytes are read,
the branch's form for the path is hashed after them,
and only then is the private name removed,
so a program that opens that name between the read and the unlink writes bytes the comparison never saw,
and the unlink removes them.
Closing it needs a removal that proves nothing can write into the file,
which no unlink offers;
the window is the length of those two reads.
The person would see a program's write into a file that was set aside vanish,
with the merge's content in the checkout and nothing in the output saying that anything was removed.
The limit is that removal's:
a process in another user namespace,
or one that cannot be read,
is not seen,
so a file such a process holds open is removed with its private name.
A scan that was cut at its bound,
or could not list the processes,
proves nothing and the file is kept.
A private name carries the key of the merge record that made it,
and only a name that a record still there names is put back:
a name nothing claims is what a merge that already ended left behind,
and putting it back would bring back a file the person deleted.
A name an earlier release wrote carries no such key,
so it is left where it is and named,
with the path it belongs to,
and the person moves it back;
the record it belongs to then finishes as it would with the path gone,
and the bytes it holds are neither moved nor removed.
A take whose read of the bytes fails after the rename puts that file back before it reports anything,
and a file that cannot go back is named and counted as installed;
a take or a write that fails puts every file it already took back,
and a path the merge deletes is never removed again after its take,
so a file that appeared in that instant stays;
a copy a private name holds is decided by the bytes it carries,
never by its name alone:
a private name this release wrote carries the key of the merge that took it,
so a record that is still there is the only claim that may put it back,
and a name an earlier release wrote carries no key at all,
so its bytes decide,
a copy whose raw bytes are the branch's own file form for the path,
the merge's own file form,
or the person's captured file being the file the merge judged there,
which a merge that has not finished puts back when the path is empty,
while a copy holding anything else stays beside its path,
because a merge that ended leaves a name whose file the person may have deleted themselves;
a copy a merge keeps is recorded inside the worktree's Git directory,
one file per copy under `monochromatic-git-policy-cli-kept-copies/`,
naming the copy and the path it belongs to,
and every command that runs the recovery decides every recorded copy again,
so a copy no later merge touches is not a file no command ever ends:
a copy whose merge still runs is left for that merge to finish,
because a command beside a live merge that put the copy back under it would make Git refuse the checkout the
merge was part way through making,
and a copy whose merge key names a record that was written after the command read the owners is left too:
the records are read again beside the copy it decides,
so a merge that started in the moment between the two
keeps the file it took instead of meeting it again under the path it cleared,
a copy whose path is empty and whose merge stopped without finishing goes back,
a copy whose bytes the file at its path holds again,
and whose permissions that file has too,
is removed only on the
proof its own removal asks for:
the identity and the bytes it read are still its own,
no process this command can read holds it open,
and the file at its path is still the entry the comparison read,
holding those bytes with those permissions,
because a save or a chmod that arrived after the comparison would leave the copy the last name of the older
bytes or the older mode,
a copy whose bytes that file holds but whose permissions it does not keeps its bytes and is named with that reason,
where the permissions are every bit a person can set,
the setuid,
setgid and sticky bits above the read,
write and execute bits of each of the owner,
the group and
everyone,
because Git records only the execute ones and a copy that is the only entry carrying a special bit is
not the second file of those bytes,
and so does a copy a program still holds open,
a copy whose bytes the branch's own commit holds for that path is removed with that commit named and with the
`git restore` command that reaches those bytes,
and any other copy stays,
is named on standard error with `mv` and `rm` as the two commands that end it,
with both paths in them quoted the way a shell reads a word back
(`crate::shell_quote`,
the one owner of that spelling,
`commands_that_end_the_copy` in `src/native/merge/merge_isolation_carry_writes.rs`),
and is removed the moment the file at its path holds its bytes again;
a copy whose bytes nothing else holds is the person's alone,
and no command decides it for them;
the emptied directories of every path the merge deletes are pruned on each attempt,
whether or not the path already stands at its end state,
because a kill between finishing a child's private name and removing the directory it emptied leaves an empty directory
that `checkout-index` would refuse.
A branch another program moved away from the merge is read from `git merge-base --is-ancestor`'s own status and never from
what it printed,
so a "no" is the answer that the merge is not in the branch:
the paths go back to the moved head with the person's captured work,
and no command is refused for it.
The settling of a branch that moved reads the checkout and the index the same way the completion does,
so a directory the branch's own files explain is read the same way there,
and a staged entry for a path the merge changes that is neither the merge's nor the moved head's stops it,
naming the paths and keeping the staged content.
A checkout holding the merge's file for one path and the branch's content for the others is a writing that a kill
stopped between two files,
so it is put back to the moved head like every other path of a begun merge,
and the line naming the moved branch is true of the checkout it leaves.
Whether the branch was built on the merge is `git merge-base --is-ancestor`'s exit status:
`0` is yes,
`1` is no,
and any other exit,
and a command that never started,
is a read that failed.
The status is read and never the failure's wording,
because a `1` and a start that failed are worded alike
(`is_ancestor_awaiting` in `src/native/merge/merge_isolation_recovery.rs`,
the same reading as `ancestry_of` in `src/native/push/auto_push_reconcile_basics_and_limits.rs`),
so a branch another command reset away from the merge is settled as moved rather than refused as partly installed.
A read that fails keeps the record,
and the next command decides from a reading it makes itself.
A directory that stands where the merge puts a file is the branch's own when the merge deletes every file below it:
nothing is set aside for it,
the merge's own deletions empty it before the checkout,
and the merge's file takes its place.
That judgement is made through every directory below the path,
because a tree diff names the files the merge deletes and no directory of its own;
an emptied directory below the path is the branch's only where the merge deletes a path under it,
which is where the pruning of those deletions reaches,
and any other emptied directory stops the finish before anything is written.
A directory that no such deletion explains is someone else's and stops the finish,
naming the path,
and so does a directory that holds any file the merge does not delete,
such as an untracked file of the person's:
it is refused before anything is written,
because replacing it would leave the merge half installed and the file in the way.
A record that is discarded,
a record whose owner still runs and a branch that already names the merge
change nothing of the branch and say nothing.
A merge saved on a side ref that cannot be finished now never stops a command while nothing of it has been installed:
the line says where the merge is and what blocked it,
and the command goes on.
A merge saved on a side ref whose paths hold the person's own edits is installed once the lock is free:
nothing of it is in the index or the checkout,
and the capture the record keeps holds their bytes and their mode,
so the merge takes each file aside,
writes its own content,
and the carry puts their work back over it.
A file the capture does not hold,
because it was saved again after the merge was saved,
is not written over at all:
the record goes,
the merge's commit stays on its ref,
and the line names that ref and the `git merge` command that makes the merge on the bytes the file holds now,
where a recorded ref another command repointed is not that promise:
the commit is kept on a name of the
command's own,
written with an all-zero old value so no other command's commit is replaced,
and the line names
that name too,
because a commit whose record and ref are both gone is collected by `git gc`
(`keep_the_merge_on_a_side_ref_awaiting` in `src/native/merge/merge_isolation_recovery_cleanup.rs`).
A record that names no side ref and has nothing of the merge in the checkout does not stop a command either:
another branch's checkout has nothing for it to finish and nothing of it was saved,
so the record goes and the merge is run again on the branch it is for.
Once its index has been written or a file of it moved aside it is a begun merge,
so a refusal stops the command and says the merge is partly installed in that worktree;
that is decided by what the index and the checkout hold when the refusal is made,
not by which step refused,
so a foreign file or a staged entry met after the install began stops the command too.
What Git's own update left at each path the merge writes is read once,
after the checkout moved and before the carry,
and written beside the record as `<transaction id>.landed`,
under a private name that is renamed onto its own,
so a kill inside the write leaves no reading rather than half of one;
the recovery of a merge whose branch already names it judges each path against that reading rather than against its bytes
alone,
so a save that arrives after a kill and holds the merge's own bytes is a different entry from the one Git wrote,
stands,
and is named.
The reading is not flushed:
a kill leaves the page cache intact and a power loss leaves it absent;
the reading goes with the record it belongs to,
and the record is removed before it,
so a kill inside that removal leaves a reading no command reads rather than a record whose reading is gone.
A path the reading does not name is one nothing proves Git's update wrote:
where the file there holds exactly the merge's own bytes,
which is also what a save made after a kill holds,
the merge finishes and the file stays as it is,
the bytes the carry would have written are kept beside it under a private name,
and that copy is recorded and named.
A path whose entry is neither a regular file nor a symbolic link is never read for its bytes:
a pipe has none,
and asking Git to hash one waits for a writer that may never come,
so such a path proves nothing and every copy beside it stays.
A merge carried onto a head the merge was built on reads what this command's own settling wrote,
never the reading the merge published,
because the settling wrote those files itself and the published entries stand no longer:
a captured path is judged by the entry this command's write left,
and a file written in the instant after that read stands and is named.
The line a recovery writes about a path it carried says which merge it is finishing,
never a repository ref of the branch it lands on.
The line about a path the merge writes and the person's side of which holds nothing says that nothing stood there when
the merge captured it,
and never that the person deleted it:
a file a command set aside before the capture leaves the same state,
and the line about that copy names it.
The branch is resolved before anything else,
so a merge whose branch already names the merge commit is settled from that alone and its leftover record goes without a
worktree read that may have become impossible.
What they hold is otherwise read once,
before anything can refuse,
the index first and the worktree second:
every refusal after that point reports from that one reading,
an index that cannot be read counts as holding the merge,
a worktree that cannot be read is reported by what the index holds,
and a merge whose own commits,
refs or branch cannot be read counts as holding it too,
because a refusal then cannot prove the checkout is untouched.
That reading has a window of its own:
the index is read before the worktree,
so another command can stage one of the merge's paths between them,
and a worktree read that then fails is judged by the entries the index held a moment before.
The index then holds the merge's entries for some paths and not for every one,
which is not the answer that stops the command,
so a saved merge's refusal is let past:
the person would see this command go on and `git diff --cached` show the merge's entries as ordinary staged work,
with the line naming what blocked the finish and nothing saying that a merge's entries are among the staged ones.
Closing it needs one read of the index and the worktree together,
which no pair of Git commands offers;
the record is kept,
so the first command that meets a readable worktree settles the merge.
A merge counts as installed when one of its paths both holds its side in the index or the files and is a path the branch does not already hold that side for.
That is the only evidence an install leaves:
an independent commit of the same content gives the branch that content,
so the path stops being evidence,
while an install leaves the branch as it was,
whatever another command does to the checkout afterwards,
and a merge that another command touched after the install still counts as installed.
A record is kept for the next command when anything of the merge is present at all,
so a half-installed merge is never only discarded.
A begun merge's refusal still stops it,
because the checkout may hold that merge's files.
Standard output is never touched by these lines,
because scripts read it.
A field of bytes that is present and does not decode is damage,
never absence.
An intent whose `repositoryHex`,
`commonHex`,
`gitdirHex`,
`indexHex`,
`cleanupHex` or `pathsHex` does not decode is no intent,
and a restart drops the batch as it drops any damaged record
instead of reading the plain field beside it,
which names another index,
another worktree or another list of paths.
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

A drain claims every pending record of the kinds it serves in one listing,
and renews those claims while it runs them (`service_claim_renewal.rs`).
The order of the kinds within one round is therefore the order in which a claimed record ends,
and the kind that runs last waits claimed for as long as the kinds before it run.
A step of the slow kinds can take minutes:
emptying one trash of a removed worktree took 18 minutes on 2026-11-10
(`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
The review of a landed commit therefore runs before the steps that free space,
because it is bounded by its own time limit (`REVIEW_CEILING`),
it decides nothing about any push,
and its entry in the review log is the whole of what it produces;
measured in the main checkout on 2026-11-11,
two reviews claimed at `10:24:46-05:00` were renewed with the trashes of one round until `10:39`,
while the trashes ahead of them were emptied.

The service takes a queued item by renaming its file into `claimed`,
and the file name carries the clock reading of the holder's last renewal
(`src/native/landing/claim_time.rs`,
`src/native/landing/work_item.rs`).
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
or not 32 lowercase hexadecimal digits that are not all zero (a fresh image holds `uninitialized`) counts as no machine ID.
The boot ID alone cannot say the holder is dead,
because boot IDs are random per kernel boot and two hosts that drain one work directory on a shared filesystem
have unrelated ones:
each would otherwise take the other's live claims at once.
The wrapper supports one host per repository,
and the owner answered a request to support a repository two hosts share (2026-11-10) with
"Not supported (Recommended)":
a checkout used from two machines is not protected.
These readings tell a restarted host from a running one and close no race between two,
and a command of another host is an owner of another PID namespace,
which the patience of `FOREIGN_OWNER_PATIENCE` judges by age (`src/native/process/pid_namespace.rs`).

Why the lease and its renewal stay:
a service that is killed leaves its claims in `claimed`,
and the file name is the only record of who held them and since when,
so a lease is how a later instance knows a claim was abandoned without a process identity or a lock file.
Several instances drain one directory in two cases:
the common directory of a bare repository,
whose linked worktrees each run an instance (`drain_common_records`),
and a service started by hand or by a downgrade beside the unit's own,
which the manager's one main process per unit does not allow.
The renewal that keeps a live holder's claim from looking abandoned is in-process state shared by two threads,
and it takes no lock:
the renewal thread alone knows the names claims have now and renames them,
the drain thread tells it of each claim over a channel and settles by finding the claim's file by its id and generation
(a renewal changes the reading in the name and nothing else),
asking again when a renewal renamed the file between the look and the rename,
and the two share only a flag per claim for taken over and settled.
Neither side waits for the other.

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
(kind `maintenance`,
a 64-digit object name,
a 32-digit generation,
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

A slow worktree add is split by step from `<common-git-dir>/monochromatic-git-policy-cli-worktree-add-timing.jsonl`,
written the same way as the commit
timing file below (one append,
no flush to disk,
size-capped by the same appender,
never changing the add's result).
Code:
`src/native/worktree/add_timing.rs`.
Each line has `schemaVersion`,
`time`,
`startMillis`,
`pid`,
`release`,
`exitCode`,
`totalMs`,
`steps`,
`otherMs`,
`work` and `notes`.
`steps` maps a step name to `{ms, calls}`;
steps are disjoint (a step inside another is taken out of the outer one) and
`otherMs` is what no step covers,
so the steps and `otherMs` add up to `totalMs`.
The steps are,
in order:
`recover`,
`chooseRung` (its parts `refusals`,
`checkSource`,
`claimStage`,
and the Git runs inside,
named
`git <subcommand>`),
`gitAdd`,
`finishRung` (its parts `bindRegistration`,
`recordBinding`,
`prepareTarget`,
`takeSnapshot` with
`snapshotAhead` (its part `snapshotAheadProof`),
`snapshotAheadWait` and
`snapshotIoctl`,
`readOwnIndex`,
`prune`,
`removeStubs`,
`replacePointer`,
`walkSnapshot`,
`installIndexFile`,
then `applyDifference`
or `restoreDirty`,
`exchange`,
`settleDisplaced`,
`markComplete` with `markCompleteFile` and
`markCompleteDirectory`,
`releaseClaim`),
`copyIgnored`,
`handOverTrash` and `repair`.
`work` sums time spent on worker threads by name over all threads (`cargoRepairWorkspace`,
`cargoCloneTree`,
`cargoClean`,
`cargoMetadataWorktree`,
`cargoMetadataSource`,
`repairPnpm`,
`repairToolchain`,
`repairCargo`,
`readConfig`,
`trashIgnored`,
`untrackedScan`);
threads overlap,
so `work` is not part of the sum rule,
and a name whose sum exceeds the step that waited for it
is work that ran side by side.
`notes` names the rung taken (`route`),
the cause when the add was not a snapshot (`slowRoute`),
and how the snapshot was found (`snapshotAhead`:
`claimed`,
`waited`,
`timed-out`,
`given-up`,
`not-for-a-source-with-changes` or
`no-service`).

The completion marker (`markCompleteFile`) is written by rename without a flush to disk,
by the owner's decision of
2026-11-03 ("Drop the fsync (Recommended)").
The rename and the subvolume swap before it land in one btrfs transaction,
so a crash keeps both or neither:
never a half-made
worktree,
and nothing of the user's is lost.
A flush here made the add durable before the next periodic commit,
but because the swap renamed a subvolume,
btrfs turned that
one flush into a full filesystem commit that waited on every other writer (Linux `fs/btrfs/inode.c` rename-exchange sets the
log-full-commit flag;
`fs/btrfs/file.c` then commits),
measured at 1.2 to 18 seconds per add.
A power loss within one periodic commit of an add can therefore undo that add;
the next invocation finds no marker and prints a
notice,
and running the add again recreates it.

The add's three other records follow the same rule,
by the coordinator's decision of 2026-11-10 and its measurement here on 2026-11-11:
the stage claim,
the add's destination record and the binding written into that record are published whole by one rename each and
flushed by nobody
(`write_atomic_unsynced`,
`worktree_copy_stage_claim.rs`).
They were the first waits of every add:
five concurrent adds of 24 packages took 29,487 to 35,574 ms with 9,282 to 13,903 ms in the destination record alone,
and one `fsync` of the claim's temporary file took 11,239 ms,
because a flush there is a commit of the whole filesystem and waited behind every other writer.
A kill keeps all three in the page cache,
so a kill loses nothing of them.
A power loss can take them back,
and what it takes is the report of an interrupted add,
the name of a stage nothing else removes,
and the identities recovery would need to remove objects it otherwise keeps and reports:
nothing of the person's,
which is the direction the rule above calls safe.
The directory of the journal root is flushed by the sweep that changed it and never by one that only looked:
an add that found other adds' live claims removed nothing and paid 4,600 to 13,900 ms of its `recover` step for the
flush before this rule.

An add that is still running after `ADD_STANDING_NOTICE_AFTER` (five seconds) says which step it is in,
and again at `ADD_STANDING_NOTICE_REPEAT` (fifteen seconds) while it stays in one step,
on a thread of its own that ends with the add
(`src/native/worktree/add_standing_notice.rs`).
The owner,
2026-11-03:
"60s is fine for adding worktrees",
with "an add that is still waiting after a few seconds says what it waits for",
and the waits it names are the timing record's own steps,
each said as the work it is.
The line goes to the standard error,
is advisory only,
and nothing waits for it:
a stream nobody drains holds that one thread and never the add.
`CLI_GIT_ADD_NOTICE_SECONDS` moves the wait,
and a setting that is missing,
is not a number or is zero leaves the shipped one.

A slow commit is split by phase from logs that exist without verbose mode.
Neither log reaches the person's terminal,
neither starts a process or flushes to disk,
and neither holds a message,
an identity or a secret.

The wrapper appends one JSON line per commit that reached the service to `<common-git-dir>/monochromatic-git-policy-cli-commit-timing.jsonl`
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
`stop`
(why the commit did not land;
`null` for one that did,
because `outcome` and `exitCode` say that nothing landed and what the process returned and never whether a policy correctly refused content or a machine failed:
`kind` is `policy` for an error finding,
with `policy` naming the registry entry that reported,
`core-finding` for a fixed transform's rejection,
with `core` and `code` naming it,
`git-step` for a Git step that failed,
with `step` the program and the subcommand and `status` the status it exited with (absent when the step never exited by itself,
which is a start that was refused or a step ended at its ceiling),
`native-exit` for real Git declining the command,
with `step` and `status` as before,
`engine-failure` for the wrapper's own engine failing where no step explains the stop,
with `code` the engine failure's stable code,
and `unidentified` where no step,
policy,
transform or engine failure explains the stop,
which is where a route that stops a commit with words of its own (the branch kept moving,
the content was pruned,
the service was not there) lands,
and which is the signal the owner's rules call a defect that should log more;
`stop` is null for a command that ended without landing and without failing,
because nothing stopped it;
the failing step is the note the Git runner keeps (`src/native/commit/git_step_failure.rs`),
dropped as soon as another step starts,
and every field is a name the wrapper compiled in or a number,
never a message,
a path or a candidate's content),
`totalMs`,
the disjoint phases
`routeMs`
(what the command spent before its first preparation began:
finding the worktree's route to a service,
this commit's identity,
the standard input a pathspec file is read from,
and the client's settings),
`prepareMs`,
`buildMs`,
`provisionMs`,
`submitMs`,
`afterLandingMs`,
`gateMs`,
`handoffMs` and `otherMs`
(the remainder,
never negative),
`prepareSpans`
(what the preparation's own work did,
by name,
in the order each name was first noted,
empty for a commit that noted no step:
`name` is the step,
`calls` how often it ran,
`totalMs` the whole of its time and `longestMs` its longest run;
the names are `index stat check` for the in-process answer of whether the real index still records its worktree's stat data,
`index read` for the one reading of an index's bytes that every private copy of the commit is written from,
`index seed` for the walk of those bytes that decides which way an index may seed a private commit index,
`index equals base` for the in-process answer of whether the captured index already holds the base's tree,
`index copy` for a copy of an index into a private one,
`policy blob write` for storing a correction's bytes,
`policy entry write` for pointing a private index entry at a corrected blob,
and `candidates prepared` for the candidate set a pass's content policies share,
which the first content policy that asks pays for,
`policy <name> at <lifecycle point>` for every policy a pass evaluated,
with that policy's registry name and the trigger whose pass ran it,
summed over the convergence rounds that ran it again,
so a preparation's own pass is told from the post-commit gate's;
a step that starts Git is stated here as the work and in `steps` as the child it started),
`prepareSpansDropped`
(how many step names came after that list reached its bound of 64,
so a longer run is counted rather than lost),
the two costs measured wherever they fell
(`scratchRemovalMs`,
`removalsThatDidNotFinish` and
`stderrMs`:
time spent removing scratch trees,
how many of those removals ended without finishing within their ceiling,
and time spent writing to standard error,
which waits for whoever reads it;
each of them lies inside whichever phase it fell in,
and inside `otherMs` when no phase covers it),
and `prepareParts`
(the preparation's stretches by name,
`{"layout":{"ms":12,"calls":1},...}`,
each figure the time the stretch spent without a Git child,
because the steps are recorded apart,
a name marked again by a retry added together with `calls` saying how many stretches the sum is made of,
at most 64 entries with the rest counted in `preparePartsDropped`),
`buildParts`
(the build phase's stretches by name,
in the same shape as `prepareParts` and with the same rule,
its own list and its own bound stated in `buildPartsDropped`:
a build's name is never read as a preparation's),
`buildPartsDropped`
`indexBuilds`
(one entry per build of the private commit index,
in the order built:
`kind` is `first` for the command's own preparation and `retry` for a rebuild after the branch moved,
and `statDataKept` says whether that build started from a copy of the real index,
keeping the cached stat data of every entry its tree shares with it;
a build from the base or the tree alone holds none,
and the scratch commit then hashes every file of the work tree),
`scratchSteps`
(one entry per build of the scratch `git commit`,
empty for a commit that ran none,
in the order the builds ended:
`ceilingMs` the ceiling that build ran under,
`null` for a commit that may open an editor,
`armed` whether anything enforced it,
which is a notice thread that ran,
the wait that owns the step when no thread could name it,
and false for a commit that may open an editor,
`stoppedAfterMs` how long the step had lasted when the ceiling ended it,
recorded only when a signal was really taken for the step,
`heldAfterCeilingMs` how long the step had lasted when its ceiling stood aside for a step the kernel held in a disk wait,
`null` when no ceiling stood aside,
`stallCeilingMs` the bound the step then ran under (`SCRATCH_COMMIT_STEP_STALL_CEILING`),
`null` when no ceiling stood aside,
`noticeMs` how long the notice's own end took after the step ended,
`wroteMs` how long the writer spent on the lines it finished,
`linesDropped` how many lines it dropped because a reader stopped taking them,
`looks` and `looksFound` how many looks it made for the step and how many found it,
`below` one entry per program that ran below the step with the milliseconds the same process number,
start time and name were seen at the look interval,
kept with a zero bound when a look saw it only once,
`belowCpuMs` the processor time everything below the step had used at the last look,
`belowUnlisted` whether the programs below the step could not be listed when it was ended,
so one of them may still be running,
`belowUnended` how many programs below the step were not ended because the kernel refused a descriptor for one of them
or refused the signal to it,
so each of those may still be running and the stop's words say so,
`blindLooks` how many looks could not be made at all because `/proc` refused the read,
and `blockedLooks` how many looks found one of them in uninterruptible sleep;
an entry whose `armed` is true and whose `looks` is zero is a notice thread that did not answer within its bound;
the record's own `git commit` step figure is the child alone,
so `noticeMs` and `below` are never inside it),
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
The record carries two accounts of the preparation,
and they answer different questions:
`prepareSpans` names the work a site in the wrapper measured around itself,
`prepareParts` and `buildParts` name the stretches no Git child covered,
and a Git child that a span started is inside that span and outside every part.
The build phase's names are `branch`,
`message`,
`amend tree`,
`fitting request`,
`request`,
`shadow`,
`shadow skeleton`,
`shadow config`,
`shadow links`,
`shadow conclusion`,
`shadow refs`,
`client build`,
`commit read back` and `amend fields`,
and a build that stops inside itself closes the phase where the command stops,
so its time is stated by `buildMs` and never left to `otherMs`.
Durations come from the monotonic clock in whole milliseconds,
and the start stamp from the wall clock.
The file moves to the next free generation (`monochromatic-git-policy-cli-commit-timing.jsonl.1`,
then `.2`,
and on) at one mebibyte,
and a failed write never changes the command's result.
All five logs (commit timing,
worktree-add timing,
push trigger,
audit and the standards-review shadow log) are written by one appender:
one `write` per line or entry,
and no lock.
It moves the file aside only when the path still names the file it opened,
so two writers that both see a full file cannot take the new small file in place of the history,
and a file that ends without a newline (a write a full disk or a signal cut short) gets the next line on a fresh line,
so one torn line is lost alone.
A reader skips empty lines and lines that are not JSON.
Every log keeps its rotated files under the next free number (`.1`,
`.2`,
and on;
the highest is the newest),
claimed with a hard link,
which fails when the name is taken,
so two writers never take one number,
nothing is replaced and no lock is held;
a name that is not a regular file ends the numbering with an error rather than counting up for ever.
The generations of one log may take `LOG_HISTORY_BYTES` (32 MiB) together,
the live file aside,
and the oldest go first,
never the newest.
A reader of the standards-review log reads the log and every generation of it,
so a commit whose entry rotated away is not reviewed a second time.
The owner answered on 2026-11-10 in the question tool,
"Keep more,
capped (Recommended)":
32 MiB per log,
and an audit line naming a saved copy that still exists is never dropped.
The bound is six days of commit timing at the pace measured on 2026-11-10
(1,143,526 bytes of 229 records in 5.4 hours)
and the whole history of an ordinary audit log
(782,685 bytes in 1,619 lines over twenty days).
The audit log's generations are the only ones whose lines name something outside the log,
so before the pass removes one it reads it,
and a generation holding a `saved-conflict` or `saved-nested-repository` line whose copy place exists stays past the bound;
so does one that cannot be read,
one with either of those lines that cannot be parsed,
and one that names a place the line could not show as text,
because a wrong answer in that direction costs disk and the other costs the only account of a copy.
A file left at a private name by a killed rotation is counted and kept,
and a second name left for a generation by a kill between the link and the removal is removed.
The reading that decides whether a generation pins happens on a thread of its own,
never on the path of the command that rotated the log,
because it was measured at a median of 369 ms and a 99th percentile of 432 ms where every generation
pinned,
against 385 us and 580 us for the rotation that schedules it
(`foundation/log_rotation_cost_measurement_tests.rs`).
The audit measurement,
the derivation of the 32 MiB and the choice
(a pick the coordinator made under the owner's delegation,
open to the owner's veto)
are in `doc/planning/cli-git-shipped-limits-audit.md`.

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
`SYSLOG_IDENTIFIER=monochromatic-git-policy-cli-service`,
`CLI_GIT_AT_MS`,
`CLI_GIT_RELEASE`,
`CLI_GIT_INSTANCE`
and,
where the line has them,
`CLI_GIT_REQUEST_ID`,
`CLI_GIT_BRANCH`,
`CLI_GIT_OUTCOME`,
`CLI_GIT_BATCH`,
`CLI_GIT_OID` and `CLI_GIT_REPOSITORY`.
A line about a batch,
a landing,
a push or a held commit names the repository it is about,
as `in the repository at <worktree root>` in prose and as `repository=<worktree root>` in a line of fields,
beside the batch or the branch and before anything about the outcome:
one journal covers every worktree of the machine,
including the fixtures of the tests,
so a line without a repository is read as one about the repository its reader is in
(`src/native/service/service_timing.rs`,
`in_the_repository_at`).
A value is carried as written:
one that holds a newline goes in the protocol's length-prefixed form
(`NAME`,
a newline,
the byte length as eight little-endian bytes,
the value,
a newline),
so it starts no field,
and other control bytes are kept.
The line on standard error stays one line,
with every control character replaced by a space.
A field name is at most 64 characters of upper-case letters,
digits and underscores,
and starts with a letter:
this is systemd's `journal_field_valid` (`src/libsystemd/sd-journal/journal-file.c`),
which refuses a digit first,
a leading underscore (the journal's own fields) and a longer name,
and journald drops such a field without a word,
so the datagram is built without it.
The socket is unconnected and non-blocking,
no lock is taken on that path,
and a line the socket does not take (the journal restarting,
a full queue) is written to standard error,
prefixed `<priority>` when standard error is the journal's stream.
That write never waits:
on a socket it is `MSG_DONTWAIT`,
on a pipe or file a test or a redirection put there it happens only after a poll shows room,
in pieces of at most a page.
A line the stream took only in part is kept:
its rest is written first,
before any other line,
so two lines never run together.
A line that finds no room is dropped and counted;
the next entry that gets through says how many were dropped,
as `CLI_GIT_LOG_DROPPED` on a native entry or as a warning line on the stream.
Dropped lines are lost,
not spooled:
a lost line is better than a stuck command.
A partial line still unfinished when the process ends stays unfinished in the stream.
The signal handler sends its line the same way,
with `sendto`,
and writes the fallback line to standard error through the same non-waiting send,
using only `send`,
`poll` and `write`,
which `signal-safety(7)` lists:
a standard error that is a full pipe or stream loses the line,
and the service still ends by the signal.
Without `$JOURNAL_STREAM` (a terminal,
a file,
a test) the lines are plain on standard error.
Every line ends with `at_ms=<epoch milliseconds>`,
and the secrets of any URL in it are replaced by `<credentials>`.
The lines are:
a `started` line before `ready`,
an `answered` line for every request with the phases of its stay
(`read_ms`,
`queued_ms`,
`settle_ms`,
`lock_wait_ms` and the holder when one is named,
the split of the lock wait into the making of the marker (`lock_stage_ms`) and the wait for a holder (`lock_place_ms`),
and when the lock was found held `lock_looks`,
`lock_found` and `lock_last`,
`reads_ms`,
`objects_ms`,
`journal_ms`,
`ref_ms`,
`index_update_ms` (named `sync_ms` in records written before 2026-11-04),
`lock_hold_ms` (how long the landing held the index,
not a phase),
`queued_behind` (how many requests were still waiting for their answer when the landing finished,
the batch's own taken out),
`reads_config_ms`,
`reads_branch_ms`,
`reads_parents_ms`,
`reads_changed_ms`,
`reads_content_ms`,
`reads_witness_ms`,
`reads_conclusion_ms`,
`reads_owed_ms` and
`reads_follow_ms`
(parts of `reads_ms`,
whose sum they never exceed),
`syncs_waited` and
`syncs_queued`
(how many of the landing's syncs it waited for and how many it handed to the background task),
`sync_held_ms`
(the time it spent waiting for its own),
`syncs` (each sync as `what=<ms>` or `what=queued`,
in the order asked),
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

### Paths of changed files that are not text

A commit's changed paths are the bytes Git lists.
`git diff-index --cached -r --raw -z` is read as bytes (`changes_of` in
`src/native/landing/service_prepare_changes_config_and_pathspec_steps.rs`),
a `Change` holds a `PathBuf` (`service_protocol_messages_and_name_checks.rs`),
and the listing is never decoded as text,
so one file whose name is not text no longer sends the whole commit to native Git.

Every path in a protocol message or a line-oriented record carries two fields
(`path_bytes_in_records.rs`):
the plain field,
which is the readable spelling of the path with U+FFFD for each byte that is not text,
and the field named with the `Hex` suffix,
which is the path's bytes in lowercase hexadecimal.
A reader prefers the bytes field,
so a name that is not text survives whole;
a record with no bytes field is one an earlier release wrote,
and the plain field is the path.
A bytes field that is present and does not decode is damage and no path:
the reader refuses the record rather than naming what the plain field spells,
because that is a different name.

The commit request's changes hold `path`,
and `pathHex` only when the plain field cannot spell the name,
so a request of `MAX_CHANGES` changes stays inside `MAX_LINE_BYTES` at the path length that bound was measured for;
the batch intent holds `paths=` and `pathsHex=`,
so a restart builds the index from the same names the batch committed
(`IndexPaths::Named` in `src/native/landing/service_land_follow.rs`).
The `fast-import` stream quotes each name from its bytes,
writing every byte below 0x20,
a quote,
a backslash and 0x7f as the octal escape `fast-import` reads
(`quoted` in `src/native/landing/service_land_git_steps_and_import_stream.rs`),
and the checks for paths that changed or that two requests share compare bytes.
The readable plain field is used only where a path is a key for something else:
the compiled-rules warm-up after a landing.

A request whose line would not fit `MAX_LINE_BYTES` is not sent.
The wrapper measures the line the client would write (`commit_request_fits` in
`src/native/service/service_protocol_encode_messages_to_lines.rs`)
and,
when it does not fit,
builds the commit itself and sends it whole,
which is the route such a commit took before a changed path carried its bytes
(`service_commit.rs`).
Nothing is printed about the choice:
the words are the ones that route already says.
The ceiling itself does not move,
so `MAX_CHANGES` changes of a text name at the measured path length still fit one line,
and only a name that is not text pays for its bytes.

The commit request speaks protocol version 11,
the first whose landing answers `held`
when a reference lock keeps a commit off its branch:
the answer carries the commit and the words the command prints,
and a client of version 10 reads a message type it does not know as a line it cannot read,
so a service of that version retires before a request is sent.
Version 10 was the first whose landing answers `reference-locked`
when a reference lock refused its branch write,
which version 11 keeps for the one case a commit still ends on:
a hold whose record cannot be written.
Version 9 was the first that names the commit's scratch tree in a field of its own.
A service of version 8 works the tree out from the request's index,
which is the worktree's real index rather than a tree,
so it lands the commit and leaves the tree to the age sweep.
The version is what keeps a service built before a field from being served,
not the release text:
`reach_current` (`src/native/service/service_client_reach_service.rs`)
serves only a service whose welcome names its own version and its own release,
and asks any other to retire before a request is sent.
Two builds of one crate version name the same release,
so a service built before a change to the request would pass the release check.
Version 8 was the first that carries a changed path's bytes;
a version 7 service takes the readable spelling of a name that is not text
and reports a successful commit of another file.

### Compatibility and degradation

The executable is tested with real Git 2.56.0,
the release the gate image's distribution package carried on 2026-11-04,
and `MINIMUM_GIT_VERSION` in `src/native/invocation/git_version.rs` names it.
It asks Git for its version once per invocation before commits,
hooks and locks,
and on an older Git it prints one warning to standard error
that names the real Git it found,
the version Git reported,
the version the wrapper was tested with,
and that the command continues,
then runs the command as usual
(owner decision,
2026-11-04:
"Raise minimum,
drop the test,
merely print a warning on older git and not refuse.").
Only a version that cannot be read,
or a Git that cannot start,
stops the command with an `engine-failure` event with code `content-unavailable` and exit `2`,
whose message names the real Git and the remedy (point the wrapper at a working Git,
 then run the command again).
Read-only commands are never checked.
The constant moves with the test image's Git:
when the gate image's `git` package moves to a newer release,
the constant is raised to it with the first gate run on that release.
Code written while the minimum was 2.55.0 assumed no feature introduced after 2.55.0,
and no test now runs on an older Git,
so behavior on one is untested.

The retired TypeScript wrapper had no version gate and degraded one feature at a time on Git 2.39.5 and 2.40.0:
a probe of `git merge-tree --write-tree --merge-base` (fail fast with `concurrent-commit/head-moved` without it),
the `core.lockfilePid` key of Git 2.54.0,
 and config-based hooks of Git 2.54.0.
That design and the observations of 2026-09-26 are in this section at commit
`f4fc1fa213ece65672e4314983a75266837892f9`.

#### An answer read in process is only ever Git's

A question the wrapper answers in process (`repo_read_*.rs`) counts as answered only when every stream is Git's:
standard output,
standard error and the exit code.
Any shape whose streams Git would word differently is a decline,
and real Git answers it,
so an answer stays Git's byte for byte rather than nearly.
Two shapes turn on this rule,
and each is measured against real Git 2.56.0 in
the differential controls of `repo_read_query_tests.rs` and `repo_read_query_refs_tests.rs`:

- An index carrying an optional extension the readers know nothing about.
  Git prints `ignoring <signature> extension` on standard error and answers,
  so the reader declines instead of answering in silence.
  `TREE`,
  `UNTR` and `EOIE` are read without a word on any body they are handed,
  so those answer,
  and `REUC`,
  `FSMN` and `IEOT` answer only when their bodies read the way Git reads them,
  because Git words a body it cannot read for those three.
  An `IEOT` table is read when `index.threads` turns threaded loading on and a valid
  end-of-entries (`EOIE`) extension is present:
  `read_ieot_extension` (`read-cache.c`,
  Git 2.56.0) prints `error: invalid IEOT version <n>`
  for a version other than 1,
  and it then seeks each block by the table's own byte offsets,
  so a table naming other bytes is a different listing or none at all
  (measured on Git 2.56.0,
  a first block one byte off ends Git without a word,
  and a first block pointing into the header prints `fatal: unknown index entry format 0xc0110000`).
  The reader therefore answers an `IEOT` body only when it is version 1,
  names at least one block,
  and every block begins where this reader parsed an entry and together they cover all of them;
  in a version 4 index every block must also begin an entry that carries its whole pathname,
  because Git reads a block with no preceding entry,
  so a block that starts on a compressed pathname would give that entry another name
  (measured on Git 2.56.0 with a hand-built index whose second block begins on the entry written as the suffix `z`
  of `a/z`:
  `index.threads=3` lists `a/a` and `z`,
  where `index.threads=1` lists `a/a` and `a/z`);
  any other body is a decline,
  so Git answers it.
  An unknown lower-case signature is a required extension,
  which Git stops on and the reader already declined.
- A ref name Git itself would refuse.
  The reader takes the names Git's own rule takes (`refname_disposition` and
  `check_refname_component` in `refs.c`):
  every printable ASCII byte but space and `*`,
  `:`,
  `?`,
  `[`,
  `\`,
  `^` and `~`,
  with `..`,
  `@{`,
  a component led by `.`,
  a component ending in `.lock` and a whole name ending in `.` refused.
  `@`,
  `+`,
  `{` and `}` are therefore answered,
  which is what this repository's packed `refs/tags/@monochromatic-dev/...` tags need,
  and the bytes above ASCII,
  the control bytes and DEL are kept as declines,
  because the reader cannot prove Git's bytes for them.
  A listing that holds a name Git refuses declines whole:
  Git words such a name on standard error (`ignoring ref with broken name`) and prints the other refs without it.

#### A ref listing past a measured count is left to Git

`for-each-ref` is the one in-process answer whose cost grows with the repository rather than
sitting in a Git start:
every ref it prints is checked against the object database,
because Git drops a ref whose object is missing (`warning: ignoring broken ref`)
and the answer must drop it too.
Measured on 2026-11-09 (release profile,
arms alternating in one binary,
this repository with 1442 refs and four packs of 514k objects):
the answer took 13.7 ms at the median against 3.8 ms for `git for-each-ref`,
and 10.8 ms of it was that check.
Of those 10.8 ms,
6.6 ms were the loose path lookup for every packed object;
the packs are asked before the loose paths now,
which leaves 4.3 ms for the check and 7.1 ms for the answer.
The layout read and the ref walk alone take 1.8 ms,
and with the measured 3.5 us per ref the answer meets the Git start near 490 refs and loses
above them.
`REF_LISTING_LIMIT` (256) is set well below that meeting point,
because a larger object database makes each check dearer while the Git start stays a constant.
A repository above the limit declines before the object database is opened or `refs/` is walked,
and pays the layout read (0.5 ms) and the count of `packed-refs` before Git answers,
which is the price of the capability rather than of the answer.
Below the limit the answer is kept:
a two-ref repository answers in 0.30 ms against 3.8 ms for the Git start.

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
so its files carry the occurrence:
`service-prepared-<n>.reached` and `service-prepared-<n>.release`),
and,
of an isolated merge:
`merge-carried` (the capture and the record are durable and the checkout is untouched),
`merge-intent` (record durable and Git not yet started),
`merge-clear-index` and `merge-clear-file`
(the index records of the clear are written and none of its worktree files yet;
then each worktree file of the clear,
numbered by occurrence as `merge-clear-file-<n>`),
`merge-cleared`,
`merge-applied` (branch moved and record not yet removed),
`merge-carry-index` and `merge-carry-file`
(the same two instants for the carry back),
and `merge-carried-back`.
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
A guarded form that runs under the hatch leaves one `hatch` line instead,
and a form that is only recorded (`git worktree remove`,
`git branch -d`/`-D`) leaves the same `logged` line it leaves without the hatch,
because the owner asked for that line and the hatch skips the policy that writes it.
Escape-looking option values and pathspecs remain ordinary Git arguments.
The legacy fixed-rule implementations no longer participate in dispatch.

### Hints

A finding,
an engine failure and a warn-unsafe notice may carry an additive `hint` string field after `message`.
The hint guesses what the person was trying to do and names the next step;
it never changes `message`,
`code` or `fix`.
Every finding code of a policy whose message has no alternative of its own,
and every engine failure code,
has a hint,
and the test suite fails when one is added without it.
Plain stderr notices that report a divergence from real Git's behavior end with a hint in the same voice.
A reader that ignores unknown fields is unaffected.

### Discarding commands

`linked-worktree-only` guards every command form that can overwrite or delete uncommitted work:
`stash` (except `list`,
`show` and `create`),
`clean` without a dry run,
`reset` in the hard,
merge,
keep,
mixed and default modes,
`restore`,
`checkout` with paths or a force option,
`checkout` and `switch` that move `HEAD`,
`switch` with a force or discard option,
`rm` and `mv` with force,
`checkout-index` with force,
every `read-tree` form Git accepts,
and the abort forms of `merge`,
`rebase`,
`cherry-pick`,
`revert` and `am`.
Read-tree is read from its own option table,
because every form Git accepts writes the index and,
with `-u`,
the worktree:
`git read-tree -m -u HEAD` reverts a staged edit and deletes a staged new file from the worktree,
and a plain `git read-tree HEAD` or `git read-tree --empty` replaces the staged entries.
The two forms that write nothing pass:
`-n` or `--dry-run`,
and `--debug-unpack`,
which returns before Git writes the index (`read-tree.c`).
A region Git refuses,
like `-u` without `-m`,
`--reset` or `--prefix`,
is the refusal and passes.
The word this reading reads is the word Git will run.
An ordinary alias (`alias.<name>`,
either spelling) is expanded the way Git expands it,
up to 16 expansions,
and the guard decides on the expansion,
which is also the argument list the wrapper forwards and the audit record names.
One `git config --null --list` answers which value Git keeps:
`config_alias_cb` overwrites what it kept on every matching variable,
so when a repository holds both `alias.<name>` and `[alias "<name>"] command`,
the last one in config order runs,
and a plain name matches whatever its case while a subsection matches byte for byte.
A matching variable with no value is Git's own `missing value` refusal,
so the wrapper leaves that alias to Git and Git prints it.
The value is split as `split_cmdline` splits it,
on space,
TAB,
LF and CR only (Git's `GIT_SPACE`),
so a vertical tab or a form feed stays inside one word,
and a value that ends with one of those four characters keeps the empty last word Git gives it,
which is what makes `git -c 'alias.x=status ' x` fail as `git status ''` does.
Two aliases are left unexpanded and forwarded as the caller typed them,
because the expansion the wrapper read is not the command that runs:
one whose value moves the directory or sets an environment variable,
which Git refuses with `changes environment variables` and runs nothing for
(`-C` with a directory,
`--git-dir`,
`--work-tree`,
`--namespace`,
`--bare`,
`--shallow-file`,
`--attr-source`,
`--literal-pathspecs` and its family,
`--no-optional-locks`,
`--no-advice`,
`-P`),
and one whose name an installed `git-<name>` program carries,
which Git runs before it looks any alias up.
Whether a program carries the name is read from Git's own search:
the directory the caller's last `--exec-path=` names,
else a nonempty `GIT_EXEC_PATH`,
else the path Git reports for itself,
and then every `PATH` entry.
A relative value in either of the first two,
and every `PATH` entry without a leading separator,
is read from the directory Git runs in after the caller's ordered `-C` chain,
which is why that chain is walked rather than guessed;
an empty `--exec-path=` adds no directory,
as Git adds none.
The `git` a shell alias starts searches from another directory:
Git changes to the top of the working tree before it starts that shell
(`git.c:388` to `git.c:407`),
so a relative entry names there the directory the shell searches,
not the one the caller's `-C` reached.
A repository Git found by walking up to it is the case that changes directory;
with no repository to walk to,
and with one `--git-dir` or `GIT_DIR` named,
Git changes to nothing and the shell searches where `-C` left it.
The reading tells the two apart by the prefix Git reports for the directory it was started in,
which it writes only in the case that changes directory
(`alias_run_directory` in `src/native/invocation/external_git_command.rs`).
An answer the reading cannot prove leaves the alias to Git,
which then decides.
The endings that reach that answer are starts of Git that fail for the exec path question,
and the alias lookup is another start of the same Git with the same globals,
environment and directory,
so it fails with them and there is no alias to read.
A failure that hits only the first start can still leave an alias to Git,
and that is this reading's remaining limit.
No fact of the caller then describes a repository the command never runs in,
and the audit names what Git runs.
A second window needs another process:
those relative entries are made absolute from a pathname the reading captures once,
before any probe,
rather than read again from where the process is when the probe runs,
so a program that renames that directory within the few instructions between the capture and the probe
and puts an executable `git-<word>` where the stale pathname still reaches
is read as carrying the name,
while Git,
whose working directory is the renamed one,
expands the alias instead
(`is_executable_file` in `src/native/invocation/external_git_command.rs`,
reached with the pathname captured in `resolve_forwarded_command`).
The person then sees the command run through the alias,
not the installed program the wrapper answered with:
`git -c 'alias.wipe=reset --hard' wipe` prints `HEAD is now at <commit>`
and the uncommitted changes to their tracked files are gone,
with no guard notice and no refusal.
No command renames a working directory inside those few instructions,
so the window takes a second program working against the same path.
A built-in word is decided without reading any configuration,
so no process is started for it.
The expansion is resolved once per invocation and read by all three of them:
the guard,
the fixed argument transforms and the argument list forwarded to Git,
so the tokens a transform adds land on the command Git runs
(`git go` with `alias.go=checkout tracks` is guarded and transformed as `git checkout tracks`,
and the wrapper forwards that expansion).
A built-in word is decided without reading any configuration,
so no process is started for it.
A shell alias (`!...`) and an undefined name are forwarded as they are:
a shell command cannot be judged by reading it,
and Git runs it with its own `git-core` directory first on `PATH`,
so a `git` inside it is real Git and not this wrapper,
which a control measures (`alias.fixup=!command -v git` prints Git's own `git`,
not the wrapper first on `PATH`).
One plain form of it is read,
only so the guard can decide:
a value written `!git <words>` whose bytes hold no quote,
backslash,
`$`,
backtick,
glob,
brace or separator,
and whose words name no global option of their own.
Git appends the caller's own words to the value before the shell reads it
(`git.c:401` to `git.c:402`),
so the command followed is the value's words and then the caller's:
a value of exactly `!git` starts that command with the caller's first word,
and `!git` with `reset --hard` after it is refused where `git reset --hard` is refused.
The Git inside the shell also reads the configuration and the exec path the caller passed,
which Git exports to what it starts
(`config.c:469` to `config.c:504`,
`exec-cmd.c:303` to `exec-cmd.c:310`),
so `-c 'alias.outer=!git inner' -c 'alias.inner=reset --hard' outer` is decided on the inner reset.
`!git reset --hard` is then refused where `git reset --hard` is refused,
and the alias itself is still forwarded unchanged,
because the shell keeps the `GIT_DIR` and `GIT_PREFIX` Git gives it,
which a substituted argument list would lose.
Everything else,
`!sh -c 'git reset --hard'` and `!git -C <directory> reset --hard` included,
runs as Git runs it:
through real Git,
with no guard and no index-writer coordination,
because the wrapper never sees that command start.
`git by the word Git will run` is the boundary of this guard.

A form is rejected in the main worktree and outside any worktree always,
and in a linked worktree while `git status --porcelain=v1 --untracked-files=no` is non-empty.
A `clean` counts untracked files as well.
An ignored file is not a reason to refuse a clean in a linked worktree,
by the owner's answer of 2026-11-10 ("All four are deleted,
exit 0,
and the audit log records the command",
after the same owner's "Log only" for `git worktree remove`,
whose reason was that a refusal only moves the same loss to `rm -rf`):
`git clean -x` counts tracked and untracked files,
and `git clean -X`,
which deletes ignored files and nothing else,
asks nothing at all,
runs,
and leaves one `allowed` line whose `dirty` is `null`,
because the policy asked no question.
The main checkout still refuses both,
a tracked change or an untracked file still refuses the clean that deletes it,
and `git clean -fd` keeps the ignored files as it always did.
A `git clean -X` that carries its own exclusion patterns (`-e`,
`--exclude`,
in every spelling Git's table accepts,
clustered and abbreviated included) is measured by Git itself instead:
Git adds those patterns to the ignore rules the clean applies,
so a file the status reports as untracked is one this clean deletes and the status cannot say so.
The question is two `git clean --dry-run` runs over the same facts,
rebuilt from them and never edited from the typed region
(`-n`,
`-f` as often as written and at least once,
`-d` when written,
the `-X` or `-x` mode,
each `-e` pattern,
then `--` and the paths),
the second one without the `-e` patterns,
because an edited region would carry `-i`,
whose dry run reads the terminal,
or `-q`,
which would silence it;
a dry run that prints nothing deletes nothing,
and a dry run Git refuses,
either of the two,
is a `fact-unavailable` finding.
Both answers name a path the same way,
so they are compared as Git printed them:
only a path the first answer names and the second does not is one the patterns alone call ignored,
and that path is an untracked file this clean would lose,
so it is still refused.
A path both answers name is one the worktree itself ignores,
which is no reason to refuse anything any more.
A `-x` clean needs no such reading:
it deletes every untracked file whether a pattern calls it ignored or not,
and a plain `-f` clean only loses files from that set when a pattern moves them out of it,
which the status measurement over-reports rather than under-reports.
A clean in a linked worktree that deletes ignored files with two or more `-f` keeps every nested Git repository first.
Git removes such a repository whole (`builtin/clean.c`,
`if (force > 1) rm_flags = 0`),
with its own history and its uncommitted files,
and the outer worktree reports it as one ignored directory,
so nothing else the policy reads can see the work inside it.
One force never removes a nested repository,
so a clean with one force asks nothing about them.
Git names the paths itself:
the typed clean's `--dry-run` names each path Git would remove,
and every one of those names is read as a path,
because Git writes the trailing slash of a directory it names at the top level and leaves it off one it names from
inside a directory that survives (`remove_dirs` prints both),
so the spelling says nothing.
A name that is a file holds no repository below it.
Each removed path is then read for the directories below it that hold a `.git` entry,
whether that entry is a directory or a file,
which is what Git itself counts as a repository (`is_nonbare_repository_dir`).
The wrapper walks only the paths Git named,
never the worktree,
and it opens no directory below a repository it found,
because moving that one takes everything inside it.
Git's one-force answer is no list of the same repositories and is not read:
at one force the directory walk skips every nested repository itself (`DIR_SKIP_NESTED_GIT`),
without a word,
so a repository below an untracked directory was deleted unsaved while that answer named nothing,
and `msg_would_skip_git_dir` reaches the answer only for a repository inside a directory the answer already lists.
A path Git had to quote (a name holding a newline,
a quote or another byte it cannot print) is decoded
before anything reads it as a path,
because the quoted text is not the name.
A path that is not a plain relative path names nothing the wrapper may look inside,
a path reached through a symbolic link is removed as the link alone,
and a directory the wrapper cannot open stops the command,
because what it holds was not seen.
Only the outermost of two repositories is kept,
because moving a repository takes everything inside it.
Each repository is moved whole to
`<common git dir>/monochromatic-git-policy-cli/discarded/<stamp>/<path below the worktree>`,
the clean then runs as typed,
the person is told where each one is before Git starts,
and the audit line carries `outcome` `saved-nested-repository` with one `copies` entry per repository,
written `<path> => <place>`.
A rename keeps every byte and the repository's own `.git` without copying anything,
the place is the folder the age and size limits already prune,
and a generation whose line names a copy still on disk outlives those limits like a saved conflicted file's.
A move the system refuses stops the command and puts back every repository that save had already moved,
so the worktree is exactly as it was;
a put-back the system refuses in turn is the one exception,
and the rejection names that repository and the place it is,
because nothing else would.
The deliberate way to delete such a repository with the rest is the escape hatch,
which the rejection names.
An interactive clean is not moved at all:
its menu is answered after Git starts,
so nothing can ask the person before Git deletes,
and a repository moved before the menu is one the person never chose to delete,
while a person who then quits has a worktree a command that deleted nothing has changed.
Such a clean runs with `--no-force -f` written at the end of its own options when its menu would offer a nested repository,
which lands Git on the one force at which it keeps every nested repository itself,
the menu then offers exactly what the clean deletes,
and the notice names the repository that stayed,
the same clean without `-i` (which saves it whole first) and the escape hatch (which deletes it with the rest).
The pair is not written where the person asked for the worktree enforcement to be skipped.
A dry run that never answered,
which is one that ended without an exit status,
stops the command instead of reading as Git's refusal of the clean,
because a refusal deletes nothing while a read that never answered says nothing at all;
Git's own refusal of the clean still lets it run and Git says why.
The forms are `Clean`,
`CleanIgnored` and `CleanIgnoredOnly`,
so a rejection says which files that spelling deletes.
A form with no question of its own asks only for the location,
so its one audit line stands beside the exempt and hatch lines that name no measurement either.
A `HEAD`-moving form is rejected only while tracked changes exist.
`git stash drop` and `git stash clear` are measured against the repository instead:
they delete entries of `refs/stash`,
which every worktree of the repository shares,
so a clean linked worktree does not make them safe.
The query is `git rev-parse --verify --quiet refs/stash`,
which reads no worktree,
and the record's `dirty` field says whether that query found an entry.
A `drop` or `clear` runs where the repository holds no entry,
because then it deletes nothing.
A `checkout` or `switch` that changes any file is also forwarded with `--no-overwrite-ignore`,
directly after the subcommand,
so Git itself refuses a switch that would replace an ignored file the target tree tracks,
naming the file.
Git decides that inside the process that changes the files,
so no measurement window exists for it.
The token is not added where the region changes no file,
where the caller already answered with `--overwrite-ignore` or `--no-overwrite-ignore`,
or where Git refuses the region;
a caller's own `--overwrite-ignore` after it still wins,
because Git applies the last spelling.
A forced `checkout -f` or `reset --hard` overrides the token,
so the ignored file it would replace inside the process is decided before the command runs:
for a forced `checkout`,
a forced `switch` and a hard `reset` (`--hard`,
`--merge`,
`--keep`),
the guard reads the paths of the tree that form writes (`git ls-tree -r -z --name-only <revision>`,
`HEAD` when the form names no tree)
and asks Git which of them the worktree holds as ignored files,
then refuses,
naming the paths,
while any path is both.
Every directory on the way to each of those paths is asked about as well,
because a tree creates the directories it needs,
and a worktree that holds a file where one of them must go loses that file
(Git removes it to make the directory);
an entry Git lists inside a directory the tree also has is not counted,
because writing the tree leaves it alone.
An entry Git lists inside a path the tree holds as a regular file is counted:
the tree replaces that directory with the file,
so every file inside it is deleted,
and that is the answer a filter that kept only exact names would drop.
Only these three forms are read that way,
and only in a linked worktree;
the question is one `git ls-files --others --ignored --exclude-standard` over those paths alone,
so no query walks the worktree's ignored files
(measured on this repository's worktree at 1,860 tracked paths and their 42 directories:
`git ls-tree` 4 to 6 ms and `git ls-files` 8 to 11 ms per start,
in ten-run batches with a load average of 61).
Every path is named with `:(top,literal)` magic,
anchored at the top of the worktree and read as itself,
and the query turns off a caller's `--literal-pathspecs`,
which would otherwise make the magic a path.
A revision Git cannot resolve is read as `HEAD`,
because every form that names no tree writes `HEAD`'s content.
A revision spelled `-`,
which only `switch` and `checkout` read as the previous branch,
is read as `@{-1}` before the tree is listed,
because no other Git command resolves `-` and the tree would otherwise be read as `HEAD`'s;
`@{-N}` is resolved by Git itself wherever an object name is read.
A tree or an answer Git cannot read is a `fact-unavailable` finding,
never a clean worktree.
A status Git cannot read is a `fact-unavailable` finding,
never a clean worktree.
Repositories under an allowlisted tool cache are exempt.
The list is chosen by the invocation's own environment:
uv's cache (`UV_CACHE_DIR`,
else `XDG_CACHE_HOME/uv`,
else `HOME/.cache/uv`),
mise's data directory (`MISE_DATA_DIR`,
else `XDG_DATA_HOME/mise`,
else `HOME/.local/share/mise`),
and Cargo's build-script output directory (`OUT_DIR`,
which Cargo sets for every build script and passes on to what it starts).
The third is where a crate that vendors another project clones that project
(`libghostty-vt-sys` cloned Ghostty into it) and runs Git in that clone,
so the clone is the build step's checkout and not the person's work.
Cargo's build directory is the fourth and the widest:
`CARGO_BUILD_DIR`,
then `CARGO_BUILD_BUILD_DIR`,
then `CARGO_TARGET_DIR`,
the first of them that is set and not empty.
The owner's answer of 2026-11-11 chose this entry (`#3024`,
option 1 of `doc/planning/git-wrapper-open-issues-round-4.md`),
because the vendored clone keeps every rule only while its directory is exempt,
and a second build step,
an interrupted first one,
or a person running Git in that clone
reaches the same trees without `OUT_DIR` in its environment.
The three names are not the same setting to Cargo:
Cargo never reads `CARGO_BUILD_DIR`;
`CARGO_BUILD_BUILD_DIR` is the environment spelling of `build.build-dir`
(Cargo's configuration reference),
and `build.build-dir` defaults to `build.target-dir`,
whose own variable is `CARGO_TARGET_DIR`.
The wrapper reads no Cargo configuration file,
because `build.build-dir` and `build.target-dir` resolve a relative value against the directory of the file that
set them,
can hold the `{workspace-root}`,
`{cargo-cache-home}` and `{workspace-path-hash}` templates,
and are host configuration the environment-named list exists to keep out of the wrapper's source.
A relative value names nothing:
Cargo resolves one against a directory this invocation cannot know
(`CARGO_TARGET_DIR` against Cargo's own working directory,
a configuration value against the file's directory),
and a Git process a build script starts works in that script's package directory,
so exempting a guess could cover a repository nobody named.
The limit of this entry is its width:
a repository someone deliberately keeps under the build directory
gets no worktree policies at all.
An empty value of any of these variables counts as unset.
A repository whose Git directory and worktree top both lie inside one of them is exempt,
so the exemption covers the tool's own checkout and nothing else:
a linked worktree keeps its Git directory in the main clone
(`<clone>/.git/worktrees/<name>`)
while its files stand where they were added,
and a worktree added outside that directory is the person's work even when the clone it came from is the tool's.
The wrapper says so on standard error before Git runs,
naming the variable and the directory it points at,
so a variable that happens to name a repository is visible to the person:
"this repository is exempt from the worktree guard because `MISE_DATA_DIR` names <directory>,
which contains both its Git directory and its worktree".
The comparison is made on resolved paths:
each allowlisted directory is resolved before it is compared,
and Git's identity query prints the resolved path of the worktree and the Git directory
for every spelling an invocation can use
(the working directory reached through a symbolic link,
`-C`,
`GIT_WORK_TREE`,
`--work-tree`,
`core.worktree`,
and a worktree added through a symbolic link all printed it on Git 2.56.0),
so an exemption never depends on how a path was spelled.
The record carries the same name in `facts.exemptedBy`
and the outcome `exempt`.
The rejection names the form,
what it destroys,
an alternative that fits the likely intent,
and the deliberate override (`--no-enforce-worktree` directly after the subcommand).

#### The abort forms while their operation waits

An operation that stopped with conflicts makes `git status` report its own state,
so the rule above refused every abort form for exactly as long as it was the command a person needs:
in a linked worktree a conflicted merge could be finished but not aborted without the hatch.
The abort forms are decided instead by the operation's own state in the worktree's Git directory,
read in one start of Git that asks `rev-parse --path-format=absolute --git-path` for
`MERGE_HEAD`,
`CHERRY_PICK_HEAD`,
`REVERT_HEAD`,
`rebase-merge`,
`rebase-apply/rebasing`,
`rebase-apply/applying`
and `sequencer`
(`repository_facts_abort_discard.rs`,
`rule_abort_operation.rs`).
`MERGE_HEAD` is a merge,
`rebase-merge` or `rebase-apply/rebasing` a rebase of either backend,
`CHERRY_PICK_HEAD`,
`REVERT_HEAD` or `sequencer` a cherry-pick or a revert,
and `rebase-apply/applying` an `am`,
which is read last so a rebase that applies patches is a rebase.
An answer that is not one path per name is a `fact-unavailable` rejection,
never "nothing waits".

Three answers follow from that state.
Where no operation of the form's kind waits,
Git refuses the command itself
(`There is no merge to abort (MERGE_HEAD missing)`)
and discards nothing,
so the command is left to Git and the record says `allowed`
with `dirty` null,
because no status was read.
Where the form's own operation waits,
the work that is not the operation's own is copied first and the command runs
(the owner's "save,
then allow",
2026-11-02,
which the conflicted-path carve-out below already follows).
Where the answer cannot be read,
the rejection above stands,
which loses nothing.

That work is read from Git's own listings.
For a merge whose `AUTO_MERGE` ref resolves,
it is the paths where the worktree differs from the result Git recorded
(`git diff --name-only -z AUTO_MERGE`)
or where the index does,
unmerged entries left out
(`git diff --cached --name-only -z --diff-filter=ACDMRTB AUTO_MERGE`):
an untouched conflicted path holds the recorded result and unmerged index entries,
so it is not named,
and a resolution written or staged over it is.
For a merge without `AUTO_MERGE`,
a rebase,
a cherry-pick,
a revert and an `am`,
every uncommitted tracked change is named
(`git diff --name-only -z` and `git diff --cached --name-only -z`),
because those aborts return with a hard reset that keeps nothing.
A listing that fails is a rejection,
never an empty list.
The copies are written by the same saver,
into the same `monochromatic-git-policy-cli/discarded/<UTC time>/` folder,
with the same bound,
and the notice names the command,
the folder and each copy before Git runs;
a path with nothing to copy is named as such,
and a copy that cannot be written rejects the command with the reason.
The record is `saved-abort` with the copies when something was saved,
and `allowed` with `dirty` null when the operation's own state was all there was.

The ignored-file question a hard reset gets is asked here too,
because the abort writes a tree back over the worktree:
the tree is `ORIG_HEAD`,
which Git records before a merge,
a rebase,
a pick or an `am` stops
(measured with Git 2.56.0 on a single conflicted cherry-pick,
where `ORIG_HEAD` held the commit `HEAD` was at before the pick),
and a repository whose `ORIG_HEAD` does not resolve is read as `HEAD`,
which is the commit the operation started at until it moves `HEAD`.
An ignored file at one of those paths keeps the rejection and is named,
as for a forced `checkout` and a hard `reset`.

The main worktree and a directory outside a worktree keep the rejection,
because the rule above refuses a form there always;
`git merge --quit` and its siblings stop the operation and keep the index and files,
and the hatch runs the abort anyway.
The check runs after the allowlisted tool cache has answered `exempt`,
so a repository no guard reads gets no copy either.

#### The window between the measurement and the command

The rule above is a statement about the state the wrapper measured,
not about the state at the instant Git changes
files.
The measurement is one `git status` in a child process;
the wrapper then replaces its own process with real Git,
which does the discarding.
Between the status leaving and Git starting there is,
on the allowed path,
the audit append (one `write`,
never synced),
the notices on standard error,
and the in-process decisions of the fixed transforms and of the forwarding plan:
no second Git start,
no lock,
no sync,
no wait.
A write that lands inside that window is not seen,
and nothing in the wrapper can close it:
only the process that changes files can decide,
against the state it is about to change.

Where Git itself has such a check,
the wrapper hands the decision to Git instead of measuring,
and then there is no window:
a `checkout` or `switch` that changes files is forwarded with `--no-overwrite-ignore`,
so Git names the ignored file it would replace and refuses
(see "A `HEAD`-moving form" above).
Git's own refusal of a switch or checkout that would overwrite a tracked modification covers the same ground for
tracked files.
`reset --hard`,
`restore`,
`clean` and the other forms have no equivalent option,
so their guard is a measurement and keeps this window.

A person who needs the stronger promise runs the discard in a disposable worktree
(`git worktree add <path> <commit>`),
where the work being discarded is not theirs.

#### The two forms that are recorded and never refused

Two commands run exactly as Git runs them,
by the owner's decision,
and leave one audit line each
(outcome `logged`):

- `git worktree remove` in any worktree.
  The owner's words:
  "This one is pretty intentional because agents are just as likely to run a rm rf,
  so preventing git worktree remove buys us nothing."
  The line's `facts` hold `worktree` (the path as typed),
  `force` (whether `-f` or `--force` was given),
  `head` (the removed worktree's `HEAD` ref,
  read from its own Git directory),
  `changed` (how many tracked paths differed from that `HEAD`,
  from one `git status --untracked-files=no` in the target,
  read under `CHANGED_PATHS_CEILING` so a slow disk cannot hold the removal,
  and left out when that read does not answer in time),
  and `counted`,
  which says `tracked changes only`.
  Untracked and ignored files are deliberately not counted:
  walking them is what would make a removal of a large worktree slow,
  and the record says what it counted rather than claiming the worktree was clean.
  The target is the path Git resolves before it removes anything:
  the named path,
  or the one registered worktree whose path components the name ends,
  from one `git worktree list --porcelain -z`,
  whose NUL-terminated fields keep a path that holds a line break whole
  (splitting such a path at its line break would read another worktree),
  asked first because every read of the target has to happen before Git removes it
  (measured at 32 to 50 ms per start in this repository's worktree,
  in ten-run batches with a load average of 70),
  so `git worktree remove topic` describes `/r/worktrees/topic`,
  and not a directory of that name beside the command.
  A name no worktree answers is the path the command named,
  and a name that matches several worktrees is one Git refuses,
  so neither claims a worktree the command will not remove.
  The line is written whether or not `--no-enforce-worktree` was given:
  the hatch skips the policy that writes it,
  and the line is the owner's own requirement,
  so it is written where a guarded form's hatch line would be.
  Both files are read only when they are regular files,
  and each is opened without waiting (`O_NONBLOCK`) and read up to 4,096 bytes:
  a FIFO named `.git` with no writer,
  or one whose writer never writes,
  would otherwise hold the removal for ever,
  and a record that cannot be completed is left out rather than waited for.
  A relative `gitdir:` pointer is resolved against the worktree that holds it,
  as Git resolves it,
  so the `HEAD` named is that worktree's,
  and a pointer Git would refuse names no `HEAD` at all.
- `git branch -d`,
  `git branch -D` and `git branch --delete` from the main worktree.
  The owner's choice is "Allow,
  add audit line".
  One line per branch that exists,
  with `facts` holding `branch` (the name as typed),
  `reference` (the full reference the deletion took:
  `refs/heads/<name>`,
  or `refs/remotes/<name>` when `-r` or `--remotes` was given),
  `commit` (the full object name that reference pointed at,
  so `git update-ref <reference> <commit>` restores it) and `merged`
  (whether that commit is an ancestor of the command's `HEAD`,
  which is what Git's own `-d` means).
  The namespace is read from the command,
  because the same name can exist as a local branch and as a remote-tracking reference at two different commits,
  and a record that read the local branch would name the wrong commit to restore.
  A reference name is bytes and not text:
  a branch name that is not UTF-8 is passed to `rev-parse` as it is written,
  and the line names it twice,
  `reference` as its text and `referenceHex` as its bytes,
  the way the record's own `gitDirHex`,
  `cwdHex` and `argumentsHex` keep theirs.
  The text field is how a person reads the line;
  the hex field is the identity,
  because a text field for `refs/heads/lost-\xff` holds replacement characters,
  and restoring from it would create a different branch.
  A name Git expands before deleting is read through the expansion Git's own branch deletion performs,
  which depends on the namespace it deletes from.
  A local one is `git check-ref-format --branch`,
  Git's `copy_branchname` under `INTERPRET_BRANCH_LOCAL` behind Git's own branch-name checks,
  so `@{-1}` (the branch the worktree visited before the current one) resolves to that branch,
  and a revision that is no branch name (`HEAD`,
  `@`,
  `main@{1}`) resolves to nothing.
  A remote-tracking one is `git rev-parse --symbolic-full-name --verify`,
  read only when the operand spells a branch mark (`@{`) and Git's answer is a `refs/remotes/`
  reference,
  which is what `INTERPRET_BRANCH_REMOTE` leaves of `@{u}` and `@{push}`,
  and which `@{-1}` and `HEAD` never are because they resolve to a local branch.
  General revision resolution is never asked for a name of its own:
  it answers `refs/heads/main` for `HEAD`,
  and a line naming `main` for a deletion Git refused would restore a branch the command
  never took,
  where the owner's line states what the deletion took.
  The line names the branch the typed or expanded name resolved to and its commit,
  so the owner's line is written for every deletion Git runs;
  the typed reference is asked about first,
  so a plain name costs the one query it always did,
  and a reference literally named `refs/heads/HEAD` is still a branch of its own that Git
  deletes and the line records.
  A line states what the deletion was about to take,
  not what it took.
  `deletion_facts` reads the reference and its commit,
  `record_audited` appends the line,
  and only then does real Git resolve the reference again and delete it,
  so the read,
  the line and the deletion are three steps,
  and the wrapper holds no lock across them.
  Another command can move the branch in that gap
  (`git update-ref refs/heads/lost <newer> <older>`),
  and Git then deletes the newer tip while the line names the older one:
  restoring from the line restores a tip the deletion did not remove,
  and the tip it did remove is named by no line.
  The deleted branch's own reflog goes with its reference
  (measured on Git 2.56.0:
  no `.git/logs/refs/heads/<name>` survives `git branch -D <name>`),
  so that tip stays only where the object store and any other reference or reflog still reach it.
  `writtenBeforeGitRuns` already says the line is no result,
  and the gap is accepted rather than closed by a lock:
  "Use as few locks as possible",
  and a lock over the reference would have to be held across the person's whole Git command.
  The region is read with the wrapper's own `git branch` option table and tokenizer,
  the same ones every other `git branch` fact uses,
  so every spelling and abbreviation Git accepts is read
  (`--remote` is `--remotes`,
  `-rD` is one cluster,
  and the value of `-u` is never a name:
  `-ur` sets the upstream of `r` and selects nothing)
  and no list of spellings can fall behind Git's table.
  A name with no reference in that namespace writes no line:
  it deletes nothing and Git's own error is the answer.
  `--delete-merged` is not read,
  because it deletes only branches a merged branch already contains,
  and an abbreviation of `--delete` is a region Git refuses for ambiguity with `--delete-merged` in Git 2.56.0.

#### Conflicted files during a merge, rebase, cherry-pick or revert

The owner's decision of 2026-11-02 (choice "save,
then allow") is one exception to the rule above.
While a merge,
a rebase of either backend,
a cherry-pick or a revert is in progress,
a `checkout` or `restore` command line that takes a side of conflicted files is let through,
in the main worktree and in a linked worktree,
after the wrapper copies the current worktree file of each path.
In progress means `MERGE_HEAD`,
`CHERRY_PICK_HEAD` or `REVERT_HEAD` in the worktree's Git directory,
a `rebase-merge` directory,
or a `rebase-apply` directory that holds `rebasing`
(with `applying` it is `git am`,
whose conflicts are not covered).
A conflicted path is a path with an unmerged index entry (stage 1,
2 or 3),
read from the index file in process:
no Git process runs for the decision.
An index that cannot be read in process (split,
sparse,
carrying an extension the readers cannot vouch for,
unreadable,
or chosen by `GIT_INDEX_FILE`)
shows no path as conflicted,
and the usual rejection stays.

The forms covered are:

- `checkout [<tree-ish>] -- <path>...`,
  with `--ours`,
  `--theirs`,
  `-2`,
  `-3`,
  `-m`,
  `--merge`,
  `--conflict` and the quiet,
  progress and overlay options.
- `restore -- <path>...` with `--ours`,
  `--theirs`,
  `-m`,
  `--merge`,
  `--conflict`,
  `-s` or `--source`,
  `-W` or `--worktree` (with `-S` only beside `-W`),
  and the quiet,
  progress,
  overlay and ignore options.
  Abbreviated `restore` options are not read and keep the rejection.
  `-W`,
  `--worktree`,
  `-S` and `--staged` count only where Git parses them as options:
  the token after `--source`,
  `-s` or `--conflict` is that option's value,
  so in `restore --source -W --staged -- f`
  (a tag can be named `-W`) the `-W` is a revision,
  the restore is index-only,
  and the rejection stays.
The rejection reads the line the same way and names `git restore --staged` (the policy's own classification of an
index-only restore skips those values and reads `-sW` as `-s` with the value `W`).

Every other line keeps the rejection:
a force,
a patch,
a path list read from a file,
a branch option,
a form without `--`
before the paths,
an index-only `restore --staged`,
and any `reset` or other guarded command
(the abort forms of `merge`,
`rebase`,
`cherry-pick`,
`revert` and `am` are decided by the carve-out above,
which lets one run only where the operation it undoes waits).
A path is read by its literal spelling below the working directory.
A directory,
a trailing slash,
a wildcard,
pathspec magic or a path outside the worktree is not a conflicted file.
A global pathspec mode changes which files a pathspec names (`--icase-pathspecs` and `GIT_ICASE_PATHSPECS` match `A.rs`
for `a.rs`;
the glob,
noglob and literal forms change the reading of the characters),
so a line under one keeps the
rejection:
the layout reader declines every global option except `-C` and every `GIT_` variable it does not tolerate,
and `core.ignorecase` does not widen a pathspec.
The rejection of such a line stays,
and when the line qualifies,
an operation is in progress,
the index is readable and
every path it names is a conflicted file,
the message gains one sentence:
it names the mode (`--icase-pathspecs`,
`--glob-pathspecs`,
`--noglob-pathspecs`,
`--literal-pathspecs`,
`--no-literal-pathspecs`,
`GIT_ICASE_PATHSPECS`,
`GIT_GLOB_PATHSPECS`,
`GIT_NOGLOB_PATHSPECS`,
`GIT_LITERAL_PATHSPECS`)
and says the same command without it would be saved first and then allowed.
The layout reader is asked a second time,
without the mode,
only for such a line.
Pathspec magic on a conflicted file's name (`:(icase)a.rs`,
`:(glob)a.rs`,
`:(literal)a.rs`,
`:(top)a.rs`,
`:(exclude)a.rs`,
`:!a.rs`,
`:^a.rs`,
`:/a.rs`) is rejected the same way,
and when every token that is not a plain conflicted file is such magic
the message names the magic and the file and says that naming the file itself is saved first and then allowed.
Other magic (`:(attr:...)`,
`:(prefix:...)`),
a wildcard,
a clean file,
a mix,
a form the carve-out does not cover and a
repository with no operation in progress get no such sentence,
because the carve-out would refuse those anyway.
A line that names a path that is not conflicted is rejected whole,
with the rejection text of the form
followed by a sentence that names every such path.
A path that Git resolves in the index as the command runs (`checkout <tree-ish> -- <path>` stages it) stops being conflicted,
so a later line that names it is judged by the rules again.

The copy is written and synced before Git starts.
It goes to `<common-git-dir>/monochromatic-git-policy-cli/discarded/<UTC time as YYYY-MM-DDTHH-MM-SS>/<path from the top of the worktree>`,
next to `monochromatic-git-policy-cli-audit.jsonl`;
a second invocation in the same second gets `-1`,
`-2` and so on after the time.
The directories are private (`0700`),
and the copy has the file's content and permission bits
or the symbolic link's target.
The wrapper prints,
before Git runs,
for each path:

```text
monochromatic-git-policy-cli: saved your copy of a.rs to
  /path/to/repo/.git/monochromatic-git-policy-cli/discarded/2026-11-02T14-01-07/a.rs
  before this command replaces it (a.rs was conflicted).
```

A conflicted path with no file in the worktree has nothing to copy:
the wrapper says so in one line and allows the command.
If a copy cannot be written (a full disk,
a refused permission,
a place that is not a directory,
a file that is neither a
regular file nor a symbolic link),
the command is rejected with a finding that names the path or place and the system's reason,
no cut-short copy is left,
and nothing is changed.

The file is found without following a symbolic link.
Each directory in front of the name is opened one component at a time below the worktree's top level,
and a symbolic link or a regular file in front of the name stops the command:
Git writes `dir/file` by replacing whatever stands at `dir` with a real directory,
so that link or file would be lost unsaved.
The finding names the directory (`dir`) and what it is,
and says to move it out of the way or save it first.
A directory in front of the name that does not exist means there is nothing to copy.
The copy is read from the opened file.
Its device,
inode,
size,
modification time and change time are compared before and after the read,
the byte count must equal the size,
and the name must still lead,
without a link,
to the object that was read.
Any difference (an atomic save over the path,
an append,
a truncation or a rewrite in place) removes the copy,
rejects the command with a finding that names the file and says to run it again once nothing is writing to it,
and changes nothing.
A write that lands after the last comparison and before Git starts cannot be seen from outside Git.
A file that another program changes while the save runs is the user's risk.

The folder is bounded after each save that wrote at least one copy,
once the copy is complete and synced (so a kill while copying costs no older copy),
with nothing waiting on it.
The folder is opened once without following a link at `monochromatic-git-policy-cli` or `discarded`
(a link there,
or a non-directory,
keeps every copy and is said in the notice and by the doctor),
and everything is read and removed through that descriptor,
never through a path walked again.
No lock covers the folder (the folder's `flock` of earlier releases is gone):
passes may overlap,
and none waits for another or turns away because another runs.
A file has one owner,
the pass whose `unlinkat` of it succeeded (the kernel lets one succeed),
and only that pass counts and audits it,
so overlapping passes split a directory's files between their audit lines and every copy is audited once.
A file or a directory that another pass removed first is not an error,
and a directory that is gone before the pass opens it is not a problem.
A directory is removed only when,
opened by name without following a link,
it is still the directory the listing saw (device and inode).
Its change time moves when another pass removes files from it,
so a moved change time is not a replacement by itself:
the pass looks at what the directory holds now,
and keeps it,
naming the cause,
only when it holds a file the listing did not see.
Its files go through its descriptor,
files of a directory before its subdirectories,
then the empty directory is removed by inode,
so a replacement under the same name is never touched.
A link inside a save is removed as a link and never entered;
a tree deeper than 128 levels is kept.
The pass reads only directories whose names are a save time
(`YYYY-MM-DDTHH-MM-SS`,
with `-<number>` when a second save fell in one second);
the age of a save is the time in its name,
not a modification time,
a name must be a real calendar date and time (February 30 and second 60 are not names the wrapper makes),
and a name says only its second,
so every age is counted from the end of that second.
It removes every save older than `discardedCopies.maxAgeDays` days (default 30),
then,
while the folder's files total more than `discardedCopies.maxSize` bytes (default one gibibyte),
the oldest remaining save,
until the total is within the limit or nothing is within reach.
A save younger than 24 hours (counted from the end of the second in its name) is out of reach of both limits,
and so is the save that was just written,
so a burst of saves is never cut by size and a save made with an old clock survives its own pass.
Both settings are keys of the `discardedCopies` section of `monochromatic-git-policy-cli.config.jsonc`:
`maxAgeDays`,
a safe integer of at least 1,
and `maxSize`,
a safe integer of at least 1 (bytes) or a string of digits with an optional suffix `k`,
`m` or `g`
(powers of 1024,
any case).
A value that is not that is a configuration failure that names the key,
as for every other key of the file
(exit `2` before any policy runs);
zero never means "no limit" or "remove everything",
and nothing is removed.
An unknown key under the section is refused with `maxAgeDays` and `maxSize` named.
Each removed save is followed by one audit line per pass that removed copies from it,
written once those copies are gone:
`schemaVersion`,
`time`,
`level`,
`timeMillis`,
`release`,
`pid`,
`form` (`PruneDiscarded`),
`outcome` (`pruned`),
`reason` (`age` or `size`),
`limit` (days or bytes),
`folderBytes` (the folder before the pass),
`path`,
`bytes`,
`copiesTotal` and `copies` (at most one hundred names).
A directory removed in part has a line whose `outcome` is `partial` and whose `copies` are the files that went.
Sizes and ages are computed in 128-bit integers,
so no sum or difference wraps.
Nothing fails silently:
a directory that cannot be read has no size to judge by and is kept and named,
a removal that fails stays and is named (as removed in part when some files went),
and a pass that cannot append to the audit file removes nothing and says so.
The sentences are in the notice of the command that saved the copy and in `git monochromatic-git-policy-cli doctor`.
`git monochromatic-git-policy-cli doctor` writes to standard error,
when the folder holds any save,
its path,
total bytes,
number of copies and saves,
the oldest save and the limits in force.

Every decision about a guarded form is appended as one JSON line to `<common-git-dir>/monochromatic-git-policy-cli-audit.jsonl`
with `schemaVersion`,
`timeMillis`,
`pid`,
`parentPid`,
`worktree`,
`form`,
`outcome`
(`rejected`,
`allowed`,
`exempt`,
`hatch`,
`logged`,
`saved-conflict`,
`saved-nested-repository` or `saved-abort`),
`dirty`,
`cwd` and `arguments`.
Each path and argument this line names is written beside its text as its bytes
(`gitDirHex`,
`cwdHex`,
`argumentsHex`),
the two-field form every other record of a path takes (`path_bytes_in_records.rs`),
so a command whose working directory or argument is not UTF-8 is still the command the line names:
the line exists to attribute a loss to it.
The text fields hold what a reader of an earlier release read.
A `saved-conflict` line also has `copies`,
a list of `<path> => <place>` (`none` when there was nothing to copy),
and so do a `saved-nested-repository` line,
whose places hold the repositories a clean would have deleted,
and a `saved-abort` line.
A `logged` line also has `writtenBeforeGitRuns` set to `true`,
and no other outcome has the field.
`dirty` is whether the worktree or the repository held what the form could destroy,
`false` when the answer was no,
and `null` when the policy asked nothing:
a `git clean -X` in a linked worktree deletes ignored files only,
so it has nothing to measure.
The wrapper appends that line before it replaces itself with Git,
so the line cannot know whether Git ran the deletion or refused it:
`git branch -d` of a branch nothing else contains is refused,
`git worktree remove` of a dirty worktree without `--force` is refused,
and each leaves the same line as the deletion that ran.
The field says that in words a reader cannot take for the result,
where `outcome` alone said `logged` for both.
A `rejected` line of a forced `checkout`,
a forced `switch` or a hard `reset` whose tree tracks a path the worktree ignores also has
`ignoredAndTracked`,
the paths it read,
and its `dirty` is `false` because the tracked state of the worktree was clean:
the refusal is about the ignored files,
and the paths are named so a reader sees which of them the command would have replaced.
The line also has `time` (the ISO form),
`level` (`warn` for `rejected`),
`release` and `gitDir` (the invocation's own Git directory,
which `worktree` the kind word does not name).
The file moves aside at one mebibyte to the next free generation,
`monochromatic-git-policy-cli-audit.jsonl.<number>`,
and the generations are kept up to 32 MiB together.
A generation that names a saved copy still on disk is never removed,
even past that bound (see "Timing logs").
A failed write never changes the command's result.

After an explicit-path landing computes its post-index,
the wrapper lists the stage entries of the pre-landing and post indexes,
and warns with the differing paths outside the committed and guarded paths.
The warning never changes the landing.

### The remote settings every worktree shares

The owner answered "Refuse in worktrees (Recommended)" on 2026-11-10
(`doc/handover/questions-for-the-owner.md`,
"A push URL changed from a worktree"),
after a worktree of this repository ran `git remote set-url --push origin DISABLED` as if it were a clone.
Every worktree of a repository shares one `.git/config`,
so that change broke every push of the main checkout until someone removed the entry,
with nothing recording which command had made it.

In a linked worktree,
which is any worktree whose Git directory is not the common one,
wherever it sits on disk,
`linked-worktree-only` rejects a command that writes those shared remote settings,
with the finding code `shared-remote-settings`:

`git remote set-url`,
`git remote add`,
`git remote remove`,
`git remote rm`,
`git remote rename`,
`git remote set-branches` and `git remote set-head`,
in every position Git accepts `git remote`'s own options.
`git remote`'s only option before a subcommand is `OPT__VERBOSE`:
`-v`,
`--verbose`,
its unique abbreviations and `--no-verbose` with theirs,
so `git remote -v set-url origin <url>` and `git remote --verb add beta <url>` are the same writes as the plain
forms.
The subcommand words are matched exactly,
because Git dispatch matches them exactly and refuses every abbreviation.
The tokens after the subcommand belong to the subcommand's own option table and are not read as this one's,
so `git remote add -t main beta <url>` is still `add`.
A word no built-in shares is resolved as an alias before this reading,
so `git -c alias.su=remote set-url su origin <url>` is the same rejection.

`git config` rejects a write whose key or section falls under `remote.`:

- the plain `name value` form (two arguments),
  `set-all` (three arguments,
  which replaces the values a pattern selects),
  `--add`,
  `--unset`,
  `--unset-all` and `--replace-all`;
- `--remove-section` and `--rename-section`,
  including a rename whose new section lies under `remote.`
  because it creates remote settings just as renaming out of `remote.` removes them;
- the `set`,
  `unset`,
  `remove-section` and `rename-section` subcommands.

The key or section is the first token Git hands the command as an argument,
and the scope is read only where Git reads it:
with `PARSE_OPT_STOP_AT_NON_OPTION` every option after the key is data,
so `git config remote.origin.url --worktree` stores the text `--worktree` in the shared file and is rejected.
Section names are compared without case,
as Git compares them,
while the subsection keeps its case,
so `Remote.Origin.URL` is a write to the same section as `remote.origin.url`.
A region Git refuses writes nothing and is not rejected:
no action and four or more arguments,
two actions at once,
and two scope options at once.
`git config --edit` names no key and edits the file the region selects,
which without a scope option is the shared one:
an editor session is rejected where the shared file is,
because the editor can change any remote setting,
and `--edit` with `--global`,
`--system`,
`--worktree` or a `--file` of its own runs.
The `edit` subcommand takes no argument,
so `git config edit <key>` is Git's own refusal and not this one,
while the legacy `--edit` runs the editor whatever follows it and is rejected as the editor session it is.

No read is rejected.
No write to another file is rejected:
`--global`,
`--system`,
`--worktree` and `--blob`,
the last of which Git refuses for a write anyway.
`--file` names another file only when the path it takes is not `$GIT_COMMON_DIR/config`:
the path is resolved the way the filesystem resolves it,
from the directory the command runs in,
so an absolute spelling,
one written with `=`,
a relative one,
one through `..` or a link and one reached after `-C` are the plain form's rejection when they name that file,
and a path that names any other file runs.
That directory is where the command really starts,
moved by its `-C` chain:
`--work-tree` names the tree Git reads and writes,
never the directory a relative `--file` path is read from,
and `git config` does not move into it,
so a relative path under a `--work-tree` elsewhere is read from the same place as without one.
The Git a `!` alias's shell runs starts somewhere else again:
Git changes to the top of the working tree before it starts that shell
(`git.c:388` to `git.c:407`),
so a relative `--file` path written there is read from that top,
while the caller's `-C` chain moves neither.
A `-c` global option is one command's override,
changes no file and is not a write to the shared settings.
The main checkout,
a bare repository,
a place outside any worktree and a repository under an allowlisted tool directory keep today's behavior.

The `--file` check decides from a path,
and real Git opens that path after the wrapper has decided.
A process that replaces the path in between,
by repointing a link or by putting a link where a file was,
makes Git open a file the wrapper did not compare,
and the write lands there.
No re-check of the same path closes that,
because a swap can follow every check,
and every complete answer either refuses more than Git or takes the write away from Git:
refusing every `--file` write in a linked worktree,
forwarding the resolved path instead,
which moves the same swap one step rather than closing it,
writing the file inside the wrapper,
which replaces Git's own locking and its byte-preserving rewrite,
or holding `$GIT_COMMON_DIR/config.lock` for the command's whole life,
which would fail every legitimate concurrent write from the main checkout.
The guard therefore stops the command it was asked to stop,
not a process that writes that file itself,
under `--no-enforce-worktree` or without Git.
A link is resolved before the comparison,
so a spelling that reaches the shared file through one is refused;
a hard link is another name for those bytes and is no bypass,
because Git replaces a hard link rather than writing through it,
which leaves the shared bytes what they were.

The rejection changes nothing,
exits non-zero and never starts Git.
It says that every worktree of the repository shares the setting,
names the file (`<common-git-dir>/config`) and the main checkout,
and names both ways forward:
run the same command in the main checkout,
or make a real clone for a copy whose remote settings are its own
(`git clone --no-hardlinks <path> <destination>`).
It also names `--no-enforce-worktree`,
the hatch of this policy,
which runs the command as before:
directly after `remote`,
the only position a command without a ported table reads it,
or in option position of `git config`.
A run under the hatch leaves one `hatch` line in the audit file whose `form` is `SharedRemote`,
with the working directory and the arguments,
so the change is attributable;
a rejection leaves no audit line,
because nothing was written.
The refusal is the guard's decision on the command Git will run,
so a shell alias (`git -c 'alias.su=!git remote set-url' su origin <url>`) is refused as the command is.

`src/native/binary/binary_shared_remote_tests.rs` runs every refused form in a linked worktree made inside and
one made outside the repository folder,
every read and every write to another file,
the hatch,
the main checkout and an allowlisted cache through the built executable,
and compares the shared configuration byte for byte before and after each rejection.
It also runs every spelling of the shared file a `--file` write can take,
a relative one under a `--work-tree` that names another directory included,
and both editor forms,
requires the shared bytes unchanged and the editor never started,
and keeps the positive control that the same editor on the same worktree does change the shared file when Git
itself runs it.
A `--file` write from the Git a `!` alias's shell runs is refused from the top of the working tree that Git
changed to,
with the same real-Git control writing the shared file through the same relative spelling.
The decision is `src/native/command/rule_shared_remote.rs`,
the path a `--file` region names is read by `src/native/command/command_config.rs`,
and the comparison with `$GIT_COMMON_DIR/config` is `src/native/policy/policy_linked_worktree.rs`.

### Dependent version bump

`mono/dependent-version-bump` reads the publish set from the generated registry configuration
(`package/config/pnpr/config.yaml`) as the document format it is,
never as a line of text (`dependent_version_publishable.rs`).
Every `packages:` list under an `auth.oidc[]` workload is the set,
because a package any trusted CI identity may publish is published;
the names are read in document order without repeats,
so a narrower workload list above the publish set cannot hide it.
The `packages` mapping under `registries` grants a permission pattern to a scope and lists no
publishable name,
and is never read as one.
The executable links no YAML parser.
`package/config/pnpr/src/generated-config-render.ts` writes the file,
so that template is the reader's whole specification:
`dependent_version_yaml` and `dependent_version_yaml_lines` accept exactly the
line forms it writes and refuse every other line by name and number,
never keeping text they would have to guess at.
The accepted forms are a blank line,
a comment line beginning `#`,
`key: value`,
`key:` with its block below,
and a sequence item whose dash is followed by exactly one space.
Indentation counts spaces,
never a tab,
and reaches fourteen spaces at most,
the depth of the template's `claims` children.
A line beginning `---` that is not exactly `---` is refused,
because a document marker belongs alone on its line and this reader reads one document.
A tag,
anchor,
alias,
block scalar,
directive,
reserved character,
flow collection,
comment or quote indicator,
the `-`,
`?` and `:` that open an item or an explicit key,
an empty key,
a colon a space or tab follows inside a value,
text a closing quote does not separate,
and an interior carriage return are refused too.
A space or a tab separates a key from its value and starts a comment,
and an interior tab stays part of a plain scalar as it does to a YAML parser.
A list item holds no `#`,
because the template writes its items without a comment,
so a `#` on one is refused with its line;
the colon that makes a key is looked for only before a comment,
so a colon inside one names no key.
Because no line sits deeper than fourteen spaces,
the block recursion is bounded by that indentation and the reader runs on constant stack;
a configuration nested deeper is refused at its first over-deep line
instead of exhausting the stack.
Every line is read once,
and a repeated key is refused in constant time,
so a configuration of any length is one pass.
A general YAML parser is not a dependency of this executable:
it would accept every YAML document while the publish-set traversal would still
have to map a shape set wider than the generator writes,
which is the silent miss this reader refuses instead.

A publish set that cannot be read is a `policy-incomplete` engine failure,
which stops `check` and `fix` with exit `2`:
the file is not tracked,
no `auth.oidc` workload lists a package name,
or the text is not block YAML this executable reads.
That is the status every input these commands cannot use already has,
and an unjudged ripple therefore never reads as a clean workspace.
The diagnostic names the file,
the cause,
and the escape hatch.

#### What a commit does with the ripple

A wrapped `git commit` plans the same ripple as `check` and `fix`,
as the owner's answer of 2026-11-10 decides
(quoted in "Finding and patch validation")
and the decision of 2026-09-15 records
(`doc/decision/private-npm-registry.md:158-177`).
When a workspace manifest's `version` differs from the base the commit recorded,
every publishable dependent is patch-bumped,
transitively,
and its manifest joins the commit as an added path
(`policy_added_path::commit_refusal`,
"Added paths").
After the landing the wrapper writes each committed manifest into the worktree
while that copy still holds the bytes the bump was computed against
(`commit_worktree_records.rs`,
`worktree_completion.rs`);
a copy that moved in the meantime is kept and reported.

A commit never fails over the ripple.
Each of the following is a note,
never a finding that blocks and never an engine failure that exits `2`:

- a dependent whose `HEAD`,
  commit-index,
  real-index and worktree copies do not all hold the
  manifest the bump was computed against,
  which is somebody's unstaged or staged work,
  so the dependent alone is left out of the commit and keeps its version;
- a dependent whose version cannot be patch-bumped automatically;
- a publish set the commit cannot read,
  or any other input the planner cannot use;
- a real index that could not be listed,
  which leaves out every dependent whose copies that listing would have proved,
  each named with the reason Git gave.

A note prints as an engine-failure event with code `policy-incomplete` whose chosen hint says the command
is not stopped and nothing was discarded,
and which remedy fits:
`LEFT_ALONE_HINT` asks the person to commit the file a copy holds or to restore it,
and `UNPROVEN_RIPPLE_HINT` asks for the Git read that failed to answer first
(`policy/policy_events.rs`).
The pass goes on whatever the policy's severity.
The policy reports it on the commit path only:
`git add` and every other forwarded command still apply no correction,
so they plan nothing.

### Forbidden root context

Reject root `CONTEXT.md` for commit candidates and direct check.
Do not reject nested files with that basename.
Deleted root candidates are clean.

### Forbidden strings

Use candidate bytes rather than incidental worktree bytes.
Post-commit scan uses landed commit ground truth.
Manual push runs a Git `git push --dry-run --porcelain` probe with every hook off and full object names
(`-c core.abbrev=no`),
parses Git's own report of the updates (`manual_push_porcelain.rs`),
reads local objects the report does not name with one `git rev-parse`,
and validates the reported remote OIDs with `git ls-remote --refs` before policy evaluation;
a report that does not state the remote's value (a deletion,
a refusal) takes it from `ls-remote` alone.
A destination that now holds exactly the object its update sends is no move at all,
and that update adds nothing to the scan:
the commit's own auto-push publishes the same commit while a typed push is on its way,
which is the one push that can change a destination between those two reads,
and nothing is left for the typed push to publish
(`src/native/push/manual_push_probe.rs`,
`confirm_group`).
A destination that moved to any other value stays the `remote-moved` failure below.
The probe adds `--no-quiet` to its copy of the options,
because `-q` empties the report,
and skips the line Git adds for
`-u`,
`--set-upstream` and `push.autoSetupRemote`;
any other line outside the report format stops the probe and is named.
A single destination's printed location is used as it is,
which saves the query for the push URLs.
The probe starts no program of the wrapper's,
writes no hook or script,
and every Git child of it is bounded by the
query deadline.
When the probe's Git ends with a failure status and a message before it reports any destination,
Git has refused the push
itself (a branch to delete that the remote lacks,
a refspec naming nothing,
an unknown remote or option):
nothing could be
published,
so the wrapper ends with Git's exit status and Git's standard error as Git wrote it,
with no event and no hint.
A failure of the probe's own reading (Git cannot run,
a failure without a message,
a report outside the format,
a remote that
moved to a value the push does not send) stays a `content-unavailable` engine failure and exits `2`.
That one code covers causes the person answers differently,
so the failure carries its cause and the event's hint is the
cause's,
not the code's:
a remote that moved says to run the push again;
a remote whose refs could not be read says to check
the network,
the address and the credentials,
and its message masks the secrets of the address;
Git that cannot start or
ends without a message says to run Git's own `--dry-run`;
output the wrapper cannot read is reported as a defect with the
output of `git --version`;
a pushed object that is not a commit says to push a commit;
and only content missing from the
clone keeps the code's hint about fetching history.
A message that already says what to do (a deadline with its setting,
a
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
Skip the bytes of a candidate whose blob a parent's tree already holds,
as "Content already in history" defines.
Load unique Git blobs through one `git cat-file --batch` process per evaluation.
The scanner reads candidate content in process,
and no scanner file is materialized.
Wrapper-added latency for every forwarded real Git command must remain strictly less than `2,000 ms`.

Every commit made through the wrapper must not take `5,000 ms` or more longer than the same commit made with native Git;
the bound holds for each commit,
not for a percentile of them.
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

A scanned candidate is an added or modified path,
so a rename lists its file as an addition,
and a merge lists every file that one parent lacks as an addition,
although the bytes are already in history.
A blob's object name is the hash of its bytes,
so a candidate whose blob a parent's tree holds carries no new byte:
rescanning it only refuses moves of content that was accepted when it entered history.
The policy therefore reads no bytes for such a candidate.

The reference trees depend on the request that produced the candidates:

- `Add`,
  `Staged` and `StagedAtBase`:
  the tree of the baseline commit,
   which is the recorded base of a commit transaction,
  else the commit `HEAD` names.
   An unborn branch has none.
- `Landed`:
  the tree of every parent of the landed commit.
   A root commit has none.
- `Pushed`:
  the tree of every parent of a pushed commit that is not itself a pushed commit,
   so only history the remote already had vouches.
   A push that carries a tip tree (a tip with no known published commit) has no such parents and skips nothing,
   because its candidates share one version with the commits' candidates.
- `Direct`:
  none.
   A direct check reports what the selected files hold now.

The rule is exact and local:

- The comparison is the blob object name of the candidate against the blob object names of the reference trees,
   per candidate.
   It never compares pathnames and never uses rename or similarity detection.
   A changed byte changes the object name and the candidate is scanned in full.
- Only scanned blobs qualify.
   A deleted path,
  a submodule entry and the rules file and canonical self-match sources the scanner excludes
   keep their existing handling,
  and a gitlink in a reference tree never vouches for a blob.
- A candidate that this invocation's fix corrected never qualifies,
   because its bytes are no longer its object's.
- A known candidate is still given to the scanner with empty content,
   so a pathname that matches a rule is reported as before,
  masked.
- Each reference tree is listed at most once,
  in reference order,
  and listing stops when every candidate is placed.
   No Git process starts for a version without a scannable candidate or without a reference.
- Failing to list the reference trees never allows a candidate:
   every candidate is scanned in full and a warning names the failure.
- Each skip is a decision record on the debug channel (`MONOCHROMATIC_VERBOSE=true`):
   the scanner's masked pathname,
  the blob's object name,
  and the commit whose tree holds it.

Security reasoning.
The policy exists to keep secrets from entering history and from being published.
A blob a parent already holds is not new exposure:
it is already in the tree the new commit builds on,
and in a push it is already on the remote.
Scanning it again blocks legitimate moves without protecting anything new.
A push counts only parents the remote already had,
so a secret introduced by a pushed commit is scanned at that commit even when a later pushed commit copies it.

Tradeoffs.
Content that entered history through an earlier bypass stays unscanned when it is copied or moved,
and so does content at the paths the scanner never reads,
such as the rules file and its own rule sources,
which is what lets the scanner package move.
A rule added after a file entered history applies to the file again only when its bytes change.
The skip can only be as strong as the parent trees:
it is never taken from the worktree or the index.

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

A file whose Git `text` attribute is unset (`-text`,
which the `binary` macro includes) keeps its exact bytes.
Only the value `unset` preserves a file:
`set`,
`auto`,
an unspecified attribute,
and any other value leave the file checked.
A package declares its exact-byte inputs in its own `.gitattributes`,
for example `seed/** -text` beside a fuzz target,
so the exemption moves with the package and no path list in this policy needs to follow it.
No package path is listed:
fuzz seeds (`forbidden-strings/cli.fuzz`,
`jsonc-edit/core-rust.fuzz`)
and the toml-edit fixture sources (`package/test-fixture/toml-edit/.gitattributes`)
are exempt through their own `-text` declarations.

The attribute is read from the state the candidates were listed from,
never from the worktree:

- Candidates of `git add`,
  of a direct command,
  and of a commit transaction are read from the private index,
  so a `.gitattributes` staged by the same command or commit applies to it,
  and an unstaged edit of one does not.
- Candidates of a landed commit are read from that commit's tree.
- Candidates of a manual push are read from the tree of the commit,
  or the pushed tip tree,
  that first lists each path.

Git resolves the attribute,
so nested `.gitattributes` files,
macros,
the repository's `info/attributes` and the configured global attributes file apply as they do for Git itself.
The attribute of a file is asked only when its bytes are not canonical,
in one batch for the whole lifecycle,
so a lifecycle whose files all end with one LF starts no attribute query.
A lookup that fails fails the policy:
the file is never assumed to be text or binary.

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

The harness that produced the older records in `perf/` (lifecycle,
concurrent commits,
manual push,
forbidden strings
and the reserve sweeps) and its `perf:*` tasks no longer exist;
a new measurement of those scenarios needs a new harness.
The `native:perf` task (`bin/native-perf.ts`) exists and measures the release executable on forwarded commands and on
commits (explicit-path and service,
unsigned and SSH-signed),
as "Latency requirements" in the README describes;
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
four sweeps (the first at revision `2ec229081`,
the other three at `d42a4fab3`,
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
- the btrfs controls pass on this host,
  which a release that is not an alpha waits for before it dispatches
  (`mise run //package/git-policy/cli:native:release:host-controls`):
  the container gate runs on an overlay filesystem,
  where each of those controls prints `BTRFS CONTROL NOT RUN`,
  passes,
  and leaves the container run no evidence they work on btrfs;
  a real release refuses to dispatch when one of them fails,
  when one prints that line for any reason but needing a filesystem that is not btrfs,
  when the run was stopped at the deadline it is given,
  or when the run holds no passing status line for one of the controls the gate requires;
  an alpha release dispatches without the controls
  (the owner's standing rule of 2026-11-11,
  "make the release pipeline avoid running any extra work for alphas"),
  and a `--dry-run` skips the controls and starts the workflow's dry run instead;
- the release executable builds with `cargo build --release --locked --bin monochromatic-git-policy-cli`;
- the `git` that `PATH` resolves first leads to `monochromatic-git-policy-cli`
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
