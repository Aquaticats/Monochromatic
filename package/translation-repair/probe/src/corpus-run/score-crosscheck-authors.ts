import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { CrosscheckItem, } from './judge-crosscheck.ts';
import { MIN_JUDGED_CLAIMS, } from './judge-independence.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Score crosscheck authors
// The author table `score-crosscheck` prints: each critic's share of the
// accepted and control arms, and whether each arm can carry a rate.

/**
 Column widths of the author table, wide enough that no row runs into its
 neighbour. Object-literal values, so the numbers stay readable here rather
 than becoming named constants nobody can picture.
 */
const COLUMN = {
  model: 52,
  accepted: 10,
  control: 9,
  sole: 6,
} as const;

/**
 One author's share of each arm.

 @example
 ```ts
 const row: AuthorRow = { modelId: 'hf:cat/Tabby-1', accepted: 3, control: 1, sole: 2, };
 ```
 */
export type AuthorRow = {
  /**
   Model that proposed the claims.
   */
  readonly modelId: string;

  /**
   Claims of theirs the panel accepted.
   */
  readonly accepted: number;

  /**
   Claims of theirs the panel did not accept.
   */
  readonly control: number;

  /**
   Claims they authored alone, across both arms.
   */
  readonly sole: number;
};

/**
 One author's counts so far, raised as each claim they proposed is read.
 */
type AuthorCounts = {
  accepted: number;
  control: number;
  sole: number;
};

/**
 Tallies each author's claims per arm.

 Counts a claim once for EVERY author, not once per claim. The question the
 crosscheck asks is per author, so a claim two critics proposed belongs to
 both their populations; summing the column therefore exceeds the claim count
 whenever critics agreed, which on this run they almost never do.

 @param items - judgeable claims from the census

 @returns One row per author, most accepted claims first, authors with as many
 accepted claims as each other in code-point order of model id, so the order
 never depends on which artifact the directory listed first

 @example
 ```ts
 const rows = tallyAuthors({ items, },);
 ```
 */
export function tallyAuthors(
  { items, }: { readonly items: readonly CrosscheckItem[]; },
): readonly AuthorRow[] {
  /**
   Running counts keyed by model id.
   */
  const rows = new Map<string, AuthorCounts>();

  for (
    const {
      arm,
      proposers,
    } of items
  ) {
    /**
     Whether one author raised this claim alone, computed once per claim
     rather than once per author of it.
     */
    const soleAuthored = proposers.length === 1;
    for (const modelId of proposers) {
      /**
       This author's row, created on first sight.
       */
      const row = rows.get(modelId,) ?? {
        accepted: 0,
        control: 0,
        sole: 0,
      };
      // Only the two arms a rate is computed over are counted here. An
      // `undecided` claim belongs to neither, and adding it to control would
      // put the panel's non-verdicts into a column read as rejections.
      if (arm === 'accepted')
        row.accepted += 1;
      else if (arm === 'control')
        row.control += 1;
      if (soleAuthored)
        row.sole += 1;
      rows.set(
        modelId,
        row,
      );
    }
  }

  return [...rows.entries(),]
    .map(function toRow([modelId, counts,]: ReadonlyEntry<string, Readonly<AuthorCounts>>,): AuthorRow {
      return {
        modelId,
        accepted: counts.accepted,
        control: counts.control,
        sole: counts.sole,
      };
    },)
    .toSorted(function byAccepted(
      left,
      right,
    ): number {
      return (right.accepted - left.accepted) || compareCodePoints({
        left: left.modelId,
        right: right.modelId,
      },);
    },);
}

/**
 Renders the author table header.

 @returns Header line

 @example
 ```ts
 console.log(authorHeaderLine(),);
 ```
 */
export function authorHeaderLine(): string {
  return [
    'AUTHOR'
      .padEnd(COLUMN.model,),
    'accepted'
      .padStart(COLUMN.accepted,),
    'control'
      .padStart(COLUMN.control,),
    'sole'
      .padStart(COLUMN.sole,),
    '  floor',
  ].join('',);
}

/**
 Renders one author's row, saying plainly whether each arm can carry a rate.

 @param row - one author's counts

 @returns Row line

 @example
 ```ts
 console.log(authorLine({ row, },),);
 ```
 */
export function authorLine({ row, }: { readonly row: AuthorRow; },): string {
  /**
   Which arms hold enough claims for a rate to be reported over them.

   Both arms must clear it independently. A crosscheck reports the GAP
   between them, and a gap is only as trustworthy as its thinner side.
   */
  const clears = [
    (row.accepted >= MIN_JUDGED_CLAIMS) ? 'accepted' : '',
    (row.control >= MIN_JUDGED_CLAIMS) ? 'control' : '',
  ].filter(function isSet(name,): boolean {
    return name !== '';
  },);

  return [
    row.modelId
      .padEnd(COLUMN.model,),
    String(row.accepted,)
      .padStart(COLUMN.accepted,),
    String(row.control,)
      .padStart(COLUMN.control,),
    String(row.sole,)
      .padStart(COLUMN.sole,),
    `  ${(clears.length === 0) ? 'neither' : clears.join('+',)}`,
  ].join('',);
}

//endregion Score crosscheck authors
