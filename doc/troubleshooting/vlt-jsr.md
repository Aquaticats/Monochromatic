# vlt 1.0.0-rc.24 install fails on `@jsr/*` transitive deps: JSR's npm bridge omits `GET /<pkg>/<version>` and vlt has no fallback

Status:
 unreported as of 2026-04-04.
 Discovered during Bun-to-vlt
migration.
 Affects vlt 1.0.0-rc.
24 (latest) with any `@jsr/*`
transitive dependency.

## Symptom

Running `vlt install` on a project that has any `@jsr/*`
transitive dependency fails.
 Minimal reproducer:

```bash
mkdir /tmp/vlt-jsr-repro && cd /tmp/vlt-jsr-repro
echo '{ "dependencies": { "happy-opfs": ">=2.0.2" } }' > package.json
echo '{ "config": { "scope-registries": { "@jsr": "https://npm.jsr.io/" } } }' > vlt.json
vlt install
# Error: failed to fetch manifest
# url: https://npm.jsr.io/@jsr/std__path/1.1.4
```

`happy-opfs@2.0.2` depends on
`"@std/path": "npm:@jsr/std__path@^1.1.4"`.
 Without
`scope-registries`,
 the error is a 404 on
`registry.npmjs.org/@jsr/std__path` (the `@jsr` scope does not
exist on npmjs.
org at all).

## Root cause

This is a dual bug:
 JSR's npm bridge does not implement the
version-specific manifest endpoint,
 and vlt does not fall back
to the full packument when the version-specific endpoint
returns 404.

### Bug A: JSR's npm bridge omits `GET /<pkg>/<version>`

The npm registry API includes a version-specific manifest
endpoint:

```bash
curl -sI https://registry.npmjs.org/@types/node/22.0.0
# HTTP/2 200
```

JSR's `npm.jsr.io` does not implement this:

```bash
curl -sI https://npm.jsr.io/@jsr/std__path/1.1.4
# HTTP/2 404
```

The full packument endpoint works fine:

```bash
curl -sI https://npm.jsr.io/@jsr/std__path
# HTTP/2 200
```

