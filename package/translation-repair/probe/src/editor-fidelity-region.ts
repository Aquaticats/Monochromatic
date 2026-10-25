import {
  sentenceRegionAround,
  type SentenceRegion,
} from '@monochromatic-dev/translation-repair-repair/ts';

//region Editor fidelity insertion region
// THE REGION AN INSERTED SENTENCE IS QUOTED IN, wide enough that deleting it is not the fix.
//
// WHY NOT THE SENTENCE ALONE. A critic quotes the sentence it flags, and an editor shown that sentence as the whole of
// the region fixes the defect by replacing it with nothing. A reply that does only that, without reading the
// original, is then a clean fix of every insertion, and a card that sends it is scored as well as one that reasons.
// The re-check of round 3 measured it: equivalent, nothing else touched, in 22 of 23 insertions. Quoting the sentence
// with the one either side of it on its line makes the deletion take the neighbours with it, so the only clean fix is
// to write them back and drop the inserted one.
//
// THIS IS A DEPARTURE FROM PRODUCTION, stated as one. Production quotes the span a critic flagged, which for an
// addition is the added sentence. The probe widens the region on purpose, to measure the editor and not the rule.
//
// A SENTENCE IS NEVER REACHED ACROSS A LINE BREAK, because the sentence region itself stops there: a heading or a
// list line is its own region. An inserted sentence with no neighbour on either side reports zero, and the trial
// refuses it: its quoted region would be the sentence alone.

/**
 Line break, past which a neighbour is never taken.
 */
const LINE_BREAK = '\n';

/**
 Where the nearest character that says something sits on the same line, or that none does.

 @example
 ```ts
 const found: SayingOnLine = { present: true, at: 12, };
 ```
 */
type SayingOnLine =
  | {
    /**
     A character that says something is on the line.
     */
    readonly present: true;

    /**
     Its offset.
     */
    readonly at: number;
  }
    | {
    /**
     A line break or the end of the text comes first.
     */
      readonly present: false;
    };

/**
 Finds the nearest character that says something on the same line, walking from one position.

 @param text - passage read

 @param from - position of the first character to test

 @param step - one to walk forward, minus one to walk back

 @returns Where it is, or that a line break or the end of the text comes first

 @example
 ```ts
 const found = sayingOnLine({ text, from: 12, step: -1, },);
 ```
 */
function sayingOnLine(
  {
    text,
    from,
    step,
  }: {
    readonly text: string;
    readonly from: number;
    readonly step: 1 | -1;
  },
): SayingOnLine {
  for (let cursor = from; (cursor >= 0) && (cursor < text.length); cursor += step) {
    /**
     Character under the cursor.
     */
    const character = text.charAt(cursor,);
    if (character === LINE_BREAK)
      break;
    if (character.trim() !== '') {
      return {
        present: true,
        at: cursor,
      };
    }
  }
  return { present: false, };
}

/**
 The region of an inserted sentence widened by the sentence that precedes it and the one that follows it, on its line.

 @param text - the damaged text

 @param region - the inserted sentence, whole, with the separators left out

 @returns The widened region, and how many neighbours it took (zero, one or two); a side with no neighbour on the
 line keeps the sentence's own edge

 @example
 ```ts
 const { region: widened, neighbours, } = insertionRegionWithNeighbours({ text, region, },);
 ```
 */
export function insertionRegionWithNeighbours(
  {
    text,
    region,
  }: {
    readonly text: string;
    readonly region: SentenceRegion;
  },
): {
  readonly region: SentenceRegion;
  readonly neighbours: number;
} {
  /**
   Last character of the sentence before, when one is on the line.
   */
  const before = sayingOnLine({
    text,
    from: region.startOffset - 1,
    step: -1,
  },);
  /**
   First character of the sentence after, when one is on the line.
   */
  const after = sayingOnLine({
    text,
    from: region.endOffset,
    step: 1,
  },);
  return {
    region: {
      startOffset: before.present
        ? sentenceRegionAround({
          text,
          startOffset: before.at,
          endOffset: before.at + 1,
        },)
          .startOffset
        : region.startOffset,
      endOffset: after.present
        ? sentenceRegionAround({
          text,
          startOffset: after.at,
          endOffset: after.at + 1,
        },)
          .endOffset
        : region.endOffset,
    },
    neighbours: [
      before,
      after,
    ].filter(function isPresent(side,): boolean {
      return side.present;
    },)
      .length,
  };
}

//endregion Editor fidelity insertion region
