//! Frame encoding:
//!  flip the bottom-up readback and write it as PNG or BMP,
//!  plus the
//! parallel encoder worker pool the 60fps recorder feeds.
//!
//! Encoding is the expensive part of capture,
//!  so it runs OFF the render thread:
//!  the
//! recorder reads a frame back into a pooled buffer and hands it to this pool,
//!  whose
//! worker threads flip and encode in parallel and return the buffer for reuse.
//!  PNG uses
//! the fastest deflate settings;
//!  BMP is a near-free raw path for when PNG cannot keep up.

/// What:
///      Grouped `use` of the buffered writer,
///  file,
///  path,
///  atomics,
///  mpsc pool channel,
///           and threads.
/// Why:
///       Workers write files,
///  count errors,
///  and return buffers to the pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import fs from "node:fs"; import { Worker } from "node:worker_threads";
/// ```
use std::{
    io::BufWriter,
    fs::File,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread::JoinHandle,
};

/// What:
///      Grouped `use` of the PNG/BMP encoders,
///  the color type,
///  and the encode trait.
/// Why:
///       `write_png`/`write_bmp` call these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PngEncoder, BmpEncoder } from "image";
/// ```
use image::{
    codecs::{
        bmp::BmpEncoder,
        png::{CompressionType, FilterType, PngEncoder},
    },
    ColorType, ImageEncoder,
};

/// What:
///      `use anyhow::{Context, Result};`.
///  Error helpers.
/// Why:
///       The encode functions return `Result` and annotate failures.
use anyhow::{Context, Result};

/// What:
///      `use crossbeam_channel::Receiver as WorkReceiver;`.
///  The multi-consumer receive
///           half of the work queue.
///  Aliased so it reads distinctly from the mpsc pool
///           `Receiver`.
/// Why:
///       Several worker threads drain one frame queue;
///  std mpsc is single-consumer.
use crossbeam_channel::Receiver as WorkReceiver;

/// What:
///      `use tracing::warn;`.
///  Log macro.
/// Why:
///       Report a per-frame encode failure without aborting the pool.
use tracing::warn;

/// Output image format for captured frames.
///
/// What:
///      `pub enum Format { Png, Bmp }`.
///  Two variants.
///  PNG is compressed (small files,
///           more CPU);
///  BMP is raw (near-free encode,
///  large files).
/// Why:
///       Let the caller trade file size against encode cost for sustained 60fps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Format = "png" | "bmp";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Compressed PNG (fast deflate settings).
    Png,
    /// Uncompressed BMP (near-zero encode cost).
    Bmp,
}

/// Parsing and file-extension helpers for the output format.
///
/// What:
///      `impl Format { ... }`.
///  Methods to parse a name and name the file extension.
/// Why:
///       Keep format naming beside the enum.
impl Format {
    /// Parse a format name,
    ///  defaulting is handled by the caller.
    ///
    /// What:
    ///      `pub fn parse(name: &str) -> Result<Format, String>`.
    ///  Maps `"png"`/`"bmp"`
    ///           to a variant,
    ///  or returns an error string.
    /// Why:
    ///       The `record` control command accepts an optional format token.
    ///
    /// @example
    /// ```ts
    /// Format.parse("bmp"); // => Bmp
    /// ```
    pub fn parse(name: &str) -> Result<Format, String> {
        // What:     `match name { "png" => Ok(Png), "bmp" => Ok(Bmp), other => Err(...) }`.
        // Why:      Only two supported formats.
        match name {
            "png" => return Ok(Format::Png),
            "bmp" => return Ok(Format::Bmp),
            other => return Err(format!("unknown capture format: {other} (want png or bmp)")),
        }
    }

    /// File extension for this format.
    ///
    /// What:
    ///      `pub fn extension(self) -> &'static str`.
    ///  Program-lifetime string.
    /// Why:
    ///       Frame filenames use it.
    pub fn extension(self) -> &'static str {
        // What:     `match self { Png => "png", Bmp => "bmp" }`. Tail expression.
        // Why:      Name the file correctly.
        match self {
            Format::Png => return "png",
            Format::Bmp => return "bmp",
        }
    }
}

