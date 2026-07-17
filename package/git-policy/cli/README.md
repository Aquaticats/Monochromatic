# cli-git

Git wrapper that enforces safety rules before forwarding to the real git binary.
The package also exposes a side-effect-free policy authoring API.

## Runtime

Cli-git requires Node `^22.18.0 || >=24.11.0`.
Installing the package exposes a shadowing `git` executable.
Put the package's `node_modules/.bin` directory before the real Git directory on `PATH`;
the wrapper resolves and forwards to the next real Git executable.

PATH shadowing covers only processes that resolve the `git` command through that `PATH`.
An absolute path to the real executable,
a `PATH` with the wrapper directory removed,
a GUI using an embedded Git library,
and direct libgit2 consumers bypass cli-git completely.
That bypass also skips startup transaction recovery,
trusted policies,
fixed transforms,
and post-commit auto-push.
Use a known absolute real-Git path only for deliberate bypasses such as fixture setup or forensic inspection.
Cli-git's own real-Git resolver rejects its package entry and package-manager shims that point back to that entry,
then follows PATH directory order and Windows `PATHEXT` extension order.

The package is prepared for npm distribution,
but registry publication is deliberately deferred to issue #358.
The tarball contains one `dist/final/node/index.mjs` artifact.
That self-contained static bundle is both the executable and the inert package-root import.

## Automatic linked-worktree state

Every forwarded Git invocation inside an effective repository observes linked-worktree administrative identities
before and after real Git runs.
This outcome-based comparison detects ordinary Git aliases and worktrees that Git retains after a command or
`post-checkout` hook returns nonzero.
When new linked worktrees exist,
cli-git snapshots the invoking worktree's paths selected by Git's standard ignore stack after real Git and its hooks
settle,
then installs that state into every new worktree.
A bare repository has an empty source set.
No repository-specific copy configuration exists.

The snapshot preserves regular files,
directories,
symbolic-link target text,
and portable permission bits.
File copies request copy-on-write and use Node's full-copy fallback when the filesystem cannot clone extents.
Registered worktree roots nested beneath an ignored source tree are excluded,
as are cli-git's private staging paths.
Source paths remain selected even when the destination branch does not ignore them.
Sockets,
FIFOs,
and device nodes fail the copy without being opened.

Existing destination entries are never overwritten.
An exact type,
mode,
link target,
and byte match is accepted;
a difference fails after retaining the Git-created worktree.
Cli-git compares the completed private stage back to the source before installation,
so source instability fails unless exact final equivalence remains.
A successful copy emits one summary line on stderr.
Copy-only failure exits `2`.
When real Git also failed,
cli-git preserves Git's numeric status,
or uses `1` when Git ended by signal without a numeric status.

Private stages live beside their destination for same-filesystem copy-on-write.
Durable journals and a recoverable process-identity lock live under
`<git-common-dir>/cli-git-worktree-copy/v1`.
A later cli-git invocation validates destination registration,
stage ownership,
permissions,
and path containment before resuming an interrupted installation.
Malformed or conflicting recovery state fails closed and remains available for diagnosis.

## Policy authoring API

Importing the package root does not inspect process arguments,
read files,
resolve Git,
write output,
or start the executable.
The identity helpers preserve their input objects so TypeScript retains concrete policy names,
plugin namespaces,
and Valibot output types.

```ts
import {
  defineConfig,
  definePlugin,
  definePolicy,
  definePolicyOptions,
} from '@monochromatic-dev/git-policy-cli';
import * as v from 'valibot';

const options = definePolicyOptions(v.object({
  requiredSuffix: v.string(),
}));

const suffixPolicy = definePolicy({
  name: 'suffix',
  defaultSeverity: 'error',
  warnSafe: true,
  triggers: ['direct-check'],
  options,
  check: async ({ options: parsedOptions }) => {
    void parsedOptions.requiredSuffix;
    return [];
  },
});

const examplePlugin = definePlugin({
  name: 'example',
  policies: [suffixPolicy],
});

export default defineConfig({
  plugins: {
    example: examplePlugin,
  },
  policies: {
    'example/suffix': ['warn', { requiredSuffix: '.ts' }],
  },
});
```

