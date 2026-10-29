# Vello 0.11.0 returns Ok after drawing a blank or corrupt frame when a scene passes its fixed buffer limits

Tool under test:
`vello`,
`vello_encoding` and `vello_shaders` 0.11.0 (crates.io checksums
`913abde56cab6131b62c0cc69b1a93b671d3c34b3097beb3714fd1dc93be8115`,
`b1fb9548d2b46b0f79137b5b1d208ecbd76d432f74e7a2ba838615d117f11532` and
`2e653222cee49f2571729c3e4b0fa99bc4f826c6bb9487dbb88b825d552b3a27`) on wgpu 30.0.1 with the Vulkan backend,
Mesa 26.2.4 RADV on an AMD Radeon RX 7600,
Linux 7.2.9.
The IDE (`package/ide/app`) uses it to draw the code view into a texture.
Investigated and rerun on 2026-10-20.

Two failures share one cause:
Vello 0.11 sizes its working buffers with literals instead of from the scene or the target,
and a scene that does not fit is not reported.
They differ in what the application sees.

- Variant 1 draws a blank frame (every pixel `0,0,0,0`) and reports `Ok(())`.
  It is known upstream.
- Variant 2 draws a frame with whole tiles missing (base color only),
  with the failure counter at zero,
  and reports `Ok(())`.
  I found no upstream report of it.

Upstream `main` (commit `7d70631ac848925972c85171dc064a803003f796`,
2026-10-20) has the same shaders and the same size logic for both,
and the harness under "Verification" reproduces both there as well.
A fix of about ten lines for variant 2 is prototyped under "Prototype".
The filing decision is "do not file as written";
see "Upstream filing decision".

## Symptom

### Variant 1: blank frame

When a scene needs more lines,
tiles,
segment counts,
binning space or command-list space than a bump buffer holds,
a later stage returns early and the fine stage writes no pixel.
Because the fine stage returns before it writes anything,
a texture keeps what it held before;
a fresh texture reads back as all `0,0,0,0`,
not even the base color (rendering over an existing frame was not run).
`Renderer::render_to_texture` still returns `Ok(())`.

In the harness,
57 selected rows of the IDE's tile clip structure (three full-height clip rectangles per row,
on a 1113 by 8160 target) need 2,101,248 tiles against a 2,097,152 limit:
all 9,082,080 pixels are `0,0,0,0`,
`failed=2`,
and `render_to_texture` returned `Ok`.
56 rows need 2,064,384 tiles and render exactly.
The study's text probe (glyph runs instead of rectangles) found the same edge:
56 rows rendered,
58 did not.

### Variant 2: corrupt frame with failed=0

Every 16 by 16 pixel tile owns 64 `u32` at the start of the command-list buffer (`ptcl`) before any dynamic
allocation.
Nothing checks that the buffer holds all those initial regions.
With the default `ptcl` of 8,388,608 elements the limit is 131,072 tiles,
which is 33,554,432 pixels,
or 128 MiB of RGBA8 texture.
A 4096 by 8208 target has 131,328 tiles:
the last tile row (65,536 pixels,
rows 8192 to 8207) is drawn as the base color `30,30,30,255` instead of its content,
`failed=0`,
and `render_to_texture` returned `Ok`.
A 4096 by 16384 target loses its whole lower half (33,554,432 pixels).

This is silent only when no tile outgrows its initial region.
A scene in which some tile does reaches the one guard that exists,
which sets failure bit 16 and turns the frame blank (variant 1) instead.

### What an application can observe

- `render_to_texture` returns `Result<()>` and carries no counters.
- Only the deprecated `render_to_texture_async` returns `BumpAllocators`,
  and only when the `debug_layers` feature or an instrumented `vello` turns the readback on.
- `render.rs` logs one `log::warn!` ("Trying to paint too large image") when the number of coarse workgroups exceeds
  256 (a workgroup covers a bin of 16 by 16 tiles,
  so about 65,536 tiles).
  It names issue 680,
  which was closed as fixed by pull request 1700 (in 0.10.0),
  it says nothing about `ptcl`,
  and it needs an installed `log` logger.
  I read this code;
  the harness installs no logger,
  so I did not see the warning fire.
- The IDE installs a logger, and the warning fired there (2026-10-22): a tile of 1358 by 12288 pixels counts
  6 by 48 = 288 bins of 256 by 256 pixels, and one of 6838 by 2400 counts 27 by 10 = 270.
  A build that cuts each texture of its tile to at most 256 bins
  (`tile_texture_rows` in `package/ide/app/src/shaped_text/limit.rs`: `256 * floor(256 / ceil(width / 256))`
  rows) logged none in six captures where the build before it logged five lines in each of three.
  The bin count is `ceil(width / 256) * ceil(height / 256)`, so a size on the 256 pixel grid is what the rule
  needs, and 256 bins of 256 by 256 pixels is exactly the 64 MiB that this document keeps a tile under.

## Root cause

### Step 1: the seven bump buffers are literals

```rust
// vello_encoding-0.11.0/src/config.rs:398-408
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

The comment is the maintainers' own statement that the sizes are placeholders.
`RenderConfig::new` copies them into the uniform the shaders read,
and `vello` allocates the buffers from them:

```rust
// vello_encoding-0.11.0/src/config.rs:184-190
                lines_size: buffer_sizes.lines.len(),
                binning_size: buffer_sizes.bin_data.len() - layout.bin_data_start,
                tiles_size: buffer_sizes.tiles.len(),
                seg_counts_size: buffer_sizes.seg_counts.len(),
                segments_size: buffer_sizes.segments.len(),
                blend_size: buffer_sizes.blend_spill.len(),
                ptcl_size: buffer_sizes.ptcl.len(),
```

```rust
// vello-0.11.0/src/render.rs:243-244
        let ptcl_buf =
            ResourceProxy::new_buf(buffer_sizes.ptcl.size_in_bytes().into(), "vello.ptcl_buf");
```

No public option changes them.
`RendererOptions` has four fields (`use_cpu`,
`antialiasing_support`,
`num_init_threads`,
`pipeline_cache`),
`render_full` and `WgpuEngine` are `pub(crate)`,
and the one public low-level entry point,
`Render::render_encoding_coarse`,
calls `RenderConfig::new` itself.
Pull request 1861 (open) would add a setter.

### Step 2: every dynamic stage guards its own counter and sets a failure bit

The bits are defined in one place:

```wgsl
// vello_shaders-0.11.0/shader/shared/bump.wgsl:4-9
// Bitflags for each stage that can fail allocation.
const STAGE_BINNING: u32 = 0x1u;
const STAGE_TILE_ALLOC: u32 = 0x2u;
const STAGE_FLATTEN: u32 = 0x4u;
const STAGE_PATH_COUNT: u32 = 0x8u;
const STAGE_COARSE: u32 = 0x10u;
```

Flatten counts lines and drops writes past the buffer,
and binning turns an overfull lines counter into bit 4:

```wgsl
// vello_shaders-0.11.0/shader/flatten.wgsl:768-785
// Writes a line into a the `lines` buffer at a pre-allocated location designated by `line_ix`.
fn write_line(line_ix: u32, path_ix: u32, p0: vec2f, p1: vec2f) {
    bbox = vec4(min(bbox.xy, min(p0, p1)), max(bbox.zw, max(p0, p1)));
    if line_ix < config.lines_size {
        lines[line_ix] = LineSoup(path_ix, p0, p1);
    }
}

fn write_line_with_transform(line_ix: u32, path_ix: u32, p0: vec2f, p1: vec2f, t: Transform) {
    let tp0 = transform_apply(t, p0);
    let tp1 = transform_apply(t, p1);
    write_line(line_ix, path_ix, tp0, tp1);
}

fn output_line(path_ix: u32, p0: vec2f, p1: vec2f) {
    let line_ix = atomicAdd(&bump.lines, 1u);
    write_line(line_ix, path_ix, p0, p1);
}
```

```wgsl
// vello_shaders-0.11.0/shader/binning.wgsl:64-75
    if local_id.x == 0u {
        let failed = atomicLoad(&bump.lines) > config.lines_size;
        sh_previous_failed = u32(failed);
    }
    // also functions as barrier to protect zeroing of bitmaps
    let failed = workgroupUniformLoad(&sh_previous_failed);
    if failed != 0u {
        if global_id.x == 0u {
            atomicOr(&bump.failed, STAGE_FLATTEN);
        }
        return;
    }
```

Binning sets bit 1 when its bin data does not fit,
tile allocation sets bit 2 when the tiles do not fit,
and the segment-count stage drops writes past `seg_counts_size`:

```wgsl
// vello_shaders-0.11.0/shader/binning.wgsl:159-164
        // element_count is the number of draw objects covering this thread's bin
        var chunk_offset = atomicAdd(&bump.binning, element_count);
        if chunk_offset + element_count > config.binning_size {
            chunk_offset = 0u;
            atomicOr(&bump.failed, STAGE_BINNING);
        }
