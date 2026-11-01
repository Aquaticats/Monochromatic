# IDE text and minimap on the GPU

Status: proposal, measured on 2026-10-19, nothing in `package/ide/app` was changed.
The recommendation needs the owner answers in the last section before any migration stage starts.

## Requirement

The owner decided on 2026-10-19 that the IDE must never paint text itself, then widened it twice the same day:

- Only CPU painting is banned. Vello is not mandatory; FemtoVG, Vello, or another GPU renderer are all acceptable.
- Text does not have to come from Slint's builtin `Text` elements. A custom GPU path outside them is in scope.
- The ban covers every pixel the application produces, not only text: the minimap picture, the overscan tile, any
  pixel buffer the CPU fills and uploads as an image, and any software-renderer path.
  CPU work stays allowed for layout, shaping, highlighting, and building scene or vertex data.

The IDE must stay usable at every stage, and the scroll rule of `doc/planning/slint-ide-scroll-pipeline.md` still
binds: a scroll step must not depend on UI-thread CPU work that can miss a frame.

## How the measurements were made

- Spikes live outside the repository, because they are large (about 75 KB with the drivers):
  `~/temp/agent/ide-vello-spike` (builtin `Text` designs and the capability probe),
  `~/temp/agent/ide-vello-spike2` (the custom GPU design), `~/temp/agent/ide-vello-spike-patched` (the Slint patch
  check), and `~/temp/agent/ide-vello-app` (a copy of the IDE with the renderer features added, no source edit).
  Raw results are JSON lines under `~/temp/agent/ide-vello-runs`.
- Cargo setup: the IDE's `Cargo.lock` was copied, Slint is 1.18.1 from crates.io (the IDE has no git ref or
  `[patch]`). Fetching used the stable cargo; builds ran in the IDE's container image `localhost/monochromatic/ide`
  (bounded: `bin/container-bounds.ts` reports 21 CPUs and 46 GiB; no network at build time).
- Runs happened in the installed nested compositor (`monochromatic-nested-wayland-session` 0.6.1) at 1147x2000,
  scale 1, the size of the owner's drag reproduction, on the real host (AMD Radeon RX 7600; Slint's wgpu adapter
  string was `AMD Radeon RX 7600 (RADV NAVI33)`). Host load was not recorded.
- Cadence: `record DIR 60 bmp` while a `scroll` (120 steps of 16 ms, 3000 px) or a `drag` on the right edge
  (120 steps of 16 ms, 600 px) ran, then a hash of every frame. A "changed" frame differs from its predecessor.
  The recorder dropped no frame in any run. The probe can see a stall: the debug builds and the real IDE under Vello
  show frozen frames, so a null result elsewhere is not a blind probe.
- Cost: a synchronous `take_snapshot` per scroll step (release builds, 120 steps of 40 px). It includes a
  readback of the 1147x2000 image, so absolute values are not frame times; compare cells with the static scene of the
  same renderer (Vello 5.4 to 6.0 ms, FemtoVG 14 to 15 ms). Identical configurations varied by 0.5 ms at most
  (six Vello runs of one cell: 10.48 to 11.03 ms), so smaller differences are noise.
- The code corpus is the owner's 489-line `client.unit.test.ts` (20 characters and 1.92 colored runs per non-blank
  line with the spike's tokenizer) and, for a denser check, the IDE's `viewport.rs` (37 characters per line).
  Real Helix highlighting may yield more runs per line than the spike's tokenizer; element counts scale with that.

## Measured results

### Question 1: does Vello build, start, and draw

- The IDE's Slint 1.18.1 builds with `renderer-vello`. In a copy of the package (container bounded to 21 CPUs; the
  state of the target cache was not recorded), the debug build took 31.5 s and the release build 59.0 s.
  The release binary of the copy with both new renderer features is 100.6 MB; the installed
  `~/.local/bin/monochromatic-ide` (a different build) is 84.5 MB.
- The feature must be named in `Cargo.toml`. With the command-line form `--features slint/renderer-vello`, cargo
  built the vello crates without writing them to `Cargo.lock`, so the build was not locked.
  With the feature in the `slint` dependency, `cargo metadata --offline` grew the lock from 653 to 683 entries:
  `vello` 0.10, `vello_encoding`, `vello_shaders`, `anyrender`, `anyrender_vello`, `i-slint-renderer-anyrender`,
  `peniko`, `color`, `khronos-egl`, `guillotiere`, `pollster` and others, plus a second wgpu family (29.0.4, eight
  entries, next to the 30.0.1 the lock already holds). `renderer-femtovg-wgpu` alone adds 6 entries (653 to 659),
  and design C's set (`renderer-femtovg-wgpu`, `unstable-wgpu-30`, `vello` 0.11 which uses wgpu 30) adds 16 (669).