/// A captured frame handed to the encoder pool.
///
/// What:
///      `pub struct Frame { pub index: u64, pub width: u32, pub height: u32, pub
///           pixels: Vec<u8> }`.
///  `pixels` is the bottom-up RGBA readback;
///  the worker owns
///           it,
///  encodes it,
///  then returns it to the buffer pool.
/// Why:
///       Carries one frame's data plus its sequence number across the channel.
pub struct Frame {
    /// Monotonic frame index,
    ///  used in the filename.
    pub index: u64,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Bottom-up RGBA pixels (pooled buffer,
    ///  returned after encode).
    pub pixels: Vec<u8>,
}

/// A running pool of encoder worker threads plus the free-buffer pool.
///
/// What:
///      `pub struct EncoderPool { ... }`.
///  Owns the send half of the work queue,
///  the
///           receive/return halves of the free-buffer pool,
///  the failure counter,
///  and the
///           worker join handles.
/// Why:
///       The recorder pushes frames in,
///  pulls free buffers out,
///  and shuts the pool down.
pub struct EncoderPool {
    /// Send half of the work queue (render thread -> workers).
    work_tx: crossbeam_channel::Sender<Frame>,
    /// Receive half of the free-buffer pool (render thread pulls reusable buffers).
    free_rx: Receiver<Vec<u8>>,
    /// Send half of the free-buffer pool (seeded,
    ///  and workers return buffers here).
    free_tx: Sender<Vec<u8>>,
    /// Count of frames that failed to encode.
    failures: Arc<AtomicU64>,
    /// Worker thread join handles.
    handles: Vec<JoinHandle<()>>,
}

/// Lifecycle and buffer-flow methods for the encoder pool.
///
/// What:
///      `impl EncoderPool { ... }`.
///  Construct,
///  hand out / return buffers,
///  submit
///           frames,
///  and shut down.
/// Why:
///       The recorder drives the pool entirely through these methods.
impl EncoderPool {
    /// Spawn `workers` encoder threads writing `format` frames into `dir`.
    ///
    /// What:
    ///      `pub fn new(dir: PathBuf, format: Format, workers: usize, pool_buffers:
    ///           usize) -> EncoderPool`.
    ///  Creates the work queue and the pre-seeded free
    ///           buffer pool,
    ///  then spawns the worker threads.
    /// Why:
    ///       One call stands up the whole off-thread encode stage.
    ///
    /// @example
    /// ```ts
    /// const pool = EncoderPool.new("/tmp/frames", Format.Png, 6, 18);
    /// ```
    pub fn new(dir: PathBuf, format: Format, workers: usize, pool_buffers: usize) -> EncoderPool {
        // What:     `let (work_tx, work_rx) = crossbeam_channel::bounded::<Frame>(pool_buffers);`.
        //           A bounded MPMC queue; bounding it caps in-flight frames (and thus memory).
        // Why:      Backpressure: if encoders fall behind, the queue fills and the render
        //           thread drops rather than growing memory without bound.
        let (work_tx, work_rx) = crossbeam_channel::bounded::<Frame>(pool_buffers);

        // What:     `let (free_tx, free_rx) = std::sync::mpsc::channel::<Vec<u8>>();`. The
        //           free-buffer pool: multiple producers (workers return buffers), one
        //           consumer (the render thread takes free buffers).
        // Why:      Reuse buffers instead of allocating a 3.7MB frame every 16ms.
        let (free_tx, free_rx) = std::sync::mpsc::channel::<Vec<u8>>();

        // What:     `for _ in 0..pool_buffers { free_tx.send(Vec::new()).ok(); }`. Seed the
        //           pool with empty buffers (they grow on first fill and are reused after).
        // Why:      Give the render thread buffers to hand out from the first tick.
        for _ in 0..pool_buffers {
            free_tx.send(Vec::new()).ok();
        }

        // What:     `let failures = Arc::new(AtomicU64::new(0));`. Shared failure counter.
        // Why:      Report encode failures at stop; `Arc` shares it across worker threads.
        let failures = Arc::new(AtomicU64::new(0));

        // What:     `let mut handles = Vec::with_capacity(workers);`. Collect join handles.
        // Why:      Join them on shutdown.
        let mut handles = Vec::with_capacity(workers);

        // What:     `for id in 0..workers { ... handles.push(spawn); }`. Spawn each worker.
        // Why:      Parallelise encoding across cores.
        for id in 0..workers {
            // What:     Clone the per-worker channel ends and shared state.
            //           `work_rx.clone()` (crossbeam receiver is cloneable for MPMC),
            //           `free_tx.clone()` (mpsc sender is cloneable), `dir.clone()`,
            //           `failures.clone()` (Arc clone bumps the refcount).
            // Why:      Each thread owns its handles.
            let worker_rx = work_rx.clone();
            let worker_free_tx = free_tx.clone();
            let worker_dir = dir.clone();
            let worker_failures = Arc::clone(&failures);

            // What:     `let handle = std::thread::Builder::new().name(...).spawn(move || {
            //           worker_loop(...) }).expect(...);`. Spawn a named worker.
            // Why:      Run the drain-and-encode loop off the render thread.
            let handle = std::thread::Builder::new()
                .name(format!("nws-encoder-{id}"))
                .spawn(move || {
                    worker_loop(worker_rx, worker_free_tx, worker_dir, format, worker_failures);
                })
                .expect("spawning an encoder worker failed");

            handles.push(handle);
        }

        // What:     `EncoderPool { ... }`. Assemble the pool handle (tail expression). `dir`
        //           and `format` are captured by the workers, not stored, so they are not
        //           fields.
        // Why:      Return it to the recorder.
        return EncoderPool {
            work_tx,
            free_rx,
            free_tx,
            failures,
            handles,
        }
    }

