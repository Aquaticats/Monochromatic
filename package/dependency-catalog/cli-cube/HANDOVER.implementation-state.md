# Handover: deps-cube implementation in progress

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/pnpm-workspace-catalog` is now `package/dependency-catalog/core-pnpm`;
> `@monochromatic-dev/module-pnpm-workspace-catalog` is now `@monochromatic-dev/dependency-catalog-core-pnpm`;
> `package/dev-script/deps-cube` is now `package/dependency-catalog/cli-cube`;
> `@monochromatic-dev/module-test` is now `@monochromatic-dev/test-harness-core`.

## What you are picking up

The package `package/dev-script/deps-cube/` is being built per the approved plan at
`/home/user/.claude/plans/make-a-new-workspace-serialized-puppy.md`.
 Read that plan first;
 it has the full design,
 the dimension mapping,
 the deck.
gl scene composition,
 the UI/UX spec,
 the data-acquisition pipeline,
 the library-audit rationale (deck.
gl approved at 72.7% TS as an explicit user-confirmed exception),
 and the verification checklist.

Tool premise:
 scatter-plot every catalog entry in the `pnpm-workspace.yaml` in 3D feature space (6 dims = 3 spatial + color + shape
 + size),
 with deck.
gl WebGL rendering and a custom HTML control panel for dim swapping,
 3-state boolean filtering,
 range sliders,
 name search,
 display toggles,
 and URL-hash bookmarking.
 Output:
 `<package>/dist/deps-cube-<YYYY-MM-DDTHH-MM-SSZ>.html` (filename carries ISO 8601 UTC down to seconds with `:`→`-`,
 so each run produces a distinct artifact;
 the package directory is located via `findPackageRoot` walking up from `import.meta.dirname`).
 Stdout:
 exactly `Saved to <abs-path>`.

**Status (2026-05-12,
 ninth handover;
 visual-fix iteration)**:
 tasks 1 through 12 done;
 task 13 still partial (headless verification done;
 manual Firefox interaction checks pending).
 The eighth handover described three Task-13 bugs that were fixed.
 THIS handover covers a second round of visual fixes (Tasks 14 to 22) addressing six visual issues found when actually
 opening the HTML in Firefox:
 stretched ellipses (flat 2D `ScatterplotLayer` glyphs foreshortened at 30° camera tilt),
 invisible wireframe,
 fragmented overlapping axis labels,
 indistinguishable leaf vs non-leaf,
 double scrollbar,
 single-handle range sliders.
 The user also pivoted the backdrop visual:
 scene should resemble a classic 3D coordinate-system diagram (three green coordinate planes + arrowed axes with tick
 marks + `X`/`Y`/`Z` capital labels + origin `O`),
 not a wireframe box with threshold planes.
 Implemented:

- New deps `@deck.gl/mesh-layers` + `@luma.gl/engine` (catalog + `package.json` + pnpm install).
- New `src/deck-geometries.ts`:
   `sphereGeometry`,
   `octahedronGeometry` (flat-shaded with 24 duplicated vertices),
   `coneGeometryX`/`Y`/`Z` (pre-rotated per axis via luma.
  gl's `verticalAxis` prop).
- Rewrote `src/deck-scatter.ts` to use `SimpleMeshLayer` with the sphere/octahedron geometries instead of
- `ScatterplotLayer`.
   Glyphs are now true 3D solids (spheres for leaf,
   octahedra for non-leaf) round/diamond-silhouetted from every camera angle.
- New `probeRadiusWorld({probe, state, bounds})` accessor in `src/deck-accessors.ts` returning world-space radius
- scaled to 0.5 to 3% of the bounds diagonal (`Math.hypot(dx, dy, dz)`).
- Rewrote `src/deck-layers.ts`:
   replaced wireframe-box + threshold-planes with `buildAxisShaftLayer` (3 thick PathLayer segments from min-corner to
   tip),
   `buildAxisArrowheadLayers` (3 cone `SimpleMeshLayer`s at tips),
   `buildAxisTickLayer` (5 ticks per axis).
- New `src/deck-planes.ts`:
   `buildCoordinatePlaneLayers` (3 pale-green `PolygonLayer`s at floor/back/side walls intersecting at the data box's
   min corner) + `buildThresholdLineLayer` (thin guide lines on the planes at the threshold values;
   replaces the previous opaque threshold-plane fills while preserving the heuristic meaning).
- Rewrote `src/deck-labels.ts`:
   split into four layer factories:
   `buildAxisCapitalsLayer` (bold `X`/`Y`/`Z` at arrow tips,
   24px),
   `buildAxisSubtitlesLayer` (small dim-name labels alongside),
   `buildOriginLabelLayer` (single `O` at the min corner),
   `buildNameLabelsLayer` (per-glyph names,
   unchanged behaviour).
- Rewired `src/deck-config.ts` `buildLayers` to call the new factories in the right back-to-front order;
   the `showWireframe` display toggle now controls the coordinate planes (the wireframe-box layer is gone).
- `src/styles.css`:
   `html, body { block-size: 100vh; overflow: hidden }` and `#canvas-host { block-size: 100vh; overflow: hidden }` to
   remove the page-level scrollbar (now only the controls panel scrolls).
   Added `.range-track` rules:
   two `<input type="range">` stacked on a single track via `position: absolute`;
   only the `::-webkit-slider-thumb` / `::-moz-range-thumb` accept pointer events so both can be dragged independently.
   No `background-color` override on the canvas;
   green is forgiving of dark backdrops,
   so the OS dark/light scheme is respected.
- `src/render-controls.ts` `renderRangeRow`:
   wraps the two inputs in `<div class="range-track">` so the dual-thumb CSS applies.
- Test fix:
   `test/render-controls.unit.test.ts` updated for the new HTML structure (asserts `class="range-track"` × 6,
   twelve range inputs,
   six `range-row` classes via prefix match).

Lint,
 types,
 and tests all clean (0 errors;
 ~454 stylistic warnings,
 all non-blocking).
 9 PASS / 0 FAIL / 1 SKIP across the eight test files.
 The CLI runs end-to-end and produces a fresh `dist/deps-cube-*.html` (~826KB;
 slight uptick from the previous 803KB driven by the mesh-layers + luma.
gl/engine additions in the inlined controller bundle).

**Status (2026-05-12,
 tenth handover;
 coordinate-system backdrop iteration 2)**:
 when the user opened the iteration-1 HTML in Firefox they could see only ONE green coordinate plane (back wall):
 the floor and side wall were missing entirely.
 Diagnosis via reading deck.
gl source:
 `SolidPolygonLayer`'s default tessellator runs earcut on the XY projection of the polygon.
 For our floor (constant `y`) and side wall (constant `x`) the XY projection collapses to a line,
 earcut produces zero triangles,
 and the layer renders nothing.
 The fix is `_full3d: true`,
 which permutes the largest-area plane onto XY before earcut and back after;
 that prop is exposed only by `SolidPolygonLayer`,
 not the high-level `PolygonLayer` wrapper.
 Implemented:

- `src/deck-planes.ts`:
   swapped `PolygonLayer` for `SolidPolygonLayer` and passed `_full3d: true` on each of the three plane layers.
   Bumped `COORDINATE_PLANE_COLOR` alpha from 30 to 60 so the planes read clearly without dominating the data.
