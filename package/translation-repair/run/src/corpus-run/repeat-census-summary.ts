import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type CensusGroup,
  REFRAIN_SETTINGS,
} from './repeat-census-lines.ts';
import {
  CONTROL_ENTRY_ID,
  CONTROL_MAXIMAL_LINES,
  CONTROL_REFRAIN_LINES,
  type ControlReading,
  controlReadingOf,
  coversAllLines,
  occupiedLinesOf,
} from './repeat-census-control.ts';
import {
  type EntryGroups,
  packageSummary,
} from './repeat-census-package.ts';
import type { censusOfPage, } from './repeat-census-page.ts';

//region Repeat census summary
// The printed summary of the census: counts, the body-only and same-line
// groups by entry, and the positive control. Nothing here reads the corpus.

/**
 Occurrence count from which a group is counted in the open bucket.
 */
const MANY_OCCURRENCES = 5;

/**
 Occurrence counts the histogram names one by one.
 */
const SIZE_TWO = 2;

/**
 Occurrence count of a three-occurrence group.
 */
const SIZE_THREE = 3;

/**
 Occurrence count of a four-occurrence group.
 */
const SIZE_FOUR = 4;

/**
 A group of the census with the entry it stands in.
 */
type PlacedGroup = {
  /**
   Entry id, the directory name under `people/`.
   */
  readonly id: string;

  /**
   The group.
   */
  readonly group: CensusGroup;
};

/**
 Line-role flags that take a group out of the body-only count, as the design's census defined them: a front-matter,
 blockquote, heading, HTML or footnote line. In-quote and same-line flags do not count.
 */
const NON_BODY_FLAGS: ReadonlySet<string> = new Set([
  'frontmatter',
  'quote',
  'heading',
  'html',
  'footnote',
]);

/**
 One entry's page and its census.
 */
export type EntryCensus = {
  /**
   Entry id, the directory name under `people/`.
   */
  readonly id: string;

  /**
   Census of that page.
   */
  readonly census: ReturnType<typeof censusOfPage>;

  /**
   Groups the package's own detection finds on that page.
   */
  readonly groups: EntryGroups['groups'];
};

/**
 Counts the groups with two, three, four and five or more occurrences.

 @param groups - groups to count

 @returns Count per bucket, from two occurrences up to five or more

 @example
 ```ts
 const histogram = sizeHistogram({ groups, },);
 ```
 */
function sizeHistogram({ groups, }: { readonly groups: readonly CensusGroup[]; },): {
  readonly two: number;
  readonly three: number;
  readonly four: number;
  readonly fiveOrMore: number;
} {
  /**
   Occurrence count of each group.
   */
  const counts = groups.map(function countOf(group,): number {
    return group.occurrences
      .length;
  },);
  return {
    two: counts.filter(function isTwo(count,): boolean {
      return count === SIZE_TWO;
    },)
      .length,
    three: counts.filter(function isThree(count,): boolean {
      return count === SIZE_THREE;
    },)
      .length,
    four: counts.filter(function isFour(count,): boolean {
      return count === SIZE_FOUR;
    },)
      .length,
    fiveOrMore: counts.filter(function isMany(count,): boolean {
      return count >= MANY_OCCURRENCES;
    },)
      .length,
  };
}

/**
 Detail of a group for the printed summary: its entry, kind, lines and flags.

 @param entry - group with the entry it stands in

 @returns Printable detail, without any wording

 @example
 ```ts
 const detail = detailOf({ entry, },);
 ```
 */
function detailOf(
  {
    entry,
  }: {
    readonly entry: PlacedGroup;
  },
): {
  readonly id: string;
  readonly kind: string;
  readonly lines: readonly number[];
  readonly flags: readonly string[];
} {
  return {
    id: entry.id,
    kind: entry.group
      .kind,
    lines: entry.group
      .occurrences
      .map(function lineOfOccurrence(occurrence,): number {
        return occurrence.line;
      },),
    flags: entry.group
      .flags,
  };
}

/**
 Reads the positive control off the line reading: the entry whose refrain and
 sentence the design reads as the live case.

 @param entries - each entry's census

 @returns `absent` where the control entry is not among them, else whether each control held

 @example
 ```ts
 const reading = lineControlOf({ entries, },);
 ```
 */