    /// Take a free buffer from the pool,
    ///  if one is available.
    ///
    /// What:
    ///      `pub fn take_buffer(&self) -> Option<Vec<u8>>`.
    ///  `None` when every buffer is
    ///           in flight (encoders are behind).
    /// Why:
    ///       The render thread uses this to decide whether to capture or drop a frame.
    pub fn take_buffer(&self) -> Option<Vec<u8>> {
        // What:     `self.free_rx.try_recv().ok()`. Non-blocking take; `.ok()` turns the
        //           empty/disconnected error into `None`. Tail expression.
        // Why:      Never block the 60fps render thread waiting for a buffer.
        return self.free_rx.try_recv().ok()
    }

    /// Submit a filled frame to the encoder pool.
    ///
    /// What:
    ///      `pub fn submit(&self, frame: Frame) -> Result<(), Frame>`.
    ///  Non-blocking
    ///           send;
    ///  on a full queue it hands the frame back so the caller can recycle it.
    /// Why:
    ///       Preserve cadence:
    ///  a full queue means encoders are behind,
    ///  so drop this
    ///           frame rather than block.
    pub fn submit(&self, frame: Frame) -> Result<(), Frame> {
        // What:     `self.work_tx.try_send(frame).map_err(|err| err.into_inner())`. Try to
        //           enqueue; on failure recover the `Frame` from the send error.
        // Why:      Return the buffer to the caller for reuse when dropping.
        return self.work_tx.try_send(frame).map_err(|err| return err.into_inner())
    }

    /// Return a buffer to the free pool (used when a frame is dropped or reclaimed).
    ///
    /// What:
    ///      `pub fn return_buffer(&self, buffer: Vec<u8>)`.
    ///  Push a buffer back.
    /// Why:
    ///       Recycle a dropped frame's buffer.
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        // What:     `self.free_tx.send(buffer).ok();`. Return it; ignore a closed pool.
        // Why:      Keep the pool populated.
        self.free_tx.send(buffer).ok();
    }

    /// Stop the pool:
    ///  close the work queue,
    ///  join workers,
    ///  and report the failure count.
    ///
    /// What:
    ///      `pub fn shutdown(self) -> u64`.
    ///  Consumes the pool,
    ///  returns how many frames
    ///           failed to encode.
    /// Why:
    ///       Clean teardown when recording stops.
    pub fn shutdown(self) -> u64 {
        // What:     `drop(self.work_tx);`. Close the work queue's send half.
        // Why:      Once no senders remain and the queue drains, each worker's `recv`
        //           returns an error and the worker exits.
        drop(self.work_tx);

        // What:     `for handle in self.handles { let _ = handle.join(); }`. Wait for every
        //           worker to finish draining. `let _ =` ignores a worker panic result.
        // Why:      Ensure all queued frames are written before returning.
        for handle in self.handles {
            let _ = handle.join();
        }

        // What:     `self.failures.load(Ordering::Relaxed)`. Read the final failure count.
        // Why:      Report it to the caller.
        return self.failures.load(Ordering::Relaxed)
    }
}

