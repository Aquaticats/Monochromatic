// Pure rules for the track file actions: which trash route a track takes, the Undo state of one trashed file,
// and the names and MIME types the copy-back needs. No Android types, so the host JVM tests every branch.

// What:     `package dev.monochromatic.musicplayer.core` places the file action rules beside the menu model.
// Why:      The rules need no device, and the Android actions and the unit tests share one implementation.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `internal const val TRASH_SAF_AUTHORITY: String` is the provider that serves folders the user granted.
// Why:      Tracks from a picked folder carry this authority, and the route rule matches on it.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_SAF_AUTHORITY = "com.android.externalstorage.documents";
// ```
/** Authority of the storage provider that serves folders the user picked through the system folder chooser. */
internal const val TRASH_SAF_AUTHORITY: String = "com.android.externalstorage.documents"

// What:     `internal const val TRASH_MEDIA_AUTHORITY: String` is the authority of the device-wide media library.
// Why:      Tracks from the system music library carry this authority, and the route rule matches on it.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_MEDIA_AUTHORITY = "media";
// ```
/** Authority of the Android media library, which holds the device-wide music library. */
internal const val TRASH_MEDIA_AUTHORITY: String = "media"

// What:     `internal const val TRASH_CONTENT_PREFIX: String` is the scheme prefix every content URI starts with.
// Why:      The authority is the text between this prefix and the next slash.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_CONTENT_PREFIX = "content://";
// ```
/** Scheme prefix that every content URI string begins with. */
internal const val TRASH_CONTENT_PREFIX: String = "content://"

// What:     `internal const val TRASH_TREE_SEGMENT: String` marks the granted tree inside a document URI.
// Why:      The tree identifier is the text after this segment, and comparing it decides whether a grant covers a
//           track.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_TREE_SEGMENT = "/tree/";
// ```
/** Path segment that precedes the tree identifier in a granted document URI. */
internal const val TRASH_TREE_SEGMENT: String = "/tree/"

// What:     `internal const val TRASH_NAME_MAX_CHARS: Int` caps a file name inside a feedback message.
// Why:      Two visible lines hold a bounded name, and a longer name is shortened with an ellipsis.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_NAME_MAX_CHARS = 48;
// ```
/** Longest file name, in characters, that a feedback message shows before it is shortened. */
internal const val TRASH_NAME_MAX_CHARS: Int = 48

// What:     `internal const val TRASH_ELLIPSIS: String` marks a shortened file name.
// Why:      The ellipsis character is written as an escape so the source stays plain ASCII.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_ELLIPSIS = "…";
// ```
/** Ellipsis character appended to a file name that is too long to show whole. */
internal const val TRASH_ELLIPSIS: String = "…"

// What:     `internal const val TRASH_MIME_DEFAULT: String` is the MIME type of an extension the table does not know.
// Why:      A document still needs a MIME type when the provider reports none, and the octet stream type is the neutral
//           one.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_MIME_DEFAULT = "application/octet-stream";
// ```
/** Generic MIME type used when a file extension has no entry in the audio table. */
internal const val TRASH_MIME_DEFAULT: String = "application/octet-stream"

// What:     `private val TRASH_MIME_BY_EXTENSION: Map<String, String>` maps each accepted audio extension to a MIME
//           type.
// Why:      The copy-back creates a document with this type when the provider reports none, and the table covers
//           every extension in the app's accepted set.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_MIME_BY_EXTENSION: Record<string, string> = { flac: "audio/flac", mp3: "audio/mpeg", ... };
// ```
/** Audio MIME type for each accepted file extension, in lower case. */
private val TRASH_MIME_BY_EXTENSION: Map<String, String> = mapOf(
    "flac" to "audio/flac",
    "wav" to "audio/wav",
    "wave" to "audio/wav",
    "mp3" to "audio/mpeg",
    "ogg" to "audio/ogg",
    "oga" to "audio/ogg",
    "opus" to "audio/opus",
    "m4a" to "audio/mp4",
    "m4b" to "audio/mp4",
    "mp4" to "audio/mp4",
    "aac" to "audio/aac",
    "aiff" to "audio/aiff",
    "aif" to "audio/aiff",
    "aifc" to "audio/aiff",
)

