# Music player Android app

Android music-player client implemented with Jetpack Compose and a native Rust audio engine.
This package is a Gradle island with a nested Rust `cdylib` crate.
Mise owns tool provisioning and every supported build,
test,
install,
run,
and lint command.

## Architecture

Kotlin owns the Android application shell:
Compose UI,
permissions,
content URIs,
services,
media-session integration,
persistence,
and background work.
Rust owns decoding,
true-peak analysis,
the playback worker,
and the realtime AAudio output path.
Kotlin opens Android-managed media and passes file descriptors and coarse commands across JNI;
decoded samples and realtime buffers stay in Rust.

Application and native entry points:

- [`MainActivity.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/MainActivity.kt) is the launcher activity and
  Compose UI root.
- [`PlaybackService.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/PlaybackService.kt) owns the media session
  and service-backed player controller so playback survives activity recreation.
- [`NativeBridge.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/NativeBridge.kt) declares the Kotlin JNI
  surface loaded from `libmusicplayer_native.so`.
- [`rust/src/lib.rs`](rust/src/lib.rs) is the nested Rust crate root and exports the JNI entry points.
- [`rust/Cargo.toml`](rust/Cargo.toml) defines the standalone `musicplayer-native` `cdylib` crate.

[`mise.toml`](mise.toml) is the command interface for the package.
Its native build uses `cargo-ndk` to compile `arm64-v8a` for physical devices and `x86_64` for emulators,
then places both shared libraries under `app/src/main/jniLibs` for Gradle packaging.
[`app/build.gradle.kts`](app/build.gradle.kts) defines the single Android application module and its debug and
release variants.

## Prepare the Android toolchain

Run commands from the repository root.
Install the repo-managed JDK,
Android command-line tools,
Rust toolchain,
and `cargo-ndk`,
then install this app's SDK and NDK components:

```bash
mise install
mise run prepare:android
```

`prepare:android` accepts Android licenses and installs platform `android-37.0`,
build tools `37.0.0`,
NDK `29.0.13846066`,
platform tools,
and the `aarch64-linux-android` and `x86_64-linux-android` Rust targets.
The root `prepare` task also runs `prepare:android`,
but includes unrelated workspace preparation.
Mise exports `ANDROID_HOME` and `ANDROID_SDK_ROOT`,
so no machine-local `local.properties` is required.

## Build, install, and run

```bash
mise run //package/music-player/android-app:build:native
mise run //package/music-player/android-app:build
mise run //package/music-player/android-app:build:release
mise run //package/music-player/android-app:install
mise run //package/music-player/android-app:run:release
```

- `build:native` compiles only the release-mode Rust shared libraries for both supported ABIs.
- `build` rebuilds the native libraries and assembles the debug APK.
- `install` installs the debug APK under the release package name, which fails with a signature mismatch on a device
  that holds the release-signed app.
  Setting the environment variable `ORG_GRADLE_PROJECT_debugApplicationIdSuffix` to `.duck` installs it beside that app
  as `dev.monochromatic.musicplayer.duck`, with its own data, so the installed app is never replaced or wiped.
- `build:release` rebuilds the native libraries and assembles the optimized,
  non-debuggable release APK.
  The release variant uses the debug signing key so it can be installed without a separate keystore,
  unless `MUSIC_PLAYER_KEYSTORE_FILE`,
  `MUSIC_PLAYER_KEYSTORE_PASSWORD`,
  `MUSIC_PLAYER_KEY_ALIAS`,
  and `MUSIC_PLAYER_KEY_PASSWORD` are all set in the environment (or as Gradle properties),
  in which case it is signed with that key.
  Setting only some of them fails the build.
  The release workflow signs this way with the project key (`doc/runbook/release-the-music-player.md`).
  The app's `versionName` and `versionCode` come from the `version` in `../desktop-app/Cargo.toml`.
- `install` rebuilds the native libraries and installs the debug APK on a connected device.
- `run:release` builds the release APK,
  installs it on every attached device,
  and launches `MainActivity` on each device.

## Test

```bash
mise run //package/music-player/android-app:test:unit
mise run //package/music-player/android-app:test:instrumented:build
mise run //package/music-player/android-app:test:instrumented
mise run //package/music-player/android-app:test:instrumented:device
```

- `test:unit` runs the pure-logic JVM tests without a device or native build.
  It also runs the Compose wiring tests described below, which need JDK 21 or newer.
- `test:instrumented:build` builds the app and instrumented-test APKs without running them.
- `test:instrumented` rebuilds the native libraries and runs all instrumented tests on a connected device.
  Set `ANDROID_SERIAL` when more than one device is attached.
  Gradle's connected-test flow reinstalls the app and can clear its persisted SAF grant and warm peak cache.
- `test:instrumented:device` installs both APKs with `adb install -r` and runs the silent
  `PeakSweepWorkerTest` class while retaining app data and grants.
  Android's `am instrument` can exit successfully when a test fails,
  so verify that its output reports `OK (` rather than `FAILURES!!!`.

### Compose wiring tests

`app/src/test/kotlin/dev/monochromatic/musicplayer/wiring` renders the real `trackListPane` and `playerHost`
on the host JVM with Robolectric 4.17 and the Compose UI test rule (`ui-test-junit4`, version from the Compose BOM;
`ui-test-manifest` in debug), over a `PlayerController` with `FakeAudioEngine` and invented tracks.
No emulator is involved.
They cover the Compose call sites that no pure function can reach:

- which row is selected for a playing index, and how the marker moves on next and previous,
- where a page opens, what a reveal and a folder reveal scroll to, and that a track change alone never scrolls,
- that a track added to the shown folder gets its row and supporting line, and a folder change opens its page,
- that the next measure pass starts at the first visible track, seen through a recording content provider.
- that a reported analysis failure is a system notification and never an in-app message, that nothing at all appears
  with notifications off, and that the tap-time play failure still shows in the app (`AnalysisFailureWiringTest`, which
  reads the feedback queue and the notifications that the Robolectric shadow records).
- the Search page restyle (D126), in `SearchPageWiringTest`, which renders `searchPage` itself with Robolectric native
  graphics (`@GraphicsMode(NATIVE)`) and reads pixels back with `captureToImage`:
  the status strip and the header read the canvas color (no band) in light and in true-black dark,
  the divider pixel is `outlineVariant`, a result glyph sits in a 40 dp `secondaryContainer` circle that is round at its
  corners, the empty state has a 72 dp circle and stays above an invented 300 dp keyboard (also at 200 percent text),
  the title and glyph keep the header's columns on the cover and on the inner panel (D58) with the rounded row shape
  8 dp before them and outside the crease clear zone,
  the keyboard ring sits just inside the rounded row (outer ring at the edge, near ring inside it, corner bare),
  and the row clips its indication to the rounded shape.
  The window insets are dispatched to the composition root with `ViewCompat.dispatchApplyWindowInsets`,
  and keyboard focus needs `InputModeManager.requestInputMode(InputMode.Keyboard)` first,
  because a Compose test starts in touch mode, in which buttons and rows cannot take focus.
  The host cannot draw a press ripple into a capture, so the shape of the indication is checked with Material's focus
  layer instead; a press on a device is part of the emulator pass.
- the flush rows beside the crease (D130), in `RowAlignmentWiringTest` and `TemplateEditorAlignmentWiringTest`, which
  render the real right pane (title bar, subfolder heading, rows) under a provided pane clearance with native graphics:
  the title bar, the subfolder heading and the row text start at one x for no clearance, the 22.64 dp floor, a 10 dp
  and a 60 dp clearance, the end side ends at one x under an end clearance, the playing row's fill begins at root
  pixel 0 and its keyboard ring (outer ring at pixel 0, near ring inside it) keeps at least 8 dp from the text, and the
  template editor's first field row starts its text under the `Fields` heading with and without the crease inset.

- the Search field caret, in `SearchFieldCaretWiringTest`: typing after a layout change that rebuilds the header lands
  after the kept query, `Ctrl+F` on an open page puts the caret at the end without selecting, a caret the person placed
  in the middle is kept, and it comes back after the state is saved and restored,
- a tap on a Search result through the real host, in `PlayerHostWiringTest`: the first tap selects the track paused,
  the second starts it (D127).

The waits for a scroll or for a background measure pass use `waitUntil` with a wall-clock limit,
and every `@Test` has a `timeout`,
so a mutant that stops a scroll or a frame should fail fast instead of hanging the run.
`app/src/test/resources/robolectric.properties` selects API 36 (the app's `targetSdk`) and a bare `Application`.
Robolectric loads its framework jar for that API from `app/build/robolectric-runtime`,
which Gradle fills from the `robolectricRuntime` configuration (`org.robolectric:android-all-instrumented`),
so a run needs the network only once, to fill the Gradle cache, and works with `--offline` afterwards.
The test JVM also gets Robolectric's documented `--add-opens` flags (see `app/build.gradle.kts`).
The wiring cases add about 13 s to the 1,202-test run (measured from the result files of the last full run).

## Lint and clean

```bash
mise run //package/music-player/android-app:lint
mise run //package/music-player/android-app:lint:detekt
mise run //package/music-player/android-app:clean
```

- `lint` runs Android Lint against the debug variant.
- `lint:detekt` enforces the repository's Kotlin documentation rules.
- The root `mise run lint` runs the repository's Rust line-budget and rustdoc checks over the nested crate.
- `clean` removes Gradle build outputs.
  It does not remove the Rust target directory or generated JNI libraries.

## Design records

Use durable decisions for architecture and maintenance context:

- [Android port decision](../../../doc/decision/music-player-android-port.md)
- [Source Root session decision](../../../doc/decision/music-player-session-source-root.md)
- [Live-update rescan decision](../../../doc/decision/music-player-live-update-rescan.md)
- [Just-in-time shuffle decision](../../../doc/decision/music-player-jit-shuffle.md)
- [Peak-sweep parallelism decision](DECISION.peak-sweep-parallelism.md)

## Licenses page

Settings ends with an About group whose Licenses row opens the license notices of the app.
The text is the APK asset `THIRD-PARTY-LICENSES.txt`.
`ThirdPartyLicensesAssetTask` in [`app/build.gradle.kts`](app/build.gradle.kts) packs it into every variant:
the file from `mise run //package/music-player/android-app:notices` when
`app/build/third-party-licenses/THIRD-PARTY-LICENSES.txt` exists,
else a short note, so `build` and the instrumented tests work without cargo-about.
With `MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES=true` (the release workflow) a missing file fails the build.
[`LicensesPage.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/LicensesPage.kt) reads the asset on the IO
dispatcher, [`LicenseNoticeBlocks.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/LicenseNoticeBlocks.kt)
cuts it into blocks (section and subsection titles, wrapped prose, preformatted lines; at most 40 lines each) on the
default dispatcher, and a `LazyColumn` draws them.
The list is one keyboard stop: Up, Down, Page Up, Page Down, Home and End scroll it.
Back returns to Settings; on the unfolded screen Escape does too.

## Settings icons

The Settings rows carry the Material Symbols `queue_music`, `play_circle`, `autoplay` and `license` icons,
and the switch thumb carries `check` and `close`,
embedded as vector path data in [`SettingsIcons.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/SettingsIcons.kt).
The icons are Apache License 2.0 material from `google/material-design-icons`,
and the license text is `LICENSES/Apache-2.0.txt` at the repository root.
The source files, commit and style are named in the [package README](../README.md#third-party-icons).

## Custom control sizing

A requested minimum control size applies to both the visible control face and its owned layout target unless the
requirement explicitly says touch-target-only.
Transparent hit padding does not satisfy a requested visible minimum.

Custom interactive Compose controls reserve at least `48dp` by `48dp` inside layout.
Do not rely on Compose expanding touch targets outside undersized bounds,
because adjacent expanded targets can overlap.

## Theme and crease

The theme follows the system accent as its only platform color (A3, D108).
On API 31 and later the root is the middle tone of the system accent palette, `android.R.color.system_accent1_500`,
and below that it is the Fold baseline `#4E5E8B`.
Every role is then derived from the root in OKLCH by
[`core/ThemeRoles.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/ThemeRoles.kt),
using the one OKLCH implementation in
[`core/OklchColor.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/OklchColor.kt).
Both are ports of the desktop's `oklch.rs` and `theme_roles.rs`,
and `app/src/test/resources/theme-roles-vector.txt` (written by the Rust code, with its source hashes in the header)
proves that the same root gives the same sRGB roles in Kotlin and Rust.
[`PlayerTheme.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/PlayerTheme.kt) maps the roles onto every
Material 3 slot,
so the platform's `dynamicLightColorScheme` and `dynamicDarkColorScheme` are not used,
and carries the roles Material has no slot for (the Search match fill) in `LocalThemeRoles`.
Dark keeps a pure-black canvas under every root (D41) and light keeps a white canvas (D34).
Every other gray, the dark deck included, carries the root's hue unless the root is completely neutral (D118),
so the deck is `#0A0A0D` only at the baseline root, and no accent is grayer than the grays of its scheme (D121).
The track menu card is the canvas color itself, with a 1 dp `outline-variant` outline (D119),
and every text role is fitted to WCAG AA on the surfaces it is drawn on, for any root.
The app draws no gradient.
A future one must interpolate in OKLCH (D108), because Android blends gradient stops in sRGB.

The crease comes from Jetpack WindowManager (D110, E2):
[`CreaseDetection.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/CreaseDetection.kt) reads the window's
`FoldingFeature` list through `WindowInfoTracker`,
and [`core/CreaseReading.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/CreaseReading.kt) decides:
a vertical fold is a crease,
its reported width is the width of its bounds,
and the clearance is `max(7.5 mm, reported width)`.
Until the first layout report the `android.hardware.sensor.hinge_angle` system feature stands in.
The player, Settings and Search read the same answer.

On the unfolded player each pane provides its own clearance
([`PaneClearanceLocals.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/PaneClearanceLocals.kt),
rule in [`core/PaneClearance.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/PaneClearance.kt)):
text in the left pane (folder names, the deck, the undo toast) ends half of `max(7.5 mm, crease width)` before the
shared edge,
text in the right pane (header, track rows, subfolder headers, the empty notice, the first-run text) starts that far
after it,
and backgrounds and containers still span the band.
A window with no crease keeps the ordinary insets.
Beside the crease only text dodges it (D130): a track row's box (the current row's fill, the state layers and the ring)
spans the pane from edge to edge, and its text starts in the column of the subfolder headings and the title bar
([`core/TrackRowGeometry.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/TrackRowGeometry.kt)).

## Keyboard and focus

An attached keyboard gets the D106 map.
[`core/KeyBindings.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/KeyBindings.kt) is the table, one row per
chord with its scope, and `KeyBindingsTest` has one test per chord.
The older D102 chords stay in `core/KeyboardMap.kt`; `core/HostKeyboardModel.kt` asks the table first, then the older
classifier.
Implemented: `Alt+Left` and `Alt+Right` seek 10 seconds (60 with `Shift`), `Ctrl+Home` restarts,
`Ctrl+Shift+1` to `Ctrl+Shift+4` pick the mode, `Alt+Up` and `Alt+Down` pick the previous and next folder,
`Ctrl+Shift+G` shows the playing track's folder, `Alt+1` and `Alt+2` focus the folder names and the track list,
`Ctrl+Shift+O` opens the system folder chooser, `Escape` closes the cover's folder picker,
inside a focused list the arrows, `Home`, `End`, `PageUp`, `PageDown` and `Enter`
(a letter jumps in the rail and the names),
and on a track row `Menu` or `Alt+Enter` open its menu (D114), and `Delete`, `Ctrl+Shift+C` and `Ctrl+Shift+E`
trash it, copy its file name and show it in the file manager (also from the open menu).
There is no rescan or re-analysis key (D113) and no `Shift+F10`.
Left out on Android: the key hint badges, tooltips, `Ctrl+Q` and double `Shift`.

The folder rail, the folder names and the track list are each one focus stop (`Tab` passes them in one step) with their
own focus ring, moved by the arrow keys; their entries take no focus of their own,
so TalkBack still reads every entry through its semantics.
The focus indicator is a ring and nothing else (D117), drawn the way Firefox draws `:focus-visible` but hugging the
element: a 2 dp ring in the `primary` role flush against the element's edge, and a 1 dp ring in the `on-primary` role
around it, which keeps the ring readable on any surface.
Both rings follow the element's own shape (a pill stays a pill, a circle a circle, and a square corner gets a 2 dp outer
radius), reach 3 dp outside the element by default and move no content.
Every ring shows for keyboard focus only (CSS `:focus-visible`): the element or its list has focus and the last input
was a key, which the app reads from Compose's input mode, the same mechanism for a button, an icon button, a row that
takes focus itself and an entry of a one-stop list.
A tap or a touch that follows a key press hides the ring even though the list still holds focus, and the next key press
shows it again.
An entry that sits flush inside a list that clips its children (a track row, a rail cell, a Search result, a template
editor field) draws the same two rings just inside its own edge instead (D120): the 1 dp `on-primary` ring on the edge
and the 2 dp `primary` ring inside it, with the same corner radii as the outside pair.
Such a list reserves no padding around its entries, and the text keeps its place by starting 8 dp beyond the inner edge
of the ring at least.
The folder names are not flush in a clipping list
(the wrapped lines have room around them), so their rings stay outside,
and the track menu rows keep their reserved room.
Focus adds no fill, tint, state layer, bold or size change; hover and press keep their own state layers.
The drawing is [`FocusRing.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/FocusRing.kt) and the geometry, a spec
of widths, gap and square-corner radius, is
[`core/FocusRingGeometry.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/FocusRingGeometry.kt).
The geometry also holds the placement (outside or inside) and the keyboard-only rule, so `FocusRingGeometryTest` states
both.
`FOCUS_RING_OUTSET` is the room that a clipping parent leaves for an outside ring, which only the track menu still does.
`FocusRingContrastTest` checks the `primary` ring at 3 to 1 or better against every surface a focused element sits on,
and the `on-primary` ring against the `primary` ring, in both appearances and over a sweep of accent roots.

## Playback modes and the track menu

`In order` plays to the end of the folder on screen and then stops,
with the first track of that folder loaded and paused (D2);
`Next` still wraps around,
and `Repeat` replays the one track.
`core/Queue.kt` returns no next track for a natural end in `In order`,
and `PlayerController.handleNaturalEnd` turns that into the stop.
Every entry of the system playlist has its own id (`core/PlaybackEntryIds.kt`),
so a short folder in a shuffle mode can queue the same file again without the duplicate-id crash of the first review.

The track menu has six rows,
and each has one access key taken from its own words
(`core/TrackMenuAction.kt`, one table):
`P` for Play, `S` for Start shuffle from here, `D` for File details, `F` for Show in file manager, `C` for Copy filename
and `T` for Move to trash.
The letter is underlined in the label,
and pressing it with a hardware keyboard while the menu is open runs the row and closes the menu.
File details has no key of its own outside the menu.

## Tapping a track row (D124)

A tap, or `Enter` on the row under the focus ring, goes through `PlayerController.activateRow`.
On a row that is not the current track it only selects the row,
unless Auto-play on select is on (D129), which the host passes in:
`rowActivationFor` in `core/RowActivation.kt` is the one rule, and `activateRow` maps it to the engine calls.
With the setting off `selectIndex` makes the row current, pauses the engine, and loads the track paused, so the deck
shows it and nothing plays; with it on `playIndex` selects and starts it.
On the current track, playing or paused, it toggles play and pause through `togglePlay`, the call of the deck's Play
button, so a track restored from the saved session starts, and a track whose last load failed is loaded again.
The explicit Play row of the track menu still starts a track that is not current at once (`playIndex`).
A Search result for a track follows the same rule (D127): `PlayerHost.openSearchResult` selects a result that is not
the current track (`SearchActivation.SelectTrack`, through `activateRow`, so it also starts with the setting on) and
toggles with `togglePlay` for the current one; `wiring/PlayerHostWiringTest` taps a result through the real host.
The setting is stored by `SessionStore` in the `session` preferences under `auto_play_on_select`, outside `Session`
(a missing key is on, D133), and `AutoPlaySettingWiringTest` covers the switch row and the taps with it on and off.
The desktop app follows the same rule.
`PlayerControllerRowTest`, `PlayerControllerRowFailureTest` and `wiring/RowTapWiringTest` pin it.

## The library and the analysis follow the files

There is no rescan or re-analysis action (D113).
The system library follows a `ContentObserver` on the MediaStore audio collection,
and a library from a folder grant is read again when the app returns to the front;
both go through one debounced rescan in `PlaybackService`,
and a change that arrives while a load runs is answered once the load ends (`core/RescanPlan.kt`).

The analysis service decides what to measure from a ledger of the files it has swept
(`core/SweepLedger.kt`, stored in `sweep-ledger.bin`).
A file is new work when its address is unknown or when its size or modification time changed
(`core/TrackSignatures.kt`, read with the rows of the scan),
so a replaced or edited file is measured again.
A file that could not be decoded is tried again at the next start and whenever the library gains new work,
and it is reported once, in plain words, with no retry button.
The report is a system notification (D123), posted by the analysis service when the files first fail
(`AnalysisFailures.kt`, `AnalysisFailureNotification.kt`; wording in `core/AnalysisFailureNotice.kt`),
never an in-app message.
It is separate from the progress notification, on its own low-importance channel `analysis-failure`,
and tapping it opens the app.
When it cannot be posted (the notification permission is denied, notifications or the channel are off, or the post
fails) nothing is shown in the app: one warning without file names goes to the log, and the file keeps no row mark.
The tap-time message `Could not play <file>` stays in the app.
The measures that the open page shows are read again when the analysis measured their tracks (`MeasureInvalidation`).

## Search and a library of a million tracks

Search answers from an in-memory index (D112), never from composition.
[`core/SearchService.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/SearchService.kt) owns the index and a
worker thread that answers each query,
and [`SearchHost.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/SearchHost.kt) hands the newest answer to the
Search page.
The index keeps the normalized track and top-level folder names in one array,
buckets of word starts by their first, first two and first three UTF-16 units,
and a cached best-500 list per first character (`core/SearchIndex*.kt`).
A term of up to three units is answered from the buckets alone,
and a longer term is verified on the narrowed candidates.
Answers equal the old matcher's (tier, kind, name, library index, count, truncated flag and highlight ranges):
the old implementation is kept as a test oracle in `src/test/.../core/ReferenceSearch.kt`,
and `SearchIndexParityTest` and `ScaleSearchTest` compare the two on randomized and synthetic libraries up to 1,000,000
tracks.

A scan never blocks the screen (D111).
`LibraryScanner` hands each batch of found tracks to `LibraryIngest`, which sorts, derives the pages and builds the
Search index on its own thread, then gives the main thread a finished `BuiltLibrary` to swap in.
Search receives the new tracks after the player shows them and signals the open Search page,
which re-answers in place:
the query, the caret and the scroll position stay,
and results are keyed by track so a row the user is about to tap does not move.
While a scan runs an empty answer reads "Still looking through your library" and the Search button stays enabled.

### Search field caret

The host keeps the query across a fold change
(`rememberSaveable` next to the route), and D66 empties it for a new visit.
The field in `searchHeader` is a `TextFieldState`, whose edits apply at once,
and it is the source of the query while the page is open
(the host's query only seeds it, and edits are mirrored to the host):
a header rebuilt for the other panel starts with the caret at the end of the kept query,
the caret the person placed survives an activity recreation,
and `Ctrl+F` on an open Search page puts the caret at the end and selects nothing (D126).
The key handler presses `SearchFocusPort` (`HostKeyboard.kt`) and the open page has registered the caret move,
so a character typed right after the chord (7 ms later on the emulator) lands after the query, not before it.
`restoreOnlyWhileHolding` keeps a field saved by an earlier layout from overwriting a newer kept query.
`wiring/SearchFieldCaretWiringTest` pins the rendered field, including a press and a character in one event.

### Search page style

D126 restyled the page within the earlier decisions; the geometry is in
[`core/SearchLayout.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/core/SearchLayout.kt) and the drawing in
[`SearchPage.kt`](app/src/main/kotlin/dev/monochromatic/musicplayer/SearchPage.kt).
The status strip, the header and the results are the canvas color (pure white or pure black),
with a 1 dp `outlineVariant` divider under the 72 dp header.
A result row is flat on the canvas with a 40 dp `secondaryContainer` circle holding its 24 dp glyph,
centered in the same 48 dp slot as Back (D58),
and a rounded 21.3 dp shape that starts 8 dp before the content
and carries the press indication and the keyboard ring just inside its edge (D117, D120).
The empty and no-result states put a 32 dp magnifier in a 72 dp `secondaryContainer` circle
above a `titleMedium` title at weight 500.
They start 72 dp under the divider on both panels,
are padded by the keyboard and navigation insets on the cover,
and scroll when the space above the keyboard is short.
The exact specification that the desktop page copies is in `package/music-player/design/decisions.md` (D126).

`./gradlew :app:testDebugUnitTest` includes the 1,000,000-track cases, so its test JVM runs with a 3 GiB heap and the
serial collector (`app/build.gradle.kts`): G1's loop strip mining made short counted loops about twenty times slower on
the development host, which made timings measure the collector.