`defineConfig` rejects unknown statically known policy IDs and option values that do not match a policy's Valibot
output.
`ABSENT_GIT_VALUE` represents mutable candidate revisions,
missing object IDs,
and direct operations without a Git subcommand.
It is an in-process unique symbol and is never serialized to JSONL.

## Shipped optional policies

Repo-owned policies ship in the same package-root MJS artifact but remain disabled until trusted config registers them.
The current optional export is `repositoryPolicyPlugin`.

```ts
import {
  defineConfig,
  repositoryPolicyPlugin,
} from '@monochromatic-dev/git-policy-cli';

export default defineConfig({
  plugins: {
    mono: repositoryPolicyPlugin,
  },
  policies: {
    'mono/forbidden-root-context': 'error',
  },
});
```

Importing the package does not register this plugin,
load repository config,
or start the executable.

## Policy engine and management

The shadow executable runs configurable core policies through the unified policy contract before fixed command
transformers:
`require-root`,
`linked-worktree-only`,
`branch-worktree-only`,
`add-explicit`,
and `final-newline`.
All default to `error`.
Validated policy settings support `off`,
`warn`,
and `error`.
`branch-worktree-only` and `final-newline` are warn-safe;
configuring another current built-in as `warn` also produces a non-blocking `configuration-warning` JSONL event.
Repository-root `cli-git.config.mjs` or `cli-git.config.ts` can configure built-ins and register namespaced plugin policies
after exact-snapshot trust.
Built-ins run in the listed order.
Atomic push,
commit only,
and status hints off then run as fixed non-configurable transforms.
Trusted plugin policies follow in namespace and declaration order and receive both exact raw arguments and final
transformed arguments.
Expected commit-only rejections are `core-finding` JSONL events rather than configurable policy findings.

Policy findings and engine failures are compact LF-terminated JSONL.
Forwarded wrapper invocations write events to stderr;
direct management checks write them to stdout.
Exit code `0` means no blocking finding,
`1` means at least one policy finding remained,
and `2` means configuration,
usage,
or engine failure.
Events are buffered until the pass settles,
and sequence values start at zero for every invocation.

### Commit autofix transaction

For explicit-path commits,
`--no-only` index commits,
pathspec-file selections,
amend and allow-empty edge cases,
and merge,
cherry-pick,
or revert conclusions,
trusted pre-forward policies receive lazy candidate bytes from a locked private Git index.
A fixable finding returns one `git-unified` patch bound to the candidate's opaque target ID and repository path.
Cli-git validates that the patch contains exactly one matching ordinary text target,
writes the bytes to a private file,
and invokes real Git as `git apply --cached --3way` with `GIT_INDEX_FILE` pointing only at private state.
Policies cannot select an undeclared target or mutate the real index or worktree.

Any changed private index restarts the complete ordered policy sequence with a higher candidate version.
Provisional findings remain buffered;
only the final unchanged pass is rendered.
Exact repeated private-index bytes are a cycle,
and eight changed passes is the convergence limit.
Non-overlapping proposals compose sequentially;
a conflict blocks with exit `2` while its unmerged state remains private.

Explicit-path mode builds the intended tree from `HEAD` plus selected worktree paths,
then reconciles only those landed entries into a copy of the original index.
Explicit `--no-only` mode patches a copy of the complete real index.
The completed index is installed atomically only after real Git succeeds,
so policy failures,
patch conflicts,
and failed commit hooks leave real index and worktree bytes unchanged by cli-git.
Interactive and patch selection runs once through native Git against the copied private index;
include selection stages into that same private index.
Policies inspect the exact chosen private candidate without applying automatic patches:
canonical candidates commit the settled private index,
while a required correction blocks with direct-fix guidance.
Unmerged indexes block automatic correction.

Direct `git cli-git fix` uses the same eight-pass policy convergence against a disposable private index.
It accepts exactly one scope,
`--all` or pathspecs after `--`,
revalidates worktree bytes before installation,
uses same-directory atomic replacements with rollback copies,
and verifies that real index bytes remain exact.
Successful corrections emit only a `fix-summary` JSONL event.

