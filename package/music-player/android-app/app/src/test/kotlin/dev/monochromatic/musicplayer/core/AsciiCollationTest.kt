// The ASCII collation shortcut must order names exactly as the rail collator does.

// What:     `package ...core` places the cases beside the shortcut.
// Why:      The cases reach the internal object through the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

import java.util.Random
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies the weight model against the collator on every ASCII pair and on random strings. */
class AsciiCollationTest {
    /** The model fits the JVM collator, so the shortcut is active in the tests. */
    @Test
    fun theModelFitsTheCollator() {
        assertTrue(AsciiCollation.usable)
    }

    /** Compares one hundred thousand random ASCII string pairs, all 128 code points included. */
    @Test
    fun randomAsciiStringsOrderLikeTheCollator() {
        val collator = railCollator()
        val random = Random(99L)
        repeat(100_000) {
            val left = String(CharArray(random.nextInt(9)) { random.nextInt(128).toChar() })
            val right = String(CharArray(random.nextInt(9)) { random.nextInt(128).toChar() })
            val a = AsciiCollation.keyOf(left)!!
            val b = AsciiCollation.keyOf(right)!!
            assertEquals(
                "'${left.map { it.code }}' vs '${right.map { it.code }}'",
                Integer.signum(collator.compare(left, right)),
                Integer.signum(AsciiCollation.compare(a, b)),
            )
        }
    }

    /** Names with a character outside ASCII get no shortcut. */
    @Test
    fun nonAsciiNamesFallBack() {
        assertNull(AsciiCollation.keyOf("é"))
        assertNull(AsciiCollation.keyOf("aあ"))
    }
}

/** Verifies the unsigned byte comparison that replaces `Arrays.compareUnsigned`, which Android 8 to 12 lack. */
class CompareUnsignedBytesTest {
    /** Compares random byte strings with the JDK's own comparison, which this test host has. */
    @Test
    fun matchesTheJdkComparison() {
        val random = java.util.Random(77L)
        repeat(5000) {
            val left = ByteArray(random.nextInt(6)).also { random.nextBytes(it) }
            val right = ByteArray(random.nextInt(6)).also { bytes -> random.nextBytes(bytes) }
            if (random.nextInt(3) == 0 && left.isNotEmpty()) {
                left.copyInto(right, 0, 0, minOf(left.size, right.size))
            }
            assertEquals(
                Integer.signum(java.util.Arrays.compareUnsigned(left, right)),
                Integer.signum(compareUnsignedBytes(left, right)),
            )
        }
    }

    /** A byte above 127 sorts after one below it, and a prefix sorts first. */
    @Test
    fun treatsBytesAsUnsignedAndPrefixesFirst() {
        assertTrue(compareUnsignedBytes(byteArrayOf(0x80.toByte()), byteArrayOf(0x7F)) > 0)
        assertTrue(compareUnsignedBytes(byteArrayOf(1, 2), byteArrayOf(1, 2, 3)) < 0)
        assertEquals(0, compareUnsignedBytes(byteArrayOf(), byteArrayOf()))
    }
}
