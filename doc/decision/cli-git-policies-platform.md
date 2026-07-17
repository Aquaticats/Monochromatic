# cli-git pluggable Git policies platform

## Status

Accepted.
Implementation is tracked by GitHub issues #342 through #357.
The public package is prepared for npm distribution,
but registry publication remains indefinitely deferred in #358.

The canonical implementation interface is
`package/git-policy/cli/SPEC.md`.
The execution record is
`doc/handover/cli-git-policies-platform.md`.

## Decision

Turn `package/git-policy/cli` into a pluggable Git policies platform modeled on Oxlint.
Keep the existing shadowing `git` executable as the enforcement point.
Expose configurable built-in and third-party policies,
fixed command transforms,
trusted repository configuration,
direct check/fix management commands,
and stable JSONL findings.

Build the platform first,
migrate the existing cli-git and hk behavior onto it,
then retire hk and Pkl as the capstone.
Pkl leaves with hk because this repository uses Pkl only for hk configuration.

The platform must work without mise.
Its package,
configuration,
trust registry,
and management commands therefore cannot assume mise is installed.

## Goals

- Preserve and make configurable the current Git safeguards.
- Provide a small policy-authoring interface for repo-local and distributed plugins.
- Keep ordinary real-Git stdout intact.
- Fail closed when an enabled policy cannot complete.
- Apply commit normalizers without staging unrelated worktree bytes.
- Bootstrap every newly registered linked worktree with the invoking worktree's ignored filesystem state.
- Execute only explicitly trusted,
  exact stored config artifacts.
- Provide an npm-ready Node package and shadowing `git` bin.
- Keep CI enforcement independent from local wrapper trust.

## Non-goals

- Sandboxing trusted JavaScript.
- Making PATH shadowing impossible to bypass.
- Replacing CI as the authoritative enforcement layer.
- Loading native plugins in the first release.
- Shipping standalone native executables in the first release.
- Publishing the package to npm as part of the active implementation sequence.

## Why a PATH wrapper

The wrapper already intercepts Git and can transform command arguments.
That supports behavior that native hooks cannot express,
including atomic push,
commit-only,
and status-hint transforms.
PATH placement also avoids a separate hook installation step.

The limitation is explicit:
absolute real-Git paths,
PATH order changes,
GUI clients using libgit2,
and other non-wrapper Git implementations bypass local policy enforcement.
Native hooks have different bypasses,
including `--no-verify` and missing installation.
Local enforcement remains best-effort fast feedback;
CI remains authoritative.

## Linked-worktree ignored state

Use the PATH wrapper's complete real-Git lifecycle to synchronize local ignored state into every newly registered linked
worktree.
Do not classify only literal `worktree add` argv.
Compare the effective repository's linked administrative identities before and after forwarding so ordinary aliases and
worktrees retained after Git or hook failure are covered.
Snapshot after real Git and `post-checkout` settle.

Git's standard ignore stack is the sole source selector.
Do not add repository-specific configuration.
Bare repositories contribute an empty source set.
Exclude nested registered worktree roots and cli-git's reserved private stages from recursive source traversal.
Copy regular files,
directories,
and symbolic links while preserving permission bits and link target text.
Request same-filesystem copy-on-write with full-copy fallback.
Reject special filesystem entries without opening them.

Never overwrite destination state.
Accept exact existing entries,
including source-selected paths the destination branch does not ignore.
A differing destination entry fails after retaining the new worktree.
Require exact final source-to-stage equivalence before installation;
perfectly reverted transient mutations are outside the portable guarantee.

Use private sibling staging,
a common-directory journal,
a recoverable process-identity lock,
and ownership-checked rollback.
Recovery must validate path containment,
private ownership,
and linked registration before resuming or deleting state.
Copy-only failure is exit `2`.
When Git also failed,
preserve its numeric status or use `1` for a signal without a numeric status.
Write one stderr summary after every successful new-worktree synchronization.

This behavior is fixed lifecycle infrastructure rather than a configurable policy.
It must work before trusted repository code and without mise.

## Policy module

A policy is one unified module.
It may validate command facts,
scan candidate bytes,
and return canonicalizing patches in the same run.
The earlier mutually exclusive validator,
scanner,
and normalizer kinds are rejected.

