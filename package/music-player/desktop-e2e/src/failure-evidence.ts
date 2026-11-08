/**
 The file a failing check leaves behind: its text, the latest states of the session it ran in, and the session's own
 files beside it.
 
 A failure file holds what the app said it did, which is `ui_state` lines only. It cannot say what the app was waiting
 for, nor whether its unit never ran because it ran out of its quota or was throttled on the disk. Those answers sit in
 the unit's log and in its control group, so they are copied at the moment a check fails, before the scratch directory
 and the unit go away.

 @example
 ```ts
 import { writeFailureEvidence, } from './failure-evidence.ts';

 const id = 'play/enter-plays-row';
 const path = await writeFailureEvidence({
   detail: 'timed out waiting for now-playing',
   evidenceDir: '/work/app/target/e2e/failures',
   id,
   title: 'Enter plays the row',
   source: session,
 },);
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

import type { UiState, } from './ui-state.ts';

/**
 The fields of an app state that a failure file keeps: where the keyboard and the page were.
 */
export type EvidenceState = Pick<UiState, 'focus' | 'reason' | 'route' | 'seq' | 'unfolded' | 'zone'>;

/**
 A file that was read and is written beside the failure file.
 */
export type KeptEvidenceFile = {
  readonly kind: 'kept';
  /**
   Name added to the failure file's name after a dot, such as `unit.log`.
   */
  readonly name: string;
  /**
   Path its content was read from.
   */
  readonly source: string;
  /**
   What was read there.
   */
  readonly text: string;
};

/**
 A file that has nothing to keep, such as the accounting of a unit that had already stopped.
 */
export type MissingEvidenceFile = {
  readonly kind: 'missing';
  readonly name: string;
  readonly source: string;
  /**
   Why there is no content, written into the failure file.
   */
  readonly reason: string;
};

/**
 One file a failing check keeps beside its failure file.
 */
export type EvidenceFile = KeptEvidenceFile | MissingEvidenceFile;

/**
 The session whose states and files a failing check keeps.
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
  /**
   Reads the files the session keeps beside a failure file: the unit's log and the accounting of its control group.
   They are read while the session still runs, because both are gone once it stops.
   */
  readonly readSessionFiles: () => Promise<readonly EvidenceFile[]>;
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
 Writes the file that keeps a failing check's text and the latest states of the session it ran in, and writes the
 session's own files beside it. A problem while writing is logged and reported in the failure line, because the
 evidence must never hide the failure it describes.

 @param detail - why the check failed

 @param evidenceDir - folder of the failure files; nothing is written when absent

 @param id - check id

 @param source - the session whose states and files are kept with the failure

 @param title - what the check shows

 @returns the file's path, or the empty string when there is no folder to write to or the file could not be written

 @example
 ```ts
 const id = 'play/enter-plays-row';
 const path = await writeFailureEvidence({
   detail: 'timed out waiting for now-playing',
   evidenceDir: '/work/app/target/e2e/failures',
   id,
   title: 'Enter plays the row',
   source: session,
 },);
 ```
 */
export async function writeFailureEvidence(
  {
    detail,
    id,
    title,
    source,
    evidenceDir,
  }: {
    readonly detail: string;
    readonly evidenceDir?: string;
    readonly id: string;
    readonly source?: EvidenceSource;
    readonly title: string;
  },
): Promise<string> {
  if (evidenceDir === undefined)
    return '';

  /**
   Tagged logger naming the writer.
   */
  const rl = tagged({
    tag: writeFailureEvidence.name,
    l: logger,
  },);

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
   Name shared by the failure file and the files kept beside it.
   */
  const base = id.replaceAll(
    '/',
    '--',
  );

  /**
   The files the session handed over, empty when no session was attached or it could not read them.
   */
  const files = source === undefined ? [] : await sessionFiles({
    source,
    id,
  },);

  /**
   Where the file goes.
   */
  const path = join(
    evidenceDir,
    `${base}${EVIDENCE_SUFFIX}`,
  );
  try {
    await mkdir(
      evidenceDir,
      { recursive: true, },
    );
    /**
     One line per file the session keeps: where it landed and where its content came from, or why it was not kept.
     */
    const fileLines = await Promise.all(files.map(async function keepOne(file,): Promise<string> {
      if (file.kind === 'missing')
        return `${base}.${file.name} not kept: ${file.reason} (${file.source})`;
      /**
       Where this file goes, beside the failure file.
       */
      const target = join(
        evidenceDir,
        `${base}.${file.name}`,
      );
      try {
        await writeFile(
          target,
          file.text,
        );
        return `${target} <- ${file.source}`;
      }
      catch (error: unknown) {
        rl.error(`could not write the file ${target} of ${id} beside its failure file: ${caughtValueText(error,)}`,);
        return `${base}.${file.name} not kept: ${caughtValueText(error,)} (${file.source})`;
      }
    },));
    /**
     The lines about the files the session keeps, empty when it handed none over.
     */
    const fileSection = fileLines.length === 0 ? [] : [
      'the session\'s own files, beside this one:',
      ...fileLines,
    ];
    await writeFile(
      path,
      [
        `check: ${id}`,
        `title: ${title}`,
        `failure: ${detail}`,
        ...stateLines,
        ...fileSection,
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
 Reads the files a session keeps beside a failure file, and says so when it cannot.

 @param source - the session

 @param id - check id, for the log line

 @returns the files, or an empty list when the session could not hand them over
 */
async function sessionFiles(
  {
    source,
    id,
  }: {
    readonly id: string;
    readonly source: EvidenceSource;
  },
): Promise<readonly EvidenceFile[]> {
  /**
   Tagged logger naming the read.
   */
  const rl = tagged({
    tag: sessionFiles.name,
    l: logger,
  },);
  try {
    return await source.readSessionFiles();
  }
  catch (error: unknown) {
    rl.warn(`could not read the files of session ${source.label} for ${id}: ${caughtValueText(error,)}`,);
    return [];
  }
}
