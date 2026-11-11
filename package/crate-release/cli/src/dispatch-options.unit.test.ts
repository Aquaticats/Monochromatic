import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseCrateRegistry } from './crate-registry.ts';
import { parseDispatchArguments } from './dispatch-options.ts';
import { CrateReleaseError } from './error.ts';

/**
 Registry the tests read: the scope decides how long a note may be before the subject is too long.
 */
const registry = parseCrateRegistry(JSON.stringify({
  'forbidden-strings': {
    crateDir: 'package/forbidden-strings/cli',
    scope: 'forbidden-strings/cli',
  },
}));

/**
 Parse with a fixed root and the test registry.

 @param argv - arguments after the script path
 @returns the parsed options
 */
function parse(argv: readonly string[]) {
  return parseDispatchArguments({
    argv,
    root: '/repo',
    registry: function testRegistry() {
      return registry;
    }
  });
}

/**
 Registry of a crate that names a dispatch gate.
 */
const gatedRegistry = parseCrateRegistry(JSON.stringify({
  gated: {
    crateDir: 'package/gated/cli',
    scope: 'gated/cli',
    dispatchGate: [
      'node',
      'bin/host-controls.ts',
      '--quick',
    ],
  },
}));

/**
 Parse an argument list against a registry of the test's choosing.

 @param argv - arguments after the script path
 @param named - registry the options are resolved against
 @returns the parsed options
 */
function parseWith({
  argv,
  named,
}: {
  readonly argv: readonly string[];
  readonly named: ReturnType<typeof parseCrateRegistry>;
}) {
  return parseDispatchArguments({
    argv,
    root: '/repo',
    registry: function chosenRegistry() {
      return named;
    }
  });
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

await describe({
  name: '',
  children: [
    describe({
      name: parseDispatchArguments.name,
      children: [
        it({
          name: 'reads the crate, the versions and the note with no dry run and no repository by default',
          fn: async () => {
            expect(parse(['--crate', 'forbidden-strings', '0.6.0', '0.6.1', 'Skip the generated rule sources.'])).toEqual({
              root: '/repo',
              crate: 'forbidden-strings',
              from: '0.6.0',
              to: '0.6.1',
              note: 'Skip the generated rule sources.',
              dryRun: false,
            });
          },
        }),
        it({
          name: 'resolves the dispatch gate of the crate from the crate\'s own directory, with the rest of the command as its arguments',
          fn: async () => {
            const options = parseWith({
              argv: [
                '--crate',
                'gated',
                '0.1.0',
                '0.1.1',
                'Run the controls first.',
              ],
              named: gatedRegistry,
            });
            expect(options.dispatchGate).toEqual({
              program: 'node',
              args: [
                'bin/host-controls.ts',
                '--quick',
              ],
              cwd: '/repo/package/gated/cli',
            });
          },
        }),
        it({
          name: 'leaves the dispatch gate off the options of a crate that names none',
          fn: async () => {
            expect(parse([
              '--crate',
              'forbidden-strings',
              '0.6.0',
              '0.6.1',
              'Skip the generated rule sources.',
            ]).dispatchGate).toBeUndefined();
          },
        }),
        it({
          name: 'reads --dry-run as a switch that takes no value, and --repo and --root',
          fn: async () => {
            const options = parse([
              '--crate',
              'forbidden-strings',
              '--dry-run',
              '--repo',
              'Aquaticats/Monochromatic',
              '--root',
              '/other',
              '0.6.0',
              '0.6.1',
              'n',
            ]);
            expect(options.dryRun).toBe(true);
            expect(options.repo).toBe('Aquaticats/Monochromatic');
            expect(options.root).toBe('/other');
          },
        }),
        it({
          name: 'accepts a note that starts with a hyphen after a double hyphen',
          fn: async () => {
            expect(parse(['--crate', 'forbidden-strings', '--', '0.6.0', '0.6.1', '-x']).note).toBe('-x');
          },
        }),
        it({
          name: 'refuses a missing crate',
          fn: async () => {
            expectRefusal({ argv: ['0.6.0', '0.6.1', 'n'], mention: '--crate' });
          },
        }),
        it({
          name: 'refuses a crate the registry does not list, before any workflow run is spent on it',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'nope', '0.6.0', '0.6.1', 'n'], mention: 'forbidden-strings' });
          },
        }),
        it({
          name: 'refuses the wrong number of positional arguments and says what they are',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '0.6.0', '0.6.1'], mention: '<from> <to> <note>' });
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '0.6.0', '0.6.1', 'fix', 'it'], mention: '<from> <to> <note>' });
          },
        }),
        it({
          name: 'refuses versions that are not release versions, or equal',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '0.6', '0.6.1', 'n'], mention: '<from>' });
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '0.6.0', 'v0.6.1', 'n'], mention: '<to>' });
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '0.6.0', '0.6.0', 'n'], mention: 'must differ' });
          },
        }),
        it({
          name: 'refuses a note that cannot be a commit subject: a line break, or more than the longest note',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '0.6.0', '0.6.1', 'a\nb'], mention: 'note' });
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '0.6.0', '0.6.1', 'x'.repeat(201)], mention: 'note' });
          },
        }),
        it({
          name: 'refuses a repository that is not owner/name',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '--repo', 'o', '0.6.0', '0.6.1', 'n'], mention: '--repo' });
          },
        }),
        it({
          name: 'refuses an unknown option and names it, and a value given to the switch',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '--bogus', '0.6.0', '0.6.1', 'n'], mention: '--bogus' });
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '--dry-run=yes', '0.6.0', '0.6.1', 'n'], mention: '--dry-run' });
          },
        }),
      ],
    }),
  ],
});
