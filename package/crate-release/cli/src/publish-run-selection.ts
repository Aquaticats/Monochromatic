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
 Pick the publish run that one release started out of the manually started runs of the publish workflow.

 The list holds every manual run of the publish workflow, whoever started it, so the time alone cannot tell a release's
 run from another dispatch made in the same window. A run belongs to the release when it was created at or after the
 release run and the commit it was built from holds the target version in the crate's manifest: before the release
 pushes its bump, no commit holds the target version, and the release starts its publish run on `main` after the push,
 so the commit is the bump or a later one. The earliest such run is taken, because a later one can only be another
 dispatch made after the bump landed. The list of the runs does not say which crate a run publishes, so a dispatch for
 another crate made on `main` after the bump and before the release's own cannot be told apart.

 The runs are asked about oldest first and the asking stops at the first match, so a long list costs few calls. A run
 whose manifest cannot be read is skipped with its reason, because a commit that no longer exists would otherwise hide
 every later run.

 @param listing - JSON text printed by `gh run list --event workflow_dispatch --json databaseId,headSha,createdAt`

 @param notBefore - creation time of the release run, as the API wrote it

 @param version - version the release publishes

 @param manifestPath - repository-relative path of the crate's manifest, named in the diagnostic

 @param readVersion - reads the version a commit holds in that manifest

 @returns the run, or a miss that names every run seen and why it was skipped

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
   One line per run that was seen and passed over, with the reason.
   */
  const skipped: string[] = [];
  for (const run of parsed.value
    .runs) {
    if (Date.parse(run.createdAt) < earliest) {
      skipped.push(`${describeRun(run)} was created before the release run (${notBefore}).`);
      continue;
    }
    /**
     The version the run's commit holds in the manifest, or why it is unknown.
     */
    const held = await readVersion(run.headSha);
    if (!held.found) {
      skipped.push(`${describeRun(run)}: ${manifestPath} could not be read at that commit: ${held.reason}`);
      continue;
    }
    if (held.value === version)
      return foundValue(run);
    skipped.push(
      `${describeRun(run)} holds version ${JSON.stringify(held.value)} in ${manifestPath}, not ${JSON.stringify(version)}.`,
    );
  }
  if (parsed.value
    .unreadableEntries
    > 0)
    skipped.push(
      `${String(parsed.value
        .unreadableEntries)} entries of the listing lack a run identifier, a commit or a creation time.`,
    );
  return missingValue([
    `No listed run was created at or after ${notBefore} from a commit whose ${manifestPath} holds version ${version}.`,
    skipped.length === 0 ? 'The listing holds no run.' : 'Runs seen and why each was passed over:',
    ...skipped.map(function bullet(line) {
      return `  - ${line}`;
    }),
  ].join('\n'));
}
