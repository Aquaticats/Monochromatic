# cli-git policies platform implementation grilling handover

## Status

Shared understanding was confirmed and implementation was authorized on 2026-07-09.
Issues `#280` and `#341` through `#357` are complete and closed.
The cli-git platform sequence has no active implementation slice.
The Pi Bash result-loss priority investigation is complete through implementation,
user-boundary verification,
and `doc/troubleshooting/pi-bash-output-spool-write-failure.md`.

The canonical decision is
`doc/decision/cli-git-policies-platform.md`.
The implementation interface is
`package/git-policy/cli/SPEC.md`.
This handover remains the current execution-state and evidence record.

Earlier checkpoint sections are retained as chronological evidence;
the `Release-readiness checkpoint on 2026-07-11` section and this status are authoritative when an older checkpoint says
work remains open.
Treat any unrelated worktree changes as concurrent work and do not stage,
stash,
restore,
or otherwise disturb them.

## Original implementation objective

Turn `package/git-policy/cli` into a standalone pluggable Git policy platform,
migrate the current cli-git safeguards and hk-managed checks,
then retire hk and Pkl after parity,
performance,
CI,
and cleanup requirements pass.

The wrapper continues to shadow `git` on `PATH` and forwards ordinary Git work to the resolved real Git executable by
absolute path.

## Verified current-state facts

- `package/git-policy/cli/package.json` maps both its package import and `git` binary to the same
  `dist/final/node/index.mjs` artifact.
- The package-root import exposes authoring declarations and `repositoryPolicyPlugin` without starting cli-git or enabling
  the optional policy.
- `package/git-policy/repository/src/index.ts` remains the canonical repository-policy source.
  File-enforcer generates `package/git-policy/cli/src/optional/repository-policy.ts` for self-contained source-level
  trusted config bundling.
- `package/git-policy/api` owns the cycle-free policy contract used by separately owned policy source packages.
- `package/git-policy/cli/src/bin.ts` runs the configurable policy engine before a sequential fixed-transform `RULES` array.
- Current policy order is:
  - `require-root`
  - `linked-worktree-only`
  - `branch-worktree-only`
  - `add-explicit`
- Current fixed-transform order is:
  - `atomicPush`
  - `commitOnly`
  - `statusHintsOff`
- `branch-worktree-only` is active,
  documented in `package/git-policy/cli/README.md`,
  and tested,
  but the source decision omitted it from the supposedly complete behavior classification.
- Current Git is 2.54.0 on this host.
  `git help --all` lists none of `policy`,
  `trust`,
  `check`,
  or `fix` as complete Git subcommands.
- Git has a real `git for-each-repo --keep-going` option,
  so cli-git must not consume a generic `--keep-going` token.
- Current Git documentation confirms that explicit-path `git commit` records worktree contents for the named paths and
  ignores staged contents for other paths.
- A disposable Git 2.54.0 fixture verified:
  - two unified patches based on the same blob and touching different lines compose through
    `git apply --cached --3way` against a copied index;
  - an overlapping patch creates unmerged temporary-index stages and exits nonzero;
  - the real index and worktree remain untouched when `GIT_INDEX_FILE` points at the copied index.
- Mise behavior was verified at upstream commit `21091fd55736794c1c8ef33da9f535c30bb20188`:
  - normal `mise trust` trusts one config;
  - `mise trust --all` walks ancestors and descendants;
  - automatic descendant trust occurs only when the trusted root config declares `monorepo_root = true`;
  - the relevant source is `src/cli/trust.rs`,
    `src/config/mod.rs:1903-1908`,
    and `src/config/config_file/mod.rs:378-386` in the disposable clone
    `/tmp/agent/mise-trust-20260709`.
- The repository Oxlint wrapper uses an eight-pass autofix cap in
  `package/dev-script/task-util/src/oxlint-fix-loop.ts` and detects stable and cyclic states.

## Settled policy inventory revisions

### Existing branch safeguard

`branch-worktree-only` becomes a built-in configurable policy.

- Default severity remains `error`,
  preserving current behavior.
- `warn` is considered safe metadata-wise;
  configuring it does not emit an additional dangerous-warning diagnostic.
- Its per-invocation escape hatch skips the policy for the complete invocation lifecycle,
  including findings,
  fixes,
  post-commit checks,
  and auto-push gating.

Warn-safety metadata follows consequence:

- warn-unsafe:
  `require-root`,
  `add-explicit`,
  `linked-worktree-only`,
  and `forbidden-strings`;
- warn-safe:
  `branch-worktree-only`,
  `forbidden-root-context`,
  and `final-newline`.

An unsafe `warn` remains a valid explicit configuration but emits a configuration warning.

### Unified Oxlint-style policy shape

The earlier mutually exclusive validator,
scanner,
and normalizer policy kinds are rejected.
One policy can combine all three behaviors.

- A policy returns structured findings.
- A finding may be command-level,
  path-located,
  or fixable.
- Fixability is represented by optional patch bytes on a finding.
- Expected violations are findings,
  not thrown exceptions.
- A thrown exception or incomplete policy run is an engine failure and always blocks.
- Option-bearing policies use Valibot schemas directly for runtime validation.
- Invalid config or invalid policy options are engine failures.
- One unified policy context exposes cheap parsed command facts directly and expensive Git-derived inputs through lazy
  async methods.
- Lazy Git queries are memoized only for one candidate-tree version.
  Applying a patch invalidates tree-dependent memoized data before the next policy runs.

The exact TypeScript types remain open.

### Config composition

Cli-git does not merge shared and local config layers.
`defineConfig` receives one final config object and is only a typed identity helper.
A consumer that wants inheritance performs an ordered deep merge in ordinary JavaScript before calling
`defineConfig`,
using `deepmerge-ts` or its own explicit merge logic.
The merge library and its array or collection semantics remain consumer dependencies and visible source code;
they are not cli-git runtime behavior.

### Policy registration and defaults

Plugins use an Oxlint-style consumer-chosen namespace map:

```js
export default defineConfig({
  plugins: {
    mono: forbiddenStringsPlugin,
  },
  policies: {
    'mono/forbidden-strings': 'error',
    'require-root': 'error',
  },
});
```

- Built-in IDs are unprefixed.
- Plugin IDs are `<namespace>/<policy>`.
- Every registered policy,
  built-in or plugin,
  uses its own declared default severity when omitted from `policies`.
- Registering a plugin therefore activates its policy defaults.
- An explicit `off` override disables a policy persistently.

### Execution order and stopping

The earlier phase-gated aggregation model was replaced.

Each policy pass uses this staged order:

1. Built-in configurable safety policies run in fixed core order.
2. Fixed command transformers derive the forwarded command.
3. Plugin policies run in namespace registration order and each plugin's policy declaration order.

Core policies can inspect raw and semantic command facts.
Plugin policies receive both raw and transformed command facts and predict candidate content from the transformed
command.
Fixed transforms must be idempotent because a changed candidate tree restarts the whole pass.

- Policies execute sequentially.
- Severity and option overrides do not reorder policies.
- The engine stops at the first remaining error-severity finding by default.
- Warning findings do not stop execution.
- `--cli-git-keep-going` continues later policies after an error finding,
  but Git still does not run if any error remains.
- `--cli-git-keep-going` never continues after an engine,
  plugin,
  patch,
  or transaction failure.
  Those failures stop immediately with exit code `2`.
- Cli-git does not infer policy independence or parallelize policies.
  A plugin that needs parallel checks composes them inside one policy,
  for example with `Promise.all`.

## Settled distribution and management surface

### Distribution

`@monochromatic-dev/git-policy-cli` becomes npm-ready instead of remaining private.
The prepared artifact is the Node package only;
standalone native executables are out of the first platform release.
Actual npm registry publication is recorded in #358 and deferred indefinitely until a maintainer explicitly resumes
it.
The package installs the shadowing `git` bin and includes the runtime needed for trusted TypeScript config bundling.
The same package exposes side-effect-free public config and plugin API entry points,
including `defineConfig`,
`definePlugin`,
`definePolicy`,
Valibot-backed option helpers,
and public types.
Importing an API entry point must never execute bin startup code.
Third-party plugins peer-depend on the compatible `@monochromatic-dev/git-policy-cli` API version;
there is no separate API or types package.
External non-mise users install the npm package and put its bin directory before real Git on `PATH`.

The package now declares `^22.18.0 || >=24.11.0` as its explicit Node runtime contract.
Trust bundling uses a lazy direct Rolldown dependency;
tsdown remains development-only build tooling.

`publishConfig`,
package contents,
installation documentation,
and end-user PATH verification remain active implementation work.
Registry authentication,
publish-workflow enablement,
and the upload itself belong only to deferred issue #358.

### Management commands

Cli-git intercepts a namespaced Git subcommand rather than installing a second executable:

```text
git cli-git trust
git cli-git check
git cli-git fix
git cli-git untrust
git cli-git status
```

Management commands use Optique.
Direct `check` and `fix` require exactly one scope source:
`--all` or Git pathspecs after `--`.
A repeatable policy filter narrows the otherwise complete enabled policy set.
The exact option names beyond the settled surface remain implementation work.
Management commands do not load repo config before performing trust recovery or inspection.

The real Git executable continues to be invoked by its resolved absolute path for forwarded commands.

## Settled config discovery and build model

### Discovery precedence

- If repo-root `cli-git.config.mjs` exists,
  it wins and is treated as a consumer-built runtime artifact.
- Otherwise,
  repo-root `cli-git.config.ts` is supported as cli-git-built source.
- Keeping both files is valid;
  `.mjs` wins.
- No config means built-in policies only.

### Consumer-built MJS

`cli-git.config.mjs` must be self-contained except for Node built-in imports.
All local and package JavaScript is expected to be bundled into the artifact.
Imports from the cli-git authoring API or plugin packages exist in consumer source only and must be bundled away;
a directly hand-written MJS artifact instead exports raw validated data or inlines its helpers.
The artifact contract is reproducibility guidance,
not a sandbox:
trusted code can use Node APIs to access files,
the network,
processes,
or dynamically loaded code.

### Cli-git-built TypeScript

`git cli-git trust` owns the first TypeScript build.
Ordinary Git commands never build an untrusted TypeScript config.
A previously trusted path explicitly relaxed through `CLI_GIT_NO_PARANOID` is the exception described below.

- Lazily import Rolldown only for trust builds.
- Call Rolldown's public `rolldown()` API with Node platform targeting,
  Node ESM output,
  dependency bundling,
  and `codeSplitting: false`.
