//! Screenshot capture:
//!  render one frame and read the framebuffer back as pixels.
//!
//! This runs on the main (GL-context) thread.
//!  It binds an offscreen texture the size of the
//! output mode,
//!  composites the current frame into it,
//!  and copies it to CPU memory with the
//! renderer's `ExportMem` primitive.
//!  Capturing off screen keeps every capture the size the
//! screen has now:
//!  the window's own back buffer only takes a new size after the next swap,
//! and nothing swaps while recording or while the parent window is hidden.
//!  `read_frame` fills a caller-owned buffer (so the 60fps recorder
//! can reuse buffers instead of allocating each frame);
//!  `capture` builds on it to write a
//! single PNG.
//!  PNG encoding of the raw pixels lives in the `encoder` module,
//!  off this
//! thread,
//!  so the recorder's per-tick cost stays minimal.

/// What:
///      `use std::path::Path;`.
///  Borrowed filesystem path.
/// Why:
///       `capture` writes to a caller-provided path.
use std::path::Path;

/// What:
///      Grouped `use` of the dmabuf `Fourcc` format tag,
///  the render-element,
///  renderer,
///           and texture types,
///  the `Bind`/`ExportMem`/`Offscreen`/`Texture` traits,
///           `render_output`,
///  and the `Rectangle`/`Size`/`Buffer` geometry.
/// Why:
///       Everything the readback references.
///  `Offscreen` adds `create_buffer`,
///  `Bind`
///           makes a texture the draw target,
///  `Texture` adds `size`,
///  and `ExportMem` adds
///           `copy_framebuffer` / `map_texture` to the renderer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Fourcc, WaylandSurfaceRenderElement, GlesRenderer, ExportMem, renderOutput, Rectangle } from "smithay";
/// ```
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            element::surface::WaylandSurfaceRenderElement,
            gles::{GlesRenderer, GlesTexture},
            Bind, ExportMem, Offscreen, Texture,
        },
    },
    desktop::space::render_output,
    utils::{Buffer, Rectangle, Size},
};

/// What:
///      `use anyhow::{Context, Result};`.
///  Error helpers.
/// Why:
///       The functions return `Result` and annotate each fallible step.
use anyhow::{Context, Result};

/// What:
///      `use crate::{encoder, render::CLEAR_COLOR, state::Compositor};`.
///  Reuse the
///           PNG encoder,
///  the shared clear colour,
///  and the state.
/// Why:
///       Screenshots composite with the same background as the live frame and encode
///           through the same code path as the recorder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as encoder from "./encoder";
/// import { CLEAR_COLOR } from "./render";
/// import { Compositor } from "./state";
/// ```
use crate::{encoder, render::CLEAR_COLOR, state::Compositor};

/// Number of bytes per pixel in the read-back `Abgr8888` framebuffer.
///
/// What:
///      `pub const BYTES_PER_PIXEL: usize = 4;`.
///  `usize` because it multiplies a
///           pixel count to a byte count.
/// Why:
///       Shared by the readback size check and the recorder's buffer sizing.
pub const BYTES_PER_PIXEL: usize = 4;

