/**
 Read of the commit an attempt landed, found by the token its message carries.

 @module
 */

import type { AttemptRecord, } from './ledger-fixture.ts';
import { realGit, } from './repository-fixture.ts';
import type { ScenarioContext, } from './scenario-model-fixture.ts';

/**
 Commit carrying an attempt's token on the local branches.

 @param context - scenario context

 @param attempt - finished attempt

 @returns commit ID, empty when none landed

 @example
 ```ts
 const oid = await landedCommit({ context, attempt: victimRecord });
 ```
 */
export async function landedCommit({
  context,
  attempt,
}: Readonly<{
  context: ScenarioContext;
  attempt: AttemptRecord;
}>,): Promise<string> {
  return (await realGit({
    repository: context.repository,
    args: [
      'log',
      '--branches',
      '--format=%H',
      '--fixed-strings',
      `--grep=[${attempt.token}]`,
    ],
  },)).trim();
}