A policy declares:

- a policy-local name;
- a default severity;
- whether `warn` is safe;
- a Valibot option schema when it accepts options;
- applicable lifecycle triggers;
- one asynchronous check function.

A policy returns structured findings.
A finding may be command-level,
path-located,
or fixable.
A fixable finding carries unified Git patch bytes for exactly one located path.
Expected policy violations are findings.
Thrown exceptions,
invalid options,
incomplete checks,
invalid patches,
and unavailable required content are engine failures.
Engine failures block immediately with exit code `2`.

The policy context exposes immutable cheap command facts directly.
Expensive Git-derived facts use lazy asynchronous methods.
Lazy values are memoized for one candidate-state version only.
Applying a patch invalidates candidate-dependent memoized data before another policy runs.

Exact public TypeScript declarations and invariants live in
`package/git-policy/cli/SPEC.md`.

## Policy identifiers and configuration

Built-in policy IDs are unprefixed.
Plugin IDs use `<consumer-namespace>/<policy-name>`.
The consumer chooses each plugin namespace in its config.
Namespace registration order and each plugin's declaration order are significant.

`defineConfig` is a typed identity helper.
Cli-git does not merge shared and local configuration.
Consumers that need inheritance merge ordinary JavaScript objects before calling `defineConfig`,
preferably with `deepmerge-ts` or explicit merge logic.
Collection merge semantics remain visible consumer code rather than hidden cli-git behavior.

Every registered policy uses its declared default when omitted from `policies`.
Registering a plugin therefore activates that plugin's declared defaults.
An explicit `off` persistently disables a policy.
A configured policy value is either a severity or a severity-plus-options tuple.

## Severity and warn safety

Severity is `off`,
`warn`,
or `error`.

- `off` skips the policy.
- `warn` emits findings but does not block real Git.
- `error` blocks when an error finding remains after convergence.

An explicitly configured unsafe `warn` remains valid,
but config loading emits a non-blocking `configuration-warning` event in the same JSONL stream as policy findings.

Warn-unsafe policies are:

- `require-root`;
- `add-explicit`;
- `linked-worktree-only`;
- `forbidden-strings`.

Warn-safe policies are:

- `branch-worktree-only`;
- `forbidden-root-context`;
- `final-newline`.

All current safety policies default to `error`.

## Policy and transform order

Each pass uses this fixed staged order:

1. Built-in configurable safety policies in fixed core order.
2. Fixed idempotent command transformers.
3. Plugin policies in consumer namespace registration order and plugin declaration order.

The built-in order is:

1. `require-root`.
2. `linked-worktree-only`.
3. `branch-worktree-only`.
4. `add-explicit`.
5. `final-newline`.

The fixed transformer order is:

1. atomic push;
2. commit only;
3. status hints off.

Post-commit policies now inspect exact landed OID and tree facts before auto-push.
Error findings and engine failures retain the commit,
block backup,
return `2`,
and emit explicit landed-commit JSONL.
Auto-push remains a fixed side effect only after that gate settles clean or warning-only.

Core policies inspect raw and semantic command facts.
Plugin policies receive raw and transformed command facts and predict candidate content from the transformed command.
Severity and option overrides do not reorder policies.
Policies execute sequentially.
A plugin that needs parallel internal checks owns that concurrency.

The default stops at the first remaining error finding.
`--cli-git-keep-going` continues after policy findings so later findings can be collected,
but real Git still does not run while any error remains.
The flag never continues after an engine,
plugin,
patch,
or transaction failure.
Cli-git does not consume Git's own generic `--keep-going` options.

## Escape hatches

A per-invocation policy escape hatch is `--no-enforce-<policy-id>` in flag position.
Cli-git strips it before forwarding.
A token after Git's `--` pathspec separator remains a pathspec rather than an escape hatch.
The escape applies to the complete invocation lifecycle:
findings,
fixes,
post-commit checks,
and auto-push gating.
Persistent disable uses config severity `off`.

`git commit --no-verify` does not skip cli-git policies after hk retirement.
Documentation must state this behavior change.

## Distribution

