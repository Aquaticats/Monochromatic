// File summary (folds in the old KDoc that lived on `object MediaStoreSource`):
// This file reads the device's shared audio library out of Android's `MediaStore` (a
// system-wide content database of every audio file the OS has indexed) and turns each
// row into a `Track` the player can show and play. It is the Android analog of the
// desktop player's "scan a chosen music folder" step, with one honest divergence:
// the desktop walks a single folder you pick, whereas `MediaStore` is the whole device's
// audio collection, so this code filters to `IS_MUSIC != 0` to drop ringtones, alarms,
// and notification sounds. (A "point at one folder" source, matching the desktop more
// exactly, is a separate file that lands in a later slice.) Each indexed row becomes a
// playable `content://media/...` URI plus a folder-relative display path; the finished
// list is sorted by display path in Unicode code-point order so it matches the desktop's
// bytewise path sort and the on-page load order stays faithful. The query runs on the IO
// thread pool because it is cursor I/O (reading rows one at a time) over the whole
// collection, which can be large.

// What:     A Kotlin `package` declaration. It names the dotted namespace every symbol in
//           this file belongs to (here `dev.monochromatic.musicplayer`). Other files in
//           the same package can refer to these symbols without importing them.
// Why:      Android/Kotlin group code by package; this places `MediaStoreSource` in the
//           app's main package so the rest of the app can find it.
//
// In TS you'd write (pseudocode):
// ```ts
// // no statement — TS uses the file path / module system instead of a `package` line
// ```
package dev.monochromatic.musicplayer

// What:     An `import` of a single type, `ContentResolver`, from the Android framework
//           package `android.content`. `ContentResolver` is the object you hand a query
//           to in order to read a content database (here, `MediaStore`).
// Why:      The `query` function below takes a `ContentResolver` parameter and calls
//           `.query(...)` on it; without this import the type name is unknown.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ContentResolver } from "android/content";
// ```
import android.content.ContentResolver

// What:     Imports `ContentUris`, an Android helper class whose static method
//           `withAppendedId(uri, id)` builds a row-specific `content://` URI by appending
//           a numeric row id onto a base collection URI.
// Why:      We need it to turn each row's numeric `_ID` into the playable per-track URI.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ContentUris } from "android/content";
// ```
import android.content.ContentUris

// What:     Imports `Build`, an Android class exposing the device's OS information.
//           We use `Build.VERSION.SDK_INT` (the running Android API level as an integer)
//           and `Build.VERSION_CODES.Q` (the constant for Android 10's API level, 29).
// Why:      Older Android versions lack the `RELATIVE_PATH` column, so we branch on the
//           API level to pick the right columns and base URI.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Build } from "android/os";
// ```
import android.os.Build

// What:     Imports `MediaStore`, the Android class that names the system audio/video/image
//           content database and all its column names and base URIs (e.g.
//           `MediaStore.Audio.Media._ID`).
// Why:      Every column name, the base collection URI, and the `IS_MUSIC` flag come from
//           this class; it is the heart of the query.
//
// In TS you'd write (pseudocode):
// ```ts
// import { MediaStore } from "android/provider";
// ```
import android.provider.MediaStore

// What:     Imports `Log`, Android's logging facility. `Log.i(tag, message)` writes an
//           "info"-level line to the system log (logcat).
// Why:      We log the final row count so an on-device verification run can read back how
//           many music tracks the query found.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log

// What:     Imports the `compareByCodePoint` function from this app's own `core` package.
//           It compares two strings by Unicode code point and returns an `Int`: negative
//           if the first sorts before the second, zero if equal, positive otherwise.
// Why:      We sort the finished track list with it so the order matches the desktop's
//           bytewise path sort exactly.
//
// In TS you'd write (pseudocode):
// ```ts
// import { compareByCodePoint } from "./core";
// ```
import dev.monochromatic.musicplayer.core.compareByCodePoint

// What:     Imports `isAppleDoubleSidecar`, this app's own `core` predicate that returns
//           `true` when a final filename starts with Apple's `._` resource-fork sidecar
//           marker.
// Why:      MediaStore's `IS_MUSIC` filter can still surface sidecars with audio-looking
//           names, so this source needs the same sidecar rule as SAF and desktop scans.
//
// In TS you'd write (pseudocode):
// ```ts
// import { isAppleDoubleSidecar } from "./core";
// ```
import dev.monochromatic.musicplayer.core.isAppleDoubleSidecar

