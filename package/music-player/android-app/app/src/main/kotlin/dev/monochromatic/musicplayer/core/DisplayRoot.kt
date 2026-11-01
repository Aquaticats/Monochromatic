// Which folder the library's relative display paths are measured from (D86, D3, D28). The desktop player strips the
// folder the user opened and shows what is inside it, so a library whose only content is one top-level folder shows
// that folder's own name instead of loose tracks under a bare letter. Android has two sources and the root of each is
// known from the tracks' own URIs: a granted folder lists paths that are already relative to the grant, and the
// system audio library lists paths relative to the volume, where the music sits under `Music`.

// What:     `package dev.monochromatic.musicplayer.core` places the rule beside the path stripping that uses it.
// Why:      The rule has no Android types, so unit tests reach it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

/** Start of every system audio library URI (`content://media/external/audio/media/<id>`). */
private const val MEDIA_STORE_URI_PREFIX: String = "content://media/"

/** Segment of a granted folder's document URI that precedes the tree identifier. */
private const val TREE_URI_SEGMENT: String = "/tree/"

/** Directory of the system audio library under which a device keeps its music. */
private const val MUSIC_DIRECTORY: String = "Music"

// What:     `fun displayRootOf(uri: String): List<String>?` names the segments that a library's paths are relative to.
// Why:      A granted folder is the root itself (its paths already start inside it), so nothing is stripped; the system
//           library's root is its `Music` directory, so the many-artists library still shows artists and the
//           one-folder library shows the folder. Any other URI has no known root and keeps the old shared-directory
//           stripping.
//
// In TS you'd write (pseudocode):
// ```ts
// const displayRootOf = (uri: string): string[] | null =>
//   uri.includes("/tree/") ? [] : uri.startsWith("content://media/") ? ["Music"] : null;
// ```
/**
 * Returns the root segments to strip from the display paths of the library that [uri] belongs to: empty for a track
 * of a granted folder, `Music` for a track of the system audio library, and null when the source is not known.
 */
fun displayRootOf(uri: String): List<String>? = when {
    uri.contains(TREE_URI_SEGMENT) -> emptyList()
    uri.startsWith(MEDIA_STORE_URI_PREFIX) -> listOf(MUSIC_DIRECTORY)
    else -> null
}
