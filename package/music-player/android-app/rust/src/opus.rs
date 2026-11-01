//! Opus decode path.
//!  symphonia 0.6 demuxes Ogg/Opus (parsing `OpusHead`,
//! reporting the pre-skip in `Track::delay`,
//!  and yielding raw Opus packets) but
//! the symphonia meta-crate exposes no Opus decoder (the `symphonia-codec-opus`
//! crate exists but is not wired into the `all` feature set),
//!  so we feed those
//! raw packets to libopus through the `opus` crate.
//!  Output is always 48 kHz.
//! This is the Android port;
//!  the decode semantics match the desktop crate.

/// What:
///      `use symphonia::core::errors::Error;` imports symphonia's error enum.
/// Why:
///       We match its `ResetRequired` variant (treated as end-of-track) and propagate
///           any other error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SymphoniaError } from "symphonia/errors";
/// ```
use symphonia::core::errors::Error;

/// What:
///      `use symphonia::core::formats::{FormatReader, Track};`.
///  `FormatReader` = the
///           demuxer trait;
///  `Track` = one track's id,
///  codec params,
///  and timing (channels,
///           pre-skip `delay`,
///  `num_frames`).
///  The seeking enums (`SeekMode`/`SeekTo`) are
///           not imported here because the actual seek happens inside `seek_format` in
///           `decode.rs`;
///  this module holds the demuxer and delegates.
/// Why:
///       `OpusSource` stores a `Box<dyn FormatReader>` and reads channels/delay/
///           frame-count from the owned `Track` `new` receives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FormatReader, Track } from "symphonia/formats";
/// ```
use symphonia::core::formats::{FormatReader, Track};

/// What:
///      `use crate::decode::{seek_format, AudioSpec, Source};` imports the shared seek
///           helper,
///  our spec record,
///  and the `Source` interface from the sibling
///           `decode.rs` module.
///  The `{ ... }` is a grouped import (one statement pulling
///           several names from the same path);
///  the names appear in the file's own order
///           (`seek_format` first),
///  which has no runtime meaning,
///  it is just how the line
///           is written.
/// Why:
///       `OpusSource` returns an `AudioSpec`,
///  implements `Source`,
///  and delegates its
///           `seek` to `seek_format` so the start-frame math lives in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { seekFormat, AudioSpec, Source } from "./decode";
/// ```
use crate::decode::{seek_format, AudioSpec, Source};

/// What:
///      `use crate::error::PlayerError;` imports our app-wide error type.
/// Why:
///       Every fallible method here returns `PlayerError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PlayerError } from "@/error";
/// ```
use crate::error::PlayerError;

/// What:
///      `const OPUS_RATE: u32 = 48_000;`.
///  A named compile-time constant:
///  the fixed
///           output sample rate libopus always decodes to.
///  `u32` is a 32-bit UNSIGNED
///           integer;
///  siblings the reader might expect:
///  `i32` (signed),
///  `u16`,
///  `u64`,
///           `usize`.
///  The `_` in `48_000` is a digit separator (purely cosmetic).
/// Why:
///       Opus is defined to output 48 kHz;
///  naming it avoids a magic number.
///  `u32` is
///           chosen (not `i32`/`u64`) because it matches `AudioSpec.rate` and the opus
///           crate's API,
///  so no casts are needed when this value flows into them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OPUS_RATE = 48_000;
/// ```
const OPUS_RATE: u32 = 48_000;

/// What:
///      `const MAX_FRAMES_PER_CHANNEL: usize = 5760;`.
///  The largest number of samples
///           per channel a single Opus packet can decode to (a 120 ms frame at 48 kHz =
///           0.120 * 48000 = 5760).
///  `usize` is the unsigned integer wide enough to address
///           any byte in memory on this platform (32 bits on a 32-bit OS,
///  64 on a 64-bit
///           OS);
///  siblings:
///  `u32`,
///  `u64`,
///  `i32`,
///  `i64`.
/// Why:
///       We pre-allocate a scratch buffer big enough for any packet so `decode_float`
///           never overflows it.
///  `usize` (not `u32`/`u64`) because it sizes and indexes a
///           buffer,
///  which is exactly what every std collection/slice API wants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_FRAMES_PER_CHANNEL = 5760;
/// ```
const MAX_FRAMES_PER_CHANNEL: usize = 5760;

