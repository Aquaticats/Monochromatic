# Vector design engine: technology decisions

Records technology decisions for the 2D vector design engine
(an OSS Figma alternative targeting 1M-node documents on native desktop).
Future sessions consult this before re-proposing rejected paths.

This document is appended to,
 not rewritten.
When a downstream choice forces re-evaluation of an earlier one,
mark the earlier decision superseded;
 do not delete it.

## Context

OSS license required for the project itself.
Performance target:
 1M-node documents without stuttering.
"Node" means smallest unit (a `<span>a</span>`-sized atom).
Native desktop only,
 no browser,
 no webview.

The 1M target is a hard requirement,
 confirmed by the user.
Real design files have nested components rather than 1M independent visible nodes,
but the rated case is 1M visible at the user's stated p99 frame-time budget.

### Frame-time budget

Stuttering means 99th-percentile frame time,
 not 60 fps average.
Concrete budget:
 **p99 frame time ≤ 16.67 ms (60 fps)** on the hardware floor.

### Hardware floor

The development machine,
 taken as the slowest acceptable target
(designers working in 1M-node files are assumed to have similar or better hardware):

- AMD Ryzen 7 8700F (8 cores / 16 threads,
   5.05 GHz boost)
- 62 GB RAM
- AMD Radeon RX 7600 (8 GB VRAM,
   RDNA 3 / Navi 33)
- Linux 6.19 / Bazzite
- Mesa 26.0.5 with Vulkan and OpenGL 4.6

### Memory budget (back-of-envelope)

Numbers from rough calculation,
 to be verified in phase 0:

- Bevy ECS:
   1M entities × ~250 bytes/entity (Transform 64 B + GlobalTransform 64 B
  - Aabb 24 B + NodeKind 4 B + StyleHandle 8 B + per-kind data ~32 B
  - change-tick metadata ~40 B + archetype overhead amortized) ≈ 250 MB RAM.
- Document geometry (paths,
   text glyphs,
   image references):
   variable.
  Allow ~1 GB for a typical large document.
- BVH spatial index:
   1M × ~32 bytes ≈ 32 MB RAM.
- Vello GPU buffers (path encoding,
   scene state):
   variable;
   budget ~500 MB VRAM.
- Tile pyramid:
   lazy,
   LRU-capped.
   Per-tile cost is 256×256 RGBA8 = 256 KB.
  Cap working set at ~3 GB VRAM (covers ~12k tiles in cache).
- Glyph atlas,
   framebuffers,
   staging buffers:
   ~500 MB VRAM.

Totals:
 ~1.8 GB RAM,
 ~4 GB VRAM working set under 8 GB cap.
Headroom for the OS,
 the browser the user has open,
 and the document's actual asset weight.

### Engine scope

Render plus pan/zoom plus select-to-inspect.
No editing,
 no persistence,
 no collaboration in this phase.

## D1. Distribution model: true native binary

Picked:
 standalone Rust binary using winit for the window.

Rejected:

### Web (browser, WebGPU)

User explicitly chose native over web.
Browsers have per-tab memory ceilings
(V8 heap caps,
 WASM 4 GB linear memory cap,
 GPU process memory limits)
that force compromises (eviction,
 paging,
 document chunking)
at 1M-node working sets.
WebGPU support across Chromium,
 Firefox,
 and Safari is uneven on Linux.

### Patch Chromium to remove the 2 GB tab limit and ship the patched browser

Violates the native pick.
Independent of the pick,
 shipping a custom Chromium fork carries ongoing costs:
rebasing patches against upstream every release cycle,
carrying the security review burden of a browser engine,
asking users to install a custom browser (a hard sell for a design tool).

### Tauri (webview plus WebGPU)

Was an early default based on stack-match (the monorepo is TS/Bun heavy).
With native picked,
 the webview wrapper adds nothing it does not also cost.
