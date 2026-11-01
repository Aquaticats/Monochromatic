import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  AttemptRecord,
  AuxiliaryRecord,
} from './ledger-fixture.ts';
import { rejectionReachedPerson, } from './push-rejection-fixture.ts';

//region Fixtures

/**
 Notice the service leaves for a refused push, as a later wrapped command shows it.
 */
const NOTICE = 'The backup push of branch main failed for commit abcdef0.\n'
  + ' ! [rejected] main -> main (non-fast-forward)\nauto-push to origin failed\n';

/**
 Builds an attempt record.

 @param startedAt - monotonic start time

 @param stderr - complete stderr

 @returns attempt record

 @example
 ```ts
 attempt({ startedAt: 1, stderr: '' });
 ```
 */
function attempt({ startedAt, stderr, }: Readonly<{ startedAt: number; stderr: string; }>,): AttemptRecord {
  return {
    label: `w${startedAt}`,
    token: `E2E-TOKEN-${startedAt}`,
    mode: 'explicit',
    selectedPaths: ['a.txt',],
    captured: [],
    headBefore: '0'.repeat(40,),
    startedAt,
    expectedBranch: 'main',
    outcome: { exitCode: 0, stdout: '', stderr, durationMs: 1, },
    killed: false,
    requiresRemote: true,
  };
}

/**
 Builds a non-commit wrapped command.

 @param stderr - complete stderr

 @returns auxiliary record

 @example
 ```ts
 auxiliary('');
 ```
 */
function auxiliary(stderr: string,): AuxiliaryRecord {
  return { label: 'status', outcome: { exitCode: 0, stdout: '', stderr, durationMs: 1, }, mustSucceed: true, wrapper: true, };
}

//endregion Fixtures

await describe({
  name: rejectionReachedPerson.name,
  children: [
    it({
      name: 'accepts the rejection on the attempt\'s own stderr',
      fn: async () => {
        const own = attempt({ startedAt: 1, stderr: NOTICE, },);
        expect(rejectionReachedPerson({ attempt: own, attempts: [own,], auxiliaries: [], },),).toBe(true,);
      },
    },),
    it({
      name: 'accepts the notice a later wrapped command shows, from a commit or from any other wrapped command',
      fn: async () => {
        const own = attempt({ startedAt: 1, stderr: '', },);
        const later = attempt({ startedAt: 2, stderr: NOTICE, },);
        expect(rejectionReachedPerson({ attempt: own, attempts: [own, later,], auxiliaries: [], },),).toBe(true,);
        expect(rejectionReachedPerson({ attempt: own, attempts: [own,], auxiliaries: [auxiliary(NOTICE,),], },),).toBe(true,);
      },
    },),
    it({
      name: 'rejects a notice from an earlier command, from the foreign commit, or without the rejection',
      fn: async () => {
        const own = attempt({ startedAt: 5, stderr: '', },);
        const earlier = attempt({ startedAt: 2, stderr: NOTICE, },);
        const foreign = { ...attempt({ startedAt: 6, stderr: NOTICE, },), mode: 'foreign', } as const;
        expect(rejectionReachedPerson({ attempt: own, attempts: [own, earlier,], auxiliaries: [], },),).toBe(false,);
        expect(rejectionReachedPerson({ attempt: own, attempts: [own, foreign,], auxiliaries: [], },),).toBe(false,);
        expect(rejectionReachedPerson({ attempt: own, attempts: [own, attempt({ startedAt: 6, stderr: 'quiet', },),], auxiliaries: [], },),).toBe(false,);
      },
    },),
  ],
},);
