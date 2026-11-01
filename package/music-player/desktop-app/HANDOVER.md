# Handover: building the `music-player` app

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Working handover for the in-progress Slint music player.
 Read this top to bottom before continuing;
 it captures every
design decision,
 the build environment,
 what exists,
 and the exact next steps with the API research already done.

## Goal

A native Linux music player:
 Slint UI,
 Wayland + PipeWire only,
 broad codec support.
 Toy/personal scope.
Approved plan:
 `/home/user/.claude/plans/build-a-music-player-frolicking-nebula.md`.

## Resolved decisions (do not relitigate)

- Language:
   Rust.
   Slint binding.
- Audio output:
   native `pipewire-rs` client (crate `pipewire = "0.8"`),
   not cpal/rodio.
  > Superseded 2026-06-03: the crate moved to `pipewire = "0.10"` (`Cargo.toml`, locked at 0.10.1 in `Cargo.lock`).
  > The Linux backend is still the native client; macOS and Windows use `cpal` (`src/output_cpal.rs`).
- Codecs:
   all symphonia 0.5.
  x codecs via `features = ["all"]` (FLAC,
   WAV/PCM,
   MP3,
   Vorbis,
   AAC-LC,
   ALAC,
   ADPCM,
   AIFF),
  plus Opus via the `opus` crate (libopus).
   See "Opus" below.
  > Superseded 2026-06-01: the crate moved to `symphonia = "0.6"` the day it was scaffolded (`Cargo.toml`, locked at
  > 0.6.1 in `Cargo.lock`); its API differs from 0.5, and `src/decode.rs` is written against 0.6.
- Sample rate:
   per-track native rate declared to PipeWire;
   PipeWire resamples to the device.
   No resampler crate.
  Gapless is permanently out of scope (the user was explicit).
- Music source:
   ad-hoc queue (open file/folder,
   it becomes the queue).
   No library/database.
- Transport:
   play/pause,
   seek,
   volume,
   next/prev,
   plus shuffle and repeat (repeat-all and repeat-one).
- Metadata display:
   filename only.
   No tag parsing,
   no album art.
   The seek bar's duration/position come from the
  decoder (frame count / rate),
   which is not tag metadata,
   so the position bar stays.
- Volume:
   applied as a PCM gain multiply before the ring buffer (simple,
   reliable).
   Native PipeWire stream `Props`
  volume is an unverified optional upgrade,
   not the default.
- Time values:
   `f64` seconds across engine + messages (seek/position/duration share a unit,
   derived from exact frame
  counts),
   narrowed to Slint's `float` (f32) only at the property edge.
   Volume is `f32` end to end.
   If you ever want
  the messages to carry `u64` frame counts instead,
   the user is open to it but f64-seconds is the current contract.
- Placement:
   `package/music-player/desktop-app/` (new category,
   parallel to `desktop-daemon`).
   Package/binary name `music-player`,
   lib crate `music_player`.
- File picker:
   `rfd` crate (Wayland via `xdg-desktop-portal`);
   CLI path args also accepted.
- Session persistence:
   save queue paths + current index + position + volume + shuffle/repeat to a JSON file under the
  config dir;
   restore on launch,
   prune tracks whose files moved.
- Cargo workspace:
   none.
   `music-player` is standalone like `package/cli/forbidden-strings`.

## Build environment (important: host must stay clean)

The host is an immutable-style Fedora (`/home` -> `var/home`) and the user refused host package installs
(`pipewire-devel`,
 `opus-devel`,
 `clang` are absent).
 All cargo work runs inside a Fedora podman container.

> Superseded: `opus-devel` is no longer needed anywhere.
> Since 2026-06-08 the `opus` crate (`opus = "0.4"`, locked at 0.4.0) builds libopus 1.6.1 from source through
> `opusic-sys` (locked at 0.7.5), which needs `cmake` and `ninja-build`; the `Containerfile` installs those and no
> `opus-devel`.
> The mise tasks now run cargo on the host when `cargo` and the PipeWire and font `pkg-config` libraries are present,
> and fall back to the container otherwise (see the header of `mise.toml`).

