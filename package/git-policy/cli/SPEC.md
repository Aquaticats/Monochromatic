# cli-git policy platform implementation specification

## Status and authority

This file is the canonical implementation interface for
`doc/decision/cli-git-policies-platform.md`.
Runtime code,
public declarations,
tests,
and user documentation must conform to it.
Changing a settled product behavior requires a new decision update rather than an implementation shortcut.

Npm registry publication is not part of this specification.
Issue #358 records that indefinitely deferred action.

## Module seams

The package has these external seams:

- the shadowing `git` executable;
- the side-effect-free authoring and optional-policy exports from `@monochromatic-dev/git-policy-cli`;
- repository-root config artifacts;
- JSONL policy events;
- the per-account exact-snapshot trust registry.

The package has internal seams for filesystem identity,
real-Git execution,
account-home lookup,
clock,
registry storage,
and prompts.
Production adapters own operating-system effects.
Disposable tests replace those adapters without production environment overrides.

Importing authoring or optional-policy exports must not inspect process arguments,
read files,
resolve Git,
resolve external scanner executables,
write output,
register a policy,
or start the executable.

## Single artifact and shipped optional policies

The public cli-git tarball contains exactly one MJS artifact.
That artifact exports authoring declarations and optional repo-owned policies,
and is also the executable named by the `git` bin entry.
Cli-git's own module graph uses static imports and must not emit shared chunks,
secondary MJS entries,
or package-relative dynamic imports.
Bundled library implementations may retain their dynamic imports.
Exact stored MJS configuration execution may use one computed dynamic import so top-level `await` remains supported.
Neither exemption may emit another artifact or load an untrusted package-relative target.
Direct-invocation detection calls CLI startup only when Node executes the artifact as the program entry.
Importing it as a module remains side-effect free.

Repo-owned policy implementations remain in separate workspace source packages for ownership and focused tests.
The cli-git build statically bundles them into the one artifact.
No private workspace policy package may remain as a runtime or declaration dependency of the packed artifact.

Importing an optional policy returns its plugin definition only.
It never changes the built-in registry or enabled policy set.
A consumer enables it by registering the plugin under a chosen namespace in trusted config.
A no-config invocation therefore retains built-ins only.

The root package export includes `repositoryPolicyPlugin` and,
after issue #354 lands,
`forbiddenStringsPlugin`.
The forbidden-strings export bundles its Node adapter but not the Rust scanner binary.
Scanner lookup defaults to `forbidden-strings` on `PATH` only when the enabled policy executes.

## Public authoring declarations

The public package exports the following semantic declarations.
Implementation may split them across source files,
but the package root re-exports them.

