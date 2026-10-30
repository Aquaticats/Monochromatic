import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  defaultMemoryMax,
  defaultUnitSuffix,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import { parseReleaseArguments } from './release-options.ts';

/**
 Required options of the usual crate.
 */
const requiredOptions: readonly string[] = [
  '--crate-name',
  'monochromatic-git-policy-cli',
  '--crate-dir',
  'package/git-policy/cli',
  '--scope',
  'git-policy/cli',
];

/**
 Parse with a fixed repository root.

 @param argv - arguments after the script path
 @returns the parsed options
 */
function parse(argv: readonly string[]) {
  return parseReleaseArguments({ argv, root: '/repo' });
}

/**
 Run a parse that must throw and return what it threw.

 @param argv - arguments after the script path
 @returns the thrown value
 */
function thrownBy(argv: readonly string[]): unknown {
  try {
    parse(argv);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Expect a parse to refuse its input with a message that holds the given text.

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
  const caught = thrownBy(argv);
  expect(caught).toBeInstanceOf(CrateReleaseError);
  expect((caught as Error).message).toContain(mention);
}

await describe({
  name: '',
  children: [
    describe({
      name: parseReleaseArguments.name,
      children: [
        it({
          name: 'reads the required options and the three positional arguments with the defaults',
          fn: async () => {
            expect(parse([...requiredOptions, '0.4.2', '0.4.3', 'fix it'])).toEqual({
              root: '/repo',
              crateName: 'monochromatic-git-policy-cli',
              crateDirectory: 'package/git-policy/cli',
              scope: 'git-policy/cli',
              from: '0.4.2',
              to: '0.4.3',
              note: 'fix it',
              pinFiles: [],
              checkBin: undefined,
              lockTasks: [],
              memoryMax: defaultMemoryMax,
              unitName: `monochromatic-git-policy-cli${defaultUnitSuffix}`,
            });
          },
        }),
        it({
          name: 'reads every optional option and keeps repeated ones in order',
          fn: async () => {
            const options = parse([
              ...requiredOptions,
              '--root',
              '/other',
              '--pin-file',
              'package/git-policy/cli.fuzz/Cargo.toml',
              '--pin-file',
              'package/x/Cargo.toml',
              '--check-bin',
              'cli-git-native',
              '--lock-task',
              '//package/a:lock',
              '--lock-task',
              '//package/b:lock',
              '--memory-max',
              '8G',
              '--unit-name',
              'cli-git-publish-dry',
              '0.4.2',
              '0.4.3',
              'n',
            ]);
            expect(options.root).toBe('/other');
            expect(options.pinFiles).toEqual(['package/git-policy/cli.fuzz/Cargo.toml', 'package/x/Cargo.toml']);
            expect(options.checkBin).toBe('cli-git-native');
            expect(options.lockTasks).toEqual(['//package/a:lock', '//package/b:lock']);
            expect(options.memoryMax).toBe('8G');
            expect(options.unitName).toBe('cli-git-publish-dry');
          },
        }),
        it({
          name: 'accepts a note that starts with a hyphen after a double hyphen',
          fn: async () => {
            expect(parse([...requiredOptions, '--', '0.4.2', '0.4.3', '-x']).note).toBe('-x');
          },
        }),
        it({
          name: 'accepts a two-digit patch release and a prerelease target',
          fn: async () => {
            const options = parse([...requiredOptions, '0.4.9', '0.4.10-rc.1', 'n']);
            expect(options.from).toBe('0.4.9');
            expect(options.to).toBe('0.4.10-rc.1');
          },
        }),
        it({
          name: 'refuses a missing crate name',
          fn: async () => {
            expectRefusal({ argv: ['--crate-dir', 'p', '--scope', 's', '0.1.0', '0.1.1', 'n'], mention: '--crate-name' });
          },
        }),
        it({
          name: 'refuses a missing crate directory',
          fn: async () => {
            expectRefusal({ argv: ['--crate-name', 'c', '--scope', 's', '0.1.0', '0.1.1', 'n'], mention: '--crate-dir' });
          },
        }),
        it({
          name: 'refuses a missing scope',
          fn: async () => {
            expectRefusal({ argv: ['--crate-name', 'c', '--crate-dir', 'p', '0.1.0', '0.1.1', 'n'], mention: '--scope' });
          },
        }),
        it({
          name: 'refuses too few positional arguments and says what they are',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '0.4.2', '0.4.3'], mention: '<from> <to> <note>' });
          },
        }),
        it({
          name: 'refuses too many positional arguments, such as a note that was not quoted',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '0.4.2', '0.4.3', 'fix', 'it'], mention: '<from> <to> <note>' });
          },
        }),
        it({
          name: 'refuses a target equal to the current version',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '0.4.2', '0.4.2', 'n'], mention: '0.4.2' });
          },
        }),
        it({
          name: 'refuses a current version that is not a version',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '0.4', '0.4.3', 'n'], mention: '<from>' });
          },
        }),
        it({
          name: 'refuses a target that is not a version',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '0.4.2', '0.4.3; rm', 'n'], mention: '<to>' });
          },
        }),
        it({
          name: 'refuses a note with a line break',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '0.4.2', '0.4.3', 'a\nb'], mention: 'note' });
          },
        }),
        it({
          name: 'refuses an unknown option and names it',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--bogus', '0.4.2', '0.4.3', 'n'], mention: '--bogus' });
          },
        }),
        it({
          name: 'refuses an option given without its value',
          fn: async () => {
            expectRefusal({ argv: ['0.4.2', '0.4.3', 'n', '--crate-name'], mention: '--crate-name' });
          },
        }),
        it({
          name: 'refuses an absolute crate directory',
          fn: async () => {
            expectRefusal({
              argv: ['--crate-name', 'c', '--crate-dir', '/etc', '--scope', 's', '0.1.0', '0.1.1', 'n'],
              mention: '--crate-dir',
            });
          },
        }),
        it({
          name: 'refuses a crate directory that leaves the repository',
          fn: async () => {
            expectRefusal({
              argv: ['--crate-name', 'c', '--crate-dir', '../x', '--scope', 's', '0.1.0', '0.1.1', 'n'],
              mention: '--crate-dir',
            });
          },
        }),
        it({
          name: 'refuses a pin file that leaves the repository',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--pin-file', '../x/Cargo.toml', '0.4.2', '0.4.3', 'n'], mention: '--pin-file' });
          },
        }),
        it({
          name: 'refuses a pin file that is not a manifest, because its lockfile is derived from its directory',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--pin-file', 'package/x/Other.toml', '0.4.2', '0.4.3', 'n'], mention: 'Cargo.toml' });
          },
        }),
        it({
          name: 'accepts a pin file that is the manifest at the repository root',
          fn: async () => {
            expect(parse([...requiredOptions, '--pin-file', 'Cargo.toml', '0.4.2', '0.4.3', 'n']).pinFiles).toEqual(['Cargo.toml']);
          },
        }),
        it({
          name: 'refuses a lock task that mise would read as an option',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--lock-task', '--help', '0.4.2', '0.4.3', 'n'], mention: '--lock-task' });
          },
        }),
        it({
          name: 'refuses a check binary name with shell syntax',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--check-bin', 'a;b', '0.4.2', '0.4.3', 'n'], mention: '--check-bin' });
          },
        }),
        it({
          name: 'refuses a memory cap that is not a systemd value',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--memory-max', '12G;x', '0.4.2', '0.4.3', 'n'], mention: '--memory-max' });
          },
        }),
        it({
          name: 'refuses a unit name with characters systemd would reject',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--unit-name', 'a b', '0.4.2', '0.4.3', 'n'], mention: '--unit-name' });
          },
        }),
        it({
          name: 'refuses a scope that would break the commit subject',
          fn: async () => {
            expectRefusal({
              argv: ['--crate-name', 'c', '--crate-dir', 'p', '--scope', 'a):b', '0.1.0', '0.1.1', 'n'],
              mention: '--scope',
            });
          },
        }),
      ],
    }),
  ],
});
