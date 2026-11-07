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
 Memory ceiling, as a systemd `MemoryMax` value, of one session unit, which holds the compositor and the app.

 Measured on 2026-11-07 by sampling every `music-player-e2e-*` unit of this runner every 200 ms during a full run
 (122 checks passed, 209 s, one unrelated Unicode-typing check failed) and a `seek` run: the widest `memory.peak` of
 any unit was 411123712 bytes (the `wide` session of the `seek` run), and the app process alone peaked at 138828 KiB
 resident. 2G is about five times the widest peak. The value before this measurement, 8G, had no record. Raise it with
 `E2E_UNIT_MEMORY_MAX` when a unit is killed for memory.
 */
export const SESSION_MEMORY_MAX = '2G';

/**
 CPU ceiling, as a systemd `CPUQuota` value, of one session unit; `400%` is four hardware threads.

 The widest CPU time of any unit in the measurement of `SESSION_MEMORY_MAX` was 3.3 s over a session that lasted
 several seconds (`usage_usec` 3286059 for the `wide` session of the full run), far under one thread on average, so four
 threads leave room for the compositor's render and encoder threads without letting a runaway app take the host. Raise
 it with `E2E_UNIT_CPU_QUOTA`.
 */
export const SESSION_CPU_QUOTA = '400%';

/**
 Task limit, as a systemd `TasksMax` value, of one session unit. A control group counts threads as well as processes.

 The widest `pids.peak` of any unit in the measurement of `SESSION_MEMORY_MAX` was 88 (the `visual-light` session), and
 the app alone ran 66 threads. 512 is a power of two over five times that, because a start that hits the limit fails
 with a bare `EAGAIN` that names no limit. Raise it with `E2E_UNIT_TASKS_MAX`.
 */
export const SESSION_TASKS_MAX = '512';

/**
 Disk weight, as a systemd `IOWeight` value, of one session unit. The owner's rule (`CLAUDE.md`, rule RXS) is never
 above 10; a unit without the property gets systemd's default of 100. No option raises it.
 */
export const SESSION_IO_WEIGHT = 10;

/**
 The memory, CPU and task limits of one session unit, each a systemd value.
 */
export type UnitLimits = {
  readonly cpuQuota: string;
  readonly memoryMax: string;
  readonly tasksMax: string;
};

/**
 The limits a session unit gets when the run names none.
 */
export const DEFAULT_UNIT_LIMITS: UnitLimits = {
  memoryMax: SESSION_MEMORY_MAX,
  cpuQuota: SESSION_CPU_QUOTA,
  tasksMax: SESSION_TASKS_MAX,
};

/**
 Display name to use when the host environment names none.
 */
const FALLBACK_DISPLAY = 'wayland-0';

/**
 Where the compositor of a session draws.

 `headless` renders without any display, on the named render node (or `software`), so nothing appears on the desktop
 of the machine. `visible` opens a nested window in the host session, and only an explicit request gives it
 (`--visible` or `E2E_VISIBLE=1`, see `config.ts`).
 */
export type DisplayMode = HeadlessDisplay | VisibleDisplay;

/**
 A session that renders without any display, on one device.
 */
export type HeadlessDisplay = {
  /**
   Render node path such as `/dev/dri/renderD129`, or `software`.
   */
  readonly gpu: string;
  readonly kind: 'headless';
};

/**
 A session that opens a nested window in the host session.
 */
export type VisibleDisplay = {
  readonly kind: 'visible';
};

/**
 The variables through which a process finds a display of the host. A headless session gets none of them: the unit
 environment leaves them out and `UnsetEnvironment` removes the copies the systemd user manager holds, so even a
 compositor started without `--headless` finds no parent display and refuses to start instead of opening a window.
 */
