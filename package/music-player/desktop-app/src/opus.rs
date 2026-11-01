//! Opus decode path.
//!  symphonia 0.6 demuxes Ogg/Opus (parsing `OpusHead`,
//! reporting the pre-skip in `Track::delay`,
//!  and yielding raw Opus packets) but
//! the symphonia meta-crate exposes no Opus decoder (the `symphonia-codec-opus`
//! crate exists but is not wired into the `all` feature set),
//!  so we feed those
//! raw packets to libopus through the `opus` crate.
//!  Output is always 48 kHz.

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
///      `use crate::decode::{AudioSpec, Source, seek_format};` imports our spec
///           record,
///  the `Source` interface,
///  and the shared seek helper from the sibling
///           `decode.rs` module.
/// Why:
///       `OpusSource` returns an `AudioSpec`,
///  implements `Source`,
///  and delegates its
///           `seek` to `seek_format` so the start-frame math lives in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AudioSpec, Source, seekFormat } from "./decode";
/// ```
use crate::decode::{AudioSpec, Source, seek_format};

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
///  `u32` matches `AudioSpec.rate`
///           and the opus API;
///  the `_` in `48_000` is a digit separator (purely cosmetic).
/// Why:
///       Opus is defined to output 48 kHz;
///  naming it avoids a magic number.
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
///  `usize` because it sizes a buffer (memory length),
///           which is what `usize` is for.
/// Why:
///       We pre-allocate a scratch buffer big enough for any packet so `decode_float`
///           never overflows it.
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
///  `usize` to compare against the `usize` channel
///           count without a cast.
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
///  The live Opus decode state.
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
    ///  Owning,
    ///  heap,
    ///  type-erased Ogg demuxer
    ///           (single owner;
    ///  not the shared `Rc`/`Arc`).
    /// Why:
    ///       Source of raw Opus packets.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// format: FormatReader;
    /// ```
    format: Box<dyn FormatReader>,
    /// What:
    ///      `decoder: opus::Decoder`.
    ///  The libopus decoder VALUE (owned by this struct,
    ///           not boxed:
    ///  it is a concrete type,
    ///  not a trait object).
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
    ///  Id of the Opus track (matches `u32` symphonia ids).
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
    ///  Channel count kept as `usize` (1 or 2) for buffer math
    ///           (frames * channels) without casts.
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
    ///  Cached rate(=48000)/channels/duration.
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
    ///  A reusable owned f32 buffer libopus writes into each
    ///           call (sized `MAX_FRAMES_PER_CHANNEL * channels`).
    /// Why:
    ///       Avoid allocating a fresh buffer per packet.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scratch: Float32Array;
    /// ```
    scratch: Vec<f32>,
    /// What:
    ///      `pre_skip: usize`.
    ///  Remaining encoder-delay frames-per-channel to discard
    ///           at the very start (Opus prepends silence/priming).
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
///  Inherent methods (the constructor).
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
    ///           Takes ownership of the demuxer and the owned `Track`;
    ///  builds a libopus
    ///           decoder.
    ///  `Self` is `OpusSource`.
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
        //           Select the audio codec params out of the optional `CodecParameters`
        //           enum (`.as_ref()` borrows the `Option`, `.and_then(|cp| cp.audio())`
        //           picks the audio variant). `.ok_or_else(...)` turns `None` into our
        //           error; `?` unwraps.
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
                return PlayerError::Unsupported("opus: no audio codec parameters".to_string())
            })?;

        // What:     `let channels = match &audio_params.channels { ... };`. Pattern-match
        //           the borrowed `&Option<Channels>` to a numeric count, erroring if
        //           absent. Matching on `&...` keeps `c` a `&Channels`.
        // Why:      We must know the layout to configure libopus.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const channels = audio.channels?.count();
        // if (channels == null) throw new PlayerError.Unsupported("opus: unknown channels");
        // ```
        let channels = match &audio_params.channels {
            // What:     `Some(c) => c.count()`. Present: `.count()` -> `usize`.
            // Why:      Real channel count.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // channels = c.count();
            // ```
            Some(c) => c.count(),
            // What:     `None => return Err(PlayerError::Unsupported(...))`. Missing layout:
            //           `.to_string()` makes an owned message; return early.
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

        // What:     `let opus_channels = match channels { ... };`. Map the numeric count to
        //           libopus's `Channels` enum, rejecting anything else.
        // Why:      libopus's API takes the enum, not a number.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const opusChannels = channels === 1 ? Channels.Mono
        //   : channels === 2 ? Channels.Stereo
        //   : (() => { throw new PlayerError.Unsupported(`opus: ${channels} channels`); })();
        // ```
        let opus_channels = match channels {
            // What:     `MONO => opus::Channels::Mono`. The `Channels::Mono` enum variant
            //           (a unit constructor, value 1).
            // Why:      One channel.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case 1: return Channels.Mono;
            // ```
            MONO => opus::Channels::Mono,
            // What:     `STEREO => opus::Channels::Stereo`. The stereo variant (2).
            // Why:      Two channels.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // case 2: return Channels.Stereo;
            // ```
            STEREO => opus::Channels::Stereo,
            // What:     `other => return Err(PlayerError::Unsupported(format!(...)))`. Bind
            //           the unmatched count to `other`; `format!` builds an owned `String`
            //           with it interpolated (`{other}`); return early.
            // Why:      We only support mono/stereo (no surround) in this toy player.
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
        //           Constructs the libopus decoder at 48 kHz for the layout. `?` converts
        //           an `opus::Error` to `PlayerError` and returns on fail.
        // Why:      The decoder we feed packets to.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const decoder = new OpusDecoder(OPUS_RATE, opusChannels);
        // ```
        let decoder = opus::Decoder::new(OPUS_RATE, opus_channels)?;

        // What:     `let pre_skip = track.delay.unwrap_or(0) as usize;`. In 0.6 the pre-skip
        //           lives on `Track::delay` (`Option<u32>`; the Ogg mapper read it from
        //           `OpusHead`), not on the codec params as in 0.5. `.unwrap_or(0)` defaults
        //           to 0; `as usize` widens to the buffer-math type.
        // Why:      Number of priming frames-per-channel to discard at the start.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const preSkip = track.delay ?? 0;
        // ```
        let pre_skip = track.delay.unwrap_or(0) as usize;

        // What:     `let duration_secs = match track.num_frames { ... };`. Compute seconds
        //           from the total frame count (0.6 `Track::num_frames`; always at 48 kHz
        //           for Opus).
        // Why:      The seek bar needs the track length.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const durationSecs = track.numFrames != null ? track.numFrames / 48000 : 0;
        // ```
        let duration_secs = match track.num_frames {
            // What:     `Some(n) => n as f64 / OPUS_RATE as f64`. Cast both to f64 before
            //           dividing (integer division would truncate).
            // Why:      seconds = frames / 48000.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return n / 48000;
            // ```
            Some(n) => n as f64 / OPUS_RATE as f64,
            // What:     `None => 0.0`. Unknown frame count -> unknown duration.
            // Why:      Avoid guessing.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return 0;
            // ```
            None => 0.0,
        };

        // What:     `let spec = AudioSpec { rate: OPUS_RATE, channels: channels as u16, duration_secs };`.
        //           Build the public spec; `channels as u16` narrows the `usize` count to
        //           the `u16` field; `duration_secs` is field shorthand.
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
        //           `vec![value; count]` macro builds a `Vec<f32>` of `count` copies of
        //           `0.0f32` (the `f32` suffix fixes the float type).
        // Why:      A buffer guaranteed large enough for one decoded Opus packet.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const scratch = new Float32Array(MAX_FRAMES_PER_CHANNEL * channels);
        // ```
        let scratch = vec![0.0f32; MAX_FRAMES_PER_CHANNEL * channels];

        // What:     `Ok(OpusSource { format, decoder, track_id, channels, spec, scratch, pre_skip })`.
        //           Build the struct (field shorthand for same-named locals) and wrap in
        //           `Ok`. Tail -> return.
        // Why:      Return the ready Opus source.
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
///  Implements the shared interface.
/// Why:
///       So `open()` can return it as `Box<dyn Source>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // OpusSource implements the Source interface: spec(), next_chunk(), seek()
/// ```
impl Source for OpusSource {
    /// What:
    ///      `fn spec(&self) -> AudioSpec { self.spec }`.
    ///  Returns a copy of the cached
    ///           spec (`AudioSpec` is `Copy`).
    /// Why:
    ///       Report the stream shape.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spec(): AudioSpec { return this.spec; }
    /// ```
    fn spec(&self) -> AudioSpec {
        // What:     `self.spec` tail expression -> returned BY VALUE (copy, since
        //           `AudioSpec` derives `Copy`).
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
    ///  Pulls the next
    ///           Opus packet and decodes it to interleaved f32.
    ///  `&mut self` because decoding
    ///           advances the demuxer and decoder.
    /// Why:
    ///       Produce the next PCM block (or EOF).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextChunk(): number[] { ... }
    /// ```
    fn next_chunk(&mut self) -> Result<Vec<f32>, PlayerError> {
        // What:     `loop { ... }`. Repeat because some packets are other tracks or get
        //           fully consumed by pre-skip and yield nothing.
        // Why:      Keep going until we have audible samples or hit EOF.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (true) { ... }
        // ```
        loop {
            // What:     `let packet = match self.format.next_packet() { ... };`. In 0.6
            //           `next_packet` returns `Result<Option<Packet>>`: `Ok(Some(p))` is a
            //           packet, `Ok(None)` is clean end-of-stream (0.5 signalled EOF via an
            //           `UnexpectedEof` IoError).
            // Why:      Get a raw Opus packet, handling end-of-stream.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const p = format.nextPacket(); // null at EOF
            // ```
            let packet = match self.format.next_packet() {
                // What:     `Ok(Some(p)) => p`. A packet was produced; unwrap it.
                // Why:      We have something to decode.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // packet = p;
                // ```
                Ok(Some(p)) => p,
                // What:     `Ok(None) => return Ok(Vec::new())`. End of stream -> empty Vec
                //           (our signal).
                // Why:      End of file -> stop cleanly.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return [];
                // ```
                Ok(None) => return Ok(Vec::new()),
                // What:     `Err(Error::ResetRequired) => return Ok(Vec::new())`. Treat
                //           reset-required as end-of-track.
                // Why:      Simple player: end instead of resetting.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (e.kind === "ResetRequired") return [];
                // ```
                Err(Error::ResetRequired) => return Ok(Vec::new()),
                // What:     `Err(e) => return Err(e.into())`. Other errors: `.into()`
                //           converts via the `From` impl to `PlayerError` and propagates.
                // Why:      Surface genuine demux failures.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // throw toPlayerError(e);
                // ```
                Err(e) => return Err(e.into()),
            };

            // What:     `if packet.track_id != self.track_id { continue; }`. Skip packets
            //           that are not our Opus track. In 0.6 `track_id` is a public FIELD
            //           (the 0.5 `track_id()` getter was removed).
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
            //           `packet.data` is the raw Opus bytes (`Box<[u8]>`; 0.6 made it a
            //           public field, replacing the 0.5 `packet.buf()` getter); `&packet.data`
            //           deref-coerces to the `&[u8]` libopus wants. `&mut self.scratch` lends
            //           the output buffer mutably; `false` = "this is not
            //           forward-error-correction recovery". Returns frames PER CHANNEL
            //           (`usize`). `?` converts an `opus::Error` to `PlayerError`.
            // Why:      Decode one packet into the scratch buffer.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const frames = decoder.decodeFloat(packet.data, scratch, false);
            // ```
            let frames = self
                .decoder
                .decode_float(&packet.data, &mut self.scratch, false)?;

            // What:     `let total = frames * self.channels;`. Total INTERLEAVED sample
            //           count (frames-per-channel times channels).
            // Why:      That many leading entries of `scratch` are valid this call.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const total = frames * this.channels;
            // ```
            let total = frames * self.channels;

            // What:     `let drop_frames = self.pre_skip.min(frames);`. `.min(frames)`
            //           returns the smaller of the remaining pre-skip and this packet's
            //           frame count, so we never drop more than we have.
            // Why:      Discard priming frames spread across the first packet(s).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const dropFrames = Math.min(this.preSkip, frames);
            // ```
            let drop_frames = self.pre_skip.min(frames);

            // What:     `self.pre_skip -= drop_frames;`. Subtract what we are about to drop
            //           from the remaining pre-skip counter.
            // Why:      Track how much priming is left for later packets.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.preSkip -= dropFrames;
            // ```
            self.pre_skip -= drop_frames;

            // What:     `let start = drop_frames * self.channels;`. The interleaved offset
            //           where audible samples begin (after dropped frames).
            // Why:      Slice past the priming samples.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const start = dropFrames * this.channels;
            // ```
            let start = drop_frames * self.channels;

            // What:     `let samples = &self.scratch[start..total];`. A BORROWED sub-slice
            //           (`&...[a..b]`) of the scratch buffer covering the audible
            //           interleaved range.
            // Why:      The portion we actually return.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const samples = scratch.subarray(start, total);
            // ```
            let samples = &self.scratch[start..total];

            // What:     `if samples.is_empty() { continue; }`. If this packet was entirely
            //           pre-skip (or empty), get the next one.
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
            //           slice into an owned `Vec<f32>`; `Ok(...)` wraps it.
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
    ///  Jump the demuxer
    ///           to a time and clear decoder state.
    /// Why:
    ///       Implement seeking for Opus.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seek(secs: number): void { ... }
    /// ```
    fn seek(&mut self, secs: f64) -> Result<(), PlayerError> {
        // What:     `seek_format(self.format.as_mut(), self.track_id, secs)?`. Call the
        //           shared helper from `decode.rs`. `self.format` is a
        //           `Box<dyn FormatReader>` (an owned, heap Ogg demuxer); `.as_mut()`
        //           reborrows it as `&mut dyn FormatReader` so the helper can seek it
        //           without taking ownership. `?` unwraps the `Ok` or returns the error.
        // Why:      Opus streams begin at a non-zero frame (the pre-skip becomes the track's
        //           `start_ts`), so the old `SeekTo::Time { time: 0s }` mapped to frame 0 and
        //           was rejected as out-of-range whenever the bar was dragged to the
        //           beginning. The helper adds `start_ts`, so second 0 lands on the first
        //           audible frame.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // seekFormat(this.format, this.trackId, secs);
        // ```
        seek_format(self.format.as_mut(), self.track_id, secs)?;

        // What:     `self.decoder.reset_state()?;`. Clears libopus's internal state so
        //           post-seek output has no leftover from before. Returns `Result<()>`; `?`
        //           converts/propagates an `opus::Error`.
        // Why:      Avoid stale samples / artifacts after a seek.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.decoder.resetState();
        // ```
        self.decoder.reset_state()?;

        // What:     `self.pre_skip = 0;`. After seeking we are mid-stream, so there is no
        //           encoder priming left to discard.
        // Why:      Pre-skip only applies at the very start of the stream.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.preSkip = 0;
        // ```
        self.pre_skip = 0;

        // What:     `Ok(())`. Success with the unit value. Tail -> return.
        // Why:      Seek done.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return;
        // ```
        return Ok(())
    }
}
