# Done; Framework evaluation

Written Feb 9,
 2026.
 Historical record only:
 it documents the reasoning behind the original vanilla TS + Bun choice over SvelteKit,
 Vue Vapor,
 or Web Components frameworks.
 The current app runtime is Node with h3 and tsdown,
 as documented in `README.md` and `SPEC.md`.

## Context

The original plan used SvelteKit with SSR (adapter-bun).
 While working through the Svelte tutorial,
 accumulated architectural concerns led to a full framework re-evaluation.
 See `svelte-first-impressions.md` in Nextcloud notes for the complete critique.

## Svelte; why we're leaving

Documented extensively in the Svelte notes.
 Summary of architectural problems:

- **Shared mutable state with no access control.
  ** Exported `$state` is a global mutable singleton.
   Any importer can read and write.
   No `readonly()` equivalent.
- **Implicit context rules compound.
  ** `bind:value` does two-way binding on text inputs but also silent type coercion on number inputs.
   `{#each}` without `key` silently corrupts component state via positional diffing.
   `+=` vs `= x + 1` produces different dependency tracking.
   `.svelte.js` extension is a compiler directive with no in-code indicator.
- **No props contract.
  ** Parent components can't see what's required vs optional without reading child source.
   `$props()` spreading dumps everything onto DOM elements unfiltered.
- **Framework abstractions over platform primitives.
  ** `{@attach}` reinvents `<dialog>` + `connectedCallback`.
   `style:` directives are inline styles with nicer syntax.
   Conditional class syntax where `data-` attributes work.
- **Compilation advantage irrelevant for JS-heavy apps.
  ** Svelte's pitch is "less JS shipped.
  " Done is inherently JS-heavy (real-time task management,
   AI curation,
   10k cards).
   The app needs JS regardless.
   What remains is architectural weakness.

## Vue Vapor; architecturally right, practically wrong (for now)

### What it gets right

Vue Vapor (Vue 3.6,
 currently beta.
5 as of Jan 30 2026) addresses every Svelte concern:

- `defineProps<{ title: string }>()`:
   required is default,
   optional is explicit (`?`),
   TypeScript enforces at call site
- `readonly(ref)`:
   export read-only proxy,
   mutations stay internal
- `v-for` requires `:key` by ESLint default;
   no silent positional corruption
- `$attrs` excludes declared props/emits;
   controlled forwarding
- `v-model` decomposes to `:value` + `@update:modelValue`:
   interceptable,
   not opaque
- Composition API only in Vapor;
   simpler than old Options + Composition split
- No file-extension compiler magic (`.vapor.vue` is opt-in per component)
- alien-signals reactivity rewrite;
   SolidJS/Svelte-level performance

### What blocks it for this project

1. **SSR is beta.
   ** Vue 3.6 SSR/hydration for Vapor is checked off on the roadmap but only in beta.
   5.
    The original plan relied heavily on SSR.
