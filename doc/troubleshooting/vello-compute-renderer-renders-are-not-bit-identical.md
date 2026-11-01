# Vello 0.11 (compute renderer) draws one scene to slightly different pixels on every render, because tile segments are ordered by GPU atomics and summed in floating point

Tool under test:
Vello 0.11.0 (`vello`,
`vello_encoding` 0.11.0,
`vello_shaders` 0.11.0;
git commit
`f3000c8d9e7a9c7e6abb09b9587238bf8b4860ff`,
tag `v0.11.0`,
package path `research/vello_research`) on wgpu 30.0.1
with the Vulkan backend,
`AaConfig::Area`,
on an AMD RX 7600 (RADV),
the Ryzen 9 7950X integrated GPU (RADV) and
llvmpipe,
Fedora 44 with Mesa.
`package/ide/app` draws its code view with it.
The study that found it is the section "Flakes after the merge" of `doc/planning/ide-gpu-snapshot-harness.md`,
cause 1.

## Symptom

Rendering one unchanged scene again into the same texture does not give the same texels.
The difference is at most one level of one channel (premultiplied),
in a few hundred of about a million texels,
and no error is reported.
The IDE's tests compared two window frames with `==` and failed at random because of it;
the study measured,
in the
test `repeated_renders_of_one_tile_spread`
(`package/ide/app/src/native/gpu_texture_probe_tests.rs:159`),
a 697 by 1392 tile rendered 200 times:
all 200 differed from the first,
by 10 to 32 texels and one level at most,
always the same few texels
(for example `x` 199 and 200 of rows with `y` modulo 16 equal to 8,
alpha 57 against 58).
I did not rerun that test for this document (see the limits);
the standalone program below reproduces the behavior.

Program output,
697 by 1392 texture,
60 lines of JetBrains Mono at 13.5 px,
unhinted,
one translucent rectangle inside
a clip layer,
each render compared with the first:

```text
adapter AMD Radeon RX 7600 (RADV NAVI33)
200 renders compared with the first (one Renderer): 200 differ, worst difference 1 level(s)
differing texels per differing render: min 224 max 312
distinct texel positions that ever differed: 1354
y modulo 16 of those positions: {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15}
  texel (24, 13) differed in 18 of 200 renders
```

The other runs of the same program:

- A new `Renderer` for every render (50 renders,
  RX 7600):
  50 differ,
  245 to 311 texels,
  worst 1 level.
- Raphael integrated GPU (50 renders):
  50 differ,
  230 to 296 texels,
  1029 distinct positions,
  worst 1 level.
- llvmpipe (20 renders):
  20 differ,
  31 to 50 texels,
  239 distinct positions,
  worst 1 level.

The positions of the unstable texels depend on the scene;
this scene has fractional glyph positions and a different
set of ties from the IDE's test,
so its positions are not the ones in the study.

## Root cause

The area-coverage path of Vello's compute renderer sums,
for every pixel,
the coverage contributions of the path
segments of its tile in floating point,
in the order the segments were stored.
That order is decided by an atomic counter that the invocations of an earlier stage increment concurrently.

Stage one counts and places segments.
`path_count.wgsl` gives each segment its slot inside its tile with an atomic increment,
in whichever order the
invocations happen to reach it:

```wgsl
// vello_shaders-0.11.0/shader/path_count.wgsl:189-195
            let seg_within_slice = atomicAdd(&tile[base + x].segment_count_or_ix, 1u);
            // Pack two count values into a single u32
            let counts = (seg_within_slice << 16u) | subix;
            let seg_count = SegmentCount(line_ix, counts);
            let seg_ix = seg_base + i - imin;
            if seg_ix < config.seg_counts_size {
                seg_counts[seg_ix] = seg_count;
```

Stage two writes each segment at that slot:

```wgsl
// vello_shaders-0.11.0/shader/path_tiling.wgsl:170-171
        let segment = Segment(p0, p1, y_edge);
        segments[seg_start + seg_within_slice] = segment;
```

So two renders of one scene store the same segments of a tile in different orders.
The fine stage then adds their contributions one after another,
as `f32`:

