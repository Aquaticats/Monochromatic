// A cheap stand-in for the rail collator on names that are plain ASCII. Building a `CollationKey` for a name costs
// microseconds, and a Search over a big library breaks ties between thousands of equally ranked names with them.
// For ASCII text the collator at primary strength depends only on a per-character weight, so a name can be turned into
// a short byte string whose plain comparison gives the collator's order. The weights are read from the collator itself
// and the model is checked against it when first used, so a platform whose collator does not fit it falls back.

// What:     `package dev.monochromatic.musicplayer.core` places the ASCII shortcut beside the ranking that uses it.
// Why:      Only the result order reads it, and the tests compare it with the collator directly.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import java.text.Collator

/** Number of ASCII code points, which is the size of the weight table. */
private const val ASCII_COUNT: Int = 0x80

/** Weight stored for a character the collator ignores at primary strength. */
private const val IGNORED: Int = -1

// What:     `internal object AsciiCollation` holds the per-character weights and the checked key builder.
// Why:      The weights are computed once, from the platform's own collator, so the shortcut tracks whatever order
//           that collator has instead of a copy of it that could drift.
//
// In TS you'd write (pseudocode):
// ```ts
// const AsciiCollation = { weights: computeWeights(), keyOf(name) { /* bytes or null */ } };
// ```
/** Primary-strength collation weights of the ASCII characters, validated against the collator. */
internal object AsciiCollation {
    /** Weight of each ASCII character: equal for characters the collator treats as equal, [IGNORED] if ignorable. */
    private val weights: IntArray = computeWeights(railCollator())

    /** Whether the weight model reproduced the collator on every pair of ASCII characters and on sample strings. */
    val usable: Boolean = validate(railCollator(), weights)

    /** Forces the one-time validation now and reports whether the shortcut is in use. */
    fun prepare(): Boolean = usable

    // What:     `fun keyOf(name: String): ByteArray?` turns a name into bytes whose order is the collator's order.
    // Why:      Comparing two byte arrays is a tight loop, where comparing two collation keys is a string comparison
    //           after an expensive build.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keyOf = (name: string): Uint8Array | null => (isAscii(name) ? Uint8Array.from(weightsOf(name)) : null);
    // ```
    /** Returns the weight string of an ASCII [name], or null for a name with other characters or an unusable model. */
    fun keyOf(name: String): ByteArray? {
        if (!usable) {
            return null
        }
        /** Weights of the characters that count, in order. */
        val key = ByteArray(name.length)
        /** Number of weights written to the key so far. */
        var size = 0
        for (unit in name) {
            if (unit.code >= ASCII_COUNT) {
                return null
            }
            /** Weight of this character, or [IGNORED]. */
            val weight: Int = weights[unit.code]
            if (weight != IGNORED) {
                key[size] = weight.toByte()
                size += 1
            }
        }
        return key.copyOf(size)
    }

    /** Compares two weight strings the way the collator compares the names they came from. */
    fun compare(left: ByteArray, right: ByteArray): Int = compareUnsignedBytes(left, right)
}

// What:     `internal fun compareUnsignedBytes(left: ByteArray, right: ByteArray): Int` compares two byte strings.
// Why:      `java.util.Arrays.compareUnsigned` needs Android API 33 and the app supports API 26, so the lexicographic
//           comparison of unsigned bytes is written out; a shorter string that is a prefix of the longer sorts first.
//
// In TS you'd write (pseudocode):
// ```ts
// const compareUnsignedBytes = (a: Uint8Array, b: Uint8Array) => { /* first difference, then length */ };
// ```
/** Returns a negative, zero or positive number as [left] sorts before, equal to or after [right], bytes unsigned. */
internal fun compareUnsignedBytes(left: ByteArray, right: ByteArray): Int {
    /** Number of leading positions both strings have. */
    val shared: Int = minOf(left.size, right.size)
    for (position in 0 until shared) {
        /** Difference of the two bytes read as unsigned values. */
        val difference: Int = (left[position].toInt() and BYTE_MASK) - (right[position].toInt() and BYTE_MASK)
        if (difference != 0) {
            return difference
        }
    }
    return left.size - right.size
}

/** Mask that reads a signed byte as an unsigned value. */
private const val BYTE_MASK: Int = 0xFF

/** Reads the weight of every ASCII character from [collator], or returns an empty table when it cannot be read. */
private fun computeWeights(collator: Collator): IntArray {
    /** Characters that are not ignorable, ordered by the collator. */
    val ordered: List<Char> = (0 until ASCII_COUNT).map { code -> code.toChar() }
        .filter { unit -> collator.compare(unit.toString(), "") != 0 }
        .sortedWith { left, right -> collator.compare(left.toString(), right.toString()) }
    /** Weight of each ASCII character, [IGNORED] until the sorted order assigns one. */
    val table = IntArray(ASCII_COUNT) { IGNORED }
    /** Weight of the character group being assigned. */
    var weight = 0
    for (position in ordered.indices) {
        if (position > 0 && collator.compare(ordered[position - 1].toString(), ordered[position].toString()) != 0) {
            weight += 1
        }
        table[ordered[position].code] = weight
    }
    return table
}

/** Checks that comparing weight strings equals comparing the strings with [collator] on a battery of samples. */
private fun validate(collator: Collator, weights: IntArray): Boolean {
    /** Every ASCII character as a one-character sample. */
    val singles: List<String> = (0 until ASCII_COUNT).map { code -> code.toChar().toString() }
    /** Random short strings over a mixed alphabet, which exercises ignorable characters, case and punctuation. */
    val random = java.util.Random(VALIDATION_SEED)
    /** Random short strings over a mixed alphabet. */
    val mixed: List<String> = List(VALIDATION_STRINGS) {
        String(
            CharArray(random.nextInt(VALIDATION_MAX_LENGTH + 1)) {
                VALIDATION_ALPHABET[random.nextInt(VALIDATION_ALPHABET.length)]
            },
        )
    }
    for (left in singles + mixed) {
        for (right in singles + mixed.take(VALIDATION_PEERS)) {
            if (Integer.signum(collator.compare(left, right)) !=
                Integer.signum(
                    compareUnsignedBytes(keyForValidation(left, weights), keyForValidation(right, weights)),
                )
            ) {
                return false
            }
        }
    }
    return true
}

/** Seed of the validation strings, fixed so a failure repeats. */
private const val VALIDATION_SEED: Long = 12_345L

/** Number of random strings in the validation battery. */
private const val VALIDATION_STRINGS: Int = 400

/** Number of random strings each sample is compared with, besides the single characters. */
private const val VALIDATION_PEERS: Int = 60

/** Longest random validation string. */
private const val VALIDATION_MAX_LENGTH: Int = 7

/** Characters of the random validation strings: letters in both cases, digits, separators, punctuation, controls. */
private const val VALIDATION_ALPHABET: String = "aAbBcCzZ019 -_.([/'\"!#&+~\u0001\t\n"

/** Builds the weight string of [sample] from [weights], the same way [AsciiCollation.keyOf] does. */
private fun keyForValidation(sample: String, weights: IntArray): ByteArray {
    /** Weights of the sample's characters that count. */
    val key = ByteArray(sample.length)
    /** Number of weights written to the key so far. */
    var size = 0
    for (unit in sample) {
        if (weights[unit.code] != IGNORED) {
            key[size] = weights[unit.code].toByte()
            size += 1
        }
    }
    return key.copyOf(size)
}
