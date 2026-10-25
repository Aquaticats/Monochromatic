import type { EditorFidelityTrial, } from './editor-fidelity-trials.ts';

//region Editor fidelity score
// THE DETERMINISTIC READING OF ONE EDITED TEXT against its trial. No model judges: every reading is a comparison of
// strings the trial already holds.
//
// FIXED has three kinds, tried in order. EXACT: the patched text is the reference, byte for byte. EQUIVALENT: it
// differs from the reference only in how much whitespace separates words, or follows the last word. DEFECT-ONLY: the
// defect is gone from the quoted region but something else changed too, which is also collateral. Only EXACT and
// EQUIVALENT are a clean fix; the probe's headline counts nothing else, because a reply that destroys the sentence
// around a number, or deletes the region it was shown, reads as defect-only.
//
// WHY WHITESPACE IS THE ONE EQUIVALENCE. The apply gate replaces the region and joins nothing: an editor whose
// replacement keeps a separator the reference lacks leaves the patched text with one space or line break more than the
// reference, and nothing else differs. Counting that as a miss would score the gate and not the editor, and the
// repository's own splice (`spliceOutSentence`) draws the same line when it removes a sentence and keeps one of the
// two separators. Runs of whitespace collapse to one space, and whitespace at the very end of the text is ignored,
// because a reviewed reference can end where its section ends and a perfect edit there leaves the separator that
// preceded the removed sentence at the end; no other normalization is applied, so a changed word, number or
// punctuation mark never passes as equivalent.
//
// WHEN THE DEFECT IS GONE FROM AN INSERTION. The inserted sentence is gone when the replacement shares no run of
// `SHARED_RUN_CHARS` characters with it, case and spacing aside: a softened sentence still says what the original does
// not, and the rule is that an addition is removed.
//
// COLLATERAL is any change outside the defect. The gate lets an edit land only inside the envelope, so a change
// outside the region would be a finding about the gate and is reported as collateral, with the defect not isolated.
// Inside the region, collateral is a change to the words around the defect.

/**
 Characters of the inserted sentence that a replacement may not repeat, in a row, if the sentence is to count as
 removed. Long enough that two English sentences do not share a run by chance, short enough that a softened or
 shortened copy of the sentence still does.
 */
const SHARED_RUN_CHARS = 16;

/**
 How a fix read.
 */
export type EditorFixKind = 'exact' | 'equivalent' | 'defect-only' | 'none';

/**
 The reading of one edited text.

 @example
 ```ts
 const score: EditorFidelityScore = { fixed: 'exact', collateral: false, };
 ```
 */
export type EditorFidelityScore = {
  /**
   Whether the defect is fixed, and how.
   */
  readonly fixed: EditorFixKind;

  /**
   Whether anything outside the defect changed.
   */
  readonly collateral: boolean;
};

/**
 Whether one character separates rather than says anything.

 @param character - single character to test

 @returns Whether it is whitespace

 @example
 ```ts
 const blank = isBlank({ character: '\n', },);
 ```
 */
function isBlank({ character, }: { readonly character: string; },): boolean {
  return character.trim() === '';
}

/**
 Collapses every run of whitespace to one space, which is the one normalization the equivalence allows.

 @param text - text to normalize

 @returns The text with each whitespace run written as a single space

 @example
 ```ts
 // flat is 'a b c'
 const flat = collapseWhitespace({ text: 'a  b\n\nc', },);
 ```
 */
function collapseWhitespace({ text, }: { readonly text: string; },): string {
  /**
   Characters kept, each run of blanks written once as a space.
   */
  const kept: string[] = [];
  /**
   Whether the character read last was blank.
   */
  let previousBlank = false;
  for (const character of text) {
    /**
     Whether this character separates rather than says anything.
     */
    const blank = isBlank({ character, },);
    if (!(blank && previousBlank))
      kept.push(blank ? ' ' : character,);
    previousBlank = blank;
  }
  /**
   The text with its blank runs collapsed.
   */
  const collapsed = kept.join('',);
  return collapsed;
}

/**
 Whether a replacement repeats a run of the inserted sentence, however it is cased or spaced.

 @param replacement - what the region became

 @param sentence - the inserted sentence, separators included

 @returns Whether some run of `SHARED_RUN_CHARS` characters of the sentence (the whole sentence when it is shorter)
 appears in the replacement

 @example
 ```ts
 const repeats = repeatsRun({ replacement: 'She travelled to a distant village.', sentence, },);
 ```
 */
