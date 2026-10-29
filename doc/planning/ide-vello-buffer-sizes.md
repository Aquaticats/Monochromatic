# IDE Vello buffer sizes

Why the Vello renderer of the IDE takes about 262 MiB of video memory for any scene,
which part of that is Vello's own constants and which part is the allocator under it,
and what each way of reducing it would cost.
This is research for the owner; no IDE source and no lockfile was changed.
Every number comes from a command run on 2026-10-20 on the owner's host
(AMD Radeon RX 7600, RADV, Vulkan, Vello 0.11.0, wgpu 30.0.1, wgpu-hal 30.0.1);
what was not run is listed under "Not measured".
All sizes are in MiB (2^20 bytes) unless a count says otherwise.
The blank and corrupt frames at these limits are traced and reproduced in [the Vello note][vello note].

## Result in one paragraph

The 262 MiB are two things.
Vello 0.11 asks for seven bump-allocated buffers whose element counts are constants (165 MiB, as the harness document
found), and wgpu's default `MemoryHints::Performance` hands device memory out in blocks of 128 to 256 MiB, so one more
256 MiB block is reserved the moment the first block is full.
The 97 MiB that were "not attributed" are allocator slack, reserved block space that nothing occupies:
in the IDE-like case (a 1113 by 8160 tile plus the sign) wgpu's own allocator report lists 211.6 MiB allocated inside
448 MiB of reserved blocks.
Setting `memory_hints: MemoryHints::MemoryUsage` on the device (one field in `src/gpu_device.rs`) takes the process
from 458.2 to 234.4 MiB with Vello untouched.
Shrinking the seven constants as well, which needs a patched Vello, takes it to 194.4 (hint `MemoryUsage`) or 162.4
(hint `Manual` 8 to 32 MiB), but the smaller buffers are only safe if the IDE first stops clipping with full-height
rectangles, because Vello 0.11 answers an overflow by silently drawing nothing.

A separate finding matters more than any memory figure, and belongs to whoever owns `src/tile_scene/`:
with the constants as they are today, `split` clips a selected row with rectangles as tall as the whole texture, and a
tile 8160 pixels tall overflows the tiles buffer from 58 selected rows on (56 rows rendered, 58 did not).
A window 2000 pixels tall shows 83 rows, so selecting one screenful already produces a blank or stale tile.
Section "Two findings for the IDE" has the numbers; this was reproduced in a probe that copies the clip structure, not
in the application.

## Where the sizes come from

### The seven constants

`BufferSizes::new` takes the scene's `Layout` and the `WorkgroupCounts`
(`vello_encoding-0.11.0/src/config.rs:363`), and computes the seventeen "known size" buffers from them
(`config.rs:364` to `396`: paths, draw objects, clips, bin headers).
The seven bump-allocated buffers do not use them.
Their sizes are literals, under a comment that admits it
(`config.rs:398` to `408`):

```rust
// The following buffer sizes have been hand picked to accommodate the vello test scenes as
// well as paris-30k. These should instead get derived from the scene layout using
// reasonable heuristics.
let bin_data = BufferSize::new(1 << 18);
let tiles = BufferSize::new(1 << 21);
let lines = BufferSize::new(1 << 21);
let seg_counts = BufferSize::new(1 << 21);
let segments = BufferSize::new(1 << 21);
// 16 * 16 (1 << 8) is one blend spill, so this allows for 4096 spills.
let blend_spill = BufferSize::new(1 << 20);
let ptcl = BufferSize::new(1 << 23);
```

`vello-0.11.0/src/render.rs:205` calls `RenderConfig::new(&layout, params.width, params.height, ...)`,
and `render.rs:219` allocates every buffer from `cpu_config.buffer_sizes`.
The element sizes follow from wgpu's allocator report (section "The allocator under the constants"): lines 24 bytes,
segments 24, tiles 8, segment counts 8, and 4 for the three `u32` buffers.
That gives lines 48, segments 48, ptcl 32, tiles 16, segment counts 16, blend spill 4, bin data 1: 165 MiB,
for a 64 by 64 scene as for a 1113 by 16384 one.

