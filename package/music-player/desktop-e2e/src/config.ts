/**
 Where the runner finds the app, the compositor and its scratch space, from the environment and the app directory.
 
 Environment variables: `E2E_ONLY` (a check id or group prefix to run alone), `E2E_OUT` (screenshots and summary,
 default `target/e2e` under the app directory), `E2E_TMP` (scratch root, default `temp/agent/music-player-e2e` under the
 home directory), `E2E_LIBRARY_GENERATOR` (the library generator script), `E2E_COMPOSITOR` (the compositor executable,
 default the one mise has installed, found by `compositor-path.ts`).
 
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

/**
 Everything the run needs to know about its surroundings.
 */
export type RunConfig = {
  readonly appDir: string;
  readonly compositorPath: string;
  readonly generatorPath: string;
  readonly only: string;
  readonly outputRoot: string;
  readonly releaseBinary: string;
  readonly scratchRoot: string;
};

/**
 Values that replace the defaults, each present only when the user set it.
 */
export type RunOverrides = {
  readonly compositorPath?: string;
  readonly generatorPath?: string;
  readonly only?: string;
  readonly outputRoot?: string;
  readonly scratchRoot?: string;
};

/**
 Reads the overrides from the environment variables `E2E_ONLY`, `E2E_OUT`, `E2E_TMP`, `E2E_LIBRARY_GENERATOR` and
 `E2E_COMPOSITOR`.
 
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
    E2E_LIBRARY_GENERATOR,
    E2E_ONLY,
    E2E_OUT,
    E2E_TMP,
  } = process.env;
  return {
    ...E2E_COMPOSITOR === undefined ? {} : { compositorPath: E2E_COMPOSITOR, },
    ...E2E_LIBRARY_GENERATOR === undefined ? {} : { generatorPath: E2E_LIBRARY_GENERATOR, },
    ...E2E_ONLY === undefined ? {} : { only: E2E_ONLY, },
    ...E2E_OUT === undefined ? {} : { outputRoot: E2E_OUT, },
    ...E2E_TMP === undefined ? {} : { scratchRoot: E2E_TMP, },
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
  };
}
