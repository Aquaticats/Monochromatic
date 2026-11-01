// ============================================================================
// File summary (folds in the old KDoc that sat on `object SafTreeSource`)
// ============================================================================
//
// This file walks a user-chosen Storage Access Framework (SAF) tree and turns
// its audio files into the `Track` list the player consumes. It is the Android
// realization of the desktop's "point at one folder" model: the user grants a
// directory through `ACTION_OPEN_DOCUMENT_TREE`, and this scans that directory
// and every descendant, unlike `MediaStoreSource` which reads the device-wide
// audio collection. A file is enqueued by the same extension allowlist the
// desktop uses (`isAudioFile`), so the two sources agree on what counts as
// music; each yields a playable `content://.../tree/.../document/...` URI built
// from the document's opaque id (never from its name) and a tree-relative
// display path, and the rows are returned sorted by display path in Unicode
// code-point order, matching `MediaStoreSource`'s contract so the two are
// interchangeable through the same pagination.
//
// The walk is ITERATIVE with an explicit work stack (not recursion), so an
// arbitrarily deep tree cannot exhaust the call stack, and a visited-id set
// makes a misbehaving provider that reports a cycle terminate rather than loop.
// A single unreadable directory is logged and skipped instead of aborting the
// whole scan; a revoked grant on the root itself surfaces to the caller, which
// falls back to the device-wide source.
//
// The query runs on the IO thread pool because it is cursor I/O over a whole
// subtree. `compareByCodePoint`, `isAudioFile`, and `joinDisplayPath` are
// siblings in the `.core` package; `Track` is a sibling in this package.
// ============================================================================

// What:     `package dev.monochromatic.musicplayer` names the namespace this object
//           lives in, reachable elsewhere as
//           `dev.monochromatic.musicplayer.SafTreeSource`.
// Why:      So `LibrarySource` can call `SafTreeSource.query(...)`.
//
// In TS you'd write (pseudocode):
// ```ts
// // No `package` line in TS; the file path is the module identity.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.content.ContentResolver` pulls in `ContentResolver`, the
//           object you hand a query to in order to read a content provider.
// Why:      `query` takes a `ContentResolver` and runs `.query(...)` on it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ContentResolver } from "android/content";
// ```
import android.content.ContentResolver

// What:     `import android.net.Uri` pulls in `Uri`, Android's parsed URI type
//           (e.g. a `content://.../tree/...` document-tree URI).
// Why:      `query` takes the granted tree `Uri`, and per-document URIs are built
//           as `Uri` values below.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Uri } from "android/net";
// ```
import android.net.Uri

// What:     `import android.provider.DocumentsContract` pulls in `DocumentsContract`,
//           the SAF contract class of static helpers (`getTreeDocumentId`,
//           `buildChildDocumentsUriUsingTree`, `buildDocumentUriUsingTree`, ...).
// Why:      We use those helpers to walk the tree and build per-document URIs.
//
// In TS you'd write (pseudocode):
// ```ts
// import { DocumentsContract } from "android/provider";
// ```
import android.provider.DocumentsContract

// What:     `import android.provider.DocumentsContract.Document` imports the NESTED
//           type `Document` from inside `DocumentsContract` (the `.Document` is a
//           member access into the outer class, not a separate package). `Document`
//           holds the column-name constants (`COLUMN_DOCUMENT_ID`,
//           `COLUMN_DISPLAY_NAME`, `COLUMN_MIME_TYPE`) and `MIME_TYPE_DIR`.
// Why:      Importing the nested type lets us write `Document.COLUMN_*` directly
//           instead of `DocumentsContract.Document.COLUMN_*` everywhere.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Document } from "android/provider/DocumentsContract";
// ```
import android.provider.DocumentsContract.Document

// What:     `import android.util.Log` pulls in `Log`, Android's logger.
//           `Log.i(tag, msg)` writes info; `Log.w(tag, msg, throwable)` writes a
//           warning with an attached exception.
// Why:      We log the scanned-file count (info) and each skipped unreadable
//           directory (warning).
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log

// What:     `import dev.monochromatic.musicplayer.core.LibraryDeltaGate` imports the app's
//           own `BatchEmitGate` CLASS from the sibling `.core` package: the pure rule
//           that decides when a streaming scan has accumulated enough new tracks to emit
//           another sorted-so-far batch.
// Why:      The scan creates one gate per call and asks it, after each appended track,
//           whether to emit a batch to the screen.
//
// In TS you'd write (pseudocode):
// ```ts
// import { BatchEmitGate } from "./core/BatchEmitGate";
// ```
import dev.monochromatic.musicplayer.core.LibraryDeltaGate
import dev.monochromatic.musicplayer.core.TrackSignatures

// What:     `import dev.monochromatic.musicplayer.core.LIBRARY_BATCH_SIZE` imports the
//           app's own top-level `Int` constant (the shared streaming threshold, around
//           128) from `.core`.
// Why:      The gate is constructed with this threshold, so both sources stream on the
//           same policy.
//
// In TS you'd write (pseudocode):
// ```ts
// import { LIBRARY_BATCH_SIZE } from "./core/BatchEmitGate";
// ```
import dev.monochromatic.musicplayer.core.LIBRARY_BATCH_SIZE

// What:     `import dev.monochromatic.musicplayer.core.compareByCodePoint` imports the
//           app's own `compareByCodePoint` FUNCTION from the sibling `.core` package.
//           It compares two strings by Unicode code point and returns an `Int`
//           (negative/zero/positive), the standard comparator result.
// Why:      The final track list is sorted with it so the order matches the
//           desktop's bytewise path sort exactly.
//
// In TS you'd write (pseudocode):
// ```ts
// import { compareByCodePoint } from "./core";
// ```
import dev.monochromatic.musicplayer.core.compareByCodePoint

