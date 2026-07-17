# Rolldown 1.2.0 TypeScript trust builds inherit consumer tsconfig failures by default

## Symptom

`git cli-git trust --yes` fails while bundling a repository-root
`cli-git.config.ts` when the consumer repository's nearest `tsconfig.json`
extends a package that is unavailable to the trust build:

```text
Failed to load tsconfig '@monochromatic-dev/config-typescript/dom': Tsconfig not found
```

The failure occurs before cli-git can disclose or store the trusted bundle.
It makes a consumer's TypeScript project configuration an undeclared bootstrap
dependency of cli-git's config bundler.
GitHub issue [#393](https://github.com/Aquaticat/Monochromatic/issues/393)
tracks the cli-git correction separately from ignored-state worktree copying.

## Root cause

### Cli-git leaves Rolldown's tsconfig behavior implicit

`package/git-policy/cli/src/trust/typescript-builder.ts:348-356` sets the consumer
repository as Rolldown's working directory but does not set `tsconfig`:

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
Rolldown 1.2.0 disproves that reading.

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
 or decorator
semantics from an ambient TypeScript project.
The accidental auto-discovery therefore belongs at cli-git's Rolldown call,
not in the consumer dependency contract.

## Verification

The installed package reported Rolldown `1.2.0`.
The source checkout matched tag `v1.2.0` and commit
`03e1e3422cd85495c9863ff3bc3b24212d9f4be2`.

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

## Verified workarounds

### Disable ambient tsconfig discovery at cli-git's bundler boundary

Add the explicit supported option to the call in
`package/git-policy/cli/src/trust/typescript-builder.ts`:

```ts
await using build = await rolldown({
  cwd: discovered.repositoryRoot,
  input: discovered.configPath,
  tsconfig: false,
  platform: 'node',
```

The verification catalog proves this setting succeeds while the discovered
config still has an unresolvable `extends`.
The tradeoff is intentional:
 trusted cli-git config cannot rely on ambient
`paths`,
 JSX,
 decorator,
 or class-field settings from the consumer tsconfig.
Issue #393 must add a built packed-consumer regression before landing this fix.

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

Rolldown 1.2.0 changed the default to `true`.
The type declaration,
 Rust default,
 and reproduced default failure all reject
this reading.

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
