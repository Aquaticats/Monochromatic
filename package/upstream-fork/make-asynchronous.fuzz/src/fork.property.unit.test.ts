/**
 Fork-only invariants: worker-source fidelity, settlement restoration, and
 `withSignal` member shape.
 
 @module
 */

import {
  array,
  assert,
  asyncProperty,
  integer,
  property,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  errorReporterSource,
  getResult,
  makeAsynchronous,
  makeAsynchronousIterable,
  makeCallWorkerBody,
  makeIterableWorkerBody,
  nodeWorkerPreambleSource,
} from '@monochromatic-dev/upstream-fork-make-asynchronous/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import {
  type IterableFunctionSpec,
  iterableFunctionSpecArb,
  payloadArb,
  type WrappedFunctionSpec,
  wrappedFunctionSpecArb,
} from './workload-arbitrary.ts';
import {
  buildIterableFunction,
  buildWrappedFunction,
} from './spec-fixtures.ts';
import { describeSettlement, } from './settlement.ts';

//region Helpers

/**
 Caller arguments matching one spec's declared arity.
 
 @param declaredArgumentCount - Argument slots the fixture declares.
 
 @param seed - Seed string distinguishing each slot.
 
 @returns Argument tuple for one call.
 */
function argsFor(
  {
    declaredArgumentCount,
    seed,
  }: {
    readonly declaredArgumentCount: number;
    readonly seed: string;
  },
): readonly never[] {
  /**
   Argument tuple matching the fixture's declared arity.
   */
  const args: never[] = [];
  for (let index = 0; index < declaredArgumentCount; index += 1)
    args.push(`${seed}-${index}` as never,);
  return args;
}

//endregion Helpers

