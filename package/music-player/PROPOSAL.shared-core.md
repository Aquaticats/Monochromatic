# Music player: one Rust core, generated platform apps

A proposal to collapse the desktop and Android music players onto a single shared Rust core,
then use that work to seed a published,
 reusable Rust package that builds one UI codebase into
both a Slint desktop app and a Jetpack Compose Android app.

## Status and audience

Proposal and design document,
 dated 2026-06-21,
 refined 2026-07-03 with the slopo drift measurements.
 Not yet implemented.

Written to be passed around inside and outside the team.
 It assumes familiarity with Rust and
mobile or desktop app structure,
 but not with this repository.
 Internal references use relative
links so a reader with the repo checked out can follow them.

## The endgame (north star)

The ambition is larger than this one app.
 The endgame is a published,
 reusable Rust package
(working name "slint-or-jetpack") that **any Rust app can import to build one UI codebase into two
native UIs**:
 a Slint desktop app and an Android Jetpack Compose app.
 You describe the UI and wire
it to your Rust logic once;
 the package's build step produces both the desktop Slint app and the
Android Jetpack Compose app,
 each idiomatic on its platform.

The music player is the first consumer and the proving ground,
 not the published product.
 Its two
apps already share most of their behavior,
 so refactoring it onto a shared core and then onto this
UI layer is what designs the package's API,
 surfaces the hard cases (a real audio engine,
 real
platform integration,
 real persistence),
 and shows whether "write the UI once in Rust,
 get Slint
and Compose" holds up under a non-trivial app rather than a demo.

Two distinct things come out of this work:

- `music-player-core`:
   the music player's own shared domain logic (decode,
   normalization,
   queue,
  pagination,
   session,
   engine).
   An in-repo crate the app owns.
   **Not** the published artifact.

- "slint-or-jetpack":
   the general cross-platform UI package,
   factored out of what the two
  music-player adapters end up having in common.
   **This** is the artifact that graduates into a
  published,
   versioned package other Rust apps depend on.

This is the direction,
 not a finished design.
 The central technical bet,
 that one UI description
can render as idiomatic Slint on the desktop and idiomatic Jetpack Compose on Android,
 is also the
hardest part and the main open question.
 The staged plan below deliberately builds the shared core
and thin per-platform adapters first,
 so the UI package has a real app to be extracted from and
proven against before anyone else depends on it.

## Why: the same player, built twice

Today `package/music-player` ships the same player implemented twice.

- `desktop-app/` is Rust plus Slint.
   Its `src/` holds the full domain in Rust:
   decode,
   opus,
  true-peak,
   fingerprint,
   queue,
   pagination,
   relpath,
   session,
   plus a PipeWire and cpal engine.

- `android-app/` is Kotlin plus Jetpack Compose over a Rust library reached through hand-written
  JNI.
   The Rust side re-implements decode,
   opus,
   true-peak,
   fingerprint,
   and the engine;
   the
  Kotlin side **separately** re-implements queue,
   pagination,
   relpath,
   session,
   shuffle,
   and
  normalization in `core/*.kt`.

The duplication is real and already drifting.
 The two Rust `truepeak.rs` files differ by
+460/-251 lines (Android added a windowed scan for tracks over 90 seconds that desktop lacks),
`opus.rs` by +229/-133,
 and `error.rs` by +219/-144.
 The pure logic lives in two languages with
no shared source of truth,
 so a fix or a tweak has to be made,
 reviewed,
 and tested twice,
 and the
two copies quietly diverge between those moments.

The original Android port decision (`../../doc/decision/music-player-android-port.md`,
 2026-06-12)
already planned the fix:
 ship the whole engine,
 including queue,
 pagination,
 relpath,
 and session,
as a library reached through UniFFI,
 with Kotlin doing only Compose UI,
 audio output,
 and file
access.
 A later audit (`android-app/kotlin-rust-boundary.md`) reversed that and kept the pure logic
in Kotlin for simplicity.
 This proposal returns to the original direction and pushes past it.

## The drift is measured now

Since this proposal was first written,
the repository runs slopo,
an embedding-based duplicate-code detector configured in `../../slopo.conf.yaml`,
across the whole monorepo.
It groups near-identical code into clusters and reports this player's duplication on every run,
which turns the drift from a one-time observation into a standing,
 measured signal.

The clusters for these two apps are kept visible on purpose.
Every other by-design near-duplicate in the repo is dismissed in `../../slopo.ignore.txt`,
but the music-player android and desktop pairs are deliberately left out of that list,
so they keep surfacing in the report.
A comment in that file records why and points back to this proposal:
do not dismiss them until the Android core is rewritten in Rust and the divergence is gone.
The detector keeps surfacing exactly the debt this proposal exists to remove.

