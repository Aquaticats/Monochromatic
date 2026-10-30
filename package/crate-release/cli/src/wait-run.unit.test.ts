import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ProgramCall } from './run-program.ts';
import type { WaitOptions } from './wait-options.ts';
import { waitForPublish } from './wait-run.ts';

/**
 Options of the fixture wait: three attempts, two seconds apart.
 */
const options: WaitOptions = {
  root: '/repo',
  crateName: 'c',
  sha: 'abcdef0',
  version: '0.4.10',
  workflow: 'cargo-publish.yml',
  repo: undefined,
  attempts: 3,
  pollIntervalSeconds: 2,
  runLimit: 10,
  tagTemplate: '{crate}-v{version}',
  assetTemplates: ['{crate}-{version}-a.tar.gz', '{crate}-{version}-b.tar.gz'],
  userAgent: 'agent',
  maxTimeSeconds: 30,
};

/**
 Run listing that holds the commit.
 */
const foundListing = JSON.stringify([{ databaseId: 42, headSha: 'abcdef0123456789abcdef0123456789abcdef01' }]);

/**
 Complete release, as `gh release view --json assets` prints it.
 */
const completeRelease = JSON.stringify({ assets: [{ name: 'c-0.4.10-a.tar.gz' }, { name: 'c-0.4.10-b.tar.gz' }] });

/**
 Published version, as the crates.io API prints it.
 */
const publishedVersion = JSON.stringify({ version: { created_at: '2026-10-29T12:00:00+00:00' } });

/**
 Scripted answers of the outside tools.
 */
type Script = {
  readonly listings?: readonly { readonly status: number; readonly stdout: string; readonly stderr: string }[];
  readonly watchStatus?: number;
  readonly crates?: string;
  readonly release?: { readonly status: number; readonly stdout: string; readonly stderr: string };
};

/**
 Run the wait against scripted tools.

 @param script - answers of gh and curl
 @param waitOptions - options of the wait
 @returns exit status, every call, every sleep and every printed line
 */
async function record({
  script,
  waitOptions = options,
}: {
  readonly script: Script;
  readonly waitOptions?: WaitOptions;
}) {
  const calls: ProgramCall[] = [];
  const sleeps: number[] = [];
  const printed: string[] = [];
  let listingIndex = 0;
  const status = await waitForPublish({
    options: waitOptions,
    run: async function run(call) {
      calls.push(call);
      if (call.program === 'curl')
        return { status: 0, stdout: script.crates ?? publishedVersion, stderr: '' };
      if (call.args[1] === 'list') {
        const answer = script.listings?.[Math.min(listingIndex, (script.listings.length) - 1)];
        listingIndex += 1;
        return answer ?? { status: 0, stdout: foundListing, stderr: '' };
      }
      if (call.args[1] === 'watch')
        return { status: script.watchStatus ?? 0, stdout: '', stderr: '' };
      return script.release ?? { status: 0, stdout: completeRelease, stderr: '' };
    },
    sleep: async function sleep(milliseconds) {
      sleeps.push(milliseconds);
    },
    print: function print(line) {
      printed.push(line);
    },
  });
  return { status, calls, sleeps, printed };
}

/**
 Listing answer that holds no run yet.
 */
const emptyListing = { status: 0, stdout: '[]', stderr: '' };

