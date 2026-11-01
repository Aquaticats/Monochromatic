// A file that is replaced or edited keeps its URI, so the analysis tells a changed file from the one it measured by a
// signature of the file's size and modification time, which the library scan reads for free with each row.

// What:     `package ...core` places the signature store beside the sweep ledger that consumes it.
// Why:      The store has no Android types, so unit tests reach it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import java.util.concurrent.ConcurrentHashMap

// What:     `private const val MODIFIED_MIX: Long` is an odd 64-bit constant that spreads the modification time over
//           the
//           signature.
// Why:      Size and time are combined by one multiply and one xor, so two files that differ in either almost never
//           share a signature, and a missing value (zero) leaves the other half untouched.
//
// In TS you'd write (pseudocode):
// ```ts
// const MODIFIED_MIX = 0x9e3779b97f4a7c15n;
// ```
/** Odd multiplier that spreads the modification time across the signature. */
private const val MODIFIED_MIX: Long = -7046029254386353131L

// What:     `object TrackSignatures` holds the signature of each scanned track by URI for the life of the process.
// Why:      The sources read size and modification time with the rows they already query. The analysis compares them
//           with the ledger to find replaced or edited files. The store is never cleared during the process, because a
//           clear between one scan and a later check made every file look changed.
//
// In TS you'd write (pseudocode):
// ```ts
// const signatures = new Map<string, bigint>();
// export const TrackSignatures = { record(uri, size, modifiedMs) {}, of(uri) {}, clear() {} };
// ```
/** Signatures of the tracks of the latest scans, keyed by URI. */
object TrackSignatures {
    /** Signature by track URI; safe for the scanning thread and the analysis thread to share. */
    private val signatures: ConcurrentHashMap<String, Long> = ConcurrentHashMap()

    /**
     * Combines a size and a modification time into one signature.
     *
     * @param sizeBytes file size, or 0 when the source did not report one
     * @param modifiedMs modification time in milliseconds, or 0 when the source did not report one
     * @return a signature that changes when either number changes; 0 only when both are unknown
     */
    fun signatureOf(sizeBytes: Long, modifiedMs: Long): Long = sizeBytes xor (modifiedMs * MODIFIED_MIX)

    /**
     * Remembers the signature of one scanned track.
     *
     * @param uri track URI exactly as the library lists it
     * @param signature value from [signatureOf]
     */
    fun record(uri: String, signature: Long) {
        signatures[uri] = signature
    }

    /** Returns the recorded signature of [uri], or 0 when the scan did not report one. */
    fun of(uri: String): Long = signatures[uri] ?: 0L

    /** Forgets every signature; the tests use it between cases. */
    fun clear() {
        signatures.clear()
    }
}