### No public option shrinks them

`RendererOptions` has four fields (`vello-0.11.0/src/lib.rs:375` to `405`): `use_cpu`, `antialiasing_support`,
`num_init_threads`, `pipeline_cache`.
`render_full` is `pub(crate)` (`render.rs:84`), `WgpuEngine` is `pub(crate)` in a private module
(`wgpu_engine.rs:31`, `lib.rs:120`), and the one public low-level entry,
`vello::low_level::Render::render_encoding_coarse`, calls `RenderConfig::new` itself (`render.rs:205`),
so the low-level API cannot override the sizes either.
`AaSupport::area_only()`, which the IDE already passes (`src/native/gpu_tile.rs:309`), only drops the MSAA pipelines;
the allocator report of an area-only renderer lists no MSAA buffer.
`num_init_threads` and `pipeline_cache` concern shader compilation.
`use_cpu: true` does remove memory (measured: 263.1 instead of 456.9 MiB for a 697 by 1392 tile plus the sign, because
lines, tiles and segment counts then live in host memory) but a render took 31 to 37 ms instead of 3 to 4 ms.
That is ten times slower on the CPU; it was measured once per setting, without a spread, and is not a candidate.
The `bump_estimate` feature (`vello_encoding-0.11.0/src/lib.rs:37`, `71`) is instrumentation only:
the renderer never reads it, and glyph runs are not estimated
(`vello-0.11.0/src/scene.rs:436`: `// TODO: Integrate BumpEstimator with the glyph cache.`), so it knows nothing about
the IDE's scenes, which are all glyph runs.

### The allocator under the constants

wgpu-hal sets the block sizes from the device's `memory_hints`
(`wgpu-hal-30.0.1/src/lib.rs:439` to `456`, used at `src/vulkan/adapter.rs:2951`):

- `Performance` (the default): device blocks of 128 to 256 MiB, host blocks of 64 to 128 MiB.
- `MemoryUsage`: device blocks of 8 to 64 MiB, host blocks of 4 to 32 MiB.
- `Manual { suballocated_device_memory_block_size }`: the range given, clamped to 4 to 256 MiB, host half of it.

The IDE builds its device with `..wgpu::DeviceDescriptor::default()` (`src/gpu_device.rs:216` to `219`), so it gets
`Performance`.
Slint offers the same field on a device it creates itself (`WGPUSettings::device_memory_hints`,
`i-slint-core-1.18.1/graphics/wgpu_30.rs:49`), which matters only if a path lets Slint open the device.

`device.generate_allocator_report()` (wgpu 30, `src/api/device.rs:630`) shows the effect for a 1113 by 8160 tile and
the sign, both textures kept:

- `Performance`: 211.6 MiB allocated, 448.0 MiB reserved in blocks of 128, 256 and 64 MiB.
  The allocations of 1 MiB or more are the seven constants (165.0 together), the tile texture (36.2), `vello.scene`
  (4.0), an unlabeled texture (4.1), `vello.tagmonoid_buf` (1.5) and `vello.info_bin_data_buf` (1.0, part of the 165).
- `MemoryUsage`: the same 211.6 allocated, 224.2 reserved.

A device with no renderer already holds 193.6 MiB under `Performance`, 13.6 under `MemoryUsage` and 25.6 under `Manual`
16 to 64 MiB (measured with `drm-memory-vram` right after `request_device`; the harness document found 193 MiB
for a process without a renderer).
Those figures equal one device block plus one host block plus 1.6 MiB (128 + 64 + 1.6, 8 + 4 + 1.6, 16 + 8 + 1.6), and
the report after the renders lists exactly those blocks plus one of 256 MiB.
That the 256 MiB block is added by Vello's first render, because the first 128 MiB block cannot also hold the 96 MiB of
lines and segments buffers and the rest, is an inference from those sums; the report was not taken at each step.
It gives 256 MiB for the block and 6 MiB for small dedicated allocations, which is the 262 MiB measured, and a first
render with the seven constants at 1,024 elements added 6.0 MiB only.
So the unattributed 97 MiB are allocator slack, not buffers.