- Close the disposable bundle after in-memory generation so native workers do not retain the process.
- Accept exactly one JavaScript output chunk with no unresolved non-Node imports or extra assets.
- A failed or incomplete bundle aborts trust.
- The trusted cache executes an immutable bundle,
  not live source modules.
- Automatic invalidation covers:
  - `cli-git.config.ts`;
  - its statically resolved relative local module graph.
- Lockfiles do not participate in invalidation.
- Bare package imports,
  including repo-local workspace package imports,
  do not participate in invalidation.
- An explicit repeat `git cli-git trust` always rebuilds TypeScript and is the refresh path for excluded package
  imports.
  Changed cached bundle bytes are disclosed before replacement even when tracked source bytes are unchanged.
- Under `CLI_GIT_NO_PARANOID`,
  an MJS mtime/size change triggers private snapshot replacement and validation,
  while a TypeScript entry or tracked-relative-module mtime/size change triggers an automatic Rolldown rebuild during
  the next config-loading Git command.
  Those metadata values are cache signals only,
  not trust checks;
  replacement or rebuild failure blocks with exit code `2` and retains the previous record.
- Package-only changes still require explicit trust refresh because package imports are outside tracked invalidation.
- Trust prints a warning when TypeScript source uses bare package imports excluded from automatic invalidation,
  even though their current JavaScript is bundled into the cached artifact.
- Tsdown source was inspected at commit `b89bc3f7bd6615158fb77dfacbe103546a4b722e`,
  package version 0.22.4,
  under `/tmp/agent/tsdown-cli-git-20260709`.
  Its public `build()` returns output bundles;
  `deps.alwaysBundle` can force dependencies;
  config discovery can be disabled;
  and tsdown does not expose an esbuild-style metafile option.
- Exact local-module graph extraction from Rolldown output chunks,
  absolute imports,
  native modules,
  and package assets still require a disposable integration prototype.

## Settled config-loading boundary

- Known read-only and inspection commands do not load repo config.
- Mixed commands such as `branch` and `tag` receive argument-aware classification so read-only forms can skip loading.
- Unknown aliases,
  external subcommands,
  and future Git commands load trusted config by default.
- Classifier ambiguity must take the config-loading path unless later revised.
- An untrusted or changed config blocks a config-loading Git command.
  Cli-git does not proceed with built-ins only.
- The diagnostic directs the user to `git cli-git trust`.
- The previously planned global config-discovery environment kill-switch is deleted;
  it was added in error.

The complete read-only and mixed-command grammar remains open and needs source-backed tests.

## Settled trust model revisions

### Trust consent

Trust uses two stages because recursive-child intent lives inside arbitrary config code and cannot be discovered safely
before authorizing that code.

Before the first consent prompt,
`git cli-git trust` prints every fact available without executing config:

- config path and format;
- filesystem ID;
- exact source-snapshot status and changed path list on re-trust;
- exact cached-bundle snapshot status for TypeScript;
- non-self-contained TypeScript imports or equivalent warning;
- explicit arbitrary-code authority;
- notice that recursive-child intent will be evaluated only after root execution is authorized.

The warning must state that config and bundled plugins run with user permissions and may:

- read and write files;
- run programs;
- access the network;
- automatically modify Git content;
- behave incorrectly despite cli-git's transaction safeguards.

Local use then asks `Trust this config? [y/N]`.
The root approval stays in-memory until config execution and validation succeed;
a thrown or invalid config leaves no persistent trust record.

After successful authorized execution:

- no child-trust declaration completes root trust with no second prompt;
- a child-trust declaration prints the exact recursive root and descendant authority,
  then asks for separate consent before recording the recursive marker.

`git cli-git trust --yes` prints both disclosures when applicable and skips both input reads for explicit CI use.

### Untrusted and changed artifacts

- Default posture uses exact stored byte snapshots,
  not cryptographic content hashes.
- Consumer-built MJS stores and compares the complete artifact bytes.
  After an exact match,
  cli-git executes the stored snapshot copy rather than the live entry file,
  closing the entry-file compare-then-swap window.
  Trusted code still has ambient Node authority and may deliberately load other live files.
- Cli-git-built TypeScript compares tracked live source bytes to stored source snapshots during ordinary strict
  execution and executes the stored cached bundle without rebuilding.
  Explicit trust and relaxed rebuild compare candidate bundle bytes to the stored bundle before replacement.
- Recursively auto-enrolled child configs also execute their stored snapshot or cached bundle,
  never a live file that was merely compared.
- First execution blocks until explicit trust.
- A later covered source or artifact byte change blocks until re-trust.
- Trust and plugin failures are exit code `2`.

### Registry

- Trust records are per-user and stored as one record directory per trust key,
  containing validated metadata and exact snapshot files.
- Production registry location derives from the OS account home,
  not `HOME`,
  XDG,
  AppData,
  or a dedicated environment override.
- Tests inject a registry location through internal APIs.
- Per-key files must use atomic replacement and safe permissions.
- Exact platform paths,
  record schema,
  canonicalization,
  symlink handling,
  and crash recovery remain open.

The underlying trust identity remains the pair of filesystem ID and canonical artifact path.
Cli-git does not hash that key or trusted content;
registry path mapping and exact byte comparisons must preserve the complete identity.
`@monochromatic-dev/module-fs-id` remains a prerequisite.

### No-paranoid relaxation syntax

`CLI_GIT_NO_PARANOID` is a comma-separated list of `<filesystem-id>:<path>` entries.
Percent escaping protects comma and percent characters.
Entries split on the first decoded colon because filesystem IDs are colon-free and Windows paths may contain later
colons.

Malformed or suspicious entries emit a prominent warning,
are ignored,
and leave the affected path under strict snapshot behavior.
They do not waive first trust.
Exact canonical encoding and decoder-error behavior still need tests.

### Recursive root trust

Recursive trust is declared as `trust: { children: true }`,
matching mise's config-declared monorepo-root model rather than making every trust command recursive.

- A root declaration triggers the second trust-consent stage after root config execution and validation.
- The second disclosure warns before consent that descendant authority will be recorded and names the exact root.
- The root declaration waives separate first approval for descendant configs only after that second consent.
- Recursive path authority intentionally crosses filesystem and mount boundaries without another prompt.
  The disclosure states that descendants on current or future mounted volumes inherit authority and that mount swaps can
  introduce newly authorized child configs.
- Descendant filesystem IDs are still recorded for exact identity and revocation provenance,
  but a new child filesystem does not require separate consent.
- First descendant encounter auto-enrolls exact snapshots of that descendant's covered files.
- Later descendant byte changes block for re-trust.
- Trust records track provenance.
- Untrusting a recursive root revokes descendant records inherited from that root.
- Untrusting a nested recursive root also revokes every outer recursive root that currently authorizes that nested path.
  This intentionally removes inherited authority from sibling subtrees too and avoids persistent deny-boundary state.
- Before revocation,
  `git cli-git untrust` lists every affected recursive root.
- Descendants explicitly trusted separately remain trusted.

The config property name remains open.

### CI

External consumers use an explicit CI step:

```text
git cli-git trust --yes
```

There is no detected-CI auto-trust path and no separate expected-snapshot environment bypass.

## Settled finding and process contract

- Policy findings use stable JSONL only.
- Autofix passes buffer findings internally and emit only the final stable pass to the public JSONL stream.
  Intermediate findings are provisional and never appear as authoritative output.
- There is no separate prose,
  JSON-array,
  or SARIF policy formatter.
- Every finding includes a human-readable `message` field.
- Wrapper-mode findings go to stderr so real Git stdout remains intact.
- Direct `git cli-git check` and `fix` findings go to stdout.
- Exit meanings when real Git was not run:
  - `0`:
     success or warning findings only;
  - `1`:
     one or more error-severity findings;
  - `2`:
     trust,
    config,
    plugin,
    transaction,
    or engine failure.
- A forwarded real Git command normally preserves real Git's exit code.
- Exception:
  a post-commit policy or engine failure after Git created the commit returns exit code `2`,
  blocks auto-push,
  and does not roll back the commit.
  Its JSONL event must state that the commit landed and include the new commit OID so callers do not retry blindly.

The JSONL schema and schema-version field remain open.

## Settled normalizer and patch model

### Patch ownership

- A fixable finding returns unified Git patch bytes for exactly its located path and against the candidate bytes the
  context supplied.
- A composed policy returns multiple findings for multiple paths rather than one cross-path patch.
- The plugin does not choose or write a patch path.
- Cli-git writes patch bytes into a private temporary `.patch` file,
  applies it,
  and removes it.
- Git diff and three-way merge semantics compose patches.
- Patch conflicts remain in temporary state only and block the command.
- Patch path traversal,
  renames,
  mode changes,
  submodules,
  unexpected paths,
  and binary behavior need an explicit allowlist and tests.

### Automatic fixing

- Matching pre-forward commit normalizers always auto-fix.
- Trust disclosure is the consent boundary for that mutation authority.
- Push and direct check remain read-only.
- `git cli-git fix` is the explicit direct worktree-fix surface.
- Direct `check` and `fix` require explicit paths or `--all`.
- Direct fix leaves the real index unchanged.

### Unsupported commit modes

For `git commit --patch`,
`--interactive`,
`--include`,
or another mode not yet reproduced transactionally:

- run the applicable policy in check mode;
- proceed if no fix is required;
- block with direct-fix guidance if a fix is required;
- never skip the policy silently.

### Multipass convergence

- Within one pass,
  policies run in settled order and later policies see patches applied by earlier policies.
- An error finding whose patch applies cleanly is provisional:
  it does not block that pass,
  and a later global pass must prove the finding disappeared.
- If any policy changes the candidate tree,
  every finding from that pass is provisional and the next pass restarts from the first core policy.
- Without keep-going,
  an unpatched error stops the current pass before later policies run.
- With keep-going,
  later patches may change the candidate and cause a restart;
  only findings from the final unchanged pass can block or emit.
- Use an eight-pass cap for the complete policy sequence,
  matching the repository Oxlint wrapper.
- Store each pass candidate in private temporary storage and compare complete file bytes instead of hashing them.
- Detect stability by exact ordered path-and-byte equality with the preceding state.
- Detect cross-policy cycles by streaming exact comparisons against non-adjacent candidate states retained only for the
  eight-pass run;
  do not retain duplicate complete snapshots in process memory.
