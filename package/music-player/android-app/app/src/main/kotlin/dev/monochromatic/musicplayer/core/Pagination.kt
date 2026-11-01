// ===========================================================================
// FILE SUMMARY (folds in the old KDoc's domain content):
//
// Pure queue pagination on two axes, a faithful port of the desktop's
// `pagination.rs`. Each track's display string (its queue-relative path, see
// the `relativeDisplayPaths` helper / `DisplayPath.kt`) is grouped onto a
// `Page`:
//
//   - A track inside a subfolder (its relative path contains `/`) groups by
//     its top-level folder under the loaded root (one level only); the page
//     label is that single folder.
//   - A track sitting directly at the root (no `/`) groups by first letter,
//     with fixed buckets: the 26 English letters A-Z (case-insensitive), plus
//     a single `#` catch-all for digits, symbols, CJK, and non-English letters.
//
// Pages come out folder-pages-first (case-insensitively by path), then the A-Z
// letter pages, then the `#` catch-all. Folder labels are case-folded for the
// sort only (never for display or bucketing), so lowercase-led folders
// interleave with capitalized ones instead of trailing after the last
// uppercase folder. `Queue` consumes `paginate` and `pageOfIndex` to confine
// playback to a page, so the playback scope and the visible tab can never
// drift.
//
// There are NO imports in this file: `Page` and `PageEntry` live in the same
// package (`Page.kt`), and `Pair` / `Comparable` / `Character` are visible
// without an import (the first two are Kotlin builtins, `Character` is
// java.lang, auto-imported on the JVM).
// ===========================================================================

// What:     `package dev.monochromatic.musicplayer.core` declares the
//           NAMESPACE every declaration in this file belongs to. A "package"
//           is Kotlin's folder-mirroring module path: anything declared here
//           is reachable as `dev.monochromatic.musicplayer.core.paginate`,
//           and siblings in the SAME package (here `Page`, `PageEntry` from
//           `Page.kt`) are visible with no import line.
// Why:      We need it so `Page`/`PageEntry` resolve without imports and so
//           other packages can address these functions by their full path.
// Gotcha:   Unlike a TS module, the package line does not export or import
//           anything by itself. Visibility is controlled per-declaration by
//           the `private` / `internal` / (default) `public` keyword instead.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent. Picture every file under
// // ".../musicplayer/core/" sharing one namespace, where same-folder
// // declarations see each other for free.
// ```
package dev.monochromatic.musicplayer.core

// What:     `private const val PAGE_SEPARATOR: String = "/"`. `val` declares
//           an immutable binding (cannot be reassigned, like TS `const`).
//           `const` additionally means COMPILE-TIME constant (the value is
//           inlined; only primitives and `String` may be `const`). `private`
//           scopes it to THIS FILE only. `: String` is the explicit type
//           annotation. Sibling type the reader might expect: `Char` (a single
//           UTF-16 code unit, written with single quotes `'/'`); we use the
//           multi-char-capable `String` here.
// Why:      The display strings join path segments with `/`; we split on the
//           same character to find a track's parent folder. One named constant
//           keeps the join char and the split char from drifting apart.
// Gotcha:   Kotlin `String` is a GC'd reference type exactly like TS `string`.
//           There is NO owned-vs-borrowed distinction (that is a Rust concept);
//           do not read anything into "is it a copy".
//
// In TS you'd write (pseudocode):
// ```ts
// const PAGE_SEPARATOR = "/";
// ```
/**
 * Defines page separator value for this music-player component; the TypeScript-oriented notes above explain its
 * source and use.
 */
private const val PAGE_SEPARATOR: String = "/"

// What:     `private const val FOLDER_GROUP: Int = 0`. `Int` is Kotlin's
//           SIGNED 32-bit integer (range about ±2.1 billion). Siblings the
//           reader might expect: `Long` (signed 64-bit), `Short` (16-bit),
//           `Byte` (8-bit), `UInt`/`ULong` (unsigned). This is the sort-group
//           tag for folder pages.
// Why:      The page sort key pairs this tag with a label so folder pages sort
//           BEFORE letter pages regardless of how the label texts compare.
// Gotcha:   `Int` is NOT TS's `number`. It is a fixed-width 32-bit integer; it
//           does not auto-widen to a float or bigint, and overflow wraps
//           silently. We pick `Int` (not `Long`) because these are tiny tags
//           and `Int` is Kotlin's default integer, matching the `List` index
//           and size types used elsewhere in this file.
//
// In TS you'd write (pseudocode):
// ```ts
// const FOLDER_GROUP = 0;
// ```
/**
 * Defines folder group value for this music-player component; the TypeScript-oriented notes above explain its
 * source and use.
 */
private const val FOLDER_GROUP: Int = 0

// What:     `private const val LETTER_GROUP: Int = 1`. Same `Int` (signed
//           32-bit; siblings `Long`/`Short`/`Byte`) sort-group tag, for the
//           A-Z letter pages.
// Why:      Letter pages must sort AFTER folder pages but BEFORE the catch-all;
//           the tag value `1` sits between `0` and `2` to encode that order.
//
// In TS you'd write (pseudocode):
// ```ts
// const LETTER_GROUP = 1;
// ```
/**
 * Defines letter group value for this music-player component; the TypeScript-oriented notes above explain its
 * source and use.
 */
private const val LETTER_GROUP: Int = 1

// What:     `private const val CATCH_ALL_GROUP: Int = 2`. Sort-group tag (`Int`,
//           signed 32-bit; siblings `Long`/`Short`/`Byte`) for the `#` page.
// Why:      The catch-all must sort LAST, after every A-Z letter page; `2` is
//           the largest of the three group tags.
//
// In TS you'd write (pseudocode):
// ```ts
// const CATCH_ALL_GROUP = 2;
// ```
/**
 * Defines catch all group value for this music-player component; the TypeScript-oriented notes above explain its
 * source and use.
 */
