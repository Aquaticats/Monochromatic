/**
 Differential property tests: the fork must produce the same observable
 traces as upstream `p-limit` 7.3.3 on the same generated workload.
 
 The oracle is upstream itself: every scheduling, counter, clearQueue, and
 concurrency-change behavior this fork claims is checked against the
 implementation it was derived from.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import pLimitUpstream, {
  limitFunction as limitFunctionUpstream,
} from 'p-limit';
import {
  assert,
  array,
  asyncProperty,
  boolean,
  integer,
  property,
} from 'fast-check';

import {
  limitFunction,
  pLimit,
} from '@monochromatic-dev/upstream-fork-p-limit/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import { workloadArb, } from './limit-arbitrary.ts';
import {
  createForkAdapter,
  createUpstreamAdapter,
} from './limiter-adapters.ts';
import {
  failureFor,
  runWorkload,
  type WorkloadTrace,
} from './workload.ts';

//region Helpers

/**
 Members both implementations attach to a limiter.
 */
const LIMIT_MEMBERS: readonly string[] = [
  'activeCount',
  'pendingCount',
  'concurrency',
  'clearQueue',
  'map',
];

/**
 Echo function driving the `limitFunction` surface comparison.
 */
async function echo(value: string,): Promise<string> {
  return value;
}

/**
 Normalizes one property descriptor for comparison across implementations:
 bound functions are never reference-equal, so only descriptor shape and
 flags are compared.
 
 @param descriptor - Own descriptor from the value under comparison.
 
 @returns Comparable projection of the descriptor.
 
 @example
 ```ts
 normalizeDescriptor(Object.getOwnPropertyDescriptor(limit, 'map',),);
 ```
 */
function normalizeDescriptor(descriptor: PropertyDescriptor,): Record<string, unknown> {
  return {
    present: true,
    kind: (('get' in descriptor) || ('set' in descriptor))
      ? 'accessor'
      : 'data',
    enumerable: descriptor.enumerable
      ?? false,
    configurable: descriptor.configurable
      ?? false,
    writable: descriptor.writable
      ?? false,
    hasGet: (typeof descriptor.get) === 'function',
    hasSet: (typeof descriptor.set) === 'function',
    hasValue: 'value' in descriptor,
  };
}

/**
 Comparable own-member surface of a limiter or limited function.
 
 @param value - Object whose attached members are compared.
 
 @param members - Member names to project.
 
 @returns Normalized descriptor per member name.
 
 @example
 ```ts
 memberSurface(limit, ['map',],);
 ```
 */
function memberSurface(
  value: object,
  members: readonly string[],
): Record<string, unknown> {
  return Object.fromEntries(members.map(function projectMember(member: string,): readonly [string, unknown] {
    /**
     Own descriptor of this member,
     absent when the member is missing.
     */
    const descriptor = Object.getOwnPropertyDescriptor(
      value,
      member,
    );
    return [
      member,
      (descriptor === undefined)
        ? {
          present: false,
        }
        : normalizeDescriptor(descriptor,),
    ];
  },),);
}

//endregion Helpers

/**
 Normalizes a trace for cross-implementation comparison: settlements are
 sorted by call id because settle order across simultaneous settlements is a
 microtask-timing artifact rather than a semantic difference.
 
 @param trace - Trace from one workload run.
 
 @returns Comparable projection of the trace.
 
 @example
 ```ts
 expect(comparable(forkTrace,)).toEqual(comparable(upstreamTrace,),);
 ```
 */
function comparable(trace: WorkloadTrace,): Record<string, unknown> {
  return {
    startOrder: [...trace.startOrder,],
    activeAtStart: [...trace.activeAtStart,],
    boundAtStart: [...trace.boundAtStart,],
    maxOverlap: trace.maxOverlap,
    settlements: trace.settlements
      .toSorted(function byId(left, right,) {
        return left.id.localeCompare(right.id,);
      },),
    pendingAfterStep: [...trace.pendingAfterStep,],
    finalActiveCount: trace.finalActiveCount,
    finalPendingCount: trace.finalPendingCount,
  };
}

//endregion Helpers