The clusters measured on 2026-07-03 fall into the same two tracks the staged plan already separates.

- Same-language Rust,
   re-implemented between `android-app/rust/src` and `desktop-app/src`.
   This is the audio DSP core,
   the stage-2 scope:
   `decode`,
   `opus`,
   `error`,
   `truepeak`,
   the `engine` worker loop,
   and the peak-cache service (`android-app/rust/src/service.rs` against
  `desktop-app/src/peakcache_service.rs` and `desktop-app/src/peakcache_handle.rs`).
   A shared crate can absorb these directly,
   because both copies are already Rust.

- Cross-language,
   re-implemented between `android-app/app/src/main/kotlin/.../core` in Kotlin and
  `desktop-app/src` in Rust.
   This is the pure logic,
   the stage-3 and stage-4 scope:
   `Queue`,
   `Pagination`,
   `RelPath`,
   `AudioExtensions`,
   `Normalization`,
   and `PlayerController`,
   each paired with its `desktop-app/src` counterpart.
   `Normalization.kt` still duplicates gain logic that already lives in the `truepeak-core` crate,
   so even the one module where extraction has started has not retired its Kotlin copy.

A cross-language pair cannot be de-duplicated by extracting a shared function,
because the two copies are in different languages.
Its only real remediation is this proposal's rewrite,
not a shared helper,
which is why those clusters stay visible in the report rather than being dismissed as by-design.

## Target architecture

```text
package/music-player/
  core/                         shared library: all platform-agnostic logic, published over time
  desktop-app/                  thin adapter: Slint UI + PipeWire/cpal output + filesystem source
  android-app/
    rust/                       thin adapter: UniFFI facade + AAudio output, depends on the core
    app/                        thin adapter: Compose UI + MediaSession + SAF/MediaStore
```

Within the music player,
 the core holds the behavior.
 Each app is a thin adapter that supplies a
platform UI,
 an audio output backend,
 file access,
 and platform integration,
 and otherwise defers
to the core.
 These two adapters are also the raw material the "slint-or-jetpack" package is later
factored out of.

What the core owns (moves out of both apps):

- Audio:
   `decode` (the `Source` trait plus the symphonia and opus implementations and the shared
  `open_media_source` tail),
   `opus`,
   `truepeak`,
   `error`,
   `fingerprint`.
- Pure logic:
   `queue`,
   `pagination`,
   `page`,
   `relpath`,
   `displaypath`,
   `shuffle_mode`,
   `session`
  model,
   `normalization` gain formula,
   `audio_extensions` allowlist and predicate.
- Engine:
   the decode to ring-buffer to gain worker loop,
   the atomic control block,
   and the command
  channel,
   all behind an `Output` trait.
- Persistence:
   a Turso-backed peak cache and session store,
   plus the snapshot type the UI reads.

What stays in each app (the thin adapter):

- Desktop:
   the Slint UI,
   the PipeWire and cpal output implementations,
   the KDE and taskbar
  integration,
   the filesystem watcher,
   the command-line entry,
   and platform thread scheduling.
- Android Rust:
   the UniFFI facade that replaces hand-written JNI,
   and the AAudio output.
- Android Kotlin:
   Compose UI,
   the `MediaSessionService`,
   permissions,
   the SAF persistable
  permission grant,
   the SAF and MediaStore enumeration that hands display strings and content URIs
  to the core,
   the WorkManager peak-sweep scheduler,
   and audio focus handling.

## Key decisions

- **Move the pure UI-driving logic into Rust.
  ** Queue,
   pagination,
   relpath,
   session,
   shuffle,
   and
  normalization become Rust in the core;
   the Kotlin re-implementations are deleted.
   This is the
  single source of truth that ends the drift.

- **Bind Kotlin to Rust with UniFFI.
  ** Generated,
   type-safe Kotlin bindings carry the structured
  snapshots (page lists,
   track lists,
   enums,
   session) across the boundary,
   replacing hand-written
  JNI marshaling.
   This was the original port plan and is the foundation of the generation endgame.

- **Unify the playback engine behind an `Output` trait.
  ** The decode-to-output worker becomes
  shared;
   each platform implements only the output backend.

- **Move both apps' persistence to Turso.
  ** One storage layer in the core,
   synchronous
  (`turso_core`,
   which also lets the desktop drop its async runtime),
   replaces the desktop's JSON
  session plus Turso cache and Android's SharedPreferences plus JSON cache.

## Design seams

- **decode opener split.
  ** `core::decode::open_media_source(Box<dyn MediaSource>, Hint)` is the
  shared tail (Android already factored this).
   Each app keeps a thin opener:
   desktop from a path,
  Android from a borrowed file descriptor opened off a `content://` URI.

