// Real implementations of the track context menu actions that do not move a file: copy the file name, show the folder,
// start shuffle from a track, and read whether the held folder grant can change files. Failures are thrown as
// TrackActionException so the host can show each one in the feedback overlay.

// What:     `package dev.monochromatic.musicplayer` places the menu actions beside the host that calls them.
// Why:      The host calls these functions with the context and the controller, both in this package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports from `android.content` bring in the context, clipboard, and intent types the actions use.
// Why:      Copy filename writes the clipboard, and show in file manager starts an activity through an intent.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ClipboardManager, Intent } from "android/content";
// ```
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.DocumentsContract
import android.util.Log
import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.core.TRASH_SAF_AUTHORITY
import dev.monochromatic.musicplayer.core.TrackActionFailure
import dev.monochromatic.musicplayer.core.authorityOf
import dev.monochromatic.musicplayer.core.trackFileNameOf

// What:     `private const val TRACK_ACTION_TAG: String` names the log lines of this file.
// Why:      The tag separates menu action decisions from the trash flow and the rest of the app.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_ACTION_TAG = "TrackActions";
// ```
/** Tag of the log lines written by the track menu actions. */
private const val TRACK_ACTION_TAG: String = "TrackActions"

// What:     `class TrackActionException` carries one user-visible failure out of a track action.
// Why:      Throwing a typed exception keeps the failure name in the type, so the host maps it to its one message.
//
// In TS you'd write (pseudocode):
// ```ts
// class TrackActionException extends Error { constructor(readonly failure: TrackActionFailure, cause?: unknown) {} }
// ```
/**
 * Failure of one track action, naming which user-visible message applies, with the underlying cause when there is one.
 */
class TrackActionException(
    // What:     `val failure: TrackActionFailure` names the message the host shows.
    // Why:      The host maps this one value to the feedback text, so the failure stays typed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // readonly failure: TrackActionFailure;
    // ```
    /** Which user-visible failure occurred. */
    val failure: TrackActionFailure,
    cause: Throwable? = null,
) : Exception(failure.name, cause)

// What:     `internal fun trackActionFailure(...)` throws the typed failure for one call site.
// Why:      Call sites use it as a value that never returns, so each function keeps few throw statements.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackActionFailure(failure, cause?): never { throw new TrackActionException(failure, cause); }
// ```
/** Throws a track action exception for one failure; the call site reads as a failure value that never returns. */
internal fun trackActionFailure(failure: TrackActionFailure, cause: Throwable? = null): Nothing =
    throw TrackActionException(failure, cause)

// What:     `internal data class TrackMenuTarget` pins the track a menu was opened for.
// Why:      The menu must act on the track it opened for, even if a rescan moves the library index meanwhile.
//
// In TS you'd write (pseudocode):
// ```ts
// type TrackMenuTarget = { indexInLibrary: number; track: Track };
// ```
/** Library index and track of the row whose context menu is open. */
internal data class TrackMenuTarget(
    // What:     `val indexInLibrary: Int` is the load-order index the controller plays by.
    // Why:      Play and shuffle start the track at this index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // indexInLibrary: number;
    // ```
    /** Load-order index of the track when the menu opened. */
    val indexInLibrary: Int,
    // What:     `val track: Track` is the track itself, with its URI and display path.
    // Why:      Trash and file-system actions use the URI, so they never act on a row index alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // track: Track;
    // ```
    /** Track the menu was opened for, with its URI and display path. */
    val track: Track,
)

// What:     `internal fun copyFilename(...)` places the track's file name on the clipboard.
// Why:      Copy filename is a plain clipboard write, and a failed write must be reported instead of looking copied.
//
// In TS you'd write (pseudocode):
// ```ts
// async function copyFilename(track: Track): Promise<void> { await
//   navigator.clipboard.writeText(trackFileNameOf(track.displayPath)); }
// ```
/** Copies the track's file name to the system clipboard, throwing CLIPBOARD_FAILED when the clipboard refuses. */
internal fun copyFilename(context: Context, track: Track) {
    /** File name of the track, without its folders. */
    val name: String = trackFileNameOf(track.displayPath)
    /** System clipboard service, or null when the device has none. */
    val clipboard: ClipboardManager = context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
        ?: trackActionFailure(TrackActionFailure.CLIPBOARD_FAILED)
    try {
        clipboard.setPrimaryClip(ClipData.newPlainText("File name", name))
    } catch (expectedFailure: SecurityException) {
        Log.w(LOG_TAG, "$TRACK_ACTION_TAG clipboard write refused", expectedFailure)
        trackActionFailure(TrackActionFailure.CLIPBOARD_FAILED, expectedFailure)
    }
    Log.i(LOG_TAG, "$TRACK_ACTION_TAG file name copied")
}

// What:     `internal fun showInFileManager(...)` opens the folder that holds the track in a file manager.
// Why:      The folder is reachable only for picked-folder tracks, so the media library refuses with a message.
//
// In TS you'd write (pseudocode):
// ```ts
// function showInFileManager(track: Track): void { window.open(folderUrlOf(track)); }
// ```
/**
 * Opens the folder of a picked-folder track in the system file manager.
 *
 * @throws TrackActionException with FILE_MANAGER_NO_FOLDER for media-library tracks, or FILE_MANAGER_FAILED when no
 *     folder can be confirmed or no app opens it
 */