- Selecting it at run time needs only the feature: `parse_backend_env_var` maps `SLINT_BACKEND=vello` to winit plus
  vello (`i-slint-backend-selector-1.18.1/lib.rs`, the `"vello" => ("winit", "vello")` arm), `winit-vello` works
  the same, `BackendSelector::renderer_name("vello")` is the API form (`api.rs`), and
  `Backend::builder().with_renderer_name("vello")` is what `src/launcher.rs` already forwards for any `winit-*` value.
  Vello is last in the default chain and in the fallback list (`i-slint-backend-winit-1.18.1/lib.rs:151-154`,
  `184-185`), so with FemtoVG enabled it is never chosen implicitly.
- There is no experimental opt-in flag for the renderer. `SLINT_ENABLE_EXPERIMENTAL_FEATURES`
  (`i-slint-compiler-1.18.1/lib.rs:280`) gates `.slint` syntax, not renderers. Vello renders through wgpu 29 and
  refuses any other wgpu selection (`i-slint-backend-winit-1.18.1/lib.rs:1294-1306`).
- It draws under the nested compositor with no compositor change. `SLINT_DEBUG_PERFORMANCE=refresh_lazy,console`
  printed `vello renderer on WGPU (Vulkan backend; adapter: AMD Radeon RX 7600 (RADV NAVI33))`; the compositor's `gpu`
  command answered `device=parent`, renderer `AMD Radeon RX 7600 (radeonsi, navi33, ...)`.
- Frame cadence under the scroll stream, release, 489 lines: the spike changed 119 of 119 frames under both Vello and
  FemtoVG. The real IDE (current CPU tile pipeline) changed 118 of 119 frames under FemtoVG, and 66 of 119 under
  Vello, with a frozen tail of 53 frames and the view ending about half way down the stream. The drag stream:
  FemtoVG 119 of 121, Vello 49 of 121 with a frozen tail of 72 frames. In debug builds the IDE under Vello changed 85
  of 120 (scroll) and 35 of 121 (drag). The cause of the real-IDE freeze under Vello was not found; the spike does not
  show it, so it is specific to the current pipeline (a 3-view-tall `Image` that changes by re-centering).
  This disproves "Vello changes nothing" for the current pipeline, though the pipeline is the part to be deleted.
- The real IDE also runs under `renderer-femtovg-wgpu` (`SLINT_BACKEND=winit-femtovg-wgpu`, backend string
  `FemtoVG renderer with WGPU backend`): scroll 118 of 119 changed, drag 120 of 121.

### Question 2: what Slint 1.18.1 text can do

Layout engine:

- Slint lays out text with Parley in every renderer: `shared-parley` is enabled by the FemtoVG renderer
  (`i-slint-renderer-femtovg-1.18.1/Cargo.toml:79`),
  the software renderer (`i-slint-renderer-software-1.18.1/Cargo.toml:63`),
  and the anyrender renderer behind Vello. The IDE lock already has Parley 0.11.1 and fontique 0.11.1,
  the versions Slint uses, so the IDE and Slint can shape with the same crates. The feature `unstable-fontique-011`
  exposes Slint's font collection (`slint-1.18.1/lib.rs:733-775`) so both can share one set of font identities.
- Slint sets no font features, no tab stops, and no line-break override on its Parley builder
  (`i-slint-core-1.18.1/textlayout/sharedparley/shaping.rs:64-150`). Wrapping maps `word-wrap` to
  `WordBreak::Normal` plus `OverflowWrap::Anywhere`, `char-wrap` to `BreakAll`, and `no-wrap` to `KeepAll`
  (`shaping.rs:126-138`).

Observed on screen (`probe` binary, identical under FemtoVG and Vello):

- A tab in a `Text` draws as a missing-glyph box; there are no tab stops. The `.slint` string literal has no `\t`
  escape (compile error "Unknown escape sequence"), so tabs only arrive from Rust.
- Ligatures are on (JetBrains Mono `calt`), matching the IDE default. No property turns them off or sets other features.
- `letter-spacing`, `font-weight` (variable axis works: 300, 400, 700, 800 differ), `font-family`, `font-italic`
  (a request for italic without the italic face loaded stays upright), `line-height-factor`, and `stroke` exist.
- Wrapping breaks by UAX 14 with a per-character fallback: `foo(bar, baz); qux.quux(a,b)+c` breaks after
  `baz);` and after `qux.quux(a,b)+c`, not after `,` or `(`. A continuation row starts at column 0 with no hanging
  indent. Both differ from the IntelliJ rules and indent of `doc/planning/slint-ide-soft-wrap.md`.
