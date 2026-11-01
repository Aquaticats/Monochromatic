// Trash and Undo for one track, by route (decision D105). A picked folder is copied to the app cache, verified, and
//   then
// deleted; Undo recreates the document from the copy. The device-wide media library goes through Android's own trash
// request, and Undo is Android's own restore request. Nothing is deleted before its copy is verified.

// What:     `package dev.monochromatic.musicplayer` places the trash flow beside the player host that runs it.
// Why:      The host calls these functions with the controller, the feedback state, and the library in one package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.app.Activity` supplies the success result code of the Android trash request.
// Why:      The request reports success as RESULT_OK, and the callback compares against that constant.
//
// In TS you'd write (pseudocode):
// ```ts
// import { RESULT_OK } from "android/app/Activity";
// ```
import android.app.Activity
import android.app.PendingIntent
import android.content.ContentResolver
import android.content.Context
import android.content.IntentSender
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.DocumentsContract
import android.provider.DocumentsContract.Document
import android.provider.MediaStore
import android.util.Log
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.IntentSenderRequest
import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.staticCompositionLocalOf
import dev.monochromatic.musicplayer.core.TrackActionFailure
import dev.monochromatic.musicplayer.core.TrashRecord
import dev.monochromatic.musicplayer.core.TrashRoute
import dev.monochromatic.musicplayer.core.copyMatches
import dev.monochromatic.musicplayer.core.isDescendantDocumentId
import dev.monochromatic.musicplayer.core.mimeTypeFor
import java.io.File
import java.io.FileNotFoundException
import java.io.IOException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

// What:     `private const val TRASH_CACHE_DIR: String` names the folder under the app cache that holds trashed copies.
// Why:      A single named folder lets the startup sweep remove every copy in one step.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_CACHE_DIR = "trash";
// ```
/** Name of the folder under the app cache directory that holds the copies of trashed files. */
private const val TRASH_CACHE_DIR: String = "trash"

// What:     `private const val TRASH_FLOW_TAG: String` names the log lines of this file.
// Why:      The tag separates trash decisions from the rest of the app in a device log.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_FLOW_TAG = "TrashFlow";
// ```
/** Tag of the log lines written by the trash flow. */
private const val TRASH_FLOW_TAG: String = "TrashFlow"

// What:     `internal val LocalTrashMediaStoreRequest` carries the activity's trash request launcher down the tree.
// Why:      The launcher must be registered by the activity, and a composition local reaches the host without a
//           parameter chain.
//
// In TS you'd write (pseudocode):
// ```ts
// const LocalTrashMediaStoreRequest = createContext<TrashMediaStoreRequest | null>(null);
// ```
/** Composition local that holds the media-library trash request, or null when the activity did not provide one. */
internal val LocalTrashMediaStoreRequest: ProvidableCompositionLocal<TrashMediaStoreRequest?> =
    staticCompositionLocalOf { null }

