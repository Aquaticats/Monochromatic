/** Public source and built entry points must share ownership in one Node realm. @module */
import { AsyncResource, } from 'node:async_hooks';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core';
// oxlint-disable-next-line test-import/require-eventual-artifact -- Intentional source/artifact interoperability; rationale in doc/troubleshooting/sinon-context-owned-stubs.md.
import { it as sourceIt, } from '@monochromatic-dev/test-harness-core/ts';
import { it as neutralIt, } from '../dist/final/neutral/index.mjs';

/** Created before any test executes, so callbacks deliberately have no test owner. */
const contextless = new AsyncResource('module-test pre-existing consumer',);
/** Dispose the synthetic resource after every overlapping attempt has settled. */
using resourceCleanup = {
  [Symbol.dispose](): void {
    contextless.emitDestroy();
  },
};

await describe({
  name: 'shared harness copies',
  children: [{ label: 'source', run: sourceIt, }, { label: 'neutral', run: neutralIt, },].flatMap(({ label, run: otherIt, },) => ([0, 1,] as const).map(firstFinisher => it({
    name: `${label} and Node mixed writers restore with owner ${String(firstFinisher,)} finishing first`,
    fn: async (): Promise<void> => {
      const target = {
        method(input: string,): string {
          return `original:${input}`;
        },
      };
      const original = Object.getOwnPropertyDescriptor(target, 'method',);
      const ready = [Promise.withResolvers<void>(), Promise.withResolvers<void>(),];
      const finished = [Promise.withResolvers<void>(), Promise.withResolvers<void>(),];
        /** Await each public descriptor directly: descriptor-private scheduling symbols are not cross-copy APIs. */
      const settled = await Promise.allSettled([it, otherIt,].map(async (run, index,): Promise<void> => {
        const signal = ready[index];
        const completion = finished[index];
        const first = finished[firstFinisher];
        if ((signal === undefined) || (completion === undefined) || (first === undefined))
          throw new Error('Missing deterministic owner barrier',);
        using completed = {
          [Symbol.dispose](): void {
            completion.resolve();
          },
        };
        await run({ name: `copy owner ${String(index,)}`, fn: async ({ sinon, }: TestContext,): Promise<void> => {
          using readyOnFailure = {
            [Symbol.dispose](): void {
              signal.resolve();
            },
          };
            /** The companion entry spies while the Node bundle stubs the same original function. */
          const fake = index === 0 ? sinon.stub(target, 'method',).returns('bundled',) : sinon.spy(target, 'method',);
          signal.resolve();
          await Promise.all(ready.map(entry => entry.promise),);
          expect(target.method('promise',),).toBe(index === 0 ? 'bundled' : 'original:promise',);
          await wait(1,);
          expect(target.method('timer',),).toBe(index === 0 ? 'bundled' : 'original:timer',);
          contextless.runInAsyncScope(() => {
            expect(target.method('contextless',),).toBe('original:contextless',);
          },);
          await neutralIt({ name: 'neutral unstubbed reader', fn: async (): Promise<void> => {
            expect(target.method('neutral',),).toBe('original:neutral',);
          }, },);
          if (index !== firstFinisher) {
            await first.promise;
            expect(Reflect.get(target, 'method',),).toBe(fake,);
            expect(typeof Object.getOwnPropertyDescriptor(target, 'method',)?.get,).toBe('function',);
          }
          expect(fake.callCount,).toBe(2,);
        }, },);
      },),);
      expect(settled.map(result => result.status),).toEqual(['fulfilled', 'fulfilled',],);
      expect(Object.getOwnPropertyDescriptor(target, 'method',),).toEqual(original,);
    },
  },)),),
},);
