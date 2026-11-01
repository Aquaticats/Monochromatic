import {
  artifactsDirOf,
  type PoolPolicy,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { gatherAttributionEntries, } from './attribution-read.ts';
import { buildAttributionReport, } from './attribution-report.ts';
import { requireArtifactsDir, } from './score-artifacts-dir.ts';
import {
  attributionCriticLine,
  attributionHeaderLine,
} from './score-attribution-table.ts';
import {
  printMalformedArtifacts,
  printPartialJoinWarning,
  printUnattributedWarning,
} from './score-attribution-warnings.ts';

//region Score attribution run
// Reads critic attribution out of a run's settled artifacts and reports it as
// RATES rather than tallies: what each critic was asked, what it raised, and
// how often an accepted issue rested on it alone.
//
// Prints no corpus text. Claim ids are hashes and model ids are model ids, so
// this output is safe to paste anywhere the artifacts themselves are not.

/**
 Reads a run's artifacts and prints per-critic calibration.

 @param runsDir - runs directory the report describes, which the first line
 names

 @param policy - generation policy the entry read from its environment
 (`readPoolPolicy`), handed to the pool

 @param repository - checkout a required commit is resolved in and every
 recorded commit is placed against, which the entry names: the package's own
 in a run, a throwaway one in a case

 @throws {@link StatedRefusalError} when the runs directory holds no readable
 `artifacts` directory, after the first line has named it

 @example
 ```ts
 await printAttribution({ runsDir, policy, repository, },);
 ```
 */
export async function printAttribution(
  {
    runsDir,
    policy,
    repository,
  }: {
    readonly runsDir: string;
    readonly policy: PoolPolicy;
    readonly repository: string;
  },
): Promise<void> {
  /**
   Directory this run wrote artifacts into.
   */
  const artifactsDir = artifactsDirOf({ runsDir, },);

  // NAMES THE RUN IT READ, first line, always. `resolveRunsDir` falls back to a
  // default when TRANSLATION_REPAIR_RUNS_DIR is unset, so a report can describe
  // a different run than the reader has in mind and every count it prints will look
  // like an answer about theirs. Pointing this at the wrong directory produced a
  // clean set of zeros that read as "nothing to report" rather than as "wrong
  // run", which is the failure this whole project keeps rediscovering.
  console.log(`SOURCE ${artifactsDir}`,);
  await requireArtifactsDir({ artifactsDir, },);

  /**
   Entries that parsed, and the artifacts that did not.
   */
  const {
    entries,
    malformed,
  } = await gatherAttributionEntries({
    artifactsDir,
    policy,
    repository,
  },);

  /**
   Report over every settled artifact.
   */
  const report = buildAttributionReport({ entries, },);

  printMalformedArtifacts({ malformed, },);

  console.log(
    `POPULATION eligible=${String(report.eligibleEntries,)} `
      + `ineligible=${String(report.ineligibleEntries,)} `
      + `chunks=${String(report.chunks,)}`,
  );
  if (report.eligibleEntries === 0) {
    console.log(
      'NOTE no entry carries attribution yet. Entries settled before it '
        + 'existed record none, and they are excluded rather than counted as '
        + 'critics that raised nothing.',
    );
    return;
  }

  console.log(`\n${attributionHeaderLine()}`,);
  for (const critic of report.critics)
    console.log(attributionCriticLine({ critic, },),);

  console.log(
    `\nSUPPORT sole=${String(report.soleProposerAccepted,)} `
      + `multi=${String(report.multiProposerAccepted,)} `
      + `selfRepeated=${String(report.selfRepeatedAccepted,)} `
      + `unattributed=${String(report.unattributedAccepted,)} `
      + `partialJoin=${String(report.partialJoinAccepted,)}`,
  );
  console.log(
    'NOTE sole means an accepted issue rested on exactly one critic, which is '
      + 'legitimate: the reference run had gpt-oss-120b as the sole finder of a '
      + 'planted seed. selfRepeated means one critic emitted the same claim '
      + 'twice, which must never read as agreement. That distinction is what '
      + 'issue 65 asks about duplicates.',
  );
  printPartialJoinWarning({ count: report.partialJoinAccepted, },);
  printUnattributedWarning({ count: report.unattributedAccepted, },);
}

//endregion Score attribution run
