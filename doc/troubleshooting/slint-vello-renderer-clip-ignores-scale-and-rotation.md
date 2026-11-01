# Slint 1.18.1 experimental Vello renderer (anyrender) skips every item that lies outside the unscaled window under `transform-scale` or `transform-rotation`

Tool under test:
Slint 1.18.1 (`slint`,
`i-slint-core`,
`i-slint-renderer-anyrender` 1.18.1,
git commit
`372cf0ee5577c3dfec309a45e7b778ba4e81b734`) with the `renderer-vello` feature (Vello 0.10.0 and wgpu 29.0.4 inside Slint 1.18.1),
selected with `SLINT_BACKEND=winit-vello`,
on Fedora 44 with Mesa and an AMD RX 7600 (RADV).
The comparison renderer is `winit-femtovg` (OpenGL).
The study that found it is the section "Question 4:
the minimap on the GPU" of `doc/planning/ide-vello-text.md`;
the design that relies on Slint's Vello renderer is its "Design B".

## Symptom

An element drawn under `transform-scale` below 1,
or under `transform-rotation`,
loses every child item whose local
position lies outside the window's size in unscaled,
unrotated coordinates.
The test program has a 1600 px tall group of 40 text rows (and,
in a second scene,
40 rectangles),
drawn with `transform-scale: 0.2` in a 600 by 400 window.
On screen the whole group is 320 px tall.
Dark pixels per 40 px horizontal band of the screenshot,
from the top:

```text
text rows, FemtoVG:        290 290 290 290 290 290 290 290 0 0
text rows, Vello:          310 310   0   0   0   0   0   0 0 0
rectangle rows, FemtoVG:  1800 1800 1800 1800 1800 1800 1800 1800 0 0
rectangle rows, Vello:    1800 1800    0    0    0    0    0    0 0 0
```

FemtoVG draws all eight bands (320 px).
Vello draws two (80 px),
which is the window's 400 px height times the scale 0.2.
Rotation fails the same way.
A group at `y: 300px`,
rotated by `-90deg` about its top-left corner,
so that local y runs to the right,
shows 15 strips of 40 px in FemtoVG (rows 0 to 14,
the 600 px window) and 3 in Vello (rows 0 to 2,
local y below 100):

```text
rotated rows, FemtoVG:  3729 3729 3729 3729 3729 3729 3729 3729 3729 3729 3729 3729 3729 3729 3729
rotated rows, Vello:    1937 1937 1937    0    0    0    0    0    0    0    0    0    0    0    0
```

(dark pixels per 40 px vertical strip from the left;
the two renderers antialias differently,
so the counts per strip
differ,
and only where a strip is empty matters).
No error or log line is printed.

## Root cause

Slint's item renderer interface says the clip is kept in the current transformed coordinates:

```rust
// i-slint-core-1.18.1/item_rendering.rs:661-662
    /// Get the current clip bounding box in the current transformed coordinate.
    fn get_current_clip(&self) -> LogicalRect;
```

The core uses it to skip items that cannot be seen,
for every item type,
and the shaped-text drawing uses it to skip
whole lines:

```rust
// i-slint-core-1.18.1/item_rendering.rs:706-722
    fn filter_item(
        ...
        (
            self.get_current_clip().intersects(&bounding_rect),
            item_geometry.origin,
            Some(item_geometry.size),
        )
```

```rust
// i-slint-core-1.18.1/textlayout/sharedparley/draw.rs:78-86
/// The vertical extent the renderer clip lets anything be drawn in,
as a physical y range in the
/// current item's coordinates.
A conservative superset of what is visible -- see
/// [`crate::item_rendering::ItemRenderer::get_current_clip`] -- so it is a safe bound for
/// skipping draw work,
never for layout decisions.
pub(super) fn visible_band(item_renderer: &impl GlyphRenderer) -> Range<PhysicalLength> {
    let scale_factor = item_renderer.scale_factor();
    let clip = item_renderer.get_current_clip();
```

The anyrender item renderer moves its `clip_rect` in `translate` but not in `rotate` or `scale`:

```rust
// i-slint-renderer-anyrender-1.18.1/itemrenderer.rs:684-701
    fn translate(&mut self, distance: LogicalVector) {
        self.current_state.clip_rect = self.current_state.clip_rect.translate(-distance);
        ...
    }

    fn rotate(&mut self, angle_in_degrees: f32) {
        self.current_state.transform =
            self.current_state.transform.pre_rotate(angle_in_degrees.to_radians().into());
    }

    fn scale(&mut self, x_factor: f32, y_factor: f32) {
        self.current_state.transform =
            self.current_state.transform.pre_scale_non_uniform(x_factor as f64, y_factor as f64)
    }
```

So under `transform-scale: 0.2` the visible region of the group,
in the group's own coordinates,
is five times larger
than the window,
but `get_current_clip` still returns the window rectangle,
and `filter_item` drops every item beyond it.
The contract in `visible_band` ("a conservative superset of what is visible") is false after a downscale.
An upscale or a pure translation should not trigger it,
because an unscaled clip is then a superset
(by reasoning from the code;
I did not run an upscale scene).

FemtoVG keeps the same state and does follow the transform:

```rust
// i-slint-renderer-femtovg-1.18.1/itemrenderer.rs:784-817 (abridged)
    fn rotate(&mut self, angle_in_degrees: f32) {
        ...
        let clip = &mut self.state.last_mut().unwrap().scissor;
        // Compute the bounding box of the rotated rectangle
        let (sin, cos) = (-angle_in_radians).sin_cos();
        ...
    }

    fn scale(&mut self, x_factor: f32, y_factor: f32) {
        self.canvas.borrow_mut().scale(x_factor, y_factor);
        let clip = &mut self.state.last_mut().unwrap().scissor;
        clip.origin.x /= x_factor;
        clip.origin.y /= y_factor;
        clip.size.width /= x_factor;
        clip.size.height /= y_factor;
    }
```

Slint's maintainers fixed this exact defect twice already in other renderers,
which settles that it is a bug and not
a design choice:

- FemtoVG pull request 11105 "FemtoVG:
  Fix transform-scale breaking item clipping" (merged 2026-03-24):
  "the scissor (used for intersection clipping) needs to be adjusted not only in terms of size but also origin."
  and pull request 11147 "FemtoVG:
  Fix transform-rotation breaking item clipping" (merged 2026-03-26):
  "The scissor rect needs the inverse rotation,
  just like how translate uses -distance and scale uses 1 / factor."
  Both fix issue 10904 "Images can sometime vanish when a large transform scale is applied" (closed).
- Pull request 11441 "Fix partial renderer culling items under scale/rotate transforms" (merged 2026-04-20):
  "Items under `transform-scale < 1` with large local coordinates were incorrectly computed as outside the dirty region
  and culled".

The anyrender renderer arrived later (pull request 12576,
merged 2026-07-31;
the Vello renderer pull request 12991,
merged 2026-09-01) and its `rotate` and `scale` never got the fix.
On master (`277e7676ced5d51262a54f9e1342b40f40badf33`,
2026-10-20) `internal/renderers/anyrender/itemrenderer.rs`
still has the same two bodies (`fn rotate` at line 826 and `fn scale` at line 831).

An earlier reading in the study said the culling was in the text drawing alone;
the rectangle scene above shows that
`filter_item` culls every item type,
so the defect is not specific to `Text`.

## Verification

Program (Slint 1.18.1,
`renderer-femtovg` for the comparison and `renderer-vello`):

```toml
# vello-scale-clip/Cargo.toml
[package]
name = "vello-scale-clip"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
slint = { version = "=1.18.1", default-features = false, features = [
    "std",
    "backend-default",
    "renderer-femtovg",
    "renderer-vello",
    "compat-1-2",
] }
```

