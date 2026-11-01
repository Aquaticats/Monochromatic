/**
 The quit session: Ctrl+Q in a session of its own, since the app ends.
 
 @example
 ```ts
 import { runQuitSession, } from './quit-session.ts';
 
 await runQuitSession(context,);
 ```
 
 @packageDocumentation
 */

import { pathExists, } from './files.ts';
import {
  type RunContext,
  startContextSession,
} from './context.ts';
import { expect, } from './errors.ts';
import { WIDE_WINDOW, } from './layout.ts';
import { pollUntil, } from './poll.ts';
import { SESSION_READY_MS, } from './timing.ts';
import type { UiState, } from './ui-state.ts';

/**
 How many times the app process is looked for after Ctrl+Q.
 */
const EXIT_LOOKS = 40;

/**
 Pause between two looks for the app process.
 */
const EXIT_LOOK_PAUSE_MS = 200;

/**
 Condition: a folder is shown, so the player page is up.
 
 @param state - a state of the app
 
 @returns whether a folder shows
 
 @example
 ```ts
 hasFolder(state,);
 ```
 */
function hasFolder(state: UiState,): boolean {
  return state.folder !== '';
}

/**
 Runs the quit session from start to the end of the app.
 
 @param context - the run's context
 
 @throws CheckFailureError when the session cannot start or never shows the player page
 
 @example
 ```ts
 await runQuitSession(context,);
 ```
 */
export async function runQuitSession(context: RunContext,): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;

  /**
   The running quit session; leaving the function stops it.
   */
  await using session = await startContextSession({
    context,
    options: {
      ...WIDE_WINDOW,
      scheme: 'dark',
      accent: '3B82F6',
      library: context.library,
      label: 'quit',
    },
  },);
  await session.waitState({
    predicate: hasFolder,
    what: 'the player page',
    timeoutMs: SESSION_READY_MS,
  },);
  recorder.setGroup('quit',);
  await recorder.check({
    id: 'quit/ctrl-q',
    title: 'Ctrl+Q quits the app',
    body: async function ctrlQQuitsApp(): Promise<string> {
      /**
       Process id of the app, found by its own unit.
       */
      const pid = await session.findAppPid();
      expect({
        condition: pid !== '',
        message: 'the app process was not found',
      },);
      await session.chord({
        mods: ['ctrl',],
        key: 'q',
      },);

      /**
       Whether the process is gone.
       */
      const gone = await pollUntil({
        attempts: EXIT_LOOKS,
        intervalMs: EXIT_LOOK_PAUSE_MS,
        order: 'wait-then-check',
        check: async function processIsGone(): Promise<boolean> {
          return !(await pathExists(`/proc/${pid}`,));
        },
      },);
      expect({
        condition: gone,
        message: `the app process ${pid} is still running after Ctrl+Q`,
      },);
      return `process ${pid} ended`;
    },
  },);
}