// What:     `enum class TrashRoute` names the one way a track can be moved to trash.
// Why:      The route decides which Android API runs, and UNAVAILABLE means no safe route exists, so nothing is
//           deleted.
//
// In TS you'd write (pseudocode):
// ```ts
// type TrashRoute = "SAF_DELETE_WITH_COPY_BACK" | "MEDIASTORE_TRASH_REQUEST" | "UNAVAILABLE";
// ```
/** The closed set of trash routes a track can take. */
enum class TrashRoute {
    // What:     `SAF_DELETE_WITH_COPY_BACK` deletes the document in the granted folder and keeps a cache copy for Undo.
    // Why:      A picked folder has no system trash, so the app copies the bytes first and restores them on Undo.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // SAF_DELETE_WITH_COPY_BACK = "SAF_DELETE_WITH_COPY_BACK";
    // ```
    /** Copies the bytes to the app cache, deletes the document, and recreates it from the copy on Undo. */
    SAF_DELETE_WITH_COPY_BACK,

    // What:     `MEDIASTORE_TRASH_REQUEST` asks Android to trash the item through its own confirmation.
    // Why:      The device-wide library has a system trash, and Android owns the restore.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // MEDIASTORE_TRASH_REQUEST = "MEDIASTORE_TRASH_REQUEST";
    // ```
    /** Hands the item to Android's own trash request, which shows the system confirmation. */
    MEDIASTORE_TRASH_REQUEST,

    // What:     `UNAVAILABLE` means no route can move this track to trash safely.
    // Why:      Without a verified route the app must refuse rather than delete a file it cannot restore.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // UNAVAILABLE = "UNAVAILABLE";
    // ```
    /** No safe route exists for this track, so the action refuses and the file is not touched. */
    UNAVAILABLE,
}

// What:     `fun trashRouteFor(...)` chooses the trash route from the track URI and the folder grant.
// Why:      The route is the only decision that depends on where the file lives, so it is pure and tested here.
//
// In TS you'd write (pseudocode):
// ```ts
// function trashRouteFor(trackUri: string, treeUri: string | null, writableGrant: boolean): TrashRoute { ... }
// ```
/**
 * Returns the trash route for one track: the media library routes to Android's request, a picked folder routes to the
 * copy-back only when the held tree covers the track and holds a writable grant, and anything else is unavailable.
 */
fun trashRouteFor(trackUri: String, treeUri: String?, writableGrant: Boolean): TrashRoute {
    // What:     `val authority: String?` is the provider authority of the track URI.
    // Why:      The authority alone separates the media library from a picked folder and from unknown sources.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const authority = authorityOf(trackUri);
    // ```
    /** Authority of the track URI, or null when the text is not a content URI. */
    val authority: String? = authorityOf(trackUri)
    if (authority == TRASH_MEDIA_AUTHORITY) {
        return TrashRoute.MEDIASTORE_TRASH_REQUEST
    }
    if (authority == TRASH_SAF_AUTHORITY) {
        return safRouteFor(trackUri = trackUri, treeUri = treeUri, writableGrant = writableGrant)
    }
    return TrashRoute.UNAVAILABLE
}

// What:     `private fun safRouteFor(...)` decides the picked-folder route.
// Why:      The copy-back needs both a tree that covers the track and a writable grant, and missing either means no
//           route.
//
// In TS you'd write (pseudocode):
// ```ts
// function safRouteFor(trackUri: string, treeUri: string | null, writable: boolean): TrashRoute { ... }
// ```
/** Returns the copy-back route when the held tree covers the track and holds write access, otherwise unavailable. */
private fun safRouteFor(trackUri: String, treeUri: String?, writableGrant: Boolean): TrashRoute {
    // What:     `val trackTree: String?` is the tree identifier the track was listed under.
    // Why:      A track listed under another tree is outside the held grant, so its tree must match exactly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const trackTree = treeIdOf(trackUri);
    // ```
    /** Tree identifier inside the track URI, or null when the URI has no tree segment. */
    val trackTree: String? = treeIdOf(trackUri)
    /** Tree identifier of the held grant, or null when no grant is held. */
    val heldTree: String? = treeUri?.let { uri -> treeIdOf(uri) }
    if (trackTree == null || heldTree == null || !writableGrant) {
        return TrashRoute.UNAVAILABLE
    }
    if (trackTree != heldTree) {
        return TrashRoute.UNAVAILABLE
    }
    return TrashRoute.SAF_DELETE_WITH_COPY_BACK
}