```ts
import type * as v from 'valibot';

export type PolicySeverity = 'off' | 'warn' | 'error';

export type ActivePolicySeverity = Exclude<PolicySeverity, 'off'>;

export type PolicyTrigger =
  | 'pre-forward'
  | 'post-commit'
  | 'manual-push'
  | 'direct-check'
  | 'direct-fix';

export type GitObjectId = string;

export declare const ABSENT_GIT_VALUE: unique symbol;

export type AbsentGitValue = typeof ABSENT_GIT_VALUE;

export type RepositoryPath = string;

export type CandidateFileMode =
  | 'regular'
  | 'executable'
  | 'symlink'
  | 'submodule';

export type CandidateChange =
  | 'added'
  | 'modified'
  | 'deleted';

export type CandidateFile = {
  readonly targetId: string;
  readonly path: RepositoryPath;
  readonly revision: GitObjectId | AbsentGitValue;
  readonly mode: CandidateFileMode;
  readonly change: CandidateChange;
  readonly bytes: () => Promise<Uint8Array>;
};

export type PushUpdate = {
  readonly localOid: GitObjectId | AbsentGitValue;
  readonly remoteOid: GitObjectId | AbsentGitValue;
  readonly remoteName: string;
  readonly remoteRef: string;
};

export type PolicyCommandFacts = {
  readonly rawArgs: readonly string[];
  readonly transformedArgs: readonly string[];
  readonly subcommand: string | AbsentGitValue;
  readonly effectiveCwd: string;
  readonly repositoryRoot: string;
  readonly escapedPolicyIds: ReadonlySet<string>;
};

export type LazyPolicyGitFacts = {
  readonly candidates: () => Promise<readonly CandidateFile[]>;
  readonly headOid: () => Promise<GitObjectId | AbsentGitValue>;
  readonly landedCommitOid: () => Promise<GitObjectId | AbsentGitValue>;
  readonly pushUpdates: () => Promise<readonly PushUpdate[]>;
};

export type PolicyContext = {
  readonly candidateVersion: number;
  readonly trigger: PolicyTrigger;
  readonly command: PolicyCommandFacts;
  readonly git: LazyPolicyGitFacts;
  readonly signal: AbortSignal;
};

export type FindingLocation = {
  readonly byteStart: number;
  readonly byteEnd: number;
};

export type PolicyPatch = {
  readonly kind: 'git-unified';
  readonly targetId: string;
  readonly path: RepositoryPath;
  readonly bytes: Uint8Array;
};

export type PolicyFinding = {
  readonly code: string;
  readonly message: string;
  readonly path?: RepositoryPath;
  readonly location?: FindingLocation;
  readonly patch?: PolicyPatch;
};

export type PolicyCheckInput<TOptions> = {
  readonly context: PolicyContext;
  readonly options: Readonly<TOptions>;
};

export type PolicyDefinition<
  TOptions = undefined,
  TName extends string = string,
> = {
  readonly name: TName;
  readonly defaultSeverity: PolicySeverity;
  readonly warnSafe: boolean;
  readonly triggers: readonly PolicyTrigger[];
  readonly options?: Readonly<v.GenericSchema<unknown, TOptions>>;
  readonly check: (input: PolicyCheckInput<TOptions>) => Promise<readonly PolicyFinding[]>;
};

export type PluginDefinition<
  TPolicies extends readonly PolicyDefinition<unknown>[] =
    readonly PolicyDefinition<unknown>[],
  TName extends string = string,
> = {
  readonly name: TName;
  readonly policies: TPolicies;
};

export type PolicySetting<TOptions = unknown> =
  | PolicySeverity
  | readonly [PolicySeverity, TOptions];

export type BuiltInPolicyId =
  | 'require-root'
  | 'linked-worktree-only'
  | 'branch-worktree-only'
  | 'add-explicit';

export type PluginMap = Readonly<Record<string, PluginDefinition>>;

export type CliGitConfig<TPlugins extends PluginMap = PluginMap> = {
  readonly plugins?: TPlugins;
  readonly policies?: Readonly<Record<string, PolicySetting>>;
  readonly trust?: {
    readonly children?: boolean;
  };
};

type CliGitConfigInput = {
  readonly plugins?: PluginMap;
  readonly policies?: Readonly<Record<string, unknown>>;
  readonly trust?: {
    readonly children?: boolean;
  };
};

type ConfigPlugins<TConfig extends CliGitConfigInput> =
  TConfig extends { readonly plugins: infer TPlugins extends PluginMap }
    ? TPlugins
    : never;

type ConfigPolicies<TConfig extends CliGitConfigInput> =
  TConfig extends {
    readonly policies: infer TPolicies extends Readonly<Record<string, unknown>>;
  }
    ? TPolicies
    : never;

type PluginPolicyForId<
  TPlugins extends PluginMap,
  TId extends string,
> = TId extends `${infer TNamespace}/${infer TName}`
  ? TNamespace extends keyof TPlugins
    ? Extract<TPlugins[TNamespace]['policies'][number], PolicyDefinition<unknown, TName>>
    : never
  : never;

type AllowedPolicySetting<
  TConfig extends CliGitConfigInput,
  TId extends PropertyKey,
> = TId extends BuiltInPolicyId
  ? PolicySeverity
  : TId extends string
    ? [PluginPolicyForId<ConfigPlugins<TConfig>, TId>] extends [never]
      ? never
      : PluginPolicyForId<ConfigPlugins<TConfig>, TId> extends PolicyDefinition<infer TOptions>
        ? PolicySetting<TOptions>
        : never
    : never;

type CheckedPolicySettings<TConfig extends CliGitConfigInput> = {
  readonly [TId in keyof ConfigPolicies<TConfig>]:
    ConfigPolicies<TConfig>[TId] extends AllowedPolicySetting<TConfig, TId>
      ? ConfigPolicies<TConfig>[TId]
      : never;
};

export declare function definePolicy<
  const TName extends string,
  const TOptions = undefined,
>(
  definition: Readonly<PolicyDefinition<Readonly<TOptions>, TName>>,
): PolicyDefinition<Readonly<TOptions>, TName>;

export declare function definePlugin<
  const TName extends string,
  const TPolicies extends readonly PolicyDefinition<unknown>[],
>(
  definition: PluginDefinition<TPolicies, TName>,
): PluginDefinition<TPolicies, TName>;

export declare function defineConfig<const TConfig extends CliGitConfigInput>(
  config: TConfig
    & CliGitConfig<ConfigPlugins<TConfig>>
    & { readonly policies?: CheckedPolicySettings<TConfig> },
): TConfig;

export declare function definePolicyOptions<const TInput, const TOutput>(
  schema: Readonly<v.GenericSchema<TInput, TOutput>>,
): v.GenericSchema<TInput, TOutput>;
```

### Declaration invariants

- `definePolicy`,
  `definePlugin`,
  `defineConfig`,
  and `definePolicyOptions` return their argument unchanged.
- `defineConfig` derives valid plugin policy IDs and option outputs from the concrete namespace map and preserved policy
  tuple.
  Unknown IDs and wrong option values are TypeScript errors when definitions are statically known.
- Runtime config loading still validates the resulting value rather than trusting TypeScript types.
- Policy and plugin names are non-empty kebab-case identifiers without `/`.
- Consumer namespaces are non-empty kebab-case identifiers without `/`.
- Built-in IDs are their policy names.
- Plugin IDs are `<namespace>/<policy-name>`.
- Duplicate effective IDs are config failures.
- A policy default may be `off`.
- Omitted options become the policy schema's documented default only when the schema supplies one.
  Otherwise omitted required options are a config failure.
- A policy without an option schema receives `undefined`.
- Plugin declaration order is the array order.
- Config namespace order is JavaScript own-string-key insertion order.
- The engine copies returned arrays,
  finding objects,
  sets,
  argument arrays,
  and patch bytes before retaining them.
- `RepositoryPath` values use Git's slash-separated repository-relative form.
  Absolute paths,
  empty paths,
  `.` segments,
  `..` segments,
  NUL,
  and backslash separators are invalid.
- `GitObjectId` is opaque to plugins.
  The engine validates object IDs before invoking Git.
- `targetId` is invocation-local and opaque.
  Plugins must return the exact target ID supplied by a candidate.
- `revision` is `ABSENT_GIT_VALUE` for a mutable candidate and a Git object ID for immutable historical content.
- Deleted and submodule candidates reject `bytes()` with a typed engine-owned unavailable-content error.
- `bytes()` returns a fresh copy on every call.
- Lazy methods memoize success or failure for one `candidateVersion`.
- Any candidate mutation increments `candidateVersion` and creates a new context.
- Policy cancellation uses `signal`.
  Returning after cancellation is ignored;
  throwing because of cancellation remains an engine event rather than a finding.

### Policy completion

A check completes only after its returned promise resolves and every lazy operation it awaited completed successfully.
An exception,
rejection,
invalid return value,
invalid finding,
or required unavailable candidate is an engine failure.
An empty finding array is a successful clean result.

