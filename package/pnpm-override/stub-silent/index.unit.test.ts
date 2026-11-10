import { readFile, } from 'node:fs/promises';
import { createRequire, } from 'node:module';
import { setTimeout as delay, } from 'node:timers/promises';

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import stub from './index.cjs';

//region Fixtures

/**
 CommonJS loader anchored at this test, so a case can compare against what a consumer's `require` of the published
 entry point returns instead of trusting the ESM interop value alone.
 */
const requireFromHere = createRequire(import.meta.url,);

/**
 Value the CommonJS loader resolves for the published entry point.
 */
const requiredStub: unknown = requireFromHere('./index.cjs',);

/**
 Text a primitive coercion of the stub produces, which this package's README documents. The coercion case asserts both
 that the runtime produces this text and that the README states it, so the two cannot drift apart silently.
 */
const PRIMITIVE_TEXT = '[stub-silent]';

/**
 What every case that expects a stub-shaped value compares against, so a failure names the identity instead of asking
 chai to stringify the stub, which throws the same `TypeError` the coercion case covers.
 */
const STUB_IDENTITY = 'the stub itself';

/**
 Long enough for the promise resolution jobs of a settled await to run, short enough that a deadlocked stub does not
 stall the suite.
 */
const SETTLE_DEADLINE_MS = 1_000;

/**
 Sentinel the deadline arm of an await case resolves to, so a failure reads as "the stub never settled" instead of
 comparing two stub-looking values.
 */
const DEADLINE = Symbol('stub await did not settle before the deadline',);

/**
 Property keys the promise machinery reads to decide whether a value is thenable.
 */
const PROMISE_MACHINERY_KEYS = [
  'then',
  'catch',
  'finally',
] as const;

/**
 Stub values a consumer may await, each labeled by the shape it exercises.
 */
const AWAITABLE_STUB_EXPRESSIONS: readonly {
  readonly label: string;
  readonly produce: () => unknown;
}[] = [
  { label: 'the stub itself', produce: () => stub, },
  { label: 'a property read chain', produce: () => stub.foo?.bar, },
  { label: 'a call chain', produce: () => stub.foo?.bar?.(), },
  { label: 'a call result', produce: () => stub(), },
];

/**
 Awaits `value` under a deadline, so a value that never settles fails an assertion instead of hanging the case.

 @param value - Stub value or chain to await

 @returns The settled value, or `DEADLINE` when the await did not settle in time
 */
async function awaitWithinDeadline(value: unknown,): Promise<unknown> {
  return Promise.race([
    Promise.resolve(value,),
    delay(SETTLE_DEADLINE_MS,).then(() => DEADLINE,),
  ],);
}

/**
 Interpolates `value` into a template literal, the substitution shape a consumer reaches the primitive coercion
 through.

 @param value - Value to interpolate

 @returns The text the template literal produces

 @example
 ```ts
 templateCoerce(1,); // => '1'
 ```
 */
function templateCoerce(value: unknown,): string {
  // oxlint-disable-next-line typescript/restrict-template-expressions -- the case must interpolate the stub itself; `String()` is a different coercion entry point than the template substitution this package has to survive
  return `${value}`;
}

/**
 Names what a stub-shaped value turned out to be, without stringifying it.

 @param value - Value produced by a call, a property read, a construction, or an await

 @returns `STUB_IDENTITY`, the deadline marker, or the `typeof` of anything else
 */
function describeStubValue(value: unknown,): string {
  if (value === stub)
    return STUB_IDENTITY;
  if (value === DEADLINE)
    return 'the await deadline, so the value never settled';
  return `a ${typeof value}`;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region Declaration

    describe({
      name: 'declaration',
      children: [
        it({
          name: 'lets a consumer call the stub, chain property reads, and construct it, each of which yields the stub',
          fn: async () => {
            // `noUncheckedIndexedAccess` types every unknown key as possibly undefined, so the chain reads
            // optionally; the runtime answers the same stub either way.
            const called = stub();
            const chained = stub.foo?.bar?.();
            const constructed = new stub();

            expect(describeStubValue(called,),).toBe(STUB_IDENTITY,);
            expect(describeStubValue(chained,),).toBe(STUB_IDENTITY,);
            expect(describeStubValue(constructed,),).toBe(STUB_IDENTITY,);
            expect(describeStubValue(requiredStub,),).toBe(STUB_IDENTITY,);
          },
        },),
        it({
          name: 'types a call, a construction, and a property read as the stub itself instead of unknown',
          fn: async () => {
            expectTypeOf<ReturnType<typeof stub>>().toEqualTypeOf<typeof stub>();
            expectTypeOf<InstanceType<typeof stub>>().toEqualTypeOf<typeof stub>();
            // NonNullable strips the absence `noUncheckedIndexedAccess` adds to an index-signature read, leaving the
            // declared read type this case pins.
            expectTypeOf<NonNullable<typeof stub.foo>>().toEqualTypeOf<typeof stub>();
          },
        },),
      ],
    },),
    //endregion Declaration

    //region Has checks

    describe({
      name: 'has checks',
      children: [
        it({
          name: 'answers true for prototype instead of throwing the proxy invariant TypeError',
          fn: async () => {
            expect('prototype' in stub,).toBe(true,);
          },
        },),
        it({
          name: 'answers true for every non-configurable own key of the wrapped function and false for the others',
          fn: async () => {
            for (const key of Reflect.ownKeys(stub,)) {
              const descriptor = Reflect.getOwnPropertyDescriptor(stub, key,);
              if (descriptor === undefined)
                throw new Error(`The function the stub wraps reports no descriptor for ${String(key,)}, an own key.`,);
              expect(key in stub,).toBe(descriptor.configurable === false,);
            }
          },
        },),
        it({
          name: 'answers false for ordinary property names and for the configurable own keys of the function',
          fn: async () => {
            expect('someMethod' in stub,).toBe(false,);
            expect('length' in stub,).toBe(false,);
            expect('name' in stub,).toBe(false,);
          },
        },),
      ],
    },),
    //endregion Has checks

    //region Get checks

    describe({
      name: 'get checks',
      children: [
        ...AWAITABLE_STUB_EXPRESSIONS.map(function mapExpression(entry,) {
          return it({
            name: `settles when awaited instead of deadlocking: ${entry.label}`,
            fn: async () => {
              const settled = await awaitWithinDeadline(entry.produce(),);
              expect(describeStubValue(settled,),).toBe(STUB_IDENTITY,);
            },
          },);
        },),
        ...PROMISE_MACHINERY_KEYS.map(function mapPromiseKey(key,) {
          return it({
            name: `reads ${key} as absent so the promise machinery skips the stub`,
            fn: async () => {
              expect(typeof stub[key],).toBe('undefined',);
            },
          },);
        },),
        it({
          name: 'answers a primitive coercion with the text the README documents instead of throwing a TypeError',
          fn: async () => {
            const readme = await readFile(new URL('README.md', import.meta.url,), 'utf8',);

            expect(String(stub,),).toBe(PRIMITIVE_TEXT,);
            expect(templateCoerce(stub,),).toBe(PRIMITIVE_TEXT,);
            expect(readme,).toContain(PRIMITIVE_TEXT,);
          },
        },),
      ],
    },),
    //endregion Get checks
  ],
},);
