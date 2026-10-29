/**
 TypeScript snippets from upstream docs that do not type-check against the
 published declarations, each pinned with `@ts-expect-error` so `lint:types`
 turns red when the declarations start accepting the documented code.

 Each snippet also type-checks with the same errors on TypeScript 4.7 through
 7.0 (`doc/audit/deepmerge-ts-surface-2026-09-24.md`, "Docs conformance").
 The runtime calls still run, so a runtime change shows up here too.

 Checked against deepmerge-ts 8.0.2 (docs at upstream `17fc99cb`), 2026-09-24.

 @module
 */

import type {
  DeepMergeBuiltInMetaData,
  DeepMergeFunctionsDefaultURIs,
  DeepMergeHKT,
  DeepMergeLeaf,
  DeepMergeLeafURI,
} from 'deepmerge-ts';

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

await describe({
  name: 'deepmerge-ts documentation snippets rejected by the declarations',
  children: [
    it({
      name: 'docs error: deepmergeCustom.md writes DeepMergeLeaf<Ts>, but the declaration requires three type arguments',
      // Dates example and "Customizing the Meta Data" example, each inside `declare module`.
      fn: async () => {
        // @ts-expect-error -- TS2314: Generic type 'DeepMergeLeaf' requires 3 type argument(s), as the docs write it.
        expectTypeOf<DeepMergeLeaf<[1,]>>().not.toBeNever();
        expectTypeOf<DeepMergeLeaf<[1, 2,], DeepMergeFunctionsDefaultURIs, undefined>>().toEqualTypeOf<2>();
      },
    },),
    it({
      name: 'docs error: the key-path metaDataUpdater example is rejected because mergeInfo.key is unknown, not PropertyKey',
      // deepmergeCustom.md "Customizing the Meta Data" declares `keyPath: ReadonlyArray<PropertyKey>`;
      // DeepMergeMergeInfo.key is `unknown`, so `[mergeInfo.key]` is `({} | null)[]` (TS2322).
      fn: async () => {
        /**
         The documented function, verbatim apart from formatting.
         */
        const byPath = target.deepmergeCustom<unknown, { DeepMergeOthersURI: DeepMergeLeafURI; }, { keyPath: readonly PropertyKey[]; }>({
          // @ts-expect-error -- TS2322: the updater returns `{ keyPath: ({} | null)[] }`.
          metaDataUpdater: function keyPath(previousMeta, mergeInfo,) {
            if (previousMeta === undefined) {
              if (mergeInfo.key === undefined)
                return { keyPath: [], };
              return { keyPath: [mergeInfo.key,], };
            }
            if (mergeInfo.key === undefined)
              return previousMeta;
            return { ...previousMeta, keyPath: [...previousMeta.keyPath, mergeInfo.key,], };
          },
        },);
        expect(byPath({ a: { b: 1, }, }, { a: { c: 2, }, },),).toEqual({ a: { b: 1, c: 2, }, },);
      },
    },),
    it({
      name: 'docs error: the TSDoc metaDataUpdater example is rejected without a metadata type argument',
      // src/types/options.ts MetaDataUpdater @example calls deepmergeCustom({ metaDataUpdater }) with no
      // type arguments, so M is DeepMergeBuiltInMetaData and `path` exists on neither side.
      fn: async () => {
        /* oxlint-disable typescript/no-unsafe-assignment -- the snippet spreads `previousMeta?.path`, error-typed by the pinned TS2339. */
        /**
         The TSDoc example's call.
         */
        const byPath = target.deepmergeCustom({
          // @ts-expect-error -- TS2769: `{ path }` shares no property with DeepMergeBuiltInMetaData.
          metaDataUpdater: (previousMeta, mergeInfo,) => ({
            // @ts-expect-error -- TS2339: Property 'path' does not exist on DeepMergeBuiltInMetaData.
            path: [...(previousMeta?.path ?? []), mergeInfo.key,],
          }),
        },);
        /* oxlint-enable typescript/no-unsafe-assignment */
        expect(byPath({ a: { b: 1, }, }, { a: { c: 2, }, },),).toEqual({ a: { b: 1, c: 2, }, },);
      },
    },),
    it({
      name: 'docs error: the deepmergeIntoCustom assertion example omits DeepMergeFilterValuesURI, which DeepMergeFunctionsURIs requires',
      // deepmergeCustom.md "Deepmerge Into Custom", CustomizedDeepmergeInto (TS2344 on 6.0, TS2741 on 7.0).
      fn: async () => {
        expectTypeOf<DeepMergeHKT<
          [{ a: 1; },],
          // @ts-expect-error -- Property 'DeepMergeFilterValuesURI' is missing, as in the documented type.
          {
            DeepMergeArraysURI: DeepMergeFunctionsDefaultURIs['DeepMergeArraysURI'];
            DeepMergeMapsURI: DeepMergeFunctionsDefaultURIs['DeepMergeMapsURI'];
            DeepMergeOthersURI: DeepMergeFunctionsDefaultURIs['DeepMergeOthersURI'];
            DeepMergeRecordsURI: DeepMergeFunctionsDefaultURIs['DeepMergeRecordsURI'];
            DeepMergeSetsURI: DeepMergeFunctionsDefaultURIs['DeepMergeSetsURI'];
          },
          DeepMergeBuiltInMetaData
          >>().toBeNever();
      },
    },),
  ],
},);
