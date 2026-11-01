# Streaming library load and a resume-persistence fix for the Android music player

Status:
 implemented (production code and host-JVM tests landed;
 on-device verification is the remaining step,
 driven by the user).
Audience:
 someone seeing this product for the first time.
It introduces the app,
 then proposes one focused change with two interlocking parts.

This revision folds in review feedback:
the original plan was correct in shape but under-specified three real edge cases
that only appear once the library list becomes interactive while it is still loading.
Those are called out inline.

## What this document is

The Android music player opens to a blank "loading" spinner for several seconds on every launch
while it scans your music folder,
then shows the whole library at once.
This plan replaces that with a list that fills in as the scan discovers tracks,
so you see music almost immediately.

While investigating that,
 we found a second,
 separate bug:
the app erases its own "resume where you left off" state on every launch.
The two are caused by code in the same area and share one fix,
so this plan folds them together.

## The product, in brief

Monochromatic is a multi-package workspace.
One of its apps is a music player,
which exists as a desktop build (written in Rust)
and as this Android port.

The Android port is built from three layers:

- A Jetpack Compose user interface (Kotlin),
   the screen you see and tap.
- A media3 `MediaSession`,
   which gives the system the lock-screen and notification controls,
  and keeps audio playing when the app is in the background.
- A Rust audio engine that does the real decoding and output,
  called from Kotlin over JNI (the Java-to-native bridge).

It plays a local music library.
The library comes from one of two sources:

- A folder you grant through Android's folder picker
  (the Storage Access Framework,
   "SAF"),
  which the app scans recursively.
- Or,
   if you grant audio permission and have not picked a folder,
  the device-wide media collection ("MediaStore").

Tracks are grouped into browsable pages shown as tabs:
one page per top-level folder,
plus A to Z pages for loose files at the root,
plus a single catch-all page for digits and symbols.
You tap a row to play it.
The seek bar,
 volume slider,
 shuffle control,
 and repeat toggle sit above the track list
and are usable at all times.

The app remembers where you left off
(the selected track,
 its playback position,
 and your volume,
 shuffle,
 and repeat settings)
and restores that on the next launch.
It also re-scans the source when you bring the app back to the foreground,
so files added,
 removed,
 or renamed since last time self-correct.

For deeper background,
 see the existing notes:

- [Android port decision](../decision/music-player-android-port.md)
- [Live-update rescan decision](../decision/music-player-live-update-rescan.md)
- [Session source-root decision](../decision/music-player-session-source-root.md)
- [Package context](../../package/music-player/android-app/CONTEXT.md)

The three decision links spelled `../decisions/` when this plan was written on 2026-06-13.
The `docs/decisions/` family became `doc/decision/` on 2026-07-15 (`bd19d57e20`),
so the targets moved to `../decision/`
and the `../decisions/` spelling no longer resolves.

## The two problems

### Problem one: the cold-start load blocks the whole library

When the app launches,
it scans the source end to end,
collects every track,
sorts them,
and only then hands the finished list to the screen.
Until that finishes,
 the track area shows a spinner and nothing else.
On a real device with a library of about 3,600 tracks,
the prior on-device logs showed that scan completing several seconds after launch.

The user's framing:
if the folder holds artists Ado,
 Bruno,
 and Charon,
the screen should show each as it is read,
not make you wait for the full scan before showing anything.

### Problem two: launching the app erases your resume state

On every cold start,
the saved "where you left off" state
(selected track,
 position,
 volume,
 shuffle,
 repeat)
is overwritten with blank defaults
before it is ever read back.
The library itself still loads,
but the per-session resume is lost on each launch.

This is an ordering bug,
 walked through under "How resume state is persisted" below.

## How the relevant parts work today

### The scan pipeline

Both sources follow the same shape.
They accumulate every track on a background thread,
sort the whole list by display path at the very end,
and return the finished list:

```kotlin
// package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/SafTreeSource.kt
suspend fun query(resolver: ContentResolver, treeUri: Uri): List<Track> = withContext(Dispatchers.IO) {
    // walk every directory under treeUri, appending audio files to `tracks`
    // ...
    tracks.sortedWith { left, right -> compareByCodePoint(left.displayPath, right.displayPath) }
}
```

The service launches that scan and hands the result to the controller (the "brain"),
through one of three entry points:

- `restoreLibrary` on cold start,
   which also reapplies saved settings and reselects the saved track.
- `openLibrary` after you pick a new folder.
- `reconcileLibrary` on foreground re-scan,
  which preserves the currently playing track by its URI and does not show a loading state.

The controller then rebuilds a single immutable snapshot,
 `PlayerUiState`,
which Compose observes and repaints from.

### How the UI decides what to show

The track list renders whenever the queue is non-empty.
The spinner only appears while the queue is empty and a load is in progress,
and that gate lives only inside the track list:

```kotlin
// package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/MainActivity.kt
if (state.queueSize == 0) {
    if (state.loading) {
        LoadingNotice()
    } else {
        Text("No music found in your audio library.")
    }
    return
}
// otherwise: render the LazyColumn of pages and rows
```

This is the key lever:
the screen already knows how to show a partial library.
If tracks arrive a batch at a time,
the queue becomes non-empty on the first batch
and the list grows on each repaint,
with no change to this gate.

Two ordering details matter:

- Pagination groups rows into pages but keeps each page's rows in the order they arrive,
  sorting only the pages themselves.
  So the incoming list must already be sorted for rows to land in the right place.
- The seek,
   volume,
   shuffle,
   and repeat controls render above the track list,
  outside that gate.
  They are live during the load,
   not just after it.
  This is what makes the mid-load interaction cases below real.

### How resume state is persisted

The controller has a persist callback,
 `onPersist`,
invoked only at the end of `refresh()`
(`PlayerController.kt:1684`).
The service wires that callback in `onCreate`,
and then,
 a few lines later,
 starts the load:

```kotlin
// package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlaybackService.kt
controller.onPersist = { saveSession() }   // line 467
// ...
ensureLibraryLoaded()                      // line 527
```

`ensureLibraryLoaded` sets `libraryLoaded = true`,
 then calls `controller.beginLoad()`,
which sets the loading flag and repaints.
That repaint fires `onPersist`,
 which calls `saveSession()`:

```kotlin
// package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlaybackService.kt
fun saveSession() {
    if (!libraryLoaded) return   // line 903: passes, libraryLoaded is already true
    SessionStore.save(this, controller.currentSession())
}
```

At that instant the controller is still empty,
so `currentSession()` reports nothing selected,
 position zero,
 and default volume.
`SessionStore.save` writes those blanks unconditionally,
erasing the real saved values.
Only after the multi-second scan does the load coroutine call `SessionStore.load`,
which now reads back the blanks it just wrote,
so the restore restores nothing.

The guard's intent is right:
do not save before a library is loaded,
 so startup cannot overwrite a good session.
But `libraryLoaded` flips at load start,
 not at load delivery,
so the guard passes too early to protect anything.

### Which fields persist, and how (audit)

`onPersist` fires only from `refresh()`,
 and not every change calls `refresh()`.
This matters for the mid-load cases,
 so it is spelled out:

- Selected track:
   persists when you tap a row
  (`playIndex` to `playCurrent` to `refresh` to `onPersist`).
- Shuffle and repeat:
   persist when changed
  (`setShuffle` and `setRepeatTrack` each call `refresh`).
- Volume:
   does not fire `onPersist`.
  `setVolume` updates the snapshot directly without calling `refresh`
  (`PlayerController.kt:1189`),
  so volume reaches storage only when some later `saveSession` runs
  (any other refresh-firing change,
   or the lifecycle saves in `onStop` and `onDestroy`).
- Position:
   not part of the snapshot at all (it is polled separately for the seek bar),
  so it too reaches storage only when `saveSession` runs,
   which reads `engine.positionSec()` at that moment.

