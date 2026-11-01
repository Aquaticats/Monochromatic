# Handover: porting music-player to Android (Jetpack Compose + Kotlin)

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Working state for porting `package/music-player/desktop-app` (Rust + Slint) to Android,
 targeting the connected
Pixel 6.
 Invoked via `/grill-with-docs`.
 The grilling (design interview) is COMPLETE;
 no Android code is written
yet.
 The authoritative decision record is `doc/decision/music-player-android-port.md`;
 read it first.
 This
handover adds the working state,
 measured facts,
 and exact next steps.

## Status

- All product forks are resolved (see the ADR).
   The design tree is fully walked.
- Identity unified to `dev.monochromatic.musicplayer` and committed.
- The user said "Go":
   the build phase has started.
   The derisking milestone (toolchain + scaffold + Media3
  skeleton playing real audio on the Pixel 6) is DONE and committed.
   See "Build progress" below.
- The real Media3 variant is well underway:
   the pure logic is ported to Kotlin (56 tests),
   the real Compose UI is
  built,
   the library reads the device's real MediaStore audio library,
   and the player is now hosted in a
  `MediaSessionService` for background/lockscreen/notification,
   all verified on device.
   The SAF chosen-root source,
  the true-peak `AudioProcessor` (async measure-on-miss + shared cache),
   and the WorkManager charging background
  sweep (`PeakSweepWorker`,
   low-priority decode) have all since landed and are verified on device,
   so the media3
  variant is feature-complete;
   the full-Rust variant has now begun.
   See "Build progress" and "Next steps".