A durable no-follow transaction directory retains exact original and prepared index snapshots,
expected parent and tree identities,
a private nonce-bearing reflog action,
process birth identity,
and exact directory,
index-artifact,
and real-index lock identities before reference advancement.
Every later wrapper invocation recovers before trusted config execution,
installs or recognizes the exact prepared index after an interrupted landed commit,
and fails closed on active owners,
reused PIDs,
replaced locks,
replaced or unsafe artifacts,
read-only transaction filesystem setup,
or unrelated reference/index movement.
Filesystem setup errors emit `content-unavailable` JSONL and leave exact ref,
index,
and worktree state unchanged.

Recovery is automatic rather than a separate management command.
Run the next ordinary cli-git invocation after the interrupted owner has terminated.
Before loading trusted repository code,
cli-git inspects the Git-provided `cli-git-transaction` administrative path and either restores the original index,
installs the prepared post-commit index,
recognizes an already completed installation,
or fails closed with the retained transaction path.
An active owner or conflicting ref,
reflog,
index,
lock,
artifact,
or filesystem identity produces exit `2` without discarding evidence.
Do not remove the retained directory merely to silence that diagnostic;
the preserved snapshots and journal are the evidence needed to distinguish an unlanded commit from a landed commit
whose index installation was interrupted.

Use the namespaced Optique management commands:

```sh
git cli-git trust
git cli-git trust --yes
git cli-git untrust
git cli-git status
git cli-git check --all
git cli-git check --policy require-root -- path/to/file
```

`status` reports whether repository config is absent,
untrusted,
trusted and unchanged,
changed,
or corrupt.
It does not execute live repository config.
This trust-status schema supersedes the temporary built-in policy inventory emitted by the first policy-engine slice.

Direct check requires exactly one scope source:
`--all` or at least one pathspec following `--`.
Policy filters accept registered built-in and trusted plugin IDs.
Git global options remain before the namespace,
for example `git -C /repo cli-git check --all`.

`check` and `fix` are cli-git commands and are never forwarded as Git subcommands.
Both require exactly one scope:
`--all` or pathspecs after `--`.
Repeated `--policy <id>` options narrow execution to named registered policies.
`check` reads exact worktree candidates and emits findings to stdout without changing worktree or index bytes.
`fix` converges privately,
revalidates each worktree source before installation,
atomically replaces only changed worktree paths,
and verifies that the real index remains byte-identical.
An untrusted or changed repository config blocks either command before plugin execution;
built-in policies remain available when repository config is absent.

## Exact MJS trust

Discovery is bounded to the effective repository root.
`cli-git.config.mjs` takes precedence over `cli-git.config.ts`,
and either path must be a regular non-symlink file.
Known inspection-only Git commands skip config loading.
`branch` and `tag` use argument-aware classification;
unknown or ambiguous commands take the config-loading path and therefore block on untrusted config.

First config-loading use exits `2` without executing repository code and points to `git cli-git trust`.
Trust validates UTF-8,
JavaScript syntax,
and module edges before consent.
Static and dynamic import syntax may name only Node built-ins;
local files,
packages,
computed dynamic imports,
and extra artifact assets are rejected.
A self-contained MJS file may export raw config data,
or a consumer build may bundle authoring helpers and plugin code into that one artifact.

Before prompting,
trust prints the canonical path,
complete filesystem identity and stability,
exact snapshot state and byte count,
retained Node built-ins,
and arbitrary-code authority.
`--yes` is explicit noninteractive consent for CI.
Without `--yes`,
noninteractive input declines with exit `2`.
Trusted code is not sandboxed:
it runs with full account file,
process,
network,
and Git authority after consent.
`--yes` changes only how consent is collected;
it grants the same authority as interactive consent.
Trust attaches to the disclosed canonical config path,
filesystem identity,
and exact bytes,
not to a branch name,
commit,
repository remote,
or signer.
Changing bytes requires explicit re-trust unless that exact identity was already trusted and selected for relaxed refresh.

