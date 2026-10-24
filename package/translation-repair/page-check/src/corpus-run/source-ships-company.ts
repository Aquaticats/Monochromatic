import { declaredNamesOf, } from './source-ships-declared.ts';
import { leavesFlowOpen, } from './source-ships-encoding.ts';
import { holdsIndentedFenceMark, } from './source-ships-fence.ts';
import { runTextOf, } from './source-ships-runs.ts';
import type { WouldShipSlice, } from './would-ship-text.ts';

//region Source ships company
// THE RUNS THAT PARSE ALONE AND CANNOT STAND BESIDE ANOTHER. The first step of the composition writes as literal text
// the runs that do not parse as a text of their own. Three more kinds of run parse alone and not in the page, whatever
// the English between them says, and the step takes them too, so that the second step, which writes every run as
// literal text, is not what makes the page parse:
//
// - A run that ends inside a fence or display math, which the page does not end where the run does: the opening line
//   of the next run closes it, and the text of that run after the line is outside it, where a brace or a tag in it is
//   code (`source-ships-open-flow.ts`). Written as literal text, the run is closed at its end.
// - A run with a fence mark indented four spaces or more, which the plain reading takes for text and the strict grammar
//   for a fence that swallows the page up to the mark that closes it (`source-ships-fence.ts`).
// - A run that declares a name an earlier run still declares as it was written, since a page declares a name once
//   (`source-ships-declared.ts`). The first run keeps its module; the runs after it are written as literal text.

/**
 Runs that cannot stand beside the others as they are written.

 @param runs - slice indices of each run, in page order

 @param slices - every slice's reading, in comparison-row order

 @returns Runs holding an indented fence mark, the runs redeclaring a name and the runs ending inside a fence or display
 math, in page order

 @example
 ```ts
 const unfit = runsUnfitForCompany({ runs, slices, },);
 ```
 */
export function runsUnfitForCompany(
  {
    runs,
    slices,
  }: {
    readonly runs: readonly (readonly number[])[];
    readonly slices: readonly WouldShipSlice[];
  },
): readonly (readonly number[])[] {
  /**
   Names the runs kept as written declare.
   */
  const declared = new Set<string>();
  return runs.filter(function isUnfit(run,): boolean {
    /**
     Text the run puts on the page.
     */
    const text = runTextOf({
      run,
      slices,
    },);
    /**
     Names the run declares.
     */
    const names = declaredNamesOf({ text, },);
    /**
     Whether a name of the run is declared by a run before it.
     */
    const redeclares = names.some(function seen(name,): boolean {
      return declared.has(name,);
    },);
    if (redeclares || holdsIndentedFenceMark({ text, },))
      return true;
    if (leavesFlowOpen({ text, },))
      return true;
    for (const name of names)
      declared.add(name,);
    return false;
  },);
}

//endregion Source ships company
