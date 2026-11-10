import { createRequire, } from 'node:module';

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

            expect(called,).toBe(stub,);
            expect(chained,).toBe(stub,);
            expect(constructed,).toBe(stub,);
            expect(requiredStub,).toBe(stub,);
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
  ],
},);
