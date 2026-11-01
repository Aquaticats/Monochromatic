# Slint 1.17 MCP `take_screenshot` returns opaque black for a live winit FemtoVG OpenGL window

Tool under test:
Slint 1.17.0 with the embedded MCP server,
the winit backend,
and the FemtoVG OpenGL renderer.

Surface trigger:
a running application sets `SLINT_MCP_PORT` and leaves `SLINT_BACKEND` empty
so Slint creates its normal live winit window.

Failure mode:
`take_screenshot` returns a successful PNG whose pixels are all opaque black,
even though the live window is visibly rendered and MCP element inspection reports the populated UI.

## Symptom

A music-player instance ran within the repository's isolated
`monochromatic-nested-wayland-session` compositor at 1280 by 800.
The embedded server at `http://127.0.0.1:9316/mcp` returned a PNG content item,
not an MCP error,
 for `take_screenshot`.

ImageMagick measured that decoded PNG as follows:

```text
width=1280 height=800 min=0 max=0 mean=0 unique=1
```

The nested compositor's framebuffer screenshot of the same window was visibly populated
and measured 710 distinct colors.
MCP `get_element_tree` also reported the expected 1280 by 800 root and loaded music rows.

The same application and fixture under `SLINT_BACKEND=headless` produced a non-black
480 by 600 MCP screenshot.
That separates the failure from MCP HTTP transport,
base64 decoding,
and PNG encoding.

## Root cause

This is a FemtoVG OpenGL snapshot defect beneath the MCP transport.
MCP's image tool only obtains a window snapshot and encodes the returned pixel buffer:

```rust
// internal/backends/testing/introspection/mod.rs:249-279
let buffer =
    window.take_snapshot().map_err(|e| format!("Error grabbing window screenshot: {e}"))?;
image::write_buffer_with_format(
    &mut cursor,
    buffer.as_bytes(),
    buffer.width(),
    buffer.height(),
    image::ExtendedColorType::Rgba8,
    format,
)
```

`Window::take_snapshot()` delegates to its renderer:

```rust
// internal/core/api.rs:809-814
pub fn take_snapshot(&self) -> Result<SharedPixelBuffer<Rgba8Pixel>, PlatformError> {
    self.0.window_adapter().renderer().take_snapshot()
}
```

The project enables the live `backend-winit` and `renderer-femtovg` features in
`package/music-player/desktop-app/Cargo.toml:104`.
For FemtoVG,
Slint passes a render callback to the selected graphics backend:

```rust
// internal/renderers/femtovg/lib.rs:472-482
fn take_snapshot(&self) -> Result<SharedPixelBuffer<Rgba8Pixel>, PlatformError> {
    let size = self
        .maybe_window_adapter
        .borrow()
        .as_ref()
        .and_then(|w| w.upgrade())
        .map(|a| a.size())
        .unwrap_or_default();
    let canvas = self.canvas.borrow().as_ref().cloned();
    self.graphics_backend
        .take_snapshot_pixels(canvas, size.width, size.height, &|| self.render())
```

The 1.17 OpenGL implementation accepts that callback as `_render`,
does not invoke it,
and asks FemtoVG for a screenshot of its current framebuffer:

```rust
// internal/renderers/femtovg/opengl.rs:243-269, Slint v1.17.0
fn take_snapshot_pixels(
    &self,
    canvas: Option<CanvasRc<Self::Renderer>>,
    _width: u32,
    _height: u32,
    _render: &dyn Fn() -> Result<(), PlatformError>,
) -> Option<Result<SharedPixelBuffer<Rgba8Pixel>, PlatformError>> {
    let canvas = canvas?;
    Some((|| {
        self.opengl_context.borrow().ensure_current()?;
        let screenshot = canvas.borrow_mut().screenshot()?;
```

FemtoVG 0.25.1 implements that screenshot by calling `glReadPixels`:

```rust
// femtovg-0.25.1/src/renderer/opengl.rs:870-899
fn screenshot(&mut self) -> Result<ImgVec<RGBA8>, ErrorKind> {
    let w = self.view[0] as usize;
    let h = self.view[1] as usize;
    let mut image = ImgVec::new(/* RGBA output */, w, h);
    unsafe {
        self.context.read_pixels(
            0,
            0,
            self.view[0] as i32,
            self.view[1] as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(image.buf_mut().align_to_mut().1)),
        );
    }
```

