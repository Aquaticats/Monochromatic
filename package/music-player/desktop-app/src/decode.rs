//! Decoding:
//!  turn an audio file on disk into interleaved `f32` PCM samples.
//!
//! One demux path (symphonia probes the container and demuxes packets) feeds
//! two decode paths:
//!  symphonia's own decoders for FLAC/WAV/MP3/Vorbis/AAC/ALAC,
//! and the `opus` crate (libopus) for Opus,
//!  because the symphonia 0.6 meta-crate
//! exposes no Opus decoder (the `symphonia-codec-opus` crate exists but is not
//! wired into the `all` feature set).
//!  `open()` picks the path;
//!  both implement
//! `Source`.

/// What:
///      `use std::fs::File;` brings the file-handle type into scope.
///  `File` is an
///           owning handle to an open OS file;
///  dropping it closes the file.
/// Why:
///       We open the audio file and hand the handle to symphonia's stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { open as fsOpen } from "node:fs/promises";
/// ```
use std::fs::File;

/// What:
///      `use std::path::Path;` imports the borrowed filesystem-path type.
///  `Path` is an
///           unsized,
///  borrowed view of a path (sibling:
///  `PathBuf`,
///  the owned,
///  growable
///           version,
///  like `&str` vs `String`).
/// Why:
///       `open`/`decode_all` take `&Path` because they only read the path,
///  they do not
///           need to own it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::path::Path;

/// What:
///      `use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};`
///           imports decode machinery from the 0.6 `audio` codec sub-module:
///           `AudioDecoder` (the trait every audio decoder implements;
///  was the un-prefixed
///           `Decoder` in 0.5,
///  renamed because 0.6 also has video and subtitle decoder
///           traits),
///  `AudioDecoderOptions` (decoder knobs;
///  we use defaults,
///  which keep
///           gapless playback on;
///  was `DecoderOptions` in 0.5).
/// Why:
///       `SymphoniaSource` holds a `Box<dyn AudioDecoder>` and builds it with default
///           options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AudioDecoder, AudioDecoderOptions } from "symphonia/codecs/audio";
/// ```
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};

/// What:
///      `use symphonia::core::codecs::audio::well_known::CODEC_ID_OPUS;` imports the
///           Opus codec id constant (an `AudioCodecId`,
///  a newtype around `u32`,
///  value
///           0x1001).
///  In 0.5 this was the top-level `CODEC_TYPE_OPUS`;
///  0.6 moved well-known
///           codec ids into a `well_known` sub-module and made each codec family
///           (audio/video/subtitle) its own id type.
/// Why:
///       We compare the track's codec id against it to route Opus to libopus.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CODEC_ID_OPUS } from "symphonia/codecs/audio/wellKnown";
/// ```
use symphonia::core::codecs::audio::well_known::CODEC_ID_OPUS;

/// What:
///      `use symphonia::core::errors::Error;` imports symphonia's own error enum
///           (IoError,
///  DecodeError,
///  ResetRequired,
///  ...).
///  Same path and variants as 0.5.
/// Why:
///       We match its variants to skip a bad packet apart from real failures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SymphoniaError } from "symphonia/errors";
/// ```
use symphonia::core::errors::Error;

/// What:
///      `use symphonia::core::formats::probe::Hint;` imports a struct that gives the
///           prober a hint (like the file extension) to speed format detection.
///  In 0.5 this
///           lived at `symphonia::core::probe::Hint`;
///  0.6 moved the whole `probe` module
///           under `formats`.
/// Why:
///       We pass the file extension so probing is fast and reliable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Hint } from "symphonia/formats/probe";
/// ```
use symphonia::core::formats::probe::Hint;

/// What:
///      `use symphonia::core::formats::{...};` imports demuxer types:
///  `FormatOptions`
///           (demux knobs,
///  defaults),
///  `FormatReader` (the trait a demuxed container
///           implements:
///  lists tracks,
///  yields packets),
///  `SeekMode` (Accurate vs Coarse),
///           `SeekTo` (where to seek:
///  by time or by frame),
///  `Track` (one track's id + codec
///           params + timing),
///  `TrackType` (audio / video / subtitle,
///  used to ask for the
///           first audio track).
/// Why:
///       We probe into a `FormatReader`,
///  pick the first audio `Track`,
///  pull packets,
///  and
///           seek by absolute frame.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FormatOptions, FormatReader, SeekMode, SeekTo, Track, TrackType } from "symphonia/formats";
/// ```
use symphonia::core::formats::{
    FormatOptions, FormatReader, SeekMode, SeekTo, Track, TrackType,
};

/// What:
///      `use symphonia::core::io::MediaSourceStream;` imports the buffered stream
///           wrapper symphonia reads bytes from.
/// Why:
///       symphonia's probe takes a `MediaSourceStream`,
///  not a raw `File`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { MediaSourceStream } from "symphonia/io";
/// ```
use symphonia::core::io::MediaSourceStream;

/// What:
///      `use symphonia::core::meta::MetadataOptions;` imports the tag/meta reader knobs
///           (we pass defaults;
///  we ignore tags entirely).
/// Why:
///       The probe call requires a `MetadataOptions` argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { MetadataOptions } from "symphonia/meta";
/// ```
use symphonia::core::meta::MetadataOptions;

/// What:
///      `use symphonia::core::units::{Duration, Timestamp};` imports symphonia's 0.6
///           timeline new-types.
///  `Timestamp` wraps an `i64` count of timebase ticks (for
///           these audio formats one tick = one frame at the sample rate);
///  `Duration` wraps
///           a `u64` span of the same ticks.
///  In 0.5 `TimeStamp` was a bare `u64` alias;
///  0.6
///           made both real types that force checked/saturating arithmetic.
///  Sibling you
///           might expect for seeking:
///  `Time` (seconds),
///  which we deliberately do NOT use,
///           because `SeekTo::Time` maps second 0 to frame 0,
///  and Ogg/Opus streams start at
///           a non-zero frame (the encoder pre-skip),
///  so "seek to 0 seconds" gets rejected
///           as out-of-range;
///  we seek by absolute frame instead (see `seek_format`).
/// Why:
///       `SeekTo::Timestamp` needs a `Timestamp`;
///  `Timestamp::saturating_add` adds a
///           `Duration`,
///  which lets us offset the stream's start frame so "the beginning"
///           lands on the real first frame rather than the invalid frame 0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Duration, Timestamp } from "symphonia/units";
/// ```
use symphonia::core::units::{Duration, Timestamp};

