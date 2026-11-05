/**
 Showing the service's push failure notices to the harness's own wrapped command.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { isAbsent, } from './service-connection-fixture.ts';
import {
  runWrapper,
  type ScenarioActors,
} from './worker-fixture.ts';

/**
 Directory of the failure notices under the common Git directory (`NOTICE_DIRECTORY` in `src/native/push/push_failure_notice.rs`).
 */
const NOTICE_SEGMENTS: readonly string[] = [
  'monochromatic-git-policy-cli',
  'notices',
];

/**
 Runs one read-only wrapped command when the service left a push failure notice, so the notice reaches a stderr.

 The service pushes after the commit returned and leaves a notice for the next wrapped command in the repository to show,
 as a person's next command would.
 A scenario whose commands all finished before the push failed has no such command, so the observation runs one,
 after the service settled and before the ledger is read.
 It runs only when a notice exists, so a run whose pushes all worked records nothing extra.

 @param actors - scenario repository and ledger

 @throws the read's failure when the notice directory cannot be read for a reason other than being absent

 @example
 ```ts
 await showPushNotices({ repository, ledger });
 ```
 */
export async function showPushNotices(actors: ScenarioActors,): Promise<void> {
  /**
   Names of the notices left.
   */
  const notices: string[] = [];
  /**
   Directory the service leaves them in.
   */
  const directory = join(
    actors.repository
      .gitDir,
    ...NOTICE_SEGMENTS,
  );
  try {
    notices.push(...await readdir(directory,),);
  }
  catch (error: unknown) {
    // A repository whose pushes all worked has no notice directory; any other failure is real.
    if (!isAbsent(error,))
      throw error;
  }
  if (notices.length === 0)
    return;
  await runWrapper({
    ...actors,
    label: 'show push notices',
    args: [
      'status',
      '--short',
    ],
    mustSucceed: true,
  },);
}
