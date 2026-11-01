import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import {
  type DispatchedRun,
  parseDispatchedRuns,
} from './run-results.ts';

/**
 Reads the version a commit holds in the crate's manifest.
 */
export type ManifestVersionReader = (sha: string) => Promise<Lookup<string>>;

/**
 What the selection decided about one run: the run, whether it is the release's, and the reason in words.
 */
type Verdict = {
  readonly run: DispatchedRun;
  readonly isOwn: boolean;
  readonly why: string;
};

/**
 Describe a run for a diagnostic: its identifier, when it was created and the commit it was built from.

 @param run - the run

 @returns the description

 @example
 ```ts
 describeRun({ runId: '1', headSha: 'abc', createdAt: '2026-11-01T10:00:00Z' });
 ```
 */
function describeRun(run: DispatchedRun): string {
  return `run ${run.runId} (created ${run.createdAt}, commit ${run.headSha})`;
}

/**
 Write the diagnostic for a listing that holds no run of the release.

 @param verdicts - what was decided about each listed run, oldest first

 @param unreadableEntries - how many entries of the listing were not runs

 @param looked - what was looked for, as a sentence

 @returns the diagnostic: what was looked for, then one line per run seen and why it was passed over

 @example
 ```ts
 describeMiss({ verdicts: [], unreadableEntries: 0, looked: 'No listed run matched.' });
 ```
 */
function describeMiss({
  verdicts,
  unreadableEntries,
  looked,
}: {
  readonly verdicts: readonly Verdict[];
  readonly unreadableEntries: number;
  readonly looked: string;
}): string {
  /**
   One line per run that was seen and passed over, with the reason, then one for the entries that were not runs.
   */
  const seen = [
    ...verdicts.map(function describeVerdict(verdict) {
      return `${describeRun(verdict.run)} ${verdict.why}`;
    }),
    ...Array.from(
      { length: unreadableEntries === 0 ? 0 : 1 },
      function describeUnreadable() {
        return `${String(unreadableEntries)} entries of the listing lack a run identifier, a commit or a creation time.`;
      },
    ),
  ];
  /**
   The diagnostic, one line each.
   */
  const lines = [
    looked,
    seen.length === 0 ? 'The listing holds no run.' : 'Runs seen and why each was passed over:',
    ...seen.map(function bullet(line) {
      return `  - ${line}`;
    }),
  ];
  return lines.join('\n');
}

/**
 Pick the publish run that one release started out of the manually started runs of the publish workflow.

 The list holds every manual run of the publish workflow, whoever started it, so the time alone cannot tell a release's
 run from another dispatch made in the same window. A run belongs to the release when it was created at or after the
 release run and the commit it was built from holds the target version in the crate's manifest: before the release
 pushes its bump, no commit holds the target version, and the release starts its publish run on `main` after the push,
 so the commit is the bump or a later one. The earliest such run is taken, because a later one can only be another
 dispatch made after the bump landed. The list of the runs does not say which crate a run publishes, so a dispatch for
 another crate made on `main` after the bump and before the release's own cannot be told apart.

 The manifest is read at the commit of every run created at or after the release run, all at once, because the list is
 bounded by the run limit. A run whose manifest cannot be read is passed over with its reason, because a commit that no
 longer exists would otherwise hide every later run.

 @param listing - JSON text printed by `gh run list --event workflow_dispatch --json databaseId,headSha,createdAt`

 @param notBefore - creation time of the release run, as the API wrote it

 @param version - version the release publishes

 @param manifestPath - repository-relative path of the crate's manifest, named in the diagnostic

 @param readVersion - reads the version a commit holds in that manifest

 @returns the run, or a miss that names every run seen and why it was passed over

 @example
 ```ts
 await selectPublishRun({ listing: '[]', notBefore: '2026-11-01T10:00:00Z', version: '1.0.1', manifestPath, readVersion });
 ```
 */
export async function selectPublishRun({
  listing,
  notBefore,
  version,
  manifestPath,
  readVersion,
}: {
  readonly listing: string;
  readonly notBefore: string;
  readonly version: string;
  readonly manifestPath: string;
  readonly readVersion: ManifestVersionReader;
}): Promise<Lookup<DispatchedRun>> {
  /**
   The runs of the listing, oldest first.
   */
  const parsed = parseDispatchedRuns(listing);
  if (!parsed.found)
    return parsed;
  /**
   Earliest start the run may have, as a time.
   */
  const earliest = Date.parse(notBefore);
  /**
   What was decided about each run, oldest first.
   */
  const verdicts = await Promise.all(parsed.value
    .runs
    .map(async function judge(run): Promise<Verdict> {
      if (Date.parse(run.createdAt) < earliest)
        return {
          run,
          isOwn: false,
          why: `was created before the release run (${notBefore}).`
        };
      /**
       The version the run's commit holds in the manifest, or why it is unknown.
       */
      const held = await readVersion(run.headSha);
      if (!held.found)
        return {
          run,
          isOwn: false,
          why: `cannot be judged, because ${manifestPath} could not be read at that commit: ${held.reason}`
        };
      return {
        run,
        isOwn: held.value === version,
        why: `holds version ${JSON.stringify(held.value)} in ${manifestPath}, ${
          held.value === version ? 'as released' : `not ${JSON.stringify(version)}`
        }.`
      };
    }));
  /**
   The earliest run that nothing speaks against.
   */
  const own = verdicts.find(function isOwn(verdict) {
    return verdict.isOwn;
  });
  return own === undefined
    ? missingValue(describeMiss({
      verdicts,
      unreadableEntries: parsed.value
        .unreadableEntries,
      looked:
        `No listed run was created at or after ${notBefore} from a commit whose ${manifestPath} holds version ${version}.`
    }))
    : foundValue(own.run);
}