/// What:
///      `use crate::error::PlayerError;` imports our one app-wide error type.
///  `crate::`
///           means "from the root of this crate" (sibling form:
///  `super::` = parent module,
///           `self::` = current module).
/// Why:
///       Every fallible function here returns `PlayerError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PlayerError } from "@/error";
/// ```
use crate::error::PlayerError;

/// What:
///      `use crate::opus::OpusSource;` imports the Opus-specific decoder source defined
///           in our sibling `opus.rs` module.
/// Why:
///       `open()` constructs an `OpusSource` when the track is Opus.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { OpusSource } from "./opus";
/// ```
use crate::opus::OpusSource;

// What:     `#[derive(Clone, Copy, Debug)]` auto-generates three traits: `Clone` (explicit
//           duplication), `Copy` (cheap implicit bitwise copy on assignment, since every
//           field is a plain number), `Debug` (`{:?}` printing).
// Why:      `Copy` lets us return an `AudioSpec` by value freely without ownership
//           ceremony; `Debug` helps tests print mismatches.
//
// In TS you'd write (pseudocode):
// ```ts
// // no decorator; just an interface below
// ```
#[derive(Clone, Copy, Debug)]
/// What:
///      `pub struct AudioSpec { ... }` declares a public record describing a decoded
///           stream's shape.
///  Fields:
///           - `rate: u32`.
///  Unsigned 32-bit integer of samples-per-second (e.g. 44100,
///             48000).
///  Siblings a reader might expect:
///  `u16` (too small for 96000/192000),
///             `u64`/`usize` (overkill),
///  `i32` (rate is never negative).
///           - `channels: u16`.
///  Channel count (1 = mono,
///  2 = stereo).
///  Siblings:
///  `u8` (would
///             fit,
///  but `u16` is the conventional audio-API width),
///  `usize` (that's for
///             memory indexing),
///  `u32` (more range than any real layout needs).
///           - `duration_secs: f64`.
///  Track length in seconds as a 64-bit float.
///  Sibling:
///             `f32` (too coarse for long tracks),
///  `u64` frames (the UI thinks in seconds),
///             `Duration` (we standardised on bare f64 seconds across the engine).
/// Why:
///       Callers (engine,
///  UI,
///  tests) need rate + channels to configure PipeWire and the
///           seek/position bar's total length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AudioSpec = { rate: number; channels: number; durationSecs: number };
/// ```
pub struct AudioSpec {
    /// What:
    ///      `pub rate: u32`.
    ///  Samples per second;
    ///  see the struct comment for the type
    ///           choice.
    /// Why:
    ///       PipeWire needs the native rate to set up the stream.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rate: number;
    /// ```
    pub rate: u32,
    /// What:
    ///      `pub channels: u16`.
    ///  Channel count;
    ///  see the struct comment for the type
    ///           choice.
    /// Why:
    ///       Interleaving and PipeWire layout both need the channel count.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// channels: number;
    /// ```
    pub channels: u16,
    /// What:
    ///      `pub duration_secs: f64`.
    ///  Total seconds;
    ///  see the struct comment for the type
    ///           choice.
    /// Why:
    ///       The seek bar's maximum and the "x:xx / y:yy" label use it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// durationSecs: number;
    /// ```
    pub duration_secs: f64,
}

/// What:
///      `pub trait Source: Send { ... }` declares an interface (`trait`) that any
///           decode source must implement.
///  `: Send` is a SUPERTRAIT bound meaning "values of
///           this type are safe to move to another thread".
///  Sibling bound:
///  `Sync` ("safe to
///           share by reference across threads") which we do NOT require,
///  because only one
///           thread (the engine) ever touches a `Source`.
/// Why:
///       The engine runs on its own thread and owns the active source,
///  so the source
///           must be `Send`.
///  The trait lets symphonia and Opus sources be used
///           interchangeably behind `Box<dyn Source>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Source {
///   spec(): AudioSpec;
///   nextChunk(): number[];   // empty array means EOF
///   seek(secs: number): void;
/// }
/// ```
pub trait Source: Send {
    /// What:
    ///      `fn spec(&self) -> AudioSpec;` a method signature (no body,
    ///  the implementor
    ///           provides it).
    ///  `&self` borrows the source read-only.
    /// Why:
    ///       Callers query rate/channels/duration without consuming it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spec(): AudioSpec;
    /// ```
    fn spec(&self) -> AudioSpec;

    /// What:
    ///      `fn next_chunk(&mut self) -> Result<Vec<f32>, PlayerError>;`.
    ///  `&mut self` =
    ///           exclusive borrow (decoding advances internal state).
    ///  `Vec<f32>` is an owned,
    ///           growable array of 32-bit floats (sibling:
    ///  `&[f32]` borrowed slice,
    ///  `[f32; N]`
    ///           fixed array);
    ///  we return owned so the caller can keep it past this call.
    ///  An
    ///           EMPTY `Vec` is the agreed signal for end-of-stream.
    /// Why:
    ///       Pull the next block of interleaved samples,
    ///  or learn we are done.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextChunk(): number[]; // [] means EOF
    /// ```
    fn next_chunk(&mut self) -> Result<Vec<f32>, PlayerError>;

    /// What:
    ///      `fn seek(&mut self, secs: f64) -> Result<(), PlayerError>;`.
    ///  `Result<(), E>`
    ///           means "succeeds with no value,
    ///  or fails with E";
    ///  `()` is the empty/unit type
    ///           (like `void`).
    /// Why:
    ///       Jump playback to a position in seconds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seek(secs: number): void;
    /// ```
    fn seek(&mut self, secs: f64) -> Result<(), PlayerError>;
}