await describe({
  name: 'upstream p-limit parity',
  children: [
    it({
      name: 'scheduling traces match upstream p-limit on every generated workload',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            /**
             Options both limiters are constructed with.
             */
            const options = {
              concurrency: workload.concurrency,
              rejectOnClear: workload.rejectOnClear,
            };
            const forkTrace = await runWorkload({
              adapter: createForkAdapter(options,),
              workload,
            },);
            const upstreamTrace = await runWorkload({
              adapter: createUpstreamAdapter(options,),
              workload,
            },);
            expect(comparable(forkTrace,),).toEqual(comparable(upstreamTrace,),);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'map results and mapper failures match upstream p-limit',
      fn: async () => {
        await assert(
          asyncProperty(
            array(
              integer(),
              {
                minLength: 1,
                maxLength: 12,
              },
            ),
            integer({
              min: 1,
              max: 4,
            }),
            async (values: readonly number[], concurrency: number,) => {
              /**
               Index whose mapper call fails, derived from the input length.
               */
              const failingIndex = values.length - 1;

              /**
               Mapper failing at the generated index and doubling otherwise.
               */
              function mapper(value: number, index: number,): number {
                if (index === failingIndex)
                  throw failureFor(`m${String(index,)}`,);
                return value * 2;
              }

              /**
               Fork limiter under comparison.
               */
              const forkLimit = pLimit({ concurrency, },);
              /**
               Upstream limiter under comparison.
               */
              const upstreamLimit = pLimitUpstream({ concurrency, });

              let forkCaught: unknown;
              try {
                await forkLimit.map({
                  iterable: values,
                  mapper,
                },);
              }
              catch (error) {
                forkCaught = error;
              }

              /**
               Upstream's two-positional-argument `map`, detached so lint does
               not read the call as `Array.prototype.map(callback, thisArg)`.
               */
              const upstreamMap = upstreamLimit.map;
              let upstreamCaught: unknown;
              try {
                await upstreamMap(
                  values,
                  function upstreamMapper(value: number, index: number,): number {
                    return mapper(
                      value,
                      index,
                    );
                  },
                );
              }
              catch (error) {
                upstreamCaught = error;
              }

              expect(forkCaught,).toBeInstanceOf(Error,);
              expect(upstreamCaught,).toBeInstanceOf(Error,);
              expect((forkCaught as Error).message,).toBe((upstreamCaught as Error).message,);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'constructor validation messages match upstream p-limit',
      fn: async () => {
        assert(
          property(
            integer({
              min: -3,
              max: 3,
            }),
            integer({
              min: -3,
              max: 3,
            }),
            (concurrencyValue: number, rejectValue: number,) => {
              /**
               Options object shared by both constructors, with `rejectOnClear`
               carrying a generated non-boolean when it is not zero.
               */
              const options = {
                concurrency: concurrencyValue,
                rejectOnClear: (rejectValue === 0)
                  ? true
                  : rejectValue,
              };

              /**
               Error message the fork's constructor threw, if any.
               */
              let forkMessage = '';
              try {
                pLimit(options as never,);
              }
              catch (error) {
                forkMessage = (error as Error).message;
              }

              /**
               Error message the upstream constructor threw, if any.
               */
              let upstreamMessage = '';
              try {
                pLimitUpstream(options as never,);
              }
              catch (error) {
                upstreamMessage = (error as Error).message;
              }

              expect(forkMessage,).toBe(upstreamMessage,);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),
    it({
      name: 'attached member descriptors and enumerable keys match upstream p-limit',
      fn: async () => {
        assert(
          property(
            integer({
              min: 1,
              max: 6,
            }),
            boolean(),
            (concurrency: number, rejectOnClear: boolean,) => {
              /**
               Options shared by both implementations.
               */
              const options = {
                concurrency,
                rejectOnClear,
              };
              /**
               Fork limiter under comparison.
               */
              const forkLimit = pLimit(options,);
              /**
               Upstream limiter under comparison.
               */
              const upstreamLimit = pLimitUpstream(options,);
              expect(memberSurface(
                forkLimit,
                LIMIT_MEMBERS,
              ),).toEqual(memberSurface(
                upstreamLimit,
                LIMIT_MEMBERS,
              ),);
              expect(Object.keys(forkLimit,),).toEqual(Object.keys(upstreamLimit,),);

              /**
               Fork wrapped function under comparison.
               */
              const forkLimited = limitFunction({
                fn: echo,
                options,
              },);
              /**
               Upstream wrapped function under comparison.
               */
              const upstreamLimited = limitFunctionUpstream(
                echo,
                options,
              );
              expect(memberSurface(
                forkLimited,
                [
                  'clearQueue',
                ],
              ),).toEqual(memberSurface(
                upstreamLimited,
                [
                  'clearQueue',
                ],
              ),);
              expect(Object.keys(forkLimited,),).toEqual(Object.keys(upstreamLimited,),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),
  ],
},);
