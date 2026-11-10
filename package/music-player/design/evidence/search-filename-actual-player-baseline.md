# Static actual-player-renderer filename baseline

## Purpose and boundary

This closes the copied-row rendering gap for a bounded filename comparison.
The debug host invokes the production `playerScreen` in `MainActivity.kt`,
its private pager/row renderers and the real `PlayerController`.
It uses synthetic `Track` paths and an `AudioEngine` recording double with
no audio output.
It does not execute the full `MainActivity.onCreate` path,
source loading,
service binding,
peak indexing or real playback.

The capture is an actual production-renderer baseline with authored paused
inputs,
not a live-player baseline or implementation acceptance.
The Search A and E2 selections are unchanged;
this baseline depicts current production ordinary-player chrome,
not the selected future Search layout.

## Provenance and setup

Prototype `720ba418f` supplied the debug-only host,
fixture,
no-audio engine and tests.
The tracked production Android source/resource diff against main revision
`4da3fa051` was empty at capture preparation.
A complete filesystem recheck,
including untracked and ignored files,
found the same 43 non-JNI source/resource files with identical bytes.
Main additionally holds 10 generated JNI binaries absent from the prototype.
The complete `src/main` trees are therefore not identical.
Native packaging equivalence was not established;
the initial broader source-tree claim is withdrawn.
The retained installed APK SHA-256 is
`84edf1e75cc8e9325e19ab0c23d3f1f314bc271a3479e8f3bc920f78d66fcd9e`.

The [witness manifest][witnesses] names all `32` views:

- Both panels,
  light/dark and `100%`/`200%` text scale.
- Long suffix-distinct pairs with no current row,
  first current row and second current row.
- Short `Cam.flac`/`Cam.mp3` pairs with no current row.

Every accepted view checked installed APK identity,
current keyboard-closed fields,
measured panel/scale,
intended hierarchy labels,
baseline window focus and stable retained app-area RGB.
The native logger recorded `CHROMIUM_TABS` in every view.
The focused host did not report the playback or peak service in its
capture-specific service dump.
These are capture prerequisites,
not activation or speech evidence.

The disposable `Fold_No_Hardware_Probe` runtime was independently checked
at `6442450944` memory bytes and `2000000000` NanoCPUs.
Original AVD state was not changed.
The fresh settings snapshot was taken before this matrix:
font scale `2.0`,
night mode yes,
base panel state `2`,
accessibility disabled,
enabled services `null`,
and stay-on setting `1`.
Restoration and owned-runtime shutdown remain due after the permitted
comparison captures finish.

## Actual renderer and mirrored host

The debug activity mirrors production `enableEdgeToEdge`,
full-size `MaterialTheme`/`Surface` hosting and the inherited application
theme.
No study padding,
forced orientation,
visible picker or overlay changes the player width.

Production's private `musicPlayerColorScheme` is not invoked by reflection.
Its dynamic-color selection and true-black dark surface/background
expression are mirrored at the debug host boundary.
The actual production row,
pager and chrome implementations are invoked,
not copied.
Older-platform color fallback behavior was not exercised by this Android
`37` matrix.
The existing prototype build also adds debug-only extended Material icons;
its build configuration is not claimed identical to production.
No dependency was selected or installed by this baseline task.

The controlled disposable debug-signature rejection required uninstall and
reinstall of the disposable app only.
That reset app-local preferences;
`CHROMIUM_TABS` is the measured resulting first-install style,
not a reconstruction of an earlier app preference.

## Authored library and state

The common authored root is `StudyLibrary`.
Two target tracks sit beneath `Cult of Luna`,
with the long pair beneath its `Live` subfolder.
An additional `StudyLibrary/ZAnchor/Anchor.opus` keeps common-root trimming
from swallowing the intended artist-level parent.
It produces the additional visible `ZAnchor` page tab.
The target page is `Cult of Luna` in every captured scene.

No file with these names is read or indexed.
The suffixes are literal filename label content,
not measured encoding or inferred kind.
The synthetic URI identities are distinct even when a visible row
ellipsizes their distinguishing suffixes.

