/** Fake timers retain ordinary semantics but cannot restore across completed attempts. @module */
import { describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core';

await describe({
  name: 'ordinary fake timer ownership',
  concurrency: 1,
  children: [
    it({
      name: 'a retained clock cannot uninstall a later attempts timer replacement',
      fn: async (): Promise<void> => {
        /** Each nested attempt supplies a distinct controller. */
        const clocks: TestContext['sinon']['clock'][] = [];
        /** Original host method must survive both attempts' cleanup. */
        const original = globalThis.setTimeout;
        await it({
          name: 'first clock',
          fn: async ({ sinon, }: TestContext,): Promise<void> => {
            clocks.push(sinon.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout',], },),);
          },
        },);
        expect(globalThis.setTimeout,).toBe(original,);
        await it({
          name: 'second clock',
          fn: async ({ sinon, }: TestContext,): Promise<void> => {
            /** Changing only these timers avoids faking the harness deadline clock. */
            const clock = sinon.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout',], },);
            /** Capture the actual host replacement before invoking old cleanup. */
            const installed = globalThis.setTimeout;
            /** The first attempt completed before this controller was installed. */
            const [previous,] = clocks;
            if (previous === undefined)
              throw new Error('First attempt did not provide a clock',);
            /** Sinon exposes uninstall at runtime although its sandbox clock type lists restore. */
            const uninstall: unknown = Reflect.get(previous, 'uninstall',);
            if ((typeof uninstall) !== 'function')
              throw new Error('Sinon clock did not provide uninstall',);
            Reflect.apply(uninstall, previous, [],);
            previous.restore();
            expect(globalThis.setTimeout,).toBe(installed,);
            /** Local timers still execute under ordinary Sinon clock semantics. */
            const timerSpy = sinon.spy((): void => {},);
            setTimeout(timerSpy, 1,);
            clock.tick(1,);
            expect(timerSpy.callCount,).toBe(1,);
          },
        },);
        expect(globalThis.setTimeout,).toBe(original,);
      },
    },),
    it({
      name: 'manual clock restoration retires that installation within an active attempt',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const first = sinon.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout',], },);
        first.restore();
        sinon.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout',], },);
        const installed = globalThis.setTimeout;
        first.restore();
        expect(globalThis.setTimeout,).toBe(installed,);
        const setTickMode: unknown = Reflect.get(first, 'setTickMode',);
        if ((typeof setTickMode) !== 'function')
          throw new Error('Sinon clock did not provide setTickMode',);
        expect((): void => {
          Reflect.apply(setTickMode, first, [{ mode: 'nextAsync', },],);
        },).toThrow('uninstalled',);
      },
    },),
    it({
      name: 'fake timer installation refuses to overwrite active contextual global methods',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        /** The target is process-global but disposable sandbox cleanup restores it in this test process. */
        sinon.stub(globalThis, 'setTimeout',);
        expect(() => sinon.useFakeTimers({ toFake: ['setTimeout',], },),).toThrow('context-owned',);
      },
    },),
  ],
},);
