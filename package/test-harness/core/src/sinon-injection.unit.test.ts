/** Injected factories retain ownership without changing application-owned destination members. @module */
import { describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core';

await describe({
  name: 'injected sandbox capabilities',
  children: [
    it({
      name: 'injected method factories route by context and reject after completion',
      fn: async (): Promise<void> => {
        const helper = (): string => 'application helper';
        const destination = { helper, };
        const target = { method: (): string => 'original', };
        const getterReads: number[] = [];
        Object.defineProperty(destination, 'computed', {
          enumerable: true,
          get(): () => string {
            getterReads.push(1,);
            return helper;
          },
        },);
        const late: (() => unknown)[] = [];
        await it({ name: 'injection owner', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          /** Runtime checking covers the actual injection namespace rather than inventing a narrowed facade. */
          const inject: unknown = Reflect.get(sinon, 'inject',);
          if ((typeof inject) !== 'function')
            throw new Error('Sandbox did not provide inject',);
          expect(Reflect.apply(inject, sinon, [destination,],),).toBe(destination,);
          const stub: unknown = Reflect.get(destination, 'stub',);
          if ((typeof stub) !== 'function')
            throw new Error('Sinon did not inject stub',);
          const fake: unknown = Reflect.apply(stub, destination, [target, 'method',],);
          expect(Reflect.get(target, 'method',),).toBe(fake,);
          expect(target.method(),).toBeUndefined();
          await it({ name: 'uninjected reader', fn: async (): Promise<void> => {
            expect(target.method(),).toBe('original',);
          }, },);
          late.push(() => Reflect.apply(stub, destination, [target, 'method',],),);
        }, },);
        for (const mutate of late)
          expect(mutate,).toThrow('completed',);
        expect(target.method(),).toBe('original',);
        expect(destination.helper,).toBe(helper,);
        expect(destination.helper(),).toBe('application helper',);
        expect(getterReads.length,).toBe(0,);
      },
    },),
    it({
      name: 'a restored mock controller cannot restore a newer contextual replacement',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { method: (): string => 'original', other: (): string => 'other', };
        const mock = sinon.mock(target,);
        mock.expects('method',).once();
        target.method();
        mock.verify();
        sinon.stub(target, 'method',).returns('new generation',);
        mock.restore();
        expect(target.method(),).toBe('new generation',);
        expect(() => mock.expects('other',),).toThrow('restored',);
      },
    },),
    it({
      name: 'verifyAndRestore releases contextual methods even when ordinary mock verification fails',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { method: (): string => 'original', expected: (): string => 'expected', };
        sinon.stub(target, 'method',);
        sinon.mock(target,).expects('expected',).once();
        expect(() => sinon.verifyAndRestore(),).toThrow();
        expect(target.method(),).toBe('original',);
        expect(target.expected(),).toBe('expected',);
      },
    },),
  ],
},);
