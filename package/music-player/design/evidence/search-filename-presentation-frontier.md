# Search filename-presentation frontier

## Purpose and scope

The user accepted D75 to D80's accessibility design direction and asked
for the next design-only work.
This investigation checks filename presentation before manufacturing
another visual questionnaire.
Search A,
E2's information clearance,
the inner folder browser and complete deck stay selected.
No production Search,
matcher selection,
new IME study or original-AVD change is authorized.

File-extension presentation remains open within the required configurable
supporting-text template model.
D81 records editing through Settings.
The incompatible fixed-policy question is withdrawn;
neither the authored layouts nor existing player code selects a default
or renderer fallback.

## Source audit

### Ordinary-player source retains filename suffixes

Android's
`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/Pagination.kt`
function `rowDisplay` constructs the active folder's slash-terminated
prefix and removes only that prefix.
It does not strip a filename extension.
`MainActivity.kt`'s track row passes `item.name` to `rowDisplay` and paints
the returned text with single-line ellipsis.

Desktop's
`package/music-player/desktop-app/src/pagination.rs`
function `row_display` also removes only the folder label followed by a
slash.
Its documented example changes `Ado/B/C.opus` to `B/C.opus`.
That preserves `.opus`.

These are source observations,
not a new installed-player test or proof that the selected designs have
been implemented.
They establish a filename-preserving incumbent to test,
not authority to select Search's appearance.

### D11 is a distinct Settings design

D11 shows `Another Xronixle` instead of
`かめりあ(Camellia) - Another Xronixle.flac`
when the common-prefix setting is on.
The subject is the Settings control;
the example illustrates a shortened filename label.
It does not separately define Search extension visibility,
an extension parser or the match field.
Directory-prefix removal,
artist-prefix shortening and extension presentation are distinct
operations.
Preserve D11 rather than treating current code as its implementation or
silently adding a global extension switch.

### Search evidence uses illustrative names

The debug linked worktree's `SearchRankingFixture.kt` supplies literal
`Cam` and `Camellia Waltz` titles without source filenames.
Its same-title example distinguishes parents,
`Camellia` and `Cult of Luna`.
It does not exercise `Cam.flac` and `Cam.mp3` in one parent.

The selected Search row renderers allow enlarged titles and supporting
context to wrap.
They are not equivalent to the ordinary player's single-line ellipsis.
The source audit alone therefore cannot establish the appearance or
cost of retaining suffixes in Search.

`search-matching-acceptance-ledger.md` already separates rendered
extensions from extension-only eligibility and exactness.
This round does not choose whether `flac` generates a hit or how `cam`
is scored against `Cam.flac`.

## Consequential cases

- Same-folder `Cam.flac` and `Cam.mp3` must be distinguishable before
  activation.
  Parent context alone cannot distinguish them.
- Long equal stems with different suffixes test whether wrapping or
  truncation hides the distinguishing information.
- Equal filenames under different parents need useful parent paths,
  not only the same leaf folder label.
- A folder named `Camellia.flac` keeps its literal folder name.
  Track presentation must not infer item kind from a dot.
- Extensionless,
  uppercase-suffix,
  multi-dot,
  dot-prefixed and Unicode filenames need explicit examples.
  A suffix is filename text,
  not proof of the encoded audio format.
- Any collision-sensitive design needs a known source/query scope.
  Currently visible rows cannot establish uniqueness;
  filtered,
  later-arriving or off-screen partners must not silently change identity.

## Verified fixture preparation

The debug linked worktree added synthetic complete-filename labels in
short,
long and edge scenes without changing the selected row renderers.
These strings are not actual media files or indexed results.
Prototype commits are `1cbbba772` and `1ff1d135a`.
The host-JVM `SearchFilenameFixtureTest` report records 5 tests,
zero failures,
zero errors and zero skips.
The unknown-scene test failed when the guard was replaced by a short-scene
fallback;
restoring the exact source restored the passing test run.
The rebuilt debug APK SHA-256 is
`9fade97bc256c4597d14833bb01d9058aa1f65cfec4502debddbd3cb51b11983`.

The successful build used the existing package's `prototype:build` task
through `mise run --skip-tools` in a 6 GiB/2 CPU container.
The initial container failed while trying to resolve/install unrelated
root tools;
read-only installed tools and plugins plus the explicit skip flag
reached the package task.
No dependency was selected or installed by the filename work.
Build and unit tests are preparation,
not native label evidence.

A later display positive control repaired the label study's container
bridge;
`doc/troubleshooting/fedora-44-xvfb-run-auto-display.md` records why a
successful `/usr/bin/true` did not prove Xvfb readiness.
The actual display control passed before the new disposable boot.
`podman inspect` reports `6442450944` memory bytes and `2000000000`
NanoCPUs for `fold-search-filename-avd`.
The first display control did not establish GLX readiness.
An explicit driver-directory override enabled GLX and the disposable
Android boot;
the troubleshooting document separates that recovery from a System UI
dialog that blocked the first capture.
Neither failed attempt supplied accepted filename evidence.