function repeatsRun(
  {
    replacement,
    sentence,
  }: {
    readonly replacement: string;
    readonly sentence: string;
  },
): boolean {
  /**
   The replacement without case or spacing.
   */
  const flatReplacement = collapseWhitespace({ text: replacement, },)
    .toLowerCase();
  /**
   The sentence without case, spacing or its separators.
   */
  const flatSentence = collapseWhitespace({ text: sentence, },)
    .trim()
    .toLowerCase();
  /**
   Width of the run tested.
   */
  const width = Math.min(
    SHARED_RUN_CHARS,
    flatSentence.length,
  );
  if (width === 0)
    return false;
  for (let at = 0; at <= (flatSentence.length - width); at += 1) {
    if (flatReplacement.includes(flatSentence.slice(
      at,
      at + width,
    ),))
      return true;
  }
  return false;
}

/**
 Whether the region of the edited text fixes the defect, by the family's own test.

 @param trial - trial read

 @param replacement - what the region became

 @returns Whether the defect is gone from the region: the inserted sentence repeated nowhere in it, or the clean number
 back and the altered one gone

 @example
 ```ts
 const fixed = fixesDefect({ trial, replacement: '', },);
 ```
 */
function fixesDefect(
  {
    trial,
    replacement,
  }: {
    readonly trial: EditorFidelityTrial;
    readonly replacement: string;
  },
): boolean {
  if (trial.damageKind === 'insertion') {
    return !repeatsRun({
      replacement,
      sentence: trial.damagedDefect,
    },);
  }
  return replacement.includes(trial.cleanDefect,) && (!replacement.includes(trial.damagedDefect,));
}

/**
 Reads an edited text against its trial.

 @param trial - trial the edit answered

 @param patchedText - the translation after the apply gate

 @returns Whether the defect is fixed and whether anything else changed

 @example
 ```ts
 const { fixed, collateral, } = scoreEditedText({ trial, patchedText, },);
 ```
 */
export function scoreEditedText(
  {
    trial,
    patchedText,
  }: {
    readonly trial: EditorFidelityTrial;
    readonly patchedText: string;
  },
): EditorFidelityScore {
  if (patchedText === trial.cleanText) {
    return {
      fixed: 'exact',
      collateral: false,
    };
  }
  /**
   Whether the patched text is the reference once whitespace runs count as one and whitespace at the end is ignored.
   */
  const equivalent = collapseWhitespace({ text: patchedText, },)
    .trimEnd()
    === collapseWhitespace({ text: trial.cleanText, },)
      .trimEnd();

  /**
   Text the region held before the edit, kept in front of and behind it.
   */
  const head = trial.damagedText
    .slice(
    0,
    trial.region
      .startOffset,
    );
  /**
   Text after the region, which the edit must leave as it was.
   */
  const tail = trial.damagedText
    .slice(trial.region
      .endOffset,);
  if (
    (!patchedText.startsWith(head,))
      || (!patchedText.endsWith(tail,))
      || (patchedText.length < (head.length + tail.length))
  ) {
    return {
      fixed: equivalent ? 'equivalent' : 'none',
      collateral: true,
    };
  }

  /**
   What the region became.
   */
  const replacement = patchedText.slice(
    head.length,
    patchedText.length - tail.length,
  );
  if (fixesDefect({
    trial,
    replacement,
  },)) {
    return equivalent
      ? {
        fixed: 'equivalent',
        collateral: false,
      }
      : {
        fixed: 'defect-only',
        collateral: true,
      };
  }

  /**
   Region text in front of the defect, which an edit that leaves the surroundings alone keeps.
   */
  const before = trial.damagedText
    .slice(
    trial.region
      .startOffset,
    trial.defectStart,
    );
  /**
   Region text behind the defect.
   */
  const after = trial.damagedText
    .slice(
    Math.min(
      trial.defectStart
        + trial.damagedDefect
          .length,
      trial.region
        .endOffset,
    ),
    trial.region
      .endOffset,
    );
  return {
    fixed: 'none',
    collateral: !(replacement.startsWith(before,) && replacement.endsWith(after,)),
  };
}

//endregion Editor fidelity score