- Full-Rust variant,
   step 1 (native-toolchain derisk,
   Task #10) is DONE and verified on device:
   cargo-ndk produces a
  16KB-aligned arm64 cdylib that the GrapheneOS Pixel 6 loads via `System.loadLibrary` and JNI-calls (`OK (1 test)`).
  GrapheneOS blocks dynamic *code* loading (which killed Slint) but a JNI `.so` from the APK is fine.
   The hybrid
  variant is dropped as a deliverable (owner directive 2026-06-12):
   a Rust meter over Media3-decoded PCM fixes only
  the meter,
   not the decode-bound cost,
   so it cannot move performance enough.
   Step 2 (cross-compile,
   Task #11) is
  also DONE and verified on device:
   libopus (opusic-sys,
   cmake-built from source) and symphonia 0.6 `all` both
  cross-compile for arm64 and run on the device (`OK (3 tests)`:
   a libopus decoder constructs via
  `opus_decoder_create` and the symphonia registry initializes on the arm64 CPU).
   Next is the engine itself
  (Task #12);
   see "Next steps".
- Full-Rust variant:
   the standalone primitives are de-risked on device (toolchain #10,
   libopus + symphonia #11,
  native decode ~10x faster than MediaCodec #15,
   AAudio output 43 ms #16,
   `content://` fd decode #17),
   and the engine
  itself now PLAYS on device (Task #12,
   Milestone 1,
   below):
   the decode -> ring -> AAudio loop,
   the `AudioEngine` JNI
  seam,
   transport (load-fd/play/pause/seek/volume/position/duration/ended),
   and the production auto-advance chain all
  mechanically verified (silent) on the Pixel 6.
   One check is still owed before ranking variants (#13):
   a brief AUDIBLE
  play through the real app (the volume-0 tests prove mechanics and real-time position proves real PCM flows,
   but only
  ears confirm channel order and glitch-free output).
   The sample-rate unknown is RESOLVED:
   the engine opens AAudio at
  `spec.rate` and the on-device test confirmed position tracks real-time (0.494 s in ~0.5 s of playback,
   no 8.8% drift)
  and an accurate mid-track seek.
   The full-Rust variant is now FEATURE-COMPLETE vs Media3 (Milestone 2 done):
   audio
  focus + becoming-noisy (`6ec84c84`),
   and true-peak normalization (`e641f76e`,
   native 4x-oversampled measure
  `7a5e632b` + gain wiring;
   the rust flavor's `measureTrackPeak` sweep seam is now implemented too,
   so its background
  peak sweep works).
   Audible playback is CONFIRMED:
   the user tested the real app and confirmed the full-Rust variant
  plays correctly (the human-ears check the volume-0 tests could not provide;
   channel order / glitch-free output now
  verified by the user).
   The Media3-vs-Rust metric comparison (#13) is DONE and decisive (below):
   rust is ~86-187x
  faster at the same decode + true-peak operation on the same tracks.
   WINNER:
   the full-Rust variant.
  The queue/advance/shuffle stay in Kotlin's `PlayerController` (the native engine is only the per-track primitive).

Dum-dum-non-ts pass (#14,
 DONE):
 the `dum-dum-non-ts` comment skill is now applied across the WHOLE package,
 60 files
(8 Rust + 52 Kotlin),
 in three batches (pilot `1c53482f`,
 batch 1 `be2e0d58`,
 batch 2 `ca6160b7`),
 ~40k lines of
What/Why/TS-map/pseudocode teaching comments.
 Done with a mix of a Workflow fan-out and three `spawn-claude` child
sessions (the workflow rate-limited on the last 25 Kotlin files,
 so those went to spawn-claude).
 Comments only:
 a
per-file comment-stripped diff vs the pre-pass code is empty for all 60,
 all three flavors assemble,
 the unit tests
pass,
 both androidTest source sets compile,
 and the Rust `max-lines` lint stays green (dum-dum comments are excluded
from the budget).

Variant collapse + minSdk floor (#9,
 DONE,
 commit `19f591bb2`):
 the media3 and hybrid build flavors are deleted
(owner directive:
 only the full-Rust variant has acceptable performance).
 The flavor dimension is gone,
 so the app
is now a single variant:
 the `rust` source set folded into `src/main` (RustEngine,
 NativeBridge,
 EngineFactory,
PeakMeasurer,
 plus `jniLibs`),
 the rust + worker instrumented tests folded into `src/androidTest`,
 and the media3
engine,
 the offline `Media3TruePeakDecoder`,
 the ExoPlayer gain processors,
 and the per-flavor factories are removed.
`media3-session` stays (the engine-agnostic session layer that projects to the system notification);
 `media3-exoplayer`
is gone.
 mise tasks collapsed to the single variant (`build`,
 `build:native`,
 `install`,
 `test:unit`,
`test:instrumented`,
 `lint`).
 With media3 deleted,
 the only thing gating the app above the native AAudio hard floor
was `Context.RECEIVER_NOT_EXPORTED` (API 33) in RustEngine;
 routing it through `ContextCompat.registerReceiver` drops
that ceiling,
 so the true floor is `minSdk = 26`.
 Confirmed measured,
 not guessed:
 lint `NewApi` at minSdk 26 reports
zero API-level findings across the whole app,
 the native `.so` is rebuilt at `--platform 26` and its dynamic symbols
import only base API-26 AAudio functions (no setUsage/28 or channelMask/32),
 and it stays 16KB-aligned.
 The previous
`minSdk = 36` was an arbitrary single-target lock,
 not a real dependency.
 THE PORT IS COMPLETE:
 all tasks #4 to #17
are done;
 the app is a single full-Rust-engine variant at the true minSdk floor of 26.
 (On-device re-verification of
the collapsed APK was explicitly waived by the owner;
 the build,
 unit tests,
 instrumented-test assemble,
 and lint all
pass on the collapsed structure.
)

Session redesign features (later session,
 implemented):
 the desktop's source-root session redesign (the three ADRs
`doc/decision/music-player-{session-source-root,jit-shuffle,live-update-rescan}.md`,
 each "applies to both") is now
ported to Android.
 The deferred Session persistence seam (below) is wired,
 and restore auto-correction + live updating
are built.
 Five commits:
 Session model collapsed to selected-track URI + settings + position,
 dropping the
materialized queue and `pruneUnplayable` (the Source Root is NOT persisted;
 `LibrarySource` re-resolves it each
launch);
 just-in-time shuffle in `core/Queue.kt` (history grows as you go,
 `cycleStart`,
 +4 tests);
 `SessionStore`
(SharedPreferences);
 and the controller/service/activity wiring (`PlayerController.restoreLibrary`/`reconcileLibrary`,
keyed by `loadedUri` not index so it survives a rescan;
 `PlaybackService.rescan`/`saveSession`,
 rescan guarded to
no-op while a load is in flight so it cannot cancel the cold-start restore;
 `MainActivity` rescans in
`onServiceConnected` (the reliable foreground hook,
 not `onResume`) and saves in `onStop`).
 Verified to the limit
reachable without a device:
 `core` is unit-tested (`test:unit`,
 the JIT + Session cases) and the whole app +
instrumented source set compile-check and lint clean.
 The RUNTIME behaviors that only a device exercises remain owed
(owner-waived per the collapse note):
 Rescan-does-not-restart-playback,
 Restore reselects + seeks,
 and the ON_RESUME
live update.
 The MediaSession timeline shrinking to `[anchor]` after a reconcile under JIT shuffle is the known
on-device-only consequence of the just-in-time history.

Metric comparison (#13,
 decisive):
 a like-for-like fresh head-to-head,
 the same first 8 MediaStore tracks run through
the same operation (full decode + 4x Catmull-Rom true peak) on each engine:
 `NativeBridgeTest.measureTruePeakOnDevice`
(rust,
 `nativeMeasureTruePeak` = symphonia/libopus) vs `Media3TruePeakDecoderTest.measureLibraryTimingForComparison`
(media3,
 MediaExtractor + MediaCodec).
 Rust did all 8 in 6.7 s total (per track 17-3252 ms);
 media3 took ~12.8 min for
just 6 tracks and had not finished the 8th when stopped,
 e.g. track ...861 took 606 s (10 min) on media3 vs 3.25 s on
rust (187x),
 ...858 17.5 s vs 97 ms (181x),
 ...852 3.46 s vs 29 ms (120x),
 ...796 100.7 s vs 1.16 s (86x).
 This
validates the variant's premise:
 in-process libopus/symphonia avoids the MediaCodec/Codec2 per-buffer binder overhead,
which is catastrophic for the OFFLINE decode-as-fast-as-possible peak sweep (and means lower decode CPU during playback
too;
 MediaCodec only has to keep up with 1x real time during playback,
 so the gap there is smaller).
 Correctness
bonus:
 rust's true peak matched media3 BIT-EXACTLY on the 5 lossless tracks (0.012969971,
 0.005311966,
 0.016937256,
0.05836296,
 0.028305292) and diverged only on the lossy track (rust 1.43 vs media3 1.15,
 expected decoder variance),
an independent confirmation the native meter is correct.
 Caveat:
 this is the offline/sweep decode path;
 a steady-state
playback-CPU comparison was not separately run (the offline number already settles the winner,
 and playback decode is
the same symphonia/libopus path measured here).

## Build progress (this session)

Full-Rust engine,
 first playable (Task #12,
 Milestone 1):
 `bbf1b6a9` (native engine) + `8d310210` (Kotlin seam +
test) build the per-track playback primitive behind `AudioEngine`.
 `engine.rs` is the main-thread handle (an mpsc
`Sender<Command>` + `Arc<Control>` of atomics;
 the value behind the JNI `jlong`).
 `engine_worker.rs` runs the single
worker thread that owns the decode `Source`,
 the SPSC ring producer,
 and the AAudio stream,
 processing Load/Seek/Quit;
the realtime AAudio callback only pops,
 gates on the play flag,
 applies volume,
 zero-fills underruns,
 flags
end-of-track,
 and advances the played-frame counter.
 Load and seek both rebuild the output through `reconfigure_output`
(the desktop's ring-flush mechanism),
 opening AAudio at `spec.rate`.
 `lib.rs` adds the 12 `nativeEngine*` JNI fns;
`RustEngine.kt` resolves the `content://` URI to a borrowed PFD and passes its fd inside `use {}` (native dups
synchronously),
 with a 200 ms main-thread poller turning the pull-based native playing/ended state into
`onPlayingChanged`/`onTrackEnded`.
 Two deliberate improvements over desktop:
 volume in the callback (instant,
 not
1 s-lagged),
 position from frames played (accurate,
 not decode-ahead).
 Mechanically verified on the Pixel 6 via
`RustEngineTest` (`am instrument`,
 volume 0,
 silent,
 no session PLAYING):
 `playsPausesSeeksThroughRustEngine` got
`pos=0.494 s dur=9.929 s playWhenReady=true` after play,
 position froze exactly while paused (`a=b=0.4965`),
 and a seek
to `dur/2=4.96 s` resumed to `5.33 s` (real-time position,
 no 8.8% drift,
 confirms the stream opened at the track rate);
`autoAdvancesOnceOnNaturalEnd` drives the production chain (`PlayerController` over `RustEngine`,
 a track seeked near its
end) and saw a single natural-end advance from track `...852` to `...855` (scope index 1).
 Advisor fixes in `98fcbc70`:
`RustEngine.load` no longer eagerly resets `endedHandled` (that reopened a double-advance window;
 the falling-edge rearm
in `poll()` is correct),
 and `handle_load` sets the play gate before opening the stream.
 Verified that ndk 0.9's
`AudioStream::Drop` calls `AAudioStream_close` (`audio.rs`:
1413),
 so `reconfigure_output` dropping the stream is a clean
stop,
 no leak.
 STILL OWED before #13 ranking:
 one brief AUDIBLE play through the real app (channel order,
 glitch-free).
Milestone 2 (Media3 parity,
 not needed to measure decode/output):
 audio focus,
 becoming-noisy,
 true-peak normalization gain.

Milestone 1,
 the derisk,
 is complete and verified at the user boundary:

- `929789f1e` feat(music-player-android):
   scaffold.
   `package/music-player/android-app/` is a Gradle island (AGP
  9.2.1 / Gradle 9.5.1 / Kotlin 2.2.10 / Compose BOM 2026.05.01 / Media3 1.10.1),
   one `:app` with three product
  flavors on the `engine` dimension (`media3`,
   `hybrid`,
   `rust`),
   each with its own `applicationIdSuffix`
  (`.media3` etc.) so all three install side by side.
   The engine sits behind an `AudioEngine` interface;
   each
  flavor supplies its own `createAudioEngine` factory.
   `media3` is the real Media3/ExoPlayer engine;
   `hybrid` and
  `rust` are throwing stubs so the flavor architecture compiles before the NDK work.
   All three flavors build green.
  mise tasks (`build:media3`,
   `build`,
   `install:media3`,
   `lint`,
   `clean`) shell to the committed Gradle wrapper.
- On-device verification (Pixel 6,
   flock-guarded):
   the `media3` APK installs,
   launches,
   renders the Compose UI,
  lists files,
   and PLAYS.
   Tapping a real Opus and a real FLAC drove ExoPlayer through the platform Codec2 decoders
  (`c2.android.opus.decoder`,
   `c2.android.flac.decoder`) to a live `AudioTrack`.
   Confirmed finding:
   no
  `media3-decoder-*` dependency is needed;
   the platform decodes Opus and FLAC.
   The ADR codec section is corrected
  to match.
- Measured already:
   APK sizes media3 14.97 MB vs the stub flavors 11.53 MB,
   so Media3 + ExoPlayer adds ~3.4 MB.
- AGP 9 gotchas hit and fixed:
   `core-ktx 1.19.0` (and the current Compose BOM) require `compileSdk 37`,
   so AGP 9
  rejects compiling against 36 (the exact rejection the runtime vet documented);
   bumped to compileSdk 37,
   targetSdk
  stays 36.
   `BuildConfig` is off by default in AGP 9;
   enabled `buildFeatures { buildConfig = true }`.
- Getting test audio onto the device:
   `adb push` into the app's external files dir
  (`/sdcard/Android/data/<appId>/files/`) works on Android 16 / GrapheneOS and creates the dir;
   the skeleton reads
  that dir (no storage permission needed),
   which isolates "does Media3 decode" from "does SAF/MediaStore work".
  Clean-named fixtures staged at `/tmp/agent/musictest/` (test1.
  opus,
   test2.
  flac,
   test3.
  m4a,
   test4.
  mp3).
- Milestone 1.5,
   the pure-logic Kotlin port,
   is DONE (a 6-agent workflow,
   then central verification).
   The
  platform-independent `core` package now holds faithful ports of relpath,
   pagination,
   queue,
   true-peak,
   peak
  cache,
   session,
   and the audio-extension allowlist,
   against a fixed contract (ShuffleMode,
   Page/PageEntry).
   52
  host JVM JUnit tests pass via `mise run //package/music-player/android-app:test:unit` (no device).
   True-peak
  was verified line-for-line against `truepeak.rs`.
   Each port was driven by the Rust module plus its `_tests.rs`
  as the test oracle;
   deferrals are listed under "Integration seams left by the port" below.
- Milestone 2 (in progress),
   the real player UI,
   is DONE and verified on device.
   The debug skeleton is replaced by
  the desktop's narrow single-column layout in Compose (`MainActivity.kt`):
   seek bar,
   volume,
   a wrapping control
  row (3-state shuffle radios,
   prev/play-next,
   repeat-track checkbox),
   the page-tab grid,
   and the selected page's
  track list with tap-to-play (tap plays;
   tap the current row toggles pause).
   New `PlayerController` wires the
  ported `Queue` + `paginate` over the current local-files source and drives the engine,
   following the playing
  track's page;
   `AudioEngine` was expanded to load/play/pause/seek/volume/position + a track-ended callback,
   and
  `Media3Engine` implements it on ExoPlayer.
   Verified:
   tapping a row loaded+played via the platform decoder,
   the
  highlight moved,
   the seek bar advanced (0:24/0:35),
   and Play flipped to Pause.
   Two deliberate platform-idiom
  choices to revisit if the owner wants exact desktop fidelity:
   the controls use Material3 RadioButton/Checkbox/
  Slider/Button (not the desktop's plain-HTML-styled customs),
   and the page tabs use filled vs OutlinedButton (not
  the desktop's primary flag);
   the custom VScrollBar is dropped in favor of LazyColumn's native scroll.
   Earlier
  this milestone,
   a real defect was fixed:
   the skeleton drew under the status bar (edge-to-edge with no insets),
  now handled by `enableEdgeToEdge()` + a Scaffold,
   plus a system-following dark/light theme.
- Milestone 2 (cont.
  ),
   the real library source,
   is DONE and verified on device.
   `MediaStoreSource.query` reads the
  device audio collection filtered to `IS_MUSIC != 0` (SDK-branched collection URI;
   `RELATIVE_PATH` projection
  guarded behind API 29,
   `DATA` fallback below;
   codepoint-sorted by display path via the core's `compareByCodePoint`).
  A new `Track(uri, displayPath)` splits the playback `content://` URI from the display path:
   the display paths feed
  the unchanged ported queue/pagination (so the desktop common-prefix trim and folder grouping apply as-is),
   and
  `PlayerController` keeps a parallel `uris` list it plays by load-order index (`openLibrary` replaces `openTracks`).
  `MainActivity` requests `READ_MEDIA_AUDIO` (33+) / `READ_EXTERNAL_STORAGE` (<=32) behind a permission gate,
   then
  queries on grant.
   The full permission flow is verified on device (revoke -> relaunch -> the auto-launched system
  dialog,
   which on GrapheneOS adds a "Setup Storage Scopes" option beside Allow/Don't allow -> tap Allow -> the
  `granted=true` callback loads the library;
   the no-permission state renders the gate without crashing).
   All three
  flavors (media3/hybrid/rust) still assemble green after this shared-`main` change.
   Verified on device (API 36):
  the real 3617-track library lists with two folder pages (`Plain`,
  `2025MAR26`),
   Unicode display paths intact;
   tapping an opus row loaded the exact `content://` id and played
  (`3LAU Emma Hewitt - Alive Again.opus`,
   duration 3:40,
   position ticking,
   Pause shown);
   a short WAV ended and
  auto-advanced.
   A background research workflow corroborated every recall-prone fact against primary docs
  (content:
  // plays via `ContentDataSource`,
   no custom DataSource;
   audio is all-or-nothing,
   no partial tier;
  `RELATIVE_PATH` carries the documented trailing slash).
   Honest divergence (advisor-flagged):
   MediaStore is the
  device-wide library behind the `IS_MUSIC` heuristic,
   NOT the desktop's single chosen root;
   the SAF chosen-root
  source (next) is what restores that semantic.
- Milestone 3,
   the `MediaSessionService`,
   is DONE and verified at the user boundary on device.
   The engine + queue
  brain moved off the activity into `PlaybackService : MediaSessionService`,
   so audio outlives the activity.
   The
  brain (`PlayerController`) stays the single source of truth,
   projected to a `MediaSession` through a
  `SimpleBasePlayer` subclass (`BrainPlayer`):
   `getState()` reports the queue's current scope in playback order so
  the framework's final Next/Previous matches the queue,
   and the `handle*` commands route back into the brain.
   The
  service registers the session with the notification manager itself (`addSession` in `onCreate`),
   so the system
  notification,
   lockscreen,
   and foreground-on-play work with NO app-side `MediaController`.
   The in-app Compose UI
  binds the service via a private `LocalBinder` (single process) for a direct handle to the same brain:
   it reads
  `uiState` and drives actions as before,
   while the session projects the very same brain to the system (one source
  of truth,
   two views).
   The activity no longer creates or releases the controller (releasing on compose-dispose
  would have killed the background playback this milestone enables);
   the service owns it.
   `media3-session` moved to
  a flavor-agnostic `implementation` (the session layer is engine-independent;
   it carries `SimpleBasePlayer` via
  `media3-common` transitively),
   `media3-exoplayer` stays media3-only.
   Manifest gained the `mediaPlayback`
  foreground-service declaration + `FOREGROUND_SERVICE`/`FOREGROUND_SERVICE_MEDIA_PLAYBACK`/`POST_NOTIFICATIONS`.
  `Queue` gained `playbackOrder`/`cursorPosition`/`moveCursorTo` (4 new tests,
   56 total) so the wrapper maps a
  timeline window index back into the scope;
   core semantics unchanged.
   Verified on device (API 36):
   activity bind ->
  service auto-create -> self-load 3617 tracks;
   tap-to-play through the service-owned brain (`content://` load,
  session PLAYING with the right active item);
   foreground service active as `mediaPlayback`;
   notification posted
  (prev/play/next + title);
   backgrounded (HOME) playback continues;
   screen-off continuity (device Dozing,
   position
  advanced 0 -> 33.8s);
   media-button Next/Prev advance the queue foreground,
   backgrounded,
   and from the lockscreen;
  prev-goback (<3s) and prev-restart (>3s) both correct;
   the lockscreen media widget renders + its pause toggled
  PLAYING -> PAUSED;
   the in-app UI stayed in sync after lockscreen/media-key actions (Pause button + highlighted row
  matched the session);
   and a natural-end auto-advance fired (`track ended; advancing` -> loaded the next track,
  kept playing).
   An audio-focus correctness fix landed alongside (`Media3Engine` now enables `handleAudioFocus` +
  `handleAudioBecomingNoisy`;
   it was missing,
   so a phone call would not have ducked and a headphone unplug would not
  have paused).
   An adversarial review workflow then found and fixed three real defects:
   `BrainPlayer` omitted
  `COMMAND_RELEASE` (so `SimpleBasePlayer.release()` early-returned and the inner ExoPlayer leaked on every destroy);
  `getState()` reported actual `isPlaying` (false during the buffering window) as `playWhenReady`,
   flickering the
  notification icon on every track change (now reports the engine's play intent via `AudioEngine.playWhenReady()`);
  and `handleSeek` discarded `positionMs` for `COMMAND_SEEK_TO_MEDIA_ITEM` (external controllers only).
   Two review
  findings were deferred (see "Deferred from the MediaSessionService review" below).
- Device state to resume from:
   the latest `media3` debug APK (scroll fix `bab7f556`,
   offline true-peak decoder
  `5d89f21d` which is built but NOT yet wired into playback,
   minSdk 36) is installed on the Pixel 6
  (`dev.monochromatic.musicplayer.media3`) with `READ_MEDIA_AUDIO` and `POST_NOTIFICATIONS` granted.
   IMPORTANT:
   the
  persisted SAF grant for `content://com.android.externalstorage.documents/tree/primary:Plain/Music` was WIPED when
  the decoder's `connectedMedia3DebugAndroidTest` run uninstalled the app (AGP uninstalls both APKs after a
  connected test;
   uninstall revokes persisted URI grants),
   so the app currently falls back to the device-wide
  MediaStore (3638 `IS_MUSIC`).
   To restore the owner's Plain/Music library,
   tap the top-bar "Folder" action and
  re-pick `Plain/Music` (the picker needs a human tap;
   the grant + choice are the user's).
   The scroll fix was
  verified on the MediaStore "Plain" page (~3000 rows scroll;
   the tab row scrolls off with the list).
   Playback is
  hosted in `PlaybackService`.
   To exercise the MediaStore path deliberately,
   just do not re-pick a folder.
  Rebuild + reinstall with `mise run //package/music-player/android-app:build:media3` then
  `adb -s 1C171FDF600KWW install -r app/build/outputs/apk/media3/debug/app-media3-debug.apk`;
   re-grant with
  `adb -s 1C171FDF600KWW shell pm grant dev.monochromatic.musicplayer.media3 android.permission.READ_MEDIA_AUDIO`
  (and `... android.permission.POST_NOTIFICATIONS`).
   The seek-bar slider responds to `input tap` at device y=305
  (NOT 292;
   y=292 lands in dead space above it);
   track rows respond around y=1180 + ~140px pitch;
   media-button
  Next/Prev/PlayPause are `adb shell input keyevent 87/88/85` and drive the session.
   Resident-noise rule (user
  directive,
   2026-06-12):
   the Pixel 6 is in a shared space,
   so EVERY on-device playback test must pause immediately
  after confirming playback started (start -> verify `state=PLAYING` via `dumpsys media_session` -> `input keyevent
  85` to pause at once);
   never leave audio running while inspecting state.
   Note:
   many on-device tracks
  under `2025MAR26/` are 30-minute field recordings,
   so a natural-end test needs a near-end seek,
   not a wait.
  `uiautomator dump` works intermittently on this Compose surface (some dumps return empty;
   retry a few times,
   or
  fall back to `screencap`);
   it did yield the page tabs,
   track rows,
   and the "Open" button (the folder action,
  renamed from "Folder" and moved into the control row).
   Owner overlay:
   the device runs the owner's own "fooview"
  floating tool,
   which draws a small circular widget near the bottom-left of every screenshot;
   it is NOT part of
  this app (a `uiautomator dump` of the app shows no such node),
   so ignore it when reading captures.
   Drive taps by
  coordinate (screen is 1080x2400) and read back via logcat (`MusicPlayer:I MediaStoreSource:I SafTreeSource:I
  LibraryRoot:I PlaybackService:I Media3Engine:I Media3TruePeak:I`) and `dumpsys media_session` (shows
  `state=PLAYING`,
   position,
   and the track
  title,
   the most reliable playback check).
   SDK is now mise-provisioned via `ANDROID_HOME` (no `local.properties`);
  see "Build environment:
   self-contained via mise" below.
  adb is flock-guarded on `~/temp/agent/adb-phone.lock` for current runs,
   serial `1C171FDF600KWW`.

## Build environment: self-contained via mise (2026-06-13)

The toolchain is provisioned entirely by mise;
 the former dependency on a machine-local Android SDK under
`/var/tmp` (a reapable path) referenced from a gitignored `local.properties` is gone.

- Root `mise.no-env.toml` `[tools]` owns the `java` (temurin-21),
   `android-sdk` (cmdline-tools,
   left at `latest`),
  and `cargo:cargo-ndk` (4.1.2) tools.
   The package `mise.toml` declares no tools of its own;
   it inherits these.
- The root `prepare:android` task (auto-run by the `prepare` umbrella,
   idempotent) installs the SDK components
  sdkmanager owns and mise cannot:
   `platforms;android-37.0`,
   `build-tools;37.0.0`,
   `ndk;29.0.13846066`,
  `platform-tools`,
   plus the `aarch64-linux-android` rustup target.
   It accepts licenses non-interactively.
- `build:native` derives and exports `ANDROID_NDK_HOME` from `$ANDROID_HOME/ndk`.
   This is REQUIRED:
   opusic-sys
  (bundled libopus) reads `ANDROID_NDK_HOME` to apply the cmake android toolchain;
   without it the libopus build
  silently configures for the host (x86),
   which then fails the aarch64 compile with `-msse unsupported`.
   cargo-ndk
  autodetects the NDK for its own Rust cross-toolchain but does NOT export the var to child build scripts.
- `build:native` also self-heals a poisoned opusic-sys cmake cache:
   a stale build dir configured for the host
  (cached `SSE1_SUPPORTED`) or against a different NDK is purged before building,
   forcing a clean reconfigure.
  CMake never re-evaluates cached compiler checks,
   so without this a single host configure would wedge every later
  aarch64 build.
- Gradle resolves the SDK from `ANDROID_HOME` (set by the android-sdk tool),
   so no `local.properties` is needed.

Verified host-side this session:
 `mise install` + `prepare:android` provision from scratch (including fresh
license acceptance),
 `build:native` cross-compiles a valid aarch64 NEON `.so` (no SIMD disabled),
 the poison-cache
guard purges and rebuilds clean,
 and `build` (assembleDebug) assembles the APK with no `local.properties`.
On-device run was not re-verified this session (no behavioral code changed).

## Committed work (on main)

Pre-build (desktop side):
 `7ea4ad07` unify identity to dev.
monochromatic.
musicplayer (`src/identity.rs`,
`src/peakcache.rs`,
 `src/session.rs`,
 `macos/Info.plist`,
 `README.md`);
 `44b4affe` record the Android port ADR.

Android package (`package/music-player/android-app/`),
 in order:

- `929789f1e` scaffold:
   3-flavor Gradle island,
   all flavors build green.
- `e7ced156` docs:
   milestone 1 + codec-claim correction.
- `683da414` port relpath to the Kotlin `core` + host JUnit harness (`testMedia3DebugUnitTest`).
- `ebb5051a` shared core type contract (ShuffleMode,
   Page/PageEntry).
- `611e03b0` port the pure-logic core (pagination,
   queue,
   true-peak,
   peak cache,
   session,
   audio extensions);
   52 tests.
- `c70cedee` docs:
   core port + integration seams.
- `d855776d` fix:
   edge-to-edge insets,
   Material3 top bar,
   system dark/light theme.
- `c87fa94d` real player UI on the ported queue/pagination (narrow layout,
   tap-to-play),
   verified on device.
- `c90cd858` docs:
   real UI milestone + remaining storage/service work.
- `f22f97d0` read the real library from MediaStore (`Track` split,
   `IS_MUSIC` query,
   permission gate);
   verified on device.
- `bca34483` fix:
   enable audio focus + becoming-noisy on the ExoPlayer (was missing).
- `99fb7b88` host the player in a `MediaSessionService` (`PlaybackService` + `BrainPlayer`/`SimpleBasePlayer`,
   `LocalBinder` UI channel,
   flavor-agnostic `media3-session`,
   FGS manifest,
   `Queue` cursor accessors + 4 tests);
   verified on device.
- `c287dc42` fix:
   release engine on destroy (`COMMAND_RELEASE`),
   notification play-state via `playWhenReady` intent,
   `handleSeek` honors `positionMs` (adversarial-review fixes).
- `fa647b61` add the SAF chosen-folder library source:
   `SafTreeSource` (iterative `DocumentsContract` walk,
   visited-guarded,
   per-directory resilient),
   `LibraryRoot` (persisted grant + held-check),
   `core.DisplayPath` boundary sanitizer + tests,
   source selection + `reloadFromRoot` in the service,
   picker in the activity.
- `1ef469de` fix:
   deliver folder picks via an activity-scoped launcher
   (composition teardown when the picker opened was dropping the result)
   and show a loading notice during scans
   (was flashing "No music found" for the whole scan);
   both found on device.
- `ef90fd1a` fix:
   cancel a superseded library load so a folder re-pick is not overwritten by a slow concurrent self-load.

True-peak foreground normalization (later in this session;
 intervening commits omitted,
 git log is the backstop):
the offline decoder (`Media3TruePeakDecoder`,
 `MediaExtractor`+`MediaCodec`,
 verified against known-peak WAV
fixtures),
 the `GainNormalizationProcessor` + `GainRenderersFactory` pipeline stage,
 the process-singleton
`PeakCacheStore` (atomic JSON at `filesDir/peaks.json`) and `TrackFingerprint`,
 and async measure-on-miss in
`Media3Engine.load` (immediate `playWhenReady`,
 gain resolved in parallel,
 applied under a generation guard).
End-to-end verified on device (cache miss measures,
 persists,
 hits in ~23ms on restart).
 minSdk raised to 36.
Then the performance pass (see "True-peak measure performance"):
 `db6efc2c` bulk PCM conversion (the 24x fix),
`cd64fce2` `maxInteriorAbs` window-term hoist (~20% meter).
 `becdcd16` had reverted the block-the-start design
back to async and added the debug-signed release build + `build:media3:release` task.

Native Rust toolchain (this session):
 `d97ea46e` derisks the cargo-ndk -> JNI path the full-Rust variant needs,
proven on the GrapheneOS Pixel 6 before any engine porting.
 Adds the standalone `rust/` cdylib crate (`jni` 0.21,
pure Rust,
 no C yet),
 a `NativeBridge` facade plus an `androidTestRust` instrumented test asserting a trivial
`nativePing()` across the JNI boundary (`OK (1 test)` via `am instrument`,
 not `connectedAndroidTest`),
 and mise
tasks (`build:rust:native` and the rust assembles that depend on it) that run cargo-ndk behind an `ANDROID_NDK_HOME`
guard.
 The .
so is 16KB-aligned (NDK r29 linker default) and stored uncompressed in the rust APK.
 `23bb5559` removes
the over-broad root `*.lock` ignore (it swallowed Cargo.
lock;
 the build-tool locks stay ignored via directory rules)
and tracks the crate's `Cargo.lock` for reproducible builds.
 Noted to investigate during metrics (Task #13):
 the
rust debug APK is ~36 MB (vs the older media3 ~15 MB figure);
 the 439 KB .
so is not the cause,
 so it is likely
accumulated shared `main` plus debug `ui-tooling`,
 to be confirmed.

Native decode dependencies (this session):
 `256dd194` adds the full-Rust variant's decode crates and proves they
cross-compile and run on the device (Task #11).
 `opus` (opus-rs HEAD,
 opusic-sys backend) builds libopus 1.6.1 from
source via cmake;
 cargo-ndk wires the cmake cross-toolchain automatically (no manual `CMAKE_TOOLCHAIN_FILE`),
 and the
linker keeps only the decoder path (`opus_decoder_create`/celt/silk symbols present,
 encoder dead-stripped).
`symphonia` 0.6 `all` is pure Rust and cross-compiles with no extra toolchain.
 Two self-tests (`nativeOpusSelfTest`
constructs a decoder,
 `nativeSymphoniaSelfTest` initializes the prober + codec registry) pass on device (`OK (3
tests)` via `am instrument`).
 The .
so grew to 3.9 MB with all codecs and stays 16KB-aligned;
 it is kept on
symphonia `all` deliberately (see Resolved decisions).

Native decode port + benchmark (this session):
 `285ce21c` ports the desktop decode path
(`Source`/`open`/`SymphoniaSource`/`OpusSource` + a minimal `PlayerError`,
 desktop module names kept for reuse) into
the native crate and adds `nativeDecodeBenchmark`,
 the full-Rust go/no-go.
 Result:
 native opus decode ~0.032
us/sample vs the Media3 MediaCodec ~0.33 us/sample baseline,
 about an order of magnitude faster (see "Native decode
benchmark").
 Strong GO;
 the engine build (Task #12) is greenlit.
 Light comments only (dum-dum deferred to Task #14).

AAudio output backend (this session):
 `ab24debb` adds the chosen output (`output.rs`,
 raw `ndk::audio`,
 `ndk` 0.9
`audio` feature,
 pure Rust,
 no C/C++ build,
 zero JNI).
 A silent `nativeOutputLatencyProbe` opens a LowLatency
PCM_Float 48k stereo stream with a zero-fill data callback and reads presentation latency from
`AAudioStream_getTimestamp` ((`frames_written - frame_position`)/rate);
 on the Pixel 6 it measured 43.0 ms (tunable
lower via `bufferSizeInFrames`),
 test passes,
 no session PLAYING (inaudible).
 With this,
 the standalone primitives of
the full-Rust variant (toolchain,
 libopus + symphonia decode,
 native decode ~10x faster,
 AAudio output) are de-risked
on device.
 One unknown the isolated probes could NOT surface must still be verified in the engine (Task #12):
sample-rate matching,
 this `output.rs` hardcodes 48k and opus is always 48k so the silent probe matched by luck,
 but
FLAC/MP3 decode at the track's native rate (often 44.1k),
 so the engine must open the AAudio stream at `spec.rate`
(reopening on rate change) or playback runs ~8.8% fast and pitched up,
 a correctness requirement not wiring.
 The
`content://` fd path that was the other unknown is now resolved (#17,
 below).
 The rest is integration:
 the decode ->
output loop (decode on a worker + lock-free ring,
 never decode/alloc in the AAudio callback),
 the Kotlin `AudioEngine`
JNI seam (raw JNI,
 pull-based callbacks),
 and the queue/controller port.
 Optional hardening,
 only if the 10x must be
bulletproof not directional:
 a same-device same-file media3 decode comparison.

content:
// fd decode (this session,
 #17):
 `d5f313e3` adds the fd decode path the engine needs.
 `decode.rs` gains
`open_borrowed_fd(RawFd)`:
 it dups the borrowed Android `ParcelFileDescriptor.getFd()` with std
`BorrowedFd::borrow_raw(fd).try_clone_to_owned()` (F_DUPFD_CLOEXEC,
 no libc dep) and owns ONLY the dup,
 so the JVM
keeps and closes the original PFD.
 This is the load-bearing safety choice:
 calling `File::from_raw_fd` on the borrowed
fd is a DETERMINISTIC fdsan `SIGABRT` on API 30+ (the PFD fdsan-tags its fd;
 bionic `close()` aborts on an owner-tag
mismatch,
 reproduced verbatim in mobile-ffmpeg #634 on a SAF fd),
 whereas the dup is untagged and closes cleanly.
 Two
guards are load-bearing:
 dup SYNCHRONOUSLY inside the JNI call (a late dup on the worker thread hits a fd Kotlin's
`use{}` already closed),
 and `if fd < 0` before `borrow_raw` (it panics on -1 and a panic across `extern "system"`
aborts).
 A `std::fs::File` over a regular-file fd is a seekable `MediaSource` (symphonia `is_file()`),
 so it feeds the
shared `open_media_source` tail directly with an empty `Hint`;
 probe never hard-seeks the source.
 `lib.rs` adds
`nativeDecodeFdBenchmark(fd)` and factors the timing loop into a shared `benchmark_decode`.
 The instrumented test
`NativeBridgeTest.decodeFromContentFd` queries MediaStore (READ_MEDIA_AUDIO granted via `adb pm grant`,
 no SAF picker),
opens one real `content://media/...` track per format via `openFileDescriptor(uri,"r").use { pfd -> ...(pfd.fd) }`,
 and
decodes through the dup path:
 on the GrapheneOS Pixel 6 it measured opus 0.033,
 flac 0.015,
 mp3 0.022 us/sample,
 test
OK,
 no session PLAYING.
 This proves MediaProvider returns a seekable fd,
 symphonia probes/decodes/seeks over it,
 and
the dup protocol does not double-close (any double-close would have been a hard SIGABRT,
 not a soft failure).

SAF caveat (stronger than mere code-identity):
 for a Plain track the SAF document fd is produced by the SAME code this
test already exercised.
 `FileSystemProvider.openFileForRead` on an emulated/visible path (`/storage/emulated/0/Plain`,
exactly where the library lives) delegates via `MediaStore.scanFile` -> `openTypedAssetFileDescriptor` -> MediaProvider
(the on-device-tested path),
 and the only fallback is a `ParcelFileDescriptor.open(target, MODE_READ_ONLY)` regular
file;
 both are seekable.
 The one device-specific danger,
 fd ownership / double-close,
 is provider-independent (the PFD
fdsan-tags its fd in its own constructor regardless of which provider produced it) and is empirically proven by this
test.
 A SAF tree grant cannot be created headlessly (it needs the `ACTION_OPEN_DOCUMENT_TREE` picker;
 no
`pm grant`/`content` path self-grants a tree URI),
 so the SAF fd is confirmed incidentally once the engine plays a
granted folder,
 not skipped.

Engine invariant for #12 (the one place the test and the engine diverge):
 the dup stays read+seek-valid AFTER the
original PFD closes,
 because the two fds share one open file description,
 so the file stays open until both close and
fdsan never touches the untagged dup.
 The test decodes while the PFD is still open;
 the engine will decode on a worker
AFTER Kotlin's `use{}` closes the PFD,
 which is correct precisely because of the shared OFD.
 So #12's `load()` must
keep the dup-backed `Source` alive and must never lazily re-dup or stash the raw borrowed fd.
 Also fold into #12's
verification list:
 this test exercises only `seek(0.0)` (rewind),
 so scrub-to-arbitrary-position must be confirmed
during engine verification (not an fd-derisk gap,
 since the fd is a confirmed-seekable regular file,
 just a checklist
item so it does not fall off).

Background true-peak sweep (this session):
 `1867fda2` adds `PeakSweepWorker` (a WorkManager `CoroutineWorker`,
periodic,
 charging-only),
 `PeakSweepScheduler` (unique periodic,
 `KEEP`-deduped,
 enqueued from `PlaybackService`
when a library becomes available),
 the engine-agnostic `measureAndCache` + `SweepOutcome`,
 the shared
`LibrarySource` enumeration seam (extracted from `PlaybackService` so the sweep and playback fingerprint the same
URIs and the cache actually hits),
 and a per-flavor `measureTrackPeak` seam mirroring `createAudioEngine` (media3
decodes on a `nice 19` thread,
 hybrid/rust stubs throw).
 The offline decoder (`Media3TruePeakDecoder.measure`)
gained an optional `dispatcher` param so the foreground measure-on-miss is unchanged.
 Verified on device via `am
instrument` (not `connectedAndroidTest`,
 which uninstalls and wipes the SAF grant):
 the worker enumerated the real
3638-track library,
 decoded+cached one track (7.2s,
 `audio/raw`),
 and a bounded re-run was a cache-hit skip;
 both
instrumented tests pass.

Note:
 concurrent sessions (an iOS vet) interleave their own commits on `main`;
 those are not part of this work.

## Resolved decisions (do not relitigate; rationale in the ADR)

- UI:
   Jetpack Compose.
   Forced,
   not chosen:
   Slint's Android backend crashes on this GrapheneOS device (dynamic
  code loading via `InMemoryDexClassLoader` is blocked),
   verified on-device by the prior kopia vet.
- Engine:
   build all three variants behind one `AudioEngine` interface:
   pure Kotlin + Media3;
   hybrid (Rust
  true-peak `.so` via UniFFI,
   Media3 plays);
   full Rust reuse (whole engine `.so`).
   Pick by measuring everything on
  device (size,
   cold-start,
   latency,
   battery,
   memory,
   build/FFI complexity,
   maintainability,
   desktop-sharing,
   CI
  robustness,
   and anything else relevant).
   Instrument metrics from the start.
- Distribution:
   plan for Play Store,
   so scoped storage,
   no `MANAGE_EXTERNAL_STORAGE`.
- Storage:
   SAF folder tree (`ACTION_OPEN_DOCUMENT_TREE`) for explicit Open;
   MediaStore (`READ_MEDIA_AUDIO`) for the
  default library.
   MediaStore is rootless,
   so its pagination is rebuilt from `RELATIVE_PATH`,
   not tags.
   Files are
  `content://` URIs;
   Media3 reads them directly;
   the Rust variants open an fd via
  `ContentResolver.openFileDescriptor`.
- Media:
   standard media app.
   `MediaSessionService` hosts the player;
   Compose UI is a `MediaController` client.
  Background playback via `mediaPlayback` foreground service,
   notification,
   lockscreen,
   audio focus,
   headset.
- UX:
   tap-to-play only (tap loads+plays,
   tap the playing row pauses);
   phone-first single column (drop the desktop
  900px two-pane).
- True-peak:
   keep the eager whole-library sweep,
   run it via a WorkManager periodic worker constrained to charging
  only.
   Device-idle was dropped (owner directive 2026-06-12):
   it trips Android Lint's IdleBatteryChargingConstraints
  and would block the sweep while the phone is simply in use.
   Contention is handled instead by decoding at the lowest
  thread priority (`nice 19`,
   the Android analog of the desktop's idle-priority worker),
   so the sweep yields to
  playback while still progressing whenever plugged in.
   The loading track is measured asynchronously on a cache miss
  (a one-time mid-song gain correction).
   The DSP and cache are unchanged.
- Decode (full-Rust variant):
   `symphonia` stays on the `all` feature set;
   do NOT narrow it for size (owner directive
  2026-06-12:
   APK size is not a concern for Android apps).
   Opus is decoded by the `opus` crate (opusic-sys/libopus),
  the one codec symphonia's meta-crate does not wire in.
- FFI binding (full-Rust variant):
   raw JNI,
   not UniFFI (revises the ADR's UniFFI note).
   The `AudioEngine` seam is 11
  methods;
   raw JNI is already proven on GrapheneOS (the smoke test),
   while UniFFI drags in JNA whose GrapheneOS
  behavior is the exact unknown to avoid.
   Callbacks are kept pull-based (the UI already polls position) so no
  native->JVM callback machinery (`AttachCurrentThread`,
   global refs,
   cached method ids) is needed.
   Decided
  2026-06-12 (advisor-endorsed);
   settle the final shape when building the engine surface (Task #12).
- Audio output (full-Rust variant):
   raw `ndk::audio` (the `ndk` 0.9 crate,
   `audio` feature),
   pure-Rust AAudio,
   no
  C/C++ build,
   opens a low-latency output stream (`AudioPerformanceMode::LowLatency`) with zero JNI;
   `libaaudio` is a
  sysroot system lib (`#[link(name = "aaudio")]`),
   not bundled.
   Decided 2026-06-12 AFTER research found cpal and
  AAudio are the SAME engine on this device (cpal 0.16+ wraps `ndk::audio`;
   Oboe also resolves to AAudio at API 27+),
  so a latency bake-off would only show near-parity;
   the owner chose the thinnest pure-Rust path and dropped the cpal
  build (the earlier build-both-and-measure directive is superseded by that finding).
   Latency is still measured on
  device,
   silently:
   write zero-filled buffers and read `AAudioStream_getTimestamp`,
   latency = (framesWritten -
  framePosition) / sampleRate,
   so the resident-noise rule is not engaged until real audio plays.
   Gotcha:
   `ndk`'s
  `AudioStream` is neither `Send` nor `Sync`,
   so wrap it in `Arc<Mutex<...>>` (with a manual `unsafe impl Send`) if it
  crosses threads,
   exactly as cpal does.
   cpal alternative (NOT chosen) would have been 0.18.1 + the `realtime` feature
  (default cpal does not request LowLatency on Android).
- Placement:
   `package/music-player/android-app/` (new category).
   Identity:
   `dev.monochromatic.musicplayer`.
- minSdk 36 (raised from 26 on 2026-06-12 by owner directive:
   single-target app for the owner's Pixel 6,
   so no
  older-release support and modern APIs without compat guards),
   compileSdk 37,
   targetSdk 36,
   JDK 21,
   AGP 9.
  x,
  Gradle 9.
  x,
   latest Compose BOM and Media3.

## Measured hard facts (verified this session; do not re-research)

- Device:
   Pixel 6 (oriole),
   GrapheneOS,
   Android 16 / API 36,
   arm64-v8a,
   security patch 2026-06-01.
   adb serial
  `1C171FDF600KWW`.
   The build fingerprint spoofs stock Google (GrapheneOS behavior).
   Other sessions may attach
  devices:
   guard adb with `flock "${HOME}/temp/agent/adb-phone.lock"` and target `-s 1C171FDF600KWW`.
- Library (host `/home/user/Seafile/Plain/Music`,
   3857 files):
   Opus 2584,
   FLAC 852,
   AAC 16 (m4a/mp4),
   MP3 13,
   plus
  a few Vorbis and Opus-in-webm/mkv.
   No ALAC,
   AIFF,
   or ADPCM.
   Every file is Media3-native.
   The phone now HAS the
  library on-device (this was synced since the prior session):
   MediaStore indexes 3633 audio rows,
   3617 with
  `IS_MUSIC=1`,
   the real music under `relative_path=Plain/Music/` (the synced Seafile Plain/Music) plus
  field-recording WAVs under `2025MAR26/...`;
   the rest is `IS_MUSIC=0` ringtone/notification clutter the source
  filters out.
   So MediaStore verification runs against the real library,
   no pushed fixtures or scan trigger needed.
- Media3 codec coverage (audited from the androidx/media source clone at `/tmp/agent/media3-20260612`,
   may be
  reaped):
   Opus and FLAC via bundled software decoders (`decoder_opus`,
   `decoder_flac`);
   AAC,
   MP3,
   Vorbis via the
  platform;
   WAV IMA-ADPCM in `WavExtractor`;
   ALAC demuxed by the MP4 `BoxParser` but decoding needs the FFmpeg NDK
  extension or a device codec;
   AIFF has no extractor at all.
   `content://` is read via
  `DefaultDataSource` to `ContentDataSource` (`openAssetFileDescriptor`).
   `DefaultAudioSink.Builder.setAudioProcessors`
  and a built-in `GainProcessor` exist (true-peak gain as a custom AudioProcessor is viable).
- `mediaPlayback` foreground service is exempt from the 6h/24h cap (that cap is `dataSync` and `mediaProcessing`
  only),
   so background playback is uncapped.
- Compose instrumented tests must pin `androidx.test >= 3.7.0` / `runner 1.7.0` on Android 15/16,
   or the Compose
  BOM's transitive Espresso 3.5.0 crashes (`InputManager.getInstance` removed).
   Maestro drives black-box E2E via
  `testTag`.

### True-peak measure performance (media3 variant, device-profiled, release build)

Profiled on the Pixel 6 against the real opus library by splitting the measure into stages (decode-only via a
raw drain that sums output bytes;
 decode+convert;
 full decode+convert+meter;
 and an in-memory meter-only
instrumented probe over a 20M-sample `FloatArray`,
 run via `am instrument` against the installed release app so
ART optimizes the DSP).
 All numbers are per-sample so they compare across track lengths;
 a "3.7-minute opus
stereo track" is ~21M samples.

- MediaCodec opus decode (`audio/opus`,
   2ch):
   ~0.33 us/sample (~7s/track,
   ~13x realtime).
   WAV (`audio/raw`)
  decode is trivial (~0.21 us/sample) so WAV was always meter-bound;
   opus is decode-plus-meter bound.
- short->float conversion:
   ~0.06 us/sample (~1.2s).
   This WAS ~1.39 us/sample (~29.5s,
   the single largest cost)
  because `toFloatChunk` did one bounds-checked `ByteBuffer.get(index)` per sample through a lambda.
   Fixed in
  `db6efc2c` by bulk-copying into a `ShortArray` then a primitive loop (24x).
   This was a real bug,
   not physics.
- Catmull-Rom true-peak meter:
   ~0.42 us/sample isolated (in-memory probe),
   ~0.66 us/sample in-context
  (decode-interleaved,
   ~1.5x cache/interleave penalty,
   NOT a little-core throttle).
   Hoisting the window-only
  cubic terms out of the per-sample loop (`maxInteriorAbs`,
   `cd64fce2`) cut it ~20%;
   that 20% proves the cubic
  arithmetic was not the dominant cost,
   so further Kotlin micro-optimization (flatten the window array,
   kill the
  `i % channels` idiv) is diminishing returns.
   The ~0.42 us/sample is real ART scalar-FP cost (no SIMD,
  un-elided bounds checks),
   which native Rust SIMD eliminates.
- Full opus measure:
   ~36.5s+ before,
   ~19s after both fixes for a 3.2-minute track.
   Start latency is unaffected
  (async,
   ~425ms,
   meets the "tap -> playing under 1s" rule);
   the measure runs in the background.
- Do NOT claim native "won't be sub-1s on this device":
   the in-context decode/meter times were measured on the
  background measure thread and are an upper bound,
   not a hardware floor.
   The full-Rust variant (symphonia
  decode + native meter) IS the experiment that determines the true device floor;
   whether portable-Rust
  symphonia beats the platform MediaCodec opus decoder on this CPU is unknown and must be measured there.
- User decisions this session:
   (1) optimize the Kotlin meter now,
   full-Rust variant after the Kotlin variant is
  done;
   (2) on a cache miss,
   apply the gain mid-song when the measure lands (the current async behavior,
   a
  one-time level correction) rather than caching-only-for-next-play.
   So `Media3Engine.load` needs no change.
- Sweep economics:
   ~19s/track x 3638 tracks ~= 19h of decode for a full first sweep,
   spread across charging windows
  (15-min minimum periodic interval,
   ~10-min cap per run,
   resume-via-cache between runs).
   The decode runs at lowest
  thread priority,
   so it is full-speed when the phone is idle and yields under playback;
   the Rust decode cost
  matters here too.
   Known minor cost:
   because the work is periodic-forever,
   every charging window re-runs
  `LibrarySource.load` (a full `SafTreeSource` tree walk,
   or a MediaStore query) even once the library is fully
  cached,
   so a future "wakeups while desk-charging" symptom traces to this re-enumeration;
   it is cheap per run and
  batched by the system,
   not worth optimizing until observed.
- Production scheduling verified on device (`1867fda2`,
   launched 2026-06-12):
   the enqueue path ran without crashing,
  `dumpsys jobscheduler` shows the `#PeakSweepWorker#` job with `charging=true batteryNotLow=false deviceIdle=false`,
  and `WM-WorkerWrapper: Starting work for ...PeakSweepWorker` confirms the real worker executes while charging.
   The
  session stayed `PAUSED` throughout (no audio).
   This path is invisible to `TestListenableWorkerBuilder`,
   which
  bypasses `WorkManager.getInstance()`,
   so it must be checked by launching the app,
   not only by the instrumented test.

### Native decode benchmark (go/no-go, device-measured)

`nativeDecodeBenchmark` (Task #15,
 `285ce21c`) decodes a real library file fully to interleaved f32 PCM on the
device,
 decode-only (no output),
 and reports microseconds per interleaved sample,
 directly comparable to the Media3
MediaCodec figures above.
 Measured on the Pixel 6 against files sourced from the device's `Plain` mirror:

- opus (`Ahrix - Nova.opus`,
   symphonia Ogg demux + libopus):
   0.032 us/sample (~320x realtime).
- flac (a real Ado track,
   symphonia):
   0.020 us/sample.

Versus the recorded Media3 MediaCodec opus decode (~0.33 us/sample),
 native opus decode is about an order of
magnitude faster:
 in-process libopus avoids the Codec2 binder round-trips and MediaCodec buffer plumbing the
framework path pays per buffer.
 The am-instrument total (1.26s) matches the sum of the two decodes,
 so real decode
work happened.
 This greenlights the full-Rust engine on the decode dimension (the native SIMD true-peak meter is
expected to beat the ART meter on top of that).
 Caveat:
 this is a fresh same-file native measure against a recorded
media3 aggregate,
 not a same-session same-file head-to-head;
 a media3 re-measure on `bench.opus` would tighten the
exact ratio but cannot flip an order-of-magnitude gap.

Fixture recipe (silent;
 reuse for re-runs):
 the rust flavor has no storage permission and scoped storage blocks
direct `/sdcard/Plain` File access,
 so copy on-device into the rust app's external files dir and make it app-readable:
`adb shell 'DIR=/sdcard/Android/data/dev.monochromatic.musicplayer.rust/files; mkdir -p "$DIR"; cp "/sdcard/Plain/Music/<file>" "$DIR/bench.opus"; chmod 0666 "$DIR/bench.opus"'`.
 The `chmod` is required:
 a shell-written
file is owned `shell` mode 0660,
 which the app uid cannot read (`open` returns -2);
 `adb push` would instead assign
app ownership.
 The benchmark test reads `bench.opus`/`bench.flac` from `getExternalFilesDir(null)` and skips (not
fails) when absent.

### Resident-noise rule (standing constraint)

The Pixel 6 is in a shared space.
 EVERY on-device playback test MUST pause the instant `dumpsys media_session`
reports `state=PLAYING(3)`,
 via `adb shell input keyevent 127` (the match string is `state=PLAYING`,
 NOT
`state=3`).
 The background measure continues after the pause (it is on a separate coroutine,
 independent of
player state),
 so pausing the audio does not abort the measurement being profiled.

## Build environment (host, as of this session)

Resolved and in use this session (all reused from the prior vets,
 nothing freshly downloaded except API-37 was
attempted;
 these `/var/tmp` paths can be reaped,
 so a future session may need to re-point or re-install):

- Android SDK:
   `sdk.dir=/var/tmp/vet-jc/android-sdk` (set in the gitignored `local.properties`).
   It is the jetpack
  vet's SDK and the only one with `platforms/android-37.0` + `build-tools/37.0.0` already present,
   which
  `compileSdk 37` requires.
   Note:
   the tauri vet SDK (`/var/tmp/tauri-vet-work/android-sdk`) has the NDK but its
  cmdline-tools (rev 12.0) cannot fetch `platforms;android-37` (it only parses repo XML up to v3 and skips the v4
  android-37 entry,
   "Failed to find package");
   the vet-jc SDK already had android-37.0 installed,
   so use it.
- JDK:
   Temurin 21 via mise (`/home/user/.local/share/mise/installs/java/temurin-21.0.11+10.0.LTS`).
   The package
  `mise.toml` pins `java = "temurin-21"`,
   so `mise run //package/music-player/android-app:<task>` sets JAVA_HOME
  automatically;
   Gradle reads the SDK from `local.properties`,
   so no ANDROID_HOME env is needed.
- Gradle:
   the committed wrapper (9.5.1) drives all builds;
   it was generated with the standalone
  `/var/tmp/vet-jc/gradle-9.5.1`.
- `cargo-ndk` 4.1.2 (`/home/user/.cargo/bin/cargo-ndk`,
   via `cargo binstall`;
   invoke as `cargo ndk`,
   never the binary
  directly,
   which refuses by design).
   Proven recipe this session:
   `export
  ANDROID_NDK_HOME=/var/tmp/tauri-vet-work/android-sdk/ndk/29.0.13846066` (an NDK r29-beta3;
   the vet-jc SDK has none)
  then `cargo ndk -t arm64-v8a --platform 35 -o <jniLibs-dir> build --release`.
   Gotchas,
   all paid for already:
   the
  API-level flag is `--platform` / `-P` (capital);
   lowercase `-p` is cargo's `--package`,
   which makes cargo-ndk panic
  with "unknown package:
   35",
   and its panic handler DUMPS THE WHOLE ENVIRONMENT (it printed unrelated shell secrets),
  so the `build:rust:native` mise task guards on `ANDROID_NDK_HOME` and refuses to invoke cargo-ndk without it.
   The
  NDK's aarch64 clang wrappers top out at `android35` (no `android36`),
   so the native API floor is 35 (the .
  so runs
  fine on the device's 36,
   and a lower floor also helps the later minSdk bisect).
   NDK r28+ makes 16KB page alignment
  the linker default,
   so the .
  so is 16KB-aligned with no extra flag (verified:
   every LOAD segment at 0x4000);
   the
  old `-Wl,-z,max-page-size=16384` is unnecessary on this NDK.
   Rust Android targets are all installed.
- Build matrix in use:
   AGP 9.2.1,
   Gradle 9.5.1,
   Kotlin 2.2.10,
   Compose BOM 2026.05.01,
   Media3 1.10.1,
   compileSdk
  37,
   targetSdk 36,
   minSdk 36.
   Full vet recipe context in
  `doc/decision/kotlin-android-kopia-pcloud-vet-report/vet-jetpack-compose.md` and `vet-android-runtime.md`.

## Reference artifacts

- ADR (authoritative decisions):
   `doc/decision/music-player-android-port.md`.
- Desktop behavioral survey (full structured spec of every module + the Slint UI + per-rule Android-port deltas):
  produced by a workflow this session.
   Output JSON at
  `/tmp/claude-1000/-var-home-user-Monochromatic/17b04683-9e7b-4fa8-84d1-e2c4fc9a363c/tasks/w8cj0v5yb.output` (may
  be reaped).
   Regenerate by re-running the saved script at
  `/home/user/.claude/projects/-var-home-user-Monochromatic/17b04683-9e7b-4fa8-84d1-e2c4fc9a363c/workflows/scripts/music-player-desktop-survey-wf_e71b903d-8d8.js`.
  The durable source of truth is the desktop source itself.
- Desktop source:
   `package/music-player/desktop-app/src/*.rs` and `ui/app.slint`;
   `README.md` has a
  module-by-module Layout section.
   Key exact specs already extracted into the survey:
   the audio-extension
  allowlist (`flac, wav, wave, mp3, ogg, oga, opus, m4a, m4b, mp4, aac, aiff, aif, aifc`),
   the pagination
  tie-breaks,
   the true-peak constants (CEILING `0.8912509`,
   `gain = min(CEILING/peak, 1.0)`,
   4x Catmull-Rom),
   the
  FNV-1a `path+size+mtime` cache fingerprint,
   and the full Slint UI property/callback surface.
- kopia Android stack decision + vet reports (the GrapheneOS,
   Compose-on-device,
   and runtime findings):
  `doc/decision/kotlin-android-kopia-pcloud-stack.md` and `doc/decision/kotlin-android-kopia-pcloud-vet-report/`.

## Resolved: config-dir migration (2026-06-12)

- The identity unification (`7ea4ad07d`) already moved the resolver to `ProjectDirs::from("dev", "monochromatic",
  "musicplayer")`,
   which on Linux is `~/.config/musicplayer`;
   the README documents this.
   The user authorized the
  data move,
   so the old dir was renamed:
   `mv --no-clobber ~/.config/music-player ~/.config/musicplayer` (atomic
  same-filesystem rename,
   guarded on no running player process and a non-existent target).
   Both `peaks.json`
  (299561 B) and `session.json` (397536 B) carried over losslessly with mtimes preserved;
   the old path is gone.
- Verified the target path with a real invocation of the identical `directories = "5"` crate and the app's exact
  triple:
   `config_dir = /home/user/.config/musicplayer`,
   so a current-source desktop build finds the migrated data.
- Caveat for a future session:
   the installed binary `~/.local/bin/music-player` (Jun 11,
   pre-unification) still
  resolves `~/.config/music-player` and would recreate that old dir with an empty session if run.
   Rebuild and
  reinstall the desktop app (`mise run //package/music-player/desktop-app:build` then redeploy to `~/.local/bin`)
  so the installed binary uses the new path;
   not done here because the desktop app is not this port's focus.

## Next steps (the build phase, when the user says go)

Done:
 step 1 (toolchain),
 step 2 (scaffold),
 the skeleton half of step 3 (Media3 plays real audio on device),
 and
the pure-logic Kotlin port (the `core` package,
 52 tests green).
 Remaining:

1. Finish the real Media3 variant.
    DONE:
    the Compose UI (narrow layout,
    tap-to-play),
    the MediaStore library source
   (`openLibrary(List<Track>)`),
    the `MediaSessionService` (background/lockscreen/notification),
    and the SAF
   chosen-folder source,
    all verified on device.
    The SAF source (`SafTreeSource` + `LibraryRoot`) walks an
   `ACTION_OPEN_DOCUMENT_TREE` grant over `DocumentsContract` (iterative work-stack,
    visited-guarded,
    per-directory
   resilient),
    persists it via `takePersistableUriPermission`,
    and feeds the same `openLibrary` with tree-relative
   `displayPath`s codepoint-sorted to match MediaStore's contract;
    a held root wins over MediaStore and survives
   process death,
    a revoked or moved grant is detected before scanning so it cannot crash the headless service.
   Verified on the Pixel 6 against `Plain/Music` (3486 files):
    a loading notice during the multi-second scan,
   playback from a `content://` document URI,
    cold-load after force-stop with no re-pick,
    and loose root files
   (no-slash display paths,
    which MediaStore never produces) grouped without breaking.
    The picker is the one
   interactive spot that needs a human tap (driven via adb to open it;
    the folder choice + grant is the user's).
   DONE since:
    true-peak as a Media3 `AudioProcessor` (the `GainNormalizationProcessor` gain stage,
    async
   measure-on-miss,
    shared `PeakCacheStore`) and the WorkManager charging sweep (`PeakSweepWorker`,
    low-priority
   decode),
    both verified on device.
    The media3 variant is now feature-complete;
    next is the full-Rust variant.
   Instrument metrics from the start.

### Deferred from the MediaSessionService review

The adversarial review confirmed two more real findings,
 deferred deliberately (not blockers for this milestone):

- ALL-shuffle rebuild cost (LOW,
   unmeasured):
   in `ShuffleMode.ALL` the scope is the whole library,
   so
  `BrainPlayer.getState()` rebuilds N `MediaItemData` (and `setPlaylist` runs O(N) uid-dedup + timeline build) on
  every track transition,
   even though a plain advance only moves the cursor (`order` is unchanged).
   Fix when/if a
  Systrace shows a real main-thread hitch on a several-thousand-track library:
   cache the `MediaItemData` list in
  `BrainPlayer` keyed on the `queue.playbackOrder()` identity (or an order-generation counter bumped in
  `rebuildScopeOrder`),
   rebuild only when the order changes,
   and overlay the current item's duration separately.
- Hybrid/rust flavors crash on launch (pre-existing,
   not a regression):
   `createAudioEngine` throws
  `NotImplementedError`,
   now from `PlaybackService.onCreate` (the activity's `BIND_AUTO_CREATE` creates the service).
  Those flavors were never runnable;
   they are throwing stubs until the NDK work (per the ADR).
   When picking them up,
  either return a no-op `AudioEngine` (so the cross-flavor wiring runs end to end) or gate the service on
  `BuildConfig.FLAVOR`.
   Only the `media3` flavor runs today.
2. Full-Rust variant.
    The hybrid variant is dropped as a deliverable (owner directive 2026-06-12:
    a Rust meter over
   Media3-decoded PCM fixes only the meter,
    not the decode-bound cost,
    so it would not move performance enough;
    the
   decode is the lever,
    which only the full-Rust variant pulls).
    Steps 1 and 2 are DONE:
    the native toolchain
   (Task #10,
    cargo-ndk -> 16KB-aligned arm64 .
   so -> JNI) and the decode cross-compile (Task #11,
    libopus + symphonia
   `all` both cross-compile and run on device,
    `OK (3 tests)`).
    Remaining (Task #12):
    (b) port the desktop
   `Source`/decode/opus/truepeak/queue logic into the native crate,
    replace the PipeWire/cpal output with
   AAudio/AudioTrack,
    and implement the Kotlin `AudioEngine` seam over JNI;
    (c) feed `content://` fds via
   `ContentResolver.openFileDescriptor` (the first real end-to-end decode of a device file).
    The bridge is raw JNI
   today (smoke test);
    raw JNI vs UniFFI for the real engine interface is an OPEN decision (the ADR's UniFFI predates
   knowing whether UniFFI's JNA dependency behaves on GrapheneOS),
    to settle when building the real surface.
3. Verify each variant on device;
    Maestro for E2E;
    `androidx.test` pinned `>= 3.7.0`.
    Compare all metrics and let
   the user pick the winner.
4. Finalization (owner directive,
    2026-06-12):
    once the app is genuinely finished,
    bisect `minSdk` downward to the
   lowest API level that still builds and lints clean WITHOUT any source change (no new compat guards),
    then set
   `minSdk` to that floor.
    minSdk is at 36 now only to avoid old-API contortions during development;
    the true floor
   is whatever the APIs actually used permit.
    Method:
    lower `minSdk` in `app/build.gradle.kts`,
    run
   `mise run //package/music-player/android-app:build` + `:lint` (the `NewApi` lint check is the oracle:
    it errors
   when a used API exceeds `minSdk`),
    and binary-search to the lowest value with zero `NewApi` errors and no edits.
   Run for every flavor (each pulls in different APIs),
    and take the maximum of the per-flavor floors.

### Integration seams left by the port (wire these in milestone 2)

The `core` modules are pure;
 the platform parts were deliberately deferred and injected as parameters so the wiring
is explicit:

- True-peak:
   `measureTruePeak(channels, chunks: Sequence<FloatArray>)` takes the decoded PCM stream;
   feed it from
  the platform decoder (an offline ExoPlayer/MediaCodec decode pass for the Media3 variant,
   symphonia for the Rust
  variants).
   The gain APPLICATION (`process_sample`,
   gain-then-clamp,
   not yet ported) becomes the Media3
  `AudioProcessor`;
   port it there.
- Peak cache:
   `core` has FNV-1a fingerprint + in-memory map only.
   JSON load/save,
   the atomic write,
   the
  unsaved-counter batching,
   and the config-dir path are deferred;
   back them with Android app-private storage and a
  serialization choice (no library was added).
   The fingerprint takes `size` + `mtime` as parameters;
   supply them
  from `DocumentFile`/MediaStore.
   Cross-language match with the desktop cache is NOT required (Android cache is new).
- Session:
   `core` has the model + `pruneUnplayable(fileExists predicate)`.
   `load`/`save`/`session_path` and the
  `ShuffleMode` <-> wire-name mapping (`"Off"`/`"WithinPage"`/`"All"`) are deferred;
   add them with app-private
  storage.
   Feed `pruneUnplayable` a SAF/MediaStore existence check.
- Storage walk:
   `audioFilesSorted` is the pure per-directory filter-then-sort.
   Both sources are now DONE.
  `MediaStoreSource` returns a flat `IS_MUSIC` query,
   codepoint-sorted by display path.
   `SafTreeSource` walks a chosen
  `DocumentsContract` tree with an iterative work-stack (visited-guarded against provider cycles,
   per-directory
  try/catch so one unreadable folder does not abort the scan),
   filters by `core.isAudioFile`,
   and codepoint-sorts the
  whole result.
   It deliberately does NOT reproduce the desktop's per-directory DFS order (parent files before
  subfolder files):
   it collects everything then global-codepoint-sorts,
   matching `MediaStoreSource` so the two are
  interchangeable through the same pagination.
   Provider `DISPLAY_NAME`s are clamped at the path-assembly boundary
  (`core.joinDisplayPath`/`sanitizeComponent`:
   a separator inside a name cannot widen folder depth,
   control chars
  collapse to spaces),
   since names come from a provider,
   not a filesystem.
- Queue:
   `Queue.new()` seeds from `System.nanoTime()`;
   `Queue.withRngSeed(Long)` is deterministic for session
  restore and tests.
   The shuffle uses `kotlin.random.Random`,
   not the desktop's xorshift64 (sequence not portable).
- Not yet ported (small,
   port when needed):
   `frames_to_secs` and `file_name_of` (`playback.rs` utilities).

GrapheneOS testing notes proven this session:
 `adb push` into `/sdcard/Android/data/<appId>/files/` works for
fixtures;
 for the MediaStore default library expect `READ_MEDIA_AUDIO` to be revocable-off by default like INTERNET
was in the vet (grant via `adb shell pm grant <appId> android.permission.READ_MEDIA_AUDIO` for testing);
 SAF
persistable grants are per-applicationId,
 so each flavor needs its own grant.

## Gotchas

- GrapheneOS:
   the player needs no INTERNET permission (offline).
   Dynamic code loading is blocked (this is what
  kills the Slint UI,
   but a Rust `.so` loaded via JNI is fine).
   Avoid All Files Access (scoped storage only).
- The user values faithful porting and corrected under-surveying twice.
   Survey the desktop source before assuming
  behavior;
   the MediaStore default library is the one spot that does not map 1:1 to the desktop's folder model.
- Kotlin files DO need the `dum-dum-non-ts` comment convention (an earlier handover claim that they were exempt was
  wrong,
   owner-corrected 2026-06-12):
   the convention applies to every non-TS source file,
   Kotlin and Rust alike.
   The
  comment pass is deferred to finalization (Task #14,
   alongside the minSdk bisect) and is NOT written during
  development,
   so the existing Kotlin and Rust files deliberately still lack it.
   General repo rules apply:
   run
  everything through mise tasks,
   commit eagerly with scoped pathspecs,
   no AI-attribution trailers.
- Instrumented tests (`mise run //package/music-player/android-app:test:instrumented` ->
  `connectedMedia3DebugAndroidTest`) run on the connected device via gradle's own adb,
   so pin `ANDROID_SERIAL` and
  hold the `~/temp/agent/adb-phone.lock` flock around the run to coordinate with concurrent sessions.
   CAVEAT:
   AGP
  UNINSTALLS both the app and the test APK after a connected run,
   which also revokes the app's persisted SAF grant
  and clears its data;
   this is what wiped the owner's Plain/Music grant.
   To verify instrumented tests without
  disturbing an installed app,
   prefer building the test APK and driving `am instrument` manually (install both
  APKs,
   run,
   leave them installed),
   or just reinstall + re-grant the app afterward.
   The harness is minimal (runner
  1.7.0,
   no Espresso,
   so the Compose BOM's Espresso-3.5.0-on-Android-16 crash cannot trigger);
   decode-only tests
  produce no audio.
