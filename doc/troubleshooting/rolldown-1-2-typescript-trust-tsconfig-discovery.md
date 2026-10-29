# Rolldown 1.2.0 TypeScript trust builds inherit consumer tsconfig failures by default

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

`git cli-git trust --yes` fails while bundling a repository-root
`cli-git.config.ts` when the consumer repository's nearest `tsconfig.json`
extends a package that is unavailable to the trust build:

```text
Build failed with 2 errors:

[RESOLVE_ERROR] Could not resolve 'node:module' in \0rolldown/runtime.js
Tsconfig not found

[TSCONFIG_ERROR] Failed to load tsconfig '@monochromatic-dev/config-typescript/dom': Tsconfig not found
```

Both diagnostics have the same cause.
The `node:module` line does not mean that Node's built-in module is absent.
Rolldown's resolver failed while loading the consumer tsconfig before it could classify that built-in.

The failure occurs before cli-git can disclose or store the trusted bundle.
It makes a consumer's TypeScript project configuration an undeclared bootstrap
dependency of cli-git's config bundler.
GitHub issue [#393](https://github.com/Aquaticat/Monochromatic/issues/393)
tracked the cli-git correction separately from ignored-state worktree copying.

## Root cause

### Before resolution cli-git left Rolldown's tsconfig behavior implicit

At pre-fix commit `0db4382c1`,
`package/git-policy/cli/src/trust/typescript-builder.ts:348-356` set the consumer
repository as Rolldown's working directory but did not set `tsconfig`:

```ts
await using build = await rolldown({
  cwd: discovered.repositoryRoot,
  input: discovered.configPath,
  platform: 'node',
  treeshake: true,
  transform: {
    define: {
```

That omission was initially read as meaning that the bundler would transpile
TypeScript without consulting a project configuration.
Rolldown's public option contract disproves that reading.

### The coupling entered through the original tsdown builder

The bug was not introduced by the later Rolldown `1.2.0` lockfile update.
The original TypeScript trust implementation at commit `16e2de74d3851206c83d230629e774e33c6a1539`
set tsdown's `config: false` but omitted tsdown's separate `tsconfig` option at
`packages/cli/git/src/trust/typescript-builder.ts:414-423` in that commit:

```ts
const outputs = await build({
  config: false,
  cwd: discovered.repositoryRoot,
  entry: discovered.configPath,
  platform: 'node',
  format: 'esm',
  outDir: buildDirectory,
  write: false,
  clean: false,
  dts: false,
```

The accepted issue required tsdown config-file discovery to be disabled.
`config: false` met that requirement,
but it did not disable `tsconfig.json` discovery.
Tsdown `0.22.4` resolves an omitted `tsconfig` by searching upward from `cwd` at
`src/features/tsconfig.ts:24-27`:

```ts
if (tsconfig !== false) {
  if (tsconfig === true || tsconfig == null) {
    tsconfig = findTsconfig(cwd)
```

Tsdown then passed the discovered path into Rolldown at
`src/features/rolldown.ts:244-253`:

```ts
const inputOptions = await mergeUserOptions(
  {
    input: entry,
    cwd,
    external,
    resolve: {
      alias,
    },
    tsconfig: tsconfig || undefined,
```

The direct-Rolldown refactor in commit
`6cca70db8be4d28f9c9235a8bf2b299faf829393` preserved that accidental behavior by
omitting the top-level Rolldown option.
Rolldown `1.1.5`,
the version locked when that refactor landed,
already used `TsConfig::Auto(true)` at
`crates/rolldown_common/src/inner_bundler_options/types/tsconfig.rs:16-20`:

```rust
impl Default for TsConfig {
  fn default() -> Self {
    Self::Auto(true)
  }
}
```

The later `1.2.0` update exposed no new default in this path.
A direct runtime probe against both versions reproduced automatic discovery.
The actual discovery bug therefore dates to the first TypeScript trust builder and
remained hidden while builds ran in populated repositories.

### Rolldown defaults omitted `tsconfig` to discovery

The source trace uses Rolldown tag `v1.2.0`,
commit `03e1e3422cd85495c9863ff3bc3b24212d9f4be2`.
`packages/rolldown/src/options/input-options.ts:803-810` declares the default:

```ts
/**
 * Configures TypeScript configuration file resolution and usage.
 * {@include ./docs/tsconfig.md}
 * @default true
 */
tsconfig?: boolean | string;
```

`crates/rolldown_common/src/inner_bundler_options/types/tsconfig.rs:16-20`
turns an omitted binding value into enabled automatic discovery:

```rust
impl Default for TsConfig {
  fn default() -> Self {
    Self::Auto(true)
  }
}
```

The JavaScript binding forwards the value at
`packages/rolldown/src/utils/bindingify-input-options.ts:110-115`:

```ts
optimization: inputOptions.optimization,
context: inputOptions.context,
tsconfig: inputOptions.resolve?.tsconfigFilename ?? inputOptions.tsconfig,
```

`crates/rolldown_binding/src/utils/normalize_binding_options.rs:724-727`
converts a boolean to the Rust mode:

```rust
tsconfig: input_options.tsconfig.map(|v| match v {
  Either::A(v) => TsConfig::Auto(v),
  Either::B(s) => TsConfig::Manual(s.into()),
})
```

`crates/rolldown_resolver/src/resolver_config.rs:99-104` enables resolver discovery
when that boolean is true:

```rust
tsconfig: match tsconfig {
  TsConfig::Auto(v) => v.then_some(oxc_resolver::TsconfigDiscovery::Auto),
  TsConfig::Manual(config_file) => {
```

The trust entry is inside the consumer repository,
 so discovery finds the
consumer's `tsconfig.json` and follows its `extends` relation.
When that relation cannot resolve,
 Rolldown formats the observed diagnostic at
`crates/rolldown_error/src/build_diagnostic/events/tsconfig_error.rs:37-39`:

```rust
let tsconfig =
  tsconfig_path.map_or_else(String::new, |path| format!(" '{}'", opts.stabilize_path(path)));
format!("Failed to load tsconfig{tsconfig}: {reason}")
```

### The `node:module` diagnostic is collateral

Rolldown's generated Node runtime imports `node:module` at
`crates/rolldown/src/runtime/runtime-head-node.js:1`:

```js
import { createRequire } from 'node:module';
```

The resolver receives automatic tsconfig discovery and Node built-in handling in the
same option object at
`crates/rolldown_resolver/src/resolver_config.rs:97-101` and `:125-134`:

```rust
let default_options = OxcResolverOptions {
  cwd: Some(cwd.to_path_buf()),
  tsconfig: match tsconfig {
    TsConfig::Auto(v) => v.then_some(oxc_resolver::TsconfigDiscovery::Auto),
```

```rust
modules: resolve_options.modules.unwrap_or_else(|| vec!["node_modules".into()]),
// ...
builtin_modules: matches!(platform, Platform::Node),
```

Resolution runs through that combined resolver at
`crates/rolldown_resolver/src/resolver.rs:159-168`:

```rust
let mut resolution = if let Some(importer) = importer {
  if importer.is_absolute() {
    selected_resolver.resolve_file(importer, specifier)
  } else {
    selected_resolver.resolve_file(self.cwd.join(importer), specifier)
  }
} else {
  selected_resolver.resolve(self.cwd.as_path(), specifier)
};
```

When loading the inherited config fails first,
`crates/rolldown_error/src/utils/resolve_error.rs:17-20` renders the nested resolver reason:

```rust
ResolveError::NotFound(_) => "Cannot find module".to_string(),
ResolveError::TsconfigNotFound(_) => "Tsconfig not found".to_string(),
```

A Rolldown `1.1.5` probe containing only `export default 1` still emitted the
`node:module` resolver diagnostic alongside the tsconfig diagnostic.
That proves the generated runtime line is a second report of the same failed config load,
not another bootstrap dependency.

### Disabling discovery is a supported boundary

Rolldown does not need a consumer tsconfig to strip TypeScript syntax.
`crates/rolldown/src/utils/prepare_build_context.rs:364-376` gives
`TsConfig::Auto(false)` a transformer path without config resolution:

```rust
match tsconfig {
  TsConfig::Manual(_) | TsConfig::Auto(true) => Box::new(TransformOptions::new_raw(
    RawTransformOptions::new(
      raw_transform_options,
      Arc::new(resolver.clone_default_resolver()),
    ),
    target,
    jsx_preset,
  )),
  TsConfig::Auto(false) => Box::new(TransformOptions::new(
    merge_transform_options_with_tsconfig(raw_transform_options, None, &mut warnings)?,
    target,
    jsx_preset,
  )),
}
```

Cli-git config trust already requires ordinary relative or package imports to
resolve through the bundle graph.
It does not promise to inherit consumer path aliases,
JSX settings,
or decorator semantics from an ambient TypeScript project.
The accidental auto-discovery therefore belongs at cli-git's Rolldown call,
not in the consumer dependency contract.

### Existing pre-fix tests omitted the adversarial state

The builder unit fixture creates only `cli-git.config.ts` and its private output
directory at
`package/git-policy/cli/src/trust/typescript-builder.unit.test.ts:41-51`:

```ts
const repository = await realpath(
  await mkdtemp(join(tmpdir(), 'cli-git-typescript-',),),
);
const configPath = join(repository, 'cli-git.config.ts',);
await writeFile(configPath, source,);
const buildDirectory = join(repository, '.private-build',);
await mkdir(buildDirectory,);
```

Tests that need cli-git's authoring package explicitly add consumer-local package
state at `package/git-policy/cli/src/trust/typescript-builder.unit.test.ts:147-151`:

```ts
const packageDirectory = await realpath(join(import.meta.dirname, '../..',),);
const fixtureScope = join(fixture.repository, 'node_modules', '@monochromatic-dev',);
await mkdir(fixtureScope, { recursive: true, },);
await symlink(packageDirectory, join(fixtureScope, 'git-policy-cli',), 'dir',);
```

At pre-fix commit `0db4382c1`,
the packed fixture installed cli-git under `/work` at
`package/git-policy/cli/src/trust/fixture/built-trust-consumer.ts:49-65`,
then created the TypeScript repository at `/work/typescript` in
`package/git-policy/cli/src/trust/fixture/built-typescript-consumer.ts:31-42`:

```ts
await execute({
  command: 'npm',
  args: ['install', '--ignore-scripts', '/fixture/cli.tgz',],
  cwd: '/work',
},);
```

```ts
const repository = '/work/typescript';
await mkdir(repository,);
```

That repository has no tsconfig,
and Node package lookup can walk to `/work/node_modules`.
It exercises clean absence,
not a present tsconfig whose `extends` target is unavailable.
A no-tsconfig fixture alone therefore cannot regress this bug:
removing `tsconfig: false` would still let Rolldown discover nothing and succeed.

### A package-resolution failure is masked behind the tsconfig failure

The root repository config imports the source export directly as
`@monochromatic-dev/git-policy-cli/ts`.
At pre-fix commit `0db4382c1`,
the resolver hook special-cased only the package root and asked Rolldown to
resolve the source export from the consumer importer at
`package/git-policy/cli/src/trust/typescript-builder.ts:143-155`:

```ts
if (source === CLI_GIT_PACKAGE_IMPORT) {
  bareImports.add(source,);
  const resolved = await this.resolve(
    CLI_GIT_SOURCE_IMPORT,
    importer,
    { skipSelf: true, },
  );
```

A direct `/ts` import reaches the generic bare-import branch at
`package/git-policy/cli/src/trust/typescript-builder.ts:183-186`:

```ts
if (isAbsolute(source,))
  throw new TypeScriptBuildError(`Absolute TypeScript import is outside tracked graph: ${source}`,);
bareImports.add(source,);
return null;
```

After setting `tsconfig: false`,
a clean linked worktree outside the wrapper installation's ancestor path cannot
resolve either form through consumer `node_modules`.
A direct probe then left
`@monochromatic-dev/git-policy-cli/ts` unresolved.

The same generic branch affects other bare imports.
For example,
`valibot` is a packed runtime dependency at
`package/git-policy/cli/package.json:49-55`:

```jsonc
"dependencies": {
  "acorn": "catalog:",
  "nano-spawn": "catalog:",
  "rolldown": "catalog:",
  "type-fest": "catalog:",
  "typescript": "catalog:",
  "valibot": "catalog:"
}
```

A config or tracked relative module importing `valibot` still cannot find it when
the wrapper installation is outside consumer ancestry.
The complete package-resolution design must cover artifact-provided runtime
packages as well as cli-git's own specifiers.

The pre-fix packed package already contained the needed source export at
`package/git-policy/cli/package.json:10-16` and included its source tree at `:20-22`:

```jsonc
"exports": {
  ".": {
    "types": "./dist/final/node/index.d.mts",
    "import": "./dist/final/node/index.mjs"
  },
  "./ts": "./src/index.ts"
}
```

```jsonc
"files": [
  "dist/final/node",
  "src",
```


The pre-fix installed source export was not a clean production anchor.
`package/git-policy/cli/src/index.ts:7-9` statically imports executable startup before
exposing authoring declarations:

```ts
import { runCliGit, } from './bin.ts';
```

A disposable packed-artifact probe used
`createRequire(installedEntryUrl).resolve(CLI_GIT_SOURCE_IMPORT)` to anchor that
current source tree.
With `tsconfig: false`,
a poison consumer tsconfig,
a relative TypeScript import,
and no consumer `node_modules`,
Rolldown generated a final self-contained MJS chunk and the loaded config retained
the relative import's value.
However,
resolution also emitted warnings from dead executable modules before tree shaking,
including:

```text
Could not resolve '@monochromatic-dev/caught-value-core/ts' in .../src/bin.ts
Could not resolve '@monochromatic-dev/logger-core/ts' in .../src/bin.ts
```

Those workspace packages are not installed runtime dependencies of the packed
artifact.
Final-output validation happened to remove the dead executable graph,
but relying on unresolved warnings is not an acceptable source interface.
The anchor probe proves the package-location mechanism only.
It did not prove pre-fix `src/index.ts` was a clean authoring entry.

## Verification

The installed package reported Rolldown `1.2.0`.
The source checkout matched tag `v1.2.0` and commit
`03e1e3422cd85495c9863ff3bc3b24212d9f4be2`.
The historical comparison used Rolldown tag `v1.1.5`,
commit `f09947ab017d6df74299f691853dcfc4f4f0f86e`,
and tsdown tag `v0.22.4`.

### End-user reproduction

At repository commit `0db4382c1`,
the installed cli-git `0.0.1` wrapper created a detached linked worktree with
`--no-worktree-copy`.
The fixture had `cli-git.config.ts` and the tracked root `tsconfig.json`,
but no `node_modules`.
Both consecutive runs of this command exited `2` with the exact two-error symptom:

```console
git -C /tmp/agent/mono-393-repro-20260719 cli-git trust --yes
```

Running the same public Rolldown API catalog against `1.1.5` also failed with an
omitted `tsconfig` and succeeded with `tsconfig: false`.
This rules out the later `1.2.0` lockfile update as the introducing event.
The linked worktree was removed after the repeated reproduction.

### Public API catalog

This Node harness creates independent consumer directories and calls Rolldown's
public API for each case:

```ts
// /tmp/rolldown-tsconfig-catalog/reproduce.mts
import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
import { rolldown, } from 'rolldown';

const root = await mkdtemp(join(tmpdir(), 'rolldown-tsconfig-catalog-',),);
const cases = [
  { name: 'absent-default', config: undefined, setting: undefined, },
  { name: 'valid-default', config: {}, setting: undefined, },
  { name: 'missing-extends-default', config: { extends: '@missing/config/dom', }, setting: undefined, },
  { name: 'missing-extends-true', config: { extends: '@missing/config/dom', }, setting: true, },
  { name: 'missing-extends-false', config: { extends: '@missing/config/dom', }, setting: false, },
] as const;

for (const item of cases) {
  const directory = join(root, item.name,);
  await mkdir(directory,);
  await writeFile(join(directory, 'entry.ts',), 'export default 1\n',);
  if (item.config !== undefined)
    await writeFile(join(directory, 'tsconfig.json',), JSON.stringify(item.config,),);
  try {
    await using build = await rolldown({
      cwd: directory,
      input: join(directory, 'entry.ts',),
      ...(item.setting === undefined ? {} : { tsconfig: item.setting, }),
    },);
    await build.generate({ format: 'esm', },);
    console.log(`${item.name}: success`,);
  }
  catch (error: unknown) {
    console.log(`${item.name}: ${error instanceof Error ? error.message : String(error,)}`,);
  }
}
await rm(root, { recursive: true, force: true, },);
```

The command was:

```console
node --experimental-strip-types /tmp/rolldown-tsconfig-catalog/reproduce.mts
```

### Clean catalog

- Omitted `tsconfig` with no discovered `tsconfig.json`:
   success.
- Omitted `tsconfig` with an empty valid `tsconfig.json`:
   success.
- Explicit `tsconfig: false` with an unresolvable `extends`:
   success.

### Failing catalog

- Omitted `tsconfig` with `extends: "@missing/config/dom"`:
  `[TSCONFIG_ERROR] Failed to load tsconfig '@missing/config/dom': Tsconfig not found`.
- Explicit `tsconfig: true` with the same `extends`:
   the same diagnostic.

## Resolution

Cli-git now disables ambient project discovery explicitly at
`package/git-policy/cli/src/trust/typescript-builder.ts:132-145`:

```ts
await using build = await rolldown({
  cwd: discovered.repositoryRoot,
  input: discovered.configPath,
  platform: 'node',
  tsconfig: false,
```

The package `/ts` export now names the source-only module at
`package/git-policy/cli/package.json:10-16`:

```json
"exports": {
  ".": {
    "types": "./dist/final/node/index.d.mts",
    "import": "./dist/final/node/index.mjs"
  },
  "./ts": "./src/authoring.ts"
}
```

`package/git-policy/cli/src/trust/typescript-source-capture.ts:270-294`
anchors cli-git's own imports to that installed source,
keeps consumer resolution first for other packages,
falls back only to package names in the packed runtime manifest,
and rejects any remaining unresolved package before output generation:

```ts
const packageName = artifactPackageName(source,);
if (((typeof packageName) !== 'symbol')
  && (packageName === CLI_GIT_PACKAGE_IMPORT)
  && ((source === CLI_GIT_PACKAGE_IMPORT) || (source === CLI_GIT_SOURCE_IMPORT)))
  return installedAuthoringSourcePath;
const consumerResolved = await this.resolve(
  source,
  importer,
  { skipSelf: true, },
);
if ((consumerResolved !== null)
  && ((consumerResolved.external === undefined) || (consumerResolved.external === false)))
  return consumerResolved;
if (((typeof packageName) !== 'symbol') && ARTIFACT_RUNTIME_PACKAGE_NAMES.has(packageName,))
  return artifactImportPath(source,);
throw new TypeScriptBuildError(`Bare TypeScript package import did not resolve into bundle: ${source}`,);
```

The unavailable scoped-subpath regression at
`package/git-policy/cli/src/trust/typescript-builder.unit.test.ts:212-224`
asserts that rejection remains a `TypeScriptBuildError` after Rolldown wraps plugin failures.

The packed wrapper fixture installs under `/opt/cli-git`,
outside `/work` consumer ancestry,
at `package/git-policy/cli/src/trust/fixture/built-trust-consumer.ts:47-75`.
Its poison repository has no initial entries,
imports `/ts`,
imports `valibot` and `acorn/package.json` from a tracked relative module,
and carries an unresolvable ambient config at
`package/git-policy/cli/src/trust/fixture/built-typescript-consumer.ts:35-103`.
The separate clean fixture carries no tsconfig or consumer `node_modules`.

The least-trusting command completed with `built-trust-consumer-ok`:

```console
mise run //package/git-policy/cli:test:built:trust
```

That run also retained strict relative-source invalidation,
relaxed rebuild,
package warnings,
stored execution,
and the complete packed trust lifecycle.

## Verified workarounds

### Disable discovery and introduce a dedicated authoring source entry

The complete internal correction has four parts.
First,
set the explicit supported option at cli-git's Rolldown seam:

```ts
await using build = await rolldown({
  cwd: discovered.repositoryRoot,
  input: discovered.configPath,
  tsconfig: false,
  platform: 'node',
```

Second,
move the source-level authoring exports behind a dedicated module that does not
import `bin.ts` or executable-only workspace modules.
Point package export `/ts` at that module,
and let the executable `src/index.ts` re-export it before its direct-entry branch.
This places the source-authoring interface at a seam that excludes wrapper startup.

Third,
resolve both `@monochromatic-dev/git-policy-cli` and `/ts` to that dedicated module
from the running installed artifact.
The location mechanism is:

```ts
const packageSourcePath = createRequire(import.meta.url)
  .resolve('@monochromatic-dev/git-policy-cli/ts');
```

Compute the path only inside the TypeScript build path and pass it into the capture
plugin.
That keeps normal wrapper startup lazy.

Fourth,
resolve every other bare import from the consumer first.
When consumer resolution returns no module,
derive the package name from the bare specifier:
the first segment for an unscoped package and the first two segments for a scoped
package.
Fall back to ESM resolution from the installed artifact only when that package name
is declared in cli-git's packed runtime dependency manifest.
This supports package subpaths without treating the complete subpath as a manifest
key.
Apply the rule to imports from the root config and every tracked relative source.
Keep artifact-resolved packages outside exact-source invalidation and retain the
existing bare-package disclosure.
This preserves consumer-selected package versions,
makes the promised artifact dependencies available,
and avoids exposing unrelated packages from the installation prefix.

The Rolldown catalog proves `tsconfig: false` succeeds with an unresolvable
`extends`.
The initial packed-artifact probe proved the location mechanism and final-output behavior,
but its unresolved warnings disqualified pre-fix `src/index.ts` as the final source
entry.
The resolution section records successful packed-wrapper verification of the dedicated entry.

The tradeoff is intentional:
trusted cli-git config cannot rely on ambient `paths`,
JSX,
decorator,
or class-field settings from the consumer tsconfig.
Those semantics would need explicit cli-git-owned transform options instead of an
ambient project file.

The issue #393 resolution uses two packed consumer cases:

- no tsconfig,
  proving the clean baseline;
- a poison tsconfig extending an unavailable package,
  proving the builder does not consult it.

Install the tarball in a prefix outside both consumer repositories,
put only that prefix's bin on `PATH`,
and leave both repositories without
`node_modules`.
Exercise the actual `git cli-git trust --yes` command for package-root and `/ts`
imports.
Also import an artifact runtime dependency such as `valibot` from a tracked relative
module,
and assert that no unresolved-import warning appears.
Coverage for fallback eligibility includes unscoped and scoped package roots
plus both subpath forms.
The poison case is the red-before-fix regression;
the no-tsconfig case is not.

### Make the consumer's extended package available

Installing every package named by the consumer tsconfig lets Rolldown complete
auto-discovery.
The tradeoff is an inverted bootstrap contract:
 cli-git trust then depends on
project type-check dependencies that the trusted config does not import.
It also leaves trust behavior sensitive to unrelated tsconfig edits.

### Use a self-contained MJS config

A repository can bundle its config independently and trust
`cli-git.config.mjs`,
 which takes precedence over the TypeScript form.
The tradeoff is that the repository must own and keep another generated artifact
synchronized.

## What does not work

### Treat omitted `tsconfig` as disabled

Rolldown `1.1.5` and `1.2.0` both default omission to automatic discovery.
The type declaration,
Rust default,
and reproduced failures reject this reading.
The `1.2.0` dependency update did not introduce the bug.

### Add only `tsconfig: false`

This removes the first error,
but a linked worktree outside the wrapper installation path then exposes the
unresolved cli-git authoring import.
The source export must also be anchored to the installed artifact.

### Anchor the current `src/index.ts` without splitting authoring from execution

The packed probe produced a valid final chunk only after warning about unresolved
workspace imports reached through `bin.ts`.
Tree shaking dead code after failed resolution is not a clean package contract and
may stop working if an executable module gains a retained side effect.
Use a dedicated authoring source entry instead.

### Anchor only cli-git's own package specifiers

This leaves direct bare imports such as `valibot` dependent on consumer
`node_modules`,
even when the wrapper artifact installed that runtime package.
Use consumer-first resolution with a packed-runtime-dependency fallback for every
tracked source.

### Use only a no-tsconfig packed fixture

Rolldown succeeds when automatic discovery finds no config,
so this fixture passes with and without the required opt-out.
It is a useful baseline but not a regression test for ambient-config isolation.

### Pin or downgrade Rolldown

A direct `1.1.5` probe reproduced the same discovery and `node:module` errors.
Pinning that version would preserve the defect while hiding the ownership error.

### Change Rolldown's `cwd` to cli-git's package directory

That can avoid the consumer tsconfig by changing discovery location,
 but it also
changes the base for resolution and violates the trust build's consumer-root
module graph.
It fixes the symptom by moving the build out of its intended repository.

### Report the diagnostic as a missing cli-git runtime dependency

The named package comes from the consumer's ambient tsconfig `extends`,
 not from
the trusted config's import graph.
Adding that package to cli-git would solve only one repository-specific spelling
and contradict the no-repository-specific-configuration requirement.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry names Rolldown or this tsconfig behavior.
Open and closed Rolldown issue searches for `tsconfig extends Tsconfig not found`
found no matching defect.
The merged [Rolldown PR #7817](https://github.com/rolldown/rolldown/pull/7817),
`feat(tsconfig)!: enable auto-discovery by default`,
 is the deciding existing
thread.
Its migration guide explicitly says consumers that need the previous no-config
behavior must set `tsconfig: false`.

1. **Is it really upstream's fault?
   ** No.
   Rolldown documents automatic discovery as the default and provides the exact
   opt-out cli-git needs.
2. **Can upstream fix it?
   ** No upstream defect was identified.
   Changing the default again would reverse PR #7817 rather than correct this
   consumer call.
3. **Are they supporting this use case?
   ** Yes.
   The public `boolean | string` option and PR #7817 migration guide support
   disabling project config explicitly.
4. **Would the repository welcome our contribution?
   ** Not applicable because
   no change is requested.
   `CONTRIBUTING.md`,
    `.github/ISSUE_TEMPLATE/`,
    and
   `.github/PULL_REQUEST_TEMPLATE.md` contain no AI-assistance ban,
    but that does
   not turn expected behavior into a reportable defect.
5. **Will they likely fix it?
   ** No fix should be requested.
   The merged default-change PR explicitly chose this behavior.
6. **Have we prototyped a compatible minimal fix?
   ** Yes,
    at the consumer
   boundary.
   The `tsconfig: false` catalog case succeeds without modifying Rolldown.

There is nothing additive to post on PR #7817 and no upstream issue to file.
The actionable artifact is internal issue #393 plus the verified consumer-side
option.