/// Render the current frame and copy the framebuffer into `buffer` as raw RGBA pixels.
///
/// What:
///      `pub fn read_frame(state: &mut Compositor, buffer: &mut Vec<u8>) ->
///           Result<(u32, u32)>`.
///  Mutably borrows the state and a caller-owned byte buffer
///           it fills;
///  returns the frame's `(width, height)`.
///  The pixels are bottom-up
///           (as `glReadPixels` returns them) and are NOT flipped here;
///  the encoder flips
///           when writing,
///  keeping this hot path to a bind,
///  a render,
///  and one copy.
/// Why:
///       The shared readback primitive for both single screenshots and the recorder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readFrame(state, buffer: Uint8Array): [number, number] { ... }
/// ```
///
/// @example
/// ```ts
/// const buf = [];
/// const [w, h] = readFrame(state, buf);
/// ```
pub fn read_frame(state: &mut Compositor, buffer: &mut Vec<u8>) -> Result<(u32, u32)> {
    // What:     `state.output.current_mode()` returns `Option<Mode>`; `.context(...)?` turns
    //           `None` into an error and unwraps `Some`. `.size` is the physical framebuffer size.
    // Why:      The output mode is what the screen is now, set as soon as a resize or scale
    //           change applies. The window's EGL back buffer keeps its old size until the next
    //           swap, which never comes while recording or while the parent window is hidden.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const size = state.output.currentMode()?.size ?? throwError("no mode");
    // ```
    let size = state
        .output
        .current_mode()
        .context("the nested output has no mode to capture")?
        .size;

    // What:     `let width = size.w as u32; let height = size.h as u32;`. Unsigned dims.
    // Why:      The image dimensions are reported as `u32`.
    let width = size.w as u32;
    let height = size.h as u32;

    // What:     A block scoping the render/readback borrows so they end before returning.
    // Why:      The renderer and the capture texture are borrowed mutably; release them after
    //           copying pixels out.
    {
        // What:     `let renderer = state.backend.renderer();`. A mutable borrow of the GLES
        //           renderer only; the output, space, and texture fields stay usable.
        // Why:      Capture renders off screen, without the window's back buffer.
        let renderer = state.backend.renderer();

        // What:     `Size<i32, Buffer>` is a size tagged as buffer pixels (siblings: `Physical`,
        //           `Logical`); `(w, h).into()` builds it from a tuple.
        // Why:      Offscreen buffers are sized in buffer pixels.
        let texture_size: Size<i32, Buffer> = (size.w, size.h).into();

        // What:     `match &state.capture_texture { Some(texture) => ..., None => false }` borrows
        //           the cached texture, if any, and compares its size with the frame size.
        // Why:      Reuse one texture across a recording instead of allocating one per frame.
        let reusable = match &state.capture_texture {
            Some(texture) => texture.size() == texture_size,
            None => false,
        };
        if !reusable {
            // What:     `Offscreen::<GlesTexture>::create_buffer(renderer, format, size)` allocates
            //           a GPU texture the renderer can draw into; `Some(...)` stores it.
            // Why:      A new frame size needs a texture of that size.
            let texture: GlesTexture = Offscreen::<GlesTexture>::create_buffer(renderer, Fourcc::Abgr8888, texture_size)
                .map_err(|err| anyhow::anyhow!("creating the capture texture failed: {err:?}"))?;
            state.capture_texture = Some(texture);
        }

        // What:     `.as_mut()` lends the stored texture mutably; `.context(...)?` covers the
        //           impossible `None` without a panic.
        // Why:      `bind` needs a mutable borrow of the texture it draws into.
        let texture = state
            .capture_texture
            .as_mut()
            .context("the capture texture is missing after creation")?;

        // What:     `let mut framebuffer = renderer.bind(texture).map_err(...)?;`. Binds the
        //           texture as the draw target; the returned target borrows the texture.
        // Why:      Need a target to draw and then read back.
        let mut framebuffer = renderer
            .bind(texture)
            .map_err(|err| anyhow::anyhow!("binding the capture texture failed: {err:?}"))?;

        // What:     `render_output::<_, WaylandSurfaceRenderElement<GlesRenderer>, _, _>(...)
        //           .map_err(...)?;`. Composite the current committed client content.
        // Why:      The readback should reflect the latest frame.
        render_output::<_, WaylandSurfaceRenderElement<GlesRenderer>, _, _>(
            &state.output,
            renderer,
            &mut framebuffer,
            1.0,
            0,
            [&state.space],
            &[],
            &mut state.damage_tracker,
            CLEAR_COLOR,
        )
        .map_err(|err| anyhow::anyhow!("rendering the frame for readback failed: {err:?}"))?;

        // What:     `let region: Rectangle<i32, Buffer> = Rectangle::from_size((size.w,
        //           size.h).into());`. The whole framebuffer in BUFFER coordinates.
        // Why:      `copy_framebuffer` reads a buffer-space region.
        let region: Rectangle<i32, Buffer> = Rectangle::from_size((size.w, size.h).into());

        // What:     `let mapping = renderer.copy_framebuffer(&framebuffer, region,
        //           Fourcc::Abgr8888).map_err(...)?;`. Copy into a CPU-readable mapping in
        //           `Abgr8888` (memory order R, G, B, A on little-endian, i.e. RGBA).
        // Why:      Move GPU pixels somewhere readable.
        let mapping = renderer
            .copy_framebuffer(&framebuffer, region, Fourcc::Abgr8888)
            .map_err(|err| anyhow::anyhow!("copy_framebuffer failed: {err:?}"))?;

        // What:     `let pixels = renderer.map_texture(&mapping).map_err(...)?;`. A
        //           read-only byte slice of the mapping (borrows the renderer).
        // Why:      Access the pixel bytes.
        let pixels = renderer
            .map_texture(&mapping)
            .map_err(|err| anyhow::anyhow!("map_texture failed: {err:?}"))?;

        // What:     `buffer.clear(); buffer.extend_from_slice(pixels);`. Empty the target
        //           buffer, then copy every pixel byte into it.
        // Why:      Hand the caller an owned copy so the renderer borrow can end.
        buffer.clear();
        buffer.extend_from_slice(pixels);
    }

    // What:     `let expected = width as usize * height as usize * BYTES_PER_PIXEL;`. The
    //           byte count a tightly packed RGBA frame should have.
    // Why:      Guard against an unexpected stride.
    let expected = width as usize * height as usize * BYTES_PER_PIXEL;

    // What:     `if buffer.len() != expected { return Err(...); }`. Reject a mismatch.
    // Why:      A stride mismatch would corrupt every downstream image.
    if buffer.len() != expected {
        return Err(anyhow::anyhow!(
            "readback size mismatch: got {} bytes, expected {expected}",
            buffer.len()
        ));
    }

    // What:     `Ok((width, height))`. Return the dimensions (tail expression).
    // Why:      The caller needs them to encode.
    return Ok((width, height))
}

