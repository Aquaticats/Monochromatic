/**
 Property tests proving the fork's store invariants:
 file round-trips,
 item-count consistency,
 reserved-key invisibility,
 defaults surviving `clear` and `reset`,
 dot-notation reads,
 `appendToArray` order,
 and bad-value writes leaving the store untouched.
 
 @module
 */

import { readFileSync, } from 'node:fs';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type Arbitrary,
  array,
  assert,
  asyncProperty,
  constantFrom,
  tuple,
} from 'fast-check';

import {
  type Conf,
  createConf,
} from '@monochromatic-dev/upstream-fork-conf/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import {
  createTempStoreDir,
  jsonClone,
} from './store-fixture.ts';
import {
  type JsonRecord,
  type JsonValue,
  type StoreOperationSequence,
  STORE_DEFAULTS,
  jsonValueArb,
  keyArb,
  sequenceArb,
} from './store-operations.ts';
import { runOnFork, } from './store-adapters.ts';

//region Helpers

/**
 Removes reserved bookkeeping key paths from a generated value.
 
 `set` refuses payloads addressing `__internal__` at any depth,
 mirroring upstream,
 so unscrubbed values would test that refusal (pinned separately by the
 `reserved \`__internal__\` key never surfaces` property) instead of the
 invariant under test.
 
 @param value - Generated JSON value.
 
 @returns Value without reserved key paths.
 
 @example
 ```ts
 scrubReservedKeys({ theme: { '__internal__.x': 1, }, }); // => { theme: {} }
 ```
 */
function scrubReservedKeys(value: JsonValue,): JsonValue {
  if (Array.isArray(value,))
    return value.map(function scrubItem(item: JsonValue,): JsonValue {
      return scrubReservedKeys(item,);
    },);
  if (((typeof value) === 'object') && (value !== null))
    return Object.fromEntries(
      Object.entries(value,)
        .filter(function isUserKey(entry: readonly [string, JsonValue],): boolean {
          return !isReservedKeyPath(entry[0],);
        },)
        .map(function scrubEntry(entry: readonly [string, JsonValue],): [string, JsonValue] {
          return [
            entry[0],
            scrubReservedKeys(entry[1],),
          ];
        },),
    );
  return value;
}

/**
 Top-level keys whose contents the reserved bookkeeping scan hides:
 `__internal__` itself and `__internal__.`-prefixed paths.
 */
function isReservedKeyPath(key: string,): boolean {
  return (key === '__internal__')
    || key.startsWith('__internal__.',);
}

/**
 Removes reserved bookkeeping entries from file contents,
 yielding what the user-visible store must show.
 
 @param fileContents - Parsed config-file contents.
 
 @returns File contents without reserved top-level keys.
 
 @example
 ```ts
 withoutReservedKeys({ __internal__: 1, theme: 'dark', });
 // => { theme: 'dark' }
 ```
 */
function withoutReservedKeys(fileContents: JsonRecord,): JsonRecord {
  return Object.fromEntries(Object.entries(fileContents,)
    .filter(function keepUserEntry([key,]: readonly [string, JsonValue],): boolean {
      return !isReservedKeyPath(key,);
    },),);
}

/**
 Runs one thunk and returns whatever it threw,
 or `undefined` when it returned normally.
 
 @param thunk - Operation expected to throw.
 
 @returns Thrown value,
 or `undefined` when nothing was thrown.
 
 @example
 ```ts
 captureThrown(function bad() {
   throw new TypeError('nope');
 }); // => TypeError
 ```
 */