Prepare `@monochromatic-dev/git-policy-cli` as a public npm-ready Node package.
The package installs the shadowing `git` bin and exports side-effect-free authoring modules from the same package.
Importing authoring helpers must not execute CLI startup code.

Public authoring exports include:

- `defineConfig`;
- `definePlugin`;
- `definePolicy`;
- Valibot-backed option helpers;
- policy,
  finding,
  patch,
  config,
  trigger,
  and context types.

The public API uses one exported unique `ABSENT_GIT_VALUE` symbol for mutable revisions,
missing object IDs,
and command forms without a subcommand.
This represents domain absence without nullable unions and is never serialized to JSONL.

Repo-owned policies retain separate workspace source packages but are bundled into the single public cli-git MJS
artifact and exported from `@monochromatic-dev/git-policy-cli`.
Importing a shipped policy does not register or enable it;
consumer config must register the plugin under a chosen namespace.
The public tarball contains every runtime needed by those exports except explicitly documented external executables
such as the `forbidden-strings` scanner.
The same MJS artifact is both the side-effect-free import target and the executable bin;
CLI startup runs only when Node invokes that artifact directly.
Cli-git's own module graph has no package-relative dynamic imports or secondary chunks.
Dynamic imports retained by bundled libraries are exempt.
The exact stored-MJS execution boundary may retain one computed dynamic import to support top-level `await`.

Third-party plugins peer-depend on a compatible cli-git package version.
Private workspace runtime helpers are bundled into the Node artifact rather than left as unresolved registry
requirements.
The Node engine range is the package's explicit runtime contract;
build tooling remains a development dependency.

The package must pass `npm pack` inspection and installation in a disposable non-workspace consumer.
Registry upload,
registry authentication,
publish-workflow enablement,
and provenance publication belong only to indefinitely deferred issue #358.

## Management commands

Cli-git intercepts one namespaced Git subcommand:

```text
git cli-git trust [--yes]
git cli-git check [--policy <id>]... (--all | -- <pathspec>...)
git cli-git fix [--policy <id>]... (--all | -- <pathspec>...)
git cli-git untrust
git cli-git status
```

Management parsing uses Optique.
`check` and `fix` require exactly one scope source:
`--all` or one or more pathspecs after `--`.
A repeated `--policy` filters the enabled set.
An empty filter means every enabled policy.

`trust`,
`untrust`,
and `status` inspect or recover trust without first executing repository config.
`status` reports stable exact-trust state rather than executing config to inventory policies.
`check` and `fix` load only trusted config.
Ordinary Git commands continue to invoke the resolved real Git executable by absolute path.

## Configuration discovery

Discovery is bounded to the effective Git repository root.
Precedence is:

1. `cli-git.config.mjs`;
2. `cli-git.config.ts`;
3. built-in policies only.

Keeping both files is valid;
MJS wins.

A consumer-built MJS artifact must be self-contained except for Node built-in imports.
Static imports,
re-exports,
and literal dynamic imports may name only Node built-ins;
computed dynamic imports and additional artifact assets are rejected.
Consumer source may import cli-git authoring helpers and plugins,
but those imports must be bundled into the runtime artifact.
A directly hand-written artifact exports raw validated data or inlines helpers.
Trusted config is not sandboxed and runs with the user's full permissions.

For TypeScript source,
`git cli-git trust` lazily imports Rolldown and invokes its public bundle interface directly with
Node ESM output and `codeSplitting: false`.
The runtime closes the disposable bundle after in-memory generation,
including failure paths.
Trust accepts exactly one JavaScript output chunk,
no unresolved non-Node imports,
and no extra assets.
Ordinary Git commands never build untrusted TypeScript.

The stored TypeScript invalidation graph covers:

- `cli-git.config.ts`;
- statically resolved relative local modules.

It excludes lockfiles and bare package or workspace imports.
Trust warns about those excluded package imports.
Explicit re-trust always rebuilds and is the refresh path for them.
Changed cached bundle bytes are disclosed before replacement.

## Config-loading classification

Known read-only commands skip config loading.
Mixed commands such as `branch` and `tag` use argument-aware classification.
Unknown aliases,
external subcommands,
future Git commands,
and ambiguous forms take the config-loading path.

