/**
 Model and invariant properties over exotic inputs from
 `./exotic-arbitraries.ts`: typed arrays, buffers, boxed primitives, errors,
 weak collections, promises, functions with own properties, Proxy-wrapped
 records and arrays, foreign-realm arrays, Array/Set/Map subclasses, and
 record-like objects over non-plain prototypes, whose kind the accepted
 characterization pins (`./exotic-record-like.ts`); the `deepmerge` property
 requires record-likes to meet other values often enough for that kind to
 decide results.

 - `deepmerge` and `deepmergeFastUnsafe` agree with `./model.ts`: exotic
   leaves resolve to the last value by identity, and containers merge by
   their built-in kind (a subclass merges like its base, into a base-class
   result; `./known-defect-exotic.unit.test.ts` asks upstream whether that is
   intended).
 - `deepmerge` never mutates exotic arguments.
 - `deepmergeInto` with one exotic source agrees with the model. One source
   keeps leaf objects out of the known first-value-typing region
   (`./known-defect.unit.test.ts`), which needs a later record at the same key.

 Run plan and seed policy: see `./fuzz-budget.ts`.

 @module
 */

import {
  assert,
  property,
  tuple,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { treeArbitraries, } from './arbitraries.ts';
import {
  exoticArgumentsArbitrary,
  exoticRecordArbitrary,
} from './exotic-arbitraries.ts';
import { meetsRecordLike, } from './exotic-record-like.ts';
import {
  fingerprint,
  identityTable,
} from './exotic-fingerprint.ts';
import { fuzzRunPlan, } from './fuzz-budget.ts';
import {
  minimumReach,
  reachTally,
} from './reach-tally.ts';
import { modelMerge, } from './model.ts';
import {
  NO_MISMATCH,
  shapeMismatch,
  snapshotObject,
} from './shape.ts';
import { target, } from './target.ts';

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Runs in which a record-like object must meet another value, so its kind decides the result.
 */
const MINIMUM_REACH = minimumReach(RUN.params.numRuns,);

/**
 Writable plain targets for `deepmergeInto`, free of the known into regions.
 */
const plainTargetArbitrary = treeArbitraries({
  exotic: false,
  objectLeaves: false,
  undefinedLeaves: false,
},).record.filter(function isPlainRecord(record,) {
  return Object.getPrototypeOf(record,) === Object.prototype;
},);

await describe({
  name: 'deepmerge-ts on exotic inputs',
  children: [
    it({
      name: 'deepmerge matches the model on exotic trees',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Record-like meetings this property must reach.
         */
        const tally = reachTally(['recordLikeMet',],);
        assert(
          property(
            exoticArgumentsArbitrary,
            function deepmergeMatchesModel(values,) {
              if (meetsRecordLike(values,))
                tally.hit('recordLikeMet',);
              tally.endRun();
              expect(shapeMismatch({
                actual: target.deepmerge(...values,),
                expected: modelMerge({ values, },),
              },),)
                .toBe(NO_MISMATCH,);
            },
          ),
          RUN.params,
        );
        expect(tally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
    it({
      name: 'deepmergeFastUnsafe matches the model on exotic trees',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(
            exoticArgumentsArbitrary,
            function fastMatchesModel(values,) {
              expect(shapeMismatch({
                actual: target.deepmergeFastUnsafe(...values,),
                expected: modelMerge({ values, },),
              },),)
                .toBe(NO_MISMATCH,);
            },
          ),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmerge never mutates exotic record arguments',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(
            tuple(
              exoticRecordArbitrary,
              exoticRecordArbitrary,
            ),
            function exoticArgumentsUnchanged(values,) {
              /**
               Identity session shared by both fingerprints.
               */
              const table = identityTable();
              /**
               Pre-merge fingerprint; `snapshot` would drop subclass prototypes.
               */
              const before = fingerprint({ table, value: values, },);
              target.deepmerge(...values,);
              expect(fingerprint({ table, value: values, },),).toBe(before,);
            },
          ),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmergeInto from one exotic source matches the model',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(
            plainTargetArbitrary,
            exoticRecordArbitrary,
            function intoMatchesModel(generated, source,) {
              /**
               Private target this run may mutate.
               */
              const mutableTarget = snapshotObject(generated,);
              /**
               Model prediction from the untouched target.
               */
              const expected = modelMerge({ values: [generated, source,], },);
              target.deepmergeInto(
                mutableTarget,
                source,
              );
              expect(shapeMismatch({
                actual: mutableTarget,
                expected,
              },),)
                .toBe(NO_MISMATCH,);
            },
          ),
          RUN.params,
        );
      },
    },),
  ],
},);