The engine invokes one policy at a time.
Policy code may use `Promise.all` internally when its own checks are independent.

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

A patch is valid only when all conditions hold:

- trigger is `pre-forward` or `direct-fix`;
- finding has `path`;
- patch path equals finding path;
- patch target ID identifies the same mutable candidate and path;
- revision is `ABSENT_GIT_VALUE`;
- patch bytes are one textual unified Git diff;
- old and new paths resolve to exactly the candidate path;
- no rename,
  copy,
  mode change,
  submodule,
  binary patch,
  absolute path,
  traversal,
  or second path is present;
- applying with Git three-way semantics changes bytes;
- resulting candidate remains a regular or executable file;
- direct fix preserves the original file mode.

A patch that applies but produces identical bytes is invalid rather than a convergence change.
Patch validation failure exits `2` and identifies the policy and path without echoing patch bytes.

## Authoring example

This minimal plugin rejects a root context file.

```ts
import {
  defineConfig,
  definePlugin,
  definePolicy,
  type PolicyFinding,
} from '@monochromatic-dev/git-policy-cli';

const forbiddenRootContext = definePolicy({
  name: 'forbidden-root-context',
  defaultSeverity: 'error',
  warnSafe: true,
  triggers: [
    'pre-forward',
    'direct-check',
  ],
  async check({ context, }): Promise<readonly PolicyFinding[]> {
    const candidates = await context.git.candidates();
    const hasRootContext = candidates.some(
      function isRootContext(candidate,): boolean {
        return candidate.path === 'CONTEXT.md'
          && candidate.change !== 'deleted';
      },
    );

    if (!hasRootContext)
      return [];

    return [{
      code: 'root-context-forbidden',
      message: 'Root CONTEXT.md is forbidden; read source directly.',
      path: 'CONTEXT.md',
    },];
  },
});

const repositoryPlugin = definePlugin({
  name: 'repository',
  policies: [forbiddenRootContext,],
});

export default defineConfig({
  plugins: {
    mono: repositoryPlugin,
  },
  policies: {
    'mono/forbidden-root-context': 'error',
  },
  trust: {
    children: true,
  },
});
```

A policy with options passes a Valibot schema directly.

```ts
import * as v from 'valibot';
import {
  definePolicy,
  definePolicyOptions,
  type PolicyFinding,
} from '@monochromatic-dev/git-policy-cli';

const options = definePolicyOptions(v.object({
  marker: v.optional(v.string(), 'forbidden'),
}),);

export const markerPolicy = definePolicy({
  name: 'marker',
  defaultSeverity: 'off',
  warnSafe: true,
  triggers: ['direct-check',],
  options,
  async check({ options: parsedOptions, }): Promise<readonly PolicyFinding[]> {
    return [{
      code: 'configured-marker',
      message: parsedOptions.marker,
    },];
  },
});
```

Consumer-built MJS must bundle the authoring imports away.
A hand-written self-contained MJS artifact may export the equivalent raw object.

## Configuration validation

Config loading performs these steps in order:

1. Validate top-level object shape.
2. Validate plugin namespaces and plugin values.
3. Register built-ins in fixed order.
4. Register plugins in namespace and declaration order.
5. Resolve each effective policy ID.
6. Apply omitted declared defaults.
7. Parse configured option values through the policy's Valibot schema.
8. Emit configuration warnings for explicit unsafe `warn` settings.
9. Validate trust declaration.

Unknown policy IDs,
duplicate IDs,
invalid severities,
missing required options,
options supplied to a schema-less policy,
and Valibot failures exit `2` before any policy runs.
Valibot issues are rendered into an engine-failure event without serializing arbitrary schema objects.

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
or ambiguous form loads trusted config.

Short-circuit global version and help forms preserve real Git behavior and do not load config.
Management dispatch occurs only when the parsed subcommand is exactly `cli-git`.
A pathspec named `cli-git` does not dispatch management.

## Management grammar

The accepted grammar is:

```text
git [<git-global-options>] cli-git trust [--yes]
git [<git-global-options>] cli-git untrust
git [<git-global-options>] cli-git status
git [<git-global-options>] cli-git check [--policy <id>]... --all
git [<git-global-options>] cli-git check [--policy <id>]... -- <pathspec>...
git [<git-global-options>] cli-git fix [--policy <id>]... --all
git [<git-global-options>] cli-git fix [--policy <id>]... -- <pathspec>...
```

Rules:

- Management parsing uses Optique.
- `--yes` is valid only for `trust`.
- `--policy` is repeatable only for `check` and `fix`.
- Repeated identical policy IDs are deduplicated while preserving first occurrence order.
- `--all` and `-- <pathspec>...` are mutually exclusive and one is required.
- The pathspec form requires at least one token after `--`.
- Tokens after `--` are passed to real Git pathspec expansion without wrapper reinterpretation.
- Unknown management flags and extra positionals are usage errors with exit `2`.
- A usage error never loads repository config.
- `check` and `fix` load trusted config after grammar and scope validation.
- `trust`,
  `untrust`,
  and `status` never execute live repository config during preflight.
- `fix` changes selected worktree files and verifies every real index blob is unchanged.

Trust management emits one compact LF-terminated JSON object on stdout.
These management objects carry `schemaVersion` but no policy-event `sequence`.
Human trust and recursive-authority disclosures remain on stderr.

```ts
export type TrustSummary = {
  readonly schemaVersion: 1;
  readonly type: 'trust-summary';
  readonly configPath: string;
  readonly trusted: true;
};

export type TrustStatus = {
  readonly schemaVersion: 1;
  readonly type: 'trust-status';
  readonly configPresent: boolean;
  readonly trusted: boolean;
  readonly unchanged: boolean;
  readonly configPath?: string;
  readonly filesystemId?: string;
  readonly reason:
    | 'no-config'
    | 'untrusted'
    | 'trusted'
    | 'changed'
    | 'corrupt';
};

export type UntrustSummary = {
  readonly schemaVersion: 1;
  readonly type: 'untrust-summary';
  readonly configPath: string | null;
  readonly removed: boolean;
  readonly affectedRoots: readonly string[];
};
```

