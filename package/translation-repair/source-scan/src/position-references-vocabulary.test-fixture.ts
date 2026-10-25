/**
 The vocabulary of a reference by position (ledger D33), read by
 `position-references.test-fixture.ts` and
 `position-references-text.test-fixture.ts`: the words that point, the words
 before them that make a phrase a comparison or a placement, and the words
 after them that make it a comparison.

 WORD LISTS ARE WRITTEN AS ONE SPACED STRING EACH and split once, so a list of
 a hundred nouns reads as a paragraph rather than a hundred lines.

 @module
 */

/**
 A set of the words a spaced list names.

 @param words - words separated by single spaces

 @returns The words as a set

 @example
 ```ts
 wordSet({ words: 'above below', },).has('above',); // true
 ```
 */
function wordSet({ words, }: { readonly words: string; },): ReadonlySet<string> {
  return new Set(words.split(' ',),);
}

//region Vocabulary

/**
 Words that point by position.
 */
export const POSITIONS: ReadonlySet<string> = wordSet({ words: 'above below', },);

/**
 Verbs that, before a position, cite what they point at.
 */
export const REFERENCE_VERBS: ReadonlySet<string> = wordSet({
  words: 'see as noted described shown listed defined stated explained mentioned quoted named discussed given '
    + 'written documented recorded argued said covered introduced cited established',
},);

/**
 Words that, before a position, make it a comparison, a bound or a placement
 rather than a pointer: "at or above quorum", "unbounded below", "far above
 anything", "sits above U+2E80", "stands above" a footnote on a page.

 WHAT A LIST OF NOUNS COULD NOT DO. The guard first read a position as a
 pointer only after a listed structure noun, and a scan for positions ending
 a phrase found a couple of hundred pointers after nouns no list held ("the
 estimate below", "the walk below"). Any word before a position is now read
 as pointing, and this list holds the words that read otherwise.
 */
export const POSITION_MARKERS: ReadonlySet<string> = wordSet({
  words: 'or and at from bounded unbounded times far never directly level levels up set sit sits fall falls stand '
    + 'stands stood',
},);

/**
 Words that, after a position, make it a comparison rather than a pointer:
 "ranks the house rules above its own", "a rate above one", "the threshold
 below which", "a voice below half".

 A WORD THAT IS ALSO A VERB DOES NOT BELONG HERE (re-check 7, nit 2). "ties" stood in this list for "preservation above
 ties", and let "the asked count below ties the script" pass, the verb read as the object. The one comparison in the
 texts that used the noun is in the exemptions, with its reason. "total" can be a verb as well, but no sentence of the
 texts reads it as one after a position, and the list keeps it until one does.
 */
export const COMPARED_OBJECTS: ReadonlySet<string> = wordSet({
  words: 'the a an its their his her one zero it that this what any every each some all which baseline threshold '
    + 'quorum half twice hard exact total regression both anything two three four five six seven eight nine ten '
    + 'sixteen',
},);

/**
 Characters that, one space or more after a position, name its target or make
 it a comparison: a figure, a code span, a quoted heading, a link, a sign.
 */
export const NAMING_OPENERS: ReadonlySet<string> = new Set([
  '$',
  '`',
  '"',
  '\u{201C}',
  '{',
  '-',
  '+',
],);

/**
 Words that open a structure noun before "before this" or "after it".
 */
const DETERMINERS: ReadonlySet<string> = wordSet({ words: 'the this that every each all', },);

/**
 Nouns that, before "before this" or "after it", name a unit of the text.
 */
const SEQUENCE_NOUNS: ReadonlySet<string> = wordSet({
  words: 'case cases test tests paragraph paragraphs section sections note notes comment comments entry entries '
    + 'heading headings bullet bullets sentence sentences list lists table tables figure figures rule rules',
},);

/**
 Words naming the text a reference sits in: "earlier in this file".
 */
const CONTAINERS: ReadonlySet<string> = wordSet({
  words: 'file module doc document section entry ledger test suite list comment region note paragraph chapter log',
},);

/**
 Characters a wrapped string or comment leaves between two words of one
 phrase once its lines are joined: spaces, the quote that closed one line's
 string and the sign that joined the next.
 */
export const JOIN_GAP: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\'',
  '`',
  '+',
],);

/**
 Markup a wrapped comment or quote-block line opens with, longest first so a
 doc comment is not read as a plain one.
 */
export const CONTINUATION_MARKERS: readonly string[] = [
  '/**',
  '//',
  '/*',
  '*/',
  '*',
  '>',
];

/**
 Characters of context kept on each side of a reference, which an exemption
 is matched against.
 */
export const WINDOW_REACH = 80;

/**
 Sequences a reference by position spells word by word: a reference verb
 before a position ("see above"), a structure noun before "before this" or
 "after it", a sequence noun before "before this one", a position "in this
 file", its kin "further up in this file", and "the former" or "the latter".
 */
export const SEQUENCES: readonly (readonly ReadonlySet<string>[])[] = [
  [
    REFERENCE_VERBS,
    POSITIONS,
  ],
  [
    DETERMINERS,
    SEQUENCE_NOUNS,
    wordSet({ words: 'before after', },),
    wordSet({ words: 'this it them', },),
  ],
  [
    SEQUENCE_NOUNS,
    wordSet({ words: 'before after', },),
    wordSet({ words: 'this', },),
    wordSet({ words: 'one', },),
  ],
  [
    wordSet({ words: 'earlier later above below', },),
    wordSet({ words: 'in', },),
    wordSet({ words: 'this', },),
    CONTAINERS,
  ],
  [
    wordSet({ words: 'further higher lower', },),
    wordSet({ words: 'up down', },),
    wordSet({ words: 'in', },),
    wordSet({ words: 'this', },),
    CONTAINERS,
  ],
  [
    wordSet({ words: 'the', },),
    wordSet({ words: 'former latter', },),
  ],
];

//endregion Vocabulary
