// What:     `package dev.monochromatic.musicplayer.core` places the folder name beside the first-run copy.
// Why:      The first-run copy names the folder, so the name rule sits with the copy and is testable on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `private const val PRIMARY_VOLUME_ID: String = "primary"` names the volume of the shared storage.
// Why:      Android labels the internal shared storage "primary" in every tree document id.
//
// In TS you'd write (pseudocode):
// ```ts
// const PRIMARY_VOLUME_ID = "primary";
// ```
/** Volume id that Android uses for the internal shared storage. */
private const val PRIMARY_VOLUME_ID: String = "primary"

// What:     `private const val PRIMARY_VOLUME_NAME: String = "Internal storage"` is the name of that volume.
// Why:      A folder at the root of the internal storage has no path segment, so the volume names it.
//
// In TS you'd write (pseudocode):
// ```ts
// const PRIMARY_VOLUME_NAME = "Internal storage";
// ```
/** Name shown for a folder at the root of the internal shared storage. */
private const val PRIMARY_VOLUME_NAME: String = "Internal storage"

// What:     `private const val UNNAMED_FOLDER_NAME: String = "Selected folder"` is the last-resort name.
// Why:      A document id with neither a path segment nor a volume id must still give the copy a non-blank name.
//
// In TS you'd write (pseudocode):
// ```ts
// const UNNAMED_FOLDER_NAME = "Selected folder";
// ```
/** Name used when a tree document id names neither a folder nor a volume. */
private const val UNNAMED_FOLDER_NAME: String = "Selected folder"

// What:     `private const val VOLUME_SEPARATOR: Char = ':'` splits the volume id from the path.
// Why:      Tree document ids have the form `volume:path`, so the separator is named once.
//
// In TS you'd write (pseudocode):
// ```ts
// const VOLUME_SEPARATOR = ":";
// ```
/** Separator between the volume id and the path inside a tree document id. */
private const val VOLUME_SEPARATOR: Char = ':'

// What:     `private const val FOLDER_PATH_SEPARATOR: Char = '/'` splits the segments of the path.
// Why:      The folder name is the last segment of the path, so the separator is named once.
//
// In TS you'd write (pseudocode):
// ```ts
// const FOLDER_PATH_SEPARATOR = "/";
// ```
/** Separator between the segments of the path inside a tree document id. */
private const val FOLDER_PATH_SEPARATOR: Char = '/'

// What:     `fun folderNameFromDocumentId(documentId: String): String` names a held folder from its tree document id.
// Why:      The name is the last path segment, the volume names the storage root, and the fallback keeps the name
//           non-blank, because the first-run copy throws on a blank name.
//
// In TS you'd write (pseudocode):
// ```ts
// function folderNameFromDocumentId(documentId: string): string {
//   // "primary:Music/Albums" -> "Albums"; "primary:" -> "Internal storage"
// }
// ```
/**
 * Returns the display name of the folder that a tree document id names.
 *
 * @param documentId tree document id in the form `volume:path`, as the document provider reports it
 * @return the last path segment, or the volume name at the root, never blank
 */
fun folderNameFromDocumentId(documentId: String): String {
    /** Path inside the volume with surrounding separators removed, empty for the volume root. */
    val path: String = documentId.substringAfter(VOLUME_SEPARATOR, "").trim(FOLDER_PATH_SEPARATOR)
    /** Last segment of the path, or an empty text when the path has no segment. */
    val lastSegment: String = path.substringAfterLast(FOLDER_PATH_SEPARATOR).trim()
    if (lastSegment.isNotEmpty()) {
        return lastSegment
    }
    /** Volume id before the separator, or an empty text when the document id has none. */
    val volume: String = documentId.substringBefore(VOLUME_SEPARATOR).trim()
    if (volume == PRIMARY_VOLUME_ID) {
        return PRIMARY_VOLUME_NAME
    }
    if (volume.isNotEmpty()) {
        return volume
    }
    return UNNAMED_FOLDER_NAME
}

// What:     `private const val SYSTEM_LIBRARY_NAME: String = "Music library"` is the heading of the system audio
//           library.
// Why:      Without a held folder the library is whatever Android's MediaStore indexes, which has no folder name, and
//           the heading names it in plain words ("Music library"), not by the platform's API name.
//
// In TS you'd write (pseudocode):
// ```ts
// const SYSTEM_LIBRARY_NAME = "Music library";
// ```
/** Plain-words heading of the library that Android's system audio index provides when no folder is held. */
private const val SYSTEM_LIBRARY_NAME: String = "Music library"

// What:     `fun libraryDisplayName(heldFolderName: String?): String` names the source the library was read from.
// Why:      The unfolded left pane is headed by the name of the opened folder; when no folder is held the library is
//           the system audio library, which has no folder name, so a plain-words heading is used.
//
// In TS you'd write (pseudocode):
// ```ts
// function libraryDisplayName(heldFolderName: string | null): string {
//   return heldFolderName?.trim() || "Music library";
// }
// ```
/** Returns the held folder's name, or "Music library" for the system audio library when no folder is held. */
fun libraryDisplayName(heldFolderName: String?): String =
    heldFolderName?.trim().orEmpty().ifEmpty { SYSTEM_LIBRARY_NAME }