```

```wgsl
// vello_shaders-0.11.0/shader/tile_alloc.wgsl:92-98
    if local_id.x == WG_SIZE - 1u {
        let count = sh_tile_count[WG_SIZE - 1u];
        var offset = atomicAdd(&bump.tile, count);
        if offset + count > config.tiles_size {
            offset = 0u;
            atomicOr(&bump.failed, STAGE_TILE_ALLOC);
        }
```

```wgsl
// vello_shaders-0.11.0/shader/path_count.wgsl:193-196
            let seg_ix = seg_base + i - imin;
            if seg_ix < config.seg_counts_size {
                seg_counts[seg_ix] = seg_count;
            }
```

Coarse turns an overfull `seg_counts` counter into bit 8 (workgroup 0 records it and every workgroup returns),
and sets bit 16 when a tile outgrows its initial command list and the dynamic part does not fit or when the blend
spill does not fit:

```wgsl
// vello_shaders-0.11.0/shader/coarse.wgsl:161-179
    // Exit early if prior stages failed, as we can't run this stage.
    // We need to check only prior stages, as if this stage has failed in another workgroup, 
    // we still want to know this workgroup's memory requirement.   
    if local_id.x == 0u {
        var failed = atomicLoad(&bump.failed) & (STAGE_BINNING | STAGE_TILE_ALLOC | STAGE_FLATTEN);
        if atomicLoad(&bump.seg_counts) > config.seg_counts_size {
            failed |= STAGE_PATH_COUNT;
        }
        // Reuse sh_part_count to hold failed flag, shmem is tight
        sh_part_count[0] = u32(failed);
    }
    let failed = workgroupUniformLoad(&sh_part_count[0]);
    if failed != 0u {
        if wg_id.x == 0u && local_id.x == 0u {
            // propagate PATH_COUNT failure to path_tiling_setup so it doesn't need to bind config
            atomicOr(&bump.failed, failed);
        }
        return;
    }
```

```wgsl
// vello_shaders-0.11.0/shader/coarse.wgsl:68-86
fn alloc_cmd(size: u32) {
    if cmd_offset + size >= cmd_limit {
        // We might be able to save a little bit of computation here
        // by setting the initial value of the bump allocator.
        let ptcl_dyn_start = config.width_in_tiles * config.height_in_tiles * PTCL_INITIAL_ALLOC;
        var new_cmd = ptcl_dyn_start + atomicAdd(&bump.ptcl, PTCL_INCREMENT);
        if new_cmd + PTCL_INCREMENT > config.ptcl_size {
            // This sets us up for technical UB, as lots of threads will be writing
            // to the same locations. But I think it's fine, and predicating the
            // writes would probably slow things down.
            new_cmd = 0u;
            atomicOr(&bump.failed, STAGE_COARSE);
        }
        ptcl[cmd_offset] = CMD_JUMP;
        ptcl[cmd_offset + 1u] = new_cmd;
        cmd_offset = new_cmd;
        cmd_limit = cmd_offset + (PTCL_INCREMENT - PTCL_HEADROOM);
    }
}
```

```wgsl
// vello_shaders-0.11.0/shader/coarse.wgsl:459-470
    if bin_tile_x + tile_x < config.width_in_tiles && bin_tile_y + tile_y < config.height_in_tiles {
        ptcl[cmd_offset] = CMD_END;
        var blend_ix = 0u;
        if max_blend_depth > BLEND_STACK_SPLIT {
            let scratch_size = (max_blend_depth - BLEND_STACK_SPLIT) * TILE_WIDTH * TILE_HEIGHT;
            blend_ix = atomicAdd(&bump.blend, scratch_size);
            if blend_ix + scratch_size > config.blend_size {
                atomicOr(&bump.failed, STAGE_COARSE);
            }
        }
        ptcl[blend_offset] = blend_ix;
    }
```

The `segments` buffer has no check of its own (`segments_size` is in the uniform and no shader reads it):

```wgsl
// vello_shaders-0.11.0/shader/path_tiling.wgsl:165-172
        if !is_down {
            let tmp = p0;
            p0 = p1;
            p1 = tmp;
        }
        let segment = Segment(p0, p1, y_edge);
        segments[seg_start + seg_within_slice] = segment;
    }