- Fallback glyphs work (CJK takes two columns, 18 px at 15 px; emoji draw as monochrome outlines).
- `StyledText` takes colors only through CommonMark plus `<font color="...">` and `<u>`. Leading blanks are dropped,
  four or more leading blanks are a compile error ("Markdown code blocks are not supported"), inner double blanks are
  kept, punctuation must be backslash-escaped, and the element has no wrap, letter-spacing, weight, or line-height
  property (its line pitch is the font's natural 20 px at 15 px, not the IDE's 24 px row). Upstream closed a request
  for declarative colored children as won't-do (slint-ui/slint #13651); underline on `Text` is an open request (#3192).
- A read-only `TextInput` draws a selection from `set-selection-offsets` with the selection colors, one color for the
  whole text, and no caret in the probe. Its hit-testing and caret rectangles exist only on the internal `Renderer`
  trait (`i-slint-core-1.18.1/renderer.rs`, `text_input_byte_offset_for_position` and its siblings)
  and need an `ItemRc`, so they are not reachable through public API.

IDE text features against Slint elements (the geometry work always stays in the application's Parley, because `Text`
exposes no substring geometry):

- Syntax-colored code in JetBrains Mono: covered by one `Text` per colored run at x positions from the application's
  layout, or one `StyledText` per row if its limits are accepted.
- Pixel tab stops (multiples of two space advances): not covered by `Text`; the application splits runs at tabs and
  places them.
- Soft wrap with IntelliJ break rules and continuation indent: not covered by `wrap`; the application breaks rows and
  each visual row is one no-wrap element.
- Programming-ligature caret and selection halves, point-to-character mapping, caret, Up and Down aim: not covered;
  stay in Parley (as today).
- Selection and find rectangles, caret, line numbers, severity letters: already Slint `Rectangle` and `Text` today.
- Selected text in one ink: needs the runs of selected rows re-emitted in the selection ink or a clipped second copy.
- Diagnostic underlines (wavy, dashed, dotted, sparse dots): not covered by `Text`; `Path` elements or rectangles.
- Hint and message rows (Inter 13 px at 70 percent ink, at pixel x of a character): covered by `Text` at those x.
- Wrap sign in the gutter: `Text` with the symbol chain, `Path` for the drawn arrow.
- Per-feature OpenType settings (`SourceTypography::features`): not covered; only the default `calt` matches.

### Question 3: windowed rows for 50,000 lines

Element counts for a 2000 px window at 24 px rows: 83.3 rows are visible, 90 slots are laid out. With one number
`Text` and 1.92 run `Text` elements per non-blank line that is about 260 elements; the twelve-slot variant
creates 1,170 (most invisible); a minimap of one sixth needs 500 rows (4 px each), 500 elements with one color per
row or about 960 with runs.

Spike results, release, `take_snapshot` per step (median ms, Vello first, FemtoVG second):

- One `Text` per row, per colored run, twelve fixed run slots, one `StyledText` per row (all with ring updates):
  5.9 and 13.8; 5.7 and 14.1; 6.5 and 15.2; 6.0 and 13.9. The static scene is 5.8 and 14.7. The element kind does not
  matter at this scale; the update strategy does.
- Update strategy, one `Text` per run, 50,000 lines: every slot rebound to the next row each time the first row
  changes (a `for slot in N` over `all-rows[first + slot]`) cost 10.5 to 11.0 (Vello) and 18.9 (FemtoVG); setting
  every row of a model, and replacing the model, cost the same; a ring (`VecModel` `remove(0)` and `push`, so each
  row element keeps its line) cost 5.7 and 14.1, which equals the static scene.
- The first row as a Slint binding on `content-y` cost the same as native-driven for 90 rows, but see the minimap
  below: a binding on `content-y` makes every dependent item re-evaluate each step even when its value is unchanged
  (a 489-row minimap whose first row never changes cost +10.5 ms per step).
- Culling probe: rows laid out outside the Flickable view are not free. 500 extra rows cost +16 ms per step (both
  renderers), 5,000 extra rows +185 ms, because Slint re-binds and re-lays them out. The window must stay tight.
- Per-step scroll size (Vello, ring): 8 px 6.0, 40 px 5.7, 200 px 6.4 ms.
- Debug builds are about twelve times slower: one `Text` per run, 489 lines, 124 ms (Vello) and 140 ms (FemtoVG) per
  step. The IDE sets only `debug = 0` for `[profile.dev]`, so Slint, Parley, and Vello run unoptimized in debug.
