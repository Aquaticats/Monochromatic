# Use zeptomatch for pi model-scope globs

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Context

`package/pi-shared/model-selection/src/scope-patterns.ts` resolves pi model-scope glob patterns against in-memory
model slugs and bare model ids.
 Advisor consumes that shared package through
`package/pi-plugin/advisor/package.json`,
 which previously also declared an unused direct matcher dependency.

The matcher must handle string matching only.
 It must not traverse the filesystem or add directory-expansion
semantics to model selection.

## Decision

Use `zeptomatch` directly in `package/pi-shared/model-selection` and remove direct `minimatch` declarations from
pi package manifests.

`zeptomatch` fits because it is an in-memory glob matcher with TypeScript declarations,
 MIT licensing,
 and a catalogued
transitive presence through `tiny-readdir-glob`.
 Source audit on 2026-06-04:

- `/tmp/agent/zeptomatch-20260604/package.json` declares runtime dependencies on `grammex` and `graphmatch`.
- `/tmp/agent/zeptomatch-20260604/src/index.ts` exposes `zeptomatch(glob, path, options)` and
  `zeptomatch.compile(glob, options)`.
- `/tmp/agent/zeptomatch-20260604/src/types.ts` shows the options surface is only `{ partial?: boolean }`.
- `/tmp/agent/zeptomatch-20260604/readme.md` documents star,
   globstar,
   question mark,
   character classes,
   braces,
  negation,
   separator normalization,
   and no special dot-file handling.

Because `zeptomatch` has no `nocase` option,
 the shared model-selection code lowercases both the pattern and candidate
before matching.
 The scoped model surface is intentionally case-insensitive for provider/model slug matching.

## Alternatives

1. **Zeptomatch,
    chosen.**

   - Pros:
      direct string matcher,
      shipped types,
      already present through catalogued packages,
      no filesystem traversal.
   - Cons:
      no `nocase` option,
      so callers must normalize case explicitly;
      no extglobs or POSIX classes.

2. **Picomatch.**

   - Pros:
      broad minimatch-like glob syntax and a `nocase` option.
   - Cons:
      no bundled TypeScript declarations in the installed v4 package,
      so this repo would need another ambient shim.

3. **In-repo matcher.**

   - Pros:
      no external runtime dependency.
   - Cons:
      the repo would own glob semantics,
      escaping rules,
      character classes,
      and future edge-case tests.

4. **tiny-readdir-glob.**

   - Pros:
      already catalogued and useful for filesystem glob expansion.
   - Cons:
      wrong abstraction for model selection because it traverses directories through `tiny-readdir` before matching.

Ranking:
 1 > 2 > 3 > 4.
 Zeptomatch beats Picomatch because shipped types avoid another ambient module.
 Picomatch beats
an in-repo matcher because existing glob implementations carry more edge-case coverage.
 An in-repo matcher beats
`tiny-readdir-glob` because it would at least keep model selection in-memory.

## Consequences

- `package/pi-shared/model-selection/package.json` declares `zeptomatch` from the pnpm catalog.
- `package/pi-plugin/advisor/package.json` no longer declares a matcher dependency directly.
- Scope-pattern tests cover case-insensitive canonical-slug and bare-id glob matching.
- Scope-pattern tests cover brace,
   character-class,
   globstar,
   slash-boundary,
   and escaped-star glob syntax.
- Pi extension builds that bundle `@monochromatic-dev/pi-shared-model-selection` also bundle `zeptomatch`,
   `grammex`,
  and `graphmatch`.
   This matches the shared tsdown Node config's self-contained extension bundling policy.