2. **No meta-framework ready.
   ** Nuxt 4 (stable July 2025,
    currently 4.3.1) does not integrate Vapor yet;
    the roadmap checkbox is unchecked.
    `vue-i18n` has a known Vapor incompatibility (GitHub #2234).
    Vue Router has an open Vapor PR but isn't merged.
3. **No form actions pattern.
   ** SvelteKit's `+page.server.ts` load functions + form actions + `use:enhance` have no Vue equivalent.
    You'd wire `fetch()` manually or build your own.

### Revised assessment after dropping SSR requirement

With SSR off the table,
 Vapor's beta status becomes less concerning;
 client-only SPA rendering is more stable than SSR/hydration.
 But this raised the question:
 if the app is a client-side SPA served from Bun,
 do we need a frontend framework at all?

## Do we actually need a framework?

Checked Done's spec against what frameworks provide:

<table>
<thead>
<tr>
<th>Framework feature</th>
<th>Done's actual need</th>
</tr>
</thead>
<tbody>
<tr>
<td>**Reactivity**</td>
<td>One `setInterval(1000)` for timer display. Everything else is user-initiated (tap -> fetch -> re-render).</td>
</tr>
<tr>
<td>**Routing**</td>
<td>5 screens. A `switch` on `pathname` covers it. Task details and search are overlays, may not even need routes.</td>
</tr>
<tr>
<td>**Client-side state**</td>
<td>All state lives in server SQLite. Client state is: one timer tick number,
"is section collapsed" booleans, "is overlay open" boolean.</td>
</tr>
<tr>
<td>**Component model**</td>
<td>5 screens, maybe 3-4 reusable elements (task card, chip editor, collapsible section, FAB).</td>
</tr>
</tbody>
</table>

No reactive state graph,
 no complex routing,
 no client-side store.
 A framework solves problems Done doesn't have.

## Web Components frameworks considered

- **Lit 3.0**:
   Most mature WC library.
   SSR is experimental ("Lit Labs"),
   only Node (not Bun).
   Repository appears less actively maintained than desired.
   Good for components,
   no app architecture.
- **Enhance**:
   Full WC app framework with file-based routing and WASM SSR.
   Small team/community,
   routing tied to Architect/AWS deployment model.
   Too niche for a competition.
- **Stencil**:
   Ionic's WC compiler.
   Component compiler,
   not app framework.
   No routing,
   no server integration.
- **Plain custom elements**:
   No framework needed for 5 screens and 3-4 reusable elements.

## Chosen approach: vanilla TS + Bun.serve() + Bun.build()

### Architecture

- **One process,
   one command:
  ** `bun --watch src/server.ts`
- **Server:
  ** `Bun.serve()` handles API routes (DB,
   AI proxy) and serves built client assets
- **Client:
  ** Plain TypeScript with `document.createElement`,
   custom elements where reuse is needed
- **Build:
  ** `Bun.build()` runs at server startup;
   bundles,
   tree-shakes,
   and (in prod) minifies client TS into JS
- **Dev reload:
  ** `bun --watch` restarts server on any file change,
   which re-runs `Bun.build()`,
   producing fresh client bundles.
   Same code path in dev and prod.
- **Operational advantage:
  ** Orchestrator spawns per-user Bun processes.
   New processes immediately get the latest build since `Bun.build()` runs at process start.

### Why this works for Done

- All state is server-authoritative (SQLite).
   Client is a thin view layer.
- No reactivity needed beyond one `setInterval`.
- 5 screens is trivially handleable with URL pathname matching.
- Tree-shaking via `Bun.build()` keeps imports like zod-mini efficient.
- Zero dev/prod code path divergence;
   same `Bun.build()` call,
   only `minify` flag differs.
- TypeScript throughout;
   Bun runs server TS natively,
   `Bun.build()` handles client TS.
- No framework abstractions over platform primitives;
   uses `<dialog>`,
   `data-` attributes,
   CSS directly.

### What we give up

- **No progressive enhancement.
  ** Forms require JS.
   Acceptable;
   Done is inherently a JS app.
- **No HMR (hot module replacement).
  ** Full page reload on save via `bun --watch`.
   Acceptable for 5 screens.
- **No component-scoped CSS.
  ** Use BEM or `data-` attribute selectors.
   Acceptable for competition scope.
- **Manual DOM construction.
  ** More verbose than templates.
   Acceptable;
   the DOM surface is small.

### Tradeoffs vs Vue Vapor post-competition

If Done grows beyond competition scope (more screens,
 complex state,
 collaborative features),
 Vue Vapor becomes worth revisiting once 3.6 is stable and Nuxt integrates it.
 The database layer,
 AI integration,
 orchestrator,
 and Docker setup are all framework-agnostic and won't change.

## The broader question: what actually needs a UI framework?

Done is not a simple app.
 It has a multi-tenant orchestrator with auth and process spawning,
 AI integration with structured output parsing,
 a task blocking dependency graph with circular dependency handling,
 server-authoritative timers with client-side display math,
 two-way GitHub sync with lossless round-trip,
 FTS5 full-text search,
 email reminders,
 daily backups,
 and Docker deployment.
 It's genuinely complex;
 none of that complexity lives in the UI layer.

This prompted a harder question:
 if *this* app doesn't need a framework,
 what does?

### Categories that supposedly need frameworks

**Collaborative editors (Google Docs,
 Figma):
** Figma wrote their own renderer on Canvas/WebGL,
 not a component framework.
 The hard problem is CRDT/OT conflict resolution,
 not UI reactivity.
 No startup should build this from scratch regardless.

**Spreadsheets:
** Genuine reactive dependency graph (cell A1 depends on B2 depends on C3).
 But the rendering is a grid:
 typically a `<canvas>` or virtualized table,
 not a component tree.
 The reactive engine is the spreadsheet formula evaluator,
 not the UI framework.

**DAWs / video editors:
** The rendering path is WebGL/WebGPU.
 A UI framework doesn't help with waveform rendering or timeline scrubbing at 60fps.
 These should wait for WebGPU maturity anyway.

**Chat with presence:
** Typing indicators,
 read receipts,
 message arrival,
 presence dots:
 each is one or two DOM mutations on specific,
 known elements.
 New message = `container.appendChild(messageEl)`.
 Typing indicator = `indicator.hidden = !isTyping`.
 Presence = `dot.dataset.online = "true"` + CSS.
 Each WebSocket event maps to an independent DOM update.
 No dependency graph,
 no cascading recalculations.
 A `switch` statement in `onmessage` covers it.
 Discord and Slack use React with 500+ engineers
 and regularly rebuild their rendering pipeline
 because React's reconciliation is the performance problem at their scale.

**Gmail/Slack-scale SPAs:
** Dozens of views,
 deep linking,
 split panels.
 But no startup should build this scope,
 and the ones that did (Superhuman,
 Linear) ended up fighting their frameworks.

**Offline-first apps (Notion,
 Linear):
** The hard part is sync and conflict resolution,
 not UI state.
 Debatable whether offline-first is the right direction at all;
 it trades server simplicity for enormous client complexity.
 And the state container is really a local database (IndexedDB,
 SQLite via WASM),
 not a framework store.

**Complex undo/redo:
** A version control problem.
 Should defer to git-like approaches (event sourcing,
 operation logs),
 not UI framework state snapshots.

### Where this lands

The set of apps that genuinely need a client-side UI framework is close to empty.
 The things that look like they need frameworks either need a custom rendering engine (Figma,
 DAWs) or are doing independent DOM mutations that don't benefit from a reactive dependency graph (chat,
 dashboards,
 CRUD apps).

