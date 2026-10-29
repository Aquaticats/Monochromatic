# Consolidate real-Git executable resolution for Pi auto-mode

Status:
 implemented.
The user confirmed common-platform-path priority and then authorized implementation.

## Outcome

Implementation landed in these commits:

- `a671af16e` added `@monochromatic-dev/git-executable-core`;
- `a8fd5b68e` hardened resolver diagnostics and platform-path tests;
- `2aef2d409` migrated git-policy-cli and deleted its resolver copy;
- `612ae1853` migrated auto-mode and deleted its resolver copy;
- `842057939` registered workspace dependency metadata;
- `219d7f94b` rejected non-regular candidates and bounded script inspection;
- `e142bca71` rejected oversized scripts and bounded cache lifetime;
- `d13354d43` proved in-flight deduplication and least-recently-used eviction through built artifact;
- `627636830` completed package license texts required by `LGPL-3.0-or-later`.

The shared resolver now owns common-platform-path priority,
Windows `PATHEXT`,
native executable recognition,
self-shim exclusion,
and a 16-entry least-recently-used successful-resolution cache.
Auto-mode continues to query mutable worktree metadata for every read event.

The post-change verbose auto-mode unit harness recorded one `/usr/bin/git` resolution across seven linked-worktree
metadata collections and no `git not executable at` records.
A Pi SDK host verification loaded the built auto-mode extension,
executed two real built-in `read` tool calls,
and recorded one resolver success,
two fresh metadata collections,
and no missing-candidate records.
The resolver records are positive controls proving that verbose resolver diagnostics were captured.

## Problem

Pi intentionally inherits every workspace package bin directory on `PATH`.
Auto-mode currently appends `git` to every entry,
checks every candidate concurrently,
and repeats that resolution for every `read` event.
The resulting `ENOENT` records are expected candidate misses,
not failed Git commands.

The diagnosis and reproduction are in
[`doc/troubleshooting/pi-auto-mode-repeated-git-path-probes.md`][diagnosis].

Two owned modules currently implement real-Git resolution:

- `package/git-policy/cli/src/resolve-git.ts` has platform-aware executable naming,
  native executable detection,
  complete self-shim markers,
  common platform path priority,
  and sequential lookup.
- `package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts` duplicates part of that behavior,
  omits Windows `PATHEXT`,
  starts every candidate check with `Promise.all()`,
  and logs every expected miss.

The duplicate implementations have already diverged.
The resolver belongs behind one owned module interface.

## Recommendation

Create `package/git-executable/core` as `@monochromatic-dev/git-executable-core`.
Move real-Git executable selection from git-policy-cli into that package,
then make git-policy-cli and auto-mode depend on it through `/ts`.

Keep linked-worktree trust policy in auto-mode.
Executable selection and worktree authorization have different owners and lifetimes:

- executable selection is platform and process environment infrastructure;
- linked-worktree authorization is auto-mode security policy;
- worktree metadata can change while Pi is running and must remain fresh.

This split creates a deep executable-resolution module with a small interface,
while preserving locality for guardrail policy.

## Proposed interface

The external seam should expose one operation:

```ts
// package/git-executable/core/src/index.ts
export async function resolveRealGit(
  options?: ResolveRealGitOptions,
): Promise<string>;
```

`ResolveRealGitOptions` should retain only inputs that affect executable lookup and support disposable tests:

- `pathEnv`;
- `platform`;
- `pathExtensions`;
- `cwd`,
  for empty or relative `PATH` entries;
- preferred platform paths,
  when a test needs disposable common-path fixtures.

The interface invariants should state:

- preferred common platform paths win only when they are exposed by `PATH`;
- remaining candidates follow `PATH` and Windows `PATHEXT` order;
- shims that delegate to `@monochromatic-dev/git-policy-cli` are never returned;
- native ELF,
  PE,
  Mach-O,
  and universal Mach-O executables are accepted without complete text scanning;
- no usable candidate throws a dedicated resolution error;
- successful concurrent calls with identical effective candidate sequences share one lookup;
- rejected lookups are not cached.

The effective candidate sequence,
after relative entries are resolved against `cwd`,
should be the memoization identity.
That avoids stale reuse when `process.cwd()` changes while `PATH` contains empty or relative entries.

Do not expose a selection-mode option.
The user explicitly confirmed that git-policy-cli's tested common-platform-path priority is the required canonical
behavior.
Auto-mode must adopt it when `PATH` contains an earlier non-common real Git and a later common real Git.
Guard the invariant with a two-real-Git test.

## Logging

Expected candidate absence is control flow,
not a diagnostic event.
Do not emit one debug record for every `ENOENT` or `ENOTDIR` candidate.

Emit one debug record for a successful resolution that names:

- selected executable;
- selection policy;
- whether self-shims were skipped.

Log unexpected access or inspection failures with the candidate path and caught value.
A final no-candidate error should summarize the attempted candidate classes without printing a record per ordinary
miss.

## Auto-mode integration

Remove these auto-mode-owned details:

- `CLI_GIT_PACKAGE_NAME`;
- bundled entry markers;
- `isCliGitShimForSelf()`;
- `resolveGitCandidate()`;
- `resolveRealGit()`.

`linkedWorktreeReadAllowlistedDirs()` should call the shared resolver.
Successful resolution should be reused for equal effective inputs,
so every `read` no longer rescans the broad `PATH`.

Keep these operations fresh for now:

- `git worktree list --porcelain`;
- per-worktree `git rev-parse` classification;
- secret-path and write guards.

Do not cache linked-worktree roots as part of this change.
A worktree can be added or removed during a Pi session,
and stale authorization data is a security regression.

After the resolver change lands,
measure the remaining metadata subprocess cost.
If evidence warrants another change,
make linked-worktree classification demand-driven only for reads that ordinary cwd,
skill,
and scratch checks would otherwise reject.
That later change needs structured path-signal evidence so secret and home-dotfile checks cannot be bypassed.

## Git-policy-cli integration

Move the implementation and resolver tests,
not merely a wrapper,
to `@monochromatic-dev/git-executable-core`.
Git-policy-cli should import the shared module and retain no candidate-selection markers of its own.

This preserves git-policy-cli's current common-path priority while removing implementation ownership from a CLI
package.
It also prevents auto-mode from depending on git-policy-cli's policy engine,
published package root,
or authoring interface.

Enumerate every current `./resolve-git.ts` import before moving the file.
Update production and test imports together so no internal copy remains.

## Verification

### Shared module tests

Use disposable executable fixtures to cover every exported branch:

- broad `PATH` with many absent package-bin candidates;
- common platform Git later in `PATH` wins over an earlier non-common real Git;
- preferred candidate absent falls back to first usable remaining candidate;
- every Unix and Windows self-shim marker is skipped;
- native ELF,
  PE,
  Mach-O,
  and universal Mach-O headers are accepted;
- Windows names follow `PATHEXT` order and path identity is case-insensitive;
- empty and relative `PATH` entries use injected `cwd`;
- concurrent equal calls perform one lookup;
- changed effective inputs perform another lookup;
- rejected lookup is retried rather than cached;
- no candidate throws the dedicated error.

### Consumer tests

Git-policy-cli tests must prove ordinary forwarding still selects the shared resolver result.

Auto-mode tests must prove:

- repeated `read` events reuse successful executable resolution;
- no expected missing-candidate record is emitted;
- linked worktree metadata is still queried fresh;
- linked reads remain accepted;
- main-worktree roots are not added as cross-worktree trust;
- writes and secret-looking reads remain guarded.

### User-boundary verification

Build through package mise tasks,
start a fresh Pi process,
and exercise:

- an ordinary read inside current worktree;
- a read in an attached linked worktree;
- a write in that linked worktree;
- a secret-looking read in that linked worktree;
- creation of another linked worktree followed by a read during the same Pi session.

Confirm the selected real Git path is logged once per effective environment,
expected package-bin misses are absent,
and authorization behavior remains unchanged.

## Options considered

### Shared executable-resolution package

#### Pros

- one implementation owns platform lookup and self-shim knowledge;
- auto-mode does not depend on a policy CLI package;
- both current callers justify a real seam;
- tests exercise the same interface as production callers;
- future resolver fixes have locality.

#### Cons

- adds a package that must satisfy package completeness rules;
- moves imports across git-policy-cli;
- auto-mode must migrate to the user-confirmed common-path priority.

### Export a git-policy-cli resolver subpath

#### Pros

- reuses the current implementation with fewer moved files;
- requires less package setup;
- removes auto-mode's duplicate.

#### Cons

- makes auto-mode depend on a CLI and its dependency graph;
- widens git-policy-cli's published interface with infrastructure unrelated to policy authoring;
- leaves executable selection owned at the wrong seam;
- still requires auto-mode to adopt the confirmed common-path priority.

### Repair and cache auto-mode's local resolver

#### Pros

- preserves auto-mode's current first-real-Git `PATH` order;
- has narrower immediate file churn;
- removes repeated auto-mode scans.

#### Cons

- keeps duplicate platform and self-shim logic;
- leaves Windows behavior behind git-policy-cli;
- future fixes can diverge again;
- fails the deletion test because resolver complexity reappears in two callers.

### Suppress candidate logs only

#### Pros

- changes the fewest lines;
- removes the visible symptom.

#### Cons

- retains every filesystem probe;
- retains duplicate resolver logic;
- retains repeated work on every read;
- hides the design defect instead of correcting it.

## Ranking

Ranking:
 shared package > git-policy-cli subpath > local auto-mode repair > log suppression.

The shared package beats the subpath because it places the seam at executable infrastructure rather than a policy CLI.
The subpath beats local repair because one owner is more valuable than preserving duplicated implementation.
Local repair beats log suppression because it removes repeated lookup work instead of hiding its records.

[diagnosis]: ../troubleshooting/pi-auto-mode-repeated-git-path-probes.md
