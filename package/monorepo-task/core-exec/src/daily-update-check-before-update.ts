/**
 The first pass of the daily update's package checks: every package is checked before anything is updated, so a check
 that already fails is known before the update touches anything.

 @module
 */

import { CHECK_PASS_DEADLINE_MS, } from './daily-update-check-bounds.ts';
import { runChecks, } from './daily-update-check-run.ts';
import { checksFromTaskNames, } from './daily-update-package-check-discovery.ts';
import type {
  BeforeUpdate,
  CheckEffects,
} from './daily-update-package-check-types.ts';
import type { UpdateGroup, } from './daily-update-plan.ts';
import type { DailyUpdateEffects, } from './daily-update-run-types.ts';

/**
 A file outside the repository with its text.
 */
type ExternalFile = {
  /**
   Absolute path.
   */
  readonly path: string;
  /**
   Text, empty when the file could not be read.
   */
  readonly text: string;
};

/**
 Reads the text of files outside the repository; a file that cannot be read counts as empty.

 @param effects - the outside world

 @param paths - absolute paths

 @returns path to text

 @example
 ```ts
 const texts = await readExternal({ effects, paths: ['/config/toolchain.lock.json',], },);
 ```
 */
export async function readExternal({
  effects,
  paths,
}: {
  readonly effects: DailyUpdateEffects;
  readonly paths: readonly string[];
},): Promise<ReadonlyMap<string, string>> {
  // Reading one file does not depend on reading another, so the files are read side by side.
  /**
   Each file with its text.
   */
  const entries = await Promise.all(paths.map(async function readOne(path,): Promise<ExternalFile> {
    try {
      return {
        path,
        text: await effects.readText(path,),
      };
    }
    catch (error) {
      effects.log(
        `a file outside the repository could not be read, so it counts as empty: ${path}: ${String(error,)}`,
      );
      return {
        path,
        text: '',
      };
    }
  },),);
  return new Map(entries.map(function keyed({
    path,
    text,
  },) {
    return [
      path,
      text,
    ] as const;
  },),);
}

/**
 Checks every package before the update.

 @param effects - the outside world of the update

 @param checkEffects - the outside world of the checks

 @param groups - the update's groups, whose outside files are read

 @returns what is needed after the update

 @example
 ```ts
 const before = await checkBeforeUpdate({ effects, checkEffects, groups, },);
 ```
 */
export async function checkBeforeUpdate({
  effects,
  checkEffects,
  groups,
}: {
  readonly effects: DailyUpdateEffects;
  readonly checkEffects: CheckEffects;
  readonly groups: readonly UpdateGroup[];
},): Promise<BeforeUpdate> {
  /**
   Commit the repository is at before the update.
   */
  const headBefore = await effects.headCommit();
  /**
   Files outside the repository, as they are before.
   */
  const externalBefore = await readExternal({
    effects,
    paths: groups.flatMap(function outside(group,) {
      return group.externalFiles;
    },),
  },);
  /**
   Checks of every package.
   */
  const checks = checksFromTaskNames({ taskNames: await checkEffects.listCheckTaskNames(), },);
  effects.log(`checking ${checks.length} package checks before the update`,);
  return {
    headBefore,
    externalBefore,
    checks,
    results: await runChecks({
      effects: checkEffects,
      checks,
      deadlineAt: checkEffects.now() + CHECK_PASS_DEADLINE_MS,
      log: effects.log,
    },),
  };
}
