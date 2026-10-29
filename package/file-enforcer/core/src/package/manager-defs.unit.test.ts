import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  MANAGER_KEYS,
  MANAGERS,
  type PackageManager,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region MANAGERS ordering

    describe({
      name: 'MANAGERS',
      children: [
        it({
          name: 'has brew as the first entry (preferred on immutable distros)',
          fn: async () => {
            const [first,] = MANAGERS.keys();
            expect(first,).toBe('brew',);
          },
        },),
        it({
          name: 'contains every PackageManager variant',
          fn: async () => {
            /** All values from the PackageManager union */
            const expected: readonly PackageManager[] = [
              'apk',
              'apt',
              'brew',
              'choco',
              'dnf',
              'pacman',
              'scoop',
              'zypper',
            ];
            for (const name of expected)
              expect(MANAGERS.has(name,),).toBe(true,);
            expect(MANAGERS.size,).toBe(expected.length,);
          },
        },),
        it({
          name: 'every entry has non-empty check, search, and install arrays',
          fn: async () => {
            for (const [name, def,] of MANAGERS) {
              expect(
                def.check.length,
              )
                .toBeGreaterThan(
                0,
                );
              expect(
                def.search.length,
              )
                .toBeGreaterThan(
                0,
                );
              expect(
                def.install.length,
              )
                .toBeGreaterThan(
                0,
                );
              /** install template must contain {pkg} placeholder */
              expect(def.install.includes('{pkg}',),).toBe(true,);
              /** search template must contain {pkg} placeholder */
              expect(def.search.includes('{pkg}',),).toBe(true,);
            }
          },
        },),
      ],
    },),

    //endregion MANAGERS ordering

    //region MANAGER_KEYS

    describe({
      name: 'MANAGER_KEYS',
      children: [
        it({
          name: 'is derived from MANAGERS and has the same size',
          fn: async () => {
            expect(MANAGER_KEYS.size,).toBe(MANAGERS.size,);
          },
        },),
        it({
          name: 'contains every key from MANAGERS',
          fn: async () => {
            for (const name of MANAGERS.keys())
              expect(MANAGER_KEYS.has(name,),).toBe(true,);
          },
        },),
      ],
    },),
    //endregion MANAGER_KEYS
  ],
},);
