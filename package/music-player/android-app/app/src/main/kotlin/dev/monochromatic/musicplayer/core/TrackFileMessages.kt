// Feedback text for the track file actions. Each message names the affected file in plain words and fits two
// visible lines. Pure functions only, so the host JVM checks the wording and the line count.

// What:     `package dev.monochromatic.musicplayer.core` places the message text beside the file action rules.
// Why:      The messages are data built from the file name, so the core layer owns them and the tests read them
//           directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import dev.monochromatic.musicplayer.LoadFailure
import java.util.Locale

// What:     `internal const val TRASH_ERROR_ID: String` is the id of the single error message of the track actions.
// Why:      A fixed id lets the next action error replace the visible one instead of stacking another.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_ERROR_ID = "track-action-error";
// ```
/** Id of the error message that the track actions show; a later error replaces it. */
internal const val TRASH_ERROR_ID: String = "track-action-error"

// What:     `enum class TrackActionFailure` lists every failure the track actions can report to the user.
// Why:      A closed set keeps each failure's wording in one place, so no action shows an unnamed or silent error.
//
// In TS you'd write (pseudocode):
// ```ts
// type TrackActionFailure = "COPY_FAILED" | "DELETE_FAILED" | ...;
// ```
/** The closed set of failures a track action can show; each one has exactly one message. */
enum class TrackActionFailure {
    // What:     `COPY_FAILED` means the bytes could not be copied to the cache, so nothing was deleted.
    // Why:      The message must say the file is untouched, since the copy step runs before any delete.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // COPY_FAILED = "COPY_FAILED";
    // ```
    /** The copy to the app cache failed or did not match the size, so the file was not deleted. */
    COPY_FAILED,

    // What:     `DELETE_FAILED` means the provider refused or failed to delete after a verified copy.
    // Why:      The file is still in place, so the message says it was not changed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // DELETE_FAILED = "DELETE_FAILED";
    // ```
    /** The provider did not delete the document, so the file remains where it was. */
    DELETE_FAILED,

    // What:     `ROUTE_UNAVAILABLE` means the held folder grant does not cover the file or lacks write access.
    // Why:      The user must choose the folder again, which the message says.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // ROUTE_UNAVAILABLE = "ROUTE_UNAVAILABLE";
    // ```
    /** The folder grant does not cover this file or cannot change files, so nothing was deleted. */
    ROUTE_UNAVAILABLE,

    // What:     `NO_PARENT` means the folder that holds the file could not be confirmed.
    // Why:      Undo needs the folder, so the app refuses to delete a file it could not restore.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // NO_PARENT = "NO_PARENT";
    // ```
    /** The containing folder could not be confirmed, so the file was kept. */
    NO_PARENT,

    // What:     `RESTORE_FAILED` means Undo could not recreate the file.
    // Why:      The message says where the copy is kept, since it is the only remaining copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // RESTORE_FAILED = "RESTORE_FAILED";
    // ```
    /** Undo could not recreate the file, and its copy stays in app storage until the app restarts. */
    RESTORE_FAILED,

    // What:     `MEDIASTORE_FAILED` means Android's trash request could not start or was refused with an error.
    // Why:      Android owns the trash here, so the file was not changed and the message says so.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // MEDIASTORE_FAILED = "MEDIASTORE_FAILED";
    // ```
    /** Android's trash request could not be started, so the file was not changed. */
    MEDIASTORE_FAILED,

    // What:     `MEDIASTORE_RESTORE_FAILED` means Android's restore request could not be started.
    // Why:      The item stays in the Android trash, which the message says.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // MEDIASTORE_RESTORE_FAILED = "MEDIASTORE_RESTORE_FAILED";
    // ```
    /** Android's restore request could not be started, so the item is still in the Android trash. */
    MEDIASTORE_RESTORE_FAILED,

    // What:     `MEDIASTORE_TOO_OLD` means this Android version has no trash request.
    // Why:      The app cannot move the item to trash on that version, and the message says so.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // MEDIASTORE_TOO_OLD = "MEDIASTORE_TOO_OLD";
    // ```
    /** This Android version has no trash request, so the item was not changed. */
    MEDIASTORE_TOO_OLD,

    // What:     `FILE_MANAGER_FAILED` means no app opened the folder of the file.
    // Why:      A silent failure would look like success, so the user is told the folder did not open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // FILE_MANAGER_FAILED = "FILE_MANAGER_FAILED";
    // ```
    /** No file manager could open the folder of the file. */
    FILE_MANAGER_FAILED,

    // What:     `FILE_MANAGER_NO_FOLDER` means the folder of a media-library track is not known to the app.
    // Why:      The app has no documented folder route for the media library, so it says so instead of guessing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // FILE_MANAGER_NO_FOLDER = "FILE_MANAGER_NO_FOLDER";
    // ```
    /** The folder of this media-library track has no route the app can open. */
    FILE_MANAGER_NO_FOLDER,