/// What:
///      `pub fn open(path: &Path) -> Result<Box<dyn Source>, PlayerError>`.
///  `Box<dyn
///           Source>` is an owning pointer to a heap value whose concrete type is erased to
///           "something implementing `Source`" (dynamic dispatch).
///  Siblings:
///  `Rc<dyn Source>`
///           / `Arc<dyn Source>` are SHARED pointers;
///  we use `Box` because exactly one owner
///           (the engine) holds the source.
/// Why:
///       Probe the file,
///  find its audio track,
///  and return the right kind of decoder
///           without the caller caring which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function open(path: string): Source { ... }
/// ```
pub fn open(path: &Path) -> Result<Box<dyn Source>, PlayerError> {
    // What:     `File::open(path)?`. `File::open` returns `Result<File, io::Error>`; the `?`
    //           operator UNWRAPS the `Ok` value, or RETURNS the error early (converting it
    //           to `PlayerError` via our `From` impl).
    // Why:      Get an OS handle to the audio file, or bail with an i/o error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const file = await fsOpen(path);
    // ```
    let file = File::open(path)?;

    // What:     `MediaSourceStream::new(Box::new(file), Default::default())`. `Box::new(file)`
    //           heap-allocates the file and erases it to the `MediaSource` trait object the
    //           stream wants. `Default::default()` builds the default options struct (type
    //           inferred from the arg).
    // Why:      symphonia reads bytes through this buffered stream wrapper.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const mss = new MediaSourceStream(file, {});
    // ```
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // What:     `let mut hint = Hint::new();`. `Hint::new()` is a constructor returning an
    //           empty hint. `mut` marks the binding mutable so we can add the extension below.
    // Why:      A starting point to tell the prober the file's extension.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hint = new Hint();
    // ```
    let mut hint = Hint::new();

    // What:     `if let Some(ext) = path.extension().and_then(|e| e.to_str())`.
    //           `path.extension()` returns `Option<&OsStr>` (the extension, or `None`).
    //           `.and_then(|e| e.to_str())` runs the closure only if present; `e.to_str()`
    //           converts the OS string to `Option<&str>` (fails on non-UTF-8). `if let
    //           Some(ext) = ...` runs the body only when both succeeded, binding the inner
    //           `&str` to `ext`. `|e| e.to_str()` is a closure of one arg.
    // Why:      Feed the extension as a hint so probing is fast/reliable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ext = extname(path).replace(/^\./, "");
    // if (ext) hint.withExtension(ext);
    // ```
    if let Some(ext) = path.extension().and_then(|e| return e.to_str()) {
        // What:     `hint.with_extension(ext);` records the extension on the hint.
        // Why:      Gives the prober a strong signal of the container format.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // hint.withExtension(ext);
        // ```
        hint.with_extension(ext);
    }

    // What:     `symphonia::default::get_probe().probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())?`.
    //           `get_probe()` returns the global format prober; `.probe(...)` sniffs the
    //           stream and returns `Result<Box<dyn FormatReader>>` (0.5's `.format(...)`
    //           returned a `ProbeResult`; 0.6 hands the reader back directly). `&hint` lends
    //           the hint read-only; `mss` is MOVED in (the stream is consumed); the option
    //           structs are now passed BY VALUE (0.5 took `&` references). Trailing `?`
    //           unwraps or returns.
    // Why:      Detect the container and obtain a demuxer for it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const format = getProbe().probe(hint, mss, {}, {});
    // ```
    let format = symphonia::default::get_probe().probe(
        &hint,
        mss,
        FormatOptions::default(),
        MetadataOptions::default(),
    )?;

    // What:     a BLOCK expression `{ ... }` that produces a `(u32, bool, Track)` tuple and
    //           binds it to `(track_id, is_opus, track)`. The block scopes the immutable
    //           borrow of `format` (from `.first_track_known_codec()`) so it ends before we
    //           move `format` into the source below.
    // Why:      Read the track id, decide if it is Opus, and clone the track, then release
    //           the borrow so `format` is free to move.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [trackId, isOpus, track] = (() => {
    //   const t = format.firstTrackKnownCodec(TrackType.Audio);
    //   if (!t) throw new PlayerError.Unsupported("no audio track");
    //   const audio = t.codecParams?.audio();
    //   if (!audio) throw new PlayerError.Unsupported("track has no audio codec parameters");
    //   return [t.id, audio.codec === CODEC_ID_OPUS, structuredClone(t)];
    // })();
    // ```
    let (track_id, is_opus, track) = {
        // What:     `format.first_track_known_codec(TrackType::Audio)`. A 0.6 `FormatReader`
        //           helper: returns `Option<&Track>` for the first AUDIO track whose codec id
        //           is known (non-null). This replaces the 0.5 `tracks().iter().find(...)`.
        // Why:      Locate the first decodable audio track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const track = format.firstTrackKnownCodec(TrackType.Audio);
        // ```
        let track = format
            .first_track_known_codec(TrackType::Audio)
            // What:     `.ok_or_else(|| PlayerError::Unsupported(...))?`. Converts the
            //           `Option<&Track>` into a `Result`: `Some(x)` -> `Ok(x)`, `None` ->
            //           `Err(closure())`. `.to_string()` makes an OWNED `String` from the
            //           `&str` literal. Trailing `?` unwraps the `&Track` or returns the error.
            // Why:      A file with no audio track is unsupported; report it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (!track) throw new PlayerError.Unsupported("no audio track");
            // ```
            .ok_or_else(|| return PlayerError::Unsupported("no audio track".to_string()))?;

        // What:     `track.codec_params.as_ref().and_then(|cp| cp.audio())`. In 0.6
        //           `Track::codec_params` is `Option<CodecParameters>` (an enum over
        //           audio/video/subtitle); `.as_ref()` turns `&Option<_>` into
        //           `Option<&CodecParameters>`; `.and_then(|cp| cp.audio())` maps it to
        //           `Option<&AudioCodecParameters>` (the audio variant) via the closure.
        //           `.ok_or_else(...)?` unwraps or errors.
        // Why:      We need the audio codec id to decide whether this is Opus.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const audio = track.codecParams?.audio();
        // if (!audio) throw new PlayerError.Unsupported("track has no audio codec parameters");
        // ```
        let audio_params = track
            .codec_params
            .as_ref()
            .and_then(|cp| return cp.audio())
            .ok_or_else(|| {
                return PlayerError::Unsupported("track has no audio codec parameters".to_string())
            })?;

        // What:     `(track.id, audio_params.codec == CODEC_ID_OPUS, track.clone())`.
        //           `track.id` is a `u32` (Copy). `audio_params.codec == CODEC_ID_OPUS`
        //           compares the `AudioCodecId` new-types (both `Copy`, `Eq`).
        //           `track.clone()` DEEP-COPIES the `Track` (id + codec params + timing) so
        //           it outlives the `format` borrow. Tail expression -> block value.
        // Why:      We need an owned `Track` (carrying delay/num_frames/start_ts and the
        //           audio params) to hand the source after we move `format`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [track.id, audio.codec === CODEC_ID_OPUS, structuredClone(track)];
        // ```
        (track.id, audio_params.codec == CODEC_ID_OPUS, track.clone())
    };

    // What:     `if is_opus { ... } else { ... }`. Branch on the Opus flag computed above
    //           (the `track`/`format` borrow has ended, so `format` can move).
    // Why:      Opus needs the libopus path; everything else uses symphonia.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (isOpus) { ... } else { ... }
    // ```
    if is_opus {
        // What:     `OpusSource::new(format, track, track_id)?`. Calls the Opus source
        //           constructor, MOVING `format` and the owned `track` into it. `?` unwraps
        //           or returns the error.
        // Why:      Build the libopus-backed source.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const source = OpusSource.create(format, track, trackId);
        // ```
        let source = OpusSource::new(format, track, track_id)?;

        // What:     `Ok(Box::new(source))`. `Box::new(source)` heap-allocates and erases the
        //           concrete type to `dyn Source`; `Ok(...)` wraps it as the success value.
        //           Tail expression -> returned.
        // Why:      Hand back the boxed trait object on success.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return source;
        // ```
        return Ok(Box::new(source))
    } else {
        // What:     `SymphoniaSource::new(format, track, track_id)?`. Builds the
        //           symphonia-decoder source, moving `format`/`track` in.
        // Why:      Decode FLAC/WAV/MP3/Vorbis/AAC/ALAC with symphonia.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const source = SymphoniaSource.create(format, track, trackId);
        // ```
        let source = SymphoniaSource::new(format, track, track_id)?;

        // What:     `Ok(Box::new(source))`. Same boxing/wrapping as above.
        // Why:      Return the boxed trait object.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return source;
        // ```
        return Ok(Box::new(source))
    }
}

