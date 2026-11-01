// The analysis service must start again whenever the library holds a track it has not looked at, not only for the first
// full index (M9 of the polish review), and a file that was replaced or edited counts as new again (D113).

// What:     `package ...core` places the ledger beside the other analysis rules.
// Why:      It is pure logic, so unit tests reach it without Android.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import java.nio.ByteBuffer

// What:     `private const val FNV_OFFSET: Long` and `FNV_PRIME: Long` are the constants of the 64-bit FNV-1a hash.
// Why:      A track is remembered by a 64-bit hash of its URI, so a million tracks cost eight megabytes and no strings.
//
// In TS you'd write (pseudocode):
// ```ts
// const FNV_OFFSET = 0xcbf29ce484222325n;
// const FNV_PRIME = 0x100000001b3n;
// ```
/** Start value of the 64-bit FNV-1a hash. */
private const val FNV_OFFSET: Long = -3750763034362895579L

/** Multiplier of the 64-bit FNV-1a hash. */
private const val FNV_PRIME: Long = 1099511628211L

/** Bytes of the marker and the two counts that frame the stored form. */
private const val HEADER_BYTES: Int = Int.SIZE_BYTES + Int.SIZE_BYTES + Int.SIZE_BYTES

/** Marks the stored form of the ledger, so an older or damaged file is not read as hashes. */
private const val LEDGER_MAGIC: Int = 0x53574C32

/** Odd multiplier that spreads a file signature across the key of its track. */
private const val SIGNATURE_MIX: Long = -7046029254386353131L

// What:     `data class SweepPlan(...)` is what one sweep has to do: the tracks to analyze and why.
// Why:      The service starts only for a non-empty plan, and the log and the tests read the counts.
//
// In TS you'd write (pseudocode):
// ```ts
// type SweepPlan = { todo: string[]; fresh: number; retried: number };
// ```
/** The tracks one sweep analyzes, with how many are new or changed and how many are retries of earlier failures. */
data class SweepPlan(
    /** URIs of the tracks to analyze, new and changed ones first. */
    val todo: List<String>,
    /** How many of [todo] the analysis never saw, or saw with another signature. */
    val fresh: Int,
    /** How many of [todo] failed earlier and are tried again. */
    val retried: Int,
)

// What:     `data class SweepRecord(...)` is the ledger after a sweep with the files that failed for the first time.
// Why:      D113: a failure is reported once per file; the ledger knows which failures it already had.
//
// In TS you'd write (pseudocode):
// ```ts
// type SweepRecord = { ledger: SweepLedger; newFailures: string[] };
// ```
/** A ledger updated with a sweep's results, and the URIs whose failure is new and must be reported. */
data class SweepRecord(
    /** The ledger after the sweep. */
    val ledger: SweepLedger,
    /** URIs that failed now and had not failed with the same signature before. */
    val newFailures: List<String>,
)

// What:     `class SweepLedger(...)` remembers which tracks the analysis has looked at (swept) and which it could not
//           decode (failed), each as a sorted array of hashes of the track's URI and file signature.
// Why:      A swept track needs no work. A track with an unknown URI or a changed signature is new work. A failed track
//           is tried again when the app starts or when the library gained new work, and is reported only the first
//           time.
//
// In TS you'd write (pseudocode):
// ```ts
// class SweepLedger {
//   plan(uris: string[], signatureOf: (uri: string) => bigint, retryFailed: boolean): SweepPlan { /* ... */ }
//   record(done: Track[], failed: Track[]): SweepRecord { /* ... */ }
//   toBytes(): Uint8Array {}
// }
// ```
/** The tracks the analysis has swept and the ones it failed on, keyed by URI and file signature. */
class SweepLedger private constructor(private val swept: LongArray, private val failed: LongArray) {
    /** Number of tracks the ledger remembers as swept. */
    val sweptCount: Int
        get() = swept.size

    /** Number of tracks the ledger remembers as failed. */
    val failedCount: Int
        get() = failed.size

    /**
     * Decides what a sweep has to do for the library as it is now.
     *
     * @param uris every track URI of the library
     * @param signatureOf file signature of a URI as the latest scan read it, 0 when unknown
     * @param retryFailed whether failures are tried again now (the app started); they are also tried again whenever
     * there is new or changed work
     * @return the tracks to analyze; empty when there is nothing to do
     */
    fun plan(uris: List<String>, signatureOf: (String) -> Long, retryFailed: Boolean): SweepPlan {
        /** Tracks the analysis never saw, or saw with another signature. */
        val fresh = ArrayList<String>()
        /** Tracks that failed before with the same signature. */
        val earlierFailures = ArrayList<String>()
        for (uri in uris) {
            /** Key of this track with its current signature. */
            val key: Long = keyOf(uri, signatureOf(uri))
            if (swept.binarySearch(key) >= 0) {
                continue
            }
            if (failed.binarySearch(key) >= 0) {
                earlierFailures.add(uri)
            } else {
                fresh.add(uri)
            }
        }
        /** Whether the failures are tried again in this sweep. */
        val retry: Boolean = retryFailed || fresh.isNotEmpty()
        /** The failures to try again, none when no reason to retry exists. */
        val retried: List<String> = if (retry) earlierFailures else emptyList()
        return SweepPlan(todo = fresh + retried, fresh = fresh.size, retried = retried.size)
    }

