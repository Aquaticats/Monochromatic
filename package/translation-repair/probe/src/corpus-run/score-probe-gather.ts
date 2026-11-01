import { join, } from 'node:path';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type ArtifactProbeReading,
  readArtifactProbe,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { indexReadingsByIssue, } from '@monochromatic-dev/translation-repair-run/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { summarizeStageRoster, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  keepEligible,
  listArtifactFiles,
  type PoolPolicy,
  resolvePool,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { GatheredProbe, } from './probe-telemetry-report.ts';

//region Score probe gather
// Reads every settled artifact of a run into the probe readings and coverage
// counts the `score-probe` report prints.

/**
 One artifact's probe reading, kept beside the file name it came from.
 */
type NamedProbeReading = {
  readonly name: string;
  readonly reading: ArtifactProbeReading;
};

/**
 Reads every settled artifact of a run.

 @param artifactsDir - directory the pass writes entries into

 @param policy - generation policy the entry read from its environment, which
 decides which generations the pool admits (`readPoolPolicy`)

 @param repository - checkout a required commit is resolved in and every
 recorded commit is placed against, which the entry names: the package's own
 in a run, a throwaway one in a case

 @returns Readings and coverage counts across every artifact

 @throws {@link ArtifactParseError} when a present probe field is malformed,
 because a count nobody can trust is worse than no count

 @example
 ```ts
 const gathered = await gatherProbeReadings({ artifactsDir, policy, repository, },);
 ```
 */
export async function gatherProbeReadings(
  {
    artifactsDir,
    policy,
    repository,
  }: {
    readonly artifactsDir: string;
    readonly policy: PoolPolicy;
    readonly repository: string;
  },
): Promise<GatheredProbe> {
  /**
   One directory listing, shared with the census.

   Taken once and threaded through, because the accumulation writes into this
   directory continuously: a second listing inside the census would classify a
   different set of files from the one this reader goes on to read.
   */
  const listed = textsInCodePointOrder({ texts: (await listArtifactFiles({ artifactsDir, },)), },);

  /**
   Artifact file names, JSON only.
   */
  const names = keepEligible({
    names: listed,
    eligible: await resolvePool({
      artifactsDir,
      names: listed,
      policy,
      repository,
    },),
  },);

  /**
   One reading set per artifact, read concurrently and kept beside the name
   it came from, so no later step pairs the two by position.

   Every parse failure carries the artifact path it came from, so a malformed
   file names itself regardless of read order. Of several malformed files the
   one reported is the earliest in the sorted list, whichever read ended
   first, since `allInInputOrder` reports by position.
   */
  const perEntry = await allInInputOrder({
    members: names.map(async function toReading(name,): Promise<NamedProbeReading> {
      return {
        name,
        reading: readArtifactProbe({
          value: await readRunJson({
            path: join(
              artifactsDir,
              name,
            ),
          },),
          path: name,
        },),
      };
    },),
  },);

  /**
   Every reading paired with its owning issue, across every artifact.
   */
  const owned = perEntry.flatMap(function toOwned({ reading, },) {
    return reading.owned;
  },);

  /**
   Stage findings, one list per artifact.
   */
  const findingsPerEntry = perEntry.map(function toFindings({ reading, },) {
    return reading.findings;
  },);

  return {
    // Grouped rather than flattened, because envelope ids are derived from the
    // text they cover and so repeat across documents that share a paragraph.
    // Flattening let the summary collapse two entries' unrelated regions into
    // one.
    readings: perEntry.map(function toGroup({
      name,
      reading,
    },) {
      return {
        entryId: name,
        readings: reading.readings,
      };
    },),
    byIssueId: indexReadingsByIssue({ owned, },),
    // Issues whose slice the naturalness lane rewrote after the probe ran, so
    // the probe's verdict is about wording that did not ship.
    refinedIssueIds: new Set(owned
      .filter(function wasRefined(entry,) {
        return entry.refined;
      },)
      .map(function toIssueId(entry,) {
        return entry.issueId;
      },),),
    refinementReadings: perEntry.map(function toRefinementGroup({
      name,
      reading,
    },) {
      return {
        entryId: name,
        readings: reading.refinementReadings,
      };
    },),
    editorRoster: summarizeStageRoster({
      entries: findingsPerEntry,
      stage: 'editor',
    },),
    refineRoster: summarizeStageRoster({
      entries: findingsPerEntry,
      stage: 'refine',
    },),
    entriesWithRewrites: perEntry
      .filter(function rewroteSomething({ reading, },) {
        return reading.hasRewrites;
      },)
      .length,
    entries: perEntry.length,
    repairShippedRecords: perEntry.reduce(
      function addShipped(
        sum,
        { reading, },
      ) {
        return sum + reading.repairShippedRecords;
      },
      0,
    ),
    repairUnprobedRecords: perEntry.reduce(
      function addUnprobed(
        sum,
        { reading, },
      ) {
        return sum + reading.repairUnprobedRecords;
      },
      0,
    ),
  };
}

//endregion Score probe gather
