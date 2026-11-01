/**
 The wide session: the app in the wide window, dark scheme, blue accent, with every check that needs the two-pane
 layout. The checks run in the order that leaves the app in the state the next one expects.
 
 @example
 ```ts
 import { runWideSession, } from './wide-session.ts';
 
 await runWideSession(context,);
 ```
 
 @packageDocumentation
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { writeFileManagerStub, } from './clipboard.ts';
import {
  type RunContext,
  startContextSession,
} from './context.ts';
import { WIDE_WINDOW, } from './layout.ts';
import { SESSION_READY_MS, } from './timing.ts';
import type { UiState, } from './ui-state.ts';
import { checkBadges, } from './wide-badges.ts';
import { checkBreakpoint, } from './wide-breakpoint.ts';
import { checkClickThrough, } from './wide-interaction.ts';
import {
  checkBrowsing,
  checkFolderList,
} from './wide-browse.ts';
import { checkDoubleShift, } from './wide-double-shift.ts';
import { checkLists, } from './wide-lists.ts';
import { checkMenuPlacement, } from './wide-menu.ts';
import { checkLaunchAndFocus, } from './wide-focus.ts';
import { checkPlayback, } from './wide-playback.ts';
import { checkMenuAndDelete, } from './wide-row-menu.ts';
import { checkRowChords, } from './wide-rows.ts';
import { checkSearchAndTooltip, } from './wide-search.ts';
import { checkSelectOnly, } from './wide-select.ts';
import { checkSeeking, } from './wide-seek.ts';
import { checkSettings, } from './wide-settings.ts';
import { checkTrackList, } from './wide-track-list.ts';

/**
 Condition: the wide layout shows the player page with its first folder.
 
 @param state - a state of the app
 
 @returns whether a folder is shown and the layout is unfolded
 
 @example
 ```ts
 isUnfoldedWithFolder(state,);
 ```
 */
function isUnfoldedWithFolder(state: UiState,): boolean {
  return (state.folder !== '') && state.unfolded;
}

/**
 Runs the wide session from start to quit.
 
 @param context - the run's context
 
 @throws CheckFailureError when the session cannot start or never shows the player page
 
 @throws SafetyStopError when the app plays anywhere but the private sink
 
 @example
 ```ts
 await runWideSession(context,);
 ```
 */
export async function runWideSession(context: RunContext,): Promise<void> {
  /**
   File in which the file manager stub records its arguments.
   */
  const fileManagerRecord = join(
    context.base,
    'xdg-open.record',
  );
  await writeFile(
    fileManagerRecord,
    '',
  );
  await writeFileManagerStub({
    binDir: join(
      context.base,
      'bin',
    ),
    record: fileManagerRecord,
  },);

  /**
   The running wide session; leaving the function stops it.
   */
  await using session = await startContextSession({
    context,
    options: {
      ...WIDE_WINDOW,
      scheme: 'dark',
      accent: '3B82F6',
      library: context.library,
      label: 'wide',
    },
  },);
  await session.waitState({
    predicate: isUnfoldedWithFolder,
    what: 'the unfolded player page with its first folder',
    timeoutMs: SESSION_READY_MS,
  },);
  await session.findAppPid();
  await checkLaunchAndFocus({
    context,
    session,
  },);
  await checkPlayback({
    context,
    session,
  },);
  await checkSeeking({
    context,
    session,
  },);
  await checkBrowsing({
    context,
    session,
  },);
  await checkFolderList({
    context,
    session,
  },);
  await checkTrackList({
    context,
    session,
  },);
  await checkRowChords({
    context,
    session,
    fileManagerRecord,
  },);
  await checkMenuAndDelete({
    context,
    session,
  },);
  await checkMenuPlacement({
    context,
    session,
  },);
  await checkSearchAndTooltip({
    context,
    session,
  },);
  await checkSelectOnly({
    context,
    session,
  },);
  await checkDoubleShift({
    context,
    session,
  },);
  await checkSettings({
    context,
    session,
  },);
  await checkBadges({
    context,
    session,
  },);
  await checkClickThrough({
    context,
    session,
  },);
  await checkLists({
    context,
    session,
  },);
  await checkBreakpoint({
    context,
    session,
  },);
}