/// One encoder worker:
///  drain frames,
///  encode each,
///  return its buffer to the pool.
///
/// What:
///      `fn worker_loop(work_rx: WorkReceiver<Frame>, free_tx: Sender<Vec<u8>>, dir:
///           PathBuf, format: Format, failures: Arc<AtomicU64>)`.
///  Loops until the work
///           queue closes.
/// Why:
///       The body of every encoder thread.
fn worker_loop(
    work_rx: WorkReceiver<Frame>,
    free_tx: Sender<Vec<u8>>,
    dir: PathBuf,
    format: Format,
    failures: Arc<AtomicU64>,
) {
    // What:     `let mut flipped: Vec<u8> = Vec::new();`. Per-worker scratch buffer for the
    //           top-down copy, reused across frames.
    // Why:      Avoid allocating a flip buffer every frame.
    let mut flipped: Vec<u8> = Vec::new();

    // What:     `while let Ok(frame) = work_rx.recv() { ... }`. Block for the next frame;
    //           `recv` errors (loop exits) once the queue is closed and drained.
    // Why:      Process frames as they arrive, then exit cleanly on shutdown.
    while let Ok(frame) = work_rx.recv() {
        // What:     `let path = dir.join(format!("frame-{:08}.{}", frame.index,
        //           format.extension()));`. Build the per-frame output path.
        // Why:      Zero-padded index keeps the sequence sorted.
        let path = dir.join(format!("frame-{:08}.{}", frame.index, format.extension()));

        // What:     `flip_rows(&frame.pixels, &mut flipped, frame.width, frame.height);`.
        //           Copy rows bottom-up into `flipped` top-down.
        // Why:      `glReadPixels` is bottom-up; images are top-down.
        flip_rows(&frame.pixels, &mut flipped, frame.width, frame.height);

        // What:     `if let Err(err) = write(&flipped, frame.width, frame.height, &path,
        //           format) { warn!(...); failures.fetch_add(1, Ordering::Relaxed); }`.
        //           Encode; on failure log and bump the counter.
        // Why:      One bad frame must not stop the recording.
        if let Err(err) = write(&flipped, frame.width, frame.height, &path, format) {
            warn!("encoding frame {} failed: {err:#}", frame.index);
            failures.fetch_add(1, Ordering::Relaxed);
        }

        // What:     `free_tx.send(frame.pixels).ok();`. Return the readback buffer to the
        //           pool for reuse.
        // Why:      Keep the buffer pool populated so the render thread never allocates.
        free_tx.send(frame.pixels).ok();
    }
}

/// Copy `src` (bottom-up RGBA) into `dst` (top-down),
///  reusing `dst`'s allocation.
///
/// What:
///      `fn flip_rows(src: &[u8], dst: &mut Vec<u8>, width: u32, height: u32)`.
///  Reverses
///           the row order with one linear pass over the rows.
/// Why:
///       Turn the GPU-native bottom-up readback into a top-down image buffer.
fn flip_rows(src: &[u8], dst: &mut Vec<u8>, width: u32, height: u32) {
    // What:     `let stride = width as usize * BYTES_PER_PIXEL;`. Bytes per row.
    // Why:      Row boundaries for the copy.
    let stride = width as usize * crate::screenshot::BYTES_PER_PIXEL;

    // What:     `dst.clear(); dst.reserve(src.len());`. Reset and ensure capacity.
    // Why:      Reuse the allocation across frames.
    dst.clear();
    dst.reserve(src.len());

    // What:     `for row in (0..height as usize).rev() { ... }`. Iterate rows from the last
    //           to the first (`.rev()` reverses the range).
    // Why:      Writing bottom rows first produces a top-down buffer.
    for row in (0..height as usize).rev() {
        // What:     `let start = row * stride;`. Byte offset of this row in `src`.
        // Why:      Slice the source row.
        let start = row * stride;

        // What:     `dst.extend_from_slice(&src[start..start + stride]);`. Append the row.
        // Why:      Build the flipped buffer row by row.
        dst.extend_from_slice(&src[start..start + stride]);
    }
}

/// Flip a bottom-up readback top-down,
///  then encode it to `path` in `format`.
///
/// What:
///      `pub fn write_flipped(raw: &[u8], width: u32, height: u32, path: &Path, format:
///           Format) -> Result<()>`.
///  `raw` is the bottom-up `glReadPixels` output;
///  this
///           allocates a scratch buffer,
///  flips into it,
///  and encodes.
/// Why:
///       Single screenshots (not on a hot path) share the recorder's flip+encode logic
///           through one call,
///  so both produce upright images.
///
/// @example
/// ```ts
/// writeFlipped(rawBottomUpPixels, 800, 600, "/tmp/a.png", Format.Png);
/// ```
pub fn write_flipped(raw: &[u8], width: u32, height: u32, path: &Path, format: Format) -> Result<()> {
    // What:     `let mut flipped: Vec<u8> = Vec::new();`. A one-off scratch buffer.
    // Why:      Hold the top-down copy for encoding.
    let mut flipped: Vec<u8> = Vec::new();

    // What:     `flip_rows(raw, &mut flipped, width, height);`. Reverse the row order.
    // Why:      `glReadPixels` is bottom-up; images are top-down.
    flip_rows(raw, &mut flipped, width, height);

    // What:     `write(&flipped, width, height, path, format)`. Encode the flipped pixels
    //           (tail expression).
    // Why:      Produce the upright file.
    return write(&flipped, width, height, path, format)
}

