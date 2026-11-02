import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isAlreadyCommitted,
  isReportedConflict,
} from './scenario-model-fixture.ts';
import {
  isBranchKeptChanging,
  isPrunedBeforeLanding,
} from './scenario-outcome-fixture.ts';

//region Fixtures

/**
 A settled attempt reduced to what the conflict check reads.

 @param fields - exit code and standard error of the attempt

 @returns the attempt of the path `shared.txt`

 @example
 ```ts
 attemptOf({ exitCode: 1, stderr: '' });
 ```
 */
function attemptOf({
  exitCode,
  stderr,
}: Readonly<{
  exitCode: number;
  stderr: string;
}>,): Parameters<typeof isReportedConflict>[0] {
  return {
    selectedPaths: ['shared.txt',],
    outcome: {
      exitCode,
      stdout: '',
      stderr,
      durationMs: 1,
    },
  };
}

/**
 The finding line the wrapper prints for a commit whose captured change overlaps what landed.
 */
const FINDING = '{"type":"core-finding","code":"concurrent-commit/replay-conflict","message":"... `shared.txt` ..."}\n';

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: isReportedConflict.name,
      children: [
        it({
          name: 'accepts exit 1 with the replay-conflict finding that names one of the attempt\'s paths',
          fn: async () => {
            expect(isReportedConflict(attemptOf({ exitCode: 1, stderr: FINDING, },),),).toBe(true,);
          },
        },),
        it({
          name: 'rejects another exit code, another finding, and a finding that names no path of the attempt',
          fn: async () => {
            expect(isReportedConflict(attemptOf({ exitCode: 0, stderr: FINDING, },),),).toBe(false,);
            expect(isReportedConflict(attemptOf({ exitCode: 2, stderr: FINDING, },),),).toBe(false,);
            expect(isReportedConflict(attemptOf({
              exitCode: 1,
              stderr: FINDING.replace('replay-conflict', 'already-committed',),
            },),),).toBe(false,);
            expect(isReportedConflict(attemptOf({
              exitCode: 1,
              stderr: FINDING.replace('shared.txt', 'other.txt',),
            },),),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isAlreadyCommitted.name,
      children: [
        it({
          name: 'accepts exit 1 with the already-committed finding and nothing else',
          fn: async () => {
            const finding = FINDING.replace('replay-conflict', 'already-committed',);
            expect(isAlreadyCommitted(attemptOf({ exitCode: 1, stderr: finding, },),),).toBe(true,);
            expect(isAlreadyCommitted(attemptOf({ exitCode: 0, stderr: finding, },),),).toBe(false,);
            expect(isAlreadyCommitted(attemptOf({ exitCode: 1, stderr: FINDING, },),),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isBranchKeptChanging.name,
      children: [
        it({
          name: 'accepts exit 1 with the bounded-retry message and nothing else',
          fn: async () => {
            const message = 'The branch main changed 3 times while this commit was being prepared, so it was not made. Run the commit again.\n';
            expect(isBranchKeptChanging(attemptOf({ exitCode: 1, stderr: message, },),),).toBe(true,);
            expect(isBranchKeptChanging(attemptOf({ exitCode: 0, stderr: message, },),),).toBe(false,);
            expect(isBranchKeptChanging(attemptOf({ exitCode: 1, stderr: FINDING, },),),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isPrunedBeforeLanding.name,
      children: [
        it({
          name: 'accepts exit 1 with the message that names the prune and nothing else',
          fn: async () => {
            const message = 'The files of this commit were removed from the repository while the commit was being made, so it was not made.\n';
            expect(isPrunedBeforeLanding(attemptOf({ exitCode: 1, stderr: message, },),),).toBe(true,);
            expect(isPrunedBeforeLanding(attemptOf({ exitCode: 0, stderr: message, },),),).toBe(false,);
            expect(isPrunedBeforeLanding(attemptOf({ exitCode: 1, stderr: FINDING, },),),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
