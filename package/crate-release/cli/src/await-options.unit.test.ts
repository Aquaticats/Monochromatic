import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseAwaitArguments } from './await-options.ts';
import {
  defaultAssetTemplates,
  defaultAttempts,
  defaultCurlMaxTimeSeconds,
  defaultFollowAttempts,
  defaultPollIntervalSeconds,
  defaultRunListLimit,
  defaultTagTemplate,
  defaultUserAgent,
  defaultWorkflow,
  manifestFileName,
} from './constants.ts';
import { parseCrateRegistry } from './crate-registry.ts';
import { CrateReleaseError } from './error.ts';

/**
 Registry the tests read.
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
  return parseAwaitArguments({ argv, root: '/repo', registry: () => registry });
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
      name: parseAwaitArguments.name,
      children: [
        it({
          name: 'reads the crate, the run and the version with the publish check defaults',
          fn: async () => {
            expect(parse(['--crate', 'forbidden-strings', '123456789', '0.6.1'])).toEqual({
              root: '/repo',
              crateName: 'forbidden-strings',
              manifestPath: `${registry.get('forbidden-strings')?.crateDirectory}/${manifestFileName}`,
              releaseRunId: '123456789',
              version: '0.6.1',
              workflow: defaultWorkflow,
              attempts: defaultAttempts,
              pollIntervalSeconds: defaultPollIntervalSeconds,
              followAttempts: defaultFollowAttempts,
              runLimit: defaultRunListLimit,
              tagTemplate: defaultTagTemplate,
              assetTemplates: defaultAssetTemplates,
              userAgent: defaultUserAgent,
              maxTimeSeconds: defaultCurlMaxTimeSeconds,
            });
          },
        }),
        it({
          name: 'reads the run out of the address that the dispatch command printed',
          fn: async () => {
            const options = parse(['--crate', 'forbidden-strings', 'https://github.com/o/r/actions/runs/987', '0.6.1']);
            expect(options.releaseRunId).toBe('987');
          },
        }),
        it({
          name: 'names the manifest of the crate that the registry lists, which holds the version a publish run must carry',
          fn: async () => {
            const options = parse(['--crate', 'forbidden-strings', '987', '0.6.1']);
            expect(options.manifestPath).toBe(`${registry.get('forbidden-strings')?.crateDirectory}/${manifestFileName}`);
          },
        }),
        it({
          name: 'reads every publish check option, as the wait command does',
          fn: async () => {
            const options = parse([
              '--crate',
              'forbidden-strings',
              '--workflow',
              'other.yml',
              '--repo',
              'o/r',
              '--attempts',
              '3',
              '--poll-interval',
              '1',
              '--run-limit',
              '5',
              '--tag-template',
              't-{version}',
              '--asset-template',
              'a-{version}.tgz',
              '--user-agent',
              'ua',
              '--max-time',
              '9',
              '--root',
              '/other',
              '123',
              '0.6.1',
            ]);
            expect(options.workflow).toBe('other.yml');
            expect(options.repo).toBe('o/r');
            expect(options.attempts).toBe(3);
            expect(options.pollIntervalSeconds).toBe(1);
            expect(options.runLimit).toBe(5);
            expect(options.tagTemplate).toBe('t-{version}');
            expect(options.assetTemplates).toEqual(['a-{version}.tgz']);
            expect(options.userAgent).toBe('ua');
            expect(options.maxTimeSeconds).toBe(9);
            expect(options.root).toBe('/other');
          },
        }),
        it({
          name: 'refuses a missing crate and a crate the registry does not list',
          fn: async () => {
            expectRefusal({ argv: ['123', '0.6.1'], mention: '--crate' });
            expectRefusal({ argv: ['--crate', 'nope', '123', '0.6.1'], mention: 'forbidden-strings' });
          },
        }),
        it({
          name: 'refuses a run that is neither digits nor a run address, naming the argument',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'forbidden-strings', 'abc', '0.6.1'], mention: '<run>' });
            expectRefusal({ argv: ['--crate', 'forbidden-strings', 'https://github.com/o/r/pull/9', '0.6.1'], mention: '<run>' });
          },
        }),
        it({
          name: 'refuses a version that would change the meaning of a URL',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '123', '0.6.1/../x'], mention: '<version>' });
          },
        }),
        it({
          name: 'refuses the wrong number of positional arguments and says what they are',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '123'], mention: '<run> <version>' });
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '123', '0.6.1', 'x'], mention: '<run> <version>' });
          },
        }),
        it({
          name: 'refuses the options of the wait command that name a crate by its crates.io name',
          fn: async () => {
            expectRefusal({ argv: ['--crate', 'forbidden-strings', '--crate-name', 'x', '123', '0.6.1'], mention: '--crate-name' });
          },
        }),
      ],
    }),
  ],
});
