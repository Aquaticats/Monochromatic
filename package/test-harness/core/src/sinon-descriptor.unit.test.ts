/** Descriptor conflicts and unsupported mutation families through the public harness. @module */
import { createSandbox, } from 'sinon';
import { describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core';

/**
 Captures a deliberately failing nested test without hiding unexpected success.
 @param descriptor - nested attempt whose restoration should fail
 @returns reported test error
 @example
 ```ts
 const failure = await capturedFailure(it({ name: 'conflict', fn }));
 ```
 */
async function capturedFailure(descriptor: PromiseLike<unknown>,): Promise<Error> {
  /** Capturing outside the catch keeps successful completion distinguishable from failure. */
  const failures: unknown[] = [];
  try {
    await descriptor;
  }
  catch (error) {
    failures.push(error,);
  }
  /** Expected failures are reported as Errors by the public harness. */
  const [failure,] = failures;
  if (!(Error.isError(failure,)))
    throw new Error('Nested conflict fixture unexpectedly succeeded',);
  return failure;
}

await describe({
  name: 'context-owned descriptor boundaries',
  children: [
    it({
      name: 'other mutation APIs reject before overwriting an active contextual slot',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { method: (): string => 'original', other: (): string => 'other', };
        const other = Object.getOwnPropertyDescriptor(target, 'other',);
        const fake = sinon.stub(target, 'method',).returns('owned',);
        expect(() => {
          target.method = () => 'assigned';
        },).toThrow('Assignment',);
        expect(() => sinon.stub(target,),).toThrow('context-owned',);
        expect(() => sinon.spy(target,),).toThrow('context-owned',);
        expect(() => sinon.replace(target, 'method', () => 'replacement',),).toThrow('context-owned',);
        expect(() => sinon.replaceGetter(target, 'method', () => () => 'getter',),).toThrow('context-owned',);
        expect(() => sinon.replaceSetter(target, 'method', () => {},),).toThrow('context-owned',);
        expect(() => sinon.define(target, 'method', () => 'defined',),).toThrow('context-owned',);
        expect(() => sinon.spy(target, 'method', ['get',],),).toThrow('context-owned',);
        const mock = sinon.mock(target,);
        expect(() => mock.expects('method',),).toThrow('context-owned',);
        expect(Reflect.get(target, 'method',),).toBe(fake,);
        expect(Object.getOwnPropertyDescriptor(target, 'other',),).toEqual(other,);
      },
    },),
    it({
      name: 'inherited, copied, and proxy aliases report the alias without claiming descriptor damage',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { method: (): string => 'original', };
        sinon.stub(target, 'method',);
        const inherited = { method: (): string => 'placeholder', };
        Object.setPrototypeOf(inherited, target,);
        Reflect.deleteProperty(inherited, 'method',);
        const copied = { method: (): string => 'placeholder', };
        const descriptor = Object.getOwnPropertyDescriptor(target, 'method',);
        if (descriptor === undefined)
          throw new Error('Contextual descriptor was not installed',);
        Object.defineProperty(copied, 'method', descriptor,);
        const proxy = new Proxy(target, {},);
        for (const alias of [inherited, copied, proxy,])
          expect(() => sinon.stub(alias, 'method',),).toThrow('alias',);
      },
    },),
    it({
      name: 'deletion and redefinition are both preserved and reported during cleanup',
      fn: async (): Promise<void> => {
        const target = { deleted: (): string => 'deleted', redefined: (): string => 'redefined', };
        const foreign = { value: (): string => 'foreign', enumerable: false, configurable: true, writable: false, };
        const failure = await capturedFailure(it({ name: 'externally changed slots', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          sinon.stub(target, 'deleted',);
          sinon.stub(target, 'redefined',);
          Reflect.deleteProperty(target, 'deleted',);
          Object.defineProperty(target, 'redefined', foreign,);
        }, },),);
        expect(failure.cause,).toBeInstanceOf(AggregateError,);
        if (!(failure.cause instanceof AggregateError))
          throw new Error('Expected independent cleanup failures',);
        expect(failure.cause.errors.length,).toBe(2,);
        expect(Object.hasOwn(target, 'deleted',),).toBe(false,);
        expect(Object.getOwnPropertyDescriptor(target, 'redefined',),).toEqual(foreign,);
        /** Failed cleanup must not leave stale registry ownership on a subsequently repaired target. */
        Object.defineProperty(target, 'deleted', { configurable: true, writable: true, value: (): string => 'repaired', },);
        await it({ name: 'owner after external repair', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          sinon.stub(target, 'deleted',).returns('new owner',);
          expect(target.deleted(),).toBe('new owner',);
        }, },);
        expect(target.deleted(),).toBe('repaired',);
      },
    },),
    it({
      name: 'direct-import Sinon descriptor changes are preserved instead of silently overwritten',
      fn: async (): Promise<void> => {
        const target = { method: (): string => 'original', };
        const external = createSandbox();
        const foreign = (): string => 'external';
        using cleanup = {
          [Symbol.dispose](): void {
            external.restore();
          },
        };
        await capturedFailure(it({ name: 'direct replacement conflict', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          sinon.stub(target, 'method',);
          external.stub(target, 'method',).value(foreign,);
        }, },),);
        expect(Reflect.get(target, 'method',),).toBe(foreign,);
      },
    },),
    it({
      name: 'same-owner double wrapping fails without losing the first replacement',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { method: (): string => 'original', };
        const first = sinon.stub(target, 'method',).returns('first',);
        expect(() => sinon.stub(target, 'method',),).toThrow('already wrapped',);
        expect(Reflect.get(target, 'method',),).toBe(first,);
        expect(target.method(),).toBe('first',);
      },
    },),
  ],
},);