export function lineControlOf({ entries, }: { readonly entries: readonly EntryCensus[]; },): ControlReading {
  /**
   Positive control: the entry whose refrain and sentence the design reads as the live case.
   */
  const control = entries.find(function isControl(entry,): boolean {
    return entry.id === CONTROL_ENTRY_ID;
  },);
  if (control === undefined)
    return 'absent';
  return controlReadingOf({
    refrain: control.census
      .refrain
      .some(function coversLines(group,): boolean {
        return coversAllLines({
          held: occupiedLinesOf({ occurrences: group.occurrences, },),
          lines: CONTROL_REFRAIN_LINES,
        },);
      },),
    maximal: control.census
      .maximal
      .some(function coversLines(group,): boolean {
        return coversAllLines({
          held: occupiedLinesOf({ occurrences: group.occurrences, },),
          lines: CONTROL_MAXIMAL_LINES,
        },);
      },),
  },);
}

/**
 Builds the summary the census prints.

 @param entries - each entry's census

 @returns Counts and the positive control, as printable data

 @example
 ```ts
 const summary = summarise({ entries, },);
 ```
 */
export function summarise({ entries, }: { readonly entries: readonly EntryCensus[]; },): Record<string, unknown> {
  /**
   Every maximal group, with its entry.
   */
  const maximal = entries.flatMap(function maximalOf(entry,) {
    return entry.census
      .maximal
      .map(function withEntry(group,): PlacedGroup {
        return {
          id: entry.id,
          group,
        };
      },);
  },);

  /**
   Every refrain group under the main setting, with its entry.
   */
  const refrain = entries.flatMap(function refrainOf(entry,) {
    return entry.census
      .refrain
      .map(function withEntry(group,): PlacedGroup {
        return {
          id: entry.id,
          group,
        };
      },);
  },);

  /**
   Every group under the main setting, maximal and refrain.
   */
  const all = [
    ...maximal,
    ...refrain,
  ];

  /**
   Groups that carry no non-body flag.
   */
  const bodyOnly = all.filter(function bodyOnlyGroup(entry,): boolean {
    return entry.group
      .flags
      .every(function notNonBody(flag,): boolean {
        return !NON_BODY_FLAGS.has(flag,);
      },);
  },);
  return {
    entries: entries.length,
    package: packageSummary({ entries, },),
    maximalGroups: maximal.length,
    refrainGroups: refrain.length,
    pagesWithAnyGroup: new Set(all.map(function idOf(entry,) {
      return entry.id;
    },),).size,
    maximalSizeHistogram: sizeHistogram({ groups: maximal.map(function groupOf(entry,) {
      return entry.group;
    },), },),
    refrainSizeHistogram: sizeHistogram({ groups: refrain.map(function groupOf(entry,) {
      return entry.group;
    },), },),
    refrainCountsPerSetting: {
      main: entries.reduce(
        function sumMain(
          total,
          entry,
        ) {
          return total
            + entry.census
              .refrainCounts
              .main;
        },
        0,
      ),
      loose: entries.reduce(
        function sumLoose(
          total,
          entry,
        ) {
          return total
            + entry.census
              .refrainCounts
              .loose;
        },
        0,
      ),
      tight: entries.reduce(
        function sumTight(
          total,
          entry,
        ) {
          return total
            + entry.census
              .refrainCounts
              .tight;
        },
        0,
      ),
    },
    settings: REFRAIN_SETTINGS,
    bodyOnlyGroups: bodyOnly.length,
    bodyOnlyPages: new Set(bodyOnly.map(function idOf(entry,) {
      return entry.id;
    },),).size,
    bodyOnlyEntryIds: textsInCodePointOrder({ texts: [...new Set(bodyOnly.map(function idOf(entry,) {
      return entry.id;
    },),),], },),
    bodyOnlyDetail: bodyOnly.map(function detailOfBody(entry,) {
      return detailOf({ entry, },);
    },),
    sameLineDetail: all.filter(function sameLine(entry,) {
      return entry.group
        .flags
        .includes('same-line',);
    },)
      .map(function detailOfEntry(entry,) {
        return detailOf({ entry, },);
      },),
    sameLineGroups: all.filter(function sameLine(entry,) {
      return entry.group
        .flags
        .includes('same-line',);
    },)
      .length,
    groupsWithListOccurrence: all.filter(function listed(entry,) {
      return entry.group
        .flags
        .includes('list',);
    },)
      .length,
    control: lineControlOf({ entries, },),
  };
}

//endregion Repeat census summary
