//! Off-thread thumbnail decoding with a bounded,
//!  evicting texture cache.
//!
//! A dedicated worker thread decodes image files with the `image` crate (never on the render
//! path:
//!  synchronous per-frame decode collapses the frame rate),
//!  sending back raw RGBA pixels
//! over an async channel drained on the GTK main context.
//!  The main thread wraps the pixels in a
//! `MemoryTexture`,
//!  caches it,
//!  and fulfils every `Picture` that requested that path.
//!  The cache is a
//! least-recently-used map bounded by total bytes,
//!  so browsing many images keeps decoded memory
//! bounded and an evicted preview simply re-decodes on scroll-back.

/// What:
///  imports the interior-mutability cells used for the main-thread cache and counters.
/// Why:
///  the cache,
///  pending map,
///  and running totals live behind `RefCell`/`Cell` inside an `Rc`.
use std::cell::{Cell, RefCell};
/// What:
///  imports the hash-map container.
/// Why:
///  the cache and the pending-requesters map are both keyed by path.
use std::collections::HashMap;
/// What:
///  imports the borrowed and owned path types.
/// Why:
///  requests borrow a path;
///  the channel and cache own `PathBuf` keys.
use std::path::{Path, PathBuf};
/// What:
///  imports the reference-counted pointer.
/// Why:
///  the shared state is held by the service handle and the main-context drain future.
use std::rc::Rc;

/// What:
///  imports the GTK widget-extension traits (`set_paintable`).
/// Why:
///  a decoded texture is set on the requesting `Picture` via a prelude trait.
use gtk4::prelude::*;
/// What:
///  imports the memory-format enum,
///  the CPU-memory texture,
///  and the base texture type.
/// Why:
///  decoded RGBA pixels become a `MemoryTexture` upcast to `Texture` for the cache.
use gtk4::gdk::{MemoryFormat, MemoryTexture, Texture};
/// What:
///  imports the glib byte-buffer and its module (`spawn_future_local`).
/// Why:
///  pixels are wrapped in `glib::Bytes` for the texture,
///  and the drain runs on the main context.
use gtk4::glib::{self, Bytes};
/// What:
///  imports the picture widget.
/// Why:
///  a preview pane shows its thumbnail in a `Picture`.
use gtk4::Picture;

/// What:
///  imports the thumbnail size and cache-byte-budget constants.
/// Why:
///  decode scales to `THUMB_SIZE`;
///  eviction keeps the cache under `THUMB_CACHE_BYTES`.
use crate::constants::{THUMB_CACHE_BYTES, THUMB_SIZE};

/// What:
///  bytes per RGBA pixel.
/// Why:
///  texture stride and cache byte-cost are pixel count times this.
const RGBA_CHANNELS: usize = 4;

/// What:
///  a decoded thumbnail's raw RGBA pixels and dimensions.
/// Why:
///  the `Send` payload crossing the worker-to-main channel;
///  not a GTK type so it moves threads.
struct Decoded {
    /// Row-major RGBA8 pixels,
    ///  `width * height * 4` bytes.
    pixels: Vec<u8>,
    /// Decoded width in pixels.
    width: i32,
    /// Decoded height in pixels.
    height: i32,
}

/// What:
///  one cached texture with its byte cost and last-use tick.
/// Why:
///  the byte cost drives the budget;
///  the tick drives least-recently-used eviction.
struct Cached {
    /// The GPU-uploadable texture shown in previews.
    texture: Texture,
    /// Byte cost counted against the cache budget.
    bytes: usize,
    /// Monotonic tick of this entry's most recent use.
    used: u64,
}

/// What:
///  shared main-thread thumbnail state:
///  the request sender,
///  the cache,
///  the pending-requesters
///       map,
///  the running byte total,
///  and the use counter.
/// Why:
///  one `Rc<ThumbInner>` is held by the service and by the main-context drain future.
struct ThumbInner {
    /// Sends a path to the decode worker;
    ///  unbounded,
    ///  so sending never blocks the UI thread.
    request_tx: async_channel::Sender<PathBuf>,
    /// Path -> cached texture.
    cache: RefCell<HashMap<PathBuf, Cached>>,
    /// Path -> pictures awaiting that path's decode (deduplicates concurrent requests).
    pending: RefCell<HashMap<PathBuf, Vec<Picture>>>,
    /// Sum of cached entries' byte costs.
    total_bytes: Cell<usize>,
    /// Monotonic use counter feeding `Cached::used`.
    tick: Cell<u64>,
}

