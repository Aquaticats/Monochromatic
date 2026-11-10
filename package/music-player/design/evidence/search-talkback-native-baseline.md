# Disposable Fold Search TalkBack baseline and acceptance boundary

## Scope and controls

This is a **debug-only baseline**,
 not native verification of a finished
Search implementation.
The disposable `Fold_No_Hardware_Probe` ran in a Podman container with
inspected limits of 6 GiB memory and 2 CPUs.
The installed debug APK SHA-256 was
`1caee7060acfb5bbcd9a02142b4c9bada6b5886517157d25fef4a48b9b1e9c05`.
The sampled inner panel was 2076 × 2152 physical px at 390dpi,
Android font scale `2.0`,
under the `search-deck-right-lift-retain-e2floor7p5-imeviewport-rankmixed-results-light`
candidate with fixed `cam` results.
The static result rows have no click handlers;
TalkBack 17.0.0.889642762 was enabled on this **disposable guest**
with its `Display speech output` overlay.
No Search query was typed,
no result was activated and no input method was opened.
Each inner physical-swipe capture checked that Android reported
`mInputShown=false` and `mImeWindowVis=0`.
Raw status-bearing screenshots and the unsanitized full OCR transcript are
private scratch artifacts;
this document reports bounded inspected utterances,
not those raw files.

The baseline-control launch showed “Folders” as initial speech and
“Open.
 Button” after the first emulator-gRPC right swipe.
That positive control exercised the **same** real TalkBack overlay later
used for Search.
The emulator's shipped
`$ANDROID_SDK_ROOT/emulator/lib/emulator_controller.proto:173-180,974-1074`
defines host `sendTouch` and a touch event's physical coordinates,
pressure and target display.
`doc/troubleshooting/android-emulator-37-talkback-speech-capture.md`
records why guest shell swipes,
hardware Tab,
UI Automator XML alone and TTS log lines do not substitute for a
physical-swipe speech-overlay capture.

## Inspected unfolded sequence

The scripted inner swipes reached “Back to player.
 Button” at recorded
step `2`,
`cam. Edit box. Search music` at step `3`,
and “Clear search.
 Button” at step `4`.
The next captured stops entered the **left alphabet rail** (recorded
steps `5` to `21`) before leaving that list at step `22`.
The right-side positive results were reached at step `23`:
TalkBack spoke the `Cam` title,
then spoke `Track · Cult of Luna · exact filename` at step `24` as a
**different** focus stop.
It likewise focused `Camellia` at step `25` and its
`Folder · opens this folder` supporting text at step `26` separately.
Other directly named results followed before the traversal entered the
left folder grid at recorded step `36` and the playback deck around
step `58`.
Later swipes wrapped through the same regions again.
These are **scripted swipe indices** from a private OCR transcript;
some repeated entries arose while speech or frames changed,
so they are not an exact count of unique accessibility nodes or time to
task completion.
The first title/context pair was also visually inspected in the private
TalkBack overlay crop.

This current debug composition therefore interleaves the left browser
header,
right Search header,
left alphabet rail,
right result title/detail text,
left folder grid and deck.
It neither copies D39's complete player-only folder/deck/track order nor
keeps the Search query and its results contiguous for sequential TalkBack
navigation.
The debug-only `PersistentResultLine` emits title and supporting text
as separate non-clickable nodes;
`uiautomator dump` corroborated unclickable result text,
but cannot reveal the spoken sentence by itself.
No evidence here says a result activation works,
that edit focus was requested under D63,
or that a future semantic grouping already speaks as one button.

## Folded-cover boundary

The same APK displayed fixed `cam` results on the 1080 × 2424px cover;
UI Automator found Back,
query,
Clear and result text,
and read-only input-method state remained hidden.
A TalkBack overlay showed initial focus on “Back to player.
 Button”.