// What:     `fun authorityOf(uri: String): String?` reads the authority from a content URI string.
// Why:      The route and the file manager actions both need the provider name, and a non-content string has none.
//
// In TS you'd write (pseudocode):
// ```ts
// function authorityOf(uri: string): string | null { return new URL(uri).host; }
// ```
/** Returns the text between the content scheme and the next slash, or null when the string is not a content URI. */
fun authorityOf(uri: String): String? {
    if (!uri.startsWith(TRASH_CONTENT_PREFIX)) {
        return null
    }
    /** Text that follows the content scheme, starting with the authority. */
    val rest: String = uri.removePrefix(TRASH_CONTENT_PREFIX)
    /** Index of the slash that ends the authority, or -1 when the URI has no path. */
    val slash: Int = rest.indexOf('/')
    /** Authority text, taken from the URI up to its first slash. */
    val authority: String = if (slash < 0) rest else rest.substring(0, slash)
    return authority.ifEmpty { null }
}

// What:     `fun treeIdOf(uri: String): String?` reads the tree identifier from a tree or document URI string.
// Why:      Two URIs refer to the same granted tree when their identifiers match, and a prefix match is not enough.
//
// In TS you'd write (pseudocode):
// ```ts
// function treeIdOf(uri: string): string | null { /* text between "/tree/" and the next "/" */ }
// ```
/**
 * Returns the tree identifier between the tree segment and the next slash, or null when the URI has no tree segment.
 */
fun treeIdOf(uri: String): String? {
    /** Index of the tree segment inside the URI, or -1 when the URI has none. */
    val start: Int = uri.indexOf(TRASH_TREE_SEGMENT)
    if (start < 0) {
        return null
    }
    /** Index where the tree identifier begins, right after the tree segment. */
    val from: Int = start + TRASH_TREE_SEGMENT.length
    /** Index of the slash that ends the tree identifier, or the end of the string. */
    val end: Int = uri.indexOf('/', from).let { found -> if (found < 0) uri.length else found }
    return uri.substring(from, end).ifEmpty { null }
}

// What:     `fun trackFileNameOf(displayPath: String): String` returns the last segment of a track's display path.
// Why:      The menu title, Copy filename, and the copy-back fallback all need the bare file name.
//
// In TS you'd write (pseudocode):
// ```ts
// function trackFileNameOf(displayPath: string): string { return displayPath.split("/").pop() ?? displayPath; }
// ```
/** Returns the text after the last slash of a display path, or the whole path when it has no slash. */
fun trackFileNameOf(displayPath: String): String = displayPath.substringAfterLast('/')

// What:     `fun mimeTypeFor(fileName: String): String` returns the audio MIME type for a file name.
// Why:      A document recreated on Undo needs a MIME type, and the table covers every accepted audio extension.
//
// In TS you'd write (pseudocode):
// ```ts
// function mimeTypeFor(fileName: string): string { return TRASH_MIME_BY_EXTENSION[extensionOf(fileName)] ??
//   "application/octet-stream"; }
// ```
/** Returns the MIME type for the extension of a file name, or the generic type when the extension is unknown. */
fun mimeTypeFor(fileName: String): String {
    /** Extension of the file name in lower case, or an empty string when the name has no dot. */
    val extension: String = fileName.substringAfterLast('.', "").lowercase()
    return TRASH_MIME_BY_EXTENSION[extension] ?: TRASH_MIME_DEFAULT
}

// What:     `fun shortName(name: String): String` shortens a long file name for a feedback message.
// Why:      Two visible lines hold a bounded name, so a name longer than the limit ends in an ellipsis.
//
// In TS you'd write (pseudocode):
// ```ts
// function shortName(name: string): string { return name.length <= 48 ? name : name.slice(0, 47) + "…"; }
// ```
/** Returns the name unchanged when it fits the limit, otherwise its leading characters followed by an ellipsis. */
fun shortName(name: String): String {
    if (name.length <= TRASH_NAME_MAX_CHARS) {
        return name
    }
    return name.take(TRASH_NAME_MAX_CHARS - TRASH_ELLIPSIS.length) + TRASH_ELLIPSIS
}

