/**
 Verifies that the built DSH adapter loads without Pi's coding SDK or its lockfile package.

 @module
 */

import { registerHooks, } from 'node:module';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

await describe({
  name: '',
  children: [
    //region Runtime dependency boundary

    it({
      name: 'loads the plugin and estimates context without the Pi coding SDK or proper-lockfile',
      fn: async function testRuntimeDependencies() {
        /**
         Guard the complete runtime import graph, including host dependencies.
         */
        const hooks = registerHooks({
          resolve: function resolve(specifier, context, nextResolve) {
            if (specifier.startsWith('@earendil-works/pi-coding-agent',) || (specifier === 'proper-lockfile'))
              throw new Error(`DSH Advisor attempted to load an unrelated runtime dependency: ${specifier}`,);
            return nextResolve(specifier, context,);
          },
        },);
        /**
         Restore module loading even when the import or assertion throws.
         */
        using cleanup = {
          [Symbol.dispose]: function disposeHooks() {
            hooks.deregister();
          },
        };
        /**
         Built plugin entry imported after forbidden dependencies become unavailable.
         */
        const loaded = await import('../dist/final/node/index.mjs');
        expect(loaded.apply,).toBeTypeOf('function',);
        expect(loaded.dshAdvisorTranscriptSerialization.estimateInputTokens({
          systemPrompt: 'review carefully',
          contextText: 'hello world',
        },),).toBeGreaterThan(0,);
      },
    },),

    //endregion Runtime dependency boundary
  ],
},);