// What:     `internal class TrashMediaStoreRequest` launches Android's trash request and routes its result to one
//           callback.
// Why:      The activity result arrives later in another callback, so the pending outcome is held until it returns.
//
// In TS you'd write (pseudocode):
// ```ts
// class TrashMediaStoreRequest { launch(sender: IntentSender, onResult: (ok: boolean) => void): void; deliver(code:
//   number): void; }
// ```
/** Starts an Android trash or restore request and delivers its success to the callback that asked for it. */
internal class TrashMediaStoreRequest(
    // What:     `private val launcher` is the activity result launcher that starts the system confirmation.
    // Why:      The launcher is registered once by the activity, and this class only calls it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly launcher: Launcher<IntentSenderRequest>;
    // ```
    /** Activity result launcher registered by the activity for intent sender requests. */
    private val launcher: ActivityResultLauncher<IntentSenderRequest>,
) {
    // What:     `private var waiting` holds the callback of the request in flight.
    // Why:      Only one request runs at a time, so one slot is enough to route the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private waiting: ((ok: boolean) => void) | null = null;
    // ```
    /** Callback of the request in flight, or null when no request is waiting for a result. */
    private var waiting: ((Boolean) -> Unit)? = null

    // What:     `fun launch(...)` starts one system request and remembers where its result goes.
    // Why:      The callback runs only when this request returns, so the outcome is never delivered to a stale caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // launch(sender: IntentSender, onResult: (ok: boolean) => void): void { ... }
    // ```
    /** Starts the system request for one intent sender and stores the callback that receives whether it succeeded. */
    fun launch(intentSender: IntentSender, onResult: (Boolean) -> Unit) {
        Log.i(LOG_TAG, "$TRASH_FLOW_TAG media store request launched")
        waiting = onResult
        launcher.launch(IntentSenderRequest.Builder(intentSender).build())
    }

    // What:     `fun deliver(resultCode: Int)` passes the activity result to the waiting callback.
    // Why:      A result with no waiting request is logged and dropped, so a stray result cannot change the library.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // deliver(resultCode: number): void { const cb = this.waiting; this.waiting = null; cb?.(resultCode === RESULT_OK);
    //   }
    // ```
    /** Hands the result code of the system request to the waiting callback as success or not. */
    fun deliver(resultCode: Int) {
        // What:     `val callback` takes the waiting callback and clears the slot.
        // Why:      Clearing first means a second delivery cannot run the same callback again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const callback = this.waiting; this.waiting = null;
        // ```
        /** Callback waiting for this result, or null when no request was in flight. */
        val callback: ((Boolean) -> Unit)? = waiting
        waiting = null
        if (callback == null) {
            Log.w(LOG_TAG, "$TRASH_FLOW_TAG media store result with no request code=$resultCode")
            return
        }
        Log.i(LOG_TAG, "$TRASH_FLOW_TAG media store result code=$resultCode")
        callback(resultCode == Activity.RESULT_OK)
    }
}

// What:     `internal object TrashUndoStore` holds the one trash record whose Undo message is visible.
// Why:      The record must survive a configuration change and be reachable from the Undo action and the cleanup check,
//           so it lives at process level, and every access is synchronized.
//
// In TS you'd write (pseudocode):
// ```ts
// const TrashUndoStore = { pending: null as TrashRecord | null };
// ```
/** Process-level holder of the pending trash record and of the one-time startup sweep. */
internal object TrashUndoStore {
    // What:     `private var pending: TrashRecord?` is the record whose Undo is offered now.
    // Why:      Only one Undo message exists at a time, so one slot holds the record it names.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let pending: TrashRecord | null = null;
    // ```
    /** Record whose Undo message is visible, or null when no Undo is offered. */
    private var pending: TrashRecord? = null

    // What:     `private var sweptThisProcess: Boolean` records whether the startup sweep already ran.
    // Why:      A configuration change recomposes the host, and the sweep must not delete copies of a live Undo.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let sweptThisProcess = false;
    // ```
    /** True once the startup sweep has run in this process. */
    private var sweptThisProcess: Boolean = false

    // What:     `@Synchronized fun replace(record: TrashRecord?): TrashRecord?` sets the pending record and returns the
    //           old one.
    // Why:      The caller removes the old record's copy, because its Undo message is replaced by the new one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // replace(record: TrashRecord | null): TrashRecord | null { const old = pending; pending = record; return old; }
    // ```
    /** Stores the new pending record and returns the one it replaced, or null when none was pending. */
    @Synchronized
    fun replace(record: TrashRecord?): TrashRecord? {
        /** Record that was pending before this call. */
        val previous: TrashRecord? = pending
        pending = record
        return previous
    }

    // What:     `@Synchronized fun takeForRestore(messageId: String): TrashRecord?` removes the pending record when its
    //           message id matches.
    // Why:      The record leaves the store before its restore starts, so the cleanup check cannot delete the copy
    //           mid-restore.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // takeForRestore(messageId: string): TrashRecord | null { ... }
    // ```
    /** Removes and returns the pending record whose message id matches, or null when the id matches nothing pending. */
    @Synchronized
    fun takeForRestore(messageId: String): TrashRecord? {
        /** Record that is pending right now, or null when none is. */
        val current: TrashRecord? = pending
        if (current == null || current.messageId != messageId) {
            return null
        }
        pending = null
        return current
    }

