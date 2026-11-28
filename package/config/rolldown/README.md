# @monochromatic-dev/config-rolldown

Shared raw-rolldown build configuration for Monochromatic packages.
Successor to `@monochromatic-dev/config-tsdown`;
see `doc/planning/tsdown-removal.md` for the migration record
and `doc/philosophy/tool-choices.md` for the bundler decision.

## Pi dependency trust

PJJ:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

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
- A Node flavor build never deletes a file of its output directory before it has the file to
  put there.
  `nodeConfig` hands the bundler a staging directory beside the output directory
  (`<parent>/.<name>.building`) and `atomicOutputDirectoryPlugin` renames one staged file into
  place per emitted name,
  shared chunks and assets first and entry chunks last,
  in the ordered `post` group of `writeBundle`.
  So every path under the output directory resolves to one whole file at every instant of a
  build:
  the previous build's,
  or this one's.
  A build that dies before its write phase leaves the output directory untouched and its
  staging directory behind for the next build of the same package to clear.
  Files this build did not emit are removed once the new ones are in place,
  which is the end state `output.cleanDir` produced;
  a consumer that sets `cleanDir: false` keeps that removal off,
  which is how a second config writing into a shared output directory leaves its sibling's
  files alone.
- Raw rolldown's `output.cleanDir` is per-config and not watch-safe;
  per-entry configs sharing an out dir keep it off
  and rely on the owning mise task's pre-clean step,
  and their files are still written in place,
  so a reader can see one of those bundles half written while its build runs.
- The client and neutral flavors' default configs still set `output.cleanDir: true` and write
  their files in place;
  no command the repository installs runs from `dist/client` or `dist/final/neutral`.
