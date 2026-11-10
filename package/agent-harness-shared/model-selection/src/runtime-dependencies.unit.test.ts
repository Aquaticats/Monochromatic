/**
 Verifies that the built model-selection entry loads without Pi packages.

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
      name: 'imports and estimates tokens with every Pi runtime dependency rejected',
      fn: async function testRuntimeDependencies(): Promise<void> {
        /**
         Loader boundary rejects direct and transitive Pi package imports.
         */
        const hooks = registerHooks({
          resolve: function resolve(specifier, context, nextResolve) {
            if (specifier.startsWith('@earendil-works/pi-',))
              throw new Error(`Model selection attempted to load Pi dependency: ${specifier}`,);
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
         Public built entry loaded only after the dependency guard is installed.
         */
        const loaded = await import('../dist/final/node/index.mjs');
        expect(loaded.estimateAdvisorInputTokens({
          systemPrompt: 'review carefully',
          contextText: 'hello world',
        },),).toBeGreaterThan(0,);
      },
    },),

    //endregion Runtime dependency boundary
  ],
},);
