import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  CliInvocationError,
  parseCliArguments,
} from '../dist/final/node/index.mjs';

await describe({
  name: parseCliArguments.name,
  children: [
    it({
      name: 'parses explicit interactive and applied modes',
      fn: async () => {
        expect(parseCliArguments({
          arguments: [
            '--interactive',
            '--repo',
            'https://github.com/Aquaticat/issues-api',
            'review.json',
          ],
        },),).toStrictEqual({
          kind: 'run',
          mode: 'interactive',
          input: {
            kind: 'file',
            path: 'review.json',
          },
          repositoryUrl: 'https://github.com/Aquaticat/issues-api',
        },);
        expect(parseCliArguments({
          arguments: [
            '--non-interactive',
            '--apply',
            '--non-security-only',
            'review.json',
          ],
        },),).toStrictEqual({
          kind: 'run',
          mode: 'non-interactive',
          input: {
            kind: 'file',
            path: 'review.json',
          },
          applyAuthority: 'non-security-only',
        },);
        expect(parseCliArguments({
          arguments: [
            '--interactive',
            '{"status":"complete","comments":[]}',
          ],
        },),).toStrictEqual({
          kind: 'run',
          mode: 'interactive',
          input: {
            kind: 'inline-json',
            text: '{"status":"complete","comments":[]}',
          },
        },);
      },
    },),
    it({
      name: 'rejects missing or contradictory modes and stdin path',
      fn: async () => {
        /**
         Invalid invocation argument vectors.
         */
        const cases = [
          ['review.json',],
          ['--interactive', '--non-interactive', 'review.json',],
          ['--non-interactive', '-',],
          ['--non-interactive', '{}',],
          ['--non-interactive',],
          ['--interactive',],
          ['--interactive', '--apply', 'review.json',],
        ];
        for (const arguments_ of cases) {
          /**
           Captured invocation misuse.
           */
          let caught: unknown;
          try {
            parseCliArguments({ arguments: arguments_, },);
          }
          catch (error: unknown) {
            caught = error;
          }
          expect(caught,).toBeInstanceOf(CliInvocationError,);
        }
      },
    },),
  ],
},);