```rust
// vello-scale-clip/src/main.rs
slint::slint! {
    export component ScaleApp inherits Window {
        width: 600px;
        height: 400px;
        background: white;
        Rectangle {
            x: 0px;
            y: 0px;
            width: 600px;
            height: 1600px;
            transform-scale: 0.2;
            transform-origin: { x: 0px, y: 0px };
            for row in 40: Text {
                x: 0px;
                y: row * 40px;
                text: "XXXXXXXXXXXXXXXXXXXX";
                font-size: 30px;
                color: black;
            }
        }
    }

    export component RectApp inherits Window {
        width: 600px;
        height: 400px;
        background: white;
        Rectangle {
            x: 0px;
            y: 0px;
            width: 600px;
            height: 1600px;
            transform-scale: 0.2;
            transform-origin: { x: 0px, y: 0px };
            for row in 40: Rectangle {
                x: 0px;
                y: row * 40px;
                width: 300px;
                height: 30px;
                background: black;
            }
        }
    }

    export component RotateApp inherits Window {
        width: 600px;
        height: 400px;
        background: white;
        Rectangle {
            x: 0px;
            y: 300px;
            width: 600px;
            height: 1600px;
            transform-rotation: -90deg;
            transform-origin: { x: 0px, y: 0px };
            for row in 40: Text {
                x: 0px;
                y: row * 40px;
                text: "XXXXXXXXXXXXXXXXXXXX";
                font-size: 30px;
                color: black;
            }
        }
    }
}

fn main() {
    match std::env::var("SCENE").as_deref() {
        Ok("rotate") => RotateApp::new().unwrap().run().unwrap(),
        Ok("rect") => RectApp::new().unwrap().run().unwrap(),
        _ => ScaleApp::new().unwrap().run().unwrap(),
    }
}
```

Run each case in the repository's nested compositor (headless,
so no window appears on the desktop),
take one screenshot,
and count dark pixels per band with ImageMagick.
The first script hosts the program and writes the screenshot;
the second counts.
`SCENE` is `scale` (default),
`rect` or `rotate`;
`SLINT_BACKEND` is `winit-vello` or `winit-femtovg`.

```js
// shot.mjs
// Usage: node shot.mjs <out.png> <WxH> <command...>   (environment is inherited, e.g. SLINT_BACKEND=winit-vello)
// Hosts the command in the headless nested compositor, waits for the first frames, writes one PNG, and stops.
import { spawn } from 'node:child_process';
import { createConnection } from 'node:net';
import { mkdtempSync, existsSync, rmSync } from 'node:fs';
import { join } from 'node:path';

const [out, size, ...command] = process.argv.slice(2);
const directory = mkdtempSync(join(process.env.HOME + '/temp/agent', 'shot.'));
const socket = join(directory, 'nws.sock');
const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
const child = spawn(
  'monochromatic-nested-wayland-session',
  ['--headless', '--socket', socket, '--size', size, '--', ...command],
  { stdio: ['ignore', 'pipe', 'pipe'] },
);
let log = '';
child.stdout.on('data', chunk => (log += chunk));
child.stderr.on('data', chunk => (log += chunk));
const control = line =>
  new Promise((resolveReply, reject) => {
    const connection = createConnection(socket);
    let reply = '';
    connection.on('error', reject);
    connection.on('close', () => reject(new Error('closed without a reply to ' + line)));
    connection.on('data', chunk => {
      reply += chunk;
      if (reply.includes('\n')) {
        connection.end();
        resolveReply(reply.trim());
      }
    });
    connection.write(line + '\n');
  });
let status = 0;
try {
  const deadline = Date.now() + 60000;
  while (true) {
    if (existsSync(socket)) {
      const answer = await control('ping').catch(() => 'no');
      if (answer.startsWith('ok')) break;
    }
    if (Date.now() > deadline) throw new Error('compositor never answered');
    if (child.exitCode !== null) throw new Error('ended early: ' + log.slice(-1500));
    await wait(300);
  }
  await wait(4000);
  console.log('screenshot:', await control('screenshot ' + out));
  const lines = log.split('\n').filter(line => /panicked|thread '/i.test(line));
  console.log(lines.slice(-5).join('\n'));
} catch (error) {
  console.log('failed:', String(error));
  status = 1;
} finally {
  child.kill('SIGTERM');
  await wait(500);
  rmSync(directory, { recursive: true, force: true });
}
process.exit(status);
```