// What:     Imports the `BatchEmitGate` class from this app's own `core` package: the pure
//           rule that decides when a streaming scan has accumulated enough new tracks to
//           emit another sorted-so-far batch.
// Why:      This source creates one gate per call and asks it, after each appended track,
//           whether to emit a batch to the screen.
//
// In TS you'd write (pseudocode):
// ```ts
// import { BatchEmitGate } from "./core/BatchEmitGate";
// ```
import dev.monochromatic.musicplayer.core.LibraryDeltaGate
import dev.monochromatic.musicplayer.core.TrackSignatures

// What:     Imports the top-level `Int` constant `LIBRARY_BATCH_SIZE` (the shared streaming
//           threshold, around 128) from `core`.
// Why:      The gate is constructed with this threshold, so both sources stream on the same
//           policy.
//
// In TS you'd write (pseudocode):
// ```ts
// import { LIBRARY_BATCH_SIZE } from "./core/BatchEmitGate";
// ```
import dev.monochromatic.musicplayer.core.LIBRARY_BATCH_SIZE

// What:     Imports `Dispatchers`, a Kotlin coroutines object that names the thread pools
//           coroutines can run on. We use `Dispatchers.IO`, the pool meant for blocking
//           input/output work (file/network/database reads).
// Why:      The `withContext(Dispatchers.IO)` call below moves the cursor reading off the
//           main/UI thread so it never blocks the screen.
//
// In TS you'd write (pseudocode):
// ```ts
// // no equivalent — JS is single-threaded; picture a Worker pool named "IO"
// ```
import kotlinx.coroutines.Dispatchers

// What:     Imports `withContext`, a coroutine function. `withContext(dispatcher) { ... }`
//           runs the block on the given dispatcher (thread pool) and suspends the caller
//           until it finishes, returning the block's value.
// Why:      It is how `query` runs its body on `Dispatchers.IO` and still returns a value
//           to the caller as if it were a normal function.
//
// In TS you'd write (pseudocode):
// ```ts
// import { withContext } from "kotlinx/coroutines"; // ~ await runOnWorker(fn)
// ```
import kotlinx.coroutines.withContext

// What:     `object MediaStoreSource { ... }` declares a singleton named `MediaStoreSource`.
//           In Kotlin, `object` (not `class`) means "there is exactly one instance, created
//           lazily on first use, and you call its members through the name directly"
//           (`MediaStoreSource.query(...)`), never with `new`.
// Why:      This source holds no per-instance state; it is a namespaced bag of one public
//           function plus helpers, so a single shared instance is exactly right.
// Gotcha:   `object` here is NOT a generic "any object" type like TS's `object`. It is
//           Kotlin's keyword for a compiler-managed singleton.
//
// In TS you'd write (pseudocode):
// ```ts
// export const MediaStoreSource = {
//   // ...members below...
// };
// ```
/**
 * Defines media store source object for this music-player component; the TypeScript-oriented notes above explain
 * its shared role.
 */
object MediaStoreSource {
    // What:     `private const val SOURCE_TAG: String = "MediaStoreSource"` declares a
    //           compile-time constant. `private` hides it outside this object; `const`
    //           means the value is known at compile time and inlined; `val` means it can
    //           never be reassigned; `: String` is the explicit type annotation.
    // Why:      It is the logcat category tag passed to `Log.i` so the verification run can
    //           grep the log for just this source's output.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const SOURCE_TAG: string = "MediaStoreSource";
    // ```
    /**
     * Defines source tag value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private const val SOURCE_TAG: String = "MediaStoreSource"

    /** Milliseconds in one second, because MediaStore reports the modification time in seconds. */
    private const val MILLIS_PER_SECOND: Long = 1000L

