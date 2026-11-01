import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import type {
  CorpusPin,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { GradingCandidate, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  type ArtifactFileName,
  artifactsDirOf,
  type EligibleEntries,
  keepEligible,
  type PoolPolicy,
  resolvePool,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type BandedEntry,
  loadEntry,
} from './draw-entry-load.ts';
import { listSettledNames, } from './sample-artifact-names.ts';

//region Draw sample pool
// Reads every settled artifact of a runs directory into the banded entries a
// draw samples from, and the pool of accepted issues across them.

/**
 What one runs directory holds that a draw may sample.
 */
export type DrawPool = {
  /**
   Entries the pool admitted, with the commit and pipeline each recorded.
   */
  readonly eligible: EligibleEntries;

  /**
   Artifact file names present in the run and admitted.
   */
  readonly names: readonly ArtifactFileName[];

  /**
   Every settled entry banded with its candidates.
   */
  readonly entries: readonly BandedEntry[];

  /**
   The full accepted-issue pool across every entry.
   */
  readonly pool: readonly GradingCandidate[];
};

/**
 Reads the settled artifacts of a runs directory.

 @param runsDir - runs directory a pass wrote into

 @param pin - corpus clone and commit each entry's source is read at, the
 draw's own, so the bands and the commit the sheets record come from one pin

 @param readSource - reads a corpus file at the pin: `readCorpusFile` in a run

 @param policy - generation policy the entry read from its environment, which
 decides which generations the pool admits (`readPoolPolicy`)

 @param repository - checkout a required commit is resolved in and every
 recorded commit is placed against, which the entry names: the package's own
 in a run, a throwaway one in a case

 @returns The admitted entries and their pool

 @throws {@link StatedRefusalError} When the runs directory holds no artifacts
 directory, and whatever the pool and the entry reader refuse with

 @example
 ```ts
 const { entries, pool, } = await readDrawPool({ runsDir, pin, readSource: readCorpusFile, policy, repository, },);
 ```
 */
export async function readDrawPool(
  {
    runsDir,
    pin,
    readSource,
    policy,
    repository,
  }: {
    readonly runsDir: string;
    readonly pin: CorpusPin;
    readonly readSource: typeof readCorpusFile;
    readonly policy: PoolPolicy;
    readonly repository: string;
  },
): Promise<DrawPool> {
  /**
   Per-entry artifact directory.
   */
  const artifactsDir = artifactsDirOf({ runsDir, },);

  /**
   One directory listing, shared with the census.

   Taken once and threaded through, because the accumulation writes into this
   directory continuously: a second listing inside the census would classify a
   different set of files from the one this draw goes on to read, so an
   artifact arriving between the two would join the census while never
   entering the candidate pool.
   */
  const listed = textsInCodePointOrder({ texts: await listSettledNames({ runsDir, },), },);

  /**
   Entries this draw may pool, with the commit each recorded.
   */
  const eligible = await resolvePool({
    artifactsDir,
    names: listed,
    policy,
    repository,
  },);

  /**
   Artifact file names present in the run.
   */
  const names = keepEligible({
    names: listed,
    eligible,
  },);

  /**
   Every settled entry banded with its candidates.
   */
  const entries = await allInInputOrder({
    members: names.map(function load(name,) {
      return loadEntry({
        artifactsDir,
        name,
        eligible,
        pin,
        readSource,
      },);
    },),
  },);

  return {
    eligible,
    names,
    entries,
    pool: entries.flatMap(function candidates(entry,) {
      return entry.candidates;
    },),
  };
}

//endregion Draw sample pool