Source citations in [`jsr-io/jsr`](https://github.com/jsr-io/jsr):

- `lb/main.ts:89-119`:
   `npm.jsr.io` is a Cloudflare Worker that
  proxies to R2 static storage.
- `api/src/s3_paths.rs:62-71`:
   the R2 bucket stores only the
  full packument at the `@jsr/scope__name` key.
   No per-version
  objects are stored,
   so the version-specific URL path maps to a
  nonexistent R2 key.

Other registries have partial support too:
 GitHub Packages
(`npm.pkg.github.com`) returns 405 for the same pattern.

### Bug B: vlt has no fallback in `#registryManifestRequest`

Source citations in
[`vltpkg/vltpkg`](https://github.com/vltpkg/vltpkg)
(`src/package-info/src/index.ts`):

```ts
// Line 601-608: isSingle branch uses version-specific endpoint exclusively
const mani = spec.range?.isSingle
  ? await this.#registryManifestRequest(spec, options,) // /<pkg>/<version>
  : pickManifest(
    await this.packument(f, options,), // /<pkg> (full)
    spec,
    options,
  );
```

```ts
// Line 407: constructs the version-specific URL
const pakuURL = new URL(`${name}/${version}`, registry,);

// Line 413-423: throws immediately on non-200, no fallback
if (response.statusCode !== 200) {
  throw this.#resolveError(spec, options, 'failed to fetch manifest', {
    url: pakuURL,
    response,
  },);
}
```

vlt already knows JSR registries are different:
 it skips tarball
URL guessing for JSR at `spec/src/browser.ts:648-651`.
 The
equivalent carve-out is missing from the manifest resolution
path.

### Bug C (related but separate): `defaultScopeRegistries` not merged on first install

vlt has
`defaultScopeRegistries = { '@jsr': 'https://npm.jsr.io/' }` in
`spec/src/browser.ts:57-58`,
 which should auto-route `@jsr/*`
packages.
 However,
 `getOptions()` at line 72 uses
`options?.['scope-registries'] ?? {}` which does not merge
`defaultScopeRegistries`.
 The defaults are only merged during
lockfile loading
(`graph/src/transfer-data/load.ts:48-51`),
 not during fresh
resolution.

On a first install (no lockfile),
 `@jsr/*` transitive deps fall
through to `registry.npmjs.org` unless `scope-registries` is
manually configured in `vlt.json`.
 Even when configured,
 Bug B
blocks the install.

## Verification

Versions under test:

- vlt:
   1.0.0-rc.
  24 (latest as of 2026-04-04)
- Node.
  js:
   25.9.0
- Triggering package:
   `happy-opfs@2.0.2` ->
  `@jsr/std__path@^1.1.4`
- JSR source examined:
   `jsr-io/jsr` main branch,
   `lb/main.ts`
  and `api/src/s3_paths.rs`
- vlt source examined:
   `vltpkg/vltpkg` main branch,
  `src/package-info/src/index.ts`

The reproducer above runs cleanly against these versions.

## Verified workarounds (attempted; none satisfactory)

### Attempted: `scope-registries` for `@jsr` in `vlt.json`

Routes the request to `npm.jsr.io`,
 then hits Bug B (version
endpoint 404).
 Tradeoff:
 cleanly directed traffic;
 still
broken.

### Attempted: `jsr:` protocol in catalog entries

Works for direct deps but not their transitive `@jsr/*` deps.
Tradeoff:
 only fixes hand-written manifests;
 library authors'
manifests cannot be retrofitted.

### Attempted: using npm versions of JSR packages

Works for some (`zod`,
 `@optique/*`) but not all.
 Some npm
packages like `happy-opfs` have `@jsr/*` transitive deps baked
into their `package.json`.
 Tradeoff:
 depends on each library's
choices;
 not portable.

### Attempted: removing `scope-registries`

`@jsr/*` packages 404 on npmjs.
org instead.
 Tradeoff:
 same
result,
 different error string.

## What does not work

- Setting `--registry https://npm.jsr.io/` globally:
   rewrites
  all package fetches to JSR,
   breaks everything else.
- Patching `vlt.json` with a hand-written manifest for the
  affected package:
   vlt validates against the installed package
  manifest;
   the patch would have to be regenerated for every
  version bump.
- Waiting for an existing upstream fix:
   no PR open in either
  repo as of 2026-04-04.

## Why we would file this upstream (two reports)

All 5 constraints hold;
 the issue is genuinely upstream and
fixable.
 Walking the constraints for each report:

### vltpkg/vltpkg report

1. **Is it really upstream's fault?
   ** Yes;
    the `#registryManifestRequest` path has no fallback for registries that only serve full packuments.
2. **Can upstream fix it?
   ** Yes;
    three plausible patches (any one would work):
   - Option A:
      fall back to full packument on 404 in `#registryManifestRequest`.
   - Option B:
      skip the `isSingle` optimisation for JSR scope-registries (mirrors the existing tarball-guess carve-out).
   - Option C:
      merge `defaultScopeRegistries` into `getOptions()`
       so `@jsr` routing works on first install
       (this addresses Bug C but does not solve Bug B by itself).
3. **Are they supporting this use case?
   ** vlt has a built-in `@jsr` default in `spec/src/browser.ts:57-58`;
    JSR support is an explicit goal.
4. **Will they likely fix it?
   ** Probably;
    the project is in active rc development.
5. **Have we prototyped a minimal fix?
   ** Patches A,
    B,
    C are sketched above;
    only A is exercised against the reproducer.

Decision:
 worth filing.

### jsr-io/jsr report

1. **Is it really upstream's fault?
   ** Yes;
    npm-registry compatibility implies the version-specific endpoint.
2. **Can upstream fix it?
   ** Yes;
    implement the endpoint in the Cloudflare Worker by extracting the version from the cached full packument JSON.
    No R2 storage change needed.
3. **Are they supporting this use case?
   ** `npm.jsr.io` is the documented bridge for npm consumers;
    partial endpoint coverage breaks the contract.
4. **Will they likely fix it?
   ** Plausible;
    the team has been responsive to compatibility reports historically.
5. **Have we prototyped a minimal fix?
   ** Architectural sketch above;
    no prototype code.

Decision:
 worth filing.

## Draft upstream issues (kept as reference; revise before filing)

### Draft for vltpkg/vltpkg

```md
**Title**: `package-info`: version-specific manifest request 404s on registries that only serve full packuments (JSR, GitHub Packages)

**Labels**: bug, registry, jsr

**Description**:

`vlt install` fails on any project with `@jsr/*` transitive dependencies. The version-specific manifest URL (`/@jsr/<pkg>/<version>`) returns 404 on `npm.jsr.io`, and `#registryManifestRequest` has no fallback to the full packument endpoint.

Source trace:

- `src/package-info/src/index.ts:601-608`: `isSingle` branch uses the version-specific endpoint exclusively.
- `src/package-info/src/index.ts:407`: constructs the URL.
- `src/package-info/src/index.ts:413-423`: throws on non-200.

Reproduction: see test case above (omitted here for brevity; insert before filing).

**Suggested fix**: Option A is the safest; fall back to full packument on 404 in `#registryManifestRequest`. Option B (skip the optimisation for JSR scope-registries) mirrors the existing tarball-URL carve-out at `spec/src/browser.ts:648-651`.

Related issue: `defaultScopeRegistries` is not merged in `getOptions()` (`spec/src/browser.ts:72`), so `@jsr/*` routing fails on first install without an explicit `vlt.json`.
```

### Draft for jsr-io/jsr

```md
**Title**: `npm.jsr.io` does not implement version-specific packument endpoint (`GET /@jsr/<pkg>/<version>`)

**Labels**: bug, npm-compat

**Description**:

`npm.jsr.io` serves the full packument at `/@jsr/<pkg>` but returns 404 at `/@jsr/<pkg>/<version>`. The npm registry includes this endpoint and clients (vlt, possibly others) rely on it.

Source trace:

- `lb/main.ts:89-119`: Cloudflare Worker routing to R2.
- `api/src/s3_paths.rs:62-71`: only the full packument key is stored.

**Suggested fix**: implement the endpoint in the Worker by extracting the requested version from the cached full packument JSON, returning the per-version object. No R2 storage change required.

Related but distinct: pnpm had a different issue (pnpm/pnpm#10915) about tarball URL stripping.
```

## Environment

- vlt:
   1.0.0-rc.
  24 (latest as of 2026-04-04)
- Node.
  js:
   25.9.0
- Triggering package:
   `happy-opfs@2.0.2` ->
  `@jsr/std__path@^1.1.4`
- JSR source examined:
   `jsr-io/jsr` main branch,
   `lb/main.ts`
  and `api/src/s3_paths.rs`
- vlt source examined:
   `vltpkg/vltpkg` main branch,
  `src/package-info/src/index.ts`

## Upstream bug: build metadata in version spec

Filed from:
 Monochromatic monorepo,
 vlt 1.0.0-rc.
24,
 Node.
js v25.9.0,
 Linux x86_64
Re-verified at vltpkg/vltpkg commit `8ece488d` (tag `v1.0.0-rc.29-1`),
 Node v26.1.0,
 Linux x86_64

---

**Title:
** `failed to fetch manifest` when a dependency spec contains semver build metadata (`+suffix`)

### Description

`vlt install` fails with `ERESOLVE` / `failed to fetch manifest` when any package
in the dependency graph declares a dependency using an exact version that includes
semver build metadata (the `+<hash>` suffix).

The npm registry strips build metadata from version keys at publish time,
so the version `1.0.0-dev.1692+5c265bd4` is stored as `1.0.0-dev.1692`.
vlt constructs a per-version manifest URL that includes the `+` suffix verbatim,
which the registry does not recognize.

### Reproduction

#### Minimal reproduction

Any `package.json` that depends on a package whose transitive dependency graph
contains a build-metadata-tagged version spec:

```json
{
  "dependencies": {
    "@optique/run": "1.0.0-dev.1692"
  }
}
```

`@optique/run@1.0.0-dev.1692` declares `"@optique/core": "1.0.0-dev.1692+5c265bd4"`
in its `dependencies`.
 This triggers the bug.

```sh
mkdir /tmp/vlt-repro && cd /tmp/vlt-repro
echo '{"dependencies":{"@optique/run":"1.0.0-dev.1692"}}' > package.json
vlt install
```

#### Expected behavior

vlt strips build metadata from the version string before constructing the registry URL,
fetches `https://registry.npmjs.org/@optique/core/1.0.0-dev.1692` (200 OK),
and installs successfully.

#### Actual behavior

vlt constructs `https://registry.npmjs.org/@optique/core/1.0.0-dev.1692+5c265bd4`,
which returns 404,
 and throws:

```text
Error: failed to fetch manifest
  [cause]: {
    code: 'ERESOLVE',
    spec: Spec2 {
      type: 'registry',
      spec: '@optique/core@1.0.0-dev.1692+5c265bd4',
      registrySpec: '1.0.0-dev.1692+5c265bd4',
      overridden: false,
    },
    url: https://registry.npmjs.org/@optique/core/1.0.0-dev.1692+5c265bd4,
  }
```

#### Registry verification

```sh
## Works -- version without build metadata
curl -I "https://registry.npmjs.org/@optique/core/1.0.0-dev.1692"
## HTTP/2 200

## Fails -- version with build metadata (literal +)
curl -I "https://registry.npmjs.org/@optique/core/1.0.0-dev.1692+5c265bd4"
## HTTP/2 404

## Fails -- version with URL-encoded + (%2B)
curl -I "https://registry.npmjs.org/@optique/core/1.0.0-dev.1692%2B5c265bd4"
## HTTP/2 404
```

### Root cause analysis

The bug spans two files in the vltpkg/vltpkg monorepo:

#### 1. Spec parser preserves build metadata verbatim

`src/spec/src/browser.ts:644`:

```ts
this.registrySpec = this.bareSpec;
```

When `bareSpec` is `"1.0.0-dev.1692+5c265bd4"`,
 `registrySpec` retains the `+` suffix.

#### 2. Manifest URL includes unstripped build metadata

`src/package-info/src/index.ts:476-478` (rc.
29;
 was `:405-407` at rc.
24):

```ts
const version = hasLeadingRange ? registrySpec.slice(1,) : registrySpec;
const pakuURL = new URL(`${name}/${version}`, registry,);
```

`version` is `"1.0.0-dev.1692+5c265bd4"`.
 The `new URL()` call preserves `+` literally
in the path component.
 The npm registry does not serve versions at paths containing `+`.

#### 3. Single-version fast path is the only affected code path

`src/package-info/src/index.ts:714-716` (rc.
29;
 was `:601-603` at rc.
24):

```ts
const mani = spec.range?.isSingle
  ? await this.#registryManifestRequest(spec, options,)
  : pickManifest(await this.packument(f, options,), spec, options,);
```

Version specs with build metadata parse as `isSingle === true`,
 triggering the
per-version manifest fetch.
 Range specs (`>=1.0.0-dev.0`) go through the full
packument path,
 which fetches by package name only and is not affected.

### Suggested fix

**Option A:
 strip in `#registryManifestRequest` (minimal,
 targeted):
**

```ts
// src/package-info/src/index.ts
const version = hasLeadingRange ? registrySpec.slice(1,) : registrySpec;
const versionClean = version.replace(/\+.*$/, '',);
const pakuURL = new URL(`${name}/${versionClean}`, registry,);
```

**Option B:
 strip in Spec parser (comprehensive,
 prevents downstream issues):
**

```ts
// src/spec/src/browser.ts:644
this.registrySpec = this.bareSpec.replace(/\+.*$/, '',);
```

Option B is more comprehensive but changes the Spec's stored value,
which could affect lockfile serialization or other consumers that rely on
the full semver string.
 Option A is safer as a first fix.

Both options align with SemVer 2.0.0 spec items 10-11:

> "Build metadata MUST be ignored when determining version precedence.
> "

The `conventionalRegistryTarball` getter (which constructs the guessed tarball URL)
may also need patching for completeness.
 In the verified end-to-end repro,
Option A alone unblocks `vlt install`:
 the registry's manifest response carries
the build-metadata-stripped `dist.tarball`,
 so the guessed tarball URL is
not exercised on the success path.
 Patch it if you want belt-and-suspenders
coverage for registries that omit `dist.tarball`.

#### Verification of Option A

Cloned `vltpkg/vltpkg` at commit `8ece488d` (`v1.0.0-rc.29-1`) into a fresh
`mktemp -d` workspace,
 bootstrapped with the published rc.
29 (`vlt install`
in the monorepo),
 then ran the source-tree
`scripts/bins/vlt` against a one-line `package.json`
(`{"dependencies":{"@optique/run":"1.0.0-dev.1692"}}`).

Pre-patch,
 `vlt install` failed:

```text
Resolve Error: failed to fetch manifest
  While fetching: https://registry.npmjs.org/@optique/core/1.0.0-dev.1692+5c265bd4
  Response: { statusCode: 404, … }
```

Post-Option-A patch,
 `vlt install` exited 0,
 added both
`@optique/run@1.0.0-dev.1692` and `@optique/core@1.0.0-dev.1692`,
and wrote a valid `vlt-lock.json`.
 The fetched `@optique/core` tarball's
own `package.json` still records `"version": "1.0.0-dev.1692+5c265bd4"`,
which is preserved unchanged on disk.

The existing leading-prefix-strip test
(`src/package-info/test/index.ts:661`,
 "manifest strips leading semver
characters") covers the same URL-construction block this patch extends
and remains green under Option A;
 it exercises `=/^/~/v` prefixes and
the new `+`-suffix strip slots in alongside without altering that
behaviour.

`src/package-info` test suite under Node v26.1.0:
 141 of 148 pass
post-patch,
 identical pre-patch.
 The 7 failures (`cache hit - manifest
returned from cache` and 6 sibling subtests in `cache manifests`) hit
`The property 'options.recursive' is no longer supported. Received true`,
the Node v26 removal of `recursive` from `fs.cp` / `fs.cpSync`.
 They are
unrelated to manifest URL construction.

### Why this is rare

npm has stripped `+<build>` from version strings at publish time since 2014
(npm/npm#6379).
 The public npm registry has zero packages with build metadata
in version keys.
 The trigger requires a package that pins a dependency specifier
to an exact version with build metadata;
 dependency specifier strings are
not stripped by `npm publish`,
 only the `version` field is.
 The `@optique` project
does this because its release toolchain writes `+<git-sha>` into both fields.

npm,
 pnpm,
 and yarn all handle this case by stripping or ignoring build metadata
before making registry requests.

### Environment

Original report:

- vlt:
   1.0.0-rc.
  24
- Node.
  js:
   v25.9.0
- OS:
   Linux 6.17.7-ba29.
  fc43.
  x86_64 (Fedora/Bazzite)
- Registry:
   <https://registry.npmjs.org/>

Constraint-5 re-verification (2026-05-17):

- vlt:
   1.0.0-rc.
  29 + 1 commit,
   source tree at `8ece488d`
- Node.
  js:
   v26.1.0
- OS:
   Linux 6.19.14-ogc5.1.
  fc44.
  x86_64 (Fedora/Bazzite)
- Registry:
   <https://registry.npmjs.org/>