function captureThrown(thunk: () => void,): unknown {
  try {
    thunk();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

/**
 Keys the round-trip fixture writes:
 plain and dotted keys only,
 since reserved keys are covered by their own invariant.
 */
const ROUND_TRIP_KEYS: readonly string[] = [
  'theme',
  'retries',
  'items',
  'nested',
  'a',
  'b',
  'a.b',
  'extra',
  'weird key',
  'ü',
];

/**
 Generated whole-store replacement over round-trip keys.
 */
const roundTripRecordArb: Arbitrary<JsonRecord> = array(
  tuple(
    constantFrom(...ROUND_TRIP_KEYS,),
    jsonValueArb,
  ),
  {
    maxLength: 4,
  },
)
  .map(function toRecord(entries: readonly (readonly [string, JsonValue])[],): JsonRecord {
    return Object.fromEntries(entries,);
  },);

/**
 Builds a fork store in a disposable temp directory.
 
 @param cwd - Disposable directory the config file lives in.
 
 @returns Fork store seeded with {@link STORE_DEFAULTS}.
 
 @example
 ```ts
 await using dir = await createTempStoreDir();
 const config = createForkStore(dir.path);
 ```
 */
function createForkStore(cwd: string,): Conf<Record<string, JsonValue>> {
  return createConf<Record<string, JsonValue>>({
    cwd,
    defaults: STORE_DEFAULTS,
  });
}

//endregion Helpers

await describe({
  name: 'fork invariants',
  children: [
    it({
      name: 'store replacement round-trips through the config file',
      fn: async () => {
        await assert(
          asyncProperty(roundTripRecordArb, async (value: JsonRecord,) => {
            /**
             Disposable directory holding this store's config file.
             */
            await using dir = await createTempStoreDir();
            const config = createForkStore(dir.path,);
            config.store = value;
            const fileJson = JSON.parse(readFileSync(
              config.path,
              'utf8',
            ),) as JsonRecord;
            expect(jsonClone(fileJson,),).toEqual(jsonClone(value,),);
            expect(jsonClone(config.store,),).toEqual(jsonClone(value,),);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'rereading the config file yields the user-visible store',
      fn: async () => {
        await assert(
          asyncProperty(sequenceArb, async (sequence: StoreOperationSequence,) => {
            const observation = await runOnFork(sequence,);
            expect(withoutReservedKeys(observation.fileContents as JsonRecord,),).toEqual(observation.finalStore,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'size equals the store key count after any operation sequence',
      fn: async () => {
        await assert(
          asyncProperty(sequenceArb, async (sequence: StoreOperationSequence,) => {
            const observation = await runOnFork(sequence,);
            expect(observation.size,).toBe(Object.keys(observation.finalStore,)
              .length,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'reserved `__internal__` key never surfaces',
      fn: async () => {
        await assert(
          asyncProperty(jsonValueArb, async (payload: JsonValue,) => {
            /**
             Disposable directory holding this store's config file.
             */
            await using dir = await createTempStoreDir();
            const config = createForkStore(dir.path,);
            expect(captureThrown(function reservedSingleSet(): void {
              config.set({
                key: '__internal__',
                value: payload,
              },);
            },),).toBeInstanceOf(TypeError,);
            expect(captureThrown(function reservedPathSet(): void {
              config.set({
                key: '__internal__.x',
                value: payload,
              },);
            },),).toBeInstanceOf(TypeError,);
            expect(captureThrown(function reservedMultiSet(): void {
              config.set({
                values: Object.fromEntries([
                  [
                    '__internal__',
                    payload,
                  ],
                ],),
              },);
            },),).toBeInstanceOf(TypeError,);
            expect(captureThrown(function nestedReservedSet(): void {
              config.set({
                values: Object.fromEntries([
                  [
                    'a',
                    Object.fromEntries([
                      [
                        '__internal__',
                        payload,
                      ],
                    ],),
                  ],
                ],),
              },);
            },),).toBeInstanceOf(TypeError,);
            expect(captureThrown(function reservedAppend(): void {
              config.appendToArray({
                key: '__internal__',
                value: payload,
              },);
            },),).toBeInstanceOf(TypeError,);

            // Whole-store replacement is the one write that may carry the
            // reserved key into the file; whatever the fork does with it,
            // the reserved key must stay off the public surface.
            captureThrown(function reservedReplacement(): void {
              config.store = Object.fromEntries([
                [
                  '__internal__',
                  payload,
                ],
                [
                  'keep',
                  1,
                ],
              ],);
            },);

            /**
             User-visible store keys after the reserved-key attempts.
             */
            const visibleKeys = Object.keys(config.store,);
            expect(visibleKeys.some(isReservedKeyPath,),).toBe(false,);
            expect(config.has('__internal__',),).toBe(false,);
            expect(config.get('__internal__',),).toBe(undefined,);
            /**
             Iterated entry keys,
             which must agree with the store's own keys.
             */
            const iteratedKeys = [...config,].map(function toKey(entry: readonly [string, unknown],): string {
              return entry[0];
            },);
            expect(iteratedKeys.some(isReservedKeyPath,),).toBe(false,);
            expect(config.size,).toBe(visibleKeys.length,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'defaults survive clear and reset',
      fn: async () => {
        await assert(
          asyncProperty(jsonValueArb, async (fresh: JsonValue,) => {
            /**
             Disposable directory holding this store's config file.
             */
            await using dir = await createTempStoreDir();
            const config = createForkStore(dir.path,);
            /**
             Generated value without reserved key paths:
             multi-item `set` refuses reserved payloads at any depth.
             */
            const safeFresh = scrubReservedKeys(fresh,);
            config.set({
              values: {
                theme: safeFresh,
                retries: safeFresh,
                items: [safeFresh,],
              },
            },);
            config.clear();
            expect(jsonClone(config.store,),).toEqual(jsonClone(STORE_DEFAULTS,),);
            config.set({
              values: {
                theme: safeFresh,
                retries: safeFresh,
              },
            },);
            config.reset({
              keys: ['theme',],
            },);
            expect(
              jsonClone(config.get('theme',) as JsonValue,),
            ).toEqual(jsonClone(STORE_DEFAULTS.theme as JsonValue,),);
            expect(
              jsonClone(config.get('retries',),),
            ).toEqual(jsonClone(safeFresh,),);
            config.reset({
              keys: [
                'theme',
                'retries',
              ],
            },);
            expect(jsonClone(config.store,),).toEqual(jsonClone(STORE_DEFAULTS,),);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'dot-notation writes read back',
      fn: async () => {
        await assert(
          asyncProperty(jsonValueArb, async (value: JsonValue,) => {
            /**
             Disposable directory holding this store's config file.
             */
            await using dir = await createTempStoreDir();
            const config = createForkStore(dir.path,);
            config.set({
              key: 'nested.flag',
              value,
            },);
            expect(config.has('nested.flag',),).toBe(true,);
            expect(
              jsonClone(config.get('nested.flag',),),
            ).toEqual(jsonClone(value,),);
            expect(
              jsonClone(config.get('nested',),),
            ).toEqual(jsonClone({
              flag: value,
            },),);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'appendToArray preserves order',
      fn: async () => {
        await assert(
          asyncProperty(
            array(
              jsonValueArb,
              {
                minLength: 1,
                maxLength: 5,
              },
            ),
            async (values: readonly JsonValue[],) => {
              /**
               Disposable directory holding this store's config file.
               */
              await using dir = await createTempStoreDir();
              const config = createForkStore(dir.path,);
              for (const value of values)
                config.appendToArray({
                  key: 'a',
                  value,
                },);
              expect(
                jsonClone(config.get('a',),),
              ).toEqual(jsonClone(values,),);
              config.clear();
              for (const value of values)
                config.appendToArray({
                  key: 'items',
                  value,
                },);
              expect(
                jsonClone(config.get('items',),),
              ).toEqual(jsonClone(values,),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'set-with-bad-value writes nothing and leaves the store file unchanged',
      fn: async () => {
        await assert(
          asyncProperty(keyArb, async (key: string,) => {
            /**
             Disposable directory holding this store's config file.
             */
            await using dir = await createTempStoreDir();
            const config = createForkStore(dir.path,);
            config.set({
              key: 'a',
              value: 1,
            },);
            /**
             File bytes before the bad-value attempts.
             */
            const before = readFileSync(
              config.path,
              'utf8',
            );
            /**
             Store contents before the bad-value attempts.
             */
            const storeBefore = jsonClone(config.store,);

            expect(captureThrown(function undefinedSingleValue(): void {
              config.set({
                key,
                value: undefined,
              },);
            },),).toBeInstanceOf(TypeError,);
            expect(captureThrown(function undefinedMultiValue(): void {
              config.set({
                values: Object.fromEntries([
                  [
                    key,
                    undefined,
                  ],
                ],),
              },);
            },),).toBeInstanceOf(TypeError,);
            expect(captureThrown(function functionValue(): void {
              config.set({
                key,
                value: function badValue(): void {
                  return undefined;
                },
              },);
            },),).toBeInstanceOf(TypeError,);
            expect(captureThrown(function symbolValue(): void {
              config.set({
                key,
                value: Symbol('store value of unsupported type',),
              },);
            },),).toBeInstanceOf(TypeError,);
            expect(captureThrown(function undefinedAppendValue(): void {
              config.appendToArray({
                key,
                value: undefined,
              },);
            },),).toBeInstanceOf(TypeError,);

            expect(readFileSync(
              config.path,
              'utf8',
            ),).toBe(before,);
            expect(jsonClone(config.store,),).toEqual(storeBefore,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),
  ],
},);