The no-current condition uses `openLibrary` without autoplay.
Selected conditions use real `finishLoad` restoration of the intended
synthetic URI paused.
The host tests verify both identities,
page membership,
exact `rowDisplay` text and absence of autoplay.
The authored selected position is `66` seconds with `275` seconds duration;
these are not media measurements.
The native selected views show the real current-row background treatment
and a `Play` control,
not a playing-state claim.

## Inspected findings

All eight native contact sheets were inspected,
including each selection condition and the short-name visibility control.

- Inner at `100%`:
  both long rows visibly retain `.flac`/`.mp3` in light and dark,
  whether neither,
  the first or the second row is current.
- Inner at `200%`:
  the long rows ellipsize before those distinguishing suffixes in every
  captured selection/theme condition.
- Cover at `100%` and `200%`:
  the long rows ellipsize before those suffixes in every captured
  selection/theme condition.
- Both panels and scales,
  no-current condition only:
  the short pair visibly retains `.flac`/`.mp3` in light and dark.
  Short first/second-current conditions were not captured.
- Current selection:
  the actual filled current-row background moves between the correct rows.
  It does not recover a clipped suffix in the tested states.
- Page context:
  `Cult of Luna`,
  `ZAnchor` and the long-row `Live/` prefix remain visible.
  All authored target rows fit without scrolling.

Hierarchy parsing found `64` target semantic-row records with validated
rectangle syntax.
Their measured horizontal bounds were `[29, 2047]` inner and `[29, 1051]`
cover,
so semantic row widths were `2018` and `1022` physical pixels respectively.
These include the row's layout/padding context;
they are not ink bounds or an accessibility target acceptance result.
No nonempty app text node entered the removed top status strip.

The source-shaped ordinary-row replica clipped the long suffixes in all
of its narrower Search-pane controls.
The actual ordinary player retains them on inner at `100%`.
The replica therefore cannot substitute for this actual renderer or
justify a blanket claim that ordinary-player suffixes are always hidden.
Different width,
chrome,
parent-prefix and supporting-text contexts also prevent attributing the
comparison solely to ellipsis.

## Privacy and lifecycle verification

Only inspected crops are published.
The top system-status strip is removed;
bottom navigation remains.
Inner crops are `2076 × 2016` pixels,
cover crops `1080 × 2273` pixels.
Exact retained RGB,
opacity,
metadata-free PNG chunks and a changed-byte sensitivity control passed.
Raw status-bearing frames,
hierarchies and logs stay private.

The first capture harness focus check incorrectly queried only the
`dumpsys window windows` section,
which did not supply the expected current-focus field in that probe.
A full `dumpsys window` query showed `FilenameBaselineActivity` as the
focused host.
The corrected matrix uses that full query and distinguishes a missing
field from a different focused window.
No blocked-overlay cause was established for the earlier rejection.

After the complete matrix,
the immediate release-log check failed.
A follow-up read contained
`FilenameBaselineEngine.release: released paused renderer double`,
with no `AndroidRuntime` fatal exception in that captured log scope.
The original failed immediate check is retained in provenance.
A later logger observation does not isolate callback timing from log
transport timing or establish a general shutdown guarantee.

The [host-JVM logging-boundary investigation][logging] records the initial
four test failures,
injected diagnostic writer,
specific event assertions and fresh autoplay-guard mutation proof.
Complete restored tests and the APK rebuild passed before accepted frames.

## Next comparison

Continue usable Search alternatives from the [filename comparison plan][plan].
Compare literal suffix placement while holding identifying information
constant,
then visibility separately using explicitly authored scopes.
Neither this baseline nor the negative control selects a suffix policy,
matcher,
parser,
collision algorithm or production implementation.

[witnesses]: ../question/evidence/search-filename-actual-player-witnesses.json
[logging]: ../../../../doc/troubleshooting/android-agp-9-host-jvm-log-boundary.md
[plan]: ../../../../doc/planning/music-player-search-filename-comparison.md
