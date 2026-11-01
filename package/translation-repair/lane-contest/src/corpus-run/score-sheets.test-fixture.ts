/**
 Graded sheets and manifests for the `score-*` runners' cases.

 The sheet is the shape a grader leaves: one heading per item, carrying the
 grade between its marker and its legend. The manifest is the file written
 beside the sheet, naming the set each position came from.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { computeDrawDigest, } from '../../dist/final/node/index.mjs';

/**
 Legend every item heading of a fixture sheet ends with.
 */
const LEGEND = '(Y = real damage, N = invented)';

/**
 One item of a fixture draw, as the digest and the manifest read it.
 */
type DrawItem = {
  readonly position: number;
  readonly entryId: string;
  readonly issueId: string;
};

/**
 The graded headings of a fixture sheet.

 @param grades - what the grader wrote after each heading's `grade:`, in order

 @returns One heading with its body per grade

 @example
 ```ts
 const headings = headingsOf({ grades: ['Y',], },);
 ```
 */
function headingsOf({ grades, }: { readonly grades: readonly string[]; },): readonly string[] {
  return grades.map(function toHeading(
    grade,
    index,
  ): string {
    return `### ${String(index + 1,)}. grade: ${grade}  ${LEGEND}\n\nThe cat sat.\n`;
  },);
}

/**
 The items of a fixture draw.

 @param count - items drawn

 @param issueIds - issue at each position, absent to name them after the position

 @returns One item per position

 @example
 ```ts
 const items = drawItemsOf({ count: 2, },);
 ```
 */
function drawItemsOf(
  {
    count,
    issueIds,
  }: {
    readonly count: number;
    readonly issueIds?: readonly string[];
  },
): readonly DrawItem[] {
  return Array.from(
    { length: count, },
    function toItem(
      _unused,
      index,
    ): DrawItem {
      return {
        position: index + 1,
        entryId: `cat${String(index,)}`,
        issueId: issueIds?.[index] ?? `adjudicated/nap${String(index,)}`,
      };
    },
  );
}

/**
 One graded detection sheet, with a heading per grade.

 @param grades - what the grader wrote after each heading's `grade:`, in order

 @returns Sheet text

 @example
 ```ts
 const text = catSheetText({ grades: ['Y', 'N',], },);
 ```
 */
export function catSheetText({ grades, }: { readonly grades: readonly string[]; },): string {
  return [
    '# Whisker sheet',
    '',
    ...headingsOf({ grades, },),
  ].join('\n',);
}

/**
 A manifest naming the set of each sheet position.

 @param kinds - set each position came from, in sheet order

 @returns Manifest text

 @example
 ```ts
 const text = catManifestText({ kinds: ['control', 'flagged',], },);
 ```
 */
export function catManifestText({ kinds, }: { readonly kinds: readonly string[]; },): string {
  return JSON.stringify({
    items: kinds.map(function toItem(
      kind,
      index,
    ): Readonly<Record<string, unknown>> {
      return {
        position: index + 1,
        entryId: `cat${String(index,)}`,
        kind,
      };
    },),
  },);
}

/**
 A graded detection sheet that declares the draw it belongs to.

 @param seed - draw seed the header declares, absent to leave the header bare

 @param corpusSha - corpus commit the header declares

 @param drawDigest - fingerprint the header declares, absent to leave it out

 @param grades - what the grader wrote after each heading's `grade:`, in order

 @returns Sheet text

 @example
 ```ts
 const text = catDrawSheetText({ seed: 'round-cats', corpusSha: 'abc1234', grades: ['Y',], },);
 ```
 */
export function catDrawSheetText(
  {
    seed,
    corpusSha,
    drawDigest,
    grades,
  }: {
    readonly seed?: string;
    readonly corpusSha: string;
    readonly drawDigest?: string;
    readonly grades: readonly string[];
  },
): string {
  return [
    '# Whisker draw sheet',
    ...((seed === undefined) ? [] : [`Draw seed: ${seed}`,]),
    `Corpus pin: ${corpusSha}`,
    ...((drawDigest === undefined) ? [] : [`Draw digest: ${drawDigest}`,]),
    '',
    ...headingsOf({ grades, },),
  ].join('\n',);
}