/// Encode top-down RGBA pixels to `path` in `format`.
///
/// What:
///      `fn write(pixels: &[u8], width: u32, height: u32, path: &Path, format: Format)
///           -> Result<()>`.
///  Dispatches to the PNG or BMP encoder.
/// Why:
///       One dispatch point shared by the pool and single screenshots.
fn write(pixels: &[u8], width: u32, height: u32, path: &Path, format: Format) -> Result<()> {
    // What:     `match format { Png => write_png(...), Bmp => write_bmp(...) }`. Route by
    //           format (tail expression).
    // Why:      Select the encoder.
    match format {
        Format::Png => return write_png(pixels, width, height, path),
        Format::Bmp => return write_bmp(pixels, width, height, path),
    }
}

/// Encode top-down RGBA pixels to a PNG file with the fastest deflate settings.
///
/// What:
///      `pub fn write_png(pixels: &[u8], width: u32, height: u32, path: &Path) ->
///           Result<()>`.
///  Uses `CompressionType::Fast` and `FilterType::NoFilter`.
/// Why:
///       Minimise per-frame CPU while still producing a compressed file;
///  also the
///           single-screenshot path.
///
/// @example
/// ```ts
/// writePng(pixels, 800, 600, "/tmp/a.png");
/// ```
pub fn write_png(pixels: &[u8], width: u32, height: u32, path: &Path) -> Result<()> {
    // What:     `let file = BufWriter::new(File::create(path).with_context(...)?);`. Create
    //           the file and wrap it in a buffered writer.
    // Why:      Buffered writes are faster than many small syscalls.
    let file = BufWriter::new(
        File::create(path).with_context(|| format!("creating {}", path.display()))?,
    );

    // What:     `let encoder = PngEncoder::new_with_quality(file, CompressionType::Fast,
    //           FilterType::NoFilter);`. Build a fast, filterless PNG encoder.
    // Why:      Fast deflate + no per-row filter is the cheapest PNG for a live capture.
    let encoder = PngEncoder::new_with_quality(file, CompressionType::Fast, FilterType::NoFilter);

    // What:     `encoder.write_image(pixels, width, height, ColorType::Rgba8.into())
    //           .context(...)?;`. Encode the RGBA pixels. `.into()` widens `ColorType` to
    //           the `ExtendedColorType` the trait wants.
    // Why:      Produce the PNG.
    encoder
        .write_image(pixels, width, height, ColorType::Rgba8.into())
        .with_context(|| format!("encoding PNG {}", path.display()))?;

    // What:     `Ok(())`. Success.
    // Why:      Frame written.
    return Ok(())
}

/// Encode top-down RGBA pixels to an uncompressed BMP file.
///
/// What:
///      `pub fn write_bmp(pixels: &[u8], width: u32, height: u32, path: &Path) ->
///           Result<()>`.
///  Raw pixels plus a header,
///  so encoding is near-free.
/// Why:
///       The reliable sustained-60fps path when PNG's deflate cannot keep up.
pub fn write_bmp(pixels: &[u8], width: u32, height: u32, path: &Path) -> Result<()> {
    // What:     `let mut file = BufWriter::new(File::create(path).with_context(...)?);`.
    //           `mut` because `BmpEncoder::new` borrows the writer mutably.
    // Why:      Create the destination.
    let mut file = BufWriter::new(
        File::create(path).with_context(|| format!("creating {}", path.display()))?,
    );

    // What:     `BmpEncoder::new(&mut file).encode(pixels, width, height,
    //           ColorType::Rgba8.into()).context(...)?;`. Write the BMP.
    // Why:      Emit the raw frame with a BMP header.
    BmpEncoder::new(&mut file)
        .encode(pixels, width, height, ColorType::Rgba8.into())
        .with_context(|| format!("encoding BMP {}", path.display()))?;

    // What:     `Ok(())`. Success.
    // Why:      Frame written.
    return Ok(())
}