/// What:
///      `pub(crate) fn seek_format(format: &mut dyn FormatReader, track_id: u32, secs: f64) -> Result<(), PlayerError>`.
///           Reposition a demuxer to a wall-clock offset,
///  expressed as SECONDS FROM THE
///           AUDIBLE START,
///  by converting it to an ABSOLUTE frame timestamp the container
///           accepts.
///  `&mut dyn FormatReader` is a mutable borrow of any demuxer (we lend
///           it,
///  the caller keeps ownership;
///  sibling `Box<dyn FormatReader>` would take
///           ownership,
///  which we do not want here).
/// Why:
///       `SeekTo::Time { time: 0s }` maps to frame 0,
///  but Ogg/Opus streams begin at a
///           non-zero frame (the encoder pre-skip becomes the track's `start_ts`),
///  so seeking
///           to "0 seconds" was rejected with "requested seek timestamp is out-of-range for
///           stream" whenever the user dragged the bar to the very beginning.
///  Adding
///           `start_ts` makes second 0 land on the real first audible frame.
///  Shared by both
///           decode paths (`SymphoniaSource` here and `OpusSource` in `opus.rs`) so the fix
///           and the timeline math live in exactly one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function seekFormat(format, trackId, secs) {
///   const track = format.tracks().find((t) => t.id === trackId);
///   if (!track) throw new Error("seek: track not found");
///   const startTs = track.startTs;            // 0.6: timing lives on Track
///   const nFrames = track.numFrames;          // 0.6: was track.codecParams.nFrames
///   const sampleRate = track.codecParams?.audio()?.sampleRate;
///   if (sampleRate == null) throw new Error("seek: unknown sample rate");
///   const offset = Math.round(Math.max(0, secs) * sampleRate);
///   let ts = startTs + offset;
///   if (nFrames != null) ts = Math.min(ts, startTs + nFrames);
///   format.seek("accurate", { ts, trackId });
/// }
/// ```
pub(crate) fn seek_format(
    format: &mut dyn FormatReader,
    track_id: u32,
    secs: f64,
) -> Result<(), PlayerError> {
    // What:     `format.tracks().iter().find(|t| t.id == track_id)`. `tracks()` borrows the
    //           demuxer's track list (`&[Track]`, a borrowed slice; siblings
    //           `Vec<Track>`/`[Track; N]` would be owned/fixed-size); `.iter()` makes a
    //           read-only cursor over it; `.find(|t| ...)` returns the first track whose `id`
    //           matches, as `Option<&Track>`. The closure `|t| t.id == track_id` takes each
    //           track by reference `t` and compares its `id` field. `.ok_or_else(|| ...)`
    //           turns `None` into our error. `?` unwraps the `Some` or returns the error.
    // Why:      We need this track's timeline parameters; a missing id means the caller
    //           passed a stale track, which is a real failure.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const track = format.tracks().find((t) => t.id === trackId);
    // if (!track) throw new Error("seek: track not found");
    // ```
    let track = format
        .tracks()
        .iter()
        .find(|t| return t.id == track_id)
        .ok_or_else(|| return PlayerError::Unsupported("seek: track not found".to_string()))?;

    // What:     `let start_ts: Timestamp = track.start_ts;`. Copy the track's first-frame
    //           timestamp out of the borrowed track. In 0.6 timing lives directly on `Track`
    //           (was `codec_params.start_ts` in 0.5). `Timestamp` is a `Copy` new-type around
    //           `i64` (sibling `Time` would be seconds, which we are avoiding).
    // Why:      For Ogg/Opus this is the pre-skip frame; adding it shifts our "seconds from
    //           audible start" onto the container's absolute timeline.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const startTs = track.startTs;
    // ```
    let start_ts: Timestamp = track.start_ts;

    // What:     `let rate = track.codec_params.as_ref().and_then(|cp| cp.audio()).and_then(|a| a.sample_rate).ok_or_else(|| ...)?;`.
    //           Dig the sample rate out of the optional audio codec params: `.as_ref()`
    //           borrows the `Option<CodecParameters>`, `.and_then(|cp| cp.audio())` selects
    //           the audio variant, `.and_then(|a| a.sample_rate)` reads its `Option<u32>`
    //           rate. `.ok_or_else(|| ...)` turns any `None` along the way into our error;
    //           `?` unwraps.
    // Why:      We convert seconds to frames with the rate; without it the seek target is
    //           undefined, so failing loudly beats guessing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rate = track.codecParams?.audio()?.sampleRate;
    // if (rate == null) throw new Error("seek: unknown sample rate");
    // ```
    let rate = track
        .codec_params
        .as_ref()
        .and_then(|cp| return cp.audio())
        .and_then(|a| return a.sample_rate)
        .ok_or_else(|| return PlayerError::Unsupported("seek: unknown sample rate".to_string()))?;

    // What:     `let n_frames = track.num_frames;`. Copy the optional total audible frame
    //           count (`Option<u64>`; `Some(n)` if the container knew the length, `None`
    //           otherwise). In 0.6 this is `Track::num_frames` (was `codec_params.n_frames`
    //           in 0.5).
    // Why:      Used below to clamp the seek so we never ask for a frame past the end. This
    //           is the last read of `track`, so the borrow of `format` ends here and the
    //           mutable `format.seek(...)` below is allowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const nFrames = track.numFrames;
    // ```
    let n_frames = track.num_frames;

    // What:     `let secs_clamped = if secs > 0.0 { secs } else { 0.0 };`. An `if/else`
    //           EXPRESSION flooring the requested offset at zero.
    // Why:      The slider's minimum is 0, but a stray negative would make the frame-count
    //           cast below saturate oddly; clamping keeps it well-defined.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const secsClamped = Math.max(0, secs);
    // ```
    let secs_clamped = if secs > 0.0 { secs } else { 0.0 };

    // What:     `let offset_frames = (secs_clamped * f64::from(rate)).round() as u64;`.
    //           `f64::from(rate)` widens the `u32` rate to `f64` losslessly; the multiply
    //           gives a fractional frame count; `.round()` picks the nearest whole frame;
    //           `as u64` truncates the now-integral float to a frame count (a float-to-int
    //           `as` cast saturates at 0 / `u64::MAX` instead of wrapping).
    // Why:      Convert "seconds from start" into "frames from start".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const offsetFrames = Math.round(secsClamped * rate);
    // ```
    let offset_frames = (secs_clamped * f64::from(rate)).round() as u64;

    // What:     `let mut target_ts: Timestamp = start_ts.saturating_add(Duration::new(offset_frames));`.
    //           `mut` because we may clamp it below. `Duration::new(offset_frames)` wraps the
    //           `u64` frame offset as a `Duration` (0.6 forces typed timeline arithmetic).
    //           `Timestamp::saturating_add(Duration)` adds without overflowing (caps instead
    //           of wrapping; sibling `checked_add` would return `Option` instead).
    // Why:      The absolute frame to seek to is the stream's start plus our offset.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let targetTs = startTs + offsetFrames;
    // ```
    let mut target_ts: Timestamp = start_ts.saturating_add(Duration::new(offset_frames));

    // What:     `if let Some(total_frames) = n_frames { ... }`. Run the block only when the
    //           total length is known, binding the inner `u64` to `total_frames`.
    // Why:      Only clamp when we actually know where the end is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (nFrames != null) { ... }
    // ```
    if let Some(total_frames) = n_frames {
        // What:     `let max_ts = start_ts.saturating_add(Duration::new(total_frames));`. The
        //           last valid absolute frame is the start plus the audible length;
        //           `Duration::new(total_frames)` wraps the `u64` count as a `Duration`.
        // Why:      Compute the upper bound the demuxer will accept.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const maxTs = startTs + nFrames;
        // ```
        let max_ts = start_ts.saturating_add(Duration::new(total_frames));

        // What:     `if target_ts > max_ts { target_ts = max_ts; }`. `Timestamp` derives
        //           `Ord`, so `>` compares the wrapped `i64`s directly. Pull the target back
        //           to the end if rounding pushed it past the final frame.
        // Why:      Seeking one frame past the end would itself be out-of-range; the slider's
        //           maximum equals the duration, so this guards that edge.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (targetTs > maxTs) targetTs = maxTs;
        // ```
        if target_ts > max_ts {
            target_ts = max_ts;
        }
    }

    // What:     `format.seek(SeekMode::Accurate, SeekTo::Timestamp { ts: target_ts, track_id })?`.
    //           `SeekMode::Accurate` lands exactly (sibling `Coarse` is fast but
    //           approximate). `SeekTo::Timestamp { ... }` seeks by an absolute timestamp
    //           (sibling `SeekTo::Time` seeks by seconds, which is what caused the
    //           out-of-range bug). `seek` returns `Result<SeekedTo>`; we discard the
    //           `SeekedTo` (the `;`) and let `?` convert/propagate any symphonia error.
    // Why:      Perform the actual reposition at the corrected absolute frame.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // format.seek("accurate", { ts: targetTs, trackId });
    // ```
    format.seek(
        SeekMode::Accurate,
        SeekTo::Timestamp {
            ts: target_ts,
            track_id,
        },
    )?;

    // What:     `Ok(())`. Wrap the unit value `()` as success. Tail -> return.
    // Why:      Seek succeeded; there is no value to hand back.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return;
    // ```
    return Ok(())
}