```wgsl
// vello_shaders-0.11.0/shader/fine.wgsl:1013-1044 (abridged)
    for (var i = 0u; i < n_segs; i++) {
        let seg_off = fill.seg_data + i;
        let segment = segments[seg_off];
        ...
                let a = (b + 0.5 * (d * d - c * c) - xmin) / (xmax - xmin);
                area[i] += a * dy;
        ...
        let y_edge = sign(delta.x) * clamp(xy.y - segment.y_edge + 1.0, 0.0, 1.0);
        for (var i = 0u; i < PIXELS_PER_THREAD; i += 1u) {
            area[i] += y_edge;
        }
    }
```

Floating-point addition is not associative,
so a different order changes the last bits of `area[i]`,
and the blend and the store to an 8-bit texture (`textureStore(output, ..., rgba_sep)`,
`fine.wgsl:1395`) turn a
change in the last place into one level when the value sits on a rounding tie.
This is the same reading as the study's ("a coverage on a rounding tie,
where the order in which the GPU's atomics
put the segments of a tile in order changes the float sum in its last place").

I tested the reading with a probe instead of leaving it as inference:
the probe patch
([`vello-compute-renderer-renders-are-not-bit-identical.patch`](vello-compute-renderer-renders-are-not-bit-identical.patch))
changes only `fill_path` in `fine.wgsl` so that each segment's contribution is rounded to a 16.16 fixed-point integer
and summed as an `i32`.
Integer addition is associative,
so the order of the segments can no longer change the sum.
With it,
the program's renders are bit-identical (see the verification),
which shows that the summation order is
enough to explain the differences in this scene.
It does not show that it is the only source in other scenes.

## Verification

Versions:
`vello` 0.11.0 and `vello_shaders` 0.11.0 from the registry,
wgpu 30.0.1 (`wgpu-core` 30.0.1),
`skrifa` 0.44.0,
rustc 1.101.0-nightly (2026-10-17),
Mesa 26.2.x,
font `JetBrainsMono-Regular.ttf`
(any monospace TrueType font works,
with different tie positions).

```toml
# vello-noise/Cargo.toml
[package]
name = "vello-noise"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
vello = { version = "=0.11.0", default-features = false, features = ["wgpu"] }
wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"] }
skrifa = "=0.44.0"
pollster = "0.4"
```

