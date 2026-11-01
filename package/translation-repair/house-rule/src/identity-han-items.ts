import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { isHanOnly, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Han items of a declared value
// LEDGER F-3 (shihai4h, 2026-09-26). The archive's front matter declares the
// aliases "小柿子, 猫小泪, u3" on both sides, so the identity line read
// `TRANSLATION declares "小柿子, 猫小泪, u3"` and presented two handles still
// in Han as the declared English, and both runs then wrote 小柿子 into
// English prose. A TRANSLATION value's items written in Han alone are named
// on the line as no English rendering, so a writer renders them by the handle
// rule rather than copying them. Read item by item, since the value that
// caused this mixes Han items with a Latin one.

/**
 Separators an alias list is written with: the ASCII comma, the full-width
 comma and the enumeration comma.
 */
const ITEM_SEPARATORS: ReadonlySet<string> = new Set([
  ',',
  '，',
  '、',
],);

/**
 Items of a declared value, split on its list separators and trimmed.

 @param value - declared value as written

 @returns Non-empty items in order

 @example
 ```ts
 declaredItems({ value: '小橘子, 猫猫、u3', },); // ['小橘子', '猫猫', 'u3']
 ```
 */
function declaredItems({ value, }: { readonly value: string; },): readonly string[] {
  // ONE PASS PER SEPARATOR, each splitting what the previous one left.
  return Array.from(ITEM_SEPARATORS,)
    .reduce<readonly string[]>(
    function splitOn(
      items,
      separator,
    ): readonly string[] {
      return items.flatMap(function pieces(item,): readonly string[] {
        return item.split(separator,);
      },);
    },
    [value,],
    )
    .map(function trimmed(item,): string {
      return item.trim();
    },)
    .filter(function nonEmpty(item,): boolean {
      return item !== '';
    },);
}

/**
 Clause naming the items of a TRANSLATION value still written in Han alone,
 empty when there are none.

 @param value - TRANSLATION side's declared value

 @returns Clause to append to the declaration line, with its leading space

 @example
 ```ts
 hanItemsClause({ value: '小橘子, u3', },); // ' (of which "小橘子" is still in Han, which is no English rendering)'
 ```
 */
export function hanItemsClause({ value, }: { readonly value: string; },): string {
  /**
   Items written in Han alone, quoted.
   */
  const hanItems = declaredItems({ value, },)
    .filter(function hanOnly(item,): boolean {
      return isHanOnly({ text: item, },);
    },)
    .map(function quoted(item,): string {
      return `"${item}"`;
    },);
  if (hanItems.length === 0)
    return '';
  /**
   Verb agreeing with the number of items named.
   */
  const verb = wordForCount({
    count: hanItems.length,
    one: 'is',
    many: 'are',
  },);
  return ` (of which ${hanItems.join(' and ',)} ${verb} still in Han, which is no English rendering)`;
}

//endregion Han items of a declared value
