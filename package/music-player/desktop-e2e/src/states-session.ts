/**
 The states sessions: the app on small libraries built for one purpose each, to see what the window shows while a
 folder is being read, how loose files and a single top-level folder are named, what Play does with nothing selected,
 where In order stops, and what an empty folder says.

 @example
 ```ts
 import { runStatesSession, } from './states-session.ts';

 await runStatesSession(context,);
 ```

 @packageDocumentation
 */

import {
  type RunContext,
  startContextSession,
} from './context.ts';
import { WIDE_WINDOW, } from './layout.ts';
import { checkMainLibrary, } from './states-main.ts';
import { checkRefresh, } from './states-refresh.ts';
import { SCAN_PAUSE_MS, } from './states-conditions.ts';
import {
  buildEmptyLibrary,
  buildMainLibrary,
  buildRefreshLibrary,
  buildSingleLibrary,
  EMPTY_LIBRARY_NAME,
} from './states-library.ts';
import {
  checkEmptyFolder,
  checkSingleFolder,
} from './states-small.ts';

/**
 Runs the states sessions from start to quit.

 @param context - the run's context

 @throws CheckFailureError when a session cannot start

 @example
 ```ts
 await runStatesSession(context,);
 ```
 */
export async function runStatesSession(context: RunContext,): Promise<void> {
  {
    /**
     The running session on the main library; leaving the block stops it.
     */
    await using session = await startContextSession({
      context,
      options: {
        ...WIDE_WINDOW,
        scheme: 'dark',
        accent: '3B82F6',
        library: await buildMainLibrary(context.base,),
        label: 'states-main',
        extraEnv: { MUSIC_PLAYER_SCAN_PAUSE_MS: String(SCAN_PAUSE_MS,), },
      },
    },);
    await session.findAppPid();
    await checkMainLibrary({
      context,
      session,
    },);
  }
  {
    /**
     The running session on the single-folder library; leaving the block stops it.
     */
    await using session = await startContextSession({
      context,
      options: {
        ...WIDE_WINDOW,
        scheme: 'light',
        accent: '3B82F6',
        library: await buildSingleLibrary(context.base,),
        label: 'states-single',
      },
    },);
    await checkSingleFolder({
      context,
      session,
    },);
  }
  {
    /**
     The running session on the folder without audio; leaving the block stops it.
     */
    await using session = await startContextSession({
      context,
      options: {
        ...WIDE_WINDOW,
        scheme: 'dark',
        accent: '3B82F6',
        library: await buildEmptyLibrary(context.base,),
        label: 'states-empty',
      },
    },);
    await checkEmptyFolder({
      context,
      session,
      name: EMPTY_LIBRARY_NAME,
    },);
  }
  {
    /**
     The library the refresh session watches, built before the app starts so its files are there at launch.
     */
    const refreshLibrary = await buildRefreshLibrary(context.base,);

    /**
     The running session on the refresh library; leaving the block stops it.
     */
    await using session = await startContextSession({
      context,
      options: {
        ...WIDE_WINDOW,
        scheme: 'dark',
        accent: '3B82F6',
        library: refreshLibrary,
        label: 'states-refresh',
      },
    },);
    await checkRefresh({
      context,
      session,
      library: refreshLibrary,
    },);
  }
}
