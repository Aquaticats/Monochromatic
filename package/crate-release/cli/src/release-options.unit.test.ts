import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  defaultCpuQuota,
  defaultMemoryMax,
  defaultReleaseHost,
  defaultTasksMax,
  defaultUnitSuffix,
} from './constants.ts';
import { parseCrateRegistry } from './crate-registry.ts';
import { CrateReleaseError } from './error.ts';
import { parseReleaseArguments } from './release-options.ts';

/**
 Registry the `--crate` tests read: one crate with every field, one with only the required fields.
 */
const testRegistry = parseCrateRegistry(JSON.stringify({
  'monochromatic-git-policy-cli': {
    crateDir: 'package/git-policy/cli',
    scope: 'git-policy/cli',
    pinFiles: ['package/git-policy/cli.fuzz/Cargo.toml'],
    checkBins: ['monochromatic-git-policy-cli'],
    lockDirs: ['package/git-policy/cli.fuzz'],
  },
  'plain-crate': {
    crateDir: 'package/plain/cli',
    scope: 'plain/cli',
  },
}));

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
  return parseReleaseArguments({ argv, root: '/repo', registry: () => testRegistry });
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
              checkBins: [],
              lockDirectories: [],
              host: defaultReleaseHost,
              memoryMax: defaultMemoryMax,
              cpuQuota: defaultCpuQuota,
              tasksMax: defaultTasksMax,
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
              'monochromatic-git-policy-cli',
              '--check-bin',
              'second-bin',
              '--lock-dir',
              'package/a',
              '--lock-dir',
              'package/b',
              '--host',
              'runner',
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
            expect(options.checkBins).toEqual(['monochromatic-git-policy-cli', 'second-bin']);
            expect(options.lockDirectories).toEqual(['package/a', 'package/b']);
            expect(options.host).toBe('runner');
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
          name: 'refuses a lock directory that leaves the repository',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--lock-dir', '../x', '0.4.2', '0.4.3', 'n'], mention: '--lock-dir' });
          },
        }),
        it({
          name: 'refuses a host that is neither local nor runner and lists the hosts',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--host', 'cloud', '0.4.2', '0.4.3', 'n'], mention: 'local, runner' });
          },
        }),
        it({
          name: 'refuses the retired lock task option, because the lockfile refresh is a cargo call in a directory now',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--lock-task', '//package/a:lock', '0.4.2', '0.4.3', 'n'], mention: '--lock-task' });
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
          name: 'reads --cpu-quota and --tasks-max',
          fn: async () => {
            const options = parse([...requiredOptions, '--cpu-quota', '700%', '--tasks-max', '77', '0.4.2', '0.4.3', 'n']);
            expect(options.cpuQuota).toBe('700%');
            expect(options.tasksMax).toBe('77');
          },
        }),
        it({
          name: 'defaults the CPU quota and the task limit to systemd values',
          fn: async () => {
            const options = parse([...requiredOptions, '0.4.2', '0.4.3', 'n']);
            expect(options.cpuQuota).toBe(defaultCpuQuota);
            expect(options.tasksMax).toBe(defaultTasksMax);
          },
        }),
        it({
          name: 'refuses a CPU quota that is not a systemd value',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--cpu-quota', '5;x', '0.4.2', '0.4.3', 'n'], mention: '--cpu-quota' });
          },
        }),
        it({
          name: 'refuses a task limit that is not a systemd value',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--tasks-max', '5;x', '0.4.2', '0.4.3', 'n'], mention: '--tasks-max' });
          },
        }),
        it({
          name: 'refuses a unit name with characters systemd would reject',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--unit-name', 'a b', '0.4.2', '0.4.3', 'n'], mention: '--unit-name' });
          },
        }),
        it({
          name: 'reads every argument of a crate from the registry with --crate and keeps the defaults of the rest',
          fn: async () => {
            expect(parse(['--crate', 'monochromatic-git-policy-cli', '--host', 'runner', '0.4.2', '0.4.3', 'fix it'])).toEqual({
              root: '/repo',
              crateName: 'monochromatic-git-policy-cli',
              crateDirectory: 'package/git-policy/cli',
              scope: 'git-policy/cli',
              from: '0.4.2',
              to: '0.4.3',
              note: 'fix it',
              pinFiles: ['package/git-policy/cli.fuzz/Cargo.toml'],
              checkBins: ['monochromatic-git-policy-cli'],
              lockDirectories: ['package/git-policy/cli.fuzz'],
              host: 'runner',
              memoryMax: defaultMemoryMax,
              cpuQuota: defaultCpuQuota,
              tasksMax: defaultTasksMax,
              unitName: `monochromatic-git-policy-cli${defaultUnitSuffix}`,
            });
          },
        }),
        it({
          name: 'reads a registry entry that has only the required fields as a crate without pins, binaries or lock directories',
          fn: async () => {
            const options = parse(['--crate', 'plain-crate', '0.1.0', '0.1.1', 'n']);
            expect(options.pinFiles).toEqual([]);
            expect(options.checkBins).toEqual([]);
            expect(options.lockDirectories).toEqual([]);
          },
        }),
        it({
          name: 'still lets --memory-max and --unit-name tune a local dry run of a registry crate',
          fn: async () => {
            const options = parse(['--crate', 'plain-crate', '--memory-max', '4G', '--unit-name', 'u', '0.1.0', '0.1.1', 'n']);
            expect(options.memoryMax).toBe('4G');
            expect(options.unitName).toBe('u');
          },
        }),
        it({
          name: 'refuses a crate that the registry does not list and names the crates it lists',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'nope', '0.1.0', '0.1.1', 'n'], mention: 'monochromatic-git-policy-cli, plain-crate' });
          },
        }),
        it({
          name: 'refuses --crate beside any option that spells the crate out, and names every one given',
          fn: async () => {
            const caught = thrownBy(['--crate', 'plain-crate', '--scope', 's', '--pin-file', 'package/x/Cargo.toml', '0.1.0', '0.1.1', 'n']);
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('--scope, --pin-file');
          },
        }),
        it({
          name: 'refuses a note that does not fit the commit subject of a registry crate before any file changes',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'plain-crate', '0.1.0', '0.1.1', 'a\nb'], mention: 'note' });
          },
        }),
        it({
          name: 'does not read the registry when the crate is spelled out',
          fn: async () => {
            const options = parseReleaseArguments({
              argv: [...requiredOptions, '0.4.2', '0.4.3', 'n'],
              root: '/repo',
              registry: function unreadable(): never {
                throw new Error('the registry must not be read');
              },
            });
            expect(options.crateName).toBe('monochromatic-git-policy-cli');
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
