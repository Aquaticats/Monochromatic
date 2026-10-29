import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  binaryExists,
  detectManager,
  isRoot,
  NO_PACKAGE_MANAGER,
  resetManagerCache,
  resetRootCache,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region detectManager

    describe({
      name: detectManager.name,
      children: [
        it({
          name: 'returns a real manager on a standard system',
          fn: async () => {
            resetManagerCache();
            const manager = await detectManager();
            /** Every CI and dev machine has at least one package manager */
            expect(manager,).not.toBe(NO_PACKAGE_MANAGER,);
          },
        },),
        it({
          name: 'caches the result across calls',
          fn: async () => {
            resetManagerCache();
            const first = await detectManager();
            const second = await detectManager();
            expect(first,).toBe(second,);
          },
        },),
      ],
    },),

    //endregion detectManager

    //region binaryExists

    describe({
      name: binaryExists.name,
      children: [
        it({
          name: 'returns true for a binary that exists',
          fn: async () => {
            /** `ls` exists on every POSIX system */
            const exists = await binaryExists({ binary: 'ls', },);
            expect(exists,).toBe(true,);
          },
        },),
        it({
          name: 'returns false for a binary that does not exist',
          fn: async () => {
            const exists = await binaryExists({
              binary: 'nonexistent-binary-that-should-not-exist-42',
            },);
            expect(exists,).toBe(false,);
          },
        },),
      ],
    },),

    //endregion binaryExists

    //region isRoot

    describe({
      name: isRoot.name,
      children: [
        it({
          name: 'returns a boolean',
          fn: async () => {
            resetRootCache();
            const result = isRoot();
            expect(typeof result,).toBe('boolean',);
          },
        },),
        it({
          name: 'caches the result across calls',
          fn: async () => {
            resetRootCache();
            const first = isRoot();
            const second = isRoot();
            expect(first,).toBe(second,);
          },
        },),
      ],
    },),
    //endregion isRoot
  ],
},);
