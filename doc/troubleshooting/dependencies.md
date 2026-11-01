# Package management aggregator: pnpm 9.x blocklist mechanism, 24 transitive bans, vlt 0.x semver-build-metadata manifest fetch failure, per-package override audits

The vlt entry below is the only entry in this file that follows the
troubleshooting-doc shape (it is a real external-tool bug).
 The
remaining sections are workspace audit trails for dependency
substitution decisions;
 they document the "why we substituted /
removed / kept" rationale and explicitly do not file upstream because
the upstream packages are not defective,
 only unsuitable for this
workspace's policy.
 See "Why we do not file the policy entries
upstream" at the bottom of this file for the 5-constraint walk.

## Global blocklist (substitution vs removal)

For "this package is banned globally;
 warn and substitute or remove during install,
"
see `doc/dependency-blocklist.md`.
That doc covers the two-home mechanism:
`.pnpmfile.mjs` at the repo root for substitution stubs (`action: 'throw'` or `action: 'silent'`),
`pnpm-workspace.yaml`'s `overrides` block for removal (pnpm's native `"name": "-"` primitive).

The parent-scoped overrides documented in this file
(`jspdf>canvg`,
 `@earendil-works/pi-ai>@google/genai`,
 the rest of the `pi-*` audit below)
are surgical removals for specific dependents.
They sit alongside the global mechanisms,
 not inside them,
 and stay in `pnpm-workspace.yaml`.

## Pre-emptive bans on poorly-maintained transitive utilities

`.pnpmfile.mjs` declares 24 transitive utilities as blocked with `action: 'throw'`
as of 2026-05-12.
None were in the resolved graph at the time the entries were added
(verified:
 zero matches in `pnpm-lock.yaml`).
One has since entered the graph:
 `convert-source-map`,
 pulled transitively by
StrykerJS via `@babel/core`.
 See its deep-dive under "Package Management Warnings"
below for why the throwing stub is never loaded at runtime.
A second throwing-stub substitution,
 `js-yaml`,
 was added later as a deliberate
in-graph substitution rather than a forward-looking canary:
 it was already in the
graph via `stylelint > cosmiconfig`
 (stylelint removed 2026-10-04,
 see `doc/decision/remove-stylelint.md`),
 and the policy strips the real package.
 It is
not one of the 24 pre-emptive bans;
 see its own deep-dive below.
The policy is a forward-looking canary:
if a future manifest declares one of these as a dependency,
`pnpm install` emits a stderr `[blocked-dep]` line naming the consumer,
and any runtime `require` on the package throws an error referencing
`doc/dependency-blocklist.md`.

The decision criterion is the same across all 24:
each package is either abandoned (no commits in 3+ years on upstream),
a polyfill rendered obsolete by every supported runtime (Node 22+,
 Bun),
or a piece of the express 4.
x micro-utility family that the workspace has chosen
not to take a dependency on (h3 replaces express;
 elysia was audited but
not adopted,
 see `AUDIT.md`).
Per-package rationale is the `reason` field in `.pnpmfile.mjs`,
which surfaces verbatim in the install-time warning.

The 24 split into four groups:

- Express 4.
  x family (11):
  `cookie-signature`,
   `destroy`,
   `etag`,
   `forwarded`,
   `fresh`,
   `methods`,
  `proxy-addr`,
   `statuses`,
   `toidentifier`,
   `utils-merge`,
   `vary`.
- Polyfills obsolete on every supported runtime (3):
  `fs.realpath`,
   `regenerator-runtime`,
   `unpipe`.
- Abandoned utilities with trivial native or catalog replacements (6):
  `extglob`,
   `for-in`,
   `repeat-element`,
   `repeat-string`,
   `sax`,
   `set-blocking`.
- Source-map / browser-data / JSON utilities replaced by bundler built-ins
  or catalog deps (4):
  `caniuse-lite` (blocked except `browserslist` itself,
   which resolves
  `.browserslistrc` for file-enforcer and config-tsdown),
  `convert-source-map` (now in the graph via StrykerJS;
   see its deep-dive below),
  `fast-json-stable-stringify`,
   `source-map-resolve`.

### Conditions for revisiting

Remove or relax an entry when:

- A workspace package gains a legitimate need for one of the listed packages
  (e.g. a deliberate adoption of express,
   an SDK that pins old ajv).
- A previously-abandoned package returns to active maintenance and the
  rationale recorded here no longer applies.

In both cases,
 update the entry's `reason` (or delete it) and record the
change here so future readers see the audit trail.

### Verification

After editing the POLICY block,
 run `mise run prepare:pnpm:install`.
pnpm rewrites the `pnpmfileChecksum` line in `pnpm-lock.yaml`
and re-runs resolution.
Allowed exceptions do not print warnings;
 currently `browserslist` is allowed
for `caniuse-lite` so file-enforcer and config-tsdown can resolve real browser data.
For every non-allowed dependent,
 no substitutions should occur and no `[blocked-dep]`
lines should print.
 The checksum line in `pnpm-lock.yaml` is committed alongside
the `.pnpmfile.mjs` change.

## vlt 0.x fails to fetch manifest when dependency spec includes semver build metadata `+<hash>` suffix

### Symptom

`vlt install` fails with `Error: failed to fetch manifest` on a dependency
whose version spec includes semver build metadata (the `+<hash>` suffix):

```text
Error: failed to fetch manifest
  [cause]: {
    code: 'ERESOLVE',
    spec: Spec2 { spec: '@optique/core@1.0.0-dev.1692+5c265bd4', … },
    url: https://registry.npmjs.org/@optique/core/1.0.0-dev.1692+5c265bd4,
  }
```

Restoring the lockfile does not help.
Graph modifiers (`"modifiers"` in `vlt.json`) do not help either;
the failing code path runs during node extraction,
 which bypasses modifier logic.

### Root cause

Two independent issues combine to produce the failure:

**1.
 Upstream publisher ships build metadata in dependency specs.
**