private const val CATCH_ALL_GROUP: Int = 2

// What:     `const val CATCH_ALL_LABEL: String = "#"`. The single-char
//           caption of the catch-all page, held as a `String` (sibling: `Char`,
//           a single code unit `'#'`). `String` lets it interoperate directly
//           with the other `String` labels without conversion.
// Why:      One spot defines the catch-all caption, shared by the bucket key
//           and any test, so it cannot diverge.
//
// In TS you'd write (pseudocode):
// ```ts
// const CATCH_ALL_LABEL = "#";
// ```
/**
 * Defines catch all label value for this music-player component; the TypeScript-oriented notes above explain its
 * source and use.
 */
const val CATCH_ALL_LABEL: String = "#"

// What:     `private fun letterKey(name: String): Pair<Int, String>` declares
//           a FILE-PRIVATE function. `fun` is Kotlin's function keyword.
//           Parameter `name: String` is the display string. The return type
//           `Pair<Int, String>` is Kotlin's two-element generic pair class
//           (`Pair<A, B>`): a real class, NOT a language-level tuple, holding
//           a `.first` (the `Int` sort-group tag) and a `.second` (the
//           `String` bucket label). Siblings the reader might expect:
//           `Triple<A, B, C>` for three elements; a `data class` for named
//           fields. We use `Pair` because exactly two values travel together.
// Why:      Compute the (group, label) page key for a ROOT-LEVEL track (one
//           with no folder) using its first letter, with fixed A-Z buckets
//           plus a `#` catch-all, so a flat folder browses by first letter
//           without exploding into one page per distinct character.
//
// In TS you'd write (pseudocode):
// ```ts
// function letterKey(name: string): [number, string] { ... }
// ```
/**
 * Defines letter key behavior for this music-player component; the TypeScript-oriented notes above explain its
 * call shape and effects.
 */
private fun letterKey(name: String): Pair<Int, String> {
    // What:     `val first: Char? = name.firstOrNull()`. `val` is an immutable
    //           binding. The type `Char?` is a NULLABLE `Char`: the trailing
    //           `?` means "either a `Char` or `null`". `Char` is a single
    //           UTF-16 code unit; siblings the reader might expect: `String`
    //           (zero-or-more chars), `Int` (the code unit as a number).
    //           `name.firstOrNull()` returns the first character of the string,
    //           or `null` when the string is empty (the `...OrNull` suffix is
    //           Kotlin's "return null instead of throwing on absence" convention).
    // Why:      The first character decides the letter bucket; an empty name
    //           must not crash, so we need the nullable-on-empty variant.
    // Gotcha:   `Char?` is plain nullability (the `?` on the TYPE), the closest
    //           analogue to an `Option<char>` but it is NOT a wrapper object you
    //           unwrap; you compare it against `null` directly (see next line).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const first: string | undefined = name[0]; // undefined when name is ""
    // ```
    /**
     * Defines first value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    val first: Char? = name.firstOrNull()
    /** Plain base letter of the first character, so an accented initial shares its base letter's page (D28). */
    val letter: Char? = first?.let { initial -> foldedLatinInitial(initial) }
    // What:     `return if (letter != null) { ... } else { ... }` (formerly a test for the 26 English letters).
    //           In Kotlin `if/else` is an EXPRESSION whose value is returned.
    //           `first != null` is a null check. `'a'..'z'` builds a `CharRange`
    //           (an inclusive range of characters from `a` to `z`); the `in`
    //           operator tests membership in that range. `'a'`/`'z'`/`'A'`/`'Z'`
    //           are `Char` literals (single quotes). The `&&`/`||` are the usual
    //           short-circuit boolean operators.
    // Why:      A first character that is one of the 26 English letters (either
    //           case) goes to its letter bucket; anything else falls to the
    //           catch-all. This is the branch decision that splits the two arms.
    // Gotcha:   Inside this `if` arm Kotlin SMART-CASTS `first` from `Char?` to
    //           non-null `Char`, so the body may use `first.uppercaseChar()`
    //           with no `!!`/`?.` — the compiler already proved it is non-null.
    //           TS does the same narrowing after a `!== undefined` guard.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (first !== undefined && /[a-zA-Z]/.test(first)) {
    //   // ...letter-bucket arm (first is narrowed to string here)...
    // } else {
    //   // ...catch-all arm...
    // }
    // ```
    return if (letter != null) {
        // What:     `Pair(LETTER_GROUP, first.uppercaseChar().toString())`.
        //           `Pair(a, b)` is the CONSTRUCTOR of the `Pair` class (Kotlin
        //           constructors are called like plain functions, no `new`).
        //           `first.uppercaseChar()` returns the uppercase `Char` (the
        //           identity for `A`-`Z`); `.toString()` CONVERTS that single
        //           `Char` into a one-character `String` (the bucket label is a
        //           `String`, not a `Char`). This is the tail expression of the
        //           `if` arm, so its value becomes the `return`ed value.
        // Why:      Case-fold so `a` and `A` share the `A` page, and tag it
        //           `LETTER_GROUP` so it sorts among the letter pages.
        // Gotcha:   `uppercaseChar()` is ASCII-simple here (it acts on one
        //           `Char`); the broader-Unicode uppercasing lives in `sortKey`
        //           below. `.toString()` on a `Char` is a TYPE CONVERSION (Char
        //           -> length-1 String), not a "describe the object" call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [LETTER_GROUP, first.toUpperCase()];
        // ```
        Pair(LETTER_GROUP, letter.uppercaseChar().toString())
    } else {
        // What:     `Pair(CATCH_ALL_GROUP, CATCH_ALL_LABEL)`. Constructs the
        //           `Pair` for the catch-all bucket: the `CATCH_ALL_GROUP` tag
        //           and the shared `"#"` label constant. Tail expression of the
        //           `else` arm, so it is the `return`ed value when the first
        //           char is a digit, symbol, CJK, accented/non-English letter,
        //           or the name was empty (`first` was `null`).
        // Why:      Everything that is not a plain English letter lands on `#`,
        //           so a flat folder does not sprout one page per odd character.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [CATCH_ALL_GROUP, CATCH_ALL_LABEL];
        // ```
        Pair(CATCH_ALL_GROUP, CATCH_ALL_LABEL)
    }
}