/// What:
///      `const STEREO: usize = 2;`.
///  Named constant for the stereo channel count,
///  used
///           to branch mono vs stereo.
///  `usize` (not `u32`/`u8`) so it compares against the
///           `usize` channel count without a cast.
/// Why:
///       Avoid a bare `2` literal when classifying the layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STEREO = 2;
/// ```
const STEREO: usize = 2;

/// What:
///      `const MONO: usize = 1;`.
///  Named constant for the mono channel count.
///  `usize`
///           (not `u32`/`u8`) so it compares against the `usize` channel count without a
///           cast.
/// Why:
///       Avoid a bare `1` literal when classifying the layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MONO = 1;
/// ```
const MONO: usize = 1;

/// What:
///      `pub struct OpusSource { ... }`.
///  `pub` = visible outside this module;
///  `struct`
///           declares a record type (a bundle of named fields).
///  This one holds the live
///           Opus decode state.
/// Why:
///       Bundles the demuxer,
///  libopus decoder,
///  and reusable scratch so the `Source`
///           methods can advance them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class OpusSource implements Source { format; decoder; trackId; channels; spec; scratch; preSkip; }
/// ```
pub struct OpusSource {
    /// What:
    ///      `format: Box<dyn FormatReader>`.
    ///  `Box<T>` is an owning pointer to a
    ///           heap-allocated `T` with a SINGLE owner;
    ///  `dyn FormatReader` is a
    ///           type-erased trait object (any concrete demuxer implementing the trait).
    ///           Siblings of `Box`:
    ///  `Rc<T>` and `Arc<T>` (both shared/reference-counted).
    /// Why:
    ///       Source of raw Opus packets;
    ///  we own it exclusively and free it when the
    ///           struct drops.
    ///  `Box` (not `Rc`/`Arc`) because nothing else shares it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// format: FormatReader;
    /// ```
    format: Box<dyn FormatReader>,
    /// What:
    ///      `decoder: opus::Decoder`.
    ///  The libopus decoder VALUE stored inline (owned by
    ///           this struct,
    ///  not boxed:
    ///  it is a concrete type,
    ///  not a trait object).
    ///  The
    ///           `opus::` prefix is a path into the `opus` crate's module.
    /// Why:
    ///       Decodes each Opus packet to f32 PCM.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// decoder: OpusDecoder;
    /// ```
    decoder: opus::Decoder,
    /// What:
    ///      `track_id: u32`.
    ///  Id of the Opus track.
    ///  `u32` (not `usize`/`u64`) because it
    ///           matches symphonia's track-id width.
    /// Why:
    ///       Skip packets from other tracks.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trackId: number;
    /// ```
    track_id: u32,
    /// What:
    ///      `channels: usize`.
    ///  Channel count kept as `usize` (1 or 2).
    ///  `usize` (not
    ///           `u16`/`u32`) so the buffer math (`frames * channels`,
    ///  slice indices) needs
    ///           no casts.
    /// Why:
    ///       Slice the decoded output and size the scratch buffer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// channels: number;
    /// ```
    channels: usize,
    /// What:
    ///      `spec: AudioSpec`.
    ///  Cached rate(=48000)/channels/duration record,
    ///  stored by
    ///           value (owned inline).
    /// Why:
    ///       `spec()` returns it directly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spec: AudioSpec;
    /// ```
    spec: AudioSpec,
    /// What:
    ///      `scratch: Vec<f32>`.
    ///  `Vec<f32>` is a heap-allocated growable array of 32-bit
    ///           floats that THIS struct owns;
    ///  siblings:
    ///  `&[f32]` (a borrowed view that
    ///           owns nothing) and `[f32; N]` (a fixed-size stack array).
    ///  libopus writes
    ///           into it each call (sized `MAX_FRAMES_PER_CHANNEL * channels`).
    /// Why:
    ///       Avoid allocating a fresh buffer per packet.
    ///  `Vec` (not `&[f32]`/`[f32; N]`)
    ///           because it is owned,
    ///  reusable,
    ///  and sized at runtime from the channel count.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scratch: Float32Array;
    /// ```
    scratch: Vec<f32>,
    /// What:
    ///      `pre_skip: usize`.
    ///  Remaining encoder-delay frames-per-channel to discard at
    ///           the very start (Opus prepends silence/priming).
    ///  `usize` (not `u32`) to match
    ///           the frame-count arithmetic it participates in.
    /// Why:
    ///       Dropping them avoids a click and aligns playback to t=0.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// preSkip: number;
    /// ```
    pre_skip: usize,
}