export const HOST_DISPLAY_VARIABLES: readonly string[] = [
  'WAYLAND_DISPLAY',
  'WAYLAND_SOCKET',
  'DISPLAY',
  'XAUTHORITY',
];

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
 
 @param display - where the compositor draws; only a visible session is told the host display
 
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
    display,
    sinkName,
    xdg,
    binDir,
    host,
    extraEnv,
  }: {
    readonly binDir: string;
    readonly display: DisplayMode;
    readonly extraEnv?: Readonly<Record<string, string>>;
    readonly host: HostEnvironment;
    readonly sinkName: string;
    readonly xdg: string;
  },
): Readonly<Record<string, string>> {
  /**
   The variables of the unit, before a headless session loses the display ones.
   */
  const environment = {
    ...display.kind === 'visible' ? { WAYLAND_DISPLAY: host.waylandDisplay, } : {},
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
  return display.kind === 'visible' ? environment : withoutHostDisplay(environment,);
}

/**
 Drops every variable through which a display of the host is found.

 @param environment - variables of a unit

 @returns the same variables without the ones in `HOST_DISPLAY_VARIABLES`

 @example
 ```ts
 withoutHostDisplay({ DISPLAY: ':0', PATH: '/bin', },); // { PATH: '/bin' }
 ```
 */
function withoutHostDisplay(environment: Readonly<Record<string, string>>,): Readonly<Record<string, string>> {
  return Object.fromEntries(Object.entries(environment,)
    .filter(function isNotHostDisplay([name,]: EnvironmentEntry,): boolean {
      return !HOST_DISPLAY_VARIABLES.includes(name,);
    },),);
}

/**
 One variable of the unit's environment as `Object.entries` gives it: the name, then the value.
 */
type EnvironmentEntry = readonly [
  name: string,
  value: string,
];

/**
 Builds the `systemd-run` argument vector that starts the compositor with the app inside a transient user unit whose
 output goes to the session log.
 
 @param display - where the compositor draws: a headless session gets `--headless --gpu <device>` and no host display
 variable; a visible one gets neither
 
 @param unit - unit name
 
 @param logPath - file that receives the unit's standard output and error
 
 @param environment - variables set on the unit
 
 @param compositorPath - compositor executable
 
 @param socket - path of the compositor's control socket
 
 @param options - window and appearance
 
 @param releaseBinary - the app executable
 
 @param limits - memory, CPU and task limits of the unit; the disk weight is fixed
 
 @returns the arguments, in order
 
 @example
 ```ts
 const args = systemdRunArguments({ display, unit, logPath, environment, compositorPath, socket, options, releaseBinary, limits, },);
 ```
 */
export function systemdRunArguments(
  {
    display,
    unit,
    logPath,
    environment,
    compositorPath,
    socket,
    options,
    releaseBinary,
    limits,
  }: {
    readonly compositorPath: string;
    readonly display: DisplayMode;
    readonly environment: Readonly<Record<string, string>>;
    readonly limits: UnitLimits;
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
    `--property=MemoryMax=${limits.memoryMax}`,
    `--property=CPUQuota=${limits.cpuQuota}`,
    `--property=TasksMax=${limits.tasksMax}`,
    `--property=IOWeight=${String(SESSION_IO_WEIGHT,)}`,
    `--property=StandardOutput=append:${logPath}`,
    `--property=StandardError=append:${logPath}`,
    ...display.kind === 'headless'
      ? [`--property=UnsetEnvironment=${HOST_DISPLAY_VARIABLES.join(' ',)}`,]
      : [],
    ...Object.entries(environment,)
      .map(function setenv([key, value,]: EnvironmentEntry,): string {
        return `--setenv=${key}=${value}`;
      },),
    compositorPath,
    ...display.kind === 'headless'
      ? [
        '--headless',
        '--gpu',
        display.gpu,
      ]
      : [],
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
 Builds the whole `systemd-run` argument vector of a session: its environment, then the command line that starts the
 compositor and the app. `startSession` runs exactly this vector, so a test of it covers what the host gets.

 @param display - where the compositor draws
 
 @param unit - unit name
 
 @param logPath - file that receives the unit's standard output and error
 
 @param sinkName - name of the private null sink the app is pointed at
 
 @param xdg - scratch XDG root of the session
 
 @param binDir - directory of the stubs
 
 @param host - host values
 
 @param extraEnv - variables a session adds or replaces
 
 @param compositorPath - compositor executable
 
 @param socket - path of the compositor's control socket
 
 @param options - window and appearance
 
 @param releaseBinary - the app executable
 
 @param limits - memory, CPU and task limits of the unit
 
 @returns the arguments after `systemd-run`, in order
 
 @example
 ```ts
 const args = sessionLaunchArguments({ display, unit, logPath, sinkName, xdg, binDir, host, compositorPath, socket, options, releaseBinary, limits, },);
 ```
 */
export function sessionLaunchArguments(
  {
    display,
    unit,
    logPath,
    sinkName,
    xdg,
    binDir,
    host,
    extraEnv,
    compositorPath,
    socket,
    options,
    releaseBinary,
    limits,
  }: {
    readonly binDir: string;
    readonly compositorPath: string;
    readonly display: DisplayMode;
    readonly extraEnv?: Readonly<Record<string, string>>;
    readonly host: HostEnvironment;
    readonly limits: UnitLimits;
    readonly logPath: string;
    readonly options: SessionOptions;
    readonly releaseBinary: string;
    readonly sinkName: string;
    readonly socket: string;
    readonly unit: string;
    readonly xdg: string;
  },
): readonly string[] {
  return systemdRunArguments({
    display,
    unit,
    logPath,
    environment: sessionEnvironment({
      display,
      sinkName,
      xdg,
      binDir,
      host,
      ...extraEnv === undefined ? {} : { extraEnv, },
    },),
    compositorPath,
    socket,
    options,
    releaseBinary,
    limits,
  },);
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