// What:     `fun looseFolderLabel(fileName: String): String` names the virtual folder a loose root-level track sits in.
// Why:      D28: a loose track lives on the page of its initial letter (A to Z, accents folded; `#` for anything
//           else), which is the same page the picker shows. Search names that same folder instead of its own group.
//
// In TS you'd write (pseudocode):
// ```ts
// function looseFolderLabel(fileName: string): string { return letterKey(fileName)[1]; }
// ```
/** Returns the letter label of the page a loose track named [fileName] sits on: A to Z, or `#`. */
fun looseFolderLabel(fileName: String): String = letterKey(fileName).second

// What:     `private fun pageKey(name: String): Pair<Int, String>` declares a
//           file-private function returning the `(group, label)` `Pair`
//           (`Pair<Int, String>`: a two-element pair class, `.first` the `Int`
//           sort-group tag, `.second` the `String` label; sibling `Triple`).
//           Parameter `name: String` is the track's display string.
// Why:      ONE spot decides a track's page (folder vs first-letter), so the
//           bucket key and the displayed label can never drift apart.
//
// In TS you'd write (pseudocode):
// ```ts
// function pageKey(name: string): [number, string] { ... }
// ```
/**
 * Defines page key behavior for this music-player component; the TypeScript-oriented notes above explain its
 * call shape and effects.
 */
