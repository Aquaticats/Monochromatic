/**
 Where the runner finds the app, the compositor and its scratch space, from the environment and the app directory.
 
 Environment variables: `E2E_ONLY` (a check id or group prefix to run alone), `E2E_OUT` (screenshots and summary,
 default `target/e2e` under the app directory), `E2E_TMP` (scratch root, default `temp/agent/music-player-e2e` under the
 home directory), `E2E_LIBRARY_GENERATOR` (the library generator script), `E2E_COMPOSITOR` (the compositor executable,
 default the one mise has installed, found by `compositor-path.ts`), and `E2E_UNIT_MEMORY_MAX`, `E2E_UNIT_CPU_QUOTA` and
 `E2E_UNIT_TASKS_MAX` (the systemd limits of each session unit, defaulting to the measured values in `session-launch.ts`),
 `E2E_VISIBLE` (exactly `1` asks for a nested window on the desktop; anything else, or nothing, keeps the run headless)
 and `E2E_GPU` (the render device of a headless run, `software` or a render node path, replacing the probe).
 
 @example
 ```ts
 import { loadRunConfig, overridesFromEnvironment, } from './config.ts';
 
 const config = loadRunConfig({
   appDir: process.cwd(),
   home: homedir(),
   overrides: overridesFromEnvironment(),
   defaultCompositorPath: '/opt/nested/bin/nested',
 },);
 ```
 
 @packageDocumentation
 */

import { join, } from 'node:path';

import {
  DEFAULT_UNIT_LIMITS,
  type UnitLimits,
} from './session-launch.ts';

/**
 The only value of `E2E_VISIBLE` that asks for a nested window.
 */
const VISIBLE_ENABLED = '1';

/**
 Everything the run needs to know about its surroundings.
 */
export type RunConfig = {
  readonly appDir: string;
  readonly compositorPath: string;
  readonly generatorPath: string;
  /**
   The render device a person named for a headless run, or the empty string when the run probes for one.
   */
  readonly gpu: string;
  readonly only: string;
  readonly outputRoot: string;
  readonly releaseBinary: string;
  readonly scratchRoot: string;
  readonly unitLimits: UnitLimits;
  /**
   True only when a person asked for a nested window on the desktop; every other run is headless.
   */
  readonly visible: boolean;
};

/**
 Values that replace the defaults, each present only when the user set it.
 */
export type RunOverrides = {
  readonly compositorPath?: string;
  readonly generatorPath?: string;
  readonly gpu?: string;
  readonly only?: string;
  readonly outputRoot?: string;
  readonly scratchRoot?: string;
  readonly unitCpuQuota?: string;
  readonly unitMemoryMax?: string;
  readonly unitTasksMax?: string;
  readonly visible?: boolean;
};

/**
 Reads the overrides from the environment variables `E2E_ONLY`, `E2E_OUT`, `E2E_TMP`, `E2E_LIBRARY_GENERATOR`,
 `E2E_COMPOSITOR`, `E2E_UNIT_MEMORY_MAX`, `E2E_UNIT_CPU_QUOTA`, `E2E_UNIT_TASKS_MAX`, `E2E_VISIBLE` and `E2E_GPU`.
 
 @returns the overrides whose variables are set
 
 @example
 ```ts
 const overrides = overridesFromEnvironment();
 ```
 */
export function overridesFromEnvironment(): RunOverrides {
  /**
   Values of the variables, each possibly unset.
   */
  const {
    E2E_COMPOSITOR,
    E2E_GPU,
    E2E_LIBRARY_GENERATOR,
    E2E_ONLY,
    E2E_OUT,
    E2E_TMP,
    E2E_UNIT_CPU_QUOTA,
    E2E_UNIT_MEMORY_MAX,
    E2E_UNIT_TASKS_MAX,
    E2E_VISIBLE,
  } = process.env;
  return {
    ...E2E_COMPOSITOR === undefined ? {} : { compositorPath: E2E_COMPOSITOR, },
    ...E2E_LIBRARY_GENERATOR === undefined ? {} : { generatorPath: E2E_LIBRARY_GENERATOR, },
    ...E2E_ONLY === undefined ? {} : { only: E2E_ONLY, },
    ...E2E_OUT === undefined ? {} : { outputRoot: E2E_OUT, },
    ...E2E_TMP === undefined ? {} : { scratchRoot: E2E_TMP, },
    ...E2E_UNIT_CPU_QUOTA === undefined ? {} : { unitCpuQuota: E2E_UNIT_CPU_QUOTA, },
    ...E2E_UNIT_MEMORY_MAX === undefined ? {} : { unitMemoryMax: E2E_UNIT_MEMORY_MAX, },
    ...E2E_UNIT_TASKS_MAX === undefined ? {} : { unitTasksMax: E2E_UNIT_TASKS_MAX, },
    ...(E2E_GPU === undefined) || (E2E_GPU === '') ? {} : { gpu: E2E_GPU, },
    // Exactly "1": a stray "0", "false" or empty value must never open a window on the desktop.
    ...E2E_VISIBLE === VISIBLE_ENABLED ? { visible: true, } : {},
  };
}

/**
 Reads the run's configuration.
 
 @param appDir - directory of the desktop app, where `target/release/music-player` lives
 
 @param home - home directory of the current user
 
 @param overrides - values that replace the defaults
 
 @param defaultCompositorPath - compositor executable used when the overrides name none; the caller finds the one mise
 has installed (see `compositor-path.ts`), because that needs a command and this function stays free of side effects
 
 @returns the paths and the check filter
 
 @example
 ```ts
 const config = loadRunConfig({ appDir: '/work/app', home: '/home/me', overrides: {}, },);
 ```
 */
export function loadRunConfig(
  {
    appDir,
    home,
    overrides,
    defaultCompositorPath,
  }: {
    readonly appDir: string;
    readonly defaultCompositorPath: string;
    readonly home: string;
    readonly overrides: RunOverrides;
  },
): RunConfig {
  return {
    appDir,
    compositorPath: overrides.compositorPath ?? defaultCompositorPath,
    generatorPath: overrides.generatorPath ?? join(
      home,
      'temp',
      'agent',
      'agent-scale',
      'make-scale-library.ts',
    ),
    gpu: overrides.gpu ?? '',
    visible: overrides.visible ?? false,
    only: overrides.only ?? '',
    outputRoot: overrides.outputRoot ?? join(
      appDir,
      'target',
      'e2e',
    ),
    releaseBinary: join(
      appDir,
      'target',
      'release',
      'music-player',
    ),
    scratchRoot: overrides.scratchRoot ?? join(
      home,
      'temp',
      'agent',
      'music-player-e2e',
    ),
    unitLimits: {
      memoryMax: overrides.unitMemoryMax ?? DEFAULT_UNIT_LIMITS.memoryMax,
      cpuQuota: overrides.unitCpuQuota ?? DEFAULT_UNIT_LIMITS.cpuQuota,
      tasksMax: overrides.unitTasksMax ?? DEFAULT_UNIT_LIMITS.tasksMax,
    },
  };
}