- `src/script/state.ts`:
   `DEFAULT_DISPLAY_TOGGLES.showThresholdPlanes` flipped from `true` to `false`.
   The brown threshold guide lines were confusing default-on chrome;
   they're an opt-in heuristic overlay now.
- New `src/script/scheme.ts`:
   exports `ChromeColors` type and `detectScheme()` that branches on
   `globalThis.matchMedia('(prefers-color-scheme: dark)').matches` to return either a light-on-dark or dark-on-light
   palette for the chrome (axes / ticks / capitals / origin / name labels).
- `src/script/controller.ts`:
   `createSession` calls `detectScheme()` once and stores the palette on the session;
   passed through to every `buildLayers` call via the existing render path.
   New `chrome: ChromeColors` field on the `Session` type.
- `src/deck-config.ts` `buildLayers`:
   accepts `chrome: ChromeColors`,
   forwards to every chrome-coloured factory (`buildAxisShaftLayer`,
   `buildAxisArrowheadLayers`,
   `buildAxisTickLayer`,
   `buildOriginLabelLayer`,
   `buildAxisCapitalsLayer`,
   `buildAxisSubtitlesLayer`,
   `buildNameLabelsLayer`).
   Coordinate planes still use the static `COORDINATE_PLANE_COLOR` (green works on either backdrop).
- `src/deck-labels.ts`:
   dropped the opaque-white label backgrounds (they dominated the dark scene);
   colours now come from `chrome`.
   Subtitles moved from the arrow-tip neighbourhood to the axis midpoint so they don't compete with the capitals.
   `SUBTITLE_LABEL_SIZE_PX` 12 → 10,
   `TIP_LABEL_OFFSET_FRACTION` 0.18 → 0.22,
   `ORIGIN_OFFSET_FRACTION` 0.04 → 0.08.
   Deleted the now-unused `LABEL_BACKGROUND_COLOR` / `AXIS_LABEL_COLOR` / `ORIGIN_LABEL_COLOR` / `NAME_LABEL_COLOR`
   constants.
- `src/deck-layers.ts`:
   accepts `chrome` and reads `axis` and `axisTick` colours from it;
   deleted the `AXIS_SHAFT_COLOR` / `AXIS_TICK_COLOR` constants.
- Test fix:
   `test/state.unit.test.ts` updated the `defaultState` display-toggle assertion to expect `showThresholdPlanes: false`
   (it was the only test referencing the flipped default).

Lint,
 types,
 and tests all clean (0 errors;
 ~471 stylistic warnings,
 all non-blocking).
 8 PASS / 0 FAIL / 1 SKIP across the eight test files.
 The CLI runs end-to-end and produces a fresh `dist/deps-cube-2026-05-13T02-18-53Z.html` (~822KB).

Pending:
 manual Firefox interaction verification;
 open the new HTML and confirm all three green planes render at every camera angle (drag-rotate to test);
 no brown threshold-guide lines by default;
 axes + capitals + origin in light gray on dark mode (or dark gray on light mode);
 subtitles in 10px at axis midpoints;
 no white-rectangle label backgrounds.

**Status (2026-05-12,
 eleventh handover;
 glyph-scale + per-glyph name labels)**:
 with iteration-2's coordinate-system backdrop reading correctly,
 the user asked for two further tweaks:
 shrink the spheres / octahedra by 0.5× so they no longer obscure each other in dense regions,
 and label every glyph (including the unknown-cluster spheres) with its npm name by default.
 Implemented:

- `src/deck-accessors.ts`:
   `RADIUS_MIN_WORLD_FRACTION` 0.005 → 0.0025 and `RADIUS_MAX_WORLD_FRACTION` 0.03 → 0.015;
   halved diagonal fractions used by `probeRadiusWorld` so every mesh-layer glyph renders at half its previous
   world-space size.
- `src/script/state.ts`:
   `DEFAULT_DISPLAY_TOGGLES.nameLabels` flipped from `'none'` to `'all'`.
   The radio control still offers `none` / `topN` / `all`;
   the default is just no longer hidden.
- `src/deck-labels.ts`:
   `buildNameLabelsLayer` no longer filters out probes with `unknownReason !== null` or null `probePosition`.
   For those,
   the per-glyph label position is sourced from `unknownClusterPosition` (the same accessor `buildUnknownClusterLayer`
   uses),
   so the upper-right unknown bucket also gets its names painted alongside the spheres.
   New import:
   `unknownClusterPosition` from `./deck-accessors.ts`.
- Test fix:
   `test/state.unit.test.ts` assertion updated to expect `nameLabels: 'all'` and the test name extended to mention name
   labels.

Lint,
 types,
 and tests all clean (0 errors;
 471 stylistic warnings;
 8 PASS / 0 FAIL / 1 SKIP).
 Fresh HTML at `dist/deps-cube-2026-05-13T02-28-46Z.html` (~822KB;
 no size delta from iteration-2 since the bundle contents are unchanged).

**Status (2026-05-12,
 fifteenth handover;
 both orientations per texture)**:
 iteration-6 added a single canvas-Y flip on the assumption that luma.
gl would upload with `flipY: false`.
 luma.
gl's source does default to `flipY: false`,
 but the empirical result on the sphere was still inverted text;
 likely because the equirectangular UV mapping combines with the orientation of the canvas in a way the single flip
 didn't undo.
 Instead of chasing the math,
 the user proposed a guaranteed-correct workaround:
 print BOTH an upright and a rotated-180° copy on the same texture,
 so at least one is readable from any rotation.
 That is now the implementation in `src/deck-textures.ts`:

- Sphere textures get two stripes at `v = 0.35` (upright) and `v = 0.65` (rotated 180°),
   each with two horizontal repetitions at `u = 0.25 / 0.75`.
   Four text instances per texture;
   the viewer sees at least one right-side-up copy on the visible hemisphere regardless of camera angle.
- Octahedron textures get two stripes inside the UV face triangle at `v = 1/4` (upright) and `v = 1/2` (rotated 180°).
   Each of the 8 faces shows both orientations,
   so a glance at any face yields a readable name.
- `paintFlippedName` (the previous `scale(1, -1)` helper) replaced by paired `paintUpright` (no transform) and
- `paintRotated180` (`translate / rotate(PI)`);
   the rotation keeps the glyph centred under its anchor point without the Y-mirror artefacts that `scale(1, -1)`
   introduces on some glyph features.
- Constants renamed:
   `SPHERE_EQUATOR_V` → `SPHERE_UPRIGHT_V` + `SPHERE_FLIPPED_V`;
   `OCTAHEDRON_CENTROID_V` → `OCTAHEDRON_UPRIGHT_V` + `OCTAHEDRON_FLIPPED_V`.

Lint,
 types,
 and tests all clean (0 errors;
 8 PASS / 0 FAIL / 1 SKIP).
 Fresh HTML at `dist/deps-cube-2026-05-13T03-02-47Z.html`.

**Status (2026-05-12,
 fourteenth handover;
 texture fixes:
 vertical flip + auto-shrink + 2-repeat)**:
 iteration-6 added a single canvas-Y flip on the assumption that luma.
gl would upload with `flipY: false`.
 luma.
gl's source does default to `flipY: false`,
 but the empirical result on the sphere was still inverted text;
 likely because the equirectangular UV mapping combines with the orientation of the canvas in a way the single flip
 didn't undo.
 The previous attempt's contents follow.