/// Render the current frame and write it to `path` as a single PNG.
///
/// What:
///      `pub fn capture(state: &mut Compositor, path: &Path) -> Result<()>`.
///  Reads one
///           frame and encodes it synchronously (a single screenshot is not on a hot path).
/// Why:
///       The `screenshot` control command's implementation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function capture(state, path): void { ... }
/// ```
///
/// @example
/// ```ts
/// capture(state, "/tmp/frame.png");
/// ```
pub fn capture(state: &mut Compositor, path: &Path) -> Result<()> {
    // What:     `let mut pixels = Vec::new();`. A fresh buffer for the one frame.
    // Why:      A single screenshot need not reuse buffers.
    let mut pixels = Vec::new();

    // What:     `let (width, height) = read_frame(state, &mut pixels)?;`. Read the frame.
    // Why:      Fill `pixels` and learn the dimensions.
    let (width, height) = read_frame(state, &mut pixels)?;

    // What:     `encoder::write_flipped(&pixels, width, height, path, encoder::Format::Png)
    //           .with_context(...)?;`. `read_frame` returns bottom-up pixels, so use the
    //           flip+encode helper (the same flip the recorder's workers apply) to write an
    //           upright PNG.
    // Why:      Single screenshots must be upright, matching the recorded frames.
    encoder::write_flipped(&pixels, width, height, path, encoder::Format::Png)
        .with_context(|| format!("writing screenshot to {}", path.display()))?;

    // What:     `Ok(())`. Success.
    // Why:      Signal the screenshot was written.
    return Ok(())
}
