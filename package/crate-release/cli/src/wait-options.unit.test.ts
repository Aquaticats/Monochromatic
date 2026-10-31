import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  defaultAssetTemplates,
  defaultAttempts,
  defaultCurlMaxTimeSeconds,
  defaultPollIntervalSeconds,
  defaultRunListLimit,
  defaultTagTemplate,
  defaultUserAgent,
  defaultWorkflow,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import { parseWaitArguments } from './wait-options.ts';

/**
 Required option of the usual crate.
 */
const requiredOptions: readonly string[] = ['--crate-name', 'monochromatic-git-policy-cli'];

/**
 Short commit hash used by the cases.
 */
const sha = 'abcdef0';

/**
 Parse with a fixed repository root.

 @param argv - arguments after the script path
 @returns the parsed options
 */
function parse(argv: readonly string[]) {
  return parseWaitArguments({ argv, root: '/repo' });
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
      name: parseWaitArguments.name,
      children: [
        it({
          name: 'reads the crate name and the two positional arguments with the defaults',
          fn: async () => {
            expect(parse([...requiredOptions, sha, '0.4.10'])).toEqual({
              root: '/repo',
              crateName: 'monochromatic-git-policy-cli',
              sha,
              version: '0.4.10',
              workflow: defaultWorkflow,
              attempts: defaultAttempts,
              pollIntervalSeconds: defaultPollIntervalSeconds,
              runLimit: defaultRunListLimit,
              tagTemplate: defaultTagTemplate,
              assetTemplates: defaultAssetTemplates,
              userAgent: defaultUserAgent,
              maxTimeSeconds: defaultCurlMaxTimeSeconds,
            });
          },
        }),
        it({
          name: 'expects the x86_64 and aarch64 linux tarballs by default',
          fn: async () => {
            expect(defaultAssetTemplates).toEqual([
              '{crate}-{version}-x86_64-unknown-linux-gnu.tar.gz',
              '{crate}-{version}-aarch64-unknown-linux-gnu.tar.gz',
            ]);
          },
        }),
        it({
          name: 'reads every optional option and keeps repeated asset templates in order',
          fn: async () => {
            expect(parse([
              ...requiredOptions,
              '--root',
              '/other',
              '--workflow',
              'publish.yml',
              '--repo',
              'owner/name',
              '--attempts',
              '3',
              '--poll-interval',
              '5',
              '--run-limit',
              '20',
              '--tag-template',
              'v{version}',
              '--asset-template',
              '{crate}-{version}.tar.gz',
              '--asset-template',
              '{crate}-{version}-sha256sums.txt',
              '--user-agent',
              'my-check',
              '--max-time',
              '9',
              sha,
              '1.0.0-rc.1',
            ])).toEqual({
              root: '/other',
              crateName: 'monochromatic-git-policy-cli',
              sha,
              version: '1.0.0-rc.1',
              workflow: 'publish.yml',
              repo: 'owner/name',
              attempts: 3,
              pollIntervalSeconds: 5,
              runLimit: 20,
              tagTemplate: 'v{version}',
              assetTemplates: ['{crate}-{version}.tar.gz', '{crate}-{version}-sha256sums.txt'],
              userAgent: 'my-check',
              maxTimeSeconds: 9,
            });
          },
        }),
        it({
          name: 'lowercases an upper-case hash',
          fn: async () => {
            expect(parse([...requiredOptions, 'ABCDEF0', '1.0.0']).sha).toBe('abcdef0');
          },
        }),
        it({
          name: 'refuses a missing crate name',
          fn: async () => {
            expectRefusal({ argv: [sha, '1.0.0'], mention: '--crate-name' });
          },
        }),
        it({
          name: 'refuses the wrong number of positional arguments and says what they are',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, sha], mention: '<sha> <version>' });
            expectRefusal({ argv: [...requiredOptions, sha, '1.0.0', 'extra'], mention: '<sha> <version>' });
          },
        }),
        it({
          name: 'refuses a hash that would change the meaning of a query',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '")) | .databaseId, ("', '1.0.0'], mention: '<sha>' });
          },
        }),
        it({
          name: 'refuses a hash that is too short to identify a commit',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, 'abc', '1.0.0'], mention: '<sha>' });
          },
        }),
        it({
          name: 'refuses a version that would change the meaning of a URL',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, sha, '1.0.0/../../x'], mention: '<version>' });
          },
        }),
        it({
          name: 'refuses a workflow name that gh would read as an option',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--workflow=-x', sha, '1.0.0'], mention: '--workflow' });
          },
        }),
        it({
          name: 'refuses a repository that is not owner/name',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--repo', 'owner', sha, '1.0.0'], mention: '--repo' });
            expectRefusal({ argv: [...requiredOptions, '--repo', 'a/b/c', sha, '1.0.0'], mention: '--repo' });
            expectRefusal({ argv: [...requiredOptions, '--repo', 'a b/c', sha, '1.0.0'], mention: '--repo' });
          },
        }),
        it({
          name: 'refuses attempts, interval, limit and time that are not positive whole numbers',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--attempts', '0', sha, '1.0.0'], mention: '--attempts' });
            expectRefusal({ argv: [...requiredOptions, '--poll-interval', '1.5', sha, '1.0.0'], mention: '--poll-interval' });
            expectRefusal({ argv: [...requiredOptions, '--run-limit', '-1', sha, '1.0.0'], mention: '--run-limit' });
            expectRefusal({ argv: [...requiredOptions, '--max-time', 'x', sha, '1.0.0'], mention: '--max-time' });
          },
        }),
        it({
          name: 'refuses a tag template without the version placeholder',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--tag-template', 'release', sha, '1.0.0'], mention: '--tag-template' });
          },
        }),
        it({
          name: 'refuses an asset template without the version placeholder',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--asset-template', 'a.tar.gz', sha, '1.0.0'], mention: '--asset-template' });
          },
        }),
        it({
          name: 'refuses a user agent with a line break',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--user-agent', 'a\nb', sha, '1.0.0'], mention: '--user-agent' });
          },
        }),
        it({
          name: 'refuses an unknown option and names it',
          fn: async () => {
            expectRefusal({ argv: [...requiredOptions, '--bogus', sha, '1.0.0'], mention: '--bogus' });
          },
        }),
      ],
    }),
  ],
});
