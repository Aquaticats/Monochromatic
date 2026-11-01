# Music player on Android asked the media store for a column it lacks, once per track (Android 17, 10,000 tracks)

The Android app computed the cache fingerprint of a track from its byte size and a last-modified time.
For a media store row it asked the media provider for the document column `last_modified`, which the media store does
not have.
The provider threw `IllegalArgumentException: Invalid column last_modified` and logged a stack of about forty lines in
its own process, once for every fingerprint the app computed.
At 10,000 tracks the cold sweep and every start of the app produced thousands of these.
The fix (commit `cfd1d882c2`) stops asking.
No fingerprint changed, and the scroll jank that the release-build audit recorded at 10,000 tracks is not caused by
these calls.

This document records the root cause, the measurements before and after on the Android emulator, and the scroll jank
result with the emulator crash confound separated.
It answers findings 2 and 3 of `doc/audit/music-player-release-build-measurements-2026-10-15.md`.

## Symptom

On a release build with an invented library of 10,000 tracks the audit counted the log line
`Invalid column last_modified` 772 to 1,005 times in 15 seconds with the app in the foreground and no input, and none at
1,000 tracks.
Each exception comes with the provider's `Writing exception to parcel` stack in logcat, so a 17-second window carried
48,000 to 117,000 log lines.
The audit could not say what repeated the call, and could not tell whether the scroll jank at 10,000 tracks (13 of 23
windows with more than 100 janky frames, against 2 of 20 at 1,000 tracks) came from library size or from the emulator
sessions that later crashed.

## Root cause

### Who throws

The media provider enforces strict columns for any caller that is not the provider itself, and a projection naming an
unknown column throws.
Android main branch, fetched 2026-10-18; the line numbers are those of the fetched files, and the behavior is the
one the emulator log shows (the emulator image is the build `CE2A.260420.050`).

`packages/providers/MediaProvider/src/com/android/providers/media/MediaProvider.java:6052`:

```java
        qb.setStrict(true);
        if (isCallingPackageSelf()) {
            // When caller is system, such as the media scanner, we're willing
            // to let them access any columns they want
        } else {
            qb.setTargetSdkVersion(getCallingPackageTargetSdkVersion());
            qb.setStrictColumns(true);
            qb.setStrictGrammar(true);
        }
```

`frameworks/base/core/java/android/database/sqlite/SQLiteQueryBuilder.java:1122`:

```java
            throw new IllegalArgumentException("Invalid column " + userColumn);
```

`frameworks/base/core/java/android/database/DatabaseUtils.java:100` (the provider's binder thread writes the exception
to the reply parcel and logs it, because `IllegalArgumentException` is not one of the exceptions that skip the log):

```java
        } else if (e instanceof IllegalArgumentException) {
            code = 2;
        ...
        if (logException) {
            Log.e(TAG, "Writing exception to parcel", e);
        }
```

So the behavior is documented and intended: a third-party caller that names a column the media store does not have gets
an exception, and the provider logs it.
The cost of the mistake falls on the provider's process (a binder round trip and a forty-line stack per call).

### Who asked, and how often

Before the fix `TrackFingerprint.queryLastModifiedMs` asked every URI, whatever its provider
(`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/TrackFingerprint.kt:442` and
`:457` at `main` before the fix):

```kotlin
    private fun queryLastModifiedMs(context: Context, uri: Uri): Long =
        runCatching {
            context.contentResolver
                .query(uri, arrayOf(DocumentsContract.Document.COLUMN_LAST_MODIFIED), null, null, null)
        ...
        }.getOrDefault(UNKNOWN_MODIFIED_MS)
```

The exception was swallowed into the unknown time (zero), which is why nothing looked wrong in the app.
Three callers computed a fingerprint, each at these lines on `main` before the fix:

- `TrackMeasures.kt:169`, once per row of the page, in the page measure pass.
- `PeakSweep.kt:257`, once per swept track.
- `RustEngine.kt:812`, once per track load, to find the normalization gain.