An untrusted or changed config blocks a config-loading command and directs the user to
`git cli-git trust`.
Cli-git does not continue with built-ins only.
There is no global config-discovery kill switch.

## Exact-snapshot trust

Trust stores and compares exact bytes.
It never uses SHA-256 or another content hash for trusted source,
artifacts,
registry keys,
or candidate-state comparisons.

The trust identity is the complete pair of filesystem ID and canonical config path.
The registry path uses reversible encoding of that complete identity rather than a digest.
Trust records keep validated metadata and exact snapshot files in one per-key directory.
MJS capture opens the no-follow source handle before filesystem identity resolution.
Linux resolves identity through that process descriptor;
other hosts require same-handle metadata and final live-path device and inode agreement around path-based resolution.
After a successful exact-byte comparison,
cli-git executes the stored MJS snapshot or stored TypeScript bundle,
not the live entry file that was compared.
This closes the entry-file compare-then-swap window.
It is not a sandbox:
trusted code retains ambient Node authority and may deliberately read or dynamically load other live files.

Production registry location derives from the operating-system account home rather than repository-controlled
`HOME`,
XDG,
or AppData environment variables.
Tests inject a registry root through internal adapters.
Registry replacement is atomic,
rejects symlinks and junctions in registry ancestry,
requires current-account ownership and private POSIX modes,
and applies and verifies protected Windows ACLs on directories and files.

First execution always requires explicit trust.
A later covered byte change blocks until re-trust unless that exact path is in relaxed mode.
Trust,
config,
and plugin failures exit `2`.

## Trust consent

Trust has two consent stages because recursive intent can be learned only by executing authorized config.

Before root consent,
`git cli-git trust` prints:

- config path and format;
- filesystem identity;
- exact source and artifact snapshot changes;
- TypeScript self-containment or excluded-import warnings;
- arbitrary-code authority;
- notice that recursive intent is evaluated only after root execution is authorized.

The disclosure states that trusted code may read and write files,
run programs,
access the network,
automatically modify Git content,
and behave incorrectly despite transaction safeguards.

Root approval remains in memory until stored-artifact execution and validation succeed.
Failure leaves no persistent record.
If validated config declares child trust,
cli-git prints a second disclosure naming the root and descendant authority and requests separate consent.
`--yes` prints both applicable disclosures and skips input reads.
CI uses this explicit form and receives no detected-CI auto-trust.

## Relaxed exact-path mode

`CLI_GIT_NO_PARANOID` is a comma-separated list of exact
`<filesystem-id>:<canonical-path>` entries.
Percent escaping protects comma and percent characters.
Each decoded entry splits on its first colon because filesystem IDs are colon-free and Windows paths may contain later
colons.

Malformed or suspicious entries emit a prominent warning,
are ignored,
and leave that path in strict mode.
They never waive first trust.

For a previously trusted MJS path in relaxed mode,
a size or modification-time change triggers private snapshot replacement,
self-containment checks,
and config validation without renewed consent.
Failure retains the previous record and exits `2` for that invocation.
Unchanged metadata intentionally continues executing the previous stored snapshot.

For a previously trusted TypeScript path in relaxed mode,
entry or tracked-relative-module size or modification-time changes trigger automatic rebuild on the next config-loading
command.
Metadata is only a cache signal.
The new bundle is stored and used only after a successful build and config validation.
Build failure exits `2`.
Package-only changes remain outside automatic invalidation and require explicit re-trust.

## Recursive trust

Config declares recursive authority with:

```ts
trust: {
  children: true,
}
```

The second consent authorizes descendant configs under the exact canonical repository root.
Authority intentionally crosses filesystem and mount boundaries,
including future mounts beneath the root.
The disclosure states that consequence.

First descendant encounter verifies each authorizing root still has its trusted identity and exact bytes,
then auto-enrolls and stores an exact descendant snapshot without another prompt.
A mount replacement cannot reuse a prior filesystem-identity record;
it receives a fresh snapshot only while an unchanged recursive root still authorizes that path.
Later descendant byte changes block.
Records preserve authorizing-root provenance.
Untrusting a recursive root removes records inherited only from that root.
Separately trusted descendants remain trusted.

