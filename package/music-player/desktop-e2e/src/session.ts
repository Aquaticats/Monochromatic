/**
 One app session: a compositor and the app inside a transient systemd user unit, with the operations a check needs.
 
 @example
 ```ts
 import { startSession, } from './session.ts';
 
 await using session = await startSession({ options, base, config, sinkName, runId: process.pid, },);
 ```
 
 @packageDocumentation
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { RunConfig, } from './config.ts';
import {
  type Control,
  createControl,
} from './control.ts';
import { runCommand, } from './command.ts';
import { expect, } from './errors.ts';
import {
  ensureDirectory,
  pathExists,
} from './files.ts';
import { pollUntil, } from './poll.ts';
import {
  cgroupBelongsToUnit,
  hostEnvironment,
  type SessionOptions,
  sessionEnvironment,
  systemdRunArguments,
  unitNameOf,
} from './session-launch.ts';
import {
  SOCKET_POLL,
  STOP_POLL,
} from './timing.ts';
import {
  createUiStateLog,
  type UiStateLog,
} from './ui-state-log.ts';

/**
 Subdirectories of the scratch XDG root.
 */
const XDG_SUBDIRECTORIES: readonly string[] = [
  'config',
  'data',
  'cache',
  'state',
];

/**
 How many trailing characters of the log a socket timeout message shows.
 */
const TAIL_CHARS = 600;

/**
 Unit states in which the unit still runs or is on its way out.
 */
const LIVE_UNIT_STATES: ReadonlySet<string> = new Set([
  'active',
  'activating',
  'deactivating',
]);

/**
 A running session: the log reader, the compositor controls, and the identity of the unit.
 */
export type Session = Control & UiStateLog & {
  readonly [Symbol.asyncDispose]: () => Promise<void>;
  /**
   The app process id found by `findAppPid`, or the empty string.
   */
  readonly appPid: () => string;
  /**
   Looks for the app process in this session's own unit and remembers it.
   */
  readonly findAppPid: () => Promise<string>;
  readonly label: string;
  readonly scheme: 'dark' | 'light';
  /**
   Directory of the screenshots.
   */
  readonly shots: string;
  /**
   Asks the compositor to quit and stops the unit when it lingers.
   */
  readonly stop: () => Promise<void>;
  readonly unit: string;
  /**
   Scratch XDG root of the session.
   */
  readonly xdg: string;
};

/**
 Reads the unit state word of a session unit.
 
 @param unit - unit name without the suffix
 
 @returns the word `systemctl is-active` prints
 
 @example
 ```ts
 await unitState('music-player-e2e-1-wide',); // 'active'
 ```
 */
async function unitState(unit: string,): Promise<string> {
  /**
   Output of `systemctl is-active`.
   */
  const { stdout, } = await runCommand({
    command: 'systemctl',
    args: [
      '--user',
      'is-active',
      `${unit}.service`,
    ],
  },);
  return stdout.trim();
}

/**
 Saves Auto-play on select off in the scratch configuration unless a session file is already there, so a launch whose
 checks click a row sees it only select, and a second launch over the same configuration keeps what the first saved.
 
 @param xdg - scratch XDG root of the session
 
 @example
 ```ts
 await seedAutoPlayOff(xdg,);
 ```
 */
async function seedAutoPlayOff(xdg: string,): Promise<void> {
  /**
   Directory the app reads its settings from under the scratch configuration home.
   */
  const directory = join(
    xdg,
    'config',
    'musicplayer',
  );

  /**
   The session file the app reads.
   */
  const file = join(
    directory,
    'session.json',
  );
  if (await pathExists(file,))
    return;
  await ensureDirectory(directory,);
  await writeFile(
    file,
    '{"auto_play_on_select":false}',
  );
}

/**
 Starts a session: scratch directories, then the compositor with the app inside a transient unit.
 
 @param options - window and appearance
 
 @param base - scratch directory of the run
 
 @param config - paths of the run
 
 @param sinkName - name of the private null sink the app is pointed at
 
 @param runId - process id of the runner, which makes the unit name unique
 
 @returns the running session; disposing it stops the unit
 
 @throws CheckFailureError when `systemd-run` fails or the compositor socket never appears
 
 @example
 ```ts
 await using session = await startSession({ options, base, config, sinkName, runId: process.pid, },);
 ```
 */
