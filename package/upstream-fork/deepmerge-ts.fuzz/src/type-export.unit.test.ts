/**
 Type-level tests that reference every type deepmerge-ts exports, each with
 its defaults and with concrete arguments.

 The historical-recall audit (`doc/audit/deepmerge-ts-recall-2026-09-24.md`,
 row `944b428` / `#304`) found the sidecar referenced only two exported
 types, so a declaration that broke an exported type's defaults went
 unnoticed. `./type-export-name.ts` lists the names these tests cover, and
 `./type-export-name.unit.test.ts` compares that list with the declaration
 file.

 @module
 */

import type {
  DeepMergeArraysDefaultHKT,
  DeepMergeBuiltInMetaData,
  DeepMergeCircularReferencesDefaultHKT,
  DeepMergeFastUnsafeOptions,
  DeepMergeFastUnsafeUtils,
  DeepMergeFilterValuesDefaultHKT,
  DeepMergeFunctionsDefaults,
  DeepMergeFunctionsDefaultsFastUnsafe,
  DeepMergeFunctionsDefaultURIs,
  DeepMergeFunctionsURIs,
  DeepMergeFunctionURItoKind,
  DeepMergeHKT,
  DeepMergeIntoFastUnsafeOptions,
  DeepMergeIntoFastUnsafeUtils,
  DeepMergeIntoFunctionsDefaults,
  DeepMergeIntoFunctionsDefaultsFastUnsafe,
  DeepMergeIntoOptions,
  DeepMergeIntoUtils,
  DeepMergeLeaf,
  DeepMergeLeafURI,
  DeepMergeMapsDefaultHKT,
  DeepMergeMergeInfo,
  DeepMergeMetaData,
  DeepMergeNoFilteringURI,
  DeepMergeOptions,
  DeepMergeRecordsDefaultHKT,
  DeepMergeSetsDefaultHKT,
  DeepMergeUtils,
  DeepMergeValueReference,
  FilterOut,
  GetDeepMergeFunctionsURIs,
} from 'deepmerge-ts';