Untrusting a nested recursive root also revokes every outer recursive root currently authorizing that nested path.
Cli-git lists every affected root before revocation.
The cascade intentionally removes inherited authority from sibling subtrees and avoids persistent deny-boundary state.

## Findings and JSONL

Policy findings use stable JSON Lines.
The schema version is frozen in
`package/git-policy/cli/SPEC.md`.
Every finding includes a human-readable message.
Public output contains only findings from the final stable pass.
Changed-pass findings are provisional and never emitted as authoritative results.

Wrapper-mode policy events go to stderr so real Git stdout remains intact.
Direct `check` and `fix` policy events go to stdout.

When real Git did not run:

- exit `0` means success or warning findings only;
- exit `1` means one or more error findings;
- exit `2` means trust,
  config,
  plugin,
  patch,
  transaction,
  or engine failure.

A forwarded command normally preserves real Git's exit code.
If a commit lands and a post-commit policy or engine failure follows,
cli-git returns `2`,
leaves the commit intact,
blocks auto-push,
and emits a landed-commit event containing the new commit OID.
Callers must not retry the commit blindly.
An ordinary auto-push network or remote failure is not an engine failure:
cli-git surfaces complete push output,
leaves the commit local,
and preserves the successful commit command's exit code `0`.

## Patch ownership and validation

A fixable finding carries unified Git patch bytes against the candidate bytes supplied to the policy.
Each patch addresses exactly the finding's located path.
A multi-file policy returns one finding per path.
Plugins never choose or write temporary patch files.

Cli-git writes bytes to private temporary files and applies them with Git three-way semantics against private candidate
state.
It rejects traversal,
renames,
mode changes,
submodules,
unexpected paths,
and unsupported binary patches.
Conflicts remain in temporary state and exit `2`.

## Automatic and direct fixes

Matching pre-forward commit normalizers automatically apply fixes.
Trust consent authorizes that mutation capability.
Push and direct check are read-only.
Direct fix modifies selected worktree files but leaves every real index blob unchanged.
Direct check and fix require explicit pathspecs or `--all`.

For unsupported commit modes such as `--patch`,
`--interactive`,
or `--include`,
cli-git checks the candidate without mutation.
It proceeds when content is already canonical and blocks with direct-fix guidance only when correction is required.

## Whole-sequence convergence

A patch proposal ends the current pass before any later policy runs.
After cli-git applies the patch to private state,
all findings from that pass remain provisional and execution restarts from the first built-in policy.
A later policy therefore runs only against candidate state containing every earlier applied patch.

Convergence uses at most eight complete policy passes.
Candidate states compare exact ordered path-and-byte content.
Repeated non-adjacent states are cycles.
The implementation streams retained private snapshots for comparison and does not keep duplicate complete states in
process memory.

A stable state with remaining errors exits `1`.
A cycle or changing eighth pass exits `2`.
Only final stable-pass findings are emitted.

## Transactional commit gate

The transaction is proven through packed shadow-bin disposable repositories for explicit-path and `--no-only`
commits,
pathspec files including stdin and NUL forms,
selected deletion and untracked paths,
amend,
allow-empty,
and merge,
cherry-pick,
or revert conclusions.
It covers exact lazy candidates,
private three-way patch application,
whole-sequence restart,
composition,
conflict blocking,
failed hooks,
and exact real-state preservation.
Interactive and patch selection runs once through native Git against the copied private index;
include selection stages into that private index.
Policies inspect the exact chosen candidate read-only and block only when correction is required.
Plugins never mutate the real index or worktree through the policy interface.
Cli-git holds the real index lock,
uses private indexes,
and journals the non-atomic gap between Git's reference update and real-index replacement.

Index commits copy the real index,
apply patches through `GIT_INDEX_FILE`,
run real Git against that index,
and install the result only after success.
Explicit-path commits build a commit index from `HEAD` plus selected worktree paths,
then build a post-commit index from the original real index plus selected paths from the landed commit.
Merge,
cherry-pick,
and revert conclusions use index semantics only.

