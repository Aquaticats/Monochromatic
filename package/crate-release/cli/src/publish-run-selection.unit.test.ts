import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import {
  type ManifestVersionReader,
  selectPublishRun,
} from './publish-run-selection.ts';

/**
 What the tests release: the version, the manifest that holds it and the time the release run was created.
 */
const version = '0.5.1';
const previousVersion = '0.5.0';
const manifestPath = 'package/git-policy/cli/Cargo.toml';
const releaseCreatedAt = '2026-11-01T14:00:00Z';

/**
 One listed run, as `gh run list` prints it.
 */
type ListedRun = {
  readonly databaseId: number;
  readonly headSha: string;
  readonly createdAt: string;
};

/**
 Runs of the scene of the failure this selection was written for, newest first as `gh` lists them: a run of an
 unrelated dispatch created after the release run but before the release's own, then the release's own, then a later
 dispatch, then a run that is older than the release run.
 */
const olderRun: ListedRun = { databaseId: 10, headSha: 'a'.repeat(40), createdAt: '2026-11-01T13:00:00Z' };
const unrelatedRun: ListedRun = { databaseId: 11, headSha: 'b'.repeat(40), createdAt: '2026-11-01T14:14:48Z' };
const ownRun: ListedRun = { databaseId: 12, headSha: 'c'.repeat(40), createdAt: '2026-11-01T14:15:05Z' };
const laterRun: ListedRun = { databaseId: 13, headSha: 'd'.repeat(40), createdAt: '2026-11-01T14:30:00Z' };
const sceneListing = JSON.stringify([laterRun, ownRun, unrelatedRun, olderRun]);

/**
 Versions the commits of the scene hold: only the bump and what follows it hold the released one.
 */
const sceneVersions = new Map<string, string>([
  [olderRun.headSha, previousVersion],
  [unrelatedRun.headSha, previousVersion],
  [ownRun.headSha, version],
  [laterRun.headSha, version],
]);

/**
 Build a reader that answers from a table and records the commits it was asked about.

 @param versions - version per commit; a commit absent from the table cannot be read
 @returns the reader and the commits asked about, in order
 */
function readerOf(versions: ReadonlyMap<string, string>) {
  const asked: string[] = [];
  async function readVersion(sha: string): ReturnType<ManifestVersionReader> {
    asked.push(sha);
    const held = versions.get(sha);
    return held === undefined ? missingValue(`HTTP 404 for ${sha}`) : foundValue(held);
  }
  return { readVersion, asked };
}

/**
 Select with the scene's settings.

 @param listing - JSON text of the run listing
 @param versions - version per commit
 @param notBefore - creation time of the release run
 @returns the selection and the commits asked about
 */
async function select({
  listing = sceneListing,
  versions = sceneVersions,
  notBefore = releaseCreatedAt,
}: {
  readonly listing?: string;
  readonly versions?: ReadonlyMap<string, string>;
  readonly notBefore?: string;
} = {}) {
  const { readVersion, asked } = readerOf(versions);
  const result: Lookup<{ readonly runId: string; readonly headSha: string }> = await selectPublishRun({
    listing,
    notBefore,
    version,
    manifestPath,
    readVersion,
  });
  return { result, asked };
}

/**
 The reason of a miss, or an empty text for a find.

 @param result - a selection
 @returns the reason
 */
function reasonOf(result: Lookup<unknown>): string {
  return result.found ? '' : result.reason;
}