import {
  describe,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Default URIs with filtering turned off, for the leaf and filter types.
 */
type NoFilterURIs = GetDeepMergeFunctionsURIs<{ DeepMergeFilterValuesURI: DeepMergeNoFilteringURI; }>;

await describe({
  name: 'deepmerge-ts exported types',
  children: [
    it({
      name: 'option, utility, and metadata types accept their defaults',
      fn: async () => {
        expectTypeOf<DeepMergeOptions>().not.toBeAny();
        expectTypeOf<DeepMergeOptions>().toHaveProperty('mergeArrays',);
        expectTypeOf<DeepMergeIntoOptions>().toHaveProperty('mergeArrays',);
        expectTypeOf<DeepMergeFastUnsafeOptions>().not.toHaveProperty('metaDataUpdater',);
        expectTypeOf<DeepMergeIntoFastUnsafeOptions>().not.toHaveProperty('maxDepth',);
        expectTypeOf<DeepMergeUtils['actions']>().toEqualTypeOf<Readonly<{ defaultMerge: symbol; skip: symbol; }>>();
        expectTypeOf<DeepMergeIntoUtils['actions']>().toEqualTypeOf<Readonly<{ defaultMerge: symbol; }>>();
        expectTypeOf<DeepMergeFastUnsafeUtils>().toEqualTypeOf<DeepMergeUtils<undefined>>();
        expectTypeOf<DeepMergeIntoFastUnsafeUtils>().toEqualTypeOf<DeepMergeIntoUtils<undefined>>();
        expectTypeOf<DeepMergeMetaData>().toBeUnknown();
        expectTypeOf<undefined>().toExtend<DeepMergeBuiltInMetaData>();
        expectTypeOf<DeepMergeMergeInfo>().toHaveProperty('key',);
        expectTypeOf<DeepMergeValueReference<number>>().toEqualTypeOf<{ value: number; }>();
      },
    },),
    it({
      name: 'default merge function tables expose every documented member',
      fn: async () => {
        expectTypeOf<keyof DeepMergeFunctionsDefaults>().toEqualTypeOf<
          'mergeArrays' | 'mergeCircularReferences' | 'mergeMaps' | 'mergeOthers' | 'mergeRecords' | 'mergeSets'
          >();
        expectTypeOf<keyof DeepMergeIntoFunctionsDefaults>().toEqualTypeOf<
          'mergeArrays' | 'mergeCircularReferences' | 'mergeMaps' | 'mergeOthers' | 'mergeRecords' | 'mergeSets'
          >();
        expectTypeOf<keyof DeepMergeFunctionsDefaultsFastUnsafe>().toEqualTypeOf<
          'mergeArrays' | 'mergeMaps' | 'mergeOthers' | 'mergeRecords' | 'mergeSets'
          >();
        expectTypeOf<keyof DeepMergeIntoFunctionsDefaultsFastUnsafe>().toEqualTypeOf<
          'mergeArrays' | 'mergeMaps' | 'mergeOthers' | 'mergeRecords' | 'mergeSets'
          >();
      },
    },),
    it({
      name: 'URI tables resolve the defaults and an override',
      fn: async () => {
        expectTypeOf<GetDeepMergeFunctionsURIs<{ DeepMergeOthersURI: DeepMergeLeafURI; }>>().toEqualTypeOf<DeepMergeFunctionsDefaultURIs>();
        expectTypeOf<NoFilterURIs['DeepMergeFilterValuesURI']>().toEqualTypeOf<DeepMergeNoFilteringURI>();
        expectTypeOf<NoFilterURIs['DeepMergeOthersURI']>().toEqualTypeOf<DeepMergeLeafURI>();
        expectTypeOf<DeepMergeFunctionsDefaultURIs>().toExtend<DeepMergeFunctionsURIs>();
        expectTypeOf<DeepMergeFunctionURItoKind<[1, 2,], DeepMergeFunctionsDefaultURIs, undefined>['DeepMergeLeafURI']>()
          .toEqualTypeOf<2>();
      },
    },),
    it({
      name: 'default kind types compute the documented results',
      fn: async () => {
        expectTypeOf<DeepMergeHKT<[{ a: 1; }, { b: 2; },], DeepMergeFunctionsDefaultURIs, undefined>>()
          .toEqualTypeOf<{ a: 1; b: 2; }>();
        expectTypeOf<DeepMergeRecordsDefaultHKT<[{ a: [1,]; }, { a: [2,]; },], DeepMergeFunctionsDefaultURIs, undefined>>()
          .toEqualTypeOf<{ a: [1, 2,]; }>();
        expectTypeOf<DeepMergeArraysDefaultHKT<[[1,], [2,],], DeepMergeFunctionsDefaultURIs, undefined>>()
          .toEqualTypeOf<[1, 2,]>();
        expectTypeOf<DeepMergeSetsDefaultHKT<[Set<1>, Set<2>,]>>().toEqualTypeOf<Set<1 | 2>>();
        expectTypeOf<DeepMergeMapsDefaultHKT<[Map<'a', 1>, Map<'b', 2>,]>>().toEqualTypeOf<Map<'a' | 'b', 1 | 2>>();
        expectTypeOf<DeepMergeLeaf<[1, undefined,], DeepMergeFunctionsDefaultURIs, undefined>>().toEqualTypeOf<1>();
        expectTypeOf<DeepMergeLeaf<[1, undefined,], NoFilterURIs, undefined>>().toEqualTypeOf<undefined>();
        expectTypeOf<DeepMergeCircularReferencesDefaultHKT<[1, 2,], DeepMergeFunctionsDefaultURIs, undefined>>()
          .toEqualTypeOf<2>();
        expectTypeOf<DeepMergeFilterValuesDefaultHKT<[1, undefined, 2,]>>().toEqualTypeOf<[1, 2,]>();
        expectTypeOf<FilterOut<[1, 'a', 2,], string>>().toEqualTypeOf<[1, 2,]>();
      },
    },),
  ],
},);
