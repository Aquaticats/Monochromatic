/**
 Differential oracle against upstream `pify` 6.1.0: the same generated spec
 drives both implementations and every observable outcome must match
 structurally.
 
 The one documented deviation is the error class name for invalid input
 (`InvalidInputError` extends `TypeError` with upstream's message text), so
 wrap failures compare by message and TypeError-ness.
 
 @module
 */

import {
  assert,
  asyncProperty,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import {
  callbackArgumentArb,
  type ModuleSpec,
  moduleSpecArb,
  type OptionsSpec,
  optionsSpecArb,
  type WrappedFunctionSpec,
  wrappedFunctionSpecArb,
} from './pify-arbitrary.ts';
import {
  forkAdapter,
  type PifyAdapter,
  type PifiedView,
  upstreamAdapter,
} from './pify-adapters.ts';
import {
  buildModule,
  buildOptions,
  buildWrappedFunction,
} from './spec-fixtures.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { describeSettlement, } from './settlement.ts';

//region Oracle helpers

/**
 Comparable outcome of one call: a settlement for promisified calls, a plain
 value for raw returns, or a throw description for raw members invoked
 without their callback.
 */
type CallOutcome = {
  /**
   Outcome kind discriminating returns from settlements from throws.
   */
  readonly kind: 'settled' | 'returned' | 'threw';
  /**
   Comparable payload: settlement, raw value, or throw description.
   */
  readonly value: unknown;
};

/**
 Runs one call through an adapter and describes its outcome.
 
 @param adapter - Implementation whose call shape is used.
 
 @param view - Pified view to call through.
 
 @param key - Member key to call; the empty string means the view itself.
 
 @param args - Caller arguments forwarded before the callback.
 
 @returns Comparable outcome description.
 */
async function describeOutcome(
  {
    adapter,
    view,
    key,
    args,
  }: {
    readonly adapter: PifyAdapter;
    readonly view: PifiedView;
    readonly key: string;
    readonly args: readonly unknown[];
  },
): Promise<CallOutcome> {
  try {
    /**
     Raw call result: a promise when the member is promisified.
     */
    const result: unknown = key === ''
      ? adapter.callMain(view as (...call: never[]) => unknown, args,)
      : adapter.callMember(view, key, args,);
    if (result instanceof Promise)
      return {
        kind: 'settled',
        value: await describeSettlement(result,),
      };
    return {
      kind: 'returned',
      value: result,
    };
  }
  catch (error) {
    return {
      kind: 'threw',
      value: {
        errorName: Error.isError(error,) ? error.name : typeof error,
        message: caughtValueText(error,),
      },
    };
  }
}

/**
 Describes one wrap plus its member probes as comparable entries.
 
 @param adapter - Implementation whose call shape is used.
 
 @param input - Module or function fixture to wrap.
 
 @param options - Runtime options record to wrap with.
 
 @param keys - Member keys to probe after wrapping.
 
 @param probeArguments - Caller arguments used for each probe.
 
 @returns Comparable entries in probe order.
 */
async function describeWorld(
  {
    adapter,
    input,
    options,
    keys,
    probeArguments,
  }: {
    readonly adapter: PifyAdapter;
    readonly input: object;
    readonly options: Record<string, unknown>;
    readonly keys: readonly string[];
    readonly probeArguments: readonly (readonly unknown[])[];
  },
): Promise<readonly unknown[]> {
  /**
   Wrap result: view, or a throw description.
   */
  const view = adapter.wrap(input, options,);
  if ('thrownMessage' in (view as object))
    return [
      {
        wrap: view,
      },
    ];

  /**
   Comparable entries accumulated in probe order.
   */
  const entries: unknown[] = [];
  for (const args of probeArguments)
    // oxlint-disable-next-line eslint/no-await-in-loop -- probe order is part of the observable outcome stream being compared, so entries must complete one at a time
    entries.push(await describeOutcome({
      adapter,
      view: view as PifiedView,
      key: '',
      args,
    },),);
  for (const key of keys) {
    try {
      /**
       Member value observed through the view, captured alongside the call
       outcome so member-shape differences are part of the comparison.
       */
      const member: unknown = Reflect.get(view as object, key,);
      entries.push(
        {
          key,
          // oxlint-disable-next-line eslint/no-await-in-loop -- member probes complete one at a time so both implementations observe the same probe order
          outcome: await describeOutcome({
            adapter,
            view: view as PifiedView,
            key,
            args: [],
          },),
          memberIsFunction: (typeof member) === 'function',
          memberIsView: member === view,
        },
      );
    }
    catch (error) {
      /**
       A member read that throws (upstream `pify` crashes member selection
       when `exclude` is explicitly `undefined`) is itself a comparable
       outcome.
       */
      entries.push(
        {
          key,
          threwGet: {
            errorName: Error.isError(error,) ? error.name : typeof error,
            message: caughtValueText(error,),
          },
        },
      );
    }
  }
  return entries;
}

/**
 Runs one generated world on both implementations and asserts their outcome
 entries match structurally.
 
 @param functionSpec - Wrapped-function behavior to materialize twice.
 
 @param moduleSpec - Module members and values to materialize twice.
 
 @param optionsSpec - Options record to materialize twice.
 
 @param probeArguments - Caller arguments for the function-view calls.
 
 @returns Whether both implementations produced identical entries.
 */
async function compareWorld(
  {
    functionSpec,
    moduleSpec,
    optionsSpec,
    probeArguments,
  }: {
    readonly functionSpec: WrappedFunctionSpec;
    readonly moduleSpec: ModuleSpec;
    readonly optionsSpec: OptionsSpec;
    readonly probeArguments: readonly (readonly unknown[])[];
  },
): Promise<boolean> {
  /**
   Comparable entries produced by each implementation over its own fresh
   fixtures.
   */
  const perImplementation = await Promise.all([forkAdapter, upstreamAdapter,].map(
    async function buildAndRun(adapter: PifyAdapter,): Promise<readonly unknown[]> {
      /**
       Fresh fixtures for this implementation.
       */
      const builtFunction = buildWrappedFunction({
        spec: functionSpec,
      },);
      const module = buildModule({
        spec: moduleSpec,
      },);
      /**
       Member keys probed on the function view's wrapping of the module:
       generated function members plus generated value members.
       */
      const keys = [
        ...moduleSpec.members.map(function readKey(member: { readonly key: string; },): string {
          return member.key;
        },),
        ...Object.keys(moduleSpec.values,),
      ];
      /**
       Entries of the function-view world and the module-view world,
       concatenated in probe order.
       */
      const functionEntries = await describeWorld({
        adapter,
        input: builtFunction.fn as object,
        options: buildOptions({
          spec: optionsSpec,
        },),
        keys: [],
        probeArguments,
      },);
      const moduleEntries = await describeWorld({
        adapter,
        input: module,
        options: buildOptions({
          spec: optionsSpec,
        },),
        keys,
        probeArguments: [
          [],
        ],
      },);
      return [
        ...functionEntries,
        ...moduleEntries,
      ];
    },
  ),);

  expect(JSON.stringify(perImplementation[0],),).toBe(JSON.stringify(perImplementation[1],),);
  return true;
}

/**
 Runs two sequential wraps over one target per implementation and asserts
 both implementations' outcome entries match: upstream `pify`'s selection
 cache freezes each member decision at first touch, so the wrap sequence is
 part of the observable behavior under comparison.
 
 @param moduleSpec - Module members and values to materialize twice.
 
 @param firstOptionsSpec - Options of the first wrap.
 
 @param secondOptionsSpec - Options of the second wrap.
 
 @returns Whether both implementations produced identical entries.
 */
async function compareSequentialWraps(
  {
    moduleSpec,
    firstOptionsSpec,
    secondOptionsSpec,
  }: {
    readonly moduleSpec: ModuleSpec;
    readonly firstOptionsSpec: OptionsSpec;
    readonly secondOptionsSpec: OptionsSpec;
  },
): Promise<boolean> {
  /**
   Member keys probed on both wraps' views.
   */
  const keys = [
    ...moduleSpec.members.map(function readKey(member: { readonly key: string; },): string {
      return member.key;
    },),
    ...Object.keys(moduleSpec.values,),
  ];
  /**
   Comparable entries produced by each implementation over one shared target.
   */
  const perImplementation = await Promise.all([forkAdapter, upstreamAdapter,].map(
    async function buildAndRun(adapter: PifyAdapter,): Promise<readonly unknown[]> {
      /**
       Shared target both wraps observe.
       */
      const module = buildModule({
        spec: moduleSpec,
      },);
      /**
       Entries of both wraps over the shared target, in wrap order.
       */
      const firstEntries = await describeWorld({
        adapter,
        input: module,
        options: buildOptions({
          spec: firstOptionsSpec,
        },),
        keys,
        probeArguments: [
          [],
        ],
      },);
      const secondEntries = await describeWorld({
        adapter,
        input: module,
        options: buildOptions({
          spec: secondOptionsSpec,
        },),
        keys,
        probeArguments: [
          [],
        ],
      },);
      return [
        ...firstEntries,
        ...secondEntries,
      ];
    },
  ),);

  expect(JSON.stringify(perImplementation[0],),).toBe(JSON.stringify(perImplementation[1],),);
  return true;
}

//endregion Oracle helpers

await describe({
  name: 'differential',
  children: [
    //region Settlements

    it({
      name: 'wrapped-function settlements match upstream pify',
      fn: async () => {
        await assert(
          asyncProperty(
            wrappedFunctionSpecArb,
            optionsSpecArb,
            callbackArgumentArb,
            async function sameSettlements(functionSpec, optionsSpec, callerArgument,) {
              await compareWorld({
                functionSpec,
                moduleSpec: {
                  members: [],
                  values: {},
                },
                optionsSpec,
                probeArguments: [
                  [callerArgument,],
                  [],
                ],
              },);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'module member selection and settlements match upstream pify',
      fn: async () => {
        await assert(
          asyncProperty(
            moduleSpecArb,
            optionsSpecArb,
            async function sameMembers(moduleSpec, optionsSpec,) {
              await compareWorld({
                functionSpec: {
                  behavior: {
                    kind: 'callback',
                    payload: ['probe',],
                  },
                  delivery: 'sync',
                  declaredArgumentCount: 0,
                },
                moduleSpec,
                optionsSpec,
                probeArguments: [
                  [],
                ],
              },);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'sequential wraps over one target match upstream pify first-touch caching',
      fn: async () => {
        await assert(
          asyncProperty(
            moduleSpecArb,
            optionsSpecArb,
            optionsSpecArb,
            async function sameFirstTouch(moduleSpec, firstOptions, secondOptions,) {
              await compareSequentialWraps({
                moduleSpec,
                firstOptionsSpec: firstOptions,
                secondOptionsSpec: secondOptions,
              },);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    //endregion Settlements

    //region Totality

    it({
      name: 'wrap failures match upstream pify messages',
      fn: async () => {
        await assert(
          asyncProperty(
            optionsSpecArb,
            async function sameWrapFailures(optionsSpec,) {
              /**
               Inputs invalid for both implementations.
               */
              const invalidInputs: readonly unknown[] = [
                null,
                undefined,
                0,
                'abc',
                true,
              ];
              for (const input of invalidInputs) {
                /**
                 Both implementations' wrap descriptions over one input.
                 */
                // oxlint-disable-next-line eslint/no-await-in-loop -- each invalid input is compared pairwise before the next, so a mismatch names its input
                const wraps = await Promise.all([forkAdapter, upstreamAdapter,].map(
                  async function wrapOne(adapter: PifyAdapter,): Promise<readonly unknown[]> {
                    return describeWorld({
                      adapter,
                      input: input as object,
                      options: buildOptions({
                        spec: optionsSpec,
                      },),
                      keys: [],
                      probeArguments: [],
                    },);
                  },
                ),);
                expect(JSON.stringify(wraps[0],),).toBe(JSON.stringify(wraps[1],),);
              }
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    //endregion Totality
  ],
},);