- **Output trait.
  ** `trait Output { fn reconfigure(&mut self, spec, playing) -> producer;
  fn set_playing(&mut self, playing); }`.
   Desktop's PipeWire and cpal output and Android's AAudio
  stream each implement it;
   the shared worker drives it.
   Today neither side abstracts output.

- **UniFFI across the crate boundary,
   confirmed before relied upon.
  ** UniFFI generally needs a
  type's own crate to carry its record and enum derives,
   so the core carries feature-gated derives
  (off for desktop,
   on for Android) and the Android facade is generated in library mode,
   which
  collects exported types across every crate linked into the Android shared object.
   This is a known
  multi-crate rough edge and is the first thing the build proves with a small spike;
   the fallback
  is thin wrapper types in the facade,
   which reintroduces some duplication.

- **The UI reads snapshots per action,
   not per frame.
  ** The controller rebuilds an immutable UI
  snapshot only on a user action or an engine event,
   and Compose observes it.
   So the core returns
  one snapshot per action across the boundary,
   which is the bounded cost that makes moving the
  state into Rust sound rather than a per-frame tax.

## Constraints that must not break

These are correctness contracts,
 not preferences.
 Breaking one breaks both apps.

- **Fingerprint bit-parity.
  ** Both apps call the one `track_fingerprint` function in `truepeak-core`
  (XXH3-64 from `twox-hash` with the default seed 0,
   over a domain tag,
   the length-prefixed path,
   the size,
   and the mtime nanoseconds),
   so identical inputs hash identically by construction.
  The known-answer tests in `truepeak-core/src/fingerprint_tests.rs` pin the digests to the
  xxHash reference C implementation,
   and `NativeBridgeTest` asserts the same constants through the Android JNI entry.
  No `-C target-feature` flag is involved.
  Any change to the tag,
   the seed,
   the layout,
   or the hash re-keys both peak caches and needs a `SCHEMA_VERSION` bump.
  (Until 2026-10-14 this contract was `gxhash` major 3 plus an AES target feature in both
  `.cargo/config.toml` files;
   see `doc/decision/music-player-fingerprint-hash.md`.)

- **True-peak normalization:
   both apps are wrong today;
   replace them with one adaptive algorithm.
  **
  Neither implementation is correct.
   Desktop scans every track in full,
   wasteful for long content.
  Android samples a few windows and multiplies by a magic 1.26 factor,
   which over-attenuates in the
  common case and,
   when a window misses the loudest moment,
   under-attenuates into clipping.
   The
  shared core implements one correct algorithm with no safety-factor fudge.
   The cheap probe only
  ever classifies the track;
   it never sets the gain,
   so the applied gain is always either exact or
  absent,
   never an estimate:
    - Tracks shorter than 60 seconds:
       scan in full,
       compute the exact true peak,
       normalize from it.
    - Tracks 10 minutes or longer:
       do not normalize (gain correction of 0 dB,
       that is,
       play at the
      decoded level);
       full-scanning the longest content is not worth it.
    - Tracks from 60 seconds to under 10 minutes:
      probe four 15-second windows at the beginning plus one-third plus two-thirds plus the end.
      Use that probe only to decide hot or not hot.
      Convert each probed window's peak to dBTP.
      Then compute the classifier values:

      ```text
      spread_db = max_window_dbtp - min_window_dbtp
      headroom_db = 1 + min(3, 0.25 * spread_db)
      hot = max_window_dbtp >= -1 - headroom_db
      ```

  A safe classifier result means no normalization.
  The gain correction is 0 dB.
  A hot classifier result means scanning the remaining unsampled parts.
  Then compute the exact true peak over the whole track and normalize from that exact value.
  This is a behavior change.
  Very long tracks stop being normalized.
  Mid-length tracks are normalized only when the probe flags them.
  It is one agreed,
   correct algorithm rather than a choice between two flawed ones.
  The hot classifier is pinned by the full-library benchmark below.
  It is not a guessed safety factor.

  Benchmark evidence from 2026-06-21:
    - Measured against `~/Seafile/Plain/Music` with the desktop app's Catmull-Rom true-peak meter and
      decoder path.
    - 3954 audio-extension files found.
      3953 measured.
      One AAC-in-MP4 decode path failed with Symphonia's existing `aac too complex` error.
    - 3842 tracks fell in the 60-second to under-10-minute probe band.
      3652 of them truly exceeded the -1 dBTP ceiling.
    - The selected adaptive rule above produced zero false "not hot" results.
      It triggered full scans for 3752 probe-band tracks and skipped full scans for 90 probe-band
      tracks.
    - The best fixed margin rule with zero false skips was 4 dB.
      It full-scanned 3811 probe-band tracks.
      That is 59 more than the adaptive rule.
      Fixed margins of 3 dB and below missed at least one hot track.
    - Scanning every successfully decoded file in full would decode 879779 seconds of audio.
      The selected rule decodes 854070 seconds.
      It saves 25709 decoded-audio seconds.
      That is 2.9% while keeping exact gain whenever it applies gain.