- `Containerfile` builds image `localhost/monochromatic/music-player` (fedora:
  41 + rustup stable toolchain + gcc,
  clang+clang-devel for the libspa-sys bindgen,
   pkgconf,
   pipewire-devel,
   opus-devel (superseded 2026-06-08: replaced by cmake and ninja-build, see the note above),
   and GUI runtime libs for `run`).
  The Rust toolchain is rustup's current stable (rustc 1.96 at last rebuild),
   NOT Fedora's `rust` package (1.91.1):
  the Slint dependency is pinned to a git master revision (1.17.0-dev) for smooth mouse-wheel scrolling
  (slint-ui/slint#11338,
   unreleased;
   the latest release 1.16.1 predates it),
   and 1.17 requires rustc >= 1.92.
  rustup installs the proxies into `/usr/local/bin` (CARGO_HOME=/usr/local during the install) so the run-time
  `music-player-cargo:/cargo` volume does not shadow them;
   RUSTUP_HOME=/rustup holds the toolchain.
   Revert the pin
  to a crates.
  io `version` once a Slint release including #11338 ships.
   See
  `doc/troubleshooting/slint-flickable-smooth-scroll.md`.
  > Superseded: the git pin moved to crates.io Slint 1.17 on 2026-06-24 and to crates.io 1.18.1 on 2026-10-11.
  > `Cargo.toml` and `Cargo.lock` hold 1.18.1, so the revert described above is done.
- `mise.toml` tasks wrap `podman run` (mount package dir at `/work`,
   cargo registry in named volume `music-player-cargo`,
  `--security-opt label=disable`):
   `image`,
   `build`,
   `build:debug`,
   `lint` (cargo check),
   `lint:clippy`,
   `test`,
  `run` (Wayland/PipeWire/D-Bus/DRI passthrough),
   `gen:fixtures` (host ffmpeg).
- The image is already built.
   Rebuild with `mise run //package/music-player/desktop-app:image` if the Containerfile changes.

Gotchas learned:
- Reference the image as `localhost/monochromatic/music-player` (fully-qualified).
   A bare short name triggers
  "short-name resolution enforced but cannot prompt without a TTY" in the non-interactive task context.
- Do NOT pipe `podman build ... | tail`:
   the pipe exit code is `tail`'s,
   which masked a real build failure once.
- Live Wayland (`wayland-0`) + PipeWire + pipewire-pulse are running on the host,
   so end-to-end GUI + audio
  verification via `mise run //package/music-player/desktop-app:run` is possible on this machine.

## What exists now (committed checkpoint)

Build infra:
 `Cargo.toml`,
 `Containerfile`,
 `mise.toml`,
 `build.rs` (slint-build compiles `ui/app.slint`),
`.gitignore`,
 `README.md` placeholder,
 `ui/app.slint` (placeholder window).

Source modules (all carry the mandated `dum-dum-non-ts` TS-explainer comments;
 keep that style,
 it is enforced):
- `src/command.rs`:
   `RepeatMode` enum (Off/All/One,
   serde),
   `Command` (UI -> engine),
   `Update` (engine -> UI).
   Done.
- `src/queue.rs`:
   pure queue model (load order + shuffle order + cursor),
   seedable xorshift PRNG for deterministic
  shuffle,
   methods (set_tracks,
   advance(natural),
   prev,
   play_index,
   set_shuffle,
   set_repeat,
   display_names,
  current_index/path).
   Full unit tests for every branch.
   Done.
- `src/session.rs`:
   `Session` (serde) + `prune_missing` + `load`/`save` under `directories::ProjectDirs`.
   Tests for
  round-trip and pruning.
   Done.
- `src/error.rs`:
   `PlayerError` enum (Io/Decode/Opus/Unsupported/Audio) + Display + Error + From impls for io,
  symphonia,
   opus errors (so `?` works).
   Done.
- `src/lib.rs`:
   declares `command`,
   `queue`,
   `session`,
   `error`.
- `src/main.rs`:
   placeholder that opens the Slint window.
   Real wiring pending.

Status:
 compilation not yet confirmed green,
 but close.
 A `cargo check` (mise `lint`) compiled the entire dependency
tree (slint,
 symphonia,
 pipewire,
 opus,
 all transitive) and failed only on a missing BUILD dep:
`yeslogic-fontconfig-sys` (a Slint font dependency) needs `fontconfig.pc`.
 Fixed by adding `fontconfig-devel` and