internal fun showInFileManager(context: Context, track: Track) {
    if (authorityOf(track.uri) != TRASH_SAF_AUTHORITY) {
        Log.w(LOG_TAG, "$TRACK_ACTION_TAG show in file manager has no folder route for this source")
        trackActionFailure(TrackActionFailure.FILE_MANAGER_NO_FOLDER)
    }
    /** URI of the track document under its granted tree. */
    val documentUri: Uri = Uri.parse(track.uri)
    /** Tree URI of the grant that lists the track, built from the document's tree identifier. */
    val treeUri: Uri = try {
        DocumentsContract.buildTreeDocumentUri(TRASH_SAF_AUTHORITY, DocumentsContract.getTreeDocumentId(documentUri))
    } catch (expectedFailure: IllegalArgumentException) {
        Log.w(LOG_TAG, "$TRACK_ACTION_TAG track URI has no tree", expectedFailure)
        trackActionFailure(TrackActionFailure.FILE_MANAGER_FAILED, expectedFailure)
    }
    /** Document id of the folder that holds the track, confirmed by the provider. */
    val parentId: String = parentIdOf(context = context, treeUri = treeUri, documentUri = documentUri)
        ?: trackActionFailure(TrackActionFailure.FILE_MANAGER_FAILED)
    /** URI of the folder itself, for the file manager to open. */
    val folderUri: Uri = DocumentsContract.buildDocumentUriUsingTree(treeUri, parentId)
    /** Intent that asks any app able to view folders to open the folder with read access. */
    val intent = Intent(Intent.ACTION_VIEW)
        .setDataAndType(folderUri, DocumentsContract.Document.MIME_TYPE_DIR)
        .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    try {
        context.startActivity(intent)
    } catch (expectedFailure: Exception) {
        Log.w(LOG_TAG, "$TRACK_ACTION_TAG no app opened the folder", expectedFailure)
        trackActionFailure(TrackActionFailure.FILE_MANAGER_FAILED, expectedFailure)
    }
    Log.i(LOG_TAG, "$TRACK_ACTION_TAG folder opened in file manager")
}

// What:     `internal fun writableGrantCovers(...)` reports whether the held folder grant can still change files.
// Why:      The copy-back needs write access, and the picker grant persisted by the app is read-only in the current
//           build.
//
// In TS you'd write (pseudocode):
// ```ts
// function writableGrantCovers(treeUri: string | null): boolean { return persisted.some(p => p.uri === treeUri &&
//   p.write); }
// ```
/** Returns true when the persisted permissions include write access for the given tree URI. */
internal fun writableGrantCovers(context: Context, treeUri: Uri?): Boolean {
    if (treeUri == null) {
        return false
    }
    /** Persisted permissions the system holds for this app. */
    val permissions = context.contentResolver.persistedUriPermissions
    /** Whether a persisted permission for this tree grants write access. */
    val writable: Boolean = permissions.any { permission ->
        permission.uri.toString() == treeUri.toString() && permission.isWritePermission
    }
    Log.i(LOG_TAG, "$TRACK_ACTION_TAG writable grant covers tree=$writable")
    return writable
}

// What:     `internal fun shuffleFromHere(...)` starts shuffled playback of the page at the given track.
// Why:      D7 names shuffle from a track, and the page scope is set first so the shuffle follows the visible list.
//
// In TS you'd write (pseudocode):
// ```ts
// function shuffleFromHere(index: number): void { controller.setPlaybackMode("shuffle_page");
//   controller.playIndex(index); }
// ```
/** Sets shuffle over the displayed page, then starts playback at the given library index. */
internal fun shuffleFromHere(controller: PlayerController, indexInLibrary: Int) {
    Log.i(LOG_TAG, "$TRACK_ACTION_TAG shuffle from here index=$indexInLibrary")
    controller.setPlaybackMode(PlaybackMode.SHUFFLE_PAGE)
    controller.playIndex(indexInLibrary)
}

// What:     `internal fun trackFileSizeBytes(...)` reads the size of one track file from its provider.
// Why:      M5: File details shows the size. A provider answers a size query with a database read, so the caller runs
//           it
//           off the main thread; the size is null when the provider does not report one.
//
// In TS you'd write (pseudocode):
// ```ts
// async function trackFileSizeBytes(track: Track): Promise<number | null> { /* query OpenableColumns.SIZE */ }
// ```
/** Returns the file size the provider reports for the track, or null when it reports none or the file is gone. */
internal fun trackFileSizeBytes(context: Context, track: Track): Long? {
    if (track.uri.startsWith("/")) {
        /** The file at an absolute path, whose length is its size. */
        val file = java.io.File(track.uri)
        return if (file.isFile) file.length() else null
    }
    try {
        context.contentResolver.query(
            Uri.parse(track.uri),
            arrayOf(android.provider.OpenableColumns.SIZE),
            null,
            null,
            null,
        )?.use { cursor ->
            if (cursor.moveToFirst() && !cursor.isNull(0)) {
                return cursor.getLong(0)
            }
        }
    } catch (expectedFailure: Exception) {
        Log.w(LOG_TAG, "$TRACK_ACTION_TAG size query failed", expectedFailure)
    }
    return null
}