private fun pageKey(name: String): Pair<Int, String> {
    // What:     `val slash: Int = name.indexOf(PAGE_SEPARATOR)`. Immutable
    //           binding of type `Int` (signed 32-bit; siblings `Long`/`Short`).
    //           `name.indexOf("/")` returns the position of the FIRST `/`, or
    //           the SENTINEL `-1` when the string contains no `/`.
    // Why:      A `/` means the track lives in a subfolder; the segment before
    //           the first `/` is its top-level folder. `-1` means root-level.
    // Gotcha:   This is the `-1` SENTINEL convention, NOT a null/Option. The
    //           next line tests `slash >= 0`, not `slash != null`. (Rust's twin
    //           used `Option<usize>`/`None` here; Kotlin uses the C-style `-1`.)
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const slash = name.indexOf("/"); // -1 when absent
    // ```
    /**
     * Defines slash value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    val slash: Int = name.indexOf(PAGE_SEPARATOR)
    // What:     `return if (slash >= 0) { ... } else { ... }`. An `if/else`
    //           EXPRESSION (its value is returned). `slash >= 0` tests the
    //           `-1` sentinel: a real index means "found a `/`".
    // Why:      Branch on whether the track has a folder: with a `/` we group
    //           by the top folder, without one we fall back to letter bucketing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (slash >= 0) {
    //   // ...folder arm...
    // } else {
    //   // ...letter-bucket arm...
    // }
    // ```
    return if (slash >= 0) {
        // What:     `Pair(FOLDER_GROUP, name.substring(0, slash))`. Constructs
        //           the folder `Pair`. `name.substring(0, slash)` returns the
        //           substring from index 0 UP TO (not including) `slash` — i.e.
        //           the text before the first `/`, which is the top-level
        //           folder. Tail expression of the arm -> the `return`ed value.
        // Why:      Group by ONE folder level only (the top folder under the
        //           loaded root); deeper nesting shows in the row path, not the
        //           tab label.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [FOLDER_GROUP, name.slice(0, slash)];
        // ```
        Pair(FOLDER_GROUP, name.substring(0, slash))
    } else {
        // What:     `letterKey(name)`. Calls the helper above to compute the
        //           first-letter bucket. Tail expression of the `else` arm, so
        //           its returned `Pair` becomes this function's return value.
        // Why:      Root-level tracks (no `/`) paginate by their first letter.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return letterKey(name);
        // ```
        letterKey(name)
    }
}

// What:     `private fun sortKey(label: String): String = label.uppercase()`.
//           A file-private function written in EXPRESSION-BODY form: the
//           `= <expr>` after the signature means the function simply RETURNS
//           that expression (no `{ }` block, no `return` keyword). Parameter
//           `label: String`, return type `String`. `label.uppercase()`
//           Unicode-aware-uppercases the whole string and returns a fresh
//           `String` (it folds accented and non-English letters too, not just
//           ASCII).
// Why:      The case-folded form of a page label, used ONLY to ORDER pages,
//           never to display or bucket them. Folding case first gives the human
//           "ignore case" order the tab bar wants instead of raw code-point
//           order (which would put every uppercase letter before every
//           lowercase one). A-Z letter pages and `#` are already uppercase, so
//           this is the identity for them.
//
// In TS you'd write (pseudocode):
// ```ts
// const sortKey = (label: string): string => label.toUpperCase();
// ```
/**
 * Defines sort key behavior for this music-player component; the TypeScript-oriented notes above explain its
 * call shape and effects.
 */
private fun sortKey(label: String): String = label.uppercase()

// What:     `internal fun compareByCodePoint(left: String, right: String): Int`.
//           `internal` is a VISIBILITY keyword meaning "visible everywhere in
//           THIS module/compilation unit, but not to other modules". Siblings
//           the reader might expect: `private` (this file only), `public` (the
//           default, visible everywhere), `protected`. Two `String` params and
//           an `Int` result (signed 32-bit; siblings `Long`/`Short`).
// Why:      Compare two strings by Unicode CODE POINT, matching the desktop
//           Rust's byte-lexicographic ordering (which equals code-point order),
//           instead of Kotlin's default UTF-16 code-UNIT `compareTo`. The two
//           diverge only for supplementary characters (those above U+FFFF, made
//           of surrogate pairs), so this keeps the page sort faithful for any
//           folder name, not just the ASCII test vectors. `internal` (not
//           `private`) so the test module in the same compilation unit can call
//           it directly.
//
// In TS you'd write (pseudocode):
// ```ts
// function compareByCodePoint(left: string, right: string): number { ... }
// ```
/**
 * Defines compare by code point behavior for this music-player component; the TypeScript-oriented notes above
 * explain its call shape and effects.
 */
internal fun compareByCodePoint(left: String, right: String): Int {
    // What:     `var leftOffset = 0`. `var` declares a MUTABLE binding (can be
    //           reassigned), the opposite of `val`. No explicit type, so Kotlin
    //           INFERS `Int` from the literal `0`. A cursor into `left`.
    // Why:      We walk both strings code point by code point; this counter
    //           tracks how far into `left` we are. It must be `var` because the
    //           loop advances it.
    // Gotcha:   This and `rightOffset` are the ONLY mutable bindings in the file;
    //           everything else is `val`. Mutability here is deliberate cursor
    //           state for the manual surrogate-aware walk.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let leftOffset = 0;
    // ```
    /**
     * Defines left offset value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    var leftOffset = 0
    // What:     `var rightOffset = 0`. A second MUTABLE `Int` cursor (type
    //           inferred from `0`), this one into `right`.
    // Why:      We advance the two strings INDEPENDENTLY because a code point
    //           may occupy one or two UTF-16 units, so the offsets do not move
    //           in lockstep.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let rightOffset = 0;
    // ```
    /**
     * Defines right offset value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    var rightOffset = 0
    // What:     `while (leftOffset < left.length && rightOffset < right.length)`.
    //           A `while` loop (the side-effecting cursor form). `.length` is
    //           the string's UTF-16 code-UNIT count. The loop runs while BOTH
    //           cursors are still inside their strings.
    // Why:      Compare the strings position by position until one runs out or
    //           a differing code point is found.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (leftOffset < left.length && rightOffset < right.length) { ... }
    // ```
    while (leftOffset < left.length && rightOffset < right.length) {
        // What:     `val leftCodePoint: Int = left.codePointAt(leftOffset)`.
        //           Immutable `Int` (signed 32-bit; siblings `Long`/`Char`).
        //           `codePointAt(i)` reads the full Unicode CODE POINT starting
        //           at UTF-16 index `i`, combining a surrogate pair into one
        //           value when present (so it can exceed U+FFFF). This is
        //           `java.lang.String.codePointAt`, callable directly via JVM
        //           interop.
        // Why:      We must compare by code point, not by code unit, to match
        //           the desktop ordering for supplementary characters.
        // Gotcha:   `codePointAt` returns a CODE POINT, not the `.charCodeAt`
        //           code UNIT. For surrogate pairs the two differ; that gap is
        //           exactly why this manual walk exists.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const leftCodePoint = left.codePointAt(leftOffset)!;
        // ```
        /**
         * Defines left code point value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        val leftCodePoint: Int = left.codePointAt(leftOffset)
        // What:     `val rightCodePoint: Int = right.codePointAt(rightOffset)`.
        //           The same code-point read (immutable `Int`; sibling `Char`)
        //           for the `right` string at its own cursor.
        // Why:      We need the matching code point on the right side to compare
        //           against the left one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rightCodePoint = right.codePointAt(rightOffset)!;
        // ```
        /**
         * Defines right code point value for this music-player component; the TypeScript-oriented notes above
         * explain its source and use.
         */
        val rightCodePoint: Int = right.codePointAt(rightOffset)
        // What:     `if (leftCodePoint != rightCodePoint) { ... }`. A plain
        //           equality branch on two `Int`s.
        // Why:      The first position where the code points differ decides the
        //           whole comparison; we can stop and return there.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (leftCodePoint !== rightCodePoint) { ... }
        // ```
        if (leftCodePoint != rightCodePoint) {
            // What:     `return leftCodePoint.compareTo(rightCodePoint)`.
            //           `Int.compareTo(other)` returns a negative / zero /
            //           positive `Int` when the receiver is less than / equal
            //           to / greater than `other`. Early `return` out of the
            //           function with that three-way result.
            // Why:      At the first differing code point, the smaller code
            //           point sorts first; `compareTo` yields exactly the
            //           negative/zero/positive contract a comparator owes.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return leftCodePoint - rightCodePoint;
            // ```
            return leftCodePoint.compareTo(rightCodePoint)
        }
        // What:     `leftOffset += Character.charCount(leftCodePoint)`.
        //           `Character.charCount(cp)` (java.lang.Character, JVM interop)
        //           returns how many UTF-16 code UNITS that code point occupies:
        //           `1` for the basic plane, `2` for a surrogate-pair character.
        //           `+=` advances the cursor by that many units.
        // Why:      We must skip the WHOLE character we just read, which may be
        //           one or two UTF-16 units, so the next read lands on the next
        //           real code point rather than on the trailing surrogate half.
        // Gotcha:   Advancing by a fixed `1` here (the naive approach) would
        //           split surrogate pairs and corrupt the comparison for emoji /
        //           CJK extension characters. `charCount` is what makes the walk
        //           surrogate-correct.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // leftOffset += leftCodePoint > 0xffff ? 2 : 1;
        // ```
        leftOffset += Character.charCount(leftCodePoint)
        // What:     `rightOffset += Character.charCount(rightCodePoint)`. The
        //           same surrogate-aware cursor advance for the `right` string.
        // Why:      Keep the right cursor on real code-point boundaries too, for
        //           the same reason as the left one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // rightOffset += rightCodePoint > 0xffff ? 2 : 1;
        // ```
        rightOffset += Character.charCount(rightCodePoint)
    }
    // What:     `return (left.length - leftOffset).compareTo(right.length - rightOffset)`.
    //           Reached only when the loop ended because at least one string ran
    //           out with no differing code point found so far. Each
    //           `<str>.length - <offset>` is the count of UTF-16 units NOT yet
    //           consumed (zero when fully consumed). `.compareTo` three-way
    //           compares those two remainders.
    // Why:      When one string is a prefix of the other, the SHORTER one sorts
    //           first; comparing leftover lengths encodes "shorter prefix wins,
    //           equal length means equal strings".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return (left.length - leftOffset) - (right.length - rightOffset);
    // ```
    return (left.length - leftOffset).compareTo(right.length - rightOffset)
}