    // What:     `@Synchronized fun dropIfHidden(visibleIds: Set<String>): TrashRecord?` removes the pending record when
    //           its message is no longer visible.
    // Why:      An expired, dismissed, or replaced Undo message means its copy will never be needed, so the record is
    //           released.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // dropIfHidden(visible: Set<string>): TrashRecord | null { ... }
    // ```
    /** Removes and returns the pending record when its message id is not among the visible ids, otherwise null. */
    @Synchronized
    fun dropIfHidden(visibleIds: Set<String>): TrashRecord? {
        /** Record that is pending right now, or null when none is. */
        val current: TrashRecord? = pending
        if (current == null || visibleIds.contains(current.messageId)) {
            return null
        }
        pending = null
        return current
    }

    // What:     `@Synchronized fun claimStartupSweep(): Boolean` returns true for the first caller in the process only.
    // Why:      The sweep deletes every cached copy, so it must run once per process and never while a record is live.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // claimStartupSweep(): boolean { if (swept) return false; swept = true; return true; }
    // ```
    /** Returns true the first time it is called in this process and false afterwards. */
    @Synchronized
    fun claimStartupSweep(): Boolean {
        if (sweptThisProcess) {
            return false
        }
        sweptThisProcess = true
        return true
    }
}

// What:     `internal fun removeCopy(record: TrashRecord)` deletes the cache copy of one trashed file.
// Why:      The copy is needed only while its Undo can run, so it is removed as soon as that window closes.
//
// In TS you'd write (pseudocode):
// ```ts
// function removeCopy(record: TrashRecord): void { fs.rmSync(record.copyPath, { force: true }); }
// ```
/** Deletes the cache copy of one trashed file and its folder, logging whether the deletion worked. */
internal fun removeCopy(record: TrashRecord) {
    /** Path of the cache copy, or null when the route kept none. */
    val path: String = record.copyPath ?: return
    /** Cache file that holds the bytes of the trashed file. */
    val file = File(path)
    if (!file.exists()) {
        Log.i(LOG_TAG, "$TRASH_FLOW_TAG copy already gone id=${record.id}")
        return
    }
    if (!file.delete()) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG copy could not be deleted id=${record.id}")
        return
    }
    file.parentFile?.delete()
    Log.i(LOG_TAG, "$TRASH_FLOW_TAG copy removed id=${record.id}")
}

// What:     `internal fun purgeTrashCopies(context: Context)` deletes every cached copy left by an earlier process.
// Why:      A copy from a previous run has no live Undo message, so it is removed at the first start of the process.
//
// In TS you'd write (pseudocode):
// ```ts
// function purgeTrashCopies(context: Context): void { fs.rmSync(join(context.cacheDir, "trash"), { recursive: true,
//   force: true }); }
// ```
/** Deletes the trash cache folder and everything in it, logging the outcome. */
internal fun purgeTrashCopies(context: Context) {
    /** Cache folder that holds every copy of a trashed file. */
    val folder = File(context.cacheDir, TRASH_CACHE_DIR)
    if (!folder.exists()) {
        return
    }
    /** Whether the cache folder was removed in full. */
    val removed: Boolean = folder.deleteRecursively()
    Log.i(LOG_TAG, "$TRASH_FLOW_TAG startup sweep removed=$removed")
}

// What:     `internal class TrashDocument` holds the provider facts the trash needs for one document.
// Why:      Name, MIME type, and size are read once, before anything is deleted, and Undo relies on exactly these
//           values.
//
// In TS you'd write (pseudocode):
// ```ts
// type TrashDocument = { name: string; mimeType: string | null; sizeBytes: number };
// ```
/** Name, MIME type, and size of one document as its provider reports them. */
internal data class TrashDocument(
    // What:     `val name: String` is the display name the provider reports.
    // Why:      Undo recreates the document under exactly this name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // name: string;
    // ```
    /** Display name the provider reports. */
    val name: String,
    // What:     `val mimeType: String?` is the MIME type the provider reports, or null when it reports none.
    // Why:      Undo passes this type to the provider, and a null value falls back to the audio table.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mimeType: string | null;
    // ```
    /** MIME type the provider reports, or null when it reports none. */
    val mimeType: String?,
    // What:     `val sizeBytes: Long` is the byte count the provider reports.
    // Why:      The copy must match this size before the original may be deleted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // sizeBytes: number;
    // ```
    /** Byte count the provider reports. */
    val sizeBytes: Long,
)

