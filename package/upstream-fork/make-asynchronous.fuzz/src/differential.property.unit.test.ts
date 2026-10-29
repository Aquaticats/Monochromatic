/**
 Differential oracle against upstream `make-asynchronous` 2.1.0: the same
 generated spec drives both implementations and every observable outcome
 must match structurally.
 
 Call shapes differ by design (the fork takes `{ args }` tuples where
 upstream takes rest arguments), so the adapters bridge that gap and the
 oracle compares settlements, drained values, error restoration, and abort
 behavior. Worker-source templates stay byte-identical, and one property
 pins that directly.
 
 @module
 */

import {
  array,
  assert,
  asyncProperty,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  errorReporterSource,
  makeCallWorkerBody,
  makeIterableWorkerBody,
  nodeWorkerPreambleSource,
} from '@monochromatic-dev/upstream-fork-make-asynchronous/ts';

import { readFile, } from 'node:fs/promises';

import { fuzzRuns, } from './fuzz-budget.ts';
import {
  type AsyncAdapter,
  forkAdapter,
  upstreamAdapter,
} from './async-adapters.ts';
import { describeSettlement, } from './settlement.ts';
import {
  type BaseUrlSpec,
  baseUrlSpecArb,
  type IterableFunctionSpec,
  iterableFunctionSpecArb,
  type WrappedFunctionSpec,
  wrappedFunctionSpecArb,
} from './workload-arbitrary.ts';
import {
  buildBaseUrl,
  buildIterableFunction,
  buildWrappedFunction,
} from './spec-fixtures.ts';

//region Oracle helpers

/**
 Upstream worker-source templates read from the installed
 `make-asynchronous` package, compared verbatim against the fork's.
 */
type UpstreamSources = {
  /**
   Raw upstream `index.js` text.
   */
  readonly text: string;
};

/**
 Reads upstream's `index.js` once per property run.
 
 @returns Upstream source text.
 */
async function readUpstreamSources(): Promise<UpstreamSources> {
  /**
   Upstream entry resolved through the fuzz package's dependency.
   */
  const entry = import.meta.resolve('make-asynchronous',);
  /**
   Raw upstream source text.
   */
  const text = await readFile(
    new URL(entry,),
    'utf8',
  );
  return { text, };
}

/**
 Caller arguments matching one spec's declared arity.
 
 @param declaredArgumentCount - Argument slots the fixture declares.
 
 @returns Argument tuple of JSON strings.
 */
function argsFor(declaredArgumentCount: number,): readonly never[] {
  /**
   Argument tuple matching the fixture's declared arity.
   */
  const args: never[] = [];
  for (let index = 0; index < declaredArgumentCount; index += 1)
    args.push(`arg-${index}` as never,);
  return args;
}

/**
 Runs one single-call spec on one implementation and describes its outcome.
 
 @param adapter - Implementation whose call shape is used.
 
 @param spec - Outcome and arity to materialize fresh for this run.
 
 @param base - Base URL option to materialize fresh for this run.
 
 @returns Comparable settlement description.
 */
async function describeCall(
  {
    adapter,
    spec,
    base,
  }: {
    readonly adapter: AsyncAdapter;
    readonly spec: WrappedFunctionSpec;
    readonly base: BaseUrlSpec;
  },
): Promise<unknown> {
  /**
   Fresh fixture for this implementation.
   */
  const built = buildWrappedFunction({ spec, },);
  try {
    return await describeSettlement(adapter.call(
      built.fn,
      buildBaseUrl({ spec: base, },),
      argsFor(built.declaredArgumentCount,),
    ),);
  }
  catch (error) {
    return {
      adapterThrew: true,
      error: String(error,),
    };
  }
}

/**
 Runs one iterable spec on one implementation and describes its outcome.
 
 @param adapter - Implementation whose iteration shape is used.
 
 @param spec - Values, failure flag, and arity to materialize fresh.
 
 @param base - Base URL option to materialize fresh for this run.
 
 @returns Comparable drain description.
 */
