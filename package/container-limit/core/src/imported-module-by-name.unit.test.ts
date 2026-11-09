import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { importedModuleByName, } from './imported-module-by-name.ts';

/**
 The imports of a fixture text, read as TypeScript.
 */
function importedFrom(text: string,): ReadonlyMap<string, string> {
  return importedModuleByName({
    text,
    language: 'typescript',
  },);
}

await describe({
  name: importedModuleByName.name,
  children: [
    it({
      name: 'reads the names of a multi-line clause and the module its from names',
      fn: async () => {
        expect(importedFrom("import {\n  buildBounds,\n  targets,\n} from './fuzz-constants.ts';",),).toEqual(
          new Map([
            ['buildBounds', './fuzz-constants.ts',],
            ['targets', './fuzz-constants.ts',],
          ],),
        );
      },
    },),
    it({
      name: 'reads a default import and a renamed one',
      fn: async () => {
        expect(importedFrom("import run, { type Limit as Bound } from '../x.ts';",),).toEqual(
          new Map([
            ['run', '../x.ts',],
            ['Limit', '../x.ts',],
            ['Bound', '../x.ts',],
          ],),
        );
      },
    },),
    it({
      name: 'binds nothing for an import with no clause',
      fn: async () => {
        expect(importedFrom("import './x.ts';",).size,).toBe(0,);
      },
    },),
    it({
      name: 'keeps a package specifier as it stands',
      fn: async () => {
        expect(importedFrom("import { join } from 'node:path';",).get('join',),).toBe('node:path',);
      },
    },),
    it({
      name: 'binds nothing for a commented-out import',
      fn: async () => {
        expect(importedFrom("// import { a } from './x.ts';",).size,).toBe(0,);
      },
    },),
    it({
      name: 'reads the imports of a TOML node script',
      fn: async () => {
        expect(importedModuleByName({
          text: 'x = """\nimport { a } from \'./x.ts\'\n"""',
          language: 'toml',
        },).get('a',),).toBe('./x.ts',);
      },
    },),
  ],
},);