// What:     `internal fun queryDocument(...)` reads the name, MIME type, and size of one document.
// Why:      The trash decision needs these values, and a document that reports no size cannot be verified.
//
// In TS you'd write (pseudocode):
// ```ts
// function queryDocument(resolver, uri): TrashDocument | null { ... }
// ```
/** Returns the document facts, or null when the provider gives no row or no name or size. */
internal fun queryDocument(context: Context, uri: Uri): TrashDocument? {
    /** Content resolver the provider query runs through. */
    val resolver = context.contentResolver
    /** Columns requested from the provider. */
    val columns: Array<String> = arrayOf(
        Document.COLUMN_DISPLAY_NAME,
        Document.COLUMN_MIME_TYPE,
        Document.COLUMN_SIZE,
    )
    /** Cursor over the document facts, or null when the provider returns none. */
    val cursor = resolver.query(uri, columns, null, null, null) ?: return null
    cursor.use { rows ->
        if (!rows.moveToFirst() || rows.isNull(0) || rows.isNull(2)) {
            return null
        }
        return TrashDocument(
            name = rows.getString(0),
            mimeType = if (rows.isNull(1)) null else rows.getString(1),
            sizeBytes = rows.getLong(2),
        )
    }
}

// What:     `internal fun documentExists(...)` reports whether the provider still lists the document.
// Why:      A delete that reports failure may still have removed the file, and only the listing settles it.
//
// In TS you'd write (pseudocode):
// ```ts
// function documentExists(resolver, uri): boolean { ... }
// ```
/** Returns true when the provider still returns a row for the document, and true when the query itself fails. */
internal fun documentExists(context: Context, uri: Uri): Boolean {
    return try {
        context.contentResolver.query(uri, arrayOf(Document.COLUMN_DOCUMENT_ID), null, null, null)?.use { rows ->
            rows.moveToFirst()
        } ?: false
    } catch (expectedFailure: Exception) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG existence query failed, assuming present", expectedFailure)
        true
    }
}

// What:     `internal fun parentIdOf(...)` finds the nearest folder above a document inside a tree.
// Why:      Undo recreates the document in this folder, so the folder must be confirmed before any delete.
//
// In TS you'd write (pseudocode):
// ```ts
// function parentIdOf(treeUri, documentUri): string | null { /* longest ancestor id confirmed by isChildDocument */ }
// ```
/** Returns the document id of the nearest confirmed parent folder, or null when no parent can be confirmed. */
internal fun parentIdOf(context: Context, treeUri: Uri, documentUri: Uri): String? {
    /** Content resolver the path and child queries run through. */
    val resolver = context.contentResolver
    /** Path of ids the provider reports for the document, in whatever order it uses. */
    val path: DocumentsContract.Path = DocumentsContract.findDocumentPath(resolver, documentUri) ?: return null
    /** Document id of the file itself. */
    val documentId: String = DocumentsContract.getDocumentId(documentUri)
    /** Ancestor ids whose path text the document id starts with, so each one really contains the document. */
    val ancestors: List<String> = path.path.filter { ancestor -> isDescendantDocumentId(ancestor, documentId) }
    /** Nearest ancestor, which is the one with the longest id. */
    val parent: String = ancestors.maxByOrNull { ancestor -> ancestor.length } ?: return null
    /** Folder URI that the document is recreated under. */
    val parentUri: Uri = DocumentsContract.buildDocumentUriUsingTree(treeUri, parent)
    if (!parentContainsDocument(
            resolver = resolver,
            parentUri = parentUri,
            documentUri = documentUri,
            parentId = parent,
            documentId = documentId,
        )
    ) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG parent not confirmed by provider")
        return null
    }
    return parent
}