```js
// bands.mjs
// Usage: node bands.mjs <png> <bandSize> [columns]
// Prints the number of dark pixels (luma below half) in each horizontal band of the screenshot,
// or in each vertical strip when `columns` is given.
import { execFileSync } from 'node:child_process';
const [png, bandSize, mode] = process.argv.slice(2);
const height = Number(execFileSync('magick', ['identify', '-format', '%h', png], { encoding: 'utf8' }));
const width = Number(execFileSync('magick', ['identify', '-format', '%w', png], { encoding: 'utf8' }));
const band = Number(bandSize);
const columns = mode === 'columns';
const total = columns ? width : height;
const counts = [];
for (let start = 0; start < total; start += band) {
  const size = Math.min(band, total - start);
  const geometry = columns ? `${size}x${height}+${start}+0` : `${width}x${size}+0+${start}`;
  const mean = Number(
    execFileSync('magick', [png, '-crop', geometry, '+repage', '-colorspace', 'Gray', '-threshold', '50%', '-format', '%[fx:1-mean]', 'info:'], {
      encoding: 'utf8',
    }),
  );
  counts.push(Math.round(mean * size * (columns ? height : width)));
}
console.log(`${png}: ${width}x${height}, dark pixels per ${band}px ${columns ? 'strip from the left' : 'band from the top'}: ${counts.join(' ')}`);
```

```sh
# Run from the crate directory after `cargo build`, with the two scripts beside it.
SCENE=scale SLINT_BACKEND=winit-vello node shot.mjs "${HOME}/temp/agent/vello.png" 600x400 ./target/debug/vello-scale-clip
node bands.mjs "${HOME}/temp/agent/vello.png" 40
SCENE=rotate SLINT_BACKEND=winit-vello node shot.mjs "${HOME}/temp/agent/rotate.png" 600x400 ./target/debug/vello-scale-clip
node bands.mjs "${HOME}/temp/agent/rotate.png" 40 columns
```

### Runs that fail (Vello renderer, `SLINT_BACKEND=winit-vello`)

- `SCENE=scale`:
  bands 3 to 10 empty (the table in the symptom).
- `SCENE=rect`:
  the same cut.
- `SCENE=rotate`:
  strips 4 to 15 empty.

### Runs that work

- The same three scenes with `SLINT_BACKEND=winit-femtovg`:
  all eight bands,
  all eight bands,
  all fifteen strips.
- The same three scenes on Vello with the prototype patch (below):
  `310 310 310 310 310 310 310 310 0 0`,
  `1800 1800 1800 1800 1800 1800 1800 1800 0 0`,
  and 15 strips of `1937`.

## Verified workarounds

### Scale by font size and geometry instead of `transform-scale`

The study measured a minimap built from elements with a sixth of the font size,
with no transform,
and found it works
in both renderers (`doc/planning/ide-vello-text.md`,
"Question 4").
I did not re-run that variant for this document.
Tradeoff:
the application lays out the scaled geometry itself,
and it gives up the single scaled group.

### Patch the renderer crate at the consumer boundary

`[patch.crates-io]` entries for every `slint` and `i-slint-*` crate of the lock pointing at a clone with the patch
below build and run (verified:
the three scenes above).
Tradeoff:
every `i-slint-*` crate must be patched together,
the clone has to track the Slint version,
and the repository
would carry a vendored fork of an experimental renderer.

### Use FemtoVG for scaled content

FemtoVG over OpenGL (`SLINT_BACKEND=winit-femtovg`) draws all three scenes;
I did not run FemtoVG over wgpu on them.
Tradeoff:
it is not a Vello renderer,
so this gives up the point of using Slint's Vello renderer.

## What does not work

- The earlier reading that only `Text` is affected:
  the rectangle scene shows that every item type is culled.
- `cache-rendering-hint` on the scaled group:
  it does nothing under Vello in 1.18.1
  (`visit_layer` is not implemented;
  the study measured it),
  so it does not route around the culling.
- Not tried:
  making the window or a clipping parent large enough that the unscaled clip covers the group.
  It could work for a bounded group,
  but the window size then no longer matches what the application needs.

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
`typescript-project-references.md`) do not name Slint or its renderers for
this behavior.

Duplicate search,
`gh search issues` and `gh search prs` on `slint-ui/slint`,
open and closed:
`anyrender scale clip`,
`vello transform-scale`,
`vello renderer clip_rect`,
`get_current_clip`,
`transform-scale text clipped`,
`renderer-vello transform`,
`transform-rotation clip`,
`anyrender`,
`vello renderer missing items`,
`vello experimental`,
`item culled rotation`,
`filter_item clip scale`,
`transform-scale`,
`renderer-vello`.
No issue or pull request reports the anyrender defect.
The related threads are the ones cited under the root cause (10904,
11105,
11147,
11441,
all read),
plus
`transform-scale` hits 11250 "FemtoVG fonts looks bad when scaled" and 11519 "core:
fix
absolute_clip_rect_and_geometry for items under Transform",
which are about other code.
Nothing in the tracker says the anyrender renderer should keep an unscaled clip.