- Scale 2: 22.3 (Vello) against 54.3 (FemtoVG) for one scroll step of the plain view.
- Cadence at 120 steps in 2 s, release, spike with ring rows and a minimap: scroll, 489 and 50,000 lines: 119 of 119
  changed for both renderers. Minimap drag, 489 lines: 120 of 121 both. Minimap drag, 50,000 lines (the minimap slice
  rebinds 700 elements each step): 60 of 121 for Vello (longest run of identical frames 2) and 61 of 121 for FemtoVG
  (longest run 1), that is every other frame. Dense file, 2,000 lines, drag: Vello 115 of 121, FemtoVG 116 of 121.
  Debug builds, 489 lines: scroll 31 (Vello) and 39 (FemtoVG) of 119; drag 40 and 41 of 121.

### Question 4: the minimap on the GPU

Three Slint-element variants were measured (single-color rows, 500 rows of a 4 px pitch): the same elements under
`transform-scale` one sixth, elements with a sixth of the font size, and the scaled group with
`cache-rendering-hint`.

- Vello cannot use `transform-scale` for text in 1.18.1: rows are culled by an unscaled clip. The cause is in
  `i-slint-renderer-anyrender-1.18.1/itemrenderer.rs:698-701`: `scale` changes the transform but not `clip_rect`, and
  text drawing skips lines outside that clip (`i-slint-core-1.18.1/textlayout/sharedparley/draw.rs:78-92`);
  FemtoVG scales its scissor (`i-slint-renderer-femtovg-1.18.1/itemrenderer.rs:810-817`). Screenshots
  show Vello drawing only the top part and the left 60 percent of a scaled minimap and FemtoVG drawing all of it.
  A one-line change (divide `clip_rect` by the factors in `scale`) in a patched copy of the crate
  (`~/temp/agent/ide-vello-patched`) draws the whole minimap. `master` of 2026-10-19 still has the bug.
  The font-size variant works in both renderers.
- `cache-rendering-hint` does nothing under Vello: `visit_layer` is not implemented, so the core default
  continues to the children (`i-slint-core-1.18.1/item_rendering.rs:623-629`; only the FemtoVG and Skia item
  renderers implement it). Under FemtoVG it caches a texture of the group; with native-driven first rows the layer
  costs 14.3 to 14.4 ms against 14.2 to 14.5 for no minimap, while the same rows uncached cost +2.4 to +3.1 ms.
- Vello re-records the whole scene every frame (`vello.rs` `render_scene_to_target` resets and redraws), so thousands of
  tiny `Text` elements cost per frame, not per change. Measured: 489 single-run rows +3.1 ms, 500 rows with runs
  (about 960 elements) +3.7 ms over the plain view. That is cheap next to the off-screen cost above.
- Crispness: at 2.5 px glyphs every Slint-element variant is a legible but soft smudge in both renderers; I did not
  compare against the existing minimap bitmap, so "as crisp as now" is not established. Zoomed crops are in
  `~/temp/agent/ide-vello-runs/zoom`.
- The native-driven first row matters most: with `first` computed in Rust and set only when it changes, the 489-line
  minimap costs +3.1 ms (Vello) and +2.4 ms (FemtoVG); bound to the scroll ratio in Slint it cost +10.5 ms.

### Question 4b: a GPU design outside Slint's text elements