// What:     `private fun parentContainsDocument(...)` asks whether a folder contains a document.
// Why:      `DocumentsContract.isChildDocument` exists only from API 29 (Android 10), and the app supports API 26 and
//           later, where calling it throws `NoSuchMethodError`. Older versions compare the path text of the two
//           document ids instead, the same rule that picked the candidate parent, so the answer stays equal for the
//           path-shaped ids the candidate list is built from.
//
// In TS you'd write (pseudocode):
// ```ts
// function parentContainsDocument(a): boolean {
//   return sdk >= 29 ? provider.isChild(a) : isDescendantDocumentId(a.parentId, a.documentId);
// }
// ```
/** Returns whether the provider (API 29 and later) or the id path text (API 26 to 28) puts the document inside. */
private fun parentContainsDocument(
    resolver: ContentResolver,
    parentUri: Uri,
    documentUri: Uri,
    parentId: String,
    documentId: String,
): Boolean {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        return DocumentsContract.isChildDocument(resolver, parentUri, documentUri)
    }
    return isDescendantDocumentId(parentId, documentId)
}

// What:     `private fun copyToCache(...)` copies the document bytes into the app cache and verifies the copy.
// Why:      The original is deleted only after this returns, so a failed or short copy leaves the file in place.
//
// In TS you'd write (pseudocode):
// ```ts
// async function copyToCache(uri, facts, id): Promise<string> { /* write, then compare size; throw on mismatch */ }
// ```
/** Copies the document into the trash cache folder, verifies its size, and returns the cache file. */
private suspend fun copyToCache(context: Context, uri: Uri, facts: TrashDocument, id: String): File {
    /** Folder for this trash operation under the app cache. */
    val folder = File(context.cacheDir, "$TRASH_CACHE_DIR/$id")
    if (!folder.isDirectory && !folder.mkdirs()) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG copy folder could not be created id=$id")
        trackActionFailure(TrackActionFailure.COPY_FAILED)
    }
    /** Cache file that receives the bytes of the document. */
    val file = File(folder, facts.name)
    /** Byte count written by the copy. */
    val copied: Long = withContext(Dispatchers.IO) {
        try {
            context.contentResolver.openInputStream(uri)?.use { input ->
                file.outputStream().use { output -> input.copyTo(output) }
            } ?: throw FileNotFoundException("no stream for $uri")
        } catch (cancellation: CancellationException) {
            throw cancellation
        } catch (expectedFailure: IOException) {
            Log.w(LOG_TAG, "$TRASH_FLOW_TAG copy failed id=$id", expectedFailure)
            file.delete()
            trackActionFailure(TrackActionFailure.COPY_FAILED, expectedFailure)
        }
    }
    if (!copyMatches(expectedBytes = facts.sizeBytes, copiedBytes = copied) || file.length() != copied) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG copy size mismatch id=$id expected=${facts.sizeBytes} copied=$copied")
        file.delete()
        trackActionFailure(TrackActionFailure.COPY_FAILED)
    }
    Log.i(LOG_TAG, "$TRASH_FLOW_TAG copy verified id=$id bytes=$copied")
    return file
}

// What:     `internal suspend fun trashSaf(...)` moves one picked-folder document to trash by copy, verify, and delete.
// Why:      The picked folder has no system trash, so the copy is the only way Undo can restore the file.
//
// In TS you'd write (pseudocode):
// ```ts
// async function trashSaf(track, treeUri, id): Promise<TrashRecord> { /* facts, parent, copy, verify, delete */ }
// ```
/**
 * Copies a picked-folder track to the cache, verifies the copy, deletes the document, and returns the record that Undo
 * needs.
 *
 * @throws TrackActionException when any step fails; the document is deleted only after every check passed
 */
