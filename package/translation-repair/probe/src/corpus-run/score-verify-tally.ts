import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { GradedItem, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { VerifyManifestItem, } from './score-verify-manifest.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Score verify tally
// Joins the graded items to the manifest rows by position, counts the grades
// per set, and renders one report line per set.

/**
 Width the set name is padded to, so the two report lines align.
 */
const KIND_COLUMN_WIDTH = 8;

/**
 Decimal places a precision figure carries.
 */
const PRECISION_DIGITS = 3;

/**
 Counts of one set's graded flags.

 @example
 ```ts
 const tally: KindTally = { damage: 2, invented: 1, unscored: 0, };
 ```
 */
export type KindTally = {
  /**
   Flags the reader called real damage.
   */
  damage: number;

  /**
   Flags the reader rejected.
   */
  invented: number;

  /**
   Items left ungraded.
   */
  unscored: number;
};

/**
 Adds one graded item to its set's tally.

 @param tally - tally to add into

 @param item - graded sheet item

 @example
 ```ts
 addGrade({ tally, item, },);
 ```
 */
function addGrade(
  {
    tally,
    item,
  }: {
    readonly tally: KindTally;
    readonly item: GradedItem;
  },
): void {
  if (item.verdict === 'real-defect') {
    tally.damage += 1;
    return;
  }
  if (item.verdict === 'false-positive') {
    tally.invented += 1;
    return;
  }
  tally.unscored += 1;
}

/**
 Counts the graded flags of each set the manifest names.

 Built from the labels the manifest actually carries, because the two sheets
 partition on different things. A fixed pair of keys silently dropped every
 item whose label was not one of them, which would have scored a whole sheet as
 empty while reporting success.

 @param graded - graded sheet items, in sheet order

 @param manifest - manifest rows, in the same order

 @returns One tally per set label, in order of first appearance

 @throws {@link StatedRefusalError} when the sheet and the manifest differ in length, since a
 positional join between them would mislabel verdicts

 @example
 ```ts
 const tallies = tallyByKind({ graded, manifest, },);
 ```
 */
export function tallyByKind(
  {
    graded,
    manifest,
  }: {
    readonly graded: readonly GradedItem[];
    readonly manifest: readonly VerifyManifestItem[];
  },
): ReadonlyMap<string, KindTally> {
  if (graded.length !== manifest.length) {
    throw new StatedRefusalError({
      says: `sheet carries ${String(graded.length,)} ${
        wordForCount({
          count: graded.length,
          one: 'item',
          many: 'items',
        },)
      } and manifest carries ${
        String(manifest.length,)
      }; a positional join between them would mislabel verdicts, so neither file describes the other`,
    },);
  }

  /**
   Tally per label, built from the labels the manifest actually carries.
   */
  const tallies = new Map<string, KindTally>();
  for (const [index, item,] of graded.entries()) {
    /**
     Manifest row for this position.
     */
    const row = manifest[index];
    if (row === undefined) {
      throw new Error(
        'unreachable: a sheet position has no manifest row, though the sheet and the manifest were checked to be of one length',
      );
    }

    /**
     Tally this row belongs to, created on first sight of its label.
     */
    const tally = tallies.get(row.kind,) ?? {
      damage: 0,
      invented: 0,
      unscored: 0,
    };
    addGrade({
      tally,
      item,
    },);
    tallies.set(
      row.kind,
      tally,
    );
  }

  return tallies;
}

/**
 Renders the report lines of the sets, ordered by label.

 @param tallies - tally per set label

 @returns One line per set, in code-point order of the labels

 @example
 ```ts
 for (const line of verifyKindLines({ tallies, },))
   console.log(line,);
 ```
 */
export function verifyKindLines({ tallies, }: { readonly tallies: ReadonlyMap<string, Readonly<KindTally>>; },): readonly string[] {
  return [...tallies,]
    .toSorted(function byLabel(
      left: ReadonlyEntry<string, Readonly<KindTally>>,
      right: ReadonlyEntry<string, Readonly<KindTally>>,
    ): number {
      return compareCodePoints({
        left: left[0],
        right: right[0],
      },);
    },)
    .map(function toLine([kind, tally,]: ReadonlyEntry<string, Readonly<KindTally>>,): string {
      /**
       Flags this set contributed that carry a verdict.
       */
      const scored = tally.damage + tally.invented;
      return `${kind.padEnd(KIND_COLUMN_WIDTH,)} flags=${
        String(scored + tally.unscored,)
      } realDamage=${String(tally.damage,)} invented=${
        String(tally.invented,)
      } unscored=${String(tally.unscored,)} precision=${
        scored === 0 ? 'n/a' : (tally.damage / scored).toFixed(PRECISION_DIGITS,)
      }`;
    },);
}

//endregion Score verify tally