```

That is deliberate.
A maintainer explained it in pull request 673 on 2024-09-25:
"the number of tiles written in path_tiling should always be less than or equal to the count determined in the
path_count stage.
That,
in turn,
is checked against the buffer size in coarse,
and then downstream of that,
path_tiling_setup zeroes out the dispatch of path_tiling,
so effectively it doesn't run."

### Step 3: a set bit blanks the frame and nothing reports it

```wgsl
// vello_shaders-0.11.0/shader/path_tiling_setup.wgsl:20-31
@compute @workgroup_size(1)
fn main() {
    if atomicLoad(&bump.failed) != 0u {
        indirect.count_x = 0u;
        // signal fine rasterizer that failure happened (it doesn't bind bump)
        ptcl[0] = ~0u;
    } else {
        let segments = atomicLoad(&bump.seg_counts);
        indirect.count_x = (segments + (WG_SIZE - 1u)) / WG_SIZE;
    }
    indirect.count_y = 1u;
    indirect.count_z = 1u;
```

```wgsl
// vello_shaders-0.11.0/shader/fine.wgsl:1070-1074
    if ptcl[0] == ~0u {
        // An earlier stage has failed, don't try to render.
        // We use ptcl[0] for this so we don't use up a binding for bump.
        return;
    }
```

The fine stage reads no command list and writes no pixel,
so the frame is all or nothing.
The application is not told:

```rust
// vello-0.11.0/src/lib.rs:476-484
    pub fn render_to_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        scene: &Scene,
        texture: &TextureView,
        params: &RenderParams,
    ) -> Result<()> {
        let (recording, target) = render::render_full(
```

The counters are downloaded only when `robust` is true,
which is `cfg!(feature = "debug_layers")`,
and the code that would rerun the frame is a `TODO`:

```rust
// vello-0.11.0/src/lib.rs:729-732
        // TODO: turn this on; the download feature interacts with CPU dispatch.
        // Currently this is always enabled when the `debug_layers` setting is enabled as the bump
        // counts are used for debug visualiation.
        let robust = cfg!(feature = "debug_layers");
```

```rust
// vello-0.11.0/src/lib.rs:764-766
        // TODO: apply logic to determine whether we need to rerun coarse, and also
        // allocate the blend stack as needed.
        self.engine.free_download(bump_buf);
```

```text
// vello-0.11.0/src/render.rs:94-98
#[cfg(feature = "wgpu")]
/// Create a single recording with both coarse and fine render stages.
///
/// This function is not recommended when the scene can be complex, as it does not
/// implement robust dynamic memory.
```

```rust
// vello-0.11.0/src/render.rs:528-530
        if robust {
            recording.download(*bump_buf.as_buf().unwrap());
        }
```

The maintainers chose this on purpose.
Issue 366 (2023,
still open) records the decision for production use:
"all production uses will use a 'fire and forget' modality",
with the previous contents of the target kept on failure and a non-blocking "get last success/failure" call that the
application polls at the start of the next frame.
Pull request 537 (merged 2024-03-28) added the early-exit logic and said:
"For now,
rendering appears to freeze".
The client-facing half (the readback and the retry) is pull request 606,
open since 2024-06,
described by its author as "not shippable".

### Step 4: variant 2, the initial regions are outside every guard

The layout of a tile's command list:

```wgsl
// vello_shaders-0.11.0/shader/shared/ptcl.wgsl:4-7
// Layout of per-tile command list
// Initial allocation, in u32's.
const PTCL_INITIAL_ALLOC = 64u;
const PTCL_INCREMENT = 256u;
```

Coarse starts each tile at `tile_ix * 64`,
and the dynamic part begins after the initial regions of all `width_in_tiles * height_in_tiles` tiles:

```wgsl
// vello_shaders-0.11.0/shader/coarse.wgsl:190-194
    let tile_x = local_id.x % N_TILE_X;
    let tile_y = local_id.x / N_TILE_X;
    let this_tile_ix = (bin_tile_y + tile_y) * config.width_in_tiles + bin_tile_x + tile_x;
    cmd_offset = this_tile_ix * PTCL_INITIAL_ALLOC;
    cmd_limit = cmd_offset + (PTCL_INITIAL_ALLOC - PTCL_HEADROOM);
```

The only comparison with `ptcl_size` is in `alloc_cmd` (shown in Step 2),
which runs only when a tile outgrows its 64 elements.
A tile that fits its initial region never reaches it.
The end-of-tile write `ptcl[cmd_offset] = CMD_END` and the blend offset write (both shown in Step 2) are
unconditional.
The counter that the shader increments,
`bump.ptcl`,
counts the dynamic part only (`coarse.wgsl:73`),
although the comment on `BumpAllocators` says to compare each counter with its `_size` field,
which is a total:

```rust
// vello_encoding-0.11.0/src/config.rs:27-30
    pub failed: u32,
    // Final needed dynamic size of the buffers. If any of these are larger
    // than the corresponding `_size` element reallocation needs to occur.
    pub binning: u32,
```

So the initial regions of tiles from index `ptcl_size / 64` on are written and read past the end of the buffer.
Fine reads them without a check:

```wgsl
// vello_shaders-0.11.0/shader/fine.wgsl:1086-1094
    var cmd_ix = tile_ix * PTCL_INITIAL_ALLOC;
    let blend_offset = ptcl[cmd_ix];
    cmd_ix += 1u;
    // main interpretation loop
    while true {
        let tag = ptcl[cmd_ix];
        if tag == CMD_END {
            break;
        }
```

Vello compiles its shaders with run-time bounds checks off:

```rust
// vello-0.11.0/src/wgpu_engine.rs:846-855
        // SAFETY: We only call this with trusted shaders (written by Vello developers)
        let shader_module = unsafe {
            device.create_shader_module_trusted(
                wgpu::ShaderModuleDescriptor {
                    label: Some(label),
                    source: wgpu::ShaderSource::Wgsl(wgsl),
                },
                wgpu::ShaderRuntimeChecks::unchecked(),
            )
        };
```

An out-of-bounds access is therefore undefined behavior.
On the RADV driver tested here the writes are dropped and the reads return 0,
which is `CMD_END` (`ptcl.wgsl:13`),
so the tile shows the base color and nothing else.
The harness shows exactly that:
every wrong pixel is `30,30,30,255`,
and the wrong pixels are exactly the tiles with index at or above `ptcl_size / 64` (65,536 pixels for 256 tiles over
the limit).
Other drivers are not tested.
Issue 680 and pull request 1686 report a system hang on Apple hardware for the analogous out-of-bounds bin access,
so "base color only" should not be assumed elsewhere.

The CPU path (`RendererOptions::use_cpu`) has the same layout and indexes a Rust slice,
so it panics instead of drawing a wrong frame:

```rust
// vello_shaders-0.11.0/src/cpu/coarse.rs:54-58
        if self.cmd_offset + size >= self.cmd_limit {
            let ptcl_dyn_start =
                config.width_in_tiles * config.height_in_tiles * PTCL_INITIAL_ALLOC;
            let chunk_size = PTCL_INCREMENT.max(size + PTCL_HEADROOM);
            let new_cmd = ptcl_dyn_start + bump.ptcl;
```

```rust
// vello_shaders-0.11.0/src/cpu/coarse.rs:67-69
    fn write(&mut self, ptcl: &mut [u32], offset: u32, value: u32) {
        ptcl[(self.cmd_offset + offset) as usize] = value;
    }
```

### Readings that were wrong or incomplete

- The first version of this document said a minimap slice must stay under about 1,600 rows at one sixth scale.
  That figure is for the study's reduced 1,048,576-line profile;
  at the default 2,097,152 lines the bound is about 3,200 rows.
  The counters behind it were rerun (321,716 lines for 503 rows,
  2,608,414 lines and `failed=4` for 4,096 rows);
  the bound itself is arithmetic on them.
- That version also gave only 56 and 58 for the selection edge.
  With rectangles the edge is 57 (the harness found it);
  with glyph runs 57 was not measured.
- The 256-bin limit (issue 680) does not explain variant 2.
  Pull request 1700 removed it and is in 0.10.0 (`research/CHANGELOG.md`,
  "Fixed"),
  hence in 0.11.0,
  and the control with oversized `ptcl` renders the 1024-bin target 4096 by 16384 exactly.

### Upstream `main` today

Compared with the installed 0.11.0 sources,
`main` (shallow clone of `https://github.com/linebender/vello`,
commit `7d70631ac848925972c85171dc064a803003f796`) has byte-identical `coarse.wgsl`,
`fine.wgsl`,
`binning.wgsl`,
`path_count.wgsl`,
`flatten.wgsl`,
`path_tiling.wgsl`,
`path_tiling_setup.wgsl`,
`shared/ptcl.wgsl`,
`shared/bump.wgsl`,
`vello_encoding/src/config.rs`,
`vello_research/src/lib.rs`,
`render.rs`,
`wgpu_engine.rs` and `vello_shaders/src/cpu/coarse.rs` (the package paths there are under `research/`).
Only `tile_alloc.wgsl` differs,
by the DirectX 12 workgroup-broadcast change of pull request 1992.
So the missing initial-region check is absent upstream today,
there is no retry for variant 1,
and the literals are unchanged.
The harness reproduces both variants on `main` (see "Verification").

## Verification

Versions under test are in the first paragraph.
The failing behavior is deterministic:
four builds of the harness (the original,
a fresh rebuild from the steps of this document,
`main` built in a container,
and a first version with another output format) printed identical wrong-pixel counts for the variant 2 scenes.

### Harness

A solid-color scene has an exactly known image,
so the harness needs no reference renderer:
it renders,
reads the texture back,
counts pixels that differ from the known color by more than one level,
and prints the allocator counters.
Three scene families:

- `bands:W:H` is one opaque rectangle per 256 pixel row band,
  few commands per tile (variant 2).
- `lines:N` is one rectangle whose top edge is cut into N collinear pieces,
  which flatten into N plus 3 lines.
- `rows:N:full` and `rows:N:band` are the IDE's selection:
  N rows of 24 pixels on 1113 by 8160,
  three clips per row,
  each as tall as the texture (`full`) or as tall as its row (`band`).

Two instrumentation edits of copies of the registry crates are needed;
neither changes behavior.
The first turns on the bump-counter readback (`vello` `lib.rs:732`),
the second lets `VELLO_BUMP_SIZES` override the seven literals so that the cause can be isolated and a reference with
oversized buffers can be rendered.

```toml
# Cargo.toml (first version, only to download the crates)
[package]
name = "vello-buffer-harness"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
vello = "=0.11.0"
pollster = "0.4"

[workspace]
```

The harness source,
saved as `src/main.rs` next to that Cargo.toml:

```rust
// src/main.rs
#![recursion_limit = "256"]
// Headless Vello 0.11.0 harness: renders solid-color scenes whose every pixel is known exactly,
// then counts the pixels that are wrong and prints the allocator counters.
//
// usage: harness <scene>...   scenes: bands:W:H  lines:N  rows:N:full|band
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use vello::kurbo::{Affine, BezPath, Rect};
use vello::peniko::{Color, Fill};
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene, wgpu};

const BASE: [u8; 4] = [0x1e, 0x1e, 0x1e, 0xff];

fn ink(k: u32) -> [u8; 4] {
    [(60 + (k * 37) % 190) as u8, (60 + (k * 91) % 190) as u8, (60 + (k * 53) % 190) as u8, 0xff]
}

fn color(c: [u8; 4]) -> Color {
    Color::from_rgba8(c[0], c[1], c[2], c[3])
}

/// A scene plus a function giving the exact color every pixel must have.
struct Case {
    w: u32,
    h: u32,
    scene: Scene,
    expected: Box<dyn Fn(u32, u32) -> [u8; 4]>,
}

fn build(name: &str) -> Case {
    let p: Vec<&str> = name.split(':').collect();
    let mut scene = Scene::new();
    match p[0] {
        // One opaque rectangle per 256-pixel band: tiles = (w / 16) * (h / 16), few commands per tile.
        "bands" => {
            let (w, h): (u32, u32) = (p[1].parse().unwrap(), p[2].parse().unwrap());
            for k in 0..h.div_ceil(256) {
                let r = Rect::new(0.0, (k * 256) as f64, w as f64, ((k + 1) * 256).min(h) as f64);
                scene.fill(Fill::NonZero, Affine::IDENTITY, color(ink(k)), None, &r);
            }
            Case { w, h, scene, expected: Box::new(|_, y| ink(y / 256)) }
        }
        // One rectangle whose top edge is cut into n collinear pieces: n + 3 flattened lines.
        "lines" => {
            let n: u32 = p[1].parse().unwrap();
            let mut path = BezPath::new();
            path.move_to((16.0, 16.0));
            for i in 1..=n {
                path.line_to((16.0 + 224.0 * i as f64 / n as f64, 16.0));
            }
            path.line_to((240.0, 240.0));
            path.line_to((16.0, 240.0));
            path.close_path();
            scene.fill(Fill::NonZero, Affine::IDENTITY, color(ink(1)), None, &path);
            Case {
                w: 256,
                h: 256,
                scene,
                expected: Box::new(
                    |x, y| {
                        if (16..240).contains(&x) && (16..240).contains(&y) { ink(1) } else { BASE }
                    },
                ),
            }
        }
        // The IDE selection: n rows of 24 px, three clips per row, each clip as tall as the texture ("full")
        // or as tall as its row ("band").
        "rows" => {
            let (n, band): (u32, bool) = (p[1].parse().unwrap(), p[2] == "band");
            let (w, h) = (1113u32, 8160u32);
            for k in 0..n {
                let (top, bottom) = ((k * 24) as f64, ((k + 1) * 24) as f64);
                for (l, r) in [(0.0, 300.0), (300.0, 700.0), (700.0, 1113.0)] {
                    let clip = Rect::new(l, if band { top } else { 0.0 }, r, if band { bottom } else { h as f64 });
                    scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
                    scene.fill(Fill::NonZero, Affine::IDENTITY, color(ink(k)), None, &Rect::new(l, top, r, bottom));
                    scene.pop_layer();
                }
            }
            Case { w, h, scene, expected: Box::new(move |_, y| if y / 24 < n { ink(y / 24) } else { BASE }) }
        }
        other => panic!("unknown scene {other}"),
    }
}

fn read_pixels(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let bytes_per_row = (w * 4).next_multiple_of(256);
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (bytes_per_row * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let data = staging.slice(..).get_mapped_range().expect("mapped");
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for row in 0..h as usize {
        pixels.extend_from_slice(&data[row * bytes_per_row as usize..][..(w * 4) as usize]);
    }
    pixels
}

fn main() {
    let mut description = wgpu::InstanceDescriptor::new_without_display_handle();
    description.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(description);
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN));
    let want = std::env::var("HARNESS_ADAPTER").unwrap_or_default();
    let adapter = adapters
        .iter()
        .find(|a| {
            let i = a.get_info();
            if want.is_empty() { i.device_type == wgpu::DeviceType::DiscreteGpu } else { i.name.contains(&want) }
        })
        .expect("adapter")
        .clone();
    let limits = wgpu::Limits::default().using_resolution(adapter.limits());
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("harness"),
        required_limits: limits,
        ..Default::default()
    }))
    .expect("device");
    println!(
        "adapter={} VELLO_BUMP_SIZES={}",
        adapter.get_info().name,
        std::env::var("VELLO_BUMP_SIZES").unwrap_or_default()
    );
    let mut renderer = Renderer::new(
        &device,
        RendererOptions {
            use_cpu: std::env::var("HARNESS_CPU").is_ok(),
            antialiasing_support: AaSupport::area_only(),
            num_init_threads: None,
            pipeline_cache: None,
        },
    )
    .expect("renderer");

    for name in std::env::args().skip(1) {
        let Case { w, h, scene, expected } = build(&name);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("target"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let params =
            RenderParams { base_color: color(BASE), width: w, height: h, antialiasing_method: AaConfig::Area };

        // The stable entry point: this is what an application calls, and what must report a failure.
        let result = renderer.render_to_texture(&device, &queue, &scene, &view, &params);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let pixels = read_pixels(&device, &queue, &texture, w, h);
        let (mut wrong, mut zero, mut first, mut last, mut sample) = (0u64, 0u64, u32::MAX, 0u32, [0u8; 4]);
        for y in 0..h {
            for x in 0..w {
                let got = &pixels[((y * w + x) * 4) as usize..][..4];
                let want = expected(x, y);
                if got.iter().zip(&want).any(|(a, b)| a.abs_diff(*b) > 1) {
                    wrong += 1;
                    if got == [0, 0, 0, 0] {
                        zero += 1;
                    }
                    if wrong == 1 {
                        sample = [got[0], got[1], got[2], got[3]];
                    }
                    first = first.min(y);
                    last = last.max(y);
                }
            }
        }

        // The deprecated async entry point returns the allocator counters; the instrumented vello reads them back.
        let stop = Arc::new(AtomicBool::new(false));
        let poller = {
            let (device, stop) = (device.clone(), stop.clone());
            std::thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    let _ = device.poll(wgpu::PollType::Poll);
                    std::thread::sleep(std::time::Duration::from_micros(200));
                }
            })
        };
        #[allow(deprecated)]
        let bump = pollster::block_on(renderer.render_to_texture_async(
            &device,
            &queue,
            &scene,
            &view,
            &params,
            vello::low_level::DebugLayers::none(),
        ));
        stop.store(true, Ordering::Relaxed);
        poller.join().unwrap();
        let counters = match bump {
            Ok(Some(b)) => format!("failed={} lines={} tile={} ptcl={}", b.failed, b.lines, b.tile, b.ptcl),
            other => format!("counters={other:?}"),
        };
        let (rows, pixel) = if wrong == 0 {
            ("-".to_string(), "-".to_string())
        } else {
            (format!("{first}..{last}"), format!("{},{},{},{}", sample[0], sample[1], sample[2], sample[3]))
        };
        println!(
            "{name} render={} wrong={wrong} zero={zero} rows={rows} px={pixel} {counters}",
            if result.is_ok() { "Ok" } else { "Err" }
        );
    }
}
```

The second edit,
saved as `instrument-config.diff` in the same directory:

```diff
--- a/src/config.rs
+++ b/src/config.rs
@@ -406,6 +406,24 @@
         // 16 * 16 (1 << 8) is one blend spill, so this allows for 4096 spills.
         let blend_spill = BufferSize::new(1 << 20);
         let ptcl = BufferSize::new(1 << 23);
+        // HARNESS PATCH: VELLO_BUMP_SIZES=lines,bin_data,tiles,seg_counts,segments,blend_spill,ptcl (elements).
+        let (lines, bin_data, tiles, seg_counts, segments, blend_spill, ptcl) =
+            match std::env::var("VELLO_BUMP_SIZES") {
+                Ok(v) => {
+                    let n: Vec<u32> = v.split(',').map(|s| s.trim().parse().unwrap()).collect();
+                    let bin_data = BufferSize::new(n[1].max(layout.bin_data_start));
+                    (
+                        BufferSize::new(n[0]),
+                        bin_data,
+                        BufferSize::new(n[2]),
+                        BufferSize::new(n[3]),
+                        BufferSize::new(n[4]),
+                        BufferSize::new(n[5]),
+                        BufferSize::new(n[6]),
+                    )
+                }
+                Err(_) => (lines, bin_data, tiles, seg_counts, segments, blend_spill, ptcl),
+            };
         Self {
             path_reduced,
             path_reduced2,
```

Then download the dependencies,
copy the two registry crates and apply both edits:

```sh
# shell: run in the directory that holds Cargo.toml, src/ and instrument-config.diff
cargo fetch
cp --recursive ~/.cargo/registry/src/*/vello-0.11.0 vello-0.11.0
cp --recursive ~/.cargo/registry/src/*/vello_encoding-0.11.0 vello_encoding-0.11.0
sed --in-place 's/let robust = cfg!(feature = "debug_layers");/let robust = true;/' vello-0.11.0/src/lib.rs
git apply --directory=vello_encoding-0.11.0 instrument-config.diff
```

Append this to Cargo.toml and build:

```toml
# Cargo.toml (append)
[patch.crates-io]
vello = { path = "vello-0.11.0" }
vello_encoding = { path = "vello_encoding-0.11.0" }
```

```sh
# shell: build
cargo build --release
```

The Cargo.lock that resolved this was wgpu 30.0.1.
The harness takes the Vulkan discrete GPU (`HARNESS_ADAPTER=RAPHAEL` picks the integrated one;
not run).
I rebuilt it from these steps in an empty directory and it printed the same counts as the original build.

### Scenes that render cleanly (default sizes)

```text
$ ./target/release/vello-buffer-harness bands:1024:1024 bands:4096:8192 lines:1000 lines:2097000 \
   rows:8:full rows:56:full rows:340:band
bands:1024:1024 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=16 tile=4096 ptcl=0
bands:4096:8192 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=128 tile=131072 ptcl=0
lines:1000 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=1003 tile=196 ptcl=0
lines:2097000 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=2097003 tile=196 ptcl=0
rows:8:full render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=192 tile=294912 ptcl=391680
rows:56:full render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=1344 tile=2064384 ptcl=1305600
rows:340:band render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=8160 tile=97920 ptcl=0
```

`bands:4096:8192` is exactly 131,072 tiles,
the largest target whose initial regions fit the default `ptcl`.
`lines:2097000` is 2,097,003 flattened lines,
149 under the limit.
`rows:56:full` needs 2,064,384 of 2,097,152 tiles.
`rows:340:band` is the full-height scene with each clip cut to its row:
97,920 tiles,
a select-all of 340 rows.

### Variant 1: scenes that render blank

```text
$ ./target/release/vello-buffer-harness rows:57:full rows:340:full lines:2097200 lines:2600000
rows:57:full render=Ok wrong=9082080 zero=9082080 rows=0..8159 px=0,0,0,0 failed=2 lines=1368 tile=2101248 ptcl=0
rows:340:full render=Ok wrong=9082080 zero=9082080 rows=0..8159 px=0,0,0,0 failed=2 lines=8160 tile=12533760 ptcl=0
lines:2097200 render=Ok wrong=65536 zero=65536 rows=0..255 px=0,0,0,0 failed=4 lines=2097203 tile=0 ptcl=0
lines:2600000 render=Ok wrong=65536 zero=65536 rows=0..255 px=0,0,0,0 failed=4 lines=2600003 tile=0 ptcl=0
```

`failed=2` is tile allocation (bit 2) and `failed=4` is flatten (bit 4).
After a failure the stages that did not run report 0 (`tile=0` after bit 4,
`ptcl=0` after bit 2),
so one failed render does not say what the buffers of those stages need.
The five bits were also exercised by shrinking one buffer at a time with `VELLO_BUMP_SIZES` (`rows:8:full`;
the fields are lines,
bin data,
tiles,
segment counts,
segments,
blend spill,
`ptcl`):

```text
$ VELLO_BUMP_SIZES=2097152,0,2097152,2097152,2097152,1048576,8388608 \
   ./target/release/vello-buffer-harness rows:8:full
rows:8:full render=Ok wrong=9082080 zero=9082080 rows=0..8159 px=0,0,0,0 failed=1 lines=192 tile=0 ptcl=0
```

```text
$ VELLO_BUMP_SIZES=2097152,262144,2097152,1000,2097152,1048576,8388608 \
   ./target/release/vello-buffer-harness rows:8:full
rows:8:full render=Ok wrong=9082080 zero=9082080 rows=0..8159 px=0,0,0,0 failed=8 lines=192 tile=294912 ptcl=0
```

```text
$ VELLO_BUMP_SIZES=2097152,262144,2097152,2097152,2097152,1048576,300000 \
   ./target/release/vello-buffer-harness rows:8:full
rows:8:full render=Ok wrong=9082080 zero=9082080 rows=0..8159 px=0,0,0,0 failed=16 lines=192 tile=294912 ptcl=391680
```

The blend-spill route to bit 16 was not exercised (the harness has no scene that nests blends deeply enough).

### Variant 2: scenes that render with tiles missing and failed=0

```text
$ ./target/release/vello-buffer-harness bands:4096:8208 bands:4096:12288 bands:4096:16384
bands:4096:8208 render=Ok wrong=65536 zero=0 rows=8192..8207 px=30,30,30,255 failed=0 lines=132 tile=131328 ptcl=0
bands:4096:12288 render=Ok wrong=16777216 zero=0 rows=8192..12287 px=30,30,30,255 failed=0 lines=192 tile=196608 ptcl=0
bands:4096:16384 render=Ok wrong=33554432 zero=0 rows=8192..16383 px=30,30,30,255 failed=0 lines=256 tile=262144 ptcl=0
```

Isolating the cause on small targets:
with `ptcl` forced to 131,072 elements (2,048 tiles) the edge moves to exactly 2,048 tiles,
and a scene that needs a dynamic allocation is reported instead of corrupted:

```text
$ VELLO_BUMP_SIZES=2097152,262144,2097152,2097152,2097152,1048576,131072 \
   ./target/release/vello-buffer-harness bands:1024:512 bands:1024:528 bands:1024:1024 rows:8:full
bands:1024:512 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=8 tile=2048 ptcl=0
bands:1024:528 render=Ok wrong=16384 zero=0 rows=512..527 px=30,30,30,255 failed=0 lines=12 tile=2112 ptcl=0
bands:1024:1024 render=Ok wrong=524288 zero=0 rows=512..1023 px=30,30,30,255 failed=0 lines=16 tile=4096 ptcl=0
rows:8:full render=Ok wrong=9082080 zero=9082080 rows=0..8159 px=0,0,0,0 failed=16 lines=192 tile=294912 ptcl=391680
```

The last line is bit 16 and a blank frame (variant 1 behavior):
that scene outgrows initial regions,
so it reaches the guard that exists.
The three `bands` scenes never do.

### Positive controls

The harness detects a wrong frame (the variant catalogs) and it can see a right one:
the same failing scenes with every buffer oversized render with zero wrong pixels
(lines 4,194,304,
bin data 1,048,576,
tiles 16,777,216,
segment counts 4,194,304,
segments 4,194,304,
blend spill 1,048,576 and `ptcl` 16,777,216 elements):

```text
$ VELLO_BUMP_SIZES=4194304,1048576,16777216,4194304,4194304,1048576,16777216 \
   ./target/release/vello-buffer-harness rows:340:full bands:4096:16384 bands:1024:1024
rows:340:full render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=8160 tile=12533760 ptcl=7180800
bands:4096:16384 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=256 tile=262144 ptcl=0
bands:1024:1024 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=16 tile=4096 ptcl=0
```

`rows:340:full` needs 12,533,760 tiles and `ptcl=7,180,800`,
which is why it fails at the defaults and renders when the buffers are large enough.
So the failures are about buffer size and nothing else in the scene.
With the fix of "Prototype" the variant 2 rows turn from wrong to right and the variant 1 rows stay as they are.

### Fixed copy, same commands

```text
$ ./target/release/vello-buffer-harness bands:4096:8208 bands:4096:12288 bands:4096:16384
bands:4096:8208 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=132 tile=131328 ptcl=0
bands:4096:12288 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=192 tile=196608 ptcl=0
bands:4096:16384 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=256 tile=262144 ptcl=0
```

```text
$ VELLO_BUMP_SIZES=2097152,262144,2097152,2097152,2097152,1048576,131072 \
   ./target/release/vello-buffer-harness bands:1024:512 bands:1024:528 bands:1024:1024 rows:8:full
bands:1024:512 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=8 tile=2048 ptcl=0
bands:1024:528 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=12 tile=2112 ptcl=0
bands:1024:1024 render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=16 tile=4096 ptcl=0
rows:8:full render=Ok wrong=0 zero=0 rows=- px=- failed=0 lines=192 tile=294912 ptcl=391680
```

```text
$ ./target/release/vello-buffer-harness rows:57:full rows:340:full lines:2097200 lines:2600000
rows:57:full render=Ok wrong=9082080 zero=9082080 rows=0..8159 px=0,0,0,0 failed=2 lines=1368 tile=2101248 ptcl=0
rows:340:full render=Ok wrong=9082080 zero=9082080 rows=0..8159 px=0,0,0,0 failed=2 lines=8160 tile=12533760 ptcl=0
lines:2097200 render=Ok wrong=65536 zero=65536 rows=0..255 px=0,0,0,0 failed=4 lines=2097203 tile=0 ptcl=0
lines:2600000 render=Ok wrong=65536 zero=65536 rows=0..255 px=0,0,0,0 failed=4 lines=2600003 tile=0 ptcl=0
```

The scene that failed with bit 16 under the forced `ptcl` (`rows:8:full`) renders too,
because the floor also leaves it dynamic room.
The fix does not hide variant 1:
`rows:57:full` and the two `lines` scenes still fail with bits 2 and 4.
The clean scenes stay clean (same output as the table of clean scenes).

### CPU path

```text
$ VELLO_BUMP_SIZES=2097152,262144,2097152,2097152,2097152,1048576,131072 HARNESS_CPU=1 \
   ./target/release/vello-buffer-harness bands:1024:1024
thread 'main' (652405) panicked at ~/.cargo/registry/src/index.crates.io-*/vello_shaders-0.11.0/src/cpu/coarse.rs:68:9:
index out of bounds: the len is 131072 but the index is 131073
```

Without the fix `use_cpu: true` panics at `cpu/coarse.rs:68`,
loud instead of corrupt.
With the fix it renders correctly (counters are not available on that path):

```text
$ VELLO_BUMP_SIZES=2097152,262144,2097152,2097152,2097152,1048576,131072 HARNESS_CPU=1 \
   ./target/release/vello-buffer-harness bands:1024:1024
bands:1024:1024 render=Ok wrong=0 zero=0 rows=- px=- counters=Err(UnavailableBufferUsed("vello.bump_buf", "download"))
```

### Numbers from the study

The study (`doc/planning/ide-vello-buffer-sizes.md`) used a text probe in a scratch directory.
I reran it (same binary,
same environment switches).
These reproduced exactly:
`tile` 17,004 tiles and 247,104 lines;
`tilemax` (1113 by 16384,
682 rows) 33,453 tiles;
`mini` (503 rows at one sixth scale) 321,716 lines,
311,358 segments;
`minimax` (4096 rows) `failed=4` with 2,608,414 lines;
`tilesel8` 310,774 tiles and `ptcl` 391,680;
`tilesel40` 1,485,872 tiles;
`tilesel56` 2,073,414 tiles,
`failed=0`;
`tilesel58` 2,146,856 tiles,
`failed=2`;
`tilesel` (all 340 rows,
full-height clips) 12,502,344 tiles,
`failed=2`;
`tilesel` with each clip cut to its row 66,504 tiles and `ptcl` 256.
These reproduced with a small difference:
the corrupt text frame at 1113 by 16384 with `ptcl` 4,194,304 differed from the oversized reference in 290,939 bytes
(the study:
290,992) by up to 184 levels (same);
with `ptcl` 4,700,000 it differed in 199 bytes by at most one level (the study:
284),
and a default render differed from the reference in 250 bytes by at most one level.
The count of one-level bytes changes by tens from run to run;
the corrupt count changes in the last digits,
the cause does not.
The study's video memory figures were not rerun and are not used here.

## Verified workarounds

### Clip each rectangle to its own row

Replace the full-height clip rectangle `(l, 0, r, h)` by `(l, top, r, bottom)` of the row being drawn.
Verified in the harness:
`rows:340:band` renders exactly with 97,920 tiles,
`rows:340:full` fails with 12,533,760 tiles.
The text probe (`PROBE_CLIP=band`) reproduced 66,504 tiles and `ptcl` 256 for a select-all.
Tradeoffs:
a glyph that extends past its row box (accents,
descenders of a tight line height) is cut at the row edge,
which changes the pixels there;
the harness fills rectangles that stay inside their rows and does not test this,
and the study's check of pixel agreement (117 of 36 million bytes differing by one level) was not rerun.
The clip count and the draw count per row are unchanged.

### Keep the target at 131,072 tiles or fewer at the default size

The limit is a tile count,
not a byte count:
`ceil(width / 16) * ceil(height / 16) <= 131072`,
for RGBA8 that is 128 MiB.
Verified:
`bands:4096:8192` clean,
`bands:4096:8208` wrong.
At exactly 131,072 tiles the default `ptcl` has no room left for dynamic allocation:
`bands:4096:8192` is clean only because no tile outgrows its 64 initial elements,
and a scene that needs one more chunk at that size fails loudly with bit 16 (a blank frame,
variant 1).
Tradeoff:
it caps the texture;
the IDE keeps each texture of its tile at 64 MiB or less,
which is safe by this rule.

### Raise `ptcl` to hold the initial regions (patched `vello_encoding`)

The prototype fix under "Prototype" does this in `RenderConfig::new`.
Verified in the harness on 0.11.0 sources and on `main`.
Tradeoffs:
the application must vendor `vello_encoding` through `[patch.crates-io]`;
the buffer grows to `4 * (64 * tiles + 4,194,304)` bytes,
which is 80 MiB instead of 32 MiB for 4096 by 16384;
targets of 65,536 tiles or fewer keep today's 8,388,608 elements;
the buffer must stay within the device's `max_buffer_size` (268,435,456 bytes by default,
so about 983,040 tiles):
a 16384 by 16384 target then fails at buffer creation with a wgpu validation error naming `vello.ptcl_buf` (observed
with the fix;
without the fix the same target could not be read back by this harness,
so its silent behavior was not measured).

### Detect a failed frame with the counters

Set `robust` to true in `vello` (the harness's first edit) and call `render_to_texture_async`,
whose return value holds `BumpAllocators`.
`failed != 0` means variant 1 happened.
Verified in the harness (every failing row of the variant 1 catalog printed its bit).
Tradeoffs:
it does not detect variant 2 (`failed=0` there);
the API is deprecated and the return type is unstable;
the readback is 32 bytes per frame but needs the device polled from another thread (the harness uses a poller thread)
and its stall was not measured;
retrying with larger buffers needs a way to set them,
which only the patched crate offers.

### Keep a minimap slice under about 3,200 rows at one sixth scale

Derived,
not run at other row counts:
503 rows needed 321,716 lines (about 640 per row),
and the default holds 2,097,152 lines.
4,096 rows failed with `failed=4`.
Tradeoff:
the slice height becomes a rendering constraint of the minimap.

## What does not work

- Trusting the result of `render_to_texture`.
  It is `Ok(())` in every failing row of both catalogs.
- Waiting for the `log::warn!` about "too large image".
  It fires for more than about 65,536 tiles,
  and only if the application installs a logger,
  and it says nothing about `ptcl`.
  Read,
  not run.
- The `debug_layers` feature as a failure detector.
  It also captures and downloads the lines buffer,
  and that download panics when the lines overflowed (`render_to_texture_async`,
  `lib.rs:805`:
  the slice length is `bump.lines` elements,
  past the buffer).
  Issue 720 shows the same wgpu message from the async pipeline.
  From the study and the issue;
  I did not rerun it.
- `use_cpu: true` as a fix.
  It turns variant 2 into a panic (verified in "CPU path") and the study measured ten times the render time.
- Shrinking the buffers by patching `vello_encoding` without all of:
  row-height clips,
  a `ptcl` computed from the target,
  and a way to see a failure.
  An undersized buffer blanks or corrupts a frame with no error.
- The hypothesis that the 256-bin limit (issue 680) causes variant 2.
  Excluded under "Readings that were wrong or incomplete":
  pull request 1700 is in 0.11.0 and the oversized-`ptcl` control renders 1024 bins correctly.
- The hypothesis that an unchecked `segments` overflow is a third silent failure.
  The maintainers' argument (pull request 673,
  quoted in Step 2) says the `seg_counts` check covers it;
  the harness never overflowed `segments` without first failing `seg_counts`.
- Pull request 1686 (checked shaders for binning and coarse).
  It was closed in favor of pull request 1700 and addresses the bin count;
  it would not guard the initial regions.

## Upstream filing artifact (do not file as written)

### Upstream filing decision

The default policy is not to file.
Each check below was made on 2026-10-20.

#### Check of the out-of-scope list

`.out-of-scope/` holds eleven files.
A case-insensitive search for `vello`,
`linebender`,
`wgpu`,
`gpu`,
`render`,
`slint` and `compute` matched one line,
an incidental mention of Slint in `.out-of-scope/cargo-workspace.md`.
No exemption names Vello,
Linebender,
GPU rendering or this bug class.

#### Contribution policy of linebender/vello

- `README.md`:
  "Contributions are welcome by pull request.
  The Rust code of conduct applies."
  There is no `CONTRIBUTING.md`,
  no issue template,
  no pull request template and no code of conduct file in the repository (the community profile reports each as
  missing),
  and `linebender/.github` does not exist.
- Organization guidance in the `linebender/linebender.github.io` repository:
  `content/contributor-guidelines.md` says "To propose a nontrivial change,
  it is better to file an issue first rather than sending a PR",
  and that issue reports are valued and "can be closed if there's no clear action to be taken".
- `content/wiki/llm_policy.md` ("LLM contribution policy for Linebender projects",
  published at `https://linebender.org/wiki/llm-policy/`) allows LLM-assisted contributions with disclosure ahead of
  time,
  in the pull request description.
  It says:
  "we do not allow LLM bots,
  PRs that were generated end-to-end by LLMs,
  or AI-generated PR descriptions";
  "In discussion spaces like Github comments and the Zulip server,
  please avoid posting AI-generated analyses,
  even if you vetted them";
  and "We will not merge agentic markdown files."
  Using LLMs "for debugging,
  exploration,
  testing ... is accepted without disclosure".
- Maintainer behavior under it:
  `linebender/kurbo` issue 607 (a long list of findings from a Claude bug hunt) was closed by Raph Levien on
  2026-09-02 with "Closing as not consistent with our AI policy".
  In `linebender/vello` pull request 1992 (an external contributor's fix to `tile_alloc.wgsl`,
  body ending in "Generated with Claude Code"),
  a member removed it from the merge queue on 2026-10-15 to ask about `Co-authored-by` trailers for LLM entities,
  and Raph Levien force-pushed the branch and merged it the same day (what he changed was not inspected).
  On pull request 1700 (a member's change,
  merged 2026-06-13) Raph Levien wrote "I did use Fable to help me review".
  Pull request 1861 (an external contributor,
  "AI usage:
  This PR was prepared with assistance from an AI coding tool",
  opened 2026-08-31) has had no comment or review in seven weeks and conflicts with `main`.

#### Duplicate search

Run with `gh search issues` and `gh search prs` on `--repo linebender/vello`,
open and closed,
one query per term,
up to 40 results per query.
Terms for both issues and pull requests:
`ptcl`,
`PTCL_INITIAL_ALLOC`,
`ptcl_size`,
`initial allocation ptcl`,
`bump failed`,
`bump.failed`,
`silent corruption`,
`silently`,
`buffer too small`,
`buffer size`,
`tile count`,
`too large image`,
`large render target`,
`blank`,
`black`,
`overflow`,
`out of bounds`,
`robust`,
`reallocate`,
`estimate`,
`lines_size`,
`tiles_size`.
Terms for pull requests also `bump allocation` and `bump buffer`.
Terms for issues also `ShaderRuntimeChecks`,
`unchecked`,
`max texture`,
`big texture`,
`huge`,
`tall`,
`command list`,
`initial region`,
`ptcl overflow`,
`corrupt`,
`garbage`,
`4096`,
`bins`,
`render target size`,
`texture too large`,
`wrong output`,
`missing content`,
`failed bump`.
(The first pass hit GitHub's search rate limit and lost the pull request queries after the first few;
they were rerun in full with a delay between calls.)
Separate searches covered the LLM policy (`AI usage`,
`AI-assisted`,
`LLM`,
`AI policy`,
`LLM policy`,
in the repository and in the organization).
I read the threads that came up in full:

- Variant 1:
  [issue 366][] (robust dynamic memory,
  open),
  [issue 175][] and [pull request 181][] (the 2022 design),
  [pull request 257][] and [pull request 537][] (GPU-side early exit),
  [pull request 606][] (readback and reallocation,
  open,
  "not shippable"),
  [pull request 673][] (segments,
  closed) and [pull request 708][] (same capacity for `segments` and `seg_counts`,
  open),
  [issue 541][] (bump estimation,
  open),
  [issue 720][] (the same blank-frame report from a user,
  open,
  with two later real-world cases) and [pull request 1861][] (configurable sizes,
  open,
  conflicting).
- Large targets:
  [issue 680][] (closed by [pull request 1700][]),
  [issue 548][] (closed,
  not planned,
  an Apple system hang with large windows) and [pull request 1686][] (closed).
- Variant 2:
  no issue or pull request mentions the unchecked initial `ptcl` region.
  [pull request 606][] is the nearest.
  Its diff adds `layout.ptcl_dyn_start = width_in_tiles * height_in_tiles * PTCL_INITIAL_ALLOC` and adds it to the
  needed size in `BumpAllocators::memory`,
  so its author knew the dynamic part starts after the initial regions;
  it also starts the buffer at `1 << 17` elements and grows it only from the dynamic counter.
  From reading the diff (not run),
  a scene that never needs a dynamic allocation would then keep 131,072 elements,
  which is 2,048 tiles.
  I cannot say whether that is intended.
- Keyword search is not proof of absence:
  the repository has 210 open issues and pull requests and GitHub's search is lexical.
  What I can say is that the queries listed here and the threads they surfaced do not contain the claim.

Variant 1 therefore has a duplicate (issues 366 and 720) and variant 2 has none.

#### Constraint 1: is it really upstream's fault?

Variant 1:
yes by design,
and acknowledged.
The literals are labeled placeholders (`config.rs:398`),
the fire-and-forget choice is recorded in issue 366,
and the work to report and retry is open.
Variant 2:
yes.
The maintainers' own design statement in issue 175 is "all memory writes are conditional on the offset being less than
the allocated buffer size",
every other bump region follows it (the guards in Step 2),
and the initial regions are the exception.
The behavior is not wording or a documented restriction:
no document says that the target must have at most 131,072 tiles,
and the only warning is tied to a different limit.

#### Constraint 2: can upstream fix it?

Yes.
The fix for variant 2 is a size computation in the one function that knows both the buffer size and the tile count
(ten lines plus tests,
in `vello_encoding` only).
Variant 1 needs the large readback-and-retry design of issue 366,
which is architectural work but not impossible.

#### Constraint 3: do they support this use case?

Soft yes.
Rendering targets larger than 4096 by 4096 pixels is supported work:
pull request 1700 ("Rendering scenes whose binning requires more than 256 bins") and its changelog entry,
and the wgpu texture limit of 16,384 is what the harness uses.
Against that,
`README.md` and `ARCHITECTURE.md` describe the package published as `vello` as the original compute-centric renderer
that "remains an experimental implementation" and sits under `research/`,
with Vello GPU "intended to become the primary renderer for production GPU use cases".
It still receives fixes (pull requests 1700 in June and 1992 in October 2026,
and the 0.11.0 release on 2026-10-02).

#### Constraint 4: would the repository welcome our contribution?

Yes,
with disclosure;
the drafts in this document are not yet in a form that may be filed.
No ban was found:
the README welcomes pull requests,
the organization policy allows disclosed LLM assistance,
and an LLM-assisted external fix to the same shader family was merged on 2026-10-15.
The same policy asks for human-written descriptions and discussion ("avoid posting AI-generated analyses,
even if you vetted them";
"AI-generated PR descriptions" are not allowed),
and the contributor guidelines prefer an issue before a nontrivial pull request.
Everything in this document was produced by an LLM agent,
so the drafts in this section cannot be filed as they stand.
A human who reproduced the harness runs,
read the source trace and the prototype,
and then wrote the report and the disclosure in their own words could truthfully file.
The disclosure would have to say that an LLM agent found,
traced and prototyped the fix and name what the human verified.

#### Constraint 5: will they likely fix it?

Soft yes.
No document or maintainer statement declines this class of fix,
and no issue or pull request touches the initial regions.
Related buffer-sizing work has stalled for lack of review bandwidth (DJMcNab in issue 720,
2024-10-21:
"Progress on resolving this has stalled due to a shortage of review bandwidth"),
and the nearest recent external change to these shaders was accepted within eight days (pull request 1992).
Nothing here is a signal against,
and nothing guarantees a response (pull request 1861 has none).

#### Constraint 6: have we prototyped a minimal fix compatible with their architecture?

Yes,
for variant 2;
see "Prototype".
For variant 1 nothing was prototyped,
because every candidate piece is already covered by an open thread:
the missing report and retry are issue 366,
issue 720 and pull request 606;
the panic in the `debug_layers` download (`lib.rs:805`) is the wgpu error already pasted in issue 720 and sits in a
path the maintainers call undermaintained;
the unchecked `segments` buffer is deliberate (pull request 673);
and the `bump.ptcl` counter that counts only the dynamic part is accounted for by pull request 606's `ptcl_dyn_start`.

#### Recommendation (the owner decides; nothing was filed)

Do not file now.
The IDE does not need an upstream change:
the row-height clips and the 131,072-tile rule from the workarounds protect it,
and the fix is a small patch it could carry under `[patch.crates-io]` if it ever needs bigger targets.

The options for the owner,
with pros and cons:

- Post nothing upstream.
  Pro:
  no policy friction,
  no maintainer load,
  and the IDE is already protected.
  Con:
  the hazard stays for other users and becomes an ordinary setting if pull request 1861 lands.
- File one new issue for variant 2,
  written by a human from the new-issue draft with the disclosure,
  linking the patch.
  Pro:
  it is the unit the contributor guidelines ask for before a nontrivial change,
  and no duplicate exists.
  Con:
  it costs a human rewrite and a truthful disclosure,
  the response is uncertain,
  and the package is labeled experimental.
- Comment on pull request 1861 with the draft comment.
  Pro:
  it reaches the place where the hazard becomes reachable.
  Con:
  that pull request has had no review in seven weeks and conflicts with `main`,
  and the comment needs the same human rewrite.
- Open a pull request with the patch.
  Pro:
  the least work for maintainers if it is accepted (three small hunks).
  Con:
  the guidelines prefer an issue first,
  and the policy forbids an end-to-end LLM-generated pull request or description.

Ranking:
post nothing now > file the issue > comment on pull request 1861 > open a pull request.
Posting nothing beats the issue because the IDE gains nothing from it today and filing costs a human rewrite;
the issue beats the comment because pull request 1861 is stalled while the issue is the unit the guidelines ask for;
the comment beats the pull request because it needs a few human-written lines instead of a reviewed change set.
Say nothing on the variant 1 threads (nothing to add,
see "Draft").

### Prototype

The fix ([the patch file][fix patch]) is made in a disposable shallow clone of `https://github.com/linebender/vello`
at commit `7d70631ac848925972c85171dc064a803003f796` (origin URL and commit checked before editing;
the clone's push URL was set to `DISABLED`;
no fork exists under our account and none was created).

Why this fix.
The coarse shader cannot repair the buffer it runs in,
and a shader-only check (setting bit 16 when `width_in_tiles * height_in_tiles * 64 > ptcl_size`) would turn the
corrupt frame into a blank one without making the target render,
while needing the same change in the CPU shader copy.
The initial regions are exactly computable from the target size,
unlike the dynamic part that the maintainers say they cannot estimate.
`RenderConfig::new` is the one place that has the tile counts and the buffer sizes together,
and the comment on the literals in `BufferSizes::new` asks for the sizes to be derived from the layout.
So the patch raises `ptcl` to the initial regions plus a dynamic part of at least `1 << 22` elements,
which is what the default size already leaves a 4096 by 4096 target:

- Targets of 65,536 tiles or fewer keep exactly the current size,
  so existing memory use does not change.
- Larger targets get `4 * (64 * tiles + 4,194,304)` bytes of command list,
  up to the device's buffer limit.
- The CPU path and the GPU path are fixed in one place.
- The constant `1 << 22` is a heuristic that the maintainers may want to tune.
- Pull request 1861's setter would need the same floor applied after its sizes are chosen.

The diff has three hunks (two code,
one test module),
so it is kept as a file beside this document.

Verification,
with no upstream script run on the host:

- Harness on 0.11.0 sources (the steps of "Harness",
  then `git apply --directory=vello_encoding-0.11.0 -p3` of the patch):
  the "Fixed copy,
  same commands" outputs,
  variant 2 wrong pixels from 65,536,
  16,777,216 and 33,554,432 to 0,
  and from 16,384 and 524,288 to 0 with `ptcl` forced to 131,072.
- The same harness built against the `main` clone (instrumented the same way) inside a container
  (`podman run --rm --userns=keep-id --network=none --memory=4g --cpus=4`,
  only the clone,
  the harness and a cargo home mounted,
  no environment secrets,
  dependencies fetched beforehand) and run on the host GPU:
  `main` without the fix reproduces both variants with the same counts as 0.11.0 (`bands:4096:8208` wrong=65536
  `failed=0`,
  `rows:57:full` `failed=2`,
  `use_cpu` panic at `cpu/coarse.rs:68`),
  `main` with the two code hunks of the patch renders the variant 2 scenes exactly and still fails the variant 1
  scenes.
- Upstream unit tests,
  in the same kind of container:
  `cargo test -p vello_encoding --lib` passes 33 tests with the patch.
  With only the `ptcl` hunks reverted,
  the new test `ptcl_holds_the_initial_region_of_every_tile` fails and the control
  `ptcl_keeps_the_default_size_up_to_4096_by_4096_pixels` passes,
  so the test detects the missing floor.
- Not run:
  formatting by the upstream's pinned toolchain and clippy (the container image has neither;
  `rustfmt --check` from the host,
  with the repository's `rustfmt.toml`,
  accepts the patched file),
  the `vello_research_tests` snapshot suite,
  and any non-Vulkan or non-RADV driver.

### Draft (do not file as written)

The new-issue draft for variant 2 follows.
It was written by an LLM agent and the Linebender policy asks for a human-written report with disclosure,
so a human must rewrite it in their own words before anything is posted.

~~~md
Title: `ptcl` can be smaller than the initial command lists of the target's tiles (tiles missing, `failed == 0`)

Labels: (none; the repository has no issue template)

LLM content: an LLM agent (Claude) found this, traced the shaders, wrote the reproduction harness and the patch.
I ran the harness on my machine and read the trace and the patch; I wrote this text myself. (To be adapted by the
human filer; state what you verified.)

Vello 0.11.0 and `main` at 7d70631, Vulkan, RADV (Mesa 26.2.4), AMD RX 7600.

Each 16 by 16 tile owns `PTCL_INITIAL_ALLOC` (64) `u32` at the start of the `ptcl` buffer
(`shader/shared/ptcl.wgsl:6`, `coarse.wgsl:193`), and the dynamic part starts after all of them
(`coarse.wgsl:72`). `config.ptcl_size` is compared with the buffer only in `alloc_cmd` (`coarse.wgsl:74`), which runs
only when a tile outgrows its 64 elements. If `width_in_tiles * height_in_tiles * 64 > ptcl_size`, the initial
regions of the last tiles are written and read out of bounds (shaders are `ShaderRuntimeChecks::unchecked()`), no
failure bit is set, and `render_to_texture` returns `Ok(())`.

`BufferSizes::new` uses `1 << 23` elements for any target, so this starts at 131,072 tiles (for example 4096 by 8208).

Reproduction (solid rectangles, exact expected pixels): a harness that renders `bands:W:H` and counts wrong pixels
(source: <link to a gist or repository the filer controls>).

- 4096 by 8192 (131,072 tiles): 0 wrong pixels.
- 4096 by 8208 (131,328 tiles): 65,536 wrong pixels (the last tile row shows the base color), `failed == 0`.
- 4096 by 16384: 33,554,432 wrong pixels (the lower half), `failed == 0`.
- With oversized buffers the same scenes render exactly. With `ptcl` forced to 131,072 on a 1024 by 528 target,
  16,384 pixels are wrong, i.e. exactly the tiles at or above index 2,048.

On the RADV driver the missing tiles show the base color (reads past the end return 0, which is `CMD_END`). Other
drivers are untested; the analogous out-of-bounds access in the bin stage hung Apple GPUs (#680, #1686).

It is not the 256-bin limit: #1700 is in 0.10.0 and the oversized-`ptcl` control renders 1024 bins correctly.

Suggested fix: size `ptcl` in `RenderConfig::new` as `max(1 << 23, width_in_tiles * height_in_tiles * 64 + (1 << 22))`.
Targets up to 4096 by 4096 keep the current size. The patch also adds two unit tests (one fails without the fix).
(patch attached or linked by the filer.) I would also guard #1861's setter with the same floor.

Related: #366, #606 (its `ptcl_dyn_start` accounting), #1861, #680.
~~~

The additive comment draft for pull request 1861 (post only if that pull request moves,
rewritten by a human):

~~~md
Draft comment on #1861 (do not post as written)

With this API, a `ptcl` smaller than `width_in_tiles * height_in_tiles * 64` makes the render silently wrong: the
initial per-tile command lists are outside the only `ptcl_size` check (`coarse.wgsl:74`), so `failed` stays 0 and
the tiles past the end are drawn as the base color. I reproduced it on 0.11.0 by forcing `ptcl` to 131,072 on a 1024
by 528 target (16,384 wrong pixels, `failed == 0`). The profile in this description has `ptcl: 1 << 22`, which is
safe only up to 65,536 tiles (4096 by 4096). A clamp such as `ptcl.max(tiles * 64 + dynamic_min)` after the sizes are
chosen in `RenderConfig::new_with_bump_sizes` would prevent it.
~~~

For variant 1 there is nothing to add.
Issues 366 and 720 and pull requests 606 and 1861 already contain the cause,
the design decision,
the missing readback and retry,
and several real-world cases (two in issue 720 answer the maintainers' request for them).
The IDE's selection scene is one more such case,
and a "me too" is not additive.
The measurement that is specific to this application (the cost of a full-height clip rectangle is the whole target's
tiles) is a property of Vello's clip handling that the maintainers know.

## Not verified and limits

- Only Vello 0.11.0 and `main` at 7d70631,
  only Vulkan on RADV,
  only an AMD RX 7600.
  Other drivers and other GPUs may not show "base color" for the missing tiles of variant 2,
  and the out-of-bounds accesses are undefined behavior.
- The IDE itself was not run on these failures;
  the blank frame was reproduced with a harness that copies its clip structure,
  and with the study's text probe,
  not the application (an agent is reproducing it in the application;
  see `doc/handover/slint-ide-0x.md`).
- The `log::warn!` of `render.rs:207`,
  the `debug_layers` panic and the study's pixel-agreement check of row-height clips were read or taken from the study
  and not rerun.
- The blend-spill route to failure bit 16,
  strokes,
  color glyphs and very long lines were not exercised.
- The reading of pull request 606's `ptcl` start size was not run.
- Targets of 16384 by 16384:
  only the fixed behavior (a validation error at buffer creation) was seen.
- The two unit tests of the patch were run only on `main` (container,
  `cargo test -p vello_encoding --lib`);
  the 0.11.0 registry copies were built with `cargo build`,
  which skips test code,
  and were not tested.

## Related

- The study with the video memory measurements and the option ranking:
  `doc/planning/ide-vello-buffer-sizes.md`.
- The handover that tracks the IDE reproduction:
  `doc/handover/slint-ide-0x.md`.

[fix patch]: vello-silent-blank-or-corrupt-frames-at-fixed-buffer-limits.patch
[issue 175]: https://github.com/linebender/vello/issues/175
[issue 366]: https://github.com/linebender/vello/issues/366
[issue 541]: https://github.com/linebender/vello/issues/541
[issue 548]: https://github.com/linebender/vello/issues/548
[issue 680]: https://github.com/linebender/vello/issues/680
[issue 720]: https://github.com/linebender/vello/issues/720
[pull request 181]: https://github.com/linebender/vello/pull/181
[pull request 257]: https://github.com/linebender/vello/pull/257
[pull request 537]: https://github.com/linebender/vello/pull/537
[pull request 606]: https://github.com/linebender/vello/pull/606
[pull request 673]: https://github.com/linebender/vello/pull/673
[pull request 708]: https://github.com/linebender/vello/pull/708
[pull request 1686]: https://github.com/linebender/vello/pull/1686
[pull request 1700]: https://github.com/linebender/vello/pull/1700
[pull request 1861]: https://github.com/linebender/vello/pull/1861