/// What:
///      `struct SymphoniaSource { ... }`.
///  A record holding the live decode state for a
///           non-Opus track.
/// Why:
///       Bundles the demuxer + decoder + reusable buffer so `next_chunk`/`seek` can
///           advance them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SymphoniaSource implements Source { format; decoder; trackId; spec; pending; nFrames; }
/// ```
struct SymphoniaSource {
    /// What:
    ///      `format: Box<dyn FormatReader>`.
    ///  An owning,
    ///  heap,
    ///  type-erased demuxer.
    ///           (Sibling pointers `Rc`/`Arc` would be shared;
    ///  this is single-owner.)
    /// Why:
    ///       We pull packets from it each `next_chunk`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// format: FormatReader;
    /// ```
    format: Box<dyn FormatReader>,
    /// What:
    ///      `decoder: Box<dyn AudioDecoder>`.
    ///  Owning,
    ///  heap,
    ///  type-erased audio decoder
    ///           (0.6 renamed the 0.5 `Decoder` trait to `AudioDecoder`).
    /// Why:
    ///       Turns packets into PCM audio buffers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// decoder: AudioDecoder;
    /// ```
    decoder: Box<dyn AudioDecoder>,
    /// What:
    ///      `track_id: u32`.
    ///  The id of the track we decode (a packet stream may
    ///           interleave several tracks).
    ///  `u32` because symphonia ids are `u32` (sibling
    ///           `usize` would force casts against the API).
    /// Why:
    ///       Skip packets that belong to other tracks.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trackId: number;
    /// ```
    track_id: u32,
    /// What:
    ///      `spec: AudioSpec`.
    ///  The cached rate/channels/duration.
    ///  NOTE:
    ///  for some codecs
    ///           (AAC/ALAC in MP4) the channel count is unknown until the first packet is
    ///           decoded,
    ///  so `new` refreshes this after priming.
    /// Why:
    ///       `spec()` returns it without recomputing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spec: AudioSpec;
    /// ```
    spec: AudioSpec,
    /// What:
    ///      `pending: Option<Vec<f32>>`.
    ///  The first decoded chunk,
    ///  buffered by `new` while
    ///           priming.
    ///  `Some(chunk)` until the first `next_chunk` consumes it,
    ///  then `None`.
    /// Why:
    ///       Priming decodes one packet early (to learn the real spec) and must not lose
    ///           that first audio block.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pending: number[] | null;
    /// ```
    pending: Option<Vec<f32>>,
    /// What:
    ///      `n_frames: Option<u64>`.
    ///  Total decoded frames the container reported,
    ///  if
    ///           known (`u64` because frame counts of long tracks exceed `u32`;
    ///  sibling
    ///           `usize` would vary by platform width).
    /// Why:
    ///       Duration is recomputed (`n_frames / rate`) after priming reveals the true
    ///           rate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nFrames: number | null;
    /// ```
    n_frames: Option<u64>,
}