## Inspected native witnesses

Internal task 136 produced 24 keyboard-closed views:
short,
long and edge labels across both panels,
100% and 200% text,
light and dark.
The [witness manifest][filename-manifest] records every expected matrix
combination,
the installed APK digest,
selected scene,
measured panel dimensions and retained crop.
These are synthetic complete-filename strings,
not actual media files or indexed results.

The retained app-area crops are 2076 × 2016 pixels for inner and
1080 × 2273 pixels for cover.
Only the top system-status strips were removed;
bottom navigation remains.
Every crop was inspected,
its retained RGB bytes matched its native frame,
and native/published opacity and PNG chunk checks passed.
A separately changed PNG was rejected by the same retained-pixel
verifier.
Raw frames and hierarchies remain private.

### What the captured states show

- Both `Cam.flac` and `Cam.mp3` visibly retain their distinguishing
  suffixes with identical supporting parent context.
- Both long equal-stem titles retain `.flac` or `.mp3` visibly.
  At 200% text,
  each title takes four lines inner and three lines cover.
  All rows in these fixture lists fit without scrolling.
- The dotted folder remains `Camellia.flac`,
  with a folder glyph and `Folder` label.
  The uppercase,
  multi-dot,
  dot-prefixed,
  Unicode and extensionless examples remain literal.
- The inner views retain the folder region and complete deck;
  cover views remain full width without an added deck.

Inspect the [inner long-name witness][inner-long] and
[cover long-name witness][cover-long] at 200% text for the observed
wrapping and visible suffixes.
The manifest links the complete matrix.

This establishes the tested labels in their captured states,
not universal filename legibility,
unchanged result capacity,
comparative superiority,
activation,
spoken output or focus behavior.
No extension-visibility preference was adopted.

## Inspected comparative controls

Internal task 137 assesses whether incumbent advice or a consequential
visual comparison is supported.
The original matrix alone did not isolate the suffix's contribution to
wrapping or test equal complete filenames under ancestors sharing a
leaf-folder label.
Prototype commits `7ecfb4880` and `a2e74a012` supply these bounded controls:

- Equal `Cam.flac` labels with `Collection A / Live` and
  `Collection B / Live` context.
- A matched long-name Search control omitting only the known literal
  suffixes.
  Its equal titles and context are intentionally undisambiguated,
  not a proposed usable presentation.
- A source-shaped ordinary-row control using the real `rowDisplay`
  utility and copied one-line `Text` ellipsis/padding.
  `MainActivity.trackRow` is private;
  this replica avoids reflection and production changes.
  It does not exercise the live ordinary player,
  current-track decoration,
  click behavior,
  complete player chrome or D11 Settings.

The controls preserve the selected Search host and remain nonfunctional.
The first host-JVM run caught an omitted shared-entry-point route:
`Unknown Search ranking fixture: rankfilecontext`.
Prototype commit `a2e74a012` added the routes and checked both parent-hit
flag values.
The restored fixture report now records 9 tests with zero failures,
errors or skips;
removing the unknown-scene guard was rejected again.
The rebuilt control APK SHA-256 is
`9e1f40a131931f13b44e1a6e8200c30bce553c777cee6b5cc615631765698f9d`.
The [separate control manifest][control-manifest] links 32 inspected,
keyboard-closed views:
long,
context,
suffix-omitted and ordinary-row controls across both panels,
100%/200% text and light/dark.
Exact retained RGB,
opacity,
PNG metadata checks and a changed-PNG rejection passed.
The original 24-view matrix retains its original APK provenance.
The first control attempt was interrupted by Node.js's ADB-output buffer
limit,
not a filename or keyboard failure;
[the troubleshooting record][buffer-record] explains the tested correction.

### Measured results and limitations

- Full Search visibly retains `.flac` and `.mp3` in every long-name
  control view.
  The suffix-omitted control shows identical titles and parent context;
  it is deliberately ambiguous,
  not a usable proposed design.
- Removing only those literal suffixes did not reduce title line counts
  in these samples:
  two lines at 100% on either panel,
  four inner and three cover at 200%.
  Corresponding title and supporting-text rectangles match in all
  eight panel/scale/theme combinations.
  This is measured text geometry,
  not action bounds or a general result-capacity claim.
- The [layout measurements][control-layout] validate each intended full
  title's occurrence and text rectangles.
  A shorter native `Cam.flac` title produces a smaller measured height,
  establishing that the extraction can detect height differences.
  A changed RGB byte separately checks the pixel comparator.
- New full-long controls have identical retained app-area RGB to the
  corresponding original long-name captures.
  This covers those views only,
  not APK equivalence or other behavior.
- `Collection A / Live` and `Collection B / Live` remain visibly distinct
  beneath the equal `Cam.flac` titles in every context view.
  This proves those authored paths are visible,
  not that arbitrary ancestor paths or real source identities are solved.
- The ordinary-row replica ellipsizes the long names before their
  distinguishing suffixes in every tested state.
  It uses `Live/` before the filename and omits Search's row decoration
  and support.
  The observation therefore does not isolate ellipsis alone or establish
  live ordinary-player behavior.