`@optique/run@1.0.0-dev.1692` declares `"@optique/core": "1.0.0-dev.1692+5c265bd4"`.
Both tarballs also contain `"version": "1.0.0-dev.1692+5c265bd4"` in their `package.json`.
The npm registry strips `+<build>` from the **version key** at publish time
(`npm publish` has done this since 2014,
 per npm/npm#6379),
but dependency **specifier strings** pass through unmodified.

This is extremely rare:
 npm stripping `+` from versions means the public registry
has zero packages with build metadata in version keys.
 The `@optique` project
is an outlier because its toolchain writes `+<git-sha>` into both the version field
and cross-package dependency pinning before publish.

**2.
 vlt passes build metadata verbatim into registry manifest URLs.
**

The npm registry returns 404 for version URLs containing `+`:

- `GET /@optique/core/1.0.0-dev.1692`:
   200 OK
- `GET /@optique/core/1.0.0-dev.1692+5c265bd4`:
   404
- `GET /@optique/core/1.0.0-dev.1692%2B5c265bd4`:
   404

vlt hits this because it never strips build metadata from the version string
before constructing the per-version manifest URL.

### Detailed code trace

All paths in this section are relative to the **vltpkg/vltpkg** monorepo.

**Entry point:
 dependency spec parsing:
**

`src/spec/src/browser.ts:644` sets `registrySpec = bareSpec` verbatim,
preserving the `+5c265bd4` suffix from the dependency declaration.

**Trigger:
 single-version fast path:
**

`src/package-info/src/index.ts:714-716` (rc.
29;
 was `:601-603` at rc.
24):

```ts
const mani = spec.range?.isSingle
  ? await this.#registryManifestRequest(spec, options,)
  : pickManifest(await this.packument(f, options,), spec, options,);
```

A version with build metadata parses as `isSingle === true` (the `+` part is metadata,
not a range operator),
 so vlt takes the fast path instead of fetching the full packument.

**Failure:
 URL construction:
**

`src/package-info/src/index.ts:476-478` (rc.
29;
 was `:405-407` at rc.
24):

```ts
const version = hasLeadingRange ? registrySpec.slice(1,) : registrySpec;
const pakuURL = new URL(`${name}/${version}`, registry,);
```

`version` becomes `"1.0.0-dev.1692+5c265bd4"`.
In URL semantics,
 `+` is not a path-safe character;
`new URL()` preserves it literally rather than encoding it,
and the npm registry does not recognize the resulting path.

**Second trigger:
 extraction bypass of modifiers:
**

`src/graph/src/reify/extract-node.ts:57` hydrates the spec from the node's DepID:

```ts
const spec = hydrate(node.id, node.name, options,);
```

The DepID is built from `mani.version` (`src/dep-id/src/browser.ts:537-542`,
verified intact at rc.
29).
When the manifest comes from the npm registry packument,
 `mani.version` is
`"1.0.0-dev.1692"` (no build metadata),
 so the hydrated spec is clean.
But when the manifest comes from a dependency declaration that includes build metadata
(e.g. `@optique/run`'s dep on `@optique/core@1.0.0-dev.1692+5c265bd4`),
the spec flows through `fetchManifestsForDeps`
(`Spec.parse` call at `src/graph/src/ideal/append-nodes.ts:590`)
where `Spec.parse(name, bareSpec, options)` preserves the `+` suffix.
Graph modifiers apply at `src/graph/src/ideal/append-nodes.ts:269-288`
of the same file but are scoped
to the graph-building phase;
 the extraction code path at `extract-node.ts`
creates specs independently and does not consult modifiers.

### Why this was never reported before

npm has stripped `+<build>` from version strings at publish time since 2014.
Across thousands of checked packages (semver,
 typescript,
 react,
 electron,
 webpack,
lodash,
 express,
 next,
 vue,
 angular,
 prettier,
 rollup,
 esbuild,
 etc.),
zero have build metadata in any version key or dependency specifier on the public registry.
The vlt issue tracker (vltpkg/vltpkg) has no reports matching this pattern;
existing "failed to fetch manifest" issues (#1534,
 #1263,
 #260) all have different causes.

The trigger requires a package that pins a dependency to an **exact version
with build metadata** (not a range).
 This is functionally unreachable
through normal npm publishing workflows.

### Verification

Originally reported at vlt 1.0.0-rc.
24 (line numbers above match rc.
24
and have been annotated with the current rc.
29 lines where they shifted).
Re-verified at vlt `1.0.0-rc.29-1` (commit `8ece488d`,
`v1.0.0-rc.29-1-g8ece488d`,
 fetched 2026-05-17).
 File paths in this
document are relative to that monorepo's `src/` tree.

Trigger package:
 `@optique/run@1.0.0-dev.1692` declaring
`"@optique/core": "1.0.0-dev.1692+5c265bd4"`.

Catalogues:

- **Works**:
   `npm install`,
   `pnpm install`,
   `yarn install` against the
  same trigger package.
   All three strip `+<build>` from the version
  string before constructing the registry URL.
- **Works**:
   registry URL `GET /@optique/core/1.0.0-dev.1692` (without
  build metadata) returns 200 OK.
- **Fails**:
   `vlt install` with the trigger package.
   URL
  `GET /@optique/core/1.0.0-dev.1692+5c265bd4` returns 404,
   and the
  percent-encoded variant `%2B5c265bd4` also returns 404.
- **Fails (no recovery)**:
   `vlt install` with `vlt.json` graph
  modifiers (`"#@optique/core": ">=1.0.0-dev.0"`,
  `"#@optique/run": ">=1.0.0-dev.0"`).
   Both run with
  `overridden: false` because the failure is in the extraction path
  (`src/graph/src/reify/extract-node.ts:57`),
   which is downstream of
  modifier application (`src/graph/src/ideal/append-nodes.ts:269-288`).
- **Fails (no recovery)**:
   lockfile restoration.
   vlt re-resolves
  specs during install.
- **Fails (no recovery)**:
   cache clearing
  (`rm -rf ~/.cache/vlt/{package-info,registry-client}`).
   Build
  metadata originates from the upstream manifest,
   not stale cache.

### Workaround

No reliable workaround exists within vlt's configuration:

- **Graph modifiers** (`"modifiers"` in `vlt.json`) override specs during
  ideal graph building but do not apply during node extraction.
  Tested with `"#@optique/core": ">=1.0.0-dev.0"` and
  `"#@optique/run": ">=1.0.0-dev.0"`:
   both failed,
   error shows `overridden: false`.
- **Lockfile restoration** does not help because vlt re-resolves specs during install.
- **Cache clearing** (`rm -rf ~/.cache/vlt/{package-info,registry-client}`)
  does not help because the build metadata originates from the upstream manifest,
  not from stale cache data.

Viable alternatives until vlt is patched:

- **Pin to a version of `@optique/*` that does not use build metadata**
  (if one exists)
- **Use a different package manager** (npm,
   pnpm,
   yarn) for installs;
  all three strip build metadata before making registry requests
- **Ask the `@optique` maintainer** to stop publishing dependency specs
  with build metadata suffixes

### Suggested fix

Strip build metadata from the version string in `#registryManifestRequest`
before constructing the URL:

```ts
// src/package-info/src/index.ts, inside #registryManifestRequest
const version = hasLeadingRange ? registrySpec.slice(1,) : registrySpec;
const versionClean = version.replace(/\+.*$/, '',);
const pakuURL = new URL(`${name}/${versionClean}`, registry,);
```

A more comprehensive fix would strip build metadata in the `Spec` parser
(`src/spec/src/browser.ts:644`) so all downstream consumers see clean versions.
This aligns with SemVer 2.0.0 items 10-11,
 which state build metadata
**MUST** be ignored when determining version precedence.

### Why we would file this upstream (5 constraints)

Walked against the 5-constraint upstream-filing check.

1. **Upstream's fault?
   ** Yes.
    SemVer 2.0.0 items 10-11 state build
   metadata MUST be ignored when determining version precedence;
    vlt's
   single-version fast path violates this by treating `+<build>` as
   part of the addressable version.
    The npm registry follows the spec
   by stripping `+` from version keys at publish time
   (npm/npm#6379,
    2014);
    vlt's URL construction does not.
2. **Can upstream fix it?
   ** Yes;
    one-line fix in
   `src/package-info/src/index.ts:476-478` (rc.
   29) to strip `+.*$` before URL
   construction,
    or a broader fix in `src/spec/src/browser.ts:644` so
   all downstream consumers see clean versions.
    Both are tractable
   given the cited code.
3. **Supporting this use case?
   ** vlt is positioned as a general-purpose
   npm-compatible package manager;
    SemVer-compliant version handling
   is in scope.
4. **Will they fix it?
   ** Plausible.
    vlt is actively developed;
    the
   change is small.
    The issue tracker has no existing report for this
   pattern (issues #260,
    #1263,
    #1534 are all different "failed to
   fetch manifest" causes).
    Acceptance is likely but not guaranteed.
5. **Minimal-fix prototype?
   ** Yes.
    Cloned `vltpkg/vltpkg` at commit
   `8ece488d` (tag `v1.0.0-rc.29-1`) into a fresh `mktemp -d`
   workspace,
    ran `vlt install` (via the bootstrap published rc.
   29) to
   resolve the workspace,
    then exercised the source-tree
   `scripts/bins/vlt` against a one-line `package.json` declaring
   `"@optique/run": "1.0.0-dev.1692"`.

   Pre-patch (verbatim):

   ```text
   Resolve Error: failed to fetch manifest
     While fetching: https://registry.npmjs.org/@optique/core/1.0.0-dev.1692+5c265bd4
     Response: { statusCode: 404, … }
   ```

   Applied the single-hunk patch:

   ```diff
   diff --git a/src/package-info/src/index.ts b/src/package-info/src/index.ts
   --- a/src/package-info/src/index.ts
   +++ b/src/package-info/src/index.ts
   @@ -475,7 +475,8 @@ export class PackageInfoClient {
        )
        const version =
          hasLeadingRange ? registrySpec.slice(1) : registrySpec
   -    const pakuURL = new URL(`${name}/${version}`, registry)
   +    const versionClean = version.replace(/\+.*$/, '')
   +    const pakuURL = new URL(`${name}/${versionClean}`, registry)
        const response = await this.registryClient.request(pakuURL, {
          headers: {
            accept: 'application/json',
   ```

   Post-patch `vlt install` succeeded with exit 0,
    adding both
   `@optique/run@1.0.0-dev.1692` and `@optique/core@1.0.0-dev.1692`
   (the build-metadata suffix is stripped only from the URL;
    the
   fetched tarball's `package.json` still records
   `"version": "1.0.0-dev.1692+5c265bd4"`,
    which the registry tolerates).
   The fix did not require touching `conventionalRegistryTarball`:
    the
   manifest fetch returns the registry-stripped `dist.tarball`,
    so the
   guessed-URL fallback is not exercised on the success path.

   `src/package-info` test suite under Node v26.1.0:
    141 of 148 pass
   post-patch.
    The 7 failures (`cache hit - manifest returned from
   cache` and 6 sibling subtests) are pre-existing at HEAD and
   identical pre- and post-patch;
    they fail with
   `The property 'options.recursive' is no longer supported. Received true`,
   which is the Node v26 removal of the `recursive` option from
   `fs.cp` / `fs.cpSync`,
    unrelated to manifest URL construction.
   The existing leading-prefix-strip test
   (`src/package-info/test/index.ts:661`,
    "manifest strips leading
   semver characters") exercises the same URL-construction block this
   patch extends and remains green;
    the new `+`-strip slots in
   immediately after the existing `=/^/~/v` strip.

**Decision:
 file upstream.
** All five constraints hold with verified
evidence.
 A draft bug report is kept at
`BUG-REPORT.vlt-build-metadata.md` (referenced below);
 the verified
diff and verification commands above can be carried into the upstream
issue as-is.
 Re-validate line numbers against the then-current vlt HEAD
before filing if more than a few weeks have passed.

### Draft upstream issue (do not file as-is; re-validate against current vltpkg/vltpkg HEAD before filing)

````md
**Title:** vlt fails to fetch manifest when dependency spec includes
semver build metadata `+<hash>` suffix

**Labels:** bug, package-info, spec-parsing

**Description:**

`vlt install` fails with `Error: failed to fetch manifest` when a
dependency declaration includes semver build metadata in its version
string. SemVer 2.0.0 items 10-11 state build metadata MUST be ignored
for version precedence; npm registry strips `+<build>` from version
keys at publish time (npm/npm#6379, 2014). vlt's single-version fast
path does not.

**Reproduction:**

`@optique/run@1.0.0-dev.1692` declares
`"@optique/core": "1.0.0-dev.1692+5c265bd4"`. Running `vlt install`
against this dependency:

```text
Error: failed to fetch manifest
  [cause]: {
    code: 'ERESOLVE',
    spec: Spec2 { spec: '@optique/core@1.0.0-dev.1692+5c265bd4', … },
    url: https://registry.npmjs.org/@optique/core/1.0.0-dev.1692+5c265bd4,
  }
```

The registry returns 404 for paths containing `+`. Compare:

- `GET /@optique/core/1.0.0-dev.1692`: 200 OK
- `GET /@optique/core/1.0.0-dev.1692+5c265bd4`: 404
- `GET /@optique/core/1.0.0-dev.1692%2B5c265bd4`: 404

`npm install`, `pnpm install`, `yarn install` all succeed against the
same trigger package.

**Code trace** (line numbers as of `8ece488d` / `v1.0.0-rc.29-1`):

- `src/spec/src/browser.ts:644` sets `registrySpec = bareSpec`
  verbatim, preserving `+5c265bd4`.
- `src/package-info/src/index.ts:714-716`: version with `+` parses as
  `isSingle === true`, so vlt takes the fast path.
- `src/package-info/src/index.ts:476-478`:

  ```ts
  const version = hasLeadingRange ? registrySpec.slice(1,) : registrySpec;
  const pakuURL = new URL(`${name}/${version}`, registry,);
  ```

  `version` becomes `"1.0.0-dev.1692+5c265bd4"`. `new URL()` preserves
  `+` literally; the npm registry does not recognize the resulting
  path.
- `src/graph/src/reify/extract-node.ts:57` hydrates the spec from the
  node's DepID; the extraction-time spec is built from
  `fetchManifestsForDeps`
  (`Spec.parse` call at `src/graph/src/ideal/append-nodes.ts:590`),
  which preserves the `+` suffix.

**Suggested fix:**

Strip build metadata before URL construction:

```ts
// src/package-info/src/index.ts, inside #registryManifestRequest
const version = hasLeadingRange ? registrySpec.slice(1,) : registrySpec;
const versionClean = version.replace(/\+.*$/, '',);
const pakuURL = new URL(`${name}/${versionClean}`, registry,);
```

A more comprehensive fix would strip in the `Spec` parser
(`src/spec/src/browser.ts:644`) so all downstream consumers see clean
versions.

**Tested against:** trigger package
`@optique/run@1.0.0-dev.1692` declaring
`"@optique/core": "1.0.0-dev.1692+5c265bd4"`.
````

### Status

No upstream issue filed yet as of 2026-04-04.
 Constraint-5 prototype
re-verified at vlt commit `8ece488d` (`v1.0.0-rc.29-1`) on 2026-05-17;
patch verified end-to-end on a fresh clone (pre-patch 404,
 post-patch
exit 0).
 See `BUG-REPORT.vlt-build-metadata.md` and the in-line draft
above;
 both now carry the rc.
29 line numbers.

## Package Management Warnings

### Don't run `pnpm up`

It will turn `>=` in `package.json` into exact versions.

### `node-domexception` (substituted via workspace shim)

`pnpm install` previously emitted `[WARN] 1 deprecated subdependencies found: node-domexception@1.0.0`.
Install-time only;
 runtime was unaffected.
The override `'node-domexception': 'link:package/pnpm-override/shim-node-domexception'` in `pnpm-workspace.yaml`
now substitutes the deprecated upstream package with a workspace shim whose `index.cjs` is the
single line `module.exports = globalThis.DOMException`.
 The warning is gone;
 consumers
(fetch-blob,
 node-fetch,
 the libsql and gaxios subtrees) see the native `DOMException`
constructor exactly as before.

#### Dependency chain

```text
node-domexception@1.0.0
└─┬ fetch-blob@3.2.0
  ├─┬ formdata-polyfill@4.0.10
  │ └─┬ node-fetch@3.3.2
  │   ├─┬ @libsql/hrana-client@0.6.2 → @libsql/client → @libsql/kysely-libsql
  │   │                              → @monochromatic-dev/webapp-forge-server
  │   └─┬ gaxios@7.1.4 → gcp-metadata → google-auth-library → @google/genai
  │                    → @earendil-works/pi-ai → @earendil-works/pi-coding-agent
  │                    → @monochromatic-dev/pi-{morph-compact,terminal-title}
  └── node-fetch@3.3.2 [deduped]
```

`pnpm why node-domexception` reproduces this on demand.

#### Why it cannot be removed

`fetch-blob@3.2.0/from.js:3` does `import DOMException from 'node-domexception'`
and `from.js:86` calls `throw new DOMException(...)`.

`node-fetch@3.3.2/src/index.js` and `src/utils/multipart-parser.js` both import
from `fetch-blob/from.js`,
 so the import path executes at runtime whenever
node-fetch is loaded.

A `'fetch-blob>node-domexception': '-'` override removes the package from the
install but leaves the static import in `fetch-blob/from.js`:
 node-fetch
crashes at module load.

#### Why upgrading does not help

- `node-fetch@3.3.2` is the latest published v3;
   v4 is in beta and still
  depends on `fetch-blob@^3.1.4`.
- `fetch-blob@4.0.0` (latest) still declares `node-domexception: ^1.0.0`.
- `formdata-polyfill@4.0.10` (latest) declares `fetch-blob: ^3.1.2`,
   pinning
  the v3 line.
- `@libsql/hrana-client@0.10.0` (latest) and `gaxios@7.1.4` (latest) both still
  depend on `node-fetch@^3.3.2`.
   Native `fetch` migration is upstream work that
  has not happened.

#### Why overriding to v2 does not help

`node-domexception@2.0.2` is also deprecated ("Use your platform's native
DOMException instead").
 Critically,
 v2 changed the API:

- v1 `index.js`:
   `module.exports = globalThis.DOMException` (default export is
  the constructor).
- v2 `index.js`:
   side-effect-only;
   sets `globalThis.DOMException ??= ...` and
  exports nothing.

v2 never assigns `module.exports`,
 so it stays at the CJS default `{}`.
 Under
Node ESM,
 `import DOMException from 'node-domexception'` resolves to
`module.exports`,
 i.e. `{}`.
 `throw new DOMException(...)` then throws
`TypeError: DOMException is not a constructor` at runtime.

#### Why runtime is fine

v1's `index.js` ends with `module.exports = globalThis.DOMException`.
 On Node
17+ and Bun,
 `globalThis.DOMException` is the native class,
 so the import
yields the platform constructor.
 The deprecated package is a no-op shim at
runtime;
 the deprecation message is purely an npm-registry-level annotation
read by pnpm at install time.

#### Workaround applied

`package/pnpm-override/shim-node-domexception/` is the workspace shim package.
Files:
 `package.json` (private,
 name `@monochromatic-dev/pnpm-override-shim-node-domexception`,
CJS,
 exports `./index.cjs` and `./index.d.cts`),
 `index.cjs`
(`'use strict'; module.exports = globalThis.DOMException;`),
 `index.d.cts`
(`declare const _: new (message?: string, name?: string) => Error; export = _;`),
plus `mise.toml` and `README.md`.

`pnpm-workspace.yaml`'s `overrides` block wires it in with
`'node-domexception': 'link:package/pnpm-override/shim-node-domexception'`.
 pnpm rewrites
every transitive `node-domexception` edge to point at the workspace path;
 the
real npm package is no longer installed under `node_modules/.pnpm/`.

The shim mechanism is API-identical-by-construction to `node-domexception@1.0.0`,
so every consumer of the package keeps the same observable behaviour at runtime.
The drop-in nature is what justifies the global (not parent-scoped) override.

See `doc/dependency-blocklist.md` for the mechanism reference,
 the decision
rule (throw / silent / remove / shim),
 and the worked example.

### `@google/genai` (suppressed via pnpm override)

The override line `'@earendil-works/pi-ai>@google/genai': '-'` in
`pnpm-workspace.yaml` removes `@google/genai` (and its `protobufjs`,
`google-auth-library`,
 `gcp-metadata`,
 `gaxios` subtree) from
`@earendil-works/pi-ai`'s resolved dependencies.
 After a clean
reinstall (`rm -rf node_modules && pnpm install`),
 the package is
genuinely absent from `node_modules/.pnpm/`.
 A non-clean `pnpm install`
or `pnpm install --force` can preserve orphan symlinks from a previous
install state;
 if those need clearing,
 `pnpm prune` removes them.

#### Why suppression is safe at the runtime layer

`@earendil-works/pi-ai` reaches `@google/genai` only through dynamic
`import()`,
 so pi-ai does not crash at module load when the package is
absent:

- `dist/providers/register-builtins.js:48-63` defines `createLazyStream`,
  whose returned closure invokes `loadModule()` only when the stream is
  called.
   The wrapper has a `.catch` that converts a module-resolution
  failure into a stream-error event.
- `dist/providers/register-builtins.js:100-118` defines
  `loadGoogleProviderModule` and `loadGoogleVertexProviderModule`,
   which
  call `import("./google.js")` and `import("./google-vertex.js")` lazily.
- The static `import "@google/genai"` lines live in `dist/providers/google.js:1`,
  `dist/providers/google-vertex.js:1`,
   and `dist/providers/google-shared.js:4`.
  All three modules are reached only via the lazy chain above;
   no `.js` file
  outside that chain static-imports any of them.

#### What stops working

Nothing in normal operation.
 `package/pi-plugin/auto-mode` only references the
Google APIs by string identifier in `src/judge-tool.ts:83`
(`toolChoiceForApi`) and the matching unit tests,
 which never touch
`@google/genai`.
 If a Google model were ever routed through `streamSimple`,
pi-ai's `createLazyStream` catches the resolution error and emits a
stream-error event.

#### Note on `partial-json`

`@earendil-works/pi-ai>partial-json` was previously in the same override
block but cannot be suppressed:
 `pi-ai/dist/utils/json-parse.js:1`
statically imports `partial-json`,
 and that module is re-exported from
pi-ai's entry (`dist/index.js:12: export * from "./utils/json-parse.js"`).
Any consumer importing pi-ai (including `package/pi-plugin/auto-mode`) crashes
at module load when `partial-json` is absent.
 The override was removed
to restore the runtime contract.
 The remaining `@earendil-works/pi-ai>*`
overrides target packages that pi-ai either does not import
(`chalk`,
 `undici`,
 `zod-to-json-schema`) or imports only inside lazy
provider modules (`@aws-sdk/client-bedrock-runtime`,
 `@google/genai`,
`@mistralai/mistralai`,
 `proxy-agent`).

`@earendil-works/pi-coding-agent>*` and `@earendil-works/pi-tui>*`
received the same audit.
 Every dep that pi-coding-agent's `dist/index.js`
re-exports through a module with a static import was removed from the
override list:
 `chalk`,
 `cli-highlight`,
 `diff`,
 `extract-zip`,
`file-type`,
 `glob`,
 `hosted-git-info`,
 `ignore`,
 `jiti` (imported as
`jiti/static`),
 `minimatch`,
 `proper-lockfile`,
 `strip-ansi`,
 `uuid`,
`yaml`.
 The same applies to pi-tui's `get-east-asian-width` (`utils.js`)
and `marked` (`markdown.js`).

`@earendil-works/pi-tui>koffi` is removed.
 The package declares `koffi`
as an optional dependency,
 and `dist/terminal.js` only reaches it inside
`enableWindowsVTInput()`:
 the method returns immediately unless
`process.platform === "win32"`,
 then wraps `cjsRequire("koffi")` in
`try`/`catch`.
 If the require fails,
 pi-tui degrades Windows VT input
support so Shift+Tab is not distinguishable from Tab;
 Linux and macOS
never reach the import.
 Parent-scoped removal is therefore the lightest
blocklist mechanism:
 no native FFI package reaches `node_modules`,
 and
pi-tui's documented fallback path remains intact.

The overrides retained for pi-coding-agent are:

- `@mariozechner/clipboard`:
   `utils/clipboard-native.js` wraps
  `require("@mariozechner/clipboard")` in `try`/`catch`,
   so the missing
  package falls back to `clipboard = null`.
- `marked`:
   pi-coding-agent vendors `core/export-html/vendor/marked.min.js`
  and does not statically import the npm `marked` package.

`undici` was previously retained as a removal override on the assumption that
"library consumers never load it.
" That audit missed the `bin` entry:
`package.json` maps `"pi": "dist/cli.js"`,
 and `dist/cli.js:8` statically
imports `undici` (it installs an `EnvHttpProxyAgent` with disabled body /
header timeouts).
 The workspace ships a `commit` shell alias that resolves
to `pi 'commit all'`,
 so the `pi` binary is a first-class consumer of this
workspace.
 Run-the-binary,
 not just import-the-library,
 is part of the
"reachable from each pi-* package's entry" audit.
 Removal is still rejected.

As of 2026-06-19,
 `pnpm audit --json` and GitHub Dependabot alerts #57,
 #58,
and #59 reported three advisories against `undici@8.3.0`:
GHSA-38rv-x7px-6hhq / CVE-2026-9675,
GHSA-pr7r-676h-xcf6 / CVE-2026-9678,
and GHSA-vmh5-mc38-953g / CVE-2026-9697.
 The vulnerable range is
`>=8.0.0 <8.5.0`,
 and the patched floor is `8.5.0`.
 Every audit path reached
`undici` through `@earendil-works/pi-coding-agent`,
 and
`@earendil-works/pi-coding-agent@0.79.7` still pins `undici: 8.3.0`,
 so a
normal pi package bump does not clear the advisory.

`pnpm-workspace.yaml` now carries a global version override,
`'undici': '>=8.5.0'`.
 This is not a blocklist/removal override:
 it keeps
`undici` installed for the reachable pi CLI path while forcing any present or
future workspace consumer onto the patched security floor.
 Regenerating the
lockfile with `pnpm install --lockfile-only --ignore-scripts` resolves
`undici@8.5.0`,
 and `pnpm audit --json` returns zero advisories.

The overrides retained for pi-tui are `chalk` and `mime-types`,
 neither
of which the dist statically imports.
 `koffi` is removed for the optional
Windows-only fallback described above.

After this cleanup,
 `mise run //package/pi-plugin/{auto-mode,morph-compact}:test:unit`
both pass.
 `package/pi-plugin/terminal-title:test:unit` fails on unrelated string
assertions (`expected '✳ X' to equal 'π X'`) that predate the override
work;
 those are test-fixture issues to fix separately.

### `image-size` (removed through an `@loaders.gl/textures` override)

#### Symptom

On 2026-08-12,
Dependabot alerts
[#77](https://github.com/Aquaticat/Monochromatic/security/dependabot/77) and
[#78](https://github.com/Aquaticat/Monochromatic/security/dependabot/78)
reported `image-size@0.7.5` for
GHSA-5p2g-fcmc-qvqq / CVE-2025-71329 and
GHSA-w3rx-r6r6-pgpr / CVE-2025-71330.
Both advisories cover every published `image-size` version through `2.0.2`,
and both list no patched version.

The pre-remediation dependency chain was:

```text
@monochromatic-dev/dependency-catalog-cli-cube
└── @deck.gl/mesh-layers@9.3.9
    └── @loaders.gl/gltf@4.4.4
        └── @loaders.gl/textures@4.4.4
            └── texture-compressor@1.0.2
                └── image-size@0.7.5
```

`pnpm why image-size --recursive` reproduced that chain and found no second consumer.

#### Root cause

The npm package `@loaders.gl/textures@4.4.4` was inspected at its registry
`gitHead`,
`visgl/loaders.gl@44e7a4e978a63fad0ee257fedb688826f5f279e5`.
`modules/textures/package.json:77-85` declares `texture-compressor`
unconditionally:

```json
"dependencies": {
  "@loaders.gl/images": "4.4.4",
  "@loaders.gl/loader-utils": "4.4.4",
  "@loaders.gl/schema": "4.4.4",
  "@loaders.gl/worker-utils": "4.4.4",
  "@math.gl/types": "^4.1.0",
  "ktx-parse": "^0.7.0",
  "texture-compressor": "^1.0.2"
}
```

The package never imports the dependency as JavaScript.
At the same commit,
`git grep texture-compressor -- modules/textures` found only the manifest edge,
a documentation link,
a comment,
and the CLI argument.
`modules/textures/src/lib/encoders/encode-texture.ts:17-30` invokes the name
through `npx`:

```ts
const args = [
  // Note: our actual executable is `npx`, so `texture-compressor` is an argument
  'texture-compressor',
  '--type', 's3tc',
  '--compression', 'DXT1',
  '--quality', 'normal',
  '--input', inputUrl,
  '--output', outputUrl
];
const childProcess = new ChildProcessProxy();
await childProcess.start({
  command: 'npx',
  arguments: args,
  spawn: options
});
```

The declaration therefore exists only to put the CLI on `npx`'s local search path
for the experimental `CompressedTextureWriter`.
The workspace never names that writer or its encoder.
Its only `@deck.gl/mesh-layers` imports are `SimpleMeshLayer` in
`package/dependency-catalog/cli-cube/src/deck-layers.ts:33` and
`package/dependency-catalog/cli-cube/src/deck-scatter.ts:31`.
A repository-wide search found no first-party
`CompressedTextureWriter`,
`encodeImageURLToCompressedTextureURL`,
or direct `@loaders.gl/textures` reference.

The upstream diagnosis matches this source trace.
[visgl/loaders.gl#3552](https://github.com/visgl/loaders.gl/issues/3552)
documented the unconditional vulnerable chain,
and
[visgl/loaders.gl#3553](https://github.com/visgl/loaders.gl/pull/3553)
removed it on 2026-08-12 at merge commit
`ce8b32afcd24c5d59b96321424f2bc9c8f83e1f5`.
That commit's `docs/upgrade-guide.md:149-152` says the CLI was only used by
`CompressedTextureWriter` and all other texture loaders and writers are unaffected.
No stable release contains the change yet:
`@loaders.gl/textures@4.4.4` remains the latest stable package,
while the source tree is on `5.0.0-alpha.1`.

#### Verified workaround

`pnpm-workspace.yaml` now carries this parent-scoped removal:

```yaml
'@loaders.gl/textures>texture-compressor': '-'
```

This matches the dependency removal already merged upstream without replacing
`@deck.gl/mesh-layers` or changing the dependency-audit visualization.
The scope matters:
if another package deliberately adopts `CompressedTextureWriter`,
it must install and audit a compressor itself rather than inheriting this removal
silently.

Tradeoff:
the installed 4.4.4 writer still runs `npx texture-compressor` without
`--no`,
so invoking that unused writer could ask npm to download the missing CLI.
The workspace does not expose or invoke the writer.
Upstream merge commit `ce8b32a` adds `npx --no -- texture-compressor` so its
unreleased writer rejects instead of downloading when the application does not
supply the CLI.

#### Verification

The remediation was tested with pnpm 11.21.0 on 2026-08-12.

**Vulnerable catalog before the override:**

- `pnpm why image-size --recursive` reported the single chain through
  `texture-compressor@1.0.2`.
- `pnpm-lock.yaml` contained resolution and snapshot entries for
  `texture-compressor@1.0.2` and `image-size@0.7.5`.
- GitHub reported alerts #77 and #78 as open against `pnpm-lock.yaml`.

**Clean catalog after the override:**

- `mise run prepare:pnpm:install` completed successfully and removed both packages.
- `pnpm why image-size --recursive` and
  `pnpm why texture-compressor --recursive` produced no dependency paths.
- An uncapped search of `pnpm-lock.yaml` found no `image-size` or
  `texture-compressor` resolution,
  snapshot,
  or dependency edge.
- `pnpm audit --json` reported no advisories:
  zero low,
  moderate,
  high,
  or critical vulnerabilities across 718 resolved dependencies.
- `mise run //package/dependency-catalog/cli-cube:build`,
  `:test:unit`,
  and `:lint:types` passed with the compressor absent.

`mise run //package/dependency-catalog/cli-cube:lint:oxlint` remains red on 50 errors
and 445 warnings in untouched package source.
The security commits contain no change under
`package/dependency-catalog/cli-cube/src`,
and the lockfile already resolved the same
`oxlint@1.78.0` before this remediation,
so that baseline is not evidence against the dependency removal.

A disposable built-CLI fixture advanced through package loading and the npm probe,
then hit an existing entry-path mismatch:
`render-html` requests
`package/dependency-catalog/cli-cube/src/scripts/controller.ts`,
while the tracked file is
`package/dependency-catalog/cli-cube/src/script/controller.ts`.
The failed boundary probe does not exercise `texture-compressor` or `image-size`;
build,
unit,
type,
lockfile,
and audit checks are the evidence for this scoped dependency change.

The generated lockfile also synchronized stale catalog specifier metadata already
present in `pnpm-workspace.yaml`.
Those entries retained their existing resolved versions;
the only removed package resolutions were `texture-compressor@1.0.2` and
`image-size@0.7.5`.

#### What does not work

- Upgrading `@deck.gl/mesh-layers` from 9.3.9 to the current 9.3.10 does not
  remove the chain.
  Its published manifest still requests `@loaders.gl/gltf@^4.4.3`,
  and the latest stable `@loaders.gl/textures@4.4.4` still declares
  `texture-compressor`.
- Overriding `image-size` to another npm release cannot patch the advisories.
  GitHub marks every release through the latest `2.0.2` vulnerable and lists no
  patched version.
- Patching `image-size@0.7.5` locally would retain a version that Dependabot
  identifies as vulnerable,
  and it would maintain dead code that no workspace path uses.
- Dismissing the alerts as unused would leave the vulnerable package installed.
  Removing the unreachable CLI edge produces a clean dependency graph instead.
- Removing `@deck.gl/mesh-layers` would remove the vulnerable chain,
  but it would also remove the `SimpleMeshLayer` geometry used for every scatter
  glyph and axis arrowhead.

#### Upstream filing decision

No `.out-of-scope/` entry covers loaders.gl,
`texture-compressor`,
or dependency security remediation.
The upstream issue and pull-request searches found exact duplicates,
so no new issue or comment should be posted.
Issue #3552 and merged PR #3553 already contain the dependency chain,
source trace,
workaround,
and verification;
a comment from this workspace would add nothing.
Open follow-up PR #3555 only changes how an application-supplied compressor is
resolved and does not restore the vulnerable dependency for ordinary consumers.

The filing constraints resolve as follows:

1. **Upstream's fault?**
   Yes.
   Version 4.4.4 installs a deprecated CLI for every textures consumer even though
   source code only launches it for one experimental writer.
2. **Can upstream fix it?**
   Yes.
   PR #3553 removed the manifest edge while preserving the application-supplied
   CLI path.
3. **Does upstream support the use case?**
   Yes.
   `@loaders.gl/textures` documents `CompressedTextureWriter`,
   and package installation without an unused vulnerable tool is part of that
   published surface.
4. **Would upstream welcome the contribution?**
   Yes.
   `CONTRIBUTING.md:1-13` welcomes contributions,
   `.github/ISSUE_TEMPLATE/bug-report.yml:13-28` requests actionable,
   deduplicated reports,
   no contribution policy or tracker result bans AI-assisted work,
   and maintainers merged PR #3553.
5. **Will upstream fix it?**
   Yes,
   already fixed on `master` by merge commit `ce8b32a`.
6. **Was a minimal compatible fix prototyped?**
   Yes.
   The candidate fix was absent from installed 4.4.4 and present on current
   `master`.
   The workspace applied the same dependency removal at its consumer boundary,
   regenerated the lockfile,
   and verified the clean graph and zero-advisory audit.

**Decision:
post nothing upstream.**
The exact issue is closed by a merged fix,
and this section is the workspace audit trail until a stable loaders.gl release
carries it.

### ms (kept intentionally; no override)

`pnpm install` emits no warning for `ms`;
 the package is not deprecated and
carries no security advisory as of 2026-05-12.
The upstream source is 3024 bytes across 162 lines,
 MIT-licensed,
 with zero
runtime dependencies.
The investigation below records why the blocklist mechanism was considered
and rejected,
 so future reviewers do not repeat the analysis.

#### Dependency chain

```text
ms@2.1.3
├─┬ debug@4.4.3
│ ├── @tokenizer/inflate → file-type → @earendil-works/pi-coding-agent
│ │                     → @monochromatic-dev/pi-{auto-mode,morph-compact,terminal-title}
│ ├── extract-zip → @earendil-works/pi-coding-agent [deduped]
│ ├── micromark → @monochromatic-dev/dev-script-inference-canary-viewer
│ │             → @monochromatic-dev/webapp-content-messages-demo
│ │             → mdast-util-from-markdown → remark-* → @mdx-js/mdx
│ │             → @monochromatic-dev/ssg-aquati.cat
│ └── stylelint → @monochromatic-dev/config-stylelint
│                → stylelint-config-recommended → stylelint-config-standard
│                → monochromatic (workspace root devDependency)
└─┬ logform@2.7.0
  ├── winston → neovim → @monochromatic-dev/mcp-nvim
  └── winston-transport → winston [deduped]
```

`pnpm why ms` reproduces this on demand.
No workspace package imports `ms` directly;
 the only declared consumers are
`debug@4.4.3` and `logform@2.7.0`.

#### API surface used

Both consumers call `ms(number)` for the short-form duration humaniser and
nothing else.

- `debug/src/common.js:14` assigns `createDebug.humanize = require('ms')`,
  and `debug` calls `ms(this.diff)` to produce log prefixes such as `+2m`.
- `logform/ms.js:4` imports `ms`;
   `logform/ms.js:15` writes
  `info.ms` as a template literal that prepends `+` to `ms(this.diff)`.

Neither path uses the string-parser overload (`ms('2 days')`) or the
`{ long: true }` formatter option.

#### Why each blocklist action was rejected

- Removal (`'ms': '-'` in `pnpm-workspace.yaml`):
  `debug/src/common.js:14` hard-requires `ms` at module load.
  A missing module crashes every code path that loads `debug` with
  `MODULE_NOT_FOUND`,
   which transitively breaks stylelint,
   micromark,
  pi-coding-agent,
   and every workspace target listed in the chain above.
- Throwing stub (`POLICY` entry with `action: 'throw'` in `.pnpmfile.mjs`):
  loading the workspace stub throws at the same import site.
  Same crash as removal,
   with a custom error message;
   not a viable
  replacement.
- Silent stub (`POLICY` entry with `action: 'silent'` in `.pnpmfile.mjs`):
  the call returns the callable Proxy defined in
  `package/pnpm-override/stub-silent/index.cjs:3-19`.
  Embedding the result in a template literal (the consumer pattern at
  `logform/ms.js:15`) triggers primitive coercion.
  Every Proxy `get` returns the Proxy itself,
   including `Symbol.toPrimitive`,
  so coercion fails with
  `TypeError: Cannot convert object to primitive value`.
  The build does not stay green.
- API-compatible shim (`package/shim/ms/` wired via a `link:` override in
  `pnpm-workspace.yaml`):
  the pattern works technically;
   the precedent is
  `package/pnpm-override/shim-node-domexception/`.
  The cost is one new workspace package (implementation,
   type declaration,
  tests,
   README,
   `mise.toml`) plus this audit-trail entry,
   set against no
  current trigger.
  The upstream package is not deprecated,
   has no advisory,
   and emits no
  install warning.
  A reimplementation must reproduce upstream behaviour exactly:
   rounding
  boundaries,
   the 1.5x plural threshold for the long form,
   the 100-character
  parse cap,
   and `NaN` / non-finite handling.
  The downstream `debug` output is read by tooling across the ecosystem;
  subtle deviation regresses log fixtures.
  The maintenance burden and regression risk outweigh the marginal
  governance benefit of removing one well-behaved transitive.

#### Conditions for revisiting

Reopen the question when any of the following occurs:

- Upstream `ms` is deprecated,
   or pnpm starts emitting an install warning
  for it.
- A security advisory is filed against any maintained `ms` release.
- The package's license changes from MIT.
- A workspace package gains a direct `import 'ms'`,
   making first-party
  code subject to the policy.
- A workspace-wide policy is adopted that bans unjustified external
  transitive dependencies regardless of pain point.

The `@types/ms@2.1.0` types-only package,
 pulled in via
`micromark → @types/debug → @types/ms`,
 is left in place for the same
reason:
 no runtime impact,
 no install warning,
 no advisory.
If a shim is ever introduced,
 override `@types/ms` to `'-'` so the
workspace shim's own `index.d.cts` owns the types.

See `doc/dependency-blocklist.md` for the decision rule
(throw / silent / remove / shim) and the `node-domexception` entry above
for the worked example of the API-compatible shim path.

### `proper-lockfile` (substituted via workspace shim)

The upstream `proper-lockfile@4.1.2` (moxystudio/node-proper-lockfile,
 last
commit 2021-01) is abandoned.
 It enters the resolved graph as a transitive
of `@earendil-works/pi-coding-agent@0.74.0`:
`proper-lockfile@4.1.2 → @earendil-works/pi-coding-agent → @monochromatic-dev/pi-plugin-auto-mode`.
`pnpm why proper-lockfile` reproduces the chain.

The override `'proper-lockfile': 'link:package/pnpm-override/shim-proper-lockfile'` in
`pnpm-workspace.yaml` substitutes the abandoned upstream with a workspace shim
whose `index.cjs` implements `lockSync(path, options)` and
`lock(path, options)` via `node:fs.mkdirSync` on a sibling `.<basename>.lock`
directory.
 The shim is decoupled from the silent stub's thenable trap (see
below),
 so both the sync withLock and async withLockAsync paths in
pi-coding-agent work unchanged.

#### Why it cannot be removed

The two consumer modules inside pi-coding-agent both hard-import the package
at module scope:

- `dist/core/auth-storage.js:12`:
   `import lockfile from "proper-lockfile";`
- `dist/core/settings-manager.js:4`:
   `import lockfile from "proper-lockfile";`

Both modules are re-exported from the package's barrel:

- `dist/index.js:6`:
   `export { AuthStorage, FileAuthStorageBackend, InMemoryAuthStorageBackend } from "./core/auth-storage.js";`
- `dist/index.js:22`:
   `export { SettingsManager } from "./core/settings-manager.js";`

A `'proper-lockfile': '-'` override removes the package from the install but
leaves the static import in `auth-storage.js` and `settings-manager.js`:
loading `@earendil-works/pi-coding-agent` from `package/pi-plugin/auto-mode` crashes
with `Cannot find package 'proper-lockfile'` before any first-party code runs.

#### Why the silent stub does not work

`package/pnpm-override/stub-silent/index.cjs` is a `Proxy` over a no-op function whose
`get` trap returns `module.exports` for every property.
 That makes the stub
a thenable:
 `await stub` enters the Promise-resolution machinery,
 calls
`stub.then(resolve, reject)`,
 the `apply` trap returns the stub (neither
callback is invoked),
 and the await never settles.

Consequence:

- Sync path (`SettingsManager` startup load,
   `settings-manager.js:69` only
  acquires the lock when the settings file already exists;
   sync release
  returns the Proxy itself,
   the `release()` call is harmless):
   works.
- Async path (`AuthStorage.getApiKey` during model resolution,
   reachable
  from `core/model-registry.js:519` and `:585`):
   `release = await lockfile.lock(...)`
  hangs on the thenable trap.
   The pi CLI hangs on startup model lookup.

A `then` carve-out in the silent stub would fix this,
 but the workspace
silent stub does not currently have one,
 and editing the stub to add it
weakens the contract for every other policy entry.

#### Why upgrading does not help

`proper-lockfile@4.1.2` is the latest published version (the repo's last
commit is from 2021-01).
 Upstream is unmaintained;
 there is no upgrade path.

#### Runtime call sites exercised by the shim

Every pi invocation reaches one or both of these paths:

- `dist/main.js:377`:
   `SettingsManager.create(cwd, agentDir)` runs at
  startup.
   `FileSettingsStorage` only calls `lockfile.lockSync` when the
  settings file already exists (`settings-manager.js:69`);
   on a fresh
  machine,
   the sync lock is skipped until the file is created.
- `dist/main.js:408`:
   `AuthStorage.create()` runs at startup.
- `dist/core/sdk.js:90,92`,
   `dist/core/agent-session-services.js:56,57`,
  `dist/core/resource-loader.js:121`,
   `dist/package-manager-cli.js:304,358`
  ;
   defaulted SDK construction;
   same call shape.
- `dist/core/model-registry.js:519`:
   `await this.authStorage.getApiKey(model.provider, ...)`
  runs during model resolution.
   `FileAuthStorageBackend.getApiKey` goes through
  `withLockAsync`,
   which does `release = await lockfile.lock(...)` and
  `await release()`.
   Both ends of the async lock contract must execute
  correctly for `pi` to start.

#### Shim contract and known simplifications

Implemented:

- `lockSync(path, options)`:
   synchronous,
   atomic `mkdirSync` of `.<basename>.lock`
  in the target's parent directory.
   Throws an `Error` with `code === 'ELOCKED'`
  on `EEXIST`.
   Callers handle retries themselves (`auth-storage.js:32-54`
  and `settings-manager.js:38-60` both wrap the call in a 10-attempt sync
  retry loop with a 20ms busy-wait between attempts).
- `lock(path, options)`:
   async with internal retry per `options.retries`
  (number or object).
   Backoff is exponential by `factor` starting at
  `minTimeout`,
   capped by `maxTimeout`.
   The return value is a callable
  release function;
   `await release()` works because `await undefined`
  resolves immediately.

Omitted (relative to upstream):

- No `fs.realpath` resolution.
   Both pi-coding-agent callsites always pass
  `{ realpath: false }` (`auth-storage.js:38`,
   `settings-manager.js:44`),
  so the shim treats every target as already-resolved.
- No stale-lock detection.
   If pi crashes while holding the lock,
   the next
  invocation throws ELOCKED until the user removes
  `<agentDir>/.auth.json.lock` or `<agentDir>/.settings.json.lock` manually.
- No `onCompromised` callback.
   The option is accepted but never invoked;
  pi-coding-agent's `lockCompromised` flag stays false,
   so its
  `throwIfCompromised()` checks are no-ops.
- No `retries.randomize` jitter.
   Backoff is deterministic.

The workspace does not run concurrent pi instances,
 so the omissions are
acceptable.
 A `pi` crash recovery would require manual `rmdir` on the lock
directory.

#### Verification caveat

The plan's verification step that runs
`node -e "import('@earendil-works/pi-coding-agent').then(...)"` only proves
the module loads through the shim resolution.
 It does not exercise
`withLockAsync` under a real `getApiKey` call.
 The earlier steps
(direct shim probes for sync acquire,
 async acquire,
 ELOCKED throw)
cover the shim contract;
 the end-to-end `pi --help` step covers `SettingsManager.create`
but only reaches the sync lock acquisition when `<agentDir>/settings.json`
already exists (see `settings-manager.js:69`).
 To exercise the async lock
path during verification on a fresh host,
 run `pi` once interactively or
seed `<agentDir>/settings.json` with `{}` before re-running `pi --help`.

See `doc/decision/proper-lockfile-removal.md` for the decision rationale
(why a shim was preferred over silent stub or pure removal) and
`doc/dependency-blocklist.md` for the policy reference.

### `convert-source-map` (throwing stub; never loaded by Stryker)

This is the one pre-emptive ban that has since entered the resolved graph.
`@monochromatic-dev/dev-script-mutation-test` runs StrykerJS 9.6.1,
 which pulls
`convert-source-map` transitively.
 The `action: 'throw'` policy substitutes it with
`@monochromatic-dev/pnpm-override-stub-throwing`,
 so the canary in "Pre-emptive bans" above has
fired for this one package.
 Mutation testing is unaffected:
 Stryker never loads the
package at runtime,
 so the throwing stub is never imported.

#### Dependency chain

```text
@monochromatic-dev/dev-script-mutation-test
└─┬ @stryker-mutator/core@9.6.1
  └─┬ @stryker-mutator/instrumenter@9.6.1
    └─┬ @babel/core@7.29.7
      └── convert-source-map  (substituted: link:package/pnpm-override/stub-throwing)
```

`@babel/core@7.29.7` is the sole consumer (`pnpm-lock.yaml:6129`,
`convert-source-map: link:package/pnpm-override/stub-throwing`).
 No workspace package depends on
`@babel/core` directly;
 it arrives only through the Stryker instrumenter.

#### Why runtime is fine

Stryker's instrumenter is parse-only.
 For each source file it calls
`babel.parseAsync` (`@stryker-mutator/instrumenter/dist/src/parsers/ts-parser.js`,
`js-parser.js`),
 wraps the AST in `new File(...)`
(`transformers/babel-transformer.js`),
 traverses it manually,
 and prints with
`@babel/generator` (`printers/ts-printer.js`,
 `mutant.js`,
`generate(..., { sourceMaps: false })`).
 None of these enter Babel's `transform`
pipeline.

Every `convert-source-map` call site in `@babel/core` lives in that transform
pipeline and is a lazy,
 memoized `require`:
`lib/transformation/normalize-file.js:21-27,56,62`,
 `lib/transformation/file/generate.js`,
`lib/transformation/read-input-source-map-file.js`.
 In `normalize-file.js` the require
fires only when `options.inputSourceMap !== false` and the source carries an
inline-sourcemap object or a `//# sourceMappingURL` comment (lines 54-81).
 Hand-written
`.ts` source has neither,
 and `babel.parseAsync` routes through `lib/parse.js` ->
`parser/index.js`,
 never `normalize-file.js`.

A sweep of every `@stryker-mutator/*` dist (core,
 instrumenter,
 typescript-checker,
util,
 api) found zero `transformSync` / `transformAsync` / `transformFromAst`,
 zero
direct `convert-source-map` import,
 and zero `istanbul-lib-source-maps`.
 The
typescript-checker uses the TypeScript compiler API,
 not Babel.
 The mutation config
sets `coverageAnalysis: 'off'` and uses the `command` test runner,
 so no
coverage or source-map path runs either.

Verified empirically (read-only probe,
 2026-06-06):
 instrumenting a sample `.ts`
string through the real `Instrumenter` produced 16 mutants and instrumented output
while a `Module._load` interceptor recorded zero requests for `convert-source-map`.
As a positive control,
 a direct `require('convert-source-map')` resolved from
`@babel/core`'s location flipped the same interceptor and threw the stub's
`[blocked-dep]` error,
 confirming both that the interceptor catches real requests and
that the installed module is the throwing stub.

#### Why the throw action is correct

Babel's `require("convert-source-map")` is a hard require with no `try/catch` guard,
so `throw` (a loud,
 doc-pointing failure at the import site) is the right substitution
per the decision rule in "Global blocklist (substitution vs removal)" above.
 It stays
inert only because Stryker never exercises the transform pipeline that would trigger it.

#### Why removal was not used

A parent-scoped override `'@babel/core>convert-source-map': '-'` in `pnpm-workspace.yaml`
would also work and is functionally equivalent for this graph.
 The throwing stub was
kept instead because its message points at `doc/dependency-blocklist.md`,
 which is more
useful than a bare `Cannot find module 'convert-source-map'` if a future change ever
reaches the transform pipeline.
 Revisit and switch to removal (or a functional shim) if:
a Stryker upgrade starts calling `babel.transform*`,
 `coverageAnalysis` is enabled,
 or a
plugin or reporter is added that transforms code.

#### Verification

`grep convert-source-map pnpm-lock.yaml` shows only the `@babel/core` edge.
 A real
mutation run (for example `mise run //package/file-enforcer/core:test:mutation`)
completes and reports killed and survived mutants without any `convert-source-map`
error.
 See `doc/troubleshooting/stryker-container-runtime.md` for the Stryker runtime
notes and `doc/dependency-blocklist.md` for the policy reference.

### `js-yaml` (throwing stub; never loaded by cosmiconfig)

`js-yaml` is not a pre-emptive ban;
 it was already in the resolved graph,
 pulled
transitively by `stylelint@17.6.0` through `cosmiconfig@9.0.1`.
 The workspace does
not consume it:
 all YAML parsing uses the `yaml` package
(`package/dependency-catalog/cli-cube/src/catalog.ts`,
`package/ssg/aquati.cat/src/lib/content.ts`).
 The `action: 'throw'` policy
substitutes it with `@monochromatic-dev/pnpm-override-stub-throwing`,
 removing the real js-yaml
package,
 and the `argparse` edge it carried,
 from `node_modules`.

#### Dependency chain

```text
stylelint@17.6.0
└─┬ cosmiconfig@9.0.1
  └── js-yaml  (substituted: link:package/pnpm-override/stub-throwing)
```

`cosmiconfig` is the sole consumer of js-yaml in the tree (verified against every
workspace `package.json` and `pnpm-lock.yaml`).
 No workspace package depends on
cosmiconfig directly;
 it arrives only through stylelint.

#### Why runtime is fine

cosmiconfig references js-yaml in exactly one place,
 a lazy require inside
`loadYaml` (`dist/loaders.js`):

```js
let yaml;
function loadYaml(filepath, content) {
  if (yaml === undefined) { yaml = require('js-yaml'); }
  return yaml.load(content);
}
```

`loadYaml` is registered (`dist/defaults.js:92-94`) only for the `.yaml`,
 `.yml`,
and extensionless (`noExt`) config formats;
 `.mjs`/`.cjs`/`.js` route to `loadJs`,
`.ts` to `loadTs`,
 and `.json` to `loadJson`,
 none of which touch js-yaml.
 The
repo's only stylelint config is `stylelint.config.mjs`,
 so cosmiconfig uses
`loadJs`,
 never `loadYaml`.
 There is no extensionless `.stylelintrc`,
 no
`.stylelintrc.yaml`/`.yml`,
 and no `"stylelint"` key in any `package.json`,
 so the
YAML loader is unreachable and `require('js-yaml')` never executes.
 No eager
js-yaml require exists in cosmiconfig outside `loaders.js`.

Verified empirically:
 `mise run lint:stylelint` loads `stylelint.config.mjs` and
completes with zero `[blocked-dep]` throws.
 As a positive control,
 requiring
js-yaml from cosmiconfig's resolution location throws the stub's `[blocked-dep]`
error,
 confirming the installed module is the throwing stub.

#### Why the throw action is correct

cosmiconfig's `require('js-yaml')` is a hard require with no `try/catch` guard,
 so
`throw` (a loud,
 doc-pointing failure at the import site) is the right substitution
per the decision rule in "Global blocklist (substitution vs removal)" above.
 It
stays inert because the YAML loader is never reached.

#### Why removal was not used

A removal (`'js-yaml': '-'`,
 or the parent-scoped `'cosmiconfig>js-yaml': '-'`,
 in
`pnpm-workspace.yaml`) would also keep the build green today and matches the
argparse precedent.
 The throwing stub was kept instead because its message points
at `doc/dependency-blocklist.md`,
 which is more useful than a bare
`MODULE_NOT_FOUND` if a future change ever adds a YAML stylelint config and reaches
`loadYaml`.
 Revisit and switch to removal (or a functional shim backed by the
`yaml` package) if a deliberate YAML stylelint config is introduced.

#### Verification

`rg js-yaml pnpm-lock.yaml` shows only the `link:package/pnpm-override/stub-throwing` redirect
under cosmiconfig,
 with no `js-yaml@4.1.1` resolution block.
 The argparse edge
js-yaml previously carried is gone;
 argparse remains removed for wawoff2 via the
`'argparse': '-'` override.
 See `doc/dependency-blocklist.md` for the policy
reference.

### `@homebridge/dbus-native` (replaces `dbus-next` in kwin-key-helper)

`dbus-next@0.10.2` (last published 2022-04-28,
 effectively unmaintained) was the
sole source of a burst of Dependabot advisories reached only through
`package/kwin/key-helper`.
 It was first mitigated with two `pnpm.overrides` (commit 516980e19) and then
retired for good by migrating the daemon to `@homebridge/dbus-native` (#369),
 which removes the vulnerable chain at its root.
 The interim overrides (`'dbus-next>usocket': '-'`,
 `'dbus-next>xml2js': '>=0.5.0'`) and the `allowBuilds: usocket: false` entry are
gone with `dbus-next` itself.

#### Original dependency chain (historical)

Thirteen of the advisories sat under `dbus-next`'s single optional native
dependency `usocket`:

```txt
kwin-key-helper
`-- dbus-next@0.10.2
    |-- xml2js@^0.4.17                       GHSA-776f-qx25-q3cc (prototype pollution)
    `-- usocket@0.3.0            (optional native)
        `-- node-gyp@7.1.2       (install-time build tool)
            `-- request@2.88.2   (deprecated, EOL) GHSA-p8p7-x288-28g6 (SSRF, no patch)
                |-- form-data                 GHSA-fjxv-7rqg-78g4, GHSA-hmw2-7cc7-3qxx
                |-- tar                        7 advisories (GHSA-8qq5-... through GHSA-vmf3-...)
                |-- tough-cookie              GHSA-72xf-g2v4-qvf3
                |-- qs                        GHSA-6rw7-vpxm-498p
                `-- uuid                      GHSA-w5hq-g745-h8pq
```

`request@2.88.2` was the last release that package will ever ship,
 so its SSRF advisory has no patched version;
 the only remediation is removing `request` from the tree,
 which the migration
does by construction.

#### Why `@homebridge/dbus-native` is clean

`@homebridge/dbus-native@0.7.7` is actively maintained (published 2026-07-08)
and its tree carries no `request`,
 `node-gyp`,
 `usocket`,
 or `tar`.
 Its transitive deps are `event-stream@4.0.1` (post-incident line,
 no advisory),
`minimist@1.2.8` (past the prototype-pollution fix),
 and `xml2js@0.6.2` (already
past GHSA-776f-qx25-q3cc).
 The swap reported `+8 -9` at install.

The server-only `sax` situation persists and the `sax-stub` alias stays:
`@homebridge/dbus-native/lib/bus.js` statically requires `./introspect`,
 which
loads `xml2js`,
 whose parser does `require('sax')` at module load.
 That path is client-only (parsing a remote object's introspection XML);
 key-helper is a D-Bus service and answers Introspect from `lib/stdifaces.js`
without `xml2js`,
 so `sax.parser()` is never reached at runtime.
 The shipped `index.d.ts` types only the client path,
 so the server surface used
here (`sessionBus`,
 `MessageBus.exportInterface`,
 `MessageBus.requestName`) is
declared locally in `package/kwin/key-helper/src/dbus-native.d.ts`.

#### Verification

`grep -c dbus-next pnpm-lock.yaml` is zero and `pnpm why request node-gyp usocket
tar -r` finds none under `@homebridge/dbus-native`.
 `mise run //package/kwin/key-helper:lint:types`,
 `:lint:oxlint`,
`:build:js:node`,
 and `:test:unit` all pass.
 The rebuilt SEA binary was run against the live KDE session bus:
 it registered
`org.monochromatic.KeyHelper`,
 `gdbus introspect` listed all five methods with
correct signatures,
 and `gdbus call ... SetActiveWindow "verifytestclass"`
returned an empty success reply,
 confirming method dispatch through
`@homebridge/dbus-native`.

### `katex` (dropped for GHSA-238p-pmpm-9mq7; substituted via a throwing workspace shim)

`katex` is not installed in this workspace.
Commit `c92efa816c` (2026-10-19) overrode `'micromark-extension-math>katex': '-'` for GHSA-238p-pmpm-9mq7 / CVE-2026-103923
(`>=0.11.0 <0.18.2` lets existing prototype pollution bypass katex's trust restrictions).
The only consumer is `micromark-extension-math@3.1.0`,
and its latest release still asks for `katex: ^0.16.0`,
so the fixed release cannot be reached in range.
Removal alone broke every unbundled load of `remark-math`;
the override now points at a shim,
`'micromark-extension-math>katex': 'link:package/pnpm-override/shim-katex'`,
whose every member throws and which never renders.

#### Dependency chain

```text
katex  (substituted: link:package/pnpm-override/shim-katex)
└─┬ micromark-extension-math@3.1.0
  └─┬ remark-math@6.0.0
    └── @monochromatic-dev/translation-repair-document-structure  (src/parse-mdx.ts imports remark-math)
```

#### Why removal was not enough

`micromark-extension-math`'s `package.json` exports only `"."`,
and its `index.js` re-exports `mathHtml` from `lib/html.js`.
`lib/html.js` runs `import katex from 'katex'` and reads `katex.renderToString` when the module loads.
With the package removed,
loading `remark-math` threw
`ERR_MODULE_NOT_FOUND: Cannot find package 'katex' imported from .../micromark-extension-math/lib/html.js`.
The workspace uses only the syntax extension (`math`),
never `mathHtml`,
but an ESM re-export loads the whole module either way.

Built bundles were not affected:
rolldown drops `lib/html.js` because the package declares `sideEffects: false`,
which is why corpus runs from built output kept working.

#### Why the failure stayed hidden

Unit tests import the unbundled sources,
so every test file that reaches `parse-mdx.ts` loads `remark-math` directly.
They passed only because the scratch worktrees sat under a directory that held its own `node_modules`
(another session's npm install),
which contained a `katex`.
Node found it by walking up the directory tree from `micromark-extension-math`.
That install was rewritten at 2026-10-21 17:10:03 UTC without `katex`,
and from then on 1,172 test files of 26 packages failed at load in worktrees under that directory.
The change that dropped `katex` had been checked the same way,
so its note that the unit suite passed,
math cases included,
rested on the parent directory and not on the repository.

#### Why a shim and not a stub

`package/pnpm-override/stub-throwing` throws when it is loaded,
which is the wrong moment here:
`lib/html.js` loads it on every import of `remark-math`.
`package/pnpm-override/shim-katex` loads cleanly and exposes `renderToString` and the other members as functions,
so the read in `lib/html.js` succeeds and only a call (or `new ParseError()`) throws `KatexNotInstalledError`.
The message names katex,
the advisory,
the override and the reason the shim exists.
This follows the precedent of `package/pnpm-override/shim-node-domexception`,
except that the shim cannot be API-identical,
because the real API is what the advisory rules out.

Its `index.d.cts` also exports a `KatexOptions` type:
once `katex` resolves to a package that has a `types` field,
TypeScript no longer falls back to `@types/katex`
for `micromark-extension-math`'s `import type { KatexOptions } from 'katex'`.
Measured on 2026-10-21 (UTC) with a probe that typed `{ trust: true }` as `HtmlOptions`:
with the type left out of the shim,
`trust` (a real katex option) was an unknown property,
so the types of the real package were not reached;
with the shim as committed,
it type-checks.

#### Verification

- Red,
  from a worktree with no `node_modules` in any parent directory and before the shim,
  one `document-structure` unit test file failed at load with the `ERR_MODULE_NOT_FOUND` above.
- pnpm honours `link:` in a `parent>child` selector override:
  `node_modules/.pnpm/micromark-extension-math@3.1.0/node_modules/katex` is a symlink to `package/pnpm-override/shim-katex`.
- Green,
  the same file and the whole `document-structure` unit task pass in that worktree;
  `package/pnpm-override/shim-katex` has its own tests for the import and for every member.
- The built `corpus-pass` bundle is byte-identical before and after the override:
  its pipeline digest and a hash over every output file are equal.

Revisit when `micromark-extension-math` accepts a `katex` release outside the advisory range,
or when the workspace needs `mathHtml`.
See `package/pnpm-override/shim-katex/README.md`.

## Why we do not file the policy entries upstream

Walked the 5-constraint upstream-filing check once for the whole policy
category (pre-emptive bans,
 node-domexception,
 @google/genai,
 ms,
proper-lockfile,
 and the global blocklist mechanism).
 The conclusion is
the same across all of them;
 doing the audit once at the category
level is correct since the constraints do not differ per-entry.

1. **Upstream's fault?
   ** No. Each upstream package functions as
   documented;
    the substitution / removal / ban is a workspace policy
   decision about runtime profile (Node 22+ / Bun),
    maintenance
   posture (3+ years abandoned),
    API surface (single-method
   utilities replaced by platform primitives),
    or licensing /
   provenance posture.
    The packages are not defective;
    they are
   unsuitable for this workspace's policy.
2. **Can upstream fix it?
   ** Not applicable.
    Upstream would have to
   un-abandon,
    re-license,
    or merge with a platform primitive.
    The
   policy decision is upstream-independent.
3. **Supporting this use case?
   ** Not applicable.
    The use case
   ("don't pull this package into this workspace") is workspace-local.
4. **Will they fix it?
   ** Not applicable.
    Several packages are
   abandoned (no commits in 3+ years);
    some are deprecated by their
   maintainer's own README ("Use your platform's native DOMException
   instead").
    The policy already takes upstream signals into account
   when making the decision.
5. **Minimal-fix prototype?
   ** Not applicable.

**Decision:
 no upstream report for any policy entry.
** Each entry's
documentation in this file is the workspace-internal audit trail
justifying the decision.
 The "Conditions for revisiting" subsections
spell out the trigger that would re-open the entry,
 so a future
reviewer can validate the decision without re-deriving the analysis.

The `vlt` section above is the only entry in this file that gets a
separate "Why we would file this upstream" walk because vlt's bug is
a genuine spec violation on their side,
 not a workspace policy choice.
