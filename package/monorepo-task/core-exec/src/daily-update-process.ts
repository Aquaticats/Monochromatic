/**
 The daily update's process: options, the run lock and the real effects around {@link runDailyUpdate}.
 The entry that calls {@link main} is `daily-update.ts`.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { parseArgs, } from 'node:util';

import { realCheckEffects, } from './daily-update-check-effects.ts';
import { runUpdateWithChecks, } from './daily-update-check-phase.ts';
import { realEffects, } from './daily-update-effects.ts';
import { takeRunLock, } from './daily-update-lock.ts';
import { dailyUpdateGroups, } from './daily-update-plan.ts';

/**
 Runs the daily update with the real effects.

 @param argv - command-line arguments after the program name

 @returns the process exit code: 1 when a group failed or another run holds the lock, otherwise 0

 @example
 ```ts
 process.exitCode = await main(process.argv.slice(2,),);
 ```
 */
export async function main(argv: readonly string[],): Promise<number> {
  /**
   Options after `--repository` and `--global-lock`.
   */
  const { values, } = parseArgs({
    args: [...argv,],
    options: {
      repository: { type: 'string', },
      'global-lock': { type: 'string', },
    },
    strict: true,
  },);
  /**
   Absolute path of the repository, the working directory by default.
   */
  const repository = values.repository ?? process.cwd();
  /**
   Path of the machine-global lock the global group restores on failure.
   */
  const globalLock = values['global-lock']
    ?? join(
      process.env
        .XDG_CONFIG_HOME
        ?? join(
          homedir(),
          '.config',
        ),
      'monochromatic-dev',
      'toolchain',
      'toolchain.lock.json',
    );
  /**
   The run lock, released when this function ends.
   */
  await using lock = await takeRunLock(join(
    repository,
    '.cache',
    'toolchain-daily-update.lock',
  ),);
  if (!lock.acquired) {
    process.stderr
      .write('daily-update: another run holds the lock; this one does nothing.\n',);
    return 1;
  }
  /**
   What the groups did and what the package checks found; only the groups decide the exit code.
   */
  const result = await runUpdateWithChecks({
    effects: realEffects(repository,),
    checkEffects: realCheckEffects(repository,),
    groups: dailyUpdateGroups({ globalLockPath: globalLock, },),
  },);
  return result.exitCode;
}