    /**
     * Returns the ledger after a sweep.
     *
     * @param done tracks that were measured or already cached, as URI and signature pairs
     * @param failures tracks that could not be decoded, as URI and signature pairs
     * @return the new ledger, and the failed URIs that had not failed with this signature before
     */
    fun record(done: List<Pair<String, Long>>, failures: List<Pair<String, Long>>): SweepRecord {
        /** Keys of the tracks now swept. */
        val doneKeys: Set<Long> = done.map { (uri, signature) -> keyOf(uri, signature) }.toSet()
        /** Keys of the tracks that failed now. */
        val failedKeys: Set<Long> = failures.map { (uri, signature) -> keyOf(uri, signature) }.toSet()
        /** Failed URIs whose key was not in the failed array before. */
        val newFailures: List<String> = failures
            .filter { (uri, signature) -> failed.binarySearch(keyOf(uri, signature)) < 0 }
            .map { (uri, _) -> uri }
        /** Swept keys now: the earlier ones plus the new ones, minus any that failed this time. */
        val nextSwept: LongArray = (swept.toList() + doneKeys)
            .filter { key -> key !in failedKeys }
            .distinct()
            .sorted()
            .toLongArray()
        /** Failed keys now: the earlier ones plus the new failures, minus any that succeeded this time. */
        val nextFailed: LongArray = (failed.toList() + failedKeys)
            .filter { key -> key !in doneKeys }
            .distinct()
            .sorted()
            .toLongArray()
        return SweepRecord(ledger = SweepLedger(nextSwept, nextFailed), newFailures = newFailures)
    }

    /** Encodes the ledger: a marker, then the swept count and hashes, then the failed count and hashes. */
    fun toBytes(): ByteArray {
        /** Buffer sized for the marker, two counts and every hash. */
        val buffer: ByteBuffer = ByteBuffer.allocate(HEADER_BYTES + (swept.size + failed.size) * Long.SIZE_BYTES)
        buffer.putInt(LEDGER_MAGIC)
        buffer.putInt(swept.size)
        swept.forEach { hash -> buffer.putLong(hash) }
        buffer.putInt(failed.size)
        failed.forEach { hash -> buffer.putLong(hash) }
        return buffer.array()
    }

    /** Holds the empty ledger, the decoder and the hashes. */
    companion object {
        /** A ledger that remembers no track, which is the state of an install that never analyzed anything. */
        val EMPTY: SweepLedger = SweepLedger(LongArray(0), LongArray(0))

        /**
         * Decodes a ledger written by [toBytes]; bytes of any other shape give the empty ledger, so everything is
         * analyzed again.
         *
         * @param bytes stored form, possibly damaged or from an older version
         * @return the decoded ledger, or [EMPTY] when the bytes cannot be one
         */
        fun fromBytes(bytes: ByteArray): SweepLedger {
            if (bytes.size < HEADER_BYTES) {
                return EMPTY
            }
            /** Reader over the stored form. */
            val buffer: ByteBuffer = ByteBuffer.wrap(bytes)
            if (buffer.getInt() != LEDGER_MAGIC) {
                return EMPTY
            }
            /** Declared number of swept hashes. */
            val sweptSize: Int = buffer.getInt()
            if (sweptSize < 0 || buffer.remaining() < sweptSize.toLong() * Long.SIZE_BYTES + Int.SIZE_BYTES) {
                return EMPTY
            }
            /** Swept hashes in stored order. */
            val sweptHashes = LongArray(sweptSize) { buffer.getLong() }
            /** Declared number of failed hashes. */
            val failedSize: Int = buffer.getInt()
            if (failedSize < 0 || buffer.remaining() != failedSize * Long.SIZE_BYTES) {
                return EMPTY
            }
            /** Failed hashes in stored order. */
            val failedHashes = LongArray(failedSize) { buffer.getLong() }
            sweptHashes.sort()
            failedHashes.sort()
            return SweepLedger(sweptHashes, failedHashes)
        }

        /**
         * Hashes a URI with 64-bit FNV-1a over its UTF-16 units and mixes in the file signature.
         *
         * @param uri track URI
         * @param signature size and modification time signature, 0 when the scan did not report them
         * @return the key of this version of this file
         */
        internal fun keyOf(uri: String, signature: Long): Long {
            /** Running hash. */
            var hash: Long = FNV_OFFSET
            for (unit in uri) {
                hash = (hash xor unit.code.toLong()) * FNV_PRIME
            }
            return hash xor (signature * SIGNATURE_MIX)
        }
    }
}

// What:     `fun shouldStartAnalysis(sourceReadable: Boolean, plan: SweepPlan): Boolean` decides whether to start the
//           analysis service.
// Why:      The service shows a notification, so it starts only when there is something to analyze: a library that can
//           be read and a plan with at least one track.
//
// In TS you'd write (pseudocode):
// ```ts
// const shouldStartAnalysis = (sourceReadable: boolean, plan: SweepPlan) => sourceReadable && plan.todo.length > 0;
// ```
/**
 * Decides whether to start the analysis service.
 *
 * @param sourceReadable whether music access is held or a folder grant is held, so the library can be listed
 * @param plan what the next sweep would analyze
 * @return true when a library can be read and the plan holds at least one track
 */
fun shouldStartAnalysis(sourceReadable: Boolean, plan: SweepPlan): Boolean =
    sourceReadable && plan.todo.isNotEmpty()
