import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { defaultPushAttempts } from './constants.ts';
import {
  type CrateRegistry,
  parseCrateRegistry,
} from './crate-registry.ts';
import { CrateReleaseError } from './error.ts';
import { parsePushArguments } from './push-options.ts';

/**
 Registry the tests read: one crate with a lock directory and one without.
 */
const registry: CrateRegistry = parseCrateRegistry(JSON.stringify({
  'locked-crate': {
    crateDir: 'package/locked/cli',
    scope: 'locked/cli',
    lockDirs: ['package/locked/cli.fuzz', 'package/other/cli.fuzz'],
  },
  'plain-crate': {
    crateDir: 'package/plain/cli',
    scope: 'plain/cli',
  },
}));

/**
 The required options of a push.
 */
const required: readonly string[] = [
  '--crate',
  'locked-crate',
  '--repo',
  'Aquaticats/Monochromatic',
  '--branch',
  'main',
  '--scratch-dir',
  '/runner/temp',
];

/**
 Parse with a fixed root and the test registry.

 @param argv - arguments after the script path
 @returns the parsed options
 */
function parse(argv: readonly string[]) {
  return parsePushArguments({ argv, root: '/repo', registry: () => registry });
}

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
  const caught = thrownBy(() => parse(argv));
  expect(caught).toBeInstanceOf(CrateReleaseError);
  expect((caught as Error).message).toContain(mention);
}

/**
 Take an option and its value out of a command line.

 @param argv - the command line
 @param name - the option to remove
 @returns the command line without the option and its value
 */
function without({
  argv,
  name,
}: {
  readonly argv: readonly string[];
  readonly name: string;
}): readonly string[] {
  const at = argv.indexOf(name);
  return argv.filter(function keep(_value, index) {
    return (index !== at) && (index !== (at + 1));
  });
}

await describe({
  name: '',
  children: [
    describe({
      name: parsePushArguments.name,
      children: [
        it({
          name: 'reads the required options, the crate arguments from the registry and the default number of sends',
          fn: async () => {
            expect(parse(required)).toEqual({
              root: '/repo',
              repo: 'Aquaticats/Monochromatic',
              branch: 'main',
              scratchDirectory: '/runner/temp',
              crateName: 'locked-crate',
              crateDirectory: 'package/locked/cli',
              lockDirectories: ['package/locked/cli.fuzz', 'package/other/cli.fuzz'],
              maxAttempts: defaultPushAttempts,
            });
          },
        }),
        it({
          name: 'reads a crate without lock directories as one with none',
          fn: async () => {
            const argv = ['--crate', 'plain-crate', ...without({ argv: required, name: '--crate' })];
            expect(parse(argv).lockDirectories).toEqual([]);
          },
        }),
        it({
          name: 'reads --max-attempts as a whole number of sends',
          fn: async () => {
            const given = defaultPushAttempts + 1;
            expect(parse([...required, '--max-attempts', String(given)]).maxAttempts).toBe(given);
          },
        }),
        it({
          name: 'refuses --max-attempts that is not a positive whole number',
          fn: async () => {
            for (const value of ['0', '-1', '1.5', 'many', ''])
              expectRefusal({ argv: [...required, '--max-attempts', value], mention: '--max-attempts' });
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
              registry: () => registry,
            });
            expect(options.root).toBe('/other');
          },
        }),
        ...['--crate', '--repo', '--branch', '--scratch-dir'].map(function missingCase(name) {
          return it({
            name: `refuses a missing ${name}`,
            fn: async () => {
              expectRefusal({ argv: without({ argv: required, name }), mention: name });
            },
          });
        }),
        it({
          name: 'refuses a crate the registry does not list and names the crates it lists',
          fn: async () => {
            expectRefusal({
              argv: ['--crate', 'nope', ...without({ argv: required, name: '--crate' })],
              mention: 'locked-crate, plain-crate',
            });
          },
        }),
        it({
          name: 'refuses a repository that is not owner/name',
          fn: async () => {
            expectRefusal({
              argv: ['--repo', 'o', ...without({ argv: required, name: '--repo' })],
              mention: '--repo',
            });
          },
        }),
        it({
          name: 'refuses a branch that starts with a hyphen',
          fn: async () => {
            expectRefusal({
              argv: ['--branch', '-main', ...without({ argv: required, name: '--branch' })],
              mention: '--branch',
            });
          },
        }),
        it({
          name: 'refuses a scratch directory that is not absolute, because the request file would land in the checkout',
          fn: async () => {
            expectRefusal({
              argv: ['--scratch-dir', 'tmp', ...without({ argv: required, name: '--scratch-dir' })],
              mention: '--scratch-dir',
            });
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