The registry root comes from the operating-system account database rather than `HOME`,
XDG,
or AppData environment overrides.
Record keys reversibly encode the complete filesystem identity and canonical config path without hashing.
Registry directories and files reject symlinks,
unsafe ownership,
unsafe POSIX modes,
and unsafe Windows ACLs.
Candidate writes use an exclusive per-key lock,
private temporary directory,
fsync,
validated rename,
and rollback on incomplete replacement.

Every config-loading command captures the live file through a no-follow handle.
Linux resolves filesystem identity through the open process descriptor;
other hosts bracket path-based identity resolution with same-handle metadata and final live-path device and inode
agreement.
The command then compares exact live bytes with the stored snapshot and executes only the private stored copy.
Changed bytes exit `2` until explicit re-trust.

A config that validates with `trust: { children: true }` triggers a second disclosure and consent stage.
The disclosure names the exact repository root and states that recursive authority intentionally crosses filesystem
boundaries,
including current and future mounted volumes.
The first encounter with a strict descendant captures and validates exact bytes in private state,
records every unchanged recursive root that authorizes it,
and installs a stored executable snapshot without another prompt.
Recursive consent therefore delegates future exact-snapshot enrollment to every descendant repository,
even when that descendant is on another filesystem or a volume mounted later.
It does not execute live descendant bytes directly:
each descendant still passes syntax,
configuration,
stable-read,
private-storage,
and exact stored-snapshot checks before execution.
A filesystem replacement cannot reuse the prior record:
the changed filesystem identity requires a fresh exact auto-enrollment while an unchanged recursive root still
authorizes the path.
Changed descendant bytes block until explicit re-trust.

`untrust` removes inherited descendant authority through a recoverable registry transaction.
Separately explicit descendant trust survives outer-root removal.
Untrusting a nested recursive root also revokes outer recursive roots that authorize it and discloses every affected root
before mutation.
If a root config was deleted,
`untrust` recovers its stored records from the canonical repository root.

When root `cli-git.config.mjs` is absent,
cli-git discovers `cli-git.config.ts`.
Explicit trust lazily imports Rolldown and calls its public bundle API directly with
Node ESM output and `codeSplitting: false`.
The disposable bundle is closed explicitly after in-memory generation,
so ordinary Git commands never initialize Rolldown and trust builds do not retain native workers.
It accepts one JavaScript chunk only,
captures the entry and every statically resolved relative local source through stable no-follow reads,
and rechecks the complete graph after build completion.
Bare package code is bundled but excluded from automatic invalidation,
so trust names each such package warning.
Strict commands compare every tracked source's exact bytes and execute only the private stored bundle.

`CLI_GIT_NO_PARANOID` relaxes only exact identities that were already trusted.
Its comma-separated entries use `<filesystem-id>:<canonical-path>`;
encode percent as `%25` and comma as `%2C` (case-insensitive on input).
Malformed entries and current-path entries with another filesystem identity emit JSONL warnings and retain strict mode.
For an exact relaxed MJS path,
source size or mtime changes trigger private snapshot validation and replacement.
For an exact relaxed TypeScript path,
tracked source size or mtime changes trigger a private rebuild and replacement.
Refresh failure blocks with exit `2` and retains the previous record;
metadata signals never waive first trust.

The maintained packed-bin integration check builds the unpublished tarball,
installs its shadowing `git` executable in a bounded disposable container,
and exercises untrusted blocking,
trust,
status,
stored plugin checks,
changed bytes,
two-stage recursive consent,
cross-filesystem enrollment,
mount replacement,
concurrent enrollment and revocation,
and cascading untrust:

```sh
mise run //package/git-policy/cli:test:built:trust
```

Every built-in accepts a generic one-invocation escape named `--no-enforce-<policy-id>`.
The existing `--no-enforce-worktree`,
`--no-enforce-worktree-branch`,
and `--no-enforce-bulk-add` aliases remain supported.
Escapes skip that policy for the complete invocation and are stripped before real Git runs.
They are recognized only in flag position before Git's `--` pathspec separator;
option values and pathspecs with the same bytes remain untouched.
`--cli-git-keep-going` is also wrapper-only and preserves fixed policy order while allowing later finding-producing policies to run.
Engine failures always stop immediately.