`status` inspects exact trust state without executing live config.
`configPath` and `filesystemId` are absent when no supported config is present.
Deleted-config recovery uses `null` for `UntrustSummary.configPath` because no canonical config path remains.
Trust management failures emit one schema-version-one `engine-failure` event on stdout and exit `2`.
A classified `TrustedConfigError` retains its stable code;
an unclassified management or recovery failure uses `trust-failed`.
This schema supersedes the temporary built-in policy inventory from the first policy-engine slice.

## Lifecycle

### Pre-forward

Built-in policies run against raw semantic command facts.
Fixed transforms then produce `transformedArgs`.
Plugin policies run against the predicted candidate set for the transformed command.
Applicable normalizer patches run through whole-sequence convergence.
Remaining errors block before real Git.

### Linked-worktree ignored-state synchronization

Before forwarding any Git invocation whose effective global options select a repository,
cli-git captures the common Git directory and linked-worktree administrative identity set.
After real Git and its hooks return,
cli-git compares the administrative set again.
Every new identity is a synchronization destination,
regardless of whether argv named `worktree add`,
an ordinary Git alias expanded to it,
or real Git returned nonzero after retaining registration.
A missing or stale pre-existing worktree root does not hide a newly created identity.

For a non-bare invocation,
the source is the invoking effective worktree root after Git and `post-checkout` settle.
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
There is no repository configuration for this behavior.

Before staging,
exclude every registered worktree root strictly nested under the source.
Private stage names are reserved and excluded at every path component.
Stage each destination separately in a mode-`0700` sibling directory so file cloning stays on the destination filesystem.
Copy regular files with an exclusive copy-on-write request whose unsupported-filesystem behavior falls back to a full
copy.
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
Immediate rollback visits only transaction-owned paths in child-first order;
it removes a selected path only when it still exactly matches the private stage,
and removes an unselected scaffold only when it remains empty.
Every path whose ownership cannot be proved remains in place and is named in the diagnostic.

Each destination uses a schema-versioned journal under
`<git-common-dir>/cli-git-worktree-copy/v1`.
Journal writes use a private no-follow temporary file,
file synchronization,
atomic rename,
and parent-directory synchronization where supported.
Persist selected-entry intent before destination mutation.
A completion phase makes stage removal and journal removal recoverable in either crash order.
A process-birth-identity lock serializes installation and recovery,
rejects live contention after bounded acquisition,
and reclaims stale PID reuse safely.

Every later repository invocation checks for pending journals before forwarding.
Recovery validates that journal paths remain canonical,
the stage is a private owned directory beside the destination,
the destination still resolves to a linked registration under the same common directory,
and every intended entry exists in the private manifest.
Malformed,
replaced,
missing,
or conflicting state fails closed without deleting the journal or a path outside the validated private stage.
Completed cleanup is resumed without reinstalling entries.

After all newly registered destinations settle successfully,
write exactly one human summary line to stderr.
Copy-only failure exits `2`.
If real Git returned nonzero and synchronization also fails,
preserve Git's numeric status;
use `1` when Git ended by signal without a numeric status.
If Git failed but synchronization succeeds,
preserve its status after writing the successful copy summary.

### Post-commit

After a successful non-dry-run commit,
resolve the landed OID from real Git rather than assuming `HEAD` text.
Run applicable post-commit policies against committed ground truth.
`landedCommitOid()` returns the exact resolved commit;
`candidates()` lazily enumerates its complete recursive tree and reads blobs by object ID rather than worktree path.
Only a clean or warning-only result allows auto-push.
A policy or engine failure leaves the commit intact,
blocks push,
returns `2`,
and emits `commit-landed` after the causal events.
Once the landed OID is known,
repository-root or candidate-fact setup failure emits `content-unavailable` plus the same explicit landed state.

### Manual push

Resolve actual local and remote updates with Git-native information.
Scan every content-bearing commit or tree state required by enabled policies.
A pure deletion has no content target.
An indeterminate content-bearing range is `content-unavailable` and exits `2`.
Manual push never applies policy patches.

### Direct check

Use worktree bytes selected by explicit pathspecs or `--all`.
Run applicable direct-check policies read-only.
Emit final findings to stdout.

### Direct fix

Use worktree bytes selected by explicit pathspecs or `--all`.
Apply eligible patches to private candidate state through whole-sequence convergence,
then atomically replace only changed selected worktree files.
Snapshot the complete real index before and after and fail if any index blob changes.
Emit final findings and one fix summary to stdout.

## Policy order and stopping

Each pass runs:

1. built-in policies in fixed order;
2. fixed transforms in fixed order;
3. plugin policies in namespace and declaration order.

The built-in order is
`require-root`,
`linked-worktree-only`,
`branch-worktree-only`,
then `add-explicit`.
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

The engine retains up to eight candidate states in private temporary storage.
States serialize as ordered entries of repository path,
mode,
revision identity,
and exact bytes.
No content digest participates.

After each pass:

- exact equality with the preceding state means stable;
- exact equality with any non-adjacent retained state means cycle failure;
- a changed eighth pass means pass-limit failure;
- otherwise start the next pass at the first built-in policy.

Stable warning-only or clean state exits `0` when Git does not run.
Stable state with errors exits `1`.
Cycle and pass-limit failures exit `2`.
Only events derived from the stable pass are emitted as findings.

## JSONL schema version 1

Each event is one compact JSON object followed by LF.
Fields not listed for an event are absent rather than carrying a JSON `null` value.
The in-process `ABSENT_GIT_VALUE` sentinel is never serialized.
Unknown fields may be added only in a backward-compatible schema revision.
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