## What happens when a scene needs more than the buffers hold

### The failure path in 0.11

Each stage that bump-allocates tests its counter against the buffer size and sets a bit in `bump.failed`
(`vello_shaders-0.11.0/shader/shared/bump.wgsl:5` to `9`: binning 1, tile allocation 2, flatten 4, path count 8,
coarse 16):
`binning.wgsl:65` and `72` (lines), `binning.wgsl:160` to `163`, `tile_alloc.wgsl:94` to `98` (tiles),
`coarse.wgsl:73` to `80` (ptcl), `coarse.wgsl:166` (segment counts).
A failed stage makes the later ones return, and `path_tiling_setup.wgsl:22` to `26` writes `ptcl[0] = ~0u`, after which
`fine.wgsl:1070` to `1074` returns before it writes a single pixel:

```wgsl
if ptcl[0] == ~0u {
    // An earlier stage has failed, don't try to render.
```

So a failed frame is all or nothing, and `Renderer::render_to_texture` still returns `Ok(())`
(`lib.rs:476` to `498`): nothing in the IDE would notice.
There is no retry.
`render_to_texture_async_internal` reads the bump buffer back only when `robust` is true, which is
`cfg!(feature = "debug_layers")` (`lib.rs:732`), and carries the comment
`// TODO: apply logic to determine whether we need to rerun coarse, and also allocate the blend stack as needed.`
(`lib.rs:764`).
The only public way to see the counters in 0.11 is the deprecated `render_to_texture_async` with `debug_layers`, and
that path panics in `lib.rs:805` when the lines buffer overflowed (`slice offset 0 size 62601936 is out of range for
buffer of size 50331648`; seen while measuring).

The counters are readable after a render.
`BumpAllocators` (`vello_encoding-0.11.0/src/config.rs:24` to `34`) holds `failed` and the final needed size of each
buffer, with the comment `If any of these are larger than the corresponding _size element reallocation needs to occur`.
After a failure the failed stage's counter still counts past the buffer (`lines` read 2,608,414 for a size of
2,097,152), but the stages after it return early and report 0 (`minimax`: binning, tiles, segments, ptcl all 0), so one
failed render does not show what the later buffers need.

Measured with the probe (section "The probe"), on a fresh texture after a failed frame:
the pixels were all zero (`0,0,0,0`), not even the base color, because the fine stage never ran;
in the application the tile would keep its previous content.
`render_to_texture` returned `Ok` for `minimax` and for the full-height select-all,
both of which fail at the default sizes.

### A silent corruption the failure bits do not cover

The per-tile command lists start with 64 `u32` per 16 by 16 pixel tile (`shared/ptcl.wgsl:6`, `coarse.wgsl:72`),
and the dynamic part starts after them.
`ptcl_size` is checked only against the dynamic part.
When `width_in_tiles * height_in_tiles * 64` exceeds the buffer, no bit is set and the image is wrong:
a 1113 by 16384 tile has 71,680 tiles, 4,587,520 elements, and with `ptcl` at 4,194,304 the render reported
`failed = 0` and 290,992 bytes differed from the reference by up to 184 levels.
With `ptcl` at 4,700,000 the same render differed in 284 bytes by at most one level (the usual noise).
So `ptcl` has a floor of 64 elements per tile (one byte per pixel) plus the dynamic part.
The default 8,388,608 covers 131,072 tiles, which is 33.5 million pixels, twice what the IDE's 64 MiB byte rule allows,
so the floor never binds at the default and binds any smaller value.
This also bears on the harness document's open question whether the 64 MiB byte rule should rise for the GPU tile:
with the default `ptcl`, a texture above 128 MiB (131,072 tiles of RGBA8) corrupts silently with `failed = 0`, so
raising the rule needs a `ptcl` computed from the target.

## What the IDE's scenes need

### The probe

