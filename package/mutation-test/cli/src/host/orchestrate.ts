/**
 Host orchestration: enumerate, shard, run, reshard on taint, confirm.
 
 Trust model: killed results are accepted from any shard position;
 survived and timeout results are confirmed as mutant number 1 in a
 fresh container unless they already ran at position 1. Taint-abandoned
 remainders reshard at half size, bottoming out at single-mutant shards,
 which guarantees termination.
 
 @example
 ```ts
 const outcome = await orchestrateRun(options);
 ```
 */

import pLimit from 'p-limit';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { enumeratePackage, } from './enumerate-package.ts';
import { ensureRuntimeImage, } from './image.ts';
import { runOneShard, } from './run-one-shard.ts';
import {
  composeReshard,
  composeShards,
} from './shards.ts';
import type { Mutant, } from '../engine/types.ts';
import type {
  FinalMutantResult,
  MutantEntry,
  OrchestrateOptions,
  RunOutcome,
} from './orchestrate-types.ts';
import type {
  ShardManifest,
  ShardReport,
} from '../shard-schema.ts';

export type {
  FinalMutantResult,
  OrchestrateOptions,
  RunOutcome,
} from './orchestrate-types.ts';

/**
 Module logger for host orchestration.
 */
const l = tagged({ tag: 'mutation-test', },);

/**
 Retries allowed for a single-mutant shard whose container fails
 without producing a usable result; afterwards the mutant is final
 runtimeError so the reshard loop always terminates.
 */
const MAX_SINGLE_RETRIES = 1;

/**
 Runs the full orchestration flow against a prepared image.
 
 @param options - Run options.
 
 @returns Final per-mutant results with provenance.
 
 @throws Error when a shard reports results for unknown mutant ids or
 mutants remain unresolved after all rounds.
 
 @mutates options - `JSON.stringify` may invoke hooks on shard manifests derived from options.
 
 @example
 ```ts
 const outcome = await orchestrateRun(options);
 ```
 */
