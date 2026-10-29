/**
 Type-level divergence found while building the historical-recall controls
 (`doc/audit/deepmerge-ts-recall-2026-09-24.md`): with a non-default filter
 URI, deepmerge-ts turns a leaf union into a tuple one member at a time, and
 a union of 50 members already exceeds TypeScript's instantiation depth
 (TS2589, TypeScript 6.0.2 and 7.0.2). The property then types as `any`.

 The `@ts-expect-error` pins the diagnostic, so `lint:types` fails (TS2578)
 when upstream fixes it; the runtime value is pinned beside it.

 Upstream reporting: the combined issue draft, held locally for the user.

 @module
 */

import type { DeepMergeNoFilteringURI, } from 'deepmerge-ts';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

/**
 A 50-member string-literal union, such as a status or country code.
 */
type Status =
  | 's0' | 's1' | 's2' | 's3' | 's4' | 's5' | 's6' | 's7' | 's8' | 's9'
    | 's10' | 's11' | 's12' | 's13' | 's14' | 's15' | 's16' | 's17' | 's18' | 's19'
    | 's20' | 's21' | 's22' | 's23' | 's24' | 's25' | 's26' | 's27' | 's28' | 's29'
    | 's30' | 's31' | 's32' | 's33' | 's34' | 's35' | 's36' | 's37' | 's38' | 's39'
    | 's40' | 's41' | 's42' | 's43' | 's44' | 's45' | 's46' | 's47' | 's48' | 's49';

await describe({
  name: 'deepmerge-ts union depth divergence found by the recall run',
  children: [
    it({
      name: 'filterValues false with DeepMergeNoFilteringURI exceeds instantiation depth on a 50-member union',
      fn: async () => {
        /**
         Merge function with the documented no-filtering URI.
         */
        const noFilter = target.deepmergeCustom<unknown, { DeepMergeFilterValuesURI: DeepMergeNoFilteringURI; }>({ filterValues: false, },);
        /**
         First record.
         */
        const first: { status: Status; } = { status: 's0', };
        /**
         Second record.
         */
        const second: { status: Status; } = { status: 's1', };
        /**
         Merged record; its type is where TS2589 surfaces.
         */
        const merged = noFilter(
          first,
          second,
        );
        // @ts-expect-error -- TS2589: type instantiation is excessively deep; `merged.status` types as `any`.
        const typed: { status: Status; } = merged;
        expect(typed.status,).toBe('s1',);
      },
    },),
  ],
},);