/// What:
///  handle to the thumbnail service.
/// Why:
///  the strip holds one;
///  preview panes request thumbnails through it.
pub struct Thumbnails {
    /// The shared main-thread state.
    inner: Rc<ThumbInner>,
}

/// What:
///  constructor and request entry point for the thumbnail service.
/// Why:
///  the single surface preview panes use;
///  construction wires the worker and the drain.
impl Thumbnails {
    /// What:
    ///  start the decode worker thread and the main-context result drain,
    ///  returning a handle.
    /// Why:
    ///  named `start` (not `new`) because it spawns a thread;
    ///  called once on the main thread.
    pub fn start() -> Self {
        let (request_tx, request_rx) = async_channel::unbounded::<PathBuf>();
        let (result_tx, result_rx) = async_channel::unbounded::<(PathBuf, Option<Decoded>)>();
        std::thread::Builder::new()
            .name("thumbnail-decoder".to_owned())
            .spawn(move || worker_loop(&request_rx, &result_tx))
            .expect("spawn thumbnail worker thread");
        let inner = Rc::new(ThumbInner {
            request_tx,
            cache: RefCell::new(HashMap::new()),
            pending: RefCell::new(HashMap::new()),
            total_bytes: Cell::new(0),
            tick: Cell::new(0),
        });
        let drain = inner.clone();
        glib::spawn_future_local(async move {
            while let Ok((path, decoded)) = result_rx.recv().await {
                deliver(&drain, &path, decoded);
            }
        });
        return Self { inner }
    }

    /// What:
    ///  show `path`'s thumbnail in `picture`:
    ///  set it now on a cache hit,
    ///  else decode off-thread
    ///       and set it when ready,
    ///  deduplicating concurrent requests for the same path.
    /// Why:
    ///  only the first requester of an uncached path enqueues a decode;
    ///  the rest ride along.
    pub fn request(&self, path: &Path, picture: &Picture) {
        if let Some(texture) = self.touch(path) {
            picture.set_paintable(Some(&texture));
            return;
        }
        let mut pending = self.inner.pending.borrow_mut();
        let waiters = pending.entry(path.to_path_buf()).or_default();
        waiters.push(picture.clone());
        if waiters.len() == 1 {
            let _ = self.inner.request_tx.try_send(path.to_path_buf());
        }
    }

    /// What:
    ///  return a cache hit for `path`,
    ///  bumping its last-use tick.
    /// Why:
    ///  keeps the most-recently-shown thumbnails out of the eviction victim set.
    fn touch(&self, path: &Path) -> Option<Texture> {
        let mut cache = self.inner.cache.borrow_mut();
        let entry = cache.get_mut(path)?;
        let tick = self.inner.tick.get() + 1;
        self.inner.tick.set(tick);
        entry.used = tick;
        return Some(entry.texture.clone())
    }
}