/// What:
///      `impl OpusSource { ... }`.
///  An inherent-impl block:
///  it attaches methods directly
///           to the `OpusSource` type (here just the constructor `new`),
///  not via a trait.
/// Why:
///       Holds `new`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class OpusSource { static create(...) { ... } }
/// ```
impl OpusSource {
    /// What:
    ///      `pub fn new(format: Box<dyn FormatReader>, track: Track, track_id: u32) -> Result<Self, PlayerError>`.
    ///           A public function.
    ///  It takes ownership of the boxed demuxer and the owned
    ///           `Track` (both moved in),
    ///  plus a `u32` track id.
    ///  `-> Result<Self, PlayerError>`
    ///           returns either `Ok(Self)` on success or `Err(PlayerError)` on failure;
    ///           `Result<T, E>` is Rust's two-variant outcome type (it has no exceptions).
    ///           `Self` is an alias for `OpusSource`.
    /// Why:
    ///       Set up Opus decoding for this track,
    ///  rejecting >2 channels.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static create(format, track, trackId): OpusSource { ... }
    /// ```
    pub fn new(
        format: Box<dyn FormatReader>,
        track: Track,
        track_id: u32,
    ) -> Result<Self, PlayerError> {
        // What:     `let audio_params = track.codec_params.as_ref().and_then(|cp| cp.audio()).ok_or_else(|| ...)?;`.
        //           `track.codec_params` is an `Option<CodecParameters>` (maybe-present).
        //           `.as_ref()` turns `&Option<T>` into `Option<&T>` so we borrow the inside
        //           rather than move it out. `.and_then(|cp| cp.audio())` runs the closure
        //           `|cp| cp.audio()` only when present, flattening `Option<Option<_>>` to
        //           one `Option`. `.ok_or_else(|| ...)` converts `None` into an `Err` built
        //           by the closure. The trailing `?` unwraps `Ok` or returns the `Err` from
        //           `new`.
        // Why:      The channel layout lives on the audio params.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const audio = track.codecParams?.audio();
        // if (!audio) throw new PlayerError.Unsupported("opus: no audio codec parameters");
        // ```
        let audio_params = track
            .codec_params
            .as_ref()
            .and_then(|cp| return cp.audio())
            .ok_or_else(|| {
                // What:     `PlayerError::Unsupported("opus: no audio codec parameters".to_string())`.
                //           Constructs the `Unsupported` variant of our error enum
                //           (`::` reaches a variant inside the type). The literal is a
                //           `&'static str` (borrowed, baked into the binary); `.to_string()`
                //           ALLOCATES a fresh owned `String` copy of it.
                // Why:      The `Unsupported` variant holds an owned `String`, so we must
                //           convert the borrowed literal into one the error can keep.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // new PlayerError.Unsupported("opus: no audio codec parameters")
                // ```
                return PlayerError::Unsupported("opus: no audio codec parameters".to_string())
            })?;

        // What:     `let channels = match &audio_params.channels { ... };`. `match` is
        //           pattern-matching (an exhaustive switch on a value's shape). `&audio_params.channels`
        //           BORROWS the `Option<Channels>` field (the `&` means "lend, do not move
        //           out"), so each arm sees a `&Channels`. The result of the whole `match`
        //           is assigned to `channels`.
        // Why:      We must know the layout to configure libopus.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const channels = audio.channels?.count();
        // if (channels == null) throw new PlayerError.Unsupported("opus: unknown channels");
        // ```
        let channels = match &audio_params.channels {
            // What:     `Some(c) => c.count()`. The `Some(c)` pattern matches the present
            //           variant of `Option` and binds its inner `&Channels` to `c`;
            //           `c.count()` returns the channel count as a `usize`. This arm's value
            //           becomes the `match` result.
            // Why:      Real channel count.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // channels = c.count();
            // ```
            Some(c) => c.count(),
            // What:     `None => { return Err(...) }`. The `None` pattern matches the absent
            //           variant of `Option`. `Err(PlayerError::Unsupported("...".to_string()))`
            //           builds the failure variant of `Result`, wrapping an owned `String`
            //           (`.to_string()` allocates it from the borrowed literal); `return`
            //           exits `new` early with that error.
            // Why:      Cannot decode without knowing channels.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw new PlayerError.Unsupported("opus: unknown channel layout");
            // ```
            None => {
                return Err(PlayerError::Unsupported(
                    "opus: unknown channel layout".to_string(),
                ))
            }
        };

        // What:     `let opus_channels = match channels { ... };`. A `match` on the numeric
        //           `channels` (`usize`) that maps it to libopus's `Channels` enum, with a
        //           catch-all arm rejecting anything else. Result assigned to `opus_channels`.
        // Why:      libopus's API takes the enum, not a number.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const opusChannels = channels === 1 ? Channels.Mono
        //   : channels === 2 ? Channels.Stereo
        //   : (() => { throw new PlayerError.Unsupported(`opus: ${channels} channels`); })();
        // ```
        let opus_channels = match channels {
            // What:     `MONO => opus::Channels::Mono`. Matches when `channels` equals the
            //           `MONO` constant (1); yields `opus::Channels::Mono`, a unit enum
            //           variant (a variant carrying no data) reached via `::`.
            // Why:      One channel.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case 1: return Channels.Mono;
            // ```
            MONO => opus::Channels::Mono,
            // What:     `STEREO => opus::Channels::Stereo`. Matches when `channels` equals
            //           `STEREO` (2); yields the stereo unit variant.
            // Why:      Two channels.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case 2: return Channels.Stereo;
            // ```
            STEREO => opus::Channels::Stereo,
            // What:     `other => { return Err(PlayerError::Unsupported(format!(...))) }`. A
            //           catch-all arm: the bare name `other` binds whatever count did not
            //           match above. `format!("...{other}...")` is a macro (the `!` marks a
            //           macro call) that builds an owned `String` with `other` interpolated.
            //           `Err(PlayerError::Unsupported(...))` wraps it as the failure variant;
            //           `return` exits early.
            // Why:      We only support mono/stereo (no surround) in this player.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw new PlayerError.Unsupported(`opus: ${other} channels (only mono/stereo)`);
            // ```
            other => {
                return Err(PlayerError::Unsupported(format!(
                    "opus: {other} channels (only mono/stereo supported)"
                )))
            }
        };

        // What:     `let decoder = opus::Decoder::new(OPUS_RATE, opus_channels)?;`.
        //           `opus::Decoder::new(...)` is the associated constructor of the libopus
        //           decoder (`::new` is a static-like function on the type), called at 48 kHz
        //           for the chosen layout. It returns `Result<Decoder, opus::Error>`; the
        //           trailing `?` unwraps `Ok` or, via a `From` conversion, turns an
        //           `opus::Error` into `PlayerError` and returns it from `new`.
        // Why:      The decoder we feed packets to.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const decoder = new OpusDecoder(OPUS_RATE, opusChannels);
        // ```
        let decoder = opus::Decoder::new(OPUS_RATE, opus_channels)?;

        // What:     `let pre_skip = track.delay.unwrap_or(0) as usize;`. `track.delay` is an
        //           `Option<u32>` (the Ogg mapper read the pre-skip from `OpusHead`).
        //           `.unwrap_or(0)` returns the inner `u32` if present, else substitutes `0`.
        //           `as usize` is a numeric cast widening the `u32` to the buffer-math type.
        // Why:      Number of priming frames-per-channel to discard at the start.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const preSkip = track.delay ?? 0;
        // ```
        let pre_skip = track.delay.unwrap_or(0) as usize;

        // What:     `let duration_secs = match track.num_frames { ... };`. `track.num_frames`
        //           is an `Option<u64>` (total frames, always at 48 kHz for Opus). The
        //           `match` yields a seconds value, assigned to `duration_secs`.
        // Why:      The seek bar needs the track length.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const durationSecs = track.numFrames != null ? track.numFrames / 48000 : 0;
        // ```
        let duration_secs = match track.num_frames {
            // What:     `Some(n) => n as f64 / OPUS_RATE as f64`. `Some(n)` binds the present
            //           frame count; `n as f64` and `OPUS_RATE as f64` cast both operands to
            //           64-bit float (sibling: `f32`) BEFORE dividing, so the division is
            //           float (integer `/` would truncate). This arm's value is the result.
            // Why:      seconds = frames / 48000.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return n / 48000;
            // ```
            Some(n) => n as f64 / OPUS_RATE as f64,
            // What:     `None => 0.0`. The absent variant yields the float literal `0.0`
            //           (the `.0` makes it an `f64`, not an integer).
            // Why:      Unknown frame count -> unknown duration; avoid guessing.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return 0;
            // ```
            None => 0.0,
        };

        // What:     `let spec = AudioSpec { rate: OPUS_RATE, channels: channels as u16, duration_secs };`.
        //           A struct literal building an `AudioSpec`. `rate: OPUS_RATE` sets the rate
        //           field; `channels: channels as u16` casts the `usize` count NARROWER to a
        //           `u16` field; `duration_secs` is field shorthand (field name equals the
        //           local of the same name).
        // Why:      Report 48 kHz + layout + length to callers.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const spec = { rate: OPUS_RATE, channels, durationSecs };
        // ```
        let spec = AudioSpec {
            rate: OPUS_RATE,
            channels: channels as u16,
            duration_secs,
        };

        // What:     `let scratch = vec![0.0f32; MAX_FRAMES_PER_CHANNEL * channels];`. The
        //           `vec![value; count]` macro (the `!` marks it a macro) builds a `Vec<f32>`
        //           holding `count` copies of `0.0f32` (the `f32` suffix pins the element
        //           type to 32-bit float). The count is `MAX_FRAMES_PER_CHANNEL * channels`.
        // Why:      A buffer guaranteed large enough for one decoded Opus packet.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const scratch = new Float32Array(MAX_FRAMES_PER_CHANNEL * channels);
        // ```
        let scratch = vec![0.0f32; MAX_FRAMES_PER_CHANNEL * channels];

        // What:     `Ok(OpusSource { format, decoder, track_id, channels, spec, scratch, pre_skip })`.
        //           Builds the struct (every field is shorthand: field name = local of the
        //           same name) and wraps it in `Ok`, the success variant of `Result`. No
        //           trailing `;`, so this is the function's tail expression: Rust auto-returns
        //           it from `new`.
        // Why:      Return the ready Opus source with the success channel set.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new OpusSource(format, decoder, trackId, channels, spec, scratch, preSkip);
        // ```
        return Ok(OpusSource {
            format,
            decoder,
            track_id,
            channels,
            spec,
            scratch,
            pre_skip,
        })
    }
}

