//region Repeat groups bounds
// Constants of the repeat-group rule, each with the census it was measured by,
// and the character sets the reading and the clauses are cut on.

/**
 Shortest maximal repeat, in non-space characters.

 Source: the design's census, Rule A (section 2.1), which fixed twelve on the
 pinned corpus at commit a41fc607ea5a70d8a7625cc67d5ed8c444f53379. Exactly
 twelve is reported; eleven is not.
 */
export const MAXIMAL_REPEAT_MINIMUM = 12;

/**
 Shortest common prefix of two refrain clauses.

 Source: the design's census, Rule B (section 2.1), main setting, prefix of four
 characters, pinned corpus at commit a41fc607ea5a70d8a7625cc67d5ed8c444f53379.
 */
export const REFRAIN_PREFIX_MINIMUM = 4;

/**
 Shortest common suffix of two refrain clauses.

 Source: the design's census, Rule B (section 2.1), main setting, suffix of
 three characters, pinned corpus at commit a41fc607ea5a70d8a7625cc67d5ed8c444f53379.
 */
export const REFRAIN_SUFFIX_MINIMUM = 3;

/**
 Fewest characters a refrain's varying slot may hold.

 Source: the design's Rule B (section 2.1), which requires each slot to hold at
 least one character, since a slot of none is an identical clause.
 */
export const REFRAIN_SLOT_MINIMUM = 1;

/**
 Most characters a refrain's varying slot may hold.

 Source: the design's census, Rule B (section 2.1), main setting, slot of eight
 characters, pinned corpus at commit a41fc607ea5a70d8a7625cc67d5ed8c444f53379.
 */
export const REFRAIN_SLOT_MAXIMUM = 8;

/**
 Fewest characters a clause may hold to be compared at all.

 Source: the design's census, Rule B (section 2.1), main setting, clause of six
 characters, pinned corpus at commit a41fc607ea5a70d8a7625cc67d5ed8c444f53379.
 */
export const REFRAIN_CLAUSE_MINIMUM = 6;

/**
 Punctuation that ends a refrain clause, the design's clause boundary set
 (section 2.1, Rule B). Quotation marks are boundaries too, so a clause never
 runs into quoted speech.
 */
export const CLAUSE_BOUNDARY: string = '。！？!?，,、；;：:…「」『』“”"()（）';

/**
 Opening quotation marks, each one raising the quoted depth of the wording after it.
 */
export const QUOTE_OPENERS: string = '「“『';

/**
 Closing quotation marks, each one lowering the quoted depth of the wording after it.
 */
export const QUOTE_CLOSERS: string = '」”』';

/**
 Straight double quotation mark, which opens a quotation when none of its kind is open
 in the paragraph and closes the one that is, since one character stands for both ends.
 The pinned corpus writes it in body wording, so it counts as quotation like the paired marks.
 */
export const STRAIGHT_QUOTE: string = '"';

/**
 Character standing between two runs of wording, so that no repeat spans it.

 Built from its code point rather than written as an escape, so that no
 escape reaches a tool that decodes one. A page holding this code point reads
 as a break, which is the safe reading.
 */
export const BREAK: string = String.fromCodePoint(0,);

/**
 Letters that open an inline tag when they follow a less-than sign.
 */
export const TAG_LETTERS: string = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';

/**
 Schemes a bare URL begins with, which make it markup.
 */
export const URL_SCHEMES: readonly string[] = [
  'https://',
  'http://',
];

//endregion Repeat groups bounds
