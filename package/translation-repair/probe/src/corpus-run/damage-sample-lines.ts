import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Damage sample lines
// The sentences the damage command says about its pool and its sheet, built
// from counts alone so each noun is shown in the form its count asks for.

/**
 Says how many rows filled a passage the archive had no wording for, as a
 clause: the rows are counted, never drawn from.

 @param filledWithoutIncumbent - shipped rows with no incumbent wording

 @returns The clause, with its verb in agreement

 @example
 ```ts
 const clause = filledClause({ filledWithoutIncumbent: 2, },);
 ```
 */
function filledClause(
  { filledWithoutIncumbent, }: { readonly filledWithoutIncumbent: number; },
): string {
  return `${String(filledWithoutIncumbent,)} shipped ${
    wordForCount({
      count: filledWithoutIncumbent,
      one: 'row',
      many: 'rows',
    },)
  } had no incumbent wording and ${
    wordForCount({
      count: filledWithoutIncumbent,
      one: 'is',
      many: 'are',
    },)
  } not drawn from`;
}

/**
 The two lines a draw prints about its pool.

 @param regionCount - shipped regions across both lanes

 @param filledWithoutIncumbent - shipped rows with no incumbent wording

 @param seed - draw seed

 @returns The pool line and the line about the rows left out of it

 @example
 ```ts
 const lines = poolLines({ regionCount: 1, filledWithoutIncumbent: 0, seed: 'damage-round-one', },);
 ```
 */
export function poolLines(
  {
    regionCount,
    filledWithoutIncumbent,
    seed,
  }: {
    readonly regionCount: number;
    readonly filledWithoutIncumbent: number;
    readonly seed: string;
  },
): readonly string[] {
  return [
    `DAMAGE pool ${String(regionCount,)} shipped ${
      wordForCount({
        count: regionCount,
        one: 'region',
        many: 'regions',
      },)
    } across both lanes, seed ${seed}`,
    // Reported rather than dropped: a slice filled where the archive had no
    // English replaced nothing, so no edit could have damaged anything there,
    // and the honest question about it belongs on a different sheet.
    `DAMAGE ${filledClause({ filledWithoutIncumbent, },)}`,
  ];
}

/**
 The sentence a draw refuses with when its pool holds no region.

 @param filledWithoutIncumbent - shipped rows with no incumbent wording

 @returns The refusal's words

 @example
 ```ts
 const says = emptyPoolSays({ filledWithoutIncumbent: 0, },);
 ```
 */
export function emptyPoolSays(
  { filledWithoutIncumbent, }: { readonly filledWithoutIncumbent: number; },
): string {
  return 'the settled entries ship no replacement over an archive wording to draw from '
    + `(${filledClause({ filledWithoutIncumbent, },)}), so no sheet is written; a sheet with no item would be `
    + 'kept and refuse the next run';
}

/**
 The line saying how many items the sheet was written with.

 @param items - items on the sheet

 @param runsDir - directory the sheet landed in

 @returns The closing line

 @example
 ```ts
 const line = wroteLine({ items: 2, runsDir: '/runs', },);
 ```
 */
export function wroteLine(
  {
    items,
    runsDir,
  }: {
    readonly items: number;
    readonly runsDir: string;
  },
): string {
  return `DAMAGE wrote ${String(items,)} ${
    wordForCount({
      count: items,
      one: 'item',
      many: 'items',
    },)
  } to ${runsDir}/damage-sheet.md`;
}

//endregion Damage sample lines