However,
cover physical-swipe delivery was **not** validated:
the prior host-gRPC helper targeted the inner main display by default,
and adding `TouchEvent.display = 1` did not advance cover focus in the
observed probe.
The emulator controller's `getDisplayConfigurations` still reported
its main 2076 × 2152 configuration when the 1080 × 2424 cover was active;
Android's active logical display ID was `0` on the cover.
A single guest `input touchscreen -d 0 swipe` also left the shown Back
focus unchanged.
These null probes do not establish that cover traversal is impossible or
matches the inner order:
there was no positive control showing the selected cover input path could
advance focus.
No cover TalkBack sequence,
result speech,
keyboard fit or activation is claimed.

## Design consequence and unverified surface

A Search-specific reading order should place the active Search header and
result list together before traversing the retained folder browser and
deck,
without hiding those real left controls from accessibility.
Query edit focus and initial accessibility focus remain distinct:
Material Search guidance permits Back or field entry;
D75 adopts query-first for the just-invoked task,
with Back still reachable in reverse traversal.
It does not claim the fixture implements that policy.
Result rows need one actionable semantic unit with own name,
kind,
parent context and D72/D73's actual action;
the current separate non-clickable text stops are only a baseline.
D76 to D80 adopt contiguous traversal,
row meaning,
current-query announcements,
return targets and focus continuity as design goals.
They still require later host-application verification.
The fixed TalkBack fixture cannot prove that desired changes are built.
The original AVD was not launched or changed,
and no new IME experiment was run.

## Inspected cropped witnesses

These witnesses retain exact pixels in x `[0,2076)`,
y `[136,2152)` of the original 2076 × 2152px panel.
Only the system-status strip is omitted;
the bottom navigation strip is retained,
sometimes behind the speech overlay.
The resulting 2076 × 2016px files are **status-cropped witnesses**,
not full-panel geometry references or clean visual-design captures.
TalkBack's overlay can cover app content.
Each output is opaque,
stripped of PNG metadata and pixel-compared with the same raw crop.
Raw screenshots and OCR remain private.
`../question/evidence/search-talkback-witnesses.json` records the crop,
APK hash,
font scale,
TalkBack version and individual output SHA-256 values.
Utterance punctuation in prose is descriptive,
not a recording of audio timing or pronunciation.

### Query at scripted swipe 3

The green focus rectangle encloses the query;
the overlay reads “cam.
 Edit box.
 Search music”.

![Query focus and matching TalkBack overlay](https://monochromatic-lfs.aquaticat.workers.dev/d43727d93dd33debf56196c182163346e7e35a05a46d1c7321325d7cda0937cd/package/music-player/design/question/evidence/search-talkback-inner-query-s200.png)

### Left alphabet rail at scripted swipe 5

The first stop after Clear enters the left alphabet rail rather than the
right Search results.
The overlay's `17 items` is the rail's list context,
not a Search result count.

![Alphabet rail focus between query header and Search results](https://monochromatic-lfs.aquaticat.workers.dev/28e7eab8b8dd64465ce2e197c10ed00a1f807863510fdde83f38ca638d232758/package/music-player/design/question/evidence/search-talkback-inner-alphabet-s200.png)

### Result title at scripted swipe 23

The green rectangle encloses only `Cam`;
the overlay reads `Cam`.

![Cam result title focused independently from supporting text](https://monochromatic-lfs.aquaticat.workers.dev/79c045a7488f982767f775423eeb5dc0b264b439e4b933da86bb48f6a4bdc7c7/package/music-player/design/question/evidence/search-talkback-inner-title-s200.png)

### Result support at scripted swipe 24

The following focus rectangle encloses supporting text instead of the
title,
and the overlay reads `Track · Cult of Luna · exact filename`.
This corroborates the separate-stop baseline,
not actionable row semantics.

![Cam supporting text on the next TalkBack stop](https://monochromatic-lfs.aquaticat.workers.dev/62cf1cbb495a6f8e8d697b9a71c23454b75cdde916735c3b70e2af74d1d765a3/package/music-player/design/question/evidence/search-talkback-inner-support-s200.png)