```rust
// vello-noise/src/main.rs
// Renders one Vello 0.11 scene (60 lines of monospace text, unhinted, at fractional positions) into a 697 by 1392
// texture many times on one renderer, reads every render back, and compares each with the first.
//
// Usage: vello-noise <font file> [renders] [fresh|shared] [adapter name substring]
//   renders: how many renders to compare with the first (default 200)
//   fresh:   build a new Renderer for every render instead of reusing one
//   adapter: default is the high-performance adapter
use std::sync::Arc;

use skrifa::instance::{LocationRef, Size};
use skrifa::{FontRef, MetadataProvider};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Blob, Color, Fill, FontData};
use vello::wgpu;
use vello::{AaConfig, AaSupport, Glyph, RenderParams, Renderer, RendererOptions, Scene};

const WIDTH: u32 = 697;
const HEIGHT: u32 = 1392;

fn scene_of(font_bytes: &Arc<Vec<u8>>) -> Scene {
    let font = FontData::new(Blob::new(font_bytes.clone()), 0);
    let font_ref = FontRef::new(font_bytes.as_slice()).expect("a font");
    let charmap = font_ref.charmap();
    let size = 13.5_f32;
    let metrics = font_ref.glyph_metrics(Size::new(size), LocationRef::default());
    let text = "let value = compute_total(first_item, second_item, third_item); // word word word";
    let mut scene = Scene::new();
    for line in 0..60 {
        let mut x = 0.0_f32;
        let glyphs: Vec<Glyph> = text
            .chars()
            .map(|character| {
                let id = charmap.map(character).expect("a glyph");
                let glyph = Glyph { id: id.to_u32(), x, y: 0.0 };
                x += metrics.advance_width(id).expect("an advance");
                glyph
            })
            .collect();
        scene
            .draw_glyphs(&font)
            .font_size(size)
            .transform(Affine::translate((4.25, 18.75 + f64::from(line) * 22.0)))
            .hint(false)
            .brush(Color::from_rgba8(220, 220, 220, 255))
            .draw(Fill::NonZero, glyphs.into_iter());
    }
    // A clip, as the selection and annotations of the IDE use.
    scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &Rect::new(100.0, 0.0, 400.0, f64::from(HEIGHT)));
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(40, 90, 200, 120),
        None,
        &Rect::new(120.5, 100.25, 380.5, 700.75),
    );
    scene.pop_layer();
    scene
}

fn read_back(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<u8> {
    let padded = (WIDTH * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(padded) * u64::from(HEIGHT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |result| result.expect("map"));
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let mapped = slice.get_mapped_range().expect("mapped range");
    let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for row in 0..HEIGHT as usize {
        let start = row * padded as usize;
        pixels.extend_from_slice(&mapped[start..start + (WIDTH * 4) as usize]);
    }
    pixels
}

fn options() -> RendererOptions {
    RendererOptions {
        use_cpu: false,
        antialiasing_support: AaSupport::area_only(),
        num_init_threads: None,
        pipeline_cache: None,
    }
}

/// Premultiplied difference of two straight RGBA texels,
the largest over the four channels.
fn difference(left: &[u8], right: &[u8]) -> u8 {
    let premultiply = |value: u8, alpha: u8| (u32::from(value) * u32::from(alpha) / 255) as u8;
    (0..3)
        .map(|channel| {
            premultiply(left[channel], left[3]).abs_diff(premultiply(right[channel], right[3]))
        })
        .max()
        .unwrap()
        .max(left[3].abs_diff(right[3]))
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let font_bytes = Arc::new(std::fs::read(&arguments[1]).expect("read the font file"));
    let renders: usize = arguments.get(2).map_or(200, |value| value.parse().expect("a count"));
    let fresh = arguments.get(3).is_some_and(|value| value == "fresh");

    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(descriptor);
    let wanted = arguments.get(4).map(|name| name.to_lowercase());
    let adapter = match wanted {
        Some(name) => pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
            .into_iter()
            .find(|adapter| adapter.get_info().name.to_lowercase().contains(&name))
            .expect("an adapter with that name"),
        None => pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .expect("an adapter"),
    };
    println!("adapter {}", adapter.get_info().name);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
        ..Default::default()
    }))
    .expect("a device");
    let scene = scene_of(&font_bytes);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("tile"),
        size: wgpu::Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let params = RenderParams {
        base_color: Color::from_rgba8(0, 0, 0, 0),
        width: WIDTH,
        height: HEIGHT,
        antialiasing_method: AaConfig::Area,
    };
    let mut shared = Renderer::new(&device, options()).expect("renderer");
    let mut render_once = |device: &wgpu::Device| {
        if fresh {
            let mut renderer = Renderer::new(device, options()).expect("renderer");
            renderer.render_to_texture(device, &queue, &scene, &view, &params).expect("render");
        } else {
            shared.render_to_texture(device, &queue, &scene, &view, &params).expect("render");
        }
        read_back(device, &queue, &texture)
    };
    let first = render_once(&device);
    // `DUMP=path` writes the first render as raw RGBA (697 by 1392) so that two builds can be compared.
    if let Ok(path) = std::env::var("DUMP") {
        std::fs::write(path, &first).expect("write the dump");
    }
    let mut renders_that_differ = 0;
    let mut worst = 0u8;
    let mut positions = std::collections::BTreeMap::<(u32, u32), usize>::new();
    let mut counts = Vec::new();
    for _ in 0..renders {
        let again = render_once(&device);
        let mut differing = 0;
        for (index, (left, right)) in first.chunks_exact(4).zip(again.chunks_exact(4)).enumerate() {
            let delta = difference(left, right);
            if delta > 0 {
                differing += 1;
                worst = worst.max(delta);
                *positions.entry((index as u32 % WIDTH, index as u32 / WIDTH)).or_default() += 1;
            }
        }
        if differing > 0 {
            renders_that_differ += 1;
        }
        counts.push(differing);
    }
    println!(
        "{renders} renders compared with the first ({}): {renders_that_differ} differ, worst difference {worst} level(s)",
        if fresh { "a new Renderer each time" } else { "one Renderer" }
    );
    println!(
        "differing texels per differing render: min {} max {}",
        counts.iter().filter(|count| **count > 0).min().unwrap_or(&0),
        counts.iter().max().unwrap_or(&0)
    );
    println!("distinct texel positions that ever differed: {}", positions.len());
    let rows: std::collections::BTreeSet<u32> = positions.keys().map(|(_, y)| y % 16).collect();
    println!("y modulo 16 of those positions: {rows:?}");
    for ((x, y), times) in positions.iter().take(8) {
        println!("  texel ({x}, {y}) differed in {times} of {renders} renders");
    }
}
```

