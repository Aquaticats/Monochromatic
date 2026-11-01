# Tool Choices

## Package registry: npm only, no JSR, no GitHub deps

All dependencies must resolve from the npm registry (`registry.npmjs.org`).
No JSR (`npm.jsr.io`),
 no `github:` shorthand,
 no `git+ssh://` dependencies.

### Why not JSR

JSR's npm compatibility bridge (`npm.jsr.io`) has chronic quality issues:

- **Broken registry endpoints**:
   JSR does not implement the version-specific packument endpoint
  (`GET /<package>/<version>`) that the npm API defines.
   Returns 404.
  This is because `npm.jsr.io` is backed by static R2 objects,
   not a dynamic API.
  Package managers that use this endpoint (vlt,
   potentially others) fail on JSR transitive deps.
  See `TROUBLESHOOTING.vlt-jsr.md` for the full investigation.
- **Past incidents**:
   `bun install` failures with 502 errors on JSR's `If-None-Match` handling
  (jsr-io/jsr#1323).
   See `TROUBLESHOOTING.jsr.md`.
- **Transitive contamination**:
   npm packages published from JSR (e.g. `happy-opfs`) embed
  `@jsr/*` transitive dependencies in their `package.json`.
   These only resolve if the
  package manager has JSR scope routing configured (which most don't by default).
  This makes seemingly normal npm packages silently broken for non-JSR-aware PMs.

JSR packages that have npm equivalents are referenced by their npm versions:

- <!-- TODO: deprecate Optique --> `@optique/core`,
  <!-- TODO: deprecate Optique --> `@optique/run`:
   published to both npm and JSR (use npm `dev` tag for 1.
  x)
- `valibot`:
   on npm as `valibot@>=1.4.0` (Standard Schema-compatible)

### Why no GitHub deps

`github:user/repo` dependencies use `git clone` under the hood.
 This fails in
sandboxed environments without SSH key access and adds git as a runtime dependency
for package installation.
 HTTPS fallback configuration varies across package managers.

For CSS/font assets like TODS,
 vendor the file directly into the package source
with attribution comments.

## Framework: Astro > Nue

Astro:
 most supported static site generator.

NueJS:
 requires less common markdown format support.

## Editor: VSCode/VSCodium/Neovide > WebStorm

## Linting and formatting

- **oxlint**:
   primary JS/TS linter
- **dprint**:
   universal formatter

Rejected alternatives:

- **Biome**:
   insufficient rules
- **Stylelint**:
   removed 2026-10-04 and not an adoption candidate.
   All `pnpm audit` findings for the unpatched `braces` advisory (GHSA-vfj7-8cjw-p6xm)
   were reachable only through stylelint's tree,
   and the repo writes little CSS.
   A first-party replacement is tracked in issue #587;
   see `doc/decision/remove-stylelint.md`.
- **ESLint**:
   removed 2026-03-13 and not an adoption candidate.
   Oxlint covers the rules we relied on (gaps were filled by writing `@monochromatic-dev/oxlint-plugin-tsdoc`,
   `@monochromatic-dev/oxlint-plugin-no-restricted-syntax`,
   and `@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type`),
   and oxlint runs orders of magnitude faster.

## Testing: Node + module-test + Playwright

Unit tests run on Node through the in-house `@monochromatic-dev/test-harness-core` harness (Jest-style API on chai + sinon).
 Browser and end-to-end tests run on Playwright (in Podman for reproducibility).

Alternatives rejected:

1. **Vitest**:
    not an adoption candidate.
    Pulls in Vite's transform pipeline,
    which we do not want as a test-time dependency now that Vite is removed from the build path.

1. **WebdriverIO**
   - Firefox ESR support
   - No `prefers-contrast`/`prefers-reduced-motion` emulation
   - No Firefox user.
     js/Chrome flags support
   - Host configuration breaks reproducibility

1. **Playwright standalone**
   - No unit testing

## Code search: Agentic search > Semantic search

Semantic search (embedding-based vector search over codebases) consistently underperforms
agentic search (grep,
 glob,
 and file reading driven by an LLM agent).
Agentic search adapts its strategy based on intermediate results,
while semantic search returns a fixed set of "similar" results that often miss the actual target.
This aligns with findings from the Claude Code team at Anthropic,
who found agentic search outperforms semantic/embedding-based retrieval for code navigation.

No plans to implement an MCP server for codebase indexing or semantic code search.

## AI SDK: OpenAI SDK > Vercel AI SDK

Vercel AI SDK forces React dependencies for non-React projects:

- **Dependency chain**:
   `ai` → `@ai-sdk/react` → `swr` → `react`
- **Bloated tree**:
   Frontend UI concerns bundled with backend logic
- **No core package**:
   Missing modular `@ai-sdk/core` without UI dependencies

OpenAI SDK:
 direct API integration without unnecessary dependencies.

## Type checking: native tsc > classic tsc6

The monorepo type-checks with the native (Go) TypeScript compiler via `mise run lint:types`.
TypeScript 7 ships that compiler as the canonical `typescript` package
(`typescript@7.0.1-rc`,
 binary `tsc`);
 it previously ran through the
`@typescript/native-preview` dev builds whose binary was `tsgo`.

The classic JavaScript-based compiler survives only as `@typescript/typescript6`
(binary `tsc6`),
 which re-exports the TypeScript 6 programmatic API that the native
compiler will not provide until TypeScript 7.1.
Only tooling that imports the `typescript` module needs it,
e.g. `@stryker-mutator/typescript-checker` in the mutation-test package.

The Claude Code TypeScript LSP plugin has been removed (2026-03-10) because:

- It ran a different compiler than `lint:types`,
   producing diagnostics against the wrong checker
- It frequently serves stale diagnostics that do not reflect the current file state

A Claude Code `PostToolUse` hook will replace it,
triggering the package-specific `lint:types` task on every Edit/Write of a `.ts` file.
This gives fresh native-compiler diagnostics scoped to the affected package
without the staleness and tool mismatch of the LSP plugin.

## HTTP framework: h3 v2 > Hono > Elysia

h3 v2:
 minimal HTTP primitive (216-line core class) built on srvx (universal server) + rou3 (router).
Zero unnecessary abstractions.
 Utilities are standalone tree-shakeable functions,
 not framework methods.

Production apps that only need routing use raw `Bun.serve()` directly.
h3 is for packages that need cross-runtime support,
 middleware composition,
 or validation.

### Why not Elysia

Elysia generates request handlers as JavaScript strings via `new Function()` at startup (`compose.ts`,
 2,805 lines).
A separate system (`sucrose.ts`,
 763 lines) parses handler source code via `Function.toString()` with regex
to infer which context properties each handler accesses,
then the code generator omits unused parsing steps from the generated function.

Problems with this approach:

- **Security**:
   code generation without input sanitization produced an RCE chain
  (GHSA-8vch-m3f4-q8jf + GHSA-hxj9-33pp-j2cc,
   patched 1.4.17).
  Cookie config was interpolated into generated code strings unsanitized.
- **Self-defeating optimizer**:
   issue #1604 showed AOT-enabled Elysia at 3,853 req/s
  vs 175,951 req/s with `aot: false` (45x regression).
  Hono (no code generation) hit 237,229 req/s in the same benchmark.
- **Dual codepath divergence**:
   AOT and non-AOT modes behave differently.
  Issues #1753,
   #952,
   #1458 document parsing,
   hook,
   and routing inconsistencies between modes.
- **Fragile source analysis**:
   `Function.toString()` breaks under minification (issue #1617),
  `bun build --minify` breaks integrating libraries (issue #740).
  Sucrose uses hardcoded character positions that assume unminified source formatting.
- **CSP incompatible**:
   `new Function()` requires `unsafe-eval` in Content-Security-Policy.
  Cloudflare Workers blocks it entirely (issue #58).
  The `aot: false` workaround itself breaks routing in plugin compositions (issue #1244).
- **Diminishing returns**:
   Bun.
  serve() absorbs framework optimizations over time
  (built-in route tree with SIMD parameter decoding,
   static responses,
   cookie parsing).
  V8/JSC JIT compilers already perform function inlining,
   dead code elimination,
  and type specialization that Elysia reimplements in userland.
  The optimization gap narrows with each runtime release while the complexity cost is fixed.
- **God class**:
   `index.ts` is 8,292 lines containing routing,
   lifecycle hooks,
  plugin system,
   decorators,
   state management,
   guards,
   schema models,
   and compilation.

### Why not Hono

Hono's core HTTP framework is clean:
 74-line koa-style compose,
 539-line base class,
5 router implementations,
 zero runtime dependencies,
 no code generation.

Rejected for scope creep:

- **Client-side JSX runtime** (`hono/jsx/dom`):
   virtual DOM reconciler (792-line `render.ts`),
  hooks,
   state management,
   hydration.
   A React alternative inside an HTTP router.
- **CSS-in-JS** (`hono/css`):
   CSS generation utilities in a server framework.
- **Static site generation** (`hono/helper/ssg`):
   build-time page generation.
- 5,000 lines (21% of source) dedicated to client-side rendering concerns.

These are behind subpath exports and tree-shake away,
but they signal a trajectory toward full-stack framework territory.
Maintainer attention splits across HTTP routing,
 JSX reconciliation,
 and SSG:
concerns that belong in separate packages.

Hono's TypeScript inference also slows tsserver on large apps (issues #3945,
 #3869)
and breaks with 3+ chained middlewares (#3587).

### Why h3 v2

- **Minimal core**:
   H3 class is 216 lines.
   Delegates routing to rou3,
   server adapters to srvx.
- **Focused scope**:
   HTTP primitives only.
   No JSX,
   no CSS,
   no SSG,
   no client runtime.
- **Lazy evaluation**:
   getters and symbols defer body parsing,
   session loading,
  and response header construction.
   Same "skip what you don't use" benefit as Elysia's Sucrose
  without source code parsing.
- **Standard Schema validation**:
   works with Zod,
   Valibot,
   ArkType via Standard Schema v1 interface.
  No bundled validation library.
- **Battle-tested indirectly**:
   srvx + rou3 power Nuxt/Nitro's deployment base.
- **No code generation**:
   no `new Function()`,
   no `eval()`,
   no `Function.toString()`.
  Request handling is straightforward function dispatch.

### h3 v2 stability

h3 v2 is in RC (v2.0.1-rc.
16 as of 2026-03-09).
 v1 still receives patches.
The RC label reflects API finalization,
 not instability;
the core is functionally complete and the underlying srvx + rou3 are production-deployed at scale.
Nitro v3 will require h3 v2,
 which sets a hard timeline for stabilization.

Accepted risk:
 API surface changes between RC and stable.
Mitigation:
 Elysia usage in this repo is minimal (two experimental packages with basic routing).
Migration surface is small enough that any h3 v2 API changes are trivial to absorb.

## Bundler: raw rolldown (superseded tsdown on 2026-07-15)

Raw rolldown drives every build flavor through the repo-owned
`@monochromatic-dev/config-rolldown` package.
Decision record and pilot evidence:
`doc/planning/tsdown-removal.md`.

Driver:
 layer reduction.
tsdown's value to this repo compressed into small owned glue
(externals from package.json,
 `.mjs` filename templates,
a shebang chmod plugin,
 task-level clean),
while its unused subsystems,
hidden timers,
and lifecycle management caused unaccountable delays.
The first revisit trigger below fired (rolldown 1.0.0 stable,
 2026-05-07).
tsdown meanwhile moved into the rolldown org as the official library layer;
we removed it anyway because official upstream ownership does not shrink
the unused machinery it carries here.
Declarations stay on `rolldown-plugin-dts` with `generator: 'oxc'`
(benched 2026-07-15:
the tsgo backend cannot emit across inlined workspace sources
and costs four times oxc's declaration increment where it does build).

The section below records the superseded 2026-03-01 decision for history.

### Superseded: tsdown > raw rolldown (2026-03-01)

tsdown (v0.20.3) is a ~5,000-line config translator and plugin orchestrator on top of rolldown.
Full source audit completed 2026-03-01.

#### Why not raw rolldown

Rolldown's primary customer is Vite (application bundling).
Library bundling is a secondary concern for the rolldown team.
tsdown fills the gap with:

- **Auto-externalization**:
   reads `dependencies` + `peerDependencies` from `package.json`
  and marks them external via a rolldown `resolveId` plugin (`dep.ts`,
   193 lines).
  Also scans the final bundle for leaked node_modules imports (`inlineOnly` safety net).
- **DTS generation**:
   wires `rolldown-plugin-dts`,
   handles the CJS special case
  (separate rolldown build pass for `.d.cts` files when format is CJS + DTS enabled).
- **Output extension matrix**:
   `.mjs`/`.cjs`/`.js` determined by format x `package.json` `type` x `fixedExtension` flag.
- **Watch-safe clean**:
   in watch mode,
   deletes only previous output chunks instead of nuking the directory.
  Prevents race conditions when watch-building multiple interdependent packages in parallel.
- **Config file loading**:
   detects Bun/native TS support and uses native `import()` with zero transpilation overhead,
  falls back to `unrun` otherwise.
   No jiti/esbuild for config loading.

#### What tsdown does that we don't use but is fine to carry

- Workspace mode (92 lines,
   half-baked:
   no topological sort,
   shallow config merge,
   no error isolation).
  We use mise for monorepo orchestration instead.
- Hooks system (hookable,
   86 lines).
   Redundant with mise task sequencing.
- pkg/ directory (publint,
   attw,
   exports generation;
   821 lines,
   16% of codebase).
  Post-build validation,
   not bundling.
   Would be better as a separate tool.
- CSS handling (160 lines).
   Admitted workaround until rolldown supports CSS syntax lowering natively.

#### Known weaknesses (accepted)

- Test coverage skews toward utils;
   core pipeline (`build.ts`,
   `rolldown.ts`,
   `dep.ts`,
   `output.ts`) is untested.
- TSDoc nearly absent on the public `UserConfig` type surface.
- Six files exceed 200 lines;
   `resolveUserConfig` is a ~200-line procedural block.

#### Why we're cautious about dropping it

- sxzz (Kevin Deng) is the author of both tsdown and `rolldown-plugin-dts`,
   a Vite team member,
  and works directly with the rolldown codebase.
   If raw rolldown were sufficient for library bundling,
  he wouldn't maintain a 5,000-line wrapper.
   The pain points he's solving likely include edge cases
  in CJS interop,
   DTS dual-format builds,
   shebang preservation,
   and `node:` protocol handling
  that don't show up in a feature matrix but break real packages on npm.
- Full source audit (2026-03-01) confirmed general code quality is high and nothing suspicious.
  The architecture is sound (config translator + plugin orchestrator),
   dependencies are reasonable,
  and there's no unnecessary complexity in the core path.
   The weaknesses listed above are
  engineering tradeoffs,
   not red flags.

#### When to revisit

- Rolldown 1.0 stable ships (currently rc.
  6 as of 2026-02-26).
- Rolldown absorbs auto-external from `package.json` or ships a library preset.
- `rolldown-plugin-dts` gets absorbed into rolldown core.
- Any of these would shrink the gap enough to reconsider raw rolldown + mise.
