# Visualization library choice for `deps-cube`

Records the technology choice behind `package/dependency-catalog/cli-cube/`.
 Future
sessions consult this before re-proposing rejected paths.

This document is appended to,
 not rewritten.
 When a downstream choice forces
re-evaluation of an earlier one,
 mark the earlier decision superseded;
 do not
delete it.

## Context

`deps-cube` is a workspace audit tool.
 It reads every entry in the
`catalogs` and `catalog` blocks of `pnpm-workspace.yaml`,
 probes seven
attributes per package (TS ratio,
 source bytes,
 days-since-last-commit,
install size,
 weekly downloads,
 package age,
 license class),
 and emits a
single self-contained HTML file that renders the catalog as an interactive
3D scatter plot.
 Six channels are used:
 x,
 y,
 z,
 color,
 shape,
 size.

The intent is to surface non-TypeScript,
 tiny,
 abandoned,
 niche,
 leaf
packages visually so the next library-replacement audit can target the most
worthwhile candidates first.
 The tool is a sibling of the existing static
governance pieces (`.pnpmfile.mjs` blocklist throwing-stub policy,
`doc/dependency-blocklist.md`,
 per-package decision docs in this directory).

Constraints driving the library choice:

- Browser-only rendering (the file opens via `file://`).
   No server.
- ~120 catalog entries at the time of writing.
   Headroom for 10x growth.
- Interactive 3D camera (orbit,
   zoom,
   pan) with picking for tooltip + click.
- The audit tool's premise is that non-TypeScript deps are worth replacing.
  A viz library carrying a large non-TS surface would undermine the tool's
  own argument;
   the workspace's stated threshold is 95% TS by GitHub
  Linguist bytes.
- Single bundled output,
   no external `<link>` or `<script src>` references.
- Node runtime for the build step;
   rolldown with `format: 'iife'` and
  `minify: true` resolves and bundles transitively.

## Decision

Use `@deck.gl/core@9.3.2` plus `@deck.gl/layers@9.3.2`.
 Both MIT,
`type: "module"`.
 The `Deck` class provides the lifecycle,
 picking,
 and
event wiring;
 `OrbitView` plus its built-in `OrbitController` provide the
3D camera.
 Layer composition through `ScatterplotLayer`,
 `PolygonLayer`,
`PathLayer`,
 and `TextLayer` covers the scatter glyphs,
 coordinate planes,
threshold guide lines,
 axes,
 and labels respectively.

### TypeScript-ratio exception (user-approved)

The workspace's 95% TS heuristic is not met by deck.
gl.
 Verified
2026-05-12 via `gh api repos/visgl/deck.gl/languages`:

- TypeScript:
   3 671 115 bytes (72.56%)
- JavaScript:
   746 125 bytes (14.75%)
- HTML:
   238 542 bytes (4.71%)
- Python:
   202 176 bytes (4.00%)
- CSS:
   113 103 bytes (2.24%)
- Jupyter Notebook:
   72 374 bytes (1.43%)
- Other (Makefile,
   Shell,
   MDX,
   Jinja,
   Dockerfile):
   15 843 bytes (0.31%)
- Total:
   5 059 278 bytes.

The JavaScript portion is dominated by shader-bundle scripts (`bundle/`),
legacy submodule entry shims,
 and Jest fixtures for layer-level tests.
 The
TypeScript-stricter Linguist breakdown of just `modules/core/src` and
`modules/layers/src` (the surface this package imports) is materially
higher than 72.56%;
 the whole-repo ratio is dragged down by docs (HTML
example scaffolds,
 Jupyter notebooks for the data-science adjacent
modules) and Python tooling for the website build.

The user,
 presented with the Linguist breakdown via a screenshot of the
GitHub language sidebar,
 approved the exception explicitly.
 The
deciding evidence is the user's authorization,
 not an independent
re-derivation here.
 Subsequent sessions must not re-litigate the
threshold for this dependency without renewed user input.

## Rejected alternatives

Each subsection names the specific gate the candidate fails,
 per the
`AGENTS.md` rule "cite the specific incompatibility,
 not 'doesn't fit'".

### `plotly.js`