export async function startSession(
  {
    options,
    base,
    config,
    sinkName,
    runId,
  }: {
    readonly base: string;
    readonly config: RunConfig;
    readonly options: SessionOptions;
    readonly runId: number;
    readonly sinkName: string;
  },
): Promise<Session> {
  /**
   Tagged logger naming the session start.
   */
  const rl = tagged({
    tag: startSession.name,
    l: logger,
  },);
  /**
   Session label, which names the unit and the files of the session.
   */
  const { label, } = options;

  /**
   Name of the session's unit.
   */
  const unit = unitNameOf({
    runId,
    label,
  },);

  /**
   Control socket of the compositor.
   */
  const socket = join(
    base,
    `${label}.sock`,
  );

  /**
   File the unit writes the app's output to.
   */
  const logPath = join(
    base,
    `${label}.log`,
  );

  /**
   Directory of the screenshots.
   */
  const shots = join(
    config.outputRoot,
    'shots',
  );

  /**
   Scratch XDG root of the session.
   */
  const xdg = join(
    base,
    `${label}-xdg`,
  );

  /**
   Directory of the stubs that come first on the search path.
   */
  const binDir = join(
    base,
    'bin',
  );
  await Promise.all([
    ...XDG_SUBDIRECTORIES.map(function makeXdgDirectory(sub,): Promise<void> {
      return ensureDirectory(join(
        xdg,
        sub,
      ),);
    },),
    ensureDirectory(join(
      base,
      `${label}-home`,
    ),),
    ensureDirectory(binDir,),
    ensureDirectory(shots,),
  ],);
  await writeFile(
    logPath,
    '',
  );
  if (options.keepSettingsDefaults !== true)
    await seedAutoPlayOff(xdg,);

  /**
   Reader of the session log.
   */
  const log = createUiStateLog({ logPath, },);

  /**
   Compositor controls.
   */
  const control = createControl({
    socket,
    shotsDir: shots,
    label,
  },);

  /**
   The result of `systemd-run`.
   */
  const launched = await runCommand({
    command: 'systemd-run',
    args: systemdRunArguments({
      unit,
      logPath,
      environment: sessionEnvironment({
        sinkName,
        xdg,
        binDir,
        host: hostEnvironment(process.getuid?.() ?? 0,),
        ...options.extraEnv === undefined ? {} : { extraEnv: options.extraEnv, },
      },),
      compositorPath: config.compositorPath,
      socket,
      options,
      releaseBinary: config.releaseBinary,
    },),
  },);
  expect({
    condition: launched.status === 0,
    message: `systemd-run failed: ${launched.stderr}`,
  },);
  rl.info(`started unit ${unit}`,);

  /**
   Whether the control socket appeared.
   */
  const appeared = await pollUntil({
    attempts: SOCKET_POLL.attempts,
    intervalMs: SOCKET_POLL.intervalMs,
    order: 'check-then-wait',
    check: function socketExists(): Promise<boolean> {
      return pathExists(socket,);
    },
  },);
  expect({
    condition: appeared,
    message: `the compositor socket never appeared; log: ${await log.tail(TAIL_CHARS,)}`,
  },);

  /**
   The app process id, once found.
   */
  const found = { appPid: '', };

  /**
   Looks for the app process in this session's own unit. Other `music-player` processes, such as the user's own player,
   are not looked at beyond their control group and are never signaled.
   
   @returns the process id, or the empty string
   */
  async function findAppPid(): Promise<string> {
    /**
     Process ids of every process named `music-player`.
     */
    const { stdout, } = await runCommand({
      command: 'pgrep',
      args: [
        '--exact',
        'music-player',
      ],
    },);
    for (const pid of stdout.trim()
      .split('\n',)
      .filter(function isPid(entry,): boolean {
        return entry !== '';
      },))
      try {
        if (cgroupBelongsToUnit({
          cgroup: await readFile(
            `/proc/${pid}/cgroup`,
            'utf8',
          ),
          unit,
        },)) {
          found.appPid = pid;
          return pid;
        }
      }
      catch (error: unknown) {
        rl.debug(`process ${pid} ended before its cgroup was read: ${String(error,)}`,);
      }
    return '';
  }

  /**
   Tells whether the unit left the live states.

   @returns whether the unit is no longer active, activating or deactivating
   */
  async function unitHasSettled(): Promise<boolean> {
    return !LIVE_UNIT_STATES.has(await unitState(unit,),);
  }

  /**
   Asks the compositor to quit, waits for the unit to leave the live states, and stops it when it lingers.
   */
  async function stop(): Promise<void> {
    try {
      await control.send('quit',);
    }
    catch (error: unknown) {
      rl.debug(`the compositor was already gone: ${String(error,)}`,);
    }

    /**
     Whether the unit left the live states by itself.
     */
    const settled = await pollUntil({
      attempts: STOP_POLL.attempts,
      intervalMs: STOP_POLL.intervalMs,
      order: 'check-then-wait',
      check: unitHasSettled,
    },);
    if (!settled) {
      rl.warn(`unit ${unit} did not quit by itself: stopping it`,);
      await runCommand({
        command: 'systemctl',
        args: [
          '--user',
          'stop',
          `${unit}.service`,
        ],
      },);
    }
  }

  return {
    ...control,
    ...log,
    label,
    scheme: options.scheme,
    shots,
    unit,
    xdg,
    findAppPid,
    stop,
    appPid: function appPid(): string {
      return found.appPid;
    },
    [Symbol.asyncDispose]: stop,
  };
}
