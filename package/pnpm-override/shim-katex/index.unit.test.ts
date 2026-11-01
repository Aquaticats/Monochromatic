import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import katex from './index.cjs';

// The names `lib/html.js` of micromark-extension-math and the rest of katex's public surface expose.
const THROWING_MEMBERS = [
  '__defineFunction',
  '__defineMacro',
  '__defineSymbol',
  '__parse',
  '__renderToDomTree',
  '__renderToHTMLTree',
  '__setFontMetrics',
  'render',
  'renderToString',
] as const;

/**
 Calls a shim member and hands back what it threw, so each case asserts on the error itself.
 */
function caughtFrom(call: () => unknown,): unknown {
  try {
    call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'import resolution',
      children: [
        it({
          name: 'a default import of the shim exposes renderToString as a function at load time',
          fn: async () => {
            // `lib/html.js` runs `const renderToString = katex.renderToString` while loading.
            expect(katex.renderToString,).toBeTypeOf('function',);
          },
        },),
        it({
          name: 'remark-math loads through micromark-extension-math, whose html.js imports katex',
          fn: async () => {
            const { default: remarkMath, } = await import('remark-math',);
            expect(remarkMath,).toBeTypeOf('function',);
          },
        },),
      ],
    },),
    describe({
      name: 'members',
      children: [
        ...THROWING_MEMBERS.map(function mapMember(member,) {
          return it({
            name: `${member} throws KatexNotInstalledError naming katex, the advisory and the override`,
            fn: async () => {
              const caught = caughtFrom(() => katex[member]('x',),);
              expect(caught,).toBeInstanceOf(katex.KatexNotInstalledError,);
              const error = caught as InstanceType<typeof katex.KatexNotInstalledError>;
              expect(error.name,).toBe('KatexNotInstalledError',);
              expect(error.member,).toBe(member,);
              expect(error.message,).toContain(`katex.${member}`,);
              expect(error.message,).toContain('GHSA-238p-pmpm-9mq7',);
              expect(error.message,).toContain('micromark-extension-math>katex',);
              expect(error.message,).toContain('lib/html.js',);
            },
          },);
        },),
        it({
          name: 'constructing ParseError throws KatexNotInstalledError',
          fn: async () => {
            const caught = caughtFrom(() => new katex.ParseError(),);
            expect(caught,).toBeInstanceOf(katex.KatexNotInstalledError,);
            expect((caught as InstanceType<typeof katex.KatexNotInstalledError>).member,).toBe('ParseError',);
          },
        },),
        it({
          name: 'exports exactly the members the types declare',
          fn: async () => {
            expect(Object.keys(katex,).toSorted(),).toEqual([
              ...THROWING_MEMBERS,
              'KatexNotInstalledError',
              'ParseError',
            ].toSorted(),);
          },
        },),
      ],
    },),
  ],
},);