internal suspend fun trashSaf(context: Context, track: Track, treeUri: Uri, id: String): TrashRecord {
    /** URI of the document to trash. */
    val documentUri: Uri = Uri.parse(track.uri)
    /** Name, MIME type, and size read before anything changes. */
    val facts: TrashDocument = withContext(Dispatchers.IO) { queryDocument(context, documentUri) }
        ?: trackActionFailure(TrackActionFailure.COPY_FAILED)
    /** Folder that holds the document, confirmed by the provider before any delete. */
    val parentId: String = withContext(Dispatchers.IO) { parentIdOf(context, treeUri, documentUri) }
        ?: trackActionFailure(TrackActionFailure.NO_PARENT)
    /** Verified cache copy of the document bytes. */
    val copy: File = copyToCache(context = context, uri = documentUri, facts = facts, id = id)
    /** Whether the provider reported success for the delete call. */
    val deleted: Boolean = withContext(Dispatchers.IO) {
        try {
            DocumentsContract.deleteDocument(context.contentResolver, documentUri)
        } catch (expectedFailure: Exception) {
            Log.w(LOG_TAG, "$TRASH_FLOW_TAG delete threw id=$id", expectedFailure)
            false
        }
    }
    if (!deleted && withContext(Dispatchers.IO) { documentExists(context, documentUri) }) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG delete refused and document still present id=$id")
        copy.delete()
        trackActionFailure(TrackActionFailure.DELETE_FAILED)
    }
    Log.i(LOG_TAG, "$TRASH_FLOW_TAG document trashed id=$id deleted=$deleted")
    return TrashRecord(
        id = id,
        displayName = facts.name,
        route = TrashRoute.SAF_DELETE_WITH_COPY_BACK,
        trackUri = track.uri,
        treeUri = treeUri.toString(),
        parentDocumentId = parentId,
        mimeType = facts.mimeType ?: mimeTypeFor(facts.name),
        copyPath = copy.absolutePath,
        sizeBytes = facts.sizeBytes,
    )
}

// What:     `internal suspend fun restoreSaf(...)` recreates a trashed picked-folder document from its cache copy.
// Why:      Undo must put back the same name and bytes, and a restore that cannot be verified is removed rather than
//           kept.
//
// In TS you'd write (pseudocode):
// ```ts
// async function restoreSaf(context, record): Promise<void> { /* create, write, verify, remove copy */ }
// ```
/**
 * Creates the document again under its original name, writes the cached bytes, verifies name and size, and removes the
 * cache copy.
 *
 * @throws TrackActionException with RESTORE_FAILED when any step fails; the cache copy is kept in that case
 */
internal suspend fun restoreSaf(context: Context, record: TrashRecord) {
    /** Tree the document belonged to, from the record. */
    val treeUri: Uri = Uri.parse(record.treeUri ?: trackActionFailure(TrackActionFailure.RESTORE_FAILED))
    /** Folder id the document belonged to, from the record. */
    val parentId: String = record.parentDocumentId ?: trackActionFailure(TrackActionFailure.RESTORE_FAILED)
    /** Cache file that holds the bytes to write back. */
    val source = File(record.copyPath ?: trackActionFailure(TrackActionFailure.RESTORE_FAILED))
    if (!source.isFile || !copyMatches(expectedBytes = record.sizeBytes, copiedBytes = source.length())) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG restore refused, cache copy does not match id=${record.id}")
        trackActionFailure(TrackActionFailure.RESTORE_FAILED)
    }
    /** Folder the document is recreated in. */
    val parentUri: Uri = DocumentsContract.buildDocumentUriUsingTree(treeUri, parentId)
    /** Document created by the provider, or null when the provider refused. */
    val created: Uri = withContext(Dispatchers.IO) {
        try {
            DocumentsContract.createDocument(context.contentResolver, parentUri, record.mimeType, record.displayName)
        } catch (expectedFailure: Exception) {
            Log.w(LOG_TAG, "$TRASH_FLOW_TAG create threw id=${record.id}", expectedFailure)
            null
        }
    } ?: trackActionFailure(TrackActionFailure.RESTORE_FAILED)
    if (!writeAndVerify(context = context, created = created, source = source, record = record)) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG restored copy failed verification, removing it id=${record.id}")
        withContext(Dispatchers.IO) { deleteCreated(context, created) }
        trackActionFailure(TrackActionFailure.RESTORE_FAILED)
    }
    source.delete()
    Log.i(LOG_TAG, "$TRASH_FLOW_TAG document restored id=${record.id}")
}

