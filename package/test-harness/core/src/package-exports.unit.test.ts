/** Both runner artifacts retain the ordinary test-author export surface after extraction. @module */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core';

await describe({
  name: 'runner package exports',
  children: ['node', 'neutral',].map(entry => it({
    name: `${entry} preserves the existing runtime exports and forwarded helpers`,
    fn: async () => {
      const harness = entry === 'node'
        ? await import('@monochromatic-dev/test-harness-core')
        : await import('../dist/final/neutral/index.mjs');
      const exportNames = Object.keys(harness,).toSorted();
      expect(exportNames,).toEqual([
        'DEFAULT_CONCURRENCY',
        'caught',
        'createScopedExpect',
        'describe',
        'expect',
        'expectTypeOf',
        'extractAssertionExpression',
        'extractLocationSubstring',
        'formatErrorDeep',
        'formatFailure',
        'isIntegerString',
        'it',
        'readAssertionSites',
      ],);
      harness.expect('forwarded',).toBe('forwarded',);
      harness.expectTypeOf<string>().toEqualTypeOf<string>();
      const [scoped, tracker,] = harness.createScopedExpect();
      scoped(true,).toBeTruthy();
      expect(tracker.count,).toBe(1,);
      expect(harness.isIntegerString('123',),).toBe(true,);
      const diagnostic = await harness.formatFailure({ summary: 'consumer verdict', value: new Error('consumer error',), },);
      expect(diagnostic,).toContain('consumer verdict',);
      expect(diagnostic,).toContain('consumer error',);
    },
  },),),
},);
