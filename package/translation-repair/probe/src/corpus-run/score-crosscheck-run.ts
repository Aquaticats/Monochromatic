import {
  artifactsDirOf,
  type PoolPolicy,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { gatherAttributionEntries, } from './attribution-read.ts';
import {
  buildCrosscheckCensus,
  statusBreakdown,
} from './judge-crosscheck.ts';
import { MIN_JUDGED_CLAIMS, } from './judge-independence.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { requireArtifactsDir, } from './score-artifacts-dir.ts';
import {
  authorHeaderLine,
  authorLine,
  tallyAuthors,
} from './score-crosscheck-authors.ts';
import {
  printCrosscheckMalformed,
  printJoinFailureWarning,
  printUnjudgeableWarning,
} from './score-crosscheck-warnings.ts';

//region Score crosscheck run
// Reports the population a judge crosscheck would run over, WITHOUT spending a
// single call. The judging pass itself is expensive and contends with the
// corpus pass for the same per-model slots, so knowing in advance whether the
// population can carry a rate is worth more than starting one and finding out.
//
// Prints no corpus text. Claim ids are hashes and model ids are model ids, so
// this output is safe to paste anywhere the artifacts themselves are not.

/**
 Reads a run's artifacts and prints the crosscheck population.

 @param runsDir - runs directory the report describes, which the first line names

 @param roster - models available to judge, which decides who may judge each claim

 @param policy - generation policy the entry read from its environment
 (`readPoolPolicy`), handed to the pool

 @param repository - checkout a required commit is resolved in and every
 recorded commit is placed against, which the entry names: the package's own
 in a run, a throwaway one in a case

 @throws {@link StatedRefusalError} when the runs directory holds no readable
 `artifacts` directory, after the first line has named it

 @example
 ```ts
 await printCrosscheck({ runsDir, roster: RUN_MODELS.judgeModelIds, policy, repository, },);
 ```
 */
export async function printCrosscheck(
  {
    runsDir,
    roster,
    policy,
    repository,
  }: {
    readonly runsDir: string;
    readonly roster: readonly RosterModelId[];
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

  printCrosscheckMalformed({ malformed, },);

  /**
   The enumerated population, both arms.
   */
  const census = buildCrosscheckCensus({
    entries,
    roster,
  },);

  /**
   Judgeable and unjudgeable claims, bound to names so counting them reads as
   one member step rather than as a chain through the census.
   */
  const {
    items,
    unjudgeable,
  } = census;

  /**
   Claims per arm, counted separately because `undecided` belongs in no rate.
   */
  const accepted = items
    .filter(function isAccepted({ arm, },): boolean {
      return arm === 'accepted';
    },);

  /**
   Claims the panel decided against, the only legitimate control.
   */
  const control = items
    .filter(function isControl({ arm, },): boolean {
      return arm === 'control';
    },);

  /**
   Claims the panel declined to decide, held out of every rate.
   */
  const undecided = items
    .filter(function isUndecided({ arm, },): boolean {
      return arm === 'undecided';
    },);

  console.log(
    `POPULATION entries=${String(census.entriesCovered,)} `
      + `withoutAttribution=${String(census.entriesWithoutAttribution,)} `
      + `judgeable=${String(items.length,)} `
      + `unjudgeable=${String(unjudgeable.length,)} `
      + `legacyClaims=${String(census.unattributedLegacyClaims,)} `
      + `joinFailures=${String(census.unattributedJoinFailures,)}`,
  );
  printJoinFailureWarning({ count: census.unattributedJoinFailures, },);
  console.log(
    `ARMS accepted=${String(accepted.length,)} `
      + `control=${String(control.length,)} `
      + `undecided=${String(undecided.length,)}`,
  );
  console.log(
    'NOTE undecided is needs-human, held OUT of every rate rather than filed '
      + 'as control. Rejected means the panel decided against a claim, so a '
      + 'judge can agree or disagree with it; needs-human means the panel '
      + 'declined to decide, and agreement with a verdict never given is '
      + 'undefined. Those claims lean supported on this run, so folding them '
      + 'into control would fill it with claims the panel mostly believed. '
      + 'From repair cache version 34 needs-human also holds a claim a supported '
      + 'majority settled at neutral, which asserts no defect, so the accepted '
      + 'arm no longer carries those claims and reads higher by construction '
      + 'against earlier runs.',
  );

  if (items.length === 0) {
    // Which of three states left nothing to judge, because each says something
    // different: the whole roster authored every claim, the entries record
    // attribution that no issue joins to, or the entries predate attribution.
    if (unjudgeable.length > 0) {
      console.log(
        'NOTE every attributed claim was proposed by the whole roster, so no '
          + 'claim has a judge to seat and there is nothing to crosscheck.',
      );
      printUnjudgeableWarning({ count: unjudgeable.length, },);
      return;
    }
    console.log(
      (census.entriesWithoutAttribution < census.entriesCovered)
        ? 'NOTE the entries carry attribution, but none of their issues names '
          + 'an attributed claim, so there is nothing to crosscheck.'
        : 'NOTE no entry carries attribution yet, so no claim can have its author '
          + 'barred and there is nothing to crosscheck. Entries settled before '
          + 'attribution existed record no proposer.',
    );
    return;
  }

  // Control-arm and undecided claims broken down by why the panel did not
  // accept them.
  /**
   Statuses of the claims the panel did not accept, which the call words as
   nothing where it accepted every judgeable claim.
   */
  const breakdown = statusBreakdown({
    claims: [
      ...control,
      ...undecided,
    ],
  },);
  console.log(`NON-ACCEPTED BY STATUS ${((control.length + undecided.length) === 0) ? 'none' : breakdown}`,);

  console.log(`\n${authorHeaderLine()}`,);
  for (const row of tallyAuthors({ items, },))
    console.log(authorLine({ row, },),);

  console.log(
    `\nNOTE the floor column reads against MIN_JUDGED_CLAIMS=${
      String(MIN_JUDGED_CLAIMS,)
    }, which is a provisional guard rather than a calibrated threshold. An `
      + 'author clearing neither arm is not excluded from the run; it simply '
      + 'cannot carry a per-author rate yet.',
  );
  console.log(
    'NOTE this crosscheck can bar a claim\'s AUTHORS and cannot bar its '
      + 'adjudicators: the whole roster sits as critics, panel and judges, '
      + 'so no seat outside the panel exists to bar one with. It measures '
      + 'whether a verdict survives being re-asked without its author, never '
      + 'precision.',
  );
  printUnjudgeableWarning({ count: unjudgeable.length, },);
}

//endregion Score crosscheck run