`sequence` starts at `0` for each cli-git invocation and increments by one in emission order.
Trust management objects use the separate schema specified in `Management grammar`.

### Trust warning object

```ts
export type TrustWarningObject = {
  readonly schemaVersion: 1;
  readonly type: 'trust-warning';
  readonly code:
    | 'relaxed-entry-malformed'
    | 'relaxed-entry-filesystem-mismatch'
    | 'typescript-package-import-not-invalidated';
  readonly message: string;
};
```

A trust warning is not a policy event and has no `sequence`.
It is one compact LF-terminated JSON object on stderr.
It may accompany direct-command policy JSONL on stdout without corrupting that stream.

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
  readonly location?: FindingLocation;
  readonly fix: 'none' | 'available';
};
```

`code` is the complete `<policy-id>/<policy-local-code>` value.
Patch bytes are never emitted.
`available` means the final stable finding still offers a patch that was not applied in this read-only or unsupported
mode.
A corrected finding from a provisional pass is not emitted.
Consequently successful autofix emits only `fix-summary`,
not the corrected finding.

### Configuration warning event

```ts
export type ConfigurationWarningEvent = EventBase & {
  readonly type: 'configuration-warning';
  readonly trigger: PolicyTrigger;
  readonly policyId: string;
  readonly code: 'warn-unsafe';
  readonly message: string;
};
```

An explicitly configured warn-unsafe policy emits this non-blocking event even when its check returns no finding.
Configuration warnings share the same JSONL stream and invocation-local sequence as findings and engine failures;
they never write ad hoc prose to the machine stream.

### Core finding event

```ts
export type CoreFindingEvent = EventBase & {
  readonly type: 'core-finding';
  readonly trigger: 'pre-forward';
  readonly coreId: 'commit-only';
  readonly code:
    | 'commit-only/all-flag'
    | 'commit-only/pathspec-required'
    | 'commit-only/staged-changes-ignored';
  readonly message: string;
};
```

Core findings are expected rejections from fixed non-configurable behavior.
They block with exit `1`,
share policy-event sequencing,
and cannot be disabled or assigned a severity through repository config.

### Fix summary event

```ts
export type FixSummaryEvent = EventBase & {
  readonly type: 'fix-summary';
  readonly trigger: 'pre-forward' | 'direct-fix';
  readonly passes: number;
  readonly changedPaths: readonly RepositoryPath[];
};
```

`changedPaths` are unique and sorted by Git path byte order.
The event is absent when no bytes changed.

### Engine failure event

```ts
export type EngineFailureCode =
  | 'config-invalid'
  | 'config-untrusted'
  | 'config-changed'
  | 'core-incomplete'
  | 'plugin-threw'
  | 'policy-incomplete'
  | 'content-unavailable'
  | 'patch-invalid'
  | 'patch-conflict'
  | 'fix-cycle'
  | 'fix-pass-limit'
  | 'transaction-failed'
  | 'trust-failed';

export type EngineFailureEvent = EventBase & {
  readonly type: 'engine-failure';
  readonly code: EngineFailureCode;
  readonly message: string;
  readonly trigger?: PolicyTrigger;
  readonly policyId?: string;
  readonly path?: RepositoryPath;
};
```

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
Trust management summaries,
statuses,
and failures use stdout;
human trust disclosures use stderr.
Real Git inherits normal stdio.
Debug logs must not corrupt the selected JSONL event stream.

When real Git does not run:

- `0` means clean or warnings only;
- `1` means final error findings;
- `2` means usage,
  trust,
  config,
  plugin,
  patch,
  transaction,
  or engine failure.

When real Git runs,
preserve its exit code except for a landed commit followed by a post-commit policy or engine failure,
which returns `2`.
An ordinary failed auto-push preserves the successful commit result and exits `0` after surfacing the push failure.

## Trust registry schema version 1

### Account-derived root

Production registry roots are:

- Linux and macOS:
  `<os-account-home>/.local/state/cli-git/trust/v1`;
- Windows:
  `<os-account-home>/AppData/Local/cli-git/trust/v1`.

The account home comes from an operating-system account adapter,
not `HOME`,
`USERPROFILE`,
XDG,
AppData,
or repository environment variables.
Failure to resolve the account home is a trust failure.
Tests inject the complete registry root through an internal adapter unavailable from config or production environment.
The account-derived or injected root must be lexically identical to its native real path;
any symlink or junction in its ancestor chain fails closed.

### Reversible record path

A trust identity is:

```ts
export type TrustIdentity = {
  readonly filesystemId: string;
  readonly canonicalConfigPath: string;
};
```

Encode each UTF-8 identity field with unpadded base64url.
Do not hash it.
The record path is:

```text
<root>/records/<base64url-filesystem-id>/path/<path-chunk-0>/<path-chunk-1>/.../record.json
```

Split encoded canonical path into fixed 120-character chunks;
only the last chunk may be shorter.
An empty encoded value is invalid.
Decoding every segment must reproduce the exact identity stored in `record.json`.
A mismatch is corruption and fails closed.
This reversible hierarchy avoids filesystem component limits while preserving the complete identity.

### Record metadata

`record.json` is UTF-8 JSON with this shape:

```ts
export type TrustSourceRecord = {
  readonly canonicalPath: string;
  readonly snapshotFile: string;
  readonly size: string;
  readonly mtimeNanoseconds: string;
};

