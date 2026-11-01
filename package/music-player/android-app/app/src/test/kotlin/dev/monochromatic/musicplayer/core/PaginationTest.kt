// What:     `package dev.monochromatic.musicplayer.core` names the namespace this test file
//           lives under. It is the SAME package as the code under test (`Pagination.kt`,
//           `Page.kt`), so this file calls `paginate`/`pageOfIndex` and uses `Page`/`PageEntry`
//           by their short names with no import. The package must mirror the directory path.
// Why:      Sharing the package lets the tests reach the package-level functions and the
//           `Page`/`PageEntry` types without importing them; test and main source sets merge
//           into one package at compile time.
//
// In TS you'd write (pseudocode):
// ```ts
// // No keyword — the file path IS the module; here it shares a namespace with the SUT.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import org.junit.Assert.assertEquals` imports the static `assertEquals` function
//           from JUnit 4's `org.junit.Assert` class, so it can be called unqualified.
// Why:      The value-equality assertions below (`assertEquals(expected, actual)`) need it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assertEquals } from "@junit/assert";
// ```
import org.junit.Assert.assertEquals

// What:     `import org.junit.Assert.assertNull` imports the static `assertNull` function,
//           which FAILS the test unless its argument is `null`. This import is NOT present in
//           the other test files; it is here because `pageOfIndex` can return `null`.
// Why:      The miss assertion below (`assertNull(pageOfIndex(pages, 99))`) needs it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assertNull } from "@junit/assert";
// ```
import org.junit.Assert.assertNull

// What:     `import org.junit.Assert.assertTrue` imports the static `assertTrue` function
//           (asserts a `Boolean` is `true`) from `org.junit.Assert`.
// Why:      The emptiness assertion below (`assertTrue(paginate(...).isEmpty())`) needs it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { assertTrue } from "@junit/assert";
// ```
import org.junit.Assert.assertTrue

// What:     `import org.junit.Test` imports the `Test` ANNOTATION class (a type, not a
//           function) used as the `@Test` marker on each test method; the runner runs every
//           method tagged with it.
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
// Host-JVM unit tests for `paginate` and `pageOfIndex`, ported one-to-one from the desktop
// player's inline `pagination.rs` test module so the Kotlin port stays faithful to the Rust
// behaviour. Every input vector and expected value matches the Rust oracle; pagination is
// pure (no RNG, JSON, or filesystem), so no test needed adaptation beyond translating
// `Vec`/`Some`/`None` to Kotlin `List`/`Int?`/`null`. The cases pin: empty input yields zero
// pages (not one empty page); same-top-folder collapse to one folder page; distinct folders
// sorted by path; case-insensitive folder and root-letter grouping with original casing kept
// in labels; the `#` catch-all for non-English roots; the folders-then-letters-then-`#` sort
// order; and `pageOfIndex` hit/miss (`Int?`) lookup.

// What:     `class PaginationTest { ... }` declares a JUnit 4 test class the runner
//           instantiates to invoke each `@Test`-marked method. Unlike the other test classes,
//           this one also holds two private HELPER methods (below) shared by several tests.
// Why:      Groups every pagination test, plus the two small extraction helpers they reuse.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("Pagination", () => {
//   // ...helpers + each @Test fun become functions / test(...) calls inside here...
// });
// ```
class PaginationTest {
    // What:     `private fun indicesOf(page: Page): List<Int> = page.entries.map { it.index }`
    //           declares a PRIVATE helper method (visible only inside this class), taking one
    //           `Page` parameter and returning a read-only `List<Int>`. The `=` (no `{ }`) is an
    //           EXPRESSION BODY: the single expression after it IS the return value.
    //           - `page.entries` reads the page's read-only `List<PageEntry>` property.
    //           - `.map { it.index }` runs a TRAILING LAMBDA over each entry, pulling out its
    //             `index` field. `it` is Kotlin's implicit name for the lambda's single
    //             parameter (one `PageEntry`). The result `List<Int>` is the return.
    // Why:      Pull just the load-order indices out of a page's entries, mirroring the Rust
    //           test helper, so assertions can compare index lists directly.
    // Gotcha:   `it` is the auto-named single lambda parameter (one `PageEntry` here), not a
    //           keyword; `.map` returns a NEW list, it does not mutate `entries`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function indicesOf(page: Page): number[] {
    //   return page.entries.map((entry) => entry.index);
    // }
    // ```
    private fun indicesOf(page: Page): List<Int> = page.entries.map { it.index }

