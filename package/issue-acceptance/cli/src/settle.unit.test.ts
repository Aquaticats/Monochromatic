/**
 Tests for the settlement run, driven by a fake branch reader and a fake tracker.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ChildProcessError,
  IssueAcceptanceError,
  settlePlan,
  type GitReader,
  type IssueEntry,
  type IssueTracker,
  type Plan,
} from '../dist/final/node/index.mjs';

/**
 Returns the error an asynchronous call rejects with.
 
 @param action - Call expected to refuse.
 
 @returns Whatever the call threw.
 
 @throws {@link Error} when the call resolves, which means it refused nothing.
 
 @example
 ```ts
 const refused = await rejectionOf(() => settlePlan({ ...request }));
 ```
 */
async function rejectionOf(action: () => Promise<unknown>,): Promise<unknown> {
  try {
    await action();
  }
  catch (error) {
    return error;
  }
  throw new Error('The call resolved where the test expects a refusal.',);
}

/**
 Branch reader that answers one count for every check.
 
 @param count - Matches every count check finds.
 
 @param tracked - Paths the branch tracks.
 
 @returns A branch reader for the test.
 
 @example
 ```ts
 const reader = readerFor({ count: 0 });
 ```
 */
function readerFor({
  count = 0,
  tracked = ['README.md',],
  present = true,
}: {
  readonly count?: number;
  readonly tracked?: readonly string[];
  readonly present?: boolean;
},): GitReader {
  return {
    countMatches: async function countMatches() {
      return count;
    },
    fileText: async function fileText() {
      return { kind: 'absent', };
    },
    pathExists: async function pathExists() {
      return present;
    },
    trackedPaths: async function trackedPaths() {
      return tracked;
    },
    isOnBranch: async function isOnBranch() {
      return true;
    },
  };
}

/**
 Tracker that records every call it receives.
 
 @param state - State the fake repository reports for every issue.
 
 @param failClose - Whether closing throws, as a refused GitHub call would.
 
 @returns The tracker and the calls it recorded.
 
 @example
 ```ts
 const { tracker, calls } = trackerFor({ state: 'OPEN' });
 ```
 */
function trackerFor({
  state = 'OPEN',
  failClose = false,
}: {
  readonly state?: string;
  readonly failClose?: boolean;
},): { readonly tracker: IssueTracker; readonly calls: string[] } {
  /**
   Calls the tracker received, in order.
   */
  const calls: string[] = [];
  return {
    calls,
    tracker: {
      state: async function stateOf() {
        calls.push('state',);
        return state;
      },
      close: async function close({ number, },) {
        calls.push(`close ${String(number,)}`,);
        if (failClose) {
          throw new ChildProcessError({
            message: 'gh issue close failed',
            stderr: 'refused',
            exitCode: 1,
          },);
        }
      },
      comment: async function comment({ number, },) {
        calls.push(`comment ${String(number,)}`,);
      },
      commentOnCommit: async function commentOnCommit({ commit, },) {
        calls.push(`commit ${commit}`,);
        return '99';
      },
      read: async function read() {
        throw new ChildProcessError({
          message: 'the settle run never reads an issue whole',
          stderr: '',
        },);
      },
      listOpenWithLabels: async function listOpenWithLabels() {
        return [];
      },
    },
  };
}

/**
 One issue entry that closes when the count of `README.md` is zero.
 
 @param issue - Issue number.
 
 @param action - What the plan does once the check holds.
 
 @returns The entry.
 
 @example
 ```ts
 const entry = entryFor({ issue: 7, action: 'close' });
 ```
 */
function entryFor({
  issue,
  action = 'close',
}: {
  readonly issue: number;
  readonly action?: 'close' | 'comment';
},): IssueEntry {
  return {
    issue,
    action,
    checks: [{ kind: 'exists', path: 'README.md', },],
    comment: `Evidence for ${String(issue,)}.`,
  };
}