**Status (2026-05-12,
 fourteenth handover;
 texture fixes:
 vertical flip + auto-shrink + 2-repeat)**:
 iteration-5 baked the names but two artefacts showed up:
 text rendered upside-down on the camera-facing side of every sphere,
 and at 4 repetitions on a 512 px texture long names like `happy-rusty` overflowed their 128 px slot and overlapped.
 Both fixed in `src/deck-textures.ts`:

- Vertical flip:
   text is drawn with a `ctx.save() / translate / scale(1, -1) / draw / restore()` wrapper.
   Reason:
   luma.
  gl's `SphereGeometry` writes `texCoord_v = 1 - latitude` (north pole at v=1,
   south pole at v=0) while WebGL's default canvas upload puts canvas-top at v=0.
   Without the flip,
   text drawn at canvas-middle ends up with its top half south-of-equator on the sphere,
   reading upside-down to anyone looking at the equator from outside.
   With the flip,
   the text renders right-side-up.
   Applies to the octahedron path too (faces are orientation-symmetric,
   so the flip never makes things worse).
- 2 repetitions instead of 4;
   slot width 256 px,
   room for a wider type.
- Auto-shrink:
   a `pickFontSize` helper calls `ctx.measureText` once at the max font size and rescales proportionally to fit
   `slot * 0.85`,
   clamped to `[MIN_FONT_SIZE_PX, FONT_SIZE_PX] = [22, 56]`.
   The line width of the outline is scaled to match (`OUTLINE_WIDTH_PX * fontSize / FONT_SIZE_PX`) so the rim stays
   visually consistent.
- Cache key already includes the name so the auto-shrink result is captured automatically;
   nothing to change there.

Lint,
 types,
 and tests all clean (0 errors;
 500 stylistic warnings;
 8 PASS / 0 FAIL / 1 SKIP).
 Fresh HTML at `dist/deps-cube-2026-05-13T02-54-15Z.html`.

**Status (2026-05-12,
 thirteenth handover;
 names baked into the mesh textures)**:
 iteration-4's painted labels still used a billboard `TextLayer` with `depthCompare: 'always'`,
 so back-glyph labels rendered on top of nearer glyphs (clearly visible in a Firefox screenshot where a
 back-octahedron's name appeared larger than the front octahedron occluding it).
 The user identified the fix:
 "If you made the text part of mesh this would not happen.
 Are you able to bake text into mesh?
" Yes;
 and that is now the implementation.
 The floating `TextLayer` is gone;
 each probe gets its own `SimpleMeshLayer` with a per-probe canvas texture that has the colour and the npm name baked
 in.
 Depth testing then handles occlusion the same way it does for any other mesh surface;
 a sphere behind a sphere shows only the parts not occluded.

- New `src/deck-textures.ts`:
   `makeProbeTexture({ probe, fillColor, shape, withName })` renders a 512×512 canvas:
   fill background with the probe's data-derived colour,
   then (when `withName`) draw the npm name in bold sans-serif 56 px white with a black 6 px outline.
   Two UV layouts:
  - `'sphere'`:
     name repeated 4× along the equator (`v = 0.5`) so the equirectangular wrap shows the label from any longitude.
  - `'octahedron'`:
     name once at the face triangle's centroid (`v = 1/3` under the UV layout `(0,0),(1,0),(0.5,1)` that every face
     shares).

  Textures cached by `(catalogKey, rgba, shape, withName)` so repeated renders reuse the same canvas (and the same GPU
  upload).
- `src/deck-geometries.ts`:
   octahedron now writes a UV triangle `(0,0),(1,0),(0.5,1)` per face (was all-zero `TEXCOORD_0`).
   The same UV patch on every face means the baked-name texture appears identically on each octahedron facet.
- `src/deck-scatter.ts` rewritten;
   each factory now returns `readonly Layer[]` with one `SimpleMeshLayer` per probe rather than a single layer holding
   the whole bucket.
   Per-layer `opacity` (0.05 vs 1) supplies the filter-fade effect (since `texture` overrides `getColor` in deck.
  gl's SimpleMeshLayer,
   per-instance alpha is no longer available).
   Lighting stays on by default so the meshes keep their 3D shading.
- New `src/deck-scatter-helpers.ts`:
   `partitionProbes` (leaf / non-leaf / unknown bucketing) and `computeNameBakeSet` (which `originalIndex` values get a
   baked name under the current `nameLabels` toggle) extracted so the main scatter file stays under the 300-line cap.
- `src/deck-labels.ts`:
   `buildNameLabelsLayer` removed entirely (the corresponding visual is now baked into the textures);
   module docstring updated;
   unused `nameLabel` / `probePosition` / `unknownClusterPosition` / `PackageProbe` / `AppState` imports + the
   `TOP_N_NAMES` / `NAME_LABEL_SIZE_PX` / `NAME_LABEL_OFFSET_FRACTION` constants + the painted-label fill / outline
   constants are all gone.
- `src/deck-config.ts` `buildLayers`:
   drops the `nameLabels === 'none'` branch (no separate name-label layer to gate);
   flattens the three scatter factories' `readonly Layer[]` returns directly into the layer groups.

Per-probe layers (≈ 117 at the current catalog size) trade extra draw calls for correct depth-of-text rendering.
 A texture-atlas + per-instance UV transform extension would consolidate back to three draw calls but is not worth the
 complexity at this scale.

Lint,
 types,
 and tests all clean (0 errors;
 495 stylistic warnings;
 8 PASS / 0 FAIL / 1 SKIP).
 Fresh HTML at `dist/deps-cube-2026-05-13T02-48-13Z.html` (~824KB).

**Status (2026-05-12,
 twelfth handover;
 names painted on the balls)**:
 iteration-3 floated the names just above each glyph as a HUD label;
 the user asked for them to feel painted onto the balls themselves (basketball-with-team-logo metaphor).
 Implemented in a single file:

- `src/deck-labels.ts` `buildNameLabelsLayer`:
  - Position is the glyph center directly (no `+ nameOffset`).
     The `NAME_LABEL_OFFSET_FRACTION` constant is gone.
  - `getColor` hardcoded to white `[255, 255, 255, 255]` instead of `chrome.nameLabel`;
     the SDF outline supplies contrast on any ball colour,
     so OS-scheme adaptation is no longer needed for this specific layer.
     Other label layers (capitals,
     subtitles,
     origin) still draw from `chrome` since they sit against the scene backdrop,
     not on opaque glyphs.
  - `chrome` param dropped from the function signature;
     `deck-config.ts` `buildLayers` no longer forwards it for name labels (other factories untouched).
  - Painted-letter look:
     `fontFamily: 'sans-serif'`,
     `fontWeight: 700`,
     `fontSettings: { sdf: true }`,
     `outlineColor` pure black,
     `outlineWidth: 3` (deck.
    gl-relative units;
     produces a ~10 to 15 % rim around the white fill at 11 px).
  - Depth override:
     `parameters: { depthCompare: 'always' }` so the billboard text sits on top of the opaque sphere/octahedron rather
     than being half-occluded by the mesh's front face.
     Verified via cloned deck.
    gl source at `modules/extensions/src/terrain/terrain-pass.ts:90` and `modules/core/src/passes/screen-pass.ts:36`
    that `depthCompare: 'always'` is the WebGPU-style key deck.
    gl 9 / luma.
    gl v9 layers accept on `parameters`.