GitHub Linguist breakdown (`gh api repos/plotly/plotly.js/languages` at
audit time) puts the repository at effectively 0% TypeScript.
 Plotly is a
pure-JavaScript project.
 Fails the workspace TS heuristic outright,
 and
the failure direction is the same one the audit tool was built to
identify;
 adopting plotly to render the audit tool would make the tool
self-contradicting.
 Secondary:
 bundle weight,
 on the order of 3 MB
minified,
 against deck.
gl's ~750 KB measured here.
 Tertiary:
 `scatter3d`
is WebGL-only in plotly too,
 so the "SVG-friendly" attribute that
sometimes lands plotly in workspace tooling is moot for the 3D case.

### `three.js`

Same gate:
 GitHub Linguist reports the repository as essentially 100%
JavaScript.
 Same self-contradiction.
 Beyond that,
 `three.js` provides
primitives (mesh,
 geometry,
 material,
 renderer) rather than a scatter
abstraction;
 the build-out to reach picking,
 hover,
 tooltip,
 and a
3D scatter glyph layer would be in the 500-700 line range against
deck.
gl's accessor-driven `ScatterplotLayer`.

### `echarts` (with `echarts-gl` for 3D)

Linguist puts the main `apache/echarts` repository at 88.2% TypeScript
at audit time.
 Below the 95% workspace heuristic,
 and the user has not
been asked to approve an exception specifically for echarts.
 Without
that explicit approval the gate stands.

### `d3-3d` plus `d3` modular packages

`d3-3d` is ~100% TypeScript (~48 KB),
 but provides projection helpers
only,
 not a complete 3D scatter widget.
 The remaining d3 surface for
picking,
 camera,
 and event routing comes from the broader d3 ecosystem,
which is mostly JavaScript.
 Rejected on TS for the JS modules,
 and on
feature surface for d3-3d alone.

### `@thi.ng/*` composite

The `@thi.ng/*` ecosystem clears the TS heuristic comfortably (99.98% TS
across published packages at audit time).
 No shipped 3D scatter widget
exists:
 `@thi.ng/viz` is 2D;
 `@thi.ng/scenegraph` and
`@thi.ng/geom-webgl` are marked ALPHA.
 A composite implementation would
require on the order of 400+ lines of glue to wire the
scenegraph,
 picking,
 and orbit controls together against the deck.
gl
accessor-driven baseline.
 If deck.
gl became unavailable in a future
audit,
 this is the path to revisit.

### `@nivo/*`, `visx`, `recharts`

All clear the TS heuristic.
 All are React-locked,
 and `deps-cube` ships
no React (the controller is a vanilla `Deck` instantiation against a
pre-existing `<canvas>` element).
 Beyond the React lock-in,
 none ship a
3D scatter primitive;
 the orbit camera and picking would still need a
custom layer underneath,
 which negates the saved code the visualization
library is meant to provide.

### Hand-rolled WebGL or Canvas

Zero deps,
 full control over the bundle weight.
 Estimated at ~1100 lines
to reach feature parity with the deck.
gl-backed implementation
(orbit-camera matrix math,
 picking via id-buffer rendering,
 hover/click
hit-tests,
 ScatterplotLayer-equivalent accessor pipeline,
 text label
SDF or Canvas-2D overlay).
 Against the ~300 lines of glue the deck.
gl
version needs (`deck-config.ts`,
 `deck-accessors.ts`,
 `deck-layers.ts`,
`deck-scatter.ts`,
 `deck-labels.ts`,
 totalling about 280 code lines net
of imports and types),
 the LOC delta is the deciding factor.

## Audit notes

`@deck.gl/core@9.3.2` and `@deck.gl/layers@9.3.2` were on disk at audit
time,
 both MIT licensed,
 both `"type": "module"`.

### Transitive dependency surface

Verified by reading `node_modules/.pnpm/@deck.gl+core@9.3.2/node_modules/@deck.gl/core/package.json`
and `node_modules/.pnpm/@deck.gl+layers@9.3.2_*/node_modules/@deck.gl/layers/package.json`.

vis.
gl ecosystem (all MIT,
 all `type: "module"`):

- `@loaders.gl/core@4.4.1`,
   `@loaders.gl/images@4.4.1`,
  `@loaders.gl/schema@4.4.1`
