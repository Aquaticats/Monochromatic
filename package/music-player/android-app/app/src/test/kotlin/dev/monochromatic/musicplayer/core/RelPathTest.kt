// What:     `package dev.monochromatic.musicplayer.core` names the namespace this test file
//           lives under. It is the SAME package as the code under test (`RelPath.kt`), so this
//           file calls `relativeDisplayPaths` by its short name with no import. The package must
//           mirror the directory path.
// Why:      Sharing the package lets the tests reach the package-level `relativeDisplayPaths`
//           function without importing it; test and main source sets merge into one package at
//           compile time.
//
// In TS you'd write (pseudocode):
// ```ts
// // No keyword — the file path IS the module; here it shares a namespace with the SUT.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import org.junit.Assert.assertEquals` imports the static `assertEquals` function
//           from JUnit 4's `org.junit.Assert` class, callable unqualified.
// Why:      The value-equality assertions below need it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assertEquals } from "@junit/assert";
// ```
import org.junit.Assert.assertEquals

// What:     `import org.junit.Assert.assertTrue` imports the static `assertTrue` function
//           (asserts a `Boolean` is `true`).
// Why:      The emptiness assertion below (`assertTrue(relativeDisplayPaths(...).isEmpty())`)
//           needs it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assertTrue } from "@junit/assert";
// ```
import org.junit.Assert.assertTrue

// What:     `import org.junit.Test` imports the `Test` ANNOTATION class (a type) used as the
//           `@Test` marker on each test method; the runner runs every method tagged with it.
// Why:      Without it we could not write `@Test`, and the runner would find no tests.
//
// In TS you'd write (pseudocode):
// ```ts
// import { test } from "vitest"; // each @Test method becomes a test("...", () => {...})
// ```
import org.junit.Test

// =============================================================================
// File summary (folds in the old class KDoc's domain content)
// =============================================================================
//
// Host-JVM unit tests for `relativeDisplayPaths`, ported one-to-one from the desktop player's
// `relpath_tests.rs` so the Kotlin port stays faithful to the Rust behaviour. The cases pin how
// a queue's absolute track paths are reduced to display strings RELATIVE to their common root:
// empty input yields empty; a single track keeps only its filename; distinct albums keep their
// relative folders; a single shared folder yields bare filenames; mixed-depth paths strip only
// the SHARED top; and duplicate paths each keep their filename.