    // What:     `CLIPBOARD_FAILED` means the file name could not be copied.
    // Why:      A failed clipboard write must not look like a copied name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // CLIPBOARD_FAILED = "CLIPBOARD_FAILED";
    // ```
    /** The file name could not be placed on the clipboard. */
    CLIPBOARD_FAILED,
}

// What:     `fun trackActionFailureMessage(...)` builds the error feedback for one failed track action.
// Why:      Every failure uses the same message id, so the newest error replaces the previous one.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackActionFailureMessage(failure: TrackActionFailure, fileName: string): FeedbackMessage { ... }
// ```
/** Returns the error message for one failure, naming the file in its first line. */
fun trackActionFailureMessage(failure: TrackActionFailure, fileName: String): FeedbackMessage {
    /** Name of the affected file, shortened so the message stays within two lines. */
    val name: String = shortName(fileName)
    return FeedbackMessage(
        id = TRASH_ERROR_ID,
        text = failureText(failure = failure, name = name),
        kind = FeedbackKind.ERROR,
        actionLabel = null,
    )
}

// What:     `private fun failureText(...)` gives the two-line wording of one failure.
// Why:      The wording lives beside the failure names, so a change to one message is one edit.
//
// In TS you'd write (pseudocode):
// ```ts
// function failureText(failure: TrackActionFailure, name: string): string { /* exhaustive switch */ }
// ```
/** Returns the two-line text of one failure, with the shortened file name in its first line. */
private fun failureText(failure: TrackActionFailure, name: String): String = when (failure) {
    TrackActionFailure.COPY_FAILED -> "Could not copy $name.\nNothing was moved to trash."
    TrackActionFailure.DELETE_FAILED -> "Could not move $name to trash.\nThe file was not changed."
    TrackActionFailure.ROUTE_UNAVAILABLE ->
        "Could not move $name to trash.\nChoose the folder again to allow changes."
    TrackActionFailure.NO_PARENT -> "Could not move $name to trash.\nIts folder could not be read, so it was kept."
    TrackActionFailure.RESTORE_FAILED ->
        "Could not put $name back.\nIts copy stays in app storage until the app restarts."
    TrackActionFailure.MEDIASTORE_FAILED -> "Could not move $name to trash.\nThe file was not changed."
    TrackActionFailure.MEDIASTORE_RESTORE_FAILED -> "Could not restore $name.\nIt is still in the Android trash."
    TrackActionFailure.MEDIASTORE_TOO_OLD -> "Could not move $name to trash.\nThis Android version cannot do it."
    TrackActionFailure.FILE_MANAGER_FAILED -> "Could not open the folder of $name.\nNo file manager opened it."
    TrackActionFailure.FILE_MANAGER_NO_FOLDER ->
        "Could not open the folder of $name.\nThe app has no folder route for this library."
    TrackActionFailure.CLIPBOARD_FAILED -> "Could not copy the file name of $name."
}

// What:     `fun trashUndoMessage(record: TrashRecord): FeedbackMessage` builds the Undo message of one trash.
// Why:      The message carries the record's own id, so its Undo action finds the right record and no other.
//
// In TS you'd write (pseudocode):
// ```ts
// function trashUndoMessage(record: TrashRecord): FeedbackMessage {
//   return { id: `trash-undo-${record.id}`, text: `Moved ${record.name} to trash`, kind: "UNDO", actionLabel: "Undo" };
// }
// ```
/** Returns the Undo message that says the named file was moved to trash and offers the Undo action. */
fun trashUndoMessage(record: TrashRecord): FeedbackMessage = FeedbackMessage(
    id = record.messageId,
    text = "Moved ${shortName(record.displayName)} to trash",
    kind = FeedbackKind.UNDO,
    actionLabel = TRASH_UNDO_ACTION_LABEL,
)

// What:     `internal const val TRASH_UNDO_ACTION_LABEL: String` is the visible label of the Undo action.
// Why:      The label is accepted D8 wording, and a single constant keeps every Undo message identical.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_UNDO_ACTION_LABEL = "Undo";
// ```
/** Visible label of the Undo action on a trash message. */
internal const val TRASH_UNDO_ACTION_LABEL: String = "Undo"

// What:     `internal const val TRACK_LOAD_ERROR_ID: String` is the id of the message shown when a tapped track cannot
//           be opened.
// Why:      A second failure replaces the first instead of stacking, because the overlay keeps one message per id.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_LOAD_ERROR_ID = "track-load-error";
// ```
/** Id of the error message for a track that could not be opened for playback. */
internal const val TRACK_LOAD_ERROR_ID: String = "track-load-error"

