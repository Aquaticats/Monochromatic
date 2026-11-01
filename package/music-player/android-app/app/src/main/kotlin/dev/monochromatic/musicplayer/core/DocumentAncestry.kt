// Document id containment for storage providers whose ids are paths, used where the platform call is missing.

// What:     `package ...core` places the check beside the other platform-independent file logic.
// Why:      The check is pure string logic, so JVM tests cover the fallback without a content provider.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `fun documentIdPrefixOf(ancestorId)` gives the text that every descendant id starts with.
// Why:      A volume root id ends in a colon and has no slash (`primary:`), so its descendants start with the id
//           itself; any other folder id is followed by a slash.
//
// In TS you'd write (pseudocode):
// ```ts
// function documentIdPrefixOf(id: string): string { return id.endsWith(":") ? id : `${id}/`; }
// ```
/**
 * Returns the text that every descendant of the given ancestor document id starts with.
 *
 * @param ancestorId document id of the folder or volume root
 * @return the id itself for a volume root that ends in a colon, and the id plus a slash otherwise
 */
fun documentIdPrefixOf(ancestorId: String): String = if (ancestorId.endsWith(":")) ancestorId else "$ancestorId/"

// What:     `fun isDescendantDocumentId(...)` tests path containment between two document ids.
// Why:      `DocumentsContract.isChildDocument` exists only from API 29 and the app supports API 26 and later;
//           for providers whose ids are paths (the external storage provider), the id prefix gives the same
//           answer for the folders that `parentIdOf` considers. A document is not its own descendant, and a
//           sibling whose name merely starts with the folder name (`Music2` against `Music`) is not one either.
//
// In TS you'd write (pseudocode):
// ```ts
// function isDescendantDocumentId(parentId, documentId) {
//   return documentId !== parentId && documentId.startsWith(prefix(parentId));
// }
// ```
/**
 * Tells whether [documentId] lies inside [parentId] by comparing the path text of the two ids.
 *
 * @param parentId document id of the candidate containing folder
 * @param documentId document id of the candidate descendant
 * @return true when the id starts with the parent's prefix and is longer than it
 */
fun isDescendantDocumentId(parentId: String, documentId: String): Boolean {
    /** Text that every descendant of the parent starts with. */
    val prefix: String = documentIdPrefixOf(parentId)
    return documentId.length > prefix.length && documentId.startsWith(prefix)
}
