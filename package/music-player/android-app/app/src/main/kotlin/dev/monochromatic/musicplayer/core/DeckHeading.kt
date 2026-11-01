// Pure text for the transport deck heading. This file has no Compose imports, so the folder label is unit-tested
// on the JVM.

// What:     `package dev.monochromatic.musicplayer.core` places the heading text beside the builder that uses it.
// Why:      The builder calls `deckFolderLabelFor` directly, so both share this package and need no extra imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `private const val DECK_PATH_SEPARATOR: Char = '/'` separates the segments of a display path.
// Why:      Display paths use `/`, so the directory part and the file name are cut on the same character.
//
// In TS you'd write (pseudocode):
// ```ts
// const DECK_PATH_SEPARATOR = "/";
// ```
/** Separator between the segments of a display path. */
private const val DECK_PATH_SEPARATOR: Char = '/'

// What:     `fun deckFolderLabelFor(displayPath: String, pageLabel: String): String` names a track's folder.
// Why:      The deck's heading shows the folder before the title. A track at the root of its page has no directory
//           part, so the page name stands in for the folder, and nothing playing gives an empty label.
//
// In TS you'd write (pseudocode):
// ```ts
// const deckFolderLabelFor = (displayPath: string, pageLabel: string): string => {
//   if (displayPath === "") return "";
//   const directory = displayPath.split("/").slice(0, -1).pop() ?? "";
//   return directory === "" ? pageLabel : directory;
// };
// ```
/**
 * Name of the directory directly holding the track, [pageLabel] when the path has no directory part, or an empty
 * text when the path is empty.
 */
fun deckFolderLabelFor(displayPath: String, pageLabel: String): String {
    if (displayPath.isEmpty()) {
        return ""
    }
    /** Path of the directory that holds the file, which is empty when the file sits at the root of its page. */
    val parentPath: String = displayPath.substringBeforeLast(DECK_PATH_SEPARATOR, "")
    /** Innermost directory name of the path, which is the last segment of the parent path. */
    val directory: String = parentPath.substringAfterLast(DECK_PATH_SEPARATOR)
    if (directory.isEmpty()) {
        return pageLabel
    }
    return directory
}