`src/deck-config.ts` `buildLayers` updated call site (one less prop on `buildNameLabelsLayer`).

Lint,
 types,
 and tests all clean (0 errors;
 473 stylistic warnings;
 two more than the iteration-3 baseline,
 from the new SDF/outline declarations).
 8 PASS / 0 FAIL / 1 SKIP.
 Fresh HTML at `dist/deps-cube-2026-05-13T02-35-01Z.html` (~823KB).

**Last commits** (most recent first):

- `c2a2079a feat(dev-script/deps-cube): interpolate probe colour in OKLCH`
- `80a5f175 feat(dev-script/deps-cube): pin camera-controls guide to top of sidebar`
- `677af470 fix(dev-script/deps-cube): paint upright AND rotated-180° copies per texture` (iteration 7)
- `bd6e6049 fix(dev-script/deps-cube): flip texture Y + auto-shrink long names + 2 reps` (iteration 6)
- `62d80159 feat(dev-script/deps-cube): bake names into per-probe mesh textures` (iteration 5)
- `059201f2 feat(dev-script/deps-cube): paint name labels onto each ball/octahedron` (iteration 4)
- `217ffab5 feat(dev-script/deps-cube): halve glyph size and label every glyph by default` (iteration 3)
- `21b62437 feat(dev-script/deps-cube): all 3 coordinate planes via SolidPolygonLayer + scheme-aware chrome` (iteration
- 2)
- `220818cb feat(dev-script/deps-cube): 3D mesh glyphs + coordinate-system backdrop + dual-thumb sliders` (iteration 1)
- `5d552cdf fix(dev-script/deps-cube): use ISO 8601 UTC down to seconds in output filename`
- `dbd34c10 feat(dev-script/deps-cube): tsdown build with bin pointing to dist/final/node/cli.mjs`
- `594ce226 fix(dev-script/deps-cube): walk up to package.json instead of hardcoded`..`for dist resolution`
- `b0364026 fix(dev-script/deps-cube): deep-clone defaults in defaultState to prevent shared-reference corruption`
- (task 13 fix 3)
- `775efcf1 fix(dev-script/deps-cube): pass partial-unknowns through range filter by default` (task 13 fix 2)
- `9376aff8 fix(dev-script/deps-cube): anchor output to <package>/dist/ via import.meta.dirname` (task 13 fix 1)
- `77656a03 test(dev-script/deps-cube): add eight unit-test files for task 12` (task 12)
- `cfa2d156 docs(dev-script/deps-cube): add depth-matched library audit for deck.gl` (task 11)
- `c37ee14c feat(dev-script/deps-cube): add HTML composer + CLI entry` (task 10)
- `a496f91c feat(dev-script/deps-cube): add browser-side scene controller` (task 9)
- `d8142cba feat(dev-script/deps-cube): add HTML control panel renderer` (task 8)
- `df8f7fb3 docs(dev-script/deps-cube): handover update; tasks 6 to 7 done, 8 to 13 pending` (second handover)
- `c2ce8e45 feat(dev-script/deps-cube): add deck.gl config + layer factories` (task 7)
- `a0609584 feat(dev-script/deps-cube): add browser-side filter + state modules` (task 6)
- `d6d1aae6 docs(dev-script/deps-cube): handover doc covering tasks 1-5 done, 6-13 pending` (first handover)
- `7c57662e feat(dev-script/deps-cube): scaffold package and probe pipeline` (tasks 1 to 5)

## Task tracker (matches TaskList IDs 1 to 13)

Done:

- ~~Task 1 (scaffold)~~:
   `package.json` (bin `deps-cube` → `src/cli.ts`,
   deps on `@deck.gl/core`,
   `@deck.gl/layers`,
   `string-dedent`,
   `@monochromatic-dev/module-es`,
   `@monochromatic-dev/module-pnpm-workspace-catalog`,
   `nano-spawn`,
   `p-limit`),
   `mise.toml`,
   `tsconfig.json`,
   `README.md`.
- ~~Task 2 (catalog entry)~~:
   `pnpm-workspace.yaml` carries `@deck.gl/core: '>=9.3.2'` and `@deck.gl/layers: '>=9.3.2'`.
- ~~Task 3 (catalog parser)~~:
   `package/module/pnpm-workspace-catalog` owns YAML parsing,
   validation,
   prototype-safe maps,
   and located-file discovery.
   `src/catalog.ts` adapts its raw default-plus-named entries into the
   resolved `CatalogEntry` shape;
   alias decoder remains exported as `decodeAlias`.
- ~~Task 4 (cache)~~:
   `src/cache.ts` exports `createCache({ rootDir? })`.
   JSON file per (name,
   version) at `~/.cache/monochromatic/deps-cube/<name>/<version>.json`;
   per-field `{ value, fetchedAt }`;
   atomic tmp+rename writes.
- ~~Task 5 (probe)~~:
   `src/probe.ts` (orchestrator) + `src/probe-fields.ts` (per-field probes + helpers) + `src/probe-transitive.ts`
   (depth-bounded dep walk).
   Failed entries return a stub via `failedProbe` with `unknownReason: 'private-or-404'`.
- ~~Task 6 (filter + state)~~:
   `src/script/filter.ts` exports `computeVisibleIndices`,
   `extractDim`,
   `derivedBool`,
   `searchMatches`,
   plus the type vocabulary (`ToggleKey`,
   `ToggleValue`,
   `ToggleState`,
   `DataDimKey`,
   `ChannelKey`,
   `DimMapping`,
   `RangeState`).
   `src/script/state.ts` exports `defaultState({ probes })`,
   `encodeState`/`decodeState` (URL-encoded JSON,
   no base64),
   `readStateFromHash`/`writeStateToHash`,
   plus the `AppState`/`ViewState`/`DisplayToggleState` types.
   Both files are pure functions,
   no DOM,
   browser-bundle-safe.
- ~~Task 8 (HTML control panel)~~:
   `src/render-controls.ts` exports `renderControls({ probes, state })` returning the full `<aside id="controls">`
   fragment:
   6 dim `<select>` dropdowns (with channel-incompatible options rendered `disabled`),
   7 three-state radio `<fieldset>`s,
   6 range-slider pairs (`min`/`max` from full data extent;
   `value` from current state),
   search input with escaped value,
   4 display checkboxes + `name-labels <select>`,
   visibility counter,
   reset button.
   Private helpers:
   `renderDimDropdown`,
   `renderToggleRow`,
   `renderRangeRow`,
   `renderDisplaySection`,
   `escapeAttr`,
   `computeChannelExtent`.
   Shared metadata extracted to `src/dim-meta.ts` (`DIM_DISPLAY_NAMES`,
   `DIM_KINDS`,
   `CHANNEL_ACCEPTED_KINDS`,
   `TOGGLE_LABELS`,
   `acceptsDim`);
   `deck-labels.ts` now imports `DIM_DISPLAY_NAMES` from there.
   Channel acceptance:
   `x`/`y`/`z`/`color` accept all kinds (renderer normalises gracefully);
   `shape` accepts binary+categorical;
   `size` accepts continuous only.
