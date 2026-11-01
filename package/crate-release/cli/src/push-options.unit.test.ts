import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CrateReleaseError } from './error.ts';
import { parsePushArguments } from './push-options.ts';

/**
 The required options of a push.
 */
const required: readonly string[] = [
  '--repo',
  'Aquaticats/Monochromatic',
  '--branch',
  'main',
  '--scratch-dir',
  '/runner/temp',
];

/**
 Run an action that must throw and return what it threw.

 @param action - call under test
 @returns the thrown value
 */
function thrownBy(action: () => unknown): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Expect a parse to be refused with a message that holds the given text.

 @param argv - arguments after the script path
 @param mention - text the message must hold
 */
function expectRefusal({
  argv,
  mention,
}: {
  readonly argv: readonly string[];
  readonly mention: string;
}): void {
  const caught = thrownBy(() => parsePushArguments({ argv, root: '/repo' }));
  expect(caught).toBeInstanceOf(CrateReleaseError);
  expect((caught as Error).message).toContain(mention);
}

await describe({
  name: '',
  children: [
    describe({
      name: parsePushArguments.name,
      children: [
        it({
          name: 'reads the three required options and the default root',
          fn: async () => {
            expect(parsePushArguments({ argv: required, root: '/repo' })).toEqual({
              root: '/repo',
              repo: 'Aquaticats/Monochromatic',
              branch: 'main',
              scratchDirectory: '/runner/temp',
            });
          },
        }),
        it({
          name: 'lets --root name the checkout, and does not look for the default root then',
          fn: async () => {
            const options = parsePushArguments({
              argv: [...required, '--root', '/other'],
              root: function unreadable(): never {
                throw new Error('the root must not be searched');
              },
            });
            expect(options.root).toBe('/other');
          },
        }),
        ...['--repo', '--branch', '--scratch-dir'].map(function missingCase(name) {
          return it({
            name: `refuses a missing ${name}`,
            fn: async () => {
              const argv = required.filter(function keep(_value, index) {
                const at = required.indexOf(name);
                return (index !== at) && (index !== (at + 1));
              });
              expectRefusal({ argv, mention: name });
            },
          });
        }),
        it({
          name: 'refuses a repository that is not owner/name',
          fn: async () => {
            expectRefusal({ argv: ['--repo', 'o', '--branch', 'main', '--scratch-dir', '/t'], mention: '--repo' });
          },
        }),
        it({
          name: 'refuses a branch that starts with a hyphen',
          fn: async () => {
            expectRefusal({ argv: ['--repo', 'o/r', '--branch', '-main', '--scratch-dir', '/t'], mention: '--branch' });
          },
        }),
        it({
          name: 'refuses a scratch directory that is not absolute, because the request file would land in the checkout',
          fn: async () => {
            expectRefusal({ argv: ['--repo', 'o/r', '--branch', 'main', '--scratch-dir', 'tmp'], mention: '--scratch-dir' });
          },
        }),
        it({
          name: 'refuses a positional argument, because the command takes options only',
          fn: async () => {
            expectRefusal({ argv: [...required, 'extra'], mention: 'options only' });
          },
        }),
        it({
          name: 'refuses an unknown option and names it',
          fn: async () => {
            expectRefusal({ argv: [...required, '--force'], mention: '--force' });
          },
        }),
      ],
    }),
  ],
});