`freetype-devel` to the Containerfile;
 the image is being rebuilt.
 The cargo cache volume `music-player-cargo` is now warm,
so the next `lint`/`test` only recompiles from fontconfig-sys onward plus our crate.
 First action next session:
`mise run //package/music-player/desktop-app:image` if not already done,
 then `mise run //package/music-player/desktop-app:lint`
to confirm `command`/`queue`/`session`/`error` compile (high confidence;
 the failure was environmental,
 not source).

## Next steps (in order)

1. Confirm `cargo check` is green;
    fix any path/type issues in `error.rs`.
2. Write `src/opus.rs` and `src/decode.rs` (designs + verified APIs below),
    add `pub mod decode; pub mod opus;` to
   `lib.rs`.
3. `mise run //package/music-player/desktop-app:gen:fixtures` to create per-codec fixtures,
    then run `test`.
4. Write `src/output.rs` (PipeWire ThreadLoop + Stream,
    ringbuf consumer in the process callback,
    per-track
   reconnect at native rate/channels) and `src/engine.rs` (controller thread:
    command loop owning queue + current
   `Box<dyn Source>`,
    decode -> ringbuf with volume gain,
    position/state Updates).
5. Build the real `ui/app.slint` and wire `src/main.rs` (callbacks -> Commands via mpsc,
    Updates ->
   `slint::invoke_from_event_loop`,
    `rfd` open + CLI args,
    session save on quit).
6. Verify on Wayland (run task),
    then clippy clean (`lint:clippy`,
    `-D warnings`).
    Write
   `doc/troubleshooting/opus-symphonia-libopus.md` per the troubleshooting-doc rule.

## Verified API facts (researched this session; do not re-research)

> Superseded: the symphonia facts in this section describe 0.5.4, which the app left on 2026-06-01.
> The shipped crate is 0.6.1 and `src/decode.rs` uses the 0.6 API, so read that file for the calls in use.
> The `opus` crate facts stay current for `opus = "0.4"`.

Sources cloned at `/tmp/agent/symphonia-20260601` (checked out tag `v0.5.4`) and `/tmp/agent/opus-rs-20260601`.

- `symphonia-codec-opus` on master is an EMPTY 1-byte stub (no implementation);
   that is why we use the `opus` crate.
- We pin stable `symphonia = "0.5"` (0.5.4) and use its 0.5.
  x API,
   NOT the master/0.6 API (the master examples differ).
  Superseded 2026-06-01: the app uses `symphonia = "0.6"` (0.6.1 locked) and its 0.6 API.
- symphonia 0.5.4 decode loop:
  - `let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?; let mut format = probed.format;`
  - find track:
     `format.tracks().iter().find(|t| t.codec_params.codec != CODEC_TYPE_NULL)`.
  - `CodecParameters` derives `Clone` (clone it to end the `format` borrow before moving `format` into the source).
  - decoder:
     `symphonia::default::get_codecs().make(&params, &DecoderOptions::default())?`.
  - `format.next_packet()` returns `Result<Packet>`;
     end of stream is `Err(Error::IoError(e))` with
    `e.kind() == ErrorKind::UnexpectedEof`;
     also handle `Err(Error::ResetRequired)` as end.
     Skip `DecodeError`/`IoError`
    on `decoder.decode()`.
  - `packet.track_id()`,
     `packet.buf()`.
  - interleaved f32:
     `SampleBuffer::<f32>::new(audio_buf.capacity() as u64, *audio_buf.spec())` then
    `buf.copy_interleaved_ref(audio_buf)` then `buf.samples() -> &[f32]`.
     Recreate the SampleBuffer if a later packet's
    `capacity()` exceeds the current one.
     `SampleBuffer::capacity()` exists.
  - codec_params fields:
     `sample_rate: Option<u32>`,
     `channels: Option<Channels>` (`.count()`),
     `n_frames: Option<u64>`,
    `delay: Option<u32>` (Opus pre-skip),
     `codec: CodecType`.
  - duration_secs = n_frames / sample_rate.
  - seek:
     `format.seek(SeekMode::Accurate, SeekTo::Time { time: Time::from(secs), track_id: Some(track_id) })?` then
    `decoder.reset()`.
     `impl From<f64> for Time` exists.
  - `CODEC_TYPE_OPUS = CodecType(0x1005)`.
     `FormatReader` and `Decoder` traits are `: Send`.
