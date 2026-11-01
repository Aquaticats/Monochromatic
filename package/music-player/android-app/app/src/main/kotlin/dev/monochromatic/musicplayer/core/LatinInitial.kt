// An accented first letter belongs with its base letter in the picker rail and in the page a loose track lands on
// (D17, D28): Émilie Simon sits under E, not in a cell of its own.

// What:     `package ...core` places the folding rule beside the rail and the pagination that share it.
// Why:      One rule keeps the rail cell of a folder and the page letter of a loose track in agreement.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import java.text.Normalizer

// What:     `private const val FIRST_ACCENTED_LATIN: Int = 0xC0` and the next constant bound the Latin-1 Supplement and
//           Latin Extended-A and B letters that can carry an accent.
// Why:      Below the range the letter is plain ASCII or punctuation, above it are other scripts; only this range is
//           worth a decomposition.
//
// In TS you'd write (pseudocode):
// ```ts
// const FIRST_ACCENTED_LATIN = 0xc0;
// const LAST_ACCENTED_LATIN = 0x24f;
// ```
/** First code point of the accented Latin letters. */
private const val FIRST_ACCENTED_LATIN: Int = 0xC0

/** Last code point of the accented Latin letters. */
private const val LAST_ACCENTED_LATIN: Int = 0x24F

// What:     `internal fun foldedLatinInitial(first: Char): Char?` returns the plain A to Z or a to z letter a first
//           character stands for, or null when it is not a Latin letter.
// Why:      Canonical decomposition splits É into E and an accent mark, so the base letter is the first piece. Letters
//           with no decomposition (Ø, Æ, ß) are distinct letters and return null.
//
// In TS you'd write (pseudocode):
// ```ts
// function foldedLatinInitial(first: string): string | null {
//   if (/[A-Za-z]/.test(first)) return first;
//   const base = first.normalize("NFD")[0];
//   return /[A-Za-z]/.test(base) && base !== first ? base : null;
// }
// ```
/** Maps a first character to its plain Latin base letter, keeping its case, or null when it has none. */
internal fun foldedLatinInitial(first: Char): Char? {
    if (first in 'a'..'z' || first in 'A'..'Z') {
        return first
    }
    if (first.code < FIRST_ACCENTED_LATIN || first.code > LAST_ACCENTED_LATIN) {
        return null
    }
    /** The first piece of the canonical decomposition, which is the base letter when the character is accented. */
    val base: Char = Normalizer.normalize(first.toString(), Normalizer.Form.NFD)[0]
    return if (base in 'a'..'z' || base in 'A'..'Z') base else null
}