/// What:
///      `impl Source for OpusSource { ... }`.
///  A trait-impl block:
///  it makes `OpusSource`
///           satisfy the shared `Source` interface by providing the trait's methods.
/// Why:
///       So `open()` can return it as `Box<dyn Source>` (a type-erased `Source`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // OpusSource implements the Source interface: spec(), next_chunk(), seek()
/// ```
impl Source for OpusSource {
    /// What:
    ///      `fn spec(&self) -> AudioSpec { self.spec }`.
    ///  A method taking `&self` (an
    ///           immutable BORROW of the instance:
    ///  read-only,
    ///  no ownership taken) and
    ///           returning an `AudioSpec` by value.
    /// Why:
    ///       Report the stream shape.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spec(): AudioSpec { return this.spec; }
    /// ```
    fn spec(&self) -> AudioSpec {
        // What:     `self.spec`. A bare tail expression (no `;`), so it is RETURNED. Because
        //           `AudioSpec` derives `Copy`, returning it makes a bitwise COPY rather than
        //           moving the field out of `self` (which a `&self` borrow could not do).
        // Why:      Hand back the spec.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.spec;
        // ```
        return self.spec
    }

    /// What:
    ///      `fn next_chunk(&mut self) -> Result<Vec<f32>, PlayerError>`.
    ///  A method taking
    ///           `&mut self` (an EXCLUSIVE mutable borrow:
    ///  it may mutate the instance,
    ///  and no
    ///           other borrow may coexist) because decoding advances the demuxer and decoder.
    ///           Returns `Ok(Vec<f32>)` of interleaved samples or an `Err(PlayerError)`.
    /// Why:
    ///       Produce the next PCM block (or EOF).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextChunk(): number[] { ... }
    /// ```
    fn next_chunk(&mut self) -> Result<Vec<f32>, PlayerError> {
        // What:     `loop { ... }`. An infinite loop (exits only via an inner `return`/`break`).
        //           Needed because some packets belong to other tracks or are fully consumed
        //           by pre-skip and yield nothing.
        // Why:      Keep going until we have audible samples or hit EOF.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (true) { ... }
        // ```
        loop {
            // What:     `let packet = match self.format.next_packet() { ... };`.
            //           `self.format.next_packet()` returns `Result<Option<Packet>, Error>`:
            //           `Ok(Some(p))` is a real packet, `Ok(None)` is clean end-of-stream,
            //           `Err(...)` is a demux error. The `match` destructures all three and
            //           assigns the unwrapped packet to `packet`.
            // Why:      Get a raw Opus packet, handling end-of-stream.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const p = format.nextPacket(); // null at EOF
            // ```
            let packet = match self.format.next_packet() {
                // What:     `Ok(Some(p)) => p`. Nested patterns: `Ok` (success) wrapping
                //           `Some` (present), binding the inner `Packet` to `p`; the arm
                //           yields `p` as the `match` result.
                // Why:      We have something to decode.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // packet = p;
                // ```
                Ok(Some(p)) => p,
                // What:     `Ok(None) => return Ok(Vec::new())`. Success-but-absent means
                //           end-of-stream. `Vec::new()` is the associated constructor for an
                //           EMPTY `Vec<f32>` (no allocation); `Ok(...)` wraps it; `return`
                //           exits `next_chunk`. An empty Vec is our agreed EOF signal.
                // Why:      End of file -> stop cleanly.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return [];
                // ```
                Ok(None) => return Ok(Vec::new()),
                // What:     `Err(Error::ResetRequired) => return Ok(Vec::new())`. Matches the
                //           specific `ResetRequired` variant inside the `Err`; we treat it as
                //           end-of-track and return the empty-Vec EOF signal.
                // Why:      Simple player: end instead of resetting.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (e.kind === "ResetRequired") return [];
                // ```
                Err(Error::ResetRequired) => return Ok(Vec::new()),
                // What:     `Err(e) => return Err(e.into())`. Any other error binds to `e`;
                //           `.into()` calls the `From`/`Into` conversion that turns a
                //           `symphonia` `Error` into our `PlayerError`; `Err(...)` re-wraps it
                //           and `return` propagates it.
                // Why:      Surface genuine demux failures.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // throw toPlayerError(e);
                // ```
                Err(e) => return Err(e.into()),
            };

            // What:     `if packet.track_id != self.track_id { continue; }`. Compares the
            //           packet's track id against ours; `!=` is plain inequality. `continue`
            //           skips to the next loop iteration. In symphonia 0.6 `track_id` is a
            //           public FIELD (not a getter).
            // Why:      A container can interleave multiple tracks.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (packet.trackId !== this.trackId) continue;
            // ```
            if packet.track_id != self.track_id {
                continue;
            }

            // What:     `let frames = self.decoder.decode_float(&packet.data, &mut self.scratch, false)?;`.
            //           `packet.data` is the raw Opus bytes (`Box<[u8]>`, a boxed byte slice);
            //           `&packet.data` borrows it read-only and deref-coerces to the `&[u8]`
            //           libopus wants. `&mut self.scratch` lends the output buffer MUTABLY so
            //           the decoder can write into it. `false` = "this is not
            //           forward-error-correction recovery". It returns `Result<usize, opus::Error>`
            //           (frames PER CHANNEL); the trailing `?` unwraps or converts the error
            //           to `PlayerError` and returns.
            // Why:      Decode one packet into the scratch buffer.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const frames = decoder.decodeFloat(packet.data, scratch, false);
            // ```
            let frames = self
                .decoder
                .decode_float(&packet.data, &mut self.scratch, false)?;

            // What:     `let total = frames * self.channels;`. Plain multiplication of two
            //           `usize` values; `self.channels` reads the field through `&mut self`.
            //           Result is the total INTERLEAVED sample count (frames-per-channel
            //           times channels).
            // Why:      That many leading entries of `scratch` are valid this call.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const total = frames * this.channels;
            // ```
            let total = frames * self.channels;

            // What:     `let drop_frames = self.pre_skip.min(frames);`. `.min(frames)` returns
            //           the smaller of the remaining pre-skip and this packet's frame count
            //           (both `usize`), so we never drop more than this packet contains.
            // Why:      Discard priming frames spread across the first packet(s).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const dropFrames = Math.min(this.preSkip, frames);
            // ```
            let drop_frames = self.pre_skip.min(frames);

            // What:     `self.pre_skip -= drop_frames;`. Subtract-assign on the field through
            //           the mutable `self` borrow, reducing the remaining pre-skip counter.
            // Why:      Track how much priming is left for later packets.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.preSkip -= dropFrames;
            // ```
            self.pre_skip -= drop_frames;

            // What:     `let start = drop_frames * self.channels;`. Plain `usize`
            //           multiplication giving the interleaved offset where audible samples
            //           begin (after the dropped priming frames).
            // Why:      Slice past the priming samples.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const start = dropFrames * this.channels;
            // ```
            let start = drop_frames * self.channels;

            // What:     `let samples = &self.scratch[start..total];`. `start..total` is a
            //           half-open range (`start` included, `total` excluded). Indexing a
            //           `Vec` with a range yields a `[f32]` slice; the leading `&` BORROWS it
            //           as `&[f32]` (a read-only view that owns nothing, not a copy).
            // Why:      The portion we actually return.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const samples = scratch.subarray(start, total);
            // ```
            let samples = &self.scratch[start..total];

            // What:     `if samples.is_empty() { continue; }`. `.is_empty()` returns `true`
            //           when the slice has length 0; `continue` then jumps to the next loop
            //           iteration to fetch another packet.
            // Why:      Never return an empty Vec except at true EOF (which would be misread
            //           as end-of-stream).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (samples.length === 0) continue;
            // ```
            if samples.is_empty() {
                continue;
            }

            // What:     `return Ok(samples.to_vec());`. `.to_vec()` COPIES the borrowed
            //           `&[f32]` slice into a freshly allocated owned `Vec<f32>` (the caller
            //           keeps it past this stack frame, which a borrow could not allow);
            //           `Ok(...)` wraps it as the success variant; `return` hands it back.
            // Why:      Hand the caller owned interleaved samples.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return Array.from(samples);
            // ```
            return Ok(samples.to_vec());
        }
    }

