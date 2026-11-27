import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type GuestRunSteps,
  runWholeGuestRun,
} from './mvm-guest-whole-run.ts';

/**
 The guest one run of these tests names, and the log directory it keeps.
 */
const chosen = {
  name: 'guest-a',
  logDirectory: '/logs',
};

/**
 One run's four steps, as the test that watches them reads them back.
 */
type RecordedRun = {
  /**
   The steps that ran, in the order they ran.
   */
  readonly calls: readonly string[];
  /**
   The steps themselves, which the run is given.
   */
  readonly steps: GuestRunSteps;
};

/**
 One run's four steps, each noting that it ran, and each refusing when the table names it.

 @param refusals - the step names that refuse, each with the error to refuse with

 @returns the steps, and the list the ones that ran are noted in

 @example
 ```ts
 const run = recordedSteps({ refusals: { work: new Error('no') } });
 ```
 */
function recordedSteps({
  refusals,
}: {
  readonly refusals: Readonly<Record<string, Error>>;
},): RecordedRun {
  /**
   The steps that ran, in the order they ran.
   */
  const calls: string[] = [];
  /**
   The step one name performs.
   */
  function stepOf({ name, }: { readonly name: string; },): (run: { readonly chosen: typeof chosen; }) => Promise<void> {
    return async function recordedStep() {
      calls.push(name,);
      /**
       The error this step refuses with, or nothing when it passes.
       */
      const refusal = refusals[name];
      if (refusal !== undefined)
        throw refusal;
    };
  }
  return {
    calls,
    steps: {
      create: stepOf({ name: 'create', },),
      prepare: stepOf({ name: 'prepare', },),
      work: stepOf({ name: 'work', },),
      destroy: stepOf({ name: 'destroy', },),
    },
  };
}

/**
 The error one of these tests reads back out of a run that refused.

 @param work - the work the run was asked to perform

 @returns the error the run refused with

 @example
 ```ts
 const refusal = await refusalOf({ work: async function refuse() { throw new Error('no'); } });
 ```
 */
async function refusalOfRun({ steps, }: { readonly steps: GuestRunSteps; },): Promise<unknown> {
  try {
    await runWholeGuestRun({
      label: 'git-policy test guest',
      chosen,
      steps,
    },);
    return undefined;
  }
  catch (caught) {
    return caught;
  }
}

await describe({
  name: runWholeGuestRun.name,
  children: [
    it({
      name: 'performs the four steps in the order a run owes them',
      fn: async () => {
        /**
         The run these steps are handed to.
         */
        const run = recordedSteps({ refusals: {}, },);
        /**
         What the run refused with, which is nothing when every step passed.
         */
        const refusal = await refusalOfRun({ steps: run.steps, },);
        expect(refusal,).toBeUndefined();
        expect(run.calls,).toEqual([
          'create',
          'prepare',
          'work',
          'destroy',
        ],);
      },
    },),
    it({
      name: 'destroys the guest when the work refused, and reports the work s own refusal',
      fn: async () => {
        /**
         What the work refuses with.
         */
        const refused = new Error('the arm refused',);
        /**
         The run these steps are handed to.
         */
        const run = recordedSteps({ refusals: { work: refused, }, },);
        /**
         What the run refused with.
         */
        const refusal = await refusalOfRun({ steps: run.steps, },);
        expect(refusal,).toBe(refused,);
        expect(run.calls,).toEqual([
          'create',
          'prepare',
          'work',
          'destroy',
        ],);
      },
    },),
    it({
      name: 'performs no step after the one that refused, but still destroys the guest',
      fn: async () => {
        /**
         What the preparation refuses with.
         */
        const refused = new Error('the preparation refused',);
        /**
         The run these steps are handed to.
         */
        const run = recordedSteps({ refusals: { prepare: refused, }, },);
        /**
         What the run refused with.
         */
        const refusal = await refusalOfRun({ steps: run.steps, },);
        expect(refusal,).toBe(refused,);
        expect(run.calls,).toEqual([
          'create',
          'prepare',
          'destroy',
        ],);
      },
    },),
    it({
      name: 'destroys nothing when the guest was never made, and reports that step s refusal',
      fn: async () => {
        /**
         What the creation refuses with.
         */
        const refused = new Error('the name is already listed',);
        /**
         The run these steps are handed to.
         */
        const run = recordedSteps({ refusals: { create: refused, }, },);
        /**
         What the run refused with.
         */
        const refusal = await refusalOfRun({ steps: run.steps, },);
        expect(refusal,).toBe(refused,);
        expect(run.calls,).toEqual(['create',],);
      },
    },),
    it({
      name: 'reports a destruction that refused when nothing else did',
      fn: async () => {
        /**
         What the destruction refuses with.
         */
        const refused = new Error('the guest would not go',);
        /**
         The run these steps are handed to.
         */
        const run = recordedSteps({ refusals: { destroy: refused, }, },);
        /**
         What the run refused with.
         */
        const refusal = await refusalOfRun({ steps: run.steps, },);
        expect(refusal,).toBe(refused,);
      },
    },),
    it({
      name: 'reports the work s refusal when the destruction refused as well',
      fn: async () => {
        /**
         What the work refuses with.
         */
        const workRefused = new Error('the arm refused',);
        /**
         The run these steps are handed to, which refuses in two of them.
         */
        const run = recordedSteps({
          refusals: {
            work: workRefused,
            destroy: new Error('the guest would not go',),
          },
        },);
        /**
         What the run refused with.
         */
        const refusal = await refusalOfRun({ steps: run.steps, },);
        expect(refusal,).toBe(workRefused,);
      },
    },),
  ],
},);
