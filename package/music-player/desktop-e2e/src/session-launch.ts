/**
 What a session is made of before it starts: the unit name, the environment of the app, the `systemd-run` command
 line, and the two guards that keep a run away from processes that are not its own.
 
 Every session runs inside its own transient systemd user unit. The unit holds the compositor and the app, so the unit
 name is the only handle the runner needs: it finds the app by the unit in its control group, and it never signals a
 process by id or by name.
 
 @example
 ```ts
 import { unitNameOf, } from './session-launch.ts';
 
 unitNameOf({ runId: 4242, label: 'wide', },); // 'music-player-e2e-4242-wide'
 ```
 
 @packageDocumentation
 */

import { join, } from 'node:path';

/**
 Prefix of every unit name this runner creates. The refusal to start while another run is active looks for it.
 */
export const UNIT_PREFIX = 'music-player-e2e-';

/**
 Memory ceiling of one session unit.
 */
const SESSION_MEMORY_MAX = '8G';

/**
 Display name to use when the host environment names none.
 */
const FALLBACK_DISPLAY = 'wayland-0';

/**
 Values of the host environment that the session needs.
 */
export type HostEnvironment = {
  readonly path: string;
  readonly runtimeDir: string;
  readonly waylandDisplay: string;
};

/**
 The window and appearance of one session.
 */
export type SessionOptions = {
  readonly accent: string;
  readonly appArgs?: readonly string[];
  readonly extraEnv?: Readonly<Record<string, string>>;
  readonly height: number;
  /**
   True when the launch finds no saved settings at all, so Auto-play on select reads its default, on (D133). Every other
   launch starts with the setting saved off, because most checks click a row and expect it to only select (D124).
   */
  readonly keepSettingsDefaults?: boolean;
  readonly label: string;
  readonly library: string;
  readonly scheme: 'dark' | 'light';
  readonly width: number;
};

/**
 Names the unit of a session.
 
 @param runId - process id of the runner, so two runs never share a unit name
 
 @param label - session label such as `wide`
 
 @returns the unit name without the `.service` suffix
 
 @example
 ```ts
 unitNameOf({ runId: 1, label: 'narrow', },); // 'music-player-e2e-1-narrow'
 ```
 */
export function unitNameOf(
  {
    runId,
    label,
  }: {
    readonly label: string;
    readonly runId: number;
  },
): string {
  return `${UNIT_PREFIX}${runId}-${label}`;
}

/**
 Reads the host values the session needs from the current process.
 
 @param uid - user id, for the default runtime directory
 
 @returns the display, runtime directory and executable search path
 
 @example
 ```ts
 const host = hostEnvironment(1000,);
 ```
 */
export function hostEnvironment(uid: number,): HostEnvironment {
  return {
    path: process.env
      .PATH
      ?? '',
    runtimeDir: process.env
      .XDG_RUNTIME_DIR
      ?? `/run/user/${uid}`,
    waylandDisplay: process.env
      .WAYLAND_DISPLAY
      ?? FALLBACK_DISPLAY,
  };
}

/**
 Builds the environment of the unit: the private sink as the only audio target, scratch XDG directories, and a `bin`
 directory first on the search path, where stubs stand in for the file manager and the clipboard tools.
 
 @param sinkName - name of the private null sink; `PIPEWIRE_NODE` points the app at it
 
 @param xdg - scratch XDG root of the session
 
 @param binDir - directory of the stubs
 
 @param host - host values
 
 @param extraEnv - variables a session adds or replaces
 
 @returns the variables, all text
 
 @example
 ```ts
 const env = sessionEnvironment({ sinkName, xdg, binDir, host, },);
 ```
 */
