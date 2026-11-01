import {
  isAsciiDigit,
  isAsciiDigits,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { isWhitespace, } from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Consistency label form
// THE LABEL A JUDGE WROTE, READ AGAINST THE LABELS THE SHEET SHOWED (the lead's ruling of 2026-10-15 on the run of
// TianqiChen66625). Seven replies of that run named the option that won anyway and were refused for the way they wrote
// it: `Occurrence 3` and `Candidate 2` for `OCCURRENCE 3` and `CANDIDATE 2`, and a bare `3` or `2`. A name that stands
// for exactly one shown label is that label, whatever its case, whatever white space surrounds it or runs inside it, and
// whether it is the label or only the number the label ends in; a name that stands for none, or for two, is still
// refused, and so is the word of another sheet (a choice among candidates is shown places too, so `Occurrence 2` there
// would name a place, not a candidate). The label a reply is stored under is the shown one, in the sheet's own form.

/**
 A label or a name in the form labels are compared in: white space trimmed and collapsed to single spaces, upper case.

 @param text - label or name as written

 @returns The comparison form

 @example
 ```ts
 const form = comparisonFormOf({ text: ' Occurrence   3 ', },); // 'OCCURRENCE 3'
 ```
 */
function comparisonFormOf({ text, }: { readonly text: string; },): string {
  /**
   The characters of the trimmed name, each white space character read as one space.
   */
  const characters = Array.from(
    text.trim(),
    function spaced(character,): string {
      return isWhitespace({ character, },) ? ' ' : character;
    },
  );
  return characters
    .filter(function startsNoRepeatSpace(
      character,
      at,
    ): boolean {
      return (character !== ' ') || (characters[at - 1] !== ' ');
    },)
    .join('',)
    .toUpperCase();
}

/**
 The ASCII digits a label ends in.

 @param label - shown label

 @returns The digits as a list of one, none when the label does not end in a digit

 @example
 ```ts
 const [number,] = trailingNumberOf({ label: 'OCCURRENCE 12', },); // '12'
 ```
 */
function trailingNumberOf({ label, }: { readonly label: string; },): readonly string[] {
  /**
   The label's characters.
   */
  const characters = Array.from(label,);
  /**
   Where the last character that is no digit stands, before the start when every character is a digit.
   */
  const lastOther = characters.findLastIndex(function isOther(character,): boolean {
    return !isAsciiDigit({ character, },);
  },);
  /**
   The digits after it.
   */
  const digits = characters
    .slice(lastOther + 1,)
    .join('',);
  return (digits === '') ? [] : [digits,];
}

/**
 The shown labels of one sheet, looked up by the form they are compared in and by the number they end in.
 */
type ShownIndex = {
  readonly byForm: ReadonlyMap<string, readonly string[]>;
  readonly byNumber: ReadonlyMap<string, readonly string[]>;
};

/**
 Appends a label to the list a key holds, in place, starting the list where the key has none.

 @param map - lookup being built

 @param key - comparison form or trailing number the label is found under

 @param label - shown label

 @example
 ```ts
 appendTo({ map: byNumber, key: '3', label: 'OCCURRENCE 3', },);
 ```
 */
function appendTo(
  {
    map,
    key,
    label,
  }: {
    readonly map: Map<string, string[]>;
    readonly key: string;
    readonly label: string;
  },
): void {
  /**
   The labels the key already holds.
   */
  const list = map.get(key,);
  if (list === undefined) {
    map.set(
      key,
      [label,],
    );
    return;
  }
  list.push(label,);
}

/**
 Reads a list of shown labels into the lookups by comparison form and by trailing number.

 @param shown - labels the sheet showed

 @returns The lookups, each label listed once under each key it has

 @example
 ```ts
 const { byNumber, } = shownIndexOf({ shown: ['OCCURRENCE 1', 'OCCURRENCE 2',], },); // byNumber.get('2') is ['OCCURRENCE 2']
 ```
 */
function shownIndexOf({ shown, }: { readonly shown: readonly string[]; },): ShownIndex {
  /**
   Labels by comparison form, built once.
   */
  const byForm = new Map<string, string[]>();
  /**
   Labels by trailing number, built once.
   */
  const byNumber = new Map<string, string[]>();
  for (const label of new Set(shown,)) {
    appendTo({
      map: byForm,
      key: comparisonFormOf({ text: label, },),
      label,
    },);
    /**
     The digits the label ends in, if it ends in any.
     */
    const [number,] = trailingNumberOf({ label, },);
    if (number !== undefined)
      appendTo({
        map: byNumber,
        key: number,
        label,
      },);
  }
  return {
    byForm,
    byNumber,
  };
}

/**
 Builds the lookup of the one shown label a name stands for, over a list of shown labels read once, so a caller that looks up
 many names of one sheet pays for the list once. The lookup keeps nothing between calls of this function: it answers for the
 labels the list held when it was built.

 @param shown - labels the sheet showed, the only labels a reply may use

 @returns A function from what the judge wrote as a label to the shown label, as a list of one, none when the name stands for no
 shown label or for more than one

 @example
 ```ts
 const labelOf = shownLabelReaderOf({ shown: ['OCCURRENCE 3', 'OCCURRENCE 4',], },);
 const [label,] = labelOf('4',); // 'OCCURRENCE 4'
 ```
 */
export function shownLabelReaderOf({ shown, }: { readonly shown: readonly string[]; },): (name: string) => readonly string[] {
  /**
   The shown labels by comparison form and by trailing number, read from the list once however many names are looked up.
   */
  const {
    byForm: formTable,
    byNumber: numberTable,
  } = shownIndexOf({ shown, },);
  return function shownLabelFor(name: string,): readonly string[] {
    /**
     The name in the form labels are compared in.
     */
    const form = comparisonFormOf({ text: name, },);
    /**
     Whether the name is only a number.
     */
    const isNumber = isAsciiDigits({ text: form, },);
    /**
     The shown labels in the name's comparison form.
     */
    const byForm = formTable.get(form,) ?? [];
    /**
     The shown labels ending in the number, when the name is only a number.
     */
    const byNumber = isNumber ? (numberTable.get(form,) ?? []) : [];
    /**
     Every distinct shown label the name stands for.
     */
    const standsFor = [
      ...new Set([
        ...byForm,
        ...byNumber,
      ],),
    ];
    return (standsFor.length === 1) ? standsFor : [];
  };
}

/**
 Finds the one shown label a name stands for.

 @param name - what the judge wrote as a label

 @param shown - labels the sheet showed, the only labels a reply may use

 @returns The shown label as a list of one, none when the name stands for no shown label or for more than one

 @example
 ```ts
 const [label,] = shownLabelOf({ name: ' occurrence 3 ', shown: ['OCCURRENCE 3', 'OCCURRENCE 4',], },); // 'OCCURRENCE 3'
 ```
 */
export function shownLabelOf(
  {
    name,
    shown,
  }: {
    readonly name: string;
    readonly shown: readonly string[];
  },
): readonly string[] {
  return shownLabelReaderOf({ shown, },)(name,);
}

//endregion Consistency label form