- Every row in these authored lists fits without scrolling.
  No scrolled witness was needed for these particular controls.

The control matrix did not capture the actual ordinary-player screen.
The subsequent [actual-renderer baseline][actual-baseline] now supplies
matched names,
real production chrome,
measured available width,
folder/page context and current-row decoration on disposable data.
It closes that static rendering gap,
not full `MainActivity` integration,
source loading or live playback.
The copied replica still cannot substitute for the actual renderer.

### Publication verification

The final verifier checked the complete 32-view control matrix,
published hashes,
retained APK digest,
crop geometry,
PNG chunks,
layout records and local document references.
Scoped Markdown lint and GitHub-rendered checks passed;
the three canonical-document prefixes have zero changed-block findings
and retain 185 existing findings outside the changed blocks.
The documented Node.js buffer harness also passed as written.

An initial private verifier wrongly treated `stableAppFrames` as a count.
The capture record defines it as a boolean stable-frame result;
strict boolean validation corrected the verifier without changing any
published frame or manifest.
It does not imply activation or other behavioral verification.
Internal task 137 is complete as a bounded comparison assessment,
not filename-presentation acceptance.

## Original/control-runtime shutdown

The owned filename runtime was shut down through its container's ADB
`emu kill` after the control capture.
Container,
ADB-device and owner-PID checks are empty.
The measured control-run snapshot was restored first:
200% font scale,
night mode enabled,
cover state,
accessibility disabled and no enabled accessibility service.
That snapshot does not reconstruct settings before the original matrix;
no such baseline was retained.
No original-AVD state was changed.

## Actual-renderer baseline continuation

The user directed further bounded design work after the assessment.
Prototype `720ba418f` invokes public production `playerScreen`,
its real private rows/pager and `PlayerController`,
with synthetic paused inputs and a no-audio engine.
The [inspected baseline][actual-baseline] and
[separate witness manifest][actual-witnesses] record its 32-view matrix,
mirrored host color factory,
measured `CHROMIUM_TABS` style and excluded service/source behavior.

The actual full-width ordinary player retains the long pair's suffixes
on inner at 100% in both themes and all captured selection conditions.
It clips them on inner at 200% and cover at both scales.
The short pair remains visible in all short-name controls.
The narrower copied-row control therefore did not predict every actual
player state.
Width,
chrome and parent context matter;
this is not a causal isolation of ellipsis or a presentation preference.

The new disposable runtime was restored to its fresh settings snapshot,
which is distinct from the stopped control runtime,
then gracefully stopped.
Container,
ADB target,
owning process and matching emulator process absence were verified.

## Assessment and proposed next comparison

The bounded assessment supports another consequential comparison,
not incumbent-only advice or a selected extension preference.
The measured omission control supplies no title-height saving for these
samples and loses the distinguishing information.
The source-shaped replica cannot justify copying its one-line treatment
into Search.
Neither finding proves the current full-title treatment superior to an
unbuilt alternative.

[The proposed comparison plan][comparison-plan] keeps visibility and
placement independent.
It compares complete filenames in titles with stem-first titles that
retain exact literal suffixes in supporting information,
then examines visibility using explicitly authored unambiguous and
collision cases.
Conditional visibility needs a known fixture scope,
including a partner outside displayed results;
this does not authorize matching or collision-detection implementation.
The continued-work authorization produced meaningful debug-only options:
64 initial views and six scrolled literal-name witnesses,
all inspected and published with separate provenance.
[The usable comparison](search-filename-usable-comparison.md) retains the
bounded measurements.
Its fixed-policy recommendations and question are withdrawn because they
omitted configurable supporting-text templates edited in Settings.
Task 141 corrected [the self-contained review](../questions/search-filename-comparison.html)
to verified evidence-only inspection with optional observations.
Captures remain authored layouts,
not a template implementation or default selection.
Fresh consumer checks decoded and opened all 102 images in the
initial desktop/light interaction pass.
Optional observations and modal/zoom/pan controls were separately checked
in each desktop/mobile light/dark context.
There are no policy votes;
this does not verify each preview in each environment.
Dependent implementation remains unauthorized.
No hide/show or placement preference has been adopted.
TalkBack,
activation,
focus transitions and keyboard-open behavior remain outside the proof.

[filename-manifest]: ../questions/evidence/search-filename-witnesses.json
[inner-long]: ../questions/evidence/search-filename-inner-long-dark-s200.png
[cover-long]: ../questions/evidence/search-filename-cover-long-dark-s200.png
[control-manifest]: ../questions/evidence/search-filename-control-witnesses.json
[control-layout]: ../questions/evidence/search-filename-control-layout.json
[buffer-record]: ../../../../doc/troubleshooting/node-26-adb-dumpsys-buffer-limit.md
[comparison-plan]: ../../../../doc/planning/music-player-search-filename-comparison.md
[actual-baseline]: search-filename-actual-player-baseline.md
[actual-witnesses]: ../questions/evidence/search-filename-actual-player-witnesses.json