Git's `--no-verify` is not a cli-git bypass.
Cli-git policies and commit transaction processing run before forwarding,
and post-commit policy gating and auto-push run after successful real Git.
The option is forwarded unchanged so real Git can skip only the `pre-commit` and `commit-msg` hooks assigned by
[Git's `git commit` documentation](https://git-scm.com/docs/git-commit/2.55.0).
Use the policy-specific `--no-enforce-<policy-id>` escape when an enabled policy permits an explicit one-invocation
bypass.
Fixed transforms retain their own named controls,
including `--no-atomic`,
`--no-only`,
and `--no-enforce-only`.

Issue `#357` retired hk and Pkl.
A machine with stale `hook.hk-*` Git configuration can follow
`doc/runbook/remove-retired-hk-git-config.md` and run the explicit `cleanup:hk-git-config` task.
The task removes only requested local or global hk namespaces;
disposable fixtures prove unrelated hook keys remain unchanged.

## Rules

**Require root**:
 when the effective working directory is inside a git
repository,
 rejects commands unless that directory is the repository root
(where `.git` lives).
 `.git` may be a directory or a file,
 so linked worktrees
and submodules are checked the same way normal repositories are.
 The effective
directory is `process.cwd()` with every pre-subcommand `-C <path>` applied in
order,
 matching git's own resolution,
 so `git -C /repo status` from anywhere
passes the check.
 When no `.git` is found up the tree from the effective
directory,
 the command passes through to real git untouched,
 so git itself
reports the missing-repo error if relevant.
 Exempt subcommands:
 `init`,
`clone`,
 `version`,
 `help`,
 and `config` with `--global`/`--system`/`--list`.

**Linked worktree only for risky worktree commands**:
 rejects guarded commands
unless the effective working directory is a linked git worktree root.
 Guarded
forms are all `git stash`,
 state-changing `git clean`,
 and destructive
`git reset` modes (`--hard`,
 `--merge`,
 `--keep`).
 This blocks the main
worktree and `--git-dir` / `--work-tree` forms launched from unrelated
directories,
 because these commands can revert,
 delete,
 or rewrite filesystem
state outside what the caller expects from current cwd.
 `git clean -n` and
`git clean --dry-run` pass through because Git documents them as inspection
only;
 `git clean -i` remains guarded because interactive mode can delete
selected paths.
 Clean dry-run,
 no-dry-run,
 interactive,
 and no-interactive
options are evaluated in argv order to match Git's last-option-wins behavior.
Run guarded commands from a linked worktree root,
 or pass
`-C <linked-worktree-root>` before the subcommand.
 Pass
`--no-enforce-worktree` after the guarded subcommand and before any `--`
pathspec separator to bypass linked-worktree enforcement for one invocation;
the wrapper strips the flag before forwarding to real git.
 The `require-root` policy still rejects linked-worktree subdirectories.

Repositories under a baked-in tool-cache directory are exempt from this rule.
A third-party tool (currently uv,
 whose git cache resolves from `UV_CACHE_DIR`,
`$XDG_CACHE_HOME/uv`,
 or `~/.cache/uv`) owns disposable clones there and runs
destructive git against them itself,
 so `git reset --hard` and the other
guarded forms pass through instead of being rejected as a main worktree.
 The
exempt set is `DEFAULT_ALLOWED_WORKTREE_DIRS` in `src/allowed-worktree-dirs.ts`;
both the configured directory and the repository's git-dir are realpath-resolved
before a segment-aware containment check,
 so symlinks such as `/home` aliasing
`/var/home` do not defeat the match.
 Add sibling tool caches there when a new
tool needs the same exemption.

**Branch worktree only**:
 rejects branch creation in the current worktree and
redirects to creating the branch together with its own linked worktree.
 Guarded
forms are `git branch <name>`,
 `git branch -c` / `git branch -C` /
`git branch --copy`,
 `git checkout -b` / `git checkout -B`,
`git checkout --orphan`,
 `git checkout --track`,
 `git switch -c` /
`git switch -C`,
 `git switch --create` / `git switch --force-create`,
`git switch --orphan`,
 and `git switch --track`.
 The wrapper also
rejects the implicit remote-tracking branch guess that `git switch <name>` and
`git checkout <name>` perform when exactly one matching remote branch exists and
no local branch exists.
 Listing,
 deletion,
 rename,
 upstream edits,
 detached
checkouts,
 path checkouts,
 and `git worktree add -b <branch> <path> [<start-point>]`
pass through.
 Pass
`--no-enforce-worktree-branch` after the guarded subcommand and before any `--`
pathspec separator to bypass for one invocation;
 the wrapper strips the flag
before forwarding to real git.

**Add explicit**:
 rejects `git add` invocations that use bulk-staging
patterns (`.`,
 `./`,
 `*`,
 `:/`,
 `-A`/`--all`,
 `-u`/`--update`),
 which sweep
up paths the caller did not intend to stage and leave the index in a state
that does not match a single logical change.
 Pathspecs after `--` are still
scanned for broad pathspecs such as `.` and `*`,
 so `git add -- .` is rejected
for the same reason as `git add .`.
 Name the paths explicitly,
 or pass
`--no-enforce-bulk-add` to bypass for one invocation;
 the flag is stripped
before forwarding to real git.
 The policy walks pre-subcommand global options
the same way atomic-push does.

**Atomic push**:
 injects `--atomic` into `git push` commands automatically,
ensuring all refs update together or none do.
 Override with `--no-atomic`.
The rule walks pre-subcommand global options (`-C <path>`,
 `-c key=val`,
`--git-dir <path>`,
 etc.) so forms like `git -C /repo push` still fire.

**Commit only**:
 injects `-o` (a.
k.
a.
 `--only`) into `git commit` commands so
every commit must name the paths it includes rather than picking up whatever is
staged.
 `git commit -m <msg>` without paths is rejected by the wrapper before
git can emit its opaque `No paths with --include/--only` fatal.
 `git commit -a`
and `git commit --all` are rejected because they stage tracked modifications
implicitly.
 Pathless commits remain valid when git permits them with `--amend`,
`--allow-empty`,
 or `--pathspec-from-file`.
 Because a pathless only-mode commit reuses HEAD's existing tree (git documents
that `git commit --only` with no pathspec ignores the index),
 pathless `--amend`