WebKitGTK (Tauri's Linux backend) has historically lagged Chromium on WebGPU;
pursuing the webview path would require verifying current support
before committing to Linux.

### Electron

Same problems as Tauri plus a heavier runtime
(Chromium plus Node,
 hundreds of MB even before app code).

### Cloud rendering with thin client (Stadia-style)

Server runs the engine,
 client streams pixels.
Breaks the native-offline expectation (a design tool that needs network is a different product).
Adds infrastructure burden inappropriate for a self-hostable OSS project.
Latency hurts interactive editing in ways that do not show up in non-interactive use.

### Hosted desktop (Citrix-style)

Run the application remotely,
 view via remote desktop protocol.
Same drawbacks as cloud rendering plus higher input-to-pixel latency.
Does not deliver "native desktop.
"

### Mobile (iOS / Android)

Outside the desktop pick.
Touch-first UX differs enough from desktop design tools
that the engine could ship later but not as v1.

### VR / AR (Quest, Vision Pro)

Research-grade for design tools.
Outside scope.

### Headless engine library (no UI)

Distribute the engine as a library;
 consumer apps build their own UI.
Reasonable as a future packaging option but does not deliver
an OSS Figma alternative on its own.
Re-evaluate if the engine matures and external consumers ask for it.

## D2. Implementation language: Rust

Picked:
 Rust for both the engine and the application binary.

C and C++ are excluded categorically as project source languages.
The user's stated reason:
 too hard to read and write.
Even with the dum-dum-non-ts skill convention
(labeled comment blocks plus TypeScript pseudocode above each concept),
the cognitive load is unworkable.
This rules out using C or C++ as primary language for the project itself
and rules out forking codebases where modifications would land in C or C++ source.
Using Rust libraries that wrap C or C++ as a transitive dependency is fine;
the project source remains Rust.

Acknowledged residual cost:
 debugging GPU driver issues (wgpu's Vulkan backend,
Mesa RADV) or font-rendering issues (skrifa's font crates) may require reading
C or C++ source even without modifying it.
Mitigation:
 file upstream bugs rather than patch locally;
 use wgpu's debug layers
and validation to isolate issues before descending into the underlying driver.
This cost does not overturn D2;
 flagging it so future sessions notice.

Rejected:

### C and C++

Excluded categorically by user preference (above).
Performance is not the issue;
 readability and writeability is.
Rust gives equivalent runtime performance,
a graphics ecosystem
(wgpu,
 Bevy,
 lyon,
 swash,
 glam are all first-class Rust libraries),
modern tooling (cargo,
 clippy,
 rust-analyzer),
and memory safety at compile time.

### Go

Stop-the-world GC at 1M nodes risks frame stutter that the p99 target rules out.
Graphics ecosystem (Ebiten,
 raylib bindings) is far behind Rust's.

### Zig

Language and ecosystem still pre-1.0;
graphics library coverage is thin.
Worth revisiting in a few years;
 not now.

- **C#** (via MonoGame,
   Stride,
   etc.).
  GC concerns same as Go.
  Outside the rest of the planned stack
  (no other C# in the monorepo or in adjacent tools).

### Java / Kotlin (JVM, JavaFX, LibGDX)

Mature graphics ecosystem.
Same GC concerns as Go and C# at p99.
AOT-compiled options (GraalVM native-image) reduce GC but lose JIT optimizations.
Outside the rest of the planned stack.

### Swift

First-class on Apple platforms;
 Linux support exists but is second-class.
GPU story leans Metal,
 which limits cross-platform options.
ARC has reference-cycle pitfalls similar to manual memory management.
Cross-platform parity with Rust is weaker.

- **OCaml,
   F#,
   Haskell** (functional).
  Strong type systems,
   niche but interesting.
  Smaller graphics ecosystems
  (no production-grade wgpu equivalent).
  GC for Haskell brings the same p99 risk;
  OCaml's GC is shorter-pause but the library gap dominates the decision.

## D3. GPU abstraction: wgpu

Picked:
 wgpu,
 with backend selection delegated to wgpu's runtime
(Vulkan on Linux via Mesa RADV;
 Metal on macOS;
 DX12 on Windows).

This is the GPU-abstraction layer.
The renderer library that sits on top of wgpu is decided in D10.

### Hardware-floor risk: wgpu plus RADV plus Navi 33

wgpu's Vulkan backend has had historical edge cases on Mesa RADV,
and the RX 7600 (Navi 33,
 RDNA 3) is a mid-range GPU where compute-shader
support could surface driver issues that high-end Navi cards do not.
Phase 0a (see Phase 0 plan below) includes a wgpu smoke test
(triangle,
 indirect draw,
 simple compute shader) on the target GPU
before any architectural commitment.
Cheap;
 catches driver-level blockers early.

Rejected:

#### Direct Vulkan, Metal, and DX12 implementations

Three platform-specific renderers cost years of engineering
that wgpu eliminates.
Vulkan API in particular is famously verbose;
hand-rolling it for a 2D engine is wasted effort.

#### OpenGL only

wgpu already has a GL backend for compatibility,
so picking GL outright forfeits modern features (compute shaders,
 indirect draw,
 bindless)
that the LOD architecture (D6) relies on,
in exchange for nothing.

#### Skia / CanvasKit

Mature 2D library used in Chrome,
 Flutter,
 Android.
C++ source rules it out per D2's categorical exclusion;
any Skia bug or extension would require writing C++.
Rust bindings exist (skia-safe) but do not let us patch Skia itself.
Independent of D2:
 Skia owns its own rendering loop and integrates poorly
with custom render-graph designs at our scale;
pulling Skia into a Bevy app means fighting both stacks at the same time.

- **bgfx,
   The Forge,
   Filament,
   sokol** (C and C++ graphics libraries).
  All ruled out by D2's categorical exclusion of C and C++.
  None of them offer Rust APIs as first-class;
  using them from Rust either requires writing FFI shims for every extension point,
  or requires editing C / C++ source directly when a fix is needed.
  Independent of D2:
   The Forge and Filament are 3D-game-pipeline renderers
  with no native 2D vector path support.
  sokol is below wgpu's abstraction level
  and lacks compute-shader maturity for the LOD design.

#### SDL3 GPU API

Newer API in SDL3,
 viable but smaller Rust ecosystem than wgpu.
No upside over wgpu for a Rust project.

## D4. Game engine: Bevy

Picked:
 Bevy (Rust ECS plus wgpu game engine).
The user's framing was "develop as if developing a game,
"
which makes a real engine the right starting point rather than first-principles.

Rejected:

### Unity

Closed-source engine.
 Shipping a closed-source dependency in an OSS project
creates licensing exposure (per-seat fees,
 royalty thresholds).
The 2023 install-fee policy reversal demonstrated the contract is not stable.
Workflow assumptions (project files,
 asset import) are tuned for game development,
not design tools.

### Unreal Engine

Source-available under a custom EULA,
 not OSS in the standard sense.
5 percent royalty above $1M revenue.
Architecture is 3D-first with 2D as an afterthought;
binary size starts in the hundreds of MB before any project code.
Contributing fixes upstream as an OSS project is awkward under the EULA.

### Godot

A legitimate OSS alternative
(MIT license,
 mature 2D primitives:
 CanvasItem,
 Polygon2D,
 Path2D).
Rejected in favor of Bevy because:
(a) Rust integration is via gdext,
 less native than Bevy's first-class Rust support;
(b) Godot's 2D scene tree at 1M instances requires bypassing the standard Node2D system
in favor of MultiMeshInstance2D,
which removes the scene-tree integration that is the centerpiece of Godot's framework;
(c) ECS aligns with the per-frame access pattern of D5
better than Godot's scene-tree node model.
Re-evaluate if Bevy proves unworkable in phase 0.

- **Defold** (King's 2D-focused OSS engine,
   Apache 2.0).
  Production-proven 2D engine.
  Engine internals are C++ (D2 rules them out).
  Extension surface is Lua;
  even if we accepted Lua scripting,
  any renderer change for our 1M-node target lands in the C++ core,
  which we cannot touch per D2.

### Cocos2d / Cocos Creator

Mature 2D engine ecosystem with C++,
 JavaScript,
 Lua,
 and Python bindings,
each flavor with subtly different APIs.
C++ core ruled out by D2;
there is no first-class Rust binding,
so any engine-level change would mean writing C++.

- **O3DE / Lumberyard** (Amazon,
   Apache 2.0).
  AAA-grade OSS engine.
  C++ core ruled out by D2.
  Independent of D2:
   3D-first with a large surface area;
  adopting O3DE for 2D vector design would use a small slice of the engine
  while paying for the rest in build time and complexity.
- **Stride** (.
  NET game engine,
   OSS).
  Modern OSS engine with 3D and 2D support.
  C# (D2 ruled it out).

### Build our own game engine

Bevy itself is 5+ years of work and still pre-1.0.
Reimplementing window,
 input,
 scheduling,
 plugin system,
 asset loading,
and render graph would dwarf the project itself.
Build only what is missing on top of Bevy
(custom render plugins,
 tile pyramid,
 BVH).

### macroquad / miniquad / ggez (lighter Rust 2D frameworks)

Smaller than Bevy and easier to start with.
Rejected because the LOD architecture (D6) needs render-graph control,
asset system for the tile cache,
 and ECS for entity management;
Bevy provides those,
 and reaching for the simpler frameworks means rebuilding them.

- **Fyrox** (Rust 3D engine).
  3D-first,
   smaller community than Bevy,
  no advantage over Bevy for our 2D use case.
- **GPUI** (the framework Zed is built on).
  Rust plus GPU,
   but designed for code-editor workloads
  (text-and-rectangle heavy,
   not vector-path heavy).
  No public 1M-entity demonstration;
  the architecture is shaped around editor primitives,
   not arbitrary 2D scenes.

### Bare wgpu plus winit, no engine

Workable but loses Bevy's ECS,
 scheduling,
 plugin system,
 and render graph.
Rejected because the user framed this as game development,
and the engine value-add (ECS plus render graph) is exactly what helps at 1M-entity scale.

### Blender plugin

Blender is OSS,
 but its 2D-edit surface (Grease Pencil) is animation-oriented,
not design-tool oriented.
Plugin API is Python (outside the language pick at D2).
Distribution requires installing Blender first,
 a hard ask for design-tool users.
User-experience expectations are Figma-like,
 not Blender-like.

### RPG Maker / Stardew Valley engine (MonoGame)

RPG Maker proper is closed-source.
MonoGame is OSS but C# (D2).
Both are tile-based 2D game frameworks;
the tile assumption conflicts with vector-path content.
Stardew Valley shipped at small scene counts,
 not 1M nodes;
no calibration data at the target scale.

## D5. Data layout: Bevy ECS for all 1M nodes (deferred pending phase 0)

**Status:
 deferred until phase 0 measurement.
**
This decision rests on a bound asserted by D6 that has not yet been validated;
see "Why deferred" below.
The provisional pick stands as the working assumption,
not a final commitment.

Provisional pick:
 All 1M nodes as Bevy ECS entities,
with a small set of archetypes
(rect / path / text / image,
 each carrying Transform + GlobalTransform + Aabb +
NodeKind + StyleHandle plus its kind-specific component).
At small N (~4 archetypes),
 Bevy's archetype tables give
struct-of-arrays layout per archetype with O(1) per-system table lookup.

### Why deferred

The earlier rationale for rejecting hand-rolled SoA storage cited
"the tile-cheat architecture bounds the per-frame inner-loop entity count to 3,600
(48 px LOD threshold against a 4K viewport:
 3840 × 2160 / 48 / 48).
"

That arithmetic is wrong:
 3,600 is the number of non-overlapping 48 px tiles
that fit in a 4K viewport,
 not the number of entities.
Real design files contain overlap (groups,
 nested frames,
 dense regions,
 layers),
so the live-entity count above the 48 px threshold is open-ended above 3,600.
At a comfortable zoom in a dense document,
 live-entity count could be tens of thousands.

The SoA-vs-ECS perf trade-off depends on the actual live-entity count,
which is not yet measured.
Phase 0b (Bevy port;
 see Phase 0 plan) measures it.
If live-entity count stays in the low thousands,
 ECS is fine.
If it climbs into tens of thousands or higher,
 SoA may win on inner-loop iteration speed.

This decision also depends on D6 being valid;
if phase 0a invalidates the tile pyramid thesis,
the entire D5 framing changes.

Rejected provisionally (pending phase 0 measurement):

#### Hand-rolled SoA Vec arrays as a separate document model

Cleaner perf control on the inner loop and SIMD-friendly.
Cost:
 reimplementing query,
 change-detection,
 parallelism,
 and ID allocation.
Whether the perf win justifies the cost depends on the actual inner-loop size,
which phase 0 will measure.

- **Bevy bypassed for storage** (just wgpu,
   no entities).
  Fights Bevy's render graph (which expects entities for cull/draw passes),
  and the selection / inspector UI loses the natural "selected entity" binding
  that ECS gives for free.
  This rejection is independent of the bound discussion;
  bypassing Bevy for storage while still using Bevy for rendering
  fragments the model unprofitably.

#### Other Rust ECS frameworks: hecs, specs, legion, flecs (Rust binding)

Once D4 picked Bevy,
 the ECS is bevy_ecs.
Switching to a different ECS framework while staying in Bevy
fragments the entity model and breaks Bevy's plugin assumptions.
These frameworks are alternatives to Bevy plus its ECS together,
not alternatives to bevy_ecs while keeping Bevy.

## D6. Zoom-out cheat: tile pyramid plus LOD threshold (deferred pending phase 0)

**Status:
 deferred until phase 0a measurement.
**
The architecture is the rendering thesis;
phase 0a (bare wgpu,
 no Bevy) measures whether 1M visible at 60 fps p99
is reachable without this complexity.
If it is,
 this decision collapses entirely.
If it is not,
 the tile-pyramid + LOD-threshold approach is the working design.

Provisional pick:
At any zoom,
 render only nodes whose screen-space bounding box is at least
48 px on either axis as live geometry.
Nodes smaller than that get rendered into pre-built tile-pyramid raster textures
(256x256,
 multiple zoom levels in mipmap-like hierarchy)
that compose under the live foreground.

### 48 px threshold: justification and tunability

The 48 px figure matches the project's minimum touch-target standard
in the CSS rules (`AGENTS.md`).
The interactability anchor is real:
a node smaller than 48 px on screen is not directly clickable in our UX,
so live-rendering it spends compute on something the user cannot single-target anyway.

The threshold is a design parameter,
 not a settled rule.
Phase 0 measurement may show that
per-node rendering cost stays cheap up to a much smaller threshold
(say 16 px),
 in which case the threshold can drop and live-rendered fidelity rises;
or that it crosses zero at a higher threshold (say 96 px),
in which case the tile pyramid covers more.
Defer the final value until phase 0 produces real numbers.

### Tile pyramid lifecycle (gap-fill from review)

The earlier draft did not address tile build,
 invalidation,
 or zoom transitions.
The intended design:

#### Build at load

lazy.
The viewport's current-zoom tiles render first
(one to ~30 tiles for a 4K viewport at the current zoom level).
Adjacent zoom levels (one in,
one out) build in the background after the
initial paint to absorb pinch-zoom transitions.
Deeper levels build on demand as the user zooms.

#### Build worker

dedicated Rayon-pooled worker,
off the render thread.
Per-tile budget:
aim for 100 ms target,
hard cap 500 ms;
tiles that exceed the cap split or downgrade to a coarser-LOD substitute
rather than block the worker.

#### Cache

in-memory LRU keyed by (document hash,
zoom level,
tile x,
y).
Disk overflow on top of the in-memory cache for working sets that exceed
the VRAM cap from the memory budget.

- **Invalidate on edit** (out of v1 scope per D8,
   but architectural shape matters):
  edited node's bounding box,
   expanded to tile boundaries at every level,
  marks tiles dirty.
  Worker repaints in priority order (in-viewport first,
   then nearest,
   then
  prefetch ring around the viewport).

#### Zoom transitions

when the user zooms across an LOD boundary,
the tile being shown is whatever is in cache;
if a higher-resolution tile is not yet ready,
the previous level's tile upscales (blurry-but-instant) until the new tile arrives.
This is the same approach map renderers use (Google Maps,
OpenStreetMap)
and it stays inside the p99 frame budget because the tile-render work is async.

### Why deferred

Two reasons.

First,
 the architecture's complexity is justified only by 1M visible at 60 fps p99.
If phase 0a shows that a simpler architecture
(viewport cull plus instanced rendering,
 no tile pyramid)
clears the bar on the target hardware,
 the entire D6 disappears.
The user picked "1M as hard requirement" but did not commit to the tile pyramid
specifically;
 the architecture is shaped by the target,
 not picked.

Second,
 every other architectural decision (D5,
 D10) depends on D6 being correct.
Validating D6 first lets the others stand on measurement,
 not theory.

Rejected (these stand even if D6 itself stands or falls):

#### Full real-time 1M render every frame, no LOD

At 60 fps this gives 16 ns per node,
 below GPU dispatch granularity for
per-instance work.
No shipped 2D vector editor handles this case;
no public benchmarks place Figma,
 tldraw,
 Penpot,
 or Excalidraw
at 1M visible at 60 fps.

#### Simpler virtualization: render only viewport-intersecting nodes, no LOD

Leaves the worst case (zoomed-out view,
 all 1M visible) unsolved.
At a zoom where every node fits in the viewport,
the worst case is the rated case;
without LOD we still need to render 1M nodes per frame.
Phase 0a tests whether this case is actually reachable without LOD;
if it is,
 this rejection collapses too.

## D7. Document foundation: greenfield

Picked:
 Build the engine and document model from scratch within the monorepo,
under a new package category named in D9.

Rejected:

### Fork Penpot

Penpot's renderer is web-stack (SVG plus Canvas2D and WebGL),
not directly portable to native Rust plus Bevy.
Replacing the renderer is most of the work being undertaken;
porting Penpot's data model costs more than building one fresh
given that we have figma/to-penpot to read Penpot files later if needed.

- **Spike Penpot first to calibrate the wall** (load 100k nodes,
   measure).
  Worth doing as a calibration exercise to learn where existing tools break,
  but not as the project foundation.
  Defer to phase 0 or after as a comparison data point;
  do not block on it.
- **Use,
   fork,
   or contribute to Graphite** (Apache 2.0,
   Rust workspace,
  active alpha as of 2026-05-09 with daily commits;
  verified by reading [Graphite's README](https://github.com/GraphiteEditor/Graphite)
  and `Cargo.toml` in `/tmp/graphite-repo`).
  Graphite is the closest existing analogue:
  Rust,
   GPU-accelerated,
   native-desktop,
   OSS,
   vector-graphics editor.
  Investigated as a serious candidate per user direction.
  Rejected for our use case because Graphite's design philosophy diverges:
  (a) Graphite is built around a node-graph procedural model
  ("nondestructive editing workflow that combines layer-based compositing
  with node-based generative design"),
  not a Figma-style scene-tree component design;
  (b) Their roadmap targets vector + raster + photo editing + motion graphics
  - desktop publishing + VFX compositing as a generalized creative tool,
    not a focused design tool for component-based design systems;
    (c) Their frontend is Svelte (web tech) with a desktop wrapper crate;
    our D1 picked true native,
     and adopting Graphite means inheriting
    the webview-style stack we rejected;
    (d) The "1M visible nodes at 60 fps p99" target is not Graphite's target;
    they have not solved the problem we are trying to solve,
    and contributing 1M-node Figma-style perf upstream means reshaping
    their architecture,
     not adding to it.
    Net:
     Graphite is a different product solving an adjacent problem.
    Greenfield with awareness of Graphite's existence is the answer;
    contribution back is sensible if our work produces general-purpose pieces
    (a Vello render plugin pattern,
     a tile-pyramid library) that fit Graphite's needs.
- **Use,
   fork,
   or contribute to OpenPencil** (MIT,
   AI-native,
   .
  fig file support,
  recent project as of 2026-04 to 2026-05-09;
  multiple ecosystem repos found via `gh search repos openpencil`).
  Different focus:
   AI agents driving design,
   "Design-as-Code,
  "
  "concurrent Agent Teams.
  "
  Their value proposition is the AI orchestration layer,
  not the underlying renderer.
  Not the same niche;
   brief mention here so future sessions do not miss it.
- **Lunacy** (Icons8,
   free desktop design tool,
   closed-source).
  Not OSS;
  rules it out as an integration target for an OSS project.
  Worth knowing it exists as a competitive landscape data point.
- **Fork Inkscape** (mature OSS vector editor,
   GPL v3,
   C++,
   GTK,
   Cairo renderer).
  Inkscape ships a working vector editor today with thousands of users,
  which makes it superficially attractive.
  Rejected because:
  (a) Inkscape's renderer is Cairo (CPU),
   the exact component failing our
  perf target and the one we are rewriting;
   keeping Inkscape's renderer
  defeats the project's purpose;
  (b) The data model and UI are both C++,
   which D2 rules out categorically;
  forking Inkscape and modifying it means writing C++,
   which is the
  cognitive-load reason D2 exists;
  (c) The UI is GTK,
   which conflicts with our winit-based native-window
  approach (D1);
  (d) GPL v3 license forces all code linked against Inkscape to be GPL,
  limiting future relicensing and integration with non-GPL libraries;
  (e) After replacing the renderer,
   the data model,
   and the UI,
   the only
  remaining inheritance is the file format (.
  svg with Inkscape extensions),
  which is straightforward to read without forking.
  Net:
   the time saved by forking is consumed by replacing every part we keep.
- **Fork Krita** (KDE's raster-primary editor,
   GPL v3,
   C++,
   Qt).
  C++ rules it out per D2 alone.
  Independent of D2:
   Krita is raster-primary;
   vector layers are secondary
  in their data model,
   so even if we could touch the C++ source,
  Krita's data model is the wrong starting point for a vector-first design tool.
- **Fork tldraw or Excalidraw** (web-based,
   MIT-licensed).
  Both are JavaScript / TypeScript code targeting Canvas2D or WebGL.
  Web stack conflicts with D1;
   language conflicts with D2.
  Replacing the renderer with native Rust means rewriting the entire app.
- **CRDT-from-day-one** (build the data model on Yjs / Automerge / loro
  even though collab is out of v1 scope per D8).
  Pre-builds for a feature not in scope.
  CRDT data structures cost memory and CPU per operation;
  paying that cost while not using collab violates the
  "apply YAGNI to architecture" rule in `AGENTS.md`.
  Re-evaluate when collab enters scope.

## D8. Engine scope: render plus pan/zoom plus select-to-inspect

Picked:
 Engine done means the user can see a 1M-node document,
navigate it fluidly,
and click a node to view its component data in a side panel.
No editing,
 no marquee,
 no save/load,
 no collaboration.

Rejected for v1:

- **Engine plus basic editing tools** (pen,
   transforms,
   undo/redo).
  Doubles the scope before the rendering thesis is validated.
  Worth doing in v2 once the engine is proven.

### Engine plus real-time collaboration

Triples the scope and introduces CRDT design pressure on the data model
before the data model is even validated against the renderer.

### Full Figma parity

Multi-year scope;
 not the current ask.
Re-evaluate after v1 ships.

### Just-a-viewer (no selection, read-only)

Smallest possible scope.
Rejected because the user explicitly asked for select-to-inspect;
pure viewer does not satisfy the requirement.

### Headless engine library (no UI at all, embeddable into other apps)

Smaller scope but does not deliver a standalone OSS Figma alternative,
which is the project's stated purpose.
Future packaging option,
 not v1.

### Lab-grade research only (paper plus benchmarks, no usable tool)

Validates the rendering thesis but does not deliver a tool.
User wants a working app,
 not a research artifact.

## D9. Package category name: `vector-design`

Picked:
 `package/vector-design/` as the category,
with `engine/` and `app/` as subpackages
(parallels `package/figma/{kiwi,penpot}` in pattern).

Rejected:

### `package/editor/`

Too generic;
 a monorepo of this size will accumulate other "editors"
(rich-text,
 code,
 configuration) that have nothing to do with vector design.

### `package/desktop-app/`

Mirrors `desktop-daemon/` but is too generic for the same reason.
We may have multiple desktop apps over time.

### `package/figma-alt/`

Specific to the comparison;
ties the category name to a competitor whose design we may diverge from.

### `package/design-tool/`

Reasonable alternative.
Picked `vector-design` because it names the technology family (vector graphics)
not the use case;
the engine could host other vector-graphics work
(illustration,
 diagram,
 schematic) without category renaming.

## D10. Path and text rendering library inside Bevy: deferred to phase 0

Picked:
 deferred.
 Phase 0 includes a benchmark of Vello against
hand-rolled SDF / tessellation pipelines on top of Bevy's render graph.

The candidates:

### Vello (Linebender, Apache 2.0 OR MIT)

GPU compute-centric 2D vector renderer using wgpu directly.
Designed for paths,
 gradients,
 text,
 clipping;
 uses prefix-sum algorithms on
compute shaders to parallelize work that tessellation-based renderers
hand to the CPU.
Used in production as the rendering backend for Linebender's Xilem GUI toolkit.
A `bevy_vello` integration crate embeds Vello as a Bevy render plugin,
so adopting Vello does not break D4.

Strengths (verified by reading
[Vello's README](https://github.com/linebender/vello/blob/main/README.md),
clone in `/tmp/vello-repo`):

- Designed for our exact workload (paths,
   gradients,
   MSDF text,
   clips)
- Apache 2.0 OR MIT dual-license,
   compatible with most OSS licenses we might pick
- bevy_vello integration is a maintained Linebender project
- Uses wgpu (matches D3)
- Adopting it shrinks phases 2 and 3 (less custom shader code to write):
  Vello provides path rendering,
   gradients,
   and text out of the library

Weaknesses (also from the README):

- Marked "alpha state" by the maintainers
- Known unfinished work blocking some production use cases:
  - Glyph caching (issue #204):
     matters at 1M-glyph scale
  - Blur and filter effects (issue #476):
     matters for design-tool features
    (drop shadows,
     layer effects)
  - GPU memory allocation strategy (issue #366):
     matters for our 8 GB VRAM cap
  - Conflations artifacts (issue #49):
     rendering correctness
- Published performance figure (177 fps for paris-30k on M1 Max at 1600x1600)
  does not extrapolate to 1M nodes;
   needs measurement on our workload
- Pre-1.0 with API churn

### Hand-rolled on Bevy plus wgpu

Custom render plugins for paths,
 MSDF text,
 and tile pyramid.
Path rendering via lyon (CPU tessellation) plus custom SDF fragment shaders.
Text via swash for shaping,
 MSDF atlas,
 custom fragment shaders.

Strengths:

- API stability under our control
- Tailored to the LOD architecture (D6 plus the 48 px threshold)
- No upstream "alpha" risk

Weaknesses:

- Phases 2 and 3 of the implementation plan (3 to 4 weeks each)
  go to writing the path and text pipelines from scratch
- We own the bug surface
- Less battle-tested than Vello's compute-shader approach for vector

### Phase 0 acceptance criteria for Vello (split per concern)

The earlier draft had a single criterion that conflated Bevy's instancing
throughput with Vello's path-rendering pipeline.
Split into criteria that test what each layer is responsible for:

1.  **Bevy entity scale** (tested in phase 0b regardless of Vello adoption):
    1M Bevy ECS entities at 60 fps p99 on the target hardware,
    with a system that updates 10k randomly per frame.
2.  **Vello path rendering**:
    render a realistic-shape vector benchmark
    (e.g. paris-30k or a synthetic stress scene with comparable path complexity)
    through bevy_vello at 60 fps p99 on the target hardware.
    This validates Vello's compute-shader path pipeline,
    which is what adopting Vello buys us.
3.  **Vello text rendering at our scale**:
    render the visible-glyph count expected after the 48 px LOD threshold
    (or whatever threshold phase 0 settles on)
    at 60 fps p99 with Vello's current glyph caching.
4.  **Feature gap acceptable**:
    Vello's "alpha" caveats
    (blur and filter effects unimplemented,
     glyph-caching optimization in progress,
    GPU memory allocation strategy in progress,
     conflations artifacts)
    do not block v1 use cases.
    D8 scope (render plus pan/zoom plus select-to-inspect) does not require blur
    or filter effects,
     so this criterion is met by inspection,
     not benchmark.

If 1 plus 2 plus 3 pass,
 adopt Vello and shrink phases 2 and 3 of the
implementation plan.
If any fail,
 fall back to hand-rolled pipelines.

### Compounding pre-1.0 risk

Bevy is pre-1.0 (D4) and Vello is alpha (this decision).
Their version compatibility is mediated by bevy_vello,
 a third upstream project.
A breaking change in Bevy's render-graph API can break bevy_vello until upstream
catches up;
 a breaking change in Vello can do the same.
The probability of at least one blocking version-skew event during a multi-month
development cycle is meaningful.

Mitigation strategy:

- Pin all three (Bevy,
   Vello,
   bevy_vello) to known-working versions in `Cargo.toml`.
- Upgrade on the project's schedule,
   not upstream's.
- If bevy_vello lags an upgrade we want,
   maintain a private fork of bevy_vello
  with the compatibility patches;
   contribute back upstream when ready.
- Consider Vello vs hand-rolled as a switchable abstraction in the codebase
  (a render-strategy trait with both implementations) so a forced fallback
  remains a one-flag change rather than a rewrite.

### Rejected outright (within the renderer-library category)

- **Pathfinder** (Mozilla,
   archived).
  Predecessor concept to Vello;
   no longer maintained.
  Vello's README explicitly cites Pathfinder as inspiration and supersedes it.
- **WebRender** (Firefox,
   Rust).
  Designed as a layer compositor inside Servo and Firefox;
  not designed as a standalone 2D vector library.
  Library boundaries do not fit our use without sizable work.

## D11. Hit-testing strategy

Picked:
 A CPU-side BVH (bounding volume hierarchy) over the document's nodes,
maintained independently of the render path.
Click resolution flows:
 screen coordinate → camera inverse → document coordinate
→ BVH query → candidate set → AABB filter → per-pixel test for vector accuracy
(via a color-picker framebuffer when needed).

This approach decouples hit-testing from rendering.
The BVH knows about every node in the document,
regardless of whether the node was live-rendered or rasterized into a tile.
A click in a tile-rendered region resolves to an entity through the BVH,
not through the tile;
the tile is purely visual.

Implications:

- Tile content needs no entity metadata baked in.
  Tiles stay as pure raster textures.
- BVH update cost is incremental:
  rebuild (or refit) only on transform changes,
   not on every frame.
- For nodes below the LOD threshold (D6),
  hit-testing still works (the BVH knows them);
  the UX consequence is that the user cannot directly target individually
  without zooming in,
  matching Figma's behavior.

Rejected:

### Render-coupled hit-test (color picking framebuffer for the whole scene)

Requires a second render pass at hit time;
costly when the document is large and the click is rare.
Vector accuracy is the only place a color-picker pass earns its keep,
and it is used only as a tail of the BVH-driven path.

### Storing entity IDs in tile metadata

Forces tiles to carry per-pixel entity attribution,
doubling tile storage and complicating tile invalidation.
No advantage over the BVH path.

### Linear scan over visible entities

O(N) per click;
 fine for thousands,
 fails for hundreds of thousands
in dense regions.

## Phase 0 plan (hybrid: bare wgpu first, then port to Bevy)

The user's directive:
 build the cheapest possible proof first,
then layer on architectural commitments only after measurement.

### Phase 0a: bare wgpu rendering thesis (3 to 5 days)

No Bevy.
 No ECS.
 No Vello.
Just `wgpu` plus `winit` plus a hand-written render loop.

Build:

1.  A wgpu smoke test:
    render a triangle,
     dispatch a compute shader,
     validate RADV-on-Navi-33
    does not have edge cases that block compute-heavy pipelines.
2.  1M instanced placeholder rectangles in a single draw call.
    Pan and zoom via a camera transform.
    Frustum-cull on CPU (a single AABB test per node,
     no spatial index yet).
    Measure p99 frame time across the zoom range:
    tightest (one node fills viewport) to loosest (all 1M visible).
3.  Add a tile-pyramid prototype only if step 2 cannot hit p99 ≤ 16.67 ms
    in the all-1M-visible case.
    The tile pyramid is a complexity earned by failure on step 2;
    it is not a default.

Decision gates:

- If step 2 hits the budget,
  **D6 collapses entirely**.
  The architecture shrinks to viewport cull plus instanced rendering.
  Phase 0b ports this minimal architecture to Bevy.
- If step 2 fails the budget,
   step 3 measures whether the tile pyramid
  rescues it.
  If yes,
   D6 is justified;
  proceed to phase 0b with the tile pyramid as a real component.
  If no,
   the 1M target is not reachable as stated and we return to the user.

### Phase 0b: port to Bevy (3 to 5 days)

Take whatever architecture phase 0a settled on and port to Bevy:

1.  Replace the bare-wgpu render loop with a Bevy app.
2.  Move the rectangles into Bevy ECS entities
    (single archetype:
     Transform + GlobalTransform + Aabb + RectMarker).
3.  Replicate the phase 0a benchmark;
     measure the delta.
4.  If delta is small (Bevy adds < 10 percent overhead),
     D4 stands.
    If delta is large,
     investigate before committing.
5.  Test ECS at 1M with mixed archetypes
    (rect / path / text placeholders,
     ~250k each)
    to validate D5's small-N archetype assumption.
6.  If Vello adoption is on the table (D10),
     wire bevy_vello in
    and run the D10 split criteria.

### Decision gates after phase 0

After phase 0 measures,
 the deferred decisions become real:

- D5 stands or moves to hand-rolled SoA based on inner-loop count measurement.
- D6 stands or collapses based on phase 0a step 2 result.
- D10 (Vello vs custom) is decided by the split criteria.

Anything in this document that current state marks "deferred"
becomes either confirmed or replaced by a measured-decision section
after phase 0 wraps.

## Conventions for future decisions

When proposing a major technology change to this project:

1.  Add a new D<N> section with the picked option and the rejected
    alternatives.
2.  State the reason for rejection per alternative concretely
    (not "doesn't fit";
     cite the specific incompatibility,
     license clause,
    benchmark,
     or measurement).
3.  Update earlier decisions if a downstream choice forces re-evaluation
    (mark superseded with a pointer to the new decision;
     do not delete).
4.  Reference related decisions inline (e.g. "see D4 for engine pick")
    rather than re-explaining context.