- ~~Task 9 (browser-side controller)~~:
   split across four files to stay under the line cap:
  - `src/script/controller.ts`:
     entry point.
     Reads `globalThis.__PROBES__`,
     runs `defaultState` overlaid with URL-hash state,
     instantiates
     `new Deck<OrbitView>({ views: orbitView, initialViewState, controller: true, layers, getTooltip, onClick })`,
     then wires `onViewStateChange` via `setProps` (deferred to avoid the `session` forward reference).
     Owns the `Session` type,
     `recomputeVisibility` / `rerenderLayers` / `syncHash` render-path helpers,
     and the `pickedProbe` runtime check for `info.object`.
     `start()` builds the session,
     runs `syncDomFromState`,
     defines a `commit` closure capturing `session` + `probes`,
     then calls every `wire*` function with that closure.
  - `src/script/controller-events.ts`:
     six `wire*` functions (`wireDimDropdowns`,
     `wireToggles`,
     `wireRanges`,
     `wireSearch`,
     `wireDisplay`,
     `wireReset`).
     Each registers DOM listeners that mutate `session.state` in place and call `commit()` (no args).
     Declares its own `Session` shape *without* the `deck` field,
     so the controller's full `Session` is structurally assignable with no cast.
     Dim swap also recomputes scene bounds and resets the channel's slider min/max/value to the new dim's extent.
  - `src/script/controller-dom.ts`:
     `el` / `elInput` / `elSelect` typed accessors using `instanceof` (no `as HTMLInputElement` casts at call sites),
     plus `syncDomFromState` which writes every state value back into the DOM after a reset / URL-hash overwrite.
  - `src/script/controller-tooltip.ts`:
     `formatTooltipHtml` (10-row tooltip table with HTML-escaped probe fields) + lazy-initialised
     `<aside id="pinned-tooltip">` with a close button,
     exposed as `pinTooltip` / `unpinTooltip`.
     Used by the deck.
    gl `getTooltip` hover callback (transient) and the `onClick` handler (pinned).
  - **`src/script/state.ts` change**:
     `ViewState.target` is now mutable `[number, number, number]` instead of `readonly`,
     because deck.
    gl's `OrbitViewState.target` requires mutable.
  - **Bundle smoke test**:
     `rolldown src/script/controller.ts --format=iife --minify` produces a 754KB bundle (513 modules,
     including `@luma.gl/core` transitives).
     About 2× the audit estimate (~400KB);
     the extra comes from luma.
    gl + math.
    gl + probe.
    gl.
     Worth recording in `doc/decision/deps-cube.md` under "implementation notes" when task 11 lands.
- ~~Task 7 (deck.
  gl config + layer factories)~~:
   split across current layer modules:
  - `src/deck-config.ts`:
     `orbitView`,
     `computeSceneBounds`,
     and `buildLayers` orchestrator.
  - `src/deck-accessors.ts`:
     pure per-probe accessors for position,
     OKLCH colour,
     radius,
     and shape bucket.
  - `src/deck-layers.ts`:
     axis shafts,
     arrowhead cones,
     and tick marks.
  - `src/deck-planes.ts`:
     coordinate planes via `SolidPolygonLayer` plus threshold guide lines via `PathLayer`.
  - `src/deck-scatter.ts`:
     per-probe `SimpleMeshLayer` spheres and octahedra with baked canvas textures.
  - `src/deck-scatter-helpers.ts`:
     partitioning and top-N name-bake selection.
  - `src/deck-labels.ts`:
     axis capitals,
     axis subtitles,
     and origin label.
  - `src/deck-textures.ts`:
     per-probe texture baking for colours and package names.
  - `src/oklch.ts`:
     OKLCH-to-sRGB conversion for the colour ramp.