// What:     `fun copyMatches(...)` decides whether a cache copy holds exactly the bytes the provider reported.
// Why:      The original is deleted only after this returns true, so an unknown size or any mismatch refuses the
//           delete.
//
// In TS you'd write (pseudocode):
// ```ts
// function copyMatches(expected: number | null, copied: number): boolean { return expected !== null && expected ===
//   copied; }
// ```
/** Returns true only when the provider reported a size and the copied byte count equals it. */
fun copyMatches(expectedBytes: Long?, copiedBytes: Long): Boolean =
    expectedBytes != null && expectedBytes >= 0L && expectedBytes == copiedBytes

// What:     `enum class TrashUndoState` names the four states of one trashed file's Undo.
// Why:      Undo can run at most once at a time, and a finished Undo never runs again, so the states are explicit.
//
// In TS you'd write (pseudocode):
// ```ts
// type TrashUndoState = "PENDING" | "RESTORING" | "RESTORED" | "FAILED";
// ```
/** The closed set of Undo states of one trashed file. */
enum class TrashUndoState {
    // What:     `PENDING` means the file is in trash and its Undo can still run.
    // Why:      Only a pending record may begin a restore, and only a pending record's cache copy is kept.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // PENDING = "PENDING";
    // ```
    /** The file is in trash and Undo is offered. */
    PENDING,

    // What:     `RESTORING` means an Undo is running now.
    // Why:      A restoring record is never cleaned up, so the cache copy cannot vanish mid-restore.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // RESTORING = "RESTORING";
    // ```
    /** An Undo is running, so the record must not be cleaned up. */
    RESTORING,

    // What:     `RESTORED` means Undo put the file back.
    // Why:      A restored record offers nothing more, and its cache copy has already been removed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // RESTORED = "RESTORED";
    // ```
    /** Undo put the file back at its original place. */
    RESTORED,

    // What:     `FAILED` means Undo ran and did not put the file back.
    // Why:      A failed record offers no retry, and its cache copy stays until the app restarts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // FAILED = "FAILED";
    // ```
    /** Undo ran and the file was not put back. */
    FAILED,
}