Candidate-fix gate:
the FemtoVG and partial-renderer fixes are absent from the anyrender source in 1.18.1 and on master
(file read at `277e7676ced5d51262a54f9e1342b40f40badf33`),
and the symptom and inputs match the earlier reports,
so those fixes are precedent for the change,
not a fix that already covers this renderer.

The six constraints:

1. **Is it really upstream's fault?**
   Yes.
   `ItemRenderer::get_current_clip` is specified in the transformed coordinates,
   the core and the text drawing rely on
   it,
   the sibling renderers implement it,
   and the anyrender renderer does not.
2. **Can upstream fix it?**
   Yes:
   two functions.
3. **Are they supporting this use case?**
   Yes:
   `transform-scale` and `transform-rotation` are documented properties,
   the renderer is under active work
   (the Vello renderer is "experimental" in the 1.18 changelog,
   and 1.19 moves it to wgpu 30),
   and the same defect was
   fixed for FemtoVG and the partial renderer within the last seven months.
4. **Would the repo welcome our contribution?**
   Yes.
   `CONTRIBUTING.md` welcomes issues and pull requests,
   points AI assistants to `AGENTS.md`,
   asks that pull requests
   be small and focused (this one is),
   and asks contributors to make sure they wrote the contribution themselves;
   issue templates exist (`1-bug-report.yaml`);
   no AI ban or disclosure rule was found.
5. **Will they likely fix it?**
   Likely:
   a maintainer fixed the FemtoVG twin in two small pull requests,
   merged three to four weeks after issue 10904
   was opened (2026-03-01),
   and renderer parity fixes are merged continuously.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Yes,
   below.

All six hold.
The draft is fileable as written,
and it needs a human author's description if the owner files it
(the contribution guide asks that the contributor wrote it);
nothing was filed.
My recommendation is to file it,
as a pull request with the patch or as an issue that links the patch,
because the evidence is a reproduction with a measured before and after,
a source trace,
and three precedent fixes.

### Prototype

Disposable clone:
`gh repo clone slint-ui/slint <dir> -- --depth 1 --branch v1.18.1`,
`origin`
`https://github.com/slint-ui/slint.git`,
`HEAD` `372cf0ee5577c3dfec309a45e7b778ba4e81b734`
(equal to the `vcs_info` of the registry copy).
The change is in `internal/renderers/anyrender/itemrenderer.rs`:
`rotate` and `scale` also update `clip_rect` the way FemtoVG's do (inverse rotation,
inverse factors,
bounding box),
through a small `bounding_rect` helper.
The diff is 51 lines in one file:
[`slint-vello-renderer-clip-ignores-scale-and-rotation.patch`](slint-vello-renderer-clip-ignores-scale-and-rotation.patch).
It applies to master at `277e7676ced5d51262a54f9e1342b40f40badf33` (`git apply --check`) and was formatted with the
tag's `rustfmt` (`rustfmt --edition 2024`).

Before and after are the Vello rows in the symptom and the "Runs that work" list:
`310 310 0 ...` becomes `310` in all eight bands,
`1800 1800 0 ...` becomes `1800` in all eight bands,
and 3 rotated strips become 15,
with the FemtoVG output as the reference.
The build used `[patch.crates-io]` entries for `i-slint-backend-linuxkms`,
`-selector`,
`-winit`,
`i-slint-common`,
`-compiler`,
`-core`,
`-core-macros`,
`i-slint-renderer-anyrender`,
`-femtovg`,
`-skia`,
`-software`,
`slint` and
`slint-macros`,
pointing at the clone.
Not done:
a negative scale factor (a mirror) was not tested;
the helper takes the bounding box of the two corners,
so it should be correct,
but no scene exercised it.
No Slint test suite was run,
and no screenshot test was added to the clone.

### Draft (fileable; needs a human-written description)

Title:
`Vello renderer: items under transform-scale or transform-rotation are culled by an untransformed clip`

Labels:
`bug`,
`vello` (suggested)

````md
## Description

