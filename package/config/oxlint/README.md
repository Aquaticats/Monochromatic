# @monochromatic-dev/config-oxlint

Shared oxlint configuration for Monochromatic repositories,
exported as a typed `OxlintConfig` object and consumed by the root `oxlint.config.ts`.

## Usage

The root `oxlint.config.ts` imports and re-exports the shared config:

```typescript
// oxlint.config.ts (monorepo root)
import base from '@monochromatic-dev/config-oxlint/ts';
import { defineConfig, } from 'oxlint';

export default defineConfig({ ...base, },);
```

Uses spread instead of `extends` because `extends` only merges rules:
top-level fields like `categories`,
 `env`,
 `ignorePatterns`,
 `overrides`,
and `plugins` are not inherited through `extends`.

The package default export resolves to the prebuilt,
 self-contained
`dist/final/node/index.mjs`,
 which bundles the config and references three
co-located plugin sidecars (`plugin-*.mjs`) by relative `file://` URL.
It is publishing output for consumers outside this repository.
A repository root lints through the `/ts` source subpath (`/ts/*` for individual
files) instead,
 which needs no build before linting and keeps a lint run reading live
plugin source (rule ST3, `doc/decision/workspace-ts-source-imports.md`).

## Structure

```text
src/
  config-base.ts    -- config shape minus jsPlugins, shared by both entries
  index.ts          -- dev entry (./ts): spreads base, resolves plugins to /ts source
  index.node.ts     -- built entry (.): spreads base, points jsPlugins at sidecars
  plugin-tsdoc.ts                 -- sidecar entry, bundles tsdoc plugin /ts source
  plugin-no-restricted-syntax.ts          -- sidecar entry, bundles syntax plugin source
  plugin-prefer-readonly-parameter-type.ts -- sidecar entry, bundles semantic readonly plugin source
  plugin-stylistic.ts                     -- sidecar entry, bundles stylistic plugin source
  overrides.ts      -- file-pattern overrides (.d.ts, .test.ts, .config.*, etc.)
  rules/
    tsdoc.ts        -- tsdoc jsPlugin rule severity configuration
    correctness.ts  -- correctness, typescript, jest suppression, perf rules
    restriction.ts  -- restriction rules and no-disable enforcement
    style.ts        -- style and pedantic rules
```

## jsPlugin resolution

The built entry (`index.node.ts` -> `dist/final/node/index.mjs`) points `jsPlugins`
at co-located sidecar `.mjs` files via relative `file://` URLs,
 so no plugin
resolution happens at lint time (the optimization in issue #238).

The development entry (`index.ts`,
 the `./ts` export) instead resolves each plugin's
`/ts` source subpath with `import.meta.resolve()` at config evaluation time,
 because
oxlint's Rust resolver doesn't understand pnpm workspace package names (Node.
js
handles workspace resolution).
 This lets development linting track live plugin source
without a rebuild.

## Type-aware options

The shared config enables `options.typeAware` and `options.typeCheck`.
Consumers inherit both through the root config import,
so mise tasks do not need equivalent CLI flags.

## Related packages

- **@monochromatic-dev/oxlint-plugin-tsdoc**:
   jsPlugin providing TSDoc validation rules
- **@monochromatic-dev/oxlint-plugin-no-restricted-syntax**:
   jsPlugin for banned syntax patterns
- **@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type**:
   jsPlugin for semantic readonly types and mutation contracts
- **@monochromatic-dev/oxlint-plugin-stylistic**:
   jsPlugin for TSDoc asterisk-prefix,
   one-item-per-line,
   semicolon,
   and expression-structure formatting
- **@monochromatic-dev/oxlint-plugin-shared**:
   shipped runtime helpers imported by plugin rules
- **@monochromatic-dev/oxlint-plugin-test-support**:
   private fixture-test helpers for plugin unit tests

The first four packages are JS plugins (`jsPlugins`),
not config packages.
They implement rule logic;
this package configures rule severity and options.
Those four plugin packages are declared as dependencies of this package:
the development entry resolves their `/ts` source via `import.meta.resolve()`,
and the build bundles that source into the sidecars.
The shared package is a runtime dependency of plugin packages;
the test-support package is dev-only.