- A stable candidate with remaining error findings is exit code `1`.
- A cycle or cap reached while still changing is exit code `2`.

### Working commit transaction design

This design was independently reviewed but still needs a complete disposable-repository prototype before it becomes an
implementation commitment.

- Never let plugins mutate the real index or worktree through the normalizer API.
- Hold the real index lock for the transaction.
- Journal the gap between Git's ref update and cli-git's real-index replacement.
- For index commits such as explicit `--no-only`:
  - copy the real index;
  - apply normalizer patches through `GIT_INDEX_FILE`;
  - invoke real Git against the temporary index;
  - atomically install the resulting index after commit success.
- For normal explicit-path commits with injected `--only`:
  - build a commit index from `HEAD` plus selected worktree paths;
  - apply normalizer patches there;
  - internally remove `--only` and pathspecs because the temporary index now represents the intended tree;
  - build a post-commit index from the original real index plus selected paths from the new commit;
  - atomically install that post-commit index after success.
- For merge,
  cherry-pick,
  and revert conclusions:
  - use index-commit semantics only;
  - do not emulate partial explicit-path conclusions.
- On patch conflict or commit failure:
  - discard temporary indexes and patches;
  - leave the real index and worktree unchanged by cli-git.
- No Git primitive atomically updates both the commit ref and a separate real index.
  The implementation therefore needs a durable journal and recovery on the next cli-git invocation.

Required prototype coverage includes partial staging,
unrelated staged paths,
unstaged tails,
deletions,
untracked selected paths,
pathspec files,
amend,
allow-empty,
sequencer conclusions,
hook failure,
commit failure,
patch conflict,
process interruption,
and recovery after the ref moved but before index replacement.

## Existing source-decision choices not yet reopened

Unless later grilling changes them:

- Fixed core transformers remain:
  - atomic push;
  - commit-only;
  - status hints off.
- Post-commit auto-push remains a fixed core side effect.
- Built-in configurable safety policies include:
  - require-root;
  - add-explicit;
  - linked-worktree-only;
  - branch-worktree-only,
    added during grilling.
- Repo/plugin policies include:
  - forbidden-root-context;
  - forbidden-strings;
  - final-newline.
- Forbidden-strings scans predicted commit content before forwarding,
  committed ground truth before auto-push,
  and manual push ranges.
- The source decision's derived fail-open rule for an indeterminate content-bearing manual push range is superseded.
  An enabled scanner that cannot determine required content has not completed its policy run,
  so it blocks with engine exit code `2`.
- A pure ref deletion carries no content and does not fail merely for lacking a content range.
- CI remains authoritative and independent of the wrapper.
- Final-newline exclusions and exact-byte semantics remain those in the source decision.
- Platform-first sequencing remains in force.
- hk and Pkl retirement remains the capstone after parity,
  performance,
  docs,
  CI,
  and per-machine cleanup requirements pass.

## Implementation checkpoints

### Issue #280 validated Git roots

- Reproduced fs-path and cli-git disagreement with Git through deterministic empty-directory,
  malformed-gitfile,
  and unusable-target fixtures before applying the fix.
- Added shared Git administrative validation for regular and symbolic HEAD,
  SHA-1 and SHA-256 detached IDs,
  objects,
  refs,
  relative gitfile targets,
  linked-worktree `commondir`,
  regular-file checks,
  and NUL rejection.
- Invalid nearer `.git` entries are skipped so a valid outer root remains discoverable.
- Added typed `GitRepositoryRootNotFoundError` and made cli-git consume the shared finder instead of independent
  `find-up` existence checks.
- Fs-path build,
  full tests,
  type lint,
  and zero-warning Oxlint pass.
  Cli-git build,
  all unit and integration tests,
  type lint,
  and zero-warning Oxlint pass.
- A disposable built-wrapper fixture verified native Git 2.54.0 handling for an invalid ancestor and cli-git handling
  for real normal,
  linked-worktree,
  linked subdirectory,
  and submodule roots.
- Commits `cdf023791`,
  `3c848a747`,
  `1499ed61e`,
  and `8609616c8` contain implementation,
  adversarial coverage,
  symbolic HEAD parity,
  and source-backed troubleshooting evidence.
- Independent final review reported no required corrections.

### Issue #343 package and authoring API

- Split executable startup into `src/bin.ts` and made the package-root `src/index.ts` a side-effect-free authoring API.
- Added identity-preserving `defineConfig`,
  `definePlugin`,
  `definePolicy`,
  and `definePolicyOptions` helpers plus policy,
  finding,
  patch,
  config,
  trigger,
  and lazy-context contracts.
- Added exported `ABSENT_GIT_VALUE` unique symbol for domain absence without nullable unions.
- Prepared public package metadata with Node `^22.18.0 || >=24.11.0`,
  built declarations,
  licenses,
  side-effect metadata,
  shadow `git` bin,
  tsdown runtime,
  TypeScript 7,
  Valibot,
  and Optique.
- `pack:npm` works around pnpm issue #9566 with a command-local forced non-deduplicated install before `pnpm pack`.
- The tarball contains only audited runtime chunks,
  declarations,
  package metadata,
  documentation,
  and licenses;
  it has no `workspace:` or `catalog:` specs,
  and no repository-only or sensitive content.
- A disposable npm consumer imported the API without filesystem changes,
  compiled valid config,
  rejected unknown IDs and wrong options,
  emitted and ran a self-contained tsdown config bundle,
  resolved the packaged bin first on `PATH`,
  and forwarded real Git 2.54.0.
- Build,
  type lint,
  zero-warning Oxlint,
  all package unit tests,
  README lint,
  final standalone install,
  and independent review pass.
- Commits `ae4c09853` and `58f1d7754` contain the implementation and contract evidence.
  Commits `9ced4d01e`,
  `e36b89d73`,
  and `0108b0db2` preserve the tested but ineffective Valibot allow-list path and its removal.
- Registry publication and workflow enablement were not performed;
  deferred issue #358 remains untouched.

### Issue #344 first complete policy-engine path

- Moved `require-root` from the legacy rule array into a configurable built-in policy using the public unified contract.
- Added stable schema-versioned JSONL finding and engine-failure events,
  invocation-local sequence values,
  final-pass buffering,
  wrapper stderr routing,
  direct-check stdout routing,
  and exit codes `0`,
  `1`,
  and `2`.
- Added runtime-authoritative Valibot validation for this optionless built-in's severity configuration.
  Source-level engine fixtures prove persistent `off`,
  warn-unsafe `warn`,
  default `error`,
  unknown-ID failure,
  fixed registry order,
  keep-going collection,
  and immediate stop on a thrown policy.
- Added flag-position parsing for `--no-enforce-require-root` and `--cli-git-keep-going` while preserving option values and
  post-`--` pathspec bytes.
- Added Optique `git cli-git status` and built-in-only `check` dispatch.
  Direct check requires exactly one scope source,
  rejects positional input before `--`,
  and preserves preceding Git global options such as `-C`.
  Optique still owns command and option parsing;
  the small pre-separator scan enforces Git's pathspec boundary rather than duplicating Optique's grammar.
- Kept the package-root authoring import side-effect free and continued resolving forwarded Git through an absolute path.
- Changed built CLI integration tests to execute `dist/final/node/bin.mjs`.
  Disposable repositories verify exact settled event bytes,
  stderr and stdout routing,
  escapes,
  direct and wrapper failures,
  global chdir,
  status output,
  malformed grammar,
  and preservation of every legacy safeguard.
- Fixed module-logger's expected ancestor `ENOENT` probes so they cannot contaminate machine-readable stderr;
  its direct regression test proves missing paths stay silent and unexpected stat failures remain reported.
- The final npm tarball was rebuilt after the last corrections.
  A disposable external install imported the authoring API,
  verified the executable shebang,
  selected the packaged shadow bin on `PATH`,
  exercised wrapper and direct JSONL from a repository subdirectory,
  rejected mixed scope with exit `2`,
  stripped the escape,
  and observed no logger contamination.
- Build,
  type lint,
  zero-warning Oxlint,
  all cli-git tests,
  module-logger lint and direct regression tests,
  tarball audit,
  and final independent review pass.
- Commits `10f962c5c`,
  `a3cabafee`,
  `6bf9ab513`,
  `481f93072`,
  `2e9471e21`,
  and `bf5df520a` contain the engine,
  management grammar,
  built coverage,
  logger isolation,
  and review corrections.

### Issue #345 single-artifact MJS trust

- Added Git 2.54-aware config-loading classification:
  known inspection commands skip config;
  `branch` and `tag` inspect positional,
  listing,
  mutation,
  clustered-short,
  and inline long-option forms;
  unknown and ambiguous commands load config.
- Added repository-root config discovery with MJS precedence,
  chained Git `-C` handling,
  regular-file and symlink rejection,
  and explicit TypeScript deferral to #347.
- Added strict MJS UTF-8 and Acorn syntax validation.
  Static imports,
  re-exports,
  and literal dynamic imports accept only Node built-ins;
  local,
  package,
  computed dynamic,
  and extra artifact module edges fail before consent.
- Added runtime-authoritative config,
  plugin,
  policy,
  severity,
  and Valibot option validation.
  Trusted plugin policy definitions run through the existing deterministic policy registry and direct-check path.
- Added OS-account-derived registry roots independent of repository environment variables.
  Complete filesystem-ID and canonical-path identities use reversible unpadded base64url components without hashes.
- Added no-follow candidate capture that opens the source before filesystem identity resolution.
  Linux identity resolution uses the open process descriptor;
  other hosts bracket path-based resolution with same-handle metadata and final live-path device and inode agreement.
  Degraded and failed identity diagnostics stay caller-owned rather than contaminating JSONL stderr.
- Added private schema-version-one records,
  exact source snapshots,
  exclusive per-key lock directories,
  fsync,
  validated temporary directories,
  atomic rename,
  rollback,
  exact untrust,
  and fail-closed handling for malformed,
  interrupted,
  concurrent,
  changed,
  throwing,
  or unsafe records.
- Added canonical registry ancestry checks,
  POSIX account ownership and mode verification,
  and protected Windows ACL application and read-time verification for directories and files.
- Added informed consent with path,
  filesystem identity and stability,
  exact snapshot state,
  retained built-ins,
  and full-account authority.
  Root candidate code is not executed before affirmative consent;
  `--yes` supports explicit noninteractive CI use.
