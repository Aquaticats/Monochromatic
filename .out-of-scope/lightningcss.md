# Why not LightningCSS

This project does not use LightningCSS, and does not file or track LightningCSS
upstream bugs as GitHub issues.

## Why this is out of scope

The CSS build (`packages/build-tool/css/`,
`@monochromatic-dev/css-cli`) ran a LightningCSS-based pipeline for a
while, then removed LightningCSS entirely in favour of a pure-JS PostCSS
pipeline. LightningCSS no longer appears in any `package.json` or in the
lockfile. Two LightningCSS limitations forced the move:

- `customAtRules` cannot coexist with `var()`. Configuring `@mixin`/`@apply`
  as custom at-rules makes the parser fail on any unrelated declaration that
  uses `var(--x)`, with `failed to deserialize ... Specifier, found ()`. var()
  is foundational to the design system, so this is a hard blocker. Tracked
  upstream at parcel-bundler/lightningcss#1081 (open, no fix).
- `bundle()` has no `node_modules` resolution. It does relative-path
  resolution only: no `node_modules` walk, no `exports`-field support, no
  workspace links. A cross-package monorepo `@import` cannot resolve.

The first workaround stacked PostCSS (for mixins) and oxc-resolver (for import
resolution) on top of LightningCSS. That stack carried its own failures (a
pnpm-isolated-mode symlink loss for oxc-resolver) and ran three tools where one
would do. The pure-JS PostCSS pipeline replaced all of it: a custom PostCSS
plugin handles `@import` inlining with an in-house `node_modules` resolver, and
PostCSS handles `@mixin`/`@apply`. No native binary, no LightningCSS, no
oxc-resolver in the CSS build.

With LightningCSS removed, its bugs no longer affect this workspace. A local
GitHub issue tracking an upstream LightningCSS fix has nothing to act on: even
if #1081 were fixed, re-adopting LightningCSS would be a fresh dependency
decision, not an automatic switch. A perpetual "watch upstream" tracker is
therefore issue clutter.

## What we use instead

- The in-house `@monochromatic-dev/css-cli` package: pure-JS PostCSS
  pipeline, no native binary. `docs/troubleshooting/css-tooling.md` holds the
  full history of why each candidate (LightningCSS, oxc-resolver under pnpm
  isolated mode, Vite/Astro) was tried and dropped.
- For mixin processing: PostCSS plugins on the source CSS
  (`packages/build-tool/css/src/mixin.ts`).
- For `@import` resolution: the in-house resolver in
  `packages/build-tool/css/src/package-resolver.ts`.

## Examples of this category

The following local tracking issue was closed as out-of-scope per this policy:

- `#147` Track LightningCSS #1081 (customAtRules broken with var()) for
  potential CSS-tooling re-evaluation. Mooted by removing LightningCSS from the
  CSS build entirely.

## Re-evaluation

Revisit only with a concrete reason to add LightningCSS back as a dependency
(for example a measured performance need the pure-JS pipeline cannot meet), and
only after a fresh dependency-source audit. Tool selection lives in
`docs/philosophy/tool-choices.md`; audit records live under `docs/audit/`. An
upstream LightningCSS fix alone is not the trigger.