/// What:
///  whether `path`'s extension names a decodable raster image.
/// Why:
///  preview panes request a thumbnail only for images;
///  other files show a typed icon instead.
pub fn is_image(path: &Path) -> bool {
    return matches!(
        path.extension()
            .and_then(|ext| return ext.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some(
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tiff" | "tif" | "avif" | "qoi"
        )
    )
}

/// What:
///  the decode worker loop:
///  block for a path,
///  decode it,
///  send the result back,
///  until the
///       request channel closes.
/// Why:
///  keeps every image decode off the render thread;
///  a decode failure sends `None` so the
///      requesters are cleared rather than left pending forever.
fn worker_loop(
    request_rx: &async_channel::Receiver<PathBuf>,
    result_tx: &async_channel::Sender<(PathBuf, Option<Decoded>)>,
) {
    while let Ok(path) = request_rx.recv_blocking() {
        let decoded = decode(&path);
        if result_tx.send_blocking((path, decoded)).is_err() {
            break;
        }
    }
}

/// What:
///  decode `path` into RGBA pixels scaled to fit `THUMB_SIZE`.
/// Why:
///  `thumbnail` fast-downscales preserving aspect,
///  so a huge image costs little memory;
///  returns
///      `None` on any decode error (unsupported format,
///  unreadable file).
fn decode(path: &Path) -> Option<Decoded> {
    let image = image::open(path).ok()?;
    let thumb = image.thumbnail(THUMB_SIZE, THUMB_SIZE).to_rgba8();
    let width = i32::try_from(thumb.width()).ok()?;
    let height = i32::try_from(thumb.height()).ok()?;
    return Some(Decoded {
        pixels: thumb.into_raw(),
        width,
        height,
    })
}

/// What:
///  on the main thread,
///  fulfil `path`'s requesters with the decoded pixels:
///  build a texture,
///       cache it (evicting as needed),
///  and set it on every waiting picture.
/// Why:
///  a failed decode still clears the pending requesters so they do not leak.
fn deliver(inner: &Rc<ThumbInner>, path: &Path, decoded: Option<Decoded>) {
    let waiters = inner.pending.borrow_mut().remove(path).unwrap_or_default();
    let Some(pixels) = decoded else {
        tracing::warn!(path = %path.display(), "thumbnail decode failed");
        return;
    };
    let texture = build_texture(&pixels);
    insert_evicting(inner, path, texture.clone(), byte_cost(&pixels));
    for picture in waiters {
        picture.set_paintable(Some(&texture));
    }
    tracing::debug!(
        path = %path.display(),
        width = pixels.width,
        height = pixels.height,
        cache_bytes = inner.total_bytes.get(),
        "thumbnail ready"
    );
}

/// What:
///  the cache byte cost of a decoded thumbnail.
/// Why:
///  RGBA is four bytes per pixel;
///  this is what the budget counts.
fn byte_cost(decoded: &Decoded) -> usize {
    return decoded.width as usize * decoded.height as usize * RGBA_CHANNELS
}

/// What:
///  wrap decoded RGBA pixels in a GPU-uploadable `Texture`.
/// Why:
///  `MemoryTexture` takes CPU memory with an explicit stride;
///  upcast so the cache is uniform.
fn build_texture(decoded: &Decoded) -> Texture {
    let stride = decoded.width as usize * RGBA_CHANNELS;
    let bytes = Bytes::from(&decoded.pixels);
    return MemoryTexture::new(
        decoded.width,
        decoded.height,
        MemoryFormat::R8g8b8a8,
        &bytes,
        stride,
    )
    .upcast::<Texture>()
}

/// What:
///  insert `texture` for `path`,
///  then evict least-recently-used entries until the cache is
///       under its byte budget.
/// Why:
///  keeps decoded-image memory bounded regardless of how many previews are opened.
fn insert_evicting(inner: &Rc<ThumbInner>, path: &Path, texture: Texture, bytes: usize) {
    let tick = inner.tick.get() + 1;
    inner.tick.set(tick);
    let replaced = inner.cache.borrow_mut().insert(
        path.to_path_buf(),
        Cached {
            texture,
            bytes,
            used: tick,
        },
    );
    let mut total = inner.total_bytes.get() + bytes;
    if let Some(old) = replaced {
        total -= old.bytes;
    }
    inner.total_bytes.set(total);
    evict_to_budget(inner);
}

/// What:
///  evict the least-recently-used cached thumbnail until the total is within budget.
/// Why:
///  one victim per pass keeps the accounting exact;
///  a bounded small cache makes the scan cheap.
fn evict_to_budget(inner: &Rc<ThumbInner>) {
    while inner.total_bytes.get() > THUMB_CACHE_BYTES {
        let victim = inner
            .cache
            .borrow()
            .iter()
            .min_by_key(|(_, cached)| return cached.used)
            .map(|(victim_path, _)| return victim_path.clone());
        let Some(victim_path) = victim else {
            break;
        };
        if let Some(removed) = inner.cache.borrow_mut().remove(&victim_path) {
            inner.total_bytes.set(inner.total_bytes.get() - removed.bytes);
        }
    }
}