and `--allow-empty` forms get one more check before `-o` is injected:
 when the
index differs from HEAD,
 the wrapper rejects the command instead of letting an
injected `-o` silently ignore the staged changes,
 which would otherwise turn
`git add <path>` followed by `git commit --amend --no-edit` into a silent no-op
(new commit hash,
 old tree,
 change left staged,
 no warning).
 The rejection
message names the explicit choices:
 pass pathspecs to include those paths
(`git commit --amend <path>`),
 pass `--only` to proceed without them,
 or pass
`--no-only` to commit the entire index.
 When the index matches HEAD,
 or git
cannot answer (for example before the first commit),
 the pathless form passes
through with `-o` injected as before.
 Skipped when `-o`,
 `--only`,
 or
`--no-only` is already present (the user has made an explicit choice),
 and
when `-i`/`--include` (any accepted abbreviation) is present,
 because git
forbids combining include mode with `--only` and the user already chose how
paths combine with the index.
 Pathless commits during a merge,
 cherry-pick,
or revert conclusion pass through without `-o`:
 git forbids partial commits
in those states ("cannot do a partial commit during a merge"),
 so the
pathless `git commit` that records the resolution works unchanged instead of
being rejected with advice that would dead-end.
 The pathless rejection
message also names `--no-only` as the commit-the-entire-index choice.
 Escape
hatch for a single invocation:
 pass `--no-enforce-only`,
 which is stripped
before forwarding to real git.
 Pathspec detection uses a scanner for known
separated-value commit options,
 so no-value flags such as `-q` and `--dry-run`
do not consume the following pathspec while wrapper-only validation runs.
 The
rule walks pre-subcommand global options the same way atomic-push does.

**Status hints off**:
 injects `-c advice.statusHints=false` before `git status`
so git suppresses its stock hints,
 which suggest patterns the wrapper rejects
(`git add` with bulk patterns,
 `git commit -a` colliding with commit-only's
`-o`).
 A cli-git note prints after the status output describing the wrapper's
constraints.
 Skipped when the user has already set `advice.statusHints` via
