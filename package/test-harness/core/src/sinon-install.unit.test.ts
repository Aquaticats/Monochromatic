/** Factory failure must not leave adapter descriptors or unregistered partial fakes. @module */
import { describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core';
import { createSandbox, } from 'sinon';

await describe({
  name: 'Sinon installation failure',
  children: [
    it({
      name: 'failed contextual construction preserves an existing direct-Sinon wrapper immediately',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const external = createSandbox();
        using cleanup = { [Symbol.dispose]: (): void => external.restore(), };
        const target = { method: (): string => 'original', };
        external.stub(target, 'method',).returns('external',);
        const before = Object.getOwnPropertyDescriptor(target, 'method',);
        expect(() => sinon.stub(target, 'method',),).toThrow('already wrapped',);
        expect(Object.getOwnPropertyDescriptor(target, 'method',),).toEqual(before,);
        expect(target.method(),).toBe('external',);
        external.restore();
        sinon.stub(target, 'method',).returns('new owner',);
        expect(target.method(),).toBe('new owner',);
      },
    },),
    it({
      name: 'partial injection guards factories before an application setter retains them',
      fn: async (): Promise<void> => {
        const late: (() => unknown)[] = [];
        await it({ name: 'failed injection owner', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          const captured: unknown[] = [];
          const destination = {};
          Object.defineProperty(destination, 'spy', {
            configurable: true,
            get(): unknown {
              return captured.at(-1,);
            },
            set(this: unknown, factory: unknown): void {
              expect(this,).toBe(destination,);
              captured.push(factory,);
            },
          },);
          Object.defineProperty(destination, 'stub', { value: undefined, writable: false, },);
          const inject: unknown = Reflect.get(sinon, 'inject',);
          if ((typeof inject) !== 'function')
            throw new Error('Missing Sinon inject factory',);
          expect((): void => {
            Reflect.apply(inject, sinon, [destination,],);
          },).toThrow('stub',);
          const [factory,] = captured;
          if ((typeof factory) !== 'function')
            throw new Error('Injection did not expose its first factory',);
          const target = { method: (): string => 'original', };
          Reflect.apply(factory, destination, [target, 'method',],);
          expect(target.method(),).toBe('original',);
          late.push(() => Reflect.apply(factory, destination, [target, 'method',],),);
        }, },);
        for (const invoke of late)
          expect(invoke,).toThrow('completed',);
      },
    },),
    it({
      name: 'partial rollback retains construction and cleanup errors while restoring other members',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const constructionError = new Error('construction stopped',);
        const target = {
          first: (): string => 'first',
          witness: (): string => 'witness',
          second: (): string => 'second',
        };
        const witness = Object.getOwnPropertyDescriptor(target, 'witness',);
        Object.defineProperty(target.second, 'restore', {
          get(): never {
            Object.defineProperty(target, 'first', { configurable: false, writable: false, },);
            throw constructionError;
          },
        },);
        const failures: unknown[] = [];
        try {
          sinon.stub(target,);
        }
        catch (error) {
          failures.push(error,);
        }
        const [failure,] = failures;
        if (!(failure instanceof AggregateError))
          throw new Error('Expected construction and rollback aggregate', { cause: failure, },);
        const errors: readonly unknown[] = failure.errors;
        expect(errors[0],).toBe(constructionError,);
        expect(errors[1],).toBeInstanceOf(AggregateError,);
        expect(Object.getOwnPropertyDescriptor(target, 'witness',),).toEqual(witness,);
        expect(target.witness(),).toBe('witness',);
      },
    },),
    ...(['stub', 'spy',] as const).map(operation => it({
      name: `failed whole-object ${operation} restores only newly introduced descriptors`,
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const external = createSandbox();
        using cleanup = { [Symbol.dispose]: (): void => external.restore(), };
        const other = { method: (): string => 'external', };
        const target = { first: (): string => 'original', second: external.stub(other, 'method',), };
        const independent = { method: (): string => 'untouched', };
        sinon.stub(independent, 'method',).returns('existing replacement',);
        const before = Object.getOwnPropertyDescriptors(target,);
        expect(() => operation === 'stub' ? sinon.stub(target,) : sinon.spy(target,),).toThrow('already wrapped',);
        expect(Object.getOwnPropertyDescriptors(target,),).toEqual(before,);
        expect(target.first(),).toBe('original',);
        expect(independent.method(),).toBe('existing replacement',);
        sinon.stub(target, 'first',).returns('fresh',);
        expect(target.first(),).toBe('fresh',);
      },
    },)),
  ],
},);
