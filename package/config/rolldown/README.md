# @monochromatic-dev/config-rolldown

Shared raw-rolldown build configuration for Monochromatic packages.
Successor to `@monochromatic-dev/config-tsdown`;
see `doc/planning/tsdown-removal.md` for the migration record
and `doc/philosophy/tool-choices.md` for the bundler decision.

## Flavors

- `./.ts`:
  neutral (browser-compatible) library builds,
  declarations via `rolldown-plugin-dts` with the `oxc` generator,
  output `dist/final/neutral/[name].mjs`.
- `./.node.ts`:
  Node library,
   CLI,
   and plugin builds,
  same declaration setup,
  output `dist/final/node/[name].mjs`.
  `perEntryNodeConfig` builds each input as its own self-contained bundle
  so committed and published bundles keep stable names and no shared chunks.
- `./.client.ts`:
  self-contained browser scripts,
  no declarations,
  output `dist/client/[name].js`.

AP5:
 Client-side bundling packages:
 add `rolldown.client.config.ts` extending `@monochromatic-dev/config-rolldown/.client.ts`,
 a `build:js:client` task,
 and this package as a devDependency.

## Behavior notes

- Externalization reads the consuming package's `package.json` at build time:
  declared dependencies and peer dependencies stay external
  unless a bundle pattern forces them inline;
  undeclared bare imports (transitives of inlined workspace source) bundle by
  omission so artifacts stay self-contained outside the monorepo.
- Bin-targeted entry chunks of Node builds get `#!/usr/bin/env node`
  injected at byte 0 (`bin-shebang.ts`) when the emitted chunk has no shebang,
  so a built CLI can never fall through to `/bin/sh`;
  a shebang already present is a user error and is left untouched.
  A `bin` entry pointing at output a build does not emit is ignored here,
  typo detection is not this plugin's job.
- A family library build (`familyLibraryNodeConfig`) refuses to run
  under a bundler other than the version its checkout installs
  (`checkout-bundler.ts`):
  the build runs whichever `rolldown` comes first on `PATH`,
  and another checkout's version emits different bytes,
  under a pipeline digest that then names a build this checkout cannot reproduce.
  Family configs list the library build first,
  so the refusal stops the runner builds listed after it.
- Targets come from Browserslist through `browserslistTargets`,
  formatted as rolldown engine strings
  (rolldown rejects raw Browserslist queries).
- Raw rolldown's `output.cleanDir` is per-config and not watch-safe;
  per-entry configs sharing an out dir keep it off
  and rely on the owning mise task's pre-clean step.
