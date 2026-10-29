import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { resolveSpecifier, } from './resolve.ts';

await describe({
  name: resolveSpecifier.name,
  children: [
    //region Node built-in resolution

    it({
      name: 'resolves node: prefixed built-in modules',
      fn: async () => {
        const result = await resolveSpecifier({ specifier: 'node:path', },);
        expect(result,).toBe('node:path',);
      },
    },),

    it({
      name: 'resolves bare built-in module names',
      fn: async () => {
        const result = await resolveSpecifier({ specifier: 'path', },);
        expect(result,).toBe('path',);
      },
    },),

    //endregion Node built-in resolution

    //region Installed package resolution

    it({
      name: 'resolves a package installed in the workspace',
      fn: async () => {
        const result = await resolveSpecifier({ specifier: 'nano-spawn', },);
        expect(result,).toContain('nano-spawn',);
      },
    },),

    //endregion Installed package resolution

    //region Error cases

    it({
      name: 'throws for a specifier that does not exist anywhere',
      fn: async () => {
        await expect(
          resolveSpecifier({
            specifier: 'this-package-definitely-does-not-exist-anywhere-12345',
          },),
        )
          .rejects
          .toThrow('Cannot resolve',);
      },
    },),

    it({
      name: 'includes search locations in error message',
      fn: async () => {
        await expect(resolveSpecifier({ specifier: 'nonexistent-pkg-98765', },),)
          .rejects
          .toThrow('CWD',);
      },
    },),
    //endregion Error cases
  ],
},);