- ~~Task 10 (HTML composer + CLI)~~:
   three files plus a CSS asset:
  - `src/render-html.ts`:
     `renderHtml({ probes })` builds the final document:
     uses rolldown with `format: 'iife'` and `minify: true` to obtain the controller IIFE,
     inlines `globalThis.__PROBES__ = <json>` ahead of the controller script,
     embeds the control-panel fragment from `renderControls`,
     and wraps everything in `<!doctype html>...</html>` with the CSS from `styles.css` in a `<style>` block.
     Helpers:
     `bundleController` (throws when `result.success === false`,
     joining `result.logs.map(log => log.message)`),
     `escapeForScriptTag` (replaces `</script` → `<\/script` and `<!--` → `<\!--` so the inline JS can't escape the
     script tag).
     Empty-probes smoke test:
     770KB output,
     513 modules,
     document terminates cleanly with `</html>`.
  - `src/cli.ts`:
     `#!/usr/bin/env node` entry;
     top-level `await readCatalog()` → `createCache()` → `await probeAll({ entries, cache })` →
     `await renderHtml({ probes })` → `mkdir(distDir, { recursive: true })` → `writeFile(absPath, html, 'utf8')` →
     ``console.log(`Saved to ${absPath}`)``.
     `currentRunOutputFilename()` returns `deps-cube-<YYYY-MM-DDTHH-MM-SSZ>.html`:
     ISO 8601 UTC down to seconds,
     with `:` rewritten as `-` for filesystem safety;
     each invocation gets a distinct artifact and re-runs do not overwrite.
     Output is anchored to `<package>/dist/` via `PACKAGE_ROOT` (from `./find-package-root.ts`,
     walking up to this package's own `package.json`),
     not `process.cwd()`:
     the audit is per-monorepo,
     not per-cwd,
     and the package-root walk keeps the path stable in both source mode and built mode (`dist/final/node/cli.mjs`).
     Each top-level `const` carries its own TSDoc (required at module root).
  - `src/index.ts`:
     re-exports the library surface (`readCatalog`,
     `decodeAlias`,
     `CatalogEntry`;
     `createCache`,
     `Cache`;
     `probeAll`,
     `PackageProbe`,
     `LicenseClass`,
     `UnknownReason`;
     `renderHtml`;
     `renderControls`).
     The CLI in `cli.ts` is bin-only and not re-exported.
  - `src/styles.css`:
     page CSS imported via `with { type: 'text' }`.
     Native nesting (3 levels max),
     logical properties (`inline-size`,
     `padding-inline`,
     `border-inline-start-*`),
     `rem` sizing,
     `:focus-visible` on every interactive element,
     `min-block-size: 3rem` touch targets,
     design-token custom properties at `:root` with `prefers-color-scheme: dark` overrides.
     No `border`/`padding`/`margin` shorthands;
     only single-axis (`padding-block`,
     `padding-inline`) or single-concept (`border-radius`) shorthands;
     sided borders use longhand `border-block-start-width` / `-style` / `-color`.
  - `src/css.d.ts`:
     ambient `declare module '*.css'` shim so TypeScript types the text import as `string` (mirrors the existing
     `svg.d.ts` shim in `inference-canary-viewer`).
  - **Layout**:
     HTML body is `display: flex` with `<main id="canvas-host">`
     (`flex-grow: 1; position: relative; min-block-size: 100vh`) and `<aside id="controls">` (`inline-size: 22rem`)
     side-by-side.
     The deck.
    gl canvas is supplied as `<canvas id="deck-canvas">` inside the main element;
     the controller passes `canvas: 'deck-canvas'` to `new Deck<OrbitView>({...})` so deck.
    gl uses the pre-existing canvas instead of creating its own attached to `document.body`.
     **Controller change**:
     `src/script/controller.ts` `createSession` now passes `canvas: 'deck-canvas'` to the `Deck` constructor (no other
     changes).

Done (continued):

- ~~Task 11 (decision doc)~~:
   repo-root `doc/decision/deps-cube.md` follows the pattern of the existing decision docs (`font-subsetting.md`,
   `readable-stream-shim.md`):
   Context,
   Decision with the TS-ratio exception (attributed to the user's screenshot-confirmed authorization,
   not an independent re-derivation),
   Rejected alternatives (plotly.
  js / three.
  js / echarts / d3-3d / @thi.
  ng/* / @nivo-visx-recharts / hand-rolled,
   each with the specific gate they fail),
   Audit notes (transitive surface enumerated package-by-package with license + module-type,
   build provenance via vis.
  gl's `ocular-bundle`,
   maintenance signals re-verified at audit time via `gh api repos/visgl/deck.gl` + `npm api`),
   Implementation notes (2x bundle weight vs plan estimate explained by the transitive cone,
   mesh-glyph migration,
   pre-existing canvas wiring choice,
   top-N name-bake TODO,
   monorepo-housed packages routed to Unknown cluster).
   Numbers in the doc come from `gh api repos/visgl/deck.gl/{languages,contributors}` (TS 72.56%,
   contributors 277+) and `https://api.npmjs.org/downloads/point/last-week/` (~630K/week for `@deck.gl/core` and
   `@deck.gl/layers`),
   measured 2026-05-12.
   No em-dashes,
   no en-dashes,
   no tables,
   lines under 120,
   sentence-case headings,
   ATX max 3 levels.

Done (continued):

- ~~Task 12 (unit tests)~~:
   nine `test/*.unit.test.ts` files:
   `cache`,
   `catalog`,
   `filter`,
   `state`,
   `render-controls`,
   `render-html`,
   `probe`,
   `deck-config`,
   and `oklch`.
   Each is self-contained with local fixtures,
   uses the `@monochromatic-dev/module-test` harness (`describe({ name, children: [it(...)] })`),
   and creates temp directories under `os.tmpdir()` with an `await using` disposable for cleanup.
   The mise task wiring uses the root template (`[tasks.'test:unit'] extends = "test:unit"`),
   which runs `node <file>` per `*.unit.test.ts` in parallel via the root `task_templates."test:unit"`.
   The `CLAUDE.md` rule "Never invoke raw tools (`bun test`,
   `oxlint`,
   etc.) directly;
   use the corresponding mise task" caught this;
   the package's earlier `[tasks.test] run = "bun test"` definition was replaced with the template extension.

  Run via `mise run //package/dev-script/deps-cube:test:unit`.
   Reports 9 PASS lines,
   0 FAIL,
   1 SKIP.
   Coverage per file:

  - **cache**:
     read miss;
     round-trip;
     TTL semantics (null = never expire,
     negative = always expire);
     multiple fields coexisting in one file;
     rootDir exposure;
     parent-directory auto-creation for scoped names (`@scope/pkg`);
     malformed JSON treated as miss.
  - **catalog**:
     `decodeAlias` for plain ranges,
     `npm:` aliases (scoped/unscoped,
     with/without `@range`);
     `readCatalog` for default + named blocks;
     `npm:` decoding inside catalog entries;
     throw paths for missing yaml and empty catalog blocks.
     Uses an upward search constrained to a temp dir so the test doesn't accidentally pick up the real
     `pnpm-workspace.yaml` higher in the tree.
  - **filter**:
     `extractDim` log scaling + null pass-through;
     binary/categorical numeric mapping;
     `derivedBool` for every toggle key including unknown-input branches;
     `searchMatches` substring (case-insensitive),
     `/regex/` form,
     malformed-regex fallback;
     `computeVisibleIndices` for all-any baseline,
     single-toggle filters,
     composed audit-target pattern (non-TS + leaf + stale + permissive),
     name search,
     narrow range.
  - **state**:
     `defaultState` produces the plan dim mapping,
     "any" toggles,
     and ranges matching `Math.min/max(...extractDim values)` per channel (this indirectly covers the same extent logic
     `computeSceneBounds` runs at render time);
     `encodeState`/`decodeState` round-trip;
     malformed-JSON / missing-keys fallback paths;
     `readStateFromHash` empty / corrupt / round-trip.
  - **render-controls**:
     structural counts (6 dim rows,
     7 toggle rows × 3 radios,
     6 range rows × 2 sliders);
     presence of every required id (`dim-<channel>`,
     `range-<channel>-<min|max>`,
     `search`,
     `display-*`,
     `name-labels`,
     `visibility-counter`,
     `reset`);
     visibility counter starts at `N of N visible`;
     `shape` dropdown disables continuous options (`tsRatio`,
     `logSourceBytes`) while leaving `isLeafNumeric` selectable;
     search-input value is HTML-attribute escaped (`"><script>` doesn't survive).
  - **render-html**:
     stubs rolldown bundling with a fake bundle result returning a fake bundle.
     Asserts the composed document starts with `<!doctype html>`,
     has `<meta charset>`/`<meta viewport>`/`<title>`,
     contains no external `<link rel="stylesheet">` or `<script src=...>` references,
     inlines the probe array as `globalThis.__PROBES__`,
     embeds the stubbed bundle text,
     and includes `<style>...</style>` plus the control panel + canvas.
     Verifies `</script` and `<!--` are neutralised inside both the data and the bundle text;
     verifies the failure path joins all bundler logs into the thrown error.
     **`concurrency: 1`** on this describe block;
     the bundler stub is process-global,
     so concurrent stubs can trip sinon's "already wrapped" guard.
  - **probe**:
     pure helpers (`parseRepository` for plain URL / `git+https` / `github:` shorthand / object with `directory` /
     non-GH / undefined / empty;
     `classifyLicense` for every class;
     `resolveVersion` for pinned vs range);
     `probeAll` exercised against a pre-populated file cache covering every `UnknownReason` branch (`null` known,
     `no-repo`,
     `non-github`,
     `monorepo`,
     `private-or-404`).
     The `private-or-404` case uses `sinon.stub(globalThis, 'fetch').rejects(...)` so the field probe's manifest fetch
     fails and the orchestrator emits a `failedProbe` stub via the catch in `probeAll`.
  - **deck-config (accessors)**:
     covers the importable `src/deck-accessors.ts` helpers:
     `probePosition` (known + null spatial);
     `unknownClusterPosition` (offsets beyond bounds,
     deterministic per index,
     distinct per index);
     `probeFillColor` (visible=255 / filtered=13 alpha,
     grey for unknown colour dim,
     red↔green ramp);
     `probeRadius` (in `[3, 30]`px,
     minimum for null/zero size dim);
     `probeIsFilled` (false for leaves,
     true for non-leaves under the default shape mapping,
     hollow for unknown shape dim).
  - **oklch**:
     covers OKLCH conversion,
     endpoint interpolation,
     midpoint amber colour,
     and out-of-gamut clamping.

  **Layer-count snapshot from the plan is blocked upstream**:
   importing `src/deck-config.ts` at test runtime fails because `@loaders.gl/schema-utils@4.4.1` statically imports
   `@math.gl/types` without declaring it as a dependency,
   so Node's module-load resolver can't find the latter.
   The rolldown bundle path tree-shakes the unreachable path,
   so the produced HTML still works.
   The `deck-config.unit.test.ts` file has a single `it({ skip: '<reason>' })` placeholder so the limitation surfaces
   in test output.
   Upstream fix path:
   add `@math.gl/types` to `@loaders.gl/schema-utils`'s `package.json` dependencies (track via `visgl/loaders.gl` repo
   when revisiting).

  Sinon stubbing of global namespace methods (`globalThis.fetch` and bundler seams) requires
  `oxlint-disable typescript-eslint/no-unsafe-call` and `no-unsafe-member-access` because sinon's overload set doesn't
  unify with the fetch global and bundler seam types.
   The disables are wrapped tightly around the `sinon.stub(...).resolves/rejects(...)` lines per the `AGENTS.md`
   disable-comment rule.

Partial:

- **Task 13** (in progress):
   end-to-end smoke run.
   Headless verification complete via `agent-browser` against the most recently written
   `file://.../dist/deps-cube-<YYYY-MM-DDTHH-MM-SSZ>.html`:
  - CLI exits with one line of stdout:
     `Saved to <abs-path>` ✓
  - HTML is self-contained (no `<link rel="stylesheet">`,
     no `<script src=…>`) ✓
  - 115 probes inlined;
     visibility counter reads `115 of 115 visible` on first load ✓ (was 61 of 115 before the filter fix)
  - Zero browser-console errors and zero page errors during deck.
    gl init ✓
  - Search filtering works (`react` → 2 of 115;
     `etag` → 0 of 115,
     no etag-style packages in this catalog) ✓
  - 3-state toggle composition for the audit-target pattern (`tsMajority=no` + `isLeaf=yes` + `recent=no`) narrows to 1
  - probe:
     `mitata` ✓
  - Display toggles flip state (`#display-planes` checked → unchecked without error) ✓
  - Reset button restores defaults across all radios + ranges + search ✓ (was broken before the defaultState clone fix)
  - URL hash encodes state (`#state=…`,
     ~1.1KB encoded) ✓
  - `hasKnownRepo='yes'` toggle hides partial-unknowns (`93 of 115`,
     matching the 22 probes with non-null `unknownReason`) ✓
  - Re-run from cache completes in <2s ✓

  Manual Firefox verification still needed for the interactive 3D pieces that headless tools cannot exercise
  meaningfully:
   drag-rotate,
   shift-drag pan,
   scroll-zoom,
   double-click reset of the camera;
   visual confirmation of glyph positions,
   coordinate planes,
   threshold guide lines,
   and partial-unknown rendering;
   hover-tooltip and click-pinned-tooltip behaviour.
   Open the most recent `package/dev-script/deps-cube/dist/deps-cube-*.html` in Firefox ESR 140+ and walk steps 4,
   10,
   11,
   13,
   16,
   17 of the plan's verification checklist.

## State on disk (verified before this handover)

```text
package/dev-script/deps-cube/
├── HANDOVER.implementation-state.md   ← this file
├── README.md
├── mise.toml
├── package.json
├── tsconfig.json
├── src/
│   ├── cache.ts                 ← JSON file cache, per-key TTL, atomic writes
│   ├── catalog.ts               ← shared-reader adapter + alias decode
│   ├── cli.ts                   ← #!/usr/bin/env node; readCatalog → probeAll → renderHtml → writeFile
│   ├── css.d.ts                 ← ambient `declare module '*.css'` shim for text imports
│   ├── deck-accessors.ts        ← per-probe pure accessors (position/color/radius/shape)
│   ├── deck-config.ts           ← orbit view + computeSceneBounds + buildLayers orchestrator
│   ├── deck-labels.ts           ← axis capitals/subtitles + origin TextLayers
│   ├── deck-layers.ts           ← axis shafts, arrowhead cones, tick marks
│   ├── deck-planes.ts           ← coordinate planes + threshold guide lines
│   ├── deck-scatter.ts          ← per-probe SimpleMeshLayer spheres/octahedra
│   ├── deck-scatter-helpers.ts  ← probe partitioning + name-bake selection
│   ├── deck-textures.ts         ← per-probe canvas texture baking
│   ├── dim-meta.ts              ← shared display names, kinds, channel-acceptance, toggle labels
│   ├── index.ts                 ← library re-exports (readCatalog, createCache, probeAll, renderHtml…)
│   ├── probe.ts                 ← orchestration + PackageProbe type
│   ├── probe-fields.ts          ← per-field probes + helpers (gh + registry)
│   ├── probe-transitive.ts      ← depth-bounded dep walk
│   ├── oklch.ts                 ← OKLCH → sRGB colour conversion
│   ├── render-controls.ts       ← Node-side HTML emitter for the control-panel <aside>
│   ├── render-html.ts           ← composes the final HTML (rolldown bundle + probe literal + controls)
│   ├── styles.css               ← page CSS, native nesting, logical properties, design tokens
│   └── scripts/
│       ├── controller.ts        ← bootstrap, Deck instantiation, render path, pickedProbe, start()
│       ├── controller-dom.ts    ← el/elInput/elSelect typed accessors + syncDomFromState
│       ├── controller-events.ts ← wire* functions for every control surface
│       ├── controller-tooltip.ts← formatTooltipHtml + pinned-tooltip DOM management
│       ├── filter.ts            ← computeVisibleIndices + extractDim + derivedBool + searchMatches
│       └── state.ts             ← AppState, defaultState, URL-hash ser/deser
└── test/
    ├── cache.unit.test.ts        ← read/write/TTL/atomic semantics
    ├── catalog.unit.test.ts      ← decodeAlias + readCatalog over fixture pnpm-workspace.yaml
    ├── deck-config.unit.test.ts  ← deck-accessors (deck-config import blocked by loaders.gl bug)
    ├── filter.unit.test.ts       ← extractDim / derivedBool / searchMatches / computeVisibleIndices
    ├── oklch.unit.test.ts        ← OKLCH conversion + interpolation
    ├── probe.unit.test.ts        ← parseRepository / classifyLicense / probeAll (5 UnknownReason branches)
    ├── render-controls.unit.test.ts ← structural counts + id presence + attribute escape
    ├── render-html.unit.test.ts  ← bundler stub + self-contained-document assertions
    └── state.unit.test.ts        ← defaultState ranges + encode/decode round-trip + hash helpers
```

The split (`probe.ts` / `probe-fields.ts` / `probe-transitive.ts` and `deck-config.ts` / `deck-accessors.ts` /
`deck-layers.ts` / `deck-scatter.ts` / `deck-labels.ts`) is enforced by the `eslint/max-lines: 300` rule.
 Keep new files under that;
 the rule is configured with `skipBlankLines: true, skipComments: true`,
 so raw line counts can exceed 300 as long as code-only lines stay under (verified `render-controls.ts` at 402 raw
 lines passes cleanly).

## Verification before declaring each task complete

After every code change,
 run all five in sequence (none can be skipped):

```sh
mise run //package/dev-script/deps-cube:build
mise run //package/dev-script/deps-cube:lint:types
mise run //package/dev-script/deps-cube:lint:oxlint
mise run //package/dev-script/deps-cube:test:unit
mise run //package/dev-script/deps-cube:run
```

**Never invoke `bun test` directly**:
 the project rule "Never invoke raw tools (`bun test`,
 `oxlint`,
 etc.) directly;
 use the corresponding mise task" applies here.
 The `test:unit` task extends the root `task_templates."test:unit"` which runs `node <file>` per `*.unit.test.ts` in
 parallel;
 the harness (`@monochromatic-dev/module-test`) uses top-level `await describe(...)`,
 so each file is its own test process.
 Bun's built-in `bun test` runner has a separate result accumulator that doesn't track these harness-driven results;
 running through the template is the only way to get an accurate exit code and reporter output.

The first two pass cleanly now.
 oxlint reports ~278 stylistic warnings on the package (mostly `argument-per-line`,
 `tuple-per-line`,
 `array-element-per-line` against multi-arg helpers and tuple literals,
 plus `tsdoc/tag-lines` against doc blocks with adjacent `@param` lines);
 ignore those unless tidying for release.
 The blocking errors caught across sessions were:

- `eslint/prefer-template` (two pre-existing in `catalog.ts` (string concat in error messages)) fixed by switching to
- template literals.
- `tsdoc/check-tag-names`:
   TSDoc reads bare `@anthropic-ai` inside `@example` block as an unknown tag;
   escape with `\@`.
- `eslint/max-lines`:
   fix by splitting;
   never disable.
- `no-restricted-syntax/require-destructured-params`:
   function declarations with 2+ params must use destructured-object form;
   exempt only when the signature is dictated by external APIs (e.g. Array.
  prototype callbacks).
- `typescript-eslint/no-unsafe-type-assertion`:
   the disable comment must be on the line directly preceding the cast;
   if the cast spans multiple lines,
   extract the expression first so the cast is on a single line.

## Notable design constraints already enforced

- **No `let` at function-body root or module root**:
   `probeAll` uses `p-limit` instead of a counter;
   `unknownReason` is a named helper instead of an IIFE.
   Mutating-set patterns (`partitionProbes` push into three arrays) are fine;
   they don't trigger the rule since the lets aren't at the function root.
- **No arrow functions**:
   every callback is a named `function` expression.
   Array-prototype callbacks (`(value, index)`) are exempt from the destructured-object-param rule but still must be
   named functions.
- **No try-finally**:
   `fetchJson` uses `AbortSignal.timeout(30000)`.
   `decodeState` uses try-catch (allowed).
- **exactOptionalPropertyTypes**:
   all optional fields are `field?: T | undefined`.
- **`JSON.parse` / `response.json()` / `Object.fromEntries` casts**:
   surrounded by `// oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- <reason>` per `AGENTS.md`.
   The disable must be on the line directly before the cast;
   if `Object.fromEntries(...) as T` spans multiple lines,
   extract to a `const record` first.
- **deck.
  gl type quirks**:
   `PathGeometry`,
   `PolygonGeometry`,
   and `Position` are mutable in deck.
  gl's typings.
   Datum shapes must use `[number, number, number,][]` not `readonly (readonly [number, number, number,])[]`.
   Accessors that return positions must return mutable tuples too;
   `probePosition` and `unknownClusterPosition` both return mutable types.

## Notes for the next session

- **deck.
  gl bundling**:
   deck.
  gl is installed at `node_modules/@deck.gl/core` and `node_modules/@deck.gl/layers`.
   Don't try to inline the dist as raw text;
   `render-html.ts` uses rolldown with `format: 'iife'` and `minify: true` so rolldown resolves and bundles deck.
  gl transitively from the controller's imports.
   Output goes into a `<script>` block inside the generated HTML.
- **Rolldown log type**:
   `result.logs` is `Array<BuildMessage | ResolveMessage>`.
   Both classes carry a `message: string` field;
   use `log.message` rather than `String(log,)` (the latter triggers `typescript-eslint/no-base-to-string` because the
   type does not declare a custom `toString`).
- **`canvas: string` not `parent`**:
   deck.
  gl's `Deck` accepts either `parent?: HTMLDivElement | null` (creates its own canvas inside the supplied div) or
  `canvas?: HTMLCanvasElement | string | null` (uses the supplied element).
   The `string` form is the id of an existing `<canvas>`:
   simpler than typing through `HTMLDivElement`,
   no `as` cast needed.
   `controller.ts` passes `canvas: 'deck-canvas'`;
   `render-html.ts` emits `<canvas id="deck-canvas">` inside `<main id="canvas-host">`.
- **CSS asset via `with { type: 'text' }`**:
   `styles.css` is imported as a `string` from `render-html.ts`.
   Requires the ambient shim in `css.d.ts` (`declare module '*.css'`) so TypeScript types the import.
   Sibling-package precedent:
   `inference-canary-viewer/src/svg.d.ts` does the same for `.svg`.
- **CSS border rule**:
   `AGENTS.md` bans `border` / `padding` / `margin` shorthands (multi-axis + multi-sub-property).
   Single-axis (`padding-block`,
   `padding-inline`,
   `margin-inline-end`) and single-concept (`border-radius`,
   `inset`,
   `gap`) shorthands are fine;
   sided borders use longhand `border-block-start-width` / `-style` / `-color` (and the equivalent for
   `border-inline-*`).
   `outline` is similar;
   write `outline-width: 2px; outline-style: solid; outline-color: var(...); outline-offset: 2px;`.
- **Plan deviation worth recording in `doc/decision/deps-cube.md`**:
   the chosen visual distinction for the "shape" channel is filled vs stroked (not circle vs diamond).
   Reason:
   ScatterplotLayer renders circles only;
   supporting diamonds means IconLayer with custom icon textures (more code,
   harder to type) or SimpleMeshLayer (3D geometry,
   overkill at 120 points).
   Filled/stroked is the simplest binary distinction that doesn't require auxiliary assets.
   Document under "implementation notes".
- **Display-name source**:
   `src/dim-meta.ts` owns `DIM_DISPLAY_NAMES`,
   `DIM_KINDS`,
   `CHANNEL_ACCEPTED_KINDS`,
   `TOGGLE_LABELS`,
   and the `acceptsDim` predicate.
   Both `deck-labels.ts` (axis labels) and `render-controls.ts` (dim dropdowns + toggle legends) import from here.
   Add new dim-meta there,
   not in either consumer.
- **Top-N name labels**:
   currently ranks by `daysSinceLastCommitOrNull` descending (oldest first).
   Subject to refinement once the audit-target scoring is formalised;
   could be a weighted score across staleness + small size + non-TS + low downloads.
- **Probes with all-zero continuous fields**:
   `failedProbe` returns zeroes (logSourceBytes=0,
   etc.).
   Combined with `Math.max(value, 1)` in `extractDim`,
   these render at `log10(1) = 0` on every spatial axis.
   They'll cluster at the origin if the unknown-cluster routing fails to catch them;
   cross-check by setting `unknownReason: 'private-or-404'` on all failed probes (already done) and ensure
   `partitionProbes` routes them to the unknown bucket via the `probe.unknownReason !== null` check.
- **Linguist `bytes(TypeScript) / sum(bytes)` is whole-repo**:
   monorepo-housed packages (`repository.directory` set) intentionally return `null` for `tsRatioOrNull` and
   `sourceBytesOrNull`:
   the viz must place these in the Unknown cluster region,
   not silently coerce to 0.
   This is already the case.
- **`oxlint-disable-next-line` placement**:
   must be on the line directly before the violation.
   If the violation spans multiple lines (e.g. a multi-line cast),
   extract the expression to a single-line `const` first and put the disable comment on the line directly above that.
   See `state.ts`'s `computeFullRanges` for the pattern.

Plan,
 README,
 this handover,
 and the repo-root audit doc (`doc/decision/deps-cube.md`,
 task 11) are the source of truth.
 The README documents external behaviour;
 this handover documents in-progress state;
 the plan documents the design decisions;
 the audit doc documents the library trade-off.