export function sessionEnvironment(
  {
    sinkName,
    xdg,
    binDir,
    host,
    extraEnv,
  }: {
    readonly binDir: string;
    readonly extraEnv?: Readonly<Record<string, string>>;
    readonly host: HostEnvironment;
    readonly sinkName: string;
    readonly xdg: string;
  },
): Readonly<Record<string, string>> {
  return {
    WAYLAND_DISPLAY: host.waylandDisplay,
    XDG_RUNTIME_DIR: host.runtimeDir,
    PIPEWIRE_NODE: sinkName,
    RUST_LOG: 'info,music_player=debug',
    RUST_BACKTRACE: '1',
    XDG_CONFIG_HOME: join(
      xdg,
      'config',
    ),
    XDG_DATA_HOME: join(
      xdg,
      'data',
    ),
    XDG_CACHE_HOME: join(
      xdg,
      'cache',
    ),
    XDG_STATE_HOME: join(
      xdg,
      'state',
    ),
    PATH: `${binDir}:${host.path}`,
    MUSIC_PLAYER_ACCENT: '',
    MUSIC_PLAYER_COLOR_SCHEME: '',
    ...extraEnv,
  };
}

/**
 Builds the `systemd-run` argument vector that starts the compositor with the app inside a transient user unit whose
 output goes to the session log.
 
 @param unit - unit name
 
 @param logPath - file that receives the unit's standard output and error
 
 @param environment - variables set on the unit
 
 @param compositorPath - compositor executable
 
 @param socket - path of the compositor's control socket
 
 @param options - window and appearance
 
 @param releaseBinary - the app executable
 
 @returns the arguments, in order
 
 @example
 ```ts
 const args = systemdRunArguments({ unit, logPath, environment, compositorPath, socket, options, releaseBinary, },);
 ```
 */
export function systemdRunArguments(
  {
    unit,
    logPath,
    environment,
    compositorPath,
    socket,
    options,
    releaseBinary,
  }: {
    readonly compositorPath: string;
    readonly environment: Readonly<Record<string, string>>;
    readonly logPath: string;
    readonly options: SessionOptions;
    readonly releaseBinary: string;
    readonly socket: string;
    readonly unit: string;
  },
): readonly string[] {
  return [
    '--user',
    `--unit=${unit}`,
    '--collect',
    `--property=MemoryMax=${SESSION_MEMORY_MAX}`,
    `--property=StandardOutput=append:${logPath}`,
    `--property=StandardError=append:${logPath}`,
    ...Object.entries(environment,)
      .map(function setenv([key, value,],): string {
        return `--setenv=${key}=${value}`;
      },),
    compositorPath,
    '--socket',
    socket,
    '--size',
    `${options.width}x${options.height}`,
    '--color-scheme',
    options.scheme,
    '--accent-color',
    options.accent,
    '--',
    releaseBinary,
    ...options.appArgs ?? [options.library,],
  ];
}

/**
 Tells whether a process belongs to a unit, from the text of its `/proc/<pid>/cgroup` file. The unit's service
 path must appear as a whole path component, so a unit named `x-1-wide` is not mistaken for `x-11-wide`.
 
 @param cgroup - content of the process's cgroup file
 
 @param unit - unit name without the `.service` suffix
 
 @returns whether the process runs in that unit's service
 
 @example
 ```ts
 cgroupBelongsToUnit({ cgroup: '0::/user.slice/a-1-wide.service\n', unit: 'a-1-wide', },); // true
 ```
 */
export function cgroupBelongsToUnit(
  {
    cgroup,
    unit,
  }: {
    readonly cgroup: string;
    readonly unit: string;
  },
): boolean {
  return cgroup.includes(`/${unit}.service`,);
}

/**
 Builds the refusal message for a run that finds another run active.
 
 @param activeUnits - output of `systemctl --user list-units` for this runner's unit prefix, active and activating
 units only
 
 @returns the refusal text, or the empty string when no run is active
 
 @example
 ```ts
 runRefusal('',); // ''
 ```
 */
export function runRefusal(activeUnits: string,): string {
  /**
   The listing without surrounding white space.
   */
  const running = activeUnits.trim();
  return running === '' ? '' : `another end-to-end run is active, and this one must run alone:\n${running}`;
}