The takeaway used below:
the terminal restore cannot assume "the user changed nothing during the load",
because the controls were live,
 and a mid-load volume change in particular leaves no `onPersist` trace.

## The plan

### Part one: stream the cold-start load

Make the scan emit the library in growing,
 already-sorted batches
instead of one final list.
Each source gains an optional progress callback;
when it has accumulated a threshold of new tracks (around 128),
it sorts what it has so far and emits that.

The emit decision is one shared,
 count-based gate.
Tracking the last emitted size is what keeps it from firing on every iteration after the first threshold:

```kotlin
// shape only, inside each source's scan, both sources gain `onBatch` (default null)
var lastEmittedCount = 0
suspend fun maybeEmitBatch() {
    if (onBatch != null && tracks.size - lastEmittedCount >= BATCH_SIZE) {
        lastEmittedCount = tracks.size
        onBatch(tracks.sortedWith(byDisplayPath))
    }
}
```

The gate is called from inside each source's innermost row loop,
not only between directories.
This is a correction to the first draft.

The MediaStore source reads one cursor,
 so the gate goes inside that cursor loop.

The SAF source walks directories,
 and each directory's rows are read inside a helper,
 `scanDirectory`,
which today swallows exceptions.
Emitting only between directories (the first draft's approach) would not stream a flat library
or a single large artist folder at all,
because nothing emits until that whole directory finishes.
So the gate goes inside the per-directory cursor loop instead,
and `scanDirectory` is made cancellation-safe so a superseding load is not swallowed:

```kotlin
// package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/SafTreeSource.kt
try {
    // cursor loop: append each audio file, then:
    maybeEmitBatch()   // suspends to hop to the main thread; can throw CancellationException
} catch (cancellation: CancellationException) {
    throw cancellation                       // a newer load cancels this one; do not swallow it
} catch (failure: Exception) {
    Log.w(SOURCE_TAG, "skipping unreadable directory ${frame.documentId}", failure)
}
```

The scan runs on a background thread,
so the callback hops to the main thread before touching the screen state:

```kotlin
// package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlaybackService.kt
val tracks = LibrarySource.load(this@PlaybackService) { batch ->
    withContext(Dispatchers.Main) { controller.reconcileLibrary(batch) }
}
```

Each batch reuses the existing `reconcileLibrary`.
That method already does the right thing for a growing list:
it adopts the new track set,
re-points the playing track by its URI if one is set,
leaves the loading flag untouched,
and does not touch shuffle,
 repeat,
 or volume.
A subtle point makes this safe:
on a cold start nothing is selected until you tap a row,
and a row you can tap is,
 by definition,
 already discovered,
so it is present in every later batch.
The "track is gone" branch in `reconcileLibrary` therefore never fires on a real selection during streaming.

### Part two: stop the resume clobber, and protect mid-load actions

Replace the wrong guard with the right one.
Add a flag that becomes true only when a library has actually been delivered to the controller,
and gate `saveSession` on that instead of on `libraryLoaded`:

```kotlin
// package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlaybackService.kt
private var sessionRestored: Boolean = false

fun saveSession() {
    if (!sessionRestored) return   // do not write until the saved session has been read and applied
    SessionStore.save(this, controller.currentSession())
}
```

`libraryLoaded` stays as it is for its other jobs
(keeping the load to a single run,
 and gating the foreground re-scan).
Now `beginLoad`'s repaint,
 and every streaming batch's repaint,
find `sessionRestored` still false and skip the write,
so the saved session survives untouched until it is read.

### Putting the load phase in order

Because the controls are live during the load,
the terminal restore must not overwrite anything the user could have changed in the meantime,
and it must not read the engine position before an async seek has landed.
The ordering below handles all of that.

Read the saved session up front (a read never clobbers),
apply the saved settings immediately so they are correct from the first frame and serve as the baseline
the user can override,
then start the load:

```kotlin
// package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlaybackService.kt
val session = SessionStore.load(this)   // read first; nothing has saved over it
controller.applySettings(session)       // shuffle, repeat, volume now (not at the end)
controller.beginLoad()                  // isLoading = true; repaint
```

When the scan finishes,
 finalize.
The terminal restore reselects only the track and its position,
never the settings (those were applied early and may have been changed since):

```kotlin
// new controller method; returns which path it took
enum class FinishLoadResult { RestoredSavedSession, KeptUserSelectionDuringLoad }

fun finishLoad(tracks: List<Track>, session: Session): FinishLoadResult =
    if (loadedUri != null) {
        // you tapped a track while it was still loading: keep your choice, just adopt the full list
        isLoading = false
        reconcileLibrary(tracks)
        FinishLoadResult.KeptUserSelectionDuringLoad
    } else {
        // no tap: reselect the saved track and seek to the saved position; do not touch settings
        restoreSelectedTrack(tracks, session)   // restoreLibrary minus the settings lines; clears isLoading
        FinishLoadResult.RestoredSavedSession
    }
```

The tap branch clears the loading flag itself,
because `reconcileLibrary` does not,
otherwise a library that finished empty would be stuck on the spinner.

The service flips `sessionRestored` only after `finishLoad` returns,
and saves immediately only on the kept-tap path:

```kotlin
// package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlaybackService.kt
val result = controller.finishLoad(tracks, session)
sessionRestored = true
if (result == FinishLoadResult.KeptUserSelectionDuringLoad) {
    saveSession()   // persist the mid-load tap; its position is real, the engine has been playing it
}
```

This is the fix for the async-position hazard.
`restoreSelectedTrack` calls `engine.load` then `engine.seekTo`,
but those only post commands to the Rust worker thread;
`engine.positionSec()` keeps returning `0.0` until the worker applies them
(it returns `0.0` whenever the sample rate is still the "nothing loaded" zero,
confirmed in `rust/src/engine.rs:949`).
So if persistence were enabled during the restore branch's repaint,
it would write position `0.0` over the real saved position.
Flipping `sessionRestored` after `finishLoad`,
 and not saving on the restore path,
leaves the on-disk position intact;
the engine seeks shortly after,
 and the next ordinary save records the real position.
The kept-tap path is safe to save immediately because that track has been playing for seconds,
so its position is already real.

### How the parts interlock

Part one makes the list interactive during the load,
 which creates three ways a terminal restore could
stomp on the user:
overwrite a tapped track,
 overwrite a changed setting,
 or overwrite the saved position with a not-yet-seeked `0.0`.
Part two's single delivery flag,
 the early settings application,
 and the kept-tap branch
close all three,
 and also neutralize the pre-existing cold-start clobber.
That shared `saveSession` path is why these belong in one change rather than two.

## Design choices and trade-offs

- Emit sorted-so-far,
   not raw discovery order.
  Pagination keeps each page's rows in arrival order,
  so an unsorted stream would show rows scrambled and then jerk into place at the end.
  Sorting the accumulated list on each batch keeps the rows correct for what is known so far.
  Re-sorting a few thousand items a couple dozen times happens on the background thread and is negligible.

- Sorted-so-far is correct but not perfectly stable,
   and that needs one more touch.
  As earlier-sorting folders are discovered late,
   page positions shift,
  and the visible page is tracked by a numeric index,
   not a label.
  If Charon shows first at index 0 and Ado is discovered later,
  index 0 now means Ado,
   so the tab you were looking at can jump.
  Mitigation:
   while streaming with no track selected,
  preserve the viewed page by its label,
   re-resolving its index after each batch.
  Once a track is playing,
   the existing "follow the current track's page" behaviour already covers this.

- Reuse `reconcileLibrary` for batches rather than a bespoke "append" method.
  It already preserves a tapped selection by URI,
   leaves settings alone,
   and leaves the loading flag alone,
  which is exactly the streaming behaviour,
   with no new branch to maintain.

- Apply saved settings early,
   restore only the track at the end.
  The controls are live during the load,
  so settings restored at the end would silently undo a mid-load change
  (and a mid-load volume change leaves no `onPersist` trace to detect).
  Applying them up front makes them the baseline the user can override,
  and has the side benefit that volume and shuffle look right from the first frame.
  This early application is safe against the later track load,
   which is the non-obvious part:
  volume is a persistent atomic gain on the engine's shared control block
  (`rust/src/engine.rs:185`),
   not reset by `load` (only the per-track normalization gain is),
  so an early `setVolume` survives the terminal `engine.load`;
  and shuffle is a stored mode field that `rebuildScopeOrder` re-reads on every `setTracks` and
  `playIndex` (`core/Queue.kt`),
   so setting the mode before any tracks exist still yields the
  correct scope once tracks arrive and the saved track is reselected.
  Without those two properties this reordering would silently drop volume or mis-scope shuffle,
  so it was checked rather than assumed.

- A count threshold,
   not per-file or per-folder emits.
  Per-file would repaint thousands of times;
  per-folder could still emit hundreds of times on a deep tree,
  and would not stream a flat or single-large-folder library at all.
  A shared count gate of around 128 keeps the batch count low while showing the first rows quickly,
  regardless of folder shape.

- Gate persistence on delivery,
   not on load start.
  This matches the guard's original intent and is the smallest change that fixes the clobber
  without detaching the persist callback,
   which streaming needs attached so a mid-load tap can be saved.

## Scope and non-goals

- Stream only the cold-start load (`ensureLibraryLoaded`).
  That is the "several seconds on every launch" pain.

- Keep the foreground re-scan (`rescan`) atomic.
  There the previous library is already on screen,
  so streaming would shrink the visible list to the first batch and then regrow it,
   a visible flicker.
  Streaming only helps when there is nothing on screen yet.

- Leave the folder-pick load (`reloadFromRoot`) blocking for now.
  It can share the streaming machinery later,
  but reusing `reconcile` there changes how old playback stops when the new folder lacks the old track,
  which is its own decision.
  One thing it must still do under this change:
  flip `sessionRestored` to true when its library is delivered,
  or a first-run user who picks a folder would never persist anything.
  Folder pick has no saved-position restore,
   so flipping it just before `openLibrary` is safe,
  unlike the cold-start path where flipping before the restore would re-expose the async-position hazard.

## Edge cases the plan handles

- Empty library:
   no batches emit,
   the terminal restore clears the loading flag,
  and the screen correctly shows "No music found" rather than a stuck spinner.

- You tap a track mid-load:
   the tap is preserved across later batches
  (re-pointed by URI),
   the terminal step keeps it instead of overriding it with the saved track,
  and it is persisted immediately afterward.

- You change volume,
   shuffle,
   or repeat mid-load:
  the saved values were applied as the baseline up front,
  the streaming batches do not reset them,
  and the terminal restore does not reapply settings,
   so your change survives.

- Saved non-zero position:
   the restore path does not persist during its async seek,
  so the stored position is not rewritten to `0.0` before the engine catches up.

- A newer load supersedes an older one (for example a folder pick during a scan):
  the cancellation propagates through the batch callback rather than being swallowed,
  because `scanDirectory` rethrows `CancellationException`.

- A saved track was deleted since last run:
  the existing restore auto-correction still applies (the URI lookup fails and nothing is selected).

### Known limitation: mid-load setting change, no-tap path, immediate kill

There is one deliberately accepted gap,
 called out so it is not a surprise.
On the no-tap path the terminal step intentionally does not save
(saving there would read the not-yet-seeked engine position and write `0.0`,
 the very clobber this avoids).
So a setting changed during the load is applied live but reaches storage only at the next save trigger
(any later refresh-firing change,
 or the `onStop` save when the app is backgrounded),
not the instant the load finishes.
A swipe-kill in the sub-second window between the load finishing and any further interaction
would lose that one mid-load setting change and restore the previously saved value next launch.
The tap path does not have this gap (it saves immediately,
 and a tapped track's position is already real).
Closing it for settings too would mean persisting a session composed from the saved position
rather than the engine's,
 which is more machinery than this edge warrants;
it is recorded here instead.

## Tests

There are no automated tests today for `PlayerController`,
 `PlaybackService`,
 or the two sources;
existing coverage is the pure `core` package (host JVM) and the native bridge (instrumented).
This change adds ordering-sensitive behaviour that a single on-device pass will not pin down,
so it should land with regression tests at the layer each concern lives in:

- Controller logic (host JVM,
   with a fake `AudioEngine` and the saved-position read from the fake).
  `PlayerController` is not in `core` and uses Compose snapshot state,
  so this needs the Compose runtime on the unit-test classpath;
  snapshot get and set work outside a composition,
   so behaviour is checked by reading `uiState` after each call.
  Cases:
   `finishLoad` keeps a mid-load tap;
   `finishLoad` reselects the saved track when no tap;
  the restore path does not reapply settings (a setting changed after `applySettings` survives);
  a vanished saved track clears the selection;
   an empty terminal clears `isLoading`;
  a streaming batch preserves a tapped selection by URI across growth.

- Batch emission logic (host JVM).
  The walk itself is coupled to `ContentResolver`,
   but the emit gate is not.
  Extract the count gate and the sorted-snapshot step into a pure helper and test it directly:
  it emits at the threshold,
   not on every call,
   tracks the last emitted size,
   and emits sorted-so-far.

- Persistence gating and source walks (Robolectric on host,
   or instrumented).
  These need `Context`,
   `SharedPreferences`,
   and a content provider.
  Cases:
   a refresh during the load does not write while `sessionRestored` is false,
   and does after delivery;
  a superseded load's later emissions are ignored.

Implementation status:
the first two layers landed as host-JVM JUnit tests
(`BatchEmitGateTest` for the emit gate;
 `PlayerControllerTest` plus a `FakeAudioEngine` for the
controller logic,
 including the page-by-label preservation),
 and pass under `mise run //package/music-player/android-app:test:unit`.
The third layer (Robolectric persistence gating and source-walk cancellation) is deliberately deferred:
it is the only layer needing new test infrastructure (`Context`,
 `SharedPreferences`,
 a fake content provider),
and its core logic (the `sessionRestored` boolean gate and the `CancellationException` rethrow) is exercised
indirectly by the controller tests and held to the on-device pass below.

## How we will verify on-device

This runs on a real connected device,
so the build is installed and the user drives it.

- Streaming:
   rows appear in the list before the "scan complete" log line,
  and the first rows show within a fraction of the old full-scan wait.
  A new per-batch log line makes the incremental emission visible in logcat.
  Check both a folder-per-artist library and a flat single-folder library,
  since the in-loop gate is what makes the flat case stream.

- Resume fix,
   including position:
  select a track,
   scrub to a clear non-zero position (say 1:30),
   set a non-default volume,
  fully force-stop and relaunch,
  and confirm the track,
   the position,
   and the volume all come back.
  Position is the field the async-seek hazard would silently reset,
  so it is checked explicitly,
   not just "a track came back".

- Mid-load interaction:
  while the list is still filling in,
   change shuffle and volume,
   or tap a track,
  and confirm the choice survives the rest of the load rather than snapping back to the saved values.

## Files this will touch

- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/SafTreeSource.kt`:
  add the batch callback;
   emit by count inside the per-directory cursor loop;
  make `scanDirectory` a `suspend` function that rethrows `CancellationException`.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/MediaStoreSource.kt`:
  add the batch callback;
   emit by count inside the cursor loop.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/LibrarySource.kt`:
  forward the callback through `load` and `scanRoot`.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlayerController.kt`:
  add `applySettings`,
   split `restoreLibrary` into settings-only and track-only halves,
  add `finishLoad` returning `FinishLoadResult`,
  and preserve the viewed page by label while streaming with no current track.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlaybackService.kt`:
  read the session and apply settings before the load,
   stream the cold-start load,
  add the `sessionRestored` flag,
   re-gate `saveSession`,
   and flip the flag at delivery in both load paths.
