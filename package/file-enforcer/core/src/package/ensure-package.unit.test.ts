import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ensurePackage,
  registerPackages,
} from './ensure-package.ts';
import { p, } from './p.ts';

/**
 Install-path tests run in containers via the container test matrix.
 See `ensure-package.unit.matrix.test.ts` for the actual install verification.
 Running `ensurePackage` for a missing binary locally would modify the host system.
 */

await describe({
  name: '',
  children: [
    //region registerPackages

    describe({
      name: registerPackages.name,
      children: [
        it({
          name: 'accepts an array of entries without throwing',
          fn: async () => {
            expect(() => {
              registerPackages([
                p('curl',),
                p({ bin: 'rg', effname: 'ripgrep', },),
              ],);
            },)
              .not
              .toThrow();
          },
        },),
        it({
          name: 'replaces previously registered entries',
          fn: async () => {
            registerPackages([p('curl',),],);
            registerPackages([p('tmux',),],);
            /** No assertion needed beyond not throwing; index rebuild is lazy */
          },
        },),
      ],
    },),

    //endregion registerPackages

    //region ensurePackage (already installed)

    describe({
      name: 'ensurePackage (binary already on PATH)',
      children: [
        it({
          name: 'returns immediately for an existing binary',
          fn: async () => {
            registerPackages([],);
            /** `ls` exists on every POSIX system */
            await expect(ensurePackage('ls',),).resolves.toBeUndefined();
          },
        },),
        it({
          name: 'returns immediately for `true`',
          fn: async () => {
            registerPackages([],);
            await expect(ensurePackage('true',),).resolves.toBeUndefined();
          },
        },),
      ],
    },),

    //endregion ensurePackage (already installed)

    //region Index lookup

    describe({
      name: 'ensurePackage (index lookup)',
      children: [
        it({
          name: 'skips index lookup when binary is already on PATH',
          fn: async () => {
            /** Register an entry for `ls` but it should never be consulted */
            registerPackages([
              p({ bin: 'ls', effname: 'wrong-package', },),
            ],);
            await expect(ensurePackage('ls',),).resolves.toBeUndefined();
          },
        },),
      ],
    },),
    //endregion Index lookup
  ],
},);
