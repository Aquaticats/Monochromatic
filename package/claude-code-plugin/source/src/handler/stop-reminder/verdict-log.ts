/**
 Append-only record of every Luna verdict and the action taken.

 The 0.60 cutoff and the choice to release a background wait while a task is
 running both rest on unlabeled data, so each decision is kept with the inputs
 needed to reconstruct it: label probabilities, the hook state read, and the
 tail of the message. Hand-labelling this file later is what replaces the
 guess. The message tail stays on this machine; nothing here is sent anywhere.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  appendFile,
  mkdir,
} from 'node:fs/promises';
import {
  homedir,
} from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

/**
 Environment variable overriding the log path, for tests and disposable runs.
 */
const VERDICT_LOG_ENV = 'MONOCHROMATIC_STOP_VERDICT_LOG' as const;

/**
 Characters of the message tail kept per record.
 */
const MESSAGE_TAIL_CHARS = 600;

/**
 Module logger for verdict recording.
 */
const logger = tagged({ tag: 'stop-reminder-verdict-log', },);

/**
 One verdict record.
 */
type VerdictRecord = {
  readonly sessionId: string;
  readonly outcome: 'luna' | 'fallback' | 'skipped';
  readonly detail: string;
  readonly action?: 'block' | 'release';
  readonly situation?: string;
  readonly probabilities?: Readonly<Record<string, number>>;
  readonly backgroundTasks: readonly {
    readonly type: string;
    readonly status: string;
  }[];
  readonly depth: number;
  readonly message: string;
  readonly permissionMode: string;
  readonly sessionCrons: number;

  /**
   Whether the hook had already blocked this turn, which makes the stop a repeated one.

   Absent from records written before 2026-10-25.
   */
  readonly stopHookActive: boolean;
};

/**
 Resolves the log path.

 @param env - process environment

 @returns override path, else a file under the user's state directory

 @example
 ```ts
 verdictLogPath({}); // '<home>/.local/state/monochromatic/stop-reminder/verdicts.jsonl'
 ```
 */
function verdictLogPath(env: Readonly<NodeJS.ProcessEnv>,): string {
  return env[VERDICT_LOG_ENV]
    ?? join(
      homedir(),
      '.local',
      'state',
      'monochromatic',
      'stop-reminder',
      'verdicts.jsonl',
    );
}

/**
 Appends one record, never failing the hook.

 A hook that throws emits no stdout, which Claude Code reads as permission to
 stop, so a full disk must not turn into a silent release of every other check.

 @param path - log file path

 @param record - verdict to append

 @example
 ```ts
 await appendVerdict({ path, record });
 ```
 */
async function appendVerdict(
  {
    path,
    record,
  }: {
    readonly path: string;
    readonly record: VerdictRecord;
  },
): Promise<void> {
  /**
   Function-scoped logger so every line names this call.
   */
  const l = tagged({
    tag: appendVerdict.name,
    l: logger,
  },);
  try {
    await mkdir(
      dirname(path,),
      { recursive: true, },
    );
    await appendFile(
      path,
      `${JSON.stringify({
        at: new Date().toISOString(),
        ...record,
        message: record.message
          .slice(-MESSAGE_TAIL_CHARS,),
      },)}\n`,
    );
  }
  catch (error) {
    l.warn(`could not append the verdict: ${String(error,)}`,);
  }
}

export type { VerdictRecord, };

export {
  appendVerdict,
  MESSAGE_TAIL_CHARS,
  VERDICT_LOG_ENV,
  verdictLogPath,
};