// What:     `private suspend fun writeAndVerify(...)` writes the cache bytes into the created document and checks name
//           and size.
// Why:      The restore counts as done only when the provider reports the original name and the original byte count.
//
// In TS you'd write (pseudocode):
// ```ts
// async function writeAndVerify(created, source, record): Promise<boolean> { ... }
// ```
/** Writes the cached bytes into the created document and returns true only when its name and size match the record. */
private suspend fun writeAndVerify(context: Context, created: Uri, source: File, record: TrashRecord): Boolean =
    withContext(Dispatchers.IO) {
        try {
            context.contentResolver.openOutputStream(created, "w")?.use { output ->
                source.inputStream().use { input -> input.copyTo(output) }
            } ?: return@withContext false
            /** Facts of the created document, or null when the provider gives none. */
            val facts: TrashDocument = queryDocument(context, created) ?: return@withContext false
            facts.name == record.displayName && facts.sizeBytes == record.sizeBytes
        } catch (cancellation: CancellationException) {
            throw cancellation
        } catch (expectedFailure: Exception) {
            Log.w(LOG_TAG, "$TRASH_FLOW_TAG write failed id=${record.id}", expectedFailure)
            false
        }
    }

// What:     `private fun deleteCreated(...)` removes a document that a failed restore created.
// Why:      A half-written document would be a second copy with the wrong content, so it is removed best effort.
//
// In TS you'd write (pseudocode):
// ```ts
// function deleteCreated(context, created): void { /* deleteDocument, logged on failure */ }
// ```
/** Deletes a document created by a failed restore, logging the outcome instead of throwing. */
private fun deleteCreated(context: Context, created: Uri) {
    try {
        /** Whether the provider removed the document a failed restore created. */
        val removed: Boolean = DocumentsContract.deleteDocument(context.contentResolver, created)
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG removed failed restore removed=$removed")
    } catch (expectedFailure: Exception) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG removing failed restore threw", expectedFailure)
    }
}

// What:     `internal fun mediaStoreRequest(...)` builds Android's trash or restore request for one media item.
// Why:      The request is available from Android 11, and older systems must refuse with a message instead of failing
//           silently.
//
// In TS you'd write (pseudocode):
// ```ts
// function mediaStoreRequest(context, uri, trash: boolean): IntentSender { ... }
// ```
/** Returns the pending intent of Android's trash or restore request, or throws when this system cannot make one. */
internal fun mediaStoreRequest(context: Context, uri: Uri, trash: Boolean): PendingIntent {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG media store trash unsupported sdk=${Build.VERSION.SDK_INT}")
        trackActionFailure(TrackActionFailure.MEDIASTORE_TOO_OLD)
    }
    return try {
        MediaStore.createTrashRequest(context.contentResolver, listOf(uri), trash)
    } catch (expectedFailure: Exception) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG create trash request failed trash=$trash", expectedFailure)
        trackActionFailure(TrackActionFailure.MEDIASTORE_FAILED, expectedFailure)
    }
}

// What:     `internal fun isMediaStoreTrashed(...)` reads the trashed flag of one media item, including trashed rows.
// Why:      Android reports success before the item really moved, so the app checks the flag before it claims success.
//
// In TS you'd write (pseudocode):
// ```ts
// function isMediaStoreTrashed(context, uri): boolean | null { /* query with MATCH_INCLUDE and read is_trashed */ }
// ```
/** Returns the trashed flag of the media item, or null when the row cannot be read. */
internal fun isMediaStoreTrashed(context: Context, uri: Uri): Boolean? {
    /** Query arguments that include trashed rows, which the default query hides. */
    val arguments = Bundle().apply {
        putInt(MediaStore.QUERY_ARG_MATCH_TRASHED, MediaStore.MATCH_INCLUDE)
    }
    return try {
        context.contentResolver.query(uri, arrayOf(MediaStore.MediaColumns.IS_TRASHED), arguments, null)?.use { rows ->
            if (rows.moveToFirst()) rows.getInt(0) != 0 else null
        }
    } catch (expectedFailure: Exception) {
        Log.w(LOG_TAG, "$TRASH_FLOW_TAG trashed-flag query failed", expectedFailure)
        null
    }
}