export type TrustRecord = {
  readonly schemaVersion: 1;
  readonly identity: TrustIdentity;
  readonly repositoryRoot: string;
  readonly format: 'mjs' | 'typescript';
  readonly sources: readonly TrustSourceRecord[];
  readonly executableSnapshotFile: string;
  readonly executableSize: string;
  readonly recursiveChildren: boolean;
  readonly authorizingRoots: readonly TrustIdentity[];
  readonly recordedAt: string;
};
```

Sizes and nanosecond values are decimal strings to avoid JSON integer precision loss.
`recordedAt` is an RFC 3339 UTC audit timestamp and never participates in trust equality.
Source order is canonical config entry first,
then canonical relative-module path byte order.
Authorizing roots use canonical path byte order and contain no duplicates.

Snapshot file paths are slash-separated record-relative names beneath `snapshots/`.
They cannot be absolute or contain empty,
`.` ,
or `..` segments.
MJS executes its stored source snapshot.
TypeScript executes its stored one-chunk ESM bundle.
No metadata field contains a digest or hash.

### Atomicity and permissions

Write a complete sibling temporary record directory,
fsync its files and directories where supported,
validate it by reopening with no-follow semantics,
then atomically exchange or rename it into place.
Interrupted temporary directories are ignored and recoverable.
Readers never follow symlinks or reparse-point substitutions inside the registry.

On POSIX:

- registry and record directories are mode `0700`;
- metadata and snapshots are mode `0600`;
- every registry-owned directory and file belongs to current effective account;
- unexpected ownership or group or other permission bits fail closed.

On Windows,
the registry adapter disables inheritance and applies an ACL limited to the current account and system administrators
to every registry directory,
metadata file,
and snapshot.
Every read independently verifies the protected ACL and required principals.
Failure to apply or verify protection fails closed.

### MJS self-containment

MJS preflight decodes strict UTF-8 and parses ECMAScript module syntax without execution.
Static imports,
re-exports,
and literal dynamic imports may name only Node built-ins.
Local paths,
package names,
computed dynamic imports,
and additional artifact assets are rejected.
These checks constrain declared artifact module edges,
not ambient authority after consent:
trusted code can still use Node built-ins such as filesystem,
module,
or process APIs to access live state.

### Exact comparison and execution

MJS candidate capture opens the canonical config with no-follow semantics before resolving filesystem identity.
On Linux,
identity resolution targets `/proc/<pid>/fd/<fd>` without resolving that descriptor link first,
so mount identity comes from the opened object.
On hosts without a process-addressable descriptor path,
identity resolution remains path-based but is bracketed by same-handle metadata before and after reading,
final live-path device and inode agreement,
and exact byte length.
Degraded filesystem identity remains explicit in the trust disclosure rather than writing logger text into JSONL event streams.

Strict MJS trust compares live entry bytes to the stored executable snapshot.
Strict TypeScript trust compares every tracked live source byte sequence to its stored source snapshot and does not
rebuild during ordinary Git commands.
Comparison streams bytes and does not compute a digest.
After source equality,
execution opens the already validated stored executable snapshot.
The live entry path is never imported.
This closes entry-file compare-then-swap only.
Trusted code retains ambient Node authority and can deliberately read or dynamically load other live files;
self-containment validation does not sandbox or intercept that behavior.

For TypeScript,
explicit trust always builds a candidate bundle before consent.
Only explicit trust or a relaxed metadata-triggered rebuild compares candidate bundle bytes to the stored bundle for
disclosure and replacement.
Ordinary strict execution never rebuilds or claims to derive expected bundle bytes.
Persistent replacement occurs only after both consent stages and config validation succeed.

## Trust consent protocol

Root preflight safely reads bytes and metadata but does not execute config.
It emits a human-readable disclosure to stderr containing every item required by the decision.
Interactive approval accepts only an explicit affirmative response.
EOF,
empty input,
invalid input,
or noninteractive input without `--yes` declines and leaves no record.

Root approval authorizes execution of the candidate stored artifact in temporary registry state.
Validation failure deletes temporary state.
If config does not declare child authority,
validated root state is installed atomically.

If config declares `trust.children: true`,
a second disclosure names the canonical repository root and states that current and future descendant mounts inherit
authority.
Declining the second stage installs ordinary root trust with `recursiveChildren: false`.
Accepting installs it with `recursiveChildren: true`.
`--yes` prints and accepts every applicable disclosure.

## Recursive enrollment and revocation

A recursive root authorizes config paths whose canonical repository roots are strict descendants of the authorized
canonical root.
The path test is component-aware,
not string-prefix matching.
Filesystem identity does not restrict inheritance:
recursive authority intentionally covers same-filesystem and mounted-volume descendants.
Before authorizing a new enrollment,
every covering recursive root must retain its exact trusted identity and bytes.
A missing,
changed,
or mount-replaced root cannot authorize new records.

First descendant encounter builds or validates config in private state,
stores exact snapshots,
and records every currently authorizing recursive root.
It does not prompt.
An auto-enrolled descendant may itself retain `recursiveChildren: true` without another prompt because its authority stays
inside an already consented outer subtree;
removing that inherited outer authority cascades through the nested root.
A mount replacement at the same canonical path has a different filesystem identity and cannot reuse the previous
record.
If an unchanged outer root still authorizes the path,
the replacement receives a new exact auto-enrollment.
Later byte changes fail closed and require explicit trust.

Explicitly trusting a descendant adds an independent self-authorizer that survives outer-root removal.
Record updates are provenance transactions across all affected record directories.

For `untrust`:

1. Resolve the exact current identity without executing config.
2. Compute every inherited descendant record affected by that identity.
3. If the target is a recursive root nested beneath authorizing recursive roots,
   include those outer roots and their inherited descendants.
4. Print the complete affected recursive-root list.
5. Acquire deterministic registry locks in encoded identity byte order.
6. Write the complete new provenance state privately.
7. Atomically install every new record or removal journal.
8. Fsync changed record parents and the private journal directory before transaction completion.
9. Recover interrupted transactions before any later trust operation.

Journals use no-follow private-file validation and reject symbolic links,
non-files,
unsafe ownership,
unsafe modes,
and unsafe Windows ACLs.
When the target config was deleted,
`untrust` resolves the canonical repository root and revokes its sole matching stored record without executing code.

A descendant with an independent explicit authorizer remains installed after inherited authorizers are removed.

## Relaxed-mode parser

The raw value is a comma-separated sequence.
A percent escape is exactly `%25` for percent or `%2C` for comma,
accepted case-insensitively on decode and emitted uppercase on encode.
No other percent escape is valid.
Decode each entry,
then split on the first colon.
Both fields must be non-empty.
Filesystem ID must satisfy the fs-id module's colon-free validated output grammar.
Canonical path must equal platform canonicalization of the config under consideration.

Malformed entries emit one prominent warning per raw entry and are ignored.
A well-formed entry naming the current path with a nonmatching filesystem ID emits a suspicious-entry warning and is
ignored.
Well-formed entries for other paths remain quiet even when their volume is absent.
No entry waives first trust.

For a previously trusted MJS identity with an exact matching entry,
compare stored and live size plus modification time.
A metadata change copies live bytes into a private candidate snapshot,
reruns self-containment checks,
executes and validates that private stored candidate,
then atomically installs it and uses the validated config value for the invocation.
Failure exits `2` and retains the previous record without executing it for that invocation.
When metadata is unchanged,
relaxed mode intentionally continues executing the previous stored snapshot even if an attacker preserved metadata while
changing live bytes.

For a previously trusted TypeScript identity with an exact matching entry,
compare stored and live size plus modification time for tracked sources.
Any metadata change triggers a private rebuild.
A successful rebuild executes and validates the private stored candidate,
then atomically replaces source and executable snapshots and uses that validated config value.
Failure exits `2` and retains the previous record without executing it for that invocation.

## TypeScript build contract

Lazily import Rolldown only when a TypeScript trust build is required.
Invoke its public `rolldown()` interface with:

- Node platform;
- ESM format;
- one output chunk;
- dependencies bundled by default;
- `codeSplitting: false`;
- in-memory generation in the private build directory.

Close the disposable bundle after generation,
including failure paths,
so native workers and the async runtime cannot retain the process.
Accept exactly one JavaScript chunk and no asset,
source-map,
native binary,
or additional chunk.
Reject unresolved imports except Node built-ins.
Reject nonliteral dynamic imports whose targets cannot be bundled.

Extract the tracked relative-local source graph from Rolldown chunk module metadata and canonicalize every path.
The graph must include the entry.
Modules outside the repository root,
symlink escapes,
and missing graph entries fail trust.
Bare package modules are bundled but excluded from source invalidation and produce a trust warning.
After bundle generation completes,
re-capture the entry identity and every tracked source;
identity,
exact bytes,
size,
and mtime must still match build inputs before trust can proceed.

Relaxed-entry parser warnings use distinct stable codes for malformed entries and current-path filesystem-identity
mismatches.
A TypeScript rebuild that bundles package imports emits `typescript-package-import-not-invalidated`.
Concurrent relaxed rebuilds may serialize successfully or one may fail closed on the per-record writer lock;
a later invocation must load the complete winning record without repair.

## Transaction protocol

The production transaction supports explicit-path and `--no-only` commits,
pathspec files including stdin and NUL forms,
selected deletions and untracked files,
amend,
allow-empty,
and merge,
cherry-pick,
or revert conclusions.
It holds the real index lock,
constructs candidate facts from a private index,
validates one-target ordinary text patches,
applies them sequentially through `git apply --cached --3way`,
and restarts the whole ordered policy sequence after exact candidate changes.
Only the final unchanged pass emits findings.
Policy exceptions,
patch conflicts,
and failed Git hooks discard private state without changing real index or worktree bytes.
Packed shadow-bin fixtures prove both modes,
non-overlapping composition,
overlap blocking,
unstaged-tail preservation,
unrelated staged preservation,
and failure rollback.

Interactive and patch selection runs through native Git once against the copied private index;
include selection stages into that private index.
Policies receive the exact chosen candidate but cannot apply automatic patches:
canonical content commits the settled private index,
while a proposed patch blocks with direct-fix guidance.
Unmerged indexes block automatic correction.
The implementation uses a transaction directory outside the worktree with:

- original index snapshot;
- private commit index;
- optional post-commit index;
- patch files;
- exact candidate-state files;
- journal metadata;
- intended original and resulting commit OIDs.

Never mutate the real index or worktree while evaluating policies.
Hold the real index lock before deriving transaction state and through final installation or rollback.
Do not invoke Git with a lock path as `GIT_INDEX_FILE`.

### Index commit

For ordinary index semantics:

1. Copy the real index.
2. Apply selected patches to the copy with `git apply --cached --3way`.
3. Run real Git with the copied index.
4. On success,
   journal the landed OID and intended index bytes.
5. Atomically install the resulting index.
6. Mark the journal complete and remove private state.

### Explicit-path commit

For injected commit-only semantics:

1. Build a commit index from `HEAD`.
2. Add exact selected worktree paths to that index using Git pathspec semantics.
3. Apply policy patches to the commit index.
4. Remove pathspecs and internal `--only` before invoking real Git because the private index is the complete intended
   tree.
5. Build a post-commit index from the original index plus selected paths from the landed commit.
6. Journal the landed OID and post-commit index bytes.
7. Atomically install the post-commit index.
8. Complete and remove the journal.

Merge,
cherry-pick,
and revert conclusions use index-commit semantics only.
Unsupported interactive/include modes use read-only checks and direct-fix guidance when needed.

### Recovery

At wrapper startup,
before config loading or forwarding,
recover any journal for the exact repository and index path.
Recovery validates original OID,
landed OID,
current ref,
original index bytes,
and intended index bytes without hashes.
It either installs the intended post-commit index,
recognizes an already completed install,
or blocks with a precise manual-recovery diagnostic.
It never silently guesses after unrelated ref or index changes.
A prepared journal records expected parent OIDs,
intended tree,
exact original and prepared index snapshots,
a private nonce-bearing `GIT_REFLOG_ACTION`,
transaction-directory,
original-index,
post-index,
and real-index-lock device/inode identities,
and owner PID plus process-birth identity before real Git can advance the ref.
When interruption happens before the exact landed-OID marker,
recovery requires current OID and the nonce-bearing action in the latest `HEAD` reflog entry;
missing or later reflog movement fails closed.
Recovery runs before trusted repository config,
refuses active owners while treating PID reuse as stale ownership,
stabilizes exact prepared artifacts through verified same-filesystem hard links,
installs only through an owner-preserving hard link rather than the mutable lock pathname,
refuses unsafe or replaced filesystem artifacts,
and preserves conflicting evidence after unrelated ref or index movement.
Concurrent wrapper processes serialize on the real index lock and transaction journal lock.

### Required disposable fixtures

- ordinary staged commit;
- explicit-path commit;
- explicit `--no-only`;
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
- interruption before real Git;
- interruption after ref update and before index install;
- interruption after index install and before journal completion;
- concurrent wrapper attempts;
- read-only administrative filesystem failure and healthy next invocation;
- hk duplicate-separator regression bytes.

Each fixture asserts exact ref,
index,
and worktree bytes before and after.
State-mutating verification uses disposable repositories only.

## Policy-specific parity

### Built-ins

The configurable core policies preserve every accepted and rejected command fixture through the unified policy engine.
Their fixed order is `require-root`,
`linked-worktree-only`,
`branch-worktree-only`,
`add-explicit`,
then `final-newline`.
All default to `error`;
`branch-worktree-only` and `final-newline` are warn-safe.
Unsafe warn configuration emits a `configuration-warning` JSONL event but still forwards when only warning findings
remain.
Generic `--no-enforce-<policy-id>` escapes and the legacy safeguard aliases skip the complete policy lifecycle and are
stripped before real Git.
Escape-looking option values and pathspecs remain ordinary Git arguments.
The legacy fixed-rule implementations no longer participate in dispatch.

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
Do not infer destination state from cached tracking refs,
push output,
or hand-written refspec interpretation.
Scan each newly reachable commit's own delta against its parents:
every parent for merges,
the whole tree only for parentless commits,
and deletions publish no content and are skipped.
Directly pushed annotated-tag,
tree,
and blob targets scan their complete content.
Deduplicate exact candidate identities across updates.
Load unique Git blobs through one `git cat-file --batch` process per evaluation.
Materialize scanner files with no more than 64 concurrent lanes.
Wrapper-added latency for every forwarded real Git command must remain strictly less than `2,000 ms`.
Explicit dry runs bypass manual-push policies.
Pure ref deletion is clean.
Indeterminate required content exits `2` with `content-unavailable`.

Default scanner resolution uses `forbidden-strings` from `PATH`.
An explicit policy option may choose another executable.
Always invoke the scanner with an argument array and no shell.
Before explicit temporary-file scanning,
apply the scanner's path-anchored `--all` exclusions for its configured rules file and canonical self-match sources.
Do not exclude unrelated nested files sharing those basenames.
Exit `1` is parsed as redacted findings;
the scanner must not expose matched bytes.
Missing executable,
unexpected non-finding status,
process interruption,
malformed output,
and scanner-owned materialized-file read failure throw from the plugin.
A thrown plugin callback emits `plugin-threw` and exits `2`;
invalid completed plugin output emits `policy-incomplete`.
The policy defaults to error and is warn-unsafe.
Preserve the independent SLSA-attested CI invocation.

### Final newline

`final-newline` is a core policy enabled at error severity by default.
Selected non-empty text ends with exactly one LF.
Remove every terminal LF before adding one.
Empty and binary-looking bytes stay unchanged.
A file with CRLF content receives one terminal LF without normalizing interior line endings.

Preserve these exclusion families exactly:

```text
package/fuzz/forbidden-strings/seeds/**
package/rust-module/forbidden-regex.fuzz/seeds/**
package/test-fixture/toml-edit/src/**
**/dist/final/node/**
**/bundle/node/**
```

Commit patches affect only would-be-committed bytes.
Direct fix affects selected worktree bytes only.
Every real index entry remains exact during direct fix.

## Benchmark method

Issue #356 records measured budgets;
this specification does not invent numeric thresholds.

The benchmark harness uses built production artifacts and disposable repositories.
Each scenario has a paired direct-real-Git baseline using the same executable,
repository,
command,
filesystem cache state,
and stdio sink.
Network operations use a local bare remote.
Scanner and plugin fixtures are deterministic and local.

Measure these scenarios separately:

- no config forwarded command;
- known read-only command;
- strict trusted MJS;
- strict trusted cached TypeScript;
- relaxed TypeScript rebuild;
- built-in validator;
- external scanner;
- normalizer clean path;
- normalizer changed path;
- post-commit policy and local auto-push.

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

- package README exists and matches this interface;
- lint has zero warnings;
- type checks pass;
- every exported code path has a test;
- built artifacts contain the Node shebang;
- `npm pack` contains only intended runtime,
  declarations,
  README,
  licenses,
  and package metadata;
- a disposable non-workspace project installs the tarball;
- that project imports every authoring helper without CLI side effects;
- its PATH resolves the packaged `git` first;
- wrapper and all management commands run through the built shim;
- MJS and TypeScript trust execute stored artifacts;
- direct fix proves index preservation;
- Linux,
  macOS,
  and Windows trust adapters have real-host evidence.

No step uploads to npm.

## Contract fixture verification

Issue #341 verifies the declarations and authoring examples in a disposable TypeScript consumer.
It also parses every management command form with an Optique fixture and rejects the mutually exclusive or missing
scope forms.
Runtime slices must replace those contract fixtures with built-package user-boundary tests rather than relying on the
document-only evidence.