// What:     `class RelPathTest { ... }` declares a JUnit 4 test class the runner instantiates to
//           invoke each `@Test`-marked method.
// Why:      Groups every test for the `RelPath.kt` `relativeDisplayPaths` function.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("RelPath", () => {
//   // ...each @Test fun below becomes a test(...) call inside here...
// });
// ```
class RelPathTest {
    // What:     `@Test` is an ANNOTATION (metadata, no code) marking the method below as a test
    //           the JUnit runner executes and reports.
    // Why:      Registers `emptyInputYieldsEmpty` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty input yields empty", () => {
    // ```
    @Test
    // What:     `fun emptyInputYieldsEmpty() { ... }` declares a no-parameter test method
    //           returning `Unit` (Kotlin's "void"), block body.
    // Why:      Pins that an empty input list yields an empty output list (no common-prefix math
    //           on zero paths, no crash).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...assertion below... *\/ }
    // ```
    fun emptyInputYieldsEmpty() {
        // What:     `assertTrue(relativeDisplayPaths(emptyList()).isEmpty())` is the single-arg
        //           `assertTrue(condition)`. The condition is built by:
        //           - `emptyList()` — a stdlib factory returning a shared zero-length read-only
        //             `List` (sibling: `listOf(...)` for a populated list).
        //           - `relativeDisplayPaths(...)` — the function under test, returning
        //             `List<String>`.
        //           - `.isEmpty()` — a `List` predicate, true when there are zero elements.
        // Why:      Assert an empty input produces an empty result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(relativeDisplayPaths([]).length === 0).toBe(true);
        // ```
        assertTrue(relativeDisplayPaths(emptyList()).isEmpty())
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `singleTrackKeepsOnlyFilename` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("single track keeps only filename", () => {
    // ```
    @Test
    // What:     `fun singleTrackKeepsOnlyFilename() { ... }` declares a no-arg `Unit`-returning
    //           test method, block body.
    // Why:      Pins that with one track, the WHOLE directory prefix is the common prefix, so only
    //           the bare filename remains. A single-track queue shows just the file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...assertion below... *\/ }
    // ```
    fun singleTrackKeepsOnlyFilename() {
        // What:     `assertEquals(listOf("01.flac"), relativeDisplayPaths(listOf("/music/Artist/Album/01.flac")))`
        //           is `assertEquals(expected, actual)` across multiple lines.
        //           - EXPECTED `listOf("01.flac")` is an immutable `List<String>` (factory, not
        //             `new`).
        //           - ACTUAL `relativeDisplayPaths(listOf("/music/Artist/Album/01.flac"))` strips
        //             the common prefix from a single absolute path, leaving the filename.
        // Why:      With one path, everything up to the filename is "shared", so only `01.flac`
        //           survives.
        // Gotcha:   Argument order: `assertEquals(expected, actual)` is backwards from
        //           `expect(actual)`; list equality is structural.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(relativeDisplayPaths(["/music/Artist/Album/01.flac"])).toEqual(["01.flac"]);
        // ```
        assertEquals(
            listOf("01.flac"),
            relativeDisplayPaths(listOf("/music/Artist/Album/01.flac")),
        )
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `distinctAlbumsKeepRelativeFolders` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("distinct albums keep relative folders", () => {
    // ```
    @Test
    // What:     `fun distinctAlbumsKeepRelativeFolders() { ... }` declares a no-arg
    //           `Unit`-returning test method, block body.
    // Why:      Pins that when two paths share only `/music/`, the part AFTER that shared root
    //           (the differing `A/Alb/...` vs `B/Alb/...`) is kept, so the UI shows each track's
    //           folder.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...assertion below... *\/ }
    // ```
    fun distinctAlbumsKeepRelativeFolders() {
        // What:     `assertEquals(listOf("A/Alb/01.flac", "B/Alb/01.flac"), relativeDisplayPaths(listOf("/music/A/Alb/01.flac", "/music/B/Alb/01.flac")))`
        //           is `assertEquals(expected, actual)`. EXPECTED is an immutable `List<String>` of
        //           the relative paths; ACTUAL strips the shared `/music/` from both absolute paths
        //           (the input is an immutable `listOf(...)`).
        // Why:      The common prefix is only `/music/`, so the differing folder tails are kept.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(relativeDisplayPaths(["/music/A/Alb/01.flac", "/music/B/Alb/01.flac"])).toEqual(["A/Alb/01.flac", "B/Alb/01.flac"]);
        // ```
        assertEquals(
            listOf("A/Alb/01.flac", "B/Alb/01.flac"),
            relativeDisplayPaths(listOf("/music/A/Alb/01.flac", "/music/B/Alb/01.flac")),
        )
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `withoutARootTheSharedDirectoryIsStripped` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("without a root the shared directory is stripped", () => {
    // ```
    @Test
    // What:     `fun withoutARootTheSharedDirectoryIsStripped() { ... }` declares a no-arg `Unit`-returning
    //           test method, block body.
    // Why:      Pins that without a library root (D86) the whole shared folder (`/m/A/Alb/`) is the common
    //           prefix, so only the filenames remain. With a root the folder keeps its name; see DisplayRootTest.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...assertion below... *\/ }
    // ```
    fun withoutARootTheSharedDirectoryIsStripped() {
        // What:     `assertEquals(listOf("01.flac", "02.flac"), relativeDisplayPaths(listOf("/m/A/Alb/01.flac", "/m/A/Alb/02.flac")))`
        //           is `assertEquals(expected, actual)`. EXPECTED is an immutable `List<String>`;
        //           ACTUAL strips the fully-shared `/m/A/Alb/` folder, leaving bare filenames.
        // Why:      Both paths share the entire folder, so only `01.flac` and `02.flac` remain.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(relativeDisplayPaths(["/m/A/Alb/01.flac", "/m/A/Alb/02.flac"])).toEqual(["01.flac", "02.flac"]);
        // ```
        assertEquals(
            listOf("01.flac", "02.flac"),
            relativeDisplayPaths(listOf("/m/A/Alb/01.flac", "/m/A/Alb/02.flac")),
        )
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `mixedDepthStripsOnlySharedTop` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("mixed depth strips only shared top", () => {
    // ```
    @Test
    // What:     `fun mixedDepthStripsOnlySharedTop() { ... }` declares a no-arg `Unit`-returning
    //           test method, block body.
    // Why:      Pins that paths of DIFFERENT depths strip only the shared TOP folder (`/m/`): a
    //           root-level `loose.flac` keeps its bare name while a nested track keeps its folder
    //           tail. The common prefix never eats past where the paths diverge.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...assertion below... *\/ }
    // ```
    fun mixedDepthStripsOnlySharedTop() {
        // What:     `assertEquals(listOf("loose.flac", "A/Alb/01.flac"), relativeDisplayPaths(listOf("/m/loose.flac", "/m/A/Alb/01.flac")))`
        //           is `assertEquals(expected, actual)`. EXPECTED is an immutable `List<String>`;
        //           ACTUAL strips only the shared `/m/` from both differing-depth paths.
        // Why:      The shared prefix is just `/m/`, so `loose.flac` (root) and `A/Alb/01.flac`
        //           (nested) keep everything after it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(relativeDisplayPaths(["/m/loose.flac", "/m/A/Alb/01.flac"])).toEqual(["loose.flac", "A/Alb/01.flac"]);
        // ```
        assertEquals(
            listOf("loose.flac", "A/Alb/01.flac"),
            relativeDisplayPaths(listOf("/m/loose.flac", "/m/A/Alb/01.flac")),
        )
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `duplicatePathsKeepFilename` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("duplicate paths keep filename", () => {
    // ```
    @Test
    // What:     `fun duplicatePathsKeepFilename() { ... }` declares a no-arg `Unit`-returning test
    //           method, block body.
    // Why:      Pins that two IDENTICAL paths each keep their filename (the common prefix is the
    //           whole folder), proving duplicates are not deduped or mishandled.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...assertion below... *\/ }
    // ```
    fun duplicatePathsKeepFilename() {
        // What:     `assertEquals(listOf("x.flac", "x.flac"), relativeDisplayPaths(listOf("/m/A/x.flac", "/m/A/x.flac")))`
        //           is `assertEquals(expected, actual)`. EXPECTED is an immutable `List<String>`
        //           with the filename twice; ACTUAL strips the shared `/m/A/` from both identical
        //           paths.
        // Why:      Two copies of the same path both reduce to `x.flac`; duplicates survive as two
        //           entries.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(relativeDisplayPaths(["/m/A/x.flac", "/m/A/x.flac"])).toEqual(["x.flac", "x.flac"]);
        // ```
        assertEquals(
            listOf("x.flac", "x.flac"),
            relativeDisplayPaths(listOf("/m/A/x.flac", "/m/A/x.flac")),
        )
    }
}