Patch conflict,
unmerged state,
or commit failure leaves the real index and worktree unchanged by cli-git.
Before ref advancement,
a durable no-follow journal records exact original and prepared indexes,
expected parent and tree identities,
owner PID plus process-birth identity,
a private nonce-bearing reflog action,
and exact directory,
prepared-index,
and real-index-lock device/inode identities.
Startup recovery runs before trusted config,
stabilizes and installs owned artifacts through verified hard links,
installs or recognizes exact prepared state after interruption,
and fails closed on active owners,
replaced locks,
replaced or unsafe artifacts,
read-only filesystem setup failures,
or unrelated ref/index movement.
Setup failures emit stable `content-unavailable` JSONL and preserve exact ref,
index,
and worktree state.
Required fixtures are enumerated in the implementation spec.

## Repository policy migration

### Built-in safeguards

These configurable built-ins now run through one fixed-order policy registry with behavior parity:

- `require-root`;
- `linked-worktree-only`;
- `branch-worktree-only`;
- `add-explicit`.

Their persistent severities and complete-invocation escapes are resolved before any policy executes.
Legacy safeguard aliases remain compatible and are stripped with the generic policy escapes before real Git.
Atomic push,
commit only,
and status hints off now run as one idempotent fixed-transform stage between built-ins and trusted plugins.
Plugins receive both the exact raw command and final transformed command.
Expected commit-only rejection emits a non-configurable `core-finding` JSONL event;
unexpected transform failure emits `core-incomplete` and exits `2`.
Auto-push remains fixed behavior later in the staged lifecycle.

### Forbidden root context

`forbidden-root-context` runs before commit and on direct check.
It rejects a root `CONTEXT.md` candidate.
It is the first repo plugin migration and proves the minimal finding path.

### Forbidden strings

`forbidden-strings` wraps the separately built,
SLSA-attested scanner and remains inert until trusted config registers it.
It scans:

- predicted would-be-committed content before forwarding;
- committed ground truth after commit and before auto-push;
- actual content-bearing ranges for manual push;
- direct check scope.

Manual push uses Git's pre-push protocol from a private `--dry-run --verify` probe,
then validates every negotiated remote OID through `git ls-remote --refs`.
It scans every newly reachable commit tree and final commit,
tag,
tree,
or blob state.
Commits created outside the wrapper and forbidden content removed by a later commit therefore remain covered.
Manual-push blob content is loaded through one `git cat-file --batch` process,
repeated scanner-equivalent historical states are removed,
and scanner files use at most 64 concurrent materialization lanes.
This avoids subprocess fan-out while preserving exact historical bytes.
Explicit user dry runs do not run manual-push policies.

An enabled scanner that cannot determine a required content-bearing push range has not completed and exits `2`.
A pure ref deletion carries no content and is exempt from that failure.
Missing executables,
interruption,
unexpected status,
malformed output,
and scanner-owned candidate read failure are distinct diagnostics that emit `plugin-threw` and exit `2`.
Redacted scanner findings emit qualified finding events and exit `1` at error severity.
The policy defaults to the `forbidden-strings` executable on `PATH`,
accepts an explicit executable option,
and never invokes a shell.
Because candidate scanning uses explicit temporary paths,
the adapter applies the scanner's path-anchored `--all` exclusions for its configured rules file and canonical
self-match sources before materialization;
unrelated nested basename matches remain scannable.
The policy defaults to error and is warn-unsafe.

### Final newline

For selected non-empty text files,
`final-newline` requires exactly one final LF.
Fixing removes a final LF run and appends one LF.
Empty and binary-looking files remain byte-identical.

Exact exclusion families are:

- `package/fuzz/forbidden-strings/seeds/**`
  (originally `corpus/**`;
   renamed when committed seeds moved to tracked `seeds/<target>/` dirs,
   see `doc/decision/gitignore-negations.md`);
- `package/rust-module/forbidden-regex.fuzz/seeds/**`;
- `package/test-fixture/toml-edit/src/**`;
- `**/dist/final/node/**`;
- `**/bundle/node/**`
  (committed Claude Code plugin bundles,
   which moved out of `dist/`;
   see `doc/decision/gitignore-negations.md`).

Commit fixing changes only exact would-be-committed blobs.
Unstaged tails and unrelated staged paths remain byte-identical.
The implementation must cover hk's duplicate-separator partial-staging regression.
Direct fix replaces the old `hk fix --all --step final-newline --no-stage` capability without touching the index.