// What:     `private data class PageSortKey(`. `data class` is a Kotlin class
//           that AUTO-GENERATES `equals`/`hashCode` (structural, field by
//           field), `toString`, `copy`, and `componentN` accessors (which power
//           destructuring like `val (a, b) = key`). `private` scopes the type
//           to this file. The constructor params declared with `val` ON the
//           constructor (below) double as the public read-only properties.
// Why:      A composite map key that ORDERS pages: sort-group first, then the
//           case-folded label, then the original label as a tiebreaker, so two
//           folders that case-fold alike (`Reol` and `REOL`) stay separate
//           buckets, ordered deterministically. Mirrors the desktop Rust
//           `BTreeMap` key tuple `(u8, String, String)`. A `data class` gives
//           the structural equality/hashing a map key needs for free.
// Gotcha:   Unlike a plain class, a `data class` gives value-style `equals`/
//           `hashCode` over its constructor properties; that is what lets it act
//           as a map key by VALUE rather than by object identity.
//
// In TS you'd write (pseudocode):
// ```ts
// type PageSortKey = { group: number; fold: string; label: string };
// // (with structural equality / a derived string key for map use)
// ```
/**
 * Defines page sort key type for this music-player component; the TypeScript-oriented notes above explain its
 * role.
 */
private data class PageSortKey(
    // What:     `val group: Int,`. A constructor-declared, immutable property of
    //           type `Int` (signed 32-bit; siblings `Long`/`Short`). The
    //           sort-group tag (`FOLDER_GROUP`, `LETTER_GROUP`, or
    //           `CATCH_ALL_GROUP`). The trailing comma separates it from the
    //           next property.
    // Why:      Primary ordering axis: folder pages (0) before letter pages (1)
    //           before the catch-all (2), independent of any label text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // group: number;
    // ```
    val group: Int,
    // What:     `val fold: String,`. Immutable `String` constructor property
    //           (sibling: `Char`). The case-folded label, the PRIMARY text
    //           ordering within a group.
    // Why:      Ordering by the folded form gives the "ignore case" tab order;
    //           keeping it as a separate field avoids re-folding on every
    //           comparison.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fold: string;
    // ```
    val fold: String,
    // What:     `val label: String,`. Immutable `String` constructor property
    //           (sibling: `Char`). The ORIGINAL label, retained both for display
    //           and as the equal-fold tiebreaker.
    // Why:      Two folders that fold identically (`Reol`/`REOL`) must remain
    //           distinct buckets; the raw label breaks the tie deterministically
    //           and is also what the tab actually shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // label: string;
    // ```
    val label: String,
// What:     `) : Comparable<PageSortKey> {`. The `)` closes the constructor
//           parameter list; `: Comparable<PageSortKey>` declares that this
//           class IMPLEMENTS the `Comparable` interface specialized to itself
//           (`<PageSortKey>` is the type argument). Implementing `Comparable`
//           means the class promises a `compareTo` method, which lets standard
//           sorts (`sortedBy`, sorted maps) order instances of it.
// Why:      `paginate` sorts the page keys; making the key `Comparable<Self>`
//           lets `sortedBy { it.key }` order pages with no separate comparator
//           argument.
//
// In TS you'd write (pseudocode):
// ```ts
// // No Comparable interface; provide a compare method used as
// // arr.sort((a, b) => a.compareTo(b)).
// ```
) : Comparable<PageSortKey> {
    // What:     `override fun compareTo(other: PageSortKey): Int`. `override` is
    //           MANDATORY in Kotlin when supplying a method the interface (here
    //           `Comparable`) declares; omitting it is a compile error. Param
    //           `other: PageSortKey`, result `Int` (signed 32-bit; the
    //           three-way negative/zero/positive comparator contract).
    // Why:      Provide the lexicographic order the Rust tuple sort used: group,
    //           then case-folded label, then original label, with code-point
    //           string ordering to mirror Rust's `String: Ord`.
    // Gotcha:   `override` is a real keyword carrying meaning (the method must
    //           match an inherited/interface signature), not a doc annotation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // compareTo(other: PageSortKey): number { ... }
    // ```
    /**
     * Defines compare to behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    override fun compareTo(other: PageSortKey): Int {
        // What:     `val byGroup: Int = group.compareTo(other.group)`. Immutable
        //           `Int`. `group.compareTo(other.group)` three-way compares the
        //           two `Int` group tags (negative/zero/positive). `other` is the
        //           other key; `group` (no receiver) is THIS key's property.
        // Why:      The group is the primary ordering axis; compute it first.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const byGroup = this.group - other.group;
        // ```
        /**
         * Defines by group value for this music-player component; the TypeScript-oriented notes above explain
         * its source and use.
         */
        val byGroup: Int = group.compareTo(other.group)
        // What:     `if (byGroup != 0) return byGroup`. A single-line guard:
        //           when the groups differ, that result already decides the
        //           order, so return it immediately. (Early return / throw-and-
        //           return-early style.)
        // Why:      Short-circuit: no need to compare labels once the groups
        //           differ.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (byGroup !== 0) return byGroup;
        // ```
        if (byGroup != 0) {
            return byGroup
        }
        // What:     `val byFold: Int = compareByCodePoint(fold, other.fold)`.
        //           Immutable `Int`. Calls the code-point comparator on the two
        //           case-folded labels (this key's `fold` vs `other.fold`).
        // Why:      Same group: order by the case-folded label using the
        //           code-point comparison that mirrors Rust's `String` ordering.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const byFold = compareByCodePoint(this.fold, other.fold);
        // ```
        /**
         * Defines by fold value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val byFold: Int = compareByCodePoint(fold, other.fold)
        // What:     `if (byFold != 0) return byFold`. Early-return guard: when
        //           the folded labels differ, return that ordering.
        // Why:      Short-circuit before the final tiebreaker.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (byFold !== 0) return byFold;
        // ```
        return if (byFold != 0) {
            byFold
        } else {
            // What:     `compareByCodePoint(label, other.label)`. The final
        //           tiebreaker: code-point compare the ORIGINAL labels (this
        //           key's `label` vs `other.label`). This is the function's
        //           last expression / return.
        // Why:      Equal group and equal fold means two case-variant folders
        //           (`Reol`/`REOL`); the raw label breaks the tie so they stay
        //           distinct and deterministically ordered.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return compareByCodePoint(this.label, other.label);
        // ```
            compareByCodePoint(label, other.label)
        }
    }
}

// What:     `fun paginate(names: List<String>): List<Page>`. A PUBLIC function
//           (no visibility keyword means `public` in Kotlin). Param
//           `names: List<String>` is a READ-ONLY list of strings. Sibling the
//           reader might expect: `MutableList<String>` (a list you can add to)
//           and `Array<String>` (a fixed-size array); we take the read-only
//           `List` because we only iterate it. Returns `List<Page>` (read-only).
// Why:      Group the display strings into pages, sorted folder-pages-first
//           (case-insensitively), then A-Z letter pages, then `#`; entries
//           within each page stay in load order. `Queue` calls this whenever the
//           queue changes to rebuild the tabs and the visible page. An empty
//           input yields an empty list, not one empty page.
//
// In TS you'd write (pseudocode):
// ```ts
// function paginate(names: readonly string[]): Page[] {
//   const groups = new Map<string, PageEntry[]>(); // keyed by a serialized PageSortKey
//   names.forEach((name, index) => {
//     const [group, label] = pageKey(name);
//     const key = serialize({ group, fold: sortKey(label), label });
//     (groups.get(key) ?? groups.set(key, []).get(key)!).push({ index, name });
//   });
//   return [...groups.entries()]
//     .sort((a, b) => compareKey(a[0], b[0]))
//     .map(([key, entries]) => ({ label: deserialize(key).label, entries: [...entries] }));
// }
// ```
/**
 * Defines paginate behavior for this music-player component; the TypeScript-oriented notes above explain its
 * call shape and effects.
 */
fun paginate(names: List<String>): List<Page> {
    // What:     `val groups: MutableMap<PageSortKey, MutableList<PageEntry>> = mutableMapOf()`.
    //           Immutable BINDING (`val`) to a MUTABLE MAP. `MutableMap<K, V>`
    //           is a map you can insert into (sibling: read-only `Map<K, V>`).
    //           Its values are `MutableList<PageEntry>` (a list you can append
    //           to; sibling: read-only `List<PageEntry>`). `mutableMapOf()` is
    //           the factory that builds an empty `LinkedHashMap` (insertion-
    //           ordered). Keyed by the `PageSortKey` `data class`.
    // Why:      Accumulate entries per page key as we scan the names; we need a
    //           map we can grow (hence `MutableMap`) and per-bucket lists we can
    //           append to (hence `MutableList`).
    // Gotcha:   `val` makes only the BINDING immutable, NOT the map; we still
    //           insert into it below. This is `mutableMapOf` (a plain hash map),
    //           NOT a sorted map like Rust's `BTreeMap` — the ordering is added
    //           by an EXPLICIT `.sortedBy` step later, not "for free" by the map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const groups = new Map<string, PageEntry[]>(); // not sorted; sorted later
    // ```
    /**
     * Defines groups value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    val groups: MutableMap<PageSortKey, MutableList<PageEntry>> = mutableMapOf()
    // What:     `names.forEachIndexed { index, name -> ... }`. `forEachIndexed`
    //           iterates the list, passing BOTH the position and the element to
    //           the LAMBDA. The `{ index, name -> ... }` is a TRAILING-LAMBDA:
    //           when a lambda is the last argument, Kotlin lets you write it in
    //           braces AFTER the call instead of inside the parentheses. `index`
    //           and `name` are the two lambda parameters, the `->` separates the
    //           parameter list from the body.
    // Why:      We need both the load-order index (for `PageEntry.index`) and the
    //           name itself for every track.
    // Gotcha:   Argument order is FLIPPED versus TS: Kotlin gives `(index, name)`,
    //           TS gives `(name, index)`. Easy to transpose.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // names.forEach((name, index) => { ... });
    // ```
    names.forEachIndexed { index, name ->
        // What:     `val (group, label) = pageKey(name)`. DESTRUCTURING
        //           declaration: `pageKey(name)` returns a `Pair`, and
        //           `val (group, label) = ...` pulls its `.first`/`.second` into
        //           two `val`s via the Pair's auto-generated `component1()` /
        //           `component2()` operators.
        // Why:      Decide which page this name belongs to and unpack the tag and
        //           label in one line.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [group, label] = pageKey(name);
        // ```
        val (group, label) = pageKey(name)
        // What:     `val key = PageSortKey(group, sortKey(label), label)`.
        //           Constructs the composite key (calling the `PageSortKey`
        //           constructor like a function, no `new`). `sortKey(label)`
        //           computes the case-folded middle field; `group` and the raw
        //           `label` flank it. The variable type is INFERRED as
        //           `PageSortKey`.
        // Why:      A key whose natural order is (group, folded label, raw label)
        //           gives case-insensitive page order without losing the display
        //           label or merging case-variant folders.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const key = { group, fold: sortKey(label), label };
        // ```
        /**
         * Defines key value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val key = PageSortKey(group, sortKey(label), label)
        // What:     `groups.getOrPut(key) { mutableListOf() }.add(PageEntry(index = index, name = name))`.
        //           `getOrPut(key) { default }` returns the value for `key`,
        //           INSERTING the trailing-lambda's `mutableListOf()` (a fresh
        //           empty mutable list) on first sight of that key; the `{ }` is
        //           the default-producing lambda. `.add(...)` appends to the
        //           returned list. `PageEntry(index = index, name = name)`
        //           constructs the entry using NAMED ARGUMENTS (`index =` /
        //           `name =`), which document each position at the call site.
        // Why:      Bucket the entry under its page key, creating the bucket on
        //           demand; named args make it obvious which value is the index
        //           and which is the display name.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (groups.get(key) ?? groups.set(key, []).get(key)!).push({ index, name });
        // ```
        groups.getOrPut(key) { mutableListOf() }.add(PageEntry(index = index, name = name))
    }
    // What:     `return groups.entries`. `.entries` is the read-only SET of the
    //           map's `(key, value)` pairs. This begins a multi-line method
    //           chain (continued on the next lines) whose final value is
    //           returned.
    // Why:      We materialize the accumulated buckets into the ordered list of
    //           pages by iterating the map entries, sorting them, and mapping
    //           each to a `Page`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [...groups.entries()]
    // ```
    return groups.entries
        // What:     `.sortedBy { it.key }`. `sortedBy { selector }` returns a NEW
        //           sorted list, ordering by the `Comparable` value the
        //           trailing-lambda selects. `it` is the IMPLICIT single
        //           parameter name Kotlin gives a one-arg lambda (here each map
        //           entry); `it.key` is that entry's `PageSortKey`, which sorts
        //           via the `compareTo` we wrote above.
        // Why:      This is the EXPLICIT sort step (Kotlin's plain map did not
        //           sort for us): it produces the folder-then-letter-then-`#`,
        //           case-insensitive page order.
        // Gotcha:   `it` is the auto-named lambda parameter (no `->` needed for a
        //           single argument). It is NOT a keyword; it just defaults to
        //           "the one argument".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // .sort((a, b) => a[0].compareTo(b[0]))
        // ```
        .sortedBy { it.key }
        // What:     `.map { (key, entries) -> Page(label = key.label, entries = entries.toList()) }`.
        //           `.map { transform }` builds a new list by transforming each
        //           element. The lambda uses DESTRUCTURING of its single
        //           map-entry parameter: `(key, entries)` unpacks the entry's
        //           `component1()`/`component2()` (its key and value list). The
        //           body constructs a `Page` with NAMED ARGS. `entries.toList()`
        //           CONVERTS the `MutableList<PageEntry>` into a read-only
        //           `List<PageEntry>` snapshot. This is the chain's final
        //           expression, so its `List<Page>` is the function's return.
        // Why:      Turn each sorted bucket into a `Page` whose label is the raw
        //           display label and whose entries are a read-only copy (so the
        //           returned page cannot be mutated through the internal mutable
        //           list).
        // Gotcha:   `.toList()` here is a read-only-COPY conversion (mutable ->
        //           immutable view), the Kotlin analogue of `[...entries]`; it is
        //           not the Rust ownership move the desktop twin described.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // .map(([key, entries]) => ({ label: key.label, entries: [...entries] }));
        // ```
        .map { (key, entries) -> Page(label = key.label, entries = entries.toList()) }
}

// What:     `fun pageOfIndex(pages: List<Page>, index: Int): Int?`. A PUBLIC
//           function (no visibility keyword). Params: `pages: List<Page>`
//           (read-only list; siblings `MutableList`/`Array`) and `index: Int`
//           (signed 32-bit load-order position; siblings `Long`/`Short`). The
//           return type `Int?` is a NULLABLE `Int` (the trailing `?` means
//           "an `Int` or `null`").
// Why:      Find which page holds a given load-order track index, for
//           auto-following the now-playing track to its tab. The nullable
//           return reports "no page holds it" as `null`.
//
// In TS you'd write (pseudocode):
// ```ts
// function pageOfIndex(pages: readonly Page[], index: number): number | null { ... }
// ```
/**
 * Defines page of index behavior for this music-player component; the TypeScript-oriented notes above explain
 * its call shape and effects.
 */
fun pageOfIndex(pages: List<Page>, index: Int): Int? {
    // What:     `val position: Int = pages.indexOfFirst { page -> page.entries.any { it.index == index } }`.
    //           Immutable `Int` (signed 32-bit). `indexOfFirst { predicate }`
    //           returns the index of the FIRST element matching the
    //           trailing-lambda predicate, or the SENTINEL `-1` if none match.
    //           The outer lambda names its parameter `page` (via `page ->`).
    //           Inside, `page.entries.any { it.index == index }` is `true` when
    //           ANY entry on that page has the matching load-order index; that
    //           inner `any { }` lambda uses the implicit `it` for each entry.
    // Why:      Scan the pages once to find the first page containing the wanted
    //           index; `any` short-circuits on the first matching entry.
    // Gotcha:   `indexOfFirst` returns the `-1` SENTINEL on no match, NOT null;
    //           the next line converts that sentinel to `null`. The inner `it`
    //           (the entry) is a DIFFERENT implicit parameter from any outer one;
    //           the explicit `page ->` on the outer lambda avoids shadowing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const position = pages.findIndex(
    //   (page) => page.entries.some((e) => e.index === index),
    // ); // -1 when not found
    // ```
    /**
     * Defines position value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    val position: Int = pages.indexOfFirst { page -> page.entries.any { it.index == index } }
    // What:     `return if (position < 0) null else position`. An `if/else`
    //           EXPRESSION used as the return value. `position < 0` tests the
    //           `-1` sentinel; on no match it yields the literal `null`,
    //           otherwise the found `position`. Because the result type is
    //           `Int?`, returning `null` here is legal.
    // Why:      Convert the `-1`-means-absent sentinel into the nullable result
    //           the caller expects (`null` for "no page holds it").
    // Gotcha:   This is the sentinel-to-null bridge that the desktop Rust did NOT
    //           need (its `.position()` already returned `Option`/`None`); Kotlin
    //           gets a `-1` from `indexOfFirst` and maps it to `null` by hand.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return position < 0 ? null : position;
    // ```
    return if (position < 0) null else position
}

// What:     `fun pageIdentity(page: Page): String` prefixes the displayed label
//           with its folder or root-page kind.
// Why:      Folder page `A` and root-letter page `A` need distinct identities while
//           retaining the same displayed text.
//
// In TS you'd write (pseudocode):
// ```ts
// function pageIdentity(page: Page): string { /* kind plus label */ }
// ```
/** Returns stable page identity distinct from its potentially duplicated label. */
fun pageIdentity(page: Page): String {
    /** Folder prefix that distinguishes folder entries from root-level names. */
    val folderPrefix: String = "${page.label}/"
    /** Whether first entry proves this is a folder page. */
    val folderPage: Boolean = page.entries.firstOrNull()?.name?.startsWith(folderPrefix) == true
    return if (folderPage) "folder:${page.label}" else "root:${page.label}"
}

// What:     `fun rowDisplay(label: String, name: String): String { ... }` declares a PUBLIC
//           function (no visibility keyword means `public`). Params: `label` is a page's tab
//           caption and `name` is one of that page's track display strings; both are `String`
//           (sibling: `Char`, a single UTF-16 code unit). Returns the `String` a row should
//           SHOW.
// Why:      A FOLDER tab already names its top-level folder, so repeating it on every row
//           (`Ado/B/C.opus` under the `Ado` tab) is noise; show `B/C.opus` instead. A LETTER
//           or `#` tab groups loose root-level files with no folder segment, so their names
//           stay whole. A pure helper keeps this identical to the desktop's
//           `pagination::row_display` and unit-testable.
//
// In TS you'd write (pseudocode):
// ```ts
// function rowDisplay(label: string, name: string): string {
//   const prefix = label + "/";
//   return name.startsWith(prefix) ? name.slice(prefix.length) : name;
// }
// ```
/**
 * Defines row display behavior for this music-player component; the TypeScript-oriented notes above explain its
 * call shape and effects.
 */
fun rowDisplay(label: String, name: String): String {
    // What:     `val prefix: String = label + PAGE_SEPARATOR` declares a read-only (`val`)
    //           `String` `prefix`. `+` here is String CONCATENATION (not a numeric add);
    //           `PAGE_SEPARATOR` is the file-private `"/"` constant reused so the join char
    //           matches the split char elsewhere in this file.
    // Why:      A folder-page name is exactly `<label>/...`, so we test for that whole prefix
    //           (the trailing `/` is what stops a letter label `A` from matching `Apple.flac`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const prefix = label + "/";
    // ```
    /**
     * Defines prefix value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    val prefix: String = label + PAGE_SEPARATOR
    // What:     `return if (name.startsWith(prefix)) name.substring(prefix.length) else name`.
    //           An `if/else` EXPRESSION whose value is returned. `name.startsWith(prefix)` is a
    //           plain forward compare (not a regex); `name.substring(prefix.length)` returns
    //           the text AFTER the `<label>/` prefix; the `else` arm returns the whole `name`.
    // Why:      Strip the folder prefix on folder pages (`Ado/B/C.opus` -> `B/C.opus`); leave
    //           letter / `#` page names untouched, since a root file like `Apple.flac` never
    //           starts with `A/`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return name.startsWith(prefix) ? name.slice(prefix.length) : name;
    // ```
    return if (name.startsWith(prefix)) name.substring(prefix.length) else name
}