- `@luma.gl/core@9.3.3`,
   `@luma.gl/engine@9.3.3`,
  `@luma.gl/shadertools@9.3.3`,
   `@luma.gl/webgl@9.3.3`
- `@math.gl/core@4.1.0`,
   `@math.gl/polygon@4.1.0`,
  `@math.gl/sun@4.1.0`,
   `@math.gl/types@4.1.0`,
  `@math.gl/web-mercator@4.1.0`
- `@probe.gl/env@4.1.1`,
   `@probe.gl/log@4.1.1`,
  `@probe.gl/stats@4.1.1`

External (the only non-vis.
gl transitives):

- `gl-matrix@3.4.4` (MIT,
   dual CJS+ESM,
   used by `@luma.gl/*` for
  matrix math;
   pure JS but vendored upstream is also pure JS,
   so this is
  not a swap candidate)
- `mjolnir.js@3.0.0` (MIT,
   `type: "module"`,
   vis.
  gl org,
   event manager)
- `earcut@2.2.4` (ISC,
   CJS-only,
   polygon triangulation by Vladimir
  Agafonkin for Mapbox;
   used by `@math.gl/polygon`)
- `@mapbox/tiny-sdf@2.2.0` (BSD-2-Clause,
   `type: "module"`,
   SDF font
  generator;
   used by `@deck.gl/layers`'s `TextLayer`)

Licenses are uniformly permissive (MIT,
 BSD-2,
 ISC).
 No copyleft or
non-OSS surface in the transitive cone.

### Build provenance

vis.
gl publishes from public source on GitHub.
 Build tool is
`ocular-bundle` (the vis.
gl-maintained bundler wrapper around
webpack/rollup).
 Each package's `package.json` `prepublishOnly` script
references the local `ocular-bundle` invocation.
 The published `dist/`
output checked in to npm matches `src/` shape via the `exports` map.
Vis.
gl is a Linux Foundation project (originated at Uber).
 Reproducible
builds are achievable from the public source under the documented
`build-bundle` scripts.

### Maintenance signals

`gh api repos/visgl/deck.gl` at audit time (2026-05-12):

- Created:
   2015-12-15
- Default branch:
   master
- Stars:
   14 171
- Forks:
   2 224
- Watchers (subscribers):
   1 650
- Open issues:
   459
- Last push:
   2026-05-12 (same day as the audit)

`gh api repos/visgl/deck.gl/contributors --paginate` returned at least
277 entries (three pages of `per_page=100` returned 100,
 100,
 77).
 The
plan-stage estimate of "250+" is conservative;
 actual is closer to 280
at audit time.

`@deck.gl/core` and `@deck.gl/layers` weekly downloads from
`https://api.npmjs.org/downloads/point/last-week`:

- `@deck.gl/core`:
   630 143 / week
- `@deck.gl/layers`:
   622 185 / week

Single-maintainer concentration is low (the vis.
gl org has ~20 active
core maintainers across deck.
gl,
 luma.
gl,
 math.
gl,
 and loaders.
gl).

### Functional fit

`OrbitView` paired with `controller: true` provides drag-rotate around
the data center,
 shift-drag pan,
 scroll zoom,
 and a double-click reset
hook out of the box.
 `ScatterplotLayer` exposes per-point accessor
functions for position (`getPosition`),
 color (`getFillColor`),
 radius
(`getRadius`),
 and stroke (`getLineColor`,
 `lineWidthMinPixels`).
Picking is per-pixel WebGL hit testing via the `pickable: true` prop;
hover triggers the `getTooltip` callback,
 click triggers `onClick`.
Layer rebuilds are incremental via `deck.setProps({ layers: [...] })`;
no re-fetch,
 no full re-render.
 The total custom code saved over a
hand-rolled equivalent is in the 600 to 800 line range,
 dominated by
the matrix math and the picking pipeline.

## Implementation notes

Items worth recording for future sessions revisiting the package.

### Bundle weight is roughly 2x the planning estimate

The plan-stage estimate placed the inlined deck.
gl footprint at
"~400 KB" (200 KB core + 150 KB layers).
 Measured at task 10 with
`bun build src/scripts/controller.ts --format=iife --minify
--target=browser`:

- Output:
   754 348 bytes (~754 KB)
- Modules bundled:
   513

The delta is explained by the transitive surface enumerated above.
`@deck.gl/core` alone pulls in four `@luma.gl/*` packages,
 four
`@math.gl/*` packages,
 three `@probe.gl/*` packages,
 `gl-matrix`,
 and
`mjolnir.js`.
 `@deck.gl/layers` adds `@loaders.gl/{images,schema}`,
`@mapbox/tiny-sdf`,
 and `earcut`.
 The plan-stage estimate counted the
two `@deck.gl/*` packages only and did not walk the dependency closure.

The final HTML output (with empty probes) is ~770 KB,
 of which ~754 KB
is the inlined IIFE bundle.
 With 120 probes serialised as JSON
(`window.__PROBES__`),
 the document grows by approximately 30 to 60 KB
depending on probe completeness.
 Total file size at the operating scale
is in the 800 to 850 KB range.

This is well below the rejected plotly.
js footprint (~3 MB) and stays
opening in browsers via `file://` without performance concerns at the
120-glyph scale.

### Filled-vs-stroked deviation from plan (superseded)

This section describes the first deck.
gl implementation.
 It was
superseded by the mesh-glyph migration in commit `220818cb`,
 then by
per-probe baked-name textures in commit `62d80159`.

### Mesh glyphs supersede filled-vs-stroked circles

The current implementation encodes leaf packages as spheres and non-leaf
packages as octahedra,
 both rendered through `SimpleMeshLayer` in
`package/dependency-catalog/cli-cube/src/deck-scatter.ts`.
 Unknown probes are
rendered as spheres in the unknown cluster.

The change was made after Firefox visual verification showed flat
`ScatterplotLayer` glyphs foreshortening into ellipses at the orbit
camera's tilt.
 True 3D meshes preserve round and diamond-like silhouettes
across camera angles and allow package names to be baked into per-probe
textures so depth testing occludes labels correctly.

`partitionProbes` now lives in
`package/dependency-catalog/cli-cube/src/deck-scatter-helpers.ts` and routes each
probe to one of three buckets:
 leaf,
 non-leaf,
 or unknown.
 The unknown
bucket is still rendered at the edge of the scene so missing repository or
monorepo-scoped measurements do not collapse to the origin.

### Pre-existing `<canvas>` vs deck.gl-created `<canvas>`

deck.
gl's `Deck` constructor accepts either a `parent: HTMLDivElement`
prop (deck.
gl creates its own canvas as a child),
 or a `canvas` prop
typed as `HTMLCanvasElement | string` (deck.
gl uses the supplied canvas
element or the element id).
 The implementation supplies
`<canvas id="deck-canvas">` inside the flex-layout main element,
 and
the controller passes `canvas: 'deck-canvas'` to `new Deck`.
 The
`parent` form was rejected because deck.
gl types `parent` as
`HTMLDivElement | null` and the existing `el()` typed-accessor in
`controller-dom.ts` returns `HTMLElement`;
 the string-id form of
`canvas` is accepted natively,
 so no narrowing was needed.

### Top-N name labels ranking

The "show top-N risk labels" display option currently sorts visible
probes by `daysSinceLastCommitOrNull` descending (oldest first) and
takes the top 10.
 Subject to refinement once the audit-target scoring
is formalised;
 a weighted score across staleness,
 small size,
 non-TS,
and low downloads would better surface the audit corner of the cube.
The implementation lives in `computeNameBakeSet` in
`package/dependency-catalog/cli-cube/src/deck-scatter-helpers.ts`;
 the resulting
set controls per-probe texture baking in `src/deck-scatter.ts`.

### Linguist `bytes(TypeScript) / sum(bytes)` is whole-repo

Monorepo-housed catalog entries (`repository.directory` set) cannot use
Linguist for the per-subdirectory TS ratio;
 the per-directory breakdown
is not exposed by the GitHub API.
 Such entries return `null` for
`tsRatioOrNull` and `sourceBytesOrNull` from the probe;
 the
visualization places them in the Unknown cluster region rather than
coercing the null to 0 at the origin.
 Verified in `src/probe-fields.ts`
and `src/deck-scatter.ts`'s `partitionProbes`.
