/** Descriptor authority on public behavior selectors and collection members. @module */
import { describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core';
import type { SinonStub, } from 'sinon';

/** Ordinary object shape shared by the supported and fallback probes. */
type Target = { toString: () => string; };

/** Produce the distinct public fake-returning operation families. */
function createReplacement({ sinon, mode, }: {
  readonly sinon: TestContext['sinon'];
  readonly mode: 'method' | 'object' | 'function-object' | 'instance';
},): { target: Target; fake: SinonStub; } {
  function functionTarget(): void {}
  if (mode === 'instance') {
    const target = sinon.createStubInstance(Date,);
    return { target, fake: target.toString.returns('original',), };
  }
  if (mode === 'function-object') {
    functionTarget.toString = (): string => 'original';
    return { target: functionTarget, fake: sinon.stub(functionTarget,).toString.returns('original',), };
  }
  const target = { toString: (): string => 'original', };
  return { target, fake: mode === 'object' ? sinon.stub(target,).toString : sinon.stub(target, 'toString',), };
}

await describe({
  name: 'returned Sinon descriptor authority',
  children: [
    ...(['onCall', 'onFirstCall', 'onSecondCall', 'onThirdCall',] as const).map(selector => it({
      name: `${selector} preserves private configuration and rejects setter conversion`,
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { method: (): string => 'original', };
        const fake = sinon.stub(target, 'method',);
        const behavior = fake[selector](0,);
        behavior.value(() => 'private',);
        expect(target.method(),).toBe('private',);
        await it({ name: 'unowned reader', fn: async (): Promise<void> => {
          expect(target.method(),).toBe('original',);
        }, },);
        expect(() => behavior.set(() => {},),).toThrow('context-owned data-method',);
        expect(target.method(),).toBe('private',);
        fake.restore();
        sinon.stub(target, 'method',).returns('fresh',);
        expect(() => behavior.value(() => 'stale',),).toThrow('restored',);
        expect(target.method(),).toBe('fresh',);
      },
    },)),
    ...(['method', 'object', 'function-object', 'instance',] as const).map(mode => it({
      name: `${mode} members and behavior objects cannot mutate a later generation`,
      fn: async (): Promise<void> => {
        const entries: { target: Target; fake: SinonStub; behavior: SinonStub; }[] = [];
        await it({ name: 'old owner', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          const replacement = createReplacement({ sinon, mode, },);
          replacement.fake.returns('owned',);
          expect(replacement.target.toString(),).toBe('owned',);
          entries.push({ ...replacement, behavior: replacement.fake.onCall(0,), },);
        }, },);
        const [entry,] = entries;
        if (entry === undefined)
          throw new Error('Missing completed replacement',);
        await it({ name: 'new owner', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          sinon.stub(entry.target, 'toString',).returns('fresh',);
          const installed = Object.getOwnPropertyDescriptor(entry.target, 'toString',);
          for (const controller of [entry.fake, entry.behavior,]) {
            for (const operation of ['get', 'set', 'value',] as const) {
              expect(() => controller[operation]((): void => {},),).toThrow('completed',);
              expect(Object.getOwnPropertyDescriptor(entry.target, 'toString',),).toEqual(installed,);
            }
          }
          entry.fake.restore();
          expect(entry.target.toString(),).toBe('fresh',);
          expect(Object.getOwnPropertyDescriptor(entry.target, 'toString',),).toEqual(installed,);
          entry.fake.onCall(entry.fake.callCount,).returns('local',);
          expect(entry.fake(),).toBe('local',);
        }, },);
      },
    },)),
  ],
},);
