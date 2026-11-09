import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { definedTextByName, } from './defined-text-in-source-text.ts';

await describe({
  name: definedTextByName.name,
  children: [
    it({
      name: 'reads a binding past its type annotation',
      fn: async () => {
        expect(definedTextByName({
          blankedSourceText: 'const bounds: readonly string[] = [1];',
        },).get('bounds',),).toBe('[1]',);
      },
    },),
    it({
      name: 'reads a builder function body',
      fn: async () => {
        expect(definedTextByName({
          blankedSourceText: 'function build({ repoRoot }: { readonly repoRoot: string }): readonly string[] { return [repoRoot]; }',
        },).get('build',),).toBe('return [repoRoot];',);
      },
    },),
    it({
      name: 'reads a generator function body',
      fn: async () => {
        expect(definedTextByName({
          blankedSourceText: 'function* names() { yield 1; }',
        },).get('names',),).toBe('yield 1;',);
      },
    },),
    it({
      name: 'reads every binding of a file, later ones included, and follows a multi-line initializer',
      fn: async () => {
        const definitions = definedTextByName({
          blankedSourceText: 'const a = 1\nconst b = [\n  1,\n]\n',
        },);
        expect([...definitions.keys(),],).toEqual(['a', 'b',],);
        expect(definitions.get('b',),).toBe('[\n  1,\n]',);
      },
    },),
    it({
      name: 'binds nothing for a use of a name, a destructuring or a parameter',
      fn: async () => {
        const definitions = definedTextByName({
          blankedSourceText: 'const { a } = b(c)\nfunction f(g) { return g }',
        },);
        expect(definitions.has('a',),).toBe(false,);
        expect(definitions.has('g',),).toBe(false,);
        expect([...definitions.keys(),],).toEqual(['f',],);
      },
    },),
    it({
      name: 'keeps a binding whose initializer ends at its semicolon',
      fn: async () => {
        expect(definedTextByName({
          blankedSourceText: 'const a = 1; const b = 2;',
        },).get('a',),).toBe('1',);
      },
    },),
  ],
},);