// What:     `data class TrashRecord` holds everything Undo needs for one trashed file, as an immutable value.
// Why:      Undo must not depend on the live library, so the record carries the names, tree, parent, and cache path.
//
// In TS you'd write (pseudocode):
// ```ts
// type TrashRecord = { id: string; displayName: string; route: TrashRoute; state: TrashUndoState; ... };
// ```
/** Everything one Undo needs for one trashed file, kept as an immutable value that transitions return as a copy. */
data class TrashRecord(
    // What:     `val id: String` is the unique key of this trash, used to name its feedback message and cache folder.
    // Why:      A later trash never reuses an earlier record's identity, so cleanup targets exactly one file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // id: string;
    // ```
    /** Unique key of this trash operation. */
    val id: String,
    // What:     `val displayName: String` is the document's name as the provider reported it.
    // Why:      Undo recreates the document under this exact name, and the message names the file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // displayName: string;
    // ```
    /** File name the provider reported, which Undo recreates. */
    val displayName: String,
    // What:     `val route: TrashRoute` is the route the trash used.
    // Why:      Undo runs through the same route, so the record names the route.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // route: TrashRoute;
    // ```
    /** Route the trash used, which Undo follows. */
    val route: TrashRoute,
    // What:     `val trackUri: String` is the URI the track had before it was trashed.
    // Why:      A media-library Undo requests the same item, and the logs name the file by this URI.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // trackUri: string;
    // ```
    /** URI the track had before the trash. */
    val trackUri: String,
    // What:     `val treeUri: String?` is the granted tree the file was deleted from, or null for the media library.
    // Why:      Undo recreates the document under this tree, which may differ from the tree held later.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // treeUri: string | null;
    // ```
    /** Granted tree the file was deleted from, or null for the media library. */
    val treeUri: String?,
    // What:     `val parentDocumentId: String?` is the document identifier of the folder that held the file.
    // Why:      Undo creates the document inside this folder, and a null value means no parent was confirmed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // parentDocumentId: string | null;
    // ```
    /** Document identifier of the folder that held the file, or null when none was confirmed. */
    val parentDocumentId: String?,
    // What:     `val mimeType: String` is the MIME type the provider reported for the document.
    // Why:      Undo passes this type to the provider so the recreated document keeps its extension.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mimeType: string;
    // ```
    /** MIME type the provider reported, or the audio table's type when it reported none. */
    val mimeType: String,
    // What:     `val copyPath: String?` is the cache file that holds the bytes, or null for the media library.
    // Why:      Undo writes these bytes back, and the cache copy is removed once Undo succeeds or the record is
    //           dropped.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // copyPath: string | null;
    // ```
    /** Absolute path of the cache copy of the bytes, or null when the route keeps no copy. */
    val copyPath: String?,
    // What:     `val sizeBytes: Long` is the byte count the copy was verified against.
    // Why:      Undo checks the restored document against this size before it reports success.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // sizeBytes: number;
    // ```
    /** Byte count the cache copy was verified against, or zero for the media library. */
    val sizeBytes: Long,
    // What:     `val state: TrashUndoState` is where this record stands in its Undo.
    // Why:      Each transition checks this value, so an Undo cannot run twice or resume after it finished.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state: TrashUndoState = "PENDING";
    // ```
    /** Current Undo state, starting as pending. */
    val state: TrashUndoState = TrashUndoState.PENDING,
) {
    // What:     `init` checks the fields once when the record is created.
    // Why:      A blank id or name would produce an unnamed message or an unfindable cache folder.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (id.trim() === "") throw new Error("Trash record id must not be blank.");
    // ```
    init {
        require(id.isNotBlank()) { "Trash record id must not be blank." }
        require(displayName.isNotBlank()) { "Trash record display name must not be blank." }
        require(sizeBytes >= 0L) { "Trash record size must not be negative, was $sizeBytes." }
    }

    // What:     `val messageId: String` is the feedback message id that carries this record's Undo action.
    // Why:      Dismissal and the Undo action name the message by this id, so a replaced message is never mistaken for
    //           this one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // get messageId(): string { return `trash-undo-${this.id}`; }
    // ```
    /** Feedback message id of this record's Undo message. */
    val messageId: String
        get() = TRASH_UNDO_ID_PREFIX + id

    // What:     `fun beginRestore(): TrashRecord` moves a pending record to restoring.
    // Why:      Restoring is the only state in which Undo runs, and a record that is not pending cannot begin one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // beginRestore(): TrashRecord { if (this.state !== "PENDING") throw new Error("not pending"); return { ...this,
    //   state: "RESTORING" }; }
    // ```
    /** Returns a copy in the restoring state; throws when the record is not pending. */
    fun beginRestore(): TrashRecord {
        require(state == TrashUndoState.PENDING) { "Only a pending trash can be restored, was $state." }
        return copy(state = TrashUndoState.RESTORING)
    }

    // What:     `fun finishRestore(restored: Boolean): TrashRecord` ends a restoring record as restored or failed.
    // Why:      The outcome is recorded once, and only a restoring record can finish.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // finishRestore(restored: boolean): TrashRecord { ... }
    // ```
    /** Returns a copy in the restored or failed state; throws when the record is not restoring. */
    fun finishRestore(restored: Boolean): TrashRecord {
        require(state == TrashUndoState.RESTORING) { "Only a restoring trash can finish, was $state." }
        /** State that the finished restore moves the record to. */
        val next: TrashUndoState = if (restored) TrashUndoState.RESTORED else TrashUndoState.FAILED
        return copy(state = next)
    }
}

// What:     `internal const val TRASH_UNDO_ID_PREFIX: String` begins the id of every Undo message.
// Why:      Each Undo message id is built from this prefix and the record id, so ids never collide with error messages.
//
// In TS you'd write (pseudocode):
// ```ts
// const TRASH_UNDO_ID_PREFIX = "trash-undo-";
// ```
/** Prefix of the feedback message id of every Undo message. */
internal const val TRASH_UNDO_ID_PREFIX: String = "trash-undo-"