/**
 A plan holding one repository, one checkout, and the given entries.
 
 @param issues - Entries the run settles.
 
 @returns The plan.
 
 @example
 ```ts
 const plan = planFor({ issues: [entryFor({ issue: 7 })] });
 ```
 */
function planFor({
  issues,
  commitCorrections = [],
}: {
  readonly issues: readonly IssueEntry[];
  readonly commitCorrections?: Plan['commitCorrections'];
},): Plan {
  return {
    repository: 'OWNER/NAME',
    checkout: '/repo',
    branch: 'main',
    issues,
    commitCorrections,
  };
}

await describe({
  name: settlePlan.name,
  children: [
    it({
      name: 'prints each verdict and writes nothing without apply',
      fn: async () => {
        /**
         Lines the run printed.
         */
        const lines: string[] = [];
        /**
         Tracker and the calls it received.
         */
        const { tracker, calls } = trackerFor({},);
        /**
         What the run decided.
         */
        const report = await settlePlan({
          plan: planFor({ issues: [entryFor({ issue: 7, },),], },),
          reader: readerFor({},),
          tracker,
          apply: false,
          write: function collect(line,) {
            lines.push(line,);
          },
        },);
        expect(lines,).toContain('main: 1 tracked paths in /repo',);
        expect(lines,).toContain('#7 holds on main: would close',);
        expect(calls,).toEqual([],);
        expect(report.issues[0]?.kind,).toBe('would-settle',);
        expect(report.failed,).toBe(false,);
      },
    },),
    it({
      name: 'closes an issue whose checks hold, after reading its state',
      fn: async () => {
        /**
         Lines the run printed.
         */
        const lines: string[] = [];
        /**
         Tracker and the calls it received.
         */
        const { tracker, calls } = trackerFor({},);
        /**
         What the run did.
         */
        const report = await settlePlan({
          plan: planFor({ issues: [entryFor({ issue: 7, },),], },),
          reader: readerFor({},),
          tracker,
          apply: true,
          write: function collect(line,) {
            lines.push(line,);
          },
        },);
        expect(calls,).toEqual(['state', 'close 7',],);
        expect(lines,).toContain('#7 closed',);
        expect(report.issues[0]?.kind,).toBe('settled',);
      },
    },),
    it({
      name: 'comments and leaves open an issue the plan only annotates',
      fn: async () => {
        /**
         Tracker and the calls it received.
         */
        const { tracker, calls } = trackerFor({},);
        /**
         What the run did.
         */
        const report = await settlePlan({
          plan: planFor({
            issues: [entryFor({
              issue: 7,
              action: 'comment',
            },),],
          },),
          reader: readerFor({},),
          tracker,
          apply: true,
          write: function ignore() {},
        },);
        expect(calls,).toEqual(['state', 'comment 7',],);
        expect(report.issues[0]?.kind,).toBe('left-open',);
      },
    },),
    it({
      name: 'leaves an already closed issue alone',
      fn: async () => {
        /**
         Tracker and the calls it received.
         */
        const { tracker, calls } = trackerFor({ state: 'CLOSED', },);
        /**
         Lines the run printed.
         */
        const lines: string[] = [];
        /**
         What the run did.
         */
        const report = await settlePlan({
          plan: planFor({ issues: [entryFor({ issue: 7, },),], },),
          reader: readerFor({},),
          tracker,
          apply: true,
          write: function collect(line,) {
            lines.push(line,);
          },
        },);
        expect(calls,).toEqual(['state',],);
        expect(lines,).toContain('#7 is already CLOSED, nothing done',);
        expect(report.issues[0]?.kind,).toBe('already-closed',);
      },
    },),
    it({
      name: 'names every check that failed and touches nothing',
      fn: async () => {
        /**
         Tracker and the calls it received.
         */
        const { tracker, calls } = trackerFor({},);
        /**
         Lines the run printed.
         */
        const lines: string[] = [];
        /**
         What the run decided.
         */
        const report = await settlePlan({
          plan: planFor({ issues: [entryFor({ issue: 7, },),], },),
          reader: readerFor({ present: false, },),
          tracker,
          apply: true,
          write: function collect(line,) {
            lines.push(line,);
          },
        },);
        expect(calls,).toEqual([],);
        expect(lines,).toContain('#7 DOES NOT HOLD: exists README.md answered false',);
        expect(report.issues[0]?.kind,).toBe('does-not-hold',);
        expect(report.failed,).toBe(true,);
      },
    },),
    it({
      name: 'continues to the next issue when one close fails',
      fn: async () => {
        /**
         Tracker and the calls it received.
         */
        const { tracker, calls } = trackerFor({ failClose: true, },);
        /**
         Lines the run printed.
         */
        const lines: string[] = [];
        /**
         What the run did.
         */
        const report = await settlePlan({
          plan: planFor({
            issues: [
              entryFor({ issue: 7, },),
              entryFor({ issue: 8, },),
            ],
          },),
          reader: readerFor({},),
          tracker,
          apply: true,
          write: function collect(line,) {
            lines.push(line,);
          },
        },);
        expect(calls,).toEqual(['state', 'close 7', 'state', 'close 8',],);
        expect(lines,).toContain('#7 FAILED: gh issue close failed',);
        expect(report.issues.map(function kindOf(settlement,): string {
          return settlement.kind;
        },),).toEqual(['failed', 'failed',],);
      },
    },),
    it({
      name: 'refuses a branch that tracks no path, which means the checkout or branch is wrong',
      fn: async () => {
        /**
         Value the run threw.
         */
        const refused = await rejectionOf(() => settlePlan({
          plan: planFor({ issues: [], },),
          reader: readerFor({ tracked: [], },),
          tracker: trackerFor({},).tracker,
          apply: false,
          write: function ignore() {},
        },),);
        expect(refused,).toBeInstanceOf(IssueAcceptanceError,);
        expect((refused as Error).message,).toContain('tracks no path',);
      },
    },),
    it({
      name: 'records a commit correction only when asked to apply',
      fn: async () => {
        /**
         Tracker and the calls it received.
         */
        const { tracker, calls } = trackerFor({},);
        /**
         Plan holding one correction.
         */
        const plan = planFor({
          issues: [],
          commitCorrections: [{ commit: '2b309f8f25', body: 'Correction.', },],
        },);
        /**
         What the dry run said.
         */
        const dry = await settlePlan({
          plan,
          reader: readerFor({},),
          tracker,
          apply: false,
          write: function ignore() {},
        },);
        /**
         What the applied run said.
         */
        const applied = await settlePlan({
          plan,
          reader: readerFor({},),
          tracker,
          apply: true,
          write: function ignore() {},
        },);
        expect(dry.corrections[0]?.kind,).toBe('would-comment',);
        expect(applied.corrections[0]?.kind,).toBe('commented',);
        expect(calls,).toEqual(['commit 2b309f8f25',],);
      },
    },),
    it({
      name: 'reports a failed correction without losing the issue settlements',
      fn: async () => {
        /**
         Tracker whose correction call refuses.
         */
        const tracker: IssueTracker = {
          ...trackerFor({},).tracker,
          commentOnCommit: async function commentOnCommit() {
            throw new ChildProcessError({
              message: 'gh api failed',
              stderr: 'refused',
              exitCode: 1,
            },);
          },
        };
        /**
         Lines the run printed.
         */
        const lines: string[] = [];
        /**
         What the run did.
         */
        const report = await settlePlan({
          plan: planFor({
            issues: [entryFor({ issue: 7, },),],
            commitCorrections: [{ commit: '2b309f8f25', body: 'Correction.', },],
          },),
          reader: readerFor({},),
          tracker,
          apply: true,
          write: function collect(line,) {
            lines.push(line,);
          },
        },);
        expect(lines,).toContain('commit 2b309f8f25: FAILED: gh api failed',);
        expect(report.corrections[0]?.kind,).toBe('failed',);
        expect(report.issues[0]?.kind,).toBe('settled',);
        expect(report.failed,).toBe(true,);
      },
    },),
  ],
},);
