/** Built assertion tracking and type-assertion exports. @module */
import { describe, expect as verify, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  createScopedExpect,
  expect,
  expectTypeOf,
  type ScopedExpect,
} from '@monochromatic-dev/test-harness-core-expect';

await describe({
  name: createScopedExpect.name,
  children: [
    it({
      name: 'keeps counters and assertion requirements private to each scope',
      fn: async () => {
        const [scoped, tracker,] = createScopedExpect();
        const [, otherTracker,] = createScopedExpect();
        verify(tracker,).toEqual({ count: 0, requiresAtLeastOne: false, },);
        scoped.assertions(2,);
        scoped.hasAssertions();
        scoped('same',).toBe('same',);
        scoped('different',).not.toBe('same',);
        verify(tracker,).toEqual({ count: 2, expected: 2, requiresAtLeastOne: true, },);
        verify(otherTracker,).toEqual({ count: 0, requiresAtLeastOne: false, },);
        expectTypeOf(scoped,).toEqualTypeOf<ScopedExpect>();
      },
    },),
    it({
      name: 'counts both asynchronous matcher paths',
      fn: async () => {
        const [scoped, tracker,] = createScopedExpect();
        // These legacy matcher calls are the implementation under test.
        await scoped(Promise.resolve('value',),).resolves.toBe('value',);
        const rejected = Promise.reject(new Error('rejected',),);
        await scoped(rejected,).rejects.toThrow('rejected',);
        verify(tracker.count,).toBe(2,);
      },
    },),
    it({
      name: 'counts failed assertions before propagating their error',
      fn: async () => {
        const [scoped, tracker,] = createScopedExpect();
        verify(() => scoped(1,).toBe(2,),).toThrow();
        verify(tracker.count,).toBe(1,);
      },
    },),
    it({
      name: 'retains every asymmetric matcher on the scoped function',
      fn: async () => {
        const [scoped, tracker,] = createScopedExpect();
        for (const key of ['stringContaining', 'stringMatching', 'objectContaining', 'anything', 'any', 'arrayContaining',] as const)
          verify(scoped[key],).toBe(expect[key],);
        verify(tracker.count,).toBe(0,);
      },
    },),
  ],
},);
