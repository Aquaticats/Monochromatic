/**
 Holds the footnote relabel closure to the one property its callers rely on:
 no two archive labels end on one label.

 LEDGER T8, EIGHTEENTH BATCH: `closeFootnoteRelabel` checked its own result
 for two archive labels merging and refused it, a branch no input reached,
 since the closure is injective by construction (its TSDoc states why). The
 check went, and this file reads the property instead, over every map a
 small universe allows: every nonempty set of archive labels, every nonempty
 set of original labels, and every set of relations between them, each
 closed result's destinations read for a repeat.

 It also counts the closed results that took the forced pair and those that
 displaced a label to a fresh one, so a change that stopped reaching either
 construction would fail here rather than leave the property read over the
 plain relations alone.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { closeFootnoteRelabel, } from '../dist/final/node/index.mjs';

/**
 Labels both namespaces draw from. Digits, which the parser's identifier
 normalization leaves as they are, so a destination is read by its spelling.
 */
const LABELS: readonly string[] = [
  '1',
  '2',
  '3',
];

/**
 One label relation, as the closure takes and returns it.
 */
type Relation = {
  readonly from: string;
  readonly to: string;
};

/**
 Every subset of a list, each in the list's own order, the empty one first.

 @param items - list to draw from

 @returns Each subset once

 @example
 ```ts
 subsetsOf({ items: ['1', '2',], },); // [[], ['1'], ['2'], ['1', '2']]
 ```
 */
function subsetsOf<const ItemT,>({ items, }: { readonly items: readonly ItemT[]; },): readonly (readonly ItemT[])[] {
  /**
   Subsets so far, each later one the earlier extended by one item.
   */
  const subsets: (readonly ItemT[])[] = [[],];
  for (const item of items) {
    /**
     The subsets so far, each extended by this item.
     */
    const extended = subsets.map(function withItem(subset,): readonly ItemT[] {
      return [
        ...subset,
        item,
      ];
    },);
    subsets.push(...extended,);
  }
  return subsets;
}

/**
 Archive labels a closed map sends to one label, said once per merge.

 @param archiveLabels - labels the archive carries

 @param map - closed rewrites

 @returns One line per destination two labels share, none when every
 destination is distinct

 @example
 ```ts
 mergesOf({ archiveLabels: ['1', '2',], map: [{ from: '1', to: '2', },], },); // ['2 <- 1, 2']
 ```
 */
function mergesOf(
  {
    archiveLabels,
    map,
  }: {
    readonly archiveLabels: readonly string[];
    readonly map: readonly Relation[];
  },
): readonly string[] {
  /**
   Where each moved label goes.
   */
  const moves = new Map(map.map(function entryOf(relation,): readonly [
    string,
    string,
  ] {
    return [
      relation.from,
      relation.to,
    ];
  },),);
  /**
   Archive labels by the label each ends on.
   */
  const landings = Map.groupBy(archiveLabels, function destinationOf(label,): string {
    return moves.get(label,) ?? label;
  },);
  return [...landings.entries(),]
    .filter(function merged([, labels,],): boolean {
      return labels.length > 1;
    },)
    .map(function said([destination, labels,],): string {
      return `${destination} <- ${labels.join(', ',)}`;
    },);
}

await describe({
  name: `${closeFootnoteRelabel.name} over every small universe`,
  children: [
    it({
      name: 'SENDS NO TWO ARCHIVE LABELS TO ONE in any closed result, the forced pair and the fresh displacement '
        + 'among them, which a check inside the closure once refused and no input reached',
      fn: async () => {
        /**
         Nonempty label sets a namespace can hold.
         */
        const namespaces = subsetsOf({ items: LABELS, },)
          .filter(function nonempty(labels,): boolean {
            return labels.length > 0;
          },);
        /**
         Every closed result that merged labels, said with its input.
         */
        const merges: string[] = [];
        /**
         Closed results, those taking the forced pair, and those displacing a
         label.
         */
        const reached = {
          closed: 0,
          eliminated: 0,
          retained: 0,
        };
        for (const archiveLabels of namespaces) {
          for (const originalLabels of namespaces) {
            /**
             Every relation the two namespaces allow.
             */
            const relations = archiveLabels.flatMap(function relationsFrom(from,): readonly Relation[] {
              return originalLabels.map(function relationTo(to,): Relation {
                return {
                  from,
                  to,
                };
              },);
            },);
            for (const map of subsetsOf({ items: relations, },)) {
              /**
               The closure of this map.
               */
              const closure = closeFootnoteRelabel({
                map,
                archiveLabels,
                originalLabels,
              },);
              if (closure.kind !== 'closed')
                continue;
              reached.closed += 1;
              reached.eliminated += (closure.eliminated.length > 0) ? 1 : 0;
              reached.retained += (closure.retained.length > 0) ? 1 : 0;
              for (const merge of mergesOf({
                archiveLabels,
                map: closure.map,
              },)) {
                merges.push(
                  `archive [${archiveLabels.join(',',)}] original [${originalLabels.join(',',)}] map ${
                    JSON.stringify(map,)
                  }: ${merge}`,
                );
              }
            }
          }
        }
        expect(merges,).toEqual([],);
        expect({
          closed: reached.closed > 0,
          eliminated: reached.eliminated > 0,
          retained: reached.retained > 0,
        },).toEqual({
          closed: true,
          eliminated: true,
          retained: true,
        },);
      },
    },),
  ],
},);