The current back buffer can hold no usable presented frame when this read occurs.
On this host it yielded opaque black.
The same mechanism can instead expose stale pixels.
Slint issue [#9239][] records that one-frame-behind manifestation;
a maintainer attributes it to `glReadPixels` from FemtoVG's back buffer.

Open pull request [#13114][] is the matching candidate fix.
It replaces `_render` with `render`,
then invokes `render()?` before `canvas.screenshot()`.
The candidate is absent from the pinned v1.17.0 source:
its parameter remains `_render` and no invocation appears between `ensure_current()` and `screenshot()`.

## Verification

Version evidence:

- Slint source clone:
   `slint-ui/slint` tag `v1.17.0`,
   commit
  `fdde7a535305d2ab2d4072dee637bad186a49723`.
- Music player lockfile:
   FemtoVG 0.25.1,
   crates.io checksum
  `f43d05da42e81724c16d34a150fb7fda53d3f786e5a94673ee526ff8602f6a`.
- Host capture:
   the repository's `monochromatic-nested-wayland-session`
  with a 1280 by 800 private Wayland output.

The retained harness starts the isolated compositor through
`mise run //package/nested-wayland-session/cli:run`,
uses its socket `screenshot` request for the actual GPU framebuffer,
and drives the app's local MCP endpoint through raw JSON-RPC.
The debug music-player binary had first been built by
`mise run //package/music-player/desktop-app:mcp` with Slint's MCP feature.

Working catalog:

- `SLINT_BACKEND=headless` with the embedded MCP server returns a non-black real-data PNG.
- The nested compositor's `screenshot` control request returns a populated live-GPU PNG.
- `get_element_tree` returns the live window dimensions and loaded elements in the live-GPU session.

Failing catalog:

- A live winit FemtoVG OpenGL session with `SLINT_BACKEND=` returns a successful,
  all-black MCP `take_screenshot` PNG.
- The failure survives valid window discovery,
  valid element-tree inspection,
  and a populated nested-compositor framebuffer.

## Verified workarounds

Use `SLINT_BACKEND=headless` for MCP-owned visual evidence.
It uses Slint's software rasterizer in this package and returns real pixels.

Tradeoff:
it does not exercise the live GPU renderer or the compositor integration.

For live-GPU screenshots,
use the repository's `monochromatic-nested-wayland-session` socket command:

```text
screenshot /absolute/output.png
```

Tradeoff:
the compositor captures the full isolated output rather than an MCP window handle.
It cannot replace MCP's element-level inspection,
but it captures the pixels an end user receives and does not alter the host desktop.

## What does not work

Do not treat a successful MCP image content block as evidence that the live renderer's image is usable.
The all-black PNG had no JSON-RPC error.
Inspect decoded pixels or the rendered image itself.

Do not substitute the MCP image for the nested compositor's framebuffer when validating live winit rendering.
MCP's structural tools still work in that mode,
but its renderer snapshot is defective on the pinned FemtoVG OpenGL path.

Do not change the host desktop theme or capture the host display as a workaround.
The nested compositor supplies an isolated output and deterministic color scheme.

## Upstream filing decision

No new issue or comment will be posted.

The upstream tracker already contains the matching issue [#9239][].
The reporter later confirmed that Skia avoids the defect,
and maintainer `tronical` called the FemtoVG OpenGL behavior a valid bug.
The same maintainer said they did not expect to fix the OpenGL route directly,
preferring a future FemtoVG wgpu screenshot implementation.

Open pull request [#13114][] contains the exact missing-render candidate fix.
Its collaborator review asks for an offscreen follow-up because rendering during a screenshot also presents a frame.
The issue and pull request already contain a stronger renderer differential than this capture adds,
so a duplicate comment would add nothing.

The filing constraints resolve as follows:

1.  **Upstream fault:**
     yes.
     Slint's public `Window::take_snapshot()` reaches the defective FemtoVG OpenGL readback path.
2.  **Upstream fixability:**
     yes.
     Pull request [#13114][] demonstrates one minimal compatible change.
3.  **Supported use case:**
     yes.
     Slint documents MCP screenshots and exposes `Window::take_snapshot()` publicly.
4.  **Contribution welcome:**
     yes.
     `CONTRIBUTING.md` welcomes issues and pull requests,
     subject to its CLA.
5.  **Likely fix:**
     no. The maintainer's comment on [#9239][] explicitly leans away from fixing this OpenGL route,
    and [#13114][] remains open pending a semantics decision.
6.  **Local prototype:**
     not required.
     Constraint 5 fails,
     and upstream already has the exact one-file prototype in [#13114][].

No `.out-of-scope/` entry applies to Slint.
No separate issue draft or comment draft is retained because the existing thread has no missing actionable evidence.

[#9239]: https://github.com/slint-ui/slint/issues/9239
[#13114]: https://github.com/slint-ui/slint/pull/13114