```sh
# Run from the crate directory.
cargo build --release
./target/release/vello-noise "${HOME}/.local/share/fonts/JetBrainsMono-Regular.ttf" 200
./target/release/vello-noise "${HOME}/.local/share/fonts/JetBrainsMono-Regular.ttf" 50 fresh
./target/release/vello-noise "${HOME}/.local/share/fonts/JetBrainsMono-Regular.ttf" 50 shared raphael
./target/release/vello-noise "${HOME}/.local/share/fonts/JetBrainsMono-Regular.ttf" 20 shared llvmpipe
```

### Runs that fail (stock `vello` 0.11.0)

Every command above:
all renders differ from the first,
worst difference 1 level (the figures in the symptom).

### Runs that work (the probe patch)

The probe patch applied to a clone of `linebender/vello` at `v0.11.0` (`origin` `https://github.com/linebender/vello.git`,
`HEAD` `f3000c8d9e7a9c7e6abb09b9587238bf8b4860ff`,
equal to the `vcs_info` of the registry copy),
and the program
rebuilt with `[patch.crates-io]` entries for `vello`,
`vello_encoding` and `vello_shaders` pointing at the clone's
`research/vello_research`,
`research/vello_encoding` and `research/vello_shaders`:

```text
adapter AMD Radeon RX 7600 (RADV NAVI33)
200 renders compared with the first (one Renderer): 0 differ, worst difference 0 level(s)
```

and the same 0 of 20 on llvmpipe and 0 of 50 on Raphael.
The first render of the stock build and of the patched build differ by at most one premultiplied level,
in 605 of
970224 texels (`DUMP=<path>` writes the first render as raw RGBA,
and a Node script comparing two dumps printed
`0: 969619` and `1: 605`),
so the probe draws the same picture to within the noise it removes.
I did not measure the probe's speed.
The comparison script:

```js
// compare-dumps.mjs
// Usage: node compare-dumps.mjs <a.rgba> <b.rgba>
// Premultiplied per-texel difference histogram of two raw straight-RGBA dumps of equal size.
import { readFileSync } from 'node:fs';
const [pathA, pathB] = process.argv.slice(2);
const a = readFileSync(pathA);
const b = readFileSync(pathB);
const premultiply = (value, alpha) => Math.trunc((value * alpha) / 255);
const histogram = new Map();
for (let index = 0; index < a.length; index += 4) {
  let delta = Math.abs(a[index + 3] - b[index + 3]);
  for (let channel = 0; channel < 3; channel++) {
    delta = Math.max(delta, Math.abs(premultiply(a[index + channel], a[index + 3]) - premultiply(b[index + channel], b[index + 3])));
  }
  histogram.set(delta, (histogram.get(delta) ?? 0) + 1);
}
console.log(`${a.length / 4} texels; difference in premultiplied levels -> texel count:`);
for (const [delta, count] of [...histogram].sort((x, y) => x[0] - y[0])) console.log(`  ${delta}: ${count}`);
```

Run as `DUMP=stock.rgba ./vello-noise <font> 5`,
the same with `DUMP=patched.rgba` for the patched build,
then `node compare-dumps.mjs stock.rgba patched.rgba`.

## Verified workarounds

### Compare renders with the measured noise, and only after a re-render

`package/ide/app/src/native/tile_reads.rs:22-60` defines `RENDER_NOISE = 1` and
`same_texel`/`same_rows`/`same_buffers`,
which compare premultiplied texels to within one level,
so the color of an
invisible texel does not count.
The stability tests use them wherever a frame follows a re-render
(`package/ide/app/src/native/annotation_stability_tests.rs`),
a positive control
(`the_noise_tolerance_tells_a_shifted_row_from_a_re_render`,
`annotation_stability_tests.rs:753`) shows that one level
passes and two levels or a shifted row do not,
and `repeated_renders_of_one_tile_spread` fails when the noise exceeds
`RENDER_NOISE`.
Tradeoff:
a real one-level regression in a texel cannot be told from noise,
and the constant is a measurement of
one host's GPUs and one scene family;
a new scene type needs a new measurement.

### Make the sum order-independent in a vendored shader

The probe patch is a workaround only in the sense that it shows the way:
it needs a vendored `vello_shaders`.
Tradeoff:
the repository would carry and rebuild a fork of an experimental renderer's shader,
its cost on large scenes
is unmeasured,
and the fixed-point scale (16 bits of fraction,
an `i32` sum) limits a tile to a few tens of
thousands of summed segments before the sum overflows.
It is not used in the IDE.