- **The audio-extension allowlist is shared,
   the filesystem walk is not.
  ** Share the extension set
  and the predicate;
   the desktop keeps its recursive filesystem expansion,
   and Android keeps its
  SAF and MediaStore enumeration,
   both filtering through the shared allowlist.

- **The SAF permission grant stays in Kotlin.
  ** The chosen library root identity moves into the
  shared session,
   but the Android persistable-permission token management is a platform concern and
  stays in the adapter.

## Staged plan

Each stage ends with both apps building and their tests green,
 committed before the next.

1.  Scaffold the core crate and prove the UniFFI multi-crate path with a throwaway spike before any
    binding design rests on it.
2.  Migrate the audio DSP core (decode,
     opus,
     error,
     fingerprint),
     the proven duplication and lowest
    risk,
     and replace both true-peak implementations with the one adaptive algorithm above.
     Point
    both apps at the core,
     delete their copies,
     keep each app's thin opener.
3.  Migrate the pure logic (queue,
     pagination,
     page,
     relpath,
     displaypath,
     shuffle,
     session model,
    normalization,
     audio extensions).
     The desktop Rust is the canonical source;
     its tests come
    along.
4.  Stand up the UniFFI facade,
     add the codegen step to the Android build,
     and delete the Kotlin
    re-implementations;
     Compose consumes the generated bindings and the snapshot type.
5.  Unify the engine behind the `Output` trait;
     each app provides only its output backend.
     This is
    the largest reconciliation and may itself be split into sub-steps.
6.  Move persistence to Turso in the core;
     delete the per-app stores.
7.  Documentation,
     linting,
     and user-boundary verification on both platforms.

Beyond stage 7,
 the framework endgame:
 factor out what the desktop Slint adapter and the Android
Compose adapter have in common into the standalone "slint-or-jetpack" package,
 prove its API against
a second consumer,
 and publish it.
 The music player stays its first user.

## Verification

Verification must cross both build boundaries,
 not just compile.

- Core:
   run the full test suite plus the Rust linter (line budget and required documentation) and
  clippy,
   all zero-error.
- Parity:
   a fingerprint test asserting equal inputs hash equally,
   and a synthetic-PCM true-peak
  test,
   both passing against the unified core;
   these guard the two parity contracts above.
- Desktop:
   build,
   then run on a real folder and confirm playback,
   transport,
   seek,
   volume,
   shuffle
  and repeat,
   pagination,
   and session restore at the user boundary.
- Android:
   cross-compile for device and emulator,
   assemble the app,
   confirm the generated Kotlin
  compiles,
   and run unit and on-device tests including real audio,
   background playback,
   and
  lockscreen controls.
- Persistence:
   confirm peak-cache rows survive a hard process kill on both platforms and that a
  cache miss self-heals.

## Open questions and risks

- **UniFFI multi-crate generation.
  ** Library mode collecting types across the core and the facade
  is the load-bearing assumption for the thin Android adapter.
   Proven by spike in stage 1;
   the
  fallback adds some duplication.
- **The true-peak algorithm change.
  ** Ratified by the benchmarked classifier above,
   but it still
  alters playback levels on both platforms and only modestly reduces decoded audio,
   because the local
  library is mostly already hot.
- **One UI description,
   two native UIs.
  ** This is the "slint-or-jetpack" package's central bet and
  its hardest part.
   Slint and Jetpack Compose are different rendering models,
   so a shared UI
  description that produces idiomatic results on both is real research,
   and may land as a shared
  view-model plus per-platform view templates rather than full UI generation.
   The music-player
  refactor is scoped to reach a clean shared core and thin adapters first;
   extracting and publishing
  the UI package is the follow-on it is designed to enable.
- **Publishing.
  ** Choosing a registry,
   an API stability policy,
   and a versioning cadence for the
  published UI package once a second Rust app depends on it alongside the music player.

## Supersedes

This proposal supersedes the earlier `android-app/kotlin-rust-boundary.md` recommendation,
 which
argued for keeping the pure logic re-implemented in Kotlin.
 That document represented one team
member's opinion,
 favoring simplicity;
 the team's direction is that maintainability beats
simplicity.
 See that file's superseded banner for the history.