/// What:
///      `impl SymphoniaSource { ... }`.
///  An inherent-method block (methods tied to the
///           type itself,
///  not to a trait).
/// Why:
///       Holds the `new` constructor and the `decode_next_raw` helper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SymphoniaSource { static create(...) {} private decodeNextRaw() {} }
/// ```
impl SymphoniaSource {
    /// What:
    ///      `fn new(format: Box<dyn FormatReader>, track: Track, track_id: u32) -> Result<Self, PlayerError>`.
    ///           `Self` is the type being impl'd (`SymphoniaSource`).
    ///  Takes ownership of
    ///           `format` and the owned `track` (0.6 moved timing onto `Track`,
    ///  so we keep the
    ///           whole track instead of just the codec params).
    /// Why:
    ///       Build a decoder from the track's audio params and cache the spec.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static create(format, track, trackId): SymphoniaSource { ... }
    /// ```
    fn new(
        format: Box<dyn FormatReader>,
        track: Track,
        track_id: u32,
    ) -> Result<Self, PlayerError> {
        // What:     `let audio_params = track.codec_params.as_ref().and_then(|cp| cp.audio()).ok_or_else(|| ...)?;`.
        //           Dig the audio codec params out of the optional `CodecParameters` enum:
        //           `.as_ref()` borrows the `Option`, `.and_then(|cp| cp.audio())` selects the
        //           audio variant. `.ok_or_else(...)` turns `None` into our error; `?` unwraps
        //           to a `&AudioCodecParameters`.
        // Why:      `make_audio_decoder` and the initial spec both read from it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const audio = track.codecParams?.audio();
        // if (!audio) throw new PlayerError.Unsupported("track has no audio codec parameters");
        // ```
        let audio_params = track
            .codec_params
            .as_ref()
            .and_then(|cp| return cp.audio())
            .ok_or_else(|| {
                return PlayerError::Unsupported("track has no audio codec parameters".to_string())
            })?;

        // What:     `symphonia::default::get_codecs().make_audio_decoder(audio_params, &AudioDecoderOptions::default())?`.
        //           `get_codecs()` returns the global codec registry; `.make_audio_decoder`
        //           builds a `Box<dyn AudioDecoder>` for these audio params (0.5 had a single
        //           `.make(...)`; 0.6 split it per codec family). `audio_params` is already a
        //           borrow; `&AudioDecoderOptions::default()` lends default options. `?`
        //           unwraps.
        // Why:      Obtain a concrete decoder for this codec.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const decoder = getCodecs().makeAudioDecoder(audio, {});
        // ```
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(audio_params, &AudioDecoderOptions::default())?;

        // What:     `let rate = audio_params.sample_rate.unwrap_or(0);`. `sample_rate` is
        //           `Option<u32>`; `.unwrap_or(0)` yields the inner value or `0` if absent.
        //           (Sibling `.unwrap_or_else(|| ...)` defers the default; ours is a constant
        //           so `unwrap_or` is enough.)
        // Why:      An initial rate; priming below refreshes it from the decoder.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rate = audio.sampleRate ?? 0;
        // ```
        let rate = audio_params.sample_rate.unwrap_or(0);

        // What:     `let channels = match &audio_params.channels { Some(c) => c.count(), None => 0 };`.
        //           Pattern-match the borrowed `&Option<Channels>` (we match on `&...` so `c`
        //           is a `&Channels`, leaving `audio_params` intact): if present, `c.count()`
        //           returns the channel count as `usize`; else `0`.
        // Why:      An initial count; for AAC/ALAC it is `None` here and priming below fills
        //           it from the first decoded frame.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const channels = audio.channels ? audio.channels.count() : 0;
        // ```
        let channels = match &audio_params.channels {
            // What:     `Some(c) => c.count()`. Binds the inner `Channels` by reference to `c`
            //           and calls `.count()` (returns `usize`).
            // Why:      Real channel count when known.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // channels = c.count();
            // ```
            Some(c) => c.count(),
            // What:     `None => 0`. The absent case yields zero.
            // Why:      Unknown layout -> 0 (refreshed after priming).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // channels = 0;
            // ```
            None => 0,
        };

        // What:     `let n_frames = track.num_frames;`. Copy the optional total frame count
        //           off the track (Copy `Option<u64>`). Done before the struct literal so the
        //           `audio_params` borrow of `track` is free to end.
        // Why:      Duration is computed from it after priming reveals the true rate.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const nFrames = track.numFrames;
        // ```
        let n_frames = track.num_frames;

        // What:     `let spec = AudioSpec { rate, channels: channels as u16, duration_secs: 0.0 };`.
        //           Struct literal; `channels as u16` narrows the `usize` count to our `u16`
        //           field. `duration_secs` starts `0.0` and is set after priming reveals the
        //           true rate.
        // Why:      An initial spec; the priming step finalises rate/channels/duration.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const spec = { rate, channels, durationSecs: 0 };
        // ```
        let spec = AudioSpec {
            rate,
            channels: channels as u16,
            duration_secs: 0.0,
        };

        // What:     `let mut source = SymphoniaSource { ... };`. Build the struct (moving
        //           fields in), `pending` starts empty, `n_frames` from the container. `mut`
        //           because priming below mutates it. NOT wrapped in `Ok` yet: we prime first.
        // Why:      We need a live `source` to call `decode_next_raw` on for priming.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const source = new SymphoniaSource(format, decoder, trackId, spec, null, nFrames);
        // ```
        let mut source = SymphoniaSource {
            format,
            decoder,
            track_id,
            spec,
            pending: None,
            n_frames,
        };

        // What:     `let first = source.decode_next_raw()?;`. Decode the first audible chunk
        //           now. This advances the demuxer/decoder and, as a side effect, refreshes
        //           `source.spec.rate`/`channels` from the decoded frame's true signal spec.
        //           `?` propagates errors.
        // Why:      Learn the real channel count (AAC/ALAC report it only after the first
        //           decode) and capture the first audio block.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const first = source.decodeNextRaw();
        // ```
        let first = source.decode_next_raw()?;

        // What:     `let duration_secs = match (source.n_frames, source.spec.rate) { ... };`.
        //           Now that priming set the true rate, compute seconds (a `match` over the
        //           pair).
        // Why:      Duration needs the accurate rate, available only post-priming.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const durationSecs = source.nFrames != null && source.spec.rate ? source.nFrames / source.spec.rate : 0;
        // ```
        let duration_secs = match (source.n_frames, source.spec.rate) {
            // What:     `(Some(n), r) if r > 0 => n as f64 / r as f64`. A GUARDED arm: frame
            //           count present AND rate positive; cast both to f64 before dividing.
            // Why:      seconds = frames / frames-per-second.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return n / r;
            // ```
            (Some(n), r) if r > 0 => n as f64 / r as f64,
            // What:     `_ => 0.0`. Otherwise unknown duration.
            // Why:      Avoid divide-by-zero / unknown.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return 0;
            // ```
            _ => 0.0,
        };

        // What:     `source.spec.duration_secs = duration_secs;`. Store the computed length on
        //           the cached spec.
        // Why:      The seek bar's maximum comes from here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // source.spec.durationSecs = durationSecs;
        // ```
        source.spec.duration_secs = duration_secs;

        // What:     `source.pending = Some(first);`. Stash the primed first chunk so
        //           `next_chunk` hands it out before pulling new packets. `Some(...)` wraps
        //           the value into the `Option` field.
        // Why:      Do not lose the first audio block we decoded during priming.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // source.pending = first;
        // ```
        source.pending = Some(first);

        // What:     `Ok(source)`. Wrap the primed source as success. Tail -> return.
        // Why:      Hand back the ready source.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return source;
        // ```
        return Ok(source)
    }

