// Unit tests for the order of persistable grant flags requested for a picked folder.

// What:     `package dev.monochromatic.musicplayer.core` places this test beside the grant ordering it checks.
// Why:      The test reaches the public ordering function in the same package.
//
// In TS you'd write (pseudocode):
// ```ts
// // File path supplies the test module namespace.
// ```
package dev.monochromatic.musicplayer.core

// What:     Imports from JUnit bring in the assertions and the test annotation.
// Why:      Each ordering is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

// What:     `private const val TEST_READ_FLAG: Int` and `TEST_WRITE_FLAG` are distinct single-bit flags.
// Why:      Distinct bits let the tests see whether the write bit was added to the first attempt only.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEST_READ_FLAG = 1; const TEST_WRITE_FLAG = 2;
// ```
/** Read flag used by the grant ordering tests. */
private const val TEST_READ_FLAG: Int = 1

/** Write flag used by the grant ordering tests, a bit distinct from the read flag. */
private const val TEST_WRITE_FLAG: Int = 2

// What:     `class FolderGrantAttemptsTest` groups the tests for the grant ordering.
// Why:      The fallback order decides whether a read-only picker grant still persists.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("folder grant attempts", () => { ... });
// ```
/** Verifies that read and write are requested first and read only is the fallback. */
class FolderGrantAttemptsTest {
    // What:     `readWriteIsFirstAttempt` checks that the first request carries both flags.
    // Why:      A picker that grants write access must keep write access across restarts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("first", () => expect(folderGrantAttempts(1, 2)[0]).toBe(3));
    // ```
    /** Confirms the first attempt is the union of the read and write flags. */
    @Test
    fun readWriteIsFirstAttempt() {
        assertEquals(TEST_READ_FLAG or TEST_WRITE_FLAG, folderGrantAttempts(TEST_READ_FLAG, TEST_WRITE_FLAG)[0])
    }

    // What:     `readOnlyIsFallbackAttempt` checks that the second request carries the read flag alone.
    // Why:      A provider that refuses write access must still accept the read-only grant the library needs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("fallback", () => expect(folderGrantAttempts(1, 2)[1]).toBe(1));
    // ```
    /** Confirms the fallback attempt is the read flag alone, without the write bit. */
    @Test
    fun readOnlyIsFallbackAttempt() {
        assertEquals(TEST_READ_FLAG, folderGrantAttempts(TEST_READ_FLAG, TEST_WRITE_FLAG)[1])
    }

    // What:     `alwaysTwoAttempts` checks the list length.
    // Why:      The caller's fallback loop assumes exactly two attempts, the last one being read only.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("length", () => expect(folderGrantAttempts(1, 2).length).toBe(2));
    // ```
    /** Confirms the ordering always lists exactly two attempts. */
    @Test
    fun alwaysTwoAttempts() {
        assertEquals(2, folderGrantAttempts(TEST_READ_FLAG, TEST_WRITE_FLAG).size)
    }

    // What:     `neverWriteOnlyAttempt` checks that no attempt requests write without read.
    // Why:      A write-only grant is not a request the provider contract describes, so the list must never contain one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("no write only", () => expect(folderGrantAttempts(1, 2)).not.toContain(2));
    // ```
    /** Confirms no attempt is the write flag by itself. */
    @Test
    fun neverWriteOnlyAttempt() {
        assertEquals(false, folderGrantAttempts(TEST_READ_FLAG, TEST_WRITE_FLAG).contains(TEST_WRITE_FLAG))
    }
}