## CI

CI remains independent of wrapper trust.
Forbidden-strings continues to run its SLSA-attested binary directly.
Before hk removal,
issue `#356` added an independent final-newline check that invokes the direct checker without loading unrelated root
tooling.
Do not reintroduce generic hk execution in CI.

External consumers that exercise trusted config in CI run:

```text
git cli-git trust --yes
```

## Performance and packaging gates

Measure the built shim for:

- no config;
- read-only command;
- strict MJS;
- strict cached TypeScript;
- relaxed TypeScript rebuild;
- validator;
- scanner;
- normalizer;
- post-commit lifecycle.

The spec defines fixture and sampling methods.
Numeric budgets are set only from measured baselines during #356.
Package readiness includes lint,
type checks,
tests,
`npm pack` audit,
and a disposable non-workspace installation and invocation.

## Retirement capstone

Issue `#357` retired hk and Pkl only after policy parity,
trust,
transaction,
performance,
package,
documentation,
and independent CI gates passed.

The capstone removed tool declarations,
managed lock entries,
root hk config,
and obsolete Pkl IDE config.
It rewrote current planning and troubleshooting guidance,
updated forbidden-strings documentation,
and added a verified idempotent per-machine cleanup for `hook.hk-*` Git config.
Issues `#143` and `#160` are superseded by the completed cli-git platform.

Actual npm registry publication remains outside this capstone.

## Alternatives considered

### Native Git hooks

Rejected as the primary enforcement point because hooks cannot express command transforms and require separate
installation.
CI remains the backstop for PATH bypasses.

### Separate policy kinds

Rejected because validation,
content scanning,
and normalization often share selection and Git context.
One finding model and one check interface provide greater locality.

### Shared config merge behavior

Rejected because array and collection merge semantics are product choices.
Consumer-owned JavaScript composition keeps those choices explicit.

### Live or dependency-resolved MJS execution

Rejected because dependency resolution changes trust inputs and importing the compared live entry leaves an entry-file
compare-then-swap window.
Stored artifacts make the entry execution target exact;
they do not restrict trusted code's ambient authority to load other live code deliberately.

### Hash-based trust

Rejected.
Trust stores and compares exact bytes and executes stored copies.
Hashes would add an unnecessary representation while failing to remove the need to preserve executable bytes.

### Global relaxed or discovery bypasses

Rejected because repository-controlled environment could use an unkeyed bypass.
Relaxation names one exact filesystem and path and never waives first trust.

### Fail-open incomplete scans

Rejected for content-bearing operations.
An enabled policy that cannot obtain required content has not completed and must block.

### Retire hk before platform parity

Rejected because the accepted sequence is platform-first with one capstone removal after evidence is complete.

## Supply-chain boundary

Retiring hk and Pkl removed hk's digest-only mise installation surface.
Forbidden-strings remains built in-repo and distributed to CI with SLSA provenance.
Cli-git trust decides whether to execute one repository artifact;
it does not certify plugin provenance or package versions.
Consumer lockfiles and self-contained builds govern plugin inputs.

## References

- `package/git-policy/cli/SPEC.md`:
  canonical implementation interface and verification contract.
- `doc/handover/cli-git-policies-platform.md`:
  implementation state and evidence.
- `package/git-policy/cli/README.md` and `package/git-policy/cli/src/index.ts`:
  current wrapper behavior.
- `package/git-policy/cli/src/escape-hatch.ts`:
  parser-based invocation escape hatches.
- `package/module/fs-id/README.md`:
  implemented filesystem identity prerequisite and verified platform behavior.
- `doc/planning/final-newline-normalization.md`:
  final-newline behavior and exclusions.
- `doc/troubleshooting/hk-partial-staging-final-newline.md`:
  exact partial-staging regression.
- `.github/workflows/forbidden-strings.yml`:
  independent scanner CI.
- GitHub issues #341 through #356:
  completed dependency-ordered implementation and release-readiness slices.
- GitHub issue #357:
  completed hk and Pkl retirement capstone.
- GitHub issue #358:
  recorded and indefinitely deferred npm publication.