    /// What:
    ///      `fn seek(&mut self, secs: f64) -> Result<(), PlayerError>`.
    ///  A method taking
    ///           `&mut self` (exclusive mutable borrow) and a 64-bit float `secs`.
    ///  It returns
    ///           `Result<(), PlayerError>`;
    ///  `()` is the unit type (the "no value" type,
    ///  like
    ///           TS `void`),
    ///  so success carries no payload.
    /// Why:
    ///       Jump the demuxer to a time and clear decoder state.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seek(secs: number): void { ... }
    /// ```
    fn seek(&mut self, secs: f64) -> Result<(), PlayerError> {
        // What:     `seek_format(self.format.as_mut(), self.track_id, secs)?`. Calls the
        //           shared helper from `decode.rs`. `self.format` is a `Box<dyn FormatReader>`
        //           (an owned, heap Ogg demuxer); `.as_mut()` reborrows it as
        //           `&mut dyn FormatReader` so the helper can seek it WITHOUT taking
        //           ownership. The trailing `?` unwraps the `Ok(())` or returns the error.
        // Why:      Opus streams begin at a non-zero frame (the pre-skip becomes the track's
        //           `start_ts`), so a naive `SeekTo::Time { time: 0s }` mapped to frame 0 and
        //           was rejected as out-of-range whenever the bar was dragged to the
        //           beginning. The helper adds `start_ts`, so second 0 lands on the first
        //           audible frame.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // seekFormat(this.format, this.trackId, secs);
        // ```
        seek_format(self.format.as_mut(), self.track_id, secs)?;

        // What:     `self.decoder.reset_state()?;`. Calls libopus's state reset on the owned
        //           decoder (through the mutable `self` borrow) so post-seek output has no
        //           leftover from before. It returns `Result<(), opus::Error>`; the trailing
        //           `?` unwraps or converts/propagates the error as `PlayerError`.
        // Why:      Avoid stale samples / artifacts after a seek.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.decoder.resetState();
        // ```
        self.decoder.reset_state()?;

        // What:     `self.pre_skip = 0;`. Plain assignment to the field through the mutable
        //           `self` borrow. After seeking we are mid-stream, so there is no encoder
        //           priming left to discard.
        // Why:      Pre-skip only applies at the very start of the stream.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.preSkip = 0;
        // ```
        self.pre_skip = 0;

        // What:     `Ok(())`. Builds the success variant of `Result` carrying `()` (the unit
        //           value, "nothing"). No trailing `;`, so this is the tail expression and is
        //           auto-returned from `seek`.
        // Why:      Signal the seek completed without error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return;
        // ```
        return Ok(())
    }
}