    // What:     `private fun labelsOf(pages: List<Page>): List<String> = pages.map { it.label }`
    //           declares a second private helper, taking a read-only `List<Page>` and returning
    //           a read-only `List<String>`, expression body. `.map { it.label }` runs a trailing
    //           lambda over each `Page`, pulling its `label` field; `it` is the implicit single
    //           parameter (one `Page`).
    // Why:      Collect page labels in tab order, mirroring the Rust `pages.iter().map(|p| p.label)`,
    //           so assertions can compare label lists directly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function labelsOf(pages: Page[]): string[] {
    //   return pages.map((page) => page.label);
    // }
    // ```
    private fun labelsOf(pages: List<Page>): List<String> = pages.map { it.label }

    // What:     `@Test` is an ANNOTATION (metadata, no code) marking the method below as a test
    //           the JUnit runner executes and reports.
    // Why:      Registers `emptyInputYieldsNoPages` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty input yields no pages", () => {
    // ```
    @Test
    // What:     `fun emptyInputYieldsNoPages() { ... }` declares a no-parameter test method
    //           returning `Unit` (Kotlin's "void"), block body. The name is the report label.
    // Why:      Ported from the Rust `empty_input_yields_no_pages`: NO names means NO pages, not
    //           one empty page. This matters because the UI must show zero tabs for an empty
    //           queue, not a single blank tab.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...assertion below... *\/ }
    // ```
    fun emptyInputYieldsNoPages() {
        // What:     `assertTrue(paginate(emptyList()).isEmpty())` calls the SINGLE-argument
        //           `assertTrue(condition)`. The condition is built by:
        //           - `emptyList()` — a stdlib factory returning a shared, zero-length read-only
        //             `List` (sibling factories: `listOf(...)` for a populated list,
        //             `mutableListOf()` for a mutable one).
        //           - `paginate(...)` — the function under test, returning `List<Page>`.
        //           - `.isEmpty()` — a `List` predicate returning `true` when there are zero
        //             pages.
        // Why:      Assert paginating an empty input produces an empty page list.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(paginate([]).length === 0).toBe(true);
        // ```
        assertTrue(paginate(emptyList()).isEmpty())
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `sameTopFolderCollapsesOneLevel` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("same top folder collapses one level", () => {
    // ```
    @Test
    // What:     `fun sameTopFolderCollapsesOneLevel() { ... }` declares a no-arg `Unit`-returning
    //           test method, block body.
    // Why:      Ported from the Rust `same_top_folder_collapses_one_level`: two tracks in
    //           DIFFERENT deeper subfolders (`Album1`, `Album2`) but the SAME top folder
    //           (`Artist`) collapse onto one `Artist` page, keeping their load-order indices.
    //           This pins the one-level folder grouping (deeper nesting does not create more
    //           pages).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...arrange + assertions below... *\/ }
    // ```
    fun sameTopFolderCollapsesOneLevel() {
        // What:     `val pages = paginate(listOf("Artist/Album1/01.flac", "Artist/Album2/01.flac"))`
        //           declares a read-only local `pages` (`val`; type `List<Page>` inferred). It
        //           paginates a `listOf(...)` immutable-list of two subfolder display paths.
        // Why:      Capture the page list once for the three assertions below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pages = paginate(["Artist/Album1/01.flac", "Artist/Album2/01.flac"]);
        // ```
        val pages = paginate(listOf("Artist/Album1/01.flac", "Artist/Album2/01.flac"))
        // What:     `assertEquals(1, pages.size)` is `assertEquals(expected, actual)`: EXPECTED
        //           is the `Int` literal `1`; ACTUAL is `pages.size`, the `Int` element count of
        //           the `List<Page>`.
        // Why:      Both tracks share a top folder, so exactly one page exists.
        // Gotcha:   `assertEquals(expected, actual)` is backwards from `expect(actual)`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pages.length).toEqual(1);
        // ```
        assertEquals(1, pages.size)
        // What:     `assertEquals("Artist", pages[0].label)` is `assertEquals(expected, actual)`.
        //           ACTUAL is `pages[0].label`: `pages[0]` indexes the list at position 0 (one
        //           `Page`), and `.label` reads that page's `String` label property.
        // Why:      The single page's label is the shared top folder name `Artist`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pages[0].label).toEqual("Artist");
        // ```
        assertEquals("Artist", pages[0].label)
        // What:     `assertEquals(listOf(0, 1), indicesOf(pages[0]))` is `assertEquals(expected, actual)`.
        //           - EXPECTED `listOf(0, 1)` is an immutable `List<Int>` (factory function, not
        //             `new`).
        //           - ACTUAL `indicesOf(pages[0])` calls the private helper on the first page,
        //             returning its entries' load-order indices.
        // Why:      Confirm both tracks landed on this page in load order (indices 0 then 1).
        // Gotcha:   List equality here is structural (element-by-element), like TS `toEqual`, not
        //           reference identity.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(indicesOf(pages[0])).toEqual([0, 1]);
        // ```
        assertEquals(listOf(0, 1), indicesOf(pages[0]))
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `distinctFoldersSortedByPath` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("distinct folders sorted by path", () => {
    // ```
    @Test
    // What:     `fun distinctFoldersSortedByPath() { ... }` declares a no-arg `Unit`-returning
    //           test method, block body.
    // Why:      Ported from the Rust `distinct_folders_sorted_by_path`: two tracks in different
    //           folders, given OUT of sorted order (`Pop` before `Jazz`), come out as separate
    //           pages SORTED by folder path, each entry keeping its original load index. Pins
    //           that pages sort by path while entries remember where they came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...arrange + assertions below... *\/ }
    // ```
    fun distinctFoldersSortedByPath() {
        // What:     `val pages = paginate(listOf("Pop/b.flac", "Jazz/a.flac"))` declares a
        //           read-only `List<Page>` local `pages`, paginating two folder paths given with
        //           `Pop` before `Jazz` (deliberately not yet sorted).
        // Why:      Capture the page list once for the assertions below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pages = paginate(["Pop/b.flac", "Jazz/a.flac"]);
        // ```
        val pages = paginate(listOf("Pop/b.flac", "Jazz/a.flac"))
        // What:     `assertEquals(listOf("Jazz", "Pop"), labelsOf(pages))` is
        //           `assertEquals(expected, actual)`. EXPECTED `listOf("Jazz", "Pop")` is an
        //           immutable `List<String>`; ACTUAL `labelsOf(pages)` calls the private helper to
        //           collect labels in tab order.
        // Why:      Despite the input order, the pages come out sorted by path: `Jazz` before
        //           `Pop`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(labelsOf(pages)).toEqual(["Jazz", "Pop"]);
        // ```
        assertEquals(listOf("Jazz", "Pop"), labelsOf(pages))
        // What:     `assertEquals(1, pages[0].entries[0].index)` is `assertEquals(expected, actual)`.
        //           ACTUAL `pages[0].entries[0].index` chains three accesses: `pages[0]` (first
        //           page), `.entries` (its `List<PageEntry>`), `[0]` (first entry), `.index` (that
        //           entry's `Int` load index). EXPECTED is the `Int` literal `1`. (Folds in the
        //           original inline note: the Jazz page holds the second input, load index 1,
        //           despite sorting first.)
        // Why:      The `Jazz` page sorts FIRST but holds the SECOND input (`Jazz/a.flac`, load
        //           index 1), proving entries keep their original load index across the sort.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pages[0].entries[0].index).toEqual(1);
        // ```
        assertEquals(1, pages[0].entries[0].index)
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `folderPagesSortCaseInsensitively` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder pages sort case-insensitively", () => {
    // ```
    @Test
    // What:     `fun folderPagesSortCaseInsensitively() { ... }` declares a no-arg
    //           `Unit`-returning test method, block body.
    // Why:      Ported from the Rust `folder_pages_sort_case_insensitively`: folder tops with
    //           mixed-case first letters, given in an order a raw codepoint sort would mangle,
    //           interleave CASE-INSENSITIVELY while their displayed labels keep their original
    //           casing (`r-906` precedes `Reol` because `-` sorts before `E`). Pins that the
    //           sort key is case-folded but the label text is not.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...arrange + assertion below... *\/ }
    // ```
    fun folderPagesSortCaseInsensitively() {
        // What:     `val pages = paginate(listOf( ... ))` declares a read-only `List<Page>` local
        //           `pages`. The `listOf(...)` immutable-list factory holds four folder paths in a
        //           deliberately awkward order; the call spans several lines for readability.
        // Why:      Capture the paginated, sorted pages for the assertion below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pages = paginate(["Zedd/a.flac", "daniwellP/b.flac", "Reol/c.flac", "r-906/d.flac"]);
        // ```
        val pages = paginate(
            listOf(
                "Zedd/a.flac",
                "daniwellP/b.flac",
                "Reol/c.flac",
                "r-906/d.flac",
            ),
        )
        // What:     `assertEquals(listOf("daniwellP", "r-906", "Reol", "Zedd"), labelsOf(pages))`
        //           is `assertEquals(expected, actual)`. EXPECTED is an immutable `List<String>`
        //           of the labels in case-insensitive sort order; ACTUAL is `labelsOf(pages)`.
        // Why:      Verify the case-folded ordering (`daniwellP`, `r-906`, `Reol`, `Zedd`) while
        //           each label keeps its original casing.
        // Gotcha:   `r-906` sorts before `Reol` because, after case-folding both to `r...`, the
        //           next characters compare `-` (code 45) before `e`, so the hyphen wins.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(labelsOf(pages)).toEqual(["daniwellP", "r-906", "Reol", "Zedd"]);
        // ```
        assertEquals(listOf("daniwellP", "r-906", "Reol", "Zedd"), labelsOf(pages))
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `caseVariantFoldersStayDistinctPages` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("case variant folders stay distinct pages", () => {
    // ```
    @Test
    // What:     `fun caseVariantFoldersStayDistinctPages() { ... }` declares a no-arg
    //           `Unit`-returning test method, block body.
    // Why:      Ported from the Rust `case_variant_folders_stay_distinct_pages`: two folders
    //           differing ONLY in case (`REOL` vs `Reol`) stay SEPARATE pages, ordered by the
    //           original label after the shared case-folded key (uppercase-led first). Pins that
    //           case-folding orders pages but does not MERGE folder pages that differ only in
    //           case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...arrange + assertions below... *\/ }
    // ```
    fun caseVariantFoldersStayDistinctPages() {
        // What:     `val pages = paginate(listOf("REOL/a.flac", "Reol/b.flac"))` declares a
        //           read-only `List<Page>` local `pages`, paginating two folder paths that differ
        //           only in case.
        // Why:      Capture the pages for the assertions below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pages = paginate(["REOL/a.flac", "Reol/b.flac"]);
        // ```
        val pages = paginate(listOf("REOL/a.flac", "Reol/b.flac"))
        // What:     `assertEquals(2, pages.size)` is `assertEquals(expected, actual)`: EXPECTED
        //           `Int` `2`, ACTUAL `pages.size` (page count).
        // Why:      The two case-variant folders produce TWO pages, not one merged page.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pages.length).toEqual(2);
        // ```
        assertEquals(2, pages.size)
        // What:     `assertEquals(listOf("REOL", "Reol"), labelsOf(pages))` is
        //           `assertEquals(expected, actual)`. EXPECTED is an immutable `List<String>`;
        //           ACTUAL is `labelsOf(pages)`.
        // Why:      Confirm the two distinct pages, ordered uppercase-led first (`REOL` before
        //           `Reol`) after the shared case-folded key.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(labelsOf(pages)).toEqual(["REOL", "Reol"]);
        // ```
        assertEquals(listOf("REOL", "Reol"), labelsOf(pages))
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `rootLettersMergeCaseInsensitively` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("root letters merge case-insensitively", () => {
    // ```
    @Test
    // What:     `fun rootLettersMergeCaseInsensitively() { ... }` declares a no-arg
    //           `Unit`-returning test method, block body.
    // Why:      Ported from the Rust `root_letters_merge_case_insensitively`: three ROOT-level
    //           names (no `/`) starting with the same letter in different cases merge onto one
    //           `A` letter page, indices preserved in load order. Pins the case-insensitive
    //           first-letter bucketing for root tracks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...arrange + assertions below... *\/ }
    // ```
    fun rootLettersMergeCaseInsensitively() {
        // What:     `val pages = paginate(listOf("apple.flac", "Apricot.flac", "AVOCADO.flac"))`
        //           declares a read-only `List<Page>` local `pages`, paginating three root-level
        //           names all starting with `a`/`A` in different cases.
        // Why:      Capture the pages for the assertions below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pages = paginate(["apple.flac", "Apricot.flac", "AVOCADO.flac"]);
        // ```
        val pages = paginate(listOf("apple.flac", "Apricot.flac", "AVOCADO.flac"))
        // What:     `assertEquals(1, pages.size)` is `assertEquals(expected, actual)`: EXPECTED
        //           `Int` `1`, ACTUAL the page count.
        // Why:      All three root names share the same letter bucket, so one page.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pages.length).toEqual(1);
        // ```
        assertEquals(1, pages.size)
        // What:     `assertEquals("A", pages[0].label)` is `assertEquals(expected, actual)`.
        //           ACTUAL `pages[0].label` reads the single page's label.
        // Why:      The merged letter page is labelled with the uppercase canonical letter `A`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pages[0].label).toEqual("A");
        // ```
        assertEquals("A", pages[0].label)
        // What:     `assertEquals(listOf(0, 1, 2), indicesOf(pages[0]))` is
        //           `assertEquals(expected, actual)`. EXPECTED is an immutable `List<Int>`; ACTUAL
        //           is the helper extracting the page's entry indices.
        // Why:      All three tracks landed on this page in load order (0, 1, 2).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(indicesOf(pages[0])).toEqual([0, 1, 2]);
        // ```
        assertEquals(listOf(0, 1, 2), indicesOf(pages[0]))
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `nonLetterRootNamesGoToCatchAll` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("non-letter root names go to catch-all", () => {
    // ```
    @Test
    // What:     `fun nonLetterRootNamesGoToCatchAll() { ... }` declares a no-arg `Unit`-returning
    //           test method, block body.
    // Why:      Ported from the Rust `non_letter_root_names_go_to_catch_all`: root-level names
    //           whose first character is a digit (`1`), CJK (`初`), symbol (`#`), and accented
    //           non-English letter (`é`) all land on the single `#` catch-all page. Pins that
    //           anything outside A-Z falls into one bucket.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...arrange + assertions below... *\/ }
    // ```
    fun nonLetterRootNamesGoToCatchAll() {
        // What:     `val pages = paginate(listOf("1 song.flac", "初音.flac", "#tag.flac", "élan.flac"))`
        //           declares a read-only `List<Page>` local `pages`, paginating four root-level
        //           names whose first characters are a digit, CJK, a symbol, and an accented
        //           letter.
        // Why:      Capture the pages for the assertions below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pages = paginate(["1 song.flac", "初音.flac", "#tag.flac", "élan.flac"]);
        // ```
        val pages = paginate(listOf("1 song.flac", "初音.flac", "#tag.flac", "Øystein.flac"))
        // What:     `assertEquals(1, pages.size)` is `assertEquals(expected, actual)`: EXPECTED
        //           `Int` `1`, ACTUAL the page count.
        // Why:      All four non-English-letter roots collapse into one catch-all page.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pages.length).toEqual(1);
        // ```
        assertEquals(1, pages.size)
        // What:     `assertEquals("#", pages[0].label)` is `assertEquals(expected, actual)`.
        //           ACTUAL `pages[0].label` reads the single page's label.
        // Why:      The catch-all page is labelled with the literal `#`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pages[0].label).toEqual("#");
        // ```
        assertEquals("#", pages[0].label)
        // What:     `assertEquals(listOf(0, 1, 2, 3), indicesOf(pages[0]))` is
        //           `assertEquals(expected, actual)`. EXPECTED is an immutable `List<Int>`; ACTUAL
        //           extracts the page's entry indices.
        // Why:      All four tracks landed on the catch-all page in load order (0, 1, 2, 3).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(indicesOf(pages[0])).toEqual([0, 1, 2, 3]);
        // ```
        assertEquals(listOf(0, 1, 2, 3), indicesOf(pages[0]))
    }

    /** Verifies accented Latin initials share the page of their base letter (D28), so É sits with E. */
    @Test
    fun accentedRootNamesShareTheirBaseLetterPage() {
        val pages = paginate(listOf("élan.flac", "Eno.flac", "Émilie.flac", "zed.flac"))
        assertEquals(listOf("E", "Z"), pages.map { page -> page.label })
        assertEquals(listOf(0, 1, 2), indicesOf(pages[0]))
    }

    /** Verifies the label a loose track gets in the picker is the one `looseFolderLabel` reports. */
    @Test
    fun looseFolderLabelNamesThePageOfALooseTrack() {
        assertEquals("A", looseFolderLabel("Adele-Meow.mp3"))
        assertEquals("A", looseFolderLabel("aaa lowercase loose.mp3"))
        assertEquals("E", looseFolderLabel("Émilie.flac"))
        assertEquals("#", looseFolderLabel("01 - Numbered Loose.m4a"))
        assertEquals("#", looseFolderLabel("初音.flac"))
        assertEquals("#", looseFolderLabel(""))
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `foldersPrecedeLettersPrecedeCatchAll` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folders precede letters precede catch-all", () => {
    // ```
    @Test
    // What:     `fun foldersPrecedeLettersPrecedeCatchAll() { ... }` declares a no-arg
    //           `Unit`-returning test method, block body.
    // Why:      Ported from the Rust `folders_precede_letters_precede_catch_all`: a folder, a
    //           letter, and a catch-all track are ordered so the folder's label (`Zed`) sorts
    //           AFTER the letter's (`A`); the result proves the SORT GROUP (folder, then letter,
    //           then `#`), not the label text, orders the three axes. Pins the cross-axis order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...arrange + assertion below... *\/ }
    // ```
    fun foldersPrecedeLettersPrecedeCatchAll() {
        // What:     `val pages = paginate(listOf("Zed/x.flac", "apple.flac", "1.flac"))` declares a
        //           read-only `List<Page>` local `pages`: one subfolder track (`Zed`), one
        //           root-letter track (`apple` -> `A`), one non-letter root (`1` -> `#`).
        // Why:      Capture the pages for the assertion below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pages = paginate(["Zed/x.flac", "apple.flac", "1.flac"]);
        // ```
        val pages = paginate(listOf("Zed/x.flac", "apple.flac", "1.flac"))
        // What:     `assertEquals(listOf("Zed", "A", "#"), labelsOf(pages))` is
        //           `assertEquals(expected, actual)`. EXPECTED is an immutable `List<String>`;
        //           ACTUAL is `labelsOf(pages)`.
        // Why:      Even though `Zed` sorts after `A` alphabetically, the FOLDER page comes
        //           first, then the LETTER page, then the `#` catch-all, proving the axis-group
        //           order dominates the label text.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(labelsOf(pages)).toEqual(["Zed", "A", "#"]);
        // ```
        assertEquals(listOf("Zed", "A", "#"), labelsOf(pages))
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `pageOfIndexFindsAndMisses` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("page of index finds and misses", () => {
    // ```
    @Test
    // What:     `fun pageOfIndexFindsAndMisses() { ... }` declares a no-arg `Unit`-returning test
    //           method, block body.
    // Why:      Ported from the Rust `page_of_index_finds_and_misses`: load index 2 (`c.flac`)
    //           lives on the THIRD page (the `C` letter page after two folder pages), and an
    //           OUT-OF-RANGE index belongs to NO page. Pins `pageOfIndex`'s hit (returns the
    //           page position) and miss (returns `null`) behaviour.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...arrange + assertions below... *\/ }
    // ```
    fun pageOfIndexFindsAndMisses() {
        // What:     `val pages = paginate(listOf("A/x.flac", "B/y.flac", "c.flac"))` declares a
        //           read-only `List<Page>` local `pages`: two folder pages (`A`, `B`) and one
        //           root-letter page (`C`).
        // Why:      Capture the pages so `pageOfIndex` can be probed for a hit and a miss.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pages = paginate(["A/x.flac", "B/y.flac", "c.flac"]);
        // ```
        val pages = paginate(listOf("A/x.flac", "B/y.flac", "c.flac"))
        // What:     `assertEquals(2, pageOfIndex(pages, 2))` is `assertEquals(expected, actual)`.
        //           - EXPECTED is the `Int` literal `2` (a page POSITION).
        //           - ACTUAL `pageOfIndex(pages, 2)` returns `Int?` (a page position OR `null`):
        //             here it finds load index 2 (`c.flac`) on the third page (position 2).
        // Why:      Confirm the hit case returns the correct page position.
        // Gotcha:   `pageOfIndex` returns a NULLABLE `Int?`; here `assertEquals` compares the
        //           non-null `2`, and the next line checks the `null` branch separately.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pageOfIndex(pages, 2)).toEqual(2);
        // ```
        assertEquals(2, pageOfIndex(pages, 2))
        // What:     `assertNull(pageOfIndex(pages, 99))` calls `assertNull(value)`, which FAILS
        //           unless its argument is `null`. The argument `pageOfIndex(pages, 99)` looks up
        //           an out-of-range load index `99`, which belongs to no page and so returns the
        //           `null` variant of `Int?`.
        // Why:      Confirm the miss case returns `null` rather than throwing or returning a
        //           bogus page.
        // Gotcha:   `assertNull` is a DISTINCT assertion from `assertEquals(null, ...)`; it exists
        //           precisely to read the nullable result here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(pageOfIndex(pages, 99)).toBeNull();
        // ```
        assertNull(pageOfIndex(pages, 99))
    }

    // What:     `@Test` annotation marking the next method as a JUnit test (metadata only).
    // Why:      Registers `rowDisplayTrimsOnlyFolderTabPrefix` with the runner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("row display trims only folder tab prefix", () => {
    // ```
    @Test
    // What:     `fun rowDisplayTrimsOnlyFolderTabPrefix() { ... }` declares a no-parameter test
    //           method returning `Unit`, block body.
    // Why:      Pins that folder tabs drop the `<label>/` prefix while letter / `#` tabs keep
    //           the whole name. Mirrors the desktop's `row_display_trims_only_folder_tab_prefix`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // () => { /* ...assertions below... *\/ }
    // ```
    fun rowDisplayTrimsOnlyFolderTabPrefix() {
        // What:     `assertEquals("B/C.opus", rowDisplay("Ado", "Ado/B/C.opus"))` is
        //           `assertEquals(expected, actual)`: EXPECTED `"B/C.opus"`; ACTUAL the helper
        //           result. A folder page (label `Ado`) strips the `Ado/` prefix.
        // Why:      The `Ado` tab already names the folder; the row shows the path below it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(rowDisplay("Ado", "Ado/B/C.opus")).toEqual("B/C.opus");
        // ```
        assertEquals("B/C.opus", rowDisplay("Ado", "Ado/B/C.opus"))
        // What:     `assertEquals("Apple.flac", rowDisplay("A", "Apple.flac"))`. A LETTER page
        //           (label `A`) whose root file merely starts with `A` but has no `/`: returned
        //           UNCHANGED.
        // Why:      Loose files grouped by first letter have no folder to trim; the bare `A`
        //           must not be chopped off the filename.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(rowDisplay("A", "Apple.flac")).toEqual("Apple.flac");
        // ```
        assertEquals("Apple.flac", rowDisplay("A", "Apple.flac"))
        // What:     `assertEquals("#tag.flac", rowDisplay("#", "#tag.flac"))`. The `#` catch-all
        //           page: a root file starting with `#` is returned unchanged.
        // Why:      The catch-all is a letter-style tab; its loose files keep their names.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(rowDisplay("#", "#tag.flac")).toEqual("#tag.flac");
        // ```
        assertEquals("#tag.flac", rowDisplay("#", "#tag.flac"))
        // What:     `assertEquals("song.flac", rowDisplay("A", "A/song.flac"))`. A FOLDER named
        //           `A` (its names are `A/...`): this IS a folder tab, so the `A/` prefix is
        //           stripped.
        // Why:      The distinction is the `/` after the label, not the label's length: a
        //           one-letter FOLDER still trims, unlike a one-letter LETTER bucket.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // expect(rowDisplay("A", "A/song.flac")).toEqual("song.flac");
        // ```
        assertEquals("song.flac", rowDisplay("A", "A/song.flac"))
    }
}