async function describeIteration(
  {
    adapter,
    spec,
    base,
  }: {
    readonly adapter: AsyncAdapter;
    readonly spec: IterableFunctionSpec;
    readonly base: BaseUrlSpec;
  },
): Promise<unknown> {
  /**
   Fresh fixture for this implementation.
   */
  const built = buildIterableFunction({ spec, },);
  /**
   Values drained in yield order.
   */
  const values: unknown[] = [];
  try {
    values.push(...await adapter.iterate(
      built.fn,
      buildBaseUrl({ spec: base, },),
      argsFor(built.declaredArgumentCount,),
    ),);
    return {
      values,
    };
  }
  catch (thrown) {
    return {
      values,
      thrown: {
        tag: 'rejected',
        value: thrown,
      },
    };
  }
}

//endregion Oracle helpers

await describe({
  name: 'upstream make-asynchronous parity',
  children: [
    it({
      name: 'single-call settlements match upstream make-asynchronous',
      fn: async () => {
        await assert(
          asyncProperty(
            wrappedFunctionSpecArb,
            baseUrlSpecArb,
            async function sameSettlements(spec: WrappedFunctionSpec, base: BaseUrlSpec,) {
              /**
               Settlement per implementation over freshly built fixtures.
               */
              const settlements = await Promise.all([forkAdapter, upstreamAdapter,].map(
                async function runOne(adapter: AsyncAdapter,) {
                  return describeCall({
                    adapter,
                    base,
                    spec,
                  },);
                },
              ),);
              expect(JSON.stringify(settlements[0],),).toBe(JSON.stringify(settlements[1],),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
      timeout: 120_000,
    },),

    it({
      name: 'iteration drains match upstream make-asynchronous',
      fn: async () => {
        await assert(
          asyncProperty(
            iterableFunctionSpecArb,
            baseUrlSpecArb,
            async function sameDrains(spec: IterableFunctionSpec, base: BaseUrlSpec,) {
              /**
               Drain per implementation over freshly built fixtures.
               */
              const drains = await Promise.all([forkAdapter, upstreamAdapter,].map(
                async function runOne(adapter: AsyncAdapter,) {
                  return describeIteration({
                    adapter,
                    base,
                    spec,
                  },);
                },
              ),);
              expect(JSON.stringify(drains[0],),).toBe(JSON.stringify(drains[1],),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
      timeout: 120_000,
    },),

    it({
      name: 'concurrent calls match upstream make-asynchronous in order',
      fn: async () => {
        await assert(
          asyncProperty(
            array(
              wrappedFunctionSpecArb,
              {
                maxLength: 6,
                minLength: 1,
              },
            ),
            async function sameConcurrent(specs: readonly WrappedFunctionSpec[],) {
              /**
               Settlements per implementation, each spec over fresh fixtures.
               */
              const perImplementation = await Promise.all([forkAdapter, upstreamAdapter,].map(
                async function runAll(adapter: AsyncAdapter,) {
                  return Promise.all(specs.map(
                    async function runOne(spec: WrappedFunctionSpec,) {
                      return describeCall({
                        adapter,
                        base: {
                          kind: 'absent',
                          value: '',
                        },
                        spec,
                      },);
                    },
                  ),);
                },
              ),);
              expect(JSON.stringify(perImplementation[0],),).toBe(JSON.stringify(perImplementation[1],),);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
      timeout: 180_000,
    },),

    it({
      name: 'worker-source templates match upstream make-asynchronous verbatim',
      fn: async () => {
        /**
         Upstream source text for template comparison.
         */
        const upstream = await readUpstreamSources();
        expect(upstream.text,).toContain(errorReporterSource.trim(),);
        expect(upstream.text,).toContain(nodeWorkerPreambleSource.trim(),);
        expect(upstream.text,).toContain('globalThis.onmessage = async ({data: {id, arguments_}}) => {',);
        expect(makeCallWorkerBody('function probe() {}',),).toContain('function probe() {}',);
        expect(makeIterableWorkerBody('function * probe() {}',),).toContain('function * probe() {}',);
      },
    },),
  ],
},);