// What:     `import dev.monochromatic.musicplayer.core.isAudioFile` imports the app's
//           own `isAudioFile(name)` FUNCTION (returns `Boolean`) from `.core`: the
//           shared extension allowlist that decides whether a file name is music.
// Why:      We enqueue only files this allowlist accepts, so SAF and MediaStore
//           agree on what counts as music.
//
// In TS you'd write (pseudocode):
// ```ts
// import { isAudioFile } from "./core";
// ```
import dev.monochromatic.musicplayer.core.isAudioFile

// What:     `import dev.monochromatic.musicplayer.core.joinDisplayPath` imports the
//           app's own `joinDisplayPath(prefix, name)` FUNCTION (returns `String`)
//           from `.core`: it joins a folder prefix and a child name into one
//           tree-relative display path with exactly one slash.
// Why:      Each child's display path is built with it as the walk descends.
//
// In TS you'd write (pseudocode):
// ```ts
// import { joinDisplayPath } from "./core";
// ```
import dev.monochromatic.musicplayer.core.joinDisplayPath

// What:     `import kotlinx.coroutines.CancellationException` pulls in the special
//           exception a coroutine throws when it is cancelled. It is an `Exception`
//           subtype, but a SPECIAL one: catching and swallowing it would break
//           structured cancellation, so it must always be re-thrown.
// Why:      When a newer load supersedes this scan, the emit hop is cancelled and this
//           exception surfaces inside `scanDirectory`; we must rethrow it (not treat it
//           as an unreadable-directory failure) so the cancellation propagates.
// Gotcha:   In TS a cancelled async op is just a rejected promise; here cancellation is
//           a distinct exception class you are REQUIRED to re-throw, not absorb.
//
// In TS you'd write (pseudocode):
// ```ts
// import { CancellationException } from "kotlinx/coroutines";
// ```
import kotlinx.coroutines.CancellationException

// What:     `import kotlinx.coroutines.Dispatchers` pulls in `Dispatchers`, the
//           coroutines object naming the thread pools. We use `Dispatchers.IO` for
//           blocking input/output.
// Why:      `query` runs its cursor I/O on `Dispatchers.IO`, off the UI thread.
//
// In TS you'd write (pseudocode):
// ```ts
// // no equivalent — JS is single-threaded; picture a Worker pool named "IO"
// ```
import kotlinx.coroutines.Dispatchers

// What:     `import kotlinx.coroutines.withContext` pulls in `withContext`, the
//           coroutine function that runs a block on a given dispatcher, suspends the
//           caller until it finishes, and returns the block's value.
// Why:      It is how `query` runs its body on `Dispatchers.IO` and still returns a
//           value to the caller.
//
// In TS you'd write (pseudocode):
// ```ts
// import { withContext } from "kotlinx/coroutines"; // ~ await runOnWorker(fn)
// ```
import kotlinx.coroutines.withContext

// What:     `object SafTreeSource { ... }` declares a SINGLETON named `SafTreeSource`.
//           In Kotlin `object` (not `class`) means "exactly one instance, created
//           lazily, whose members you call through the name directly"
//           (`SafTreeSource.query(...)`), never with `new`.
// Why:      The source holds no per-instance state; it is a namespaced bag of one
//           public function plus helpers and constants, so a single shared instance
//           is right.
// Gotcha:   `object` here is NOT TS's structural `object` type; it is Kotlin's
//           keyword for a compiler-managed singleton.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SafTreeSource = {
//   // ...members below...
// };
// ```
/**
 * Defines saf tree source object for this music-player component; the TypeScript-oriented notes above explain
 * its shared role.
 */