    /// What:
    ///      `fn decode_next_raw(&mut self) -> Result<Vec<f32>, PlayerError>`.
    ///  A PRIVATE
    ///           helper (no `pub`):
    ///  pull packets until one decodes to a non-empty interleaved
    ///           block,
    ///  returning it;
    ///  an empty `Vec` means true end-of-stream.
    ///  Also refreshes
    ///           `self.spec.rate`/`channels` from each decoded frame's actual audio spec.
    /// Why:
    ///       Shared by `new` (priming) and `next_chunk` so the decode loop is written once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private decodeNextRaw(): number[] { ... } // [] means EOF
    /// ```
    fn decode_next_raw(&mut self) -> Result<Vec<f32>, PlayerError> {
        // What:     `loop { ... }`. Infinite loop; we `return` out. Needed because some
        //           packets are other tracks, fail to decode, or decode to zero frames
        //           (priming) and must be skipped.
        // Why:      Keep pulling until we get audible samples or hit EOF.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (true) { ... }
        // ```
        loop {
            // What:     `let packet = match self.format.next_packet() { ... };`. In 0.6
            //           `next_packet` returns `Result<Option<Packet>>`: `Ok(Some(p))` is a
            //           packet, `Ok(None)` is clean end-of-stream (0.5 signalled EOF via an
            //           `UnexpectedEof` IoError, which is now always an error).
            // Why:      Get a packet, handling end-of-stream cleanly.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const p = format.nextPacket(); // null at EOF
            // ```
            let packet = match self.format.next_packet() {
                // What:     `Ok(Some(p)) => p`. A packet was produced; unwrap it.
                // Why:      Something to consider.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // packet = p;
                // ```
                Ok(Some(p)) => p,
                // What:     `Ok(None) => return Ok(Vec::new())`. End of stream -> empty Vec
                //           (our EOF signal to callers).
                // Why:      Normal end of file.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return [];
                // ```
                Ok(None) => return Ok(Vec::new()),
                // What:     `Err(Error::ResetRequired) => return Ok(Vec::new())`. Treat
                //           reset-required (e.g. chained Ogg) as end-of-track.
                // Why:      End cleanly instead of resetting.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (e.kind === "ResetRequired") return [];
                // ```
                Err(Error::ResetRequired) => return Ok(Vec::new()),
                // What:     `Err(e) => return Err(e.into())`. `.into()` converts and propagates.
                // Why:      Surface genuine demux failures.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // throw toPlayerError(e);
                // ```
                Err(e) => return Err(e.into()),
            };

            // What:     `if packet.track_id != self.track_id { continue; }`. Skip packets from
            //           other tracks. In 0.6 `track_id` is a public FIELD (0.5 had a
            //           `track_id()` getter, now removed).
            // Why:      Containers can interleave multiple tracks.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (packet.trackId !== this.trackId) continue;
            // ```
            if packet.track_id != self.track_id {
                continue;
            }

            // What:     `match self.decoder.decode(&packet) { ... }`. Decode the packet;
            //           `&packet` lends it read-only. Returns a `GenericAudioBufferRef` (an
            //           enum over sample formats) borrowing the decoder.
            // Why:      Turn the packet into PCM, handling skippable errors.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // let audio; try { audio = decoder.decode(packet); } catch (e) { ... }
            // ```
            match self.decoder.decode(&packet) {
                // What:     `Ok(decoded) => { ... }`. `decoded` is the decoded buffer reference
                //           (borrows the decoder).
                // Why:      Refresh the spec and copy out interleaved f32.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // // decoded is the decoded audio buffer
                // ```
                Ok(decoded) => {
                    // What:     `let spec = decoded.spec();`. `.spec()` returns `&AudioSpec`
                    //           (0.5's `SignalSpec` was replaced by `AudioSpec`). Borrows
                    //           `decoded` read-only.
                    // Why:      The decoder's true rate/channels live here.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const spec = decoded.spec();
                    // ```
                    let spec = decoded.spec();

                    // What:     `self.spec.rate = spec.rate();`. `.rate()` reads the sample
                    //           rate (0.5 exposed a `rate` field; 0.6 uses a getter). Assigns
                    //           into `self.spec` (disjoint from `self.decoder`, which `decoded`
                    //           borrows).
                    // Why:      codec-header rate may be missing/wrong; trust the decode.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // this.spec.rate = spec.rate();
                    // ```
                    self.spec.rate = spec.rate();

                    // What:     `self.spec.channels = spec.channels().count() as u16;`.
                    //           `.channels()` returns `&Channels`; `.count()` -> `usize`; `as
                    //           u16` narrows to our field.
                    // Why:      Fills the channel count AAC/ALAC omit at probe time.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // this.spec.channels = spec.channels().count();
                    // ```
                    self.spec.channels = spec.channels().count() as u16;

                    // What:     `let mut out: Vec<f32> = Vec::new();`. A fresh growable f32
                    //           buffer. Explicit type annotation so the copy below infers the
                    //           sample type `f32`.
                    // Why:      The owned interleaved block we will return; replacing 0.5's
                    //           reused `SampleBuffer` (0.6 has no `SampleBuffer`).
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const out: number[] = [];
                    // ```
                    let mut out: Vec<f32> = Vec::new();

                    // What:     `decoded.copy_to_vec_interleaved(&mut out);`. Copies the planar
                    //           samples into `out` as interleaved f32, RESIZING `out` to the
                    //           exact interleaved length first (so a stale length cannot leak
                    //           through). `&mut out` lends it mutably.
                    // Why:      Produce `[L,R,L,R,...]` in one call (0.5 needed a `SampleBuffer`
                    //           + `copy_interleaved_ref` + `samples()`).
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // decoded.copyToVecInterleaved(out);
                    // ```
                    decoded.copy_to_vec_interleaved(&mut out);

                    // What:     `if out.is_empty() { continue; }`. A 0-frame packet (e.g.
                    //           Vorbis priming) yields nothing; fetch the next packet instead
                    //           of signalling EOF.
                    // Why:      An empty return means EOF to callers; do not confuse a priming
                    //           packet with end-of-stream.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // if (out.length === 0) continue;
                    // ```
                    if out.is_empty() {
                        continue;
                    }

                    // What:     `return Ok(out);`. Wrap the owned interleaved samples as
                    //           success and return (moves `out`, no extra copy).
                    // Why:      Hand back owned interleaved samples.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // return out;
                    // ```
                    return Ok(out);
                }
                // What:     `Err(Error::DecodeError(_)) => continue`. Skip one bad packet (the
                //           `_` ignores the message).
                // Why:      One corrupt packet should not kill playback.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (e.kind === "DecodeError") continue;
                // ```
                Err(Error::DecodeError(_)) => continue,
                // What:     `Err(Error::IoError(_)) => continue`. A packet that failed to decode
                //           due to an I/O error is skipped (per the 0.6 decode-loop guidance);
                //           true EOF now arrives as `Ok(None)`.
                // Why:      Robustness against partial packets.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (e.kind === "IoError") continue;
                // ```
                Err(Error::IoError(_)) => continue,
                // What:     `Err(e) => return Err(e.into())`. Fatal: convert+propagate.
                // Why:      Surface genuine decoder failures.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // throw toPlayerError(e);
                // ```
                Err(e) => return Err(e.into()),
            }
        }
    }
}