    // What:     `@Suppress("DEPRECATION")` is an annotation (metadata attached to the next
    //           declaration), not a comment. It tells the Kotlin compiler "do not warn me
    //           about using deprecated APIs inside `query`."
    // Why:      `query` reads the legacy `DATA` column on old Android versions; that column
    //           is deprecated under scoped storage but is the only folder-aware option pre
    //           API 29, so we deliberately silence the warning here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // @ts-expect-error / eslint-disable-next-line — silence deprecated-API warning
    // ```
    @Suppress("DEPRECATION")
    // What:     `suspend fun query(resolver, onBatch): List<Track> = ...` declares a function
    //           named `query` with two params: `resolver` (a `ContentResolver`) and the
    //           optional `onBatch`. `suspend` marks it a coroutine function that may pause
    //           and resume (so it can await background work); it must be called from another
    //           coroutine. `: List<Track>` is the return type, a read-only list of `Track`.
    //           The `=` form is an expression body: the function returns the value of the
    //           single expression on the right (here the `withContext { ... }` block).
    // Why:      This is the one public entry point: hand it a resolver, get back the device's
    //           music tracks. `suspend` lets it do IO without blocking the UI thread. When
    //           `onBatch` is supplied, the scan ALSO emits the tracks found since the previous
    //           batch, in the order found and not sorted, as it reads the cursor (the caller
    //           sorts, see `LibraryIngest`); when null (the default), the scan stays atomic
    //           and only the final sorted list returns.
    // Gotcha:   `List<Track>` is READ-ONLY (no `.add`); the mutable cousin is `MutableList`.
    //           A `suspend` function looks synchronous but can only be called from a coroutine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function query(
    //   resolver: ContentResolver,
    //   onBatch: ((batch: readonly Track[]) => Promise<void>) | null = null,
    // ): Promise<readonly Track[]> {
    //   return await runOnWorker(/* IO pool */ async () => {
    //     // ...body below...
    //   });
    // }
    // ```
    /**
     * Defines query behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    suspend fun query(
        // What:     `resolver: ContentResolver` is the resolver to read MediaStore through.
        // Why:      The cursor query runs on it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // resolver: ContentResolver,
        // ```
        resolver: ContentResolver,
        // What:     `onBatch: (suspend (List<Track>) -> Unit)? = null` is an OPTIONAL streaming
        //           callback. The type reads as "a SUSPENDING function taking a read-only
        //           `List<Track>` and returning `Unit`, OR null"; `= null` is the default, so
        //           existing callers that pass nothing get the old atomic scan.
        // Why:      It is how the scan STREAMS: each emitted batch is handed to this callback so
        //           the UI can repaint the partial library before the whole scan finishes.
        // Gotcha:   `suspend` is part of the function type, so a call to it is itself a
        //           suspension/await point; if a newer load cancels this scan there, a
        //           `CancellationException` propagates straight out of `query` (there is no
        //           catch here to swallow it).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // onBatch: ((batch: readonly Track[]) => Promise<void>) | null = null,
        // ```
        onBatch: (suspend (List<Track>) -> Unit)? = null,
    ): List<Track> = withContext(Dispatchers.IO) {
        // What:     `val gate: BatchEmitGate<Track> = BatchEmitGate(LIBRARY_BATCH_SIZE) { left, right ->
        //           compareByCodePoint(left.displayPath, right.displayPath) }`
        //           creates ONE gate for this scan. `BatchEmitGate(...)` is the constructor (no
        //           `new`); the first argument is the threshold, and the trailing lambda is
        //           SAM-converted to the `Comparator<Track>` it expects (a `(left, right) =>
        //           number` compare by display path).
        // Why:      A FRESH gate per call keeps each scan's running-total private, which matters
        //           because two scans (foreground load and the background peak sweep) can run at
        //           once; a shared gate would corrupt across them.
        // Gotcha:   Declared as a `val` LOCAL, never a field on this singleton object, for that
        //           concurrency reason.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const gate = new BatchEmitGate<Track>(
        //   LIBRARY_BATCH_SIZE,
        //   (left, right) => compareByCodePoint(left.displayPath, right.displayPath),
        // );
        // ```
        /**
         * Defines gate value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val gate: LibraryDeltaGate<Track> = LibraryDeltaGate(LIBRARY_BATCH_SIZE)
        // What:     `val hasRelativePath: Boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q`
        //           declares an unreassignable local `Boolean`. `Build.VERSION.SDK_INT` is the
        //           device's Android API level as an `Int`; `Build.VERSION_CODES.Q` is the
        //           constant `29` (Android 10). The `>=` compares two ints, giving `true` when
        //           the device is API 29 or newer.
        // Why:      API 29+ has the `RELATIVE_PATH` column (a real folder path); older versions
        //           do not. This flag gates every later "new vs old" branch in one place.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const hasRelativePath: boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q;
        // ```
        /**
         * Defines has relative path value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        val hasRelativePath: Boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q
        // What:     `val collection = if (hasRelativePath) { A } else { B }`. In Kotlin `if`
        //           is an EXPRESSION: each branch's last value becomes the result, and that
        //           result is assigned to `collection` (a `Uri`). Branch A asks MediaStore for
        //           the API-29+ "external volume" audio URI; branch B uses the legacy
        //           `EXTERNAL_CONTENT_URI`. No explicit type is written; Kotlin infers `Uri`.
        // Why:      The base URI we query (and later append row ids to) differs by API level;
        //           this picks the correct one once.
        // Gotcha:   Unlike TS, Kotlin's `if/else` returns a value, so this whole block is an
        //           assignment, not a control-flow side effect.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const collection = hasRelativePath
        //   ? MediaStore.Audio.Media.getContentUri(MediaStore.VOLUME_EXTERNAL)
        //   : MediaStore.Audio.Media.EXTERNAL_CONTENT_URI;
        // ```
        /**
         * Defines collection value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        val collection = mediaStoreCollection(hasRelativePath)
        // RELATIVE_PATH (API 29+) is the scoped-storage folder path; DATA is the legacy absolute path
        // kept only as the < API 29 fallback. Requesting RELATIVE_PATH on an older platform throws
        // "unknown column", so it is added conditionally.
        // What:     `val projection: Array<String> = buildList { ... }.toTypedArray()`.
        //           `buildList { ... }` is a Kotlin builder: inside the lambda you call `add(x)`
        //           on an implicit list and it returns the finished read-only `List<String>`.
        //           `.toTypedArray()` is a CONVERSION call that copies that list into a typed
        //           `Array<String>` (a fixed-size array), because the Android query API wants an
        //           `Array<String>` of column names, not a `List`. `: Array<String>` is the
        //           explicit element type.
        // Why:      We build the list of column names to read, adding the path column
        //           conditionally so we never request a column the platform lacks.
        // Gotcha:   The `buildList` Gotcha (carried from the inline comment above): requesting
        //           `RELATIVE_PATH` on a pre-API-29 device throws "unknown column", which is the
        //           whole reason the `add` for the path column is wrapped in the `if` below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const projection: string[] = (() => {
        //   const cols: string[] = [];
        //   cols.push(MediaStore.Audio.Media._ID);
        //   cols.push(MediaStore.Audio.Media.DISPLAY_NAME);
        //   if (hasRelativePath) cols.push(MediaStore.Audio.Media.RELATIVE_PATH);
        //   else cols.push(MediaStore.Audio.Media.DATA);
        //   return cols;
        // })();
        // ```
        /**
         * Defines projection value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        val projection: Array<String> = mediaStoreProjection(hasRelativePath)
        // What:     `val selection = "${MediaStore.Audio.Media.IS_MUSIC} != 0"`. The
        //           `"${ ... }"` is a string TEMPLATE: the `${expr}` is replaced by the value
        //           of `expr` (here the column name string `IS_MUSIC`) inside the literal, so
        //           the result is a SQL-style WHERE clause like `is_music != 0`. No explicit
        //           type is written; Kotlin infers `String`.
        // Why:      It is the query's filter so only rows flagged as music (not ringtones,
        //           alarms, or notifications) come back.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const selection = `${MediaStore.Audio.Media.IS_MUSIC} != 0`;
        // ```
        /**
         * Defines selection value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        val selection = "${MediaStore.Audio.Media.IS_MUSIC} != 0"

        // What:     `val tracks: MutableList<Track> = mutableListOf()` declares a growable,
        //           mutable list of `Track`. `MutableList<Track>` is the explicit type (the
        //           writable cousin of the read-only `List<Track>`); `mutableListOf()` is the
        //           factory that creates an empty one.
        // Why:      We accumulate one `Track` per cursor row, which needs a list we can `.add`
        //           to as we iterate.
        // Gotcha:   The earlier `List<Track>` is read-only; this `MutableList<Track>` is the
        //           one you can append to. Picking the wrong one is a compile error in Kotlin,
        //           unlike TS where any array can be pushed to.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const tracks: Track[] = [];
        // ```
        /**
         * Defines tracks value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val tracks: MutableList<Track> = mutableListOf()
        // What:     `resolver.query(collection, projection, selection, null, null)?.use { cursor -> ... }`.
        //           `.query(...)` runs the database query and returns a `Cursor?` (a cursor that
        //           might be null if the query failed). The `?.` is a SAFE-CALL: if the left side
        //           is null, the whole `?.use { ... }` is skipped and evaluates to null; otherwise
        //           `.use { ... }` runs the lambda and GUARANTEES the cursor is closed afterward
        //           (even on exception). `{ cursor -> ... }` is a trailing lambda whose single
        //           parameter `cursor` is the non-null cursor. The two `null`s are the unused
        //           `selectionArgs` and `sortOrder` query arguments.
        // Why:      We read every matching row from the cursor; `use` ensures the native cursor
        //           handle is always released, and `?.` makes a null cursor a no-op instead of a
        //           crash.
        // Gotcha:   `?.` short-circuits the ENTIRE chained call on null (TS optional chaining works
        //           the same), and `use {}` is Kotlin's resource-closing helper (like a `using`
        //           block), not a plain method call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const cursor = resolver.query(collection, projection, selection, null, null);
        // if (cursor) {
        //   try {
        //     // ...body below, `cursor` is non-null here...
        //   } finally {
        //     cursor.close();
        //   }
        // }
        // ```
        resolver.query(collection, projection, selection, null, null)?.use { cursor ->
            // What:     `val idColumn: Int = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media._ID)`.
            //           `getColumnIndexOrThrow(name)` returns the integer position of that column in
            //           each row, or throws if the column is missing. `: Int` is the explicit type.
            // Why:      Reading a value from a row is done by column index, not by name, so we look
            //           up each column's index once before the loop.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const idColumn: number = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media._ID);
            // ```
            /**
             * Defines id column value for this music-player component; the TypeScript-oriented notes above
             * explain its source and use.
             */
            val idColumn: Int = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media._ID)
            /** Column of the file size, which with the modification time tells an edited file from a measured one. */
            val sizeColumn: Int = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.SIZE)
            /** Column of the modification time in seconds. */
            val modifiedColumn: Int = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DATE_MODIFIED)
            // What:     `val nameColumn: Int = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DISPLAY_NAME)`
            //           looks up the column index of the file-name column, throwing if absent.
            // Why:      We need the file-name column's position to read each row's name.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const nameColumn: number = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DISPLAY_NAME);
            // ```
            /**
             * Defines name column value for this music-player component; the TypeScript-oriented notes above
             * explain its source and use.
             */
            val nameColumn: Int = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DISPLAY_NAME)
            // What:     `val pathColumn: Int = if (hasRelativePath) { ... } else { ... }`. Another
            //           `if` USED AS AN EXPRESSION: whichever branch runs, its
            //           `getColumnIndexOrThrow` result (an `Int`) is assigned to `pathColumn`. On
            //           API 29+ we resolve the `RELATIVE_PATH` index; otherwise the `DATA` index.
            // Why:      The path column we request differs by API level (matching the projection
            //           branch above), so its index must be looked up under the same branch.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const pathColumn: number = hasRelativePath
            //   ? cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.RELATIVE_PATH)
            //   : cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DATA);
            // ```
            /**
             * Defines path column value for this music-player component; the TypeScript-oriented notes above
             * explain its source and use.
             */
            val pathColumn: Int = if (hasRelativePath) {
                // What:     `cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.RELATIVE_PATH)`
                //           returns the index of the relative-folder-path column, throwing if it is
                //           missing. As the branch's last expression it becomes the branch's value.
                // Why:      Resolve the `RELATIVE_PATH` index on API 29+ for later per-row reads.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.RELATIVE_PATH)
                // ```
                cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.RELATIVE_PATH)
            } else {
                // What:     `cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DATA)` returns the
                //           index of the legacy absolute-path column, throwing if missing. As the
                //           branch's last expression it becomes the branch's value.
                // Why:      Resolve the `DATA` index pre-API-29 for later per-row reads.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DATA)
                // ```
                cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DATA)
            }
            // What:     `while (cursor.moveToNext()) { ... }`. `moveToNext()` advances the cursor to
            //           the next row and returns `true` while a row exists, `false` once exhausted.
            //           The loop body reads the current row.
            // Why:      Standard cursor iteration: process every matching row exactly once.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // while (cursor.moveToNext()) {
            //   // ...read current row...
            // }
            // ```
            while (cursor.moveToNext()) {
                // What:     `val name: String = mediaStoreTrackName(cursor.getString(nameColumn))
                //           ?: continue`. `getString(index)` returns a nullable `String?`; the
                //           helper returns null for missing names and AppleDouble sidecars. The `?:`
                //           Elvis operator skips to the next row when that cleaned name is null.
                // Why:      A row with no file name is unusable, and a `._song.mp3` sidecar is not
                //           playable music even when MediaStore indexed it.
                // Gotcha:   `?: continue` is Elvis with a control-flow right side, not a ternary.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const name = mediaStoreTrackName(cursor.getString(nameColumn));
                // if (name == null) continue;
                // ```
                /**
                 * Defines name value for this music-player component; the TypeScript-oriented notes above
                 * explain its source and use.
                 */
                val name: String = mediaStoreTrackName(cursor.getString(nameColumn)) ?: continue
                // What:     `val id: Long = cursor.getLong(idColumn)` reads the `_ID` value as a
                //           `Long`, a 64-bit signed integer. `: Long` is the explicit type.
                // Why:      MediaStore row ids are 64-bit, and the URI builder below
                //           (`ContentUris.withAppendedId`) takes a `Long`, so we read it as one.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const id: number = cursor.getLong(idColumn); // 64-bit row id, fits in JS number
                // ```
                /**
                 * Defines id value for this music-player component; the TypeScript-oriented notes above explain
                 * its source and use.
                 */
                val id: Long = cursor.getLong(idColumn)
                // What:     `val rawPath: String? = cursor.getString(pathColumn)`. The `String?` type
                //           (note the trailing `?`) means "a `String` OR null"; we keep the null here
                //           rather than skipping, because a missing path is handled gracefully by the
                //           display-path helper below.
                // Why:      Some rows legitimately have no folder path; we want to fall back to the
                //           bare name, not drop the track, so we preserve the nullability.
                // Gotcha:   The trailing `?` on the TYPE (`String?`) marks nullability; do not confuse
                //           it with the `?.` safe-call or `?:` Elvis operators.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const rawPath: string | null = cursor.getString(pathColumn);
                // ```
                /**
                 * Defines raw path value for this music-player component; the TypeScript-oriented notes above
                 * explain its source and use.
                 */
                val rawPath: String? = cursor.getString(pathColumn)
                // What:     `val displayPath: String = displayPathOf(rawPath = rawPath, name = name, isRelative =
                //           hasRelativePath)`
                //           calls the private helper with NAMED ARGUMENTS (`paramName = value`), which
                //           label each argument at the call site. The result is a non-null `String`.
                // Why:      Turn the row's raw path and name into the folder-relative display string the
                //           UI shows and the sort uses.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const displayPath: string = displayPathOf(rawPath, name, hasRelativePath);
                // ```
                /**
                 * Defines display path value for this music-player component; the TypeScript-oriented notes
                 * above explain its source and use.
                 */
                val displayPath: String = displayPathOf(rawPath = rawPath, name = name, isRelative = hasRelativePath)
                // What:     `val uri: String = ContentUris.withAppendedId(collection, id).toString()`.
                //           `withAppendedId(baseUri, id)` builds a `Uri` pointing at this exact row by
                //           appending the numeric id; `.toString()` is a CONVERSION call that turns
                //           that `Uri` object into its `String` form (e.g. `content://media/...`).
                // Why:      The player stores and opens tracks by string URI, so we materialize the
                //           per-row `content://` URI as text here.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const uri: string = ContentUris.withAppendedId(collection, id).toString();
                // ```
                /**
                 * Defines uri value for this music-player component; the TypeScript-oriented notes above explain
                 * its source and use.
                 */
                val uri: String = ContentUris.withAppendedId(collection, id).toString()
                // What:     `tracks.add(Track(uri = uri, displayPath = displayPath))`. `Track(...)`
                //           constructs a `Track` value (Kotlin calls the constructor WITHOUT a `new`
                //           keyword); `uri = ...` / `displayPath = ...` are named constructor
                //           arguments. `tracks.add(...)` appends the new `Track` to the mutable list.
                // Why:      Record this row as a playable, displayable track in our accumulator.
                // Gotcha:   No `new` keyword: `Track(...)` IS the constructor call. The `name = value`
                //           pairs are named args, not assignments.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // tracks.push(new Track(uri, displayPath)); // or { uri, displayPath }
                // ```
                tracks.add(Track(uri = uri, displayPath = displayPath))
                TrackSignatures.record(
                    uri,
                    TrackSignatures.signatureOf(
                        sizeBytes = cursor.getLong(sizeColumn),
                        modifiedMs = cursor.getLong(modifiedColumn) * MILLIS_PER_SECOND,
                    ),
                )
                // What:     `if (onBatch != null) { val batch = gate.nextBatch(tracks); if (batch != null) {
                //           onBatch(batch) } }`
                //           streams a batch when one is requested. `gate.nextBatch(tracks)` returns a
                //           sorted-so-far `List<Track>?` (the batch to emit) or null (not yet). When
                //           non-null, `onBatch(batch)` is CALLED, and because `onBatch`'s type is
                //           `suspend`, this call AWAITS (it hops to the main thread to repaint).
                // Why:      Show the partial library as it grows; the gate keeps this from firing on
                //           every row (only once per `LIBRARY_BATCH_SIZE` new tracks). MediaStore reads
                //           one flat cursor, so the gate inside this loop is what makes that single
                //           large source stream at all.
                // Gotcha:   This is the scan's SUSPENSION POINT. If a newer load supersedes this one,
                //           the await throws `CancellationException`, which propagates straight out of
                //           `query` (there is no catch here to swallow it).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (onBatch !== null) {
                //   const batch = gate.nextBatch(tracks);
                //   if (batch !== null) await onBatch(batch);
                // }
                // ```
                emitLibraryBatchIfReady(onBatch = onBatch, gate = gate, tracks = tracks)
            }
        }
        // What:     `Log.i(SOURCE_TAG, "queried ${tracks.size} music tracks from MediaStore")`.
        //           `Log.i(tag, message)` writes an info-level log line. The message is a string
        //           template: `${tracks.size}` is replaced by the list's length (`size` is the
        //           Kotlin property; TS calls it `length`).
        // Why:      Emit the final count so an on-device verification can read back how many tracks
        //           the query produced.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // console.info(`[${SOURCE_TAG}] queried ${tracks.length} music tracks from MediaStore`);
        // ```
        emitLibraryTail(onBatch = onBatch, gate = gate, tracks = tracks)
        Log.i(SOURCE_TAG, "queried ${tracks.size} music tracks from MediaStore")
        // What:     `tracks.sortedWith { left, right -> compareByCodePoint(left.displayPath, right.displayPath) }`.
        //           `sortedWith { ... }` returns a NEW sorted `List<Track>` (it does not mutate
        //           `tracks`) ordered by the given comparator. `{ left, right -> ... }` is a trailing
        //           lambda comparator: it receives two `Track`s and returns the `Int` from
        //           `compareByCodePoint` (negative/zero/positive). With no trailing `;` this whole
        //           expression is the block's value, hence the function's return value.
        // Why:      Produce the final list ordered by display path in code-point order, matching the
        //           desktop's bytewise sort, and hand it back to the caller.
        // Gotcha:   This is the TAIL EXPRESSION of the `withContext` block: no `return` keyword and no
        //           `;`, yet its value is what `query` resolves to. `sortedWith` is non-mutating,
        //           unlike TS's in-place `Array.prototype.sort`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [...tracks].sort((left, right) =>
        //   compareByCodePoint(left.displayPath, right.displayPath),
        // );
        // ```
        tracks.sortedWith { left, right -> compareByCodePoint(left.displayPath, right.displayPath) }
    }

    // What:     `private fun mediaStoreCollection(hasRelativePath: Boolean)` chooses the base URI.
    // Why:      API 29+ uses the external-volume collection, older devices use the legacy URI.
    /** Base MediaStore collection URI for this device API level. */
    private fun mediaStoreCollection(hasRelativePath: Boolean): android.net.Uri = if (hasRelativePath) {
        MediaStore.Audio.Media.getContentUri(MediaStore.VOLUME_EXTERNAL)
    } else {
        MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
    }

    // What:     `private fun mediaStoreProjection(hasRelativePath: Boolean)` builds column names.
    // Why:      RELATIVE_PATH exists only on API 29+, while DATA is the legacy fallback.
    /** Column projection used for the MediaStore query. */
    private fun mediaStoreProjection(hasRelativePath: Boolean): Array<String> = buildList {
        add(MediaStore.Audio.Media._ID)
        add(MediaStore.Audio.Media.DISPLAY_NAME)
        add(MediaStore.Audio.Media.SIZE)
        add(MediaStore.Audio.Media.DATE_MODIFIED)
        if (hasRelativePath) {
            add(MediaStore.Audio.Media.RELATIVE_PATH)
        } else {
            add(MediaStore.Audio.Media.DATA)
        }
    }.toTypedArray()

    // What:     `internal fun mediaStoreTrackName(rawName: String?): String?` declares a
    //           module-visible helper. `String?` means a `String` or null; the return is also
    //           nullable so the caller can use one Elvis `?: continue` for every unusable row.
    // Why:      Keep the cursor loop to one jump statement while still dropping missing names and
    //           AppleDouble `._` sidecars before track construction.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function mediaStoreTrackName(rawName: string | null): string | null {
    //   // ...body below...
    // }
    // ```
    /** Clean MediaStore display name, or null when the row should be skipped. */
    internal fun mediaStoreTrackName(rawName: String?): String? {
        // What:     `if (rawName == null) { return null }` checks for a missing display name.
        // Why:      Rows without names cannot produce display paths or sidecar decisions.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (rawName === null) return null;
        // ```
        if (rawName == null) {
            return null
        }
        // What:     `if (isAppleDoubleSidecar(rawName)) { return null }` calls the shared
        //           AppleDouble predicate and turns a matching sidecar into a skipped row.
        // Why:      A `._song.mp3` row is metadata, not a playable track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (isAppleDoubleSidecar(rawName)) return null;
        // ```
        if (isAppleDoubleSidecar(rawName)) {
            return null
        }
        // What:     `return rawName` returns the non-null, non-sidecar display name.
        // Why:      The cursor loop can use this as the track name.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return rawName;
        // ```
        return rawName
    }

    // What:     `private fun displayPathOf(rawPath: String?, name: String, isRelative: Boolean): String = when { ... }`
    //           declares a private helper with three params: `rawPath` (nullable `String?`), `name`
    //           (`String`), `isRelative` (`Boolean`). It returns a non-null `String`. The `= when { ... }`
    //           is an expression body whose value is a `when` (Kotlin's multi-branch conditional).
    //           A `when { cond -> value }` with no subject is an if/else-if chain: the first branch whose
    //           condition is true supplies the result.
    // Why:      Centralizes the "how do we label this row" rule so the loop stays simple: relative
    //           folder + name on new APIs, the absolute path on old ones, or the bare name when there
    //           is no path at all.
    // Gotcha:   `when { }` (no parentheses/subject) is NOT a `switch` on a value; it is an ordered
    //           if/else-if chain. `String?` again means "string or null".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function displayPathOf(rawPath: string | null, name: string, isRelative: boolean): string {
    //   if (rawPath == null || rawPath === "") return name;
    //   if (isRelative) return `${rawPath.replace(/\/$/, "")}/${name}`;
    //   return rawPath;
    // }
    // ```
    /**
     * Defines display path of behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private fun displayPathOf(rawPath: String?, name: String, isRelative: Boolean): String = when {
        // What:     `rawPath.isNullOrEmpty() -> name`. `isNullOrEmpty()` is a Kotlin extension that
        //           safely returns `true` when `rawPath` is either null OR the empty string (it can be
        //           called even on a nullable value). The `-> name` makes this branch evaluate to
        //           `name`, the bare file name.
        // Why:      With no usable path, the only sensible label is the file name itself (a degenerate
        //           row).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (rawPath == null || rawPath === "") return name;
        // ```
        rawPath.isNullOrEmpty() -> name
        // What:     `isRelative -> "${rawPath.removeSuffix("/")}/$name"`. When `isRelative` is true,
        //           the branch value is a string template: `rawPath.removeSuffix("/")` drops a single
        //           trailing slash if present (so `Music/` becomes `Music`), then `/$name` appends a
        //           slash and the file name. `$name` is the shorthand template form for `${name}`.
        // Why:      `RELATIVE_PATH` is a trailing-slash folder; we join it to the file name with exactly
        //           one slash to form `<folder>/<name>`.
        // Gotcha:   At this branch `rawPath` is already known non-null (the null/empty case was handled
        //           above), so calling a method on it directly is safe.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (isRelative) return `${rawPath.endsWith("/") ? rawPath.slice(0, -1) : rawPath}/${name}`;
        // ```
        isRelative -> "${rawPath.removeSuffix("/")}/$name"
        // What:     `else -> rawPath`. The catch-all branch: when there is a path and it is NOT a
        //           relative folder (the pre-API-29 absolute `DATA` path), use it verbatim. As the
        //           branch value it becomes the function's return.
        // Why:      An absolute `DATA` path is already a full, usable display string, so we return it
        //           as is.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return rawPath;
        // ```
        else -> rawPath
    }
}