object SafTreeSource {
    // What:     `private const val SOURCE_TAG: String = "SafTreeSource"` declares a
    //           private compile-time `String` constant (`const` = compile-time + inlined;
    //           `val` = never reassigned).
    // Why:      The logcat tag, so on-device verification can read the scanned-file
    //           count back under just this source's tag.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const SOURCE_TAG: string = "SafTreeSource";
    // ```
    /**
     * Defines source tag value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private const val SOURCE_TAG: String = "SafTreeSource"

    // What:     `private val PROJECTION: Array<String> = arrayOf( ... )` declares a
    //           private, read-only (`val`) field of type `Array<String>` (a FIXED-SIZE
    //           array of strings; sibling `List<String>` is the indexed read-only LIST
    //           interface, `MutableList<String>` the growable one). `arrayOf(a, b, c)`
    //           is the stdlib factory that builds an `Array` from the listed elements.
    // Why:      The Android `query` API wants the columns-to-read as an `Array<String>`
    //           (not a `List`), so we precompute the three columns once: the document
    //           id, the display name, and the mime type (which flags a directory).
    // Gotcha:   `Array<String>` is a true fixed-size array (what the SAF API demands),
    //           distinct from Kotlin's `List`/`MutableList` collection interfaces.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly PROJECTION: string[] = [
    //   Document.COLUMN_DOCUMENT_ID,
    //   Document.COLUMN_DISPLAY_NAME,
    //   Document.COLUMN_MIME_TYPE,
    // ];
    // ```
    /**
     * Defines projection value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private val PROJECTION: Array<String> = arrayOf(
        // What:     `Document.COLUMN_DOCUMENT_ID` is the column-name `String` constant for
        //           a row's opaque document id, listed as the first `arrayOf` element.
        // Why:      We read each child's id to build its per-document URI and to dedupe.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Document.COLUMN_DOCUMENT_ID,
        // ```
        Document.COLUMN_DOCUMENT_ID,
        // What:     `Document.COLUMN_DISPLAY_NAME` is the column-name constant for a
        //           row's display name (the file/folder name).
        // Why:      We read each child's name to build its display path and to test the
        //           audio allowlist.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Document.COLUMN_DISPLAY_NAME,
        // ```
        Document.COLUMN_DISPLAY_NAME,
        // What:     `Document.COLUMN_MIME_TYPE` is the column-name constant for a row's
        //           mime type; the special value `MIME_TYPE_DIR` marks a directory.
        // Why:      We read each child's mime type to tell a subdirectory (to recurse
        //           into) from a file (to maybe enqueue).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Document.COLUMN_MIME_TYPE,
        // ```
        Document.COLUMN_MIME_TYPE,
        Document.COLUMN_SIZE,
        Document.COLUMN_LAST_MODIFIED,
    )

    // What:     `private data class Frame(val documentId: String, val prefix: String)`
    //           declares a private NESTED "data class" named `Frame` with two read-only
    //           (`val`) `String` fields. `data` auto-generates structural
    //           `equals`/`hashCode`/`toString`/`copy`/`componentN` from the
    //           constructor properties (see how a data class works in `Track.kt`).
    // Why:      It is one pending directory in the depth-first walk: the document to
    //           list (`documentId`) and the already-sanitized folder path its children
    //           hang under (`prefix`, empty for the chosen root).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // type Frame = { readonly documentId: string; readonly prefix: string };
    // ```
    /**
     * Defines frame type for this music-player component; the TypeScript-oriented notes above explain its role.
     */
    private data class Frame(val documentId: String, val prefix: String)

    // What:     `private data class ChildRow(...)` stores one non-null SAF row.
    // Why:      Null ids or names are unusable, so cursor decoding happens once before row handling.
    /** Non-null child document row read from a SAF cursor. */
    private data class ChildRow(val id: String, val name: String, val mimeType: String?, val signature: Long)

