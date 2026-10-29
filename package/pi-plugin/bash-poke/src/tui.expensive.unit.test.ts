/**
 Fake-terminal verification of the built extension inside a real Pi TUI.

 This suite is the only place the keystroke path is exercised: Pi's TUI submit
 handler, its stdin buffering, and its widget area are what make a background
 `!` command work, and none of them exist in a fake extension API. It drives a
 real Pi process inside `tmux`, which supplies both the pseudo-terminal and
 `capture-pane` for observation.

 It is named expensive so the default unit run skips it, and it also skips
 inside a Pi session unless explicitly requested, because spawning a nested
 agent from within an agent is how recursive token burn starts.

 @module
 */

import { execFile, } from 'node:child_process';
import { randomUUID, } from 'node:crypto';
import { mkdtemp, rm, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { promisify, } from 'node:util';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { constants, } from '../dist/final/node/index.mjs';

//region Constants

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 
 The generic `promisify(execFile)` form trips `strict-void-return`, because
 Node's overload set includes void-returning callbacks.
 See `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ =
  promisify;

/**
 Awaitable process runner, so no case shells out synchronously.
 */
const run = promisifyExecFile(execFile, );

/**
 Package directory holding the built bundle and the local Pi binary.
 */
const packageDir = join(import.meta.dirname, '..', );

/**
 Pi executable this package depends on, resolved from its own bin directory.
 */
const piBinary = join(packageDir, 'node_modules', '.bin', 'pi', );

/**
 Built extension bundle handed to Pi with `-e`.
 */
const builtExtension = join(packageDir, 'dist', 'final', 'node', 'index.mjs', );

/**
 Environment flag that forces this suite to run inside a Pi session.
 */
const TUI_OVERRIDE_FLAG = 'BASH_POKE_TUI_VERIFY';

/**
 Milliseconds Pi is given to reach its editor.
 */
const STARTUP_DEADLINE_MS = 40_000;

/**
 Milliseconds a pane assertion is given to become true.
 */
const PANE_DEADLINE_MS = 20_000;

/**
 Milliseconds between pane polls.
 */
const POLL_INTERVAL_MS = 250;

/**
 Terminal columns, wide enough that a poke card does not wrap mid-assertion.
 */
const SESSION_COLUMNS = 160;

/**
 Terminal rows, tall enough to keep the widget and the editor on screen.
 */
const SESSION_ROWS = 40;

/**
 Milliseconds a cancelled job is given to prove it never pokes.
 */
const SETTLE_MS = 1_000;

/**
 Shared `it` options that skip this suite inside a Pi session.
 
 Pi sets `PI_CODING_AGENT` for every child process, which is the cheapest
 reliable signal that an agent session is what runs this suite, and spawning a
 nested agent from inside one is how recursive token burn starts.
 */
const skipOptions = ((process.env.PI_CODING_AGENT !== undefined)
  && (process.env[TUI_OVERRIDE_FLAG] !== '1'))
  ? {
    skip: `skipped inside a Pi session; set ${TUI_OVERRIDE_FLAG}=1 to run it anyway`,
  }
  : {};

//endregion Constants

//region Types

/**
 Live tmux session running one nested Pi TUI.
 */
type TmuxSession = {
  /**
   Session name every tmux command targets.
   */
  readonly name: string;

  /**
   Kills the session and removes its directories.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

//endregion Types

//region Terminal driving

/**
 Reads the visible pane text.
 
 @param name - tmux session to capture
 
 @returns pane contents as one string
 
 @example
 ```ts
 await capturePane({ name: 'bp-1', },);
 ```
 */
async function capturePane({ name, }: { readonly name: string; }, ): Promise<string> {
  /**
   Captured stdout of the pane snapshot.
   */
  const { stdout, } = await run('tmux', ['capture-pane', '-p', '-t', name, ], );
  return stdout;
}

/**
 Polls the pane until it contains the needed text.
 
 @param name - tmux session to capture
 
 @param needle - text that must appear
 
 @param deadlineMs - milliseconds to keep polling
 
 @returns pane text that satisfied the poll, or the last capture when it never did
 
 @example
 ```ts
 await waitForPaneText({ name: 'bp-1', needle: 'exit 0', deadlineMs: 5000, },);
 ```
 */
async function waitForPaneText(
  {
    name,
    needle,
    deadlineMs,
  }: {
    readonly name: string;
    readonly needle: string;
    readonly deadlineMs: number;
  },
): Promise<string> {
  /**
   Instant after which polling is pointless.
   */
  const deadline = Date.now() + deadlineMs;

  /**
   Most recent capture, returned so a failure message can show the real pane.
   */
  let pane = '';
  /* oxlint-disable eslint/no-await-in-loop -- polling a terminal is inherently sequential: each capture must complete, and its interval elapse, before the next one can show a changed pane. */
  while (Date.now() < deadline) {
    pane = await capturePane({ name, }, );
    if (pane.includes(needle, ))
      return pane;
    await wait(POLL_INTERVAL_MS, );
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return pane;
}

/**
 Types one line into the editor and submits it.
 
 @param name - tmux session to drive
 
 @param text - editor text, sent literally so no key name is interpreted
 
 @example
 ```ts
 await sendLine({ name: 'bp-1', text: '! printf hi', },);
 ```
 */
async function sendLine(
  {
    name,
    text,
  }: {
    readonly name: string;
    readonly text: string;
  },
): Promise<void> {
  await run('tmux', ['send-keys', '-t', name, '-l', text, ], );
  await run('tmux', ['send-keys', '-t', name, 'Enter', ], );
}

/**
 Presses one named key.
 
 @param name - tmux session to drive
 
 @param key - tmux key name, such as `Escape`
 
 @example
 ```ts
 await sendKey({ name: 'bp-1', key: 'Escape', },);
 ```
 */
async function sendKey(
  {
    name,
    key,
  }: {
    readonly name: string;
    readonly key: string;
  },
): Promise<void> {
  await run('tmux', ['send-keys', '-t', name, key, ], );
}

/**
 Reports whether a tmux session still exists.
 
 @param name - session to look for
 
 @returns whether a kill is still needed
 
 @example
 ```ts
 await sessionExists({ name: 'bp-1', },);
 ```
 */
async function sessionExists({ name, }: { readonly name: string; }, ): Promise<boolean> {
  try {
    await run('tmux', ['has-session', '-t', name, ], );
    return true;
  }
  catch (error: unknown) {
    // tmux reports a missing session as an error exit, which is the answer this
    // probe exists to give rather than a fault.
    return !String(error, ).includes('find session');
  }
}

//endregion Terminal driving

//region Session lifecycle

/**
 Starts a nested Pi TUI in tmux against disposable work and agent directories.
 
 The agent directory is empty and Pi is told to load no extensions except this
 one, so the nested session has no provider credentials, no user settings, and
 no other extension to recurse through.
 
 @returns live session, killed on disposal
 
 @example
 ```ts
 await using session = await startPiTui();
 ```
 */
async function startPiTui(): Promise<TmuxSession> {
  /**
   Disposable working directory the nested Pi treats as its project.
   */
  const work = await mkdtemp(join(scratchRoot(), 'bash-poke-tui-work-', ), );

  /**
   Disposable agent directory, empty so no global extension or credential loads.
   */
  const agent = await mkdtemp(join(scratchRoot(), 'bash-poke-tui-agent-', ), );

  /**
   Session name, unique so parallel runs cannot capture each other's panes.
   */
  const name = `bp-${randomUUID().split('-').join('', ).slice(0, 12, )}`;

  /**
   Shell line tmux runs in the pane, keeping the pane alive after Pi exits so a
   failure can still be captured.
   */
  const command = [
    `PI_CODING_AGENT_DIR=${agent}`,
    'PI_OFFLINE=1',
    piBinary,
    '--no-extensions',
    `-e ${builtExtension}`,
    '--no-session',
    '--no-context-files',
    '--no-approve',
    '; echo PI_EXIT=$?',
    '; sleep 300',
  ].join(' ');

  await run('tmux', [
    'new-session',
    '-d',
    '-s',
    name,
    '-x',
    String(SESSION_COLUMNS, ),
    '-y',
    String(SESSION_ROWS, ),
    '-c',
    work,
    command,
  ], );

  /**
   Pane text proving Pi reached its editor before any case sends keystrokes.
   */
  const ready = await waitForPaneText({
    name,
    needle: work,
    deadlineMs: STARTUP_DEADLINE_MS,
  }, );
  if (!ready.includes(work, )) {
    await run('tmux', ['kill-session', '-t', name, ], );
    throw new Error(`Pi never showed its working directory; pane was:\n${ready}`);
  }

  return {
    name,
    async [Symbol.asyncDispose](): Promise<void> {
      if (await sessionExists({ name, }, ))
        await run('tmux', ['kill-session', '-t', name, ], );
      await rm(work, { recursive: true, force: true, }, );
      await rm(agent, { recursive: true, force: true, }, );
    },
  };
}

//endregion Session lifecycle

await describe({
  name: 'Pi TUI through a fake terminal',
  concurrency: 1,
  children: [
    it({
      name: 'hands the editor back with a placeholder and pokes when the command exits',
      ...skipOptions,
      timeout: STARTUP_DEADLINE_MS + PANE_DEADLINE_MS,
      fn: async () => {
        await using session = await startPiTui();

        /**
         Marker unique to this case, so no earlier transcript can satisfy it.
         */
        const marker = `poke${randomUUID().split('-').join('', ).slice(0, 8, )}`;
        await sendLine({ name: session.name, text: `! printf ${marker}`, }, );

        /**
         Pane captured once the poke card has been rendered.
         */
        const pane = await waitForPaneText({
          name: session.name,
          needle: `[bash finished] $ printf ${marker} (exit 0)`,
          deadlineMs: PANE_DEADLINE_MS,
        }, );
        expect(pane.includes(constants.PENDING_NOTE, ), ).toBe(true);
        expect(pane.includes('bash-poke exit 0', ), ).toBe(true);
        expect(pane.includes(marker, ), ).toBe(true);
        // The default instruction is empty, so the card must end at its closing
        // fence instead of trailing a machine-written imperative. Checking the
        // constant here would be vacuous once it is the empty string.
        expect(
          pane.split('\n', )
            .map(function trimLine(line: string, ): string {
              return line.trim();
            }, )
            .includes('continue', ),
        ).toBe(false);
      },
    }, ),
    it({
      name: 'leaves a hidden command to Pi without a poke',
      ...skipOptions,
      timeout: STARTUP_DEADLINE_MS + PANE_DEADLINE_MS,
      fn: async () => {
        await using session = await startPiTui();

        /**
         Marker unique to this case.
         */
        const marker = `hidden${randomUUID().split('-').join('', ).slice(0, 8, )}`;
        await sendLine({ name: session.name, text: `!! printf ${marker}`, }, );

        /**
         Pane captured once Pi's own execution has shown the output.
         */
        const pane = await waitForPaneText({
          name: session.name,
          needle: marker,
          deadlineMs: PANE_DEADLINE_MS,
        }, );
        expect(pane.includes(`$ printf ${marker}`, ), ).toBe(true);
        // A hidden command belongs to Pi, so no poke card may claim it.
        expect(pane.includes(`[bash finished] $ printf ${marker}`, ), ).toBe(false);
        expect(pane.includes(constants.PENDING_NOTE, ), ).toBe(false);
      },
    }, ),
    it({
      name: 'shows a live progress row and cancels the job on a lone escape',
      ...skipOptions,
      timeout: STARTUP_DEADLINE_MS + (PANE_DEADLINE_MS * 2),
      fn: async () => {
        await using session = await startPiTui();
        await sendLine({ name: session.name, text: '! sleep 300', }, );

        /**
         Pane captured once the progress row is above the editor.
         */
        const running = await waitForPaneText({
          name: session.name,
          needle: '! sleep 300 (',
          deadlineMs: PANE_DEADLINE_MS,
        }, );
        expect(running.includes(constants.PENDING_NOTE, ), ).toBe(true);

        await sendKey({ name: session.name, key: 'Escape', }, );

        /**
         Pane captured once the cancellation has been reported.
         */
        const cancelled = await waitForPaneText({
          name: session.name,
          needle: 'Cancelled 1 background command(s)',
          deadlineMs: PANE_DEADLINE_MS,
        }, );
        expect(cancelled.includes('[bash finished] $ sleep 300', ), ).toBe(false);

        await wait(SETTLE_MS, );

        /**
         Pane after the cancelled job had every chance to report.
         */
        const settled = await capturePane({ name: session.name, }, );
        expect(settled.includes('[bash finished] $ sleep 300', ), ).toBe(false);
      },
    }, ),
  ],
}, );