await describe({
  name: 'fork invariants',
  children: [
    //region Sources

    it({
      name: 'worker sources carry the serialized function and the reply protocol keys',
      fn: async () => {
        assert(
          property(
            string(),
            function sourceCarries(serialized: string,) {
              expect(makeCallWorkerBody(serialized,),).toContain(serialized,);
              expect(makeCallWorkerBody(serialized,),).toContain('{id, output}',);
              expect(makeCallWorkerBody(serialized,),).toContain('reportError(error, id)',);
              expect(makeIterableWorkerBody(serialized,),).toContain(serialized,);
              expect(makeIterableWorkerBody(serialized,),).toContain('Symbol.asyncIterator',);
              expect(makeIterableWorkerBody(serialized,),).toContain('Iterator result is not an object',);
              expect(errorReporterSource,).toContain('getErrorProperties',);
              expect(nodeWorkerPreambleSource,).toContain('node:worker_threads',);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    //endregion Sources

    //region Settlements

    it({
      name: 'single calls settle with their specified outcome through the fork',
      fn: async () => {
        await assert(
          asyncProperty(
            wrappedFunctionSpecArb,
            string(),
            async function specifiedOutcome(spec: WrappedFunctionSpec, seed: string,) {
              /**
               Fresh fixture for this run.
               */
              const built = buildWrappedFunction({ spec, },);
              /**
               Wrapped function under test.
               */
              const wrapped = makeAsynchronous({ fn: built.fn, },);
              /**
               Settlement observed through the fork.
               */
              /**
               Argument tuple matching the fixture's declared arity.
               */
              const callArgs = argsFor({
                declaredArgumentCount: built.declaredArgumentCount,
                seed,
              },);
              const settlement = await describeSettlement(wrapped({ args: [...callArgs,], },),);
              if (spec.outcome.kind === 'resolve') {
                expect(settlement.tag,).toBe('resolved',);
                expect(JSON.stringify(settlement.value,),).toBe(JSON.stringify(spec.outcome.payload ?? null,),);
              }
              else if (spec.outcome.kind === 'rejectError') {
                expect(settlement.tag,).toBe('rejected',);
                expect((settlement.value as { readonly message?: unknown; }).message,).toBe(spec.outcome.payload,);
              }
              else {
                expect(settlement.tag,).toBe('rejected',);
                expect(JSON.stringify(settlement.value,),).toBe(JSON.stringify(spec.outcome.payload,),);
              }
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'iterations drain generated values in order through the fork',
      fn: async () => {
        await assert(
          asyncProperty(
            iterableFunctionSpecArb,
            string(),
            async function specifiedValues(spec: IterableFunctionSpec, seed: string,) {
              /**
               Fresh fixture for this run.
               */
              const built = buildIterableFunction({ spec, },);
              /**
               Wrapped iterable under test.
               */
              const wrapped = makeAsynchronousIterable({ fn: built.fn, },);
              /**
               Drain state collected across the iteration below.
               */
              const drain = {
                caught: undefined as unknown,
                values: [] as unknown[],
              };
              /**
               Argument tuple matching the fixture's declared arity.
               */
              const iterationArgs = argsFor({
                declaredArgumentCount: built.declaredArgumentCount,
                seed,
              },);
              try {
                for await (const value of wrapped({ args: [...iterationArgs,], }))
                  drain.values.push(value,);
              }
              catch (error) {
                drain.caught = error;
              }
              if (spec.throwsAfter) {
                expect(drain.caught,).toBeInstanceOf(Error,);
                expect((drain.caught as Error).message,).toBe('iterable fixture failed',);
              }
              else
                expect(drain.caught,).toBeUndefined();
              expect(JSON.stringify(drain.values,),).toBe(JSON.stringify(spec.values,),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'falsy results resolve and falsy failures reject through getResult',
      fn: async () => {
        assert(
          property(
            payloadArb,
            integer(),
            function falsyProtocol(payload: unknown, id: number,) {
              expect(getResult({
                id,
                output: payload,
              },),).toEqual(payload,);
              /**
               Thrown payload observed through the reply protocol.
               */
              const observed = {
                caught: undefined as unknown,
              };
              try {
                getResult({
                  error: payload,
                  id,
                },);
              }
              catch (error) {
                observed.caught = error;
              }
              expect(observed.caught,).toEqual(payload,);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'withSignal stays enumerable on both wrapped forms',
      fn: async () => {
        assert(
          property(
            integer({
              max: 100,
              min: 0,
            }),
            function memberShape(seed: number,) {
              /**
               Wrapped call form under test.
               */
              const wrapped = makeAsynchronous({
                fn: function identity(value: number,): number {
                  return value + seed;
                },
              },);
              /**
               Wrapped iterable form under test.
               */
              const iterated = makeAsynchronousIterable({
                *fn (value: number,): Generator<number> {
                  yield value + seed;
                },
              },);
              for (const candidate of [wrapped, iterated,]) {
                expect(Object.keys(candidate,),).toEqual([
                  'withSignal',
                ],);
                /**
                 Own descriptor of the attached member.
                 */
                const descriptor = Object.getOwnPropertyDescriptor(
                  candidate,
                  'withSignal',
                );
                expect(descriptor?.enumerable,).toBe(true,);
                expect(descriptor?.writable,).toBe(true,);
                expect(descriptor?.configurable,).toBe(true,);
              }
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'concurrent calls stay isolated under generated payloads',
      fn: async () => {
        await assert(
          asyncProperty(
            array(
              payloadArb,
              {
                maxLength: 10,
                minLength: 1,
              },
            ),
            async function isolatedCalls(payloads: readonly unknown[],) {
              /**
               Wrapped echo function under test.
               */
              const wrapped = makeAsynchronous({
                fn: function identity(value: unknown,): unknown {
                  return value;
                },
              },);
              /**
               Settlements in call order, one worker per call.
               */
              const settled = await Promise.all(payloads.map(function callEcho(payload: unknown,) {
                return wrapped({ args: [payload], },);
              },),);
              expect(JSON.stringify(settled,),).toBe(JSON.stringify(payloads,),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
      timeout: 60_000,
    },),

    //endregion Settlements
  ],
},);
