/**
 Differential property tests: the fork must produce the same observable
 traces as upstream `quick-lru` 7.3.0 on the same generated workload.
 
 The oracle is upstream itself: every storage, recency, expiry, eviction,
 resize, and `evict` behavior this fork claims is checked against the
 implementation it was derived from, including the member descriptors and
 the constructor failure texts.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  assert,
  property,
} from 'fast-check';
import QuickLRUUpstream from 'quick-lru';

import {
  createQuickLru,
} from '@monochromatic-dev/upstream-fork-quick-lru/ts';
import { installFakeClock, } from '@monochromatic-dev/upstream-fork-quick-lru/ts/test-support.ts';

import {
  junkInputArb,
  workloadArb,
} from './cache-arbitrary.ts';
import {
  createForkAdapter,
  createUpstreamAdapter,
} from './cache-adapters.ts';
import { fuzzRuns, } from './fuzz-budget.ts';
import {
  runWorkload,
} from './cache-workload.ts';

//region Fixtures

/**
 Instant every workload run starts from; both caches observe identical
 time through the shared controlled clock.
 */
const CLOCK_START = 1_700_000_000_000;

/**
 Members upstream attaches as writable class-prototype methods.
 */
const METHOD_MEMBERS: readonly PropertyKey[] = [
  'set',
  'get',
  'has',
  'peek',
  'delete',
  'clear',
  'expiresIn',
  'resize',
  'evict',
  'keys',
  'values',
  'entries',
  'entriesAscending',
  'entriesDescending',
  'forEach',
  'toString',
  Symbol.iterator,
  Symbol.for('nodejs.util.inspect.custom',),
];

/**
 Members upstream attaches as getter-only class-prototype accessors.
 */
const GETTER_MEMBERS: readonly PropertyKey[] = [
  'size',
  'maxSize',
  'maxAge',
  '__oldCache',
  Symbol.toStringTag,
];

//endregion Fixtures

//region Helpers

/**
 Normalizes one property descriptor for comparison across
 implementations: function values are never reference-equal, so only
 descriptor shape and flags are compared.
 
 @param descriptor - Own descriptor from the surface under comparison.
 
 @returns Comparable projection of the descriptor.
 
 @example
 ```ts
 normalizeDescriptor(Object.getOwnPropertyDescriptor(fork, 'get',),);
 ```
 */
function normalizeDescriptor(descriptor?: PropertyDescriptor,): Record<string, unknown> {
  if (descriptor === undefined)
    return {
      present: false,
    };

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
 Builds one cache and renders how its construction failed, for failure
 text comparison across implementations. The fork's configuration errors
 keep upstream's message text verbatim while naming their own classes, so
 the comparison normalizes on messages exactly as upstream renders them.
 
 @param build - Construction to run.
 
 @returns `constructed`, or the thrown failure's normalized text.
 
 @example
 ```ts
 constructionOutcome(function construct(): unknown {
   return new QuickLRUUpstream({ maxSize: 0, },);
 },);
 ```
 */
function constructionOutcome(build: () => unknown,): string {
  try {
    build();
    return 'constructed';
  }
  catch (error) {
    return `threw ${Error.isError(error,) ? error.message : caughtValueText(error,)}`;
  }
}

//endregion Helpers

await describe({
  name: 'upstream differential',
  concurrency: 1,
  children: [
    it({
      name: 'pins every member descriptor to upstream class-prototype flags',
      fn: async () => {
        /**
         Fork cache whose own members are compared.
         */
        const fork = createQuickLru<string, string>({
          maxSize: 1,
        },);
        /**
         Upstream cache whose prototype members are compared.
         */
        const upstream = new QuickLRUUpstream<string, string>({
          maxSize: 1,
        },);
        /**
         Upstream member source: everything lives on the class prototype.
         */
        const upstreamMembers = Reflect.getPrototypeOf(upstream,) ?? {};

        for (const member of [
          ...METHOD_MEMBERS,
          ...GETTER_MEMBERS,
        ]) {
          expect(
            normalizeDescriptor(Object.getOwnPropertyDescriptor(fork, member,),),
          )
            .toEqual(normalizeDescriptor(Object.getOwnPropertyDescriptor(upstreamMembers, member,),),);
        }
      },
    },),

    it({
      name: 'produces identical traces on identical generated workloads',
      fn: async () => {
        assert(
          property(
            workloadArb,
            function compareTraces(workload,): void {
              using clock = installFakeClock({
                startMilliseconds: CLOCK_START,
              },);
              /**
               Fork cache trace over the generated workload.
               */
              const forkTrace = runWorkload({
                adapter: createForkAdapter({
                  maxSize: workload.maxSize,
                  maxAge: workload.maxAge,
                  onEviction: workload.onEviction,
                },),
                workload,
                clock,
                startMilliseconds: CLOCK_START,
              },);
              /**
               Upstream cache trace over the same generated workload.
               */
              const upstreamTrace = runWorkload({
                adapter: createUpstreamAdapter({
                  maxSize: workload.maxSize,
                  maxAge: workload.maxAge,
                  onEviction: workload.onEviction,
                },),
                workload,
                clock,
                startMilliseconds: CLOCK_START,
              },);
              expect(upstreamTrace,).toEqual(forkTrace,);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'fails or builds identically on every junk constructor input',
      fn: async () => {
        assert(
          property(
            junkInputArb,
            function compareConstruction(input,): void {
              expect(
                constructionOutcome(function constructFork(): unknown {
                  return createQuickLru<string, string>(input as never,);
                },),
              )
                .toBe(constructionOutcome(function constructUpstream(): unknown {
                  return new QuickLRUUpstream<string, string>(input as never,);
                },),);
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
