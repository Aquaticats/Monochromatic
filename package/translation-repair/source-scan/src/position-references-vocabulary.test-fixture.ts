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

 @returns Words as a set

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
 below which", "a voice below quorum".

 A WORD THAT IS ALSO A VERB DOES NOT BELONG HERE ALONE (re-check 7, nit 2). "ties" stood in this list for "preservation
 above ties", and let "the asked count below ties the script" pass, the verb read as the object. It left the list, and
 the one comparison in the texts that used the noun is in the exemptions, with its reason.

 WHAT THE SCAN READS BEYOND THE WORD (re-check 8, nit A). Three groups of words in this list are something else as
 well, and each is read with the word after it, joined to it by spaces alone:
 `COMPARED_UNLESS_OBJECT_FOLLOWS` holds the words that are verbs ("the lines below zero the counter"), which compare
 unless a determiner, a quantifier or a pronoun follows; `COMPARED_ONLY_BEFORE_NAMED_THING` holds the quantifiers that
 float away from their noun ("the cases below all pass", "the checks below each assert a whisker"), which compare only
 when a determiner, "of", "else", "other" or "others" follows or nothing joins them to a next word ("above all, it
 naps"); `COMPARED_ONLY_BEFORE_KNOWN_NOUN` holds "that", which compares only before a noun of `NOUNS_AFTER_THAT`.

 THIRTEEN WORDS LEFT THE LIST because no text the guard reads needed them: an, his, her, some, half, exact, both and the
 numbers four, six, seven, eight, nine and ten. The count is of raw findings, with no exemption applied, that appear
 when one word alone is deleted; "regression" adds one, a true comparison, and stays. A position before a word that
 left is a pointer: "the lines below exact a fixed count", "the bowls below half the set" and "the checks below both
 fail" are found, and so is a comparison such as "a rate above four", which then needs an exemption with its reason.

 "that" AFTER A POSITION COMPARES ONLY BEFORE A NOUN THE SCAN KNOWS (ledger B952). A list of verbs ledger B951 had
 named read the verb after "that" in one form, so "the line below that writes it" and "the paragraph above that
 explained" passed. The reading is now the inverse: "that" compares only when a word of `NOUNS_AFTER_THAT` follows it
 ("a roster below that floor", "a cat above that of the dog"), and anything else, a verb in any form, a clause, a
 punctuation mark or the end of the phrase, reads the position as a pointer. The cost is that a new comparison such as
 "a rate above that <noun>" is found until the noun joins `NOUNS_AFTER_THAT` or the sentence gets an exemption with
 its reason.

 WHAT IT STILL DOES NOT READ. A quantifier followed by "of" reads as a comparison ("the persist below each of them"),
 which hides "the cases below all of which fail". Any other word that is both a noun and a verb, added to the list
 later, lets a pointer through unless it joins a group.
 */
export const COMPARED_OBJECTS: ReadonlySet<string> = wordSet({
  words: 'the a its their one zero it that this what any every each all which baseline threshold quorum twice hard '
    + 'total regression anything two three five sixteen',
},);

/**
 Words in the compared list that are also verbs: "the lines below zero the counter". After a position, one of these
 compares only when no word that opens an object or a clause follows it.
 */
export const COMPARED_UNLESS_OBJECT_FOLLOWS: ReadonlySet<string> = wordSet({
  words: 'baseline zero threshold total',
},);

/**
 Words in the compared list that float away from the noun they count: "the cases below all pass", "the checks below
 each assert a whisker". After a position, one of these compares only when what follows names the thing compared.
 */
export const COMPARED_ONLY_BEFORE_NAMED_THING: ReadonlySet<string> = wordSet({ words: 'all each', },);

/**
 Words in the compared list that open a clause or point at a thing: "the lines below that write it" and "the statement
 above that a run-off keeps its key holds" point, and "a roster below that floor" compares. After a position, one of
 these compares only when a word of `NOUNS_AFTER_THAT` follows it, since a list of the words that make it a pointer
 could never be complete (verbs take many forms).
 */
export const COMPARED_ONLY_BEFORE_KNOWN_NOUN: ReadonlySet<string> = wordSet({ words: 'that', },);

/**
 Words that, after a position and "that", show it compares: "a roster below that floor", "a cap below that speed",
 "a nap below that size", "a gap above that opening", "a cat above that of the dog". The six nouns are the ones the
 texts the guard reads put there (floor, speed, size, opening, calibration, percentile); "of" opens "that of".
 */
export const NOUNS_AFTER_THAT: ReadonlySet<string> = wordSet({
  words: 'floor speed size opening calibration percentile of',
},);

/**
 Words that open an object or a clause after a word of the compared list: a determiner, a quantifier or a pronoun.
 */
export const OBJECT_OPENERS: ReadonlySet<string> = wordSet({
  words: 'a an the no both every all some any its their his her this these those it they we',
},);

/**
 Words that, after a quantifier of the compared list, show it names what is compared: "above all else", "below each of
 them", "above all others".
 */
export const QUANTIFIED_NAMES: ReadonlySet<string> = wordSet({ words: 'of else other others', },);

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
