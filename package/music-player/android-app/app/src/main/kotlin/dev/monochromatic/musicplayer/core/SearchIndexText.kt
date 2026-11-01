// Text preparation for the Search index. It writes the trimmed normalized form of a title into a reusable buffer
// without allocating per character, and it reads the word-start keys of that form. D112 normalizes a million names
// once, off the main thread, so this path avoids the per-character objects that `normalizeForSearch` creates.

// What:     `package dev.monochromatic.musicplayer.core` places the index text helpers beside the matcher.
// Why:      They must apply exactly the matcher's normalization rules, so they share its package-private pieces.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

/** First code point that is not ASCII; every code point below it is its own NFKD form and has no mark. */
private const val ASCII_LIMIT: Int = 0x80

/** Distance between an ASCII capital letter and its lowercase letter. */
private const val ASCII_CASE_OFFSET: Int = 'a'.code - 'A'.code

/** Number of code points in the Basic Multilingual Plane, which is the size of the piece cache. */
private const val BMP_SIZE: Int = 0x10000

/** Starting capacity of a text buffer, large enough for almost every file name. */
private const val INITIAL_TEXT_CAPACITY: Int = 256

// What:     `private val pieceCache: Array<String?>` remembers the normalized form of each BMP code point.
// Why:      Normalizing one code point costs a `Normalizer` call and two allocations, and a library repeats the same
//           few thousand non-ASCII code points a million times. The result depends on the code point alone.
//
// In TS you'd write (pseudocode):
// ```ts
// const pieceCache: (string | undefined)[] = new Array(0x10000);
// ```
/**
 * Normalized form of each BMP code point, filled on first use.
 *
 * Writers race benignly: every writer stores an equal immutable string, so a reader sees null or the right text.
 */
private val pieceCache: Array<String?> = arrayOfNulls(BMP_SIZE)

// What:     `internal class TextBuffer` is a growable run of UTF-16 units that one preparation pass reuses.
// Why:      Building a title's normalized form character by character into a fresh `StringBuilder` per name would
//           allocate a million builders; one buffer per pass allocates one.
//
// In TS you'd write (pseudocode):
// ```ts
// class TextBuffer { chars: Uint16Array; size = 0; append(c: number) { /* grow, then write */ } }
// ```
/** A reusable, growable array of UTF-16 units. */
internal class TextBuffer {
    /** Backing storage; only the first [size] units are meaningful. */
    var chars: CharArray = CharArray(INITIAL_TEXT_CAPACITY)
        private set

    /** Number of meaningful units. */
    var size: Int = 0
        private set

    /** Empties the buffer without releasing its storage. */
    fun clear() {
        size = 0
    }

    /** Drops every unit past [newSize], which must not exceed the current size. */
    fun shrinkTo(newSize: Int) {
        size = newSize
    }

    /** Appends one unit, doubling the storage when it is full. */
    fun append(unit: Char) {
        if (size == chars.size) {
            chars = chars.copyOf(size * 2)
        }
        chars[size] = unit
        size += 1
    }
}

// What:     `private fun pieceOf(codePoint: Int): String` returns the normalized form of a non-ASCII code point.
// Why:      The matcher normalizes every code point on its own, so a cached answer is exact. Supplementary code
//           points are rare and go straight to the matcher.
//
// In TS you'd write (pseudocode):
// ```ts
// const pieceOf = (cp: number) => pieceCache[cp] ??= normalizedPieceOf(cp);
// ```
/** Returns the cached normalized form of a non-ASCII code point. */
private fun pieceOf(codePoint: Int): String {
    if (codePoint >= BMP_SIZE) {
        return normalizedPieceOf(codePoint)
    }
    /** Cached text for this code point, or null before its first use. */
    val cached: String? = pieceCache[codePoint]
    if (cached != null) {
        return cached
    }
    /** Freshly computed normalized form. */
    val computed: String = normalizedPieceOf(codePoint)
    pieceCache[codePoint] = computed
    return computed
}

// What:     `internal fun normalizeTitleInto(buffer: TextBuffer, text: String, from: Int, to: Int)` writes the search
//           form of a title that sits inside a longer text.
// Why:      The search form is `normalizeForSearch(title).trim()`. ASCII code points are their own decomposition and
//           carry no mark, so they only need lowercasing; every other code point takes the matcher's own rule.
//
// In TS you'd write (pseudocode):
// ```ts
// const searchForm = (title: string) => normalizeForSearch(title).trim();
// ```
/**
 * Replaces the contents of [buffer] with the search form of `text[from, to)`: NFKD, non-spacing marks removed, one
 * code point lowercased at a time, then trimmed of surrounding whitespace the way `String.trim` does.
 */
internal fun normalizeTitleInto(buffer: TextBuffer, text: String, from: Int = 0, to: Int = text.length) {
    buffer.clear()
    /** Offset of the next unit to read. */
    var index: Int = from
    while (index < to) {
        /** Unit at the current offset, which decides whether the cheap ASCII path applies. */
        val unit: Char = text[index]
        if (unit.code < ASCII_LIMIT) {
            buffer.append(if (unit in 'A'..'Z') (unit.code + ASCII_CASE_OFFSET).toChar() else unit)
            index += 1
        } else {
            /** Code point that starts at this offset, which is one pair of units when it is supplementary. */
            val codePoint: Int = text.codePointAt(index)
            for (piece in pieceOf(codePoint)) {
                buffer.append(piece)
            }
            index += Character.charCount(codePoint)
        }
    }
    trimInPlace(buffer)
}

// What:     `private fun trimInPlace(buffer: TextBuffer)` removes leading and trailing whitespace from a buffer.
// Why:      `matchTier` trims the name, so the stored form is already trimmed and the tier test needs no copy.
//
// In TS you'd write (pseudocode):
// ```ts
// const trimInPlace = (b: number[]) => { /* drop leading and trailing whitespace */ };
// ```
/** Trims the buffer with the same whitespace test as `String.trim`, which is `Char.isWhitespace`. */
private fun trimInPlace(buffer: TextBuffer) {
    /** Offset of the first unit that is not whitespace, or the size when the buffer is blank. */
    var first = 0
    while (first < buffer.size && buffer.chars[first].isWhitespace()) {
        first += 1
    }
    /** Offset just past the last unit that is not whitespace. */
    var last: Int = buffer.size
    while (last > first && buffer.chars[last - 1].isWhitespace()) {
        last -= 1
    }
    if (first > 0) {
        System.arraycopy(buffer.chars, first, buffer.chars, 0, last - first)
    }
    buffer.shrinkTo(last - first)
}
