import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  hasFilePathWithExtension,
  hasLineNumberSuffix,
  lineHasCitation,
} from './uncertainty-citations.ts';

await describe({
  name: 'uncertainty-citations',
  children: [
    describe({
      name: hasFilePathWithExtension.name,
      children: [
        it({
          name: 'matches a path with a recognised extension',
          fn: async () => {
            expect(hasFilePathWithExtension('see package/x/y.ts for details',),).toBe(true,);
          },
        },),
        it({
          name: 'matches a bare filename with an extension',
          fn: async () => {
            expect(hasFilePathWithExtension('a.ts',),).toBe(true,);
          },
        },),
        it({
          name: 'matches a multi-letter extension like tsx',
          fn: async () => {
            expect(hasFilePathWithExtension('open Component.tsx now',),).toBe(true,);
          },
        },),
        it({
          name: 'matches case-insensitively',
          fn: async () => {
            expect(hasFilePathWithExtension('FILE.TS',),).toBe(true,);
          },
        },),
        it({
          name: 'does not match a dotted extension at the very start',
          fn: async () => {
            expect(hasFilePathWithExtension('.ts is the extension',),).toBe(false,);
          },
        },),
        it({
          name: 'does not match when a trailing word char breaks the boundary',
          fn: async () => {
            expect(hasFilePathWithExtension('config.tsconfig',),).toBe(false,);
          },
        },),
        it({
          name: 'does not match plain prose',
          fn: async () => {
            expect(hasFilePathWithExtension('plain prose with no path',),).toBe(false,);
          },
        },),
        it({
          name: 'returns false for the empty string',
          fn: async () => {
            expect(hasFilePathWithExtension('',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: hasLineNumberSuffix.name,
      children: [
        it({
          name: 'matches a :N suffix',
          fn: async () => {
            expect(hasLineNumberSuffix('see file.ts:42 for details',),).toBe(true,);
          },
        },),
        it({
          name: 'matches a five-digit suffix',
          fn: async () => {
            expect(hasLineNumberSuffix('x:12345',),).toBe(true,);
          },
        },),
        it({
          name: 'does not match a six-digit run (boundary broken)',
          fn: async () => {
            expect(hasLineNumberSuffix('x:123456',),).toBe(false,);
          },
        },),
        it({
          name: 'does not match a colon with no digits',
          fn: async () => {
            expect(hasLineNumberSuffix('foo: bar',),).toBe(false,);
          },
        },),
        it({
          name: 'does not match when a trailing word char breaks the boundary',
          fn: async () => {
            expect(hasLineNumberSuffix('x:42y',),).toBe(false,);
          },
        },),
        it({
          name: 'returns false for the empty string',
          fn: async () => {
            expect(hasLineNumberSuffix('',),).toBe(false,);
          },
        },),
        it({
          name: 'finds a suffix after many bare colons (linear scan)',
          fn: async () => {
            expect(hasLineNumberSuffix(`${':'.repeat(100_000,)}:42`,),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: lineHasCitation.name,
      children: [
        it({
          name: 'is true when a file path is present',
          fn: async () => {
            expect(lineHasCitation('skip; see package/x/y.ts',),).toBe(true,);
          },
        },),
        it({
          name: 'is true when AGENTS.md is named',
          fn: async () => {
            expect(lineHasCitation('skip; AGENTS.md bans it',),).toBe(true,);
          },
        },),
        it({
          name: "is false for an uncited dismissal",
          fn: async () => {
            expect(lineHasCitation("the project doesn't use JSX",),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
