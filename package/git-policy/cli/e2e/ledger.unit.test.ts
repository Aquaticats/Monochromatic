import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type AttemptRecord,
  contentOf,
  createLedger,
  workloadDigest,
} from './ledger-fixture.ts';

//region Fixtures

/**
 Builds an attempt record.

 @param label - attempt label

 @param bytes - captured bytes of `a.txt`

 @param exitCode - process exit code

 @returns attempt record

 @example
 ```ts
 attempt({ label: 'w0', bytes: 'a', exitCode: 0 });
 ```
 */
function attempt({
  label,
  bytes,
  exitCode,
}: Readonly<{
  label: string;
  bytes: string;
  exitCode: number;
}>,): AttemptRecord {
  return {
    label,
    token: `E2E-TOKEN-${label}`,
    mode: 'explicit',
    selectedPaths: ['a.txt',],
    captured: [{ path: 'a.txt', bytes: Buffer.from(bytes,), },],
    headBefore: String(exitCode,).repeat(40,),
    startedAt: exitCode,
    expectedBranch: 'main',
    outcome: { exitCode, stdout: '', stderr: '', durationMs: exitCode, },
    killed: false,
    requiresRemote: true,
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: contentOf.name,
      children: [
        it({
          name: 'distinguishes absent, empty, and different bytes',
          fn: async () => {
            expect(contentOf(),).toEqual({ state: 'absent', },);
            expect(
              contentOf(Buffer.from('',),),
            ).not.toEqual(
              contentOf(Buffer.from('x',),),
            );
            expect(
              contentOf(Buffer.from('x',),),
            ).toEqual(
              contentOf(Buffer.from('x',),),
            );
          },
        },),
      ],
    },),
    describe({
      name: createLedger.name,
      children: [
        it({
          name: 'records latest worktree intent, staging, attempts, and auxiliaries',
          fn: async () => {
            const ledger = createLedger();
            ledger.recordWorktree({ path: 'a.txt', bytes: Buffer.from('1',), },);
            ledger.recordWorktree({ path: 'a.txt', },);
            ledger.recordStaged({ path: 'b.txt', staged: true, },);
            ledger.recordStaged({ path: 'c.txt', staged: true, },);
            ledger.recordStaged({ path: 'c.txt', staged: false, },);
            ledger.addAttempt(attempt({ label: 'w0', bytes: 'a', exitCode: 0, },),);
            ledger.addAuxiliary({ label: 'add', outcome: attempt({ label: 'x', bytes: '', exitCode: 0, },).outcome, mustSucceed: true, wrapper: true, },);
            const snapshot = ledger.snapshot();
            expect(snapshot.worktree.get('a.txt',),).toEqual({ state: 'absent', },);
            expect([...snapshot.staged,],).toEqual(['b.txt',],);
            expect(snapshot.attempts,).toHaveLength(1,);
            expect(snapshot.auxiliaries,).toHaveLength(1,);
          },
        },),
      ],
    },),
    describe({
      name: workloadDigest.name,
      children: [
        it({
          name: 'ignores completion order and timing facts but not captured bytes',
          fn: async () => {
            const first = attempt({ label: 'w0', bytes: 'a', exitCode: 0, },);
            const second = attempt({ label: 'w1', bytes: 'b', exitCode: 2, },);
            expect(workloadDigest([first, second,],),).toBe(workloadDigest([second, first,],),);
            expect(workloadDigest([first, second,],),)
              .toBe(workloadDigest([attempt({ label: 'w0', bytes: 'a', exitCode: 1, },), second,],),);
            expect(workloadDigest([first, second,],),)
              .not.toBe(workloadDigest([attempt({ label: 'w0', bytes: 'changed', exitCode: 0, },), second,],),);
          },
        },),
      ],
    },),
  ],
},);