`~/temp/agent/vello-buffers/probe` is a headless program (wgpu 30 and Vello 0.11.0 from the registry, Parley 0.11,
JetBrains Mono, the 560-line `dense.rs` corpus of `~/temp/agent/ide-vello-spike2`, tokenized into six inks as that
spike did).
It renders the scenes of this section, reads the process's own `drm-memory-vram` from `/proc/self/fdinfo`,
and can print the allocator report.
A local copy of `vello_encoding` reads `VELLO_BUMP_SIZES` (seven element counts) in `BufferSizes::new`, and a local
copy of `vello` sets `robust = true` so that `render_to_texture_async` returns the bump counters.
Both live under the scratch directory and reach the probe through `[patch.crates-io]` path entries.
The repository was not touched.

The scenes (the row is 24 px, 15 px glyphs, line numbers in the tile scenes):

- `small`: 64 by 64, two rows.
- `sign`: 1113 by 24.
- `testtile`: 697 by 1392 (the native tests' tile).
- `tile`: 1113 by 8160, 340 rows.
- `tile2x`: 2226 by 7488 at scale 2, 156 rows.
- `tilemax`: 1113 by 16384, 682 rows (the texture side limit).
- `mini`: 120 by 2012 at scale 1/6, 503 rows.
- `mini2x`: 240 by 4024 at scale 1/3, 503 rows.
- `minimax`: 120 by 16384 at scale 1/6, 4096 rows.
- `tilesel8`, `tilesel40`, `tilesel`: the `tile` scene with 8, 40 and all 340 rows selected,
  encoded the way `src/tile_scene/split.rs:98` to `130` does: three `push_clip_layer` clips per selected row, each as
  tall as the texture (`src/tile_scene/encode.rs:59` to `65`), the row drawn once inside each.

### Peak use per scene (default sizes, counters read back)

The counters, in elements, with the default size in parentheses (lines 2,097,152; tiles 2,097,152; segment counts and
segments 2,097,152; dynamic ptcl 8,388,608 minus 64 per tile; binning 262,144):

- `small`: lines 1,482; tiles 8; segments 112.
- `tile` (340 rows): lines 247,104; segments 283,600; tiles 17,004; binning 1,295; ptcl 0.
- `tile2x`: lines 149,312; segments 184,921; tiles 23,197.
- `tilemax` (682 rows): lines 486,893; segments 558,565; tiles 33,453; binning 2,568.
- `mini` (503 rows at 1/6): lines 321,716; segments 311,358; tiles 2,171; binning 733.
- `minimax` (4096 rows at 1/6): lines 2,608,414, which is 124% of the default; the render failed (`failed = 4`,
  flatten).
- `tilesel8` with full-height clips: tiles 310,774; ptcl 391,680.
- `tilesel40` with full-height clips: tiles 1,485,872 (71% of the default); ptcl 1,044,480.
- `tilesel` (select all) with full-height clips: tiles 12,502,344 (6 times the default), ptcl 7,180,800,
  binning 153,902;
  the default render failed (`failed = 2`, tile allocation).
- `tilesel` with each clip cut to its own row's height: tiles 66,504; ptcl 256; binning 6,636; lines 700,422;
  segments 321,702.
  The default render succeeded, and so did the profiles of section "Profiles".

Blend spill was 0 in every run.
Lines per row are about 640 at scale 1/6 for the minimap and about 730 per row in the tile (the count includes the line
numbers); segments are 0.97 to 1.15 times the lines.

### Two findings for the IDE

The full-height clips are the large cost of a selection:
the tile count of a select-all fell from 12.5 million to 66 thousand, and the ptcl from 7.2 million to 256 elements,
when each clip rectangle covers only its row.
The images agreed (117 bytes of 36 million differed by one level, the same noise as between two default renders).
The cost per selected row is the whole tile area, because the three clips of a row together span the texture's width
and every clip spans its height: (310,774 - 17,004) / 8 = 36,721 tiles for 8 rows and (1,485,872 - 17,004) / 40 =
36,722 for 40.
The default 2,097,152 tiles therefore run out after about 56 selected rows on a tile 8160 pixels tall:
`tilesel56` needed 2,073,414 tiles and rendered, `tilesel58` needed 2,146,856 and failed (`failed = 2`).
A shorter tile raises the limit in proportion (a tile 6000 pixels tall allows about 76 rows).
With the default constants, a tile of 340 selected rows already fails, which this probe reproduces but the application
was not run on.
`split` emits the clips when the selected ink differs from the ordinary ink;
`src/selection_ink.rs` picks white or black for the selected ink, which no syntax ink equals in the spike's palette,
so the clips would be emitted and the tile would stay blank or stale.
The fix is small: the `Rect` in `encode` gets the row's top and bottom, which `Item::Clip` would have to carry.
It belongs with whoever owns `src/tile_scene/`.

A minimap slice must stay under about 1,600 rows at scale 1/6 for 1,048,576 lines (about 3,200 rows for the default
2,097,152), and the default already fails at 4,096 rows.
That bound belongs in the minimap rewrite's tile height, not in a buffer size.

## Reducing it: measurements

### Profiles

The element counts, in the order lines, bin data, tiles, segment counts, segments, blend spill, ptcl:

- `default`: 2,097,152; 262,144; 2,097,152; 2,097,152; 2,097,152; 1,048,576; 8,388,608 (165.0 MiB).
- `ide_sized`: 1,048,576; 65,536; 262,144; 1,048,576; 1,048,576; 65,536; 6,291,456 (82.5 MiB).
- `half`: 1,048,576; 131,072; 1,048,576; 1,048,576; 1,048,576; 524,288; 4,194,304 (about 90 MiB).
- `tiny`: 1,024 each (0.07 MiB).

### Correctness against a reference

`big` (lines 4,194,304; bin data 1,048,576; tiles 16,777,216; counts and segments 4,194,304; blend 1,048,576; ptcl
16,777,216) never overflowed in these scenes.
Each profile was rendered in a fresh process and compared byte for byte with `big`:

- `ide_sized` matched `big` within one level (at most 283 bytes in 36 to 73 million differed by one) on `small`,
  `sign`, `tile`, `tile2x`, `tilemax`, `mini`, `mini2x`, and on `tilesel8`, `tilesel40` and `tilesel` with row-height
  clips.
  It failed on `minimax` (lines) and on the three full-height-clip selections (a tiles buffer of 262,144 elements
  cannot hold the 310,774 that `tilesel8` needs).
- `default` matched on everything except `minimax` and the full-height `tilesel`.
- `half` matched except `tilemax` (the ptcl floor, silent corruption) and `tilesel40`, `tilesel` and `minimax`
  with full-height clips.
- Every render that matched differed from `big` by at most one level in at most a few hundred bytes, as two default
  renders do among themselves (92 to 138 bytes in `tile`); the harness document accepted that tolerance.

### Video memory of the whole process

Fresh process, `tile` and `sign` rendered and both textures kept; total `drm-memory-vram` after the second render.
The 21-test-process figure uses `testtile` and `sign`.

- Hint `Performance`, default sizes (today): 458.2 MiB; testtile: 456.9.
- Hint `Performance`, `ide_sized`: 458.2 MiB (the tile pushes the total into a second block); testtile: 200.9.
- Hint `MemoryUsage`, default sizes: 234.4 MiB; testtile: 196.9.
- Hint `MemoryUsage`, `ide_sized`: 194.4 MiB; testtile: 156.9.
- Hint `Manual` 8 to 32 MiB, default sizes: 234.4 MiB; `ide_sized`: 162.4 MiB; testtile `ide_sized`: 124.9.
- Hint `Manual` 16 to 64 MiB, default sizes: 278.4 MiB; `ide_sized`: 166.4 MiB; testtile `ide_sized`: 128.9.
- Hint `MemoryUsage`, `tiny` (what would be left if the seven buffers vanished): 74.4 MiB; testtile: 36.9.

The same run repeated gave the same numbers (458.2 twice, 456.9 twice, 455.6 for a `small` first render).
Render times did not change with the hint or the profile beyond run noise (first tile 9.2 to 12.3 ms in the sixteen
cells of the tile run, the sign 0.7 to 1.4 ms); the spread of an unchanged setting was not measured separately.

For the 21 parallel test processes that the harness document counts at about 9.5 GiB (456.9 MiB each for a test tile
and sign): `MemoryUsage` alone gives 196.9 MiB each, 4.0 GiB; with `ide_sized` 156.9 MiB each, 3.2 GiB; the floor with
`MemoryUsage` and no bump buffers is 36.9 MiB each, 0.8 GiB.

### The integrated GPU

The host's second Vulkan adapter is the Raphael integrated GPU (`AMD Ryzen 9 7950X ... RADV RAPHAEL_MENDOCINO`, picked
with `PROBE_ADAPTER=RAPHAEL`); sysfs `mem_info_vram_total` gives it a dedicated pool of 512 MiB (536,870,912 bytes)
next to the RX 7600's 8 GiB.
The probe reads `drm-memory-gtt` as well: it stayed between 2 and 8 MiB in every run, so the blocks are all counted as
`drm-memory-vram` here too.
For `testtile` and the sign, total `drm-memory-vram` of the process:

- Hint `Performance`, default sizes (today): 448.4 MiB, which is 88% of the 512 MiB pool for one test process.
- Hint `Performance`, `ide_sized`: 192.4 MiB.
- Hint `MemoryUsage`, default sizes: 188.4 MiB; with `ide_sized` 148.4; with `tiny` 28.4.
- Hint `Manual` 8 to 32 MiB: 188.4 with default sizes, 116.4 with `ide_sized`.

So the allocator behaves the same on the integrated device, and the saving of e1 is 260 MiB there (448.4 to 188.4),
against a pool that default Vello nearly fills alone.
A `tile` scene on the integrated device was not run.

## What can and cannot be removed

Can be removed without touching Vello: the allocator slack, 224 MiB in the IDE-like case (458.2 to 234.4), by the
device hint alone.
The 180 MiB the device reserves before Vello exists (193.6 to 13.6) go with it, for the test processes as well.

Can be removed with a Vello patch and a safe use of it: up to 82 MiB of the 165 (to the 82.5 of `ide_sized`),
which shows as 40 MiB more under `MemoryUsage` (234.4 to 194.4) and 72 MiB more under `Manual` 8 to 32 MiB (to 162.4),
because blocks quantize the savings.
`ide_sized` is only valid for tiles up to 1113 by 16384, banded clips,
and minimap slices under about 1,600 rows at 1/6.

