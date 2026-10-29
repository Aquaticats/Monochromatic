# cli-git 0.0.1 on GitHub Actions ubuntu-24.04: `/home` rewriting hides repository config

## Symptom

The independent final-newline workflow reached its trust step and emitted:

```json
{"schemaVersion":1,"sequence":0,"type":"engine-failure","code":"trust-failed","message":"No repository configuration was found at the canonical repository root."}
```

The failure reproduced in hosted runs `29165776240`,
`29165832538`,
and `29165889435`.
It persisted when the workflow:

- ran from the Actions checkout;
- passed the checkout through Git global `-C`;
- cloned the checkout into a disposable repository;
- ran with the disposable repository as process cwd.

Real Git still recognized the disposable repository and its `cli-git.config.mjs`.
Run `29165988807` isolated the mismatch:

```text
git rev-parse --show-toplevel: /home/runner/work/_temp/final-newline
findGitRepoRoot():             /var/home/runner/work/_temp/final-newline
```

The runner image was `ubuntu-24.04` version `20260705.232.1`.

## Root cause

This was repository code behavior,
not a GitHub Actions checkout or Git failure.

The pre-fix implementation in commit `ac70fa47c^`,
`package/fs-path/core/src/root-discovery.ts:578` to `588`,
rewrote every root beginning with `/home/`:

```ts
export function normalizeHomeRoot(root: string,): string {
  if (root === HOME_ROOT)
    return VAR_HOME_ROOT;
  if (root.startsWith(HOME_PREFIX,)) {
    return root.replace(
      HOME_PREFIX,
      VAR_HOME_PREFIX,
    );
  }
  return root;
}
```

`findRootByWalkingUp` applied that transform after successfully matching a marker,
regardless of whether `/home` was an ostree alias for `/var/home`.
On the Ubuntu runner,
this changed an existing repository root into a different path identity.

`package/git-policy/cli/src/trust/config-discovery.ts:142` then accepted that transformed root.
`package/git-policy/cli/src/trust/config-discovery.ts:149` joined `cli-git.config.mjs` to it.
The resulting nonexistent candidate was treated as config absence,
which made trust return `trust-failed`.

The fix in `package/fs-path/core/src/root-discovery.ts:635` to `642` returns the path that actually matched:

```ts
const root = await walkUpRoot({
  dir: startDir,
  fs,
  matches,
  missingMessage,
},);
rootDiscoveryLogger.debug(`resolved root discovery result ${root}`,);
return root;
```

Config discovery retains its own realpath boundary where canonical filesystem identity is required.
Root walking no longer fabricates a platform-specific spelling.

The earlier hypothesis that GitHub Actions failed to preserve `.git` or step cwd was wrong.
Run `29165988807` proved `.git`,
`cli-git.config.mjs`,
and `git rev-parse --show-toplevel` were all valid before the package root finder changed the path.

## Verification

### Version and harness

- cli-git package version:
   `0.0.1`.
- Failing runner image:
   `ubuntu-24.04` version `20260705.232.1`.
- Regression test:
   `package/fs-path/core/src/find-monorepo-root.unit.test.ts`.
- Hosted harness:
   `.github/workflows/final-newline.yml`.

Run local package verification with:

```sh
mise run //package/fs-path/core:buildAndTest
mise run //package/fs-path/core:lint:types
mise run //package/fs-path/core:format:oxlint
```

The regression fixture starts under logical `$HOME` when it uses `/home/...`.
It proves `findGitRepoRoot` returns the exact logical fixture root instead of inventing `/var/home/...`.

### Working catalog

- A Git root under `/tmp` keeps its matching path identity.
- A Git root reached through this Fedora ostree host's logical `/home/user` spelling keeps that spelling.
- Mise,
  Git,
  and pnpm root discovery agree for the current checkout.
- The final-newline workflow uses a disposable clone and a minimal `cli-git.config.mjs`.

### Failing catalog before the fix

- Checkout root plus default cwd:
   run `29165776240`.
- Checkout root plus Git `-C`:
   run `29165832538`.
- Disposable clone plus Git `-C`:
   run `29165889435`.
- Disposable clone plus process cwd:
   run `29165942247`.
- Instrumented disposable clone:
   run `29165988807` exposed `/home` becoming `/var/home`.

## Verified workarounds

### Preserve runtime-native root identity

Commit `ac70fa47c` removes unconditional home-root rewriting.
This is the selected fix because the root walker now returns only a path whose marker it actually inspected.

Tradeoff:
callers that specifically want canonical filesystem identity must resolve it at their own boundary.
cli-git already does so in config discovery and trust identity capture.

### Use a disposable minimal-config clone in CI

Removed on 2026-10-14 at the owner's request, together with `package/git-policy/cli/bin/final-newline-workflow.mjs`:
the check rebuilt the wrapper on every pull request.
The description that follows is the historical design.

`.github/workflows/final-newline.yml` clones the checked-out revision under runner temporary storage,
writes `export default {};` as `cli-git.config.mjs`,
trusts it,
and runs only `final-newline` through direct check.

Tradeoff:
the workflow copies the tracked worktree before scanning.
It intentionally excludes checkout-local dependency artifacts and unrelated root policy configuration.

## What does not work

- Passing `-C "$GITHUB_WORKSPACE"` did not help because root discovery rewrote the successfully parsed path later.
- Creating a disposable clone did not help while the same unconditional rewrite remained.
- Setting the process cwd to the clone did not help because the root finder changed the matched cwd after walking.
- Running `realpath` in the shell did not match the package's former hard-coded transform on this runner.
- Treating the failure as missing config hid the actual marker-path mismatch and provided no useful diagnostic.

## Upstream filing decision

### Constraint check

1. **Is it really upstream's fault?
   ** No.
   Git and GitHub Actions preserved a valid repository and config;
   this repository changed `/home` to `/var/home` unconditionally.
2. **Can upstream fix it?
   ** Not at the relevant boundary.
   GitHub cannot correct a path rewrite implemented by `module-fs-path`.
3. **Are they supporting this use case?
   ** GitHub Actions supports checkout,
   process cwd,
   and temporary repositories,
   all of which worked under real Git.
4. **Would the repo welcome our contribution?
   ** Not evaluated as an upstream contribution because constraint one fails.
5. **Will they likely fix it?
   ** Not applicable because there is no upstream defect to fix.
6. **Have we prototyped a minimal fix compatible with their architecture?
   ** Yes at the consumer boundary.
   Commit `ac70fa47c` removes the unsupported transform and adds a regression fixture.

No matching `.out-of-scope/` entry names GitHub runner path identity or `module-fs-path`.
No upstream duplicate search was performed because the evidence eliminated upstream behavior as the defect,
and no upstream filing is proposed.

### Filing artifact

~~~md
Do not file as-is.

There is no GitHub Actions issue to report.
The failure came from Monochromatic's unconditional `/home` to `/var/home` rewrite and was fixed in `module-fs-path`.
~~~