`ide-vello-spike2` draws Parley layouts with Vello 0.11 (which uses wgpu 30, the same major as Slint's wgpu 30 path)
into two `Rgba8Unorm` textures on Slint's own device and imports them with `slint::Image::try_from(texture)`.
It needs Slint's `renderer-femtovg-wgpu` and `unstable-wgpu-30` features and the device limits
`wgpu::Limits::default()` (Slint's default downlevel limits allow no storage buffers and Vello panics at start).
The 1147x6000 tile (three views, like the CPU tile it replaces) is re-rendered only when the view leaves its middle
region; the 120 px minimap texture holds 500 rows at one sixth with colored runs and is re-rendered when its first
row changes. A Slint `Flickable` moves the tile image, a Slint `TouchArea` and `Rectangle` make the box, and the Slint
scene composes everything, so it composes with widgets like any `Image`.

- Cadence, release: scroll 119 of 119 (489 lines) and 119 of 119 (50,000); drag 120 of 121 and 121 of 121.
  Dense 2,000-line file drag: 121 of 121. Debug build, 489 lines: scroll 70 of 119, drag 68 of 121 (the builtin-`Text`
  designs changed 31 to 41).
- Cost per step (snapshot including a GPU wait on re-renders): 13.4 ms for 50,000 lines (minimap re-rendered every
  step, 7.3 ms of CPU encode and submit; three tile re-centers of 6.4 ms); 16.7 ms for the dense 2,000-line file
  (minimap 10.9 ms). The `Text` designs with a minimap on the same dense file cost 23.3 ms (Vello) and 30.0 ms
  (FemtoVG) per step. There is no row cache in the spike, so the minimap shapes 500 lines per step; a row cache
  removes most of that. A GPU timestamp was not taken, so these are not GPU times.
- Memory: process RSS 89 to 103 MiB (builtin `Text` designs 96 to 140 MiB, the current IDE 173 to 184 MiB with
  Helix and the tree). Textures: 27.7 MB tile and 1 MB minimap; GPU memory (`mem_info_vram_used` of the RX 7600)
  grew by 300 to 360 MiB over the desktop's 6.5 GiB baseline (Vello `Text` design: about 300 MiB; FemtoVG `Text`:
  85 to 120 MiB; current IDE under FemtoVG: 160 MiB; under Vello 420 MiB after settling and 625 MiB after the drag).
  Those VRAM figures include the nested compositor's own buffers and a busy desktop; trust them to about 50 MiB.
- What it keeps: Parley layout, shaped rows, wrap and tab geometry, the row map, hit testing, caret, selection and find
  rectangles, and the tile re-center policy. What it deletes: every Swash raster path.
  One shaper draws and measures, so glyph and geometry cannot disagree (the builtin-`Text` designs shape every run
  twice, once in the application and once in Slint, and must trust the two agree).

### Where CPU pixels exist today and must be deleted

All of these produce pixels on the CPU in the current code and are replaced by any design:

- Glyph raster into the tile: `src/text_raster.rs`, `src/glyph_cache.rs`, `src/selection_paint.rs`,
  `src/selection_ink.rs` (per-pixel ink blending), `src/annotation_paint.rs` (underline pixels), the `swash`
  dependency, and the raster half of `src/shaped_text/`.
- The tile as an image: `src/native/tile.rs`, `src/native/tile_build.rs` and `tile_build/`, `src/native/tile_pass.rs`
  (the shaping half stays), `src/native/render/paint.rs` and `render/present.rs` (`set_source_image` with
  `Image::from_rgba8_premultiplied`), `src/source_frame.rs` stamp logic that only guards pixels, the raster limit
  tests (`tests/raster_limit.rs`, `raster_reuse.rs`, `raster_invalidation.rs`, `src/native/refused_tile_tests.rs`).
- The wrap sign picture: `src/wrap_sign.rs` and `set_wrap_sign` in `render/present.rs`.
- The minimap picture and band cache: `src/minimap/build.rs`, `render.rs`, `downscale.rs` (box average), `fills.rs`,
  `cache.rs`, `src/native/minimap_build.rs` and `minimap_build/`, `src/native/minimap/picture.rs` (`set_minimap_image`),
  `minimap/request.rs`, `minimap/pace.rs` (the planning and pointer geometry stay), and `tests/minimap_picture.rs`,
  `tests/minimap_downscale.rs`.
- The software renderer: the `renderer-software` feature in `Cargo.toml`, and the `winit-software` spelling that
  `src/launcher.rs` forwards. The native tests use Slint's testing backend with its software renderer for
  `take_snapshot`; that is test-only CPU, to be replaced by a GPU snapshot harness or accepted as test scaffolding.
- Slint's own chrome text (tree, find bar, search, hover) is rasterized by FemtoVG's glyph atlas: FemtoVG calls Swash
  for each glyph on the CPU, cuts a coverage mask, and uploads it into an atlas texture
  (`femtovg-0.27.0/src/text.rs:574-640` and `739-800`;
  Slint enables it at `i-slint-renderer-femtovg-1.18.1/Cargo.toml:59`).
  This is library code, cached per glyph, size, and subpixel phase, but it is CPU glyph raster. Only Slint's Vello
  renderer draws glyphs as GPU vector paths. Whether that counts is the first owner question.

## FemtoVG and Vello side by side

The owner asked for both renderers on every question. Each line names Slint's own `Text` elements unless it says
otherwise; the full numbers are under the questions.

- Builds and selection: FemtoVG is the default and needs nothing; Vello needs `renderer-vello` in `Cargo.toml`
  (+30 lock entries) and `SLINT_BACKEND=winit-vello` or `renderer_name("vello")`.
- Text support (Question 2): identical, both use the same Parley layout and offer the same properties; no tab stops,
  features, or wrap rules in either.
- Window rows (Question 3): equal behavior and equal rules (ring rows, tight window, no `content-y` bindings).
  Cost per scroll step: Vello 5.7 ms, FemtoVG 14.1 ms; at scale 2, 22.3 against 54.3 ms.
  Cadence in release: both 119 of 119 for scroll; both about every other frame (60 and 61 of 121) for the 50,000-line
  drag with a rebinding minimap slice; both 31 to 41 of about 120 in debug.
- Minimap (Question 4): FemtoVG works with `transform-scale` and with a cached layer (+0 ms once the slice is
  stable); Vello draws `transform-scale` wrongly and ignores `cache-rendering-hint`, but font-size scaling works
  (+3.1 ms Vello, +2.4 ms FemtoVG).
- Real IDE as it is today: FemtoVG 118 of 119 (scroll) and 119 of 121 (drag); Vello 66 of 119 and 49 of 121.
  In debug: FemtoVG 113 and 100, Vello 85 and 35.
- GPU purity: Vello draws glyphs as GPU paths; FemtoVG keeps a CPU Swash glyph atlas.
  `renderer-femtovg-wgpu` has the same atlas, so the wgpu variant does not change this.
- Memory (VRAM above the desktop baseline): Vello `Text` about 300 MiB, FemtoVG `Text` 85 to 120 MiB.

## Designs

### Design A: builtin `Text` elements on FemtoVG

Application Parley geometry places one `Text` per colored run (or per wrapped row), rows are a ring model of about
90 slots, the minimap is a cached `Layer` of the same rows at a sixth of the font size, selection ink by re-emitting
runs. Slint stays on `renderer-femtovg`.

- Pros:
  no new crate and no lock change; the app already runs and measured 118 of 119 and 119 of 121 frames;
  `cache-rendering-hint` works, so the minimap costs about nothing while the slice is stable;
  `transform-scale` and the rendering notifier (`frame_trace.rs`) keep working;
  FemtoVG is the default and best tested renderer; the test harness stays.
- Cons:
  FemtoVG rasterizes every glyph with Swash on the CPU into an atlas, so the code view is still CPU glyph raster,
  which the widened rule forbids literally;
  text is shaped twice (application and Slint) and per-run elements are the only way to carry colors;
  a dirty-cascade and ring discipline is needed or the minimap and rows relayout every step;
  50,000-line drag showed every other frame in the spike (61 of 121); a cached layer cannot help there because the
  slice's first row changes on every drag step, and a sliding (ring) minimap model was not measured;
  debug builds drop to about a third of the frames (39 of 119).

### Design B: builtin `Text` elements on Slint's Vello renderer

Same element design, with `renderer-vello`, `SLINT_BACKEND=winit-vello`, and the font-size minimap (the transform
variant clips wrongly).

- Pros:
  glyphs are GPU vector paths, so the code view, the minimap, and all chrome text have no CPU pixel at all, the only
  option where that is literally true for the whole window;
  cheapest per-step cost of the two `Text` designs in the spike (5.7 against 14.1 ms; 22.3 against 54.3 at scale 2);
  Swash and every raster path go away without custom GPU code.
- Cons:
  experimental in Slint 1.18.1, no renderer-side `cache-rendering-hint`, no rendering notifier, no wgpu texture import
  (`itemrenderer.rs:1236-1245` returns nothing for them), a transform-clip bug that needs an upstream or patched fix;
  +30 lock entries including a second wgpu family;
  the real IDE froze for about 1 s under Vello with the current tile (cause unknown) and needs a fresh check once the
  tile is gone;
  Vello re-records the whole scene each frame, so element counts cost every frame;
  same double shaping and per-run element cost as A;
  50,000-line drag showed every other frame (60 of 121);
  an extra 350 to 600 MiB of GPU memory.

### Design C: Parley drawn by Vello into wgpu textures shown as Slint images

Described under Question 4b: tile and minimap textures on Slint's wgpu 30 device, Slint on `renderer-femtovg-wgpu`.
Selection ink, diagnostic underlines (wavy and dashed as vector paths), hint rows, and the wrap sign can all be vector
scene content, so no `Image` picture and no per-pixel code would remain; the spike draws only glyph runs, and these
parts are argued, not built.

- Pros:
  every code-view and minimap pixel is produced by the GPU, with color runs on the minimap;
  one Parley layout draws and measures: ligature halves, tabs, wrap rules, indent, and hit tests stay exact and no
  second shaper exists;
  keeps the overscan tile policy, so scrolling inside the tile does no work;
  best measured cadence: 121 of 121 under the 50,000-line drag and 68 to 70 of about 120 in debug against 31 to 41
  (the `Text` designs' drag used a minimap slice rebound every step; a cached or larger slice was not measured);
  smaller footprint than the `Text` designs (RSS 89 to 103 MiB);
  adds only the vello crates (wgpu 30 is already locked) and removes Swash.
- Cons:
  depends on Slint's `unstable-wgpu-30` and `renderer-femtovg-wgpu`, both semver-exempt, and changes the whole
  application's renderer from GL to wgpu (measured to run, not soaked);
  chrome text still goes through FemtoVG's CPU glyph atlas;
  custom scene code (about 300 lines in the spike for rows and the minimap) replaces `Text`;
  Vello's compute shaders need storage buffers, so the whole application's wgpu device must be requested with
  `wgpu::Limits::default()` instead of Slint's downlevel defaults, which excludes downlevel adapters for the entire
  IDE (the integrated Raphael GPU of this host was not exercised);
  the headless pixel tests cannot import a wgpu texture, so painted-ink tests need a GPU snapshot harness;
  Slint's rendering notifier hands over the device only after setup, so the textures are created in the notifier;
  the tile is 6,048 px tall at scale 1, so a 2,000 px window at scale 2 needs a 12,096 px texture, above the 8,192 px
  default `max_texture_dimension_2d`, and the tile would have to be split or shortened (not measured).

### Designs considered and not measured

- Slint's Skia renderer with wgpu textures: not enabled in the IDE and not built here; no claim is made about it.
- Draw the code view on its own wgpu surface beside Slint: loses Slint composition (popups over text, focus,
  accessibility); not measured, argued only.
- One `Text` per syntax class holding the whole page with blanks for other classes: monochrome-column hack, breaks on
  fallback-width glyphs and multiplies glyph count by the class count; rejected without measuring.

## Ranking

The first owner question under the last heading selects between two rankings, because it decides whether FemtoVG's
CPU glyph atlas counts as the banned CPU painting. Slint's chrome text (tree, find bar, search, hover) goes through
that atlas whenever Slint runs on FemtoVG, which is every design except B.

Strict reading (the atlas counts as CPU painting): B > C > A.

- B ahead of C: B is the only design in which no pixel of the window, chrome included, comes from CPU glyph raster;
  C draws the code view and minimap on the GPU but its chrome still goes through FemtoVG's atlas, and Slint's Vello
  renderer cannot import a wgpu texture, so C cannot be moved onto Vello for the chrome.
- C ahead of A: C produces the code view and the minimap on the GPU with no raster path left, while A leaves every
  code glyph on the CPU atlas as well as the chrome.

Lenient reading (the atlas is accepted): C > A > B.

- C ahead of A: C holds 121 of 121 frames under the 50,000-line drag where A holds 61 of 121, shapes each line once,
  and keeps the application's exact tab, wrap, and ligature geometry; the cadence comparison is against an
  unoptimized `Text` minimap (a rebinding slice), so the gap is an upper bound, not a measured ceiling for A.
- A ahead of B: A adds no crates and has a working layer cache, no clip bug, and no unexplained freeze, and its per-step
  cost disadvantage (14.1 against 5.7 ms) did not show up in cadence (both 119 of 119 for scroll).

My GPU renderer recommendation follows the same split. If the atlas counts, Vello through design B, with the
font-size minimap and the one-line clip fix proposed upstream (design B has the least custom code, and the freeze of
the real IDE must be re-checked once the tile is gone). If the atlas is accepted, Vello 0.11 drawing into textures
under Slint's FemtoVG-on-wgpu renderer (design C), because it measured best and keeps one shaper. FemtoVG as the
renderer of the `Text` elements (design A) is the choice only when simplicity outweighs both.

## Migration stages (design C, with design B differences at the end)

Each stage leaves the app usable and has its own check. Stages 1 and 2 are reversible by a flag until stage 5. Stages 3 and 4 were merged into one, and so were stages 5 and 6.

1. Measurement and profile groundwork. Add `[profile.dev.package."*"] opt-level` for dependencies (debug builds are
   twelve times slower in Slint and Vello). Keep the cadence driver with a positive control (the debug build must show
   frozen frames). Check: the pattern of changed frames under `scroll` and `drag` for today's pipeline is recorded.
2. Renderer switch. Enable `renderer-femtovg-wgpu` and `unstable-wgpu-30`, select it in `src/launcher.rs`
   (`WGPUConfiguration::Automatic` with `Limits::default()`). Check: the existing native tests, the container
   suite, and a nested-compositor run (done once here: 118 of 119 and 120 of 121 frames).
   Done on 2026-10-20 (section "Stage 2" of `doc/planning/ide-gpu-snapshot-harness.md`), with one change: the launcher
   opens the device itself (`WGPUConfiguration::Manual`) because `Automatic` can pick a CPU adapter.
3. and 4. (merged into one stage) The code view on the GPU. A module draws a Parley layout into a Vello scene and a
   tile texture on the device from the rendering notifier; the tile re-center policy and worker stay, but the worker
   produces scene data, not pixels. Selection ink, diagnostic underlines, hint and message rows, and the wrap sign are
   scene content too. Find boxes and selection fills stay Slint rectangles behind the tile, which FemtoVG draws on the
   GPU already. Delete the tile's CPU painter (`TextRaster::paint`, `wrap_sign.rs` raster); the rest of
   `text_raster.rs`, `glyph_cache.rs`, `selection_paint.rs`, the `annotation_paint.rs` pixel code and `swash` stay
   until stage 5 because the minimap picture still uses them (stage 6 deletes them). Check: the structural test that
   scrolling inside the tile does no scene build and renders nothing, the selection, annotation, find and wrap sign
   tests rewritten against paint items and GPU readback, a guard per piece that fails a test when the piece is
   removed, and the cadence run.
   Done on 2026-10-20 (section "Stages 3 and 4" of `doc/planning/ide-gpu-snapshot-harness.md`).
5. and 6. (merged into one stage) The minimap as a GPU slice texture, and the end of the CPU pipeline. The worker
   shapes the lines of the slice with a row cache and encodes them as paint items in chunks of at most 512 rows;
   the one Vello renderer of the code view renders each chunk and copies it into the slice texture, and the box,
   pointer, and planning code stay. Deleted: `downscale.rs`, `build.rs`, `render.rs`, `key.rs`, `cache.rs` of the
   minimap, `text_raster.rs`, `glyph_cache.rs`, `selection_paint.rs`, the pixel code of `annotation_paint.rs`, the
   `swash` dependency, and the CPU minimap tests. Check: the drag cadence for 489 and 50,000 lines, a crispness
   comparison against the removed bitmap, `tall_window_minimap_tests`, a guard per thing the minimap draws, and a
   repository search that no `from_rgba8` or `SharedPixelBuffer` remains in application code. The software renderer
   feature was already removed from the application in stage 0b (test scaffolding reads GPU snapshots).
   Done on 2026-10-20 and 2026-10-21 (section "Stages 5 and 6" of `doc/planning/ide-gpu-snapshot-harness.md`).

For design B the stages differ at 2 (enable `renderer-vello` in `Cargo.toml`, patch or upstream the one-line clip fix
before the minimap), 3 and 5 (ring rows of `Text` elements, font-size minimap), and need a re-run of the Vello freeze
check once the tile is gone.

## What I could not measure

- Real GPU frame time: there is no GPU timestamp in these runs; the snapshot cost includes a readback, and cadence is
  what the compositor recorder sees at 60 Hz.
- The real IDE on a 50,000-line file under any renderer, and any design inside the real application beyond the spikes.
- The cause of the real-IDE freeze under Vello, and whether it recurs with design B in the real application.
- Visual parity with today's glyph raster (the spike's Vello draws unhinted outlines; weights may look different).
- Minimap crispness against the existing bitmap, and any metric of crispness.
- Syntax run density of real Helix highlighting (the spike's tokenizer gives 1.5 to 1.9 runs per line, with the
  denser Rust file at 1.50 runs and 37 characters per line against 1.92 and 20 for the TypeScript file).
- Memory beyond RSS and a noisy VRAM counter shared with the desktop.
- Skia, accessibility, input-method, resize, and multi-monitor behavior under the wgpu renderers.
- The native test suite against any design.

## Questions only the owner can answer

- Does FemtoVG's glyph atlas, which rasterizes each glyph with Swash on the CPU and uploads the mask, count as the
  CPU painting that is banned? If yes, FemtoVG is out for the chrome as well and only Slint's Vello renderer draws
  every pixel on the GPU; if no, design A is allowed and is the simplest.
- Is a dependency on Slint's `unstable-wgpu-30` and `renderer-femtovg-wgpu` acceptable for the whole application,
  given that both are semver-exempt and the pinned Slint is 1.18.1?
- If design B or the chrome-on-Vello reading wins: may the repository carry a one-line Slint patch (a vendored crate and
  `[patch.crates-io]`) or should the fix go upstream first and the minimap use font-size scaling meanwhile?
- May `Cargo.lock` take the vello family (16 new entries for design C, 30 for design B including a wgpu 29 family)?
- Is a soft minimap (2.5 px glyphs) the accepted crispness, or must the minimap match the current bitmap's sharpness
  before the old code is deleted?
- May native tests that read pixels use Slint's software renderer as test scaffolding, or must they run on a GPU
  snapshot harness?
- Should a `[profile.dev.package."*"]` optimization level be added for dependencies (it changes debug build time)?