Frameworks solve the *developer ergonomics* problem (components,
 templates,
 file-based routing,
 hot reload tooling) not an *architectural* problem.
 And the ergonomics gap is shrinking as runtimes like Bun absorb the tooling pieces (native TS,
 `--watch`,
 `Bun.build()` with tree-shaking,
 `Bun.serve()` with file routing).

Done is strong evidence for this:
 an app with AI,
 multi-tenant orchestration,
 real-time timers,
 dependency graphs,
 full-text search,
 external sync,
 and email notifications;
 the UI is the simplest layer,
 trivially handleable with vanilla TS and platform APIs.

### Post-competition: empirical validation

The above is coherent reasoning but still speculation.
 After the competition,
 we'll rewrite Done's UI layer in multiple frameworks and compare empirically.
 The codebase is structured for this:
 `src/lib/` (database,
 AI,
 email,
 sync),
 `orchestrator/`,
 and Docker setup are all framework-agnostic.
 Only `src/server/page/`,
 `src/client/`,
 and the router change per framework.
 Planned rewrites:

- Vanilla TS + Bun (competition baseline)
- SvelteKit
- Vue Vapor (once 3.6 stable)
- SolidJS / SolidStart
- React / Next.
  js (control group)
- Lit (Web Components)

Compare on:
 lines of code,
 bundle size,
 time-to-implement,
 dev experience friction,
 runtime performance,
 and whether any framework provided an architectural advantage the vanilla version lacked.
