// What:     `package dev.monochromatic.musicplayer.core` places this test beside the derivation it checks.
// Why:      The test reaches the internal pure functions in the same package as the code under test.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit and `kotlin.math` bring in the assertions and the absolute value.
// Why:      Each role is compared with the Rust value channel by channel, and the largest difference is reported.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import kotlin.math.abs
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class ThemeRolesVectorTest` compares the Kotlin roles with the roles the desktop's Rust code derived.
// Why:      D108 and A3 want the same root to give the same theme on both platforms. The file
//           `src/test/resources/theme-roles-vector.txt` was written by the desktop's `theme_roles.rs` (see its header
//           for the source hashes), and this test recomputes every line.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("shared theme vector", () => { test("kotlin equals rust", () => { ... }); });
// ```
/**
 * Verifies that `deriveRoles` reproduces the Rust roles for every root of the vector, in both schemes.
 *
 * The tolerance is zero: every channel of every role equals the Rust value. The two implementations evaluate the same
 * formulas in 64-bit floats, but `Math.cbrt` and `Math.pow` on the JVM and `f64::cbrt` and `f64::powf` in Rust's system
 * math library could differ in the last bit and round one channel to the neighbouring step; the measured maximum is
 * printed by `everyRoleMatchesRustExactly` and asserted below. The desktop crate has the other half of the check: its
 * test `theme_roles::fixture_tests` requires every line of the same file to equal what `derive_roles` computes.
 */
class ThemeRolesVectorTest {
    /** One line of the vector: a root and the 18 role values of one scheme. */
    private data class VectorLine(
        /** Name of the root, for the failure message. */
        val rootName: String,
        /** The root color. */
        val root: Srgb8,
        /** Scheme of this line. */
        val scheme: ThemeScheme,
        /** Role values by name, in vector order. */
        val roles: List<Pair<String, Srgb8>>,
    )

    /** Parses the resource into lines, failing loudly if the resource is missing. */
    private fun readVector(): List<VectorLine> {
        val stream = javaClass.classLoader!!.getResourceAsStream(VECTOR_RESOURCE)
            ?: error("The shared vector $VECTOR_RESOURCE is missing from the test resources.")
        return stream.bufferedReader().readLines()
            .filter { line -> line.isNotBlank() && !line.startsWith("#") }
            .map { line ->
                val parts: List<String> = line.split(" ")
                VectorLine(
                    rootName = parts[0],
                    root = Srgb8.fromRgbInt(parts[1].toInt(HEX_RADIX)),
                    scheme = if (parts[2] == "dark") ThemeScheme.DARK else ThemeScheme.LIGHT,
                    roles = parts.drop(FIRST_ROLE_PART).map { pair ->
                        val (name, hex) = pair.split("=")
                        name to Srgb8.fromRgbInt(hex.toInt(HEX_RADIX))
                    },
                )
            }
    }

    /** Largest absolute channel difference between two colors. */
    private fun channelDistance(first: Srgb8, second: Srgb8): Int =
        maxOf(abs(first.red - second.red), abs(first.green - second.green), abs(first.blue - second.blue))

    /** Confirms the vector covers at least 52 roots in both schemes and every named case of the task. */
    @Test
    fun theVectorCoversTheRequiredRoots() {
        val lines: List<VectorLine> = readVector()
        val names: Set<String> = lines.map { line -> line.rootName }.toSet()
        assertTrue("only ${names.size} roots", names.size >= MINIMUM_ROOTS)
        assertEquals(names.size, lines.count { line -> line.scheme == ThemeScheme.DARK })
        assertEquals(names.size, lines.count { line -> line.scheme == ThemeScheme.LIGHT })
        for (required in listOf("baseline", "periwinkle", "red", "yellow", "green", "blue", "black", "white")) {
            assertTrue("missing root $required", required in names)
        }
        assertTrue("no near-gray root", names.any { name -> name.startsWith("gray") })
        for (required in NEAR_GRAY_AND_NEUTRAL_ROOTS) {
            assertTrue("missing root ${required.first}", required.first in names)
            for (line in lines.filter { line -> line.rootName == required.first }) {
                assertEquals("root ${required.first}", Srgb8.fromRgbInt(required.second), line.root)
            }
        }
        assertEquals(12, names.count { name -> name.startsWith("sweep-mid-") })
        assertEquals(12, names.count { name -> name.startsWith("sweep-deep-") })
        assertTrue(lines.all { line -> line.roles.size == EXPECTED_ROLE_COUNT })
    }

    /** Confirms every role of every line equals the Rust value exactly and reports the largest difference. */
    @Test
    fun everyRoleMatchesRustExactly() {
        var largest = 0
        var exact = 0
        var total = 0
        for (line in readVector()) {
            val derived: List<Pair<String, Srgb8>> = deriveScheme(rootTerms(line.root), line.scheme).toNamedList()
            assertEquals(line.roles.map { pair -> pair.first }, derived.map { pair -> pair.first })
            for ((expected, actual) in line.roles.zip(derived)) {
                val distance: Int = channelDistance(expected.second, actual.second)
                largest = maxOf(largest, distance)
                total += 1
                if (distance == 0) {
                    exact += 1
                }
                assertTrue(
                    "${line.rootName} ${line.scheme} ${expected.first}: rust ${expected.second.toHex()} " +
                        "kotlin ${actual.second.toHex()}",
                    distance <= TOLERANCE,
                )
            }
        }
        println("shared vector: $exact of $total roles exactly equal, largest channel difference $largest")
        assertTrue("the largest difference $largest exceeds $TOLERANCE", largest <= TOLERANCE)
    }

    /** Confirms the baseline root (the 4E5E8B line) and the absent-accent call derive the same palette. */
    @Test
    fun noAccentIsTheBaselineLine() {
        assertEquals(deriveRoles(null), deriveRoles(BASELINE_ROOT))
    }

    private companion object {
        /** Resource path of the vector written by the desktop code. */
        const val VECTOR_RESOURCE: String = "theme-roles-vector.txt"

        /** Radix of the hexadecimal colors. */
        const val HEX_RADIX: Int = 16

        /** Number of leading fields before the role pairs: root name, root color, scheme. */
        const val FIRST_ROLE_PART: Int = 3

        /** Number of roles per line. */
        const val EXPECTED_ROLE_COUNT: Int = 18

        /** Fewest roots the vector must hold, each in both schemes. */
        const val MINIMUM_ROOTS: Int = 52

        /**
         * Roots D118 draws its line with: the first six have unequal channels and must still tint every gray, the
         * last four have equal channels and must give true grays; each with its color.
         */
        val NEAR_GRAY_AND_NEUTRAL_ROOTS: List<Pair<String, Int>> = listOf(
            "weak-808088" to 0x808088, "weak-7F8080" to 0x7F8080, "weak-8A8A8B" to 0x8A8A8B,
            "weak-8A8B8A" to 0x8A8B8A, "weak-808081" to 0x808081, "weak-FEFFFF" to 0xFEFFFF,
            "neutral-8A8A8A" to 0x8A8A8A, "gray-mid" to 0x808080, "black" to 0x000000, "white" to 0xFFFFFF,
        )

        /** Accepted difference per channel, in 8-bit steps. */
        const val TOLERANCE: Int = 0
    }
}