Cannot be removed by any size setting: the tile texture (36.2 MiB for 1113 by 8160), `vello.scene` (4.0), the unlabeled
texture (4.1, kept even by `tiny`), `vello.tagmonoid_buf` (1.5) and the other scene-sized buffers (about 25 MiB
beyond the texture in the `tiny` run), and the 13.6 MiB the device reserves even under `MemoryUsage`.
The `ptcl` buffer cannot go below 64 elements per 16 by 16 tile.

## Options, ranked

The letters follow the question that asked for them: (a) leave it, (b) a `[patch]` of Vello, (c) an upstream change,
(d) another Vello version, (e) anything else, which here gave two options, e1 (the device's memory hint) and e2
(`use_cpu`).

Ranking: e1 > b > c > a > d > e2.
e1 beats b because it removes 224 of the 264 MiB that b can reach, with one field and no dependency change,
while b adds a vendored crate, a failure mode that is silent, and an IDE change in `src/tile_scene/`.
b beats c because c gives nothing until a Vello release carries it and its pull request has sat open since 2026-08-31,
while b delivers the savings now and can be deleted when c lands.
c beats a because a rebased pull request or a comment with these figures is cheap and moves the fix to the place that
will maintain it, while a keeps the cost for good; c and a do not exclude each other, a is only the state until c or b.
a beats d because there is no other Vello version to move to, so d is not an option that exists yet.
d beats e2 because e2 saves memory at ten times the render time and moves work onto the CPU.

### Option e1: set the device's memory hint

Saves 224 MiB for the IDE-like process (458.2 to 234.4), 260 MiB for a test process (456.9 to 196.9), and the 180 MiB
of device baseline.
Change: one field, `memory_hints: wgpu::MemoryHints::MemoryUsage` (or `Manual` with a range), at
`src/gpu_device.rs:216` to `219`; the native tests' device is built by the same function.
Risk: Slint's FemtoVG and everything else share the device and its allocator, and smaller blocks mean more driver
allocation calls.
The probe shows no render-time change for Vello, but the application with Slint was not run (see "Not measured").
`Manual` 8 to 32 MiB equals `MemoryUsage` with the default sizes and is better with smaller ones
(162.4 against 194.4), at the price of choosing a number.

### Option b: patch `vello_encoding` and size the buffers

Saves another 40 to 72 MiB on top of e1 (194.4 or 162.4 against 234.4), and 256 MiB for small-scene test processes
under the default `Performance` hint (456.9 to 200.9); it is the only way to leave the hint alone and still shrink the
tests.
Change, smallest form: a copy of `vello_encoding` 0.11.0 (192 KB, 4,905 lines of source) under `[patch.crates-io]`
with a path, and a setter or a profile type read in `BufferSizes::new` (a few dozen lines in `config.rs`; the probe's
reader is ten).
The IDE reaches the patched crate through `vello::low_level::vello_encoding`, so `vello` itself needs no patch.
Pull request 1861 is the clean form of the same change (`BumpAllocationSizes`, `RenderConfig::new_with_bump_sizes`,
`Renderer::set_bump_allocation_sizes`, and it clears the buffer pool when the sizes change);
it touches `vello` as well, so applying it means vendoring `vello` (270 KB) too.
What the repository allows: lockfiles are regenerated by cargo, never edited by hand, so a path patch means running
cargo in `package/ide/app` and reviewing the `Cargo.lock` diff.
The IDE's containers run with `--network=none` after `cargo fetch --locked` (`package/ide/app/mise.toml:9`),
so a path patch of vendored sources builds offline, while a git patch would have to be known to the fetch step.
The repository has a precedent: the file manager carried a `[patch.crates-io]` to a Slint fork
(`doc/handover/file-manager-native-dnd.md`) and deleted it with the old Slint sources
(`doc/handover/file-manager-gtk-build.md:45`).
`doc/planning/ide-vello-text.md:435` puts the same question to the owner for a one-line Slint patch (a vendored crate
and `[patch.crates-io]`, or upstream first), and a search of `doc/` found no recorded answer;
`doc/planning/monorepo-manager-route-research/hcl-tooling.md:926` proposes a patched vendored crate as well.
The IDE-specific rule is still the owner's to state.
Risk: an undersized buffer blanks or corrupts a frame with no error, so b is acceptable only with all of:
clips cut to their row in `encode`; a minimap slice height under the bound; a `ptcl` size computed from the target
(64 elements per tile plus headroom), not a constant; and a way to see a failure.
Seeing one needs a second small patch in `vello` (`robust = true` at `lib.rs:732` and the counters returned from
`render_to_texture`), which the probe already uses and which reads back 32 bytes per frame
(the stall that costs was not measured).
Growing a failed buffer and rendering again needs that readback; it is the design the upstream discussion avoided
because it blocks (issue 366).

### Option c: an upstream change

Pull request 1861, "Make Vello Classic bump-buffer reservations configurable", was opened on 2026-08-31 (author
Quik2007, prepared with an AI tool by its own statement) and is open.
GitHub reported it as conflicting on 2026-10-20, after the repository reorganisation of 2026-09-07 and 2026-09-08
(pull requests 1872 and 1883) put the classic renderer under `research/`.
It keeps the defaults, adds `BumpAllocationSizes`, and states what it does not do: no automatic sizing and no recovery.
Its own test used 60.5 MiB instead of 165.0 MiB of requested capacity and reported `failed == 1` for `mmark_80k`.
Related and open: issue 366 (robust dynamic memory; the maintainers chose "fire and forget" with a later readback),
pull request 606 (robust allocation work in progress, "not shippable" by its own description), issue 541 (estimate the
buffers at resolve time), pull request 708 (one size for `segments` and `seg_counts`).
No merged change sizes the buffers from the scene or retries on overflow;
`research/vello_encoding/src/config.rs` on `main` still has the literals (checked with `gh api` on 2026-10-20; the last
commit to the file is the reorganisation of 2026-09-07).
Owner-side work would be a rebased pull request or a comment with the figures of this document.
It saves nothing in the IDE until a release carries it.

### Option a: leave it

Costs 224 MiB per IDE process that e1 would not, and about 9.4 GiB for 21 parallel test processes on a card of 8 GiB.
No risk and no work.

### Option d: another Vello version

Not available.
Vello 0.10.0 has the same literals (`vello_encoding-0.10.0/src/config.rs:402` to `408`), 0.11.0 is the newest on
crates.io (released 2026-10-02), and `main` has them too.
The sparse-strip renderers in the same repository (`vello_gpu` 0.3.0, `vello_hybrid` 0.2.0) have another architecture;
their video memory and their fit with the IDE's wgpu 30 device were not measured.

### Option e2: run the early stages on the CPU

`RendererOptions::use_cpu` saves 194 MiB in the measured case (263.1 against 456.9) and costs ten times the render time
and CPU load.
The owner decided on 2026-10-19 that nothing in the IDE paints on the CPU; the stages moved here are not painting, but
the cost makes the question moot.

## Not measured

- The application itself: every figure is from the headless probe with its own device.
  Whether Slint's FemtoVG renderer, the glyph atlases and the shared device behave the same under `MemoryUsage` or
  `Manual` was not tried, and neither was the frame cadence under scrolling.
- Other drivers and other GPUs: the integrated Raphael device was measured with the same probe
  (section "The integrated GPU"), but Intel, NVIDIA, other Mesa versions and other operating systems were not,
  and the application was not run on it.
- Scenes with strokes (the wavy underline and dashes of `src/tile_scene/underline.rs`), color glyphs, very long lines
  (a minified file), nested clips deep enough to spill the blend stack, and tabs and wrapping.
  The corpus has lines up to 117 characters.
- Whether the layout drops glyphs beyond the texture's width before encoding.
  If it does not, a line of tens of thousands of characters overflows `lines` at any size listed here.
- The minimap as it will be rewritten: only two stand-in slices (503 and 4096 rows) were rendered.
- The unlabeled 4.1 MiB texture was not traced to its creator.
  It is consistent with the persistent image atlas, 1024 by 1024 RGBA8 (`vello_encoding-0.11.0/src/image_cache.rs:9`,
  `vello-0.11.0/src/render.rs:161` to `174`), but that was not confirmed.
- The select-all failure is reproduced from the clip structure of `split.rs` and `encode.rs`, not in the application.
- Run-to-run spread of the render times.
  Video memory was identical in every repeated run.

## Reproduction

All under `~/temp/agent/vello-buffers/` (scratch, not in the repository):

- `vello_encoding-0.11.0-patched/src/config.rs`: the `VELLO_BUMP_SIZES` reader at the literals.
- `vello-0.11.0-patched/src/lib.rs:732`: `let robust = true;`.
- `probe/`: `cargo build --release --offline` (video memory and allocator report, `PROBE_REPORT=1`, `PROBE_KEEP=1`) and
  the same with `--features bump` into `CARGO_TARGET_DIR=target-bump` (counters and pixel dumps).
  Scenes are arguments; `PROBE_HINT` is `usage` or `manual:8-32`; `PROBE_CLIP=band` cuts the clips to their row;
  `PROBE_CPU=1` selects `use_cpu`.
- `matrix.mjs <scenes>` runs the hint and profile grid (results `matrix2.jsonl`, `matrix3.jsonl`);
  `correctness.mjs` renders every profile with pixels and compares them with `big` (`correctness.jsonl`);
  `bump-default.jsonl` holds the counters of the unmodified sizes.
- `pr1861.diff` is the upstream pull request's patch for reading.

[vello note]: ../troubleshooting/vello-silent-blank-or-corrupt-frames-at-fixed-buffer-limits.md
