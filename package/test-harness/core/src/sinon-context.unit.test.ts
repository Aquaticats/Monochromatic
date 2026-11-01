/** Context-selected methods through the built public harness. @module */
import { describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core';

await describe({
  name: 'context-owned methods',
  children: [
    ...(['stub', 'spy',] as const).map(factory => it({
      name: `concurrent ${factory} owners keep independent histories and an original reader`,
      fn: async (): Promise<void> => {
        /** Object identity and arguments exercise the actual consumer call path. */
        const target = {
          label: 'original',
          method(input: string,): string {
            return `${this.label}:${input}`;
          },
        };
        /** Exact final descriptor comparison catches incomplete restoration. */
        const original = Object.getOwnPropertyDescriptor(target, 'method',);
        /** Every owner reaches the barrier while its fake remains installed. */
        const ready = [Promise.withResolvers<void>(), Promise.withResolvers<void>(),];
        /** An unstubbed reader must complete before either owner is released. */
        const reader = Promise.withResolvers<void>();
        await describe({
          name: 'overlapping siblings',
          children: [
            ...ready.map((signal, index,) => describe({
              name: `sibling ${String(index,)}`, concurrency: 1,
              children: [it({
                name: 'method owner',
                fn: async ({ sinon, }: TestContext,): Promise<void> => {
                  // The failure control must settle even when the second factory throws before the barrier.
                  using releaseOnFailure = {
                    [Symbol.dispose](): void {
                      signal.resolve();
                    },
                  };
                  /** Each context owns an ordinary Sinon fake with its own history. */
                  const fake = factory === 'stub'
                    ? sinon.stub(target, 'method',).returns(`fake ${String(index,)}`,)
                    : sinon.spy(target, 'method',);
                  signal.resolve();
                  await Promise.all(ready.map(entry => entry.promise),);
                  await reader.promise;
                  expect(Reflect.get(target, 'method'),).toBe(fake,);
                  expect(target.method('input',),).toBe(factory === 'stub' ? `fake ${String(index,)}` : 'original:input',);
                  expect(fake.callCount,).toBe(1,);
                  expect(fake.firstCall.thisValue,).toBe(target,);
                  fake.restore();
                  expect(target.method('restored',),).toBe('original:restored',);
                },
              },),],
            },)),
            it({
              name: 'unstubbed reader',
              fn: async (): Promise<void> => {
                await Promise.all(ready.map(entry => entry.promise),);
                using release = {
                  [Symbol.dispose](): void {
                    reader.resolve();
                  },
                };
                expect(target.method('reader',),).toBe('original:reader',);
              },
            },),
          ],
        },);
        expect(Object.getOwnPropertyDescriptor(target, 'method',),).toEqual(original,);
      },
    },)),
    it({
      name: 'nested tests override ownership while unstubbed nested tests see the original',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        /** Outer and nested attempts deliberately share only this fixture. */
        const target = { method: (): string => 'original', };
        sinon.stub(target, 'method',).returns('parent',);
        await it({ name: 'nested reader', fn: async (): Promise<void> => {
          expect(target.method(),).toBe('original',);
        }, },);
        await it({ name: 'nested writer', fn: async ({ sinon: child, }: TestContext,): Promise<void> => {
          child.stub(target, 'method',).returns('child',);
          expect(target.method(),).toBe('child',);
        }, },);
        expect(target.method(),).toBe('parent',);
      },
    },),
  ],
},);
