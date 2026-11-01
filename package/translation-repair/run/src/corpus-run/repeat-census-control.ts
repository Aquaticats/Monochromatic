import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Repeat census control
// The positive control of the census: the entry and the source lines the design
// reads as the live case, which every census of the pinned corpus must find,
// and the decision which controls failed, which the command turns into its exit.

/* oxlint-disable no-magic-numbers -- the control lines are the source lines the design cites for the live case, which are data rather than arithmetic */
/**
 Entry the design reads as the live case.
 */
export const CONTROL_ENTRY_ID = 'TianqiChen666';

/**
 Lines of the live case the positive check reads: the refrain's four lines.
 */
export const CONTROL_REFRAIN_LINES: readonly number[] = [
  17,
  25,
  33,
  37,
];

/**
 Lines of the live case the positive check reads for the repeated sentence.
 */
export const CONTROL_MAXIMAL_LINES: readonly number[] = [
  35,
  55,
];
/* oxlint-enable no-magic-numbers */

/**
 What one reading of the census found for the positive controls: the control
 entry absent from the census, or each control held or not.

 @example
 ```ts
 const reading: ControlReading = { refrainCoversLines17To37: true, maximalCoversLines35And55: false, };
 ```
 */
export type ControlReading =
  | 'absent'
    | {
    /**
     Whether a refrain group stands on every one of the refrain's four lines.
     */
      readonly refrainCoversLines17To37: boolean;

    /**
     Whether a maximal group stands on both lines of the repeated sentence.
     */
      readonly maximalCoversLines35And55: boolean;
    };

/**
 Builds the reading of a census that found the control entry, under the names
 the census prints the controls by.

 @param refrain - whether a refrain group covers the refrain's lines

 @param maximal - whether a maximal group covers the repeated sentence's lines

 @returns Reading with each control held or not

 @example
 ```ts
 const reading = controlReadingOf({ refrain: true, maximal: false, },);
 ```
 */
export function controlReadingOf(
  {
    refrain,
    maximal,
  }: {
    readonly refrain: boolean;
    readonly maximal: boolean;
  },
): ControlReading {
  return {
    refrainCoversLines17To37: refrain,
    maximalCoversLines35And55: maximal,
  };
}

/**
 Names of the controls one reading failed, each led by the reading's own name
 (`lines` or `package`). A reading that found no control entry failed both of
 its controls, since nothing was checked, and says why beside each name.

 @param source - name of the reading, as the failures are led by it

 @param reading - what that reading found

 @returns One name for each control that read false, or both where the entry is absent

 @example
 ```ts
 const failed = failuresOfReading({ source: 'lines', reading: 'absent', },);
 ```
 */
function failuresOfReading(
  {
    source,
    reading,
  }: {
    readonly source: 'lines' | 'package';
    readonly reading: ControlReading;
  },
): readonly string[] {
  if (reading === 'absent') {
    return Object.keys(controlReadingOf({
      refrain: false,
      maximal: false,
    },),)
      .map(function absentName(name,): string {
        return `${source}.${name} (the ${CONTROL_ENTRY_ID} entry is absent from the census)`;
      },);
  }
  return Object.entries(reading,)
    .filter(function readFalse([, held,]: ReadonlyEntry<string, boolean>,): boolean {
      return !held;
    },)
    .map(function failedName([name,]: ReadonlyEntry<string, boolean>,): string {
      return `${source}.${name}`;
    },);
}

/**
 Names of every positive control that read false or could not be read, over
 both readings of the census, so the command can exit non-zero naming them
 while still printing its whole report.

 @param readings - what the line reading and the package's own detection each found

 @returns Failed control names, the line reading's first and each reading's in the order the report prints them; empty when every control held

 @example
 ```ts
 const failed = failedControlsOf({ readings: { lines: 'absent', package: 'absent', }, },);
 ```
 */
export function failedControlsOf(
  {
    readings,
  }: {
    readonly readings: {
      readonly lines: ControlReading;
      readonly package: ControlReading;
    };
  },
): readonly string[] {
  return [
    ...failuresOfReading({
      source: 'lines',
      reading: readings.lines,
    },),
    ...failuresOfReading({
      source: 'package',
      reading: readings.package,
    },),
  ];
}

/**
 Lines a group's occurrences stand on.

 @param occurrences - places the group stands, each with its one-based line

 @returns The line of each occurrence, in the order given

 @example
 ```ts
 const held = occupiedLinesOf({ occurrences: [{ line: 17, },], },);
 ```
 */
export function occupiedLinesOf(
  { occurrences, }: { readonly occurrences: readonly { readonly line: number; }[]; },
): readonly number[] {
  return occurrences.map(function lineOfOccurrence(occurrence,): number {
    return occurrence.line;
  },);
}

/**
 Whether a group stands on every one of the given lines.

 @param held - lines the group has an occurrence on

 @param lines - lines every one of which must be occupied

 @returns Whether each line holds an occurrence of the group

 @example
 ```ts
 const covers = coversAllLines({ held: [17, 25,], lines: [17, 25,], },);
 ```
 */
export function coversAllLines(
  {
    held,
    lines,
  }: {
    readonly held: readonly number[];
    readonly lines: readonly number[];
  },
): boolean {
  return lines.every(function occupied(line,): boolean {
    return held.includes(line,);
  },);
}

//endregion Repeat census control