export async function orchestrateRun(options: OrchestrateOptions,): Promise<RunOutcome> {
  /**
   Logger scoped to the run.
   */
  const rl = tagged({
    tag: orchestrateRun.name,
    l,
  },);
  /**
   Enumerated per-file groups and suppression-ignored mutants.
   */
  const {
    groups: allGroups,
    ignored,
  } = await enumeratePackage(options,);
  /**
   Groups whose selected-test set can actually kill mutants; test-less
   files short-circuit below instead of burning containers on mutants
   nothing can detect.
   */
  const groups = allGroups.filter(function hasTests(group,): boolean {
    return group.tests
      .length
      > 0;
  },);
  /**
   Mutant lookup by id for reshard rounds.
   */
  const mutantById = new Map<string, MutantEntry>(
    groups.flatMap(function toEntries(group,): readonly (readonly [
      string,
      MutantEntry,
    ])[] {
      return group.mutants
        .map(function toEntry(mutant,): readonly [
          string,
          MutantEntry,
        ] {
          return [
            mutant.id,
            {
              mutant,
              tests: group.tests,
            },
          ];
        },);
    },),
  );
  /**
   Usable runtime image reference.
   */
  const image = await ensureRuntimeImage({
    repoRoot: options.repoRoot,
    skipImageBuild: options.skipImageBuild,
  },);
  /**
   Bounded concurrency gate across shard containers.
   */
  const limit = pLimit(options.containers,);
  /**
   Final result per mutant id.
   */
  const finals = new Map<string, FinalMutantResult>();
  /**
   Reshard round counter per mutant id.
   */
  const rerunCounts = new Map<string, number>();
  /**
   Container-failure retries per mutant id at shard size 1.
   */
  const singleRetries = new Map<string, number>();
  /**
   Infrastructure anomalies observed across shards.
   */
  const infraErrors: string[] = [];
  /**
   Total shard containers launched.
   */
  const counters = { shards: 0, };

  for (const group of allGroups) {
    if (group.tests
      .length
      > 0)
      continue;

    for (const mutant of group.mutants) {
      finals.set(
        mutant.id,
        {
          mutant,
          status: 'survived',
          position: 1,
          rerunCount: 0,
          confirmed: true,
          detail: 'no tests select this file; mutant trivially survives',
        },
      );
    }
  }

  /**
   Fetches one mutant entry, throwing on unknown ids.
   
   @param id - Mutant id from a shard report.
   
   @returns Lookup entry.
   */
  function entryOrThrow(id: string,): MutantEntry {
    /**
     Lookup entry for this id.
     */
    const entry = mutantById.get(id,);

    if (entry === undefined)
      throw new Error(`unknown mutant id ${id} in shard results`,);

    return entry;
  }

  /**
   Records taint fallout for one unrun id, finalising ids whose
   single-mutant containers keep failing.
   
   @param options2 - Unrun id and the shard size it failed under.
   
   @returns Whether the id still needs resharding.
   */
  function recordUnrun(options2: {
    readonly id: string;
    readonly failedSize: number;
  },): boolean {
    rerunCounts.set(
      options2.id,
      (rerunCounts.get(options2.id,) ?? 0) + 1,
    );

    if (options2.failedSize > 1)
      return true;

    /**
     Container-failure retries already spent on this mutant.
     */
    const retries = singleRetries.get(options2.id,) ?? 0;

    if (retries < MAX_SINGLE_RETRIES) {
      singleRetries.set(
        options2.id,
        retries + 1,
      );
      return true;
    }

    finals.set(
      options2.id,
      {
        mutant: entryOrThrow(options2.id,)
          .mutant,
        status: 'runtimeError',
        position: 1,
        rerunCount: rerunCounts.get(options2.id,) ?? 0,
        confirmed: true,
        detail: 'container failed repeatedly on single-mutant shard',
      },
    );
    return false;
  }

  /**
   Runs one round of manifests, returning ids needing another round.
   
   @param manifests - Manifests for this round.
   
   @returns Mutant ids to reshard.
   
   @mutates manifests - `JSON.stringify` may invoke hooks on shard manifests.
   */
  async function runRound(manifests: readonly ShardManifest[],): Promise<readonly string[]> {
    /**
     Reports from every shard in this round.
     */
    const reports = await Promise.all(manifests.map(
      /**
       Runs one manifest through bounded container concurrency.
       
       @param manifest - Shard manifest that may expose serialization hooks.
       
       @returns completed shard report.
       
       @mutates manifest - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
       */
      async function runLimited(manifest,): Promise<ShardReport> {
        counters.shards += 1;
        return await limit(
          async function launch(): Promise<ShardReport> {
            return await runOneShard({
              manifest,
              image,
              repoRoot: options.repoRoot,
              resources: options.resources,
              selinuxRelabel: options.selinuxRelabel,
            },);
          },
        );
      },
    ),);
    /**
     Ids requiring resharding after this round.
     */
    const needsReshard: string[] = [];

    for (const [
      manifestIndex,
      report,
    ] of reports.entries()) {
      if (!report.baseline
        .green) {
        // Red baseline: this shard's selected tests fail unmutated, so no
        // mutant verdict is possible; resharding would re-fail the same
        // baseline forever. Finalise the shard's mutants as runtimeError
        // and let the rest of the run proceed.
        infraErrors.push(`${report.shardId}: red baseline (${report.baseline
          .detail})`,);

        for (const id of report.unrun) {
          finals.set(
            id,
            {
              mutant: entryOrThrow(id,)
                .mutant,
              status: 'runtimeError',
              position: 1,
              rerunCount: rerunCounts.get(id,) ?? 0,
              confirmed: true,
              detail: 'red baseline: selected tests fail on unmutated code',
            },
          );
        }
        continue;
      }

      if (report.anomaly !== '')
        infraErrors.push(`${report.shardId}: ${report.anomaly}`,);

      for (const result of report.results) {
        finals.set(
          result.id,
          {
            mutant: entryOrThrow(result.id,)
              .mutant,
            status: result.status,
            position: result.position,
            rerunCount: rerunCounts.get(result.id,) ?? 0,
            confirmed: result.position === 1,
            detail: result.detail,
          },
        );
      }

      for (const id of report.unrun) {
        if (recordUnrun({
          id,
          failedSize: manifests[manifestIndex]
            ?.mutants
            .length
            ?? 1,
        },))
          needsReshard.push(id,);
      }
    }

    return needsReshard;
  }

  /* oxlint-disable no-await-in-loop */
  // Rounds are inherently sequential: each consumes the previous round's
  // taint fallout. Parallelism lives inside runRound via the p-limit gate.
  {
    /**
     Mutable round state: manifests to run and their shard size.
     */
    const round = {
      manifests: composeShards({
        groups,
        shardSize: options.shardSize,
        timeoutFloorMs: options.timeoutFloorMs,
        timeoutFactor: options.timeoutFactor,
        packagePath: options.packagePath,
      },),
      size: options.shardSize,
    };

    while (round.manifests
      .length
      > 0) {
      rl.info(
        `round: ${String(round.manifests
          .length,)} shards at size ${String(round.size,)}`,
      );

      /**
       Ids needing another round after taint fallout.
       */
      const ids = await runRound(round.manifests,);
      round.size = Math.max(
        1,
        Math.floor(round.size / 2,),
      );
      round.manifests = composeReshard({
        entryOf: entryOrThrow,
        timeoutFloorMs: options.timeoutFloorMs,
        timeoutFactor: options.timeoutFactor,
        packagePath: options.packagePath,
        ids,
        size: round.size,
      },);
    }

    /**
     Survived or timed-out results needing position-1 confirmation.
     */
    const toConfirm = [...finals.values(),]
      .filter(function needsConfirmation(result,): boolean {
        return ((result.status === 'survived') || (result.status === 'timeout'))
          && (!result.confirmed);
      },);

    if (toConfirm.length > 0) {
      rl.info(`confirming ${String(toConfirm.length,)} survivors/timeouts in fresh containers`,);

      /**
       Mutable confirmation state; failed confirmation containers loop
       until MAX_SINGLE_RETRIES finalises them as runtimeError.
       */
      const confirmation = {
        ids: toConfirm.map(function toId(result,): string {
          return result.mutant
            .id;
        },),
      };

      while (confirmation.ids
        .length
        > 0) {
        confirmation.ids = [...await runRound(composeReshard({
          entryOf: entryOrThrow,
          timeoutFloorMs: options.timeoutFloorMs,
          timeoutFactor: options.timeoutFactor,
          packagePath: options.packagePath,
          ids: confirmation.ids,
          size: 1,
        },),),];
      }
    }
  }
  /* oxlint-enable no-await-in-loop */

  /**
   Mutant ids with no final result after all rounds; must be empty.
   */
  const missing = [...mutantById.keys(),]
    .filter(function unresolved(id,): boolean {
      return !finals.has(id,);
    },);

  if (missing.length > 0)
    throw new Error(`run finished with ${String(missing.length,)} unresolved mutants`,);

  return {
    results: [...finals.values(),]
      .toSorted(function byFileLine(
        a,
        b,
      ): number {
        return (a.mutant
          .file
          < b.mutant
            .file ? -1 : (a.mutant
              .file
              > b.mutant
                .file ? 1 : 0))
          || (a.mutant
            .start
            - b.mutant
              .start);
      },),
    ignored,
    infraErrors,
    shardCount: counters.shards,
  };
}
