//region Renders as nothing
// WHETHER A TEXT SHOWS A READER ANYTHING, asked one way wherever a model's
// wording or reason must say something (ledger B40). The checks it replaces
// asked `trim()`, which answers a different question: it removes the
// ECMAScript whitespace and keeps every invisible character that is not
// whitespace. A translator's reply of one zero-width space (U+200B), word
// joiner (U+2060) or soft hyphen (U+00AD) passed the wire guard and the slate
// filter that way, and the intake fold (`invisible-variants.ts`) then removed
// its every character, so an empty candidate reached the judges. A reply of
// one Hangul filler (U+3164) is not folded at all, and the deterministic
// floor passes it over an original that says something.
//
// NOTHING, HERE, IS WHAT UNICODE SAYS RENDERS AS NOTHING: White_Space;
// Default_Ignorable_Code_Point, which Unicode derives from the format
// characters, the variation selectors and the fillers, less White_Space; and
// the controls (Cc), which no page shows as text. THE PROPERTIES ARE READ FROM
// the runtime's own Unicode tables (the `\p{...}` classes of `SHOWS_NOTHING` and `IS_DEFAULT_IGNORABLE`), so the set is
// the one of whichever Unicode version the runtime ships; the source applies
// no version of its own. Every character the intake fold removes is
// default-ignorable, and every one it turns into a space is White_Space, so a
// text this calls something still says something after the fold.

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one code point, anchored at both ends with one class and no quantifier, so the test is bounded and cannot backtrack; Unicode White_Space and Default_Ignorable_Code_Point have no string API */
/**
 Whether one code point shows nothing: Unicode White_Space, a
 default-ignorable code point, or a control.
 */
const SHOWS_NOTHING = /^[\p{White_Space}\p{Default_Ignorable_Code_Point}\p{Cc}]$/u;

/**
 Whether one code point is a default-ignorable one and nothing else: a format character, a variation selector, a filler, with
 no white space or control.
 */
const IS_DEFAULT_IGNORABLE = /^\p{Default_Ignorable_Code_Point}$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Whether a text shows a reader nothing at all: empty, or made only of
 whitespace, default-ignorable code points and controls.

 @param text - wording or reason as a model wrote it, before or after the
 intake fold, which cannot change this answer

 @returns Whether no character of it is visible

 @example
 ```ts
 rendersAsNothing({ text: '\u{200B}', },); // true: a zero-width space
 ```
 */
export function rendersAsNothing({ text, }: { readonly text: string; },): boolean {
  for (const character of text) {
    if (!SHOWS_NOTHING.test(character,))
      return false;
  }
  return true;
}

/**
 Whether one character is a default-ignorable code point (Unicode's Default_Ignorable_Code_Point): a zero-width or format
 character, the soft hyphen, the combining grapheme joiner, a filler, a variation selector, the byte order mark. Unicode derives
 the property without White_Space, so a space is none of them.

 @param character - one whole code point, which is two UTF-16 units beyond the first plane

 @returns Whether the character is default-ignorable

 @example
 ```ts
 const ignorable = isDefaultIgnorable({ character: '\u{200B}', },); // true
 ```
 */
export function isDefaultIgnorable({ character, }: { readonly character: string; },): boolean {
  return IS_DEFAULT_IGNORABLE.test(character,);
}

//endregion Renders as nothing
