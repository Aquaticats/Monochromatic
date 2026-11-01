// Pure ordering of the persistable grant flags requested for a picked folder. Read and write come first, read only
// follows as the fallback when the provider refuses write access. No Android types, so the host JVM tests the order.

// What:     `package dev.monochromatic.musicplayer.core` places the grant ordering beside the other pure rules.
// Why:      The order is a decision the host JVM can check without a device or a picker.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `fun folderGrantAttempts(...)` lists the flag sets to request for a picked folder, most capable first.
// Why:      A picker may grant only read access, and the app then needs the read-only set so the library still
//           persists.
//
// In TS you'd write (pseudocode):
// ```ts
// function folderGrantAttempts(read: number, write: number): number[] { return [read | write, read]; }
// ```
/**
 * Returns the persistable flag sets to request, read and write first and read only second.
 * The list always has exactly these two entries, so the caller can fall back from the first to the second.
 */
fun folderGrantAttempts(readFlag: Int, writeFlag: Int): List<Int> = listOf(readFlag or writeFlag, readFlag)