await describe({
  name: '',
  children: [
    describe({
      name: waitForPublish.name,
      children: [
        it({
          name: 'finds the run at once, watches it, checks crates.io and the release, and succeeds',
          fn: async () => {
            const result = await record({ script: {} });
            expect(result.status).toBe(0);
            expect(result.sleeps).toEqual([]);
            expect(result.printed).toEqual([
              'run 42',
              'watch exit 0',
              'crates.io 0.4.10: 2026-10-29T12:00:00+00:00',
              'release c-v0.4.10 assets: c-0.4.10-a.tar.gz, c-0.4.10-b.tar.gz',
              'PUBLISHED',
            ]);
          },
        }),
        it({
          name: 'issues exactly the gh and curl calls with the repository root as working directory',
          fn: async () => {
            const result = await record({ script: {} });
            expect(result.calls).toEqual([
              {
                program: 'gh',
                args: ['run', 'list', '--workflow=cargo-publish.yml', '--limit=10', '--json=databaseId,headSha'],
                cwd: '/repo',
              },
              { program: 'gh', args: ['run', 'watch', '42', '--exit-status'], cwd: '/repo' },
              {
                program: 'curl',
                args: [
                  '--silent',
                  '--show-error',
                  '--max-time',
                  '30',
                  '--user-agent',
                  'agent',
                  'https://crates.io/api/v1/crates/c/0.4.10',
                ],
                cwd: '/repo',
              },
              { program: 'gh', args: ['release', 'view', 'c-v0.4.10', '--json=assets'], cwd: '/repo' },
            ]);
          },
        }),
        it({
          name: 'passes the repository to every gh call when one is named',
          fn: async () => {
            const result = await record({ script: {}, waitOptions: { ...options, repo: 'o/n' } });
            const ghCalls = result.calls.filter(function isGh(call) {
              return call.program === 'gh';
            });
            expect(ghCalls).toHaveLength(3);
            for (const call of ghCalls)
              expect(call.args).toContain('--repo=o/n');
          },
        }),
        it({
          name: 'waits the poll interval between attempts until the run appears',
          fn: async () => {
            const result = await record({ script: { listings: [emptyListing, emptyListing, { status: 0, stdout: foundListing, stderr: '' }] } });
            expect(result.status).toBe(0);
            expect(result.sleeps).toEqual([2000, 2000]);
          },
        }),
        it({
          name: 'gives up after the last attempt without sleeping after it and names the bound',
          fn: async () => {
            const result = await record({ script: { listings: [emptyListing] } });
            expect(result.status).toBe(1);
            expect(result.sleeps).toEqual([2000, 2000]);
            expect(result.printed).toEqual(['no cargo-publish.yml run for abcdef0 within 6 seconds']);
            expect(result.calls.filter(function isList(call) {
              return call.args[1] === 'list';
            })).toHaveLength(3);
          },
        }),
        it({
          name: 'shows the last gh error when listing failed every time',
          fn: async () => {
            const result = await record({
              script: { listings: [{ status: 1, stdout: '', stderr: 'gh: not logged in' }] },
            });
            expect(result.status).toBe(1);
            expect(result.printed.join('\n')).toContain('gh: not logged in');
          },
        }),
        it({
          name: 'keeps trying after a listing error and succeeds when a later attempt finds the run',
          fn: async () => {
            const result = await record({
              script: { listings: [{ status: 1, stdout: '', stderr: 'timeout' }, { status: 0, stdout: foundListing, stderr: '' }] },
            });
            expect(result.status).toBe(0);
            expect(result.sleeps).toEqual([2000]);
          },
        }),
        it({
          name: 'still checks crates.io and the release when the workflow run failed, and fails',
          fn: async () => {
            const result = await record({ script: { watchStatus: 1 } });
            expect(result.status).toBe(1);
            expect(result.printed).toContain('watch exit 1');
            expect(result.printed.join('\n')).toContain('crates.io 0.4.10:');
            expect(result.printed.at(-1)).toContain('PUBLISH CHECK FAILED');
          },
        }),
        it({
          name: 'fails when crates.io does not know the version',
          fn: async () => {
            const result = await record({ script: { crates: '{"errors":[{"detail":"Not Found"}]}' } });
            expect(result.status).toBe(1);
            expect(result.printed).toContain('crates.io 0.4.10: not found');
          },
        }),
        it({
          name: 'fails when the release lacks an expected tarball',
          fn: async () => {
            const result = await record({
              script: { release: { status: 0, stdout: JSON.stringify({ assets: [{ name: 'c-0.4.10-a.tar.gz' }] }), stderr: '' } },
            });
            expect(result.status).toBe(1);
            expect(result.printed.at(-1)).toContain('c-0.4.10-b.tar.gz');
          },
        }),
        it({
          name: 'shows the gh error when the release does not exist',
          fn: async () => {
            const result = await record({
              script: { release: { status: 1, stdout: '', stderr: 'release not found' } },
            });
            expect(result.status).toBe(1);
            expect(result.printed).toContain('release c-v0.4.10 assets: release not found');
          },
        }),
      ],
    }),
  ],
});
