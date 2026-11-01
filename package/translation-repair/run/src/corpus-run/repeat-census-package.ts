import { chunkByHeadings, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type RepeatGroup,
  repeatGroupsOf,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  CONTROL_ENTRY_ID,
  CONTROL_MAXIMAL_LINES,
  CONTROL_REFRAIN_LINES,
  type ControlReading,
  controlReadingOf,
  coversAllLines,
  occupiedLinesOf,
} from './repeat-census-control.ts';

//region Repeat census package
// The census's second reading: the package's own detection (`repeatGroupsOf`,
// which reads the parse and not the lines) run over the same pages, so that its
// counts stand beside the line census's and the live case is found by the code
// that will ship. The slices are the parse's heading chunks, which hold every
// body paragraph, so no occurrence is expected to stay unplaced.

/**
 One entry's repeat groups as the package finds them.
 */
export type EntryGroups = {
  /**
   Entry id, the directory name under `people/`.
   */
  readonly id: string;

  /**
   Groups the package's detection returns for that page.
   */
  readonly groups: readonly RepeatGroup[];
};

/**
 A group the package found, with the entry it stands in.
 */
type PlacedGroup = {
  /**
   Entry id, the directory name under `people/`.
   */
  readonly id: string;

  /**
   The group.
   */
  readonly group: RepeatGroup;
};

/**
 Runs the package's detection over one page, sliced by its headings.

 @param text - page text

 @returns Repeat groups of the page

 @throws {@link Error} when the page does not parse, as the parse itself refuses

 @example
 ```ts
 const groups = packageGroupsOfPage({ text, },);
 ```
 */
export function packageGroupsOfPage({ text, }: { readonly text: string; },): readonly RepeatGroup[] {
  return repeatGroupsOf({
    text,
    slices: chunkByHeadings({ document: parseDocument({ text, },), },),
  },);
}

/**
 Reads the positive control off the package's own detection: the groups of the
 live case's entry.

 @param entries - each entry's groups

 @returns `absent` where the control entry is not among them, else whether each control held

 @example
 ```ts
 const reading = packageControlOf({ entries, },);
 ```
 */
export function packageControlOf({ entries, }: { readonly entries: readonly EntryGroups[]; },): ControlReading {
  /**
   Positive control: the groups of the live case's entry.
   */
  const control = entries.find(function isControl(entry,): boolean {
    return entry.id === CONTROL_ENTRY_ID;
  },);
  if (control === undefined)
    return 'absent';
  return controlReadingOf({
    refrain: control.groups
      .some(function coversRefrain(group,): boolean {
        return (group.kind === 'refrain') && coversAllLines({
          held: occupiedLinesOf({ occurrences: group.occurrences, },),
          lines: CONTROL_REFRAIN_LINES,
        },);
      },),
    maximal: control.groups
      .some(function coversMaximal(group,): boolean {
        return (group.kind === 'maximal') && coversAllLines({
          held: occupiedLinesOf({ occurrences: group.occurrences, },),
          lines: CONTROL_MAXIMAL_LINES,
        },);
      },),
  },);
}

/**
 Builds the printed summary of the package's groups over the corpus.

 @param entries - each entry's groups

 @returns Counts, one detail row per group (entry, kind, lines, no wording) and the positive control

 @example
 ```ts
 const summary = packageSummary({ entries, },);
 ```
 */
export function packageSummary({ entries, }: { readonly entries: readonly EntryGroups[]; },): Record<string, unknown> {
  /**
   Every group with the entry it stands in.
   */
  const placed = entries.flatMap(function groupsOf(entry,): readonly PlacedGroup[] {
    return entry.groups
      .map(function withEntry(group,): PlacedGroup {
        return {
          id: entry.id,
          group,
        };
      },);
  },);
  return {
    groups: placed.length,
    maximalGroups: placed.filter(function isMaximal(entry,): boolean {
      return entry.group
        .kind
        === 'maximal';
    },)
      .length,
    refrainGroups: placed.filter(function isRefrain(entry,): boolean {
      return entry.group
        .kind
        === 'refrain';
    },)
      .length,
    pagesWithAnyGroup: new Set(placed.map(function idOf(entry,) {
      return entry.id;
    },),).size,
    unplacedOccurrences: placed.reduce(
      function sumUnplaced(
        total,
        entry,
      ) {
        return total
          + entry.group
            .unplaced
            .length;
      },
      0,
    ),
    detail: placed.map(function detailOf(entry,) {
      return {
        id: entry.id,
        kind: entry.group
          .kind,
        lines: occupiedLinesOf({ occurrences: entry.group
          .occurrences, },),
        unplacedLines: entry.group
          .unplaced
          .map(function lineOfUnplaced(occurrence,): number {
            return occurrence.line;
          },),
      };
    },),
    control: packageControlOf({ entries, },),
  };
}

//endregion Repeat census package