`-c` (the user's explicit choice wins);
 both the valued form
(`-c advice.statusHints=true`) and git's bare boolean-true form
(`-c advice.statusHints`) count,
 matched case-insensitively the way git
matches config keys.

## Post-commit auto-push

After a successful real `git commit`,
the wrapper resolves the exact landed commit OID before backup.
Trusted `post-commit` plugin policies receive that landed OID and lazy landed-delta candidates:
only the files that commit changed against its parents,
never the unchanged remainder of the tree.
A clean or warning-only result permits automatic push.
An error finding,
policy exception,
or post-commit setup failure blocks push,
returns exit `2`,
leaves the commit intact,
and emits JSONL ending with `commit-landed` and the exact OID so automation does not retry the commit blindly.
A matching full-lifecycle policy escape skips that plugin's post-commit gate.

The wrapper then backs up the permitted new commit by pushing it.
Dry runs do not run post-commit policies or push:
 besides `--dry-run` itself,
 git documents
`--short`,
 `--porcelain`,
 `--long`,
 and `-z`/`--null` as implying a dry run,
and the wrapper recognises all of them (in any accepted long-option
abbreviation) via the same parsed commit region the commit-only rule uses.
 A
branch that already has a configured upstream is pushed with a plain
`git push`,
 following that upstream wherever it lives;
 only a branch with no
upstream yet is pushed with `git push --set-upstream origin HEAD`,
 so a
branch tracking another remote never has its tracking configuration silently
re-pointed to origin.
 The push runs against the real git binary
in the same directory the commit landed in (respecting pre-subcommand
`-C <path>`),
 so it does not re-enter the wrapper yet still fires git's native
pre-push hook.

The push output is filtered:
 on a clean push only the GitHub `remote:` lines are
shown;
 on a failed push the full output is shown so a rejection,
 a pre-push
block,
 or an offline error stays diagnosable.
 Auto-push is skipped silently
when there is nowhere to back up to (no upstream and no `origin` remote),
 and
skipped with a printed note when HEAD is detached (mid-rebase,
mid-cherry-pick,
 or a detached checkout),
 where pushing `HEAD` cannot name a
branch and previously produced a confusing guaranteed failure.

Because git ignores a post-commit hook's exit status,
 a failed backup push is
surfaced but never changes the commit command's own exit code:
 the commit stays
saved locally and a later `git push` retries it.

## Performance gates

Run the packed lifecycle gate with:

```console
mise run //package/git-policy/cli:perf:lifecycle-latency
```

The task builds an unpublished npm tarball and runs it in a disposable Podman container
limited to 2 GiB RAM and 2 CPUs.
The fixture contains 2,048 tracked files so accidental repository-wide candidate reads remain observable.
Each scenario validates six warm-up samples,
then records 30 samples with median,
p95,
median absolute deviation,
raw timings,
and paired wrapper-added latency for every scenario.

The matrix covers no-config forwarding,
read-only commands,
strict MJS and TypeScript snapshots,
relaxed TypeScript rebuilds,
config validation,
scanner checks,
clean and changed final-newline paths,
and post-commit policy work.
`perf/lifecycle-latency-2026-07-16.json` stores the measured baseline.
Each enforced ceiling is twice its baseline maximum rounded up to the next 25 milliseconds;
every ceiling remains below 2,000 milliseconds.

## How it works

The wrapper shadows the system `git` binary on PATH (via mise bin linkage).
It scans PATH to find the real git binary,
 skipping its own entry.
Self-shim detection checks both the package name and the bundled entry path
`package/git-policy/cli/dist/final/node/index.mjs`,
 because pnpm-generated shims can
point at the built file without naming the package.
Arguments pass through built-in policy,
fixed-transform,
and trusted-plugin stages;
the real git is then spawned with exactly the final transformed arguments and full stdio inheritance.
 After a successful
commit,
 the new commit is auto-pushed to origin as described above.

## Adding rules

Rules are functions with the signature
`(args: readonly string[]) => readonly string[] | Promise<readonly string[]>`.
A rule can return modified args or throw to reject the command.
Add new rules to the `RULES` array in `src/index.ts`.
