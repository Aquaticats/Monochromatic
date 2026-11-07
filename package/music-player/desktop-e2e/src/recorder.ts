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

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

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
import type { UiState, } from './ui-state.ts';

/**
 The fields of an app state that a failure file keeps: where the keyboard and the page were.
 */
export type EvidenceState = Pick<UiState, 'focus' | 'reason' | 'route' | 'seq' | 'unfolded' | 'zone'>;

/**
 Writes a state as one line of a failure file.

 @param state - the state

 @returns the sequence number and the fields that say where the keyboard and the page were

 @example
 ```ts
 describeState({ seq: 1, reason: 'key', zone: 'surface', focus: '', route: 'Player', unfolded: true, },);
 ```
 */
function describeState(state: EvidenceState,): string {
  return `seq=${state.seq} reason=${state.reason} zone=${state.zone} focus=${state.focus} `
    + `route=${state.route} unfolded=${state.unfolded}`;
}

/**
 The session whose states a failing check keeps.
 */
export type EvidenceSource = {
  /**
   Name of the session, such as `wide`.
   */
  readonly label: string;
  /**
   Reads the log up to now and gives every state read so far, oldest first.
   */
  readonly readStates: () => Promise<readonly EvidenceState[]>;
};

/**
 How many of the latest states a failure file keeps. Ten covers the burst of key presses and answers before a
 timeout (the Search answer race on 2026-11-07 needed the last three) and stays readable in a terminal.
 */
export const EVIDENCE_STATES = 10;

/**
 The reason of the state lines a playing track writes on every position change. They carry nothing a failure needs and a
 playing track writes them without pause, so a file of the last states would hold only them.
 */
const POSITION_REASON = 'position';

/**
 Suffix of a failure file's name.
 */
const EVIDENCE_SUFFIX = '.failure.txt';

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
   Names the session whose states the next failing checks keep; the latest call wins.
   */
  readonly attach: (source: EvidenceSource,) => void;
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
 
 @param evidenceDir - folder that receives one file per failing check, with its text and the latest states; none
 written when absent
 
 @returns the recorder
 
 @example
 ```ts
 const recorder = createRecorder({ only: 'seek', },);
 ```
 */
export function createRecorder(
  {
    only,
    evidenceDir,
  }: {
    readonly evidenceDir?: string;
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
   Every session attached so far; the last one is the session a failing check keeps the states of.
   */
  const attached: EvidenceSource[] = [];

  /**
   Writes the file that keeps a failing check's text and the latest states of the attached session. A problem while
   writing is logged and reported in the failure line, because the evidence must never hide the failure it describes.

   @param id - check id

   @param title - what the check shows

   @param detail - why the check failed

   @returns the file's path, or the empty string when the recorder has no folder or the file could not be written
   */
  async function keepEvidence(
    {
      id,
      title,
      detail,
    }: {
      readonly detail: string;
      readonly id: string;
      readonly title: string;
    },
  ): Promise<string> {
    if (evidenceDir === undefined)
      return '';

    /**
     The session to read, if one is attached.
     */
    const source = attached.at(-1,);

    /**
     The lines about the session's states.
     */
    const stateLines: string[] = [];
    if (source === undefined)
      stateLines.push('no session was attached when the check failed',);
    else
      try {
        /**
         Every state the session has read.
         */
        const states = await source.readStates();

        /**
         The states other than position ticks.
         */
        const changes = states.filter(function isChange(state,): boolean {
          return state.reason !== POSITION_REASON;
        },);

        /**
         The latest of them.
         */
        const kept = changes.slice(-EVIDENCE_STATES,);

        /**
         The state the session was in last, whatever its reason.
         */
        const final = states.at(-1,);
        stateLines.push(
          `session: ${source.label}`,
          `last ${kept.length} of ${changes.length} ui_state changes, oldest first `
            + `(${states.length - changes.length} position ticks skipped):`,
          ...kept.map(describeState,),
          ...final === undefined ? ['no ui_state line was read',] : [`the final state, ${describeState(final,)}`,],
        );
      }
      catch (error: unknown) {
        rl.warn(`could not read the states of session ${source.label} for ${id}: ${caughtValueText(error,)}`,);
        stateLines.push(
          `session: ${source.label}`,
          `the states could not be read: ${caughtValueText(error,)}`,
        );
      }

    /**
     Where the file goes.
     */
    const path = join(
      evidenceDir,
      `${
        id.replaceAll(
          '/',
          '--',
        )
      }${EVIDENCE_SUFFIX}`,
    );
    try {
      await mkdir(
        evidenceDir,
        { recursive: true, },
      );
      await writeFile(
        path,
        [
          `check: ${id}`,
          `title: ${title}`,
          `failure: ${detail}`,
          ...stateLines,
          '',
        ].join('\n',),
      );
      return path;
    }
    catch (error: unknown) {
      rl.error(`could not write the failure file ${path}: ${caughtValueText(error,)}`,);
      return '';
    }
  }

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

      /**
       Where the failure text and the latest states were kept.
       */
      const evidence = await keepEvidence({
        id,
        title,
        detail,
      },);
      recorded.push({
        id,
        title,
        ok: false,
        skipped: false,
        detail,
        ...evidence === '' ? {} : { evidence, },
        ms: Date.now() - started,
      },);
      printLine(`FAIL ${id} ${title} :: ${detail}${evidence === '' ? '' : ` (evidence: ${evidence})`}`,);
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
    attach: function attach(source,): void {
      attached.push(source,);
    },
    outcomes: function outcomes(): readonly Outcome[] {
      return recorded;
    },
    setGroup: function setGroup(group,): void {
      current.group = group;
    },
  };
}
