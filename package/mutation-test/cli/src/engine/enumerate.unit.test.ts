import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { enumerateMutants, } from '../../dist/final/node/index.mjs';

/**
 Enumerates one snippet and returns replacement texts for one family.
 
 @param options - Snippet source and operator family under test.
 
 @returns Replacement texts emitted for the family.
 
 @example
 ```ts
 familyReplacements({ source: 'const x = 1 + 2;', operator: 'arithmetic' });
 ```
 */
function familyReplacements(options: {
  readonly source: string;
  readonly operator: string;
},): readonly string[] {
  return enumerateMutants({
    file: 'src/snippet.ts',
    source: options.source,
  },)
    .mutants
    .filter(function byFamily(mutant,): boolean {
      return mutant.operator === options.operator;
    },)
    .map(function toText(mutant,): string {
      return mutant.replacement;
    },);
}

await describe({
  name: '',
  children: [
    describe({
      name: 'operator families',
      children: [
        it({
          name: 'arithmetic swaps + with -',
          fn: async () => {
            expect(familyReplacements({
              source: 'export const x: number = 1 + 2;\n',
              operator: 'arithmetic',
            },),).toEqual(['-',],);
          },
        },),
        it({
          name: 'equality emits boundary and negation variants for <',
          fn: async () => {
            expect(familyReplacements({
              source: 'export const x = 1 < 2;\n',
              operator: 'equality',
            },),).toEqual([
              '<=',
              '>=',
            ],);
          },
        },),
        it({
          name: 'logical swaps && and ignores comment operators',
          fn: async () => {
            expect(familyReplacements({
              source: 'export const x = true /* || */ && false;\n',
              operator: 'logical',
            },),).toEqual(['||',],);
          },
        },),
        it({
          name: 'conditional forces if tests and loop tests',
          fn: async () => {
            /**
             Conditional replacements for an if plus while snippet.
             */
            const texts = familyReplacements({
              source: 'if (a > 1) { b(); }\nwhile (c) { d(); }\n',
              operator: 'conditional',
            },);
            expect(texts,).toContain('true',);
            expect(texts.filter(function isFalse(text,): boolean {
              return text === 'false';
            },),).toHaveLength(2,);
          },
        },),
        it({
          name: 'boolean swaps literals and removes negation',
          fn: async () => {
            expect(familyReplacements({
              source: 'export const x = !ready;\nexport const y = true;\n',
              operator: 'boolean',
            },),).toEqual([
              'ready',
              'false',
            ],);
          },
        },),
        it({
          name: 'string empties literals and fills empty ones',
          fn: async () => {
            expect(familyReplacements({
              source: "export const a = 'hi';\nexport const b = '';\n",
              operator: 'string',
            },),).toEqual([
              "''",
              "'mutation-test was here!'",
            ],);
          },
        },),
        it({
          name: 'string skips import sources, keys, and type literals',
          fn: async () => {
            expect(familyReplacements({
              source: "import { a } from 'node:fs';\nexport const o = { key: 1 };\nexport type T = 'lit';\n",
              operator: 'string',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'type-erased subtrees produce no mutants at all',
          fn: async () => {
            expect(enumerateMutants({
              file: 'src/snippet.ts',
              source: "export type T = 'a' | 1;\nexport interface I { m(k: 'x',): 'y'; }\nexport declare function g(a: number,): string;\n",
            },).mutants,).toEqual([],);
          },
        },),
        it({
          name: 'type arguments skip while value arguments still mutate',
          fn: async () => {
            expect(familyReplacements({
              source: "export const s = identity<'a'>('b',);\n",
              operator: 'string',
            },),).toEqual(["''",],);
          },
        },),
        it({
          name: 'template empties and skips tagged templates',
          fn: async () => {
            expect(familyReplacements({
              // oxlint-disable-next-line no-template-curly-in-string -- fixture source deliberately embeds template syntax
              source: 'export const a = `x${1}y`;\nexport const b = tag`z`;\n',
              operator: 'string',
            },),).toEqual(['``',],);
          },
        },),
        it({
          name: 'array and object empty out',
          fn: async () => {
            expect(familyReplacements({
              source: 'export const a = [1];\n',
              operator: 'array',
            },),).toEqual(['[]',],);
            expect(familyReplacements({
              source: 'export const o = { a: 1 };\n',
              operator: 'object',
            },),).toEqual(['{}',],);
          },
        },),
        it({
          name: 'unary swaps sign and update swaps direction',
          fn: async () => {
            expect(familyReplacements({
              source: 'export const x = -y;\n',
              operator: 'unary',
            },),).toEqual(['+y',],);
            expect(familyReplacements({
              source: 'let i = 0;\ni++;\n',
              operator: 'update',
            },),).toEqual(['i--',],);
          },
        },),
        it({
          name: 'optional chaining drops ?. with correct dot handling',
          fn: async () => {
            expect(familyReplacements({
              source: 'export const a = o?.b;\nexport const c = f?.();\n',
              operator: 'optional-chaining',
            },),).toEqual([
              '.',
              '',
            ],);
          },
        },),
        it({
          name: 'method swaps startsWith and removes trim',
          fn: async () => {
            /**
             Method replacements for swap plus removal snippet.
             */
            const texts = familyReplacements({
              source: "export const a = s.startsWith('x');\nexport const b = s.trim();\n",
              operator: 'method',
            },);
            expect(texts,).toContain('endsWith',);
            expect(texts,).toContain('s',);
          },
        },),
        it({
          name: 'block empties non-empty blocks and arrow forces undefined',
          fn: async () => {
            expect(familyReplacements({
              source: 'export function f(): void { g(); }\n',
              operator: 'block',
            },),).toEqual(['{}',],);
            expect(familyReplacements({
              source: 'export const f = () => 5;\n',
              operator: 'arrow',
            },),).toEqual(['undefined',],);
          },
        },),
        it({
          name: 'regex negates escape classes and swaps quantifiers',
          fn: async () => {
            /**
             Regex replacements for one pattern literal.
             */
            const texts = familyReplacements({
              source: String.raw`export const r = /\d+/g;
`,
              operator: 'regex',
            },);
            expect(texts,).toContain(String.raw`/\D+/g`,);
            expect(texts,).toContain(String.raw`/\d*/g`,);
          },
        },),
      ],
    },),
    describe({
      name: 'suppression',
      children: [
        it({
          name: 'next-line directive ignores matching families with reason',
          fn: async () => {
            /**
             Enumeration of a snippet with a next-line suppression.
             */
            const result = enumerateMutants({
              file: 'src/snippet.ts',
              source: "// mutation-test-disable-next-line string -- filler noise\nexport const a = 'hi';\nexport const b = 'yo';\n",
            },);
            expect(result.ignored,).toHaveLength(1,);
            expect(result.ignored[0]?.reason,).toBe('filler noise',);
            expect(result.ignored[0]?.line,).toBe(2,);
            expect(result.mutants
              .filter(function strings(mutant,): boolean {
                return mutant.operator === 'string';
              },),).toHaveLength(1,);
          },
        },),
        it({
          name: 'file directive ignores family everywhere',
          fn: async () => {
            /**
             Enumeration of a snippet with a file-wide suppression.
             */
            const result = enumerateMutants({
              file: 'src/snippet.ts',
              source: "// mutation-test-disable-file string\nexport const a = 'hi';\nexport const b = 'yo';\n",
            },);
            expect(result.ignored,).toHaveLength(2,);
            expect(result.mutants
              .filter(function strings(mutant,): boolean {
                return mutant.operator === 'string';
              },),).toHaveLength(0,);
          },
        },),
        it({
          name: 'bare next-line directive suppresses every family',
          fn: async () => {
            /**
             Enumeration with an unqualified next-line suppression.
             */
            const result = enumerateMutants({
              file: 'src/snippet.ts',
              source: '// mutation-test-disable-next-line\nexport const x = 1 + 2;\n',
            },);
            expect(result.mutants,).toHaveLength(0,);
            expect(result.ignored.length,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'unknown family in directive throws',
          fails: true,
          fn: async () => {
            enumerateMutants({
              file: 'src/snippet.ts',
              source: '// mutation-test-disable-next-line bogus\nexport const x = 1;\n',
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'enumeration invariants',
      children: [
        it({
          name: 'dedupes identical spans from overlapping families',
          fn: async () => {
            /**
             Enumeration where conditional forcing overlaps if-test forcing.
             */
            const result = enumerateMutants({
              file: 'src/snippet.ts',
              source: 'if (a > 1) { b(); }\n',
            },);
            /**
             Span-plus-text keys for duplicate detection.
             */
            const keys = result.mutants
              .map(function toKey(mutant,): string {
                return `${String(mutant.start,)}:${String(mutant.end,)}:${mutant.replacement}`;
              },);
            expect(new Set(keys,).size,).toBe(keys.length,);
          },
        },),
        it({
          name: 'throws on unparsable source',
          fails: true,
          fn: async () => {
            enumerateMutants({
              file: 'src/snippet.ts',
              source: 'const = ;;;',
            },);
          },
        },),
        it({
          name: 'ids are unique across a real enumeration',
          fn: async () => {
            /**
             Enumeration over a mixed-feature snippet.
             */
            const result = enumerateMutants({
              file: 'src/snippet.ts',
              // oxlint-disable-next-line no-template-curly-in-string -- fixture source deliberately embeds template syntax
              source: 'export function f(a: number,): string {\n  if (a > 1 && a < 10) return `big ${a}`;\n  return a.toString().trim();\n}\n',
            },);
            expect(new Set(result.mutants
              .map(function toId(mutant,): string {
                return mutant.id;
              },),).size,).toBe(result.mutants.length,);
          },
        },),
      ],
    },),
  ],
},);