/// What:
///      `impl Source for SymphoniaSource { ... }`.
///  Implements our `Source` interface for
///           this type.
/// Why:
///       So `open()` can return it as `Box<dyn Source>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // SymphoniaSource implements Source: spec(), next_chunk(), seek()
/// ```
impl Source for SymphoniaSource {
    /// What:
    ///      `fn spec(&self) -> AudioSpec { self.spec }`.
    ///  Read-only borrow;
    ///  returns a COPY
    ///           of the cached spec (`AudioSpec` is `Copy`).
    /// Why:
    ///       Hand callers the stream shape.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spec(): AudioSpec { return this.spec; }
    /// ```
    fn spec(&self) -> AudioSpec {
        // What:     `self.spec` as the tail expression -> returned by value (copy).
        // Why:      Return the cached spec.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.spec;
        // ```
        return self.spec
    }

    /// What:
    ///      `fn next_chunk(&mut self) -> Result<Vec<f32>, PlayerError>`.
    ///  Exclusive borrow;
    ///           advances the demuxer/decoder by one packet.
    /// Why:
    ///       Produce the next block of interleaved samples (or EOF).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextChunk(): number[] { ... }
    /// ```
    fn next_chunk(&mut self) -> Result<Vec<f32>, PlayerError> {
        // What:     `if let Some(chunk) = self.pending.take() { return Ok(chunk); }`. `.take()`
        //           REPLACES `self.pending` with `None` and returns its previous value as an
        //           `Option`. `if let Some(chunk) = ...` runs only when a buffered chunk
        //           exists, binding it to `chunk`.
        // Why:      The first call hands out the chunk decoded during priming (in `new`)
        //           before pulling any further packets.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.pending != null) { const c = this.pending; this.pending = null; return c; }
        // ```
        if let Some(chunk) = self.pending.take() {
            // What:     `return Ok(chunk);`. Wrap the buffered chunk as success.
            // Why:      Deliver the primed first block.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return chunk;
            // ```
            return Ok(chunk);
        }

        // What:     `self.decode_next_raw()`. Tail expression: delegate to the shared decode
        //           loop (defined in the inherent impl above), which pulls/decodes the next
        //           audible block or signals EOF.
        // Why:      All steady-state decoding goes through one place.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.decodeNextRaw();
        // ```
        return self.decode_next_raw()
    }

    /// What:
    ///      `fn seek(&mut self, secs: f64) -> Result<(), PlayerError>`.
    ///  Jump the demuxer
    ///           to a time,
    ///  then reset the decoder.
    /// Why:
    ///       Implement the seek control.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seek(secs: number): void { ... }
    /// ```
    fn seek(&mut self, secs: f64) -> Result<(), PlayerError> {
        // What:     `seek_format(self.format.as_mut(), self.track_id, secs)?`. Call the shared
        //           helper (defined above this struct). `self.format` is a `Box<dyn
        //           FormatReader>` (an owned, heap demuxer); `.as_mut()` reborrows it as a
        //           `&mut dyn FormatReader` so the helper can seek it without taking
        //           ownership. `?` unwraps the `Ok` or returns the error.
        // Why:      The helper converts our "seconds from audible start" into the container's
        //           absolute frame timestamp (adding the stream's `start_ts`), so dragging the
        //           bar to the very beginning seeks to the real first frame instead of the
        //           invalid frame 0.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // seekFormat(this.format, this.trackId, secs);
        // ```
        seek_format(self.format.as_mut(), self.track_id, secs)?;

        // What:     `self.decoder.reset();`. Clears the decoder's internal state so it does
        //           not emit stale samples from before the seek.
        // Why:      Required after a demuxer seek for correct output.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.decoder.reset();
        // ```
        self.decoder.reset();

        // What:     `self.pending = None;`. Drop any buffered primed chunk; after a seek it
        //           belongs to the pre-seek position.
        // Why:      Avoid replaying the start of the track when seeking before the first
        //           `next_chunk` consumed the primed chunk.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.pending = null;
        // ```
        self.pending = None;

        // What:     `Ok(())`. Wrap the unit value `()` as success. Tail -> return.
        // Why:      Seek succeeded with no value to return.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return;
        // ```
        return Ok(())
    }
}

/// What:
///      `#[cfg(test)] #[path = "decode_tests.rs"] mod tests;` declares a test-only
///           submodule whose code lives in the sibling file `decode_tests.rs`.
///  `#[cfg(test)]`
///           gates it to test builds only;
///  `#[path = "..."]` aims the module at a flat sibling
///           file instead of the default `decode/tests.rs` subdirectory lookup.
///  The file stays
///           the `tests` CHILD of decode,
///  so its `use super::*` reaches the module items
///           (including private ones) unchanged.
/// Why:
///       Keep `decode.rs` to production code;
///  the tests live beside it without inflating
///           this file or its max-lines budget (sibling `*_tests.rs` files are exempt from the
///           linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // decode.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "decode_tests.rs"]
mod tests;
