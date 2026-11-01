/**
 The recorder of a run: it runs each check, keeps its outcome, and prints its result line.
 
 A failing check never stops the run; only a safety stop does.
 
 @example
 ```ts
 import { createRecorder, } from './recorder.ts';
 
 const recorder = createRecorder({ only: '', },);
 await recorder.verify({ id: 'a/b', title: 'a thing holds', body: holds, },);
 ```
 
 @packageDocumentation
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { SafetyStopError, } from './errors.ts';
import {
  type Outcome,
  printLine,
} from './report.ts';
import { isSelected, } from './selection.ts';

/**
 What a check gives back to the recorder: the text that follows its PASS line.
 */
export type CheckBody = () => Promise<string>;

/**
 The recorder's operations.
 */
export type Recorder = {
  /**
   Runs a check whose pass line carries a detail text.
   */
  readonly check: (
    check: {
      readonly body: CheckBody;
      readonly id: string;
      readonly title: string
    },
  ) => Promise<void>;
  /**
   Lists the recorded outcomes in the order they were recorded.
   */
  readonly outcomes: () => readonly Outcome[];
  /**
   Records a check that cannot run here, with the evidence.
   */
  readonly skip: (skipped: {
    readonly id: string;
    readonly reason: string;
    readonly title: string
  },) => void;
  /**
   Names the group the following checks belong to.
   */
  readonly setGroup: (group: string,) => void;
  /**
   Runs a check that has no detail text.
   */
  readonly verify: (
    check: {
      readonly body: () => Promise<void>;
      readonly id: string;
      readonly title: string
    },
  ) => Promise<void>;
};

/**
 Creates a recorder that applies the run's filter.
 
 @param only - the filter text; empty runs everything
 
 @returns the recorder
 
 @example
 ```ts
 const recorder = createRecorder({ only: 'seek', },);
 ```
 */
export function createRecorder(
  {
    only,
  }: {
    readonly only: string;
  },
): Recorder {
  /**
   Tagged logger naming the recorder.
   */
  const rl = tagged({
    tag: createRecorder.name,
    l: logger,
  },);

  /**
   Outcomes in the order they were recorded.
   */
  const recorded: Outcome[] = [];

  /**
   The group the next checks belong to, kept in a one-slot holder.
   */
  const current = { group: '', };

  /**
   Tells whether a check passes the run's filter.
   
   @param id - check id
   
   @returns whether the check runs
   */
  function selects(id: string,): boolean {
    return isSelected({
      only,
      id,
      group: current.group,
    },);
  }

  /**
   Runs one check and records its outcome, whichever way it ends.
   
   @param id - check id
   
   @param title - what the check shows
   
   @param body - the check; gives back the detail text of the pass line
   
   @throws SafetyStopError when the check found the app playing outside the private sink
   */
  async function check(
    {
      id,
      title,
      body,
    }: {
      readonly body: CheckBody;
      readonly id: string;
      readonly title: string;
    },
  ): Promise<void> {
    if (!selects(id,)) {
      rl.debug(`${id} is not selected by the filter "${only}"`,);
      return;
    }

    /**
     Start of the check, in milliseconds since the epoch.
     */
    const started = Date.now();
    try {
      /**
       Detail text of the pass line.
       */
      const detail = await body();
      recorded.push({
        id,
        title,
        ok: true,
        skipped: false,
        detail,
        ms: Date.now() - started,
      },);
      printLine(`PASS ${id} ${title}${detail === '' ? '' : ` :: ${detail}`}`,);
    }
    catch (error: unknown) {
      if (error instanceof SafetyStopError)
        throw error;

      /**
       Why the check failed.
       */
      const detail = caughtValueText(error,);
      recorded.push({
        id,
        title,
        ok: false,
        skipped: false,
        detail,
        ms: Date.now() - started,
      },);
      printLine(`FAIL ${id} ${title} :: ${detail}`,);
    }
  }

  /**
   Runs a check that has nothing to report on a pass.
   
   @param id - check id
   
   @param title - what the check shows
   
   @param body - the check
   */
  async function verify(
    {
      id,
      title,
      body,
    }: {
      readonly body: () => Promise<void>;
      readonly id: string;
      readonly title: string;
    },
  ): Promise<void> {
    await check({
      id,
      title,
      body: async function bodyWithoutDetail(): Promise<string> {
        await body();
        return '';
      },
    },);
  }

  /**
   Records a check that cannot run in this environment; it counts neither as a pass nor as a failure.
   
   @param id - check id
   
   @param title - what the check would show
   
   @param reason - the evidence that it cannot run
   */
  function skip(
    {
      id,
      title,
      reason,
    }: {
      readonly id: string;
      readonly reason: string;
      readonly title: string;
    },
  ): void {
    if (!selects(id,))
      return;
    recorded.push({
      id,
      title,
      ok: true,
      skipped: true,
      detail: reason,
      ms: 0,
    },);
    printLine(`SKIP ${id} ${title} :: ${reason}`,);
  }

  return {
    check,
    verify,
    skip,
    outcomes: function outcomes(): readonly Outcome[] {
      return recorded;
    },
    setGroup: function setGroup(group,): void {
      current.group = group;
    },
  };
}