With `SLINT_BACKEND=winit-vello` (1.18.1, and master at `277e767`), items under `transform-scale < 1` or
`transform-rotation` are culled when their local position is outside the window's size.

`AnyrenderItemRenderer::translate` updates `clip_rect`, but `rotate` and `scale` do not
(`internal/renderers/anyrender/itemrenderer.rs`, `fn rotate` and `fn scale`).
`ItemRenderer::get_current_clip` is documented as "the current clip bounding box in the current transformed
coordinate", and `filter_item` (`internal/core/item_rendering.rs`) and `visible_band`
(`internal/core/textlayout/sharedparley/draw.rs`) rely on it. FemtoVG fixed the same thing in #11105 and #11147,
and the partial renderer in #11441.

## Reproduction

A 600x400 window with a 600x1600 `Rectangle` under `transform-scale: 0.2` and `transform-origin: { x: 0px, y: 0px }`,
holding 40 `Text` rows (or 40 `Rectangle` rows) at `y: row * 40px`.
FemtoVG draws all rows (320 px on screen); Vello draws the rows whose local y is below 400 (80 px).
`transform-rotation: -90deg` on a group at `y: 300px` shows the same cut (3 of 15 visible rows).

## Suggested fix

Apply the FemtoVG logic in `rotate` (bounding box of the clip rotated by the inverse angle) and `scale` (clip divided
by the factors). Patch against `v1.18.1`, applies to master (attached when filing). With it Vello draws the same
rows as FemtoVG in all three scenes.
````

## Other findings made while reading the same code

These are capability gaps or fixed upstream,
so they have no filing draft.

### Wgpu texture images are not drawn by the Vello renderer in 1.18.1; fixed on master

`Image::try_from(wgpu::Texture)` images and `Window::set_rendering_notifier()` do not work with the Vello renderer in
1.18.1:

```rust
// i-slint-renderer-anyrender-1.18.1/itemrenderer.rs:1241-1245
        // Remaining variants hold live GPU resources (borrowed GL textures,
        // wgpu textures behind the unstable-wgpu-* features) that this
        // backend-agnostic renderer cannot import.
        #[allow(unreachable_patterns)]
        _ => None,
```

Pull request 13975 "anyrender:
Draw WGPU texture images in the vello renderer" (merged 2026-10-12) adds both:
"The vello renderer used to skip images created from a `wgpu::Texture` with `Image::try_from()`,
and
`Window::set_rendering_notifier()` failed with `Unsupported`.
It now supports both".
Pull request 13974 (merged 2026-10-10) moves the renderer to Vello 0.11 and wgpu 30,
selected with
`require_wgpu_30()`.
The 1.19.0 section of `CHANGELOG.md` on master (unreleased) lists "Vello:
Uses WGPU 30 and supports rendering
notifiers and WGPU texture images."
So the limit applies to 1.18.1 only;
no release carries the fix yet (the latest release is 1.18.1,
2026-09-21).

### Slint's text has no tab stops, no font feature control and no ligature switch

`ranged_builder` in `i-slint-core-1.18.1/textlayout/sharedparley/shaping.rs` (starting at line 65) sets the line height,
the overflow-wrap mode and the font request on the Parley builder,
and nothing else;
a search of the file for
`feature` and `tab` finds only an unrelated comment.
The study observed a tab drawing as a missing-glyph box and ligatures always on
(`doc/planning/ide-vello-text.md`,
"Question 2").
This is a capability gap of Slint's `Text`,
the same under FemtoVG and Vello;
the IDE keeps its code view in its own
Parley layout for this reason.
It is not a defect and was not searched in the tracker.

## Evidence and limits

- Sources read in the registry copies named above;
  the clone of `slint-ui/slint` at tag `v1.18.1` is also the base of
  the prototype;
  master was read at `277e7676ced5d51262a54f9e1342b40f40badf33`.
- The program,
  the screenshot script and the band counter are scratch files on the machine that found the defect
  (`~/temp/agent/gpu-findings/`),
  not in the repository;
  the program's source is in this document.
- One host,
  one Mesa,
  AMD only;
  the Vello renderer drew on the GPU the nested compositor left to Mesa's default.
- Slint's test suite and its screenshot tests were not run against the patch.
- The study's earlier note that "a one-line change" fixes it was imprecise:
  `rotate` needs the same treatment,
  and the change is a few lines.