The page measure pass is the one that explains "10,000 tracks and never 1,000".
`PlayerHost.kt:626` starts it at every process start, over every track of the selected page, in batches
(`core/PageMeasureRefresh.kt:67`, `readPageInBatches`), and `TrackMeasures.readMeasures` logs one line per batch
(`track measures read count=N`).
A page of 1,000 tracks finishes in under 2 seconds after the start, before the audit's 12-second idle window opened.
A page of 10,000 tracks takes 9.2 to 21.9 seconds (the starts are listed under "Result: Invalid column last_modified
before and after"), so the window opened while the pass was still reading, and every row it read threw once.
The exceptions were never idle behavior: they are the cost of reading the page, paid again at every start.

## Verification

### Unit cases

`TrackFingerprintModifiedTimeTest` (5 cases, Robolectric, invented providers):

- At the red commit `127cde590a` (the test, with the read made `internal`): 4 cases ran, 2 failed with
  `no query names the column the media store lacks expected:<0> but was:<3>` and
  `the refused column is asked for once, not once per row expected:<1> but was:<5>`.
  The value assertions of the media store case (`listOf(0L, 0L, 0L)`, "as before") passed there, which measures that the
  old code returned the unknown time for media store rows.
- On the branch: 5 of 5 pass (`app/build/test-results/testDebugUnitTest`, summed from the XML).

### Fingerprint value unchanged, on the device

`TrackFingerprintMediaStoreValueTest` (instrumented, commit `00fe30a859`) reads 25 media store rows of the invented
library and requires each fingerprint to equal `NativeBridge.nativeFingerprint(uri, size, 0)`.
It ran on the Pixel 9 Pro Fold emulator (Android 17, API 37) against a debug build of `main` and a debug build of the
fix, on the same private AVD copy, and logged each `uri`, `size` and `fingerprint`:
25 of 25 lines are identical between the two builds, and the assertion held in both.
So the media store fingerprints are unchanged, the peak cache and the duration cache stay valid across the upgrade, and
no cache version bump is made.
Where the column is valid (a Storage Access Framework provider) the value is read exactly as before; the unit case
`aDocumentProvidersTimeIsReturnedUnchangedForEveryRow` pins it.

The remaining behavior change is deliberate and recorded in the code comment: once a provider refuses the column with a
message that names it, that authority is not asked again in the process, so a provider that reports the column for some
documents and refuses it for others would read the unknown time for the rest.
The media store never reports it, and the Storage Access Framework providers that the app uses report it for every
document.

### Method of the emulator measurements

- Builds: `before.apk` is `assembleRelease` of `main` at `dd3b868b11`, `after.apk` is `assembleRelease` of this branch.
  Both are debug-signed local release builds of version 0.1.4, not minified, installed with `adb install`.
  The release dex of `before.apk` holds none of the new log messages and that of `after.apk` holds three
  (`strings --bytes=6` over `classes*.dex`).
- Device: the Pixel 9 Pro Fold AVD, emulator 37.2.12, API 37 with 16 KiB pages, 4 cores and 4 GB, on the inner display
  (two-pane layout), launched as in "Working agent launch: headful, host GPU, no container" of
  `doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`: its own `systemd-run --user` unit with
  `MemoryMax=12G`, a fresh reflink copy of the owned AVD per session, own ports, no `-no-window`, no `xvfb-run`.
  No `wm size` or `wm density` ran.
- One session per build, each on a fresh copy, in the same order: 1,000 tracks, then 10,000, then 1,000 again (the last
  cell separates a state left by the 10,000 work from library size).
  Libraries are invented tones (0.5 s, mono, 44.1 kHz) in one folder, pushed to `/sdcard/Music/` and scanned.
  A 64 MiB logcat buffer was set (`logcat -G 64M`) so that no burst overflows it; the audit's buffer size was not
  recorded, so its counts are not compared with these.
- Per cell, in this order: one cold run (`pm clear`, start, wait for the sweep's last line), one discarded warm-up
  scroll window per session, five idle windows, five early scroll windows, five settled scroll windows.
  An idle window is `am force-stop`, `am start -W`, 12 s of no input, then 15 s counted, as in the audit.
  An early scroll window starts 10 s after the process start, so the page pass can still be running.
  The five settled windows run back to back in one process after the page pass logged completion
  (`page measures are complete attempted=` at least 90 percent of the library).
  A window is eight swipe pairs (`input swipe X 1772 X 633 250`, `sleep 0.6`, back), 14 s, read with
  `dumpsys gfxinfo reset` and `dumpsys gfxinfo`; slow means more than 100 janky frames, the audit's rule.
- Counts come from `logcat -d -v epoch` lines whose timestamps fall in the window.
  Host CPU pressure was read from `/proc/pressure/cpu` (the `some` total over the window) and, before each start, the
  driver waited up to 5 minutes for the 10-second average to fall under 12 percent.
  The host is shared with other agents; the pressure of the scroll windows of sessions 2 and 3 is 0.7 to 25.5 percent.
  Session 1 recorded no pressure, and sessions 2 and 3 recorded it ten times too large (a units slip in the driver);
  the values here are divided by ten.
- Sessions: 1 (`before`, crashed), 2 (`after`, no crash), 3 (`before`, no crash).
  Sessions 4 and 5 (`after`, with a Settings-app scroll window as a control after each app window) crashed after 4 and
  2 minutes and the crash relaunch limit was reached, so they carry only a 1,000-track cold run and a few idle windows.

### Result: `Invalid column last_modified` before and after

Cold run, whole run from the process start until the sweep ended and the first page pass finished:

- 1,000 tracks, before: 2,896 and 2,881 (sessions 3, first and last cell) and 2,856 (session 1);
  after: 0 in the first and last cell of session 2 and in the first cells of sessions 4 and 5.
  The count matches the fingerprint calls: 1,936 rows read by the page pass plus 1,000 swept tracks is 2,936 against
  2,896.
- 10,000 tracks, before: 27,137 (session 3) and 24,932 (session 1); after: 0 (session 2).
  17,184 rows read by the page pass plus 10,000 swept tracks is 27,184 against 27,137.

Idle windows (5 repeats per cell, counts in run order, median and range):

- 1,000 tracks, before (session 3, both cells): 0 in all 10; after (session 2, both cells): 0 in all 10.
- 10,000 tracks, before (session 3): 941, 2,729, 0, 9, 0 (median 9, range 0 to 2,729);
  before (session 1): 2,475, 2,816, 571, 0, 0 (median 571, range 0 to 2,816);
  after (session 2): 0, 0, 0, 0, 0 (median 0).

The counts follow the page pass, not the clock.
Rows read by `TrackMeasures.readMeasures` in the same window, session 3: 928, 2,728, 0, 0, 0 against exceptions 941,
2,729, 0, 9, 0; session 1: 2,464, 2,800, 568, 0, 0 against 2,475, 2,816, 571, 0, 0.
Exceptions and rows read agree to within 1 percent in every window where the pass was running: one exception per row,
which is the link the audit lacked.
The windows with 0 are the starts at which the pass had already finished before the 12-second mark, which depends on
host load.
After the fix, window 3 of session 2 read 3,544 rows and logged none.

Time from the process start to the pass completing at 10,000 tracks (the five idle starts, seconds):
before (session 3) 13.5, 21.9, 12.1, 12.2, 11.8, median 12.2;
after (session 2) 9.4, 9.2, 15.8, 10.0, 9.8, median 9.8.
At 1,000 tracks it was 1.5 to 2.1 s before and 1.2 to 1.4 s after.
The medians differ by 2.4 s and the range of either build is larger than that, so the pass is not shown to be faster
after the fix; it still queries the size of every row.

### Result: scroll jank with the crash confound separated

Sessions 2 and 3 ran every cell to its end without an emulator crash (27 minutes each).
Janky frames per window (of 706 to 832 rendered), median, interquartile range, minimum to maximum, windows over 100:

- Session 3 (`before`), 1,000 tracks, first cell (10 windows): median 14, 10.3 to 16.8, 1 to 101, 1 slow.
- Session 3, 10,000 tracks (10 windows): median 9, 7.5 to 21.3, 5 to 122, 2 slow
  (early windows 5, 23, 107, 9, 122; settled windows 16, 6, 9, 7, 9).
- Session 3, 1,000 tracks, last cell (10 windows): median 15.5, 10.8 to 17.5, 9 to 110, 1 slow.
- Session 2 (`after`), 1,000 tracks, first cell (10 windows): median 9, 7.0 to 15.5, 6 to 80, 0 slow.
- Session 2, 10,000 tracks (10 windows): median 42, 19.3 to 87.8, 5 to 130, 2 slow
  (early windows 31, 130, 8, 17, 69; settled windows 53, 94, 118, 5, 26).
- Session 2, 1,000 tracks, last cell (10 windows): median 9.5, 6.5 to 16.0, 3 to 81, 0 slow.
- Pooled over the two sessions: 10,000 tracks 4 of 20 windows slow, 1,000 tracks 2 of 40.
  Fisher exact test, two-sided: p = 0.089.
  Before against after at 10,000 tracks: 2 of 10 each, p = 1.0.

What this shows:

- The jank is not caused by the exceptions or fixed by removing them: the same number of slow windows at 10,000 tracks
  before (2 of 10) and after (2 of 10).
  In session 2 the slow settled windows ran with no page pass active (0 rows read, 0 exceptions).
- The larger library has more slow windows in these two crash-free sessions (4 of 20 against 2 of 40), but the
  difference is not established (p = 0.089), and the two sessions disagree on where the slow windows fall: session 3's
  settled 10,000-track windows all have 6 to 16 janky frames, session 2's have 5 to 118.
- Host CPU pressure explains the session 2 outliers better than size does: its settled windows with 118, 94 and 53 janky
  frames ran at 25.5, 16.3 and 14.2 percent pressure, the two with 5 and 26 at 7.3 and 2.9 percent; session 3's settled
  windows ran at 3.6 to 5.9 percent.
- Slow windows are not specific to the large library.
  Session 1's 1,000-track cell ran under host load 28 to 54 (other agents' work) and 6 of its 15 windows were slow
  (5 of 5 early windows, 116 to 149 janky frames) with the same library that gave 0 of 5 early slow windows at load 5 to
  7 in session 2.
  Session 3's 1,000-track cells had 2 slow windows (101 and 110 janky frames), and the first of them had a 99th
  percentile of GPU time of 4,950 ms, the value of the audit's slow 1,000-track window.
- Where the frames were lost: in every slow window of sessions 2 and 3 `Number Slow issue draw commands` is within 10 of
  the janky count, `Number Slow UI thread` is 0 to 6, and there are 0 to 6 garbage collections.
  The time is spent issuing draw commands on the render thread, not in the app's main thread.

Conclusion: the jank at 10,000 tracks persists as a weak size effect in sessions that did not crash (4 of 20 windows
against 2 of 40), it is the same with and without the fix, and it tracks host CPU and GPU contention, which
also produced slow windows at 1,000 tracks.
It was not profiled: a profile needs an on-demand repeat of a slow window, and the emulator crashed in sessions 4 and 5
(see "Emulator crashes during this work") before a profiling session could run, with the relaunch limit of two reached.
No code change is made for it.
The next step is the one the audit named, a `perfetto` trace of slow and normal windows at 10,000 tracks, on a host
that is quiet, with the Settings-app control window of the unfinished sessions 4 and 5 to separate host contention from
the app.

### Emulator crashes during this work

Three emulator processes died; each was the headful host-GPU launch of
`doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`, not a container.

- Session 1: `SIGSEGV` in `qemu-system-x86_64` (`coredumpctl` pid 248809, 2026-10-18 00:12 local time, core 1.6 GB),
  23 minutes after the launch, in the 10,000-track scroll windows (the third `early-deep` window).
  Stack: `__strlen_evex` under `gfxstream::host::gl::TextureResize::TextureResize` under
  `gfxstream::host::gl::ColorBufferGl::create`, as in the entries of 2026-10-13 and 2026-10-15.
- Session 4: `SIGABRT` (pid 3671930, 01:58 local time, core 1.4 GB), 4 minutes after the launch, in the 1,000-track
  cell, in a gRPC callback (`grpc::internal::CallbackWithSuccessTag::Proceed`), the incident of 2026-10-14.
- Session 5: `SIGSEGV` (pid 4002170, 02:26 local time, core 1.4 GB), 2 minutes after the launch, in the 1,000-track
  cell, with the same `TextureResize` stack as session 1.

An additional emulator launch for the instrumented tests (not a crash) never became visible to `adb` for 30 minutes
(cause not established; the launch before it had used the same adb server port, and a launch on other ports worked at
once).
The session driver of the first three sessions did not notice a dead emulator and recorded empty windows for 25 minutes
in session 4 until a window needed a scrollable node; the driver now stops at the first `adb` error that names a lost
device.

## Verified fix

Commit `cfd1d882c2`: `TrackFingerprint.queryLastModifiedMs` returns the unknown time without a query for the media store
authority (`MediaStore.AUTHORITY`), and remembers an authority that refuses the column by name for the life of the
process.
Result: 0 exceptions in every cold run and idle window of the sessions on the fixed build (sessions 2, 4 and 5), against
thousands on the build of `main` in the same protocol; fingerprint values unchanged for 25 of 25 media store rows.

## What does not work

- Reading the audit's "idle foreground" count as steady idle work: the app does nothing at idle; the count is the
  tail of the page pass that follows every process start.
- Looking for a second source of the exceptions: the only Kotlin code that names the column is
  `TrackFingerprint.queryLastModifiedMs` and `SafTreeSource` (tree documents, which the media store path never reaches);
  after the fix the counts are 0 in every window, and the count before the fix matches the fingerprint calls to within
  1.5 percent.
- Comparing the audit's 772 to 1,005 with the counts here: the audit's windows opened at an unrecorded point of the pass
  and its logcat buffer size was not recorded.
- Blaming the jank on the exceptions: the slow-window counts at 10,000 tracks are equal before and after (2 of 10 each).

## Upstream filing decision

Nothing is filed upstream.
The tracker of the Android Open Source Project was not searched, because the first constraint fails: strict columns for
third-party callers and the logging of the exception are the intended behavior of the media provider (the code under
"Who throws"), and the defect was in this app's query.
`.out-of-scope/` was not consulted for the same reason; no upstream issue or comment is drafted.