    // What:     `suspend fun query(resolver, treeUri, onBatch): List<Track> = withContext(Dispatchers.IO) { ... }`
    //           declares the one public entry point. `suspend` marks it a coroutine
    //           function (it can await background work); three named params (`resolver`,
    //           `treeUri`, and the optional `onBatch`); `: List<Track>` is the
    //           read-only-list return type. The `= withContext(Dispatchers.IO) { ... }`
    //           expression body runs the trailing lambda on the IO pool and returns its
    //           value.
    // Why:      Scan `treeUri` and every directory beneath it, returning its audio
    //           tracks sorted by display path. The caller must hold a read grant for
    //           `treeUri`; listing the root without one throws (which the caller treats
    //           as a fall-back signal, not a crash). `suspend` keeps the cursor I/O off
    //           the UI thread. When `onBatch` is supplied, the scan also emits the tracks
    //           found since the previous batch, in the order found and NOT sorted, AS IT
    //           RUNS (the caller sorts, see `LibraryIngest`); when it is null (the
    //           default), the scan stays atomic and only the final sorted list is returned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function query(
    //   resolver: ContentResolver,
    //   treeUri: Uri,
    //   onBatch: ((batch: readonly Track[]) => Promise<void>) | null = null,
    // ): Promise<readonly Track[]> {
    //   return await runOnWorker(() => { // runs on the IO pool
    //     // ...body below...
    //   });
    // }
    // ```
    /**
     * Defines query behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    suspend fun query(
        // What:     `resolver: ContentResolver` is the resolver to query through.
        // Why:      Every directory listing goes through it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // resolver: ContentResolver,
        // ```
        resolver: ContentResolver,
        // What:     `treeUri: Uri` is the granted tree URI to scan recursively.
        // Why:      Child documents are addressed relative to this granted tree.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // treeUri: Uri,
        // ```
        treeUri: Uri,
        // What:     `onBatch: (suspend (List<Track>) -> Unit)? = null` is an OPTIONAL
        //           callback. The type `(suspend (List<Track>) -> Unit)?` reads as "a
        //           SUSPENDING function taking a read-only `List<Track>` and returning
        //           `Unit` (void), OR null". `suspend` inside the type means calling it can
        //           await (it hops to the main thread to touch the screen). `= null` is the
        //           default, so existing callers that pass nothing get the old atomic scan.
        // Why:      It is how the scan STREAMS: each emitted batch is handed to this
        //           callback so the UI can repaint the partial library before the whole
        //           scan finishes.
        // Gotcha:   The outer `?` makes the whole FUNCTION nullable (a function or null);
        //           `suspend` is part of the function type, meaning a call to it is itself a
        //           suspension/await point that can throw `CancellationException`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // onBatch: ((batch: readonly Track[]) => Promise<void>) | null = null,
        // ```
        onBatch: (suspend (List<Track>) -> Unit)? = null,
    ): List<Track> = withContext(Dispatchers.IO) {
        // What:     `val gate: BatchEmitGate<Track> = BatchEmitGate(LIBRARY_BATCH_SIZE) { left, right ->
        //           compareByCodePoint(left.displayPath, right.displayPath) }`
        //           creates ONE gate for this scan. `BatchEmitGate(...)` is the constructor
        //           (no `new` in Kotlin); the first argument is the threshold, and the
        //           trailing lambda `{ left, right -> ... }` is SAM-converted to the
        //           `Comparator<Track>` it expects (a `(left, right) => number` compare by
        //           display path).
        // Why:      A FRESH gate per call keeps each scan's running-total private, which
        //           matters because two scans (foreground load and the background peak
        //           sweep) can run at once; a shared gate would corrupt across them.
        // Gotcha:   Declared as a `val` LOCAL, never a field on this singleton object, for
        //           exactly that concurrency reason.
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
        // What:     `val rootDocumentId: String = DocumentsContract.getTreeDocumentId(treeUri)`
        //           declares a read-only `String` local `rootDocumentId`.
        //           `getTreeDocumentId(treeUri)` extracts the granted tree's root
        //           document id from the URI.
        // Why:      The walk must start from the root directory's document id.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rootDocumentId: string = DocumentsContract.getTreeDocumentId(treeUri);
        // ```
        /**
         * Defines root document id value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        val rootDocumentId: String = DocumentsContract.getTreeDocumentId(treeUri)
        // What:     `val pending: ArrayDeque<Frame> = ArrayDeque()` declares a read-only
        //           binding `pending` (the `val` locks the NAME, not the contents) of type
        //           `ArrayDeque<Frame>`, a DOUBLE-ENDED QUEUE (you can push/pop at either
        //           end). `ArrayDeque()` is its constructor (no `new` keyword). We use it
        //           as a LIFO STACK below via `addLast`/`removeLast`. Sibling collections:
        //           `MutableList<Frame>` (a plain growable list), or using the same deque
        //           as a FIFO queue via `removeFirst`.
        // Why:      The explicit work stack is what makes the walk ITERATIVE (no recursion),
        //           so an arbitrarily deep tree cannot exhaust the call stack.
        // Gotcha:   `val` makes `pending` non-rebindable but the deque CONTENTS are still
        //           mutable (`addLast`/`removeLast` work fine); Kotlin separates "rebind the
        //           name?" from "mutate the contents?".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pending: Frame[] = []; // used as a LIFO stack via push/pop
        // ```
        /**
         * Defines pending value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val pending: ArrayDeque<Frame> = ArrayDeque()
        // What:     `pending.addLast(Frame(documentId = rootDocumentId, prefix = ""))`
        //           pushes one item onto the stack. `addLast(x)` appends to the deque's
        //           tail (the "top" of our stack). `Frame(...)` constructs a `Frame` (no
        //           `new`); `documentId = ...` / `prefix = ...` are NAMED constructor
        //           arguments. The root's `prefix` is the empty string.
        // Why:      Seed the walk with the root directory frame.
        // Gotcha:   No `new` keyword: `Frame(...)` IS the constructor call; the `name = value`
        //           pairs are named args, not assignments.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // pending.push({ documentId: rootDocumentId, prefix: "" });
        // ```
        pending.addLast(Frame(documentId = rootDocumentId, prefix = ""))
        // What:     `val visited: MutableSet<String> = mutableSetOf()` declares a read-only
        //           binding `visited` of type `MutableSet<String>` (an editable SET of
        //           unique strings; sibling read-only `Set<String>`). `mutableSetOf()` is
        //           the factory for an empty one.
        // Why:      It records every document id already visited so a provider that reports
        //           the same document twice (a cycle) terminates rather than loops.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const visited: Set<string> = new Set();
        // ```
        /**
         * Defines visited value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val visited: MutableSet<String> = mutableSetOf()
        // What:     `val tracks: MutableList<Track> = mutableListOf()` declares a read-only
        //           binding `tracks` of type `MutableList<Track>` (the GROWABLE list
        //           cousin of the read-only `List<Track>`). `mutableListOf()` makes an
        //           empty one.
        // Why:      We accumulate one `Track` per audio file as the walk discovers them,
        //           which needs a list we can `.add` to.
        // Gotcha:   The return type is the read-only `List<Track>`; this `MutableList` is the
        //           one we append to. Picking the wrong one is a compile error in Kotlin.
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

        // What:     `while (pending.isNotEmpty()) { ... }` loops while the work stack still
        //           has frames. `isNotEmpty()` is the `List`/deque predicate (true when
        //           there is at least one element).
        // Why:      Drain the stack: process every directory frame until none remain.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (pending.length > 0) { ... }
        // ```
        while (pending.isNotEmpty()) {
            // What:     `val frame: Frame = pending.removeLast()` declares a read-only `Frame`
            //           local `frame` and POPS it off the stack. `removeLast()` removes and
            //           returns the deque's tail element, making the deque a LIFO stack
            //           (depth-first walk). Sibling `removeFirst()` would make it FIFO
            //           (breadth-first).
            // Why:      Take the next directory to list; LIFO gives a depth-first traversal.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const frame: Frame = pending.pop()!;
            // ```
            /**
             * Defines frame value for this music-player component; the TypeScript-oriented notes above explain
             * its source and use.
             */
            val frame: Frame = pending.removeLast()
            // What:     `if (!visited.add(frame.documentId)) { continue }`. `visited.add(x)`
            //           returns a `Boolean`: `true` if `x` was NEWLY inserted, `false` if it
            //           was ALREADY in the set. `!` negates it, so the condition is "this id
            //           was already visited." `continue` then skips to the next loop
            //           iteration. (Folds in the old inline note: a provider that reports a
            //           document twice, a cycle or a hardlink-like alias, must not loop
            //           forever; the first visit wins, later ones are dropped.)
            // Why:      Drop a re-reported document so a cyclic provider terminates.
            // Gotcha:   Kotlin's `Set.add` returns whether the element was new; JS's
            //           `Set.add` returns the set itself, so the one-line "add-or-skip"
            //           idiom does not translate directly.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (visited.has(frame.documentId)) continue;
            // visited.add(frame.documentId);
            // ```
            if (!visited.add(frame.documentId)) {
                // What:     `continue` skips the rest of this iteration and goes to the next.
                // Why:      Already visited this document; do not list it again.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // continue;
                // ```
                continue
            }
            // What:     `scanDirectory(resolver = ..., treeUri = ..., frame = ..., pending = ..., tracks = ..., onBatch
            //           = onBatch, gate = gate)`
            //           calls the private helper with NAMED ARGUMENTS (`paramName = value`),
            //           which label each argument at the call site. It lists one directory:
            //           pushing subdirectories onto `pending`, appending audio files to
            //           `tracks`, and (when `onBatch` is non-null) emitting a streamed batch
            //           through the shared `gate` after each appended file.
            // Why:      Do the actual per-directory listing for this frame, threading the
            //           SAME `onBatch` and `gate` through every directory so the running
            //           total and emit cadence span the whole scan.
            // Gotcha:   `scanDirectory` is now a `suspend` call (it can emit, which awaits);
            //           that is legal here because `query`'s body already runs in a coroutine.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // await scanDirectory(resolver, treeUri, frame, pending, tracks, onBatch, gate);
            // ```
            scanDirectory(
                resolver = resolver,
                treeUri = treeUri,
                frame = frame,
                pending = pending,
                tracks = tracks,
                onBatch = onBatch,
                gate = gate,
            )
        }

        // What:     `Log.i(SOURCE_TAG, "scanned ${tracks.size} audio files under $treeUri")`
        //           writes an info log line. The message is a string template:
        //           `${tracks.size}` is the list length (`.size` is Kotlin's `.length`),
        //           and `$treeUri` is the URI's string form.
        // Why:      Emit the final count so on-device verification can read it back.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // console.info(`[${SOURCE_TAG}] scanned ${tracks.length} audio files under ${treeUri}`);
        // ```
        emitLibraryTail(onBatch = onBatch, gate = gate, tracks = tracks)
        Log.i(SOURCE_TAG, "scanned ${tracks.size} audio files under $treeUri")
        // What:     `tracks.sortedWith { left, right -> compareByCodePoint(left.displayPath, right.displayPath) }`
        //           is the TAIL EXPRESSION of the `withContext` lambda (no trailing `;`), so
        //           its value is what `query` returns. `sortedWith { ... }` returns a NEW
        //           sorted `List<Track>` (it does NOT mutate `tracks`) ordered by the given
        //           comparator. `{ left, right -> ... }` is a trailing-lambda comparator: it
        //           receives two `Track`s and returns the `Int` from `compareByCodePoint`
        //           (negative/zero/positive).
        // Why:      Produce the final list ordered by display path in code-point order,
        //           matching the desktop's bytewise sort and `MediaStoreSource`'s contract.
        // Gotcha:   This is the TAIL EXPRESSION (no `return`, no `;`), and `sortedWith` is
        //           NON-mutating, unlike TS's in-place `Array.prototype.sort`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [...tracks].sort((left, right) =>
        //   compareByCodePoint(left.displayPath, right.displayPath),
        // );
        // ```
        tracks.sortedWith { left, right -> compareByCodePoint(left.displayPath, right.displayPath) }
    }

    // What:     `private fun signatureOfRow(cursor)` reads the size and modification time of the current row.
    // Why:      The analysis compares them with its ledger to find a replaced or edited file; a provider that reports
    //           neither gives zero, which leaves the file identified by its URI alone.
    /** Signature of the current cursor row's file, or 0 when the provider reports neither size nor time. */
    private fun signatureOfRow(cursor: android.database.Cursor): Long {
        /** Column of the size, or -1 when the provider has none. */
        val sizeColumn: Int = cursor.getColumnIndex(Document.COLUMN_SIZE)
        /** Column of the modification time, or -1 when the provider has none. */
        val modifiedColumn: Int = cursor.getColumnIndex(Document.COLUMN_LAST_MODIFIED)
        /** File size in bytes, or 0 when unknown. */
        val size: Long = if (sizeColumn >= 0 && !cursor.isNull(sizeColumn)) cursor.getLong(sizeColumn) else 0L
        /** Modification time in milliseconds, or 0 when unknown. */
        val modified: Long =
            if (modifiedColumn >= 0 && !cursor.isNull(modifiedColumn)) cursor.getLong(modifiedColumn) else 0L
        return TrackSignatures.signatureOf(sizeBytes = size, modifiedMs = modified)
    }

    // What:     `private fun childRowOf(...)` converts the current cursor row to a child record.
    // Why:      The scan loop can use one `continue` for unusable rows instead of one per column.
    /** Current cursor row as a non-null child record, or null when id/name is missing. */
    private fun childRowOf(
        cursor: android.database.Cursor,
        idColumn: Int,
        nameColumn: Int,
        mimeColumn: Int,
    ): ChildRow? {
        /** Raw document id from the cursor, or null when the provider returned an unusable row. */
        val childId: String? = cursor.getString(idColumn)
        /** Raw display name from the cursor, or null when the provider returned an unusable row. */
        val name: String? = cursor.getString(nameColumn)
        return if (childId == null || name == null) {
            null
        } else {
            ChildRow(
                id = childId,
                name = name,
                mimeType = cursor.getString(mimeColumn),
                signature = signatureOfRow(cursor),
            )
        }
    }

    // What:     `private suspend fun scanDirectory( resolver, treeUri, frame, pending, tracks, onBatch, gate ) { ... }`
    //           declares a private helper with seven named parameters and a block body
    //           (no return value, so the return type is the implicit `Unit`, Kotlin's
    //           "void"). `suspend` marks it a coroutine function because it may emit a
    //           streamed batch, which awaits.
    // Why:      List one directory's children: push subdirectories onto `pending`, append
    //           audio files to `tracks`, and (when streaming) emit a sorted-so-far batch
    //           through `gate`/`onBatch` after each appended file. An unreadable directory
    //           is logged and skipped so it cannot abort the rest of the walk; the root's
    //           own failure propagates (there is nothing left to scan). A coroutine
    //           cancellation (a newer load superseding this one) is RE-THROWN, not
    //           swallowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // async function scanDirectory(
    //   resolver: ContentResolver,
    //   treeUri: Uri,
    //   frame: Frame,
    //   pending: Frame[],
    //   tracks: Track[],
    //   onBatch: ((batch: readonly Track[]) => Promise<void>) | null,
    //   gate: LibraryDeltaGate<Track>,
    // ): Promise<void> { ... }
    // ```
    /**
     * Defines scan directory behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private suspend fun scanDirectory(
        // What:     `resolver: ContentResolver` is the resolver to query through.
        // Why:      Each child query goes through it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // resolver: ContentResolver,
        // ```
        resolver: ContentResolver,
        // What:     `treeUri: Uri` is the granted tree URI; the access leveraged for every
        //           child query and per-document URI built below.
        // Why:      Child documents are addressed relative to the granted tree.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // treeUri: Uri,
        // ```
        treeUri: Uri,
        // What:     `frame: Frame` is the directory being listed and its display-path
        //           prefix.
        // Why:      We list `frame.documentId` and hang children under `frame.prefix`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // frame: Frame,
        // ```
        frame: Frame,
        // What:     `pending: ArrayDeque<Frame>` is the shared work stack subdirectories are
        //           pushed onto.
        // Why:      Discovered subdirectories must go back on the stack to be listed later.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // pending: Frame[],
        // ```
        pending: ArrayDeque<Frame>,
        // What:     `tracks: MutableList<Track>` is the shared accumulator audio files are
        //           appended to.
        // Why:      Discovered audio files are added here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // tracks: Track[],
        // ```
        tracks: MutableList<Track>,
        // What:     `onBatch: (suspend (List<Track>) -> Unit)?` is the same optional
        //           streaming callback `query` received: a SUSPENDING function taking a
        //           read-only `List<Track>`, or null.
        // Why:      Threaded through so this directory's appended files can trigger a
        //           streamed batch; null means "do not stream".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // onBatch: ((batch: readonly Track[]) => Promise<void>) | null,
        // ```
        onBatch: (suspend (List<Track>) -> Unit)?,
        // What:     `gate: BatchEmitGate<Track>` is the shared, per-scan emit gate.
        // Why:      Using the SAME gate across every directory keeps the running total and
        //           emit cadence consistent over the whole scan, not reset per directory.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // gate: LibraryDeltaGate<Track>,
        // ```
        gate: LibraryDeltaGate<Track>,
    ) {
        // What:     `val childrenUri: Uri = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri,
        //           frame.documentId)`
        //           declares a read-only `Uri` local `childrenUri`. The helper builds the
        //           URI that, when queried, lists the CHILDREN of `frame.documentId` within
        //           the granted tree.
        // Why:      We must query this children-URI to enumerate the directory's contents.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const childrenUri: Uri = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, frame.documentId);
        // ```
        /**
         * Defines children uri value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        val childrenUri: Uri = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, frame.documentId)
        // What:     `try { ... } catch (cancellation: CancellationException) { ... }
        //           catch (expectedFailure: Exception) { ... }`
        //           is a TRY with TWO catch clauses, checked in order: the MORE SPECIFIC
        //           `CancellationException` first, then the broad `Exception`. Kotlin lets
        //           you TYPE each caught value.
        // Why:      A single unreadable directory (a transient provider error, a per-folder
        //           permission quirk) must be logged and skipped, not abort the whole walk.
        //           BUT now the listing has a real suspension point: when streaming, the
        //           emit hops to the main thread, and a newer load can CANCEL this scan
        //           there, surfacing a `CancellationException`. That must be re-thrown so
        //           the cancellation propagates; only a genuine provider failure is
        //           swallowed-and-skipped. (This corrects the old note that claimed the
        //           cursor read held no suspension point: the streamed emit now does.)
        // Gotcha:   Clause ORDER matters: `CancellationException` is itself an `Exception`,
        //           so the specific clause must precede the broad one, or every
        //           cancellation would be wrongly absorbed as a directory failure.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try {
        //   // ...listing, possibly awaiting onBatch...
        // } catch (e) {
        //   if (e instanceof CancellationException) throw e;
        //   Log.w(SOURCE_TAG, `skipping unreadable directory ${frame.documentId}`, e);
        // }
        // ```
        try {
            // What:     `resolver.query(childrenUri, PROJECTION, null, null, null)?.use { cursor -> ... }`.
            //           `.query(...)` runs the children query and returns a `Cursor?`
            //           (nullable). `?.` is a SAFE-CALL: a null cursor skips the whole
            //           `?.use { ... }`; otherwise `.use { ... }` runs the trailing lambda and
            //           GUARANTEES the cursor is closed afterward. `{ cursor -> ... }` binds the
            //           non-null cursor. The three `null`s are the unused selection,
            //           selectionArgs, and sortOrder arguments.
            // Why:      Read every child row safely, always releasing the cursor handle.
            // Gotcha:   `?.` short-circuits on null (like TS optional chaining); `use {}` is
            //           Kotlin's resource-closing helper, not a plain method.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // resolver.query(childrenUri, PROJECTION, null, null, null)?.use((cursor) => { ... });
            // ```
            resolver.query(childrenUri, PROJECTION, null, null, null)?.use { cursor ->
                // What:     `val idColumn: Int = cursor.getColumnIndexOrThrow(Document.COLUMN_DOCUMENT_ID)`
                //           looks up the integer POSITION of the document-id column in each
                //           row (throws if missing). `: Int` is the 32-bit index type (a
                //           column index never approaches the `Int` ceiling, and the Cursor
                //           API takes `Int`, so `Long` would be wrong here).
                // Why:      Rows are read by column index, so we resolve each index once
                //           before the loop.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const idColumn: number = cursor.getColumnIndexOrThrow(Document.COLUMN_DOCUMENT_ID);
                // ```
                /**
                 * Defines id column value for this music-player component; the TypeScript-oriented notes above
                 * explain its source and use.
                 */
                val idColumn: Int = cursor.getColumnIndexOrThrow(Document.COLUMN_DOCUMENT_ID)
                // What:     `val nameColumn: Int = cursor.getColumnIndexOrThrow(Document.COLUMN_DISPLAY_NAME)`
                //           resolves the display-name column's index (throws if missing).
                // Why:      Needed to read each child's name.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const nameColumn: number = cursor.getColumnIndexOrThrow(Document.COLUMN_DISPLAY_NAME);
                // ```
                /**
                 * Defines name column value for this music-player component; the TypeScript-oriented notes above
                 * explain its source and use.
                 */
                val nameColumn: Int = cursor.getColumnIndexOrThrow(Document.COLUMN_DISPLAY_NAME)
                // What:     `val mimeColumn: Int = cursor.getColumnIndexOrThrow(Document.COLUMN_MIME_TYPE)`
                //           resolves the mime-type column's index (throws if missing).
                // Why:      Needed to tell a subdirectory from a file.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const mimeColumn: number = cursor.getColumnIndexOrThrow(Document.COLUMN_MIME_TYPE);
                // ```
                /**
                 * Defines mime column value for this music-player component; the TypeScript-oriented notes above
                 * explain its source and use.
                 */
                val mimeColumn: Int = cursor.getColumnIndexOrThrow(Document.COLUMN_MIME_TYPE)
                // What:     `while (cursor.moveToNext()) { ... }`. `moveToNext()` advances the
                //           cursor to the next row and returns `true` while a row exists,
                //           `false` once exhausted.
                // Why:      Standard cursor iteration: process every child row once.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // while (cursor.moveToNext()) { ... }
                // ```
                while (cursor.moveToNext()) {
                    // What:     `val childId: String = cursor.getString(idColumn) ?: continue`.
                    //           `getString(index)` returns a `String?` (nullable). The ELVIS
                    //           `?:` uses the left value when non-null, else evaluates the
                    //           right side `continue` (skip to the next row). So `childId` is a
                    //           guaranteed non-null `String`, or we move on.
                    // Why:      A child with no id is unusable; drop it and keep scanning.
                    // Gotcha:   `?: continue` is Elvis with a control-flow right side (legal
                    //           because `continue` has the "never" type), NOT a ternary.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const maybeId = cursor.getString(idColumn);
                    // if (maybeId == null) continue;
                    // const childId: string = maybeId;
                    // ```
                    /**
                     * Defines child id value for this music-player component; the TypeScript-oriented notes
                     * above explain its source and use.
                     */
                    val child: ChildRow = childRowOf(
                        cursor = cursor,
                        idColumn = idColumn,
                        nameColumn = nameColumn,
                        mimeColumn = mimeColumn,
                    ) ?: continue
                    // What:     `val name: String = cursor.getString(nameColumn) ?: continue`.
                    //           Same Elvis-or-skip shape as `childId`: a non-null name, or
                    //           `continue` to the next row.
                    // Why:      A child with no name is unusable; skip it.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const maybeName = cursor.getString(nameColumn);
                    // if (maybeName == null) continue;
                    // const name: string = maybeName;
                    // ```
                    /**
                     * Defines name value for this music-player component; the TypeScript-oriented notes above
                     * explain its source and use.
                     */
                    // What:     `val mimeType: String? = cursor.getString(mimeColumn)` declares a
                    //           NULLABLE `String?` (the trailing `?` = "a `String` OR null"). We
                    //           KEEP the null here (no `?: continue`) because a null mime is
                    //           simply "not a directory."
                    // Why:      A missing mime type just means "treat as a file," so we preserve
                    //           the nullability rather than dropping the row.
                    // Gotcha:   The trailing `?` is on the TYPE (`String?` = nullable), not the
                    //           `?.`/`?:` operators.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const mimeType: string | null = cursor.getString(mimeColumn);
                    // ```
                    /**
                     * Defines mime type value for this music-player component; the TypeScript-oriented notes
                     * above explain its source and use.
                     */
                    // What:     `val childPath: String = joinDisplayPath(frame.prefix, name)`
                    //           declares a read-only `String` by joining this directory's
                    //           prefix and the child's name into one tree-relative display path.
                    // Why:      Every child (folder or file) needs its display path computed for
                    //           grouping/sorting.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const childPath: string = joinDisplayPath(frame.prefix, name);
                    // ```
                    /**
                     * Defines child path value for this music-player component; the TypeScript-oriented notes
                     * above explain its source and use.
                     */
                    val childPath: String = joinDisplayPath(frame.prefix, child.name)
                    // What:     `if (mimeType == Document.MIME_TYPE_DIR) { ... } else if (isAudioFile(name)) { ... }`
                    //           branches on the child kind. `==` is value equality; comparing a
                    //           `String?` (`mimeType`) to a `String` constant is NULL-SAFE in
                    //           Kotlin (a null `mimeType` simply is not equal, no crash).
                    //           `isAudioFile(name)` is the allowlist predicate.
                    // Why:      A directory is pushed for later listing; an audio file is
                    //           enqueued; anything else is ignored.
                    // Gotcha:   Kotlin's `==` on a nullable left side is null-safe (yields false
                    //           if null); a raw equality on a possibly-null value is fine here.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // if (mimeType === Document.MIME_TYPE_DIR) { ... }
                    // else if (isAudioFile(name)) { ... }
                    // ```
                    if (child.mimeType == Document.MIME_TYPE_DIR) {
                        // What:     `pending.addLast(Frame(documentId = childId, prefix = childPath))`
                        //           pushes a new directory frame onto the work stack.
                        //           `addLast(x)` appends to the deque tail (stack top);
                        //           `Frame(...)` constructs the frame with named args.
                        // Why:      Schedule this subdirectory to be listed in a later loop
                        //           iteration (the iterative descent).
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // pending.push({ documentId: childId, prefix: childPath });
                        // ```
                        pending.addLast(Frame(documentId = child.id, prefix = childPath))
                    } else if (isAudioFile(child.name)) {
                        // What:     `val documentUri: Uri = DocumentsContract.buildDocumentUriUsingTree(treeUri,
                        //           childId)`
                        //           builds the playable per-document URI from the OPAQUE document
                        //           id (never from the name), within the granted tree.
                        // Why:      The engine needs a stable `content://` URI to open the file;
                        //           deriving it from the id keeps it valid even if the name has
                        //           odd characters.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // const documentUri: Uri = DocumentsContract.buildDocumentUriUsingTree(treeUri, childId);
                        // ```
                        /**
                         * Defines document uri value for this music-player component; the TypeScript-oriented
                         * notes above explain its source and use.
                         */
                        val documentUri: Uri = DocumentsContract.buildDocumentUriUsingTree(treeUri, child.id)
                        // What:     `tracks.add(Track(uri = documentUri.toString(), displayPath = childPath))`
                        //           appends a new `Track` to the accumulator. `Track(...)`
                        //           constructs it (no `new`) with named args; `documentUri.toString()`
                        //           is a TYPE-CONVERSION call turning the `Uri` into its `String`
                        //           form (the player stores URIs as strings).
                        // Why:      Record this audio file as a playable, displayable track.
                        // Gotcha:   No `new` keyword: `Track(...)` IS the constructor; the
                        //           `name = value` pairs are named args.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // tracks.push({ uri: documentUri.toString(), displayPath: childPath });
                        // ```
                        tracks.add(Track(uri = documentUri.toString(), displayPath = childPath))
                        TrackSignatures.record(documentUri.toString(), child.signature)
                        // What:     `if (onBatch != null) { val batch = gate.nextBatch(tracks); if (batch != null) {
                        //           onBatch(batch) } }`
                        //           streams a batch when one is requested. `gate.nextBatch(tracks)`
                        //           returns a sorted-so-far `List<Track>?` (the batch to emit) or
                        //           null (not yet). When non-null, `onBatch(batch)` is CALLED, and
                        //           because `onBatch`'s type is `suspend`, this call AWAITS (it hops
                        //           to the main thread to repaint).
                        // Why:      Show the partial library as it grows; the gate keeps this from
                        //           firing on every file (only once per `LIBRARY_BATCH_SIZE` new
                        //           tracks). The check sits right after a track is appended, so the
                        //           emit cadence tracks real discovery even inside one huge folder.
                        // Gotcha:   This is the scan's SUSPENSION POINT. If a newer load supersedes
                        //           this one, the await throws `CancellationException` here, which
                        //           the outer catch RE-THROWS rather than swallowing.
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
            }
        } catch (cancellation: CancellationException) {
            // What:     `throw cancellation` re-throws the caught `CancellationException`
            //           unchanged, propagating it to the coroutine machinery instead of
            //           treating it as a directory failure. `cancellation` is the value bound
            //           by the `catch (cancellation: CancellationException)` clause.
            // Why:      A newer load (for example a folder pick during a scan) cancels this
            //           one through the streamed emit; swallowing the cancellation would let
            //           the superseded scan keep emitting stale batches over the new library.
            // Gotcha:   This clause MUST come before `catch (expectedFailure: Exception)`, since
            //           `CancellationException` is an `Exception`; otherwise the broad clause
            //           would catch it first and wrongly skip the directory.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (e instanceof CancellationException) throw e;
            // ```
            throw cancellation
        } catch (expectedFailure: Exception) {
            // What:     `Log.w(SOURCE_TAG, "skipping unreadable directory ${frame.documentId}", expectedFailure)`
            //           writes a WARNING log line with the thrown `failure` attached (the
            //           third `Throwable` argument records the stack trace). The message is a
            //           string template with `${frame.documentId}` interpolated.
            // Why:      A single directory we cannot read is logged and skipped, so it cannot
            //           abort the whole walk (the root's own failure propagates out of `query`
            //           instead, because there is nothing left to scan).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // console.warn(`[${SOURCE_TAG}] skipping unreadable directory ${frame.documentId}`, expectedFailure);
            // ```
            Log.w(SOURCE_TAG, "skipping unreadable directory ${frame.documentId}", expectedFailure)
        }
    }
}