await describe({
  name: '',
  children: [
    describe({
      name: selectPublishRun.name,
      children: [
        it({
          name: 'takes the release run own publish run, not an unrelated dispatch created earlier in the same window',
          fn: async () => {
            const { result } = await select();
            expect(result).toEqual({
              found: true,
              value: { runId: String(ownRun.databaseId), headSha: ownRun.headSha, createdAt: ownRun.createdAt },
            });
          },
        }),
        it({
          name: 'asks about the commits of the runs since the release run only, oldest first, never about an older run',
          fn: async () => {
            const { asked } = await select();
            expect(asked).toEqual([unrelatedRun.headSha, ownRun.headSha, laterRun.headSha]);
            expect(asked).not.toContain(olderRun.headSha);
          },
        }),
        it({
          name: 'takes a run built from a later commit than the bump, which still holds the released version',
          fn: async () => {
            const listing = JSON.stringify([laterRun, unrelatedRun]);
            const { result } = await select({ listing });
            expect(result.found ? result.value.runId : '').toBe(String(laterRun.databaseId));
          },
        }),
        it({
          name: 'takes the earliest of several runs that hold the released version',
          fn: async () => {
            const listing = JSON.stringify([laterRun, ownRun]);
            const { result } = await select({ listing });
            expect(result.found ? result.value.runId : '').toBe(String(ownRun.databaseId));
          },
        }),
        it({
          name: 'takes a run created at exactly the release run time, comparing instants not text',
          fn: async () => {
            const listing = JSON.stringify([{ ...ownRun, createdAt: '2026-11-01T16:00:00+02:00' }]);
            const { result } = await select({ listing, notBefore: releaseCreatedAt });
            expect(result.found).toBe(true);
          },
        }),
        it({
          name: 'never takes a run older than the release run, even when its commit holds the released version',
          fn: async () => {
            const listing = JSON.stringify([olderRun]);
            const versions = new Map([[olderRun.headSha, version]]);
            const { result, asked } = await select({ listing, versions });
            expect(result.found).toBe(false);
            expect(asked).toEqual([]);
            expect(reasonOf(result)).toContain(`run ${String(olderRun.databaseId)}`);
            expect(reasonOf(result)).toContain(`created before the release run (${releaseCreatedAt})`);
          },
        }),
        it({
          name: 'skips a run whose commit cannot be read and takes a later one that matches',
          fn: async () => {
            const versions = new Map([[ownRun.headSha, version]]);
            const { result } = await select({ versions });
            expect(result.found ? result.value.runId : '').toBe(String(ownRun.databaseId));
          },
        }),
        it({
          name: 'names the run, its commit and the failure when no commit can be read',
          fn: async () => {
            const { result } = await select({ versions: new Map() });
            expect(result.found).toBe(false);
            expect(reasonOf(result)).toContain(`run ${String(ownRun.databaseId)}`);
            expect(reasonOf(result)).toContain(`commit ${ownRun.headSha}`);
            expect(reasonOf(result)).toContain(`${manifestPath} could not be read at that commit: HTTP 404 for ${ownRun.headSha}`);
          },
        }),
        it({
          name: 'names what was looked for and why each run was passed over when none is the release run own',
          fn: async () => {
            const versions = new Map(sceneVersions).set(ownRun.headSha, previousVersion).set(laterRun.headSha, previousVersion);
            const { result } = await select({ versions });
            const reason = reasonOf(result);
            expect(result.found).toBe(false);
            expect(reason).toContain(`created at or after ${releaseCreatedAt}`);
            expect(reason).toContain(`${manifestPath} holds version ${version}`);
            for (const run of [unrelatedRun, ownRun, laterRun]) {
              expect(reason).toContain(`run ${String(run.databaseId)} (created ${run.createdAt}, commit ${run.headSha})`);
            }
            expect(reason).toContain(`holds version ${JSON.stringify(previousVersion)} in ${manifestPath}, not ${JSON.stringify(version)}`);
            expect(reason).toContain(`run ${String(olderRun.databaseId)}`);
          },
        }),
        it({
          name: 'says the listing holds no run when it is empty',
          fn: async () => {
            const { result, asked } = await select({ listing: '[]' });
            expect(result.found).toBe(false);
            expect(reasonOf(result)).toContain(`No listed run was created at or after ${releaseCreatedAt}`);
            expect(reasonOf(result)).toContain('The listing holds no run.');
            expect(asked).toEqual([]);
          },
        }),
        it({
          name: 'counts the entries of the listing that are not runs',
          fn: async () => {
            const entries: readonly unknown[] = [null, { databaseId: 1 }];
            const { result } = await select({ listing: JSON.stringify(entries) });
            expect(reasonOf(result)).toContain(`${String(entries.length)} entries of the listing lack`);
          },
        }),
        ...['', 'not json', '{}', 'null'].map(function malformedCase(listing) {
          return it({
            name: `reports a miss for the malformed listing ${JSON.stringify(listing)} without asking about any commit`,
            fn: async () => {
              const { result, asked } = await select({ listing });
              expect(result.found).toBe(false);
              expect(asked).toEqual([]);
            },
          });
        }),
      ],
    }),
  ],
});