- Added trust,
  untrust,
  and trust-status management output.
  First config-loading use blocks with exit `2` without execution;
  unchanged commands execute the private stored snapshot;
  exact changed bytes block until re-trust.
- Unit and subprocess coverage exercises classification,
  discovery,
  self-containment,
  pre-consent non-execution,
  exact lifecycle,
  direct plugin findings,
  concurrent writers,
  interrupted candidates,
  registry ancestry and record symlinks,
  unsafe permissions,
  changed bytes,
  and management exits.
- `mise run //package/git-policy/cli:test:built:trust` builds the unpublished tarball and passed in a bounded disposable
  Node container.
  It installs the package's actual shadowing `git` bin and verifies pure JSONL first-use and changed-byte failures,
  trust disclosure,
  status,
  stored plugin execution,
  read-only bypass,
  and untrust.
- Cli-git formatting,
  types,
  unit tests,
  build,
  npm packing,
  module-fs-id formatting,
  types,
  build,
  and unit tests pass locally.
- Maintained packed-bin task passed after final runtime corrections with `built-trust-consumer-ok`.
- Windows ACL workflow run
  [29072787369](https://github.com/Aquaticat/Monochromatic/actions/runs/29072787369)
  passed protected directory and file ACL application plus deliberate broad-ACL tamper rejection,
  emitting `windows-trust-acl-ok`.
- Independent review found assigned-long-option bypass,
  filesystem-identity ordering,
  registry-ancestor,
  Windows ACL,
  degraded-warning,
  built-test maintenance,
  and status-documentation gaps.
  Commits after the review corrected the actionable boundaries and added regression evidence.
- Commits `af5e0a8bb`,
  `ff2d4831a`,
  `30efc0e2c`,
  `7bcd00a55`,
  `521b667eb`,
  and `3c46a4b82` contain the initial runtime,
  lifecycle tests,
  storage containment hardening,
  reviewed boundary fixes,
  contract and workflow evidence,
  handle-backed Linux identity,
  pre-return lock cleanup,
  caller-owned filesystem diagnostics,
  and the verified Windows ACL repair.
- Final independent closure review found no remaining actionable repo-controlled correctness or security blocker.
  Privileged non-Linux mount-swap concerns are outside the checked-out-repository threat model and are documented by
  the guarded path-based identity behavior.
- Issue #345 was closed after all acceptance criteria and evidence gates passed.
- Recursive trust remains #346;
  TypeScript trust remains #347;
  actual npm publication remains deferred in #358.

### Issue #346 recursive snapshot trust

- Added two-stage consent for `trust: { children: true }`.
  Root execution and runtime validation happen in disposable private state before persistence.
  Declining the second stage installs ordinary explicit trust;
  `--yes` prints and accepts both disclosures.
- Recursive authority intentionally crosses filesystem boundaries beneath the exact canonical repository root.
  Descendant enrollment records exact bytes,
  stored executable snapshots,
  and every currently authorizing recursive root without another prompt.
- Each recursive root must still match its trusted filesystem identity and exact stored bytes before it can authorize a
  new descendant.
  Changed,
  deleted,
  and mount-replaced roots fail closed.
- Explicit descendant trust adds a self-authorizer that survives outer-root removal.
  Auto-enrolled nested recursive roots remain bounded by outer provenance and do not require another consent stage.
- Added cascading revocation that removes inherited authority,
  preserves independent self-authorizers,
  and revokes outer roots plus affected sibling authority when a nested recursive root is untrusted.
  Every affected recursive root is disclosed before mutation.
- Added config-deletion recovery:
  `untrust` can find the sole stored record from the canonical repository root without executing missing code.
- Added registry-wide enrollment and revocation serialization,
  deterministic per-record locks,
  no-follow private transaction journals,
  parent and journal-directory fsync,
  idempotent recovery,
  and fail-closed symbolic-link and non-file rejection.
- Added disposable unit fixtures for declined consent,
  invalid and changed roots,
  exact descendant changes,
  nested roots,
  siblings,
  deleted-root recovery,
  interrupted journals,
  journal symlinks,
  concurrent enrollment and revocation,
  and independent explicit descendants.
- Extended the maintained packed-bin container with a mounted tmpfs descendant.
  It verifies cross-filesystem inheritance,
  mount replacement at the same canonical path,
  fresh exact enrollment after replacement,
  process-level enrollment and revocation contention,
  and final cascade cleanup through the published-shape shadowing `git` executable.
  The completed task emitted `built-trust-consumer-ok`.
- Independent review identified stale changed-root authorization,
  transaction durability,
  config-deletion revocation,
  journal-link safety,
  integration-fixture,
  and documentation gaps.
  Commit `f4bb63b99` addresses those findings.
  A follow-up review found duplicate mount-replacement identities and transaction-directory substitution gaps;
  commit `bf37063ed` revokes every same-root identity and validates the journal directory during creation,
  recovery,
  and settlement.
  Final independent review found no remaining concrete #346 acceptance blocker.
- Final cli-git formatting,
  type checks,
  unit tests,
  build,
  npm packing,
  selected documentation lint,
  and the maintained packed-bin trust task pass.
  The final packed task emitted `built-trust-consumer-ok` after mount replacement,
  absent-config multi-identity untrust,
  process-level enrollment and revocation contention,
  and cascade verification.
- Commits `53b262593`,
  `f421486be`,
  `f4bb63b99`,
  `6255bd205`,
  and `bf37063ed` contain the recursive runtime,
  mounted packed-bin fixtures,
  review corrections,
  and contract documentation.
  Issue #346 was closed after every acceptance criterion and evidence gate passed.
  TypeScript trust remains #347;
  actual npm publication remains deferred in #358.

### Issue #347 TypeScript bundle trust

- Added MJS-precedence fallback discovery for root `cli-git.config.ts` to the trust runtime.
- TypeScript trust originally used tsdown's public `build()` API.
  Issue #356 replaced that middle layer with a lazy direct Rolldown `rolldown()` call,
  in-memory Node ESM generation,
  package bundling,
  `codeSplitting: false`,
  and explicit bundle closure.
  `doc/troubleshooting/cli-git-post-commit-tree-latency.md` records the measured trust-build result.
- The build accepts exactly one JavaScript entry chunk and rejects extra outputs,
  unresolved non-Node module edges,
  computed dynamic imports,
  source escapes,
  and native modules.
  Literal local and bare-package dynamic imports are inlined;
  package assets are accepted only when contained in the sole JavaScript bundle.
- A source-capture Rolldown plugin supplies exact no-follow bytes for the entry and resolved relative local graph.
  Bare package imports are bundled but excluded from invalidation and disclosed before consent.
  The entry identity and every tracked source are re-captured after complete build output and must still match exact bytes,
  metadata,
  and identity.
- Root approval executes and validates only the private stored candidate bundle.
  Persistent records include every exact source snapshot plus one immutable executable bundle;
  failed builds and validation leave no record.
  Repeated explicit trust always rebuilds and discloses bundle-byte state.
- Strict loading compares all tracked source bytes and executes only the stored bundle.
  Recursive TypeScript roots and descendants use the same stored-bundle model;
  authorizing roots are revalidated again immediately before descendant installation.
- Added exact `CLI_GIT_NO_PARANOID` comma and percent grammar.
  `%25` and `%2C` are the only escapes;
  malformed entries and current-path filesystem mismatches use separate JSONL warning codes and never waive first trust.
- Relaxed MJS paths use source size and mtime as private replacement signals.
  Relaxed TypeScript paths rebuild after any tracked metadata change,
  require the rebuilt identity to equal the authorized identity,
  validate before atomic replacement,
  and retain the prior record on failure.
- Disposable tests cover local graph changes,
  literal and computed dynamic imports,
  bare package imports,
  package assets,
  native modules,
  source symlink escapes,
  stored execution after package removal,
  recursive roots and descendants,
  changed authorizers during enrollment,
  invalid builds,
  exact relaxed grammar,
  rollback,
  and cache races.
- The maintained packed-bin container verifies TypeScript first-use blocking,
  trust disclosures,
  bare package and relative source bundling,
  stored policy execution,
  strict invalidation,
  relaxed rebuild,
  malformed-entry JSONL,
  process-level rebuild contention,
  retry,
  and untrust through the actual shadowing `git` executable.
  The final task emitted `built-trust-consumer-ok`.
- Independent review identified recursive-authorizer,
  mount-identity,
  final-graph,
  dynamic-package,
  warning-code,
  documentation,
  and packed-race gaps.
  Commit `4c99836ae` addresses the actionable findings;
  final independent review found no remaining concrete correctness or security blocker after the packed task passed.
- Commits `16e2de74d`,
  `e99f3ebb6`,
  `2007c8ec4`,
  `4c99836ae`,
  and `d4df86973` contain the initial builder,
  relaxed refresh,
  packed verification,
  reviewed race fixes,
  and external option diagnosis.
  Issue #347 was closed after every acceptance criterion and evidence gate passed.
  Actual npm publication remains deferred in #358.

### Issue #348 configurable safeguard migration

- Registered `linked-worktree-only`,
  `branch-worktree-only`,
  and `add-explicit` after `require-root` in one canonical built-in registry shared by execution and trusted config
  validation.
- Preserved default `error` severity for all four safeguards.
  Only `branch-worktree-only` is warn-safe;
  the other built-ins emit warning metadata when trusted config selects `warn`.
- Added persistent trusted-config `off` and `warn` coverage through the packed shadow executable.
  No-config unit and built-wrapper fixtures preserve previous default enforcement.
- Centralized generic and legacy escape parsing before policy execution.
  Flag-position escapes are stripped before real Git;
  option values and pathspecs with the same bytes are preserved.
  The engine retains escaped policy IDs for later lifecycle stages.
- Preserved stash,
  clean,
  reset,
  explicit and implicit branch creation,
  and bulk-add parsing through parser and disposable-repository suites.
- Added a disposable ignored-root-sentinel clean fixture covering `HEAD`,
  `config`,
  `hooks`,
  `objects`,
  and `refs` without mutating the real worktree.
- Kept the baked tool-cache allowlist segment-aware and realpath-safe with configured-cache and repository git-dir
  fixtures.
- Removed the superseded safeguard modules from `src/rules/` after policy-engine unit and wrapper parity passed;
  checks and their focused tests now live under `src/policy-engine/`.
- Commits `7cae6d950`,
  `4d6601f03`,
  `4e619ed82`,
  `0d6a61b38`,
  `01cfcd12e`,
  `e4eef7bff`,
  `6cde57dae`,
  `fff4e02a0`,
  `acb1f0590`,
  and `0595293f9` contain the engine migration,
  trusted registry correction,
  old-rule retirement,
  packed consumer coverage,
  machine-readable warning correction,
  and final sentinel evidence.
- Formatting,
  type checks,
  the full unit suite,
  selected Markdown lint,
  and the packed npm shadow-bin consumer passed.
  The final packed run was `proc_17`.
- Independent review found and then verified corrections for ad hoc warning output,
  missing escape context,
  and checker-only sentinel coverage.
  Its final closure pass found no remaining concrete blocker.
- Issue #348 was closed after every acceptance criterion and evidence gate passed.
  Actual npm publication remains deferred in #358.

### Issue #349 fixed command-transform stage

- Split policy execution into configurable built-in and trusted-plugin stages.
- Added the fixed-transform stage between them in the canonical atomic-push,
  commit-only,
  and status-hints-off order.
- Plugin contexts retain exact raw arguments while exposing final transformed arguments.
  Unit coverage and a trusted packed-plugin fixture validate both views.
- Removed fixed transforms from `bin.ts`'s legacy sequential rule pipeline;
  the engine result is now the sole final argv forwarded to real Git.
- Added pass-idempotence coverage for injected atomic push,
  explicit `--no-atomic`,
  commit-only injection,
  stripped commit escape state,
  and status-hints configuration.
- Converted expected commit-only rejections to typed `core-finding` JSONL events with stable codes.
  Unexpected transform exceptions become `core-incomplete` engine failures with exit `2`.
- Preserved commit parser,
  dirty-index,
  pathspec-file,
  include/only,
  and sequencer parity through retained focused and disposable Git suites.
- Added end-user status coverage for native-hint suppression,
  explicit user override,
  and uncorrupted porcelain output.
- Commit `c410b3a11` contains the initial staged engine and structured core findings;
  commits `906338dce` and `175172e8d` contain pass stability,
  packed plugin facts,
  exact real-Git argv capture,
  native status behavior,
  and unexpected-transform failure evidence.
  Commit `afb4d7c0f` records the contract and handover.
- Formatting,
  types,
  build,
  the full unit suite,
  selected Markdown lint,
  and packed npm shadow-bin run `proc_19` passed.
- Independent final review found no remaining source or test blocker after exact argv,
  status,
  failure,
  and JSONL contract corrections.
- Issue #349 was closed after every acceptance criterion and evidence gate passed.
  Actual npm publication remains deferred in #358.

### Issue #350 post-commit backup gate

- Widened engine lifecycle input to carry exact transformed arguments,
  canonical repository root,
  and lifecycle-specific lazy Git facts.
- After successful non-dry-run commit,
  real Git resolves the exact landed commit OID before any backup gate work.
- Post-commit candidates lazily enumerate the complete landed tree and read exact object bytes by blob OID;
  policies never read incidental worktree bytes for landed ground truth.
- Clean and warning-only post-commit results permit the existing auto-push behavior.
- Error findings,
  policy exceptions,
  and setup failures after OID resolution block backup,
  return `2`,
  retain the commit,
  and emit causal JSONL followed by `commit-landed` with the exact OID.
- Full-lifecycle plugin escapes now skip both pre-forward and post-commit checks for the same invocation.
- Preserved commit dry-run/output classification,
  configured upstream precedence,
  origin fallback,
  detached and no-remote skips,
  filtered success output,
  and ordinary backup-failure exit behavior.
- Packed disposable remotes cover successful backup,
  policy finding block,
  engine failure block,
  escaped backup,
  dry-run and porcelain exclusion,
  failed backup,
  non-origin upstream,
  detached HEAD,
  and no remote.
- Commits `65bf23dee`,
  `5e077357e`,
  and `379efccca` contain the landed facts,
  lifecycle gate,
  packed routing matrix,
  and setup-failure correction.
- Formatting,
  types,
  build,
  the full unit suite,
  selected Markdown lint,
  and packed npm shadow-bin run `proc_22` passed after the setup-failure correction.
- Independent final review found no remaining blocker and scoped escape evidence to implemented pre-forward,
  direct-check,
  and post-commit stages.
- Commit `494a56ca3` records the final contract and verification handover.
- Issue #350 was closed after every acceptance criterion and evidence gate passed.
  Actual npm publication remains deferred in #358.

### Issue #351 private-index autofix tracer

- Added arity-aware explicit commit pathspec extraction;
  wrapper escape flags and option values cannot become transaction paths.
- Policy results retain ordered patch proposals and accept monotonic candidate versions.
- Added locked disposable transaction workspaces outside the worktree,
  private commit and post-commit indexes,
  lazy exact candidate bytes,
  and atomic real-index installation only after successful Git commit.
- Explicit-path mode builds from `HEAD` plus selected worktree paths,
  removes internal only/pathspec arguments before private-index commit,
  and reconciles only selected landed entries into the original index copy.
- Explicit `--no-only` mode patches a complete copy of the real index and installs the resulting index after success.
- Patch proposals must bind exact opaque target ID,
  concrete Git-resolved path,
  and candidate blob revision in the patch index header;
  destination-grammar validation rejects stale bases,
  extra targets,
  mismatched headers,
  line-delimiter injection,
  renames,
  copies,
  mode changes,
  and binary directives.
- Patches apply sequentially with real `git apply --cached --3way` against `GIT_INDEX_FILE`;
  overlap conflicts remain private and return `content-unavailable`.
- Any patch proposal ends the provisional pass before later policies run;
  after an exact index change the full ordered engine restarts,
  provisional events are discarded,
  only the final unchanged pass emits,
  eight changed passes cap convergence,
  and prior exact states stream from private snapshots for cycle detection.
- Packed fixtures cover explicit-path and copied-index commits,
  selected canonical blobs,
  unrelated staged content,
  unstaged worktree tails,
  non-overlapping composition,
  overlapping conflict,
  policy exception,
  failed pre-commit hook,
  successful hook observation of patched private staged bytes,
  Git-magic pathspec resolution,
  pre-subcommand `-C` execution,
  and exact real index/worktree/ref preservation.
- Commits `e19c7340a`,
  `e07102aad`,
  `7e40e109c`,
  `f15c4cea8`,
  `8389a156b`,
  `75179dc49`,
  `ab30eb84a`,
  `35d4ae62e`,
  and `ecf2c09df` contain the parser seam,
  patch retention,
  private-index primitives,
  convergent integration,
  boundary hardening,
  evidence,
  exact snapshot streaming,
  and independent-review corrections.
- Formatting,
  type checks,
  build,
  full unit suite,
  and packed shadow-bin runs `proc_4`,
  `proc_5`,
  and `proc_6` pass.
- Independent final review found no remaining #351 blocker and confirmed durable recovery,
  exhaustive modes,
  signal,
  filesystem-error,
  and concurrent-writer recovery remain #352 scope.
- Issue #351 was closed after every acceptance criterion and evidence gate passed.
  Actual npm publication remains deferred in #358.

### Issue #352 recovery and exhaustive commit modes

- Added commit parser facts and transaction selection for pathspec files,
  stdin and NUL forms,
  deletions,
  untracked selected paths,
  amend,
  allow-empty,
  and merge,
  cherry-pick,
  or revert conclusions.
- Interactive and patch selection runs once through native Git against the copied private index;
  include selection stages into that private index.
  Policies inspect exact chosen candidates read-only,
  commit canonical private state without replaying selection,
  and return direct-fix guidance when correction is required.
- Automatic correction rejects unmerged indexes with exact path diagnostics.
- Durable prepared journals retain original,
  commit,
  and post-index snapshots,
  expected parent OIDs,
  intended tree,
  owner PID plus process-birth identity,
  a private nonce-bearing `GIT_REFLOG_ACTION`,
  and exact transaction-directory,
  prepared-index,
  and real-index-lock device/inode identities before ref advancement.
- Recovery runs before trusted config and distinguishes pre-ref cleanup,
  post-ref installation,
  already-installed state,
  active owners,
  replaced locks,
  unsafe artifacts,
  and conflicting external ref or index movement.
- Packed fixtures exercise mode compatibility,
  interruption before and after ref advancement,
  exact index installation,
  concurrent wrappers,
  lock replacement,
  conflicting ref movement,
  symlink rejection,
  read-only administrative filesystem failure with a healthy next invocation,
  standard-input pathspec capture,
  and signal cleanup.
- Recovery consumes artifacts through no-follow descriptors,
  stabilizes exact prepared files through verified hard links,
  installs the index through an owner-preserving hard link rather than a mutable lock pathname,
  distinguishes a live owner from PID reuse,
  and requires current OID plus the private nonce in the latest `HEAD` reflog entry when interruption precedes the
  exact landed-OID marker.
- Checkpoints `41a48556f`,
  `c5b217ae3`,
  `bbfe7e8fd`,
  `8c7e378c9`,
  `07405c2e4`,
  `19c333622`,
  `01cd5e292`,
  `33132c065`,
  and `3979281e9` contain expanded modes,
  adversarial fixtures,
  no-follow recovery,
  reflog attribution,
  exact native selection,
  patch-grammar hardening,
  owner-preserving artifact installation,
  and completed-install recovery evidence.
- Formatting,
  type checking,
  build,
  full unit tests,
  and packed shadow-bin runs through final `proc_6` pass;
  the independent closure review found no remaining blocker after standard-input and filesystem-error evidence landed.
- Actual npm publication remains deferred in #358.
- Issue #352 closed on 2026-07-10 after checkpoint `9d1447d29` completed the documentation contract and the independent
  closure audit found no blockers.

### Issue #353 repository policy plugin checkpoint

- Added private package `@monochromatic-dev/git-policy-repository` so repository policy code remains outside cli-git
  core.
- Added root `cli-git.config.ts` registering `repositoryPolicyPlugin` under consumer-chosen namespace `mono` with
  `mono/forbidden-root-context` active at its declared error default.
- The policy is warning-safe and matches only a non-deleted root `CONTEXT.md` candidate during `pre-forward` and
  `direct-check`.
- Added private-index add candidate derivation and unchanged-inclusive direct-check pathspec projection.
  Wrapper-only escape controls are removed before private Git runs,
  and candidate resources remain alive until lazy policy evaluation settles.
- Direct checks now derive exact worktree/index state for explicit Git pathspecs or `--all` and retain stdout JSONL;
  wrapper findings retain stderr JSONL.
- Added packed fixture coverage for root versus nested paths,
  pathspec-separator escape-looking tokens,
  add and commit blocking,
  direct checks,
  error,
  warn,
  off,
  lifecycle escape,
  changed strict trust,
  and state preservation.
- Commit `66bcce672` contains the implementation checkpoint.
  Commit `504e7b499` adds generated package license texts.
- Package format,
  type checks,
  unit tests,
  and build pass.
  The full cli-git unit suite also passed through the process tool both sequentially and with its normal parallelism.
- Two synchronous Pi Bash attempts to run the same cli-git unit task lost their tool results and led to Pi restarts.
  The priority investigation traced the recurrent batch failure to `EDQUOT` from Pi's unguarded full-output spill stream,
  installed a local Pi patch,
  and passed source,
  external consumer,
  direct TUI,
  and original-command verification.
  `doc/handover/pi-bash-result-loss.md` contains the resume evidence.
- The first packed run exposed fixture assertions that expected the policy-local code rather than the public qualified
  code.
  The maintained fixture now asserts `mono/forbidden-root-context/root-context-forbidden`.
- The final packed fixture installs both the cli-git and repository-policy tarballs.
  Trusted config imports the real `@monochromatic-dev/git-policy-cli/ts` and
  `@monochromatic-dev/git-policy-repository/ts` subpaths,
  and direct check covers explicit pathspec plus `--all` scope.
- The final packed container emitted `built-trust-consumer-ok`.
  Cli-git and repository-policy type checks,
  zero-warning Oxlint,
  and unit suites passed.
- Independent closure review found no remaining acceptance blocker.
  The unchanged-candidate classification observation does not affect this policy because every non-deleted root
  candidate is forbidden.
- Final correction commits `f312473a7`,
  `5b939695b`,
  `3f4e82cb1`,
  `3d53207ba`,
  `d3bf30fe9`,
  `3bd4eb45a`,
  `2d2a5ce0d`,
  and `c956ad88c` contain package,
  import,
  JSONL,
  direct-scope,
  strict-loading,
  and built-artifact test corrections.
- Issue #353 closed on 2026-07-10 after every acceptance criterion passed.
- Issues #354 through #357 remain dependency-ordered after #353.
  Npm publication remains indefinitely deferred in #358.

### Issue #354 forbidden-strings policy decisions

- The policy implementation lives in a separate private source package under
  `package/git-policy/forbidden-strings/`.
  It does not broaden `@monochromatic-dev/git-policy-repository`.
- Repo-owned policy source packages are statically bundled into cli-git's single public MJS artifact and exported from
  `@monochromatic-dev/git-policy-cli`.
  The same artifact is the executable bin and the side-effect-free import target.
  It contains no secondary chunks or cli-git-owned package-relative dynamic imports.
  Dynamic imports retained inside bundled libraries are exempt.
  The computed import that executes an exact stored MJS config is exempt so top-level `await` remains supported.
  Importing it does not enable shipped policies;
  trusted consumer config must register each plugin explicitly.
- Scanner resolution defaults to the `forbidden-strings` executable on `PATH`.
  Policy options may override the executable path;
  invocation always uses an argument array without shell interpolation.
- The scanner runs with repository root as cwd,
  so its existing rules precedence remains `--rules`,
  `FORBIDDEN_STRINGS_RULES`,
  then root `forbidden-strings.local.txt`.
- Manual-push facts query authoritative remote OIDs with `git ls-remote`.
  Push dry-run output and cached tracking refs are not policy ground truth.
- A content-bearing update whose remote range cannot be established fails closed with engine exit `2`.
  A pure deletion has no content range and does not fail merely for lacking one.
- An unreadable plugin-owned scanner temporary file is `plugin-threw`,
  not a policy finding or candidate-setup `content-unavailable` event.
- The policy defaults to error and is warn-unsafe.
- The existing scanner source and independent forbidden-strings CI remain separate;
  this slice wraps the binary rather than modifying its matching engine.

### Issue #354 implementation checkpoint

- Manual push now obtains Git-native update records from a private `--dry-run --verify` pre-push hook and validates
  destination OIDs through `git ls-remote --refs`.
  It scans every newly reachable commit tree and final commit,
  annotated-tag,
  tree,
  or blob state.
  Stale negotiation and indeterminate required content fail closed;
  explicit dry runs bypass manual-push policies and pure deletion remains contentless.
- Canonical policy source lives in `package/git-policy/forbidden-strings/`.
  File-enforcer maintains its cli-local mirror under
  `package/git-policy/cli/src/optional/forbidden-strings/` so the single cli-git MJS artifact exports the inert
  plugin without a package cycle or secondary runtime chunk.
- Root `cli-git.config.ts` registers `forbiddenStringsPlugin` under namespace `security` and points it at the repository's
  release scanner binary.
  Default plugin behavior still resolves `forbidden-strings` from `PATH`.
- Scanner invocation uses an argument array without a shell.
  Redacted exit-`1` output becomes findings.
  Missing executable,
  interruption,
  unexpected status,
  malformed output,
  and scanner-owned materialized-file read failure produce distinct messages under `plugin-threw` and exit `2`.
  Invalid completed plugin output remains `policy-incomplete`.
- Packed non-workspace fixtures cover direct check,
  predicted commit content,
  full-lifecycle escape,
  post-commit auto-push gating,
  manual pushes of externally created history,
  warning severity,
  and every scanner failure class.
  `mise run //package/git-policy/cli:test:built:trust` completed with `built-trust-consumer-ok` after the source export
  declared `nano-spawn` as a packed runtime dependency.
- A disposable repository copied the exact root config and used the real Rust scanner through the built artifact.
  Direct check and pre-forward findings blocked,
  clean post-commit policy evaluation auto-pushed,
  and an externally created forbidden commit was rejected by manual push while the bare remote stayed unchanged.
- Scanner-engine performance remains documented in `package/cli/forbidden-strings/PERF.md`.
  The cli-git integration measurement used the built tarball,
  scanner `0.1.9`,
  the 11 files changed from `origin/main` at revision `6eccb3064250a3c521d13f6824e4658f428c7628`,
  and a disposable Linux x64 container capped at 2 GiB RAM and 2 CPUs.
  Across 30 paired samples,
  direct scanner median was 1.993 ms,
  cli-git policy median was 269.900 ms,
  and wrapper-added median was 267.907 ms.
  This slice-specific measurement remained a baseline rather than a budget;
  raw samples live in `package/git-policy/cli/perf/forbidden-strings-2026-07-10.json`.
  Issue `#356` later added and hosted-verified the complete enforced lifecycle budget matrix.
- The first real repository push exposed unbounded process fan-out in manual-push candidate loading.
  Each lazy historical blob launched `git cat-file blob` inside an unbounded `Promise.all`,
  exhausting the host process table and producing `spawn /bin/bash EAGAIN` before a forced reboot.
  The remote remained at `ef179a737` and local commits remained intact.
- Commit `f621432a6` replaced per-blob processes with one `git cat-file --batch` process,
  removed scanner-equivalent historical duplicates,
  and bounded scanner-file materialization.
  Commits `b9b5a72c6`,
  `1830851c0`,
  and `470c0f5dc` measured storage concurrency and retained the 64-lane optimum after 128 lanes increased contention.
- A packed full-repository benchmark at revision `a9096c04a` ran 30 paired manual-push samples after six stable warm-up
  pairs in a disposable Linux container capped at 2 GiB RAM and 2 CPUs.
  Its 1 GiB `/tmp` tmpfs matches the host's verified temporary-storage type.
  Direct Git median was 54.702 ms,
  wrapper median was 1,174.731 ms,
  added median was 1,119.389 ms,
  added p95 was 1,152.066 ms,
  and worst added latency was 1,205.014 ms.
  Every sample remained below the strict 2,000 ms ceiling.
  The committed harness is `package/git-policy/cli/perf/manual-push-latency-benchmark.ts`;
  raw samples live in `package/git-policy/cli/perf/manual-push-latency-2026-07-10.json`.
- The latest hosted forbidden-strings workflow at pre-integration `origin/main` revision `ef179a737` failed on one
  credential-shaped patch-context line and credential-shaped literals in the concurrently landed forbidden-regex tests.
  Commits `c35e012b6` and `a8baa6024` preserve the tests' runtime bytes and patch applicability while removing those
  source-level matches.
  The exact pull-request workflow composition,
  baseline plus shared appendix against every `origin/main...HEAD` changed file,
  then passed locally for all 20 changed paths.
  A full-tree run with the active repository rules also passed with no findings.
  The first integration push used the explicit `security/forbidden-strings` lifecycle escape because intermediate commits
  retained the old remote violation before later commits removed it;
  every other policy remained enforced.
  Hosted push workflow [29130940967](https://github.com/Aquaticat/Monochromatic/actions/runs/29130940967) then passed its
  independent full-tree scan at revision `295a61834`.
- A following unescaped incremental push exposed a path-semantics mismatch:
  explicit `candidate-N` temporary paths bypassed the scanner's `--all` exclusions and reported its canonical rules
  sources as findings.
  The wrapper blocked the push and left remote `main` unchanged.
  Commit `298876aeb` filters the configured rules file and four canonical self-match sources before materialization while
  retaining unrelated nested basename candidates.
  Focused unit,
  type,
  Oxlint,
  cli-git build,
  and packed trust checks pass.
  After refreshing the exact stored TypeScript config bundle,
  an unescaped real wrapper push advanced `main` to `bd27bdd10`;
  hosted full-tree workflow [29131221984](https://github.com/Aquaticat/Monochromatic/actions/runs/29131221984) passed.
- Key implementation checkpoints are `b9dc927ef`,
  `a5918a490`,
  `bacfe57b1`,
  `54b316aae`,
  `df7e6dec0`,
  `18d7940c3`,
  `6eccb3064`,
  `c35e012b6`,
  `a8baa6024`,
  `f621432a6`,
  `470c0f5dc`,
  `3d8c240da`,
  `a9096c04a`,
  `295a61834`,
  and `298876aeb`.

### Issue #342 filesystem identity

- Added `@monochromatic-dev/module-fs-id` with source-qualified colon-free IDs,
  stable and warned degraded results,
  adapter fixtures,
  fresh per-call volume observation,
  and a narrow package-root interface.
- Linux uses `findmnt` UUID with GNU `stat` `f_fsid` fallback.
- macOS maps arbitrary paths through `df -P`,
  parses structured `diskutil` plist Volume UUID,
  and falls back to BSD `stat` device identity.
- Windows reads locale-invariant CIM volume serial and accepts only a nonzero Node-stat device fallback.
- Build,
  type lint,
  Oxlint with zero warnings,
  unit tests,
  README lint,
  and a disposable built consumer call pass.
- Workflow run [29064685354](https://github.com/Aquaticat/Monochromatic/actions/runs/29064685354) passed preferred and
  degraded evidence on Linux,
  macOS,
  and Windows.
- A physical macOS 26.5.2 host at `ssh m1` independently passed the same script;
  its tiny disposable directory was removed after verification.
- Commits `5b5b6b098`,
  `b7636ce69`,
  `9641fb381`,
  `b058283d2`,
  and `91db0a466` contain the package,
  fixes,
  tests,
  host evidence,
  and required troubleshooting record.
- Independent final review reported no required corrections.

### Issue #341 contract

- Reconciled `doc/decision/cli-git-policies-platform.md` with every settled grilling revision.
- Added canonical implementation interface `package/git-policy/cli/SPEC.md`.
- A disposable TypeScript 7.0.1-rc consumer compiled both authoring examples and proved statically known unknown policy IDs
  and wrong option values are rejected.
- An Optique 1.1.1 fixture parsed every documented management form and rejected missing,
  conflicting,
  empty,
  and misplaced scope forms.
- Repository Markdown lint passes for the decision,
  spec,
  and handover.
- Commits `0a22e16e3` and `32c938a8b` contain the contract and final strict-TypeScript correction.
- Independent final review reported no required corrections and #341 was closed.

## Final-newline migration state

Issue `#355` implements `final-newline` as an enabled-by-default core policy at error severity.
The core order places it after `add-explicit`.
It is warn-safe because warning mode reports without applying patches.

Exact normalization collapses terminal LF runs to one LF and appends one LF when absent.
It preserves interior CRLF bytes and skips empty,
NUL-containing,
invalid UTF-8,
deleted,
symlink,
submodule,
and exact excluded candidates.
The migrated exclusions match hk:
`package/fuzz/forbidden-strings/corpus/**`,
`package/test-fixture/toml-edit/src/**`,
and `**/dist/final/node/**`.

Commit correction applies full-content single-path Git patches to a private index.
Direct fix requires explicit pathspecs or `--all`,
converges whole-policy passes in a private index,
revalidates worktree bytes,
installs atomically,
and leaves the real index byte-identical.
Successful correction emits one aggregate policy-neutral `fix-summary` after stable convergence;
provisional findings are not emitted.
Read-only check and manual push never carry patches.

Acceptance fixtures now cover eight changed passes,
cross-policy cycles through streamed exact path,
mode,
and content snapshots,
Git-byte-ordered summary paths,
canonical and blocked interactive selection modes,
the hk partial-staging regression,
all exclusion families,
and packed check,
fix,
commit,
and push invocations.
Private candidate metadata loads use deterministic 64-lane mapping rather than repository-sized process fan-out.
The 2026-07-11 checkout contained 7,321 tracked paths;
an end-user `git cli-git check --policy final-newline --all` completed successfully in 47 seconds with no findings.
Performance acceptance and optimization beyond that behavior proof belong to issue `#356`.

The final verification set passed cli-git build,
zero-warning Oxlint,
type checking,
full unit tests,
packed trust and lifecycle consumption,
Markdown lint,
and the independent full-tree forbidden-strings scan.
A packed-test registry race discovered during acceptance was traced to readers observing lock-owner JSON between
exclusive creation and write;
`registry-recursive-lock.ts` now publishes owner metadata through an atomic sibling rename,
and repeated 32-contender regression coverage passes.
The committed manual-push benchmark is type-checked under the cli-git package and remains separate from the shipped
single-MJS artifact.

## Release-readiness checkpoint on 2026-07-11

Issues `#354` through `#357` are complete and closed.
Issue `#356` passed its independent standards and specification review after every required correction landed.
Issue `#357` then completed hk/Pkl retirement and its own independent review.
Npm publication remains deferred to `#358` and is not authorized.

Final hosted evidence at release commit `375dd8ea9`:

- final-newline workflow run `29171565809` passed typed shell-free orchestration and direct checking;
- lifecycle-performance workflow run `29171565793` passed all paired budgets and uploaded raw evidence;
- forbidden-strings workflow run `29171565821` passed independently;
- cross-platform trust workflow run `29171565815` passed Linux,
  macOS,
  and Windows,
  including strict and relaxed MJS and TypeScript trust,
  real filesystem identity,
  registry paths,
  and Windows ACL rejection.

The Windows sequence fixed two host-only defects:
real Git resolution now follows PATH directory order plus Windows `PATHEXT`,
and every newly created nested trust-registry component receives its own protected ACL before deeper creation.
Missing records are classified before ACL probing so Windows absence remains `untrusted` rather than `corrupt`.
Troubleshooting evidence lives in
`doc/troubleshooting/cli-git-windows-pathext-resolution.md` and
`doc/troubleshooting/cli-git-windows-nested-registry-acls.md`.

The measured lifecycle baseline is
`package/git-policy/cli/perf/lifecycle-latency-2026-07-16.json`.
Its scenario-specific ceilings are derived as twice the measured maximum rounded to 25 milliseconds,
and every ceiling is below 2,000 milliseconds.
The performance workflow stores each run's complete raw-sample JSON as a retained CI artifact.
The artifact from run `29171565793` was downloaded and parsed successfully:
all required scenarios contain 30 direct and wrapped samples and use the wrapper-added metric.
The final review also broadened trust workflow triggers to the complete `package/git-policy/**`,
fs-id,
and fs-path dependency scope.
The dirty-worktree commit path improved from 18.64 seconds to 0.74 seconds after post-commit content policies were
restricted to the landed delta while complete landed-tree metadata remained available.

The unpublished npm tarball passes the bounded non-workspace packed consumer.
The audit reduced it from 221 entries and 494,783 compressed bytes to 146 entries and 402,357 bytes;
unit tests,
host-evidence programs,
and packed fixture sources are now excluded while `src/index.ts`,
the one `index.mjs` artifact,
and `index.d.mts` remain.
Publication workflow state was not changed.

## Retirement checkpoint on 2026-07-11

Issue `#357` removed root `hk.pkl`,
obsolete `.idea/pklSettings.xml`,
hk and Pkl declarations from canonical `mise.no-env.toml`,
the managed `mise.toml` output,
and four legacy or active lockfile blocks.
Commit `122c5dbd2` records that infrastructure boundary.

The first push after removal failed because this checkout still has local `hook.hk-pre-push` configuration and the
retired executable was no longer provisioned.
No real local or global Git configuration was changed.
The checkpoint used native `--no-verify` only to bypass that stale hook;
cli-git manual-push policies still executed and the push succeeded.

Commit `a9b07385c` adds `mise run cleanup:hk-git-config -- --local --global`.
The command resolves real Git through cli-git,
requires explicit scopes,
removes only keys beginning with `hook.hk-`,
and reports exact removed names.
`test:hk-config-cleanup` passes independent disposable unconfigured,
configured,
repeated,
local,
global,
unrelated-key preservation,
and root-task fixtures.
The fresh-context procedure is `doc/runbook/remove-retired-hk-git-config.md`.
Automated verification never touches real per-user Git state.

Current documentation now treats hk/Pkl behavior as historical evidence,
points local enforcement to cli-git,
and keeps independent scanner and final-newline CI authoritative.
Issues `#143` and `#160` are closed as superseded.

Final local build,
types,
zero-warning lint,
units,
cleanup fixture,
direct checks,
and complete packed non-workspace lifecycle passed.
The final tarball has 146 entries and 403,641 compressed bytes;
repository-only `src/maintenance` is absent.
Independent Standards and Specification review reported no unresolved required finding.

Final hosted evidence at retirement commit `f771a237a`:

- forbidden-strings run `29172447991` passed;
- final-newline run `29172447995` passed;
- lifecycle-performance run `29172447986` passed;
- cross-platform trust run `29172448009` passed Linux,
  macOS,
  and Windows.

Issue `#357` is closed.
Npm publication remains deferred to `#358`.

## GitHub implementation tracker

The confirmed plan is recorded as dependency-linked GitHub issues.
Dependency-ordered implementation slices:

- [#341](https://github.com/Aquaticat/Monochromatic/issues/341),
   completed:
  finalized the canonical decision and implementation contract.
- [#342](https://github.com/Aquaticat/Monochromatic/issues/342),
   completed:
  built and verified the cross-platform filesystem-ID prerequisite.
- [#343](https://github.com/Aquaticat/Monochromatic/issues/343),
  completed:
  prepared and externally verified the npm tarball boundary and side-effect-free public API without publishing.
- [#344](https://github.com/Aquaticat/Monochromatic/issues/344),
  completed:
  ran `require-root` through the packaged JSONL engine with Optique management commands and built consumer evidence.
- [#345](https://github.com/Aquaticat/Monochromatic/issues/345),
  completed:
  trusted and executed one stored MJS plugin snapshot with local,
  packed-bin,
  Windows ACL,
  and final independent review evidence.
- [#346](https://github.com/Aquaticat/Monochromatic/issues/346),
  completed:
  add recursive snapshot trust and cascading revocation.
- [#347](https://github.com/Aquaticat/Monochromatic/issues/347),
  completed:
  build and cache trusted TypeScript config.
- [#348](https://github.com/Aquaticat/Monochromatic/issues/348),
  completed:
  migrate configurable command safeguards.
- [#349](https://github.com/Aquaticat/Monochromatic/issues/349),
  completed:
  stage fixed command transforms.
- [#350](https://github.com/Aquaticat/Monochromatic/issues/350),
  completed:
  gate post-commit auto-push through policy lifecycle.
- [#351](https://github.com/Aquaticat/Monochromatic/issues/351),
  completed:
  deliver the first real private-index autofix transaction.
- [#352](https://github.com/Aquaticat/Monochromatic/issues/352),
  completed:
  harden commit modes and interrupted-index recovery.
- [#353](https://github.com/Aquaticat/Monochromatic/issues/353),
  completed:
  migrated forbidden-root-context as the first repo plugin.
- [#354](https://github.com/Aquaticat/Monochromatic/issues/354),
  completed:
  migrated forbidden-strings across commit and push lifecycle with bounded manual-push materialization.
- [#355](https://github.com/Aquaticat/Monochromatic/issues/355),
  completed:
  implemented transactional final-newline normalization,
  exact whole-policy convergence,
  packed lifecycle parity,
  and index-neutral direct check/fix.
- [#356](https://github.com/Aquaticat/Monochromatic/issues/356),
  completed:
  independent CI,
  cross-platform trust,
  paired measured performance gates,
  raw CI artifacts,
  JSONL compatibility,
  npm-pack readiness,
  user documentation,
  and independent review passed without publishing.
- [#357](https://github.com/Aquaticat/Monochromatic/issues/357),
  completed:
  hk/Pkl infrastructure was removed,
  exact cleanup passed disposable fixtures,
  documentation and related issues were migrated,
  and all capstone gates passed.

Deferred issue:

- [#358](https://github.com/Aquaticat/Monochromatic/issues/358):
  actually publish the npm package and verify the registry artifact.
  It is recorded now,
  labeled `needs-triage`,
  and deferred indefinitely pending an explicit maintainer resume decision.

Existing #139 remains relevant to packaged CLI shebang behavior.
Issues #143 and #160 are closed as superseded by the completed cli-git platform and hk/Pkl retirement.

## Implementation sequence

This is platform-first and checkpointed.
Each phase receives scoped commits as soon as changes exist;
a later phase does not wait to record an earlier phase's work.

### Reconcile the decision and write the implementation contract

- Update `doc/decision/cli-git-policies-platform.md` from this handover:
  unified policies,
  exact snapshots,
  two-stage recursive trust,
  npm distribution,
  fixed-transform ordering,
  global autofix passes,
  and post-commit exits.
- Write the implementation spec deferred by that decision.
- Freeze public TypeScript API shapes,
  JSONL event schema,
  management grammar,
  trust-record schema,
  and benchmark methods before production implementation depends on them.
- Verify examples by compiling and invoking the smallest consumer fixture.

### Build the filesystem-ID prerequisite

- Implement `@monochromatic-dev/module-fs-id` from its completed contract now documented in
  `package/module/fs-id/README.md`.
- Keep OS command execution behind injected adapters so Linux,
  macOS,
  and Windows branches have fixture coverage.
- Verify real command output on all three operating systems through CI or real hosts before trusting parser claims.
- Guarantee a registry-path-safe ID representation and exact stable/degraded metadata.
- Finish the package only after README,
  lint,
  type checks,
  unit tests,
  and a consuming import call pass.

### Reshape the npm-ready cli-git package boundary

- Separate side-effectful bin startup from side-effect-free public API modules.
- Export `defineConfig`,
  `definePlugin`,
  `definePolicy`,
  Valibot-backed helpers,
  and public types from `@monochromatic-dev/git-policy-cli`.
- Make the package manifest npm-ready,
  add publication metadata without enabling registry upload,
  and include only required build artifacts,
  source declarations,
  README,
  and licenses.
- Bundle private workspace helpers such as fs-id and logger into the public Node artifact rather than publishing them as
  unresolved private runtime dependencies.
- Verify `npm pack`,
  install the tarball in a disposable non-workspace project,
  import the API,
  place the bin first on `PATH`,
  and run both wrapper and management commands.
- Do not enable publish workflow or upload the package;
  those actions belong only to indefinitely deferred issue #358.

### Implement the core policy engine and migrate current safeguards

- Add immutable parsed-command facts and lazy memoized Git context methods.
- Add Valibot config and option validation.
- Add unified findings with optional single-path patch bytes.
- Add default severity,
  warn-safety,
  namespace registration,
  core/plugin ordering,
  lifecycle escape hatches,
  and `--cli-git-keep-going`.
- Add final-pass-only JSONL and settled exit-code behavior.
- Split fixed transformers from configurable policies and make transforms idempotent.
- Migrate `require-root`,
  `linked-worktree-only`,
  `branch-worktree-only`,
  and `add-explicit` with behavior-parity fixtures before deleting their old rule functions.
- Keep atomic-push,
  commit-only,
  status-hints-off,
  and auto-push fixed while moving them into the staged pipeline.

### Implement config discovery and trust

- Build the read-only and mixed-command classifier from current Git documentation and parser fixtures.
- Implement account-derived registry paths,
  per-key snapshot directories,
  atomic writes,
  permissions,
  canonicalization,
  and symlink defenses.
- Implement MJS self-contained validation,
  exact snapshot copies,
  and execution from stored bytes.
- Prototype tsdown programmatic output,
  local graph extraction,
  one-chunk enforcement,
  assets/native failures,
  and immutable cache loading before landing the TypeScript path.
- Implement strict snapshot comparison,
  relaxed metadata-triggered rebuilds,
  comma/percent `CLI_GIT_NO_PARANOID` parsing,
  malformed-entry warnings,
  and no global discovery bypass.
- Implement two-stage recursive trust,
  cross-filesystem inheritance,
  child auto-enrollment,
  provenance,
  nested outer-root revocation,
  and explicit CI trust.
- Exercise races and interrupted writes with concurrent disposable processes.

### Prototype and implement transactional autofix

- Before editing production transaction code,
  build a disposable Git prototype for copied indexes,
  explicit-path commit indexes,
  post-commit index reconciliation,
  exact worktree preservation,
  and crash journals.
- Prove ordinary index commits,
  injected-only commits,
  amend,
  allow-empty,
  merge/cherry-pick/revert conclusions,
  deletions,
  untracked selected paths,
  partial staging,
  unrelated staging,
  hook failure,
  patch conflict,
  and process interruption.
- Apply engine-owned patch files with `git apply --cached --3way` against private indexes.
- Implement eight whole-sequence passes with exact byte comparison,
  cycle detection,
  provisional changed-pass findings,
  and engine-stop semantics.
- Install a real index only after successful commit and journal recovery checks.
- Leave unsupported interactive modes read-only and block only when a fix is required.

### Add repo policies in migration order

- Add `forbidden-root-context` first to prove a plugin validator/finding shape.
- Add `forbidden-strings` next to prove candidate content,
  committed ground truth,
  manual push ranges,
  external binary failures,
  and post-commit auto-push gating.
- Add `final-newline` last to prove patch fixes,
  global convergence,
  exact exclusions,
  direct check/fix,
  and partial-staging safety.
- Preserve every source-decision exclusion and old hk trigger path in named parity fixtures.

### Complete lifecycle, CI, performance, and package-readiness verification

- Implement direct `check`,
  `fix`,
  trust,
  untrust,
  and status user-boundary tests through the built shim.
- Add independent final-newline CI before hk removal;
  keep forbidden-strings CI independent of the wrapper.
- Measure no-config,
  read-only,
  strict MJS,
  strict cached TypeScript,
  relaxed TypeScript rebuild,
  validator,
  scanner,
  normalizer,
  and post-commit paths.
- Set budgets from measured baselines in the implementation spec and fail repeatable benchmark fixtures when they regress.
- Run package lint,
  type checks,
  tests,
  npm tarball install,
  and cross-platform trust verification.
- Obtain independent standards and specification review before retirement.
- Stop at verified npm tarball readiness;
  actual registry publication remains outside the active sequence in #358.

### Retire hk and Pkl as the capstone

- Confirm all parity,
  performance,
  CI,
  trust,
  documentation,
  and public-package gates are green.
- Remove hk and Pkl config/tool references,
  stale IDE config,
  and obsolete documentation named by the source decision.
- Provide and verify idempotent per-machine hk Git-config cleanup.
- Verify the end-user `git` shim,
  direct policy commands,
  CI checks,
  and clean npm install after removal.

## Authorized linked-worktree follow-up

A later authorized follow-up adds fixed lifecycle infrastructure that copies the invoking worktree's complete
Git-ignored filesystem state into every newly registered linked worktree.
Outcome-based administrative identity comparison covers ordinary aliases,
`--no-checkout`,
and worktrees retained after Git or hook failure.
The settled contract is now canonical in
`doc/decision/cli-git-policies-platform.md` and
`package/git-policy/cli/SPEC.md`.

The implementation uses destination-sibling private staging,
copy-on-write with full-copy fallback,
exact final source equivalence,
non-overwriting installation,
registered nested-worktree exclusion,
ownership-checked rollback,
and validated common-directory journal recovery.
Bare repositories contribute an empty source set.
Copy-only failure exits `2`;
a concurrent Git failure retains its numeric status or signal fallback `1`.

Built-wrapper fixtures cover the standard ignore stack,
permissions,
symlinks,
alias and no-checkout registration,
source-only ignore state,
exact and differing collisions,
hook failure,
signaled Git failure,
nested and stale registered worktrees,
bare repositories,
special files,
journal recovery,
and malicious journal containment.
Issue [#393](https://github.com/Aquaticat/Monochromatic/issues/393)
separately tracks Rolldown's TypeScript trust bootstrap dependency;
`doc/troubleshooting/rolldown-1-2-typescript-trust-tsconfig-discovery.md`
records the source trace and verified `tsconfig: false` boundary.

## Remaining work

The earlier evidence gaps and issues `#356` and `#357` are closed.
No active cli-git platform implementation slice remains.
Do not publish npm artifacts;
`#358` remains indefinitely deferred.

## Required verification before implementation completion

- Unit tests for every policy metadata,
  severity,
  ordering,
  default,
  escape-hatch,
  JSONL,
  and error branch.
- Disposable Git integration fixtures for allowed and rejected commands.
- Exact-byte partial-staging and worktree-preservation tests.
- Trust tests for first use,
  changed content,
  recursive inheritance,
  revocation provenance,
  mount swaps,
  malformed environment entries,
  registry races,
  and journal recovery.
- Real macOS,
  Linux,
  and Windows filesystem-ID verification.
- End-user invocation through the built `git` shim,
  not direct module calls only.
- Direct `git cli-git trust`,
  `check`,
  `fix`,
  `untrust`,
  and status invocations.
- Performance gates measured on no-config,
  trusted MJS,
  trusted cached TypeScript,
  validator-only,
  scanner,
  and normalizer paths.
- Package README,
  zero-warning lint,
  type checks,
  and tests covering every exported path.
- Independent review against both repository standards and the final implementation spec.

## Keep updated

Update this file immediately after each settled decision,
important source finding,
prototype result,
implementation checkpoint,
verification result,
and commit.

Issues `#341` through `#357` completed their contract gates.
Resume only if an explicitly authorized follow-up changes the platform or reactivates deferred publication issue `#358`.