## What does not work

- Expecting `==` on two renders:
  false for every adapter tried,
  including llvmpipe.
- A new `Renderer` per render:
  still different (50 of 50).
- Another adapter:
  the two RADV GPUs and llvmpipe all differ.
- Reading the difference as a driver or hardware fault:
  the order comes from Vello's own atomic counter,
  and the probe
  removes the difference on all three adapters.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` entries (`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
`typescript-project-references.md`) do not name Vello or GPU rendering noise.

Duplicate search,
`gh search issues` and `gh search prs` on `linebender/vello`,
open and closed:
`deterministic`,
`nondeterministic`,
`non-deterministic`,
`bit-identical`,
`flaky`,
`atomicAdd order`,
`different output each render`,
`reproducible rendering`,
`snapshot test GPU`,
`float summation order`,
`fine.wgsl area`,
`segment order tile`,
`compute renderer research`,
`vello_research`,
`move vello into research`,
`compute renderer`,
`classic renderer`.
The one close hit is issue 1314 "[classic] Non-deterministic rendering bug when stroking rounded rectangles" (closed)
with pull request 1323 "Fix non-deterministic GPU stroke artifacts" (merged 2025-12-19):
a different defect,
in the
stroker's join logic,
with "severe frame-to-frame corruption" that the maintainers fixed as a bug.
Nothing reports or discusses the last-place noise in the area accumulation.
Other hits (`classic renderer` issues 2002 and 1975,
open) are unrelated gradient defects.

The six constraints:

1. **Is it really upstream's fault?**
   No.
   It is a consequence of the architecture (a parallel append whose order is set by atomics,
   and a floating-point sum
   in that order),
   not a malfunction,
   and Vello's own snapshot tests treat it as expected:
   `research/vello_research_tests/src/snapshot.rs` compares with a perceptual (FLIP) mean threshold,
   and its
   documentation of `assert_mean_less_than` says the value "could potentially be non-zero (i.e. there is a slight
   difference between the new and previous results) due to e.g. fast math on the GPU or other platform specific
   factors."
2. **Can upstream fix it?**
   Yes in principle:
   the probe makes 200 renders identical on three adapters.
   A real fix needs a speed and overflow analysis that I did not do.
3. **Are they supporting this use case?**
   No signal.
   No document promises repeatable pixels,
   and the compute renderer is described as research.
4. **Would the repo welcome our contribution?**
   No ban was found:
   the repository has no `CONTRIBUTING.md`,
   issue template,
   pull request template or code of
   conduct (`gh api repos/linebender/vello/community/profile`),
   and a search of its Markdown for AI or LLM policy text
   found none.
   Absence of a policy is not a fail.
5. **Will they likely fix it?**
   Upstream leans the other way,
   in its documents:
   the repository `README.md` calls the compute renderer "an
   experimental implementation for compute-capable GPUs" and says "Vello GPU is intended to become the primary renderer
   for production GPU use cases";
   issue 1871 (closed 2026-09-14) records the agreed direction "Move the current
   compute-centric renderer to `research/vello_research`";
   `v0.11.0` is released as "Vello Research v0.11.0".
   Fixes still land in the research tree (pull request 1992 on `tile_alloc`,
   merged 2026-10-15),
   but for defects,
   not
   for bit-exactness.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Only as a probe of the cause,
   not as a proposed change (no speed or overflow analysis).

Constraints 1,
3 and 5 fail,
so the auto-prototype step is not triggered and nothing is filed.
The probe was run to establish the cause for this document.
Draft:
none;
nothing to add to the upstream tracker,
which has no thread about this and would receive a request for a
guarantee that the project does not make.
If the repository ever depends on exact renders,
the evidence above (the cause,
the probe,
the three adapters) is the
starting point,
after a speed measurement.

## Evidence and limits

- Reproduced with a standalone program on three Vulkan adapters;
  I did not rerun the IDE's test
  `repeated_renders_of_one_tile_spread` or the `test:native` suite for this document (they build the whole IDE in a
  container),
  so the figures about that test are the study's.
- The probe was run on one scene family (text,
  a clip layer and a translucent fill);
  other scenes (dashed strokes,
  error waves) were not run through it.
- The probe changes only the area-coverage path (`AaConfig::Area`);
  MSAA paths were not touched or tested.
- The surrounding study is `doc/planning/ide-gpu-snapshot-harness.md`,
  section "Flakes after the merge".