// What:     `fun trackLoadFailureMessage(reason: LoadFailure, fileName: String): FeedbackMessage` words why a tapped
//           track did not start.
// Why:      M6: the open of a track file can stall when the media provider is busy; the message says what happened
//           and what to do, in two lines like the other file errors.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackLoadFailureMessage(reason: LoadFailure, fileName: string): FeedbackMessage { ... }
// ```
/** Returns the error message for a track that could not be opened, naming the file and the likely remedy. */
fun trackLoadFailureMessage(reason: LoadFailure, fileName: String): FeedbackMessage = FeedbackMessage(
    id = TRACK_LOAD_ERROR_ID,
    text = when (reason) {
        LoadFailure.UNREADABLE -> "Could not play ${shortName(fileName)}.\nIts file could not be opened."
        LoadFailure.TIMED_OUT ->
            "Could not play ${shortName(fileName)}.\nThe music library did not answer in time. Try again in a moment."
    },
    kind = FeedbackKind.ERROR,
    actionLabel = null,
)

// What:     `internal const val TRACK_DETAILS_ID: String` is the id of the File details notice.
// Why:      Choosing File details again on another track replaces the notice instead of stacking.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRACK_DETAILS_ID = "track-details";
// ```
/** Id of the notice that shows the details of one track file. */
internal const val TRACK_DETAILS_ID: String = "track-details"

// What:     `private const val BYTES_PER_UNIT: Double = 1024.0` is the step between the sizes the notice names.
// Why:      The desktop's File details uses the same step, so both apps print one size the same way.
//
// In TS you'd write (pseudocode):
// ```ts
// const BYTES_PER_UNIT = 1024;
// ```
/** Bytes in a kilobyte, megabyte or gigabyte of the printed size. */
private const val BYTES_PER_UNIT: Double = 1024.0

// What:     `fun humanFileSize(bytes: Long): String` writes a byte count as `1.5 MB`.
// Why:      File details names the size in the unit a person reads at a glance; the format (one decimal, KB, MB or GB)
//           is the desktop's `human_size`.
//
// In TS you'd write (pseudocode):
// ```ts
// function humanFileSize(bytes: number): string {
//   if (bytes >= 1024 ** 3) return (bytes / 1024 ** 3).toFixed(1) + " GB";
//   if (bytes >= 1024 ** 2) return (bytes / 1024 ** 2).toFixed(1) + " MB";
//   return (bytes / 1024).toFixed(1) + " KB";
// }
// ```
/** Returns the size as one decimal and the largest unit of KB, MB or GB that keeps it at one or more. */
fun humanFileSize(bytes: Long): String {
    /** The size as a fraction so the division below keeps its decimals. */
    val value: Double = bytes.toDouble()
    return when {
        value >= BYTES_PER_UNIT * BYTES_PER_UNIT * BYTES_PER_UNIT ->
            String.format(Locale.ROOT, "%.1f GB", value / (BYTES_PER_UNIT * BYTES_PER_UNIT * BYTES_PER_UNIT))
        value >= BYTES_PER_UNIT * BYTES_PER_UNIT ->
            String.format(Locale.ROOT, "%.1f MB", value / (BYTES_PER_UNIT * BYTES_PER_UNIT))
        else -> String.format(Locale.ROOT, "%.1f KB", value / BYTES_PER_UNIT)
    }
}

// What:     `fun shortFolderOf(displayPath: String): String` names the last two folders of a track's path.
// Why:      A message about a file says where it sits, but a deep path does not fit a two-line notice; the desktop's
//           `short_folder` keeps the last two names behind an ellipsis.
//
// In TS you'd write (pseudocode):
// ```ts
// function shortFolderOf(displayPath: string): string {
//   const folders = displayPath.split("/").slice(0, -1);
//   return folders.length > 2 ? "…/" + folders.slice(-2).join("/") : folders.join("/");
// }
// ```
/** Returns the folders above the file, shortened to the last two behind an ellipsis, or empty at the library root. */
fun shortFolderOf(displayPath: String): String {
    /** Folder names above the file, outermost first. */
    val folders: List<String> = displayPath.split('/').dropLast(1).filter { name -> name.isNotEmpty() }
    return if (folders.size > 2) "…/" + folders.takeLast(2).joinToString("/") else folders.joinToString("/")
}

// What:     `fun trackDetailsMessage(...)` builds the File details notice for one track.
// Why:      M5: File details did nothing. The desktop shows the file name, the size and the folder as a notice
//           (`details_text`); Android shows the same facts the same way.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackDetailsMessage(fileName: string, sizeBytes: number | null, displayPath: string): FeedbackMessage { ...
// }
// ```
/** Returns the notice that lists a track file's name, size and folder, with "size unknown" when the size is missing. */
fun trackDetailsMessage(fileName: String, sizeBytes: Long?, displayPath: String): FeedbackMessage {
    /** The size text, or the words that say it could not be read. */
    val size: String = if (sizeBytes == null) "size unknown" else humanFileSize(sizeBytes)
    /** Where the file sits, or the library root's wording when it sits directly in the library. */
    val folder: String = shortFolderOf(displayPath).ifEmpty { "library root" }
    return FeedbackMessage(
        id = TRACK_DETAILS_ID,
        text = "${shortName(fileName)}\n$size · $folder",
        kind = FeedbackKind.NOTICE,
        actionLabel = null,
    )
}
