//region Obligatory differences
// LEDGER L5, 2026-09-28. The critic carried this block alone; the panel
// judging the critic's claims carried one line on conjunctions, connectives,
// pronouns and small words, so a claim the critic's own rules would never
// file could still be upheld, and one was: an addition claim against a
// possessor English had to state (TianqiChen66616 slice 20). Rendered sheets
// showed the panel carried none of this block. Stated once here, in neutral
// voice, so the critic filing a claim and the panel voting on it read the same
// rule; each sheet adds the line saying what it does with such a difference.

/**
 Differences between the languages that are never defects, one rule per line.

 @example
 ```ts
 const policy = `${OBLIGATORY_DIFFERENCES}\nVote unsupported on a claim whose whole case is one of these.`;
 ```
 */
export const OBLIGATORY_DIFFERENCES: string = `Obligatory differences between the two languages are never defects. Each language forces choices the other leaves open, and meeting the TRANSLATION's own requirements is not an addition, an omission, or a mistranslation:
- Supplying what the TRANSLATION's grammar or readability requires and the ORIGINAL can omit (a subject or object, a pronoun, a possessor, a conjunction or connective, a number, an article, a tense) is REQUIRED, not added. It is a defect only when the supplied choice is the WRONG one, which the reading the ORIGINAL supports shows.
- Punctuation and quotation conventions differ. Adding quotation marks, italics, or other marks the TRANSLATION's conventions call for, to set off speech, a title, or a nickname the ORIGINAL marks by other means or not at all, is not an addition.
- A distinction one language marks and the other does not (Chinese marks plural address in a pronoun; English does not) cannot be carried over. Rendering it with the only available form is not an omission, and it is no defect when the TRANSLATION has no means to make the distinction.
- Where the ORIGINAL leaves a connection to context that the TRANSLATION's reader cannot recover, making it explicit is legitimate. It is a defect only when the added reading is unsupported by the ORIGINAL, not merely because it is absent from the words.`;

//endregion Obligatory differences