/**
 A sample manifest written before the draw digest existed.

 @param seed - draw seed

 @param corpusSha - corpus commit

 @param count - items drawn

 @param digested - whether the manifest carries the draw fingerprint

 @param issueIds - issue at each position, absent to name them after the position

 @returns Manifest text

 @example
 ```ts
 const text = catSampleManifestText({ seed: 'round-cats', corpusSha: 'abc1234', count: 2, digested: false, },);
 ```
 */
export function catSampleManifestText(
  {
    seed,
    corpusSha,
    count,
    digested,
    issueIds,
  }: {
    readonly seed: string;
    readonly corpusSha: string;
    readonly count: number;
    readonly digested: boolean;
    readonly issueIds?: readonly string[];
  },
): string {
  /**
   Fingerprint of the draw, which only a digested manifest carries.
   */
  const drawDigest = catDrawDigest({
    seed,
    corpusSha,
    count,
    ...((issueIds === undefined) ? {} : { issueIds, }),
  },);
  return JSON.stringify({
    seed,
    corpusSha,
    ...(digested ? { drawDigest, } : {}),
    items: drawItemsOf({
      count,
      ...((issueIds === undefined) ? {} : { issueIds, }),
    },),
  },);
}

/**
 Blind pre-grades, one per sheet position.

 @param verdicts - verdict recorded at each position, in order

 @returns Pre-grades text

 @example
 ```ts
 const text = catPreGradesText({ verdicts: ['real-defect', 'false-positive',], },);
 ```
 */
export function catPreGradesText({ verdicts, }: { readonly verdicts: readonly string[]; },): string {
  return JSON.stringify(verdicts.map(function toGrade(
    verdict,
    index,
  ): Readonly<Record<string, unknown>> {
    return {
      index: index + 1,
      verdict,
      note: '',
    };
  },),);
}

/**
 Fingerprint of a fixture draw, as the manifest and the sheet header carry it.

 @param seed - draw seed

 @param corpusSha - corpus commit

 @param count - items drawn

 @param issueIds - issue at each position, absent to name them after the position

 @returns Digest text

 @example
 ```ts
 const drawDigest = catDrawDigest({ seed: 'round-cats', corpusSha: 'abc1234', count: 2, },);
 ```
 */
export function catDrawDigest(
  {
    seed,
    corpusSha,
    count,
    issueIds,
  }: {
    readonly seed: string;
    readonly corpusSha: string;
    readonly count: number;
    readonly issueIds?: readonly string[];
  },
): string {
  return computeDrawDigest({
    seed,
    corpusSha,
    items: drawItemsOf({
      count,
      ...((issueIds === undefined) ? {} : { issueIds, }),
    },),
  },);
}

/**
 A graded repair sheet that declares the draw it belongs to.

 @param seed - draw seed the header declares

 @param corpusSha - corpus commit the header declares

 @param drawDigest - fingerprint the header declares, absent to leave it out

 @param grades - what the grader wrote after each item's `repair grade:`, in order

 @returns Sheet text

 @example
 ```ts
 const text = catRepairSheetText({ seed: 'round-cats', corpusSha: 'abc1234', grades: ['Y',], },);
 ```
 */
export function catRepairSheetText(
  {
    seed,
    corpusSha,
    drawDigest,
    grades,
  }: {
    readonly seed: string;
    readonly corpusSha: string;
    readonly drawDigest?: string;
    readonly grades: readonly string[];
  },
): string {
  return [
    '# Whisker repair sheet',
    `Draw seed: ${seed}`,
    `Corpus pin: ${corpusSha}`,
    ...((drawDigest === undefined) ? [] : [`Draw digest: ${drawDigest}`,]),
    '',
    ...grades.map(function toItem(
      grade,
      index,
    ): string {
      return `### ${String(index + 1,)}. The cat sat\n\n- repair grade: ${grade}  (Y = fixes, N = does not)\n`;
    },),
  ].join('\n',);
}