- symphonia 0.5.4 Ogg demuxer HAS an Opus mapper (`symphonia-format-ogg/src/mappings/opus.rs`):
   parses `OpusHead`,
  sets `delay` = pre-skip,
   consumes `OpusHead`/`OpusTags`,
   and yields raw Opus audio packets via `next_packet()`.
- opus crate:
   `opus::Decoder::new(sample_rate: u32, channels: opus::Channels) -> Result<Decoder>`;
  `decode_float(&mut self, input: &[u8], output: &mut [f32], fec: bool) -> Result<usize>` returns frames PER CHANNEL;
  `reset_state()`.
   `Channels::Mono = 1`,
   `Channels::Stereo = 2`.
   `Decoder` is `Send`.
   Error type `opus::Error`.
  Max Opus frame is 120ms = 5760 frames/channel at 48kHz;
   size the output buffer `5760 * channels`.

## Designed (not yet written) decode modules

`src/decode.rs` public surface:

```rust
#[derive(Clone, Copy, Debug)]
pub struct AudioSpec { pub rate: u32, pub channels: u16, pub duration_secs: f64 }

pub trait Source: Send {
    fn spec(&self) -> AudioSpec;
    fn next_chunk(&mut self) -> Result<Vec<f32>, PlayerError>; // empty Vec == EOF
    fn seek(&mut self, secs: f64) -> Result<(), PlayerError>;
}

pub fn open(path: &Path) -> Result<Box<dyn Source>, PlayerError>; // probe, clone params, route Opus vs symphonia
pub fn decode_all(path: &Path) -> Result<(AudioSpec, Vec<f32>), PlayerError>; // test helper: drain to end
```

- `SymphoniaSource` (in `decode.rs`):
   holds `Box<dyn FormatReader>`,
   `Box<dyn Decoder>`,
   `track_id`,
  `Option<SampleBuffer<f32>>`,
   `AudioSpec`.
   `next_chunk` runs the decode loop above,
   returns one decoded packet's
  interleaved samples per call.
   `seek` -> `format.seek(...)` + `decoder.reset()`.
- `OpusSource` (in `opus.rs`):
   holds `Box<dyn FormatReader>`,
   `opus::Decoder`,
   `track_id`,
   `channels: usize`,
  `AudioSpec` (rate 48000),
   reusable scratch `Vec<f32>` of `5760*channels`,
   `pre_skip: usize` (= params.
  delay).
   Only
  mono/stereo supported (else `PlayerError::Unsupported`).
   `next_chunk` feeds `packet.buf()` to `decode_float`,
   drops
  the first `pre_skip` frames,
   returns interleaved samples.
   `seek` -> `format.seek(...)` + `decoder.reset_state()` +
  `pre_skip = 0`.
- decode tests:
   one per codec (`tone.wav/flac/mp3/ogg/opus`,
   `tone.aac.m4a`,
   `tone.alac.m4a`) calling `decode_all`
  and asserting rate>0,
   channels>=1,
   non-empty,
   interleaved length % channels == 0.
   Fixtures generated by
  `gen:fixtures`.

## Architecture target (threads)

UI thread (Slint event loop) <-> mpsc `Command` channel <-> engine/controller thread (owns queue + current
`Box<dyn Source>`,
 decodes into a `ringbuf` SPSC producer,
 applies volume gain) -> PipeWire `ThreadLoop` Stream
process callback (RT thread) drains the ringbuf consumer.
 Updates go engine -> UI via `slint::invoke_from_event_loop`.
On track change,
 tear down + recreate the PipeWire stream at the new track's native rate/channels (per-track native
rate;
 no gapless).

## Conventions reminders

- Every non-TS source file needs the `dum-dum-non-ts` comment blocks (What / Why / TS map / pseudocode),
   including
  sibling-justification for numeric type choices.
   This is enforced and the user has already corrected one lapse.
- Run only via mise tasks (which wrap podman);
   never raw cargo on the host.
- Commit eagerly at coherent checkpoints with Conventional Commits,
   scoped pathspec (no `git add -A`/`.`).
